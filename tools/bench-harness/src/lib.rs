#![forbid(unsafe_code)]

//! Dependency-free timing and report primitives for performance experiments.
//!
//! This crate does not register a framework workload, infer counters, or select a Rust default.
//! Host timings are advisory; only representative physical Apple-device Release measurements can
//! support iOS replacement decisions.

use std::fmt::Write as _;
use std::hint::black_box;
use std::time::Instant;

/// Minimum measured sample count used for p95/p99 reporting.
pub const MIN_SAMPLE_COUNT: usize = 100;

/// Caller-declared context for the measured samples.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MeasurementContext {
    /// Host microbenchmark. This is advisory, not authoritative iOS replacement evidence.
    HostMicrobenchmark {
        /// Host/OS label supplied by the caller.
        host_label: String,
    },
    /// Release measurements from a representative physical Apple device.
    RepresentativeAppleDeviceRelease {
        /// Device model supplied by the caller.
        device_model: String,
        /// OS version supplied by the caller.
        os_version: String,
        /// SDK version supplied by the caller.
        sdk_version: String,
    },
}

impl MeasurementContext {
    fn validate(&self) -> Result<(), BenchmarkError> {
        match self {
            Self::HostMicrobenchmark { host_label } => require_non_empty("host label", host_label),
            Self::RepresentativeAppleDeviceRelease {
                device_model,
                os_version,
                sdk_version,
            } => {
                require_non_empty("device model", device_model)?;
                require_non_empty("OS version", os_version)?;
                require_non_empty("SDK version", sdk_version)
            }
        }
    }

    fn write_json_fields(&self, output: &mut String) {
        match self {
            Self::HostMicrobenchmark { host_label } => {
                output.push_str("\"measurement_class\":\"advisory-host-microbenchmark\",");
                output.push_str("\"host_label\":");
                write_json_string(host_label, output);
            }
            Self::RepresentativeAppleDeviceRelease {
                device_model,
                os_version,
                sdk_version,
            } => {
                output.push_str("\"measurement_class\":\"representative-apple-device-release\",");
                output.push_str("\"device_model\":");
                write_json_string(device_model, output);
                output.push_str(",\"os_version\":");
                write_json_string(os_version, output);
                output.push_str(",\"sdk_version\":");
                write_json_string(sdk_version, output);
            }
        }
    }
}

/// Explicit workload/build metadata. Values are provided by the benchmark owner, not inferred.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BenchmarkMetadata {
    /// Rust target triple, for example `aarch64-apple-ios`.
    pub target_triple: String,
    /// Optimization mode/profile and any relevant compiler flags.
    pub optimization_mode: String,
    /// Stable benchmark/workload identifier.
    pub workload_id: String,
    /// Workload input shape, such as `1000-byte UTF-8 payload`.
    pub input_shape: String,
    /// Caller-declared host/device evidence context.
    pub context: MeasurementContext,
}

impl BenchmarkMetadata {
    /// Create benchmark metadata without inferring target, build profile, or evidence context.
    pub fn new(
        target_triple: impl Into<String>,
        optimization_mode: impl Into<String>,
        workload_id: impl Into<String>,
        input_shape: impl Into<String>,
        context: MeasurementContext,
    ) -> Self {
        Self {
            target_triple: target_triple.into(),
            optimization_mode: optimization_mode.into(),
            workload_id: workload_id.into(),
            input_shape: input_shape.into(),
            context,
        }
    }

    fn validate(&self) -> Result<(), BenchmarkError> {
        require_non_empty("target triple", &self.target_triple)?;
        require_non_empty("optimization mode", &self.optimization_mode)?;
        require_non_empty("workload id", &self.workload_id)?;
        require_non_empty("input shape", &self.input_shape)?;
        self.context.validate()
    }
}

/// Optional counters explicitly supplied by workload instrumentation.
///
/// `None` means unmeasured/unsupplied; it is serialized as JSON `null`. The harness never
/// estimates or fabricates these values.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExplicitCounters {
    /// Explicitly measured allocation count.
    pub allocation_count: Option<u64>,
    /// Explicitly measured copy count.
    pub copy_count: Option<u64>,
    /// Explicitly measured copied bytes.
    pub bytes_copied: Option<u64>,
    /// Explicitly measured energy in nanojoules.
    pub energy_nanojoules: Option<u64>,
}

