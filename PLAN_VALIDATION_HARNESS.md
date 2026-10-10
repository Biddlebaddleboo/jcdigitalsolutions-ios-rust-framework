# PLAN_VALIDATION_HARNESS.md — Workstream G3: Parity and Benchmark Harnesses

## Objective

## Boundary with completed shared tooling

G3 concerns **semantic differential parity and benchmarking**, not creation or repair of the already installed R1/R2 build/validation engines. `ios-rust-validate` validates only profiles registered in `tools/validation/specs/validation-v1.json`; it does not supply Apple reference implementations, candidate workloads, performance baselines, or real-device execution. Keep existing parity/benchmark harness source and independently required ABI checks. Add focused declarative validation or Python assertions only when they match the engine schema, without moving benchmark execution into a text-only guard. Preserve the no-suite `cargo xtask parity` behavior until a real reference/candidate suite exists. Consult `docs/SHARED_TOOLING.md` for tool invocation.


Provide reusable, dependency-free harnesses for differential correctness and performance evidence. A harness is not evidence by itself: do not mark parity or performance as passed until a real Apple reference adapter and candidate run on the stated target.

## Dependencies

- Foundation A and G host tooling are integrated
- D1 supplies stable portable value contracts
- F1 core C ABI is integrated

## Write scope

- `tools/parity-harness/**`
- `tools/bench-harness/**`
- `tools/xtask/**`
- `.github/workflows/ci.yml`
- shared validation/performance documentation
- focused harness unit fixtures

Both tools packages must be local Cargo members matched by the existing `tools/*` glob. Do not modify capability APIs, Apple reference implementations, C ABI declarations, or root workspace configuration. Restore any temporary root lockfile edits before commit; the orchestrator regenerates the integrated lockfile.

## Parity harness

- Define fixed-input cases, typed success/error outcomes, and a comparator for a statically selected reference/candidate pair.
- Preserve error category and native code where the capability contract requires them.
- Permit only explicit per-suite normalization; record every normalized field in output.
- Return a mismatch record containing case ID, input, normalized reference result, candidate result, and OS/SDK labels so it can become a regression fixture.
- Test equal results, unequal values, unequal errors, and explicit normalization with deterministic fake adapters.
- Keep platform frameworks out of the shared harness. Apple adapters belong to capability-specific test targets.
- Keep `cargo xtask parity` unavailable when no real reference/candidate suite is registered; the message must explain the missing inputs.

## Benchmark harness

- Record target triple, optimization mode, workload ID/input shape, warmup/sample counts, raw sample values, median, p95, and p99 latency.
- Allow candidate code to report explicit allocation/copy/byte/energy counters, but do not infer or fabricate them.
- Distinguish host microbenchmarks (advisory) from Release measurements on representative physical Apple hardware (authoritative for iOS replacement selection).
- Include sample validation for stable statistics and JSON output; no external benchmark/runtime dependency is required.
- Do not provide a `select Rust default` switch. Apple remains the default until parity and representative-device evidence exist.

## CI and checks

- Run `sh bindings/c/check.sh` on macOS so C11/C++17 headers, C layout, symbols, and a linked C consumer remain checked.
- Keep parity and device-performance suites advisory/manual until concrete adapters and hardware are available; do not add fake passing fixtures.
- Run format, Clippy, tests, dependency/ABI audits, docs, and zero-Swift-source checks for the added tools packages; run `no_std` checks for portable crates, not the std-based parity/benchmark harness packages

## Current status

