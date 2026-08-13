//! What kicli will and will not write.
//!
//! Some files cannot be written back without losing something. KiCad drops
//! comments on save. A stamp newer than kicli understands may carry tokens
//! whose meaning kicli guesses wrong. A bare atom starting with `#` reads back
//! as a comment. In each case kicli refuses and says why, rather than writing a
//! file that opens cleanly and means something else.

use super::version::{FormatVersion, MAX_SCHEMATIC_VERSION};
use kicli_sexpr::Doc;

/// What the caller allows.
#[derive(Clone, Copy, Debug)]
pub struct WriteOptions {
    /// Write a file that carries `#` comments, losing them as KiCad would.
    pub allow_comment_loss: bool,
    /// The newest stamp kicli will write.
    pub max_version: FormatVersion,
}

impl Default for WriteOptions {
    fn default() -> Self {
        Self {
            allow_comment_loss: false,
            max_version: MAX_SCHEMATIC_VERSION,
        }
    }
}

/// Why kicli will not write a file.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WriteRefusal {
    /// The file carries comments, which writing would drop.
    #[error(
        "the file carries {count} comment(s), which writing would drop; pass --allow-comment-loss to write anyway"
    )]
    WouldDropComments {
        /// How many comments.
        count: usize,
    },

    /// The stamp is newer than kicli was built against.
    #[error(
        "the file's format stamp {found} is newer than {known}, which kicli understands; writing could drop tokens it did not recognise"
    )]
    VersionTooNew {
        /// The stamp in the file.
        found: u32,
        /// The newest stamp kicli knows.
        known: u32,
    },

    /// An atom cannot survive the round trip.
    #[error(
        "the file holds {count} bare atom(s) starting with '#', which read back as comments once laid out"
    )]
    UnrepresentableAtoms {
        /// How many such atoms.
        count: usize,
    },
}

/// A file kicli is willing to write, and what writing it changed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WritePlan {
    /// The bytes to write.
    pub bytes: String,
    /// Did the layout change because the input was not in KiCad's own form?
    pub reformatted: bool,
    /// Why, when it did.
    pub reason: Option<String>,
}

/// The stamp in `(version ...)`, when the file has one.
#[must_use]
pub fn format_version(doc: &Doc) -> Option<FormatVersion> {
    let root = doc.root()?;
    doc.children(root)
        .iter()
        .find(|&&child| doc.head_is(child, "version"))
        .and_then(|&child| doc.children(child).get(1).copied())
        .and_then(|atom| doc.atom_text(atom))
        .and_then(|text| text.parse().ok())
        .map(FormatVersion::new)
}

/// Decide whether to write, and what the result would be.
///
/// # Errors
///
/// Returns a [`WriteRefusal`] when writing would lose something: comments the
/// caller has not agreed to drop, tokens from a format newer than kicli knows,
/// or an atom that cannot be read back as itself.
pub fn plan_write(doc: &Doc, options: WriteOptions) -> Result<WritePlan, WriteRefusal> {
    if let Some(version) = format_version(doc)
        && version > options.max_version
    {
        return Err(WriteRefusal::VersionTooNew {
            found: version.stamp(),
            known: options.max_version.stamp(),
        });
    }

    let unrepresentable = doc.unrepresentable_atoms();
    if !unrepresentable.is_empty() {
        return Err(WriteRefusal::UnrepresentableAtoms {
            count: unrepresentable.len(),
        });
    }

    if doc.has_comments() && !options.allow_comment_loss {
        return Err(WriteRefusal::WouldDropComments {
            count: doc
                .node_ids()
                .filter(|&id| doc.atom_text(id).is_some_and(|t| t.starts_with('#')))
                .count(),
        });
    }

    let reformatted = !doc.is_canonical();
    Ok(WritePlan {
        bytes: doc.emit(),
        reformatted,
        reason: reformatted.then(|| {
            "the input was not in KiCad's own layout, so the whole file was laid out again"
                .to_owned()
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_canonical_file_writes_without_reformatting() {
        let doc = Doc::parse("(kicad_sch\n\t(version 20260306)\n)\n").expect("parses");
        let plan = plan_write(&doc, WriteOptions::default()).expect("writes");
        assert!(!plan.reformatted);
        assert!(plan.reason.is_none());
    }

    #[test]
    fn a_newer_stamp_is_refused_unless_the_ceiling_is_raised() {
        let doc = Doc::parse("(kicad_sch\n\t(version 20260803)\n)\n").expect("parses");
        assert_eq!(
            plan_write(&doc, WriteOptions::default()),
            Err(WriteRefusal::VersionTooNew {
                found: 20_260_803,
                known: 20_260_306
            })
        );

        let raised = WriteOptions {
            max_version: FormatVersion::new(20_260_803),
            ..WriteOptions::default()
        };
        assert!(plan_write(&doc, raised).is_ok());
    }

    #[test]
    fn a_bare_hash_atom_is_refused() {
        let doc = Doc::parse("(a #PWR01)").expect("parses");
        assert_eq!(
            plan_write(&doc, WriteOptions::default()),
            Err(WriteRefusal::UnrepresentableAtoms { count: 1 })
        );
    }
}
