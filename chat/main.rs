//! `MarinaOS` chat entry point, delegating inference to the sibling ask client.
#![forbid(unsafe_code)]

fn main() {
    if std::env::args()
        .skip(1)
        .any(|arg| arg == "--help" || arg == "-h")
    {
        println!("MarinaOS Chat\nUsage: chat [--plain] [--timing | --benchmark [--runs N]] <prompt>\nUses the sibling ask executable and Talvik IPC.");
        return;
    }
    let result: std::io::Result<std::process::ExitStatus> =
        std::env::current_exe().and_then(|path| {
            let mut command = std::process::Command::new(
                path.with_file_name(format!("ask{}", std::env::consts::EXE_SUFFIX)),
            );
            command.args(std::env::args_os().skip(1));
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                Err(command.exec())
            }
            #[cfg(not(unix))]
            {
                command.status()
            }
        });
    match result {
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(error) => {
            eprintln!("MarinaOS chat could not start sibling ask: {error}");
            std::process::exit(1);
        }
    }
}
