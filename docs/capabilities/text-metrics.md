# Portable System Font Metrics

`framework-ui::FontMetrics` stores a positive point size and finite ascent, descent, and leading
values. `TextMetricsBackend` defines one system-font query. Units and font choice remain
backend-defined; signed metric values are preserved

This is a narrow partial slice of capability row 056. It does not shape text, measure width, find
glyph bounds, wrap lines, or compute paragraph layout. It has no custom-font, font-style,
Dynamic-Type, or UIKit contract. See the [iOS CoreText guide](../ios/text-metrics.md)
