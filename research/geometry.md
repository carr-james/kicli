# R7 — Geometry engine prerequisites

Status: pin-placement maths **empirically verified against KiCad 10.0.5 itself**
(§3.4 — 16/16 predicted pin coordinates match KiCad's ERC output exactly). Text
bounding-box maths is specified from source at tag `10.0.5`; its one missing
input (glyph advance widths) is a licensing question, see §5.4 and Q2.

Prerequisite reading: [`sch-format.md`](sch-format.md) §2.3 (units) and §3.4
(symbol record).

---

## ⚠ Contradictions and cautions for `spec/SPEC.md`

1. **`kicli sch erc` cannot use `kicad-cli`'s JSON coordinates as-is.** KiCad
   10.0.5's ERC JSON exporter reports schematic coordinates **100× too small**
   while labelling them `"mm"`. Root cause: `eeschema/erc/erc_report.cpp:161`
   constructs `UNITS_PROVIDER unitsProvider( pcbIUScale, m_reportUnits )` — the
   PCB scale (1e6 IU/mm) applied to schematic data (1e4 IU/mm). The plain-text
   report path (`erc_report.cpp:63`) correctly uses `schIUScale`. Still present
   on `master` (10.99). Verified empirically in §3.5. SPEC §4 `kicli sch erc`
   must either multiply by 100 (version-gated) or parse the text report.
   This is a bug worth filing upstream — see Q4.

2. **SPEC §7 grid discipline needs a second grid.** The Constitution fixes the
   *placement* grid at 50 mil, but KiCad's own defaults put field text on a
   finer implied grid (autoplaced fields land on arbitrary IU, e.g. `246.7512`
   in the corpus). A hard "everything on 50 mil" lint would flag every
   KiCad-autoplaced field. Recommendation: grid rule applies to
   *connectable* geometry (pins, wire ends, junctions, labels, sheet pins) and
   not to field text. Needs James's decision (Q1).

3. **Nothing in SPEC covers symbol bounding boxes**, but every readability lint
   (overlap, crowding, text-over-wire) needs them, and they are strictly harder
   than pin positions because they involve text extents. §5 specifies them;
   §5.4 flags the licensing constraint that follows.

---

## 1. Coordinate systems — three of them, and two Y directions

| Space | Origin | +Y | Where |
|---|---|---|---|
| Schematic file | page top-left | **down** | every `(at …)` in a `.kicad_sch` outside `lib_symbols` |
| Library file | symbol anchor | **up** | every `(at …)` inside `lib_symbols` / `.kicad_sym` |
| KiCad internal | page top-left | **down** | in-memory; what the sources compute in |

The library→internal flip is done at parse time, not at draw time:

```cpp
// sch_io_kicad_sexpr_parser.h:169
VECTOR2I parseXY( bool aInvertY = false )
{
    xy.x = parseInternalUnits( "X coordinate" );
    xy.y = aInvertY ? -parseInternalUnits( "Y coordinate" ) : parseInternalUnits( "Y coordinate" );
}
```

`parseXY( true )` is used in exactly 19 places, all inside the library-symbol
parser (pins, graphics, library field positions). The writer negates on the way
out (`sch_io_kicad_sexpr_lib_cache.cpp:672,699,751` — `-aPin->GetPosition().y`).

**Rule for kicli:** on reading `lib_symbols` content, negate Y immediately; on
writing it back, negate again. Everything else in kicli works in schematic
(Y-down) space.

Units: 1 IU = 100 nm, `int32`. See `sch-format.md` §2.3. All geometry below is
in IU unless a mm value is quoted from a file.

---

## 2. The symbol transform

### 2.1 Representation

KiCad carries a 2×2 integer matrix per placed symbol
(`libs/kimath/include/transform.h`):

```cpp
class TRANSFORM { public: int x1, y1, x2, y2; };
TRANSFORM() : x1(1), y1(0), x2(0), y2(1) {}   // identity
```

applied as (`libs/kimath/src/transform.cpp:44`):

```cpp
VECTOR2I TRANSFORM::TransformCoordinate( const VECTOR2I& p ) const
{
    return VECTOR2I( x1 * p.x + y1 * p.y,
                     x2 * p.x + y2 * p.y );
}
```

Note the unusual member naming: the matrix rows are `(x1, y1)` and `(x2, y2)`.
It is **not** `[[x1,x2],[y1,y2]]`. Getting this backwards silently swaps the 90°
and 270° cases, which is the classic third-party-tool bug.

### 2.2 Building it from the file

The parser sets the matrix from `(at x y rot)`
(`sch_io_kicad_sexpr_parser.cpp:3182-3196`):

| `rot` | `TRANSFORM(x1,y1,x2,y2)` |
|---|---|
| 0 | `(1, 0, 0, 1)` |
| 90 | `(0, 1, -1, 0)` |
| 180 | `(-1, 0, 0, -1)` |
| 270 | `(0, -1, 1, 0)` |

Anything else is a parse error (`Expecting( "0, 90, 180, or 270" )`).

Then `(mirror x|y)` is applied as an **incremental** transform
(`sch_io_kicad_sexpr_parser.cpp:3198-3206` → `SCH_SYMBOL::SetOrientation`,
`sch_symbol.cpp:2393`), with

```
MIRROR_X = (1, 0, 0, -1)      MIRROR_Y = (-1, 0, 0, 1)
```

composed as (`sch_symbol.cpp:2537-2541`):

```
new.x1 = m.x1*t.x1 + m.x2*t.y1
new.y1 = m.y1*t.x1 + m.y2*t.y1
new.x2 = m.x1*t.x2 + m.x2*t.y2
new.y2 = m.y1*t.x2 + m.y2*t.y2
```

Order matters and is fixed by the file: `at` (hence rotation) is always written
before `mirror`, and the parser applies them in file order.

### 2.3 The eight orientations, resolved

Composing the table above gives the complete set kicli needs (verified in §3.4):

| rot | mirror | x1 | y1 | x2 | y2 | maps (px,py) → |
|---|---|---|---|---|---|---|
| 0 | — | 1 | 0 | 0 | 1 | (px, py) |
| 90 | — | 0 | 1 | -1 | 0 | (py, −px) |
| 180 | — | −1 | 0 | 0 | −1 | (−px, −py) |
| 270 | — | 0 | −1 | 1 | 0 | (−py, px) |
| 0 | x | 1 | 0 | 0 | −1 | (px, −py) |
| 0 | y | −1 | 0 | 0 | 1 | (−px, py) |
| 90 | x | 0 | 1 | 1 | 0 | (py, px) |
| 90 | y | 0 | −1 | −1 | 0 | (−py, −px) |

(180/270 + mirror reduce to entries already in this table — the group has only
8 elements. kicli should normalise to the `(rot, mirror)` pair KiCad would
write, which is what the GUI does when you rotate a mirrored symbol.)

---

## 3. Pin position resolution

### 3.1 The formula

```
abs_pin = symbol.at + M · (lib_pin.x, −lib_pin.y)
```

where `M` is the §2.3 matrix, `symbol.at` is the schematic-space placement, and
`lib_pin` is the pin's `(at …)` as it appears in the file's `lib_symbols`
(library space, Y-up) — hence the explicit negation of Y.

Source chain: `SCH_PIN::GetPosition` (`eeschema/sch_pin.cpp:254-260`)

```cpp
return symbol->GetTransform().TransformCoordinate( m_position ) + symbol->GetPosition();
```

with `m_position` already Y-flipped at parse time (§1); identical logic in
`SCH_SYMBOL::GetPinPhysicalPosition` (`sch_symbol.cpp:2954-2960`).

### 3.2 Which pins exist

A placed symbol shows the pins of the child library symbols whose name suffix
matches its `(unit N)` and `(body_style N)`, **plus** those with unit 0 and/or
body style 0 (the "common to all" pseudo-units). See `sch-format.md` §4 for the
`NAME_<unit>_<bodyStyle>` convention.

### 3.3 Pin geometry (the drawn segment)

The library pin record is `(pin <electrical_type> <graphic_style> (at X Y ANGLE) (length L) …)`.

- `(at X Y)` is the **connection point** (the free end that wires attach to).
- `ANGLE` ∈ {0, 90, 180, 270} maps to `PIN_RIGHT / PIN_UP / PIN_LEFT / PIN_DOWN`
  (`sch_io_kicad_sexpr_parser.cpp:1670-1676`).
- The pin body segment runs from the connection point *in the direction of
  ANGLE* (library space, Y-up) for `length`.

Verified against the shipped `Device:R`
(`/Applications/KiCad/KiCad.app/Contents/SharedSupport/symbols/Device.kicad_sym`):
body rectangle spans y ∈ [−2.54, 2.54]; pin 1 is `(at 0 3.81 270)` `(length 1.27)`
— 3.81 − 1.27 = 2.54, i.e. it runs *down* from its tip to the body edge. Pin 2
is `(at 0 -3.81 90)`, running *up* to −2.54. Both consistent.

After transform, the drawn direction is `M · dir(ANGLE_flipped)`; the connection
point is what matters for connectivity and for the grid lint.

### 3.4 Empirical verification (recipe R7-A)

The whole chain was checked against KiCad by exploiting the fact that ERC
reports the coordinates of every unconnected pin.

```sh
# 1. Generate a schematic placing Device:R at all 8 orientations at known
#    positions (script: exp/r7 generator, embeds the R symbol in lib_symbols).
# 2. Ask KiCad where the pins are:
kicad-cli sch erc --format json --severity-all -o erc.json geomtest.kicad_sch
kicad-cli sch erc --format report --units mm -o erc.txt geomtest.kicad_sch
# 3. Compare "Symbol Rn Pin p" positions against the formula in §3.1.
```

Result — all 16 pins match exactly (KiCad JSON values ×100, see §3.5):

| ref | rot | mirror | pin | predicted (mm) | KiCad (mm) |
|---|---|---|---|---|---|
| R1 | 0 | — | 1 | 25.4, 21.59 | 25.4, 21.59 |
| R1 | 0 | — | 2 | 25.4, 29.21 | 25.4, 29.21 |
| R2 | 90 | — | 1 | 46.99, 25.4 | 46.99, 25.4 |
| R2 | 90 | — | 2 | 54.61, 25.4 | 54.61, 25.4 |
| R3 | 180 | — | 1 | 76.2, 29.21 | 76.2, 29.21 |
| R3 | 180 | — | 2 | 76.2, 21.59 | 76.2, 21.59 |
| R4 | 270 | — | 1 | 105.41, 25.4 | 105.41, 25.4 |
| R4 | 270 | — | 2 | 97.79, 25.4 | 97.79, 25.4 |
| R5 | 0 | x | 1 | 25.4, 54.61 | 25.4, 54.61 |
| R5 | 0 | x | 2 | 25.4, 46.99 | 25.4, 46.99 |
| R6 | 0 | y | 1 | 50.8, 46.99 | 50.8, 46.99 |
| R6 | 0 | y | 2 | 50.8, 54.61 | 50.8, 54.61 |
| R7 | 90 | x | 1 | 72.39, 50.8 | 72.39, 50.8 |
| R7 | 90 | x | 2 | 80.01, 50.8 | 80.01, 50.8 |
| R8 | 90 | y | 1 | 105.41, 50.8 | 105.41, 50.8 |
| R8 | 90 | y | 2 | 97.79, 50.8 | 97.79, 50.8 |

**This table is the M2 fixture.** The generated schematic, the ERC JSON, and the
comparison script should be committed to `fixtures/geometry/` and the assertion
re-run in `cargo test`, satisfying Constitution §11.

Sanity notes visible in the table: mirror-x on a vertical part swaps pin 1/2
(R1 vs R5); rot 90 vs rot 270 mirror each other about the anchor (R2 vs R4);
rot 90 + mirror y is *not* the same as rot 270 (R8 = R4 here only because the
part is symmetric about its own axis — do not generalise from a resistor, use an
asymmetric fixture symbol as well).

### 3.5 The ERC JSON unit bug (recipe R7-B)

Same file, same run, two exporters:

```
erc.txt : @(25.40 mm, 21.59 mm): Symbol R1 Pin 1 [Passive, Line]      ← correct
erc.json: "coordinate_units": "mm", "pos": {"x": 0.254, "y": 0.2159}  ← 100× small
```

`eeschema/erc/erc_report.cpp`:

```cpp
:63   UNITS_PROVIDER unitsProvider( schIUScale, m_reportUnits );   // text report — right
:161  UNITS_PROVIDER unitsProvider( pcbIUScale, m_reportUnits );   // JSON report — wrong
```

`pcbIUScale` is 1e6 IU/mm, `schIUScale` is 1e4 (`include/base_units.h:72,111-114`),
hence exactly 100×. Confirmed unfixed on `master`.

Implication for kicli: `sch erc --output json` must scale by 100 when the
detected `kicad_version` is affected, and must have a self-test that would catch
the day it is fixed (e.g. assert a known fixture's coordinates against the text
report as well). Do not silently trust the header's `coordinate_units`.

---

## 4. Field and label text position

### 4.1 Instance fields are absolute

A `property` inside a placed `symbol` (or `sheet`, or `global_label`) carries an
**absolute schematic-space** `(at x y rot)`. Example from the corpus
(`ampli_ht.kicad_sch:2703`): symbol at `148.59 162.56`, its `Reference` field at
`148.59 165.1`. The field's `rot` is the text's own angle (0 or 90 for
schematic text), independent of the symbol's rotation value in the file.

Consequences for the `kicli field move` command (SPEC §4, D3):

- Moving a field is a direct edit of that `property`'s `(at …)`; no transform
  composition is needed. This is the operation Konnect cannot do (R6) and it is
  cheap.
- Moving/rotating the *symbol* must move its fields too if kicli wants to match
  GUI behaviour — the GUI moves fields with the symbol and (for rotation)
  re-derives field positions. kicli should move fields rigidly with the symbol
  by default and expose `--keep-field-positions`. Needs confirmation (Q3).
- `fields_autoplaced yes` marks fields KiCad may reposition on its own. If kicli
  hand-places a field it should clear that flag, or KiCad will overwrite the
  work. **This is the specific mechanism behind the "text repositioning gap"
  complaint about other tools** — see R6.

### 4.2 Library fields are relative and Y-up

Property positions inside `lib_symbols` are relative to the symbol anchor in
library space, and are Y-flipped exactly like pins
(`sch_io_kicad_sexpr_lib_cache.cpp:672`). They serve as the template from which
KiCad derives an instance field's absolute position when a symbol is first
placed.

---

## 5. Text bounding boxes

Everything here is `EDA_TEXT::GetTextBox` (`common/eda_text.cpp:742-870`) plus
`FONT::StringBoundaryLimits` (`common/font/font.cpp:451-478`) and
`STROKE_FONT::GetTextAsGlyphs` (`common/font/stroke_font.cpp:203-292`).

### 5.1 Single-line extents (stroke font)

```
cursor = 0
for each char c:
    if c == ' ' : cursor += size.x * spaceWidth              # spaceWidth = bbox width of glyph 0
    elif c == '\t': cursor snaps to next multiple of 4 base-widths
    else        : cursor += size.x * glyphBBox[c - ' '].end.x
extents.x = cursor − size.x * INTER_CHAR          # INTER_CHAR = 0.2
extents.y = size.y
```

then (`font.cpp:466-471`) for stroke fonts the box is inflated by
`round(thickness * 1.5)` on all sides "to catch diacriticals, descenders".

Unknown/unprintable code points are substituted with `'?'`
(`stroke_font.cpp:262-266`).

### 5.2 Box assembly, multi-line, and justification

From `GetTextBox`:

```
thickness      = effective pen width (§5.3)
extents        = StringBoundaryLimits(text, size, thickness, bold, italic)
fudgeFactor    = round(extents.y * 0.17)          # stroke fonts only
textsize.y    += fudgeFactor
if text contains "~{" : textsize.y += extents.y / 6     # overbar headroom
multi-line     : textsize.x = max over lines
                 textsize.y += (nLines − 1) * interline
italicOffset   = italic ? round(size.y * ITALIC_TILT) : 0    # ITALIC_TILT = 1/8

origin = drawPos   (i.e. the field's (at) position)
horizontal:
  LEFT   : if mirrored, x −= (w − italicOffset)
  CENTER : x −= (w − italicOffset) / 2
  RIGHT  : if !mirrored, x −= (w − italicOffset)
vertical:
  TOP    : offset y by −fudgeFactor
  CENTER : y −= h / 2
  BOTTOM : y −= h ; then offset y by +fudgeFactor
normalize()
```

Interline (`stroke_font.cpp:194-199`, `include/font/font_metrics.h:53-58`):

```
interline = size.y * m_InterlinePitch * 0.9583        # m_InterlinePitch = 1.68
          = size.y * 1.609944
```

Other metric constants (`font_metrics.h:60-63`): `m_OverbarHeight = 1.23`,
`m_UnderlineOffset = -0.16`.

The box is computed **unrotated**; text rotation is applied by rotating the box
about `drawPos` (`EDA_TEXT::TextHitTest`, `eda_text.cpp:872-877`, uses
`GetRotated( aPoint, GetDrawPos(), -GetDrawRotation() )`). For lint purposes
kicli should carry an oriented box (centre, size, angle) rather than an AABB,
because schematic text is routinely at 90°.

### 5.3 Effective pen width

`EDA_TEXT::GetEffectiveTextPenWidth` (`eda_text.cpp:449-462`): if the stored
`(thickness …)` is ≤ 1 IU, derive it from the text width
(`common/gr_text.cpp:35-68`):

```
bold   : round(size / 5)
normal : round(size / 8)
demibold (outline fonts only) : round(size / 6)
```

where `size = min(size.x, size.y)`.

Schematic defaults (`eeschema/default_values.h`): `DEFAULT_TEXT_SIZE 50` mil
(= 1.27 mm), `DEFAULT_LINE_WIDTH_MILS 6`, `DEFAULT_WIRE_WIDTH_MILS 6`,
`DEFAULT_JUNCTION_DIAM 36` mil.

### 5.4 The glyph table problem (licensing)

Exact text extents require the per-glyph advance widths of KiCad's default
stroke font, "Newstroke". In KiCad these live in
`common/newstroke_font.cpp`, whose header reads:

> Copyright (C) 2010 vladimir uryvaev … This program is free software; you can
> redistribute it and/or modify it under the terms of the **GNU General Public
> License** … version 2 … or (at your option) any later version.

Constitution §9 forbids vendoring that into an MIT/Apache-2.0 tool. Options, in
order of preference:

1. **Measure, don't copy.** Derive the ~95 ASCII advance widths (and any others
   needed) by rendering a calibration sheet with `kicad-cli sch export svg` — a
   build-time/one-off *measurement of behaviour*, stored as our own numeric
   table with our own provenance note. Facts about spacing are not the font
   program. This also gives an exactness check against the real renderer, and
   reuses the R11 pipeline. **Recommended.**

   **Addendum (from R3, after this section was first written): the measurement
   is easier than expected.** KiCad's SVG plotter emits, for every text item, an
   invisible `<text …  textLength="…">` element whose `textLength` comes from
   KiCad's own font engine, alongside the stroke paths
   (`kicad-cli.md` §5.4). A 94-glyph calibration run shows
   `textLength(XY) − textLength(X) − textLength(Y)` is a constant
   −0.4572 mm = −3 × pen width for every tested pair, so advances are
   recoverable directly as `advance(c) = textLength(c) − 3·penWidth`
   (`kicad-cli.md` §5.5, with sample values and the caveat that the relation to
   `GetTextBox`'s `INTER_CHAR` term still needs confirming across sizes and
   styles).
