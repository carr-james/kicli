//! kicli measures text the way KiCad measures it.
//!
//! The calibration sheet carries one text item per printable ASCII glyph, plus
//! pairs, tabs, overbars and multi-line cases, at three sizes in normal, bold
//! and italic. KiCad's own SVG exporter reports the width of every one of them
//! in a `textLength` attribute, computed by the same font engine the editor
//! draws with. Those widths are committed beside this test as the oracle.
//!
//! The hermetic test asserts the port reproduces the oracle. The env-gated test
//! re-measures with `kicad-cli` and fails when the committed oracle is stale.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use kicli::geometry::font::{DEFAULT_PEN_WIDTH, bold_pen_width, clamp_pen_width, string_extents};
use kicli::geometry::{Iu, Size};
use kicli::model::{Item, Schematic};
use kicli_sexpr::{Doc, NodeId};

/// How far the port may differ from KiCad's own measurement.
///
/// The port should agree exactly. The tolerance covers the four decimal places
/// the SVG writer prints, not a modelling gap.
const TOLERANCE: i32 = 1;

/// One line of text, with everything its width depends on.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Line {
    /// The text of the line, markup and all.
    text: String,
    /// The text size.
    size: Size,
    /// The pen KiCad draws it with.
    pen: Iu,
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/text")
        .join(name)
}

/// Where the committed measurements live.
fn oracle_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/text_extents.expected")
}

/// Every line of every text item on the calibration sheet, in file order.
fn calibration_lines() -> Vec<Line> {
    let source = std::fs::read_to_string(fixture("calibration.kicad_sch"))
        .expect("the calibration sheet is readable");
    let doc = Doc::parse(&source).expect("the calibration sheet parses");
    let schematic = Schematic::read(&doc).expect("the calibration sheet reads");

    let mut lines = Vec::new();
    for item in &schematic.items {
        let Item::Text(text) = item else {
            continue;
        };
        let (size, bold) = effects_of(&doc, text.node);
        let pen = if bold {
            bold_pen_width(size.x)
        } else {
            DEFAULT_PEN_WIDTH
        };
        let pen = clamp_pen_width(pen, size);
        for line in text.text.split('\n') {
            lines.push(Line {
                text: line.to_owned(),
                size,
                pen,
            });
        }
    }
    assert!(!lines.is_empty(), "the calibration sheet has no text");
    lines
}

/// The size and boldness of a text item, read from its `effects`.
fn effects_of(doc: &Doc, node: NodeId) -> (Size, bool) {
    let mut size = Size::new(12_700, 12_700);
    let mut bold = false;
    for &child in doc.children(node) {
        if !doc.head_is(child, "effects") {
            continue;
        }
        for &effect in doc.children(child) {
            if !doc.head_is(effect, "font") {
                continue;
            }
            for &setting in doc.children(effect) {
                if doc.head_is(setting, "size") {
                    let values = doc.children(setting);
                    let read = |index: usize| {
                        values
                            .get(index)
                            .and_then(|&id| doc.atom_as_iu(id))
                            .map(Iu)
                            .unwrap_or_default()
                    };
                    // KiCad writes the size as height then width.
                    size = Size {
                        x: read(2),
                        y: read(1),
                    };
                }
                if doc.head_is(setting, "bold") {
                    bold = doc
                        .children(setting)
                        .get(1)
                        .and_then(|&id| doc.atom_text(id))
                        != Some("no");
                }
            }
        }
    }
    (size, bold)
}

/// Read the committed measurements.
fn read_oracle(text: &str) -> BTreeMap<Line, Iu> {
    let mut measurements = BTreeMap::new();
    for record in text.lines() {
        if record.starts_with('#') || record.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = record.splitn(5, ' ').collect();
        assert_eq!(fields.len(), 5, "record needs five fields: {record}");
        let number = |field: &str| Iu(field.parse().expect("a field is an integer"));
        let line = Line {
            text: unescape(fields[4]),
            size: Size {
                x: number(fields[0]),
                y: number(fields[1]),
            },
            pen: number(fields[2]),
        };
        measurements.insert(line, number(fields[3]));
    }
    measurements
}

