# iOS Natural Language contextual-model asset status

`ios_natural_language_status::english_contextual_embedding_assets()` reports whether Apple has on-device assets for its English contextual model. Its result is `Unavailable` below iOS 17.0, `LanguageUnavailable` if the generated English language constant is nil, `NoModel` if Apple returns no English model object, or `AssetsAvailable` / `AssetsNotAvailable` from `NLContextualEmbedding::hasAvailableAssets`

The call creates a model object only. It does not load the model, accept text, compute vectors, request assets, or start a download. `AssetsAvailable` does not promise that a later model load or vector call will succeed

The local iPhoneOS26.5 SDK marks `NLContextualEmbedding` available from iOS 17.0 and `NLLanguageEnglish` from iOS 12.0. The local `objc2-natural-language` 0.3.2 binding exposes the needed symbols with features `NLContextualEmbedding` and `NLLanguage`; no `block2` or Core ML feature is active

See [D54](../../../PLAN_CAPABILITIES_NATURALLANGUAGE_STATUS.md), [B59](../../../PLAN_IOS_NATURALLANGUAGE_STATUS.md), and Apple’s [NLContextualEmbedding reference](https://developer.apple.com/documentation/naturallanguage/nlcontextualembedding)
