# R3 — `kicad-cli` capabilities audit (KiCad 10.0.5)

Status: every flag list below is `--help` output from **`kicad-cli 10.0.5`**
(Homebrew cask `kicad` 10.0.5, macOS/arm64), and every behaviour claim has a
command you can re-run. Exit codes are cross-checked against
`include/cli/exit_codes.h` at source tag `10.0.5`.

---

## ⚠ Contradictions and cautions for `spec/SPEC.md`

1. **SPEC §4's exit-code table collides with `kicad-cli`'s.** SPEC assigns
   `3 = verification failure` and `4 = file/parse error`; `kicad-cli` uses
   `3 = ERR_INVALID_INPUT_FILE` and `5 = ERR_RC_VIOLATIONS` (§4.3). kicli must
   **translate**, never pass through, and the docs should say so explicitly —
   an agent that sees exit 5 from `kicli sch erc` and looks it up in kicli's
   table would be misled.

2. **`kicad-cli sch erc --format json` reports schematic coordinates 100× too
   small** (see `geometry.md` §3.5 for the root cause and proof). Any kicli
   command that surfaces ERC positions must correct for it.

3. **There is no region/zoom rendering.** `sch export svg` has no `--region`,
   `--zoom`, `--bbox`, or `--around` flag (§5.1). SPEC §10's `--region` and
   `--around R7 --radius 50mm` must be implemented by kicli as SVG
   post-processing (viewBox rewrite), which the output format supports cleanly
   (§5.3). This confirms the guess in the research task.

4. **`kicad-cli sch upgrade` loses bus aliases** (`sch-format.md` §5.6). Do not
   wire it into any kicli code path.

5. **SPEC §10 says renders are "passive output only" — the SVG is richer than
   that suggests.** It carries per-text-item `<desc>` content and
   `class="stroked-text"` groups (§5.4), which is exactly what the annotation
   overlay (R11) needs, and it carries `textLength` values computed by KiCad's
   own font engine, which resolves the font-metrics licensing problem in
   `geometry.md` §5.4 (§5.5 below).

---

## 1. Availability and installation

| Platform | Channel | Verified |
|---|---|---|
| macOS (arm64) | Homebrew cask `kicad` — `brew info --cask kicad` → 10.0.5, 5.5 GB, installs `/Applications/KiCad/KiCad.app`; `/opt/homebrew/bin/kicad-cli` is a symlink to `…/KiCad.app/Contents/MacOS/kicad-cli` | yes, this machine |
| Arch Linux | `extra/kicad` 10.0.5-1 (archlinux.org package API, fetched 2026-08-12), optional `kicad-library`, `kicad-library-3d`, `kicad-demos` | package version verified via API; **the file list endpoint was 502 at the time of writing, so "`kicad-cli` is in the main package" is inferred, not verified.** Check on the target box with `pacman -Ql kicad \| grep kicad-cli`. |

Consequences for kicli:

- `kicad-cli` is a *fat* dependency (a full KiCad install). It must be
  **optional**: parsing, geometry, lint and scoring must all work without it;
  only `sch erc`, `sch render` and netlist/BOM export need it.
- Path discovery: `$KICLI_KICAD_CLI` → `kicli.toml` `kicad_cli_path` → `PATH` →
  `/Applications/KiCad/KiCad.app/Contents/MacOS/kicad-cli`. SPEC §12 already
  reserves the config key.
- **Version check on first use.** `kicad-cli version --format plain` returns
  `10.0.5`. kicli should refuse (or warn loudly) on a major version other than
  10, because both the file format and the JSON report schema move.

### 1.1 Cold-start cost

The **first** invocation on this machine took >120 s — fontconfig cache
building, with `Fontconfig warning: … 49-sansserif.conf … invalid constant used
: math` on stderr. Subsequent runs (warm):

