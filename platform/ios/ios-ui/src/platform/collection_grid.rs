use core::cell::{Cell, RefCell};
use core::marker::PhantomData;
use core::ptr::NonNull;
use std::collections::HashMap;
use std::rc::Rc;
use std::vec::Vec;

use block2::RcBlock;
use framework_core::{Error, ErrorKind, Result};
use framework_ui::Frame;
use ios_runtime::ffi::catch_unwind;
use objc2::ffi::NSInteger;
use objc2::rc::{Allocated, Retained, Weak};
use objc2::runtime::{AnyObject, NSObject, ProtocolObject};
use objc2::{
    ClassType, DefinedClass, MainThreadMarker, MainThreadOnly, Message, define_class, msg_send, sel,
};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_foundation::{NSArray, NSIndexPath, NSNumber, NSObjectProtocol, NSString};
use objc2_ui_kit::{
    NSCollectionLayoutContainer, NSCollectionLayoutDimension, NSCollectionLayoutEnvironment,
    NSCollectionLayoutGroup, NSCollectionLayoutItem, NSCollectionLayoutSection,
    NSCollectionLayoutSize, NSCollectionLayoutSpacing, NSDirectionalEdgeInsets, UIButton,
    UIButtonType, UICollectionLayoutSectionOrthogonalScrollingBehavior, UICollectionView,
    UICollectionViewCell, UICollectionViewCompositionalLayout, UICollectionViewDelegate,
    UICollectionViewDiffableDataSource, UICollectionViewDiffableDataSourceCellProvider,
    UIControlEvents, UIControlState, UILabel, UIScrollViewDelegate, UISwitch, UIViewAutoresizing,
};

use crate::collection_grid_policy::{
    IosCollectionGridEvent, IosCollectionGridInsets, IosCollectionGridItem,
    IosCollectionGridItemContent, IosCollectionGridLayout, IosCollectionGridSection, action_event,
    changed_item_ids, complete_snapshot_apply, ensure_snapshot_apply_idle, invoke_event_callback,
    replace_for_snapshot_apply, selection_event, toggle_event, validate_snapshot,
};

use super::{IosUiBackend, IosView, main_thread_marker, native_frame};

const CELL_REUSE: &str = "JcdIosUiCollectionGridCell";

type GridSectionProvider = dyn Fn(
    NSInteger,
    NonNull<ProtocolObject<dyn NSCollectionLayoutEnvironment>>,
) -> *mut NSCollectionLayoutSection;
type GridCellProvider = dyn Fn(
    NonNull<UICollectionView>,
    NonNull<NSIndexPath>,
    NonNull<AnyObject>,
) -> *mut UICollectionViewCell;

struct GridData {
    sections: Vec<IosCollectionGridSection>,
    item_indices: HashMap<u64, (usize, usize)>,
}

impl GridData {
    fn item(&self, id: u64) -> Option<&IosCollectionGridItem> {
        let (section, item) = *self.item_indices.get(&id)?;
        self.sections.get(section)?.items.get(item)
    }
}

fn make_grid_data(sections: Vec<IosCollectionGridSection>) -> Result<GridData> {
    validate_snapshot(&sections)?;
    let count: usize = sections.iter().map(|section| section.items.len()).sum();
    let mut item_indices = HashMap::new();
    item_indices
        .try_reserve(count)
        .map_err(|_| Error::new(ErrorKind::ResourceExhausted))?;
    for (section_index, section) in sections.iter().enumerate() {
        for (item_index, item) in section.items.iter().enumerate() {
            item_indices.insert(item.id, (section_index, item_index));
        }
    }
    Ok(GridData {
        sections,
        item_indices,
    })
}

#[derive(Default)]
struct GridCellState {
    item_id: Cell<u64>,
    committed_toggle: Cell<bool>,
    target: RefCell<Option<Weak<GridTarget>>>,
    title: RefCell<Option<Retained<UILabel>>>,
    detail: RefCell<Option<Retained<UILabel>>>,
    tertiary: RefCell<Option<Retained<UILabel>>>,
    action: RefCell<Option<Retained<UIButton>>>,
    toggle: RefCell<Option<Retained<UISwitch>>>,
}

