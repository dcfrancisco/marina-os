#!/usr/bin/env python3
"""Nonprivileged regression against the production rootfs assembly helper."""
import pathlib
import subprocess
import tempfile
import unittest

HELPER = pathlib.Path(__file__).resolve().parent / 'lib/image-rootfs.sh'


class RootfsAssembly(unittest.TestCase):
    def assemble(self, stale, esp=''):
        with tempfile.TemporaryDirectory() as temp:
            source = pathlib.Path(temp) / 'source'
            target = pathlib.Path(temp) / 'target'
            (source / 'etc').mkdir(parents=True)
            target.mkdir()
            (source / 'etc/retained').write_text('preserve me\n')
            (source / 'etc/retained').chmod(0o640)
            (source / 'etc/link').symlink_to('retained')
            if stale:
                (source / 'etc/fstab').write_text('UUID=obsolete / ext4 defaults 0 1\n')
            # Original code fails here: target/etc does not exist yet.
            with self.assertRaises(FileNotFoundError):
                (target / 'etc/fstab').write_text('too early')
            subprocess.run(['sh', '-eu', '-c',
                            '. "$1"; assemble_image_rootfs "$2" "$3" root-123 "$4"',
                            'test', str(HELPER), str(source), str(target), esp], check=True)
            expected = 'UUID=root-123 / ext4 defaults 0 1\n'
            if esp:
                expected += f'UUID={esp} /boot/efi vfat umask=0077 0 1\n'
            self.assertEqual((target / 'etc/fstab').read_text(), expected)
            self.assertEqual((target / 'etc/retained').read_text(), 'preserve me\n')
            self.assertEqual((target / 'etc/retained').stat().st_mode & 0o777, 0o640)
            self.assertTrue((target / 'etc/link').is_symlink())
            self.assertNotIn('root-123', (source / 'etc/fstab').read_text() if stale else '')

    def test_fresh_root(self):
        self.assemble(False)

    def test_stale_fstab_cannot_overwrite_final_configuration(self):
        self.assemble(True)

    def test_uefi_root_and_esp_entries_persist(self):
        self.assemble(True, 'ABCD-1234')

    def test_copy_failure_does_not_publish_fstab(self):
        with tempfile.TemporaryDirectory() as temp:
            target = pathlib.Path(temp) / 'target'
            target.mkdir()
            result = subprocess.run(['sh', '-eu', '-c',
                '. "$1"; assemble_image_rootfs "$2/missing" "$2/target" root-123',
                'test', str(HELPER), temp], capture_output=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertFalse((target / 'etc/fstab').exists())

    def test_reservation_rejects_existing_files_links_and_devices(self):
        with tempfile.TemporaryDirectory() as temp:
            target = pathlib.Path(temp) / 'new.raw'
            command = ['sh', '-eu', '-c', '. "$1"; reserve_image_file "$2"',
                       'test', str(HELPER)]
            subprocess.run(command + [str(target)], check=True)
            self.assertEqual(target.stat().st_mode & 0o777, 0o600)
            target.write_text('preserve')
            dangling = pathlib.Path(temp) / 'dangling'
            dangling.symlink_to(pathlib.Path(temp) / 'missing')
            for path in (target, dangling, pathlib.Path('/dev/null')):
                result = subprocess.run(command + [str(path)], capture_output=True)
                self.assertNotEqual(result.returncode, 0)
            self.assertEqual(target.read_text(), 'preserve')


if __name__ == '__main__':
    unittest.main()
