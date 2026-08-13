# R10 — Compact representation design

Status: the three views are specified below and **measured on real schematics**.
A working connectivity extractor was built for this research and validated
against `kicad-cli`'s own netlist: for `demos/complex_hierarchy/ampli_ht`, both
produce **exactly 25 nets** (§3.2).

Headline numbers (measured, §6):

| Sheet | symbols | raw `.kicad_sch` | connectivity view | compression |
|---|---|---|---|---|
| `ampli_ht` | 46 | 112,996 B | **1,463 B** | 77× |
| `in_out_conn` | 130 | 359,095 B | **6,933 B** | 52× |
| `csi` | 125 | 702,137 B | **6,215 B** | 113× |
| `One-Air-Max` | 234 | 624,834 B | **7,896 B** | 79× |

Prerequisite reading: [`sch-format.md`](sch-format.md) §3.6 (path/handle model),
[`geometry.md`](geometry.md) §3 (pin resolution).

---

## ⚠ Contradictions and cautions for `spec/SPEC.md`

1. **SPEC §5 says "delta view keyed on content hash snapshots" — that under-
   specifies what is hashed.** Hashing file bytes makes the delta useless
   (KiCad reorders items on save, `sch-format.md` §1.1, so every save looks like
   a total rewrite). The snapshot must be a hash *per object*, keyed by UUID, over
   that object's semantic content. §5 specifies it.

2. **SPEC §4's `sch view --view connectivity` needs a sheet-scope decision.**
   Connectivity is a property of the *hierarchy*, not of one file: a net leaving
   through a sheet pin continues elsewhere. Views must state their scope
   explicitly (`--sheet` = one sheet with ports marked; default = whole project
   with per-sheet sections). Q1.

3. **Net names are not stable identifiers.** KiCad derives unnamed net names
   from an arbitrary member pin (`Net-(R1-Pad1)`), so they change when an
   unrelated symbol is renumbered. Any agent-facing view that uses net names as
   handles will produce diffs that look like electrical changes and are not.
   §3.4 proposes stable synthetic net ids alongside the display name.

4. **Connectivity is not purely geometric.** A naive union-find over wire
   endpoints gives 37 nets on a sheet where KiCad reports 25; the difference is
   entirely the *name-based* merges (power symbols, labels). SPEC's M2
   "connectivity extraction" must implement those merge rules explicitly, and
   the netlist comparison against `kicad-cli` should be a test gate (§3.2).

---

## 1. What the corpus actually looks like

Measured over the 115-sheet KiCad 10.0.5 demo corpus (recipe C in
`sch-format.md` §0.2):

| Statistic | Value |
|---|---|
| median symbols per sheet | **37** |
| largest sheet | 374 symbols, 719 wires (`dcdc.kicad_sch`) |
| largest file | 1.98 MB (`front-panel-io.kicad_sch`, 200 symbols) |
| typical mid-size sheet | 125–235 symbols, 300–520 wires, 100–120 labels |

So "a realistic Eurorack sheet" (30–60 symbols) sits comfortably *below* the
median of this corpus. The budget targets in §6 are set against the 130–234
symbol sheets, which is the pessimistic case.

File size is dominated by `lib_symbols` (the embedded library cache) and by
`(xy …)` runs, neither of which an agent needs to see. That is where the 50–110×
compression comes from.

---

## 2. Design principles

1. **Line-oriented, one record per line.** Agents edit and quote lines; a line
   is also the natural diff unit.
2. **Every line starts with a one-letter record type.** Cheap to parse, cheap to
   grep, and it lets an agent filter without a parser (`grep '^N '`).
3. **Terse text is the default; JSON is the twin.** Measured overhead of JSON:
   **2.3× minified, 3.4× pretty** (§6.2). At agent context prices that is not a
   rounding error.
4. **Identifiers before coordinates.** Connectivity mentions no coordinates at
   all; the layout digest is where geometry lives. Most agent tasks need only
   one of the two.
5. **Stable ordering.** Symbols by refdes (natural sort), nets by descending pin
   count then by name, so that a diff between two runs shows only real changes.
