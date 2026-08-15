//! Building the smallest schematic that shows one rule.
//!
//! A probe is built item by item and written as text, because the question a
//! probe asks is usually about a shape the typed model cannot yet hold. The
//! builder writes what KiCad writes: the same record order, the same field
//! set, and coordinates in the precision KiCad uses.
//!
//! Every helper here panics rather than returning an error. A probe that
//! cannot be built is a broken instrument, and a test standing on a broken
//! instrument must stop rather than report a finding.

use std::path::{Path, PathBuf};

/// The root sheet uuid every probe uses.
const ROOT: &str = "00000000-0000-4000-8000-999999999999";

/// The uuid of the sheet symbol a probe with one child sheet draws.
const CHILD: &str = "00000000-0000-4000-8000-cccccccccccc";

/// One probe drawing, built item by item.
pub struct Probe {
    name: String,
    /// Where the probe's files go. A probe writes into a sub-directory of this
    /// named for itself.
    ///
    /// The caller supplies it because `CARGO_TARGET_TMPDIR` exists for test and
    /// bench targets only, and a library cannot read it.
    directory: PathBuf,
    /// The file name, without the extension.
    file: &'static str,
    /// Where this file is placed, and what each placement calls its symbols.
    ///
    /// A sheet placed twice gives every symbol on it two instance records, one
    /// per sheet path, with a different reference designator each time. The
    /// suffix distinguishes them, so `R1` on the second placement is `R1b`.
    paths: Vec<(String, &'static str)>,
    /// The uuid prefix, so a child's uuids differ from its parent's.
    series: u32,
    /// The uuid of the sheet this file is.
    sheet_uuid: String,
    symbols: Vec<String>,
    items: Vec<String>,
    next_uuid: u32,
}

impl Probe {
    /// A probe named for the question it asks, writing under a directory.
    #[must_use]
    pub fn new(name: &str, directory: PathBuf) -> Self {
        Self {
            name: name.to_owned(),
            directory,
            file: "probe",
            paths: vec![(format!("/{ROOT}"), "")],
            series: 1,
            sheet_uuid: ROOT.to_owned(),
            symbols: vec![resistor()],
            items: Vec::new(),
            next_uuid: 0,
        }
    }

    /// A probe for the child sheet this one draws.
    #[must_use]
    pub fn child_of(parent: &Probe) -> Self {
        Self::named_child_of(parent, "child", CHILD, 2)
    }

    /// The same, for a probe that draws more than one child.
    #[must_use]
    pub fn named_child_of(parent: &Probe, file: &'static str, uuid: &str, series: u32) -> Self {
        Self {
            name: parent.name.clone(),
            directory: parent.directory.clone(),
            file,
            paths: vec![(format!("/{ROOT}/{uuid}"), "")],
            series,
            sheet_uuid: uuid.to_owned(),
            symbols: vec![resistor()],
            items: Vec::new(),
            next_uuid: 0,
        }
    }

    /// Place this file again, under a second sheet symbol of the parent.
    ///
    /// The suffix names the second placement's symbols, so one drawing carries
    /// `R1` and `R1b`. Call it before placing any symbol.
    pub fn also_placed_at(&mut self, uuid: &str, suffix: &'static str) {
        self.paths.push((format!("/{ROOT}/{uuid}"), suffix));
    }

    /// A fresh uuid. The counter makes every probe file reproducible.
    fn uuid(&mut self) -> String {
        self.next_uuid += 1;
        format!(
            "00000000-0000-4000-800{}-{:012}",
            self.series, self.next_uuid
        )
    }

    /// Draw the sheet symbol that places the child, with one port on it.
    pub fn sheet(&mut self, port: &str, at: (&str, &str)) {
        self.sheet_named(CHILD, "child", port, at, "0");
    }

    /// The same, for a named child, with the port on the edge the angle says.
    ///
    /// The angle is which way the port points: 0 puts it on the right edge of
    /// the sheet symbol, 180 on the left. KiCad moves a port whose angle
    /// disagrees with its position, which takes it off the wire that was drawn
    /// to meet it, so a probe that gets the angle wrong measures a drawing it
    /// did not intend.
    pub fn sheet_named(
        &mut self,
        uuid: &str,
        name: &str,
        port: &str,
        at: (&str, &str),
        angle: &str,
    ) {
        self.sheet_of_size(
            uuid,
            name,
            at,
            ("25.4", "25.4"),
            &[Port {
                name: port,
                at,
                angle,
            }],
        );
    }

