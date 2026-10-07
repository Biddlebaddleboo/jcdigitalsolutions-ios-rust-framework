#![forbid(unsafe_code)]

//! Dependency-free primitives for fixed-case differential correctness checks.
//!
//! The harness contains no Apple implementation and registers no suites. A caller supplies a
//! statically dispatched reference adapter and candidate adapter, fixed inputs, and any explicit
//! top-level result fields that the suite permits both results to normalize.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

/// JSON-compatible fixed input or observable result value.
#[derive(Clone, Debug)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Integer(i64),
    Unsigned(u64),
    Float(f64),
    String(String),
    Array(Vec<Self>),
    Object(BTreeMap<String, Self>),
}

impl PartialEq for JsonValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Bool(left), Self::Bool(right)) => left == right,
            (Self::Integer(left), Self::Integer(right)) => left == right,
            (Self::Unsigned(left), Self::Unsigned(right)) => left == right,
            (Self::Float(left), Self::Float(right)) => left.to_bits() == right.to_bits(),
            (Self::String(left), Self::String(right)) => left == right,
            (Self::Array(left), Self::Array(right)) => left == right,
            (Self::Object(left), Self::Object(right)) => left == right,
            _ => false,
        }
    }
}

impl Eq for JsonValue {}

impl JsonValue {
    /// Encode this value as deterministic JSON. Object keys use sorted map order.
    pub fn to_json(&self) -> Result<String, JsonError> {
        let mut output = String::new();
        self.write_json(&mut output)?;
        Ok(output)
    }

    fn write_json(&self, output: &mut String) -> Result<(), JsonError> {
        match self {
            Self::Null => output.push_str("null"),
            Self::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
            Self::Integer(value) => {
                write!(output, "{value}").expect("writing to String cannot fail")
            }
            Self::Unsigned(value) => {
                write!(output, "{value}").expect("writing to String cannot fail")
            }
            Self::Float(value) => {
                if !value.is_finite() {
                    return Err(JsonError::NonFiniteFloat);
                }
                let value = value.to_string();
                output.push_str(&value);
                if !value.contains('.') && !value.contains('e') && !value.contains('E') {
                    output.push_str(".0");
                }
            }
            Self::String(value) => write_json_string(value, output),
            Self::Array(values) => {
                output.push('[');
                for (index, value) in values.iter().enumerate() {
                    if index != 0 {
                        output.push(',');
                    }
                    value.write_json(output)?;
                }
                output.push(']');
            }
            Self::Object(fields) => {
                output.push('{');
                for (index, (key, value)) in fields.iter().enumerate() {
                    if index != 0 {
                        output.push(',');
                    }
                    write_json_string(key, output);
                    output.push(':');
                    value.write_json(output)?;
                }
                output.push('}');
            }
        }
        Ok(())
    }
}

/// JSON serialization failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JsonError {
    /// JSON has no representation for NaN or positive/negative infinity.
    NonFiniteFloat,
}

/// A contract-relevant error returned by either implementation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ErrorOutcome {
    /// Stable suite-defined semantic error category.
    pub category: String,
    /// Native error code when the capability contract requires it.
    pub native_code: Option<i64>,
}

impl ErrorOutcome {
    /// Create an error outcome while preserving its category and optional native code.
    pub fn new(category: impl Into<String>, native_code: Option<i64>) -> Self {
        Self {
            category: category.into(),
            native_code,
        }
    }
}

/// Typed success/error result compared by the harness.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// Successful semantic result.
    Success(JsonValue),
    /// Failure with category and optional native code.
    Error(ErrorOutcome),
}

impl Outcome {
    fn normalized(&self, normalization: &Normalization) -> Self {
        match self {
            Self::Success(JsonValue::Object(fields)) => {
                let mut fields = fields.clone();
                for field in &normalization.ignored_top_level_fields {
                    fields.remove(field);
                }
                Self::Success(JsonValue::Object(fields))
            }
            other => other.clone(),
        }
    }

    fn to_json_value(&self) -> JsonValue {
        let mut fields = BTreeMap::new();
        match self {
            Self::Success(value) => {
                fields.insert("kind".into(), JsonValue::String("success".into()));
                fields.insert("value".into(), value.clone());
            }
            Self::Error(error) => {
                fields.insert("kind".into(), JsonValue::String("error".into()));
                fields.insert("category".into(), JsonValue::String(error.category.clone()));
                fields.insert(
                    "native_code".into(),
                    error
                        .native_code
                        .map(JsonValue::Integer)
                        .unwrap_or(JsonValue::Null),
                );
            }
        }
        JsonValue::Object(fields)
    }
}

