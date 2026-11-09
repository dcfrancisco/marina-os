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
    for arg in args:
        print('fixture-digest  ' + arg)
elif name == 'dpkg-deb':
    package = pathlib.Path(args[1]).name
    if args[2] == 'Package':
        print('wrong-package' if os.environ.get('BAD_DEB') == 'name' else package.removesuffix('.deb'))
    elif args[2] == 'Architecture':
        print('arm64' if os.environ.get('BAD_DEB') == 'arch' else 'amd64')
    elif args[2] == 'Version':
        print('0.1.1' if os.environ.get('BAD_DEB') == 'version' and 'cli' in package else '0.1.0')
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
    def build(self, fail=False, bad_deb=None):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            mocks = directory / 'bin'
            mocks.mkdir()
            for name in ['id', 'uname', 'debootstrap', 'chroot', 'parted', 'mount',
                         'umount', 'losetup', 'mkfs.ext4', 'mkfs.vfat', 'blkid',
                         'sha256sum', 'sync', 'dpkg-deb']:
                path = mocks / name
                path.write_text(STUB)
                path.chmod(0o755)
            password = directory / 'password'
            password.write_text('$6$fixture$not-a-real-password-hash')
            core = directory / 'core.deb'
            cli = directory / 'cli.deb'
            core.write_bytes(b'core fixture')
            cli.write_bytes(b'cli fixture')
            output = directory / 'image.raw'
            log = directory / 'calls.jsonl'
            env = {**os.environ, 'PATH': str(mocks) + ':' + os.environ['PATH'],
                   'TMPDIR': tmp, 'BUILD_TEST_LOG': str(log)}
            if fail:
                env['FAIL_IMAGE_UNMOUNT'] = '1'
            if bad_deb:
                env['BAD_DEB'] = bad_deb
            result = subprocess.run(['bash', str(SCRIPTS / 'build-marinaos-linux-image.sh'),
                                     '--output', str(output), '--admin-password-hash-file',
                                     str(password), '--core-deb', str(core), '--cli-deb',
                                     str(cli)], env=env, capture_output=True, text=True)
            calls = [json.loads(line) for line in log.read_text().splitlines()]
            if bad_deb:
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(output.exists())
                self.assertFalse(any(c['tool'] == 'debootstrap' for c in calls))
                self.assertIn('versions must match' if bad_deb == 'version' else
                              'invalid package identity', result.stderr)
                return
            if fail:
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(output.with_suffix('.raw.sha256').exists())
                self.assertFalse(output.with_suffix('.raw.manifest').exists())
                self.assertIn('cleanup incomplete', result.stderr)
                return
            self.assertEqual(result.returncode, 0, result.stderr)
            account = next(c for c in calls if c['tool'] == 'chroot' and c['args'][1] == 'useradd')
            self.assertEqual(account['args'][-1], 'marina-admin')
            self.assertIn('sudo,marinaos-readers', account['args'])
            installed = next(c for c in calls if c['tool'] == 'chroot' and
                             '/tmp/marinaos-core.deb' in c['args'])
            self.assertIn('/tmp/marinaos-cli.deb', installed['args'])
            self.assertTrue(any(c['tool'] == 'chroot' and c['args'][1:] ==
                                ['systemctl', 'enable', 'marinaosd.service'] for c in calls))
            grub = next(c for c in calls if c['tool'] == 'chroot' and c['args'][1] == 'grub-install')
            self.assertIn('--removable', grub['args'])
            self.assertIn('--no-nvram', grub['args'])
            configured = next(c for c in calls if 'fstab' in c)
            self.assertEqual(configured['fstab'], 'UUID=root-123 / ext4 defaults 0 1\n'
                             'UUID=ABCD-1234 /boot/efi vfat umask=0077 0 1\n')
            self.assertEqual(configured['hostname'], 'marinaos\n')
            final_detach = max(i for i, c in enumerate(calls)
                               if c['tool'] == 'losetup' and '-d' in c['args'])
            image_hash = next(i for i, c in enumerate(calls) if c['tool'] == 'sha256sum'
                              and c['args'] == [str(output)])
            self.assertLess(final_detach, image_hash)
            self.assertTrue(output.with_suffix('.raw.manifest').exists())
            self.assertTrue(output.with_suffix('.raw.marinaos-packages.sha256').exists())
            self.assertNotIn('talvik', log.read_text())
            self.assertNotIn('not-a-real-password', log.read_text())

    def test_full_uefi_build_control_flow(self):
        self.build()

    def test_failed_unmount_prevents_success_evidence(self):
        self.build(fail=True)

    def test_rejects_unexpected_packages_before_bootstrap(self):
        for defect in ('name', 'arch', 'version'):
            with self.subTest(defect=defect):
                self.build(bad_deb=defect)


if __name__ == '__main__':
    unittest.main()
