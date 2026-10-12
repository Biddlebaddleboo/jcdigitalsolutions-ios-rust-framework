#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Bounded Rust-native PCM output through the public iOS AVFAudio graph"]

#[cfg(any(test, target_os = "ios"))]
use core::cell::UnsafeCell;
#[cfg(any(test, target_os = "ios"))]
use core::sync::atomic::{AtomicU32, Ordering};
#[cfg(any(test, target_os = "ios"))]
use std::vec::Vec;

#[cfg(target_os = "ios")]
use block2::RcBlock;
#[cfg(target_os = "ios")]
use core::ffi::c_void;
#[cfg(target_os = "ios")]
use core::marker::PhantomData;
#[cfg(target_os = "ios")]
use core::ptr::{self, NonNull};
#[cfg(target_os = "ios")]
use objc2::AnyThread;
#[cfg(target_os = "ios")]
use objc2::rc::Retained;
#[cfg(target_os = "ios")]
use objc2::runtime::Bool;
#[cfg(target_os = "ios")]
use objc2_avf_audio::{AVAudioCommonFormat, AVAudioEngine, AVAudioFormat, AVAudioSourceNode};
#[cfg(target_os = "ios")]
use objc2_core_audio_types::{AudioBufferList, AudioTimeStamp, kAudio_NoError, kAudio_ParamError};
#[cfg(target_os = "ios")]
use std::rc::Rc;

#[cfg(target_os = "ios")]
type SourceRenderBlock =
    dyn Fn(NonNull<Bool>, NonNull<AudioTimeStamp>, u32, NonNull<AudioBufferList>) -> i32;
#[cfg(target_os = "ios")]
type NativeAudioGraph = (
    Retained<AVAudioEngine>,
    Retained<AVAudioSourceNode>,
    RcBlock<SourceRenderBlock>,
);

/// Maximum queue capacity in PCM sample frames.
pub const MAX_PCM_OUTPUT_CAPACITY_FRAMES: u32 = 1_048_576;

/// Interleaved linear PCM sample representation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PcmSampleFormat {
    /// Signed 16-bit integer samples.
    Int16,
    /// IEEE-754 32-bit floating-point samples.
    Float32,
}

/// Explicit PCM format for a native audio-output stream.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PcmOutputFormat {
    sample_rate_hz: f64,
    channels: u32,
    sample_format: PcmSampleFormat,
}

impl PcmOutputFormat {
    /// Creates a format with a finite positive sample rate and one or two channels.
    pub fn new(
        sample_rate_hz: f64,
        channels: u32,
        sample_format: PcmSampleFormat,
    ) -> Result<Self, PcmOutputFormatError> {
        if !sample_rate_hz.is_finite() || sample_rate_hz <= 0.0 {
            return Err(PcmOutputFormatError::InvalidSampleRate);
        }
        if !(1..=2).contains(&channels) {
            return Err(PcmOutputFormatError::UnsupportedChannelCount);
        }
        Ok(Self {
            sample_rate_hz,
            channels,
            sample_format,
        })
    }

    /// Returns the requested sample rate in hertz.
    pub const fn sample_rate_hz(self) -> f64 {
        self.sample_rate_hz
    }

    /// Returns the number of interleaved channels.
    pub const fn channels(self) -> u32 {
        self.channels
    }

    /// Returns the PCM sample representation.
    pub const fn sample_format(self) -> PcmSampleFormat {
        self.sample_format
    }
}

/// A rejected PCM output format.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PcmOutputFormatError {
    /// The sample rate must be finite and greater than zero.
    InvalidSampleRate,
    /// This API supports one or two channels.
    UnsupportedChannelCount,
}

