# R1 — `.kicad_sch` format deep-dive (KiCad 10.0.5)

Status: verified against KiCad **10.0.5** (source tag `10.0.5`, binary
`kicad-cli 10.0.5` on macOS/Homebrew). Every claim below is either (a) linked to
a source file at that tag, or (b) reproducible with the recipes in §0.

---

## 0. Method and reproduction environment

### 0.1 Tools used

| Thing | Version | How obtained |
|---|---|---|
| `kicad-cli` | 10.0.5 | `kicad-cli --version` (Homebrew cask KiCad 10.0.5, macOS) |
| KiCad source | tag `10.0.5`, commit `18fb9289` ("Tag stable version 10.0.5") | `git clone --depth 1 https://gitlab.com/kicad/code/kicad.git && git fetch --depth 1 origin tag 10.0.5 && git checkout 10.0.5` |
| KiCad 9 reference | tag `9.0.0` | `git fetch --depth 1 origin tag 9.0.0` |

Note: KiCad's `master` at time of writing is 10.99 development with schematic
file version `20260803`. **Do not read format facts off `master`** — it is ahead
of 10.0.x. All source line references below are at tag `10.0.5`.

### 0.2 Canonical-output corpus (recipe C)

`kicad-cli sch upgrade --force <file>` re-serialises a schematic through
KiCad's own writer. This is the cheapest way to obtain *exactly* what KiCad 10
writes without running the GUI:

```sh
git clone --depth 1 https://gitlab.com/kicad/code/kicad.git src/kicad
cd src/kicad && git fetch --depth 1 origin tag 10.0.5 && git checkout 10.0.5
cp -r demos ../demos_v10
cd ../demos_v10
find . -name '*.kicad_sch' -exec kicad-cli sch upgrade --force {} \;
```

This yields 115 `.kicad_sch` files at `(version 20260306)` covering every item
type in the schematic grammar except `group`, `arc`, `circle` and `bezier`.
Everything in §3 is quoted verbatim from that corpus, with file:line given.

### 0.3 v9→v10 diff (recipe D)

The `demos/complex_hierarchy` project ships at `(version 20250114)` — i.e.
*exactly* the KiCad 9.0 format. Upgrading it produces a pure v9→v10 diff:

```sh
cp -r src/kicad/demos/complex_hierarchy exp/ && cp -r exp/complex_hierarchy exp/ch_orig
cd exp/complex_hierarchy && for f in *.kicad_sch; do kicad-cli sch upgrade "$f"; done
diff -u ../ch_orig/complex_hierarchy.kicad_sch complex_hierarchy.kicad_sch
```

§5 is derived from that diff.

### 0.4 What was *not* verified

- No GUI run. Everything is `kicad-cli` + source reading. Where GUI behaviour
  might differ (notably §5.6 bus aliases) it is called out explicitly.
- `group`, `arc`, `circle`, `bezier`, `symbol_instances` (legacy), embedded
  files other than fonts: grammar taken from the parser/writer source only, no
  corpus example.

---

## ⚠ Contradictions and cautions for `spec/SPEC.md`

1. **SPEC §6 "byte-identity" is achievable but only via a re-implemented
   pretty-printer, not via "preserve the input bytes".** KiCad's writer emits a
   flat token stream and then runs a *whole-file* re-formatter
   (`KICAD_FORMAT::Prettify`, §2.1). There is therefore no whitespace to
   "preserve": any file KiCad wrote is already in canonical form, and any file
   it writes next will be too. The Constitution §1 requirement is best met by
   (i) a CST that preserves input bytes for untouched regions, and (ii) an
   emitter for touched regions that reproduces `Prettify` exactly. See R2.

2. **KiCad reorders every item in the file on save.** `SCH_IO_KICAD_SEXPR::Format`
   sorts all schematic items by `(item type enum, UUID)` before writing
   (`eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr.cpp:438-457`). So "kicli's
   output is byte-identical to the input" and "kicli's output is byte-identical
   to what KiCad would write for the same design" are *different* properties and
   the second is only obtainable by adopting KiCad's sort order. SPEC should say
   which one is the gate. Recommendation: gate on the first (input-preserving),
   test the second as an informational property.

3. **`(version …)` is a date stamp, not a semver, and 10.0.x has already moved
   twice.** SPEC §3 D1 says "target KiCad 10.0 file formats" — that needs a
   concrete floor: 10.0.5 writes `20260306`; a 10.0.0 install writes an earlier
   stamp. kicli must decide whether it refuses files newer than its known
   maximum. Recommendation: parse any version, refuse to *write* a file whose
   version stamp is greater than the maximum kicli was built against (see Q3).

4. **Coordinates are integers, not reals.** SPEC §6 talks about grid snapping in
   mm. The file's numbers are `int32` internal units of **100 nm**
   (`include/base_units.h:72`, `SCH_IU_PER_MM = 1e4`). 50 mil = 1.27 mm =
   12700 IU. All geometry in kicli should be integer IU internally; mm is a
   presentation unit only. This is a strengthening of the spec, not a conflict,
   but it needs writing down before M1.

5. **`kicad-cli sch upgrade` is lossy for bus aliases (§5.6) — do not use it as
   kicli's migration path**, and do not use it in fixtures generation without
   knowing this.

