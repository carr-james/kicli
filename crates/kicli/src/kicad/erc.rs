//! Running KiCad's electrical rule check, and never trusting its coordinates.
//!
//! `kicad-cli sch erc` writes two reports of one run. The JSON one carries each
//! violation item's **identifier**, which joins straight to kicli's own object
//! handles, and the text one carries none. The JSON one also reports every
//! schematic coordinate **a hundred times too small while labelling it `mm`**.
//! So kicli reads both: the JSON for what the violation is and which object it
//! is about, the text for a control on the numbers.
//!
//! # The bug, and what kicli does about it
//!
//! KiCad 10.0.5 builds the JSON exporter's units provider with `pcbIUScale`
//! (1e6 internal units per millimetre) where the text exporter uses
//! `schIUScale` (1e4) — `eeschema/erc/erc_report.cpp:161` against `:63`,
//! `include/base_units.h:72,111-114`. Unfixed on `master`.
//!
//! kicli therefore reads the JSON number back **at the scale KiCad divided
//! it by**: multiplying by `pcbIUScale` recovers the schematic internal units
//! exactly, which is the ×100 correction `spec/SPEC.md` §14.2 requirement 1
//! permits. It is exact integer arithmetic and no coordinate here goes near a
//! float — [`corrected`] is the whole of it, and it carries the citation.
//!
//! # The correction is sanity-checked on every run, not only in a test
//!
//! §14.2's warning is that the day upstream fixes this, the correction must be
//! **removed, never double-applied** — a coordinate wrong by 10,000× in a tool
//! whose job is where things are drawn. A test alone would catch that in CI and
//! not on the machine of someone running a fixed KiCad, so [`scale_verdict`]
//! re-decides it from the two reports on **every run**, and a run that no
//! longer shows the bug **fails loudly** rather than reporting a corrected
//! coordinate.
//!
//! `cargo test -p kicli --test erc_canary` is the committed-fixture canary
//! §14.2 requirement 2 asks for, and it calls the same function.
//!
//! # Two measurements that shaped this, both against KiCad 10.0.5
//!
//! **The text report is lossy and the JSON is not.** The text report rounds a
//! millimetre reading to three decimals and then strips trailing zeros to a
//! floor of two, so `25.4321 mm` prints as `25.432` — one decimal short of the
//! schematic's own 1e-4 mm resolution. The JSON's number, multiplied back by
//! 1e6, is exact. Measured on `tests/fixtures/erc/precision`, which is
//! committed for the purpose. **This is why the corrected JSON is the position
//! and the text report is only the control**: reading positions from the text
//! would throw away a digit of every coordinate kicli owns.
//!
//! **The scale error is in the descriptions too.** The same wrong units
//! provider formats the item descriptions, so the JSON calls a 12.70 mm wire
//! `Horizontal Wire, length 0.1270 mm`. kicli takes each item's description
//! from the text report for that reason.

use std::path::Path;

use crate::geometry::{Iu, Point};
use crate::kicad::{CliFailure, KicadCli, PROGRAM, Runner};
use crate::lint::erc::{Item, KicadSeverity, RuleCheck, Violation};
use crate::model::items::{SheetPath, Uuid};

/// Internal units per millimetre the JSON exporter divided by, in error.
///
/// This is KiCad's `pcbIUScale`. A schematic's own scale is
/// [`crate::geometry::UNITS_PER_MM`], which is a hundred times smaller, and the
/// ratio of the two is the whole bug.
const JSON_UNITS_PER_MM: i64 = 1_000_000;

/// The same scale as a count of decimal places, which is how a decimal is read.
const JSON_FRACTION_DIGITS: u32 = JSON_UNITS_PER_MM.ilog10();

/// The smallest step the text report can express, in schematic internal units.
///
/// The text report rounds a millimetre reading to three decimals, and a
/// schematic internal unit is 1e-4 mm, so the report's steps are ten units
/// apart. Measured on `tests/fixtures/erc/precision`.
const TEXT_STEP: i32 = 10;

