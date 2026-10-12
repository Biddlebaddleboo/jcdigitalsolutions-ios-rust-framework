# iOS PhotosUI image picker

`ios-photo-picker` presents the public, out-of-process `PHPickerViewController` for one image. It does not open the Photos database, enumerate assets, request PhotoKit authorization, or require `NSPhotoLibraryUsageDescription` for picker use. The existing `ios-photos` crate remains a separate PhotoKit authorization/read/save surface and continues to reject PhotosUI imports.

## Use

Call `IosPhotoPickerSession::present` on the UIKit main thread with a visible presenter, a nonzero maximum byte count, and a `Send + 'static` one-shot callback:

```rust,ignore
let session = IosPhotoPickerSession::present(
    main_thread,
    presenter,
    32 * 1024 * 1024,
    move |event| match event {
        IosPhotoPickerEvent::Selected(image) => {
            consume(image.path(), image.byte_len());
        }
        IosPhotoPickerEvent::Cancelled => {}
        IosPhotoPickerEvent::Failed(error) => report(error),
    },
)?;
```

The picker applies `PHPickerFilter.imagesFilter()`, limits selection to one result, and prefers `PHPickerConfigurationAssetRepresentationMode::Current`. The selected `NSItemProvider` supplies its `public.image` file representation. Rust performs no image decode or transcode; it synchronously copies the exact provider-file bytes into a uniquely created child of Foundation's `NSTemporaryDirectory()`. The result exposes its copied byte length and is rejected if it is empty, not a regular file, or exceeds the caller's `u64` ceiling. The copy uses a fixed 32 KiB buffer, checked byte counts, a `0700` directory, and a `0600` file.

The copy is byte-identical to the provider's temporary representation, **not necessarily to the library's original file**: PhotosUI may provide a compatible representation. The provider's temporary file is deleted when its callback returns, so the copy occurs inside that callback before its return. The result file remains valid while `IosPickedImage` is alive and is removed on drop. `IosPickedImage::into_path` transfers file and parent-directory cleanup to the caller.

The callback can run on the main thread for selection cancellation or on a provider-controlled background queue for transfer results. UI operations remain main-thread-only. `cancel()` cancels the provider's `NSProgress` and settles `Cancelled` once. Dropping a pending session cancels and detaches the callback; it does not publish late output. Callback panics are contained at Objective-C boundaries.

## Public SDK binding evidence

The selected Xcode iPhoneOS SDK declares `PHPickerViewController`, `PHPickerViewControllerDelegate`, `initWithConfiguration:`, and `picker:didFinishPicking:` in `System/Library/Frameworks/PhotosUI.framework/Headers/PHPicker.h`. Its iOS availability region declares the controller as a `UIViewController` under `TARGET_OS_IPHONE`; the same header documents the picker as out-of-process and says Photo Library authorization is not needed. `platform.rs` binds only that public UIKit controller/delegate surface with crate-private `objc2` declarations under `cfg(target_os = "ios")`; it does not use PhotoKit enumeration or undocumented Swift runtime symbols. The provider's temporary file is copied synchronously inside its completion callback because that file is only valid until the callback returns.

## Verification boundary

The registered `ios-photo-picker` profile checks host state/copy regressions, device and Simulator iOS 14.0 compilation, framework imports, public selector linkage, source boundaries, and documentation. Link probes are compiled but not executed. These checks do not prove visible picker presentation, user selection, authorization prompts (none are requested by this API), provider transfer behavior, cancellation timing, file-provider behavior, or physical-device runtime.
