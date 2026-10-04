//! `KI-HIER-001` implements no ERC check, and this is the check that can fail.
//!
//! `spec/SPEC.md` §11.1: KiCad 10's rule check implements 47 checks and
//! **kicli's lint engine implements none of them.** `PLAN.md`'s exit-criteria
//! table names *"ERC layering"* as a gate that must fail on *"kicli
//! implementing an ERC check, or double-counting one it reports"*. For
//! `KI-HIER-001` the whole deliverable is the attribution and the gate, so the
//! thing that must be provably absent is a detector — and an absence is exactly
//! the claim that passes every test written for the behaviour, because the
//! behaviour is identical either way. A rule that read sheet pins and compared
//! them to hierarchical labels would report the same two findings on the
//! fixture and go green.
//!
//! So the claim is made at the level it can fail: **over the rule file's
//! source**. The rule may reach the drawing's rule check and nothing else.
//!
//! # The two halves, and the boundary between them
//!
//! 1. **The rule file names no way to the drawing's own geometry.** A detector
//!    has to start somewhere, and every road in runs through one of
//!    [`Drawing`](kicli::lint::Drawing)'s accessors or one of three modules.
//!    Those are the lists below, and `drawing.rule_check()` is the one
//!    permitted road.
//! 2. **The rule's code lives in that one file.** A detector written elsewhere
//!    and reported under this rule's code would satisfy the first half, so the
//!    code is swept for across every source of the crate and must appear in
//!    exactly one of them.
//!
//! What bounds the remaining hole is the companion sweep
//! `the_linter_holds_no_write_path`: the linter may name five modules of this
//! crate at all, so the surface a detector could arrive through without naming
//! it is short and readable in one sitting.
//!
//! **Every absence check here carries a presence control, and the sweep is
//! shown refusing a detector rather than only accepting the file.** A sweep
//! that read the wrong path, or looked for spellings nothing uses, would report
//! a clean file and mean nothing by it.

use std::path::{Path, PathBuf};

/// The rule file, relative to the crate.
const RULE_FILE: &str = "src/lint/rules/hier.rs";

/// The rule's published code.
const CODE: &str = "KI-HIER-001";

/// Every way a rule reaches what the drawing holds.
///
/// Taken from [`Drawing`](kicli::lint::Drawing)'s own accessor list rather than
/// from memory: `schematic`, `doc`, `library`, `nets` and `definition_of` are
/// all of them, and `path` is excluded because `delegate` needs it and it
/// carries no geometry. `resolve_pins` is the geometry helper a pin-reading
/// rule reaches for next, and it is listed so that a detector routed around the
/// accessors still trips.
const ROADS_IN: [&str; 6] = [
    "schematic()",
    ".doc()",
    "library()",
    "nets()",
    "definition_of",
    "resolve_pins",
];

/// The modules a detector would have to name.
///
/// The typed objects, the integer geometry, and the net partition. A rule that
/// compares a sheet pin with a hierarchical label needs the first; one that
/// compares their positions needs the second.
const MODULES: [&str; 3] = ["crate::model", "crate::geometry", "crate::connectivity"];

/// The one road in this rule may take.
const PERMITTED: &str = "rule_check()";

/// What a delegation must still do, so the file is not merely empty.
const DELEGATION: [&str; 2] = ["delegate(", "hier_label_mismatch"];

fn crate_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{} reads: {error}", path.display()))
}

/// The text with its comments removed, so the sweep reads code.
///
/// The module documentation above this rule explains at length why it holds no
/// detector, and it names `drawing.schematic()` while doing so. A sweep that
/// read prose would fail on the explanation of its own subject.
fn code_of(text: &str) -> String {
    let letters: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut index = 0;
    while index < letters.len() {
        if letters[index] == '/' && letters.get(index + 1) == Some(&'/') {
            while index < letters.len() && letters[index] != '\n' {
                index += 1;
            }
        } else if letters[index] == '/' && letters.get(index + 1) == Some(&'*') {
            index += 2;
            while index < letters.len()
                && !(letters[index] == '*' && letters.get(index + 1) == Some(&'/'))
            {
                index += 1;
            }
            index = (index + 2).min(letters.len());
        } else {
            out.push(letters[index]);
            index += 1;
        }
    }
    out
}

