# MarinaOS image corrective implementation — 2026-11-09

Outcome: WP-0088 rootfs assembly defect corrected; Ubuntu 24.04 amd64 UEFI/GPT
foundation composition implemented; next unblocked WP-0089 package ownership
guard implemented. **Actual image construction and QEMU boot remain NOT TESTED.**
No bootable distribution or milestone completion is claimed.

The earlier architecture reconciliation was already present as working-tree
changes and is preserved. No Rust runtime, policy, approval, evidence or model
implementation was replaced. OIP and Delivery One remain outside scope.

## Changes and traceability

| Status | Change | Source evidence |
| --- | --- | --- |
| VERIFIED | Populate rootfs with rsync before writing final fstab. Existing bootstrap fstab cannot overwrite UUID entries; permissions, content and symlinks survive. | `scripts/lib/image-rootfs.sh:3`; `scripts/build-oid-linux-image.sh:102`; `scripts/test-image-rootfs.py:13` |
| VERIFIED | Reserve a new mode-0600 image file atomically with O_EXCL; existing files, links and device nodes are rejected. | `scripts/lib/image-rootfs.sh:29`; rootfs reservation test |
| VERIFIED source; real build NOT TESTED | Both builders unmount and detach before hashing. Cleanup stops rather than recursively deleting a still-mounted workspace. Debian model ownership is set inside its chroot instead of requiring a matching host account. | `scripts/build-oid-linux-image.sh:43`, `:89`, `:109`; `scripts/build-marinaos-linux-image.sh:32`, `:108` |
| VERIFIED | Historical manifest now declares the MBR/BIOS layout its builder actually creates. Original Debian builder and integration payload are retained. | `image/manifest.yaml:1`; `scripts/build-oid-linux-image.sh:97` |
| VERIFIED contract; boot NOT TESTED | Separate consumed Ubuntu 24.04 amd64 manifest: UEFI/GPT, 512 MiB ESP, ext4 root, Secure Boot disabled, no inference dependencies. Invalid target/layout/package contracts fail before privileges are needed. | `image/ubuntu-24.04-amd64.json:1`; `scripts/image-plan.py:10` |
| VERIFIED partition integration | Production partition helper creates primary and backup GPT with ESP type GUID, 1 MiB start, 512 MiB ESP and root at 513 MiB. Tested using real parted/sfdisk/sgdisk on a disposable regular file, without mounting or loop devices. | `scripts/lib/image-rootfs.sh:24`; `scripts/test-marinaos-image.py:77` |
| INCOMPLETE qualification | Ubuntu builder installs distro kernel/systemd and ordinary password-protected `marina-admin` administrator; locks root, suppresses package service startup during construction, enables serial login. It writes final identity/GRUB configuration after rootfs copy and installs the UEFI fallback loader with no NVRAM writes. | `scripts/build-marinaos-linux-image.sh:51`, `:63`, `:83`, `:102` |
| VERIFIED fixture behavior | Full builder control-flow fixtures verify final fstab, bootloader flags and detach-before-hash order. Injected unmount failure withholds success checksum/manifest and retains workspace. These fixtures substitute privileged tools; they are not boot proof. | `scripts/test-marinaos-builder.py:46` |
| VERIFIED fixture behavior; QEMU NOT TESTED | New boot gate runs QEMU with no network, disposable disk overlay and private firmware variables. Records serial output, image digest and boot-only status. Kernel output alone fails; login marker is required. | `scripts/qualify-marinaos-qemu.py:19`; `scripts/test-marinaos-image.py` |
| VERIFIED package metadata/lifecycle | Mutual Conflicts prevents oid-marina/talvik overwriting shared commands/unit. Relocated rootless dpkg rejects both installation orders and retains the first service file. No Replaces, automatic removal or data migration added. | `scripts/package-oid-deb.sh:71`; `scripts/package-talvik-deb.sh:52`; `scripts/test-linux-package-ownership.py:28` |
| NOT TESTED in CI | Added Ubuntu 24.04 foundation CI for nonprivileged rootfs/partition/builder/package tests, separate from optional Talvik release workflows. | `.github/workflows/marinaos-foundation.yml:1` |

ADRs 0046, 0050 and 0052 and WP-0088/0089 now record implementation scope.
The user's instruction authorizes UEFI/GPT implementation; the broader proposed
ADRs and release qualification are not silently declared accepted. A new image
profile reuses the existing rootfs boundary and normal distro tools; it does not
add a service manager, package manager or model registry.

