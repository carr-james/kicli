//! `KI-TXT-001` reports two pieces of visible text that overlap, and nothing
//! else.
//!
//! # Where the expectations come from
//!
//! Each fixture here is a committed schematic whose bytes `kicad-cli sch
//! upgrade` wrote, so the drawing is one KiCad accepts and lays out. Every
//! expectation is then derived **in this file's own arithmetic**, from the
//! string, the anchor and the angle the fixture records, and never from the
//! rule's own comparison. The two derivations share only
//! [`kicli::geometry::text::text_box`], which `tests/text_metrics.rs` measures
//! against KiCad's own font engine.
//!
//! This file's arithmetic is axis-aligned rectangle intersection, which is
//! valid here and only here: every string in every fixture sits at a multiple
//! of 90 degrees, and a box turned by a right angle is axis-aligned on the
//! page. That is asserted rather than assumed — see
//! [`a_right_angled_box_is_axis_aligned_on_the_page`].
//!
//! # What is checked where
//!
//! The **ratio boundary** is exercised twice, at two grains, because neither
//! grain can do the other's job.
//!
//! - Here, through two real drawings that differ by **one internal unit** of
//!   overlap: `one_pair.kicad_sch` holds a pair covering 19.997 % of the
//!   smaller box and `boundary.kicad_sch` holds one covering 20.002 %. One
//!   fires and the other does not. One internal unit is the smallest change a
//!   KiCad coordinate can record, so this is the finest boundary a drawing can
//!   express.
//! - In the rule's own `tests` module, on the comparison itself, at exactly
//!   20 % and at 20 % plus one square internal unit. **Exactly 20 % is not
//!   reachable through a drawing at all** for these boxes: the threshold is
//!   `20 · area / 100`, and the smaller box's area is not a multiple of five,
//!   so no arrangement of it lands on the line. The comparison has to be asked
//!   directly.

use kicli::geometry::text::{TextStyle, text_box};
use kicli::geometry::{Angle, Point, Rect};
use kicli::lint::score::Density;
use kicli::lint::{Drawing, Engine, Finding, Gate, RuleId};
use kicli::model::items::Item;
use kicli::model::{Hierarchy, Schematic};
use kicli_sexpr::Doc;
use std::path::{Path, PathBuf};

/// The rule under test.
const RULE: RuleId = RuleId("KI-TXT-001");

/// The published share of the smaller box that must be covered.
const RATIO: (i64, i64) = (20, 100);

/// The committed fixture this binary reads.
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sch/text_overlap")
        .join(name)
}

/// Every `KI-TXT-001` finding of one fixture, in report order.
fn findings_of(name: &str) -> Vec<Finding> {
    let root = fixture(name);
    let hierarchy = Hierarchy::load(&root).expect("the hierarchy loads");
    let drawings: Vec<Drawing<'_>> = hierarchy
        .placements
        .iter()
        .map(|placement| {
            let file = &hierarchy.files[placement.file];
            Drawing::read(&file.doc, &file.schematic, &placement.path)
        })
        .collect();
    Engine::of_every_rule()
        .examine_all(&drawings)
        .into_iter()
        .filter(|finding| finding.rule == RULE)
        .collect()
}

/// One piece of text a fixture draws, as this file reads it off the file.
struct Written {
    /// The handle a finding names it by.
    handle: String,
    /// The string, as written.
    text: String,
    /// The anchor the file records.
    at: Point,
    /// The string's own angle.
    angle: Angle,
    /// The text effects, read from the same list the rule reads.
    style: TextStyle,
    /// Is the text drawn at all?
    drawn: bool,
}

impl Written {
    /// The page-space box, turned by the string's own angle.
    ///
    /// Axis-aligned, which is correct for a right angle and for nothing else.
    fn turned(&self) -> Rect {
        let boxed = text_box(&self.text, self.at, self.angle, &self.style);
        let corners = boxed.corners();
        corners
            .iter()
            .skip(1)
            .fold(Rect::around(corners[0]), |so_far, &corner| {
                so_far.union(Rect::around(corner))
            })
    }

    /// The same box left **unturned**: the lookalike this rule must not be.
    fn unturned(&self) -> Rect {
        text_box(&self.text, self.at, self.angle, &self.style).bounds()
    }
}

