# MarinaOS reconciliation inventory — 2026-11-09

Source: `c6edcdba71e67b21079f31d946cae8c190a79798`. All source-declared statuses below are historical metadata, **not functionality evidence**. Current amendments are in the original records. See the [assessment](MARINAOS-LINUX-FOUNDATION-RECONCILIATION-2026-11-09.md) and [checks](MARINAOS-LINUX-FOUNDATION-CHECKS-2026-11-09.md).

The complete [tracked source listing](evidence/marinaos-foundation-2026-11-09/tracked-files.txt) proves the scope of the absence findings. ADR-0022 is also absent; numeric continuity must not be assumed. No `distro/` directory is present; distribution assets live under `image/`, `packaging/`, `docker/` and `scripts/`.

## ADRs

| Source / original line | Declared status | Scope / disposition |
| --- | --- | --- |
| `docs/adr/0001-project-vision.md:3` | Accepted | ADR-0001: Project Vision. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0002-rust-primary-language.md:3` | Accepted | ADR-0002: Rust as the Primary Implementation Language. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0003-backend-neutral-runtime.md:3` | Accepted | ADR-0003: Backend-Neutral Runtime. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0004-runtime-console-separation.md:3` | Accepted | ADR-0004: Runtime and Console Separation. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0005-intent-based-operations.md:3` | Accepted | ADR-0005: Intent-Based Operations. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0006-security-by-default.md:3` | Accepted | ADR-0006: Security by Default. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0007-hardware-aware-runtime.md:3` | Accepted | ADR-0007: Hardware-Aware Runtime. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0008-plugin-architecture.md:3` | Accepted | ADR-0008: Plugin Architecture. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0009-operation-planning-framework.md:3` | Accepted | ADR 0009: Canonical operation planning framework. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0010-oid-model-runner-boundary.md:3` | Accepted | ADR-0010: OID-Owned Local Model Runner Boundary. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0011-local-runner-and-omnirouter-strategy.md:3` | Accepted | ADR-0011: OID-Owned Local Runner and Multi-Backend Router Strategy. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0012-individual-model-runner-installation.md:3` | Accepted | ADR-0012: Individually Installable Model Runner. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0013-cross-platform-standalone-runner-api.md:3` | Accepted | ADR-0013: Cross-Platform Standalone Runner and Assistant API. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0014-linux-distribution-and-service-integration.md:3` | Accepted | ADR-0014: Linux Distribution and Service Integration. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0015-ai-capability-and-provider-contract.md:3` | Accepted | ADR-0015: AI Capability, Provider, and Assistant Contract. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0016-ai-desktop-self-management.md:3` | Accepted | ADR-0016: AI Desktop Self-Management Boundary. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0017-oid-os-management-model.md:3` | Proposed | ADR-0017: OID OS-Management Model Track. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0018-rust-clipper-linux-subsystem.md:3` | Proposed | ADR-0018: Rust CA-Clipper Linux Subsystem. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0019-product-brand-architecture.md:3` | Accepted | ADR-0019: OID and Standalone Runtime Product Brand Architecture. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0020-marina-model-training-fine-tuning-and-lifecycle-management.md:3` | Proposed | ADR-0020: Marina Model Training, Fine-Tuning, and Lifecycle Management. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0021-marina-transport-api-and-authorization.md:3` | Proposed | ADR-0021: Marina Transport, API, and Authorization Architecture. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0023-oid-linux-distribution-boot-lifecycle-and-system-services.md:3` | Proposed | ADR-0023: OID Linux Distribution, Boot Lifecycle, and System Services. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0024-marina-windows-native-build-and-qualification.md:3` | Proposed | ADR-0024: Marina Windows Native Build and Qualification Requirements. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0025-talvik-runtime-scheduling-and-ide-interoperability.md:3` | Accepted for M2 implementation | ADR-0025: Talvik Runtime Scheduling, Resource Governance, and IDE Interoperability. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0026-talvik-native-inference-performance.md:3` | Accepted for incremental M2 implementation | ADR-0026: Talvik Native Inference Performance and Worker Isolation. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0027-marinaos-product-and-execution-architecture.md:3` | Proposed | ADR-0027: MarinaOS Product and Execution Architecture Proposal. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0028-inference-performance-measurement.md:3` | Accepted | ADR-0028: Inference Performance Measurement. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0029-talvik-optional-vulkan-acceleration.md:3` | Accepted; CUDA/Vulkan/Metal hardware qualification pending | ADR-0029: Optional Vulkan Acceleration for Talvik. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0030-talvik-per-user-windows-msi.md:3` | Proposed | ADR-0030: Per-User MSI Packaging for Talvik on Windows. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0031-openai-inference-api-compatibility.md:3` | Accepted | ADR-0031: OpenAI Inference API Compatibility. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0032-talvik-ollama-model-runner-api.md:3` | Accepted | ADR-0032: Talvik Ollama-Compatible Model Runner API. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0033-talvik-local-vision-input.md:3` | Accepted | ADR-0033: Talvik Local Vision Input. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0034-talvik-gemma-4-default-model.md:3` | Accepted | ADR-0034: Gemma 4 E4B-it as Talvik's Default Model. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0035-native-embedded-chat-templates.md:5` | Accepted | ADR-0035: Execute Embedded Chat Templates. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0036-marinaos-product-identity.md:3` | Proposed | ADR-0036: MarinaOS product identity and intent-driven architecture. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0037-intent-driven-os-consolidation.md:3` | Proposed | ADR-0037: Intent-Driven OS consolidation and migration. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0038-native-marina-intelligence.md:3` | Proposed | ADR-0038: Native Marina Intelligence. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0039-intent-schema-and-execution-lifecycle.md:3` | Proposed | ADR-0039: Intent schema and execution lifecycle. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0040-durable-agent-orchestration.md:3` | Proposed | ADR-0040: Durable agent orchestration. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0041-linux-capability-and-tool-contracts.md:3` | Proposed | ADR-0041: Linux capability and tool contracts. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0042-policy-authority-and-privilege-separation.md:3` | Proposed | ADR-0042: Policy authority and privilege separation. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0043-independent-supervision-and-recovery.md:3` | Proposed | ADR-0043: Independent supervision and recovery. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0044-context-and-user-memory-lifecycle.md:3` | Proposed | ADR-0044: Context and user memory lifecycle. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0045-proactive-system-assistance.md:3` | Proposed | ADR-0045: Policy-bounded proactive system assistance. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0046-talvik-package-and-inference-integration.md:3` | Proposed | ADR-0046: Talvik package and inference integration. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0047-hardware-and-model-resource-admission.md:3` | Proposed | ADR-0047: Hardware and model resource admission. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0048-oip-intelligence-interoperability.md:3` | Proposed | ADR-0048: OIP intelligence interoperability. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0049-delivery-os-execution-interoperability.md:3` | Proposed | ADR-0049: Delivery OS execution interoperability. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0050-ubuntu-distribution-and-image-strategy.md:3` | Proposed | ADR-0050: Ubuntu-based distribution and image strategy. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0051-software-updates-rollback-and-recovery.md:3` | Proposed | ADR-0051: Software updates, rollback, and recovery. Preserve existing scope; see conflict matrix for M0 applicability. |
| `docs/adr/0052-installation-packaging-and-release-qualification.md:3` | Proposed | ADR-0052: Installation, packaging, and release qualification. Preserve existing scope; see conflict matrix for M0 applicability. |
## Work packages

