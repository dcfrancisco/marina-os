# ADR-0052: Installation, packaging, and release qualification

- Status: Proposed
- Related: ADR-0046, ADR-0050, ADR-0051

## Context
Release artifacts must install without a local compiler or llama.cpp build
environment.

## Decision
Qualify native DEB first, RPM where compatible, and per-user archives across
target architectures. Each release includes checksums, native libraries,
service identities, permissions, model-storage policy, clean-machine install,
real inference, upgrade, uninstall, and evidence.

## Alternatives and consequences
Source-only installation excludes intended users. More artifacts increase CI and
signing work but make support claims measurable.

## Migration, acceptance, open questions
Integrate existing Talvik release workflows; do not replace them. Accept clean
installation and end-to-end inference before declaring a package supported.
Open: release repository and signing ownership.

## Linux foundation reconciliation — 2026-11-09

Separate OS qualification from optional Talvik qualification. MarinaOS M0 must install and boot Ubuntu 24.04 amd64 offline with Talvik absent, provide ordinary login/diagnostics/recovery, and preserve state through reboot and package transitions. Native inference is only a Talvik release gate. WP-0088/0089 retain ownership. Firmware, installer and signing acceptance remain Proposed, not proven by existing CI.

Evidence and approval boundaries: [reconciliation report](../reports/MARINAOS-LINUX-FOUNDATION-RECONCILIATION-2026-11-09.md).

## Corrective implementation — 2026-11-09

WP-0088 now has a QEMU/OVMF boot-to-serial-login gate with no network and disposable disk/firmware state. Only an actual QEMU run may qualify boot. Fixture tests do not qualify it; installation, authenticated login, reboot persistence and recovery require separate evidence. WP-0089 now rejects co-installation of oid-marina and talvik using mutual Conflicts metadata; automatic replacement/data migration is deliberately not implemented.

See [image validation](../validation/marinaos-linux-image.md) and the
[implementation evidence](../reports/MARINAOS-IMAGE-IMPLEMENTATION-2026-11-09.md).
