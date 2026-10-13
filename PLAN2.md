# PLAN2.md — Complete Low-Level iOS APIs Needed for Zero-Swift Applications

**Repository:** `jcdigitalsolutions/jcdigitalsolutions-ios-rust-framework`  
**Target branch:** `main`  
**Verified framework baseline:** `1b23785758523efb6de3b2433c18842e33f4a039`  
**Read-only consumer:** `jcdigitalsolutions/rideshare-bot` at `71323f02cd9c0947248673b206efd1ca6dc72d26`  
**Status:** Approved implementation handoff. This task changes the framework only; do not migrate rideshare-bot.

## Objective and constraints

Supply the missing **generic low-level iOS capabilities** needed for Swift-free apps like rideshare-bot, without implementing app-specific logic, protocols, UI widgets, micro-ui behavior, or Mono1 conversion in the framework. Use public Apple APIs, objc2 where appropriate, existing framework ownership/threading conventions, optional capability-scoped crates/features and no mandatory async runtime or Swift source. Prefer extending existing primitives only when source inspection establishes a real gap. Minimize allocations, copies, binary size and lifetime hazards.

## Verified repository facts

- `platform/ios/ios-app-scenes/src/lib.rs` provides scene activation snapshots, typed scene IDs and lifecycle observation; its README explicitly says it **does not replace host scene delegates or perform UI operations**.
- `platform/ios/ios-app-lifecycle/src/lib.rs` provides application lifecycle notifications and selected open-URI/settings helpers; not a complete UIApplication entry/hosting API.
- `platform/ios/ios-ui/src/custom_render_host.rs` exports `IosCustomRenderHost`, `Rgba8Frame`, `HostGeometry`, `PixelDamageRect`, `attach_to`, `set_frame`, `geometry`, `present`, `detach`, `native_view`. `attach_to` expects a caller-owned UIView; it does **not** own a scene/window/controller.
- `platform/ios/ios-ui/src/custom_render_touch.rs` exports `IosCustomRenderTouchInput::{attach,detach,next_event,cancel_active,fault,native_view}`; `custom_render_text_input.rs` exports `IosCustomTextInput::{attach,snapshot,set_text,set_selection,request_focus,resign_focus,cancel_composition,set_frame,native_text_view}`; accessibility and display-link primitives are already exported from `ios-ui/src/lib.rs`.
- `platform/ios/ios-sharing/src/document_export.rs` implements `IosDocumentExportSession`, `IosDocumentExportFile`, `start`, `start_staged`, `cancel` via UIDocumentPickerViewController. `share_platform.rs` rejects local file URLs in generic shares; a document picker is **not** identical to a UIActivityViewController share sheet.
- `platform/ios/ios-files/src/lib.rs::IosFiles` supplies sandbox-relative file operations and backup-time *inspection*; that getter is not an exclusion setter.
- `platform/ios/callkit/ios-call-provider/src/lib.rs` already has Rust-owned CallKit provider/actions; `platform/ios/ios-background-execution/src/lib.rs` already has background execution leases.
- Consumer `ios/RideshareBot/MicroUiApp.swift` owns UIApplicationDelegate/ UIWindowSceneDelegate, UIWindow, root UIViewController/UIView, touch, draw, safe-area/scale, input, UIAlertController prompts, and UIActivityViewController file sharing. `ios/RideshareBot/Logging/BinaryLogStore.swift` creates Application Support and sets `URLResourceValues.isExcludedFromBackup=true`. `RemoteHoppCallKitCoordinator.swift` has a Rust-backed active CallKit provider path.

**Do not mistake existing primitives for missing features.** Explicitly verify their source-level contracts and only add the narrow missing seam.

## A. P0 — Rust-owned UIKit entry point, application and scene/window host

**Inspect first:** `platform/ios/ios-app-scenes/{src/lib.rs,README.md}`, `platform/ios/ios-app-lifecycle/src/lib.rs`, `platform/ios/ios-ui/src/{custom_render_host.rs,platform.rs}`, existing runtime/main-thread facilities, `docs/{IOS_BUILD.md,OBJC_INTEROP.md,OWNERSHIP.md}`, `docs/ios/app-scenes.md`.

Implement Rust-owned UIKit startup/scene hosting through public APIs: `UIApplicationMain`, UIApplicationDelegate application configuration callback, `UISceneConfiguration`, `UISceneSession`, `UIWindowSceneDelegate`, `UIWindowScene`, `UIWindow`, `UIViewController`, `UIView`, `makeKeyAndVisible`, connected/active/inactive/background/disconnect callbacks, and necessary runtime class registration via objc2. Reuse existing scene identities/events and main-thread types, rather than duplicate them.

