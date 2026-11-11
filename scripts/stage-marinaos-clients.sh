#!/usr/bin/env bash
# Stage MarinaOS clients without touching host services or Talvik installations.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DESTDIR="${DESTDIR:?set DESTDIR to a package staging root}"
BIN_DIR="${1:?usage: DESTDIR=ROOT bash stage-marinaos-clients.sh BIN_DIR NATIVE_LIB_DIR}"
LIB_DIR="${2:?native library directory required}"
DESTDIR="$(realpath -m "$DESTDIR")"
[[ "$DESTDIR" != / ]] || { echo "refusing to stage into the host root" >&2; exit 2; }
[[ -d "$LIB_DIR" ]] || { echo "native library directory not found: $LIB_DIR" >&2; exit 1; }
for name in console chat ask; do
  [[ -x "$BIN_DIR/$name" ]] || { echo "missing executable: $name" >&2; exit 1; }
  [[ ! -e "$DESTDIR/usr/bin/$name" && ! -L "$DESTDIR/usr/bin/$name" ]] || { echo "refusing to overwrite existing command: $name" >&2; exit 1; }
done
install -d "$DESTDIR/usr/bin" "$DESTDIR/usr/lib/marinaos" "$DESTDIR/usr/share/marinaos" \
  "$DESTDIR/etc/marinaos" "$DESTDIR/usr/lib/systemd/system" "$DESTDIR/usr/lib/tmpfiles.d" "$DESTDIR/usr/lib/sysusers.d"
for name in console chat ask; do install -m 0755 "$BIN_DIR/$name" "$DESTDIR/usr/bin/$name"; done
find "$LIB_DIR" -maxdepth 1 \( -type f -o -type l \) \( -name 'libllama.so*' -o -name 'libggml*.so*' -o -name 'libmtmd.so*' -o -name 'liboid-chat-template.so*' \) -exec cp -a {} "$DESTDIR/usr/lib/marinaos/" \;
install -m 0755 "$ROOT/packaging/oid/oid-readiness" "$DESTDIR/usr/lib/marinaos/readiness"
install -m 0644 "$ROOT/packaging/marinaos/console.service" "$DESTDIR/usr/lib/systemd/system/oid-console.service"
install -m 0644 "$ROOT/packaging/marinaos/readiness.service" "$DESTDIR/usr/lib/systemd/system/oid-readiness.service"
install -m 0644 "$ROOT/packaging/marinaos/clients-tmpfiles.conf" "$DESTDIR/usr/lib/tmpfiles.d/marinaos-clients.conf"
install -m 0640 "$ROOT/packaging/marinaos/clients.env" "$DESTDIR/etc/marinaos/marina-client.env"
install -m 0644 "$ROOT/README.md" "$DESTDIR/usr/share/marinaos/README.md"
install -m 0644 "$ROOT/packaging/marinaos/clients-sysusers.conf" "$DESTDIR/usr/lib/sysusers.d/marinaos-clients.conf"
