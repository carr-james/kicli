//! Which drawn items belong to the same conductor.
//!
//! The graph holds one node per connectable item, per placement of the sheet it
//! is drawn on, and a union-find over them. The geometric merge rules are
//! KiCad's, read from `eeschema/connection_graph.cpp` and `eeschema/sch_label.cpp`
//! at tag 10.0.5 and measured against `kicad-cli sch export netlist`:
//!
//! 1. Items that share a connection point exactly are one conductor. A wire
//!    has two connection points, its ends, and nothing in between
//!    (`SCH_LINE::GetConnectionPoints`).
//! 2. A junction joins every wire or bus that passes through its position,
//!    interior included (`CONNECTION_GRAPH::updateItemConnectivity`, the
//!    `GetBusesAndWires` call). A pin or sheet pin that merely lies on a
//!    wire's interior does not join it, and two wires crossing with no
//!    junction do not join either.
//! 3. A label joins the wires its anchor lies on. Two or more such wires all
//!    join (`updateItemConnectivity`, the label special case); exactly one
//!    joins only when no pin, label, sheet pin or no-connect sits at the
//!    anchor as well (`SCH_LABEL_BASE::UpdateDanglingState`, which connects
//!    the label to a segment it hits only while the label is still dangling).
//!
//! Names finish the job, because a drawing connects by name as well as by
//! geometry:
//!
//! 4. Labels of equal text join: local labels within one placement of one
//!    sheet, global labels across the project, and a hierarchical label with
//!    the like-named pin of the sheet symbol that draws its placement.
//! 5. Power-symbol pins of equal value join across the project.
//!
//! A bundle never joins a single net: a bus, a bus label and a bus entry to a
//! bus carry a bundle, and the union-find refuses to join the two kinds, as
//! KiCad's subgraph walk does.
//!
//! All arithmetic is integer. A point is on a segment or it is not.

use super::MergeRules;
use super::names::{net_name, unescape_net_name};
use crate::geometry::{Point, resolve_pins};
use crate::model::hierarchy::{Hierarchy, Placement};
use crate::model::items::{Item, LabelKind, LineKind, Refdes, SheetPath, Symbol, Uuid};
use crate::model::library::{LibrarySymbol, definition_of, read_library};
use kicli_sexpr::{Doc, NodeId};
use std::collections::BTreeMap;

/// Does an item carry one net, or a bundle of them?
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Carrier {
    /// One net.
    Net,
    /// A bundle, drawn as a bus.
    Bus,
}

/// A pin of a placed symbol, as the graph needs it.
#[derive(Clone, Debug)]
pub(crate) struct PinNode {
    /// The reference designator on this sheet path, when the file records one.
    pub reference: Option<Refdes>,
    /// The pin number, as text.
    pub number: String,
    /// The placed symbol.
    pub symbol: Uuid,
    /// Is the symbol a power symbol? A netlist leaves those pins out.
    pub power: bool,
    /// The symbol's value, which for a power symbol is the net name.
    pub value: String,
}

/// What a node is.
#[derive(Clone, Debug)]
pub(crate) enum NodeKind {
    /// A wire or bus segment.
    Line,
    /// A junction dot.
    Junction,
    /// A bus entry.
    BusEntry,
    /// A no-connect marker.
    NoConnect,
    /// A label of any of the four kinds.
    Label {
        /// Which kind of label.
        kind: LabelKind,
        /// The text, which for the first three kinds is a net name.
        text: String,
    },
    /// One pin on the border of a sheet symbol.
    SheetPin {
        /// The pin name, which matches a hierarchical label in the child file.
        name: String,
        /// The sheet symbol the pin belongs to.
        sheet_item: Uuid,
    },
    /// A pin of a placed symbol.
    Pin(PinNode),
}

/// One connectable item, on one placement of the sheet it is drawn on.
#[derive(Clone, Debug)]
pub(crate) struct Node {
    /// Which placement the item is drawn on.
    pub sheet: usize,
    /// One net or a bundle.
    pub carrier: Carrier,
    /// What the item is.
    pub kind: NodeKind,
    /// The points at which the item connects.
    points: Vec<Point>,
    /// The body of a line, which a junction or a label may lie on.
    segment: Option<(Point, Point)>,
}

