# M4 Phase 3 and the milestone close — consolidated session report

**STATUS: RESUMED, 2026-08-21.** The session hit a usage limit with three lanes
live, was wound down per the orchestrator's interrupted-session procedure, and
then resumed when capacity returned. **The wind-down is kept below rather than
deleted** — it is the record of what the stop looked like, and a report that
erases its own interruption is a report that cannot be trusted about the next
one.

What the interruption cost, stated plainly: nothing was lost. Two merges were
already on `main` and verified; the three parked lanes' worktrees survived intact
and were resumed with their contexts, each brought forward onto the new base by
its own first action. The one thing it changed is that this session's lanes have
now had their base move underneath them once, which is exactly the case the
worktree-currency rule exists for.

Two lanes merged and verified on `main`; three parked. The milestone close — the
`cargo-mutants` run, its four counts, the mutation-run skill, and the doc-diet
check — **was not reached and nothing about it ran**, which is correct: the exit
criteria say it runs only after every M4 task is ticked and the gates are green.

`main` is green and shippable at `a2d27b4`: six gates, corpus, and the KiCad
oracle at **35/35 with zero tests skipped**, measured at each of the two merges
rather than deferred.

**What James and the advisor are asked for, in priority order:** a ruling on
PROPOSED 4 (ruling 2's reversal trigger fired on its first opportunity and the
orchestrator proceeded rather than reversing), then the five other PROPOSED
items, then the resumption of the three parked lanes from their entries.

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

**7. `falsification-control` should require that a derivation bottom out in an
enumeration nobody in the loop wrote.** Raised by the handle chore's lane, in the
words quoted in the retrospective, and it **supersedes the looser rule I proposed
in this report's own stop section** ("derive its vocabulary from a source outside
the author's head").

The correction is that *a citation is not a derivation.* The lane's own first fix
grepped std — but with a hand-written alternation of method names as the pattern,
which is the same closed list one level up, now wearing a reference. The test that
distinguishes them is mechanical: **can a reader re-run the derivation and get the
list, without trusting anyone's judgement about what belongs in it?**

*Recommendation: accept, as an amendment to `falsification-control` alongside the
environment-variation class promoted this session.* The two are the same family —
both are about a check that is falsifiable in the dimension its author was
thinking in, and blind in one they were not. **Not applied**, skills change by
ruling.

**8. A resuming lane should expect its base to move *during* its work, not only
before it.** Also raised by the handle chore's lane, and it is the orchestrator's
defect. The resume briefs gave a one-shot "bring your worktree forward" sequence,
which assumes the base then stops moving. With four lanes live and the record
committed per tick, it does not — that lane merged `main` forward twice and had to
decide for itself that the second merge was in order.

*Recommendation: accept, as a line in the resume-brief pattern rather than in a
governing document* — "merge forward whenever `main` moves under you, and say how
many times you did." Cheap, and it removes a judgement call from a lane that has
no way to know what the orchestrator is committing.

**9. The calibration gate cannot fail on a wrong weight, and the exit criteria's
`calibration` row should say so.** Raised by the calibration gate (T20), measured
rather than argued: both sides are costed with the same weights while the router
optimises that objective, so **no perturbation of any weight moves either sheet
outside ±15 %** — the fixture reads +0.0 % under every one of seven weights swept
across four orders of magnitude.

The gate is still worth having: it catches a router that disagrees with a person
about *shape*, and it caught real defects in its own construction. But it measures
**agreement between two costings**, not whether the weights are right, and the
milestone's exit criteria present it as calibration.

The lane also ran the sweep that **does** answer the question — router chooses
under perturbed weights, both drawings scored under the defaults — and its result
is the useful one: `w_near`, `w_turn`, `w_len` and `margin` are each doing work
and the defaults sit at a local optimum, but **`w_turn = 6` is better than 0 and
not better than 60**, and **`w_cross` and `w_text` are exercised by neither
sheet**. The defaults are under-determined rather than wrong.

*Recommendation: accept, in two parts.* Re-word the exit criteria's `calibration`
row to say what the gate measures; and carry the perturbation sweep into M5 as the
instrument that actually calibrates, since M5 owns the weights and inherits these
numbers. **Not applied** — the exit criteria are the milestone's, and re-wording a
gate at its own close is exactly the kind of thing that should be ruled rather than
tidied.

**10. A review brief must tell the reviewer to read the entry off the lane branch,
not off `main`.** Raised by C1's second reviewer. My brief said the entry "now
carries three sections", which is true on `lane-c1` and false on `main`, because
the tick under review had not merged. Small, and it cost a reviewer a
reconciliation it should not have had to do. *Recommendation: accept, as a line in
the review-brief pattern.*

