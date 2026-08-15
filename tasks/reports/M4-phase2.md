# M4 Phase 2 — consolidated session report

**STATUS: CHECKPOINT 2 IN PROGRESS.** Checkpoint 1 is complete and closed — all
five of its tasks merged and ticked with recorded reviewer verdicts, six gates
green on `main`, the netlist oracle at 35/35 with zero tests skipped, measured at
every merge. Checkpoint 2 opened under the advisor's rulings on the checkpoint-1
review and is the live section: **Session 3 — checkpoint 2**, at the end of this
file. Sessions 1 and 2 are preserved unaltered, because a report that rewrites
its own history is not a record.

Orchestrator sessions opened 2026-08-15. James holds intent; the advisor chat
reviews and rules; the session coordinates the subagents that do the work.
Review happens in batches rather than at task boundaries, so this report and the
task entries in `tasks/M4.md` are the review surface: every tick, PROPOSED item
and finding is recorded with the evidence a retroactive ruling needs.

Appended per tick, not written at the end.

---

# Session 1 — interrupted on a token limit

**STATUS AT ITS STOP: INTERRUPTED.** One task of nine merged, none ticked.

## What holds on `main` at the stop

| | |
|---|---|
| Merged | the delta view (T17), `71bb1d6`, via merge commit on `main` |
| Ticked | **nothing** — no reviewer ran before the limit |
| Six gates on the merged result | pass: fmt, clippy, test, doc, deny, clean |
| `cargo test --features corpus` | pass |
| `KICLI_TEST_KICAD_CLI=1 cargo test --features corpus` | pass, **zero tests skipped**, `corpus::netlist_partition_matches_kicad_corpus` green — the netlist oracle holds at **35 of 35** |
| Not merged | the candidate shapes (T9) and `wire draw` (T14), both mid-task with uncommitted drafts |
| Not dispatched | T10, T11, T12, T13, T15, T16 |

## Orchestrator commits, before any lane work

**`6d5ec92` — the crossing names the wire, and the net is attributed at the
seam.** Ruling R3 transcribed into the contract it came from.
`research/wire-routing.md` §8 reported a crossing as `{ net, at }`; the search
cannot answer that, because the obstacle map knows the **wire** on a cell and
whose net a wire carries is connectivity's answer. The frozen
`route::report::Crossing` carries the wire handle always and
`net: Option<String>` filled by the caller. §8's JSON gains the `wire` key, its
text form names the wire, and `spec/SPEC.md` §9 states the seam rule. No code
changed; six gates green.

**`76e170c` — tick review and the dogfood gate become working practice.** Both
adopted on advisor recommendation, provenance recorded in `CLAUDE.md`. Tick
review: no implementer ticks its own task, the reviewer gets the entry and the
diff rather than the implementer's account, and the verdict is recorded beside
the tick. Dogfood gate: an LLM agent with only `AGENT.md` and the binary attempts
a brief cold, and what it fumbles becomes a defect list in `tasks/dogfood.md`. A
dry run this milestone; it gates nothing.

## Lane assignments

| Lane | Owns | Tasks |
|---|---|---|
| A — the router | `crates/kicli/src/route/**` and its module list, `crates/kicli/tests/route_*` | T9, T10, T11, T12, T13 |
| B — the verbs | `crates/kicli/src/edit/wire.rs` and `edit.rs`'s module list, `crates/kicli/src/cli/edit/wire.rs`, `crates/kicli/src/cli/view.rs`, `cli/args.rs`, `crates/kicli/tests/edit_wire*`, `tests/delta_view.rs` | T14, T15, T16, T17 |

`Cargo.toml`, `lib.rs`, the fixture `MANIFEST` and `tasks/M4.md` stay with the
orchestrator. Each lane owns the module-list line of its own module file, which
are disjoint files (`route.rs` for Lane A, `edit.rs` for Lane B).

### Coordination decisions the lane table did not settle

**`AGENT.md` is designated to one task at a time.** T17 documents the delta verb
and T16 adds the `[routing]` section beside the verbs that read it. Both are
Lane B, so they are sequenced rather than parallel: T17 holds the file first,
T16 takes it after T17 merges.

**The four-end detection crosses the lane boundary, and T12 is where it shows.**
`edit::mark::wire_ends_at` is private today, and T12 requires one implementation
called by both the mark verb and the router — which is a Lane A task reaching
into `crates/kicli/src/edit/mark.rs`. T15 (`wire delete`) also reasons about
wire ends, to report the junctions it stranded. So `edit/mark.rs` is designated
to whichever of T12 and T15 runs first, and **T12 and T15 never run
concurrently**. Recorded here because the lane table assigns files to lanes and
this one file is wanted by both.

**The label proposal is Lane A's, but performing it is Lane B's file.** T13's
router half — the threshold test, the name derivation, the label positions — is
`route/`. Its check `auto_labels_writes_the_pair_and_says_so` needs
`--auto-labels` to actually write, which is the command layer calling the
existing `edit::label` path. PROPOSED, and cheap to reverse: **T13 is sequenced
last in Lane A, after Lane B's verb surface has merged**, and its subagent holds
both the `route/` proposal and the small `--auto-labels` wiring, because by then
no Lane B task is active on that file. The alternative — splitting T13 across
two subagents — would leave a check that neither of them can run.

## Per-tick record

**No ticks. One merge.**

### The delta view (T17), Lane B — merged, not ticked

Landed: `sch view --view delta` wired to machinery that already existed;
`view::delta` and `view::snapshot` were not modified. Evidence in the T17 entry
of `tasks/M4.md`, section "Implemented and merged, 2026-08-15"; commit `71bb1d6`
on branch `worktree-agent-a0ec9c0d048960db0`, merged to `main`. Ten checks, each
shown capable of failing, with the falsification table in the entry.

**Reviewer verdict: none.** The tick-review practice was adopted in this
session's second commit and no reviewer was dispatched before the limit. The
next session dispatches `tick-reviewer` against the entry and the diff, and
ticks only on APPROVE. This is the correct outcome of the practice, not a gap in
it: the work is merged and green, and the tick is a separate claim that has not
been earned yet.

### `wire delete` (T15), Lane B — merged and TICKED

Merged as `5753e4b` from `worktree-agent-ad5ebde9f056ce4cb` (lane commits
`5c06d74`, `871d61e`, `3dfd528`). Merged check green: six gates, corpus,
environment-gated, **zero ignored, oracle 35/35**. Thirteen falsifications
recorded.

**The four-end detection did not fork**, which was the structural risk in
granting this task `edit/mark.rs`. Verified at merge: the diff is exactly
`fn wire_ends_at` → `pub(crate) fn wire_ends_at` plus a doc paragraph recording
*why* it is visible — two questions rest on it and must rest on the same answer.
No second implementation. T12 will find the reason at the function.

**The oracle measurement is made**: KiCad reports one net becoming two across
three joined segments, and the T losing its dropper leaving the crossbar joined
and the stem alone.

### The T14 gap fix — merged, and it sharpened the claim past what the review asked

Merged as `ed14794`. The third arm is in, and the implementer **re-measured
rather than adopting the reviewer's coordinates**; they reproduced exactly. The
new arm is falsified three ways, including one the brief did not ask for: giving
the third arm the *agreeing* angles with stubs left at relocated positions fails
all four ports, which shows the arm depends on the angle and not on geometry
alone.

**Both claims were narrowed to the port's connection point** — a sharper reading
than the review demanded. Connectivity is the only instrument in play, so what
three arms measure is where KiCad *joins* the port, not where it *draws* the
graphic. The distinction was the implementer's, not the reviewer's and not mine.

### A* over turn-aware states (T10), Lane A — merged and TICKED

Merged as `13e3ee0`-era merge commit from `worktree-agent-ab3fce95dc0125eb4`
(lane commit `00402c7`). Merged check green: six gates, corpus,
environment-gated, **zero ignored, oracle 35/35**. Seventeen falsifications
recorded. `Heading::EVERY` moved to `route::terminal` with one order for the
search, the U-shaped candidates and the derived comparison that settles queue
ties — which also settles the shapes task's first PROPOSED item.

**Its tick review was dispatched with a specific instruction, because the implementer reported
unprompted that its own first falsification sweep was contaminated:
`git checkout --` cannot restore a not-yet-committed new file, and with multiple
pathspecs it restores **nothing** when one fails, so an early sweep ran every
break on top of the previous ones. A stacked sweep's readings are worthless — a
check failing with five breaks applied says nothing about which break it detects.
The implementer says it caught this. **The reviewer was told not to settle that
by reading the entry's assurance.**

**Verdict: APPROVE, and the contamination question is answered by measurement.**
Twelve of eighteen rows re-measured in isolation from a checksum-verified scratch
tree. The proof is the *shape* of the results, not any assurance: six rows
claiming nothing happens produce a fully green workspace, and four rows claiming
a narrow failure fail exactly one test in the entire workspace. **A stacked sweep
can produce neither.** The twin check was verified from both homes, and the
reviewer could tell that apart from a decorative check precisely because six
other breaks in the same sweep produced nothing. Two table citations named the
wrong assertion inside the right check and are corrected; both are under-claims,
and **nothing over-claims anywhere**, which is itself evidence against
contamination.

## Per-tick summary at checkpoint 1

| Task | Merged | Tick | Rejections |
|---|---|---|---|
| the delta view (T17) | `71bb1d6` | ✅ APPROVE | 1 — undeclared scope excursion; the lane table was wrong |
| the candidate shapes (T9) | `a0a4e03` | ✅ APPROVE | 0 |
| `wire draw` (T14) | `686aa72` + `ed14794` | ✅ APPROVE | 1 — the claim outran the measurement |
| `wire delete` (T15) | `5753e4b` | ✅ APPROVE | 0 |
| A\* (T10) | `af5c184` | ✅ APPROVE | 0 |

Every merge ran the full check with corpus and the environment-gated run: six
gates, zero ignored, netlist oracle **35/35**, at every one.

## Findings, by lane

**Lane B, the delta view (T17): the task text was wrong about the degraded
form, and the correction is narrower than the entry.** The entry says a
comparison against a **file** "degrades to hashes and names". True before
`spec/SPEC.md` §7.3's 2026-08-13 amendment added the display column; false of a
snapshot this version writes — the existing check
`a_delta_against_a_saved_state_reads_like_one_against_a_design` already shows a
file-based delta reading identically to a design-based one. The real degradation
is a snapshot written **before** the column existed, which carries four columns
and can then say only that an object was edited. The output states which of the
two forms it holds, measured from the state actually read.

**Lane B, the delta view (T17): an anti-vacuity control earned its place the
day it was written.** `a_delta_right_after_a_mutation_reports_nothing_changed`
**passes** against an implementation that reports nothing ever — the break was
made deliberately (`Delta::between(&saved, &saved)`) and the check did not
notice. Its control, the same harness reporting an edit made behind kicli's
back, is what makes the pair evidence. This is the exact failure mode the
falsification rule exists to catch, caught in the act.

**Lane B, the delta view (T17): a budget fallback that costs more than it
saves.** Measured at a 120-byte budget: a two-line delta produced a 238-byte
summary against a 180-byte full form. The fallback is now skipped when the
summary would be larger. `view/scope.rs` has the same latent property for the
index and was left alone as out of scope — recorded as a chore below.