**Proposed additions (names adjustable to repository conventions):** `IosApplicationHost`, `IosSceneHost`, typed `IosSceneId`/`IosScenePhase`, bounded `IosSceneContext`, application-facing scene lifecycle handler registration. These are **new proposals**, not verified existing symbols. Expose: Rust app startup; register callback handlers; create/retain a per-scene window/root controller; attach native child UIView; present window; observe scene transitions; dismantle scene safely. Document required Xcode `main`, symbols, scene manifest/Info.plist, principal class and code-signing setup; avoid Swift `@main`.

**Invariants:** Delegates stay alive while UIKit can invoke; separate owned state per scene; no retain cycles; enforce main-thread affinity; catch panic at Objective-C boundary; protect reentrant registration/disconnect and late callbacks; deterministic teardown; no globally owned app-specific state. Scene attachment must not assume a single window. Use public supported SDK symbols and identify missing binding requirements precisely. Test scene startup, multi-scene transitions, reconnect/disconnect, owner release, callback ordering, panic containment. Ship a **zero-Swift** sample that creates/displays a window/view, not merely a successful link check.

## B. P0 — Generic UIKit presentation primitives

**Inspect:** `platform/ios/ios-ui/src/platform.rs`, existing view/controller and presentation APIs, `ios-sharing` presenter ownership, `docs/{OWNERSHIP.md,OBJC_INTEROP.md}`. Determine existing alert APIs before any addition.

Supply narrow reusable API for `UIAlertController`, `UIAlertAction`, style alert/action sheet, `UIViewController.present/dismiss`, addAction/addTextField, `UITextField` configuration including secure input and numeric keyboard, and `UIPopoverPresentationController` anchor on iPad.

**Proposed additions:** `IosAlertSession`, `IosAlertActionStyle` (default/cancel/destructive), `IosAlertTextFieldConfig`, `IosAlertResult` and explicit presenter-bound creation/action callback/dismiss. Naming is illustrative. Avoid app-specific peer selection or settings methods.

**Invariants:** At most one completion; cancellation, scene disconnect and programmatic dismiss all release callbacks; no stale native pointers or unsafe temporary string borrowing; no main-thread blocking; correct iPad anchoring. Test action order, selection, bounded input, secure text, cancellation, double completion, wrong-thread usage, presenter deallocation, multi-scene presentation.

## C. P1 — Compose existing custom rendering, input and accessibility with Rust-owned host

**Inspect:** `platform/ios/ios-ui/src/{custom_render_host.rs,custom_render_touch.rs,custom_render_text_input.rs,custom_render_accessibility.rs,display_link.rs,lib.rs}` and A's agreed host API.

Prefer reusing current types, adding at most a small compositional `IosCustomUiSession` or equivalent **only if** lifecycle/ownership cannot be expressed cleanly through current public APIs. Expose attaching render view to owned root controller, safe-area/scale/viewport updates, touch events, text input/focus/composition, keyboard geometry, accessibility updates, frame scheduling/pause and disconnect cleanup. Do not create a new renderer or business-level UI abstraction.

**Pixel policy:** Keep existing `Rgba8Frame`; consumer may convert packed Mono1 externally. Do **not** add Mono1 or whole-frame conversion to this plan. Benchmark allocations, extra copies and host memory; avoid mandatory full-frame copies introduced by composition. Test resizing, safe area, touch coordinates, focused secure text, display-link background pause, accessibility updates and scene destruction during input. Ship a zero-Swift tiny RGBA8-frame sample.

## D. P1 — Sandbox backup-exclusion setter

**Inspect:** `platform/ios/ios-files/src/lib.rs::IosFiles`, `AppDirectory`, `AppPath`, `path_validation`, sandbox-root/descriptor helpers, existing NSURL resource-key interop and `framework-files` error conventions.

Implement explicit opt-in setter/getter for `NSURLIsExcludedFromBackupKey` (Foundation URL resource values), e.g. `IosFiles::set_excluded_from_backup(AppPath<'_>, bool) -> Result<(), FileError>` and `is_excluded_from_backup`. These are proposed APIs, not existing functions. Retain path validation, sandbox containment, symlink protections, and truthful error semantics; reject missing/malformed paths, report failed setter rather than ignore it; do not confuse backup-time metadata with exclusion. Test file/directory on/off, repeated idempotent call, missing entry, malicious symlink and permission failure. No automatic exclusion on arbitrary app files.

## E. Validate current document export; extend sharing only if generic parity gap proven

**Inspect:** `platform/ios/ios-sharing/src/{lib.rs,document_export.rs,document_export_state.rs,share_platform.rs}`, presenter contexts and staging/callback state.

