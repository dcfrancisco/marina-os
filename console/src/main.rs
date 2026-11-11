//! AI Console executable.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod audio;
mod chat_ui;
mod commands;
mod editor;
mod foundation;
mod history;
mod input_router;
mod ipc;
mod ipc_runtime;
mod operations;
mod presence;
mod prompt;
mod renderer;
mod shell;
mod signals;
mod status_bar;
mod terminal_ui;
mod tui;

fn main() {
    if std::env::args()
        .skip(1)
        .any(|arg| arg == "--help" || arg == "-h")
    {
        println!("MarinaOS Console\nUsage: console\nInteractive shell and Talvik client. Type :help for commands and :quit to exit.");
        return;
    }
    let application = match tui::Application::start() {
        Ok(application) => application,
        Err(error) => {
            eprintln!("MarinaOS Console failed to start: {error}");
            std::process::exit(1);
        }
    };
    if let Err(error) = application.run() {
        eprintln!("MarinaOS Console failed to start: {error}");
        std::process::exit(1);
    }
}