/// Explicit suite-level result normalization.
///
/// Only named top-level fields in successful object results are ignored. Error category and
/// native code are never normalized. The sorted field list is copied into every report/mismatch.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Normalization {
    ignored_top_level_fields: Vec<String>,
}

impl Normalization {
    /// Ignore these named top-level fields in successful object results from both adapters.
    pub fn ignore_top_level_fields(fields: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let fields = fields
            .into_iter()
            .map(Into::into)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        Self {
            ignored_top_level_fields: fields,
        }
    }

    /// Return the exact fields that this suite normalizes.
    pub fn fields(&self) -> &[String] {
        &self.ignored_top_level_fields
    }
}

/// One deterministic input case.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Case {
    /// Stable case identifier suitable for regression fixtures.
    pub id: String,
    /// Input supplied identically to both adapters.
    pub input: JsonValue,
}

impl Case {
    /// Create a fixed-input case.
    pub fn new(id: impl Into<String>, input: JsonValue) -> Self {
        Self {
            id: id.into(),
            input,
        }
    }
}

/// Fixed suite metadata and cases.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Suite {
    /// Stable suite identifier.
    pub id: String,
    /// Apple OS/runtime label for this run, supplied by the suite owner.
    pub os_label: String,
    /// SDK label for this run, supplied by the suite owner.
    pub sdk_label: String,
    /// Explicit normalization permitted by this suite.
    pub normalization: Normalization,
    /// Fixed cases; generated inputs should be materialized as cases by the suite owner.
    pub cases: Vec<Case>,
}

impl Suite {
    /// Create a suite from fixed cases and explicit labels.
    pub fn new(
        id: impl Into<String>,
        os_label: impl Into<String>,
        sdk_label: impl Into<String>,
        normalization: Normalization,
        cases: Vec<Case>,
    ) -> Self {
        Self {
            id: id.into(),
            os_label: os_label.into(),
            sdk_label: sdk_label.into(),
            normalization,
            cases,
        }
    }
}

/// Statically dispatched reference/candidate adapter.
pub trait Adapter {
    /// Execute one fixed input and return its semantic result.
    fn run(&self, input: &JsonValue) -> Outcome;
}

impl<F> Adapter for F
where
    F: Fn(&JsonValue) -> Outcome,
{
    fn run(&self, input: &JsonValue) -> Outcome {
        self(input)
    }
}

/// Result record for a case that differs after explicit normalization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MismatchRecord {
    /// Stable input case identifier.
    pub case_id: String,
    /// Identical input passed to both adapters.
    pub input: JsonValue,
    /// Raw reference result before normalization.
    pub reference_result: Outcome,
    /// Raw candidate result before normalization.
    pub candidate_result: Outcome,
    /// Reference result after the suite's explicit normalization.
    pub normalized_reference_result: Outcome,
    /// Candidate result after the suite's explicit normalization.
    pub normalized_candidate_result: Outcome,
    /// OS/runtime label supplied by the suite owner.
    pub os_label: String,
    /// SDK label supplied by the suite owner.
    pub sdk_label: String,
    /// Every field ignored for this comparison.
    pub normalized_fields: Vec<String>,
}

/// Differential result for one fixed suite.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParityReport {
    /// Suite identifier.
    pub suite_id: String,
    /// OS/runtime label supplied by the suite owner.
    pub os_label: String,
    /// SDK label supplied by the suite owner.
    pub sdk_label: String,
    /// Every field normalized for the suite.
    pub normalized_fields: Vec<String>,
    /// Identifiers of cases whose normalized outcomes matched.
    pub matching_case_ids: Vec<String>,
    /// Full evidence records for all mismatches.
    pub mismatches: Vec<MismatchRecord>,
}

impl ParityReport {
    /// Number of fixed cases that matched after explicit normalization.
    pub fn matching_case_count(&self) -> usize {
        self.matching_case_ids.len()
    }

    /// Encode the report as deterministic JSON.
    pub fn to_json(&self) -> Result<String, JsonError> {
        self.as_json_value().to_json()
    }

    fn as_json_value(&self) -> JsonValue {
        let mut fields = BTreeMap::new();
        fields.insert("schema_version".into(), JsonValue::Unsigned(1));
        fields.insert("suite_id".into(), JsonValue::String(self.suite_id.clone()));
        fields.insert("os_label".into(), JsonValue::String(self.os_label.clone()));
        fields.insert(
            "sdk_label".into(),
            JsonValue::String(self.sdk_label.clone()),
        );
        fields.insert(
            "normalized_fields".into(),
            strings_to_json(&self.normalized_fields),
        );
        fields.insert(
            "matching_case_ids".into(),
            strings_to_json(&self.matching_case_ids),
        );
        fields.insert(
            "mismatches".into(),
            JsonValue::Array(
                self.mismatches
                    .iter()
                    .map(MismatchRecord::as_json_value)
                    .collect(),
            ),
        );
        JsonValue::Object(fields)
    }
}

