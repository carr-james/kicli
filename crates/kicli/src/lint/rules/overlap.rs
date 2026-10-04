//! `KI-OVL-001` — two symbol bodies overlap.
//!
//! Two bodies that share even one internal unit of area draw their graphics and
//! their pins on top of each other, so at the point of overlap at least one of
//! the two is not there to be read. That is the loss of the marks the drawing
//! is made of, not a judgement about how tightly a sheet is packed.
//!
//! # The tier, argued from the north star
//!
//! The rule rests on **no published source**, and this file says so rather than
//! borrowing one. `lane-t5` measured the catalogue's nearest candidate — Graham
//! Sutherland §10, *"Don't try to squash things into the smallest possible
//! space"* — and recorded it as **not this rule**; it is about density, and
//! density is legible. So the tier is argued from `tasks/M5/RULES.md`'s north
//! star, which is the sentence `RULES.md` says a tier is argued from:
//!
//! > The tool must validate the important aspects of quality schematics. It
//! > must never reward a schematic that is impossible to read and understand.
//!
//! 1. A pair of overlapping bodies hides drawing from the reader, so a sheet
//!    holding one cannot be read as drawn — the second sentence's subject
//!    exactly.
//! 2. Tier 2 **cannot** carry that claim: a normalised rule's cost is capped at
//!    `w · reference` however often it fires, which is the measurement BLOCKED
//!    3 recorded — a sheet where every wire crosses another **scores 67**, at
//!    ten wires and at ten thousand. A sheet whose every symbol sat on another
//!    would score in the same band, and a score in that band is a reward.
//! 3. Tier 1 is the only tier whose mechanism matches the claim, because one
//!    occurrence fails the gate and the verdict does not depend on how much of
//!    the sheet was lost; and the detection is **exact** — integer box
//!    intersection with no tolerance anywhere — so the gate fires only where
//!    drawing really is hidden and never on a sheet that is merely tight.
//!
//! The third sentence is the one that earns the tier. A blocking rule with a
//! tolerance would be unarguable from that sentence, which is why the published
//! catalogue's `--allow` list is a list and not a threshold.
//!
//! # Which box, and why it is the other one from `KI-TXT-001`
//!
//! The **body box**: graphics and pins, no text. `spec/SPEC.md` §8 assigns the
//! two boxes by rule family in one sentence — *"**body box** (graphics + pins,
//! no text) for overlap rules, and **full box** (∪ visible field boxes) for
//! text-collision rules"* — so this rule reads [`crate::geometry::SymbolBoxes::body`]
//! and `KI-TXT-001` reads `full`. Crossing them would make both rules wrong
//! plausibly: this rule would fire on two symbols whose *reference designators*
//! sit near each other while nothing drawn overlaps at all.
//!
//! The box comes from [`crate::geometry::symbol_boxes`], which already applies
//! the placement's orientation — `Transform::from_file(angle, mirror)`, with
//! the mirror composed second. Nothing about rotation is re-derived here. A
//! rule that measured an unrotated box would report a symbol at 90 degrees as
//! overlapping things it does not touch, and the two rotated pairs in
//! `tests/lint_symbol_overlap.rs` are what hold that down.
//!
//! # Power symbols are included
//!
//! The catalogue says so: *"Power symbols included."* They cluster, so they are
//! the obvious thing to exempt, and exempting them would be wrong — two ground
//! symbols drawn on top of each other are two net names a reader cannot
//! separate. Nothing in this file tests [`crate::model::items::Symbol::is_power`],
//! and `power_symbols_are_not_exempt` is the check that keeps it that way. Note
//! the asymmetry it measures: the scorer's own
//! [`crate::lint::score::Density::symbols`] *excludes* power symbols, so a
//! sheet of nothing but overlapping power symbols has a symbol count of zero
//! and a finding all the same.
//!
//! # Where this under-reports, and never over-reports
//!
//! A symbol the file embeds no definition for draws nothing, so it has no body
//! and is skipped — the same omission [`crate::lint::drawing::Drawing::definition_of`]
//! makes everywhere else. A symbol that draws no graphics and no visible pin
//! has a box of no width, which shares no area with anything by the same two
//! comparisons that exclude a shared edge. Both are missed findings rather than
//! false ones, which is the direction a blocking rule must err in.

