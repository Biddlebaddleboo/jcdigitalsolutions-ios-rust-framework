# PLAN.md — Active framework implementation handoff

## Authority and baseline

Repository: `Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework`; branch: `main`. Planning audit baseline: `f3b14892a560da05a086caf9e6d30a0c9beda25e`. **Before execution, verify the current remote `main` SHA and reconcile any relevant changes.** This file supersedes the former infrastructure-only stop instruction; it does not authorize indiscriminate implementation of every historical `PLAN_*.md` file.

## Objective and current state

Resume bounded work on genuinely unfinished framework capabilities, backends, bindings and cross-cutting requirements with R1 shared native build and R2 shared validation already implemented. R3 verified interop work is distinct: reconcile any truly unfinished R3 requirements against current source before authorizing changes. The completed reusable build and validation engines run as pinned, PATH-installed `ios-rust-build` and `ios-rust-validate` binaries. Their engine source was removed from ordinary `main` at `8b68d2909de9b5e926130265ba241c80ab653e99`; the immutable historical source and release provenance remain available for separately authorized maintenance. The operational contract is `docs/SHARED_TOOLING.md`; `AGENTS.md` remains authoritative for architectural, safety and agent behavior rules.

Historical capability coverage is **partial, not complete** merely because a crate or scalar getter exists. `docs/capabilities/capability-status.json` defines canonical capability identities and support states. Compiler, link, simulator-runtime and physical-device proofs remain distinct. Prior Xcode 26.6 / iOS SDK 26.5 findings do not satisfy an Xcode 27.x qualification requirement. The historic Ubuntu `objc2` Clippy failure was traced to unconditional Apple-only dependencies in `platform/ios/ios-sign-in-with-apple-status/Cargo.toml`; they are now target-gated to iOS. CI runs Linux host features on Ubuntu and keeps full `--all-features` Clippy on macOS, where the intentional `swift-abi-generated` Apple-runtime guard can pass. Actions run [38030784599](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38030784599) and [38031119997](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38031119997) confirmed the Ubuntu job passes, including Clippy and its automatically triggered unit tests. Locale-dependent ordering was fixed in Files, Connectivity, Connection and URL, then in a 39-script iOS import sweep, preserving all allowlists and guards. After `3900cd1` isolated VideoToolbox, all 39 focused audits pass under UTF-8 locale: 31 passed in three parallel batches, seven had passed earlier, and the audio-playback audit passed after the feature fix. Runs [38031393177](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38031393177), [38031642660](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38031642660) and [38032001726](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38032001726) successively exposed missing `x86_64-apple-darwin` and undeclared `rg` requirements; `e51c09d` installs the target, `d77f79a` uses standard `grep` for MessageUI guards, and `9cb1c19` provisions ripgrep for later package checks. Runs [38032154390](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38032154390) and [38032286404](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38032286404) passed provisioning and Ubuntu, then failed the Apple Pay import allowlist because ambient locale changed `PassKit`/`libSystem.B.dylib` order. `cb5b281` pins `LC_ALL=C`; the next run [38032694034](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38032694034) confirms Apple Pay passes and Ubuntu remains green, but macOS fails the ImageIO archive audit because Xcode 16.4 `nm` cannot read Rust LLVM 21 objects. `bbc0ad9` installs Rust `llvm-tools` and uses its version-matched `llvm-nm` without weakening symbol assertions; focused device and Simulator archive audits pass locally on Xcode 26.6. The same run's standalone audio import gate exposed an accidental default VideoToolbox dependency shared with B50; `3900cd1` makes it opt-in for B50 and the C feature, and the focused audio device/Simulator audit now passes with the original allowlist. Current Xcode 27 lane and local package-gate status is recorded below.

