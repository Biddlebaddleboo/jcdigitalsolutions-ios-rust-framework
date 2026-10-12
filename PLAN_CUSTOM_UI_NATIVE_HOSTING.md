# PLAN_CUSTOM_UI_NATIVE_HOSTING — Reusable native iOS custom-rendering capabilities

## Status, scope, and cleanup contract

**Standalone temporary implementation plan.** Repository: `jcdigitalsolutions/jcdigitalsolutions-ios-rust-framework`; target: `main`; inspected base: `5d46b96a424e542af989670b72d65050db46bbf2`.

This plan is deliberately independent of existing planning files: **do not overwrite, revise, combine with, or delete any other `PLAN*.md`**. This repository must remain a general-purpose low-level native framework; `micro-ui` is one possible client, not a dependency or architectural owner.

**Deletion requirement:** Once *all* phases below are implemented, integrated, validated, and reviewed, the implementation orchestrator **MUST delete this exact `PLAN_CUSTOM_UI_NATIVE_HOSTING.md` file and commit the deletion**. If any required work remains unresolved, retain the file and report remaining scope. Do not delete other plan files. An implementing agent should not leave this temporary plan in the final merged tree.

**Physical-device testing is explicitly NOT required**, and lack of an iPhone must not block completion. Require local tests and appropriate iOS simulator checks, distinguishing executed evidence from skipped/unavailable tests. No unperformed device tests may be claimed.

## Objective

Provide the minimal, modular, Rust-native public-Apple-API capabilities for a third-party/custom Rust UI engine to:
1. create and lifecycle-manage a single native rendering host view in a caller-controlled UIKit window/view hierarchy;
2. submit borrowed Rust-owned framebuffer content without undocumented copies or dangling references;
3. receive stable geometry, touch, keyboard/text-input, and accessibility interoperability events;
4. optionally present through Metal if supported and beneficial.

The framework **does not** implement widgets, layout algorithms, declarative syntax, a VDOM, a custom rendering engine, a global executor, a platform-neutral graphical object model, or micro-ui-specific logic. Native UIKit remains native; custom-rendered applications opt into the proposed capabilities.

## Verified repository facts — starting point

- `platform/ios/ios-runtime/src/lib.rs` currently exports `main_thread` and `ffi`, with feature-scoped `dispatch` and timers. This crate explicitly is not a process-wide runtime or UI model.
- `ios_runtime::main_thread::MainThread` and `ios_runtime::ffi::catch_unwind` already exist. Reuse their actual contracts.
- `examples/ios-minimal/src/app.rs` creates UIKit `UIApplication`, `UIWindow`, `UIViewController`, `UIView`, `UILabel`, and `UIButton` in Rust; its `AppDelegate` and `ButtonTarget` use `objc2::define_class!` and `objc2` ownership and selectors. `examples/ios-minimal/Cargo.toml` demonstrates target-gated `objc2-ui-kit` features.
- Root `Cargo.toml` declares `objc2`, `objc2-ui-kit`, `objc2-core-graphics`, and `objc2-metal` dependencies. Their declaration does **not** prove a reusable framebuffer or Metal host API already exists.
- `docs/ARCHITECTURE.md` and `docs/API_DESIGN.md` prescribe modular capability crates, no mandatory global runtime, static selection where practical, Rust fast paths, optional native escape hatches, no shipping Swift source, and supported public Apple APIs. The initial UIKit backend is native and the framework is *not* to become a Flutter-like renderer.
- `docs/IOS_BUILD.md` reports an iOS minimal example and simulator evidence, **not** complete custom rendering, touch, IME, or accessibility capability. Archive/linkage tests are not runtime proof.
- `AGENTS.md` mandates use of documented external shared build/validation tooling. Inspect `docs/SHARED_TOOLING.md` and capability-specific validation specs before extending them; do not alter shared validator engines, pins, archives, or maintenance-owned workflows.