define_class!(
    #[unsafe(super(UICollectionViewCell))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosUiCollectionGridCell"]
    #[ivars = GridCellState]
    struct GridCell;

    impl GridCell {
        #[unsafe(method_id(initWithFrame:))]
        fn init_with_frame(this: Allocated<Self>, frame: CGRect) -> Retained<Self> {
            let this = this.set_ivars(GridCellState::default());
            // SAFETY: UIKit constructs collection cells on the main thread, and this is the
            // UICollectionViewCell designated frame initializer.
            let this: Retained<Self> = unsafe { msg_send![super(this), initWithFrame: frame] };
            Self::install_controls(this)
        }

        #[unsafe(method(gridActionTapped:))]
        fn action_tapped(&self, _sender: &UIButton) {
            let _keep_alive = self.retain();
            let item_id = self.ivars().item_id.get();
            let target = { self.ivars().target.borrow().as_ref().and_then(Weak::load) };
            if let Some(target) = target { target.emit_action(item_id); }
        }

        #[unsafe(method(gridToggleChanged:))]
        fn toggle_changed(&self, sender: &UISwitch) {
            let _keep_alive = self.retain();
            let state = self.ivars();
            let requested = sender.isOn();
            let committed = state.committed_toggle.get();
            sender.setOn_animated(committed, false);
            let target = { state.target.borrow().as_ref().and_then(Weak::load) };
            if let Some(target) = target { target.emit_toggle(state.item_id.get(), requested); }
        }
    }

    unsafe impl NSObjectProtocol for GridCell {}
);

impl GridCell {
    fn install_controls(this: Retained<Self>) -> Retained<Self> {
        // SAFETY: initWithFrame: is called by UIKit while constructing a collection cell.
        let marker = unsafe { MainThreadMarker::new_unchecked() };
        let content = this.contentView();
        let title = UILabel::new(marker);
        let detail = UILabel::new(marker);
        let tertiary = UILabel::new(marker);
        for label in [&title, &detail, &tertiary] {
            label.setNumberOfLines(1);
        }
        let action = UIButton::buttonWithType(UIButtonType::System, marker);
        // SAFETY: the cell implements the registered selector and is retained by UIKit while the
        // control is in its content view.
        unsafe {
            action.addTarget_action_forControlEvents(
                Some(&*this),
                sel!(gridActionTapped:),
                UIControlEvents::TouchUpInside,
            );
        }
        let toggle = UISwitch::new(marker);
        // SAFETY: the cell implements the registered selector and is retained by UIKit while the
        // control is in its content view.
        unsafe {
            toggle.addTarget_action_forControlEvents(
                Some(&*this),
                sel!(gridToggleChanged:),
                UIControlEvents::ValueChanged,
            );
        }
        content.addSubview(&title);
        content.addSubview(&detail);
        content.addSubview(&tertiary);
        content.addSubview(&action);
        content.addSubview(&toggle);
        let state = this.ivars();
        *state.title.borrow_mut() = Some(title);
        *state.detail.borrow_mut() = Some(detail);
        *state.tertiary.borrow_mut() = Some(tertiary);
        *state.action.borrow_mut() = Some(action);
        *state.toggle.borrow_mut() = Some(toggle);
        this
    }

