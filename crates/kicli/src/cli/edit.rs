//! The commands that write, at the command boundary.
//!
//! Every one of them opens the project, decides which file and which sheet path
//! it is editing, hands the change to [`crate::edit`], and prints the mutation
//! result. The write itself, the invariants and the `@last-write` snapshot all
//! happen below this layer, so no command here can forget one of them.
//!
//! Two rules shape this module. A command names one sheet placement, because a
//! reference designator and a drawn unit are properties of the sheet path and
//! not of the symbol. And an error is a row of the exit-code table before it is
//! a message, so a caller reads the code and knows what kind of thing failed.

pub mod address;
pub mod field;
pub mod label;
pub mod mark;
pub mod net;
pub mod symbol;
pub mod text;

use super::args::Global;
use super::exit::ExitCode;
use super::locate::Loaded;
use super::output::{Failure, Report};
use crate::geometry::Iu;
use crate::model::hierarchy::Hierarchy;
use crate::model::items::{Schematic, SheetPath};
use crate::model::mutate::{Mutation, MutationError, Target, commit, state_before};
use crate::model::write::{WriteOptions, WriteRefusal};
use crate::model::write_file::WriteError;
use crate::view::snapshot::{Snapshot, SnapshotError};
use kicli_sexpr::Doc;
use serde_json::Value;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// One project, opened for editing, with the placement a command works on.
pub struct Editing {
    /// The project, with every file of the sheet tree parsed.
    pub loaded: Loaded,
    /// Which file of the tree the command edits.
    pub file: usize,
    /// Where the write lands, and under what rules.
    pub place: Place,
    /// The timestamp every snapshot of this run records.
    pub taken: String,
}

/// Where a write lands, and under what rules.
///
/// The values are owned rather than borrowed from the project, so a command can
/// hold this and the tree it is changing at the same time.
pub struct Place {
    /// The file to write.
    path: PathBuf,
    /// The project directory, which holds the snapshot cache.
    project: PathBuf,
    /// The sheet path the file is being edited as.
    sheet_path: SheetPath,
    /// The placement grid.
    grid: Iu,
    /// What kicli may and may not write.
    options: WriteOptions,
}

impl Place {
    /// The target the editing commands take.
    #[must_use]
    pub fn target(&self) -> Target<'_> {
        Target {
            path: &self.path,
            project: &self.project,
            sheet_path: &self.sheet_path,
            grid: self.grid,
            options: self.options,
        }
    }

    /// The sheet path the command is editing on.
    #[must_use]
    pub const fn sheet_path(&self) -> &SheetPath {
        &self.sheet_path
    }

    /// The placement grid.
    #[must_use]
    pub const fn grid(&self) -> Iu {
        self.grid
    }

    /// What kicli may and may not write.
    #[must_use]
    pub const fn options(&self) -> WriteOptions {
        self.options
    }
}

impl Editing {
    /// Open the project a command was pointed at, at one sheet placement.
    ///
    /// Without `--sheet` the placement is the root sheet, which is the one a
    /// caller standing in a project means.
    ///
    /// # Errors
    ///
    /// Returns a [`Failure`] when the project does not read, or when `--sheet`
    /// names a path this project does not have.
    pub fn open(global: &Global) -> Result<Self, Failure> {
        let loaded = Loaded::for_command(global)?;
        let index = placement_of(&loaded, global.sheet.as_deref())?;
        let placement = &loaded.hierarchy.placements[index];
        let file = placement.file;

        let place = Place {
            path: loaded.hierarchy.files[file].path.clone(),
            project: loaded.directory.clone(),
            sheet_path: placement.path.clone(),
            grid: loaded.config.grid.step,
            options: WriteOptions {
                allow_comment_loss: global.allow_comment_loss,
                max_version: loaded.config.formats.max_schematic_version,
            },
        };

        Ok(Self {
            loaded,
            file,
            place,
            taken: now(),
        })
    }

    /// The tree of the file being edited.
    pub fn doc(&mut self) -> &mut Doc {
        &mut self.loaded.hierarchy.files[self.file].doc
    }

    /// The typed objects of the file being edited.
    #[must_use]
    pub fn schematic(&self) -> &Schematic {
        &self.loaded.hierarchy.files[self.file].schematic
    }

    /// The root schematic of the project, which the nets are read from.
    #[must_use]
    pub fn root(&self) -> PathBuf {
        self.loaded.root.clone()
    }

    /// The tree, the target and the timestamp at once.
    ///
    /// The editing commands that write for themselves need all three together.
    /// They come from one borrow of this value, so the tree can be changed while
    /// the target that says where it lands is in hand.
    pub fn parts(&mut self) -> (&mut Doc, Target<'_>, &str) {
        let Self {
            loaded,
            file,
            place,
            taken,
        } = self;
        (
            &mut loaded.hierarchy.files[*file].doc,
            place.target(),
            taken,
        )
    }

    /// The whole sheet tree, the target and the timestamp at once.
    ///
    /// A mark and a rename are only meaningful against the project they sit in,
    /// so those commands take the tree rather than one file of it.
    pub fn tree_parts(&mut self) -> (&mut Hierarchy, Target<'_>, &str) {
        let Self {
            loaded,
            place,
            taken,
            ..
        } = self;
        (&mut loaded.hierarchy, place.target(), taken)
    }

    /// The state to compare a change against, taken before the change.
    ///
    /// # Errors
    ///
    /// Returns [`SnapshotError`] when the file cannot be hashed.
    pub fn state(&self) -> Result<Snapshot, SnapshotError> {
        let file = &self.loaded.hierarchy.files[self.file];
        state_before(
            &file.doc,
            &file.schematic,
            self.place.sheet_path(),
            &self.taken,
        )
    }

