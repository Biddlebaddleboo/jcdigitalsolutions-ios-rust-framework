# PLAN_IOS_URL.md — Workstream B20: Strict Foundation URL Value

## Status

B20 source, docs, and local compile/link gates pass on Xcode 26.6 build 17F113, iOS SDK 26.5, and
Rust 1.94.1. Device and Simulator link probes both report minos 17.0. No probe execution, native
URL acceptance, runtime availability, URL-open behavior, parser parity, or performance was checked

## Objective

Add a general iOS adapter for an absolute portable `Uri` that Foundation accepts as a strict RFC
3986 URL. Keep the exact portable source text authoritative and retain an owned `NSURL` for native
APIs that require one

## Dependencies

- D8 `framework-format::Uri` borrows and preserves exact RFC 3986 URI text
- B11 `ios-browser` uses a narrower external HTTPS URL-handler path; B20 does not change B11's
  completion or URL-open behavior
- The installed Xcode 26.6 / iOS SDK 26.5 marks
  `NSURL URLWithString:encodingInvalidCharacters:` as available on iOS 17.0 and later
- Apple documents link-time-dependent `NSURL` parser behavior around iOS 17; the strict initializer
  with `encodingInvalidCharacters: false` rejects invalid input rather than trying to encode it

## Write scope

- `PLAN_IOS_URL.md`
- `platform/ios/ios-url/**`
- `docs/ios/url.md`

Root owns workspace/lockfile reconciliation, CI and validation workstream G14, capability-manifest
and status updates, shared indexes, and aggregate progress docs. Do not edit those shared files or
the existing `ios-browser` implementation

## API and behavior

- Add an `ios-url` package that depends on `framework-format` and the existing workspace
  `objc2` / `objc2-foundation` bindings with only the required `NSString` and `NSURL` features
- Add an owner such as `NativeUrl<'uri>` that holds the source `Uri<'uri>` and
  `Retained<NSURL>`
- Provide a fallible constructor that passes the exact `Uri::as_str()` text to
  `NSURL::URLWithString_encodingInvalidCharacters` with `false`; map a `nil` result to a documented
  `FoundationRejectedUri` error
- Provide borrowed `&NSURL` and original-source `Uri<'uri>` accessors. Do not expose a raw pointer,
  transfer ownership, normalize, percent-decode, resolve references, or infer URL equivalence
- Do not call the older `URLWithString:` initializer as a fallback: apps linked on or after iOS 17
  may get different parsing and automatic percent/IDNA encoding behavior
- Declare and document iOS 17.0 as the API floor for this crate. The application must use a
  deployment target that honors that floor; compile/link success does not prove runtime availability
- Keep the adapter synchronous, caller-owned, and free of network requests, UIKit, global state,
  required executor, `Send` bounds, Swift source, or a new third-party dependency

## Boundaries and evidence

- A portable `Uri` is syntactically valid under D8's RFC 3986 parser, but Foundation may still
  reject a value; construction is fallible and this workstream does not claim every `Uri` maps
- The `Uri` retained in `NativeUrl` remains the exact source value. Do not claim that Foundation's
  parsed components, `absoluteString`, or downstream behavior are identical for every URI
- Do not add `UriReference`, file URL/resource access, URL normalization, URL opening, reachability,
  Safari/WebKit, or networking behavior
- Require only the Foundation framework; no permission, entitlement, or Info.plist key is needed
- Compile, strict Clippy, linked imports, and deployment metadata are not runtime/parser-parity or
  performance evidence

## Apple API basis

The installed SDK header `System/Library/Frameworks/Foundation.framework/Headers/NSURL.h` marks
`URLWithString:encodingInvalidCharacters:` as iOS 17.0+. Apple documents that setting
`encodingInvalidCharacters` to false leaves input unencoded and returns `nil` for explicitly
invalid URL text; it also documents parser behavior changes for apps linked on or after iOS 17.
See Apple's [`NSURL URLWithString:encodingInvalidCharacters:`](https://developer.apple.com/documentation/foundation/nsurl/urlwithstring%3Aencodinginvalidcharacters%3A?changes=latest_b__1_6&language=objc),
[`NSURL URLWithString:`](https://developer.apple.com/documentation/foundation/nsurl/urlwithstring%3A?changes=_2_1&language=objc),
and [NSURL parser behavior](https://developer.apple.com/documentation/foundation/nsurl/urlwithstring%3Arelativetourl%3A?changes=_1_8&language=objc)

## Validation and handoff

- Run package format/check/strict Clippy/rustdoc, device and Simulator check/strict Clippy, and a
  build-only import probe for Foundation, libSystem, and required Objective-C runtime imports
- Verify link-probe deployment metadata is iOS 17.0 or later on both Apple targets
- Run host workspace Clippy/tests, `cargo xtask docs-check`, `cargo xtask zero-swift-source`, YAML
  parsing, and `git diff --check` after root integration
- Do not run a linked probe or claim native URL acceptance, URL-open behavior, parity, or
  performance without an executable iOS 17+ runtime and explicit evidence
- Report exact API names, API floor, linked imports, checks, and unverified runtime assumptions

## Validation record

The inspected SDK header at
`/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/Foundation.framework/Headers/NSURL.h`
declares `+[NSURL URLWithString:encodingInvalidCharacters:]` with `API_AVAILABLE(ios(17.0))`
The crate uses `objc2-foundation` 0.3.2's `NSURL::URLWithString_encodingInvalidCharacters(&NSString, bool)` and passes `false`; it does not use the older initializer. The owner retains `Uri<'uri>` and `Retained<NSURL>`, and exposes only borrowed accessors

On Rust 1.94.1, these commands passed

- `cargo +1.94.1 fmt --manifest-path platform/ios/ios-url/Cargo.toml -- --check`
- `cargo +1.94.1 check --locked --offline -p ios-url`
- `cargo +1.94.1 check --locked --offline -p ios-url --target aarch64-apple-ios`
- `cargo +1.94.1 check --locked --offline -p ios-url --target aarch64-apple-ios-sim`
- `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-url -- -D warnings`
- `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-url --target aarch64-apple-ios -- -D warnings`
- `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-url --target aarch64-apple-ios-sim -- -D warnings`
- `cargo +1.94.1 doc --locked --offline -p ios-url --no-deps`
- `sh platform/ios/ios-url/check-link-imports.sh`
- `sh -n platform/ios/ios-url/check-link-imports.sh`
- `cargo +1.94.1 --locked --offline xtask docs-check`
- `cargo +1.94.1 --locked --offline xtask zero-swift-source`
- `git diff --check`

The device and Simulator probes link only `Foundation`, `libSystem.B.dylib`, and `libobjc.A.dylib` as direct libraries. `nm -u` shows `_objc_alloc`, `_objc_getClass`, and `_objc_msgSend`; the binary strings include `NSString`, `NSURL`, `initWithBytes:length:encoding:`, and `URLWithString:encodingInvalidCharacters:`. The script rejects the exact legacy `URLWithString:` selector and unrelated Swift/Python, Security, network, WebKit, UIKit, Safari, and URL loading symbols. `vtool` reports minos 17.0 and SDK 26.5 for both device and Simulator. Neither probe was executed; no Foundation parser result, app use, URL-open result, runtime availability, parity, or performance was verified
