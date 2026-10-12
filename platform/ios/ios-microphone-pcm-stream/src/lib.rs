#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Authorized live iOS microphone PCM input, input-port snapshots, and preferred-input selection"]

#[cfg(any(target_os = "ios", test))]
use core::cell::UnsafeCell;
#[cfg(any(target_os = "ios", test))]
use core::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, AtomicU64, Ordering};

pub use framework_audio::{PcmInputFormat, PcmInputStream};
#[cfg(any(target_os = "ios", test))]
use framework_media_authorization::MediaAuthorizationStatus;

#[cfg(target_os = "ios")]
use block2::RcBlock;
#[cfg(target_os = "ios")]
use core::marker::PhantomData;
#[cfg(target_os = "ios")]
use core::ptr::NonNull;
#[cfg(target_os = "ios")]
use framework_media_authorization::MediaAuthorization;
#[cfg(target_os = "ios")]
use ios_media_authorization::IosMediaAuthorization;
#[cfg(target_os = "ios")]
use objc2::runtime::{AnyObject, ProtocolObject};
#[cfg(target_os = "ios")]
use objc2::{AnyThread, rc::Retained};
#[cfg(target_os = "ios")]
use objc2_avf_audio::{
    AVAudioCommonFormat, AVAudioEngine, AVAudioFormat, AVAudioInputNode, AVAudioNodeTapBlock,
    AVAudioPCMBuffer, AVAudioSession, AVAudioSessionInterruptionNotification,
    AVAudioSessionRouteChangeNotification, AVAudioTime,
};
#[cfg(target_os = "ios")]
use objc2_foundation::{
    NSDictionary, NSNotification, NSNotificationCenter, NSNotificationName, NSNumber,
    NSObjectProtocol,
};
#[cfg(any(target_os = "ios", test))]
use std::panic::{AssertUnwindSafe, catch_unwind};
#[cfg(target_os = "ios")]
use std::rc::Rc;
#[cfg(target_os = "ios")]
use std::sync::Arc;
#[cfg(any(target_os = "ios", test))]
use std::sync::Weak;

#[cfg(any(target_os = "ios", test))]
const MAX_CAPACITY_FRAMES: u32 = 1_048_576;

/// Errors from format, ring, authorization, or native engine setup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MicrophoneStreamError {
    /// The microphone is not currently authorized.
    AuthorizationNotGranted,
    /// The requested iOS runtime is older than the AVAudioEngine API.
    UnsupportedOperatingSystem,
    /// The ring capacity is zero or exceeds the bounded memory limit.
    InvalidCapacity,
    /// The ring storage allocation failed.
    AllocationFailed,
    /// No usable microphone input format is available.
    InputUnavailable,
    /// The native input sample rate is not finite and positive.
    InvalidSampleRate,
    /// The native input is not Float32 PCM.
    UnsupportedSampleFormat,
    /// The native input does not have one or two channels.
    UnsupportedChannelCount,
    /// The native Float32 channel stride does not match its layout.
    UnsupportedChannelLayout,
    /// The ring is too small for the requested tap buffer size.
    CapacityTooSmall,
    /// AVAudioEngine rejected the start request.
    EngineStartFailed,
    /// The stream has already had a start attempt or has stopped.
    AlreadyStarted,
    /// No successful start has established an input format.
    NotStarted,
    /// The read slice length is not a whole number of interleaved frames.
    InvalidOutputLength,
    /// A tap callback reported a changed or unsupported input layout.
    InputFormatChanged,
}

/// One owned iOS audio input port description from a point-in-time session snapshot.
///
/// Metadata is converted from `NSString` to Rust UTF-8 with the pinned Foundation binding. The
/// conversion does not normalize well-formed Unicode text, but it does not define lossless handling
/// for malformed UTF-16; Apple-provided port metadata is expected to be valid Unicode.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg(any(target_os = "ios", test))]
pub struct IosAudioInputPort {
    port_type: String,
    name: String,
    uid: String,
}

#[cfg(any(target_os = "ios", test))]
impl IosAudioInputPort {
    /// Returns the system port-type string copied as UTF-8.
    ///
    /// Well-formed Unicode text is not normalized.
    pub fn port_type(&self) -> &str {
        &self.port_type
    }

    /// Returns the descriptive name reported by the system.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the system-assigned identifier for this port snapshot.
    ///
    /// The identifier is not promised to persist across reconnects, app launches, hardware changes,
    /// or operating-system updates.
    pub fn uid(&self) -> &str {
        &self.uid
    }
}

/// Failure to query input ports on an unsupported iOS version.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg(target_os = "ios")]
pub enum MicrophoneInputQueryError {
    /// `AVAudioSession.availableInputs` requires iOS 7.0 or newer.
    UnsupportedOperatingSystem,
}

/// Failure to select an input from the host session's currently available ports.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg(any(target_os = "ios", test))]
pub enum MicrophoneInputSelectionError {
    /// `AVAudioSession.setPreferredInput:error:` requires iOS 7.0 or newer.
    UnsupportedOperatingSystem,
    /// The requested system-assigned port UID is empty.
    InvalidUid,
    /// No currently available input port has the requested UID.
    InputNotAvailable,
    /// `AVAudioSession.setPreferredInput:error:` returned a native error.
    NativeFailure,
}

/// Failure to subscribe to public AVAudioSession notifications.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg(any(target_os = "ios", test))]
pub enum IosAudioSessionEventMonitorError {
    /// The route-change and interruption notifications require iOS 6.0 or newer.
    UnsupportedOperatingSystem,
}

#[cfg(any(target_os = "ios", test))]
const ROUTE_CHANGE_EVENT: u8 = 1 << 0;
#[cfg(any(target_os = "ios", test))]
const INTERRUPTION_BEGAN_EVENT: u8 = 1 << 1;
#[cfg(any(target_os = "ios", test))]
const INTERRUPTION_ENDED_EVENT: u8 = 1 << 2;
#[cfg(any(target_os = "ios", test))]
const INTERRUPTION_UNCLASSIFIED_EVENT: u8 = 1 << 3;

/// Notification categories observed since the last audio-session event drain.
///
/// Each flag is coalesced, not counted. This value does not preserve notification order, route
/// details, or interruption resumption options. Both interruption-type flags may be set when
/// opposite events occur before the next drain.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg(any(target_os = "ios", test))]
pub struct IosAudioSessionEventFlags {
    route_change: bool,
    interruption_began: bool,
    interruption_ended: bool,
    interruption_unclassified: bool,
}

#[cfg(any(target_os = "ios", test))]
impl IosAudioSessionEventFlags {
    /// Returns whether at least one route-change notification occurred before this drain.
    pub const fn route_change(self) -> bool {
        self.route_change
    }