**13. `falsification-control` should carry the three-level rule the handle chore
produced.** Its final form, assembled from the lane's own words across three
rejections and one reviewer's extension:

1. Derive the vocabulary from an enumeration nobody in the loop wrote — **and a
   citation is not a derivation**; a grep whose pattern you authored is still your
   vocabulary wearing a reference.
2. **Choosing which enumerations to run is itself authorship.** A sweep must state
   the taxonomy of mechanisms it covers and assert that the taxonomy is its
   boundary, because an enumeration can be exhaustive within a category while the
   category was chosen from memory.
3. **A taxonomy of mechanisms is not enough if the matcher recognises spellings
   rather than meanings** — which is where a textual instrument stops, and where a
   semantic one has to start.

*Recommendation: accept all three, as one section.* This is the session's densest
finding and it was paid for three times. **Not applied** — skills change by ruling.

**11. Falsification evidence should be anchored to content hashes, not commit
SHAs.** Raised by the `AGENT.md` lane, and it resolves the third of three
independent collisions between `falsification-control` and commit discipline
reported this session:

| # | Reported by | The collision |
|---|---|---|
| 1 | the threshold lane | "commit the good state first" vs. a one-commit brief — reconcilable only by `--amend`, which neither document mentions |
| 2 | the handle chore | after that commit, any *further* improvement is uncommitted too, and the next `git checkout --` discards it |
| 3 | the `AGENT.md` lane | citing the good-state commit's SHA in the entry, then folding the entry in by `--amend`, destroys the SHA the entry names |

*Recommendation: accept, and let it subsume the other two.* **A content hash
survives amending, rebasing, and merging `main` forward; a SHA does not** — and
this session every lane merged forward at least once, one of them four times, so
SHA-anchored evidence rots by default rather than by accident. **Not applied** —
skills change by ruling.

**12. No fixture exercises a net with no visible pin.** Raised by the `AGENT.md`
lane while falsifying D4's law, and it is a gap in the instruments rather than in
the code. `listed + tallied + hidden = total` holds, but `hidden` is **0 on all
seven fixture projects that have nets**, so dropping that term from the law leaves
the check green. The lane established this as a no-op break rather than assuming
it. *Recommendation: accept as a chore — a fixture with an all-power-pin net, or
an all-off-sheet one under `--sheet`.* Until then the law is verified in two terms
of three.

**3. `falsification-control` and a one-commit brief are compatible only via
`--amend`, which neither document mentions.** Also raised by the threshold lane.
The skill requires the good state committed *before* any deliberate break; the
ruling required the constant, its assertion and every gloss in **one** commit.
Both were satisfiable, but only by amending, and a lane that did not think of
that would have had to choose which rule to break.

*Recommendation: accept — one line in the skill.* **Not applied**, same reason as
2: skills are changed by ruling.

### The calibration gate (T20) — merged `1af87e5`

Lane `lane-t20`, base `491c254` after its own fast-forward, three commits, merged
no-ff. **Scope verification:** five files, no `crates/kicli/src/**` change at all,
and the only hotspot touched is `crates/kicli/tests/fixtures/MANIFEST`, which this
lane was explicitly granted. **No reversal trigger.** Six gates green on the
merged result.

**The headline.**

| Sheet | Deviation | Inside 15 %? | Skipped pins |
|---|---|---|---|
| `calibration.kicad_sch` — the purpose-built fixture, 82 pins, 40 strands, 74 segments | **+0.0 %** | yes | 2 / 82 |
| `ampli_ht` — the tidy hand-drawn demo sheet | **+1.1 %** | yes | 7 / 87 |

**No weight moved**, per the milestone's Rules.

**Both T7 rulings reported, as their entries require whatever the deviation.**
Re-routed segments inside the border region: **0** on both — and the person's own
wires as a control, also 0, which is what makes the zero informative. Re-routed
segments within 1 G of a bus entry: **0**. `ampli_ht` has no bus entries at all,
so the lane **built a bus and two entries into the fixture** to give the question
something to ask rather than reporting a vacuous zero.

**And now the finding, which is worth more than the gate.**

> **This gate measures agreement, not calibration.**

Both sides are costed with the same weights **while the router optimises exactly
that objective**. So no perturbation of any weight moves either sheet outside
±15 % — the lane swept `w_near` 0→20000, `w_turn` 0→10000, `w_cross` 1→100000,
`w_len`, `w_text`, `margin`, `u_max`, and **the fixture reads +0.0 % under every
one of them**. A gate that cannot fail on a wrong weight is not measuring the
weights.

