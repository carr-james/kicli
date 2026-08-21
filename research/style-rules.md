# R8 — Schematic style rules: the scored catalogue

Supersedes the draft `schematic-lint-rule-catalogue.md`. **That file is not
present in this repository** (checked the working tree and the full git history
at `d2f3e93`); it was therefore not available to reconcile against, and this
catalogue is built from primary sources. If the draft exists elsewhere, hand it
over and this doc gets a reconciliation pass — see Q1.

Per the research brief, **Tier 3 is cut from scoring entirely.** Only Tier 1
(blocking) and Tier 2 (scored) appear below.

Prerequisite reading: [`geometry.md`](geometry.md) — every detection rule here is
expressed in terms of its primitives (resolved pin positions, oriented text
boxes, body boxes).

---

## ⚠ Contradictions and cautions for `spec/SPEC.md`

1. **KiCad 10's ERC already implements 47 checks**, including several that
   "schematic style" lists usually claim: `four_way_junction`,
   `endpoint_off_grid`, `similar_labels`, `label_dangling`,
   `unconnected_wire_endpoint`, `no_connect_dangling`, `duplicate_reference`
   (§2). SPEC §9 treats scoring and ERC as parallel; they should be *layered*:
   **kicli's lint engine must not re-implement any ERC check.** It consumes ERC
   findings and adds what ERC structurally cannot express — everything about
   *where things are drawn*.

2. **SPEC §9's "blocking rules fail `sch score --gate`" needs a source of truth
   for Tier 1.** Half the natural Tier 1 candidates are ERC errors, which means
   `--gate` implies running ERC, which implies `kicad-cli` is required for
   gating. Either accept that (recommended) or restrict `--gate` to kicli-native
   rules and say so loudly.

3. **A per-sheet 0–100 score is under-specified in SPEC §9** in one important
   way: normalisation. A sheet with 4 symbols and one crossing is worse than a
   sheet with 200 symbols and one crossing. §5 proposes density-normalised
   penalties; the alternative (absolute counts) will not survive calibration.

4. **Several rules need a "significant net" concept** that SPEC does not define
   (§4.9, §4.12). Proposed definition in §3.3.

---

## 1. Sources and their weight