**Orchestrator: the output contract asked the search for something it cannot
know.** `research/wire-routing.md` §8 reported a crossing as `{ net, at }`. The
obstacle map knows the **wire** on a cell; whose net a wire carries is
connectivity's answer. Transcribed in `6d5ec92`: `wire` is always present, `net`
is `Option` filled by the caller at the seam, and `spec/SPEC.md` §9 states the
rule. No code changed — `route::report::Crossing` was already frozen this way.

**Orchestrator: three coordination calls the lane table did not settle**, all
PROPOSED and listed below. The lane table assigns files to lanes; these are the
three files two lanes both want.

## Reviewer rejections

None. No review ran.

## Dogfood defect list

**No run.** The dogfood dry run was gated on Lane B's first routing verb being
usable end to end, which is `wire draw` (T14). That task did not complete, so
there was nothing to attempt a brief against. `tasks/dogfood.md` does not exist
yet and is correctly absent rather than empty.

## PROPOSED items, in entry order

**From the orchestrator, before any lane work:**

1. **`AGENT.md` is held by one task at a time.** The delta view (T17) and the
   verb surface (T16) both write it. Sequenced rather than parallel: T17 first,
   T16 after it merges. Recommendation: accept; it cost nothing and the
   alternative is a merge conflict in a document.
2. **The four-end detection crosses the lane boundary.**
   `edit::mark::wire_ends_at` is private, and the four-way task (T12) requires
   one implementation called by both the mark verb and the router — a Lane A
   task reaching into a Lane B file that `wire delete` (T15) also wants, to
   report the junctions it stranded. Proposed: `edit/mark.rs` is designated to
   whichever of T12 and T15 runs first, and **the two never run concurrently**.
   Recommendation: accept.
3. **The label proposal (T13) is sequenced last in Lane A.** Its router half is
   `route/`, but its check `auto_labels_writes_the_pair_and_says_so` needs
   `--auto-labels` to write, which is the command layer. Proposed: T13 runs
   after Lane B's verb surface has merged, and its implementer holds both files,
   because by then no Lane B task is active. The alternative — splitting T13
   across two subagents — leaves a check neither of them can run.
   Recommendation: accept.

**From the delta view (T17)** — the six in its entry, not repeated here in full:
the exit codes for an absent (1) against a malformed (4) saved state; refusing
`--sheet` when it disagrees with the state; three added assertions in
`tests/agent_doc.rs` that make the entry's third check capable of failing; the
one-line `cli.rs` dispatch change; the budget fallback skip; and the current
state carrying no stamp. Evidence and recommendations are in the T17 entry.

**Chore proposed, not yet written into the chore list:** `view/scope.rs` shares
the "fallback larger than the thing it replaces" property found in the delta
view's budget path. It was out of scope and untouched. It wants the same
measurement.

## BLOCKED items

**None at the stop.** No governing-document conflict was hit, and no lane needed
a change to the frozen surface. The session stopped on a resource limit, which
is not a BLOCKED item and needs no ruling — only more budget.

## Rulings received on this report, and what they changed

The advisor ruled on the interrupted stop. Applied here:

| Ruling | Effect |
|---|---|
| R1 wind-down accepted; merged-and-green vs ticked affirmed as precedent | the delta view (T17) stays merged and unticked; the next session's first act is its tick review |
| R2 drafts are reference, not resumption | added to the `lane-implementer` definition — a fresh implementer starts from the entry, and every adopted line is falsified as if newly written |
| R3 hook refusal message | amended: it names the remedy (put cargo on PATH, install rustup) and never the escape hatch. Both branches re-measured under `env -i` |
| R4, R5, R6 the three coordination calls | PROMOTED, and written into the Phase 2 status section as binding rather than proposed |
| R7 the delta view's six in-entry items | ALL PROMOTED; the entry now records them as ruled, with the note that proposing the two scope excursions was correct |
| R8 the `scope.rs` twin | recorded as chore **C2** in the milestone's chore list, with the chore-runner eligibility condition the ruling attached |
| R12, R13 | affirmed; nothing to change — the absences were already recorded as absences |

**R11's citation rule is adopted and is already in use here**: a section is cited
as role plus document plus number — "the output contract
(`research/wire-routing.md` §8)". The rule is not yet written into a governing
document, for the reason below.

### Two amendments were ruled but arrived without their text

R9 and R10 each say the amendment wording is "in the session opener below", and
R11 adopts a citation rule whose binding home is not named. **No opener or
amendment text arrived with the rulings.** Nothing has been invented in their
place: writing my own wording for a binding governing document, where the author
said they would supply it, is exactly the substitution these documents exist to
prevent.

What is ratified and is being followed in practice, pending its text:

1. **R9 — environment-gated checks in lanes.** The reading is ratified: the rule
   is gate discipline. A lane-local run never counts toward "done"; only the
   orchestrator's merged run does. It was never a ban on a lane making a
   measurement its own task owes. `CLAUDE.md`'s line still reads the old way.
2. **R10 — session grain.** The continuous model wins: work is dispatched,
   merged and recorded at task grain, but the session runs under `/goal` to its
   stop. Two costs are paid — the wind-down procedure becomes standing practice
   in the orchestrator definition, and `/goal` conditions become **checkpoints**
   rather than whole phases, so a budget must survive about half a phase.
   `CLAUDE.md`'s "sessions end at task boundaries" line still reads the old way.
3. **R11 — the citation rule** binds the advisor's own prompts first, and wants
   a home: the milestone's Rules section already carries the task-naming rule it
   extends, which is the natural place.

Until those three land, `CLAUDE.md` contradicts two ratified readings. That is a
governing-document conflict of the kind the BLOCKED rule covers, so it is parked
here rather than resolved by precedence.

## Workflow retrospective

Diagnostic rather than graded. The interruption is itself retrospective data.

**What earned its keep.**

- **`ENGINEERING.md`'s "every check shown capable of failing".** It caught a
  real vacuous check in the delta view within one task — the "nothing changed"
  check passing against an implementation that reports nothing ever. Without the
  rule that check would have shipped as evidence of a property it does not test.
- **The frozen `route::report`, and freezing it *before* the lanes split.** Two
  lanes compiled against one shape all session and neither asked to change it.
  The Phase 1 ruling that writing §8's shape is transcription rather than design
  was correct.
- **`CLAUDE.md`'s "the orchestrator runs the full check, corpus included, at
  every lane merge — not only at milestone end".** It is what makes a
  one-task merge on an interrupted session safe to leave on `main`.
- **The lane table's file scopes.** The one file two lanes both touched
  (`cli/args.rs`) merged without conflict, because the brief told the lane to
  keep its diff surgical and to quote it back.

**What I worked around, and what was ambiguous.**

- **`CLAUDE.md`: "Sessions end at task boundaries; stopping early to hand over a
  clean state beats pushing through."** This session was explicitly told to run
  continuously through Phase 2 instead. The instruction was right for a
  batch-review workflow, and the line it overrides is the one that would have
  produced a cleaner stop. The two are in tension and the tension is unrecorded
  anywhere but here. **The token limit arrived exactly where that line predicted
  it would.**
- **"SPEC §8" in the session brief.** `spec/SPEC.md` §8 is mutation semantics;
  the output contract that needed the amendment is `research/wire-routing.md`
  §8. Resolved by reading both rather than by asking. This is precisely the
  drift the milestone's own naming rule warns about — "a task is named by its
  role, with the number in parentheses" — and the same hazard applies to section
  numbers across documents, which no rule currently covers.
- **`CLAUDE.md`: "Corpus-gated and environment-gated checks do not run in lane
  worktrees."** Read literally this forbids `wire draw` (T14) from making the
  sheet-pin measurement it owes, which requires `KICLI_TEST_KICAD_CLI=1`. I read
  it as a statement about the gate discipline rather than a ban on measuring,
  and told that lane to run its own probe explicitly. **This is a real ambiguity
  in a binding document and wants a ruling**, because the measurement is the
  point of that task and a lane that cannot measure cannot do it.
- **Three lanes at once against a rule written for lanes.** `CLAUDE.md` says
  "one task lane per subagent" and the Phase 2 table names two lanes; I ran
  three subagents by splitting Lane B along file scope (T14 and T17 share no
  file but `cli/args.rs`). Defensible under "two subagents never own the same
  module", but it is an extension of the rule rather than an application of it.

**What no document covered, and I had to decide.**

- **Whether to preserve two lanes' uncommitted work as WIP commits.** Both
  interrupted worktrees hold substantial unverified drafts (616 lines of tests;
  581 lines of implementation plus 130 lines of shared probe-crate changes). A
  `--no-verify` commit would preserve them against directory cleanup but would
  break `ENGINEERING.md`'s "the gates pass at every commit". **Decided: record
  rather than commit.** The branches and worktree paths are named in each task
  entry, and both entries state that the drafts were never compiled, never run
  and never falsified, so they carry no standing as evidence. The risk accepted
  is that a `git worktree prune` loses them; the work is a draft that the record
  makes reproducible.
- **`cargo` is not on `PATH` in the session shell.** Every command needs
  `export PATH="$HOME/.cargo/bin:$PATH"`. Undocumented anywhere; every brief had
  to carry it. Worth a line in the environment notes.

  **This bit the new pre-commit hook, measured at this session's last commit,
  and James ruled it fixed on the spot.** The first attempt failed with
  `.githooks/pre-commit: line 10: exec: cargo: not found` and no commit was
  made. A git hook does not read the interactive shell's profile, so it cannot
  assume `cargo` is found the way a developer's terminal finds it.

  **The defect was usability, not safety.** `exec` failing under `set -e` still
  refused the commit, so no unchecked commit could pass — but the operator sees
  a shell error naming a line number rather than a gate result, and the obvious
  workaround is `KICLI_SKIP_HOOK=1`, which turns a missing tool into a skipped
  gate suite.

  Fixed: the hook resolves `cargo` on `PATH`, then under
  `${CARGO_HOME:-$HOME/.cargo}/bin`, and refuses with a sentence that says the
  gates could not run when it finds neither. **Both branches measured**, each
  run with `env -i` and a bare `PATH=/usr/bin:/bin`: with cargo absent from PATH
  the hook found it and all six gates passed; with `CARGO_HOME=/tmp/nope` it
  printed the refusal and exited 1. The failing branch is the falsification —
  the hook was shown capable of refusing, not only of passing.
- **Whether a merged-but-unreviewed task may sit on `main`.** Decided yes, with
  the entry saying so in its own heading, because the alternative is either an
  unreviewed tick or discarding green work. The tick and the merge are separate
  claims and the record now separates them.

**Subagent WORKFLOW NOTEs: none exist, and that is the finding.** The
`lane-implementer` definition that asks for one arrived with the agentic-layer
change *after* the three subagents were dispatched, so no brief requested one
and none was returned. The two failed lanes returned nothing at all beyond a
truncated last line. Nothing has been reconstructed or paraphrased into this
section: the raw answers are the data, and for this session the data is absent.

**One measurement about the workflow itself.** Of roughly 210k subagent tokens
spent on the task that finished, the session's own limit arrived with two of
three lanes mid-task. Three concurrent implementer lanes plus an orchestrator
holding the full governing-document set is the shape that ran out. The next
session may want fewer concurrent lanes, or an orchestrator that reads less and
points more.

