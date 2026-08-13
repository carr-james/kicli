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
    /// The pins of the net, sorted by reference designator then pin number.
    pub pins: Vec<NetPin>,
    /// The sheet paths the net is drawn on, sorted.
    pub sheets: Vec<SheetPath>,
}

/// The nets of one project.
#[derive(Clone, Debug, Default)]
pub struct Nets {
    nets: Vec<Net>,
}

impl Nets {
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
    let mut graph = graph::Graph::build(hierarchy);
    Nets {
        nets: names::nets_of(&mut graph),
    }
}
