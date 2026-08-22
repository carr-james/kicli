//! Where a placed symbol's pins are, and what may be connected to each one.
//!
//! The other three views describe a drawing. This one answers a question an
//! agent asks **before** it edits: *I am about to draw a wire onto this part —
//! where does it attach, and what will be accepted?* Both halves matter. A
//! coordinate alone leaves the caller to guess the rest, learn its guess was
//! wrong from a write command's refusal, and try again; that round trip is the
//! defect this view exists to remove (`tasks/dogfood.md`, run 1, defects 2 and
//! 6).
//!
//! **Nothing here writes, and nothing here can.** The module is handed a loaded
//! file and answers from it. `tests/pin_view_writes_nothing.rs` is the check
//! that says so about the whole command path rather than about this module.
//!
//! **The answer is built from the router's own machinery, never from a second
//! copy of it.** The connection point and the escape direction are
//! [`Terminal::of_pin`]. Whether a wire may take the escape step is
//! [`Obstacles::entering`], the same query the search makes at every step. A
//! view that re-derived either would drift from what the router accepts, and a
//! view of what the router accepts is the whole point.

use std::fmt::Write as _;

use serde_json::{Value, json};

use crate::connectivity::Nets;
use crate::edit::wire::wires_through;
use crate::geometry::pins::ResolvedPin;
use crate::geometry::{Iu, Point, resolve_pins};
use crate::model::items::{Mirror, SheetPath, Symbol, Uuid};
use crate::model::{LoadedFile, definition_of, read_library};
use crate::route::obstacles::{Obstacles, PinObstacle};
use crate::route::sheet::{Routed, SheetObjects};
use crate::route::terminal::has_room;
use crate::route::window::Window;
use crate::route::{Heading, Terminal};

/// The record letter every pin line starts with.
///
/// One letter, as every other view's records use, so a caller filters with
/// `grep` and never needs a parser.
const RECORD: char = 'P';

/// Why a request about a symbol's pins could not be answered.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PinsError {
    /// The placement carries no reference designator on this sheet path.
    #[error("{lib_id} has no reference designator on {sheet}, so its pins cannot be named")]
    Unreferenced {
        /// The library identifier the symbol was placed from.
        lib_id: String,
        /// The sheet path that was asked about.
        sheet: String,
    },
    /// The file embeds no definition for the placement, so it draws no pins.
    #[error(
        "{reference} is placed from {lib_id}, whose definition this file does not embed, \
         so kicli cannot say where its pins are. Run project check."
    )]
    NoDefinition {
        /// The reference designator.
        reference: String,
        /// The library identifier that resolved to nothing.
        lib_id: String,
    },
    /// The symbol has no pin of that number.
    #[error("{reference} has no pin {number}. It has {}.", .had.join(", "))]
    NoSuchPin {
        /// The reference designator.
        reference: String,
        /// The pin number that was asked for.
        number: String,
        /// The pin numbers the symbol does have.
        had: Vec<String>,
    },
}

/// One pin, with everything a caller needs to draw a wire to it.
#[derive(Clone, Debug)]
pub struct PinFacts {
    /// The pin number, as text. The `N` of a `REF.N` address.
    pub number: String,
    /// The pin name the library gives it. `~` is the library's own "no name".
    pub name: String,
    /// The electrical type, such as `passive` or `power_in`.
    pub electrical: String,
    /// Is the pin drawn by nothing? A hidden power pin still connects.
    pub hidden: bool,
    /// Where the pin connects. This is the point `--from-pin` resolves to.
    pub at: Point,
    /// Which way a wire must leave, when the pin fixes one.
    pub escape: Option<Heading>,
    /// The first point a wire from this pin may reach.
    ///
    /// A route must step one grid step along the escape direction before it may
    /// turn, so this point is a legal terminus for a one-segment wire from the
    /// pin. It is the pin's own position when the pin fixes no direction.
    pub escape_at: Point,
    /// Is the pin on the placement grid?
    ///
    /// A wire may not start off the grid: the router refuses rather than
    /// snapping, because moving somebody's pin is not a routing decision.
    pub on_grid: bool,
    /// What bars the escape step, when something does.
    ///
    /// A pin whose escape is barred can take no wire at all until the thing in
    /// the way moves.
    pub blocked_by: Option<String>,
    /// Do three wire ends already meet here?
    ///
    /// A route's own end would be the fourth, which `spec/SPEC.md` §9 Q2
    /// refuses; the router offsets the terminus by one grid step and reports
    /// the adjustment.
    pub crowded: bool,
    /// The net the pin is already on, when the drawing joins it to something.
    pub net: Option<String>,
}

