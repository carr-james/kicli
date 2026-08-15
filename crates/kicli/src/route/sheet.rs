//! One loaded sheet, sorted into the lists the search reads.
//!
//! The search is pure over boxes, pins and segments. Something has to turn a
//! file into those lists, and this is it: symbol bodies through
//! [`symbol_boxes`], pins through [`resolve_pins`], label and text boxes
//! through [`text_box`], and the wires, junctions and no-connects as the file
//! draws them.
//!
//! **Whose a wire is, is not the search's question.** A wire of the net being
//! routed is free to enter and ends the route; a wire of any other net blocks
//! along its own axis. Deciding which is which is connectivity's work, so the
//! caller states the answer in [`Routed`] and this module records it.

use kicli_sexpr::{Doc, NodeId};

use crate::geometry::text::TextStyle;
use crate::geometry::{Point, Rect, Size, resolve_pins, symbol_boxes, text_box};
use crate::model::items::{Item, SheetPath, Uuid};
use crate::model::{LoadedFile, definition_of, read_library};
use crate::route::obstacles::{PinObstacle, Segment, SheetGeometry};
use crate::route::terminal::{Heading, Obstruction};

/// Internal units per mil, in a schematic.
///
/// KiCad holds a page size in mils and scales it by `IU_PER_MILS`, which for a
/// schematic is `1e4 · 0.0254` (`include/base_units.h:69-86`). A page size is
/// therefore a whole number of internal units, and no float is needed to reach
/// it.
const IU_PER_MIL: i32 = 254;

/// The page sizes KiCad names, landscape, in mils.
///
/// `PAGE_INFO::standardPageSizes` (`common/page_info.cpp:46-67`) at tag 10.0.5.
/// The A series is written there in millimetres and converted with
/// `EDA_UNIT_UTILS::Mm2mils`, which rounds; the rounded mil values are what
/// KiCad works in, so they are what is written here. `User` carries its own
/// size in the file and is not in this table.
const PAGE_SIZES: &[(&str, i32, i32)] = &[
    ("A5", 8_268, 5_827),
    ("A4", 11_693, 8_268),
    ("A3", 16_535, 11_693),
    ("A2", 23_386, 16_535),
    ("A1", 33_110, 23_386),
    ("A0", 46_811, 33_110),
    ("A", 11_000, 8_500),
    ("B", 17_000, 11_000),
    ("C", 22_000, 17_000),
    ("D", 34_000, 22_000),
    ("E", 44_000, 34_000),
    ("GERBER", 32_000, 32_000),
    ("USLetter", 11_000, 8_500),
    ("USLegal", 14_000, 8_500),
    ("USLedger", 17_000, 11_000),
];

/// The page a file names when it names none, which is KiCad's own default.
const DEFAULT_PAGE: (&str, i32, i32) = PAGE_SIZES[1];

/// The smallest and largest page a schematic may carry, in internal units.
///
/// `MIN_PAGE_SIZE_MM` and `MAX_PAGE_SIZE_EESCHEMA_MM` (`include/page_info.h:39-
/// 41`), which the parser clamps a custom size to
/// (`sch_io_kicad_sexpr_parser.cpp:2141-2159`).
const PAGE_LIMITS: (i32, i32) = (254_000, 30_480_000);

/// What the route being planned owns.
///
/// Two lists, because two of the obstacle rules ask whose an object is. A wire
/// of the net being routed is free to enter and ends the route there; a pin the
/// route ends on is a terminal rather than an obstacle. Neither question is
/// geometry, so neither is answered here.
#[derive(Clone, Copy, Debug, Default)]
pub struct Routed<'a> {
    /// The wires of the net being routed, by identifier.
    pub wires: &'a [Uuid],
    /// The terminals the route ends on, named as [`Terminal::name`] names them:
    /// `R12.1` for a symbol pin, the port name for a sheet pin.
    ///
    /// [`Terminal::name`]: crate::route::Terminal::name
    pub terminals: &'a [String],
}

impl Routed<'_> {
    /// Is this wire part of the net being routed?
    fn owns_wire(&self, uuid: &Uuid) -> bool {
        self.wires.contains(uuid)
    }

    /// Is this pin one the route ends on?
    fn owns_terminal(&self, name: &str) -> bool {
        self.terminals.iter().any(|terminal| terminal == name)
    }
}

/// The objects of one sheet, in the lists [`SheetGeometry`] borrows.
///
/// The lists are owned here and lent to the search, because the search holds
/// slices so that it copies nothing while it runs.
#[derive(Clone, Debug, Default)]
pub struct SheetObjects {
    /// The area of the page the sheet is drawn on.
    page: Rect,
    symbol_bodies: Vec<Obstruction>,
    sheet_bodies: Vec<Obstruction>,
    pins: Vec<PinObstacle>,
    junctions: Vec<Obstruction>,
    no_connects: Vec<Obstruction>,
    segments: Vec<Segment>,
    texts: Vec<Obstruction>,
}

