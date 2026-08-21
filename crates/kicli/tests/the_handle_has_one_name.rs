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
//! # The claim, stated so it can be checked
//!
//! > No expression anywhere under `crates/` takes the first eight characters
//! > or bytes of a string, at a literal width of eight, **by any of the four
//! > mechanisms enumerated below**, except at the sites in `ACCOUNTED` with
//! > the reason each is there.
//!
//! The claim used to end "in any spelling the standard library offers for
//! doing so". That was wider than any grep can be, and it was wrong in the
//! specific way this chore keeps being wrong: the derivation behind it
//! enumerated `str`, `String` and `Iterator` **methods**, so
//! `format!("{:.8}", uuid)` — `core::fmt` precision, as std-offered as
//! `chars().take(8)` and the way an engineer would actually write it — sat
//! outside the derivation while sounding inside the claim.
//!
//! ## The taxonomy is the boundary
//!
//! Four mechanisms are covered, and naming them **is** the limit of the claim:
//!
//! 1. method calls on `str` and `String`;
//! 2. method calls on `[T]`, reached by `as_bytes` or `into_bytes`;
//! 3. `Iterator` adaptors over `chars`, `char_indices`, `bytes`, `into_chars`;
//! 4. `core::fmt` precision in a format spec, in any macro that takes one.
//!
//! Plus the `core::ops::Index` range forms, which are an operator rather than
//! a method and are listed with (1).
//!
//! ## What is outside it, named rather than implied
//!
//! - **A width that is not the literal 8 in the expression**: `&uuid[..LEN]`
//!   behind a `const LEN = 8`, a width read from configuration, and the
//!   indirect format precisions `{:.*}`, `{:.1$}`, `{:.n$}` — the last three
//!   are `core::fmt`'s own grammar, but binding a `$`-parameter to its
//!   argument means parsing the macro call, which this does not do.
//! - **A hand-rolled loop** with its own counter, or `retain`/`take_while`
//!   closing over one.
//! - **A user macro** whose body expands to any of the above.
//! - **A helper from outside `std`** — no dependency offers one today, and a
//!   new dependency would need its own enumeration.
//! - **Anything visible only after monomorphisation or const evaluation.**
//!
//! Closing those needs a lint over compiled MIR rather than a reader of source
//! text. That is separate work, recorded as such in the C1 entry rather than
//! half-done here.
//!
//! **And the honest residual: a fifth mechanism may exist.** Three of the four
//! above were found by somebody else pointing at a gap. What is claimed is
//! that these four are covered exhaustively and that the boundary is written
//! down — not that the taxonomy is complete. A reader who finds a fifth is
//! finding a real defect in this sweep, and the sweep says so out loud rather
//! than leaving them to discover it against a sentence that promised more.

use std::path::{Path, PathBuf};

/// This file, which names every spelling and so must not sweep itself.
const SELF: &str = "crates/kicli/tests/the_handle_has_one_name.rs";

/// The file that holds the one rule, relative to the workspace root.
const DEFINER: &str = "crates/kicli/src/model/items.rs";

