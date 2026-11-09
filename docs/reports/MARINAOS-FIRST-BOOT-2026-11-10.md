# MarinaOS first image: recovery and qualification record

Status at this checkpoint: **STOPPED at the user's request**. The first offline UEFI run observed
kernel startup, the serial login prompt and a successful administrator login,
but the full lifecycle gate failed. It does not qualify installation, an ISO,
the milestone or a release.

The existing Ubuntu 24.04 amd64 manifest specifies a 6144 MiB regular-file
UEFI/GPT disk. The disposable Ubuntu build VM's captured builder log reports
installation of `marinaos-core` and `marinaos-cli` 0.1.0, enabling the service,
installing UEFI GRUB, and creating `/home/builder/marinaos.raw`. The VM disk and
its image are regular virtual-disk files on the development SSD. This log is
evidence of what the builder reported, not proof of a successful guest boot.

Local private working evidence is under `.marinaos-vm-work/`, excluded from Git:
`build-vm.build.log`, the image and input/package inventories, `admin-password`,
and recovery logs. The password and disk image are sensitive test artifacts and
must not be committed. Transfer completed and the host image matched the guest's
recorded SHA-256:
`827fdc4aab5b12e42bc2e840e4dc2d6e58381057ad34b4ea3f87fd750f79b45b`.

The 2026-11-10 preflight found 52 GiB available on the development filesystem
and no `/dev/kvm` visible in the sandbox. A later host check found AMD-V, loaded
KVM modules and `/dev/kvm`, but the current user lacked read/write access, so
the build and qualification used TCG. QEMU 8.2.2 and matching 4 MiB OVMF
code/variables files are in the existing `marinaos-qemu-tools:local` container.
The manifest plan and the
builder and image fixture suites passed; these do not prove VM boot. The local
QEMU tools container lacks `xorriso`, `mksquashfs`, `grub-mkrescue` and
`grub-mkstandalone`; ISO construction is deferred until disk boot passes.

The guest build command, recorded in `.marinaos-vm-work/run-build-vm.sh`, was:

```sh
sudo bash /home/builder/source/scripts/build-marinaos-linux-image.sh \
  --output /home/builder/marinaos.raw \
  --admin-password-hash-file /home/builder/admin.hash \
  --core-deb /home/builder/core.deb --cli-deb /home/builder/cli.deb
```

The nonprivileged checks run at this checkpoint were:

```sh
bash scripts/build-marinaos-linux-image.sh --plan
python3 scripts/test-marinaos-image.py
python3 scripts/test-marinaos-builder.py
python3 -m py_compile scripts/marinaos_lifecycle.py scripts/qualify-marinaos-qemu.py
cargo fmt --all -- --check
```

All exited zero; the two Python test suites reported 11 and 3 tests,
respectively. They do not establish boot or service qualification.

The resumed transfer used `.marinaos-vm-work/resume-transfer.sh` inside that
container. The first qualification used `--network none`, checked the
image's recorded SHA-256 first, then invokes:

```sh
python3 /source/scripts/qualify-marinaos-qemu.py \
  --image /work/marinaos.raw \
  --ovmf-code /usr/share/OVMF/OVMF_CODE_4M.fd \
  --ovmf-vars /usr/share/OVMF/OVMF_VARS_4M.fd \
  --report /work/lifecycle-resumed.json --timeout 1200 --full \
  --password-file /work/admin-password
```

The container mounts the repository read-only at `/source` and the private SSD
workspace at `/work`. The harness uses TCG and a disposable persistent overlay,
and records a separate serial log. Its report, serial output, image digest and
the observed per-step outcomes appear below.

The first run's `lifecycle-resumed.json` and `lifecycle-resumed.serial.txt`
recorded **PASS** for the kernel and boot-to-login checks and **FAIL** for the
automated login check, which timed out. The serial log shows the authenticated
administrator shell and systemd reporting `marinaosd.service` started. The
client, recovery, shutdown, reboot and persistence checks were not run. The
harness sent its first command immediately after the password; the absence of
that command in the serial log and the subsequently displayed shell prompt
indicate input was sent before shell readiness. The harness now waits for that
prompt. Its focused suite passes 12 tests, including a delayed-shell fixture.
The second run used `lifecycle-shell-ready.json`: kernel, boot and login passed,
but systemd readiness failed because `systemctl` reported `Failed to connect to
bus: No such file or directory`. Its serial log is
`lifecycle-shell-ready.serial.txt`. The manifest did not include D-Bus, the
likely cause; the Ubuntu `dbus` package was added and a new image rebuild was
started. Service API, recovery, shutdown, reboot and persistence remain
untested. The original image and both failed-run reports are retained.

