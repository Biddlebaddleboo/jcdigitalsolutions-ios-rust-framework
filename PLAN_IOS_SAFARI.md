# PLAN_IOS_SAFARI.md — Workstream B64: In-App Safari HTTPS Presentation

## Status

Implementation and scoped compile/link validation are complete in the isolated worktree and root
checkout. This is an iOS-only SafariServices view-controller wrapper; it is distinct from B11's
external URL-handler request and B33's WKWebView. Root integration adds B64 to row 030 without
changing aggregate support counts

## Objective

Create one public `SFSafariViewController` for an explicitly supplied HTTPS URI, and let the host
present it from its own UIKit controller. Do not implement browser controls, web-content access,
or an alternate web engine

## Dependencies and API evidence

- Reuse `framework-format::Uri` for exact caller-borrowed URI text and `ios-runtime::MainThread`
  for UIKit thread affinity; add no portable web contract
- The inspected public iOS 26.5 SDK header declares `SFSafariViewController` and
  `initWithURL:` available from iOS 9.0. Rust 1.94.1's arm64 device target supports a minimum
  deployment target of iOS 10.0, so the link gate uses 10.0 for device and 14.0 for Simulator.
  The crate sets no deployment target. The declaration was verified in
  `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/SafariServices.framework/Headers/SFSafariViewController.h`
- The URL initializer supports HTTP and HTTPS. This slice accepts only HTTPS with a nonempty
  authority and a Foundation URL host
- No generated `objc2-safari-services` bindings are present in the local Cargo source cache. The
  adapter uses a typed objc2 class declaration for only the public `SFSafariViewController`
  superclass and `initWithURL:` selector, matching the inspected header. It does not use raw
  `objc_msgSend` or hand-declare a C ABI function
- No new workspace dependency is required; the adapter uses existing `objc2`,
  `objc2-foundation`, and `objc2-ui-kit` bindings with `UIResponder` and `UIViewController`
  features. It has no `block2` dependency
- The crate-local `build.rs` links the public `SafariServices` framework for iOS targets; typed
  Objective-C class names alone do not cause the framework to enter the Mach-O dependency list

## Write scope

- `PLAN_IOS_SAFARI.md`
- `PLAN_VALIDATION_IOS_SAFARI.md`
- `platform/ios/ios-safari/**`
- `docs/ios/safari.md`

Do not edit the portable URI/web contracts, B11 `ios-browser`, B33 `ios-web`, root capability
manifest, root plan/index files, shared CI, C bindings, or other backend crates. The orchestrator
owns shared workspace and capability-status reconciliation

## API and ownership

- `IosSafariViewController::new(MainThread, Uri)` rejects non-HTTPS schemes, missing/empty
  authority, Foundation URL construction failure, and a missing/empty Foundation host
- `view_controller(&self)` returns only a borrowed `UIViewController` for host-managed UIKit
  presentation. The host supplies its presenter and controls presentation/dismissal lifecycle
- The wrapper retains its Safari controller and a `MainThreadMarker`; it is `!Send`/`!Sync` and
  must be created, accessed, and dropped on the main thread
- No Safari delegate is installed and no callback crosses into Rust. The app's existing UIKit
  controller owns presentation and dismissal. The wrapper does not dismiss or cancel on drop
- `Ok` means only that a native controller was constructed. It does not prove presentation,
  request start, page load, network success, or visible content

## Privacy and non-goals

- SafariServices owns the browser chrome, website rendering, transport, cookies, and navigation;
  the wrapper cannot inspect page contents, navigation history, request/response headers, or
  cookie state
- Do not add `WKWebView`, JavaScript, custom navigation delegates, reader mode, custom activity
  buttons, connection prewarming, downloads, authentication UI, external URL opening, or
  `canOpenURL`
- Do not claim a privacy prompt, ATS exception, local-network access, extension availability,
  Safari app routing, or cookie-sharing behavior without separate current evidence
- No live URL, UI, network, or page action is run by validation

## Validation and handoff

- Check and strict-Clippy `ios-safari` for arm64 iOS device and Simulator
- Link minimal Release consumers without executing them; check exact direct imports, the
  `SFSafariViewController` class/`initWithURL:` metadata and Objective-C lookup/message symbols,
  reject Swift/Python runtimes and unrelated
  URLSession, WebKit, and Network.framework imports
- Keep the example link probe's imports and API use behind `target_os = "ios"` and provide a host
  no-op `main`, so workspace all-target Clippy can compile the probe package on macOS
- Use `vtool -show-build` to verify the device link's iOS 10.0 minimum and the Simulator link's
  iOS 14.0 minimum
- Run formatting, rustdoc, docs-check, zero-Swift-source, and `git diff --check`; do not add or run
  tests for this follow-up
- Report exact changed paths, lockfile delta, imports, availability evidence, and the limits above.
  Compile/link evidence does not establish Safari runtime behavior or browser parity
