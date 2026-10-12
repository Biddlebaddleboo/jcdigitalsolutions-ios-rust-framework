//! Owned UserNotifications action and category configuration values.

use alloc::string::String;
use alloc::vec::Vec;
use core::ops::BitOr;
use framework_notifications::NotificationCategoryId;
use framework_notifications::response::NotificationActionId;

/// Allocation-light choices applied to an iOS notification action.
///
/// The flags select authentication before execution, destructive styling, or foreground
/// activation. No option is enabled by default.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IosNotificationActionOptions(u8);

impl IosNotificationActionOptions {
    /// Create an action with no special behavior.
    pub const NONE: Self = Self(0);
    /// Require authentication before executing the action.
    pub const AUTHENTICATION_REQUIRED: Self = Self(1);
    /// Mark the action as destructive.
    pub const DESTRUCTIVE: Self = Self(2);
    /// Activate the app in the foreground when the action is selected.
    pub const FOREGROUND: Self = Self(4);

    /// Combines action options.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Reports whether every option in `other` is present.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl BitOr for IosNotificationActionOptions {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

/// An owned button or text-input action for a registered notification category.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IosNotificationAction {
    identifier: NotificationActionId,
    title: String,
    options: IosNotificationActionOptions,
    kind: IosNotificationActionKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum IosNotificationActionKind {
    Button,
    TextInput {
        button_title: String,
        placeholder: String,
    },
}

impl IosNotificationAction {
    /// Creates a standard button with an exact identifier, title, and options.
    pub fn button(
        identifier: NotificationActionId,
        title: String,
        options: IosNotificationActionOptions,
    ) -> Self {
        Self {
            identifier,
            title,
            options,
            kind: IosNotificationActionKind::Button,
        }
    }

    /// Creates a text-input action with an exact identifier and owned prompt strings.
    pub fn text_input(
        identifier: NotificationActionId,
        title: String,
        button_title: String,
        placeholder: String,
        options: IosNotificationActionOptions,
    ) -> Self {
        Self {
            identifier,
            title,
            options,
            kind: IosNotificationActionKind::TextInput {
                button_title,
                placeholder,
            },
        }
    }

    /// Borrows the exact action identifier.
    pub fn identifier(&self) -> &NotificationActionId {
        &self.identifier
    }

    /// Borrows the action's visible title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the configured action options.
    pub const fn options(&self) -> IosNotificationActionOptions {
        self.options
    }

    pub(crate) fn text_input_details(&self) -> Option<(&str, &str)> {
        match &self.kind {
            IosNotificationActionKind::Button => None,
            IosNotificationActionKind::TextInput {
                button_title,
                placeholder,
            } => Some((button_title, placeholder)),
        }
    }
}

/// An owned category and ordered set of actions registered with the app's notification center.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IosNotificationCategory {
    identifier: NotificationCategoryId,
    actions: Vec<IosNotificationAction>,
    custom_dismiss_action: bool,
}

impl IosNotificationCategory {
    /// Creates a category with no custom actions or dismissal callback.
    pub const fn new(identifier: NotificationCategoryId) -> Self {
        Self {
            identifier,
            actions: Vec::new(),
            custom_dismiss_action: false,
        }
    }

    /// Adds one action, rejecting a repeated action identifier within this category.
    pub fn add_action(
        &mut self,
        action: IosNotificationAction,
    ) -> Result<(), IosNotificationConfigurationError> {
        if self
            .actions
            .iter()
            .any(|existing| existing.identifier == action.identifier)
        {
            return Err(IosNotificationConfigurationError::DuplicateActionIdentifier);
        }
        self.actions.push(action);
        Ok(())
    }

    /// Borrows the category's exact identifier.
    pub fn identifier(&self) -> &NotificationCategoryId {
        &self.identifier
    }

    /// Borrows the actions in registration order.
    pub fn actions(&self) -> &[IosNotificationAction] {
        &self.actions
    }

