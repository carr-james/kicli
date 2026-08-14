//! Adding a new object to a sheet: where it goes, and what it is called.
//!
//! Both questions have one answer each, and every command that makes an object
//! asks them. A new object goes before the trailing metadata, so the file keeps
//! the shape KiCad writes. Its identifier is derived from a seed rather than
//! read from a random source, so one command run twice over one design produces
//! one file and a test is repeatable.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use kicli_sexpr::{Doc, NodeId};
use sha2::{Digest, Sha256};

use crate::model::items::Uuid;

/// Where a new object goes among a sheet's children.
///
/// It goes before the trailing metadata, so the file keeps the shape KiCad
/// writes: the objects of the drawing, then `sheet_instances`, then the
/// embedded fonts.
pub(crate) fn insertion_index(doc: &Doc, root: NodeId) -> usize {
    let children = doc.children(root);
    children
        .iter()
        .position(|&child| {
            doc.head_is(child, "sheet_instances") || doc.head_is(child, "embedded_fonts")
        })
        .unwrap_or(children.len())
}

/// An identifier for a new object that no object of this file already has.
///
/// The value is a function of `seed`, so one command run twice over one file
/// gives one answer.
pub(crate) fn fresh_uuid(doc: &Doc, seed: &str) -> Uuid {
    Identifiers::for_document(doc, seed)
        .next()
        .unwrap_or_else(|| Uuid(uuid_from(seed)))
}

/// A run of identifiers for the objects one command makes.
///
/// A placement needs one for the symbol and one for each of its pins. Every
/// value is derived from the seed and checked against the file and against the
/// ones already handed out, so no two objects of one command collide.
///
/// # Examples
///
/// ```
/// use kicli::edit::insert::Identifiers;
/// use kicli_sexpr::Doc;
///
/// let doc = Doc::parse("(kicad_sch)\n").expect("parses");
/// let made: Vec<_> = Identifiers::for_document(&doc, "a seed").take(2).collect();
/// assert_ne!(made[0], made[1]);
/// assert_eq!(made[0].0.len(), 36, "the shape KiCad's reader expects");
/// ```
pub struct Identifiers {
    /// Every identifier the file holds, and every one handed out so far.
    taken: BTreeSet<String>,
    /// The text every value is derived from.
    seed: String,
    /// The next derivation to try.
    next: u32,
}

impl Identifiers {
    /// Start a run of identifiers no object of `doc` already carries.
    #[must_use]
    pub fn for_document(doc: &Doc, seed: &str) -> Self {
        Self {
            taken: doc.uuid_index().into_keys().collect(),
            seed: seed.to_owned(),
            next: 0,
        }
    }
}

impl Iterator for Identifiers {
    type Item = Uuid;

    fn next(&mut self) -> Option<Uuid> {
        while self.next < u32::MAX {
            let candidate = uuid_from(&format!("{} {}", self.seed, self.next));
            self.next += 1;
            if self.taken.insert(candidate.clone()) {
                return Some(Uuid(candidate));
            }
        }
        None
    }
}

/// One identifier, derived from text.
fn uuid_from(seed: &str) -> String {
    let digest = Sha256::digest(seed.as_bytes());
    let mut hex = String::with_capacity(32);
    for byte in digest.iter().take(16) {
        let _ = write!(hex, "{byte:02x}");
    }
    // The version and variant nibbles, which KiCad's reader expects to find.
    hex.replace_range(12..13, "4");
    hex.replace_range(16..17, "8");
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

#[cfg(test)]
mod tests {
    use super::{Identifiers, fresh_uuid, uuid_from};
    use kicli_sexpr::Doc;

    #[test]
    fn an_identifier_has_the_shape_kicad_reads() {
        let made = uuid_from("anything");
        assert_eq!(made.len(), 36);
        assert_eq!(made.as_bytes()[14], b'4', "the version nibble");
        assert_eq!(made.as_bytes()[19], b'8', "the variant nibble");
    }

    #[test]
    fn one_seed_gives_one_identifier() {
        let doc = Doc::parse("(kicad_sch)\n").expect("parses");
        assert_eq!(fresh_uuid(&doc, "seed"), fresh_uuid(&doc, "seed"));
        assert_ne!(fresh_uuid(&doc, "seed"), fresh_uuid(&doc, "other"));
    }

    #[test]
    fn a_run_skips_what_the_file_already_holds() {
        let doc = Doc::parse("(kicad_sch)\n").expect("parses");
        let first = fresh_uuid(&doc, "seed");
        let source = format!("(kicad_sch (junction (uuid \"{}\")))\n", first.0);
        let crowded = Doc::parse(&source).expect("parses");
        assert_ne!(
            fresh_uuid(&crowded, "seed"),
            first,
            "the identifier the file holds is not handed out again"
        );

        let run: Vec<_> = Identifiers::for_document(&doc, "seed").take(3).collect();
        assert_eq!(run[0], first, "a run starts where one identifier would");
        assert_ne!(run[1], run[0]);
        assert_ne!(run[2], run[1]);
    }
}