impl PinFacts {
    /// The state words the record ends with, in a fixed order.
    ///
    /// Fixed, because a caller matching on the tail of a line must not have to
    /// care which of the conditions happen to hold.
    #[must_use]
    pub fn state(&self) -> Vec<String> {
        let mut words = vec![match &self.net {
            Some(name) => format!("net={name}"),
            None => "free".to_owned(),
        }];
        if !self.on_grid {
            words.push("off-grid".to_owned());
        }
        if let Some(handle) = &self.blocked_by {
            words.push(format!("blocked={handle}"));
        }
        if self.crowded {
            words.push("crowded".to_owned());
        }
        if self.hidden {
            words.push("hidden".to_owned());
        }
        words
    }

    /// Is this pin one a wire can be drawn to as the drawing stands?
    ///
    /// Being on a net does not disqualify a pin: joining another wire to a net
    /// is how a net grows. Being off the grid or boxed in does.
    #[must_use]
    pub fn is_reachable(&self) -> bool {
        self.on_grid && self.blocked_by.is_none()
    }
}

/// Every pin a request asked about, and the placement they belong to.
#[derive(Clone, Debug)]
pub struct Pins {
    /// The reference designator on the sheet path asked about.
    pub reference: String,
    /// The library identifier the symbol was placed from.
    pub lib_id: String,
    /// The sheet path the placement is on.
    pub sheet: SheetPath,
    /// The symbol's anchor.
    pub at: Point,
    /// The rotation written in the file.
    pub angle: i32,
    /// The mirror written in the file: `x`, `y`, or `-` for none.
    pub mirror: &'static str,
    /// The placement grid the escape points are one step of.
    pub grid: Iu,
    /// How many pins the symbol draws, before any filter.
    pub total: usize,
    /// Was the list narrowed, and by what?
    pub filter: Option<&'static str>,
    /// The pins, in the order the library draws them.
    pub pins: Vec<PinFacts>,
}

impl Pins {
    /// Answer about one placed symbol's pins.
    ///
    /// `number` names one pin; without it every pin of the placement answers.
    ///
    /// # Errors
    ///
    /// Returns [`PinsError`] when the placement carries no reference designator
    /// on this sheet path, when the file embeds no definition to draw pins
    /// from, or when the symbol has no pin of the number asked for.
    pub fn of(
        file: &LoadedFile,
        sheet: &SheetPath,
        symbol: &Symbol,
        nets: &Nets,
        number: Option<&str>,
        grid: Iu,
    ) -> Result<Self, PinsError> {
        let reference = symbol
            .reference_on(sheet)
            .ok_or_else(|| PinsError::Unreferenced {
                lib_id: symbol.lib_id.0.clone(),
                sheet: sheet.0.clone(),
            })?
            .0
            .clone();
        let resolved = drawn_pins(file, sheet, symbol, &reference)?;
        if let Some(wanted) = number {
            if !resolved.iter().any(|pin| pin.number == wanted) {
                return Err(PinsError::NoSuchPin {
                    reference,
                    number: wanted.to_owned(),
                    had: resolved.iter().map(|pin| pin.number.clone()).collect(),
                });
            }
        }

        Ok(Self {
            pins: facts(file, sheet, &reference, &resolved, nets, number, grid),
            reference,
            lib_id: symbol.lib_id.0.clone(),
            sheet: sheet.clone(),
            at: symbol.at,
            angle: symbol.angle.0,
            mirror: match symbol.mirror {
                Some(Mirror::X) => "x",
                Some(Mirror::Y) => "y",
                None => "-",
            },
            grid,
            total: resolved.len(),
            filter: None,
        })
    }

    /// The same answer with only the pins nothing is joined to yet.
    ///
    /// The count of what the symbol draws is kept, so a narrowed answer still
    /// says how much it left out.
    #[must_use]
    pub fn only_free(mut self) -> Self {
        self.pins.retain(|pin| pin.net.is_none());
        self.filter = Some("free");
        self
    }