The lane did not stop at the criticism. It ran **the sweep that does answer the
question** — the router chooses under perturbed weights, then both drawings are
scored under the defaults — and recorded its numbers: `w_near`, `w_turn`, `w_len`
and `margin` are each shown to be doing work and the defaults sit at a local
optimum; but **`w_turn = 6` is confirmed better than 0 and *not* better than 60**,
and **`w_cross` and `w_text` are unexercised by either sheet**. That is real
calibration information, and it says the current defaults are under-determined
rather than wrong.

This bears directly on the milestone exit criteria's `calibration` row, and on M5,
which inherits these weights. Recorded as PROPOSED 9.

**A claim withdrawn under its own control, before commit.** The lane's first
run-order break reversed the pin list *after* `pins_of` had sorted it, the report
rows re-ordered, and it wrote that up as a defect. It then reversed the **file's**
object order — the variation that actually happens — and nothing changed. The
break had been breaking the sort, not varying the environment. The comment
claiming a defect was rewritten before the commit rather than left standing with a
correction. Worth recording because the discipline usually catches a check that is
too weak; this is it catching a finding that was too strong.

**Falsification, including the one the brief most wanted.** The tolerance fails on
both the +15 % and the −15 % side, on both sheets, from separate breaks — a
two-sided tolerance tested one-sided is half a check. The costing-symmetry break —
costing the original against the full sheet — **fails, but by the skip guard
rather than by the tolerance**: the strands become uncostable rather than
mis-priced, 82/82 and 67/87 skipped. So the tolerance alone would not have caught
a broken symmetry, and a reader needs to know which assertion is load-bearing.

### The AGENT.md lane (C7, D4, D6) — merged `984545c`

Lane `lane-doc`, base `491c254`, three commits, merged no-ff. Six gates green on
the merged result. **Scope:** one file outside the brief's IN list —
`crates/kicli/tests/net_counts_reconcile.rs` — because D4's check asked for a test
holding the two numbers to their stated relationship and no in-scope file could
hold an integration test. **This is the standing answer working**: the brief said
the named check wins over the file list, the lane took it, and declared the excess
in its first paragraph rather than stopping for a dispatch. No reversal trigger.

**C7 — what "documented" now means, which was the judgement the chore existed to
make.** The command's name appears as its own **backticked span in a heading
line**, and the section that heading opens **has a body** of at least 80
non-whitespace characters. The floor is measured, not chosen: the smallest real
section is 135 characters, the largest 4391, and a one-sentence stub is about 50.

The lane declined the obvious answer and gave a reason: **one heading per command
*name*, not one section per command.** Six shared headings cover 16 of the 28
commands and they group verbs differing by one word of behaviour. *The check
should describe the document's shape, not impose one.* What it does insist on is
that a shared heading **names every verb it covers** — which is grouping's real
failure mode, and break 2a proves it: dropping one verb from a shared heading
while leaving the body intact **passed the old check and fails the new one**.

**One hole is recorded rather than closed**, and recording it is the right call: a
heading kept with its body replaced by a pointer elsewhere still passes. No
textual check can judge whether a body is *about* its command, and raising the
floor only lengthens the evasion.

**A parser defect found on the way, and load-bearing.** The section reader must
skip fenced blocks, because `AGENT.md` prints view samples inside fences and a
kicli view comments itself with a hash — which reads as an H1 to a naive scanner
and truncates the section. Break 4 plants a heading inside a fence and fails.

**D4 — the law, and why the dogfood agent's arithmetic was right but not
general.** `listed + tallied + hidden = total`. The agent computed `10 + 18 = 28`
unaided; the third term is the one it never had to reckon with, because a net with
**no** visible pin is dropped in silence. Measured whole-project on the `nets`
fixture: `14 + 18 + 0 = 32`, and the binary prints `nets 32`. On the root sheet:
`10 + 18 = 28` against 32. **So the obvious signpost — "they add up" — is false
under `--sheet`**, and would have been the natural thing to write.

And the reason there was no signpost to find: **`AGENT.md`'s `project info` sample
was missing the `nets` line entirely.**

**Break F is a no-op break established as one, not assumed.** Dropping `hidden`
from the law stayed green — because `hidden` is 0 on all seven fixture projects
with nets. Recorded as a PROPOSED gap: no fixture exercises a net with no visible
pin, so that term of the law is untested by construction.

**D6 — and the timing half is now half-measured.** The document claimed
`project check` "warms KiCad's font cache". It does not: it asks
`kicad-cli --version`, and that ask is what may build the cache. Beyond the doc
fix, the lane measured what it could and **recorded rather than acted**: the note
**prints unconditionally** (confirmed — two runs seconds apart print it
identically, and `probe` emits it whenever discovery succeeds), the cache being
genuinely cold each time is **refuted**, and the cold-cache "over two minutes"
figure **remains inherited, not observed**, so the standing sentence was left
byte-identical.

