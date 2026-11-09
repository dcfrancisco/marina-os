# ADR-0050: Ubuntu-based distribution and image strategy

- Status: Proposed
- Related: ADR-0014, ADR-0023, WP-0088, WP-0089

## Context
The product must become installable, but an installer rewrite is not the first
proof.

## Decision
Start with installable packages on supported Ubuntu LTS, then build a pinned,
reproducible image and qualify boot in a VM before installer/hardware work.
Support headless and desktop operation where practical; use normal Linux/systemd
security boundaries.

## Alternatives and consequences
A custom distro first delays useful behavior. Ubuntu packages accelerate proof
but inherit base-distro lifecycle and hardware constraints.

## Migration, acceptance, open questions
Use existing packaging/image assets. Accept clean install, offline startup,
upgrade, boot, degraded AI, and recovery tests. Open: Ubuntu release and desktop.

## Linux foundation reconciliation — 2026-11-09

Resolve the baseline to Ubuntu 24.04 LTS amd64, matching existing qualification definitions. Independently installable and bootable Linux is the first deliverable; package checks are intermediate evidence. Keep Debian bookworm BIOS prototype unchanged as history until migration is understood. M0 has no Talvik/model/UI prerequisite. WP-0088/0089 own image and package migration. Firmware modes, installer mechanism, pinned build inputs and clean VM install/boot/recovery evidence must be reviewed before release. Alternative distributions require a documented blocking issue and explicit decision.

Evidence and approval boundaries: [reconciliation report](../reports/MARINAOS-LINUX-FOUNDATION-RECONCILIATION-2026-11-09.md).

## Corrective implementation — 2026-11-09

The user authorized UEFI/GPT implementation on 2026-11-09. The implemented Ubuntu 24.04 amd64 profile uses a 512 MiB FAT32 ESP (1–513 MiB), ext4 root, and GRUB x86_64-efi at the removable fallback path with no NVRAM writes. Secure Boot is not supported by this slice. The Debian bookworm BIOS/MBR profile is retained separately; the inaccurate GPT declaration in its manifest is corrected. This accepts the bounded firmware/composition choice for WP-0088 implementation, not the entire proposed ADR or release qualification.

See [image validation](../validation/marinaos-linux-image.md) and the
[implementation evidence](../reports/MARINAOS-IMAGE-IMPLEMENTATION-2026-11-09.md).
