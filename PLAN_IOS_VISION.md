# B57 — iOS Vision text-revision support query

## Objective

Expose only `VNRecognizeTextRequest.supportedRevisions` membership as an iOS query that returns a
portable, owned `TextRecognitionRevisionSupport` result. Do not create or perform a Vision request.

## Dependencies and API evidence

- D52 defines `framework-vision::TextRecognitionRevisionSupport`.
- Apple documents `VNRequest.supportedRevisions` as a runtime query for currently supported
  algorithm revisions for each request class.
- Xcode 26.6 build 17F113 / iPhoneOS SDK 26.5 marks `VNRequest.supportedRevisions` available from
  iOS 12.0 and `VNRecognizeTextRequest` available from iOS 13.0. The effective API floor is 13.0.
- `objc2-vision` 0.3.2 binds `VNRequest::supportedRevisions() -> Retained<NSIndexSet>` and
  `VNRecognizeTextRequest`. The generated class method is attached to `VNRequest`; the backend
  sends the same typed selector to the concrete `VNRecognizeTextRequest` class receiver so the
  runtime answers for that request class. `objc2-foundation` 0.3.2 binds safe
  `NSIndexSet::containsIndex`.

## Write scope

- `platform/ios/ios-vision/**`
- this plan and `PLAN_CAPABILITIES_VISION.md`

The orchestrator owns global capability status, shared root documentation, CI, and indexes. Do not
edit those surfaces, root Cargo manifests, unrelated backend crates, Swift sources, or tests.
Cargo.lock in this isolated worktree is local integration evidence and must be reconciled by the
orchestrator.

## API and bounds

- `ios_vision::text_recognition_revision_support(revision: u32) ->
  Option<framework_vision::TextRecognitionRevisionSupport>`.
- iOS API floor: 13.0, guarded with `objc2::available!(ios = 13.0, ..)`; non-iOS returns `None`.
- Call only the `supportedRevisions` class property on `VNRecognizeTextRequest`, then query
  membership in its returned `NSIndexSet`.
- Framework: `Vision`; typed binding `objc2-vision` 0.3.2, defaults disabled, features exactly
  `VNRequest` and `VNRecognizeTextRequest`; `objc2-foundation` feature `NSIndexSet`.
- No request instance, request handler, image or video input, external data access, camera,
  permission request, Info.plist usage string, entitlement, UI, recognition, model-readiness claim,
  native handle, or Swift ABI.
- A `true` result means only that the runtime lists that revision; it does not mean any image will
  be recognized successfully or that model resources are ready.

## Gates

- `sh platform/ios/ios-vision/check.sh`: format, portable no-std/host checks, strict Clippy,
  device/Simulator checks and strict Clippy, docs, and the Vision feature-surface gate.
- `sh platform/ios/ios-vision/check-link-imports.sh`: build-only device and Simulator Release links;
  assert the direct import set, the request class and selector, target `minos`, and absence of
  request-handler, image, camera, UI, Core ML, and Swift imports. The linked artifacts are never
  executed.
- Root integration places `objc2-vision` in the central workspace dependency table, updates row 064
  to `B`/partial, adds the `ios-vision` CI gate, and indexes the portable and iOS guides.

## Status and evidence

Status: bounded revision-membership query implemented; row 064 remains partial Vision support.
Apple primary docs: [VNRequest.supportedRevisions](https://developer.apple.com/documentation/vision/vnrequest/supportedrevisions)
and [VNRecognizeTextRequest](https://developer.apple.com/documentation/vision/vnrecognizetextrequest).
Pinned generated bindings: [objc2-vision 0.3.2 `VNRequest`](https://docs.rs/objc2-vision/0.3.2/objc2_vision/struct.VNRequest.html),
[`VNRecognizeTextRequest`](https://docs.rs/objc2-vision/0.3.2/objc2_vision/struct.VNRecognizeTextRequest.html),
and [`NSIndexSet`](https://docs.rs/objc2-foundation/0.3.2/objc2_foundation/struct.NSIndexSet.html).

Local SDK: Xcode 26.6 build 17F113, iPhoneOS SDK 26.5.

Passed commands:

- `cargo check -p ios-vision --lib` (initial isolated lock refresh; added `objc2-vision` 0.3.2).
- `sh platform/ios/ios-vision/check.sh`: portable no-std check, strict portable Clippy, portable
  rustdoc, host backend check and strict Clippy, backend rustdoc, iOS device and Simulator checks
  and strict Clippy, feature gate, and build-only link/import gate.
- `sh platform/ios/ios-vision/check-link-imports.sh`.
- `sh -n platform/ios/ios-vision/check.sh` and
  `sh -n platform/ios/ios-vision/check-link-imports.sh`.
- `git diff --check`.

The Release link gate passed for `aarch64-apple-ios` and `aarch64-apple-ios-sim`. Both artifacts
import only `Vision`, `Foundation`, `libobjc.A.dylib`, and `libSystem.B.dylib`. `nm` confirmed
`_objc_msgSend` and no request-handler, image, camera, UI, Core ML, or Swift imports; `strings`
confirmed `VNRecognizeTextRequest`, `supportedRevisions`, and `containsIndex`, with no forbidden
request-handler, image, camera, UI, Core ML, or Swift selectors. `vtool -show-build` reported
`minos 13.0` for the device artifact and `minos 14.0` for the Simulator artifact. These build
artifacts were inspected but never executed. No tests or Vision runtime query were run.
