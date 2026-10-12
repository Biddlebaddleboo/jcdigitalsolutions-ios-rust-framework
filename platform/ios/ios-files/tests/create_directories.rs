#[path = "../src/create_directories.rs"]
mod create_directories;

use std::{
    ffi::CString,
    fs::{self, File},
    io,
    os::unix::fs::symlink,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(1);

struct Sandbox {
    path: PathBuf,
    directory: File,
}

impl Sandbox {
    fn new() -> Self {
        let id = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "ios-files-create-directories-test-{}-{id}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        let directory = File::open(&path).unwrap();
        Self { path, directory }
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn parts(names: &[&str]) -> Vec<CString> {
    names
        .iter()
        .map(|name| CString::new(*name).unwrap())
        .collect()
}

fn create_directories(root: &File, names: &[&str]) -> io::Result<()> {
    create_directories::create_directories_at(root, &parts(names), |_, _| Ok(()), |error| error)
}

#[test]
fn nested_creation_is_idempotent_and_accepts_existing_directory_components() {
    let sandbox = Sandbox::new();

    create_directories(&sandbox.directory, &["logs", "2026", "10"]).unwrap();
    create_directories(&sandbox.directory, &["logs", "2026", "10"]).unwrap();
    create_directories(&sandbox.directory, &["logs", "2026", "11"]).unwrap();

    assert!(sandbox.path.join("logs/2026/10").is_dir());
    assert!(sandbox.path.join("logs/2026/11").is_dir());
}

#[test]
fn existing_file_components_fail_without_replacing_the_file() {
    let sandbox = Sandbox::new();
    let file = sandbox.path.join("entry");
    fs::write(&file, b"keep").unwrap();

    assert!(create_directories(&sandbox.directory, &["entry", "child"]).is_err());
    assert!(create_directories(&sandbox.directory, &["entry"]).is_err());
    assert_eq!(fs::read(file).unwrap(), b"keep");
}

#[test]
fn symlink_component_cannot_escape_the_open_root() {
    let sandbox = Sandbox::new();
    let outside = sandbox.path.with_extension("outside");
    fs::create_dir(&outside).unwrap();
    symlink(&outside, sandbox.path.join("redirect")).unwrap();

    assert!(create_directories(&sandbox.directory, &["redirect", "created"]).is_err());
    assert!(!outside.join("created").exists());

    fs::remove_dir_all(outside).unwrap();
}