    /// How many of the listed pins a wire could be drawn to as things stand.
    #[must_use]
    pub fn reachable(&self) -> usize {
        self.pins.iter().filter(|pin| pin.is_reachable()).count()
    }
}

/// What a rendered answer turned out to be.
///
/// The same idea `view::scope::Scope` carries for the other views, at the grain
/// this one works on: a symbol rather than a sheet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Listing {
    /// Every pin the request asked for, one record each.
    Records,
    /// Counts only, because the records would not fit the byte budget.
    Summary,
}

impl Listing {
    /// The word the output uses for this listing.
    #[must_use]
    pub fn token(self) -> &'static str {
        match self {
            Self::Records => "symbol",
            Self::Summary => "symbol-summary",
        }
    }
}

/// A rendered answer, and what it turned out to be.
#[derive(Clone, Debug)]
pub struct Rendered {
    /// The text of the answer.
    pub text: String,
    /// Whether it holds the records or stands in for them.
    pub listing: Listing,
    /// How many bytes the text is.
    pub bytes: usize,
}

/// Render the answer, falling back to counts when the records will not fit.
///
/// `max_bytes` is the budget from the project's configuration, the same knob
/// the other views fall back on. A forty-pin connector is the case this exists
/// for: Constitution §6 says a view that floods is wrong whatever it contains,
/// and the fallback says how to get the records back rather than truncating
/// them silently.
#[must_use]
pub fn render(pins: &Pins, max_bytes: usize) -> Rendered {
    let full = records(pins);
    if full.len() <= max_bytes {
        return Rendered {
            bytes: full.len(),
            text: full,
            listing: Listing::Records,
        };
    }
    let text = summary(pins, max_bytes, full.len());
    Rendered {
        bytes: text.len(),
        text,
        listing: Listing::Summary,
    }
}

/// The first line: which placement this is, and how it is drawn.
fn header(pins: &Pins, listing: Listing) -> String {
    format!(
        "# pins {} {}  sheet={}  at={} angle={} mirror={} grid={}  scope={}{}",
        pins.reference,
        pins.lib_id,
        pins.sheet.0,
        pins.at,
        pins.angle,
        pins.mirror,
        pins.grid,
        listing.token(),
        match pins.filter {
            Some(word) => format!("  filter={word}"),
            None => String::new(),
        }
    )
}

/// The records form: the header, the legend, one line per pin.
fn records(pins: &Pins) -> String {
    let mut out = header(pins, Listing::Records);
    out.push('\n');
    let _ = writeln!(out, "# {RECORD} num name type at heading escape state");
    for pin in &pins.pins {
        let _ = writeln!(
            out,
            "{RECORD} {} {} {} {} {} {} {}",
            word(&pin.number),
            word(&pin.name),
            word(&pin.electrical),
            pin.at,
            heading_token(pin.escape),
            pin.escape_at,
            pin.state().join(" "),
        );
    }
    if pins.pins.len() < pins.total {
        let _ = writeln!(
            out,
            "# {} of {} pin(s) listed; {}",
            pins.pins.len(),
            pins.total,
            match pins.filter {
                Some(_) => "drop --free to see the rest".to_owned(),
                None => format!("name {} alone to see them all", pins.reference),
            }
        );
    }
    out
}

/// One field of a record, never empty.
///
/// A library that names a pin nothing writes an empty string, and an empty
/// field would shorten the record by one word — which silently shifts every
/// field after it for a caller reading by position. `~` is the library's own
/// mark for a pin with no name, so it is the mark used here.
fn word(text: &str) -> &str {
    if text.is_empty() { "~" } else { text }
}

/// The counts form, for an answer too large to print in full.
fn summary(pins: &Pins, budget: usize, would_be: usize) -> String {
    let mut out = header(pins, Listing::Summary);
    let _ = writeln!(out, "  full={would_be}B budget={budget}B");
    let _ = writeln!(
        out,
        "# pins={} listed={} free={} reachable={}",
        pins.total,
        pins.pins.len(),
        pins.pins.iter().filter(|pin| pin.net.is_none()).count(),
        pins.reachable(),
    );
    let _ = writeln!(
        out,
        "# name one pin as {}.N to see it, narrow with --free, or raise view.max_bytes",
        pins.reference
    );
    out
}