First prove existing `IosDocumentExportSession` can stage bounded owned UTF-8 bytes as named .txt, present picker from A's Rust controller, return exported/cancelled outcomes, preserve staging file until native operation completes and clean up after failure/disconnect. Distinguish document-picker UX from current consumer `UIActivityViewController` file sharing. If **exact share sheet parity is necessary** and existing API cannot cover it, add only a narrowly scoped staged-local-file share entry to `IosShareSession` with owned file lease, authenticated sandbox path validation, lifetime/cleanup, duplicate-completion prevention and cancellation safety. Do not globally allow arbitrary `file://` URLs by removing the existing rejection. Document accepted native file semantics and security-scope behavior. Tests for filename/UTF-8, invalid paths, early rejection, cancel, background/disconnect, cleanup and duplicate callbacks.

## F. Validate existing CallKit/background execution; no speculative platform APIs

**Inspect:** `platform/ios/callkit/ios-call-provider/src/lib.rs`, provider/action implementation, `platform/ios/ios-background-execution/src/lib.rs`, notification/lifecycle types. Build zero-Swift Rust consumer cases: incoming report, answer/end, duplicate action, deferred action, late completion, provider reset, background lease expiration and cleanup. Add a narrow low-level API only if source and tests demonstrate an exact missing operation. Do not implement PushKit, APNs, audio, signaling or app-specific call state.

## Workstream ownership, dependencies, parallel safety

PLAN2.md authoritative. A owns app/scene host files/crate; B owns new alert/presentation-specific ios-ui files and depends on A's presenter contract; C owns custom UI composition and depends on A; D owns ios-files and is independent; E owns ios-sharing and needs A only for sample integration; F owns CallKit/background validation and is independent. One integration owner exclusively changes shared `ios-ui/src/lib.rs`, workspace manifests, common runtime hooks and central docs. Agree cross-module identity, presenter and callback ownership interfaces **before** B/C/E parallel work. Never allow simultaneous writes to same files; use named PLAN_*.md only for truly independent executor surfaces, with read-only dependencies and explicit handoff.

## Tests, validation and performance

Follow `docs/VALIDATION.md`, relevant crate `check.sh` and pinned toolchain. Run `cargo fmt --all -- --check`, applicable strict Clippy/unit/doc tests, no-default-feature checks, iOS device and Simulator builds, objc2 class/protocol/link inspections and **actual Simulator runtime**. A successful compile does not prove launch/delegate delivery. Use fake callbacks and deterministic lifecycle test doubles rather than sleeps/external services. Physical device checks required for unproven behavior.

**Mandatory zero-Swift example:** Rust entry, UIKit scene/window creation, native root view, custom RGBA8 display, touch handling, alert, text input, document export, foreground/background lifecycle and disconnect cleanup. No micro-ui or rideshare-bot dependency, no Swift source. Confirm its Xcode project, manifest and startup symbols.

Measure baseline vs result for launch time, RSS, per-scene allocations, render host scratch/copies, callback dispatch, main-thread responsiveness, binary size, and teardown retained memory. Avoid universal runtime, heavyweight registries, always-running polling or global app service state; mark unmeasured performance claims as unverified.

## Reporting and final-diff checklist

Record every added file/type/function and exact underlying public Apple API; unchanged reused APIs; ownership/ABI/threading/async/cancellation contracts; completed iOS runtime tests, comparison metrics and remaining gaps; zero-Swift sample; precise rideshare-bot symbol→framework API migration guidance **without modifying consumer**. Confirm no Swift source, app logic, custom Mono1 converter, unrelated framework changes, retain cycles, unsafe callback escape, unbounded input or leaked staged files.

## Execution handoff

1. Fetch latest `origin/main` before implementation and reconcile affected architecture; read PLAN2.md first.
2. Verify already-supported methods and lock down A's scene/presenter interface first.
3. Parallelize only non-overlapping independent workstreams in isolated branches/worktrees; integrate A then dependent B/C/E, D/F as independent.
4. Require each executor: files, commit SHA, tests, deviations and unresolved assumptions.
5. Run cross-workstream tests and zero-Swift runtime example; independently verify final diff against this plan; resolve contradictions centrally.
6. Remove all temporary `PLAN*.md` once implemented; commit and push validated framework code only (no planning files), no force push; report SHA, test matrix and remaining limitations.

**Success:** Framework supports a generic Swift-free UIKit application with scene/window ownership, custom rendering/input, prompts, local-file export and sandbox backup-exclusion, while maintaining its strict low-level abstraction boundary.
