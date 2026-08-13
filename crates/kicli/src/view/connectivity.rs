//! The connectivity view: what is joined to what, without any coordinates.
//!
//! One record per line, each line starting with a one-letter type, so an agent
//! can filter with `grep` and never needs a parser. The view mentions no
//! positions at all: most tasks need either the connections or the geometry,
//! and paying for both is how a context budget is wasted.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use crate::connectivity::Nets;
use crate::model::items::{Item, LabelKind, LibId, SheetPath, Symbol};
use crate::model::{Hierarchy, definition_of, read_library};

/// What to put in a view, and what to leave out.
#[derive(Clone, Debug, Default)]
pub struct ViewOptions {
    /// List power symbols as symbols. They are net-name carriers, so a list of
    /// sixteen `#PWR` entries is noise, and they are left out by default.
    pub include_power: bool,
    /// Append `@<uuid8>` to every record, for objects with no reference
    /// designator. It costs about a quarter of the view's size.
    pub uuids: bool,
    /// Restrict the view to one placement of one sheet.
    pub sheet: Option<SheetPath>,
}

/// The first eight characters of an identifier, which is how a view prints one.
fn short(uuid: &str) -> &str {
    uuid.get(..8).unwrap_or(uuid)
}

/// Sort key that orders `R2` before `R10`.
///
/// A plain string sort puts `R10` first, which makes a view read as though the
/// designer numbered parts at random.
pub(crate) fn natural_key(reference: &str) -> (String, u64, String) {
    let head: String = reference
        .chars()
        .take_while(|character| !character.is_ascii_digit())
        .collect();
    let rest = &reference[head.len()..];
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    let tail = rest[digits.len()..].to_owned();
    (head, digits.parse().unwrap_or(0), tail)
}

/// One line of the port list of a sheet, such as `IN(i)`.
fn port_letter(direction: &str) -> char {
    match direction {
        "output" => 'o',
        "bidirectional" => 'b',
        "tri_state" => 't',
        "passive" => 'p',
        _ => 'i',
    }
}

/// Render the connectivity view of a loaded project.
///
/// The view names its own scope on the first line, so a reader always knows
/// whether it covers one sheet or the whole project.
#[must_use]
pub fn render(hierarchy: &Hierarchy, nets: &Nets, options: &ViewOptions) -> String {
    let placements: Vec<usize> = (0..hierarchy.placements.len())
        .filter(|&index| {
            options
                .sheet
                .as_ref()
                .is_none_or(|wanted| &hierarchy.placements[index].path == wanted)
        })
        .collect();

    let mut out = String::new();
    let mut symbol_total = 0;
    let mut power_total = 0;
    let mut sheet_blocks = String::new();

    for &index in &placements {
        let (symbols, power) = write_sheet(&mut sheet_blocks, hierarchy, index, options);
        symbol_total += symbols;
        power_total += power;
    }

    let nets_shown = write_nets(hierarchy, nets, options, &placements);

    let scope = match &options.sheet {
        Some(path) => format!("sheet {}", path.0),
        None => "project".to_owned(),
    };
    let _ = writeln!(
        out,
        "# scope {scope}  sheets={} sym={symbol_total} pwr={power_total} nets={}",
        placements.len(),
        nets_shown
            .lines()
            .filter(|line| line.starts_with("N "))
            .count()
    );
    out.push_str(&sheet_blocks);
    out.push_str(&nets_shown);
    out
}

