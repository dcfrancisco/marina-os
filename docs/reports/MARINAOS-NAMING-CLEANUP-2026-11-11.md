# MarinaOS executable naming cleanup

Date: 2026-11-11. Branch: `feature/talvik-moe-aware-inference`.
Resumed the interrupted naming assignment on `564e157`, preserving its existing
uncommitted edits. This report covers source and bounded client/package tests.

## Outcome and scope

The former `oid-console` executable is `console`; the former
`oid-marina-chat` executable is exposed through `ask` and `chat`. Cargo builds
`ask` once; the small `chat/main.rs` launcher executes its sibling `ask` with the
same arguments, using Unix exec so exit status and signals are preserved.
`talvik` and `talvikd`, internal Cargo packages and source directories retain
their identities. Cargo metadata reports exactly these five binary targets.

Console startup/help, command errors, operation notices and ask timing/benchmark
output use MarinaOS branding. Timing calculations, units, provenance and missing
metrics are preserved. HTTP/API identifiers, persistent keys and environment
variables retain their compatibility names.

Changes cover the console manifest/build script, client source and tests,
`chat/main.rs`, Linux qualification workflow, client installer, Debian builder
and qualifier, staging helper, MarinaOS client service/sysusers/tmpfiles assets,
package ownership/terminal fixtures and active command documentation.

## Packaging and compatibility

`scripts/stage-marinaos-clients.sh` supports DESTDIR, rejects the host root
(including normalized aliases), and refuses existing staged public commands.
Public commands are mode 0755 in `/usr/bin`; private libraries/helper, resources
and configuration use `/usr/lib/marinaos`, `/usr/share/marinaos` and
`/etc/marinaos`. New service state/cache use `/var/lib/marinaos/console` and
`/var/cache/marinaos/console`. No existing state is moved or removed.

The rootless installer builds all three clients without Cargo `--force`, validates
an explicitly supplied native directory before installation, and keeps supplied
client libraries in the chosen prefix's `lib/marinaos`. Linux client RPATHs
support this directory. No installed Talvik binaries, libraries, models, default
model, service configuration or image assets were changed.

The `oid-marina` package and `oid` principal, `oid-console.service` and
`oid-readiness.service` names remain for installed integrations. The new units
read legacy `/etc/oid/marina-client.env`, followed by the new configuration.
`--oid-console` remains a package-builder input alias for existing tooling.
Internal `oid-*` crate names, source filenames, machine-readable fields and
environment keys remain deliberate. Historical documents and legacy image
assets retain their original names; image development is outside this task.

`command -v console` selects the existing user client at
`/home/danny/.marina/bin/console`; `ask` likewise selects the installed client.
`command -v chat` selects `/usr/sbin/chat`, owned by Debian package `ppp`
according to `dpkg-query -S`. No package owns `/usr/bin/chat` or
`/usr/bin/console` on this machine. Packaging `/usr/bin/chat` does not replace
PPP's file. If PATH selects PPP, invoke `/usr/bin/chat` explicitly. An unrelated
package owning `/usr/bin/chat` must be handled by normal dpkg ownership rejection;
forced overwrite is not supported. The fixtures also verify mutual exclusion of
`oid-marina` and `talvik` packages.

## Verification

Commands ran from the repository root with compilation parallelism limited to 2:

```sh
cargo fmt --all -- --check
cargo check --offline -j 2 -p oid-console --all-targets
LD_LIBRARY_PATH=/home/danny/.marina/lib cargo clippy --offline -j 2 -p oid-console --all-targets -- -D warnings
LD_LIBRARY_PATH=/home/danny/.marina/lib cargo test --offline -j 2 -p oid-console --bin console --bin chat --bin ask --test startup --test ipc_chat --test process_signals -- --test-threads=1
LD_LIBRARY_PATH=/home/danny/.marina/lib TALVIK_TEST_NATIVE_LIB_DIR=/home/danny/.marina/lib timeout 60s python3 scripts/test-oid-console-terminal.py target/debug
python3 scripts/test-linux-package-ownership.py
bash -n scripts/stage-marinaos-clients.sh scripts/install-oid-user.sh scripts/package-oid-deb.sh scripts/qualify-oid-deb.sh
git diff --check
```

PASS: formatting, compilation, strict Clippy, 81 Rust tests (25 ask, 43 console,
3 IPC/help, 6 signal and 4 startup), seven terminal tests, and one package
ownership/staging test. Package fixtures use fake native payloads and sanitized
maintainer scripts in disposable roots; they do not prove native package
installation or service startup. Bash syntax and diff checks passed. Unix socket
tests initially failed with sandbox `Operation not permitted`; the identical
commands passed with outside-sandbox permission. No source fix was needed for
those permission failures.

PASS: generated debug `console --help`, `chat --help`, and `ask --help` print
MarinaOS usage without requiring a daemon. The IPC regression tests verify both
ask and chat generation/timing output and reject obsolete OID labels.

```sh
LD_LIBRARY_PATH=/home/danny/.marina/lib timeout 60s target/debug/ask --timing Hello
LD_LIBRARY_PATH=/home/danny/.marina/lib timeout 60s /home/danny/.marina/bin/talvik run --default Hello
```

PASS with socket permission: both requests returned
`Hello! How can I assist you today?` from the installed Qwen baseline. Ask reported
`Model: qwen2.5-0.5b-instruct-q4_k_m`, `Backend: talvikd (llama.cpp)`,
`MarinaOS Inference Metrics`, `Client total: 5.355 s (client-measured)`,
Talvik runtime latency 5.212 s and backend inference duration 5.211 s. Output
contained no OID branding. Model load duration remained `unavailable`.
These are single-request observations, not a performance benchmark.

## Limits and next dependency

No release reinstall, host package install, model download, service restart,
OS build, VM qualification or milestone tag was performed. Existing installed
clients may still show their previous branding until deliberately upgraded.
The new package's system-state paths are not an automatic migration of old
state. Future package upgrade/systemd lifecycle qualification must account for
that separately. This task does not resume the paused image build or the
separate Talvik MoE work queue.
