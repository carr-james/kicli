# DECISIONS-R6 — resolutions for research/SUMMARY.md

James's decisions on all open questions and spec contradictions from the Phase 1
research dossier. This file is authoritative input to the spec formalisation
session. Question/contradiction numbers refer to research/SUMMARY.md.

## Spec contradictions C1–C21

**All 21 accepted.** Apply every edit to spec/SPEC.md, citing the research doc
that motivated it. Notes on specific items:

- C1: the spec must name BOTH round-trip properties (byte-identical to input for
  KiCad-authored files; semantic identity in general) and state which gates
  merges — see Q1.
- C2: version floor is KiCad 10.0.5 / format `20260306`. See Q2 for the ceiling.
- C4: kicli defines its own exit codes and always translates kicad-cli's; raw
  pass-through is forbidden.
- C14: the router's emit-a-label distance threshold and the linter's long-wire
  rule read the same config key. One knob.
- C15/C25: default shared-library layout changes to match reality — sibling
  `../shared` submodule + existing library nickname. `libs/parts` is dropped.
- C17: reword the differentiation claim — the gap kicli fills is full
  manipulation parity + deterministic scoring + context-efficient views, not
  "nobody can move field text" (kct sch tidy exists).
- C18: add a short "prior art / why not X" section to the spec covering
  kicad-tools and kicad-sch-api, grounded in ecosystem.md.
- C21: GPL demo/qa files are an external test corpus fetched by `cargo xtask
  corpus` at a pinned tag into target/ — never vendored into the repo (see Q5).

## M1 blockers

| Q | Decision |
|---|---|
| 1 | Merge gates: (a) semantic round-trip AND (b) byte-identical for KiCad-authored input. (c) "identical to what KiCad would write" is tracked informationally, not gating. |
| 2 | Yes — refuse to write files whose format stamp exceeds kicli's known maximum; config knob to override. |
| 3 | Non-canonical input: reformat, and flag `"reformatted": true` in structured output. Files containing `#` comments: refuse to write unless `--allow-comment-loss`. |
| 4 | Yes — preserve the input's prettifier mode (compact stays compact). |
| 5 | Yes — `cargo xtask corpus` fetches KiCad demos/qa at a pinned tag into target/. KiCad's files never enter the repo. **Unchanged by the GPL-3.0-or-later relicensing (2026-08-13):** vendoring them is now permitted, but the fetch keeps the repository small and the fixtures purpose-built, which is why it was worth having anyway. |
| 6 | Yes — embedded files/fonts are opaque bytes, never re-encoded; refuse any op that would move them between files. |
| 7 | Confirmed — variants: round-trip preservation only, no variant-aware editing in v1. Multi-top-level hierarchies: readable, but `--sheet` assumes a single root. |
| 8 | ~~Confirmed — MPL-2.0 dependencies are out under Constitution §9 (so `via-kicad-sexp` is out; `resvg` is fine as MIT/Apache).~~ **DISSOLVED** by the GPL-3.0-or-later relicensing (2026-08-13). MPL-2.0 is GPL-3-compatible and is now on the allowlist, so the question no longer has a subject. `via-kicad-sexp` is no longer excluded on licence grounds; it stays unused on the technical grounds in `sexpr-strategy.md` §3. |

## M2 blockers