/// How far a text reading may sit from the exact coordinate it rounded.
const TEXT_SLACK: i32 = TEXT_STEP / 2;

/// How the JSON report's coordinates stand to the text report's.
///
/// The verdict decides whether the ×100 correction is applied or refused. It is
/// read on every run, so the day upstream fixes `erc_report.cpp:161` kicli
/// stops rather than doubling the correction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScaleVerdict {
    /// The JSON is a hundred times small, which is the bug kicli corrects.
    HundredTimesSmall,
    /// The JSON already agrees with the text report: upstream has fixed it.
    ///
    /// The correction must be **removed**, not applied. Nothing in kicli may
    /// treat this as a variant of the bug.
    Fixed,
    /// Nothing in the two reports distinguishes the two readings.
    ///
    /// A report with no violations, or one whose every coordinate is the
    /// origin, says nothing about the scale either way. Recorded as its own
    /// answer rather than folded into [`Self::HundredTimesSmall`], because a
    /// check that reads nothing and reports success is the commonest way an
    /// instrument goes blind.
    Indeterminate,
    /// The two reports do not describe the same run.
    Unrelated {
        /// What did not line up.
        detail: String,
    },
}

impl ScaleVerdict {
    /// The word a message writes for this verdict.
    #[must_use]
    pub fn word(&self) -> &'static str {
        match self {
            Self::HundredTimesSmall => "a-hundred-times-small",
            Self::Fixed => "fixed-upstream",
            Self::Indeterminate => "indeterminate",
            Self::Unrelated { .. } => "unrelated",
        }
    }
}

/// One item line of the text report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextItem {
    /// Where the text report says it is. This reading is correct and coarse.
    pub at: Point,
    /// What KiCad calls the object. This reading is correct.
    pub description: String,
}

/// Run KiCad's rule check over one sheet and read both of its reports.
///
/// `scratch` is a directory kicli may write the two reports into. Neither the
/// sheet nor the project file is written: `kicad-cli sch erc` leaves
/// `.kicad_pro` untouched (measured against 10.0.5), and §14.3 forbids kicli
/// from touching it whatever KiCad does.
///
/// # Errors
///
/// Returns the [`CliFailure`] of either call, and a [`CliFailure::Failed`] when
/// the two reports cannot be trusted together — including the case that matters
/// most, upstream having fixed the scale bug.
pub fn rule_check<R: Runner>(
    cli: &KicadCli<R>,
    sheet: &Path,
    scratch: &Path,
) -> Result<RuleCheck, CliFailure> {
    let sheet_argument = argument(sheet)?;
    let json_path = scratch.join("kicli-erc.json");
    let text_path = scratch.join("kicli-erc.txt");

    cli.run(&[
        "sch",
        "erc",
        "--format",
        "json",
        "--severity-all",
        "-o",
        argument(&json_path)?,
        sheet_argument,
    ])?;
    cli.run(&[
        "sch",
        "erc",
        "--format",
        "report",
        "--units",
        "mm",
        "--severity-all",
        "-o",
        argument(&text_path)?,
        sheet_argument,
    ])?;

    let json = read(&json_path)?;
    let text = read(&text_path)?;
    read_reports(&json, &text)
}

