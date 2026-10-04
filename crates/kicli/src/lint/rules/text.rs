//! `KI-TXT-001` — two pieces of visible text overlap.
//!
//! Two strings drawn on top of each other cannot be read. At the point of
//! overlap at least one of the two is gone, and which one is gone depends on
//! the draw order rather than on anything an author decided. That is the loss
//! of the marks the drawing is made of.
//!
//! # The tier, argued from the project's stated intent
//!
//! The rule's **existence** is well sourced. Graham Sutherland's schematic
//! review notes §10 say *"Don't run wires through text"* and work the case
//! through: *"R4's reference designator overlaps the `FB_OUT` net label"*.
//! Lathrop asks that a drawing *"doesn't collide with other parts of the
//! drawing"*. Its **tier** is sourced by neither, so the tier is argued from
//! the sentence this project's scoring work is answerable to:
//!
//! > The tool must validate the important aspects of quality schematics. It
//! > must never reward a schematic that is impossible to read and understand.
//!
//! 1. Overlapping text is the second sentence's subject taken literally. A
//!    string a reader cannot separate from the string on top of it is not
//!    *hard* to read; it is **not there**. The specification names this the
//!    failure mode that motivated the tool.
//! 2. Tier 2 cannot carry that claim, and the arithmetic says so rather than
//!    the prose. A scored rule's cost is `w · n · norm`, and the per-object
//!    normaliser is `1 / max(1, N_sym / 20)`. So a rule that fires once per
//!    object it counts cannot cost more than `w · 20` however large the sheet
//!    — a ceiling that does not move. A sheet whose every string sat on
//!    another would land in the same score band as a sheet with a handful of
//!    collisions, and a band like that is a reward.
//! 3. **That argument is recomputed for a pair-counted rule rather than
//!    borrowed from a per-object one, because the borrow does not hold.** This
//!    rule's occurrence is a *pair*, and `n` is not bounded by the object
//!    count: `m` mutually overlapping text objects give `m(m-1)/2` pairs, so
//!    four of them give six findings against four objects. `n ≤ N` is false
//!    here. The conclusion survives the correction and gets **stronger**: an
//!    unbounded `n` against a fixed ceiling means the normalised cost of a
//!    totally illegible sheet still saturates at `w · 20`, while the raw count
//!    grows quadratically. The worse the sheet, the further the score is from
//!    describing it.
//! 4. Tier 1 is the only tier whose mechanism matches the claim: one
//!    occurrence fails the gate, and the verdict does not depend on how much
//!    of the sheet was lost.
//!
//! The fourth point is what the tier rests on, and the rule earns it by having
//! **no tolerance in its comparison**: an exact integer area against an exact
//! integer threshold, so the gate fires where text really is hidden.
//!
//! The published ratio is `text.overlap_ratio = 0.2`, and it is a *detection*
//! threshold rather than a tolerance in the verdict: it is what separates two
//! strings sharing a pen's worth of ink at their edges from two strings a
//! reader cannot untangle. It is compared by cross multiplication, so no
//! division is evaluated anywhere — see [`exceeds_ratio`].
//!
//! # Oriented boxes, not axis-aligned ones. This is the whole rule.
//!
//! Schematic text is routinely at 90 degrees. A box built for a turned string
//! and left unturned is wrong in **both** directions: it claims overlaps that
//! are not there and misses overlaps that are. So this rule compares the boxes
//! [`crate::geometry::TextBox::corners`] gives it — the four page-space corners
//! **after** the string's own rotation — and never
//! [`crate::geometry::TextBox::bounds`], which is the same box left unturned.
//!
//! [`crate::geometry::symbol_boxes`] calls `.axis_aligned()` on each text box
//! while building the full box, because a union of boxes is an axis-aligned
//! answer by construction. The oriented box is available one call upstream of
//! that, and this rule takes it there.
//!
//! **Measured, and worth stating because it narrows what a check can prove:**
//! at the four right angles a turned box *is* axis-aligned on the page, so
//! `.axis_aligned()` and the oriented box agree there exactly. The two part
//! company only off the right angles. The lookalike that bites a real drawing
//! is therefore the **unturned** box, not the axis-aligned one, and that is the
//! lookalike the checks in this file and in `tests/lint_overlapping_text.rs`
//! are built against.
//!
//! # Which box, and why it is the other one from `KI-OVL-001`
//!
//! The specification assigns the two boxes by rule family in one sentence:
//! *"**body box** (graphics + pins, no text) for overlap rules, and **full
//! box** (∪ visible field boxes) for text-collision rules"*. This is a
//! text-collision rule, so it takes the full box's side of that sentence.
//!
//! It takes the **constituent visible text boxes the full box is built from**,
//! not the union itself, and that is forced rather than chosen: the union
//! answers *"does this symbol's drawing reach there"* and cannot answer *"which
//! two strings are on top of each other"*. Reading
//! [`crate::geometry::SymbolBoxes::body`] here instead — the box `KI-OVL-001`
//! takes — would make both rules wrong in ways that look right on a
//! screenshot.
//!
//! # "Visible" is load-bearing, and it means the field flag
//!
//! A hidden field draws nothing, so it cannot overlap anything.
//! [`crate::model::items::Field::hidden`] is the one answer to that question
//! and it already covers the format split: the `hide` token sits inside
//! `effects` before format stamp 20251028 and beside `show_name` after it, and
//! a reader that looked in one place only would treat every hidden field of an
//! older file as drawn.
//!
//! Visibility in a schematic is a **property of a field**, and of nothing else.
//! A label, a piece of free text and a sheet pin carry no such flag, in the
//! file or in the editor, so they are always drawn and this rule always
//! compares them.
//!
//! # Where this rule under-reports, and never over-reports
//!
//! A blocking rule must err towards silence, so each gap below is a missed
//! finding rather than a false one.
//!
//! - **Pin names and numbers are not compared.** The catalogue lists them.
//!   They are laid out in library space inside
//!   [`crate::geometry::symbol_boxes`], behind a private function, and the
//!   geometry module exposes no per-pin text box. Reaching them means either a
//!   second copy of that layout here — which the engineering standards call a
//!   defect — or a widening of the geometry module, which this rule may not
//!   make. Sutherland's own worked case is a field against a label, which this
//!   rule does compare.
//! - **A box a non-stroke font made approximate is skipped.** kicli measures
//!   KiCad's stroke font exactly and has no font file to read for any other
//!   face, so such a box is a guess. The specification asks that a finding
//!   standing on one be marked `approximate`, and a [`crate::lint::Finding`]
//!   carries no field to mark it with. A blocking verdict taken from a guessed
//!   box is the one outcome this rule must not produce, so the pair is skipped
//!   instead.
//! - **A pair whose shared area is not an exact integer is skipped.** Two boxes
//!   at a relative angle that is not a multiple of 90 degrees meet at corners
//!   whose coordinates are fractions, and an exact area then needs arithmetic
//!   wider than this crate carries, while floating point is forbidden
//!   outright. [`shared_region`] reports that case rather than rounding it, and
//!   the rule stays silent. KiCad's schematic editor writes text at the four
//!   right angles only, where the question never arises.
//! - **A box with no area is skipped.** An empty field value and a zero text
//!   size both produce one, and neither draws anything to collide with.
//!
//! # Why the shared area is tested rather than the two boxes
//!
//! The lookalike is four comparisons between the boxes' own extents. It agrees
//! with this rule wherever both boxes have an area and disagrees where one does
//! not: a box of no size strictly inside another passes all four comparisons
//! while sharing an area of nothing. Testing the region states the rule once,
//! for every input, and the degenerate cases fall out of it.

