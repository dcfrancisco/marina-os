# MarinaOS Linux foundation reconciliation

Date: 2026-11-09. Source: `main`, `c6edcdba71e67b21079f31d946cae8c190a79798`;
working tree was clean before this review. This is a source assessment and
reconciliation, not boot or release qualification. Repository-relative `path:line`
references identify evidence; line numbers refer to the reviewed source unless
explicitly described as reconciliation additions.

## Outcome and authority

MarinaOS is the Linux operating-system product. M0 is an independently installable,
bootable Ubuntu 24.04 LTS amd64 foundation with usable local administration,
diagnostics and recovery without Talvik, models, network access or a companion UI.
The user-directed product scope supersedes earlier runner-first sequencing.
Existing accepted security, Rust, backend-neutrality and operation-lifecycle
decisions remain intact. Formal ADR amendments below remain Proposed unless
explicitly withdrawn; this review does not silently accept their implementation.

**INCOMPLETE:** there is no qualified MarinaOS distribution in this checkout.
WP-0088 explicitly leaves boot unverified (`docs/work-packages/WP-0088-oid-bootable-linux-image.md:46`);
the only image manifest targets Debian (`image/manifest.yaml:1`). Ubuntu 24.04 is
an existing qualification baseline, not an implemented image definition
(`docker/validation/ubuntu-24.04/Dockerfile:1`,
`.github/workflows/talvik-linux-qualification.yml:29`). No observed evidence
justifies changing that target to this review host's Ubuntu 26.04.

**VERIFIED absence in reviewed tree:** ADR-0053–0062 and WP-0112–0124 are absent.
The tracked indexes end at ADR-0052 and WP-0111 respectively
(`docs/adr/README.md:55`, `docs/work-packages/README.md:117`). The inventory
appendix records all existing files and source-declared statuses. No content,
approval, allocation or dependency is invented for the missing ranges. Restore
those proposals from their authoring branch before comparing their substance;
continue existing WP-0088/0089 correction work in the meantime.

## Architecture assessment and implementation versus design

| Finding | Status | Repository evidence | Consequence / disposition |
| --- | --- | --- | --- |
| Rust workspace and bounded operation libraries exist | VERIFIED source; tests below | `Cargo.toml:3`; `crates/common/src/operation.rs:1`; `crates/operation-coordinator/src/lib.rs:227` | Reuse these libraries; do not replace the coordinator with a new orchestrator. |
| Approval and evidence have durable file implementations | VERIFIED source; OS-wide authorization INCOMPLETE | `crates/policy-engine/src/lib.rs:213`; `crates/evidence-engine/src/lib.rs:161`; `crates/operation-coordinator/src/lib.rs:579` | Preserve journals, approval semantics and explicit recovery. File persistence does not prove root authorization or crash-safe OS upgrades. |
| Linux health, process, filesystem and systemd inspection exist | VERIFIED source | `crates/linux-skills/src/lib.rs:157`, `:222`, `:293`, `:385` | Reuse read-only adapters. Systemd health runs `systemctl is-system-running`; it is not a service manager. |
| Desktop D-Bus transport is a trait | INCOMPLETE | `crates/desktop-shell/src/lib.rs:43` | No production bus or desktop is inferred. Defer desktop work. |
| marinaosd and marinaosctl are not binaries | INCOMPLETE | `console/Cargo.toml:10` through binary declarations; `Cargo.toml:3` | Names describe future composition and client responsibilities only. Existing talvikd cannot be renamed into an OS daemon. |
| Console depends on Talvik and rejects packaged backend | CONFLICTING | `console/src/tui.rs:40`; `packaging/oid/oid-console.service:12`; `console/src/ipc.rs:33` | `marina-http` causes immediate startup failure. Even corrected IPC requires Talvik and matching socket permissions. Exclude this console from M0 boot and keep an ordinary login shell. |
| Hardware probing is inference-oriented | INCOMPLETE for OS inventory | `runtime/src/hardware.rs:105` (calls LlamaCppAdapter::devices at 122) | Reuse Linux CPU/memory facts through a neutral boundary; do not pull inference/native libraries into marinaosd to discover hardware. Storage/network/input/firmware qualification remains open. |
| Talvik daemon and native boundary are implemented | VERIFIED source; live qualification NOT TESTED here | `console/src/bin/talvikd.rs:1`; `runtime/adapters/llama-cpp/sys/build.rs:1`; `runtime/src/service.rs:1` | Preserve standalone runner, model registry and compatibility APIs as optional packages. |
| Release workflows build Talvik, not an OS installer | VERIFIED source; CI execution NOT TESTED | `.github/workflows/talvik-release.yml:155`; `.github/workflows/talvik-linux-qualification.yml:29`; `.github/workflows/talvik-api-client-qualification.yml:18` | Existing checks are reusable regression gates; none establishes VM boot/install/recovery. Strict release lint command also allows module-name repetitions at release.yml:33. |
| Old milestone and closeout claims are scoped evidence | NOT TESTED in this review unless listed below | `docs/MARINA-RELEASE-READINESS.md:65`; `docs/TALVIK-DISTRIBUTION-BOOTSTRAP-CLOSEOUT.md:15`; `docs/work-packages/WP-0078-marina-linux-native-qualification.md:47` | Preserve historical results; Ubuntu 26.04 native runner evidence does not qualify Ubuntu 24.04 boot media. |