---

# Session 2 — resumed, checkpoint 1

Opened 2026-08-15 under the advisor's rulings on the interrupted stop, with the
amendment text that Session 1 recorded as ruled-but-missing. The
governing-document conflict Session 1 parked is therefore closed rather than
carried: `CLAUDE.md` no longer contradicts a ratified reading.

## Rulings applied, before any dispatch

Provenance throughout: advisor rulings, interrupted-stop review.

| Ruling | What it changed | Where |
|---|---|---|
| R9 — environment-gated checks in lanes | `CLAUDE.md`'s line now reads that a lane-worktree run never counts toward done — only the orchestrator's merged run does — and that a lane MAY run one to make a measurement its task owes | `25be0cf` |
| R10 — session grain | `CLAUDE.md`'s "sessions end at task boundaries" replaced: work is dispatched, merged and recorded at task grain; the session runs under `/goal` to a checkpoint stop; an interrupted session is wound down per the orchestrator definition | `25be0cf` |
| R2 — drafts are reference, not resumption | added to `CLAUDE.md`'s Parallel work section as a standing rule, beside the `lane-implementer` definition that already carried it | `25be0cf` |
| R3 — the hook refusal | **already applied** in `1030aab`; verified, not re-done. The refusal names PATH and rustup and never `KICLI_SKIP_HOOK` | — |
| R4, R5, R6 — the three coordination calls | **already promoted** into `tasks/M4.md`'s Phase 2 status in `47174b8`; verified | — |
| R7 — the delta view's six | **already promoted** in the T17 entry in `47174b8`; verified | — |
| R8 — the `scope.rs` twin | **already written** as chore C2 with its chore-runner eligibility condition; verified | — |

## Ruling received mid-session: mutation testing joins the exit criteria

**James's ruling, on the advisor's recommendation.** Recorded in `tasks/M4.md`'s
exit-criteria section. **Nothing was run** — the ruling scopes it to the M4
close, after every task is ticked and the gates are green.

The shape of it: `cargo-mutants`, once, scoped to `crates/kicli/src/route/`, as
an hours-scale batch verification run that is explicitly **never** a per-commit
gate. Every survived mutant is triaged into a genuine coverage gap — filed as a
chore or task with the mutant quoted — or a benign survivor recorded with the
reason it stands. Four counts reported: generated, killed, survived-genuine,
survived-benign. The M4 run also writes
`.claude/skills/mutation-run/SKILL.md` with its own results as the worked
example.

**Why it is not a duplicate of the falsification rule, which is the question it
invites.** Hand falsification proves a check can fail *at birth*: the implementer
breaks the code and watches it go red. Nothing re-proves it afterwards. A
surviving mutant is a check whose coverage *decayed* — the code moved underneath
it and no one was watching. This session has already produced the case in
miniature: the delta view's "nothing changed" check was vacuous on the day it was
written, and only its deliberate control caught it. A mutant run is that control,
mechanised, for every check at once, later.

**The prohibition is the load-bearing half.** "Do not silently iterate tests to
zero survivors during the close" — a survivor count driven to zero by unrecorded
test edits destroys exactly the evidence the run exists to produce.

**R9 was the load-bearing one.** Read the old way it forbade `wire draw` (T14)
from making the sheet-pin measurement that is the point of that task. The
amendment is what makes this checkpoint's most important measurement dispatchable
at all.

## What holds on `main` at checkpoint 1

Baseline measured at `25be0cf`, before any lane of this session merged, so that
every later measurement has a green control behind it:

| | |
|---|---|
| Six gates | pass — run by the pre-commit hook at `25be0cf` and again at the reviewer amendment commit |
| `KICLI_TEST_KICAD_CLI=1 cargo test --features corpus` | pass, **zero ignored in every binary** |
| `corpus::netlist_partition_matches_kicad_corpus` | green, **hierarchies matched: 35/35** — the largest is `vme-wren` at 2353 nets |

*(per-lane rows appended as they land)*

## Per-tick record

### The delta view (T17), Lane B — merged, tick review REJECTED on the first pass

Landed in Session 1: `sch view --view delta` wired to machinery that already
existed. Evidence in the T17 entry of `tasks/M4.md`, sections "Implemented and
merged, 2026-08-15" and "Tick review, 2026-08-15"; commit `71bb1d6`, merged as
`f144493`.

**Reviewer verdict, first pass: REJECT.** The gap named: the diff modifies
`crates/kicli/tests/command_surface.rs`, 62 lines and two new test functions,
which is outside the file list Lane B was given and which the entry's six
recorded decisions never declare as a scope excursion. Two excursions were
declared and ruled; this would be a third, undeclared.

**Resolution: the finding is real, and it lands on the orchestrator.** This
entry's own second check bullet reads `cargo test --test command_surface`, so the
two flagged functions ARE that check and writing them is executing the task. What
is genuinely wrong is the Phase 2 lane table: it listed Lane B's files and omitted
a file that two of Lane B's own task entries require. The implementer was handed a
check it could not satisfy inside its granted scope, and no brief said what to do
about that. The reviewer's own brief compounded it by presenting the lane file
list as the answer to "does the diff exceed the entry's stated scope" — the
entry's scope is what the entry's checks require.

Corrected: `tests/command_surface.rs` joins Lane B's row in `tasks/M4.md` and is
held by one task at a time, on the same rule as `AGENT.md`, because the verb
surface (T16) names the same check. **Counted as rejection one of two**, not
waived — the defect was in the inputs, but the reviewer read its inputs
correctly and the record was wrong.

**Reviewer verdict, re-review: APPROVE. TICKED.** A fresh reviewer checked every
factual claim in the entry against the source and all held. It reported
unprompted that its tool grant was read-only, so its APPROVE was reached by
tracing code paths rather than by running anything — see the workflow
retrospective for why the shell it was promised never arrived.

**The three weighted falsification rows were then measured**, in a scratch copy
outside the repository, against a baseline taken before and again after every
revert (`command_surface` 17 passed, `delta_view` 11, `agent_doc` 5, zero failed
throughout). Each break alone:

| Break | Check | Observed |
|---|---|---|
| `Delta::between(&saved, &saved)` | `a_delta_right_after_a_mutation_reports_nothing_changed` | **passes** — vacuous on its own, as claimed |
| the same break | `a_delta_reports_an_edit_made_outside_kicli` | **fails**: `left: []` against `right: ["~ S R1.Value \"10k\" -> \"22k\""]` |
| `compared_form` always `values` | `a_delta_against_a_state_of_hashes_alone_says_so` | **fails** on the header |
| `ExitCode::File` for an absent state | `a_delta_with_no_saved_state_exits_the_code_its_row_names` | **fails**: `left: 4`, `right: 1` |

**The entry's centrepiece claim is now measured rather than asserted**: its own
first check is vacuous, and only the control makes the pair evidence. Nothing was
written, staged or committed inside the checkout or any worktree to obtain this.

### `wire draw` (T14), Lane B — merged, REJECTED, fixed, re-reviewed, TICKED

Merged as `686aa72` from `worktree-agent-a702dd29e3d9c98dc` (lane commits
`c7c40ca`, `f915b0b`, `1759a7d`, `2e7050e`). Merged check: six gates, corpus, and
the environment-gated run with **zero ignored and the netlist oracle at 35/35**.
Evidence in the T14 entry, sections "The sheet-pin measurement, made 2026-08-15"
and "Implemented, 2026-08-15". Nine falsifications recorded.

**The measurement this task has owed since T6 is made, with a control that
behaved.** A sheet pin's angle names the edge KiCad puts the port on. Measured
against kicad-cli 10.0.5, not read from `parseSchSheetPin`: one drawing, four
ports, a stub wire leaving each outwards to a resistor pin, measured twice with
**only the angle changed**. Angles agreeing with the edges the ports are written
on carry all four stubs through to the child. Angles naming the opposite edge —
same positions, same wires — leave all four reading `unconnected-(Rx-Pad1)`. The
control, two symbol pins joined by a plain wire in the same drawing, is one net in
**both** arms, so the instrument was working when it reported the break. The
provenance above `of_sheet_pin` now says measured; I verified that diff is
comment-only, no logic, as the lane claimed.

**Reviewer verdict: REJECT — the claim outruns the measurement.** The review
audited the measurement hardest, as instructed, and most of it stood. The control
is real: moved 1.27 mm off its pins it fires **before any port is judged**, and
it lives in the same file as the ports so any harness failure takes it down with
them. Only the angle changed between arms — verified by diffing the written
files, which differ on the four `(pin …)` records alone with byte-identical child
sheets. One confound was found and ruled out: the probe derives `(justify …)`
from the angle, so justify differed too; swapping justify alone, angles
untouched, still connects all four. `kicad-cli` 10.0.5 genuinely ran. Five
falsification rows reproduced verbatim.

**The gap is narrow and real.** Two arms show the angle is load-bearing and that
the stub at the written position no longer meets the port. They do NOT
distinguish *relocation to the named edge* from *the port stays put and only
accepts an approach along the angle* — both predict the reflected arm's netlist
exactly, and nothing in the test looks at where the port ended up. But the
provenance comment and PROPOSED 2 both assert relocation. PROPOSED 2 stakes a
future ruling on it, and an entry must support a retroactive ruling from the
record alone.

**The reviewer measured the missing arm before rejecting, and the claim is
true** — relocating each stub to the position the angle predicts rejoins all
four. So the fix is additive: a third arm, re-measured by the implementer rather
than adopted from the review, because a reviewer's numbers are a report and not
the implementer's evidence. A fresh implementer is closing it, along with the
second finding: PROPOSED 2 has no executable half and no owner, deferring to
"whichever task owns the sheet-pin lint" without naming one.

**Rejection one of two, counted.** Two tick reviews this session, two rejections,
both sustained on inspection. The practice is not producing rubber stamps.

**The CLI verb is not in this task, and that is a coordination defect of mine
surfacing.** A new `Command` variant fails `agent_doc_covers_every_command` until
`AGENT.md` documents it, and `AGENT.md` is held by T16. So the lane was granted
`cli/edit/wire.rs` and `cli/args.rs` by a lane table that simultaneously withheld
the file that makes them buildable. The lane parked it correctly rather than
reaching for `AGENT.md`. The whole wire CLI surface now sits in T16.

### The candidate shapes (T9), Lane A — merged and TICKED

Merged as `a0a4e03` from `worktree-agent-a554f83be40bd0b4a` (lane commit
`44f080d`). Merged check: six gates, corpus, and the environment-gated run with
**zero ignored and the oracle at 35/35**. Evidence in the T9 entry, section
"Done, 2026-08-15": a twelve-row falsification table, two task-text corrections,
three PROPOSED items. `report.rs` untouched, `kicli-probe` untouched — verified.

**The task text was corrected on the tie-break, and the correction is the
interesting part.** The brief and the entry both framed it as "two shapes cost
the same yields the earlier one in the enumeration". Only true of an exact tie on
§7's whole chain: equal-cost shapes with different paths are separated by the
lexicographic rung, not by enumeration order. The check had to measure both.