use crate::geometry::{Point, Rect, symbol_boxes};
use crate::lint::gate::Saturation;
use crate::lint::{Drawing, Findings, Rule, RuleId, Tier};
use crate::model::items::{Item, Uuid};

/// What separates the two names of an `--allow` entry.
///
/// A reference designator cannot hold a colon, so the separator cannot be part
/// of either name.
#[allow(
    dead_code,
    reason = "the --allow list has no caller: Rule::examine takes only a Drawing and \
              Engine::of takes &'static dyn Rule, so no rule can hold a value a command \
              line chose. Reported as a seam finding rather than deleted; the checks in \
              this file's tests module are the only callers until the wiring exists."
)]
const SEPARATOR: char = ':';

/// One placed symbol, reduced to what this rule compares.
struct Body {
    /// The reference designator on this sheet, or the short uuid when the
    /// placement records none. An unnamed symbol is still compared; it just
    /// cannot be written in an `--allow` entry.
    name: String,
    /// What a finding names.
    object: Uuid,
    /// The body box, in schematic coordinates.
    body: Rect,
}

/// One pair of symbols an author has said overlap on purpose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllowedPair {
    /// One of the two reference designators.
    pub one: String,
    /// The other.
    pub two: String,
}

/// The pairs an `--allow` list exempts from this rule.
///
/// **This is a list, not a tolerance, and that was decided before this file
/// was written.** `research/style-rules.md` §4: *"deliberately overlapping
/// decorative symbols are rare enough to justify an explicit `--allow` list
/// rather than a soft rule."* The reason is the tier argument in this module's
/// header — a tolerance inside a blocking gate cannot be argued from the north
/// star, because the drawing a tolerance passes is still a drawing a reader
/// cannot read. A named pair is different in kind: it is an author saying *this
/// one is on purpose*, and it names which one.
///
/// **This type is not reachable from outside the crate**, because the
/// registration seam puts a rule file behind a private `mod` the build script
/// writes. The `--allow` wiring that will need it is the seam report on
/// [`SymbolBodiesOverlap::allowing`]. The checks live in this file's own
/// `tests` module for the same reason: a doc example here could not name it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Allowed(Vec<AllowedPair>);

impl Allowed {
    /// An empty list: no pair is exempt.
    pub const NONE: Self = Self(Vec::new());

    /// Read an `--allow` list in the form the command line writes it, `R1:R2`.
    ///
    /// # Errors
    ///
    /// Returns the first entry that is not two non-empty names separated by
    /// exactly one colon. An unreadable entry is refused rather than skipped:
    /// an exemption an author believes they wrote and did not is the one
    /// failure mode of an allow list that a blocking rule cannot afford.
    #[allow(
        dead_code,
        reason = "the --allow list has no caller: Rule::examine takes only a Drawing and \
                  Engine::of takes &'static dyn Rule, so no rule can hold a value a command \
                  line chose. Reported as a seam finding rather than deleted; the checks in \
                  this file's tests module are the only callers until the wiring exists."
    )]
    pub fn read<'a>(entries: impl IntoIterator<Item = &'a str>) -> Result<Self, &'a str> {
        let mut pairs = Vec::new();
        for entry in entries {
            let Some((one, two)) = entry.split_once(SEPARATOR) else {
                return Err(entry);
            };
            if one.is_empty() || two.is_empty() || two.contains(SEPARATOR) {
                return Err(entry);
            }
            pairs.push(AllowedPair {
                one: one.to_owned(),
                two: two.to_owned(),
            });
        }
        Ok(Self(pairs))
    }

    /// Is this pair of reference designators exempt?
    ///
    /// The two are unordered, because an author writing `R1:R2` has said
    /// nothing about which of the two the file holds first.
    #[must_use]
    pub fn permits(&self, one: &str, two: &str) -> bool {
        self.0.iter().any(|pair| {
            (pair.one == one && pair.two == two) || (pair.one == two && pair.two == one)
        })
    }

    /// The pairs the list holds, in the order they were read.
    #[must_use]
    #[allow(
        dead_code,
        reason = "the --allow list has no caller: Rule::examine takes only a Drawing and \
                  Engine::of takes &'static dyn Rule, so no rule can hold a value a command \
                  line chose. Reported as a seam finding rather than deleted; the checks in \
                  this file's tests module are the only callers until the wiring exists."
    )]
    pub fn pairs(&self) -> &[AllowedPair] {
        &self.0
    }

    /// Does the list exempt nothing?
    #[must_use]
    #[allow(
        dead_code,
        reason = "the --allow list has no caller: Rule::examine takes only a Drawing and \
                  Engine::of takes &'static dyn Rule, so no rule can hold a value a command \
                  line chose. Reported as a seam finding rather than deleted; the checks in \
                  this file's tests module are the only callers until the wiring exists."
    )]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Two symbol bodies overlap.
