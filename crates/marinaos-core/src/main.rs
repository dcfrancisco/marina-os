#[cfg(target_os = "linux")]
mod linux {
    use marinaos_core::{serve_connection, SOCKET};
    use std::{
        fs, io,
        os::unix::{
            fs::{FileTypeExt, MetadataExt, PermissionsExt},
            net::{UnixListener, UnixStream},
        },
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        thread,
        time::Duration,
    };

    fn run() -> io::Result<()> {
        let path = std::env::var("MARINAOS_SOCKET").unwrap_or_else(|_| SOCKET.to_owned());
        let uid = rustix::process::geteuid().as_raw();
        if let Ok(metadata) = fs::symlink_metadata(&path) {
            if !metadata.file_type().is_socket()
                || metadata.uid() != uid
                || !matches!(UnixStream::connect(&path), Err(ref e) if e.kind() == io::ErrorKind::ConnectionRefused)
            {
                return Err(io::Error::new(
                    io::ErrorKind::AddrInUse,
                    "socket path is occupied",
                ));
            }
            fs::remove_file(&path)?;
        }
        let listener = UnixListener::bind(&path)?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
        listener.set_nonblocking(true)?;
        let stopping = Arc::new(AtomicBool::new(false));
        signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&stopping))?;
        signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&stopping))?;
        eprintln!("{{\"event\":\"started\"}}");
        while !stopping.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, _)) => {
                    if serve_connection(stream, uid).is_err() {
                        eprintln!("{{\"event\":\"connection_error\"}}");
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50))
                }
                Err(e) => return Err(e),
            }
        }
        fs::remove_file(path)?;
        eprintln!("{{\"event\":\"stopped\"}}");
        Ok(())
    }
    pub fn run_main() {
        if let Err(e) = run() {
            eprintln!(
                "{{\"event\":\"startup_failure\",\"error\":{}}}",
                serde_json::to_string(&e.to_string()).unwrap()
            );
            std::process::exit(1);
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
