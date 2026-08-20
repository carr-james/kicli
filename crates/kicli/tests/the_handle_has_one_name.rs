//! The eight-character handle is one rule, in one place.
//!
//! A report prints the first eight characters of an identifier so a caller can
//! type them back at a command. That rule is [`kicli::model::Uuid::short`],
//! with the rustdoc that says what the handle is and is not. Every private
//! copy of it is a second place the rule can drift, and a rule that drifts is
//! a handle that stops round-tripping.
//!
//! # What this sweep asks, and why it does not ask about names
//!
//! It finds **every cut of a string to its first eight characters** anywhere
//! under `crates/`, and requires each one to be on the list of accounted-for
//! sites below, each with the reason it is there.
//!
//! The first version of this sweep classified by name instead: a cut counted
//! as an identifier's if the parameter or the enclosing type was spelled
//! `uuid`, `kiid` or `identifier`. The tick reviewer broke it in two lines —
//! `fn short(id: &str)` and `impl Ident { fn short(&self) }` — both of which
//! are the exact defect this chore exists to prevent, and both of which the
//! sweep waved through in silence. A longer word list is the same instrument
//! with a longer blind spot; the next spelling is always one synonym away.
//! The chore's own history says so out loud, since the fifth copy it found had
//! been missed by an eye counting `fn short`.
//!
//! So the question is no longer "does this cut look like an identifier's". It
//! is "**is this cut accounted for**". A new cut anywhere in the workspace
//! fails until somebody writes down why it exists — which is where the
//! identifier/key distinction gets stated, in prose, in a place a reader will
//! see it.
//!
//! # What it still cannot see
//!
//! It reads text. A cut written as `&uuid[..LEN]` behind a `const LEN = 8`
//! would pass, as would one assembled at runtime. The *spellings* it knows are
//! not guessed, though: they are read off the standard library's own
//! prefix-yielding API, by the grep recorded on `CUTS`. That is the same ceiling
//! `the_router_holds_no_floating_point` and `the_four_way_rule_has_one_home`
//! work under, and it is recorded rather than papered over: the claim is that
//! no cut **in these spellings** exists unaccounted for.

use std::path::{Path, PathBuf};

/// This file, which names every spelling and so must not sweep itself.
const SELF: &str = "crates/kicli/tests/the_handle_has_one_name.rs";

/// The file that holds the one rule, relative to the workspace root.
const DEFINER: &str = "crates/kicli/src/model/items.rs";

/// How Rust spells "the first eight of these".
///
/// **Not chosen by hand.** A hand-chosen list is the name list's mistake in a
/// second costume, and it was made once here: an eleven-spelling list written
/// from memory was blind to `split_off(8)`, `drain(8..)` and
/// `split_at_mut(8)`, all three of which cut a string to eight characters and
/// all three of which passed as plants. The list below is instead read off the
/// standard library's own prefix-yielding API, which is re-derivable:
///
/// ```text
/// SRC=~/.rustup/toolchains/<toolchain>/lib/rustlib/src/rust/library
/// grep -hE '^\s+pub (const )?fn (truncate|split_at|split_at_checked|split_at_mut\
/// |get|split_off|floor_char_boundary|ceil_char_boundary|drain|chars|char_indices\
/// |bytes)\b' "$SRC"/core/src/str/mod.rs "$SRC"/alloc/src/string.rs
/// ```
///
/// Each method that can yield a prefix appears here at the width eight, plus
/// the iterator adaptors those methods feed (`take`, `nth`) and the two
/// `Index` forms. When the standard library grows another, this comment says
/// where to look rather than asking the next reader to guess.
///
/// Whitespace is stripped from a line before matching, so `get(.. 8)` and
/// `get(..8)` are the same cut. Byte and character forms are both here: which
/// one a site uses is part of what the reader below has to justify.
const CUTS: &[&str] = &[
    "take(8)",
    "nth(8)",
    "truncate(8)",
    "split_at(8)",
    "split_at_mut(8)",
    "split_at_checked(8)",
    "split_off(8)",
    "drain(8..)",
    "drain(..8)",
    "get(..8)",
    "get(0..8)",
    "get(..=7)",
    "get(0..=7)",
    "..8]",
    "..=7]",
    "floor_char_boundary(8)",
    "ceil_char_boundary(8)",
];