---

## 1. Top-level file shape

A `.kicad_sch` is one s-expression whose head is `kicad_sch`. One file = one
*screen* = one sheet's drawing. Hierarchy is by reference (`sheet` items name a
child file), never by nesting.

Order as written by `SCH_IO_KICAD_SEXPR::Format`
(`sch_io_kicad_sexpr.cpp:403-543`):

```
(kicad_sch
	(version 20260306)          ; SEXPR_SCHEMATIC_FILE_VERSION, eeschema/sch_file_versions.h:138
	(generator "eeschema")      ; literal
	(generator_version "10.0")  ; GetMajorMinorVersion()
	(uuid "…")                  ; this screen's KIID
	(paper "A3")                ; PAGE_INFO::Format
	(title_block …)             ; optional; omitted when empty
	(lib_symbols …)             ; always present, may be `(lib_symbols)`
	<items, sorted by (type, uuid)>
	(sheet_instances …)         ; only on a root sheet (HasRootInstance())
	(embedded_fonts no)         ; only on schematic's top-level sheet — see §5.5
	(embedded_files …)          ; only if non-empty
)
```

Verified head/tail (`exp/demos_v10/complex_hierarchy/complex_hierarchy.kicad_sch`):

```
(kicad_sch
	(version 20260306)
	(generator "eeschema")
	(generator_version "10.0")
	(uuid "5b9623a5-6d01-41fc-9865-e1bc779418c8")
	(paper "A4")
	(title_block
		(title "Complex hierarchy: demo")
		(date "2017-01-15")
	)
```

Top-level item tokens actually observed across the 115-file corpus, with counts
(`grep -h '^\t(' … | sed 's/ .*//' | sort | uniq -c`):

```
19279 (wire      6545 (symbol    5804 (label     4293 (junction   2405 (bus
 2000 (bus_entry 1484 (no_connect 795 (hierarchical_label 511 (text  260 (polyline
  214 (global_label 94 (sheet      45 (text_box   45 (image        37 (netclass_flag
   34 (rule_area   33 (rectangle    1 (table
```

Additional tokens the 10.0.5 parser accepts at top level but which the corpus
does not exercise (`sch_io_kicad_sexpr_parser.cpp:3064` — the `Expecting(...)`
string is the authoritative list): `bitmap` (legacy name for `image`),
`bus_alias` (read-only, see §5.6), `class_label`, `embedded_files`, `arc`,
`circle`, `bezier`, `group`, `symbol_instances` (legacy).

### 1.1 Item ordering is imposed by KiCad, not by the author

```cpp
// sch_io_kicad_sexpr.cpp:438
auto cmp = []( const SCH_ITEM* a, const SCH_ITEM* b )
        {
            if( a->Type() != b->Type() ) return a->Type() < b->Type();
            return a->m_Uuid < b->m_Uuid;
        };
std::multiset<SCH_ITEM*, decltype( cmp )> save_map( cmp );
```

The type order is the `KICAD_T` enum order (`include/core/typeinfo.h`), which is
why the corpus files list all `symbol`s, then `sheet`s, then `junction`s, then
`no_connect`s, etc. `SCH_MARKER_T` items (ERC markers) are explicitly *not*
saved.

Consequence for kicli: appending a new item at the end of the file is legal and
KiCad will accept it, but the file is then not in KiCad's canonical order, and
the next GUI save will move it. Both facts are fine; they just have to be stated
in the round-trip tests so a "KiCad moved my lines" diff is not read as a bug.

---

## 2. Lexical layer — exactly how bytes are produced

This section is the specification kicli's emitter must satisfy. It is the part
most tools get wrong.

### 2.1 The pretty-printer

The writers call `OUTPUTFORMATTER::Print` with no layout at all (e.g.
`m_out->Print( "(kicad_sch (version %d) (generator \"eeschema\") …" )`). Layout
is applied afterwards, to the whole buffer, by
`KICAD_FORMAT::Prettify( std::string&, FORMAT_MODE )`
(`common/io/kicad/kicad_io_utils.cpp:97-339`), invoked from
`PRETTIFIED_FILE_OUTPUTFORMATTER` (`common/richio.cpp:601`).

Documented rules (`include/io/kicad/kicad_io_utils.h:73-93`, verbatim):

> - All extra (non-indentation) whitespace is trimmed
> - Indentation is one tab
> - Starting a new list (open paren) starts a new line with one deeper indentation
> - Lists with no inner lists go on a single line
> - End of multi-line lists (close paren) goes on a single line at same indentation as its start

The implementation adds four things the doc comment does not mention, all of
which kicli must reproduce:

| Rule | Constant | Effect |
|---|---|---|
| `(xy …)` run-packing | `xySpecialCaseColumnLimit = 99` | consecutive `(xy …)` lists stay on one line until column 99 — this is why `polyline`/`pts` blocks wrap raggedly |
| long token wrap | `consecutiveTokenWrapThreshold = 72` | inside a list, once column ≥ 72, the *next* space becomes a newline + indent (this is what wraps base64 `data` and long token runs) |
| short-form lists | `FORMAT_MODE::COMPACT_TEXT_PROPERTIES` only, tokens `font stroke fill teardrop offset rotate scale` | keeps those on one line |
| library-table rows | `FORMAT_MODE::LIBRARY_TABLE` only, token `lib` | one row per line |