    /// Returns whether at least one interruption notification occurred before this drain.
    pub const fn interruption(self) -> bool {
        self.interruption_began || self.interruption_ended || self.interruption_unclassified
    }

    /// Returns whether a classified interruption-began notification occurred before this drain.
    pub const fn interruption_began(self) -> bool {
        self.interruption_began
    }

    /// Returns whether a classified interruption-ended notification occurred before this drain.
    pub const fn interruption_ended(self) -> bool {
        self.interruption_ended
    }

    /// Returns whether an interruption notification had a missing or unknown type before this drain.
    pub const fn interruption_unclassified(self) -> bool {
        self.interruption_unclassified
    }

    /// Returns whether neither observed notification category occurred before this drain.
    pub const fn is_empty(self) -> bool {
        !self.route_change && !self.interruption()
    }
}

#[derive(Default)]
#[cfg(any(target_os = "ios", test))]
struct PendingAudioSessionEvents(AtomicU8);

#[cfg(any(target_os = "ios", test))]
impl PendingAudioSessionEvents {
    fn record(&self, event: u8) {
        self.0.fetch_or(event, Ordering::Relaxed);
    }

    fn drain(&self) -> IosAudioSessionEventFlags {
        let events = self.0.swap(0, Ordering::Relaxed);
        IosAudioSessionEventFlags {
            route_change: events & ROUTE_CHANGE_EVENT != 0,
            interruption_began: events & INTERRUPTION_BEGAN_EVENT != 0,
            interruption_ended: events & INTERRUPTION_ENDED_EVENT != 0,
            interruption_unclassified: events & INTERRUPTION_UNCLASSIFIED_EVENT != 0,
        }
    }

    fn record_interruption(&self, interruption_type: Option<isize>) {
        let event = match interruption_type {
            Some(1) => INTERRUPTION_BEGAN_EVENT,
            Some(0) => INTERRUPTION_ENDED_EVENT,
            _ => INTERRUPTION_UNCLASSIFIED_EVENT,
        };
        self.record(event);
    }
}

#[cfg(any(target_os = "ios", test))]
fn make_audio_event_callback(
    events: Weak<PendingAudioSessionEvents>,
    event: u8,
) -> impl Fn() + Send + Sync + 'static {
    move || {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            if let Some(events) = events.upgrade() {
                events.record(event);
            }
        }));
    }
}

#[cfg(any(target_os = "ios", test))]
trait ObserverTokenRemover<T> {
    fn remove(&self, token: &T);
}

#[cfg(any(target_os = "ios", test))]
struct ObserverTokenOwner<C, T, const N: usize>
where
    C: ObserverTokenRemover<T>,
{
    remover: C,
    tokens: [Option<T>; N],
}

#[cfg(any(target_os = "ios", test))]
impl<C, T, const N: usize> ObserverTokenOwner<C, T, N>
where
    C: ObserverTokenRemover<T>,
{
    fn new(remover: C) -> Self {
        Self {
            remover,
            tokens: [const { None }; N],
        }
    }

    fn register(&mut self, token: T) {
        let Some(slot) = self.tokens.iter_mut().find(|slot| slot.is_none()) else {
            panic!("audio session observer capacity exceeded");
        };
        *slot = Some(token);
    }

    #[cfg(target_os = "ios")]
    fn remover(&self) -> &C {
        &self.remover
    }
}

#[cfg(any(target_os = "ios", test))]
impl<C, T, const N: usize> Drop for ObserverTokenOwner<C, T, N>
where
    C: ObserverTokenRemover<T>,
{
    fn drop(&mut self) {
        for token in self.tokens.iter().flatten() {
            self.remover.remove(token);
        }
    }
}

/// Reads one owned snapshot of the host-configured session's available input ports.
///
/// The host owns audio-session category, mode, activation, and route policy. This function only
/// reads `AVAudioSession.availableInputs`; a category that does not support input, such as playback,
/// can report no ports. The function does not request microphone permission, read authorization,
/// start audio I/O, mutate session state, or select a route. The returned ports are those currently
/// reported as available for routing under the session's category and mode, not all physical
/// devices, the active route, or a guarantee that capture can start. This API is available from
/// iOS 7.0.
#[cfg(target_os = "ios")]
pub fn available_input_ports() -> Result<Vec<IosAudioInputPort>, MicrophoneInputQueryError> {
    if !objc2::available!(ios = 7.0, ..) {
        return Err(MicrophoneInputQueryError::UnsupportedOperatingSystem);
    }

    // SAFETY: sharedInstance returns the documented process-wide AVAudioSession singleton.
    let session = unsafe { AVAudioSession::sharedInstance() };
    // SAFETY: availableInputs is a read-only property on the retained shared session.
    let ports = unsafe { session.availableInputs() };
    let metadata = ports.map(|ports| {
        let count = ports.count();
        (0..count).map(move |index| {
            let port = ports.objectAtIndex(index);
            // SAFETY: These read-only getters copy metadata from each retained port description.
            let port_type = unsafe { port.portType() };
            let name = unsafe { port.portName() };
            let uid = unsafe { port.UID() };
            (port_type.to_string(), name.to_string(), uid.to_string())
        })
    });
    Ok(collect_input_port_snapshot(metadata))
}

/// Requests the preferred route for a currently available audio input by its system-assigned UID.
///
/// The UID must match an entry in the host session's current `available_input_ports()` snapshot.
/// This calls the public `AVAudioSession.setPreferredInput:error:` API, available from iOS 7.0.
/// Success means the preference request was accepted; it does not prove that the active route has
/// changed or that microphone capture can start. Apple notes that selecting an input already on
/// the current route has no effect. The host remains responsible for session category, mode,
/// activation, microphone authorization, and any route or interruption observation. This function
/// does not configure or activate the session, request permission, start I/O, clear the preferred
/// input, or register observers.
#[cfg(target_os = "ios")]
pub fn select_input_port(uid: &str) -> Result<(), MicrophoneInputSelectionError> {
    if uid.is_empty() {
        return Err(MicrophoneInputSelectionError::InvalidUid);
    }
    if !objc2::available!(ios = 7.0, ..) {
        return Err(MicrophoneInputSelectionError::UnsupportedOperatingSystem);
    }

    // SAFETY: sharedInstance returns the documented process-wide AVAudioSession singleton.
    let session = unsafe { AVAudioSession::sharedInstance() };
    // SAFETY: availableInputs reads the retained session's current route-eligible input list.
    select_available_input(
        (unsafe { session.availableInputs() })
            .map(|ports| (0..ports.count()).map(move |index| ports.objectAtIndex(index))),
        uid,
        |port| {
            // SAFETY: UID is a read-only property on each retained public port description.
            unsafe { port.UID() }.to_string()
        },
        |port| {
            // SAFETY: `port` is a retained entry from this session's current availableInputs list.
            unsafe { session.setPreferredInput_error(Some(&port)) }
        },
    )
}

