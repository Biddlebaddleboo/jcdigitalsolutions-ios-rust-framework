#[cfg(target_os = "ios")]
fn main() {
    if std::env::var_os("IOS_MICROPHONE_RECORDER_LINK_PROBE").is_none() {
        return;
    }

    use ios_microphone_recorder::{IosMicrophoneRecorder, PcmRecordingFormat, PcmSampleFormat};
    use objc2_foundation::{NSString, NSURL};

    let path = NSString::from_str("/tmp/ios-microphone-recorder.caf");
    let url = NSURL::fileURLWithPath(&path);
    let Ok(format) = PcmRecordingFormat::new(48_000.0, 1, PcmSampleFormat::Float32) else {
        return;
    };
    let Ok(mut recorder) = IosMicrophoneRecorder::new(&url, format) else {
        return;
    };
    let _ = recorder.start();
    let _ = recorder.is_recording();
    recorder.stop();
}

#[cfg(not(target_os = "ios"))]
fn main() {}
