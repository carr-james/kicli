//! The linter is integer arithmetic, and that is checked rather than intended.
//!
//! Two runs over one drawing must produce the same score, on any machine,
//! forever. A float in the arithmetic would make that a hope: the same
//! expression can round two ways on two targets, and a score that differs in
//! the last place reports a drawing as changed when nothing changed. The score
//! decays exponentially, which is exactly where an author reaches for a float,
//! so the exponential is evaluated in fixed point instead.
//!
//! **The sweep exempts nothing.** Not a file, not a function, not one
//! expression. The published rule permits a float in the final exponential;
//! the linter does not take that permission, so the gate needs no hole and
//! cannot be widened by a later author who finds a hole convenient.
//!
//! # What the sweep reads, and where it stops
//!
//! Comments and string literals are removed first, so prose about floating
//! point does not fail the check and a decimal inside a message is not
//! arithmetic. What is left is code.
//!
//! Two mechanisms are refused: the type names, and the literals. The literal
//! forms are taken from the Rust grammar rather than from memory — a decimal
//! point, an exponent, or an `f32`/`f64` suffix — because a list of spellings
//! an author thought of is one spelling behind.
//!
//! **The boundary is worth stating.** A textual sweep cannot see a float that
//! arrives as another module's return type without ever being named or cast.
//! What bounds that hole is the companion sweep in
//! `the_linter_holds_no_write_path`: it whitelists the modules the linter may
//! name at all, so the surface a float could arrive through is five modules
//! long and readable in one sitting.

use std::path::{Path, PathBuf};

/// The types the linter may not name.
const FORBIDDEN: [&str; 2] = ["f32", "f64"];

/// A type the linter really does name, which the sweep expects to find.
const PRESENT: &str = "u128";

/// Every source file of the linter, including the rule files.
fn linter_sources() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lint");
    let mut found = vec![root.with_extension("rs")];
    collect(&root, &mut found);
    found.sort();
    found
}

/// Add every `.rs` file under a directory, however deep.
fn collect(dir: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, found);
        } else if path.extension().is_some_and(|end| end == "rs") {
            found.push(path);
        }
    }
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

/// Could this letter be part of a name?
fn in_a_name(letter: Option<&char>) -> bool {
    letter.is_some_and(|c| c.is_alphanumeric() || *c == '_')
}

/// The text with its comments and its literal text removed, so the sweep reads
/// arithmetic.
///
/// A character literal is skipped whole, so that a quotation mark written as a
/// character does not open a string that never closes. A lifetime is left
/// alone, because it is not a literal and does not end in a quotation mark.
fn code_of(text: &str) -> String {
    let letters: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    while at < letters.len() {
        let step = skipped(&letters, at);
        if step > 0 {
            at += step;
            continue;
        }
        out.push(letters[at]);
        at += 1;
    }
    out
}

/// How many characters to drop at this position, or none.
fn skipped(letters: &[char], at: usize) -> usize {
    if letters[at] == '/' && letters.get(at + 1) == Some(&'/') {
        return to_the_end_of_the_line(letters, at);
    }
    if letters[at] == '/' && letters.get(at + 1) == Some(&'*') {
        return to_the_end_of_the_block(letters, at);
    }
    if letters[at] == '"' {
        return to_the_end_of_the_string(letters, at);
    }
    if letters[at] == '\'' {
        return a_character_literal(letters, at);
    }
    0
}

/// The length of a line comment starting here.
fn to_the_end_of_the_line(letters: &[char], at: usize) -> usize {
    let mut end = at;
    while end < letters.len() && letters[end] != '\n' {
        end += 1;
    }
    end - at
}

/// The length of a block comment starting here.
fn to_the_end_of_the_block(letters: &[char], at: usize) -> usize {
    let mut end = at + 2;
    while end < letters.len() && !(letters[end] == '*' && letters.get(end + 1) == Some(&'/')) {
        end += 1;
    }
    (end + 2).min(letters.len()) - at
}