6. **Reduced precision by design.** Layout coordinates in 0.01 mm — one decimal
   place finer than the 50 mil grid needs, three digits shorter than the file.

---

## 3. View 1 — connectivity

### 3.1 Grammar

```
sheet <path>  sym=<n> pwr=<n> nets=<n>
# S ref value lib
S <ref> <value> <libname>            ; one per non-power symbol
# N name: pins
N <net>: <ref>.<pin> <ref>.<pin> …   ; one per net, pins sorted
H <subsheet>: <port>(<i|o|b|t|p>) …  ; hierarchy ports of child sheets
P <port>(<dir>)                      ; this sheet's own hierarchical labels
```

Real output (`ampli_ht`, first lines, verbatim from the prototype):

```
sheet ampli_ht.kicad_sch  sym=30 pwr=16 nets=25
# S ref value lib
S C201 15nF C
S C202 150nF C
S D201 1N4148 D_Small
S P201 CONN_2 CONN_2
S Q201 MPSA92 MPSA92
S R201 1K R
…
# N name: pins
N GND: C201.2 C202.2 C203.2 R205.1 R209.1 …
N HT: R210.2 R211.2 …
```

Power symbols are **not** listed as symbols — they are net-name carriers, and
listing 16 `#PWR0xx` entries is pure noise. They appear implicitly as net names.

### 3.2 Net construction rules (validated)

Union-find over these six merge relations, in order. Every one is measured
against KiCad 10.0.5, and every one has a note carrying its evidence, both
directions of it, and a reproduction recipe.

1. **shared point** — items whose connection points coincide are one net. A
   wire connects at its two ends and nowhere between them. A **bus entry** is
   the exception: it carries one member of a bundle, so it joins no other bus
   entry, no bus, no bus junction and no bus label. Two entries drawn towards
   one point of a bundle carry two different members, and joining them shorts
   two nets ([`notes/bus-entry-joins-nothing.md`](notes/bus-entry-joins-nothing.md)).
2. **junction** — a junction merges everything that meets at its point, a
   segment interior included. A pin or sheet pin that merely *lies* on a
   wire's interior does **not** merge, two wires crossing without a junction do
   **not** merge, and a wire endpoint on another wire's interior does not
   merge — that is the whole point of junction dots.

   **Corrected 2026-08-13.** This rule previously said that a pin on a segment
   interior merges. Measured against KiCad 10.0.5 it does not: two clusters of
   `crates/kicli/tests/fixtures/sch/nets/nets.kicad_sch` are identical apart
   from a junction, and only the one with the junction takes the mid-span pin
   into the net. A control run adding a junction to the other cluster flips it
   as well, so the junction is the whole of the difference. Evidence in
   [`notes/pin-on-wire-interior.md`](notes/pin-on-wire-interior.md). The 25-net
   agreement below is unaffected: no pin in `ampli_ht` sits mid-wire.
3. **label on a segment** — a label whose anchor lies on a segment interior
   joins every segment meeting at that anchor when there are two or more, and
   joins the single segment otherwise — but only when no pin, other label,
   sheet pin or no-connect shares the anchor. A label and a pin together at a
   mid-wire point form their own net and leave the wire out of it. **Added
   2026-08-13**; evidence in
   [`notes/label-on-wire-interior.md`](notes/label-on-wire-interior.md).
4. **name** — items of equal name join, and one sheet is one namespace: a
   local label, a hierarchical label, a global label and a power pin that
   carry one name on one sheet are one net, whatever their kinds. A global
   label and a power pin carry that name across the whole project as well, and
   a hierarchical label meets the like-named pin of the sheet symbol that draws
   its placement. Names are compared as KiCad escapes them, so `A/B` and
   `A{slash}B` are one name and `A-B` and `A_B` are two.

   **Widened 2026-08-13.** This rule previously kept each kind of label in a
   namespace of its own, which split five nets of KiCad's own CM5 demo
   ([`notes/one-sheet-one-namespace.md`](notes/one-sheet-one-namespace.md),
   [`notes/escaped-net-names.md`](notes/escaped-net-names.md)).
