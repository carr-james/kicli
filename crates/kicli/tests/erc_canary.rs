//! The canary that expects KiCad's 100× rule-check JSON bug.
//!
//! `spec/SPEC.md` §14.2, requirement 2, verbatim: *"A CANARY TEST that expects
//! the bug: on a committed fixture, assert `json.pos × 100 == text.pos`
//! exactly. When upstream fixes it, this test fails loudly and the workaround
//! is removed, never double-applied."*
//!
//! The failure being guarded against is a correction applied twice — a
//! coordinate wrong by 10,000× in a tool whose whole job is where things are
//! drawn. So this file asserts the bug in both directions: that the committed
//! reports show it, and that a report **without** it is not read as showing it.
//!
//! # The fixtures are committed, and they are KiCad's own bytes
//!
//! Four pairs of reports, each written by `kicad-cli` 10.0.5 in one run over
//! the schematic beside it: `erc/canary`, `erc/precision`, `geometry/
//! orientations` and `geometry/asymmetric`. Nothing here fetches or generates
//! anything, because the claim is about a *specific* KiCad version and the
//! measurement must be reproducible by anyone at any time.
//!
//! # Why one of the four is exempt from the exact assertion
//!
//! `erc/precision` places a symbol at `25.4321 mm`, and the text report writes
//! `25.432`. **The text report rounds to three decimals**, one short of the
//! schematic's own 1e-4 mm resolution, so `json × 100 == text` is *not* exact
//! there — off by one internal unit, in the text report's favour. That is a
//! measurement rather than an exception, and
//! [`the_text_report_loses_the_fourth_decimal`] is where it is made. It is also
//! the reason kicli reads positions from the corrected JSON and uses the text
//! report only as a control.

use std::path::{Path, PathBuf};

use kicli::geometry::{Iu, Point};
use kicli::kicad::erc::{
    ScaleVerdict, TextItem, corrected, read_json, read_reports, read_text, rule_check,
    scale_verdict,
};
use kicli::kicad::{Discovery, KicadCli};
use kicli::model::Config;
use kicli_probe::oracle::Kicad;
use kicli_probe::scratch::Fixtures;

/// A committed pair of reports, and the schematic they are about.
struct Pair {
    name: &'static str,
    json: String,
    text: String,
}

/// The four committed pairs, in the order they are named above.
fn pairs() -> Vec<Pair> {
    [
        ("erc/canary/canary", "erc/canary/canary"),
        ("erc/precision/precision", "erc/precision/precision"),
        ("geometry/orientations", "geometry/orientations"),
        ("geometry/asymmetric", "geometry/asymmetric"),
    ]
    .iter()
    .map(|(name, stem)| Pair {
        name,
        json: read(&fixture(&format!("{stem}.erc.json"))),
        text: read(&fixture(&format!("{stem}.erc.txt"))),
    })
    .collect()
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{} reads: {error}", path.display()))
}

/// Every corrected position of a JSON report, in the order KiCad wrote them.
fn corrected_positions(json: &str) -> Vec<Point> {
    read_json(json)
        .expect("the committed JSON report parses")
        .iter()
        .flat_map(|violation| violation.items.iter().map(|item| item.at))
        .collect()
}

/// The same report with every coordinate multiplied by a hundred.
///
/// This is the correctly-scaled report §14.2 describes and KiCad does not yet
/// write. It is built by shifting each coordinate's decimal point two places,
/// as text, so no float is involved and the resulting number is exact.
fn as_a_fixed_kicad_would_write_it(json: &str) -> String {
    let mut out = String::with_capacity(json.len());
    for line in json.lines() {
        let shifted = shift_two(line);
        out.push_str(&shifted);
        out.push('\n');
    }
    out
}

/// A `"x": 0.2159,` line with its number's decimal point moved two places.
fn shift_two(line: &str) -> String {
    let trimmed = line.trim_start();
    let key = ["\"x\": ", "\"y\": "]
        .into_iter()
        .find(|key| trimmed.starts_with(key));
    let Some(key) = key else {
        return line.to_owned();
    };
    let indent = &line[..line.len() - trimmed.len()];
    let rest = &trimmed[key.len()..];
    let (number, tail) = match rest.strip_suffix(',') {
        Some(number) => (number, ","),
        None => (rest, ""),
    };
    let (whole, fraction) = match number.split_once('.') {
        Some((whole, fraction)) => (whole.to_owned(), fraction.to_owned()),
        None => (number.to_owned(), String::new()),
    };
    let mut fraction = fraction;
    while fraction.len() < 2 {
        fraction.push('0');
    }
    let mut whole = format!("{whole}{}", &fraction[..2]);
    while whole.len() > 1 && whole.starts_with('0') {
        whole.remove(0);
    }
    let rest_of_fraction = &fraction[2..];
    let moved = if rest_of_fraction.is_empty() {
        format!("{whole}.0")
    } else {
        format!("{whole}.{rest_of_fraction}")
    };
    format!("{indent}{key}{moved}{tail}")
}

