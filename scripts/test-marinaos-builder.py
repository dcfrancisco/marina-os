#!/usr/bin/env python3
"""Exercise full builder control flow with fake privileged tools; never mount disks."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

SCRIPTS = Path(__file__).resolve().parent
STUB = r'''#!/usr/bin/python3
import json, os, pathlib, sys
name = pathlib.Path(sys.argv[0]).name
args = sys.argv[1:]
record = dict(tool=name, args=args)
if name == 'debootstrap':
    root = pathlib.Path(args[-2])
    for directory in ['etc/apt', 'etc/default/grub.d', 'usr/sbin', 'dev', 'proc', 'sys', 'boot']:
        (root / directory).mkdir(parents=True, exist_ok=True)
    (root / 'etc/fstab').write_text('obsolete bootstrap fstab\n')
elif name == 'id':
    print('0')
elif name == 'uname':
    print('x86_64')
elif name == 'losetup' and '--find' in args:
    print('/dev/loop999')
elif name == 'blkid':
    print('ABCD-1234' if args[-1].endswith('p1') else 'root-123')
elif name == 'sha256sum':
    print('fixture-digest  ' + args[0])
elif name == 'chroot':
    root = pathlib.Path(args[0])
    if args[1] == 'chpasswd':
        sys.stdin.read()  # Never log passwords, even fixture inputs.
    if args[1] == 'dpkg-query':
        print('systemd\tfixture\tamd64')
    if args[1] == 'grub-install':
        loader = root / 'boot/efi/EFI/BOOT/BOOTX64.EFI'
        loader.parent.mkdir(parents=True, exist_ok=True)
        loader.write_bytes(b'fixture EFI binary')
    if args[1] == 'grub-mkconfig':
        record['fstab'] = (root / 'etc/fstab').read_text()
        record['hostname'] = (root / 'etc/hostname').read_text()
with open(os.environ['BUILD_TEST_LOG'], 'a') as stream:
    stream.write(json.dumps(record) + '\n')
if name == 'umount' and 'image-root' in args[-1] and os.environ.get('FAIL_IMAGE_UNMOUNT'):
    sys.exit(1)
'''


class BuilderIntegration(unittest.TestCase):
    def build(self, fail=False):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            mocks = directory / 'bin'
            mocks.mkdir()
            for name in ['id', 'uname', 'debootstrap', 'chroot', 'parted', 'mount',
                         'umount', 'losetup', 'mkfs.ext4', 'mkfs.vfat', 'blkid',
                         'sha256sum', 'sync']:
                path = mocks / name
                path.write_text(STUB)
                path.chmod(0o755)
            password = directory / 'password'
            password.write_text('$6$fixture$not-a-real-password-hash')
            output = directory / 'image.raw'
            log = directory / 'calls.jsonl'
            env = {**os.environ, 'PATH': str(mocks) + ':' + os.environ['PATH'],
                   'TMPDIR': tmp, 'BUILD_TEST_LOG': str(log)}
            if fail:
                env['FAIL_IMAGE_UNMOUNT'] = '1'
            result = subprocess.run(['bash', str(SCRIPTS / 'build-marinaos-linux-image.sh'),
                                     '--output', str(output), '--admin-password-hash-file',
                                     str(password)], env=env, capture_output=True, text=True)
            calls = [json.loads(line) for line in log.read_text().splitlines()]
            if fail:
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(output.with_suffix('.raw.sha256').exists())
                self.assertFalse(output.with_suffix('.raw.manifest').exists())
                self.assertIn('cleanup incomplete', result.stderr)
                return
            self.assertEqual(result.returncode, 0, result.stderr)
            account = next(c for c in calls if c['tool'] == 'chroot' and c['args'][1] == 'useradd')
            self.assertEqual(account['args'][-1], 'marina-admin')
            grub = next(c for c in calls if c['tool'] == 'chroot' and c['args'][1] == 'grub-install')
            self.assertIn('--removable', grub['args'])
            self.assertIn('--no-nvram', grub['args'])
            configured = next(c for c in calls if 'fstab' in c)
            self.assertEqual(configured['fstab'], 'UUID=root-123 / ext4 defaults 0 1\n'
                             'UUID=ABCD-1234 /boot/efi vfat umask=0077 0 1\n')
            self.assertEqual(configured['hostname'], 'marinaos\n')
            final_detach = max(i for i, c in enumerate(calls)
                               if c['tool'] == 'losetup' and '-d' in c['args'])
            first_hash = min(i for i, c in enumerate(calls) if c['tool'] == 'sha256sum')
            self.assertLess(final_detach, first_hash)
            self.assertTrue(output.with_suffix('.raw.manifest').exists())
            self.assertNotIn('talvik', log.read_text())
            self.assertNotIn('not-a-real-password', log.read_text())

    def test_full_uefi_build_control_flow(self):
        self.build()

    def test_failed_unmount_prevents_success_evidence(self):
        self.build(fail=True)


if __name__ == '__main__':
    unittest.main()
