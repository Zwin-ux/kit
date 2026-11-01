//! The session view's state (PRD §7.1): a pure reducer over agent events and
//! keys. Finished output is queued as [`Out`] lines for the driver to print
//! once into scrollback; everything still changing lives in the live region.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use kit_loop::{AgentEvent, Outcome};
use serde_json::Value;
use std::time::{Duration, Instant};

/// A finished line, printed once to the terminal's scrollback and never edited.
#[derive(Debug, Clone, PartialEq)]
pub enum Out {
    /// The user's prompt.
    You(String),
    /// One line of model text; `first` carries the `kit ›` prefix.
    Kit { first: bool, text: String },
    /// One tool call.
    Card(Card),
    /// A warning, retry or loop notice.
    Notice(String),
    /// Plain dim text: the welcome, help, how a turn ended.
    Dim(String),
    /// An empty line between turns.
    Blank,
}

/// One numbered tool card: verb, target, outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    pub n: u32,
    pub verb: String,
    pub target: String,
    pub outcome: CardOutcome,
    /// Size, count, duration, or the first error line.
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardOutcome {
    Ok,
    Failed,
    Denied,
}

/// What the agent is doing, shown as a spinner and a word.
#[derive(Debug, Clone, PartialEq)]
pub enum Busy {
    Thinking,
    Running { tool: String, target: String },
    Asking,
}

impl Busy {
    pub fn word(&self) -> String {
        match self {
            Self::Thinking => "thinking".into(),
            Self::Running { tool, target } if target.is_empty() => format!("running {tool}"),
            Self::Running { tool, target } => format!("running {tool} {target}"),
            Self::Asking => "waiting for you".into(),
        }
    }
}

/// A boundary prompt waiting for `y`, `a` or `n`.
#[derive(Debug, Clone, PartialEq)]
pub struct Approval {
    pub tool: String,
    pub target: String,
    pub reason: String,
}

/// The user's answer to an [`Approval`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    Once,
    /// Allow this exact call again for the rest of the session.
    Always,
    Deny,
}

/// What a key asks the driver to do.
#[derive(Debug, Clone, PartialEq)]
pub enum Cmd {
    Send(String),
    Interrupt,
    Answer(Answer),
    Quit,
}

/// The status bar's fields, left to right; the right ones drop first.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Status {
    pub model: String,
    /// Prompt tokens in the last reply: how full the context is.
    pub ctx: u64,
    /// Session cost, e.g. `≈$0.041`.
    pub cost: String,
    /// `workspace`, `read`, `full`; `unsandboxed` is appended by the view.
    pub profile: String,
    pub branch: Option<String>,
}

/// The composer: one text buffer with a cursor (a byte offset on a char boundary).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Composer {
    pub text: String,
    pub cursor: usize,
}

impl Composer {
    fn insert(&mut self, c: char) {
        self.text.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }
    fn backspace(&mut self) {
        if let Some(c) = self.text[..self.cursor].chars().next_back() {
            self.cursor -= c.len_utf8();
            self.text.remove(self.cursor);
        }
    }
    fn delete(&mut self) {
        if self.cursor < self.text.len() {
            self.text.remove(self.cursor);
        }
    }
    fn left(&mut self) {
        if let Some(c) = self.text[..self.cursor].chars().next_back() {
            self.cursor -= c.len_utf8();
        }
    }
    fn right(&mut self) {
        if let Some(c) = self.text[self.cursor..].chars().next() {
            self.cursor += c.len_utf8();
        }
    }
    fn set(&mut self, text: String) {
        self.cursor = text.len();
        self.text = text;
    }
    fn take(&mut self) -> String {
        self.cursor = 0;
        std::mem::take(&mut self.text)
    }
}

/// Tool calls started but not finished.
#[derive(Debug, Clone)]
struct Pending {
    id: String,
    name: String,
    target: String,
    started: Instant,
}

/// The whole session view.
#[derive(Debug)]
pub struct Session {
    pub status: Status,
    pub composer: Composer,
    pub busy: Option<Busy>,
    pub busy_since: Option<Instant>,
    pub approval: Option<Approval>,
    /// A one-line hint in the live region, cleared by the next key.
    pub flash: Option<String>,
    /// Model text after the last newline: still streaming.
    pub partial: String,
    pub kit_started: bool,
    pending: Vec<Pending>,
    cards: u32,
    out: Vec<Out>,
    history: Vec<String>,
    recall: Option<usize>,
    quit_armed: bool,
}

