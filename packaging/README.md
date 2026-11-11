# MarinaOS/Talvik packaging

## Standalone Talvik packages

Standalone package builders are separate from the legacy system-profile package:

```sh
./scripts/package-talvik-deb.sh \
  --version 0.1.0 \
  --talvikd target/release/talvikd \
  --talvik target/release/talvik \
  --native-lib-dir /path/to/llama/lib

./scripts/package-talvik-rpm.sh \
  --version 0.1.0 \
  --talvikd target/release/talvikd \
  --talvik target/release/talvik \
  --native-lib-dir /path/to/llama/lib
```

Both builders fail closed when release binaries or native libraries are
missing. They do not download models, start services, or remove existing
model state. The packages install a system profile under `/usr/lib/talvik`,
create the unprivileged `talvik` account, and preserve `/var/lib/talvik` on
upgrade/removal. The service is enabled explicitly by the operator:

```sh
sudo apt install ./talvik_*.deb       # Debian/Ubuntu x86_64
sudo systemctl enable --now talvikd
# or: sudo dnf install ./talvik-*.rpm # Fedora/RHEL-family x86_64
```

Linux artifacts target x86_64 systems with glibc 2.35 or newer and systemd.
The RPM workflow is qualified only on the GitHub Ubuntu packaging builder;
Fedora/RHEL installation remains a package-layout check until a native RPM
runner is added. The native CPU build disables AVX, AVX2, AVX-512, FMA, F16C,
SSE4.2, and BMI2 and does not use `-march=native`; final AMD Phenom II
acceptance still requires hardware execution. Models are never downloaded
during package installation.

The tagged Talvik release workflow publishes both the Debian package and the
RPM package alongside the Linux, macOS Intel, macOS Apple Silicon, and Windows
archives. It also emits `SHA256SUMS` for the release payloads. Manual runs
publish the same files as GitHub Actions artifacts.

The macOS archives contain `install-talvik-macos.sh`. Run it from the
extracted archive with `--service` to install a per-user launchd definition:

```sh
./install-talvik-macos.sh --service
```

The macOS helper does not require a privileged `.pkg`, does not modify global
paths, and preserves models and state during uninstall. With `--service`, installation starts one per-user daemon and enables startup
at login. The loopback OpenAI base URL is `http://127.0.0.1:11435/v1`; health
and readiness are `/healthz` and `/readyz`. Create a scoped client token with
`~/.marina/bin/talvik auth token create vscode`. The service profile reserves
a 32K context and a 2,048-token output budget; memory admission still applies.
It does not claim GPU acceleration unless a backend has been qualified.

These files implement the first system-profile image composition described by
ADR-0023. Marina packaging remains governed by ADR-0014 and WP-0069.

The image assets above remain a compatibility profile. WP-0089 packages Talvik
with the new MarinaOS client staging assets below into a versioned Debian artifact with
`scripts/package-oid-deb.sh`. It includes the `talvikd` daemon, `talvik` client,
and `console`, `chat`, and `ask` binaries plus relocatable native libraries; it does not
include model files. Run `scripts/qualify-oid-deb.sh` against the resulting
package to test dpkg lifecycle behavior in an isolated root.

Build the system package on native Linux from qualified artifacts:

```sh
./scripts/package-oid-deb.sh \
  --version 0.1.0 \
  --talvikd target/release/talvikd \
  --talvik target/release/talvik \
  --console target/release/console \
  --chat target/release/chat \
  --ask target/release/ask \
  --native-lib-dir /path/to/qualified/llama/lib
```

Install or upgrade on an authorized Debian/Ubuntu host with `dpkg -i`; then
explicitly enable/start the existing units with `systemctl`. Package removal
does not purge `/var/lib/marina/models`, legacy `/var/lib/oid/evidence`, new
`/var/lib/marinaos/console/evidence`, or conffiles.
The package does not bind or claim Reference local runner's port; Marina retains its documented
loopback default and can be configured separately.

For a rootless MarinaOS client installation, use `scripts/install-oid-user.sh`.
It installs `console`, `chat`, and `ask`; Talvik remains managed by
`scripts/install-model-runner.sh` under `$HOME/.marina`.

The two services intentionally have different identities:

- `talvikd.service` runs the model runner as `marina`, owns `/var/lib/marina`,
  and exposes the Unix socket at `/run/marina/marina.sock` plus loopback HTTP
  at `127.0.0.1:12434`.
- `oid-readiness.service` and `oid-console.service` run as `oid`. MarinaOS may
  request intelligence from Talvik, but model output remains subject to MarinaOS's
  governed-operation pipeline.

