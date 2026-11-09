#!/bin/sh
# Execute only in a disposable Ubuntu 24.04 amd64 container. /source,
# /toolchain and /cargo-cache/registry are read-only inputs; /output is new.
set -eu
. /etc/os-release
[ "$ID" = ubuntu ] && [ "$VERSION_ID" = 24.04 ] && [ "$(dpkg --print-architecture)" = amd64 ] || {
  echo 'Ubuntu 24.04 amd64 is required' >&2; exit 1;
}
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq --no-install-recommends build-essential dpkg-dev binutils ca-certificates >/dev/null
export PATH="/toolchain/bin:$PATH"
export CARGO_HOME=/cargo-cache
export CARGO_TARGET_DIR=/work/target
mkdir -p /work/src
cp -a /source/. /work/src/
cd /work/src
{
  cat /etc/os-release
  dpkg --print-architecture
  rustc --version --verbose
  cargo --version
  cc --version | head -1
  ldd --version | head -1
} > /output/build-environment.txt
cargo build --locked --offline --release -p marinaos-core -p marinaos-cli
VERSION=0.1.0 BINDIR=/work/target/release OUTPUT_DIR=/output bash scripts/package-marinaos-deb.sh
cp /work/target/release/marinaosd /output/marinaosd-0.1.0
cp /work/target/release/marinaosctl /output/marinaosctl-0.1.0
# Build a distinct source fixture: change only the workspace version. This is
# not a claim that new production behavior was released.
sed -i 's/^version = "0\.1\.0"$/version = "0.1.1"/' Cargo.toml
cargo build --offline --release -p marinaos-core -p marinaos-cli
VERSION=0.1.1 BINDIR=/work/target/release OUTPUT_DIR=/output bash scripts/package-marinaos-deb.sh
cp /work/target/release/marinaosd /output/marinaosd-0.1.1
cp /work/target/release/marinaosctl /output/marinaosctl-0.1.1
{
  for bin in /output/marinaosd-* /output/marinaosctl-*; do
    echo "Binary: $bin"
    readelf -h "$bin" | grep -E 'Class:|Machine:|Type:'
    readelf -l "$bin" | grep 'interpreter'
    readelf -d "$bin" | grep NEEDED
    readelf --version-info "$bin" | grep -o 'GLIBC_[0-9.]*' | sort -Vu | tail -1
  done
  for pkg in /output/*.deb; do
    echo "Package: $pkg"
    dpkg-deb -f "$pkg" Package Version Architecture Depends
    dpkg-deb -c "$pkg"
  done
  sha256sum /output/*.deb /output/marinaosd-* /output/marinaosctl-*
} > /output/artifact-inspection.txt