/// Write one placement's header and symbol records.
///
/// Returns how many symbols were listed and how many power symbols the sheet
/// holds, listed or not.
fn write_sheet(
    out: &mut String,
    hierarchy: &Hierarchy,
    index: usize,
    options: &ViewOptions,
) -> (usize, usize) {
    let placement = &hierarchy.placements[index];
    let file = &hierarchy.files[placement.file];
    let library = read_library(
        &file.doc,
        &file.schematic.library_symbols,
        file.schematic.version,
    );

    let mut rows: Vec<(String, String, String, String)> = Vec::new();
    let mut power = 0;
    for symbol in file.schematic.symbols() {
        let Some(reference) = symbol.reference_on(&placement.path) else {
            continue;
        };
        if symbol.is_power() {
            power += 1;
            if !options.include_power {
                continue;
            }
        }
        rows.push((
            reference.0.clone(),
            symbol
                .field("Value")
                .map(|field| field.value.clone())
                .unwrap_or_default(),
            definition_of(&library, symbol).map_or_else(
                || symbol.lib_id.symbol_name().to_owned(),
                |found| LibId(found.name.clone()).symbol_name().to_owned(),
            ),
            symbol.uuid.0.clone(),
        ));
    }
    rows.sort_by_key(|row| natural_key(&row.0));

    let name = placement.name.as_deref().unwrap_or("/");
    let _ = writeln!(
        out,
        "sheet {} {name} sym={} pwr={power}",
        placement.path.0,
        rows.len()
    );
    let listed = rows.len();
    for (reference, value, library_name, uuid) in rows {
        let tail = if options.uuids {
            format!(" @{}", short(&uuid))
        } else {
            String::new()
        };
        let _ = writeln!(out, "S {reference} {value} {library_name}{tail}");
    }
    write_ports(out, hierarchy, index, options);
    (listed, power)
}

/// Write the hierarchy ports of a placement: its child sheets, and its own
/// hierarchical labels.
fn write_ports(out: &mut String, hierarchy: &Hierarchy, index: usize, options: &ViewOptions) {
    let placement = &hierarchy.placements[index];
    let file = &hierarchy.files[placement.file];

    for item in &file.schematic.items {
        let Item::Sheet(sheet) = item else { continue };
        if sheet.pins.is_empty() {
            continue;
        }
        let ports: Vec<String> = sheet
            .pins
            .iter()
            .map(|pin| format!("{}({})", pin.name, port_letter(&pin.direction)))
            .collect();
        let tail = if options.uuids {
            format!(" @{}", short(&sheet.uuid.0))
        } else {
            String::new()
        };
        let _ = writeln!(
            out,
            "H {}: {}{tail}",
            sheet.name().unwrap_or("?"),
            ports.join(" ")
        );
    }

    for label in file.schematic.labels() {
        if label.kind != LabelKind::Hierarchical {
            continue;
        }
        let direction = label.shape.as_deref().unwrap_or("input");
        let tail = if options.uuids {
            format!(" @{}", short(&label.uuid.0))
        } else {
            String::new()
        };
        let _ = writeln!(out, "P {}({}){tail}", label.text, port_letter(direction));
    }
}

/// Write the net records, and return them.
///
/// Two nets can carry the same drawn name on different placements of one sheet,
/// because a hierarchical label is local to its placement. A name that is not
/// unique is qualified with the sheet it belongs to, so that every name in a
/// view addresses exactly one net.
fn write_nets(
    hierarchy: &Hierarchy,
    nets: &Nets,
    options: &ViewOptions,
    placements: &[usize],
) -> String {
    let visible: BTreeSet<&SheetPath> = placements
        .iter()
        .map(|&index| &hierarchy.placements[index].path)
        .collect();
    let sheet_names: BTreeMap<&SheetPath, &str> = hierarchy
        .placements
        .iter()
        .map(|placement| (&placement.path, placement.name.as_deref().unwrap_or("/")))
        .collect();

    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for net in nets.nets() {
        *counts.entry(net.name.as_str()).or_default() += 1;
    }

    let mut out = String::new();
    let mut unconnected = 0;
    out.push_str("# N name[=kicad-name]: pins\n");
    for net in nets.nets() {
        let pins: Vec<String> = net
            .pins
            .iter()
            .filter(|pin| options.include_power || !pin.power)
            .filter(|pin| options.sheet.is_none() || visible.contains(&pin.sheet))
            .map(super::super::connectivity::NetPin::label)
            .collect();
        if pins.is_empty() {
            continue;
        }
        // One pin joined to nothing is not a connection. KiCad names those
        // nets `unconnected-...`, the rule check reports every one of them,
        // and listing them here costs a fifth of the view to say nothing.
        if pins.len() == 1 && net.kicad_name.starts_with("unconnected-") {
            unconnected += 1;
            continue;
        }

        let ambiguous = counts.get(net.name.as_str()).copied().unwrap_or(0) > 1;
        let name = if ambiguous {
            // Qualify with the sheet the name comes from, which is the named
            // one. A hierarchical label is local to one placement, so the root
            // sheet a net also touches would not tell two of them apart.
            let sheet = net
                .sheets
                .iter()
                .filter_map(|path| sheet_names.get(path).copied())
                .find(|name| *name != "/")
                .unwrap_or("/");
            format!("{sheet}/{}", net.name)
        } else {
            net.name.clone()
        };
        let kicad = if net.kicad_name == name {
            String::new()
        } else {
            format!("={}", net.kicad_name)
        };
        // A net drawn on more than one sheet is marked, because a per-sheet
        // view shows only part of it.
        let leaves = if net.sheets.len() > 1 { "*" } else { "" };
        let _ = writeln!(out, "N {name}{leaves}{kicad}: {}", pins.join(" "));
    }
    if unconnected > 0 {
        let _ = writeln!(
            out,
            "# {unconnected} pin(s) join nothing; sch erc lists them"
        );
    }
    out
}