Current CI and local gate status (2026-10-10): the Xcode 27 qualification lane was added at
`2a38984`. Runs [38033435519](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38033435519),
[38033753075](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38033753075),
and [38034003834](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38034003834)
reproduced the same `xcode-27` import-list failure: expected `CoreMedia` + `libSystem.B.dylib`, got
`libSystem.B.dylib` + `libswiftCoreMedia.dylib`. Run
[38034156057](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38034156057)
adds failure diagnostics and confirms Xcode `27.0` build `27A266a`, SDK `27.0`, device target
minimum iOS `12.0`, `LC_LOAD_DYLIB @rpath/libswiftCoreMedia.dylib`, and undefined `_CMTimeMake`.
This is a strong dylib dependency, not a weak load. The B22 fix at `31df44f` now constructs the
public `CMTime` fields directly with `CMTimeFlags::Valid` and epoch zero, avoiding `CMTimeMake` and
its strong Swift runtime dependency while preserving the iOS 12.0 device floor. Strict local
device/Simulator link-import and layout checks pass on Xcode 26.6 / SDK 26.5; this is not Xcode 27
qualification or runtime proof. Run [38035161806](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38035161806)
uses pre-fix `a66b4ca`: Ubuntu passed; Xcode 27 failed at the old media import gate; macOS passed
all four HomeKit checks after the prefetch fix, then failed the Foundation Models Swift oracle
because Xcode 16.4 lacks that module. The existing Foundation Models gate now runs only on
`xcode-27`, not removed. Post-fix run
[38035445897](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38035445897)
passed Ubuntu and the Xcode 27 media import/layout gate, and remains in progress through the other
qualification checks. The macOS 15 Foundation Models result in that run still uses the old
workflow condition. Xcode 27 qualification remains pending.
The `macos-15` leg of run 38033435519 reached `ios-rust-validate --all` and failed the
Photogrammetry Swift ABI oracle and AlarmKit link-import scan: default Xcode 16.4 / iOS SDK 18.5
lacks the expected Photogrammetry ABI lowering and iOS 26 AlarmKit framework. CI now runs the
complete registered-pilot batch only on `xcode-27`; package gates run on `macos-15` except the
Foundation Models compiler gate, which also requires Xcode 27. Runs
[38035445897](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38035445897)
and [38035509951](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38035509951)
pass `Validate shared native pilot capabilities` (`ios-rust-validate --all`) on Xcode 27; run
38035509951 also passes the Foundation Models gate on Xcode 27. Both runs then fail in separate
Apple-lane package checks: the Xcode 27 Sign in with Apple check lacks offline-cached
`objc2-app-kit v0.3.2`, and macOS 15's older workflow condition tries to import the unavailable
Foundation Models module. The Sign in with Apple check now performs a locked online iOS-target
prefetch before its existing offline gates; its CI recheck and the new macOS 15 skip evidence are
pending. Local Xcode 26.6 (build 17F113), SDK 26.5 gates pass for
B47 full audio (`sh platform/ios/ios-media/check-audio-playback.sh`), B50 package
(`sh platform/ios/ios-media/check.sh`), and F25 C link/import
(`sh bindings/c/check-ios-videotoolbox-link.sh`); detailed results and limits are in [PLAN_IOS_OTHER_AUDIO.md](PLAN_IOS_OTHER_AUDIO.md), [PLAN_VALIDATION_IOS_OTHER_AUDIO.md](PLAN_VALIDATION_IOS_OTHER_AUDIO.md), [PLAN_IOS_VIDEOTOOLBOX.md](PLAN_IOS_VIDEOTOOLBOX.md), and [PLAN_BINDINGS_F25.md](PLAN_BINDINGS_F25.md). These local passes do not establish runtime behavior; no tests, probe execution, live codec query, or device behavior is claimed.

## API resume checkpoint and deterministic continuation

### Last verified API checkpoint