    /// A sheet symbol of a stated size, carrying the ports the caller names.
    ///
    /// The corner, the size and each port position are given separately,
    /// because a probe that asks where KiCad puts a port must be free to write
    /// the port somewhere other than the corner. A probe that measures the
    /// port rule needs a body with four distinguishable edges, which one fixed
    /// size and one port cannot give it.
    pub fn sheet_of_size(
        &mut self,
        uuid: &str,
        name: &str,
        at: (&str, &str),
        size: (&str, &str),
        ports: &[Port<'_>],
    ) {
        let pins: Vec<String> = ports
            .iter()
            .map(|port| {
                let pin_uuid = self.uuid();
                let justify = if port.angle == "0" { "right" } else { "left" };
                format!(
                    "(pin \"{}\" bidirectional (at {} {} {})\n\
                     (effects (font (size 1.27 1.27)) (justify {justify})) (uuid \"{pin_uuid}\"))",
                    port.name, port.at.0, port.at.1, port.angle
                )
            })
            .collect();
        self.items.push(format!(
            "(sheet (at {} {}) (size {} {})\n\
             (exclude_from_sim no) (in_bom yes) (on_board yes) (dnp no)\n\
             (stroke (width 0) (type solid)) (fill (color 0 0 0 0.0000))\n\
             (uuid \"{uuid}\")\n\
             (property \"Sheetname\" \"{name}\" (at {} {} 0)\n\
             (effects (font (size 1.27 1.27)) (justify left bottom)))\n\
             (property \"Sheetfile\" \"{name}.kicad_sch\" (at {} {} 0)\n\
             (effects (font (size 1.27 1.27)) (justify left top)))\n\
             {}\n\
             (instances (project \"probe\" (path \"/{ROOT}\" (page \"2\"))))\n)",
            at.0,
            at.1,
            size.0,
            size.1,
            at.0,
            at.1,
            at.0,
            at.1,
            pins.join("\n")
        ));
    }

    /// Add a library symbol the probe places.
    pub fn define(&mut self, symbol: String) -> &mut Self {
        self.symbols.push(symbol);
        self
    }

    /// Place a symbol, with the pin numbers it draws.
    pub fn place(&mut self, library: &str, reference: &str, at: (&str, &str), pins: &[&str]) {
        self.place_symbol(&Placed::new(library, reference, at, pins));
    }

    /// Place one unit of a symbol, with a value of its own.
    pub fn place_unit(
        &mut self,
        library: &str,
        reference: &str,
        at: (&str, &str),
        unit: u32,
        value: &str,
        pins: &[&str],
    ) {
        let mut placed = Placed::new(library, reference, at, pins);
        placed.unit = unit;
        placed.value = value;
        self.place_symbol(&placed);
    }

    /// Place a symbol as described.
    pub fn place_symbol(&mut self, placed: &Placed) {
        let uuid = self.uuid();
        let pin_uuids: Vec<String> = placed.pins.iter().map(|_| self.uuid()).collect();
        let (library, reference, unit) = (placed.library, placed.reference, placed.unit);
        let angle = placed.angle;
        let mirror = placed
            .mirror
            .map_or_else(String::new, |axis| format!(" (mirror {axis})"));
        let instance_unit = placed.instance_unit.unwrap_or(unit);
        let (x, y) = placed.at;
        let attributes = placed.attributes;
        // A probe drawing holds a few dozen items, so the readable form wins
        // over the allocation clippy would save with a fold.
        #[allow(clippy::format_collect, reason = "a probe drawing is small")]
        let pin_list: String = placed
            .pins
            .iter()
            .zip(&pin_uuids)
            .map(|(number, uuid)| format!("(pin \"{number}\" (uuid \"{uuid}\"))\n"))
            .collect();
        let fields = fields(&[
            ("Reference", reference),
            ("Value", placed.value),
            ("Footprint", ""),
            ("Datasheet", ""),
            ("Description", ""),
        ]);
        #[allow(clippy::format_collect, reason = "a probe drawing is small")]
        let instances: String = self
            .paths
            .iter()
            .map(|(path, suffix)| {
                format!(
                    "(path \"{path}\" (reference \"{reference}{suffix}\") (unit {instance_unit}))"
                )
            })
            .collect();
        self.items.push(format!(
            "(symbol (lib_id \"Probe:{library}\") (at {x} {y} {angle}){mirror} (unit {unit}) (body_style 1)\n\
             {attributes}\n\
             (uuid \"{uuid}\")\n{fields}{pin_list}\
             (instances (project \"probe\" {instances}))\n)"
        ));
    }

    /// Draw a wire between two points.
    pub fn wire(&mut self, from: (&str, &str), to: (&str, &str)) {
        let uuid = self.uuid();
        self.items.push(format!(
            "(wire (pts (xy {} {}) (xy {} {})) (stroke (width 0) (type default)) (uuid \"{uuid}\"))",
            from.0, from.1, to.0, to.1
        ));
    }

    /// Draw a bundle between two points.
    pub fn bus(&mut self, from: (&str, &str), to: (&str, &str)) {
        let uuid = self.uuid();
        self.items.push(format!(
            "(bus (pts (xy {} {}) (xy {} {})) (stroke (width 0) (type default)) (uuid \"{uuid}\"))",
            from.0, from.1, to.0, to.1
        ));
    }

    /// Draw a bus entry: a stub from a wire end to a bundle.
    pub fn bus_entry(&mut self, at: (&str, &str), size: (&str, &str)) {
        let uuid = self.uuid();
        self.items.push(format!(
            "(bus_entry (at {} {}) (size {} {}) (stroke (width 0) (type default))\n\
             (uuid \"{uuid}\"))",
            at.0, at.1, size.0, size.1
        ));
    }

    /// Draw a junction, which makes a crossing a connection.
    pub fn junction(&mut self, at: (&str, &str)) {
        let uuid = self.uuid();
        self.items.push(format!(
            "(junction (at {} {}) (diameter 0) (color 0 0 0 0)\n(uuid \"{uuid}\"))",
            at.0, at.1
        ));
    }

    /// Draw a no-connect marker, which says a pin joins nothing on purpose.
    pub fn no_connect(&mut self, at: (&str, &str)) {
        let uuid = self.uuid();
        self.items.push(format!(
            "(no_connect (at {} {}) (uuid \"{uuid}\"))",
            at.0, at.1
        ));
    }

    /// Draw free text, which connects nothing and still takes room.
    pub fn free_text(&mut self, text: &str, at: (&str, &str)) {
        let uuid = self.uuid();
        self.items.push(format!(
            "(text \"{text}\" (at {} {} 0)\n\
             (effects (font (size 1.27 1.27)) (justify left bottom)) (uuid \"{uuid}\"))",
            at.0, at.1
        ));
    }

    /// Draw a label of any of the three kinds.
    pub fn label_of_kind(&mut self, head: &str, shape: &str, text: &str, at: (&str, &str)) {
        let uuid = self.uuid();
        self.items.push(format!(
            "({head} \"{text}\" {shape} (at {} {} 0)\n\
             (effects (font (size 1.27 1.27)) (justify left bottom)) (uuid \"{uuid}\"))",
            at.0, at.1
        ));
    }

    /// Draw a wire with a resistor on one end and a label on the other.
    ///
    /// This is the shape every naming probe needs: the label names the net,
    /// and the resistor pin makes the net visible in a netlist. The resistor
    /// anchor sits one pin length below the wire, so pin 1 lands on it.
    pub fn named_strand(&mut self, reference: &str, wire_y: &str, anchor_y: &str, text: &str) {
        self.strand_of_kind("label", "", reference, wire_y, anchor_y, text);
    }

    /// The same strand, named by a label of the kind asked for.
    pub fn strand_of_kind(
        &mut self,
        head: &str,
        shape: &str,
        reference: &str,
        wire_y: &str,
        anchor_y: &str,
        text: &str,
    ) {
        self.place("R", reference, ("50.8", anchor_y), &["1", "2"]);
        self.wire(("50.8", wire_y), ("76.2", wire_y));
        self.label_of_kind(head, shape, text, ("76.2", wire_y));
    }

    /// The file text.
    #[must_use]
    pub fn text(&self) -> String {
        let uuid = &self.sheet_uuid;
        let instances = if self.series == 1 {
            "(sheet_instances (path \"/\" (page \"1\")))"
        } else {
            ""
        };
        format!(
            "(kicad_sch (version 20260306) (generator \"eeschema\") (generator_version \"10.0\")\n\
             (uuid \"{uuid}\") (paper \"A4\")\n(lib_symbols\n{}\n)\n{}\n{instances}\n\
             (embedded_fonts no)\n)",
            self.symbols.join("\n"),
            self.items.join("\n")
        )
    }

    /// Every number in the drawing, as KiCad would write it.
    ///
    /// A coordinate with more than four decimals is not a KiCad coordinate.
    /// kicli's reader rejects one and an early caller read the rejection as
    /// zero, which put phantom items at the origin and joined nets that share
    /// nothing. A probe that does that measures its own defect and calls it a
    /// finding, so the drawing is checked before it is ever handed to a tool.
    fn check_precision(&self, text: &str) {
        for token in text.split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-')) {
            // Only plain decimals are coordinates. A bundle name carries a
            // range, `0..7`, which is not one and is left alone.
            let Some((whole, fraction)) = token.split_once('.') else {
                continue;
            };
            let digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
            if !digits(whole.strip_prefix('-').unwrap_or(whole)) || !digits(fraction) {
                continue;
            }
            assert!(
                fraction.len() <= 4,
                "probe {} writes {token}, which is not a number KiCad writes. \
                 Round it with millimetres().",
                self.name
            );
        }
    }

