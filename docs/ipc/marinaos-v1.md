# MarinaOS local diagnostics protocol v1

`marinaosd` listens on `/run/marinaos/marinaos.sock` (not Talvik's socket).
The systemd unit creates the directory; the daemon creates a mode 0600 socket.
Linux `SO_PEERCRED` authenticates each connection. Only UID 0 and the service
UID may use these read-only operations; socket access does not grant other
rights. Ordinary-user delegation is not implemented. There are no mutations.
`MARINAOS_SOCKET` overrides the path for isolated development and tests; use a
private directory. The client checks that the connected peer UID matches the socket file owner.
The client must also trust its socket directory: this check alone does not
establish that the owner is the installed service account.

Send one UTF-8 JSON object, then shut down the write half of the connection:
`{"version":1,"method":"hardware.inventory"}`. At most 4096 request bytes
are accepted, with a two-second total read deadline. The server replies once
with `{"version":1,"result":...,"error":null}` or a null result and an
error code: `unauthorized`, `malformed_request`, `request_too_large`,
`read_timeout`, `unsupported_version`, or `unsupported_method`. Unknown fields
are rejected. The methods are `version`, `status`, `doctor`,
`hardware.inventory`, and `capabilities.list`. Hardware fields that cannot be
observed are null. Systemd manager health is currently unknown; presence of its
runtime directory is a separate observation, not a readiness result.

The server processes connections sequentially with bounded request/read and
write waits. This is not a general-purpose high-throughput IPC service. Logs
contain lifecycle and connection error categories, not request bodies. The
systemd service requires a `marinaos` system user; `marinaos.conf` is a sysusers
input for future packaging. Neither it nor the unit has been installed or
qualified on Ubuntu 24.04. There are no MarinaOS DEB packages yet.
