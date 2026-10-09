/// A bounded set of UIKit accessibility textual contexts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccessibilityTextualContext {
    /// Text intended for word-processing content.
    WordProcessing,
    /// Narrative text.
    Narrative,
    /// Messaging content.
    Messaging,
    /// Spreadsheet content.
    Spreadsheet,
    /// File-system content.
    FileSystem,
    /// Source code.
    SourceCode,
    /// Console content.
    Console,
}