pub const HELP: &[&str] = &[
    "Enter send · Ctrl-J newline · Esc interrupt · Ctrl-C twice quit",
    "Approvals: y once · a always this session · n deny",
    "/help this list · /quit leave · Up recalls your last prompt",
];

impl Session {
    pub fn new(status: Status) -> Self {
        Self {
            status,
            composer: Composer::default(),
            busy: None,
            busy_since: None,
            approval: None,
            flash: None,
            partial: String::new(),
            kit_started: false,
            pending: vec![],
            cards: 0,
            out: vec![],
            history: vec![],
            recall: None,
            quit_armed: false,
        }
    }

    /// Lines ready for scrollback, oldest first.
    pub fn drain_out(&mut self) -> Vec<Out> {
        std::mem::take(&mut self.out)
    }

    pub fn push(&mut self, out: Out) {
        self.out.push(out);
    }

    fn set_busy(&mut self, busy: Option<Busy>, now: Instant) {
        if busy.is_some() && self.busy.is_none() {
            self.busy_since = Some(now);
        }
        if busy.is_none() {
            self.busy_since = None;
        }
        self.busy = busy;
    }

    fn flush_partial(&mut self) {
        if !self.partial.trim().is_empty() {
            let text = std::mem::take(&mut self.partial);
            self.kit_line(text);
        }
        self.partial.clear();
    }

    fn kit_line(&mut self, text: String) {
        let first = !self.kit_started;
        self.kit_started = true;
        self.out.push(Out::Kit { first, text });
    }

    /// Apply one event from the loop.
    pub fn on_agent(&mut self, e: AgentEvent, now: Instant) {
        match e {
            AgentEvent::Prompt { .. } => {}
            AgentEvent::Text { text } => {
                self.partial.push_str(&text);
                while let Some(i) = self.partial.find('\n') {
                    let line: String = self.partial.drain(..=i).collect();
                    let line = line.trim_end_matches(['\n', '\r']).to_owned();
                    if !(line.is_empty() && !self.kit_started) {
                        self.kit_line(line);
                    }
                }
            }
            AgentEvent::Thinking { .. } => {
                if self.pending.is_empty() {
                    self.set_busy(Some(Busy::Thinking), now);
                }
            }
            AgentEvent::ToolStart { id, name, args } => {
                self.flush_partial();
                let target = target(&args);
                self.set_busy(
                    Some(Busy::Running {
                        tool: name.clone(),
                        target: target.clone(),
                    }),
                    now,
                );
                self.pending.push(Pending {
                    id,
                    name,
                    target,
                    started: now,
                });
            }
            AgentEvent::ToolEnd {
                id,
                name,
                content,
                is_error,
                denied,
            } => {
                let (target, elapsed) = match self.pending.iter().position(|p| p.id == id) {
                    Some(i) => {
                        let p = self.pending.remove(i);
                        (p.target, now.saturating_duration_since(p.started))
                    }
                    None => (String::new(), Duration::ZERO),
                };
                self.cards += 1;
                let card = card(
                    self.cards, &name, target, &content, is_error, denied, elapsed,
                );
                self.out.push(Out::Card(card));
                let next = match self.pending.last() {
                    Some(p) => Busy::Running {
                        tool: p.name.clone(),
                        target: p.target.clone(),
                    },
                    None => Busy::Thinking,
                };
                self.set_busy(Some(next), now);
            }
            AgentEvent::Usage { usage, total, .. } => {
                self.status.ctx = usage.input + usage.cached_input + usage.cache_write;
                self.status.cost = total;
            }
            AgentEvent::Notice { text } => {
                self.flush_partial();
                self.out.push(Out::Notice(text));
            }
        }
    }

    /// The loop asked before crossing a boundary.
    pub fn ask(&mut self, approval: Approval, now: Instant) {
        self.flush_partial();
        self.approval = Some(approval);
        self.set_busy(Some(Busy::Asking), now);
    }

    /// The turn is over: print what is left and how it ended.
    pub fn turn_ended(&mut self, outcome: &Outcome) {
        self.flush_partial();
        self.pending.clear();
        self.approval = None;
        self.busy = None;
        self.busy_since = None;
        let why = match outcome {
            Outcome::Done => None,
            Outcome::Budget => Some("stopped: the budget is spent".to_owned()),
            Outcome::MaxTurns => Some("stopped: the reply limit was reached".to_owned()),
            Outcome::Interrupted => Some("interrupted".to_owned()),
            Outcome::Looping => Some("stopped: the agent was repeating itself".to_owned()),
            Outcome::Refused => Some("the model refused".to_owned()),
            Outcome::Error(e) => Some(format!("error: {e}")),
        };
        if let Some(why) = why {
            self.out.push(Out::Notice(why));
        }
        self.kit_started = false;
        self.out.push(Out::Blank);
    }