/// Every label, free text and visible symbol field a fixture draws.
///
/// Read from the typed model rather than from the rule, so that what this file
/// believes the drawing holds is derived separately from what the rule does.
fn written(name: &str) -> Vec<Written> {
    let source = std::fs::read_to_string(fixture(name)).expect("the fixture reads");
    let doc = Doc::parse(&source).expect("the fixture parses");
    let schematic = Schematic::read(&doc).expect("the fixture reads as a schematic");
    let mut found = Vec::new();
    for item in &schematic.items {
        match item {
            Item::Label(label) => found.push(Written {
                handle: label.uuid.short().to_owned(),
                text: label.text.clone(),
                at: label.at,
                angle: label.angle,
                style: TextStyle::read(&doc, label.node),
                drawn: true,
            }),
            Item::Text(text) => found.push(Written {
                handle: text.uuid.short().to_owned(),
                text: text.text.clone(),
                at: text.at,
                angle: text.angle,
                style: TextStyle::read(&doc, text.node),
                drawn: true,
            }),
            Item::Symbol(symbol) => {
                for field in &symbol.fields {
                    found.push(Written {
                        handle: symbol.uuid.short().to_owned(),
                        text: field.value.clone(),
                        at: field.at,
                        angle: field.angle,
                        style: TextStyle::read(&doc, field.node),
                        drawn: !field.hidden,
                    });
                }
            }
            _ => {}
        }
    }
    found
}

/// The area two axis-aligned boxes share, in this file's own arithmetic.
fn shared_area(one: Rect, two: Rect) -> i64 {
    let across = i64::from(one.end().x.0.min(two.end().x.0))
        - i64::from(one.start().x.0.max(two.start().x.0));
    let down = i64::from(one.end().y.0.min(two.end().y.0))
        - i64::from(one.start().y.0.max(two.start().y.0));
    if across <= 0 || down <= 0 {
        return 0;
    }
    across * down
}

/// The area of a box.
fn area(rect: Rect) -> i64 {
    i64::from(rect.width().0) * i64::from(rect.height().0)
}

/// Does this pair pass the published ratio, in this file's own arithmetic?
fn over_the_ratio(shared: i64, smaller: i64) -> bool {
    shared * RATIO.1 > RATIO.0 * smaller
}

#[test]
fn a_right_angled_box_is_axis_aligned_on_the_page() {
    // The licence for every rectangle comparison in this file, and the reason
    // the divergence check below is built against the UNTURNED box rather than
    // against an axis-aligned one. At a right angle the two are the same box,
    // so an axis-aligned approximation loses nothing there; the box that is
    // wrong is the one that was never turned.
    let style = TextStyle::default();
    let anchor = Point::new(1_000_000, 1_000_000);
    for angle in [0, 90, 180, 270] {
        let boxed = text_box("Ay", anchor, Angle(angle), &style);
        let corners = boxed.corners();
        let xs: Vec<i32> = corners.iter().map(|corner| corner.x.0).collect();
        let ys: Vec<i32> = corners.iter().map(|corner| corner.y.0).collect();
        // Two distinct x values and two distinct y values is what makes a
        // quadrilateral axis-aligned.
        let mut distinct_x = xs.clone();
        distinct_x.sort_unstable();
        distinct_x.dedup();
        let mut distinct_y = ys.clone();
        distinct_y.sort_unstable();
        distinct_y.dedup();
        assert_eq!(distinct_x.len(), 2, "at {angle} degrees: {corners:?}");
        assert_eq!(distinct_y.len(), 2, "at {angle} degrees: {corners:?}");
        assert_eq!(
            boxed.axis_aligned(),
            Rect::new(
                Point::new(distinct_x[0], distinct_y[0]),
                Point::new(distinct_x[1], distinct_y[1])
            ),
            "at {angle} degrees the page box is the turned box"
        );
    }

    // And off the right angles it is not, so the claim above is about right
    // angles rather than about every box.
    let slanted = text_box("Ay", anchor, Angle(45), &style);
    let corners = slanted.corners();
    let mut xs: Vec<i32> = corners.iter().map(|corner| corner.x.0).collect();
    xs.sort_unstable();
    xs.dedup();
    assert!(xs.len() > 2, "a slanted box is not axis-aligned: {corners:?}");
}

