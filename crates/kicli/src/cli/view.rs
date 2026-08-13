//! The `sch view` command.
//!
//! It prints one of the structured views of a project. A view is what an agent
//! acts on, so the terse form is the default and JSON is printed only when it
//! is asked for.

use crate::cli::args::{Global, ViewName};
use crate::cli::exit::ExitCode;
use crate::cli::locate::Loaded;
use crate::cli::output::{Failure, Report, Reporter};
use crate::connectivity::extract;
use crate::model::items::SheetPath;
use crate::view::connectivity::ViewOptions;
use crate::view::{Kind, Scope, connectivity, layout, scope};

/// Print a view of the project.
///
/// # Errors
///
/// Returns a [`Failure`] when the project cannot be found or read, or when the
/// sheet flag names a placement this project does not have.
pub fn view(
    global: &Global,
    which: ViewName,
    include_power: bool,
    uuids: bool,
    stats: bool,
    reporter: &Reporter,
) -> Result<Report, Failure> {
    let loaded = Loaded::for_command(global)?;

    let sheet = match &global.sheet {
        None => None,
        Some(path) => {
            let wanted = SheetPath(path.clone());
            if !loaded
                .hierarchy
                .placements
                .iter()
                .any(|placement| placement.path == wanted)
            {
                return Err(Failure::new(
                    ExitCode::Usage,
                    format!("no sheet of this project has the path {path}"),
                ));
            }
            Some(wanted)
        }
    };

    let options = ViewOptions {
        include_power,
        uuids,
        sheet,
    };
    let nets = extract(&loaded.hierarchy);
    let kind = match which {
        ViewName::Connectivity => Kind::Connectivity,
        ViewName::Layout => Kind::Layout,
    };
    let rendered = scope::render(
        kind,
        &loaded.hierarchy,
        &nets,
        &options,
        loaded.config.view.max_bytes,
    );

    if rendered.scope == Scope::IndexAndSummaries {
        reporter.note(&format!(
            "the whole project needs more than the {} byte budget, so this is the index",
            loaded.config.view.max_bytes
        ));
    }

    let mut json = match kind {
        Kind::Connectivity => connectivity::to_json(&loaded.hierarchy, &nets, &options),
        Kind::Layout => layout::to_json(&loaded.hierarchy, &options),
    };
    json["scope"] = rendered.scope.token().into();
    if stats {
        json["bytes"] = rendered.bytes.into();
    }

    let text = if stats {
        format!("{}# {} bytes\n", rendered.text, rendered.bytes)
    } else {
        rendered.text.clone()
    };
    Ok(Report { text, json })
}