| Command | Input | Wall time |
|---|---|---|
| `sch erc --format json` | 8-symbol test sheet | 0.50–0.52 s |
| `sch export svg` | 8-symbol test sheet | 0.17 s |
| `sch export svg` | `CM5_MINIMA_3` (9 sheets, ~4 MB, 5.6 MB of SVG out) | 0.42 s |

So the steady-state cost is fine for an agent loop, but the first call after
install is not. kicli should warm it once (e.g. in `kicli project check`) and
say what it is doing, or the agent will think it hung.

---

## 2. Complete command surface

```
kicad-cli {fp, jobset, pcb, sch, sym, version}

sch  {erc, export, upgrade}
sch export {bom, dxf, hpgl, netlist, pdf, ps, python-bom, svg}
sym  {export, upgrade}
```

Only `sch` and `sym` matter for v1; `pcb` is superseded by the IPC API for our
purposes (R5), and `jobset` is a batch-runner over the same jobs.

---

## 3. `sch erc`

```
Usage: sch erc [--help] [--output OUTPUT_FILE] [--define-var KEY=VALUE]...
       [--format VAR] [--units VAR] [--severity-all] [--severity-error]
       [--severity-warning] [--severity-exclusions] [--exit-code-violations]
       INPUT_FILE
```

- `--format` ∈ `{report, json}`, default `report`.
- `--units` ∈ `{in, mm, mils}`, default `mm` (affects the **text** report; see
  the JSON bug below).
- Severity flags are additive; default reports errors + warnings.
  `--severity-all` = errors + warnings + exclusions.
- `--define-var KEY=VALUE` injects project text variables — useful when a sheet
  references `${VAR}` that normally lives in `.kicad_pro`.

### 3.1 JSON report shape (verified)

```json
{
  "$schema": "https://schemas.kicad.org/erc.v1.json",
  "source": "geomtest.kicad_sch",
  "date": "2026-08-12T…",
  "kicad_version": "10.0.5",
  "coordinate_units": "mm",
  "included_severities": ["error", "warning"],
  "ignored_checks": [...],
  "sheets": [ { "path": "/", "uuid_path": "…", "violations": [ {
      "type": "pin_not_connected",
      "severity": "error",
      "description": "Pin not connected",
      "excluded": false,
      "items": [ { "description": "Symbol R1 Pin 1 [Passive, Line]",
                   "uuid": "0000…000a",
                   "pos": { "x": 0.254, "y": 0.2159 } } ] } ] } ]
}
```

Good news for kicli: **each violation item carries the offending object's
UUID**, so ERC findings can be joined to kicli's own object handles without
coordinate matching.

Bad news: `pos` is 100× too small (`geometry.md` §3.5). The same run's text
report says `@(25.40 mm, 21.59 mm)` for the item the JSON reports as
`(0.254, 0.2159)`.

### 3.2 Severity mapping