/// Failure to create or start an iOS PCM output graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PcmAudioOutputError {
    /// AVAudioSourceNode requires iOS 13.0 or newer.
    UnsupportedOperatingSystem,
    /// The queue capacity is zero or exceeds the public maximum.
    InvalidCapacity,
    /// The bounded sample queue could not be allocated.
    AllocationFailed,
    /// AVFAudio rejected the requested PCM format.
    UnsupportedFormat,
    /// AVAudioEngine rejected the request to start.
    EngineStartFailed,
    /// The output has already started.
    AlreadyStarted,
    /// The output has been stopped and cannot restart.
    Stopped,
}

/// Failure to copy a complete interleaved PCM write into the bounded queue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PcmWriteError {
    /// The write API does not match the configured sample representation.
    SampleFormatMismatch,
    /// The sample slice length is not a whole number of frames.
    InvalidSampleLength,
    /// The queue has no space for the complete write; no samples were copied.
    QueueFull,
    /// The output has been stopped.
    Stopped,
}

#[cfg(any(test, target_os = "ios"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Lifecycle {
    Ready,
    Running,
    Stopped,
}

/// A bounded single-producer/single-consumer PCM sample queue.
///
/// The Rust caller is the single producer and AVAudioSourceNode's render callback is the single
/// consumer. A write copies samples into preallocated storage; it never retains the caller's
/// slice. Queue operations do not allocate or lock.
#[cfg(any(test, target_os = "ios"))]
struct SampleQueue {
    samples: Vec<UnsafeCell<u32>>,
    sample_capacity: u32,
    channels: u32,
    write_index: AtomicU32,
    read_index: AtomicU32,
}

// SAFETY: Exactly one caller thread mutates the queue through &mut IosPcmAudioOutput, and the
// native render callback is the only concurrent consumer. Release/acquire indices keep each slot
// exclusively owned until the producer observes that the consumer has advanced.
#[cfg(any(test, target_os = "ios"))]
unsafe impl Sync for SampleQueue {}

#[cfg(any(test, target_os = "ios"))]
impl SampleQueue {
    fn new(capacity_frames: u32, channels: u32) -> Result<Self, PcmAudioOutputError> {
        if capacity_frames == 0 || capacity_frames > MAX_PCM_OUTPUT_CAPACITY_FRAMES {
            return Err(PcmAudioOutputError::InvalidCapacity);
        }
        let sample_capacity = capacity_frames
            .checked_mul(channels)
            .ok_or(PcmAudioOutputError::InvalidCapacity)?;
        let sample_len =
            usize::try_from(sample_capacity).map_err(|_| PcmAudioOutputError::InvalidCapacity)?;
        let mut samples = Vec::new();
        samples
            .try_reserve_exact(sample_len)
            .map_err(|_| PcmAudioOutputError::AllocationFailed)?;
        samples.resize_with(sample_len, || UnsafeCell::new(0));
        Ok(Self {
            samples,
            sample_capacity,
            channels,
            write_index: AtomicU32::new(0),
            read_index: AtomicU32::new(0),
        })
    }

    fn push_with(
        &self,
        sample_count: usize,
        mut encode: impl FnMut(usize) -> u32,
    ) -> Result<u32, PcmWriteError> {
        let sample_count_u32 = u32::try_from(sample_count).map_err(|_| PcmWriteError::QueueFull)?;
        if sample_count_u32 > self.sample_capacity {
            return Err(PcmWriteError::QueueFull);
        }
        let write = self.write_index.load(Ordering::Relaxed);
        let read = self.read_index.load(Ordering::Acquire);
        let queued = write.wrapping_sub(read);
        if queued > self.sample_capacity || sample_count_u32 > self.sample_capacity - queued {
            return Err(PcmWriteError::QueueFull);
        }
        for offset in 0..sample_count_u32 {
            let slot = write.wrapping_add(offset) % self.sample_capacity;
            let Some(cell) = self.samples.get(slot as usize) else {
                return Err(PcmWriteError::QueueFull);
            };
            // SAFETY: The release/acquire queue indices establish producer-exclusive ownership of
            // every slot in this not-yet-published range.
            unsafe { *cell.get() = encode(offset as usize) };
        }
        self.write_index
            .store(write.wrapping_add(sample_count_u32), Ordering::Release);
        Ok(sample_count_u32 / self.channels)
    }

