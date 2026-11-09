#!/usr/bin/env python3
"""Offline serial-console lifecycle qualification for a disposable QEMU overlay.

Called by qualify-marinaos-qemu.py; secrets are supplied through a private file.
"""
import hashlib
import json
from pathlib import Path
import selectors
import shutil
import subprocess
import tempfile
import time


class Console:
    def __init__(self, command, output, secret):
        self.output = output
        self.secret = secret
        self.process = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                        stderr=subprocess.STDOUT)
        self.selector = selectors.DefaultSelector()
        self.selector.register(self.process.stdout, selectors.EVENT_READ)

    def expect(self, token, timeout, start=None, failure=None):
        start = len(self.output) if start is None else start
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if token in self.output[start:]:
                return
            if failure and failure in self.output[start:]:
                raise RuntimeError('guest check failed: ' + failure.decode('ascii', 'replace'))
            if self.selector.select(timeout=0.5):
                data = self.process.stdout.read1(65536)
                if not data:
                    raise RuntimeError('qemu exited before expected serial marker')
                self.output.extend(data)
                if len(self.output) > 8 * 1024 * 1024:
                    raise RuntimeError('serial output limit exceeded')
            elif self.process.poll() is not None:
                raise RuntimeError('qemu exited before expected serial marker')
        raise RuntimeError('timeout waiting for ' + token.decode('ascii', 'replace'))

    def send(self, data):
        self.process.stdin.write(data + b'\n')
        self.process.stdin.flush()

    def command(self, command, marker, timeout=120):
        start = len(self.output)
        # Split marker in the shell command so terminal echo cannot fake success.
        wrapped = ("if { " + command + "; }; then printf '\\n__M0_%s__\\n' '" + marker +
                   "'; else printf '\\n__M0_%s__\\n' '" + marker + "_FAIL'; fi")
        self.send(wrapped.encode())
        self.expect(('__M0_' + marker + '__').encode(), timeout, start,
                    ('__M0_' + marker + '_FAIL__').encode())

    def sudo(self):
        start = len(self.output)
        self.send(b'sudo -v')
        self.expect(b'password for marina-admin:', 120, start)
        self.send(self.secret)
        self.command('sudo -n true && true', 'SUDO', 120)

    def close(self):
        if self.process.poll() is None:
            self.process.terminate()
            try:
                self.process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait()
        self.selector.close()
        self.process.stdin.close()
        self.process.stdout.close()

    def wait_shutdown(self, timeout):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if self.selector.select(timeout=0.5):
                data = self.process.stdout.read1(65536)
                if data:
                    self.output.extend(data)
            if self.process.poll() is not None:
                self.output.extend(self.process.stdout.read())
                if self.process.returncode != 0:
                    raise RuntimeError('QEMU exited unsuccessfully during shutdown')
                return
        raise RuntimeError('shutdown timed out')


