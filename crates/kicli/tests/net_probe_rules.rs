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

/// One probe drawing, built item by item.
struct Probe {
    name: &'static str,
    symbols: Vec<String>,
    items: Vec<String>,
    next_uuid: u32,
}

impl Probe {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            symbols: vec![resistor()],
            items: Vec::new(),
            next_uuid: 0,
        }
    }

    /// A fresh uuid. The counter makes every probe file reproducible.
    fn uuid(&mut self) -> String {
        self.next_uuid += 1;
        format!("00000000-0000-4000-8000-{:012}", self.next_uuid)
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
             (instances (project \"probe\" (path \"/{ROOT}\" (reference \"{reference}\") (unit {unit}))))\n)"
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
        format!(
            "(kicad_sch (version 20260306) (generator \"eeschema\") (generator_version \"10.0\")\n\
             (uuid \"{ROOT}\") (paper \"A4\")\n(lib_symbols\n{}\n)\n{}\n\
             (sheet_instances (path \"/\" (page \"1\")))\n(embedded_fonts no)\n)",
            self.symbols.join("\n"),
            self.items.join("\n")
        )
    }

    /// Write the probe to the scratch directory and return its path.
    fn write(&self) -> PathBuf {
        let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join("net-probes");
        std::fs::create_dir_all(&directory).expect("the scratch directory is writable");
        let path = directory.join(format!("{}.kicad_sch", self.name));
        std::fs::write(&path, self.text()).expect("the probe file is writable");
        path
    }

    /// kicli's partition of the probe, checked against KiCad when it is there.
    fn partition(&self) -> Partition {
        let path = self.write();
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
    unit: u32,
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