use kicli_sexpr::{Doc, NodeId};

use crate::geometry::text::{TextStyle, text_box};
use crate::geometry::{Angle, Point};
use crate::lint::gate::Saturation;
use crate::lint::{Drawing, Findings, Rule, RuleId, Tier};
use crate::model::items::{Field, Item, Uuid};

/// The share of the smaller box that must be covered before the pair reports.
///
/// The published knob is `text.overlap_ratio = 0.2`, written here as the
/// percentage it is so that the comparison needs no division. The pair is a
/// numerator over a denominator and nothing evaluates it.
const OVERLAP_RATIO: (i128, i128) = (20, 100);

/// One piece of drawn text, reduced to what this rule compares.
struct Drawn {
    /// What a finding calls it.
    ///
    /// A field is named by its owner and its own field name. Everything else
    /// is named by its kind and the handle an agent can type back at a
    /// command, because a label's or a free text's only other name is its own
    /// string and a string has no length bound.
    name: String,
    /// The object a finding names. A field has no identifier of its own, so a
    /// field is reported against the item that owns it.
    object: Uuid,
    /// The four page-space corners, after the string's own rotation.
    quad: [Point; 4],
    /// Twice the area of that quadrilateral. Twice, because a shoelace sum is
    /// exactly twice an area and halving it could lose a unit.
    twice_area: i128,
}

