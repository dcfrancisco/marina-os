# ADR-0043: Independent supervision and recovery

- Status: Proposed
- Related: ADR-0006, ADR-0023, WP-0056, WP-0062

## Context
Service liveness does not prove task progress, and a watchdog must not escalate.

## Decision
Supervisor observes service health, progress, stalls, resource exhaustion, retry
loops, deadlocks, and inference availability. It may pause/cancel/restart/escalate
within policy and records recovery state without granting authority.

## Alternatives and consequences
Process watchdogs alone miss logical stalls; policy-aware supervision adds state
and evidence. Recovery may produce an unverified state rather than a false success.

## Migration, acceptance, open questions
Extend operation journal and runtime health; accept crash, stall, cancellation,
and reconciliation tests. Open: watchdog placement on image boot.

## Linux foundation reconciliation — 2026-11-09

Clarify supervision ownership: systemd owns process/service lifecycle and restart; the existing coordinator owns operation progress and reconciliation. A separate process supervisor is not authorized. Linux recovery remains available without model or OS-management services. WP-0088 validates boot failure and recovery; logical-stall handling remains later work.

Evidence and approval boundaries: [reconciliation report](../reports/MARINAOS-LINUX-FOUNDATION-RECONCILIATION-2026-11-09.md).
