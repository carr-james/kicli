# Consolidated report — M5 checkpoint 1

**Session: the M5 plan is ratified; Phase 0 closes and Phase 1 runs.**
Per `.claude/skills/consolidated-report/SKILL.md`. Maintained per tick.

**Status: IN PROGRESS.**

The `/goal`: Phase 0 closed (opening-1 executed under its ruled owner, the
obstacle-walk chore's check landed, opening-3 done) and Phase 1 complete — the
rule identity and registration seam built with its mechanical check answered
honestly, the scoring engine's spine, and the Q1/Q5 research recorded as
PROPOSED entries awaiting James; six gates green; oracle 35/35 zero-skip. **STOP
at the seam verdict and the research proposals: Phase 2 is not dispatched until
James ratifies both.**

---

## 0. The rulings, applied before any dispatch

Provenance for all seven: **James's ratification and advisor rulings, M5 plan
review.** Applied to the record first, per the orchestrator definition's
session-start rule, and committed as `1112a87` before a single lane was
briefed.

| # | Ruling | Where it landed |
|---|---|---|
| 1 | Plan RATIFIED, amended: **Phase 3 opens at TWO lanes**, widening only on frictionless first merges | `PLAN.md` header, a new Phase 3 amendment section, and the lane table — which now carries the opening two-lane cut **and the pre-drawn seams** the phase would split along |
| 2 | James's intent is the milestone's **north star**, verbatim | `RULES.md`, at the top, above everything |
| 3 | The MIR lint is **OUT of M5**, backlogged | `carried-4-handle-lint.md`, with all four reasons and which one decided it |
| 4 | Q3/Q4 confirmed; Q2 closed by citation; **Q1+Q5 → one Phase 1 research task** | `PLAN.md` question 4; new entry `phase1-t5-…` |
| 5 | The freeze lift is the **orchestrator's**, in the main checkout | `opening-1`'s procedure re-worded; `CLAUDE.md`; `orchestrator.md`; and executed, `d4c0eb8` |
| 6 | The obstacle-walk check is a **CHORE**; third triage class added | `chore-7-…`; `mutation-run` skill; `PLAN.md` question 3 |
| 7 | The ten opening PROPOSED items **promoted** | see below |

### The north star, because everything else in this milestone answers to it

> The tool must validate the important aspects of quality schematics. It must
> never reward a schematic that is impossible to read and understand.

Recorded verbatim at the top of `RULES.md` with the rule that value-level
scoring judgements are **parked as PROPOSED or BLOCKED against this sentence,
not guessed.** It is already load-bearing rather than decorative: it is the
stated reason `carried-4` was cut (ruling 3's fourth reason), and it is written
into the T3 and T4 entries as the specific exposure each must report on.

### Ruling 2's supersession, flagged rather than absorbed

**This is the one item in the batch that needs James's eye.** The M4-close
ruling that created `opening-1` said, verbatim:

> Restore the path in the same commit […] All of it is one commit. A commit
> where the freeze is lifted and not restored must never exist on the branch.

Ruling 2 says *lift before dispatch, restore after merge, **both committed***.
Two actors in two trees cannot share a commit, so **the new ruling necessarily
creates exactly the commit the old one forbade** — and `d4c0eb8` is that commit.

Treated as a supersession rather than a governing-document conflict: later
ruling, same author, aimed specifically at this mechanism. **Not treated as
BLOCKED**, because CLAUDE.md's BLOCKED rule is about conflicts *between
documents*, and this is one person's later word on his own earlier one.
Recorded in three places (`opening-1`, `CLAUDE.md`, the lift commit message)
and flagged here, because **a superseded rule that nobody noticed being
superseded is how a rule quietly stops meaning anything.**

The window is narrowed to what it must be: the lift commit touches
`frozen-paths.txt` and nothing else, and the restore is the first commit after
the merge.

### The ten PROPOSED items, promoted

| # | Item | Applied as |
|---|---|---|
| 1 | the frozen-surface hook is bypassable by Bash | `CLAUDE.md` now says **guarded**, not enforced, with the open door named. The other honest option — extend the matcher to `Bash` — **remains open and is real work**; this is the cheap reversible half and is labelled as such |
| 2 | `cargo` not on `PATH`, said by hand in three briefs | one line each in `lane-implementer.md` and `chore-runner.md` |
| 3 | "reachable, correct, untested" had no label | third triage class in the `mutation-run` skill; the general rule (*offer three verdicts wherever you offer two*) in `orchestrator.md` |
| 4 | `falsification-control` grew 198 → 336 lines | **NOT applied — carried to the advisor**, per its own recommendation. It asks which worked examples earn their place, and every one was paid for by an incident. Not the orchestrator's call alone |
| 5 | the merged check ran on a live tree | `orchestrator.md`: the record commit precedes the check |
| 6 | obstacle walk: task or chore? | ruled — chore |
| 7 | dogfood defect 8 confirmed environmental | no action needed; the standing "clean shell environment" instruction is still unmet and that is the useful half |
| 8 | nothing reproduces the `AGENT.md` example drawings | filed as `chore-8-agent-example-recipe.md`, scheduled before Phase 2 |
| 9 | a brief's scope and its completion check contradicted | `orchestrator.md`: a brief's completion check must run everything its scope permits |
| 10 | the orchestrator's shell cwd persisted into a lane worktree, and a merge ran there | `orchestrator.md`, new section: name the checkout (`git -C <root>`) every time; **confirm a merge by reading the merge commit, not `HEAD`** |

Four of the ten (3, 5, 9, 10) are the orchestrator's own defects, self-filed.
**Item 10 acquired a sequel within the hour** — see Coordination, below.

---

## 1. Per tick

*Appended as ticks land.*

| Task | Lane | What landed | Evidence | Verdict |
|---|---|---|---|---|

---

## 2. Findings, attributed

---

## 3. Reviewer rejections

---

## 4. Dogfood

Nothing this stop. The dogfood gate is a **milestone-exit** gate and M5 has not
shipped an agent-facing command yet; `sch score` is what it will be run against.

---

## 5. PROPOSED items

---

## 6. BLOCKED items

---

## 7. Workflow retrospective

### 1. Score

### 2. Verification integrity

### 3. Record quality

### 4. Coordination

### 5. Layer and tooling

### 6. Budget

### 7. User signal

---

## 8. The stop