    /// Apply one key. Letter keys only ever type into the composer, except
    /// while an approval is waiting.
    pub fn on_key(&mut self, key: KeyEvent, now: Instant) -> Option<Cmd> {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        if ctrl && key.code == KeyCode::Char('c') {
            if self.quit_armed {
                return Some(Cmd::Quit);
            }
            self.quit_armed = true;
            self.flash = Some("Press Ctrl-C again to quit".into());
            return self.busy.is_some().then_some(Cmd::Interrupt);
        }
        self.quit_armed = false;
        self.flash = None;

        if self.approval.is_some() {
            let answer = match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => Answer::Once,
                KeyCode::Char('a') | KeyCode::Char('A') => Answer::Always,
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Answer::Deny,
                _ => {
                    self.flash = Some("y once · a always · n deny".into());
                    return None;
                }
            };
            self.approval = None;
            self.set_busy(Some(Busy::Thinking), now);
            return Some(Cmd::Answer(answer));
        }

        match key.code {
            KeyCode::Esc if self.busy.is_some() => return Some(Cmd::Interrupt),
            KeyCode::Esc => {
                self.composer.take();
            }
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.composer.insert('\n')
            }
            KeyCode::Char('j') if ctrl => self.composer.insert('\n'),
            KeyCode::Enter => return self.submit(now),
            KeyCode::Up if self.composer.text.is_empty() || self.recall.is_some() => {
                self.recall_prev()
            }
            KeyCode::Down if self.recall.is_some() => self.recall_next(),
            KeyCode::Backspace => self.composer.backspace(),
            KeyCode::Delete => self.composer.delete(),
            KeyCode::Left => self.composer.left(),
            KeyCode::Right => self.composer.right(),
            KeyCode::Home => self.composer.cursor = 0,
            KeyCode::End => self.composer.cursor = self.composer.text.len(),
            KeyCode::Char('?') if self.composer.text.is_empty() => self.help(),
            KeyCode::Char('u') if ctrl => {
                self.composer.take();
            }
            KeyCode::Char(c) if !ctrl => {
                self.recall = None;
                self.composer.insert(c);
            }
            _ => {}
        }
        None
    }

    /// Pasted text goes into the composer as typed, newlines included.
    pub fn on_paste(&mut self, text: &str) {
        for c in text.chars().filter(|c| *c != '\r') {
            self.composer.insert(c);
        }
    }

    fn submit(&mut self, now: Instant) -> Option<Cmd> {
        let text = self.composer.text.trim().to_owned();
        if text.is_empty() {
            return None;
        }
        if let Some(command) = text.strip_prefix('/') {
            self.composer.take();
            return match command.split_whitespace().next().unwrap_or("") {
                "quit" | "exit" | "q" => Some(Cmd::Quit),
                "help" => {
                    self.help();
                    None
                }
                other => {
                    self.flash = Some(format!("/{other} is not here yet. /help lists what is"));
                    None
                }
            };
        }
        if self.busy.is_some() {
            self.flash = Some("Kit is still working. Esc interrupts".into());
            return None;
        }
        self.composer.take();
        self.recall = None;
        if self.history.last() != Some(&text) {
            self.history.push(text.clone());
        }
        self.out.push(Out::You(text.clone()));
        self.set_busy(Some(Busy::Thinking), now);
        Some(Cmd::Send(text))
    }

    fn help(&mut self) {
        for line in HELP {
            self.out.push(Out::Dim((*line).into()));
        }
    }

    fn recall_prev(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let i = match self.recall {
            Some(0) => 0,
            Some(i) => i - 1,
            None => self.history.len() - 1,
        };
        self.recall = Some(i);
        self.composer.set(self.history[i].clone());
    }

    fn recall_next(&mut self) {
        match self.recall {
            Some(i) if i + 1 < self.history.len() => {
                self.recall = Some(i + 1);
                self.composer.set(self.history[i + 1].clone());
            }
            _ => {
                self.recall = None;
                self.composer.take();
            }
        }
    }
}

/// What a card names: the path, command or pattern, first line only.
pub fn target(args: &Value) -> String {
    let t = ["file_path", "command", "pattern", "path", "name"]
        .iter()
        .find_map(|k| args[*k].as_str())
        .unwrap_or("");
    t.lines().next().unwrap_or("").to_owned()
}

