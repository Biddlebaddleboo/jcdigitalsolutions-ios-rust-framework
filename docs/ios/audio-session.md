# iOS audio-session control

`ios-audio-session` exposes explicit control of the process-wide `AVAudioSession` through public AVFAudio calls. Pick a category, mode, and option set, then call `configure`; activation and deactivation are separate explicit calls. The zero-state `IosAudioSession::shared()` handle has no create/drop side effect.

```rust,ignore
use ios_audio_session::{
    AudioSessionCategory, AudioSessionCategoryOptions, AudioSessionConfiguration,
    AudioSessionDeactivationOptions, AudioSessionMode, IosAudioSession,
};

let session = IosAudioSession::shared();
session.configure(
    AudioSessionConfiguration::new(
        AudioSessionCategory::PlayAndRecord,
        AudioSessionMode::VoiceChat,
    )
    .with_options(
        AudioSessionCategoryOptions::ALLOW_BLUETOOTH_HFP
            | AudioSessionCategoryOptions::DEFAULT_TO_SPEAKER,
    ),
)?;
session.activate()?;
// Stop or pause this app's audio I/O before deactivation.
session.deactivate(AudioSessionDeactivationOptions::NOTIFY_OTHERS)?;
```

The host retains all category, mode, activation, routing, permission, and lifecycle policy. This crate does not request permission, create audio I/O, select a route, or mutate session state at handle creation or drop. Apple validates category/mode/option combinations and can reject a call with an `NSError`; `IosAudioSessionError::NativeFailure` preserves the operation and numeric error code. `activate` and `deactivate` are synchronous system calls, not real-time callback work. Stop this app's audio I/O before deactivation; Apple may reject deactivation while audio is active. Decide how this process-wide session interacts with other media and CallKit users.

The public category/mode configuration API requires iOS 10.0. Device and Simulator compile/link probes check the AVAudioSession selectors, public option declarations, framework imports, and deployment metadata; probes do not run. Host tests check option mapping and error preservation. No session, route, permission, interruption, CallKit, or audible runtime behavior is claimed.