    fn render(&self, item: &IosCollectionGridItem, target: Weak<GridTarget>) {
        let state = self.ivars();
        state.item_id.set(item.id);
        *state.target.borrow_mut() = Some(target);
        let content = self.contentView();
        let bounds = content.bounds();
        let width = bounds.size.width;
        let height = bounds.size.height;
        let title = state.title.borrow();
        let detail = state.detail.borrow();
        let tertiary = state.tertiary.borrow();
        let action = state.action.borrow();
        let toggle = state.toggle.borrow();
        let (Some(title), Some(detail), Some(tertiary), Some(action), Some(toggle)) = (
            title.as_ref(),
            detail.as_ref(),
            tertiary.as_ref(),
            action.as_ref(),
            toggle.as_ref(),
        ) else {
            return;
        };
        title.setHidden(true);
        detail.setHidden(true);
        tertiary.setHidden(true);
        action.setHidden(true);
        toggle.setHidden(true);
        match &item.content {
            IosCollectionGridItemContent::Card {
                title: text,
                detail: secondary,
            } => {
                set_label(title, text);
                set_optional_label(detail, secondary.as_deref());
                title.setFrame(CGRect::new(
                    CGPoint::new(8.0, 10.0),
                    CGSize::new((width - 16.0).max(0.0), 24.0),
                ));
                detail.setFrame(CGRect::new(
                    CGPoint::new(8.0, 38.0),
                    CGSize::new((width - 16.0).max(0.0), 20.0),
                ));
                title.setHidden(false);
            }
            IosCollectionGridItemContent::Metric {
                label,
                value,
                detail: secondary,
            } => {
                set_label(title, label);
                set_label(detail, value);
                set_optional_label(tertiary, secondary.as_deref());
                title.setFrame(CGRect::new(
                    CGPoint::new(8.0, 8.0),
                    CGSize::new((width - 16.0).max(0.0), 20.0),
                ));
                detail.setFrame(CGRect::new(
                    CGPoint::new(8.0, 30.0),
                    CGSize::new((width - 16.0).max(0.0), 34.0),
                ));
                tertiary.setFrame(CGRect::new(
                    CGPoint::new(8.0, 68.0),
                    CGSize::new((width - 16.0).max(0.0), 18.0),
                ));
                title.setHidden(false);
                detail.setHidden(false);
            }
            IosCollectionGridItemContent::Action {
                title: text,
                detail: secondary,
                enabled,
            } => {
                let native_text = NSString::from_str(text);
                action.setTitle_forState(Some(&native_text), UIControlState::Normal);
                action.setEnabled(*enabled);
                action.setFrame(CGRect::new(
                    CGPoint::new(4.0, 8.0),
                    CGSize::new((width - 8.0).max(0.0), (height * 0.58).max(32.0)),
                ));
                set_optional_label(detail, secondary.as_deref());
                detail.setFrame(CGRect::new(
                    CGPoint::new(8.0, height * 0.66),
                    CGSize::new((width - 16.0).max(0.0), 20.0),
                ));
                action.setHidden(false);
            }
            IosCollectionGridItemContent::Toggle {
                title: text,
                detail: secondary,
                is_on,
                enabled,
            } => {
                set_label(title, text);
                set_optional_label(detail, secondary.as_deref());
                title.setFrame(CGRect::new(
                    CGPoint::new(8.0, 12.0),
                    CGSize::new((width - 82.0).max(0.0), 24.0),
                ));
                detail.setFrame(CGRect::new(
                    CGPoint::new(8.0, 40.0),
                    CGSize::new((width - 82.0).max(0.0), 20.0),
                ));
                toggle.setFrame(CGRect::new(
                    CGPoint::new((width - 68.0).max(0.0), ((height - 32.0) / 2.0).max(0.0)),
                    CGSize::new(60.0, 32.0),
                ));
                toggle.setOn_animated(*is_on, false);
                toggle.setEnabled(*enabled);
                state.committed_toggle.set(*is_on);
                title.setHidden(false);
                toggle.setHidden(false);
            }
        }
        title.setAutoresizingMask(UIViewAutoresizing::FlexibleWidth);
        detail.setAutoresizingMask(UIViewAutoresizing::FlexibleWidth);
        tertiary.setAutoresizingMask(UIViewAutoresizing::FlexibleWidth);
        action.setAutoresizingMask(
            UIViewAutoresizing::FlexibleWidth | UIViewAutoresizing::FlexibleHeight,
        );
        toggle.setAutoresizingMask(
            UIViewAutoresizing::FlexibleLeftMargin
                | UIViewAutoresizing::FlexibleTopMargin
                | UIViewAutoresizing::FlexibleBottomMargin,
        );
    }

    fn clear_binding(&self) {
        let state = self.ivars();
        state.item_id.set(0);
        *state.target.borrow_mut() = None;
        if let Some(view) = state.title.borrow().as_ref() {
            view.setHidden(true);
        }
        if let Some(view) = state.detail.borrow().as_ref() {
            view.setHidden(true);
        }
        if let Some(view) = state.tertiary.borrow().as_ref() {
            view.setHidden(true);
        }
        if let Some(view) = state.action.borrow().as_ref() {
            view.setHidden(true);
            view.setEnabled(false);
        }
        if let Some(view) = state.toggle.borrow().as_ref() {
            view.setHidden(true);
            view.setEnabled(false);
            view.setOn_animated(false, false);
        }
    }
}

fn set_label(label: &UILabel, text: &str) {
    let text = NSString::from_str(text);
    label.setText(Some(&text));
    label.setHidden(false);
}

