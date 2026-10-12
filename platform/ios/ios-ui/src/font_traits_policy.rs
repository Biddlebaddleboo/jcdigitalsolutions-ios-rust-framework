/// A supported text weight applied to a native UIKit font descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IosFontWeight {
    /// Clear UIKit's bold symbolic trait.
    Regular,
    /// Set UIKit's bold symbolic trait.
    Bold,
}

/// A supported system font design applied to a native UIKit font descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IosFontDesign {
    /// Clear UIKit's monospaced symbolic trait.
    Default,
    /// Set UIKit's monospaced symbolic trait.
    Monospaced,
}

const BOLD: u32 = 1 << 1;
const MONOSPACED: u32 = 1 << 10;

pub(crate) const fn with_font_weight(traits: u32, weight: IosFontWeight) -> u32 {
    match weight {
        IosFontWeight::Regular => traits & !BOLD,
        IosFontWeight::Bold => traits | BOLD,
    }
}

pub(crate) const fn with_font_design(traits: u32, design: IosFontDesign) -> u32 {
    match design {
        IosFontDesign::Default => traits & !MONOSPACED,
        IosFontDesign::Monospaced => traits | MONOSPACED,
    }
}

#[cfg(test)]
mod tests {
    use super::{IosFontDesign, IosFontWeight, with_font_design, with_font_weight};

    const ITALIC: u32 = 1;
    const BOLD: u32 = 1 << 1;
    const MONOSPACED: u32 = 1 << 10;

    #[test]
    fn bold_weight_adds_only_the_bold_trait() {
        assert_eq!(
            with_font_weight(ITALIC | MONOSPACED, IosFontWeight::Bold),
            ITALIC | BOLD | MONOSPACED
        );
    }

    #[test]
    fn regular_weight_clears_bold_and_preserves_other_traits() {
        assert_eq!(
            with_font_weight(ITALIC | BOLD | MONOSPACED, IosFontWeight::Regular),
            ITALIC | MONOSPACED
        );
    }

    #[test]
    fn monospaced_design_adds_only_the_monospace_trait() {
        assert_eq!(
            with_font_design(ITALIC | BOLD, IosFontDesign::Monospaced),
            ITALIC | BOLD | MONOSPACED
        );
    }

    #[test]
    fn default_design_clears_monospace_and_preserves_other_traits() {
        assert_eq!(
            with_font_design(ITALIC | BOLD | MONOSPACED, IosFontDesign::Default),
            ITALIC | BOLD
        );
    }
}
