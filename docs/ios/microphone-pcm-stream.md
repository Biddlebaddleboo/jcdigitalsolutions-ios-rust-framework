# iOS live microphone PCM input

`ios-microphone-pcm-stream` exposes live Float32 samples from the public `AVAudioEngine` input-node tap. The stream preserves the native sample rate, one or two channels, channel order, and sample values; it does not resample, downmix, quantize, or request an alternate format. `start` returns the actual sample rate and channel count. Unsupported input formats or a changed callback format fail closed.

The host must provide `NSMicrophoneUsageDescription`, use its own explicit microphone-permission flow, and ensure `CaptureMedia::Microphone` is already `Authorized` before stream creation and each start. The crate never prompts. The host is also the sole owner of `AVAudioSession` policy and must configure and activate a recording-capable session before calling `start`. `AVAudioEngine.startAndReturnError` may implicitly activate the shared session; stream operations do not configure or activate it.

`available_input_ports()` is a separate read-only query, available from iOS 7.0. It converts each port's system type string, descriptive name, and system-assigned UID from `AVAudioSession.sharedInstance().availableInputs` to Rust UTF-8 `String` values with the pinned `objc2-foundation` 0.3.2 `NSString::to_string()` conversion. The conversion does not normalize well-formed Unicode text, and unknown future port-type strings are preserved when they are valid Unicode. The binding relies on `NSString::UTF8String` and assumes valid UTF-8; this API makes no lossless-handling guarantee for malformed UTF-16. Apple-provided port metadata is expected to be valid Unicode. The returned order and entries match the native array. A nil or empty native list becomes an empty `Vec`. Apple scopes this property to the host's current session category and mode, so a playback category can report no input ports. This is only a point-in-time list of ports Apple reports as available for routing, not all physical devices, the active route, or proof that capture can start. The host must coordinate its own session policy and route changes. This function does not read authorization, request permission, start audio I/O, change session state, select a route, or register observers. A UID is not promised to persist across reconnects, app launches, hardware changes, or operating-system updates.

```rust,ignore
let input_ports = available_input_ports()?;
for port in input_ports {
    println!("{}: {} ({})", port.port_type(), port.name(), port.uid());
}
```

The query does not grant permission or guarantee that the listed ports will remain available. The host remains responsible for microphone permission and session configuration before recording.

`select_input_port(uid)` is an explicit preferred-input request, available from iOS 7.0. Pass a UID from a current `available_input_ports()` result; the function matches it exactly against the host session's current `availableInputs` list and calls the public `AVAudioSession.setPreferredInput:error:` API only for that retained entry. An empty UID returns `InvalidUid`; a nil list or UID no longer present returns `InputNotAvailable`; a native setter error returns `NativeFailure`; older systems return `UnsupportedOperatingSystem`. A successful call means the preference request was accepted, not that the active route changed or capture can start. Apple documents that selecting an input already on the route has no effect. The list and route may change between query and selection.

The host remains the sole owner of audio-session category, mode, activation, microphone permission, and route/interruption lifecycle. Selection does not set those values, prompt, start I/O, clear a preferred input, or register observers. The host must track its own route state if it needs to know which input is active.

```rust,ignore
let input_ports = available_input_ports()?;
if let Some(port) = input_ports.first() {
    select_input_port(port.uid())?;
}
```

`IosAudioSessionEventMonitor::new()` subscribes to the public
`AVAudioSessionRouteChangeNotification` and `AVAudioSessionInterruptionNotification` notifications
from iOS 6.0. It records a route-change flag and coalesced interruption-began, interruption-ended,
and interruption-unclassified flags. Interruption type comes from the documented
`AVAudioSessionInterruptionTypeKey` `NSNumber`; a missing key, non-number value, or unknown enum
value sets the unclassified flag instead of being discarded. The monitor does not parse route
reasons, previous routes, or interruption resume options. Query the host-owned session or inspect
your own policy state if you need those details. The monitor never configures, activates, or
deactivates `AVAudioSession`, requests permission, starts I/O, or runs an application closure.

Notification Center invokes its observer block on the posting thread because registration uses no
operation queue. The block only reads the interruption type and updates atomic flags; it retains no
notification and runs no application code. `drain()` returns independent flags and clears them
without waiting: repeated events of each type coalesce until the next drain, counts and ordering are
not preserved, and both began and ended may be true if both occurred before one drain. A
notification concurrent with the atomic drain belongs either to the current or the next snapshot.
The fixed-size flags cannot overflow. Dropping the monitor removes both observer tokens. The
registered profile retains the existing iOS 10.0 device and iOS 14.0 Simulator compile/link floors;
these checks do not prove live notification delivery or runtime route/interruption behavior.

```rust,ignore
let monitor = IosAudioSessionEventMonitor::new()?;
let events = monitor.drain();
if events.route_change() {
    // Reconcile host-owned audio routing if required.
}
if events.interruption_began() {
    // Apply host-owned interruption-began policy.
}
if events.interruption_ended() {
    // Apply host-owned interruption-ended policy.
}
if events.interruption_unclassified() {
    // Inspect host-owned state or conservatively handle an unknown notification payload.
}
```

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

The stream is one-shot and intentionally `!Send`/`!Sync`. `start`, `stop`, and drop perform synchronous `AVAudioEngine` operations and may block, so call them away from latency-sensitive UI work. Stop and drop stop the engine and remove its tap but never deactivate or reconfigure the host-owned audio session. The event monitor reports coalesced route and interruption-type flags only; route/interruption recovery and active-route observation remain outside this API.

The registered profile compiles and links iOS 10.0 device and iOS 14.0 Simulator probes, checks Objective-C/framework imports and public SDK declarations, and runs host regressions for input-port snapshot mapping and selection, authorization policy, format validation, FIFO ordering, interleaving, wraparound, overflow, capacity, and lifecycle. These checks do not run the link probe, query a live audio session, request permission, access a microphone, or establish live capture, timing, acoustic fidelity, route selection behavior, route recovery, or physical-device behavior.
