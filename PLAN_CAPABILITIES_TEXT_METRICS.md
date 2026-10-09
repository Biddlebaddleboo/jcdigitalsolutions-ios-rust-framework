# PLAN_CAPABILITIES_TEXT_METRICS.md — Workstream D18: System Font Metrics

## Status

D18 adds a no-std `FontMetrics` value and `TextMetricsBackend` contract to existing
`framework-ui`. Two deterministic tests cover finite native values and invalid inputs. It does not
implement text shaping, width measurement, line breaks, or layout. Row 056 remains a partial
capability until the root-owned matrix records this narrow slice

## Objective

Expose scaled ascent, descent, and leading for a backend's system UI font at one positive point size.
Keep the contract free of CoreText and UIKit types

## Write scope

- `crates/framework-ui/src/lib.rs`
- `docs/capabilities/text-metrics.md`
- `PLAN_CAPABILITIES_TEXT_METRICS.md`

Do not edit shared capability matrices, umbrella plans, indexes, root manifests, or `Cargo.lock`

## Contract

- `FontMetrics::new(point_size, ascent, descent, leading)` accepts only a finite positive point size
  and finite metric values. Preserve signed native metrics; do not infer positivity for descent or
  leading
- `TextMetricsBackend::system_font_metrics` returns metrics only. Font choice and metric units are
  backend-defined
- Do not add shaping, text width, glyph bounds, paragraph layout, line breaks, font registration,
  custom-font selection, Dynamic Type, or UIKit objects
- Keep the crate `no_std`, safe, and on its current dependency set

## Validation

- Test finite metrics, signed metric values, and invalid size/metric values
- Check no-default-features compilation, strict Clippy, rustdoc, formatting, and diff whitespace
- Do not infer native behavior from portable fixtures
