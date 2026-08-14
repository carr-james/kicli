//! Net extraction from geometry and names.
//!
//! This module builds the net partition with union-find over wire endpoints,
//! segment interiors, pins, labels, and power-symbol values. The name-based
//! merges are mandatory. Geometry alone splits one ground net into many,
//! because power symbols connect by value and not by wire.
//!
//! Connectivity is whatever KiCad's netlister does. Every rule here is
//! measured against `kicad-cli sch export netlist` on a committed fixture, and
//! the rule text names the KiCad source it was read from. Where a rule and
//! KiCad disagree, KiCad is right and the rule is a bug.

mod graph;
mod names;

use crate::model::hierarchy::Hierarchy;
use crate::model::items::{Refdes, SheetPath, Uuid};

/// The merge rules, named, in the order the extractor applies them.
///
/// A rule joins items into one net. The specification and the research record
/// carry the same six names beside the evidence for each, and a test holds the
/// three lists together, so a rule cannot be changed in one place only.
///
/// # Examples
///
/// ```
/// use kicli::connectivity::MERGE_RULES;
/// assert_eq!(MERGE_RULES[0], "shared point");
/// ```
pub const MERGE_RULES: [&str; 6] = [
    "shared point",
    "junction",
    "label on a segment",
    "name",
    "power pin",
    "bundle member",
];

/// Which merge rules to apply.
///
/// Every set includes the geometric rules. The rest are switches, so that a
/// test can show what each rule is worth: geometry alone leaves one ground net
/// as one net per power symbol.
///
/// # Examples
///
/// ```
/// use kicli::connectivity::MergeRules;
/// assert!(MergeRules::ALL.power);
/// assert!(!MergeRules::GEOMETRY.labels);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MergeRules {
    /// Merge items that carry the same label name.
    pub labels: bool,
    /// Merge power-symbol pins whose symbol value is equal.
    pub power: bool,
}

impl MergeRules {
    /// Every rule: what a netlist needs.
    pub const ALL: Self = Self {
        labels: true,
        power: true,
    };
    /// The geometric rules alone.
    pub const GEOMETRY: Self = Self {
        labels: false,
        power: false,
    };
}

impl Default for MergeRules {
    fn default() -> Self {
        Self::ALL
    }
}

/// One pin of one placed symbol, at one place in the hierarchy.
///
/// A sheet placed twice gives every pin on it two entries, one per sheet path,
/// with a different reference designator each time.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NetPin {
    /// The reference designator on this sheet path.
    pub reference: Refdes,
    /// The pin number, as text.
    pub number: String,
    /// Which placement of which sheet the pin is on.
    pub sheet: SheetPath,
    /// The placed symbol the pin belongs to.
    pub symbol: Uuid,
    /// Is this a pin of a power symbol? A netlist leaves those out.
    pub power: bool,
    /// Does the symbol reach the board?
    ///
    /// `(on_board no)` marks a symbol that is drawn but not built, such as a
    /// test point or a fitting option. The drawing still joins its pins, and
    /// kicli still lists them, but a netlist leaves them out. `(dnp yes)` is
    /// a different thing and does not remove a pin: an unfitted part still
    /// has a footprint.
    pub on_board: bool,
}

impl NetPin {
    /// The pin as a netlist writes it, such as `R12.2`.
    #[must_use]
    pub fn label(&self) -> String {
        format!("{}.{}", self.reference.0, self.number)
    }
}

/// One net: everything the drawing joins into one conductor.
#[derive(Clone, Debug)]
pub struct Net {
    /// The name to show, and the handle to address the net by.
    pub name: String,
    /// Did kicli invent the name, because the drawing gives none?
    pub synthetic: bool,
    /// KiCad's own name for this net.
    ///
    /// It is an attribute, never a handle: KiCad names an unlabelled net after
    /// one of its pins, so renumbering that symbol renames the net. Agents
    /// need it to read ERC output and the editor.
    pub kicad_name: String,
    /// The pins of the net, sorted by reference designator then pin number.
    ///
    /// One pin number appears once per reference designator. A pin that a
    /// library puts in unit 0 is drawn by every unit of the symbol, so two
    /// units may carry the same pin onto one net; the net lists it once.
    pub pins: Vec<NetPin>,
    /// The sheet paths the net is drawn on, sorted.
    pub sheets: Vec<SheetPath>,
}

