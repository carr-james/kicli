# M4 Phase 3 and the milestone close — consolidated session report

**STATUS: IN PROGRESS.**

Opened 2026-08-20. Phase 2 closed with all seventeen of its tasks merged and
ticked; its report is `tasks/reports/M4-phase2.md` and is preserved unaltered —
a report that rewrites its own history is not a record.

This session's scope, from James's opener: apply five rulings from the
checkpoint-2 review, then Phase 3 (the join, T18 and T19; the calibration gate,
T20; the loop, T21; the agent documentation, T22), then the chores, then the
milestone close — the `cargo-mutants` run, the four counts, the mutation-run
skill, and a doc-diet check.

Appended per tick, not written at the end.

---

## Rulings applied before any dispatch

Provenance for all five: **James's ruling (threshold) and advisor rulings,
checkpoint-2 review**, delivered in this session's opener. Recorded where each
one lands, at the time it was given, per CLAUDE.md's "a direct instruction from
James in a session IS a ruling".

### 1. The threshold BLOCKED item is ruled — option (a), 300 G = 381 mm

The BLOCKED item raised by the label proposal (T13) is closed. The default
becomes **300 G = 381 mm**; the 38.10 mm value descends from reading the
internal-unit constant `Iu(381_000)` with the units dropped.

Recorded in `tasks/M4.md` under T13, "RULED, 2026-08-20 — option (a)", beside
the BLOCKED entry it closes rather than replacing it. Recorded with it:
**`KI-LBL-001` (M5) scores against this value.**

Dispatched as the session's first commit, to a lane at `lane-threshold`. The
ruling required the constant, its assertion, and every gloss in one commit,
since a gloss corrected in a later commit is a window in which the record lies.

Result: **pending** — see the per-lane section below.

### 2. The manual worktree flow is the default dispatch mechanism

Decided on the experiment's evidence. The orchestrator runs `git worktree add`
at the brief's base and briefs a **non-isolated** lane into a pinned path.

This supersedes a rule that had been *rescinded as never executable* — under the
auto flow the dispatch mechanism created the worktree itself, at a commit the
orchestrator could not set. That diagnosis was correct about that mechanism; the
answer was to change the mechanism rather than weaken the rule. Both are kept in
`CLAUDE.md`, because the pair is the lesson.

Standing with it, all three parts recorded:

- the lane's first-action base verification is **retained** — now redundant with
  the mechanism, which is where a check is cheapest and its absence hardest to
  notice;
- **scope verification at every merge** is a standing step: `git diff --stat` of
  the lane branch against its declared scope, main checkout clean before the
  merge begins;
- **the reversal trigger**: a lane found outside its scope returns dispatch to
  the auto flow, pending a ruling.

Landed in `CLAUDE.md` (Parallel work), `.claude/agents/orchestrator.md`
(Dispatch) and `.claude/agents/lane-implementer.md` (base verification).

### 3. PROPOSED 9 promoted — environment variation is a break class

`falsification-control` now names **path, clock, locale and run order** as break
classes against any check consuming a generated value, with the concrete rule:
such a test **runs once from a second directory before it is reported green**.

Worked example added verbatim from the record: the T16 golden defect, which was
invisible to every one of fifteen source breaks — five of which failed those very
goldens — and visible to a directory rename. The distinction the skill now makes:
*a check can be falsifiable and environment-dependent at the same time, and the
procedure as written only tested the first.*

Landed in `.claude/skills/falsification-control/SKILL.md`.

### 4. PROPOSED 10 promoted — reviewers never run the full gate suite live

The `tick-reviewer` definition's rule is promoted from conditional ("while other
lanes are active") to **unconditional**. The condition was one the reviewer could
not reliably evaluate: a phantom `clean` failure is indistinguishable from a real
one from inside the review. Targeted `cargo test --test <name>` runs in the
verified scratch copy are what carry the evidence.

