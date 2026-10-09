# iOS CoreText System Font Metrics

`ios_ui::text_metrics::system_font_metrics(point_size)` queries the current system UI font through
`CTFontCreateUIFontForLanguage` with `kCTFontUIFontSystem` (`2`) and a null language, then reads
`CTFontGetAscent`, `CTFontGetDescent`, and `CTFontGetLeading`. The native font is released once with
`CFRelease`. `IosUiBackend` also implements `framework_ui::TextMetricsBackend`

The query requires the main thread. A non-finite or non-positive size returns
`framework_core::ErrorKind::InvalidInput`; lack of a main-thread proof returns `Unavailable`; a
null font or invalid native metric returns `Platform`. CoreText metrics use points here. `CGFloat`
conversion uses the target's native float width

The Xcode 26.5 SDK headers declare all four CoreText calls at iOS 3.2. The local Release probe uses
iOS 12.0 device and iOS 14.0 Simulator deployment targets; these are probe settings, not a product
minimum. Private Rust FFI declarations match the public `CTFont.h` C declarations and are checked by
a C signature/layout fixture. No Swift source or Swift runtime is added

Font results can vary with OS version, installed system font, native point-size precision, and
system language. This does not claim parity with `UILabel`, preferred text styles, Dynamic Type,
custom fonts, shaping, glyph advances, text width, line breaks, or layout. Device and Simulator
build/link checks do not run CoreText or prove live metric output
