# Research dossier — summary

Phase 1 complete: eleven research documents, all facts verified against KiCad
**10.0.5** (binary `kicad-cli 10.0.5` on macOS + source at tag `10.0.5`), with
source links or reproduction recipes throughout.

| Doc | Task | Headline |
|---|---|---|
| [`sch-format.md`](sch-format.md) | R1 | complete v10 grammar; 22 format changes since KiCad 9; the writer's exact byte rules |
| [`geometry.md`](geometry.md) | R7 | pin maths **verified 16/16 against KiCad itself**; text-box algorithm; a KiCad unit bug |
| [`sexpr-strategy.md`](sexpr-strategy.md) | R2 | bespoke parser confirmed; **layout is a pure function of the token stream** (115/115) |
| [`kicad-cli.md`](kicad-cli.md) | R3 | every flag, measured exit codes, SVG internals, cold-start cost |
| [`style-rules.md`](style-rules.md) | R8 | 24 rules with implementable detection maths; ERC does 47 checks we must not duplicate |
| [`representation.md`](representation.md) | R10 | three views, **measured**: 50–113× compression, net extraction validated against `kicad-cli` |
| [`wire-routing.md`](wire-routing.md) | R9 | shapes-first + A*; cost weights grounded in measured wire statistics |
| [`rendering.md`](rendering.md) | R11 | crop + overlay + raster **built and verified end-to-end**; rasteriser licence clean |
| [`libraries-and-vendoring.md`](libraries-and-vendoring.md) | R4 | resolution chain; 7-step vendor checklist; your existing Eurorack layout documented |
| [`ecosystem.md`](ecosystem.md) | R6 | `kiutils` silently loses 14.7 % of a v10 file; `kicad-skip` does not; Konnect black-box only |
| [`ipc-api.md`](ipc-api.md) | R5 | transport and command inventory; **no schematic API exists**; a licensing decision to make |

---

## The ten most design-consequential findings

### 1. Byte-identical round-trip is achievable without a whitespace CST

KiCad emits a flat token stream and then runs `KICAD_FORMAT::Prettify` over the
whole file. A port of that function reproduces **115/115** canonical demo
schematics, 6/6 `.kicad_pcb` files, and a shipped `.kicad_sym` exactly from a
whitespace-stripped token stream. So kicli needs a *token-preserving* tree plus a
faithful prettifier — not a `rowan`-style CST. Simpler, smaller, and it makes
"never rewrite a token we didn't modify" a structural guarantee.
→ `sexpr-strategy.md` §2

### 2. Coordinates are `int32` internal units of 100 nm — never floats

`SCH_IU_PER_MM = 1e4`. Every coordinate in a well-formed file is an exact
multiple of 0.0001 mm. Formatting the integer as fixed-point and stripping
trailing zeros provably matches KiCad's `{:.10g}` output for all `int32` inputs,
so kicli can avoid float formatting entirely. 50 mil = 12700 IU.
→ `sch-format.md` §2.3

### 3. Reference designators live in `instances`, not in the `property`

`(property "Reference" …)` on a symbol is a cache for the currently-loaded sheet
path; the truth is `instances → project → path → reference`. A symbol placed on a
sheet instantiated twice has two references. Any tool that edits only the
property silently disagrees with KiCad. This makes sheet-path awareness a
first-order requirement of `sym set-field`, not an M8 refinement.
→ `sch-format.md` §3.4, §3.6

### 4. Pin geometry is now proven, not assumed

`abs_pin = symbol.at + M · (lib_pin.x, −lib_pin.y)`, with the 8-element
orientation group tabulated. All 16 pin positions of an 8-orientation test sheet
match KiCad's own ERC output exactly. That table is ready to become the M2
fixture.
→ `geometry.md` §2–§3

### 5. KiCad's ERC JSON reports schematic coordinates 100× too small

`erc_report.cpp:161` builds the units provider with `pcbIUScale` (1e6 IU/mm)
instead of `schIUScale` (1e4). The text report is correct; the JSON is not, and
it labels the values `"mm"`. Still unfixed on `master`. Any tool consuming
`kicad-cli sch erc --format json` is wrong today.
→ `geometry.md` §3.5

