#!/usr/bin/env bash
set -euo pipefail

# Rootless MarinaOS client installation. Talvik remains independently managed.
# Native payloads go into this client prefix, never the running Talvik home.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PREFIX="${OID_PREFIX:-${XDG_BIN_HOME:-${HOME}/.local}}"
MARINA_HOME="${MARINA_HOME:-${HOME}/.marina}"
NATIVE_LIB_DIR="${LLAMA_CPP_LIB_DIR:-}"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --prefix) PREFIX="$2"; shift 2;;
    --marina-home) MARINA_HOME="$2"; shift 2;;
    --native-lib-dir) NATIVE_LIB_DIR="$2"; shift 2;;
    --help|-h)
      echo "Usage: $0 [--prefix DIR] [--marina-home DIR] [--native-lib-dir DIR]"
      exit 0
      ;;
    *) echo "unknown option: $1" >&2; exit 2;;
  esac
done
[[ -d "$ROOT/console" ]] || { echo "console source tree not found: $ROOT/console" >&2; exit 1; }
command -v cargo >/dev/null || { echo "cargo is required" >&2; exit 1; }
# Cargo refuses commands owned by another package; never force an overwrite.
if [[ -n "$NATIVE_LIB_DIR" ]]; then
  [[ -d "$NATIVE_LIB_DIR" ]] || { echo "native library directory not found: $NATIVE_LIB_DIR" >&2; exit 1; }
  export LLAMA_CPP_LIB_DIR="$NATIVE_LIB_DIR"
fi
cargo install --path "$ROOT/console" --root "$PREFIX" --locked --bin console --bin ask
if [[ -n "$NATIVE_LIB_DIR" ]]; then
  mkdir -p "$PREFIX/lib/marinaos"
  find "$NATIVE_LIB_DIR" -maxdepth 1 \( -type f -o -type l \) \( \
    -name 'libllama.*' -o -name 'libggml*.*' -o -name 'libmtmd.*' -o -name 'liboid-chat-template.*' \) -exec cp -a {} "$PREFIX/lib/marinaos/" \;
fi
printf '%s\n' "installed MarinaOS clients: $PREFIX/bin/{console,ask}" \
  "Talvik remains independently managed in $MARINA_HOME."
