//! The readers refuse a reading that is not an answer.
//!
//! These tests need no KiCad: they hand the readers text of the shape KiCad
//! writes. What they hold is the line between "KiCad said nothing" and "KiCad
//! said there is nothing", which are different answers and must not compare
//! equal.

use kicli_probe::oracle::{Netlist, Report};

/// A netlist of one net, in the shape `kicad-cli sch export netlist` writes.
const ONE_NET: &str = r#"(export (version "E")
  (components
    (comp (ref "R1") (sheetpath (names "/") (tstamps "/")) (tstamps "/1111")))
  (nets
    (net (code "1") (name "/SPY")
      (node (ref "R1") (pin "1"))
      (node (ref "R2") (pin "2")))))"#;

/// The same file with the nets removed, which no schematic produces.
const NO_NETS: &str = r#"(export (version "E")
  (components
    (comp (ref "R1") (tstamps "/1111"))))"#;

#[test]
#[should_panic(expected = "reported no nets at all")]
fn a_netlist_with_no_nets_is_not_an_answer() {
    // A comparison against an empty partition succeeds and says nothing, so
    // the reading is refused where it is read rather than where it is used.
    let _ = Netlist::parse(NO_NETS);
}

#[test]
fn a_netlist_with_nets_is_read() {
    // The control for the refusal above: the same file, nets included.
    let netlist = Netlist::parse(ONE_NET);
    assert_eq!(netlist.nets().len(), 1);
    assert_eq!(netlist.nets()[0].name, "/SPY");
    assert_eq!(netlist.nets()[0].pins, ["R1.1", "R2.2"]);
    assert_eq!(netlist.named("SPY").len(), 1, "a name ends with its label");
    assert_eq!(netlist.reference_of("/1111", "/").as_deref(), Some("R1"));
}

#[test]
fn a_report_with_no_items_is_a_real_answer() {
    // A rule check that finds nothing is a drawing with no violation, which is
    // a true answer and not a failed reading. A report the tool never wrote is
    // the failure, and the runner panics there instead. The control that a
    // comparison needs is the caller's: it holds the before-reading against
    // the after-reading and refuses an empty before.
    let report = Report::parse("ERC report (2026-01-02)\n\n** Found 0 violations **\n");
    assert!(report.pins().is_empty());
    assert!(report.items().is_empty());
}

#[test]
fn a_report_names_the_pins_it_carries() {
    let report = Report::parse(
        "[unconnected_items]: Symbol pin not connected\n    \
         @(25.40 mm, 21.59 mm): Symbol R1 Pin 1 [Passive, Line]\n",
    );
    let pins = report.pins_of("R1");
    assert_eq!(pins.len(), 1);
    assert_eq!(pins[0].0, "1");
    assert_eq!(pins[0].1.x.millimetres(), 25.4);
    assert!(report.violation_kinds().contains("unconnected_items"));
}
