# Photos read/write authorization

`framework-photos` defines a portable status and explicit request contract for Photos read/write authorization. It does not enumerate assets, request images, or edit library content

## Use

Select a backend explicitly and query or request status

```rust
use framework_photos::{PhotoLibrary, PhotoLibraryAuthorizationStatus};
use ios_photos::IosPhotosBackend;

async fn request_photos_access() -> PhotoLibraryAuthorizationStatus {
    let mut photos = PhotoLibrary::new(IosPhotosBackend);
    photos.request_authorization().await
}
```

`PhotoLibraryAuthorizationStatus::Limited` remains distinct from `Authorized`. A status is the platform's report for read/write access at query or request completion; it does not guarantee that a later operation will succeed

The synchronous `authorization_status` query does not request authorization. The explicit request begins no earlier than its future's first poll. Dropping a pending future abandons its Rust result but does not promise cancellation of native work or dismissal of system UI. No executor or `Send` requirement is imposed

The iOS backend uses PhotoKit read/write authorization only. The host app must provide `NSPhotoLibraryUsageDescription` in its `Info.plist` with a purpose description. This crate does not configure the host app's plist

## Scope limits

This contract does not enumerate assets, fetch image data, edit or save assets, display a picker, or expose asset identifiers. It makes no claim about a live permission prompt, authorization outcome, or later asset access

See the [iOS Photos guide](../ios/photos.md) for the backend API floor and native boundary. Apple references: [PHPhotoLibrary](https://developer.apple.com/documentation/photos/phphotolibrary), [PHAuthorizationStatus](https://developer.apple.com/documentation/photos/phauthorizationstatus), and [NSPhotoLibraryUsageDescription](https://developer.apple.com/documentation/bundleresources/information-property-list/nsphotolibraryusagedescription)