/// Validated benchmark samples with deterministic latency summaries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BenchmarkRecord {
    /// Supplied target/build/workload metadata.
    metadata: BenchmarkMetadata,
    /// Number of warmup invocations performed before timing.
    warmup_sample_count: usize,
    /// Raw measured latency values in nanoseconds, in capture order.
    raw_samples_ns: Vec<u64>,
    /// Measured-value count; always equal to `raw_samples_ns.len()`.
    sample_count: usize,
    /// Median latency in nanoseconds; even-count median is integer-truncated.
    median_ns: u64,
    /// Nearest-rank 95th percentile latency in nanoseconds.
    p95_ns: u64,
    /// Nearest-rank 99th percentile latency in nanoseconds.
    p99_ns: u64,
    /// Explicit caller-supplied counters; missing values remain `None`.
    counters: ExplicitCounters,
}

impl BenchmarkRecord {
    /// Return the caller-supplied target, build, workload, and evidence context.
    pub fn metadata(&self) -> &BenchmarkMetadata {
        &self.metadata
    }

    /// Return the number of warmup invocations performed before timing.
    pub fn warmup_sample_count(&self) -> usize {
        self.warmup_sample_count
    }

    /// Return raw latency samples in capture order.
    pub fn raw_samples_ns(&self) -> &[u64] {
        &self.raw_samples_ns
    }

    /// Return the number of validated timing samples.
    pub fn sample_count(&self) -> usize {
        self.sample_count
    }

    /// Return the integer-truncated median latency in nanoseconds.
    pub fn median_ns(&self) -> u64 {
        self.median_ns
    }

    /// Return the nearest-rank 95th percentile latency in nanoseconds.
    pub fn p95_ns(&self) -> u64 {
        self.p95_ns
    }

    /// Return the nearest-rank 99th percentile latency in nanoseconds.
    pub fn p99_ns(&self) -> u64 {
        self.p99_ns
    }

    /// Return only counters supplied by workload instrumentation.
    pub fn counters(&self) -> &ExplicitCounters {
        &self.counters
    }

    /// Validate raw samples and summarize them using the count derived from the vector.
    pub fn from_samples(
        metadata: BenchmarkMetadata,
        warmup_sample_count: usize,
        raw_samples_ns: Vec<u64>,
        counters: ExplicitCounters,
    ) -> Result<Self, BenchmarkError> {
        metadata.validate()?;
        validate_sample_count(raw_samples_ns.len())?;

        let mut sorted_samples = raw_samples_ns.clone();
        sorted_samples.sort_unstable();
        let sample_count = sorted_samples.len();
        let median_ns = median(&sorted_samples);
        let p95_ns = nearest_rank_percentile(&sorted_samples, 95);
        let p99_ns = nearest_rank_percentile(&sorted_samples, 99);

        Ok(Self {
            metadata,
            warmup_sample_count,
            raw_samples_ns,
            sample_count,
            median_ns,
            p95_ns,
            p99_ns,
            counters,
        })
    }

    /// Run warmups and timed calls to one workload closure, returning raw nanosecond samples.
    ///
    /// Each sample includes `Instant` and `black_box` overhead. Batch tiny operations when timer
    /// overhead would dominate. This helper does not collect allocation/copy/energy counters.
    pub fn sample_workload<F, R>(
        warmup_sample_count: usize,
        sample_count: usize,
        mut workload: F,
    ) -> Result<Vec<u64>, BenchmarkError>
    where
        F: FnMut() -> R,
    {
        validate_sample_count(sample_count)?;
        for _ in 0..warmup_sample_count {
            black_box(workload());
        }

        let mut samples = Vec::with_capacity(sample_count);
        for _ in 0..sample_count {
            let start = Instant::now();
            drop(black_box(workload()));
            let elapsed = start.elapsed().as_nanos();
            let elapsed = u64::try_from(elapsed).map_err(|_| BenchmarkError::ElapsedTooLarge)?;
            samples.push(elapsed);
        }
        Ok(samples)
    }