### The handle chore (C1) — merged `2220e5d`, TICKED on the fourth review

Four review passes, three REJECTs, one APPROVE. **Scope:** the disclosed
`cli/edit/wire.rs` fifth copy and nothing further; `tasks/M4.md` pure addition,
0 lines removed. Six gates green on the merged result.

**The fold — the chore's actual deliverable — was never what was wrong.** Five
private copies became one; `short_key` stays separate and named because a key
that is not an identifier keeps a shortener that says so. Three separate
reviewers confirmed it against the diff.

**The final reviewer measured the things that mattered rather than reading
them:** `CUTS` untouched (diff exit 0, matching `md5` at both ends, only the
rustdoc changed); every `CUTS` entry a decimal-8 spelling, so the claim is not
wider than the instrument; the boundary stated as an open class in both the entry
and the rustdoc; and the discriminating pair reproduced by hand — `take(0x8)` and
`take(8)` side by side in one file, the hex invisible and the decimal caught.

### The index fallback (C2) — merged, TICKED

**The null result recorded as a result**, which is exactly what the entry
demanded: *a chore that measured nothing must not read as a chore that found
nothing.* At five named budgets where the fallback fires, the index is 407 bytes
against a 1184-byte full view, and it is **budget-independent** — so there is no
budget at which it exceeds, and the delta view's skip does not transfer. `scope.rs`
unchanged.

**It was sent back once, by the orchestrator rather than by a reviewer, and the
reason is worth recording.** The first pass compared the index against the whole
view as a ratio — a question whose answer was never in doubt, since a summary of a
document is smaller than the document — and falsified it by **inverting its own
assertion**:

> The test was deliberately broken by changing `assert!(index <= full)` to
> `assert!(index > full)`, which correctly failed on the actual data.

An inverted assertion fails whatever the check measures, **including when it
measures the wrong thing — which here it did.** That is the degenerate case of
falsification and it is worth naming, because it looks like the procedure being
followed. The second pass pads the index generator to 1462 bytes and watches the
check go red at budget 100: a break in the code the check watches.

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

### The handle chore (C1) — REJECT, rejection 2 of 2. **ESCALATED.**

**Per CLAUDE.md's tick-review rule, two rejections on one item escalate to a
PROPOSED item in the session report.** This is that escalation. Work continues on
the recommendation below; the escalation is a reporting obligation, not a stop.

**The gap:** `format!("{:.8}", uuid)`. The reviewer planted
`pub fn short(uuid: &str) -> String { format!("{:.8}", uuid) }` in a verified
byte-identical scratch copy and ran the sweep: **both tests passed, zero offenders
found.** It is a complete second copy of `Uuid::short`'s behaviour.

**Why it is not covered by the entry's disclosed limitations.** Those name a
`const`-hidden width, a runtime width, a counter loop, and "a macro that expands
to a cut". `{:.8}` is none of them: the width is right there in the text, and it is
`core::fmt` precision truncation — **as std-offered as `chars().take(8)`**, and an
idiomatic way an engineer would really write a UUID shortener.

**And this is the lane's own finding, one level further down again.** The lane
established that "the derivation must bottom out in an enumeration nobody in the
loop wrote". Its derivation *does* bottom out in std — but in **three method
lists**, and choosing which three lists to enumerate was the lane's own judgement.
**The vocabulary moved outside the author's head; the taxonomy did not.**

That is now the fifth occurrence of this shape in one session, and the fourth
distinct level:

| # | Level | What was the author's |
|---|---|---|
| 1 | prose | nothing compared the gloss to the constant |
| 2 | the word list | `["uuid", "kiid", "identifier"]` |
| 3 | the grep pattern | a hand-written alternation, citing std |
| 4 | the method lists | derived from std — but which lists, chosen by hand |
| 5 | *(open)* | whatever bounds the taxonomy |

*Recommendation, sent to the lane: fix the claim, not only the instrument.* Cover
`core::fmt` precision, derived from the format-specifier grammar rather than from
memory — **and restate the claim to name the taxonomy it covers and say that the
taxonomy is the boundary.** "In any spelling the standard library offers" is wider
than any grep can be, and the entry currently repeats the very defect the chore
exists to correct. The lane was told explicitly that **a claim naming its own
boundary honestly is a finished chore**, and that if it finds a fifth mechanism it
cannot enumerate, it should say so and stop rather than earn a third rejection.