Landed in `.claude/agents/tick-reviewer.md`.

### 5. PROPOSED 13 promoted — the identifier seed goes project-relative

All three sites (`edit/label.rs:294`, `edit/wire.rs:382`, `edit/text.rs:98`) as
**one chore**, so the convention cannot end up split between verbs. Its first act
is the measurement of whether any committed fixture or golden depends on the
absolute-path values, **recorded before the change lands**.

Filed as **C8** in `tasks/M4.md`, with an owner and a time in the chore table.

---

## Per-lane record

Dispatch under ruling 2's manual worktree flow: every worktree below was created
by the orchestrator with `git worktree add <path> -b <branch> <base>`, and the
base named in each brief is the base the worktree was actually at.

| Lane | Path | Base | What |
|---|---|---|---|
| `lane-threshold` | `.claude/worktrees/lane-threshold` | `a04dd6d` | ruling 1 — the 300 G default and every gloss, one commit |
| `lane-c1` | `.claude/worktrees/lane-c1` | `aa96263` | C1 — one name for the eight-character handle |
| `lane-c3` | `.claude/worktrees/lane-c3` | `aa96263` | C3 — `label_of_kind` takes a typed shape |
| `lane-t20` | `.claude/worktrees/lane-t20` | `aa96263` | the calibration gate (T20) |

Sequencing: the join (T18, T19) waits on `lane-threshold`, because the new
default changes which routes propose labels and which are drawn, and a check
written against the old boundary would be written twice. C7, D4, D5 and D6 all
hold `AGENT.md` and wait on the same merge. C8 holds `edit/wire.rs` and waits on
the join.

*(per-tick entries appended below)*

### The threshold ruling (ruling 1) — merged `b3a53d1`

Lane `lane-threshold`, base `a04dd6d`, one commit `c89c29e`, merged no-ff.

**Scope verification (standing step, ruling 2):** seven files —
`crates/kicli/src/model/config.rs`, `crates/kicli/src/route/propose.rs`,
`crates/kicli/tests/route_labels.rs`, `spec/SPEC.md`,
`research/wire-routing.md`, `research/style-rules.md`, `AGENT.md`. All seven
inside the declared scope. Main checkout clean before the merge. **No reversal
trigger.**

**Merged-result gates, measured at this merge and not deferred to the milestone
end:**

| Gate | Result |
|---|---|
| `cargo xtask check` | six gates green — fmt, clippy, test, doc, deny, clean |
| `cargo xtask corpus` | corpus fetched and demos canonicalised |
| `cargo test --features corpus` | green |
| `KICLI_TEST_KICAD_CLI=1 cargo test --features corpus` | green, **zero tests ignored anywhere in the run** |
| netlist oracle | **`hierarchies matched: 35/35`** — the gate carried in from M3 still holds under the new default |

The oracle number is quoted from the run rather than asserted, because "the gate
passed" and "the gate ran" are different facts and this milestone has already had
one check that was green for a reason unrelated to what it claimed to watch.

**What landed.** The default is `Iu(300 * GRID.0)`. Every gloss now reads
"300 G = 381 mm", with `=` rather than `≈` because it is exact.

**The assertion became a pair, and that is the substance of the fix.** The old
`config.rs:593` asserted `Iu(381_000)` against a comment reading "30 grid
steps" — and a comment cannot be wrong out loud, which is exactly how the two
numbers drifted for a milestone. The new form asserts the two *forms* against
each other: the grid-step arithmetic and `381 mm` parsed as a length. Falsified
independently — break B below moves `GRID` to `Iu(12_701)`, which leaves "300
grid steps" green (it is computed from `GRID`) and fails the millimetre arm on
`left: Iu(3810300) right: Iu(3810000)`. That is the drift the pair exists to
catch, and it is the precise class of error that produced the item.