fn card(
    n: u32,
    name: &str,
    target: String,
    content: &str,
    is_error: bool,
    denied: bool,
    elapsed: Duration,
) -> Card {
    let first = content
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .trim();
    let lines = content.lines().filter(|l| !l.trim().is_empty()).count();
    let (outcome, detail) = if denied {
        (CardOutcome::Denied, "denied".to_owned())
    } else if is_error {
        (CardOutcome::Failed, first.to_owned())
    } else {
        let detail = match name {
            "Read" => plural(lines, "line"),
            "Grep" if first.starts_with("No ") => "no matches".into(),
            "Grep" => plural(lines, "match"),
            "Glob" if first.starts_with("No ") => "no files".into(),
            "Glob" => plural(lines, "file"),
            "Bash" => format!("ok {}", seconds(elapsed)),
            "Edit" => "edited".into(),
            "Write" => "written".into(),
            _ => first.to_owned(),
        };
        (CardOutcome::Ok, detail)
    };
    Card {
        n,
        verb: name.to_owned(),
        target,
        outcome,
        detail,
    }
}

fn plural(n: usize, word: &str) -> String {
    match (n, word) {
        (1, _) => format!("1 {word}"),
        (_, "match") => format!("{n} matches"),
        _ => format!("{n} {word}s"),
    }
}