## Image and packaging blockers

| ID | Status | Evidence and correction |
| --- | --- | --- |
| F01 | CONFLICTING | `image/manifest.yaml:7` declares raw GPT; `scripts/build-oid-linux-image.sh:87` makes msdos/MBR and `:96` installs BIOS GRUB. Explicitly describe Debian BIOS prototype and select/qualify Ubuntu firmware modes before changing them. |
| F02 | CONFLICTING | Builder `:90–93` mounts a newly formatted filesystem then redirects into `$image_root/etc/fstab` before copying `/etc`; the directory is absent in the normal fresh-image path. Copy root first, then write fstab. This is a static failure inference, not an executed disk test. |
| F03 | INCOMPLETE | Builder `:50–52` uses rolling Debian mirrors/unversioned apt packages; `:99` hashes the image while still mounted. Manifest `image/manifest.yaml` is never consumed by the builder. Pin inputs, reconcile manifest with composition, unmount before final digest, record toolchain/package provenance. A checksum alone is not reproducibility. |
| F04 | CONFLICTING | Builder `:23`, `:54–78` requires, installs and enables Talvik plus console. `packaging/oid/oid-readiness.service:3` pulls Talvik with Wants (not Requires), so a failed model service is not a hard systemd boot dependency, but composition and qualification still require it. M0 must build and boot without these packages. |
| F05 | INCOMPLETE | `scripts/qualify-oid-image.sh:11–27` requires Talvik and reports status passed while inference/operation/reboot fields remain unverified. Keep this historical integration helper; define an independent Linux M0 gate with offline boot, login, reboot and recovery evidence. |
| F06 | CONFLICTING | `scripts/package-oid-deb.sh:44–61` and `scripts/package-talvik-deb.sh:42–48` both own `/usr/bin/talvik`, `/usr/bin/talvikd`, `/usr/lib/systemd/system/talvikd.service`; control stanzas at `:68` and `:49` lack Conflicts/Replaces. Do not co-install until package transition semantics and retained state are tested. Prefer one optional Talvik package owner. |
| F07 | INCOMPLETE | `scripts/build-oid-linux-image.sh:52`, `:85–97` builds a raw root disk; there is no user-account provisioning or install-to-target flow in this builder. It is not an independently installable distribution installer. Plan disposable-VM installation, credentials, bootloader, network setup and recovery before release. |
| F08 | INCOMPLETE | `scripts/package-oid-deb.sh:75` lists libc/systemd but ships native C++ libraries; `:85–98` suppresses account/tmpfiles errors and lacks service stop on removal. Qualify actual ELF dependencies and maintainer-script failures, rather than borrowing a runner package success claim. |
| F09 | CONFLICTING | `packaging/oid/oid-readiness:17` considers any response containing `"data"` ready, including an empty list; it proves neither loaded model nor OS health. Define readiness by the component being measured; OS readiness must not depend on inference readiness. |

