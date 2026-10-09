# PLAN_BINDINGS_F17.md — F17: iOS Vision revision membership C ABI

## Status

The F17 surface is integrated. `sh bindings/c/check-ios-vision.sh` passed host, iOS device, and Simulator feature isolation, strict Clippy, Release builds, C11/C++17 links, export/import checks, and deployment minimums (13.0 device, 14.0 Simulator). Its static gate also asserts aligned writable output storage for the full call, caller protection from unsynchronized access, and no pointer retention. The first gate attempt found the check script used `status_mapping.valid_result` while the manifest key is `status_mapping.success`; the assertion was corrected and the full gate then passed. Link probes were not executed; no tests ran

## Candidate and feasibility

F1–F16 do not expose a Vision API through C. D52/B57 already provide `framework_vision::TextRecognitionRevisionSupport` and `ios_vision::text_recognition_revision_support(revision: u32)`. The iOS backend returns an owned portable value or `None`; it does not create a request or require image, user-data, permission, UI, or callback state. A one-function scalar C facade is therefore narrower than wrapping a Vision object or request lifecycle

Apple defines `VNRequest.supportedRevisions` as the currently supported algorithm versions for the request class and says the property lets clients inspect runtime capabilities. `VNRecognizeTextRequest` is the concrete text-recognition class. B57 records the effective iOS API floor as 13.0, because this request class is available from iOS 13.0 even though the inherited property has an earlier floor. The existing typed backend uses `objc2-vision` 0.3.2 with defaults off, `VNRequest` and `VNRecognizeTextRequest` only, plus `objc2-foundation`'s `NSIndexSet` feature

Primary sources: [Apple `VNRequest.supportedRevisions`](https://developer.apple.com/documentation/vision/vnrequest/supportedrevisions), [Apple `VNRecognizeTextRequest`](https://developer.apple.com/documentation/vision/vnrecognizetextrequest), and the repository's [B57 plan](PLAN_IOS_VISION.md)

## Exact proposed ABI

- Cargo feature: `ios-vision`, enabled only by optional target-iOS dependency `ios-vision`
- Header: `framework_ios_vision.h`
- Export: `FrameworkStatus framework_ios_vision_text_recognition_revision_is_supported(uint32_t revision, uint8_t *out_supported)`
- Query: call `ios_vision::text_recognition_revision_support(revision)`, which sends `+[VNRecognizeTextRequest supportedRevisions]` and checks `NSIndexSet::containsIndex`. Do not create `VNRecognizeTextRequest`, a request handler, or any Vision object through the C API
- Output: required caller-owned valid, properly aligned writable `uint8_t` memory for the full synchronous call; caller prevents unsynchronized access; initialize to 0 before the query, write exactly 0 or 1 only on `FRAMEWORK_STATUS_OK`; retain no pointer and allocate no C-owned result
- Input: any `uint32_t` revision is valid; an unlisted or unknown revision is a successful false result
- Statuses: null output is `FRAMEWORK_STATUS_INVALID_ARGUMENT`; successful iOS query is `FRAMEWORK_STATUS_OK`; iOS below 13.0 is `FRAMEWORK_STATUS_UNAVAILABLE`; valid non-iOS call is `FRAMEWORK_STATUS_UNSUPPORTED`; caught Rust panic is `FRAMEWORK_STATUS_PANIC`. There is no native error result
- Lifetime/ownership: synchronous call; the C caller retains valid output storage through call return. The API does not prove memory validity or retain its address. No handle, callback, object, string, buffer, or native pointer crosses the ABI
- Thread: no main-thread rule is added; the synchronous call adds no thread-safety guarantee
- Limits: revision membership only; no recognition, image/data access, supported-language query, model readiness, or recognition-success promise

The ABI manifest entry uses key `ios_vision`, feature `ios-vision`, symbol array containing only `framework_ios_vision_text_recognition_revision_is_supported`, API text with iOS 13.0, and device/Simulator link minimums 13.0/14.0. `direct_imports_64_bit_ios.c` is `Vision`, `Foundation`, `libSystem.B.dylib`, and `libobjc.A.dylib`; C++ adds `libc++.1.dylib`. The `status_mapping` fields record invalid output, success 0/1, runtime unavailable, non-iOS unsupported, and panic. `ownership.output` records aligned writable caller memory for the full synchronous call, caller protection from unsynchronized access, and no proof of actual memory validity or retained address. No `FrameworkOwnedBuffer` creator is added

## Dedicated F17 files

- `bindings/c/src/ios_vision.rs`
- `bindings/c/include/framework_ios_vision.h`
- `bindings/c/check-ios-vision.sh`
- `docs/bindings/ios-vision.md`
- `PLAN_BINDINGS_F17.md`

Root integration is complete: the target-iOS dependency and feature, module and re-export gates, ABI manifest entry, Cargo.lock edge, F17 CI invocation, aggregate F17 status, and guide index link are present

## Focused gate outline

The focused gate is evidence: feature graphs, unsupported host behavior and import isolation, locked compilation, strict Clippy, Release builds, C11/C++17 host/device/Simulator links, one exported function, exact imports, expected selectors/strings, forbidden-symbol constraints, and deployment minimums all passed. Device link minos is 13.0 and Simulator minos is 14.0. Linked probes were not executed. No tests were run

## Existing evidence and remaining limits

B57's `sh platform/ios/ios-vision/check.sh` and `sh platform/ios/ios-vision/check-link-imports.sh` passed per `PLAN_IOS_VISION.md`. Its device and Simulator Release links import only Vision, Foundation, `libobjc.A.dylib`, and `libSystem.B.dylib`; the artifacts show minos 13.0 and 14.0 and were never executed. F17 adds the C ABI gate over this backend

F17 establishes only revision membership, not recognition or model readiness. No tests, Vision runtime query, device execution, or model readiness evidence exists
