#[cfg(target_os = "ios")]
use framework_core::Result;
#[cfg(target_os = "ios")]
use framework_ui::Frame;
#[cfg(target_os = "ios")]
use ios_ui::{
    IosButton, IosFontDesign, IosFontWeight, IosImageView, IosLabel, IosSemanticTextColor,
    IosSystemTextStyle, IosTextField, IosUiBackend, IosView,
};
#[cfg(target_os = "ios")]
use objc2_ui_kit::UIColor;

#[cfg(target_os = "ios")]
type AddSystemSymbolImageView =
    fn(&mut IosUiBackend, &mut IosView, Frame, &str, Option<&UIColor>) -> Result<IosImageView>;
#[cfg(target_os = "ios")]
type SetTextStyle<T> = fn(&mut T, IosSystemTextStyle) -> Result<()>;
#[cfg(target_os = "ios")]
type SetTextColor<T> = fn(&mut T, IosSemanticTextColor) -> Result<()>;
#[cfg(target_os = "ios")]
type SetFontWeight<T> = fn(&mut T, IosFontWeight) -> Result<()>;
#[cfg(target_os = "ios")]
type SetFontDesign<T> = fn(&mut T, IosFontDesign) -> Result<()>;

#[cfg(target_os = "ios")]
fn main() {
    core::hint::black_box(IosUiBackend::add_system_symbol_image_view as AddSystemSymbolImageView);
    core::hint::black_box(
        IosImageView::set_tint_color as fn(&mut IosImageView, &UIColor) -> Result<()>,
    );
    core::hint::black_box(IosLabel::set_system_text_style as SetTextStyle<IosLabel>);
    core::hint::black_box(IosButton::set_system_text_style as SetTextStyle<IosButton>);
    core::hint::black_box(IosTextField::set_system_text_style as SetTextStyle<IosTextField>);
    core::hint::black_box(IosLabel::set_semantic_text_color as SetTextColor<IosLabel>);
    core::hint::black_box(IosButton::set_semantic_text_color as SetTextColor<IosButton>);
    core::hint::black_box(IosTextField::set_semantic_text_color as SetTextColor<IosTextField>);
    core::hint::black_box(IosLabel::set_font_weight as SetFontWeight<IosLabel>);
    core::hint::black_box(IosButton::set_font_weight as SetFontWeight<IosButton>);
    core::hint::black_box(IosTextField::set_font_weight as SetFontWeight<IosTextField>);
    core::hint::black_box(IosLabel::set_font_design as SetFontDesign<IosLabel>);
    core::hint::black_box(IosButton::set_font_design as SetFontDesign<IosButton>);
    core::hint::black_box(IosTextField::set_font_design as SetFontDesign<IosTextField>);
}

#[cfg(not(target_os = "ios"))]
fn main() {}