These are verified architectural facts, not claims that proposed APIs below already exist. The executor must recheck latest `main` and inspect the specific target capability crates to avoid duplication.

## Implementation scope — exact first reads

Start with:
- `AGENTS.md`, `docs/ARCHITECTURE.md`, `docs/API_DESIGN.md`, `docs/OBJC_INTEROP.md`, `docs/OWNERSHIP.md`, `docs/UNSAFE.md`;
- `docs/IOS_BUILD.md`, `docs/SHARED_TOOLING.md`, `docs/VALIDATION.md`, `docs/TESTING_AND_PARITY.md`;
- `platform/ios/ios-runtime/src/lib.rs`, `platform/ios/ios-runtime/src/main_thread.rs`, `platform/ios/ios-runtime/src/ffi.rs`;
- `examples/ios-minimal/src/app.rs`, `examples/ios-minimal/Cargo.toml`, root `Cargo.toml`;
- `tools/validation/specs/validation-v1.json` and the relevant existing source/test fixtures.

Next identify existing capability crates and specific source symbols for UIKit views, graphics, touch, text, and accessibility *before* choosing new write paths. Reuse working interfaces where their contracts meet this plan. The exact proposed file name, crate name, and symbol ownership must be frozen by the orchestrator after this bounded inspection, before delegating dependent work. Prefer focused new iOS capability module/crate(s) to expanding `ios-runtime` into a UI abstraction or modifying unrelated capabilities.

## Proposed shared interfaces and invariants

Proposed conceptual types, **not existing API symbols**: `NativeRenderHost`, `FramebufferView<'a>`, `PixelFormat`, `DamageRect`, `HostGeometry`, `NativeTouchEvent`, `TextInputSink`, `AccessibilityNodeId`.

Contract to define before workstreams:
- Host creation, parent attachment/detachment, redraw/invalidation and destruction are main-thread-bound where UIKit requires it. Use `MainThread`/`objc2::MainThreadMarker` patterns, not unverified thread assertions.
- Each submit includes width, height, byte stride, format, alpha/byte-order semantics, and validated damage region. Bounds/stride/buffer arithmetic uses checked operations; reject invalid/overflow values without OOB reads, unbounded memory requests, or silent clipping of erroneous buffers.
- Explicit buffer lifetime: synchronously copied, borrowed only for the duration of a synchronous draw, or retained with an owned transfer/fence. **Never retain borrowed Rust slices after their valid lifetime**. Separate the presentation backing storage from application-owned state.
- Define frame order and invalidation on resize/backgrounding; stale submissions after teardown must fail safely or be ignored by documented contract. No use-after-free, native retain cycle, reentrant mutable borrow, or FFI unwind.
- Strong/weak native references must follow UIKit conventions. FFI callbacks contain panics using the established boundary approach; terminal errors retain diagnostics.
- Input identities and timestamps are stable for the duration of a touch sequence, including cancel; UIKit point coordinates, pixel-scale mapping, and safe areas are documented.
- Avoid unnecessary allocator activity, callback boxing, runtime registry, global lock, double render trees, or serialization between Rust layers. Any costful operation is documented.
- Consumers own widget semantics, hit testing, gesture interpretation, state, drawing policy and scheduling policy. A reusable host only conveys presentation and native platform events.
- Portable `no_std` semantic interfaces must not expose UIKit/native objects; iOS types stay target-local.
- Optional capabilities do not pull in Metal, text input, or unrelated Apple frameworks when absent. Retain native escape hatches.

## Phase 1 — Native host and CPU framebuffer (highest priority)

**Read and reuse:** `examples/ios-minimal/src/app.rs`, `platform/ios/ios-runtime/src/main_thread.rs`, `platform/ios/ios-runtime/src/ffi.rs`, relevant UIKit/graphics capability crates once identified.

**Write ownership:** The orchestrator chooses one hosting capability crate/module as the *sole owner* of host lifecycle, geometry and framebuffer interfaces; one focused example may consume it. No changes to micro-ui.