## Exact checks and results

Evidence directory: [marinaos-image-2026-11-09](evidence/marinaos-image-2026-11-09/).
Host is Ubuntu 26.04; this is not Ubuntu 24.04 runtime qualification.

| Command | Result | Log |
| --- | --- | --- |
| `python3 scripts/test-image-rootfs.py` | PASS, exit 0, 5 tests | [rootfs](evidence/marinaos-image-2026-11-09/rootfs.txt) |
| `python3 scripts/test-marinaos-image.py` | PASS, exit 0, 7 tests including actual regular-file GPT test | [image](evidence/marinaos-image-2026-11-09/image.txt) |
| `python3 scripts/test-marinaos-builder.py` | PASS, exit 0, 2 stubbed builder integration tests | [builder](evidence/marinaos-image-2026-11-09/builder.txt) |
| `python3 scripts/test-linux-package-ownership.py` | PASS, exit 0, 1 test exercising both dpkg conflict orders | [packages](evidence/marinaos-image-2026-11-09/packages.txt) |
| `python3 scripts/test-talvik-installers.py` | PASS, exit 0, 7 existing regression tests | [installers](evidence/marinaos-image-2026-11-09/installers.txt) |
| `bash scripts/build-marinaos-linux-image.sh --plan` | PASS, exit 0, validated manifest, no privileges | [plan](evidence/marinaos-image-2026-11-09/plan.json) |
| `cargo fmt --all -- --check` | PASS, exit 0 | [fmt](evidence/marinaos-image-2026-11-09/fmt.txt) |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS, exit 0 | [clippy](evidence/marinaos-image-2026-11-09/clippy.txt) |
| `LD_LIBRARY_PATH=/home/danny/.marina/lib cargo test --workspace` | PASS outside sandbox, exit 0; 296 passed, 10 ignored | [workspace](evidence/marinaos-image-2026-11-09/workspace.txt) |
| Individual `sh -n`/`bash -n` on changed shell files; Python `ast.parse` on new scripts | PASS; exact per-file invocations recorded | [syntax](evidence/marinaos-image-2026-11-09/syntax.txt) |
| `git diff --check` | PASS, exit 0 | Documentation/code whitespace check |
| `command -v debootstrap qemu-system-x86_64`; `sudo -n true` | Tools absent; sudo blocked by container no-new-privileges policy. No install attempted. | [tooling](evidence/marinaos-image-2026-11-09/tooling.txt) |

During test development the builder fixture initially omitted the bootstrap
`etc/apt` directory; it was corrected and the tests rerun. The first rootless dpkg
fixture attempted the default host log and root ownership, and failed without
modifying host services. The final fixture explicitly relocates its log and
rebuilds sanitized test packages with fixture UID/GID. All package maintainer
scripts are removed from test copies before dpkg runs. Production packages retain
their original scripts and root-owned payloads.

## Remaining gaps and next dependency

1. Obtain an authenticated disposable amd64 build VM with debootstrap and loop/
   mount capability. This container lacks debootstrap, QEMU and OVMF; no-new-privileges prevents
   elevation, so no actual root image build was attempted.
2. Build the Ubuntu profile, then execute the documented QEMU/OVMF gate. Only a
   real QEMU boot-to-login result can qualify boot. This is the next blocked
   WP-0088 dependency; the runbook is
   [MarinaOS image validation](../validation/marinaos-linux-image.md).
3. Qualify authenticated login, installation onto a disposable virtual disk,
   offline reboot persistence, package upgrades, failure and recovery. The raw
   image builder is not an install-to-target workflow.
4. Pin archive snapshots/package inputs and toolchain, then qualify repeatability,
   signed releases, firmware/hardware compatibility and network setup. The emitted
   package inventory records resolved versions; it does not make a rolling
   archive reproducible. Secure Boot is explicitly unsupported in this slice.
5. WP-0089 automatic transition/data migration between legacy Marina and Talvik
   profiles remains unimplemented; conflict rejection prevents unsafe coexistence
   while preserving the installed package. Native package/systemd qualification
   and the old optional console's HTTP/IPC mismatch remain open.

No host storage was partitioned/formatted, no host services were modified, and
no real privileged image commands were executed. Actual partition changes were
confined to temporary regular image files. Tests of mocked builder and QEMU
commands are identified as fixtures throughout this report.
