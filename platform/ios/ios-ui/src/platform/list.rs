use core::cell::RefCell;
use core::marker::PhantomData;
use std::rc::Rc;
use std::vec::Vec;

use framework_core::{Error, ErrorKind, Result};
use framework_ui::Frame;
use ios_runtime::ffi::catch_unwind;
use objc2::ffi::NSInteger;
use objc2::rc::Retained;
use objc2::runtime::{NSObject, ProtocolObject};
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, Message, define_class, msg_send};
use objc2_foundation::{NSIndexPath, NSObjectProtocol, NSString};
use objc2_ui_kit::{
    NSIndexPathUIKitAdditions, UIScrollViewDelegate, UITableView, UITableViewCell,
    UITableViewCellStyle, UITableViewDataSource, UITableViewDelegate, UITableViewStyle,
};

pub use crate::list_policy::IosListRow;
use crate::list_policy::{invoke_selection_callback, replace_rows};

use super::{IosUiBackend, IosView, main_thread_marker, native_frame};

struct ListTargetState {
    rows: RefCell<Vec<IosListRow>>,
    selection: RefCell<Box<dyn FnMut(u64)>>,
    reuse_identifier: Retained<NSString>,
    fallback_cell: Retained<UITableViewCell>,
}

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - ListTarget does not implement Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosUiListTarget"]
    #[ivars = ListTargetState]
    struct ListTarget;

    // SAFETY: NSObjectProtocol adds no required methods; the target is main-thread-only.
    unsafe impl NSObjectProtocol for ListTarget {}

    // SAFETY: The list does not need optional scroll callbacks; UITableViewDelegate inherits this protocol.
    unsafe impl UIScrollViewDelegate for ListTarget {}

    // SAFETY: UITableView invokes its data source on the main thread. The list handle owns this
    // target while active, the target self-retains during callbacks, and callback panics are caught.
    #[allow(non_snake_case)]
    unsafe impl UITableViewDataSource for ListTarget {
        #[unsafe(method(tableView:numberOfRowsInSection:))]
        fn tableView_numberOfRowsInSection(
            &self,
            _table_view: &UITableView,
            section: NSInteger,
        ) -> NSInteger {
            let _keep_alive = self.retain();
            catch_unwind(|| {
                if section != 0 {
                    return 0;
                }
                self.ivars()
                    .rows
                    .try_borrow()
                    .ok()
                    .and_then(|rows| NSInteger::try_from(rows.len()).ok())
                    .unwrap_or(0)
            })
            .unwrap_or(0)
        }

        #[unsafe(method_id(tableView:cellForRowAtIndexPath:))]
        fn tableView_cellForRowAtIndexPath(
            &self,
            table_view: &UITableView,
            index_path: &NSIndexPath,
        ) -> Retained<UITableViewCell> {
            let _keep_alive = self.retain();
            let fallback = self.ivars().fallback_cell.clone();
            catch_unwind(|| self.cell_for_row(table_view, index_path))
                .ok()
                .flatten()
                .unwrap_or(fallback)
        }
    }

    // SAFETY: UITableView invokes the delegate on the main thread. The target self-retains while
    // resolving a current row identity, releases the row borrow before app code, and contains callback panics.
    #[allow(non_snake_case)]
    unsafe impl UITableViewDelegate for ListTarget {
        #[unsafe(method(tableView:didSelectRowAtIndexPath:))]
        fn tableView_didSelectRowAtIndexPath(
            &self,
            _table_view: &UITableView,
            index_path: &NSIndexPath,
        ) {
            let _keep_alive = self.retain();
            let row_id = catch_unwind(|| {
                if index_path.section() != 0 {
                    return None;
                }
                let row = usize::try_from(index_path.row()).ok()?;
                self.ivars()
                    .rows
                    .try_borrow()
                    .ok()?
                    .get(row)
                    .map(|row| row.id)
            })
            .ok()
            .flatten();
            if let Some(row_id) = row_id {
                invoke_selection_callback(&self.ivars().selection, row_id);
            }
        }
    }
);

