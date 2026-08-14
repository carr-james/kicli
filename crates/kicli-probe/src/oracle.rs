//! Asking KiCad, and reading the answer it gives.
//!
//! An oracle record is what KiCad said about a drawing. It is never written by
//! hand: a fixture written from the same assumption as the code tests nothing.
//! Every reader here turns one of KiCad's own files into a typed answer, so a
//! test holds a record rather than a string it parses again.
//!
//! The tool runs only when `KICLI_TEST_KICAD_CLI` is set, so the default test
//! run needs no KiCad install. `KICLI_KICAD_CLI` names the binary when it is
//! not `kicad-cli` on the path.
//!
//! The tool's own output is dropped. The first run on a machine prints
//! fontconfig warnings that say nothing about the drawing.

use crate::drawing::Probe;
use kicli::connectivity::{NetPin, Nets, extract};
use kicli::geometry::{Iu, Point};
use kicli::model::Hierarchy;
use kicli_sexpr::{Doc, NodeId};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::{Command, Stdio};

/// A net partition: one sorted pin list per net.
pub type Partition = BTreeSet<Vec<String>>;

/// Say that a test did nothing, and what would make it run.
pub fn skipped(reason: &str) {
    eprintln!("skipped: set KICLI_TEST_KICAD_CLI to {reason}");
}

/// The `kicad-cli` a test measures with.
pub struct Kicad {
    binary: String,
}

impl Kicad {
    /// The tool the environment asks for, or nothing.
    #[must_use]
    pub fn found() -> Option<Self> {
        std::env::var("KICLI_TEST_KICAD_CLI").ok()?;
        Some(Self {
            binary: std::env::var("KICLI_KICAD_CLI").unwrap_or_else(|_| "kicad-cli".to_owned()),
        })
    }

    /// The same, saying why the test did nothing when the tool is not asked for.
    #[must_use]
    pub fn found_or_skip(reason: &str) -> Option<Self> {
        let found = Self::found();
        if found.is_none() {
            skipped(reason);
        }
        found
    }

    /// The netlist of a sheet, written beside it as `.net`.
    ///
    /// # Panics
    ///
    /// If the tool does not run, or writes no netlist.
    #[must_use]
    pub fn netlist_beside(&self, sheet: &Path) -> Netlist {
        let into = sheet.with_extension("net");
        self.netlist(sheet, &into)
    }

    /// The netlist of a sheet, written where the caller says.
    ///
    /// # Panics
    ///
    /// If the tool does not run, or writes no netlist.
    #[must_use]
    pub fn netlist(&self, sheet: &Path, into: &Path) -> Netlist {
        self.try_netlist(sheet, into)
            .expect("kicad-cli exported a netlist")
    }

    /// The same, answering nothing when the tool fails.
    ///
    /// A caller sweeping many projects needs the failure as data rather than as
    /// a panic, so it can name the project that would not export.
    #[must_use]
    pub fn try_netlist(&self, sheet: &Path, into: &Path) -> Option<Netlist> {
        let status = Command::new(&self.binary)
            .args(["sch", "export", "netlist", "-o"])
            .arg(into)
            .arg(sheet)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .ok()?;
        if !status.success() {
            return None;
        }
        let text = std::fs::read_to_string(into).ok()?;
        Some(Netlist::parse(&text))
    }

    /// KiCad's own electrical rule check, as a report beside the sheet.
    ///
    /// # Panics
    ///
    /// If the tool does not run, or writes no report.
    #[must_use]
    pub fn rule_check(&self, sheet: &Path) -> Report {
        let directory = sheet.parent().expect("the sheet sits in a directory");
        self.rule_check_into(sheet, &directory.join("rule-check.txt"))
    }

    /// The same, as a report where the caller says.
    ///
    /// # Panics
    ///
    /// If the tool does not run, or writes no report.
    #[must_use]
    pub fn rule_check_into(&self, sheet: &Path, into: &Path) -> Report {
        let directory = sheet.parent().expect("the sheet sits in a directory");
        let status = Command::new(&self.binary)
            .current_dir(directory)
            .args([
                "sch",
                "erc",
                "--format",
                "report",
                "--units",
                "mm",
                "--severity-all",
                "-o",
            ])
            .arg(into)
            .arg(sheet.file_name().expect("the sheet has a name"))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("kicad-cli runs");
        assert!(status.success(), "the rule check ran");
        Report::parse(&std::fs::read_to_string(into).expect("the report reads"))
    }