    fn consume_with(&self, sample_limit: u32, mut write_sample: impl FnMut(u32, u32)) -> u32 {
        let read = self.read_index.load(Ordering::Relaxed);
        let write = self.write_index.load(Ordering::Acquire);
        let available = write.wrapping_sub(read).min(self.sample_capacity);
        let count = sample_limit.min(available);
        for offset in 0..count {
            let slot = read.wrapping_add(offset) % self.sample_capacity;
            let Some(cell) = self.samples.get(slot as usize) else {
                return offset;
            };
            // SAFETY: The acquire load observes the producer's release publication. The producer
            // cannot reuse this slot until this consumer publishes the advanced read index.
            let sample = unsafe { *cell.get() };
            write_sample(offset, sample);
        }
        self.read_index
            .store(read.wrapping_add(count), Ordering::Release);
        count
    }

    fn queued_frames(&self) -> u32 {
        let write = self.write_index.load(Ordering::Acquire);
        let read = self.read_index.load(Ordering::Acquire);
        write.wrapping_sub(read).min(self.sample_capacity) / self.channels
    }
}

#[cfg(any(test, target_os = "ios"))]
fn render_samples(
    queue: &SampleQueue,
    sample_limit: u32,
    clear_output: impl FnOnce(),
    write_sample: impl FnMut(u32, u32),
) -> u32 {
    clear_output();
    queue.consume_with(sample_limit, write_sample)
}

/// A bounded iOS PCM output stream backed by AVAudioSourceNode and AVAudioEngine.
///
/// Writes are copied synchronously into a preallocated, fixed-capacity interleaved PCM queue.
/// The render callback consumes queued frames without allocation, locking, or application
/// callbacks and emits silence when it runs out of data. Construction performs no permission
/// request and does not access AVAudioSession. The host owns the session category, mode,
/// activation, and route policy. Apple documents that AVAudioEngine preparation/start may
/// implicitly activate the shared session, so the host should configure and activate its desired
/// session policy before calling `start`.
///
/// This type is intentionally local-thread-only and one-shot. `stop` tears down the graph; it
/// does not deactivate or reconfigure AVAudioSession. Any frames still queued at stop remain in
/// the bounded queue until this value is dropped, but are no longer rendered or writable.
/// Compile/link checks do not prove audible output or route behavior on a device.
#[cfg(any(test, target_os = "ios"))]
pub struct IosPcmAudioOutput {
    format: PcmOutputFormat,
    queue: std::sync::Arc<SampleQueue>,
    lifecycle: Lifecycle,
    #[cfg(target_os = "ios")]
    engine: Option<Retained<AVAudioEngine>>,
    #[cfg(target_os = "ios")]
    source: Option<Retained<AVAudioSourceNode>>,
    #[cfg(target_os = "ios")]
    render_block: Option<RcBlock<SourceRenderBlock>>,
    #[cfg(target_os = "ios")]
    _not_send_or_sync: PhantomData<Rc<()>>,
}

#[cfg(any(test, target_os = "ios"))]
impl IosPcmAudioOutput {
    /// Creates an output for an explicit PCM format and bounded queue capacity.
    ///
    /// `capacity_frames` is limited by [`MAX_PCM_OUTPUT_CAPACITY_FRAMES`]. The caller owns audio
    /// session policy and should configure/activate that session before `start`.
    pub fn new(format: PcmOutputFormat, capacity_frames: u32) -> Result<Self, PcmAudioOutputError> {
        if capacity_frames == 0 || capacity_frames > MAX_PCM_OUTPUT_CAPACITY_FRAMES {
            return Err(PcmAudioOutputError::InvalidCapacity);
        }
        #[cfg(target_os = "ios")]
        if !objc2::available!(ios = 13.0, ..) {
            return Err(PcmAudioOutputError::UnsupportedOperatingSystem);
        }
        let queue = std::sync::Arc::new(SampleQueue::new(capacity_frames, format.channels)?);
        #[cfg(target_os = "ios")]
        let (engine, source, render_block) =
            create_native_graph(format, std::sync::Arc::clone(&queue))?;
        Ok(Self {
            format,
            queue,
            lifecycle: Lifecycle::Ready,
            #[cfg(target_os = "ios")]
            engine: Some(engine),
            #[cfg(target_os = "ios")]
            source: Some(source),
            #[cfg(target_os = "ios")]
            render_block: Some(render_block),
            #[cfg(target_os = "ios")]
            _not_send_or_sync: PhantomData,
        })
    }