/// The pairs whose coordinates all fit the text report's three decimals.
///
/// `erc/precision` is deliberately not one of them; see the module note.
fn exactly_comparable() -> Vec<Pair> {
    pairs()
        .into_iter()
        .filter(|pair| pair.name != "erc/precision/precision")
        .collect()
}

#[test]
fn the_rule_check_json_reports_coordinates_a_hundred_times_small() {
    // §14.2 requirement 2, as an equality over KiCad's own bytes. `json.pos`
    // times a hundred is what `corrected` returns, so this is that assertion
    // and it runs through the correction kicli ships rather than beside it.
    let mut compared = 0;
    for pair in exactly_comparable() {
        let found = corrected_positions(&pair.json);
        let reported = read_text(&pair.text);
        assert_eq!(
            found.len(),
            reported.len(),
            "{}: the two reports name the same number of items",
            pair.name
        );
        assert!(
            !found.is_empty(),
            "{}: the reports have items to compare",
            pair.name
        );
        for (json, text) in found.iter().zip(&reported) {
            assert_eq!(
                *json, text.at,
                "{}: the JSON coordinate times a hundred is the text coordinate ({})",
                pair.name, text.description
            );
            compared += 1;
        }
        assert_eq!(
            scale_verdict(&found, &reported),
            ScaleVerdict::HundredTimesSmall,
            "{}: the verdict kicli takes at run time agrees",
            pair.name
        );
    }
    // The control on the sweep: a run that read no coordinates would assert
    // nothing and pass.
    assert!(compared > 40, "coordinates compared: {compared}");
}

#[test]
fn a_correctly_scaled_report_is_not_read_as_the_bug() {
    // The falsification, kept. §14.2's failure is the correction applied twice,
    // so the day upstream fixes `erc_report.cpp:161` every assertion above must
    // go red and kicli must refuse rather than correct. This builds that day's
    // report and checks all three of those.
    for pair in exactly_comparable() {
        let fixed = as_a_fixed_kicad_would_write_it(&pair.json);
        let found = corrected_positions(&fixed);
        let reported = read_text(&pair.text);

        assert!(
            found
                .iter()
                .zip(&reported)
                .any(|(json, text)| *json != text.at),
            "{}: a correctly scaled report fails the equality above",
            pair.name
        );
        assert_eq!(
            scale_verdict(&found, &reported),
            ScaleVerdict::Fixed,
            "{}: the run-time verdict says the bug is gone",
            pair.name
        );

        let refusal = read_reports(&fixed, &pair.text)
            .expect_err("kicli refuses a report it can no longer correct");
        let message = refusal.to_string();
        assert!(
            message.contains("erc_report.cpp:161"),
            "{}: the refusal names the upstream line: {message}",
            pair.name
        );
        assert!(
            message.contains("Remove it rather than doubling it"),
            "{}: the refusal says what to do: {message}",
            pair.name
        );
    }
}

#[test]
fn the_committed_reports_are_still_the_ones_kicli_reads() {
    // The whole reading, over KiCad's own bytes, with no process started. A
    // parse that silently produced nothing would make every claim about the
    // seam vacuous, so the counts are asserted here and nowhere else.
    let canary = pairs().into_iter().next().expect("the canary pair");
    let reading = read_reports(&canary.json, &canary.text).expect("the committed pair is read");
    assert!(reading.has_run());
    assert_eq!(reading.violations().len(), 11);
    assert_eq!(
        reading
            .violations()
            .iter()
            .flat_map(|violation| &violation.items)
            .count(),
        13
    );

    // The item descriptions come from the text report, because the same wrong
    // units provider corrupts the JSON's: it calls a 12.70 mm wire
    // `length 0.1270 mm`. Both halves are asserted, so a reading that took the
    // JSON's description would fail on the first half rather than pass quietly.
    let descriptions: Vec<&str> = reading
        .violations()
        .iter()
        .flat_map(|violation| violation.items.iter().map(|item| item.description.as_str()))
        .collect();
    assert!(
        descriptions
            .iter()
            .any(|text| text.contains("length 12.70 mm")),
        "the text report's description is the one kept: {descriptions:?}"
    );
    assert!(
        !descriptions
            .iter()
            .any(|text| text.contains("length 0.1270 mm")),
        "the JSON's own description is scaled wrong and is not used: {descriptions:?}"
    );
}

