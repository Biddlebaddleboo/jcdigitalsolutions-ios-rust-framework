use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind};

pub(crate) struct OneShotCompletion {
    action: Option<Box<dyn FnOnce()>>,
}

impl fmt::Debug for OneShotCompletion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OneShotCompletion")
            .field("pending", &self.action.is_some())
            .finish()
    }
}

impl OneShotCompletion {
    pub(crate) fn new(action: impl FnOnce() + 'static) -> Self {
        Self {
            action: Some(Box::new(action)),
        }
    }

    pub(crate) fn complete(&mut self) {
        if let Some(action) = self.action.take() {
            let _ = catch_unwind(AssertUnwindSafe(action));
        }
    }
}

impl Drop for OneShotCompletion {
    fn drop(&mut self) {
        self.complete();
    }
}