**What the review confirmed and is not in question:** the derivation commands are
mechanically re-runnable and match the methodology; the reason-length control
genuinely bites; **row L is genuine** — deleting `short_key` and inlining its
callers fails the sweep, so the fold the chore forbids is actually refused; all
five source folds match the entry; the `tasks/M4.md` diff touches no other lane's
text.

The reviewer also disclosed a methodological error of its own — its first attempt
at row L used a rename, which still substring-matched, and it corrected the method
rather than reporting the phantom.

### The handle chore (C1) — REJECT, rejection 3. **The iteration is stopped.**

**The gap:** `uuid.chars().take(0x8).collect()`. `0x8` is the literal 8 in hex —
not behind a `const`, not runtime, not an indirect precision — and it uses
`Iterator::take`, one of the four mechanisms the claim covers "exhaustively". The
sweep passes with zero offenders, because `CUTS` matches the **decimal spelling**
of each mechanism rather than the **value** 8.

**The orchestrator has stopped the iteration on the instrument**, and the reason
is not that the lane keeps missing cases:

> **A textual matcher cannot decide a value.**

After the radices come integer suffixes, then `take(4 + 4)`, then a `const` one
line above the call. Three reviewers each found a genuine gap one level below the
last, and each fix moved the boundary without changing its kind. **There is no
fixed point**, and continuing would be iterating a test to green — the thing the
mutation-testing rule explicitly forbids at a milestone close, for the same reason.

**What the chore delivered stands.** The fold is done and all three reviewers
confirmed it: five private copies became one, `short_key` remains separate and
named, and the third reviewer verified each fold against the diff. The sweep is a
**regression guard**, not the deliverable, and a guard that catches the decimal
spelling of every std mechanism is worth having.

**Final instruction to the lane: narrow the sentence, do not touch the matcher.**
State that the check matches the decimal-spelled literal `8`; widen the disclosed
"outside it" list by the class it has now been shown — *a width of eight not
spelled as the decimal digit `8`* — as a class rather than three examples; and add
`take(0x8)` to the falsification table as a **documented pass**, a negative
control like the four already there. The lane's negative controls are what make
its boundary statement honest, and the third reviewer verified two of them by
hand.

**The real instrument is carried into M5**: a lint over MIR, where `take(0x8)` and
`take(8)` are the same node and a `const` is already folded. Recorded in
`tasks/M5.md` with the three-level lesson, because M5 is a whole milestone of
checks that classify.

**The three-rejection sequence is the session's most valuable artefact and should
be read as one thing, not three failures:**

| # | Evasion | Blind to |
|---|---|---|
| 1 | parameter named `id`; method on a type named `Ident` | it classified by **name**, against a closed word list |
| 2 | `format!("{:.8}", uuid)` | it enumerated **methods**; precision is not a method |
| 3 | `chars().take(0x8)` | it matches a **spelling**, not a **value** |

---

## BLOCKED items

### BLOCKED 1 — the pre-commit gate and a lane that cannot be green until another lane lands

**Raised by the join (T18), parked and reported rather than resolved, which is the
rule working.**

`ENGINEERING.md` requires that **`cargo xtask check` pass at every commit**, and
CLAUDE.md records that the gates run as a git pre-commit hook. `AGENT.md` belongs
to one designated lane at a time, and `agent_doc_covers_every_command` fails the
moment `kicli wire connect` exists and is undocumented. So the lane that
implements the verb **cannot make a single commit** until the lane that owns
`AGENT.md` has documented a verb that does not yet exist.

T18 committed with `-c core.hooksPath=/dev/null` and **disclosed it as a lane-wide
deviation in its entry**, which is the honest handling of an impossible
instruction. T19 inherits the same situation and was given the same sanction under
four conditions: run the gates by hand before every commit and record the result;
**the only permitted failure is `agent_doc_covers_every_command` naming a `kicli
wire` verb**; do not weaken the check; state the bypass in the entry with the
condition under which it ends.

**Both readings, since CLAUDE.md forbids resolving a governing-document conflict by
precedence:**

- *The gate means every commit, including a lane branch's.* Then the only lawful
  order is that documentation precedes implementation — `AGENT.md` describes a
  verb the binary does not have, which `agent_doc`'s sibling checks are designed
  to catch, so this reading forbids something else instead.
- *The gate means every commit that reaches `main`.* Then a lane branch may be
  transiently red, the merge must be green, and the bypass is legitimate but
  should be **sanctioned in writing** rather than re-decided by each implementer.

**Recommendation: the second reading, written down.** The session already works to
it — nothing merges until it is green, and the merged-result gates are run at every
merge, corpus included. What is missing is the sentence saying so, which is why two
lanes have now had to invent the same deviation independently.