### 6. ERC already implements 47 checks — the lint engine must layer, not duplicate

Including `four_way_junction`, `endpoint_off_grid`, `similar_labels`,
`label_dangling`. Two of them default to `IGNORE`. kicli's score should consume
ERC and add only what ERC structurally cannot see: where things are *drawn*.
That also means `sch score --gate` implicitly needs `kicad-cli`.
→ `style-rules.md` §2

### 7. The incumbent Python libraries prove the thesis — one badly, one well

Round-tripping one KiCad 10.0.5 sheet: **`kiutils` 1.4.8 loses 2,912 of 19,851
tokens (14.7 %)** — 166 `hide` flags, 295 `do_not_autoplace`, 46 `body_style`,
59 `exclude_from_sim` — and KiCad still opens the result, so the loss is silent.
**`kicad-skip` 0.2.5 loses nothing** (19,851 → 19,851), differing only in
whitespace. Keep both as regression fixtures; the claim "kicli is byte-identical
where the alternatives are 14.7 % lossy" is concrete and publishable.
→ `ecosystem.md` §2

### 8. There is no schematic IPC API in KiCad 10.0.5

`schematic_commands.proto` contains no messages. Schematic work is file-based or
it does not happen. This retires a whole category of "should we use the API
instead?" and confirms the architecture; IPC is for the PCB phase only.
→ `ipc-api.md` §4.4

### 9. The rendering pipeline works, and it solves the font-metrics problem

Verified end-to-end: `kicad-cli` SVG (user units = mm, matching schematic
coordinates exactly) → `viewBox` crop → `<g id="kicli-annotations">` overlay →
PNG. Critically, KiCad's SVG emits every text item twice: an invisible `<text>`
with a **`textLength` computed by KiCad's own font engine**, plus a
`<g class="stroked-text"><desc>…</desc>` group. So exact glyph advances can be
*measured* rather than vendored from the GPL Newstroke table — the licensing
blocker in R7 dissolves. `resvg` is now MIT/Apache (it used to be MPL), so the
rasteriser is clean too.
→ `rendering.md` §3–§5, `kicad-cli.md` §5.5

### 10. The compact views hit their budget with room to spare — but connectivity is not pure geometry

Measured: connectivity view is **1.5 KB for a median (37-symbol) sheet** and
7.9 KB for the largest sheet in KiCad's entire demo corpus (234 symbols) — 50–113×
smaller than the source file, roughly 400 and 2,500 tokens respectively. JSON
costs 2.3× (minified) to 3.4× (pretty) more than the terse form.

The caveat that matters: a naive union-find over wires gives 37 nets where KiCad
reports 25. Adding the **name-based merges** (power-symbol values, labels) gives
exactly 25 — matching `kicad-cli`'s netlist. Connectivity extraction must
implement those rules explicitly, and the netlist comparison should be a
permanent test gate.
→ `representation.md` §3, §6

### Honourable mentions

- **`kicad-cli sch upgrade` silently destroys bus aliases** (they moved to
  `.kicad_pro` in v10 and the CLI does not save the project file). Never use it
  in a code path that touches user files. → `sch-format.md` §5.6
- **KiCad reorders every item on save** by `(type, uuid)`, so "byte-identical to
  the input" and "byte-identical to what KiCad would write" are different
  properties. → `sch-format.md` §1.1
- **`kicad-tools` (rjwalters) already ships `kct sch tidy`**, which repositions
  Reference/Value fields — so "nobody can move field text" is not an accurate
  framing of the ecosystem. → `ecosystem.md` §4.3
- **First `kicad-cli` run took >120 s** (fontconfig cache); warm runs are
  0.2–0.5 s. Warm it deliberately or an agent will think it hung.
  → `kicad-cli.md` §1.1

---

## Contradictions with `spec/SPEC.md`

Flagged at the top of each doc; consolidated here. None are fatal; all need a
line changed in the spec.

