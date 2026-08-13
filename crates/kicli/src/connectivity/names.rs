//! What each net is called, and how it is addressed.
//!
//! Two names per net. kicli's own name is what an agent reads and addresses: a
//! power-symbol value, a label, or a synthetic `#n<k>` assigned by descending
//! pin count then by pin list. The number is therefore a property of the
//! design: two runs over one project agree, and an edit elsewhere in the
//! drawing does not renumber it.
//!
//! KiCad's name is carried beside it, unchanged, because an agent needs it to
//! read ERC output and the editor. It is never a handle: KiCad names an
//! unlabelled net after one of its pins, so renumbering that symbol renames the
//! net. That naming is ported from `CONNECTION_SUBGRAPH::ResolveDrivers` and
//! `SCH_PIN::GetDefaultNetName` at tag 10.0.5: the driver with the highest
//! priority names the net, and equal priorities are settled by sorting the
//! names and taking the first.

use super::graph::{Graph, NodeKind};
use super::{Net, NetPin};
use crate::model::items::{LabelKind, SheetPath};
use std::collections::{BTreeMap, BTreeSet};

/// The net name a piece of text drives.
///
/// A net name may not hold a `/`, because `/` separates the sheet path from
/// the name. KiCad writes the character as `{slash}` and drops line breaks
/// (`EscapeString`, `CTX_NETNAME`, in `common/string_utils.cpp`). Two labels
/// join when these forms are equal, so `A/B` and `A{slash}B` are one net,
/// while `A-B` and `A_B` are two.
pub(crate) fn net_name(text: &str) -> String {
    let mut name = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '/' => name.push_str("{slash}"),
            '\n' | '\r' => {}
            other => name.push(other),
        }
    }
    name
}

/// The character an escape word stands for, if it is one.
///
/// The list is `UnescapeString` in `common/string_utils.cpp`.
fn escaped_character(word: &str) -> Option<char> {
    match word {
        "dblquote" => Some('"'),
        "quote" => Some('\''),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "backslash" => Some('\\'),
        "slash" => Some('/'),
        "bar" => Some('|'),
        "comma" => Some(','),
        "colon" => Some(':'),
        "space" => Some(' '),
        "dollar" => Some('$'),
        "tab" => Some('\t'),
        "return" => Some('\n'),
        "brace" => Some('{'),
        _ => None,
    }
}

/// The text an escaped name stands for.
///
/// This is the inverse of [`net_name`] over the whole escape table, and it is
/// what KiCad reads before it decides whether a name is a bundle
/// (`SCH_CONNECTION::IsBusLabel`, which calls `UnescapeString` first). A brace
/// that follows `$`, `~`, `^` or `_` is text formatting, such as an overbar,
/// so it survives unchanged.
pub(crate) fn unescape_net_name(text: &str) -> String {
    let characters: Vec<char> = text.chars().collect();
    let mut plain = String::with_capacity(text.len());
    let mut index = 0;
    while index < characters.len() {
        let character = characters[index];
        if character != '{' {
            plain.push(character);
            index += 1;
            continue;
        }
        let formatting = index > 0 && matches!(characters[index - 1], '$' | '~' | '^' | '_');
        let (word, end) = braced_word(&characters, index);
        match (formatting, end, escaped_character(&word)) {
            (false, Some(_), Some(character)) => plain.push(character),
            (_, Some(_), _) => {
                plain.push('{');
                plain.push_str(&unescape_net_name(&word));
                plain.push('}');
            }
            (_, None, _) => {
                plain.push('{');
                plain.push_str(&unescape_net_name(&word));
            }
        }
        index = end.unwrap_or(characters.len() - 1) + 1;
    }
    plain
}

/// The text between a brace and its partner, and where the partner is.
///
/// Braces nest, so the partner is the one that closes the first.
fn braced_word(characters: &[char], open: usize) -> (String, Option<usize>) {
    let mut depth = 1_u32;
    let mut word = String::new();
    for (offset, &character) in characters.iter().enumerate().skip(open + 1) {
        match character {
            '{' => depth += 1,
            '}' => depth -= 1,
            _ => {}
        }
        if depth == 0 {
            return (word, Some(offset));
        }
        word.push(character);
    }
    (word, None)
}