/// Read a pair of reports KiCad wrote, without running anything.
///
/// This is the whole of kicli's understanding of the two files, so a test drives
/// it over committed fixtures and needs no KiCad install.
///
/// # Errors
///
/// Returns [`CliFailure::Failed`] when either report does not parse, when they
/// do not describe the same run, or when the scale relation between them is no
/// longer the one kicli corrects.
pub fn read_reports(json: &str, text: &str) -> Result<RuleCheck, CliFailure> {
    let parsed = parse(json).map_err(refusal)?;
    let violations = violations_of(&parsed).map_err(refusal)?;
    let items = read_text(text);

    let mut positions: Vec<Point> = Vec::new();
    for violation in &violations {
        positions.extend(violation.items.iter().map(|item| item.at));
    }

    match scale_verdict(&positions, &items) {
        ScaleVerdict::HundredTimesSmall | ScaleVerdict::Indeterminate => {}
        ScaleVerdict::Fixed => {
            return Err(refusal(
                "KiCad's JSON rule-check report no longer reports schematic coordinates a \
                 hundred times too small, so the correction kicli applies to them is now wrong. \
                 Remove it rather than doubling it: see eeschema/erc/erc_report.cpp:161 against \
                 :63, and spec/SPEC.md \u{a7}14.2."
                    .to_owned(),
            ));
        }
        ScaleVerdict::Unrelated { detail } => {
            return Err(refusal(format!(
                "the JSON and text rule-check reports do not describe the same run, so kicli \
                 cannot check the coordinates it corrects: {detail}."
            )));
        }
    }

    Ok(RuleCheck::ran(
        described_by_the_text_report(violations, &items),
        ignored_checks(&parsed),
    ))
}

/// Which reading of the JSON's coordinates the two reports support.
///
/// `corrected` holds the JSON's positions with the ×100 correction already
/// applied, in the order the JSON reported them; `text` holds the text report's
/// items in the order it wrote them. The two orders agree, measured on both
/// committed fixtures and on the two geometry fixtures.
///
/// The verdict needs an item that **distinguishes** the two readings, so a run
/// whose every coordinate is the origin is [`ScaleVerdict::Indeterminate`]
/// rather than a confirmation of anything.
#[must_use]
pub fn scale_verdict(corrected: &[Point], text: &[TextItem]) -> ScaleVerdict {
    if corrected.len() != text.len() {
        return ScaleVerdict::Unrelated {
            detail: format!(
                "the JSON names {} item(s) and the text report {}",
                corrected.len(),
                text.len()
            ),
        };
    }

    let (mut bug_holds, mut fixed_holds) = (true, true);
    for (found, expected) in corrected.iter().zip(text) {
        bug_holds &= near(*found, expected.at);
        fixed_holds &= near(uncorrected(*found), expected.at);
    }

    match (bug_holds, fixed_holds) {
        // Both readings fit every item, so no item told them apart. That needs
        // every coordinate to be within a rounding step of a hundredth of
        // itself, which only the origin is.
        (true, true) => ScaleVerdict::Indeterminate,
        (true, false) => ScaleVerdict::HundredTimesSmall,
        (false, true) => ScaleVerdict::Fixed,
        (false, false) => ScaleVerdict::Unrelated {
            detail: "neither reading of the JSON coordinates fits the text report".to_owned(),
        },
    }
}

/// The schematic internal units a JSON coordinate stands for.
///
/// **This is the correction site.** KiCad 10.0.5 wrote this number by dividing
/// the schematic's internal units by `pcbIUScale`, 1e6 units per millimetre,
/// because `eeschema/erc/erc_report.cpp:161` builds the JSON exporter's units
/// provider with the board scale where `:63` builds the text exporter's with
/// the schematic scale, `schIUScale`, 1e4. Multiplying by 1e6 undoes exactly
/// that, and is the ×100 correction `spec/SPEC.md` §14.2 requirement 1 permits.
///
/// The arithmetic is integer and the input is the number's own text, so no
/// coordinate passes through a float (Constitution §4). A number kicli cannot
/// read exactly is refused rather than rounded.
///
/// # Examples
///
/// ```
/// use kicli::geometry::Iu;
/// use kicli::kicad::erc::corrected;
///
/// // KiCad wrote 0.254321 and meant 25.4321 mm, which is 254321 units.
/// assert_eq!(corrected("0.254321"), Some(Iu(254_321)));
/// assert_eq!(corrected("0.254"), Some(Iu(254_000)));
/// assert_eq!(corrected("1.2e-3"), None, "kicli reads plain decimals only");
/// ```
#[must_use]
pub fn corrected(number: &str) -> Option<Iu> {
    let units = scaled(number, JSON_FRACTION_DIGITS)?;
    i32::try_from(units).ok().map(Iu)
}