**Reviewer verdict: APPROVE — and it measured six of the twelve rows itself**,
one break at a time with reverts between, against a green baseline taken from a
`git archive` copy at `/tmp`. The other six it took on the entry's word and said
which. Two of its six settle the entry's load-bearing claims:

- Its own going-in objection to the tie-break correction was that the drawing's
  lexicographically smallest L is *also* the enumeration-first L, so the
  assertion would pass under either theory. Reversing the lexicographic rung
  alone moved the winner to `LVerticalFirst` with enumeration order untouched.
  The drawing does separate the rungs; the correction stands on a measurement.
- On the one-home constraint: no second implementation exists in `shapes.rs`,
  the one home is `cost.rs:185`, removing it there fails all five drawing
  checks, and dropping the `polyline` guards leaves the drawings green while
  turning the unit check red. So "unreachable" and "untested" are held apart,
  and the second is closed rather than assumed away.

**The target-cell exception did not grow a second home, and the lane measured
rather than assumed it.** With both `polyline` guards removed, all five drawing
checks still pass, because a reversal is always co-blocked by the terminal's own
body. The guards were kept with the measurement recorded beside them rather than
deleted. Whether "removing it changes nothing" means redundant or means untested
is exactly what I asked that task's reviewer to judge.

## Findings

**Lane A, the candidate shapes (T9): a probe-crate footgun, and a blast-radius
claim that does not survive checking.** `Probe::label_of_kind` interpolates its
`shape` argument verbatim (`kicli-probe/src/drawing.rs:318`), so the parameter is
not a shape but a complete s-expression fragment. A caller passing `"input"` —
what the name invites — writes `(hierarchical_label "IN" input …)`, which KiCad
does not read as a shape, while kicli's lenient reader does. Real defect, and T9
lost a debugging cycle inside an environment-gated oracle to it.

**But T9 reported that five callers in `tests/net_probe_rules.rs` "pass over a
file KiCad reads differently", and that is wrong — found independently by me and
by that task's tick reviewer, which cited more of them.** Checked before recording:
those callers pass `""` for plain labels or the full `"(shape input)"`
(`net_probe_rules.rs:123-127`). Both correct. **No existing test is
compromised.** Recorded as chore **C3** with the correction stated, because a
finding that overstates its blast radius sends the next reader hunting five
broken tests that are not broken. The smaller true finding — kicli read a file
KiCad would not — is the more interesting one, and C3 proposes opening it
separately rather than settling it in passing.

**And the reviewer caught a trap in my own chore text.** C3 as I first wrote it
asked for a check that the helper produces the `(shape …)` form. If the helper
starts wrapping its argument, every existing caller — all of which already pass
the parenthesised form — double-wraps to `(shape (shape input))`. The fix is not
"make the helper wrap" but "make the argument a type that cannot be either
form", with the six callers moving in the same commit. C3 now records the trap,
the six call sites, and an oracle check run against KiCad rather than against
kicli's lenient reader, because kicli accepting a file is not evidence KiCad
does. **A chore I wrote, corrected by a reviewer reviewing something else.**

**Lane B, `wire delete` (T15): no probe drawing and no committed fixture can be
addressed by a handle.** `Probe::uuid` writes
`00000000-0000-4000-800{series}-{n:012}` (`kicli-probe/src/drawing.rs:94-100`),
varying only the last twelve digits, so every object of every probe drawing
shares the eight-character handle `00000000`.

**Measured before recording, and wider than reported**: across
`crates/kicli/tests/fixtures/sch/`, all **151** `uuid` atoms share the same eight
leading characters. Neither instrument can exercise a handle-addressed command.
No existing test is wrong — nothing addresses by handle today — but a whole class
of test is unavailable, and `wire delete` could only test its handle path by
rewriting an identifier in the drawing text after the harness wrote it. Recorded
as chore **C5**, chore-runner eligible for the probe crate, with the fixtures
named as a known second half rather than swept in silently.

**Infrastructure, and a near-miss: lane worktrees are branched from a stale
`main`.** Reported independently by two lanes. The T14-fix lane and the
`wire delete` lane were both created at `47174b8` — two to four commits behind
`main` — although both were dispatched after the merges they depended on. Both
briefs said "X merged immediately before you, so the file exists"; in both
worktrees it did not. Both implementers noticed and fast-forwarded before
starting.

**The near-miss is the finding.** In `wire delete`'s own words, "a lane that
trusted the brief would have written the module from scratch" — which is a
duplicate implementation of a module that already exists on `main`, arriving as a
merge conflict at best and a silent second home at worst. The two other running
lanes were checked at the moment this surfaced and both had their dependencies.

PROPOSED: the orchestrator states the base commit in every brief and verifies the
worktree is at it before the lane starts work, rather than saying "merged
immediately before you". Recommendation: accept — it is one command at dispatch,
and the failure it prevents is a lane silently rebuilding merged work.

## Session 2 workflow retrospective — running

Appended as evidence arrives, not composed at the end.

### Subagent WORKFLOW NOTEs, quoted verbatim

**Tick review of the delta view (T17), first pass — REJECT:**

> The inputs gave the lane's declared file scope but not the reasoning for why
> `tests/command_surface.rs` was excluded from it despite the task's own check
> bullets naming it — the reviewer had to infer the gap from a
> diffstat/decisions-list mismatch rather than being handed a scope document that
> reconciled the two; future tick-review dispatches should include the lane
> assignment rationale alongside the file list so an omission like this is
> checkable without cross-referencing the pre-implementation task text.

**Tick review of the delta view (T17), re-review — APPROVE:**

> My actual tool grant was Read/Grep/Glob only — no Bash/shell was available
> despite the brief's explicit "You have a shell this time. Use it" and its
> `cp -r`/`cargo test` instructions; I substituted static code-path tracing
> (cross-checked against already-passing structurally-identical unit tests) for
> live execution and measured zero rows empirically. If live execution is
> required for this review to count, the tool grant needs to actually include a
> shell, not just the brief claiming one.


**The candidate shapes (T9), Lane A:**

> The brief's framing of the tie-break check — "two shapes cost the same yields
> the earlier one in the enumeration" — is only true of an exact tie on §7's
> whole chain; equal-cost shapes with different paths are separated by the
> lexicographic rung, not by enumeration order, so the check had to measure both
> and the task text needed correcting. Also, the brief said `kicli-probe` was
> off-limits but not that its hierarchical-label helper emits a token KiCad
> ignores; that cost a debugging cycle inside an environment-gated oracle, and it
> is a live correctness risk for the five kicli-only tests already relying on it.

*(The last clause is wrong — see the finding above. The five callers pass the
correct form. Quoted verbatim as the rule requires; corrected beside it, not
inside it.)*

**`wire draw` (T14), Lane B:**

> The brief designated `cli/edit/wire.rs` and `cli/args.rs` to this lane, but a
> CLI verb cannot be added without `AGENT.md`, which the same brief withholds as
> T16's — `agent_doc_covers_every_command` fails on any undocumented verb, so
> that pairing is unbuildable and the CLI surface has to sit entirely in T16.
> Second: `crates/kicli/src/edit/wire.rs` is untracked until the first commit, so
> `git checkout --` cannot restore it — falsification runs on a brand-new file
> need a commit of the good state first, which the falsification-control skill
> does not mention.

**Tick review of the candidate shapes (T9) — APPROVE:**

> The brief was unusually good — naming the two claims to press hardest (the
> tie-break correction and the one-home constraint) sent me straight to the two
> breaks that actually decide them, and my going-in objection to 3a was
> answerable by a single measurement rather than by argument. Two frictions:
> `cargo xtask check`'s `clean` gate cannot run in a `git archive` copy, so a
> reviewer using the prescribed /tmp method gets five gates and a skip, not six —
> worth saying in the brief so the skip is not read as a finding; and the entry's
> falsification table lists the two `polyline` guards as separate rows while the
> PROPOSED text measures them jointly, so verifying the rows as written cost an
> extra break that a single sentence ("each was also dropped alone") would have
> saved.

**Tick review of `wire draw` (T14) — REJECT:**

> The brief was the right shape — naming the two granted excursions with
> instructions to verify rather than trust the comment-only one caught nothing
> wrong but cost minutes, not hours, and naming which falsification rows carried
> the most weight let me spend the whole budget on those. Two things were in the
> way: `cargo` needed `export PATH="$HOME/.cargo/bin:$PATH"` as warned, but the
> repo's `target/` is 11 GB and `.claude/worktrees/` another 11 GB, so a naive
> `cp -r` to `/tmp` is a trap — `rsync -a --exclude target --exclude
> .claude/worktrees --exclude .git` gives a 3 MB tree that builds in seconds, and
> that recipe belongs in the tick-reviewer definition. Second, the diff commit
> was still HEAD-adjacent when I started but two more commits landed underneath
> me mid-review; `git log <merge>..HEAD -- <lane files>` at the start is what let
> me be sure I was measuring the reviewed state, and it should be a named step
> rather than something a reviewer thinks of.

**Both of T14's reviewer frictions are now in the `tick-reviewer` definition**,
and T9's `clean`-gate friction with them: the rsync/`git archive` recipe, the
note that a scratch copy yields five gates and a skip which is an artefact and
not a finding, and pinning the reviewed state with `git log <merge>..HEAD` before
starting. PROPOSED, cheap to reverse — a reviewer that loses an hour to a 22 GB
copy is an hour not spent breaking code.

### What earned its keep, so far

- **The tick-review practice, on its first use, rejected.** It found a real
  defect on the first tick it ever reviewed. The defect was not where it aimed —
  it named the implementer's diff and the fault was the orchestrator's lane
  table — but a review that only ever approves is decoration, and this one was
  not. It cost one re-review and bought a corrected lane table before two more
  Lane B tasks hit the same file.
- **`ENGINEERING.md`'s "controls before conclusions".** Applied to the session
  itself: the corpus and oracle baseline was measured on `main` BEFORE any lane
  merged, so every later measurement has a green control behind it. 35/35, zero
  ignored, at `25be0cf`.
- **R9, the amendment that arrived this session.** Read the old way, `CLAUDE.md`
  forbade the very measurement the `wire draw` task exists to make. The
  amendment is what made this checkpoint's most important probe dispatchable.

### What I worked around, and what was ambiguous

- **An agent definition's tool grant does not take effect mid-session.** I
  amended `tick-reviewer` to carry `Bash` on James's ruling, dispatched the
  re-review, and the reviewer reported it still had Read/Grep/Glob only. The
  amendment is committed and correct; it binds the next session. **The verdict
  it produced is therefore a traced APPROVE, not a measured one, and the
  reviewer said so unprompted rather than letting it pass as measurement** —
  which is the amendment's other half working even though its tool grant did
  not. Worked around by dispatching a separate shell-carrying agent to make the
  three weighted breaks and report observations, with the verdict staying the
  reviewer's.
- **`CLAUDE.md` says agent definitions are "version-controlled working practice,
  changed only via ruling".** James ruled the shell grant in session, directly,
  rather than through the advisor. Treated as a ruling — he holds intent — and
  recorded here with that provenance so the advisor can reverse it retroactively
  like any other. Nothing in the governing documents says whether a direct
  instruction from James is a ruling for this purpose; I read it as one.

---

# Session 3 — checkpoint 2, closing M4 Phase 2

