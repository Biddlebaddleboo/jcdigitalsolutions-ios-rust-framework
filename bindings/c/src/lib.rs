#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Static C ABI entry points over the framework's Rust-native foundation types."]

mod options_v1;

pub use framework_abi::{
    FrameworkCompletionCallback, FrameworkErrorHandle, FrameworkOperationHandle,
    FrameworkOptionsV1, FrameworkOwnedBuffer, FrameworkSlice, FrameworkStatus, FrameworkStr,
    framework_owned_buffer_destroy,
};
pub use options_v1::framework_options_v1_validate;

#[cfg(any(
    feature = "ios-transfer",
    feature = "ios-clipboard",
    feature = "ios-share",
    feature = "ios-preferences",
    feature = "secure-storage",
    feature = "notification-responses",
    feature = "ios-spritekit",
    feature = "ios-maps",
    feature = "ios-classkit-deep-link",
    feature = "ios-location",
    feature = "ios-file-provider",
    feature = "ios-proximity-reader",
    feature = "ios-crypto",
    feature = "ios-modelio-status",
    feature = "ios-sign-in-with-apple-status"
))]
extern crate alloc;

#[cfg(feature = "ios-file-provider")]
extern crate std;

#[cfg(feature = "ios-transfer")]
mod ios_transfer;

#[cfg(feature = "ios-clipboard")]
mod ios_clipboard;

#[cfg(feature = "ios-share")]
mod ios_share;

#[cfg(feature = "ios-preferences")]
mod ios_preferences;

#[cfg(feature = "ios-media-library-status")]
mod ios_media_library_status;

#[cfg(feature = "ios-spritekit")]
mod ios_spritekit;

#[cfg(feature = "ios-call-observer")]
mod ios_call_observer;

#[cfg(feature = "ios-maps")]
mod ios_maps;

#[cfg(feature = "ios-classkit-deep-link")]
mod ios_classkit_deep_link;

#[cfg(feature = "ios-location")]
mod ios_location;

#[cfg(feature = "ios-file-provider")]
mod ios_file_provider;

#[cfg(feature = "ios-vision")]
mod ios_vision;

#[cfg(feature = "ios-proximity-reader")]
mod ios_proximity_reader;

#[cfg(feature = "ios-crypto")]
mod ios_crypto;

#[cfg(feature = "ios-modelio-status")]
mod ios_modelio_status;

#[cfg(feature = "ios-sign-in-with-apple-status")]
mod ios_sign_in_with_apple_status;

#[cfg(feature = "ios-accelerate")]
mod ios_accelerate;

#[cfg(feature = "ios-key-support")]
mod ios_key_support;

#[cfg(feature = "ios-mps-status")]
mod ios_mps_status;

#[cfg(feature = "ios-videotoolbox")]
mod ios_videotoolbox;

#[cfg(feature = "ios-camera-device-status")]
mod ios_camera_device_status;

#[cfg(feature = "ios-core-ml-status")]
mod ios_core_ml_status;

#[cfg(feature = "ios-speech-status")]
mod ios_speech_status;

#[cfg(feature = "ios-natural-language-status")]
mod ios_natural_language_status;

#[cfg(feature = "ios-extension-support")]
mod ios_extension_support;

#[cfg(feature = "ios-roomplan-status")]
mod ios_roomplan_status;

#[cfg(feature = "ios-storekit2-status")]
mod ios_storekit2_status;

#[cfg(feature = "ios-game-status")]
mod ios_game_status;

#[cfg(feature = "notification-responses")]
mod notification_responses;

#[cfg(feature = "secure-storage")]
mod secure_storage;

#[cfg(feature = "notification-responses")]
pub use notification_responses::{
    FRAMEWORK_NOTIFICATION_RESPONSE_KIND_CUSTOM_ACTION,
    FRAMEWORK_NOTIFICATION_RESPONSE_KIND_DEFAULT, FRAMEWORK_NOTIFICATION_RESPONSE_KIND_DISMISS,
    FRAMEWORK_NOTIFICATION_RESPONSE_KIND_TEXT_INPUT, FrameworkNotificationResponse,
    FrameworkNotificationResponseKind, FrameworkNotificationResponseViewV1,
    framework_notification_response_create, framework_notification_response_destroy,
    framework_notification_response_get_view,
};

