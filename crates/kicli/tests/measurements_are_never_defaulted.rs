//! No measurement read is allowed to fall back to a default.
//!
//! `Doc::atom_as_iu` answers `None` both for an absent value and for one kicli
//! cannot represent. Reaching for a default there is how a rejected coordinate
//! became a zero, which moved items to the origin and produced a confident net
//! list about a drawing nobody drew.
//!
//! `Doc::check_measurements` refuses such a file at the boundary, so the bug
//! cannot recur through the load path. This test holds the other half: that no
//! new code quietly defaults a measurement anyway. Clippy has no lint for it —
//! `map_unwrap_or` and its family are about style, not about what a `None`
//! means — so the rule is executable here instead.

use std::path::{Path, PathBuf};

fn sources(root: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            sources(&path, found);
        } else if path.extension().is_some_and(|end| end == "rs") {
            found.push(path);
        }
    }
}

#[test]
fn no_measurement_read_falls_back_to_a_default() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the workspace root resolves");
    let mut files = Vec::new();
    for crate_name in [
        "crates/kicli/src",
        "crates/kicli-sexpr/src",
        "crates/kicli-probe/src",
        "xtask/src",
    ] {
        sources(&workspace.join(crate_name), &mut files);
    }
    files.sort();
    assert!(!files.is_empty(), "no sources were found to check");

    let mut offenders = Vec::new();
    for file in &files {
        let text = std::fs::read_to_string(file).expect("a source file reads");
        // The call and its fallback may sit on different lines, so the check is
        // over the whole text with whitespace squeezed out.
        let squeezed: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
        for pattern in ["atom_as_iu(", "atom_as_iu_checked("] {
            let mut from = 0;
            while let Some(at) = squeezed[from..].find(pattern) {
                let start = from + at;
                let tail = &squeezed[start..(start + 160).min(squeezed.len())];
                if tail.contains("unwrap_or_default()") || tail.contains("unwrap_or(") {
                    offenders.push(format!(
                        "{}: {tail}",
                        file.strip_prefix(&workspace).unwrap_or(file).display()
                    ));
                }
                from = start + pattern.len();
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "a measurement read falls back to a default. Propagate the refusal, or \
         read the value with a name that says an absent one is meant:\n{}",
        offenders.join("\n")
    );
}
