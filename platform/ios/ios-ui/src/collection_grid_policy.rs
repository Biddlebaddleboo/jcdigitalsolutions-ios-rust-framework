use core::cell::{Cell, RefCell};
use std::boxed::Box;
use std::string::String;
use std::vec::Vec;

use framework_core::{Error, ErrorKind, Result};

/// Section padding in UIKit points.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct IosCollectionGridInsets {
    /// Top padding.
    pub top: f64,
    /// Leading padding.
    pub leading: f64,
    /// Bottom padding.
    pub bottom: f64,
    /// Trailing padding.
    pub trailing: f64,
}

impl IosCollectionGridInsets {
    /// Zero padding.
    pub const ZERO: Self = Self {
        top: 0.0,
        leading: 0.0,
        bottom: 0.0,
        trailing: 0.0,
    };
}

/// Native compositional layout for one collection section.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IosCollectionGridLayout {
    /// A fixed number of equally sized columns.
    FixedGrid {
        /// Number of columns; must be nonzero.
        columns: u16,
        /// Fixed item height in points; must be finite and positive.
        item_height: f64,
        /// Spacing between rows; must be finite and nonnegative.
        row_spacing: f64,
        /// Spacing between columns; must be finite and nonnegative.
        column_spacing: f64,
        /// Section padding.
        insets: IosCollectionGridInsets,
    },
    /// A grid whose column count adapts to available width.
    AdaptiveGrid {
        /// Minimum item width in points; must be finite and positive.
        minimum_item_width: f64,
        /// Fixed item height in points; must be finite and positive.
        item_height: f64,
        /// Maximum column count; must be nonzero.
        maximum_columns: u16,
        /// Spacing between rows; must be finite and nonnegative.
        row_spacing: f64,
        /// Spacing between columns; must be finite and nonnegative.
        column_spacing: f64,
        /// Section padding.
        insets: IosCollectionGridInsets,
    },
    /// A horizontally scrolling section with fixed item dimensions.
    HorizontalCarousel {
        /// Fixed item width in points; must be finite and positive.
        item_width: f64,
        /// Fixed item height in points; must be finite and positive.
        item_height: f64,
        /// Spacing between items; must be finite and nonnegative.
        item_spacing: f64,
        /// Section padding.
        insets: IosCollectionGridInsets,
    },
}

/// Reusable semantic content rendered by a native collection cell.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IosCollectionGridItemContent {
    /// A general text card.
    Card {
        /// Primary card text.
        title: String,
        /// Optional secondary card text.
        detail: Option<String>,
    },
    /// A label/value metric tile.
    Metric {
        /// Metric label.
        label: String,
        /// Metric value.
        value: String,
        /// Optional supporting metric text.
        detail: Option<String>,
    },
    /// A selectable action tile; disabled actions do not emit events.
    Action {
        /// Action title.
        title: String,
        /// Optional supporting action text.
        detail: Option<String>,
        /// Whether activation is enabled.
        enabled: bool,
    },
    /// A stateful tile rendered with a native switch.
    Toggle {
        /// Toggle title.
        title: String,
        /// Optional supporting toggle text.
        detail: Option<String>,
        /// Snapshot-owned switch state.
        is_on: bool,
        /// Whether the switch accepts user input.
        enabled: bool,
    },
}

/// A typed interaction emitted for a selected collection item.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IosCollectionGridEvent {
    /// A card or metric item was selected.
    Selected {
        /// Stable item identity.
        item_id: u64,
    },
    /// An enabled action item was activated.
    ActionTriggered {
        /// Stable item identity.
        item_id: u64,
    },
    /// An enabled native switch requests this value.
    ToggleChanged {
        /// Stable item identity.
        item_id: u64,
        /// Requested state; the current snapshot remains authoritative until replacement.
        is_on: bool,
    },
}

/// One typed item rendered by a reusable native collection cell.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IosCollectionGridItem {
    /// Stable identity used for diffing and events.
    pub id: u64,
    /// Semantic content rendered by the cell.
    pub content: IosCollectionGridItemContent,
}

impl IosCollectionGridItem {
    /// Creates an owned item with stable identity and a title.
    pub fn new(id: u64, title: impl Into<String>) -> Self {
        Self {
            id,
            content: IosCollectionGridItemContent::Card {
                title: title.into(),
                detail: None,
            },
        }
    }