/// `2.1s`, or `1m 04s` past a minute.
pub fn seconds(d: Duration) -> String {
    let s = d.as_secs_f64();
    if s < 60.0 {
        format!("{s:.1}s")
    } else {
        format!("{}m {:02}s", d.as_secs() / 60, d.as_secs() % 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kit_providers::Usage;
    use serde_json::json;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }
    fn ctrl(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }
    fn typed(s: &mut Session, text: &str) {
        for c in text.chars() {
            s.on_key(key(KeyCode::Char(c)), Instant::now());
        }
    }

    #[test]
    fn enter_sends_and_echoes_the_prompt() {
        let mut s = Session::new(Status::default());
        typed(&mut s, "fix the test");
        let cmd = s.on_key(key(KeyCode::Enter), Instant::now());
        assert_eq!(cmd, Some(Cmd::Send("fix the test".into())));
        assert_eq!(s.drain_out(), vec![Out::You("fix the test".into())]);
        assert_eq!(s.busy, Some(Busy::Thinking));
        assert!(s.composer.text.is_empty());
    }

    #[test]
    fn streaming_text_prints_whole_lines_and_keeps_the_rest_live() {
        let mut s = Session::new(Status::default());
        let now = Instant::now();
        s.on_agent(
            AgentEvent::Text {
                text: "Reading the ".into(),
            },
            now,
        );
        assert!(s.drain_out().is_empty());
        s.on_agent(
            AgentEvent::Text {
                text: "test first.\nThen ".into(),
            },
            now,
        );
        assert_eq!(
            s.drain_out(),
            vec![Out::Kit {
                first: true,
                text: "Reading the test first.".into()
            }]
        );
        assert_eq!(s.partial, "Then ");
        s.turn_ended(&Outcome::Done);
        assert_eq!(
            s.drain_out(),
            vec![
                Out::Kit {
                    first: false,
                    text: "Then ".into()
                },
                Out::Blank
            ]
        );
    }

    #[test]
    fn a_tool_call_becomes_one_numbered_card() {
        let mut s = Session::new(Status::default());
        let t0 = Instant::now();
        s.on_agent(
            AgentEvent::ToolStart {
                id: "1".into(),
                name: "Bash".into(),
                args: json!({"command": "cargo test -p auth"}),
            },
            t0,
        );
        assert_eq!(
            s.busy.as_ref().unwrap().word(),
            "running Bash cargo test -p auth"
        );
        let t1 = t0 + Duration::from_millis(2100);
        s.on_agent(
            AgentEvent::ToolEnd {
                id: "1".into(),
                name: "Bash".into(),
                content: "ok".into(),
                is_error: false,
                denied: false,
            },
            t1,
        );
        let Out::Card(card) = &s.drain_out()[0] else {
            panic!("card")
        };
        assert_eq!(
            (
                card.n,
                card.verb.as_str(),
                card.target.as_str(),
                card.detail.as_str()
            ),
            (1, "Bash", "cargo test -p auth", "ok 2.1s")
        );
        assert_eq!(s.busy, Some(Busy::Thinking));
    }

    #[test]
    fn failed_and_denied_cards_say_so() {
        let mut s = Session::new(Status::default());
        let now = Instant::now();
        s.on_agent(
            AgentEvent::ToolEnd {
                id: "x".into(),
                name: "Edit".into(),
                content: "\nold_string not found".into(),
                is_error: true,
                denied: false,
            },
            now,
        );
        s.on_agent(
            AgentEvent::ToolEnd {
                id: "y".into(),
                name: "Bash".into(),
                content: "git push needs approval".into(),
                is_error: true,
                denied: true,
            },
            now,
        );
        let out = s.drain_out();
        assert!(
            matches!(&out[0], Out::Card(c) if c.outcome == CardOutcome::Failed && c.detail == "old_string not found")
        );
        assert!(matches!(&out[1], Out::Card(c) if c.outcome == CardOutcome::Denied && c.n == 2));
    }

    #[test]
    fn approvals_take_y_a_n_and_nothing_else() {
        let mut s = Session::new(Status::default());
        let now = Instant::now();
        s.ask(
            Approval {
                tool: "Bash".into(),
                target: "git push".into(),
                reason: "network".into(),
            },
            now,
        );
        assert_eq!(s.on_key(key(KeyCode::Char('x')), now), None);
        assert!(s.approval.is_some(), "other letters do not answer or type");
        assert!(s.composer.text.is_empty());
        assert_eq!(
            s.on_key(key(KeyCode::Char('a')), now),
            Some(Cmd::Answer(Answer::Always))
        );
        assert!(s.approval.is_none());
        s.ask(
            Approval {
                tool: "Bash".into(),
                target: "rm -rf build".into(),
                reason: "".into(),
            },
            now,
        );
        assert_eq!(
            s.on_key(key(KeyCode::Esc), now),
            Some(Cmd::Answer(Answer::Deny))
        );
    }

    #[test]
    fn esc_interrupts_a_turn_and_ctrl_c_twice_quits() {
        let mut s = Session::new(Status::default());
        let now = Instant::now();
        typed(&mut s, "go");
        s.on_key(key(KeyCode::Enter), now);
        assert_eq!(s.on_key(key(KeyCode::Esc), now), Some(Cmd::Interrupt));
        s.turn_ended(&Outcome::Interrupted);
        assert!(s.drain_out().contains(&Out::Notice("interrupted".into())));
        assert_eq!(s.on_key(ctrl('c'), now), None);
        assert_eq!(s.flash.as_deref(), Some("Press Ctrl-C again to quit"));
        assert_eq!(s.on_key(ctrl('c'), now), Some(Cmd::Quit));
    }

    #[test]
    fn enter_while_busy_keeps_the_draft() {
        let mut s = Session::new(Status::default());
        let now = Instant::now();
        typed(&mut s, "one");
        s.on_key(key(KeyCode::Enter), now);
        typed(&mut s, "two");
        assert_eq!(s.on_key(key(KeyCode::Enter), now), None);
        assert_eq!(s.composer.text, "two");
    }

    #[test]
    fn up_recalls_history_and_ctrl_j_adds_a_newline() {
        let mut s = Session::new(Status::default());
        let now = Instant::now();
        typed(&mut s, "first");
        s.on_key(key(KeyCode::Enter), now);
        s.turn_ended(&Outcome::Done);
        s.on_key(key(KeyCode::Up), now);
        assert_eq!(s.composer.text, "first");
        s.on_key(key(KeyCode::Down), now);
        assert!(s.composer.text.is_empty());
        typed(&mut s, "a");
        s.on_key(ctrl('j'), now);
        typed(&mut s, "b");
        assert_eq!(s.composer.text, "a\nb");
    }

    #[test]
    fn usage_updates_the_status_bar() {
        let mut s = Session::new(Status::default());
        let usage = Usage {
            input: 1200,
            cached_input: 17_000,
            output: 300,
            ..Usage::default()
        };
        s.on_agent(
            AgentEvent::Usage {
                usage,
                cost_usd: Some(0.04),
                total: "≈$0.041".into(),
            },
            Instant::now(),
        );
        assert_eq!((s.status.ctx, s.status.cost.as_str()), (18_200, "≈$0.041"));
    }

    #[test]
    fn slash_commands() {
        let mut s = Session::new(Status::default());
        let now = Instant::now();
        typed(&mut s, "/help");
        assert_eq!(s.on_key(key(KeyCode::Enter), now), None);
        assert_eq!(s.drain_out().len(), HELP.len());
        typed(&mut s, "/quit");
        assert_eq!(s.on_key(key(KeyCode::Enter), now), Some(Cmd::Quit));
    }
}