impl SheetObjects {
    /// Read one placement of one file into the router's lists.
    ///
    /// The sheet path picks the placement, because a reference designator
    /// belongs to a placement rather than to a symbol. Everything else a file
    /// draws is the same on every placement of it.
    ///
    /// A symbol whose library definition is not embedded contributes nothing:
    /// there is no box to measure and no pin to place. It is the same silence
    /// the extractor keeps, and the project check is what reports the missing
    /// definition.
    #[must_use]
    pub fn read(file: &LoadedFile, path: &SheetPath, routed: &Routed) -> Self {
        let mut objects = Self {
            page: page_area(&file.doc),
            ..Self::default()
        };
        objects.read_symbols(file, path, routed);
        objects.read_sheets(file);
        objects.read_text(file);
        objects.read_wires_and_marks(file, routed);
        objects
    }

    /// The body box and the pins of every placed symbol.
    fn read_symbols(&mut self, file: &LoadedFile, path: &SheetPath, routed: &Routed) {
        let schematic = &file.schematic;
        let library = read_library(&file.doc, &schematic.library_symbols, schematic.version);
        for symbol in schematic.symbols() {
            let name = symbol
                .reference_on(path)
                .map_or_else(|| symbol.uuid.short().to_owned(), |refdes| refdes.0.clone());
            let Some(definition) = definition_of(&library, symbol) else {
                continue;
            };
            self.symbol_bodies.push(Obstruction {
                handle: name.clone(),
                area: symbol_boxes(&file.doc, symbol, definition).body,
            });
            // A hidden pin is drawn by nothing and still connects, so a route
            // that met one would join a net the reader cannot see.
            for pin in resolve_pins(&symbol.drawn_on(path), definition) {
                let handle = format!("{name}.{}", pin.number);
                if routed.owns_terminal(&handle) {
                    continue;
                }
                self.pins.push(PinObstacle {
                    handle,
                    at: pin.position,
                    escape: Heading::from_schematic_angle(pin.direction).map(Heading::reversed),
                });
            }
        }
    }

    /// The body box of every child sheet.
    ///
    /// A sheet's own pins are terminals rather than obstacles, and they sit on
    /// the border, which the body box already covers.
    fn read_sheets(&mut self, file: &LoadedFile) {
        for sheet in file.schematic.sheets() {
            self.sheet_bodies.push(Obstruction {
                handle: sheet
                    .name()
                    .map_or_else(|| sheet.uuid.short().to_owned(), str::to_owned),
                area: Rect::from_origin(
                    sheet.at,
                    Size {
                        x: sheet.size.0,
                        y: sheet.size.1,
                    },
                ),
            });
        }
    }

    /// The box of every label and every piece of free text.
    fn read_text(&mut self, file: &LoadedFile) {
        let boxed = |text: &str, at, angle, node| {
            text_box(text, at, angle, &TextStyle::read(&file.doc, node)).axis_aligned()
        };
        for label in file.schematic.labels() {
            self.texts.push(Obstruction {
                handle: label.uuid.short().to_owned(),
                area: boxed(&label.text, label.at, label.angle, label.node),
            });
        }
        for item in &file.schematic.items {
            if let Item::Text(text) = item {
                self.texts.push(Obstruction {
                    handle: text.uuid.short().to_owned(),
                    area: boxed(&text.text, text.at, text.angle, text.node),
                });
            }
        }
    }

    /// Every wire and bus, and the junctions and no-connects on them.
    ///
    /// A bus is laid down as a wire of another net. Drawing along one reads as
    /// a connection exactly as drawing along a wire does, and this milestone
    /// routes no buses, so the conservative treatment is the right one.
    ///
    /// A bus entry is left out. It is drawn as a diagonal stub, and the lattice
    /// has no way to describe one.
    fn read_wires_and_marks(&mut self, file: &LoadedFile, routed: &Routed) {
        for line in file.schematic.lines() {
            self.segments.push(Segment {
                handle: line.uuid.short().to_owned(),
                from: line.from,
                to: line.to,
                own_net: routed.owns_wire(&line.uuid),
            });
        }
        for item in &file.schematic.items {
            match item {
                Item::Junction(junction) => self.junctions.push(Obstruction {
                    handle: junction.uuid.short().to_owned(),
                    area: Rect::around(junction.at),
                }),
                Item::NoConnect(marker) => self.no_connects.push(Obstruction {
                    handle: marker.uuid.short().to_owned(),
                    area: Rect::around(marker.at),
                }),
                _ => {}
            }
        }
    }

    /// The lists, as the search borrows them.
    #[must_use]
    pub fn geometry(&self) -> SheetGeometry<'_> {
        SheetGeometry {
            symbol_bodies: &self.symbol_bodies,
            sheet_bodies: &self.sheet_bodies,
            pins: &self.pins,
            junctions: &self.junctions,
            no_connects: &self.no_connects,
            segments: &self.segments,
            texts: &self.texts,
        }
    }

    /// The page the sheet is drawn on, which the window is clipped to.
    #[must_use]
    pub fn page(&self) -> Rect {
        self.page
    }
}

