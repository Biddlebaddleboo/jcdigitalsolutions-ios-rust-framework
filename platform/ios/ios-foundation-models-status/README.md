# iOS Foundation Models availability snapshot

This crate exposes only `SystemLanguageModel.default.isAvailable` as a point-in-time snapshot for
the default on-device language model

`Ok(true)` means Apple reports the default model entirely ready for requests at the time of the
call. `Ok(false)` does not preserve an unavailable reason. The call creates no session or prompt,
runs no inference, and shows no UI. It does not claim model support for every use case, output
quality, entitlement state, or future request success

The public Swift API is available from iOS 26.0. Apple currently labels the API as beta. The crate
uses compiler-derived C `swiftcall` thunks and `swift-abi-core` to release the owned default-model
object; it adds no Swift source

See [the focused guide](../../../docs/ios/foundation-models-status.md) and
[B236 plan record](../../../PLAN_CAPABILITIES_FOUNDATION_MODELS.md)
