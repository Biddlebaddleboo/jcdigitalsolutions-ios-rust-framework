# PLAN_CAPABILITIES_GEOMETRY.md — Workstream D16: Finite Frame Intersection

## Status

D16 extends the existing `framework-ui::Frame` value with a portable, allocation-free positive-area
intersection operation. Seven `framework-ui` tests pass, including deterministic overlap, disjoint
and touching cases, degenerate frames, and finite/extreme arithmetic. Locked no-default-features
check, rustdoc, and strict host Clippy pass. D16 does not add a general geometry model or a platform
dependency; CoreGraphics parity is outside D16

## Objective

Define the exact intersection of two finite frames in one coordinate space. Keep the operation in
the existing `framework-ui` crate; do not add a crate, dependency, or unsafe code

## Portable API and semantics

- Add `Frame::intersection(self, other: Self) -> Result<Option<Self>>`
- Both inputs must use the same coordinate space and logical units; the method does not transform
  coordinates
- Preserve `Frame::new` rules: finite origin and dimensions, non-negative width and height; zero
  dimensions remain valid `Frame` values
- Compute each maximum edge as `origin + dimension`. Return `InvalidInput` if a maximum edge is
  non-finite or if a positive dimension fails to advance its origin at `f64` precision
- Return `Ok(None)` for any zero-size input and for disjoint, edge-touching, or corner-touching
  frames. A result always has positive width and height
- Return `InvalidInput` if the result dimensions or maximum edges are non-finite or unrepresentable
  by a `Frame`; apply no epsilon or pixel rounding
- Preserve D14's `UiBackend`, `UiClient`, controls, and frame constructor behavior

## Dependencies and write scope

- `framework-core` remains the only portable dependency
- Write scope: `crates/framework-ui/**`, `docs/capabilities/ui.md`, and this plan
- Do not edit Cargo manifests/lockfile, root plans, capability manifests/counts, docs indexes, CI,
  UIKit backends, bindings, or unrelated geometry families; root owns integration

## Validation

- Add deterministic tests for overlap, identity/containment, disjoint rectangles, horizontal,
  vertical, and corner contact, zero dimensions, overflowing maximum edges, precision-collapsed
  positive dimensions, and safe finite extremes
- `cargo +1.94.1 test --locked --offline -p framework-ui`: 7 passed
- `cargo +1.94.1 check --locked --offline -p framework-ui --no-default-features`: pass
- `cargo +1.94.1 doc --locked --offline -p framework-ui --no-deps`: pass
- `cargo +1.94.1 clippy --locked --offline -p framework-ui --all-targets -- -D warnings`: pass
- `cargo +1.94.1 xtask docs-check`: pass; shared documentation index check passes
- Scoped Rust formatting, shell syntax, `git diff --check`, and scoped trailing-whitespace scan:
  pass
- Report that portable arithmetic tests do not establish CoreGraphics parity

## Exclusions

No points or general rectangle type, union, containment, transforms, paths, affine matrices,
negative dimensions, null/infinite sentinels, layout, pixels, QuartzCore, rendering, or performance
claim. The separate iOS adapter is in [PLAN_IOS_GEOMETRY.md](PLAN_IOS_GEOMETRY.md)
