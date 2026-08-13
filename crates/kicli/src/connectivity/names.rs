//! What each net is called, and how it is addressed.
//!
//! A net kicli finds no name for is given a synthetic one, `#n<k>`, assigned by
//! descending pin count then by pin list. The number is therefore a property of
//! the design: two runs over one project agree, and an edit elsewhere in the
//! drawing does not renumber it.

use super::graph::{Graph, NodeKind};
use super::{Net, NetPin};
use crate::model::items::SheetPath;
use std::collections::{BTreeMap, BTreeSet};

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

    // Descending pin count, then the pin list.
    drafts.sort_by(|left, right| {
        right
            .listed
            .len()
            .cmp(&left.listed.len())
            .then_with(|| left.listed.cmp(&right.listed))
    });

    let mut synthetic = 0_u32;
    drafts
        .into_iter()
        .map(|draft| {
            synthetic += 1;
            Net {
                name: format!("#n{synthetic}"),
                synthetic: true,
                pins: draft.pins,
                sheets: draft.sheets,
            }
        })
        .collect()
}

/// One net, before its name is assigned.
struct Draft {
    pins: Vec<NetPin>,
    sheets: Vec<SheetPath>,
    /// The pins a netlist lists, which leaves power symbols out. This is the
    /// sort key, so kicli's order matches the netlist it is checked against.
    listed: Vec<String>,
}

/// Read one class of the graph into a net, or nothing when it has no pins.
fn draft(graph: &Graph, nodes: &[usize]) -> Option<Draft> {
    let mut pins = Vec::new();
    let mut sheets = BTreeSet::new();

    for &index in nodes {
        let node = &graph.nodes[index];
        sheets.insert(graph.sheets[node.sheet].path.clone());
        if let NodeKind::Pin(pin) = &node.kind {
            if let Some(reference) = &pin.reference {
                pins.push(NetPin {
                    reference: reference.clone(),
                    number: pin.number.clone(),
                    sheet: graph.sheets[node.sheet].path.clone(),
                    symbol: pin.symbol.clone(),
                    power: pin.power,
                });
            }
        }
    }

    if pins.is_empty() {
        return None;
    }
    pins.sort();
    let listed = pins
        .iter()
        .filter(|pin| !pin.power)
        .map(NetPin::label)
        .collect();
    Some(Draft {
        pins,
        sheets: sheets.into_iter().collect(),
        listed,
    })
}