/// One cut that is allowed to exist, and the reason it does.
struct Accounted {
    /// The file it lives in, relative to the workspace root.
    file: &'static str,
    /// Text that must all appear in the declaration the cut sits under.
    ///
    /// The declaration carries its `impl` block when it is a method, so
    /// `impl Uuid` here is what stops a second `fn short` — on `Ident`, on
    /// `Handle`, on anything — from inheriting this entry by sharing a file
    /// and a function name.
    declaration: &'static [&'static str],
    /// Why this cut is not a second copy of the handle rule.
    why: &'static str,
}

/// Every cut to eight characters the workspace is allowed to contain.
///
/// An entry that matches nothing is a failure, not a comment: a site that
/// moves or is deleted must take its justification with it.
const ACCOUNTED: &[Accounted] = &[
    Accounted {
        file: DEFINER,
        declaration: &["impl Uuid", "fn short"],
        why: "The rule itself. Every identifier handle in kicli comes from here.",
    },
    Accounted {
        file: "crates/kicli/src/view/snapshot.rs",
        declaration: &["fn short_key"],
        why: "Deliberately retained, and deliberately not named for an identifier. A \
              snapshot key is an identifier only when the object it names has one: an \
              object kicli cannot name is keyed by a hash of its contents, and a field \
              by its owner and its name. Folding this into `Uuid::short` would make that \
              rule's rustdoc a lie about what it governs.",
    },
    Accounted {
        file: "crates/kicli/src/view/snapshot.rs",
        declaration: &["impl Encoding", "fn finish"],
        why: "Eight **bytes** of a SHA-256 digest, which is the width of a `ContentHash`. \
              It shortens a hash, not a string, and no caller could type it back.",
    },
    Accounted {
        file: "crates/kicli/src/edit/insert.rs",
        declaration: &["fn uuid_from"],
        why: "Not a shortening at all: it slices a 32-character hex digest into the five \
              groups a UUID is written in, and `[0..8]` is the first group. The result is \
              longer than what went in.",
    },
    Accounted {
        file: "crates/kicli-probe/tests/drawing.rs",
        declaration: &["fn a_probe_drawing_yields_distinct_handles_for_each_object"],
        why: "A probe test reading identifiers out of raw file text and cutting them \
              inline. It is a copy of the rule and is named as one here rather than \
              excused: the probe crate is another lane's, and the probe's handle usage \
              belongs to the probe-handle chore (C5). Accounted for so it is visible, \
              not so it is approved.",
    },
    Accounted {
        file: "crates/kicli-probe/tests/drawing.rs",
        declaration: &["fn sibling_probes_of_different_series_have_no_colliding_handles"],
        why: "The same inline copy, a second time, in the same probe test file. Same \
              owner, same chore (C5).",
    },
];

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the workspace root resolves")
}

/// Every Rust source in the workspace, in a stable order.
///
/// The order is sorted rather than the order the filesystem hands back, so a
/// failure names the same first offender on every machine and every run.
fn sources(workspace: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    walk(&workspace.join("crates"), &mut found);
    found.sort();
    found
}

fn walk(root: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == "target") {
                continue;
            }
            walk(&path, found);
        } else if path.extension().is_some_and(|end| end == "rs") {
            found.push(path);
        }
    }
}

fn relative(workspace: &Path, file: &Path) -> String {
    file.strip_prefix(workspace)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/")
}

