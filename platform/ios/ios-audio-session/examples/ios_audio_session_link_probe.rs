#[cfg(target_os = "ios")]
use ios_audio_session::{
    AudioSessionCategory, AudioSessionCategoryOptions, AudioSessionConfiguration,
    AudioSessionDeactivationOptions, AudioSessionMode, IosAudioSession,
};

#[cfg(target_os = "ios")]
fn main() {
    let session = IosAudioSession::shared();
    let configuration = AudioSessionConfiguration::new(
        AudioSessionCategory::PlayAndRecord,
        AudioSessionMode::VoiceChat,
    )
    .with_options(
        AudioSessionCategoryOptions::ALLOW_BLUETOOTH_HFP
            | AudioSessionCategoryOptions::DEFAULT_TO_SPEAKER,
    );
    let _ = session.configure(configuration);
    let _ = session.activate();
    let _ = session.deactivate(AudioSessionDeactivationOptions::NOTIFY_OTHERS);
}

#[cfg(not(target_os = "ios"))]
fn main() {}