/// The page a schematic file draws on.
///
/// The origin is the top-left corner of the paper, which is where a schematic
/// coordinate is measured from. A file that names no paper draws on A4, as
/// KiCad's own default does (`common/page_info.cpp:96-100`).
///
/// A paper name KiCad does not know is a file KiCad refuses to open. This
/// answers A4 for one rather than refusing, because the page is a boundary on
/// the search and not a measurement anybody reads.
///
/// The border and the title block KiCad draws inside the paper are not taken
/// off. A wire drawn over them is legal in the file, and the window is clipped
/// to the paper, which is the limit `research/wire-routing.md` §3.1 states.
#[must_use]
pub fn page_area(doc: &Doc) -> Rect {
    let Some(paper) = doc.root().and_then(|root| child_named(doc, root, "paper")) else {
        return page_of(DEFAULT_PAGE.1, DEFAULT_PAGE.2);
    };
    let values = doc.children(paper);
    let name = values.get(1).and_then(|&id| doc.atom_as_str(id));
    // `User` carries its own size, in millimetres, in the two atoms after the
    // name (`sch_io_kicad_sexpr_parser.cpp:2141-2159`).
    if name.as_deref() == Some("User") {
        let read = |index: usize| values.get(index).and_then(|&id| doc.atom_as_iu(id));
        if let (Some(width), Some(height)) = (read(2), read(3)) {
            return Rect::new(
                Point::default(),
                Point::new(clamp_page(width), clamp_page(height)),
            );
        }
    }
    let (width, height) = name
        .and_then(|name| {
            PAGE_SIZES
                .iter()
                .find(|(known, _, _)| *known == name)
                .map(|&(_, width, height)| (width, height))
        })
        .unwrap_or((DEFAULT_PAGE.1, DEFAULT_PAGE.2));
    // A portrait page is the same page turned, which swaps the two sides
    // (`PAGE_INFO::SetPortrait`, `common/page_info.cpp:166-178`).
    if values
        .iter()
        .any(|&id| doc.atom_text(id) == Some("portrait"))
    {
        return page_of(height, width);
    }
    page_of(width, height)
}

/// The page rectangle of a size in mils.
fn page_of(width: i32, height: i32) -> Rect {
    Rect::new(
        Point::default(),
        Point::new(width * IU_PER_MIL, height * IU_PER_MIL),
    )
}

/// A custom page side, held inside the limits the parser holds it to.
fn clamp_page(side: i32) -> i32 {
    side.clamp(PAGE_LIMITS.0, PAGE_LIMITS.1)
}

/// The child list of a node with this head.
fn child_named(doc: &Doc, node: NodeId, head: &str) -> Option<NodeId> {
    doc.children(node)
        .iter()
        .copied()
        .find(|&child| doc.head_is(child, head))
}

#[cfg(test)]
mod tests {
    use super::page_area;
    use crate::geometry::{Iu, Point};
    use kicli_sexpr::Doc;

    /// The smallest file that carries a paper name.
    fn sheet(paper: &str) -> String {
        format!("(kicad_sch (version 20260306) {paper})")
    }

    #[test]
    fn a_page_is_the_paper_kicad_names() {
        let doc = Doc::parse(&sheet("(paper \"A4\")")).expect("the file parses");
        let page = page_area(&doc);
        assert_eq!(page.start(), Point::default());
        // 11693 × 8268 mils, which is KiCad's own rounding of 297 × 210 mm.
        assert_eq!(page.end(), Point::new(2_970_022, 2_100_072));

        let a3 = Doc::parse(&sheet("(paper \"A3\")")).expect("the file parses");
        assert_eq!(page_area(&a3).end(), Point::new(4_199_890, 2_970_022));

        // A portrait page is the same page turned.
        let portrait = Doc::parse(&sheet("(paper \"A4\" portrait)")).expect("the file parses");
        assert_eq!(page_area(&portrait).end(), Point::new(2_100_072, 2_970_022));
    }

    #[test]
    fn a_file_with_no_paper_draws_on_a4() {
        let doc = Doc::parse("(kicad_sch (version 20260306))").expect("the file parses");
        assert_eq!(page_area(&doc).end(), Point::new(2_970_022, 2_100_072));
        // So does one naming a paper KiCad has never heard of.
        let unknown = Doc::parse(&sheet("(paper \"A9\")")).expect("the file parses");
        assert_eq!(page_area(&unknown).end(), Point::new(2_970_022, 2_100_072));
    }

    #[test]
    fn a_custom_page_carries_its_own_size() {
        let doc = Doc::parse(&sheet("(paper \"User\" 200 150)")).expect("the file parses");
        assert_eq!(page_area(&doc).end(), Point::new(2_000_000, 1_500_000));

        // A size outside the limits the parser holds a page to is held to them.
        // The smallest page a schematic may carry is 25.4 mm on a side.
        let tiny = Doc::parse(&sheet("(paper \"User\" 1 1)")).expect("the file parses");
        assert_eq!(page_area(&tiny).end(), Point::new(254_000, 254_000));
        assert_eq!(Iu(254_000), Iu(20 * crate::geometry::GRID.0));
    }
}