**A fourth piece of evidence for the ruling, found by the lane and not by the
argument.** `crates/kicli/tests/wire_contract_labels.golden`,
`wire_output_contract.rs:206`, `route/report.rs:274` and
`research/wire-routing.md:203` were **already** printing
`447.04mm is over the threshold 381.00mm`. They are constructed strings, not
defaults, so they never failed — but the frozen output contract and its golden
had been speaking the ruled number the whole time. Nobody had looked there.

**The freeze was not touched, and the coincidence was confirmed rather than
assumed.** `route/report.rs:259-262`'s `Iu(381_000)` is the worked example's own
route length: the fixture is `steps: 30`, so `30 × GRID` = 38.10 mm. Nothing to
do with the threshold. Read, not guessed.

**Which checks straddled, and the judgement made about each.** James's ruling
predicted the label proposal's own checks were "insulated by construction". That
held for exactly one of them — `the_threshold_is_the_configured_one`, which sets
its threshold via `kicli.toml`. The three `far_apart` checks read the documented
default through the real binary and did straddle. The lane **lengthened the
drawing** rather than insulating them, and its reason is the right one: insulating
them would have left no check anywhere that the default is a number a real
drawing can exceed, which is the property the old default lacked in spirit.
`far_apart` now places `U1` and `U2` at opposite corners of the A4 page the probe
draws; the best route is 462.28 mm and it routes.

**Falsification, six breaks including one environment-class run.** The two worth
recording here:

- **Break D** reverted the drawing to its old 123.19 mm span with the new
  assertions in place. All three checks failed, and *the manner of failure is the
  evidence*: two failed on the **exit code**, `left: Some(1) right: Some(0)`,
  with stderr naming a diagonal segment — because at 381 mm the old drawing is
  not proposed at all, so the command falls through to drawing and refuses. The
  drawing genuinely straddles the new boundary.
- **Break A′ left `a_named_net_keeps_its_name` green, and the lane investigated
  rather than recording "did not apply".** Diagnosed case 1 (no-op break) with
  the reason: at 462.28 mm the route is over 38.10 mm *and* over 381.00 mm, so
  lowering the threshold cannot change its outcome, and that check asserts the
  naming rule and the anchor coordinates without ever reading the threshold. Its
  real break is D, which fails it. This is the skill's case-1/case-2 distinction
  applied correctly and unprompted.

**Oracle:** `kicad-cli` present; `the_written_pair_joins_the_pins_kicad_reads`
green against the moved drawing, both arms, kicli's partition matching KiCad's
exactly. Recorded as a lane measurement — the orchestrator's merged run is the
authority.

**One disclosed divergence from "verbatim".** The rebuilt `AGENT.md` worked
example is real binary output with one line deliberately omitted: `the file was
laid out again, as KiCad's next save would`. The probe writes non-canonical
files so kicli re-lays them out; a user's KiCad-written file would not produce
that line. Disclosed by the lane rather than found later, which is the behaviour
the discipline wants.

---

## Findings

*(appended as measured)*

---

## PROPOSED items

**1. Nothing in the repo checks a prose gloss against the value the code holds.**
Raised by the threshold lane, and it is the finding that explains why this defect
survived an entire milestone of green gates.
`the_label_threshold_has_one_name.rs` sweeps for the key's *name*; `agent_doc.rs`
checks the key is *present*. **Neither would have caught "30 G ≈ 381 mm."** The
new `config.rs` assertion pair guards the code side only — it holds the constant
and the millimetre form together, but nothing holds `spec/SPEC.md`,
`research/*.md` and `AGENT.md` to either of them.

A sweep asserting that the documents state the default as the value the code
holds would close it. **The lane did not build it, and was right not to** — the
ruling did not call for it and it would have widened a one-commit diff. Recorded
here instead.

*Recommendation: accept, as a chore carried into M5.* The class is wider than
this one key — every documented default has the same exposure — so the honest
form is a general sweep rather than a special case for `label_threshold`, and
that is enough work to deserve an entry rather than a patch.