    /// Copies a complete interleaved Float32 write into the queue.
    ///
    /// Queue-full and malformed writes are rejected atomically; this method never copies a partial
    /// block or retains the caller's samples. Writes may be queued before or during playback.
    pub fn write_float32_interleaved(&mut self, samples: &[f32]) -> Result<u32, PcmWriteError> {
        if self.lifecycle == Lifecycle::Stopped {
            return Err(PcmWriteError::Stopped);
        }
        if self.format.sample_format != PcmSampleFormat::Float32 {
            return Err(PcmWriteError::SampleFormatMismatch);
        }
        if samples.len() % self.format.channels as usize != 0 {
            return Err(PcmWriteError::InvalidSampleLength);
        }
        self.queue
            .push_with(samples.len(), |index| samples[index].to_bits())
    }

    /// Copies a complete interleaved Int16 write into the queue.
    ///
    /// Queue-full and malformed writes are rejected atomically; this method never copies a partial
    /// block or retains the caller's samples. Writes may be queued before or during playback.
    pub fn write_int16_interleaved(&mut self, samples: &[i16]) -> Result<u32, PcmWriteError> {
        if self.lifecycle == Lifecycle::Stopped {
            return Err(PcmWriteError::Stopped);
        }
        if self.format.sample_format != PcmSampleFormat::Int16 {
            return Err(PcmWriteError::SampleFormatMismatch);
        }
        if samples.len() % self.format.channels as usize != 0 {
            return Err(PcmWriteError::InvalidSampleLength);
        }
        self.queue
            .push_with(samples.len(), |index| u32::from(samples[index] as u16))
    }

    /// Returns an approximate count of frames queued for the next render callbacks.
    pub fn queued_frames(&self) -> u32 {
        self.queue.queued_frames()
    }

    /// Starts the native audio graph.
    pub fn start(&mut self) -> Result<(), PcmAudioOutputError> {
        match self.lifecycle {
            Lifecycle::Ready => {}
            Lifecycle::Running => return Err(PcmAudioOutputError::AlreadyStarted),
            Lifecycle::Stopped => return Err(PcmAudioOutputError::Stopped),
        }
        #[cfg(target_os = "ios")]
        {
            let Some(engine) = self.engine.as_ref() else {
                return Err(PcmAudioOutputError::Stopped);
            };
            // SAFETY: The engine graph was built from the validated immutable PCM format and is
            // retained by self. The host owns AVAudioSession policy before this native start call.
            if unsafe { engine.startAndReturnError() }.is_err() {
                // SAFETY: Stopping a retained engine after a failed start releases partial native
                // resources and does not change AVAudioSession policy.
                unsafe { engine.stop() };
                return Err(PcmAudioOutputError::EngineStartFailed);
            }
        }
        self.lifecycle = Lifecycle::Running;
        Ok(())
    }

    /// Returns whether AVAudioEngine currently reports itself as running.
    pub fn is_running(&self) -> bool {
        if self.lifecycle != Lifecycle::Running {
            return false;
        }
        #[cfg(target_os = "ios")]
        {
            if let Some(engine) = self.engine.as_ref() {
                // SAFETY: engine is retained by self and its lifecycle is serialized by Rust borrows.
                return unsafe { engine.isRunning() };
            }
        }
        false
    }