/// Write the measurements, sorted, so two runs produce identical bytes.
fn render_oracle(measurements: &BTreeMap<Line, Iu>) -> String {
    let mut out = String::from(
        "\
# Text widths measured by KiCad, for the calibration sheet beside this file.
#
# KiCad's SVG exporter writes the width of every text item it plots. The
# widths below were read from that output, in internal units of 100 nm. They
# are the oracle kicli's own text metrics must reproduce.
#
# One record per distinct line of text: text width, text height, pen width,
# measured width, then the text. A tab is written \\t and a backslash \\\\.
#
# Regenerate with: KICLI_TEST_KICAD_CLI=1 cargo test advances_reproduce_kicad_svg
",
    );
    for (line, width) in measurements {
        let _ = writeln!(
            out,
            "{} {} {} {} {}",
            line.size.x.0,
            line.size.y.0,
            line.pen.0,
            width.0,
            escape(&line.text)
        );
    }
    out
}

/// Write a text so it fits on one record.
fn escape(text: &str) -> String {
    text.replace('\\', "\\\\").replace('\t', "\\t")
}

/// Read back an escaped text.
fn unescape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut characters = text.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            out.push(character);
            continue;
        }
        match characters.next() {
            Some('t') => out.push('\t'),
            Some('\\') => out.push('\\'),
            other => {
                out.push('\\');
                out.extend(other);
            }
        }
    }
    out
}

#[test]
fn string_extents_match_kicad_measurements() {
    let oracle = read_oracle(
        &std::fs::read_to_string(oracle_path()).expect("the committed measurements are readable"),
    );
    let lines = calibration_lines();

    let mut worst: i32 = 0;
    let mut worst_line = String::new();
    let mut seen = BTreeMap::new();
    for line in &lines {
        let measured = oracle.get(line).unwrap_or_else(|| {
            panic!(
                "KiCad measured no {:?} at {} pen {}",
                line.text, line.size, line.pen
            )
        });
        let computed = string_extents(&line.text, line.size, line.pen).x;
        let residual = computed.0 - measured.0;
        if residual.abs() > worst.abs() {
            worst = residual;
            worst_line.clone_from(&line.text);
        }
        assert!(
            residual.abs() <= TOLERANCE,
            "{:?} at {} pen {}: kicli says {computed}, KiCad says {measured}",
            line.text,
            line.size,
            line.pen
        );
        seen.insert(line.clone(), ());
    }

    println!(
        "text metrics: {} lines checked, largest residual {worst} IU on {worst_line:?}",
        lines.len()
    );
    assert_eq!(
        seen.len(),
        oracle.len(),
        "the committed measurements hold records the calibration sheet no longer has"
    );
}

#[test]
fn advances_reproduce_kicad_svg() {
    if std::env::var_os("KICLI_TEST_KICAD_CLI").is_none() {
        println!("skipped: set KICLI_TEST_KICAD_CLI=1 to measure with kicad-cli");
        return;
    }

    let svg = match export_svg() {
        Ok(svg) => svg,
        Err(reason) => {
            println!("skipped: {reason}");
            return;
        }
    };

    let measured = read_svg_widths(&svg);
    let mut oracle = BTreeMap::new();
    let mut worst: i32 = 0;
    let mut worst_line = String::new();

    for line in calibration_lines() {
        // The SVG names a text by its content, its font size and the pen of the
        // group that holds it. The font size is the text width times 4/3,
        // truncated, as `SVG_PLOTTER::Text` writes it.
        let key = (line.text.clone(), Iu(line.size.x.0 * 4 / 3), line.pen);
        let width = *measured
            .get(&key)
            .unwrap_or_else(|| panic!("the SVG has no {key:?}"));
        let computed = string_extents(&line.text, line.size, line.pen).x;
        let residual = computed.0 - width.0;
        if residual.abs() > worst.abs() {
            worst = residual;
            worst_line.clone_from(&line.text);
        }
        assert!(
            residual.abs() <= TOLERANCE,
            "{:?} at {} pen {}: kicli says {computed}, KiCad says {width}",
            line.text,
            line.size,
            line.pen
        );
        oracle.insert(line, width);
    }

    println!("text metrics: largest residual against kicad-cli is {worst} IU on {worst_line:?}");

    let rendered = render_oracle(&oracle);
    let committed = std::fs::read_to_string(oracle_path()).unwrap_or_default();
    if rendered != committed {
        std::fs::write(oracle_path(), &rendered).expect("the measurements are writable");
        panic!("the committed measurements were stale; they have been rewritten");
    }
}