- The last substantive `feat(ios)` implementation commit before the repository pivoted to planning consolidation and shared tooling is `ac944e80f195c810d145d51064bb1f9c3c875b49` (`feat(ios): expand bounded native snapshots`). It contains multiple bounded slices across HomeKit, accessibility, iOS files and VPN. **It is not proof that any entire capability or family is finished**, nor proof that one particular subsequent B/D/F/G identifier is next.
- Later commits through the current planning baseline implemented/reorganized R1/R2 tooling and planning documentation; their presence is not evidence of new completed end-user capability work. R1/R2 tooling is completed; do not rebuild it. Preserve valid R3 interop residuals as separate, explicitly justified work.
- The canonical active state is **current `main` code + tests + `docs/capabilities/capability-status.json` + surviving targeted plans**, not the age or filename of a plan. At this checkpoint the manifest reports 98 partial and 16 unsupported rows out of 114, not 98 completed capabilities. These counts must be reverified before changing the manifest.

### Mandatory one-time resume reconciliation (before implementing the next API)

1. Fetch and fast-forward to current remote `main`; record its full SHA. Confirm pinned `ios-rust-build` / `ios-rust-validate` installation and that `AGENTS.md` and `docs/SHARED_TOOLING.md` are in force. Do **not** fetch historical shared engine source.
2. Read this plan, the current capability manifest, `PLAN_CAPABILITIES.md`, `PLAN_IOS_NATIVE.md`, `PLAN_BINDINGS.md`, and `PLAN_VALIDATION.md`. Use the last API commit only as a chronological checkpoint: inspect its changed paths and associated B/D/F/G decisions if needed to resolve whether work was already integrated. Do not indiscriminately load historical `PLAN_*.md` files.
3. Build an **active residual queue** from currently incomplete requirements in the canonical manifest and their surviving targeted plans. Include relevant foundation, native iOS, C ABI, compiler/Swift ABI, validation, and performance dependencies. For each eligible item record: unchanged workstream ID, exact plan and starting symbols, current code/test evidence, unmet criteria, prerequisite IDs, write owner, required Apple host/device evidence, and `ready` / `blocked` state. Treat `research_closed`, completed one-off slices and no-go outcomes as constraints, not automatic implementation candidates. Never infer completion from a matching crate or status getter.
4. Select the **earliest ready existing workstream in its recorded prerequisite/plan order**, with preference to closing a useful existing capability contract over inventing an unrelated scalar snapshot. If the original sequence cannot be reconstructed unambiguously, or two independent candidates have no established priority, **record the ambiguity and request selection**; do not invent a next B number. If all remaining candidates are blocked, report their prerequisites and stop rather than changing unrelated APIs.
5. Start only the selected bounded workstream and its **necessary, explicitly owned prerequisite tasks**. Use isolated worktrees for parallel-safe independent scopes; serialize shared files. Before code changes, check its targeted plan against current source and tests; do not reimplement already integrated slices. Existing user authorization to continue the residual API plan does not override specific no-go, security, Apple SDK, or device validation gates.

### Execution and checkpoint update after each integration

- Follow `AGENTS.md` and `docs/SHARED_TOOLING.md`: add a declarative validator profile only for genuinely unregistered work, preserve package-local and compiler/link/ABI gates until parity is proved, and never treat skipped physical-device evidence as a pass.
- Require the executor to report existing workstream ID, source file/symbol changes, tests and failures, limitations, final diff, and implementation commit SHA. Integrate in prerequisite order, recheck remote `main`, and rerun affected cross-workstream tests.
- Recompute which requirements remain open from current code/tests and update their targeted plans, docs and manifest as appropriate. Record the most recent completed implementation SHA and the next **reconciled** ready workstream in this checkpoint before the next implementation cycle; do not automatically advance from a partial result or a blocked task.
- Continue only within the user-authorized remaining scope, and stop on ambiguous priority, architectural contradiction, unauthorised expansion, tool-engine defects, or missing mandatory evidence. Engine defects use sanitized `BUG_REPORT_*.md` and separate maintenance.

### Current resume state

Last integrated bounded API workstream: D55 at `5800f8a` (StoreKit 1 payment-ability status; evidence only, no product code change). D54 is integrated at `1ac40c6`; its existing `ios_natural_language_status::english_contextual_embedding_assets()` met the D54/B59 scope, and only dated plan evidence was added. The D54 and D55 package gates did not execute probes or tests and do not prove live assets, model/vector behavior, or payment behavior.