pub struct SymbolBodiesOverlap {
    /// The pairs this instance exempts.
    allowed: Allowed,
}

impl SymbolBodiesOverlap {
    /// The rule with no pair exempt, which is what the registry holds.
    #[must_use]
    pub const fn strict() -> Self {
        Self {
            allowed: Allowed::NONE,
        }
    }

    /// The rule with an `--allow` list.
    ///
    /// **Nothing in the crate calls this yet, and that is a seam report rather
    /// than an omission.** [`Rule::examine`] receives a
    /// [`crate::lint::drawing::Drawing`] and nothing else, and
    /// [`crate::lint::Engine::of`] takes `Vec<&'static dyn Rule>` — so no rule
    /// can carry a value a command line chose, and the lint engine has no
    /// command surface to carry one from. The list's *mechanism* is here and
    /// measured; the wiring is one edit in files this rule may not touch. See
    /// `tasks/M5/phase2-ki-ovl-001-symbol-overlap.md`.
    #[must_use]
    #[allow(
        dead_code,
        reason = "the --allow list has no caller: Rule::examine takes only a Drawing and \
                  Engine::of takes &'static dyn Rule, so no rule can hold a value a command \
                  line chose. Reported as a seam finding rather than deleted; the checks in \
                  this file's tests module are the only callers until the wiring exists."
    )]
    pub fn allowing(allowed: Allowed) -> Self {
        Self { allowed }
    }

    /// The pairs this instance exempts.
    #[must_use]
    #[allow(
        dead_code,
        reason = "the --allow list has no caller: Rule::examine takes only a Drawing and \
                  Engine::of takes &'static dyn Rule, so no rule can hold a value a command \
                  line chose. Reported as a seam finding rather than deleted; the checks in \
                  this file's tests module are the only callers until the wiring exists."
    )]
    pub fn allowed(&self) -> &Allowed {
        &self.allowed
    }
}

impl Rule for SymbolBodiesOverlap {
    fn id(&self) -> RuleId {
        RuleId("KI-OVL-001")
    }

    fn tier(&self) -> Tier {
        Tier::One
    }