2. Use the upstream Newstroke project directly
   (<http://vovanium.ru/sledy/newstroke>, author Vladimir Uryvaev; KiCad ships a
   copy of the glyph sources under `tools/newstroke/` with a README pointing
   there). Its standalone licence terms were **not** established in this
   research — see Q2.
3. Approximate with a fixed advance ratio. Cheap, but every text-overlap lint
   inherits systematic error; unacceptable for a scoring tool that must match
   human judgement (SPEC §9).

Non-default fonts: `(effects (font (face "Arial") …))` selects an outline font,
where extents depend on a system font file. kicli cannot reproduce those exactly
without a font stack. Recommendation: compute stroke-font extents exactly,
detect `face` and mark affected lint findings as `approximate` in the output.

---

## 6. Symbol bounding boxes

For overlap/crowding rules, kicli needs a symbol's box. Composition:

```
body box   = union over the selected unit/body-style child symbols of:
             rectangle/polyline/arc/circle/bezier extents (library space)
pin box    = for each visible pin: segment from connection point to body end,
             plus pin name/number text extents when displayed
             (pin_names offset, pin_numbers hide — see lib symbol header)
field box  = union over visible instance fields of §5 boxes (already absolute)
symbol box = symbol.at + M · (body ∪ pin boxes, Y-flipped)   ∪   field boxes
```

