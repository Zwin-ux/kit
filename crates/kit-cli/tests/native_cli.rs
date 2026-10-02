//! Native installation and launch preview need neither Claude nor authentication.
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "kit-native-cli-{}-{nonce}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        // Match the real working directory reported by child processes on macOS,
        // where the temporary directory's /var prefix is a symlink.
        #[cfg(unix)]
        let path = path.canonicalize().unwrap();
        Self(path)
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_kit"))
            .args(args)
            .current_dir(&self.0)
            .env("KIT_HOME", self.0.join("home"))
            .output()
            .unwrap()
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn json(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).expect("one JSON envelope on stdout")
}

#[test]
fn parallel_scratch_directories_remain_independent() {
    let scratches = std::thread::scope(|scope| {
        (0..8)
            .map(|_| scope.spawn(|| (0..64).map(|_| Scratch::new()).collect::<Vec<_>>()))
            .collect::<Vec<_>>()
            .into_iter()
            .flat_map(|thread| thread.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(
        scratches
            .iter()
            .map(|scratch| &scratch.0)
            .collect::<std::collections::HashSet<_>>()
            .len(),
        scratches.len()
    );
    assert!(scratches.iter().all(|scratch| scratch.0.is_dir()));
}

#[cfg(unix)]
#[test]
fn version_probe_does_not_claim_mod_activation_or_execution() {
    use std::os::unix::fs::PermissionsExt;
    let scratch = Scratch::new();
    let host = scratch.0.join("claude");
    std::fs::write(
        &host,
        "#!/bin/sh\n[ \"$1\" = \"--version\" ] || exit 9\nprintf '2.1.287 (Claude Code)\\n'\n",
    )
    .unwrap();
    std::fs::set_permissions(&host, std::fs::Permissions::from_mode(0o755)).unwrap();
    for structured in [true, false] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_kit"));
        command.args(["claude", "check"]);
        if structured {
            command.arg("--json");
        }
        let output = command
            .current_dir(&scratch.0)
            .env("PATH", &scratch.0)
            .output()
            .unwrap();
        assert!(output.status.success());
        if structured {
            let data = json(&output)["data"].clone();
            assert_eq!(data["versionCompatible"], true);
            assert_eq!(data["modsEnabled"], "unknown");
            assert_eq!(data["loginChecked"], false);
            assert_eq!(data["executionVerified"], false);
            assert!(data.get("mods").is_none());
        } else {
            let text = String::from_utf8(output.stdout).unwrap();
            assert!(text.contains("version compatible"));
            assert!(text.contains("Mods activation, login and execution were not checked"));
            assert!(!text.contains("Mods supported"));
        }
    }
}

#[test]
fn install_preview_is_read_only_and_real_install_preserves_existing_files() {
    let scratch = Scratch::new();
    let args = ["claude", "install", "--dir", "native plugin", "--json"];
    let preview = scratch.run(&[&args[..], &["--print"]].concat());
    assert!(preview.status.success());
    assert_eq!(json(&preview)["data"]["written"], false);
    assert!(!scratch.0.join("native plugin").exists());
    let installed = scratch.run(&args);
    assert!(
        installed.status.success(),
        "{}",
        String::from_utf8_lossy(&installed.stderr)
    );
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(scratch.0.join("native plugin/.claude-plugin/plugin.json"))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(manifest["agents"].as_array().unwrap().len(), 4);
    for file in manifest["agents"].as_array().unwrap() {
        assert!(
            scratch
                .0
                .join("native plugin")
                .join(file.as_str().unwrap())
                .is_file()
        );
    }
    let note = scratch.0.join("native plugin/note.txt");
    std::fs::write(&note, "user changes").unwrap();
    let repeated = scratch.run(&args);
    assert!(!repeated.status.success());
    assert_eq!(json(&repeated)["command"], "claude.install");
    assert_eq!(std::fs::read_to_string(note).unwrap(), "user changes");
}

#[test]
fn launch_preview_keeps_paths_as_one_argument_and_pipe_launch_is_refused() {
    let scratch = Scratch::new();
    assert!(
        scratch
            .run(&["claude", "install", "--dir", "native & plugin"])
            .status
            .success()
    );
    let args = [
        "claude",
        "launch",
        "--plugin-dir",
        "native & plugin",
        "--json",
    ];
    let preview = scratch.run(&[&args[..], &["--print"]].concat());
    assert!(preview.status.success());
    let data = json(&preview);
    assert_eq!(data["data"]["launched"], false);
    assert_eq!(data["data"]["args"].as_array().unwrap().len(), 2);
    assert_eq!(data["data"]["args"][0], "--plugin-dir");
    assert_eq!(
        data["data"]["args"][1],
        scratch.0.join("native & plugin").to_string_lossy().as_ref()
    );
    let refused = scratch.run(&args);
    assert!(!refused.status.success());
    assert_eq!(json(&refused)["command"], "claude.launch");
    assert!(String::from_utf8_lossy(&refused.stdout).contains("interactive terminal"));
}
