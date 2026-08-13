//! Derive the stroke-font glyph advances from KiCad's Newstroke data.
//!
//! KiCad measures a string by adding one advance per glyph. The advance is the
//! width recorded in the first two bytes of the glyph's Newstroke entry, over
//! the font scale of 21. This task reads those entries from a KiCad source tree
//! and writes `crates/kicli/src/geometry/advances.table`, which the crate
//! embeds. The table is committed, so `cargo test` needs neither KiCad nor the
//! network.
//!
//! `--verify` re-derives the table and compares it with the committed bytes, so
//! a KiCad upgrade that changes a glyph is reported rather than assumed away.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::corpus::PINNED_TAG;

/// The Newstroke file inside a KiCad source tree.
const NEWSTROKE_PATH: &str = "common/newstroke_font.cpp";

/// The environment variable that names a KiCad source tree.
const SOURCE_VARIABLE: &str = "KICLI_KICAD_SOURCE";

/// The scale KiCad divides every stroke coordinate by.
///
/// `STROKE_FONT_SCALE` in `common/font/stroke_font.cpp`.
const FONT_SCALE: i32 = 21;

/// Where the derived table is written, relative to the workspace root.
const TABLE_PATH: &str = "crates/kicli/src/geometry/advances.table";

/// The workspace root.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// The KiCad source tree to read.
///
/// The environment variable wins, so a checkout elsewhere can be used without
/// fetching the corpus.
fn source_root() -> PathBuf {
    match std::env::var_os(SOURCE_VARIABLE) {
        Some(path) => PathBuf::from(path),
        None => workspace_root().join("target/corpus/kicad"),
    }
}

/// Derive the table, or verify it when `--verify` is given.
pub fn run(verify_only: bool) -> ExitCode {
    let source = source_root().join(NEWSTROKE_PATH);
    let table_path = workspace_root().join(TABLE_PATH);

    let text = match std::fs::read_to_string(&source) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("text-metrics: cannot read {}: {error}", source.display());
            eprintln!("text-metrics: run `cargo xtask corpus` to fetch KiCad,");
            eprintln!("text-metrics: or set {SOURCE_VARIABLE} to a KiCad source tree.");
            return ExitCode::FAILURE;
        }
    };

    let advances = match parse_advances(&text) {
        Ok(advances) => advances,
        Err(reason) => {
            eprintln!(
                "text-metrics: {} is not readable: {reason}",
                source.display()
            );
            return ExitCode::FAILURE;
        }
    };

    let table = render_table(&advances);
    println!(
        "text-metrics: {} glyphs from {}",
        advances.len(),
        source.display()
    );

    if verify_only {
        return verify(&table_path, &table);
    }

    if let Err(error) = std::fs::write(&table_path, &table) {
        eprintln!(
            "text-metrics: cannot write {}: {error}",
            table_path.display()
        );
        return ExitCode::FAILURE;
    }
    println!("text-metrics: wrote {}", table_path.display());
    ExitCode::SUCCESS
}

/// Compare the derived table with the committed one.
fn verify(table_path: &Path, derived: &str) -> ExitCode {
    let committed = match std::fs::read_to_string(table_path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!(
                "text-metrics: cannot read {}: {error}",
                table_path.display()
            );
            return ExitCode::FAILURE;
        }
    };

    if committed == derived {
        println!("text-metrics: the committed table matches the KiCad source");
        return ExitCode::SUCCESS;
    }

    eprintln!("text-metrics: the committed table does not match the KiCad source");
    let mut differences = 0;
    for (line, (was, now)) in committed.lines().zip(derived.lines()).enumerate() {
        if was != now {
            differences += 1;
            if differences <= 10 {
                eprintln!("  line {}: committed {was:?}, derived {now:?}", line + 1);
            }
        }
    }
    if committed.lines().count() != derived.lines().count() {
        eprintln!(
            "  committed has {} lines, derived has {}",
            committed.lines().count(),
            derived.lines().count()
        );
    }
    eprintln!("text-metrics: run `cargo xtask text-metrics` to update it");
    ExitCode::FAILURE
}

