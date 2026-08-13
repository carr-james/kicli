//! Coordinates as integers, formatted the way KiCad formats them.
//!
//! A schematic coordinate is an `int32` count of 100 nm units. KiCad divides it
//! by 10000 and prints the result with ten significant digits. Because an
//! `int32` divided by 10000 never needs more than ten significant digits, that
//! print is exact, and the same string can be produced from the integer alone.
//! Formatting from the integer keeps every coordinate off the floating-point
//! path, so a value can never come back as 41.910000000000004.

/// Internal units in one millimetre, for schematics.
pub const UNITS_PER_MM: i32 = 10_000;

/// Format an internal unit the way KiCad writes it.
///
/// # Examples
///
/// ```
/// use kicli_sexpr::fmt_iu;
/// assert_eq!(fmt_iu(419_100), "41.91");
/// assert_eq!(fmt_iu(0), "0");
/// assert_eq!(fmt_iu(-12_700), "-1.27");
/// assert_eq!(fmt_iu(1), "0.0001");
/// ```
#[must_use]
pub fn fmt_iu(value: i32) -> String {
    let magnitude = i64::from(value).abs();
    let sign = if value < 0 { "-" } else { "" };
    let whole = magnitude / i64::from(UNITS_PER_MM);
    let fraction = magnitude % i64::from(UNITS_PER_MM);

    if fraction == 0 {
        return format!("{sign}{whole}");
    }

    let mut digits = format!("{fraction:04}");
    while digits.ends_with('0') {
        digits.pop();
    }
    format!("{sign}{whole}.{digits}")
}

/// Read a millimetre value into internal units, when it is exact.
///
/// Returns `None` for anything that is not a plain decimal with at most four
/// fractional digits, or that does not fit an `i32`. A value kicli cannot
/// represent exactly is a value kicli must not rewrite.
///
/// # Examples
///
/// ```
/// use kicli_sexpr::parse_iu;
/// assert_eq!(parse_iu("41.91"), Some(419_100));
/// assert_eq!(parse_iu("-1.27"), Some(-12_700));
/// assert_eq!(parse_iu("1.234567"), None);
/// ```
#[must_use]
pub fn parse_iu(text: &str) -> Option<i32> {
    let (sign, rest) = match text.strip_prefix('-') {
        Some(rest) => (-1i64, rest),
        None => (1i64, text),
    };
    if rest.is_empty() {
        return None;
    }

    let (whole_text, fraction_text) = match rest.split_once('.') {
        Some((whole, fraction)) => (whole, fraction),
        None => (rest, ""),
    };
    if whole_text.is_empty() && fraction_text.is_empty() {
        return None;
    }
    if fraction_text.len() > 4 {
        return None;
    }
    if !whole_text.bytes().all(|b| b.is_ascii_digit())
        || !fraction_text.bytes().all(|b| b.is_ascii_digit())
    {
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
    for _ in fraction_text.len()..4 {
        fraction *= 10;
    }

    let units = sign
        * (whole
            .checked_mul(i64::from(UNITS_PER_MM))?
            .checked_add(fraction)?);
    i32::try_from(units).ok()
}

/// Format an angle the way KiCad writes it.
///
/// Angles are not internal units. KiCad prints them with ten significant
/// digits, straight from the double.
#[must_use]
pub fn fmt_angle(degrees: f64) -> String {
    format_significant(degrees, 10)
}

/// Format `value` with `digits` significant digits, as C's `%g` would.
///
/// This is the general path KiCad uses for values that are not internal units.
/// It exists so [`fmt_iu`] can be checked against an independent
/// implementation that does go through floating point.
#[must_use]
pub fn format_significant(value: f64, digits: usize) -> String {
    if value == 0.0 {
        return "0".to_owned();
    }
    if !value.is_finite() {
        return format!("{value}");
    }

    let exponent = exponent_of(value);
    let digits_as_exponent = i32::try_from(digits).unwrap_or(i32::MAX);

    // C's %g switches to exponent form outside this window.
    if exponent < -4 || exponent >= digits_as_exponent {
        let mantissa_digits = digits.saturating_sub(1);
        let text = format!("{value:.mantissa_digits$e}");
        return trim_zeros_in_exponent_form(&text);
    }

    let decimals = usize::try_from((digits_as_exponent - 1 - exponent).max(0)).unwrap_or(0);
    let text = format!("{value:.decimals$}");
    trim_trailing_zeros(&text)
}

/// The power of ten of the leading digit.
fn exponent_of(value: f64) -> i32 {
    let raw = value.abs().log10().floor();
    if raw > f64::from(i32::MAX) {
        i32::MAX
    } else if raw < f64::from(i32::MIN) {
        i32::MIN
    } else {
        // The two bounds above put `raw` inside i32, and log10().floor() has no
        // fractional part, so this conversion loses nothing.
        #[allow(clippy::cast_possible_truncation)]
        {
            raw as i32
        }
    }
}

fn trim_trailing_zeros(text: &str) -> String {
    if !text.contains('.') {
        return text.to_owned();
    }
    let trimmed = text.trim_end_matches('0');
    trimmed.strip_suffix('.').unwrap_or(trimmed).to_owned()
}

fn trim_zeros_in_exponent_form(text: &str) -> String {
    match text.split_once('e') {
        Some((mantissa, exponent)) => {
            format!("{}e{exponent}", trim_trailing_zeros(mantissa))
        }
        None => trim_trailing_zeros(text),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_millimetres_carry_no_point() {
        assert_eq!(fmt_iu(0), "0");
        assert_eq!(fmt_iu(10_000), "1");
        assert_eq!(fmt_iu(-20_000), "-2");
    }

    #[test]
    fn trailing_zeros_are_stripped() {
        assert_eq!(fmt_iu(419_100), "41.91");
        assert_eq!(fmt_iu(12_700), "1.27");
        assert_eq!(fmt_iu(1_000), "0.1");
    }

    #[test]
    fn the_smallest_unit_keeps_four_decimals() {
        assert_eq!(fmt_iu(1), "0.0001");
        assert_eq!(fmt_iu(-1), "-0.0001");
    }

    #[test]
    fn the_range_ends_format() {
        assert_eq!(fmt_iu(i32::MAX), "214748.3647");
        assert_eq!(fmt_iu(i32::MIN), "-214748.3648");
    }

    #[test]
    fn parsing_inverts_formatting() {
        for value in [0, 1, -1, 12_700, -419_100, i32::MAX, i32::MIN] {
            assert_eq!(parse_iu(&fmt_iu(value)), Some(value), "value {value}");
        }
    }

    #[test]
    fn inexact_input_is_refused() {
        assert_eq!(parse_iu("1.234567"), None);
        assert_eq!(parse_iu("abc"), None);
        assert_eq!(parse_iu(""), None);
        assert_eq!(parse_iu("1e3"), None);
    }

    #[test]
    fn significant_digit_formatting_matches_the_c_rule() {
        assert_eq!(format_significant(0.0, 10), "0");
        assert_eq!(format_significant(1.27, 10), "1.27");
        assert_eq!(format_significant(179.9944, 10), "179.9944");
        assert_eq!(format_significant(90.0, 10), "90");
    }
}
