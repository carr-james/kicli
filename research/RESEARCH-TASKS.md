# kicli — Research Dossier Tasks (Phase 1)

Each task produces a markdown doc in `research/` that later phases cite as ground
truth. Facts must be verified against KiCad 10.0 specifically (not 7/8/9 docs) and
sources linked. A finding that contradicts the spec skeleton gets flagged at the
top of the doc, not silently absorbed.

## R1 — `.kicad_sch` format deep-dive (KiCad 10)
The S-expression grammar as KiCad 10 actually writes it: top-level structure,
`lib_symbols` embedding, symbol instances vs library definitions, UUID semantics,
`instances` blocks and hierarchical path data, field/text records (position,
rotation, justify, effects), wires/buses/junctions/no-connects, sheet and
sheet-pin records, project file linkage. Note every difference from KiCad 9.
→ `research/sch-format.md`

## R2 — Rust S-expression strategy
Survey existing crates (lexpr, sexp, any kicad-specific crates) for lossless
round-tripping: comment/whitespace/float-format preservation. Expected outcome is
a bespoke lossless parser (concrete syntax tree, not AST); confirm or refute, and
specify the CST design either way. Define the round-trip property test harness.
→ `research/sexpr-strategy.md`

## R3 — `kicad-cli` capabilities audit (KiCad 10)
Exact flags/behaviour for: `sch export svg` (page vs region? plot options,
background, colour theme), `sch erc` (report formats, severity mapping, exit
codes), netlist export formats, `sch export bom`. Confirm what region/zoom
rendering requires us to do ourselves (likely: post-process the SVG — crop by
viewBox — rather than expect kicad-cli support). Availability/installation on
Arch and macOS.
→ `research/kicad-cli.md`

## R4 — Library resolution and vendoring mechanics
How `sym-lib-table` (global + project) resolves `lib_id`s; how embedded
`lib_symbols` interact with external libs; what a correct vendor operation must
rewrite (symbol copy, footprint copy, 3D model path, `lib_id` rewrite in sheet +
instances, fp-lib-table entry). The git-submodule shared-library layout as a
first-class convention. Failure modes: name collisions, version drift.
→ `research/libraries-and-vendoring.md`

## R5 — KiCad IPC API from Rust
Locate the `.proto` definitions and transport details (NNG socket, framing,
handshake) for KiCad 10. Assess prost + nng crate feasibility; enumerate the
board-editor operations actually available in 10.0 relevant to our PCB scope
(board outline/edge-cuts drawing, footprint place/move, drill/NPTH creation,
user layers). Connection lifecycle: KiCad must be running with API enabled —
document setup. NO reading Konnect source for this (AGPL); use official docs,
proto files, and the Python kipy source as the reference implementation of the
protocol.
→ `research/ipc-api.md`

## R6 — Ecosystem teardown
kicad-tools (rjwalters), kicad-sch-api / circuit-synth, kicad-skip, kiutils:
representation choices, command ergonomics, what agents get wrong when driving
them, gaps (especially text/field manipulation). Konnect: black-box behavioural
notes only (feature list, observed failure modes such as the text repositioning
gap) — no source reading.
→ `research/ecosystem.md`

## R7 — Geometry engine prerequisites
The pin-position resolution maths: symbol library definition + instance position
+ rotation + mirroring → absolute pin coordinates. Text bounding-box computation
from size/justification/rotation. This underpins every lint rule and every
placement op; specify it precisely with worked examples to fixture-ify.
→ `research/geometry.md`

## R8 — Style standards consolidation
Merge sources into the rule catalogue: Olin Lathrop's rules, Andrew Greenberg's
*Actually Useful Schematics* (KiCon 2025), published vendor review checklists
(TI/ADI app-note conventions), community review checklists. Reconcile with the
existing `schematic-lint-rule-catalogue.md` (drop Tier 3 from scoring; Tiers 1–2
only). For each rule: detection maths, severity (blocking vs scored), default
weight, config knobs. Define the calibration method: score known-good open
hardware schematics vs known-bad generated ones; ranking must match human
judgement.
→ `research/style-rules.md` (supersedes the draft catalogue)

## R9 — Orthogonal schematic wire routing
Algorithm for the default auto-router: grid-based A*/Lee with cost terms
(crossings, doglegs, proximity to text/symbols), deterministic tie-breaking
(same input → same route, always), and the cost-report format returned to the
agent. When to emit a net label instead of a wire (distance threshold).
→ `research/wire-routing.md`

## R10 — Compact representation design
Concrete schemas for the three views: connectivity (nets/pins/hierarchy), layout
digest (positions, bboxes, orientations at reduced precision), delta (changes
since a named snapshot/hash). Token-budget targets against a realistic Eurorack
sheet (aim: full connectivity view of a typical sheet in low thousands of
tokens). Decide text format (likely a terse indented DSL + JSON twin).
→ `research/representation.md`

## R11 — Rendering + annotation overlay
Pipeline from `kicad-cli` SVG to agent-consumable images: region crop via
viewBox, optional annotation overlay (object UUIDs/refdes badges, grid ticks,
lint-finding markers) by SVG post-processing, SVG→PNG rasterisation choice in
Rust (resvg).
→ `research/rendering.md`

Suggested order: R1 → R7 → R2 → R3 → R8 → R10 → R9 → R11 → R4 → R6 → R5.
(R5 last: PCB is sequenced last in the build; don't block schematic work on it.)
