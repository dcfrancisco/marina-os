//! Versioned read-only MarinaOS local protocol and Linux observations.
#![cfg(target_os = "linux")]
use oid_linux_skills::{CommandSystemdHealthAdapter, SystemdHealthAdapter};
use serde_json::{json, Value};
use std::{
    fs,
    io::{self, Read, Write},
    os::unix::{
        fs::{FileTypeExt, MetadataExt},
        net::UnixStream,
    },
    time::Duration,
};

pub const SOCKET: &str = "/run/marinaos/marinaos.sock";
pub const LIMIT: u64 = 4096;
pub const VERSION: u32 = 1;
pub const TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug)]
pub struct Request {
    pub version: u32,
    pub method: String,
}
#[derive(Debug)]
pub struct Response {
    pub version: u32,
    pub result: Option<Value>,
    pub error: Option<String>,
}

pub fn error(code: &str) -> Response {
    Response {
        version: VERSION,
        result: None,
        error: Some(code.into()),
    }
}

fn field(path: &str, key: &str) -> Option<String> {
    fs::read_to_string(path).ok()?.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        (name.trim() == key).then(|| value.trim().to_owned())
    })
}
fn health() -> Value {
    match CommandSystemdHealthAdapter.manager_state() {
        Ok(state) => {
            let status = systemd_status(&state);
            json!({"status": status, "manager_state": state, "error": null})
        }
        Err(_) => {
            json!({"status":"unavailable", "manager_state":null, "error":"systemd manager query failed"})
        }
    }
}
/// Map systemd's reported state without treating transitional states as healthy.
pub fn systemd_status(state: &str) -> &'static str {
    match state {
        "running" => "healthy",
        "degraded" => "degraded",
        _ => "unavailable",
    }
}
pub fn dispatch(request: Request) -> Response {
    if request.version != VERSION {
        return error("unsupported_version");
    }
    let value = match request.method.as_str() {
        "version" => {
            json!({"name":"marinaosd", "version":env!("CARGO_PKG_VERSION"), "protocol":VERSION})
        }
        "status" => {
            json!({"daemon":"running", "hostname":fs::read_to_string("/proc/sys/kernel/hostname").ok().map(|s| s.trim().to_owned()), "systemd":health()})
        }
        "doctor" => {
            json!({"systemd":health(), "uptime_seconds":fs::read_to_string("/proc/uptime").ok().and_then(|s| s.split_whitespace().next().and_then(|v| v.parse::<f64>().ok()))})
        }
        "hardware.inventory" => {
            json!({"architecture":std::env::consts::ARCH, "cpu_model":field("/proc/cpuinfo", "model name"), "logical_cpus":std::thread::available_parallelism().ok().map(|n| n.get()), "memory_total":field("/proc/meminfo", "MemTotal"), "memory_available":field("/proc/meminfo", "MemAvailable")})
        }
        "capabilities.list" => json!([
            "version",
            "status",
            "doctor",
            "hardware.inventory",
            "capabilities.list"
        ]),
        _ => return error("unsupported_method"),
    };
    Response {
        version: VERSION,
        result: Some(value),
        error: None,
    }
}

pub fn parse_request(data: &[u8]) -> Option<Request> {
    let v: Value = serde_json::from_slice(data).ok()?;
    let o = v.as_object()?;
    if o.len() != 2 {
        return None;
    }
    Some(Request {
        version: u32::try_from(o.get("version")?.as_u64()?).ok()?,
        method: o.get("method")?.as_str()?.to_owned(),
    })
}

pub fn authorized(peer_uid: u32, service_uid: u32) -> bool {
    if peer_uid == 0 || peer_uid == service_uid {
        return true;
    }
    match (
        fs::read_to_string("/etc/passwd"),
        fs::read_to_string("/etc/group"),
    ) {
        (Ok(passwd), Ok(group)) => local_group_authorized(peer_uid, &passwd, &group),
        _ => false,
    }
}

