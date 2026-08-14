//! A field is movable text, with its own position, angle and justification.
//!
//! Its position is absolute schematic space, so a move is a direct edit. Every
//! command that sets one clears `fields_autoplaced`, or KiCad places the field
//! again on its next open and discards the work.

use kicli::edit::field::{
    FieldAddress, Horizontal, Justification, Vertical, justify, locate, move_to, rotate_to,
};
use kicli::geometry::{Angle, GRID, Iu, Point};
use kicli::model::{Schematic, SheetPath, Target, Uuid, WriteOptions};
use kicli_sexpr::Doc;
use std::path::{Path, PathBuf};

use kicli_probe::scratch::Fixtures;

/// The committed fixtures this binary reads, and the scratch it writes in.
fn fixtures() -> Fixtures {
    Fixtures::new(env!("CARGO_TARGET_TMPDIR"), env!("CARGO_MANIFEST_DIR"))
}

/// The global label of the fixture, which owns an `Intersheetrefs` field and
/// carries the autoplace flag.
const GLOBAL_LABEL: &str = "00000000-0000-4000-8000-00000000001b";

/// The field the global label owns.
const INTERSHEET: &str = "Intersheetrefs";

/// The sheet path of the fixture, which is a root sheet.
const ROOT: &str = "/00000000-0000-4000-8000-000000000002";

/// Copy the fixture into a scratch directory and hand back the copy.
fn scratch_copy(name: &str) -> PathBuf {
    fixtures().scratch_file(name, "sch/item_zoo.kicad_sch")
}

fn read(path: &Path) -> Doc {
    Doc::parse(&std::fs::read_to_string(path).expect("the file reads")).expect("it parses")
}

fn address() -> FieldAddress {
    FieldAddress {
        owner: Uuid(GLOBAL_LABEL.to_owned()),
        name: INTERSHEET.to_owned(),
    }
}

fn target<'a>(file: &'a Path, project: &'a Path, sheet_path: &'a SheetPath) -> Target<'a> {
    Target {
        path: file,
        project,
        sheet_path,
        grid: GRID,
        options: WriteOptions::default(),
    }
}

/// The lines of the `Intersheetrefs` property, as the file now holds them.
fn field_block(text: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut depth = 0_usize;
    for line in text.lines() {
        if line
            .trim_start()
            .starts_with(&format!("(property \"{INTERSHEET}\""))
        {
            depth = 1;
        }
        if depth == 0 {
            continue;
        }
        lines.push(line);
        depth += line.matches('(').count();
        depth -= line.matches(')').count().min(depth);
        if depth <= 1 && line.trim_start().starts_with(')') {
            break;
        }
    }
    assert!(!lines.is_empty(), "the file still holds the field");
    lines
}

/// Read the justification the file now carries.
fn justification_of(file: &Path) -> Justification {
    let doc = read(file);
    let schematic = Schematic::read(&doc).expect("the file reads as a schematic");
    let located = locate(&schematic, &address()).expect("the field is there");
    Justification::read(&doc, located.property)
}

#[test]
fn moving_a_field_clears_the_autoplace_flag() {
    for (case, apply) in commands() {
        let file = scratch_copy(&format!("edit_field_autoplace_{case}"));
        let project = file.parent().expect("the copy has a directory").to_owned();
        let before = std::fs::read_to_string(&file).expect("the file reads");
        assert!(
            before.contains("(fields_autoplaced yes)"),
            "the fixture starts with the flag set"
        );

        let path = SheetPath(ROOT.to_owned());
        let mut doc = read(&file);
        let mutation = apply(&mut doc, &target(&file, &project, &path));

        assert!(
            mutation.invariants.passed(),
            "{case}: every invariant held: {:?}",
            mutation.invariants.failures().collect::<Vec<_>>()
        );
        let after = std::fs::read_to_string(&file).expect("the file reads");
        assert!(
            !after.contains("fields_autoplaced"),
            "{case}: the flag is gone, so KiCad keeps the placement"
        );
    }
}

