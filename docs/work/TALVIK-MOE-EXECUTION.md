# Talvik MoE execution tracker

Objective: implement and qualify hardware-aware Talvik inference while preserving
existing contracts and the Phenom CPU baseline. Branch:
`feature/talvik-moe-aware-inference`; integration baseline `39a809e`.

| ID | Status | Dependencies and acceptance criteria | Evidence / blocker |
| --- | --- | --- | --- |
| WP-01 | IN_PROGRESS | Bounded discovery of architecture, context, experts and quantization; malformed fixtures | `8d34aa6` supplies architecture and context; `02af2fb` hardens malformed metadata. Tensor descriptors inspected on one local Gemma artifact; data extents and broader artifact coverage remain unverified. |
| WP-02 | IN_PROGRESS | Reliable dense/MoE/unknown classification and Gemma tensor/backend investigation | `479d05a` reports loaded expert metadata conservatively. Gemma 4 dense header classification verified against the matching installed CPU build; real MoE inference remains unqualified. |
| WP-03 | IN_PROGRESS | WP-01; parameter and memory accounting with explicit unknowns | Checked on-disk tensor accounting and known-type extents implemented; unsupported types remain unknown, runtime estimates and broader type coverage remain. |
| WP-04 | IN_PROGRESS | Inspect pinned backend and qualify supported placement | Pinned source and adapter placement inspected; installed CPU-only build verified. GPU placement and real MoE execution remain unqualified. |
| WP-05 | PENDING | WP-01/03; architecture-aware KV estimate and lifecycle tests | Current fixed per-token allowance is a heuristic. |
| WP-06 | PENDING | WP-01/03/04; hardware-aware suitability | Existing hardware detection is partial. |
| WP-07 | PENDING | WP-03/04; bounded reproducible benchmark | No Gemma inference measurement yet. |
| WP-08 | PENDING | WP-01/03/06; compatible IPC diagnostics and security | Existing authenticated profile extended in `479d05a`. |
| WP-09 | PENDING | Relevant work packages; regression and packaging evidence | Runtime 65, CLI 18, daemon 53 and native 21 unit tests pass; default-policy native acceptance passes in debug and release. Workspace qualification outstanding. |
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

NEXT ACTION: Build repository talvik/talvikd with required native linkage and
run the isolated default-model acceptance --report against Qwen after the admission
integration. Add a native cancellation/prefix-reuse acceptance run with the local
Qwen fixture. Then finish authenticated suitability diagnostics tests, packaging
checks and cold-context benchmarking. Preserve installed release and configuration.

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

2026-11-10 implementation and Phenom default-policy slice:

- Preserved the initial uncommitted GGUF/model diff in
  `/tmp/talvik-classification-start.patch` before editing. Completed
  `inspect_artifact_expert_evidence`: architecture-scoped metadata, complete
  Gemma 4 dense/shared FFN shapes, matching router/expert shapes and counts,
  unknown on incomplete/conflicting/malformed evidence. Other architectures
  retain conservative metadata-only classification. Inferred routing counts
  stay unset. No registry schema, placement or serving-capability changes.
- Native provenance: installed `libllama.so.0.8.0` and cached build output at
  `/home/danny/.cache/llama.cpp-talvik-build/bin/` have identical SHA-256
  `1f940cf8a33991bb34097adbad6c5cb7b1622ae66c1f6601678f31e839d273a5`.
  All three installed GGML libraries also match that build byte-for-byte.
  Its CMake cache identifies `/home/danny/.cache/llama.cpp-talvik` as source;
  that clean checkout is `e4b6c8ca6a6e8da70248da30709e897e7cc8615e`.
  Generated `common/build-info.cpp` records `e4b6c8c`, GNU 15.2.0, Linux x86_64.
  Build flags: CPU ON, NATIVE/CUDA/VULKAN OFF. This is local build provenance,
  not a reproducible-build attestation. The separate `18443257` checkout
  discussed above is not the installed build's source. No library was changed.
- The matching source's `src/models/gemma4.cpp` selects MoE layers through
  `ffn_gate_inp`, with dense FFNs on every layer. Header-only inspection of
  local Gemma (666 tensors) now reports dense with unknown routing counts.
  Gemma was never loaded or used for inference in this slice.
- Qwen generation uncovered an ABI omission: `ContextParams` lacked
  `size_t moe_cache_size` before `abort_callback`. The shifted callback was
  interpreted as a GPU MoE cache request. Added the missing field; a C probe
  compiled against the matching headers reports size 168, cache offset 112,
  abort 120, embeddings 136, samplers 144, parent context 160 on this amd64
  installation. The native regression test checks these offsets and returned
  defaults. Initial context failures at 256 and 512 tokens preceded this fix;
  Qwen generation succeeds with the corrected binding.
