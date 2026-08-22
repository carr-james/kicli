//! The `sch pins` command.
//!
//! It answers where a placed symbol's pins are and what may be connected to
//! each one. **It is read-only, and that is a property worth stating rather
//! than assuming**: it opens the project through [`Loaded`], which reads, and
//! never through `Editing`, which is the only path in this crate that reaches
//! a write. `tests/pin_view_writes_nothing.rs` measures the claim over the
//! whole command rather than over this sentence.
//!
//! The question it answers is the one an agent has just before it edits, so
//! the answer is shaped to be acted on: the escape point of each record is
//! printed in the exact `x,y` token `--to-at` parses.

use crate::cli::args::{Global, PinsArgs};
use crate::cli::edit::address;
use crate::cli::exit::ExitCode;
use crate::cli::locate::Loaded;
use crate::cli::output::{Failure, Report, Reporter};
use crate::connectivity::extract;
use crate::model::items::SheetPath;
use crate::view::pins::{Listing, Pins, render, to_json};

/// Report on a symbol's pins.
///
/// # Errors
///
/// Returns a [`Failure`] when the project cannot be found or read, when the
/// sheet flag names a placement this project does not have, when nothing on
/// that sheet answers to the target, or when the symbol has no pin of the
/// number asked for.
pub fn pins(global: &Global, args: &PinsArgs, reporter: &Reporter) -> Result<Report, Failure> {
    let loaded = Loaded::for_command(global)?;
    let placement = placement_of(&loaded, global.sheet.as_deref())?;
    let path = loaded.hierarchy.placements[placement].path.clone();
    let file = &loaded.hierarchy.files[loaded.hierarchy.placements[placement].file];

    let (named, number) = args.parts();
    let symbol = address::symbol(&file.schematic, &path, named)?;
    let nets = extract(&loaded.hierarchy);
    let answer = Pins::of(file, &path, symbol, &nets, number, loaded.config.grid.step)
        .map_err(|error| Failure::new(ExitCode::Operation, error.to_string()))?;

    let answer = if args.free {
        answer.only_free()
    } else {
        answer
    };
    let rendered = render(&answer, loaded.config.view.max_bytes);
    if rendered.listing == Listing::Summary {
        reporter.note(&format!(
            "{} has {} pins, which is more than the {} byte budget, so this is the count",
            answer.reference, answer.total, loaded.config.view.max_bytes
        ));
    }

    let mut json = to_json(&answer, rendered.listing);
    if args.stats {
        json["bytes"] = rendered.bytes.into();
        return Ok(Report {
            text: format!("{}# {} bytes\n", rendered.text, rendered.bytes),
            json,
        });
    }
    Ok(Report {
        text: rendered.text,
        json,
    })
}

/// Which placement the command reads.
///
/// The same rule every editing verb follows: `--sheet` names it, and without it
/// the root sheet answers. Stated here rather than borrowed from `cli::edit`,
/// because reaching into the editing layer for a read is how a read-only
/// command grows a write path.
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
