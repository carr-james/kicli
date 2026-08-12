# R11 — Rendering and annotation overlay

Status: the full pipeline was **built and run end-to-end** during this research —
`kicad-cli` SVG → viewBox crop → annotation overlay → PNG — and the resulting
images inspected (§3, §4). The rasteriser licensing question has a clean answer
(§5).

Prerequisite reading: [`kicad-cli.md`](kicad-cli.md) §5 (SVG export flags and
document structure).

---

## ⚠ Contradictions and cautions for `spec/SPEC.md`

1. **SPEC §10 says renders are "passive output only" (Constitution §4).** The
   pipeline below preserves that: nothing rendered ever feeds the score. Worth
   restating in `AGENT.md`, because the annotation overlay makes renders look
   like analysis output and an agent may be tempted to treat a badge as data.
   The badges are *derived from* the structured views; the views are the truth.

2. **KiCad's SVG is not byte-reproducible**: it embeds a timestamp
   (`<title>SVG Image created as geomtest.svg date 2026-08-12T03:20:36 </title>`).
   Any golden-image test must strip or normalise it (§7). SPEC's fixture policy
   should say so.

3. **SPEC §10's `--annotate uuids` is expensive and probably wrong by default.**
   A UUID badge per object is unreadable at sheet scale. `refdes`, `findings`,
   and `grid` are the useful modes; `uuids` should be region-only and
   truncated to 8 hex chars.

---

## 1. Pipeline

```
 ┌─ kicad-cli sch export svg -n -e [--black-and-white] [--pages N]
 │      → full-page SVG, user units = mm, origin = page top-left
 ├─ crop      : rewrite viewBox + width/height           (§3)
 ├─ annotate  : append one <g id="kicli-annotations">    (§4)
 ├─ rasterise : resvg/tiny-skia at a chosen scale        (§5)
 └─ emit      : .svg and/or .png, plus a JSON manifest   (§6)
```

Stages 2–4 are pure text/graphics work in kicli; only stage 1 needs `kicad-cli`.

---

## 2. Why the source SVG is well suited to this

From `kicad-cli.md` §5.3–5.4, verified:

- **User units are millimetres and coordinates match the schematic file
  directly.** A region in schematic coordinates is a region in SVG coordinates —
  no transform, no scaling, no y-flip.
- Objects are grouped by style: `<g style="fill:none; stroke:#840000; …">`, so
  colour/layer filtering is a group-level operation.
- Every text item appears as an invisible `<text …>` carrying the string plus a
  `<g class="stroked-text"><desc>TEXT</desc>` group holding the strokes. Text can
  therefore be *found* in the SVG without geometry inference.

---

## 3. Region crop (verified)

Rewrite exactly one attribute group on the root `<svg>`:

```
width="{w·s}mm" height="{h·s}mm" viewBox="{x0} {y0} {w} {h}"
```