- Existing policy audit: built-in default is Gemma, but the Phenom YAML and
  user service both explicitly disabled it (`default_model: none` and
  `TALVIK_DEFAULT_MODEL=none`). Discovery ordering did not set the daemon's
  startup default. `talvik run --default` resolves the same configuration;
  explicit CLI/API IDs bypass it. The optional chat/benchmark client retains
  its documented resident/default/smallest policy and requires an explicit
  model for separate MoE qualification. Existing qualification workflows
  already select Qwen or require a caller-supplied model; none were run with
  Gemma. Missing/load failures warn and keep the daemon running without a
  replacement model. CLI generation failures now include the requested ID.
- New `scripts/test-talvik-default-model.py` uses disposable state, the caller's
  existing small GGUF and a synthetic invalid large-model entry. Covers YAML
  selection, persistent-registry process restart, environment overrides,
  disabled bootstrap, explicit CLI/API requests, successful `run --default`,
  missing/incompatible defaults and no large-model substitution. No downloads,
  conversions, installed-model writes or real Gemma loads occur.
- Validation: `cargo fmt --all -- --check`; with
  `LD_LIBRARY_PATH=/home/danny/.marina/lib`, `cargo test -j 2 -p
  oid-llama-cpp-sys --lib` (21 passed, 8 opt-in ignored), `cargo test -j 2 -p
  oid-console --bin talvik --bin talvikd` (18 + 53 passed), `cargo test -j 2 -p
  oid-runtime --lib` (65 passed, 1 opt-in ignored), and `cargo clippy -j 2 -p
  oid-runtime -p oid-llama-cpp-sys -p oid-console --lib --bins --tests -- -D
  warnings` passed. Socket suites required execution outside the sandbox;
  sandbox failures were permission errors. The opt-in Gemma header inspection
  passed using `TALVIK_GGUF_INSPECT_PATH` and the exact ignored test command
  above. No workspace suite or real MoE inference benchmark was run.
- Default-policy acceptance command (debug):
  `LD_LIBRARY_PATH=/home/danny/.marina/lib python3
  scripts/test-talvik-default-model.py --model
  /home/danny/.marina/models/qwen2.5-0.5b-instruct-q4_k_m.gguf` passed.
  Qwen is 491,400,032 bytes, SHA-256
  `74a4da8c9fdbcd15bd1f6d01d621410d31c6fc00986f5eb687824e7b93d7a9db`.
  Initial release acceptance rejected a build without native linkage; release
  rebuilding explicitly requires `LLAMA_CPP_LIB_DIR=/home/danny/.marina/lib`,
  `LLAMA_CPP_REQUIRED=1`, `LLAMA_CPP_MTMD_REQUIRED=1`, and
  `LLAMA_CPP_CHAT_TEMPLATE_REQUIRED=1`. No changes to stopped MarinaOS work.

- Final release acceptance used the same Python command with
  `--bin-dir target/release` and passed after native linkage was required.
  Installed the validated release `talvik`/`talvikd` without replacing any
  native library. The Phenom's `/home/danny/.marina/config.yaml` now sets
  `default_model: qwen2.5-0.5b-instruct-q4_k_m`, `auto_download: false`,
  `auto_load: true`. Removed only `Environment=TALVIK_DEFAULT_MODEL=none`
  from `/home/danny/.config/systemd/user/talvikd.service`; reloaded the user
  manager and restarted the service. Prior binaries/config/unit are backed
  up under `/tmp/talvik*-installed.before`, `/tmp/talvik-config.before.yaml`
  and `/tmp/talvikd-service.before`. Installation settings are not global
  repository defaults.
- Live verification (`/tmp/talvik-phenom-default-verification.json`): release
  daemon active, backend `llama.cpp`, sole resident
  `qwen2.5-0.5b-instruct-q4_k_m`; process environment confirms YAML source and
  no default-model environment override. `/readyz` reports inference ready,
  and installed `talvik --plain run --default 'Reply only OK.'` succeeds.
  Native library SHA-256 is unchanged. No installed model was removed,
  renamed, converted or downloaded. WP-02 remains in progress for broader
  architecture coverage and actual MoE qualification; next work is WP-03 as
  specified above.


2026-11-10 WP-03 storage slice and preserved Phenom baseline:

- Source baseline `343a4e5`; installed `talvik` and `talvikd` each report
  0.1.0, git `a74f92d351d457722d379bd12d11141880f2d330`, dirty=true,
  x86_64 Linux release. SHA-256 respectively
  `037dda6156b0d4ec9429c38f99f8481f5e28c8e89cf57a1a1927daa025405701` and
  `ca3c08c92be623f76100c76e2e9ff8fe3ef84051a997929c3b5981982ea37fcf`.
  These build identities do not independently prove source commit 343a4e5.
