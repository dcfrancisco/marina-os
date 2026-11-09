# MarinaOS local diagnostics protocol v1

`marinaosd` listens on `/run/marinaos/marinaos.sock` (not Talvik's socket).
The systemd unit creates a mode 0750 directory owned by the service user and
`marinaos-readers` group. The daemon creates a mode 0660 socket. Linux
`SO_PEERCRED` authenticates each connection; root, the service UID, and locally
registered members of `marinaos-readers` may use the read-only operations.
The daemon checks local `/etc/passwd` and `/etc/group` records independently
of socket access and rejects ambiguous or missing records. NSS-only users are
not currently supported. An administrator can add a local user with
`sudo usermod -aG marinaos-readers USER`; a new login is needed for the socket
permission to take effect. Removing membership also revokes the daemon check.
This grants no authority for mutations or future privileged methods.

At the standard path the client verifies that the peer UID matches the socket
owner and the local `marinaos` account, and checks that the directory is owned
by that account and not group- or world-writable. `MARINAOS_SOCKET` overrides
the path for isolated development; the caller must trust that directory and
server, since an alternate path does not identify the installed service.

Send one UTF-8 JSON object, then shut down the write half of the connection:
`{"version":1,"method":"hardware.inventory"}`. At most 4096 request bytes
are accepted, with a two-second total read deadline. The server replies once
with `{"version":1,"result":...,"error":null}` or a null result and an
error code: `unauthorized`, `malformed_request`, `request_too_large`,
`read_timeout`, `unsupported_version`, or `unsupported_method`. Unknown fields
are rejected. The methods are `version`, `status`, `doctor`,
`hardware.inventory`, and `capabilities.list`. Hardware fields that cannot be
observed are null. Systemd `running` maps to healthy, `degraded` maps to degraded, and errors,
transitional and unknown states map to unavailable. The bounded query uses the
existing systemd health adapter.

The server processes connections sequentially with bounded request/read and
write waits. This is not a general-purpose high-throughput IPC service. Logs
contain lifecycle and connection error categories, not request bodies. The
systemd service requires a `marinaos` system user; `marinaos.conf` is a sysusers
input for the separate core package. Actual systemd-managed operation remains
a separate qualification gate.

## Build and package

On a supported Ubuntu 24.04 amd64 build host with the Rust toolchain and Debian
packaging tools installed, run:

```sh
cargo build --release -p marinaos-core -p marinaos-cli
OUTPUT_DIR=/tmp/marinaos-packages scripts/package-marinaos-deb.sh
```

The script refuses to replace existing output files. It computes native library
dependencies from the supplied binaries, so the build host's libc determines the
minimum target libc. The target needs the packages' declared runtime dependencies,
not a compiler. These commands build packages; installation, systemd lifecycle,
and release signing require the separate qualification described in WP-0089.
