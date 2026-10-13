#[cfg(target_os = "ios")]
fn start_and_drop_scoped_access(
    url: &objc2_foundation::NSURL,
) -> Result<(), ios_files::SecurityScopeStartError> {
    let access = ios_files::IosSecurityScopedAccess::start(url)?;
    drop(access);
    Ok(())
}

#[cfg(target_os = "ios")]
fn main() {
    use framework_files::{AppPath, FileError, Files, WriteOutcome};
    use objc2_foundation::NSURL;

    let append_file = Files::<ios_files::IosFiles>::append
        as for<'files, 'path, 'bytes> fn(
            &'files mut Files<ios_files::IosFiles>,
            AppPath<'path>,
            &'bytes [u8],
        ) -> Result<(), FileError>;
    let create_directories = Files::<ios_files::IosFiles>::create_directories
        as for<'files, 'path> fn(
            &'files mut Files<ios_files::IosFiles>,
            AppPath<'path>,
        ) -> Result<(), FileError>;
    let set_excluded_from_backup = ios_files::IosFiles::set_excluded_from_backup
        as for<'files, 'path> fn(
            &'files ios_files::IosFiles,
            AppPath<'path>,
            bool,
        ) -> Result<(), FileError>;
    let is_excluded_from_backup = ios_files::IosFiles::is_excluded_from_backup
        as for<'files, 'path> fn(
            &'files ios_files::IosFiles,
            AppPath<'path>,
        ) -> Result<bool, FileError>;
    let _ = std::hint::black_box((
        ios_files::IosFiles::new as fn() -> Result<ios_files::IosFiles, FileError>,
        ios_files::IosFiles::adopt_url_session_download
            as for<'files, 'url, 'path> fn(
                &'files mut ios_files::IosFiles,
                &'url NSURL,
                AppPath<'path>,
            ) -> Result<WriteOutcome, FileError>,
        append_file,
        create_directories,
        set_excluded_from_backup,
        is_excluded_from_backup,
        start_and_drop_scoped_access
            as fn(&NSURL) -> Result<(), ios_files::SecurityScopeStartError>,
    ));
}

#[cfg(not(target_os = "ios"))]
fn main() {}