    /// The drawing KiCad makes of a sheet, as SVG text.
    ///
    /// # Panics
    ///
    /// If the tool does not run, or writes no drawing.
    #[must_use]
    pub fn svg(&self, sheet: &Path) -> String {
        let into = sheet.with_extension("svg-out");
        self.try_svg(sheet, &into, &[])
            .unwrap_or_else(|reason| panic!("kicad-cli plotted {}: {reason}", sheet.display()))
    }

    /// The same, with plot options, and the refusal as text.
    ///
    /// A caller that plots a sheet KiCad may decline needs the reason to print
    /// rather than a panic, because the reason is what tells it whether the
    /// instrument or the drawing is at fault.
    ///
    /// # Errors
    ///
    /// If the tool does not run, if it refuses the sheet, or if the drawing it
    /// names is not there.
    pub fn try_svg(&self, sheet: &Path, into: &Path, options: &[&str]) -> Result<String, String> {
        std::fs::create_dir_all(into)
            .map_err(|error| format!("cannot make a directory: {error}"))?;
        let output = Command::new(&self.binary)
            .args(["sch", "export", "svg"])
            .args(options)
            .arg("-o")
            .arg(into)
            .arg(sheet)
            .output()
            .map_err(|error| format!("cannot run kicad-cli: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "kicad-cli refused the sheet: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        let stem = sheet.file_stem().ok_or("the sheet has no name")?;
        let plotted = into.join(stem).with_extension("svg");
        std::fs::read_to_string(&plotted)
            .map_err(|error| format!("cannot read {}: {error}", plotted.display()))
    }
}

/// One net of a netlist: KiCad's name for it, and its pins sorted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedNet {
    /// The name KiCad gave the net, which may be one it derived.
    pub name: String,
    /// The pins on it, as `R1.2`, sorted.
    pub pins: Vec<String>,
}

/// A netlist KiCad wrote.
pub struct Netlist {
    text: String,
    nets: Vec<NamedNet>,
}

impl Netlist {
    /// Read a netlist KiCad wrote.
    ///
    /// # Panics
    ///
    /// If the text does not parse, or carries no net at all. A reading with no
    /// nets is not an answer: a comparison against it would succeed and say
    /// nothing.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        let doc = Doc::parse(text).expect("the netlist parses");
        let root = doc.root().expect("the netlist has a root list");
        let mut nets = Vec::new();
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
                nets.push(NamedNet {
                    name: atom_of(&doc, net, "name").unwrap_or_default(),
                    pins,
                });
            }
        }
        assert!(!nets.is_empty(), "the netlist reported no nets at all");
        Self {
            text: text.to_owned(),
            nets,
        }
    }

    /// The file as KiCad wrote it.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Every net, in the order the file lists them.
    #[must_use]
    pub fn nets(&self) -> &[NamedNet] {
        &self.nets
    }

    /// The partition: one sorted pin list per net, empty nets dropped.
    #[must_use]
    pub fn partition(&self) -> Partition {
        self.nets
            .iter()
            .filter(|net| !net.pins.is_empty())
            .map(|net| net.pins.clone())
            .collect()
    }

    /// Every net name KiCad wrote.
    #[must_use]
    pub fn names(&self) -> BTreeSet<String> {
        self.nets.iter().map(|net| net.name.clone()).collect()
    }

    /// The nets whose name ends with a label's text.
    ///
    /// KiCad prefixes a local net name with the sheet path it was named on, so
    /// a test that knows the label knows the end of the name and not all of it.
    #[must_use]
    pub fn named(&self, text: &str) -> Vec<&NamedNet> {
        self.nets
            .iter()
            .filter(|net| net.name.ends_with(text))
            .collect()
    }

    /// What KiCad calls one symbol, on one sheet path.
    ///
    /// A netlist writes the symbol's own identifier as `tstamps`, and the sheet
    /// path it is on as the sheet's `tstamps`, which ends in a separator.
    ///
    /// # Panics
    ///
    /// If the text does not parse.
    #[must_use]
    pub fn reference_of(&self, symbol: &str, sheet_path: &str) -> Option<String> {
        let doc = Doc::parse(&self.text).expect("the netlist parses");
        let root = doc.root().expect("the netlist has a root list");
        let leaf = sheet_path.rsplit('/').next().unwrap_or_default();
        for &components in doc.children(root) {
            if !doc.head_is(components, "components") {
                continue;
            }
            for &component in doc.children(components) {
                if !doc.head_is(component, "comp") || atom_of(&doc, component, "tstamps")? != symbol
                {
                    continue;
                }
                let sheet = doc
                    .children(component)
                    .iter()
                    .copied()
                    .find(|&child| doc.head_is(child, "sheetpath"))?;
                if atom_of(&doc, sheet, "tstamps")?.contains(leaf) {
                    return atom_of(&doc, component, "ref");
                }
            }
        }
        None
    }
}