fn set_optional_label(label: &UILabel, text: Option<&str>) {
    if let Some(text) = text {
        set_label(label, text);
    } else {
        label.setText(None);
        label.setHidden(true);
    }
}

struct GridTargetState {
    data_source: Retained<UICollectionViewDiffableDataSource<NSNumber, NSNumber>>,
    data: Rc<RefCell<GridData>>,
    event_callback: RefCell<Box<dyn FnMut(IosCollectionGridEvent)>>,
    snapshot_apply_in_flight: Cell<bool>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosUiCollectionGridTarget"]
    #[ivars = GridTargetState]
    struct GridTarget;

    unsafe impl NSObjectProtocol for GridTarget {}
    unsafe impl UIScrollViewDelegate for GridTarget {}

    #[allow(non_snake_case)]
    unsafe impl UICollectionViewDelegate for GridTarget {
        #[unsafe(method(collectionView:shouldSelectItemAtIndexPath:))]
        fn collectionView_shouldSelectItemAtIndexPath(
            &self,
            _collection_view: &UICollectionView,
            index_path: &NSIndexPath,
        ) -> bool {
            catch_unwind(|| {
                let identifier = self
                    .ivars()
                    .data_source
                    .itemIdentifierForIndexPath(index_path)?;
                let data = self.ivars().data.try_borrow().ok()?;
                match &data.item(identifier.as_u64())?.content {
                    IosCollectionGridItemContent::Card { .. }
                    | IosCollectionGridItemContent::Metric { .. } => Some(true),
                    IosCollectionGridItemContent::Action { .. }
                    | IosCollectionGridItemContent::Toggle { .. } => Some(false),
                }
            })
            .ok()
            .flatten()
            .unwrap_or(false)
        }

        #[unsafe(method(collectionView:didSelectItemAtIndexPath:))]
        fn collectionView_didSelectItemAtIndexPath(
            &self,
            _collection_view: &UICollectionView,
            index_path: &NSIndexPath,
        ) {
            let _keep_alive = self.retain();
            let item_id = catch_unwind(|| {
                self.ivars()
                    .data_source
                    .itemIdentifierForIndexPath(index_path)
                    .map(|identifier| identifier.as_u64())
            })
            .ok()
            .flatten();
            if let Some(item_id) = item_id {
                self.emit(item_id);
            }
        }
    }
);

impl GridTarget {
    fn with_state(mtm: MainThreadMarker, state: GridTargetState) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(state);
        // SAFETY: this is a fresh NSObject subclass allocation.
        unsafe { msg_send![super(this), init] }
    }

    fn emit(&self, item_id: u64) {
        let event = catch_unwind(|| {
            let data = self.ivars().data.try_borrow().ok()?;
            selection_event(data.item(item_id)?)
        })
        .ok()
        .flatten();
        if let Some(event) = event {
            invoke_event_callback(&self.ivars().event_callback, event);
        }
    }

    fn emit_action(&self, item_id: u64) {
        let event = catch_unwind(|| {
            let data = self.ivars().data.try_borrow().ok()?;
            action_event(data.item(item_id)?)
        })
        .ok()
        .flatten();
        if let Some(event) = event {
            invoke_event_callback(&self.ivars().event_callback, event);
        }
    }

    fn emit_toggle(&self, item_id: u64, requested: bool) {
        let event = catch_unwind(|| {
            let data = self.ivars().data.try_borrow().ok()?;
            toggle_event(data.item(item_id)?, requested)
        })
        .ok()
        .flatten();
        if let Some(event) = event {
            invoke_event_callback(&self.ivars().event_callback, event);
        }
    }
}

fn native_insets(insets: IosCollectionGridInsets) -> NSDirectionalEdgeInsets {
    NSDirectionalEdgeInsets {
        top: insets.top,
        leading: insets.leading,
        bottom: insets.bottom,
        trailing: insets.trailing,
    }
}

