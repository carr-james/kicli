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
| the obstacle walk's missing check (`chore-7-obstacle-walk-check.md`) | `lane-c7` | one test in `route_obstacles.rs`, **no source change** — the third triage class's first worked example | entry "Tick — APPROVE"; lane `94eae37`; merge `5fede1b` | **APPROVE** |

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

**1. `chore-runner.md` has no worktree section, and chores are dispatched into
worktrees.** The full measurement is in Coordination, above. `lane-implementer.md`
devotes a section to the pinned path and ends *"do not write to the main
checkout"*; `chore-runner.md` is eighteen lines and does not mention worktrees.
A chore's relative paths therefore resolve to the session's default working
directory — the main checkout — unless the brief's prose wins, and this stop is
the evidence that prose does not reliably win.

*Recommendation: accept — give `chore-runner.md` the same base-verification and
pinned-path section `lane-implementer.md` has, worded for a chore.* **Not
applied**, agent definitions change by ruling.

**2. The base-verification first action is a lane rule that chores do not have,
and it would have caught this in one command.** `lane-implementer.md` requires
`git log --oneline -1` and `git status --porcelain` **in the pinned path** as the
first action. Run in the wrong tree, that command answers about the wrong tree —
but a chore made to run it *with an explicit `-C <pinned path>`* and to **paste
the result** cannot silently be somewhere else, because the paste is the proof.

CLAUDE.md already calls base verification *"load-bearing rather than defensive"*
on three saves in three dispatches. **This is the fourth save, from a failure
mode the rule was not written for**, and it is worth noting that the rule caught
it *by accident* — the orchestrator found this through a failed gate on an
unrelated commit, not through the check.

*Recommendation: accept, together with item 1 — one section, both effects.*
**Not applied.**

**4. The `git -C` rule should be a prohibition on `cd`, not an instruction to
name the checkout.** The orchestrator violated its own newly-promoted rule
within the hour, obeying its letter — see Layer and tooling. An instruction to
"name the checkout" is satisfied by naming it in *most* commands; a prohibition
on `cd` has no such gap. *Recommendation: accept — reword the rule in
`orchestrator.md` to forbid `cd` outside the main checkout outright, and note
that the same fact has now bitten in three directions in one session.* **Not
applied**, agent definitions change by ruling.

**3. The orchestrator's briefs will carry absolute paths from now on, and that
part is applied.** Brief prose is the orchestrator's own instrument and changes
without a ruling. The lesson: **an instruction to stay somewhere is weaker than
every command in the brief naming where.** The four briefs this session all said
"your pinned path is your whole world"; the one whose *example commands* used
bare relative paths is the one that went astray. Applied to the re-dispatch
immediately.

---

## 6. BLOCKED items

---

## 7. Workflow retrospective

### 1. Score

### 2. Verification integrity

### 3. Record quality

### 4. Coordination

**A chore lane wrote into the main checkout, and the root cause is a hole in the
`chore-runner` definition.**

Found by the orchestrator, in the act, at the cheapest possible moment: an
unrelated record commit failed the main checkout's pre-commit gate on a file the
orchestrator had not touched.

The measurement, taken before anything was moved:

| Tree | State |
|---|---|
| main checkout, `crates/kicli/tests/route_obstacles.rs` | **modified, +86 lines** — the chore's new test |
| `.claude/worktrees/lane-c7` | **`git status --porcelain` empty**, still at `d4c0eb8` |

The lane's assigned world was untouched and the lane had never entered it.

**The root cause is the layer, not the lane.** `.claude/agents/lane-implementer.md`
carries a whole section — *"Your base commit, before anything else"* — ending in
*"Your brief's pinned path is your whole world: do not `cd` out of it, and do not
write to the main checkout."* **`.claude/agents/chore-runner.md` is eighteen
lines and does not mention worktrees at all.** Chores have been dispatched into
pinned worktrees for two milestones under a definition that has never once said
so, and the session's default working directory is the main checkout — so every
relative path in a chore resolves there unless the brief's prose overrides it.
The brief did say it. Prose in a brief lost to the default in the harness.

**This is PROPOSED 10's sequel, from the other side.** That item was the
orchestrator's own shell working directory persisting into a lane worktree; this
is a lane's persisting into the orchestrator's. The same underlying fact — **the
Bash tool's working directory persists and is nobody's declared intent** — bit
twice in one session, in opposite directions, and the rule promoted this morning
(*name the checkout explicitly, `git -C <root>`, every time*) was written for
only one of them.

