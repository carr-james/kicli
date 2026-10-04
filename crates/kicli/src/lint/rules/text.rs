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
//! - **A pair whose shared area is not an exact integer is skipped.** Two
//!   boxes can meet at a corner whose coordinates are fractions of an internal
//!   unit, and an exact area then needs arithmetic wider than this crate
//!   carries, while floating point is forbidden outright. [`shared_region`]
//!   reports that case rather than rounding it, and the rule stays silent.
//!   KiCad's schematic editor writes text at the four right angles only, where
//!   the question never arises.
//!
//!   **Measured, because the first wording of this bullet was wrong**: it is
//!   not the relative angle that decides, it is where the edges cross. A box
//!   turned by an eighth of a turn has edges of slope exactly one once the
//!   rotation has rounded, so it crosses an axis-aligned edge on a whole unit
//!   and the region is exact — 0, 45, 90 and 135 degrees all answer. The
//!   twelfth turns are the ones that decline. The sweep is in
//!   `tests::a_relative_angle_that_is_not_a_right_angle_is_declined_rather_than_rounded`.
//! - **A box with no area is skipped.** An empty field value and a zero text
//!   size both produce one, and neither draws anything to collide with.
//!
//! # Why the shared area is tested rather than the two boxes
//!
//! The lookalike is four comparisons between the boxes' own extents. It agrees
//! with this rule wherever both boxes have an area and disagrees where one does
//! not: a box of no size strictly inside another passes all four comparisons
//! while sharing an area of nothing. Testing the region states the rule once,
//! for every input.
//!
//! **The degenerate cases do not all fall out of the region test, and the
//! measurement is in `tests::a_box_of_no_area_shares_none_where_the_extent_
//! lookalike_says_it_does`.** A box of no area clipped *by* a real box keeps
//! none, as it should. The other way round the clip is not symmetric: clipping
//! a real box by a *point* keeps the whole subject, because every inside test
//! against an edge of no length answers zero and keeps everything. So a box
//! with no area is dropped in [`add`] before the pair walk rather than left to
//! the region test, and that guard is load-bearing rather than defensive.

use kicli_sexpr::{Doc, NodeId};

use crate::geometry::text::{TextStyle, text_box};
use crate::geometry::{Angle, Point};
use crate::lint::gate::Saturation;
use crate::lint::{Drawing, Findings, Rule, RuleId, Tier};
use crate::model::items::{Field, Item, Label, SheetItem, SheetPath, Symbol, Uuid};

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
/// Returns nothing when it is not. Two edges can cross a fraction of a unit
/// from any whole one, and the exact area of a region with a fractional corner
/// needs a common denominator this arithmetic cannot carry. Reporting the case
/// is the honest answer; rounding it would put a rounding error inside a
/// blocking verdict.
///
/// It is **not** the relative angle that decides this. Two boxes an eighth of
/// a turn apart cross on whole units, because the rotation leaves an edge of
/// slope exactly one; two a twelfth of a turn apart do not. Measured by the
/// angle sweep in this file's `tests`.
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
/// One arm per item kind, each in its own function: the four kinds share no
/// code, and holding them in one `match` put this function over the line
/// budget the engineering gate enforces.
///
/// Pin names and numbers are absent, and the module header says why.
fn drawn_text(drawing: &Drawing<'_>) -> Vec<Drawn> {
    let sheet = drawing.path();
    let doc = drawing.doc();
    let mut drawn = Vec::new();
    for item in &drawing.schematic().items {
        match item {
            Item::Symbol(symbol) => add_symbol(&mut drawn, doc, symbol, sheet),
            Item::Label(label) => add_label(&mut drawn, doc, label),
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
            Item::Sheet(child) => add_sheet(&mut drawn, doc, child),
            _ => {}
        }
    }
    drawn
}

/// Every visible field of one symbol placement.
///
/// The instance record carries the reference designator of *this* placement,
/// which is what a reader sees drawn, so the owner's name is read from there
/// rather than from the symbol's own `Reference` field.
fn add_symbol(drawn: &mut Vec<Drawn>, doc: &Doc, symbol: &Symbol, sheet: &SheetPath) {
    let placed = symbol.drawn_on(sheet);
    let owner = named(
        symbol.reference_on(sheet).map(|refdes| refdes.0.as_str()),
        &symbol.uuid,
    );
    for field in &placed.fields {
        add_field(drawn, doc, field, &owner, &symbol.uuid);
    }
}

