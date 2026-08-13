//! The one place kicli writes a schematic to disk.
//!
//! Write a temporary file, verify the bytes that landed in it, then rename it
//! over the target. A rename within one directory is atomic, so a reader sees
//! either the old file or the new one and never half of either.
//!
//! Verification reads back what was written rather than trusting the tree in
//! memory. A fault in the emitter is exactly the kind of fault that would
//! otherwise be discovered by KiCad, on a file the user cannot open.

use std::path::{Path, PathBuf};

use kicli_sexpr::Doc;

use crate::model::write::{WriteOptions, WritePlan, WriteRefusal, plan_write};

/// Why a write did not happen, or did not survive its own check.
#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    /// kicli will not write this file at all.
    ///
    /// The refusals are the version ceiling and the loss of comments. Neither
    /// touches the file.
    #[error("{0}")]
    Refused(#[from] WriteRefusal),

    /// The file could not be written or renamed.
    #[error("cannot write {path}: {reason}")]
    Unwritable {
        /// The file kicli was asked to write.
        path: PathBuf,
        /// What the operating system reported.
        reason: String,
    },

    /// What was written did not read back as what was meant.
    ///
    /// The original file is untouched. This is a kicli fault, not a user one.
    #[error("what kicli wrote to {path} did not read back correctly: {reason}")]
    Unverified {
        /// The file kicli was asked to write.
        path: PathBuf,
        /// Which check failed.
        reason: String,
    },
}

/// What a completed write did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Written {
    /// The file that now holds the new bytes.
    pub path: PathBuf,
    /// Was the whole file laid out again, because it did not arrive canonical?
    pub reformatted: bool,
    /// Why it was reformatted, when it was.
    pub reason: Option<String>,
    /// How many bytes the file now holds.
    pub bytes: usize,
}

/// How the bytes reach the disk. Tests substitute a failing writer.
///
/// This is the seam that lets a test prove the original file survives a failed
/// write without corrupting a real file to do it.
pub trait Sink {
    /// Write `bytes` to `path`, replacing anything there.
    ///
    /// # Errors
    ///
    /// Returns the reason the write did not happen.
    fn write(&self, path: &Path, bytes: &[u8]) -> Result<(), String>;

    /// Rename `from` over `to`.
    ///
    /// # Errors
    ///
    /// Returns the reason the rename did not happen.
    fn rename(&self, from: &Path, to: &Path) -> Result<(), String>;

    /// Delete a file, ignoring the case where it is already gone.
    fn discard(&self, path: &Path);
}

/// The filesystem.
#[derive(Clone, Copy, Debug, Default)]
pub struct FileSystem;

impl Sink for FileSystem {
    fn write(&self, path: &Path, bytes: &[u8]) -> Result<(), String> {
        std::fs::write(path, bytes).map_err(|error| error.to_string())
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<(), String> {
        std::fs::rename(from, to).map_err(|error| error.to_string())
    }

    fn discard(&self, path: &Path) {
        let _ = std::fs::remove_file(path);
    }
}

/// Write a document over a file, atomically.
///
/// # Errors
///
/// Returns [`WriteError`] when kicli refuses the file, when the bytes cannot be
/// written, or when what was written does not read back as what was meant. In
/// every case the target file is left exactly as it was.
pub fn write_document(
    doc: &Doc,
    path: &Path,
    options: WriteOptions,
) -> Result<Written, WriteError> {
    write_document_with(doc, path, options, &FileSystem)
}

/// Write a document over a file through a given sink.
///
/// # Errors
///
/// The same as [`write_document`].
pub fn write_document_with(
    doc: &Doc,
    path: &Path,
    options: WriteOptions,
    sink: &dyn Sink,
) -> Result<Written, WriteError> {
    // A refusal happens before anything is written, so a file kicli will not
    // write is a file kicli does not touch.
    let plan: WritePlan = plan_write(doc, options)?;
    let bytes = plan.bytes.clone();

    // The temporary sits beside the target so that the rename stays inside one
    // filesystem, which is what makes it atomic.
    let temporary = temporary_beside(path);
    sink.write(&temporary, bytes.as_bytes())
        .map_err(|reason| WriteError::Unwritable {
            path: temporary.clone(),
            reason,
        })?;

    if let Err(reason) = verify(&temporary, doc, sink) {
        sink.discard(&temporary);
        return Err(WriteError::Unverified {
            path: path.to_owned(),
            reason,
        });
    }

    sink.rename(&temporary, path).map_err(|reason| {
        sink.discard(&temporary);
        WriteError::Unwritable {
            path: path.to_owned(),
            reason,
        }
    })?;

    Ok(Written {
        path: path.to_owned(),
        reformatted: plan.reformatted,
        reason: plan.reason,
        bytes: bytes.len(),
    })
}

/// Read back what was written and check it says what the tree says.
fn verify(temporary: &Path, doc: &Doc, sink: &dyn Sink) -> Result<(), String> {
    let written = std::fs::read_to_string(temporary).map_err(|error| error.to_string())?;
    let reread = Doc::parse(&written).map_err(|error| format!("it does not parse: {error}"))?;
    if !doc.structurally_eq(&reread) {
        return Err("it holds different tokens from the tree it came from".to_owned());
    }
    if reread.emit() != written {
        return Err("it is not a fixed point of the writer".to_owned());
    }
    let _ = sink;
    Ok(())
}

/// The temporary file's path: beside the target, and out of the way.
fn temporary_beside(path: &Path) -> PathBuf {
    let name = path.file_name().map_or_else(
        || "schematic".to_owned(),
        |name| name.to_string_lossy().into_owned(),
    );
    path.with_file_name(format!(".{name}.kicli-tmp"))
}

#[cfg(test)]
mod tests {
    use super::temporary_beside;
    use std::path::Path;

    #[test]
    fn the_temporary_sits_beside_the_target() {
        let temporary = temporary_beside(Path::new("/project/board.kicad_sch"));
        assert_eq!(
            temporary.parent(),
            Path::new("/project/board.kicad_sch").parent()
        );
        assert_eq!(
            temporary
                .file_name()
                .map(|name| name.to_string_lossy().into_owned()),
            Some(".board.kicad_sch.kicli-tmp".to_owned())
        );
    }
}