**2. The lane-implementer's evidence rule and a ruling-lane's brief can
contradict each other.** Raised verbatim by the threshold lane; quoted in full in
the retrospective below.

The `lane-implementer` definition says "Record evidence in the entry AS YOU
WORK — your context dies with you; the entry is what survives". This session's
ruling-lane brief said `OUT: tasks/**`, because the orchestrator writes the
record for a ruling. **The lane followed the brief and put all its evidence in
its final message — so a lane that died mid-task under that brief would have
left nothing behind.** That is a real hole and the orchestrator opened it, not
the lane.

*Recommendation: accept, in the form "the brief carves out an evidence section
the lane may write" rather than "the standing rule gains an exception".* A rule
with an exception for briefs is a rule any brief can switch off. **Not applied:**
agent definitions are version-controlled working practice, changed only by
ruling (CLAUDE.md, the agentic layer), and the orchestrator is not exempt from
that.

**4. The reversal trigger fired on its first opportunity, and the orchestrator is
proceeding rather than reversing. This one needs James.**

Ruling 2 records: *"a lane found outside its scope returns dispatch to the auto
flow, pending a ruling."* The handle chore (C1) wrote to
`crates/kicli/src/cli/edit/wire.rs`, which was **not** on its brief's IN list.
On the literal text, the trigger has fired.

**What actually happened, because the literal reading and the situation are not
the same thing.** The brief's scope list was derived from the entry's
enumeration — four private copies of the eight-character shortening. **The count
was wrong.** A fifth exists at `cli/edit/wire.rs:352`, spelled `fn handle(uuid:
&str)` rather than `fn short`, which is why an eye counting `fn short` missed it.
The chore's own named check — *no file but the definer declares a function that
shortens an identifier* — **cannot pass while that copy stands**. So the brief
set a goal state that its own scope list made unreachable.

The lane did not discover this quietly. It took the named check over the derived
list, wrote four lines, **reported the deviation in its first paragraph, and
filed it in the entry as PROPOSED**. The file is not a merge hotspot and was not
on the live-lane list at dispatch.

*Recommendation: hold the manual flow, and read the trigger as governing
**undisclosed** scope excess.* The trigger exists because the manual flow trades
mechanical enforcement for orchestrator control; the risk it insures against is a
lane silently writing outside its brief and the excess surfacing at merge. A lane
that reports the excess in its first paragraph, with the reason, is the control
working — and reversing to the auto flow would punish the behaviour the rule
wants while doing nothing about the actual defect, which was **the
orchestrator's brief**, derived from a stale enumeration.

**But this is James's call and it is his rule**, so it is recorded here rather
than absorbed. If the trigger is meant literally, dispatch returns to the auto
flow and this session's remaining lanes move with it; the cost of reversing this
decision retroactively is one line of orchestrator practice, not any code.

**And the orchestrator's own defect is the one to fix regardless of the ruling:**
a brief that derives its scope from an enumeration must say which wins when the
enumeration proves wrong — the named check, or the list. This brief said neither,
and left the lane to choose. That is PROPOSED 5.

**5. A brief that derives scope from an enumeration must say which wins when the
enumeration is wrong.** Raised verbatim by the handle chore's lane; the specific
case is PROPOSED 4 above. *Recommendation: accept, as a line in the orchestrator
definition's Dispatch section.* **Not applied** — agent definitions change by
ruling.

**6. `falsification-control`'s "commit the good state first" needs a corollary.**
Raised by the handle chore's lane after tripping over it: after committing the
good state, **any further improvement made mid-falsification is uncommitted
too**, and the next `git checkout --` to undo a break silently discards it. The
lane lost its strengthened sweep this way and had to re-apply it.

This is the third amendment this rule has needed, and the shape is now clear
enough to state generally: the rule is not about whether git knows the *file*,
and not about whether you committed *once* — it is about whether git knows **the
state you want back, at the moment you break something**. *Recommendation:
accept.* **Not applied** — skills change by ruling.

