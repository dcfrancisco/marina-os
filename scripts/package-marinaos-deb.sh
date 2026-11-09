#!/usr/bin/env bash
set -euo pipefail
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
version=${VERSION:-0.1.0}
bindir=${BINDIR:-$root/target/release}
out=${OUTPUT_DIR:-$root/artifacts}
[[ $version =~ ^[0-9][0-9A-Za-z.+~:-]*$ ]] || { echo 'invalid version' >&2; exit 2; }
for bin in marinaosd marinaosctl; do [[ -x $bindir/$bin ]] || { echo "missing executable $bindir/$bin" >&2; exit 2; }; done
[[ $(dpkg --print-architecture) = amd64 ]] || { echo 'amd64 build environment required' >&2; exit 2; }
for package in core cli; do
  [[ ! -e "$out/marinaos-${package}_${version}_amd64.deb" && ! -L "$out/marinaos-${package}_${version}_amd64.deb" ]] || {
    echo "output already exists for marinaos-$package $version" >&2; exit 2;
  }
done
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/debian" "$out"
cat > "$work/debian/control" <<CONTROL
Source: marinaos
Section: admin
Priority: optional
Maintainer: MarinaOS contributors
Standards-Version: 4.6.2

Package: marinaos-core
Architecture: amd64
Description: MarinaOS diagnostics service

Package: marinaos-cli
Architecture: amd64
Description: MarinaOS diagnostics client
CONTROL
for package in core cli; do
  stage="$work/$package"
  mkdir -p "$stage/DEBIAN" "$stage/usr/bin"
  if [[ $package = core ]]; then bin=marinaosd; else bin=marinaosctl; fi
  install -m755 "$bindir/$bin" "$stage/usr/bin/$bin"
  deps=$(cd "$work" && dpkg-shlibdeps -O -e "$stage/usr/bin/$bin")
  deps=${deps#shlibs:Depends=}
  if [[ $package = core ]]; then
    deps="$deps, systemd, init-system-helpers (>= 1.52)"
    install -Dm644 "$root/packaging/marinaos/marinaosd.service" "$stage/usr/lib/systemd/system/marinaosd.service"
    install -Dm644 "$root/packaging/marinaos/marinaos.conf" "$stage/usr/lib/sysusers.d/marinaos.conf"
    install -m755 "$root/packaging/marinaos/debian/"{postinst,prerm,postrm} "$stage/DEBIAN/"
  else
    deps="$deps, marinaos-core (>= $version)"
  fi
  cat > "$stage/DEBIAN/control" <<CONTROL
Package: marinaos-$package
Version: $version
Architecture: amd64
Section: admin
Priority: optional
Maintainer: MarinaOS contributors
Depends: $deps
Description: MarinaOS read-only system diagnostics $package
 Native local system diagnostics; no model runtime is included.
CONTROL
  dpkg-deb --root-owner-group --build "$stage" "$out/marinaos-${package}_${version}_amd64.deb"
done