where `(x0, y0, w, h)` is the region in mm and `s` is a display magnification
(the SVG's intrinsic size; independent of raster scale).

**Verified**: cropping a KiCad-exported sheet to `viewBox="89.6 13.4 24.0 24.0"`
and rendering produced exactly the expected single symbol (`R4`, rotated 270°,
its text sideways), with wires clipped at the window edge.

```sh
# recipe R11-A
kicad-cli sch export svg -o out -n -e geomtest.kicad_sch
python3 - <<'PY'   # the crop is a one-line regex substitution
re.sub(r'width="[^"]*mm" height="[^"]*mm" viewBox="[^"]*"',
       f'width="{w*4}mm" height="{h*4}mm" viewBox="{x0} {y0} {w} {h}"', src, count=1)
PY
"Google Chrome" --headless --screenshot=crop.png --window-size=800,800 crop.svg
```

Notes:

- **No clipping is needed** — content outside the viewBox is simply not shown.
  Elements are *not* removed, so the cropped file is the same size as the full
  one. If output size matters (it does for agent context when returning SVG
  rather than PNG), prune elements whose bbox misses the window; that is an
  optimisation, not a correctness requirement.
- Region selection maps directly onto SPEC §10's forms: `--region x0,y0,x1,y1`
  and `--around R7 --radius 50mm` (resolve `R7` → its `full()` box from
  `geometry.md` §6, inflate by the radius).
- Always snap the window outward to the grid and add a small margin (default
  2 G ≈ 2.54 mm) so pins at the edge are not sliced.

---

## 4. Annotation overlay (verified)

Append a single group immediately before `</svg>`:

```xml
<g id="kicli-annotations">
  <g stroke="#c8d8e8" stroke-width="0.05" opacity="0.9"> … grid lines … </g>
  <g> <rect x="101.0" y="20.4" width="9.2" height="2.6" rx="0.6" fill="#1f6feb" opacity="0.92"/>
      <text x="105.6" y="22.35" font-size="2" fill="#fff" text-anchor="middle"
            font-family="monospace">R4 90</text> </g>
  <g> <circle cx="105.41" cy="25.4" r="1.6" fill="none" stroke="#e5534b" stroke-width="0.35"/>
      <text x="107.4" y="29.6" font-size="1.8" fill="#e5534b"
            font-family="monospace">KI-GRID-001</text> </g>
</g>
```

Rendered and inspected: grid ticks, a blue refdes/orientation badge, and a red
lint marker with its rule id all composited correctly over KiCad's own drawing.

Design rules learned from doing it:

1. **One top-level group, one id.** Makes annotations strippable
   (`kicli sch render --no-annotate` on a cached SVG) and keeps kicli's marks
   from being confused with KiCad's.
2. **All annotation coordinates are schematic mm** — same space as everything
   else. No transforms.
3. **Clip to the window.** In the verified render the finding label
   `KI-GRID-001` ran off the right edge and was cut. Annotation text must be
   placed inside the viewBox: choose the side with room, and fall back to a
   numbered marker plus a legend block in a corner when there is none. This is
   the main implementation subtlety.
4. **Font must be generic** (`monospace`/`sans-serif`), never a specific family,
   because the rasteriser's font set is not the browser's. `resvg` needs a font
   database; ship with a fallback and keep annotation text short.
5. **Size annotation text in mm relative to the region**, not in absolute mm, or
   badges will be illegible on a wide region and gigantic on a tight one.
   Rule: `font-size = clamp(region_height/40, 1.0, 3.0)` mm.

### 4.1 Annotation modes

| Mode | Content | Notes |
|---|---|---|
| `refdes` | badge per symbol: `R4` (+ orientation when `--verbose`) | default when a region is small enough |
| `findings` | marker + rule id per R8/ERC finding in view | colour by severity; legend when > 6 markers |
| `grid` | ticks every 4 G, heavier every 20 G | cheap orientation aid; also makes off-grid items obvious |
| `nets` | net name at one point per net in view | most useful for wiring work |
| `uuids` | 8-char uuid per object | region-only, opt-in (Contradiction 3) |

Modes compose; kicli emits them as sibling groups inside `#kicli-annotations` so
each can be toggled.

---

## 5. Rasterisation

### 5.1 Crate choice and licences (checked 2026-08-12 via crates.io API)

| Crate | Version | Licence | Verdict |
|---|---|---|---|
| `resvg` | 0.48.1 | **Apache-2.0 OR MIT** | ✅ use |
| `usvg` | 0.48.1 | Apache-2.0 OR MIT | ✅ (pulled in by resvg) |
| `tiny-skia` | 0.12.0 | BSD-3-Clause | ✅ (resvg's backend) |
| `png` | 0.18.1 | MIT OR Apache-2.0 | ✅ |
| `oxipng` | 10.2.0 | MIT | optional, for size |

**Note for the record:** resvg/usvg were MPL-2.0 in older releases. The current
releases are MIT/Apache dual-licensed and therefore satisfy Constitution §9
without an exception. Pin the version and re-check on upgrade.

Rejected: `librsvg` (LGPL + C deps), shelling out to Inkscape/ImageMagick
(heavy, non-deterministic across versions), headless Chrome (used here only as a
verification tool — it is not a dependency kicli should acquire).

### 5.2 Scale and DPI

The SVG's intrinsic size is in mm. Browsers map mm→px via CSS at 96 dpi
(1 mm ≈ 3.7795 px), which is why the 96 mm × 96 mm verification crop rendered at
~363 px inside a 800 px window. `resvg` instead takes an explicit target size or
zoom, which is what kicli wants: **specify output pixels directly**, never dpi.

Recommended default: fit the long edge to `render.max_px` (default **1600**),
clamped so the effective resolution is ≥ 6 px/mm (below that, 1.27 mm text is
unreadable to a vision model). If the region is too large to satisfy both, emit
at 1600 px and warn that text may be illegible, suggesting a smaller region —
the agent then knows to zoom rather than squint.

### 5.3 Measured output sizes

| Image | Content | PNG |
|---|---|---|
| 24 × 24 mm crop, 800 px | one resistor | 5.4 KB |
| same + annotations, 900 px | + grid, badge, marker | 14 KB |

Full-sheet renders of the big demo sheets produce 5.6 MB of *SVG* for a 9-sheet
project (`kicad-cli.md` §1.1); PNG at 1600 px is far smaller. Neither is a
problem, but agents should be handed **regions**, not full sheets, whenever the
task is local — which is the entire point of SPEC §10.

---

## 6. Output contract

`kicli sch render` returns the file path(s) plus a manifest so the agent knows
what it is looking at:

```json
{ "sheet": "/Power",
  "region": { "x0": 89.6, "y0": 13.4, "x1": 113.6, "y1": 37.4, "units": "mm" },
  "px": { "w": 1600, "h": 1600, "mm_per_px": 0.015 },
  "annotations": ["refdes", "grid"],
  "objects_in_view": 12,
  "clipped_annotations": 0,
  "files": { "svg": "…/r7.svg", "png": "…/r7.png" },
  "source": { "tool": "kicad-cli", "version": "10.0.5" } }
```

`objects_in_view` and `clipped_annotations` are the two fields that let an agent
self-correct without looking: zero objects means the region is wrong; non-zero
clipped annotations means widen it.

---

## 7. Determinism and testing

- **Strip the timestamp.** KiCad writes
  `<title>SVG Image created as X date …</title>`; kicli must normalise it (and
  the `<desc>Image generated by Eeschema-SVG</desc>`) before hashing or diffing.
  With that removed, repeated exports of an unchanged sheet are byte-identical.
- **Golden tests** compare the *cropped, annotated SVG text* (normalised), not
  the PNG. Rasteriser output can shift by a pixel across versions; the SVG
  cannot.
- **One PNG smoke test** per platform, asserting non-blank output and expected
  ink coverage in a known window — cheap protection against "the font database
  was empty and all text vanished", which is the classic resvg failure.
- Annotation placement must be deterministic: when choosing which side of an
  object a badge goes, order candidate positions in a fixed sequence and take
  the first that fits.

---

## 8. Interaction with the rest of the system

- Region coordinates come from R10's layout digest; the agent reads a bbox and
  passes it straight in.
- Findings come from R8 with `pos` + `objects`; the overlay needs no extra data.
- The `<desc>`/`class="stroked-text"` structure (§2) means a future
  "highlight this text item" mode can select KiCad's own drawing of it rather
  than drawing a box near it.
- The font-metric calibration in `kicad-cli.md` §5.5 uses this same pipeline —
  one more reason to keep the SVG stage as a reusable module rather than a
  render-only detail.

---

## 9. Open questions for James

- **Q1 — Default render style.** `--black-and-white` (best for vision models,
  smallest, no theme dependence) or KiCad's colour theme (closer to what you see
  in the GUI)? Recommendation: black-and-white default for `--output json`
  consumers, colour when a human asked.

- **Q2 — `render.max_px` = 1600 and the 6 px/mm legibility floor.** Both are
  judgement calls made here; confirm or set your own.

- **Q3 — SVG or PNG by default?** PNG is what a vision model consumes; SVG is
  smaller for sparse regions and stays crisp. Recommendation: emit both, return
  both paths, let the agent choose.

- **Q4 — Caching.** Full-sheet SVG export is ~0.2–0.4 s; a naive implementation
  re-exports for every region. Cache the exported SVG keyed on the sheet's
  content hash (R10 §5) under `.kicli/render/`? Recommended.

---

## 10. Reproduction

| § | How |
|---|---|
| 3 | recipe R11-A above; inspect `crop.png` |
| 4 | append the `<g id="kicli-annotations">` block to the cropped SVG, re-render |
| 5.1 | `curl -s https://crates.io/api/v1/crates/resvg \| jq '.versions[0].license'` |
| 5.2 | render the same 96 mm SVG at several window sizes and measure ink extent |
| 7 | export the same sheet twice, `diff` — only the `<title>` timestamp differs |

Verification tooling used here: headless Google Chrome for rasterisation
(`--headless --screenshot`), because no Rust toolchain was available in this
environment. `resvg` should be substituted when the real pipeline is built, and
the §3/§4 outputs re-verified against it.
