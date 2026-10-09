#![cfg(target_os = "ios")]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A narrow iOS Natural Language contextual-model asset status query"]

use objc2_natural_language::{NLContextualEmbedding, NLLanguageEnglish};

/// The status of Apple English contextual-model assets
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnglishEmbeddingAssetStatus {
    /// NaturalLanguage APIs do not exist on the current iOS version
    Unavailable,
    /// The generated English language constant has no value
    LanguageUnavailable,
    /// Apple returned no English contextual-model object
    NoModel,
    /// An English model exists, but its assets are not on-device
    AssetsNotAvailable,
    /// An English model exists and its assets are on-device
    AssetsAvailable,
}

/// Read the on-device asset status for Apple's English contextual model
///
/// This creates a `NLContextualEmbedding` object for `NLLanguageEnglish` and reads
/// `NLContextualEmbedding::hasAvailableAssets`. The method does not load the model, accept text,
/// compute vectors, request assets, or start a download. The result does not guarantee a future
/// model load or vector result
///
/// This returns [`EnglishEmbeddingAssetStatus::Unavailable`] below iOS 17.0. The inspected header
/// states no main-thread requirement for the used class methods
pub fn english_contextual_embedding_assets() -> EnglishEmbeddingAssetStatus {
    if !objc2::available!(ios = 17.0, ..) {
        return EnglishEmbeddingAssetStatus::Unavailable;
    }

    // SAFETY: Apple declares this constant from iOS 12.0; the guard above enforces iOS 17.0
    let Some(language) = (unsafe { NLLanguageEnglish }) else {
        return EnglishEmbeddingAssetStatus::LanguageUnavailable;
    };

    // SAFETY: this branch enforces the iOS 17.0 class API floor; `language` is the binding's English value
    let Some(model) = (unsafe { NLContextualEmbedding::contextualEmbeddingWithLanguage(language) })
    else {
        return EnglishEmbeddingAssetStatus::NoModel;
    };

    // SAFETY: `model` is a retained NaturalLanguage object returned by the guarded class factory
    if unsafe { model.hasAvailableAssets() } {
        EnglishEmbeddingAssetStatus::AssetsAvailable
    } else {
        EnglishEmbeddingAssetStatus::AssetsNotAvailable
    }
}
