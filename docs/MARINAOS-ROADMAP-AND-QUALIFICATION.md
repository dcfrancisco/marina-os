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

Implementation evidence: [2026-11-09 corrective slice](reports/MARINAOS-IMAGE-IMPLEMENTATION-2026-11-09.md). Next dependency is actual construction and QEMU boot in a disposable privileged build VM.
