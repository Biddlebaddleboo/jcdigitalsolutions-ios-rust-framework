# PLAN_CAPABILITIES_SPRITEKIT.md — Workstream D64: Portable SpriteKit Node Position

## Status

D64 adds a no-std finite `SpriteNodePosition` value and a static `SpriteNodePositionBackend` contract for one SpriteKit node's parent-local position. It does not define scenes, rendering, animation, or a generic visual tree.

## Objective

Represent only finite x/y coordinates and one backend read/write contract for the position property of a SpriteKit `SKNode`.

## Dependencies

- `framework-core` for stable portable error kinds
- No third-party crate, allocator, platform type, or unsafe code in the portable crate

## Contract

- `SpriteNodePosition::new` accepts finite `f64` coordinates and rejects NaN or infinity as `InvalidInput`
- The backend contract reads and replaces the local `SKNode.position`; the host scene defines coordinate units
- The contract does not specify display pixels, transforms, parent hierarchy, or a rendered point

## Non-goals

- No `SKScene`, `SKView`, SceneKit, renderer, node creation contract, hierarchy, animation, physics, hit testing, file loading, or visual content
- No parity, timing, performance, or runtime claim

## Validation

- Run format, host `cargo check`, strict Clippy, and rustdoc for `framework-spritekit`
- Do not run tests
- Do not edit workspace files, shared matrices, indexes, CI, aggregate plans, or global validation docs

## Validation record

On Rust 1.94.1, format, host no-default-features check, strict Clippy, and rustdoc pass in the isolated validation worktree. No tests were run.

- `cargo fmt --manifest-path crates/framework-spritekit/Cargo.toml --package framework-spritekit -- --check`
- `cargo check --locked --offline -p framework-spritekit --no-default-features`
- `cargo clippy --locked --offline --all-targets -p framework-spritekit -- -D warnings`
- `cargo doc --locked --offline -p framework-spritekit --no-deps`