    /// Write the probe into its own directory and return its path.
    ///
    /// # Panics
    ///
    /// If the directory or the file cannot be written, or if the drawing holds
    /// a number KiCad would not write. A probe that cannot be built is a broken
    /// instrument, and a test standing on one must stop.
    #[allow(clippy::must_use_candidate, reason = "written for the file it leaves")]
    pub fn write(&self) -> PathBuf {
        let directory = self.directory.join(&self.name);
        std::fs::create_dir_all(&directory).expect("the scratch directory is writable");
        let path = directory.join(format!("{}.kicad_sch", self.file));
        let text = self.text();
        self.check_precision(&text);
        std::fs::write(&path, text).expect("the probe file is writable");
        path
    }

    /// Write this probe and its children, and return the root's path.
    ///
    /// # Panics
    ///
    /// As [`Probe::write`], for this probe or for any child.
    #[allow(clippy::must_use_candidate, reason = "written for the files it leaves")]
    pub fn write_all(&self, children: &[&Probe]) -> PathBuf {
        for child in children {
            child.write();
        }
        self.write()
    }

    /// The name this probe answers to, which names its directory too.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The directory this probe writes its files in.
    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }
}

/// One port of a sheet symbol, as a probe writes it.
///
/// The position and the angle are separate, and a probe may write them in
/// disagreement on purpose: the angle names the edge KiCad puts the port on,
/// which is not what the same number means on a symbol pin, and the way to
/// measure that is to write a port off the edge its angle names and ask the
/// tool where it ended up.
pub struct Port<'a> {
    /// The port name. A hierarchical label in the child sheet must match it.
    pub name: &'a str,
    /// Where the port is written, in millimetres.
    pub at: (&'a str, &'a str),
    /// The port angle: `0`, `90`, `180` or `270`.
    pub angle: &'a str,
}