| # | SPEC location | Contradiction | Doc |
|---|---|---|---|
| C1 | §6 / Constitution §1 | "Byte-identity" is reachable but via a prettifier port, not whitespace preservation; and KiCad's item reordering means there are **two** distinct round-trip properties | `sch-format.md` ⚠1–2, `sexpr-strategy.md` ⚠1 |
| C2 | §3 D1 | "KiCad 10.0 formats" needs a concrete floor: 10.0.5 writes `20260306`, 10.0.0 writes earlier, `master` already writes `20260803` | `sch-format.md` ⚠3 |
| C3 | §6 | Grid discipline is stated in mm; the file is integer IU, and the rule cannot apply to field text without flagging KiCad's own autoplacement | `sch-format.md` ⚠4, `geometry.md` ⚠2 |
| C4 | §4 exit codes | SPEC's codes collide with `kicad-cli`'s (`3` and `5` mean different things); kicli must translate, never pass through | `kicad-cli.md` ⚠1 |
| C5 | §4 `sch erc` | Cannot use `kicad-cli`'s JSON coordinates as-is (100× bug) | `geometry.md` ⚠1, `kicad-cli.md` ⚠2 |
| C6 | §10 `--region` | `kicad-cli` has no region rendering; kicli must crop by `viewBox` (confirmed feasible) | `kicad-cli.md` ⚠3 |
| C7 | §9 | Scoring and ERC are treated as parallel; they must be **layered** — kicli must not re-implement any of ERC's 47 checks | `style-rules.md` ⚠1 |
| C8 | §9 `--gate` | Half of Tier 1 is ERC-owned, so gating implies requiring `kicad-cli` | `style-rules.md` ⚠2 |
| C9 | §9 | A 0–100 per-sheet score needs density normalisation, which SPEC does not specify | `style-rules.md` ⚠3 |
| C10 | §5 | "Delta keyed on content hash" is under-specified: hashing file bytes is useless because KiCad reorders items; hash per object by UUID | `representation.md` ⚠1 |
| C11 | §4 `sch view` | View scope (sheet vs project) is undefined, and connectivity is a hierarchy-level property | `representation.md` ⚠2 |
| C12 | §4 | Net names are unstable identifiers (`Net-(R1-Pad1)` changes on unrelated renumbering) | `representation.md` ⚠3 |
| C13 | §4 `wire` | Missing `wire connect <pin> <net>`, which is the common agent request | `wire-routing.md` ⚠2 |
| C14 | §7 vs §9 | The router's label threshold and the linter's long-wire rule must be one knob | `wire-routing.md` ⚠1 |
| C15 | D8 | Shared-library default `libs/parts` does not match your existing layout (`hardware/shared` + `${KIPRJMOD}/../shared/…`) | `libraries-and-vendoring.md` ⚠1 |
| C16 | §8 | Vendoring has no story for the embedded `lib_symbols` cache, which is what KiCad actually draws | `libraries-and-vendoring.md` ⚠4 |
| C17 | D3 | "Field text is Konnect's gap" is true of Konnect but not of the ecosystem (`kct sch tidy` exists) — the differentiation statement needs rewording | `ecosystem.md` ⚠1 |
| C18 | §14 / prior art | SPEC has no "why not X" section; two actively-maintained agent-focused tools exist | `ecosystem.md` ⚠2 |
| C19 | §11 | PCB ops require a *running* KiCad with the API enabled — a materially different UX with no headless fallback — and must be wrapped in `BeginCommit`/`EndCommit` | `ipc-api.md` ⚠1, ⚠3 |
| C20 | §10 `--annotate uuids` | UUID badges are unreadable at sheet scale; should be region-only and truncated | `rendering.md` ⚠3 |
| C21 | D18 / Constitution §11 | Fixtures must be purpose-built, so KiCad's GPL demo files can be an *external* corpus but not vendored | `sexpr-strategy.md` ⚠2 |

---

## Open questions needing your decision

Grouped by when they block work. **Bold = I recommend this answer.**

### Blocks M1 (parser core)

