# WP-0089: OID/Marina Packaged Linux Environment

> M0 reconciliation (2026-11-09): this existing WP retains ownership. Historical
> AI-composed acceptance below is superseded for the Linux foundation by the
> [M0 readiness and next-task report](../reports/MARINAOS-LINUX-FOUNDATION-RECONCILIATION-2026-11-09.md).
> No completed work is removed; Ubuntu migration and independent boot remain unqualified.

## Purpose

Deliver the first versioned Debian/Ubuntu system-profile artifact before the
bootable-image work in WP-0088. This package composes the already-qualified
Marina Linux binary, relocatable llama.cpp libraries, OID console, existing
systemd units, and existing configuration templates.

## Existing capability inventory

- `scripts/install-model-runner.sh`: user-scoped Talvik daemon/client installer;
  preserves `$HOME/.marina` state and models.
- `scripts/install-oid-user.sh`: new rootless OID console installer; it does
  not duplicate Marina installation and shares only the canonical native
  library/model home when explicitly supplied.
- `scripts/install-model-runner.ps1`: Windows user-scoped installer.
- `scripts/build-oid-linux-image.sh`: image-only composition, not a Debian
  package or general upgrade/uninstall mechanism.
- `packaging/marina/*`, `packaging/oid/*`, `packaging/sysusers/*`, and
  `packaging/tmpfiles/*`: existing system-profile assets reused here.
- `scripts/package-oid-deb.sh`: new versioned `oid-marina` Debian artifact.
- `scripts/qualify-oid-deb.sh`: new isolated dpkg install/upgrade/failure/
  removal data-retention test; it never changes host services.

There is no separate `oid-core` executable in the current workspace. The
current OID core/session boundary is delivered by `oid-console`; this package
does not invent or claim a nonexistent binary.

## Acceptance criteria

- Build `oid-marina_<version>_amd64.deb` from native Linux binaries and pinned
  relocatable `libllama`/`libggml` libraries.
- Include `talvikd`, `talvik`, `oid-console`, systemd units, sysusers/tmpfiles,
  configuration templates, and readiness helpers.
- Preserve `/var/lib/marina/models`, `/var/lib/oid/evidence`, and conffiles on
  package removal.
- Qualify package contents and dpkg lifecycle in an isolated filesystem.
- Record package checksum and target ABI metadata.
- Do not modify host services or the Phenom installation during qualification.
- Rootless users can install `oid-console` separately while keeping Marina's
  user-scoped lifecycle independent.

## Non-goals

Bootable image work (WP-0088), model bundling by default, cloud BYOK, training,
GPU variants, and remote network exposure.

## Status

In Progress. Native Linux Docker execution produced `oid-marina_0.1.0_amd64.deb`
and `oid-marina_0.1.1_amd64.deb`. Isolated dpkg qualification passed install,
upgrade, rejected invalid upgrade, removal, and retention of models, evidence,
and configuration. The package checksum for 0.1.0 is
`7ac0f8439e1bafd34001cd12c628b66db9df762c1cde92b2378854553381248c`.

Not yet qualified: starting/restarting the service under a real systemd boot,
GGUF inference from the installed package, ABI compatibility on each supported
Ubuntu/Debian release, and a real-host install. Static unit inspection in a
minimal extracted root reports missing distro target units, so it is not being
counted as systemd qualification. No real host installation has been performed.

## Traceability

Implements ADR-0014 and supports ADR-0023. Depends on WP-0065, WP-0069,
WP-0077, and WP-0078. WP-0088 remains deferred until this packaged
environment is operationally qualified.

## Corrective implementation — 2026-11-09

Continued the next unblocked package-ownership dependency: oid-marina and talvik now declare mutual Conflicts. Rootless relocated dpkg tests with sanitized fixture packages reject both co-installation orders and preserve the previously installed service file. No host service or maintainer script executes in the fixture. Native payload, upgrade, migration, service and boot qualification remain separate; this does not close WP-0089.

Exact commands and results: [implementation report](../reports/MARINAOS-IMAGE-IMPLEMENTATION-2026-11-09.md).

## Read-only MarinaOS CLI slice

