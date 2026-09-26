//! The `kit` command grammar: one clap type per verb.
//!
//! Every flag users may depend on is declared here, so `--help`, errors,
//! "did you mean" and shell completions all come from the same place.

use clap::{Args, Parser, Subcommand, ValueEnum};
use kit_core::AgentKind;
use std::path::PathBuf;
use std::time::Duration;

const AFTER_HELP: &str = "\
Start here:
  kit setup                    Pick your agents and a focus, then install its kit
  kit show                     See the kits you can add
  kit add frontend-design      Add a kit to this repo (shows every file first)

Then prove the work:
  kit run \"add a test for X\"   Run an agent in its own worktree, then the checks
  kit land <id>                Put a proven run's changes on a new branch

Docs: https://github.com/Zwin-ux/kit#readme";

/// Set your coding agents up for one job, then prove what they do.
///
/// The first time, bare `kit` asks which agents and which focus, then
/// installs that kit. After that it opens the Control Room: dispatch many
/// runs and watch them in one place.
#[derive(Debug, Parser)]
#[command(
    name = "kit",
    version,
    after_help = AFTER_HELP
)]
pub struct Cli {
    /// Open the Control Room with sample runs (no agent is started)
    #[arg(long)]
    pub demo: bool,

    #[command(flatten)]
    pub global: GlobalArgs,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Args)]
pub struct GlobalArgs {
    /// Run as if kit was started in DIR (like git -C)
    #[arg(short = 'C', long = "repo", global = true, value_name = "DIR")]
    pub dir: Option<PathBuf>,

    /// Print one JSON result on stdout, errors included
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// List kits, or show what one kit installs
    #[command(
        after_help = "Example:\n  kit show                   list the kits\n  kit show frontend-design   skills, rules, MCP servers and hooks it brings"
    )]
    Show {
        /// Kit name, or a folder with a KIT.toml
        #[arg(value_name = "KIT")]
        kit: Option<String>,
    },

    /// Set your coding agents up for a job: pick agents, focus and scope
    #[command(
        after_help = "Bare `kit` runs this the first time. Every question is also a flag.\n\nExample:\n  kit setup\n  kit setup --agent claude --kit frontend-design --global --yes"
    )]
    Setup(SetupArgs),

    /// Install kits into your coding agents
    #[command(
        after_help = "Shows every file, key and command first, and asks before writing.\n\nExample:\n  kit add frontend-design --global           every project, every agent found\n  kit add llm-engineer --agent codex         this repo, Codex only\n  kit add backend-engineer --print           show the plan, write nothing"
    )]
    Add(AddArgs),

    /// Remove installed kits, exactly as they were added
    #[command(after_help = "Example:\n  kit remove frontend-design --global")]
    Remove(RemoveArgs),

    /// Installed kits, and anything changed by hand since
    List(ListKitsArgs),

    /// Called by agent hooks that kits install
    #[command(hide = true)]
    Hook {
        #[command(subcommand)]
        event: HookCommand,
    },

    /// Write kit.toml: the checks every run must pass
    #[command(
        after_help = "Example:\n  kit init            detect, check and write\n  kit init --print    show the proposal, write nothing"
    )]
    Init(InitArgs),

    /// Run one agent on one task in its own worktree, then the checks
    #[command(
        after_help = "Example:\n  kit run \"add a test for parse_duration\"\n  kit run --agent codex \"fix the failing test\""
    )]
    Run(RunArgs),

    /// Put a proven run's changes on a new branch
    #[command(after_help = "Example:\n  kit land 01M3F4517VJ7")]
    Land(LandArgs),

    /// List runs, or show one run's proof
    Receipt(ReceiptArgs),

    /// Check which agents are ready and whether this repo has a gate
    #[command(
        after_help = "Checks every kit installed for all your projects and in this repo, so it has no --global.\n\nExample:\n  kit doctor\n  kit doctor --start-mcp"
    )]
    Doctor(DoctorArgs),

    /// Print a shell completion script
    #[command(after_help = "Example:\n  kit completions zsh > ~/.zfunc/_kit")]
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Print the proposed kit.toml. Write and run nothing
    #[arg(short, long)]
    pub print: bool,

    /// Replace an existing kit.toml
    #[arg(short, long)]
    pub force: bool,

    /// Write the gate without running each check once first
    #[arg(long, conflicts_with = "drop_failing")]
    pub no_check: bool,

    /// Checking is the default now; kept so old scripts still parse
    #[arg(long, hide = true)]
    pub check: bool,

    /// Write the gate without the checks that fail today
    #[arg(long)]
    pub drop_failing: bool,

    /// Time limit for each check while checking
    #[arg(long, value_name = "DURATION", default_value = "5m", value_parser = parse_duration)]
    pub timeout: Duration,
}

