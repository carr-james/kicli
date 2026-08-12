# kicli — Specification (v1)

Status: skeleton — structure and decisions are fixed; sections marked
`[NEEDS-RESEARCH: Rx]` are completed from the corresponding research doc before
implementation tasks are cut. Governed by `CONSTITUTION.md`.

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
- Licence: MIT OR Apache-2.0.

## 3. Decisions record (from elicitation, 2026-08)

| # | Decision |
|---|----------|
| D1 | Target KiCad 10.0 file formats and IPC API |
| D2 | CLI-first; MCP wrapper possible later, out of v1 |
| D3 | Full manipulation parity incl. field text (Konnect's gap) |
| D4 | Eyes = structured views + renders (full sheet and region); agent looks with its own vision — kicli never calls models |
| D5 | Rust |
| D6 | Edit-existing and create-from-scratch equally weighted in v1 |
| D7 | Context-efficient representations are a core requirement, not a nicety |
| D8 | Custom libraries day 1; shared parts library = git submodule at a conventional path; vendoring in (project-local) and up (into shared lib); symbol creation out of v1 |
| D9 | Full hierarchical sheet support |
| D10 | Verification: cheap invariants auto per mutation; ERC + score explicit |
| D11 | Scoring: deterministic Tier 1 + Tier 2 rules only, per rule catalogue |
| D12 | Wire drawing: deterministic auto-route default with cost report; explicit vertices on demand |
| D13 | Addressing: UUIDs + refdes/net/sheet-path handles; spatial queries deferred |
| D14 | No tool-level undo in v1 (git suffices) |
| D15 | Per-project `kicli.toml` config |
| D16 | SPICE authoring/simulation deferred (post-v1) |
| D17 | Name: kicli; kubectl/docker-style `noun verb` CLI |
| D18 | Fixtures purpose-built, in-repo |
| D19 | Agent docs are a first-class deliverable |
| D20 | PCB parametric ops in v1, sequenced last: edge cuts, fiducials, CNC flip-registration holes, coarse footprint placement |

## 4. Command surface (draft)

Global flags: `--output json|text` (text default), `--project <dir>`,
`--sheet <path>`, `--quiet`, `--version`. Exit codes: 0 ok; 1 operation error;
2 usage error; 3 verification failure (mutation rolled back); 4 file/parse
error. `[Exact surface finalised after R10]`

```
kicli project info|check                    # project summary, health check
kicli sch view                              # compact views: --view connectivity|layout|delta
kicli sch render                            # SVG/PNG, --region, --annotate
kicli sch erc                               # kicad-cli wrapper, JSON findings
kicli sch score                             # lint + weighted score, JSON findings
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

## 5. Representations `[NEEDS-RESEARCH: R10]`

Three stable, documented views (connectivity, layout digest, delta), terse text
form + JSON twin, token budgets specified per view. Delta view keyed on content
hash snapshots.

## 6. Mutation semantics

Atomic writes (temp + verify + rename). Auto-invariants per Constitution §5.
All coordinates grid-snapped (D-grid 50 mil default, configurable) unless
`--off-grid` (which is also a lint error, so the agent feels it).
`[Pin math and bbox rules: R7]`

## 7. Wiring `[NEEDS-RESEARCH: R9]`

`wire connect A B`: deterministic orthogonal router, returns route taken +
cost breakdown (crossings, doglegs, length) so the agent can accept/redo;
threshold beyond which it emits paired net labels instead and says so.

## 8. Libraries and vendoring `[NEEDS-RESEARCH: R4]`

Shared parts library as git submodule at conventional path (default
`libs/parts`, configurable). `parts search` queries it by name/value/keywords/
footprint. Vendoring copies symbol + footprint (+3D), rewrites lib_ids and
lib tables; `--into shared` grows the catalogue.

## 9. Scoring `[NEEDS-RESEARCH: R8]`

Rule catalogue Tiers 1–2, ESLint-style findings (rule id, severity, sheet,
coords, objects, message, fix hint), weighted 0–100 score per sheet + project.
Config in `kicli.toml [rules]`: enable/disable, severity, weight, thresholds.
Blocking rules fail `sch score --gate`. Calibration set: purpose-built
good/bad fixtures; ranking must match human judgement.

## 10. Rendering `[NEEDS-RESEARCH: R3, R11]`

`sch render`: full sheet or `--region` (bbox or `--around R7 --radius 50mm`),
`--annotate uuids|refdes|findings|grid`, SVG or PNG out. Passive output only.

## 11. PCB parametric ops `[NEEDS-RESEARCH: R5]`

Via IPC against a running KiCad: rectangular/rounded outline on Edge.Cuts,
N-point fiducials, mirror-axis registration holes for CNC board flipping
(parametric: diameter, count, axis), coarse footprint placement by refdes to
coords. All parametric and deterministic; the agent supplies parameters only.

## 12. Config — `kicli.toml`

Project-local: grid, library paths/submodule path, rule config, ERC severity
mapping, render defaults, kicad-cli path override.

## 13. Agent documentation (deliverable)

`AGENT.md` shipped with releases: command reference, the canonical
look → edit → verify → score loop, worked examples (create-from-netlist,
tidy-existing, add-hierarchical-channel), representation format guide.

## 14. Out of scope (v1)

PCB routing; undo/transactions; spatial queries; SPICE; MCP server; CI; symbol
creation; KiCad version migration; multi-user concerns.

## 15. Milestones (build order)

1. **M1 Parser core** — lossless CST for .kicad_sch/.kicad_sym/.kicad_pro,
   round-trip property tests, fixture corpus bootstrapped.
2. **M2 Geometry + read** — pin math, bboxes, connectivity extraction,
   `sch view` all three views, `project info`.
3. **M3 Mutations** — sym/field/text/label/junction ops with auto-invariants.
4. **M4 Wiring** — router + `wire` commands.
5. **M5 Score** — Tier 1 rules, then Tier 2; calibration fixtures.
6. **M6 Render** — kicad-cli integration, region crop, annotation overlay.
7. **M7 Libraries** — parts search, vendoring both directions.
8. **M8 Hierarchy** — sheet ops, instance-aware everything (threaded through
   M2–M7 as constraints; surfaced fully here).
9. **M9 PCB** — IPC client + parametric ops.
10. **M10 Agent docs + polish** — AGENT.md, examples, release packaging.

Each milestone's tasks get cut from this spec + the relevant research doc, each
with an executable completion check per Constitution §11.
