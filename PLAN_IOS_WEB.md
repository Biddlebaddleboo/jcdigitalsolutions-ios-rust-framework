# PLAN_IOS_WEB.md — Workstream B33: Bounded iOS WKWebView Backend

## Objective

Create one native iOS `WKWebView` for an explicitly supplied HTTPS URL, with a narrow navigation
control surface. Do not implement SafariServices, a browser UI, arbitrary file access, JavaScript
bridges, or full browser parity.

## Dependencies

- D28 `framework-web` and its `HttpsUrl`/`WebView` contract are integrated
- `ios-runtime::main_thread::MainThread` is integrated
- Public iOS SDK declarations expose `WKWebView`, request loading, history controls, and navigation
  action/response policy callbacks
- `objc2-web-kit` 0.3.2 is available; its generated iOS class/delegate gaps are handled only with
  the exact typed objc2 declarations documented in `docs/ios/web.md`

## Write scope

- `platform/ios/ios-web/**`
- `docs/ios/web.md`

Do not edit the root workspace/dependency declarations, `Cargo.lock`, canonical capability matrix,
aggregate plans/indexes, CI, portable contract, other iOS crates, C bindings, SafariServices, or
Swift source. The orchestrator owns lockfile and shared-matrix reconciliation.

## Backend requirements

- Require `MainThread` to construct, attach, operate, and drop the view.
- Accept only one initial `framework_web::HttpsUrl<'_>`; preserve its caller text through the
  Foundation conversion and reject NSURLs without HTTPS scheme and a nonempty host.
- Use a typed Objective-C adapter around `WKWebView`; retain the weak `navigationDelegate` for the
  full wrapper lifetime and detach/stop/clear it at drop.
- Install action and response policy callbacks that allow HTTPS URLs with a nonempty host and
  cancel other schemes and new-window actions. Do not claim complete transport or subresource
  enforcement, same-origin confinement, or that a rejected response sends no request.
- Expose only view embedding plus back/forward state, back, forward, reload, and stop. Do not expose
  the native `WKWebView` object or APIs for arbitrary URL/file/HTML/data loading.
- Do not install `WKUIDelegate`, evaluate JavaScript, create a `WKUserContentController`, or add a
  script-message handler.
- Document default persistent website-data behavior, JavaScript/page behavior, privacy caveats,
  no permission claim, no load-result guarantee, native binding gap, and no browser parity.
- Derive the iOS 8.0 declaration floor from SDK headers separately from installed SDK deployment
  suggestions; do not set a crate deployment target.

## Validation and handoff

- Run device and simulator `cargo check` and strict all-target Clippy for `ios-web`.
- Run portable `framework-web` tests and checks, formatting, docs-check, zero-Swift-source, scoped
  forbidden-API gate, and `git diff --check`.
- Do not launch an app, display a live page, or send a network request.
- Report exact changed files, API floor and deployment-target distinction, dependency/features,
  commands/results, binding-shim rationale, privacy/runtime limits, deviations, and assumptions.
  Do not push.

## Status

Implemented in isolated worktree `/Users/john/Projects/.worktrees/jcdig-webkit-d28` on
`workstream/capabilities-webkit-d28`, based on `main` HEAD `7b5513fa2a4864d21a594cbf1fbd43951427155d`.
The adapter uses exact `objc2-web-kit = 0.3.2` with default features disabled and only
`WKFrameInfo`, `WKNavigation`, `WKNavigationAction`, `WKNavigationResponse`, and
`WKWebViewConfiguration`. A local typed objc2 class and protocol shim covers the missing iOS
`WKWebView` and policy-selector bindings; no direct `objc_msgSend` or runtime check was added.

Passed device/simulator `cargo check` and strict all-target Clippy through
`platform/ios/ios-web/check.sh`; `platform/ios/ios-web/check-surface.sh`; device-target rustdoc;
format, docs-check, zero-Swift-source, and diff checks. No app, UI, page, or network action was run.

The declarations and docs identify iOS 8.0 from Xcode 26.6 (build 17F113), iOS SDK 26.5 headers;
SDK deployment-target suggestions start at 12.0 and default to 26.5, with no crate target set.
Runtime page loading, content privacy behavior, permission UI, subresource filtering, and recipient
behavior are unverified and explicitly outside the claim. Root lockfile/canonical-matrix integration
remain orchestrator-owned; the worktree lock will be restored before handoff.