/// The length of a string literal starting here.
fn to_the_end_of_the_string(letters: &[char], at: usize) -> usize {
    let mut end = at + 1;
    while end < letters.len() {
        if letters[end] == '\\' {
            end += 2;
            continue;
        }
        if letters[end] == '"' {
            return end + 1 - at;
        }
        end += 1;
    }
    letters.len() - at
}

/// The length of a character literal starting here, or none if this is a
/// lifetime.
fn a_character_literal(letters: &[char], at: usize) -> usize {
    if letters.get(at + 2) == Some(&'\'') {
        return 3;
    }
    if letters.get(at + 1) == Some(&'\\') && letters.get(at + 3) == Some(&'\'') {
        return 4;
    }
    0
}

/// Every floating point literal in the code, as written.
///
/// The Rust grammar gives a float literal three shapes: a decimal point, an
/// exponent, or a type suffix. Each is refused. A number written in hex,
/// octal or binary is never a float, and a digit reached through a dot is a
/// field of a tuple rather than the start of a number.
fn float_literals(code: &str) -> Vec<String> {
    let letters: Vec<char> = code.chars().collect();
    let mut found = Vec::new();
    let mut at = 0;
    while at < letters.len() {
        if !starts_a_number(&letters, at) {
            at += 1;
            continue;
        }
        let (literal, after) = a_number(&letters, at);
        if is_a_float(&literal) {
            found.push(literal);
        }
        at = after;
    }
    found
}

/// Does a decimal number start at this position?
fn starts_a_number(letters: &[char], at: usize) -> bool {
    let before = at.checked_sub(1).and_then(|back| letters.get(back));
    letters[at].is_ascii_digit() && !in_a_name(before) && before != Some(&'.')
}

/// One numeric literal, and where it ended.
fn a_number(letters: &[char], at: usize) -> (String, usize) {
    let mut end = at;
    let radix_marked = letters.get(at) == Some(&'0')
        && matches!(letters.get(at + 1), Some('x' | 'X' | 'o' | 'O' | 'b' | 'B'));
    if radix_marked {
        end += 2;
        while in_a_name(letters.get(end)) {
            end += 1;
        }
        return (letters[at..end].iter().collect(), end);
    }
    end = past_digits(letters, end);
    if letters.get(end) == Some(&'.') && letters.get(end + 1).is_some_and(char::is_ascii_digit) {
        end = past_digits(letters, end + 1);
    } else if letters.get(end) == Some(&'.') && ends_a_float(letters.get(end + 1)) {
        end += 1;
    }
    if matches!(letters.get(end), Some('e' | 'E'))
        && (letters.get(end + 1).is_some_and(char::is_ascii_digit)
            || matches!(letters.get(end + 1), Some('+' | '-')))
    {
        end = past_digits(letters, end + 2);
    }
    while in_a_name(letters.get(end)) {
        end += 1;
    }
    (letters[at..end].iter().collect(), end)
}

/// Does a number end at a decimal point followed by this?
///
/// A trailing point makes a float, unless a name, a digit separator or a
/// second point follows it. Those three are a method call, a suffix and a
/// range, none of which is a number.
fn ends_a_float(letter: Option<&char>) -> bool {
    !in_a_name(letter) && letter != Some(&'.')
}

/// Past a run of digits and separators.
fn past_digits(letters: &[char], from: usize) -> usize {
    let mut end = from;
    while letters
        .get(end)
        .is_some_and(|letter| letter.is_ascii_digit() || *letter == '_')
    {
        end += 1;
    }
    end
}

/// Is this numeric literal a floating point one?
fn is_a_float(literal: &str) -> bool {
    if literal.starts_with("0x") || literal.starts_with("0X") {
        return false;
    }
    literal.contains('.')
        || literal.ends_with("f32")
        || literal.ends_with("f64")
        || literal.contains('e')
        || literal.contains('E')
}

