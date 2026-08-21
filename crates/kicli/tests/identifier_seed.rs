//! The identifiers a write invents name the design, not the checkout.
//!
//! Every new object kicli writes gets an identifier derived from a seed rather
//! than from a random source, so one command run twice over one design writes
//! one file. That reason says nothing about *where* the design sits, and until
//! this check existed the seed carried the document's **absolute** path — so
//! the same design in two checkouts wrote two files that differed only in
//! their identifiers. The T16 golden defect was exactly that, discovered at a
//! merge.
//!
//! So this binary writes the same design twice, at two directories of its own,
//! and compares. The control is the part that makes it a check rather than a
//! ritual: it asserts the two absolute paths really did differ, and that
//! something was really written, before it asserts the identifiers match.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use kicli::edit::label::{self, NewLabel, PortShape};
use kicli::edit::mark::PinAddress;
use kicli::edit::text::{self, NewText};
use kicli::edit::wire::{Connection, Destination, End, Polyline};
use kicli::geometry::{Angle, GRID, Point};
use kicli::model::{
    Config, Hierarchy, LabelKind, Refdes, Schematic, SheetPath, Target, WriteOptions,
};
use kicli_probe::Probe;
use kicli_sexpr::Doc;

/// Where this binary writes the checkouts it compares.
fn scratch() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("identifier-seed")
}

/// The timestamp every write here records, so no clock reaches the answer.
const TAKEN: &str = "2026-01-02T03:04:05Z";

/// A point, in the internal units a KiCad file's millimetres become.
const fn at(x: i32, y: i32) -> Point {
    Point::new(x, y)
}

/// Pin 1 of a placed symbol, as a request names it.
fn pin_of(reference: &str) -> End {
    End::Pin(PinAddress::new(Refdes(reference.to_owned()), "1"))
}

/// Four resistors, pin 1 of each at `y = 50.8`: two to draw between, two to
/// route between.
///
/// A resistor's pin 1 sits above its anchor and its body below it, so a wire
/// leaves pin 1 upwards, which is the escape both drawn ends honour.
fn four_resistors(name: &str) -> PathBuf {
    let mut probe = Probe::new(name, scratch());
    probe.place("R", "R1", ("50.8", "54.61"), &["1", "2"]);
    probe.place("R", "R2", ("76.2", "54.61"), &["1", "2"]);
    probe.place("R", "R3", ("127", "54.61"), &["1", "2"]);
    probe.place("R", "R4", ("152.4", "54.61"), &["1", "2"]);
    probe.write()
}

/// The target a write lands through: the file, its project, its sheet.
fn target<'a>(file: &'a Path, project: &'a Path, sheet: &'a SheetPath) -> Target<'a> {
    Target {
        path: file,
        project,
        sheet_path: sheet,
        grid: GRID,
        options: WriteOptions::default(),
    }
}

/// The document and the sheet path a per-file command needs.
fn open(file: &Path) -> (Doc, SheetPath) {
    let bytes = std::fs::read_to_string(file).expect("the drawing reads");
    let doc = Doc::parse(&bytes).expect("the drawing parses");
    let schematic = Schematic::read(&doc).expect("the drawing reads");
    let sheet = SheetPath::root(schematic.uuid.as_ref().expect("the drawing has a uuid"));
    (doc, sheet)
}

/// One design, written where it was written, and what the four verbs invented.
struct Checkout {
    /// The file every verb wrote to.
    file: PathBuf,
    /// The project directory it sits in.
    project: PathBuf,
    /// Every identifier the four verbs reported, in the order they ran.
    identifiers: Vec<String>,
    /// The sheet as the four verbs left it.
    bytes: String,
}

impl Checkout {
    /// Where the file sits inside its own project.
    fn inside(&self) -> &Path {
        self.file
            .strip_prefix(&self.project)
            .expect("the drawing sits in its project")
    }
}