5. **power pin** — a pin names a net when it is a power input, either on a
   power symbol, by the symbol's `Value`, which is what makes `GND` one net,
   or **hidden on an ordinary symbol**, by the pin's own name, which is what
   puts a 74-series part's invisible `VCC` on the rail. A power output names
   nothing, which is what `PWR_FLAG` is
   ([`notes/hidden-power-pin.md`](notes/hidden-power-pin.md)).
6. **bundle member** — a bundle carries its members. A net named after one
   member, on any sheet the bundle reaches, is that member, and no wire between
   them is needed. `AN[0..7]` carries `AN0` to `AN7`; `ANALOG{A[0..5]}` carries
   `ANALOG.A0` to `ANALOG.A5`; `I2C{SCL, SDA}` carries `I2C.SCL` and `I2C.SDA`
   ([`notes/bundle-members.md`](notes/bundle-members.md)).

   **Open.** Two bundles of different names, wired together, share their
   members: KiCad puts `UART.RX` and `UART_TRG.RX` on one net in
   `demos/royalblue54L_feather`, and `VRAM31` and `DQ31` on one net in
   `demos/video`. Four probes failed to reproduce the correspondence, and they
   are recorded in the note so the next attempt does not repeat them. This is
   the whole of the remaining difference against the demo corpus.

**What a net lists** is a separate question from what it joins:

- a net lists a pin **once per reference designator**, however many units of
  the symbol draw it, because a library may put a pin in unit 0 and every unit
  then draws it ([`notes/pin-shared-by-two-units.md`](notes/pin-shared-by-two-units.md));
- a net lists **no pin of a symbol marked `(on_board no)`**, and `(dnp yes)`
  and `(in_bom no)` do not remove a pin
  ([`notes/symbol-off-the-board.md`](notes/symbol-off-the-board.md));
- which unit a symbol draws is the **instance record's** business, not the
  cached `(unit …)` beside the `lib_id`
  ([`notes/instance-unit.md`](notes/instance-unit.md)).

**Validation** — `demos/complex_hierarchy/ampli_ht.kicad_sch`:

```
geometry alone (rules 1-3)         → 37 nets
every rule (with the name merges)  → 25 nets
kicad-cli sch export netlist       → 25 nets   ✓ exact match
```

Power nets found: `GND` (9 symbols), `HT` (3), `+12V` (2), `-VAA` (2) — i.e. the
12-net discrepancy is exactly the 16 power symbols collapsing into 4 nets.

This comparison is now a permanent test: for every fixture, kicli's net
partition must equal `kicad-cli`'s (`kicad-cli.md` §4). Against KiCad's whole
demo corpus, 32 of 35 hierarchies match exactly; the three that do not are the
open half of rule 6.

### 3.3 Naming

Priority for the displayed net name: power-symbol value → user label (global >
hierarchical > local) → synthetic `n<k>`.

Synthetic names are assigned by descending pin count, then by the sorted pin
list, so they are **deterministic given the design** and stable under unrelated
edits — unlike KiCad's `Net-(R1-Pad1)` scheme (Contradiction 3).

### 3.4 Handles

Every record can be addressed without coordinates:

| Object | Handle |
|---|---|
| symbol | refdes, sheet-path-qualified (`/Power/C12`) |
| pin | `C12.2` |
| net | display name, or `#n7` for synthetics |
| any object | its UUID (always accepted, never printed by default) |

`--uuids` adds a trailing `@<uuid8>` to every record for agents that need to
address objects that have no refdes (wires, junctions). Measured cost: +9 bytes
per record, roughly +25 % on the connectivity view — hence opt-in, per
Constitution §6.

---

## 4. View 2 — layout digest

### 4.1 Grammar

```
page <paper> <w>x<h>mm  used=<x0>,<y0>..<x1>,<y1>
L <ref> <x> <y> <rot> <mirror|-> [<w>x<h>]     ; symbol placement, mm@0.01
T <kind> <text> <x> <y> [<rot>]                ; labels and free text
F <ref>.<field> <dx> <dy> [<rot>]              ; field offsets, only when non-default
W <n> segments, <j> junctions, <c> crossings   ; summary, not per-wire
B <x0>,<y0>..<x1>,<y1> <density>               ; occupancy grid (optional, --dense)
```