/// One placement of a sheet, as the graph needs it.
#[derive(Clone, Debug)]
pub(crate) struct Sheet {
    /// The sheet path, in KiCad's uuid form.
    pub path: SheetPath,
    /// The sheet path in the readable form KiCad prefixes a local net name
    /// with, such as `/channel_a/`. The root sheet is `/`.
    pub human: String,
    /// The placement this one hangs from.
    parent: Option<usize>,
    /// The sheet symbol in the parent that draws this placement.
    drawn_by: Option<Uuid>,
}

/// The connection graph of one project.
pub(crate) struct Graph {
    /// Every connectable item, in placement order then file order.
    pub nodes: Vec<Node>,
    /// Every placement of every sheet.
    pub sheets: Vec<Sheet>,
    parent: Vec<usize>,
}

impl Graph {
    /// Build the graph of a loaded hierarchy.
    pub(crate) fn build(hierarchy: &Hierarchy, rules: MergeRules) -> Self {
        let mut graph = Self {
            nodes: Vec::new(),
            sheets: Vec::new(),
            parent: Vec::new(),
        };
        graph.collect(hierarchy);
        graph.parent = (0..graph.nodes.len()).collect();

        graph.merge_shared_points();
        graph.merge_junctions();
        graph.merge_labels_on_segments();
        if rules.labels {
            graph.merge_by_name();
            graph.merge_hierarchy();
        }
        if rules.power {
            graph.merge_power();
        }
        graph
    }

    /// The class of a node, after every merge.
    pub(crate) fn class_of(&mut self, node: usize) -> usize {
        find(&mut self.parent, node)
    }

    /// Read every placement into nodes.
    fn collect(&mut self, hierarchy: &Hierarchy) {
        let libraries: Vec<Vec<LibrarySymbol>> = hierarchy
            .files
            .iter()
            .map(|file| {
                read_library(
                    &file.doc,
                    &file.schematic.library_symbols,
                    file.schematic.version,
                )
            })
            .collect();

        for (index, placement) in hierarchy.placements.iter().enumerate() {
            let human = match (placement.parent, &placement.name) {
                (Some(above), Some(name)) => format!("{}{name}/", self.sheets[above].human),
                _ => "/".to_owned(),
            };
            self.sheets.push(Sheet {
                path: placement.path.clone(),
                human,
                parent: placement.parent,
                drawn_by: placement
                    .path
                    .segments()
                    .last()
                    .map(|last| Uuid(last.to_owned())),
            });
            let file = &hierarchy.files[placement.file];
            let library = &libraries[placement.file];
            for item in &file.schematic.items {
                self.read_item(index, item, &file.doc, library, placement);
            }
        }
    }

    /// Read one item of one placement into nodes.
    fn read_item(
        &mut self,
        sheet: usize,
        item: &Item,
        doc: &Doc,
        library: &[LibrarySymbol],
        placement: &Placement,
    ) {
        match item {
            Item::Line(line) => {
                let carrier = match line.kind {
                    LineKind::Wire => Carrier::Net,
                    LineKind::Bus => Carrier::Bus,
                };
                self.nodes.push(Node {
                    sheet,
                    carrier,
                    kind: NodeKind::Line,
                    points: vec![line.from, line.to],
                    segment: Some((line.from, line.to)),
                });
            }
            Item::Junction(junction) => {
                self.push_at(sheet, Carrier::Net, NodeKind::Junction, junction.at);
            }
            Item::NoConnect(marker) => {
                self.push_at(sheet, Carrier::Net, NodeKind::NoConnect, marker.at);
            }
            Item::BusEntry(entry) => self.nodes.push(Node {
                sheet,
                carrier: Carrier::Net,
                kind: NodeKind::BusEntry,
                points: vec![entry.at, entry_end(doc, entry.node, entry.at)],
                segment: None,
            }),
            Item::Label(label) => {
                let carrier = carrier_of_name(&label.text);
                let kind = NodeKind::Label {
                    kind: label.kind,
                    text: label.text.clone(),
                };
                self.push_at(sheet, carrier, kind, label.at);
            }
            Item::Sheet(child) => {
                for pin in &child.pins {
                    let carrier = carrier_of_name(&pin.name);
                    let kind = NodeKind::SheetPin {
                        name: pin.name.clone(),
                        sheet_item: child.uuid.clone(),
                    };
                    self.push_at(sheet, carrier, kind, pin.at);
                }
            }
            Item::Symbol(symbol) => self.read_symbol(sheet, symbol, library, placement),
            Item::Text(_) | Item::Other { .. } => {}
        }
    }

