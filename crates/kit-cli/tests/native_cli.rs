//! Native installation and launch preview need neither Claude nor authentication.
use std::path::PathBuf;
use std::process::{Command, Output};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("kit-native-cli-{}-{nonce}", std::process::id()));
        std::fs::create_dir(&path).unwrap();
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