`cargo run -p marinaos-cli -- <version|status|doctor|hardware> [--json]`
now prints JSON from local kernel files and systemd. Read failures are reported;
systemd bus failure is not reported as health. The command does not require
Talvik. It is an early standalone client, not yet a daemon client. No
`marinaosd`, authenticated IPC, dedicated service account, MarinaOS DEBs,
installation lifecycle or image integration is implemented in this slice.
Those and native Ubuntu 24.04 package/boot qualification remain open.

## Local diagnostics IPC implementation

The read-only CLI now calls `marinaosd` using the documented
[local protocol](../ipc/marinaos-v1.md). The daemon authenticates kernel peer
UIDs and permits only root or its own UID; ordinary-user delegation is open.
It reports observed kernel facts and explicitly leaves systemd manager health
unknown. The service unit and sysusers definition are packaging inputs, not an
installed or qualified package. No daemon or CLI DEB has been produced.

`cargo test -p marinaos-core -p marinaos-cli --offline` passed outside the
sandbox, including socket framing, unauthorized UID, input limits, timeout,
hardware observation and daemon restart. The sandbox rejected Unix socket
creation with `Operation not permitted`. Real systemd installation, separate
UID service startup, package install/upgrade/removal and Ubuntu 24.04 boot are
unverified. CLI server authentication, ordinary-user delegation, readiness,
and full service/package qualification remain open.

The real CLI-to-daemon command test `python3 scripts/test-marinaos-ipc.py`
passed outside the sandbox using a temporary directory and an unprivileged
same-UID daemon. It verified all five CLI commands and SIGTERM cleanup. It did
not install or start the systemd unit. Strict workspace Clippy and formatting
passed; the workspace test suite passed outside the sandbox with
`LD_LIBRARY_PATH=/home/danny/.marina/lib`. Without that library path, the
existing console test executable failed to load `libllama.so.0`.

## Ordinary-reader access and native package slice

The standard socket directory is owned by the service UID and the
`marinaos-readers` group, mode 0750; the socket is mode 0660. `SO_PEERCRED`
UIDs are checked against local account and group files independently of socket
access. Root and the daemon UID also have read-only access. Missing or ambiguous
records fail closed; NSS-only accounts are not supported. The standard-path
client verifies the peer against the local service UID, socket owner and
non-writable parent directory. Custom socket paths remain a development trust
boundary. The existing systemd adapter now has a one-second process deadline;
the daemon distinguishes running, degraded and unavailable manager states.
See [protocol v1](../ipc/marinaos-v1.md) and ADR-0042.

`scripts/package-marinaos-deb.sh` builds separate core and CLI DEBs from existing
binaries, derives shared-library dependencies, installs the systemd unit and
sysusers definition, and uses Debian systemd helpers. No Talvik, model, GPU or
compiler payload is included. The system account is retained after removal;
`/var/lib/marinaos` is not package-owned or deleted. Build on the oldest
supported target to avoid a newer glibc requirement. These are development
artifacts, not signed or release-qualified packages.

Evidence from this workspace: 0.1.0 and 0.1.1 DEBs built from the same debug binaries on
Ubuntu 26.04 with dependencies computed by `dpkg-shlibdeps`; their libc6 minimum
was 2.39. `scripts/test-marinaos-ubuntu-container.sh` passed in a disposable
Ubuntu 24.04 container (base digest
`sha256:fe81560990fae86dc4743202b2cb4d3df98d0bcf949173cecef15ce1127ef852`).
It installed packages and dependencies, started and restarted the daemon manually
under its dedicated account, checked reader access, outsider denial, inability
to write the socket directory, revocation despite a retained process group,
upgrade, removal, and retention of a test file and service account. One initial
revocation test exposed a connection reset; after fixing the daemon to read a
bounded request before returning `unauthorized`, the complete test passed.

The container had no systemd PID 1, and `policy-rc.d` suppressed service starts.
Thus actual systemd-managed startup, restart, running-service upgrade, boot,
policy under a real system manager, purge, and installation on a clean Ubuntu
VM remain unverified. A host read-only systemd query and the temporary daemon
both reported `degraded`; an inaccessible bus reports unavailable. Rust fmt,
strict workspace Clippy and `cargo test --workspace --offline` passed; the latter
required outside-sandbox Unix sockets and `LD_LIBRARY_PATH=/home/danny/.marina/lib`.
No host packages, services or physical disks were modified.
