# MarinaOS first image: recovery and qualification record

Status at this checkpoint: **IN PROGRESS**. The first offline UEFI run observed
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

The 2026-11-10 preflight found 52 GiB available on the development filesystem,
no `/dev/kvm`, and QEMU 8.2.2 plus matching 4 MiB OVMF code/variables files in
the existing `marinaos-qemu-tools:local` container. The manifest plan and the
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
container. The running qualification uses `--network none`, checked the
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
observed per-step outcomes remain pending at this checkpoint.

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

At this checkpoint, Ubuntu archive metadata signature validation passed and the
rebuild is still downloading and validating packages. The changed image is not
yet built or boot-tested.

The exact host command that queued the run was:

```sh
docker run --rm --network none \
  -v /home/danny/codes/marina-os:/source:ro \
  -v /home/danny/codes/marina-os/.marinaos-vm-work:/work \
  marinaos-qemu-tools:local bash /work/run-resumed-qualification.sh
```