/// Run all four seeding verbs over one drawing, in a directory of its own.
///
/// The four are the four sites that build a seed from the document: `wire
/// draw`, `wire connect`, `label add` and `text add`. Each is given the same
/// request in every checkout, so the only thing that differs between two calls
/// of this function is the directory `name` puts the design in.
fn write_everything(name: &str) -> Checkout {
    let file = four_resistors(name);
    let project = file
        .parent()
        .expect("the drawing sits in a directory")
        .to_owned();
    let routing = Config::default().routing;
    let mut identifiers = Vec::new();

    // wire draw: up from R1.1, across, and down into R2.1.
    let mut hierarchy = Hierarchy::load(&file).expect("the drawing loads");
    let sheet = hierarchy.placements[0].path.clone();
    let drawn = kicli::edit::wire::draw(
        &mut hierarchy,
        &Polyline {
            from: pin_of("R1"),
            to: pin_of("R2"),
            via: vec![at(508_000, 457_200), at(762_000, 457_200)],
        },
        &routing,
        &target(&file, &project, &sheet),
        TAKEN,
    )
    .expect("the polyline is drawable");
    assert_eq!(drawn.report.added.wires.len(), 3, "one record per segment");
    identifiers.extend(drawn.report.added.wires.iter().map(|uuid| uuid.0.clone()));

    // wire connect: the router's own path from R3.1 to R4.1.
    let mut hierarchy = Hierarchy::load(&file).expect("the drawing loads");
    let sheet = hierarchy.placements[0].path.clone();
    let where_to = target(&file, &project, &sheet);
    let plan = kicli::edit::wire::plan(
        &hierarchy,
        &Connection {
            from: pin_of("R3"),
            to: Destination::End(pin_of("R4")),
        },
        &routing,
        &where_to,
    )
    .expect("the connection is plannable");
    let connected = kicli::edit::wire::draw_plan(&mut hierarchy, &plan, &routing, &where_to, TAKEN)
        .expect("the route is drawable");
    assert_eq!(
        connected.report.added.wires.len(),
        3,
        "the router leaves R3.1, crosses, and drops into R4.1"
    );
    identifiers.extend(
        connected
            .report
            .added
            .wires
            .iter()
            .map(|uuid| uuid.0.clone()),
    );
    identifiers.extend(
        connected
            .report
            .added
            .junctions
            .iter()
            .map(|uuid| uuid.0.clone()),
    );

    // label add: a local label on free space, joining nothing.
    let (mut doc, sheet) = open(&file);
    let labelled = label::add(
        &mut doc,
        &target(&file, &project, &sheet),
        &file,
        &NewLabel {
            kind: LabelKind::Local,
            text: "SEED".to_owned(),
            at: at(254_000, 1_016_000),
            angle: Angle(0),
            shape: PortShape::Passive,
        },
        TAKEN,
    )
    .expect("the label is added");
    identifiers.push(labelled.uuid.0.clone());

    // text add: a note on free space.
    let (mut doc, sheet) = open(&file);
    let noted = text::add(
        &mut doc,
        &target(&file, &project, &sheet),
        &NewText {
            text: "a note the seed check writes".to_owned(),
            at: at(254_000, 1_143_000),
            angle: Angle(0),
            size: None,
        },
        TAKEN,
    )
    .expect("the text is added");
    identifiers.push(noted.uuid.0.clone());

    let bytes = std::fs::read_to_string(&file).expect("the drawing reads");
    Checkout {
        file,
        project,
        identifiers,
        bytes,
    }
}

#[test]
fn one_design_in_two_checkouts_writes_one_set_of_identifiers() {
    let here = write_everything("a-checkout");
    let elsewhere = write_everything("a-checkout-somewhere-else-entirely");

    // The control, first, because the assertion below is worthless without it.
    // Two runs that shared a directory would agree about everything and prove
    // nothing.
    assert!(
        here.file.is_absolute() && elsewhere.file.is_absolute(),
        "both checkouts are named absolutely: {} and {}",
        here.file.display(),
        elsewhere.file.display()
    );
    assert_ne!(
        here.file, elsewhere.file,
        "the two checkouts really are two directories"
    );
    assert_ne!(
        here.project, elsewhere.project,
        "and two projects, not one project written twice"
    );
    assert_eq!(
        here.inside(),
        elsewhere.inside(),
        "the design is at the same place inside each of them"
    );

    // The second control: the verbs really invented something. Two empty runs
    // would also be equal.
    assert_eq!(
        here.identifiers.len(),
        8,
        "three drawn segments, three routed ones, a label and a note: {:?}",
        here.identifiers
    );
    assert_eq!(
        here.identifiers.iter().collect::<BTreeSet<_>>().len(),
        here.identifiers.len(),
        "every identifier one design wrote is its own: {:?}",
        here.identifiers
    );

    assert_eq!(
        here.identifiers, elsewhere.identifiers,
        "the same design at two absolute paths wrote two sets of identifiers"
    );
    assert_eq!(
        here.bytes, elsewhere.bytes,
        "and so the two files are the same file"
    );
}

#[test]
fn one_seed_twice_over_one_file_still_writes_two_objects() {
    // The seed no longer says where the checkout is, so two projects at one
    // relative path reach one seed. That is only safe if a repeated seed is
    // resolved rather than written, which is what this checks — on the verb,
    // not on the derivation.
    let file = four_resistors("one-seed-twice");
    let project = file
        .parent()
        .expect("the drawing sits in a directory")
        .to_owned();
    let note = || NewText {
        text: "the very same note".to_owned(),
        at: at(254_000, 1_143_000),
        angle: Angle(0),
        size: None,
    };

    let (mut doc, sheet) = open(&file);
    let first = text::add(&mut doc, &target(&file, &project, &sheet), &note(), TAKEN)
        .expect("the first note is added");

    let (mut doc, sheet) = open(&file);
    let second = text::add(&mut doc, &target(&file, &project, &sheet), &note(), TAKEN)
        .expect("the second note is added");

    assert_ne!(
        first.uuid, second.uuid,
        "the second object of an identical seed took the identifier the file did not hold"
    );
    let written = std::fs::read_to_string(&file).expect("the drawing reads");
    assert!(
        written.contains(&first.uuid.0) && written.contains(&second.uuid.0),
        "both notes are in the file under their own identifiers"
    );
}
