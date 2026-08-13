//! The box a text item occupies on the page.
//!
//! The box is built unrotated, from the string extents of the widest line, and
//! then turned about the draw position. It is therefore an oriented box, not an
//! axis-aligned one: schematic text is routinely at 90 degrees, and an
//! axis-aligned box around a vertical label overstates its width by a factor of
//! ten.
//!
//! Ported from `EDA_TEXT::GetTextBox` (`common/eda_text.cpp:742-870`) and
//! `EDA_TEXT::GetEffectiveTextPenWidth` (`common/eda_text.cpp:449-466`) at tag
//! 10.0.5. KiCad is GPL-3.0-or-later, as is kicli.

use kicli_sexpr::{Doc, NodeId};

use crate::geometry::font::{
    DEFAULT_PEN_WIDTH, bold_pen_width, clamp_pen_width, normal_pen_width, string_extents,
};
use crate::geometry::{Angle, Iu, Point, Rect, Size};

/// The schematic default text size, 50 mil.
///
/// `DEFAULT_TEXT_SIZE` in `eeschema/default_values.h`.
pub const DEFAULT_TEXT_SIZE: Iu = Iu(12_700);

/// How far an italic glyph leans, as a fraction of the text height.
///
/// `ITALIC_TILT` in `include/font/font.h:62`.
const ITALIC_TILT: (i64, i64) = (1, 8);

/// How much taller than its extents a stroke-font box is.
///
/// `EDA_TEXT::GetTextBox` (`common/eda_text.cpp:773`) calls it a fudge factor.
const FUDGE: (i64, i64) = (17, 100);

/// The height a line takes, as a fraction of the text height.
///
/// `METRICS::GetInterline` uses `m_InterlinePitch = 1.68`
/// (`include/font/font_metrics.h:60`) and `STROKE_FONT::GetInterline`
/// (`common/font/stroke_font.cpp:194-199`) scales it by 0.9583 "to match legacy
/// spacing". The product is 1.609944.
const INTERLINE: (i64, i64) = (201_243, 125_000);

/// The overbar mark, which raises the top of the box.
const OVERBAR_MARK: &str = "~{";

/// The name KiCad gives its own stroke font.
///
/// `KICAD_FONT_NAME` in `include/font/kicad_font_name.h`.
const STROKE_FONT_NAME: &str = "KiCad Font";

/// Which side of the anchor the text sits on, across the line.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum HorizontalJustify {
    /// The text starts at the anchor.
    Left,
    /// The text is centred on the anchor.
    #[default]
    Centre,
    /// The text ends at the anchor.
    Right,
}

/// Which side of the anchor the text sits on, along the line.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum VerticalJustify {
    /// The text hangs below the anchor.
    Top,
    /// The text is centred on the anchor.
    #[default]
    Centre,
    /// The text stands above the anchor.
    Bottom,
}

/// Everything about a text item except the text and where it sits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextStyle {
    /// The glyph size, width and height.
    pub size: Size,
    /// The pen the file names, when it names one.
    pub thickness: Option<Iu>,
    /// Is the text bold? Bold widens the pen; the glyphs do not change.
    pub bold: bool,
    /// Is the text italic? Italic leans the glyphs; the extents do not change.
    pub italic: bool,
    /// Which side of the anchor the text sits on, across the line.
    pub horizontal: HorizontalJustify,
    /// Which side of the anchor the text sits on, along the line.
    pub vertical: VerticalJustify,
    /// Is the text mirrored?
    ///
    /// A schematic never mirrors text: KiCad's schematic reader accepts the
    /// `mirror` keyword and drops it
    /// (`eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:862-863`).
    /// A board does mirror text, and the box swaps sides when it does.
    pub mirrored: bool,
    /// The font face the file names, when it names one.
    ///
    /// A face other than KiCad's own selects an outline font, whose widths live
    /// in a font file kicli does not read.
    pub face: Option<String>,
}

impl Default for TextStyle {
    /// The style KiCad assumes when a file names nothing.
    ///
    /// Both justifications default to centre
    /// (`common/font/text_attributes.cpp:24-41`).
    fn default() -> Self {
        Self {
            size: Size {
                x: DEFAULT_TEXT_SIZE,
                y: DEFAULT_TEXT_SIZE,
            },
            thickness: None,
            bold: false,
            italic: false,
            horizontal: HorizontalJustify::default(),
            vertical: VerticalJustify::default(),
            mirrored: false,
            face: None,
        }
    }
}