fn build_section(
    layout: IosCollectionGridLayout,
    available_width: f64,
    marker: MainThreadMarker,
) -> Retained<NSCollectionLayoutSection> {
    let (group, insets, row_spacing, item_spacing, carousel) = match layout {
        IosCollectionGridLayout::FixedGrid {
            columns,
            item_height,
            row_spacing,
            column_spacing,
            insets,
        } => (
            grid_group(
                columns,
                item_height,
                column_spacing,
                available_width,
                insets,
                marker,
            ),
            insets,
            row_spacing,
            column_spacing,
            false,
        ),
        IosCollectionGridLayout::AdaptiveGrid {
            minimum_item_width,
            item_height,
            maximum_columns,
            row_spacing,
            column_spacing,
            insets,
        } => {
            let width =
                (available_width - insets.leading - insets.trailing).max(minimum_item_width);
            let columns = (((width + column_spacing) / (minimum_item_width + column_spacing))
                .floor() as u16)
                .clamp(1, maximum_columns);
            (
                grid_group(
                    columns,
                    item_height,
                    column_spacing,
                    available_width,
                    insets,
                    marker,
                ),
                insets,
                row_spacing,
                column_spacing,
                false,
            )
        }
        IosCollectionGridLayout::HorizontalCarousel {
            item_width,
            item_height,
            item_spacing,
            insets,
        } => {
            let width = NSCollectionLayoutDimension::absoluteDimension(item_width, marker);
            let height = NSCollectionLayoutDimension::absoluteDimension(item_height, marker);
            let size =
                NSCollectionLayoutSize::sizeWithWidthDimension_heightDimension(&width, &height);
            let item = NSCollectionLayoutItem::itemWithLayoutSize(&size);
            let group =
                NSCollectionLayoutGroup::horizontalGroupWithLayoutSize_repeatingSubitem_count(
                    &size, &item, 1,
                );
            (group, insets, 0.0, item_spacing, true)
        }
    };
    let section = NSCollectionLayoutSection::sectionWithGroup(&group);
    section.setContentInsets(native_insets(insets));
    section.setInterGroupSpacing(if carousel { item_spacing } else { row_spacing });
    if carousel {
        section.setOrthogonalScrollingBehavior(
            UICollectionLayoutSectionOrthogonalScrollingBehavior::Continuous,
        );
    }
    section
}

fn grid_group(
    columns: u16,
    item_height: f64,
    spacing: f64,
    width: f64,
    insets: IosCollectionGridInsets,
    marker: MainThreadMarker,
) -> Retained<NSCollectionLayoutGroup> {
    let count = f64::from(columns);
    let content_width =
        (width - insets.leading - insets.trailing - spacing * f64::from(columns.saturating_sub(1)))
            .max(count);
    let item_width = NSCollectionLayoutDimension::absoluteDimension(content_width / count, marker);
    let item_height_dimension = NSCollectionLayoutDimension::absoluteDimension(item_height, marker);
    let item_size = NSCollectionLayoutSize::sizeWithWidthDimension_heightDimension(
        &item_width,
        &item_height_dimension,
    );
    let item = NSCollectionLayoutItem::itemWithLayoutSize(&item_size);
    let group_width = NSCollectionLayoutDimension::fractionalWidthDimension(1.0, marker);
    let group_height = NSCollectionLayoutDimension::absoluteDimension(item_height, marker);
    let group_size =
        NSCollectionLayoutSize::sizeWithWidthDimension_heightDimension(&group_width, &group_height);
    let group = NSCollectionLayoutGroup::horizontalGroupWithLayoutSize_repeatingSubitem_count(
        &group_size,
        &item,
        columns as NSInteger,
    );
    if spacing > 0.0 {
        group.setInterItemSpacing(Some(&NSCollectionLayoutSpacing::fixedSpacing(
            spacing, marker,
        )));
    }
    group
}

fn make_snapshot(
    sections: &[IosCollectionGridSection],
    changed: &[u64],
) -> Retained<objc2_ui_kit::NSDiffableDataSourceSnapshot<NSNumber, NSNumber>> {
    let snapshot = objc2_ui_kit::NSDiffableDataSourceSnapshot::new();
    let section_ids: Vec<_> = sections
        .iter()
        .map(|section| NSNumber::new_u64(section.id))
        .collect();
    let section_array = NSArray::from_retained_slice(&section_ids);
    snapshot.appendSectionsWithIdentifiers(&section_array);
    for (section, section_id) in sections.iter().zip(section_ids.iter()) {
        let item_ids: Vec<_> = section
            .items
            .iter()
            .map(|item| NSNumber::new_u64(item.id))
            .collect();
        let item_array = NSArray::from_retained_slice(&item_ids);
        snapshot.appendItemsWithIdentifiers_intoSectionWithIdentifier(&item_array, section_id);
    }
    if !changed.is_empty() {
        let changed_ids: Vec<_> = changed.iter().copied().map(NSNumber::new_u64).collect();
        let changed_array = NSArray::from_retained_slice(&changed_ids);
        snapshot.reloadItemsWithIdentifiers(&changed_array);
    }
    snapshot
}