Resume reconciliation after D55: `origin/main` was fetched at `c55410d14c57c3ffdc1982cf62ae3180dd18c468`; it remains an ancestor of local `main`. Integrated code workstreams: F34 at `e1e9f89` adds C++17 `OwnedBuffer::from_transfer`; A at `0a5f40a` adds `framework_owned_buffer_copy` and bumps the C ABI minor to 1.2; B435 at `56ec81b` adds `WidgetCenter.invalidateConfigurationRecommendations()` for iOS 16+; F35 at `2073c4a` adds direct-owned C error detail and bumps the ABI minor to 1.3; F36 at `81ed76b` adds an isolated PyO3 `OwnedBytes` extension; F37 at `0f2c471` adds the move-only C++17 `framework::ErrorDetail` owner and borrowed `std::string_view` view for F35. F37 formatting, C++17 syntax-only consumer compilation, and whitespace checks passed in its workstream and again after integration. No tests, linked consumers, or runtime calls ran for F37. After F35, root `framework-abi`/`framework-c-api` check, C example syntax, and release archive build passed; F35 also passed format, manifest/symbol parity, C11/C++17 syntax and docs checks in its isolated worktree. F36 passed offline check, strict Clippy, rustdoc, and release linking; its extension imports no `libpython`, and root Cargo metadata excludes `framework-python`. No tests, Python imports, or native runtime calls ran for F35/F36. Apple documents B435's method as inactive on iOS, so success reports only that the native call returned. B436 was blocked and recorded at `a68777c`: the available C++ `String` construction path truncates embedded NUL, the exact-byte route requires shipping Swift source, and the Foundation alternative relies on an excluded underscored symbol. D97 records no WidgetKit successor at `1058e48`. F23 root wiring was present at `bf544b2`; its static and device/Simulator link-import gates passed, and linked probes were not executed. F24 root wiring was present at `c55410d`; both F24 gates passed, including the iOS 12.2 device and iOS 14.0 Simulator link checks; probes were not executed. F25–F33 already have current-main integrations and targeted plan evidence; no test or live-runtime claims are added here. No new binding workstream is recorded after F37. The remaining B/D queue has no selected successor after B436 (blocked) and D97 (no recorded successor). Under step 4 above, the next capability workstream is ambiguous and needs user selection; do not invent a new ID. Recheck remote `main` before each integration; keep runtime behavior and device evidence distinct from compiler/link results.

R3 residual reconciliation is integrated at `3a33fa2` (`refactor(swift-abi): share value layout`). A shared `swift_abi_value_storage_layout_from_metadata` helper replaces duplicate metadata/VWT, size/destroy, and alignment checks in AlarmKit `AuthorizationState` and Photogrammetry `Limits`; ActivityKit remains unchanged because its scalar path uses no Swift value. The workstream reports passing format, Rust core check, targeted device/Simulator ABI oracles, AlarmKit Swift/C IR compile, device/Simulator import checks, docs, zero-Swift-source, and diff checks. No tests, linked probes, runtime calls, or physical-device queries ran. This does not close the Xcode 27.x qualification gap or the C6 async/Swift task-entry no-go.