    /// Encode the record as deterministic JSON without an external serialization dependency.
    pub fn to_json(&self) -> String {
        let mut output = String::new();
        output.push('{');
        output.push_str("\"schema_version\":1,");
        output.push_str("\"target_triple\":");
        write_json_string(&self.metadata.target_triple, &mut output);
        output.push_str(",\"optimization_mode\":");
        write_json_string(&self.metadata.optimization_mode, &mut output);
        output.push_str(",\"workload_id\":");
        write_json_string(&self.metadata.workload_id, &mut output);
        output.push_str(",\"input_shape\":");
        write_json_string(&self.metadata.input_shape, &mut output);
        output.push(',');
        self.metadata.context.write_json_fields(&mut output);
        output.push_str(",\"warmup_sample_count\":");
        write!(output, "{}", self.warmup_sample_count).expect("writing to String cannot fail");
        output.push_str(",\"sample_count\":");
        write!(output, "{}", self.sample_count).expect("writing to String cannot fail");
        output.push_str(",\"raw_samples_ns\":[");
        for (index, value) in self.raw_samples_ns.iter().enumerate() {
            if index != 0 {
                output.push(',');
            }
            write!(output, "{value}").expect("writing to String cannot fail");
        }
        output.push_str("],\"median_ns\":");
        write!(output, "{}", self.median_ns).expect("writing to String cannot fail");
        output.push_str(",\"p95_ns\":");
        write!(output, "{}", self.p95_ns).expect("writing to String cannot fail");
        output.push_str(",\"p99_ns\":");
        write!(output, "{}", self.p99_ns).expect("writing to String cannot fail");
        output.push_str(",\"percentile_method\":\"nearest-rank\",\"counters\":{");
        write_optional_counter(
            "allocation_count",
            self.counters.allocation_count,
            &mut output,
        );
        output.push(',');
        write_optional_counter("copy_count", self.counters.copy_count, &mut output);
        output.push(',');
        write_optional_counter("bytes_copied", self.counters.bytes_copied, &mut output);
        output.push(',');
        write_optional_counter(
            "energy_nanojoules",
            self.counters.energy_nanojoules,
            &mut output,
        );
        output.push_str("}}\n");
        output
    }
}

/// Invalid metadata or samples.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BenchmarkError {
    /// A required user-provided metadata label was empty.
    EmptyMetadata(&'static str),
    /// Fewer than [`MIN_SAMPLE_COUNT`] timing samples were supplied.
    InsufficientSamples {
        /// Actual number of supplied samples.
        actual: usize,
        /// Required minimum sample count.
        minimum: usize,
    },
    /// An elapsed nanosecond value did not fit in the report's `u64` field.
    ElapsedTooLarge,
}

fn require_non_empty(label: &'static str, value: &str) -> Result<(), BenchmarkError> {
    if value.is_empty() {
        Err(BenchmarkError::EmptyMetadata(label))
    } else {
        Ok(())
    }
}

fn validate_sample_count(sample_count: usize) -> Result<(), BenchmarkError> {
    if sample_count < MIN_SAMPLE_COUNT {
        Err(BenchmarkError::InsufficientSamples {
            actual: sample_count,
            minimum: MIN_SAMPLE_COUNT,
        })
    } else {
        Ok(())
    }
}

fn median(sorted_samples: &[u64]) -> u64 {
    let middle = sorted_samples.len() / 2;
    if sorted_samples.len() % 2 == 0 {
        ((u128::from(sorted_samples[middle - 1]) + u128::from(sorted_samples[middle])) / 2) as u64
    } else {
        sorted_samples[middle]
    }
}

fn nearest_rank_percentile(sorted_samples: &[u64], percentile: usize) -> u64 {
    let sample_count = sorted_samples.len() as u128;
    let percentile = percentile as u128;
    let rank = usize::try_from((sample_count * percentile).div_ceil(100))
        .expect("nearest rank cannot exceed the sample count");
    sorted_samples[rank - 1]
}

fn write_optional_counter(name: &str, value: Option<u64>, output: &mut String) {
    write_json_string(name, output);
    output.push(':');
    match value {
        Some(value) => write!(output, "{value}").expect("writing to String cannot fail"),
        None => output.push_str("null"),
    }
}

fn write_json_string(value: &str, output: &mut String) {
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{0008}' => output.push_str("\\b"),
            '\u{000c}' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character <= '\u{001f}' => {
                write!(output, "\\u{:04x}", character as u32)
                    .expect("writing to String cannot fail");
            }
            character => output.push(character),
        }
    }
    output.push('"');
}

