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
//! 4. Items of equal name join, and one sheet is one namespace: a local
//!    label, a hierarchical label, a global label and a power pin that carry
//!    one name on one sheet are one net, whatever their kinds. A global label
//!    and a power pin carry the name across the whole project as well, and a
//!    hierarchical label meets the like-named pin of the sheet symbol that
//!    draws its placement.
//! 5. A pin names a net when it is a power input, either on a power symbol,
//!    by the symbol's value, or hidden on an ordinary symbol, by its own
//!    name.
//!
//! 6. A bundle carries its members. A net named after one member, on any
//!    sheet the bundle reaches, is that member, and no wire between the two
//!    is needed. Where one bus carries two bundle names, their corresponding
//!    members are one net as well: a vector member corresponds by its place
//!    in the range, a group member by its own name.
//!
//! A bundle never joins a single net: a bus, a bus label and a bus entry to a
//! bus carry a bundle, and the union-find refuses to join the two kinds, as
//! KiCad's subgraph walk does.
//!
//! All arithmetic is integer. A point is on a segment or it is not.

use super::MergeRules;
use super::names::{net_name, unescape_net_name};
use crate::geometry::{Point, ResolvedPin, resolve_pins};
use crate::model::hierarchy::{Hierarchy, Placement};
use crate::model::items::{Item, LabelKind, LineKind, Refdes, SheetPath, Symbol, Uuid};
use crate::model::library::{LibrarySymbol, definition_of, read_library};
use kicli_sexpr::{Doc, NodeId};
use std::collections::{BTreeMap, BTreeSet};

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
    /// The net this pin names across the project, or the empty string.
    ///
    /// A power symbol names the net with its own value. An ordinary symbol
    /// with a hidden power input names the net with the pin's name.
    pub power_name: String,
    /// Does the symbol reach the board? A netlist lists no pin of a symbol
    /// that does not.
    pub on_board: bool,
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
        graph.merge_by_name(rules);
        if rules.labels {
            graph.merge_hierarchy();
            graph.merge_bus_members(rules);
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
            Item::Symbol(symbol) => self.read_symbol(sheet, symbol, doc, library, placement),
            Item::Text(_) | Item::Other { .. } => {}
        }
    }

    /// Read the pins one placed symbol draws.
    fn read_symbol(
        &mut self,
        sheet: usize,
        symbol: &Symbol,
        doc: &Doc,
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
        let on_board = on_board(doc, symbol.node);
        for pin in resolve_pins(&drawn_on(symbol, placement), definition) {
            let kind = NodeKind::Pin(PinNode {
                reference: reference.cloned(),
                number: pin.number.clone(),
                symbol: symbol.uuid.clone(),
                power,
                power_name: power_name(&pin, power, &value),
                on_board,
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
    ///
    /// A bus entry is the exception. It carries one member of a bundle, so it
    /// joins the wire at its wire end and joins nothing else at its bus end:
    /// not the bus, not a junction on the bus, not a bus label there, and not
    /// another bus entry (`SCH_BUS_WIRE_ENTRY::ConnectionPropagatesTo`). Two
    /// entries into one point of a bus carry two different members, so
    /// joining them would short two nets together.
    fn merge_shared_points(&mut self) {
        for ((sheet, point), group) in self.by_point() {
            let entries: Vec<usize> = group
                .iter()
                .copied()
                .filter(|&node| matches!(self.nodes[node].kind, NodeKind::BusEntry))
                .collect();
            if entries.is_empty() {
                for pair in group.windows(2) {
                    self.union(pair[0], pair[1]);
                }
                continue;
            }

            let others: Vec<usize> = group
                .iter()
                .copied()
                .filter(|&node| !matches!(self.nodes[node].kind, NodeKind::BusEntry))
                .collect();
            for pair in others.windows(2) {
                self.union(pair[0], pair[1]);
            }
            let bundled = self.bus_through(sheet, point);
            let partner = others
                .iter()
                .copied()
                .find(|&node| !(bundled && matches!(self.nodes[node].kind, NodeKind::Junction)));
            if let Some(partner) = partner {
                for entry in entries {
                    self.union(entry, partner);
                }
            }
        }
    }

    /// Does a bundle pass through this point of this placement?
    fn bus_through(&self, sheet: usize, point: Point) -> bool {
        self.lines_through(sheet, point)
            .into_iter()
            .any(|line| self.nodes[line].carrier == Carrier::Bus)
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

    /// Rule 4: items of equal name join.
    ///
    /// One sheet has one namespace. A local label, a hierarchical label, a
    /// global label and a power pin that carry one name on one sheet are one
    /// net, whatever their kinds (`CONNECTION_GRAPH::processSubGraphs`, which
    /// absorbs every same-sheet subgraph whose driver name matches). The
    /// global kinds — a global label and a power pin — carry that name across
    /// the whole project as well.
    ///
    /// A netclass flag carries a netclass name and not a net name, so its text
    /// joins nothing.
    fn merge_by_name(&mut self, rules: MergeRules) {
        let mut per_sheet: BTreeMap<(usize, String), Vec<usize>> = BTreeMap::new();
        let mut project: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for named in self.naming_items(rules) {
            if named.everywhere {
                project
                    .entry(named.name.clone())
                    .or_default()
                    .push(named.node);
            }
            per_sheet
                .entry((named.sheet, named.name))
                .or_default()
                .push(named.node);
        }
        let groups: Vec<Vec<usize>> = per_sheet
            .into_values()
            .chain(project.into_values())
            .collect();
        self.union_each(&groups);
    }

    /// Every item that names a net, with the name it drives.
    fn naming_items(&self, rules: MergeRules) -> Vec<Named> {
        let mut found = Vec::new();
        for (index, node) in self.nodes.iter().enumerate() {
            let (name, everywhere) = match &node.kind {
                NodeKind::Label { kind, text } if rules.labels => match kind {
                    LabelKind::Local | LabelKind::Hierarchical => (net_name(text), false),
                    LabelKind::Global => (net_name(text), true),
                    LabelKind::NetclassFlag => continue,
                },
                NodeKind::Pin(pin) if rules.power && !pin.power_name.is_empty() => {
                    (net_name(&pin.power_name), true)
                }
                _ => continue,
            };
            found.push(Named {
                node: index,
                sheet: node.sheet,
                name,
                everywhere,
            });
        }
        found
    }

    /// Rule 6: a bundle carries its members wherever it reaches, and two
    /// bundles on one bus carry each other's.
    ///
    /// A bundle names one net per member. A net of that name, on any sheet the
    /// bundle reaches, is that member, and no wire between the two is needed:
    /// `CONNECTION_GRAPH::processSubGraphs` links a bundle to every same-sheet
    /// net whose name is one of its members, and `propagateToNeighbors` then
    /// carries the member along the bundle through the hierarchy.
    ///
    /// Where one bus carries two bundle names, their corresponding members are
    /// one net as well. Members are collected by what they correspond to
    /// rather than by name, so `UART.RX` and `UART_TRG.RX` land in one group
    /// and join. `CONNECTION_GRAPH::matchBusMember` decides the
    /// correspondence, and [`Correspondence`] records what it measures.
    ///
    /// A bundle names its members in the scope of its own driver, not of the
    /// sheet each member is drawn on. Two bundles that share a scope therefore
    /// share every member whose name they both carry, though no bus joins
    /// them: `DQ[0..31]` and `DQ[0..15]` driven on one sheet have one `DQ0`
    /// between them. Two bundles of equal name in different scopes keep their
    /// members apart.
    fn merge_bus_members(&mut self, rules: MergeRules) {
        let buses = self.read_buses();
        let by_place = self.nets_by_place(rules);

        // One group per correspondence within a bus, and one per member name
        // within a scope. A member joins through either.
        let mut groups = Vec::new();
        let mut by_scope: BTreeMap<(String, String), Vec<usize>> = BTreeMap::new();
        for bus in buses.values() {
            // The driver that names the bus names its members too. A member of
            // any other bundle on that bus is an alias of the corresponding
            // one, so it is filed under the driver's name and not its own.
            let named = bus.names_of_driver();
            for (corresponds, members) in &bus.carried {
                let group: Vec<usize> = members
                    .iter()
                    .flat_map(|member| {
                        bus.reaches
                            .iter()
                            .filter_map(|&sheet| by_place.get(&(sheet, member.clone())))
                            .flatten()
                            .copied()
                    })
                    .collect();
                if let Some((_, _, scope, _)) = &bus.driver {
                    let canonical = named.get(corresponds).or_else(|| members.iter().next());
                    if let Some(canonical) = canonical {
                        by_scope
                            .entry((scope.clone(), canonical.clone()))
                            .or_default()
                            .extend(&group);
                    }
                }
                if group.len() > 1 {
                    groups.push(group);
                }
            }
        }
        groups.extend(by_scope.into_values().filter(|group| group.len() > 1));
        self.union_each(&groups);
    }

    /// What each bus of the graph carries, reaches, and is named by.
    fn read_buses(&mut self) -> BTreeMap<usize, Bus> {
        let mut buses: BTreeMap<usize, Bus> = BTreeMap::new();
        for index in self.nodes_where(|node| node.carrier == Carrier::Bus) {
            let (sheet, name) = (self.nodes[index].sheet, name_of(&self.nodes[index]));
            let ranked = self.rank_of_driver(index);
            let bus = buses.entry(self.class_of(index)).or_default();
            bus.reaches.insert(sheet);
            let Some(name) = name else { continue };
            if let Some(candidate) = ranked {
                if bus.driver.as_ref().is_none_or(|best| candidate < *best) {
                    bus.driver = Some(candidate);
                }
            }
            for member in bus_members_of(&name) {
                bus.carried
                    .entry(member.corresponds)
                    .or_default()
                    .insert(net_name(&member.name));
            }
        }
        buses
    }

    /// Every net-naming item of the graph, by the sheet and name it offers.
    fn nets_by_place(&mut self, rules: MergeRules) -> BTreeMap<(usize, String), Vec<usize>> {
        let mut by_place: BTreeMap<(usize, String), Vec<usize>> = BTreeMap::new();
        for named in self.naming_items(rules) {
            if self.nodes[named.node].carrier == Carrier::Net {
                by_place
                    .entry((named.sheet, named.name))
                    .or_default()
                    .push(named.node);
            }
        }
        by_place
    }

    /// How strong a driver one bus item is, and the scope it would name in.
    ///
    /// The answer is `(priority, qualified name, scope)`, which orders exactly
    /// as KiCad chooses a driver: by priority first, then by the name itself.
    /// The scope is the sheet the driver is drawn on, written the way KiCad
    /// prefixes a net name; a global label names in no sheet at all, so its
    /// scope is empty. The priority order is `CONNECTION_SUBGRAPH::PRIORITY`,
    /// which [`super::names`] follows for net names.
    fn rank_of_driver(&self, index: usize) -> Option<(u8, String, String, String)> {
        let node = &self.nodes[index];
        let text = name_of(node)?;
        let sheet = || self.sheets[node.sheet].human.clone();
        let (rank, scope) = match &node.kind {
            NodeKind::Label {
                kind: LabelKind::Global,
                ..
            } => (0, String::new()),
            NodeKind::Label {
                kind: LabelKind::Local,
                ..
            } => (1, sheet()),
            NodeKind::Label {
                kind: LabelKind::Hierarchical,
                ..
            } => (2, sheet()),
            NodeKind::SheetPin { .. } => (3, sheet()),
            _ => return None,
        };
        Some((rank, format!("{scope}{}", net_name(&text)), scope, text))
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

/// One item that names a net, as the name merges need it.
struct Named {
    node: usize,
    sheet: usize,
    name: String,
    /// Does the name carry across the project, or only across its sheet?
    everywhere: bool,
}

/// The name an item carries, if it carries one.
fn name_of(node: &Node) -> Option<String> {
    match &node.kind {
        NodeKind::Label { text, .. } => Some(text.clone()),
        NodeKind::SheetPin { name, .. } => Some(name.clone()),
        _ => None,
    }
}

/// One bus of the graph: what it carries, where it reaches, what names it.
#[derive(Default)]
struct Bus {
    /// The member net names it carries, by what they correspond to.
    carried: BTreeMap<Correspondence, BTreeSet<String>>,
    /// The sheets its items are drawn on.
    reaches: BTreeSet<usize>,
    /// Its strongest driver, as [`Graph::rank_of_driver`] reports one.
    driver: Option<(u8, String, String, String)>,
}

impl Bus {
    /// The member name the driver gives each correspondence.
    fn names_of_driver(&self) -> BTreeMap<Correspondence, String> {
        let Some((_, _, _, text)) = &self.driver else {
            return BTreeMap::new();
        };
        bus_members_of(text)
            .into_iter()
            .map(|member| (member.corresponds, net_name(&member.name)))
            .collect()
    }
}

/// What a bundle member corresponds to in another bundle on the same bus.
///
/// `CONNECTION_GRAPH::matchBusMember` compares vector members by index and
/// everything else by its own name. Measured against KiCad 10.0.5: `AA[0..2]`
/// against `BB[5..6]` joins `AA0` to `BB5`, so the index that counts is the
/// member's place in the range and not the number written in the name.
/// `UART{TX, RX}` against `UART_TRG{TX, RX}` joins by `TX` and `RX`, and
/// `ANALOG{A[0..1]}` against `BB[0..1]` joins by place, because a group whose
/// member is a vector holds vector members.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Correspondence {
    /// A vector member, by its place in the range.
    Place(usize),
    /// Any other member, by its own name without the group's.
    Name(String),
}

/// One member of a bundle: the net it names, and what it corresponds to.
struct BusMember {
    /// The net name the member stands for, such as `UART.RX`.
    name: String,
    /// What the member matches in another bundle on the same bus.
    corresponds: Correspondence,
}

/// The members a bundle name stands for, each with what it corresponds to.
///
/// A vector, `AN[0..7]`, stands for `AN0` to `AN7`. A group, `I2C{SCL, SDA}`,
/// stands for `I2C.SCL` and `I2C.SDA`: the group's name, a stop, then the
/// member's. A group with no name of its own, `{A B}`, stands for its members
/// alone, and a member may itself be a vector, so `ANALOG{A[0..5]}` stands for
/// `ANALOG.A0` to `ANALOG.A5` (`NET_SETTINGS::ParseBusVector` and
/// `ParseBusGroup`, and `SCH_CONNECTION::ConfigureFromLabel` for the stop).
fn bus_members_of(name: &str) -> Vec<BusMember> {
    let plain = unescape_net_name(name);
    // A group is read first: its own member list may hold a vector, and a
    // vector read out of `ANALOG{A[0..5]}` would take the brace for a prefix.
    let Some((prefix, members)) = group_parts(&plain) else {
        return vector_members(&plain)
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(place, name)| BusMember {
                name,
                corresponds: Correspondence::Place(place),
            })
            .collect();
    };
    let mut found = Vec::new();
    for member in members {
        // A member that is itself a vector keeps the place it holds in that
        // vector; one that is a plain name corresponds by that name.
        let expanded = bus_members_of(&member);
        let inner = if expanded.is_empty() {
            vec![BusMember {
                corresponds: Correspondence::Name(member.clone()),
                name: member,
            }]
        } else {
            expanded
        };
        for member in inner {
            found.push(BusMember {
                name: if prefix.is_empty() {
                    member.name
                } else {
                    format!("{prefix}.{}", member.name)
                },
                corresponds: member.corresponds,
            });
        }
    }
    found
}

/// The members of a vector, such as `AN[0..7]`.
///
/// The range may be written either way round, and a suffix of `+`, `-`, `P`
/// or `N` follows the index.
fn vector_members(plain: &str) -> Option<Vec<String>> {
    let open = plain.find('[')?;
    let rest = &plain[open + 1..];
    let close = rest.find(']')?;
    let suffix = &rest[close + 1..];
    if !suffix
        .chars()
        .all(|character| matches!(character, '+' | '-' | 'P' | 'N' | '}'))
    {
        return None;
    }
    let (first, last) = rest[..close].split_once("..")?;
    let (first, last) = (first.parse::<i64>().ok()?, last.parse::<i64>().ok()?);
    if first == last {
        return None;
    }
    let prefix = &plain[..open];
    Some(
        (first.min(last)..=first.max(last))
            .map(|index| format!("{prefix}{index}{suffix}"))
            .collect(),
    )
}

/// The name and the member list of a group, such as `USB{DP DM}`.
///
/// A space or a comma separates members. A brace that follows `$`, `~`, `^`
/// or `_` draws formatting, such as the overbar of `~{RESET}`, so it nests
/// inside a member name rather than ending the list.
fn group_parts(plain: &str) -> Option<(String, Vec<String>)> {
    let characters: Vec<char> = plain.chars().collect();
    let open = characters.iter().position(|&character| character == '{')?;
    if open > 0 && matches!(characters[open - 1], '$' | '~' | '^' | '_') {
        return None;
    }
    let prefix: String = characters[..open].iter().collect();
    if prefix.contains(' ') || prefix.contains('[') || prefix.contains(']') {
        return None;
    }

    let mut members = Vec::new();
    let mut member = String::new();
    let mut depth = 0_u32;
    for (index, &character) in characters.iter().enumerate().skip(open + 1) {
        match character {
            '{' => {
                if index == 0 || !matches!(characters[index - 1], '$' | '~' | '^' | '_') {
                    return None;
                }
                depth += 1;
                member.push('{');
            }
            '}' if depth > 0 => {
                depth -= 1;
                member.push('}');
            }
            '}' => {
                if !member.is_empty() {
                    members.push(member);
                }
                return Some((prefix, members));
            }
            ' ' | ',' if depth == 0 => {
                if !member.is_empty() {
                    members.push(std::mem::take(&mut member));
                }
            }
            other => member.push(other),
        }
    }
    None
}

/// The symbol as it is drawn on one placement.
///
/// The unit a symbol draws is a property of the sheet path, like the
/// reference designator: the `(unit …)` beside the `lib_id` is a cache of
/// whichever sheet was loaded last, and the truth is the instance record
/// (`SCH_SYMBOL::GetUnitSelection`). A sheet whose cache and instance
/// disagree draws the instance's unit, which is a different set of pins with
/// different numbers.
fn drawn_on(symbol: &Symbol, placement: &Placement) -> Symbol {
    let mut drawn = symbol.clone();
    if let Some(unit) = symbol
        .placements
        .iter()
        .find(|instance| instance.path == placement.path)
        .map(|instance| instance.unit)
    {
        drawn.unit = unit;
    }
    drawn
}

/// Does this symbol reach the board?
///
/// `(on_board no)` says it does not, and a netlist then lists no pin of it:
/// the exporter drops the node (`NETLIST_EXPORTER_XML::makeListOfNets`, the
/// `ResolveExcludedFromBoard` test). `(dnp yes)` and `(in_bom no)` do not:
/// a part that is not fitted still has a footprint on the board.
///
/// The item model does not hold this attribute, so it is read from the tree,
/// as a bus entry's size is.
fn on_board(doc: &Doc, node: NodeId) -> bool {
    let Some(list) = doc
        .children(node)
        .iter()
        .copied()
        .find(|&child| doc.head_is(child, "on_board"))
    else {
        return true;
    };
    doc.children(list)
        .get(1)
        .and_then(|&id| doc.atom_as_str(id))
        .is_none_or(|said| said != "no")
}

/// The net one pin names across the project, or the empty string.
///
/// A power symbol names a net through a power input: the rail says "I am +5V".
/// A power OUTPUT on a power symbol says only "something drives this net",
/// which is what `PWR_FLAG` is, and it names nothing. Merging those by value
/// joins every flagged net in the project into one, which is four rails on
/// KiCad's own CM5 demo.
///
/// An ordinary symbol names a net as well, through a **hidden** power input,
/// and the name is the pin's own name rather than the symbol's value. That is
/// how a chip with invisible `VCC` and `GND` pins reaches those rails with no
/// wire drawn (`SCH_PIN::IsGlobalPower`, the `!IsVisible()` case, and
/// `SCH_PIN::GetDefaultNetName`). A visible power input on an ordinary symbol
/// names nothing: it must be wired.
fn power_name(pin: &ResolvedPin, power_symbol: bool, value: &str) -> String {
    if pin.electrical != "power_in" {
        return String::new();
    }
    if power_symbol {
        value.to_owned()
    } else if pin.hidden {
        pin.name.clone()
    } else {
        String::new()
    }
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
    use super::{Carrier, Correspondence, carrier_of_name, on_segment};
    use crate::geometry::Point;

    /// The member names a bundle stands for, in order.
    fn bus_members(name: &str) -> Vec<String> {
        super::bus_members_of(name)
            .into_iter()
            .map(|member| member.name)
            .collect()
    }

    /// What each member of a bundle corresponds to, in order.
    fn corresponds(name: &str) -> Vec<Correspondence> {
        super::bus_members_of(name)
            .into_iter()
            .map(|member| member.corresponds)
            .collect()
    }

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

    #[test]
    fn a_bundle_name_expands_to_its_members() {
        assert_eq!(bus_members("AN[0..2]"), ["AN0", "AN1", "AN2"]);
        // The range may be written either way round.
        assert_eq!(bus_members("AN[2..0]"), ["AN0", "AN1", "AN2"]);
        // A suffix follows the index.
        assert_eq!(bus_members("D[0..1]P"), ["D0P", "D1P"]);
        // A group's name, a stop, then the member's.
        assert_eq!(bus_members("I2C{SCL, SDA}"), ["I2C.SCL", "I2C.SDA"]);
        assert_eq!(bus_members("USB{DP DM}"), ["USB.DP", "USB.DM"]);
        // A group with no name of its own carries its members alone.
        assert_eq!(bus_members("{A B}"), ["A", "B"]);
        // A member may be a vector, and an overbar stays inside a member.
        assert_eq!(
            bus_members("ANALOG{A[0..2]}"),
            ["ANALOG.A0", "ANALOG.A1", "ANALOG.A2"]
        );
        assert_eq!(bus_members("SWD{~{RESET}, IO}"), ["SWD.~{RESET}", "SWD.IO"]);
        // A plain name carries no members.
        assert!(bus_members("GND").is_empty());
        assert!(bus_members("~{RESET}").is_empty());
    }

    #[test]
    fn a_member_corresponds_by_place_in_a_vector_and_by_name_in_a_group() {
        use Correspondence::{Name, Place};
        // A vector member corresponds by its place, not by the number in its
        // name, so `AA[0..1]` and `BB[5..6]` correspond pair by pair.
        assert_eq!(corresponds("AA[0..1]"), [Place(0), Place(1)]);
        assert_eq!(corresponds("BB[5..6]"), [Place(0), Place(1)]);
        // A group member corresponds by its own name, without the group's, so
        // `UART{TX, RX}` and `UART_TRG{TX, RX}` correspond pair by pair.
        assert_eq!(
            corresponds("UART{TX, RX}"),
            [Name("TX".to_owned()), Name("RX".to_owned())]
        );
        assert_eq!(
            corresponds("UART_TRG{TX, RX}"),
            [Name("TX".to_owned()), Name("RX".to_owned())]
        );
        // A group whose member is a vector holds vector members, which is why
        // `ANALOG{A[0..1]}` corresponds to `BB[0..1]` and not by name.
        assert_eq!(corresponds("ANALOG{A[0..1]}"), [Place(0), Place(1)]);
    }
}
