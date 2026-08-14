//! Each merge rule, on the smallest drawing that shows it.
//!
//! A rule here was measured before it was implemented: the drawing was built,
//! `kicad-cli sch export netlist` was asked what it joins, and the answer is
//! the expectation below. The drawing is built by this file rather than
//! committed, so the rule, its evidence and its drawing stay in one place.
//!
//! The default run compares kicli against those recorded answers. With
//! `KICLI_TEST_KICAD_CLI` set, every probe is exported by `kicad-cli` as well
//! and the recorded answer is checked against the tool, so a stale expectation
//! is caught rather than trusted.

use kicli::connectivity::{NetPin, Nets, extract};
use kicli::model::Hierarchy;
use kicli_sexpr::Doc;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// A net partition: one sorted pin list per net.
type Partition = BTreeSet<Vec<String>>;

/// The root sheet uuid every probe uses.
const ROOT: &str = "00000000-0000-4000-8000-999999999999";

/// The uuid of the sheet symbol a probe with a child sheet draws.
const CHILD: &str = "00000000-0000-4000-8000-cccccccccccc";

/// One probe drawing, built item by item.
struct Probe {
    name: &'static str,
    /// The file name, without the extension.
    file: &'static str,
    /// The sheet path the symbols of this file are placed on.
    path: String,
    /// The uuid prefix, so a child's uuids differ from its parent's.
    series: u32,
    /// The uuid of the sheet this file is.
    sheet_uuid: String,
    symbols: Vec<String>,
    items: Vec<String>,
    next_uuid: u32,
}