Note that `TRANSFORM::TransformCoordinate( BOX2I )` transforms only the two
corners and relies on `Normalize()` (`transform.cpp:50-56`), which is correct
for the axis-aligned 90° group but would be wrong for arbitrary angles —
harmless here, since symbols only take the 8 orientations.

Two boxes are worth distinguishing in kicli's model, because lint rules want
different ones:

- **body box** — graphics + pins, no text. Used for "symbols overlap".
- **full box** — including visible fields. Used for "text collides with
  something".

KiCad itself makes the same distinction (`SCH_SYMBOL::GetBodyBoundingBox()` vs
`GetBoundingBox()`), which is a good sign the split is the right one.

---

## 7. Grid

- Default schematic grid: 50 mil = 1.27 mm = **12700 IU**.
- Snap = round-half-away-from-zero to the nearest multiple, applied to
  *connectable* geometry (§Contradiction 2).
- Off-grid detection is exact integer arithmetic: `pos.x % 12700 != 0`.
- Common legacy hazard: files converted from KiCad 5 mil-based coordinates land
  on 12700-IU multiples anyway; genuinely off-grid pins usually come from
  hand-edited or generated files — which is exactly what kicli produces, hence
  the Constitution §7 rule.

---

## 8. Implications for kicli's design

1. Geometry module works in `i32` IU with a `Point`/`Box` newtype; mm only at
   the CLI boundary.
