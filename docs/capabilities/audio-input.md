# Portable live PCM input

`framework-audio` defines a no_std, dependency-free data contract for live PCM readers. The current
format is Float32 with interleaved channels, a finite positive sample rate, and a nonzero channel
count. It describes samples returned to the caller; a backend may read planar native buffers and
interleave them without changing sample values, sample rate, or channel order.

`PcmInputStream` is a generic Rust trait with an associated backend error. Applications can write
statically dispatched processing code over `S: PcmInputStream` without routing the Rust fast path
through C ABI calls or dynamic dispatch. The trait defines frame-count reads, an unchanged output
tail, incomplete-frame rejection, and cumulative dropped-frame accounting. It does not prescribe
permission, native session policy, input selection, stream startup, or shutdown.

```rust,ignore
use framework_audio::{PcmInputFormat, PcmInputStream};

fn read_frames<S: PcmInputStream>(input: &mut S, output: &mut [f32]) -> Result<u32, S::Error> {
    input.try_read_interleaved(output)
}

let format = PcmInputFormat::float32_interleaved(48_000.0, 2)?;
assert_eq!(format.channels(), 2);
```

`PcmInputFormat::frames_for_interleaved_samples` validates that a sample count contains whole
frames and fits the contract's `u32` frame result. A read returns the number of frames copied, not
the number of samples; derive copied sample count as `frames * channels`. A backend error leaves the
output unchanged, and samples after the returned prefix are also unchanged. The contract does not
promise a particular native source, acoustic quality, latency, or runtime availability.

The iOS implementation is `ios-microphone-pcm-stream`; its permission, AVAudioSession, lifecycle,
and overflow policies remain platform-specific. See the [iOS live microphone guide](../ios/microphone-pcm-stream.md).