The console is a TTY service, not a graphical desktop. A missing model or
missing client token is a degraded readiness state and does not prevent boot.
For the legacy image profile, provision `/etc/oid/marina-client.env` with an inference-scoped token after
installation, then restart `oid-readiness.service`. The model installer must
verify a SHA-256 before placing an artifact in the Marina model directory.

Install these files under their conventional `/usr/lib/systemd/system`,
`/usr/lib/sysusers.d`, `/usr/lib/tmpfiles.d`, `/etc/marina`, and `/etc/oid`
locations when building the image. The image builder performs this mapping.


### Local milestone packaging and qualification

A local archive is a preview until the gates in WP-0111 pass. Build both client
and daemon from the same clean commit with the pinned native payload, then run:

```sh
scripts/package-talvik-unix.sh target/release "$HOME/.marina/lib" /tmp/talvik-local-preview.tar.gz
TMPDIR=/tmp scripts/smoke-release-install.sh /tmp/talvik-local-preview.tar.gz
python3 scripts/test-talvik-installers.py
```

The archive contains binaries, native libraries, installers, build identity, and
license; it does not include models, user configuration, or authentication tokens.
The native payload must include `liboid-chat-template` as well as llama.cpp,
ggml, and mtmd. The installer rejects archives missing this required renderer.
Linux DEB/RPM packages also declare the `libstdc++` runtime dependency used by
the native llama.cpp and chat-template libraries.
The smoke check installs into an isolated directory with developer library paths
removed, starts a daemon without model downloads, checks CLI status, stops it,
and verifies uninstall preserves model state.

For opt-in real-model API qualification, start a native daemon and register the
Qwen Coder and vision models, then run `python3 scripts/test-talvik-native-api.py`.
Set `TALVIK_TEST_TOKEN_FILE` to a private token file; optional `TALVIK_TEST_BASE_URL`,
`TALVIK_TEST_VISION_MODEL`, `TALVIK_TEST_CODER_MODEL`, and `TALVIK_TEST_REPORT`
select the service, model IDs, and JSON evidence path. This is a bounded check
of chat, coding, vision, standard SSE and a tool-result round trip. It does not
execute filesystem tools or certify any IDE's Agent/Plan mode.


### MarinaOS client staging

`DESTDIR=/path/to/stage bash scripts/stage-marinaos-clients.sh BIN_DIR NATIVE_LIB_DIR`
stages `console`, `chat`, and `ask` as mode 0755 public commands in `/usr/bin`.
It refuses an existing staged command rather than overwriting it. Private client
libraries/readiness helpers use `/usr/lib/marinaos`; resources use
`/usr/share/marinaos`, configuration uses `/etc/marinaos`, and the staged client
units use `/var/lib/marinaos` and `/var/cache/marinaos`. This is an offline package
staging mechanism, never a host installer. The Debian system-profile builder uses
this helper. No `/usr/local/bin` installation occurs.

The existing `oid-marina` Debian package identity and `oid` service principal are
retained for package/account compatibility. Unit names `oid-console.service` and
`oid-readiness.service` remain because existing boot dependencies reference them;
their new client payload uses MarinaOS paths. `--oid-console` remains an input
alias for `--console` because existing image tooling uses it; image generation
is outside this change. No legacy executable aliases or copied binaries are added.
Historical image assets and reports retain their original paths and names.

Talvik's separate package, native library identifiers (including
`liboid-chat-template`), model IDs and existing system/user configuration paths
remain unchanged. This client staging change does not migrate installed service
state. Existing environment keys and serialized fields retain their names.
The new client package uses `/etc/marinaos/marina-client.env`; existing
configuration files are not automatically moved or deleted. The client units load
the legacy `/etc/oid/marina-client.env` first, then the new file so explicit new
settings take precedence. Per-user installation defaults to `~/.local/bin` via a `~/.local` Cargo prefix;
legacy client configuration/state overrides remain compatible rather than being
silently relocated. New user configuration should follow XDG conventions.

PPP owns `/usr/sbin/chat` on the qualification machine. The package's distinct
`/usr/bin/chat` does not overwrite it; use that full path if PATH selects PPP.
A conflicting `/usr/bin/chat` from any other package must be resolved by normal
package ownership checks, never forced installation. No console system-package
collision was found; the current `console` is the user-installed MarinaOS client.

Rootless `scripts/install-oid-user.sh` installs the three clients without Cargo's
`--force` and places explicitly supplied native payloads in the client prefix's
`lib/marinaos`; it does not write the existing Talvik home or models.