**Schematics use `FORMAT_MODE::NORMAL`.** Proof: in corpus output `(effects (font (size 1.27 1.27)))` is written expanded over three lines —

```
		(effects
			(font
				(size 1.27 1.27)
			)
		)
```

— which is what `NORMAL` produces; `COMPACT_TEXT_PROPERTIES` would collapse the
`font` list.

A file always ends with exactly one `\n` (`kicad_io_utils.cpp:336`, comment
"newline required at end of line / file for POSIX compliance").

Quote handling inside the prettifier tracks `inQuote` and counts backslashes so
that `\\"` is not mistaken for an escaped quote (`kicad_io_utils.cpp:314-323`).
kicli's emitter must do the same.

### 2.2 String quoting and escaping

`OUTPUTFORMATTER::Quotes` (`common/richio.cpp:468-504`) escapes exactly four
characters and nothing else:

| char | emitted as |
|---|---|
| `\n` | `\n` |
| `\r` | `\r` |
| `\` | `\\` |
| `"` | `\"` |

Everything else, including UTF-8, tabs, and `(`/`)` inside strings, is written
raw (`Quotew` converts `wxString`→UTF-8 first, `richio.cpp:507-515`). There is
**no `\t`, `\uXXXX`, or octal escape** in this format; a literal tab inside a
string is a literal tab byte. See the `text_box` example in §3.7 for real
`\"…\"` and `\n` escapes in the wild.

`GetQuoteChar` (`richio.cpp:344-375`) decides whether a bare symbol needs
quoting (`#` first char, empty string, or containing any of `\t`, space, `(`,
`)`, `%`, `{`, `}`, or a non-initial `-`). In practice the schematic writers
call `Quotew` unconditionally for user strings, so *every* string field in a
`.kicad_sch` is quoted, and every enum-ish token (`yes`, `no`, `input`, `solid`,
`left`, …) is bare. kicli should follow the same simple rule: user data quoted
always, keyword tokens bare always.

### 2.3 Numbers

Internal unit: **1 IU = 100 nm** for schematics.

```cpp
// include/base_units.h:72
constexpr double SCH_IU_PER_MM = 1e4;  ///< Schematic internal units 1=100nm.
```

Formatting (`common/eda_units.cpp:194-225`):

```cpp
engUnits = aValue / 1e4;                       // int IU → mm
if( engUnits != 0.0 && fabs( engUnits ) <= 0.0001 )
    buf = fmt::format( "{:.10f}", engUnits );  // then strip trailing zeros, then a trailing '.'
else
    buf = fmt::format( "{:.10g}", engUnits );
```

Angles (`eda_units.cpp:186-191`): `fmt::format("{:.10g}", degrees)`.

Consequences:

- Every coordinate in a well-formed file is an exact multiple of 0.0001 mm and
  round-trips through `int32` losslessly. `41.91` mm = 419100 IU.
- `{:.10g}` is *not* `printf("%g")`: it is fmt's shortest-round-trip-ish `g`
  with 10 significant digits, so `0` prints as `0`, `1.27` as `1.27`,
  `179.9944` as `179.9944`. Rust's `format!("{}", f64)` does **not** match in
  general; kicli needs a small formatter that mimics `{:.10g}` (drop trailing
  zeros, no `+` on exponent, switch to exponent form on the same thresholds).
  Because all schematic values are `IU/1e4` with |IU| < 2^31, the exponent case
  can only arise for the `<= 0.0001` branch, which is handled separately. A
  practical implementation: format the integer IU as a fixed-point decimal with
  4 fractional digits and strip trailing zeros and a trailing `.`. That is
  provably identical to KiCad's output for all `int32` inputs and avoids float
  formatting entirely. **Recommended for kicli.**
- Non-coordinate reals do exist and use other paths: `(color r g b a)` alpha is
  a plain double, `(scale …)` uses `fmt::format("{:g}")`
  (`sch_io_kicad_sexpr.cpp:1064`). These need their own formatters; see §5.4 for
  the v9→v10 alpha formatting change.

### 2.4 Booleans

KiCad 10 writes `(token yes)` / `(token no)` via `KICAD_FORMAT::FormatBool`.
Bare-token booleans (`(hide)`, `(fields_autoplaced)`) were the KiCad 7 style;
the 10.0.5 parser still accepts them for old files, but 10.0.5 never writes
them. kicli should accept both on read and always write the `yes`/`no` form.

---

## 3. Item grammars (verbatim from the v10 corpus)

All examples are real bytes from recipe-C output; tabs shown as-is.

### 3.1 Wire / bus

```
	(wire
		(pts
			(xy 251.46 139.7) (xy 267.97 139.7)
		)
		(stroke
			(width 0)
			(type solid)
		)
		(uuid "00bffc77-589f-45f5-9deb-265f58dadb83")
	)
```
`CM5.kicad_sch:5643`. `bus` is identical except the head token and
`(type default)` in the observed sample (`CM5.kicad_sch:5763`). A wire is
always exactly two points — KiCad splits polylines into segments; a *graphic*
polyline uses the `polyline` token instead (§3.8) and carries no connectivity.