/// Every net of a built graph, ordered and named.
pub(crate) fn nets_of(graph: &mut Graph) -> Vec<Net> {
    let mut members: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for node in 0..graph.nodes.len() {
        let class = graph.class_of(node);
        members.entry(class).or_default().push(node);
    }

    let mut drafts: Vec<Draft> = members
        .into_values()
        .filter_map(|nodes| draft(graph, &nodes))
        .collect();

    // Descending pin count, then the pin list. KiCad's name settles the rare
    // pair with the same pins, so the order is total.
    drafts.sort_by(|left, right| {
        right
            .listed
            .len()
            .cmp(&left.listed.len())
            .then_with(|| left.listed.cmp(&right.listed))
            .then_with(|| left.kicad_name.cmp(&right.kicad_name))
    });

    let mut synthetic = 0_u32;
    drafts
        .into_iter()
        .map(|draft| {
            let synthesised = draft.name.is_none();
            let name = draft.name.unwrap_or_else(|| {
                synthetic += 1;
                format!("#n{synthetic}")
            });
            Net {
                name,
                synthetic: synthesised,
                kicad_name: draft.kicad_name,
                pins: draft.pins,
                sheets: draft.sheets,
            }
        })
        .collect()
}

/// One net, before its synthetic handle is assigned.
struct Draft {
    name: Option<String>,
    kicad_name: String,
    pins: Vec<NetPin>,
    sheets: Vec<SheetPath>,
    /// The pins a netlist lists, which leaves power symbols out. This is the
    /// sort key, so kicli's order matches the netlist it is checked against.
    listed: Vec<String>,
}

/// The names a net's items offer, split by what offers them.
///
/// A label appears twice: as its own text, which is what kicli shows, and as
/// the net name it drives, which is how KiCad writes it. The two differ when
/// the text holds a character a net name may not hold.
#[derive(Default)]
struct Drivers {
    global: Vec<(String, String)>,
    power: Vec<(String, String)>,
    local: Vec<(String, String)>,
    hierarchical: Vec<(String, String)>,
    sheet_pins: Vec<(String, String)>,
    pins: Vec<String>,
    pin_count: usize,
}

/// Read one class of the graph into a net, or nothing when it has no pins.
fn draft(graph: &Graph, nodes: &[usize]) -> Option<Draft> {
    let mut drivers = Drivers::default();
    let mut pins = Vec::new();
    let mut sheets = BTreeSet::new();

    for &index in nodes {
        let node = &graph.nodes[index];
        let sheet = &graph.sheets[node.sheet];
        sheets.insert(sheet.path.clone());
        match &node.kind {
            NodeKind::Label { kind, text } => read_label(&mut drivers, *kind, text, &sheet.human),
            NodeKind::SheetPin { name, .. } => drivers
                .sheet_pins
                .push((format!("{}{}", sheet.human, net_name(name)), name.clone())),
            NodeKind::Pin(pin) => {
                drivers.pin_count += 1;
                if pin.power {
                    drivers
                        .power
                        .push((net_name(&pin.value), pin.value.clone()));
                }
                if let Some(reference) = &pin.reference {
                    if !pin.power {
                        drivers
                            .pins
                            .push(format!("{}-Pad{}", reference.0, pin.number));
                    }
                    pins.push(NetPin {
                        reference: reference.clone(),
                        number: pin.number.clone(),
                        sheet: sheet.path.clone(),
                        symbol: pin.symbol.clone(),
                        power: pin.power,
                    });
                }
            }
            NodeKind::Line | NodeKind::Junction | NodeKind::BusEntry | NodeKind::NoConnect => {}
        }
    }

    if pins.is_empty() {
        return None;
    }
    pins.sort();
    // A pin of unit 0 is drawn on every unit of a symbol, so one pin number
    // reaches a net once per unit. A net lists it once
    // (`NETLIST_EXPORTER_XML::makeListOfNets`, the `alg::remove_duplicates`
    // call). The rule is per net and not per symbol: when the units are wired
    // apart, the pin number is listed on each net it reaches.
    pins.dedup_by(|left, right| left.reference == right.reference && left.number == right.number);
    let listed = pins
        .iter()
        .filter(|pin| !pin.power)
        .map(NetPin::label)
        .collect();
    Some(Draft {
        name: display_name(&drivers),
        kicad_name: kicad_name(&drivers),
        pins,
        sheets: sheets.into_iter().collect(),
        listed,
    })
}

