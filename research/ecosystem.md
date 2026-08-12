# R6 — Ecosystem teardown

Status: the two general-purpose Python libraries were **installed and
round-trip-tested against a real KiCad 10.0.5 file** (§2) — the results are
decisive and are the strongest argument for kicli's parser design. Konnect was
examined **black-box only**; §5 states exactly what was and was not done.

---

## ⚠ Contradictions and cautions for `spec/SPEC.md`

1. **SPEC D3 frames field-text manipulation as "Konnect's gap", implying nobody
   has it. That is not accurate as a general claim about the ecosystem.**
   `kicad-tools` (rjwalters) ships `kct sch tidy`, which repositions
   `Reference`/`Value` fields deterministically and documents netlist-invariance
   as a design property (§4.3). kicli's differentiation has to be stated more
   precisely — see §6.

2. **SPEC has no "prior art / why not X" section**, and it needs one: two of the
   four Python projects surveyed are agent-oriented and actively maintained. The
   honest answer for kicli is lossless round-trip + determinism + parity +
   single binary, not "nothing else exists".

3. **Constitution §9 compliance note**: no Konnect source was read. The
   observations in §5 come from the MCP tool catalogue exposed to this session
   and from published descriptions — the same information any user of the
   product sees.

---

## 1. What was surveyed

