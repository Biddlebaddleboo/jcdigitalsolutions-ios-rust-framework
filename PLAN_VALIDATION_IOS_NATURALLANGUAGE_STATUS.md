# G53 — Natural Language asset-status gate

## Scope

Gate D54/B59's iOS-only status query for Apple’s built-in English contextual-model assets. It does
not load a model, accept text, compute vectors, request assets, or start a download.

## Commands

- `sh platform/ios/ios-natural-language-status/scripts/check.sh`
- `sh platform/ios/ios-natural-language-status/scripts/check-link-imports.sh`

## Evidence boundary

The package check covers format, host/device/Simulator compilation and strict Clippy, rustdoc,
documentation and zero-Swift gates, API-surface bounds, and device/Simulator Release link-import
inspection. The link probes are built and inspected, never executed. They do not establish live asset
state, model-load success, vector output, or language coverage beyond the built-in English constant.
No tests are added or run by this gate.

## Status

The focused package gate passed in the isolated D54/B59 worktree and again in the integrated root
checkout. Local validation used Rust 1.94.1, Xcode 26.6 build 17F113,
and iPhoneOS/iPhoneSimulator SDK 26.5; the plan’s Xcode 27.x baseline is not met.