`(width 0)` means "use the netclass/default width"; it is not a zero-width line.

### 3.2 Junction / no-connect / bus entry

```
	(junction
		(at 331.47 119.38)
		(diameter 1.016)
		(color 0 0 0 0)
		(uuid "03031cc9-a48c-4c04-99fa-13b3bd865cf7")
	)
	(no_connect
		(at 161.29 162.56)
		(uuid "03c2e5dd-7fb1-48fe-917d-e132c4bc099b")
	)
	(bus_entry
		(at 55.88 86.36)
		(size 2.54 2.54)
		(stroke (width 0) (type default))
		(uuid "0104cc5a-dc6a-465c-9d8b-1f657d1a9fe2")
	)
```
`(diameter 0)` = use default; `(color 0 0 0 0)` = use theme colour.

### 3.3 Labels

Four distinct tokens, three shapes of record:

```
	(label "GPIO12"
		(at 157.48 116.84 180)
		(effects (font (size 1.27 1.27)) (justify right bottom))
		(uuid "026d94a5-3d01-4fc0-a2f2-3823903f66fa")
	)

	(hierarchical_label "MOSI_GPIO20"
		(shape input)
		(at 163.83 111.76 0)
		(effects (font (size 1.27 1.27)) (justify left))
		(uuid "048e19fd-b11f-47d8-a92a-2fdbb97dca6e")
	)

	(global_label "VOUT"
		(shape output)
		(at 237.49 53.34 0)
		(fields_autoplaced yes)
		(effects (font (size 1.524 1.524)) (justify left))
		(uuid "e5aefc98-2a36-4b47-b1b3-6340fc8f1986")
		(property "Intersheetrefs" "${INTERSHEET_REFS}"
			(at 246.7512 53.34 0)
			(hide yes) (show_name no) (do_not_autoplace no)
			(effects (font (size 1.27 1.27)) (justify left))
		)
	)

	(netclass_flag ""
		(length 2.54)
		(shape round)
		(at 337.82 30.48 0)
		(effects (font (size 1.27 1.27)) (justify left bottom))
		(uuid "59218fad-a9e3-4093-be73-61cbb8037569")
		(property "Netclass" "90Ohm-diff_CSI" …)
		(property "Component Class" "" …)
	)
```
(`CM5.kicad_sch:8763`, `CM5.kicad_sch:9793`, `amplifier-ac.kicad_sch:2128`,
`csi.kicad_sch:14493` — reindented here for width; the corpus has each
subordinate list on its own line per §2.1.)

Key points for kicli:

- **Labels carry `property` children.** A global label owns its
  `Intersheetrefs` field; a netclass flag owns `Netclass` and
  `Component Class`. These are `SCH_FIELD`s with their own `at`/`effects`, i.e.
  they are movable text and fall squarely under Constitution §2.
- Label rotation lives in the third element of `(at x y rot)` and takes 0/90/
  180/270 only.
- `shape` ∈ `input output bidirectional tri_state passive` for hierarchical and
  global labels; `netclass_flag` uses `round`/`rectangle`/`dot`/`diamond`.
- A `label` (local) has no `shape`.

### 3.4 Symbol instance

```
	(symbol
		(lib_id "complex_hierarchy:POT")
		(at 148.59 162.56 0)
		(unit 1)
		(body_style 1)
		(exclude_from_sim no)
		(in_bom yes)
		(on_board yes)
		(in_pos_files yes)
		(dnp no)
		(uuid "00000000-0000-0000-0000-00004b3a1357")
		(property "Reference" "RV201"
			(at 148.59 165.1 0)
			(show_name no)
			(do_not_autoplace no)
			(effects (font (size 1.27 1.27)))
		)
		(property "Value" "4,7K" …)
		(property "Footprint" "Potentiometer_THT:…" …)
		(property "Datasheet" "" (at …) (hide yes) (show_name no) (do_not_autoplace no) …)
		(property "Description" "" … )
		(pin "1" (uuid "0096e1bb-61bf-49dc-a2ee-7b2453e86da5"))
		(pin "2" (uuid "c96031df-bb02-4c4c-9e2e-a86ad67da51b"))
		(pin "3" (uuid "89a48842-7976-4172-8dd0-d7a3754c94fb"))
		(instances
			(project "complex_hierarchy"
				(path "/5b9623a5-6d01-41fc-9865-e1bc779418c8/00000000-0000-0000-0000-00004b3a1333"
					(reference "RV201")
					(unit 1)
				)
				(path "/5b9623a5-6d01-41fc-9865-e1bc779418c8/00000000-0000-0000-0000-00004b3a13a4"
					(reference "RV301")
					(unit 1)
				)
			)
		)
	)
```
`exp/r1/complex_hierarchy/ampli_ht.kicad_sch:2703` — a genuinely
multiply-instantiated symbol (the same sheet is placed twice), which is the
case most third-party tools get wrong.

Optional/positional details from `saveSymbol` (`sch_io_kicad_sexpr.cpp:709-998`):