/// The word for an escape direction.
///
/// The axes rather than the compass, as [`Heading`] itself is named: the
/// schematic's Y grows downwards and "up" would have to be explained at every
/// use. A pin that fixes no direction may be met from any side, which is `*`.
fn heading_token(heading: Option<Heading>) -> &'static str {
    match heading {
        Some(Heading::PlusX) => "+x",
        Some(Heading::MinusX) => "-x",
        Some(Heading::PlusY) => "+y",
        Some(Heading::MinusY) => "-y",
        None => "*",
    }
}

/// The same content as [`render`], as JSON.
///
/// Every key is present at both listings, as the route contract's own JSON is:
/// a caller parses one shape whatever came back, and `pins` is an empty list
/// rather than a missing key when the records did not fit.
#[must_use]
pub fn to_json(pins: &Pins, listing: Listing) -> Value {
    let records: Vec<Value> = if listing == Listing::Summary {
        Vec::new()
    } else {
        pins.pins
            .iter()
            .map(|pin| {
                json!({
                    "number": pin.number,
                    "name": pin.name,
                    "electrical": pin.electrical,
                    "hidden": pin.hidden,
                    "at": pin.at.to_string(),
                    "heading": heading_token(pin.escape),
                    "escape": pin.escape_at.to_string(),
                    "on_grid": pin.on_grid,
                    "blocked_by": pin.blocked_by,
                    "crowded": pin.crowded,
                    "net": pin.net,
                    "state": pin.state(),
                })
            })
            .collect()
    };
    json!({
        "reference": pins.reference,
        "lib_id": pins.lib_id,
        "sheet": pins.sheet.0,
        "at": pins.at.to_string(),
        "angle": pins.angle,
        "mirror": pins.mirror,
        "grid": pins.grid.to_string(),
        "scope": listing.token(),
        "filter": pins.filter,
        "total": pins.total,
        "listed": pins.pins.len(),
        "reachable": pins.reachable(),
        "pins": records,
    })
}

/// Every pin the placement draws, as the sheet path draws them.
///
/// The unit comes from the sheet path rather than from the cache beside the
/// `lib_id`, because a sheet placed twice draws a different unit on each
/// placement — the rule [`Symbol::drawn_on`] states.
fn drawn_pins(
    file: &LoadedFile,
    sheet: &SheetPath,
    symbol: &Symbol,
    reference: &str,
) -> Result<Vec<ResolvedPin>, PinsError> {
    let schematic = &file.schematic;
    let library = read_library(&file.doc, &schematic.library_symbols, schematic.version);
    let definition = definition_of(&library, symbol).ok_or_else(|| PinsError::NoDefinition {
        reference: reference.to_owned(),
        lib_id: symbol.lib_id.0.clone(),
    })?;
    Ok(resolve_pins(&symbol.drawn_on(sheet), definition))
}

/// One record per pin the request asked for.
///
/// The blockages are computed over **every** pin of the placement and then
/// filtered, rather than computed over the filtered list: a sibling pin is an
/// obstacle whether or not the caller asked to see it.
fn facts(
    file: &LoadedFile,
    sheet: &SheetPath,
    reference: &str,
    resolved: &[ResolvedPin],
    nets: &Nets,
    number: Option<&str>,
    grid: Iu,
) -> Vec<PinFacts> {
    let barred = blockages(file, sheet, reference, resolved, grid);
    resolved
        .iter()
        .zip(barred)
        .filter(|(pin, _)| number.is_none_or(|wanted| pin.number == wanted))
        .map(|(pin, blocked_by)| {
            let terminal = Terminal::of_pin(reference, pin);
            PinFacts {
                number: pin.number.clone(),
                name: pin.name.clone(),
                electrical: pin.electrical.clone(),
                hidden: pin.hidden,
                at: terminal.at,
                escape: terminal.escape,
                escape_at: terminal.escape_point(grid),
                on_grid: terminal.is_on_grid(grid),
                blocked_by,
                crowded: !has_room(terminal.at, &file.schematic),
                // A net of one pin joined to nothing is not a connection, so
                // the pin reads `free`. `Net::joins_nothing` is the one
                // implementation of that question and the connectivity view
                // asks it too, so the two views never disagree about one pin.
                net: nets
                    .net_of(reference, &pin.number)
                    .filter(|net| !net.joins_nothing())
                    .map(|net| net.name.clone()),
            }
        })
        .collect()
}

