use framework_core::{Error, ErrorKind};
use framework_preferences::{
    AtomicityRequirement, PreferenceError, UpdateAtomicity, WriteOptions, WriteOutcome,
};

pub(crate) fn accepted_write_outcome(
    options: WriteOptions,
) -> Result<WriteOutcome, PreferenceError> {
    if options.atomicity() == AtomicityRequirement::RequireAtomic {
        Err(PreferenceError::Backend(Error::new(ErrorKind::Unsupported)))
    } else {
        Ok(WriteOutcome::new(UpdateAtomicity::NotGuaranteed))
    }
}
