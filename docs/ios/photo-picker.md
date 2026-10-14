# iOS PhotosUI image picker

`ios-photo-picker` presents the public, out-of-process `PHPickerViewController` for one or a bounded set of images. It does not open the Photos database, enumerate assets, request PhotoKit authorization, or require `NSPhotoLibraryUsageDescription` for picker use. The existing `ios-photos` crate remains a separate PhotoKit authorization/read/save surface and continues to reject PhotosUI imports.

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
        IosPhotoPickerEvent::SelectedMultiple(images) => consume_many(images),
        IosPhotoPickerEvent::Cancelled => {}
        IosPhotoPickerEvent::Failed(error) => report(error),
    },
)?;
```

For a bounded multi-image session, call `IosPhotoPickerSession::present_multiple` with a limit and aggregate ceiling:

```rust,ignore
let session = IosPhotoPickerSession::present_multiple(
    main_thread,
    presenter,
    10,
    256 * 1024 * 1024,
    move |event| match event {
        IosPhotoPickerEvent::SelectedMultiple(images) => consume_many(images),
        IosPhotoPickerEvent::Cancelled => {},
        IosPhotoPickerEvent::Failed(error) => report(error),
        IosPhotoPickerEvent::Selected(image) => drop(image),
        _ => {},
    },
)?;
```

The existing `IosPhotoPickerSession::present` API still limits selection to one result and keeps its per-image `max_bytes` contract. The new `present_multiple` API takes `selection_limit` and `max_aggregate_bytes`; the selection limit must be 2 through 10, and the aggregate-byte ceiling must be nonzero and no greater than 256 MiB. The UI may return fewer than the configured limit, including one image. `IosPhotoPickerEvent::SelectedMultiple` contains the copied images in the order reported by PhotosUI.

Both APIs apply `PHPickerFilter.imagesFilter()` and prefer `PHPickerConfigurationAssetRepresentationMode::Current`. Each selected `NSItemProvider` supplies its `public.image` file representation. Rust performs no image decode or transcode; it synchronously copies the exact provider-file bytes into a uniquely created child of Foundation's `NSTemporaryDirectory()`. Each result exposes its copied byte length and is rejected if it is empty, not a regular file, or exceeds its applicable byte ceiling. Multi-image transfers reserve each provider-reported file length before copy under one synchronized aggregate budget; at most 10 provider requests can be retained at once. The copy uses a fixed 32 KiB buffer, checked byte counts, a `0700` directory, and a `0600` file.

Multi-image completion is all-or-nothing: a provider, file, copy, or aggregate-size failure publishes `Failed` once and removes any already-copied files. Other provider requests already started may finish, but their late callbacks do not copy or publish results. `cancel()` publishes `Cancelled` once, cancels all retained `NSProgress` objects, and removes partial output. Dropping the pending session cancels provider progress, detaches the callback, suppresses late events, and removes partial output. Callback panics are contained.

The copy is byte-identical to the provider's temporary representation, **not necessarily to the library's original file**: PhotosUI may provide a compatible representation. The provider's temporary file is deleted when its callback returns, so the copy occurs inside that callback before its return. The result file remains valid while `IosPickedImage` is alive and is removed on drop. `IosPickedImage::into_path` transfers file and parent-directory cleanup to the caller.

The callback can run on the main thread for selection cancellation or on a provider-controlled background queue for transfer results. UI operations remain main-thread-only.

## Public SDK binding evidence

The selected Xcode iPhoneOS SDK declares `PHPickerViewController`, `PHPickerViewControllerDelegate`, `initWithConfiguration:`, and `picker:didFinishPicking:` in `System/Library/Frameworks/PhotosUI.framework/Headers/PHPicker.h`. Its iOS availability region declares the controller as a `UIViewController` under `TARGET_OS_IPHONE`; the same header documents the picker as out-of-process and says Photo Library authorization is not needed. `platform.rs` binds only that public UIKit controller/delegate surface with crate-private `objc2` declarations under `cfg(target_os = "ios")`; it does not use PhotoKit enumeration or undocumented Swift runtime symbols. The provider's temporary file is copied synchronously inside its completion callback because that file is only valid until the callback returns.

## Verification boundary

The registered `ios-photo-picker` profile checks host selection/state/copy regressions, device and Simulator iOS 14.0 compilation, framework imports, public selector linkage, source boundaries, and documentation. Link probes are compiled but not executed. These checks do not prove visible picker presentation, user selection, authorization prompts (none are requested by this API), provider transfer behavior, cancellation timing, file-provider behavior, or physical-device runtime.