No builder, installer, package maintainer script, mount, partition, format or
host-service mutation was executed for this assessment. Debian artifacts remain
historical source, with defects documented rather than silently migrated.

## ADR conflict matrix and supersession

All ADRs are enumerated in the inventory appendix. The following is the actionable
conflict matrix; accepted records keep their historical status and receive only
explicit scope notes where needed.

| Records | Status / conflict | Reconciliation and approval needed |
| --- | --- | --- |
| 0001, 0019 | CONFLICTING runner-first proof and OID brand (`0001:11`, `0019:1`) | Product identity and first-deliverable portions superseded in current planning by the user's MarinaOS direction. ADR-0036/0037 amendments record this; formal acceptance of these amendments remains required. |
| 0002–0009 | VERIFIED accepted contracts, implementation varies | Preserve Rust, neutrality, client separation, policy and planning. No inference dependency may be imposed on Linux boot. ADR-0005's original console wording is historical; existing explicit shell behavior remains. |
| 0010–0016, 0025–0026, 0028–0029, 0031–0035 | VERIFIED existing runner/security decisions | Retain for optional Talvik and governed operations; no supersession of APIs, model defaults or native safety. These do not define M0 prerequisites. |
| 0014, 0046 | CONFLICTING legacy Marina system paths and current Talvik packaging | Keep 0014 accepted profiles as compatibility obligations; 0046 must approve package ownership/migration with data retention, no duplicate unit or registry. Optional inference qualification is separate from M0. |
| 0017–0018, 0020–0021, 0024, 0030 | INCOMPLETE/deferred outside M0 | Models, Clipper, training, remote inference and Windows installers remain separate historical work. No completion or rejection inferred from source. |
| 0023, 0050 | CONFLICTING mandatory AI composition and package-first release framing | Amend existing proposals for Ubuntu 24.04 LTS amd64, independently installable/bootable end deliverable, optional Talvik and Debian preservation. Approval needed for image/installer/firmware contract, not for inspection or isolated correction work. |
| 0027, 0036–0037 | CONFLICTING umbrella/orchestrator-first planning | Consolidate in existing 0036/0037; no second OS control plane. Formal proposed status remains. |
| 0038–0045, 0047 | INCOMPLETE advanced intent, agents, memory, proactive help and resource plans | Defer AI features. Preserve 0039–0042 contract work. Amend 0043: systemd supervises processes; journal reconciliation handles operations. Any new privileged service needs principal/authentication review under 0042. |
| 0048–0049 | CONFLICTING excluded external products | Withdraw proposals from active project architecture on user direction, retaining historical text. OIP and Delivery One/Delivery OS have no adapter, dependency, API or qualification role. |
| 0051 | INCOMPLETE update/rollback mechanism | Keep apt/dpkg package ownership and state retention; approve signing authority, rollback limits, recovery artifacts and interrupted-update procedure. Do not imply A/B partitions exist. |
| 0052 | CONFLICTING inference gate applied to OS | Amend: Linux OS qualification is independent from optional Talvik native inference qualification. Approval required before a supported-release claim. |
| 0053, 0054, 0055, 0056, 0057, 0058, 0059, 0060, 0061, 0062 | INCOMPLETE: absent | Cannot assess or approve unseen proposals. Do not create replacement IDs. Review against 0023/0036/0037/0042/0043/0046/0050–0052 when restored. |

ADR filename/line evidence for every row and exact status is in the appendix;
all decision bodies remain at their existing paths. Proposed records are not
made accepted by this report. OIP/Delivery withdrawal is a scope instruction,
not approval of any replacement external integration.

## Duplicate configuration and subsystem inventory