| Project | Version tested | Licence | Language | Posture |
|---|---|---|---|---|
| [`kiutils`](https://github.com/mvnmgrx/kiutils) | 1.4.8 (PyPI) | MIT | Python dataclasses | general file library, "KiCad 6.0 and up" |
| [`kicad-skip`](https://github.com/psychogenic/kicad-skip) | 0.2.5 (PyPI) | — (repo has no LICENSE file at HEAD) | Python over `sexpdata` | REPL-friendly manipulation, "KiCad 7+" |
| [`kicad-tools`](https://github.com/rjwalters/kicad-tools) | 0.20.0 | MIT | Python | **explicitly agent-focused**, JSON-out CLI |
| [`circuit-synth`](https://github.com/circuit-synth/circuit-synth) | (repo HEAD) | MIT | Python | code-generates KiCad projects from Python |
| [`kicad-python`](https://gitlab.com/kicad/code/kicad-python) (kipy) | 0.8.0.dev0 | MIT | Python | official IPC API bindings — see R5 |
| Konnect | as exposed to this session | AGPL | — | MCP server, black-box only |

---

## 2. The round-trip test (the important result)

### 2.1 Recipe R6-A

```sh
python3 -m venv venv && venv/bin/pip install kiutils kicad-skip
cp demos_v10/complex_hierarchy/ampli_ht.kicad_sch in.kicad_sch   # KiCad 10.0.5, version 20260306

# kiutils
from kiutils.schematic import Schematic
Schematic().from_file("in.kicad_sch").to_file("out_kiutils.kicad_sch")

# kicad-skip
import skip
skip.Schematic("in.kicad_sch").write("out_skip.kicad_sch")
```

Then compare (a) bytes, (b) the **token list** (a whitespace-insensitive, escape-
aware tokenisation), (c) per-token-name counts.

### 2.2 Results

| | bytes in | bytes out | byte-identical | token list identical | tokens lost |
|---|---|---|---|---|---|
| `kiutils` 1.4.8 | 112,996 | 90,261 | no | **no** (19,851 → 16,939) | **2,912 (14.7 %)** |
| `kicad-skip` 0.2.5 | 112,996 | 90,753 | no | **yes** (19,851 → 19,851) | 0 |

Both outputs are still accepted by KiCad (`kicad-cli sch erc` exits 0 on each),
which is precisely what makes the `kiutils` result dangerous: **the loss is
silent.**

### 2.3 What `kiutils` drops

Token-name diff, input → output:

| token | occurrences lost | consequence |
|---|---|---|
| `do_not_autoplace` | 295 | KiCad may re-autoplace fields the user had pinned |
| `hide` | **166** | **hidden fields and pin names become visible** |
| `exclude_from_sim` | 59 | simulation scope silently changes |
| `in_pos_files` | 58 | pick-and-place inclusion silently changes |
| `body_style` | 46 | De Morgan/alternate body style reset to default |
| `duplicate_pin_numbers_are_jumpers` | 12 | jumper-pin semantics lost |
| `embedded_fonts` | 12 | embedded-font declaration lost |
| `pin_numbers` | 3 | pin-number visibility lost |
| `generator_version`, `bold`, `italic` | 1 each | metadata / formatting |

It also **unquotes strings** that KiCad quotes:

```
in : (generator "eeschema") (generator_version "10.0") (uuid "994297ef-4ddc-…")
out: (generator eeschema)                              (uuid 994297ef-4ddc-…)
```

which is legal to KiCad's lexer but diverges from the canonical writer
(`sch-format.md` §2.2) and would make any byte-comparison test meaningless.

**Diagnosis**: `kiutils` is a typed dataclass AST — exactly the architecture
`sexpr-strategy.md` §3.3 rejects. Tokens introduced after the library's model was
written (all of the above are KiCad 8/9/10 additions) have nowhere to live, so
they vanish on write. This is not a bug that can be fixed once; it is a
structural property that recurs every KiCad release.

### 2.4 What `kicad-skip` gets right

`kicad-skip` parses to a generic s-expression tree (`sexpdata`) and writes it
back, so **every token survives** — 19,851 in, 19,851 out, identical list. Its
output differs from the input only in whitespace, because it does not implement
KiCad's prettifier (`sch-format.md` §2.1).

That is a genuinely good result and worth saying plainly: of everything
surveyed, `kicad-skip` is the only library whose round-trip preserves the data.
It also validates kicli's plan — token-preserving tree + faithful prettifier
gets you from "semantically lossless" (skip) to "byte-identical" (kicli).

Caveats observed during the test: it emitted `Passed key -- can't parsy` to
stdout twelve times while loading a valid KiCad 10 file — diagnostics leaking
into stdout, which breaks any tool that parses stdout, and suggests unhandled
constructs are being skipped in its *convenience layer* even though the
underlying tree is intact.

---

## 3. Representation choices, compared

| Project | Model | Unknown tokens | Byte fidelity | Number handling |
|---|---|---|---|---|
| `kiutils` | typed dataclasses per KiCad object | **dropped** | no | parsed to Python floats/strings |
| `kicad-skip` | generic sexp tree + convenience wrappers | preserved | no (layout only) | kept as tokens |
| `kicad-tools` | own parser → Python objects + JSON | partial (targeted edits) | edits are surgical by design | — |
| `circuit-synth` | Python DSL → generated project | n/a (generator) | n/a | n/a |
| **kicli (planned)** | token-preserving tree + span reuse + prettifier port | **preserved** | **yes** | int IU, never float |

---

## 4. Per-project notes

### 4.1 `kiutils`

- Clean, well-documented dataclass API; pleasant to write against.
- README targets "KiCad 6.0 and up"; the format has moved a long way since.
- Fine for read-only extraction where the dropped attributes do not matter.
  **Unsafe for any write path** on KiCad 8+ files (§2.3).

### 4.2 `kicad-skip`

- REPL-first ergonomics (`schematic.symbol.R14.dnp`, TAB completion), by far the
  nicest interactive experience surveyed.
- Search by location/connection/type; direct access to symbol pin locations.
- Token-lossless (§2.4). No prettifier, so diffs against KiCad-written files are
  whole-file.
- No LICENSE file present at the cloned HEAD — if kicli ever wanted to depend on
  or port from it, that would need resolving. (kicli does not need to.)

### 4.3 `kicad-tools` (rjwalters) — the closest prior art

Explicitly "Tools for AI agents to work with KiCad projects", MIT, actively
released (0.20.0), JSON output on every command, no running KiCad required.
Entry points include `kct` plus standalone `kicad-symbols`, `kicad-nets`,
`kicad-erc`, `kicad-drc`, `kicad-bom`, `kicad-pcb-query`, `kicad-pcb-modify`,
`kicad-lib-symbols`, and modules for analysis, audit, creepage, cost, DRC,
export, design.

Most relevant: **`kct sch tidy`** — autoplaces `Reference`/`Value` fields, whose
own docstring states:

> Resets the positions of visible Reference and Value fields to deterministic
> offsets relative to each symbol's placed body bounding box… The operation is
> strictly cosmetic: only the `(at x y angle)` of Reference/Value `property`
> nodes changes… so the netlist, ERC, BOM, and CPL are provably unchanged.
> …byte-parity with Eeschema's algorithm … is an explicit non-goal.

Its documented limits are informative for kicli:

- power/virtual symbols skipped;
- **hidden fields never moved**;
- symbols whose `lib_id` is absent from embedded `lib_symbols` skipped with a
  warning (the §3 cache-dependency from R4);
- multi-unit symbols use *pin extents* as the body box because per-unit body
  graphics are not tracked — i.e. an approximation where `geometry.md` §6
  specifies the exact computation.

So the honest competitive statement is: field *tidying* exists; field
*manipulation at full parity* (arbitrary fields, arbitrary objects, explicit
positions, justification, visibility, rotation) does not, and neither does an
exact body-box model or a lossless round-trip guarantee.

### 4.4 `circuit-synth`

A different shape entirely: define circuits in Python, generate KiCad projects,
with `kicad-to-python` / `python-to-kicad` sync scripts. Useful prior art for
"generate from netlist" (SPEC §13's worked example) but it owns the source of
truth, whereas kicli's premise is that the `.kicad_sch` *is* the source of truth
and humans keep editing it in the GUI.

### 4.5 `kicad-python` (kipy)

Official IPC bindings — covered in R5. Noted here because `kicad-tools`' README
points at kipy as the way to push results into a running KiCad, which is exactly
the split kicli plans (files offline, IPC only for PCB ops).

---

## 5. Konnect — black-box observations only

**Method and boundary.** Per CLAUDE.md and Constitution §9, no Konnect source was
read, cloned, or decompiled. What follows comes from (a) the MCP tool catalogue
this session was given, and (b) one read-only introspection call
(`list_toolboxes`) that returns the product's own published catalogue. No
project was opened, created, modified, or saved.

### 5.1 Observed architecture

`list_toolboxes` reports **18 toolsets, 187 tools total**, dynamically loadable:

| Category | Toolset | Tools | Description (as published by the server) |
|---|---|---|---|
| project | `project` | 6 | create/open/save/snapshot projects, launch live schematic viewer |
| schematic | `sch_components` | 17 | add, edit, move, rotate, delete schematic symbols |
| schematic | `sch_wiring` | 19 | wires, net labels, power symbols, junctions, no-connects, pin-to-pin |
| schematic | `sch_analysis` | 15 | net connectivity, pin queries, trace paths, overlap/orphan detection |
| schematic | `sch_batch` | 12 | bulk add/edit/delete/move in one call |
| schematic | `sch_export` | 6 | SVG/PDF/netlist export, ERC |
| schematic | `sch_hierarchy` | 12 | sheets, sheet pins, pin/label sync validation |
| pcb | `pcb_board`, `pcb_components`, `pcb_routing`, `pcb_export` | 11/13/12/13 | outline, layers, zones; placement; traces/vias/pours; fab exports |
| library | `library` | 14 | symbol/footprint libraries, search, registration |
| integration | `integration` | 9 | JLCPCB parts DB, Freerouting, datasheet URLs |
| verification | `verification` | 8 | ERC, DRC, design rules, KiCad UI control |
| config | `config` | 7 | user prefs, project rules, fab constraints |
| review | `design_review` | 6 | **"AI-powered design audits"** |
| templates | `templates` | 4 | reference circuits (USB-C, LDO, buck, STM32, I2C, LED) |
| manufacturing | `manufacturing` | 3 | fab package export, validation, cost estimate |

Only `project` and `config` load at startup; the rest are loaded on demand to
keep the agent's context small.

### 5.2 What that tells us, without reading anything

1. **Breadth is the strategy.** 187 tools spanning schematic, PCB, library, fab
   and part sourcing. kicli's scope (Constitution §12) is deliberately far
   narrower.
2. **`design_review` is "AI-powered".** That is the exact architectural fork:
   Constitution §4 forbids kicli from calling any model or network service for
   scoring. Konnect's audits are model-mediated; kicli's score is geometry and
   arithmetic, reproducible offline, and diffable across runs. This is the
   sharpest, most defensible difference, and it is visible from the outside.
3. **The schematic toolsets are enumerated by object class** — symbols, wiring,
   hierarchy, batch — and the published descriptions do not mention field or
   free-text manipulation. That is consistent with the gap SPEC D3 records, but
   note carefully: **this is an absence in a published description, not a
   verified absence of capability.** Nothing here should be quoted as proof
   Konnect cannot move field text; the primary evidence for that remains James's
   own experience of using it.
4. **`snapshot_project` exists** — Konnect offers project-level undo/versioning,
   where SPEC D14 says git suffices. Worth noting as a deliberate divergence
   rather than an oversight.
5. **MCP-first, tool-count-heavy.** The dynamic toolset loading is an explicit
   answer to context bloat — the same pressure Constitution §6 addresses from the
   other end (few commands, compact output).

---

## 6. What agents get wrong when driving these tools

Synthesised from the tools' own documented limitations and from the failure
modes the file format makes possible (`sch-format.md`):

| Failure | Why it happens | How kicli avoids it |
|---|---|---|
| Silent attribute loss on write | typed-AST libraries drop unknown tokens (§2.3) | token-preserving tree; unknown tokens are data, not errors |
| Editing `Reference` on the symbol only | the truth is in `instances → path` (`sch-format.md` §3.6) | field mutations are sheet-path aware by construction |
| Placing a symbol without embedding its `lib_symbols` entry | the embedded cache is what KiCad draws (R4 §3) | `sym place` copies the definition as a mandatory step |
| Fields fighting KiCad's autoplacement | `fields_autoplaced` left set after a manual move | kicli clears it whenever it sets a position (`geometry.md` Q3) |
| Off-grid coordinates from float maths | mm floats round-tripped through `f64` | integer IU throughout (`geometry.md` §1) |
| Wires that look connected but are not | endpoints 1 IU apart, or a crossing mistaken for a junction | grid snapping + explicit junction rules (R9 §5.4) |
| Diff noise burying the real change | whole-file reformat on every write | prettifier port + span reuse (`sexpr-strategy.md` §4) |
| "It opened fine" as a success criterion | KiCad accepts lossy files (§2.2) | verification per mutation (Constitution §5), netlist oracle (R3 §4) |

---

## 7. Implications for kicli

1. The §2 experiment should be **kept as a regression fixture**: kicli's own
   round-trip must be byte-identical where these two are 14.7 % lossy and
   whitespace-lossy respectively. It is a concrete, publishable claim.
2. `kicad-skip`'s REPL ergonomics are the bar for *discoverability*; kicli's
   equivalent is the compact views (R10) plus stable handles.
3. `kicad-tools` is the closest neighbour and should be cited in `AGENT.md` as
   "if you want Python and breadth, use that; kicli is for lossless,
   deterministic, parity editing".
4. Nothing in the ecosystem offers a deterministic *readability score*. That,
   plus lossless round-trip, is the defensible core.

---

## 8. Sources

- Round-trip experiment: this document §2, recipe R6-A, run against
  `kiutils` 1.4.8 and `kicad-skip` 0.2.5 from PyPI on a KiCad 10.0.5 file
  (`version 20260306`).
- Repository clones at HEAD (2026-08-12): kiutils, kicad-skip, kicad-tools,
  circuit-synth, kicad-python; metadata from their `pyproject.toml`/`setup.cfg`
  and READMEs.
- `kct sch tidy` docstring quoted from
  `kicad-tools/src/kicad_tools/cli/sch_tidy.py` (MIT).
- Konnect: MCP tool catalogue exposed to this session, plus one read-only
  `list_toolboxes` call. **No source read.**