    /// Enables or disables delivery of the platform's custom dismissal response.
    pub const fn with_custom_dismiss_action(mut self, enabled: bool) -> Self {
        self.custom_dismiss_action = enabled;
        self
    }

    /// Reports whether a custom dismissal response is enabled.
    pub const fn has_custom_dismiss_action(&self) -> bool {
        self.custom_dismiss_action
    }
}

/// A locally detectable conflict in an action/category configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IosNotificationConfigurationError {
    /// An action identifier is repeated within one category.
    DuplicateActionIdentifier,
    /// A category identifier is repeated in one complete replacement set.
    DuplicateCategoryIdentifier,
}

pub(crate) fn validate_categories(
    categories: &[IosNotificationCategory],
) -> Result<(), IosNotificationConfigurationError> {
    for (index, category) in categories.iter().enumerate() {
        if categories[..index]
            .iter()
            .any(|earlier| earlier.identifier == category.identifier)
        {
            return Err(IosNotificationConfigurationError::DuplicateCategoryIdentifier);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    fn action_id(value: &str) -> NotificationActionId {
        NotificationActionId::new(value.to_string()).unwrap()
    }

    fn category_id(value: &str) -> NotificationCategoryId {
        NotificationCategoryId::new(value.to_string()).unwrap()
    }

    #[test]
    fn action_options_keep_only_selected_flags() {
        let options = IosNotificationActionOptions::AUTHENTICATION_REQUIRED
            | IosNotificationActionOptions::FOREGROUND;
        assert!(options.contains(IosNotificationActionOptions::AUTHENTICATION_REQUIRED));
        assert!(options.contains(IosNotificationActionOptions::FOREGROUND));
        assert!(!options.contains(IosNotificationActionOptions::DESTRUCTIVE));
        assert_eq!(IosNotificationActionOptions::NONE.0, 0);
    }

    #[test]
    fn text_input_action_owns_exact_prompt_and_action_values() {
        let action = IosNotificationAction::text_input(
            action_id("reply.exact"),
            "Reply".to_string(),
            "Send".to_string(),
            "Write a reply".to_string(),
            IosNotificationActionOptions::FOREGROUND,
        );
        assert_eq!(action.identifier().as_str(), "reply.exact");
        assert_eq!(action.title(), "Reply");
        assert_eq!(action.options(), IosNotificationActionOptions::FOREGROUND);
        assert_eq!(action.text_input_details(), Some(("Send", "Write a reply")));
    }

    #[test]
    fn duplicate_actions_are_rejected_without_reordering_existing_actions() {
        let mut category = IosNotificationCategory::new(category_id("messages"));
        category
            .add_action(IosNotificationAction::button(
                action_id("reply"),
                "Reply".to_string(),
                IosNotificationActionOptions::NONE,
            ))
            .unwrap();
        assert_eq!(
            category.add_action(IosNotificationAction::button(
                action_id("reply"),
                "Duplicate".to_string(),
                IosNotificationActionOptions::NONE,
            )),
            Err(IosNotificationConfigurationError::DuplicateActionIdentifier)
        );
        assert_eq!(category.actions().len(), 1);
        assert_eq!(category.actions()[0].title(), "Reply");
    }

    #[test]
    fn duplicate_categories_are_rejected_and_empty_set_is_valid_cleanup() {
        let category = IosNotificationCategory::new(category_id("messages"));
        assert_eq!(validate_categories(&[]), Ok(()));
        assert_eq!(
            validate_categories(core::slice::from_ref(&category)),
            Ok(())
        );
        assert_eq!(
            validate_categories(&[category.clone(), category]),
            Err(IosNotificationConfigurationError::DuplicateCategoryIdentifier)
        );
    }

    #[test]
    fn custom_dismissal_is_opt_in() {
        let category = IosNotificationCategory::new(category_id("messages"));
        assert!(!category.has_custom_dismiss_action());
        assert!(
            category
                .with_custom_dismiss_action(true)
                .has_custom_dismiss_action()
        );
    }
}
