# Talvik MoE engineering baseline — 2026-11-10

This is a work-in-progress assessment, not an acceptance report. The Talvik
runtime shares this repository with other products; this work is limited to
Talvik and its existing clients and backend.

## Existing capabilities and gaps

- `oid-console` builds `talvik` and `talvikd`; `oid-runtime` owns the service,
  registry, admission and hardware discovery, and `oid-llama-cpp-sys` links an
  externally installed native library. Linux setup defaults to llama.cpp
  b12060, with native, AVX, AVX2, AVX-512, FMA, F16C, SSE4.2 and BMI2
  disabled. The installed library's source revision was not independently
  verified. No dependency was upgraded.
- Discovery records file size and filename, not parsed GGUF metadata. The
  loaded profile queries the native backend. Expert counts are now exposed in
  its existing HTTP profile; absent or inconsistent counts are unknown.
  Active and total parameters, shared experts, quantization and unloaded-model
  diagnostics still need authoritative implementation. An absent expert count
  cannot by itself establish that a model is dense.
- Placement already supports CPU, automatic, preferred/required GPU and a
  layer limit through the existing YAML/environment configuration. Expert or
  tensor placement has not been qualified against the pinned API. GPU
  execution has not been tested here.
- Existing admission considers the full registered model size, available RAM,
  a load-peak factor, a safety margin, and a fixed 16 KiB per context token.
  This last value is a heuristic, not a measured allocation or a guaranteed
  upper bound. Native loading is checked again and unloaded on rejection.
  Actual KV allocation, architecture-aware estimates and cache quantization
  controls remain unimplemented. The backend already retains contexts and
  performs prefix reuse; its isolation guarantees need separate review. It
  invalidates state after cancellation and failed decode, with fixture-based
  and some model-dependent tests.
- Hardware discovery already reports CPU, cores, RAM, SIMD, process memory and
  initialized backend devices. It does not measure bandwidth or fully evaluate
  model suitability. Generation statistics already distinguish prompt and
  generation timing. A reproducible bounded JSON benchmark with full identity,
  peak memory and cold/warm metadata remains to be implemented.
- Existing Unix IPC and HTTP have their own established contracts. HTTP
  enforces bearer authentication when configured, read scopes on model
  inspection, request body limits and context limits. The profile extension
  adds fields only; no transport, configuration or package format is added.
  Full client, packaging and upgrade qualification remains outstanding.

## Incremental work remaining

1. Extend existing discovery with bounded, validated GGUF metadata inspection;
   establish tensor-based total and active counts only where architecture
   semantics justify them, with fixtures for missing and malformed fields.
2. Inspect the pinned native headers and isolate supported placement/cache
   controls; reject unavailable controls and test CPU and any real GPU builds.
3. Replace or supplement the admission heuristic with architecture-aware
   accounting, retaining explicit unknowns for unsupported layouts and
   measuring actual allocations only when the backend exposes them. Review
   prefix reuse isolation, error recovery and eviction before changing it.
4. Extend the existing hardware and metrics interfaces for evidence-based
   suitability and bounded machine-readable benchmarks; distinguish model
   loadability from speed and measurement from estimates.
5. Qualify authenticated IPC compatibility, cancellation, configuration,
   packaging and upgrades with native dependencies and appropriate fixtures.

## Reproducible independent development

Clone the repository, create a distinct feature branch from the agreed
integration revision, and use a separate checkout or Git worktree and build
directory for each concurrent session. Rust 1.88 is the workspace minimum;
the platform's native llama.cpp libraries must match the setup instructions.
Run narrow tests first, then `cargo fmt --all -- --check`,
`cargo test --workspace`, and workspace Clippy when resources permit. Native
tests need the matching library directory in the platform loader path; tests
marked ignored require explicit local fixtures. Never commit model files,
credentials or machine-local paths. Keep benchmark artifacts outside the
checkout, identify the actual machine and model checksum in each result, and
submit small branches for Git review. Resolve overlapping changes through
review and integration, not shared mutable build outputs.

## Local qualification snapshot

On 2026-11-10 the host reported AMD Phenom II X6 1055T, six physical and
logical cores, x86_64, SSE2 and no AVX/AVX2/FMA in `lscpu`; Ubuntu 26.04 LTS,
kernel 7.0.0-42-generic, Rust/Cargo 1.96.0 and GCC 15.2.0. `free -h` showed
15 GiB total and 14 GiB available; the checkout filesystem showed 49 GiB
available. These are point-in-time readings. Memory channel configuration,
bandwidth and accelerator support were not established. The installed CPU
library and a local Qwen2.5 0.5B GGUF were used for a metadata/tokenizer test;
no inference throughput or other model qualification is claimed.