- Installed native library provenance and CPU flags are recorded above.
  SHA-256: libllama `1f940cf8a33991bb34097adbad6c5cb7b1622ae66c1f6601678f31e839d273a5`,
  libggml-base `4dea1ba5429f2098362c0042df62e5e12f27c378d8ba375c69630e52ba6018fc`,
  libggml-cpu `bd6862f8daea6d845d64ad7ffadcb0c245366c9845167acadbb54a329783c45e`,
  libggml `9e108c5567a12923887c576ac1f7e950a190959b49b80afc8e7bcd998c2e9d09`,
  libmtmd `eb693328c20be94ed3773621b679c685c00a16f7ca3d6345672d4bd2034efce6`,
  liboid-chat-template `5a74b7000b35b87c862ced43a9d330971c898e8ed272548850933ad2f39add55`.
  Qwen SHA-256 `74a4da8c9fdbcd15bd1f6d01d621410d31c6fc00986f5eb687824e7b93d7a9db`.
- Effective YAML: name `Talvik`; model directory `/home/danny/.marina/models`;
  registry `/home/danny/.marina/models.registry`; state `/home/danny/.marina`;
  default `qwen2.5-0.5b-instruct-q4_k_m`; auto_download false; auto_load true.
  The user unit points to this YAML and does not set a default-model override.
  `timeout 45s /home/danny/.marina/bin/talvik --plain run --default 'Reply only OK.'`
  returned `OK.` with exit 0 against the existing daemon. No installed files
  were rebuilt or changed. This smoke test required access outside the sandbox.
- Added a separate bounded storage inspector with checked element and byte sums,
  aligned section and offsets, file bounds and known extent overlap validation.
  Block sizes follow the pinned local ggml definitions; unsupported types retain
  unknown total bytes and cannot have complete extent/overlap validation.
  Counts include all experts, not just active parameters; disk bytes are not
  runtime memory or proof of backend compatibility. Real MoE inference remains
  unverified. The opt-in Qwen inspection found 291 tensors, 630167424 elements,
  485452288 tensor bytes, offset 5947744, file size 491400032.
- Focused GGUF tests, Qwen opt-in inspection, formatting and strict runtime
  test Clippy passed. Runtime library suite passed 66 tests, one ignored, with
  loopback access outside the sandbox. The sandbox run failed its HTTP bind and
  once hit an unrelated timing-sensitive generation assertion; the unrestricted
  rerun passed both. No workspace or actual MoE inference test was run.


2026-11-10 WP-04 independent backend inspection:

- Pinned source `e4b6c8ca` `include/llama.h` defines layer offload,
  device selection and an experimental expert device cache; its model loader
  assigns input tensors to CPU and distributes requested repeating/output
  layers to selected devices. This source inspection does not qualify an
  accelerator or establish that all operations run there.
- Talvik selects one enumerated native device, passes the requested GPU layer
  count and keeps actual offloaded-layer count unknown for GPU requests. The
  default `moe_cache_size` is zero; Talvik does not expose or qualify that
  experimental cache. Installed CMake cache confirms CPU ON, CUDA/Vulkan OFF,
  native CPU tuning OFF. No installed library or service was changed.
- The native `accelerator_used` diagnostic formerly inferred use from a GPU
  request. It now reports `None` for GPU requests and `Some(false)` for CPU
  placement; a targeted unit test covers both. The adapter does not consume it
  in public `ModelPlacement`. Next: qualify actual placement on a suitable
  accelerator build and separately test a real MoE artifact. Native unit tests
  passed 22 with 8 opt-in tests ignored.


2026-11-10 continued WP-03/05/08 implementation:

- Added every active GGML type in pinned e4b6c8ca ggml.h, with sizes from
  ggml-common.h and scalar definitions; checked all 36 active type block/byte
  pairs directly against installed libggml-base ggml_blck_size/ggml_type_size.
  Removed and future identifiers remain explicitly unsupported.
- Added optional storage_accounting to authenticated /v1/models and /api/show.
  Existing fields retain their meanings. Payload, file bytes, total parameters,
  and data offset are separate; active parameters, resident weight estimates,
  actual KV allocation and observed runtime memory remain null. Inspection
  failures return unavailable without exposing paths or OS error details.
- Added F16 full-attention KV payload estimate per token for qwen2/llama,
  using layer count, embedding/head dimensions and KV head count with checked
  arithmetic. Other architectures remain unknown. This excludes padding and
  scratch and does not yet replace admission's fixed heuristic.
- Gemma header-only storage test PASS: 666 tensors, 7463013674 elements,
  4574983336 payload bytes, data offset 15822880, file size 4590807392;
  dense evidence. No weights loaded. Focused parser suite PASS (10 tests,
  one opt-in ignored); HTTP storage diagnostic fixture PASS. Strict runtime
  and console Clippy PASS before the final KV fixture addition.
