//! How much of a project a view covers, and what it does when that is too much.
//!
//! A view defaults to the whole project. When the whole project would not fit
//! the byte budget, the view falls back to an index and per-sheet summaries.
//! Either way the output says which of the two it is, on its first line, so a
//! reader never has to guess whether it is holding everything.

use std::fmt::Write as _;

use crate::connectivity::Nets;
use crate::model::Hierarchy;
use crate::view::connectivity::ViewOptions;
use crate::view::{connectivity, layout};

/// Which of the three views to render.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// What is joined to what.
    Connectivity,
    /// Where things are drawn.
    Layout,
}

/// How much of the project the output covers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    /// Every record of every sheet.
    WholeProject,
    /// One sheet, because the caller asked for one.
    OneSheet,
    /// An index and per-sheet counts, because everything would not fit.
    IndexAndSummaries,
    /// A summary of one sheet, because that sheet alone would not fit.
    SheetSummary,
}

impl Scope {
    /// The word the output uses for this scope.
    #[must_use]
    pub fn token(self) -> &'static str {
        match self {
            Scope::WholeProject => "project",
            Scope::OneSheet => "sheet",
            Scope::IndexAndSummaries => "index",
            Scope::SheetSummary => "sheet-summary",
        }
    }
}

/// A rendered view, and what it turned out to be.
#[derive(Clone, Debug)]
pub struct Rendered {
    /// The text of the view.
    pub text: String,
    /// How much of the project it covers.
    pub scope: Scope,
    /// How many bytes the text is. Bytes, not tokens: counting tokens needs a
    /// tokenizer, and a caller can count its own.
    pub bytes: usize,
}

/// Render a view, falling back to an index when the whole project is too big.
///
/// `max_bytes` is the budget from the project's configuration.
#[must_use]
pub fn render(
    kind: Kind,
    hierarchy: &Hierarchy,
    nets: &Nets,
    options: &ViewOptions,
    max_bytes: usize,
) -> Rendered {
    let full = match kind {
        Kind::Connectivity => connectivity::render(hierarchy, nets, options),
        Kind::Layout => layout::render(hierarchy, options),
    };
    let asked_for_one_sheet = options.sheet.is_some();

    if full.len() <= max_bytes {
        let scope = if asked_for_one_sheet {
            Scope::OneSheet
        } else {
            Scope::WholeProject
        };
        return Rendered {
            bytes: full.len(),
            text: full,
            scope,
        };
    }

    // One sheet can be too big on its own. A bank of connector pins is a
    // handful of symbols and several hundred nets, so the fallback belongs at
    // the sheet as much as at the project.
    let scope = if asked_for_one_sheet {
        Scope::SheetSummary
    } else {
        Scope::IndexAndSummaries
    };
    let text = index(hierarchy, nets, options, max_bytes, full.len(), scope);
    Rendered {
        bytes: text.len(),
        text,
        scope,
    }
}

/// The index form: one line per sheet, and no records.
///
/// It names the budget it did not fit and how big the full view would have
/// been, so the caller can raise the budget or ask for one sheet instead of
/// guessing.
fn index(
    hierarchy: &Hierarchy,
    nets: &Nets,
    options: &ViewOptions,
    max_bytes: usize,
    would_be: usize,
    scope: Scope,
) -> String {
    let wanted: Vec<&crate::model::Placement> = hierarchy
        .placements
        .iter()
        .filter(|placement| {
            options
                .sheet
                .as_ref()
                .is_none_or(|path| &placement.path == path)
        })
        .collect();

    let mut out = String::new();
    let _ = writeln!(
        out,
        "# scope {}  sheets={}  full={would_be}B budget={max_bytes}B",
        scope.token(),
        wanted.len()
    );
    let _ = writeln!(
        out,
        "# raise view.max_bytes to see the records{}",
        if scope == Scope::IndexAndSummaries {
            ", or ask for one sheet with --sheet <path>"
        } else {
            ""
        }
    );

    for placement in wanted {
        let file = &hierarchy.files[placement.file];
        let placed: Vec<_> = file
            .schematic
            .symbols()
            .filter(|symbol| symbol.reference_on(&placement.path).is_some())
            .collect();
        let power = placed.iter().filter(|symbol| symbol.is_power()).count();
        let listed = placed.len() - power;
        let nets_here = nets
            .nets()
            .iter()
            .filter(|net| net.sheets.contains(&placement.path))
            .count();
        let _ = writeln!(
            out,
            "I {} {} sym={listed} pwr={power} nets={nets_here}",
            placement.path.0,
            placement.name.as_deref().unwrap_or("/")
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::Scope;

    #[test]
    fn a_scope_has_one_word_for_itself() {
        assert_eq!(Scope::WholeProject.token(), "project");
        assert_eq!(Scope::OneSheet.token(), "sheet");
        assert_eq!(Scope::IndexAndSummaries.token(), "index");
    }
}
