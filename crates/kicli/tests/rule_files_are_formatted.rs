//! Rule files are formatted, and `cargo fmt` cannot see them.
//!
//! `rustfmt` walks the module tree from the crate root. A rule file is reached
//! through a `#[path]` declaration inside a file the build script generated and
//! `include!`s, and `rustfmt` does not follow an `include!`. So `cargo fmt
//! --check` passes over a rule file whatever state it is in. Measured, not
//! assumed: a deliberately mangled rule file leaves `cargo fmt --check` at exit
//! zero.
//!
//! That is the price the registration seam pays, and this check is the refund.
//! It hands the same files to `rustfmt` directly. `rustfmt` is a pinned
//! component of this project's toolchain, so its absence is a broken
//! environment rather than a reason to skip.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The directories a generated module list hides from `cargo fmt`.
///
/// Both are read by the build script. Anything it registers, `cargo fmt`
/// cannot see, so anything it registers belongs here.
const HIDDEN: [&str; 2] = ["src/lint/rules", "tests/specimen_rules"];

/// Every `.rs` file in the directories `cargo fmt` cannot reach.
fn hidden_sources() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut found = Vec::new();
    for directory in HIDDEN {
        let Ok(entries) = std::fs::read_dir(root.join(directory)) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|end| end == "rs") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

#[test]
fn every_hidden_rule_file_is_formatted() {
    let sources = hidden_sources();

    // The control. An empty list is formatted in the way an empty page is
    // spelled correctly, and the specimen rules are what stop that.
    assert!(
        sources.len() >= 3,
        "there are hidden rule files to check: {sources:?}"
    );

    let output = Command::new("rustfmt")
        .args(["--check", "--edition", "2024"])
        .args(&sources)
        .output()
        .expect("rustfmt is a component of this project's toolchain");
    let complaint = String::from_utf8_lossy(&output.stdout);
    let reason = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "rustfmt would change a rule file:\n{complaint}{reason}"
    );
}
