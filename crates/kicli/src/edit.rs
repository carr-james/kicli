//! The commands that change a schematic.
//!
//! Every one of them follows the same path: change the tree, run the
//! invariants, write atomically, report what moved. The path itself lives in
//! [`crate::model::mutate`]; this module holds what each command changes.
//!
//! A command here never writes bytes of its own. That is what makes "every
//! mutation is verified and reported" a property of the design rather than a
//! rule each command has to remember.

pub mod field;
pub mod label;
pub mod mark;
pub mod net;
pub mod symbol;