/// File one label under the kind of name it offers.
fn read_label(drivers: &mut Drivers, kind: LabelKind, text: &str, prefix: &str) {
    let qualified = format!("{prefix}{}", net_name(text));
    match kind {
        LabelKind::Global => drivers.global.push((net_name(text), text.to_owned())),
        LabelKind::Local => drivers.local.push((qualified, text.to_owned())),
        LabelKind::Hierarchical => drivers.hierarchical.push((qualified, text.to_owned())),
        // A netclass flag carries a netclass name, not a net name.
        LabelKind::NetclassFlag => {}
    }
}

/// The name kicli shows: a power value, then a label, the widest first.
fn display_name(drivers: &Drivers) -> Option<String> {
    first_plain(&drivers.power)
        .or_else(|| first_plain(&drivers.global))
        .or_else(|| first_plain(&drivers.hierarchical))
        .or_else(|| first_plain(&drivers.local))
}

/// The name KiCad gives this net.
///
/// The driver priority is KiCad's own: a global label, then a power pin, then
/// a local label, a hierarchical label, a sheet pin, and last a pin. A net
/// named after its only pin is called unconnected rather than a net.
fn kicad_name(drivers: &Drivers) -> String {
    if let Some(name) = first_qualified(&drivers.global)
        .or_else(|| first_qualified(&drivers.power))
        .or_else(|| first_qualified(&drivers.local))
        .or_else(|| first_qualified(&drivers.hierarchical))
        .or_else(|| first_qualified(&drivers.sheet_pins))
    {
        return name;
    }
    let Some(pin) = first(&drivers.pins) else {
        return String::new();
    };
    if drivers.pin_count == 1 {
        format!("unconnected-({pin})")
    } else {
        format!("Net-({pin})")
    }
}

/// The first name in sorted order.
fn first(names: &[String]) -> Option<String> {
    names.iter().min().cloned()
}

/// The first name in sorted order, of a list that carries both forms.
fn first_plain(names: &[(String, String)]) -> Option<String> {
    names.iter().map(|(_, plain)| plain).min().cloned()
}

/// The first sheet-path-qualified name in sorted order.
fn first_qualified(names: &[(String, String)]) -> Option<String> {
    names.iter().map(|(qualified, _)| qualified).min().cloned()
}

#[cfg(test)]
mod tests {
    use super::{net_name, unescape_net_name};

    #[test]
    fn a_net_name_carries_no_slash() {
        assert_eq!(net_name("VPP/MCLR"), "VPP{slash}MCLR");
        assert_eq!(net_name("VPP{slash}MCLR"), "VPP{slash}MCLR");
        assert_eq!(net_name("A\nB"), "AB");
        assert_eq!(net_name("CC-DD"), "CC-DD");
    }

    #[test]
    fn unescaping_undoes_the_escape_words_and_nothing_else() {
        assert_eq!(unescape_net_name("VPP{slash}MCLR"), "VPP/MCLR");
        assert_eq!(unescape_net_name("A{colon}B{comma}C"), "A:B,C");
        // A brace after a formatting marker draws an overbar, so it stays.
        assert_eq!(unescape_net_name("~{RESET}"), "~{RESET}");
        // A word that is not in the table stays as it is written.
        assert_eq!(unescape_net_name("{SDA SCL}"), "{SDA SCL}");
        // An unterminated brace loses nothing.
        assert_eq!(unescape_net_name("A{B"), "A{B");
    }
}
