//! The layout digest: where things are drawn, without the connections.
//!
//! Coordinates are millimetres to two decimals, derived from the integer units
//! the geometry works in. Two decimals is one place finer than the grid needs
//! and three digits shorter than the file writes.
//!
//! Wires are summarised rather than listed. Five hundred segments is twenty
//! kilobytes of noise, and what a reader wants from them is the routing
//! quality: how many there are, how many junctions, and how often they cross.

use std::fmt::Write as _;

use crate::geometry::{Iu, Point, symbol_boxes};
use crate::model::items::{Item, LabelKind, LineKind, SheetPath};
use crate::model::{Hierarchy, LoadedFile, definition_of, read_library};
use crate::view::connectivity::ViewOptions;

/// Millimetres, to two decimals, as a digest writes them.
fn mm(value: Iu) -> String {
    let hundredths =
        (i64::from(value.0) * 100).div_euclid(i64::from(crate::geometry::UNITS_PER_MM));
    let whole = hundredths / 100;
    let fraction = (hundredths % 100).abs();
    let sign = if hundredths < 0 && whole == 0 {
        "-"
    } else {
        ""
    };
    format!("{sign}{whole}.{fraction:02}")
}

/// A point, as a digest writes it.
fn point(at: Point) -> String {
    format!("{} {}", mm(at.x), mm(at.y))
}

/// Is a point on a segment, ends included? Exact integer arithmetic.
fn on_segment(point: Point, segment: (Point, Point)) -> bool {
    let (a, b) = segment;
    let cross = i64::from(b.x.0 - a.x.0) * i64::from(point.y.0 - a.y.0)
        - i64::from(b.y.0 - a.y.0) * i64::from(point.x.0 - a.x.0);
    if cross != 0 {
        return false;
    }
    let within =
        |value: i32, one: i32, other: i32| value >= one.min(other) && value <= one.max(other);
    within(point.x.0, a.x.0, b.x.0) && within(point.y.0, a.y.0, b.y.0)
}

/// Do two segments cross at a point that is an end of neither?
///
/// Exact integer orientation tests. A crossing is what a reader is counting,
/// and a floating-point near-miss would make the count depend on the machine.
fn crosses(first: (Point, Point), second: (Point, Point)) -> bool {
    let orientation = |a: Point, b: Point, c: Point| -> i64 {
        let left = i64::from(b.x.0 - a.x.0) * i64::from(c.y.0 - a.y.0);
        let right = i64::from(b.y.0 - a.y.0) * i64::from(c.x.0 - a.x.0);
        (left - right).signum()
    };
    let (p1, p2) = first;
    let (q1, q2) = second;
    // A shared endpoint is a join, not a crossing.
    if p1 == q1 || p1 == q2 || p2 == q1 || p2 == q2 {
        return false;
    }
    let d1 = orientation(p1, p2, q1);
    let d2 = orientation(p1, p2, q2);
    let d3 = orientation(q1, q2, p1);
    let d4 = orientation(q1, q2, p2);
    d1 != d2 && d3 != d4 && d1 != 0 && d2 != 0 && d3 != 0 && d4 != 0
}

/// Render the layout digest of a loaded project.
#[must_use]
pub fn render(hierarchy: &Hierarchy, options: &ViewOptions) -> String {
    let mut out = String::new();
    let scope = match &options.sheet {
        Some(path) => format!("sheet {}", path.0),
        None => "project".to_owned(),
    };
    let _ = writeln!(out, "# scope {scope}");

    for placement in &hierarchy.placements {
        if options
            .sheet
            .as_ref()
            .is_some_and(|wanted| &placement.path != wanted)
        {
            continue;
        }
        let file = &hierarchy.files[placement.file];
        render_sheet(&mut out, file, &placement.path, options);
    }
    out
}

fn render_sheet(out: &mut String, file: &LoadedFile, path: &SheetPath, options: &ViewOptions) {
    let schematic = &file.schematic;
    let library = read_library(&file.doc, &schematic.library_symbols, schematic.version);

    let paper = schematic.paper.clone().unwrap_or_else(|| "?".to_owned());
    let _ = writeln!(out, "page {paper}  sheet {}", path.0);

    // L: one placement per line, with the orientation as the file writes it,
    // because that is what a caller passes back to a rotate command.
    let mut placements: Vec<(String, String)> = Vec::new();
    for symbol in schematic.symbols() {
        let Some(reference) = symbol.reference_on(path) else {
            continue;
        };
        if symbol.is_power() && !options.include_power {
            continue;
        }
        let mirror = match symbol.mirror {
            Some(crate::model::items::Mirror::X) => "x",
            Some(crate::model::items::Mirror::Y) => "y",
            None => "-",
        };
        let size = definition_of(&library, symbol).map(|definition| {
            let boxes = symbol_boxes(&file.doc, symbol, definition);
            format!(" {}x{}", mm(boxes.body.width()), mm(boxes.body.height()))
        });
        placements.push((
            reference.0.clone(),
            format!(
                "L {} {} {} {mirror}{}",
                reference.0,
                point(symbol.at),
                symbol.angle.0,
                size.unwrap_or_default()
            ),
        ));
    }
    placements.sort_by(|left, right| {
        super::connectivity::natural_key(&left.0).cmp(&super::connectivity::natural_key(&right.0))
    });
    for (_, line) in placements {
        let _ = writeln!(out, "{line}");
    }

    write_fields(out, file, path, options);
    write_text(out, file);
    write_wires(out, file);
}

