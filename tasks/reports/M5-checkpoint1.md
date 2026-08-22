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

**5. `cargo xtask check` reports "all gates passed" without running the corpus
arm.** Full measurement in Verification integrity, above. `CLAUDE.md` requires
the orchestrator's merged check to be "corpus included"; the command that prints
the summary is a bare `cargo test`, so the 35/35 netlist oracle and every other
`#[cfg(feature = "corpus")]` check are compiled by clippy and never run.

*Recommendation: accept — the `test` gate gains a **second arm**, `cargo test
--features corpus`, reported as its own line in the summary, so the summary
enumerates what ran.* Two sub-options on the environment half, and they differ
in kind:

- the corpus arm can be made unconditional, because `cargo xtask corpus`
  fetches into `target/` and the fetch is already a documented step;
- the `KICLI_TEST_KICAD_CLI` arm **cannot**, because `kicad-cli` genuinely may
  be absent, and that is what `Kicad::found_or_skip` exists for. **What it can
  do is say so in the summary** — a `skipped` line is not a `pass` line, and the
  present output does not distinguish them.

The second half is the more valuable one and generalises past this gate: **a
summary can only report on what it was asked to run, and nothing here states
what a full run would have been.** The corpus tests are not skipped — they are
compiled out by `#[cfg(feature = "corpus")]`, so they are absent rather than
ignored, and the `0 ignored` column is identical with and without them. This is
adjacent to PROPOSED 3's two-verdict hole but is not the same: a two-verdict
brief forces a false answer, whereas this one simply never asks the question.
**Not applied** — `xtask` is a merge hotspot and this is a gate definition.

**Measured cost of accepting it:** the corpus arm is ~16 minutes of wall clock
against the bare gate's seconds, so making it unconditional in `xtask check`
would change what "run the gates before committing" costs at every commit. That
is a real trade and it is James's, not the orchestrator's. **A cheaper third
option worth naming: leave `check` as it is and have it print one line saying
which arms it did NOT run**, which costs nothing and closes the invisibility
without changing the gate's runtime.

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

**Ticked:** 1 — chore 7, APPROVE first time, no rejections.

**Merged:** 1 lane (`lane-c7` → `5fede1b`). Scope verified before the merge
(`git diff --stat d4c0eb8..lane-c7`: exactly the two IN-list files), main
checkout clean before it began, merge confirmed **by reading the merge commit's
two parents** rather than `HEAD` — the rule promoted this morning, used the
first time it applied.

**Gates, at that merge, on a quiescent tree** (record commit `186672f` preceded
the run, per the other rule promoted this morning):

| Run | Result |
|---|---|
| `cargo xtask check` | **6 of 6 pass** — fmt, clippy, test, doc, deny, clean |
| `cargo xtask corpus --verify` | 115 schematics, 36 library tables, **0 not at the pinned stamp**, verified |
| `cargo test -p kicli --features corpus`, `KICLI_TEST_KICAD_CLI=1` | **71 binaries, 535 passed, 0 failed, 0 ignored, 0 skip markers** |
| netlist oracle, `--nocapture` | **`hierarchies matched: 35/35`**, 5 tests, 16.09s |

**The oracle is 35/35 with zero skips**, which is the `/goal`'s clause — and see
Verification integrity for why the obvious way of asking that question returns a
green that means nothing.

**Rejections: none.** **Gate failures found after a tick: none.**

**Parked:** nothing. **Lanes live at the time of writing:** `lane-o1b`
(opening-1), `lane-t1` (the seam), `lane-t5` (the research), each at base
`d4c0eb8`.

### 2. Verification integrity

### `cargo xtask check` prints "all gates passed" without ever running the corpus arm

**Found by the orchestrator this stop, while running the merged check for
chore 7's merge. Measured, not inferred.**

`CLAUDE.md` binds the orchestrator: *"The orchestrator runs the full check,
**corpus included**, at every lane merge."* The command that reports on the
gates does not include it.

`xtask/src/main.rs`, the `GATES` table:

| Gate | Args |
|---|---|
| `clippy` | `clippy --all-targets --all-features -- -D warnings` |
| `test` | **`test`** — bare. No `--features corpus`, no `KICLI_TEST_KICAD_CLI` |