impl ListTarget {
    fn with_state(mtm: MainThreadMarker, state: ListTargetState) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(state);
        // SAFETY: `this` is a fresh NSObject subclass allocation and NSObject's `init` is valid.
        unsafe { msg_send![super(this), init] }
    }

    fn cell_for_row(
        &self,
        table_view: &UITableView,
        index_path: &NSIndexPath,
    ) -> Option<Retained<UITableViewCell>> {
        if index_path.section() != 0 {
            return None;
        }
        let row_index = usize::try_from(index_path.row()).ok()?;
        let (title, detail) = {
            let rows = self.ivars().rows.try_borrow().ok()?;
            let row = rows.get(row_index)?;
            (
                NSString::from_str(&row.title),
                row.detail.as_deref().map(NSString::from_str),
            )
        };
        let marker = main_thread_marker().ok()?;
        let state = self.ivars();
        let cell = table_view
            .dequeueReusableCellWithIdentifier(&state.reuse_identifier)
            .unwrap_or_else(|| {
                UITableViewCell::initWithStyle_reuseIdentifier(
                    UITableViewCell::alloc(marker),
                    UITableViewCellStyle::Subtitle,
                    Some(&state.reuse_identifier),
                )
            });
        #[allow(deprecated)]
        if let Some(label) = cell.textLabel() {
            label.setText(Some(&title));
        }
        #[allow(deprecated)]
        if let Some(label) = cell.detailTextLabel() {
            label.setText(detail.as_deref());
        }
        Some(cell)
    }
}

/// A native UIKit table that owns an app-supplied row snapshot and reuses visible cells.
///
/// The caller owns row meaning and update timing. This is not a declarative view tree: it provides
/// no bindings, automatic observation, diffing, or app-state ownership. Keep the handle alive while
/// selection callbacks and row updates should remain active.
pub struct IosListView {
    native: Retained<UITableView>,
    _target: Retained<ListTarget>,
    _not_send: PhantomData<Rc<()>>,
}

impl IosUiBackend {
    /// Adds a UIKit list to `parent` with an owned row snapshot and stable-ID selection callback.
    ///
    /// `frame` uses parent-local UIKit points. Rows must have unique `u64` IDs. UIKit reuses its
    /// native subtitle cells; the callback receives the selected row's current stable ID.
    pub fn add_list_view<F>(
        &mut self,
        parent: &mut IosView,
        frame: Frame,
        rows: Vec<IosListRow>,
        selection: F,
    ) -> Result<IosListView>
    where
        F: FnMut(u64) + 'static,
    {
        let marker = main_thread_marker()?;
        let mut validated_rows = Vec::new();
        replace_rows(&mut validated_rows, rows)?;

        let native = UITableView::initWithFrame_style(
            UITableView::alloc(marker),
            native_frame(frame),
            UITableViewStyle::Plain,
        );
        let reuse_identifier = NSString::from_str("JcdIosUiListCell");
        let fallback_cell = UITableViewCell::new(marker);
        let target = ListTarget::with_state(
            marker,
            ListTargetState {
                rows: RefCell::new(validated_rows),
                selection: RefCell::new(Box::new(selection)),
                reuse_identifier,
                fallback_cell,
            },
        );
        let data_source = ProtocolObject::from_ref(&*target);
        native.setDataSource(Some(data_source));
        let delegate = ProtocolObject::from_ref(&*target);
        // SAFETY: `target` is retained by `IosListView` for the full active interval; UITableView
        // stores its delegate weakly and is confined to this same main thread.
        unsafe { native.setDelegate(Some(delegate)) };
        parent.native.addSubview(&native);
        Ok(IosListView {
            native,
            _target: target,
            _not_send: PhantomData,
        })
    }
}

impl IosListView {
    /// Atomically replaces the complete row snapshot and asks UIKit to reload its visible cells.
    ///
    /// Duplicate identities and row counts outside UIKit's signed index range are rejected before
    /// mutation. A rejected update leaves the old rows and displayed contents unchanged.
    pub fn replace_rows(&mut self, rows: Vec<IosListRow>) -> Result<()> {
        let _main_thread = main_thread_marker()?;
        let mut current = self
            ._target
            .ivars()
            .rows
            .try_borrow_mut()
            .map_err(|_| Error::new(ErrorKind::Unavailable))?;
        replace_rows(&mut current, rows)?;
        drop(current);
        self.native.reloadData();
        Ok(())
    }

    /// Borrows the native table for UIKit configuration not provided by this wrapper.
    pub fn native_table_view(&self) -> &UITableView {
        &self.native
    }
}
