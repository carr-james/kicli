# M4 Phase 2 — consolidated session report

**STATUS: IN PROGRESS — resumed session, checkpoint 1.** The first session
stopped on its token limit; this one reopened under the advisor's rulings on
that stop. Its record is preserved below under "Session 1", unaltered, because a
report that rewrites its own history is not a record. Session 2 begins at
"Session 2 — resumed".

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
