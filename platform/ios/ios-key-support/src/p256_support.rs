use core::ffi::c_void;
use core::ptr::{self, NonNull};
use framework_core::{Error, ErrorKind, PlatformErrorCode, Result};
use framework_key_support::{
    P256PublicKey, P256VerificationSupport, P256VerificationSupportBackend,
};
use objc2_core_foundation::{
    CFData, CFDictionary, CFError, CFIndex, CFNumber, CFNumberType, CFRetained, CFType,
    kCFTypeDictionaryKeyCallBacks, kCFTypeDictionaryValueCallBacks,
};
use objc2_security::{
    SecKey, SecKeyOperationType, kSecAttrKeyClass, kSecAttrKeyClassPublic, kSecAttrKeySizeInBits,
    kSecAttrKeyType, kSecAttrKeyTypeECSECPrimeRandom,
    kSecKeyAlgorithmECDSASignatureMessageX962SHA256,
};

/// Performs an in-memory Security suitability query for one P-256 public key.
#[derive(Clone, Copy, Debug, Default)]
pub struct IosP256VerificationSupport;

impl IosP256VerificationSupport {
    /// Creates the stateless backend.
    pub const fn new() -> Self {
        Self
    }

    /// Reports whether Security considers this key suitable for ECDSA/SHA-256 message verification.
    ///
    /// This imports the caller's public key only for the query. It does not persist the key,
    /// generate or use a private key, perform verification, or guarantee a later verification call.
    /// The API is available from iOS 10.0. Call this only when the app's deployment target is iOS
    /// 10.0 or later.
    pub fn query(&self, public_key: P256PublicKey<'_>) -> Result<P256VerificationSupport> {
        <Self as P256VerificationSupportBackend>::query_p256_ecdsa_sha256_message_verification(
            self, public_key,
        )
    }
}

impl P256VerificationSupportBackend for IosP256VerificationSupport {
    fn query_p256_ecdsa_sha256_message_verification(
        &self,
        public_key: P256PublicKey<'_>,
    ) -> Result<P256VerificationSupport> {
        let bytes = public_key.as_x963_uncompressed_bytes();
        // SAFETY: The byte array is a live, fixed-size public-key value; the default allocator is
        // permitted by CFDataCreate. Core Foundation copies these 65 bytes into retained CFData.
        let data = unsafe { CFData::new(None, bytes.as_ptr(), bytes.len() as CFIndex) }
            .ok_or_else(|| Error::new(ErrorKind::ResourceExhausted))?;
        let attributes = public_key_attributes()?;
        let mut raw_error: *mut CFError = ptr::null_mut();
        // SAFETY: `attributes` holds the required key-type and key-class CFString pairs with
        // retaining type callbacks. `data` remains live for the call, and `raw_error` is writable.
        let imported_key = unsafe { SecKey::with_data(&data, &attributes, &mut raw_error) };
        let native_error = NonNull::new(raw_error).map(|error| {
            // SAFETY: SecKeyCreateWithData returns this owned CFError on failure.
            unsafe { CFRetained::from_raw(error) }
        });
        let Some(imported_key) = imported_key else {
            return Err(platform_error(native_error));
        };

        // SAFETY: The retained key came from Security, and the algorithm is Apple's static
        // ECDSA/SHA-256 message-verification constant.
        let supported = unsafe {
            imported_key.is_algorithm_supported(
                SecKeyOperationType::Verify,
                kSecKeyAlgorithmECDSASignatureMessageX962SHA256,
            )
        };
        Ok(P256VerificationSupport::from_platform_query(supported))
    }
}

fn public_key_attributes() -> Result<CFRetained<CFDictionary>> {
    // SAFETY: These generated Security constants are immutable process-lifetime CFStrings.
    let (key_type, key_class, key_size_key, p256_type, public_class) = unsafe {
        (
            kSecAttrKeyType,
            kSecAttrKeyClass,
            kSecAttrKeySizeInBits,
            kSecAttrKeyTypeECSECPrimeRandom,
            kSecAttrKeyClassPublic,
        )
    };
    let key_size_bits = 256_i32;
    // SAFETY: The pointer addresses a live i32 matching `SInt32Type`; Core Foundation's default
    // allocator is permitted. The returned number is retained for the dictionary construction.
    let key_size = unsafe {
        CFNumber::new(
            None,
            CFNumberType::SInt32Type,
            (&key_size_bits as *const i32).cast(),
        )
    }
    .ok_or_else(|| Error::new(ErrorKind::ResourceExhausted))?;
    let keys: [&CFType; 3] = [key_type, key_class, key_size_key];
    let values: [&CFType; 3] = [p256_type, public_class, &key_size];
    let mut raw_keys = keys.map(|value| value as *const CFType as *const c_void);
    let mut raw_values = values.map(|value| value as *const CFType as *const c_void);
    // SAFETY: The key array contains three live CFStrings; the value array contains two live
    // CFStrings and the live CFNumber. The type callbacks retain the entries, and both arrays
    // agree with the count and remain live during dictionary creation.
    let attributes = unsafe {
        CFDictionary::new(
            None,
            raw_keys.as_mut_ptr(),
            raw_values.as_mut_ptr(),
            3,
            &kCFTypeDictionaryKeyCallBacks,
            &kCFTypeDictionaryValueCallBacks,
        )
    };
    attributes.ok_or_else(|| Error::new(ErrorKind::ResourceExhausted))
}

fn platform_error(error: Option<CFRetained<CFError>>) -> Error {
    let mut result = Error::new(ErrorKind::Platform);
    if let Some(error) = error {
        if let Ok(code) = i32::try_from(error.code()) {
            if let Some(code) = PlatformErrorCode::new(code) {
                result = result.with_platform_code(code);
            }
        }
    }
    result
}
