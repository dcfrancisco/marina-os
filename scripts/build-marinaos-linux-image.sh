#!/bin/bash
# Ubuntu foundation builder; execute only in a disposable privileged build VM.
set -euo pipefail
SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "$SCRIPT_DIR/lib/image-rootfs.sh"
manifest="$SCRIPT_DIR/../image/ubuntu-24.04-amd64.json"
output= password_file= core_deb= cli_deb= plan_only=false
while (($#)); do
    case "$1" in
        --manifest) manifest=${2:?}; shift 2 ;;
        --output) output=${2:?}; shift 2 ;;
        --admin-password-hash-file) password_file=${2:?}; shift 2 ;;
        --core-deb) core_deb=${2:?}; shift 2 ;;
        --cli-deb) cli_deb=${2:?}; shift 2 ;;
        --plan) plan_only=true; shift ;;
        *) echo 'usage: build-marinaos-linux-image.sh [--manifest FILE] --plan | --output NEW.raw --admin-password-hash-file FILE --core-deb FILE --cli-deb FILE' >&2; exit 2 ;;
    esac
done
python3 "$SCRIPT_DIR/image-plan.py" "$manifest" >/dev/null
if "$plan_only"; then exec python3 "$SCRIPT_DIR/image-plan.py" "$manifest"; fi
[[ -n "$output" && -f "$password_file" ]] || { echo 'new output and administrator password hash file required' >&2; exit 2; }
[[ ! -e "$output" && ! -L "$output" ]] || { echo 'refusing existing output or symlink' >&2; exit 2; }
for package in "$core_deb" "$cli_deb"; do
    [[ -f "$package" && ! -L "$package" ]] || { echo 'core and CLI DEB regular files required' >&2; exit 2; }
done
# Never embed a universal password or enable automatic privileged login.
password_hash=$(cat "$password_file")
[[ "$password_hash" =~ ^\$(6|y)\$[^[:space:]:]+$ ]] || { echo 'expected one SHA-512 or yescrypt password hash' >&2; exit 2; }
for tool in debootstrap rsync parted losetup mount umount mountpoint mkfs.ext4 mkfs.vfat blkid chroot sha256sum truncate dpkg-deb; do
    command -v "$tool" >/dev/null || { echo "missing build tool: $tool" >&2; exit 1; }
