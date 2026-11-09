# ADR-0046: Talvik package and inference integration

- Status: Proposed
- Related: ADR-0012, ADR-0013, ADR-0014, ADR-0025

## Context
Talvik already has standalone binaries, native boundaries, packages, and
model/runtime state.

## Decision
Keep Talvik independently installable as daemon/client packages. MarinaOS
invokes its versioned backend-neutral contract. Evaluate `talvikd`, `talvik-cli`,
and optional integration packages; do not split without evidence.

## Alternatives and consequences
Embedding inference in MarinaOS couples release and hardware concerns. Separate
packages preserve cross-platform use but require version compatibility and health
checks. CPU/GPU claims must reflect actual execution.

## Migration, acceptance, open questions
Qualify existing DEB/RPM/archive workflows on clean Linux and preserve state on
upgrade. Accept install, real inference, health, fallback, and uninstall tests.
Open: final package naming and repository signing service.

## Linux foundation reconciliation — 2026-11-09

Talvik is optional for MarinaOS installation and Linux startup. Preserve standalone APIs and accepted ADR-0014 compatibility profiles. Before package migration, resolve shared /usr/bin/talvik, /usr/bin/talvikd and talvikd.service ownership between oid-marina and talvik; do not co-install competing owners. WP-0089 owns transition qualification. Model inference is an optional-package gate, not M0 boot acceptance.

Evidence and approval boundaries: [reconciliation report](../reports/MARINAOS-LINUX-FOUNDATION-RECONCILIATION-2026-11-09.md).

## Corrective implementation — 2026-11-09

WP-0089 implements mutual DEB Conflicts between oid-marina and talvik to prevent overwriting shared binaries and talvikd.service. A rootless relocated-dpkg fixture tests both installation orders without executing maintainer scripts. This is a protective transition guard; no Replaces field, automatic removal or state migration is authorized.

See [image validation](../validation/marinaos-linux-image.md) and the
[implementation evidence](../reports/MARINAOS-IMAGE-IMPLEMENTATION-2026-11-09.md).