| # | Question | Doc |
|---|---|---|
| 1 | Which round-trip property is the merge gate: (a) semantic, (b) byte-identical for KiCad-authored files, (c) identical to what KiCad would write (needs adopting its item sort)? **Recommend (a)+(b) as gates, (c) informational.** | R1 Q2 |
| 2 | Version ceiling policy: refuse to *write* files whose version stamp exceeds kicli's known maximum? **Recommend yes, with a config knob.** | R1 Q3 |
| 3 | Non-canonical input: silently reformat with a `"reformatted": true` flag in the output, or refuse? And refuse to write files containing `#` comments unless `--allow-comment-loss`? **Recommend reformat-with-flag; refuse on comments.** | R2 Q1 |
| 4 | Preserve the input's prettifier mode (so `CompactSave` users' files stay compact) rather than always writing NORMAL? **Recommend yes.** | R2 Q2 |
| 5 | Approve `cargo xtask corpus` fetching KiCad's demos/`qa` at a pinned tag into `target/` for round-trip testing, keeping GPL files out of the repo? **Recommend yes.** | R2 Q3 |
| 6 | Treat embedded files/fonts as opaque, never re-encoded, refusing any op that would move them between files? **Recommend yes.** | R1 Q1 |
| 7 | Confirm v1 scope for schematic **variants** (round-trip only, no variant-aware editing) and **flat multi-top-level hierarchies** (read, but `--sheet` assumes one root). | R1 Q4, Q5 |
| 8 | Confirm MPL-2.0 dependencies are out under Constitution §9 (affects `via-kicad-sexp`; does **not** affect `resvg`, which is now MIT/Apache). | R2 Q4 |

### Blocks M2 (geometry + read)

| # | Question | Doc |
|---|---|---|
| 9 | Does the blocking off-grid rule apply only to *connectable* geometry, exempting field text? **Recommend yes** — otherwise KiCad's own autoplaced fields fail the lint. | R7 Q1 |
| 10 | Font metrics: measure advance widths from KiCad's SVG `textLength` and store our own table, rather than vendoring GPL Newstroke? **Recommend yes** (and validate against IPC `GetTextExtents` later). | R7 Q2, R5 Q2 |
| 11 | View scope default for `sch view`: whole project within a byte budget, falling back to an index + per-sheet summaries? **Recommend yes**, and say which happened. | R10 Q1 |
| 12 | Net naming: synthetic stable `n<k>` names instead of mirroring KiCad's `Net-(R1-Pad1)`, accepting that unnamed nets will not match the GUI's labels? **Recommend yes.** | R10 Q5 |
| 13 | Snapshot hash: SHA-256 truncated (fewer deps) or BLAKE3? **Recommend SHA-256.** Suppress power symbols from the symbol list (`--include-power` to show)? **Recommend yes.** Token counts in `view --stats`: bytes only? **Recommend bytes only.** | R10 Q2–Q4 |

### Blocks M3 (mutations)

| # | Question | Doc |
|---|---|---|
| 14 | On symbol move/rotate, do fields move rigidly with the symbol, keeping their angles? And does kicli always clear `fields_autoplaced` when it sets a position explicitly? **Recommend yes to both.** | R7 Q3 |

### Blocks M4 (wiring)

| # | Question | Doc |
|---|---|---|
| 15 | Sanity-check the router's corner penalty (`w_turn = 6`, i.e. "detour up to 6 grid steps to avoid a corner") and margin (8 G) against your taste. | R9 Q1, Q4 |
| 16 | Four-way junctions: refuse and offset by 1 G, reporting it? **Recommend yes** (R8 penalises them). | R9 Q2 |
| 17 | Approve the calibration test "re-route every net of a known-good sheet; assert total cost within X % of the original" — and pick X. **Suggest 15 %.** | R9 Q3 |

### Blocks M5 (score)

| # | Question | Doc |
|---|---|---|
| 18 | **Do you have `schematic-lint-rule-catalogue.md`?** It is not in the repo or its history. If it exists, I will reconcile rule IDs and tier assignments rather than keeping the ones invented in R8. | R8 Q1 |
| 19 | May `sch score --gate` require `kicad-cli` (because half of Tier 1 is ERC-owned), or must gating use kicli-native rules only? | R8 Q3 |
| 20 | Confirm the score shape: `100 · exp(−penalty/25)`, with Tier 1 failures **not** reducing the score but failing the gate independently. | R8 Q4 |
| 21 | Confirm the ground/negative-supply name list for the power-direction rule covers your Eurorack conventions (`GND`, `AGND`, `DGND`, `VSS`, `-12V`, …). | R8 Q5 |
| 22 | Should I work through Andrew Greenberg's KiCon talk video? R8's documentation rules are built from published summaries, not the talk itself. | R8 Q2 |