**STATUS: IN PROGRESS.** Opened 2026-08-15 under the advisor's rulings on the
checkpoint-1 review. The /goal condition for this session: the determinism
property test (T11), the four-way avoidance (T12), the label proposal (T13) and
the wire CLI surface (T16) ticked with recorded reviewer verdicts;
handle-addressable probe drawings (C5) done; the dogfood dry run executed against
the wire verbs with its defect list in `tasks/dogfood.md`; six gates green on
`main`; the netlist oracle at 35/35 with zero tests skipped; this report current.

## Rulings applied, before any dispatch

Seven, from the checkpoint-1 review. Each was recorded where its scope is rather
than all in one document — which is `CLAUDE.md`'s own rule about where a rule
lives. All seven landed in one commit, `8babffc`, before any lane opened.

| # | Ruling | Where it landed |
|---|---|---|
| 1 | The tick reviewer's consolidated amendment — five parts | `.claude/agents/tick-reviewer.md`, new section "The scratch copy, and the evidence trail" |
| 2 | falsification-control's first amendment — restoring a file git has never seen | `.claude/skills/falsification-control/SKILL.md`, new section |
| 3 | A direct instruction from James in a session IS a ruling | `CLAUDE.md`, agentic-layer section |
| 4 | Every brief states its base commit; the orchestrator verifies the worktree | `CLAUDE.md`, Parallel work section |
| 5 | The wire CLI surface (T16) entry corrected before briefing | `tasks/M4.md`, new "Corrected before briefing" section in that entry |
| 6 | Owners named on the open chores; C5 before T16 | `tasks/M4.md`, owners table at the head of `## Chores`, plus C4's own entry |
| 7 | Mutation testing recorded as a milestone-exit criterion | `tasks/M4.md`, `## Milestone exit criteria` — a table row and a paragraph |

Three of these deserve more than a table row.

**Ruling 1 was five separate frictions consolidated into one amendment**, and
they came from four different reviews in the checkpoint-1 batch. The new section
states: a scratch directory the reviewer created itself, from `mktemp -d`, never
a guessable fixed path — two reviews at one guessable name is a review reading
another review's broken tree and reporting it as a finding; the tree checksummed
BEFORE any reading, because a review of a truncated copy reaches real conclusions
about a file that does not exist; a reviewer never writes into a path it did not
create; `rsync -a` with three exclusions and never a naive `cp -r`, with the
clean-gate skip named as **an artefact of the method, not a finding**; the
reviewed state pinned with `git log <merge>..HEAD -- <lane files>` before reading;
and a disturbed evidence trail re-established by re-measurement, never by the
entry's assurance, with the A* (T10) review as the worked example. The `Bash`
grant James ruled in session was already in the committed definition at
`bf1c0d7`; it now takes effect for the first time, because this is the next
session.

**Ruling 3 answers the exact question the checkpoint-1 retrospective raised and
could not answer from the record.** That session recorded: "Nothing in the
governing documents says whether a direct instruction from James is a ruling for
this purpose; I read it as one." The reading is now the rule, with the advisor's
half stated beside it — the advisor reviews rulings for conflict with recorded
principle and raises conflicts to James, and does not reverse him.

**Ruling 5 was the largest.** The wire CLI surface entry (T16) had been written
when Lane B's verbs were expected to carry their own CLI. They could not: a new
`Command` variant fails `agent_doc_covers_every_command` until `AGENT.md`
documents it, and `AGENT.md` is held by one task at a time — so `wire draw` (T14)
and `wire delete` (T15) both merged as **library verbs with no CLI at all**, each
recording that it left the verb behind. Three further obligations had landed on
the entry from elsewhere. The correction states all seven owned items in one
table with their provenance, so the brief is derived from the entry rather than
from four other entries. It also **narrows the entry**, which is recorded as
PROPOSED below.

## Findings

### The stale-worktree rule caught a stale worktree on its first dispatch

Ruling 4 was promoted from a *near-miss* last checkpoint. On the first two
dispatches under it, it became a **catch**.

Both lanes were launched in one message, after `3563339` was committed to `main`.
Lane A's worktree was created at `3563339` — correct. The chore lane's worktree
was created at **`3aa4803`**, the commit `main` held at session start, **two
commits stale**: it was missing both the rulings commit and the `proptest`
dev-dependency. The chore-runner ran `git log --oneline -1` as its first action,
compared it against the base commit its brief named, and stopped without touching
a file.

Measured, not inferred: `git worktree list` showed
`agent-a216a44225205e17c 3563339` for Lane A, and the chore lane's worktree was
already gone — the harness removes a worktree an agent left unchanged, which is
the same reason nothing was lost.

**Why it matters more than the near-miss did.** The chore lane's second chore
edits `crates/kicli/src/edit/wire.rs`, whose rustdoc correction (C6) was filed by
`wire delete`'s tick review. On a base two commits stale the edit would still
have applied cleanly and the gates would still have passed — the divergence was
in `Cargo.toml` and the governing documents, neither of which C5 or C6 reads.
**It would have been discovered at merge, as a conflict in a file neither chore
touched.** That is the failure mode the rule was written for, and it cost one
re-dispatch instead of one debugging session.

**What the re-dispatch changed, and it is worth keeping.** The second brief
authorises the fix in advance, under three conditions checked together: the
working tree is clean, the branch has no commits of its own, and the base commit
is reachable. Only then may the lane `git reset --hard` to its stated base, and
it must say that it did. The conditions are what make it safe — a reset is
discarding work unless all three hold — and stating them in the brief costs less
than a round trip.

## PROPOSED items, in entry order

1. **`wire connect` leaves the wire CLI surface (T16) and lands with the join
   (T18).** Recorded in full in T16's corrected entry, with the three
   alternatives and why two of them lose. In short: T18's own first sentence is
   "The router behind the verb", there is no `route::connect` entry point today,
   and building one in a Phase 2 lane moves Phase 3's join into Phase 2 — and the
   phase split is James's workflow design, escalated to him once already.
   Shipping `connect` as a placeholder that parses and refuses was the other
   option, and it loses because the verb would sit in `AGENT.md` as available for
   two tasks' worth of time, and an agent reading `AGENT.md` would call it.
   **Recommendation: accept.** Cheap to reverse — if rejected, `connect` is one
   more variant over the renderer this task builds either way.
   `agent_doc_covers_every_command` asserts over the commands that exist, so
   nothing breaks in the gap.
2. **C4's owner is M5, with the rule catalogue.** A sheet pin whose angle
   disagrees with its position is genuinely unowned, and both its open questions
   are `spec/SPEC.md` §11.4's — whether it earns a `KI-…` code, and whether kicli
   reports the disagreement or corrects it. §11.4 is what M5 builds; M4 scores no
   drawing at all. Naming an M4 task would mean opening a scoring surface this
   milestone deliberately has none of, and would decide the larger question by
   default in the milestone least equipped to. **Recommendation: accept**, and
   carry the entry into M5's file at the M4 close rather than leaving it in a
   closed milestone.

## The control, measured before any lane merged

`ENGINEERING.md`'s "controls before conclusions", applied to the session itself.
Measured on `main` at `dcd99b7`, while the first two lanes were still working, so
that every later measurement this checkpoint has a green control behind it.

| | |
|---|---|
| Six gates | pass: fmt, clippy, test, doc, deny, clean |
| `cargo xtask corpus` | fetched and canonicalised, no change |
| `KICLI_TEST_KICAD_CLI=1 cargo test --features corpus` | pass across 36 test binaries, **`0 ignored` in every one** |
| `corpus::netlist_partition_matches_kicad_corpus` | **`hierarchies matched: 35/35`**, read from the test's own output rather than from its exit code |
| `corpus::every_corpus_hierarchy_loads` | `35 hierarchies loaded` |

The 35/35 line is quoted from `--nocapture` output rather than inferred from a
green test, because a corpus test that silently loaded nothing would also be
green — the same reason every absence check in this repository carries a presence
control.

### Found while deriving the four-way brief: the adjustment has nowhere structured to go

The four-way avoidance task (T12) says the router "offsets by 1 G and **reports
that it did**". `spec/SPEC.md` §9's ruling Q2 says the same: "refuse and offset by
1 G, reporting the adjustment". Deriving the brief from the entry meant asking
where in the output that report lands, and the answer is: nowhere structured.

Measured rather than assumed:

- `research/wire-routing.md` §8's contract — the text form and the JSON keys —
  has **no field for an adjustment**. `status`, `from`, `to`, `path`, `segments`,
  `corners`, `length_mm`, `cost`, `crossings`, `added`, `alternatives_considered`,
  and that is the whole of it.
- `route::report::Report` has one free-text field, `reason: Option<String>`,
  documented as "One sentence for a person, naming the numbers a decision rests
  on. A proposal says both the length and the threshold; a refusal says what it
  refused. A route that simply worked needs none."
- **`crates/kicli/src/route/report.rs` is the only entry in
  `.claude/hooks/frozen-paths.txt`.** Adding a field to it is a frozen-surface
  change, which is BLOCKED by definition and not a lane's to make — and the hook
  enforces that against the orchestrator too, which is the rule working as
  intended rather than an obstacle.

**PROPOSED: the four-way task reports the adjustment through `reason`, and does
not touch the frozen file.** The fit is good but not perfect. `reason`'s own
rustdoc says a route that simply worked needs none — and an offset route did not
simply work, it was adjusted, so it is exactly a decision resting on numbers that
a person should see. The task's check ("the report names the adjustment") is
satisfied by prose. **Recommendation: accept**, and dispatch on it. It is cheap to
reverse in the direction that matters: a structured field added later can be
populated from the same call site.

**The half that is genuinely for the advisor, recorded and not decided here.**
Prose is not actionable by the agent this tool is built for. An agent that reads
`"reason": "terminus offset 1 G to avoid a four-way junction"` cannot branch on
it the way it can branch on `cost.turns`; it would have to parse English, which
is the thing the §8 contract exists to avoid. The alternative is a structured
field — `adjusted: { from, to, why }` or similar — in §8 and in `Report`, which
**is** a frozen-surface change and therefore a ruling, not a lane decision.
Raised now rather than at the merge, because the call site is the same either way
and the cost of deciding late is one small follow-up rather than a rewrite.

## Per-tick record

### C5 and C6, chore lane — implemented, sent back at merge review, NOT merged on the first pass

Branch `worktree-agent-a45c9420429a8414d`, commits `278c678` (C5), `23a2504`
(C6), `1728d52` (the entries). Six gates green in the lane. **Not merged.** Two
defects found reviewing the diff against the entries, one substantive.

**The lane reset its own stale base, correctly and under the conditions the brief
set.** It reported the stale base as `3aa4803`, checked that the tree was clean,
that the branch held no commits of its own and that `3563339` was reachable, then
reset and said so. The three conditions are what make a `reset --hard` safe
rather than destructive, and the lane checked all three before acting.

**Defect 1 — C5's fix does not cover the case a hierarchy actually needs, and its
control could not have seen that.** `Probe::uuid` became
`{:08x}-0000-4000-800{series}-{:012x}` over `self.next_uuid`. **The leading eight
digits no longer carry the series.** `Probe::named_child_of` creates a sibling
probe with its own counter starting from zero, so:

- `crates/kicli/tests/net_probe_rules.rs:180-181` builds `left` at series 2 and
  `right` at series 3;
- `left`'s first object takes handle `00000001`, and so does `right`'s;
- they collide — which is the exact defect C5 was filed to remove, surviving in
  the multi-sheet case. The pattern recurs at `:221`, `:249`, `:293` and `:323`.

The control was not wrong; it was **too narrow to have caught this**. It counts
distinct handles within one drawing, and the collision exists only across sibling
drawings. That is the more interesting half: a control that measures the easy
case and passes is how a fix comes to look complete. Sent back with the fix, a
widened control over a parent and two siblings, and a falsification that must
show the **contrast** — the cross-sibling assertion failing while the
within-one-drawing assertion still passes, which is the evidence that the wider
control watches something the narrow one could not.

**Defect 2 — C6's replacement sentence reintroduces the ambiguity it was filed to
remove.** It landed as "This is a boundary on how many wire ends meet at a point,
**shared with** `crate::edit::mark`'s refusal boundary, measured from opposite
directions." Read plainly, the boundary is the shared thing — which is the false
claim the chore exists to delete. It also dropped the substantive true part: what
is genuinely shared is the **implementation**, `edit::mark::wire_ends_at`, made
`pub(crate)` by `wire delete` (T15) precisely so both boundaries rest on one
answer. Sent back to say what the entry says is true: two boundaries on one
measurement, from opposite directions, sharing an implementation rather than a
value.

**And the entry itself is being corrected**, because as written the C5 "Done"
section claims a fix broader than what was measured. A chore entry that
overstates its own reach is the thing the record exists to prevent.

### Merged: C5 and C6, second pass — `c27c195` and below, merge `HEAD~1`

Both corrections landed. C5 became `{:02x}{:06x}-0000-4000-800{series}-{:012x}`,
folding the series into the leading eight digits so two siblings can no longer
collide. The widened control spans a parent and two `named_child_of` siblings,
and **its falsification shows the contrast the finding needed**: with series-blind
leading digits the narrow assertion still passes — 12 objects, 12 distinct handles
in one drawing — while the wide one fails at 9 distinct handles against 18
objects. That contrast is the evidence, not the pass.

C6's sentence became: "This measures the same quantity as `crate::edit::mark`'s
refusal boundary — how many wire ends meet at a point — and both boundaries call
the same implementation. They are two different thresholds on one measurement,
approached from opposite directions." It says what is shared (the measurement,
the implementation) and what is not (the value), and names no number.

Merged check, on the merged result in the main checkout: **six gates green; 71
test binaries, `0 failed` and `0 ignored` in every one; oracle
`hierarchies matched: 35/35`.** The oracle is the one that mattered here — C5
changes the identifier of every object in every probe drawing, and 35/35 is the
measurement that says KiCad still reads them the same way.

## Ruling received mid-session: the output contract gains a structured adjustment

Provenance: advisor ruling, 2026-08-15, answering the frozen-surface question
raised above. **Ruled in favour of the structured field**, against the PROPOSED
recommendation to use `reason`. Landed as one orchestrator commit, `dd4f659`.

`Report::adjusted` is a list, empty when nothing moved, of
`{ terminal, by, why }`. Three things about the shape are worth recording,
because each was a choice inside the contract the ruling set:

- **`terminal` names itself as `from` and `to` do**, so a caller learns which end
  moved without comparing coordinates against a request it would have to remember.
- **`by` is a displacement, not a position.** Where the terminal ended up is
  already the corresponding end of `path`, and this module's stated principle is
  that nothing derivable is stored — "a stored derivative is a second answer
  waiting to disagree with the first". The requested point is the terminus less
  `by`. `Point` was already used as a displacement by `Rect::offset`, so this
  invents no type.
- **`why` is a closed enum**, one variant so far. The ruling said never free text,
  and the reason it is right is mechanical rather than stylistic: a new reason
  becomes a compile error at every match on it, which is what makes it safe for
  an agent to switch on.

§8 was amended in the same commit — text form and JSON — per the standing rule
that the spec and the frozen contract must not disagree. The text line is omitted
entirely when nothing was adjusted.

**The freeze procedure worked, including the part that failed.** The path was
lifted from `.claude/hooks/frozen-paths.txt`, the change made, the path restored,
all in the one commit. Midway I restored the path **before** the change compiled,
and the hook refused the next edit — correctly. That is the mechanism working,
not an obstacle, and it is worth recording that the orchestrator was the one it
refused.

Falsification of the three new assertions, each break made alone against the
committed good state, watched, and reverted:

| What was broken | Which assertion caught it |
|---|---|
| `Report::of` hands back a non-empty `adjusted` | `a_route_that_moved_nothing_reports_no_adjustment` — "an empty collection, not an absent one" |
| `Adjustment::FourWayJunction`'s token changed to `"adjusted"` | `an_adjusted_terminal_says_which_by_how_much_and_why` — left `"adjusted"`, right `"four-way"` |
| `by` read as a position rather than a displacement — the subtraction dropped | the same test's derivation assertion — left `(1524000, 889000)`, right `(1524000, 876300)` |

Control re-run after the last revert: 5 passed, 0 failed.

### And the amendment I wrote this morning is one case too narrow

Found by walking into it. The falsification-control amendment ruled at
checkpoint 1 says: **a brand-new untracked file** cannot be restored by
`git checkout --`, so falsification on new files commits the good state first.

The first falsification break above was made against `report.rs` while the whole
contract change was **uncommitted**. `git checkout -- crates/kicli/src/route/report.rs`
duly restored the file to `HEAD` — and took the entire change with it, not just
the break. The file was tracked, so the amendment as written did not cover it;
the failure is identical.

**PROPOSED: widen the amendment from "a brand-new file" to "any state that is not
committed".** The rule is not about whether git knows the file, it is about
whether git knows the state you want back. An uncommitted edit to a tracked file
is exactly as unrecoverable as an untracked file, and reads as safe because the
file is tracked. Recommendation: accept, and keep the existing new-file wording
as the worked example under the wider rule. Cost of the incident: re-applying
four edits, and it would have been a whole task's work in a lane.

### Ruling received mid-session: the restore rule widens to any uncommitted state

Provenance: advisor ruling, 2026-08-15, promoting the PROPOSED above immediately.
The falsification-control skill's restore rule now reads: **git can only restore
committed state — before any deliberate break, the good state is committed,
whether the file is new or tracked-with-uncommitted-changes. A tracked file reads
as safe and is not: `checkout --` takes uncommitted work with it.**

Both incidents stay as worked examples, quoted rather than summarised: the
`wire draw` (T14) new-file case in the implementer's own words, and the
contract-amendment (`dd4f659`) tracked-file case in the orchestrator's. The
section is retitled "Commit the good state before you break anything", because
the old title named the symptom — a file git does not know about — and the rule
is about the state, not the file.

### The determinism property test (T11), Lane A — merged `986daab`, awaiting tick review

Lane commits `e2d662e`, `2d1724f`, `05d4a58`. Merged check: **six gates green.**
Diff confined to `route/obstacles.rs`, two new test files, and the entry.

**The shuffled arm found a defect rather than confirming an absence, which is
the best thing a determinism check can do.** `Obstacles::entering` named the
**first** `Block` feature laid down, and that order is the file's item order.
With two overlapping symbol bodies, the same drawing saved two ways reports
**different blockers** — same routes, same tallies, same costs, different name.
KiCad reorders items when it saves, so the report would blame one symbol today
and its neighbour tomorrow for an identical drawing. Fixed by `keep_smallest`,
which decides between equally true names by the names themselves. Recorded as
**PROPOSED** by the implementer, recommendation keep, because it changes Phase 1
behaviour no ruling covers. The rustdoc at both the method and the helper states
why, so the reason cannot go stale away from the rule.

**Two falsification rows are the ones worth reading, because the break produced a
GREEN check and the fault was the control rather than the code:**

| Broken | What happened |
|---|---|
| the shuffle permutes nothing | **nothing failed, first time.** The control compared against the drawing's own text, so it passed on layout alone. Rewritten to compare against the same file through the same writer, unshuffled; the rerun fails |
| every answer replaced by a constant | passed the shuffled arm, which had **no class counters**. Baselines now must hold a shape route, an A\* route and a refusal; the rerun fails both arms |

**One row caught nothing and is recorded as catching nothing** — successors
expanded from a `HashSet`, on the argument that a state carries its direction, so
successors of one pop are never the same state and the queue's total order fixes
every pop. Recorded rather than deleted, on the same principle T10 used for its
four unreachable rules.

**Proptest configuration, and the trade it takes:** 64 cases, ChaCha,
`RngSeed::Fixed`, shrinking on, persistence off. The seed is fixed deliberately —
coverage is bought by the case count and by each case being a whole sweep of
every pair, not by a varying seed, and a varying seed makes a gate that fails on
someone else's machine. Recorded because it is exactly the trade the brief asked
to be named.

**Sizing was measured, not guessed:** a hundred runs over every pair costs what
the pairs cost — three-symbol drawings 18.4 s, four-symbol 33 s, against a ~20 s
suite without this check, with the engines dominating (166 ms of a 193 ms pass).
Two two-symbol drawings hold all three answer classes at 11.5 s.

**Merged check on the main checkout, corpus included:** six gates green; **73
test binaries** — two more than before this merge, which is the two new test
files — with `0 failed` and `0 ignored` in every one; oracle
`hierarchies matched: 35/35`, `35 hierarchies loaded`.

Tick review dispatched. **Not ticked.**

## PROPOSED items raised by the lanes, for the advisor

3. **`Obstacles::entering` blames the smallest name rather than the first.** The
   implementer's, on a defect its own check found. It changes Phase 1 behaviour
   and no ruling covers it. **Recommendation: keep.** Every candidate name is
   equally true, so choosing between them by the names costs nothing, and the
   alternative is a report that changes its answer when KiCad re-saves an
   unchanged drawing. Cheap to reverse.
4. **A third amendment to falsification-control: a break that produces a green
   check may mean the CONTROL is wrong rather than the check.** The determinism
   implementer's, from its WORKFLOW NOTE, and two of its seven rows were exactly
   that case. **Recommendation: accept**, with those two rows as the worked
   example — they are the clearest instance in the record, because in both the
   code was innocent and the instrument was blind. The existing skill tells you
   to break the code and watch the check fail; it does not say what a green
   result means, and the intuitive reading — "the code is fine" — is the wrong
   one exactly when it matters.
5. **The base-commit dispatch rule needs a fast-forward branch.** Also the
   determinism implementer's. Its worktree came up at `3aa4803`, two commits
   behind the named base, and `3aa4803` was a **strict ancestor** of it — so it
   fast-forwarded and worked on exactly the named base rather than burning the
   dispatch on a stop-and-report. **This is a deviation from the brief as
   written, and it is recorded as one**: the brief said stop, and it did not
   stop. The judgement was good and the outcome was correct, which is precisely
   why the rule should be fixed rather than the deviation excused.
   **Recommendation: amend the rule to "fast-forward if the named base is a
   descendant of the worktree's commit and the tree is clean; stop otherwise",**
   so the safe case does not cost a round trip and the unsafe case still stops.
   Note this is the **second** stale worktree of the session out of three
   dispatches, which makes the underlying defect — recorded at C5 as "lane
   worktrees start stale" — a recurring cost rather than an incident.

