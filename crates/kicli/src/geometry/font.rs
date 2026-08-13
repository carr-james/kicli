//! How wide KiCad draws a string in its own stroke font.
//!
//! The width of a string is the sum of its glyph advances, less one inter-
//! character gap, inflated by the pen. Every step is integer arithmetic over
//! the advance table beside this file, which is derived from KiCad's Newstroke
//! font data by `cargo xtask text-metrics`.
//!
//! Ported from `common/font/stroke_font.cpp`, `common/font/font.cpp`,
//! `common/gr_text.cpp` and `include/markup_parser.h` at tag 10.0.5. KiCad is
//! GPL-3.0-or-later, as is kicli. The advance table carries Newstroke's own
//! notice: Copyright (C) 2010 vladimir uryvaev, GPL-2.0-or-later.

use std::sync::LazyLock;

use crate::geometry::{Iu, Size};

/// The advance table, derived from KiCad's Newstroke data.
const ADVANCE_TABLE: &str = include_str!("advances.table");

/// The scale KiCad divides every stroke coordinate by.
///
/// `STROKE_FONT_SCALE` in `common/font/stroke_font.cpp:45`.
const FONT_SCALE: i64 = 21;

/// The gap subtracted from the end of every drawn run.
///
/// `INTER_CHAR` in `common/font/stroke_font.cpp:209`, as a fraction of the text
/// width.
const INTER_CHAR: Ratio = Ratio { top: 1, bottom: 5 };

/// How far a tab advances, in glyph widths.
///
/// `TAB_WIDTH` in `common/font/stroke_font.cpp:208`.
const TAB_WIDTH: i64 = 4;

/// How much smaller a subscript or superscript glyph is.
///
/// `SUPER_SUB_SIZE_MULTIPLIER` in `common/font/stroke_font.cpp:210`.
const SUPER_SUB_SIZE: Ratio = Ratio { top: 4, bottom: 5 };

/// How far a superscript rises, as a fraction of its own glyph height.
///
/// `SUPER_HEIGHT_OFFSET` in `common/font/stroke_font.cpp:211`.
const SUPER_HEIGHT_OFFSET: Ratio = Ratio { top: 7, bottom: 20 };

/// How far a subscript drops, as a fraction of its own glyph height.
///
/// `SUB_HEIGHT_OFFSET` in `common/font/stroke_font.cpp:212`.
const SUB_HEIGHT_OFFSET: Ratio = Ratio { top: 3, bottom: 20 };

/// How far the string box grows on each side, in pen widths.
///
/// `FONT::StringBoundaryLimits` (`common/font/font.cpp:466-471`) inflates a
/// stroke-font box "to catch diacriticals, descenders, etc.".
const PEN_INFLATION: Ratio = Ratio { top: 3, bottom: 2 };

/// The glyph drawn for a code point the font does not hold.
///
/// `STROKE_FONT::GetTextAsGlyphs` (`common/font/stroke_font.cpp:258-263`)
/// substitutes it for every code point outside the table.
pub const SUBSTITUTION_GLYPH: char = '?';

/// The first code point the font holds.
const FIRST_CODE_POINT: u32 = 0x20;

/// The pen a schematic draws text with when the text names no thickness.
///
/// 6 mil, `DEFAULT_LINE_WIDTH_MILS` in `eeschema/default_values.h`. KiCad's
/// plotters pass this to `GetEffectiveTextPenWidth` as the default, so it is
/// the pen behind every measurement KiCad reports for a schematic.
pub const DEFAULT_PEN_WIDTH: Iu = Iu(1524);

/// An exact fraction, so a KiCad constant such as 0.2 stays exact.
#[derive(Clone, Copy, Debug)]
struct Ratio {
    /// The numerator.
    top: i64,
    /// The denominator, which is never zero.
    bottom: i64,
}

/// Round `top / bottom` half away from zero, as KiCad's `KiROUND` does.
fn round_ratio(top: i64, bottom: i64) -> i64 {
    let (top, bottom) = if bottom < 0 {
        (-top, -bottom)
    } else {
        (top, bottom)
    };
    if top >= 0 {
        (2 * top + bottom) / (2 * bottom)
    } else {
        -((-2 * top + bottom) / (2 * bottom))
    }
}