impl Probe {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            file: "probe",
            path: format!("/{ROOT}"),
            series: 1,
            sheet_uuid: ROOT.to_owned(),
            symbols: vec![resistor()],
            items: Vec::new(),
            next_uuid: 0,
        }
    }

    /// A probe for the child sheet this one draws.
    fn child_of(parent: &Probe) -> Self {
        Self::named_child_of(parent, "child", CHILD, 2)
    }

    /// The same, for a probe that draws more than one child.
    fn named_child_of(parent: &Probe, file: &'static str, uuid: &str, series: u32) -> Self {
        Self {
            name: parent.name,
            file,
            path: format!("/{ROOT}/{uuid}"),
            series,
            sheet_uuid: uuid.to_owned(),
            symbols: vec![resistor()],
            items: Vec::new(),
            next_uuid: 0,
        }
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
    fn sheet(&mut self, port: &str, at: (&str, &str)) {
        self.sheet_named(CHILD, "child", port, at, "0");
    }

    /// The same, for a named child, with the port on the edge the angle says.
    ///
    /// The angle is which way the port points: 0 puts it on the right edge of
    /// the sheet symbol, 180 on the left. KiCad moves a port whose angle
    /// disagrees with its position, which takes it off the wire that was drawn
    /// to meet it, so a probe that gets the angle wrong measures a drawing it
    /// did not intend.
    fn sheet_named(
        &mut self,
        uuid: &str,
        name: &str,
        port: &str,
        at: (&str, &str),
        angle: &str,
    ) -> String {
        let pin_uuid = self.uuid();
        let justify = if angle == "0" { "right" } else { "left" };
        self.items.push(format!(
            "(sheet (at {} {}) (size 25.4 25.4)\n\
             (exclude_from_sim no) (in_bom yes) (on_board yes) (dnp no)\n\
             (stroke (width 0) (type solid)) (fill (color 0 0 0 0.0000))\n\
             (uuid \"{uuid}\")\n\
             (property \"Sheetname\" \"{name}\" (at {} {} 0)\n\
             (effects (font (size 1.27 1.27)) (justify left bottom)))\n\
             (property \"Sheetfile\" \"{name}.kicad_sch\" (at {} {} 0)\n\
             (effects (font (size 1.27 1.27)) (justify left top)))\n\
             (pin \"{port}\" bidirectional (at {} {} {angle})\n\
             (effects (font (size 1.27 1.27)) (justify {justify})) (uuid \"{pin_uuid}\"))\n\
             (instances (project \"probe\" (path \"/{ROOT}\" (page \"2\"))))\n)",
            at.0, at.1, at.0, at.1, at.0, at.1, at.0, at.1
        ));
        uuid.to_owned()
    }

    /// Add a library symbol the probe places.
    fn define(&mut self, symbol: String) -> &mut Self {
        self.symbols.push(symbol);
        self
    }

    /// Place a symbol, with the pin numbers it draws.
    fn place(&mut self, library: &str, reference: &str, at: (&str, &str), pins: &[&str]) {
        self.place_symbol(&Placed::new(library, reference, at, pins));
    }

    /// Place one unit of a symbol, with a value of its own.
    fn place_unit(
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
    fn place_symbol(&mut self, placed: &Placed) {
        let uuid = self.uuid();
        let pin_uuids: Vec<String> = placed.pins.iter().map(|_| self.uuid()).collect();
        let (library, reference, unit) = (placed.library, placed.reference, placed.unit);
        let instance_unit = placed.instance_unit.unwrap_or(unit);
        let (x, y) = placed.at;
        let attributes = placed.attributes;
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
        self.items.push(format!(
            "(symbol (lib_id \"Probe:{library}\") (at {x} {y} 0) (unit {unit}) (body_style 1)\n\
             {attributes}\n\
             (uuid \"{uuid}\")\n{fields}{pin_list}\
             (instances (project \"probe\" (path \"{}\" (reference \"{reference}\") (unit {instance_unit}))))\n)",
            self.path
        ));
    }

    /// Draw a wire between two points.
    fn wire(&mut self, from: (&str, &str), to: (&str, &str)) {
        let uuid = self.uuid();
        self.items.push(format!(
            "(wire (pts (xy {} {}) (xy {} {})) (stroke (width 0) (type default)) (uuid \"{uuid}\"))",
            from.0, from.1, to.0, to.1
        ));
    }

    /// Draw a bundle between two points.
    fn bus(&mut self, from: (&str, &str), to: (&str, &str)) {
        let uuid = self.uuid();
        self.items.push(format!(
            "(bus (pts (xy {} {}) (xy {} {})) (stroke (width 0) (type default)) (uuid \"{uuid}\"))",
            from.0, from.1, to.0, to.1
        ));
    }

    /// Draw a bus entry: a stub from a wire end to a bundle.
    fn bus_entry(&mut self, at: (&str, &str), size: (&str, &str)) {
        let uuid = self.uuid();
        self.items.push(format!(
            "(bus_entry (at {} {}) (size {} {}) (stroke (width 0) (type default))\n\
             (uuid \"{uuid}\"))",
            at.0, at.1, size.0, size.1
        ));
    }

    /// Draw a label of any of the three kinds.
    fn label_of_kind(&mut self, head: &str, shape: &str, text: &str, at: (&str, &str)) {
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
    fn named_strand(&mut self, reference: &str, wire_y: &str, anchor_y: &str, text: &str) {
        self.strand_of_kind("label", "", reference, wire_y, anchor_y, text);
    }

    /// The same strand, named by a label of the kind asked for.
    fn strand_of_kind(
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
    fn text(&self) -> String {
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

    /// Write the probe to the scratch directory and return its path.
    fn write(&self) -> PathBuf {
        let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("net-probes")
            .join(self.name);
        std::fs::create_dir_all(&directory).expect("the scratch directory is writable");
        let path = directory.join(format!("{}.kicad_sch", self.file));
        std::fs::write(&path, self.text()).expect("the probe file is writable");
        path
    }

    /// kicli's partition of the probe, checked against KiCad when it is there.
    fn partition(&self) -> Partition {
        self.partition_with(&[])
    }

    /// Write this probe and its children, and return the root's path.
    fn write_all(&self, children: &[&Probe]) -> PathBuf {
        for child in children {
            child.write();
        }
        self.write()
    }

    /// The same, of a probe that draws child sheets.
    fn partition_with(&self, children: &[&Probe]) -> Partition {
        let path = self.write_all(children);
        let hierarchy = Hierarchy::load(&path).expect("the probe loads");
        let found = partition_of(&extract(&hierarchy));
        if let Some(tool) = kicad_cli() {
            let netlist = export_netlist(&tool, &path).expect("kicad-cli exported a netlist");
            assert_eq!(
                found,
                kicad_partition(&netlist),
                "kicli and KiCad disagree about {}",
                self.name
            );
        }
        found
    }
}

/// One placed symbol, as a probe describes it.
struct Placed<'a> {
    library: &'a str,
    reference: &'a str,
    at: (&'a str, &'a str),
    pins: &'a [&'a str],
    /// The unit written beside the `lib_id`, which is only a cache.
    unit: u32,
    /// The unit written in the instance record, which is the truth.
    instance_unit: Option<u32>,
    value: &'a str,
    /// The attributes a symbol that is built and fitted carries.
    attributes: &'a str,
}

impl<'a> Placed<'a> {
    fn new(
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
            unit: 1,
            instance_unit: None,
            value: reference,
            attributes: "(exclude_from_sim no) (in_bom yes) (on_board yes) (in_pos_files yes) (dnp no)",
        }
    }
}

/// The five fields every placed symbol carries.
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

/// One library pin.
fn pin(electrical: &str, at: (&str, &str), angle: &str, number: &str, name: &str) -> String {
    format!(
        "(pin {electrical} line (at {} {} {angle}) (length 2.54)\n\
         (name \"{name}\" (effects (font (size 1.27 1.27))))\n\
         (number \"{number}\" (effects (font (size 1.27 1.27)))))",
        at.0, at.1
    )
}

/// One library pin the editor does not draw.
fn hidden_pin(electrical: &str, at: (&str, &str), number: &str, name: &str) -> String {
    format!(
        "(pin {electrical} line (at {} {} 180) (length 2.54) (hide yes)\n\
         (name \"{name}\" (effects (font (size 1.27 1.27))))\n\
         (number \"{number}\" (effects (font (size 1.27 1.27)))))",
        at.0, at.1
    )
}

/// A power symbol: one power input at the anchor, and its value names the net.
fn power(name: &str) -> String {
    symbol(
        name,
        "#PWR",
        true,
        &[("1_1", vec![pin("power_in", ("0", "0"), "270", "1", "")])],
    )
}

/// One library symbol, from its units.
fn symbol(name: &str, reference: &str, power: bool, units: &[(&str, Vec<String>)]) -> String {
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
fn resistor() -> String {
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

/// The partition kicli reads, the way a netlist reports it.
fn partition_of(nets: &Nets) -> Partition {
    nets.nets()
        .iter()
        .map(|net| {
            net.pins
                .iter()
                .filter(|pin| !pin.power && pin.on_board)
                .map(NetPin::label)
                .collect::<Vec<String>>()
        })
        .filter(|pins| !pins.is_empty())
        .collect()
}

/// One expected net.
fn net(pins: &[&str]) -> Vec<String> {
    let mut sorted: Vec<String> = pins.iter().map(|pin| (*pin).to_owned()).collect();
    sorted.sort();
    sorted
}

/// The `kicad-cli` to run, or nothing when the caller did not ask for it.
fn kicad_cli() -> Option<String> {
    std::env::var("KICLI_TEST_KICAD_CLI").ok()?;
    Some(std::env::var("KICLI_KICAD_CLI").unwrap_or_else(|_| "kicad-cli".to_owned()))
}

/// Export a netlist of a probe and read it back.
///
/// The tool's own output is dropped: the first run on a machine prints
/// fontconfig warnings that say nothing about the netlist.
fn export_netlist(tool: &str, probe: &Path) -> Option<String> {
    let into = probe.with_extension("net");
    let status = Command::new(tool)
        .args(["sch", "export", "netlist", "-o"])
        .arg(&into)
        .arg(probe)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()?;
    if !status.success() {
        return None;
    }
    std::fs::read_to_string(&into).ok()
}

/// The partition KiCad reports, read out of a netlist it wrote.
fn kicad_partition(text: &str) -> Partition {
    let doc = Doc::parse(text).expect("the netlist parses");
    let root = doc.root().expect("the netlist has a root list");
    let mut found = Partition::new();
    for &child in doc.children(root) {
        if !doc.head_is(child, "nets") {
            continue;
        }
        for &net in doc.children(child) {
            if !doc.head_is(net, "net") {
                continue;
            }
            let mut pins: Vec<String> = doc
                .children(net)
                .iter()
                .filter(|&&node| doc.head_is(node, "node"))
                .filter_map(|&node| node_label(&doc, node))
                .collect();
            pins.sort();
            if !pins.is_empty() {
                found.insert(pins);
            }
        }
    }
    found
}

/// One `(node (ref "R1") (pin "2") ...)` as `R1.2`.
fn node_label(doc: &Doc, node: kicli_sexpr::NodeId) -> Option<String> {
    let value = |head: &str| -> Option<String> {
        let list = doc
            .children(node)
            .iter()
            .copied()
            .find(|&child| doc.head_is(child, head))?;
        doc.children(list)
            .get(1)
            .and_then(|&id| doc.atom_as_str(id))
    };
    Some(format!("{}.{}", value("ref")?, value("pin")?))
}

/// A part with a pin of unit 0, which every unit draws.
fn dual_unit() -> String {
    symbol(
        "DUAL",
        "U",
        false,
        &[
            ("0_1", vec![pin("passive", ("7.62", "0"), "180", "9", "S")]),
            (
                "1_1",
                vec![
                    pin("passive", ("0", "3.81"), "270", "1", "A"),
                    pin("passive", ("0", "-3.81"), "90", "2", "B"),
                ],
            ),
            (
                "2_1",
                vec![
                    pin("passive", ("0", "3.81"), "270", "3", "C"),
                    pin("passive", ("0", "-3.81"), "90", "4", "D"),
                ],
            ),
        ],
    )
}

#[test]
fn a_pin_shared_by_two_units_is_listed_once_per_net() {
    let mut probe = Probe::new("shared-unit-pin");
    probe.define(dual_unit());

    // Both units of U1 draw pin 9. One wire joins the two copies.
    probe.place_unit("DUAL", "U1", ("50.8", "50.8"), 1, "U1", &["1", "2", "9"]);
    probe.place_unit("DUAL", "U1", ("50.8", "76.2"), 2, "U1", &["3", "4", "9"]);
    probe.wire(("58.42", "50.8"), ("88.9", "50.8"));
    probe.wire(("58.42", "76.2"), ("88.9", "76.2"));
    probe.wire(("88.9", "50.8"), ("88.9", "76.2"));
    probe.place("R", "R1", ("88.9", "54.61"), &["1", "2"]);

    // Both units of U2 draw pin 9 as well, onto two nets of their own.
    probe.place_unit("DUAL", "U2", ("50.8", "114.3"), 1, "U2", &["1", "2", "9"]);
    probe.place_unit("DUAL", "U2", ("50.8", "139.7"), 2, "U2", &["3", "4", "9"]);
    probe.wire(("58.42", "114.3"), ("88.9", "114.3"));
    probe.wire(("58.42", "139.7"), ("88.9", "139.7"));
    probe.place("R", "R2", ("88.9", "118.11"), &["1", "2"]);
    probe.place("R", "R3", ("88.9", "143.51"), &["1", "2"]);

    let found = probe.partition();
    // One net, one entry for U1 pin 9, though two pins reach it.
    assert!(found.contains(&net(&["R1.1", "U1.9"])));
    // The rule is per net: wired apart, the pin number is on both nets.
    assert!(found.contains(&net(&["R2.1", "U2.9"])));
    assert!(found.contains(&net(&["R3.1", "U2.9"])));
}

#[test]
fn two_labels_join_when_their_net_names_are_equal() {
    let mut probe = Probe::new("escaped-label-names");

    // A slash may not stand in a net name, so KiCad writes it `{slash}`.
    // The raw form and the written form are one name.
    probe.named_strand("R1", "25.4", "29.21", "AA/BB");
    probe.named_strand("R2", "38.1", "41.91", "AA{slash}BB");
    // The escape is not a general normalisation: these two stay apart.
    probe.named_strand("R3", "50.8", "54.61", "CC-DD");
    probe.named_strand("R4", "63.5", "67.31", "CC_DD");
    // A name that holds an escape word is one net and not a bundle, so it
    // joins an ordinary net rather than refusing to.
    probe.named_strand("R5", "76.2", "80.01", "EE/FF");
    probe.named_strand("R6", "88.9", "92.71", "EE/FF");

    let found = probe.partition();
    assert!(found.contains(&net(&["R1.1", "R2.1"])));
    assert!(found.contains(&net(&["R3.1"])));
    assert!(found.contains(&net(&["R4.1"])));
    assert!(found.contains(&net(&["R5.1", "R6.1"])));
}

#[test]
fn a_bundle_carries_its_members_to_every_sheet_it_reaches() {
    let mut probe = Probe::new("bundle-members");
    let mut child = Probe::child_of(&probe);

    // The bundle leaves the root sheet through the port of the child.
    probe.sheet("AN[0..7]", ("101.6", "50.8"));
    probe.bus(("101.6", "50.8"), ("152.4", "50.8"));
    probe.label_of_kind("label", "", "AN[0..7]", ("152.4", "50.8"));
    // A member of it, and a name the bundle does not carry.
    probe.named_strand("R1", "25.4", "29.21", "AN0");
    probe.named_strand("R5", "88.9", "92.71", "ZZ9");

    child.label_of_kind(
        "hierarchical_label",
        "(shape input)",
        "AN[0..7]",
        ("101.6", "50.8"),
    );
    child.named_strand("R2", "25.4", "29.21", "AN0");
    child.named_strand("R6", "88.9", "92.71", "ZZ9");

    let found = probe.partition_with(&[&child]);
    // The two sheets' AN0 are one net, though no wire joins them.
    assert!(found.contains(&net(&["R1.1", "R2.1"])));
    // A name the bundle does not carry stays local to its sheet.
    assert!(found.contains(&net(&["R5.1"])));
    assert!(found.contains(&net(&["R6.1"])));
}

/// Draw a bundle down the sheet, name it, and hang member nets off it.
///
/// This is the shape a child sheet of the bundle-linking probe needs: a
/// hierarchical label puts the bundle on the port above, and each member is a
/// named wire joined to the bundle by a bus entry.
fn bundled_members(probe: &mut Probe, bundle: &str, members: &[(&str, &str)]) {
    bundled_members_named(
        probe,
        "hierarchical_label",
        "(shape input)",
        bundle,
        members,
    );
}

/// The same, with the bundle named by a label of the kind asked for.
fn bundled_members_named(
    probe: &mut Probe,
    head: &str,
    shape: &str,
    bundle: &str,
    members: &[(&str, &str)],
) {
    probe.bus(("127", "20.32"), ("127", "101.6"));
    probe.label_of_kind(head, shape, bundle, ("127", "20.32"));
    for (index, (reference, member)) in members.iter().enumerate() {
        let wire_y = format!("{}", 38.1 + 12.7 * index as f64);
        let anchor_y = format!("{}", 41.91 + 12.7 * index as f64);
        probe.bus_entry(("124.46", &wire_y), ("2.54", "2.54"));
        probe.wire(("99.06", &wire_y), ("124.46", &wire_y));
        probe.label_of_kind("label", "", member, ("101.6", &wire_y));
        probe.place("R", reference, ("99.06", &anchor_y), &["1", "2"]);
    }
}

#[test]
fn two_bundles_on_one_bus_join_their_corresponding_members() {
    let mut probe = Probe::new("linked-bundles");
    let first = "00000000-0000-4000-8000-cccccccccc01";
    let second = "00000000-0000-4000-8000-cccccccccc02";
    let mut left = Probe::named_child_of(&probe, "child1", first, 2);
    let mut right = Probe::named_child_of(&probe, "child2", second, 3);

    // One bus joins the ports of two children, and the two carry bundles of
    // different names. The port angle puts each pin on the edge the bus meets.
    probe.sheet_named(first, "child1", "UART{TX, RX, CTS}", ("101.6", "50.8"), "0");
    probe.sheet_named(
        second,
        "child2",
        "UART_TRG{TX, RX}",
        ("152.4", "50.8"),
        "180",
    );
    probe.bus(("101.6", "50.8"), ("152.4", "50.8"));

    bundled_members(
        &mut left,
        "UART{TX, RX, CTS}",
        &[("R1", "UART.TX"), ("R2", "UART.RX"), ("R3", "UART.CTS")],
    );
    bundled_members(
        &mut right,
        "UART_TRG{TX, RX}",
        &[("R4", "UART_TRG.TX"), ("R5", "UART_TRG.RX")],
    );

    let found = probe.partition_with(&[&left, &right]);
    // A group member corresponds by its own name, so TX joins TX and RX joins
    // RX, though the two bundles are named differently and no wire joins the
    // member nets.
    assert!(found.contains(&net(&["R1.1", "R4.1"])));
    assert!(found.contains(&net(&["R2.1", "R5.1"])));
    // A member the other bundle does not carry stays on its own net.
    assert!(found.contains(&net(&["R3.1"])));
}

#[test]
fn two_vector_bundles_on_one_bus_join_by_place_and_not_by_index() {
    let mut probe = Probe::new("linked-vectors");
    let first = "00000000-0000-4000-8000-cccccccccc01";
    let second = "00000000-0000-4000-8000-cccccccccc02";
    let mut left = Probe::named_child_of(&probe, "child1", first, 2);
    let mut right = Probe::named_child_of(&probe, "child2", second, 3);

    // The two ranges start at different numbers on purpose.
    probe.sheet_named(first, "child1", "AA[0..2]", ("101.6", "50.8"), "0");
    probe.sheet_named(second, "child2", "BB[5..6]", ("152.4", "50.8"), "180");
    probe.bus(("101.6", "50.8"), ("152.4", "50.8"));

    bundled_members(
        &mut left,
        "AA[0..2]",
        &[("R1", "AA0"), ("R2", "AA1"), ("R3", "AA2")],
    );
    bundled_members(&mut right, "BB[5..6]", &[("R4", "BB5"), ("R5", "BB6")]);

    let found = probe.partition_with(&[&left, &right]);
    // The first member of one range joins the first of the other, so the
    // number written in the name is not what corresponds.
    assert!(found.contains(&net(&["R1.1", "R4.1"])));
    assert!(found.contains(&net(&["R2.1", "R5.1"])));
    assert!(found.contains(&net(&["R3.1"])));
}

#[test]
fn a_vector_against_a_group_is_reported_and_not_reproduced() {
    let mut probe = Probe::new("mixed-bundle-kinds");
    let first = "00000000-0000-4000-8000-cccccccccc01";
    let second = "00000000-0000-4000-8000-cccccccccc02";
    let mut left = Probe::named_child_of(&probe, "child1", first, 2);
    let mut right = Probe::named_child_of(&probe, "child2", second, 3);

    // One bus, carrying a vector bundle and a group bundle at once.
    probe.sheet_named(first, "child1", "AA[0..1]", ("101.6", "50.8"), "0");
    probe.sheet_named(second, "child2", "BB{P, Q}", ("152.4", "50.8"), "180");
    probe.bus(("101.6", "50.8"), ("152.4", "50.8"));

    bundled_members(&mut left, "AA[0..1]", &[("R1", "AA0"), ("R2", "AA1")]);
    bundled_members(&mut right, "BB{P, Q}", &[("R3", "BB.P"), ("R4", "BB.Q")]);

    let path = probe.write_all(&[&left, &right]);
    let hierarchy = Hierarchy::load(&path).expect("the probe loads");
    let nets = extract(&hierarchy);

    // KiCad puts R1, R3 and R4 on one net and leaves R2 alone: it matches a
    // vector member by index against group members that have none. kicli
    // declines to reproduce that, so its answer differs here on purpose.
    if let Some(tool) = kicad_cli() {
        let netlist = export_netlist(&tool, &path).expect("kicad-cli exported a netlist");
        let kicad = kicad_partition(&netlist);
        assert!(kicad.contains(&net(&["R1.1", "R3.1", "R4.1"])));
        assert!(kicad.contains(&net(&["R2.1"])));
        assert_ne!(partition_of(&nets), kicad);
    }

    // What kicli must never do is differ in silence.
    let warnings = nets.warnings();
    assert_eq!(warnings.len(), 1, "one bus, one warning");
    assert_eq!(warnings[0].kind.code(), "mixed-bundle-kinds");
    assert_eq!(warnings[0].names, ["AA[0..1]", "BB{P, Q}"]);
    assert!(warnings[0].message().contains("AA[0..1]"));
    assert!(warnings[0].message().contains("BB{P, Q}"));
}

#[test]
fn a_bus_of_one_bundle_kind_is_not_reported() {
    // The control: two vectors, and two groups, each pair on one bus. Neither
    // mixes kinds, so neither raises a warning.
    for (left_name, right_name, members) in [
        ("AA[0..1]", "BB[0..1]", [("AA0", "AA1"), ("BB0", "BB1")]),
        ("AA{P, Q}", "BB{P, Q}", [("AA.P", "AA.Q"), ("BB.P", "BB.Q")]),
    ] {
        let mut probe = Probe::new("one-bundle-kind");
        let first = "00000000-0000-4000-8000-cccccccccc01";
        let second = "00000000-0000-4000-8000-cccccccccc02";
        let mut left = Probe::named_child_of(&probe, "child1", first, 2);
        let mut right = Probe::named_child_of(&probe, "child2", second, 3);

        probe.sheet_named(first, "child1", left_name, ("101.6", "50.8"), "0");
        probe.sheet_named(second, "child2", right_name, ("152.4", "50.8"), "180");
        probe.bus(("101.6", "50.8"), ("152.4", "50.8"));
        bundled_members(
            &mut left,
            left_name,
            &[("R1", members[0].0), ("R2", members[0].1)],
        );
        bundled_members(
            &mut right,
            right_name,
            &[("R3", members[1].0), ("R4", members[1].1)],
        );

        let path = probe.write_all(&[&left, &right]);
        let hierarchy = Hierarchy::load(&path).expect("the probe loads");
        assert!(extract(&hierarchy).warnings().is_empty());
    }
}

#[test]
fn two_bundles_in_one_scope_share_the_members_they_both_name() {
    let mut probe = Probe::new("bundle-scope");
    let first = "00000000-0000-4000-8000-cccccccccc01";
    let second = "00000000-0000-4000-8000-cccccccccc02";
    let mut left = Probe::named_child_of(&probe, "child1", first, 2);
    let mut right = Probe::named_child_of(&probe, "child2", second, 3);

    // Two bundles of one base name and two ranges. Each leaves the root sheet
    // on a bus of its own, and the two buses never touch. The label on each
    // bus is local, so the root sheet is the scope of both.
    probe.sheet_named(first, "child1", "DQ[0..2]", ("101.6", "50.8"), "0");
    probe.bus(("101.6", "50.8"), ("152.4", "50.8"));
    probe.label_of_kind("label", "", "DQ[0..2]", ("152.4", "50.8"));
    probe.sheet_named(second, "child2", "DQ[0..1]", ("101.6", "152.4"), "0");
    probe.bus(("101.6", "152.4"), ("152.4", "152.4"));
    probe.label_of_kind("label", "", "DQ[0..1]", ("152.4", "152.4"));

    bundled_members(
        &mut left,
        "DQ[0..2]",
        &[("R1", "DQ0"), ("R2", "DQ1"), ("R3", "DQ2")],
    );
    bundled_members(&mut right, "DQ[0..1]", &[("R4", "DQ0"), ("R5", "DQ1")]);

    let found = probe.partition_with(&[&left, &right]);
    // The members both bundles name are one net each, though no bus joins the
    // two bundles and the members are drawn on different sheets.
    assert!(found.contains(&net(&["R1.1", "R4.1"])));
    assert!(found.contains(&net(&["R2.1", "R5.1"])));
    // The member only one bundle names keeps a net of its own. This is the
    // control: it fails if the two bundles have collapsed into one.
    assert!(found.contains(&net(&["R3.1"])));
}

#[test]
fn two_bundles_in_different_scopes_keep_their_members_apart() {
    let mut probe = Probe::new("bundle-scopes-apart");
    let first = "00000000-0000-4000-8000-cccccccccc01";
    let second = "00000000-0000-4000-8000-cccccccccc02";
    let mut left = Probe::named_child_of(&probe, "child1", first, 2);
    let mut right = Probe::named_child_of(&probe, "child2", second, 3);

    // The same bundle name, drawn on two child sheets and nowhere else. The
    // ports carry a name no sheet uses, so nothing joins the two bundles.
    probe.sheet_named(first, "child1", "UNUSED1", ("101.6", "50.8"), "0");
    probe.sheet_named(second, "child2", "UNUSED2", ("101.6", "152.4"), "0");

    bundled_members_named(
        &mut left,
        "label",
        "",
        "DQ[0..1]",
        &[("R1", "DQ0"), ("R2", "DQ1")],
    );
    bundled_members_named(
        &mut right,
        "label",
        "",
        "DQ[0..1]",
        &[("R3", "DQ0"), ("R4", "DQ1")],
    );

    let found = probe.partition_with(&[&left, &right]);
    // Each sheet is its own scope, so an equal member name is two nets.
    assert!(found.contains(&net(&["R1.1"])));
    assert!(found.contains(&net(&["R2.1"])));
    assert!(found.contains(&net(&["R3.1"])));
    assert!(found.contains(&net(&["R4.1"])));
}

// KiCad's answer below is measured, and kicli does not yet reproduce it. A
// bundle member keeps the name its own sheet gives it instead of taking the
// name of the bundle that carries it, so a sub-range that renames at a port
// does not carry its members through. The drawing is the smallest form of what
// vme-wren draws, which is the one corpus hierarchy still unmatched. The defect
// is older than the two bundle rules beside it: it reproduces on the commit
// before either landed. The test is kept whole and does not run, so that
// closing it is a matter of deleting one attribute.
#[ignore = "measured against KiCad; kicli does not yet rename a bundle member through a port. See research/notes/bundle-members.md"]
#[test]
fn a_wide_bundle_splits_into_sub_ranges_that_rename_at_each_port() {
    let mut probe = Probe::new("subrange-chain");
    let first = "00000000-0000-4000-8000-cccccccccc01";
    let second = "00000000-0000-4000-8000-cccccccccc02";
    let mut left = Probe::named_child_of(&probe, "child1", first, 2);
    let mut right = Probe::named_child_of(&probe, "child2", second, 3);

    // A wide bundle, split into two sub-ranges. Each sub-range feeds a child
    // whose own port bundle is named differently and starts its range at zero.
    probe.sheet_named(first, "child1", "BB[0..1]", ("101.6", "50.8"), "0");
    probe.bus(("101.6", "50.8"), ("177.8", "50.8"));
    probe.label_of_kind("label", "", "AA[0..1]", ("177.8", "50.8"));
    probe.sheet_named(second, "child2", "BB[0..1]", ("101.6", "152.4"), "0");
    probe.bus(("101.6", "152.4"), ("177.8", "152.4"));
    probe.label_of_kind("label", "", "AA[2..3]", ("177.8", "152.4"));

    // The wide bundle carries the root's own member nets.
    probe.bus(("228.6", "20.32"), ("228.6", "152.4"));
    probe.label_of_kind("label", "", "AA[0..3]", ("228.6", "20.32"));
    for (index, member) in ["AA0", "AA1", "AA2", "AA3"].iter().enumerate() {
        let wire_y = format!("{}", 38.1 + 12.7 * index as f64);
        let anchor_y = format!("{}", 41.91 + 12.7 * index as f64);
        probe.bus_entry(("226.06", &wire_y), ("2.54", "2.54"));
        probe.wire(("200.66", &wire_y), ("226.06", &wire_y));
        probe.label_of_kind("label", "", member, ("203.2", &wire_y));
        let reference = format!("R{}", index + 1);
        probe.place("R", &reference, ("200.66", &anchor_y), &["1", "2"]);
    }

    bundled_members(&mut left, "BB[0..1]", &[("R5", "BB0"), ("R6", "BB1")]);
    bundled_members(&mut right, "BB[0..1]", &[("R7", "BB0"), ("R8", "BB1")]);

    let found = probe.partition_with(&[&left, &right]);
    // Each child's members correspond by place to its own sub-range, and the
    // sub-ranges correspond by name to the wide bundle.
    assert!(found.contains(&net(&["R1.1", "R5.1"])));
    assert!(found.contains(&net(&["R2.1", "R6.1"])));
    assert!(found.contains(&net(&["R3.1", "R7.1"])));
    assert!(found.contains(&net(&["R4.1", "R8.1"])));
}

#[test]
fn two_bus_entries_that_meet_do_not_join() {
    let mut probe = Probe::new("bus-entries");

    // Two entries whose bus ends land on one point of a bundle.
    probe.bus(("127", "25.4"), ("127", "76.2"));
    probe.label_of_kind("label", "", "AN[0..7]", ("127", "25.4"));
    probe.bus_entry(("124.46", "48.26"), ("2.54", "2.54"));
    probe.bus_entry(("124.46", "53.34"), ("2.54", "-2.54"));
    probe.wire(("99.06", "48.26"), ("124.46", "48.26"));
    probe.wire(("99.06", "53.34"), ("124.46", "53.34"));
    probe.label_of_kind("label", "", "AN0", ("101.6", "48.26"));
    probe.label_of_kind("label", "", "AN2", ("101.6", "53.34"));
    probe.place("R", "R1", ("99.06", "52.07"), &["1", "2"]);
    probe.place("R", "R2", ("99.06", "57.15"), &["1", "2"]);

    // The same two entries, meeting where no bundle passes.
    probe.bus_entry(("124.46", "111.76"), ("2.54", "2.54"));
    probe.bus_entry(("124.46", "116.84"), ("2.54", "-2.54"));
    probe.wire(("99.06", "111.76"), ("124.46", "111.76"));
    probe.wire(("99.06", "116.84"), ("124.46", "116.84"));
    probe.label_of_kind("label", "", "BB0", ("101.6", "111.76"));
    probe.label_of_kind("label", "", "BB2", ("101.6", "116.84"));
    probe.place("R", "R3", ("99.06", "115.57"), &["1", "2"]);
    probe.place("R", "R4", ("99.06", "120.65"), &["1", "2"]);

    let found = probe.partition();
    // Each member keeps its own net, on the bundle and off it.
    assert!(found.contains(&net(&["R1.1"])));
    assert!(found.contains(&net(&["R2.1"])));
    assert!(found.contains(&net(&["R3.1"])));
    assert!(found.contains(&net(&["R4.1"])));
}

#[test]
fn the_instance_record_says_which_unit_a_symbol_draws() {
    let mut probe = Probe::new("instance-unit");
    probe.define(symbol(
        "PAIR",
        "U",
        false,
        &[
            (
                "1_1",
                vec![
                    pin("passive", ("0", "3.81"), "270", "1", "A"),
                    pin("passive", ("0", "-3.81"), "90", "2", "B"),
                ],
            ),
            (
                "2_1",
                vec![
                    pin("passive", ("0", "3.81"), "270", "3", "C"),
                    pin("passive", ("0", "-3.81"), "90", "4", "D"),
                ],
            ),
        ],
    ));

    // The cache beside the lib_id says unit 1; the instance says unit 2.
    let mut disagreeing = Placed::new("PAIR", "U1", ("50.8", "50.8"), &["1", "2", "3", "4"]);
    disagreeing.instance_unit = Some(2);
    probe.place_symbol(&disagreeing);
    probe.wire(("50.8", "46.99"), ("76.2", "46.99"));
    probe.label_of_kind("label", "", "TOPNET", ("76.2", "46.99"));
    probe.place("R", "R1", ("76.2", "50.8"), &["1", "2"]);

    // The control: the two agree on unit 2.
    let mut agreeing = Placed::new("PAIR", "U2", ("50.8", "88.9"), &["1", "2", "3", "4"]);
    agreeing.unit = 2;
    probe.place_symbol(&agreeing);
    probe.wire(("50.8", "85.09"), ("76.2", "85.09"));
    probe.label_of_kind("label", "", "CTRLNET", ("76.2", "85.09"));
    probe.place("R", "R2", ("76.2", "88.9"), &["1", "2"]);

    let found = probe.partition();
    // Unit 2 is drawn, so the pin on the wire is pin 3 and not pin 1.
    assert!(found.contains(&net(&["R1.1", "U1.3"])));
    assert!(found.contains(&net(&["R2.1", "U2.3"])));
    // Unit 1's pins are not drawn at all.
    assert!(!found.iter().any(|pins| pins.contains(&"U1.1".to_owned())));
}

#[test]
fn a_symbol_off_the_board_is_in_no_net_list() {
    let mut probe = Probe::new("off-the-board");
    let off_board = "(exclude_from_sim no) (in_bom yes) (on_board no) (in_pos_files yes) (dnp no)";
    let not_fitted =
        "(exclude_from_sim no) (in_bom yes) (on_board yes) (in_pos_files yes) (dnp yes)";
    let not_in_bom = "(exclude_from_sim no) (in_bom no) (on_board yes) (in_pos_files yes) (dnp no)";

    let clusters = [
        ("R1", "R2", off_board, "OFFBOARD", "25.4", "29.21"),
        ("R3", "R4", not_fitted, "NOTFITTED", "50.8", "54.61"),
        ("R5", "R6", not_in_bom, "NOTINBOM", "76.2", "80.01"),
    ];
    for (left, right, attributes, name, wire_y, anchor_y) in clusters {
        let mut placed = Placed::new("R", left, ("50.8", anchor_y), &["1", "2"]);
        placed.attributes = attributes;
        probe.place_symbol(&placed);
        probe.place("R", right, ("76.2", anchor_y), &["1", "2"]);
        probe.wire(("50.8", wire_y), ("76.2", wire_y));
        probe.label_of_kind("label", "", name, ("63.5", wire_y));
    }

    let found = probe.partition();
    // A symbol that does not reach the board is in no net list at all, not
    // even the one-pin net of its unwired pin.
    assert!(found.contains(&net(&["R2.1"])));
    assert!(
        !found
            .iter()
            .any(|pins| pins.iter().any(|pin| pin.starts_with("R1.")))
    );
    // Neither `dnp` nor `in_bom` removes a pin.
    assert!(found.contains(&net(&["R3.1", "R4.1"])));
    assert!(found.contains(&net(&["R5.1", "R6.1"])));
}

#[test]
fn one_sheet_is_one_namespace() {
    let mut probe = Probe::new("one-namespace");
    probe.define(power("PWRX"));

    // A local label against a hierarchical label of the same text.
    probe.named_strand("R1", "25.4", "29.21", "LOC");
    probe.strand_of_kind(
        "hierarchical_label",
        "(shape input)",
        "R2",
        "38.1",
        "41.91",
        "LOC",
    );
    // A local label against a global label of the same text.
    probe.named_strand("R3", "50.8", "54.61", "GLB");
    probe.strand_of_kind(
        "global_label",
        "(shape input)",
        "R4",
        "63.5",
        "67.31",
        "GLB",
    );
    // A local label against a power symbol of the same value.
    probe.named_strand("R5", "76.2", "80.01", "PWRX");
    probe.place("R", "R6", ("50.8", "92.71"), &["1", "2"]);
    probe.wire(("50.8", "88.9"), ("76.2", "88.9"));
    probe.place_unit("PWRX", "#PWR01", ("76.2", "88.9"), 1, "PWRX", &["1"]);
    // A hierarchical label against a global label of the same text.
    probe.strand_of_kind(
        "hierarchical_label",
        "(shape input)",
        "R7",
        "101.6",
        "105.41",
        "HGL",
    );
    probe.strand_of_kind(
        "global_label",
        "(shape input)",
        "R8",
        "114.3",
        "118.11",
        "HGL",
    );

    let found = probe.partition();
    assert!(found.contains(&net(&["R1.1", "R2.1"])));
    assert!(found.contains(&net(&["R3.1", "R4.1"])));
    assert!(found.contains(&net(&["R5.1", "R6.1"])));
    assert!(found.contains(&net(&["R7.1", "R8.1"])));
}

#[test]
fn a_hidden_power_input_reaches_its_rail_with_no_wire() {
    let mut probe = Probe::new("hidden-power-pin");
    // One ordinary part with a hidden power input, one with a visible one.
    probe.define(symbol(
        "HID",
        "U",
        false,
        &[(
            "1_1",
            vec![
                pin("passive", ("0", "3.81"), "270", "1", "IO"),
                hidden_pin("power_in", ("7.62", "0"), "9", "VHID"),
            ],
        )],
    ));
    probe.define(symbol(
        "VIS",
        "U",
        false,
        &[(
            "1_1",
            vec![
                pin("passive", ("0", "3.81"), "270", "1", "IO"),
                pin("power_in", ("7.62", "0"), "180", "9", "VVIS"),
            ],
        )],
    ));
    probe.define(power("VHID"));
    probe.define(power("VVIS"));

    probe.place("HID", "U1", ("50.8", "50.8"), &["1", "9"]);
    probe.place_unit("VHID", "#PWR01", ("101.6", "50.8"), 1, "VHID", &["1"]);
    probe.wire(("101.6", "50.8"), ("101.6", "54.61"));
    probe.place("R", "R1", ("101.6", "58.42"), &["1", "2"]);

    probe.place("VIS", "U2", ("50.8", "88.9"), &["1", "9"]);
    probe.place_unit("VVIS", "#PWR02", ("101.6", "88.9"), 1, "VVIS", &["1"]);
    probe.wire(("101.6", "88.9"), ("101.6", "92.71"));
    probe.place("R", "R2", ("101.6", "96.52"), &["1", "2"]);

    let found = probe.partition();
    // The hidden input is on the rail its pin name asks for.
    assert!(found.contains(&net(&["R1.1", "U1.9"])));
    // The visible one is not. A power input on an ordinary symbol names a
    // net only when the editor hides it.
    assert!(found.contains(&net(&["R2.1"])));
    assert!(found.contains(&net(&["U2.9"])));
}