/// One place a source cuts a string to eight characters.
#[derive(Debug)]
struct Cut {
    /// The file, relative to the workspace root.
    file: String,
    /// The line number, as an editor counts them.
    line: usize,
    /// The line itself, trimmed.
    text: String,
    /// The declaration the cut sits in, carrying its `impl` block when it is a
    /// method.
    declaration: String,
}

/// Does this line start a function declaration?
fn is_declaration(trimmed: &str) -> bool {
    let head = trimmed
        .trim_start_matches("pub(crate) ")
        .trim_start_matches("pub(super) ")
        .trim_start_matches("pub ")
        .trim_start_matches("const ")
        .trim_start_matches("async ")
        .trim_start_matches("unsafe ")
        .trim_start_matches("extern \"C\" ");
    head.starts_with("fn ")
}

/// The declaration a line sits under, joined until its parameter list closes.
///
/// Nothing here parses Rust. Attributing a cut to the nearest declaration
/// above it is enough to say *where* the cut is, which is all the accounting
/// needs — the decision of whether a cut may exist is made by a human writing
/// a reason, not by this function guessing from what it reads.
///
/// A method takes its subject from `self`, so the enclosing `impl` line is
/// prepended for methods — and only for methods, or `fn uuid_from(seed: &str)`
/// would inherit whatever `impl` happens to sit above it.
fn enclosing_declaration(lines: &[&str], at: usize) -> String {
    let mut start = at;
    loop {
        if is_declaration(lines[start].trim_start()) {
            break;
        }
        if start == 0 {
            return String::new();
        }
        start -= 1;
    }
    let mut declaration = String::new();
    for line in lines.iter().skip(start).take(8) {
        declaration.push_str(line.trim());
        declaration.push(' ');
        if declaration.contains(')') {
            break;
        }
    }
    if parameters(&declaration).contains("self") {
        if let Some(block) = enclosing_impl(lines, start) {
            declaration.insert_str(0, &block);
        }
    }
    declaration
}

/// The `impl` block a method sits in, when it sits in one.
fn enclosing_impl(lines: &[&str], at: usize) -> Option<String> {
    (0..at).rev().find_map(|index| {
        let trimmed = lines[index].trim_start();
        trimmed
            .starts_with("impl ")
            .then(|| format!("{} ", trimmed.trim_end_matches('{').trim()))
    })
}

/// The parameter list of a declaration.
fn parameters(declaration: &str) -> &str {
    let Some(open) = declaration.find('(') else {
        return "";
    };
    let rest = &declaration[open + 1..];
    rest.find(')').map_or(rest, |close| &rest[..close])
}

/// Every cut to eight characters in a file, with the declaration it sits in.
fn cuts(file: &str, text: &str) -> Vec<Cut> {
    let lines: Vec<&str> = text.lines().collect();
    let mut found = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        // A comment describes a cut; it does not make one.
        if trimmed.starts_with("//") {
            continue;
        }
        let dense: String = trimmed.chars().filter(|c| !c.is_whitespace()).collect();
        if !CUTS.iter().any(|spelling| dense.contains(spelling)) {
            continue;
        }
        found.push(Cut {
            file: file.to_owned(),
            line: index + 1,
            text: trimmed.to_owned(),
            declaration: enclosing_declaration(&lines, index),
        });
    }
    found
}

/// Which accounted-for site a cut belongs to, if any.
fn accounted_for(cut: &Cut) -> Option<usize> {
    ACCOUNTED.iter().position(|entry| {
        entry.file == cut.file
            && entry
                .declaration
                .iter()
                .all(|part| cut.declaration.contains(part))
    })
}

/// Read every source once, so both tests agree on what was on disk.
fn scan() -> (PathBuf, Vec<(String, String)>) {
    let workspace = workspace();
    let files = sources(&workspace)
        .into_iter()
        .filter_map(|file| {
            let named = relative(&workspace, &file);
            std::fs::read_to_string(&file)
                .ok()
                .map(|text| (named, text))
        })
        .collect();
    (workspace, files)
}