#[derive(Debug, Args)]
pub struct RunArgs {
    /// What the agent should do
    #[arg(value_name = "TASK", required_unless_present = "task_flag")]
    pub task: Option<String>,

    /// The task, as a flag (same as the positional TASK)
    #[arg(short = 't', long = "task", hide = true, conflicts_with = "task")]
    pub task_flag: Option<String>,

    /// Agent to run [default: the first ready of claude, codex, grok, ollama]
    #[arg(short, long, value_enum)]
    pub agent: Option<AgentArg>,

    /// Test the pipeline without an agent. Proves nothing
    #[arg(long)]
    pub dry_run: bool,

    /// Exit 0 even when kit.toml has no checks
    #[arg(long)]
    pub allow_vacuous: bool,
}

impl RunArgs {
    /// The task from TASK or `--task`.
    pub fn task(&self) -> &str {
        self.task
            .as_deref()
            .or(self.task_flag.as_deref())
            .unwrap_or_default()
    }
}

#[derive(Debug, Args)]
pub struct LandArgs {
    /// Run id, or any unique prefix of it (see `kit receipt`)
    #[arg(value_name = "RUN")]
    pub id: String,

    /// Name of the new branch [default: kit/<first 12 chars of id>]
    #[arg(short, long, value_name = "NAME", conflicts_with = "apply")]
    pub branch: Option<String>,

    /// Apply the changes to your working tree instead. No commit
    #[arg(long)]
    pub apply: bool,

    /// Land a run the gate did not prove, or apply to a dirty tree
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct ReceiptArgs {
    #[command(subcommand)]
    pub action: Option<ReceiptAction>,

    #[command(flatten)]
    pub list: ListArgs,
}

#[derive(Debug, Subcommand)]
pub enum ReceiptAction {
    /// List runs, newest first (the default)
    #[command(visible_alias = "ls")]
    List(ListArgs),
    /// Show one run: state, checks, changes
    #[command(alias = "get")]
    Show(ShowArgs),
}

#[derive(Debug, Args)]
pub struct ListArgs {
    /// Most runs to list
    #[arg(short = 'n', long, default_value_t = 50, value_name = "N")]
    pub limit: usize,
}

#[derive(Debug, Args)]
pub struct ShowArgs {
    /// Run id, or any unique prefix of it
    #[arg(value_name = "RUN")]
    pub id: String,

    /// Include the tail of the agent's output
    #[arg(short, long)]
    pub output: bool,
}

/// Agents `--agent` accepts, in the order kit picks a default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum AgentArg {
    Claude,
    Codex,
    Grok,
    Ollama,
}

impl AgentArg {
    /// Default-pick order: the first ready agent wins.
    pub const PREFERENCE: [AgentKind; 4] = [
        AgentKind::Claude,
        AgentKind::Codex,
        AgentKind::Grok,
        AgentKind::Ollama,
    ];
}

