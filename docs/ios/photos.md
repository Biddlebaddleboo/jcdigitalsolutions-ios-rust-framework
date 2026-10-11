# iOS Photos authorization, file import, and original-resource fetch

`ios-photos` implements `framework-photos` with Apple's public PhotoKit access-level authorization API. It has no global state or host-app configuration

## Read/write status and request

The backend calls `PHPhotoLibrary::authorizationStatusForAccessLevel(PHAccessLevel::ReadWrite)` for a status query and `PHPhotoLibrary::requestAuthorizationForAccessLevel_handler(PHAccessLevel::ReadWrite, ...)` for an explicit request. It does not call the deprecated no-access-level methods; those methods report limited access as `Authorized`

The native request starts on first poll. Its callback uses synchronized `Arc`-backed state, accepts one result, and wakes after releasing the lock. Dropping the Rust future suppresses its result; PhotoKit offers no cancellation operation through this contract, so no claim is made that native work or system UI stops

The portable status preserves `NotDetermined`, `Restricted`, `Denied`, `Limited`, `Authorized`, and maps unknown future native values to `Unknown`. `Limited` is not converted to `Authorized`. Status does not establish that a later asset operation will succeed

## Read/write host requirement and API floor

For this read/write access scope, the host app must include `NSPhotoLibraryUsageDescription` in its `Info.plist` with a purpose description. This backend does not create, inspect, or modify the host plist

The access-level methods and `PHAuthorizationStatusLimited` require iOS 14.0 or later. The package pins `objc2-photos` 0.3.2 with default features disabled and enables only `PHPhotoLibrary` and `block2`. The generated binding exposes the selected methods and status constants. The package-local Objective-C fixture checks the active SDK header with an iOS 14.0 target; the link/import probe builds device and Simulator binaries without running them

## Add-only file saves

`PhotoLibraryAddOnlyAuthorizationBackend` exposes a separate add-only status and explicit request. Its status type does not contain `Limited`; unknown native values, including any status not defined for add-only access, remain `Unknown`. Use `PhotoLibrary::request_add_only_authorization` only after a user action. The save methods never request authorization and recheck add-only status before calling PhotoKit; all statuses except `Authorized` fail closed. Read/write status and request behavior remain unchanged

`IosPhotosBackend::save_jpeg_from_file` accepts a Rust path to a `.jpg` or `.jpeg` regular file. `save_movie_from_file` accepts `.mov`, `.mp4`, or `.m4v`. Paths must be absolute and losslessly UTF-8; relative, invalid UTF-8, embedded-NUL, missing, unreadable, non-regular, and wrong-extension paths fail before the transaction. PhotoKit validates file contents; filename extensions are not content proof. The implementation canonicalizes and opens the source for validation, then uses a file URL and `PHAssetCreationRequest` resource type `Photo` or `Video`

The host app must provide `NSPhotoLibraryAddUsageDescription` for add-only saves. The operation explicitly sets `PHAssetResourceCreationOptions.shouldMoveFile` to false. It does not move, delete, or mutate the source; the caller must keep it readable and unchanged until the completion future resolves. If the caller supplies a security-scoped path, it must keep its own access scope open until completion. Future drop suppresses the Rust result but does not cancel PhotoKit work

## Original still-photo resource fetch

`IosPhotosBackend::fetch_original_photo_bytes(local_identifier, maximum_byte_count, allow_network_access)` lazily fetches the original still-photo resource for one caller-supplied PhotoKit local identifier. Empty or NUL identifiers and a zero byte cap fail before native work. The first Future poll rechecks read/write status; only `Authorized` and `Limited` proceed. The method never prompts, and limited access only exposes assets the user has made available to the app

The backend passes an array with exactly that one identifier to the public `PHAsset` selector `fetchAssetsWithLocalIdentifiers:options:`, requires an image asset, and selects only a `PHAssetResourceType::Photo` resource. It does not return an edited/full-size variant or a Live Photo paired video. The caller must explicitly allow or disallow network access; when allowed, PhotoKit may retrieve an iCloud-backed resource. Native `NSError` domain and code are retained in the Rust error

PhotoKit supplies `NSData` chunks only for the callback duration. Each chunk is copied into one owned `Vec<u8>` after checked addition, the caller's byte-cap check, and fallible reservation; the future never retains callback-scoped `NSData`. Exceeding the cap, size overflow, allocation failure, a callback panic, or a native error ends the request without returning partial bytes. Dropping the future clears buffered bytes and best-effort cancels its PhotoKit data request without waiting for callbacks; late callbacks are ignored

## Boundary

This crate contains no broad asset enumeration, image-manager preview/degraded request, asset edit, PhotosUI picker, album selection, C ABI, or Swift source. Compile, Clippy, rustdoc, and link/import evidence do not show live authorization UI, host plist behavior, permission outcome, iCloud retrieval, runtime asset access, or runtime asset creation

## Apple references

- [`PHPhotoLibrary`](https://developer.apple.com/documentation/photos/phphotolibrary)
- [`PHAuthorizationStatus`](https://developer.apple.com/documentation/photos/phauthorizationstatus)
- [`PHAsset`](https://developer.apple.com/documentation/photos/phasset)
- [`PHAssetResource`](https://developer.apple.com/documentation/photos/phassetresource)
- [`PHAssetResourceManager`](https://developer.apple.com/documentation/photos/phassetresourcemanager)
- [`NSPhotoLibraryUsageDescription`](https://developer.apple.com/documentation/bundleresources/information-property-list/nsphotolibraryusagedescription)
- [`NSPhotoLibraryAddUsageDescription`](https://developer.apple.com/documentation/bundleresources/information-property-list/nsphotolibraryaddusagedescription)
