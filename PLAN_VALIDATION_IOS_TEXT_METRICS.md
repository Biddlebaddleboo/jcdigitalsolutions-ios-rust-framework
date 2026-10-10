# PLAN_VALIDATION_IOS_TEXT_METRICS.md — Workstream G17: CoreText Metrics Gates

## Status

G17's shared `ios-ui` device/Simulator target and strict Clippy gates plus the focused link/import
script are wired in CI and pass locally for B23 on Rust 1.94.1, Xcode 26.6 build 17F113, and iOS
SDK 26.5. The probe is not run and does not establish runtime font metrics or UIKit parity. No
passing CI workflow run is recorded

## Scope

- Validate D18 `framework-ui` and B23 `ios-ui` only
- Keep root CI, workspace manifests, `Cargo.lock`, shared validation docs, and capability indexes
  under root ownership
- Use device and Simulator Release builds for `ios_ui_text_metrics_link_import_probe`
- Require direct loads to equal `CoreText`, `CoreFoundation`, and `libSystem.B.dylib`
- Require undefined symbols `_CTFontCreateUIFontForLanguage`, `_CTFontGetAscent`,
  `_CTFontGetDescent`, `_CTFontGetLeading`, and `_CFRelease`
- C-compile `text-metrics-layout.c` against the active SDK headers; assert `sizeof(CGFloat) == 8`,
  `_Alignof(CGFloat) == 8`, and `sizeof(CTFontUIFontType) == 4`
- Verify target deployment metadata matches the probe settings and reject Swift/Python imports or
  source under `platform/ios/ios-ui`

## Limits

- Target checks prove compilation, symbol resolution, and load-command scope only
- Do not execute CoreText on a Simulator or device, claim exact font output, or infer native parity
  from host fixture values

## 2026-10-10 current-main recheck

At source tree `4cf1a796a6dbb6a5f6fbaa4424fbb298e8ba0762`, Rust 1.94.1, Xcode 26.6 build
17F113, and iOS SDK 26.5:

- PASS: `cargo +1.94.1 check --locked -p ios-ui --target aarch64-apple-ios` and the matching
  `aarch64-apple-ios-sim` check
- PASS: strict all-target Clippy for both targets with `-D warnings`
- PASS: `sh platform/ios/ios-ui/check-text-metrics-link-imports.sh`. The C SDK-header assertions
  passed for `sizeof(CGFloat) == 8`, `_Alignof(CGFloat) == 8`, `sizeof(CTFontUIFontType) == 4`,
  and the five declared function signatures. Both Release probes import exactly CoreFoundation,
  CoreText, and `libSystem.B.dylib`; both retain `_CTFontCreateUIFontForLanguage`,
  `_CTFontGetAscent`, `_CTFontGetDescent`, `_CTFontGetLeading`, and `_CFRelease`. Device minimum
  OS is 12.0; Simulator minimum OS is 14.0; both use SDK 26.5. Swift/Python runtime and source
  guards passed.

No passing CI workflow run is recorded. The probe executables were not run; these results do not
establish runtime font metrics, exact font output, or UIKit parity.
