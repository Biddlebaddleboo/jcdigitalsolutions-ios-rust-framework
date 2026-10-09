#[path = "../src/path_validation.rs"]
mod path_validation;

use framework_files::FileError;

#[test]
fn native_path_validation_splits_relative_components_without_normalizing() {
    // This exercises lexical path validation only, not filesystem containment.
    let parts = path_validation::path_parts("folder/subfolder/file.txt").unwrap();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0].as_bytes(), b"folder");
    assert_eq!(parts[1].as_bytes(), b"subfolder");
    assert_eq!(parts[2].as_bytes(), b"file.txt");
}

#[test]
fn native_path_validation_rejects_unsafe_relative_forms() {
    for path in [
        "",
        "/absolute",
        "trailing/",
        "C:folder",
        "z:/folder",
        "folder//file",
        ".",
        "..",
        "folder/./file",
        "folder/../file",
        "folder\\file",
        "folder\0file",
    ] {
        assert_eq!(
            path_validation::path_parts(path),
            Err(FileError::InvalidPath),
            "{path:?}"
        );
    }
}