/// A construct kicli does not join the way KiCad does.
///
/// A warning never changes the partition. It reports a drawing kicli declines
/// to reproduce, so that a difference from KiCad is announced rather than
/// silent. Silence is the failure that costs an agent most: a net list that is
/// wrong and confident reads exactly like one that is right.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NetWarningKind {
    /// One bus carries a vector bundle and a plain group at once.
    ///
    /// KiCad matches a vector member by index and a group member by name, so a
    /// vector against a group puts every group member on the vector's first
    /// net and leaves the rest of the vector alone. That falls out of comparing
    /// an index against members that have none, and KiCad's own comment above
    /// the loop calls the algorithm hacky. kicli joins corresponding members
    /// only between two vectors or two groups, and reports this instead.
    MixedBundleKinds,
}

impl NetWarningKind {
    /// A stable name for the kind, for a machine to match on.
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            Self::MixedBundleKinds => "mixed-bundle-kinds",
        }
    }
}

/// One warning, and the drawing that raised it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NetWarning {
    /// Which construct this is.
    pub kind: NetWarningKind,
    /// The sheet path the construct is drawn on.
    pub sheet: SheetPath,
    /// The bundle names on the bus, sorted.
    pub names: Vec<String>,
}

impl NetWarning {
    /// One sentence for a person, naming the construct.
    #[must_use]
    pub fn message(&self) -> String {
        match self.kind {
            NetWarningKind::MixedBundleKinds => format!(
                "One bus carries a vector bundle and a group bundle: {}. \
                 KiCad joins their members in a way kicli does not reproduce, \
                 so these nets may differ from a KiCad net list.",
                self.names.join(", ")
            ),
        }
    }
}

/// The nets of one project.
#[derive(Clone, Debug, Default)]
pub struct Nets {
    nets: Vec<Net>,
    warnings: Vec<NetWarning>,
}

impl Nets {
    /// What kicli noticed but does not reproduce, sorted and without repeats.
    ///
    /// An empty list is the normal case. A reader that ignores this list may
    /// be reading a partition that differs from KiCad's.
    #[must_use]
    pub fn warnings(&self) -> &[NetWarning] {
        &self.warnings
    }

    /// The nets, ordered by descending pin count then by pin list.
    ///
    /// The order is a property of the design, so two runs over one project
    /// give the same list in the same order.
    #[must_use]
    pub fn nets(&self) -> &[Net] {
        &self.nets
    }

    /// The net one pin is on, by reference designator and pin number.
    #[must_use]
    pub fn net_of(&self, reference: &str, number: &str) -> Option<&Net> {
        self.nets.iter().find(|net| {
            net.pins
                .iter()
                .any(|pin| pin.reference.0 == reference && pin.number == number)
        })
    }
}

/// Extract every net of a loaded hierarchy.
///
/// A symbol whose library definition is missing from the file contributes no
/// pins, and a pin on a sheet path with no instance record is joined to its
/// net but is not listed: it has no reference designator to be listed under.
///
/// # Examples
///
/// ```no_run
/// use kicli::connectivity::extract;
/// use kicli::model::Hierarchy;
///
/// let hierarchy = Hierarchy::load(std::path::Path::new("root.kicad_sch"))?;
/// let nets = extract(&hierarchy);
/// println!("{} nets", nets.nets().len());
/// # Ok::<(), kicli::model::LoadError>(())
/// ```
#[must_use]
pub fn extract(hierarchy: &Hierarchy) -> Nets {
    extract_with(hierarchy, MergeRules::ALL)
}

/// Extract the nets of a loaded hierarchy under a chosen set of rules.
///
/// [`extract`] is this function with every rule on, which is the only set that
/// agrees with KiCad. The others exist to show what each rule is worth.
#[must_use]
pub fn extract_with(hierarchy: &Hierarchy, rules: MergeRules) -> Nets {
    let mut graph = graph::Graph::build(hierarchy, rules);
    let nets = names::nets_of(&mut graph);
    let mut warnings = std::mem::take(&mut graph.warnings);
    warnings.sort();
    warnings.dedup();
    Nets { nets, warnings }
}