Implement:
1. A reusable Rust-owned UIView-backed host, using public UIKit/objc2 integration, embeddable in a caller-owned controller/window. No obligatory per-widget native view.
2. Attach/detach/destroy, layout and bounds updates, scale and safe-area reporting, visibility changes, orientation-related geometry and background/resume handling.
3. Correct CPU framebuffer presentation via suitable public Core Graphics/Core Animation path. Explicit initial supported formats, alpha, endianness, row stride and alignment; deterministic invalid-input errors; safe ownership for pixel storage; invalidation/damage contract.
4. Frame scheduling and synchronization that prevent stale frame callbacks or retained borrowed pointers. Prefer redraw on demand.
5. Narrow documentation and a reusable standalone Rust example which paints at least two changing frames without relying on high-level UI widgets.

**Gate A:** local unit/ABI tests, iOS build and simulator runtime smoke prove one-host create → present → update → resize → destroy. Report simulator availability and evidence accurately.

## Phase 2 — Native multi-touch and pointer delivery

**Read-only dependency:** Frozen Phase 1 host lifecycle and geometry contract. **Write scope:** isolated input adapter and tests, not host public interface without orchestrator approval.

Expose touch began/moved/ended/cancelled with stable per-contact identity, timestamp, coordinates, supported modifiers/metadata, and native-interruption cancellation. Support simultaneous touches, changed and coalesced input where supported without inventing guarantees. Clarify point-space vs pixel-space. Ensure teardown and loss of focus cancel active input; reentrant callbacks and stale handles are safe. Never swallow critical cancellation; gesture policy belongs to consumers.

**Gate B:** deterministic generated multi-touch sequences, ordering/cancellation tests, simulator-driven event smoke where available.

## Phase 3 — Keyboard and text input

**Read-only dependency:** host and focus/geometry lifecycle. **Write scope:** text-input bridge/capability and tests; avoid a bundled widget or editor.

Support first responder, keyboard state/geometry, input traits, committed insertion/deletion/replacement, marked/composing text, selection range and updates, Unicode/UTF-16 offset translation, focus transfers, cancellation, and teardown while composition is active. Use appropriate public UIKit text-input protocols rather than treating IME input as simple keypresses. Define precise transaction/event ordering and who owns text storage and selection. Provide a simple test consumer that edits Rust-owned text.

**Gate C:** deterministic selection/composition/Unicode tests, simulator focus/keyboard integration where available; no dangling delegate or stale callback on destroy.

## Phase 4 — Accessibility bridge

**Read-only dependency:** host geometry and lifecycle. **Write scope:** accessibility adapter and tests.

Expose native accessibility elements for independently supplied semantic nodes: stable node identity, labels, values, hints, traits, frames, activation/custom actions, order, focus/scroll notifications and removal/invalidation. Convert coordinate spaces precisely. Native elements must not invoke callbacks into destroyed Rust state; preserve UIKit accessibility ownership semantics and VoiceOver expectations.

**Gate D:** semantic-tree update tests, stale-id/action regression, simulator accessibility interoperability evidence where practical.

## Phase 5 — Optional Metal presentation

**Read-only dependency:** Phase 1 presentation contract. **Write scope:** feature-isolated Metal adapter and benchmarks, not mandatory host API redesign.

Investigate `CAMetalLayer`/Metal only after CPU baseline correctness. Permit packed palette-indexed rendering or shader-based unpacking *as one optional client format*, not a framework-wide colour policy. Define uploads, texture/drawable lifetime, synchronization, resize, device resource errors, suspend/resume and fallback to CPU. Avoid shader compilation or GPU initialization when feature unused. Benchmark CPU vs Metal at realistic screen sizes, sparse/full damage and varying refresh rates; do not promise automatic performance improvement.

**Gate E:** compile/link tests, simulator rendering where available, measurable benchmark comparison; retain CPU fallback.

## Workstreams, dependency order, and shared-file ownership