/// One pin of one symbol, as KiCad's rule-check report gives it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReportPin {
    /// The reference designator, on the sheet path the report walked.
    pub reference: String,
    /// The pin number.
    pub number: String,
    /// Where KiCad says the pin is.
    pub at: Point,
}

/// A rule-check report KiCad wrote.
///
/// An empty report is a real answer: a drawing may raise no violation at all.
/// A report the tool never wrote is not, and the runner above panics instead.
pub struct Report {
    text: String,
    pins: Vec<ReportPin>,
}

impl Report {
    /// Read a report KiCad wrote.
    ///
    /// An item line reads
    /// `@(25.40 mm, 21.59 mm): Symbol R1 Pin 1 [Passive, Line]`.
    ///
    /// # Panics
    ///
    /// If an item line carries a position that is not a pair of coordinates.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        let mut pins = Vec::new();
        for line in text.lines().map(str::trim) {
            let Some(rest) = line.strip_prefix("@(") else {
                continue;
            };
            let Some((position, description)) = rest.split_once("): ") else {
                continue;
            };
            let Some(symbol) = description.strip_prefix("Symbol ") else {
                continue;
            };
            let mut words = symbol.split_whitespace();
            let (Some(reference), Some("Pin"), Some(number)) =
                (words.next(), words.next(), words.next())
            else {
                continue;
            };
            let (x, y) = position.split_once(", ").expect("two coordinates");
            pins.push(ReportPin {
                reference: reference.to_owned(),
                number: number.to_owned(),
                at: Point {
                    x: millimetres(x),
                    y: millimetres(y),
                },
            });
        }
        Self {
            text: text.to_owned(),
            pins,
        }
    }

    /// Read a report from a file KiCad wrote.
    ///
    /// # Panics
    ///
    /// If the file does not read.
    #[must_use]
    pub fn read(path: &Path) -> Self {
        Self::parse(&std::fs::read_to_string(path).expect("the report reads"))
    }

    /// The file as KiCad wrote it.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Every pin the report named.
    #[must_use]
    pub fn pins(&self) -> &[ReportPin] {
        &self.pins
    }

    /// Where the report puts each pin, keyed by reference and pin number.
    #[must_use]
    pub fn pin_positions(&self) -> BTreeMap<(String, String), Point> {
        self.pins
            .iter()
            .map(|pin| ((pin.reference.clone(), pin.number.clone()), pin.at))
            .collect()
    }

    /// The pins of one symbol, as number and position.
    #[must_use]
    pub fn pins_of(&self, reference: &str) -> Vec<(String, Point)> {
        self.pins
            .iter()
            .filter(|pin| pin.reference == reference)
            .map(|pin| (pin.number.clone(), pin.at))
            .collect()
    }

    /// The kinds of violation the report carries, such as `pin_not_connected`.
    #[must_use]
    pub fn violation_kinds(&self) -> BTreeSet<String> {
        self.text
            .lines()
            .map(str::trim)
            .filter_map(|line| line.strip_prefix('['))
            .filter_map(|line| line.split_once(']'))
            .map(|(kind, _)| kind.to_owned())
            .collect()
    }

    /// The item lines, which is what two runs can be compared on.
    ///
    /// The counts and headings around them move whenever anything is added.
    #[must_use]
    pub fn items(&self) -> BTreeSet<String> {
        self.text
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with("@("))
            .map(str::to_owned)
            .collect()
    }
}

