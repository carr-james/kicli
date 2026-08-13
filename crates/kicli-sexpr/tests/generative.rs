//! Random trees and random bytes.
//!
//! Fixtures cover what KiCad writes. These cover what an agent might write, and
//! what a corrupt file looks like. The parser will be fed generated files, so
//! "does not panic on arbitrary bytes" is a property worth owning.

use kicli_sexpr::{Doc, FormatMode, flatten, lex, prettify};
use proptest::prelude::*;

/// Atoms that have caused trouble in real files.
fn awkward_atom() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("~".to_owned()),
        Just(String::new()),
        // A bare atom starting with `#` is deliberately absent: it is not
        // representable, and `hash_leading_atoms` covers it.
        Just("\"#PWR01\"".to_owned()),
        Just("\"\"".to_owned()),
        Just("\"a b\"".to_owned()),
        Just(r#""quote \" inside""#.to_owned()),
        Just(r#""backslash \\ then quote \"""#.to_owned()),
        Just(r#""ends with an escaped backslash \\""#.to_owned()),
        Just("\"newline \\n inside\"".to_owned()),
        Just("\"tab\there\"".to_owned()),
        Just("\"µ Ω ± °C\"".to_owned()),
        Just("20260306".to_owned()),
        Just("-0.0001".to_owned()),
        Just("214748.3647".to_owned()),
        "[a-z_]{1,8}",
    ]
}

/// A tree of lists and atoms, written as source text.
fn tree_source() -> impl Strategy<Value = String> {
    let leaf = awkward_atom();
    leaf.prop_recursive(4, 32, 4, |inner| {
        prop::collection::vec(inner, 1..5).prop_map(|parts| format!("({})", parts.join(" ")))
    })
    .prop_map(|body| {
        if body.starts_with('(') {
            body
        } else {
            format!("({body})")
        }
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// A tree written out and read back is the tree we started with.
    #[test]
    fn random_trees_survive_emit_and_parse(source in tree_source()) {
        let first = Doc::parse(&source)?;
        let written = first.emit();
        let second = Doc::parse(&written).map_err(|e| TestCaseError::fail(e.to_string()))?;
        prop_assert!(first.structurally_eq(&second));
    }

    /// Laying out an already laid out file changes nothing.
    #[test]
    fn prettify_settles_after_one_pass(source in tree_source()) {
        for mode in [FormatMode::Normal, FormatMode::CompactTextProperties, FormatMode::LibraryTable] {
            let once = prettify(&source, mode);
            prop_assert_eq!(prettify(&once, mode), once);
        }
    }

    /// Stripping layout keeps every token.
    #[test]
    fn flatten_keeps_every_token(source in tree_source()) {
        let before = lex(&source).map_err(|e| TestCaseError::fail(e.to_string()))?;
        let flattened = flatten(&source);
        let after = lex(&flattened).map_err(|e| TestCaseError::fail(e.to_string()))?;
        prop_assert_eq!(before.len(), after.len());
    }

    /// Arbitrary bytes produce an error or a tree, never a panic.
    #[test]
    fn arbitrary_text_never_panics(source in ".{0,400}") {
        let _ = lex(&source);
        let _ = Doc::parse(&source);
        let _ = flatten(&source);
        let _ = prettify(&source, FormatMode::Normal);
    }

    /// Including bytes that are mostly parentheses and quotes, which is where
    /// a hand-written scanner is most likely to walk off the end.
    #[test]
    fn arbitrary_delimiters_never_panic(source in r#"[()"\\ \t\n#]{0,200}"#) {
        let _ = lex(&source);
        let _ = Doc::parse(&source);
        let _ = flatten(&source);
        let _ = prettify(&source, FormatMode::Normal);
    }
}