#[test]
fn a_turned_label_is_not_compared_where_its_unturned_box_would_be() {
    // The check this task turns on. Two labels of the same string: one flat,
    // one at 90 degrees, with anchors chosen so that their UNTURNED boxes
    // cover most of each other and their turned boxes do not touch.
    let drawn = written("angled.kicad_sch");
    assert_eq!(drawn.len(), 2, "the fixture holds two labels");
    let (flat, upright) = (&drawn[0], &drawn[1]);
    assert_eq!(flat.angle, Angle(0));
    assert_eq!(upright.angle, Angle(90));
    assert_eq!(flat.text, upright.text, "the two strings are the same");

    // The lookalike, in this file's own arithmetic: left unturned, the two
    // boxes cover 71 % of each other, far above the published 20 %.
    let lookalike = shared_area(flat.unturned(), upright.unturned());
    let smaller = area(flat.unturned()).min(area(upright.unturned()));
    assert!(
        over_the_ratio(lookalike, smaller),
        "the unturned boxes do overlap: {lookalike} of {smaller}"
    );
    assert_eq!(lookalike * RATIO.1 / smaller, 71, "and by a wide margin");

    // The truth: turned, they share nothing at all. Not a small share — none.
    assert_eq!(
        shared_area(flat.turned(), upright.turned()),
        0,
        "the turned boxes share no area: {} against {}",
        flat.turned(),
        upright.turned()
    );

    // So the rule must say nothing. A rule built on the unturned box reports
    // one blocking finding here, on a drawing that is perfectly legible.
    let findings = findings_of("angled.kicad_sch");
    assert!(
        findings.is_empty(),
        "a turned label is clear of a flat one: {:?}",
        findings.iter().map(|f| &f.message).collect::<Vec<_>>()
    );
}

#[test]
fn two_hidden_fields_at_one_position_report_nothing_and_the_same_two_visible_do() {
    // The absence check and its presence control, on one geometry. Without the
    // control, a rule that reported nothing ever would pass the first half.
    for (name, visible) in [("hidden.kicad_sch", false), ("visible.kicad_sch", true)] {
        let drawn = written(name);
        let references: Vec<&Written> = drawn
            .iter()
            .filter(|item| item.text.starts_with('R') && item.text.len() == 2)
            .collect();
        assert_eq!(references.len(), 2, "{name} holds two reference fields");
        assert_eq!(
            references[0].at, references[1].at,
            "{name} puts them at one position"
        );
        assert_eq!(
            references.iter().all(|item| item.drawn),
            visible,
            "{name} draws them: {visible}"
        );

        // The geometry is the same in both files, and it is a total overlap.
        let one = references[0].turned();
        let two = references[1].turned();
        assert_eq!(one, two, "{name}: the two boxes are the same box");
        assert!(
            over_the_ratio(shared_area(one, two), area(one)),
            "{name}: that geometry is over the ratio"
        );

        let findings = findings_of(name);
        assert_eq!(
            findings.len(),
            usize::from(visible),
            "{name}: {:?}",
            findings.iter().map(|f| &f.message).collect::<Vec<_>>()
        );
        if visible {
            let message = &findings[0].message;
            assert!(
                message.contains("R1.Reference") && message.contains("R2.Reference"),
                "the finding names both fields: {message}"
            );
            assert_eq!(findings[0].objects.len(), 2);
            assert_ne!(findings[0].objects[0], findings[0].objects[1]);
        }
    }
}

#[test]
fn exactly_the_overlapping_pair_is_named_and_the_walk_reaches_past_two_others() {
    // Five pieces of free text. The pair that fires is the FIRST and the
    // FOURTH, so a walk that compared each object only with the next one in
    // file order finds nothing here. Between them sits a pair that overlaps by
    // 19.997 % of the smaller box, one internal unit below the published
    // threshold, so a rule with the comparison the wrong way round reports
    // that one instead.
    let drawn = written("one_pair.kicad_sch");
    assert_eq!(drawn.len(), 5, "the fixture holds five pieces of text");

    // Every pair, in this file's own arithmetic, with its share of the
    // smaller box.
    let mut over = Vec::new();
    let mut shares = Vec::new();
    for (place, one) in drawn.iter().enumerate() {
        for (also, two) in drawn.iter().enumerate().skip(place + 1) {
            let shared = shared_area(one.turned(), two.turned());
            let smaller = area(one.turned()).min(area(two.turned()));
            if shared > 0 {
                shares.push((place, also, shared * RATIO.1 / smaller));
            }
            if over_the_ratio(shared, smaller) {
                over.push((place, also));
            }
        }
    }
    // Three pairs touch; exactly one of the three is over the threshold.
    assert_eq!(
        shares,
        vec![(0, 2, 19), (0, 3, 53), (2, 3, 10)],
        "the fixture's three touching pairs and their shares"
    );
    assert_eq!(over, vec![(0, 3)], "exactly one pair is over the threshold");

    let findings = findings_of("one_pair.kicad_sch");
    assert_eq!(
        findings.len(),
        1,
        "{:?}",
        findings.iter().map(|f| &f.message).collect::<Vec<_>>()
    );
    let found = &findings[0];
    assert!(
        found.message.contains(&drawn[0].handle) && found.message.contains(&drawn[3].handle),
        "the finding names the first and the fourth: {}",
        found.message
    );
    for other in [1_usize, 2, 4] {
        assert!(
            !found.message.contains(&drawn[other].handle),
            "and names no other: {} holds {}",
            found.message,
            drawn[other].handle
        );
    }
    assert!(
        found.message.contains("53 %"),
        "the share is reported: {}",
        found.message
    );

    // The marker sits in the shared ink rather than at either anchor.
    let region = {
        let (one, two) = (drawn[0].turned(), drawn[3].turned());
        Rect::new(
            Point::new(
                one.start().x.0.max(two.start().x.0),
                one.start().y.0.max(two.start().y.0),
            ),
            Point::new(
                one.end().x.0.min(two.end().x.0),
                one.end().y.0.min(two.end().y.0),
            ),
        )
    };
    assert!(
        region.contains(found.pos),
        "the finding points at the overlap: {} is not in {region}",
        found.pos
    );
    assert_ne!(found.pos, drawn[0].at);
    assert_ne!(found.pos, drawn[3].at);
}