    /// This rule never saturates, and the declaration is a report rather than
    /// a shrug.
    ///
    /// Three reasons, in the order they bite.
    ///
    /// **It cannot change an outcome.** [`crate::lint::gate::Gate::of`] answers
    /// a Tier 1 rule with `Blocker::Blocking` before it ever asks what share
    /// the rule covered, so one overlapping pair and every pair on the sheet
    /// are the same verdict. `tasks/M5/RULES.md` records this for the whole
    /// tier; `KI-CONN-001` reached it first; this file reaches it again.
    ///
    /// **This rule counts pairs, and no denominator counts pairs.**
    /// [`crate::lint::gate::Counted`] offers symbols, wires or nothing. A
    /// finding here is one *pair*, and a sheet of `n` symbols holds
    /// `n(n-1)/2` of them, so a pair count measured against a symbol count is
    /// a ratio of two different things.
    ///
    /// **And the symbol denominator excludes objects this rule counts.**
    /// [`crate::lint::score::Density::symbols`] is documented as the
    /// **non-power** symbols. This rule includes power symbols, so
    /// `Counted::Symbols` would compare findings about power symbols against a
    /// total that leaves them out — and on a sheet of nothing but power
    /// symbols that total is zero, which
    /// [`crate::lint::gate::Saturation::is_reached`] reads as *never
    /// saturates* anyway. Declaring it would be a false declaration that
    /// happens to be inert, which is worse than declaring nothing.
    fn saturation(&self) -> Saturation {
        Saturation::NEVER
    }

    fn examine(&self, drawing: &Drawing<'_>, found: &mut Findings<'_>) {
        let bodies = bodies_of(drawing);
        for (place, one) in bodies.iter().enumerate() {
            for two in bodies.iter().skip(place + 1) {
                let Some(shared) = shared_area(one.body, two.body) else {
                    continue;
                };
                if self.allowed.permits(&one.name, &two.name) {
                    continue;
                }
                found.record(
                    shared.centre(),
                    vec![one.object.clone(), two.object.clone()],
                    format!(
                        "symbol bodies {} and {} overlap, {} by {} mm",
                        one.name,
                        two.name,
                        shared.width(),
                        shared.height()
                    ),
                );
            }
        }
    }
}

/// Every symbol of this placement that draws a body, in file order.
///
/// File order makes the pair walk deterministic and makes the first-named
/// symbol of a finding the one the file holds first.
///
/// **Power symbols are not filtered here, and that is the rule.** Grep this
/// function for `is_power` and find nothing.
fn bodies_of(drawing: &Drawing<'_>) -> Vec<Body> {
    let sheet = drawing.path();
    let mut bodies = Vec::new();
    for item in &drawing.schematic().items {
        let Item::Symbol(symbol) = item else {
            continue;
        };
        let Some(definition) = drawing.definition_of(symbol) else {
            continue;
        };
        // The instance record says which unit this placement draws, and a
        // multi-unit part's units have different bodies.
        let drawn = symbol.drawn_on(sheet);
        bodies.push(Body {
            name: symbol.reference_on(sheet).map_or_else(
                || symbol.uuid.short().to_owned(),
                |reference| reference.0.clone(),
            ),
            object: symbol.uuid.clone(),
            body: symbol_boxes(drawing.doc(), &drawn, definition).body,
        });
    }
    bodies
}

/// The region two body boxes share, when they share an **area**.
///
/// **The test is on the shared region, and it is strict on both axes. That is
/// the whole rule.** The published form is *"exact box intersection, ≥ 1 IU"*,
/// and every coordinate in a schematic is a whole number of internal units —
/// so the smallest region two boxes can share is one unit on each axis, and
/// strict `<` admits exactly that while refusing a pair that shares only an
/// edge. One `<=` here would make every symbol placed edge to edge a blocking
/// finding on a correct drawing; it is the single most likely defect in this
/// file, and it is invisible to a fixture built by eye, because a one-unit gap
/// and a one-unit overlap look identical on screen.
///
/// Both axes are tested for the same reason, separately. A pair touching at a
/// corner shares no area either, and a rule strict on one axis only would
/// report it.
///
/// # Why the test is on the region and not on the two boxes
///
/// The obvious form — four comparisons between the two boxes’ own corners — is
/// the plausible lookalike, and it was written here first. It agrees on every
/// pair of boxes that both have a size, and it **disagrees on a box that has
/// none**: a symbol drawing no graphics and no visible pin gets a box of no
/// size at its anchor ([`crate::geometry::symbol_boxes`] falls back to
/// `Rect::around`), and a box of no size sitting *strictly inside* another
/// box passes all four comparisons while sharing an area of nothing. It would
/// have been reported as a blocking overlap of `0 by 0 mm`.
///
/// It was caught by `a_box_of_no_size_shares_nothing_even_with_itself` before
/// any deliberate break was made, and the two forms are not two spellings of
/// one rule: firing on a shared **point** while refusing a shared **edge**
/// would be incoherent, because the edge is the larger set of the two. Testing
/// the region states “≥ 1 IU” once, for every input, and the degenerate cases
/// fall out of it rather than needing a case of their own.
fn shared_area(one: Rect, two: Rect) -> Option<Rect> {
    let (one_start, one_end) = (one.start(), one.end());
    let (two_start, two_end) = (two.start(), two.end());
    let start = Point {
        x: one_start.x.max(two_start.x),
        y: one_start.y.max(two_start.y),
    };
    let end = Point {
        x: one_end.x.min(two_end.x),
        y: one_end.y.min(two_end.y),
    };
    // Before `Rect::new`, which normalises its corners and would hide an
    // inverted region as a valid one.
    if start.x >= end.x || start.y >= end.y {
        return None;
    }
    Some(Rect::new(start, end))
}