| Q | Decision |
|---|---|
| 9 | Yes — the blocking off-grid rule applies to connectable geometry only; field/graphic text is exempt (KiCad's own autoplacement must pass). |
| 10 | ~~Build our own advance-width table measured from KiCad's SVG `textLength`. No GPL Newstroke vendoring.~~ **AMENDED 2026-08-13, after the relicensing:** port KiCad's own measurement logic instead — the stroke-font advance loop, the `INTER_CHAR` term and `GetTextBox` — with glyph advances derived from Newstroke (GPL-2.0-or-later, attribution preserved). A port is exact by construction; a fitted table is exact only where it was sampled. **The SVG measurement is retained as the validation oracle**, and empirical calibration is the fallback if the port and the measurement disagree by a stable linear term. Validation against IPC `GetTextExtents` still follows when the M9 IPC client exists. |
| 11 | Yes — `sch view` defaults to whole-project within a byte budget, falling back to index + per-sheet summaries; output states which mode was used. |
| 12 | Yes, synthetic stable `n<k>` names as primary — AMENDED: views also carry KiCad's current net name (e.g. `Net-(R1-Pad1)`) as an attribute, so agents can correlate with ERC output and the GUI. |
| 13 | SHA-256 truncated for snapshot hashes. Power symbols suppressed from symbol lists (`--include-power` to show). `view --stats` reports bytes only. |

## M3 blocker

| Q | Decision |
|---|---|
| 14 | Yes to both — fields move rigidly with symbol move/rotate keeping their angles; kicli always clears `fields_autoplaced` when explicitly setting a field position. |

## M4 blockers

| Q | Decision |
|---|---|
| 15 | Router defaults accepted: `w_turn = 6`, obstacle margin 8 grid units. Tune later via the Q17 calibration test if needed. |
| 16 | Yes — never create four-way junctions; offset by 1 grid unit and report the adjustment. |
| 17 | Calibration test approved: re-route every net of a known-good sheet, total cost within **15%** of original. |

## M5 blockers

| Q | Decision |
|---|---|
| 18 | The pre-research draft catalogue is retired. `research/style-rules.md` (R8) is the canonical rule spec. No reconciliation work. |
| 19 | Yes — `sch score --gate` may require `kicad-cli` (ERC-owned Tier 1 checks are layered, never re-implemented). Absence of kicad-cli = structured error per Q31. |
| 20 | Confirmed — score = `100 · exp(−penalty/25)` with density normalisation specified per C9. Tier 1 failures fail the gate independently and do not additionally reduce the score. |
| 21 | Power-direction rule name lists: defaults must cover standard Eurorack — positive {+12V, +5V}, ground/negative {GND, −12V} — plus common aliases (AGND, DGND, VSS, VEE). Per-project override in kicli.toml. Defaults suffice for James's projects. |
| 22 | Skip the Greenberg video. Published text sources only. |

## M6 blockers

| Q | Decision |
|---|---|
| 23 | Default render style: black-and-white when `--output json` (machine consumer), KiCad colour theme when human-invoked or `--style color`. Emit both SVG and PNG. |
| 24 | Confirmed — `render.max_px = 1600`, 6 px/mm legibility floor, SVG cache under `.kicli/render/` keyed on content hash. |

## M7 blockers

| Q | Decision |
|---|---|
| 25 | Yes — adopt sibling `../shared` submodule + James's existing library nickname as the default layout convention. |
| 26 | Report-only for `${KICAD9_3DMODEL_DIR}`-style env-var references. No `migrate-envvars` command in v1 (the submodule is shared across projects). |
| 27 | Warn only on the space in the shared-library nickname. No rename tooling. |
| 28 | Vendor-up conflicts (`--into shared` would overwrite a differing part): refuse with a diff summary. 3D models: copy for `--into project`, reference for `--into shared`. |

## M9 blockers

| Q | Decision |
|---|---|
| 29 | ~~Depend on `kicad-ipc-rs` (MIT, checked-in generated code) for the IPC client. No proto vendoring; upstream licensing outreach deferred indefinitely.~~ **DISSOLVED** by the GPL-3.0-or-later relicensing (2026-08-13). KiCad's GPL-3 `.proto` files may now be vendored or generated from directly, and no upstream outreach is needed. `kicad-ipc-rs` remains the default for M9 because checked-in generated code is less build machinery, not because of its licence; vendoring the protos is a free choice if it turns out cleaner. |
| 30 | Yes — minimum KiCad 10.0.0 for `pcb` commands, verified via `GetVersion` at connect. |

## Cross-cutting

| Q | Decision |
|---|---|
| 31 | Yes — `kicad-cli` is an optional dependency; commands needing it fail with a structured error naming the missing binary and the install hint. Nothing is bundled. |
| 32 | Relabel-only in v1 — kicli maps ERC severities for its own output but never edits `.kicad_pro`. |
| 33 | Hide `--variant` in v1 (consistent with Q7: variants are round-trip-only). |
| 34 | Yes, AMENDED: best-effort only — attempt IPC `GetOpenDocuments`, warn if the target document is open in a running KiCad, stay silent on any connection failure. Must never slow or break the no-KiCad-running case. |
| 35 | No upstream bug filing. kicli must never consume `kicad-cli sch erc --format json` coordinates as-is: use the text report, or apply a sanity-checked 100× correction — either way with a CANARY TEST that expects the bug, so an upstream fix breaks the test loudly and the workaround is removed rather than double-correcting. Code comment links the offending source line (erc_report.cpp:161). |
