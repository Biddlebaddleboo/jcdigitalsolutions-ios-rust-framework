# Custom-render text input

`ios-ui::IosCustomTextInput` attaches a native editable `UITextView` to a caller-owned UIKit parent, including `IosCustomRenderHost::native_view()`. It reuses UIKit's `UITextInput` implementation for insertion, deletion, replacement, selection, and input-method composition; it is not a framework-owned editor model or a keypress adapter.

## Buffer and event contract

UIKit owns the live native text buffer and marked text while the input is active. The synchronous main-thread callback receives a complete `IosCustomTextInputSnapshot` after UIKit text, selection, focus, and keyboard delegate/notification events. UIKit event order is preserved; duplicate snapshots are allowed because UIKit can report text and selection as separate events. Text edits are snapshots, not inferred key events.

The snapshot's `text` includes the complete provisional candidate during composition. A non-`None` `marked_text` range identifies that candidate as composing. A consumer that mirrors committed Rust text should update its committed model only after `marked_text` becomes `None`; reading `snapshot()` during composition does not commit or discard the candidate. The bridge retains the latest unmarked text and selection so `cancel_composition()` can restore that state. Calling `set_text()` instead performs a silent Rust-to-UIKit replacement, clears any marked range, and places the selection at the new UTF-16 end.

`selection` and `marked_text` use UTF-16 code-unit offsets, matching UIKit's documented [`UITextInput.offset(from:to:)`](https://developer.apple.com/documentation/uikit/uitextinput/offset%28from%3Ato%3A%29) unit. `Utf16Range::from_utf8_byte_range` and `to_utf8_byte_range` translate valid Rust string boundaries. They reject out-of-bounds ranges and boundaries inside a Unicode scalar, including a supplementary scalar's surrogate pair. Selection updates are silent; user-driven selection changes arrive as complete snapshots.

## Focus, keyboard, and teardown

`request_focus()` and `resign_focus()` return UIKit's synchronous first-responder result. First-responder acceptance is not proof that a software keyboard is visible. Public UIKit keyboard notifications update `keyboard_visible` and an optional `keyboard_frame` in screen-space points; hardware keyboards, system policy, or missing notification geometry may leave the keyboard hidden or frame absent. Callers must treat this state as a snapshot, not a guaranteed animation timeline.

The handle is main-thread-affine and `!Send`/`!Sync`. It owns its `UITextView`, exact weak delegate, and keyboard notification registrations. Drop disables callback delivery before resigning first responder, removes only its own delegate and notifications, and removes only its owned text view. It does not replace a parent controller, scene delegate, or another view's delegate. Rust callback panics are contained; a reentrant callback is skipped while the same callback is active.

The minimal app has an opt-in consumer that mirrors only unmarked snapshots into a Rust-owned `String` and requests focus at launch. Build and install it using [the existing iOS minimal example instructions](../../examples/ios-minimal/README.md), then launch with `--exercise-custom-text-input`. The build/link probes establish UIKit symbols and deployment metadata only; they do not establish keyboard appearance, IME composition, or runtime text correctness.

This capability uses public UIKit `UITextView`, `UITextInput`, `UITextViewDelegate`, and keyboard notification APIs. It adds no Swift source or dependency.
