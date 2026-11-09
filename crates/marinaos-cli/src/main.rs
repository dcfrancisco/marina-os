#[cfg(target_os = "linux")]
mod linux {
    use marinaos_core::{call, SOCKET};
    use serde_json::json;
    use std::{env, process};

    pub fn run_main() {
        let args: Vec<String> = env::args().skip(1).collect();
        if args.is_empty() || args.len() > 2 || (args.len() == 2 && args[1] != "--json") {
            eprintln!("usage: marinaosctl <version|status|doctor|hardware|capabilities> [--json]");
            process::exit(2);
        }
        let method = match args[0].as_str() {
            "version" => "version",
            "status" => "status",
            "doctor" => "doctor",
            "hardware" => "hardware.inventory",
            "capabilities" => "capabilities.list",
            _ => {
                eprintln!("unknown command");
                process::exit(2);
            }
        };
        let path = env::var("MARINAOS_SOCKET").unwrap_or_else(|_| SOCKET.to_owned());
        match call(&path, method) {
            Ok(response) if response.error.is_none() => println!(
                "{}",
                serde_json::to_string_pretty(&response.result).unwrap()
            ),
            Ok(response) => {
                eprintln!("{}", json!({"error":response.error}));
                process::exit(1);
            }
            Err(error) => {
                eprintln!("{}", json!({"error":error}));
                process::exit(1);
            }
        }
    }
}
#[cfg(target_os = "linux")]
fn main() {
    linux::run_main();
}
#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("MarinaOS diagnostics requires Linux");
    std::process::exit(1);
}