- The fixed-case parity API and benchmark record API exist in `tools/parity-harness/**` and `tools/bench-harness/**`; their unit fixtures use fake adapters and fixed sample vectors
- `BenchmarkRecord` summary fields are private and built by `from_samples`; read-only accessors keep the sample count and percentiles tied to the retained raw sample vector
- CI config includes workspace format, Clippy, and test gates, plus dependency/ABI audits, rustdoc, docs-check, and zero-Swift-source checks. This records configured gates, not a pass on the current tree
- G3 checks were recorded on an earlier tree: `cargo fmt --all -- --check`; `cargo clippy --locked -p parity-harness -p bench-harness -p xtask --all-targets --all-features -- -D warnings`; `cargo test --locked -p parity-harness -p bench-harness -p xtask` (19 unit tests: 5 bench, 7 parity, 7 xtask); `cargo doc --locked -p parity-harness -p bench-harness -p xtask --no-deps`; `cargo xtask no-std-check` (20 portable crates at that earlier snapshot, default and no-default features; no link proof); `cargo xtask dependency-audit` (inventory only); `cargo xtask abi-audit --output target/xtask/abi-audit.json` (source inventory only); `cargo xtask docs-check` (docs index and zero-Swift checks); and `sh bindings/c/check.sh` (`framework ABI 1.0`). That recorded F1 output predates the options-validator follow-up; the current C ABI is 1.1; C and C++ ABI 1.1 apps now link but were not executed
- The ABI audit source inventory now scans portable `crates/**` and the opt-in C ABI implementation at `bindings/c/src/**`; it remains source-only and does not certify C header layout, exports, linking, or native imports
- After checkpoint `0c83378`, `cargo +1.94.1 fmt --all -- --check`, `cargo +1.94.1 xtask no-std-check` (47 portable crates, default and no-default features; compile checks, not link proof), `cargo +1.94.1 xtask dependency-audit` (318 package nodes and 0 duplicate package-version rows; inventory only), `cargo +1.94.1 xtask abi-audit --output target/xtask/abi-audit.json`, and `cargo +1.94.1 xtask docs-check` passed; no test command was run
- `cargo xtask parity` exits with `xtask: parity is unavailable until a real Apple reference adapter and Rust candidate suite exist`; this is the expected no-suite gate, not parity evidence
- No real Apple reference adapter or Rust candidate suite exists, no framework workload exists, and no parity or performance result exists. `cargo xtask parity` remains unavailable; representative-device Release measurements and optimized-code inspection remain open

## 2026-10-10 hosted CI evidence

GitHub Actions run [38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431) passed at source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`; `Rust checks (ubuntu-24.04)`, `Rust checks (macos-15)`, and `Rust checks (xcode-27)` all concluded successfully. The G3 source paths (`tools/parity-harness/**`, `tools/bench-harness/**`, `tools/xtask/**`, `.github/workflows/ci.yml`, `docs/PERFORMANCE.md`, and this plan) have no path changes from that source SHA through checkpoint `d717f8ae4324bf2abc782816c7fb2e10dfa92105`; `docs/VALIDATION.md` was also unchanged through that checkpoint. Its later edits only record G19/G20–G29 evidence and do not alter G3 gates, so the hosted results apply to the current G3 implementation.

On both macOS jobs, the following named steps passed:

- `Format`: `cargo fmt --all -- --check`
- `Clippy all features on Apple host`: `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`
- `Unit tests`: `cargo test --locked --workspace`; G3 package fixtures passed: 5 `bench_harness`, 7 `parity_harness`, and 7 `xtask` tests (19 total)
- `Rust documentation`: `cargo doc --locked --workspace --no-deps`
- `Portable no_std checks`: `cargo xtask no-std-check` (portable crates only; compile checks, not link proof)
- `Dependency inventory`: `cargo xtask dependency-audit` (inventory only; no dependency-growth budget or transitive-`std` proof)
- `ABI source inventory`: `cargo xtask abi-audit --output target/xtask/abi-audit.json` (source inventory only; not linked-ABI proof)
- `Documentation and zero-Swift-source checks`: `cargo xtask docs-check`; the zero-Swift-source check and shared documentation index check both passed
- `C API header, layout, symbol, and consumer check`: `sh bindings/c/check.sh`; the script reported framework ABI 1.3

These gates qualify the harnesses, workspace, source inventories, documentation, and C check only. The harness tests use fake adapters; no real Apple reference adapter, registered candidate suite, or framework workload exists. The run did not produce a parity result or performance measurement; `cargo xtask parity` remains unavailable as expected. Representative-device Release measurements and optimized-code inspection remain open.

## Handoff

Report harness APIs, fixture schema, emitted benchmark fields, CI commands, actual suites registered, exact parity/benchmark evidence, and unsupported host/device checks. Clearly state when only harness unit fixtures ran.
