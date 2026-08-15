//! What a route costs, over paths a generator produces rather than paths a
//! test author chose.
//!
//! One drawing carries a wire to cross, a label to run through and a symbol to
//! crowd. Every monotone staircase between two corners of the lattice is then
//! walked over it. The paths all have the same length and differ in everything
//! else, which is what makes the parts worth reporting separately.

use kicli::geometry::{GRID, Iu, Point};
use kicli::model::{Config, Hierarchy};
use kicli::route::{Cost, Obstacles, Routed, SheetObjects, Tally, Uncostable, Window};
use kicli_probe::{Probe, pin, rectangle, symbol};
use std::path::{Path, PathBuf};

/// Where this binary writes the drawings it builds.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("route-cost")
}

/// A point from the millimetres the drawing is written in.
fn at(x: &str, y: &str) -> Point {
    let read = |text: &str| {
        Iu::from_millimetres_text(text)
            .expect("a millimetre reading")
            .0
    };
    Point::new(read(x), read(y))
}

/// A symbol with a square body and one pin on each of its four edges.
fn quad() -> String {
    symbol(
        "QUAD",
        "U",
        false,
        &[(
            "1_1",
            vec![
                rectangle(("-2.54", "-2.54"), ("2.54", "2.54")),
                pin("passive", ("-3.81", "0"), "0", "1", "W"),
                pin("passive", ("0", "-3.81"), "90", "2", "S"),
                pin("passive", ("3.81", "0"), "180", "3", "E"),
                pin("passive", ("0", "3.81"), "270", "4", "N"),
            ],
        )],
    )
}

/// The drawing every check here walks over.
///
/// A vertical wire of another net stands in the middle of the field, a label
/// sits across the top left of it, and a symbol stands to the right of the last
/// column, so that column is crowded without being blocked.
///
/// Each check writes its own copy, because two tests of one binary run at once
/// and would otherwise read a file the other was still writing. The terminals
/// are a parameter: a pin the route ends on is not an obstacle, and neither is
/// the step it escapes along.
fn field(name: &str, terminals: &[String]) -> Obstacles {
    let hierarchy = drawing(name);
    let objects = SheetObjects::read(
        hierarchy.files.first().expect("the probe has a root sheet"),
        &hierarchy
            .placements
            .first()
            .expect("the root sheet is placed")
            .path,
        &Routed {
            wires: &[],
            terminals,
        },
    );
    let window = Window::around(
        START,
        END,
        Config::default().routing.margin,
        objects.page(),
        GRID,
    );
    Obstacles::build(window, &objects.geometry())
}

/// The probe drawing itself.
fn drawing(name: &str) -> Hierarchy {
    let mut probe = Probe::new(name, scratch());
    probe.define(quad());
    probe.wire(("92.71", "87.63"), ("92.71", "95.25"));
    probe.label_of_kind("label", "", "NN", ("90.17", "91.44"));
    probe.place("QUAD", "U1", ("101.6", "91.44"), &["1", "2", "3", "4"]);
    Hierarchy::load(&probe.write()).expect("the probe loads")
}

/// The corner every generated path starts at.
const START: Point = Point::new(889_000, 889_000);
/// The corner every generated path ends at.
const END: Point = Point::new(965_200, 939_800);

/// How many grid steps the field is, across and down.
const COLUMNS: u32 = 6;
/// How many grid steps down.
const ROWS: u32 = 4;

/// Every monotone staircase from [`START`] to [`END`], as vertex lists.
///
/// A monotone path is a sequence of right and down steps in some order, so the
/// set is every arrangement of `ROWS` down steps among `COLUMNS + ROWS` steps.
/// Runs in one direction collapse into one vertex, which is the shape a wire
/// record is written from.
fn staircases() -> Vec<Vec<Point>> {
    let total = COLUMNS + ROWS;
    let mut paths = Vec::new();
    for mask in 0..1u32 << total {
        if mask.count_ones() != ROWS {
            continue;
        }
        let mut vertices = vec![START];
        let mut at = START;
        let mut last_down: Option<bool> = None;
        for step in 0..total {
            let down = mask & (1 << step) != 0;
            at = if down {
                Point::new(at.x.0, at.y.0 + GRID.0)
            } else {
                Point::new(at.x.0 + GRID.0, at.y.0)
            };
            // A vertex is where the direction changes; the run between two of
            // them is one wire record.
            if last_down.is_some_and(|last| last != down) {
                vertices.push(if down {
                    Point::new(at.x.0, at.y.0 - GRID.0)
                } else {
                    Point::new(at.x.0 - GRID.0, at.y.0)
                });
            }
            last_down = Some(down);
        }
        vertices.push(at);
        assert_eq!(at, END, "a staircase ends at the far corner");
        paths.push(vertices);
    }
    paths
}