| Source / original line | Declared status | Scope / disposition |
| --- | --- | --- |
| `docs/work-packages/WP-0001-repository-foundation.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0001: Repository Foundation. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0002-runtime-architecture-documentation.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0002: Runtime Architecture Documentation. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0003-backend-adapter-interface.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0003: Backend Adapter Interface. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0004-model-registry-design.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0004: Model Registry Design. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0005-hardware-discovery-design.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0005: Hardware Discovery Design. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0006-ai-console-design.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0006: AI Console Design. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0007-command-grammar.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0007: Command Grammar. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0008-model-lifecycle.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0008: Model Lifecycle. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0009-observability.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0009: Observability. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0010-security-architecture.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0010: Security Architecture. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0016-interactive-console.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0016: Interactive Console. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0017-adaptive-cursor-system.md:3` | Proposed | WP-0017: Adaptive Presence Indicator. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0018-runtime-event-bus.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0018: Runtime Event Bus. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0019-status-bar.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0019: Status Bar. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0020-mock-runtime.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0020: Mock Runtime. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0021-startup-experience.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0021: Startup Experience. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0022-backend-interface.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0022: Backend Interface. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0023-backend-registry.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0023: Backend Registry. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0024-model-registry.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0024: Model Registry. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0025-hardware-discovery.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0025: Hardware Discovery. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0026-llama-cpp-adapter-skeleton.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0026: llama.cpp Adapter Skeleton. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0027-native-llama-cpp-integration.md:31` | Lifecycle, GGUF loading, streaming generation, cooperative cancellation, and | WP-0027: Native llama.cpp Integration. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0028-model-discovery.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0028: Model Discovery. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0029-model-loader.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0029: Model Loader. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0030-tokenizer.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0030: Tokenizer. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0031-inference-engine.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0031: Inference Engine. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0032-streaming-generation.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0032: Streaming Generation. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0033-console-integration.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0033: Console Integration. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0034-runtime-metrics.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0034: Runtime Metrics. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0035-cancellation.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0035: Cancellation. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0036-generation-events.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0036: Generation Events. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0037-dynamic-cli-capability-framework.md:3` | Backlog — high priority, after the terminal foundation. | WP-0037: Dynamic CLI Capability Framework. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0038-operation-planning-framework.md:3` | Complete — Milestone 3. | WP-0038: Operation Planning Framework. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0039-operation-registry-and-allowlist.md:3` | Complete — Milestone 4. | WP-0039: Operation Registry and Allowlist. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0040-native-dynamic-intent-router.md:3` | Complete — Milestone 4. | WP-0040: Native/Dynamic/Intent Input Router. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0041-dynamic-capability-registration.md:3` | Complete — Milestone 4. | WP-0041: Dynamic Capability Registration. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0042-help-and-completion-integration.md:3` | Complete — Milestone 4. | WP-0042: Help and Completion Integration. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0043-durable-approval-and-recovery.md:3` | Complete — Milestone 4. | WP-0043: Durable Approval and Recovery. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0044-read-only-skill-expansion.md:3` | Complete — Milestone 4. | WP-0044: Read-only Skill Expansion. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0045-operation-coordinator.md:3` | Complete — Milestone 5. | WP-0045: Operation Coordinator. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0046-persistent-operation-state-machine.md:3` | Complete — Milestone 5. | WP-0046: Persistent Operation State Machine. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0047-native-command-execution-adapter.md:3` | Complete — Milestone 5. | WP-0047: Native Command Execution Adapter. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0048-dynamic-capability-execution-adapter.md:3` | Complete — Milestone 5. | WP-0048: Dynamic Capability Execution Adapter. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0049-evidence-verification-correlation.md:3` | Complete — Milestone 5. | WP-0049: Evidence and Verification Correlation. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0050-recovery-and-resume-cli-boundary.md:3` | Complete — Milestone 5. | WP-0050: Recovery and Resume CLI Boundary. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0051-process-inspection-skill.md:3` | Complete — Milestone 6. | WP-0051: Process Inspection Skill. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0052-filesystem-inspection-skill.md:3` | Complete — Milestone 6. | WP-0052: Filesystem Inspection Skill. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0053-systemd-health-adapter.md:3` | Complete — Milestone 6. | WP-0053: Systemd Health Adapter. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0054-dbus-desktop-boundary.md:3` | Complete — Milestone 6. | WP-0054: D-Bus Desktop Boundary. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0055-terminal-identity-and-shell-passthrough.md:3` | Complete — Milestone 7. | WP-0055: Terminal Identity and Shell Passthrough. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0056-process-and-signal-reliability.md:3` | Implemented for the tested process harness — macOS and Ubuntu 24.04 Docker | WP-0056: Process and Signal Reliability. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0057-local-runner-contract-hardening.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0057: Local Runner Contract Hardening. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0058-verified-model-cache-and-admission.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0058: Verified Model Cache and Admission. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0059-runtime-api-and-stream-transport.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0059: Runtime API and Stream Transport. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0060-backend-capability-and-router.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0060: Backend Capability and Router. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0061-compatibility-edges.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0061: Compatibility Edges. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0062-runtime-recovery-and-observability.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0062: Runtime Recovery and Observability. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0063-assistant-session-and-tool-boundary.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0063: Assistant Session and Tool Boundary. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0064-individual-model-runner-installer.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0064: Individual Model Runner Installer. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0065-cross-platform-runner-packaging.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0065: Cross-Platform Runner Packaging. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0066-standalone-service-transports.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0066: Standalone Service Transports. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0067-assistant-api-compatibility-edge.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0067: Assistant API Compatibility Edge. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0068-oid-runtime-integration.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0068: OID Runtime Integration. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0069-linux-distro-packaging-and-service.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0069: Linux Distro Packaging and Service. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0070-ai-capability-and-provider-contract.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0070: AI Capability and Provider Contract. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0071-runtime-security-and-resource-isolation.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0071: Runtime Security and Resource Isolation. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0072-ai-desktop-self-management.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0072: AI Desktop Self-Management. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0073-oid-os-management-model.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0073: OID OS-Management Model. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0074-rust-clipper-linux-subsystem.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0074: Rust CA-Clipper Linux Subsystem. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0075-product-brand-and-launch-language.md:1` | No explicit status field; NOT TESTED as a completion claim | WP-0075: Product Brand and Launch Language. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0076-marina-training-lifecycle.md:60` | Proposed. No training endpoints or training backend are implemented by this WP | WP-0076: Marina Training, Fine-Tuning, and Model Lifecycle. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0077-marina-native-transport-and-api.md:47` | In progress. The first native HTTP slice is implemented and verified on macOS | WP-0077: Marina Native Transport, API, and Authorization. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0078-marina-linux-native-qualification.md:47` | **Qualified for the tested native Linux baseline:** Ubuntu 26.04 x86_64 on an | WP-0078: Marina Linux x86_64 Native Qualification. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0079-marina-windows-native-qualification.md:41` | Blocked: no Windows runtime host or runner is currently available. | WP-0079: Marina Windows x86_64 Native Qualification. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0080-marina-macos-arm64-qualification.md:37` | Blocked: no Apple Silicon host is currently available. | WP-0080: Marina macOS arm64 Native Qualification. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0081-marina-remote-transport-qualification.md:34` | Blocked: no authenticated remote deployment environment is currently available. | WP-0081: Marina Remote Transport Qualification. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0082-marina-training-contracts.md:30` | In progress. The first hardware-independent contract slice is implemented in | WP-0082: Marina Training Contracts. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0083-marina-dataset-registration.md:25` | In progress. `runtime/src/training.rs` implements an internal production | WP-0083: Marina Dataset Registration and Validation. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0084-marina-training-job-lifecycle.md:29` | Planned. Lifecycle implementation is separate from inference serving. | WP-0084: Marina Training Job Lifecycle and Isolation. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0085-marina-cpu-training-backend.md:28` | Planned. Do not start until contracts and lifecycle isolation are accepted. | WP-0085: Marina CPU-Compatible Small-Model Training Backend. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0086-marina-training-evaluation.md:26` | Planned. No automatic promotion. | WP-0086: Marina Training Evaluation and Promotion Gates. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0087-marina-artifact-lineage.md:27` | Planned. Artifact management only; no distributed training or pretraining. | WP-0087: Marina Training Artifact and Lineage Management. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0088-oid-bootable-linux-image.md:46` | In Progress. The first image composition, systemd units, service identities, | WP-0088: OID Bootable Linux Image and Boot-to-Operation Proof. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0089-oid-packaged-linux-environment.md:49` | In Progress. Native Linux Docker execution produced `oid-marina_0.1.0_amd64.deb` | WP-0089: OID/Marina Packaged Linux Environment. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0090-marina-windows-support.md:78` | Proposed. No implementation is authorized by this planning record alone. | WP-0090: Marina Windows Support Readiness. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0091-talvik-m2-runtime-diagnostics.md:27` | Implementation slice complete on `feature/talvik-m2-runtime-ide`; deterministic | WP-0091: Talvik M2 Runtime Diagnostics and Residency Edge. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0092-talvik-m2-ide-qualification.md:29` | Configuration and API preflight assets are implemented. Interactive external | WP-0092: Talvik M2 IDE API Compatibility and Qualification. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0093-talvik-m2-retention-policy.md:27` | Implementation slice verified on `feature/talvik-m2-runtime-ide`. | WP-0093: Talvik M2 Retention and Resource Policy. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0094-talvik-m2-concurrency-qualification.md:26` | Deferred pending measurements; one active generation remains the M2 baseline. | WP-0094: Talvik M2 Concurrent Inference Qualification. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0095-talvik-m2-build-identity.md:3` | Implemented; deterministic and binary smoke verification passed. | WP-0095: Talvik M2 Build and Runtime Identity. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0098-talvik-native-inference-performance.md:91` | In progress. Stage A is implemented and deterministic tests pass. Native Qwen | WP-0098: Talvik Native Inference Performance and Worker Isolation. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0101-talvik-m3-multimodal-capability.md:3` | In progress. Local image understanding is implemented through the pinned | WP-0101: Talvik M3 Multimodal Capability. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0104-execution-capability-contract.md:3` | Proposed. This package defines architecture and contracts only; it does not | WP-0104: User/System Execution Capability Contract. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0105-inference-performance-measurement.md:3` | Implementation complete; real-GGUF release qualification remains pending. | WP-0105: Inference Performance Measurement. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0106-talvik-vulkan-acceleration.md:3` | Source implementation complete; all CPU and GPU tests and hardware | WP-0106: Talvik Optional Vulkan Acceleration. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0107-talvik-per-user-msi-installer.md:3` | Proposed. The current Windows release is a ZIP containing the Talvik binaries, | WP-0107: Talvik Per-User Windows MSI. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0108-openai-inference-api-compatibility.md:3` | In progress. ADR-0031 accepts the target profile. Talvik implements a partial | WP-0108: OpenAI Inference API Compatibility. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0109-talvik-ollama-model-runner-api.md:3` | In progress. Phase 1 route semantics are implemented. The official Ollama | WP-0109: Talvik Ollama-Compatible Model Runner API. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0110-talvik-gemma-4-default.md:3` | In progress. The pinned Gemma 4 E4B-it Q4_0 GGUF and Q8_0 vision projector are | WP-0110: Talvik Gemma 4 Default Model. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-0111-talvik-working-runner-stabilization.md:4` | Local stabilization preview; archive qualification is recorded per build. No RC claim. | WP-0111: Working local runner stabilization and RC gates. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-RUNTIME-001-marina-runtime-identity-and-composition-root.md:3` | complete (2026-09-05) | WP-RUNTIME-001 — Marina Runtime Identity and Composition Root. Preserve existing work; no completion inferred. |
| `docs/work-packages/WP-RUNTIME-002-marina-daemon-and-marinactl-client.md:3` | initial local IPC boundary implemented (2026-09-05) | WP-RUNTIME-002 — Talvik Daemon and Client. Preserve existing work; no completion inferred. |

## Build and source surfaces

| Surface | Evidence | Assessment boundary |
| --- | --- | --- |
| `.github/workflows/talvik-api-client-qualification.yml` | `.github/workflows/talvik-api-client-qualification.yml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `.github/workflows/talvik-linux-qualification.yml` | `.github/workflows/talvik-linux-qualification.yml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `.github/workflows/talvik-release.yml` | `.github/workflows/talvik-release.yml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `Cargo.toml` | `Cargo.toml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `console/Cargo.toml` | `console/Cargo.toml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `crates/common/Cargo.toml` | `crates/common/Cargo.toml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `crates/desktop-shell/Cargo.toml` | `crates/desktop-shell/Cargo.toml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `crates/docs/Cargo.toml` | `crates/docs/Cargo.toml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `crates/evidence-engine/Cargo.toml` | `crates/evidence-engine/Cargo.toml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `crates/intent-runtime/Cargo.toml` | `crates/intent-runtime/Cargo.toml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `crates/linux-skills/Cargo.toml` | `crates/linux-skills/Cargo.toml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `crates/model-client/Cargo.toml` | `crates/model-client/Cargo.toml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `crates/model-runner/Cargo.toml` | `crates/model-runner/Cargo.toml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `crates/operation-coordinator/Cargo.toml` | `crates/operation-coordinator/Cargo.toml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `crates/plugin-sdk/Cargo.toml` | `crates/plugin-sdk/Cargo.toml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `crates/policy-engine/Cargo.toml` | `crates/policy-engine/Cargo.toml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `crates/verification-engine/Cargo.toml` | `crates/verification-engine/Cargo.toml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `docker/talvik/Dockerfile` | `docker/talvik/Dockerfile:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `docker/talvik/compose.yaml` | `docker/talvik/compose.yaml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `docker/validation/ubuntu-24.04/Dockerfile` | `docker/validation/ubuntu-24.04/Dockerfile:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `image/manifest.yaml` | `image/manifest.yaml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `packaging/README.md` | `packaging/README.md:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `packaging/marina/config.yaml` | `packaging/marina/config.yaml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `packaging/marina/talvikd.service` | `packaging/marina/talvikd.service:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `packaging/oid/oid-console.service` | `packaging/oid/oid-console.service:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `packaging/oid/oid-readiness` | `packaging/oid/oid-readiness:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `packaging/oid/oid-readiness.service` | `packaging/oid/oid-readiness.service:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `packaging/sysusers/marina.conf` | `packaging/sysusers/marina.conf:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `packaging/sysusers/oid.conf` | `packaging/sysusers/oid.conf:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `packaging/sysusers/talvik.conf` | `packaging/sysusers/talvik.conf:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `packaging/talvik/README-WINDOWS.txt` | `packaging/talvik/README-WINDOWS.txt:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `packaging/talvik/SETUP-WINDOWS.cmd` | `packaging/talvik/SETUP-WINDOWS.cmd:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `packaging/talvik/config.vulkan.yaml` | `packaging/talvik/config.vulkan.yaml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `packaging/talvik/config.yaml` | `packaging/talvik/config.yaml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `packaging/talvik/default-model.yaml` | `packaging/talvik/default-model.yaml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `packaging/talvik/talvikd.plist.in` | `packaging/talvik/talvikd.plist.in:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `packaging/talvik/talvikd.service` | `packaging/talvik/talvikd.service:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `packaging/tmpfiles/marina.conf` | `packaging/tmpfiles/marina.conf:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `packaging/tmpfiles/oid.conf` | `packaging/tmpfiles/oid.conf:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `packaging/tmpfiles/talvik.conf` | `packaging/tmpfiles/talvik.conf:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `runtime/Cargo.toml` | `runtime/Cargo.toml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `runtime/adapters/llama-cpp/Cargo.toml` | `runtime/adapters/llama-cpp/Cargo.toml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `runtime/adapters/llama-cpp/sys/Cargo.toml` | `runtime/adapters/llama-cpp/sys/Cargo.toml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
| `shared/Cargo.toml` | `shared/Cargo.toml:1` | VERIFIED file present; read alongside assessment. No release/runtime qualification implied. |