    /// Stops and tears down the graph. Repeated calls are harmless; the stream cannot restart.
    pub fn stop(&mut self) {
        if self.lifecycle == Lifecycle::Stopped {
            return;
        }
        #[cfg(target_os = "ios")]
        {
            if let Some(engine) = self.engine.as_ref() {
                // SAFETY: engine is retained by self; stop ends the render callback before graph teardown.
                unsafe { engine.stop() };
                if let Some(source) = self.source.as_ref() {
                    // SAFETY: source is attached to this engine and is retained until detachment.
                    unsafe {
                        engine.disconnectNodeOutput(source);
                        engine.detachNode(source);
                    }
                }
            }
            self.source = None;
            self.render_block = None;
            self.engine = None;
        }
        self.lifecycle = Lifecycle::Stopped;
    }
}

#[cfg(any(test, target_os = "ios"))]
impl Drop for IosPcmAudioOutput {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(target_os = "ios")]
fn create_native_graph(
    format: PcmOutputFormat,
    queue: std::sync::Arc<SampleQueue>,
) -> Result<NativeAudioGraph, PcmAudioOutputError> {
    let common_format = match format.sample_format {
        PcmSampleFormat::Int16 => AVAudioCommonFormat::PCMFormatInt16,
        PcmSampleFormat::Float32 => AVAudioCommonFormat::PCMFormatFloat32,
    };
    // SAFETY: PcmOutputFormat restricts the rate to finite positive values and channels to one or
    // two; only supported AVAudioCommonFormat values are passed.
    let native_format = unsafe {
        AVAudioFormat::initWithCommonFormat_sampleRate_channels_interleaved(
            AVAudioFormat::alloc(),
            common_format,
            format.sample_rate_hz,
            format.channels,
            true,
        )
    }
    .ok_or(PcmAudioOutputError::UnsupportedFormat)?;

    let render_queue = std::sync::Arc::clone(&queue);
    let block: RcBlock<SourceRenderBlock> =
        RcBlock::new(move |is_silence, _timestamp, frames, output| {
            render_audio(&render_queue, format, is_silence, frames, output)
        });
    // SAFETY: `block` has the exact AVAudioSourceNodeRenderBlock ABI, captures only the bounded
    // thread-safe SPSC queue and immutable format, and stays retained through native initialization.
    let source = unsafe {
        AVAudioSourceNode::initWithFormat_renderBlock(
            AVAudioSourceNode::alloc(),
            &native_format,
            RcBlock::as_ptr(&block),
        )
    };
    let engine = unsafe { AVAudioEngine::init(AVAudioEngine::alloc()) };
    // SAFETY: source is a valid public AVAudioNode; the format is valid and the engine owns the
    // attached node until it is explicitly detached during stop/drop.
    unsafe {
        engine.attachNode(&source);
        let mixer = engine.mainMixerNode();
        engine.connect_to_format(&source, &mixer, Some(&native_format));
    }
    Ok((engine, source, block))
}

