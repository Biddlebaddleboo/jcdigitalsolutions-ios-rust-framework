#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Main-thread AVPlayer controls, item-end observation, asynchronous seeking, and HDR eligibility for iOS."]

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

/// A per-player AVPlayer output control failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
#[cfg(any(target_os = "ios", test))]
pub enum IosPlayerOutputError {
    /// The AVPlayer output-control API is unavailable on this iOS version.
    NativeApiUnavailable,
    /// The requested volume is not finite.
    NonFiniteVolume,
    /// The requested volume is outside the inclusive 0.0 through 1.0 range.
    VolumeOutOfRange,
    /// AVPlayer returned a volume outside the finite 0.0 through 1.0 range.
    InvalidNativeVolume,
}

#[cfg(any(target_os = "ios", test))]
fn validate_requested_player_volume(volume: f32) -> Result<(), IosPlayerOutputError> {
    if !volume.is_finite() {
        return Err(IosPlayerOutputError::NonFiniteVolume);
    }
    if !(0.0..=1.0).contains(&volume) {
        return Err(IosPlayerOutputError::VolumeOutOfRange);
    }
    Ok(())
}

#[cfg(any(target_os = "ios", test))]
fn validate_native_player_volume(volume: f32) -> Result<f32, IosPlayerOutputError> {
    if volume.is_finite() && (0.0..=1.0).contains(&volume) {
        Ok(volume)
    } else {
        Err(IosPlayerOutputError::InvalidNativeVolume)
    }
}

