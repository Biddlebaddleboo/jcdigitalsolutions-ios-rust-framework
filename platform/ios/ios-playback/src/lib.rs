#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Main-thread AVPlayer playback controls and HDR eligibility for iOS."]

#[cfg(any(target_os = "ios", test))]
use std::fs::File;
#[cfg(any(target_os = "ios", test))]
use std::io::ErrorKind;
#[cfg(any(target_os = "ios", test))]
use std::path::Path;

/// A local-file player path failed preflight validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
#[cfg(any(target_os = "ios", test))]
pub enum LocalFilePlayerError {
    /// The path is not absolute.
    NotAbsolute,
    /// The path contains an embedded NUL character.
    EmbeddedNul,
    /// The path could not be opened or inspected; this stores only its stable OS error kind.
    FileAccess(ErrorKind),
    /// The opened path does not identify a regular file.
    NotRegularFile,
}

#[cfg(any(target_os = "ios", test))]
fn validate_local_file_path(path: &str) -> Result<(), LocalFilePlayerError> {
    if !Path::new(path).is_absolute() {
        return Err(LocalFilePlayerError::NotAbsolute);
    }
    if path.contains('\0') {
        return Err(LocalFilePlayerError::EmbeddedNul);
    }
    let file = File::open(path).map_err(|error| LocalFilePlayerError::FileAccess(error.kind()))?;
    let metadata = file
        .metadata()
        .map_err(|error| LocalFilePlayerError::FileAccess(error.kind()))?;
    if !metadata.is_file() {
        return Err(LocalFilePlayerError::NotRegularFile);
    }
    Ok(())
}

/// A local file attached to one main-thread AVPlayer.
#[cfg(target_os = "ios")]
pub struct IosFilePlayer {
    player: objc2::rc::Retained<objc2_av_foundation::AVPlayer>,
}

#[cfg(target_os = "ios")]
impl IosFilePlayer {
    /// Creates an AVPlayer for an existing absolute UTF-8 local media file on the main thread.
    ///
    /// This preflight opens the path to verify it is readable and regular, then gives its unchanged
    /// path to AVPlayer. AVFoundation asynchronously evaluates asset readiness and supported media.
    pub fn from_local_file(
        path: &str,
        main_thread: objc2::MainThreadMarker,
    ) -> Result<Self, LocalFilePlayerError> {
        validate_local_file_path(path)?;
        let path = objc2_foundation::NSString::from_str(path);
        let url = objc2_foundation::NSURL::fileURLWithPath(&path);
        // SAFETY: AVPlayer and the binding require the supplied main-thread marker; the URL is a
        // Foundation file URL created from the validated, unchanged absolute path.
        let player = unsafe { objc2_av_foundation::AVPlayer::playerWithURL(&url, main_thread) };
        Ok(Self { player })
    }

    /// Signals a desire to play at AVPlayer's default rate on the main thread.
    ///
    /// This does not guarantee that the asset is ready or that audio or video output occurs.
    pub fn play(&self, _main_thread: objc2::MainThreadMarker) {
        // SAFETY: The caller supplies a main-thread marker; play requires the main thread before
        // iOS 16 and AVPlayer is main-thread-bound by its Objective-C type.
        unsafe { self.player.play() };
    }

    /// Pauses this player on the main thread.
    pub fn pause(&self, _main_thread: objc2::MainThreadMarker) {
        // SAFETY: The caller supplies a main-thread marker; AVPlayer is main-thread-bound by its
        // Objective-C type.
        unsafe { self.player.pause() };
    }

    /// Borrows the underlying AVPlayer for native integrations such as AVPlayerLayer.
    pub fn as_av_player(
        &self,
        _main_thread: objc2::MainThreadMarker,
    ) -> &objc2_av_foundation::AVPlayer {
        &self.player
    }
}

#[cfg(target_os = "ios")]
use framework_audio::HdrPlaybackEligibility;

/// Queries system HDR playback eligibility on the main thread.
///
/// Returns `None` off the main thread and on non-iOS targets. `Some(false)` is a system result,
/// not a claim that HDR playback is impossible for every asset or configuration. Returns `None`
/// below iOS 13.4 because the AVPlayer property is not available there.
#[cfg(target_os = "ios")]
pub fn hdr_playback_eligibility() -> Option<HdrPlaybackEligibility> {
    if !objc2::available!(ios = 13.4, ..) {
        return None;
    }
    let main_thread = objc2::MainThreadMarker::new()?;
    // SAFETY: The runtime guard proves this property is available, the marker proves this call is
    // on the main thread, and the class property returns a scalar without retaining an object.
    let eligible = unsafe { objc2_av_foundation::AVPlayer::eligibleForHDRPlayback(main_thread) };
    Some(HdrPlaybackEligibility::new(eligible))
}

/// Reports that this iOS-only query is unavailable on non-iOS targets.
#[cfg(not(target_os = "ios"))]
pub fn hdr_playback_eligibility() -> Option<framework_audio::HdrPlaybackEligibility> {
    None
}

#[cfg(test)]
mod tests {
    use super::{LocalFilePlayerError, validate_local_file_path};
    use std::io::ErrorKind;
    use std::path::PathBuf;

    #[test]
    fn local_file_path_accepts_absolute_regular_file() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        assert!(validate_local_file_path(path.to_str().unwrap()).is_ok());
    }

    #[test]
    fn local_file_path_accepts_unicode_utf8() {
        let path =
            std::env::temp_dir().join(format!("ios-playback-b463-媒体-{}.mov", std::process::id()));
        std::fs::write(&path, []).unwrap();
        assert!(validate_local_file_path(path.to_str().unwrap()).is_ok());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn local_file_path_rejects_relative_path() {
        assert_eq!(
            validate_local_file_path("media.mov"),
            Err(LocalFilePlayerError::NotAbsolute)
        );
    }

    #[test]
    fn local_file_path_rejects_embedded_nul() {
        assert_eq!(
            validate_local_file_path("/tmp/media\0.mov"),
            Err(LocalFilePlayerError::EmbeddedNul)
        );
    }

    #[test]
    fn local_file_path_rejects_directory() {
        let path = env!("CARGO_MANIFEST_DIR");
        assert_eq!(
            validate_local_file_path(path),
            Err(LocalFilePlayerError::NotRegularFile)
        );
    }

    #[test]
    fn local_file_path_reports_missing_file() {
        let path =
            std::env::temp_dir().join(format!("ios-playback-b463-missing-{}", std::process::id()));
        let _ = std::fs::remove_file(&path);
        assert_eq!(
            validate_local_file_path(path.to_str().unwrap()),
            Err(LocalFilePlayerError::FileAccess(ErrorKind::NotFound))
        );
    }
}