#[test]
fn one_internal_unit_more_overlap_crosses_the_published_ratio() {
    // The twin of the 19.997 % pair above, moved one internal unit. Nothing
    // else about the two drawings differs: same strings, same style, same
    // widths, the same anchor on x.
    let below = written("one_pair.kicad_sch");
    let above = written("boundary.kicad_sch");
    assert_eq!(above.len(), 2, "the boundary fixture holds one pair");

    let gap = |pair: &[&Written]| {
        i64::from(pair[1].at.y.0) - i64::from(pair[0].at.y.0)
    };
    let below_pair = [&below[0], &below[2]];
    let above_pair = [&above[0], &above[1]];
    assert_eq!(
        gap(&below_pair) - gap(&above_pair),
        1,
        "the two drawings differ by one internal unit of offset"
    );

    let share = |pair: &[&Written]| {
        let (one, two) = (pair[0].turned(), pair[1].turned());
        let shared = shared_area(one, two);
        let smaller = area(one).min(area(two));
        (shared, smaller, over_the_ratio(shared, smaller))
    };
    let (shared_below, smaller_below, fires_below) = share(&below_pair);
    let (shared_above, smaller_above, fires_above) = share(&above_pair);

    // The two shared areas differ by exactly the width of the boxes, which is
    // one internal unit of height across a box 110284 units wide.
    assert_eq!(smaller_below, smaller_above, "the boxes are the same size");
    assert_eq!(
        shared_above - shared_below,
        i64::from(above[0].turned().width().0),
        "one unit of height, across the whole width"
    );
    assert!(!fires_below, "19.997 % is below the published ratio");
    assert!(fires_above, "20.002 % is above it");

    // And the rule agrees, both ways round.
    assert!(findings_of("boundary.kicad_sch").len() == 1);
    let crowded = findings_of("one_pair.kicad_sch");
    assert_eq!(crowded.len(), 1);
    assert!(
        !crowded[0].message.contains(&below[2].handle),
        "the pair below the ratio is not the one reported: {}",
        crowded[0].message
    );
}

#[test]
fn one_overlapping_pair_fails_the_gate_whatever_the_sheet_holds() {
    // The tier's mechanism, and the measurement this rule's saturation
    // declaration rests on: the verdict never consults the share covered, so
    // no declaration this rule could make would change an outcome.
    let findings = findings_of("visible.kicad_sch");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].tier, kicli::lint::Tier::One);
    assert_eq!(findings[0].severity, kicli::lint::Severity::Error);
    assert_eq!(
        findings[0].penalty,
        kicli::lint::Penalty::ZERO,
        "a blocking rule does not move the score"
    );
    assert_eq!(
        findings[0].saturation,
        kicli::lint::Saturation::NEVER,
        "the declaration this rule makes, read back off a real finding"
    );

    for symbols in [2_u32, 2_000] {
        let gate = Gate::of(&findings, Density::of_counts(symbols, 0));
        assert!(!gate.passes(), "a sheet of {symbols} symbols fails");
        assert_eq!(gate.blockers().len(), 1);
        assert_eq!(gate.blockers()[0].word(), "blocking");
    }

    // A sheet of text with nothing wrong passes, so the gate is not failing on
    // everything. This drawing holds no symbols at all, which is the case a
    // symbol denominator could not have measured.
    let clean = findings_of("angled.kicad_sch");
    assert!(clean.is_empty());
    assert!(Gate::of(&clean, Density::of_counts(0, 0)).passes());
}