#[cfg(target_os = "ios")]
fn player_output_controls_available() -> bool {
    objc2::available!(ios = 7.0, ..)
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
use std::sync::atomic::{AtomicBool, Ordering};
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

#[cfg(any(target_os = "ios", test))]
struct PlayerItemEndState(AtomicBool);

#[cfg(any(target_os = "ios", test))]
impl PlayerItemEndState {
    fn new() -> Self {
        Self(AtomicBool::new(false))
    }

    fn report_end(&self) {
        self.0.store(true, Ordering::Release);
    }

    fn has_reached_end(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[cfg(any(target_os = "ios", test))]
fn complete_item_end_callback(state: &PlayerItemEndState) {
    let _ = catch_unwind(AssertUnwindSafe(|| state.report_end()));
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

    /// Observes the AVPlayer item that is current at this call, or returns `None` if there is no
    /// current item.
    ///
    /// The observation retains that exact item and does not follow later player-item replacements.
    /// It reports only an end notification delivered after registration; it does not infer that an
    /// item already at its end has completed. The notification block runs synchronously on the
    /// posting thread, which AVFoundation may choose independently of the registration thread. The
    /// block only stores an atomic flag; [`IosPlayerItemEndObservation::has_reached_end`] is a
    /// pollable snapshot with no app callback or queue guarantee. Drop the returned observation to
    /// remove its notification registration.
    pub fn observe_current_item_end(
        &self,
        main_thread: objc2::MainThreadMarker,
    ) -> Option<IosPlayerItemEndObservation<'_>> {
        // SAFETY: The marker proves the call is on the main thread required by AVPlayer, and
        // `currentItem` returns an owned reference when an item exists.
        let item = unsafe { self.player.currentItem() }?;
        let center = objc2_foundation::NSNotificationCenter::defaultCenter();
        let state = Arc::new(PlayerItemEndState::new());
        let callback_state = Arc::clone(&state);
        let callback =
            move |_notification: core::ptr::NonNull<objc2_foundation::NSNotification>| {
                complete_item_end_callback(&callback_state);
            };
        assert_item_end_handler_send_sync(&callback);
        let block: block2::RcBlock<dyn Fn(core::ptr::NonNull<objc2_foundation::NSNotification>)> =
            block2::RcBlock::new(callback);
        // SAFETY: The notification name is AVFoundation's public item-end notification; the
        // observed object is this exact AVPlayerItem; `None` selects synchronous delivery on the
        // posting thread; and the block is Send + Sync, panic-contained, and captures only state.
        let token = unsafe {
            center.addObserverForName_object_queue_usingBlock(
                Some(objc2_av_foundation::AVPlayerItemDidPlayToEndTimeNotification),
                Some(&*item),
                None,
                &block,
            )
        };
        Some(IosPlayerItemEndObservation {
            center,
            token,
            _item: item,
            state,
            _player: self,
            _main_thread: main_thread,
            _not_send_or_sync: core::marker::PhantomData,
        })
    }

    /// Reads AVPlayer's per-player volume on the main thread.
    ///
    /// This is a relative audio-output level in the inclusive 0.0 through 1.0 range, not the
    /// device or system volume. Returns an error if AVPlayer reports a non-finite or out-of-range
    /// value. The property is available from iOS 7.0; older systems return
    /// [`IosPlayerOutputError::NativeApiUnavailable`].
    pub fn volume(
        &self,
        _main_thread: objc2::MainThreadMarker,
    ) -> Result<f32, IosPlayerOutputError> {
        if !player_output_controls_available() {
            return Err(IosPlayerOutputError::NativeApiUnavailable);
        }
        // SAFETY: the availability guard proves the getter exists and the marker proves this call
        // is on the main thread required by AVPlayer.
        validate_native_player_volume(unsafe { self.player.volume() })
    }

    /// Sets AVPlayer's per-player relative volume on the main thread.
    ///
    /// Accepts only finite values from 0.0 through 1.0, inclusive. Invalid values return an error
    /// without a native call or coercion. This does not set device or system volume. The property
    /// is available from iOS 7.0; older systems return
    /// [`IosPlayerOutputError::NativeApiUnavailable`].
    pub fn set_volume(
        &self,
        volume: f32,
        _main_thread: objc2::MainThreadMarker,
    ) -> Result<(), IosPlayerOutputError> {
        if !player_output_controls_available() {
            return Err(IosPlayerOutputError::NativeApiUnavailable);
        }
        validate_requested_player_volume(volume)?;
        // SAFETY: the availability guard proves the setter exists, the marker proves this call is
        // on the main thread, and the input was checked to be finite and within AVPlayer's range.
        unsafe { self.player.setVolume(volume) };
        Ok(())
    }

    /// Reads AVPlayer's per-player mute flag on the main thread.
    ///
    /// This flag applies only to the player; it does not report device mute state, route
    /// suppression, or audible output. The property is available from iOS 7.0; older systems
    /// return [`IosPlayerOutputError::NativeApiUnavailable`].
    pub fn is_muted(
        &self,
        _main_thread: objc2::MainThreadMarker,
    ) -> Result<bool, IosPlayerOutputError> {
        if !player_output_controls_available() {
            return Err(IosPlayerOutputError::NativeApiUnavailable);
        }
        // SAFETY: the availability guard proves the getter exists and the marker proves this call
        // is on the main thread required by AVPlayer.
        Ok(unsafe { self.player.isMuted() })
    }

    /// Sets AVPlayer's per-player mute flag on the main thread.
    ///
    /// This does not change device mute state or system volume. The property is available from
    /// iOS 7.0; older systems return [`IosPlayerOutputError::NativeApiUnavailable`].
    pub fn set_muted(
        &self,
        muted: bool,
        _main_thread: objc2::MainThreadMarker,
    ) -> Result<(), IosPlayerOutputError> {
        if !player_output_controls_available() {
            return Err(IosPlayerOutputError::NativeApiUnavailable);
        }
        // SAFETY: the availability guard proves the setter exists and the marker proves this call
        // is on the main thread required by AVPlayer.
        unsafe { self.player.setMuted(muted) };
        Ok(())
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
fn assert_item_end_handler_send_sync<T: Send + Sync>(_: &T) {}

/// A pollable observation of one AVPlayer item's end notification.
///
/// This value borrows its player, retains the specific item current when created, and removes its
/// Foundation notification registration on drop. It is neither `Send` nor `Sync`; read its atomic
/// snapshot on the main thread that created it. AVFoundation may post the notification on another
/// thread, but the native block only updates the atomic state and invokes no app code.
#[cfg(target_os = "ios")]
pub struct IosPlayerItemEndObservation<'a> {
    center: objc2::rc::Retained<objc2_foundation::NSNotificationCenter>,
    token:
        objc2::rc::Retained<objc2::runtime::ProtocolObject<dyn objc2::runtime::NSObjectProtocol>>,
    _item: objc2::rc::Retained<objc2_av_foundation::AVPlayerItem>,
    state: Arc<PlayerItemEndState>,
    _player: &'a IosFilePlayer,
    _main_thread: objc2::MainThreadMarker,
    _not_send_or_sync: core::marker::PhantomData<std::rc::Rc<()>>,
}

#[cfg(target_os = "ios")]
impl IosPlayerItemEndObservation<'_> {
    /// Returns whether the observed item posted its end notification after this observation began.
    ///
    /// The result is latched once true and is not reset by a seek or replay. It does not establish
    /// that media output was displayed or heard.
    pub fn has_reached_end(&self) -> bool {
        self.state.has_reached_end()
    }
}

#[cfg(target_os = "ios")]
impl Drop for IosPlayerItemEndObservation<'_> {
    fn drop(&mut self) {
        // SAFETY: `token` is the observer object returned by this center; removing it detaches the
        // copied block before the retained item and state are released. The token remains valid.
        let observer =
            <objc2::runtime::ProtocolObject<dyn objc2::runtime::NSObjectProtocol> as AsRef<
                objc2::runtime::AnyObject,
            >>::as_ref(&self.token);
        unsafe { self.center.removeObserver(observer) };
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
        IosPlayerOutputError, LocalFilePlayerError, PlayerItemEndState, SeekCompletion,
        SeekOutcome, complete_item_end_callback, complete_seek_callback, media_time_parts,
        seek_outcome, validate_local_file_path, validate_native_player_volume,
        validate_requested_player_volume,
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
    fn item_end_snapshot_starts_false_and_latches_after_notification() {
        let state = PlayerItemEndState::new();
        assert!(!state.has_reached_end());

        complete_item_end_callback(&state);
        complete_item_end_callback(&state);

        assert!(state.has_reached_end());
    }

    #[test]
    fn item_end_notification_can_update_state_from_another_thread() {
        let state = Arc::new(PlayerItemEndState::new());
        let callback_state = Arc::clone(&state);

        std::thread::spawn(move || complete_item_end_callback(&callback_state))
            .join()
            .unwrap();

        assert!(state.has_reached_end());
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

    #[test]
    fn player_volume_accepts_finite_closed_range_without_coercion() {
        for volume in [0.0, -0.0, 0.5, 1.0] {
            assert_eq!(validate_requested_player_volume(volume), Ok(()));
            assert_eq!(validate_native_player_volume(volume), Ok(volume));
        }
    }

    #[test]
    fn player_volume_rejects_non_finite_inputs() {
        for volume in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(
                validate_requested_player_volume(volume),
                Err(IosPlayerOutputError::NonFiniteVolume)
            );
            assert_eq!(
                validate_native_player_volume(volume),
                Err(IosPlayerOutputError::InvalidNativeVolume)
            );
        }
    }

    #[test]
    fn player_volume_rejects_out_of_range_inputs_without_clamping() {
        for volume in [-f32::EPSILON, 1.0 + f32::EPSILON] {
            assert_eq!(
                validate_requested_player_volume(volume),
                Err(IosPlayerOutputError::VolumeOutOfRange)
            );
            assert_eq!(
                validate_native_player_volume(volume),
                Err(IosPlayerOutputError::InvalidNativeVolume)
            );
        }
    }
}
