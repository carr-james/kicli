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

use kicli_probe::oracle::Kicad;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use kicli::geometry::font::{DEFAULT_PEN_WIDTH, string_extents};
use kicli::geometry::text::{HorizontalJustify, TextStyle, VerticalJustify, text_box};
use kicli::geometry::{Angle, Iu, Point, Size};
use kicli::model::{Item, Schematic};
use kicli_sexpr::Doc;

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
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/text/calibration.textlength")
}

/// One text item of the calibration sheet.
struct CalibrationItem {
    /// The text, which may hold line breaks.
    text: String,
    /// Where the item is drawn from.
    at: Point,
    /// How far the item is turned.
    angle: Angle,
    /// Everything else the box depends on.
    style: TextStyle,
}

impl CalibrationItem {
    /// The pen KiCad draws this item with.
    fn pen(&self) -> Iu {
        self.style.pen_width(DEFAULT_PEN_WIDTH)
    }

    /// The item split into the lines KiCad measures one by one.
    fn lines(&self) -> Vec<Line> {
        self.text
            .split('\n')
            .map(|line| Line {
                text: line.to_owned(),
                size: self.style.size,
                pen: self.pen(),
            })
            .collect()
    }
}

/// Every text item of the calibration sheet, in file order.
fn calibration_items() -> Vec<CalibrationItem> {
    let source = std::fs::read_to_string(fixture("calibration.kicad_sch"))
        .expect("the calibration sheet is readable");
    let doc = Doc::parse(&source).expect("the calibration sheet parses");
    let schematic = Schematic::read(&doc).expect("the calibration sheet reads");

    let items: Vec<CalibrationItem> = schematic
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Text(text) => Some(CalibrationItem {
                text: text.text.clone(),
                at: text.at,
                angle: text.angle,
                style: TextStyle::read(&doc, text.node),
            }),
            _ => None,
        })
        .collect();
    assert!(!items.is_empty(), "the calibration sheet has no text");
    items
}

