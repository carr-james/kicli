//! The properties that make kicli safe to point at somebody's project.
//!
//! Every fixture named canonical in the manifest must come back byte for byte.
//! Every fixture at all must come back with the same tokens in the same shape.

use kicli_sexpr::{Doc, FormatMode, changed_line_count, flatten, lex, prettify};
use std::path::{Path, PathBuf};

/// One fixture, as the manifest describes it.
struct Fixture {
    path: PathBuf,
    name: String,
    mode: FormatMode,
    canonical: bool,
}

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Read the manifest and load every fixture it lists.
fn fixtures() -> Vec<Fixture> {
    let root = fixture_root();
    let manifest = std::fs::read_to_string(root.join("MANIFEST")).expect("manifest is readable");
    manifest
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            let mode = match fields[2] {
                "normal" => FormatMode::Normal,
                "compact" => FormatMode::CompactTextProperties,
                "library-table" => FormatMode::LibraryTable,
                other => panic!("unknown mode {other}"),
            };
            Fixture {
                path: root.join(fields[0]),
                name: fields[0].to_owned(),
                mode,
                canonical: fields[3] == "yes",
            }
        })
        .collect()
}

fn read(fixture: &Fixture) -> String {
    std::fs::read_to_string(&fixture.path).expect("fixture is readable")
}

#[test]
fn lexer_classifies_tokens() {
    for fixture in fixtures() {
        let source = read(&fixture);
        let tokens = lex(&source).expect("fixture lexes");
        assert!(!tokens.is_empty(), "{} has tokens", fixture.name);

        // Re-joining the tokens reproduces the source minus the whitespace
        // between tokens. Whitespace inside a quoted string belongs to its
        // token, so it survives.
        let joined: String = tokens.iter().map(|t| t.text(&source)).collect();
        let stripped: String = source.split_whitespace().collect();
        assert_eq!(
            joined.split_whitespace().collect::<String>(),
            stripped,
            "{} keeps every non-whitespace byte",
            fixture.name
        );
    }
}

#[test]
fn atom_spans_match_source() {
    for fixture in fixtures() {
        let source = read(&fixture);
        let doc = Doc::parse(&source).expect("fixture parses");
        let tokens = lex(&source).expect("fixture lexes");

        // Every token becomes exactly one node, and a list's two parentheses
        // become one node, so the counts line up.
        assert_eq!(
            doc.token_count(),
            tokens.len(),
            "{} has one node per token",
            fixture.name
        );

        for id in doc.node_ids() {
            if let Some(text) = doc.atom_text(id) {
                assert!(
                    source.contains(text) || text.is_empty(),
                    "{} atom text comes from the source",
                    fixture.name
                );
            }
        }
    }
}

#[test]
fn flatten_preserves_tokens() {
    for fixture in fixtures() {
        let source = read(&fixture);
        let before = lex(&source).expect("fixture lexes");
        let flattened = flatten(&source);
        let after = lex(&flattened).expect("flattened text lexes");

        assert_eq!(
            before.len(),
            after.len(),
            "{} keeps its token count through flatten",
            fixture.name
        );
        for (a, b) in before.iter().zip(&after) {
            assert_eq!(a.kind, b.kind, "{} keeps token kinds", fixture.name);
            assert_eq!(
                a.text(&source),
                b.text(&flattened),
                "{} keeps token text",
                fixture.name
            );
        }
    }
}

#[test]
fn prettify_reproduces_kicad_layout() {
    for fixture in fixtures().iter().filter(|f| f.canonical) {
        let source = read(fixture);
        assert_eq!(
            prettify(&flatten(&source), fixture.mode),
            source,
            "{} is reproduced from its tokens alone",
            fixture.name
        );
    }
}

#[test]
fn prettify_is_idempotent_for_every_fixture() {
    for fixture in fixtures() {
        let source = read(&fixture);
        let once = prettify(&source, fixture.mode);
        assert_eq!(
            prettify(&once, fixture.mode),
            once,
            "{} settles after one pass",
            fixture.name
        );
    }
}

#[test]
fn format_mode_is_detected_and_preserved() {
    let all = fixtures();
    let find = |name: &str| {
        all.iter()
            .find(|f| f.name == name)
            .unwrap_or_else(|| panic!("{name} is in the manifest"))
    };

    let compact = find("sch/compact_save.kicad_sch");
    let doc = Doc::parse(&read(compact)).expect("parses");
    assert_eq!(doc.mode(), FormatMode::CompactTextProperties);
    assert!(doc.is_canonical());
    assert_eq!(doc.emit(), read(compact));

    let table = find("tables/canonical/sym-lib-table");
    let doc = Doc::parse(&read(table)).expect("parses");
    assert_eq!(doc.mode(), FormatMode::LibraryTable);
    assert!(doc.is_canonical());
    assert_eq!(doc.emit(), read(table));

    // The legacy table predates KiCad 8 and is in no current mode. Writing it
    // reformats it, which is what KiCad's own next save would do.
    let legacy = find("tables/legacy/sym-lib-table");
    let doc = Doc::parse(&read(legacy)).expect("parses");
    assert!(!doc.is_canonical());
    assert_eq!(doc.mode(), FormatMode::LibraryTable);
}

#[test]
fn emit_without_edits_is_identity() {
    for fixture in fixtures().iter().filter(|f| f.canonical) {
        let source = read(fixture);
        let doc = Doc::parse(&source).expect("fixture parses");
        assert_eq!(
            doc.emit(),
            source,
            "{} is written back unchanged",
            fixture.name
        );
    }
}

