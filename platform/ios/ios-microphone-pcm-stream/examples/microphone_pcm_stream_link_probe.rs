#[cfg(target_os = "ios")]
fn main() {
    if std::env::var_os("IOS_MICROPHONE_PCM_STREAM_LINK_PROBE").is_none() {
        return;
    }

    use ios_microphone_pcm_stream::IosMicrophonePcmStream;

    let Ok(mut stream) = IosMicrophonePcmStream::new(48_000) else {
        return;
    };
    let Ok(format) = stream.start() else {
        return;
    };
    let mut samples = [0.0_f32; 2];
    let _ = stream.try_read_interleaved(&mut samples[..format.channels() as usize]);
    let _ = stream.dropped_frames();
    let _ = stream.is_running();
    stream.stop();
}

#[cfg(not(target_os = "ios"))]
fn main() {}
