# ADR-0042: Policy authority and privilege separation

- Status: Proposed
- Related: ADR-0006, ADR-0016, ADR-0027, WP-0104

## Context
AI assistance must not become unrestricted root access.

## Decision
Authority derives from authenticated OS principals and policy, not model text.
Classify operations as read-only, reversible user, privileged reversible,
destructive, security-sensitive, or irreversible. Revalidate approval at the
execution boundary and use native Linux authorization mechanisms.

## Alternatives and consequences
Root daemon simplifies integration but creates catastrophic blast radius. A
narrow user/system boundary adds IPC and packaging work while limiting risk.
Secrets never enter model context unless explicitly authorized.

## Migration, acceptance, open questions
Keep current policy engine; define principal/scope contracts before privileged
services. Accept with unauthorized, approval, replay, and privilege tests. Open:
whether a separate system executor is needed.

## M0 read-only diagnostics implementation

WP-0089 implements a limited local diagnostics boundary. The socket and its
directory admit the `marinaos-readers` group, but the daemon independently
checks the kernel peer UID against root, its service UID, or membership in that
group in the local `/etc/passwd` and `/etc/group` databases. Missing or
ambiguous local records fail closed; directory-backed NSS accounts are not yet
supported. Membership grants only the protocol's read-only methods. It does
not grant approval or authority for future operations. The client checks the
service UID, socket owner and directory for the standard system path. A custom
development socket path requires a trusted directory.