**Resolution: relocation, not redo.** The lane was stopped mid-flight, told
explicitly not to `git checkout`/`restore`/`stash` anything (which would have
destroyed its own work), and given the copy-then-restore sequence. Confirmed
after: main checkout's `crates/` is clean, `lane-c7` carries the work.
**Nothing was lost and nothing was committed to the wrong branch.**

**The reversal trigger did NOT fire, and the reason matters.** CLAUDE.md's
manual-worktree-flow trigger governs *undisclosed scope excess*. This was not
scope excess — the lane wrote exactly the file its brief named, in the wrong
tree — and the lane was told to disclose it in its final message before it
could have chosen not to. Recorded here rather than absorbed, because the
trigger's boundary is only useful if the cases that fall *outside* it are
written down too.

*See PROPOSED, below, for the two fixes this is owed.*

### 5. Layer and tooling

**WORKFLOW NOTE, chore 7 (`lane-c7`), verbatim:**

> *"The Write tool accepted `/Users/james/code/kicli/crates/kicli/tests/route_obstacles.rs` without path resolution or validation. The brief specified the worktree path only once in "Your final message" at the end and did not name the file path explicitly. The tool should have either (a) rejected the path as outside the worktree, or (b) the brief should have prefixed every file path with the worktree. The incident is correctable: a one-time notice to the Write tool or a pattern in the brief wording would prevent recurrence."*

**Correction, beside the quote and not folded into it — the note's factual claim
about the brief is wrong, and its recommendation is right.** The brief named the
pinned worktree **four times**, not once, and none of the four was in the final
paragraph:

1. the opening sentence — *"in the pinned worktree
   `/Users/james/code/kicli/.claude/worktrees/lane-c7`"*;
2. and 3. twice more, as **absolute paths inside the first-action commands**
   (`git -C /Users/james/code/kicli/.claude/worktrees/lane-c7 …`);
4. *"That path is your whole world; do not `cd` out of it and do not write to
   the main checkout."*

**But option (b) is exactly right and is the real finding.** The one place the
brief used **repo-relative** paths was the write-scope list —
`crates/kicli/tests/route_obstacles.rs` — and *that* is the path the lane
resolved, against the session's default working directory. The brief said "stay
here" four times in prose and then handed over an ambiguous path in the only
list the lane was going to act from.

**The lane's diagnosis of its own failure is better than its account of the
brief.** Worth recording as a fact about these notes: a subagent reconstructing
what it *read* is unreliable; its account of what it *did* is not. Take the
recommendation, discount the recollection.

Option (a) — a tool-level path guard — is **not** recommended. CLAUDE.md: *"This
is the only tool hook; further hooks require a triggering incident."* This is a
triggering incident, but the cheaper fix (b) is available and untried, and a
second hook should not be spent on a problem a brief pattern solves. If (b) is
applied and the failure recurs, *that* is the incident that earns the hook.

### The orchestrator committed PROPOSED 10's defect again, within the hour of promoting the rule against it

**Recorded because the timing is the finding.** This morning the orchestrator
promoted into its own definition: *"Name the checkout explicitly. `git -C
<root>`, every time. The Bash tool's working directory persists between calls."*

Reading `lane-c7`'s diff for scope verification, the orchestrator ran a `cd`
into the lane worktree — and the **next** command, a `python3` heredoc writing
this very report, failed with `FileNotFoundError` because it resolved
`tasks/reports/…` against the lane's tree.

**It failed safely, and only by luck of direction**: the report does not exist
in the lane worktree, so the write errored instead of silently creating a second
copy of the session report on a lane branch. Had the file existed there — as
every `tasks/` file the lane did not create does — the write would have
succeeded, in the wrong tree, and the orchestrator would have been maintaining a
report nobody would ever merge.

So this stop has the same underlying fact biting **three times in three
directions**: lane → main (chore 7), orchestrator → lane (this), and the
original PROPOSED 10 (orchestrator → lane, a merge). *Recommendation: the rule
as promoted says "name the checkout" but does not say **never `cd`**, and the
orchestrator obeyed the letter while `cd`-ing anyway. The stronger form is the
prohibition, not the instruction.* **Not applied** — agent definitions change by
ruling — and filed as PROPOSED 4 below.

**And PROPOSED 7 corroborates itself again.** The `zoxide` shell-configuration
warning — dogfood defect 8, triaged as environmental — appeared in the
orchestrator's own shell again this session, in the main checkout. The triage
stands; the standing instruction that the next dogfood run gets *"a clean shell
environment"* is **still unmet**, and this is the third independent sighting.

### 6. Budget

### 7. User signal

---

## 8. The stop