/// The advance numerator of every glyph, in Newstroke order.
///
/// Ported from `STROKE_FONT::loadNewStrokeFont`
/// (`common/font/stroke_font.cpp:99-191`), which reads the width of a glyph
/// from the first two bytes of its entry: `end - start`, both offset by `R`.
/// The offset cancels, so the numerator is the plain byte difference.
fn parse_advances(source: &str) -> Result<Vec<i32>, String> {
    let entries = parse_entries(source)?;
    let mut advances = Vec::with_capacity(entries.len());
    for (index, entry) in entries.iter().enumerate() {
        let bytes = entry.as_bytes();
        if bytes.len() < 2 {
            return Err(format!("glyph {index} has no width"));
        }
        advances.push(i32::from(bytes[1]) - i32::from(bytes[0]));
    }
    if advances.len() < 95 {
        return Err(format!(
            "expected at least 95 glyphs, found {}",
            advances.len()
        ));
    }
    Ok(advances)
}

/// Read the string literals of the `newstroke_font` array, in order.
fn parse_entries(source: &str) -> Result<Vec<String>, String> {
    let start = source
        .find("const char* const newstroke_font[]")
        .ok_or("the newstroke_font array is absent")?;
    let bytes = source.as_bytes();
    let mut at = source[start..]
        .find('{')
        .ok_or("the newstroke_font array has no body")?
        + start
        + 1;

    let mut entries = Vec::new();
    loop {
        at = skip_filler(bytes, at);
        match bytes.get(at) {
            None => return Err("the newstroke_font array does not end".to_owned()),
            Some(b'}') => return Ok(entries),
            Some(b'"') => {
                let (entry, next) = read_literal(bytes, at + 1)?;
                entries.push(entry);
                at = next;
            }
            Some(other) => {
                return Err(format!(
                    "unexpected byte {:?} in the newstroke_font array",
                    *other as char
                ));
            }
        }
    }
}

/// Skip whitespace, commas and C comments.
fn skip_filler(bytes: &[u8], mut at: usize) -> usize {
    loop {
        match bytes.get(at) {
            Some(b' ' | b'\t' | b'\r' | b'\n' | b',') => at += 1,
            Some(b'/') if bytes.get(at + 1) == Some(&b'/') => {
                while at < bytes.len() && bytes[at] != b'\n' {
                    at += 1;
                }
            }
            Some(b'/') if bytes.get(at + 1) == Some(&b'*') => {
                at += 2;
                while at + 1 < bytes.len() && !(bytes[at] == b'*' && bytes[at + 1] == b'/') {
                    at += 1;
                }
                at += 2;
            }
            _ => return at,
        }
    }
}

/// Read one C string literal, starting after its opening quote.
///
/// Returns the decoded bytes and the position after the closing quote. Only the
/// escapes the font data uses are decoded; anything else is an error, so a new
/// escape is reported rather than mangled.
fn read_literal(bytes: &[u8], mut at: usize) -> Result<(String, usize), String> {
    let mut decoded = Vec::new();
    loop {
        match bytes.get(at) {
            None => return Err("a glyph entry does not end".to_owned()),
            Some(b'"') => {
                let text = String::from_utf8(decoded)
                    .map_err(|_| "a glyph entry is not UTF-8".to_owned())?;
                return Ok((text, at + 1));
            }
            Some(b'\\') => {
                let escape = bytes.get(at + 1).ok_or("a glyph entry ends in a escape")?;
                let value = match escape {
                    b'\\' => b'\\',
                    b'"' => b'"',
                    b'\'' => b'\'',
                    b'n' => b'\n',
                    b't' => b'\t',
                    other => return Err(format!("unknown escape \\{}", *other as char)),
                };
                decoded.push(value);
                at += 2;
            }
            Some(other) => {
                decoded.push(*other);
                at += 1;
            }
        }
    }
}

