//! Free text and text boxes: what the five operations do to a file.
//!
//! Every test mutates a copy in a scratch directory. The committed fixture
//! tree is never written by a test.

use kicli::edit::text::{self, NewText};
use kicli::geometry::{Angle, GRID, Point, Size};
use kicli::model::{Item, Schematic, SheetPath, Target, WriteOptions};
use kicli_sexpr::Doc;
use std::path::PathBuf;

use kicli_probe::scratch::Fixtures;

/// The committed fixtures this binary reads, and the scratch it writes in.
fn fixtures() -> Fixtures {
    Fixtures::new(env!("CARGO_TARGET_TMPDIR"), env!("CARGO_MANIFEST_DIR"))
}

/// A sheet with one text item already on it, so the file is not empty.
const SHEET: &str = concat!(
    "(kicad_sch\n",
    "\t(version 20260306)\n",
    "\t(generator \"eeschema\")\n",
    "\t(generator_version \"10.0\")\n",
    "\t(uuid \"15000001-0000-4000-8000-050000000000\")\n",
    "\t(paper \"A4\")\n",
    "\t(text \"a note that stays\"\n",
    "\t\t(exclude_from_sim no)\n",
    "\t\t(at 25.4 25.4 0)\n",
    "\t\t(effects\n",
    "\t\t\t(font\n",
    "\t\t\t\t(size 1.27 1.27)\n",
    "\t\t\t)\n",
    "\t\t)\n",
    "\t\t(uuid \"00000000-0000-4000-8000-050000000001\")\n",
    "\t)\n",
    "\t(sheet_instances\n",
    "\t\t(path \"/\"\n",
    "\t\t\t(page \"1\")\n",
    "\t\t)\n",
    "\t)\n",
    "\t(embedded_fonts no)\n",
    ")\n",
);

/// A copy of the sheet in a scratch directory, with the paths a command needs.
struct Sheet {
    project: PathBuf,
    file: PathBuf,
}

impl Sheet {
    /// Write the sheet into its own scratch directory, in canonical form.
    fn new(name: &str) -> Self {
        let project = fixtures().scratch(name);
        let file = project.join("sheet.kicad_sch");
        let canonical = Doc::parse(SHEET).expect("the sheet parses").emit();
        std::fs::write(&file, &canonical).expect("the sheet is written");
        Self { project, file }
    }

    /// The file as it stands.
    fn bytes(&self) -> String {
        std::fs::read_to_string(&self.file).expect("the sheet reads")
    }

    /// The document and the sheet path a command needs.
    fn open(&self) -> (Doc, SheetPath) {
        let doc = Doc::parse(&self.bytes()).expect("the sheet parses");
        let schematic = Schematic::read(&doc).expect("the sheet reads");
        let path = SheetPath::root(schematic.uuid.as_ref().expect("the sheet has a uuid"));
        (doc, path)
    }

    /// Where a write lands, and under what rules.
    fn target<'a>(&'a self, path: &'a SheetPath) -> Target<'a> {
        Target {
            path: &self.file,
            project: &self.project,
            sheet_path: path,
            grid: GRID,
            options: WriteOptions::default(),
        }
    }

    /// The text items of the file, in file order.
    fn texts(&self) -> Vec<kicli::model::TextItem> {
        let doc = Doc::parse(&self.bytes()).expect("parses");
        let schematic = Schematic::read(&doc).expect("reads");
        schematic
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Text(text) => Some(text.clone()),
                _ => None,
            })
            .collect()
    }
}

/// The `(size w h)` of the list that owns `uuid`, as written.
fn written_size(sheet: &Sheet, uuid: &str) -> Option<(String, String)> {
    let doc = Doc::parse(&sheet.bytes()).expect("parses");
    let node = doc.uuid_index().get(uuid).copied()?;
    let size = doc
        .children(node)
        .iter()
        .copied()
        .find(|&child| doc.head_is(child, "size"))?;
    let values = doc.children(size);
    Some((
        doc.atom_text(*values.get(1)?)?.to_owned(),
        doc.atom_text(*values.get(2)?)?.to_owned(),
    ))
}

fn note(at: Point) -> NewText {
    NewText {
        text: "a new note".to_owned(),
        at,
        angle: Angle(0),
        size: None,
    }
}