/// Two pieces of visible text overlap.
pub struct OverlappingText;

impl Rule for OverlappingText {
    fn id(&self) -> RuleId {
        RuleId("KI-TXT-001")
    }

    fn tier(&self) -> Tier {
        Tier::One
    }

    /// This rule never saturates, and the declaration is a measurement rather
    /// than a shrug.
    ///
    /// **No denominator counts pairs.** [`crate::lint::gate::Counted`] offers
    /// symbols, wires or nothing. A finding here is one *pair of text
    /// objects*, and a sheet carrying `m` text objects holds `m(m-1)/2` of
    /// them, so a pair count measured against a symbol count is a ratio
    /// between two different things. `KI-WIRE-001` could declare
    /// `Counted::Wires` at a share of one half because for it `n ≤ total` is
    /// exactly true; here it is false, and four mutually overlapping strings
    /// are the counterexample — six findings against four objects.
    ///
    /// **And the symbol count is not even a count of text.** A sheet of
    /// nothing but labels and free text holds no symbols at all, so the
    /// denominator would be zero on a drawing this rule has plenty to say
    /// about — which [`crate::lint::gate::Saturation::is_reached`] reads as
    /// *never saturates* anyway. Declaring it would be a false declaration
    /// that happens to be inert, which is worse than declaring nothing.
    ///
    /// **It could not change an outcome in any case.**
    /// [`crate::lint::gate::Gate::of`] answers a blocking rule before it asks
    /// what share of the sheet the rule covered, so one overlapping pair and
    /// every pair on the sheet are the same verdict.
    fn saturation(&self) -> Saturation {
        Saturation::NEVER
    }

    fn examine(&self, drawing: &Drawing<'_>, found: &mut Findings<'_>) {
        let drawn = drawn_text(drawing);
        for (place, one) in drawn.iter().enumerate() {
            for two in drawn.iter().skip(place + 1) {
                let smaller = one.twice_area.min(two.twice_area);
                let Some(region) = shared_region(&one.quad, &two.quad) else {
                    continue;
                };
                let shared = twice_area(&region).abs();
                if !exceeds_ratio(shared, smaller) {
                    continue;
                }
                found.record(
                    middle(&region),
                    vec![one.object.clone(), two.object.clone()],
                    format!(
                        "{} and {} overlap, {} % of the smaller",
                        one.name,
                        two.name,
                        percentage(shared, smaller)
                    ),
                );
            }
        }
    }
}

/// Does the shared area pass the published ratio?
///
/// Both arguments are **twice** an area, so the factor of two is on both sides
/// of the comparison and cancels. The test is a cross multiplication and holds
/// no division, which is what makes it exact: `100 · shared > 20 · smaller` is
/// the same question as `shared / smaller > 20 / 100` and is answerable in
/// integers.
///
/// The comparison is **strict**, which is the rule as published: *"area > 20 %
/// of `min(area(a), area(b))`"*. A pair sharing exactly a fifth of the smaller
/// box does not report. One square internal unit more does. That single
/// character is this rule's entire tolerance, and
/// `the_ratio_boundary_is_one_square_unit_wide` is what holds it down.
///
/// A smaller box of no area reports nothing, whatever is shared: the
/// comparison becomes `100 · shared > 0`, and nothing can be shared with a box
/// that has no area.
fn exceeds_ratio(twice_shared: i128, twice_smaller: i128) -> bool {
    twice_shared * OVERLAP_RATIO.1 > OVERLAP_RATIO.0 * twice_smaller
}

/// What share of the smaller box is covered, in whole percent, for the message.
///
/// Integer division truncates, so the number a reader sees is never more than
/// the share really covered.
fn percentage(twice_shared: i128, twice_smaller: i128) -> i128 {
    if twice_smaller == 0 {
        return 0;
    }
    twice_shared * OVERLAP_RATIO.1 / twice_smaller
}

/// Twice the signed area of a polygon, by the shoelace sum.
///
/// The sum is exact in every case this rule meets: a coordinate is an `i32`, a
/// product of two of them fits a `i128` with room to spare, and a polygon here
/// has at most eight corners. The sign follows the winding, so callers take the
/// magnitude.
fn twice_area(points: &[Point]) -> i128 {
    let mut total: i128 = 0;
    for (place, &current) in points.iter().enumerate() {
        let next = points[(place + 1) % points.len()];
        total += i128::from(current.x.0) * i128::from(next.y.0)
            - i128::from(next.x.0) * i128::from(current.y.0);
    }
    total
}

