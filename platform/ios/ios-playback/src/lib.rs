#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Main-thread AVPlayer playback controls, asynchronous seeking, and HDR eligibility for iOS."]

#[cfg(any(target_os = "ios", test))]
use framework_media::MediaTime;

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

#[cfg(target_os = "ios")]
use core::future::Future;
#[cfg(target_os = "ios")]
use core::pin::Pin;
#[cfg(any(target_os = "ios", test))]
use core::task::{Context, Poll, Waker};
#[cfg(any(target_os = "ios", test))]
use std::panic::{AssertUnwindSafe, catch_unwind};
#[cfg(target_os = "ios")]
use std::sync::Arc;
#[cfg(any(target_os = "ios", test))]
use std::sync::{Mutex, MutexGuard};

#[cfg(any(target_os = "ios", test))]
fn media_time_parts(time: MediaTime) -> (i64, i32) {
    (time.value(), time.timescale())
}

#[cfg(any(target_os = "ios", test))]
fn seek_outcome(finished: bool) -> SeekOutcome {
    if finished {
        SeekOutcome::Completed
    } else {
        SeekOutcome::NotCompleted
    }
}

#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// The completion status reported by AVPlayer for one seek request.
pub enum SeekOutcome {
    /// AVPlayer reports that this seek completed without interruption.
    Completed,
    /// AVPlayer reports that this seek did not complete; it does not distinguish no current item
    /// from interruption or supersession by another operation.
    NotCompleted,
}

#[cfg(any(target_os = "ios", test))]
struct SeekCompletion {
    state: Mutex<SeekState>,
}

#[cfg(any(target_os = "ios", test))]
struct SeekState {
    completed: bool,
    detached: bool,
    outcome: Option<SeekOutcome>,
    waker: Option<Waker>,
}

#[cfg(any(target_os = "ios", test))]
impl SeekCompletion {
    fn new() -> Self {
        Self {
            state: Mutex::new(SeekState {
                completed: false,
                detached: false,
                outcome: None,
                waker: None,
            }),
        }
    }

    fn complete(&self, outcome: SeekOutcome) {
        let waker = {
            let mut state = self.lock();
            if state.completed {
                return;
            }
            state.completed = true;
            if state.detached {
                None
            } else {
                state.outcome = Some(outcome);
                state.waker.take()
            }
        };
        if let Some(waker) = waker {
            let _ = catch_unwind(AssertUnwindSafe(|| waker.wake()));
        }
    }

    fn poll(&self, context: &mut Context<'_>) -> Poll<SeekOutcome> {
        let mut state = self.lock();
        if let Some(outcome) = state.outcome.take() {
            return Poll::Ready(outcome);
        }
        if !state.completed
            && !state.detached
            && state
                .waker
                .as_ref()
                .is_none_or(|waker| !waker.will_wake(context.waker()))
        {
            state.waker = Some(context.waker().clone());
        }
        Poll::Pending
    }

    fn detach(&self) {
        let mut state = self.lock();
        state.detached = true;
        state.outcome = None;
        state.waker = None;
    }

