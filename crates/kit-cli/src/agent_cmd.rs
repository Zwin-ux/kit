//! `kit -p "<task>"`: Kit's own agent, once, in the current repo (PRD §4.1).
//! No terminal is needed, so a boundary prompt is a deny.

use crate::cli::AgentOpts;
use crate::engine::paths::kit_home;
use crate::kits::config::{AgentSection, Config, PermissionsSection};
use anyhow::{Context, Result, bail};
use kit_loop::{
    Agent, AgentEvent, Cancel, DenyAll, Outcome, Permissions, Prices, Profile, Rule, SessionLog,
    Workspace,
};
use kit_providers::{Anthropic, HttpTransport, OpenAiCompat, Provider};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Prices shipped with Kit. Empty until each provider's pricing page is
/// checked; `~/.kit/prices.toml` adds to it.
const PRICES: &str = include_str!("../prices.toml");
const DEFAULT_ANTHROPIC_MODEL: &str = "claude-sonnet-5-5";
const DEFAULT_LOCAL_URL: &str = "http://localhost:11434/v1";

/// A provider and model, from flags, then `[agent]`, then the environment.
pub fn provider(opts: &AgentOpts, sect: &AgentSection) -> Result<(Arc<dyn Provider>, String)> {
    let env = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
    let id = opts
        .provider
        .clone()
        .or_else(|| sect.provider.clone())
        .unwrap_or_else(|| {
            if env("ANTHROPIC_API_KEY").is_some() {
                "anthropic".into()
            } else {
                "openai-compatible".into()
            }
        });
    let transport = Arc::new(HttpTransport::new()?);
    let model = opts
        .model
        .clone()
        .or_else(|| sect.model.clone())
        .or_else(|| env("KIT_MODEL"));
    match id.as_str() {
        "anthropic" => {
            let Some(key) = env("ANTHROPIC_API_KEY") else {
                bail!(
                    "no Anthropic API key: set ANTHROPIC_API_KEY (keychain storage arrives in A0.4)"
                );
            };
            let base = opts
                .base_url
                .clone()
                .or_else(|| env("ANTHROPIC_BASE_URL"))
                .unwrap_or_else(|| "https://api.anthropic.com".into());
            Ok((
                Arc::new(Anthropic::new(base, key, transport)),
                model.unwrap_or_else(|| DEFAULT_ANTHROPIC_MODEL.into()),
            ))
        }
        "openai-compatible" => {
            let base = opts
                .base_url
                .clone()
                .or_else(|| sect.base_url.clone())
                .or_else(|| env("KIT_BASE_URL"))
                .unwrap_or_else(|| DEFAULT_LOCAL_URL.into());
            let Some(model) = model else {
                bail!(
                    "name a model for {base}: --model <id>, KIT_MODEL, or [agent] model in ~/.kit/config.toml (for Ollama: `ollama list`)"
                );
            };
            Ok((
                Arc::new(OpenAiCompat::new(base, env("KIT_API_KEY"), transport)),
                model,
            ))
        }
        other => bail!("unknown provider `{other}` (anthropic or openai-compatible)"),
    }
}

fn permissions(profile: Profile, sect: Option<&PermissionsSection>) -> Result<Permissions> {
    let parse = |rules: &[String]| -> Result<Vec<Rule>> {
        rules
            .iter()
            .map(|r| {
                Rule::parse(r)
                    .map_err(anyhow::Error::msg)
                    .with_context(|| "in [permissions] of ~/.kit/config.toml")
            })
            .collect()
    };
    let empty = PermissionsSection::default();
    let s = sect.unwrap_or(&empty);
    Ok(Permissions {
        profile,
        allow: parse(&s.allow)?,
        ask: parse(&s.ask)?,
        deny: parse(&s.deny)?,
    })
}

fn prices() -> Result<Prices> {
    let mut prices = Prices::from_toml(PRICES).map_err(anyhow::Error::msg)?;
    let user = kit_home().join("prices.toml");
    if let Ok(text) = std::fs::read_to_string(&user) {
        let extra = Prices::from_toml(&text)
            .map_err(anyhow::Error::msg)
            .with_context(|| user.display().to_string())?;
        prices.models.extend(extra.models);
    }
    Ok(prices)
}

/// `~/.kit/sessions/<folder>-<hash>`: one folder per repo.
fn session_dir(root: &Path) -> std::path::PathBuf {
    let hash = Sha256::digest(root.to_string_lossy().as_bytes());
    let name = root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "root".into());
    let safe: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    kit_home()
        .join("sessions")
        .join(format!("{safe}-{}", &format!("{hash:x}")[..12]))
}

fn gate_commands(root: &Path) -> Vec<String> {
    std::fs::read_to_string(root.join("kit.toml"))
        .ok()
        .and_then(|t| toml::from_str::<kit_core::KitConfig>(&t).ok())
        .map(|c| {
            c.gate
                .checks()
                .into_iter()
                .map(|(_, cmd)| cmd.to_owned())
                .collect()
        })
        .unwrap_or_default()
}

