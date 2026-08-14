//! Two terminals and a sheet become an ordered list of grid points, and the
//! cost of reaching them.
//!
//! The module is pure: it knows nothing of files, the command line, or
//! `kicad-cli`. It is handed the geometry a sheet holds and answers with a
//! path, so the search is as cheap to test as arithmetic.
//!
//! Every cost is an `i64` and every coordinate an [`Iu`](crate::geometry::Iu).
//! There is no floating point anywhere below this line: two runs over one sheet
//! must produce the same route, on any machine, forever.

pub mod terminal;

pub use terminal::{BlockedEscape, Heading, Obstruction, Terminal};