impl From<AgentArg> for AgentKind {
    fn from(a: AgentArg) -> Self {
        match a {
            AgentArg::Claude => AgentKind::Claude,
            AgentArg::Codex => AgentKind::Codex,
            AgentArg::Grok => AgentKind::Grok,
            AgentArg::Ollama => AgentKind::Ollama,
        }
    }
}

/// `90`, `90s`, `5m`, `1h`.
fn parse_duration(raw: &str) -> Result<Duration, String> {
    crate::init::parse_duration(raw)
        .ok_or_else(|| format!("'{raw}' is not a duration. Use 90s, 5m or 1h"))
}

/// The `command` field of a JSON envelope, from raw argv (works before parsing).
pub fn command_name(args: &[String]) -> String {
    // Skip the value of -C / --repo so `kit -C dir run` still names `run`.
    let mut prev_was_dir = false;
    let mut first = None;
    let mut second = None;
    for a in args {
        if prev_was_dir {
            prev_was_dir = false;
            continue;
        }
        if a == "-C" || a == "--repo" {
            prev_was_dir = true;
            continue;
        }
        if a.starts_with('-') {
            continue;
        }
        if first.is_none() {
            first = Some(a.as_str());
        } else {
            second = Some(a.as_str());
            break;
        }
    }
    match first {
        Some("receipt" | "receipts") => {
            let sub = match second {
                Some("show" | "get") => "show",
                _ => "list",
            };
            format!("receipt.{sub}")
        }
        Some(first) => first.to_string(),
        None => "kit".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    fn parse(line: &str) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(std::iter::once("kit").chain(line.split_whitespace()))
    }

    #[test]
    fn grammar_is_consistent() {
        Cli::command().debug_assert();
    }

    #[test]
    fn bare_kit_and_demo_open_the_control_room() {
        let cli = parse("").unwrap();
        assert!(cli.command.is_none() && !cli.demo);
        let cli = parse("--demo").unwrap();
        assert!(cli.command.is_none() && cli.demo);
    }

    #[test]
    fn run_takes_the_task_positionally_or_as_a_flag() {
        let Some(Command::Run(r)) = parse("run fix-it").unwrap().command else {
            panic!("not run");
        };
        assert_eq!(r.task(), "fix-it");
        assert_eq!(r.agent, None);
        let Some(Command::Run(r)) = parse("run --task fix-it -a codex --dry-run")
            .unwrap()
            .command
        else {
            panic!("not run");
        };
        assert_eq!(r.task(), "fix-it");
        assert_eq!(r.agent, Some(AgentArg::Codex));
        assert!(r.dry_run);
        assert!(parse("run").is_err(), "a run needs a task");
    }

    #[test]
    fn global_flags_work_before_or_after_the_command() {
        let cli = parse("-C /tmp run x --json").unwrap();
        assert_eq!(
            cli.global.dir.as_deref(),
            Some(std::path::Path::new("/tmp"))
        );
        assert!(cli.global.json);
        let cli = parse("run x --repo /tmp").unwrap();
        assert_eq!(
            cli.global.dir.as_deref(),
            Some(std::path::Path::new("/tmp"))
        );
    }

    #[test]
    fn receipt_lists_by_default() {
        let Some(Command::Receipt(r)) = parse("receipt -n 3").unwrap().command else {
            panic!("not receipt");
        };
        assert!(r.action.is_none());
        assert_eq!(r.list.limit, 3);
        let Some(Command::Receipt(r)) = parse("receipt show 01M --output").unwrap().command else {
            panic!("not receipt");
        };
        assert!(matches!(
            r.action,
            Some(ReceiptAction::Show(ShowArgs { output: true, .. }))
        ));
    }

    #[test]
    fn unknown_command_suggests_the_nearest() {
        let err = parse("doctr").unwrap_err().to_string();
        assert!(err.contains("doctor"), "{err}");
    }

    #[test]
    fn land_branch_and_apply_conflict() {
        assert!(parse("land 01M --branch b --apply").is_err());
    }

    #[test]
    fn json_error_envelopes_name_the_command() {
        let argv = |s: &str| s.split(' ').map(String::from).collect::<Vec<_>>();
        assert_eq!(command_name(&argv("run --task x --json")), "run");
        assert_eq!(command_name(&argv("-C dir run x --json")), "run");
        assert_eq!(command_name(&argv("doctor --json")), "doctor");
        assert_eq!(
            command_name(&argv("receipt show 01M --json")),
            "receipt.show"
        );
        assert_eq!(command_name(&argv("receipt --json")), "receipt.list");
        assert_eq!(command_name(&argv("--json")), "kit");
    }

    #[test]
    fn preference_order_matches_the_value_enum() {
        let from_enum: Vec<AgentKind> = AgentArg::value_variants()
            .iter()
            .map(|a| (*a).into())
            .collect();
        assert_eq!(from_enum, AgentArg::PREFERENCE);
    }
}

