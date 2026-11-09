#!/usr/bin/env python3
"""Observe Ubuntu foundation UEFI boot to serial login in a disposable QEMU overlay.

This gate proves boot only, not installation, login authentication or recovery.
"""
import argparse
import hashlib
import json
from pathlib import Path
import selectors
import shutil
import subprocess
import tempfile
import time
import marinaos_lifecycle


def boot(image, code, variables, report, timeout=180, qemu='qemu-system-x86_64'):
    for path in (image, code, variables):
        if not path.is_file():
            raise ValueError(f'expected regular file: {path}')
    if timeout <= 0:
        raise ValueError('timeout must be positive')
    report.parent.mkdir(parents=True, exist_ok=True)
    if report.exists() or report.with_suffix('.serial.txt').exists():
        raise ValueError('refusing to overwrite boot evidence')
    # QEMU comma-delimited drive options cannot safely represent comma paths.
    if any(',' in str(p.resolve()) for p in (image, code, variables)):
        raise ValueError('commas are unsupported in QEMU image paths')
    status = 'failed'
    reason = 'serial-login-not-observed'
    output = bytearray()
    with tempfile.TemporaryDirectory(prefix='marinaos-qemu-') as tmp:
        writable_vars = Path(tmp) / 'vars.fd'
        shutil.copyfile(variables, writable_vars)
        command = [qemu, '-machine', 'q35', '-accel', 'tcg', '-m', '2048',
                   '-display', 'none', '-monitor', 'none', '-serial', 'stdio',
                   '-nic', 'none', '-no-reboot',
                   '-drive', f'if=pflash,format=raw,readonly=on,file={code.resolve()}',
                   '-drive', f'if=pflash,format=raw,file={writable_vars}',
                   '-drive', f'if=virtio,format=raw,snapshot=on,file={image.resolve()}']
        start = time.monotonic()
        process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        selector = selectors.DefaultSelector()
        selector.register(process.stdout, selectors.EVENT_READ)
        try:
            while time.monotonic() - start < timeout:
                if selector.select(timeout=min(0.5, timeout)):
                    data = process.stdout.read1(65536)
                    output.extend(data)
                    if b'marinaos login:' in output:
                        status, reason = 'passed', 'serial-login-observed'
                        break
                    if not data:
                        reason = 'qemu-exited-before-login'
                        break
        finally:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
            selector.close()
            process.stdout.close()
    report.with_suffix('.serial.txt').write_bytes(output)
    with image.open('rb') as stream:
        digest = hashlib.file_digest(stream, 'sha256').hexdigest()
    result = dict(schema='marinaos.qemu-boot/v1', status=status, reason=reason,
                  image_sha256=digest, firmware='uefi', secure_boot=False,
                  networking='disabled', disk='disposable-overlay',
                  installation='not-tested', authenticated_login='not-tested',
                  reboot_persistence='not-tested', command=command)
    report.write_text(json.dumps(result, indent=2) + '\n')
    return status == 'passed'


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--image', required=True, type=Path)
    parser.add_argument('--ovmf-code', required=True, type=Path)
    parser.add_argument('--ovmf-vars', required=True, type=Path)
    parser.add_argument('--report', required=True, type=Path)
    parser.add_argument('--timeout', type=int, default=180)
    parser.add_argument('--full', action='store_true', help='verify login, recovery and reboot using a disposable persistent overlay')
    parser.add_argument('--password-file', type=Path, help='private administrator password file for --full')
    args = parser.parse_args()
    try:
        if args.full:
            if args.password_file is None:
                parser.error('--full requires --password-file')
            passed = marinaos_lifecycle.qualify(args.image, args.ovmf_code, args.ovmf_vars,
                                               args.report, args.password_file, args.timeout)
        else:
            if args.password_file is not None:
                parser.error('--password-file requires --full')
            passed = boot(args.image, args.ovmf_code, args.ovmf_vars, args.report, args.timeout)
        raise SystemExit(0 if passed else 1)
    except (OSError, ValueError) as error:
        parser.exit(2, f'boot qualification unavailable: {error}\n')