#[test]
fn text_survives_the_four_operations() {
    let sheet = Sheet::new("text_four_operations");
    let start = sheet.bytes();

    let (mut doc, path) = sheet.open();
    let added = text::add(
        &mut doc,
        &sheet.target(&path),
        &note(Point::new(508_000, 508_000)),
        "2026-01-02T03:04:05Z",
    )
    .expect("the text is added");
    assert!(!added.mutation.reformatted, "the sheet arrived canonical");
    assert!(added.mutation.invariants.passed());
    assert_ne!(sheet.bytes(), start, "the file gained the text");
    assert_eq!(sheet.texts().len(), 2);

    let (mut doc, path) = sheet.open();
    text::move_to(
        &mut doc,
        &sheet.target(&path),
        &added.uuid,
        Point::new(635_000, 762_000),
        "2026-01-02T03:04:06Z",
    )
    .expect("the text moves");
    assert!(
        sheet.bytes().contains("(at 63.5 76.2 0)"),
        "{}",
        sheet.bytes()
    );

    let (mut doc, path) = sheet.open();
    text::edit(
        &mut doc,
        &sheet.target(&path),
        &added.uuid,
        "a different note",
        "2026-01-02T03:04:07Z",
    )
    .expect("the text changes");
    assert!(sheet.bytes().contains("a different note"));

    let (mut doc, path) = sheet.open();
    text::delete(
        &mut doc,
        &sheet.target(&path),
        &added.uuid,
        "2026-01-02T03:04:08Z",
    )
    .expect("the text goes");

    assert_eq!(
        sheet.bytes(),
        start,
        "the file is as it started, byte for byte"
    );
}

#[test]
fn a_text_box_keeps_its_size() {
    let sheet = Sheet::new("text_box_size");

    let (mut doc, path) = sheet.open();
    let request = NewText {
        size: Some(Size::new(508_000, 254_000)),
        ..note(Point::new(508_000, 508_000))
    };
    let added = text::add(
        &mut doc,
        &sheet.target(&path),
        &request,
        "2026-01-02T03:04:05Z",
    )
    .expect("the box is added");

    assert_eq!(
        written_size(&sheet, &added.uuid.0),
        Some(("50.8".to_owned(), "25.4".to_owned())),
        "the box re-parses with the size it was given"
    );
    let boxed = sheet
        .texts()
        .into_iter()
        .find(|item| item.uuid == added.uuid)
        .expect("the box is in the file");
    assert!(boxed.boxed, "and it is a box, not free text");

    let before = sheet.bytes();
    let (mut doc, path) = sheet.open();
    text::resize(
        &mut doc,
        &sheet.target(&path),
        &added.uuid,
        Size::new(762_000, 304_800),
        "2026-01-02T03:04:06Z",
    )
    .expect("the box is resized");

    assert_eq!(
        written_size(&sheet, &added.uuid.0),
        Some(("76.2".to_owned(), "30.48".to_owned()))
    );
    let after = sheet.bytes();
    let differing: Vec<(&str, &str)> = before
        .lines()
        .zip(after.lines())
        .filter(|(one, other)| one != other)
        .collect();
    assert_eq!(
        differing,
        vec![("\t\t(size 50.8 25.4)", "\t\t(size 76.2 30.48)")],
        "a resize changes only the size"
    );
    assert_eq!(before.lines().count(), after.lines().count());
}

#[test]
fn free_text_is_not_snapped_to_the_grid() {
    let sheet = Sheet::new("text_off_grid");
    let (mut doc, path) = sheet.open();
    // Graphic text is exempt from the grid rule: it carries no anchor a net
    // can reach.
    text::add(
        &mut doc,
        &sheet.target(&path),
        &note(Point::new(304_900, 508_100)),
        "2026-01-02T03:04:05Z",
    )
    .expect("the text is added where it was asked for");
    assert!(
        sheet.bytes().contains("(at 30.49 50.81 0)"),
        "{}",
        sheet.bytes()
    );
}

#[test]
fn a_text_command_needs_a_text_item() {
    let sheet = Sheet::new("text_wrong_object");
    let (mut doc, path) = sheet.open();
    let missing = kicli::model::Uuid("00000000-0000-4000-8000-05000000ffff".to_owned());
    let refused = text::move_to(
        &mut doc,
        &sheet.target(&path),
        &missing,
        Point::new(0, 0),
        "2026-01-02T03:04:05Z",
    );
    assert!(refused.is_err(), "an identifier the file does not hold");
}