impl TextStyle {
    /// Read the `effects` of a text item, a label or a field.
    ///
    /// Anything the list does not name keeps its default.
    #[must_use]
    pub fn read(doc: &Doc, node: NodeId) -> Self {
        let mut style = Self::default();
        for &child in doc.children(node) {
            if !doc.head_is(child, "effects") {
                continue;
            }
            for &effect in doc.children(child) {
                match doc.head(effect) {
                    Some("font") => style.read_font(doc, effect),
                    Some("justify") => style.read_justify(doc, effect),
                    _ => {}
                }
            }
        }
        style
    }

    /// Read the `font` list of an `effects` list.
    fn read_font(&mut self, doc: &Doc, node: NodeId) {
        for &setting in doc.children(node) {
            match doc.head(setting) {
                Some("size") => {
                    // KiCad writes the height first, then the width.
                    let values = doc.children(setting);
                    let read = |index: usize| {
                        values
                            .get(index)
                            .and_then(|&id| doc.atom_as_iu(id))
                            .map(Iu)
                            .unwrap_or_default()
                    };
                    self.size = Size {
                        x: read(2),
                        y: read(1),
                    };
                }
                Some("thickness") => {
                    self.thickness = doc
                        .children(setting)
                        .get(1)
                        .and_then(|&id| doc.atom_as_iu(id))
                        .map(Iu);
                }
                Some("bold") => self.bold = yes(doc, setting),
                Some("italic") => self.italic = yes(doc, setting),
                Some("face") => {
                    self.face = doc
                        .children(setting)
                        .get(1)
                        .and_then(|&id| doc.atom_as_str(id));
                }
                _ => {
                    // The bare form, as an older file writes it.
                    match doc.atom_text(setting) {
                        Some("bold") => self.bold = true,
                        Some("italic") => self.italic = true,
                        _ => {}
                    }
                }
            }
        }
    }

    /// Read the `justify` list of an `effects` list.
    ///
    /// `mirror` is read nowhere: KiCad's schematic reader drops it.
    fn read_justify(&mut self, doc: &Doc, node: NodeId) {
        for &word in doc.children(node) {
            match doc.atom_text(word) {
                Some("left") => self.horizontal = HorizontalJustify::Left,
                Some("right") => self.horizontal = HorizontalJustify::Right,
                Some("top") => self.vertical = VerticalJustify::Top,
                Some("bottom") => self.vertical = VerticalJustify::Bottom,
                _ => {}
            }
        }
    }

    /// Does this style draw in KiCad's own stroke font?
    ///
    /// `FONT::IsStroke` (`common/font/font.cpp:166-171`).
    #[must_use]
    pub fn is_stroke(&self) -> bool {
        match &self.face {
            None => true,
            Some(face) => face.is_empty() || face == STROKE_FONT_NAME,
        }
    }

    /// The pen this text draws with.
    ///
    /// `EDA_TEXT::GetEffectiveTextPenWidth` (`common/eda_text.cpp:449-466`).
    /// A thickness of one unit or less means the file names none, so the pen
    /// comes from `default_pen`, or from the text width when there is no
    /// default either.
    ///
    /// # Examples
    ///
    /// ```
    /// use kicli::geometry::font::DEFAULT_PEN_WIDTH;
    /// use kicli::geometry::text::TextStyle;
    /// use kicli::geometry::Iu;
    ///
    /// let mut style = TextStyle::default();
    /// assert_eq!(style.pen_width(DEFAULT_PEN_WIDTH), DEFAULT_PEN_WIDTH);
    /// style.bold = true;
    /// // Bold ignores the default and derives a fifth of the text width.
    /// assert_eq!(style.pen_width(DEFAULT_PEN_WIDTH), Iu(2_540));
    /// ```
    #[must_use]
    pub fn pen_width(&self, default_pen: Iu) -> Iu {
        let mut pen = self.thickness.unwrap_or(Iu(0));
        if pen.0 <= 1 {
            pen = default_pen;
            if self.bold {
                pen = bold_pen_width(self.size.x);
            } else if pen.0 <= 1 {
                pen = normal_pen_width(self.size.x);
            }
        }
        clamp_pen_width(pen, self.size)
    }
}

/// Is a `(name yes)` list saying yes?
///
/// The bare `(name)` form means yes as well, which is how KiCad writes an
/// attribute that carries no value.
fn yes(doc: &Doc, node: NodeId) -> bool {
    doc.children(node).get(1).and_then(|&id| doc.atom_text(id)) != Some("no")
}

