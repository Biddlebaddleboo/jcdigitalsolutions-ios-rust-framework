#[path = "../src/write_policy.rs"]
mod write_policy;

use framework_core::ErrorKind;
use framework_preferences::{AtomicityRequirement, UpdateAtomicity, WriteOptions};

#[test]
fn required_atomicity_policy_returns_unsupported() {
    let error = write_policy::accepted_write_outcome(WriteOptions::new(
        AtomicityRequirement::RequireAtomic,
    ))
    .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Unsupported);
    assert_eq!(error.platform_code(), None);
}

#[test]
fn accepted_updates_report_not_guaranteed_atomicity() {
    let outcome = write_policy::accepted_write_outcome(WriteOptions::allow_non_atomic()).unwrap();
    assert_eq!(outcome.atomicity(), UpdateAtomicity::NotGuaranteed);
}
