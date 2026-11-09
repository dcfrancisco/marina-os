# MarinaOS Linux foundation verification — 2026-11-09

Review host: Ubuntu 26.04 LTS x86_64; rustc 1.96.0 (ac68faa20 2026-05-25),
cargo 1.96.0 (30a34c682 2026-05-25). Target remains Ubuntu 24.04 LTS amd64.
Initial branch `main`, commit `c6edcdba71e67b21079f31d946cae8c190a79798`, clean tree.
Checks below used the existing code; changes in this review are documentation only.

| Exact command | Result | Evidence |
| --- | --- | --- |
| `git status --short` / `git branch --show-current` / `git rev-parse HEAD` | Clean / main / commit above | Review snapshot; tracked listing linked below |
| `cargo fmt --all -- --check` | PASS, exit 0 | [fmt.txt](evidence/marinaos-foundation-2026-11-09/fmt.txt), empty successful output |
| `cargo test --workspace` | FAIL, exit 127: console test binary could not locate libllama.so.0 | [test.txt](evidence/marinaos-foundation-2026-11-09/test.txt) |
| `LD_LIBRARY_PATH=/home/danny/.marina/lib cargo test --workspace` (sandbox) | FAIL, exit 101: two local socket fixtures denied with Operation not permitted; 41 console unit tests passed | [test-with-libs.txt](evidence/marinaos-foundation-2026-11-09/test-with-libs.txt) |
| `LD_LIBRARY_PATH=/home/danny/.marina/lib cargo test --workspace` (approved outside sandbox) | PASS, exit 0 | [test-unrestricted.txt](evidence/marinaos-foundation-2026-11-09/test-unrestricted.txt) |
| `cargo test -p oid-common -p oid-intent-runtime -p oid-policy-engine -p oid-operation-coordinator -p oid-linux-skills -p oid-evidence-engine -p oid-verification-engine -p oid-desktop-shell` | PASS, exit 0 | [foundation-tests.txt](evidence/marinaos-foundation-2026-11-09/foundation-tests.txt) |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS, exit 0; no lint allowance | [clippy.txt](evidence/marinaos-foundation-2026-11-09/clippy.txt) |
| `sh -n` or `bash -n` for every scripts/*.sh and packaging/oid/oid-readiness, selected by shebang | PASS, all exit 0; every exact file command recorded | [syntax.txt](evidence/marinaos-foundation-2026-11-09/syntax.txt) |
| Python `ast.parse(path.read_text(), filename=str(path))` for every scripts/*.py | PASS; syntax only, no Python script executed | [syntax.txt](evidence/marinaos-foundation-2026-11-09/syntax.txt) |
| `git diff --check` | PASS, exit 0 | Whitespace validation of documentation changes |

Test totals (sum of reported test suites, including doc tests): 38 passed, 0 failed, 0 ignored
foundation tests; 296 passed, 0 failed, 10 ignored workspace tests. Passing tests do not prove live
GGUF inference, package lifecycle or a bootable OS. Native libraries were reused
from the existing user installation; none was installed or changed.

`git ls-tree -r --name-only HEAD` produced the complete
[tracked-files.txt](evidence/marinaos-foundation-2026-11-09/tracked-files.txt).
The inventory lists all ADR/WP documents, including source status and line anchors.
No ADR-0053–0062 or WP-0112–0124 exists in that tree. Local remote-tracking branch
names were listed with `git branch -a`; no remote fetch or assertion about unseen
remote proposals was made.

NOT TESTED: Ubuntu 24.04 container run, image construction, VM installation/boot,
UEFI/BIOS boot, Secure Boot, offline/reboot behavior, hardware matrix, package
install/upgrade/removal, signing and recovery. CI YAML was inspected; workflow
presence is not an executed CI result. No root disk/image builder, installer,
mount, format, partition or host-service command was executed. Shell parsing does
not validate destructive actions or repair the static defects in the assessment.