### Tick review of the determinism property test (T11) — APPROVE. TICKED

**The amended reviewer definition had its first outing, and every clause of it
was used.** The reviewer pinned the state before reading (`986daab..HEAD` over
the lane files, empty), took its scratch tree from `mktemp -d`, copied with
`rsync -a` and the three exclusions, and — the new clause — **verified the copy
with `rsync -n -ci` before reading a line of it**. It reported five gates and a
`clean` skip and named that skip as its method's artefact rather than a finding,
which is exactly what the amendment was written to prevent. It confirmed by
`diff` after every restore that the real checkout was untouched.

**It reproduced the entry's most valuable claim instead of accepting it**, which
is the difference the amendment asks verdicts to state. Both "green check,
faulty control" rows were re-made against the current control *and* against a
reconstruction of the old one, so the evidence is a contrast rather than a pass:
`reordered()` ignoring its argument fails the current control and passes the old
one vacuously; `answer()` returning a constant fails both arms now, and with the
baseline's class counts removed the shuffled arm alone passes vacuously. The
entry's rows are true.

**The row that caught nothing was judged on its reasoning rather than waved
through.** There is no `HashSet` in `search.rs` to re-break, so the reviewer read
the source: `State` derives `Ord` over `(at, dir)`, `Queued` over `(f, g, state)`,
`BinaryHeap::pop` returns the maximal element by that total order regardless of
insertion order, and successors of one pop never share `(at, dir)`. It then said
in its verdict that this was read and not measured. That distinction is the whole
point of the clause requiring it.

**And it found something by measuring: a claim that went stale between branch
time and merge time.** T11's "Carried" paragraph says every probe item answers to
the handle `00000000`, and gives that as the reason its ambiguous-blame case had
to be built from two symbols rather than two overlapping wires. True when the
lane branched from `3563339` — **false in the merged tree**, because C5 landed on
`main` in the chore lane while T11 was working, and `278c678`/`c31de4b` are
ancestors of `986daab`. The reviewer read `01000001`, `01000006` and others
directly out of the merged tree rather than inferring it.

Nothing is wrong with the test; only its stated justification. The paragraph is
**left standing with a correction beneath it**, because how it went stale is the
part worth reading.

**The general lesson is the reviewer's, and it is a genuine gap in how this
session works.** A "Carried" note describes **another lane's state**. The lane
writing it reads that state at **branch time**; the next owner reads the note at
**merge time**; and between those moments the other lane can land. Two lanes ran
concurrently all session, so this is structural, not a one-off.

**WORKFLOW NOTE, tick review of the determinism property test (T11), verbatim:**

> The entry's "Carried" paragraph asserts probe uuids are all `00000000` — true
> when the lane branched from `3563339`, but false in the merged tree because the
> C5 uuid-distinctness fix (`278c678`/`c31de4b`) landed on `main` first and is an
> ancestor of the merge. Worth a standing reminder that "Carried" notes
> describing another lane's state need re-checking against merge-time `main`, not
> branch-time `main`, before they're trusted by the next lane owner.

**WORKFLOW NOTE, the determinism property test (T11) implementer, verbatim:**

> The brief's "stop and report if the base is not X" rule has no branch for the
> case that actually occurred — the worktree was a lossless fast-forward away
> from the named base — so following it literally would have burned the dispatch
> on a one-command fix; the rule should say "fast-forward if the named base is a
> descendant, stop otherwise". The falsification-control skill needs a third
> amendment: a break that produces a green check may mean the *control* is wrong
> rather than the check, and two of my seven rows were exactly that.

**WORKFLOW NOTE, the C5/C6 chore lane, verbatim:**

> The orchestrator's catch on defect 1 revealed a real gap in coverage — the
> narrow control tested only within a single drawing, missing the multi-sheet
> case entirely. The widened control using `named_child_of` siblings now forces
> the issue. For defect 2, the ambiguous wording "shared boundary" was a subtle
> regression that rewording alone corrected; the intra-doc link to mark still
> verifies.

*(The chore lane's first note, before the corrections, read "No issues with the
brief, inputs, or configuration. Both chores complete cleanly." Both notes are
kept: the first is quoted here as it was written, and the gap between the two is
itself data about what a WORKFLOW NOTE is worth before review.)*

6. **PROPOSED: a carried claim about another lane is re-checked at merge time.**
   From the tick reviewer's note above. **Recommendation: accept**, as a line in
   the `task-entry-recording` skill rather than in `CLAUDE.md` — it is about how
   an entry is written, which is that skill's scope, and `CLAUDE.md`'s own rule
   says a new incident adds a worked example to the relevant skill rather than a
   rule to the file. The T11 paragraph and its correction are the worked example.

### Rulings received mid-session: worktree currency, and green-after-a-break

Provenance: advisor, 2026-08-15, promoting PROPOSED 4 and 5 above.

**1. Worktree currency, mechanism and safety net together.** Both halves landed,
each where it acts:

- `CLAUDE.md`, Parallel work: **a lane worktree is created at, or reset to, the
  brief's named base commit as part of dispatch.** Worktrees no longer start
  stale by construction, which makes the existing verify-before-work rule a
  safety net rather than the mechanism. Two stale worktrees in three dispatches
  is the measurement behind it.
- `.claude/agents/lane-implementer.md`, a new opening section replacing the bare
  stop-if-stale: **fast-forward only if the named base is a descendant of the
  worktree's commit AND the tree is clean; stop and report otherwise.** The two
  conditions are what make it safe — a non-descendant base means moving discards
  commits, a dirty tree means moving discards work, and neither is the lane's to
  discard.

The provenance is recorded in the definition itself: the determinism task (T11)
hit exactly the fast-forwardable case under a rule that said only "stop", made
the right call, and thereby deviated from its brief. **Recorded as a deviation
despite the correct outcome — the rule was wrong, not the judgement.** That
distinction is why the fix is a rule change rather than an excuse, and why the
deviation stays in the record.

**2. falsification-control, third amendment: green after a deliberate break is a
finding about the instrument.** The trap the amendment names is that green
*feels* like good news, so the row gets skipped and the table records a break
that "did not apply". The rule now: a break that leaves the check green means the
instrument may be blind, and **the instrument is investigated before any
conclusion is drawn from that row.**

Two cases, and telling them apart is the investigation: the break was a **no-op**
— real evidence that something else enforces the rule — or **the check does not
watch what it claims**, which is the dangerous one, because it will keep passing
forever. The amendment states plainly that case 2 is never recorded as case 1:
*"Removing it changes nothing" is the same sentence for "this rule is redundant"
and "my test cannot see this rule", and those are opposite findings.*

Worked examples quoted from T11 — the shuffle that permuted nothing, and every
answer replaced by a constant, both green over innocent code — with the note that
the tick reviewer re-made each against the current control *and* a reconstruction
of the old one, so the record holds a contrast rather than a pass.

**And the precedent for the other case is recorded beside them**, so the two are
not conflated: the candidate shapes (T9) measured that with both `polyline`
guards removed all five drawing checks still pass, and did **not** read that as
the guards being safe. It found the structural reason — a terminal's own body box
covers every neighbour of its own cell except the escape point — and kept both
rules while moving their measurement to `route::shapes::tests`. Case 1, diagnosed
as case 1, with the work shown.

### Ruling received mid-session: a claim about another lane expires

Provenance: advisor, 2026-08-15, promoting PROPOSED 6 above. Landed in
`.claude/skills/task-entry-recording/SKILL.md` under Evidence, which is where the
orchestrator recommended it — a rule about how an entry is written belongs in the
skill that governs entries, per `CLAUDE.md`'s own rule that a new incident adds a
worked example to the relevant skill rather than a line to that file.

The rule: **a claim about another lane's state is written pinned to a commit —
"as of `<commit>`" — and treated as expiring.** It describes the tree at writing
time, not at reading time. Whoever relies on it at merge or review time
re-verifies against the merged tree rather than trusting the note.

T11's "Carried" paragraph is the worked example, and two details of the fix are
carried into the skill because they generalise: the correction was recorded
**beneath the claim rather than over it**, since how a claim went stale is what a
later reader needs; and **nothing about the test was wrong, only its stated
justification** — which is the usual shape of this defect. The code is fine and
the record is not, which is exactly the failure a record-driven review process
has to be able to catch, and did.

### The four-way avoidance (T12), Lane A — merged `8f86e87`, awaiting tick review

Lane commits `2c6eb45`, `d0fb86c`, `2e389a1`. **Six gates green on the merged
result.** Diff confined to `route/terminal.rs`, two new test files, and the entry.

**The design.** `Approach::of(source, target, schematic, grid)` settles **both**
terminals against the drawing **before the search sees them**, and answers with
the terminals plus the `Vec<Adjusted>` a caller copies into `Report::adjusted` —
the field the advisor's ruling added earlier today, populated at the call site
the ruling anticipated. A crowded terminal steps 1 G **along a wire that already
meets the point**, chosen in `Heading::EVERY` order rather than file order, so
the terminus stays on the net the route was drawn to reach. Choosing in file
order would have re-introduced exactly the defect the determinism task (T11)
found hours earlier, in a different module.

**The off-by-one was measured from both sides**, which is what this task was most
likely to get silently wrong. `CROWDED = 3` against `mark::FOUR_WAY = 4`: that
boundary counts the ends that **are** there, this one the ends there **would
be**. `CROWDED = 4` and `CROWDED = 2` each break a *different* assertion. An
off-by-one in the safe direction — never offsetting — is invisible to a
badly-built suite, and this one is not.

**`edit/mark.rs` was designated to this task and is unchanged**, because
`wire_ends_at` already answered. That is the designation working as intended: the
point was never that the file needed editing, it was that **no second
implementation should appear**. `the_four_way_rule_has_one_home` now holds three
thresholds to one measurement — the junction verb's refusal, the wire verb's
stranded-junction report, and the router's avoidance — with a presence control on
every absence arm, stated in the test's own module doc.

**Task text yielded to measured reality, and the citation is recorded.** T12's
check text says "`mark::add_junction` at the terminus is still refused
afterwards". That **cannot mean what it says**: after a correct 1 G offset the
terminus keeps three ends, and three ends *take* a junction rather than being
refused one. Implemented as the counterfactual — a drawing carrying the two
segments the router **declined to draw**, on which `add_junction(P)` refuses with
`FourWayJunction` — with the control that on the drawing the router really
produced, `add_junction(P)` **succeeds**. Flagged to the reviewer as the single
most important thing to judge, because a substitute weaker than the original
clause would be a REJECT.

**The oracle says the offset preserves the connection, and that the junction is
what makes it** (kicad-cli 10.0.5):

| Arm | KiCad's nets |
|---|---|
| route written + junction at the landing point | `Net-(R1-Pad1)` = `{R1.1, R2.1}` |
| same drawing, junction withheld | four `unconnected-` nets |

kicli's partition equals KiCad's on both arms. **A wire end on another wire's
interior is not a connection on its own** — which is why the second arm is worth
more than the first.

**18 falsification rows**, each watched failing and restored, **every restore
checksum-verified against `2c6eb45` with `shasum -c`**. The good state was
committed before the first deliberate break — today's amendment, applied by the
first lane to work under it.