#[cfg(feature = "ios-transfer")]
pub use ios_transfer::{
    FRAMEWORK_TRANSFER_AVAILABILITY_AVAILABLE,
    FRAMEWORK_TRANSFER_AVAILABILITY_REQUIRES_ENTITLEMENT,
    FRAMEWORK_TRANSFER_AVAILABILITY_REQUIRES_PERMISSION,
    FRAMEWORK_TRANSFER_AVAILABILITY_TEMPORARILY_UNAVAILABLE,
    FRAMEWORK_TRANSFER_AVAILABILITY_UNKNOWN, FRAMEWORK_TRANSFER_AVAILABILITY_UNSUPPORTED,
    FRAMEWORK_TRANSFER_DIRECTORY_APPLICATION_SUPPORT, FRAMEWORK_TRANSFER_DIRECTORY_CACHES,
    FRAMEWORK_TRANSFER_DIRECTORY_DOCUMENTS, FRAMEWORK_TRANSFER_DIRECTORY_TEMPORARY,
    FRAMEWORK_TRANSFER_ERROR_KIND_ALREADY_EXISTS, FRAMEWORK_TRANSFER_ERROR_KIND_CANCELLED,
    FRAMEWORK_TRANSFER_ERROR_KIND_INTERNAL, FRAMEWORK_TRANSFER_ERROR_KIND_INVALID_INPUT,
    FRAMEWORK_TRANSFER_ERROR_KIND_NOT_FOUND, FRAMEWORK_TRANSFER_ERROR_KIND_PERMISSION_DENIED,
    FRAMEWORK_TRANSFER_ERROR_KIND_PLATFORM, FRAMEWORK_TRANSFER_ERROR_KIND_RESOURCE_EXHAUSTED,
    FRAMEWORK_TRANSFER_ERROR_KIND_TIMEOUT, FRAMEWORK_TRANSFER_ERROR_KIND_UNAVAILABLE,
    FRAMEWORK_TRANSFER_ERROR_KIND_UNKNOWN, FRAMEWORK_TRANSFER_ERROR_KIND_UNSUPPORTED,
    FRAMEWORK_TRANSFER_STATE_ACTIVE, FRAMEWORK_TRANSFER_STATE_CANCELLED,
    FRAMEWORK_TRANSFER_STATE_FAILED, FRAMEWORK_TRANSFER_STATE_QUEUED,
    FRAMEWORK_TRANSFER_STATE_SUCCEEDED, FrameworkIosTransferClient,
    FrameworkIosTransferEventsCompletion, FrameworkIosTransferSnapshot,
    FrameworkTransferAvailability, FrameworkTransferDirectory, FrameworkTransferErrorKind,
    FrameworkTransferHeaderV1, FrameworkTransferHeaderViewV1, FrameworkTransferIdV1,
    FrameworkTransferRequestV1, FrameworkTransferSnapshotViewV1, FrameworkTransferState,
    framework_ios_transfer_availability, framework_ios_transfer_cancel,
    framework_ios_transfer_client_create, framework_ios_transfer_client_destroy,
    framework_ios_transfer_finish_launch_without_background_events, framework_ios_transfer_forget,
    framework_ios_transfer_forward_background_events, framework_ios_transfer_snapshot_destroy,
    framework_ios_transfer_snapshot_get_header, framework_ios_transfer_snapshot_get_view,
    framework_ios_transfer_start_download, framework_ios_transfer_status,
};

#[cfg(feature = "ios-clipboard")]
pub use ios_clipboard::{
    FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_AVAILABLE,
    FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_REQUIRES_ENTITLEMENT,
    FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_REQUIRES_PERMISSION,
    FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_TEMPORARILY_UNAVAILABLE,
    FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_UNKNOWN, FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_UNSUPPORTED,
    FrameworkIosClipboard, FrameworkIosClipboardAvailability, framework_ios_clipboard_availability,
    framework_ios_clipboard_clear, framework_ios_clipboard_create, framework_ios_clipboard_destroy,
    framework_ios_clipboard_read, framework_ios_clipboard_write,
};

#[cfg(feature = "ios-share")]
pub use ios_share::{
    FRAMEWORK_IOS_SHARE_AVAILABILITY_AVAILABLE,
    FRAMEWORK_IOS_SHARE_AVAILABILITY_REQUIRES_ENTITLEMENT,
    FRAMEWORK_IOS_SHARE_AVAILABILITY_REQUIRES_PERMISSION,
    FRAMEWORK_IOS_SHARE_AVAILABILITY_TEMPORARILY_UNAVAILABLE,
    FRAMEWORK_IOS_SHARE_AVAILABILITY_UNKNOWN, FRAMEWORK_IOS_SHARE_AVAILABILITY_UNSUPPORTED,
    FRAMEWORK_IOS_SHARE_ITEM_TEXT, FRAMEWORK_IOS_SHARE_ITEM_URL,
    FRAMEWORK_IOS_SHARE_OUTCOME_COMPLETED, FRAMEWORK_IOS_SHARE_OUTCOME_DISMISSED,
    FrameworkIosShareAnchorV1, FrameworkIosShareAvailability, FrameworkIosShareCompletion,
    FrameworkIosShareItemV1, FrameworkIosShareOutcome, FrameworkIosShareRequestV1,
    FrameworkIosShareSession, framework_ios_share_cancel, framework_ios_share_session_availability,
    framework_ios_share_session_create, framework_ios_share_session_destroy,
    framework_ios_share_start,
};

