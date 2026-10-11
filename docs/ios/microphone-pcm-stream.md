# iOS live microphone PCM input

`ios-microphone-pcm-stream` exposes live Float32 samples from the public `AVAudioEngine` input-node tap. The stream preserves the native sample rate, one or two channels, channel order, and sample values; it does not resample, downmix, quantize, or request an alternate format. `start` returns the actual sample rate and channel count. Unsupported input formats or a changed callback format fail closed.

The host must provide `NSMicrophoneUsageDescription`, use its own explicit microphone-permission flow, and ensure `CaptureMedia::Microphone` is already `Authorized` before creation and each start. The crate never prompts. The host is also the sole owner of `AVAudioSession` policy and must configure and activate a recording-capable session before calling `start`. `AVAudioEngine.startAndReturnError` may implicitly activate the shared session; this crate does not call session APIs or select a category, mode, or route. The host must coordinate route changes, interruptions, and other audio-session users.

```rust,ignore
let mut stream = IosMicrophonePcmStream::new(48_000)?;
let format = stream.start()?;
let mut samples = [0.0_f32; 2048];
let frames_read = stream.try_read_interleaved(&mut samples)?;
// `frames_read` counts frames; the buffer contains `frames_read * format.channels()` samples.
let dropped_frames = stream.dropped_frames();
stream.stop();
```

The capacity is measured in frames, is preallocated before capture, and must be between 1 and 1,048,576 frames. A single tap callback produces samples into a bounded SPSC ring while the caller polls it without waiting. If a complete tap block does not fit, that entire block is dropped and its frame count is added to `dropped_frames`; the counter saturates at `u64::MAX`. This is a bounded best-effort stream, not lossless recording. Callback format changes latch an error; reads then return `InputFormatChanged` rather than reinterpret samples.

The stream is one-shot and intentionally `!Send`/`!Sync`. `start`, `stop`, and drop perform synchronous `AVAudioEngine` operations and may block, so call them away from latency-sensitive UI work. Stop and drop stop the engine and remove its tap but never deactivate or reconfigure the host-owned audio session. Route/interruption recovery and device selection are outside this API.

The registered profile compiles and links iOS 10.0 device and iOS 14.0 Simulator probes, checks Objective-C/framework imports and public SDK declarations, and runs host regressions for authorization policy, format validation, FIFO ordering, interleaving, wraparound, overflow, capacity, and lifecycle. These checks do not run the link probe, request permission, access a microphone, or establish live capture, timing, acoustic fidelity, route recovery, or physical-device behavior.