    /// Creates a label/value metric tile.
    pub fn metric(id: u64, label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            id,
            content: IosCollectionGridItemContent::Metric {
                label: label.into(),
                value: value.into(),
                detail: None,
            },
        }
    }

    /// Creates an enabled action tile.
    pub fn action(id: u64, title: impl Into<String>) -> Self {
        Self {
            id,
            content: IosCollectionGridItemContent::Action {
                title: title.into(),
                detail: None,
                enabled: true,
            },
        }
    }

    /// Creates an enabled stateful toggle tile.
    pub fn toggle(id: u64, title: impl Into<String>, is_on: bool) -> Self {
        Self {
            id,
            content: IosCollectionGridItemContent::Toggle {
                title: title.into(),
                detail: None,
                is_on,
                enabled: true,
            },
        }
    }

    /// Sets the optional secondary item text.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        let detail = Some(detail.into());
        match &mut self.content {
            IosCollectionGridItemContent::Card {
                detail: current, ..
            }
            | IosCollectionGridItemContent::Metric {
                detail: current, ..
            }
            | IosCollectionGridItemContent::Action {
                detail: current, ..
            }
            | IosCollectionGridItemContent::Toggle {
                detail: current, ..
            } => *current = detail,
        }
        self
    }

    /// Sets whether an action or toggle accepts user input.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        match &mut self.content {
            IosCollectionGridItemContent::Action {
                enabled: current, ..
            }
            | IosCollectionGridItemContent::Toggle {
                enabled: current, ..
            } => *current = enabled,
            IosCollectionGridItemContent::Card { .. }
            | IosCollectionGridItemContent::Metric { .. } => {}
        }
        self
    }
}

/// One stable section and its ordered collection items.
#[derive(Clone, Debug, PartialEq)]
pub struct IosCollectionGridSection {
    /// Stable section identity.
    pub id: u64,
    /// Layout applied to this section.
    pub layout: IosCollectionGridLayout,
    /// Ordered items in the section.
    pub items: Vec<IosCollectionGridItem>,
}

impl IosCollectionGridSection {
    /// Creates an owned section.
    pub fn new(
        id: u64,
        layout: IosCollectionGridLayout,
        items: Vec<IosCollectionGridItem>,
    ) -> Self {
        Self { id, layout, items }
    }
}