/// The reading a fixed KiCad would have written for the same object.
///
/// The inverse of the correction, used only to decide the verdict. It divides,
/// so it is exact only where the correction was; that is enough, because the
/// comparison it feeds carries the text report's own slack.
const fn uncorrected(at: Point) -> Point {
    Point {
        x: Iu(at.x.0 / 100),
        y: Iu(at.y.0 / 100),
    }
}

/// Is a corrected coordinate the one the text report rounded?
fn near(found: Point, reported: Point) -> bool {
    (found.x.0 - reported.x.0).abs() <= TEXT_SLACK && (found.y.0 - reported.y.0).abs() <= TEXT_SLACK
}

/// Read a plain decimal as an integer scaled by `10^digits`.
///
/// Exact or nothing: a number with more fraction digits than the scale holds,
/// an exponent, or anything that is not a decimal, returns [`None`]. Rounding
/// here would be a coordinate kicli invented.
fn scaled(text: &str, digits: u32) -> Option<i64> {
    let (sign, rest) = match text.strip_prefix('-') {
        Some(rest) => (-1i64, rest),
        None => (1i64, text),
    };
    let (whole_text, fraction_text) = match rest.split_once('.') {
        Some((whole, fraction)) => (whole, fraction),
        None => (rest, ""),
    };
    if whole_text.is_empty() && fraction_text.is_empty() {
        return None;
    }
    let places = usize::try_from(digits).ok()?;
    if fraction_text.len() > places {
        return None;
    }
    let is_digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
    if !is_digits(whole_text) || !is_digits(fraction_text) {
        return None;
    }

    let whole: i64 = if whole_text.is_empty() {
        0
    } else {
        whole_text.parse().ok()?
    };
    let mut fraction: i64 = if fraction_text.is_empty() {
        0
    } else {
        fraction_text.parse().ok()?
    };
    for _ in fraction_text.len()..places {
        fraction = fraction.checked_mul(10)?;
    }
    let scale = 10i64.checked_pow(digits)?;
    Some(sign * whole.checked_mul(scale)?.checked_add(fraction)?)
}

/// Every item line of the text report, in the order KiCad wrote them.
///
/// A line reads `@(25.40 mm, 21.59 mm): Symbol R1 Pin 1 [Passive, Line]`.
///
/// **A coordinate this reader cannot read exactly is not an item.** It is
/// dropped rather than defaulted, so the item counts disagree and
/// [`scale_verdict`] refuses the pair. Defaulting it to the origin is the shape
/// of the bug this crate already paid for once — a rejected coordinate became a
/// zero, which moved items to the origin and produced a confident net list about
/// a drawing nobody drew (`tests/measurements_are_never_defaulted.rs`).
#[must_use]
pub fn read_text(report: &str) -> Vec<TextItem> {
    let mut items = Vec::new();
    for line in report.lines().map(str::trim) {
        let Some(rest) = line.strip_prefix("@(") else {
            continue;
        };
        let Some((position, description)) = rest.split_once("): ") else {
            continue;
        };
        let Some((x, y)) = position.split_once(", ") else {
            continue;
        };
        let read = |reading: &str| Iu::from_millimetres_text(reading.trim_end_matches(" mm"));
        let (Some(x), Some(y)) = (read(x), read(y)) else {
            continue;
        };
        items.push(TextItem {
            at: Point { x, y },
            description: description.to_owned(),
        });
    }
    items
}

/// Every violation of the JSON report, with coordinates already corrected.
///
/// # Errors
///
/// Returns what is wrong with the report, as one sentence.
pub fn read_json(report: &str) -> Result<Vec<Violation>, String> {
    violations_of(&parse(report)?)
}

/// The report as a tree, or what is wrong with it.
fn parse(report: &str) -> Result<serde_json::Value, String> {
    serde_json::from_str(report).map_err(|error| format!("the JSON report is not JSON: {error}"))
}

