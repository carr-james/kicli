//! The router is integer arithmetic, and that is checked rather than intended.
//!
//! Two runs over one sheet must produce the same route, on any machine,
//! forever. A float in the search would make that a hope: the same expression
//! can round two ways on two targets, and a cost that differs in the last bit
//! picks a different route. Clippy has no lint for "this module holds no
//! floating point", so the sweep is the enforcement.
//!
//! The rule is about the router's own arithmetic. Millimetres at the command
//! boundary are a presentation unit and live elsewhere.

use std::path::{Path, PathBuf};

/// The types the router may not name.
const FORBIDDEN: [&str; 2] = ["f32", "f64"];

/// Every source file of the router.
fn router_sources() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/route");
    let mut found = vec![root.with_extension("rs")];
    let entries = std::fs::read_dir(&root).expect("the router has a directory");
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|end| end == "rs") {
            found.push(path);
        }
    }
    found.sort();
    found
}

/// Is this a whole word of the source, rather than part of a longer name?
fn names_type(text: &str, wanted: &str) -> bool {
    text.match_indices(wanted).any(|(start, _)| {
        let before = text[..start].chars().next_back();
        let after = text[start + wanted.len()..].chars().next();
        let part_of_a_name =
            |letter: Option<char>| letter.is_some_and(|c| c.is_alphanumeric() || c == '_');
        !part_of_a_name(before) && !part_of_a_name(after)
    })
}

#[test]
fn no_floating_point_appears_under_the_router() {
    let sources = router_sources();
    assert!(sources.len() > 3, "the router's sources were found");

    let mut offenders = Vec::new();
    for source in &sources {
        let text = std::fs::read_to_string(source).expect("a router source reads");
        for forbidden in FORBIDDEN {
            if names_type(&text, forbidden) {
                offenders.push(format!(
                    "{}: {forbidden}",
                    source.file_name().unwrap_or_default().to_string_lossy()
                ));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "the router's arithmetic must be exact: {offenders:?}"
    );
}

#[test]
fn the_sweep_can_see_what_it_is_looking_for() {
    // The control. A sweep that read nothing, or that matched nothing it was
    // given, would pass the check above while a float sat in the search.
    let sources = router_sources();
    let text: String = sources
        .iter()
        .map(|source| std::fs::read_to_string(source).expect("a router source reads"))
        .collect();
    assert!(!text.is_empty(), "the router's sources were read");
    assert!(
        names_type(&text, "i64"),
        "the router costs in i64, so the sweep is reading the router"
    );

    // And the matcher itself: it finds a type where one is named, and is not
    // fooled by a name that merely contains the letters.
    assert!(names_type("let x: f64 = 1.0;", "f64"));
    assert!(names_type("(value as f32)", "f32"));
    assert!(!names_type("let f64s_are_out = 1;", "f64"));
    assert!(!names_type("self.if64", "f64"));
}
