#![cfg(target_os = "linux")]
use std::process::Command;

#[test]
fn missing_daemon_is_explicit() {
    let output = Command::new(env!("CARGO_BIN_EXE_marinaosctl"))
        .env("MARINAOS_SOCKET", "/nonexistent/marinaos-test.sock")
        .args(["status", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("daemon unavailable"));
}
#[test]
fn unknown_commands_fail() {
    let output = Command::new(env!("CARGO_BIN_EXE_marinaosctl"))
        .arg("invalid")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
}
