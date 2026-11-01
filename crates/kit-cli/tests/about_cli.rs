//! `kit --help` credits the studio; `kit --version` stays one machine-readable
//! line, because the installers and the npm smoke test compare it exactly.

use std::process::Command;

fn kit(args: &[&str]) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_kit"))
        .args(args)
        .output()
        .expect("run kit");
    (
        out.status.success(),
        String::from_utf8(out.stdout).expect("utf-8 stdout"),
    )
}

#[test]
fn help_credits_the_studio() {
    let (ok, stdout) = kit(&["--help"]);
    assert!(ok, "{stdout}");
    assert!(
        stdout.contains("Kit by Westside Web Studio (westsidewebstudio.com)"),
        "{stdout}"
    );
    assert!(stdout.contains("© 2026 Westside Web Studio"), "{stdout}");
}

#[test]
fn version_is_one_bare_line() {
    let (ok, stdout) = kit(&["--version"]);
    assert!(ok, "{stdout}");
    assert_eq!(stdout, format!("kit {}\n", env!("CARGO_PKG_VERSION")));
}