/// Write the fields that have moved off the position their library gives them.
fn write_fields(out: &mut String, file: &LoadedFile, path: &SheetPath, options: &ViewOptions) {
    let schematic = &file.schematic;
    let library = read_library(&file.doc, &schematic.library_symbols, schematic.version);
    // F: a field is listed only when it has been moved off the position the
    // library gives it. On a tidy sheet that is nothing; on an untidy one it is
    // exactly the list of things to put back.
    for symbol in schematic.symbols() {
        let Some(reference) = symbol.reference_on(path) else {
            continue;
        };
        if symbol.is_power() && !options.include_power {
            continue;
        }
        let Some(definition) = definition_of(&library, symbol) else {
            continue;
        };
        for field in &symbol.fields {
            if field.hidden {
                continue;
            }
            let Some(default) = definition
                .fields
                .iter()
                .find(|candidate| candidate.name == field.name)
            else {
                continue;
            };
            let offset = Point {
                x: Iu(field.at.x.0 - symbol.at.x.0),
                y: Iu(field.at.y.0 - symbol.at.y.0),
            };
            // A library field's position is in library space, where Y grows
            // upwards, and the instance offset is in schematic space, where it
            // grows downwards. Comparing them without the flip reports every
            // field above the anchor as moved and every field below it as
            // placed, which is worse than reporting nothing.
            let library_default = Point {
                x: default.at.x,
                y: Iu(-default.at.y.0),
            };
            if offset == library_default {
                continue;
            }
            let _ = writeln!(
                out,
                "F {}.{} {} {}",
                reference.0,
                field.name,
                point(offset),
                field.angle.0
            );
        }
    }
}

/// Write the labels and the free text.
fn write_text(out: &mut String, file: &LoadedFile) {
    let schematic = &file.schematic;
    // T: labels and free text, which is what a reader looks for when a net has
    // no obvious source. The kind is written with the word `label add --kind`
    // takes, so a kind read out of a view goes straight back into a command.
    for label in schematic.labels() {
        let kind = match label.kind {
            LabelKind::Local => "local",
            LabelKind::Global => "global",
            LabelKind::Hierarchical => "hierarchical",
            LabelKind::NetclassFlag => "netclass",
        };
        let _ = writeln!(
            out,
            "T {kind} {} {} {}",
            label.text,
            point(label.at),
            label.angle.0
        );
    }
    for item in &schematic.items {
        if let Item::Text(text) = item {
            let _ = writeln!(
                out,
                "T text {} {} {}",
                text.text.replace('\n', "\\n"),
                point(text.at),
                text.angle.0
            );
        }
    }
}

/// Write the one-line wire summary.
fn write_wires(out: &mut String, file: &LoadedFile) {
    let schematic = &file.schematic;
    // W: the summary. --wires lists them instead.
    let segments: Vec<(Point, Point)> = schematic
        .lines()
        .filter(|line| line.kind == LineKind::Wire)
        .map(|line| (line.from, line.to))
        .collect();
    let junction_points: Vec<Point> = schematic.junctions().map(|junction| junction.at).collect();
    let mut crossings = 0;
    for (index, first) in segments.iter().enumerate() {
        for second in &segments[index + 1..] {
            if !crosses(*first, *second) {
                continue;
            }
            // A crossing with a junction on it is a connection, and the
            // linter's crossing rule excludes it for the same reason. Testing
            // for a junction on both segments avoids computing the crossing
            // point, which would not be an integer in general.
            let joined = junction_points
                .iter()
                .any(|&point| on_segment(point, *first) && on_segment(point, *second));
            if !joined {
                crossings += 1;
            }
        }
    }
    let _ = writeln!(
        out,
        "W {} segments, {} junctions, {crossings} crossings",
        segments.len(),
        junction_points.len()
    );
}

