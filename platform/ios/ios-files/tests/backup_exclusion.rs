#[path = "../src/backup_exclusion.rs"]
mod backup_exclusion;

use framework_core::ErrorKind;
use framework_files::FileError;
use objc2_foundation::NSString;
use std::{
    ffi::CString,
    fs::{self, File},
    os::unix::fs::{PermissionsExt, symlink},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

struct Sandbox {
    path: PathBuf,
    root: File,
}

impl Sandbox {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = PathBuf::from(std::env::var_os("HOME").unwrap())
            .join("Library")
            .join("Application Support");
        fs::create_dir_all(&base).unwrap();
        let path = base.join(format!(
            "ios-files-backup-exclusion-{}-{label}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        let root = File::open(&path).unwrap();
        Self { path, root }
    }

    fn leaf(name: &str) -> CString {
        CString::new(name).unwrap()
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn files_and_directories_can_be_excluded_included_and_set_idempotently() {
    let sandbox = Sandbox::new("round-trip");
    fs::write(sandbox.path.join("cache.bin"), b"cache").unwrap();
    fs::create_dir(sandbox.path.join("cache-dir")).unwrap();
    let file = Sandbox::leaf("cache.bin");
    let directory = Sandbox::leaf("cache-dir");

    for leaf in [&file, &directory] {
        backup_exclusion::set_at(&sandbox.root, &sandbox.root, leaf, true).unwrap();
        assert!(
            backup_exclusion::get_at(&sandbox.root, &sandbox.root, leaf).unwrap(),
            "{leaf:?} should be excluded"
        );
        backup_exclusion::set_at(&sandbox.root, &sandbox.root, leaf, true).unwrap();
        assert!(
            backup_exclusion::get_at(&sandbox.root, &sandbox.root, leaf).unwrap(),
            "{leaf:?} repeated exclusion should remain set"
        );
        backup_exclusion::set_at(&sandbox.root, &sandbox.root, leaf, false).unwrap();
        assert!(
            !backup_exclusion::get_at(&sandbox.root, &sandbox.root, leaf).unwrap(),
            "{leaf:?} should be included"
        );
        backup_exclusion::set_at(&sandbox.root, &sandbox.root, leaf, false).unwrap();
        assert!(
            !backup_exclusion::get_at(&sandbox.root, &sandbox.root, leaf).unwrap(),
            "{leaf:?} repeated inclusion should remain cleared"
        );
    }
}

#[test]
fn missing_entries_are_not_created_by_set_or_get() {
    let sandbox = Sandbox::new("missing");
    let missing = Sandbox::leaf("missing.bin");

    assert_eq!(
        backup_exclusion::set_at(&sandbox.root, &sandbox.root, &missing, true)
            .unwrap_err()
            .kind(),
        ErrorKind::NotFound
    );
    assert_eq!(
        backup_exclusion::get_at(&sandbox.root, &sandbox.root, &missing)
            .unwrap_err()
            .kind(),
        ErrorKind::NotFound
    );
    assert!(!sandbox.path.join("missing.bin").exists());
}

#[test]
fn malformed_leaf_forms_are_rejected_before_filesystem_lookup() {
    let sandbox = Sandbox::new("malformed");

    for name in [
        "",
        ".",
        "..",
        "../outside.bin",
        "nested/file.bin",
        "nested\\file.bin",
    ] {
        let leaf = CString::new(name).unwrap();
        assert_eq!(
            backup_exclusion::set_at(&sandbox.root, &sandbox.root, &leaf, true),
            Err(FileError::InvalidPath),
            "{name:?}"
        );
    }
}

#[test]
fn final_symlink_to_outside_file_is_rejected_without_modifying_either_target() {
    let sandbox = Sandbox::new("symlink");
    let outside = Sandbox::new("outside");
    fs::write(sandbox.path.join("inside.bin"), b"inside").unwrap();
    fs::write(outside.path.join("outside.bin"), b"outside").unwrap();
    symlink(
        outside.path.join("outside.bin"),
        sandbox.path.join("redirect"),
    )
    .unwrap();
    let link = Sandbox::leaf("redirect");
    let inside = Sandbox::leaf("inside.bin");
    let outside_file = Sandbox::leaf("outside.bin");

    for result in [
        backup_exclusion::set_at(&sandbox.root, &sandbox.root, &link, true).map(|_| false),
        backup_exclusion::get_at(&sandbox.root, &sandbox.root, &link),
    ] {
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidInput);
    }
    assert!(!backup_exclusion::get_at(&sandbox.root, &sandbox.root, &inside).unwrap());
    assert!(!backup_exclusion::get_at(&outside.root, &outside.root, &outside_file).unwrap());
}

#[test]
fn unreadable_file_reports_permission_failure() {
    let sandbox = Sandbox::new("permission-denied");
    let file_path = sandbox.path.join("unreadable.bin");
    fs::write(&file_path, b"private").unwrap();
    fs::set_permissions(&file_path, fs::Permissions::from_mode(0o0)).unwrap();
    let leaf = Sandbox::leaf("unreadable.bin");
    let result = backup_exclusion::set_at(&sandbox.root, &sandbox.root, &leaf, true);
    fs::set_permissions(&file_path, fs::Permissions::from_mode(0o600)).unwrap();

    assert_eq!(result.unwrap_err().kind(), ErrorKind::PermissionDenied);
}

#[test]
fn foundation_setter_failure_preserves_a_native_error_and_is_not_ignored() {
    let sandbox = Sandbox::new("permission");
    fs::write(sandbox.path.join("immutable.bin"), b"fixed").unwrap();
    let path = CString::new(
        sandbox
            .path
            .join("immutable.bin")
            .as_os_str()
            .as_encoded_bytes(),
    )
    .unwrap();
    let leaf = Sandbox::leaf("immutable.bin");
    // SAFETY: `path` is a live NUL-terminated path to the test file owned by this process.
    assert_eq!(
        unsafe { libc::chflags(path.as_ptr(), libc::UF_IMMUTABLE) },
        0
    );
    let result = backup_exclusion::set_at(&sandbox.root, &sandbox.root, &leaf, true);
    // SAFETY: the same live path is used to release the test's immutable-file flag.
    let clear_result = unsafe { libc::chflags(path.as_ptr(), 0) };

    assert_eq!(clear_result, 0);
    let error = result.unwrap_err();
    assert_ne!(error.kind(), ErrorKind::Unknown);
    assert!(error.platform_code().is_some());
}

#[test]
fn a_successful_set_without_a_matching_readback_is_unsupported() {
    assert_eq!(
        backup_exclusion::verify_set(true, false)
            .unwrap_err()
            .kind(),
        ErrorKind::Unsupported
    );
    assert_eq!(backup_exclusion::verify_set(false, false), Ok(()));
}

#[test]
fn unset_resource_values_mean_false_and_wrong_types_are_unsupported() {
    assert_eq!(backup_exclusion::decode_value(None), Ok(false));
    let wrong_type = NSString::from_str("not a Boolean NSNumber")
        .into_super()
        .into_super();
    assert_eq!(
        backup_exclusion::decode_value(Some(wrong_type))
            .unwrap_err()
            .kind(),
        ErrorKind::Unsupported
    );
}