- **A: Hosting + CPU raster presentation** — owns the host/surface/geometry/presentation interfaces and integration example.
- **B: Touch** — consumes A; owns its separate adapter/tests.
- **C: Text** — consumes A; owns its separate adapter/tests.
- **D: Accessibility** — consumes A; owns its separate adapter/tests.
- **E: Optional Metal** — consumes A and measurements; owns optional renderer/tests.

Implement A and freeze the native host interface first. B/C/D can be delegated in isolated branches/worktrees after A if their write surfaces are distinct; E follows CPU baseline and is integrated last. One owner controls shared host symbols and any root workspace or validation-spec hotspots; other workstreams request changes through that owner. Do not decompose merely to increase parallelism.

Integration sequence: A → B/C/D → E → full validation. Any contradiction or interface change must be reconciled centrally and communicated before downstream work proceeds. Each executor reports files changed, commit SHA, test commands/results, deviations and assumptions.

## Validation commands and evidence policy

Follow exact installed tool/selected PATH version and commands in `AGENTS.md` and `docs/SHARED_TOOLING.md`; inspect registered capability names using `ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --explain CAPABILITY_ID`, then execute the applicable `--capability`, `--changed EXPLICIT_BASE`, or `--all` checks. The repository's immutable shared tooling is out of scope.

Run at least, where supported:
- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo xtask ios-build --simulator --release`
- `cargo xtask archive-smoke` (unsigned archive/linkage only; *not* a runtime test)
- Focused simulator launch, framebuffer visual correctness, input, text and accessibility smoke checks on an available iOS simulator.

Record the actual toolchain, targeted crates, CI/tooling restrictions, commands executed, and failures/skips. Prefer local fixtures, fake clocks and synthetic input over sleeps or external services. Add memory/ownership tests, sanitizer or stress passes as supported. If the repo-mandated shared tooling reports defects follow its bug-report process; do not weaken validators.

**Explicit exemption:** Real on-device testing is **not required**. Simulators are acceptable for runtime integration. Never claim physical-device coverage or re-label skipped simulator tests as successful. Where simulator tooling is unavailable, explain missing runtime evidence clearly and assess whether the required smoke gate has actually passed; do not silently declare completion.

## Non-goals and final review

Non-goals: changing `micro-ui`, a general UI framework, widget language, mandatory GPU path, terminal/SSH rendering, Swift implementation, mandatory runtime or universal async executor, replacing UIKit, extending shared-tooling engines, or changing unrelated planning files.

Final diff must confirm:
- Reusable one-view custom rendering surface and CPU framebuffer contract.
- Correct buffer lifetime, dimensions/stride validation and redraw/teardown.
- Native multi-touch and interruption cancellation.
- Text composition/selection Unicode correctness.
- Native semantic accessibility bridge.
- Optional, measured Metal route with CPU fallback.
- Minimal iOS-only dependency impact; no unwanted portable-core changes.
- FFI safety, main-thread constraints, callback lifetime, and failure recovery.
- Completed unit, local validation and simulator checks; no physical-device test requirement.
- Existing plans untouched and no `micro-ui` dependency added.

## Required implementation handoff and cleanup

1. Verify latest `main` and reconcile changed symbols with this plan.
2. Read this file first; inspect the exact starting paths above before broader searches.
3. Implement the phases with bounded workstreams and stated ownership.
4. Integrate in order, run all applicable checks and review combined diff independently against every requirement.
5. Require executor reports listing changed files, commit IDs, tests, deviations and unresolved assumptions.
6. Resolve contradictions centrally; do not silently redesign architecture.
7. **Once all phases are implemented and validated, delete `PLAN_CUSTOM_UI_NATIVE_HOSTING.md` and commit its deletion; preserve every other `PLAN*.md`.**
8. Report implementation commits, cleanup commit, final validation and remaining limitations. If anything essential is unfinished, **retain this plan** and report the blocker rather than falsely declaring completion.
