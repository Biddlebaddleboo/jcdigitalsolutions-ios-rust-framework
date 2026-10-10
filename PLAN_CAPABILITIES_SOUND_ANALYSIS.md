# PLAN_CAPABILITIES_SOUND_ANALYSIS.md — Workstream D41: Built-in SoundAnalysis Request

## Objective

Add a portable scalar for recognition of Apple's built-in version 1 SoundAnalysis classifier request. This is an additive value in `framework-media`; it does not claim SoundAnalysis family or ShazamKit support.

## Required contract

- Keep the value copied, `no_std`, and allocation-free
- Define `true` only as successful request construction plus a nonempty known-label list
- State that the snapshot does not imply microphone access, model execution, analysis success, custom-model support, or ShazamKit catalog access
- Do not create an analyzer, provide audio, access the microphone, or request permission

## Validation

Run portable no-default check, strict Clippy, and rustdoc through `platform/ios/ios-sound-analysis/check.sh`. No model execution or audio query is claimed.

## Write scope

- Additive `SoundAnalysisSupportSnapshot` in `crates/framework-media/**`
- `docs/capabilities/sound-analysis.md`

The orchestrator owns workspace integration, the canonical matrix, CI, and aggregate-plan updates.

## D41 reconciliation evidence (2026-10-10)

The additive `SoundAnalysisSupportSnapshot` and iOS query were already present. The portable value
is a copied `bool` with a `const` constructor and getter in `framework-media`, which is `#![no_std]`,
forbids unsafe code, declares no dependencies, and does not use `alloc`. Its API docs and
`docs/capabilities/sound-analysis.md` now state that `true` means the built-in version 1 request
constructed successfully and exposed a nonempty known-classification list. They also state that
the value does not imply microphone access, model execution, analysis success, custom-model support,
or ShazamKit catalog access. No duplicate API or probe was added.

The existing iOS adapter remains read-only: on iOS 15.0 and later it constructs
`SNClassifySoundRequest` with `SNClassifierIdentifierVersion1`, returns `false` on construction
failure, and returns `true` only when `knownClassifications()` is nonempty. It creates no analyzer,
supplies no audio, accesses no microphone, and requests no permission. Older iOS releases and
non-iOS targets return `false`. No D17, B46, iOS adapter implementation, or canonical matrix change
was made.

`PATH="$PWD/target/ios-rust-tools/bin:$PATH" sh platform/ios/ios-sound-analysis/check.sh` passed
with exit code 0 on Xcode 26.6 (17F113), iOS SDK 26.5, and Rust/Cargo 1.94.1. It covered formatting;
the portable no-default check, strict Clippy, and rustdoc; host and `aarch64-apple-ios` /
`aarch64-apple-ios-sim` checks and strict Clippy; adapter rustdoc; Release device/Simulator link,
import, and symbol audits; dependency-feature guards; `cargo xtask docs-check`;
`cargo xtask zero-swift-source`; and `git diff --check`. The pinned `ios-rust-build` and
`ios-rust-validate` 0.1.0 tools were installed from the repository archives; both report source SHA
`2289e6a73257b696f6ae5ecd61ee20fd16ab8b37`. The validator list has no SoundAnalysis profile, so no
`ios-rust-validate --capability` check applies. These compile/link checks do not run a model or
prove device/Simulator runtime behavior; no audio query, microphone access, permission request,
custom-model support, or ShazamKit behavior is claimed. Xcode 27 qualification and physical-device
runtime remain unverified.
