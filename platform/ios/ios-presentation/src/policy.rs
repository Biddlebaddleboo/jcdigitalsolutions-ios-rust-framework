/// Stable caller-defined identity for one action in a selectable alert.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AlertActionId(u32);

impl AlertActionId {
    /// Create an action identity from a fixed-width caller value.
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Return the caller value associated with this action.
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Public UIKit style for one selectable alert action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AlertActionStyle {
    /// An ordinary action.
    Default,
    /// An action that performs a destructive operation.
    Destructive,
    /// The alert's single cancellation action.
    Cancel,
}

/// Borrowed title, caller identity, and style for one selectable alert action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AlertAction<'a> {
    pub(crate) id: AlertActionId,
    pub(crate) title: &'a str,
    pub(crate) style: AlertActionStyle,
}

impl<'a> AlertAction<'a> {
    /// Create one action without allocating or taking ownership of its title.
    pub const fn new(id: AlertActionId, title: &'a str, style: AlertActionStyle) -> Self {
        Self { id, title, style }
    }

    /// Return this action's caller-defined identity.
    pub const fn id(self) -> AlertActionId {
        self.id
    }

    /// Return this action's UIKit style.
    pub const fn style(self) -> AlertActionStyle {
        self.style
    }

    /// Return this action's borrowed title.
    pub const fn title(self) -> &'a str {
        self.title
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ActionListError {
    Empty,
    DuplicateId,
    MultipleCancelActions,
}

pub(crate) fn validate_action_list(actions: &[AlertAction<'_>]) -> Result<(), ActionListError> {
    if actions.is_empty() {
        return Err(ActionListError::Empty);
    }
    for index in 0..actions.len() {
        if actions[..index]
            .iter()
            .any(|previous| previous.id == actions[index].id)
        {
            return Err(ActionListError::DuplicateId);
        }
    }
    if actions
        .iter()
        .filter(|action| action.style == AlertActionStyle::Cancel)
        .count()
        > 1
    {
        return Err(ActionListError::MultipleCancelActions);
    }
    Ok(())
}

pub(crate) fn valid_popover_source_rect<T: Into<f64>>(x: T, y: T, width: T, height: T) -> bool {
    let x = x.into();
    let y = y.into();
    let width = width.into();
    let height = height.into();
    x.is_finite()
        && y.is_finite()
        && width.is_finite()
        && height.is_finite()
        && width >= 0.0
        && height >= 0.0
}

#[cfg(test)]
mod tests {
    use super::{
        ActionListError, AlertAction, AlertActionId, AlertActionStyle, valid_popover_source_rect,
        validate_action_list,
    };

    #[test]
    fn accepts_distinct_action_ids_with_at_most_one_cancel_action() {
        let actions = [
            AlertAction::new(AlertActionId::new(1), "Save", AlertActionStyle::Default),
            AlertAction::new(
                AlertActionId::new(2),
                "Discard",
                AlertActionStyle::Destructive,
            ),
            AlertAction::new(AlertActionId::new(3), "Cancel", AlertActionStyle::Cancel),
        ];

        assert_eq!(validate_action_list(&actions), Ok(()));
        assert_eq!(actions[1].id().get(), 2);
        assert_eq!(actions[1].title(), "Discard");
        assert_eq!(actions[1].style(), AlertActionStyle::Destructive);
    }

    #[test]
    fn rejects_empty_action_list() {
        assert_eq!(validate_action_list(&[]), Err(ActionListError::Empty));
    }

    #[test]
    fn rejects_duplicate_action_ids() {
        let actions = [
            AlertAction::new(AlertActionId::new(7), "First", AlertActionStyle::Default),
            AlertAction::new(
                AlertActionId::new(7),
                "Second",
                AlertActionStyle::Destructive,
            ),
        ];

        assert_eq!(
            validate_action_list(&actions),
            Err(ActionListError::DuplicateId)
        );
    }

    #[test]
    fn rejects_multiple_cancel_actions() {
        let actions = [
            AlertAction::new(
                AlertActionId::new(1),
                "Cancel one",
                AlertActionStyle::Cancel,
            ),
            AlertAction::new(
                AlertActionId::new(2),
                "Cancel two",
                AlertActionStyle::Cancel,
            ),
        ];

        assert_eq!(
            validate_action_list(&actions),
            Err(ActionListError::MultipleCancelActions)
        );
    }

    #[test]
    fn accepts_zero_and_finite_nonnegative_anchor_rects() {
        assert!(valid_popover_source_rect(0.0, 0.0, 0.0, 0.0));
        assert!(valid_popover_source_rect(-4.0, 8.0, 12.0, 16.0));
    }

    #[test]
    fn rejects_invalid_popover_source_rectangles() {
        assert!(!valid_popover_source_rect(f64::NAN, 0.0, 1.0, 1.0));
        assert!(!valid_popover_source_rect(0.0, f64::INFINITY, 1.0, 1.0));
        assert!(!valid_popover_source_rect(0.0, 0.0, -1.0, 1.0));
        assert!(!valid_popover_source_rect(0.0, 0.0, 1.0, f64::NEG_INFINITY));
    }
}
