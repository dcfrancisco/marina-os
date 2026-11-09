use serde_json::{json, Value};
use std::{env, fs, process, process::Command};

fn read(path: &str) -> Value {
    match fs::read_to_string(path) {
        Ok(value) => json!({"value": value.trim(), "error": null}),
        Err(error) => json!({"value": null, "error": error.to_string()}),
    }
}

fn systemd_state() -> Value {
    match Command::new("systemctl")
        .args(["is-system-running", "--no-legend"])
        .output()
    {
        Ok(output) => json!({
            "state": String::from_utf8_lossy(&output.stdout).trim(),
            "exit_code": output.status.code(),
            "error": String::from_utf8_lossy(&output.stderr).trim(),
        }),
        Err(error) => json!({"state": null, "exit_code": null, "error": error.to_string()}),
    }
}

fn report(command: &str) -> Option<Value> {
    match command {
        "version" => Some(json!({"name": "marinaosctl", "version": env!("CARGO_PKG_VERSION")})),
        "status" => Some(
            json!({"hostname": read("/proc/sys/kernel/hostname"), "systemd": systemd_state(), "daemon": "not implemented"}),
        ),
        "doctor" => Some(
            json!({"systemd": systemd_state(), "proc_uptime": read("/proc/uptime"), "os_release": read("/etc/os-release"), "daemon": "not implemented"}),
        ),
        "hardware" => Some(
            json!({"architecture": env::consts::ARCH, "cpu": read("/proc/cpuinfo"), "memory": read("/proc/meminfo")}),
        ),
        _ => None,
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("");
    if args.len() > 2 || (args.len() == 2 && args[1] != "--json") {
        eprintln!("usage: marinaosctl <version|status|doctor|hardware> [--json]");
        process::exit(2);
    }
    let Some(value) = report(command) else {
        eprintln!("usage: marinaosctl <version|status|doctor|hardware> [--json]");
        process::exit(2);
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&value).expect("JSON serialization")
    );
}
