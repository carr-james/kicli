# kicli — Specification (v1)

Status: **research-complete**. Every section formerly pending research is
resolved from the Phase 1 dossier in `research/`, and all 21 contradictions in
`research/SUMMARY.md` are applied with James's rulings in `DECISIONS-R6.md`.
Governed by `CONSTITUTION.md`. §20 records each contradiction and its
resolution.

Citations of the form `(doc.md §n)` point at `research/doc.md`; those documents
carry the source links and reproduction recipes. Where this spec and a research
doc disagree, this spec wins and the divergence is recorded in §20.

## 1. Purpose

kicli is a CLI tool that gives LLM agents eyes and hands in KiCad 10.0 projects:
reading and mutating schematics at full human parity, producing readable AND
electrically correct designs, with a deterministic style score as the quality
signal, plus a small set of parametric PCB setup operations. Routing and final
PCB layout remain human work.

## 2. Users and environment

- Primary: LLM agents (Claude Code and similar) shelling out to kicli.
- Secondary: James, using it directly and reviewing agent output.
- Platforms: Linux (Arch) and macOS. Rust, single binary, `cargo install kicli`
  or GitHub release binaries. No CI initially; local `cargo test` + fixtures.
- Licence: **GPL-3.0-or-later** (final, 2026-08-13). Dependencies must be
  GPL-3-compatible, which MIT, Apache-2.0, BSD, ISC, Zlib and **MPL-2.0** all
  are; AGPL is excluded (Constitution §9). Two consequences run through this
  spec. **KiCad's own GPL source, protos, fonts and demo files may be read and
  derived from freely** — the `Prettify` port (§5.3), the pin and text maths
  (§8) and the IPC protos (§13) are all clean under this licence. And Konnect
  stays black-box: AGPL is still excluded, and its source is still never read.
  Q8 and Q29 are dissolved by this decision (`DECISIONS-R6.md`).

## 3. Decisions record (from elicitation, 2026-08)

| # | Decision |
|---|----------|
| D1 | Target KiCad 10.0 file formats and IPC API. Concrete floor: **10.0.5, schematic format stamp `20260306`** (§5) |
| D2 | CLI-first; MCP wrapper possible later, out of v1 |
| D3 | Full manipulation parity incl. field text (§4 states the differentiation precisely) |
| D4 | Eyes = structured views + renders (full sheet and region); agent looks with its own vision — kicli never calls models |
| D5 | Rust |
| D6 | Edit-existing and create-from-scratch equally weighted in v1 |
| D7 | Context-efficient representations are a core requirement, not a nicety |
| D8 | Custom libraries day 1; shared parts library = git submodule at a **sibling `../shared` path with the project's existing nickname** (§10); vendoring in (project-local) and up (into shared lib); symbol creation out of v1 |
| D9 | Full hierarchical sheet support |
| D10 | Verification: cheap invariants auto per mutation; ERC + score explicit |
| D11 | Scoring: deterministic Tier 1 + Tier 2 rules only, per rule catalogue (`style-rules.md` §4 is the canonical catalogue) |
| D12 | Wire drawing: deterministic auto-route default with cost report; explicit vertices on demand |
| D13 | Addressing: UUIDs + refdes/net/sheet-path handles; spatial queries deferred |
| D14 | No tool-level undo in v1 (git suffices) |
| D15 | Per-project `kicli.toml` config |
| D16 | SPICE authoring/simulation deferred (post-v1) |
| D17 | Name: kicli; kubectl/docker-style `noun verb` CLI |
| D18 | Fixtures purpose-built, in-repo; GPL corpora are external (§18) |
| D19 | Agent docs are a first-class deliverable |
| D20 | PCB parametric ops in v1, sequenced last: edge cuts, fiducials, CNC flip-registration holes, coarse footprint placement |

## 4. Prior art, and what kicli is for

Four adjacent projects were installed and tested (`ecosystem.md`). This section
exists because two of them are actively maintained and agent-focused, and an
honest positioning is a design constraint, not marketing.

| Project | What it does well | Why kicli exists anyway |
|---|---|---|
| `kiutils` 1.4.8 (MIT, Python) | pleasant typed API, fine for read-only extraction | **loses 2,912 of 19,851 tokens (14.7 %) on one KiCad 10.0.5 sheet** — 166 `hide`, 295 `do_not_autoplace`, 46 `body_style`, 59 `exclude_from_sim` — and KiCad still opens the result, so the loss is silent. Structural: a typed AST has nowhere to put tokens it does not know (`ecosystem.md` §2.3) |
| `kicad-skip` 0.2.5 (Python) | **token-lossless** (19,851 → 19,851); best REPL ergonomics surveyed | no prettifier, so every write reformats the whole file; no LICENSE at HEAD; leaks diagnostics to stdout (`ecosystem.md` §2.4, §4.2) |
| `kicad-tools` 0.20.0 (MIT, Python) | explicitly agent-focused, JSON-out CLI; **`kct sch tidy` already repositions Reference/Value fields** | tidy is deterministic autoplacement of two fields, skips hidden fields and power symbols, and approximates multi-unit body boxes by pin extents (`ecosystem.md` §4.3) |
| `circuit-synth` (MIT, Python) | generates KiCad projects from a Python DSL | owns the source of truth; kicli's premise is that the `.kicad_sch` is the source of truth and humans keep editing it in the GUI |
| Konnect (AGPL) | breadth: 187 tools over schematic, PCB, library, fab | `design_review` is "AI-powered"; kicli's score is geometry and arithmetic, offline and diffable (Constitution §4). Black-box observation only; **no source read** (`ecosystem.md` §5) |

**The differentiation statement** (C17, `ecosystem.md` ⚠1): it is *not* "nobody
can move field text". It is:

1. **Byte-identical round-trip** where the alternatives are 14.7 % lossy or
   whitespace-lossy (§5).
2. **Full manipulation parity** — arbitrary fields on arbitrary objects
   (symbols, sheets, global labels, netclass flags, table cells), with explicit
   positions, rotation, justification and visibility — not autoplacement of two
   named fields.
3. **A deterministic readability score** that never calls a model or a network
   service, so it is reproducible and diffable across runs.
4. Single binary, no Python environment, no running KiCad for schematic work.

**Licensing, stated plainly** (§2): the Python alternatives are MIT and kicli is
GPL-3.0-or-later. Someone who must embed this code in a closed product should
use `kicad-tools`; someone who wants a tool that tracks KiCad's own behaviour
closely benefits from kicli sharing KiCad's licence, because kicli can port
KiCad's own algorithms rather than reverse-engineer them. Konnect is AGPL and is
still observed only as a black box.

`AGENT.md` (§16) must cite `kicad-tools` as the recommendation for users who
want Python and breadth, and must state kicli's licence.

## 5. File-format target and round-trip properties

### 5.1 Version policy (C2, `sch-format.md` ⚠3, §5)

- `(version …)` is a date stamp, not a semver. **kicli's known maximum is
  `20260306` (KiCad 10.0.5)**; symbol libraries `20251024`.
- kicli parses **any** version stamp and applies version-gated semantics:
  `~`-means-empty for pin name/number below `20250318`, and `hide` inside
  `(effects …)` below `20251028` (`sch-format.md` §5.1, §5.3). Getting this
  wrong silently renames pins across a version boundary.