| Surface | Status / evidence | Owner and disposition |
| --- | --- | --- |
| `/etc/marina`, `/var/lib/marina`, `/run/marina`, port 12434 | CONFLICTING with second system profile; `packaging/marina/talvikd.service:14` | Legacy runner profile. Preserve state and aliases until explicit package migration. |
| `/etc/talvik`, `/var/lib/talvik`, `/run/talvik`, port 11434 | VERIFIED standalone profile; `packaging/talvik/talvikd.service:12` | Talvik package; never a second OS config store. Resolve shared unit/file ownership before coexistence. |
| `$HOME/.marina`, TALVIK_* and MARINA_* | VERIFIED compatibility aliases; `runtime/src/config.rs:177`, `:206`, `:372`; `console/src/ipc.rs:33` | One runner loader: TALVIK override then MARINA compatibility alias then configured/default value where supported. Preserve per-user data. Not every TALVIK option has a MARINA alias. |
| System YAML versus unit environment | CONFLICTING defaults; `packaging/marina/config.yaml:1`; `packaging/talvik/config.yaml:8`; `runtime/src/config.rs:1` | Units override YAML paths/listeners. Legacy config lacks the explicit auto-bootstrap/download/load disables of current Talvik config. Verify offline behavior instead of assuming comments enforce it. |
| OID_MARINA_* / OID_RUNTIME_BACKEND | CONFLICTING obsolete HTTP console configuration; `packaging/oid/oid-console.service:11`; `console/src/tui.rs:40` | Optional console migration task; remove reliance from M0. Keep existing public aliases until migration tested. |
| sysusers/tmpfiles marina, talvik, oid | VERIFIED separate identities; `packaging/sysusers/marina.conf:2`; `packaging/sysusers/talvik.conf:2`; `packaging/tmpfiles/oid.conf:1` | Reuse systemd mechanisms. Separate operation evidence from model state; do not merge identities to fix permissions. |
| Runtime config and shared/common contracts | VERIFIED distinct scopes; `runtime/src/config.rs:1`; `shared/src/config.rs:5`; `crates/common/src/lib.rs:11` | Shared RuntimeConfig is runner data; common owns operation contracts. Do not create a third runner config parser in marinaosd. |
| Operation journal, approval journal, evidence store | VERIFIED distinct responsibilities; coordinator `:111`, policy `:213`, evidence `:161` | Correlate records; do not collapse audit and approval into a replacement database merely because all persist files. |
| Model registry versus package inventory | VERIFIED separation; `runtime/src/models.rs:1`; `scripts/package-talvik-deb.sh:49` | Talvik alone owns models; apt/dpkg owns installed OS packages. marinaosd observes or requests governed package operations. |
| Image manifest versus emitted manifest | CONFLICTING duplicated composition descriptions; `image/manifest.yaml:1`; builder `:102` | Establish one validated build input and separately emit observed output provenance. |
| Root roadmap, runtime roadmap, product roadmap | CONFLICTING priorities; `ROADMAP.md:21`; `runtime/ROADMAP.md:107`; product roadmap historical milestones | M0 is the product priority. Runtime roadmap remains a scoped optional-component backlog. |
| Python HTTP proxy versus native HTTP | VERIFIED migration edge; `scripts/marina-openai-proxy.py:1`; `console/src/bin/http.rs:1` | Keep compatibility artifact; do not add either to the Linux startup path. |

## M0 ownership and work-package readiness

`docs/MARINAOS-PRODUCT-ARCHITECTURE.md` defines the reconciled responsibilities.
No new service manager, package manager, registry, configuration framework,
desktop or inference implementation is authorized by this review.