So the corpus code is **compiled** by clippy's `--all-features` and **never
run** by the test gate. The whole `mod corpus` block in `net_oracle.rs` sits
behind `#[cfg(feature = "corpus")]` and is simply absent from the binary the
`test` gate builds.

**How close this came to being reported wrong, this stop:** the orchestrator ran
`cargo test -p kicli --test net_oracle`, got `2 passed; 0 failed`, and **that is
a green**. It was caught only because the run finished in **0.00s** — 35
hierarchies including one with 2,353 nets cannot be checked in no time. Run
correctly:

```
cargo test -p kicli --features corpus --test net_oracle   # + KICLI_TEST_KICAD_CLI=1
hierarchies matched: 35/35
test result: ok. 5 passed; 0 failed; 0 ignored — finished in 16.09s
```

**Five tests, not two, and sixteen seconds, not none.** The `/goal` for this
session asks for "oracle 35/35 zero-skip"; the naive command answers that
question `ok` while measuring nothing.

**The size of the gap, measured both ways, because the honest number is smaller
than the rhetoric wants:**

| Command | Binaries | Tests passed | Ignored |
|---|---|---|---|
| `cargo test -p kicli` (what the gate runs) | 71 | **529** | 0 |
| `cargo test -p kicli --features corpus` + `KICLI_TEST_KICAD_CLI=1` | 71 | **535** | 0 |

**Six tests.** Not a chasm — and stating it as one would be the same overclaim
this finding is about. But those six are the ones that check kicli's answers
**against KiCad itself**, over 115 real schematics and 35 hierarchies, and they
are the only checks in the suite that can catch the extractor drifting from the
thing it models. `RULES.md` makes one of them a standing milestone gate in its
own right.

**And the sharper half is the "0 ignored" column, which is identical in both
rows.** The six missing tests do not show up as skipped, or ignored, or filtered
out. `#[cfg(feature = "corpus")]` removes them from the binary, so **they are
invisible rather than skipped** — the bare run's summary is internally
consistent and completely silent about them. A reader comparing the two rows
cannot tell from either one that the other exists.

*(Correcting a looser claim made earlier in this session: the orchestrator first
described this as a summary "reporting a skip as a success". It is not — nothing
is skipped. The tests are compiled out and the count is honest about what it
ran. The defect is that **nothing anywhere states what a full run would have
been**, which is a harder problem than a mislabelled skip and is why the
recommendation below asks for a second reported arm rather than better wording.)*

**This is the exact failure mode M4's calibration row taught** — *"a gate
presented as measuring something it cannot fail on is worse than no gate, since
it spends the credibility of a real one"* — with the twist that here the gate is
real and the **summary line over it** is what overclaims. `all gates passed` is
true of the gates that ran, and a reader has no way to see which did not.

**The practice was not broken; only the instrument is.** The previous session's
report records running the corpus and environment arms separately ("all corpus
and environment arms green"), so the orchestrator role has been doing this by
hand. **That is precisely why it is worth filing: the correctness depends on the
orchestrator remembering, and nothing fails if one forgets.** The prior report's
own title — *"six of six on a quiet tree"* — names six gates, and six gates is
what `xtask check` reports whether or not corpus ran.

*Filed as PROPOSED 5, below.*

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

**The orchestrator truncated its own evidence with `tail -30` and had to re-run
a 16-minute suite.** The first full corpus-enabled run was piped through
`grep … | tail -30`, so the file kept only the last thirty matching lines. The
run **exited 0** — that fact survived — but the per-binary totals and the skip
markers did not, and those are what the `/goal`'s "oracle 35/35 zero-skip"
clause is answered from.

**This is the report's own "silent cap" rule, broken by the person maintaining
the report.** The rule: *a silent cap reads as full coverage*. A `tail -30` on a
suite of eighty-odd test binaries is a cap, it was silent, and the only reason
it did not become a false claim in this document is that the truncation was
noticed before the sentence was written rather than after.

*Recorded rather than quietly fixed, because the cost is the interesting part:*
the re-run costs ~16 minutes of wall clock and buys nothing that the first run
did not already establish — it buys only the **evidence** of it. That is the
usual price of a truncated instrument and it is why the rule exists.


### 7. User signal

---

## 8. The stop
