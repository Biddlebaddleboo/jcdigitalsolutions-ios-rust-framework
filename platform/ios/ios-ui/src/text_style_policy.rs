/// A built-in UIKit text style that follows the user's preferred content size.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IosSystemTextStyle {
    /// UIKit's large title style.
    LargeTitle,
    /// UIKit's first title style.
    Title1,
    /// UIKit's second title style.
    Title2,
    /// UIKit's third title style.
    Title3,
    /// UIKit's headline style.
    Headline,
    /// UIKit's subheadline style.
    Subheadline,
    /// UIKit's body style.
    Body,
    /// UIKit's callout style.
    Callout,
    /// UIKit's footnote style.
    Footnote,
    /// UIKit's first caption style.
    Caption,
    /// UIKit's second caption style.
    Caption2,
}

/// A UIKit semantic or system text color.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IosSemanticTextColor {
    /// Primary text, dynamic on iOS 13 and later.
    Primary,
    /// Secondary text, dynamic on iOS 13 and later.
    Secondary,
    /// UIKit's system blue.
    Blue,
    /// UIKit's system red.
    Red,
    /// UIKit's system green.
    Green,
    /// UIKit's system orange.
    Orange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DynamicTextColor {
    Primary,
    Secondary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeTextColor {
    Black,
    Gray,
    Blue,
    Red,
    Green,
    Orange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TextColorResolution {
    Dynamic(DynamicTextColor),
    Native(NativeTextColor),
}

impl IosSemanticTextColor {
    pub(crate) const fn resolve(self, dynamic_semantic_colors: bool) -> TextColorResolution {
        match self {
            Self::Primary if dynamic_semantic_colors => {
                TextColorResolution::Dynamic(DynamicTextColor::Primary)
            }
            Self::Primary => TextColorResolution::Native(NativeTextColor::Black),
            Self::Secondary if dynamic_semantic_colors => {
                TextColorResolution::Dynamic(DynamicTextColor::Secondary)
            }
            Self::Secondary => TextColorResolution::Native(NativeTextColor::Gray),
            Self::Blue => TextColorResolution::Native(NativeTextColor::Blue),
            Self::Red => TextColorResolution::Native(NativeTextColor::Red),
            Self::Green => TextColorResolution::Native(NativeTextColor::Green),
            Self::Orange => TextColorResolution::Native(NativeTextColor::Orange),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DynamicTextColor, IosSemanticTextColor, NativeTextColor, TextColorResolution};

    #[test]
    fn primary_and_secondary_use_legacy_colors_before_ios_13() {
        assert_eq!(
            IosSemanticTextColor::Primary.resolve(false),
            TextColorResolution::Native(NativeTextColor::Black)
        );
        assert_eq!(
            IosSemanticTextColor::Secondary.resolve(false),
            TextColorResolution::Native(NativeTextColor::Gray)
        );
    }

    #[test]
    fn primary_and_secondary_use_dynamic_colors_when_available() {
        assert_eq!(
            IosSemanticTextColor::Primary.resolve(true),
            TextColorResolution::Dynamic(DynamicTextColor::Primary)
        );
        assert_eq!(
            IosSemanticTextColor::Secondary.resolve(true),
            TextColorResolution::Dynamic(DynamicTextColor::Secondary)
        );
    }

    #[test]
    fn named_system_colors_do_not_change_with_semantic_color_availability() {
        let cases = [
            (IosSemanticTextColor::Blue, NativeTextColor::Blue),
            (IosSemanticTextColor::Red, NativeTextColor::Red),
            (IosSemanticTextColor::Green, NativeTextColor::Green),
            (IosSemanticTextColor::Orange, NativeTextColor::Orange),
        ];
        for (color, expected) in cases {
            assert_eq!(color.resolve(false), TextColorResolution::Native(expected));
            assert_eq!(color.resolve(true), TextColorResolution::Native(expected));
        }
    }
}
