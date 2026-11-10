# Talvik MoE execution tracker

Objective: implement and qualify hardware-aware Talvik inference while preserving
existing contracts and the Phenom CPU baseline. Branch:
`feature/talvik-moe-aware-inference`; integration baseline `39a809e`.

| ID | Status | Dependencies and acceptance criteria | Evidence / blocker |
| --- | --- | --- | --- |
| WP-01 | IN_PROGRESS | Bounded discovery of architecture, context, experts and quantization; malformed fixtures | `8d34aa6` supplies architecture and context only. |
| WP-02 | IN_PROGRESS | Reliable dense/MoE/unknown classification and Gemma tensor/backend investigation | `479d05a` reports loaded expert metadata conservatively. Gemma classification unverified. |
| WP-03 | PENDING | WP-01; parameter and memory accounting with explicit unknowns | No validated tensor accounting yet. |
| WP-04 | PENDING | Inspect pinned backend and qualify supported placement | Existing CPU and GPU-layer controls need qualification. |
| WP-05 | PENDING | WP-01/03; architecture-aware KV estimate and lifecycle tests | Current fixed per-token allowance is a heuristic. |
| WP-06 | PENDING | WP-01/03/04; hardware-aware suitability | Existing hardware detection is partial. |
| WP-07 | PENDING | WP-03/04; bounded reproducible benchmark | No Gemma inference measurement yet. |
| WP-08 | PENDING | WP-01/03/06; compatible IPC diagnostics and security | Existing authenticated profile extended in `479d05a`. |
| WP-09 | PENDING | Relevant work packages; regression and packaging evidence | Prior narrow tests in baseline; workspace qualification outstanding. |
| WP-10 | IN_PROGRESS | Safe independent work and reproducible evidence workflow | Baseline documents Git workflow; unattended invocation below. |

NEXT ACTION: Complete bounded GGUF metadata and tensor inspection, expose expert
facts without changing the registry format, and qualify the installed native
library's exact revision against the source before relying on its behavior.

The local Gemma artifact has 49 metadata entries, 666 tensors, architecture
`gemma4`, and no expert-count metadata or tensor names containing `expert`,
`exps`, `router`, or `ffn_gate_inp`. A separate local llama.cpp source checkout
at `18443257a30c884d5332abb8e7dc43c7ffe42fda` uses the optional
`blk.%d.ffn_gate_inp.weight` tensor to select Gemma 4 MoE layers, and otherwise
uses dense FFNs. This supports classifying this artifact as dense for that
source implementation; the installed library revision has not been verified
against that checkout. No MoE inference benchmark has been run.

Codex CLI 0.167.0 provides non-interactive `codex exec` and `codex exec resume`
according to local help. A reviewable invocation is `codex exec -C
~/codes/marina-os -s workspace-write 'Read docs/work/TALVIK-MOE-EXECUTION.md and
continue its next action within the existing permissions.'` A later session can
use `codex exec resume <session-id>` after inspecting `codex exec resume --help`.
The command does not remove approval boundaries and has not been launched as a
persistent process. Host session limits still apply.
