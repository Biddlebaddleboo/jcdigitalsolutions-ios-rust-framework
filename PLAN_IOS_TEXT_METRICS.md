# PLAN_IOS_TEXT_METRICS.md — Workstream B23: CoreText System Font Metrics

## Status

B23 adds `ios_ui::text_metrics::system_font_metrics` and implements the D18 backend trait with
public CoreText C functions. Source, target builds, strict Clippy, and a Release symbol/import probe
pass on Xcode 26.6 / iOS SDK 26.5. No native call was run; this is compile/link evidence only

## Objective

Bridge the portable D18 query to `CTFontCreateUIFontForLanguage`, `CTFontGetAscent`,
`CTFontGetDescent`, and `CTFontGetLeading` without Swift source or a Swift runtime

## Dependencies and API path

- D18 `framework-ui::FontMetrics` and `TextMetricsBackend`
- Existing `ios-ui` package, `objc2-core-foundation` `CGFloat`, `ios-runtime` main-thread proof
- Public C declarations in Xcode 26.5 SDK `CoreText.framework/Headers/CTFont.h`
- `CTFontUIFontType` is `uint32_t`; `kCTFontUIFontSystem` is `2`; a null language requests the
  current system language
- These functions have an SDK availability floor of iOS 3.2. Probe deployment targets of iOS 12.0
  device and iOS 14.0 Simulator are validation settings, not the product minimum
- The checkout has no pinned `objc2-core-text` package or CoreText binding. Keep private FFI
  declarations source-matched to the SDK headers and validate them with the C fixture; do not add a
  workspace dependency or hand-defined public ABI

## Write scope

- `platform/ios/ios-ui/src/lib.rs`
- `platform/ios/ios-ui/src/text_metrics.rs`
- `platform/ios/ios-ui/examples/ios_ui_text_metrics_link_import_probe.rs`
- `platform/ios/ios-ui/probes/text-metrics-layout.c`
- `platform/ios/ios-ui/check-text-metrics-link-imports.sh`
- `docs/ios/text-metrics.md`
- `PLAN_IOS_TEXT_METRICS.md`
- `PLAN_VALIDATION_IOS_TEXT_METRICS.md`

Do not edit root manifests, `Cargo.lock`, capability matrices, shared indexes, CI, or the shared
validation plan

## Runtime boundaries

- Require the iOS main thread, matching `ios-ui`; return `InvalidInput` for a non-finite or
  non-positive point size, `Unavailable` without the main-thread proof, and `Platform` for a null
  font or invalid native metrics
- Release the create-rule `CTFontRef` exactly once with `CFRelease`
- Metrics can vary by OS, installed system font, point-size precision, and system language. Do not
  claim parity with `UILabel`, a text style, Dynamic Type, or any Rust shaping/layout engine
- Do not add custom-font selection, strings, shaping, glyph advances, widths, line breaks, or view
  mutation
- Do not add Swift source

## Acceptance

- Portable D18 tests, no-std compile, rustdoc, and strict Clippy pass
- iOS device and Simulator target compile and strict Clippy pass
- C fixture confirms `CGFloat` and enum layout and compiles exact public function prototypes
- Release link/import probe accepts only `CoreText`, `CoreFoundation`, and `libSystem.B.dylib`; it
  checks all four CoreText calls and `CFRelease`, plus deployment target and no Swift/Python runtime
- Report compile/link only; do not call a build probe runtime parity