| Source | What it gives | How much to trust it |
|---|---|---|
| **KiCad 10.0.5 ERC** (`eeschema/erc/erc_item.cpp`, `erc_settings.cpp`, tag `10.0.5`) | 47 named checks with default severities — the authoritative electrical layer | Highest: it is the tool's own definition of "wrong" |
| **Olin Lathrop's schematic rules** (widely-cited EE community canon; his answer on [Codidact "Rules and guidelines for drawing schematics?"](https://electrical.codidact.com/posts/278601), mirrored in many forum threads) | The classic layout canon: positive supplies up, grounds down, inputs left, outputs right, no 4-way junctions, minimise crossings, consistent symbol orientation | High for *direction* conventions; it is opinion, but it is the opinion the field has converged on |
| **Andrew Greenberg, "Making Actually Useful Schematics in KiCad", KiCon NA 2025** ([talk page](https://pretalx.kicad.org/kicon-na-2025/speaker/ZSYQTU/), [Hackaday write-up](https://hackaday.com/2025/11/21/making-actually-useful-schematics-in-kicad/), [video](https://www.youtube.com/watch?v=X0hd_v8qRiY)) | Context and documentation rules: MFR/MPN fields, datasheet annotations, purpose notes, voltage ranges, avoid 4-way junctions, prefer explicit connections over label-only, mono-PDF legibility, minimise DNP clutter, version/date discipline | High for the *documentation* tier; note that the primary source here is the talk itself — this catalogue was built from the published summaries, **the video was not watched** (Q2) |
| Vendor/industry review checklists (Cadence, Altium, community lists) | Confirmation that flow direction, net-label naming, junction-dot clarity, and NC marking are standard review items | Medium — largely restatements; useful as corroboration, not as primary authority |

**Honesty note on sourcing:** several of the "checklist" pages surfaced by search
are SEO content farms. Nothing in this catalogue rests on one of them alone;
where such a page is the only support for a rule, the rule is marked
`corroboration: weak` and is scored at low weight.

---

## 2. Division of labour: ERC vs kicli

### 2.1 What ERC already checks (KiCad 10.0.5)

```
bus_definition_conflict    bus_entry_needed          bus_to_bus_conflict
bus_to_net_conflict        different_unit_footprint  different_unit_net
duplicate_pins             duplicate_reference       duplicate_sheet_names
endpoint_off_grid          extra_units               field_name_whitespace
footprint_filter           footprint_link_issues     four_way_junction
ground_pin_not_ground      hier_label_mismatch       isolated_pin_label
label_dangling             label_multiple_wires      lib_symbol_issues
lib_symbol_mismatch        missing_bidi_pin          missing_input_pin
missing_power_pin          missing_unit              multiple_net_names
net_not_bus_member         no_connect_connected      no_connect_dangling
pin_not_connected          pin_not_driven            pin_to_pin
power_pin_not_driven       same_local_global_label   similar_label_and_power
similar_labels             similar_power             simulation_model_issue
single_global_label        stacked_pin_name          unannotated
unconnected_wire_endpoint  undefined_netclass        unit_value_mismatch
unresolved_variable        wire_dangling
```

Defaults worth knowing (`eeschema/erc/erc_settings.cpp:95-124`): most are
`ERROR`; `endpoint_off_grid`, `similar_labels`, `similar_power`,
`label_multiple_wires`, `unconnected_wire_endpoint`, `ground_pin_not_ground`,
`lib_symbol_issues`, `no_connect_*`, `missing_unit` are `WARNING`; and
**`four_way_junction` and `single_global_label` default to `IGNORE`**.

### 2.2 The rule

> kicli's lint engine implements **no** check that appears in §2.1. It runs ERC
> (when `kicad-cli` is available), maps findings into its own finding format, and
> adds only rules ERC cannot express.

Two deliberate exceptions, both because KiCad's default is `IGNORE`:

- `four_way_junction` → kicli **scores** it (Tier 2, `KI-JCT-001`) rather than
  relying on a check the user's project has switched off.
- `single_global_label` → same treatment under `KI-LBL-003`.

kicli re-reports these as *style* findings, clearly attributed, and does not
duplicate them if the project has ERC's version enabled.

### 2.3 Why the split matters

ERC operates on the netlist graph. Everything in Constitution §3's second axis —
"a schematic is a drawing" — is invisible to it. That is precisely kicli's
territory, and it is why the score can be additive on top of ERC rather than
overlapping with it.

---

## 3. Definitions used by the rules

### 3.1 Primitives (from R7)

| Symbol | Meaning |
|---|---|
| `pin(s, p)` | absolute position of pin `p` of symbol `s` (`geometry.md` §3.1) |
| `dir(s, p)` | unit direction the pin points, after transform |
| `body(s)` | body bounding box: graphics + pins, no text (`geometry.md` §6) |
| `full(s)` | body ∪ visible field boxes |
| `tbox(t)` | oriented text box of text item / field `t` (`geometry.md` §5) |
| `seg(w)` | wire segment as an ordered pair of endpoints |
| `G` | grid step, default 12700 IU (50 mil) |

All arithmetic is exact integer IU. Every threshold below is expressed in grid
units so that the rules are resolution-independent and configurable.

### 3.2 Sheet metrics

```
N_sym   = number of placed symbols (excluding power symbols)
N_pwr   = number of power symbols
N_wire  = number of wire segments
N_net   = number of nets
A_page  = page area (from (paper …))
A_used  = area of the bounding box of all drawn items
density = Σ area(body(s)) / A_used
```

### 3.3 "Significant net"

A net is **significant** if it satisfies any of:

- it has ≥ 3 connected pins, or
- it spans a bounding box whose diagonal ≥ 20 G (≈ 254 mm at default grid), or
- it carries a user-authored label (not an auto-generated `Net-(…)` name), or
- it is a power net (connected to a power-symbol pin or a `power_in` pin).

Rationale: rules about naming, labelling, and routing quality should not fire on
the two-pin decoupling-cap stub between a pin and a via-equivalent.

### 3.4 Finding format (ESLint-style, per SPEC §9)

```json
{ "rule": "KI-FLOW-001", "tier": 2, "severity": "warning",
  "sheet": "/Power", "pos": {"x": 123.19, "y": 45.72},
  "objects": ["uuid…", "uuid…"],
  "message": "Power symbol +3V3 points down",
  "fix": "kicli sym rotate <uuid> --to 0",
  "penalty": 3.0 }
```

`fix` is a *suggested command*, not an auto-fix — kicli never mutates during
scoring.

---

## 4. The catalogue

Legend — **T**: tier (1 = blocking, 2 = scored). **W**: default weight (penalty
points per occurrence before normalisation). **Knob**: `kicli.toml [rules]` key.

The knob names below are this note's drafts. Where a rule shares a setting with
something outside the score, the shared key wins and is named here — see
`KI-LBL-001`. The rest are settled when the scorer is built, against
`spec/SPEC.md` §15's `[rules."KI-…"]` tables.

### Tier 1 — blocking (fail `sch score --gate`, weight n/a)

#### KI-GRID-001 — connectable geometry off grid
- **T** 1. **Why**: Constitution §7; off-grid pins silently fail to connect.
- **Detect**: for every pin `p` of every symbol, every wire endpoint, junction,
  no-connect, label anchor and sheet pin position `q`:
  `q.x mod G ≠ 0 ∨ q.y mod G ≠ 0` → finding.
- **Scope note**: field/text positions are **exempt** (`geometry.md`
  Contradiction 2 — KiCad's own autoplacement puts them off grid).
- **Overlap with ERC**: ERC's `endpoint_off_grid` covers *wire endpoints* only,
  as a warning. KI-GRID-001 additionally covers pins, labels and sheet pins, and
  is blocking. Report ERC's finding when present; do not double-count.
- **Knob**: `grid = "50mil"`, `grid.exempt_text = true`.

#### KI-OVL-001 — symbol bodies overlap
- **T** 1. **Why**: unreadable, and usually a placement bug.
- **Detect**: for all pairs `s≠t`: `body(s) ∩ body(t) ≠ ∅` (exact box
  intersection, ≥ 1 IU). Power symbols included.
- **False positives**: deliberately overlapping decorative symbols are rare
  enough to justify an explicit `--allow` list rather than a soft rule.
- **Knob**: `overlap.symbol = "error"`.

#### KI-WIRE-001 — wire crosses a symbol body
- **T** 1. **Why**: reads as a connection that isn't one; hides pins.
- **Detect**: segment/box intersection between `seg(w)` and `body(s)`, excluding
  the ≤ 1 G stub at each end where the wire legitimately meets a pin of `s`.
  Formally: clip `seg(w)` to `body(s)`; finding if the clipped length > 0 and
  neither endpoint of the clipped part is within 1 IU of a pin of `s`.
- **Knob**: `wire.through_symbol = "error"`.

#### KI-TXT-001 — text illegible: overlapping text
- **T** 1. **Why**: two strings drawn on top of each other cannot be read; this
  is the failure mode that motivated kicli (SPEC D3).
- **Detect**: for all pairs of visible text objects `(a,b)` (fields, labels,
  free text, sheet names, pin names/numbers): oriented-box intersection with
  area > 20 % of `min(area(a), area(b))`.
- **Note**: uses oriented boxes, not AABBs — schematic text is routinely at 90°.
- **Knob**: `text.overlap_ratio = 0.2`, `text.overlap = "error"`.

#### KI-CONN-001 — pin touches a wire but is not connected
- **T** 1. **Why**: it looks connected and is not. A pin whose connection point
  lands on a wire's interior with no junction there reads, on screen and to a
  reviewer, exactly like a connection. KiCad 10.0.5's netlister does not merge
  it, so the board is wired differently from the way the schematic reads. This
  is the most expensive class of schematic defect: it survives review.
- **Also catches the label-plus-pin case**: a pin sharing a mid-wire anchor with
  a label forms a net with the label and leaves the wire out, which draws as a
  connection and is not one (`notes/label-on-wire-interior.md`). No extra
  detection is needed — the pin's net is not the wire's net, which is the test
  below.
- **Detect**: a byproduct of the corrected extractor, needing no new geometry.
  For every pin connection point `p` and wire segment `w` where `p` lies on the
  interior of `w` (not within 1 IU of either endpoint): finding when `p` and `w`
  are in different nets after union-find. Geometric coincidence without
  electrical merge is the whole test. Sheet pins are covered the same way.
- **Fix hint**: `add a junction at <x>,<y>` — that is the one-item change that
  makes the drawing mean what it looks like. The alternative, moving the symbol
  off the wire, is a layout decision and is not suggested automatically.
- **Overlap with ERC**: none. KiCad's 47 checks have nothing for this; the
  netlister simply reports two nets, which is not a violation from its point of
  view. That makes it kicli's to catch, and it is the clearest example so far of
  what "where things are drawn" adds to electrical correctness.
- **Evidence**: measured against KiCad 10.0.5, both directions, in
  [`notes/pin-on-wire-interior.md`](notes/pin-on-wire-interior.md). The fixture
  `crates/kicli/tests/fixtures/sch/nets/nets.kicad_sch` carries one cluster of
  each kind, so the rule has a positive and a negative case from the day it is
  written.
- **Knob**: `connection.pin_on_wire = "error"`.

#### KI-HIER-001 — sheet pin / hierarchical label mismatch
- **T** 1. **Delegated to ERC** (`hier_label_mismatch`). Listed here only so the
  catalogue is complete; kicli reports ERC's finding and gates on it.

### Tier 2 — scored

#### KI-FLOW-001 — power symbol direction
- **T** 2. **W** 3. **Source**: Olin Lathrop (canon); Greenberg.
- **Rule**: positive-supply symbols point **up**, ground/negative point **down**.
- **Detect**: for each power symbol `s` with its single pin `p`:
  `d = dir(s, p)` after transform. Classify `s` as *positive* or *ground* from
  its `Value` (ground set: `GND`, `GNDA`, `GNDD`, `AGND`, `DGND`, `VSS`,
  `0V`, `EARTH`, and anything matching `^-?V?SS$`; negative set: value starts
  with `-`). Finding if a positive symbol's pin does not point up (screen −Y) or
  a ground symbol's pin does not point down (+Y). Negative supplies (`-12V`)
  behave like grounds.
- **Note**: the pin direction is what is visually meaningful, not the symbol's
  rotation value, because the library symbol may already be drawn pointing down.
- **Knob**: `flow.power_direction = 3`, `flow.ground_names = [...]`.

#### KI-FLOW-002 — signal flow left-to-right across sheet ports
- **T** 2. **W** 4 (per sheet, not per object). **Source**: Olin; Greenberg;
  every vendor checklist.
- **Detect**: consider hierarchical labels and global labels with a direction
  shape. Let `x̄_in` = mean x of `shape input`, `x̄_out` = mean x of
  `shape output`, over the sheet's content bounding box width `W`.
  Score term `= clamp((x̄_in − x̄_out) / W, 0, 1)` — 0 when inputs are left of
  outputs, rising to 1 when fully reversed. Emit one finding when > 0.25.
- **Why a sheet-level rule**: individual label positions are not wrong; the
  *aggregate* direction is what a reader perceives.
- **Knob**: `flow.lr_threshold = 0.25`, `flow.lr_weight = 4`.

#### KI-XING-001 — wire crossings without junction
- **T** 2. **W** 1 per crossing, density-normalised (§5). **Source**: Olin;
  universal.
- **Detect**: count pairs of wire segments that intersect at a point that is not
  an endpoint of both and has no junction. Use exact integer orientation tests
  (no floating point). Normalise: `crossings / max(1, N_wire/10)`.
- **Note**: crossings are not errors — a dense sheet needs some. The *rate* is
  the signal.
- **Knob**: `routing.crossing_weight = 1`, `routing.crossing_free_allowance = 2`.

#### KI-JCT-001 — four-way junction
- **T** 2. **W** 2. **Source**: Olin ("no 4-way junctions"); Greenberg; KiCad ERC
  has the check but defaults to `IGNORE`.
- **Detect**: a junction with exactly 4 collinear-pair wire ends meeting, i.e.
  degree-4 node in the wire graph at a junction point.
- **Rationale for keeping it kicli-side**: default-off in ERC, so a project that
  never touched ERC settings would silently pass.
- **Knob**: `routing.four_way = 2`.

#### KI-RTE-001 — dogleg count per net
- **T** 2. **W** 0.5 per excess corner. **Source**: readability canon; also the
  cost function R9 will optimise.
- **Detect**: for each significant net, count wire corners `C`. Lower bound
  `C_min` = 1 for an L-shaped two-pin connection, 0 if the pins are collinear.
  Penalty on `max(0, C − C_min − allowance)`, `allowance = 1`.
- **Knob**: `routing.dogleg_weight = 0.5`, `routing.dogleg_allowance = 1`.

#### KI-RTE-002 — wire length vs Manhattan lower bound
- **T** 2. **W** 2 per net exceeding the ratio. **Source**: derived; correlates
  with "spaghetti".
- **Detect**: for each significant net, `L_actual` = total wire length,
  `L_min` = Manhattan distance of the minimum spanning tree over its pin
  positions. Finding when `L_actual / L_min > 1.6`.
- **Knob**: `routing.length_ratio = 1.6`.

#### KI-LBL-001 — long connection drawn as a wire instead of a label
- **T** 2. **W** 2. **Source**: Olin; Greenberg (with the counter-pressure that
  connections should be explicit where possible — hence a *threshold*, not a
  ban).
- **Detect**: a net whose wire path's bounding-box diagonal ≥ `label_threshold`
  (default 300 G = 381 mm, i.e. more than a sheet width) and which carries no
  label.
- **Interaction with R9**: this threshold is the same one the router uses to
  decide "emit paired net labels instead of a wire". They must be one knob.
- **Knob**: `routing.label_threshold = "300G"` — the key the router reads, and
  the only one. C14 resolved this rule's threshold and the router's into one
  key; `spec/SPEC.md` §15 carries it, and neither side may grow one of its own.

#### KI-LBL-002 — auto-generated net name on a significant net
- **T** 2. **W** 1.5. **Source**: Olin ("significant nets get short
  self-explanatory names"); Greenberg.
- **Detect**: net is significant (§3.3) and its name matches
  `^Net-\(.*\)$` or `^unnamed` (KiCad's auto-name shapes).
- **Knob**: `labels.require_names_on_significant = 1.5`.

#### KI-LBL-003 — global label used exactly once
- **T** 2. **W** 1. **Source**: ERC `single_global_label` (default `IGNORE`).
- **Detect**: a global label whose name appears on exactly one sheet and once
  overall — i.e. it promises a cross-sheet connection that does not exist.
- **Knob**: `labels.lonely_global = 1`.

#### KI-TXT-002 — text collides with a wire
- **T** 2. **W** 1. **Source**: readability; the mono-PDF legibility item from
  Greenberg.
- **Detect**: `tbox(t)` intersects `seg(w)` for a wire `w` not belonging to the
  net the text names. Exclude the label's own attachment stub (within 1 G of the
  label anchor).
- **Knob**: `text.wire_collision = 1`.

#### KI-TXT-003 — inconsistent text sizes
- **T** 2. **W** 2 per extra size class beyond the allowance.
- **Detect**: histogram of distinct `(size.y)` values across visible text,
  excluding the title block. Penalty on `max(0, distinct − 3)`.
- **Rationale**: three sizes (labels, refdes/values, headings) is a coherent
  typographic system; six is noise.
- **Knob**: `text.max_size_classes = 3`.

#### KI-FLD-001 — reference/value placement inconsistency
- **T** 2. **W** 1 per non-conforming symbol.
- **Detect**: for each symbol, compute the field's offset from the symbol anchor
  in *body-local* coordinates (i.e. apply `M⁻¹`). Cluster the offsets per
  library symbol type; a symbol whose `Reference` offset differs from its type's
  modal offset by > 2 G is a finding.
- **Rationale**: this is exactly the thing a human sees instantly ("that R's
  designator is on the wrong side") and no other tool checks.
- **Knob**: `fields.placement_tolerance = "2G"`.

#### KI-FLD-002 — hidden or missing designator/value on a non-power symbol
- **T** 2. **W** 3.
- **Detect**: `Reference` or `Value` field has `(hide yes)` or empty text, and
  the symbol is not a power symbol.
- **Knob**: `fields.require_visible_ref_value = 3`.

#### KI-DOC-001 — missing manufacturer/part number on a BOM part
- **T** 2. **W** 1 per part class (grouped by `Value` + `Footprint`, not per
  instance — otherwise a 100-cap design drowns the score). **Source**: Greenberg
  (MFR/MPN).
- **Detect**: symbol is in the BOM (`in_bom yes`, not DNP) and has no non-empty
  field matching `MPN|Manufacturer Part|Mfr. Part|Part Number` (configurable).
- **Knob**: `docs.mpn_fields = [...]`, `docs.mpn_weight = 1`.

#### KI-DOC-002 — missing datasheet on an active part
- **T** 2. **W** 0.5 per part class. **Source**: Greenberg.
- **Detect**: `Datasheet` empty on a symbol with ≥ 4 pins (heuristic for
  "active/complex part").
- **Knob**: `docs.datasheet_min_pins = 4`.

#### KI-DOC-003 — title block incomplete
- **T** 2. **W** 2 per sheet. **Source**: Greenberg (version/date discipline).
- **Detect**: `title_block` missing or missing any of `title`, `rev`, `date`.
- **Knob**: `docs.title_block_required = ["title","rev","date"]`.

#### KI-DOC-004 — no explanatory text on a sheet
- **T** 2. **W** 1 per sheet. **Source**: Greenberg (notes explaining purpose,
  voltage ranges). `corroboration: weak` as a *mechanical* rule — the presence
  of text is a poor proxy for the presence of explanation, so the weight is low.
- **Detect**: sheet has zero `text`/`text_box` items outside the title block.
- **Knob**: `docs.require_sheet_notes = 1`.

#### KI-LAY-001 — page utilisation
- **T** 2. **W** up to 4 (continuous).
- **Detect**: `u = A_used / A_page`. Penalty `= 4 · clamp((0.35 − u)/0.35, 0, 1)`
  when `u < 0.35` (content huddled in a corner of a too-large page), and
  `4 · clamp((u − 0.92)/0.08, 0, 1)` when `u > 0.92` (no margin).
- **Knob**: `layout.util_min = 0.35`, `layout.util_max = 0.92`.

#### KI-LAY-002 — sheet overcrowding
- **T** 2. **W** up to 6 (continuous). **Source**: Greenberg's "one page, one
  idea" framing; hierarchical design canon.
- **Detect**: penalty `= 6 · clamp((N_sym − 60)/60, 0, 1)`. Sixty non-power
  symbols on one sheet is the point where a sheet stops being readable at A3.
- **Note**: this rule is the one most likely to need calibration; the constant is
  a starting hypothesis, not a measurement.
- **Knob**: `layout.max_symbols_per_sheet = 60`.

#### KI-LAY-003 — symbol alignment
- **T** 2. **W** 1 per unaligned symbol.
- **Detect**: project symbol anchors onto x and y; cluster with tolerance 1 G. A
  symbol is *aligned* if it shares an x-cluster or a y-cluster with ≥ 2 others.
  Finding for symbols in neither (excluding power symbols and single-symbol
  sheets).
- **Knob**: `layout.alignment_tolerance = "1G"`.

#### KI-DNP-001 — DNP clutter
- **T** 2. **W** 1 per DNP part beyond the allowance. **Source**: Greenberg
  ("minimise just-in-case/DNP parts").
- **Detect**: count symbols with `(dnp yes)`; penalty on
  `max(0, count − max(2, 0.05·N_sym))`.
- **Knob**: `dnp.allowance_ratio = 0.05`.

#### KI-SYM-001 — inconsistent orientation for two-terminal parts
- **T** 2. **W** 0.5. **Source**: Olin (consistent orientation).
- **Detect**: for each library symbol type with exactly 2 pins, take the modal
  orientation across the sheet; each symbol differing from the mode *and* not
  required by its wiring (both its pins connect along the other axis) is a
  finding.
- **Knob**: `symbols.orientation_consistency = 0.5`.

### 4.x Rules deliberately excluded

| Candidate | Why not |
|---|---|
| "Pin not connected", "duplicate reference", "similar labels", … | ERC owns them (§2.2) |
| "Use schematic symbols not packages" (Greenberg) | Not mechanically detectable without judging symbol art |
| "Design for test / design for fail" (Greenberg) | Judgement, not geometry — Tier 3 by the brief's definition |
| "Renders legibly to mono PDF" (Greenberg) | Partly covered by KI-TXT-001/002 and KI-TXT-003; a direct check would need rasterisation and human judgement |
| "Correct decoupling per IC" | Electrical judgement, needs part knowledge; out of scope for a deterministic geometry engine |

---

## 5. Scoring model

Deterministic, per sheet, then aggregated per project.

```
raw_penalty(sheet) = Σ_rules  w_r · n_r · norm_r
```

where `n_r` is the occurrence count (or the continuous term for rules that
define one) and `norm_r` is the rule's normaliser:

| Normaliser | Applies to | Definition |
|---|---|---|
| `per_object` | field/symbol/text rules | `1 / max(1, N_sym/20)` — a 20-symbol sheet is the reference size |
| `per_wire` | crossings, doglegs | `1 / max(1, N_wire/10)` |
| `per_sheet` | flow, layout, docs | `1` |

Then

```
score(sheet) = round( 100 · exp( −raw_penalty / K ) ),  K = 25
```

Why exponential rather than `100 − penalty`: it is monotone, never goes
negative, has diminishing marginal punishment (the difference between a bad
sheet and a terrible one matters less than between a good one and a mediocre
one), and it gives a stable dynamic range without clamping. `K` is the single
tuning constant and is set by calibration (§6).

Project score = symbol-count-weighted mean of sheet scores.

Blocking (Tier 1) findings do **not** enter the score. They set
`"gate": "fail"`. A schematic can score 96 and still fail the gate; that is
intentional and must be visible in the output.

Determinism requirements (Constitution §4): integer geometry only; no floating
point in *detection* (only in the final score arithmetic, where it is applied to
a deterministic penalty sum with fixed rounding); findings sorted by
`(rule, sheet, x, y, uuid)` before output.

---

## 6. Calibration method

The brief's requirement: "score known-good open hardware schematics vs
known-bad generated ones; ranking must match human judgement."

### 6.1 Corpus

| Set | Contents | Source |
|---|---|---|
| **A — good** | 8–12 sheets from well-regarded open hardware, plus KiCad's own demo projects, which are drawn by KiCad developers and are stylistically conservative | external, fetched (not vendored — same licensing reasoning as `sexpr-strategy.md` §Contradiction 2) |
| **B — bad** | Purpose-built degradations of set A: programmatically rotate power symbols, scatter fields, replace labels with long wires, shuffle symbols off alignment, oversize a sheet | generated in-repo by a fixture tool — these *are* purpose-built and belong in `fixtures/` |
| **C — agent output** | Sheets produced by an LLM agent driving kicli with no style feedback | generated once M3 exists |

Set B is the key idea: because each bad sheet is a *known perturbation* of a
good one, the expected ranking is known exactly, with no human labelling needed
for the basic monotonicity property.

### 6.2 Properties to assert

1. **Monotonicity under degradation** (automatic, no humans):
   `score(A_i) > score(B_i,k)` for every sheet `i` and every degradation `k`,
   and `score` decreases monotonically as degradations are stacked.
2. **Rule isolation**: applying degradation `k` changes only the penalty of the
   rules it targets (catches accidental coupling between rules).
3. **Human ranking agreement**: James ranks ~20 sheet pairs drawn from A ∪ C;
   require Kendall's τ ≥ 0.7 against kicli's ranking. This is the only step
   needing a human, and 20 pairs is ~15 minutes.
4. **Stability**: re-scoring an unchanged file gives a bit-identical result
   (Constitution §4); re-scoring after a no-op mutation (e.g. move a symbol and
   move it back) also does.

### 6.3 Weight fitting

Do **not** fit weights by regression on a handful of human labels — it will
overfit. Procedure instead:

1. Set all weights to the defaults in §4 (they encode a prior: direction and
   legibility errors matter most, documentation least).
2. Run properties 1–2. Fix rules, not weights, when they fail.
3. Run property 3. If τ < 0.7, examine the *disagreeing pairs* individually and
   change at most one weight per iteration, recording the reason in this file's
   changelog.
4. Freeze `K` last, by requiring that set A lands in 85–100 and set B's
   worst-degraded lands in 30–50.

---

## 7. Config surface (`kicli.toml`)

```toml
[rules]
default_tier2_enabled = true
gate_on_tier1 = true
consume_erc = true              # run kicad-cli sch erc and merge findings

[rules.grid]
grid = "50mil"
exempt_text = true

[rules."KI-XING-001"]
enabled = true
weight = 1.0
free_allowance = 2

[rules."KI-LAY-002"]
max_symbols_per_sheet = 60
weight = 6.0
```

Every rule supports `enabled`, `severity`, `weight`; rule-specific thresholds are
named in §4. Unknown keys are an error, not a warning (agents typo silently).

---

## 8. Open questions for James

- **Q1 — the seed catalogue.** `research/schematic-lint-rule-catalogue.md` is not
  in the repo or its history. If you have it, send it and I will reconcile rule
  IDs and any Tier 1/2 assignments you had already made, rather than imposing
  the IDs invented here.

- **Q2 — Greenberg's talk.** This catalogue used the published summaries of
  "Making Actually Useful Schematics in KiCad" (KiCon NA 2025), not the video.
  If his checklist is published in a citable form, it should be the primary
  source for the KI-DOC-* family. Want me to work through the video?

- **Q3 — ERC coupling.** Confirm `sch score --gate` may require `kicad-cli`
  (because half of Tier 1 is ERC-owned), or state that gating uses kicli-native
  rules only.

- **Q4 — Score shape.** Confirm the exponential mapping and `K = 25` as the
  starting point, and the rule that Tier 1 failures do not reduce the score but
  fail the gate independently.

- **Q5 — Ground-name list.** KI-FLOW-001 needs a project-configurable ground/
  negative-supply name set. Confirm the default list in §4 covers your Eurorack
  conventions (`-12V`, `+12V`, `AGND`, `DGND`…).

---

## 9. Sources

- KiCad 10.0.5 source, tag `10.0.5`: `eeschema/erc/erc_item.cpp`,
  `eeschema/erc/erc_settings.cpp:95-124`.
- Olin Lathrop, schematic drawing rules —
  <https://electrical.codidact.com/posts/278601> (his canonical answer; the page
  itself returned 403 to automated fetch, so the rule content here comes from
  widely-reproduced summaries of it and should be spot-checked against the
  original).
- Andrew Greenberg, "Making Actually Useful Schematics in KiCad", KiCon NA 2025:
  <https://pretalx.kicad.org/kicon-na-2025/speaker/ZSYQTU/>,
  <https://hackaday.com/2025/11/21/making-actually-useful-schematics-in-kicad/>,
  <https://www.youtube.com/watch?v=X0hd_v8qRiY>.
- Corroborating checklists: Cadence "Electrical Schematic Design Checklist"
  <https://resources.pcb.cadence.com/blog/2024-electrical-schematic-design-checklist>;
  Altium "Guidelines for Creating Useful PCB Schematic Symbols"
  <https://resources.altium.com/p/guidelines-creating-useful-schematic-symbols>.
