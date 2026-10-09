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
    use framework_files::{AppPath, FileError, WriteOutcome};
    use objc2_foundation::NSURL;

    let _ = std::hint::black_box((
        ios_files::IosFiles::new as fn() -> Result<ios_files::IosFiles, FileError>,
        ios_files::IosFiles::adopt_url_session_download
            as for<'files, 'url, 'path> fn(
                &'files mut ios_files::IosFiles,
                &'url NSURL,
                AppPath<'path>,
            ) -> Result<WriteOutcome, FileError>,
        start_and_drop_scoped_access
            as fn(&NSURL) -> Result<(), ios_files::SecurityScopeStartError>,
    ));
}

#[cfg(not(target_os = "ios"))]
fn main() {}
