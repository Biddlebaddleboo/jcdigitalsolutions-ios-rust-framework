use core::ffi::c_void;

use framework_core::{Error, ErrorKind, Result};
use framework_ui::{FontMetrics, TextMetricsBackend};
use ios_runtime::main_thread::MainThread;
use objc2_core_foundation::CGFloat;

use crate::IosUiBackend;

const CT_FONT_UI_FONT_SYSTEM: u32 = 2;

#[link(name = "CoreText", kind = "framework")]
unsafe extern "C" {
    #[link_name = "CTFontCreateUIFontForLanguage"]
    fn ct_font_create_ui_font_for_language(
        ui_type: u32,
        size: CGFloat,
        language: *const c_void,
    ) -> *const c_void;
    #[link_name = "CTFontGetAscent"]
    fn ct_font_get_ascent(font: *const c_void) -> CGFloat;
    #[link_name = "CTFontGetDescent"]
    fn ct_font_get_descent(font: *const c_void) -> CGFloat;
    #[link_name = "CTFontGetLeading"]
    fn ct_font_get_leading(font: *const c_void) -> CGFloat;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    #[link_name = "CFRelease"]
    fn cf_release(value: *const c_void);
}

struct NativeFont(*const c_void);

impl Drop for NativeFont {
    fn drop(&mut self) {
        // SAFETY: `NativeFont` is created only from the owned, non-null CTFont reference returned
        // by CTFontCreateUIFontForLanguage; CFRelease consumes that create-rule ownership once.
        unsafe { cf_release(self.0) };
    }
}

/// Returns ascent, descent, and leading for the current system UI font at `point_size`.
///
/// This call requires the main thread, matching the `ios-ui` backend contract. The native metric
/// values use points. A null font or non-finite native metric returns `Platform`.
pub fn system_font_metrics(point_size: f64) -> Result<FontMetrics> {
    if !point_size.is_finite() || point_size <= 0.0 {
        return Err(Error::new(ErrorKind::InvalidInput));
    }
    let native_size = point_size as CGFloat;
    if !native_size.is_finite() || native_size <= 0.0 {
        return Err(Error::new(ErrorKind::InvalidInput));
    }
    let _main_thread = MainThread::current().ok_or_else(|| Error::new(ErrorKind::Unavailable))?;

    // SAFETY: `ui_type` is kCTFontUIFontSystem (2), `native_size` is finite and positive, and a
    // null language requests the current system language per CTFontCreateUIFontForLanguage.
    let font = unsafe {
        ct_font_create_ui_font_for_language(CT_FONT_UI_FONT_SYSTEM, native_size, core::ptr::null())
    };
    if font.is_null() {
        return Err(platform_error());
    }
    let font = NativeFont(font);

    // SAFETY: `font` is a live CTFontRef returned by CoreText and remains retained through reads.
    let ascent = unsafe { ct_font_get_ascent(font.0) } as f64;
    // SAFETY: `font` is a live CTFontRef returned by CoreText and remains retained through reads.
    let descent = unsafe { ct_font_get_descent(font.0) } as f64;
    // SAFETY: `font` is a live CTFontRef returned by CoreText and remains retained through reads.
    let leading = unsafe { ct_font_get_leading(font.0) } as f64;

    FontMetrics::new(point_size, ascent, descent, leading).map_err(|_| platform_error())
}

impl TextMetricsBackend for IosUiBackend {
    fn system_font_metrics(&mut self, point_size: f64) -> Result<FontMetrics> {
        system_font_metrics(point_size)
    }
}

fn platform_error() -> Error {
    Error::new(ErrorKind::Platform)
}
