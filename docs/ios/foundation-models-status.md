# iOS Foundation Models availability snapshot

`ios-foundation-models-status` reads `SystemLanguageModel.default.isAvailable` on iOS 26.0 and
later. The synchronous call uses compiler-derived C `swiftcall` thunks and the audited
`swift-abi-core/apple-runtime` path for the owned `SystemLanguageModel` object

`Ok(true)` means Apple reports the default on-device model entirely ready for requests at the time
of the call. `Ok(false)` does not distinguish device ineligibility, Apple Intelligence settings,
model download/readiness, or another unavailable reason. The call creates no `LanguageModelSession`
or prompt, runs no inference, and shows no UI. It does not claim use-case support, output quality,
entitlement state, or later request success

Apple currently marks the API as beta. The local Xcode 26.6 / iOS 26.5 compiler and SDK checks are
below the repository's Xcode 27.x baseline. No runtime model query or device call was run

See the [B236 Foundation Models plan record](../../PLAN_CAPABILITIES_FOUNDATION_MODELS.md) and the
[package README](../../platform/ios/ios-foundation-models-status/README.md)