    /// Read the pins one placed symbol draws.
    fn read_symbol(
        &mut self,
        sheet: usize,
        symbol: &Symbol,
        library: &[LibrarySymbol],
        placement: &Placement,
    ) {
        let Some(definition) = definition_of(library, symbol) else {
            return;
        };
        let reference = symbol.reference_on(&placement.path);
        let power =
            definition.is_power || reference.is_some_and(|refdes| refdes.0.starts_with('#'));
        let value = symbol
            .field("Value")
            .map(|field| field.value.clone())
            .unwrap_or_default();
        for pin in resolve_pins(symbol, definition) {
            // A power symbol names a net through a power input: the rail says
            // "I am +5V". A power OUTPUT on a power symbol says only "something
            // drives this net", which is what PWR_FLAG is, and it names
            // nothing. Merging those by value joins every flagged net in the
            // project into one, which is four rails on KiCad's own CM5 demo.
            let names_a_net = pin.electrical == "power_in";
            let kind = NodeKind::Pin(PinNode {
                reference: reference.cloned(),
                number: pin.number.clone(),
                symbol: symbol.uuid.clone(),
                power,
                value: if names_a_net {
                    value.clone()
                } else {
                    String::new()
                },
            });
            self.push_at(sheet, Carrier::Net, kind, pin.position);
        }
    }

    /// Add a node that connects at one point and has no body.
    fn push_at(&mut self, sheet: usize, carrier: Carrier, kind: NodeKind, at: Point) {
        self.nodes.push(Node {
            sheet,
            carrier,
            kind,
            points: vec![at],
            segment: None,
        });
    }

    /// Rule 1: items that share a point exactly are one conductor.
    fn merge_shared_points(&mut self) {
        for group in self.by_point().into_values() {
            for pair in group.windows(2) {
                self.union(pair[0], pair[1]);
            }
        }
    }

    /// Rule 2: a junction joins every line that passes through it.
    fn merge_junctions(&mut self) {
        let junctions = self.nodes_where(|node| matches!(node.kind, NodeKind::Junction));

        for junction in junctions {
            let point = self.nodes[junction].points[0];
            let lines = self.lines_through(self.nodes[junction].sheet, point);
            let mut previous_bus = None;
            for line in lines {
                match self.nodes[line].carrier {
                    Carrier::Net => self.union(junction, line),
                    Carrier::Bus => {
                        if let Some(earlier) = previous_bus {
                            self.union(earlier, line);
                        }
                        previous_bus = Some(line);
                    }
                }
            }
        }
    }

    /// Rule 3: a label joins the lines its anchor lies on.
    fn merge_labels_on_segments(&mut self) {
        let by_point = self.by_point();
        let labels = self.nodes_where(|node| matches!(node.kind, NodeKind::Label { .. }));

        for label in labels {
            let sheet = self.nodes[label].sheet;
            let point = self.nodes[label].points[0];
            let lines = self.lines_through(sheet, point);
            if lines.len() == 1 && self.is_met_at(&by_point, label, sheet, point) {
                continue;
            }
            for line in lines {
                self.union(label, line);
            }
        }
    }

    /// Rule 4: labels of equal text join.
    ///
    /// A netclass flag carries a netclass name and not a net name, so its text
    /// joins nothing.
    fn merge_by_name(&mut self) {
        let mut local: BTreeMap<(usize, String), Vec<usize>> = BTreeMap::new();
        let mut global: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (index, node) in self.nodes.iter().enumerate() {
            if let NodeKind::Label { kind, text } = &node.kind {
                match kind {
                    LabelKind::Local => local
                        .entry((node.sheet, net_name(text)))
                        .or_default()
                        .push(index),
                    LabelKind::Global => global.entry(net_name(text)).or_default().push(index),
                    LabelKind::Hierarchical | LabelKind::NetclassFlag => {}
                }
            }
        }
        let groups: Vec<Vec<usize>> = local.into_values().chain(global.into_values()).collect();
        self.union_each(&groups);
    }