done
[[ $(id -u) == 0 && $(uname -m) == x86_64 ]] || { echo 'requires root in an amd64 disposable build VM' >&2; exit 1; }
size=$(python3 "$SCRIPT_DIR/image-plan.py" "$manifest" --field image_size_mib)
packages=$(python3 "$SCRIPT_DIR/image-plan.py" "$manifest" --field packages)
mirror=$(python3 "$SCRIPT_DIR/image-plan.py" "$manifest" --field mirror)
work=$(mktemp -d)
root="$work/root"; image_root="$work/image-root"; loop=
mounts=()
cleanup_mounts() {
    local i
    for ((i=${#mounts[@]}-1; i>=0; i--)); do
        umount "${mounts[i]}" || return 1
        unset 'mounts[i]'
    done
    if [[ -n "$loop" ]]; then losetup -d "$loop" || return 1; loop=; fi
}
cleanup() {
    if cleanup_mounts; then rm -rf -- "$work";
    else echo "cleanup incomplete: retained $work and loop $loop; no success manifest" >&2; fi
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
mount_at() { local destination=${!#}; mount "$@"; mounts+=("$destination"); }
mkdir -p "$root" "$image_root"
# Stage the exact package inputs before validation and record their digests.
cp -- "$core_deb" "$work/marinaos-core.deb"
cp -- "$cli_deb" "$work/marinaos-cli.deb"
for name in marinaos-core marinaos-cli; do
    package="$work/$name.deb"
    [[ $(dpkg-deb -f "$package" Package) == "$name" && $(dpkg-deb -f "$package" Architecture) == amd64 ]] || {
        echo "invalid package identity or architecture: $name" >&2; exit 1;
    }
done
core_version=$(dpkg-deb -f "$work/marinaos-core.deb" Version)
[[ -n "$core_version" && "$core_version" == "$(dpkg-deb -f "$work/marinaos-cli.deb" Version)" ]] || {
    echo 'core and CLI package versions must match' >&2; exit 1;
}
(cd "$work" && sha256sum marinaos-core.deb marinaos-cli.deb) >"$work/marinaos-package-sha256.txt"
debootstrap --arch=amd64 --variant=minbase noble "$root" "$mirror"
# Block package scripts from starting services during construction.
printf '#!/bin/sh\nexit 101\n' >"$root/usr/sbin/policy-rc.d"
chmod 0755 "$root/usr/sbin/policy-rc.d"
mount_at --bind /dev "$root/dev"
mount_at -t proc proc "$root/proc"
mount_at -t sysfs sysfs "$root/sys"
printf 'deb %s noble main restricted universe\ndeb %s noble-updates main restricted universe\ndeb https://security.ubuntu.com/ubuntu noble-security main restricted universe\n' "$mirror" "$mirror" >"$root/etc/apt/sources.list"
chroot "$root" apt-get update
# Package names are validated tokens, not shell commands.
read -r -a package_list <<<"$packages"
chroot "$root" env DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends "${package_list[@]}"
mkdir -p "$root/tmp"
cp -- "$work/marinaos-core.deb" "$root/tmp/marinaos-core.deb"
cp -- "$work/marinaos-cli.deb" "$root/tmp/marinaos-cli.deb"
chroot "$root" env DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends /tmp/marinaos-core.deb /tmp/marinaos-cli.deb
rm -- "$root/tmp/marinaos-core.deb" "$root/tmp/marinaos-cli.deb"
chroot "$root" systemctl enable marinaosd.service
chroot "$root" useradd --create-home --shell /bin/bash --groups sudo,marinaos-readers marina-admin
printf 'marina-admin:%s\n' "$password_hash" | chroot "$root" chpasswd --encrypted
unset password_hash
chroot "$root" passwd --lock root
chroot "$root" dpkg-query -W '-f=${Package}\t${Version}\t${Architecture}\n' >"$work/packages.txt"
chroot "$root" apt-get clean
cleanup_mounts
rm "$root/usr/sbin/policy-rc.d"
# Reserve a *new regular file* atomically. Devices and symlinks are never accepted.
mkdir -p "$(dirname -- "$output")"
reserve_image_file "$output"
truncate -s "${size}M" "$output"
partition_uefi_image "$output"
loop=$(losetup --find --show --partscan "$output")
[[ "$loop" =~ ^/dev/loop[0-9]+$ ]] || { echo 'unexpected loop device' >&2; exit 1; }
mkfs.vfat -F 32 "${loop}p1"
mkfs.ext4 -F "${loop}p2"
mount_at "${loop}p2" "$image_root"
root_uuid=$(blkid -s UUID -o value "${loop}p2")
esp_uuid=$(blkid -s UUID -o value "${loop}p1")
assemble_image_rootfs "$root" "$image_root" "$root_uuid" "$esp_uuid"
# All image-specific configuration follows the final root copy.
printf 'marinaos\n' >"$image_root/etc/hostname"
printf '127.0.0.1 localhost\n127.0.1.1 marinaos\n::1 localhost ip6-localhost\n' >"$image_root/etc/hosts"
printf 'MarinaOS foundation preview (Ubuntu 24.04)\\n\\l\n' >"$image_root/etc/issue"
: >"$image_root/etc/machine-id"
mkdir -p "$image_root/boot/efi" "$image_root/etc/default/grub.d"
cat >"$image_root/etc/default/grub.d/50-marinaos.cfg" <<'GRUB'
GRUB_TIMEOUT=1
GRUB_TERMINAL="console serial"
GRUB_SERIAL_COMMAND="serial --unit=0 --speed=115200"
GRUB_CMDLINE_LINUX="console=tty0 console=ttyS0,115200n8"
GRUB_DISABLE_OS_PROBER=true
GRUB
mount_at "${loop}p1" "$image_root/boot/efi"
mount_at --bind /dev "$image_root/dev"
mount_at -t proc proc "$image_root/proc"
mount_at -t sysfs sysfs "$image_root/sys"
# Removable fallback path boots without registering entries in host firmware.
chroot "$image_root" grub-install --target=x86_64-efi --efi-directory=/boot/efi --bootloader-id=MarinaOS --removable --no-nvram
chroot "$image_root" update-initramfs -u -k all
chroot "$image_root" grub-mkconfig -o /boot/grub/grub.cfg
chroot "$image_root" systemctl enable serial-getty@ttyS0.service
[[ -s "$image_root/boot/efi/EFI/BOOT/BOOTX64.EFI" ]]
sync
cleanup_mounts
# Evidence is emitted only after all filesystems are unmounted and loop detached.
cp "$manifest" "$output.inputs.json"
cp "$work/packages.txt" "$output.packages.txt"
cp "$work/marinaos-package-sha256.txt" "$output.marinaos-packages.sha256"
sha256sum "$output" >"$output.sha256"
printf 'schema=marinaos.image-build/v1\nprofile=ubuntu-24.04-amd64-uefi-gpt\nboot_qualification=not-tested\nsecure_boot=false\n' >"$output.manifest"
sha256sum "$output.inputs.json" "$output.packages.txt" "$output.marinaos-packages.sha256" >>"$output.manifest"
printf 'created %s; QEMU boot NOT TESTED\n' "$output"
