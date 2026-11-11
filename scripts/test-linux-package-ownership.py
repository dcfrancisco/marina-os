#!/usr/bin/env python3
"""Build fixture DEBs and reject conflicting ownership in a relocated dpkg root.

Fake payloads test package metadata/lifecycle only, never native execution.
"""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


def run(*args, **kwargs):
    result = subprocess.run(args, text=True, capture_output=True, **kwargs)
    if result.returncode:
        raise AssertionError(result.stderr + result.stdout)
    return result


@unittest.skipUnless(shutil.which('dpkg-deb') and shutil.which('dpkg'), 'dpkg unavailable')
class PackageOwnership(unittest.TestCase):
    def test_conflicting_packages_cannot_replace_files(self):
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            binary = work / 'binary'
            binary.write_text('#!/bin/sh\nexit 0\n')
            binary.chmod(0o755)
            libs = work / 'lib'
            libs.mkdir()
            for name in ['llama', 'ggml', 'ggml-base', 'ggml-cpu', 'mtmd', 'oid-chat-template']:
                (libs / f'lib{name}.so').write_bytes(b'not executable: package fixture')
            packages = {}
            for name, conflict in [('oid', 'talvik'), ('talvik', 'oid-marina')]:
                package = work / f'{name}.deb'
                args = ['bash', str(ROOT / f'scripts/package-{name}-deb.sh'),
                        '--talvikd', str(binary), '--talvik', str(binary),
                        '--native-lib-dir', str(libs), '--output', str(package)]
                if name == 'oid':
                    args += ['--console', str(binary), '--ask', str(binary)]
                run(*args)
                self.assertEqual(run('dpkg-deb', '-f', str(package), 'Conflicts').stdout.strip(), conflict)
                # Strip maintainer scripts before any relocated dpkg operation.
                extracted = work / name
                run('dpkg-deb', '-R', str(package), str(extracted))
                if name == 'oid':
                    self.assertEqual({p.name for p in (extracted / 'usr/bin').iterdir()},
                                     {'console', 'ask', 'talvik', 'talvikd'})
                    self.assertTrue((extracted / 'usr/bin/console').is_file())
                    for command_name in ('console', 'ask'):
                        self.assertEqual((extracted / f'usr/bin/{command_name}').stat().st_mode & 0o777, 0o755)
                    self.assertTrue((extracted / 'etc/marinaos/marina-client.env').is_file())
                    self.assertTrue((extracted / 'usr/lib/marinaos/readiness').is_file())
                    self.assertTrue((extracted / 'usr/share/marinaos/README.md').is_file())
                    client_bins = work / 'client-bins'
                    client_bins.mkdir()
                    for command_name in ('console', 'ask'):
                        (client_bins / command_name).symlink_to(binary)
                    unrelated_root = work / 'unrelated-command-stage'
                    unrelated_chat = unrelated_root / 'usr/bin/chat'
                    unrelated_chat.parent.mkdir(parents=True)
                    unrelated_chat.write_bytes(b'unrelated existing command')
                    run('bash', str(ROOT / 'scripts/stage-marinaos-clients.sh'),
                        str(client_bins), str(libs),
                        env={**os.environ, 'DESTDIR': str(unrelated_root)})
                    self.assertEqual(unrelated_chat.read_bytes(), b'unrelated existing command')
                    collision = subprocess.run(['bash', str(ROOT / 'scripts/stage-marinaos-clients.sh'),
                                                str(client_bins), str(libs)],
                                               env={**os.environ, 'DESTDIR': str(extracted)},
                                               capture_output=True, text=True)
                    self.assertNotEqual(collision.returncode, 0)
                    self.assertIn('refusing to overwrite existing command', collision.stderr)
                    self.assertFalse((extracted / 'usr/bin/oid-console').exists())
                    self.assertIn('ExecStart=/usr/bin/console',
                                  (extracted / 'usr/lib/systemd/system/oid-console.service').read_text())
                for script in ['preinst', 'postinst', 'prerm', 'postrm', 'config']:
                    (extracted / 'DEBIAN' / script).unlink(missing_ok=True)
                safe = work / f'{name}-safe.deb'
                # Preserve fixture UID/GID so rootless dpkg need not chown to root.
                run('dpkg-deb', '--build', str(extracted), str(safe))
                packages[name] = safe
            for first, second in [('oid', 'talvik'), ('talvik', 'oid')]:
                root = work / f'dpkg-{first}'
                database = root / 'var/lib/dpkg'
                database.mkdir(parents=True)
                (database / 'status').touch()
                command = ['dpkg', f'--root={root}', f'--log={root}/dpkg.log', '--force-not-root', '--force-depends', '--unpack']
                env = {**os.environ, 'LC_ALL': 'C'}
                run(*command, str(packages[first]), env=env)
                before = (root / 'usr/lib/systemd/system/talvikd.service').read_bytes()
                result = subprocess.run(command + [str(packages[second])], env=env,
                                        capture_output=True, text=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn('conflict', result.stderr.lower())
                self.assertEqual((root / 'usr/lib/systemd/system/talvikd.service').read_bytes(), before)


if __name__ == '__main__':
    unittest.main()
