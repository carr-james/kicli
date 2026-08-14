//! The instruments kicli's measurement probes are built from.
//!
//! A probe is a schematic drawn to answer one question, handed to KiCad, and
//! compared against what kicli reads from the same drawing. This crate holds
//! the parts every probe needs, so a probe file holds the question and the
//! answer and nothing else.
//!
//! # The discipline this crate encodes
//!
//! **A probe is an instrument, and an instrument fails.** Three probes in this
//! project measured their own defects before they measured KiCad: a sheet-pin
//! angle that took a port off its bus, and coordinates at full floating-point
//! precision that the reader turned into zeros. The builder below refuses both
//! rather than letting a probe report a drawing nobody drew.
//!
//! **A negative result needs a control that must fire.** Before concluding
//! anything from a probe that did not produce the expected behaviour, run a
//! variant that must produce it. If the control also fails, the instrument is
//! broken. A negative result with no passing control is not evidence.
//!
//! **An oracle record is KiCad's answer, never a hand-written expectation.** A
//! fixture written from the same assumption as the code tests nothing.
//!
//! # Scratch directories
//!
//! Every writer here takes the directory to write in. `CARGO_TARGET_TMPDIR` is
//! set for test and bench targets only, so a library cannot read it at compile
//! time and the caller passes its own.

// Undocumented public items and unsafe code are errors. This problem domain
// never needs unsafe. Pedantic lints warn; allow one only with a reason beside
// it.
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::pedantic)]

pub mod drawing;

pub mod oracle;

pub mod scratch;

pub use drawing::{
    Placed, Probe, hidden_pin, millimetres, pin, power, rectangle, resistor, symbol,
};
pub use oracle::{
    Change, Kicad, NamedNet, Netlist, Partition, Report, ReportPin, net, with_and_without,
};
pub use scratch::Fixtures;