    /// Rule 4, the hierarchy half: a hierarchical label meets the pin above it.
    fn merge_hierarchy(&mut self) {
        let mut pairs = Vec::new();
        for (index, node) in self.nodes.iter().enumerate() {
            let NodeKind::Label {
                kind: LabelKind::Hierarchical,
                text,
            } = &node.kind
            else {
                continue;
            };
            let sheet = &self.sheets[node.sheet];
            let (Some(parent), Some(drawn_by)) = (sheet.parent, sheet.drawn_by.as_ref()) else {
                continue;
            };
            for (other, candidate) in self.nodes.iter().enumerate() {
                if candidate.sheet != parent {
                    continue;
                }
                if let NodeKind::SheetPin { name, sheet_item } = &candidate.kind {
                    if net_name(name) == net_name(text) && sheet_item == drawn_by {
                        pairs.push((index, other));
                    }
                }
            }
        }
        for (label, pin) in pairs {
            self.union(label, pin);
        }
    }

    /// Rule 5: power-symbol pins of equal value join across the project.
    fn merge_power(&mut self) {
        let mut by_value: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (index, node) in self.nodes.iter().enumerate() {
            if let NodeKind::Pin(pin) = &node.kind {
                if pin.power && !pin.value.is_empty() {
                    by_value
                        .entry(net_name(&pin.value))
                        .or_default()
                        .push(index);
                }
            }
        }
        let groups: Vec<Vec<usize>> = by_value.into_values().collect();
        self.union_each(&groups);
    }

    /// Join every node of every group to the others of its group.
    fn union_each(&mut self, groups: &[Vec<usize>]) {
        for group in groups {
            for pair in group.windows(2) {
                self.union(pair[0], pair[1]);
            }
        }
    }

    /// The nodes a test holds for.
    fn nodes_where(&self, wanted: impl Fn(&Node) -> bool) -> Vec<usize> {
        self.nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| wanted(node))
            .map(|(index, _)| index)
            .collect()
    }

    /// Every node that connects at each point of each placement.
    fn by_point(&self) -> BTreeMap<(usize, Point), Vec<usize>> {
        let mut map: BTreeMap<(usize, Point), Vec<usize>> = BTreeMap::new();
        for (index, node) in self.nodes.iter().enumerate() {
            for &point in &node.points {
                map.entry((node.sheet, point)).or_default().push(index);
            }
        }
        map
    }

    /// The lines of one placement whose body holds a point.
    fn lines_through(&self, sheet: usize, point: Point) -> Vec<usize> {
        self.nodes_where(|node| {
            node.sheet == sheet
                && node
                    .segment
                    .is_some_and(|(from, to)| on_segment(from, to, point))
        })
    }

    /// Is a label already met at its anchor by something that is not a line?
    ///
    /// KiCad connects a label to a segment it merely lies on only while the
    /// label is still dangling, and a pin, another label, a sheet pin or a
    /// no-connect at the same point ends that.
    fn is_met_at(
        &self,
        by_point: &BTreeMap<(usize, Point), Vec<usize>>,
        label: usize,
        sheet: usize,
        point: Point,
    ) -> bool {
        by_point.get(&(sheet, point)).is_some_and(|nodes| {
            nodes.iter().any(|&other| {
                other != label
                    && matches!(
                        self.nodes[other].kind,
                        NodeKind::Pin(_)
                            | NodeKind::Label { .. }
                            | NodeKind::SheetPin { .. }
                            | NodeKind::NoConnect
                    )
            })
        })
    }

    /// Join two nodes, unless one carries a bundle and the other does not.
    fn union(&mut self, left: usize, right: usize) {
        let (a, b) = (find(&mut self.parent, left), find(&mut self.parent, right));
        if a == b || self.nodes[a].carrier != self.nodes[b].carrier {
            return;
        }
        self.parent[b] = a;
    }
}

/// The root of a node's class, with the path compressed on the way.
fn find(parent: &mut [usize], node: usize) -> usize {
    let mut current = node;
    while parent[current] != current {
        parent[current] = parent[parent[current]];
        current = parent[current];
    }
    current
}