/// The registered rule, with no pair exempt.
pub static OVERLAP: SymbolBodiesOverlap = SymbolBodiesOverlap::strict();

/// The rules this file declares.
pub static RULES: &[&'static dyn Rule] = &[&OVERLAP];

#[cfg(test)]
mod tests {
    use super::{Allowed, shared_area};
    use crate::geometry::{Point, Rect};

    /// A box from one corner to another, in internal units.
    fn boxed(start: (i32, i32), end: (i32, i32)) -> Rect {
        Rect::new(Point::new(start.0, start.1), Point::new(end.0, end.1))
    }

    /// The boundary, from both sides, on each axis separately.
    ///
    /// The integration checks in `tests/lint_symbol_overlap.rs` measure the
    /// same boundary through a written file; this measures it where a `<=`
    /// is one character away, and it measures the two axes independently,
    /// which a square fixture cannot.
    #[test]
    fn a_shared_edge_is_not_a_shared_area_and_one_unit_in_is() {
        let left = boxed((0, 0), (100, 100));

        // Touching along x: the right edge of one is the left edge of the
        // other.
        assert!(shared_area(left, boxed((100, 0), (200, 100))).is_none());
        // One unit of overlap along x, and one unit of gap.
        assert_eq!(
            shared_area(left, boxed((99, 0), (200, 100))),
            Some(boxed((99, 0), (100, 100))),
            "one unit along x is a shared area"
        );
        assert!(shared_area(left, boxed((101, 0), (200, 100))).is_none());

        // The same three, along y. A rule strict on x alone passes the three
        // above and fails these.
        assert!(shared_area(left, boxed((0, 100), (100, 200))).is_none());
        assert_eq!(
            shared_area(left, boxed((0, 99), (100, 200))),
            Some(boxed((0, 99), (100, 100))),
            "one unit along y is a shared area"
        );
        assert!(shared_area(left, boxed((0, 101), (100, 200))).is_none());
    }

    #[test]
    fn a_corner_touch_shares_nothing() {
        let left = boxed((0, 0), (100, 100));
        assert!(shared_area(left, boxed((100, 100), (200, 200))).is_none());
        // And one unit in on both axes does share.
        assert_eq!(
            shared_area(left, boxed((99, 99), (200, 200))),
            Some(boxed((99, 99), (100, 100)))
        );
    }

    #[test]
    fn a_box_of_no_size_shares_nothing_even_with_itself() {
        let nothing = boxed((50, 50), (50, 50));
        assert!(shared_area(nothing, nothing).is_none());
        assert!(shared_area(nothing, boxed((0, 0), (100, 100))).is_none());

        // A box with one zero side is the same answer, on either axis.
        let flat = boxed((0, 50), (100, 50));
        assert!(shared_area(flat, boxed((0, 0), (100, 100))).is_none());
        let thin = boxed((50, 0), (50, 100));
        assert!(shared_area(thin, boxed((0, 0), (100, 100))).is_none());
    }

    /// The lookalike this file was written with first, kept as a control.
    ///
    /// Four comparisons between the two boxes' own corners. It is not a
    /// paraphrase of [`shared_area`] and the next check is what shows it.
    fn box_to_box_lookalike(one: Rect, two: Rect) -> bool {
        let (one_start, one_end) = (one.start(), one.end());
        let (two_start, two_end) = (two.start(), two.end());
        one_start.x < two_end.x
            && two_start.x < one_end.x
            && one_start.y < two_end.y
            && two_start.y < one_end.y
    }

    /// The two forms agree wherever both boxes have a size, and disagree on a
    /// box that has none. That disagreement is the defect the region test
    /// fixes, measured rather than asserted in prose.
    #[test]
    fn the_lookalike_disagrees_exactly_where_a_box_has_no_size() {
        let outer = boxed((0, 0), (100, 100));

        // Agreement, across the boundary and on both sides of it.
        for other in [
            boxed((99, 0), (200, 100)),
            boxed((100, 0), (200, 100)),
            boxed((101, 0), (200, 100)),
            boxed((0, 99), (100, 200)),
            boxed((0, 100), (100, 200)),
            boxed((100, 100), (200, 200)),
            boxed((10, 10), (20, 20)),
            boxed((200, 200), (300, 300)),
        ] {
            assert_eq!(
                shared_area(outer, other).is_some(),
                box_to_box_lookalike(outer, other),
                "the two forms agree on {other}"
            );
        }

        // Disagreement: a box of no size strictly inside another. The
        // lookalike calls it an overlap of no area at all.
        let point = boxed((50, 50), (50, 50));
        assert!(box_to_box_lookalike(outer, point), "the lookalike fires");
        assert!(
            shared_area(outer, point).is_none(),
            "and this rule does not"
        );

        // And the same on one axis only: a box with no height.
        let flat = boxed((10, 50), (90, 50));
        assert!(box_to_box_lookalike(outer, flat));
        assert!(shared_area(outer, flat).is_none());
    }

    #[test]
    fn a_box_inside_another_shares_its_whole_self() {
        let outer = boxed((0, 0), (100, 100));
        let inner = boxed((10, 10), (20, 20));
        assert_eq!(shared_area(outer, inner), Some(inner));
        assert_eq!(
            shared_area(inner, outer),
            Some(inner),
            "and the answer does not depend on the order"
        );
    }

    #[test]
    fn an_allow_entry_is_two_names_and_one_colon() {
        let allowed = Allowed::read(["R1:R2", "#PWR01:#PWR02"]).expect("both entries read");
        assert_eq!(allowed.pairs().len(), 2);
        assert!(allowed.permits("R1", "R2"));
        assert!(allowed.permits("R2", "R1"), "the pair is unordered");
        assert!(allowed.permits("#PWR02", "#PWR01"), "power names read too");

        // A name not in the list is not exempt, and neither is a pair made of
        // one name from each entry.
        assert!(!allowed.permits("R1", "R3"));
        assert!(!allowed.permits("R1", "#PWR01"));

        // The empty list permits nothing at all.
        assert!(Allowed::NONE.is_empty());
        assert!(!Allowed::NONE.permits("R1", "R2"));
        assert_eq!(Allowed::read([]).expect("nothing reads"), Allowed::NONE);
    }

    #[test]
    fn an_unreadable_allow_entry_is_refused_rather_than_skipped() {
        for refused in ["R1", "", ":R2", "R1:", "R1:R2:R3", ":"] {
            assert_eq!(
                Allowed::read(["R9:R8", refused]).err(),
                Some(refused),
                "refused: {refused:?}"
            );
        }
        // And the good entry beside it does not make the list readable, so a
        // caller cannot get a half-read exemption.
        assert!(Allowed::read(["R9:R8", "R1"]).is_err());
    }
}