### Blocks M6 (render)

| # | Question | Doc |
|---|---|---|
| 23 | Default render style: black-and-white (best for vision models) or KiCad's colour theme? **Recommend B&W for JSON consumers, colour when a human asked.** Emit both SVG and PNG? **Recommend yes.** | R11 Q1, Q3 |
| 24 | Confirm `render.max_px = 1600` and the 6 px/mm legibility floor; and caching exported SVGs under `.kicli/render/` keyed on content hash. | R11 Q2, Q4 |

### Blocks M7 (libraries)

| # | Question | Doc |
|---|---|---|
| 25 | Adopt `../shared` + your existing nickname as the default shared-library layout instead of SPEC's `libs/parts`? **Recommend yes.** | R4 Q1 |
| 26 | `${KICAD9_3DMODEL_DIR}` references in the shared library: report only, or offer `kicli lib migrate-envvars`? **Recommend report first** — rewriting touches a submodule other projects share. | R4 Q2 |
| 27 | The shared nickname contains a space (`Eurorack Common`): warn only, or offer a project-wide rename? | R4 Q3 |
| 28 | Vendor-up conflict policy when `--into shared` would overwrite a differing part: **recommend refuse with a diff summary**. And copy 3D models for `--into project`, reference them for `--into shared`? | R4 Q4, Q5 |

### Blocks M9 (PCB)

| # | Question | Doc |
|---|---|---|
| 29 | **Licensing route for the IPC API.** KiCad's `.proto` files are GPL-3; the official Python client is MIT; `kicad-ipc-rs` is MIT with checked-in generated code; `kicad-api-rs` is GPL. **Recommend: depend on `kicad-ipc-rs` (MIT) for M9, and separately ask the KiCad devs to clarify/dual-license the protos.** | R5 Q1 |
| 30 | Pin a minimum KiCad version for `pcb` commands (10.0.0) and check `GetVersion` at connect? **Recommend yes.** | R5 Q4 |

### Cross-cutting / policy

| # | Question | Doc |
|---|---|---|
| 31 | `kicad-cli` is an optional dependency: commands that need it fail with a structured error rather than kicli bundling anything? **Recommend yes.** | R3 Q1 |
| 32 | ERC severities live in `.kicad_pro`. Does kicli only relabel them for its own output (safe), or may it edit `.kicad_pro`? **Recommend relabel only in v1.** | R3 Q2 |
| 33 | Expose `--variant` on export/render commands in v1, or hide it? | R3 Q3 |
| 34 | Should kicli detect a running KiCad holding the same document open (via IPC `GetOpenDocuments`) and warn before writing? | R5 Q3 |
| 35 | **Shall I file the ERC JSON 100× unit bug upstream?** It affects every consumer of `--format json`, not just kicli. | R7 Q4 |

---

## Suggested next steps

1. Answer the eight M1-blocking questions (1–8) — they are the only ones needed
   to start writing code.
2. Apply the C1–C21 spec edits, or tell me which you disagree with.
3. If `schematic-lint-rule-catalogue.md` exists, hand it over for reconciliation
   (question 18).
4. Then M1 tasks can be cut from `sch-format.md` + `sexpr-strategy.md`, each with
   the executable check Constitution §11 requires — the round-trip corpus, the
   prettifier oracle, and the 16-row pin-position fixture are all specified and
   ready to become tests.

## Research artefacts

The experiments live in this session's scratchpad, not in the repo (they are
research code, not shipping code). Each doc's "Reproduction" section has the
recipe to rebuild them from scratch:

- canonical 115-file v10 corpus (`kicad-cli sch upgrade --force` over KiCad's demos)
- `Prettify` port + corpus identity test (R2)
- 8-orientation pin-position fixture + ERC comparison (R7)
- 94-glyph font calibration sheet (R3 §5.5)
- connectivity extractor + view generator + netlist validation (R10)
- crop/overlay/raster pipeline (R11)
- `kiutils`/`kicad-skip` round-trip comparison (R6)
