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

pub use crate::list_policy::{IosListRow, IosListSection};
use crate::list_policy::{invoke_selection_callback, replace_sections};

use super::{IosUiBackend, IosView, main_thread_marker, native_frame};

struct ListTargetState {
    sections: RefCell<Vec<IosListSection>>,
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
                usize::try_from(section)
                    .ok()
                    .and_then(|section| {
                        self.ivars()
                            .sections
                            .try_borrow()
                            .ok()?
                            .get(section)
                            .and_then(|section| NSInteger::try_from(section.rows.len()).ok())
                    })
                    .unwrap_or(0)
            })
            .unwrap_or(0)
        }

        #[unsafe(method(numberOfSectionsInTableView:))]
        fn numberOfSectionsInTableView(&self, _table_view: &UITableView) -> NSInteger {
            let _keep_alive = self.retain();
            catch_unwind(|| {
                self.ivars()
                    .sections
                    .try_borrow()
                    .ok()
                    .and_then(|sections| NSInteger::try_from(sections.len()).ok())
                    .unwrap_or(0)
            })
            .unwrap_or(0)
        }

        #[unsafe(method_id(tableView:titleForHeaderInSection:))]
        fn tableView_titleForHeaderInSection(
            &self,
            _table_view: &UITableView,
            section: NSInteger,
        ) -> Option<Retained<NSString>> {
            let _keep_alive = self.retain();
            catch_unwind(|| {
                let section = usize::try_from(section).ok()?;
                self.ivars()
                    .sections
                    .try_borrow()
                    .ok()?
                    .get(section)?
                    .title
                    .as_deref()
                    .map(NSString::from_str)
            })
            .ok()
            .flatten()
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
                let section = usize::try_from(index_path.section()).ok()?;
                let row = usize::try_from(index_path.row()).ok()?;
                self.ivars()
                    .sections
                    .try_borrow()
                    .ok()?
                    .get(section)?
                    .rows
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
        let section_index = usize::try_from(index_path.section()).ok()?;
        let row_index = usize::try_from(index_path.row()).ok()?;
        let (title, detail) = {
            let sections = self.ivars().sections.try_borrow().ok()?;
            let row = sections.get(section_index)?.rows.get(row_index)?;
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
    inner: NativeList,
}

/// A native `UITableView` with independently titled sections and reused row cells.
///
/// The caller owns section meaning and snapshot timing. This remains an imperative snapshot API:
/// it has no bindings, automatic observation, or diffing.
pub struct IosSectionedListView {
    inner: NativeList,
}

struct NativeList {
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
        self.add_list_with_sections(
            parent,
            frame,
            vec![IosListSection::untitled(rows)],
            selection,
        )
        .map(|inner| IosListView { inner })
    }

    /// Adds a UIKit table with titled sections and one globally unique row-ID namespace.
    ///
    /// UIKit reuses native subtitle cells, and the callback receives the selected row's stable ID.
    /// Empty header titles, duplicate row IDs, and section or row counts UIKit cannot represent are
    /// rejected before the native table is attached.
    pub fn add_sectioned_list_view<F>(
        &mut self,
        parent: &mut IosView,
        frame: Frame,
        sections: Vec<IosListSection>,
        selection: F,
    ) -> Result<IosSectionedListView>
    where
        F: FnMut(u64) + 'static,
    {
        self.add_list_with_sections(parent, frame, sections, selection)
            .map(|inner| IosSectionedListView { inner })
    }

    fn add_list_with_sections<F>(
        &mut self,
        parent: &mut IosView,
        frame: Frame,
        sections: Vec<IosListSection>,
        selection: F,
    ) -> Result<NativeList>
    where
        F: FnMut(u64) + 'static,
    {
        let marker = main_thread_marker()?;
        let mut validated_sections = Vec::new();
        replace_sections(&mut validated_sections, sections)?;

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
                sections: RefCell::new(validated_sections),
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
        Ok(NativeList {
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
        self.inner
            .replace_sections(vec![IosListSection::untitled(rows)])
    }

    /// Borrows the native table for UIKit configuration not provided by this wrapper.
    pub fn native_table_view(&self) -> &UITableView {
        self.inner.native_table_view()
    }
}

impl IosSectionedListView {
    /// Atomically replaces every section and reloads UIKit's visible cells.
    ///
    /// IDs must remain unique across the complete snapshot. Invalid titles, duplicate IDs, or
    /// unrepresentable UIKit counts fail before mutation, preserving the displayed snapshot.
    pub fn replace_sections(&mut self, sections: Vec<IosListSection>) -> Result<()> {
        self.inner.replace_sections(sections)
    }

    /// Borrows the native table for UIKit configuration not provided by this wrapper.
    pub fn native_table_view(&self) -> &UITableView {
        self.inner.native_table_view()
    }
}

impl NativeList {
    fn replace_sections(&mut self, sections: Vec<IosListSection>) -> Result<()> {
        let _main_thread = main_thread_marker()?;
        let mut current = self
            ._target
            .ivars()
            .sections
            .try_borrow_mut()
            .map_err(|_| Error::new(ErrorKind::Unavailable))?;
        replace_sections(&mut current, sections)?;
        drop(current);
        self.native.reloadData();
        Ok(())
    }

    fn native_table_view(&self) -> &UITableView {
        &self.native
    }
}