/// Which side of a directed line a point falls on.
///
/// Positive is the left of `from -> to` under the winding
/// [`crate::geometry::TextBox::corners`] produces, zero is on the line. The
/// arithmetic widens before it subtracts, because a difference of two `i32`
/// coordinates does not always fit an `i32`.
fn side(from: Point, to: Point, at: Point) -> i128 {
    let run = i128::from(to.x.0) - i128::from(from.x.0);
    let rise = i128::from(to.y.0) - i128::from(from.y.0);
    let across = i128::from(at.x.0) - i128::from(from.x.0);
    let along = i128::from(at.y.0) - i128::from(from.y.0);
    run * along - rise * across
}

/// Where a segment crosses a directed line, when the crossing is a whole
/// number of internal units.
///
/// Returns nothing when it is not. A crossing of two edges at a relative angle
/// that is not a right angle is a fraction, and the exact area of a region with
/// a fractional corner needs a common denominator this arithmetic cannot carry.
/// Reporting the case is the honest answer; rounding it would put a rounding
/// error inside a blocking verdict.
fn crossing(from: Point, to: Point, one: Point, two: Point) -> Option<Point> {
    let at_one = side(from, to, one);
    let at_two = side(from, to, two);
    let span = at_one - at_two;
    if span == 0 {
        return None;
    }
    let moved = |start: i32, end: i32| {
        let step = (i128::from(end) - i128::from(start)) * at_one;
        if step % span != 0 {
            return None;
        }
        i32::try_from(i128::from(start) + step / span).ok()
    };
    Some(Point::new(
        moved(one.x.0, two.x.0)?,
        moved(one.y.0, two.y.0)?,
    ))
}

/// The part of a convex polygon on the inner side of one directed edge.
///
/// This is one step of the standard convex clip: walk the corners, keep the
/// ones inside, and add a crossing wherever the walk passes through the edge.
/// Returns nothing when a crossing is not a whole number of internal units.
fn clipped(subject: &[Point], from: Point, to: Point) -> Option<Vec<Point>> {
    let mut kept = Vec::with_capacity(subject.len() + 1);
    for (place, &current) in subject.iter().enumerate() {
        let previous = subject[(place + subject.len() - 1) % subject.len()];
        let was_inside = side(from, to, previous) >= 0;
        let is_inside = side(from, to, current) >= 0;
        if was_inside != is_inside {
            kept.push(crossing(from, to, previous, current)?);
        }
        if is_inside {
            kept.push(current);
        }
    }
    Some(kept)
}

/// The region two oriented boxes share, as a polygon, when it is exact.
///
/// The first box is clipped by each edge of the second. The clip polygon's
/// winding decides which side of an edge counts as inside, so it is normalised
/// first: a quadrilateral that arrives wound the other way would have every
/// inside test inverted and would report the complement of the answer.
///
/// Returns an empty polygon when the two share nothing, and nothing at all
/// when the shared region has a corner that is not a whole number of internal
/// units.
fn shared_region(one: &[Point; 4], two: &[Point; 4]) -> Option<Vec<Point>> {
    let window = positively_wound(*two);
    let mut region = one.to_vec();
    for place in 0..window.len() {
        if region.is_empty() {
            return Some(region);
        }
        region = clipped(&region, window[place], window[(place + 1) % window.len()])?;
    }
    Some(region)
}

/// The same quadrilateral, wound so that its inside is the left of each edge.
fn positively_wound(quad: [Point; 4]) -> [Point; 4] {
    if twice_area(&quad) < 0 {
        let [first, second, third, fourth] = quad;
        return [fourth, third, second, first];
    }
    quad
}

/// A point inside a convex region: the mean of its corners.
///
/// The mean of a convex polygon's corners is a convex combination of them, so
/// it is inside the polygon. It is where a finding points, which is the shared
/// ink rather than either string's anchor.
fn middle(region: &[Point]) -> Point {
    if region.is_empty() {
        return Point::default();
    }
    let count = i64::try_from(region.len()).unwrap_or(1);
    let mut across: i64 = 0;
    let mut down: i64 = 0;
    for corner in region {
        across += i64::from(corner.x.0);
        down += i64::from(corner.y.0);
    }
    let narrow = |value: i64| i32::try_from(value / count).unwrap_or_default();
    Point::new(narrow(across), narrow(down))
}

