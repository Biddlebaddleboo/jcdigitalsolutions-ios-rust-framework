# Portable Media Time

`framework-media::MediaTime` stores one finite rational value in seconds as a signed `i64` value and a positive `i32` timescale

```rust
use framework_media::MediaTime;

let frame = MediaTime::new(1001, 30_000)?;
assert_eq!(frame.value(), 1001);
assert_eq!(frame.timescale(), 30_000);
```

Comparison uses exact integer products in `i128`. It does not convert to `f64`, round, normalize the pair, or infer a time epoch. Ratios with distinct pairs can compare equal. `MediaTime` has no invalid, infinite, indefinite, or rounded state

This crate owns no clock, media playback, capture, sample, or sample-payload API. Its separate
[`OtherAudioPlaybackSnapshot`](other-audio.md) value records whether another app's audio was
present at query time. It has no heap allocation, platform dependency, executor, permission, or
native object