/// Every violation of a parsed report, with coordinates already corrected.
fn violations_of(parsed: &serde_json::Value) -> Result<Vec<Violation>, String> {
    let sheets = parsed["sheets"]
        .as_array()
        .ok_or_else(|| "the JSON report names no sheets".to_owned())?;

    let mut violations = Vec::new();
    for sheet in sheets {
        let path = SheetPath(
            sheet["uuid_path"]
                .as_str()
                .ok_or_else(|| "a sheet of the JSON report has no uuid path".to_owned())?
                .to_owned(),
        );
        let reported = sheet["violations"]
            .as_array()
            .ok_or_else(|| "a sheet of the JSON report names no violations".to_owned())?;
        for violation in reported {
            let check = text_of(&violation["type"], "a violation has no type")?;
            let word = text_of(&violation["severity"], "a violation has no severity")?;
            let severity = KicadSeverity::read(&word).ok_or_else(|| {
                format!("{PROGRAM} reported a severity kicli does not know: {word}")
            })?;
            let mut items = Vec::new();
            for item in violation["items"].as_array().unwrap_or(&Vec::new()) {
                items.push(read_item(item)?);
            }
            violations.push(Violation {
                check,
                severity,
                description: text_of(&violation["description"], "a violation has no description")?,
                sheet: path.clone(),
                items,
            });
        }
    }
    Ok(violations)
}

/// One item of the JSON report, with its coordinate corrected.
fn read_item(item: &serde_json::Value) -> Result<Item, String> {
    let coordinate = |axis: &str| -> Result<Iu, String> {
        let number = item["pos"][axis]
            .as_number()
            .ok_or_else(|| format!("an item's {axis} is not a number"))?
            .to_string();
        corrected(&number)
            .ok_or_else(|| format!("an item's {axis} is {number}, which kicli cannot read exactly"))
    };
    Ok(Item {
        uuid: item["uuid"].as_str().map(|text| Uuid(text.to_owned())),
        description: item["description"].as_str().unwrap_or_default().to_owned(),
        at: Point {
            x: coordinate("x")?,
            y: coordinate("y")?,
        },
    })
}

