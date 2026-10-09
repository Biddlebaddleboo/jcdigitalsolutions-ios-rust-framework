# PLAN_IOS_BROWSER.md — Workstream B11: External HTTPS URL Handler

## Status and evidence

B11's source and guide meet the bounded URL-handler contract: the backend accepts only an absolute HTTPS `Uri` with a nonempty Foundation host, preserves the caller text, and calls public `UIApplication.openURL:options:completionHandler:` with empty options. It exposes no UIKit handle and requires `MainThread`; success means only that a URL-handler request was issued. The guide records that callback `false` is not a page-load result, Universal Links may route elsewhere, and `Err` sends no request or callback. `UIApplication.sharedApplication` is unavailable to app extensions; no in-app browser or `LSApplicationQueriesSchemes` requirement is claimed.

The public header API floor is iOS 10.0 on Xcode 26.6 (build 17F113), iOS SDK 26.5. The installed SDK deployment minimum is separately recorded as iOS 12.0; the crate sets no deployment target. The local host is below the Xcode 27.x plan baseline.

G11's local evidence records pass for the locked device/simulator `cargo check` and strict all-target Clippy gates, `cargo xtask docs-check`, workflow YAML parsing, and `git diff --check`. Those checks establish compile/lint only. No backend-specific link/import probe, live URL-handler request, browser launch, network request, app runtime, or device/simulator behavior was tested.

## Objective

Add one bounded iOS system URL-handler path for caller-validated HTTPS URIs. This is an OS URL-handler request, not a guarantee that Safari opens, that a page loads, or that any visible browser UI appears.

## Dependencies

- D8 `framework-format::Uri` is integrated and preserves validated RFC 3986 URI text
- `ios-runtime::main_thread::MainThread` is integrated
- Public `objc2-ui-kit`, `objc2-foundation`, `objc2`, and `block2` bindings expose `UIApplication.openURL:options:completionHandler:`

## Read first

- `PLAN_IOS_NATIVE.md`
- `PLAN_CAPABILITIES_URI.md`
- `docs/ios/runtime.md`
- `docs/capabilities/uri.md`
- `docs/OBJC_INTEROP.md`
- `docs/OWNERSHIP.md`
- `docs/APP_STORE_COMPLIANCE.md`
- `docs/VALIDATION.md`

## Write scope

- `PLAN_IOS_BROWSER.md`
- `PLAN_IOS_NATIVE.md` — B11 link only
- `platform/ios/ios-browser/**`
- `docs/ios/browser.md`

Do not edit the root workspace/dependency declarations, `Cargo.lock`, capability manifest, CI, portable contracts, SafariServices, `canOpenURL`, other iOS backends, or Swift source. The orchestrator owns lockfile, capability-manifest, and validation-document integration.

## Required surface

- Add an independent `ios-browser` crate with one public API accepting `framework_format::Uri<'_>`, an `ios-runtime::main_thread::MainThread` proof, and a Rust completion callback.
- Accept only an absolute HTTPS URI with a nonempty native host. Reject other schemes and values that Foundation cannot construct as `NSURL`; do not normalize, percent-decode, resolve, or rewrite caller text.
- Call only public UIKit `UIApplication.openURL:options:completionHandler:` with empty options. Do not call `canOpenURL` or use SafariServices.
- Keep UIKit use and completion delivery main-thread-affine; require a `MainThread` proof and expose no UIKit handle.
- Document synchronous `Ok(())` as only an issued URL-handler request. The asynchronous completion bool reports UIKit's URL-open result; it does not prove a network request, page load, user view, or Safari selection. Universal Links and the system's URL-handler policy may route to another app.
- State that `Err` means UIKit receives no request and the completion does not run. Callback panic containment applies only with an unwind panic strategy; `panic=abort` ends the process on panic
- Record the declaration-derived iOS 10.0 API floor separately from the installed SDK deployment minimum. Do not set a crate deployment target.
- Document that `UIApplication.sharedApplication` is unavailable to app extensions, and that the crate does not implement an in-app Safari browser.
- State only verified Info.plist/entitlement requirements; no `LSApplicationQueriesSchemes` requirement applies to `open` when `canOpenURL` is not used.

## Validation and handoff

- Do not add or run tests. Do not simulate UIKit or call a live URL handler.
- Run device and simulator `cargo check`, strict all-target Clippy, formatting, docs-check, zero-Swift-source, and diff checks.
- Report exact check results, API floor evidence, deployment-target distinction, input semantics, callback limits, files, commit SHA, deviations, and unresolved assumptions. Do not push.
