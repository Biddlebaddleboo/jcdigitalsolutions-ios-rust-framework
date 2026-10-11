# iOS microphone PCM file recorder

`ios-microphone-recorder` records authorized microphone input to a caller-selected audio file using the public `AVAudioFormat` and `AVAudioRecorder` APIs. It supports interleaved signed 16-bit integer or 32-bit floating-point PCM, finite positive sample rates, and one or two channels.

The host must provide `NSMicrophoneUsageDescription`, run its own permission flow, and configure and activate a recording-capable `AVAudioSession` before starting. `IosMicrophoneRecorder::new` and each `start` check that `CaptureMedia::Microphone` is already `Authorized`; the crate never prompts, configures, activates, or deactivates the process-wide audio session. The host remains responsible for coordinating recording with other audio-session users.

Pass a writable local file `NSURL` whose extension matches the intended container. CAF (`.caf`) is recommended for uncompressed PCM. The public `initWithURL:format:error:` binding infers the file type from the extension when no file-type setting is supplied and overwrites an existing file at the URL. Native initialization or recording can fail if the URL, container, requested format, current route, permission, or host-owned audio session is unsuitable.

```rust,ignore
let format = PcmRecordingFormat::new(48_000.0, 1, PcmSampleFormat::Int16)?;
let mut recorder = IosMicrophoneRecorder::new(&output_url, format)?;
recorder.start()?;
// Capture while the host keeps its recording-capable AVAudioSession active.
recorder.stop(); // Closes the output file.
```

Dropping the recorder also stops it and closes the file, without changing shared audio-session state. This crate does not enumerate microphones, expose live samples, or implement an audio engine. Host tests exercise format validation and fail-closed authorization mapping. iOS device and Simulator compile/link checks do not prove microphone capture or output-file contents; no runtime proof is claimed.
