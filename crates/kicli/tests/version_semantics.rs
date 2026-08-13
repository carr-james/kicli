//! A file's version stamp decides what its tokens mean.

use kicli::model::{FormatVersion, PropertyOrder, format_version, pin_text};
use kicli_sexpr::Doc;
use std::path::Path;

fn own_fixture(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(relative);
    std::fs::read_to_string(path).expect("fixture is readable")
}

#[test]
fn version_stamp_selects_token_semantics() {
    let legacy = own_fixture("sch/v9_legacy.kicad_sch");
    let doc = Doc::parse(&legacy).expect("parses");

    let version = format_version(&doc).expect("the file carries a stamp");
    assert_eq!(version, FormatVersion::new(20_250_114));
    assert!(version.tilde_means_empty());
    assert!(version.hide_lives_in_effects());

    // The token is preserved exactly, and only its reading changes.
    assert!(
        legacy.contains("(name \"~\""),
        "the tilde is still in the file"
    );
    assert_eq!(pin_text("~", version), "");
    assert_eq!(pin_text("~", FormatVersion::new(20_260_306)), "~");

    // hide sits inside effects in this file, where a v10 file puts it beside
    // show_name instead.
    assert!(legacy.contains("(effects\n\t\t\t\t\t(font\n"));
    let hide_at = legacy.find("(hide yes)").expect("the file hides a field");
    let effects_at = legacy[..hide_at]
        .rfind("(effects")
        .expect("hide follows an effects list");
    assert!(
        !legacy[effects_at..hide_at].contains(')')
            || legacy[effects_at..hide_at].matches('(').count()
                > legacy[effects_at..hide_at].matches(')').count(),
        "hide is still inside the effects list"
    );

    // And the file comes back byte for byte regardless.
    assert_eq!(doc.emit(), legacy);

    let modern = own_fixture("sch/lib_name_redirect.kicad_sch");
    let doc = Doc::parse(&modern).expect("parses");
    let version = format_version(&doc).expect("has a stamp");
    assert!(!version.tilde_means_empty());
    assert!(!version.hide_lives_in_effects());
    assert!(modern.contains("(hide yes)"));
    assert_eq!(doc.emit(), modern);
}

#[test]
fn the_two_property_orderings_stay_apart() {
    let instance = PropertyOrder::Instance.tokens();
    let library = PropertyOrder::Library.tokens();

    let position = |order: &[&str], token: &str| {
        order
            .iter()
            .position(|t| *t == token)
            .expect("token is listed")
    };

    assert!(position(instance, "hide") < position(instance, "show_name"));
    assert!(position(library, "hide") > position(library, "show_name"));
}
