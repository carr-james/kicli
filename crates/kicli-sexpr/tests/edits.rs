//! Adding and removing objects keeps both round-trip properties.
//!
//! An edit that changes one object must leave every other byte of the file
//! alone. That is what makes a mutation reviewable in a diff, and it is the
//! property most likely to regress without anyone noticing.

use kicli_sexpr::{Doc, changed_line_count};
use std::path::{Path, PathBuf};

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Every schematic fixture this crate owns, with its bytes.
fn schematics() -> Vec<(String, String)> {
    let root = fixture_root();
    let manifest = std::fs::read_to_string(root.join("MANIFEST")).expect("manifest reads");
    manifest
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| line.split_whitespace().next())
        .filter(|path| path.ends_with(".kicad_sch") || path.ends_with(".kicad_sym"))
        .map(|path| {
            let text = std::fs::read_to_string(root.join(path)).expect("fixture reads");
            (path.to_owned(), text)
        })
        .collect()
}

const JUNCTION: &str = r#"(junction (at 25.4 25.4) (diameter 0) (uuid "added"))"#;

#[test]
fn edits_keep_the_round_trip_properties() {
    for (name, source) in schematics() {
        let mut doc = Doc::parse(&source).expect("fixture parses");
        let root = doc.root().expect("a file has a root list");
        let before = doc.emit();

        // Add an object, then take it away again. The file comes back.
        let added = doc.add_fragment(JUNCTION).expect("the fragment parses");
        doc.push_child(root, added);
        let with_extra = doc.emit();
        assert!(
            with_extra.contains("(junction"),
            "{name}: the added object is written"
        );
        assert!(with_extra.len() > before.len(), "{name}: and the file grew");

        assert!(doc.remove(added), "{name}: the added object is found again");
        assert_eq!(
            doc.emit(),
            before,
            "{name}: removing what was added gives the file back, byte for byte"
        );

        // An edited document still re-parses to the same tree.
        let added = doc.add_fragment(JUNCTION).expect("the fragment parses");
        doc.push_child(root, added);
        let emitted = doc.emit();
        let reparsed = Doc::parse(&emitted).expect("the edited file parses");
        assert!(
            doc.structurally_eq(&reparsed),
            "{name}: the tree survives a write and a read"
        );
        assert_eq!(
            reparsed.emit(),
            emitted,
            "{name}: and the file is a fixed point after the edit"
        );
    }
}

#[test]
fn an_edit_touches_only_its_own_lines() {
    let source = std::fs::read_to_string(fixture_root().join("sch/all_items.kicad_sch"))
        .expect("fixture reads");

    // One atom.
    let mut doc = Doc::parse(&source).expect("parses");
    let before = doc.emit();
    let root = doc.root().expect("root");
    let target = doc
        .children(root)
        .iter()
        .copied()
        .find(|&child| doc.head_is(child, "junction"))
        .and_then(|junction| {
            doc.children(junction)
                .iter()
                .copied()
                .find(|&child| doc.head_is(child, "at"))
        })
        .expect("the fixture has a junction with a position");
    let coordinate = doc.children(target)[1];
    doc.set_atom(coordinate, "99.06");
    assert!(
        changed_line_count(&before, &doc.emit()) <= 3,
        "one coordinate is a one-region change"
    );

    // One inserted item: the lines it occupies, and no others.
    let mut doc = Doc::parse(&source).expect("parses");
    let root = doc.root().expect("root");
    let added = doc.add_fragment(JUNCTION).expect("parses");
    doc.push_child(root, added);
    let after = doc.emit();
    let inserted = after.lines().count() - before.lines().count();
    assert_eq!(
        changed_line_count(&before, &after),
        inserted,
        "an insert changes exactly the lines it adds"
    );
}

#[test]
fn a_removed_node_is_not_a_token_any_more() {
    let source = "(kicad_sch\n\t(version 20260306)\n\t(paper \"A4\")\n)\n";
    let mut doc = Doc::parse(source).expect("parses");
    let before = doc.token_count();
    let root = doc.root().expect("root");

    let added = doc.add_fragment("(uuid \"1234\")").expect("parses");
    doc.push_child(root, added);
    assert_eq!(
        doc.token_count(),
        before + 4,
        "two parentheses and two atoms"
    );

    doc.remove(added);
    assert_eq!(
        doc.token_count(),
        before,
        "a node out of the tree is not a token of the file"
    );
}

#[test]
fn a_handle_taken_before_an_edit_still_works_after_it() {
    let mut doc =
        Doc::parse("(kicad_sch\n\t(version 20260306)\n\t(paper \"A4\")\n)\n").expect("parses");
    let root = doc.root().expect("root");
    let paper = doc
        .children(root)
        .iter()
        .copied()
        .find(|&child| doc.head_is(child, "paper"))
        .expect("the paper size is there");

    let added = doc.add_fragment("(uuid \"1234\")").expect("parses");
    doc.insert_child(root, 1, added);

    // The handle names the same object after an insert before it.
    assert!(doc.head_is(paper, "paper"));
    doc.set_atom(doc.children(paper)[1], "\"A3\"");
    let written = doc.emit();
    assert!(written.contains("(paper \"A3\")"), "{written}");
    assert!(written.contains("(uuid \"1234\")"), "{written}");
}

#[test]
fn a_fragment_that_is_not_one_list_is_refused() {
    let mut doc = Doc::parse("(kicad_sch)").expect("parses");
    assert!(doc.add_fragment("not a list").is_err());
    assert!(doc.add_fragment("(unclosed").is_err());
    assert!(doc.add_fragment("").is_err());
}
