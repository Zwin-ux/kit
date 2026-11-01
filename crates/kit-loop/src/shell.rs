//! The shell `Bash` runs, and the environment it gets.
//!
//! Unix: `bash`, else `sh`. Windows: Git Bash when installed (as Claude Code
//! uses), else PowerShell. Never `bash.exe` from System32: that is WSL's
//! launcher and starts a different machine.

use std::path::{Path, PathBuf};
use tokio::process::Command;

/// Which shell, and where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shell {
    /// bash or sh, called with `-c`.
    Posix(PathBuf),
    /// PowerShell, called with `-Command`.
    PowerShell(PathBuf),
}

impl Shell {
    /// The best shell on this machine. `KIT_SHELL` overrides.
    pub fn detect() -> Self {
        if let Some(p) = std::env::var_os("KIT_SHELL").map(PathBuf::from) {
            let name = p
                .file_stem()
                .map(|s| s.to_string_lossy().to_ascii_lowercase())
                .unwrap_or_default();
            return if name.contains("pwsh") || name.contains("powershell") {
                Self::PowerShell(p)
            } else {
                Self::Posix(p)
            };
        }
        if cfg!(windows) {
            if let Some(bash) = git_bash() {
                return Self::Posix(bash);
            }
            return Self::PowerShell(
                which("pwsh.exe").unwrap_or_else(|| PathBuf::from("powershell.exe")),
            );
        }
        Self::Posix(which("bash").unwrap_or_else(|| PathBuf::from("/bin/sh")))
    }

    /// Name for the system prompt and fenced blocks: `bash` or `powershell`.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Posix(_) => "bash",
            Self::PowerShell(_) => "powershell",
        }
    }

    /// A command that runs `script` in this shell.
    pub fn command(&self, script: &str) -> Command {
        match self {
            Self::Posix(p) => {
                let mut c = Command::new(p);
                c.arg("-c").arg(script);
                c
            }
            Self::PowerShell(p) => {
                let mut c = Command::new(p);
                c.args([
                    "-NoLogo",
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    script,
                ]);
                c
            }
        }
    }
}

fn which(name: &str) -> Option<PathBuf> {
    std::env::split_paths(&std::env::var_os("PATH")?)
        .map(|d| d.join(name))
        .find(|p| p.is_file())
}

fn git_bash() -> Option<PathBuf> {
    let from_git = which("git.exe").and_then(|git| {
        // <root>\cmd\git.exe or <root>\bin\git.exe → <root>\bin\bash.exe
        let root = git.parent()?.parent()?;
        Some(root.join("bin").join("bash.exe"))
    });
    let defaults = [
        r"C:\Program Files\Git\bin\bash.exe",
        r"C:\Program Files (x86)\Git\bin\bash.exe",
    ];
    from_git
        .into_iter()
        .chain(defaults.iter().map(PathBuf::from))
        .find(|p| p.is_file() && !is_system32(p))
}

fn is_system32(p: &Path) -> bool {
    p.to_string_lossy()
        .to_ascii_lowercase()
        .contains(r"\system32\")
}

/// Variables a tool may see (DESIGN-HARNESS Y5 minus every provider key),
/// plus toolchain homes builds need. Matching ignores case on Windows.
const PASS: &[&str] = &[
    "PATH",
    "HOME",
    "USER",
    "LOGNAME",
    "SHELL",
    "LANG",
    "TERM",
    "TMPDIR",
    "TZ",
    "HTTPS_PROXY",
    "HTTP_PROXY",
    "NO_PROXY",
    "https_proxy",
    "http_proxy",
    "no_proxy",
    "SSL_CERT_FILE",
    "SSL_CERT_DIR",
    "NODE_EXTRA_CA_CERTS",
    "CARGO_HOME",
    "RUSTUP_HOME",
    "GOPATH",
    "GOROOT",
    "JAVA_HOME",
    "NVM_DIR",
    "PNPM_HOME",
    "VIRTUAL_ENV",
    "CONDA_PREFIX",
    "PYENV_ROOT",
    "SYSTEMROOT",
    "WINDIR",
    "USERPROFILE",
    "USERNAME",
    "APPDATA",
    "LOCALAPPDATA",
    "PROGRAMDATA",
    "PROGRAMFILES",
    "PROGRAMFILES(X86)",
    "COMMONPROGRAMFILES",
    "PATHEXT",
    "COMSPEC",
    "TEMP",
    "TMP",
    "HOMEDRIVE",
    "HOMEPATH",
    "SYSTEMDRIVE",
    "OS",
    "NUMBER_OF_PROCESSORS",
    "PROCESSOR_ARCHITECTURE",
];

/// Whether a variable passes: on the list, `LC_*` or `XDG_*`, or named by the user.
/// Names that look like secrets never pass, even if named.
pub fn passes(name: &str, extra: &[String]) -> bool {
    let up = name.to_ascii_uppercase();
    if up.contains("KEY")
        || up.contains("TOKEN")
        || up.contains("SECRET")
        || up.contains("PASSWORD")
    {
        return false;
    }
    let listed = if cfg!(windows) {
        PASS.iter().any(|p| p.eq_ignore_ascii_case(name))
    } else {
        PASS.contains(&name)
    };
    listed || up.starts_with("LC_") || up.starts_with("XDG_") || extra.iter().any(|e| e == name)
}

/// The environment for a tool process: only what [`passes`].
pub fn tool_env(extra: &[String]) -> Vec<(String, String)> {
    std::env::vars().filter(|(k, _)| passes(k, extra)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_keys_never_reach_tools() {
        for k in [
            "ANTHROPIC_API_KEY",
            "OPENAI_API_KEY",
            "XAI_API_KEY",
            "GITHUB_TOKEN",
            "AWS_SECRET_ACCESS_KEY",
        ] {
            assert!(
                !passes(k, &[k.to_owned()]),
                "{k} must never pass, even when named"
            );
        }
        assert!(passes("PATH", &[]));
        assert!(passes("LC_ALL", &[]));
        assert!(passes("MY_VAR", &["MY_VAR".into()]));
        assert!(!passes("MY_VAR", &[]));
        assert!(
            !passes("CARGO_TARGET_DIR", &[]),
            "a run builds into its own target"
        );
    }

    #[test]
    fn wsl_launcher_is_never_git_bash() {
        assert!(is_system32(Path::new(r"C:\Windows\System32\bash.exe")));
        assert!(!is_system32(Path::new(
            r"C:\Program Files\Git\bin\bash.exe"
        )));
    }
}
