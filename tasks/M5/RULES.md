# M5 — Scoring: the milestone's rules

## The north star

**James's intent, recorded verbatim. Provenance: James's ratification and
advisor rulings, M5 plan review.**

> The tool must validate the important aspects of quality schematics. It must
> never reward a schematic that is impossible to read and understand.

**This sentence is the milestone's north star and its tie-breaker.** A
value-level scoring judgement — what counts as a rule, what a weight should be,
whether a finding is worth an agent's attention — is **parked as PROPOSED or
BLOCKED against this sentence, not guessed.** The second half is the sharper
half: a drawing that is unreadable must not be able to score well, and any
result where it does is a defect in the scorer regardless of which rules fired.


**Status: the milestone is OPEN and the plan is RATIFIED.** Provenance: James's
ratification and advisor rulings, M5 plan review. `PLAN.md` is binding, with one
amendment recorded there — **Phase 3 opens at two lanes**, widening mid-phase
only on evidence of frictionless first merges.

The three `opening-*` entries remain the milestone-boundary work; the plan's own
phases are now dispatchable in phase order. **Phase 2 is still gated**: no Phase
2 lane is dispatched until James ratifies the Phase 1 seam verdict and the Q1/Q5
research proposals at the checkpoint.

The plan review also ruled the five plan questions; each ruling is recorded at
its question in `PLAN.md`, and the two with consequences outside that file are:

- **the MIR lint (`carried-4-handle-lint.md`) is OUT of M5**, backlogged, with
  the reason recorded in its entry;
- **the obstacle walk's missing check is a CHORE**, not a task
  (`chore-7-obstacle-walk-check.md`), because the measurement dissolved the
  possible-defect proxy the earlier ruling's words conditioned on.

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

---

## Checkpoint 1's rulings, and what they bind from here

**Provenance for all five: James's rulings (BLOCKED 1, 2, 3; seam; Q1/Q5) and
advisor rulings, checkpoint 1 review.** Recorded before any Phase 2 lane was
dispatched, per the orchestrator definition's session-start rule.

### Phase 2 is UNGATED. The seam verdict is ratified.

`PLAN.md`'s gate read: *"no Phase 2 lane is dispatched until James ratifies both
the seam verdict and the T5 research proposals."* **Both are ratified.** The
seam verdict **PASS** stands, and with it the finding it carries: *adding a rule
is one new file under `crates/kicli/src/lint/rules/` and no edit to any existing
file.* **The lane table is confirmed as cut.**

The cost the PASS carries is ratified with it and is not forgotten: `cargo fmt
--check` **cannot see a rule file at all**, because `rustfmt` walks the module
tree from the crate root and does not follow an `include!`. One of the six gates
lost its reach over `src/lint/rules/`, repaired by
`crates/kicli/tests/rule_files_are_formatted.rs`. **A rule author's file is
formatted by that check and by nothing else.**

### A rule declares its saturation. Every rule, from here.

**BLOCKED 3 ruled — option 2.** A rule that fires at or above the **saturation
fraction** of the objects it counts becomes a **BLOCKING** finding, on §11.5's
existing tier mechanism. No weight moves; no formula changes.

This is a **first-class rule property**, built by T4 and declared by every rule
author after it, precisely so it is not a retrofit. A rule declares **what it
counts** (its denominator) and **the fraction at which it saturates**. The
fraction's provisional value is **1/2**, recorded as provisional, and **Phase 4
is explicitly authorised to move it on measurement.**

The measurement it answers: a sheet on which **every wire crosses another**
scores **67**, at ten wires and at ten thousand, because a normalised rule that
fires at most once per object it counts is capped at `w · reference` whatever
the sheet's size. **67 is not a punishment and that sheet is not readable.** The
ruling's own stated provenance is the north star's second sentence: *a drawing
that fails the gate is not rewarded whatever it scores.*

### The power-name lists live in ONE file, and it is not this milestone's to argue with

**BLOCKED 1 ruled — option 1.** `research/power-name-catalogue.md` is RATIFIED:
measured from KiCad 10.0.5's library, twelve down-pointing symbols, the `Earth`
case ruling, case-insensitive matching, the `PWR_FLAG` exemption, and **no
positive list**. It **supersedes both** previous document lists. `spec/SPEC.md`
§11.4 and `research/style-rules.md` §4 now cite it and state no list of their
own.

**The reason it was the highest-value change and not a tidiness item**, because
a rule author needs to know it: `KI-FLOW-001` defines positive as the
*complement* of the ground set, so **a name missing from the list is a symbol
checked BACKWARDS** — a guaranteed false finding on a correct drawing, not a
silent gap. Nine of the twelve were missing.

### The catalogue's provenance is measured, and six citations are owed a repair

**Q1 and Q5 are RATIFIED as measured.** With them, the numbers: 4 of 28 rules
rest on nothing published; **6 carry a citation the source does not support**,
two of those **inverted** — the cited source recommends what the rule
penalises; 10 gain a source they never had; and **24 of 28 have no source
support for their TIER.**

**The six repairs are owed, and the schedule is ruled:**

| Rule | The defect | Repaired |
|---|---|---|
| `KI-JCT-001` | the Greenberg citation is absent from his checklist | by the KI-DOC rebuild's re-derivation |
| `KI-TXT-002` | the "mono-PDF legibility item from Greenberg" does not exist | by the KI-DOC rebuild's re-derivation |
| `KI-LAY-002` | Greenberg's *"one page, one idea"* framing is not in Greenberg | by the KI-DOC rebuild's re-derivation |
| `KI-DNP-001` | **inverted** — the checklist says *"add as Do Not Place (DNP) components if you want them to show up in the BOM"* | by the KI-DOC rebuild's re-derivation |
| `KI-LBL-001` | both citations fail; its real source is Sutherland §6 | **as the rule is written**, in Phase 3 |
| `KI-SYM-001` | **contradicted** — Lathrop names rotated resistors as normal practice | **as the rule is written**, in Phase 3 |

**Each repair is recorded at the moment it is made**, in the entry of the task
that makes it. A citation quietly deleted is indistinguishable from one that was
never there.

> **Recorded discrepancy, not resolved.** The ruling says *"two via the KI-DOC
> rebuild; the rest as the affected rules are written"*. **Four** of the six are
> Greenberg attributions, and the KI-DOC rebuild is the task that reads the
> Greenberg checklist end to end — so scoping it to re-derive *every* Greenberg
> attribution covers four rather than two, at no extra cost, because it is
> reading the whole document either way. Recorded here rather than absorbed: if
> a narrower pairing was meant, this is one line of the KI-DOC entry to change.

### Tier is asserted almost everywhere, and Phase 3 is where that bites

24 of 28 tiers have no source support. A rule with no published source is a rule
this project chose to have — legitimate, merely undeclared. **A tier with no
support is a claim about whether a drawing is shippable**, and Tier 1 blocks a
build. **The north star is the sentence a tier is argued from**, and a Tier 1
rule whose entry does not argue it from that sentence is incomplete.