/// One placed symbol, as a probe describes it.
pub struct Placed<'a> {
    /// The name in the probe's own library.
    pub library: &'a str,
    /// The reference designator, before any placement suffix.
    pub reference: &'a str,
    /// The anchor position, in millimetres.
    pub at: (&'a str, &'a str),
    /// The pin numbers this placement draws.
    pub pins: &'a [&'a str],
    /// The angle the placement is drawn at.
    pub angle: &'a str,
    /// The axis the placement is mirrored about, if any: `x` or `y`.
    pub mirror: Option<&'a str>,
    /// The unit written beside the `lib_id`, which is only a cache.
    pub unit: u32,
    /// The unit written in the instance record, which is the truth.
    pub instance_unit: Option<u32>,
    /// The `Value` field.
    pub value: &'a str,
    /// The attributes a symbol that is built and fitted carries.
    pub attributes: &'a str,
}

impl<'a> Placed<'a> {
    /// A placement of one unit, built and fitted, valued after its reference.
    #[must_use]
    pub fn new(
        library: &'a str,
        reference: &'a str,
        at: (&'a str, &'a str),
        pins: &'a [&'a str],
    ) -> Self {
        Self {
            library,
            reference,
            at,
            pins,
            angle: "0",
            mirror: None,
            unit: 1,
            instance_unit: None,
            value: reference,
            attributes: "(exclude_from_sim no) (in_bom yes) (on_board yes) (in_pos_files yes) (dnp no)",
        }
    }
}