- Baseline installed Qwen smoke PASS again (OK., exit 0); installed release,
  native libraries, YAML, service and model artifacts untouched.
- Work remains IN_PROGRESS; no overall qualification or MoE inference claim.

- Combined regression PASS: `LD_LIBRARY_PATH=/home/danny/.marina/lib cargo test
  -j 2 -p oid-runtime -p oid-console -p oid-llama-cpp-sys --lib --bin talvik
  --bin talvikd` outside sandbox for loopback fixtures. Runtime 68 passed,
  native 22 passed (8 opt-in ignored), CLI 18 passed, daemon 54 passed.
  Formatting and strict Clippy with the same packages and --lib --bins --tests
  PASS after all changes. No workspace-wide or new release installation claim.


Continuation evidence after 80bdf8b:

- Repository binaries built with `LLAMA_CPP_LIB_DIR=/home/danny/.marina/lib`,
  `LLAMA_CPP_REQUIRED=1`, `LLAMA_CPP_MTMD_REQUIRED=1`,
  `LLAMA_CPP_CHAT_TEMPLATE_REQUIRED=1`, `cargo build -j 2 -p oid-console
  --bin talvik --bin talvikd`: PASS. No installation performed.
- Existing isolated default acceptance PASS against these debug binaries:
  `LD_LIBRARY_PATH=/home/danny/.marina/lib python3
  scripts/test-talvik-default-model.py --model
  /home/danny/.marina/models/qwen2.5-0.5b-instruct-q4_k_m.gguf`.
- WP-07 extends that existing harness with optional --report, two eight-token
  requests at 512 context/two CPU threads, retaining returned metrics and usage.
  Cold means new prompt with resident weights; load duration and peak memory
  remain explicitly unavailable. This is not a cold model-load benchmark.
- WP-10 workflow: use a distinct feature branch and git worktree per worker,
  set a distinct CARGO_TARGET_DIR outside the shared build tree, keep native
  dependencies pinned, commit only the assigned write set and review each
  commit before integration. Run narrow evidence commands before integration;
  never run interactive and unattended workers in this checkout concurrently.
- Exact hardware/model blockers: installed backend is CPU-only, so actual GPU
  execution/offloaded layer counts require another suitable accelerator build
  and device. Available local Gemma evidence is dense; no confirmed local MoE
  fixture is available. Actual MoE load/generation remains BLOCKED pending an
  explicitly supplied compatible local MoE artifact. No download attempted.

- Bounded Qwen report PASS at /tmp/talvik-moe-qwen-benchmark.json using debug
  80bdf8b native binaries. Both requests generated 8 tokens from 35 prompt
  tokens. New-prompt phase: TTFT 3350 ms, generation 3.13475 tokens/s, 24
  reused tokens. Warm phase: TTFT 364 ms, generation 3.40315 tokens/s, 34
  reused tokens. The first phase reused template prefix, so it is NOT a cold
  cache measurement; the harness labels it new_prompt going forward. Cold load,
  peak memory, and model load duration remain NOT MEASURED. No MoE inference.
- Additional malformed alignment fixture PASS (zero, non-power-of-two,
  wrong metadata type). GGUF filter now 11 passed, one ignored.

- Follow-up KV validation rejects explicitly zero key dimensions and layer
  counts instead of reporting a zero/fallback estimate. Focused KV test and
  strict runtime test Clippy PASS. Admission integration, suitability and
  remaining qualification remain actionable and are not declared complete.


Continuation after 2afb643 — architecture-aware admission and suitability:

- Load preflight, post-native-load admission, and generation admission now use
  bounded GGUF Qwen2/Llama F16 KV metadata. The reservation is max(estimated
  KV bytes/token, previous 16 KiB/token allowance) times requested context.
  This preserves the scratch/padding floor and unknown-layout fallback while
  rejecting larger known requirements and arithmetic overflow. No native context
  lifecycle or installed configuration changed. Metadata is re-read before
  admission, avoiding stale estimates across registry/artifact changes.
- New admission tests cover unknown/smaller/larger rates, overflow and exact
  boundaries. Runtime suite PASS 70 tests (one ignored) including cancellation,
  eviction/reload, unload, queue recovery and rejected-load preservation.
- Added conservative hardware suitability function and fixtures for supported,
  unsupported, unknown, constrained and overflowing requirements. Optional HTTP
  suitability reports a payload-based estimate at 4096 context plus KV floor
  and 512 MiB safety margin. Backend compatibility remains unknown on artifact
  inspection; passing the resource estimate does not assert loadability or speed.
  Repacking and retained native contexts remain outside this estimate.
- Strict runtime/console Clippy and formatting PASS. Installed Qwen smoke PASS
  (OK., exit 0) without restart or installation. No MarinaOS image changes.
