#!/usr/bin/env bash
set -euo pipefail

# Build the first Debian/Ubuntu system-profile artifact. This packages the
# existing Talvik and MarinaOS client assets; it does not create a second service
# implementation or bundle model data.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="${VERSION:-0.1.0}"
OUTPUT="${OUTPUT:-$ROOT/artifacts/oid-marina_${VERSION}_amd64.deb}"
TALVIKD=""; TALVIK=""; OID_CONSOLE=""; CHAT=""; ASK=""; NATIVE_LIB_DIR=""

usage() {
  echo "usage: $0 --talvikd PATH --talvik PATH --console PATH --chat PATH --ask PATH --native-lib-dir DIR [--version VERSION] [--output PATH]" >&2
  exit 2
}
while [[ $# -gt 0 ]]; do
  case "$1" in
    --talvikd) TALVIKD="$2"; shift 2;;
    --talvik) TALVIK="$2"; shift 2;;
    --console|--oid-console) OID_CONSOLE="$2"; shift 2;;
    --chat) CHAT="$2"; shift 2;;
    --ask) ASK="$2"; shift 2;;
    --native-lib-dir) NATIVE_LIB_DIR="$2"; shift 2;;
    --version) VERSION="$2"; shift 2;;
    --output) OUTPUT="$2"; shift 2;;
    --help|-h) usage;;
    *) echo "unknown option: $1" >&2; usage;;
  esac
done
[[ -x "$CHAT" && -x "$ASK" && -x "$TALVIKD" && -x "$TALVIK" && -x "$OID_CONSOLE" && -d "$NATIVE_LIB_DIR" ]] || usage
command -v dpkg-deb >/dev/null || { echo "dpkg-deb is required" >&2; exit 1; }
[[ "$VERSION" =~ ^[0-9][0-9A-Za-z.+~:-]*$ ]] || { echo "invalid Debian version: $VERSION" >&2; exit 2; }

for file in "$ROOT/packaging/marina/config.yaml" "$ROOT/packaging/marina/talvikd.service" \
  "$ROOT/packaging/marinaos/readiness.service" "$ROOT/packaging/marinaos/console.service" \
  "$ROOT/packaging/oid/oid-readiness" "$ROOT/packaging/sysusers/marina.conf" \
  "$ROOT/packaging/marinaos/clients-sysusers.conf" "$ROOT/packaging/tmpfiles/marina.conf" \
  "$ROOT/packaging/marinaos/clients-tmpfiles.conf" "$ROOT/packaging/marinaos/clients.env"; do
  [[ -f "$file" ]] || { echo "missing packaging asset: $file" >&2; exit 1; }
done

WORK="$(mktemp -d)"; trap 'rm -rf "$WORK"' EXIT; PKG="$WORK/pkg"
mkdir -p "$PKG/DEBIAN" "$PKG/usr/bin" "$PKG/usr/lib/marina" \
  "$PKG/usr/lib/systemd/system" "$PKG/usr/lib/sysusers.d" "$PKG/usr/lib/tmpfiles.d" \
  "$PKG/usr/libexec" "$PKG/etc/marina"
install -m 0755 "$TALVIKD" "$PKG/usr/bin/talvikd"
install -m 0755 "$TALVIK" "$PKG/usr/bin/talvik"
CLIENT_BIN="$WORK/clients"
mkdir -p "$CLIENT_BIN"
ln -s "$(realpath "$OID_CONSOLE")" "$CLIENT_BIN/console"
ln -s "$(realpath "$CHAT")" "$CLIENT_BIN/chat"
ln -s "$(realpath "$ASK")" "$CLIENT_BIN/ask"
DESTDIR="$PKG" bash "$ROOT/scripts/stage-marinaos-clients.sh" "$CLIENT_BIN" "$NATIVE_LIB_DIR"
find "$NATIVE_LIB_DIR" -maxdepth 1 \( -type f -o -type l \) \
  \( -name 'libllama.so*' -o -name 'libggml*.so*' -o -name 'libmtmd.so*' -o -name 'liboid-chat-template.so*' \) -exec cp -a {} "$PKG/usr/lib/marina/" \;
[[ -e "$PKG/usr/lib/marina/libllama.so" || -e "$PKG/usr/lib/marina/libllama.so.0" ]] || {
  echo "native payload does not contain libllama.so" >&2; exit 1;
}
for component in ggml ggml-base ggml-cpu mtmd oid-chat-template; do
  [[ -n "$(find "$PKG/usr/lib/marina" -maxdepth 1 -type f -name "lib${component}.so*" -print -quit)" ]] || {
    echo "native payload does not contain lib${component}.so" >&2; exit 1;
  }
done
install -m 0644 "$ROOT/packaging/marina/config.yaml" "$PKG/etc/marina/config.yaml"

install -m 0644 "$ROOT/packaging/marina/talvikd.service" "$PKG/usr/lib/systemd/system/"

install -m 0644 "$ROOT/packaging/sysusers/marina.conf" "$PKG/usr/lib/sysusers.d/"

install -m 0644 "$ROOT/packaging/tmpfiles/marina.conf" "$PKG/usr/lib/tmpfiles.d/"

install -m 0755 "$ROOT/scripts/install-oid-image-model.sh" "$PKG/usr/libexec/install-oid-image-model"
cat >"$PKG/DEBIAN/control" <<EOF
Package: oid-marina
Version: $VERSION
Conflicts: talvik
Section: misc
Priority: optional
Architecture: amd64
Maintainer: MarinaOS contributors
Depends: libc6 (>= 2.35), systemd | systemd-sysv
Description: MarinaOS governed runtime and Talvik local model runner
 Versioned native Linux runtime containing Talvik, the MarinaOS console, and the
 pinned relocatable llama.cpp backend. Models, evidence, and mutable state are
 intentionally outside the package payload.
EOF
cat >"$PKG/DEBIAN/conffiles" <<'EOF'
/etc/marina/config.yaml
/etc/marinaos/marina-client.env
EOF
cat >"$PKG/DEBIAN/postinst" <<'EOF'
#!/bin/sh
set -eu
if command -v systemd-sysusers >/dev/null 2>&1; then systemd-sysusers || true; fi
if command -v systemd-tmpfiles >/dev/null 2>&1; then systemd-tmpfiles --create || true; fi
if command -v systemctl >/dev/null 2>&1; then systemctl daemon-reload || true; fi
exit 0
EOF
cat >"$PKG/DEBIAN/postrm" <<'EOF'
#!/bin/sh
set -eu
if command -v systemctl >/dev/null 2>&1; then systemctl daemon-reload || true; fi
exit 0
EOF
chmod 0755 "$PKG/DEBIAN/postinst" "$PKG/DEBIAN/postrm"
mkdir -p "$(dirname "$OUTPUT")"
dpkg-deb --build --root-owner-group "$PKG" "$OUTPUT" >/dev/null
sha256sum "$OUTPUT" >"$OUTPUT.sha256"
printf '%s\n' "created $OUTPUT" "checksum $(cat "$OUTPUT.sha256")"