/// The same content as [`render`], as JSON.
#[must_use]
pub fn to_json(hierarchy: &Hierarchy, options: &ViewOptions) -> serde_json::Value {
    let text = render(hierarchy, options);
    let mut sheets: Vec<serde_json::Value> = Vec::new();

    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("page ") {
            let mut parts = rest.split_whitespace();
            let paper = parts.next().unwrap_or_default();
            let path = parts.nth(1).unwrap_or_default();
            sheets.push(serde_json::json!({
                "paper": paper,
                "path": path,
                "symbols": [],
                "fields": [],
                "text": [],
                "wires": serde_json::Value::Null,
            }));
        } else if let Some(sheet) = sheets.last_mut() {
            push_record(sheet, line);
        }
    }
    serde_json::json!({
        "scope": if options.sheet.is_some() { "sheet" } else { "project" },
        "sheets": sheets,
    })
}

/// Add one record line to the sheet object it belongs to.
fn push_record(sheet: &mut serde_json::Value, line: &str) {
    let mut parts = line.split_whitespace();
    match parts.next() {
        Some("L") => {
            let record = serde_json::json!({
                "reference": parts.next().unwrap_or_default(),
                "x": parts.next().unwrap_or_default(),
                "y": parts.next().unwrap_or_default(),
                "angle": parts.next().unwrap_or_default(),
                "mirror": parts.next().unwrap_or_default(),
                "size": parts.next(),
            });
            if let Some(list) = sheet["symbols"].as_array_mut() {
                list.push(record);
            }
        }
        Some("F") => {
            let record = serde_json::json!({
                "field": parts.next().unwrap_or_default(),
                "dx": parts.next().unwrap_or_default(),
                "dy": parts.next().unwrap_or_default(),
                "angle": parts.next().unwrap_or_default(),
            });
            if let Some(list) = sheet["fields"].as_array_mut() {
                list.push(record);
            }
        }
        Some("T") => {
            let kind = parts.next().unwrap_or_default().to_owned();
            let rest: Vec<&str> = parts.collect();
            // The text itself may hold spaces, so the three trailing numbers
            // are taken from the end and everything before them is the text.
            let split = rest.len().saturating_sub(3);
            let record = serde_json::json!({
                "kind": kind,
                "text": rest[..split].join(" "),
                "x": rest.get(split).copied().unwrap_or_default(),
                "y": rest.get(split + 1).copied().unwrap_or_default(),
                "angle": rest.get(split + 2).copied().unwrap_or_default(),
            });
            if let Some(list) = sheet["text"].as_array_mut() {
                list.push(record);
            }
        }
        Some("W") => {
            let numbers: Vec<u64> = line
                .split_whitespace()
                .filter_map(|word| word.parse().ok())
                .collect();
            sheet["wires"] = serde_json::json!({
                "segments": numbers.first().copied().unwrap_or(0),
                "junctions": numbers.get(1).copied().unwrap_or(0),
                "crossings": numbers.get(2).copied().unwrap_or(0),
            });
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{crosses, mm};
    use crate::geometry::{Iu, Point};

    #[test]
    fn a_millimetre_reading_keeps_two_decimals() {
        assert_eq!(mm(Iu(12_700)), "1.27");
        assert_eq!(mm(Iu(0)), "0.00");
        assert_eq!(mm(Iu(-38_100)), "-3.81");
        assert_eq!(mm(Iu(1_016_000)), "101.60");
        // Two decimals is finer than the grid, so nothing on grid is lost.
        assert_eq!(mm(Iu(-5_080)), "-0.51");
    }

    #[test]
    fn a_point_is_on_a_segment_or_it_is_not() {
        let segment = (Point::new(0, 0), Point::new(100, 0));
        assert!(super::on_segment(Point::new(50, 0), segment));
        assert!(super::on_segment(Point::new(0, 0), segment), "ends count");
        assert!(!super::on_segment(Point::new(150, 0), segment));
        assert!(!super::on_segment(Point::new(50, 1), segment));
    }

    #[test]
    fn a_crossing_is_not_a_join() {
        let horizontal = (Point::new(0, 0), Point::new(100, 0));
        let vertical = (Point::new(50, -50), Point::new(50, 50));
        assert!(crosses(horizontal, vertical), "they cross in the middle");

        // Meeting at an end is a join, however the two are drawn.
        let meeting = (Point::new(100, 0), Point::new(100, 50));
        assert!(!crosses(horizontal, meeting));

        // A T, where one segment ends on the other's interior, is not counted
        // as a crossing either: it is either a junction or a fault, and both
        // are somebody else's finding.
        let tee = (Point::new(50, 0), Point::new(50, 50));
        assert!(!crosses(horizontal, tee));

        let apart = (Point::new(0, 10), Point::new(100, 10));
        assert!(!crosses(horizontal, apart));
    }
}
