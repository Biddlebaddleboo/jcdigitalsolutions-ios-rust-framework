#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;
#[cfg(target_os = "ios")]
use ios_ui::{
    IosAlertActionStyle, IosAlertKeyboardType, IosAlertSession, IosAlertStyle,
    IosAlertTextFieldConfig,
};
#[cfg(target_os = "ios")]
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
#[cfg(target_os = "ios")]
use objc2_ui_kit::{UIView, UIViewController};

#[cfg(target_os = "ios")]
#[inline(never)]
fn alert_link_probe(presenter: &UIViewController, anchor: &UIView) {
    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let Ok(mut alert) = IosAlertSession::new(
        main_thread,
        presenter,
        IosAlertStyle::Alert,
        Some("Title"),
        Some("Message"),
    ) else {
        return;
    };
    if alert
        .add_action("Continue", IosAlertActionStyle::Default)
        .is_err()
        || alert
            .add_action("Cancel", IosAlertActionStyle::Cancel)
            .is_err()
    {
        return;
    }
    if alert
        .add_text_field(IosAlertTextFieldConfig {
            placeholder: Some("Code".to_owned()),
            initial_text: None,
            secure_text_entry: true,
            keyboard_type: IosAlertKeyboardType::NumberPad,
            maximum_utf16_units: 8,
        })
        .is_err()
    {
        return;
    }
    let _ = alert.present(|result| {
        core::hint::black_box(result);
    });
    let _ = alert.dismiss(false);
    core::hint::black_box(alert.native_alert_controller());

    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let Ok(mut sheet) = IosAlertSession::new(
        main_thread,
        presenter,
        IosAlertStyle::ActionSheet,
        Some("Actions"),
        None,
    ) else {
        return;
    };
    if sheet
        .add_action("Open", IosAlertActionStyle::Default)
        .is_err()
        || sheet
            .set_popover_anchor(
                anchor,
                CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(1.0, 1.0)),
            )
            .is_err()
    {
        return;
    }
    let _ = sheet.present(|result| {
        core::hint::black_box(result);
    });
    let _ = sheet.dismiss(false);
}

#[cfg(target_os = "ios")]
fn main() {
    core::hint::black_box(alert_link_probe as fn(&UIViewController, &UIView));
}

#[cfg(not(target_os = "ios"))]
fn main() {}