ERC severities are per-check and come from the **project** file
(`.kicad_pro` → `erc.rule_severities`), not from the CLI. `kicad-cli` can only
filter which severities are *reported*, not change them. Therefore SPEC §12's
"ERC severity mapping" config in `kicli.toml` can only be a *presentation*
mapping (kicli's own severity labels) unless kicli edits `.kicad_pro`, which is
a bigger commitment. Flagged as Q2.

### 3.3 Exit codes (measured)

| Situation | Command | Exit |
|---|---|---|
| Violations found, no flag | `sch erc -o e.json --format json good.kicad_sch` | **0** |
| Violations found, with flag | `… --exit-code-violations` | **5** |
| Input file does not exist | `sch erc -o e.json /nonexistent.kicad_sch` | **3** |
| Malformed s-expression input | truncated file | **3** |
| Invalid `--format` value | `--format bogus` | **1** |

Matches `include/cli/exit_codes.h` at tag `10.0.5`:

```
OK = 0   ERR_ARGS = 1   ERR_UNKNOWN = 2
ERR_INVALID_INPUT_FILE = 3   ERR_RC_VIOLATIONS = 5   ERR_JOBS_RUN_FAILED = 6
```

(4 is unused in that header.) **kicli must map these into its own scheme**
(SPEC §4) — see caution 1.

---

## 4. `sch export netlist`

```
--format ∈ {kicadsexpr, kicadxml, cadstar, orcadpcb2, spice, spicemodel,
            pads, allegro}          default: kicadsexpr
--variant <name>   (repeatable; ${VARIANT} substitution in the output path)
```

`kicadsexpr` output begins:

```
(export
	(version "E")
	(design
		(source "…/geomtest.kicad_sch")
		(date "2026-08-12T03:22:39")
		(tool "Eeschema 10.0.5")
		(sheet (number "1") (name "/") (tstamps "/") …
```

Relevance to kicli: the netlist is the **independent oracle** for connectivity.
kicli computes nets itself (R10 connectivity view) from geometry; a test that
compares kicli's net list against `kicad-cli`'s for every fixture is the
strongest correctness check available, and it costs one subprocess call. Strongly
recommended as an M2 gate (mirrors the L5 oracle idea in `sexpr-strategy.md` §5).

Note the embedded absolute `source` path and `date` — netlist output is **not**
reproducible byte-wise; compare parsed content, not bytes.

---

## 5. `sch export svg`

```
Usage: export svg [--output OUTPUT_DIR] [--drawing-sheet SHEET_PATH]
       [--define-var KEY=VALUE]... [--variant VAR]... [--theme THEME_NAME]
       [--black-and-white] [--exclude-drawing-sheet] [--default-font VAR]
       [--draw-hop-over] [--no-background-color] [--pages PAGE_LIST] INPUT_FILE
```

Note `--output` is a **directory**, not a file.

### 5.1 What is missing

No `--region`, `--bbox`, `--zoom`, `--dpi`, `--scale`. Rendering is always the
full page at 1:1. **Region rendering is kicli's job.** Confirmed.

### 5.2 Multi-sheet behaviour (verified)

```sh
kicad-cli sch export svg -o hier -n -e complex_hierarchy.kicad_sch
# → hier/complex_hierarchy.svg
#   hier/complex_hierarchy-ampli_ht_vertical.svg
#   hier/complex_hierarchy-ampli_ht_horizontal.svg
```

Exporting the root sheet exports **every sheet in the hierarchy**, one file per
sheet instance, named `<root-stem>-<sheet name>.svg` (root sheet keeps the bare
stem). `--pages 1` restricts to page 1 and produces only
`complex_hierarchy.svg`.

Cautions: file names derive from user-controlled sheet names (collision and
sanitisation hazards), and the mapping file→sheet-path is by name, not UUID. For
`kicli sch render --sheet <path>` the robust approach is: resolve the sheet path
→ page number via `sheet_instances`, then call with `--pages N`, then take the
single file produced.

### 5.3 SVG geometry (verified)

```xml
<svg … version="1.1"
  width="297.0022mm" height="210.0072mm"
  viewBox="0.0000 0.0000 297.0022 210.0072">
```

- **User units are millimetres**, 4 decimal places, origin at the page's
  top-left, matching schematic file coordinates directly. Cropping to a region
  is therefore a pure `viewBox` + `width`/`height` rewrite with **no coordinate
  transformation**, which is the cleanest possible outcome for SPEC §10.
- Page size is the schematic's `(paper …)` plus a hair (A4 → 297.0022 ×
  210.0072).
- `-n/--no-background-color` and `-e/--exclude-drawing-sheet` shrink the output
  dramatically (76 KB → 21 KB on the test sheet) and are the right defaults for
  agent-facing renders of a region.

### 5.4 SVG element structure (verified)

Element census for a small sheet (`-n -e`): 268 `path`, 49 `g`, 17 `desc`,
16 `text`, 8 `rect`.

Each text item is emitted **twice**:

```xml
<g style="fill:none; stroke:#0000C2; stroke-width:0.1524; …">
  <text x="166.0000" y="14.7500"
        textLength="1.2918" font-size="1.6933" lengthAdjust="spacingAndGlyphs"
        text-anchor="start" opacity="0" stroke-opacity="0">A</text>
  <g class="stroked-text"><desc>A</desc>
    <path d="M166.3421 14.2237 L…" />
    …
  </g>
</g>
```

1. An **invisible** `<text>` (`opacity="0"`) carrying the string, its anchor, its
   `font-size` and a `textLength` — for text selection/search in viewers.
2. A `<g class="stroked-text"><desc>TEXT</desc>` group holding the actual stroke
   paths.

For R11 this is a gift: annotation overlays can locate every text item by
`<desc>`, and layer/colour grouping is already expressed as `<g style="stroke:…">`.

`font-size` is the text height × 4/3 (1.27 mm → 1.6933), i.e. the usual
px/pt-style factor; do not read it as millimetres.

### 5.5 Font metrics fall out for free

`textLength` is computed by KiCad's own font engine, so string extents can be
*measured* rather than derived from a GPL glyph table (`geometry.md` §5.4, Q2).

Calibration experiment (recipe R3-A), 94 single printable-ASCII glyphs plus 4
pairs, size 1.27 mm, default pen (SVG shows `stroke-width:0.1524` = 6 mil
throughout):

```
textLength("AB") − textLength("A") − textLength("B") = −0.4572 mm
textLength("MM") − 2·textLength("M")                 = −0.4572 mm
textLength("il") − textLength("i") − textLength("l")  = −0.4572 mm
textLength("W1") − textLength("W") − textLength("1")  = −0.4572 mm
```

The offset is **constant and exactly 3 × 0.1524 mm** (3 × pen width), so the
model `textLength(s) = Σ advance(c) + 3·penWidth` fits every sample, and
per-glyph advances are recoverable as `advance(c) = textLength(c) − 3·penWidth`.
Sample values at size 1.27 mm:

| glyph | textLength (mm) | advance (mm) | advance / size |
|---|---|---|---|
| `A` | 1.2918 | 0.8346 | 0.657 |
| `M` | 1.6546 | 1.1974 | 0.943 |
| `i` | 0.8080 | 0.3508 | 0.276 |
| `m` | 1.8965 | 1.4393 | 1.133 |
| `0` | 1.4127 | 0.9555 | 0.752 |

Caveat worth stating plainly: this establishes an *exact linear relation* on the
tested samples; it does **not** yet prove `textLength` equals the extents KiCad's
`GetTextBox` uses internally (`geometry.md` §5.1-5.2 has an `INTER_CHAR = 0.2`
term the additive model does not show). Before this becomes kicli's metrics
source, run the full sweep (all glyphs, several sizes, bold/italic, and multi-
character strings) and fit/verify against `GetTextBox` behaviour on a case where
a lint rule's answer actually depends on it.

### 5.6 Other flags

- `--theme <name>` selects a colour theme; `--black-and-white` is the sane
  choice for agent vision and for diffing renders.
- `--drawing-sheet` overrides the frame; `--exclude-drawing-sheet` removes it.
- `--default-font` sets the fallback for outline-font text.
- `--draw-hop-over` draws wire hop-overs (a KiCad 10 feature) — worth exposing,
  since it changes how readable a crossing-heavy sheet looks.
- `--define-var` matters when rendering a sheet whose title block uses project
  variables.

---

## 6. `sch export bom`

Fully parameterised CSV generator:

```
--fields   default "Reference,Value,Footprint,QUANTITY,DNP"
--labels   default "Refs,Value,Footprint,Qty,DNP"
--group-by, --sort-field, --sort-asc, --filter, --exclude-dnp
--field-delimiter "," --string-delimiter "\"" --ref-delimiter "," --ref-range-delimiter "-"
--keep-tabs --keep-line-breaks
--preset / --format-preset   (named presets stored in the schematic)
```