#[cfg(any(target_os = "ios", test))]
fn select_available_input<P, I, U, S, E>(
    ports: Option<I>,
    requested_uid: &str,
    mut port_uid: U,
    set_preferred_input: S,
) -> Result<(), MicrophoneInputSelectionError>
where
    I: IntoIterator<Item = P>,
    U: FnMut(&P) -> String,
    S: FnOnce(P) -> Result<(), E>,
{
    if requested_uid.is_empty() {
        return Err(MicrophoneInputSelectionError::InvalidUid);
    }
    let port = ports
        .into_iter()
        .flatten()
        .find(|port| port_uid(port) == requested_uid)
        .ok_or(MicrophoneInputSelectionError::InputNotAvailable)?;
    set_preferred_input(port).map_err(|_| MicrophoneInputSelectionError::NativeFailure)
}

#[cfg(any(target_os = "ios", test))]
fn collect_input_port_snapshot<I>(metadata: Option<I>) -> Vec<IosAudioInputPort>
where
    I: IntoIterator<Item = (String, String, String)>,
{
    metadata
        .into_iter()
        .flatten()
        .map(|(port_type, name, uid)| IosAudioInputPort {
            port_type,
            name,
            uid,
        })
        .collect()
}

#[cfg(target_os = "ios")]
type ObserverToken = Retained<ProtocolObject<dyn NSObjectProtocol>>;

#[cfg(target_os = "ios")]
impl ObserverTokenRemover<ObserverToken> for Retained<NSNotificationCenter> {
    fn remove(&self, token: &ObserverToken) {
        let observer: &AnyObject = AsRef::<AnyObject>::as_ref(&**token);
        // SAFETY: the token came from this notification center and is removed exactly once.
        unsafe { self.removeObserver(observer) };
    }
}

#[cfg(target_os = "ios")]
fn add_audio_event_observer(
    center: &NSNotificationCenter,
    name: &'static NSNotificationName,
    events: &Arc<PendingAudioSessionEvents>,
    event: u8,
) -> ObserverToken {
    let callback = make_audio_event_callback(Arc::downgrade(events), event);
    let block: RcBlock<dyn Fn(NonNull<NSNotification>)> =
        RcBlock::new(move |_notification| callback());
    // SAFETY: `name` is one of AVFAudio's public process-lifetime notification names. With no
    // operation queue, Foundation invokes the block on the posting thread; the block captures
    // only weak atomic event state and does no user work or AVAudioSession mutation. The center
    // returns the observer token, whose owner removes it on drop.
    unsafe { center.addObserverForName_object_queue_usingBlock(Some(name), None, None, &block) }
}

#[cfg(target_os = "ios")]
fn interruption_type(notification: &NSNotification) -> Option<isize> {
    // SAFETY: AVFAudio exposes this immutable public NSString notification-key constant.
    let key = unsafe { objc2_avf_audio::AVAudioSessionInterruptionTypeKey }?;
    let user_info: Retained<NSDictionary> = notification.userInfo()?;
    let value = user_info.objectForKey(key)?;
    value
        .downcast::<NSNumber>()
        .ok()
        .map(|number| number.integerValue())
}

#[cfg(target_os = "ios")]
fn add_interruption_observer(
    center: &NSNotificationCenter,
    name: &'static NSNotificationName,
    events: &Arc<PendingAudioSessionEvents>,
) -> ObserverToken {
    let events = Arc::downgrade(events);
    let block: RcBlock<dyn Fn(NonNull<NSNotification>)> =
        RcBlock::new(move |notification: NonNull<NSNotification>| {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                if let Some(events) = events.upgrade() {
                    // SAFETY: NotificationCenter supplies a live notification for this block call;
                    // the borrowed payload is parsed immediately and is never retained.
                    let interruption_type = unsafe { interruption_type(notification.as_ref()) };
                    events.record_interruption(interruption_type);
                }
            }));
        });
    // SAFETY: `name` is AVFAudio's public process-lifetime interruption name. With no operation
    // queue, Foundation invokes the block on the posting thread; it only reads the documented
    // interruption-type NSNumber and updates weakly held atomic state. The owner removes its token.
    unsafe { center.addObserverForName_object_queue_usingBlock(Some(name), None, None, &block) }
}

/// Observes host audio-session route and interruption notification categories without changing
/// session policy.
///
/// The monitor subscribes to public AVAudioSession notifications from iOS 6.0. It does not
/// configure, activate, or deactivate the session, request permission, start audio I/O, parse route
/// details, or run user closures. The interruption observer reads only the documented interruption
/// type NSNumber; missing, mistyped, or unknown values are reported as unclassified. Foundation
/// invokes its private observer blocks on the posting thread; each block only updates atomic flags.
/// Repeated notifications of each type coalesce until `drain`; the snapshot preserves neither
/// count nor ordering. Opposite interruption flags may both be set. An atomic drain separates
/// batches: an event concurrent with the swap belongs either to the returned snapshot or the next
/// one. The state is fixed-size and cannot overflow. Observer tokens are removed on drop.
#[cfg(target_os = "ios")]
pub struct IosAudioSessionEventMonitor {
    _observers: ObserverTokenOwner<Retained<NSNotificationCenter>, ObserverToken, 2>,
    events: Arc<PendingAudioSessionEvents>,
}

#[cfg(target_os = "ios")]
impl IosAudioSessionEventMonitor {
    /// Subscribes to public route-change and interruption notifications without changing session
    /// policy. Returns `UnsupportedOperatingSystem` below iOS 6.0.
    pub fn new() -> Result<Self, IosAudioSessionEventMonitorError> {
        if !objc2::available!(ios = 6.0, ..) {
            return Err(IosAudioSessionEventMonitorError::UnsupportedOperatingSystem);
        }
        // SAFETY: these public AVFAudio notification names are process-lifetime constants when
        // present. Their absence means the runtime does not provide the supported API surface.
        let names = unsafe {
            (
                AVAudioSessionRouteChangeNotification,
                AVAudioSessionInterruptionNotification,
            )
        };
        let (Some(route_change_name), Some(interruption_name)) = names else {
            return Err(IosAudioSessionEventMonitorError::UnsupportedOperatingSystem);
        };

        let events = Arc::new(PendingAudioSessionEvents::default());
        let center = NSNotificationCenter::defaultCenter();
        let mut observers = ObserverTokenOwner::new(center);
        let route_change_observer = add_audio_event_observer(
            observers.remover(),
            route_change_name,
            &events,
            ROUTE_CHANGE_EVENT,
        );
        observers.register(route_change_observer);
        let interruption_observer =
            add_interruption_observer(observers.remover(), interruption_name, &events);
        observers.register(interruption_observer);

        Ok(Self {
            _observers: observers,
            events,
        })
    }

