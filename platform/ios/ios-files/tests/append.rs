#[path = "../src/append.rs"]
mod append;

use std::{
    ffi::CString,
    fs::{self, File},
    os::unix::fs::{PermissionsExt, symlink},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    thread,
};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(1);

struct Sandbox {
    path: PathBuf,
    directory: File,
}

impl Sandbox {
    fn new() -> Self {
        let id = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("ios-files-append-test-{}-{id}", std::process::id()));
        fs::create_dir(&path).unwrap();
        let directory = File::open(&path).unwrap();
        Self { path, directory }
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
fn append_creates_private_file_and_preserves_binary_bytes() {
    let sandbox = Sandbox::new();
    let leaf = Sandbox::leaf("app.bin");

    append::append_at(&sandbox.directory, &leaf, &[0x00, 0xff]).unwrap();
    append::append_at(&sandbox.directory, &leaf, &[0x80, 0x00]).unwrap();

    assert_eq!(
        fs::read(sandbox.path.join("app.bin")).unwrap(),
        [0x00, 0xff, 0x80, 0x00]
    );
    let mode = fs::metadata(sandbox.path.join("app.bin"))
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode & !0o600, 0);
}

#[test]
fn empty_append_does_not_create_a_file() {
    let sandbox = Sandbox::new();
    let leaf = Sandbox::leaf("empty.bin");

    append::append_at(&sandbox.directory, &leaf, &[]).unwrap();

    assert!(!sandbox.path.join("empty.bin").exists());
}

#[test]
fn existing_file_mode_is_preserved() {
    let sandbox = Sandbox::new();
    let path = sandbox.path.join("existing.bin");
    let leaf = Sandbox::leaf("existing.bin");
    fs::write(&path, [0x01, 0x02]).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();

    append::append_at(&sandbox.directory, &leaf, &[0x03]).unwrap();

    assert_eq!(fs::read(&path).unwrap(), [0x01, 0x02, 0x03]);
    assert_eq!(
        fs::metadata(path).unwrap().permissions().mode() & 0o777,
        0o640
    );
}

#[test]
fn final_symlink_is_rejected_without_modifying_its_target() {
    let sandbox = Sandbox::new();
    let target = sandbox.path.join("target.bin");
    fs::write(&target, b"keep").unwrap();
    symlink(&target, sandbox.path.join("link.bin")).unwrap();
    let leaf = Sandbox::leaf("link.bin");

    let error = append::append_at(&sandbox.directory, &leaf, b"reject").unwrap_err();

    assert_eq!(error.raw_os_error(), Some(libc::ELOOP));
    assert_eq!(fs::read(target).unwrap(), b"keep");
}

#[test]
fn final_directory_is_rejected() {
    let sandbox = Sandbox::new();
    fs::create_dir(sandbox.path.join("child")).unwrap();
    let leaf = Sandbox::leaf("child");

    assert!(append::append_at(&sandbox.directory, &leaf, b"reject").is_err());
    assert_eq!(fs::read_dir(sandbox.path.join("child")).unwrap().count(), 0);
}

#[test]
fn concurrent_append_calls_keep_each_successful_slice_contiguous() {
    const WRITERS: u8 = 4;
    const APPENDS_PER_WRITER: usize = 32;
    const RECORD_SIZE: usize = 64;
    let sandbox = Sandbox::new();
    let mut workers = Vec::new();

    for value in 0..WRITERS {
        let directory = sandbox.directory.try_clone().unwrap();
        let leaf = Sandbox::leaf("concurrent.bin");
        workers.push(thread::spawn(move || {
            let record = [value; RECORD_SIZE];
            for _ in 0..APPENDS_PER_WRITER {
                append::append_at(&directory, &leaf, &record).unwrap();
            }
        }));
    }
    for worker in workers {
        worker.join().unwrap();
    }

    let bytes = fs::read(sandbox.path.join("concurrent.bin")).unwrap();
    assert_eq!(
        bytes.len(),
        WRITERS as usize * APPENDS_PER_WRITER * RECORD_SIZE
    );
    let mut counts = [0usize; WRITERS as usize];
    for record in bytes.chunks_exact(RECORD_SIZE) {
        let value = record[0];
        assert!(record.iter().all(|byte| *byte == value));
        counts[value as usize] += 1;
    }
    assert_eq!(counts, [APPENDS_PER_WRITER; WRITERS as usize]);
}