impl MismatchRecord {
    fn as_json_value(&self) -> JsonValue {
        let mut fields = BTreeMap::new();
        fields.insert("case_id".into(), JsonValue::String(self.case_id.clone()));
        fields.insert("input".into(), self.input.clone());
        fields.insert(
            "reference_result".into(),
            self.reference_result.to_json_value(),
        );
        fields.insert(
            "candidate_result".into(),
            self.candidate_result.to_json_value(),
        );
        fields.insert(
            "normalized_reference_result".into(),
            self.normalized_reference_result.to_json_value(),
        );
        fields.insert(
            "normalized_candidate_result".into(),
            self.normalized_candidate_result.to_json_value(),
        );
        fields.insert("os_label".into(), JsonValue::String(self.os_label.clone()));
        fields.insert(
            "sdk_label".into(),
            JsonValue::String(self.sdk_label.clone()),
        );
        fields.insert(
            "normalized_fields".into(),
            strings_to_json(&self.normalized_fields),
        );
        JsonValue::Object(fields)
    }
}

/// Invalid fixed-suite configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SuiteError {
    /// A required suite/case label was empty.
    EmptyLabel(&'static str),
    /// A suite needs at least one fixed input case.
    NoCases,
    /// Case identifiers must be unique within one suite.
    DuplicateCaseId(String),
}

/// Execute the fixed inputs against a statically selected reference/candidate pair.
pub fn compare_suite<R, C>(
    suite: &Suite,
    reference: &R,
    candidate: &C,
) -> Result<ParityReport, SuiteError>
where
    R: Adapter,
    C: Adapter,
{
    validate_suite(suite)?;
    let normalized_fields = suite.normalization.fields().to_vec();
    let mut matching_case_ids = Vec::new();
    let mut mismatches = Vec::new();

    for case in &suite.cases {
        let reference_result = reference.run(&case.input);
        let candidate_result = candidate.run(&case.input);
        let normalized_reference_result = reference_result.normalized(&suite.normalization);
        let normalized_candidate_result = candidate_result.normalized(&suite.normalization);
        if normalized_reference_result == normalized_candidate_result {
            matching_case_ids.push(case.id.clone());
        } else {
            mismatches.push(MismatchRecord {
                case_id: case.id.clone(),
                input: case.input.clone(),
                reference_result,
                candidate_result,
                normalized_reference_result,
                normalized_candidate_result,
                os_label: suite.os_label.clone(),
                sdk_label: suite.sdk_label.clone(),
                normalized_fields: normalized_fields.clone(),
            });
        }
    }

    Ok(ParityReport {
        suite_id: suite.id.clone(),
        os_label: suite.os_label.clone(),
        sdk_label: suite.sdk_label.clone(),
        normalized_fields,
        matching_case_ids,
        mismatches,
    })
}

fn validate_suite(suite: &Suite) -> Result<(), SuiteError> {
    for (name, value) in [
        ("suite id", suite.id.as_str()),
        ("OS label", suite.os_label.as_str()),
        ("SDK label", suite.sdk_label.as_str()),
    ] {
        if value.is_empty() {
            return Err(SuiteError::EmptyLabel(name));
        }
    }
    if suite.cases.is_empty() {
        return Err(SuiteError::NoCases);
    }

    let mut case_ids = BTreeSet::new();
    for case in &suite.cases {
        if case.id.is_empty() {
            return Err(SuiteError::EmptyLabel("case id"));
        }
        if !case_ids.insert(case.id.as_str()) {
            return Err(SuiteError::DuplicateCaseId(case.id.clone()));
        }
    }
    Ok(())
}

