# Consolidated report — M5 checkpoint 2

**Session: the checkpoint's rulings are applied and Phase 2 runs.**
Per `.claude/skills/consolidated-report/SKILL.md`. Maintained per tick.

**Status: IN PROGRESS.**

The `/goal`: Phase 2 complete — the six Tier 1 blocking rules, T2, and T4
carrying the saturating-rule-blocks property, each ticked with recorded reviewer
verdicts; the ratified ground catalogue landed with `spec/SPEC.md` §11.4 and
`research/style-rules.md` §4 corrected to cite it; the KI-DOC rebuild task
drafted for Phase 3 with its source snapshot vendored; six gates green; oracle
35/35 zero-skip; report current per the seven areas — **or a BLOCKED item
requiring James's or the advisor's input.** **STOP before Phase 3 lanes are
dispatched.**

---

## 0. The rulings, applied before any dispatch

Provenance for all five: **James's rulings (BLOCKED 1, 2, 3; seam; Q1/Q5) and
advisor rulings, checkpoint 1 review.** Applied to the record first, per the
orchestrator definition's session-start rule, and committed as `78c4a21` and
`c585706` before a single Phase 2 lane was briefed.

| # | Ruling | Where it landed |
|---|---|---|
| 1 | **BLOCKED 3 — option 2.** A saturating rule becomes a BLOCKING finding. No weight moves, no formula changes; saturation is a first-class rule property; the fraction is provisional and Phase 4 may move it | `phase1-t4-tier-separation.md`'s AMENDMENT section, `RULES.md`, and `spec/SPEC.md` §11.5. Dispatched in T4's brief |
| 2 | **BLOCKED 1 — option 1.** T5's measured ground catalogue is RATIFIED and supersedes both document lists | **`research/power-name-catalogue.md`** — new, canonical. `spec/SPEC.md` §11.4 and `research/style-rules.md` §4 corrected to cite it **in the same commit**, per the ruling |
| 3 | **BLOCKED 2 — option 1, scoped small.** `KI-DOC-001…004` rebuilds from the published checklist as a Phase 3 task; the retrieval is snapshotted, dated, with source URL | `research/snapshots/` — vendored by `lane-snap`. The Phase 3 entry is drafted from its report |
| 4 | **Seam PASS ratified; Q1/Q5 ratified as measured.** The lane table is confirmed as cut. Six unsupported citations repaired per T5's findings | `PLAN.md` (gate lifted, state table trued), `RULES.md` (the repair schedule, with one discrepancy recorded rather than absorbed) |
| 5 | **The fourteen PROPOSED items promoted** per their recorded recommendations; 1 and 2 as ONE chore-runner amendment | see below |

### Ruling 2, because it is the one that changes what the linter calls wrong

`research/power-name-catalogue.md` is now **the only** place a power name is
written down. Both governing documents cite it and state no list of their own —
which is the shape BLOCKED 1 asked for, since the defect was never that one list
was stale but that **`spec/SPEC.md` §11.4 declared `research/style-rules.md` §4
canonical while stating a different list in its own text.** There was no reading
under which both were satisfied.

**The ruling did not pick a winner. It ratified a measurement**, which makes both
previous lists obsolete rather than making one lose.

**Why it was the highest-value change and not a tidiness item**, restated here
because a Phase 3 rule author needs it: `KI-FLOW-001` defines positive as the
**complement** of the ground set, so a name missing from the list is **not** an
unchecked symbol — it is a symbol **checked backwards**, producing a guaranteed
false finding on a correct drawing. **Nine of the twelve names KiCad actually
draws pointing down were missing.** That is the north star's expensive error.

The catalogue also carries three things neither document had: the **`Earth` case
ruling** (§11.4 wrote `EARTH`, KiCad ships `Earth`, and **neither document said
anything about case at all** — the behaviour would have fallen out of whichever
comparison an implementer happened to type); the **`PWR_FLAG` exemption**; and
the **deliberate canon-versus-library conflict**, recorded where the rule lives
so that a later lane or dogfood run cannot meet the finding, read it as a false
positive, and "fix" it.

### Ruling 5's fourteen, and where each went

| # | Item | Applied as |
|---|---|---|
| 1+2 | `chore-runner.md` has no worktree section; chores lack the base-verification first action | **ONE amendment**, per the ruling. Pinned path, `git -C <pinned path>` explicitly, and **the output PASTED in the report — the paste is the proof** |
| 3 | briefs carrying absolute paths | already applied; now written into `orchestrator.md` as applied-not-ruled |
| 4 | the `git -C` rule should be a prohibition on `cd` | `orchestrator.md`, reworded. The old rule was violated **within the hour, while obeying its letter** |
| 5 | `xtask check` reports "all gates passed" without running the corpus arm | **an entry and a lane** — `chore-9-gate-enumerates-its-arms.md`. See the deviation below |
| 6 | a tick reviewer's scratchpad is not private | `tick-reviewer.md`: the scratchpad is **contaminated**; fetch every source yourself under names you choose |
| 7 | the skill should say `--no-fail-fast` | `falsification-control` Procedure step 3 |
| 8 | a brief that lists the guards says where there are none | `orchestrator.md` |
| 9 | `probe_harness_has_one_home` classifies by name, not content | **partially held — see PROPOSED, below.** The recommendation named two directions and settled neither |
| 10 | three Tier 2 families are in no normaliser row | `spec/SPEC.md` §11.5 — the strict default is now stated rather than implied |
| 11 | the normaliser belongs on the rule | T4's brief |
| 12 | a brief's check required what its scope forbade | `orchestrator.md`, as the **symmetric** form. This is 9 inverted, and the earlier wording could not catch it |
| 13 | §11.5's justifying example does not exercise its own mechanism | `spec/SPEC.md` §11.5, corrected to two examples its own table produces, with the error recorded rather than silently fixed |
| 14 | a shared probe name is a flake no isolated run can show | `falsification-control`, as a **fifth kind of blind instrument** — worse than the four, because the ordinary debugging move makes the symptom vanish |

**One deviation, flagged rather than absorbed.** PROPOSED 5's recorded
recommendation was an **unconditional** second arm. `cargo xtask check` **is the
git pre-commit hook**, and the corpus arm is ~16 minutes — so unconditional puts
sixteen minutes on **every commit in the repository**. The entry takes the
recommendation's second arm **and** its own named cheaper third option together:
**the arm is opt-in, the enumeration is unconditional.** If James wants it
unconditional, that is one boolean.

**And one routing decision, recorded so it does not read as a slip:** PROPOSED 5
went to `lane-implementer`, not `chore-runner`, because the shape of the fix is
a design decision and `chore-runner.md` says *never design work*.

---

## 1. Per tick

*Appended as ticks land.*

| Task | Lane | What landed | Evidence | Verdict |
|---|---|---|---|---|

---

## 2. Findings, attributed

*Appended as lanes report.*

---

## 3. Reviewer rejections

*None yet.*

---

## 4. Dogfood

Nothing this stop. The dogfood gate is a **milestone-exit** gate; `sch score` is
what it will be run against, and it is not shippable until Phase 2 closes.

---

## 5. PROPOSED items

*Appended as they arise.*

---

## 6. BLOCKED items

*None yet this stop.*

---

## 7. Workflow retrospective

*Written at the stop, from the notes gathered per tick.*
