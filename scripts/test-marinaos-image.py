#!/usr/bin/env python3
"""Nonprivileged plan, safety and boot-harness tests; no disk tools are run."""
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import shutil
import tempfile
import unittest
from unittest import mock

SCRIPTS = Path(__file__).resolve().parent


def module(name, filename):
    spec = importlib.util.spec_from_file_location(name, SCRIPTS / filename)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


plan = module('image_plan', 'image-plan.py')
qemu = module('qemu_boot', 'qualify-marinaos-qemu.py')
lifecycle = module('lifecycle', 'marinaos_lifecycle.py')
MANIFEST = SCRIPTS.parent / 'image/ubuntu-24.04-amd64.json'


class ImagePlan(unittest.TestCase):
    def test_foundation_has_no_inference_dependency(self):
        actual = plan.load_plan(MANIFEST)
        self.assertEqual(actual['firmware'], 'uefi')
        self.assertEqual(actual['partition_table'], 'gpt')
        self.assertFalse(any('talvik' in p for p in actual['packages']))

    def test_rejects_incompatible_composition(self):
        for key, value in [('partition_table', 'msdos'), ('firmware', 'bios'),
                           ('architecture', 'arm64'), ('release', 'bookworm'),
                           ('secure_boot', True), ('image_size_mib', 512),
                           ('packages', ['talvik']), ('unexpected', True)]:
            with self.subTest(key=key), tempfile.TemporaryDirectory() as tmp:
                data = json.loads(MANIFEST.read_text())
                data[key] = value
                file = Path(tmp) / 'bad.json'
                file.write_text(json.dumps(data))
                with self.assertRaises(ValueError):
                    plan.load_plan(file)

    def test_plan_runs_without_root_or_native_artifacts(self):
        result = subprocess.run(['bash', str(SCRIPTS / 'build-marinaos-linux-image.sh'),
                                 '--plan'], capture_output=True, text=True, check=True)
        self.assertEqual(json.loads(result.stdout), plan.load_plan(MANIFEST))

    def test_existing_output_and_symlink_are_preserved(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)
            existing = path / 'disk'
            existing.write_text('do not truncate')
            link = path / 'link'
            link.symlink_to(existing)
            password = path / 'hash'
            password.write_text('$6$salt$hash')
            for output in (existing, link):
                result = subprocess.run(['bash', str(SCRIPTS / 'build-marinaos-linux-image.sh'),
                                         '--output', str(output), '--admin-password-hash-file',
                                         str(password)], capture_output=True, text=True)
                self.assertEqual(result.returncode, 2)
                self.assertIn('refusing existing', result.stderr)
                self.assertEqual(existing.read_text(), 'do not truncate')

    def test_missing_or_symlinked_packages_are_rejected_before_build(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)
            password = path / 'hash'
            password.write_text('$6$salt$hash')
            core = path / 'core.deb'
            core.write_bytes(b'fixture')
            link = path / 'link.deb'
            link.symlink_to(core)
            output = path / 'disk.raw'
            base = ['bash', str(SCRIPTS / 'build-marinaos-linux-image.sh'),
                    '--output', str(output), '--admin-password-hash-file', str(password)]
            for extra in ([], ['--core-deb', str(link), '--cli-deb', str(core)]):
                result = subprocess.run(base + extra, capture_output=True, text=True)
                self.assertEqual(result.returncode, 2)
                self.assertIn('DEB regular files required', result.stderr)
                self.assertFalse(output.exists())


@unittest.skipUnless(shutil.which('parted') and shutil.which('sfdisk') and shutil.which('sgdisk'),
                     'partition inspection tools unavailable')
class PartitionIntegration(unittest.TestCase):
    def test_real_gpt_on_disposable_regular_file(self):
        with tempfile.TemporaryDirectory() as tmp:
            image = Path(tmp) / 'disk.raw'
            with image.open('xb') as stream:
                stream.truncate(4096 * 1024 * 1024)
            subprocess.run(['sh', '-eu', '-c', '. "$1"; partition_uefi_image "$2"',
                            'test', str(SCRIPTS / 'lib/image-rootfs.sh'), str(image)],
                           check=True, capture_output=True)
            table = json.loads(subprocess.check_output(['sfdisk', '--json', str(image)]))['partitiontable']
            self.assertEqual(table['label'], 'gpt')
            esp, root = table['partitions']
            self.assertEqual(esp['start'] * table['sectorsize'], 1024 * 1024)
            self.assertEqual(esp['size'] * table['sectorsize'], 512 * 1024 * 1024)
            self.assertEqual(esp['type'].lower(), 'c12a7328-f81f-11d2-ba4b-00a0c93ec93b')
            self.assertEqual(root['start'] * table['sectorsize'], 513 * 1024 * 1024)
            self.assertEqual(root['type'].lower(), '0fc63daf-8483-4772-8e79-3d69d8477de4')
            validation = subprocess.check_output(['sgdisk', '--verify', str(image)], text=True)
            self.assertIn('No problems found', validation)