/// One label's own string, and then the fields a label may carry.
fn add_label(drawn: &mut Vec<Drawn>, doc: &Doc, label: &Label) {
    let owner = format!("label {}", label.uuid.short());
    add(
        drawn,
        doc,
        &label.text,
        label.at,
        label.angle,
        label.node,
        owner.clone(),
        &label.uuid,
    );
    for field in &label.fields {
        add_field(drawn, doc, field, &owner, &label.uuid);
    }
}

/// One child sheet's visible fields and the names drawn on its pins.
fn add_sheet(drawn: &mut Vec<Drawn>, doc: &Doc, child: &SheetItem) {
    let owner = named(child.name(), &child.uuid);
    for field in &child.fields {
        add_field(drawn, doc, field, &owner, &child.uuid);
    }
    for pin in &child.pins {
        add(
            drawn,
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

#[cfg(test)]
mod tests {
    use super::{clipped, exceeds_ratio, percentage, positively_wound, shared_region, twice_area};
    use crate::geometry::text::{TextStyle, text_box};
    use crate::geometry::{Angle, Point, Rect};

    /// An axis-aligned quadrilateral from two opposite corners.
    ///
    /// Wound the way [`Rect::corners`] winds, which is the winding
    /// [`crate::geometry::TextBox::corners`] produces and the one the rule's
    /// shoelace sum reads as positive.
    fn quad(start: (i32, i32), end: (i32, i32)) -> [Point; 4] {
        Rect::new(Point::new(start.0, start.1), Point::new(end.0, end.1)).corners()
    }

    /// Twice the area the rule finds two quadrilaterals share, or nothing when
    /// the shared region has a corner that is not a whole internal unit.
    fn twice_shared(one: &[Point; 4], two: &[Point; 4]) -> Option<i128> {
        Some(twice_area(&shared_region(one, two)?).abs())
    }

    /// The lookalike this rule is not: four comparisons between two extents.
    ///
    /// Written here rather than taken from the rule, so that the two answers
    /// are derived separately and can be seen to disagree.
    fn extents_overlap(one: &[Point; 4], two: &[Point; 4]) -> bool {
        let span = |quad: &[Point; 4]| {
            let xs = quad.map(|corner| corner.x.0);
            let ys = quad.map(|corner| corner.y.0);
            (
                xs.iter().copied().min().unwrap_or_default(),
                xs.iter().copied().max().unwrap_or_default(),
                ys.iter().copied().min().unwrap_or_default(),
                ys.iter().copied().max().unwrap_or_default(),
            )
        };
        let (one_left, one_right, one_top, one_bottom) = span(one);
        let (two_left, two_right, two_top, two_bottom) = span(two);
        one_left <= two_right
            && two_left <= one_right
            && one_top <= two_bottom
            && two_top <= one_bottom
    }

    /// Exactly a fifth does not report and one square internal unit more does.
    ///
    /// This grain exists because **exactly 20 % is not reachable through a
    /// drawing**: the threshold is `20 · area / 100`, so a box whose area is
    /// not a multiple of five has no arrangement that lands on the line.
    /// `tests/lint_overlapping_text.rs` measures the same boundary through two
    /// written drawings one internal unit apart, which is the finest a KiCad
    /// coordinate can express; this measures the comparison where the `>` is
    /// one character from a `>=`.
    #[test]
    fn the_ratio_boundary_is_one_square_unit_wide() {
        // Both arguments are twice an area, as the rule holds them. A smaller
        // box of 500 square units puts the threshold at exactly 100.
        let smaller = 2 * 500;
        assert!(
            !exceeds_ratio(2 * 100, smaller),
            "exactly a fifth does not report"
        );
        assert!(
            exceeds_ratio(2 * 100 + 2, smaller),
            "one square internal unit more does"
        );
        assert!(
            !exceeds_ratio(2 * 100 - 2, smaller),
            "one square internal unit less does not"
        );

        // The anti-vacuity control: the comparison is not a constant answer in
        // either direction.
        assert!(exceeds_ratio(smaller, smaller), "a total overlap reports");
        assert!(!exceeds_ratio(0, smaller), "no shared area does not");

        // The share a reader is shown truncates, so it never overstates what
        // is covered.
        assert_eq!(percentage(2 * 100 + 2, smaller), 20);
        assert_eq!(percentage(2 * 100 - 2, smaller), 19);
        assert_eq!(percentage(smaller, smaller), 100);
        assert_eq!(
            percentage(5, 0),
            0,
            "and a box of no area is not divided by"
        );
    }

    /// A shared edge and a touched corner hold no area; one unit in holds one.
    ///
    /// Both axes separately, because a clip strict on one axis alone passes
    /// the other axis's cases. The corner pair is the diagonal case a
    /// single-axis fixture cannot express.
    #[test]
    fn a_shared_edge_and_a_touched_corner_hold_no_area_and_one_unit_in_does() {
        let left = quad((0, 0), (100, 100));

        // Along x: edge against edge, one unit of overlap, one unit of gap.
        assert_eq!(twice_shared(&left, &quad((100, 0), (200, 100))), Some(0));
        assert_eq!(
            twice_shared(&left, &quad((99, 0), (200, 100))),
            Some(2 * 100),
            "one unit along x is a shared area"
        );
        assert_eq!(twice_shared(&left, &quad((101, 0), (200, 100))), Some(0));

        // The same three along y.
        assert_eq!(twice_shared(&left, &quad((0, 100), (100, 200))), Some(0));
        assert_eq!(
            twice_shared(&left, &quad((0, 99), (100, 200))),
            Some(2 * 100),
            "one unit along y is a shared area"
        );
        assert_eq!(twice_shared(&left, &quad((0, 101), (100, 200))), Some(0));

        // The corner: touched, then one unit in on both axes at once.
        assert_eq!(twice_shared(&left, &quad((100, 100), (200, 200))), Some(0));
        assert_eq!(
            twice_shared(&left, &quad((99, 99), (200, 200))),
            Some(2),
            "one square unit at the corner"
        );

        // None of those reports: a hundred square units of a ten-thousand
        // square unit box is one percent, and the corner unit is less.
        let whole = twice_area(&left).abs();
        assert_eq!(whole, 2 * 100 * 100);
        assert!(!exceeds_ratio(2 * 100, whole));
        assert!(!exceeds_ratio(2, whole));
        // While half of it does, so the pairing above is not vacuous.
        assert!(exceeds_ratio(whole / 2, whole));
    }

    /// A box of no area shares none, where the extent lookalike says it does.
    ///
    /// The lookalike's four comparisons all pass for a degenerate box strictly
    /// inside another, and there is no area to share. **The clip is not
    /// symmetric about the degenerate case**: a box of no area clipped by a
    /// real one keeps no area, while a real box clipped by a *point* is
    /// unchanged, because every inside test against an edge of no length
    /// answers zero and keeps everything. That is why [`super::add`] drops a
    /// box of no area before the pair walk rather than leaving the region test
    /// to do it.
    #[test]
    fn a_box_of_no_area_shares_none_where_the_extent_lookalike_says_it_does() {
        let left = quad((0, 0), (100, 100));
        let point = quad((50, 50), (50, 50));
        let upright = quad((50, 10), (50, 90));
        let flat = quad((10, 50), (90, 50));

        for degenerate in [point, upright, flat] {
            assert!(
                extents_overlap(&left, &degenerate),
                "the lookalike says these overlap: {degenerate:?}"
            );
            assert_eq!(twice_area(&degenerate), 0, "and it has no area to share");
            assert_eq!(
                twice_shared(&degenerate, &left),
                Some(0),
                "clipped by a real box it keeps none: {degenerate:?}"
            );
        }

        // The asymmetry, measured rather than assumed. A point's four edges
        // all have no length, so the clip is a no-op; a line's two real edges
        // are opposite, so the clip collapses the subject onto the line.
        assert_eq!(
            twice_shared(&left, &point),
            Some(2 * 100 * 100),
            "a clip by a point keeps the whole subject"
        );
        assert_eq!(twice_shared(&left, &upright), Some(0));
        assert_eq!(twice_shared(&left, &flat), Some(0));

        // Where both boxes have an area the two answers agree, so the
        // disagreement above is about the degenerate case and not the rule.
        let half = quad((50, 50), (150, 150));
        assert!(extents_overlap(&left, &half));
        assert_eq!(twice_shared(&left, &half), Some(2 * 50 * 50));
        let apart = quad((200, 200), (300, 300));
        assert!(!extents_overlap(&left, &apart));
        assert_eq!(twice_shared(&left, &apart), Some(0));
    }

    /// A relative angle that is not a right angle is declined, not rounded.
    ///
    /// Two convex quadrilaterals can meet at a point whose coordinates are
    /// fractions of an internal unit, and [`shared_region`] reports that
    /// rather than rounding it into a blocking verdict. The extent lookalike
    /// has no such scruple: it answers from boxes that are not the text's
    /// boxes and returns a definite verdict where the rule declines to give
    /// one.
    ///
    /// The second half of this check is a **sweep over the angles**, and it is
    /// what corrected the module header: the fraction is a property of where
    /// two edges cross, not of the relative angle being off a right angle.
    /// The eighth turns are exact; the twelfth turns are not.
    #[test]
    fn a_relative_angle_that_is_not_a_right_angle_is_declined_rather_than_rounded() {
        // Hand built, so that the fraction is arithmetic rather than luck.
        // The window's first edge runs at one in two, and the subject's right
        // side at x = 21 crosses it at y = 10.5.
        let subject = quad((0, 0), (21, 20));
        let window = [
            Point::new(0, 0),
            Point::new(10, 5),
            Point::new(5, 15),
            Point::new(-5, 10),
        ];
        assert!(twice_area(&window) > 0, "the window is convex and wound");
        assert_eq!(shared_region(&subject, &window), None, "the rule declines");

        // The lookalike answers, and it answers over the threshold: the two
        // extents share 150 square units of a window of 125.
        assert!(extents_overlap(&subject, &window));
        let shared_extent = 2 * 10 * 15;
        assert!(
            exceeds_ratio(shared_extent, twice_area(&window)),
            "the lookalike reports where the rule declines"
        );

        // And now the same question asked of real text boxes, swept over the
        // angles. **MEASURED, and it corrects what this rule's own module
        // header used to claim**: the decline is about whether a crossing
        // lands on a whole internal unit and NOT about the relative angle
        // being a right angle. A box turned 45 degrees has edges of slope
        // exactly one after the rotation rounds, so it crosses an
        // axis-aligned edge on a whole unit and the shared region is exact.
        let style = TextStyle::default();
        let anchor = Point::new(1_000_000, 1_000_000);
        let flat = text_box("Ay", anchor, Angle(0), &style);
        let mut exact = Vec::new();
        let mut declined = Vec::new();
        for angle in (0..180).step_by(15) {
            let turned = text_box("Ay", anchor, Angle(angle), &style);
            // Every box on this sweep sits on the flat one, so none of the
            // answers below is an answer about two boxes that miss.
            assert!(
                extents_overlap(&flat.corners(), &turned.corners()),
                "at {angle} degrees the two still sit on each other"
            );
            if shared_region(&flat.corners(), &turned.corners()).is_some() {
                exact.push(angle);
            } else {
                declined.push(angle);
            }
        }
        assert_eq!(
            exact,
            vec![0, 45, 90, 135],
            "the eighth turns cross on whole units"
        );
        assert_eq!(
            declined,
            vec![15, 30, 60, 75, 105, 120, 150, 165],
            "and every other angle on the sweep is declined"
        );
    }

    /// A window wound the other way is normalised, not inverted.
    ///
    /// The clip decides inside from the window's winding, so a quadrilateral
    /// arriving wound the other way would have every inside test inverted.
    /// The control is the un-normalised clip run by hand: it answers nothing
    /// where the two boxes cover a quarter of each other.
    #[test]
    fn a_window_wound_the_other_way_is_normalised_rather_than_inverted() {
        let left = quad((0, 0), (100, 100));
        let right = quad((50, 50), (150, 150));
        let [first, second, third, fourth] = right;
        let backwards = [fourth, third, second, first];

        // The control: the two windings really are different inputs.
        assert!(twice_area(&right) > 0);
        assert!(twice_area(&backwards) < 0);
        assert_eq!(positively_wound(backwards), right);
        assert_eq!(positively_wound(right), right, "and the right way is kept");

        // The answer does not depend on which way the window arrived.
        let quarter = Some(2 * 50 * 50);
        assert_eq!(twice_shared(&left, &right), quarter);
        assert_eq!(twice_shared(&left, &backwards), quarter);

        // Without the normalisation, every inside test is inverted and the
        // clip answers nothing at all on the same two boxes.
        let mut inverted = left.to_vec();
        for place in 0..backwards.len() {
            inverted = clipped(
                &inverted,
                backwards[place],
                backwards[(place + 1) % backwards.len()],
            )
            .expect("every corner of this pair is a whole unit");
        }
        assert_ne!(
            Some(twice_area(&inverted).abs()),
            quarter,
            "an un-normalised window does not answer the intersection"
        );
        assert_eq!(twice_area(&inverted).abs(), 0);
    }
}
