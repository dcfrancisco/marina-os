#![cfg(target_os = "linux")]
use std::{
    fs,
    process::{Child, Command, Stdio},
    thread,
    time::Duration,
};
struct Daemon(Child);
impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn start(path: &str) -> Daemon {
    let child = Daemon(
        Command::new(env!("CARGO_BIN_EXE_marinaosd"))
            .env("MARINAOS_SOCKET", path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    for _ in 0..100 {
        if std::path::Path::new(path).exists() {
            return child;
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("daemon did not start");
}
#[test]
fn client_can_reconnect_after_daemon_restart() {
    let dir = std::env::temp_dir().join(format!("marinaos-test-{}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    let path = dir.join("service.sock");
    let path = path.to_str().unwrap();
    let daemon = start(path);
    for method in [
        "version",
        "status",
        "doctor",
        "hardware.inventory",
        "capabilities.list",
    ] {
        assert!(marinaos_core::call(path, method).unwrap().result.is_some());
    }
    drop(daemon);
    let daemon = start(path);
    for _ in 0..100 {
        if marinaos_core::call(path, "version").is_ok() {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    assert!(marinaos_core::call(path, "version").is_ok());
    drop(daemon);
    fs::remove_file(path).unwrap();
    fs::remove_dir(dir).unwrap();
}
