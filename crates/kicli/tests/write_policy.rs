//! What kicli refuses to write, and what it reports when it writes.

use kicli::model::{FormatVersion, WriteOptions, WriteRefusal, plan_write};
use kicli_sexpr::Doc;
use std::path::{Path, PathBuf};

fn sexpr_fixture(relative: &str) -> String {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../kicli-sexpr/tests/fixtures")
        .join(relative);
    std::fs::read_to_string(path).expect("fixture is readable")
}

fn own_fixture(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(relative);
    std::fs::read_to_string(path).expect("fixture is readable")
}

#[test]
fn write_reports_reformatting_and_refuses_loss() {
    // A file KiCad wrote goes out unchanged, and says so.
    let canonical = sexpr_fixture("sch/all_items.kicad_sch");
    let doc = Doc::parse(&canonical).expect("parses");
    let plan = plan_write(&doc, WriteOptions::default()).expect("writes");
    assert!(!plan.reformatted);
    assert_eq!(plan.bytes, canonical);

    // A file nobody laid out this way is written in KiCad's layout, and says so.
    let awkward = sexpr_fixture("sch/noncanonical.kicad_sch");
    let doc = Doc::parse(&awkward).expect("parses");
    let plan = plan_write(&doc, WriteOptions::default()).expect("writes");
    assert!(plan.reformatted);
    assert!(plan.reason.is_some());
    assert_ne!(plan.bytes, awkward);
    // Reformatting is a layout change, not a content change.
    let before = Doc::parse(&awkward).expect("parses");
    let after = Doc::parse(&plan.bytes).expect("parses");
    assert!(before.structurally_eq(&after));

    // Comments are refused by default, because writing drops them.
    let commented = sexpr_fixture("sch/commented.kicad_sch");
    let doc = Doc::parse(&commented).expect("parses");
    assert!(matches!(
        plan_write(&doc, WriteOptions::default()),
        Err(WriteRefusal::WouldDropComments { .. })
    ));

    let permitted = WriteOptions {
        allow_comment_loss: true,
        ..WriteOptions::default()
    };
    let plan = plan_write(&doc, permitted).expect("writes when allowed");
    assert!(!plan.bytes.contains('#'), "the comments are gone");

    // A stamp newer than kicli knows is refused, and the ceiling is a knob.
    let future = own_fixture("sch/future_version.kicad_sch");
    let doc = Doc::parse(&future).expect("parses");
    assert!(matches!(
        plan_write(&doc, WriteOptions::default()),
        Err(WriteRefusal::VersionTooNew {
            found: 20_260_803,
            ..
        })
    ));

    let raised = WriteOptions {
        max_version: FormatVersion::new(20_260_803),
        ..WriteOptions::default()
    };
    assert!(plan_write(&doc, raised).is_ok());
}