#[cfg(target_os = "ios")]
fn render_audio(
    queue: &SampleQueue,
    format: PcmOutputFormat,
    is_silence: NonNull<Bool>,
    frames: u32,
    output: NonNull<AudioBufferList>,
) -> i32 {
    let Some(sample_count) = frames.checked_mul(format.channels) else {
        return kAudio_ParamError;
    };
    let bytes_per_sample = match format.sample_format {
        PcmSampleFormat::Int16 => core::mem::size_of::<i16>(),
        PcmSampleFormat::Float32 => core::mem::size_of::<f32>(),
    };
    let Some(required_bytes) = (sample_count as usize).checked_mul(bytes_per_sample) else {
        return kAudio_ParamError;
    };
    if frames == 0 {
        // SAFETY: AVAudioSourceNode provides a valid writable silence flag for every callback.
        unsafe { is_silence.as_ptr().write(Bool::YES) };
        return kAudio_NoError;
    }
    // SAFETY: AVAudioSourceNode provides a writable AudioBufferList sized for `frames` in the
    // interleaved output format fixed at graph construction.
    let list = unsafe { output.as_ref() };
    if list.mNumberBuffers != 1 {
        return kAudio_ParamError;
    }
    // SAFETY: An interleaved AudioBufferList has exactly one buffer, verified above.
    let buffer = unsafe { list.mBuffers.as_ptr().read() };
    if buffer.mNumberChannels != format.channels
        || (buffer.mDataByteSize as usize) < required_bytes
        || buffer.mData.is_null()
    {
        return kAudio_ParamError;
    }
    let data = buffer.mData.cast::<c_void>();
    let copied = render_samples(
        queue,
        sample_count,
        || {
            // SAFETY: AVFAudio promises a writable buffer with at least the requested frame count
            // and matching interleaved sample format; the checks above validate its channel/size shape.
            unsafe { ptr::write_bytes(data.cast::<u8>(), 0, required_bytes) };
        },
        |offset, sample| {
            // SAFETY: The validated output buffer is aligned for the configured AVFAudio sample type,
            // and offset is below `sample_count`, whose byte extent was checked above.
            unsafe {
                match format.sample_format {
                    PcmSampleFormat::Int16 => data
                        .cast::<i16>()
                        .add(offset as usize)
                        .write(sample as u16 as i16),
                    PcmSampleFormat::Float32 => data
                        .cast::<f32>()
                        .add(offset as usize)
                        .write(f32::from_bits(sample)),
                }
            }
        },
    );
    // SAFETY: AVAudioSourceNode provides a writable silence flag for every callback.
    unsafe { is_silence.as_ptr().write(Bool::new(copied == 0)) };
    kAudio_NoError
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_requires_finite_positive_rate_and_one_or_two_channels() {
        assert_eq!(
            PcmOutputFormat::new(f64::NAN, 1, PcmSampleFormat::Float32),
            Err(PcmOutputFormatError::InvalidSampleRate)
        );
        assert_eq!(
            PcmOutputFormat::new(f64::INFINITY, 1, PcmSampleFormat::Float32),
            Err(PcmOutputFormatError::InvalidSampleRate)
        );
        assert_eq!(
            PcmOutputFormat::new(0.0, 1, PcmSampleFormat::Float32),
            Err(PcmOutputFormatError::InvalidSampleRate)
        );
        assert_eq!(
            PcmOutputFormat::new(48_000.0, 0, PcmSampleFormat::Float32),
            Err(PcmOutputFormatError::UnsupportedChannelCount)
        );
        assert_eq!(
            PcmOutputFormat::new(48_000.0, 3, PcmSampleFormat::Float32),
            Err(PcmOutputFormatError::UnsupportedChannelCount)
        );
        assert_eq!(
            PcmOutputFormat::new(48_000.0, 2, PcmSampleFormat::Int16)
                .unwrap()
                .channels(),
            2
        );
    }

    #[test]
    fn capacity_is_bounded_and_zero_is_rejected() {
        assert!(matches!(
            SampleQueue::new(0, 1),
            Err(PcmAudioOutputError::InvalidCapacity)
        ));
        assert!(matches!(
            SampleQueue::new(MAX_PCM_OUTPUT_CAPACITY_FRAMES + 1, 1),
            Err(PcmAudioOutputError::InvalidCapacity)
        ));
        let queue = SampleQueue::new(MAX_PCM_OUTPUT_CAPACITY_FRAMES, 2).unwrap();
        assert_eq!(queue.sample_capacity, MAX_PCM_OUTPUT_CAPACITY_FRAMES * 2);
    }

    #[test]
    fn queue_copies_float_bits_fifo_and_zero_fills_underflow() {
        let queue = SampleQueue::new(3, 2).unwrap();
        let input = [0.25_f32, -0.5, f32::from_bits(0x7fc0_1234), 1.0];
        assert_eq!(
            queue
                .push_with(input.len(), |index| input[index].to_bits())
                .unwrap(),
            2
        );
        let mut output = [u32::MAX; 6];
        let output_ptr = output.as_mut_ptr();
        let copied = render_samples(
            &queue,
            output.len() as u32,
            || output.fill(0),
            |offset, sample| {
                // SAFETY: render_samples invokes this only for offsets within output.len().
                unsafe { output_ptr.add(offset as usize).write(sample) };
            },
        );
        assert_eq!(copied, 4);
        assert_eq!(&output[..4], &input.map(f32::to_bits));
        assert_eq!(&output[4..], &[0, 0]);
        assert_eq!(queue.queued_frames(), 0);
    }

    #[test]
    fn queue_preserves_signed_int16_samples() {
        let queue = SampleQueue::new(4, 1).unwrap();
        let input = [i16::MIN, -1, 0, i16::MAX];
        queue
            .push_with(input.len(), |index| u32::from(input[index] as u16))
            .unwrap();
        let mut output = [0_i16; 4];
        let copied = queue.consume_with(4, |offset, sample| {
            output[offset as usize] = sample as u16 as i16;
        });
        assert_eq!(copied, 4);
        assert_eq!(output, input);
    }

    #[test]
    fn full_queue_rejects_whole_write_without_partial_copy() {
        let queue = SampleQueue::new(2, 2).unwrap();
        queue.push_with(4, |index| index as u32 + 10).unwrap();
        assert_eq!(
            queue.push_with(2, |index| index as u32),
            Err(PcmWriteError::QueueFull)
        );
        let mut output = [0; 4];
        assert_eq!(
            queue.consume_with(4, |index, sample| output[index as usize] = sample),
            4
        );
        assert_eq!(output, [10, 11, 12, 13]);
    }

    #[test]
    fn queue_wraps_indices_without_losing_fifo_order() {
        let queue = SampleQueue::new(4, 1).unwrap();
        let near_wrap = u32::MAX - 2;
        queue.write_index.store(near_wrap, Ordering::Relaxed);
        queue.read_index.store(near_wrap, Ordering::Relaxed);
        queue.push_with(4, |index| index as u32 + 1).unwrap();
        let mut output = [0; 4];
        assert_eq!(
            queue.consume_with(4, |index, sample| output[index as usize] = sample),
            4
        );
        assert_eq!(output, [1, 2, 3, 4]);
        assert_eq!(queue.queued_frames(), 0);
    }

    #[test]
    fn host_output_surface_validates_format_length_capacity_and_lifecycle() {
        let format = PcmOutputFormat::new(48_000.0, 2, PcmSampleFormat::Float32).unwrap();
        let mut output = IosPcmAudioOutput::new(format, 2).unwrap();
        assert_eq!(
            output.write_int16_interleaved(&[1, 2]),
            Err(PcmWriteError::SampleFormatMismatch)
        );
        assert_eq!(
            output.write_float32_interleaved(&[1.0]),
            Err(PcmWriteError::InvalidSampleLength)
        );
        let mut input = [1.0, 2.0];
        assert_eq!(output.write_float32_interleaved(&input), Ok(1));
        input[0] = 9.0;
        assert_eq!(input[0], 9.0);
        let mut copied = [0_u32; 2];
        assert_eq!(
            output
                .queue
                .consume_with(2, |index, sample| copied[index as usize] = sample),
            2
        );
        assert_eq!(copied, [1.0_f32.to_bits(), 2.0_f32.to_bits()]);
        assert_eq!(output.queued_frames(), 0);
        assert_eq!(output.start(), Ok(()));
        assert_eq!(output.start(), Err(PcmAudioOutputError::AlreadyStarted));
        output.stop();
        output.stop();
        assert_eq!(
            output.write_float32_interleaved(&[3.0, 4.0]),
            Err(PcmWriteError::Stopped)
        );
        assert_eq!(output.start(), Err(PcmAudioOutputError::Stopped));
    }
}