Validation workstreams integrated after the `7276814` checkpoint: `3fea42e` target-gates Sign in with Apple dependencies, fixes the Contacts lazy-future Clippy fixture, and splits Linux host-feature Clippy from macOS full all-features Clippy. Actions runs `38030784599` and `38031119997` confirm Ubuntu Clippy and automatic unit tests pass. `99d56db`, `fbff4fd`, `e00e18e`, and `b938d8b` make iOS import audits deterministic with `LC_ALL=C sort` while preserving allowlists and guards. After `3900cd1`, all 39 focused audits pass under UTF-8 locale: 31 passed in the three batches, seven passed earlier, and the audio-playback audit passed after the feature fix. No tests ran locally. `33a6ec7` removes only a stale external-validator entry from the xtask help assertion. `e51c09d` installs `x86_64-apple-darwin`; `d77f79a` replaces MessageUI's undeclared `rg` source-guard dependency with standard `grep`; and `9cb1c19` provisions ripgrep once in macOS CI. Runs `38032154390` and `38032286404` passed Ubuntu/provisioning but failed the Apple Pay locale sort; `cb5b281` fixes it and run `38032694034` confirms that gate passes. Run `38032694034` also passed Ubuntu, then failed `Check iOS ImageIO device and Simulator targets` because Xcode 16.4 `nm` rejects LLVM 21 objects with `Unknown attribute kind (102)`. `bbc0ad9` installs Rust `llvm-tools` on macOS and selects the matching `llvm-nm`, retaining all symbol assertions; its focused device/Simulator archive audit passes locally on Xcode 26.6. `3900cd1` makes VideoToolbox an opt-in `ios-media` feature, wires the B50 and C ABI feature paths, and adds a negative guard for B47's audio-only graph; its focused audio device/Simulator import check passes and the allowlist is unchanged. The Xcode 27 lane state is recorded above; qualification has not passed. No tests ran locally.

## Active work selection and limits

1. Inspect the canonical capability status/owner, the relevant family plan, then the named workstream plan and exact files/symbols. Do not scan the entire planning corpus or treat every existing plan as pending work.
2. Classify the selected requirement against current implementation, tests and historical recorded decision: `complete`, `partial`, `blocked`, `superseded`, `not_started`, or `research_closed`. A prior `no-go` is a constraint, not an invitation to invent an implementation.
3. Implement only user-authorized remaining work. Select only the next reconciled ready workstream using the API resume checkpoint procedure; do not invent new workstreams or extend beyond the approved residual scope. Deferred work is preserved, not canceled.
4. Existing P/R/A/B/C/D/F/G identifiers and all capability IDs are permanent; never renumber, recycle or use retired gaps. For future workstreams allocate the next unused greater ID in its original series after consulting history.
5. Plans specifying a past source baseline must be reconciled to latest `main` before writing. Outdated commands do not override working architecture.

## Implementation ownership and dependencies

| Contract family | Authoritative entrypoint | Primary change owner |
|---|---|---|
| Portable semantics, `no_std`, error and lifecycle contracts | `PLAN_FOUNDATION.md`, `crates/framework-*/` | A / portable owners |
| Capability coverage, Apple platform feasibility and no-go decisions | `PLAN_CAPABILITIES.md`, canonical capability manifest, targeted `PLAN_CAPABILITIES_*.md` | D and associated backend owners |
| Native iOS implementations, app lifecycle, ObjC/Swift boundaries | `PLAN_IOS_NATIVE.md`, targeted `PLAN_IOS_*.md`, `platform/ios/*` | B |
| C ABI, C++ conveniences and optional Python bindings | `PLAN_BINDINGS.md`, targeted `PLAN_BINDINGS_*.md`, `bindings/*` | F |
| Runtime/compile/link/test requirements and evidence classifications | `PLAN_VALIDATION.md`, targeted `PLAN_VALIDATION_*.md`, `docs/SHARED_TOOLING.md` | G and each capability owner for local specs |
| Verified reusable Swift ABI primitives | `PLAN_REUSE_INTEROP.md`, `interop/swift-abi-*` | R3, only for verified remaining contract gaps |
| Performance replacements | `PLAN_REPLACEMENTS.md`, evidence-backed targeted plan | replacement owner |
| Engine bugs or unsupported tooling protocols | `BUG_REPORT_*.md`, separately authorized maintenance work | distinct tooling-maintenance Codex session, **not** API executor |

Minimize overlapping writes. If shared interfaces or files are required, assign one owner; other workstreams consume them. Explicitly order dependencies and integrate sequentially across such hotspots. Parallelize only independent work in isolated worktrees.

## Shared tooling contract for every assigned workstream