Generated pseudo-fields: `QUANTITY`, `ITEM_NUMBER`, `DNP`, `EXCLUDE_FROM_BOM`,
`EXCLUDE_FROM_BOARD`, `EXCLUDE_FROM_SIM`.

`--include-excluded-from-bom` is documented as **deprecated, no effect**.

kicli does not need BOM generation in v1 (not in SPEC §4), but `parts search`
(SPEC §8) and any "what's on this sheet" view can be sanity-checked against it.
`python-bom` (legacy XML for the old BOM scripts) is legacy; ignore.

---

## 7. `sch upgrade` and `sym upgrade`

```
sch upgrade [--force] INPUT_FILE
sym upgrade …
```

Rewrites the file through KiCad's current serialiser. Verified properties:

- **Idempotent**: `--force` twice produces byte-identical output.
- Prints `Successfully saved schematic file using the latest format`.
- Operates on **one file**, not a project: it does not update `.kicad_pro`, which
  is exactly why bus aliases are lost (`sch-format.md` §5.6).

Legitimate uses for kicli: generating canonical fixtures (with the alias caveat)
and as the L5 test oracle (`sexpr-strategy.md` §5). Not a production code path.

---

## 8. Implications for kicli's design

1. `kicad-cli` is optional and version-checked; every command that needs it must
   degrade with a clear structured error, not a panic.
2. One thin `kicad_cli` module with: locate → version check → run with captured
   stdout/stderr → map exit code → parse output. Every call site goes through it.
3. Exit-code translation table lives in that module and is documented in
   `AGENT.md` (Constitution §10).
4. ERC JSON: apply the ×100 correction, keyed on `kicad_version`, with a
   regression test that fails when upstream fixes it.
5. Render pipeline: `-n -e --black-and-white` by default → crop `viewBox` →
   optional annotate → rasterise (R11).
6. Netlist comparison as a correctness oracle in M2's test suite.
7. Warm the font cache once and tell the user; never let an agent see a
   two-minute silent hang.

---

## 9. Open questions for James

- **Q1 — Optional dependency policy.** Confirm: `kicli sch erc`/`render` fail
  with a structured "kicad-cli not found" error (exit 1) rather than kicli
  bundling or vendoring anything from KiCad.

- **Q2 — ERC severities.** They live in `.kicad_pro`. Does kicli (a) only relabel
  severities for its own output, or (b) get to edit `.kicad_pro` so
  `kicli.toml` can genuinely change ERC behaviour? (a) is the safe v1 answer.

- **Q3 — `--variant`.** All export commands take `--variant`. Should kicli
  expose it in v1 (round-trip only per `sch-format.md` Q4), or hide it?

---

## 10. Reproduction index

| § | Command |
|---|---|
| 1.1 | `/usr/bin/time -p kicad-cli sch erc -o t.json --format json geomtest.kicad_sch` |
| 3.1 | `kicad-cli sch erc --format json --severity-all -o erc.json geomtest.kicad_sch` |
| 3.3 | the five commands in the table, `echo $?` after each |
| 4 | `kicad-cli sch export netlist -o net.net geomtest.kicad_sch` |
| 5.2 | `kicad-cli sch export svg -o hier -n -e complex_hierarchy.kicad_sch` |
| 5.4 | `head -12 out/geomtest.svg`, `grep -c '<text' out/geomtest.svg` |
| 5.5 | generate one `(text "<glyph>")` item per printable ASCII char, export SVG, extract `textLength` |
| 7 | `kicad-cli sch upgrade --force f.kicad_sch` twice, `diff` the results |

Sources: `kicad-cli 10.0.5 --help` output (this machine);
`include/cli/exit_codes.h`, `eeschema/erc/erc_report.cpp` at tag `10.0.5`;
[Arch `extra/kicad` package](https://archlinux.org/packages/extra/x86_64/kicad/);
[KiCad Arch install page](https://www.kicad.org/download/details/arch-linux/).