#[test]
fn the_text_report_loses_the_fourth_decimal() {
    // The measurement that settles §14.2's open choice. KiCad wrote both
    // reports in one run over a symbol placed at 25.4321 mm. The JSON's number,
    // corrected, is that coordinate exactly. The text report's is not.
    let text = read(&fixture("erc/precision/precision.erc.txt"));
    let json = read(&fixture("erc/precision/precision.erc.json"));

    let exact = corrected_positions(&json);
    let rounded = read_text(&text);
    assert_eq!(exact.len(), rounded.len());
    assert!(!exact.is_empty(), "the precision fixture reported items");

    assert!(
        exact.iter().all(|at| at.x == Iu(254_321)),
        "the corrected JSON carries the placed coordinate exactly: {exact:?}"
    );
    assert!(
        rounded.iter().all(|item| item.at.x == Iu(254_320)),
        "the text report rounded it to three decimals: {rounded:?}"
    );

    // And the run-time control still accepts the pair, because its slack is the
    // text report's own step. A control tighter than the instrument it reads
    // would refuse every off-grid drawing kicli exists to find.
    assert_eq!(
        scale_verdict(&exact, &rounded),
        ScaleVerdict::HundredTimesSmall
    );
}

#[test]
fn the_two_reports_enumerate_the_same_items_in_the_same_order() {
    // kicli pairs the two reports positionally, so this is the assumption that
    // pairing rests on, measured on every committed pair rather than assumed.
    for pair in pairs() {
        let json = read_json(&pair.json).expect("the JSON parses");
        let items: Vec<&kicli::lint::erc::Item> = json
            .iter()
            .flat_map(|violation| violation.items.iter())
            .collect();
        let text: Vec<TextItem> = read_text(&pair.text);
        assert_eq!(
            items.len(),
            text.len(),
            "{}: the two reports name the same items",
            pair.name
        );
        assert!(!items.is_empty(), "{}: there are items", pair.name);
        for (item, line) in items.iter().zip(&text) {
            let slack = (item.at.x.0 - line.at.x.0).abs() + (item.at.y.0 - line.at.y.0).abs();
            assert!(
                slack <= 10,
                "{}: item {} does not line up with {}",
                pair.name,
                item.description,
                line.description
            );
        }
    }
}

#[test]
fn a_number_kicli_cannot_read_exactly_is_never_rounded() {
    // The correction is integer arithmetic on the number's own text
    // (Constitution §4). Anything it cannot read exactly is nothing, because a
    // rounded coordinate is a coordinate kicli invented.
    assert_eq!(corrected("0.254321"), Some(Iu(254_321)));
    assert_eq!(corrected("0.2543215"), None);
    assert_eq!(corrected("2.54e-1"), None);
}

#[test]
fn the_committed_reports_are_current() {
    // The canary asserts a property of KiCad 10.0.5's output, so the committed
    // reports must still be that output. This is the environment-gated half:
    // it runs the real binary through kicli's own gateway — the same
    // `rule_check` a `kicli sch erc` run would take — and compares what KiCad
    // writes today against the bytes beside the fixture.
    //
    // A lane worktree's green here counts toward no tick (`CLAUDE.md`); the
    // measurement it makes is what the entry records.
    let Some(_asked) = Kicad::found_or_skip("regenerate the committed ERC reports") else {
        return;
    };
    let fixtures = Fixtures::new(env!("CARGO_TARGET_TMPDIR"), env!("CARGO_MANIFEST_DIR"));
    let cli = KicadCli::locate(&Discovery::new(&Config::default()))
        .expect("kicad-cli is on this machine");
    let version = cli.version().expect("kicad-cli is a version kicli reads");
    println!("measured against kicad-cli {version}");

    for stem in ["canary", "precision"] {
        // KiCad writes a .kicad_prl beside any project it opens, so the fixture
        // is copied out and the tool runs on the copy. The fixture tree stays
        // exactly as committed.
        let directory =
            fixtures.scratch_directory(&format!("erc-currency-{stem}"), &format!("erc/{stem}"));
        rule_check(
            &cli,
            &directory.join(format!("{stem}.kicad_sch")),
            &directory,
        )
        .expect("the fresh reports read");

        for (fresh, committed) in [
            ("kicli-erc.json", format!("erc/{stem}/{stem}.erc.json")),
            ("kicli-erc.txt", format!("erc/{stem}/{stem}.erc.txt")),
        ] {
            let dated = |text: &str| -> Vec<String> {
                text.lines()
                    .map(str::trim_end)
                    .filter(|line| {
                        let line = line.trim_start();
                        !line.starts_with("\"date\":") && !line.starts_with("ERC report (")
                    })
                    .map(str::to_owned)
                    .collect()
            };
            assert_eq!(
                dated(&read(&fixture(&committed))),
                dated(&read(&directory.join(fresh))),
                "{committed} is what kicad-cli {version} writes today"
            );
        }
    }
}