/// Truncate `top / bottom` towards zero, as a C++ double-to-int cast does.
fn truncate_ratio(top: i64, bottom: i64) -> i64 {
    top / bottom
}

/// One run of advances, as the table records them.
struct Run {
    /// The first code point of the run.
    first: u32,
    /// How many code points the run covers.
    count: u32,
    /// The advance numerator, over the font scale.
    numerator: i32,
}

/// The advance table, parsed once.
static ADVANCES: LazyLock<Vec<Run>> = LazyLock::new(|| parse_table(ADVANCE_TABLE));

/// Read the committed table. Malformed records are skipped.
///
/// The table is generated and committed together, so a malformed record means
/// the file was hand-edited. The unit tests below fail in that case.
fn parse_table(text: &str) -> Vec<Run> {
    let mut runs = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields = line.split_whitespace();
        let first = fields.next().and_then(|f| u32::from_str_radix(f, 16).ok());
        let count = fields.next().and_then(|f| f.parse().ok());
        let numerator = fields.next().and_then(|f| f.parse().ok());
        if let (Some(first), Some(count), Some(numerator)) = (first, count, numerator) {
            runs.push(Run {
                first,
                count,
                numerator,
            });
        }
    }
    runs
}

/// The advance numerator of a glyph, over the font scale of 21.
///
/// A code point the font does not hold takes the numerator of
/// [`SUBSTITUTION_GLYPH`].
///
/// # Examples
///
/// ```
/// use kicli::geometry::font::advance_numerator;
/// // The space glyph is 16 twenty-firsts of the text width wide.
/// assert_eq!(advance_numerator(' '), 16);
/// ```
#[must_use]
pub fn advance_numerator(glyph: char) -> i32 {
    numerator_of(glyph as u32)
        .unwrap_or_else(|| numerator_of(SUBSTITUTION_GLYPH as u32).unwrap_or_default())
}

/// The numerator recorded for a code point, when the table holds one.
fn numerator_of(code_point: u32) -> Option<i32> {
    if code_point < FIRST_CODE_POINT {
        return None;
    }
    let runs = &*ADVANCES;
    let index = runs
        .partition_point(|run| run.first <= code_point)
        .checked_sub(1)?;
    let run = &runs[index];
    if code_point < run.first + run.count {
        Some(run.numerator)
    } else {
        None
    }
}

/// How far the cursor moves when a glyph is drawn at this text width.
///
/// `STROKE_FONT::GetTextAsGlyphs` (`common/font/stroke_font.cpp:273-277`)
/// rounds the product to the nearest internal unit.
///
/// # Examples
///
/// ```
/// use kicli::geometry::{Iu, font::glyph_advance};
/// // At the schematic default size of 1.27 mm, 'A' advances 1.0886 mm.
/// assert_eq!(glyph_advance('A', Iu(12_700)), Iu(10_886));
/// ```
#[must_use]
pub fn glyph_advance(glyph: char, width: Iu) -> Iu {
    scaled_advance(glyph, width, None)
}

/// The advance of a glyph, with an optional run scale applied first.
fn scaled_advance(glyph: char, width: Iu, scale: Option<Ratio>) -> Iu {
    let numerator = i64::from(advance_numerator(glyph)) * i64::from(width.0);
    let (top, bottom) = match scale {
        None => (numerator, FONT_SCALE),
        Some(scale) => (numerator * scale.top, FONT_SCALE * scale.bottom),
    };
    Iu(clamp_to_iu(round_ratio(top, bottom)))
}

/// Narrow a computed length to the internal-unit type.
///
/// KiCad clamps the same way in `KiROUND`. A text item this long cannot be
/// drawn on any page, so the clamp is unreachable in practice.
fn clamp_to_iu(value: i64) -> i32 {
    i32::try_from(value).unwrap_or(if value < 0 { i32::MIN } else { i32::MAX })
}

