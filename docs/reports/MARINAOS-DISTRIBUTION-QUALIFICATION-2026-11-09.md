# MarinaOS distribution qualification status — 2026-11-09

## Results in this workspace

| Gate | Result | Evidence |
| --- | --- | --- |
| Existing implementation and qualification runbook inspected | PASS | `docs/validation/marinaos-linux-image.md`; `docs/reports/MARINAOS-IMAGE-IMPLEMENTATION-2026-11-09.md` |
| Partition tools: parted, sgdisk; filesystem/copy tools: mkfs.ext4, mkfs.vfat, rsync, losetup, mount, chroot | PASS: present | Direct `command -v` inspection in this session |
| debootstrap | NOT TESTED: unavailable | `command -v debootstrap` returned missing |
| qemu-system-x86_64 | NOT TESTED: unavailable | `command -v qemu-system-x86_64` returned missing |
| OVMF code and variables firmware | NOT TESTED: unavailable | Checked `/usr/share/OVMF/OVMF_CODE_4M.fd`, `OVMF_VARS_4M.fd`, and non-4M names; absent |
| Privileged build eligibility | FAIL in this container | `sudo -n true` reports no-new-privileges prevents root elevation |
| Ubuntu 24.04 image build | NOT TESTED | Builder was not run |
| QEMU UEFI boot / login prompt | NOT TESTED | No QEMU or image |
| systemd readiness / service health | NOT TESTED | No guest boot |
| Authenticated login / reboot persistence | NOT TESTED | No guest boot |
| systemd failure/recovery | NOT TESTED | No guest boot |

No host storage, host services, or physical devices were accessed. Earlier
nonprivileged implementation tests are recorded in
`MARINAOS-IMAGE-IMPLEMENTATION-2026-11-09.md`; they are not distribution
qualification evidence.

## Reviewed commands for a separate disposable Ubuntu 24.04 amd64 VM

Run these commands in a VM dedicated to image construction and testing. The
builder requires root for chroot, loop devices and mounts. Its output is a newly
created regular raw file in `/var/tmp`; do not change `--output` to a host block
device or an existing file. Run from the repository checkout.

```bash
sudo apt-get update
sudo apt-get install -y debootstrap ubuntu-keyring qemu-system-x86 ovmf parted gdisk dosfstools e2fsprogs rsync util-linux python3 openssl
command -v debootstrap qemu-system-x86_64 sgdisk parted rsync losetup mount mkfs.ext4 mkfs.vfat chroot
ls -l /usr/share/OVMF/OVMF_CODE_4M.fd /usr/share/OVMF/OVMF_VARS_4M.fd
```

First run the nonprivileged manifest and fixture gates:

```bash
bash scripts/build-marinaos-linux-image.sh --plan
python3 scripts/test-image-rootfs.py
python3 scripts/test-marinaos-image.py
python3 scripts/test-marinaos-builder.py
python3 scripts/test-linux-package-ownership.py
```

Create an administrator hash without placing a cleartext password in shell
history. The command prompts interactively; its output file is mode 0600 and
must remain outside the repository:

```bash
umask 077
openssl passwd -6 > /var/tmp/marinaos-admin.hash
chmod 0600 /var/tmp/marinaos-admin.hash
```

Build only the named new regular image file:

```bash
sudo bash scripts/build-marinaos-linux-image.sh \
  --output /var/tmp/marinaos-noble-amd64.raw \
  --admin-password-hash-file /var/tmp/marinaos-admin.hash
```

The output is root-owned mode 0600. Make this disposable image readable by the
unprivileged QEMU user while keeping it private to that user:

```bash
sudo chown "$(id -u):$(id -g)" /var/tmp/marinaos-noble-amd64.raw
chmod 0600 /var/tmp/marinaos-noble-amd64.raw
```

Run the existing boot gate. It uses TCG, disables networking, gives the guest a
temporary disk overlay, copies OVMF variables to temporary storage, and requires
the serial `marinaos login:` prompt. It records JSON and serial evidence at new
paths; remove/rename prior reports manually before rerunning because overwrite
is rejected.

```bash
python3 scripts/qualify-marinaos-qemu.py \
  --image /var/tmp/marinaos-noble-amd64.raw \
  --ovmf-code /usr/share/OVMF/OVMF_CODE_4M.fd \
  --ovmf-vars /usr/share/OVMF/OVMF_VARS_4M.fd \
  --report /var/tmp/marinaos-boot.json \
  --timeout 300
cat /var/tmp/marinaos-boot.json
```

A passing boot-gate report proves only that QEMU reached the login prompt. For
interactive checks, use a persistent OVMF variables copy and the disposable raw
image itself. This modifies only the dedicated VM's regular image file and its
firmware variables copy. Do not add a physical disk or host block device to QEMU.

```bash
cp --reflink=auto /usr/share/OVMF/OVMF_VARS_4M.fd /var/tmp/marinaos-vars.fd
qemu-system-x86_64 \
  -machine q35 -accel tcg -m 2048 \
  -display none -monitor none -serial mon:stdio -nic none -no-reboot \
  -drive if=pflash,format=raw,readonly=on,file=/usr/share/OVMF/OVMF_CODE_4M.fd \
  -drive if=pflash,format=raw,file=/var/tmp/marinaos-vars.fd \
  -drive if=virtio,format=raw,file=/var/tmp/marinaos-noble-amd64.raw
```

At the guest prompt, record these checks and their complete outputs in the
qualification report:

```text
login: marina-admin
# Enter the password used to generate the hash.
systemctl is-system-running
systemctl is-active multi-user.target
systemctl --failed --no-legend
systemd-analyze time
printf 'marinaos-reboot-persistence-ok\n' | sudo tee /var/tmp/marinaos-persist.marker
sudo systemctl reboot
# After the serial login prompt returns:
grep -Fx marinaos-reboot-persistence-ok /var/tmp/marinaos-persist.marker
systemctl is-system-running
```

For an explicit service failure/recovery check, use a harmless transient unit
inside the guest. Its expected `failed` state tests systemd's reporting; restarting
the same unit tests recovery. It does not alter host services or disk layout.

```bash
sudo systemd-run --unit=marinaos-qualification-failure --wait /bin/false
systemctl is-failed marinaos-qualification-failure.service
sudo systemctl reset-failed marinaos-qualification-failure.service
sudo systemd-run --unit=marinaos-qualification-recovery --wait /bin/true
systemctl is-active marinaos-qualification-recovery.service
```

Stop QEMU with Ctrl-A then X. Capture the QEMU version, image SHA-256, boot JSON,
serial log, command outputs and whether login, reboot persistence and recovery
passed. Record each unavailable gate as NOT TESTED and every observed failure as
FAIL with its console output; do not infer success from image construction. This
sequence does not test installation onto another virtual disk, networking,
Secure Boot, physical hardware or release reproducibility.

## Current claim

Qualification is blocked in this container by missing debootstrap/QEMU/OVMF and
the no-new-privileges restriction. The exact host-side runbook above is ready.
Until actual QEMU evidence is captured, MarinaOS is **not claimed as a bootable
or qualified distribution**.
