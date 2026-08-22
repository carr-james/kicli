# M5 — Scoring: the milestone's rules

**Status: the milestone is OPEN, the plan is NOT ratified.** `PLAN.md` in this
directory is a PROPOSAL. Only the three `opening-*` entries are ruled work; they
land the M4-close rulings and the milestone-boundary changes. **No task named in
`PLAN.md` is dispatched until James ratifies it.**

## Why this directory exists rather than one `tasks/M5.md`

**Provenance: advisor recommendation, James-approved, M4 close (boundary
package, item 7).** One file per task, plus this one. `tasks/M4.md` reached
6,143 lines, and every pointer into it — a brief, a review, a ruling — cost a
reader the work of finding the range and trusting they had all of it. Here **an
entry pointer costs what the entry costs**: a path is the whole citation.

The content of the former `tasks/M5.md` was migrated into `carried-*.md` and
`mutation-survivors.md` in this directory. Nothing was dropped; the file itself
is gone, because two homes for one record is how a record goes stale.

## Naming

- `RULES.md` — this file. The milestone's binding header.
- `PLAN.md` — the draft plan, a PROPOSAL until ratified.
- `opening-N-<slug>.md` — the ruled M5-opening tasks.
- `carried-N-<slug>.md` — a decision M4 made and M5 owns. Each is a task
  candidate, not yet a task.
- `chore-N-<slug>.md` — a check-guarded chore.
- `mutation-survivors.md` — the M4-close `cargo-mutants` survivors, as filed.

**A task is named by its role, with its file in parentheses**: "the joined net's
contract field (`opening-1-joined-net-contract.md`)". Carried from M4's rules,
where numbers drifted under edits and a ruling that carried only a number could
be pointed at the wrong task. A path does not drift.

## Rules for this milestone

Inherited and binding, restated because a milestone's rules are read at the
start of every session:

- **Constitution §11**: every task names an executable completion check. A task
  is not done until that exact command passes.
- **Constitution §4**: detection is integer geometry only. Floating point
  appears in `score()`'s final `exp` and nowhere else, with fixed rounding.
- **Constitution §6**: findings and scores are read by an agent under a context
  budget. A view that floods is wrong, whatever it contains.
- **`ENGINEERING.md`**: `cargo xtask check` governs **every commit that reaches
  `main`**. A lane branch may be transiently red only under a written sanction;
  the merge must be green. See `ENGINEERING.md`, "The gate and the lane branch".
- **`CLAUDE.md`**: the orchestrator runs the full check, **corpus included**, at
  every lane merge. Corpus and environment gates never count toward done from
  inside a lane worktree.
- **`ENGINEERING.md`** "Testing pyramid": write the test first. Fixture
  expectations are verified against KiCad, never hand-asserted.
- Every check added is shown capable of failing, per the `falsification-control`
  skill, and the falsification is recorded in the entry.

### The weights arrive already measured, and M5 owns them

M4 was forbidden to retune `w_turn`, `w_cross`, `w_text`, `w_near` and
`label_threshold`, because they are shared with this milestone's rules. **M5 is
the milestone that may move them** — and it inherits two measurements that
constrain how:

- the re-route calibration gate (M4 T20) **measures agreement, not
  calibration**: both sides are costed with the same weights while the router
  optimises that objective, so no perturbation of any weight moves either sheet
  outside ±15 %. Promoted from PROPOSED 9 by James's ruling at the M4 close.
- the sweep that **does** answer the question was run and its numbers are
  recorded in `carried-5-calibration-sweep.md`. `w_turn = 6` is better than 0
  and **not** better than 60; `w_cross` and `w_text` are exercised by neither
  sheet. The defaults are **under-determined rather than wrong**.

A weight is moved in this milestone only with a measurement of that shape
beside it.

### Standing milestone-exit gates

- **Dogfood is a gate, not a dry run.** Provenance: advisor recommendation,
  James-approved, M4 close (boundary package, item 8). A milestone that ships
  agent-facing commands is not done until a naive-agent run has attempted them
  cold and its defect list is triaged. One run per milestone minimum; the
  sandbox rules are the established ones; an occasional haiku-model run is
  permitted as a stress variant. `tasks/dogfood.md` holds the runs.
- **Mutation testing at the close.** Standing from the M4 close, James's ruling.
  Procedure, scoping, the two triage classes and the four counts are in
  `.claude/skills/mutation-run/SKILL.md`. It runs after every task is ticked and
  the gates are green, never before, and never as a per-commit gate.
- **The netlist oracle stays at 35 of 35.** A scorer that reads connectivity
  stands on the extractor, so the extractor stays measured.

### Worked examples in `AGENT.md` are measured output

**Provenance: advisor recommendation, James-approved, M4 close (boundary
package, item 9).** Every example block a session touches is **regenerated from
a real run of the built binary** — not edited by hand to match what the code is
believed to do. D3 of the first dogfood run is the class: `AGENT.md` showed
`+ W 3300f00e (50.80,50.80) -> (63.50,50.80)` while the tool printed
`+ W 906eceb2 180.34,41.91..180.34,46.99`, and a reader trusting the document
misparsed the line. The rule's executable twin is
`opening-3-measured-examples.md`.
