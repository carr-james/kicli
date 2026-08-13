//! Tokens, the token-preserving tree, and KiCad's pretty-printer.
//!
//! This crate parses KiCad s-expression files into a tree. The tree keeps the
//! exact source text of every atom. The crate emits that tree back through a
//! port of KiCad's `KICAD_FORMAT::Prettify`
//! (`common/io/kicad/kicad_io_utils.cpp`). Two round-trip properties live here.
//! An emit of an unedited KiCad-authored file reproduces its bytes exactly. A
//! parse of any emitted file reproduces the tree it came from.
//!
//! KiCad writes a flat token stream, then reformats the whole buffer, so
//! whitespace carries no information. Keeping each atom's source text and
//! replaying the prettifier therefore reproduces the file exactly. A tree that
//! stored whitespace would be larger and would buy nothing.
//!
//! This crate knows nothing about schematics. It handles s-expressions only.
//! Schematic meaning lives in the `kicli` crate, and the crate boundary keeps
//! that direction one-way.
//!
//! # Reading a file and writing it back
//!
//! ```
//! let source = "(kicad_sch\n\t(version 20260306)\n)\n";
//! let doc = kicli_sexpr::Doc::parse(source).expect("parses");
//! assert!(doc.is_canonical());
//! assert_eq!(doc.emit(), source);
//! ```

// Undocumented public items and unsafe code are errors. This problem domain
// never needs unsafe. Pedantic lints warn; allow one only with a reason beside
// it.
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::pedantic)]

pub mod error;
pub mod lexer;
pub mod number;
pub mod prettify;
pub mod quote;
pub mod tree;

pub use error::{SexprError, Span};
pub use lexer::{Token, TokenKind, lex};
pub use number::{UNITS_PER_MM, fmt_angle, fmt_iu, format_significant, parse_iu};
pub use prettify::{FormatMode, detect_mode, flatten, prettify};
pub use quote::{quote, unquote};
pub use tree::{AtomKind, Doc, Node, NodeId, changed_line_count};