    /// Atomically drains notification-category flags without waiting for a future event.
    pub fn drain(&self) -> IosAudioSessionEventFlags {
        self.events.drain()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg(any(target_os = "ios", test))]
enum Lifecycle {
    Ready,
    Starting,
    Running,
    Stopped,
}

#[cfg(any(target_os = "ios", test))]
impl Lifecycle {
    fn begin(&mut self) -> Result<(), MicrophoneStreamError> {
        if *self != Self::Ready {
            return Err(MicrophoneStreamError::AlreadyStarted);
        }
        *self = Self::Starting;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg(any(target_os = "ios", test))]
struct ValidatedInputFormat {
    public: PcmInputFormat,
    stride: u32,
    interleaved: bool,
    tap_buffer_frames: u32,
}

#[cfg(any(target_os = "ios", test))]
fn validate_input_format(
    sample_rate_hz: f64,
    channels: u32,
    is_float32: bool,
    interleaved: bool,
    capacity_frames: u32,
) -> Result<ValidatedInputFormat, MicrophoneStreamError> {
    let public = PcmInputFormat::float32_interleaved(sample_rate_hz, channels).map_err(
        |error| match error {
            framework_audio::PcmInputFormatError::InvalidSampleRate => {
                MicrophoneStreamError::InvalidSampleRate
            }
            framework_audio::PcmInputFormatError::ZeroChannels => {
                MicrophoneStreamError::InputUnavailable
            }
        },
    )?;
    if !(1..=2).contains(&channels) {
        return Err(MicrophoneStreamError::UnsupportedChannelCount);
    }
    if !is_float32 {
        return Err(MicrophoneStreamError::UnsupportedSampleFormat);
    }

    let tap_frames = (sample_rate_hz * 0.1).ceil();
    if !tap_frames.is_finite() || tap_frames < 1.0 || tap_frames > f64::from(u32::MAX) {
        return Err(MicrophoneStreamError::InvalidSampleRate);
    }
    let tap_buffer_frames = tap_frames as u32;
    if capacity_frames < tap_buffer_frames {
        return Err(MicrophoneStreamError::CapacityTooSmall);
    }

    Ok(ValidatedInputFormat {
        public,
        stride: if interleaved { channels } else { 1 },
        interleaved,
        tap_buffer_frames,
    })
}

#[cfg(any(target_os = "ios", test))]
fn allocate_storage(capacity_frames: u32) -> Result<Box<[UnsafeCell<f32>]>, MicrophoneStreamError> {
    if capacity_frames == 0 || capacity_frames > MAX_CAPACITY_FRAMES {
        return Err(MicrophoneStreamError::InvalidCapacity);
    }
    let sample_capacity = usize::try_from(capacity_frames)
        .ok()
        .and_then(|frames| frames.checked_mul(2))
        .ok_or(MicrophoneStreamError::InvalidCapacity)?;
    let mut samples = Vec::new();
    samples
        .try_reserve_exact(sample_capacity)
        .map_err(|_| MicrophoneStreamError::AllocationFailed)?;
    samples.resize_with(sample_capacity, || UnsafeCell::new(0.0));
    Ok(samples.into_boxed_slice())
}

#[cfg(any(target_os = "ios", test))]
struct FrameRing {
    samples: Box<[UnsafeCell<f32>]>,
    capacity_frames: u32,
    format: PcmInputFormat,
    stride: u32,
    interleaved: bool,
    read_frame: AtomicU32,
    write_frame: AtomicU32,
    dropped_frames: AtomicU64,
    callback_error: AtomicBool,
}

// SAFETY: There is one producer in the AVAudioNode tap and one consumer reached through
// `&mut IosMicrophonePcmStream`. The producer publishes completed frames with `write_frame`
// Release; the consumer reads them after Acquire and publishes reuse through `read_frame`
// Release. The producer observes reuse with Acquire. The immutable format fields do not change.
#[cfg(any(target_os = "ios", test))]
unsafe impl Sync for FrameRing {}

#[cfg(any(target_os = "ios", test))]
impl FrameRing {
    fn new(
        samples: Box<[UnsafeCell<f32>]>,
        capacity_frames: u32,
        format: PcmInputFormat,
        stride: u32,
        interleaved: bool,
    ) -> Self {
        Self {
            samples,
            capacity_frames,
            format,
            stride,
            interleaved,
            read_frame: AtomicU32::new(0),
            write_frame: AtomicU32::new(0),
            dropped_frames: AtomicU64::new(0),
            callback_error: AtomicBool::new(false),
        }
    }

