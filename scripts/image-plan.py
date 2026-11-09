#!/usr/bin/env python3
"""Validate the bounded Ubuntu foundation build contract (not a YAML parser)."""
import argparse
import json
import re
from pathlib import Path


def load_plan(path):
    plan = json.loads(Path(path).read_text())
    fixed = dict(schema='marinaos.image/v1', distribution='ubuntu', release='noble',
                 version='24.04', architecture='amd64', firmware='uefi',
                 partition_table='gpt', secure_boot=False)
    for key, value in fixed.items():
        if type(plan.get(key)) is not type(value) or plan[key] != value:
            raise ValueError(f'unsupported {key}: expected {value!r}')
    if set(plan) != set(fixed) | {'image_size_mib', 'esp_end_mib', 'mirror', 'packages'}:
        raise ValueError('unknown or missing manifest fields')
    if type(plan['image_size_mib']) is not int or not 4096 <= plan['image_size_mib'] <= 65536:
        raise ValueError('image size must be 4096..65536 MiB')
    if plan['esp_end_mib'] != 513:
        raise ValueError('ESP must span 1..513 MiB')
    if plan['mirror'] != 'https://archive.ubuntu.com/ubuntu':
        raise ValueError('unreviewed Ubuntu mirror')
    packages = plan['packages']
    if not isinstance(packages, list) or not packages or any(
            not isinstance(p, str) or not re.fullmatch(r'[a-z0-9][a-z0-9+.-]*', p)
            for p in packages):
        raise ValueError('invalid package list')
    required = {'systemd-sysv', 'linux-generic', 'initramfs-tools',
                'grub-efi-amd64-bin', 'grub2-common', 'sudo'}
    if not required <= set(packages) or len(set(packages)) != len(packages):
        raise ValueError('missing foundation package or duplicate package')
    if any(p.startswith(('talvik', 'oid-', 'llama')) for p in packages):
        raise ValueError('inference/console packages are outside the foundation')
    return plan


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('manifest')
    parser.add_argument('--field', choices=['packages', 'image_size_mib', 'mirror'])
    args = parser.parse_args()
    try:
        plan = load_plan(args.manifest)
        print((' '.join(plan[args.field]) if args.field == 'packages' else plan[args.field])
              if args.field else json.dumps(plan, indent=2, sort_keys=True))
    except (ValueError, OSError, TypeError) as error:
        parser.exit(2, f'invalid image plan: {error}\n')
