# MarinaOS roadmap and qualification matrix

Current priority: **M0 — Linux foundation** (2026-11-09).
See the [reconciliation and readiness report](reports/MARINAOS-LINUX-FOUNDATION-RECONCILIATION-2026-11-09.md).
Earlier inference-first milestone sequencing is superseded for the OS product;
existing Talvik milestones retain their independent history.

| M0 step | Existing owner | Required evidence | Status |
| --- | --- | --- | --- |
| Correct image staging, then validate migration | WP-0088, ADR-0050 | Nonprivileged staging regression; Debian history preserved; Ubuntu 24.04 amd64 explicit | Assembly/GPT contracts implemented and tested; actual build unqualified |
| Reconcile packages and boot units | WP-0089, ADR-0046/0052 | Single package owner per path/unit, state retention, no mandatory Talvik | Mutual package conflicts implemented; migration and service qualification open |
| Build Ubuntu foundation | WP-0088 | Pinned inputs, manifest, toolchain/package provenance, final artifact hashes | NOT TESTED |
| Install and boot in disposable VM | WP-0088, ADR-0050/0052 | Firmware-specific boot, install, ordinary login and reboot without Talvik or network | NOT TESTED |
| Diagnostics and recovery | Existing policy/evidence/coordinator; ADR-0042/0043/0051 | Failure, interrupted-update and retained-state evidence; OS principal authorization | INCOMPLETE libraries; NOT TESTED as distribution |
| Qualify release | ADR-0052 | Signatures, checksums, supported hardware/firmware matrix and reproducible procedure | NOT TESTED |

WP-0112–0124 and ADR-0053–0062 are absent from the reviewed checkout and cannot
be marked ready or accepted. Restore their source for reconciliation rather than
allocating duplicate work. Existing WP-0088/0089 remain the foundation owners.

Companion UI, inference, model integration and desktop implementation are deferred.
Talvik remains optional; its tests do not gate Linux startup. External OIP and
Delivery products are excluded, including from acceptance scenarios.

## Milestone acceptance and tags

Completed work packages receive normal commits; completing implementation alone
does not qualify a milestone. Before accepting a milestone, record its qualifying
commit SHA, each documented acceptance criterion, exact test commands and
results, links to logs and artifact checksums, known limitations, and the
explicit acceptance decision. Failures, blocked checks, and untested criteria
remain visible and prevent acceptance. Container evidence cannot establish VM
boot, systemd service behavior or recovery.

For M0, the record must cover the foundation gates in the matrix above, including
an offline disposable-VM installation and boot, authenticated administration,
diagnostics, service and interrupted-update recovery, reboot and state retention,
and the release provenance and signing requirements. M0 is **not accepted**.
The existing package and container evidence does not satisfy these gates.

After acceptance, and only then, create an annotated milestone tag at the
qualifying commit. Never move, overwrite or reuse a tag; preserve existing tags
and releases. A GitHub Release additionally requires verified artifacts, their
checksums, installation instructions and known limitations. Artifact and package
release requirements remain owned by [ADR-0052](adr/0052-installation-packaging-and-release-qualification.md),
WP-0088 and WP-0089; this checklist does not replace their procedures.

The following product milestone names are provisional, pending reconciliation
with the historical [root roadmap](../ROADMAP.md) and ADRs. They do not change
existing tag meanings or establish acceptance criteria for later milestones.

| Milestone | Proposed annotated tag |
| --- | --- |
| M0 Linux foundation qualification | `v0.1.0-m0` |
| M1 Bootable distribution and installer | `v0.2.0-m1` |
| M2 System management and recovery | `v0.3.0-m2` |
| M3 Desktop environment | `v0.4.0-m3` |
| M4 Optional AI integration | `v0.5.0-m4` |

Implementation evidence: [2026-11-09 corrective slice](reports/MARINAOS-IMAGE-IMPLEMENTATION-2026-11-09.md). Next dependency is actual construction and QEMU boot in a disposable privileged build VM.