class BootHarness(unittest.TestCase):
    def simulate(self, text, success):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)
            files = [path / name for name in ('disk.raw', 'code.fd', 'vars.fd')]
            for f in files:
                f.write_bytes(b'fixture')
            fake = path / 'qemu'
            fake.write_text('#!/bin/sh\nprintf "%s" "' + text + '"\n')
            fake.chmod(0o755)
            report = path / 'boot.json'
            self.assertEqual(qemu.boot(*files, report, timeout=2, qemu=str(fake)), success)
            result = json.loads(report.read_text())
            self.assertEqual(result['installation'], 'not-tested')
            self.assertIn('-nic', result['command'])
            self.assertTrue(any('snapshot=on' in a for a in result['command']))
            self.assertEqual(files[0].read_bytes(), b'fixture')
            self.assertEqual(files[2].read_bytes(), b'fixture')

    def test_login_marker(self):
        self.simulate('marinaos login:', True)

    def test_kernel_output_is_not_boot_success(self):
        self.simulate('Linux kernel starting', False)

    def test_lifecycle_command_failure_cannot_emit_success(self):
        output = bytearray()
        console = lifecycle.Console(['/bin/sh'], output, b'not-used')
        try:
            console.command('true', 'OK', timeout=2)
            with self.assertRaisesRegex(RuntimeError, 'guest check failed'):
                console.command('false', 'BROKEN', timeout=2)
            self.assertNotIn(b'__M0_BROKEN__', output)
        finally:
            console.close()

    def test_lifecycle_collects_shutdown_output(self):
        output = bytearray()
        console = lifecycle.Console(['/bin/sh', '-c', 'printf "Power down"'], output, b'not-used')
        try:
            console.wait_shutdown(2)
            self.assertIn(b'Power down', output)
        finally:
            console.close()

    def test_lifecycle_waits_for_shell_after_login(self):
        guest = '''import select, sys, time
sys.stdin.readline()
print("Password:", end="", flush=True)
sys.stdin.readline()
if select.select([sys.stdin], [], [], 0.3)[0]:
    print("command arrived before shell", flush=True)
    sys.exit(1)
print("marina-admin@marinaos:~$ ", end="", flush=True)
sys.stdin.readline()
print("__M0_LOGIN1__", flush=True)
'''
        output = bytearray()
        console = lifecycle.Console(['python3', '-u', '-c', guest], output, b'fixture')
        try:
            console.login(1)
            self.assertNotIn(b'command arrived before shell', output)
        finally:
            console.close()

    def test_lifecycle_failure_is_recorded_and_password_redacted(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)
            for name in ('disk.raw', 'code.fd', 'vars.fd'):
                (path / name).write_bytes(b'fixture')
            password = path / 'password'
            password.write_bytes(b'unique-fixture-secret')
            create = path / 'qemu-img'
            create.write_text('#!/bin/sh\nfor x do last=$x; done\n: > "$last"\n')
            create.chmod(0o755)
            fake = path / 'qemu-system-x86_64'
            fake.write_text('#!/bin/sh\nprintf "marinaos login: unique-fixture-secret"\nsleep 0.2\n')
            fake.chmod(0o755)
            report = path / 'lifecycle.json'
            with mock.patch.dict(os.environ, {'PATH': str(path) + ':' + os.environ['PATH']}):
                self.assertFalse(lifecycle.qualify(path / 'disk.raw', path / 'code.fd',
                                 path / 'vars.fd', report, password, timeout=2))
            self.assertEqual(json.loads(report.read_text())['status'], 'failed')
            self.assertNotIn(b'unique-fixture-secret', report.with_suffix('.serial.txt').read_bytes())


if __name__ == '__main__':
    unittest.main()