/// Every piece of visible text this placement draws, in file order.
///
/// File order makes the pair walk deterministic and makes the first-named
/// object of a finding the one the file holds first.
///
/// Pin names and numbers are absent, and the module header says why.
fn drawn_text(drawing: &Drawing<'_>) -> Vec<Drawn> {
    let sheet = drawing.path();
    let doc = drawing.doc();
    let mut drawn = Vec::new();
    for item in &drawing.schematic().items {
        match item {
            Item::Symbol(symbol) => {
                // The instance record carries the reference designator of this
                // placement, which is what a reader sees drawn.
                let placed = symbol.drawn_on(sheet);
                let owner = named(symbol.reference_on(sheet).map(|refdes| refdes.0.as_str()), &symbol.uuid);
                for field in &placed.fields {
                    add_field(&mut drawn, doc, field, &owner, &symbol.uuid);
                }
            }
            Item::Label(label) => {
                add(
                    &mut drawn,
                    doc,
                    &label.text,
                    label.at,
                    label.angle,
                    label.node,
                    format!("label {}", label.uuid.short()),
                    &label.uuid,
                );
                let owner = format!("label {}", label.uuid.short());
                for field in &label.fields {
                    add_field(&mut drawn, doc, field, &owner, &label.uuid);
                }
            }
            Item::Text(text) => add(
                &mut drawn,
                doc,
                &text.text,
                text.at,
                text.angle,
                text.node,
                format!("text {}", text.uuid.short()),
                &text.uuid,
            ),
            Item::Sheet(child) => {
                let owner = named(child.name(), &child.uuid);
                for field in &child.fields {
                    add_field(&mut drawn, doc, field, &owner, &child.uuid);
                }
                for pin in &child.pins {
                    add(
                        &mut drawn,
                        doc,
                        &pin.name,
                        pin.at,
                        pin.angle,
                        pin.node,
                        format!("pin {}", pin.uuid.short()),
                        &pin.uuid,
                    );
                }
            }
            _ => {}
        }
    }
    drawn
}

/// The name an owner is reported under: what it is called, or its handle.
fn named(name: Option<&str>, uuid: &Uuid) -> String {
    name.map_or_else(|| uuid.short().to_owned(), str::to_owned)
}

/// Add one field, if it is drawn at all.
fn add_field(drawn: &mut Vec<Drawn>, doc: &Doc, field: &Field, owner: &str, object: &Uuid) {
    if field.hidden {
        return;
    }
    add(
        drawn,
        doc,
        &field.value,
        field.at,
        field.angle,
        field.node,
        format!("{owner}.{}", field.name),
        object,
    );
}

/// Add one piece of text, if it draws any marks at all.
///
/// **A string with no mark in it is skipped, and that is not a nicety.**
/// KiCad's own writer gives every placed symbol a `Footprint`, a `Datasheet`
/// and a `Description` field, leaves all three **visible**, leaves all three
/// **empty**, and stacks all three on one anchor — measured on a schematic
/// `kicad-cli sch upgrade` wrote. The port gives an empty string the box of its
/// own pen rather than a box of nothing, because that is what
/// `EDA_TEXT::GetTextBox` does, so those three boxes are real, identical and
/// on top of each other. A rule that counted them would report **three
/// blocking findings on every symbol of every schematic KiCad has ever
/// written**, which is a false finding on a correct drawing rather than a gap.
///
/// Whitespace is skipped on the same ground, one step further out: a string of
/// spaces has a real advance and draws no marks, so it cannot hide another
/// string's marks.
#[allow(clippy::too_many_arguments, reason = "one call site per item kind")]
fn add(
    drawn: &mut Vec<Drawn>,
    doc: &Doc,
    text: &str,
    at: Point,
    angle: Angle,
    node: NodeId,
    name: String,
    object: &Uuid,
) {
    if text.trim().is_empty() {
        return;
    }
    let style = TextStyle::read(doc, node);
    let boxed = text_box(text, at, angle, &style);
    // A guessed box must not reach a blocking verdict: see the module header.
    if boxed.is_approximate() {
        return;
    }
    let quad = boxed.corners();
    let twice = twice_area(&quad).abs();
    if twice == 0 {
        return;
    }
    drawn.push(Drawn {
        name,
        object: object.clone(),
        quad,
        twice_area: twice,
    });
}

/// The registered rule.
pub static TEXT_OVERLAP: OverlappingText = OverlappingText;

/// The rules this file declares.
pub static RULES: &[&'static dyn Rule] = &[&TEXT_OVERLAP];