- `(lib_name "GND_1")` appears *before* `lib_id` when the embedded library
  symbol's key differs from the `lib_id` — i.e. when the symbol was edited in
  place and the cache entry was uniquified. Present in the corpus
  (`grep '(lib_name' → (lib_name "GND_1")`). Any tool that keys embedded
  symbols by `lib_id` alone will mis-resolve these.
- `(mirror x)` or `(mirror y)` — never both, never `(mirror none)`. Corpus:
  317 × `(mirror x)`, 575 × `(mirror y)`. Combined with `(at … rot)` this gives
  the 8 orientations. Geometry consequences in R7.
- `(unit N)` = which unit of a multi-unit part; `(body_style N)` = 1 normal,
  2 De Morgan (added in v10, §5.2).
- `(pin "N" (uuid …))` gives each *instance* pin a UUID; used as the
  addressable handle for connectivity. `(pin "N" (uuid …) (alternate "SCL"))`
  when an alternate pin function is selected — corpus has
  `(alternate "SCL" input clock)` forms inside `lib_symbols` and the
  instance-side `(alternate "…")` form per `sch_io_kicad_sexpr.cpp:840-842`.
- `instances` is *sorted by KIID_PATH* on write (`sch_io_kicad_sexpr.cpp:929`).
- Fields (`property`) are written by `saveField` (`:1012-1035`) in the order
  `(property NAME VALUE (at x y rot) [hide] [show_name] [do_not_autoplace] (effects …))`.
  **Note the order differs from `lib_symbols` properties**, where 10.0.5 writes
  `(at) (show_name) (do_not_autoplace) (hide)`. Verified in the v9→v10 diff
  (recipe D): instance fields get `hide` first, library fields get it last. Any
  emitter aiming at byte-identity must keep two orderings.

### 3.5 Sheet and sheet pins

```
	(sheet
		(at 77.47 130.175)
		(size 25.4 12.065)
		(exclude_from_sim no) (in_bom yes) (on_board yes) (dnp no)
		(fields_autoplaced yes)
		(stroke (width 0.1524) (type solid))
		(fill (color 0 0 0 0))
		(uuid "23331233-7b48-43e1-9e62-b93026b1cf98")
		(property "Sheetname" "USB"
			(at 77.47 129.4634 0) (show_name no) (do_not_autoplace no)
			(effects (font (size 1.27 1.27) (thickness 0.254) (bold yes)) (justify left bottom))
		)
		(property "Sheetfile" "USB.kicad_sch"
			(at 77.47 142.8246 0) (hide yes) (show_name no) (do_not_autoplace no)
			(effects (font (size 1.27 1.27)) (justify left top))
		)
		(pin "VBUS_EN" input
			(at 102.87 134.62 0)
			(uuid "1669aa0d-bb0c-4b20-afc7-68806433e4b2")
			(effects (font (size 1.27 1.27)) …)
		)
		…
		(instances
			(project "CM5_MINIMA_3"
				(path "/23331233-…" (page "5"))
			)
		)
	)
```
`CM5_MINIMA_3.kicad_sch:7931`.

- `Sheetname` and `Sheetfile` are ordinary `property` records — renaming a sheet
  is a field edit, and moving the sheet *name text* is a field move.
- Sheet pin direction is a **positional** token after the name
  (`(pin "VBUS_EN" input …)`), unlike hierarchical labels which use
  `(shape input)`. Easy trap.
- A sheet pin's name must match a `hierarchical_label` in the child sheet; kicli
  must maintain both sides (see R7/M8).

### 3.6 Sheet instances, page numbers, and path semantics

Root sheet file ends with:

```
	(sheet_instances
		(path "/"
			(page "1")
		)
	)
```

Each `sheet` item carries its own `(instances (project … (path P (page N))))`
where **P is the path of the parent sheet path — it does not include the
sheet's own UUID**. Verified: in `complex_hierarchy.kicad_sch` the two sheet
items have UUIDs `…4b3a1333` and `…4b3a13a4`, both recording
`(path "/5b9623a5-6d01-41fc-9865-e1bc779418c8" (page "2"|"3"))`, where
`5b9623a5-…` is the root screen's own `(uuid …)`. The code writes
`sheetInstances[i].m_Path.AsString()` directly (`sch_io_kicad_sexpr.cpp:1206`)
and prunes instances whose path no longer resolves in the current hierarchy
(`:1178-1189`).

Symbol instance paths *do* include the chain of sheet-item UUIDs:
`/<root screen uuid>/<sheet item uuid>[/<sheet item uuid>…]`. See §3.4, where
the same symbol has two paths differing in the final sheet UUID and carrying
different references (`RV201`, `RV301`).

So the addressing model kicli needs (SPEC D13):

```
KIID_PATH = "/" + rootScreenUuid + ("/" + sheetItemUuid)*
handle    = (sheetPath, itemUuid)
```

and "reference designator" is a *property of the (symbol, sheet-path) pair*, not
of the symbol item. `(property "Reference" …)` on the item is the cached value
for the currently-loaded sheet path; the truth is in `instances`. A tool that
edits only the `property` will silently disagree with KiCad. **This is a
first-order requirement for kicli's `sym set-field` on `Reference`.**