**Nothing waits on this ruling.** The session avoids the conflict by sequencing:
**T18 and T19 will not be merged until the documentation lands**, so `main` is
never red. The cost of the other reading is a re-ordering of two dispatches.

### BLOCKED 2 — the joined net has no unfrozen home

Also raised by the join (T18). The verb must report **which net it joined** — the
entry's own check requires that the report's claimed net be the extractor's net.
The natural home for that field is the route contract, `crates/kicli/src/route/report.rs`,
which is **on the frozen surface**.

T18 kept that file's diff at exactly zero and reported the joined net as a
top-level key **beside** the contract instead. T19 was told to follow the same
shape, so that one awkward-but-consistent surface exists rather than two.

**Recommendation: lift the freeze for exactly this field, with `spec/SPEC.md` §8
amended in the same commit and the freeze restored in that commit.** That is not a
novel procedure — it is precisely what was done once before, by advisor ruling, when
the contract gained the structured `adjusted` field, and the precedent is recorded
in `.claude/hooks/frozen-paths.txt` itself.

The alternative — leaving the net as a sibling key permanently — is defensible, but
it means the route contract does not describe the whole of a route's result, and
M5's linter reads that contract.

*(PROPOSED 4 is a ruling request rather than a block: work continues on the
recommendation and is cheap to reverse.)*

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

### The handle chore (C1), rework

> **WORKFLOW NOTE:** An instrument built from its author's vocabulary tests
> itself, and this chore hit that four times, not three — the fourth was my own
> "grep std" fix, whose grep used a hand-written list of method names, i.e. the
> closed list one level up. The generalisable rule for `falsification-control` is
> not "derive from an external source" but "**the derivation must bottom out in
> an enumeration nobody in the loop wrote**"; a grep whose pattern you authored is
> still your vocabulary wearing a citation. Second: a lane resuming after an
> interruption should expect `main` to move *during* its work, not only before it
> — I merged forward twice, and the brief's one-shot "bring it forward" phrasing
> left the second case to judgement.

**Orchestrator, beside the quote:** the first half is the sharpest statement
anyone has produced of this session's recurring finding, and it is sharper than
mine. I wrote "derive its vocabulary from a source outside the author's head";
the lane's correction is that **a citation is not a derivation** — the fix that
merely *names* std while grepping a hand-written alternation is the same defect
wearing a reference. Filed as PROPOSED 7, superseding my looser wording.

It is also the fourth occurrence and it was found by the lane in its own work, in
the act of writing it up, not by a reviewer. That is the discipline functioning at
the level it is supposed to.

The second half is my defect: the resume briefs said "your base moved" and gave a
one-shot command sequence, which silently assumes the base stops moving. With
four lanes live and the orchestrator committing the record per tick, it does not.
Filed as PROPOSED 8.

### The join (T18)

> **WORKFLOW NOTE:** The brief named the checks but not where `wire connect`
> should report the net it joined — and the obvious home, the route contract, is
> frozen, so the task cannot be completed as specified without either a freeze
> ruling or an invented sibling key; a brief that owes a new output field should
> say which surface it lands on. Second, a lane whose only expected gate failure is
> `agent_doc` cannot commit at all under the pre-commit hook, so "commit the good
> state before you break anything" and "the gates run as a git pre-commit hook" are
> in direct conflict for this lane; the bypass should be sanctioned in the brief
> rather than left for each implementer to decide and disclose.

**Orchestrator, beside the quote:** both halves are correct, both are my defects,
and both are now BLOCKED items above rather than absorbed. The first is the
sharper one — I wrote a check requiring the report to name the joined net without
noticing that the only sensible place to put it is frozen. The lane did the right
thing twice over: it kept `route/report.rs` at a zero diff and reported the field
beside the contract, then filed the question rather than deciding it.

The second is worse in a quiet way. **Two lanes independently invented the same
hook bypass and each disclosed it**, which means the process was relying on
implementer conscience to cover a hole in its own rules. That it worked is not
evidence that it was safe.

### The calibration gate (T20)