/// Everything in one source that the linter may not hold.
fn offences(text: &str) -> Vec<String> {
    let code = code_of(text);
    let mut offences = Vec::new();
    for forbidden in FORBIDDEN {
        if names_type(&code, forbidden) {
            offences.push(forbidden.to_owned());
        }
    }
    offences.extend(float_literals(&code));
    offences.sort();
    offences.dedup();
    offences
}

#[test]
fn no_floating_point_appears_under_the_linter() {
    let sources = linter_sources();
    assert!(sources.len() >= 6, "the linter's sources were found");

    let mut offenders = Vec::new();
    for source in &sources {
        let text = std::fs::read_to_string(source).expect("a linter source reads");
        for offence in offences(&text) {
            offenders.push(format!(
                "{}: {offence}",
                source.file_name().unwrap_or_default().to_string_lossy()
            ));
        }
    }
    assert!(
        offenders.is_empty(),
        "the linter's arithmetic must be exact: {offenders:?}"
    );
}

#[test]
fn the_sweep_can_see_what_it_is_looking_for() {
    // The control. A sweep that read nothing, or that stripped everything it
    // was given, would pass the check above while a float sat in the score.
    let sources = linter_sources();
    let text: String = sources
        .iter()
        .map(|source| std::fs::read_to_string(source).expect("a linter source reads"))
        .collect();
    assert!(!text.is_empty(), "the linter's sources were read");
    let code = code_of(&text);
    assert!(
        names_type(&code, PRESENT),
        "the linter scores in {PRESENT}, so the sweep is reading the linter"
    );
    assert!(
        !float_literals(&code).is_empty() || code.contains('1'),
        "the sweep found numbers, so it is reading arithmetic"
    );
}

#[test]
fn the_sweep_refuses_every_way_in_it_knows() {
    // Each mechanism the Rust grammar gives a float, exercised on source the
    // linter must not hold.
    for forbidden in [
        "let scale: f64 = 1;",
        "let scale = (count as f32);",
        "let scale = 1.0;",
        "let scale = 1.;",
        "let scale = 1e10;",
        "let scale = 1E10;",
        "let scale = 2.5e-3;",
        "let scale = 1f64;",
        "let scale = 1_000f32;",
        "fn decay(raw: f64) -> f64 { raw.exp() }",
    ] {
        assert!(
            !offences(forbidden).is_empty(),
            "the sweep refuses: {forbidden}"
        );
    }
}

#[test]
fn the_sweep_permits_what_integer_arithmetic_looks_like() {
    for allowed in [
        "let whole = value / 1_000_000_000;",
        "let x = point.x.0;",
        "let nested = pair.0.1;",
        "let mask = 0xdeadbeef;",
        "let mask = 0x1e5;",
        "let bits = 0b1010;",
        "let biggest = 1.max(2);",
        "for step in 1..=TERMS { total += step; }",
        "let fits = (3..=6).contains(&length);",
        "for points in 0..200 { total += points; }",
        "struct Holder<'a> { text: &'a str }",
        "let quote = '\"';",
        "let zero = '0';",
        "let named = if64_is_not_a_type;",
        "let text = format!(\"{:09}\", value);",
    ] {
        assert!(offences(allowed).is_empty(), "the sweep permits: {allowed}");
    }

    // Prose is not arithmetic, and a decimal in a message is not a float.
    assert!(offences("// the score is 100 * exp(-raw / 25.0)\n").is_empty());
    assert!(offences("/* a penalty of 3.0 points */\n").is_empty());
    assert!(offences("assert_eq!(penalty.text(), \"1.25\");").is_empty());
    // And the sweep must not read past a comment or a string.
    assert!(!offences("// harmless\nlet scale = 1.0;\n").is_empty());
    assert!(!offences("let name = \"1.25\";\nlet scale = 2.5;\n").is_empty());
}