/// A reusable, sectioned UIKit collection with explicit atomic snapshots.
///
/// UIKit owns layout, scrolling, diffing, and cell reuse. Application state and snapshot timing
/// remain caller-owned; this is not a SwiftUI tree or automatic state-observation system.
/// Keep this handle alive while its data source and typed event callback should remain active.
pub struct IosCollectionGrid {
    native: Retained<UICollectionView>,
    target: Retained<GridTarget>,
    data: Rc<RefCell<GridData>>,
    _not_send: PhantomData<Rc<()>>,
}

impl IosUiBackend {
    /// Adds a native fixed, adaptive, or horizontal-carousel section collection.
    ///
    /// Sections and items require unique stable `u64` IDs. UIKit reuses native cells, and the
    /// callback receives typed selection, action, or requested-toggle events. iOS 12 returns
    /// `Unavailable` before collection creation because the native collection APIs require iOS 13.
    pub fn add_collection_grid<F>(
        &mut self,
        parent: &mut IosView,
        frame: Frame,
        sections: Vec<IosCollectionGridSection>,
        on_event: F,
    ) -> Result<IosCollectionGrid>
    where
        F: FnMut(IosCollectionGridEvent) + 'static,
    {
        let marker = main_thread_marker()?;
        if !objc2::available!(ios = 13.0, ..) {
            return Err(Error::new(ErrorKind::Unavailable));
        }
        let data = Rc::new(RefCell::new(make_grid_data(sections)?));
        let layout_data = data.clone();
        let provider: RcBlock<GridSectionProvider> = RcBlock::new(move |section_index: NSInteger, environment: NonNull<ProtocolObject<dyn NSCollectionLayoutEnvironment>>| {
            let section = catch_unwind(|| {
                let index = usize::try_from(section_index).ok()?;
                let layout = layout_data.try_borrow().ok()?.sections.get(index)?.layout;
                let container = unsafe { environment.as_ref() }.container();
                Some(build_section(layout, container.effectiveContentSize().width, marker))
            }).ok().flatten().unwrap_or_else(|| build_section(IosCollectionGridLayout::FixedGrid { columns: 1, item_height: 1.0, row_spacing: 0.0, column_spacing: 0.0, insets: IosCollectionGridInsets::ZERO }, 1.0, marker));
            Retained::autorelease_return(section)
        });
        // SAFETY: the valid retained block is copied by UIKit during initialization.
        let layout = unsafe {
            UICollectionViewCompositionalLayout::initWithSectionProvider(
                UICollectionViewCompositionalLayout::alloc(marker),
                RcBlock::as_ptr(&provider),
            )
        };
        let native = UICollectionView::initWithFrame_collectionViewLayout(
            UICollectionView::alloc(marker),
            native_frame(frame),
            &layout,
        );
        let reuse = NSString::from_str(CELL_REUSE);
        // SAFETY: GridCell is a registered UICollectionViewCell subclass with a frame initializer.
        unsafe {
            native.registerClass_forCellWithReuseIdentifier(Some(GridCell::class()), &reuse);
        }
        native.setAllowsSelection(true);
        let provider_data = data.clone();
        let provider_target: Rc<RefCell<Option<Weak<GridTarget>>>> = Rc::new(RefCell::new(None));
        let cell_target = provider_target.clone();
        let cell_provider: RcBlock<GridCellProvider> = RcBlock::new(
            move |collection: NonNull<UICollectionView>,
                  index_path: NonNull<NSIndexPath>,
                  item_identifier: NonNull<AnyObject>| {
                let collection = unsafe { collection.as_ref() };
                let index_path = unsafe { index_path.as_ref() };
                let cell = collection
                    .dequeueReusableCellWithReuseIdentifier_forIndexPath(&reuse, index_path);
                let cell = match cell.downcast::<GridCell>() {
                    Ok(cell) => cell,
                    Err(cell) => return Retained::autorelease_return(cell),
                };
                let rendered = catch_unwind(|| {
                    // SAFETY: UIKit supplies this nonnull identifier for the duration of the
                    // synchronous provider callback; downcast verifies its Objective-C class.
                    let item_id = unsafe { item_identifier.as_ref() }
                        .downcast_ref::<NSNumber>()?
                        .as_u64();
                    let item_data = provider_data.try_borrow().ok()?;
                    let item = item_data.item(item_id)?;
                    let target = cell_target.try_borrow().ok()?.as_ref()?.clone();
                    cell.render(item, target);
                    Some(())
                })
                .ok()
                .flatten()
                .is_some();
                if !rendered {
                    cell.clear_binding();
                }
                let cell = cell.into_super();
                Retained::autorelease_return(cell)
            },
        );
        // SAFETY: the valid retained block is copied by UIKit; its captures own the snapshot and
        // only weakly reference the callback target.
        let data_source = unsafe {
            UICollectionViewDiffableDataSource::initWithCollectionView_cellProvider(
                UICollectionViewDiffableDataSource::alloc(marker),
                &native,
                RcBlock::as_ptr(&cell_provider) as UICollectionViewDiffableDataSourceCellProvider,
            )
        };
        let target = GridTarget::with_state(
            marker,
            GridTargetState {
                data_source,
                data: data.clone(),
                event_callback: RefCell::new(Box::new(on_event)),
                snapshot_apply_in_flight: Cell::new(false),
            },
        );
        *provider_target.borrow_mut() = Some(Weak::from_retained(&target));
        let initial_snapshot = make_snapshot(&data.borrow().sections, &[]);
        target
            .ivars()
            .data_source
            .applySnapshot_animatingDifferences(&initial_snapshot, false);
        let delegate = ProtocolObject::from_ref(&*target);
        // SAFETY: target is retained by the handle and UIKit's delegate property is weak.
        unsafe {
            native.setDelegate(Some(delegate));
        }
        parent.native.addSubview(&native);
        Ok(IosCollectionGrid {
            native,
            target,
            data,
            _not_send: PhantomData,
        })
    }
}

