# ADR-0037: Intent-Driven OS consolidation and migration

- Status: Proposed
- Related: ADR-0019, ADR-0027, WP-0075, WP-0104

## Context
`intent-driven-os` is the repository and architecture history, not proof that a
second operating-system product must survive.

## Decision
Consolidate the product direction under MarinaOS documentation while retaining
OID/Talvik code, APIs, paths, and history until replacement contracts qualify.

## Alternatives and consequences
Repository rename or broad rewrite would increase compatibility and rollback risk.
Documentation-first consolidation is slower in visible naming but preserves users,
tests, and release evidence. Security and compatibility review is required for
every later executable rename.

## Migration, acceptance, open questions
Follow `docs/MARINAOS-MIGRATION-PLAN.md`; accept when inventory coverage and
backward-compatibility tests exist. Open: final public repository identity.

## Linux foundation reconciliation — 2026-11-09

M0 Linux foundation supersedes the earlier diagnostic-intent-first migration sequence. Preserve completed code and historical Debian assets. Use existing operation libraries; do not create a second service manager, package manager, model registry or configuration framework. WP-0088/0089 own the foundation. OIP and Delivery products are outside project scope.

Evidence and approval boundaries: [reconciliation report](../reports/MARINAOS-LINUX-FOUNDATION-RECONCILIATION-2026-11-09.md).
