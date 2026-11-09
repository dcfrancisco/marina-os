# ADR-0048: OIP intelligence interoperability

- Status: Withdrawn from MarinaOS scope (2026-11-09)
- Related: ADR-0015, ADR-0038

## Context
OIP can provide remote models and coordination but must not control the host.

## Decision
Expose OIP only through an authenticated, capability-scoped intelligence adapter
for discovery, remote inference, embeddings, fallback, and diagnostics. MarinaOS
remains useful with OIP disconnected; local policy owns all execution authority.

## Alternatives and consequences
OIP-only simplifies provider selection but violates offline capability. Optional
integration adds failure and privacy handling, addressed by explicit provider
identity, consent, and transmission policy.

## Migration, acceptance, open questions
Add adapter after local Talvik proof. Accept disconnect, consent, redaction, and
fallback tests. Open: stable OIP protocol version.

## Scope withdrawal — 2026-11-09

Withdrawn on explicit product direction. OIP and Delivery One/Delivery OS are
outside MarinaOS architecture and implementation. The proposal above is retained
only as historical context; no adapter, contract, dependency or acceptance gate
is authorized. See the [reconciliation report](../reports/MARINAOS-LINUX-FOUNDATION-RECONCILIATION-2026-11-09.md).
