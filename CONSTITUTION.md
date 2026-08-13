# kicli Constitution

Immutable principles. Every task, PR, and design decision must comply. If a task
appears to require violating one of these, stop and escalate to James rather than
proceeding.

## 1. Round-trip fidelity is sacred

Parsing a `.kicad_sch`, `.kicad_pcb`, `.kicad_sym`, or `.kicad_pro` file and
serialising it back MUST produce output that KiCad 10.0 opens identically. Where
byte-identity is achievable it is required; where KiCad itself is inconsistent
(whitespace, float formatting), semantic identity verified by re-parse comparison
is the floor. Any file kicli cannot round-trip losslessly, kicli must refuse to
modify. Property-based round-trip tests gate every merge.

## 2. Full manipulation parity

Anything a human can select, move, rotate, mirror, or edit in the KiCad 10
schematic editor, kicli can too — symbols, wires, buses, junctions, no-connects,
net labels, global labels, hierarchical labels and sheet pins, text items, text
boxes, graphic lines/shapes, and crucially **field text** (reference, value,
footprint, user fields) including position, rotation, justification, and
visibility. "The tool can't move that" is always a bug, never a limitation.

## 3. A schematic is a drawing that encodes a netlist

Both aspects are first-class. Electrical correctness without readability is
failure (that is Konnect's failure). Every feature is judged on both axes.

## 4. Scoring is deterministic

The lint/score engine uses geometry and heuristics only. kicli NEVER calls an
LLM, vision model, or any network service. No API keys, no telemetry, no network
I/O at runtime. Rendering exists so that an external agent can look; what the
agent sees never feeds the score.

## 5. Every mutation is verified and reported

Every mutating command automatically re-runs cheap invariants (output re-parses,
UUID references intact, pins on-grid, no orphaned instance data) and reports
results in its structured output. Failure = non-zero exit + no file written
(mutations are atomic: write temp, verify, rename). ERC and full lint/score run
only on explicit command.

## 6. Outputs are designed for LLM context budgets

Every read command has a compact, stable, documented text/JSON form. Verbosity is
opt-in, never default. If a representation would flood an agent's context on a
realistic sheet, it is wrong.

## 7. Grid discipline

All placement operations snap to the schematic grid (default 50 mil / 1.27 mm)
unless explicitly overridden with a flag that shouts about it. Off-grid pins are
a blocking lint error.

## 8. CLI conventions

`kicli <noun> <verb> [flags] [args]` (kubectl/docker style). Human-readable
output by default, `--output json` for machines, stable documented exit codes,
errors as structured JSON on stderr when `--output json`. Command surface changes
require updating the agent docs (Principle 10) in the same change.

## 9. Licensing hygiene

kicli is GPL-3.0-or-later. Dependencies must be GPL-3-compatible
(MIT/Apache/BSD/MPL-2.0 all qualify). AGPL code, including Konnect, remains
excluded — never read its source; black-box observations only. KiCad's own GPL
source, protos, fonts, and demo files may now be read and derived from freely.

## 10. Agent docs ship with the tool

An agent-facing usage document (skill/CLAUDE.md form: command surface, the
look → edit → verify → score loop, worked examples) is a first-class deliverable
maintained in lockstep with the CLI. A feature undocumented for agents is
unfinished.

## 11. Everything is machine-verifiable

Every task in the task list has an executable completion check (test, golden-file
comparison, round-trip property). Fixtures are purpose-built and live in-repo.
No CI service initially — `cargo test` and the fixture suite run locally and
must pass before any task is marked complete.

## 12. Scope fences (v1)

Out of scope for v1, do not build even if convenient: PCB routing, tool-level
undo/transactions, spatial queries, SPICE authoring/running, MCP server, CI
pipelines, symbol creation from scratch. In scope but sequenced last: PCB
parametric ops (edge cuts, fiducials, CNC flip-registration holes, coarse
placement) via the IPC API.