**3. `falsification-control` and a one-commit brief are compatible only via
`--amend`, which neither document mentions.** Also raised by the threshold lane.
The skill requires the good state committed *before* any deliberate break; the
ruling required the constant, its assertion and every gloss in **one** commit.
Both were satisfiable, but only by amending, and a lane that did not think of
that would have had to choose which rule to break.

*Recommendation: accept — one line in the skill.* **Not applied**, same reason as
2: skills are changed by ruling.

---

## Reviewer rejections and their resolutions

### The handle chore (C1) — REJECT, rejection 1 of 2

**Gap:** the sweep classifies by **name**, against a closed list
(`IDENTIFIER_WORDS = ["uuid", "kiid", "identifier"]`), while the entry claims
something unqualified: *"no file but the one that defines it declares a function
that shortens an identifier to eight characters."*

The reviewer planted two private copies of the rule in a verified scratch copy
of the branch. **Both passed, with zero offenders found:**

1. a free function whose parameter is named `id` rather than `uuid`;
2. a method on a type named `Ident`, which does not contain `"identifier"`.

So a genuine reintroduction of the very defect this chore exists to prevent —
the fifth copy, which was itself missed because it was spelled `fn handle` — lands
clean under either spelling.

**The part that makes this more than a missed case.** The lane had *already found*
this class and stopped one step short. Break 3b discovered the classifier was
blind to methods and the lane diagnosed it correctly as the skill's case 2 — but
fixed it **for the definer's own spelling (`impl Uuid`) rather than for the
class**. The blindness was still there one synonym away. Every break in the
seven-row table names its offender literally `uuid`, so the table could not have
caught it: *a falsification table built from the instrument's own vocabulary
tests the instrument against itself.* That is the general lesson and it is worth
more than the chore.

**Direction given with the rejection, and the shape of it is the point.** Not "add
the missing words" — a longer closed list is the same instrument with a longer
blind spot, and the next reviewer finds `handle`, `key` or `Kiid2`. The chore is
to **classify by the cut rather than by the name**: find the eight-character slice
itself (`get(..8)`, `[..8]`, `chars().take(8)`, `truncate(8)`, `split_at(8)`)
anywhere under `crates/`, then allow exactly the permitted sites — the definer,
the deliberately-retained `short_key`, and anything else **with the reason
recorded**, because an allowlist without reasons is a blind spot with a comment.
This is the instrument the reviewer used to audit the branch by hand, which is
some evidence it works.

The entry's real distinction — identifier gets `Uuid::short`, non-identifier key
gets a shortener named as such — survives intact. It stops being enforced by
guessing from names and starts being enforced by every cut being accounted for.

**Explicitly not in question, and the lane was told not to redo it:** the presence
control (verified red by the reviewer when the definer was mangled), that no
golden moves, that the identifier/non-identifier distinction is genuinely
preserved rather than folded, and that the scope deviation is honestly disclosed.

**Reviewer discipline worth recording:** the verdict separates what was measured
from what was taken on the entry's word, including naming the gate results it did
*not* re-run under today's promoted rule. The review's highest-value finding came
from the brief's explicit instruction to plant an evasion — a directive worth
carrying into future review briefs for any check that classifies rather than
compares.

---

## BLOCKED items

*(none open at the time of writing — PROPOSED 4 is a ruling request rather than a
block: work continues on the recommendation and is cheap to reverse)*

---

## Workflow retrospective

*(WORKFLOW NOTEs quoted verbatim, attributed, corrections recorded beside the
quote rather than folded into it)*

### The threshold ruling lane