### 3.7 Text and text boxes

```
	(text "CM mechanicals"
		(exclude_from_sim no)
		(at 320.802 250.19 0)
		(effects (font (size 2.54 2.54) (thickness 0.508) (bold yes)) (justify left bottom))
		(uuid "71a9d252-d89b-4482-bb33-edc8b45acf1c")
	)

	(text_box "IBIS models are either \"drivers\" or \"devices\".\n\nDrivers simulate…"
		(exclude_from_sim no)
		(at 203.2 15.24 0)
		(size 78.74 30.48)
		(margins 0.9525 0.9525 0.9525 0.9525)
		(stroke (width 0) (type default))
		(fill (type none))
		(effects (font (size 1.27 1.27)) (justify left top))
		(uuid "b769550e-d874-40c2-95fd-90c0b1912599")
	)
```
`CM5.kicad_sch:4588`, `ibis.kicad_sch:1288`. The `text_box` is the canonical
proof of §2.2 escaping: embedded `\"` and `\n` in a single-line token.

### 3.8 Graphics

```
	(rectangle
		(start 209.55 139.7)
		(end 179.07 165.1)
		(stroke (width 0) (type dash))
		(fill (type none))
		(uuid "8e4b05de-9845-4018-99ed-ae6e4ce40344")
	)
	(polyline
		(pts (xy 237.7016 179.0314) (xy 237.7079 179.0318) … )
		…
	)
	(rule_area
		(exclude_from_sim no) (in_bom yes) (on_board yes) (dnp no)
		(polyline (pts …) (stroke …) (fill …) (uuid …))
	)
```
`expansion_connector.kicad_sch:13386`, `CM5_MINIMA_3.kicad_sch:2458`,
`csi.kicad_sch:14455`. Note `rule_area` wraps a `polyline` child that has its
own UUID — a nested-item case kicli's addressing must handle.

`(xy …)` runs are packed to column 99 by the prettifier (§2.1); this is why
polyline blocks look ragged and why naive re-emission produces enormous diffs.

### 3.9 Images

```
	(image
		(at 280.67 267.97)
		(uuid "8f1071be-…")
		(data "iVBORw0KGgoAAAANSUhEUg…"     ; 76-char base64 chunks, one per line
			"nOy9d5RdR5UuvndVnXxz886…"
			…)
	)
```
`interf_u.kicad_sch:13657`. Chunk width is `MIME_BASE64_LENGTH = 76`
(`kicad_io_utils.cpp:63`). Optional `(scale …)` formatted with `{:g}`.

### 3.10 Tables

```
	(table
		(column_count 4)
		(border (external yes) (header yes) (stroke …))
		(separators (rows yes) (cols yes) (stroke …))
		(column_widths 22.86 22.86 22.86 22.86)
		(row_heights 5.08 …)
		(uuid "e3e69cb4-…")
		(cells
			(table_cell "Sequence" (exclude_from_sim no) (at 19.685 169.545 0)
				(size 22.86 5.08) (margins …) (span 1 1) (fill (type none))
				(effects …) (uuid "eddbedd7-…"))
			…
		)
	)
```
`power.kicad_sch:11309`. Tables are v8+ and each cell is a text object with its
own UUID — in scope for Constitution §2 "anything a human can move".

---

## 4. `lib_symbols` — the embedded cache

Every `.kicad_sch` embeds a full copy of each library symbol it uses, written by
`SCH_IO_KICAD_SEXPR_LIB_CACHE::SaveSymbol` (`sch_io_kicad_sexpr_lib_cache.cpp`),
i.e. **the same serialiser as `.kicad_sym`**. That is a large win for kicli: one
symbol-body parser serves both file types.

Shape (v10, from corpus):

```
	(lib_symbols
		(symbol "complex_hierarchy:+12V"
			(power global)
			(pin_names (offset 0))
			(exclude_from_sim no)
			(in_bom yes)
			(on_board yes)
			(in_pos_files yes)
			(duplicate_pin_numbers_are_jumpers no)
			(property "Reference" "#PWR" (at 0 -3.81 0) (show_name no) (do_not_autoplace no) (hide yes)
				(effects (font (size 1.27 1.27))))
			…
			(symbol "+12V_0_1"    ; child = one (unit, body_style) combination
				(polyline …)
				…
			)
			(symbol "+12V_1_1"
				(pin power_in line (at 0 0 90) (length 0) (name "" (effects …)) (number "1" (effects …)))
			)
			(embedded_fonts no)
		)
	)
```

- The child symbol name encodes `NAME_<unit>_<bodyStyle>`; unit 0 = "common to
  all units", body style 0 = "common to both styles".
- Library-symbol keys inside `lib_symbols` are `"LIBNICK:NAME"` — the full
  `lib_id`, except where `lib_name` (§3.4) redirects.
- `(embedded_fonts no)` appears **per library symbol** as well as per schematic.
- Property records here use the `(at) (show_name) (do_not_autoplace) (hide)`
  order (contrast §3.4).

Round-trip caution: the embedded copy and the on-disk library can drift. Which
one wins is a library-resolution question — see R4. For R1 the fact that matters
is: **the embedded copy is what KiCad draws and what ERC uses**; the external
library is only consulted on explicit update.