/// Resolve membership from local account databases; ambiguous identities fail closed.
pub fn local_group_authorized(uid: u32, passwd: &str, group: &str) -> bool {
    let accounts: Vec<_> = passwd
        .lines()
        .filter_map(|line| {
            let fields: Vec<_> = line.split(':').collect();
            if fields.len() < 4 || fields[2].parse::<u32>().ok()? != uid {
                return None;
            }
            Some((fields[0], fields[3].parse::<u32>().ok()?))
        })
        .collect();
    let groups: Vec<_> = group
        .lines()
        .filter_map(|line| {
            let fields: Vec<_> = line.split(':').collect();
            if fields.len() != 4 || fields[0] != "marinaos-readers" {
                return None;
            }
            Some((fields[2].parse::<u32>().ok()?, fields[3]))
        })
        .collect();
    if accounts.len() != 1 || groups.len() != 1 {
        return false;
    }
    let (name, primary_gid) = accounts[0];
    let (gid, members) = groups[0];
    primary_gid == gid
        || members
            .split(',')
            .any(|member| !member.is_empty() && member == name)
}

pub fn serve_connection(mut stream: UnixStream, service_uid: u32) -> io::Result<()> {
    stream.set_read_timeout(Some(TIMEOUT))?;
    stream.set_write_timeout(Some(TIMEOUT))?;
    let peer = rustix::net::sockopt::get_socket_peercred(&stream).map_err(io::Error::from)?;
    let permitted = authorized(peer.uid.as_raw(), service_uid);
    let response = {
        let mut buf = Vec::new();
        let deadline = std::time::Instant::now() + TIMEOUT;
        let read = loop {
            let Some(remaining) = deadline.checked_duration_since(std::time::Instant::now()) else {
                break Err(io::Error::new(io::ErrorKind::TimedOut, "request deadline"));
            };
            stream.set_read_timeout(Some(remaining))?;
            let mut chunk = [0u8; 1024];
            match stream.read(&mut chunk) {
                Ok(0) => break Ok(()),
                Ok(n) => {
                    buf.extend_from_slice(&chunk[..n]);
                    if buf.len() as u64 > LIMIT {
                        break Ok(());
                    }
                }
                Err(e) => break Err(e),
            }
        };
        match read {
            Ok(_) if !permitted => error("unauthorized"),
            Ok(_) if buf.len() as u64 > LIMIT => error("request_too_large"),
            Ok(_) => match parse_request(&buf) {
                Some(req) => dispatch(req),
                None => error("malformed_request"),
            },
            Err(_) => error("read_timeout"),
        }
    };
    serde_json::to_writer(
        &mut stream,
        &json!({"version": response.version, "result": response.result, "error": response.error}),
    )?;
    stream.flush()
}

pub fn call(path: &str, method: &str) -> Result<Response, String> {
    let mut stream = UnixStream::connect(path).map_err(|e| format!("daemon unavailable: {e}"))?;
    let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    let peer = rustix::net::sockopt::get_socket_peercred(&stream).map_err(|e| e.to_string())?;
    if !metadata.file_type().is_socket() || metadata.uid() != peer.uid.as_raw() {
        return Err("daemon identity does not match socket owner".into());
    }
    if path == SOCKET {
        let passwd =
            fs::read_to_string("/etc/passwd").map_err(|_| "cannot verify daemon account")?;
        let uids: Vec<u32> = passwd
            .lines()
            .filter_map(|line| {
                let fields: Vec<_> = line.split(':').collect();
                if fields.len() < 3 || fields[0] != "marinaos" {
                    return None;
                }
                fields[2].parse().ok()
            })
            .collect();
        let parent =
            fs::symlink_metadata("/run/marinaos").map_err(|_| "cannot verify daemon directory")?;
        if uids.len() != 1
            || uids[0] != peer.uid.as_raw()
            || !parent.is_dir()
            || parent.uid() != uids[0]
            || parent.mode() & 0o022 != 0
        {
            return Err("untrusted daemon identity or directory".into());
        }
    }
    stream
        .set_read_timeout(Some(TIMEOUT))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(TIMEOUT))
        .map_err(|e| e.to_string())?;
    serde_json::to_writer(&mut stream, &json!({"version": VERSION, "method": method}))
        .map_err(|e| e.to_string())?;
    stream
        .shutdown(std::net::Shutdown::Write)
        .map_err(|e| e.to_string())?;
    let mut data = Vec::new();
    stream
        .take(65537)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() > 65536 {
        return Err("response too large".into());
    }
    let value: Value = serde_json::from_slice(&data).map_err(|e| e.to_string())?;
    let response = Response {
        version: value["version"]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or("invalid response")?,
        result: (!value["result"].is_null()).then(|| value["result"].clone()),
        error: value["error"].as_str().map(str::to_owned),
    };
    if response.version != VERSION {
        return Err("unsupported response version".into());
    }
    Ok(response)
}
