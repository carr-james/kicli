//! `sch pins` is read-only, and this is the check that says so.
//!
//! **Nothing in this repository checked that a read-only command writes
//! nothing.** The claim was made in module documentation and nowhere else, and
//! a claim in a comment is a claim nobody re-checks. `sch pins` is asked before
//! every edit — that is the whole point of it — so a write path growing into it
//! would touch a caller's file on a command they ran to look.
//!
//! The measurement is the directory, not the command. Every file under the
//! project is read before and after, by name and by bytes, and the two lists
//! must be equal: a rewritten file fails, a file whose bytes came back
//! identical after a rewrite still fails on the sibling `@last-write` state
//! that every command in this crate which writes leaves behind, and a new file
//! of any name fails on the name list.

use kicli_probe::scratch::Fixtures;
use std::collections::BTreeMap;
use std::path::Path;
use std::process::{Command, Output};

/// A path no `kicad-cli` is at.
const NO_KICAD_CLI: &str = "/nonexistent/kicad-cli";

fn fixtures() -> Fixtures {
    Fixtures::new(env!("CARGO_TARGET_TMPDIR"), env!("CARGO_MANIFEST_DIR"))
}

fn kicli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_kicli"))
        .args(args)
        .env("KICLI_KICAD_CLI", NO_KICAD_CLI)
        .output()
        .expect("the binary runs")
}

/// Every file under a directory, by path relative to it, with its bytes.
///
/// Sub-directories are walked, because the state a mutation leaves behind is
/// written into one.
fn files_under(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut found = BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if let Ok(bytes) = std::fs::read(&path) {
                let name = path
                    .strip_prefix(root)
                    .unwrap_or(&path)
                    .display()
                    .to_string();
                found.insert(name, bytes);
            }
        }
    }
    found
}

/// Every form of the command, run over one project.
const EVERY_FORM: [&[&str]; 5] = [
    &["sch", "pins", "R1"],
    &["sch", "pins", "R1.1"],
    &["sch", "pins", "R20", "--free"],
    &["sch", "pins", "R20", "--stats"],
    &["sch", "pins", "R20", "--output", "json"],
];

#[test]
fn asking_where_the_pins_are_leaves_the_project_byte_identical() {
    let project = fixtures().scratch_directory("pin_view_read_only", "sch/nets");
    let path = project.to_str().expect("the path is text");
    let before = files_under(&project);
    assert!(
        before.len() >= 2,
        "the project has files to compare: {before:?}"
    );

    for form in EVERY_FORM {
        let mut arguments = form.to_vec();
        arguments.extend(["-p", path, "--quiet"]);
        let run = kicli(&arguments);
        assert_eq!(
            run.status.code(),
            Some(0),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert!(
            !run.stdout.is_empty(),
            "{arguments:?} answered something, so the run reached the command"
        );

        let after = files_under(&project);
        assert_eq!(
            after.keys().collect::<Vec<_>>(),
            before.keys().collect::<Vec<_>>(),
            "{arguments:?} left no file behind"
        );
        for (name, bytes) in &before {
            assert_eq!(
                after.get(name).map(Vec::len),
                Some(bytes.len()),
                "{arguments:?} did not resize {name}"
            );
            assert_eq!(
                after.get(name),
                Some(bytes),
                "{arguments:?} did not rewrite {name}"
            );
        }
    }
}

#[test]
fn the_check_above_would_notice_a_command_that_writes() {
    // The control. A command of this crate that *does* write, run over the same
    // project by the same walk, must fail the same comparison — otherwise the
    // check above would pass on a `sch pins` that rewrote every file.
    let project = fixtures().scratch_directory("pin_view_read_only_control", "sch/nets");
    let path = project.to_str().expect("the path is text");
    let before = files_under(&project);

    let run = kicli(&[
        "wire",
        "draw",
        "--from-pin",
        "R20.2",
        "--to-at",
        "52.07,107.95",
        "-p",
        path,
        "--quiet",
    ]);
    assert_eq!(
        run.status.code(),
        Some(0),
        "the control wrote: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    let after = files_under(&project);
    assert_ne!(after, before, "a command that writes changes the directory");
    // Named rather than left to the whole-map comparison: a write that only
    // added the `@last-write` state beside the file would satisfy `assert_ne`
    // without proving the byte comparison itself can fire.
    assert_ne!(
        after.get("nets.kicad_sch"),
        before.get("nets.kicad_sch"),
        "the root sheet's own bytes changed, so the byte comparison fires"
    );
}