/// What bars each pin's escape step, in the order the pins were resolved.
///
/// The query is [`Obstacles::entering`] — the search's own, so the answer is
/// what the router would decide rather than a second opinion about it. Four
/// things follow from that and are worth stating.
///
/// **The pin being asked about is not an obstacle to itself.** The router
/// excludes the terminals a route ends on, so this reads the sheet once with
/// every pin of the placement excluded and puts the siblings back one at a
/// time. A sibling pin one grid step away, facing the wrong way, really does
/// bar the escape, and dropping it would answer that a barred pin is free.
///
/// **A route owns the wires already at its own end**, which is what
/// [`edit::wire::plan`] gives the search: a wire at the pin ends a route rather
/// than blocking it. Measured, not assumed — with those wires foreign, this
/// view reported `R20.1` of `tests/fixtures/sch/nets` as barred by
/// `1300004c`, and `wire connect --from-pin R20.1 --to-pin R21.2` on the same
/// drawing routed straight out through that point. A view that predicts a
/// refusal the tool does not make is worse than one that predicts nothing.
///
/// **An off-grid pin is not asked about.** The lattice has no node off the
/// grid, so the query would answer "page border" and blame the page for a
/// drawing fault. The record reports `off-grid` instead, which is the true
/// reason no wire may start there.
///
/// **A pin that fixes no direction takes no escape step**, so there is nothing
/// to bar.
///
/// [`edit::wire::plan`]: crate::edit::wire::plan
fn blockages(
    file: &LoadedFile,
    sheet: &SheetPath,
    reference: &str,
    resolved: &[ResolvedPin],
    grid: Iu,
) -> Vec<Option<String>> {
    let terminals: Vec<Terminal> = resolved
        .iter()
        .map(|pin| Terminal::of_pin(reference, pin))
        .collect();
    let handles: Vec<String> = terminals
        .iter()
        .map(|terminal| terminal.name.clone())
        .collect();
    let own: Vec<Uuid> = wires_through(
        &file.schematic,
        &terminals
            .iter()
            .map(|terminal| terminal.at)
            .collect::<Vec<_>>(),
    );
    let objects = SheetObjects::read(
        file,
        sheet,
        &Routed {
            wires: &own,
            terminals: &handles,
        },
    );
    let geometry = objects.geometry();
    let foreign: Vec<PinObstacle> = geometry.pins.to_vec();
    let page = objects.page();

    terminals
        .iter()
        .enumerate()
        .map(|(index, terminal)| {
            let heading = terminal.escape?;
            if !terminal.is_on_grid(grid) {
                return None;
            }
            let escape_at = terminal.escape_point(grid);
            let mut pins = foreign.clone();
            pins.extend(
                terminals
                    .iter()
                    .enumerate()
                    .filter(|(sibling, _)| *sibling != index)
                    .map(|(_, sibling)| PinObstacle {
                        handle: sibling.name.clone(),
                        at: sibling.at,
                        escape: sibling.escape,
                    }),
            );
            let mut sheet_geometry = geometry;
            sheet_geometry.pins = &pins;
            let window = Window::around(terminal.at, escape_at, grid, page, grid);
            Obstacles::build(window, &sheet_geometry)
                .entering(escape_at, heading)
                .blocked_by
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Heading, Listing, heading_token};

    #[test]
    fn every_heading_has_one_word_and_they_are_all_different() {
        let words: Vec<&str> = Heading::EVERY
            .into_iter()
            .map(Some)
            .map(heading_token)
            .collect();
        assert_eq!(words, ["+x", "-x", "+y", "-y"]);
        assert_eq!(heading_token(None), "*", "a pin with no direction");
    }

    #[test]
    fn a_record_field_is_never_empty() {
        // An empty field would shorten the record and shift every field after
        // it for a caller reading by position.
        assert_eq!(super::word(""), "~");
        assert_eq!(super::word("VCC"), "VCC");
    }

    #[test]
    fn a_listing_has_one_word_for_itself() {
        assert_eq!(Listing::Records.token(), "symbol");
        assert_eq!(Listing::Summary.token(), "symbol-summary");
    }
}