Design choices, and why:

- **Symbols carry `rot` and `mirror` verbatim** (0/90/180/270 and `x`/`y`/`-`)
  rather than a matrix — that is what the agent will pass back to
  `kicli sym rotate`.
- **Wires are summarised, not enumerated.** 517 wire segments on a 234-symbol
  sheet is 20 KB of noise; the agent needs routing *quality*, which is the
  crossing/dogleg counts from R8, plus the ability to render (R11) when it
  really needs to look. `--wires` enumerates them when asked.
- **Field positions are emitted as offsets from the symbol anchor, and only when
  they differ from the library default.** On a tidy sheet that is a handful of
  lines; on a messy one it is exactly the list of things to fix — which makes the
  view diagnostic, not just descriptive.

### 4.2 Measured size

| Sheet | symbols | layout digest |
|---|---|---|
| `ampli_ht` | 46 | 1,354 B |
| `in_out_conn` | 130 | 6,139 B |
| `csi` | 125 | 7,076 B |
| `One-Air-Max` | 234 | 8,649 B |

Same order as connectivity, so an agent can afford both for one sheet
(≈16 KB ≈ 4–5 k tokens for the largest sheet measured).

---

## 5. View 3 — delta

### 5.1 Snapshot model

A snapshot is a map `uuid → content-hash`, plus a small header:

```
snapshot <name> <sheet-path> <iso8601> kicli/<version>
<uuid8> <kind> <hash16>
…
```

`content-hash` = BLAKE3 (or SHA-256, Q3) over a *canonical semantic encoding* of
the object: its kind, its own fields in a fixed order, coordinates as integer
IU, strings as UTF-8 — explicitly **not** its file bytes and **not** including
its position in the file. This makes the delta immune to KiCad's item reordering
(`sch-format.md` §1.1).

Two hashes per object are worth keeping:

- `h_geom` — position/orientation/size only
- `h_data` — everything else (fields, names, values)

so the delta can say "moved" vs "edited" without a second pass.

### 5.2 Delta output

```
delta <from-snapshot> -> <current>
+ S R42 10k Device:R                    ; added
- S R7 1k Device:R                      ; removed
~ L C12  moved  (120.65,88.90) -> (127.00,88.90)
~ F R3.Reference  moved  (2.54,-1.27) -> (-2.54,1.27)
~ S U1.Value  "STM32F103" -> "STM32F103C8T6"
~ N +3V3  pins +C9.1 -C4.2
= 231 objects unchanged
```

Rules:

- One line per changed object; a final `=` count for the unchanged remainder, so
  the agent knows the denominator without listing it.
- Net changes are reported as pin-set deltas, not as "net replaced", because
  that is the electrically meaningful statement.
- The delta is **ordered** by kind then handle, so re-running gives byte-identical
  output for the same pair of states (Constitution §4).

### 5.3 Where snapshots live

`.kicli/snapshots/<name>.snap` in the project directory, plus an implicit
`@last-write` snapshot updated on every kicli mutation. That gives the agent a
free "what did my last command actually change?" without any bookkeeping — which
is the single most useful delta in practice, and it directly serves Constitution
§5 (every mutation is verified and reported).

`.kicli/` must be gitignored by default; snapshots are caches, not artefacts.

---

## 6. Measurements

### 6.1 Method

```sh
# prototype: parse → resolve pins → union-find nets → emit views
python3 exp/r10/views.py <sheet.kicad_sch>
```

The prototype implements `sch-format.md` §3 parsing, `geometry.md` §3.1 pin
resolution, and §3.2's merge rules. Sizes are of the emitted UTF-8 text.

### 6.2 Results

| Sheet | symbols/nets | raw | connectivity | layout | JSON (min) | JSON (pretty) |
|---|---|---|---|---|---|---|
| `ampli_ht` | 46 / 25 | 112,996 | 1,463 | 1,354 | 3,362 | 4,971 |
| `in_out_conn` | 130 / 212 | 359,095 | 6,933 | 6,139 | — | — |
| `csi` | 125 / 177 | 702,137 | 6,215 | 7,076 | — | — |
| `One-Air-Max` | 234 / 216 | 624,834 | 7,896 | 8,649 | 14,863 | 21,784 |

