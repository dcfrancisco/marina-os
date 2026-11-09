#![cfg(target_os = "linux")]
use marinaos_core::{authorized, dispatch, parse_request, Request};

#[test]
fn systemd_states_do_not_assume_health() {
    assert_eq!(marinaos_core::systemd_status("running"), "healthy");
    assert_eq!(marinaos_core::systemd_status("degraded"), "degraded");
    for state in ["starting", "offline", "", "unknown"] {
        assert_eq!(marinaos_core::systemd_status(state), "unavailable");
    }
}

#[test]
fn local_reader_membership_fails_closed() {
    use marinaos_core::local_group_authorized;
    let passwd = "alice:x:1001:1001::/home/alice:/bin/sh\nbob:x:1002:2000::/home/bob:/bin/sh\n";
    let groups = "marinaos-readers:x:2000:alice\n";
    assert!(local_group_authorized(1001, passwd, groups));
    assert!(local_group_authorized(1002, passwd, groups));
    assert!(!local_group_authorized(1003, passwd, groups));
    assert!(!local_group_authorized(
        1001,
        passwd,
        "marinaos-readers:x:2000:malice\n"
    ));
    assert!(!local_group_authorized(
        1001,
        &format!("{passwd}evil:x:1001:1::/:/bin/sh\n"),
        groups
    ));
    assert!(!local_group_authorized(
        1001,
        passwd,
        &format!("{groups}{groups}")
    ));
}

#[test]
fn contract_validation_and_access_policy() {
    assert!(authorized(0, 10));
    assert!(authorized(10, 10));
    assert!(!authorized(11, 10));
    assert!(parse_request(br#"{"version":1,"method":"version","extra":0}"#).is_none());
    assert!(parse_request(b"{").is_none());
    assert_eq!(
        dispatch(Request {
            version: 2,
            method: "version".into()
        })
        .error
        .as_deref(),
        Some("unsupported_version")
    );
    assert_eq!(
        dispatch(Request {
            version: 1,
            method: "unknown".into()
        })
        .error
        .as_deref(),
        Some("unsupported_method")
    );
}
#[test]
fn hardware_is_observed() {
    let response = dispatch(Request {
        version: 1,
        method: "hardware.inventory".into(),
    });
    let result = response.result.unwrap();
    assert!(result["architecture"].is_string());
    assert!(result["memory_total"].as_str().unwrap().ends_with("kB"));
}

#[test]
fn socket_protocol_and_input_limits() {
    use std::{
        io::{Read, Write},
        os::unix::net::UnixStream,
        thread,
    };
    for (payload, expected) in [
        (b"{".to_vec(), "malformed_request"),
        (vec![b'x'; 4097], "request_too_large"),
        (br#"{"version":1,"method":"version"}"#.to_vec(), "marinaosd"),
    ] {
        let (mut client, server) = UnixStream::pair().unwrap();
        let uid = rustix::process::geteuid().as_raw();
        let handle = thread::spawn(move || marinaos_core::serve_connection(server, uid).unwrap());
        client.write_all(&payload).unwrap();
        client.shutdown(std::net::Shutdown::Write).unwrap();
        let mut response = String::new();
        client.read_to_string(&mut response).unwrap();
        handle.join().unwrap();
        assert!(response.contains(expected), "{response}");
    }
}

#[test]
fn unauthorized_peer_is_rejected() {
    use std::{
        io::{Read, Write},
        os::unix::net::UnixStream,
        thread,
    };
    let uid = rustix::process::geteuid().as_raw();
    if uid == 0 {
        return;
    }
    let (mut client, server) = UnixStream::pair().unwrap();
    let worker = thread::spawn(move || {
        marinaos_core::serve_connection(server, uid.wrapping_add(1)).unwrap()
    });
    client
        .write_all(br#"{"version":1,"method":"version"}"#)
        .unwrap();
    client.shutdown(std::net::Shutdown::Write).unwrap();
    let mut response = String::new();
    client.read_to_string(&mut response).unwrap();
    worker.join().unwrap();
    assert!(response.contains("unauthorized"));
}

#[test]
fn idle_client_times_out() {
    use std::{io::Read, os::unix::net::UnixStream, thread};
    let uid = rustix::process::geteuid().as_raw();
    let (mut client, server) = UnixStream::pair().unwrap();
    let worker = thread::spawn(move || marinaos_core::serve_connection(server, uid).unwrap());
    let mut response = String::new();
    client.read_to_string(&mut response).unwrap();
    worker.join().unwrap();
    assert!(response.contains("read_timeout"));
}