/// The three commands that place field text, each as one call.
type Command = fn(&mut Doc, &Target<'_>) -> kicli::model::Mutation;

fn commands() -> Vec<(&'static str, Command)> {
    vec![
        ("move", |doc, target| {
            move_to(
                doc,
                target,
                &address(),
                Point::new(889_000, 431_800),
                "2026-01-02T03:04:05Z",
            )
            .expect("the field moves")
        }),
        ("rotate", |doc, target| {
            rotate_to(doc, target, &address(), Angle(90), "2026-01-02T03:04:05Z")
                .expect("the field turns")
        }),
        ("justify", |doc, target| {
            justify(
                doc,
                target,
                &address(),
                Justification {
                    horizontal: Horizontal::Right,
                    vertical: Vertical::Top,
                },
                "2026-01-02T03:04:05Z",
            )
            .expect("the field is justified")
        }),
    ]
}

#[test]
fn a_field_is_not_snapped_to_the_grid() {
    let file = scratch_copy("edit_field_off_grid");
    let project = file.parent().expect("the copy has a directory").to_owned();
    let path = SheetPath(ROOT.to_owned());
    let mut doc = read(&file);

    // KiCad's own autoplacement lands fields on units like this one, so
    // snapping them would fight the editor.
    let odd = Point::new(851_122, 431_800);
    assert!(!odd.x.is_on_grid(), "the position is off the grid");

    let mutation = move_to(
        &mut doc,
        &target(&file, &project, &path),
        &address(),
        odd,
        "2026-01-02T03:04:05Z",
    )
    .expect("the field moves");

    assert!(
        mutation.invariants.passed(),
        "an off-grid field raises no finding: {:?}",
        mutation.invariants.failures().collect::<Vec<_>>()
    );

    let after = std::fs::read_to_string(&file).expect("the file reads");
    let block = field_block(&after);
    assert!(
        block
            .iter()
            .any(|line| line.contains("(at 85.1122 43.18 0)")),
        "the field stays exactly where it was put: {block:?}"
    );
    assert_eq!(
        Iu::from_millimetres_text("85.1122"),
        Some(odd.x),
        "and the written text is the position kicli was given"
    );
}

#[test]
fn justification_survives_a_round_trip() {
    for horizontal in Horizontal::ALL {
        for vertical in Vertical::ALL {
            let wanted = Justification {
                horizontal,
                vertical,
            };
            let file = scratch_copy(&format!("edit_field_justify_{horizontal:?}_{vertical:?}"));
            let project = file.parent().expect("the copy has a directory").to_owned();
            let path = SheetPath(ROOT.to_owned());
            let mut doc = read(&file);

            justify(
                &mut doc,
                &target(&file, &project, &path),
                &address(),
                wanted,
                "2026-01-02T03:04:05Z",
            )
            .expect("the field is justified");

            assert_eq!(
                justification_of(&file),
                wanted,
                "the pair reads back as it was written"
            );

            // The token form is KiCad's own: a centred edge writes no token,
            // and a centred pair writes no list at all. Every form below is
            // one KiCad's demo files carry.
            let after = std::fs::read_to_string(&file).expect("the file reads");
            let written: Vec<&str> = field_block(&after)
                .into_iter()
                .map(str::trim)
                .filter(|line| line.starts_with("(justify"))
                .collect();
            assert_eq!(written, tokens_for(wanted), "for {wanted:?}");
        }
    }
}

/// The line KiCad writes for one pair, or no line at all.
fn tokens_for(justification: Justification) -> Vec<&'static str> {
    match (justification.horizontal, justification.vertical) {
        (Horizontal::Center, Vertical::Center) => vec![],
        (Horizontal::Left, Vertical::Center) => vec!["(justify left)"],
        (Horizontal::Right, Vertical::Center) => vec!["(justify right)"],
        (Horizontal::Center, Vertical::Top) => vec!["(justify top)"],
        (Horizontal::Center, Vertical::Bottom) => vec!["(justify bottom)"],
        (Horizontal::Left, Vertical::Top) => vec!["(justify left top)"],
        (Horizontal::Left, Vertical::Bottom) => vec!["(justify left bottom)"],
        (Horizontal::Right, Vertical::Top) => vec!["(justify right top)"],
        (Horizontal::Right, Vertical::Bottom) => vec!["(justify right bottom)"],
    }
}