    fn add_dropped_frames(&self, frames: u32) {
        let current = self.dropped_frames.load(Ordering::Relaxed);
        let next = current.saturating_add(u64::from(frames));
        if self
            .dropped_frames
            .compare_exchange(current, next, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {
            // Only the single tap producer writes this counter. A failed compare-exchange
            // therefore signals a violated callback-serialization assumption; saturate rather
            // than loop or block the real-time thread.
            self.dropped_frames.store(u64::MAX, Ordering::Relaxed);
        }
    }

    fn try_push_with(&self, frames: u32, mut sample: impl FnMut(u32, u32) -> f32) -> bool {
        if self.callback_error.load(Ordering::Relaxed) {
            return false;
        }
        let write = self.write_frame.load(Ordering::Relaxed);
        let read = self.read_frame.load(Ordering::Acquire);
        let queued = write.wrapping_sub(read);
        if queued > self.capacity_frames {
            self.callback_error.store(true, Ordering::Relaxed);
            return false;
        }
        if frames > self.capacity_frames - queued {
            self.add_dropped_frames(frames);
            return false;
        }

        let capacity = self.capacity_frames;
        let channels = self.format.channels() as usize;
        let mut slot = write % capacity;
        for frame in 0..frames {
            let base = slot as usize * channels;
            for channel in 0..self.format.channels() {
                let index = base + channel as usize;
                // SAFETY: `slot < capacity_frames`; storage has two samples per frame and
                // channels is constrained to one or two before the ring is created. The
                // SPSC cursors ensure this slot is not read while the producer writes it.
                unsafe {
                    self.samples
                        .get_unchecked(index)
                        .get()
                        .write(sample(frame, channel))
                };
            }
            slot += 1;
            if slot == capacity {
                slot = 0;
            }
        }
        self.write_frame
            .store(write.wrapping_add(frames), Ordering::Release);
        true
    }

    #[cfg(test)]
    fn try_push_interleaved(&self, samples: &[f32]) -> Result<bool, MicrophoneStreamError> {
        let channels = self.format.channels() as usize;
        let frames = self
            .format
            .frames_for_interleaved_samples(samples.len())
            .map_err(|_| MicrophoneStreamError::InvalidOutputLength)?;
        Ok(self.try_push_with(frames, |frame, channel| {
            samples[frame as usize * channels + channel as usize]
        }))
    }

    fn read_interleaved(&self, output: &mut [f32]) -> Result<u32, MicrophoneStreamError> {
        if self.callback_error.load(Ordering::Acquire) {
            return Err(MicrophoneStreamError::InputFormatChanged);
        }
        let channels = self.format.channels() as usize;
        let requested = self
            .format
            .frames_for_interleaved_samples(output.len())
            .map_err(|_| MicrophoneStreamError::InvalidOutputLength)?;
        let read = self.read_frame.load(Ordering::Relaxed);
        let write = self.write_frame.load(Ordering::Acquire);
        let queued = write.wrapping_sub(read);
        if queued > self.capacity_frames {
            return Err(MicrophoneStreamError::InputFormatChanged);
        }
        let frames = queued.min(requested);
        let capacity = self.capacity_frames;
        let mut slot = read % capacity;
        let mut output_index = 0;
        for _ in 0..frames {
            let base = slot as usize * channels;
            for channel in 0..channels {
                let index = base + channel;
                // SAFETY: The acquired write cursor publishes initialized samples. The reader
                // does not publish `read_frame` until after these reads, so the producer cannot
                // reuse the slots during this copy.
                output[output_index] = unsafe { *self.samples.get_unchecked(index).get() };
                output_index += 1;
            }
            slot += 1;
            if slot == capacity {
                slot = 0;
            }
        }
        self.read_frame
            .store(read.wrapping_add(frames), Ordering::Release);
        Ok(frames)
    }

    fn dropped_frames(&self) -> u64 {
        self.dropped_frames.load(Ordering::Relaxed)
    }

    #[cfg(target_os = "ios")]
    fn push_native_buffer(&self, buffer: NonNull<AVAudioPCMBuffer>) {
        // SAFETY: AVAudioNode supplies a live PCM buffer for the duration of the tap callback.
        let native_buffer = unsafe { buffer.as_ref() };
        // SAFETY: The format and its properties are valid for the live tap buffer.
        let format: Retained<AVAudioFormat> = unsafe { native_buffer.format() };
        let sample_rate = unsafe { format.sampleRate() };
        let channels = unsafe { format.channelCount() };
        let is_float32 = unsafe { format.commonFormat() == AVAudioCommonFormat::PCMFormatFloat32 };
        let interleaved = unsafe { format.isInterleaved() };
        if sample_rate != self.format.sample_rate_hz()
            || channels != self.format.channels()
            || !is_float32
            || interleaved != self.interleaved
        {
            self.callback_error.store(true, Ordering::Relaxed);
            return;
        }
        // SAFETY: These getters read immutable metadata and sample pointers for this callback.
        let frames = unsafe { native_buffer.frameLength() };
        let stride = unsafe { native_buffer.stride() };
        if stride != self.stride as usize {
            self.callback_error.store(true, Ordering::Relaxed);
            return;
        }
        // SAFETY: Float32 channel data is available only for Float32 PCM. A null pointer is
        // handled as a format change before any sample access.
        let channel_data = unsafe { native_buffer.floatChannelData() };
        if channel_data.is_null() {
            self.callback_error.store(true, Ordering::Relaxed);
            return;
        }
        let channel_data = channel_data.cast_const();
        // SAFETY: The validated input format has at most two channels. The tap is installed
        // with that same native format, so this array contains the configured channel pointers.
        let first = unsafe { channel_data.read() };
        let second = if self.format.channels() == 2 {
            // SAFETY: Channel count is two, as validated before tap installation.
            unsafe { channel_data.add(1).read() }
        } else {
            first
        };
        let channels = [first, second];
        let copied = self.try_push_with(frames, |frame, channel| unsafe {
            *channels[channel as usize]
                .as_ptr()
                .add(frame as usize * stride)
        });
        let _ = copied;
    }
}

#[cfg(target_os = "ios")]
type InputTap = RcBlock<dyn Fn(NonNull<AVAudioPCMBuffer>, NonNull<AVAudioTime>)>;

/// A one-shot microphone stream with a caller-polled, bounded Float32 ring.
///
/// The host must provide `NSMicrophoneUsageDescription`, explicitly obtain microphone
/// authorization, and configure and activate an input-capable `AVAudioSession` before `start`.
/// This type does not request permission or call `AVAudioSession` APIs. Apple documents that
/// `AVAudioEngine.startAndReturnError` may implicitly activate the shared audio session; the host
/// owns that policy and must preconfigure it. Samples retain the native Float32 values and channel
/// order, but complete tap blocks are dropped and counted when the consumer falls behind.
///
/// `start`, `stop`, and drop are synchronous engine operations and may block; call them away from
/// latency-sensitive UI work. Route/interruption recovery and runtime capture proof are out of
/// scope.
#[cfg(target_os = "ios")]
pub struct IosMicrophonePcmStream {
    capacity_frames: u32,
    storage: Option<Box<[UnsafeCell<f32>]>>,
    ring: Option<Arc<FrameRing>>,
    engine: Option<Retained<AVAudioEngine>>,
    input: Option<Retained<AVAudioInputNode>>,
    tap: Option<InputTap>,
    format: Option<PcmInputFormat>,
    lifecycle: Lifecycle,
    _not_send_or_sync: PhantomData<Rc<()>>,
}

#[cfg(target_os = "ios")]
impl IosMicrophonePcmStream {
    /// Creates a bounded stream for an already-authorized microphone.
    ///
    /// `capacity_frames` is the ring's maximum frame count and is limited so allocation remains
    /// bounded. The host must configure its recording-capable audio session before `start`.
    pub fn new(capacity_frames: u32) -> Result<Self, MicrophoneStreamError> {
        if !may_stream(IosMediaAuthorization::authorization_status(
            framework_media_authorization::CaptureMedia::Microphone,
        )) {
            return Err(MicrophoneStreamError::AuthorizationNotGranted);
        }
        if !objc2::available!(ios = 8.0, ..) {
            return Err(MicrophoneStreamError::UnsupportedOperatingSystem);
        }
        if capacity_frames == 0 || capacity_frames > MAX_CAPACITY_FRAMES {
            return Err(MicrophoneStreamError::InvalidCapacity);
        }
        let storage = allocate_storage(capacity_frames)?;
        Ok(Self {
            capacity_frames,
            storage: Some(storage),
            ring: None,
            engine: None,
            input: None,
            tap: None,
            format: None,
            lifecycle: Lifecycle::Ready,
            _not_send_or_sync: PhantomData,
        })
    }

