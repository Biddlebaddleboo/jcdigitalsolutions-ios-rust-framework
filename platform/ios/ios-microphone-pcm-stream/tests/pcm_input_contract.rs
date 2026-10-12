use framework_audio::{
    PcmInputBufferError, PcmInputFormat, PcmInputFormatError, PcmInputStream, PcmSampleFormat,
    PcmSampleLayout,
};

struct FakeInput {
    format: PcmInputFormat,
    samples: [f32; 6],
    consumed: usize,
    dropped: u64,
}

impl PcmInputStream for FakeInput {
    type Error = PcmInputBufferError;

    fn format(&self) -> Option<PcmInputFormat> {
        Some(self.format)
    }

    fn try_read_interleaved(&mut self, output: &mut [f32]) -> Result<u32, Self::Error> {
        let requested = self.format.frames_for_interleaved_samples(output.len())? as usize;
        let channels = self.format.channels() as usize;
        let available = (self.samples.len() - self.consumed) / channels;
        let frames = requested.min(available);
        let sample_count = frames * channels;
        output[..sample_count]
            .copy_from_slice(&self.samples[self.consumed..self.consumed + sample_count]);
        self.consumed += sample_count;
        Ok(frames as u32)
    }

    fn dropped_frames(&self) -> u64 {
        self.dropped
    }
}

fn read_generic<S: PcmInputStream>(stream: &mut S, output: &mut [f32]) -> Result<u32, S::Error> {
    stream.try_read_interleaved(output)
}

fn fake_input() -> FakeInput {
    FakeInput {
        format: PcmInputFormat::float32_interleaved(48_000.0, 2).unwrap(),
        samples: [1.0, 10.0, 2.0, 20.0, 3.0, 30.0],
        consumed: 0,
        dropped: 4,
    }
}

#[test]
fn format_is_typed_and_frames_count_channels() {
    let format = PcmInputFormat::float32_interleaved(48_000.0, 2).unwrap();

    assert_eq!(format.sample_format(), PcmSampleFormat::Float32);
    assert_eq!(format.sample_layout(), PcmSampleLayout::Interleaved);
    assert_eq!(format.sample_rate_hz(), 48_000.0);
    assert_eq!(format.channels(), 2);
    assert_eq!(format.frames_for_interleaved_samples(6), Ok(3));
}

#[test]
fn invalid_portable_formats_are_rejected() {
    assert_eq!(
        PcmInputFormat::float32_interleaved(f64::INFINITY, 1),
        Err(PcmInputFormatError::InvalidSampleRate)
    );
    assert_eq!(
        PcmInputFormat::float32_interleaved(48_000.0, 0),
        Err(PcmInputFormatError::ZeroChannels)
    );
}

#[test]
fn generic_stream_reads_interleaved_frames_and_preserves_output_tail() {
    let mut input = fake_input();
    let mut output = [-1.0; 8];

    assert_eq!(input.format(), Some(input.format));
    assert_eq!(read_generic(&mut input, &mut output), Ok(3));
    assert_eq!(output, [1.0, 10.0, 2.0, 20.0, 3.0, 30.0, -1.0, -1.0]);
    assert_eq!(input.dropped_frames(), 4);
}

#[test]
fn partial_stereo_frame_is_rejected_without_output_or_cursor_change() {
    let mut input = fake_input();
    let mut output = [99.0; 3];

    assert_eq!(
        input.try_read_interleaved(&mut output),
        Err(PcmInputBufferError::IncompleteFrame)
    );
    assert_eq!(output, [99.0; 3]);
    assert_eq!(input.consumed, 0);
}