fn strings_to_json(values: &[String]) -> JsonValue {
    JsonValue::Array(values.iter().cloned().map(JsonValue::String).collect())
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
        Case, ErrorOutcome, JsonValue, Normalization, Outcome, Suite, SuiteError, compare_suite,
    };
    use std::collections::BTreeMap;

    fn suite(normalization: Normalization) -> Suite {
        Suite::new(
            "files.read",
            "iOS 26.5",
            "iPhoneOS SDK 26.5",
            normalization,
            vec![Case::new("case-1", JsonValue::String("fixture.txt".into()))],
        )
    }

    fn object(fields: impl IntoIterator<Item = (&'static str, JsonValue)>) -> JsonValue {
        JsonValue::Object(
            fields
                .into_iter()
                .map(|(key, value)| (key.into(), value))
                .collect::<BTreeMap<_, _>>(),
        )
    }

    #[test]
    fn equal_success_values_match() {
        let suite = suite(Normalization::default());
        let reference = |_: &JsonValue| Outcome::Success(JsonValue::Unsigned(12));
        let candidate = |_: &JsonValue| Outcome::Success(JsonValue::Unsigned(12));

        let report = compare_suite(&suite, &reference, &candidate).unwrap();

        assert_eq!(report.matching_case_count(), 1);
        assert!(report.mismatches.is_empty());
    }

    #[test]
    fn unequal_values_keep_input_results_and_platform_labels() {
        let suite = suite(Normalization::default());
        let reference = |_: &JsonValue| Outcome::Success(JsonValue::Unsigned(12));
        let candidate = |_: &JsonValue| Outcome::Success(JsonValue::Unsigned(13));

        let report = compare_suite(&suite, &reference, &candidate).unwrap();
        let mismatch = &report.mismatches[0];

        assert_eq!(mismatch.case_id, "case-1");
        assert_eq!(mismatch.input, JsonValue::String("fixture.txt".into()));
        assert_eq!(
            mismatch.reference_result,
            Outcome::Success(JsonValue::Unsigned(12))
        );
        assert_eq!(
            mismatch.candidate_result,
            Outcome::Success(JsonValue::Unsigned(13))
        );
        assert_eq!(mismatch.os_label, "iOS 26.5");
        assert_eq!(mismatch.sdk_label, "iPhoneOS SDK 26.5");
    }

    #[test]
    fn unequal_errors_preserve_category_and_native_code() {
        let suite = suite(Normalization::default());
        let reference = |_: &JsonValue| Outcome::Error(ErrorOutcome::new("not-found", Some(-43)));
        let candidate =
            |_: &JsonValue| Outcome::Error(ErrorOutcome::new("permission-denied", Some(-54)));

        let report = compare_suite(&suite, &reference, &candidate).unwrap();
        let mismatch = &report.mismatches[0];

        assert_eq!(
            mismatch.normalized_reference_result,
            Outcome::Error(ErrorOutcome::new("not-found", Some(-43)))
        );
        assert_eq!(
            mismatch.normalized_candidate_result,
            Outcome::Error(ErrorOutcome::new("permission-denied", Some(-54)))
        );
    }

    #[test]
    fn value_normalization_never_ignores_error_categories_or_native_codes() {
        let suite = suite(Normalization::ignore_top_level_fields(["native_code"]));
        let reference = |_: &JsonValue| Outcome::Error(ErrorOutcome::new("failed", Some(-1)));
        let candidate = |_: &JsonValue| Outcome::Error(ErrorOutcome::new("failed", Some(-2)));

        let report = compare_suite(&suite, &reference, &candidate).unwrap();

        assert!(report.matching_case_ids.is_empty());
        assert_eq!(report.mismatches.len(), 1);
        assert_eq!(report.mismatches[0].normalized_fields, ["native_code"]);
    }

    #[test]
    fn explicit_normalization_is_applied_and_recorded() {
        let normalization = Normalization::ignore_top_level_fields(["generated_at"]);
        let suite = suite(normalization);
        let reference = |_: &JsonValue| {
            Outcome::Success(object([
                ("value", JsonValue::Integer(7)),
                ("generated_at", JsonValue::Integer(100)),
            ]))
        };
        let candidate = |_: &JsonValue| {
            Outcome::Success(object([
                ("value", JsonValue::Integer(7)),
                ("generated_at", JsonValue::Integer(101)),
            ]))
        };

        let report = compare_suite(&suite, &reference, &candidate).unwrap();

        assert_eq!(report.matching_case_count(), 1);
        assert_eq!(report.normalized_fields, ["generated_at"]);
        assert!(report.to_json().unwrap().contains("generated_at"));
    }

    #[test]
    fn suite_rejects_duplicate_case_ids() {
        let suite = Suite::new(
            "suite",
            "iOS",
            "SDK",
            Normalization::default(),
            vec![
                Case::new("same", JsonValue::Null),
                Case::new("same", JsonValue::Null),
            ],
        );
        let adapter = |_: &JsonValue| Outcome::Success(JsonValue::Null);

        assert_eq!(
            compare_suite(&suite, &adapter, &adapter),
            Err(SuiteError::DuplicateCaseId("same".into()))
        );
    }

    #[test]
    fn json_escaping_and_non_finite_float_validation_are_explicit() {
        assert_eq!(
            JsonValue::String("quote \" slash \\ line\n".into())
                .to_json()
                .unwrap(),
            "\"quote \\\" slash \\\\ line\\n\""
        );
        assert_eq!(
            JsonValue::Float(f64::NAN).to_json(),
            Err(super::JsonError::NonFiniteFloat)
        );
        assert_eq!(JsonValue::Float(1.0).to_json().unwrap(), "1.0");
    }
}
