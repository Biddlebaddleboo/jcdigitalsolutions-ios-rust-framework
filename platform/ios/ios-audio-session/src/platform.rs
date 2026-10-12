use crate::{
    AudioSessionCategory, AudioSessionCategoryOptions, AudioSessionConfiguration,
    AudioSessionDeactivationOptions, AudioSessionMode, IosAudioSession, IosAudioSessionError,
    IosAudioSessionOperation,
};
use objc2_avf_audio::{
    AVAudioSession, AVAudioSessionCategory, AVAudioSessionCategoryAmbient,
    AVAudioSessionCategoryMultiRoute, AVAudioSessionCategoryOptions,
    AVAudioSessionCategoryPlayAndRecord, AVAudioSessionCategoryPlayback,
    AVAudioSessionCategoryRecord, AVAudioSessionCategorySoloAmbient, AVAudioSessionMode,
    AVAudioSessionModeDefault, AVAudioSessionModeMeasurement, AVAudioSessionModeMoviePlayback,
    AVAudioSessionModeSpokenAudio, AVAudioSessionModeVideoChat, AVAudioSessionModeVideoRecording,
    AVAudioSessionModeVoiceChat, AVAudioSessionSetActiveOptions,
};

fn category_value(category: AudioSessionCategory) -> Option<&'static AVAudioSessionCategory> {
    // SAFETY: these public AVFAudio category constants are weak-linked optional globals.
    unsafe {
        match category {
            AudioSessionCategory::Ambient => AVAudioSessionCategoryAmbient,
            AudioSessionCategory::SoloAmbient => AVAudioSessionCategorySoloAmbient,
            AudioSessionCategory::Playback => AVAudioSessionCategoryPlayback,
            AudioSessionCategory::Record => AVAudioSessionCategoryRecord,
            AudioSessionCategory::PlayAndRecord => AVAudioSessionCategoryPlayAndRecord,
            AudioSessionCategory::MultiRoute => AVAudioSessionCategoryMultiRoute,
        }
    }
}

fn mode_value(mode: AudioSessionMode) -> Option<&'static AVAudioSessionMode> {
    // SAFETY: these public AVFAudio mode constants are weak-linked optional globals.
    unsafe {
        match mode {
            AudioSessionMode::Default => AVAudioSessionModeDefault,
            AudioSessionMode::VoiceChat => AVAudioSessionModeVoiceChat,
            AudioSessionMode::VideoChat => AVAudioSessionModeVideoChat,
            AudioSessionMode::VideoRecording => AVAudioSessionModeVideoRecording,
            AudioSessionMode::Measurement => AVAudioSessionModeMeasurement,
            AudioSessionMode::MoviePlayback => AVAudioSessionModeMoviePlayback,
            AudioSessionMode::SpokenAudio => AVAudioSessionModeSpokenAudio,
        }
    }
}

fn category_options(options: AudioSessionCategoryOptions) -> AVAudioSessionCategoryOptions {
    AVAudioSessionCategoryOptions::from_bits_retain(options.native_bits() as _)
}

impl IosAudioSession {
    /// Applies the requested shared-session category, mode, and options.
    ///
    /// This is an explicit process-wide policy mutation. It does not activate the session,
    /// request microphone permission, select a route, or guarantee that the hardware can honor
    /// the selected combination.
    pub fn configure(
        &self,
        configuration: AudioSessionConfiguration,
    ) -> Result<(), IosAudioSessionError> {
        let category = category_value(configuration.category()).ok_or(
            IosAudioSessionError::NativeConstantUnavailable {
                operation: IosAudioSessionOperation::Configure,
            },
        )?;
        let mode = mode_value(configuration.mode()).ok_or(
            IosAudioSessionError::NativeConstantUnavailable {
                operation: IosAudioSessionOperation::Configure,
            },
        )?;
        // SAFETY: sharedInstance returns the documented process-wide audio-session singleton.
        let session = unsafe { AVAudioSession::sharedInstance() };
        // SAFETY: the typed arguments are public AVAudioSession constants/options; Apple validates
        // category/mode/option combinations and reports errors through NSError.
        unsafe {
            session.setCategory_mode_options_error(
                category,
                mode,
                category_options(configuration.options()),
            )
        }
        .map_err(|error| {
            crate::native_failure(IosAudioSessionOperation::Configure, error.code() as i64)
        })
    }

    /// Explicitly activates the shared session with no activation options.
    ///
    /// Apple implements this as a synchronous system operation; do not call it from a real-time
    /// audio callback. Permission prompts are not initiated by this method.
    pub fn activate(&self) -> Result<(), IosAudioSessionError> {
        // SAFETY: sharedInstance returns the documented process-wide audio-session singleton.
        let session = unsafe { AVAudioSession::sharedInstance() };
        // SAFETY: `true` and the empty options set are valid for setActive:withOptions:error:.
        unsafe {
            session.setActive_withOptions_error(true, AVAudioSessionSetActiveOptions::empty())
        }
        .map_err(|error| {
            crate::native_failure(IosAudioSessionOperation::Activate, error.code() as i64)
        })
    }

    /// Explicitly deactivates the shared session, optionally notifying interrupted apps.
    ///
    /// Stop or pause this app's audio I/O first. This method does not stop other capability
    /// objects or automatically run when a handle is dropped.
    pub fn deactivate(
        &self,
        options: AudioSessionDeactivationOptions,
    ) -> Result<(), IosAudioSessionError> {
        let native_options = if options.notify_others() {
            AVAudioSessionSetActiveOptions::NotifyOthersOnDeactivation
        } else {
            AVAudioSessionSetActiveOptions::empty()
        };
        // SAFETY: sharedInstance returns the documented process-wide audio-session singleton.
        let session = unsafe { AVAudioSession::sharedInstance() };
        // SAFETY: false deactivates the session; notifyOthersOnDeactivation is valid only here.
        unsafe { session.setActive_withOptions_error(false, native_options) }.map_err(|error| {
            crate::native_failure(IosAudioSessionOperation::Deactivate, error.code() as i64)
        })
    }
}