    /// Starts native-format Float32 capture and returns the actual sample rate and channels.
    pub fn start(&mut self) -> Result<PcmInputFormat, MicrophoneStreamError> {
        if !may_stream(IosMediaAuthorization::authorization_status(
            framework_media_authorization::CaptureMedia::Microphone,
        )) {
            return Err(MicrophoneStreamError::AuthorizationNotGranted);
        }
        self.lifecycle.begin()?;
        match self.start_native() {
            Ok(format) => {
                self.format = Some(format);
                self.lifecycle = Lifecycle::Running;
                Ok(format)
            }
            Err(error) => {
                self.stop_native();
                self.lifecycle = Lifecycle::Stopped;
                Err(error)
            }
        }
    }

    /// Reads available interleaved samples without waiting for more input.
    ///
    /// The slice length must be a multiple of the active channel count. The returned value is a
    /// frame count; use `PcmInputFormat::channels` to derive the number of samples read. After
    /// `stop`, already-buffered frames remain readable.
    pub fn try_read_interleaved(
        &mut self,
        output: &mut [f32],
    ) -> Result<u32, MicrophoneStreamError> {
        let ring = self
            .ring
            .as_ref()
            .ok_or(MicrophoneStreamError::NotStarted)?;
        if self.format.is_none() {
            return Err(MicrophoneStreamError::NotStarted);
        }
        ring.read_interleaved(output)
    }

    /// Returns the number of frames dropped because the ring had no space.
    pub fn dropped_frames(&self) -> u64 {
        self.ring.as_ref().map_or(0, |ring| ring.dropped_frames())
    }

    /// Returns whether AVAudioEngine currently reports itself as running.
    pub fn is_running(&self) -> bool {
        if self.lifecycle != Lifecycle::Running {
            return false;
        }
        self.engine.as_ref().is_some_and(|engine| {
            // SAFETY: engine is retained by self and lifecycle is serialized by &self/&mut self.
            unsafe { engine.isRunning() }
        })
    }

    /// Stops capture, removes the tap, and leaves any queued frames readable.
    pub fn stop(&mut self) {
        if self.lifecycle == Lifecycle::Running || self.lifecycle == Lifecycle::Starting {
            self.stop_native();
        }
        self.lifecycle = Lifecycle::Stopped;
    }

    fn start_native(&mut self) -> Result<PcmInputFormat, MicrophoneStreamError> {
        // SAFETY: AVAudioEngine is public and available from iOS 8; `new` already checked that
        // floor. The engine remains local until start succeeds.
        let engine = unsafe { AVAudioEngine::init(AVAudioEngine::alloc()) };
        // SAFETY: inputNode is the engine-owned public input node; permission was checked above.
        let input = unsafe { engine.inputNode() };
        // SAFETY: bus zero is the input node's sole input-capture output bus.
        let native_format = unsafe { input.outputFormatForBus(0) };
        // SAFETY: These are read-only public format properties on the retained format object.
        let sample_rate = unsafe { native_format.sampleRate() };
        let channels = unsafe { native_format.channelCount() };
        let is_float32 =
            unsafe { native_format.commonFormat() == AVAudioCommonFormat::PCMFormatFloat32 };
        let interleaved = unsafe { native_format.isInterleaved() };
        if channels == 0 {
            return Err(MicrophoneStreamError::InputUnavailable);
        }
        let validated = validate_input_format(
            sample_rate,
            channels,
            is_float32,
            interleaved,
            self.capacity_frames,
        )?;
        let storage = self
            .storage
            .take()
            .ok_or(MicrophoneStreamError::AlreadyStarted)?;
        let ring = Arc::new(FrameRing::new(
            storage,
            self.capacity_frames,
            validated.public,
            validated.stride,
            validated.interleaved,
        ));
        let callback_ring = Arc::clone(&ring);
        let tap: InputTap = RcBlock::new(move |buffer, _when| {
            let result = catch_unwind(AssertUnwindSafe(|| {
                callback_ring.push_native_buffer(buffer);
            }));
            if result.is_err() {
                callback_ring.callback_error.store(true, Ordering::Relaxed);
            }
        });

        // SAFETY: `native_format` is exactly the input node's output format; the retained `tap`
        // block remains alive through `removeTapOnBus` and engine shutdown.
        unsafe {
            input.installTapOnBus_bufferSize_format_block(
                0,
                validated.tap_buffer_frames,
                Some(&native_format),
                RcBlock::as_ptr(&tap) as AVAudioNodeTapBlock,
            )
        };
        // SAFETY: engine owns the configured input tap; errors are returned as NSError.
        if unsafe { engine.startAndReturnError() }.is_err() {
            // SAFETY: The engine and input remain retained locally, and the tap was installed on
            // this exact input node. Stop before removing/releasing its callback state.
            unsafe {
                engine.stop();
                input.removeTapOnBus(0);
            }
            return Err(MicrophoneStreamError::EngineStartFailed);
        }

        self.ring = Some(ring);
        self.engine = Some(engine);
        self.input = Some(input);
        self.tap = Some(tap);
        Ok(validated.public)
    }

    fn stop_native(&mut self) {
        if let Some(engine) = &self.engine {
            // SAFETY: engine is retained and all lifecycle calls are serialized by `&mut self`.
            unsafe { engine.stop() };
        }
        if let Some(input) = &self.input {
            // SAFETY: bus zero is the tap installed by `start_native`; removal follows stop.
            unsafe { input.removeTapOnBus(0) };
        }
        self.tap.take();
        self.input.take();
        self.engine.take();
    }
}

#[cfg(target_os = "ios")]
impl Drop for IosMicrophonePcmStream {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(target_os = "ios")]
impl PcmInputStream for IosMicrophonePcmStream {
    type Error = MicrophoneStreamError;

    fn format(&self) -> Option<PcmInputFormat> {
        self.format
    }

    fn try_read_interleaved(&mut self, output: &mut [f32]) -> Result<u32, Self::Error> {
        IosMicrophonePcmStream::try_read_interleaved(self, output)
    }

    fn dropped_frames(&self) -> u64 {
        IosMicrophonePcmStream::dropped_frames(self)
    }
}

#[cfg(any(target_os = "ios", test))]
const fn may_stream(status: MediaAuthorizationStatus) -> bool {
    matches!(status, MediaAuthorizationStatus::Authorized)
}

#[cfg(test)]
mod tests {
    use super::{
        FrameRing, INTERRUPTION_BEGAN_EVENT, INTERRUPTION_ENDED_EVENT, IosAudioSessionEventFlags,
        Lifecycle, MicrophoneInputSelectionError, MicrophoneStreamError, ObserverTokenOwner,
        ObserverTokenRemover, PendingAudioSessionEvents, ROUTE_CHANGE_EVENT, allocate_storage,
        collect_input_port_snapshot, make_audio_event_callback, may_stream, select_available_input,
        validate_input_format,
    };
    use crate::PcmInputFormat;
    use framework_media_authorization::MediaAuthorizationStatus;
    use std::cell::{Cell, RefCell, UnsafeCell};
    use std::rc::Rc;
    use std::sync::Arc;