/// The extents of one line of text, pen included.
///
/// This is `FONT::StringBoundaryLimits` (`common/font/font.cpp:451-478`) over
/// `STROKE_FONT::GetTextAsGlyphs` (`common/font/stroke_font.cpp:202-291`). The
/// text is one line: a line break is the caller's business, because KiCad
/// splits lines before it measures them.
///
/// Bold and italic do not change stroke-font extents. Italic tilts each glyph
/// about its own origin and bold widens the pen, so both reach these extents
/// through `thickness` alone.
///
/// # Examples
///
/// ```
/// use kicli::geometry::{Iu, Size, font::{DEFAULT_PEN_WIDTH, string_extents}};
/// let size = Size::new(12_700, 12_700);
/// // KiCad measures this string as 1.2918 mm wide.
/// assert_eq!(
///     string_extents("A", size, DEFAULT_PEN_WIDTH).x,
///     Iu(12_918)
/// );
/// ```
#[must_use]
pub fn string_extents(text: &str, size: Size, thickness: Iu) -> Size {
    let mut box_start: Option<(i64, i64)> = None;
    let mut box_end: Option<(i64, i64)> = None;
    let mut cursor_x: i64 = 0;

    for run in split_runs(text) {
        let (start, end, next) = run_box(run.text, run.style, size, cursor_x);
        cursor_x = next;
        box_start = Some(match box_start {
            None => start,
            Some((x, y)) => (x.min(start.0), y.min(start.1)),
        });
        box_end = Some(match box_end {
            None => end,
            Some((x, y)) => (x.max(end.0), y.max(end.1)),
        });
    }

    // An empty string keeps the empty box KiCad starts with, so it still grows
    // by the pen.
    let (start, end) = match (box_start, box_end) {
        (Some(start), Some(end)) => (start, end),
        _ => ((0, 0), (0, 0)),
    };
    let inflation = round_ratio(
        i64::from(thickness.0) * PEN_INFLATION.top,
        PEN_INFLATION.bottom,
    );
    Size {
        x: Iu(clamp_to_iu(end.0 - start.0 + 2 * inflation)),
        y: Iu(clamp_to_iu(end.1 - start.1 + 2 * inflation)),
    }
}

/// The box one styled run draws, and where the cursor lands after it.
///
/// The box is returned as normalised `(start, end)` corners, so the caller
/// merges runs by taking minima and maxima. Its origin is the run's own start
/// position, whose Y is the text baseline even when the run itself is raised or
/// dropped: KiCad sets the origin before it moves the cursor
/// (`common/font/stroke_font.cpp:214-228, 283-288`).
fn run_box(text: &str, style: RunStyle, size: Size, start_x: i64) -> ((i64, i64), (i64, i64), i64) {
    let scale = style.scale();
    let glyph_width = Ratio {
        top: i64::from(size.x.0) * scale.map_or(1, |ratio| ratio.top),
        bottom: scale.map_or(1, |ratio| ratio.bottom),
    };
    let glyph_height = Ratio {
        top: i64::from(size.y.0) * scale.map_or(1, |ratio| ratio.top),
        bottom: scale.map_or(1, |ratio| ratio.bottom),
    };

    // The cursor moves for a subscript or a superscript. KiCad adds the offset
    // to an integer cursor, which truncates it.
    let cursor_y = if style.subscript {
        truncate_ratio(
            glyph_height.top * SUB_HEIGHT_OFFSET.top,
            glyph_height.bottom * SUB_HEIGHT_OFFSET.bottom,
        )
    } else if style.superscript {
        -truncate_ratio(
            glyph_height.top * SUPER_HEIGHT_OFFSET.top,
            glyph_height.bottom * SUPER_HEIGHT_OFFSET.bottom,
        )
    } else {
        0
    };

    let cursor_x = advance_line(text, size, scale, start_x);
    let inter_char = round_ratio(
        glyph_width.top * INTER_CHAR.top,
        glyph_width.bottom * INTER_CHAR.bottom,
    );
    let end_x = cursor_x - inter_char;
    let end_y = truncate_ratio(
        cursor_y * glyph_height.bottom - glyph_height.top,
        glyph_height.bottom,
    );
    (
        (start_x.min(end_x), end_y.min(0)),
        (start_x.max(end_x), end_y.max(0)),
        cursor_x,
    )
}

