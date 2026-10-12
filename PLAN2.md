# PLAN2.md — Prioritize low-level UIKit and MapKit native bindings

## Status and priority
**Approved priority handoff, temporary file.** Execute ahead of lower-priority independent Apple capability expansion, but do not interrupt or overwrite in-flight work unsafely. Repository: `jcdigitalsolutions/jcdigitalsolutions-ios-rust-framework`, target `main`, reviewed at `037f0fd373d579ac0e7077c078771825032ec301`. **Verify the newest remote `main` before editing or integrating.** Read active root `PLAN.md` and MapKit capability/validation plans; reconcile overlapping tasks centrally and preserve unrelated ongoing work. This plan scopes one coupled workstream, not multiple executors.

## Objective
Expose Apple's **existing low-level native UIKit and MapKit APIs** faithfully through the Rust iOS framework so external Rust user-interface engines can embed and operate `MKMapView`. This framework remains a low-level native API layer. Application-specific map behavior and all `micro-ui` integration are out of scope.

## Verified repository facts
- `platform/ios/ios-map-view/src/lib.rs` exports `IosMapView` and `NativeMapView` on iOS; `platform/ios/ios-map-view/README.md` documents a main-thread-bound native `MKMapView`, borrowed UIKit view, region controls, marker snapshots, **one** fixed-style route polyline with a 4,096-point wrapper bound, one polygon, and opt-in foreground user tracking.
- `platform/ios/ios-maps/src/lib.rs` supplies coordinate projection/distance calls; `framework-maps` holds portable geometry.
- `docs/ios/mapkit.md`, `PLAN_CAPABILITIES_MAPKIT.md`, and `PLAN_VALIDATION_IOS_MAPKIT.md` identify the existing narrow binding slice, the iOS `objc2-map-kit 0.3.2` generated `MKMapView` availability limitation and the locally declared `objc2` iOS class/delegate ABI. Preserve validated ABI and existing APIs.
- `micro-ui` owns native-child-surface composition externally; do **not** import it here.

## Implementation scope: inspect these exact starting points
1. `platform/ios/ios-map-view/src/platform.rs` (or its platform module files), `src/route.rs`, `src/annotations.rs`, `src/polygon.rs`, `src/tracking.rs` and the public `src/lib.rs`.
2. `platform/ios/ios-map-view/README.md`, its `Cargo.toml`, associated focused tests/check scripts; `platform/ios/ios-maps/src/lib.rs`, and relevant `objc2` declarations.
3. Existing UIKit native-view APIs in the framework, at exact symbols discovered from the iOS view host/borrow surface; examine only necessary dependencies for native child-view composition.
4. Existing `PLAN_CAPABILITIES_MAPKIT.md`, `PLAN_VALIDATION_IOS_MAPKIT.md`, `PLAN.md`, and the framework-local validation spec for the map-view package. If symbols moved, expand search narrowly.

## Required low-level work
- Audit actual existing `objc2` coverage and public Apple SDK declarations before writing wrappers. Reuse/re-export correct upstream native bindings wherever possible; supply narrow missing iOS declarations only where absent or unusable, with ABI evidence.
- Expose necessary native operations for `MKMapView`, `MKPolyline`, `MKPolylineRenderer`, `MKOverlay`, `MKAnnotation`, `MKMapCamera` and `MKMapViewDelegate` where currently inaccessible. Include native overlay add/remove, renderer line properties (including dash configuration if supported), viewport/map-rect/camera methods, annotation selection and region callbacks, without inventing an application-oriented route model.
- Ensure UIKit-native `UIView` parenting, frame/bounds, layout, visibility, hit-testing, and teardown are accessible to consumers where the existing framework lacks the necessary primitives. Let the external host own layout and view hierarchy. Prefer a safe borrowed native view and native UIKit methods to a new framework-wide composition runtime.
- Enforce native main-thread affinity; preserve Objective-C retain/release, weak delegate slots, delegate lifetime, callback reentrancy and destruction safety. No callback after its Rust owner is invalid. Preserve interoperability when an external host owns an existing delegate; reject unsafe contention rather than overwrite its slot silently.
- Keep the public API as close as practical to Apple's native object/protocol model, with minimal runtime overhead, avoid duplicate caches and redundant copies. When a wrapper's old one-route/4,096-point interface cannot serve the low-level API use case, **add native access without breaking existing callers** unless an explicitly approved migration permits changes.

## Non-goals
No stable application route IDs, route simplification or chunking policy, GPS tracking, mileage calculations, Google Routes, map search, custom UI renderer, `micro-ui` dependency, cloud backup, or new high-level MapKit convenience layer. Do not duplicate an existing `objc2` binding solely for naming preferences. No changes to the private tooling repository or its releases/pins.

## Deterministic tests and acceptance
- Validate native object ownership, delegate slot occupation/release, callback teardown/reentrancy, multiple simultaneous independent native overlays, native style property propagation, annotations, camera/region operations and parent/child view lifecycle. Use focused fake/test-double tests where appropriate; tests must not depend on sleeps or network tiles.
- Build and link Apple SDK device and Simulator probes for any added Objective-C selectors, types, methods and protocol callback ABIs. Run framework-local mapped test/check commands and applicable installed validation gates; do not weaken checks. Report exact commands, target/toolchain, PASS/FAIL/SKIPPED/BLOCKED and whether physical-device map display/touch/rotation was actually observed; compile/link success is not runtime proof.
- Check minimal diffs, no Swift or custom rendering requirement, no unapproved high-level API, no accidental breakage to existing route/polygon/annotation delegates, and no leaked native objects.

## Repository isolation and documentation
Framework executor must **not clone, search, inspect, or otherwise access the private tooling repository** (including via subagents/connectors). Use only pinned installed tools and local CLI documentation. **ChatGPT owns cross-repository documentation and tooling-release coordination**. Report necessary `AGENTS.md`, `docs/SHARED_TOOLING.md`, or `docs/VALIDATION.md` changes to ChatGPT rather than editing these files.

## Completion and deletion
After implementation is integrated, focused validation is reported, and the final diff is independently reviewed, **delete this temporary `PLAN2.md` from the framework repository and include its deletion in the final implementation cleanup commit**. Do not delete active `PLAN.md` or other `PLAN_*.md` files owned by separate workstreams. Report implementation commit SHA, files changed, checks and unresolved gaps. Do not claim completion if native SDK probes or required interfaces remain unverified; retain or explicitly escalate the unfinished plan until resolved.