    /// Run the invariants over the changed tree, write it, and report it.
    ///
    /// # Errors
    ///
    /// Returns [`MutationError`] when an invariant fails or the write does not
    /// happen. Nothing is written unless every invariant holds.
    pub fn commit(&self, before: &Snapshot) -> Result<Mutation, MutationError> {
        commit(
            &self.loaded.hierarchy.files[self.file].doc,
            &self.place.target(),
            before,
            &self.taken,
        )
    }
}

/// Which placement a command works on.
fn placement_of(loaded: &Loaded, sheet: Option<&str>) -> Result<usize, Failure> {
    let Some(wanted) = sheet else {
        // The tree is loaded root first, so index zero is the root sheet.
        return Ok(0);
    };
    let wanted = SheetPath(wanted.to_owned());
    loaded
        .hierarchy
        .placements
        .iter()
        .position(|placement| placement.path == wanted)
        .ok_or_else(|| {
            Failure::new(
                ExitCode::Usage,
                format!(
                    "{} is not a sheet path of this project. Run project info to list them.",
                    wanted.0
                ),
            )
        })
}

/// The moment a run started, as the snapshot header records it.
///
/// The snapshot module never reads a clock of its own, so the reading is taken
/// here, once per run. A machine whose clock is before 1970 reports the epoch
/// rather than failing a write over a timestamp.
fn now() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    let (year, month, day) = civil_date(seconds / 86_400);
    let time = seconds % 86_400;
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        time / 3_600,
        (time % 3_600) / 60,
        time % 60
    )
}

/// The year, month and day of a count of days since 1970-01-01.
///
/// This is Howard Hinnant's `civil_from_days`, which is exact for every day the
/// proleptic Gregorian calendar covers. kicli has no date dependency and needs
/// one date, so the algorithm lives here rather than in a crate.
fn civil_date(days: u64) -> (u64, u64, u64) {
    let days = days + 719_468;
    let era = days / 146_097;
    let day_of_era = days % 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    (if month <= 2 { year + 1 } else { year }, month, day)
}

/// One thing a command did that the plain reading of the request does not say.
///
/// A note is not a failure. It is the part of the result a caller must feel,
/// because the command did something other than what it was asked for word for
/// word: it snapped a position, or it left half a sheet port for a later
/// command.
pub struct Note {
    /// The name a report writes for this note.
    pub name: &'static str,
    /// What happened, in one sentence.
    pub message: String,
}

impl Note {
    /// Build a note.
    #[must_use]
    pub fn new(name: &'static str, message: impl Into<String>) -> Self {
        Self {
            name,
            message: message.into(),
        }
    }
}

/// The report of one mutation: what changed, what was checked, what was noticed.
///
/// `object` names the thing the command addressed, under the noun's own key, so
/// a caller can address what was just made without parsing the delta lines.
#[must_use]
pub fn report(mutation: &Mutation, object: Option<(&str, Value)>, notes: &[Note]) -> Report {
    let mut text = mutation.render();
    for note in notes {
        let _ = writeln!(text, "note: {}  {}", note.name, note.message);
    }

    let mut json = mutation.to_json();
    if let Some(fields) = json.as_object_mut() {
        if let Some((key, value)) = object {
            fields.insert(key.to_owned(), value);
        }
        fields.insert(
            "notes".to_owned(),
            notes
                .iter()
                .map(|note| serde_json::json!({ "name": note.name, "message": note.message }))
                .collect::<Vec<Value>>()
                .into(),
        );
    }
    Report { text, json }
}

/// Which row of the table a failed write is.
///
/// A refusal and an unwritable file are file errors: the file kicli was asked
/// to write is the problem. A change that did not survive its own checks is a
/// verification failure, and by construction nothing was written.
#[must_use]
pub fn code_for(error: &MutationError) -> ExitCode {
    match error {
        MutationError::Invariant(_) => ExitCode::Verification,
        MutationError::Write(write) => match write {
            WriteError::Refused(_) | WriteError::Unwritable { .. } => ExitCode::File,
            WriteError::Unverified { .. } => ExitCode::Verification,
        },
        MutationError::Snapshot(_) => ExitCode::File,
    }
}

/// Which row of the table a failed snapshot is.
#[must_use]
pub const fn code_for_snapshot(_: &SnapshotError) -> ExitCode {
    ExitCode::File
}

/// Which row of the table a refusal to write a file at all is.
#[must_use]
pub const fn code_for_refusal(_: &WriteRefusal) -> ExitCode {
    ExitCode::File
}

#[cfg(test)]
mod tests {
    use super::{civil_date, now};

    #[test]
    fn the_timestamp_is_one_word_a_snapshot_accepts() {
        let stamp = now();
        assert_eq!(stamp.len(), 20, "{stamp} is an ISO 8601 instant");
        assert!(stamp.ends_with('Z'), "{stamp} is in UTC");
        assert!(!stamp.contains(' '), "{stamp} is one word");
        assert!(!stamp.contains('/'), "{stamp} is not a path");
    }

    #[test]
    fn the_calendar_agrees_with_days_it_is_known_for() {
        assert_eq!(civil_date(0), (1970, 1, 1), "the epoch itself");
        assert_eq!(civil_date(59), (1970, 3, 1), "1970 is not a leap year");
        assert_eq!(civil_date(365), (1971, 1, 1), "the year after");
        assert_eq!(civil_date(11_016), (2000, 2, 29), "2000 is a leap year");
        assert_eq!(civil_date(20_679), (2026, 8, 14), "a day past the century");
    }
}
