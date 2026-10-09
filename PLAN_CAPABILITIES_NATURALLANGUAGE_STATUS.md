# PLAN_CAPABILITIES_NATURALLANGUAGE_STATUS.md — D54: English Contextual Model Asset Status

## Objective

Add one iOS-only status query for Apple English contextual-model assets under row `065-ml-vision-language-naturallanguage-system-model-features`. This slice adds no portable Natural Language contract

## Public boundary

- `ios_natural_language_status::english_contextual_embedding_assets() -> EnglishEmbeddingAssetStatus`
- Return `Unavailable` below iOS 17.0, `LanguageUnavailable` if the generated English language constant is nil, `NoModel` if Apple returns no model object, or the on-device asset status
- Treat `AssetsAvailable` as a point-in-time asset bit only, not proof that a model load or vector call will succeed
- Do not load the model, accept text, compute vectors, request assets, start a download, or expose Apple objects
- Do not claim a general Natural Language service state; this slice checks one Apple English contextual model only

## API evidence

The local iPhoneOS26.5 SDK marks `NLContextualEmbedding` available from iOS 17.0. It declares `contextualEmbeddingWithLanguage:` and readonly property `hasAvailableAssets`. Apple says the property reports whether model assets are on-device and lists asset request and model load as separate calls. `NLLanguageEnglish` is available from iOS 12.0

The `objc2-natural-language` 0.3.2 binding exposes the class factory and asset property with features `NLContextualEmbedding` and `NLLanguage`. It maps `NLLanguageEnglish` to `Option<&'static NLLanguage>` and the factory to `Option<Retained<NLContextualEmbedding>>`. This package maps nil to `LanguageUnavailable` or `NoModel` as appropriate

## Validation and limits

- Run host, device, and simulator package check, strict Clippy, and rustdoc gates
- Build but do not execute device and simulator link probes; inspect the exact NaturalLanguage/Foundation import allowlist, Objective-C symbols, and absence of Swift runtime symbols
- Do not add or run tests, execute probes, request assets, load a model, accept text, or compute vectors
- These gates prove source and link shape only. They do not prove asset state on hardware, model-load success, vector output, or language coverage beyond the built-in English constant