| Existing WP / requested range | Readiness and dependencies | M0 disposition |
| --- | --- | --- |
| WP-0088 | INCOMPLETE; F01–F05/F07; depends on image input and package ownership decisions | Primary M0 image/boot owner. Narrow foundation acceptance independent of old AI demonstration. |
| WP-0089 | INCOMPLETE; F06/F08; historical dpkg evidence at WP-0089:49 | Preserve old package; plan transition to OS packages without bundling mandatory inference. |
| WP-0069 / WP-0071 | INCOMPLETE for OS; acceptance at WP-0069:25 / WP-0071:18 | Reuse Linux packaging/security lessons. Runner API/provider/training dependencies do not gate M0. |
| WP-0043–0054 | VERIFIED source foundation; individual source status in appendix | Preserve approval, journal, inspection and recovery libraries; test these before composing a daemon. |
| WP-0056 | NOT TESTED on booted Ubuntu 24.04 | Retain signal/recovery regressions; no substitute for actual service restart/reboot testing. |
| WP-0075 / WP-0104 | INCOMPLETE brand and execution-contract migration | Reconcile naming in docs now; authenticated local daemon/client contract must precede privileged daemon work. |
| WP-0078 | Historical qualified Ubuntu 26.04 runner, NOT TESTED here | Optional runner evidence, not Ubuntu 24.04 OS acceptance. |
| WP-0096/0097/0100/0102/0103 | INCOMPLETE ledger-only entries | Index entries exist at work-packages/README.md:102 onward, without matching standalone WP files. Preserve allocations; do not manufacture completion. |
| WP-0112, WP-0113, WP-0114, WP-0115, WP-0116, WP-0117, WP-0118, WP-0119, WP-0120, WP-0121, WP-0122, WP-0123, WP-0124 | INCOMPLETE: absent from tracked files and ledger | No trustworthy per-ID mapping or readiness claim possible. Reconcile restored proposals with existing owners before allocating work. |

M0 corrective implementation is **ready to begin in bounded slices** using
WP-0088/0089. M0 release/build qualification is **not ready**. Privileged daemon,
installer disk layout and supported-release claims need the concrete ADR reviews
above. Companion UI, local inference, model integration and desktop work are
excluded from this sequence; existing functionality stays intact.

Dependency order: source/manifest preflight → Ubuntu composition and package
ownership → isolated image build → disposable VM boot and ordinary login without
Talvik/network → VM installation and reboot → offline diagnostics, interrupted
update/recovery and state retention → release provenance/signing evidence.
Firmware mode and installer mechanism must be explicit before VM installation.
Debian BIOS is historical; Ubuntu UEFI and BIOS support are qualification choices,
not interchangeable success claims. Debian continuation is an alternative only
if a blocking Ubuntu issue is demonstrated and approved.

## Smallest concrete implementation task

Under **WP-0088**, correct root-filesystem staging order in the existing Debian
prototype builder: copy the prepared root into the mounted image before writing
`/etc/fstab`, then ensure the intended UUID entry survives. Evidence is the
statically visible ordering defect at `scripts/build-oid-linux-image.sh:90–93`.
This is a bounded correction to preserved historical work, not a distribution
switch or new architecture.

Acceptance: a nonprivileged fixture/stub test reproduces the missing `etc`
directory failure and verifies the corrected copy/write order and final fstab
content; shell syntax passes; no host block device is touched. No boot claim is
made. Follow with final-hash-after-unmount correction, explicit manifest/build
validation and a separate Ubuntu 24.04 amd64 foundation profile that requires no
Talvik artifacts. Keep that profile plan-only until its construction and VM
qualification actually run. These are next implementation tasks; this change
only reconciles architecture and evidence.

## Checks and evidence

See the [verification record](MARINAOS-LINUX-FOUNDATION-CHECKS-2026-11-09.md) and [complete ADR/WP inventory](MARINAOS-LINUX-FOUNDATION-INVENTORY-2026-11-09.md) for exact commands, exit statuses and
limits. Source inspection is VERIFIED only for what the source says. Boot,
installation, firmware support, signing, hardware compatibility and recovery
remain NOT TESTED. No claim in this report converts a documentation status or
successful syntax check into runtime qualification.

## Implementation follow-up — 2026-11-09

The [corrective implementation report](MARINAOS-IMAGE-IMPLEMENTATION-2026-11-09.md)
records subsequent work: F01 now distinguishes historical Debian BIOS/MBR from
Ubuntu UEFI/GPT; F02 has a tested copy-before-fstab fix; F03 final hashing now
follows unmount (archive pinning remains open); the new Ubuntu foundation removes
F04's mandatory inference composition; F06 has mutual package-conflict guards,
with automatic migration still open. Actual construction and QEMU boot remain
NOT TESTED. The assessment above is retained as the pre-implementation snapshot.