/// The five fields every placed symbol carries.
#[allow(clippy::format_collect, reason = "a probe drawing is small")]
fn fields(values: &[(&str, &str)]) -> String {
    values
        .iter()
        .map(|(name, value)| {
            format!(
                "(property \"{name}\" \"{value}\" (at 0 0 0) (show_name no) (do_not_autoplace no)\n\
                 (effects (font (size 1.27 1.27))))\n"
            )
        })
        .collect()
}

/// One rectangle of a symbol's body.
///
/// A body is what a router treats as an obstacle, so a probe that measures
/// routing needs symbols that have one. The coordinates are the library's, with
/// y upwards.
#[must_use]
pub fn rectangle(start: (&str, &str), end: (&str, &str)) -> String {
    format!(
        "(rectangle (start {} {}) (end {} {})\n\
         (stroke (width 0.254) (type default)) (fill (type none)))",
        start.0, start.1, end.0, end.1
    )
}

/// One library pin.
#[must_use]
pub fn pin(electrical: &str, at: (&str, &str), angle: &str, number: &str, name: &str) -> String {
    format!(
        "(pin {electrical} line (at {} {} {angle}) (length 2.54)\n\
         (name \"{name}\" (effects (font (size 1.27 1.27))))\n\
         (number \"{number}\" (effects (font (size 1.27 1.27)))))",
        at.0, at.1
    )
}

/// One library pin the editor does not draw.
#[must_use]
pub fn hidden_pin(electrical: &str, at: (&str, &str), number: &str, name: &str) -> String {
    format!(
        "(pin {electrical} line (at {} {} 180) (length 2.54) (hide yes)\n\
         (name \"{name}\" (effects (font (size 1.27 1.27))))\n\
         (number \"{number}\" (effects (font (size 1.27 1.27)))))",
        at.0, at.1
    )
}

/// A power symbol: one power input at the anchor, and its value names the net.
#[must_use]
pub fn power(name: &str) -> String {
    symbol(
        name,
        "#PWR",
        true,
        &[("1_1", vec![pin("power_in", ("0", "0"), "270", "1", "")])],
    )
}

/// One library symbol, from its units.
#[must_use]
pub fn symbol(name: &str, reference: &str, power: bool, units: &[(&str, Vec<String>)]) -> String {
    #[allow(clippy::format_collect, reason = "a probe drawing is small")]
    let bodies: String = units
        .iter()
        .map(|(unit, pins)| format!("(symbol \"{name}_{unit}\"\n{}\n)\n", pins.join("\n")))
        .collect();
    let power = if power { "(power global)" } else { "" };
    format!(
        "(symbol \"Probe:{name}\" {power} (pin_names (offset 0))\n\
         (exclude_from_sim no) (in_bom yes) (on_board yes) (in_pos_files yes)\n\
         (duplicate_pin_numbers_are_jumpers no)\n{}{bodies})",
        fields(&[
            ("Reference", reference),
            ("Value", name),
            ("Footprint", ""),
            ("Datasheet", ""),
            ("Description", ""),
        ])
    )
}

/// A resistor: pin 1 above the anchor, pin 2 below it.
#[must_use]
pub fn resistor() -> String {
    symbol(
        "R",
        "R",
        false,
        &[(
            "1_1",
            vec![
                pin("passive", ("0", "3.81"), "270", "1", ""),
                pin("passive", ("0", "-3.81"), "90", "2", ""),
            ],
        )],
    )
}

/// A coordinate in millimetres, written the way KiCad writes one.
///
/// KiCad's own files carry at most four decimals, and kicli's reader rejects
/// more. A probe that computes a position in floating point must round it, or
/// `38.1 + 12.7 * 3.0` reaches the file as `76.19999999999999` and describes a
/// drawing nobody meant.
#[must_use]
pub fn millimetres(value: f64) -> String {
    let text = format!("{value:.4}");
    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
    trimmed.to_owned()
}
