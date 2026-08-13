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
//! A bundle never joins a single net: a bus, a bus label and a bus entry to a
//! bus carry a bundle, and the union-find refuses to join the two kinds, as
//! KiCad's subgraph walk does.
//!
//! All arithmetic is integer. A point is on a segment or it is not.

use crate::geometry::{Point, resolve_pins};
use crate::model::hierarchy::{Hierarchy, Placement};
use crate::model::items::{Item, LineKind, Refdes, SheetPath, Symbol, Uuid};
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
    Label,
    /// One pin on the border of a sheet symbol.
    SheetPin,
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
    pub(crate) fn build(hierarchy: &Hierarchy) -> Self {
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
            self.sheets.push(Sheet {
                path: placement.path.clone(),
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
                self.push_at(sheet, carrier, NodeKind::Label, label.at);
            }
            Item::Sheet(child) => {
                for pin in &child.pins {
                    let carrier = carrier_of_name(&pin.name);
                    self.push_at(sheet, carrier, NodeKind::SheetPin, pin.at);
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
        for pin in resolve_pins(symbol, definition) {
            let kind = NodeKind::Pin(PinNode {
                reference: reference.cloned(),
                number: pin.number.clone(),
                symbol: symbol.uuid.clone(),
                power,
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
        let labels = self.nodes_where(|node| matches!(node.kind, NodeKind::Label));

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
                            | NodeKind::Label
                            | NodeKind::SheetPin
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
fn carrier_of_name(name: &str) -> Carrier {
    let vector = name.contains('[') && name.contains("..") && name.ends_with(']');
    if vector || name.contains('{') {
        Carrier::Bus
    } else {
        Carrier::Net
    }
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
    }
}
