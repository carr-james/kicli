//! Proof that formatting from the integer matches formatting from the double.
//!
//! kicli writes coordinates straight from the `int32`, never through a float.
//! KiCad writes them by dividing by 10000 and printing ten significant digits.
//! The two must agree for every value an `int32` can hold, and "must" here is
//! checked rather than argued: the release build sweeps the whole range.
//!
//! The two paths are genuinely independent. One is integer arithmetic and string
//! surgery; the other goes through `f64` and the standard library's float
//! formatter.

use kicli_sexpr::{fmt_iu, format_significant};

/// KiCad's own path: divide into millimetres, print ten significant digits.
fn kicad_would_write(units: i32) -> String {
    let millimetres = f64::from(units) / 10_000.0;
    if millimetres != 0.0 && millimetres.abs() <= 0.0001 {
        // KiCad takes a fixed-point branch for values this small.
        let text = format!("{millimetres:.10}");
        let trimmed = text.trim_end_matches('0');
        return trimmed.strip_suffix('.').unwrap_or(trimmed).to_owned();
    }
    format_significant(millimetres, 10)
}

/// Values worth checking whatever else runs.
fn boundary_values() -> Vec<i32> {
    let mut values = vec![
        0,
        1,
        -1,
        9,
        10,
        -10,
        9_999,
        10_000,
        10_001,
        -10_000,
        12_700,
        -12_700,
        419_100,
        i32::MAX,
        i32::MIN,
        i32::MAX - 1,
        i32::MIN + 1,
    ];
    for power in 0..10 {
        let magnitude = 10i64.pow(power);
        for offset in -2i64..=2 {
            if let Ok(value) = i32::try_from(magnitude + offset) {
                values.push(value);
                values.push(-value);
            }
        }
    }
    values
}

#[test]
fn fmt_iu_matches_kicad_for_all_i32() {
    for value in boundary_values() {
        assert_eq!(
            fmt_iu(value),
            kicad_would_write(value),
            "the two paths disagree at {value}"
        );
    }

    let threads = std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(4);

    // A debug build takes far too long to sweep four billion values, so it
    // checks a stride instead. The release build, which is what the completion
    // check runs, sweeps every one.
    let stride: i64 = if cfg!(debug_assertions) { 4_099 } else { 1 };
    if stride > 1 {
        println!("debug build: checking every {stride}th value");
    } else {
        println!("release build: checking every i32 across {threads} threads");
    }

    let span = (i64::from(i32::MAX) - i64::from(i32::MIN) + 1) / threads as i64 + 1;
    let failure = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads as i64)
            .map(|slice| {
                scope.spawn(move || {
                    let start = i64::from(i32::MIN) + slice * span;
                    let end = (start + span).min(i64::from(i32::MAX) + 1);
                    let mut value = start;
                    while value < end {
                        let units = i32::try_from(value).expect("slice stays inside i32");
                        let ours = fmt_iu(units);
                        if ours != kicad_would_write(units) {
                            return Some(units);
                        }
                        value += stride;
                    }
                    None
                })
            })
            .collect();
        handles
            .into_iter()
            .filter_map(|handle| handle.join().expect("worker finished"))
            .min()
    });

    assert_eq!(
        failure, None,
        "the integer and float paths disagree; the first value is shown"
    );
}

#[test]
fn every_formatted_value_parses_back() {
    for value in boundary_values() {
        assert_eq!(
            kicli_sexpr::parse_iu(&fmt_iu(value)),
            Some(value),
            "{value} does not survive a round trip"
        );
    }
}