    fn lock(&self) -> MutexGuard<'_, SeekState> {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

#[cfg(any(target_os = "ios", test))]
fn complete_seek_callback(cell: &SeekCompletion, finished: bool) {
    let _ = catch_unwind(AssertUnwindSafe(|| cell.complete(seek_outcome(finished))));
}

#[cfg(target_os = "ios")]
fn cm_time_from_media_time(time: MediaTime) -> objc2_core_media::CMTime {
    let (value, timescale) = media_time_parts(time);
    objc2_core_media::CMTime {
        value,
        timescale,
        flags: objc2_core_media::CMTimeFlags::Valid,
        epoch: 0,
    }
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

    /// Starts a nonblocking seek to an exact finite rational media time on the main thread.
    ///
    /// The returned future borrows this player and retains the main-thread marker until it is
    /// dropped. AVPlayer may complete a prior concurrent seek as `NotCompleted` when superseded.
    /// Dropping the future detaches Rust result interest but does not request native cancellation.
    /// A `Completed` result is only AVPlayer's uninterrupted seek-completion report; it does not
    /// prove that a frame was decoded or displayed, that playback is active, or that output began.
    pub fn seek_to(
        &self,
        time: MediaTime,
        main_thread: objc2::MainThreadMarker,
    ) -> IosPlayerSeekFuture<'_> {
        let cell = Arc::new(SeekCompletion::new());
        let callback_cell = Arc::clone(&cell);
        let callback = move |finished: objc2::runtime::Bool| {
            complete_seek_callback(&callback_cell, finished.as_bool());
        };
        assert_seek_handler_send_sync(&callback);
        let completion_handler: block2::RcBlock<dyn Fn(objc2::runtime::Bool)> =
            block2::RcBlock::new(callback);
        let time = cm_time_from_media_time(time);
        // SAFETY: The supplied marker proves this call runs on the main thread. The target
        // binding requires a sendable callback; the captured Arc is Send + Sync, and callback
        // panics are contained. The value-built CMTime is finite with a positive timescale.
        unsafe {
            self.player
                .seekToTime_completionHandler(time, &completion_handler)
        };
        IosPlayerSeekFuture {
            cell,
            _player: self,
            _main_thread: main_thread,
            _not_send_or_sync: core::marker::PhantomData,
            finished: false,
        }
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
fn assert_seek_handler_send_sync<T: Send + Sync>(_: &T) {}

/// A nonblocking AVPlayer seek request tied to the player and main thread that started it.
///
/// Dropping this future detaches its Rust completion state. AVPlayer is not asked to cancel the
/// seek, and a later native callback safely releases its owned completion state.
#[must_use = "await the seek future to observe AVPlayer's completion result"]
#[cfg(target_os = "ios")]
pub struct IosPlayerSeekFuture<'a> {
    cell: Arc<SeekCompletion>,
    _player: &'a IosFilePlayer,
    _main_thread: objc2::MainThreadMarker,
    _not_send_or_sync: core::marker::PhantomData<std::rc::Rc<()>>,
    finished: bool,
}

#[cfg(target_os = "ios")]
impl Future for IosPlayerSeekFuture<'_> {
    type Output = SeekOutcome;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this.finished {
            return Poll::Pending;
        }
        match this.cell.poll(context) {
            Poll::Ready(outcome) => {
                this.finished = true;
                Poll::Ready(outcome)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(target_os = "ios")]
impl Drop for IosPlayerSeekFuture<'_> {
    fn drop(&mut self) {
        self.cell.detach();
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
    use super::{
        LocalFilePlayerError, SeekCompletion, SeekOutcome, complete_seek_callback,
        media_time_parts, seek_outcome, validate_local_file_path,
    };
    use core::task::{Context, Poll, Waker};
    use framework_media::MediaTime;
    use std::io::ErrorKind;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::task::Wake;

    struct WakeCount(AtomicUsize);

    impl Wake for WakeCount {
        fn wake(self: Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }

        fn wake_by_ref(self: &Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    struct PanicWake;

    impl Wake for PanicWake {
        fn wake(self: Arc<Self>) {
            panic!("test waker panic");
        }

        fn wake_by_ref(self: &Arc<Self>) {
            panic!("test waker panic");
        }
    }

    fn poll_context(waker: &Waker) -> Context<'_> {
        Context::from_waker(waker)
    }

    #[test]
    fn seek_target_preserves_signed_rational_fields_exactly() {
        for (value, timescale) in [
            (0, 1),
            (3, 2),
            (6, 4),
            (-7, 11),
            (i64::MIN, i32::MAX),
            (i64::MAX, 1),
        ] {
            assert_eq!(
                media_time_parts(MediaTime::new(value, timescale).unwrap()),
                (value, timescale)
            );
        }
    }

    #[test]
    fn seek_completion_maps_native_boolean_without_claiming_its_cause() {
        assert_eq!(seek_outcome(true), SeekOutcome::Completed);
        assert_eq!(seek_outcome(false), SeekOutcome::NotCompleted);
    }

    #[test]
    fn seek_completion_settles_once_and_wakes_once() {
        let cell = SeekCompletion::new();
        let wake_count = Arc::new(WakeCount(AtomicUsize::new(0)));
        let waker = Waker::from(Arc::clone(&wake_count));
        let mut context = poll_context(&waker);

        assert_eq!(cell.poll(&mut context), Poll::Pending);
        cell.complete(SeekOutcome::NotCompleted);
        cell.complete(SeekOutcome::Completed);

        assert_eq!(wake_count.0.load(Ordering::SeqCst), 1);
        assert_eq!(
            cell.poll(&mut context),
            Poll::Ready(SeekOutcome::NotCompleted)
        );
        assert_eq!(cell.poll(&mut context), Poll::Pending);
    }

    #[test]
    fn seek_completion_accepts_callback_from_another_thread() {
        let cell = Arc::new(SeekCompletion::new());
        let wake_count = Arc::new(WakeCount(AtomicUsize::new(0)));
        let waker = Waker::from(Arc::clone(&wake_count));
        let mut context = poll_context(&waker);

        assert_eq!(cell.poll(&mut context), Poll::Pending);
        let callback_cell = Arc::clone(&cell);
        std::thread::spawn(move || callback_cell.complete(SeekOutcome::Completed))
            .join()
            .unwrap();

        assert_eq!(wake_count.0.load(Ordering::SeqCst), 1);
        assert_eq!(cell.poll(&mut context), Poll::Ready(SeekOutcome::Completed));
    }

    #[test]
    fn seek_callback_contains_waker_panic_and_keeps_completion() {
        let cell = SeekCompletion::new();
        let waker = Waker::from(Arc::new(PanicWake));
        let mut context = poll_context(&waker);

        assert_eq!(cell.poll(&mut context), Poll::Pending);
        complete_seek_callback(&cell, true);
        assert_eq!(cell.poll(&mut context), Poll::Ready(SeekOutcome::Completed));
    }

    #[test]
    fn dropped_seek_future_detaches_waker_and_ignores_late_result() {
        let cell = SeekCompletion::new();
        let wake_count = Arc::new(WakeCount(AtomicUsize::new(0)));
        let waker = Waker::from(Arc::clone(&wake_count));
        let mut context = poll_context(&waker);

        assert_eq!(cell.poll(&mut context), Poll::Pending);
        cell.detach();
        cell.complete(SeekOutcome::Completed);

        assert_eq!(wake_count.0.load(Ordering::SeqCst), 0);
        assert_eq!(cell.poll(&mut context), Poll::Pending);
    }

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