/// Walk one run, glyph by glyph, and report where the cursor lands.
///
/// The tab rule locks the cursor to the next fourth column, counted in base
/// widths from the start of the run
/// (`common/font/stroke_font.cpp:232-247`).
fn advance_line(text: &str, size: Size, scale: Option<Ratio>, start_x: i64) -> i64 {
    let width = i64::from(size.x.0);
    let space = i64::from(advance_numerator(' '));
    let mut cursor = start_x;
    let mut glyphs: i64 = 0;

    for glyph in text.chars() {
        if glyph == '\t' {
            glyphs = (glyphs / TAB_WIDTH + 1) * TAB_WIDTH - 1;
            let mut next = start_x + width * glyphs + truncate_ratio(width * space, FONT_SCALE);
            while next <= cursor {
                glyphs += TAB_WIDTH;
                next += width * TAB_WIDTH;
            }
            cursor = next;
        } else {
            cursor += i64::from(scaled_advance(glyph, size.x, scale).0);
        }
        glyphs += 1;
    }
    cursor
}

/// Which of KiCad's text styles a run carries.
///
/// Overbar is absent because it does not change a run's extents. It raises the
/// box of the whole text item instead, which is the caller's business.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct RunStyle {
    /// Is the run a subscript?
    subscript: bool,
    /// Is the run a superscript?
    superscript: bool,
}

impl RunStyle {
    /// The glyph scale the run draws at, when it is not the full size.
    ///
    /// A subscript inside a superscript carries both flags and still scales
    /// once, because KiCad tests the two flags together.
    fn scale(self) -> Option<Ratio> {
        if self.subscript || self.superscript {
            Some(SUPER_SUB_SIZE)
        } else {
            None
        }
    }
}

/// One run of text drawn in a single style.
struct TextRun<'a> {
    /// The text of the run, markers removed.
    text: &'a str,
    /// The style the markers put it in.
    style: RunStyle,
}

/// Split marked-up text into runs.
///
/// KiCad's markup is `~{...}` for an overbar, `^{...}` for a superscript and
/// `_{...}` for a subscript, and the markers nest
/// (`include/markup_parser.h:54-99`). A marker whose group never closes is
/// ordinary text, because the grammar's alternative then falls through to a
/// plain string. Braces alone are ordinary text too.
fn split_runs(text: &str) -> Vec<TextRun<'_>> {
    let mut runs = Vec::new();
    collect_runs(text, RunStyle::default(), &mut runs);
    runs
}

/// Append the runs of `text`, which is already inside `style`.
fn collect_runs<'a>(text: &'a str, style: RunStyle, runs: &mut Vec<TextRun<'a>>) {
    let bytes = text.as_bytes();
    let mut plain_from = 0;
    let mut at = 0;

    while at < bytes.len() {
        let marker = match bytes[at] {
            b'~' => Some(style),
            b'^' => Some(RunStyle {
                superscript: true,
                ..style
            }),
            b'_' => Some(RunStyle {
                subscript: true,
                ..style
            }),
            _ => None,
        };
        let opens = marker.is_some() && bytes.get(at + 1) == Some(&b'{');
        let end = if opens {
            group_end(bytes, at + 2)
        } else {
            None
        };

        match (marker, end) {
            (Some(inner), Some(end)) => {
                if plain_from < at {
                    runs.push(TextRun {
                        text: &text[plain_from..at],
                        style,
                    });
                }
                collect_runs(&text[at + 2..end], inner, runs);
                at = end + 1;
                plain_from = at;
            }
            _ => at += 1,
        }
    }

    if plain_from < bytes.len() {
        runs.push(TextRun {
            text: &text[plain_from..],
            style,
        });
    }
}

/// Where the group opened before `from` closes, when it closes at all.
///
/// Nested markers and `{identifier}` escape sequences are skipped, so their
/// closing braces do not end the group.
fn group_end(bytes: &[u8], from: usize) -> Option<usize> {
    let mut at = from;
    while at < bytes.len() {
        match bytes[at] {
            b'}' => return Some(at),
            b'~' | b'^' | b'_' if bytes.get(at + 1) == Some(&b'{') => {
                at = group_end(bytes, at + 2).map_or(at + 1, |end| end + 1);
            }
            b'{' => at = escape_end(bytes, at + 1).map_or(at + 1, |end| end + 1),
            _ => at += 1,
        }
    }
    None
}