#[cfg(test)]
mod tests {
    use super::{
        BenchmarkError, BenchmarkMetadata, BenchmarkRecord, ExplicitCounters, MIN_SAMPLE_COUNT,
        MeasurementContext,
    };
    use std::cell::Cell;

    fn metadata() -> BenchmarkMetadata {
        BenchmarkMetadata::new(
            "x86_64-apple-darwin",
            "release+lto=thin",
            "buffer-copy",
            "4096-byte input",
            MeasurementContext::HostMicrobenchmark {
                host_label: "unit-test host".into(),
            },
        )
    }

    #[test]
    fn summary_has_stable_nearest_rank_statistics_and_preserves_raw_order() {
        let raw_samples_ns = (1..=MIN_SAMPLE_COUNT as u64).rev().collect::<Vec<_>>();
        let record = BenchmarkRecord::from_samples(
            metadata(),
            25,
            raw_samples_ns.clone(),
            ExplicitCounters {
                allocation_count: Some(0),
                copy_count: Some(1),
                bytes_copied: Some(4096),
                energy_nanojoules: None,
            },
        )
        .unwrap();

        assert_eq!(record.sample_count(), 100);
        assert_eq!(record.raw_samples_ns(), raw_samples_ns);
        assert_eq!(record.median_ns(), 50);
        assert_eq!(record.p95_ns(), 95);
        assert_eq!(record.p99_ns(), 99);
    }

    #[test]
    fn rejects_too_few_samples_and_empty_metadata() {
        assert_eq!(
            BenchmarkRecord::from_samples(
                metadata(),
                0,
                vec![1; MIN_SAMPLE_COUNT - 1],
                ExplicitCounters::default(),
            ),
            Err(BenchmarkError::InsufficientSamples {
                actual: MIN_SAMPLE_COUNT - 1,
                minimum: MIN_SAMPLE_COUNT,
            })
        );

        let mut invalid = metadata();
        invalid.target_triple.clear();
        assert_eq!(
            BenchmarkRecord::from_samples(
                invalid,
                0,
                vec![1; MIN_SAMPLE_COUNT],
                ExplicitCounters::default(),
            ),
            Err(BenchmarkError::EmptyMetadata("target triple"))
        );
    }

    #[test]
    fn json_contains_required_metadata_raw_samples_and_explicit_counters() {
        let record = BenchmarkRecord::from_samples(
            metadata(),
            4,
            vec![7; MIN_SAMPLE_COUNT],
            ExplicitCounters {
                allocation_count: Some(0),
                ..ExplicitCounters::default()
            },
        )
        .unwrap();
        let json = record.to_json();

        assert!(json.contains("\"target_triple\":\"x86_64-apple-darwin\""));
        assert!(json.contains("\"optimization_mode\":\"release+lto=thin\""));
        assert!(json.contains("\"workload_id\":\"buffer-copy\""));
        assert!(json.contains("\"input_shape\":\"4096-byte input\""));
        assert!(json.contains("\"warmup_sample_count\":4"));
        assert!(json.contains("\"sample_count\":100"));
        assert!(json.contains("\"median_ns\":7"));
        assert!(json.contains("\"p95_ns\":7"));
        assert!(json.contains("\"p99_ns\":7"));
        assert!(json.contains("\"allocation_count\":0"));
        assert!(json.contains("\"copy_count\":null"));
        assert!(json.contains("\"measurement_class\":\"advisory-host-microbenchmark\""));
        assert!(json.contains("\"host_label\":\"unit-test host\""));
    }

    #[test]
    fn workload_sampler_runs_explicit_warmups_and_samples() {
        let invocations = Cell::new(0);
        let samples = BenchmarkRecord::sample_workload(3, MIN_SAMPLE_COUNT, || {
            invocations.set(invocations.get() + 1);
        })
        .unwrap();

        assert_eq!(samples.len(), MIN_SAMPLE_COUNT);
        assert_eq!(invocations.get(), MIN_SAMPLE_COUNT + 3);
    }

    #[test]
    fn json_strings_escape_quotes_slashes_and_control_characters() {
        let mut escaped = String::new();
        super::write_json_string("quote \" slash \\ line\n", &mut escaped);
        assert_eq!(escaped, "\"quote \\\" slash \\\\ line\\n\"");
    }
}
