//! The Control Room without a terminal says so and exits; it never draws into
//! a pipe, waits for keys that cannot come, or fails with a raw OS error
//! (`No such device or address (os error 6)`).

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[test]
fn control_room_without_a_terminal_exits_with_a_hint() {
    for args in [&["--demo"][..], &[]] {
        let home = std::env::temp_dir().join(format!("kit-no-tty-{}", std::process::id()));
        let started = Instant::now();
        let out = Command::new(env!("CARGO_BIN_EXE_kit"))
            .args(args)
            .env("KIT_HOME", &home)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("run kit");
        let _ = std::fs::remove_dir_all(&home);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "kit {args:?} waited for a terminal"
        );
        assert!(!out.status.success(), "kit {args:?} claimed success");
        assert!(
            stderr.contains("terminal") && !stderr.contains("os error"),
            "kit {args:?} should explain it needs a terminal: {stderr}"
        );
        assert!(out.stdout.is_empty(), "nothing drawn into the pipe");
    }
}
