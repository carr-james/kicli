//! The eight-character handle is one rule, in one place.
//!
//! A report prints the first eight characters of an identifier so a caller can
//! type them back at a command. That rule is [`kicli::model::Uuid::short`],
//! with the rustdoc that says what the handle is and is not. Every private
//! copy of it is a second place the rule can drift, and a rule that drifts is
//! a handle that stops round-tripping.
//!
//! What this sweep forbids is narrow on purpose: **a function declared to
//! shorten an identifier**. A key that is not an identifier — a snapshot's
//! content key, a hash, a hex digest — is a different thing that happens to be
//! cut at the same length, and folding it into the identifier rule would make
//! the one canonical rule a lie about what it governs. Such a shortener keeps
//! its own name, and the name says "key" rather than "uuid"; that is what the
//! classification below reads.
//!
//! Two arms:
//!
//! - Every `.rs` file under `crates/`: no function whose parameter list names
//!   an identifier may cut to eight characters.
//! - Every `.rs` file under a `src/` directory, which is the shipped code:
//!   no expression may cut an identifier to eight characters, declared in a
//!   function of its own or written inline.
//!
//! Test code outside `src/` gets the first arm only, matching the rule as its
//! chore states it: no file *declares* a second shortener. Reading an
//! identifier out of raw file text inside a test body is a different act, and
//! `crates/kicli-probe/tests/drawing.rs` does it twice; that belongs to the
//! probe-handle chore (C5), not here.

use std::path::{Path, PathBuf};

/// The file that holds the one rule, relative to the workspace root.
const DEFINER: &str = "crates/kicli/src/model/items.rs";

/// This file, which names every forbidden spelling and so must not sweep
/// itself.
const SELF: &str = "crates/kicli/tests/the_handle_has_one_name.rs";

/// The ways a source cuts something to eight characters.
const SHORTENINGS: &[&str] = &["take(8)", "truncate(8)", "get(..8)", "..8]", "nth(8)"];

/// The words that say the thing being cut is an identifier.
const IDENTIFIER_WORDS: &[&str] = &["uuid", "kiid", "identifier"];

/// One place a source cuts to eight characters.
#[derive(Debug)]
struct Cut {
    /// The file, relative to the workspace root.
    file: String,
    /// The line number, as an editor counts them.
    line: usize,
    /// The line itself, trimmed.
    text: String,
    /// The signature of the function the cut sits in, when there is one.
    signature: String,
}

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
/// Nothing here parses Rust. The private copies this sweep exists to catch are
/// three-line helpers directly above their use, and attributing a cut to the
/// nearest declaration above it is enough to read the parameter list that says
/// what is being cut.
///
/// A method takes its subject from `self`, so its parameter list names
/// nothing: `Uuid::short` itself reads as `(&self)`. For a method — and only
/// for a method, or `fn uuid_from(seed: &str)` would inherit whatever `impl`
/// happens to sit above it — the enclosing `impl` line is prepended, so the
/// type a method belongs to is part of what the classification reads.
fn enclosing_signature(lines: &[&str], at: usize) -> String {
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
    let mut signature = String::new();
    for line in lines.iter().skip(start).take(8) {
        signature.push_str(line.trim());
        signature.push(' ');
        if signature.contains(')') {
            break;
        }
    }
    if parameters(&signature).contains("self") {
        if let Some(block) = enclosing_impl(lines, start) {
            signature.insert_str(0, &block);
        }
    }
    signature
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

/// The parameter list of a signature, which is what says an argument is an
/// identifier.
fn parameters(signature: &str) -> &str {
    let Some(open) = signature.find('(') else {
        return "";
    };
    let rest = &signature[open + 1..];
    rest.find(')').map_or(rest, |close| &rest[..close])
}

/// What a declaration says it is cutting: its parameters, and the type it is
/// a method of.
///
/// The function's own **name** is deliberately left out. `fn uuid_from(seed:
/// &str)` builds an identifier out of a hash and slices the hex to lay out the
/// dashes; it is named for what it returns, not for what it cuts, and reading
/// names would make this sweep fail on it forever.
fn declared_subject(signature: &str) -> String {
    let before_fn = signature.split(" fn ").next().unwrap_or("");
    let block = if before_fn == signature {
        ""
    } else {
        before_fn
    };
    format!("{block} {}", parameters(signature))
}

fn names_an_identifier(text: &str) -> bool {
    let lowered = text.to_lowercase();
    IDENTIFIER_WORDS.iter().any(|word| lowered.contains(word))
}

/// Every cut to eight characters in a file, with the declaration it sits in.
fn cuts(file: &str, text: &str) -> Vec<Cut> {
    let lines: Vec<&str> = text.lines().collect();
    let mut found = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        // A doc comment describes a cut; it does not make one.
        if trimmed.starts_with("//") {
            continue;
        }
        if !SHORTENINGS.iter().any(|pattern| trimmed.contains(pattern)) {
            continue;
        }
        found.push(Cut {
            file: file.to_owned(),
            line: index + 1,
            text: trimmed.to_owned(),
            signature: enclosing_signature(&lines, index),
        });
    }
    found
}

/// The subject of a cut: what stands to the left of the shortening.
fn subject(cut: &Cut) -> &str {
    SHORTENINGS
        .iter()
        .filter_map(|pattern| cut.text.find(pattern).map(|at| &cut.text[..at]))
        .min_by_key(|head| head.len())
        .unwrap_or("")
}

/// Read every source once, so the two arms and the control agree on what was
/// on disk.
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
fn no_second_function_shortens_an_identifier() {
    let (_, files) = scan();
    assert!(
        files.len() > 40,
        "the workspace sources were found: {}",
        files.len()
    );

    let mut offenders = Vec::new();
    let mut permitted = Vec::new();
    for (named, text) in &files {
        if named == DEFINER || named == SELF {
            continue;
        }
        for cut in cuts(named, text) {
            let declared = names_an_identifier(&declared_subject(&cut.signature));
            // The shipped code is held to the stricter arm: an identifier is
            // not cut inline there either.
            let inline = named.contains("/src/") && names_an_identifier(subject(&cut));
            if declared || inline {
                offenders.push(format!(
                    "{}:{} {} [in `{}`]",
                    cut.file,
                    cut.line,
                    cut.text,
                    cut.signature.trim()
                ));
            } else {
                permitted.push(format!("{}:{} {}", cut.file, cut.line, cut.text));
            }
        }
    }

    // The control that the sweep read source rather than an empty list: cuts
    // of things that are not identifiers do exist, and were seen and allowed.
    assert!(
        !permitted.is_empty(),
        "the sweep read source: it found cuts to eight characters that are not identifiers"
    );
    assert!(
        offenders.is_empty(),
        "`Uuid::short` in {DEFINER} is the one eight-character handle rule; \
         a key that is not an identifier keeps a shortener whose name says so. \
         These shorten an identifier somewhere else: {offenders:#?}"
    );
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
            .any(|cut| names_an_identifier(&declared_subject(&cut.signature))),
        "the one shortener cuts to eight characters, and reads as an identifier's"
    );

    // The second half of the control: the four call sites that gave up their
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