#[cfg(feature = "ios-preferences")]
pub use ios_preferences::{
    FRAMEWORK_IOS_PREFERENCES_ALLOW_NON_ATOMIC, FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_AVAILABLE,
    FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_REQUIRES_ENTITLEMENT,
    FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_REQUIRES_PERMISSION,
    FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_TEMPORARILY_UNAVAILABLE,
    FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_UNKNOWN,
    FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_UNSUPPORTED, FRAMEWORK_IOS_PREFERENCES_REQUIRE_ATOMIC,
    FRAMEWORK_IOS_PREFERENCES_UPDATE_ATOMICITY_ATOMIC,
    FRAMEWORK_IOS_PREFERENCES_UPDATE_ATOMICITY_NOT_GUARANTEED,
    FRAMEWORK_IOS_PREFERENCES_UPDATE_ATOMICITY_UNKNOWN, FrameworkIosPreferences,
    FrameworkIosPreferencesAvailability, FrameworkIosPreferencesUpdateAtomicity,
    FrameworkIosPreferencesUpdateRequirement, framework_ios_preferences_availability,
    framework_ios_preferences_create, framework_ios_preferences_destroy,
    framework_ios_preferences_get, framework_ios_preferences_remove, framework_ios_preferences_set,
};

#[cfg(feature = "ios-spritekit")]
pub use ios_spritekit::{
    FrameworkIosSpriteKitNode, framework_ios_spritekit_node_create,
    framework_ios_spritekit_node_destroy, framework_ios_spritekit_node_get_position,
    framework_ios_spritekit_node_set_position,
};

#[cfg(feature = "ios-call-observer")]
pub use ios_call_observer::{
    FRAMEWORK_IOS_CALL_OBSERVER_STATE_CONNECTED, FRAMEWORK_IOS_CALL_OBSERVER_STATE_ENDED,
    FRAMEWORK_IOS_CALL_OBSERVER_STATE_ON_HOLD, FRAMEWORK_IOS_CALL_OBSERVER_STATE_OUTGOING,
    FrameworkIosCallObserverStateFlags, framework_ios_call_observer_active_call_snapshot,
};

#[cfg(feature = "ios-maps")]
pub use ios_maps::{
    FrameworkIosMapsDistanceMeters, framework_ios_maps_coordinate_for_map_point,
    framework_ios_maps_map_point_for_coordinate, framework_ios_maps_meters_between_map_points,
};

#[cfg(feature = "ios-classkit-deep-link")]
pub use ios_classkit_deep_link::{
    FrameworkIosClassKitBoolean, framework_ios_classkit_is_deep_link,
};

#[cfg(feature = "ios-location")]
pub use ios_location::{
    FRAMEWORK_IOS_LOCATION_AUTHORIZATION_BACKGROUND, FRAMEWORK_IOS_LOCATION_AUTHORIZATION_DENIED,
    FRAMEWORK_IOS_LOCATION_AUTHORIZATION_FOREGROUND,
    FRAMEWORK_IOS_LOCATION_AUTHORIZATION_NOT_DETERMINED,
    FRAMEWORK_IOS_LOCATION_AUTHORIZATION_RESTRICTED, FRAMEWORK_IOS_LOCATION_AUTHORIZATION_UNKNOWN,
    FRAMEWORK_IOS_LOCATION_AVAILABILITY_AVAILABLE,
    FRAMEWORK_IOS_LOCATION_AVAILABILITY_REQUIRES_ENTITLEMENT,
    FRAMEWORK_IOS_LOCATION_AVAILABILITY_REQUIRES_PERMISSION,
    FRAMEWORK_IOS_LOCATION_AVAILABILITY_TEMPORARILY_UNAVAILABLE,
    FRAMEWORK_IOS_LOCATION_AVAILABILITY_UNKNOWN, FRAMEWORK_IOS_LOCATION_AVAILABILITY_UNSUPPORTED,
    FRAMEWORK_IOS_LOCATION_OPERATION_AUTHORIZATION_QUERY,
    FRAMEWORK_IOS_LOCATION_OPERATION_AUTHORIZATION_REQUEST,
    FRAMEWORK_IOS_LOCATION_OPERATION_CURRENT, FrameworkIosLocationAuthorization,
    FrameworkIosLocationAvailability, FrameworkIosLocationOperation,
    FrameworkIosLocationOperationKind, FrameworkIosLocationReady, FrameworkIosLocationResultV1,
    framework_ios_location_authorization_query_start,
    framework_ios_location_authorization_request_start, framework_ios_location_availability,
    framework_ios_location_current_start, framework_ios_location_operation_cancel,
    framework_ios_location_operation_destroy, framework_ios_location_operation_poll,
};

