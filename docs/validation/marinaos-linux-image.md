# MarinaOS Ubuntu foundation image

WP-0088 implementation preview, 2026-11-09. Target: Ubuntu 24.04 LTS (`noble`),
amd64, UEFI/GPT, Secure Boot disabled. A disposable build VM reported creating an
image, and the transferred image matched the guest's recorded SHA-256. Actual
boot-to-login was observed; the complete lifecycle gate remains in progress, as
recorded in the [recovery record](../reports/MARINAOS-FIRST-BOOT-2026-11-10.md).
It is not an installer
or a qualified distribution. The Debian BIOS prototype remains available through
`build-oid-linux-image.sh` and its corrected MBR manifest.

## Nonprivileged checks

```sh
bash scripts/build-marinaos-linux-image.sh --plan
python3 scripts/test-image-rootfs.py
python3 scripts/test-marinaos-image.py
python3 scripts/test-marinaos-builder.py
python3 scripts/test-linux-package-ownership.py
```

The manifest `image/ubuntu-24.04-amd64.json` is consumed and validated before
privileged operations. It selects Ubuntu, GPT, a 512 MiB FAT32 ESP beginning at
1 MiB, and an ext4 root beginning at 513 MiB. Firmware and partition scheme
mismatches are rejected. Package versions currently come from Ubuntu archives;
the builder records the resolved package inventory. This is provenance, **not
bit-for-bit reproducibility**. Snapshot/date pinning remains a release gate.

Rootfs tests copy real temporary trees with rsync. GPT integration runs real
parted/sfdisk/sgdisk only against a newly created disposable regular file. Builder
control-flow tests substitute all privileged tools and do not demonstrate a boot.
The package test uses fake native payloads and strips maintainer scripts before
rootless dpkg operations in a temporary root. Its purpose is ownership conflict
rejection, not binary or systemd qualification.

## Build in a disposable amd64 Linux VM

Install `debootstrap`, `ubuntu-keyring`, `parted`, `dosfstools`, `e2fsprogs`,
`rsync`, `util-linux`, `python3` and normal shell/core utilities in that VM.
On Ubuntu, an operator can install the build and QEMU prerequisites with:

```sh
sudo apt-get update
sudo apt-get install debootstrap ubuntu-keyring parted dosfstools e2fsprogs rsync util-linux python3 qemu-system-x86 ovmf
```

The build requires root there for chroot, temporary mounts and loop devices.
The output must be a new file; existing files, links and device nodes are rejected.
Do not run the builder against a host disk or reuse an existing output path.

Provision a private file containing one SHA-512 or yescrypt password hash for the
image's `marina-admin` administrator. For example, `openssl passwd -6` prompts for the
password; redirect its output into a mode-0600 file outside the repository.
The administrator name is distinct from the legacy `marina` service account.
No default password or automatic login is installed. The root account is locked;
`marina-admin` uses ordinary console login and sudo with its password. Treat the hash
and built image as sensitive deployment artifacts.

```sh
sudo bash scripts/build-marinaos-linux-image.sh \
  --output /var/tmp/marinaos-noble-amd64.raw \
  --admin-password-hash-file /private/path/admin.hash \
  --core-deb /private/path/marinaos-core_0.1.0_amd64.deb \
  --cli-deb /private/path/marinaos-cli_0.1.0_amd64.deb
```

The image includes Linux/systemd, an ordinary serial login, local diagnostics
and package tools. It installs the supplied matching amd64 MarinaOS core and CLI
packages, enables `marinaosd`, and puts `marina-admin` in `marinaos-readers`.
It includes no Talvik, models or OID console. The input package hashes are
recorded alongside the image.
Networking configuration is not yet qualified. Service startup is suppressed
while packages are installed. Final fstab, hostname, machine identity reset,
GRUB serial configuration and ESP contents follow the root filesystem copy.
GRUB installs `EFI/BOOT/BOOTX64.EFI` with `--removable --no-nvram`; no host
firmware entry is created. The builder unmounts filesystems and detaches its
loop device before publishing checksums, input manifest and package inventory.
On unmount failure it retains the workspace and withholds success evidence.

## Actual UEFI boot gate

Install `qemu-system-x86` and `ovmf` in the test environment. Choose a matching
non-Secure-Boot OVMF code/variables pair from that installation. Run as an
ordinary user with read access to the image and firmware:

```sh
python3 scripts/qualify-marinaos-qemu.py \
  --image /var/tmp/marinaos-noble-amd64.raw \
  --ovmf-code /usr/share/OVMF/OVMF_CODE_4M.fd \
  --ovmf-vars /usr/share/OVMF/OVMF_VARS_4M.fd \
  --report /var/tmp/marinaos-boot.json \
  --timeout 300
```

The harness uses TCG, no network, a disposable disk overlay and a private copy of
firmware variables. It records serial output and the base image digest. Only an
observed `marinaos login:` prompt passes this **boot-to-login** gate. Kernel output,
a timeout or an early QEMU exit fail. Fixture tests of the harness are not QEMU
qualification. The report explicitly leaves authenticated login, installation and
reboot persistence untested. Complete those, package upgrade/recovery, firmware
matrix and signed reproducible release inputs before claiming a distribution.

For a disposable image built with a private test password file, add `--full
--password-file /private/path/admin-password` to use a temporary persistent
overlay and verify authenticated login, systemd readiness, service and CLI,
service recovery, shutdown and a second boot with a persistence check. The
overlay and firmware variables are created under the report directory; put it
on the development SSD. The password is read from a private file, never passed
on the kernel command line or intentionally written to serial evidence. This
mode records each observed step and must not be treated as passing if it fails
or times out. It does not qualify installation onto a target disk.

## Ubuntu M0 package qualification

The [WP-0089 qualification record](../work-packages/WP-0089-oid-packaged-linux-environment.md)
separates Ubuntu container evidence from the still-unmet VM and boot gates.
The build harness `scripts/qualify-marinaos-build-container.sh` expects a clean
Ubuntu 24.04 amd64 container with read-only `/source`, `/toolchain`, and
`/cargo-cache/registry`, plus an empty writable `/output`. It installs build
prerequisites in the disposable container, builds release binaries and DEBs,
and emits ABI, package and hash evidence. The 0.1.1 artifact is a version-only
fixture for the upgrade test, not a new release. No host compiler or package
installation is required on the target.

`scripts/marinaos-qualification.Dockerfile` prepares only the disposable test
harness. Mount generated packages read-only at `/packages`, mount
`scripts/test-marinaos-ubuntu-container.sh` read-only, and run it with Docker
`--network none` and `MARINAOS_SKIP_APT=1`. The test manually starts the daemon;
it is not systemd boot qualification. For actual boot, service recovery and
running-service upgrade, use a disposable VM as described above. Never point the
builder or qualification process at a physical block device.
