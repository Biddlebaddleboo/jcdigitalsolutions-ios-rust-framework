# PLAN_VALIDATION_IOS_MEDIA.md — Workstream G16: iOS CoreMedia Time Gates

## Status

G16 CI wiring, locked device/Simulator checks, strict all-target Clippy, and the final B22
link/import/layout script pass locally on Rust 1.94.1, Xcode 26.6 build 17F113, and SDK 26.5. The
arm64 probes were linked but not executed. No CI workflow run or media runtime claim

## Objective

Keep persistent Apple-target compile, lint, import, symbol, and `CMTime` ABI layout checks for
`framework-media::MediaTime` and the B22 `ios-media::IosMediaTime` adapter

## Dependencies

- D17 `framework-media::MediaTime`; the root-owned portable `no_std` probe already covers this
  crate and its selected public API
- B22 `ios-media` with exact `objc2-core-media` 0.3.2 pin and only its `CMTime` feature
- Existing CI installation of `aarch64-apple-ios` and `aarch64-apple-ios-sim`

## Write scope

- `PLAN_VALIDATION_IOS_MEDIA.md`
- `.github/workflows/ci.yml`
- Concise G16 evidence in `PLAN_VALIDATION.md` and `docs/VALIDATION.md` after root clears those
  shared validation files

Do not edit B22 source, capability matrix/counts, shared indexes, or root `no_std`/xtask code

## Required gates

- Run locked `cargo check` for `ios-media` on device and Simulator
- Run strict all-target Clippy with `-D warnings` for both Apple targets
- Run `sh platform/ios/ios-media/check-link-imports.sh`; it must build but not execute the probe,
  compile the C11 layout fixture for both SDK targets, check exact imports, require `_CMTimeMake`,
  reject out-of-scope media/runtime symbols, and verify target minimum OS metadata
- Review the exact import set and symbol attribution; CoreMedia must provide `_CMTimeMake`, and
  libSystem must resolve the linked C/runtime imports. Reject unused framework load commands
- Verify C and Rust assertions for 24-byte `CMTime`, 4-byte alignment, and offsets 0, 8, 12, and
  16 for `value`, `timescale`, `flags`, and `epoch`
- Verify no Swift source/runtime or AVFoundation, capture, sample-buffer, or unrelated media API
  import; use the repository docs and zero-Swift checks
- Keep device/Simulator artifacts build-only. Do not claim live CoreMedia or AVFoundation use,
  playback, capture, runtime parity, or performance

## B22 interface review

The adapter maps `MediaTime` to `CMTime` by value using `CMTime::new(value, timescale)` from the
pinned crate. D17 guarantees a strictly positive timescale. The Xcode 26.6 SDK header marks
`CMTime` and `CMTimeMake` available from iOS 4.0; probe deployment targets of iOS 12.0 and
Simulator 14.0 are build settings, not a statement of the API's minimum availability

## G16 local results (2026-10-08)

These locked target commands passed with Rust 1.94.1:

```sh
cargo +1.94.1 check --locked -p ios-media --target aarch64-apple-ios
cargo +1.94.1 check --locked -p ios-media --target aarch64-apple-ios-sim
cargo +1.94.1 clippy --locked -p ios-media --all-targets --target aarch64-apple-ios -- -D warnings
cargo +1.94.1 clippy --locked -p ios-media --all-targets --target aarch64-apple-ios-sim -- -D warnings
sh platform/ios/ios-media/check-link-imports.sh
```

The final script uses `-Wl,-dead_strip_dylibs` and enforces exact direct imports `CoreMedia.framework` and `/usr/lib/libSystem.B.dylib`; it passes for both targets. `dyld_info -imports` maps `_CMTimeMake` to CoreMedia and every other imported symbol to libSystem. The earlier unstripped probe had an unused CoreFoundation load command and no imported symbols from CoreFoundation; the final link strips it. No Swift/Python or out-of-scope media symbols remain

The exact artifacts are `target/ios-media-link-aarch64-apple-ios/aarch64-apple-ios/release/examples/ios_media_link_import_probe` and `target/ios-media-link-aarch64-apple-ios-sim/aarch64-apple-ios-sim/release/examples/ios_media_link_import_probe`. `file` reports arm64 Mach-O executables; `vtool -show-build` reports device iOS minimum 12.0 and Simulator minimum 14.0, SDK 26.5 for each. C11 and Rust layout assertions pass for `CMTime` size 24, alignment 4, and field offsets 0, 8, 12, and 16. The C fixture compiles with `-fsyntax-only`, so it does not emit a C object. Neither Rust probe was executed. Xcode 26.6 / SDK 26.5 is below the planned Xcode 27.x baseline. No CI workflow run or runtime media behavior, playback, capture, parity, or performance evidence is claimed

## Validation and handoff

- Record exact commands, toolchain/SDK versions, artifacts, imports, symbol and layout evidence,
  script failures/resolutions, and the CI wiring state
- Mark device and Simulator probes as linked but not executed
- State that these gates do not establish media runtime behavior or parity
