#![deny(warnings)]

#[cfg(target_os = "ios")]
use block2::RcBlock;
#[cfg(target_os = "ios")]
use core::future::Future;
#[cfg(target_os = "ios")]
use core::pin::Pin;
#[cfg(target_os = "ios")]
use ios_photos::IosPhotosBackend;
#[cfg(target_os = "ios")]
use ios_photos::IosPhotosOriginalPhotoFuture;
#[cfg(target_os = "ios")]
use objc2::runtime::Bool;
#[cfg(target_os = "ios")]
use objc2_foundation::{NSError, NSString, NSURL};
#[cfg(target_os = "ios")]
use objc2_photos::{
    PHAccessLevel, PHAssetCreationRequest, PHAssetResourceCreationOptions, PHAssetResourceType,
    PHAuthorizationStatus, PHPhotoLibrary,
};
#[cfg(target_os = "ios")]
use std::path::Path;
#[cfg(target_os = "ios")]
use std::task::{Context, Waker};

#[cfg(target_os = "ios")]
fn main() {
    // This probe is linked for symbol/import inspection and must not be run.
    let _ = core::hint::black_box(unsafe {
        PHPhotoLibrary::authorizationStatusForAccessLevel(PHAccessLevel::ReadWrite)
    });
    let _ = core::hint::black_box(unsafe {
        PHPhotoLibrary::authorizationStatusForAccessLevel(PHAccessLevel::AddOnly)
    });
    let read_write_handler = RcBlock::new(|_status: PHAuthorizationStatus| {});
    unsafe {
        PHPhotoLibrary::requestAuthorizationForAccessLevel_handler(
            PHAccessLevel::ReadWrite,
            &read_write_handler,
        )
    };
    let add_only_handler = RcBlock::new(|_status: PHAuthorizationStatus| {});
    unsafe {
        PHPhotoLibrary::requestAuthorizationForAccessLevel_handler(
            PHAccessLevel::AddOnly,
            &add_only_handler,
        )
    };

    let path = NSString::from_str("/tmp/ios-photos-link-probe.jpg");
    let file_url = NSURL::fileURLWithPath(&path);
    let change_block = RcBlock::new(move || {
        let request = unsafe { PHAssetCreationRequest::creationRequestForAsset() };
        let options = unsafe { PHAssetResourceCreationOptions::new() };
        unsafe {
            options.setShouldMoveFile(false);
            request.addResourceWithType_fileURL_options(
                PHAssetResourceType::Photo,
                &file_url,
                Some(&options),
            );
        }
    });
    let completion = RcBlock::new(|_success: Bool, _error: *mut NSError| {});
    let library = unsafe { PHPhotoLibrary::sharedPhotoLibrary() };
    unsafe {
        library.performChanges_completionHandler(RcBlock::as_ptr(&change_block), Some(&completion))
    };

    let backend = IosPhotosBackend;
    let _ = core::hint::black_box(
        backend.save_jpeg_from_file(Path::new("/tmp/ios-photos-link-probe.jpg")),
    );
    let _ = core::hint::black_box(
        backend.save_movie_from_file(Path::new("/tmp/ios-photos-link-probe.mov")),
    );

    let fetch = backend.fetch_original_photo_bytes("link-probe-id", 1, false);
    if let Ok(future) = fetch {
        let mut future = Box::pin(future);
        let mut context = Context::from_waker(Waker::noop());
        let _ = core::hint::black_box(Future::poll(future.as_mut(), &mut context));
        let _ = core::hint::black_box(
            <IosPhotosOriginalPhotoFuture as Future>::poll
                as fn(
                    Pin<&mut IosPhotosOriginalPhotoFuture>,
                    &mut Context<'_>,
                )
                    -> core::task::Poll<Result<Vec<u8>, ios_photos::PhotosFetchError>>,
        );
    }
}

#[cfg(not(target_os = "ios"))]
fn main() {}