impl IosCollectionGrid {
    /// Validates then atomically replaces the complete section and item snapshot.
    ///
    /// Invalid IDs or layouts preserve the current snapshot. Existing IDs with changed typed
    /// content are reloaded. A second call during an animated diff returns `Unavailable` without
    /// mutation; retry after UIKit completes the current apply. The same serialization applies to
    /// animated and nonanimated UIKit updates.
    pub fn replace_snapshot(
        &mut self,
        sections: Vec<IosCollectionGridSection>,
        animating_differences: bool,
    ) -> Result<()> {
        let _main_thread = main_thread_marker()?;
        let in_flight = &self.target.ivars().snapshot_apply_in_flight;
        ensure_snapshot_apply_idle(in_flight)?;
        let replacement = make_grid_data(sections)?;
        let current = self
            .data
            .try_borrow()
            .map_err(|_| Error::new(ErrorKind::Unavailable))?;
        let changed = changed_item_ids(&current.sections, &replacement.sections)?;
        let snapshot = make_snapshot(&replacement.sections, &changed);
        drop(current);
        let weak_target = Weak::from_retained(&self.target);
        let completion = RcBlock::new(move || {
            if let Some(target) = weak_target.load() {
                complete_snapshot_apply(&target.ivars().snapshot_apply_in_flight);
            }
        });
        let mut current = self
            .data
            .try_borrow_mut()
            .map_err(|_| Error::new(ErrorKind::Unavailable))?;
        replace_for_snapshot_apply(&mut *current, replacement, in_flight)?;
        drop(current);
        self.target
            .ivars()
            .data_source
            .applySnapshot_animatingDifferences_completion(
                &snapshot,
                animating_differences,
                Some(&completion),
            );
        self.native.collectionViewLayout().invalidateLayout();
        Ok(())
    }

    /// Borrows the native collection view for UIKit configuration.
    ///
    /// Do not replace its data source, delegate, layout, or registered reuse identifier.
    pub fn native_collection_view(&self) -> &UICollectionView {
        &self.native
    }
}

impl Drop for IosCollectionGrid {
    fn drop(&mut self) {
        if main_thread_marker().is_ok() {
            // SAFETY: remove the weak delegate before releasing its retained target.
            unsafe {
                self.native.setDelegate(None);
            }
            self.native.setDataSource(None);
        }
        let _ = &self.target;
    }
}
