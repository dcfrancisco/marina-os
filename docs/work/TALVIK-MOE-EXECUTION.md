# Talvik MoE execution tracker

Objective: implement and qualify hardware-aware Talvik inference while preserving
existing contracts and the Phenom CPU baseline. Branch:
`feature/talvik-moe-aware-inference`; integration baseline `39a809e`.

| ID | Status | Dependencies and acceptance criteria | Evidence / blocker |
| --- | --- | --- | --- |
| WP-01 | IN_PROGRESS | Bounded discovery of architecture, context, experts and quantization; malformed fixtures | `8d34aa6` supplies architecture and context; `02af2fb` hardens malformed metadata. Tensor descriptors inspected on one local Gemma artifact; data extents and broader artifact coverage remain unverified. |
| WP-02 | IN_PROGRESS | Reliable dense/MoE/unknown classification and Gemma tensor/backend investigation | `479d05a` reports loaded expert metadata conservatively. Gemma classification unverified. |
| WP-03 | PENDING | WP-01; parameter and memory accounting with explicit unknowns | No validated tensor accounting yet. |
| WP-04 | PENDING | Inspect pinned backend and qualify supported placement | Existing CPU and GPU-layer controls need qualification. |
| WP-05 | PENDING | WP-01/03; architecture-aware KV estimate and lifecycle tests | Current fixed per-token allowance is a heuristic. |
| WP-06 | PENDING | WP-01/03/04; hardware-aware suitability | Existing hardware detection is partial. |
| WP-07 | PENDING | WP-03/04; bounded reproducible benchmark | No Gemma inference measurement yet. |
| WP-08 | PENDING | WP-01/03/06; compatible IPC diagnostics and security | Existing authenticated profile extended in `479d05a`. |
| WP-09 | PENDING | Relevant work packages; regression and packaging evidence | Prior narrow tests in baseline; workspace qualification outstanding. |
| WP-10 | IN_PROGRESS | Safe independent work and reproducible evidence workflow | Baseline documents Git workflow; unattended invocation below. |

Runtime library tests: 60 passed with loopback access; the sandboxed run had
59 passed and one failed because its HTTP fixture could not bind. GGUF-focused
tests passed (5); `cargo clippy -p oid-runtime --lib -- -D warnings` passed
with the local native library on the loader path.

Latest slice: `models::inspect_artifact_experts` reads declared expert counts
without loading a model or changing the registry format. Wrong metadata types,
missing counts and inconsistent values remain unknown; this does not inspect
tensors or establish backend support. `LD_LIBRARY_PATH=/home/danny/.marina/lib
cargo test -j 2 -p oid-runtime --lib gguf::tests` passed 4 tests; `cargo fmt
--all -- --check` and `LD_LIBRARY_PATH=/home/danny/.marina/lib cargo clippy -j 2
-p oid-runtime --lib -- -D warnings` passed. The initial test run without the
loader path failed to start because `libllama.so.0` was not found. No workspace
suite or real-model inference was run for this slice.

Latest tensor slice: `models::inspect_artifact_tensors` returns names, shapes,
raw type identifiers and relative offsets without loading tensor data. It limits
the combined metadata and tensor-header read to 32 MiB and tensor count to
100,000; it rejects duplicate or empty names, invalid dimension counts, zero or
overflowing shapes and truncation. It does not validate tensor types, offsets,
data extents or backend compatibility. `LD_LIBRARY_PATH=/home/danny/.marina/lib
cargo test -j 2 -p oid-runtime --lib gguf::tests` passed 5 tests; `cargo fmt
--all -- --check` and `LD_LIBRARY_PATH=/home/danny/.marina/lib cargo clippy -j 2
-p oid-runtime --lib -- -D warnings` passed. No real artifact, workspace suite,
or inference was exercised in this slice.

Latest qualification: the opt-in `gguf::tests::inspect_local_artifact_descriptors`
test read `/home/danny/.marina/models/gemma-4-E4B-it-Q4_0.gguf` (4,590,807,392
bytes) with `TALVIK_GGUF_INSPECT_PATH` set to that path and
`LD_LIBRARY_PATH=/home/danny/.marina/lib cargo test -j 2 -p oid-runtime --lib
gguf::tests::inspect_local_artifact_descriptors -- --ignored --exact --nocapture`.
It passed, reporting architecture `gemma4`, `Q4_0`, unknown experts (count not
declared), 666 tensor descriptors, and no names containing `expert`, `exps`,
`router`, or `ffn_gate_inp`. This checks header parsing, not tensor data or
inference; the artifact was identified by path and size, not a fresh digest.
The regular GGUF test filter passed 5 tests with the opt-in test ignored.
`cargo fmt --all -- --check` passed. Strict Clippy with `--tests` initially
failed on a pre-existing fixture's potentially truncating cast; after replacing
it with a checked conversion, `LD_LIBRARY_PATH=/home/danny/.marina/lib cargo
clippy -j 2 -p oid-runtime --tests -- -D warnings` passed.

Installed `/home/danny/.marina/lib/libllama.so.0.8.0` has SHA-256
`1f940cf8a33991bb34097adbad6c5cb7b1622ae66c1f6601678f31e839d273a5` and
ELF build ID `e041e1153d9003a659c3db2e62a38ddbb80d7832`. Neither establishes
its source revision. The local source checkout is at `18443257a30c884d5332abb8e7dc43c7ffe42fda`;
the Linux setup script defaults to `b12060`, which is not resolvable in that
checkout. No matching installed-library provenance was found in this slice.

NEXT ACTION: Investigate local build provenance for the installed native
library, then design conservative tensor-based classification with fixtures
that preserve unknown when source/backend behavior cannot be established.

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
