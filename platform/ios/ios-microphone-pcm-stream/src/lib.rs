#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Authorized live iOS microphone PCM input"]

#[cfg(any(target_os = "ios", test))]
use core::cell::UnsafeCell;
#[cfg(any(target_os = "ios", test))]
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

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
use objc2::{AnyThread, rc::Retained};
#[cfg(target_os = "ios")]
use objc2_avf_audio::{
    AVAudioCommonFormat, AVAudioEngine, AVAudioFormat, AVAudioInputNode, AVAudioNodeTapBlock,
    AVAudioPCMBuffer, AVAudioTime,
};
#[cfg(target_os = "ios")]
use std::panic::{AssertUnwindSafe, catch_unwind};
#[cfg(target_os = "ios")]
use std::rc::Rc;
#[cfg(target_os = "ios")]
use std::sync::Arc;

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

/// The native input format returned when a stream starts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PcmInputFormat {
    sample_rate_hz: f64,
    channels: u32,
}

impl PcmInputFormat {
    /// Returns the native input sample rate in hertz.
    pub const fn sample_rate_hz(self) -> f64 {
        self.sample_rate_hz
    }

    /// Returns the native input channel count.
    pub const fn channels(self) -> u32 {
        self.channels
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
    if !sample_rate_hz.is_finite() || sample_rate_hz <= 0.0 {
        return Err(MicrophoneStreamError::InvalidSampleRate);
    }
    if !(1..=2).contains(&channels) {
        return Err(if channels == 0 {
            MicrophoneStreamError::InputUnavailable
        } else {
            MicrophoneStreamError::UnsupportedChannelCount
        });
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
        public: PcmInputFormat {
            sample_rate_hz,
            channels,
        },
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
    channels: u32,
    sample_rate_hz: f64,
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
        channels: u32,
        sample_rate_hz: f64,
        stride: u32,
        interleaved: bool,
    ) -> Self {
        Self {
            samples,
            capacity_frames,
            channels,
            sample_rate_hz,
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
        let channels = self.channels as usize;
        let mut slot = write % capacity;
        for frame in 0..frames {
            let base = slot as usize * channels;
            for channel in 0..self.channels {
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
        let channels = self.channels as usize;
        if samples.len() % channels != 0 {
            return Err(MicrophoneStreamError::InvalidOutputLength);
        }
        let frames = u32::try_from(samples.len() / channels)
            .map_err(|_| MicrophoneStreamError::InvalidOutputLength)?;
        Ok(self.try_push_with(frames, |frame, channel| {
            samples[frame as usize * channels + channel as usize]
        }))
    }

    fn read_interleaved(&self, output: &mut [f32]) -> Result<u32, MicrophoneStreamError> {
        if self.callback_error.load(Ordering::Acquire) {
            return Err(MicrophoneStreamError::InputFormatChanged);
        }
        let channels = self.channels as usize;
        if output.len() % channels != 0 {
            return Err(MicrophoneStreamError::InvalidOutputLength);
        }
        let read = self.read_frame.load(Ordering::Relaxed);
        let write = self.write_frame.load(Ordering::Acquire);
        let queued = write.wrapping_sub(read);
        if queued > self.capacity_frames {
            return Err(MicrophoneStreamError::InputFormatChanged);
        }
        let requested = u32::try_from(output.len() / channels).unwrap_or(u32::MAX);
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
        if sample_rate != self.sample_rate_hz
            || channels != self.channels
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
        let second = if self.channels == 2 {
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
            channels,
            sample_rate,
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

#[cfg(any(target_os = "ios", test))]
const fn may_stream(status: MediaAuthorizationStatus) -> bool {
    matches!(status, MediaAuthorizationStatus::Authorized)
}

#[cfg(test)]
mod tests {
    use super::{
        FrameRing, Lifecycle, MicrophoneStreamError, allocate_storage, may_stream,
        validate_input_format,
    };
    use framework_media_authorization::MediaAuthorizationStatus;
    use std::cell::UnsafeCell;

    fn ring(capacity_frames: u32, channels: u32, stride: u32) -> FrameRing {
        FrameRing::new(
            allocate_storage(capacity_frames).unwrap(),
            capacity_frames,
            channels,
            48_000.0,
            stride,
            stride == channels,
        )
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
        assert_eq!(ring.sample_rate_hz, 48_000.0);
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