#[test]
fn emit_reproduces_input_bytes() {
    let canonical: Vec<Fixture> = fixtures().into_iter().filter(|f| f.canonical).collect();
    assert!(
        !canonical.is_empty(),
        "there are canonical fixtures to check"
    );

    for fixture in &canonical {
        let source = read(fixture);
        let doc = Doc::parse(&source).expect("fixture parses");
        assert!(
            doc.is_canonical(),
            "{} is detected as canonical",
            fixture.name
        );
        assert_eq!(
            doc.emit(),
            source,
            "{} round-trips byte for byte",
            fixture.name
        );
    }
}

#[test]
fn reparse_preserves_tree() {
    for fixture in fixtures() {
        let source = read(&fixture);
        let first = Doc::parse(&source).expect("fixture parses");
        let written = first.emit();
        let second = Doc::parse(&written).expect("output parses");
        assert!(
            first.structurally_eq(&second),
            "{} keeps its tokens and shape",
            fixture.name
        );
    }
}

#[test]
fn edit_changes_only_its_own_lines() {
    let root = fixture_root().join("sch/all_items.kicad_sch");
    let source = std::fs::read_to_string(&root).expect("fixture is readable");

    // A property value: the change must stay on its own line.
    let mut doc = Doc::parse(&source).expect("parses");
    let value = find_property_value(&doc).expect("the sheet has a Value property");
    doc.set_atom(value, "\"47k\"");
    let edited = doc.emit();
    assert!(edited.contains("\"47k\""), "the edit lands in the output");
    assert_eq!(
        changed_line_count(&source, &edited),
        1,
        "a field value changes one line"
    );

    // A coordinate: same expectation.
    let mut doc = Doc::parse(&source).expect("parses");
    let coordinate = find_first_xy_coordinate(&doc).expect("the sheet has an xy");
    doc.set_atom(coordinate, "123.456");
    let edited = doc.emit();
    assert_eq!(
        changed_line_count(&source, &edited),
        1,
        "a coordinate changes one line"
    );
}

/// The value atom of the first `(property "Value" ...)` list.
fn find_property_value(doc: &Doc) -> Option<kicli_sexpr::NodeId> {
    doc.node_ids().find_map(|id| {
        if !doc.head_is(id, "property") {
            return None;
        }
        let children = doc.children(id);
        if doc.atom_text(*children.get(1)?)? == "\"Value\"" {
            children.get(2).copied()
        } else {
            None
        }
    })
}

/// The first coordinate of the first `(xy ...)` list.
fn find_first_xy_coordinate(doc: &Doc) -> Option<kicli_sexpr::NodeId> {
    doc.node_ids().find_map(|id| {
        if doc.head_is(id, "xy") {
            doc.children(id).get(1).copied()
        } else {
            None
        }
    })
}

#[test]
fn embedded_data_is_never_re_encoded() {
    let path = fixture_root().join("sch/image_data.kicad_sch");
    let source = std::fs::read_to_string(&path).expect("fixture is readable");

    // The base64 payload is the largest quoted run in the file.
    let payload = longest_quoted_run(&source).expect("the fixture carries a payload");

    let mut doc = Doc::parse(&source).expect("parses");
    let uuid = doc
        .node_ids()
        .find(|&id| doc.head_is(id, "uuid"))
        .and_then(|id| doc.children(id).get(1).copied())
        .expect("the sheet has a uuid");
    doc.set_atom(uuid, "\"00000000-0000-4000-8000-0000000000ff\"");

    let edited = doc.emit();
    assert!(
        edited.contains(&payload),
        "an edit elsewhere leaves the payload byte for byte"
    );
    assert_eq!(
        changed_line_count(&source, &edited),
        1,
        "the edit stays on its own line"
    );
}

fn longest_quoted_run(source: &str) -> Option<String> {
    lex(source)
        .ok()?
        .into_iter()
        .filter(|t| t.kind == kicli_sexpr::TokenKind::Quoted)
        .map(|t| t.text(source).to_owned())
        .max_by_key(String::len)
}

#[test]
fn no_tokens_are_lost_on_write() {
    // kiutils 1.4.8 loses 14.7 % of the tokens in a KiCad 10 sheet, silently,
    // because a typed tree has nowhere to put a token it does not know.
    // kicad-skip 0.2.5 loses none but reformats every line. kicli must lose
    // none and reformat nothing.
    let path = fixture_root().join("sch/attribute_dense.kicad_sch");
    let source = std::fs::read_to_string(&path).expect("fixture is readable");

    let before = lex(&source).expect("lexes").len();
    let doc = Doc::parse(&source).expect("parses");
    let written = doc.emit();
    let after = lex(&written).expect("output lexes").len();

    assert_eq!(before, after, "no token is lost");
    assert_eq!(written, source, "and nothing is reformatted");

    let baseline = fixture_root().join("token_loss_baseline.manifest");
    let recorded = std::fs::read_to_string(baseline).expect("baseline is readable");
    assert!(
        recorded.contains("kicli 0"),
        "the baseline records kicli losing no tokens"
    );
}

#[test]
fn quoting_escapes_four_characters() {
    // The fixture holds a text box with an escaped quote, an escaped
    // backslash, escaped newlines, and a literal tab. KiCad has no \t escape,
    // so the tab is a tab byte in the file.
    let path = fixture_root().join("sch/escapes.kicad_sch");
    let source = std::fs::read_to_string(&path).expect("fixture is readable");

    assert!(source.contains(r#"\"hello\""#), "a quote is escaped");
    assert!(source.contains(r"\\"), "a backslash is escaped");
    assert!(source.contains(r"\n\n"), "a newline is escaped");
    assert!(source.contains('\t'), "a tab is a raw byte");

    let doc = Doc::parse(&source).expect("parses");
    assert_eq!(doc.emit(), source, "escapes survive a round trip");
}