/// How Rust spells "the first eight of these".
///
/// **The vocabulary does not come from the author's head, and that is the
/// whole point of this constant.** Three instruments have now failed in this
/// chore for the same reason — each was built from what its writer could think
/// of, so its own falsification table was spelled the way it expected and
/// could not see the gap:
///
/// 1. the sweep classified by *name*, against `["uuid", "kiid", "identifier"]`,
///    and the tick reviewer walked `fn short(id: &str)` straight through it;
/// 2. the cut-based rework hand-listed *slice spellings*, and `split_off(8)`,
///    `drain(8..)` and `split_at_mut(8)` walked through that;
/// 3. the first fix for (2) grepped `std` — with a hand-written alternation of
///    method names, which is the same closed list one level up.
///
/// 4. and the enumerations that fixed (3) were **three method lists**, chosen
///    by the author. `format!("{:.8}", uuid)` is a complete second copy of the
///    rule, and passed. The vocabulary had moved outside the author's head;
///    the **taxonomy** had not.
///
/// So the list below is **exhaustive over four external enumerations**, each
/// mechanically re-derivable and each checkable against `doc.rust-lang.org` by
/// a reader who has never seen this repository. Every public method of `str`,
/// of `String`, of `Iterator` and of `[T]` was listed and considered, and the
/// `core::fmt` format-spec grammar was transcribed from std's own statement of
/// it. The ones that can yield a prefix at a literal width of eight appear
/// here; the reasoning for the rest is in the C1 task entry, item by item.
///
/// ```text
/// SRC=$(rustc --print sysroot)/lib/rustlib/src/rust/library
/// awk '/^impl str \{/,0' "$SRC"/core/src/str/mod.rs \
///   | grep -oE 'pub (const )?fn [a-z_0-9]+' | sed -E 's/.*fn //' | sort -u
/// awk '/^impl String \{/,/^impl FromUtf8Error/' "$SRC"/alloc/src/string.rs \
///   | grep -oE 'pub (const )?fn [a-z_0-9]+' | sed -E 's/.*fn //' | sort -u
/// grep -oE '^\s+fn [a-z_0-9]+' "$SRC"/core/src/iter/traits/iterator.rs \
///   | sed -E 's/.*fn //' | sort -u
/// awk '/^impl<T> \[T\] \{/,/^impl<T, const N: usize>/' "$SRC"/core/src/slice/mod.rs \
///   | grep -oE 'pub (const )?fn [a-z_0-9]+' | sed -E 's/.*fn //' | sort -u
/// sed -n '/format_spec :=/,/parameter :=/p' "$SRC"/alloc/src/fmt.rs
/// ```
///
/// **Four enumerations is not a claim that four is all there are.** It is the
/// number of mechanisms this instrument covers, and the taxonomy is stated as
/// the boundary on the module rather than implied to be complete.
///
/// Whitespace is stripped from a line before matching, so `get(.. 8)` and
/// `get(..8)` are the same cut. Byte and character forms are both here: which
/// one a site uses is part of what its accounted-for reason has to justify.
///
/// What this still cannot reach is recorded on the module, not hidden: a width
/// behind a `const`, a width computed at run time, or a hand-rolled loop.
const CUTS: &[&str] = &[
    // `str`, by index or by explicit split.
    "get(..8)",
    "get(0..8)",
    "get(..=7)",
    "get(0..=7)",
    "get_mut(..8)",
    "get_mut(0..8)",
    "split_at(8)",
    "split_at_checked(8)",
    "split_at_mut(8)",
    "split_at_mut_checked(8)",
    "floor_char_boundary(8)",
    "ceil_char_boundary(8)",
    // The `Index` operator forms, from `core::ops` rather than an inherent fn.
    "..8]",
    "..=7]",
    // `String`, which can cut in place.
    "truncate(8)",
    "split_off(8)",
    "drain(8..)",
    "drain(..8)",
    "replace_range(8..",
    // `Iterator`, which is what `chars`, `char_indices`, `bytes` and
    // `into_chars` feed. These four are not cuts themselves.
    "take(8)",
    "nth(8)",
    "zip(0..8)",
    "next_chunk::<8>",
    "array_chunks::<8>",
    // `[T]`, reached from a string by `as_bytes` or `into_bytes`. Found by
    // this lane, not by a reviewer: the first three enumerations covered
    // `str`, `String` and `Iterator`, and a byte prefix goes through none of
    // them. The range-taking `split_off` here is the slice one, which is a
    // different signature from `String`'s.
    "split_off(..8)",
    "first_chunk::<8>",
    "split_first_chunk::<8>",
    "as_chunks::<8>",
    "array_windows::<8>",
    "chunks(8)",
    "chunks_exact(8)",
    "windows(8)",
    // `core::fmt` precision, which is not a method at all and is why the
    // taxonomy — not the method lists — is the real boundary. From std's own
    // grammar: `format_spec := …['.' precision][type]`, so a literal precision
    // of eight is `.8` followed by an optional `type` and the closing brace.
    // The `type` production is transcribed from that grammar, not recalled:
    // `type := '?' | 'x?' | 'X?' | 'o' | 'x' | 'X' | 'p' | 'b' | 'e' | 'E'`.
    // Matching the spec rather than the macro name is deliberate — it covers
    // `format!`, `write!`, `panic!`, `assert!` and anything else that takes a
    // format string, including macros std has not written yet.
    ".8}",
    ".8?}",
    ".8x?}",
    ".8X?}",
    ".8o}",
    ".8x}",
    ".8X}",
    ".8p}",
    ".8b}",
    ".8e}",
    ".8E}",
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