#[test]
fn the_cost_breakdown_sums_to_its_total() {
    let weights = Config::default().routing;
    let map = field("cost-breakdown", &[]);
    let paths = staircases();
    assert_eq!(paths.len(), 210, "every arrangement of the down steps");

    let mut costed = 0;
    let mut blocked = 0;
    let mut worst = Tally::default();
    for path in &paths {
        let tally = match Tally::of_path(path, &map) {
            Ok(tally) => tally,
            Err(Uncostable::Blocked { .. }) => {
                blocked += 1;
                continue;
            }
            Err(other) => panic!("a staircase is not a route: {other}"),
        };
        let cost = Cost::of(tally, &weights);

        // Each part is its own count at its own weight, worked out here rather
        // than read back from the code that made it.
        assert_eq!(cost.length, weights.w_len * i64::from(tally.steps));
        assert_eq!(cost.turns, weights.w_turn * i64::from(tally.corners));
        assert_eq!(cost.crossings, weights.w_cross * i64::from(tally.crossings));
        assert_eq!(cost.text, weights.w_text * i64::from(tally.text_steps));
        assert_eq!(cost.proximity, weights.w_near * i64::from(tally.near_steps));

        let summed = cost.length + cost.turns + cost.crossings + cost.text + cost.proximity;
        assert_eq!(cost.total(), summed, "{tally:?}");
        assert_eq!(
            cost.total(),
            cost.parts().iter().map(|&(_, part)| part).sum::<i64>(),
            "the report prints the parts the total is made of"
        );

        // Every monotone staircase walks the same distance, so length alone
        // tells the paths apart not at all. That is why the parts exist.
        assert_eq!(tally.steps, COLUMNS + ROWS, "{path:?}");

        worst = Tally {
            steps: worst.steps.max(tally.steps),
            corners: worst.corners.max(tally.corners),
            crossings: worst.crossings.max(tally.crossings),
            text_steps: worst.text_steps.max(tally.text_steps),
            near_steps: worst.near_steps.max(tally.near_steps),
        };
        costed += 1;
    }

    // The control against a check that measured four zeros and a length: every
    // part is reached by some path, and some path is refused outright.
    assert!(costed > 20, "{costed} of {} paths costed", paths.len());
    assert!(blocked > 0, "no path was refused, so the field is empty");
    assert!(worst.corners > 0, "no path turned");
    assert!(worst.crossings > 0, "no path crossed the wire");
    assert!(worst.text_steps > 0, "no path ran through the label");
    assert!(worst.near_steps > 0, "no path crowded the symbol");
}

#[test]
fn a_path_that_cannot_be_walked_is_refused_rather_than_costed() {
    let map = field("cost-refusals", &[]);
    let step = GRID.0;

    let diagonal = [START, Point::new(START.x.0 + step, START.y.0 + step)];
    assert!(matches!(
        Tally::of_path(&diagonal, &map),
        Err(Uncostable::Diagonal { .. })
    ));

    let half = [START, Point::new(START.x.0 + step / 2, START.y.0)];
    assert!(matches!(
        Tally::of_path(&half, &map),
        Err(Uncostable::OffGrid { .. })
    ));

    let still = [START, START];
    assert!(matches!(
        Tally::of_path(&still, &map),
        Err(Uncostable::Stationary { .. })
    ));
    assert!(matches!(
        Tally::of_path(&[START], &map),
        Err(Uncostable::TooShort(1))
    ));

    // A route laid along the foreign wire is blocked, and the refusal names it.
    let along = [at("92.71", "88.9"), at("92.71", "93.98")];
    let refused = Tally::of_path(&along, &map).expect_err("the wire is in the way");
    let Uncostable::Blocked {
        handle,
        at: stopped,
    } = refused
    else {
        panic!("{refused}");
    };
    assert_eq!(stopped, at("92.71", "90.17"), "the first step onto it");
    assert!(!handle.is_empty());

    // The control: the same two points, met across the wire instead of along
    // it, cost one crossing and are not refused.
    let across = [at("90.17", "91.44"), at("95.25", "91.44")];
    let tally = Tally::of_path(&across, &map).expect("a crossing is not a block");
    assert_eq!(tally.crossings, 1);
}

#[test]
fn the_route_arrives_at_a_terminal_that_would_otherwise_block_it() {
    // A terminal sits on its own pin, which is inside its own symbol's body
    // box. The walk enters every point but the first and excepts the last from
    // blocks, so a route may arrive where it may not pass. Without the
    // exception no route could reach any pin at all.
    //
    // The pin itself, and the step it escapes along, are named as the route's
    // own, because a pin the route ends on is not an obstacle. What is left
    // covering the terminus is the symbol's body, and that is what the
    // exception has to answer for.
    let terminals = ["U1.1".to_owned()];
    let map = field("cost-terminal", &terminals);
    let pin = at("97.79", "91.44");
    let approach = at("95.25", "91.44");

    let arriving = [approach, pin];
    let tally = Tally::of_path(&arriving, &map).expect("a route may arrive at its terminal");
    assert_eq!(tally.steps, 2, "the two steps from the approach to the pin");

    // Passing through the same point instead of stopping there is refused.
    let through = [approach, at("100.33", "91.44")];
    let refused = Tally::of_path(&through, &map).expect_err("the body is in the way");
    let Uncostable::Blocked {
        handle,
        at: stopped,
    } = refused
    else {
        panic!("{refused}");
    };
    assert_eq!(handle, "U1", "the body, not the pin");
    assert_eq!(stopped, pin, "the point it was allowed to arrive at");

    // The control: with the pin left as an obstacle, the same arrival is
    // refused, so it is the caller's answer that opens the terminus and not a
    // hole in the map.
    let closed = field("cost-terminal-control", &[]);
    assert!(Tally::of_path(&arriving, &closed).is_err());
}