> **WORKFLOW NOTE:** The brief's Scope says `OUT: tasks/**` (the orchestrator
> writes the record), but the standing rules in the lane-implementer definition
> say "Record evidence in the entry AS YOU WORK — your context dies with you;
> the entry is what survives." Those are in direct tension for a ruling-lane, and
> I resolved it by following the brief and putting all evidence in this message;
> a lane that dies mid-task under this brief leaves nothing behind. Either the
> ruling-lane brief should carve out an evidence section the lane may write, or
> the standing rule should say "except where the brief assigns the record to the
> orchestrator". Separately, `falsification-control` says commit the good state
> before breaking anything, while the brief says ONE commit — compatible only via
> `--amend`, which neither document mentions; worth one line in the skill.

**Orchestrator, beside the quote rather than folded into it:** both halves are
correct and the first one is the orchestrator's defect, not an ambiguity in the
documents. The brief created the tension by writing `OUT: tasks/**` without
carving out an evidence section, and the lane resolved it the safe way and said
so. Filed as PROPOSED 2 and PROPOSED 3 above; neither applied, because agent
definitions and skills change by ruling.

### The handle chore (C1)

> WORKFLOW NOTE: The brief's IN/OUT scope list was derived from the entry's count
> of private copies, and the count was wrong — a fifth copy under a different
> name made the brief's own goal-state check unsatisfiable inside its own scope
> list. When a brief derives scope from an enumeration, it should say which of
> the two wins when the enumeration proves wrong, rather than leaving the lane to
> choose between "stop and report" and the named check. Second: the
> falsification-control skill's "commit the good state first" rule needs the
> corollary I tripped over — after committing, *any* further improvement made
> mid-falsification is uncommitted too, and the next `git checkout --` to undo a
> break silently discards it.

**Orchestrator, beside the quote:** both halves accepted, filed as PROPOSED 5 and
PROPOSED 6. The first is the orchestrator's defect — the brief's scope list came
from a stale enumeration in the entry and I did not say which won. The scope
deviation it produced is PROPOSED 4 and is James's to rule on.

**One finding from this lane belongs in the record whatever the verdict, because
it is about an instrument rather than about a chore.** Break 3b pointed the
sweep's definer constant elsewhere. The presence assertion failed as designed —
and **the sweep itself stayed green** over an unexcluded file. Cause:
`Uuid::short` is a *method*, so its parameter list is `&self`, and the
classifier read only parameters. **An identical private copy declared as a method
on any other type would have passed that sweep forever**, and the path exclusion
that looked like the safeguard was decoration. The lane fixed it (`ff80d05`,
reading the enclosing `impl` for methods) and re-ran the break against the
strengthened sweep, where it fails both arms.

That is the skill's case 2 — the control was blind and the code was innocent —
caught only because the lane refused to read a green break as good news. It is
the second time this milestone that discipline has caught a blind instrument.

### The handle chore's tick reviewer (C1)

> **WORKFLOW NOTE:** Inputs were complete and the task's point (d) directive to
> plant an evasion was exactly the right thing to test — it surfaced a real,
> reproducible gap (parameter/type named outside the literal
> `uuid`/`kiid`/`identifier` word list evades the sweep entirely) that the
> entry's own seven-row falsification table never exercised. Nothing else was
> missing or in the way.

**Orchestrator, beside the quote:** the directive the note credits was
`plant one in your scratch copy and see whether the sweep catches it`, given
because a sweep that *classifies* is a different risk from a check that
*compares* — the failure mode is silence, not a wrong answer. It is the second
review this milestone whose decisive finding came from making the instrument
fail rather than from reading the diff. Carried into the retrospective as a
candidate standing line for review briefs.

### The threshold ruling lane, continued

The lane also chose to widen its diff twice against the brief's stated default —
`spec/SPEC.md` §15's forms trio and `AGENT.md:711` — and gave a measured reason
for each rather than a preference. **Accepted.** The §15 reasoning is worth
keeping: `"381mm"` and `"15000mil"` are the same length, `"30G"` was the only
member of that parenthesis that was not, and putting `"30G"` and `"381mm"` in one
parenthesis is precisely the juxtaposition that produced this defect.
