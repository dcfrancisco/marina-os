#!/bin/sh
# Run only in an ephemeral Ubuntu 24.04 container; /packages is read-only.
set -eu
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq systemd util-linux >/dev/null
printf '#!/bin/sh\nexit 101\n' > /usr/sbin/policy-rc.d
chmod 755 /usr/sbin/policy-rc.d
dpkg -i /packages/marinaos-core_0.1.0_amd64.deb /packages/marinaos-cli_0.1.0_amd64.deb
getent passwd marinaos
getent group marinaos-readers
useradd -m -G marinaos-readers reader
useradd -m outsider
install -d -o marinaos -g marinaos-readers -m 0750 /run/marinaos
runuser -u marinaos -g marinaos-readers -- /usr/bin/marinaosd >/tmp/marinaosd.log 2>&1 &
daemon=$!
trap 'kill "$daemon" 2>/dev/null || true; wait "$daemon" 2>/dev/null || true' EXIT
n=0
while [ ! -S /run/marinaos/marinaos.sock ]; do
  n=$((n+1)); [ "$n" -lt 100 ] || { cat /tmp/marinaosd.log; exit 1; }; sleep .05
done
su -s /bin/sh reader -c 'marinaosctl version && marinaosctl status && marinaosctl doctor && marinaosctl hardware' >/tmp/reader.json
if su -s /bin/sh outsider -c 'marinaosctl status' >/dev/null 2>&1; then echo 'unauthorized access succeeded' >&2; exit 1; fi
if su -s /bin/sh reader -c 'touch /run/marinaos/spoof' >/dev/null 2>&1; then echo 'reader can alter socket directory' >&2; exit 1; fi
if su -s /bin/sh outsider -c 'touch /run/marinaos/spoof' >/dev/null 2>&1; then echo 'outsider can alter socket directory' >&2; exit 1; fi
reader_gid=$(getent group marinaos-readers | cut -d: -f3)
gpasswd -d reader marinaos-readers >/dev/null
if setpriv --reuid=reader --regid=reader --groups "$reader_gid" marinaosctl status >/tmp/revoked.out 2>&1; then
  echo 'revoked reader retained access' >&2; exit 1
fi
grep -q unauthorized /tmp/revoked.out || { cat /tmp/revoked.out; exit 1; }
kill "$daemon"; wait "$daemon" || true
trap - EXIT
gpasswd -a reader marinaos-readers >/dev/null
runuser -u marinaos -g marinaos-readers -- /usr/bin/marinaosd >/tmp/marinaosd-restart.log 2>&1 &
daemon=$!
trap 'kill "$daemon" 2>/dev/null || true; wait "$daemon" 2>/dev/null || true' EXIT
n=0
until su -s /bin/sh reader -c 'marinaosctl version' >/dev/null 2>&1; do
  n=$((n+1)); [ "$n" -lt 100 ] || { cat /tmp/marinaosd-restart.log; exit 1; }; sleep .05
done
kill "$daemon"; wait "$daemon" || true
trap - EXIT
mkdir -p /var/lib/marinaos
printf 'keep\n' > /var/lib/marinaos/retention-test
dpkg -i /packages/marinaos-core_0.1.1_amd64.deb /packages/marinaos-cli_0.1.1_amd64.deb
dpkg -r marinaos-cli marinaos-core
test "$(cat /var/lib/marinaos/retention-test)" = keep
getent passwd marinaos
printf '%s\n' 'PASS: Ubuntu 24.04 container package install, manual service restart, access, upgrade, removal and data retention'
printf '%s\n' 'NOT TESTED: systemd-managed service startup, restart and boot'
