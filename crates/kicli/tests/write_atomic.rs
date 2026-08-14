//! A write lands whole, or it does not land at all.

use kicli::model::{Sink, WriteError, WriteOptions, write_document, write_document_with};
use kicli_sexpr::Doc;

use std::path::{Path, PathBuf};

use kicli_probe::scratch::Fixtures;

/// The committed fixtures this binary reads, and the scratch it writes in.
fn fixtures() -> Fixtures {
    Fixtures::new(env!("CARGO_TARGET_TMPDIR"), env!("CARGO_MANIFEST_DIR"))
}

const SHEET: &str = "(kicad_sch\n\t(version 20260306)\n\t(paper \"A4\")\n)\n";

fn temporary_of(target: &Path) -> PathBuf {
    let name = target
        .file_name()
        .expect("a file has a name")
        .to_string_lossy();
    target.with_file_name(format!(".{name}.kicli-tmp"))
}

/// A sink that writes half of what it is given.
struct Truncating;

impl Sink for Truncating {
    fn write(&self, path: &Path, bytes: &[u8]) -> Result<(), String> {
        let text = String::from_utf8_lossy(bytes);
        std::fs::write(path, &text[..text.len() / 2]).map_err(|error| error.to_string())
    }
    fn rename(&self, from: &Path, to: &Path) -> Result<(), String> {
        std::fs::rename(from, to).map_err(|error| error.to_string())
    }
    fn discard(&self, path: &Path) {
        let _ = std::fs::remove_file(path);
    }
}

/// A sink that cannot write at all.
struct Refusing;

impl Sink for Refusing {
    fn write(&self, _path: &Path, _bytes: &[u8]) -> Result<(), String> {
        Err("the disk said no".to_owned())
    }
    fn rename(&self, _from: &Path, _to: &Path) -> Result<(), String> {
        Err("the disk said no".to_owned())
    }
    fn discard(&self, _path: &Path) {}
}

#[test]
fn atomic_write_leaves_the_original_alone() {
    let directory = fixtures().scratch("atomic_write");
    let target = directory.join("board.kicad_sch");
    std::fs::write(&target, SHEET).expect("the file is written");

    // A write that cannot start changes nothing.
    let doc = Doc::parse(SHEET).expect("parses");
    let refused = write_document_with(&doc, &target, WriteOptions::default(), &Refusing);
    assert!(matches!(refused, Err(WriteError::Unwritable { .. })));
    assert_eq!(std::fs::read_to_string(&target).expect("reads"), SHEET);

    // A successful write replaces the file and leaves no temporary.
    let mut edited = Doc::parse(SHEET).expect("parses");
    let root = edited.root().expect("root");
    let added = edited.add_fragment("(uuid \"1234\")").expect("parses");
    edited.push_child(root, added);
    let written = write_document(&edited, &target, WriteOptions::default()).expect("writes");
    assert!(!written.reformatted, "a canonical file is not reformatted");
    let now = std::fs::read_to_string(&target).expect("reads");
    assert!(now.contains("(uuid \"1234\")"), "{now}");
    assert_eq!(written.bytes, now.len());
    assert!(
        !temporary_of(&target).exists(),
        "no temporary is left behind"
    );
}

#[test]
fn a_write_verifies_the_bytes_and_not_the_tree() {
    let directory = fixtures().scratch("verify_bytes");
    let target = directory.join("board.kicad_sch");
    std::fs::write(&target, SHEET).expect("the file is written");

    let doc = Doc::parse(SHEET).expect("parses");
    let result = write_document_with(&doc, &target, WriteOptions::default(), &Truncating);

    match result {
        Err(WriteError::Unverified { reason, .. }) => {
            assert!(!reason.is_empty(), "the failure says which check failed");
        }
        other => panic!("half a file must not verify: {other:?}"),
    }
    assert_eq!(
        std::fs::read_to_string(&target).expect("reads"),
        SHEET,
        "the original is untouched"
    );
    assert!(!temporary_of(&target).exists(), "and no temporary is left");
}

#[test]
fn a_file_kicli_refuses_is_a_file_kicli_does_not_touch() {
    let directory = fixtures().scratch("refusals");

    // A stamp above the ceiling.
    let future = "(kicad_sch\n\t(version 20260803)\n)\n";
    let target = directory.join("future.kicad_sch");
    std::fs::write(&target, future).expect("written");
    let doc = Doc::parse(future).expect("parses");
    assert!(matches!(
        write_document(&doc, &target, WriteOptions::default()),
        Err(WriteError::Refused(_))
    ));
    assert_eq!(std::fs::read_to_string(&target).expect("reads"), future);
    assert!(!temporary_of(&target).exists());

    // A file carrying comments, which writing would drop.
    let commented = "# a note\n(kicad_sch\n\t(version 20260306)\n)\n";
    let target = directory.join("commented.kicad_sch");
    std::fs::write(&target, commented).expect("written");
    let doc = Doc::parse(commented).expect("parses");
    assert!(matches!(
        write_document(&doc, &target, WriteOptions::default()),
        Err(WriteError::Refused(_))
    ));
    assert_eq!(std::fs::read_to_string(&target).expect("reads"), commented);

    // The same file, with the flag that accepts the loss.
    let written = write_document(
        &doc,
        &target,
        WriteOptions {
            allow_comment_loss: true,
            ..WriteOptions::default()
        },
    )
    .expect("writes with the flag");
    assert!(written.reformatted, "dropping a comment reformats the file");
    let now = std::fs::read_to_string(&target).expect("reads");
    assert!(
        !now.contains("# a note"),
        "the comment is gone, as KiCad drops it"
    );
}

#[test]
fn a_reformatted_file_says_so_and_why() {
    let directory = fixtures().scratch("reformat");
    let target = directory.join("loose.kicad_sch");
    // Hand-indented, which KiCad would lay out again on its next save.
    let loose = "(kicad_sch\n  (version 20260306)\n  (paper \"A4\"))\n";
    std::fs::write(&target, loose).expect("written");

    let doc = Doc::parse(loose).expect("parses");
    let written = write_document(&doc, &target, WriteOptions::default()).expect("writes");
    assert!(written.reformatted);
    assert!(written.reason.is_some(), "and says why");
    assert_ne!(std::fs::read_to_string(&target).expect("reads"), loose);
}
