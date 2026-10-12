#[cfg(target_os = "ios")]
fn main() {
    if std::env::var_os("IOS_PCM_AUDIO_OUTPUT_LINK_PROBE").is_none() {
        return;
    }

    use ios_pcm_audio_output::{IosPcmAudioOutput, PcmOutputFormat, PcmSampleFormat};

    let Ok(format) = PcmOutputFormat::new(48_000.0, 1, PcmSampleFormat::Float32) else {
        return;
    };
    let Ok(mut output) = IosPcmAudioOutput::new(format, 4_800) else {
        return;
    };
    let _ = output.write_float32_interleaved(&[0.0; 2]);
    let _ = output.queued_frames();
    let _ = output.is_running();
    let _ = output.start();
    output.stop();
}

#[cfg(not(target_os = "ios"))]
fn main() {}
