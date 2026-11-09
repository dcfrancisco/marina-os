# MarinaOS product architecture

Current direction: 2026-11-09. MarinaOS is a Linux-based operating system.
The first deliverable is an independently installable and bootable distribution.
The target remains Ubuntu 24.04 LTS amd64, already used by
`docker/validation/ubuntu-24.04/Dockerfile:1`. Debian bookworm remains a historical
BIOS prototype (`image/manifest.yaml:3`, `scripts/build-oid-linux-image.sh:50`).
No blocking Ubuntu issue has been established. Alternate distributions require
recorded evidence and an explicit decision.

The [reconciliation report](reports/MARINAOS-LINUX-FOUNDATION-RECONCILIATION-2026-11-09.md)
contains the source findings, conflict matrix and M0 dependency sequence. Existing
ADRs retain their recorded status; amended proposals still require review.

## Foundation responsibilities

| Component | Responsibility | Current implementation / boundary |
| --- | --- | --- |
| Linux and systemd | Kernel, devices, users, mounts, process supervision, service ordering, resource controls, standard login and recovery targets | Distribution facilities. Existing units use systemd (`packaging/marina/talvikd.service:7`); do not create another init/service manager. |
| marinaosd | Future narrow OS-management composition: read-only system observations, typed requests, existing policy/approval/coordinator/evidence libraries; explicit reconciliation of interrupted operations | READ-ONLY SLICE: `crates/marinaos-core` provides a daemon and peer-UID restricted local diagnostics. No privileged operations or operation recovery are implemented. Its service unit is unqualified; boot and login must survive its absence. |
| marinaosctl | Future deterministic administrative client: status, diagnostics, inspect plans/results, request explicitly authorized operations, inspect recovery | READ-ONLY SLICE: `crates/marinaos-cli` calls the versioned local daemon protocol and reports daemon errors. No embedded model state or inference dependency. Administrative operations remain unimplemented. |
| Packages | apt/dpkg owns package transactions, dependencies, conffiles and package state | Reuse package assets (`scripts/package-oid-deb.sh:68`). MarinaOS may eventually plan and verify authorized transactions; it does not maintain a competing package database. |
| Configuration | Distro conffiles own OS configuration; service environment sets explicit service overrides; runtime files and evidence are separate from configuration | Preserve `/etc/oid`, `/etc/marina`, `/etc/talvik` compatibility as applicable. One authoritative loader per component; no new global configuration system. Resolve duplicate unit/package ownership before migration. |
| Hardware | Linux kernel/udev/sysfs/procfs facts, normalized read-only observations with unknown/error states | Existing Linux skills are reusable (`crates/linux-skills/src/lib.rs:157`). Runtime hardware currently calls llama.cpp device discovery (`runtime/src/hardware.rs:122`); do not import native inference into the OS daemon for hardware discovery. |
| Security | Linux UID/GID, file permissions, service isolation and native authorization enforce privilege; existing policy approves typed plans | Retain `crates/policy-engine/src/lib.rs:98` and coordinator `:227`. An authenticated OS principal is mandatory before service-side actions; model text never grants authority. Do not invent an unrestricted root executor. |
| Logging and diagnostics | journald for service logs; existing evidence store for correlated operation records; CLI reports observed failures honestly | `packaging/oid/oid-console.service:21`; `crates/evidence-engine/src/lib.rs:161`. Persistent audit permissions/retention and reboot behavior must be qualified. Logs do not replace evidence or expose credentials. |
| Recovery | systemd handles process failure; existing operation journals identify incomplete work; distro recovery environment handles boot/package failures | `crates/operation-coordinator/src/lib.rs:579`. No automatic replay of mutation, no assumed A/B images or universal package rollback. Interrupted update, configuration retention and recovery access need VM evidence. |
| Talvik | Optional separately installable model service, sole owner of model registry, artifacts and inference | Implemented runner (`console/src/bin/talvikd.rs:1`, `runtime/src/models.rs:1`). No mandatory M0 package, build input, readiness gate or network/model download during Linux startup. Preserve existing standalone behavior and APIs. |

OID crate names, existing commands, paths, journals and Marina compatibility
aliases stay stable until a tested migration replaces them. The current console
requires Talvik (`console/src/tui.rs:48`); it cannot serve as the only M0 login or
administration surface. Companion UI, desktop, local inference and model
integration implementation are outside M0. OIP and Delivery One/Delivery OS have
no architectural or implementation role in this project.

## Release boundary

Package installation is an intermediate check, not the first product deliverable.
M0 requires a reproducible build recipe and provenance, VM installation and boot,
ordinary local login, offline operation with Talvik absent, reboot persistence,
diagnostics, interrupted-update handling and recovery. Firmware mode and installer
mechanism remain explicit decisions under ADR-0050/0052. A source tree or shell
syntax pass cannot satisfy these gates.
