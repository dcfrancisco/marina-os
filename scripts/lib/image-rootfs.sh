# Shared image assembly. Source this file; it performs no operations on load.
# Destination must be the caller's freshly mounted disposable image filesystem.
assemble_image_rootfs() {
    source_root=$1
    destination_root=$2
    filesystem_uuid=$3
    esp_uuid=${4:-}
    case "$filesystem_uuid:$esp_uuid" in
        *[!a-zA-Z0-9:-]*|:*) echo 'invalid filesystem UUID' >&2; return 1 ;;
    esac
    # Copy first: a bootstrap tree may contain an empty or stale fstab.
    # Image-specific files are deliberately written only after this copy.
    rsync -aHAX "$source_root/" "$destination_root/" || return 1
    mkdir -p "$destination_root/etc" || return 1
    {
        printf 'UUID=%s / ext4 defaults 0 1\n' "$filesystem_uuid"
        if [ -n "$esp_uuid" ]; then
            printf 'UUID=%s /boot/efi vfat umask=0077 0 1\n' "$esp_uuid"
        fi
    } >"$destination_root/etc/fstab"
}

# Only a caller-created regular image file is accepted. Never a block device.
partition_uefi_image() {
    [ -f "$1" ] && [ ! -L "$1" ] || { echo 'expected regular image file' >&2; return 1; }
    parted -s "$1" mklabel gpt mkpart ESP fat32 1MiB 513MiB set 1 esp on mkpart root ext4 513MiB 100%
}

reserve_image_file() {
    # O_EXCL rejects existing regular files, dangling links and device nodes.
    python3 -c 'import os, sys; os.close(os.open(sys.argv[1], os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600))' "$1"
}