/// The keys of the checks the project turned off.
///
/// A report with no `ignored_checks` list turned nothing off, so the empty list
/// is the reading rather than a fallback.
fn ignored_checks(parsed: &serde_json::Value) -> Vec<String> {
    parsed["ignored_checks"]
        .as_array()
        .map(|checks| {
            checks
                .iter()
                .filter_map(|check| check["key"].as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

/// Replace each item's description with the text report's, which is correct.
///
/// The JSON descriptions are formatted by the same wrong units provider as its
/// coordinates, so a 12.70 mm wire is described as `length 0.1270 mm`. The two
/// reports enumerate the same items in the same order, and the caller has
/// already refused the pair when they do not.
fn described_by_the_text_report(
    mut violations: Vec<Violation>,
    text: &[TextItem],
) -> Vec<Violation> {
    let mut next = text.iter();
    for violation in &mut violations {
        for item in &mut violation.items {
            if let Some(line) = next.next() {
                item.description.clone_from(&line.description);
            }
        }
    }
    violations
}

/// A path as a command-line argument.
fn argument(path: &Path) -> Result<&str, CliFailure> {
    path.to_str().ok_or_else(|| {
        refusal(format!(
            "the path {} is not text kicli can pass to {PROGRAM}",
            path.display()
        ))
    })
}

/// Read a report KiCad said it wrote.
fn read(path: &Path) -> Result<String, CliFailure> {
    std::fs::read_to_string(path).map_err(|error| {
        refusal(format!(
            "{PROGRAM} reported no rule check kicli could read at {}: {error}",
            path.display()
        ))
    })
}

/// A report kicli will not stand on.
///
/// The reading is [`CliFailure::Failed`] because that is what it is from
/// kicli's side: the work was not completed, whatever the exit code said. It
/// carries kicli's own exit code 1 through the existing table in
/// [`crate::cli::ExitCode::for_tool_failure`], so no second table exists and no
/// raw `kicad-cli` code passes through (§6.2).
fn refusal(message: impl Into<String>) -> CliFailure {
    CliFailure::Failed {
        command: format!("{PROGRAM} sch erc"),
        message: message.into(),
    }
}

/// A JSON string field, or what is missing.
fn text_of(value: &serde_json::Value, missing: &str) -> Result<String, String> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| missing.to_owned())
}

#[cfg(test)]
mod tests {
    use super::{
        JSON_UNITS_PER_MM, ScaleVerdict, TextItem, corrected, read_text, scale_verdict, scaled,
    };
    use crate::geometry::{Iu, Point, UNITS_PER_MM};

    fn text_at(x: i32, y: i32) -> TextItem {
        TextItem {
            at: Point::new(x, y),
            description: "Symbol R1 Pin 1 [Passive, Line]".to_owned(),
        }
    }

    #[test]
    fn the_correction_is_exactly_the_ratio_of_the_two_scales() {
        // The bug is one scale used where another belongs, so the factor is
        // not a magic 100: it is 1e6 over 1e4, read out of KiCad's own headers.
        assert_eq!(JSON_UNITS_PER_MM / i64::from(UNITS_PER_MM), 100);
        assert_eq!(corrected("0.254"), Some(Iu(254_000)));
        assert_eq!(corrected("0.254321"), Some(Iu(254_321)));
        assert_eq!(corrected("0"), Some(Iu(0)));
        assert_eq!(corrected("-0.0127"), Some(Iu(-12_700)));
    }

    #[test]
    fn a_number_that_cannot_be_read_exactly_is_refused() {
        // Rounding here would be a coordinate kicli invented, so each of these
        // is nothing rather than a near miss.
        assert_eq!(corrected("0.2543215"), None, "seven fraction digits");
        assert_eq!(corrected("2.54e-1"), None, "an exponent");
        assert_eq!(corrected(""), None);
        assert_eq!(corrected("nonsense"), None);
        assert_eq!(scaled("1.5", 6), Some(1_500_000));
        assert_eq!(scaled("1.5", 0), None);
    }

    #[test]
    fn the_verdict_needs_an_item_that_tells_the_two_readings_apart() {
        // Everything at the origin satisfies both readings, so it confirms
        // neither. A check that read this as the bug would pass on any report
        // with no violations in it at all.
        assert_eq!(
            scale_verdict(&[Point::default()], &[text_at(0, 0)]),
            ScaleVerdict::Indeterminate
        );
        assert_eq!(scale_verdict(&[], &[]), ScaleVerdict::Indeterminate);
    }

    #[test]
    fn the_verdict_separates_the_bug_from_its_fix() {
        let text = [text_at(254_000, 215_900)];
        assert_eq!(
            scale_verdict(&[Point::new(254_000, 215_900)], &text),
            ScaleVerdict::HundredTimesSmall,
            "the corrected reading is the text report's"
        );
        assert_eq!(
            scale_verdict(&[Point::new(25_400_000, 21_590_000)], &text),
            ScaleVerdict::Fixed,
            "the correction over-shot by a hundred, so KiCad had already fixed it"
        );
        assert!(matches!(
            scale_verdict(&[Point::new(1, 2)], &text),
            ScaleVerdict::Unrelated { .. }
        ));
        assert!(matches!(
            scale_verdict(&[], &text),
            ScaleVerdict::Unrelated { .. }
        ));
    }

    #[test]
    fn the_text_reports_items_are_read_in_order() {
        let report = "***** Sheet /\n\
                      [pin_not_connected]: Pin not connected\n\
                      \x20   ; error\n\
                      \x20   @(25.40 mm, 21.59 mm): Symbol R1 Pin 1 [Passive, Line]\n\
                      \x20   @(76.20 mm, 50.80 mm): Horizontal Wire, length 12.70 mm\n";
        let items = read_text(report);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].at, Point::new(254_000, 215_900));
        assert_eq!(items[1].description, "Horizontal Wire, length 12.70 mm");
    }
}
