use core::cell::RefCell;
use std::boxed::Box;
use std::string::String;
use std::vec::Vec;

use framework_core::{Error, ErrorKind, Result};

/// One owned row for a reusable UIKit list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IosListRow {
    /// Stable identity returned when the row is selected.
    pub id: u64,
    /// Primary row text.
    pub title: String,
    /// Optional secondary row text.
    pub detail: Option<String>,
}

impl IosListRow {
    /// Creates an owned row with a stable identity and title.
    pub fn new(id: u64, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            detail: None,
        }
    }

    /// Sets the optional secondary row text.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

pub(crate) fn replace_rows(
    current: &mut Vec<IosListRow>,
    replacement: Vec<IosListRow>,
) -> Result<()> {
    validate_rows(&replacement)?;
    *current = replacement;
    Ok(())
}

fn validate_rows(rows: &[IosListRow]) -> Result<()> {
    validate_row_count(rows.len())?;
    let mut ids = Vec::new();
    ids.try_reserve_exact(rows.len())
        .map_err(|_| Error::new(ErrorKind::ResourceExhausted))?;
    ids.extend(rows.iter().map(|row| row.id));
    ids.sort_unstable();
    if ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(Error::new(ErrorKind::InvalidInput));
    }
    Ok(())
}

fn validate_row_count(count: usize) -> Result<()> {
    isize::try_from(count)
        .map(|_| ())
        .map_err(|_| Error::new(ErrorKind::ResourceExhausted))
}

pub(crate) type SelectionCallback = Box<dyn FnMut(u64)>;

pub(crate) fn invoke_selection_callback(callback: &RefCell<SelectionCallback>, row_id: u64) {
    catch_callback(|| {
        if let Ok(mut callback) = callback.try_borrow_mut() {
            callback(row_id);
        }
    });
}

#[cfg(target_os = "ios")]
fn catch_callback(callback: impl FnOnce()) {
    let _ = ios_runtime::ffi::catch_unwind(callback);
}

#[cfg(not(target_os = "ios"))]
fn catch_callback(callback: impl FnOnce()) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(callback));
}

#[cfg(test)]
mod tests {
    use super::{IosListRow, invoke_selection_callback, replace_rows, validate_row_count};
    use core::cell::{Cell, RefCell};
    use framework_core::ErrorKind;
    use std::rc::{Rc, Weak};
    use std::string::String;
    use std::vec;
    use std::vec::Vec;

    fn row(id: u64, title: &str) -> IosListRow {
        IosListRow::new(id, String::from(title))
    }

    #[test]
    fn accepts_unique_fixed_width_identities() {
        let mut rows = Vec::new();
        assert!(
            replace_rows(
                &mut rows,
                vec![
                    row(0, "zero"),
                    row(u64::MAX, "maximum").with_detail("detail")
                ]
            )
            .is_ok()
        );
    }

    #[test]
    fn rejects_duplicate_identities() {
        let mut rows = Vec::new();
        let error = replace_rows(&mut rows, vec![row(7, "first"), row(7, "second")]).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidInput);
        assert!(rows.is_empty());
    }

    #[test]
    fn failed_replacement_preserves_the_previous_rows() {
        let mut rows = vec![row(1, "kept")];
        let error = replace_rows(&mut rows, vec![row(2, "new"), row(2, "duplicate")]).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidInput);
        assert_eq!(rows, vec![row(1, "kept")]);
    }

    #[test]
    fn rejects_row_count_that_cannot_fit_uikit_integer() {
        let error = validate_row_count(usize::MAX).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::ResourceExhausted);
    }

    #[test]
    fn selection_callback_panic_is_contained_and_releases_its_borrow() {
        let callback = RefCell::new(
            Box::new(|_| -> () { panic!("list selection panic") }) as Box<dyn FnMut(u64)>
        );
        invoke_selection_callback(&callback, 11);
        assert!(callback.try_borrow_mut().is_ok());
    }

    #[test]
    fn reentrant_selection_callback_is_ignored() {
        type Callback = Rc<RefCell<Box<dyn FnMut(u64)>>>;
        type CallbackSlot = Rc<RefCell<Option<Weak<RefCell<Box<dyn FnMut(u64)>>>>>>;

        let callback_slot: CallbackSlot = Rc::new(RefCell::new(None));
        let calls = Rc::new(Cell::new(0));
        let nested_slot = callback_slot.clone();
        let nested_calls = calls.clone();
        let callback: Callback = Rc::new(RefCell::new(Box::new(move |row_id| {
            nested_calls.set(nested_calls.get() + 1);
            if let Some(callback) = nested_slot.borrow().as_ref().and_then(Weak::upgrade) {
                invoke_selection_callback(&callback, row_id);
            }
        })));
        *callback_slot.borrow_mut() = Some(Rc::downgrade(&callback));

        invoke_selection_callback(&callback, 22);
        assert_eq!(calls.get(), 1);
    }
}