/// The box a text item occupies, before and after its own rotation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextBox {
    /// The box as it would sit unrotated.
    bounds: Rect,
    /// The point the text turns about, which is where it is drawn from.
    pivot: Point,
    /// How far the text is turned.
    angle: Angle,
    /// Is the box a guess?
    approximate: bool,
}

impl TextBox {
    /// The box as it would sit unrotated.
    #[must_use]
    pub fn bounds(self) -> Rect {
        self.bounds
    }

    /// The point the text turns about.
    #[must_use]
    pub fn pivot(self) -> Point {
        self.pivot
    }

    /// How far the text is turned.
    #[must_use]
    pub fn angle(self) -> Angle {
        self.angle
    }

    /// How wide and how tall the box is, along the text's own axes.
    #[must_use]
    pub fn size(self) -> Size {
        self.bounds.size()
    }

    /// The middle of the box, on the page.
    #[must_use]
    pub fn centre(self) -> Point {
        self.bounds.centre().rotated(self.pivot, self.angle)
    }

    /// The four corners, on the page, in the box's own order.
    #[must_use]
    pub fn corners(self) -> [Point; 4] {
        self.bounds
            .corners()
            .map(|corner| corner.rotated(self.pivot, self.angle))
    }

    /// The smallest axis-aligned box holding the turned box.
    ///
    /// Use this only where an axis-aligned answer is what is wanted. At 45
    /// degrees it is much bigger than the text.
    #[must_use]
    pub fn axis_aligned(self) -> Rect {
        let corners = self.corners();
        corners
            .iter()
            .skip(1)
            .fold(Rect::around(corners[0]), |box_so_far, &corner| {
                box_so_far.union(Rect::around(corner))
            })
    }

    /// Is the box a guess rather than a measurement?
    ///
    /// It is a guess when the text names an outline font. kicli measures the
    /// stroke font exactly and has no font file to read for any other face, so
    /// every finding built on this box must say so.
    #[must_use]
    pub fn is_approximate(self) -> bool {
        self.approximate
    }
}

/// The box a text item occupies.
///
/// `at` is the position the file records and `angle` the item's own rotation.
/// The pen is the one a schematic draws with, so the box matches what KiCad
/// plots: `GetTextBox` itself derives the pen from the text width when the item
/// names none, while every renderer passes the schematic default of 6 mil.
///
/// # Examples
///
/// ```
/// use kicli::geometry::text::{TextStyle, text_box};
/// use kicli::geometry::{Angle, Iu, Point};
///
/// let style = TextStyle::default();
/// let boxed = text_box("A", Point::new(0, 0), Angle(0), &style);
/// // The text is centred on its anchor by default.
/// assert_eq!(boxed.centre(), Point::new(0, 0));
/// assert_eq!(boxed.bounds().width(), Iu(12_918));
/// ```
#[must_use]
pub fn text_box(text: &str, at: Point, angle: Angle, style: &TextStyle) -> TextBox {
    let (size, fudge) = box_size(text, style);
    TextBox {
        bounds: Rect::from_origin(justified_origin(at, size, fudge, style), size),
        pivot: at,
        angle,
        approximate: !style.is_stroke(),
    }
}

/// How big the box is, and the fudge factor the justification then needs.
///
/// The width is the widest line and the height is the first line plus one
/// interline per line after it. Only the first line decides the overbar
/// headroom, because that is the line KiCad measures before it merges the rest.
fn box_size(text: &str, style: &TextStyle) -> (Size, i32) {
    let pen = style.pen_width(DEFAULT_PEN_WIDTH);
    let mut lines = text.split('\n');
    let first = lines.next().unwrap_or_default();
    let extents = string_extents(first, style.size, pen);

    let mut width = extents.x.0;
    let mut height = extents.y.0;
    let fudge = clamp(round_ratio(i64::from(extents.y.0) * FUDGE.0, FUDGE.1));
    if style.is_stroke() {
        height += fudge;
    }

    let mut count: i64 = 1;
    for line in lines {
        width = width.max(string_extents(line, style.size, pen).x.0);
        count += 1;
    }
    if count > 1 {
        height += clamp(round_ratio(
            (count - 1) * i64::from(style.size.y.0) * INTERLINE.0,
            INTERLINE.1,
        ));
    }
    if first.contains(OVERBAR_MARK) {
        height += extents.y.0 / 6;
    }

    (Size::new(width, height), fudge)
}

