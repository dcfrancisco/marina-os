#![cfg(target_os = "linux")]
use marinaos_core::{authorized, dispatch, parse_request, Request};

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
    use std::{io::Read, os::unix::net::UnixStream, thread};
    let uid = rustix::process::geteuid().as_raw();
    if uid == 0 {
        return;
    }
    let (mut client, server) = UnixStream::pair().unwrap();
    let worker = thread::spawn(move || {
        marinaos_core::serve_connection(server, uid.wrapping_add(1)).unwrap()
    });
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