JSON overhead: **2.3× minified, 3.4× pretty-printed**, for identical content.

### 6.3 Token budget

**Caveat: no tokenizer was available in this environment** (`tiktoken` is not
installed), so token counts are *estimates* from byte counts. For terse,
identifier-dense ASCII of this shape, 3.0–4.0 bytes/token is the usual range;
the table uses 3.5 with the range shown.

| View | Bytes | Estimated tokens |
|---|---|---|
| connectivity, median sheet (37 sym) | ~1.5 KB | **370–500** |
| connectivity, large sheet (234 sym) | 7.9 KB | **2.0–2.6 k** |
| layout, large sheet | 8.6 KB | **2.2–2.9 k** |
| both views, large sheet | 16.5 KB | **4.1–5.5 k** |
| raw `.kicad_sch`, large sheet | 625 KB | ~160–210 k (i.e. unusable) |

**The brief's target — "full connectivity view of a typical sheet in low
thousands of tokens" — is met with a wide margin**: a typical sheet is ~400
tokens, and the largest sheet in KiCad's entire demo corpus is ~2.5 k.

Before this is treated as final, run the same measurement with the real
tokenizer (`kicli view --stats` can print it). Q4.

---

## 7. Interaction with the rest of the system

- **R8 scoring** consumes the same in-memory model; findings reference the same
  handles, so an agent can read a finding and act on it without a second query.
- **R9 routing** returns route + cost in the same coordinate convention as the
  layout digest (mm@0.01).
- **R11 rendering** takes a region in the same coordinates, so
  `sch view --view layout` → pick a bbox → `sch render --region` composes
  without translation.
- **Mutations** echo a delta fragment (§5.2) for exactly the objects they
  touched, satisfying Constitution §5's "every mutation is verified and
  reported" with no extra vocabulary.

---

## 8. Open questions for James

- **Q1 — View scope default.** Should `kicli sch view` default to the whole
  project (sections per sheet) or to the current sheet with ports marked?
  Recommendation: whole project when it fits a configurable budget
  (`view.max_bytes`, default 32 KB), otherwise an index plus per-sheet
  summaries, and say which happened.

- **Q2 — Power symbols in the symbol list.** Currently suppressed (they appear
  only as net names). Confirm — or do you want them listed for placement work?
  (`--include-power` would cover it.)

- **Q3 — Hash function for snapshots.** BLAKE3 (fast, extra dependency) or
  SHA-256 (in `sha2`, ubiquitous)? Snapshot cost is trivial either way;
  recommendation is SHA-256 truncated to 16 hex chars for fewer dependencies.

- **Q4 — Token accounting.** Do you want `kicli sch view --stats` to report a
  real token count (needs a tokenizer dependency — `tiktoken-rs` is MIT but adds
  a vocab file), or is a byte count plus a documented ratio enough?
  Recommendation: bytes only; agents can count their own tokens.

- **Q5 — Net-name stability.** Confirm the synthetic `n<k>` scheme (§3.3) rather
  than mirroring KiCad's `Net-(R1-Pad1)` names, accepting that kicli's net names
  will not match those shown in the KiCad GUI for unnamed nets. (Rules R8
  KI-LBL-002 nudges the design toward naming them anyway.)

---

## 9. Reproduction

| Artefact | How |
|---|---|
| corpus stats (§1) | count `\n\t(symbol\n` etc. over the recipe-C corpus |
| connectivity extractor | `exp/r10/extract.py` — parser + transform + union-find |
| view sizes (§6.2) | `python3 exp/r10/views.py <sheets…>` |
| net-count validation (§3.2) | `kicad-cli sch export netlist -o ah.net ampli_ht.kicad_sch` then count `(net (code …)` entries |

The prototype scripts are research artefacts, not shipping code; they should be
re-derived in Rust against the same assertions.