#[cfg(feature = "ios-file-provider")]
pub use ios_file_provider::{
    FrameworkIosFileProviderOperation, FrameworkIosFileProviderReady,
    FrameworkIosFileProviderResultV1, framework_ios_file_provider_operation_destroy,
    framework_ios_file_provider_operation_poll,
    framework_ios_file_provider_registered_domain_presence_start,
};

#[cfg(feature = "ios-vision")]
pub use ios_vision::framework_ios_vision_text_recognition_revision_is_supported;

#[cfg(feature = "ios-proximity-reader")]
pub use ios_proximity_reader::{
    FrameworkIosProximityReaderBoolean,
    framework_ios_proximity_reader_tap_to_pay_device_model_supported,
};

#[cfg(feature = "ios-crypto")]
pub use ios_crypto::framework_ios_crypto_sha256;

#[cfg(feature = "ios-modelio-status")]
pub use ios_modelio_status::framework_ios_modelio_can_import_file_extension;

#[cfg(feature = "ios-accelerate")]
pub use ios_accelerate::framework_ios_accelerate_vector_add;

#[cfg(feature = "ios-key-support")]
pub use ios_key_support::framework_ios_key_support_p256_ecdsa_sha256_message_supported;

#[cfg(feature = "ios-mps-status")]
pub use ios_mps_status::framework_ios_mps_status_preferred_device_available;

#[cfg(feature = "ios-videotoolbox")]
pub use ios_videotoolbox::framework_ios_videotoolbox_hardware_decode_supported;

#[cfg(feature = "ios-camera-device-status")]
pub use ios_camera_device_status::framework_ios_camera_device_status_has_default_video_capture_device;

#[cfg(feature = "ios-core-ml-status")]
pub use ios_core_ml_status::framework_ios_core_ml_status_has_available_compute_device;

#[cfg(feature = "ios-speech-status")]
pub use ios_speech_status::{
    FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_AUTHORIZED,
    FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_DENIED,
    FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_NOT_DETERMINED,
    FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_RESTRICTED, FrameworkIosSpeechAuthorizationStatus,
    framework_ios_speech_status_authorization_status,
};

#[cfg(feature = "ios-natural-language-status")]
pub use ios_natural_language_status::{
    FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_ASSETS_AVAILABLE,
    FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_ASSETS_NOT_AVAILABLE,
    FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_LANGUAGE_UNAVAILABLE,
    FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_NO_MODEL,
    FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_UNAVAILABLE,
    FrameworkIosNaturalLanguageAssetStatus,
    framework_ios_natural_language_english_contextual_embedding_assets,
};

#[cfg(feature = "ios-extension-support")]
pub use ios_extension_support::{
    FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_BUNDLE_UNAVAILABLE,
    FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_EMPTY_POINT_IDENTIFIER,
    FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INFO_DICTIONARY_UNAVAILABLE,
    FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INVALID_BUNDLE_PATH,
    FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INVALID_EXTENSION_DICTIONARY_TYPE,
    FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INVALID_POINT_IDENTIFIER_TYPE,
    FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_MISSING_EXTENSION_DICTIONARY,
    FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_MISSING_POINT_IDENTIFIER,
    FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_NONE, FrameworkIosExtensionMetadataError,
    framework_ios_extension_support_read_extension_point_identifier,
};

#[cfg(feature = "ios-roomplan-status")]
pub use ios_roomplan_status::framework_ios_roomplan_status_is_supported;

#[cfg(feature = "ios-storekit2-status")]
pub use ios_storekit2_status::framework_ios_storekit2_status_can_make_payments;

#[cfg(feature = "ios-game-status")]
pub use ios_game_status::framework_ios_game_status_is_local_player_authenticated;

