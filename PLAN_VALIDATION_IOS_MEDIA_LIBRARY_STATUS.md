# PLAN_VALIDATION_IOS_MEDIA_LIBRARY_STATUS.md — G65: MediaPlayer status gates

## Objective and exact starting scope
Validate the narrow `ios-media-library-status` status surface, its `platform/ios/ios-media-library-status/check.sh`, Cargo features and native import boundaries. This is static library/SDK evidence, **not** a media authorization or playback workflow.

## Completed shared tooling
Use the pinned PATH tools per `docs/SHARED_TOOLING.md`. G65 is not registered in the current four-pilot `tools/validation/specs/validation-v1.json`. The existing focused script remains authoritative for this package; a new schema-v1 profile may absorb generic checks only after matching its precise feature isolation and negative assertions. Do not modify shared engine code.

## Required gates
- Verify source formatting and `sh -n platform/ios/ios-media-library-status/check.sh`.
- Check the library on the host, `aarch64-apple-ios`, and `aarch64-apple-ios-sim` using locked resolution where supported.
- Run strict Clippy for iOS device and Simulator targets and iOS-target rustdoc.
- Inspect the Cargo feature tree: `objc2-media-player` must be absent off iOS; on iOS select only `MPMediaLibrary` plus required generated Foundation features; `default` and `block2` must remain disabled.
- Run `git diff --check`.
- **Do not** add runtime tests, link probes, authorization queries, user prompts, item reads, service calls or playback in order to claim G65 passed.

The current package-local execution entrypoint is:
```sh
sh -n platform/ios/ios-media-library-status/check.sh
sh platform/ios/ios-media-library-status/check.sh
git diff --check
```
Confirm the script still contains the above host/device/simulator, Clippy, documentation and feature gates. If it does not, run the missing focused gate explicitly rather than assuming equivalence.

## Historical evidence (not instructions to rerun obsolete temporary paths)
Before integration, a temporary workspace at `/tmp/ios-media-library-status-check.ymWzCe` successfully ran offline host/device/simulator Cargo checks, strict device/simulator Clippy and iOS rustdoc with the expected feature-tree isolation. That temporary path is historical evidence, **not** a stable workspace location for new executors. After root lockfile integration, `platform/ios/ios-media-library-status/check.sh` passed on 2026-10-08 for formatting, shell syntax, feature isolation, host/device/simulator checks, strict Clippy and iOS rustdoc. No tests or probe binaries were built or executed.

On 2026-10-10, macos-15 CI runs `38036184360` (job `114166971300`) and `38036234051` (job `114167116143`) failed at the required-feature guard after host/device/Simulator checks and both strict Clippy commands had passed. The single-quoted `grep` pattern included literal backslashes before its double quotes, while `cargo tree` emits plain double quotes, so the required `MPMediaLibrary` match failed silently before rustdoc. The positive and negative feature patterns now use plain double quotes and retain both assertions. On Cargo `1.94.1 (29ea6fb6a 2026-03-24)` / rustc `1.94.1 (e408947bf 2026-03-25)`, `sh platform/ios/ios-media-library-status/check.sh` passed formatting, host/device/Simulator checks, strict device/Simulator Clippy, feature isolation and iOS rustdoc after the correction. No runtime tests or probe binaries were built or executed; a CI rerun of this correction is still required.

## Validation migration and handoff
If registering G65 in `ios-rust-validate`, confirm host-vs-iOS Cargo features and `default`/`block2` negative assertions remain enforceable; use a bounded Python adapter only if the schema cannot capture these checks. Demonstrate both success and deliberate failure in a scoped fixture before removing the local script. Record executed gates, feature tree, target/toolchain, skips and SHA. No runtime or hardware result may be inferred.