---

## 5. Differences from KiCad 9 (empirical)

Version stamps:

| | KiCad 9.0.0 | KiCad 10.0.5 |
|---|---|---|
| `SEXPR_SCHEMATIC_FILE_VERSION` | `20250114` | `20260306` |
| `SEXPR_SYMBOL_LIB_FILE_VERSION` | `20241209` | `20251024` |

(`eeschema/sch_file_versions.h` at each tag.) The comment history in that header
is the authoritative changelog; the 22 stamps between the two are listed there.
Below are the ones that *visibly change bytes*, all confirmed by recipe D.

### 5.1 Field/property formatting moved out of `effects`

```diff
 (property "Reference" "#PWR"
 	(at 0 -3.81 0)
+	(show_name no)
+	(do_not_autoplace no)
+	(hide yes)
 	(effects
 		(font
 			(size 1.27 1.27)
 		)
-		(hide yes)
 	)
 )
```
(`20251028` "Updated properties formatting (do_not_autoplace, show_name)".)
`hide` was previously an `effects` child; it is now a property child, and
`show_name`/`do_not_autoplace` are always written explicitly. This is the
single largest source of v9→v10 diff noise: 139 added `(show_name no)` +
139 `(do_not_autoplace no)` + 96 relocated `(hide yes)` in one 3.7k-line demo
sheet.

### 5.2 New tokens written unconditionally in v10

| token | where | note |
|---|---|---|
| `(body_style 1)` | every `symbol` instance | 27 occurrences added in the demo; v9 omitted it when 1 |
| `(in_pos_files yes)` | `symbol` instances and `lib_symbols` symbols | new attribute |
| `(duplicate_pin_numbers_are_jumpers no)` | `lib_symbols` symbols | jumper-pin-group feature (`20250324`) |
| `(power global)` | `lib_symbols` power symbols | was bare `(power)`; v10 adds `global`/`local` (`20250227` local power symbols) |

### 5.3 `~` no longer means "empty"

```diff
-(name "~"
+(name ""
```
(`20250318`.) In v9 a pin name/number of `~` meant "no text". In v10 the empty
string means that and `~` is a literal tilde. **Migration hazard**: a v9 file
read by a v10-unaware tool, or vice versa, silently changes pin names. kicli
must key this off the file's `(version …)`.

### 5.4 Float formatting of colour alpha

```diff
-(color 0 0 0 0.0000)
+(color 0 0 0 0)
```
Same value, different bytes. Evidence that "semantic identity verified by
re-parse" (Constitution §1) is the right floor for *foreign* files, and that
kicli should never rewrite tokens it did not modify.

### 5.5 `embedded_fonts` placement

The v9 root sheet ended `(sheet_instances …) (embedded_fonts no))`. After
upgrading each file individually with `kicad-cli`, `(embedded_fonts no)`
disappeared from the root file. Cause (`sch_io_kicad_sexpr.cpp:527-537`): v10
writes embedded-font/file data only for `m_schematic->GetTopLevelSheet( 0 )`,
because "embedded fonts and files belong to the schematic, not to any individual
sheet". When `kicad-cli sch upgrade` loads a single sheet in isolation, that
test can fail for the file being written.

Implication: `embedded_fonts` presence is *not* a per-file invariant and kicli
must not treat its absence as corruption. Open question Q1.

### 5.6 Bus aliases moved to the project file — and `kicad-cli` loses them

v9 stored bus aliases in the schematic:

```
	(bus_alias "DPHY"
		(members "D0_N" "D0_P" "D1_N" … )
	)
```
(`demos/cm5_minima/CM5.kicad_sch`, v9 original.)

v10 stores them in `.kicad_pro` (see `demos/pic_programmer/pic_programmer.kicad_pro`,
which ships with a `bus_alias` entry and is *not* modified by an upgrade). The
10.0.5 parser still reads `bus_alias` from `.kicad_sch` for back-compat
(`sch_io_kicad_sexpr_parser.cpp:3018`, `:5087`) but the writer never emits it.

**Verified data loss** (recipe C on `demos/cm5_minima`):

```
before: CM5.kicad_sch contains 1 bus_alias ("DPHY", 10 members)
after : CM5.kicad_sch contains 0 bus_alias
        CM5_MINIMA_3.kicad_pro byte-identical to before (diff empty)
```

So the alias is simply gone. This is a `kicad-cli sch upgrade` limitation (it
does not save the project file); the GUI presumably persists them, but that was
not tested here. Either way:

- kicli must treat bus aliases as **project-file** data in v10.
- kicli must never shell out to `kicad-cli sch upgrade` as part of a mutation.
- Any fixture generated by recipe C from a v9 source must be checked for
  aliases first.

### 5.7 Other v10-era format additions (from the version log, not yet exercised)

`20240101` tables · `20240417` rule areas · `20250222` hatched fills ·
`20250425` table UUIDs · `20250513` groups can carry a design-block `lib_id` ·
`20250610` DNP flags on rule areas · `20250827` custom body styles ·
`20250829` rounded rectangles · `20250901` stacked-pin notation ·
`20250922` schematic variants · `20251012` flat schematic hierarchy ·
`20260101` PCB variants · `20260306` variant `in_bom` semantics.