- kicli **writes back the stamp it read**. It never upgrades a file, and it
  **never shells out to `kicad-cli sch upgrade`** in a path that touches user
  files — that command silently destroys bus aliases, which moved to
  `.kicad_pro` in v10 (`sch-format.md` §5.6).
- kicli **refuses to write** a file whose stamp exceeds its known maximum
  (Q2). Override: `formats.max_schematic_version` in `kicli.toml`.

### 5.2 Units (C3, `sch-format.md` ⚠4, §2.3)

Coordinates are `int32` internal units of **100 nm** (`SCH_IU_PER_MM = 1e4`).
kicli's geometry is integer IU end to end; millimetres are a presentation unit
at the CLI boundary only. 50 mil = 1.27 mm = **12700 IU**. Numbers are emitted
by formatting the integer as fixed-point with 4 fractional digits and stripping
trailing zeros and any trailing `.`. This is identical to KiCad's `{:.10g}`
output for every one of the 4,294,967,296 `int32` inputs — checked exhaustively,
not argued — with no float formatting anywhere.

### 5.3 The two round-trip properties (C1, Q1)

Refines M1's "lossless CST": kicli uses a **token-preserving syntax tree plus an
exact re-emitter**, not a whitespace CST. KiCad emits a flat token stream and
then runs `KICAD_FORMAT::Prettify` over the whole buffer, so there is no
whitespace to preserve; a port of that function reproduces **115/115** canonical
demo schematics, **19/19** `.kicad_pcb`, and the shipped `Device.kicad_sym`
exactly from a whitespace-stripped token stream. Boards only reproduce once
`pcb upgrade` has run over them: one demo board still carried
`(generator_version "9.0")` and packed sibling lists as `)(`, which KiCad 9 wrote
and KiCad 10 does not.

| Id | Property | Status |
|---|---|---|
| **P1** | `emit(parse(f)) == f` bytewise, for KiCad-authored (canonical) input | **merge gate** |
| **P2** | `parse(emit(parse(f))) ≡ parse(f)` structurally, for *every* input | **merge gate** |
| P3 | an edit changes only its own region: `changed_lines ≤ K` (K ≤ 3 for a field-value change) | merge gate per mutation command (M3+) |
| P4 | `emit(parse(f))` equals what KiCad itself would write — requires adopting KiCad's item sort | **informational only**, never gating |

P4 is separate because **KiCad reorders every item on save**, by
`(type enum, uuid)` (`sch-format.md` §1.1). "Byte-identical to the input" and
"byte-identical to what KiCad would write" are therefore different properties.
Item reordering is available as an explicit `kicli sch normalize`, never applied
silently.

### 5.4 Non-canonical input, comments, embedded data (Q3, Q4, Q6)

