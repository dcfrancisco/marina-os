#!/bin/sh
# Run only in an ephemeral Ubuntu 24.04 container; /packages is read-only.
set -eu
. /etc/os-release
[ "$ID" = ubuntu ] && [ "$VERSION_ID" = 24.04 ] && [ "$(dpkg --print-architecture)" = amd64 ] || {
  echo 'Ubuntu 24.04 amd64 is required' >&2; exit 1;
}
export DEBIAN_FRONTEND=noninteractive
if [ "${MARINAOS_SKIP_APT:-0}" != 1 ]; then
  apt-get update -qq
  apt-get install -y -qq systemd util-linux >/dev/null
fi
if command -v gcc >/dev/null 2>&1; then echo 'test target unexpectedly contains a compiler' >&2; exit 1; fi
echo 'PASS: no compiler present on target'
printf '#!/bin/sh\nexit 101\n' > /usr/sbin/policy-rc.d
chmod 755 /usr/sbin/policy-rc.d
dpkg -i /packages/marinaos-core_0.1.0_amd64.deb /packages/marinaos-cli_0.1.0_amd64.deb
test ! -e /usr/bin/talvikd
test ! -e /usr/lib/marina/libllama.so
systemd-analyze verify /usr/lib/systemd/system/marinaosd.service
echo 'PASS: systemd-analyze verify'
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
test "$(stat -c %a /run/marinaos)" = 750
test "$(stat -c %a /run/marinaos/marinaos.sock)" = 660
test "$(stat -c %U:%G /run/marinaos/marinaos.sock)" = marinaos:marinaos-readers
echo 'PASS: sysusers account and socket ownership and modes'
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
echo 'PASS: reader, outsider, socket-directory and revoked-group checks'
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
mkdir -p /var/lib/marinaos
printf 'keep\n' > /var/lib/marinaos/retention-test
old_hash=$(sha256sum /usr/bin/marinaosd | cut -d' ' -f1)
su -s /bin/sh reader -c 'marinaosctl version' | grep -q '"version": "0.1.0"'
dpkg -i /packages/marinaos-core_0.1.1_amd64.deb /packages/marinaos-cli_0.1.1_amd64.deb
new_hash=$(sha256sum /usr/bin/marinaosd | cut -d' ' -f1)
test "$old_hash" != "$new_hash"
echo "PASS: daemon binaries differ: $old_hash -> $new_hash"
# policy-rc.d suppresses systemd; the old manually started process remains live.
su -s /bin/sh reader -c 'marinaosctl version' | grep -q '"version": "0.1.0"'
echo 'PASS: old daemon remains running after package upgrade'
kill "$daemon"; wait "$daemon" || true
trap - EXIT
runuser -u marinaos -g marinaos-readers -- /usr/bin/marinaosd >/tmp/marinaosd-upgraded.log 2>&1 &
daemon=$!
trap 'kill "$daemon" 2>/dev/null || true; wait "$daemon" 2>/dev/null || true' EXIT
n=0
until su -s /bin/sh reader -c 'marinaosctl version' 2>/dev/null | grep -q '"version": "0.1.1"'; do
  n=$((n+1)); [ "$n" -lt 100 ] || { cat /tmp/marinaosd-upgraded.log; exit 1; }; sleep .05
done
echo 'PASS: manually restarted daemon reports version 0.1.1'
kill "$daemon"; wait "$daemon" || true
trap - EXIT
dpkg -r marinaos-cli marinaos-core
test "$(cat /var/lib/marinaos/retention-test)" = keep
getent passwd marinaos
printf '%s\n' 'PASS: Ubuntu 24.04 package install, reader access, manual restart, different-binary running-process upgrade, removal and data retention'
printf '%s\n' 'NOT TESTED: systemd-managed service startup, restart and boot'