#[test]
fn every_cut_to_eight_characters_is_accounted_for() {
    let (_, files) = scan();
    assert!(
        files.len() > 40,
        "the workspace sources were found: {}",
        files.len()
    );

    let mut offenders = Vec::new();
    let mut matched = vec![0_usize; ACCOUNTED.len()];
    for (named, text) in &files {
        if named == SELF {
            continue;
        }
        for cut in cuts(named, text) {
            match accounted_for(&cut) {
                Some(entry) => matched[entry] += 1,
                None => offenders.push(format!(
                    "{}:{} {} [in `{}`]",
                    cut.file,
                    cut.line,
                    cut.text,
                    cut.declaration.trim()
                )),
            }
        }
    }

    // The control that the sweep read source rather than an empty list, and
    // that the list has not rotted: every entry must still describe a real
    // site, or a justification is outliving the thing it justified.
    let stale: Vec<&str> = ACCOUNTED
        .iter()
        .zip(&matched)
        .filter(|(_, count)| **count == 0)
        .map(|(entry, _)| entry.file)
        .collect();
    assert!(
        stale.is_empty(),
        "these accounted-for sites no longer exist, so the sweep read something \
         other than the workspace it is written against: {stale:?}"
    );

    // An allowlist without reasons is a blind spot with a comment, so the
    // reason is asserted rather than merely stored. It is also what a failure
    // prints: whoever hits this sees the whole permitted set and why each
    // member is in it, which is the argument they have to join or refute.
    for entry in ACCOUNTED {
        assert!(
            entry.why.len() > 60,
            "{} carries a reason, not a label: {:?}",
            entry.file,
            entry.why
        );
    }

    assert!(
        offenders.is_empty(),
        "every cut to eight characters under crates/ must be accounted for in \
         ACCOUNTED, with the reason it is not a second copy of the handle rule \
         (`Uuid::short`, in {DEFINER}). These are not: {offenders:#?}\n\n\
         The cuts that ARE accounted for, and why:\n{}",
        permitted_set()
    );
}

/// The permitted set, spelled out for whoever a failure lands on.
fn permitted_set() -> String {
    ACCOUNTED
        .iter()
        .map(|entry| {
            format!(
                "  - {} [{}]\n      {}\n",
                entry.file,
                entry.declaration.join(" + "),
                entry.why
            )
        })
        .collect()
}

#[test]
fn the_one_rule_is_where_it_is_claimed_to_be() {
    let (workspace, files) = scan();
    let (_, definer) = files
        .iter()
        .find(|(named, _)| named == DEFINER)
        .unwrap_or_else(|| panic!("{DEFINER} was read from {}", workspace.display()));

    // The presence control. A sweep that read nothing would report every file
    // clean, so the rule it is enforcing must be found before an absence
    // counts for anything.
    assert!(
        definer.contains("impl Uuid {"),
        "{DEFINER} holds the identifier type"
    );
    assert!(
        definer.contains("pub fn short(&self) -> &str"),
        "{DEFINER} declares the one shortener"
    );
    assert!(
        cuts(DEFINER, definer)
            .iter()
            .any(|cut| accounted_for(cut) == Some(0)),
        "the one shortener cuts to eight characters, and is the first \
         accounted-for site"
    );

    // The second half of the control: the five call sites that gave up their
    // private copies now reach the one rule. A fold that deleted a helper and
    // its callers would pass the sweep and fail here.
    for caller in [
        "crates/kicli/src/edit/net.rs",
        "crates/kicli/src/edit/mark.rs",
        "crates/kicli/src/view/connectivity.rs",
        "crates/kicli/src/view/snapshot.rs",
        "crates/kicli/src/cli/edit/wire.rs",
    ] {
        let (_, text) = files
            .iter()
            .find(|(named, _)| named == caller)
            .unwrap_or_else(|| panic!("{caller} was read"));
        assert!(
            text.contains(".short()"),
            "{caller} calls the one rule rather than a copy of it"
        );
    }
}