/// Everything in one rule file that a delegation may not name.
fn offences(source: &str) -> Vec<String> {
    let code = code_of(source);
    let mut found = Vec::new();
    for road in ROADS_IN {
        if code.contains(road) {
            found.push(road.to_owned());
        }
    }
    for module in MODULES {
        if code.contains(module) {
            found.push(module.to_owned());
        }
    }
    found.sort();
    found.dedup();
    found
}

/// Every Rust source file of this crate.
fn sources() -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![crate_root().join("src")];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the directory reads") {
            let path = entry.expect("the entry reads").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|kind| kind == "rs") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

#[test]
fn the_rule_reads_the_rule_check_and_nothing_else() {
    let source = read(&crate_root().join(RULE_FILE));
    let code = code_of(&source);

    assert_eq!(
        offences(&source),
        Vec::<String>::new(),
        "{RULE_FILE} names a way to the drawing's own geometry, which makes it a detector"
    );

    // The presence controls. Each one is a thing the sweep must find, so a
    // sweep pointed at the wrong file, or at a file somebody emptied, fails
    // here rather than passing the absence check for free.
    assert!(
        code.contains(PERMITTED),
        "the rule does read the rule check: {code}"
    );
    for required in DELEGATION {
        assert!(
            code.contains(required),
            "the rule is still a delegation and still names its check: {required}"
        );
    }
    assert!(
        code.contains("impl Rule for"),
        "the sweep is reading a rule file"
    );
    assert!(
        code.contains("Tier::One"),
        "and the rule still gates, which is the other half of the deliverable"
    );
}

#[test]
fn the_sweep_refuses_the_first_line_of_every_detector_it_knows() {
    // The falsification, kept. These are the shapes a future edit would reach
    // for, and each must be refused by the sweep above rather than by a
    // reviewer's memory. The first is the literal first line of the detector
    // this rule exists to prevent.
    for detector in [
        "for item in &drawing.schematic().items {}",
        "let pins = drawing.schematic().items;",
        "use crate::model::items::Item;",
        "use crate::geometry::Point;",
        "use crate::connectivity::Nets;",
        "let doc = drawing.doc();",
        "let definition = drawing.definition_of(symbol);",
        "for pin in resolve_pins(&placed, definition) {}",
        "let library = drawing.library();",
        "let nets = drawing.nets();",
        "let items = crate::model::items::Item::Symbol(one);",
    ] {
        assert!(
            !offences(detector).is_empty(),
            "the sweep refuses: {detector}"
        );
    }

    // And it permits what a delegation really holds, so the sweep is not
    // refusing everything.
    for allowed in [
        "if drawing.rule_check().covers(CHECK) { delegate(CHECK, drawing, found); }",
        "use crate::lint::erc::delegate;",
        "use crate::lint::gate::Saturation;",
        "use crate::lint::{Drawing, Findings, Rule, RuleId, Tier};",
    ] {
        assert!(offences(allowed).is_empty(), "the sweep permits: {allowed}");
    }

    // A comment is prose, not code: the rule file's own documentation names
    // `drawing.schematic()` while explaining that it never calls it.
    assert!(offences("// never reads drawing.schematic()\n").is_empty());
    assert!(offences("//! a detector would use crate::model::items\n").is_empty());
    // And the sweep must not read past a comment into the code after it.
    assert!(!offences("// harmless\nlet a = drawing.schematic();\n").is_empty());
}

#[test]
fn the_rules_code_is_claimed_by_exactly_one_file() {
    // The second half. A detector written in another module and reported under
    // this rule's code would pass the file sweep above, so the code itself is
    // swept for across the crate. Comments are stripped first, because a later
    // author may well write about this rule somewhere else.
    let claimants: Vec<String> = sources()
        .into_iter()
        .filter(|path| code_of(&read(path)).contains(CODE))
        .map(|path| {
            path.strip_prefix(crate_root())
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();
    assert_eq!(
        claimants,
        [RULE_FILE.to_owned()],
        "exactly one file reports under {CODE}"
    );

    // The control on that sweep: it really did read the crate, and the code it
    // searched for really is a code this crate uses.
    let all = sources();
    assert!(
        all.len() > 20,
        "the crate's sources were found: {}",
        all.len()
    );
    assert!(
        all.iter()
            .any(|path| path.ends_with("lint/rules/pin_on_wire.rs")),
        "the sweep reaches the rule directory"
    );
}