/// What the card for a tool call names: the path, command or pattern.
fn target(args: &serde_json::Value) -> String {
    let t = ["file_path", "command", "pattern", "name"]
        .iter()
        .find_map(|k| args[*k].as_str())
        .unwrap_or("");
    let first = t.lines().next().unwrap_or("");
    match first.char_indices().nth(80) {
        Some((i, _)) => format!("{}…", &first[..i]),
        None => first.to_owned(),
    }
}

fn first_line(s: &str) -> &str {
    s.lines().find(|l| !l.trim().is_empty()).unwrap_or("")
}

/// Run the agent once and exit 0 only when it finished its turn.
pub async fn cmd_prompt(task: &str, opts: &AgentOpts, json: bool) -> Result<()> {
    let file = Config::load()?.unwrap_or_default();
    let sect = file.agent.clone().unwrap_or_default();
    let (provider, model) = provider(opts, &sect)?;
    let profile: Profile = opts
        .profile
        .as_deref()
        .or(sect.profile.as_deref())
        .unwrap_or("workspace")
        .parse()
        .map_err(anyhow::Error::msg)?;
    let cwd = std::env::current_dir()?;
    let root = crate::kits::install::repo_root(&cwd).unwrap_or(cwd);
    let mut ws =
        Workspace::open(&root).with_context(|| format!("cannot open {}", root.display()))?;
    ws.gate_commands = gate_commands(&ws.root);
    ws.extra_env = sect.env.clone();
    let cfg = kit_loop::Config {
        model: model.clone(),
        max_turns: opts.max_turns.or(sect.max_turns).unwrap_or(50),
        budget_usd: opts.budget.or(sect.budget_usd),
        prices: prices()?,
        text_protocol: opts.text,
        ..kit_loop::Config::default()
    };
    let id = kit_core::RunId::new().0;
    let log = SessionLog::create(&session_dir(&ws.root), &id)
        .context("cannot create the session file")?;
    let session = log.path().to_path_buf();
    let provider_id = provider.id();
    let mut agent = Agent::new(
        provider,
        ws,
        permissions(profile, file.permissions.as_ref())?,
        cfg,
    )
    .with_log(log);

    let cancel = Cancel::default();
    let on_ctrl_c = cancel.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            on_ctrl_c.cancel();
        }
    });
    let (tx, mut rx) = mpsc::channel(256);
    let printer = tokio::spawn(async move {
        // Model text goes to stdout; tool cards and notices to stderr.
        let mut text = String::new();
        while let Some(e) = rx.recv().await {
            match e {
                AgentEvent::Text { text: t } => {
                    text.push_str(&t);
                    if !json {
                        print!("{t}");
                        let _ = std::io::stdout().flush();
                    }
                }
                AgentEvent::ToolStart { name, args, .. } if !json => {
                    eprintln!("\n  ▸ {name}  {}", target(&args))
                }
                AgentEvent::ToolEnd {
                    name,
                    content,
                    is_error,
                    denied,
                    ..
                } if !json => {
                    let mark = if is_error || denied { "✗" } else { "✓" };
                    if denied {
                        eprintln!("\n  ▸ {name}");
                    }
                    eprintln!("    {mark} {}", first_line(&content));
                }
                AgentEvent::Notice { text: t } if !json => eprintln!("\n  ! {t}"),
                _ => {}
            }
        }
        text
    });
    let outcome = agent.run(task, &tx, &DenyAll, &cancel).await;
    drop(tx);
    let text = printer.await.unwrap_or_default();
    let m = &agent.meter;
    let label = match &outcome {
        Outcome::Error(e) => format!("error: {e}"),
        other => serde_json::to_value(other)
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_default()
            .replace('_', " "),
    };
    if json {
        let data = serde_json::json!({
            "outcome": outcome, "text": text, "provider": provider_id, "model": model,
            "profile": format!("{profile:?}").to_ascii_lowercase(), "replies": m.replies,
            "tokens": m.tokens, "costUsd": m.cost_usd, "costEstimated": m.estimated,
            "session": session.display().to_string(),
        });
        let ok = outcome == Outcome::Done;
        let error = (!ok).then(|| label.clone());
        println!(
            "{}",
            serde_json::to_string_pretty(&crate::envelope("prompt", ok, data, error, vec![]))?
        );
    } else {
        let t = &m.tokens;
        eprintln!(
            "\n\nkit: {label} · {model} · {} replies · {} in ({} cached) / {} out · {} · session {}",
            m.replies,
            t.input,
            t.cached_input,
            t.output,
            m.dollars(),
            crate::kits::plan::tilde(&session)
        );
    }
    match outcome {
        Outcome::Done => Ok(()),
        Outcome::Error(_) => std::process::exit(2),
        _ => std::process::exit(1),
    }
}