/// Every line of every text item on the calibration sheet, in file order.
fn calibration_lines() -> Vec<Line> {
    calibration_items()
        .iter()
        .flat_map(CalibrationItem::lines)
        .collect()
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
fn text_boxes_match_kicad_extents() {
    let oracle = read_oracle(
        &std::fs::read_to_string(oracle_path()).expect("the committed measurements are readable"),
    );
    let items = calibration_items();
    let width_of = |line: &Line| {
        *oracle
            .get(line)
            .unwrap_or_else(|| panic!("KiCad measured no {:?}", line.text))
    };

    let mut multiline = 0;
    let mut bold = 0;
    let mut italic = 0;
    let mut overbar = 0;

    for item in &items {
        let lines = item.lines();
        // A box is as wide as its widest line, which is how KiCad merges them.
        let widest = lines
            .iter()
            .map(width_of)
            .max()
            .expect("the item has a line");
        let boxed = text_box(&item.text, item.at, item.angle, &item.style);
        assert!(
            (boxed.bounds().width().0 - widest.0).abs() <= TOLERANCE,
            "{:?}: kicli boxes {} wide, KiCad measures {widest}",
            item.text,
            boxed.bounds().width()
        );

        // Every line adds its own height, so a box is never shorter than its
        // text.
        assert!(boxed.bounds().height() >= item.style.size.y);
        if lines.len() > 1 {
            multiline += 1;
            let single = text_box(&lines[0].text, item.at, item.angle, &item.style);
            assert!(
                boxed.bounds().height() > single.bounds().height(),
                "{:?} is not taller than its first line",
                item.text
            );
        }
        if item.style.bold {
            bold += 1;
        }
        if item.style.italic {
            italic += 1;
        }
        if item.text.contains("~{") {
            overbar += 1;
            let plain = item.text.replace("~{", "").replace('}', "");
            let bare = text_box(&plain, item.at, item.angle, &item.style);
            assert!(
                boxed.bounds().height() > bare.bounds().height(),
                "{:?} leaves no room for its overbar",
                item.text
            );
        }
    }

    assert!(multiline > 0 && bold > 0 && italic > 0 && overbar > 0);
    println!(
        "text boxes: {} items, {multiline} multi-line, {bold} bold, {italic} italic, {overbar} overbarred",
        items.len()
    );
}

#[test]
fn each_justification_puts_the_box_on_its_own_side_of_the_anchor() {
    let anchor = Point::new(100_000, 100_000);
    let boxed = |horizontal, vertical| {
        let style = TextStyle {
            horizontal,
            vertical,
            ..TextStyle::default()
        };
        text_box("Ay", anchor, Angle(0), &style)
    };

    for vertical in [
        VerticalJustify::Top,
        VerticalJustify::Centre,
        VerticalJustify::Bottom,
    ] {
        // Left starts at the anchor, right ends at it, centre straddles it.
        assert_eq!(
            boxed(HorizontalJustify::Left, vertical).bounds().start().x,
            anchor.x
        );
        assert_eq!(
            boxed(HorizontalJustify::Right, vertical).bounds().end().x,
            anchor.x
        );
        let centred = boxed(HorizontalJustify::Centre, vertical);
        assert!((centred.bounds().centre().x.0 - anchor.x.0).abs() <= 1);
    }

    for horizontal in [
        HorizontalJustify::Left,
        HorizontalJustify::Centre,
        HorizontalJustify::Right,
    ] {
        // Top hangs below the anchor, bottom stands above it, centre straddles.
        let top = boxed(horizontal, VerticalJustify::Top);
        let bottom = boxed(horizontal, VerticalJustify::Bottom);
        let centred = boxed(horizontal, VerticalJustify::Centre);
        assert!(top.bounds().end().y > anchor.y);
        assert!(bottom.bounds().start().y < anchor.y);
        assert!(top.bounds().start().y > bottom.bounds().start().y);
        assert!((centred.bounds().centre().y.0 - anchor.y.0).abs() <= 1);
    }

    // A mirrored text swaps the two horizontal cases, which is what a board
    // does to text on its back.
    let mirrored = |horizontal| {
        let style = TextStyle {
            horizontal,
            mirrored: true,
            ..TextStyle::default()
        };
        text_box("Ay", anchor, Angle(0), &style)
    };
    assert_eq!(mirrored(HorizontalJustify::Left).bounds().end().x, anchor.x);
    assert_eq!(
        mirrored(HorizontalJustify::Right).bounds().start().x,
        anchor.x
    );
}

#[test]
fn a_turned_box_is_the_unturned_box_turned_about_the_draw_position() {
    let anchor = Point::new(100_000, 50_000);
    let style = TextStyle {
        horizontal: HorizontalJustify::Left,
        vertical: VerticalJustify::Bottom,
        ..TextStyle::default()
    };

    let flat = text_box("Ay", anchor, Angle(0), &style);
    for angle in [90, 180, 270] {
        let turned = text_box("Ay", anchor, Angle(angle), &style);
        assert_eq!(
            turned.bounds(),
            flat.bounds(),
            "the box itself is not turned"
        );
        let expected = flat
            .bounds()
            .corners()
            .map(|corner| corner.rotated(anchor, Angle(angle)));
        assert_eq!(turned.corners(), expected);
        assert_eq!(
            turned.centre(),
            flat.bounds().centre().rotated(anchor, Angle(angle))
        );
    }

    // At 90 degrees the page-aligned box has the sides the other way round.
    let upright = text_box("Ay", anchor, Angle(90), &style);
    assert_eq!(upright.axis_aligned().width(), flat.bounds().height());
    assert_eq!(upright.axis_aligned().height(), flat.bounds().width());
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
    let Some(tool) = Kicad::found_or_skip("measure with kicad-cli") else {
        return;
    };

    let svg = match export_svg(&tool) {
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
///
/// `-n` drops the page background and `-e` the drawing sheet, so the plot holds
/// the calibration text and nothing else.
fn export_svg(tool: &Kicad) -> Result<String, String> {
    let directory = std::env::temp_dir().join(format!("kicli-text-metrics-{}", std::process::id()));
    let svg = tool.try_svg(&fixture("calibration.kicad_sch"), &directory, &["-n", "-e"]);
    let _ = std::fs::remove_dir_all(&directory);
    svg
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
