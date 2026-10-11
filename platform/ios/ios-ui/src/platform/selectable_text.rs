use core::marker::PhantomData;
use std::rc::Rc;

use framework_core::Result;
use framework_ui::Frame;
use objc2::rc::{Retained, autoreleasepool};
use objc2_foundation::NSString;
use objc2_ui_kit::UITextView;

use super::{IosUiBackend, IosView, main_thread_marker, native_frame, native_string_to_rust};

/// A retained, read-only, selectable multiline UIKit text view.
///
/// The handle is main-thread-bound. UIKit may retain the visual view after this handle drops.
pub struct IosSelectableTextView {
    native: Retained<UITextView>,
    _not_send: PhantomData<Rc<()>>,
}

impl IosUiBackend {
    /// Adds a read-only, selectable multiline text view to `parent`.
    ///
    /// The frame and text are caller-owned inputs. The returned handle owns the `UITextView`;
    /// UIKit may retain the visual view through its parent after the handle drops. Text selection
    /// uses UIKit's native selection behavior; app-specific copy actions and content policy remain
    /// with the caller.
    pub fn add_selectable_text_view(
        &mut self,
        parent: &mut IosView,
        frame: Frame,
        text: &str,
    ) -> Result<IosSelectableTextView> {
        let marker = main_thread_marker()?;
        let native = UITextView::new(marker);
        native.setFrame(native_frame(frame));
        native.setEditable(false);
        native.setSelectable(true);
        native.setScrollEnabled(true);
        autoreleasepool(|_| {
            let text = NSString::from_str(text);
            native.setText(Some(&text));
        });
        parent.native.addSubview(&native);
        Ok(IosSelectableTextView {
            native,
            _not_send: PhantomData,
        })
    }
}

impl IosSelectableTextView {
    /// Returns the current native text as a strict UTF-16 snapshot.
    ///
    /// Embedded NUL and supplementary Unicode scalars are preserved; malformed UTF-16 returns
    /// `InvalidInput` rather than replacement characters.
    pub fn text(&self) -> Result<String> {
        let _text_view_read_thread = main_thread_marker()?;
        let text = self.native.text();
        native_string_to_rust(&text)
    }

    /// Replaces the native text without enabling editing.
    ///
    /// An empty string clears the text. Rust `str` input is encoded with its explicit length,
    /// preserving embedded NUL characters.
    pub fn set_text(&mut self, text: &str) -> Result<()> {
        let _text_view_write_thread = main_thread_marker()?;
        autoreleasepool(|_| {
            let text = NSString::from_str(text);
            self.native.setText(Some(&text));
        });
        Ok(())
    }

    /// Borrows the native text view for caller-owned UIKit integration.
    pub fn native_text_view(&self) -> &UITextView {
        &self.native
    }
}