/// Write the table: a provenance header, then one record per run.
///
/// Runs of equal advance are collapsed because the font's ideograph blocks are
/// thousands of glyphs wide, which would otherwise dominate the file.
fn render_table(advances: &[i32]) -> String {
    let mut out = String::new();
    out.push_str(&header(advances.len()));

    let mut first = 0usize;
    while first < advances.len() {
        let mut last = first;
        while last + 1 < advances.len() && advances[last + 1] == advances[first] {
            last += 1;
        }
        let code_point = 0x20 + first;
        let count = last - first + 1;
        let _ = writeln!(out, "{code_point:04X} {count} {}", advances[first]);
        first = last + 1;
    }
    out
}

/// The provenance header, which carries the upstream notice.
fn header(glyph_count: usize) -> String {
    let last = 0x20 + glyph_count - 1;
    format!(
        "\
# Stroke-font glyph advances, derived from KiCad's Newstroke font data.
#
# Source: {NEWSTROKE_PATH} in KiCad at tag {PINNED_TAG}.
# Copyright (C) 2010 vladimir uryvaev <vovanius@bk.ru>
# Copyright The KiCad Developers, see AUTHORS.txt for contributors.
#
# Newstroke is free software; you can redistribute it and/or modify it under
# the terms of the GNU General Public License as published by the Free Software
# Foundation; either version 2 of the License, or (at your option) any later
# version. It is distributed in the hope that it will be useful, but WITHOUT
# ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
# FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
# details: http://www.gnu.org/licenses/old-licenses/gpl-2.0.html
#
# Generated by `cargo xtask text-metrics`. Do not edit by hand.
#
# A glyph's advance is the width in the first two bytes of its Newstroke entry,
# over the font scale of {FONT_SCALE}. At text width w the advance in internal units is
# round(numerator * w / {FONT_SCALE}).
#
# The font holds {glyph_count} glyphs, for code points 0020 to {last:04X}. A code point
# outside that range draws the substitution glyph, which is '?'.
#
# One record per run of equal advances: first code point in hexadecimal, how
# many code points the run covers, then the advance numerator.
"
    )
}

#[cfg(test)]
mod tests {
    use super::{parse_advances, parse_entries, render_table};

    /// A fragment shaped like the real font source, comments and all.
    const SAMPLE: &str = r#"
const char* const newstroke_font[] =
{
    /* // BASIC LATIN (0020-007F) */
    "JZ", /* U+20 SPACE  */
    "MWRYSZR[QZRYR[ RRSQGRFSGRSRF",
    "H\\LZO[T[VZWYXWXUWSVRTQPPNOMNLLLJMHNGPFUFXG RRCR^",
    "JZ",
};
"#;

    #[test]
    fn glyph_entries_survive_comments_and_escapes() {
        let entries = parse_entries(SAMPLE).expect("the sample parses");
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0], "JZ");
        // The escaped backslash decodes to one byte, so the width reads from
        // 'H' and '\\'.
        assert!(entries[2].starts_with("H\\LZ"));
    }

    #[test]
    fn an_advance_is_the_width_of_the_first_two_bytes() {
        let entries = parse_entries(SAMPLE).expect("the sample parses");
        let advance = |entry: &str| {
            let bytes = entry.as_bytes();
            i32::from(bytes[1]) - i32::from(bytes[0])
        };
        // 'Z' - 'J' is 16, which is the space glyph's advance numerator.
        assert_eq!(advance(&entries[0]), 16);
        // 'W' - 'M' is 10.
        assert_eq!(advance(&entries[1]), 10);
    }

    #[test]
    fn a_source_without_the_array_is_reported() {
        assert!(parse_advances("int main() { return 0; }").is_err());
    }

    #[test]
    fn equal_advances_collapse_into_one_run() {
        // Two runs: three glyphs of 16, then one of 10.
        let table = render_table(&[16, 16, 16, 10]);
        let records: Vec<&str> = table.lines().filter(|l| !l.starts_with('#')).collect();
        assert_eq!(records, vec!["0020 3 16", "0023 1 10"]);
    }
}