    #[derive(Debug)]
    struct TestInputPort {
        uid: String,
        marker: u8,
    }

    fn ring(capacity_frames: u32, channels: u32, stride: u32) -> FrameRing {
        FrameRing::new(
            allocate_storage(capacity_frames).unwrap(),
            capacity_frames,
            PcmInputFormat::float32_interleaved(48_000.0, channels).unwrap(),
            stride,
            stride == channels,
        )
    }

    #[test]
    fn audio_event_flags_coalesce_independent_categories_until_drain() {
        let events = Arc::new(PendingAudioSessionEvents::default());
        let route_change = make_audio_event_callback(Arc::downgrade(&events), ROUTE_CHANGE_EVENT);
        let interruption_began =
            make_audio_event_callback(Arc::downgrade(&events), INTERRUPTION_BEGAN_EVENT);
        let interruption_ended =
            make_audio_event_callback(Arc::downgrade(&events), INTERRUPTION_ENDED_EVENT);

        route_change();
        route_change();
        assert_eq!(
            events.drain(),
            IosAudioSessionEventFlags {
                route_change: true,
                interruption_began: false,
                interruption_ended: false,
                interruption_unclassified: false,
            }
        );
        assert!(events.drain().is_empty());

        interruption_began();
        interruption_began();
        interruption_ended();
        route_change();
        assert_eq!(
            events.drain(),
            IosAudioSessionEventFlags {
                route_change: true,
                interruption_began: true,
                interruption_ended: true,
                interruption_unclassified: false,
            }
        );
        assert!(events.drain().is_empty());
    }

    #[test]
    fn interruption_types_are_classified_and_unexpected_payloads_are_preserved() {
        let events = PendingAudioSessionEvents::default();
        events.record_interruption(Some(1));
        assert!(events.drain().interruption_began());
        events.record_interruption(Some(0));
        assert!(events.drain().interruption_ended());

        events.record_interruption(None);
        events.record_interruption(Some(7));
        let flags = events.drain();
        assert!(flags.interruption());
        assert!(flags.interruption_unclassified());
        assert!(!flags.interruption_began());
        assert!(!flags.interruption_ended());
        assert!(events.drain().is_empty());
    }

    #[test]
    fn audio_observer_callback_does_not_retain_or_update_dropped_event_state() {
        let events = Arc::new(PendingAudioSessionEvents::default());
        let weak_events = Arc::downgrade(&events);
        let callback = make_audio_event_callback(weak_events.clone(), ROUTE_CHANGE_EVENT);

        drop(events);
        callback();

        assert!(weak_events.upgrade().is_none());
    }

    struct FakeObserverRemover(Rc<RefCell<Vec<u8>>>);

    impl ObserverTokenRemover<u8> for FakeObserverRemover {
        fn remove(&self, token: &u8) {
            self.0.borrow_mut().push(*token);
        }
    }

    #[test]
    fn observer_owner_removes_each_registered_token_once_on_drop() {
        let removed = Rc::new(RefCell::new(Vec::new()));
        let mut observers =
            ObserverTokenOwner::<_, _, 2>::new(FakeObserverRemover(Rc::clone(&removed)));
        observers.register(3);
        observers.register(7);

        drop(observers);

        assert_eq!(*removed.borrow(), [3, 7]);
    }

    #[test]
    fn authorization_requires_the_existing_authorized_status() {
        assert!(!may_stream(MediaAuthorizationStatus::NotDetermined));
        assert!(!may_stream(MediaAuthorizationStatus::Restricted));
        assert!(!may_stream(MediaAuthorizationStatus::Denied));
        assert!(may_stream(MediaAuthorizationStatus::Authorized));
        assert!(!may_stream(MediaAuthorizationStatus::Unknown));
    }

    #[test]
    fn nil_and_empty_input_port_lists_form_empty_snapshots() {
        assert!(collect_input_port_snapshot(None::<Vec<(String, String, String)>>).is_empty());
        assert!(collect_input_port_snapshot(Some(Vec::new())).is_empty());
    }

    #[test]
    fn input_port_snapshot_preserves_order_unknown_types_and_exact_metadata() {
        let decomposed_name = "Cafe\u{301} microphone 🎤".to_owned();
        let first_uid = "external\0input-1".to_owned();
        let ports = collect_input_port_snapshot(Some(vec![
            (
                "VendorFutureInput".to_owned(),
                decomposed_name.clone(),
                first_uid.clone(),
            ),
            (
                "VendorFutureInput".to_owned(),
                "Built-in microphone".to_owned(),
                first_uid.clone(),
            ),
        ]));

        assert_eq!(ports.len(), 2);
        assert_eq!(ports[0].port_type(), "VendorFutureInput");
        assert_eq!(ports[0].name(), decomposed_name);
        assert_eq!(
            ports[0].name().as_bytes(),
            "Cafe\u{301} microphone 🎤".as_bytes()
        );
        assert_eq!(ports[0].uid(), first_uid);
        assert_eq!(ports[1].name(), "Built-in microphone");
        assert_eq!(ports[1].uid(), "external\0input-1");
    }

    #[test]
    fn input_selection_uses_exact_uid_and_calls_setter_once_for_the_match() {
        let setter_calls = Cell::new(0);
        let selected_marker = Cell::new(None);
        let ports = vec![
            TestInputPort {
                uid: "usb-input".to_owned(),
                marker: 1,
            },
            TestInputPort {
                uid: "bluetooth-input".to_owned(),
                marker: 2,
            },
        ];

        assert_eq!(
            select_available_input(
                Some(ports),
                "bluetooth-input",
                |port| port.uid.clone(),
                |port| {
                    setter_calls.set(setter_calls.get() + 1);
                    selected_marker.set(Some(port.marker));
                    Ok::<(), ()>(())
                },
            ),
            Ok(())
        );
        assert_eq!(setter_calls.get(), 1);
        assert_eq!(selected_marker.get(), Some(2));
    }

