//! The committed fixtures a test reads, and the scratch copy it writes.
//!
//! A test mutates a copy of a fixture, never the committed tree. `cargo xtask
//! check` compares the working tree before and after the run and fails when
//! they differ, so a test that writes outside `target/` breaks the build by
//! construction. This module is the other half: one home for the lines that
//! make the copy.
//!
//! Both roots come from the caller. `CARGO_TARGET_TMPDIR` and
//! `CARGO_MANIFEST_DIR` are read at compile time in the test binary that owns
//! them; a library sees its own, which is the wrong crate.

use std::path::{Path, PathBuf};

/// Where a test reads its fixtures, and where it writes its copies.
pub struct Fixtures {
    /// A directory under `target/`, so everything made here is scratch by
    /// construction and `cargo clean` removes it.
    scratch: PathBuf,
    /// The committed fixture tree, which no test writes.
    committed: PathBuf,
}

impl Fixtures {
    /// The fixtures of a crate whose tests keep them in `tests/fixtures`.
    ///
    /// Pass `env!("CARGO_TARGET_TMPDIR")` and `env!("CARGO_MANIFEST_DIR")` from
    /// the test binary.
    #[must_use]
    pub fn new(scratch: &str, manifest_directory: &str) -> Self {
        Self {
            scratch: PathBuf::from(scratch),
            committed: Path::new(manifest_directory).join("tests/fixtures"),
        }
    }

    /// The committed fixture at a path relative to the fixture tree.
    #[must_use]
    pub fn fixture(&self, relative: &str) -> PathBuf {
        self.committed.join(relative)
    }

    /// An empty scratch directory of its own for one test.
    ///
    /// An earlier run's directory is removed first, so a test never reads what
    /// the last run left.
    ///
    /// # Panics
    ///
    /// If the directory cannot be made.
    #[must_use]
    pub fn scratch(&self, name: &str) -> PathBuf {
        let directory = self.scratch.join(name);
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("the scratch directory is made");
        directory
    }

    /// Copy one committed fixture file into a directory, and name the copy.
    ///
    /// # Panics
    ///
    /// If the fixture does not copy.
    #[must_use]
    pub fn copy_file(&self, into: &Path, relative: &str) -> PathBuf {
        let from = self.fixture(relative);
        let name = from.file_name().expect("a fixture file has a name");
        let to = into.join(name);
        std::fs::copy(&from, &to).expect("the fixture copies");
        to
    }

    /// Copy every file of a committed fixture directory into a directory.
    ///
    /// Sub-directories are left behind. A fixture project is one directory of
    /// files, and a test that needs more says so by naming them.
    ///
    /// # Panics
    ///
    /// If the directory does not read, or a file does not copy.
    pub fn copy_directory(&self, into: &Path, relative: &str) {
        std::fs::create_dir_all(into).expect("the scratch directory is made");
        for entry in std::fs::read_dir(self.fixture(relative)).expect("the fixture directory reads")
        {
            let path = entry.expect("a directory entry reads").path();
            if path.is_file() {
                let name = path.file_name().expect("a file has a name");
                std::fs::copy(&path, into.join(name)).expect("the copy is written");
            }
        }
    }

    /// A scratch directory holding one copy of one fixture file, and the copy.
    #[must_use]
    pub fn scratch_file(&self, name: &str, relative: &str) -> PathBuf {
        let directory = self.scratch(name);
        self.copy_file(&directory, relative)
    }

    /// Copy the project a root sits in into scratch, and name the root there.
    ///
    /// The project may be anywhere, so this is what a sweep over an external
    /// corpus uses. KiCad writes a `.kicad_prl` beside any project it opens,
    /// so the tool runs on the copy and the original is left as it was.
    ///
    /// # Panics
    ///
    /// If the directory does not read, or a file does not copy.
    #[must_use]
    pub fn copy_project(&self, name: &str, root: &Path) -> PathBuf {
        let into = self.scratch(name);
        let from = root.parent().unwrap_or(Path::new("."));
        for entry in std::fs::read_dir(from).expect("the project directory reads") {
            let path = entry.expect("a directory entry reads").path();
            if path.is_file() {
                let file = path.file_name().expect("a file has a name");
                std::fs::copy(&path, into.join(file)).expect("the copy is written");
            }
        }
        into.join(root.file_name().expect("the root has a name"))
    }

    /// A scratch directory holding a copy of a whole fixture project.
    ///
    /// KiCad writes a `.kicad_prl` beside any project it opens, so a test that
    /// hands a project to the tool hands it a copy.
    #[must_use]
    pub fn scratch_directory(&self, name: &str, relative: &str) -> PathBuf {
        let directory = self.scratch(name);
        self.copy_directory(&directory, relative);
        directory
    }
}
