# PLAN_IOS_NATURALLANGUAGE_STATUS.md — B59: English Contextual Model Asset Query

## Objective

Expose a narrow iOS Rust status query for the built-in English Natural Language contextual model under row `065-ml-vision-language-naturallanguage-system-model-features`

## Scope

The isolated B59 workstream changed `platform/ios/ios-natural-language-status/**` and this plan.
Root integration adds the central `objc2-natural-language` workspace dependency, row 065 status,
CI gate, shared validation plan, and iOS documentation index; no portable Natural Language
contract is added.

## API requirements

- Export only `english_contextual_embedding_assets() -> EnglishEmbeddingAssetStatus`
- Guard both Natural Language calls with `objc2::available!(ios = 17.0, ..)` and return `Unavailable` below that floor
- Read `NLLanguageEnglish`, map nil to `LanguageUnavailable`, then call only `NLContextualEmbedding::contextualEmbeddingWithLanguage(language)` and `hasAvailableAssets`
- Map a nil language constant to `LanguageUnavailable`, a nil model object to `NoModel`, and the asset property to `AssetsAvailable` or `AssetsNotAvailable`
- Pin `objc2-natural-language` 0.3.2 in the workspace dependency table and enable only features
  `NLContextualEmbedding` and `NLLanguage`; do not enable `block2` or `objc2-core-ml`
- Do not call asset request, model load/unload, text/vector, recognition, or tagger APIs
- Add no prompt, permission, privacy key, or main-thread marker
- Do not add a portable contract because this result describes one Apple model pack

## Validation and limits

- Run `sh platform/ios/ios-natural-language-status/scripts/check.sh`
- The link script builds device and simulator probes and inspects imports, Objective-C symbols, and Swift-runtime strings; it never executes a probe
- Do not add or run tests or invoke Natural Language model APIs beyond the two status calls
- These gates do not prove runtime asset state, model load, vector results, or model availability on any device