2. One `Transform` type with the 8 orientations as an enum plus the matrix
   table from §2.3; `from_file(rot, mirror)` and `to_file()` round-trip.
3. `PinResolver`: `(lib_symbols, symbol instance) → Vec<ResolvedPin { number,
   name, position, direction, electrical_type, uuid }>`. Everything downstream
   (connectivity extraction, wire routing, ERC-style lints, `sch view`) consumes
   this.
4. Text metrics behind a trait, with the exact stroke implementation as the
   default and an `Approximate` marker on findings when the font is an outline
   font.
5. The §3.4 table is the first fixture. Add a second with an **asymmetric**
   symbol (distinct pin coordinates on all four sides) so mirror/rotate
   confusion cannot pass.

---

## 9. Open questions for James

- **Q1 — Grid rule scope.** Does the blocking off-grid lint apply only to
  connectable geometry (pins, wire endpoints, junctions, no-connects, labels,
  sheet pins), with field text exempt? (Recommended: yes, else KiCad's own
  autoplaced fields fail the lint.)

- **Q2 — Font metrics source.** Approve option 1 in §5.4 (measure advance widths
  from KiCad's own SVG output and store our own table with provenance), rather
  than vendoring Newstroke? The R3 addendum shows the measurement is a single
  `kicad-cli sch export svg` run against a generated calibration sheet, so the
  cost is low. If you'd rather use upstream Newstroke directly, someone needs to
  establish its standalone licence first.

- **Q3 — Field behaviour on symbol move/rotate.** Default: fields move rigidly
  with the symbol; rotation keeps field angles unchanged and rotates their
  positions about the symbol anchor. Confirm, and confirm that
  `fields_autoplaced` is cleared whenever kicli sets a field position
  explicitly.

- **Q4 — Upstream bug report.** Shall I write up the ERC JSON 100× unit bug
  (§3.5) as a KiCad issue? It affects any tool consuming `--format json`, not
  just kicli.

---

## 10. Source index (tag `10.0.5`)

| Topic | File:line |
|---|---|
| Transform matrix | `libs/kimath/include/transform.h:46-67`, `libs/kimath/src/transform.cpp:44-56` |
| rot → matrix | `eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3182-3196` |
| mirror composition | `eeschema/sch_symbol.cpp:2393-2545` |
| Y flip on library coords | `eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.h:169-178` |
| pin absolute position | `eeschema/sch_pin.cpp:254-260`, `eeschema/sch_symbol.cpp:2954-2960` |
| pin angle → orientation | `.../sch_io_kicad_sexpr_parser.cpp:1668-1677` |
| text box | `common/eda_text.cpp:742-870` |
| string extents | `common/font/font.cpp:451-478` |
| glyph advance loop | `common/font/stroke_font.cpp:203-292` |
| font metrics constants | `include/font/font_metrics.h:33-66`, `include/font/font.h:62` |
| pen widths | `common/gr_text.cpp:35-68` |
| schematic defaults | `eeschema/default_values.h` |
| ERC report units | `eeschema/erc/erc_report.cpp:63,161` |
| IU scales | `include/base_units.h:60-115` |