/// The lines of a file KiCad wrote, without the ones that move on every run.
///
/// A timestamp and the caller's own path say nothing about the drawing, so a
/// comparison of a committed oracle against a fresh one drops them.
#[must_use]
pub fn without_the_run_specific_lines(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim_end)
        .filter(|line| {
            let trimmed = line.trim_start();
            !trimmed.starts_with("ERC report (")
                && !trimmed.starts_with("(date ")
                && !trimmed.starts_with("(source ")
                && !trimmed.starts_with("(tool ")
        })
        .collect()
}

/// The partition kicli reads, in the form a netlist reports it.
///
/// A power pin names a rail rather than joining a part, and a symbol that does
/// not reach the board is in no net list, so neither appears here.
#[must_use]
pub fn kicli_partition(nets: &Nets) -> Partition {
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

/// One expected net, written as a pin list in any order.
#[must_use]
pub fn net(pins: &[&str]) -> Vec<String> {
    let mut sorted: Vec<String> = pins.iter().map(|pin| (*pin).to_owned()).collect();
    sorted.sort();
    sorted
}

/// Report the nets two partitions disagree about, or nothing.
#[must_use]
pub fn differences(kicli: &Partition, kicad: &Partition) -> Option<String> {
    let missing: Vec<&Vec<String>> = kicad.difference(kicli).collect();
    let extra: Vec<&Vec<String>> = kicli.difference(kicad).collect();
    if missing.is_empty() && extra.is_empty() {
        return None;
    }
    Some(format!(
        "nets KiCad found and kicli did not: {missing:?}\n\
         nets kicli found and KiCad did not: {extra:?}"
    ))
}

impl Probe {
    /// kicli's partition of this probe, checked against KiCad when it is there.
    ///
    /// # Panics
    ///
    /// If the probe does not load, or if KiCad disagrees about it.
    #[must_use]
    pub fn partition(&self) -> Partition {
        self.partition_with(&[])
    }

    /// The same, for a probe that draws child sheets.
    ///
    /// # Panics
    ///
    /// If the probe does not load, or if KiCad disagrees about it.
    #[must_use]
    pub fn partition_with(&self, children: &[&Probe]) -> Partition {
        let path = self.write_all(children);
        let hierarchy = Hierarchy::load(&path).expect("the probe loads");
        let found = kicli_partition(&extract(&hierarchy));
        if let Some(kicad) = Kicad::found() {
            assert_eq!(
                found,
                kicad.netlist_beside(&path).partition(),
                "kicli and KiCad disagree about {}",
                self.name()
            );
        }
        found
    }

    /// The loaded hierarchy of this probe, its children written beside it.
    ///
    /// # Panics
    ///
    /// If the probe does not load.
    #[must_use]
    pub fn hierarchy(&self, children: &[&Probe]) -> Hierarchy {
        let path = self.write_all(children);
        Hierarchy::load(&path).expect("the probe loads")
    }
}

/// Read a `12.34 mm` reading as internal units, without going through a float.
fn millimetres(reading: &str) -> Iu {
    Iu::from_millimetres_text(reading.trim_end_matches(" mm")).expect("a coordinate is a number")
}

/// The atom after a head, as in `(ref "R1")`.
fn atom_of(doc: &Doc, list: NodeId, head: &str) -> Option<String> {
    let child = doc
        .children(list)
        .iter()
        .copied()
        .find(|&child| doc.head_is(child, head))?;
    doc.children(child)
        .get(1)
        .and_then(|&atom| doc.atom_as_str(atom))
}

/// One `(node (ref "R1") (pin "2") …)` as `R1.2`.
fn node_label(doc: &Doc, node: NodeId) -> Option<String> {
    Some(format!(
        "{}.{}",
        atom_of(doc, node, "ref")?,
        atom_of(doc, node, "pin")?
    ))
}
