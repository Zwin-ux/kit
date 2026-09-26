//! The Control Room without a terminal says so and exits; it never draws into
//! a pipe, waits for keys that cannot come, or fails with a raw OS error
//! (`No such device or address (os error 6)`).

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[test]
fn control_room_without_a_terminal_exits_with_a_hint() {
    for args in [&["--demo"][..], &[]] {
        let home = std::env::temp_dir().join(format!("kit-no-tty-{}", std::process::id()));
        let mut child = Command::new(env!("CARGO_BIN_EXE_kit"))
            .args(args)
            .env("KIT_HOME", &home)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("run kit");
        // A regression that waits for keys must fail here, not hang the suite.
        let deadline = Instant::now() + Duration::from_secs(10);
        let status = loop {
            if let Some(status) = child.try_wait().expect("wait for kit") {
                break status;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                let _ = std::fs::remove_dir_all(&home);
                panic!("kit {args:?} waited for a terminal");
            }
            std::thread::sleep(Duration::from_millis(20));
        };
        let (mut stdout, mut stderr) = (Vec::new(), String::new());
        child
            .stdout
            .take()
            .unwrap()
            .read_to_end(&mut stdout)
            .unwrap();
        child
            .stderr
            .take()
            .unwrap()
            .read_to_string(&mut stderr)
            .unwrap();
        let _ = std::fs::remove_dir_all(&home);
        assert!(!status.success(), "kit {args:?} claimed success");
        assert!(
            stderr.contains("needs an interactive terminal") && !stderr.contains("os error"),
            "kit {args:?} should explain it needs a terminal: {stderr}"
        );
        assert!(stdout.is_empty(), "nothing drawn into the pipe");
    }
}