| Input | Behaviour |
|---|---|
| Canonical in some prettifier mode | normal path; output byte-identical except edited subtrees |
| Non-canonical, no comments | edit and emit canonically, reporting `"reformatted": true` with the reason in structured output |
| Contains `#` line comments | **refuse to write** unless `--allow-comment-loss` (KiCad's lexer accepts them and drops them on save) |
| Stamp newer than known maximum | refuse to write (§5.1) |
| Embedded files/fonts (`embedded_files`, base64 blobs) | **opaque bytes, never re-encoded**; any operation that would move them between files is refused. `embedded_fonts` absence is not corruption — v10 writes it only for the top-level sheet (`sch-format.md` §5.5) |

**Prettifier mode is detected and preserved**, not assumed (Q4). The three modes
are `NORMAL` (schematics, boards, symbol libs), `COMPACT_TEXT_PROPERTIES`
(clipboard, and *all* saves when the advanced config `CompactSave` is true), and
`LIBRARY_TABLE` (`sym-lib-table`, `fp-lib-table`). Detection: the mode in which
the input is a fixed point (`sexpr-strategy.md` §2.4). Editing one field in a
`CompactSave` user's schematic must not reformat their whole file.

### 5.5 The project file is read-only in v1

`.kicad_pro` is JSON, not an s-expression, so §5.3's properties do not carry
over to it. kicli **reads** it — bus aliases moved there in KiCad 10, and ERC
severities live there (§14.3) — and **does not write it** in v1. Nothing in v1
needs to: bus aliases are read-only through M8, and ERC severities are
relabel-only.

Measured over 37 project files (one fixture plus KiCad's demos): **34 round-trip
byte-identically** through an order-preserving JSON reader. Key order and
indentation always match. The other 3 differ in exactly one way — KiCad prints
the full decimal expansion of a double where a shortest-round-trip printer emits
fewer digits:

```
kicad: "board_outline_line_width": 0.049999999999999996,
other: "board_outline_line_width": 0.05,
```

**The values are identical doubles. Only the text differs.**

When a write path is first needed, it is built the same way the s-expression
side is: **preserve the source text of every value kicli did not modify**, and
print the values kicli does write with a formatter that matches KiCad's. Byte
identity then falls out, rather than being traded away. kicli does not adopt
"reformat and flag it": a file that opens cleanly with one number reading
differently is the failure this tool exists to prevent.

### 5.6 Reference designators live in `instances` (`sch-format.md` §3.6)

`(property "Reference" …)` on a symbol is the cached value for the currently
loaded sheet path; the truth is `instances → project → path → reference`, and a
symbol on a twice-instantiated sheet has two references. `sym set-field
Reference` is therefore **sheet-path aware from M3**, not an M8 refinement, and
must update both the cached property and the matching `instances → path` entry.

Addressing model (D13):

```
KIID_PATH = "/" + rootScreenUuid + ("/" + sheetItemUuid)*
handle    = (sheetPath, itemUuid)
```

`(lib_name "…")` redirects the `lib_symbols` cache key away from `lib_id`; code
that resolves embedded symbols by `lib_id` alone is wrong (`sch-format.md` §3.4).

### 5.7 Scope confirmations (Q7)

- **Schematic variants** (`20250922`, `20260306`): round-trip preservation only,
  no variant-aware editing in v1. `sch view` must not present a
  variant-overridden field value as the only value. `--variant` is **hidden** in
  v1 on all commands (Q33).
- **Flat multi-top-level hierarchies** (`20251012`): readable; `--sheet`
  addressing assumes a single root and errors clearly otherwise.

## 6. Command surface and exit codes

Global flags: `--output json|text` (text default), `--project <dir>`,
`--sheet <path>`, `--quiet`, `--version`.

```
kicli project info|check                    # project summary, health check
kicli sch view                              # compact views: --view connectivity|layout|delta
kicli sch render                            # SVG+PNG, --region, --annotate
kicli sch erc                               # kicad-cli wrapper, JSON findings
kicli sch score                             # lint + weighted score, JSON findings, --gate
kicli sch normalize                         # apply KiCad's item sort order (explicit only)
kicli sym place|move|rotate|mirror|delete|set-field
kicli field move|rotate|justify|show|hide   # field text as first-class objects
kicli text add|move|edit|delete             # free text and text boxes
kicli wire connect|draw|delete              # connect = auto-route; draw = explicit vertices
kicli label add|move|delete                 # net / global / hierarchical labels
kicli junction|noconnect add|delete
kicli net list|show|rename
kicli sheet list|add|pin|instance           # hierarchy operations
kicli parts search|show                     # the shared-library catalogue
kicli lib vendor                            # --into project|shared
kicli pcb outline|fiducial|reghole|place    # parametric ops via IPC (phase last)
```

`wire connect` takes **`<pin> <pin>` or `<pin> <net>`** (C13,
`wire-routing.md` ⚠2) — "connect U1.7 to +3V3" is the common agent request and
is first-class, not a composition.

`field …` accepts fields on every object that owns them, not just symbols:
symbol properties, sheet `Sheetname`/`Sheetfile`, global-label
`Intersheetrefs`, netclass-flag `Netclass`/`Component Class`, and table cells
(`sch-format.md` §3.3, §3.5, §3.10). This is the parity requirement in D3.

### 6.1 kicli's exit codes (C4, `kicad-cli.md` ⚠1)

| Code | Meaning |
|---|---|
| 0 | success (findings are data, not failure: `sch erc`/`sch score` exit 0 when they report findings without `--gate`) |
| 1 | operation error — well-formed request that could not be completed |
| 2 | usage error — bad flags or arguments |
| 3 | **verification failure** — mutation rolled back, **no file written** |
| 4 | file / parse error, or a refusal to write (version ceiling, comment loss) |
| 5 | gate failure — `sch score --gate` found Tier 1 findings or ERC errors |
| 6 | required external tool unavailable (`kicad-cli` missing or wrong major version; KiCad IPC not reachable for `pcb`) |

### 6.2 Translation of `kicad-cli`'s codes — never pass-through

`kicad-cli` uses `0 OK, 1 ERR_ARGS, 2 ERR_UNKNOWN, 3 ERR_INVALID_INPUT_FILE,
5 ERR_RC_VIOLATIONS, 6 ERR_JOBS_RUN_FAILED` (measured, and matching
`include/cli/exit_codes.h` at tag 10.0.5 — `kicad-cli.md` §3.3). These collide
with kicli's meanings. **kicli translates every one; raw pass-through is
forbidden**, and the table below is documented in `AGENT.md`.

| `kicad-cli` | kicli | Note |
|---|---|---|
| 0 | continue | — |
| 1 `ERR_ARGS` | 1 | kicli built a bad command line; this is a kicli bug and is reported with the invocation |
| 2 `ERR_UNKNOWN` | 1 | |
| 3 `ERR_INVALID_INPUT_FILE` | 4 | |
| 5 `ERR_RC_VIOLATIONS` | 1 | should never occur: kicli **never passes `--exit-code-violations`**; it parses the report instead |
| 6 `ERR_JOBS_RUN_FAILED` | 1 | |
| binary absent / not executable | 6 | structured error naming the binary and an install hint |
| major version ≠ 10 | 6 | |

## 7. Representations (resolves R10)

Three views. Terse text is the default; JSON is the twin and costs **2.3×
minified / 3.4× pretty** for identical content, so it is opt-in per
Constitution §6 (`representation.md` §6.2). Every line starts with a one-letter
record type so an agent can filter without a parser. Ordering is stable:
symbols by natural-sorted refdes, nets by descending pin count then name.

**Scope** (C11, Q11): `sch view` defaults to the **whole project** when it fits
`view.max_bytes` (default 32 KB); otherwise it emits an index plus per-sheet
summaries. The output always states which mode was used. `--sheet <path>`
restricts to one sheet with its ports marked. Connectivity is a hierarchy-level
property, so a per-sheet view marks nets that leave through sheet pins.

Power symbols are **suppressed** from symbol listings (they are net-name
carriers; 16 `#PWR0xx` lines are noise). `--include-power` shows them (Q13).
`view --stats` reports **bytes only** — no tokenizer dependency (Q13).

### 7.1 View 1 — connectivity

```
sheet <path>  sym=<n> pwr=<n> nets=<n>
# S ref value lib
S <ref> <value> <libname>                    ; one per non-power symbol
# N name[=kicad-name]: pins
N <name>[=<kicad-name>]: <ref>.<pin> …       ; one per net, pins sorted
H <subsheet>: <port>(<i|o|b|t|p>) …          ; hierarchy ports of child sheets
P <port>(<dir>)                              ; this sheet's own hierarchical labels
```

`--uuids` appends `@<uuid8>` to every record (+~25 % size) for objects with no
refdes; opt-in.

**Net construction** is not pure geometry (C-note, `representation.md` ⚠4).
Union-find over, in order: (1) coincident wire/bus endpoints; (2) a **junction**
on a segment *interior* merges what meets there — a pin or sheet pin that merely
lies on a segment interior does **not** merge, and two wires crossing without a
junction do **not** merge either; (3) pin at a wire endpoint; (4) labels — local
labels merge within a sheet, global labels project-wide, hierarchical labels
with the parent's like-named sheet pin; (5) **power symbols** — pins whose
symbol `Value` matches merge project-wide. Rules 1–3 alone give 37 nets on
`ampli_ht` where KiCad reports 25; rules 1–5 give exactly 25
(`representation.md` §3.2).

Rule (2) was corrected on 2026-08-13, having previously said that a pin on a
segment interior merges. It does not. Two clusters of one M2 fixture differ only
by a junction: without it the mid-span pin is its own unconnected net, with it
the pin joins, and a control run adding a junction to the first cluster flips it
(`research/notes/pin-on-wire-interior.md`).

**Connectivity is defined as whatever KiCad 10.0.5's netlister does.** The rules
above describe that behaviour; they do not invent it. Where a rule and KiCad
disagree, KiCad is right by definition and the rule is a bug, whatever any
document here says. This is not a licence to guess: the disagreement must be
demonstrated against `kicad-cli` on a committed fixture, and the rule text and
its evidence updated together.

**Test gate**, which is how the paragraph above is enforced rather than merely
stated: for every fixture, kicli's net partition must equal
`kicad-cli sch export netlist`'s. This is an M2 gate (`kicad-cli.md` §4).

**Naming** (C12, Q12 as amended). Display name priority: power-symbol value →
user label (global > hierarchical > local) → synthetic `#n<k>`. Synthetics are
assigned by descending pin count then sorted pin list, so they are deterministic
given the design and stable under unrelated edits — unlike KiCad's
`Net-(R1-Pad1)`, which changes when an unrelated symbol is renumbered.

**Amendment (Q12):** every net record also carries **KiCad's current net name**
as `=<kicad-name>`, emitted whenever it differs from the display name, and as a
`kicad_name` field in JSON. Agents need it to correlate kicli findings with ERC
output and with what the GUI shows. The synthetic id remains the handle; the
KiCad name is an attribute and must never be used as an identifier.

### 7.2 View 2 — layout digest

```
page <paper> <w>x<h>mm  used=<x0>,<y0>..<x1>,<y1>
L <ref> <x> <y> <rot> <mirror|-> [<w>x<h>]   ; symbol placement, mm@0.01
T <kind> <text> <x> <y> [<rot>]              ; labels and free text
F <ref>.<field> <dx> <dy> [<rot>]            ; field offsets, only when non-default
W <n> segments, <j> junctions, <c> crossings ; summary; --wires enumerates
B <x0>,<y0>..<x1>,<y1> <density>             ; occupancy grid (--dense)
```

Symbols carry `rot` and `mirror` verbatim (0/90/180/270 and `x`/`y`/`-`) because
that is what the agent passes back to `kicli sym rotate`. Field positions are
offsets from the symbol anchor emitted **only when they differ from the library
default**, which makes the view diagnostic rather than merely descriptive.

### 7.3 View 3 — delta (C10, `representation.md` ⚠1)

A snapshot is `uuid → content-hash`, **hashed per object, never over file
bytes** — KiCad reorders items on save, so a file hash makes every save look
like a total rewrite.

```
snapshot <name> <sheet-path> <iso8601> kicli/<version>
<uuid8> <kind> <h_geom16> <h_data16>
```

`content-hash` is **SHA-256 truncated to 16 hex chars** (Q13) over a canonical
semantic encoding: kind, own fields in fixed order, coordinates as integer IU,
strings as UTF-8 — explicitly excluding file position. Two hashes per object:
`h_geom` (position/orientation/size) and `h_data` (everything else), so the
delta distinguishes "moved" from "edited" in one pass.

```
delta <from-snapshot> -> <current>
+ S R42 10k Device:R
- S R7 1k Device:R
~ L C12  moved  (120.65,88.90) -> (127.00,88.90)
~ F R3.Reference  moved  (2.54,-1.27) -> (-2.54,1.27)
~ S U1.Value  "STM32F103" -> "STM32F103C8T6"
~ N +3V3  pins +C9.1 -C4.2
= 231 objects unchanged
```

Net changes are reported as pin-set deltas, not "net replaced". Output is
ordered by kind then handle, so the same pair of states always produces
byte-identical output (Constitution §4). Snapshots live in
`.kicli/snapshots/<name>.snap`, with an implicit `@last-write` updated on every
mutation; `.kicli/` is gitignored by default. Every mutation echoes a delta
fragment for exactly the objects it touched, satisfying Constitution §5 with no
extra vocabulary.

### 7.4 Budgets (`representation.md` §6)

Measured on real sheets: connectivity is **1,463 B** for a 46-symbol sheet and
**7,896 B** for the largest sheet in KiCad's entire demo corpus (234 symbols) —
50–113× smaller than the source file. Layout digest is the same order. Token
figures are byte-derived estimates (~3.5 B/token): ≈400 tokens for a median
sheet, ≈2.5 k for the largest. Budget targets for regression:

| View | Sheet | Ceiling |
|---|---|---|
| connectivity | median (37 sym) | 2 KB |
| connectivity | 234 symbols | 9 KB |
| layout | 234 symbols | 10 KB |
| both | 234 symbols | 18 KB |

A view that exceeds its ceiling is a bug (Constitution §6).

## 8. Mutation semantics (resolves R7 for geometry)

Atomic writes: temp file → verify → rename. Auto-invariants per Constitution §5
(re-parse, UUID references intact, connectable geometry on grid, no orphaned
instance data). Failure ⇒ exit 3 and **no file written**.

**Grid** (C3): snap is exact integer arithmetic on IU — round-half-away-from-zero
to the nearest multiple of `grid` (default 12700 IU). Off-grid detection is
`pos.x % 12700 != 0`. `--off-grid` overrides and is itself a lint finding, so
the agent feels it.

**Grid scope (Q9):** the blocking off-grid rule applies to **connectable
geometry only** — pins, wire and bus endpoints, junctions, no-connects, label
anchors, sheet pins. **Field and graphic text are exempt**: KiCad's own
autoplacement lands fields on arbitrary IU (e.g. `246.7512` in the corpus), so a
blanket rule would fail KiCad's own output (`geometry.md` ⚠2).

**Pin geometry** — verified 16/16 against KiCad's own ERC output
(`geometry.md` §3.4):

```
abs_pin = symbol.at + M · (lib_pin.x, −lib_pin.y)
```

`M` is the 2×2 integer matrix built from `(at … rot)` then composed with
`(mirror x|y)` in file order; the group has exactly 8 elements
(`geometry.md` §2.3). Note KiCad's row-major member naming `(x1,y1),(x2,y2)` —
transposing it silently swaps the 90° and 270° cases, the classic third-party
bug. Library coordinates are Y-up and are negated at parse time and again on
write.

**Fields on symbol move/rotate (Q14):** fields move **rigidly** with the symbol,
keeping their own angles; rotation carries their positions about the symbol
anchor. `--keep-field-positions` opts out. kicli **always clears
`fields_autoplaced`** when it sets a field position explicitly, or KiCad will
overwrite the work on next open (`geometry.md` §4.1).

**Text metrics (Q10, amended after the relicensing):** kicli **ports KiCad's own
measurement logic** — the stroke-font advance loop, the `INTER_CHAR` term, and
`GetTextBox`'s assembly of them (`geometry.md` §5.1, §5.2). §2 makes that legal,
and a port is exact by construction where a fitted table is exact only where it
was sampled. Glyph advances are derived from KiCad's Newstroke data, which is
GPL-2.0-or-later; the derived table carries its origin and copyright notice, as
does every ported function.

**The SVG measurement stays, as the oracle rather than the source.** One
`kicad-cli sch export svg` run over a generated calibration sheet yields
`textLength` per item, computed by KiCad's own font engine, and the port must
reproduce it (`kicad-cli.md` §5.5). Empirical calibration — fitting the residual
and folding it into the table — is the **fallback** if the port and the
measurement disagree and the difference is a stable linear term.

The result is validated again against IPC `GetTextExtents` once the M9 client
exists (`ipc-api.md` §4.1). Text extents feed the box maths in `geometry.md` §5;
findings that depend on a non-stroke `face` font are marked `approximate` in the
output.

Two bounding boxes are modelled, as KiCad itself does: **body box** (graphics +
pins, no text) for overlap rules, and **full box** (∪ visible field boxes) for
text-collision rules (`geometry.md` §6).

## 9. Wiring (resolves R9)

`wire connect A B` runs a deterministic orthogonal router and returns the route
plus a cost breakdown so the agent can accept it or move a symbol instead.

**Algorithm** — shapes first, A* second (`wire-routing.md` §4). Candidates are
enumerated in the fixed order I, L(h-first), L(v-first), Z(vertical-mid, `m`
ascending), Z(horizontal-mid, `m` ascending), U(offset ascending); cheapest
wins. If no shape is feasible, A* runs over turn-aware states `(x, y, dir)` with
expansion order `+x, −x, +y, −y` and a priority queue ordered on the total order
`(f, g, x, y, dir)`. Grounded in measurement: **100 % of wire endpoints in the
demo corpus are on the 50 mil grid** and 99.5 % of segments are axis-aligned, so
a lattice router is exact, not approximate (`wire-routing.md` §2).

**Obstacles**: symbol body boxes, other symbols' pins (+1 G halo), junctions and
no-connects are hard blocks; a **collinear/overlapping wire of another net is a
hard block** (it would render as a connection); a perpendicular crossing is
allowed and costed; text boxes are soft-costed. A route must escape ≥ 1 G along
the pin's own direction at both ends — a hard constraint, not a cost.

**Cost model**, all integer `i64` (no floating point anywhere in the router):

| Term | Default | Rationale |
|---|---|---|
| `w_len` | 1 / grid step | base unit |
| `w_turn` | **6** | measured median segment is 5 grid steps, so a corner must cost more than a modest detour (Q15 accepted) |
| `w_cross` | 20 | crossings are the most visible defect |
| `w_text` | 12 / step | routing through a label is nearly as bad as a crossing |
| `w_near` | 2 / step | breathing room around symbols |
| `routing.margin` | **8 G** | routing window inflation (Q15 accepted) |

**Four-way junctions (Q16):** the router **never creates one**. It offsets by
1 G and reports the adjustment. A route terminating on a same-net wire's
interior emits a junction; terminating at an existing endpoint does not.

**Label fallback (C14):** when the best path exceeds
`routing.label_threshold` (default **30 G ≈ 381 mm**) or A* reports blocked, the
router **proposes** paired labels and does not act; `--auto-labels` performs it
and says so. This is **one knob shared with the linter's `KI-LBL-001`**
(`wire-routing.md` ⚠1) — a router that emits at 250 mm while the linter
penalises above 381 mm would argue with itself.

`status` ∈ `routed | labels | blocked | invalid`. `blocked` reports the blocking
objects' handles, never a bare failure.

**Determinism is a test, not an aspiration:** for every fixture and terminal
pair, routing 100 times and across a shuffled input item order (KiCad reorders
items) must yield byte-identical output.

**Calibration gate (Q17):** re-route every net of a known-good sheet from
scratch; assert total cost is within **15 %** of the original.

Out of scope for v1: rip-up-and-reroute, global net optimisation, bus routing,
cross-sheet routing.

## 10. Libraries and vendoring (resolves R4)

**Default layout (C15, Q25)** — matches the existing Eurorack repositories; the
old `libs/parts` default is **dropped** (`libraries-and-vendoring.md` §5):

```toml
[libraries]
shared_path    = "../shared"        # relative to ${KIPRJMOD}
shared_nick    = "Eurorack Common"
symbols_dir    = "symbols"
footprints_dir = "footprints"
models_dir     = "3dmodels"
```

i.e. one submodule per repository shared by several board projects, referenced
as `${KIPRJMOD}/../shared/symbols/<name>.kicad_sym`, which survives cloning and
works from any board directory.

**Resolution chain**: `lib_id` splits at the **first** `:`; nickname is looked up
in the project `sym-lib-table`, then the global one
(`~/Library/Preferences/kicad/10.0/` on macOS, `~/.config/kicad/10.0/` on
Linux); `${VAR}` expands against KiCad's versioned env vars
(`KICAD10_SYMBOL_DIR`, …) plus user vars from `kicad_common.json`. A project row
shadows a global row of the same nickname — that is the mechanism vendoring
uses.

**The embedded cache is what KiCad draws (C16, `libraries-and-vendoring.md`
⚠4).** `sym place` must copy the definition into the sheet's `lib_symbols` as a
mandatory step, and **vendoring must rewrite the embedded entry too** or the
schematic keeps rendering the old symbol.

`lib vendor` is one atomic transaction over up to seven changes
(`libraries-and-vendoring.md` §4.2): copy the `symbol` block; copy the
`.kicad_mod`; copy 3D models and rewrite `(model …)` paths; patch the
`sym-lib-table` row; patch the `fp-lib-table` row; rewrite `(lib_id …)` on every
affected symbol **in every sheet**; rewrite the embedded `lib_symbols` key and
any `(lib_name …)`. Plus the `Footprint` **field value** on each instance, which
embeds `libnick:FootprintName` — missing this is the most common vendoring bug,
because the schematic still looks right.

Pre-commit verification (also what `project check` runs standalone): every
`lib_id` resolves through the new tables; every `Footprint` field value resolves
through `fp-lib-table`; every `(model …)` path exists after expansion; the
embedded entry matches the target library modulo the name key; every written
file re-parses and round-trips.

Policies:

- **Vendor-up conflict (Q28):** `--into shared` that would overwrite a differing
  part is **refused with a diff summary**.
- **3D models (Q28):** copied for `--into project` (self-containment is the
  point), referenced for `--into shared`.
- **`${KICAD9_3DMODEL_DIR}` references (Q26): report only.** No
  `migrate-envvars` command in v1 — rewriting touches a submodule other projects
  share. `project check` reports them, because KiCad 10's generic path expander
  does not fall back across versions.
- **Nicknames containing spaces (Q27): warn only.** No rename tooling.
- Library tables in the wild are **not** in KiCad 10 canonical format (35 of 36
  in KiCad's own demos), so kicli's first write reformats the whole file. That
  must be **stated in the command output**, not discovered in a diff
  (`sexpr-strategy.md` §2.3).
- `type "Legacy"` / `Database` / `HTTP` rows round-trip but are never written by
  v1.

## 11. Scoring (resolves R8)

### 11.1 Layering, not duplication (C7, `style-rules.md` ⚠1)

KiCad 10's ERC already implements **47 checks**, including `four_way_junction`,
`endpoint_off_grid`, `similar_labels`, `label_dangling`,
`unconnected_wire_endpoint` and `duplicate_reference`. **kicli's lint engine
implements none of them.** It runs ERC, maps findings into its own format, and
adds only what ERC structurally cannot see: where things are *drawn*.

Two deliberate exceptions, both because KiCad's default severity is `IGNORE`, so
an untouched project would silently pass: `four_way_junction` → `KI-JCT-001`,
`single_global_label` → `KI-LBL-003`. kicli attributes them clearly and does not
double-count when the project has ERC's version enabled.

### 11.2 Gating (C8, Q19)

`sch score --gate` **may require `kicad-cli`**, because half of Tier 1 is
ERC-owned. Absence of `kicad-cli` is a structured error and exit 6 (§6.1, Q31).

### 11.3 Finding format

```json
{ "rule": "KI-FLOW-001", "tier": 2, "severity": "warning",
  "sheet": "/Power", "pos": {"x": 123.19, "y": 45.72},
  "objects": ["uuid…"], "message": "Power symbol +3V3 points down",
  "fix": "kicli sym rotate <uuid> --to 0", "penalty": 3.0 }
```

`fix` is a *suggested command*. **kicli never mutates during scoring.**

### 11.4 Rule catalogue

`research/style-rules.md` §4 is the canonical catalogue (Q18 — the pre-research
draft is retired; no reconciliation work). Tier 1 (blocking): `KI-GRID-001`
off-grid connectable geometry, `KI-OVL-001` symbol bodies overlap,
`KI-WIRE-001` wire crosses a symbol body, `KI-TXT-001` overlapping text,
`KI-CONN-001` a pin touches a wire's interior with no junction, so it looks
connected and is not (§7.1, `research/notes/pin-on-wire-interior.md`),
`KI-HIER-001` (delegated to ERC). Tier 2 (scored): `KI-FLOW-001/002`,
`KI-XING-001`, `KI-JCT-001`, `KI-RTE-001/002`, `KI-LBL-001/002/003`,
`KI-TXT-002/003`, `KI-FLD-001/002`, `KI-DOC-001…004`, `KI-LAY-001…003`,
`KI-DNP-001`, `KI-SYM-001`. Tier 3 is cut from scoring entirely.

A "significant net" (used by `KI-LBL-001/002`, `KI-RTE-001/002`) has ≥ 3 pins,
or a bounding-box diagonal ≥ 20 G, or a user-authored label, or is a power net
(`style-rules.md` §3.3).

**Power-direction name lists (Q21):** defaults cover standard Eurorack —
positive `{+12V, +5V, +3V3, …}`, ground/negative `{GND, -12V, AGND, DGND, VSS,
VEE, GNDA, GNDD, 0V, EARTH}`, plus "value starts with `-`" as negative.
Per-project override via `kicli.toml`.

### 11.5 Score formula, with density normalisation (C9, Q20)

```
raw_penalty(sheet) = Σ_rules  w_r · n_r · norm_r
score(sheet)       = round( 100 · exp( −raw_penalty / K ) ),   K = 25
```

Normalisers — a sheet with 4 symbols and one crossing is worse than a sheet with
200 symbols and one crossing, so absolute counts will not survive calibration:

| Normaliser | Applies to | Definition |
|---|---|---|
| `per_object` | field / symbol / text rules | `1 / max(1, N_sym/20)` — 20 non-power symbols is the reference sheet |
| `per_wire` | crossings, doglegs | `1 / max(1, N_wire/10)` |
| `per_sheet` | flow, layout, docs | `1` |

`N_sym` excludes power symbols. Project score = symbol-count-weighted mean of
sheet scores.

**Tier 1 findings do not reduce the score.** They set `"gate": "fail"`
independently. A schematic can score 96 and still fail the gate; that is
intentional and must be visible in the output.

Determinism (Constitution §4): detection is integer geometry only; floating
point appears only in the final `exp` with fixed rounding; findings are sorted
by `(rule, sheet, x, y, uuid)` before output; re-scoring an unchanged file is
bit-identical.

### 11.6 Calibration

Set A: 8–12 known-good external sheets (fetched, not vendored). Set B:
**programmatic degradations of set A** generated in-repo — rotate power symbols,
scatter fields, replace labels with long wires, break alignment, oversize the
page. Set C: agent output once M3 exists. Assertions: (1) monotonicity —
`score(A_i) > score(B_i,k)`, decreasing as degradations stack, needing no human
labels; (2) rule isolation — a degradation changes only the penalties of the
rules it targets; (3) human agreement — Kendall's τ ≥ 0.7 over ~20 pairs James
ranks; (4) stability — re-scoring, and scoring after a no-op mutation, is
bit-identical. Weights are **not** regression-fitted; `K` is frozen last, by
requiring set A to land in 85–100 and worst-degraded set B in 30–50
(`style-rules.md` §6).

Documentation rules (`KI-DOC-*`) are built from published text sources only; the
KiCon talk video is not consulted (Q22).

## 12. Rendering (resolves R3, R11)

Pipeline, verified end-to-end (`rendering.md` §1):

```
kicad-cli sch export svg -n -e [--black-and-white] [--pages N]
  → viewBox crop  → <g id="kicli-annotations"> overlay  → resvg raster
  → .svg + .png + JSON manifest
```

Only stage 1 needs `kicad-cli`. **Renders are passive output; nothing rendered
ever feeds the score** (Constitution §4). `AGENT.md` must restate this, because
the annotation overlay makes a render look like analysis output — the badges are
*derived from* the structured views, and the views are the truth.

**Region cropping is kicli's job (C6, `kicad-cli.md` ⚠3):** `kicad-cli` has no
`--region`, `--bbox`, `--zoom` or `--dpi`. But its SVG's **user units are
millimetres with the origin at the page top-left, matching schematic coordinates
exactly**, so a crop is a pure `viewBox` + `width`/`height` rewrite with no
coordinate transformation. `--region x0,y0,x1,y1` and
`--around R7 --radius 50mm` (resolve the refdes to its full box, inflate) both
snap outward to the grid with a 2 G margin so edge pins are not sliced.

**Annotation modes**: `refdes`, `findings`, `grid`, `nets`, and `uuids`.
**`uuids` is region-only and truncated to 8 hex chars** (C20,
`rendering.md` ⚠3) — a UUID badge per object is unreadable at sheet scale. Modes
compose as sibling groups inside one `#kicli-annotations` group so each is
toggleable and all are strippable. Annotation text uses generic font families
only (`monospace`/`sans-serif`), is sized `clamp(region_height/40, 1.0, 3.0)` mm,
and is placed inside the viewBox by trying candidate sides in a fixed order —
falling back to numbered markers plus a corner legend when there is no room.

**Style (Q23):** black-and-white when `--output json` (machine consumer), KiCad's
colour theme when human-invoked or `--style color`. **Both SVG and PNG are
emitted**, both paths returned.

**Raster (Q24):** `render.max_px = 1600` on the long edge, clamped so effective
resolution is ≥ **6 px/mm**; below that, 1.27 mm text is unreadable to a vision
model. If both cannot be satisfied, emit at 1600 px and warn that text may be
illegible, suggesting a smaller region. Rasteriser is `resvg` (MIT/Apache as of
0.48.1; its older MPL-2.0 releases would also qualify now, so pin the version
for reproducible output rather than for its licence).

**Cache (Q24):** exported SVGs are cached under `.kicli/render/` keyed on the
sheet's content hash (§7.3).

Manifest fields `objects_in_view` and `clipped_annotations` let an agent
self-correct without looking: zero objects means the region is wrong, non-zero
clipped annotations means widen it.

**Determinism**: KiCad's SVG embeds a timestamp in `<title>`; kicli normalises
it (and the `<desc>Image generated by Eeschema-SVG</desc>`) before hashing or
diffing. Golden tests compare **normalised SVG text**, not PNG bytes; one PNG
smoke test per platform asserts non-blank output and expected ink coverage,
guarding against the classic empty-font-database failure.

**Cold start**: the first `kicad-cli` run on a machine can take >120 s building
the fontconfig cache; warm runs are 0.17–0.5 s. `kicli project check` warms it
deliberately and says what it is doing, or an agent will think kicli hung
(`kicad-cli.md` §1.1).

## 13. PCB parametric ops (resolves R5)

**There is no schematic IPC API in KiCad 10.0.5** — `schematic_commands.proto`
contains no messages at all (`ipc-api.md` §4.4). Schematic work is file-based or
it does not happen. IPC exists in kicli for the PCB phase only.

**C19: `pcb` commands require a running KiCad** with the API server enabled
(`api.enable_server` in `kicad_common.json`, Preferences → Plugins → Enable
KiCad API) and the board open. There is **no headless fallback**. This is a
materially different UX from every other kicli command and `AGENT.md` must say
so. Missing socket ⇒ exit 6 with the exact remedy printed.

Transport: **nng REQ0/REP0 over a Unix domain socket** (`ipc:///tmp/kicad/api.sock`,
overridable via `KICAD_API_SOCKET`, with a Flatpak path probe). Messages are
datagram-framed by nng — **kicli must not write a length prefix**. Payloads are
`google.protobuf.Any`, dispatched by type URL. The `kicad_token` is empty on the
first request and adopted from the response header thereafter, else later
requests get `AS_TOKEN_MISMATCH`. `AS_NOT_READY` and `AS_BUSY` are **normal** and
retried with bounded exponential backoff; `AS_UNIMPLEMENTED` is a hard,
clearly-labelled failure.

**Client choice (Q29, dissolved by §2):** kicli depends on **`kicad-ipc-rs`
0.5.1 (MIT, checked-in generated code)** because checked-in generated code is
less build machinery than a protobuf codegen step, not because of its licence.
KiCad's GPL-3 `.proto` files may now be vendored or generated from directly, and
M9 may take that route if it proves cleaner. No upstream licensing outreach is
needed.

**Version floor (Q30):** minimum KiCad **10.0.0** for `pcb` commands, verified
via `GetVersion` at connect; refuse otherwise (exit 6).

**Every command is wrapped in `BeginCommit`/`EndCommit`** (C19,
`ipc-api.md` ⚠3), so one kicli invocation is one KiCad undo step rather than N.

| Op | Route |
|---|---|
| rectangular / rounded outline on Edge.Cuts | `CreateItems` with `BoardGraphicShape` on `BL_Edge_Cuts` |
| N fiducials | `CreateItems` with footprint instances from a library |
| CNC flip-registration holes (diameter, count, axis) | footprints with NPTH pads (more idiomatic and DRC-clean than raw `PadStack`s) |
| coarse placement by refdes | `GetItems` → match refdes → `UpdateItems` with new positions, inside the commit |
| verification | `GetBoundingBox` / `GetItems`, plus `RefillZones` where copper is affected |

## 14. External tool integration

### 14.1 `kicad-cli` is optional (Q31)

Parsing, geometry, lint, scoring, views and mutations all work without it. Only
`sch erc`, `sch render`, netlist export and the `--gate` path need it. When it is
required and absent, kicli fails with a structured error naming the binary and
an install hint, exit 6. **Nothing is bundled or vendored.** Discovery order:
`$KICLI_KICAD_CLI` → `kicli.toml` `kicad_cli_path` → `PATH` →
`/Applications/KiCad/KiCad.app/Contents/MacOS/kicad-cli`. Major version ≠ 10 is
refused. All invocations go through one module that owns discovery, the version
check, and the §6.2 exit-code translation.

### 14.2 The ERC JSON 100× unit bug — canary required (C5, Q35)

KiCad 10.0.5's ERC JSON exporter builds its units provider with `pcbIUScale`
(1e6 IU/mm) instead of `schIUScale` (1e4), so schematic coordinates are reported
**100× too small while labelled `"mm"`**. The text report is correct. Still
unfixed on `master`. Root cause: `eeschema/erc/erc_report.cpp:161` vs `:63`
(`geometry.md` §3.5).

kicli's requirements:

1. **Never consume `kicad-cli sch erc --format json` coordinates as-is.** Either
   read the text report, or apply a sanity-checked ×100 correction to the JSON
   (the JSON is otherwise preferable, because each violation item carries the
   offending object's **UUID**, which joins directly to kicli's handles —
   `kicad-cli.md` §3.1).
2. **A CANARY TEST that expects the bug**: on a committed fixture, assert
   `json.pos × 100 == text.pos` exactly. When upstream fixes it, this test fails
   loudly and the workaround is *removed*, never double-applied.
3. The correction site carries a code comment naming `erc_report.cpp:161`.
4. No upstream bug report is filed (Q35).

### 14.3 ERC severities are read-only (Q32)

ERC severities live in `.kicad_pro` (`erc.rule_severities`) and `kicad-cli` can
only filter which are reported. In v1 kicli **relabels them for its own output
and never edits `.kicad_pro`**. `kicli.toml`'s ERC severity mapping is therefore
a presentation mapping, and the docs must say so.

### 14.4 Open-document detection — best effort only (Q34 as amended)

kicli edits `.kicad_sch` on disk; if Eeschema has the file open it will not see
the change and may overwrite it. Before a schematic write, kicli makes a
**best-effort** attempt to call IPC `GetOpenDocuments` and warns if the target
document is open in a running KiCad.

Amendment (Q34): this must **never slow or break the no-KiCad-running case**.
Concretely: attempt only when the socket path already exists; total probe budget
`ipc.probe_timeout_ms` (default **250 ms**); **any** connection, timeout or
protocol failure is swallowed silently with no warning and no delay beyond the
budget; the probe never changes an exit code and never blocks the write.

## 15. Config — `kicli.toml`

Project-local, in the project directory. Unknown keys are an **error**, not a
warning — agents typo silently.

```toml
[grid]        step = "50mil"           # 12700 IU; exempt_text = true (Q9)
[formats]     max_schematic_version = 20260306   # §5.1 ceiling; override knob
[view]        max_bytes = 32768        # whole-project vs index+summaries (Q11)
[libraries]   shared_path = "../shared"; shared_nick = "Eurorack Common"; …
[routing]     label_threshold = "30G"  # shared with rules.KI-LBL-001 (C14)
              w_turn = 6; w_cross = 20; w_text = 12; w_near = 2; margin = "8G"
[rules]       default_tier2_enabled = true; gate_on_tier1 = true; consume_erc = true
[rules."KI-XING-001"] enabled = true; weight = 1.0; free_allowance = 2
[render]      max_px = 1600; min_px_per_mm = 6; style = "auto"; cache = true
[erc]         severity_map = { … }     # presentation only (Q32)
[ipc]         probe_timeout_ms = 250   # Q34
[tools]       kicad_cli_path = "…"
```

`routing.label_threshold` is read by both the router and `KI-LBL-001`. It is one
knob (C14); duplicating it is a bug.

## 16. Agent documentation (deliverable)

`AGENT.md` ships with releases: command reference, the canonical
look → edit → verify → score loop, worked examples (create-from-netlist,
tidy-existing, add-hierarchical-channel), representation format guide, and — as
Constitution §8 and §10 require — the **exit-code table including the
`kicad-cli` translation** (§6.2), the "renders are passive" statement (§12), the
"`pcb` needs a running KiCad" statement (§13), and the recommendation of
`kicad-tools` for users who want Python and breadth (§4).

## 17. Out of scope (v1)

PCB routing; undo/transactions; spatial queries; SPICE; MCP server; CI; symbol
creation; KiCad version migration (kicli never upgrades a file, §5.1);
multi-user concerns; variant-aware editing (§5.7); `lib migrate-envvars` (§10);
library-nickname renaming (§10); bus routing and cross-sheet routing (§9);
writing `.kicad_pro` (§5.5).

**Post-v1 backlog, optional and local-only:** a coverage-guided fuzz harness for
the parser. It needs a nightly toolchain, and the repository pins stable. The
property it would defend — arbitrary bytes produce an error or a tree, never a
panic — is already held by property tests on stable. A fuzzer would add
coverage-guided search and a persistent corpus. It is never a merge gate.

## 18. Fixtures and test corpora (C21, D18, Constitution §11)

This section fixes the **provenance** of test data, not its location. Where
fixtures sit in the tree is an engineering concern and is specified in
`ENGINEERING.md`.

- **In-repo fixtures are purpose-built** — authored by us, not copied from
  anyone — and they are the gate for the default `cargo test`. Canonical-byte
  fixtures are canonicalised once with `kicad-cli sch upgrade --force` (which is
  idempotent): our content, KiCad's bytes. Any fixture derived from a v9 source
  must be checked for bus aliases first (`sch-format.md` §5.6).
- **KiCad's `demos/` and `qa/data` are never vendored.** They are an external
  corpus fetched by **`cargo xtask corpus`** at a pinned tag into `target/`,
  excluded from the default test run (Q5). Vendoring them is now permitted
  (§2), and the fetch stays: it keeps the repository small, keeps the in-repo
  fixtures purpose-built, and pins the corpus to a KiCad tag rather than to a
  copy that ages in the tree.
- The `kiutils` / `kicad-skip` round-trip comparison is kept as a **regression
  fixture**: kicli must preserve every token on the file where `kiutils` loses
  14.7 % (`ecosystem.md` §7).
- Named permanent gates: the **115-file prettifier identity test** (§5.3), the
  **16-row pin-position fixture** verified against KiCad's own ERC output
  (§8), the **netlist-partition oracle** against `kicad-cli sch export netlist`
  (§7.1), the **router determinism and 15 % calibration tests** (§9), the **ERC
  JSON canary** (§14.2), and the **view byte-budget ceilings** (§7.4).

## 19. Milestones (build order)

1. **M1 Parser core** — token-preserving tree + `Prettify` port for
   `.kicad_sch`/`.kicad_sym`/`.kicad_pro`, P1/P2 round-trip gates, fixture
   corpus and `cargo xtask corpus` bootstrapped. Tasks: `tasks/M1.md`.
2. **M2 Geometry + read** — pin maths, bboxes, text metrics table, connectivity
   extraction with the name-based merges, `sch view` all three views,
   `project info|check`. Gates: pin fixture, netlist oracle, view budgets.
3. **M3 Mutations** — sym/field/text/label/junction ops with auto-invariants,
   sheet-path-aware `Reference`, P3 locality.
4. **M4 Wiring** — router + `wire connect|draw|delete`, determinism and
   calibration gates.
5. **M5 Score** — Tier 1 then Tier 2, ERC consumption and canary, calibration
   fixtures.
6. **M6 Render** — `kicad-cli` SVG, viewBox crop, annotation overlay, resvg.
7. **M7 Libraries** — `parts search`, vendoring both directions.
8. **M8 Hierarchy** — sheet ops; instance-awareness is threaded through M2–M7 as
   a constraint and surfaced fully here.
9. **M9 PCB** — `kicad-ipc-rs` client + parametric ops, commit-wrapped.
10. **M10 Agent docs + polish** — `AGENT.md`, worked examples, release packaging.

Each milestone's tasks are cut from this spec plus the relevant research doc,
each with an executable completion check per Constitution §11.

## 20. Contradiction resolution record (C1–C21)

All 21 accepted (`DECISIONS-R6.md`). Where applied in this spec:

| # | Motivating doc | Resolution | Applied in |
|---|---|---|---|
| C1 | `sch-format.md` ⚠1–2, `sexpr-strategy.md` ⚠1 | Token-preserving tree + `Prettify` port, not a whitespace CST. **Both** round-trip properties named: P1 (byte-identical for KiCad-authored input) and P2 (semantic, all input) gate merges; P4 ("what KiCad would write") is informational because KiCad reorders items | §5.3, §19 M1 |
| C2 | `sch-format.md` ⚠3 | Concrete floor **10.0.5 / `20260306`**; parse any stamp, write back what was read, refuse to write above the known maximum with a config knob | §3 D1, §5.1 |
| C3 | `sch-format.md` ⚠4, `geometry.md` ⚠2 | Grid discipline restated in integer IU (12700 IU); the blocking rule covers **connectable geometry only**, field/graphic text exempt | §5.2, §8, §11.4 |
| C4 | `kicad-cli.md` ⚠1 | kicli defines its own codes 0–6 and **always translates** `kicad-cli`'s; pass-through forbidden; table documented in `AGENT.md` | §6.1, §6.2, §16 |
| C5 | `geometry.md` ⚠1, `kicad-cli.md` ⚠2 | ERC JSON coordinates never consumed as-is; ×100 correction or text report, plus a **canary test that expects the bug** and a code comment citing `erc_report.cpp:161`; no upstream filing | §14.2 |
| C6 | `kicad-cli.md` ⚠3 | `kicad-cli` has no region rendering; kicli crops by `viewBox` rewrite (SVG user units are mm, matching schematic coords) | §12 |
| C7 | `style-rules.md` ⚠1 | Scoring **layers on** ERC's 47 checks and re-implements none, with two documented exceptions where KiCad defaults to `IGNORE` | §11.1 |
| C8 | `style-rules.md` ⚠2 | `sch score --gate` may require `kicad-cli`; absence is a structured error, exit 6 | §11.2, §14.1 |
| C9 | `style-rules.md` ⚠3 | Score formula given with **density normalisation** (`per_object`, `per_wire`, `per_sheet`) and `K = 25`; Tier 1 fails the gate without reducing the score | §11.5 |
| C10 | `representation.md` ⚠1 | Snapshots hash **per object by UUID** over a canonical semantic encoding — never file bytes, which KiCad's reordering makes useless. SHA-256 truncated to 16 hex | §7.3 |
| C11 | `representation.md` ⚠2 | View scope defined: whole project within `view.max_bytes`, else index + per-sheet summaries, always stating which; `--sheet` marks ports | §7 |
| C12 | `representation.md` ⚠3 | Synthetic stable `#n<k>` names are the handle; **amended (Q12)** — views also carry KiCad's current net name as an attribute for ERC/GUI correlation | §7.1 |
| C13 | `wire-routing.md` ⚠2 | `wire connect <pin> <net>` added to the command surface | §6, §9 |
| C14 | `wire-routing.md` ⚠1 | Router's label threshold and `KI-LBL-001`'s long-wire rule are **one config key**, `routing.label_threshold` | §9, §15 |
| C15 | `libraries-and-vendoring.md` ⚠1 | Default shared-library layout changed to sibling `../shared` + existing nickname; `libs/parts` dropped | §3 D8, §10 |
| C16 | `libraries-and-vendoring.md` ⚠4 | Vendoring rewrites the embedded `lib_symbols` cache and the `Footprint` field values; `sym place` embeds the definition as a mandatory step | §10 |
| C17 | `ecosystem.md` ⚠1 | Differentiation reworded: lossless round-trip + full manipulation parity + deterministic offline scoring + single binary — not "nobody can move field text" (`kct sch tidy` exists) | §4 |
| C18 | `ecosystem.md` ⚠2 | Prior-art / "why not X" section added, covering `kiutils`, `kicad-skip`, `kicad-tools`, `circuit-synth`, Konnect | §4 |
| C19 | `ipc-api.md` ⚠1, ⚠3 | `pcb` requires a running KiCad with the API enabled, no headless fallback, exit 6 with the remedy; every op wrapped in `BeginCommit`/`EndCommit` | §13 |
| C20 | `rendering.md` ⚠3 | `--annotate uuids` is region-only and truncated to 8 hex chars | §12 |
| C21 | `sexpr-strategy.md` ⚠2 | GPL demo/`qa` files are an **external** corpus fetched by `cargo xtask corpus` at a pinned tag into `target/`; in-repo fixtures are purpose-built and gate the default test run | §18 |