    #[test]
    fn empty_or_unavailable_input_uid_never_calls_the_setter() {
        let setter_calls = Cell::new(0);
        let ports = vec![TestInputPort {
            uid: "usb-input".to_owned(),
            marker: 1,
        }];

        assert_eq!(
            select_available_input(
                Some(ports),
                "",
                |port| port.uid.clone(),
                |_| {
                    setter_calls.set(setter_calls.get() + 1);
                    Ok::<(), ()>(())
                },
            ),
            Err(MicrophoneInputSelectionError::InvalidUid)
        );
        assert_eq!(
            select_available_input(
                Some(vec![TestInputPort {
                    uid: "usb-input".to_owned(),
                    marker: 1,
                }]),
                "USB-input",
                |port| port.uid.clone(),
                |_| {
                    setter_calls.set(setter_calls.get() + 1);
                    Ok::<(), ()>(())
                },
            ),
            Err(MicrophoneInputSelectionError::InputNotAvailable)
        );
        assert_eq!(
            select_available_input(
                None::<Vec<TestInputPort>>,
                "usb-input",
                |port| port.uid.clone(),
                |_| {
                    setter_calls.set(setter_calls.get() + 1);
                    Ok::<(), ()>(())
                },
            ),
            Err(MicrophoneInputSelectionError::InputNotAvailable)
        );
        assert_eq!(setter_calls.get(), 0);
    }

    #[test]
    fn native_input_selection_failure_is_reported() {
        assert_eq!(
            select_available_input(
                Some(vec![TestInputPort {
                    uid: "usb-input".to_owned(),
                    marker: 1,
                }]),
                "usb-input",
                |port| port.uid.clone(),
                |_| Err::<(), _>("native route error"),
            ),
            Err(MicrophoneInputSelectionError::NativeFailure)
        );
    }

    #[test]
    fn native_format_accepts_float32_one_or_two_channel_layouts() {
        let mono = validate_input_format(48_000.0, 1, true, false, 8192).unwrap();
        assert_eq!(mono.public.channels(), 1);
        assert_eq!(mono.public.sample_rate_hz(), 48_000.0);
        assert_eq!(mono.stride, 1);
        assert_eq!(mono.tap_buffer_frames, 4800);

        let stereo_planar = validate_input_format(44_100.0, 2, true, false, 8192).unwrap();
        assert_eq!(stereo_planar.stride, 1);
        let stereo_interleaved = validate_input_format(44_100.0, 2, true, true, 8192).unwrap();
        assert_eq!(stereo_interleaved.stride, 2);
    }

    #[test]
    fn native_format_rejects_unsupported_rates_samples_channels_and_capacity() {
        assert_eq!(
            validate_input_format(f64::NAN, 1, true, false, 8192),
            Err(MicrophoneStreamError::InvalidSampleRate)
        );
        assert_eq!(
            validate_input_format(0.0, 1, true, false, 8192),
            Err(MicrophoneStreamError::InvalidSampleRate)
        );
        assert_eq!(
            validate_input_format(48_000.0, 1, false, false, 8192),
            Err(MicrophoneStreamError::UnsupportedSampleFormat)
        );
        assert_eq!(
            validate_input_format(48_000.0, 3, true, false, 8192),
            Err(MicrophoneStreamError::UnsupportedChannelCount)
        );
        assert_eq!(
            validate_input_format(48_000.0, 0, true, false, 8192),
            Err(MicrophoneStreamError::InputUnavailable)
        );
        assert_eq!(
            validate_input_format(48_000.0, 1, true, false, 4799),
            Err(MicrophoneStreamError::CapacityTooSmall)
        );
    }

    #[test]
    fn spsc_ring_preserves_fifo_and_interleaved_channel_order() {
        let ring = ring(4, 2, 1);
        assert_eq!(ring.format.sample_rate_hz(), 48_000.0);
        assert!(!ring.interleaved);
        assert!(ring.try_push_interleaved(&[1.0, 10.0, 2.0, 20.0]).unwrap());
        let mut output = [0.0; 4];
        assert_eq!(ring.read_interleaved(&mut output), Ok(2));
        assert_eq!(output, [1.0, 10.0, 2.0, 20.0]);
    }

    #[test]
    fn spsc_ring_handles_empty_reads_and_wraparound() {
        let ring = ring(3, 1, 1);
        let mut output = [0.0; 3];
        assert_eq!(ring.read_interleaved(&mut output), Ok(0));
        assert!(ring.try_push_interleaved(&[1.0, 2.0]).unwrap());
        assert_eq!(ring.read_interleaved(&mut output[..1]), Ok(1));
        assert_eq!(output[0], 1.0);
        assert!(ring.try_push_interleaved(&[3.0, 4.0]).unwrap());
        assert_eq!(ring.read_interleaved(&mut output), Ok(3));
        assert_eq!(output, [2.0, 3.0, 4.0]);
    }

    #[test]
    fn overflow_drops_whole_blocks_and_saturates_without_a_lock() {
        let ring = ring(3, 2, 2);
        assert_eq!(ring.stride, 2);
        assert!(ring.interleaved);
        assert!(ring.try_push_interleaved(&[1.0, 10.0, 2.0, 20.0]).unwrap());
        assert!(!ring.try_push_interleaved(&[3.0, 30.0, 4.0, 40.0]).unwrap());
        assert_eq!(ring.dropped_frames(), 2);
        let mut output = [0.0; 6];
        assert_eq!(ring.read_interleaved(&mut output), Ok(2));
        assert_eq!(&output[..4], &[1.0, 10.0, 2.0, 20.0]);
        assert!(ring.try_push_interleaved(&[3.0, 30.0]).unwrap());
        ring.dropped_frames
            .store(u64::MAX - 1, std::sync::atomic::Ordering::Relaxed);
        assert!(
            !ring
                .try_push_interleaved(&[4.0, 40.0, 5.0, 50.0, 6.0, 60.0])
                .unwrap()
        );
        assert_eq!(ring.dropped_frames(), u64::MAX);
    }

    #[test]
    fn reads_reject_partial_frames_and_capacity_is_checked() {
        let ring = ring(2, 2, 2);
        assert_eq!(
            ring.read_interleaved(&mut [0.0]),
            Err(MicrophoneStreamError::InvalidOutputLength)
        );
        assert!(matches!(
            allocate_storage(u32::MAX),
            Err(MicrophoneStreamError::InvalidCapacity)
        ));
        let storage: Box<[UnsafeCell<f32>]> = allocate_storage(1).unwrap();
        assert_eq!(storage.len(), 2);
    }

    #[test]
    fn one_shot_lifecycle_rejects_restart_and_stop_is_terminal() {
        let mut lifecycle = Lifecycle::Ready;
        lifecycle.begin().unwrap();
        lifecycle = Lifecycle::Running;
        assert_eq!(
            lifecycle.begin(),
            Err(MicrophoneStreamError::AlreadyStarted)
        );
        lifecycle = Lifecycle::Stopped;
        assert_eq!(
            lifecycle.begin(),
            Err(MicrophoneStreamError::AlreadyStarted)
        );
    }
}