/// Validates a complete snapshot before any native state is changed.
pub(crate) fn validate_snapshot(sections: &[IosCollectionGridSection]) -> Result<()> {
    validate_index_count(sections.len())?;
    let total_items = sections.iter().try_fold(0usize, |total, section| {
        validate_layout(section.layout)?;
        validate_index_count(section.items.len())?;
        total
            .checked_add(section.items.len())
            .ok_or_else(|| Error::new(ErrorKind::ResourceExhausted))
    })?;
    validate_index_count(total_items)?;
    let mut section_ids = Vec::new();
    section_ids
        .try_reserve_exact(sections.len())
        .map_err(|_| Error::new(ErrorKind::ResourceExhausted))?;
    let mut item_ids = Vec::new();
    item_ids
        .try_reserve_exact(total_items)
        .map_err(|_| Error::new(ErrorKind::ResourceExhausted))?;
    for section in sections {
        section_ids.push(section.id);
        item_ids.extend(section.items.iter().map(|item| item.id));
    }
    section_ids.sort_unstable();
    item_ids.sort_unstable();
    if section_ids.windows(2).any(|pair| pair[0] == pair[1])
        || item_ids.windows(2).any(|pair| pair[0] == pair[1])
    {
        return Err(Error::new(ErrorKind::InvalidInput));
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn replace_snapshot(
    current: &mut Vec<IosCollectionGridSection>,
    replacement: Vec<IosCollectionGridSection>,
) -> Result<()> {
    validate_snapshot(&replacement)?;
    *current = replacement;
    Ok(())
}

pub(crate) fn ensure_snapshot_apply_idle(in_flight: &Cell<bool>) -> Result<()> {
    if in_flight.get() {
        Err(Error::new(ErrorKind::Unavailable))
    } else {
        Ok(())
    }
}

/// Commit a previously validated snapshot and reserve its UIKit apply until completion.
pub(crate) fn replace_for_snapshot_apply<T>(
    current: &mut T,
    replacement: T,
    in_flight: &Cell<bool>,
) -> Result<()> {
    ensure_snapshot_apply_idle(in_flight)?;
    in_flight.set(true);
    *current = replacement;
    Ok(())
}

pub(crate) fn complete_snapshot_apply(in_flight: &Cell<bool>) {
    in_flight.set(false);
}

fn validate_index_count(count: usize) -> Result<()> {
    isize::try_from(count)
        .map(|_| ())
        .map_err(|_| Error::new(ErrorKind::ResourceExhausted))
}

fn valid_dimension(value: f64) -> bool {
    value.is_finite() && value > 0.0
}
fn valid_spacing(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}
fn valid_insets(insets: IosCollectionGridInsets) -> bool {
    [insets.top, insets.leading, insets.bottom, insets.trailing]
        .into_iter()
        .all(valid_spacing)
}

fn validate_layout(layout: IosCollectionGridLayout) -> Result<()> {
    let valid = match layout {
        IosCollectionGridLayout::FixedGrid {
            columns,
            item_height,
            row_spacing,
            column_spacing,
            insets,
        } => {
            columns > 0
                && valid_dimension(item_height)
                && valid_spacing(row_spacing)
                && valid_spacing(column_spacing)
                && valid_insets(insets)
        }
        IosCollectionGridLayout::AdaptiveGrid {
            minimum_item_width,
            item_height,
            maximum_columns,
            row_spacing,
            column_spacing,
            insets,
        } => {
            valid_dimension(minimum_item_width)
                && valid_dimension(item_height)
                && maximum_columns > 0
                && valid_spacing(row_spacing)
                && valid_spacing(column_spacing)
                && valid_insets(insets)
        }
        IosCollectionGridLayout::HorizontalCarousel {
            item_width,
            item_height,
            item_spacing,
            insets,
        } => {
            valid_dimension(item_width)
                && valid_dimension(item_height)
                && valid_spacing(item_spacing)
                && valid_insets(insets)
        }
    };
    if valid {
        Ok(())
    } else {
        Err(Error::new(ErrorKind::InvalidInput))
    }
}

/// Stable IDs whose typed content changed while identity remained present.
pub(crate) fn changed_item_ids(
    old: &[IosCollectionGridSection],
    new: &[IosCollectionGridSection],
) -> Result<Vec<u64>> {
    let old_count: usize = old.iter().map(|section| section.items.len()).sum();
    let new_count: usize = new.iter().map(|section| section.items.len()).sum();
    let mut old_items = Vec::new();
    old_items
        .try_reserve_exact(old_count)
        .map_err(|_| Error::new(ErrorKind::ResourceExhausted))?;
    old_items.extend(
        old.iter()
            .flat_map(|section| &section.items)
            .map(|item| (item.id, item)),
    );
    old_items.sort_unstable_by_key(|(id, _)| *id);
    let mut changed = Vec::new();
    changed
        .try_reserve(new_count.min(old_count))
        .map_err(|_| Error::new(ErrorKind::ResourceExhausted))?;
    for next_section in new {
        for next in &next_section.items {
            let previous = old_items
                .binary_search_by_key(&next.id, |(id, _)| *id)
                .ok()
                .map(|index| old_items[index].1);
            if previous.is_some_and(|previous| previous.content != next.content) {
                changed.push(next.id);
            }
        }
    }
    Ok(changed)
}

pub(crate) type EventCallback = Box<dyn FnMut(IosCollectionGridEvent)>;

pub(crate) fn selection_event(item: &IosCollectionGridItem) -> Option<IosCollectionGridEvent> {
    match &item.content {
        IosCollectionGridItemContent::Card { .. } | IosCollectionGridItemContent::Metric { .. } => {
            Some(IosCollectionGridEvent::Selected { item_id: item.id })
        }
        IosCollectionGridItemContent::Action { .. }
        | IosCollectionGridItemContent::Toggle { .. } => None,
    }
}

pub(crate) fn action_event(item: &IosCollectionGridItem) -> Option<IosCollectionGridEvent> {
    match item.content {
        IosCollectionGridItemContent::Action { enabled: true, .. } => {
            Some(IosCollectionGridEvent::ActionTriggered { item_id: item.id })
        }
        _ => None,
    }
}

pub(crate) fn toggle_event(
    item: &IosCollectionGridItem,
    is_on: bool,
) -> Option<IosCollectionGridEvent> {
    match item.content {
        IosCollectionGridItemContent::Toggle { enabled: true, .. } => {
            Some(IosCollectionGridEvent::ToggleChanged {
                item_id: item.id,
                is_on,
            })
        }
        _ => None,
    }
}

pub(crate) fn invoke_event_callback(
    callback: &RefCell<EventCallback>,
    event: IosCollectionGridEvent,
) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if let Ok(mut callback) = callback.try_borrow_mut() {
            callback(event);
        }
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::{Cell, RefCell};
    use std::rc::{Rc, Weak};
    use std::vec;

    fn fixed() -> IosCollectionGridLayout {
        IosCollectionGridLayout::FixedGrid {
            columns: 3,
            item_height: 100.0,
            row_spacing: 8.0,
            column_spacing: 8.0,
            insets: IosCollectionGridInsets::ZERO,
        }
    }

    fn section(id: u64, items: Vec<IosCollectionGridItem>) -> IosCollectionGridSection {
        IosCollectionGridSection::new(id, fixed(), items)
    }

    #[test]
    fn accepts_all_layouts_and_full_width_ids() {
        let sections = vec![
            section(0, vec![IosCollectionGridItem::new(0, "zero")]),
            IosCollectionGridSection::new(
                1,
                IosCollectionGridLayout::AdaptiveGrid {
                    minimum_item_width: 120.0,
                    item_height: 80.0,
                    maximum_columns: 5,
                    row_spacing: 4.0,
                    column_spacing: 6.0,
                    insets: IosCollectionGridInsets::ZERO,
                },
                vec![IosCollectionGridItem::new(i64::MAX as u64 + 1, "wide")],
            ),
            IosCollectionGridSection::new(
                u64::MAX,
                IosCollectionGridLayout::HorizontalCarousel {
                    item_width: 180.0,
                    item_height: 120.0,
                    item_spacing: 12.0,
                    insets: IosCollectionGridInsets::ZERO,
                },
                vec![IosCollectionGridItem::new(u64::MAX, "max")],
            ),
        ];
        assert!(validate_snapshot(&sections).is_ok());
    }

    #[test]
    fn rejects_invalid_layout_values() {
        let invalid = [
            IosCollectionGridLayout::FixedGrid {
                columns: 0,
                item_height: 100.0,
                row_spacing: 8.0,
                column_spacing: 8.0,
                insets: IosCollectionGridInsets::ZERO,
            },
            IosCollectionGridLayout::FixedGrid {
                columns: 1,
                item_height: 0.0,
                row_spacing: 0.0,
                column_spacing: 0.0,
                insets: IosCollectionGridInsets::ZERO,
            },
            IosCollectionGridLayout::AdaptiveGrid {
                minimum_item_width: f64::NAN,
                item_height: 1.0,
                maximum_columns: 1,
                row_spacing: 0.0,
                column_spacing: 0.0,
                insets: IosCollectionGridInsets::ZERO,
            },
            IosCollectionGridLayout::AdaptiveGrid {
                minimum_item_width: 1.0,
                item_height: 1.0,
                maximum_columns: 0,
                row_spacing: 0.0,
                column_spacing: 0.0,
                insets: IosCollectionGridInsets::ZERO,
            },
            IosCollectionGridLayout::HorizontalCarousel {
                item_width: 1.0,
                item_height: 1.0,
                item_spacing: -1.0,
                insets: IosCollectionGridInsets::ZERO,
            },
            IosCollectionGridLayout::FixedGrid {
                columns: 1,
                item_height: 1.0,
                row_spacing: 0.0,
                column_spacing: 0.0,
                insets: IosCollectionGridInsets {
                    top: f64::INFINITY,
                    ..IosCollectionGridInsets::ZERO
                },
            },
            IosCollectionGridLayout::AdaptiveGrid {
                minimum_item_width: -1.0,
                item_height: 1.0,
                maximum_columns: 1,
                row_spacing: f64::NAN,
                column_spacing: 0.0,
                insets: IosCollectionGridInsets::ZERO,
            },
            IosCollectionGridLayout::HorizontalCarousel {
                item_width: 1.0,
                item_height: -1.0,
                item_spacing: f64::INFINITY,
                insets: IosCollectionGridInsets {
                    trailing: -1.0,
                    ..IosCollectionGridInsets::ZERO
                },
            },
        ];
        for layout in invalid {
            assert_eq!(
                validate_snapshot(&[IosCollectionGridSection::new(1, layout, vec![])])
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidInput
            );
        }
    }

    #[test]
    fn rejects_duplicate_section_and_global_item_ids() {
        assert_eq!(
            validate_snapshot(&[section(1, vec![]), section(1, vec![])])
                .unwrap_err()
                .kind(),
            ErrorKind::InvalidInput
        );
        assert_eq!(
            validate_snapshot(&[
                section(1, vec![IosCollectionGridItem::new(7, "a")]),
                section(2, vec![IosCollectionGridItem::new(7, "b")])
            ])
            .unwrap_err()
            .kind(),
            ErrorKind::InvalidInput
        );
    }

    #[test]
    fn invalid_replacement_does_not_mutate_current_snapshot() {
        let mut current = vec![section(1, vec![IosCollectionGridItem::new(11, "kept")])];
        let replacement = vec![section(
            2,
            vec![
                IosCollectionGridItem::new(12, "new"),
                IosCollectionGridItem::new(12, "duplicate"),
            ],
        )];
        assert!(replace_snapshot(&mut current, replacement).is_err());
        assert_eq!(
            current,
            vec![section(1, vec![IosCollectionGridItem::new(11, "kept")])]
        );
    }

    #[test]
    fn second_snapshot_apply_is_rejected_without_replacing_current_state() {
        let mut current = vec![section(1, vec![IosCollectionGridItem::new(11, "current")])];
        let in_flight = Cell::new(false);
        replace_for_snapshot_apply(
            &mut current,
            vec![section(2, vec![IosCollectionGridItem::new(22, "accepted")])],
            &in_flight,
        )
        .unwrap();
        let accepted = current.clone();
        let second = replace_for_snapshot_apply(
            &mut current,
            vec![section(3, vec![IosCollectionGridItem::new(33, "rejected")])],
            &in_flight,
        );
        assert_eq!(second.unwrap_err().kind(), ErrorKind::Unavailable);
        assert_eq!(current, accepted);
        complete_snapshot_apply(&in_flight);
        assert!(
            replace_for_snapshot_apply(
                &mut current,
                vec![section(
                    3,
                    vec![IosCollectionGridItem::new(33, "after completion")]
                )],
                &in_flight
            )
            .is_ok()
        );
    }

    #[test]
    fn identifies_content_changes_only_for_stable_existing_ids() {
        let old = vec![section(
            1,
            vec![IosCollectionGridItem::new(u64::MAX, "before").with_detail("old detail")],
        )];
        let new = vec![section(
            2,
            vec![
                IosCollectionGridItem::new(u64::MAX, "before").with_detail("new detail"),
                IosCollectionGridItem::new(4, "new"),
            ],
        )];
        assert_eq!(changed_item_ids(&old, &new).unwrap(), vec![u64::MAX]);
    }

    #[test]
    fn typed_events_preserve_action_and_toggle_semantics() {
        assert_eq!(
            selection_event(&IosCollectionGridItem::metric(u64::MAX, "Trips", "12")),
            Some(IosCollectionGridEvent::Selected { item_id: u64::MAX })
        );
        assert_eq!(
            selection_event(&IosCollectionGridItem::action(5, "Run")),
            None
        );
        assert_eq!(
            selection_event(&IosCollectionGridItem::toggle(7, "Enabled", false)),
            None
        );
        assert_eq!(
            action_event(&IosCollectionGridItem::action(9, "Run")),
            Some(IosCollectionGridEvent::ActionTriggered { item_id: 9 })
        );
        assert_eq!(
            action_event(&IosCollectionGridItem::action(10, "Disabled").with_enabled(false)),
            None
        );
        assert_eq!(
            toggle_event(&IosCollectionGridItem::toggle(11, "On", false), true),
            Some(IosCollectionGridEvent::ToggleChanged {
                item_id: 11,
                is_on: true
            })
        );
        assert_eq!(
            toggle_event(&IosCollectionGridItem::toggle(12, "Off", true), false),
            Some(IosCollectionGridEvent::ToggleChanged {
                item_id: 12,
                is_on: false
            })
        );
        assert_eq!(
            toggle_event(
                &IosCollectionGridItem::toggle(13, "Disabled", false).with_enabled(false),
                true
            ),
            None
        );
    }

    #[test]
    fn event_callback_panic_is_contained_and_reentrant_call_is_ignored() {
        let panicking = RefCell::new(Box::new(|_| -> () { panic!("grid event") }) as EventCallback);
        invoke_event_callback(&panicking, IosCollectionGridEvent::Selected { item_id: 1 });
        assert!(panicking.try_borrow_mut().is_ok());
        type Callback = Rc<RefCell<EventCallback>>;
        let slot: Rc<RefCell<Option<Weak<RefCell<EventCallback>>>>> = Rc::new(RefCell::new(None));
        let calls = Rc::new(Cell::new(0));
        let nested_slot = slot.clone();
        let nested_calls = calls.clone();
        let callback: Callback = Rc::new(RefCell::new(Box::new(move |event| {
            nested_calls.set(nested_calls.get() + 1);
            if let Some(callback) = nested_slot.borrow().as_ref().and_then(Weak::upgrade) {
                invoke_event_callback(&callback, event);
            }
        })));
        *slot.borrow_mut() = Some(Rc::downgrade(&callback));
        invoke_event_callback(&callback, IosCollectionGridEvent::Selected { item_id: 2 });
        assert_eq!(calls.get(), 1);
    }
}
