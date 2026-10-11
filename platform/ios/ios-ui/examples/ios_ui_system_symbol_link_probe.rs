#[cfg(target_os = "ios")]
use framework_core::Result;
#[cfg(target_os = "ios")]
use framework_ui::Frame;
#[cfg(target_os = "ios")]
use ios_ui::{IosImageView, IosUiBackend, IosView};
#[cfg(target_os = "ios")]
use objc2_ui_kit::UIColor;

#[cfg(target_os = "ios")]
type AddSystemSymbolImageView =
    fn(&mut IosUiBackend, &mut IosView, Frame, &str, Option<&UIColor>) -> Result<IosImageView>;

#[cfg(target_os = "ios")]
fn main() {
    core::hint::black_box(IosUiBackend::add_system_symbol_image_view as AddSystemSymbolImageView);
    core::hint::black_box(
        IosImageView::set_tint_color as fn(&mut IosImageView, &UIColor) -> Result<()>,
    );
}

#[cfg(not(target_os = "ios"))]
fn main() {}