/// Where the box starts, once the justification has moved it.
///
/// The box starts at the draw position, which is right for a left and top
/// justified text that is not mirrored, and moves for every other case.
fn justified_origin(at: Point, size: Size, fudge: i32, style: &TextStyle) -> Point {
    let italic = if style.italic {
        clamp(round_ratio(
            i64::from(style.size.y.0) * ITALIC_TILT.0,
            ITALIC_TILT.1,
        ))
    } else {
        0
    };
    let lead = size.x.0 - italic;

    let mut x = at.x.0;
    match style.horizontal {
        HorizontalJustify::Left => {
            if style.mirrored {
                x -= lead;
            }
        }
        HorizontalJustify::Centre => x -= lead / 2,
        HorizontalJustify::Right => {
            if !style.mirrored {
                x -= lead;
            }
        }
    }

    let mut y = at.y.0;
    match style.vertical {
        VerticalJustify::Top => y -= fudge,
        VerticalJustify::Centre => y -= size.y.0 / 2,
        VerticalJustify::Bottom => y = y - size.y.0 + fudge,
    }

    Point::new(x, y)
}

/// Round `top / bottom` half away from zero, as KiCad's `KiROUND` does.
fn round_ratio(top: i64, bottom: i64) -> i64 {
    if top >= 0 {
        (2 * top + bottom) / (2 * bottom)
    } else {
        -((-2 * top + bottom) / (2 * bottom))
    }
}

/// Narrow a computed length to the internal-unit type.
fn clamp(value: i64) -> i32 {
    i32::try_from(value).unwrap_or(if value < 0 { i32::MIN } else { i32::MAX })
}

#[cfg(test)]
mod tests {
    use super::{HorizontalJustify, TextStyle, VerticalJustify, text_box};
    use crate::geometry::font::DEFAULT_PEN_WIDTH;
    use crate::geometry::{Angle, Iu, Point};
    use kicli_sexpr::Doc;

    /// Read the style of the one text item in a fragment.
    fn style_of(source: &str) -> TextStyle {
        let doc = Doc::parse(source).expect("the fragment parses");
        let root = doc.root().expect("the fragment has a root");
        let item = doc.children(root)[0];
        TextStyle::read(&doc, item)
    }

    #[test]
    fn outline_font_extents_are_marked_approximate() {
        let stroke = style_of("((text \"A\" (effects (font (size 1.27 1.27)))))");
        assert!(stroke.is_stroke());
        assert!(!text_box("A", Point::new(0, 0), Angle(0), &stroke).is_approximate());

        let outline = style_of("((text \"A\" (effects (font (size 1.27 1.27) (face \"Arial\")))))");
        assert_eq!(outline.face.as_deref(), Some("Arial"));
        assert!(!outline.is_stroke());
        assert!(text_box("A", Point::new(0, 0), Angle(0), &outline).is_approximate());

        // KiCad's own font is named in a file the same way, and is not an
        // outline font.
        let named =
            style_of("((text \"A\" (effects (font (size 1.27 1.27) (face \"KiCad Font\")))))");
        assert!(named.is_stroke());
        assert!(!text_box("A", Point::new(0, 0), Angle(0), &named).is_approximate());
    }

    #[test]
    fn a_style_reads_every_effect_it_is_given() {
        let style = style_of(
            "((text \"A\" (effects (font (size 2.54 1.27) (thickness 0.3) (bold yes) (italic yes)) (justify right top))))",
        );
        assert_eq!(style.size.x, Iu(12_700));
        assert_eq!(style.size.y, Iu(25_400));
        assert_eq!(style.thickness, Some(Iu(3_000)));
        assert!(style.bold);
        assert!(style.italic);
        assert_eq!(style.horizontal, HorizontalJustify::Right);
        assert_eq!(style.vertical, VerticalJustify::Top);
        // A schematic never mirrors text, whatever the justify list says.
        let mirrored = style_of("((text \"A\" (effects (justify left mirror))))");
        assert!(!mirrored.mirrored);
        // A file that names nothing gets KiCad's defaults.
        let bare = style_of("((text \"A\"))");
        assert_eq!(bare, TextStyle::default());
    }

    #[test]
    fn a_named_thickness_wins_over_the_default_pen() {
        let mut style = TextStyle {
            thickness: Some(Iu(2_000)),
            ..TextStyle::default()
        };
        assert_eq!(style.pen_width(DEFAULT_PEN_WIDTH), Iu(2_000));
        // A thickness of one unit or less is how a file says "no thickness".
        style.thickness = Some(Iu(1));
        assert_eq!(style.pen_width(DEFAULT_PEN_WIDTH), DEFAULT_PEN_WIDTH);
        // With no default either, the pen is an eighth of the text width.
        assert_eq!(style.pen_width(Iu(0)), Iu(1_588));
    }
}
