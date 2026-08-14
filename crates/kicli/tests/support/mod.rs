//! The scratch copy every test that writes needs.
//!
//! A test mutates a copy of a fixture, never the committed tree. `cargo xtask
//! check` compares the working tree before and after the run and fails when they
//! differ, so a test that writes outside `target/` breaks the build by
//! construction. This module is the other half: one home for the six lines that
//! make the copy, rather than one copy of them per test file.
//!
//! `CARGO_TARGET_TMPDIR` is under `target/`, so everything made here is scratch
//! by construction and a stale directory is removed by `cargo clean`.

// Each test binary uses the two or three of these its own fixtures need. A
// helper no single binary calls is still the shared one, so the unused warning
// says nothing true about this module.
#![allow(dead_code, reason = "each test binary uses its own subset")]

use std::path::{Path, PathBuf};

/// The committed fixture at a path relative to `tests/fixtures`.
pub fn fixture(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(relative)
}

/// An empty scratch directory of its own for one test.
///
/// An earlier run's directory is removed first, so a test never reads what the
/// last run left.
pub fn scratch(name: &str) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("the scratch directory is made");
    directory
}

/// Copy one committed fixture file into a directory, and name the copy.
pub fn copy_file(into: &Path, relative: &str) -> PathBuf {
    let from = fixture(relative);
    let name = from.file_name().expect("a fixture file has a name");
    let to = into.join(name);
    std::fs::copy(&from, &to).expect("the fixture copies");
    to
}

/// Copy every file of a committed fixture directory into a directory.
///
/// Sub-directories are left behind. A fixture project is one directory of
/// files, and a test that needs more says so by naming them.
pub fn copy_directory(into: &Path, relative: &str) {
    for entry in std::fs::read_dir(fixture(relative)).expect("the fixture directory reads") {
        let path = entry.expect("a directory entry reads").path();
        if path.is_file() {
            let name = path.file_name().expect("a file has a name");
            std::fs::copy(&path, into.join(name)).expect("the copy is written");
        }
    }
}

/// A scratch directory holding one copy of one fixture file, and the copy.
pub fn scratch_file(name: &str, relative: &str) -> PathBuf {
    let directory = scratch(name);
    copy_file(&directory, relative)
}

/// A scratch directory holding a copy of a whole fixture project.
pub fn scratch_directory(name: &str, relative: &str) -> PathBuf {
    let directory = scratch(name);
    copy_directory(&directory, relative);
    directory
}