/// Is a point on a segment, ends included?
///
/// Exact integer arithmetic in 64 bits: the cross product decides whether the
/// point is on the line, and the dot product whether it is between the ends.
fn on_segment(from: Point, to: Point, point: Point) -> bool {
    let (ax, ay) = (i64::from(from.x.0), i64::from(from.y.0));
    let (bx, by) = (i64::from(to.x.0), i64::from(to.y.0));
    let (px, py) = (i64::from(point.x.0), i64::from(point.y.0));
    let (dx, dy) = (bx - ax, by - ay);
    if dx * (py - ay) - dy * (px - ax) != 0 {
        return false;
    }
    let along = dx * (px - ax) + dy * (py - ay);
    along >= 0 && along <= dx * dx + dy * dy
}

/// Does this name stand for a bundle?
///
/// KiCad writes a bundle as a vector, `D[0..7]`, or as a group, `{A B}`, or as
/// a group with a prefix. A name with neither form is one net.
///
/// The test is of the unescaped name (`SCH_CONNECTION::IsBusLabel`, which
/// calls `UnescapeString` first). `VPP{slash}MCLR` is therefore one net named
/// `VPP/MCLR`, and not a group of one member called `slash`. A brace that
/// follows `$`, `~`, `^` or `_` draws formatting such as an overbar, so it
/// does not make a group either.
fn carrier_of_name(name: &str) -> Carrier {
    let plain = unescape_net_name(name);
    if is_bus_vector(&plain) || is_bus_group(&plain) {
        Carrier::Bus
    } else {
        Carrier::Net
    }
}

/// Is this unescaped name a vector, such as `D[0..7]`?
fn is_bus_vector(name: &str) -> bool {
    name.contains('[') && name.contains("..") && name.ends_with(']')
}

/// Is this unescaped name a group, such as `{A B}` or `USB{DP DM}`?
fn is_bus_group(name: &str) -> bool {
    let characters: Vec<char> = name.chars().collect();
    characters.iter().enumerate().any(|(index, &character)| {
        character == '{'
            && (index == 0 || !matches!(characters[index - 1], '$' | '~' | '^' | '_'))
            && characters[index + 1..].contains(&'}')
    })
}

/// The far end of a bus entry.
///
/// The item model keeps the anchor of every one-point item. A bus entry also
/// has a size, and both ends connect, so the size is read from the tree.
fn entry_end(doc: &Doc, node: NodeId, at: Point) -> Point {
    let Some(size) = doc
        .children(node)
        .iter()
        .copied()
        .find(|&child| doc.head_is(child, "size"))
    else {
        return at;
    };
    let values = doc.children(size);
    let read = |index: usize| -> i32 {
        values
            .get(index)
            .and_then(|&id| doc.atom_as_iu(id))
            .unwrap_or_default()
    };
    Point::new(at.x.0 + read(1), at.y.0 + read(2))
}

#[cfg(test)]
mod tests {
    use super::{Carrier, carrier_of_name, on_segment};
    use crate::geometry::Point;

    #[test]
    fn a_point_is_on_a_segment_or_it_is_not() {
        let from = Point::new(0, 0);
        let to = Point::new(100, 0);
        assert!(on_segment(from, to, Point::new(50, 0)));
        assert!(on_segment(from, to, from));
        assert!(on_segment(from, to, to));
        assert!(!on_segment(from, to, Point::new(101, 0)));
        assert!(!on_segment(from, to, Point::new(-1, 0)));
        assert!(!on_segment(from, to, Point::new(50, 1)));
        // A diagonal segment is measured the same way.
        let corner = Point::new(100, 100);
        assert!(on_segment(from, corner, Point::new(37, 37)));
        assert!(!on_segment(from, corner, Point::new(37, 38)));
    }

    #[test]
    fn a_bundle_is_recognised_by_its_name() {
        assert_eq!(carrier_of_name("D[0..7]"), Carrier::Bus);
        assert_eq!(carrier_of_name("{SDA SCL}"), Carrier::Bus);
        assert_eq!(carrier_of_name("USB{DP DM}"), Carrier::Bus);
        assert_eq!(carrier_of_name("D0"), Carrier::Net);
        assert_eq!(carrier_of_name("GND"), Carrier::Net);
        // An escape word is one character of one name, and an overbar is
        // formatting. Neither makes a bundle.
        assert_eq!(carrier_of_name("VPP{slash}MCLR"), Carrier::Net);
        assert_eq!(carrier_of_name("~{RESET}"), Carrier::Net);
    }
}