def qualify(image, code, variables, report, password_file, timeout=300):
    for path in (image, code, variables, password_file):
        if not path.is_file() or path.is_symlink():
            raise ValueError(f'expected regular file: {path}')
    if timeout <= 0:
        raise ValueError('timeout must be positive')
    if any(',' in str(p.resolve()) for p in (image, code, variables)):
        raise ValueError('commas are unsupported in QEMU image paths')
    secret = password_file.read_bytes().strip()
    if not secret or b'\n' in secret or len(secret) > 256:
        raise ValueError('invalid password file')
    report.parent.mkdir(parents=True, exist_ok=True)
    serial_path = report.with_suffix('.serial.txt')
    if report.exists() or serial_path.exists():
        raise ValueError('refusing to overwrite evidence')
    output = bytearray()
    steps = {key: 'not-tested' for key in ('kernel', 'boot', 'login', 'systemd', 'service',
             'cli', 'service_recovery', 'shutdown', 'reboot', 'persistence')}
    result = 'failed'
    reason = 'not started'
    command = None
    current_step = None
    with tempfile.TemporaryDirectory(prefix='marinaos-lifecycle-', dir=report.parent) as tmp:
        overlay = Path(tmp) / 'disk.qcow2'
        firmware = Path(tmp) / 'vars.fd'
        shutil.copyfile(variables, firmware)
        try:
            subprocess.run(['qemu-img', 'create', '-q', '-f', 'qcow2', '-F', 'raw',
                            '-b', str(image.resolve()), str(overlay)], check=True,
                           stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
            command = ['qemu-system-x86_64', '-machine', 'q35', '-accel', 'tcg', '-smp', '2', '-m', '2048',
                       '-display', 'none', '-monitor', 'none', '-serial', 'stdio', '-nic', 'none',
                       '-no-reboot', '-drive', f'if=pflash,format=raw,readonly=on,file={code.resolve()}',
                       '-drive', f'if=pflash,format=raw,file={firmware}',
                       '-drive', f'if=virtio,format=qcow2,file={overlay}']
            for boot_number in (1, 2):
                console = Console(command, output, secret)
                try:
                    # Search the current boot only; earlier serial output must not satisfy checks.
                    offset = len(output)
                    if boot_number == 1:
                        current_step = 'kernel'
                        console.expect(b'Linux version ', timeout, offset)
                        steps['kernel'] = 'passed'
                    current_step = 'boot' if boot_number == 1 else 'reboot'
                    console.expect(b'marinaos login:', timeout, offset)
                    steps[current_step] = 'passed'
                    current_step = 'login'
                    start = len(output)
                    console.send(b'marina-admin')
                    console.expect(b'Password:', 120, start)
                    console.send(secret)
                    console.command('true', 'LOGIN' + str(boot_number), 180)
                    steps['login'] = 'passed'
                    if boot_number == 1:
                        current_step = 'systemd'
                        console.command('state=$(systemctl is-system-running --wait || true); printf "systemd-state=%s\\n" "$state"; if [ "$state" != running ]; then systemctl --failed --no-legend --no-pager; false; fi', 'SYSTEMD', timeout)
                        steps['systemd'] = 'passed'
                        # command() requires an explicit success guard for each check.
                        current_step = 'service'
                        service = ('( systemctl is-active --quiet marinaosd.service && '
                                   'pid=$(systemctl show -p MainPID --value marinaosd.service) && '
                                   '[ "$pid" -gt 0 ] && '
                                   '[ "$(awk \'/^Uid:/ {print $2}\' /proc/"$pid"/status)" = "$(id -u marinaos)" ] ) '
                                   '|| { systemctl status marinaosd.service --no-pager; false; }')
                        console.command(service, 'SERVICE')
                        steps['service'] = 'passed'
                        current_step = 'cli'
                        console.command('marinaosctl version && marinaosctl status && marinaosctl doctor && marinaosctl hardware', 'CLI')
                        steps['cli'] = 'passed'
                        current_step = 'persistence'
                        console.command("printf '%s' marinaos-persistence-v1 > \"$HOME/.marinaos-m0-check\" && sync", 'SAVED')
                    else:
                        current_step = 'persistence'
                        console.command('test "$(cat "$HOME/.marinaos-m0-check")" = marinaos-persistence-v1 && systemctl is-active --quiet marinaosd.service && marinaosctl version >/dev/null', 'PERSIST')
                        steps['persistence'] = 'passed'
                    current_step = 'service_recovery' if boot_number == 1 else 'shutdown'
                    console.sudo()
                    if boot_number == 1:
                        recovery = ('old=$(systemctl show -p MainPID --value marinaosd.service); '
                                    'sudo -n systemctl kill -s KILL marinaosd.service && { '
                                    'ok=no; for i in $(seq 1 30); do sleep 1; '
                                    'new=$(systemctl show -p MainPID --value marinaosd.service); '
                                    'if [ "$new" != 0 ] && [ "$new" != "$old" ] && '
                                    'systemctl is-active --quiet marinaosd.service && marinaosctl version >/dev/null; '
                                    'then ok=yes; break; fi; done; [ "$ok" = yes ]; }')
                        console.command(recovery, 'RECOVERY', 180)
                        steps['service_recovery'] = 'passed'
                    current_step = 'shutdown'
                    shutdown_offset = len(output)
                    console.send(b'sudo -n systemctl poweroff')
                    console.wait_shutdown(300)
                    if not any(marker in output[shutdown_offset:] for marker in
                               (b'Power down', b'Powering off')):
                        raise RuntimeError('power-off message not observed')
                    steps['shutdown'] = 'passed'
                finally:
                    console.close()
            result, reason = 'passed', 'all checks observed'
        except (OSError, RuntimeError, subprocess.SubprocessError) as error:
            reason = str(error)
            if current_step is not None:
                steps[current_step] = 'failed'
    serial_path.write_bytes(bytes(output).replace(secret, b'[REDACTED]'))
    with image.open('rb') as stream:
        digest = hashlib.file_digest(stream, 'sha256').hexdigest()
    report.write_text(json.dumps(dict(schema='marinaos.qemu-lifecycle/v1', status=result,
                      reason=reason, steps=steps, image_sha256=digest, networking='disabled',
                      disk='disposable-persistent-overlay', acceleration='tcg',
                      command=command), indent=2) + '\n')
    return result == 'passed'
