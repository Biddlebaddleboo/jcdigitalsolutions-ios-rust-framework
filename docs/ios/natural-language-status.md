# iOS Natural Language contextual-model asset status

`ios_natural_language_status::english_contextual_embedding_assets()` checks only Apple’s built-in
English contextual model. It returns `Unavailable` below iOS 17.0, `LanguageUnavailable` if the
generated `NLLanguageEnglish` value is absent, `NoModel` if Apple returns no contextual-model
object, or whether the model assets are on-device.

The backend creates a `NLContextualEmbedding` object and reads `hasAvailableAssets`. It does not
load or unload the model, accept text, compute vectors, request assets, or start a download. An
`AssetsAvailable` result does not guarantee that a future load or vector operation will succeed.
This slice does not claim general Natural Language model coverage.

The inspected SDK declares `NLContextualEmbedding` from iOS 17.0 and `NLLanguageEnglish` from iOS
12.0. Device and Simulator link probes imported only NaturalLanguage, Foundation,
`libobjc.A.dylib`, and `libSystem.B.dylib`; probes were built and inspected, not executed. These
checks do not prove asset state on hardware, model-load success, or vector output. Local validation
used Xcode 26.6 / iOS SDK 26.5, below the plan’s Xcode 27.x baseline.

See [D54](../../PLAN_CAPABILITIES_NATURALLANGUAGE_STATUS.md),
[B59](../../PLAN_IOS_NATURALLANGUAGE_STATUS.md), and
[G53](../../PLAN_VALIDATION_IOS_NATURALLANGUAGE_STATUS.md) for API evidence and gate results.