/// Plot the calibration sheet to SVG with `kicad-cli`.
fn export_svg() -> Result<String, String> {
    let directory = std::env::temp_dir().join(format!("kicli-text-metrics-{}", std::process::id()));
    std::fs::create_dir_all(&directory)
        .map_err(|error| format!("cannot make a directory: {error}"))?;

    let output = std::process::Command::new("kicad-cli")
        .args(["sch", "export", "svg", "-n", "-e", "-o"])
        .arg(&directory)
        .arg(fixture("calibration.kicad_sch"))
        .output()
        .map_err(|error| format!("cannot run kicad-cli: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "kicad-cli refused the calibration sheet: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let plotted = directory.join("calibration.svg");
    let svg = std::fs::read_to_string(&plotted)
        .map_err(|error| format!("cannot read {}: {error}", plotted.display()))?;
    let _ = std::fs::remove_dir_all(&directory);
    Ok(svg)
}

/// Read every text width the SVG reports.
///
/// Each text is emitted twice: once as an invisible string carrying the width,
/// then as stroke paths. The pen comes from the group that holds it, which the
/// plotter emits only when it changes.
fn read_svg_widths(svg: &str) -> BTreeMap<(String, Iu, Iu), Iu> {
    let mut widths = BTreeMap::new();
    let mut pen = Iu(0);
    let mut rest = svg;

    while !rest.is_empty() {
        let next_pen = rest.find("stroke-width:");
        let next_text = rest.find("<text ");
        match (next_pen, next_text) {
            (Some(at), text_at) if text_at.is_none_or(|other| at < other) => {
                let value = &rest[at + "stroke-width:".len()..];
                let end = value.find(';').unwrap_or(value.len());
                pen = Iu::from_millimetres_text(&value[..end]).expect("a pen is a number");
                rest = &value[end..];
            }
            (_, Some(at)) => {
                let element = &rest[at..];
                let close = element.find("</text>").expect("a text element closes");
                let (element, remainder) = element.split_at(close);
                let width = attribute(element, "textLength");
                let font_size = attribute(element, "font-size");
                let content = element
                    .split_once('>')
                    .map(|(_, content)| unescape_xml(content))
                    .expect("a text element has content");
                if let Some(seen) = widths.insert((content.clone(), font_size, pen), width) {
                    assert_eq!(
                        seen, width,
                        "the SVG reports two widths for {content:?} at {font_size} pen {pen}"
                    );
                }
                rest = remainder;
            }
            _ => break,
        }
    }
    widths
}

/// Read a numeric attribute of an element, in internal units.
fn attribute(element: &str, name: &str) -> Iu {
    let marker = format!("{name}=\"");
    let start = element
        .find(&marker)
        .unwrap_or_else(|| panic!("an element has no {name}"))
        + marker.len();
    let value = &element[start..];
    let end = value.find('"').expect("an attribute closes");
    Iu::from_millimetres_text(&value[..end]).expect("an attribute is a number")
}

/// Undo the XML escaping the plotter applies to a text.
fn unescape_xml(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}