/// Where a `{identifier}` escape sequence closes, when it is one.
fn escape_end(bytes: &[u8], from: usize) -> Option<usize> {
    let mut at = from;
    while at < bytes.len() {
        match bytes[at] {
            b'}' if at > from => return Some(at),
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => at += 1,
            b'0'..=b'9' if at > from => at += 1,
            _ => return None,
        }
    }
    None
}

/// The pen a bold text draws with.
///
/// `GetPenSizeForBold` (`common/gr_text.cpp:37-40`).
#[must_use]
pub fn bold_pen_width(width: Iu) -> Iu {
    Iu(clamp_to_iu(round_ratio(i64::from(width.0), 5)))
}

/// The pen an ordinary text draws with.
///
/// `GetPenSizeForNormal` (`common/gr_text.cpp:61-64`).
#[must_use]
pub fn normal_pen_width(width: Iu) -> Iu {
    Iu(clamp_to_iu(round_ratio(i64::from(width.0), 8)))
}

/// Hold a pen down to a quarter of the text size.
///
/// `ClampTextPenSize` (`common/gr_text.cpp:71-93`), in its non-strict form,
/// which is the one text items use.
#[must_use]
pub fn clamp_pen_width(pen: Iu, size: Size) -> Iu {
    let smaller = i64::from(size.x.0.abs().min(size.y.0.abs()));
    let maximum = clamp_to_iu(round_ratio(smaller, 4));
    Iu(pen.0.min(maximum))
}

#[cfg(test)]
mod tests {
    use super::{
        ADVANCE_TABLE, DEFAULT_PEN_WIDTH, RunStyle, SUBSTITUTION_GLYPH, advance_numerator,
        bold_pen_width, clamp_pen_width, glyph_advance, normal_pen_width, round_ratio, split_runs,
        string_extents,
    };
    use crate::geometry::{Iu, Size};

    /// The schematic default text size.
    const DEFAULT_SIZE: Size = Size::new(12_700, 12_700);

    #[test]
    fn advances_cover_printable_ascii() {
        for code_point in 0x21..=0x7Eu32 {
            let glyph = char::from_u32(code_point).expect("the code point is a character");
            assert!(
                advance_numerator(glyph) > 0,
                "{glyph:?} has no advance in the table"
            );
        }
        // A space draws nothing and still advances.
        assert!(advance_numerator(' ') > 0);

        // A code point outside the table falls back to the substitution glyph.
        let beyond = char::from_u32(0x1_0000).expect("the code point is a character");
        assert_eq!(
            advance_numerator(beyond),
            advance_numerator(SUBSTITUTION_GLYPH)
        );
        assert_eq!(
            advance_numerator('\u{1}'),
            advance_numerator(SUBSTITUTION_GLYPH)
        );
        assert_eq!(SUBSTITUTION_GLYPH, '?');

        // The provenance header names the source and keeps the upstream notice.
        let header: String = ADVANCE_TABLE
            .lines()
            .take_while(|line| line.starts_with('#'))
            .collect::<Vec<&str>>()
            .join("\n");
        assert!(header.contains("newstroke_font.cpp"), "{header}");
        assert!(header.contains("tag 10.0.5"), "{header}");
        assert!(header.contains("vladimir uryvaev"), "{header}");
        assert!(header.contains("GNU General Public License"), "{header}");
        assert!(header.contains("version 2"), "{header}");
    }

    #[test]
    fn a_glyph_advance_is_its_numerator_over_the_font_scale() {
        // 'A' is 18 twenty-firsts wide, so at 1.27 mm it advances 1.0886 mm.
        assert_eq!(advance_numerator('A'), 18);
        assert_eq!(glyph_advance('A', Iu(12_700)), Iu(10_886));
        // Rounding is half away from zero, as KiCad's KiROUND is.
        assert_eq!(round_ratio(3, 2), 2);
        assert_eq!(round_ratio(-3, 2), -2);
        assert_eq!(round_ratio(1, 2), 1);
    }