Two of these deserve early attention because they change *semantics*, not just
tokens:

- **Schematic variants** (`20250922`, `20260306`): `instances → path` may carry
  `(variant (name …) (field (name …) (value …)) …)`
  (`sch_io_kicad_sexpr.cpp:962-985`). A symbol's effective fields therefore
  depend on (sheet path, variant). kicli v1 can ignore variants for editing but
  **must round-trip them**, and `sch view` must not report variant-overridden
  values as if they were the only values.
- **Flat schematic hierarchy** (`20251012`): a design may have multiple
  top-level sheets (`GetTopLevelSheets()`), so "the root sheet" is not always
  unique. Affects §3.6 path handling and SPEC §4's `--sheet` addressing.

---

## 6. Implications for kicli's design

1. **CST, not AST** (confirms SPEC M1). The only way to satisfy Constitution §1
   for files kicli did not author is to keep the original bytes for untouched
   subtrees. §2 gives the complete rules to *re-emit* touched subtrees in
   KiCad's own style, so a hybrid emitter is feasible and gives byte-identity in
   the common case.
2. **Integer IU everywhere.** Parse `41.91` → `419100` IU (`int32`), emit by
   fixed-point. Never round-trip through `f64`. Grid snap = round to nearest
   12700 IU by default.
3. **Reference designators live in `instances`.** Every field mutation on
   `Reference` must be sheet-path aware, and `sym set-field Reference` must
   update both the cached `property` and the matching `instances → path`.
4. **Fields are first-class objects everywhere**, not just on symbols: sheets
   (`Sheetname`, `Sheetfile`), global labels (`Intersheetrefs`), netclass flags
   (`Netclass`, `Component Class`). `kicli field move` must accept all of them —
   this is exactly the parity gap named in SPEC D3.
5. **Two property field orderings** (instance vs library) must be preserved by
   the emitter.
6. **Version gating.** `~`-as-empty (§5.3) and `hide` placement (§5.1) both
   depend on the file's version stamp. The parser needs the version *before* it
   interprets those tokens — trivially satisfied since `(version …)` is the
   first child.
7. **Never call `kicad-cli sch upgrade`** in any code path that touches user
   files (§5.6).
8. **Item sort order** (§1.1) should be available as an explicit
   `kicli sch normalize` operation, not applied silently.

---

## 7. Open questions for James

- **Q1 — `embedded_fonts` / `embedded_files`.** These belong to the *schematic*
  but are written into one sheet file. If kicli edits that sheet, it must
  preserve them verbatim (they can be megabytes of base64). Confirm: v1 treats
  embedded files as opaque, never re-encodes, and refuses any operation that
  would move them between files?

- **Q2 — Which round-trip property is the merge gate?** (a) `parse(write(f)) ==
  parse(f)` semantically, (b) `write(parse(f)) == f` bytewise for
  KiCad-authored files, or (c) `write(parse(f)) == what KiCad would write`
  (requires adopting the §1.1 item sort). I recommend gating on (a)+(b) and
  tracking (c) as informational. Confirm.

- **Q3 — Version ceiling policy.** 10.0.5 writes `20260306`; 10.99 already
  writes `20260803`. Should kicli refuse to *write* files whose version stamp
  exceeds its built-against maximum (safe, but breaks on every KiCad point
  release), or write back the version it read (risks silently dropping tokens it
  did not understand)? Recommendation: refuse to write unknown-newer files, and
  make the ceiling a config knob.

- **Q4 — Variants.** Confirm v1 scope: round-trip only, no variant-aware
  editing, and `sch view` reports the default variant with a flag noting others
  exist.

- **Q5 — Flat (multi-top-level) hierarchies** (`20251012`). Confirm v1 supports
  reading them but `--sheet` addressing assumes a single root, erroring clearly
  otherwise.

---

## 8. Source index

All paths relative to the KiCad tree at tag `10.0.5`.

| Topic | File |
|---|---|
| Version stamps | `eeschema/sch_file_versions.h` |
| Schematic writer | `eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr.cpp` |
| Schematic parser | `eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp` |
| Library-symbol writer (shared with `.kicad_sym`) | `eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_lib_cache.cpp` |
| Pretty-printer | `common/io/kicad/kicad_io_utils.cpp`, `include/io/kicad/kicad_io_utils.h` |
| Quoting/escaping, output formatter | `common/richio.cpp` |
| Number formatting | `common/eda_units.cpp` |
| Internal unit scale | `include/base_units.h` |

Official (non-source) documentation for the s-expression schematic format lives
at <https://dev-docs.kicad.org/en/file-formats/sexpr-schematic/>. It declares
itself as covering "the s-expression schematic file format for all versions of
KiCad from 6.0", and it lags the code badly: fetched 2026-08-12, it documents
none of `body_style`, `in_pos_files`, `show_name`, `do_not_autoplace`, or
`variant` — i.e. none of the v10 changes in §5.
**Where it disagrees with the tag-`10.0.5` sources or with recipe-C output, the
sources and the corpus win.**