#[cfg(feature = "ios-sign-in-with-apple-status")]
pub use ios_sign_in_with_apple_status::{
    FRAMEWORK_IOS_SIGN_IN_WITH_APPLE_CREDENTIAL_STATE_AUTHORIZED,
    FRAMEWORK_IOS_SIGN_IN_WITH_APPLE_CREDENTIAL_STATE_NOT_FOUND,
    FRAMEWORK_IOS_SIGN_IN_WITH_APPLE_CREDENTIAL_STATE_REVOKED,
    FRAMEWORK_IOS_SIGN_IN_WITH_APPLE_CREDENTIAL_STATE_TRANSFERRED,
    FrameworkIosSignInWithAppleCredentialState,
    FrameworkIosSignInWithAppleCredentialStateCompletion,
    framework_ios_sign_in_with_apple_credential_state_start,
};

#[cfg(feature = "ios-media-library-status")]
pub use ios_media_library_status::{
    FRAMEWORK_IOS_MEDIA_LIBRARY_AUTHORIZATION_AUTHORIZED,
    FRAMEWORK_IOS_MEDIA_LIBRARY_AUTHORIZATION_DENIED,
    FRAMEWORK_IOS_MEDIA_LIBRARY_AUTHORIZATION_NOT_DETERMINED,
    FRAMEWORK_IOS_MEDIA_LIBRARY_AUTHORIZATION_RESTRICTED,
    FrameworkIosMediaLibraryAuthorizationStatus, framework_ios_media_library_authorization_status,
};

#[cfg(feature = "secure-storage")]
pub use secure_storage::{
    framework_ios_secure_storage_read, framework_ios_secure_storage_remove,
    framework_ios_secure_storage_store,
};

use framework_abi::{ABI_VERSION_MAJOR, ABI_VERSION_MINOR};

/// Returns the ABI version as major in the high 32 bits and minor in the low 32 bits.
#[unsafe(no_mangle)]
pub extern "C" fn framework_abi_version() -> u64 {
    ((ABI_VERSION_MAJOR as u64) << 32) | ABI_VERSION_MINOR as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::{align_of, offset_of, size_of};

    fn round_up(value: usize, alignment: usize) -> usize {
        value.div_ceil(alignment) * alignment
    }

    #[test]
    fn version_uses_fixed_width_major_minor_encoding() {
        let version = framework_abi_version();
        assert_eq!((version >> 32) as u32, ABI_VERSION_MAJOR);
        assert_eq!(version as u32, ABI_VERSION_MINOR);
    }

    #[test]
    fn exported_header_layout_matches_framework_abi_types() {
        assert_eq!(
            (size_of::<FrameworkStatus>(), align_of::<FrameworkStatus>()),
            (4, 4)
        );
        assert_eq!(size_of::<FrameworkOperationHandle>(), 8);
        assert_eq!(size_of::<FrameworkErrorHandle>(), 8);
        type CFunction = unsafe extern "C" fn(
            *mut core::ffi::c_void,
            FrameworkOperationHandle,
            FrameworkStatus,
            FrameworkSlice,
        );
        assert_eq!(
            size_of::<FrameworkCompletionCallback>(),
            size_of::<Option<CFunction>>()
        );
        assert_eq!(
            align_of::<FrameworkCompletionCallback>(),
            align_of::<Option<CFunction>>()
        );
        assert_eq!(
            (
                size_of::<FrameworkOptionsV1>(),
                align_of::<FrameworkOptionsV1>()
            ),
            (16, 4)
        );
        assert_eq!(offset_of!(FrameworkOptionsV1, struct_size), 0);
        assert_eq!(offset_of!(FrameworkOptionsV1, abi_version), 4);
        assert_eq!(offset_of!(FrameworkOptionsV1, flags), 8);
        assert_eq!(offset_of!(FrameworkOptionsV1, reserved), 12);
        let pointer = size_of::<*const u8>();
        let u64_align = align_of::<u64>();
        let aggregate_align = u64_align.max(align_of::<*const u8>());
        let length_offset = round_up(pointer, u64_align);
        assert_eq!(
            size_of::<FrameworkSlice>(),
            round_up(length_offset + 8, aggregate_align)
        );
        assert_eq!(align_of::<FrameworkSlice>(), aggregate_align);
        assert_eq!(size_of::<FrameworkStr>(), size_of::<FrameworkSlice>());
        assert_eq!(align_of::<FrameworkStr>(), align_of::<FrameworkSlice>());
        assert_eq!(
            size_of::<FrameworkOwnedBuffer>(),
            round_up(length_offset + 16, aggregate_align)
        );
        assert_eq!(align_of::<FrameworkOwnedBuffer>(), aggregate_align);
    }
}