The rebuild uses `.marinaos-vm-work/rebuild-dbus.sh`; it confirmed 9.0 GiB free
inside the 14 GiB disposable build VM, uses a new regular-file output, and
captures `rebuild-dbus.build.log` and `rebuild-dbus.serial.txt`. Its host command
is:

```sh
docker run --rm -v /home/danny/codes/marina-os:/source:ro \
  -v /home/danny/codes/marina-os/.marinaos-vm-work:/work \
  marinaos-qemu-tools:local bash /work/rebuild-dbus.sh
```

Ubuntu archive metadata signature validation and base-system installation passed.
The rebuild was stopped while apt/dpkg was configuring image packages. The changed
image was not completed or boot-tested. The waiting `lifecycle-dbus` qualification
was stopped before it started; Docker timed out its graceful stop and terminated
its waiting container. No new lifecycle report was produced.

## Stop and resume checkpoint — 2026-11-10

The builder was paused before its ordinary cleanup could delete the incomplete
root filesystem. The apt/dpkg transaction was sent termination, its child exited,
and the three guest mounts (`root/sys`, `root/proc`, `root/dev`) were unmounted.
The workspace was moved to `/home/builder/marinaos-build-interrupted-20261110`
inside `.marinaos-vm-work/build-vm.qcow2`; its dpkg transaction was interrupted
and must be audited before any reuse. The builder was then terminated and the
guest was shut down. The builder emitted an expected cleanup warning for its old
temporary path because the workspace had deliberately been moved and unmounted.
Guest inspection before shutdown found no workspace mounts or loop devices. After
shutdown, the identified builder and qualification containers and their QEMU
processes were absent; host mount and loop-device inspection found no associations
with this build. Existing unrelated host loop devices were left alone.

The original `marinaos.raw`, its checksum, package files and both lifecycle
reports and serial logs remain under `.marinaos-vm-work/`. No completed ISO or
corrected `marinaos-db.raw` was produced. The development SSD had 49 GiB free
at shutdown. No binary image or private credential is committed.

The following commands later start a **copy** of the stopped build VM for
inspection only, without replacing the preserved disk or starting another build.
They require the existing local tools container and private working files. Run
from the repository root; wait for the VM to reach SSH before the inspection.

```sh
test ! -e .marinaos-vm-work/build-vm-resume.qcow2
cp --no-clobber --sparse=always .marinaos-vm-work/build-vm.qcow2 .marinaos-vm-work/build-vm-resume.qcow2
docker run -d --name marinaos-resume-inspect --network none \
  -v /home/danny/codes/marina-os/.marinaos-vm-work:/work \
  --entrypoint qemu-system-x86_64 marinaos-qemu-tools:local \
  -machine q35 -accel tcg -cpu max -smp 3 -m 3072 \
  -display none -monitor none -serial file:/work/resume-inspect.serial.txt \
  -drive if=virtio,format=qcow2,file=/work/build-vm-resume.qcow2 \
  -drive if=virtio,media=cdrom,format=raw,readonly=on,file=/work/seed.iso \
  -nic user,model=virtio-net-pci,hostfwd=tcp:127.0.0.1:2222-:22
docker exec marinaos-resume-inspect ssh -i /work/builder-key -p 2222 \
  -o BatchMode=yes -o StrictHostKeyChecking=yes -o UserKnownHostsFile=/work/known_hosts \
  builder@127.0.0.1 'sudo chroot /home/builder/marinaos-build-interrupted-20261110/root dpkg --audit'
docker exec marinaos-resume-inspect ssh -i /work/builder-key -p 2222 \
  -o BatchMode=yes -o StrictHostKeyChecking=yes -o UserKnownHostsFile=/work/known_hosts \
  builder@127.0.0.1 'sudo systemctl poweroff'
```

The existing builder always creates a fresh root filesystem and does **not**
resume this interrupted transaction. Resumption requires auditing and repairing
the copied rootfs and adding a tested continuation to the existing builder, or
performing a new build; neither has been done. Prefer native amd64 rootfs work
with retained package caches in an appropriately isolated environment for future
builds, using QEMU for qualification. The present rootless Docker setup cannot be
assumed to provide the mount, chroot and loop-device access the builder needs.
The full disk lifecycle checks, D-Bus fix verification, ISO construction and ISO
qualification remain blocked on completing a corrected image. M0 is not accepted.

The exact host command that queued the run was:

```sh
docker run --rm --network none \
  -v /home/danny/codes/marina-os:/source:ro \
  -v /home/danny/codes/marina-os/.marinaos-vm-work:/work \
  marinaos-qemu-tools:local bash /work/run-resumed-qualification.sh
```