    #[test]
    fn a_string_is_as_wide_as_its_glyphs_less_one_gap() {
        // KiCad measures "A" as 1.2918 mm and "AB" as 2.5618 mm at 1.27 mm.
        assert_eq!(
            string_extents("A", DEFAULT_SIZE, DEFAULT_PEN_WIDTH).x,
            Iu(12_918)
        );
        assert_eq!(
            string_extents("AB", DEFAULT_SIZE, DEFAULT_PEN_WIDTH).x,
            Iu(25_618)
        );
        // The height is the text height plus the pen inflation.
        assert_eq!(
            string_extents("A", DEFAULT_SIZE, DEFAULT_PEN_WIDTH).y,
            Iu(12_700 + 2 * 2_286)
        );
        // An empty string is the pen alone.
        assert_eq!(
            string_extents("", DEFAULT_SIZE, DEFAULT_PEN_WIDTH).x,
            Iu(2 * 2_286)
        );
    }

    #[test]
    fn overbar_markers_are_not_drawn_but_braces_alone_are() {
        let bar = string_extents("~{AB}", DEFAULT_SIZE, DEFAULT_PEN_WIDTH).x;
        let plain = string_extents("AB", DEFAULT_SIZE, DEFAULT_PEN_WIDTH).x;
        assert_eq!(bar, plain);

        // A brace with no marker is an ordinary glyph.
        let brace = string_extents("}", DEFAULT_SIZE, DEFAULT_PEN_WIDTH).x;
        assert!(brace > Iu(2 * 2_286));

        // A marker whose group never closes is ordinary text too.
        let unclosed = string_extents("~{AB", DEFAULT_SIZE, DEFAULT_PEN_WIDTH).x;
        assert!(unclosed > plain);
    }

    #[test]
    fn markup_splits_text_into_styled_runs() {
        let runs = split_runs("A~{B}C");
        let texts: Vec<&str> = runs.iter().map(|run| run.text).collect();
        assert_eq!(texts, vec!["A", "B", "C"]);
        assert!(runs.iter().all(|run| run.style == RunStyle::default()));

        let runs = split_runs("x^{2}");
        assert_eq!(runs.len(), 2);
        assert!(runs[1].style.superscript);
        assert!(!runs[1].style.subscript);

        // A subscript inside a superscript keeps both flags.
        let runs = split_runs("^{a_{b}}");
        assert_eq!(runs.len(), 2);
        assert!(runs[1].style.superscript && runs[1].style.subscript);

        // An escape sequence inside a group does not close it.
        let runs = split_runs("~{a{ref}b}c");
        let texts: Vec<&str> = runs.iter().map(|run| run.text).collect();
        assert_eq!(texts, vec!["a{ref}b", "c"]);
    }

    #[test]
    fn a_superscript_is_smaller_and_higher_than_its_text() {
        let plain = string_extents("2", DEFAULT_SIZE, DEFAULT_PEN_WIDTH);
        let raised = string_extents("^{2}", DEFAULT_SIZE, DEFAULT_PEN_WIDTH);
        assert!(
            raised.x < plain.x,
            "{raised:?} is not narrower than {plain:?}"
        );
        // The glyph is shorter, but it sits higher, so the box is taller.
        assert!(
            raised.y > plain.y,
            "{raised:?} is not taller than {plain:?}"
        );
    }

    #[test]
    fn a_tab_locks_the_cursor_to_the_next_fourth_column() {
        let tabbed = string_extents("\ttab", DEFAULT_SIZE, DEFAULT_PEN_WIDTH).x;
        let plain = string_extents("tab", DEFAULT_SIZE, DEFAULT_PEN_WIDTH).x;
        assert!(tabbed > plain);
        // The tab lands on the fourth base width, which is four text widths.
        assert_eq!(tabbed - plain, Iu(3 * 12_700 + 9_676));
    }

    #[test]
    fn a_pen_derives_from_the_text_width_and_is_clamped() {
        // 1.27 mm text: a quarter of one eighth, and a fifth when bold.
        assert_eq!(normal_pen_width(Iu(12_700)), Iu(1_588));
        assert_eq!(bold_pen_width(Iu(12_700)), Iu(2_540));
        // The clamp holds a wide pen down to a quarter of the smaller side.
        assert_eq!(
            clamp_pen_width(Iu(9_000), Size::new(12_700, 12_700)),
            Iu(3_175)
        );
        assert_eq!(
            clamp_pen_width(Iu(1_524), Size::new(12_700, 12_700)),
            Iu(1_524)
        );
    }
}