One row caught nothing it was aimed at and says so: **A8**, an offset stepping
perpendicular into empty space, was caught by `edit::wire::draw` refusing the
path as `Blocked` rather than by the oracle, because an off-net landing beside
that meeting point is unreachable without running along an arm.

**Carried gap, recorded for Phase 3:** the join owes copying `approach.adjusted`
into `Report::adjusted` at the real call site, **and** writing the landing-point
junction into `Report::added.junctions`. The oracle above is what makes the
second load-bearing rather than tidy.

**Merged check on the main checkout, corpus included:** six gates green; **75
test binaries** — two more than the last merge, which is this task's two new test
files — with `0 failed` and `0 ignored` in every one; oracle
`hierarchies matched: 35/35`. This task's own environment-gated arm runs and
passes on the merged tree: `the_router_never_makes_a_four_way_junction` and
`the_offset_terminus_still_joins_the_net_kicad_reads`, 2 passed.

Tick review dispatched. **Not ticked.**

### Finding: the dispatch half of the worktree ruling cannot be executed as written

**Third consecutive stale worktree.** T12's came up at `d62aa69`, **five** commits
behind its named base, and fast-forwarded under the rule promoted an hour
earlier. The lane's half of that ruling worked exactly as designed: preconditions
checked, one command, work done on the right base, and the fix disclosed in the
final message.

**The orchestrator's half did not, because the mechanism does not exist.** The
ruling says a lane worktree is "created at, or reset to, the brief's named base
commit **as part of dispatch**". The dispatch mechanism available creates the
worktree itself, at a commit the orchestrator does not choose and cannot set,
**at the moment the agent starts** — there is no point between creation and the
lane's first action where the orchestrator can intervene. So the rule as written
assigns a duty the tooling does not permit.

This is recorded rather than worked around silently, and it is not a reason to
weaken the rule — the rule is right, and the lane's half of it has now saved
three dispatches. **PROPOSED: the orchestrator creates the worktree explicitly
before dispatch** — `git worktree add -b <branch> <path> <base>` — and briefs a
non-isolated lane to work in that path, so the base is chosen rather than
inherited. Recommendation: adopt for the next dispatch and see whether it costs
anything; the cost of the status quo is now measured at three of five dispatches
starting stale, each one a round trip or a near-miss. Recorded here because the
verification rule and the lane's fast-forward branch together have caught every
instance so far, so this is a cost question rather than a correctness one.

### The verb surface (T16), Lane B — implemented, FAILED THE MERGED CHECK, backed out

Branch `worktree-agent-a8e75981ddd6a58db`, commits `2861254`, `dec0c0a`. The lane
reported six gates green. **The merged result failed the `test` gate**, so the
merge was backed out and `main` returned to `ddc2e84`, green. The branch keeps
the work.

This is the merged check earning its place. It is the orchestrator's job at every
lane merge and is never skipped, and this is the first time this session it has
found something a lane's own green run did not.

**The defect: the `routed` goldens embed randomly generated UUIDs.**
`wire_output_contract.rs` fails at line 67 on `a_drawn_route_matches_the_golden`
and `a_moved_terminal_matches_the_golden`. `a_drawn_route` calls
`edit::wire::draw`, which writes real wire records and puts **their freshly
generated identifiers** into `report.added.wires`; the golden captures them
verbatim:

```
golden:   "wires": [ "ebb43fde-5c3c-4534-8776-e35e3f8aefaf", … ]
this run: "wires": [ "9d618919-5152-4a4e-8860-23f4945012d3", … ]
```

Everything else in both forms is byte-identical — path, segments, corners,
`length_mm`, the five cost parts, `adjusted`, `labels`, `blocked_by`, `reason`.
**Only the identifiers differ, and they differ every run.** Measured twice, two
different sets, so this is neither a merge interaction with the four-way task
(T12) nor anything about this checkout.

**The part sent back is not the fix — it is the explanation.** A lane cannot
report six gates green on a state carrying this golden, so either the gates ran
before the goldens were written or something refreshed them inside the run that
checked them. Today's third falsification amendment says a surprising green is a
finding about the **instrument**, investigated before any conclusion is drawn
from it, and that applies to a green `xtask check` exactly as it applies to a
green assertion. The lane is asked to work out what happened and record it,
because **if the gates were green on a state that differed from what was
committed, that is worth more to the record than the fix is.**

The fix directed: normalise identifiers in the comparison — a stable placeholder
per distinct identifier in first-appearance order — so **count, ordering and
shape stay asserted** while the RNG is not. The lane's own module doc already
carries the principle that decides it: "a golden refreshed after a key was
dropped passes as happily as one refreshed after a fix". A generated identifier
is that problem one level down. Explicitly ruled out: dropping `added` from the
golden (count and order are contract), and making the drawing produce fixed
identifiers (fresh identifiers are correct behaviour; a test needing them changed
is measuring the wrong thing). And the normalisation must itself be falsified —
two identifiers where three belong, and segments in the wrong order — because a
normaliser that collapses everything to one placeholder hides both, which is how
this fix goes wrong.

**The lane's own finding, filed as a chore below:** `agent_doc_covers_every_command`
asserts a **mention, not a heading**. Deleting the entire `kicli wire draw`
section left it green, because the `[routing]` prose names the verb.
`agent_doc_covers_every_verb_flag` is what actually held the section in place.
That check has been relied on all milestone.

### Ruling received mid-session: worktree currency, restructured

Provenance: advisor, 2026-08-15, superseding the dispatch half of the earlier
ruling.

**The orchestrator half is rescinded as written, and recorded rather than
deleted, with the reason: it was drafted against an assumed mechanism, not the
real one.** That reason is the point. A rule that cannot be performed reads, in
the record, exactly like a rule that is being ignored — so deleting it would have
lost the distinction between "we stopped doing this" and "this was never doable".

**The lane rule is promoted from safety net to the mechanism**, on three saves in
three dispatches. The brief names the base; the lane's first action verifies,
fast-forwards only if the base is a descendant and the tree is clean, stops
otherwise; and **the orchestrator confirms the lane's base verification appears
in its output before treating the work as started** — which is the orchestrator's
executable duty, replacing the one that was not.

**The T13 experiment is approved with three conditions**, and one dispatch is
data rather than adoption. The trade is named honestly and is worth restating,
because the answer is not obvious: **the auto flow gives mechanical isolation
with a stale base repaired by rule; the manual flow gives a chosen base with
isolation by instruction.** Neither dominates. The decision is made at the stop
report on both flows' evidence, and this report will carry whether isolation held
— checkable at review, by whether the lane's commits touched only lane paths —
and how the friction compared.

### Tick review of the four-way avoidance (T12) — APPROVE. TICKED

**Eight of the entry's rows were re-made by the reviewer rather than read**,
including both sides of the off-by-one (`CROWDED = 4` at `:240`, `CROWDED = 2` at
`:295` — a genuinely different assertion, which was the claim that mattered), both
sides of the applied/reported split, both sides of `mark::FOUR_WAY`, the rename
sweep, and the presence control pointed at an empty directory, **confirmed red
rather than silently clean**. `edit/mark.rs` verified byte-for-byte unchanged, so
no second implementation of the four-end count exists. The oracle arm ran for
real with `kicad-cli` present (0.40 s against 0.02 s without).

**It re-pinned mid-review**, because the verb surface (T16) landed on `main`
underneath it. None of those commits touched T12's files, but the reviewer
checked rather than assumed — and its note asks for that re-check to become
routine rather than a one-off.

**One discrepancy found and resolved rather than absorbed.** Row A2b claimed that
with the `:240` assertion removed, `:248` also catches the break. Removing *only*
the literal four-line `assert_eq!` leaves it caught by an adjacent `assert_ne!`
instead; removing that too reproduces the entry's claim exactly. The reviewer
judged it an imprecision in naming "the `:240` assertion" rather than a
fabricated result, confirmed the substantive claim by re-measurement, and **wrote
it into the verdict**. A review that silently absorbs a mismatch is not a review,
and this one did not.

**The task-text override was checked against the rule that permits it**, not
waved through. `task-entry-recording` requires the citation in the entry; the
citation is the geometric fact — P keeps exactly three ends after a correct
offset — which the reviewer verified. The check text in `tasks/M4.md` is now
corrected **beside the clause rather than over it**, so the next reader does not
re-derive what cost this implementer a substantial share of its reasoning.

**What it took on the entry's word, and said so**: A5/A5b, A9/A10, B1/B2/B5, and
A8 — where its own attempt at the break was caught by a different assertion than
the entry names, and where the entry already discloses that row as caught
elsewhere and not claimed for the oracle.

**WORKFLOW NOTE, tick review of the four-way avoidance (T12), verbatim:**

> The repo advanced with unrelated lane commits (T16) during this review; none
> touched T12's files, but re-pinning after the initial `git log` check was
> necessary to be sure — worth building that re-check into the reviewer's default
> routine rather than relying on one pin at the start. Separately,
> falsification-table rows that cite a single line number for a multi-assertion
> removal (A2b) cost real review time to disambiguate; recording exactly which
> lines were removed (not just the resulting catch line) would remove that
> ambiguity for the next reviewer.

**WORKFLOW NOTE, the four-way avoidance (T12) implementer, verbatim:**

> The brief's stated base was right and the worktree was wrong again — a third
> consecutive stale dispatch; the verification step earns its place, but
> something upstream is handing lanes worktrees that were never reset.
> Separately, T12's own check text contains a clause that cannot be satisfied as
> written ("`add_junction` at the terminus is still refused afterwards" — after a
> correct offset the terminus keeps three ends, and three ends take a junction),
> which cost a substantial share of the task's reasoning to resolve; a check
> whose literal reading contradicts the rule it guards should be corrected in the
> task file rather than left for each implementer to re-derive.

**WORKFLOW NOTE, the verb surface (T16) implementer, verbatim:**

> The brief said the goldens were "one per status" but the `adjusted` field the
> contract gained today is empty in every status a producer exists for, so a
> fifth golden had to be invented to see the new field rendered at all — a brief
> that adds a field should say which check is expected to see it non-empty. Third
> stale-base dispatch in a row: the pre-flight check the brief prescribes works,
> but the dispatcher is still not running it.

*(Both implementers independently reported the stale base as an upstream fault,
and both were right — see the rescission above. The verb surface's second point
is a fair hit on the orchestrator's brief: it named the field to render and did
not say which check would see it non-empty, and the lane had to invent a fifth
golden to see it at all.)*

7. **PROPOSED: the reviewer re-pins at the end of a review, not only at the
   start.** From the tick reviewer's note. **Recommendation: accept**, into the
   `tick-reviewer` definition beside the existing pin instruction. Two lanes ran
   concurrently all session and a review takes ten minutes or more, so a commit
   landing underneath a running review is the normal case rather than the
   exception — this review had one.
8. **PROPOSED: a falsification row records which lines were removed, not only
   where the failure surfaced.** Also the reviewer's, from the A2b ambiguity.
   **Recommendation: accept**, into `falsification-control`. The row was not
   wrong; it was under-specified, and disambiguating it cost the reviewer real
   time — which is exactly the cost the falsification table exists to avoid.