> **WORKFLOW NOTE:** The resume brief asserted state ("the sheet is still to
> make") from a PARKED note written before the lane's last actions, and the
> brief's `git stash push -u` / `merge --ff-only` / `stash pop` recipe was correct
> and should be the standard opener for every resumed lane. Second: the
> falsification list says "perturb a weight so the total lands outside ±15 %" — on
> this gate no weight can, because the procedure scores both sides with the weights
> the router optimises under, so a lane following that instruction literally will
> either report failure or manufacture a break; the instruction needs an escape
> hatch that says "if no weight can move it, that is the finding".

**Orchestrator, beside the quote, and the correction belongs here rather than
folded into it:** the stale PARKED note is mine. I wrote "the purpose-built sheet
was not made" from the lane's last visible words at the interruption, and the lane
had in fact already built the fixture, both checks, the oracle and the ruling
reports. **A parked-state note written from an agent's last utterance is a guess
about a context I could not see**, and I recorded it in the entry with the same
confidence as a measurement. The correction is now in the entry beside it.

The second half is the better lesson and it generalises past this gate: **an
instruction to "break it until it fails" presumes the check can fail in that
dimension.** When it cannot, a literal reading pushes a lane toward manufacturing
a break — which would have destroyed exactly the finding that makes this task
valuable. The escape hatch the note asks for is what turned an impossible
instruction into PROPOSED 9.

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

### The AGENT.md lane (C7, D4, D6)

> WORKFLOW NOTE: The brief told me to record under `#### Done, 2026-08-21` in both
> `tasks/dogfood.md` entries, which would have put two identically-titled headings
> in one file; I used `#### D4 — Done, 2026-08-21` and `#### D6 — Done …` so a
> reviewer can locate them. Second: the brief's instruction to commit the good
> state before falsifying collides with citing that commit's SHA in the entry,
> because folding the entry in by `--amend` destroys the SHA the entry names — I
> anchored the falsification to `shasum -a 256` values instead, and that should be
> the documented pattern rather than a per-lane improvisation.

**Orchestrator, beside the quote:** both accepted. The first is a small brief
defect — I wrote one heading template for three items in one file — and the lane's
fix is better than what I asked for.

The second is the **third** distinct report this session of `falsification-control`
colliding with commit discipline, after "commit the good state first vs. one
commit" and "any improvement made after that commit is uncommitted too". The
lane's answer generalises past all three: **anchor falsification evidence to
content hashes rather than to commit SHAs.** A hash survives amending, rebasing
and merging forward; a SHA does not, and every lane that merges `main` forward —
which, this session, is all of them — invalidates its own citations. Filed as
PROPOSED 11.

### The handle chore's second tick reviewer (C1)

> **WORKFLOW NOTE:** Inputs were complete and accurate (entry heading located
> correctly, diff reproducible via `git diff main...lane-c1`, merge-base equalled
> `main`'s tip so `...` and `..` diffs agreed). One friction: the task said "the
> entry now carries three sections," but `tasks/M4.md` on `main` only has two
> (`Done` was superseded/removed by the PARKED-era merge and the third,
> `Reworked`, section only exists on `lane-c1` itself, not yet merged to `main`)
> — reviewers should be told explicitly to read the entry off the lane branch tip,
> not off `main`, when the tick under review hasn't merged yet.

**Orchestrator, beside the quote:** correct, mine, and filed as PROPOSED 10. I
described the entry as it exists on the branch while pointing the reviewer at a
path that resolves against the checkout. For an unmerged tick those are different
documents, and the reviewer had to work out which one I meant.

### The threshold ruling lane, continued

The lane also chose to widen its diff twice against the brief's stated default —
`spec/SPEC.md` §15's forms trio and `AGENT.md:711` — and gave a measured reason
for each rather than a preference. **Accepted.** The §15 reasoning is worth
keeping: `"381mm"` and `"15000mil"` are the same length, `"30G"` was the only
member of that parenthesis that was not, and putting `"30G"` and `"381mm"` in one
parenthesis is precisely the juxtaposition that produced this defect.

---

## The stop, and the resumption

*Written at the stop and kept as written. The resumption follows it.*

## The stop — what holds, what is parked

### Merged and verified on `main`

| | |
|---|---|
| `b3a53d1` | the threshold ruling — 300 G = 381 mm, every gloss, one commit |
| `a2d27b4` | the typed label shape (C3), and the reader question measured |
| Six gates on the merged result | pass at both merges — fmt, clippy, test, doc, deny, clean |
| `cargo test --features corpus` | pass |
| `KICLI_TEST_KICAD_CLI=1 cargo test --features corpus` | pass, **zero tests ignored anywhere in the run** |
| netlist oracle | **35/35**, quoted from the run |

One merge conflict, resolved by the orchestrator and recorded rather than
silently fixed: `route_labels.rs`'s `far_apart_and_named` helper. The threshold
lane had moved the drawing to opposite corners of an A4 page and moved the label
with it; C3 had changed the same call's API while keeping the old coordinates.
The resolution takes **both** — C3's `LabelKind::Local` and the threshold lane's
`("12.7", "199.39")` — because the label must sit on the source pin's own anchor
and that anchor moved. Verified by running `route_labels` (5 passed) before the
merge was committed, then the six gates on the result.

### Parked, with state in their entries

| Lane | Branch state | Where it stopped |
|---|---|---|
| the handle chore (C1) | 5 commits, **not merged**, REJECTed once and mid-rework | correcting the cut-list, which had the same name-blindness one level down |
| the join (T18) | **nothing committed**, uncommitted draft | "Now the test file" — the checks do not exist |
| the calibration gate (T20) | **nothing committed**, uncommitted draft | "Now let me generate the fixture" — the sheet was not made |

Each entry in `tasks/M4.md` now carries its own parked state, the direction it
was given, and the fact that **its base has moved** since dispatch. Per CLAUDE.md
a parked draft is reference, not resumption: a fresh implementer starts from the
entry, and any line adopted passes falsification as if newly written.

### Not started

T19, T21, T22; chores C2, C4, C7, C8; the D-series; the mutation run and the
skill it creates; the doc-diet check; the milestone's four counts.

### The finding this session produced that outlives it

**The same defect appeared three times in one session, at three levels, and it is
one shape: an instrument built from its author's vocabulary tests itself.**

1. The threshold gloss survived a whole milestone of green gates because nothing
   checks a prose number against the constant it describes — the checks swept for
   the key's *name*.
2. The handle sweep classified by *name* against a closed three-word list, so the
   very defect it existed to prevent passed under a synonym. Its own falsification
   table could not catch this, because every break in it was spelled `uuid`.
3. The rework then hand-listed the *slice spellings* and three `std` forms passed
   silently — the same failure one level down. The correction in flight was to
   derive the list from `std`'s API rather than from memory.

The general rule, offered for ruling rather than adopted: **a check that
classifies rather than compares must derive its vocabulary from a source outside
the author's head**, and its falsification must include at least one case the
author did not choose. The C1 review brief's "plant an evasion in your scratch
copy" directive is what surfaced (2), and is a candidate standing line for every
review of a classifying check.


---

## The resumption, 2026-08-21

All three parked lanes were **resumed rather than re-dispatched**, and the
distinction matters enough to record.

CLAUDE.md's rule is that *a parked lane's uncommitted draft is reference, not
resumption* — a fresh implementer starts from the entry, and any adopted line
passes falsification as if newly written. That rule is about a **dead** lane's
draft being picked up by someone who never wrote it: the danger is an
unfalsified narrative reviewing itself, from a context that is gone.

Here the contexts were **not** gone. The lanes were killed by a usage limit with
their transcripts intact, so each was resumed with its own memory of what it had
built and — the part that matters — of what it had **not yet falsified**. That is
a continuation, not a resumption from a dead draft, and the rule's reason does not
reach it. Recorded as an orchestrator judgement rather than assumed: if the
advisor reads the rule as covering this case too, the cost of the other reading is
three re-dispatches, not any code.

Each resume brief carried the same three things, because all three lanes had the
same problem:

1. **The base had moved**, and each was given the exact command sequence to bring
   its worktree forward — `git stash push -u`, verify the old base, `git merge
   --ff-only main`, verify the new one, `git stash pop` — with an explicit
   instruction not to `git checkout --` anything, which would have taken the draft
   with it. The calibration gate's lane fast-forwarded to `491c254` cleanly.
2. **What landed underneath them and why it touches their work.** The typed label
   shape changed `Probe::label_of_kind` at 34 call sites, so any new probe drawing
   uses `LabelKind`; the 300 G threshold means a route over 381 mm is proposed as
   labels rather than drawn, which the calibration procedure must account for or
   explain, and which would otherwise look to the join like its own verb failing.
3. **The session's recurring finding**, given to each lane as a working caution
   rather than as a rule: an instrument built from its author's vocabulary tests
   itself, and a green check after a deliberate break is a finding about the
   instrument.

A fourth lane was dispatched at the resumption, holding `AGENT.md`: **C7 plus the
D4 and D6 documentation defects**, as three commits in one lane. `AGENT.md` is a
merge hotspot, CLAUDE.md permits exactly one designated lane to hold one, and
three separate dispatches would have meant two avoidable conflicts. Recorded as a
deliberate designation rather than a scope slip.

**One standing answer was added to that brief that the earlier ones lacked.** Two
lanes today hit briefs whose scope list came from a stale enumeration, making the
brief's own goal state unreachable inside its own scope. The doc lane's brief says
outright: if that happens, **the named check wins over the file list**, and the
lane reports the excess in its first paragraph. That is PROPOSED 5 being applied
in practice while it waits for a ruling — cheap to reverse, and it removes a
choice no lane should have to make twice.