- Install verified, pinned host executables using `tools/install-tools.sh --prefix <PREFIX>`, place `<PREFIX>/bin` on `PATH`, verify both tools' version/protocol, and consult `docs/SHARED_TOOLING.md` for exact commands/schema.
- Cargo native C archive construction is performed by `ios-rust-build` via the package's minimal Rust `build.rs`; the ordinary executor may edit the capability-specific `build-spec.json` and its small bridge only when the capability actually needs native archive support. Do not recreate SDK discovery, Clang/ar loops, or the shared engine.
- Validation defaults to **declared checks** in `tools/validation/specs/validation-v1.json`, where applicable. The installed validator currently has only four configured production pilots (HomeKit identify, ActivityKit status, AlarmKit status, Photogrammetry status); **do not assume a new capability is registered or that all legacy checks are migrated**. Add a fully specified new capability declaration when supported, rather than rewriting the Rust engine.
- For unusual API-specific assertions use an optional, bounded Python 3 adapter through the versioned stdin/stdout JSON protocol. Retain necessary native fixtures and true compiler/link checks for ABI layout, ownership, Swift calling convention, weak symbols and availability. A required missing/failed/invalid adapter is never a pass.
- Retain applicable legacy package-local `check.sh`, compiler-oracle tests, import audits and CI gates until the new configuration demonstrably enforces equivalent or stronger behavior. Conversion of a check is a migration requiring negative-case parity, not just a path replacement.
- Distinguish `PASS`, `FAIL`, `SKIPPED(reason)` and `ERROR-BLOCKED`; no absent SDK/device or skipped gate may be claimed passed. Tooling does not substitute real permission, entitlement, interaction or device tests.
- Ordinary executors must not fetch, read or modify historical shared-engine source. For an apparent engine defect: check the documented CLI/schema and prerequisites, reproduce minimally, sanitize evidence, commit `BUG_REPORT_<DESCRIPTIVE_NAME>.md`, stop the affected work, and hand it to a separate maintenance session. Do not weaken gates or build an ad-hoc replacement validator.

## Workstream handoff requirements

Each assigned plan must name: exact starting files and symbols; verified current implementation; objective and remaining acceptance criteria; owned write surface and read-only dependencies; explicit non-goals; invariants around errors, ownership, async completion, cancellation, idempotency, security and persistence **where relevant**; host/Apple version limits; declarative validation entries and any custom adapters; real device/simulator evidence requirements; deterministic regression tests; exact commands; and an auditable final diff checklist. Preserve historical evidence/decisions as linked read-only references instead of flooding the routine executor context.

Before integration: verify current `main`, use isolated worktrees for parallel-safe tasks, require each executor to report changed paths, SHA, tests, skips, deviations and open assumptions, then integrate in dependency order. Re-run affected validation and independent ABI/consumer checks. Resolve plan contradictions centrally. Audit the final diff against the entire assigned contract. Remove temporary planning handoff files after implementation when that is the established workflow; do not delete durable history without evidence-backed retirement. **Do not initiate additional API work beyond specifically authorized workstreams.**

## Final review checklist

- Preserved Rust-native fast path, public Apple APIs, static backend choice, optional C/Python bindings, platform invariants, zero shipping Swift source and documented escape hatches.
- No inferred full-capability support from partial getter checks; no unsupported Swift async or private symbols.
- No lost negative assertion, deployment floor, framework import, ownership/cancellation guarantee, feature isolation or runtime evidence gate.
- No shared-engine source, dependency or bespoke replacement accidentally imported into normal checkout.
- Every unfinished workstream has a stable owner and accurate dependency; historical no-go/closed evidence remains recoverable.
- Every test command and evidence statement reflects actual current tooling and installed host requirements.

## Retired infrastructure plans (historical only)

`PLAN_REUSE_BUILD.md`, `PLAN_REUSE_VALIDATION.md`, and `PLAN_TOOLING_FINALIZATION.md` were deleted from current `main` in `f3b14892a560da05a086caf9e6d30a0c9beda25e` because their shared tooling implementation is finished. Do not recreate these plans or assign R1/R2 implementation work. Their former Git history is archival. Any unrelated API-specific validation registration is a capability/validation integration task, not an engine rewrite.
