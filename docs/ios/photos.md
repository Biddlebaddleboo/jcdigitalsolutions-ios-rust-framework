# iOS Photos authorization

`ios-photos` implements `framework-photos` with Apple's public PhotoKit access-level authorization API. It has no global state or host-app configuration

## Read/write status and request

The backend calls `PHPhotoLibrary::authorizationStatusForAccessLevel(PHAccessLevel::ReadWrite)` for a status query and `PHPhotoLibrary::requestAuthorizationForAccessLevel_handler(PHAccessLevel::ReadWrite, ...)` for an explicit request. It does not call the deprecated no-access-level methods; those methods report limited access as `Authorized`

The native request starts on first poll. Its callback uses synchronized `Arc`-backed state, accepts one result, and wakes after releasing the lock. Dropping the Rust future suppresses its result; PhotoKit offers no cancellation operation through this contract, so no claim is made that native work or system UI stops

The portable status preserves `NotDetermined`, `Restricted`, `Denied`, `Limited`, `Authorized`, and maps unknown future native values to `Unknown`. `Limited` is not converted to `Authorized`. Status does not establish that a later asset operation will succeed

## Host app requirement and API floor

For this read/write access scope, the host app must include `NSPhotoLibraryUsageDescription` in its `Info.plist` with a purpose description. This backend does not create, inspect, or modify the host plist

The access-level methods and `PHAuthorizationStatusLimited` require iOS 14.0 or later. The package pins `objc2-photos` 0.3.2 with default features disabled and enables only `PHPhotoLibrary` and `block2`. The generated binding exposes the selected methods and status constants. The package-local Objective-C fixture checks the active SDK header with an iOS 14.0 target; the link/import probe builds device and Simulator binaries without running them

## Boundary

This crate contains no asset enumeration, image request, asset edit, PhotosUI picker, C ABI, or Swift source. Compile, Clippy, rustdoc, and link/import evidence do not show live authorization UI, host plist behavior, permission outcome, or asset access

## Apple references

- [`PHPhotoLibrary`](https://developer.apple.com/documentation/photos/phphotolibrary)
- [`PHAuthorizationStatus`](https://developer.apple.com/documentation/photos/phauthorizationstatus)
- [`NSPhotoLibraryUsageDescription`](https://developer.apple.com/documentation/bundleresources/information-property-list/nsphotolibraryusagedescription)