/// Count the symbols a view would list for one placement.
#[must_use]
pub fn listed_symbols<'a>(
    symbols: impl Iterator<Item = &'a Symbol>,
    path: &SheetPath,
    include_power: bool,
) -> usize {
    symbols
        .filter(|symbol| symbol.reference_on(path).is_some())
        .filter(|symbol| include_power || !symbol.is_power())
        .count()
}

/// The same content as [`render`], as JSON.
///
/// The terse form is the default because this costs more than twice as many
/// bytes for the same content. Both carry the same records, so a reader can
/// choose by what it is doing rather than by what is available.
#[must_use]
pub fn to_json(hierarchy: &Hierarchy, nets: &Nets, options: &ViewOptions) -> serde_json::Value {
    let text = render(hierarchy, nets, options);
    let mut sheets = Vec::new();
    let mut net_records = Vec::new();

    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("sheet ") {
            let mut parts = rest.split_whitespace();
            let path = parts.next().unwrap_or_default();
            let name = parts.next().unwrap_or("/");
            sheets.push(serde_json::json!({
                "path": path,
                "name": name,
                "symbols": [],
            }));
        } else if let Some(rest) = line.strip_prefix("S ") {
            let mut parts = rest.split_whitespace();
            let record = serde_json::json!({
                "reference": parts.next().unwrap_or_default(),
                "value": parts.next().unwrap_or_default(),
                "library": parts.next().unwrap_or_default(),
            });
            if let Some(sheet) = sheets.last_mut() {
                if let Some(list) = sheet["symbols"].as_array_mut() {
                    list.push(record);
                }
            }
        } else if let Some(rest) = line.strip_prefix("N ") {
            let (head, pins) = rest.split_once(": ").unwrap_or((rest, ""));
            let leaves = head.contains('*');
            let head = head.replace('*', "");
            let (name, kicad_name) = head
                .split_once('=')
                .map_or((head.clone(), head.clone()), |(name, kicad)| {
                    (name.to_owned(), kicad.to_owned())
                });
            net_records.push(serde_json::json!({
                "name": name,
                "kicad_name": kicad_name,
                "crosses_sheets": leaves,
                "pins": pins.split_whitespace().collect::<Vec<_>>(),
            }));
        }
    }

    serde_json::json!({
        "scope": if options.sheet.is_some() { "sheet" } else { "project" },
        "sheets": sheets,
        "nets": net_records,
    })
}

#[cfg(test)]
mod tests {
    use super::{natural_key, port_letter, short};

    #[test]
    fn a_reference_sorts_by_its_number_and_not_its_text() {
        let mut references = ["R10", "R2", "C1", "R1"];
        references.sort_by_key(|reference| natural_key(reference));
        assert_eq!(references, ["C1", "R1", "R2", "R10"]);
    }

    #[test]
    fn a_port_direction_is_one_letter() {
        assert_eq!(port_letter("input"), 'i');
        assert_eq!(port_letter("output"), 'o');
        assert_eq!(port_letter("bidirectional"), 'b');
        assert_eq!(port_letter("tri_state"), 't');
        assert_eq!(port_letter("passive"), 'p');
    }

    #[test]
    fn an_identifier_prints_as_its_first_eight_characters() {
        assert_eq!(short("00000000-0000-4000-8000-030000000001"), "00000000");
        assert_eq!(short("short"), "short");
    }
}