#[derive(Debug, Args)]
pub struct AddArgs {
    /// Kit names (see kit show), or folders with a KIT.toml
    #[arg(value_name = "KIT", required = true)]
    pub kits: Vec<String>,
    /// Install for all your projects instead of this repo
    #[arg(short, long)]
    pub global: bool,
    /// Agent to set up; repeat for more. Default: every agent installed here
    #[arg(short, long, value_enum)]
    pub agent: Vec<crate::kits::writers::Agent>,
    /// Skills and rules only: no MCP servers, no hooks
    #[arg(long)]
    pub no_code: bool,
    /// Show the plan and write nothing
    #[arg(long)]
    pub print: bool,
    /// Do not ask; install the plan as shown
    #[arg(short, long)]
    pub yes: bool,
    /// Replace skill folders and config entries Kit did not write, or that
    /// were changed by hand; with an upgrade, also remove edited skills the
    /// new version dropped
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct RemoveArgs {
    #[arg(value_name = "KIT", required = true)]
    pub kits: Vec<String>,
    /// Remove from all projects instead of this repo
    #[arg(short, long)]
    pub global: bool,
    /// Do not ask
    #[arg(short, long)]
    pub yes: bool,
    /// Also remove skill folders that were edited by hand
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct DoctorArgs {
    /// Also start each kit's local MCP servers once to see they answer.
    /// Without it, doctor only reads the agents' config files
    #[arg(long)]
    pub start_mcp: bool,
}

#[derive(Debug, Args)]
pub struct ListKitsArgs {
    /// Only kits installed for all projects
    #[arg(short, long)]
    pub global: bool,
}

#[derive(Debug, Subcommand)]
pub enum HookCommand {
    /// Run a kit's after-edit hooks for the file in the hook payload (stdin)
    AfterEdit {
        kit: String,
        /// Which install's record to read (written into the hook by kit add)
        #[arg(long, value_enum)]
        scope: Option<HookScope>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum HookScope {
    Global,
    Repo,
}

#[derive(Debug, Default, Args)]
pub struct SetupArgs {
    /// Agent to set up; repeat for more (skips the question)
    #[arg(short, long, value_enum)]
    pub agent: Vec<crate::kits::writers::Agent>,
    /// Kit to install; repeat for more (skips the question)
    #[arg(short, long, value_name = "KIT")]
    pub kit: Vec<String>,
    /// Install for all your projects (skips the question)
    #[arg(short, long, conflicts_with = "this_repo")]
    pub global: bool,
    /// Install into this repo only (skips the question)
    #[arg(long)]
    pub this_repo: bool,
    /// Skills and rules only: no MCP servers, no hooks
    #[arg(long)]
    pub no_code: bool,
    /// Do not ask before installing
    #[arg(short, long)]
    pub yes: bool,
    /// Replace skill folders and config entries Kit did not write, or that
    /// were changed by hand
    #[arg(long)]
    pub force: bool,
}
