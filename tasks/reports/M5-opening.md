# Consolidated report — M5 opening

**Session goal:** apply the M4-close rulings, land the milestone-boundary
changes, and draft the M5 plan for James's ratification. **It does not begin M5
implementation.**

**Stop condition:** the draft plan, delivered, with the BLOCKED items that need
James. Reached.

---

## 1. Per tick

| Task | Lane | What landed | Evidence | Verdict |
|---|---|---|---|---|
| the M5 opening entries | orchestrator | `tasks/M5/` created: `RULES.md` and the three ruled opening entries | `a1e25ad` | n/a — orchestrator record |
| the M4-close rulings | orchestrator | `ENGINEERING.md`, `CLAUDE.md`, three agent definitions, two skills; `tasks/M5.md` migrated into `tasks/M5/` | `7a4b779` | n/a — orchestrator record |
| the joined net's contract field (`opening-1`) | `lane-o1` | **PARKED.** Entry only, 159 lines: the BLOCKED item, its measurement, and banked reconnaissance | `20f48b8`, merged `dd6bd23` | no tick — task not complete |
| the obstacle walk's direction (`opening-2`) | `lane-o2` | **MEASURED.** Entry only, 386 lines, no change under `crates/` | `0f6a87e`, merged `4b2f2d0`, ticked `7bd235a` | **APPROVE** |
| worked examples become measured output (`opening-3`) | `lane-o3` | the executable twin: two tests in `agent_doc.rs`, `Change::ALL` in `delta.rs`, five `AGENT.md` blocks regenerated | `72ce305`, merged `f69bad6`, ticked `86f1ef9` | **APPROVE** |
| the M5 plan | orchestrator | `tasks/M5/PLAN.md`, four phases, as a PROPOSAL | `43ab562` | n/a — awaiting James |

### Gates

Run twice in full, and the difference between the two runs is itself a finding.

**At the `dd6bd23` merge — five of six.** `fmt`, `clippy`, `test`, `doc`, `deny`
green; **`clean` failed as a phantom**, because the orchestrator wrote a task
entry while the check was running. See §6, PROPOSED 5. Every test arm passed.

**At the stop, on a quiescent tree — six of six.**

```
pass  fmt   pass  clippy   pass  test   pass  doc   pass  deny   pass  clean
all gates passed
```

- `corpus`: **115 schematics, 36 library tables, 0 schematics off the pinned
  stamp, verified**
- `cargo test --features corpus` and `KICLI_TEST_KICAD_CLI=1 cargo test
  --features corpus` — pass
- **255 test-result lines, every one `0 ignored`. Zero skips**, including the
  `kicad-cli` arm.
- **netlist oracle green in all three arms** — `netlist_oracle_is_current`,
  `netlist_partition_matches_kicad`, and
  `corpus::netlist_partition_matches_kicad_corpus`. **35 of 35 holds.**

**The two runs differ in one thing: whether anyone was writing.** That is the
whole of PROPOSED 5, measured rather than argued.

---

## 2. What the rulings did to the record

**All of BLOCKED 1, BLOCKED 2, and PROPOSED 1–23 are applied.** Where each
landed:

| Ruling | Landed in |
|---|---|
| BLOCKED 1 — the gate governs every commit that reaches `main` | `ENGINEERING.md`, new section "The gate and the lane branch" |
| PROPOSED 14, 19 — sanctions by cause; sanctions carry an exit procedure | the same section, as its two clauses |
| BLOCKED 2 — the joined-net contract field | `tasks/M5/opening-1-…`; **parked, see §6** |
| PROPOSED 4 — the manual flow holds | `CLAUDE.md`, reversal trigger re-worded to govern **undisclosed** excess |
| PROPOSED 13, 15, 17, 23 (+7) | `falsification-control`, **restructured** around the four blind-instrument kinds |
| PROPOSED 11 (subsuming 3, 6) | the same skill, "Git will not hold your good state", with `--amend` as corollary |
| PROPOSED 8 + 22 | the same skill, as one rule about evidence naming a moving target |
| PROPOSED 2 | `lane-implementer.md` — the evidence rule has no brief-level exception |
| PROPOSED 5, 8, 10 | `orchestrator.md` — enumeration-derived scope, evidence carve-out, review and resume brief patterns |
| PROPOSED 21 | `tick-reviewer.md` — scratch bookkeeping never to a shared file |
| PROPOSED 1, 12, 16, 18, 20 | `tasks/M5/chore-1…5` |
| PROPOSED 9 | M4's `calibration` exit row re-worded; the sweep carried in `carried-5-calibration-sweep.md` |
| the mutation triage | `mutation-survivors.md` gains the triage table; M-7's dead method is `chore-6`; M-1 became `opening-2` |
| boundary 6 — seven retrospective areas | `consolidated-report` skill |
| boundary 7 — one file per task | `tasks/M5/`, 17 files, `tasks/M5.md` retired |
| boundary 8 — dogfood as a standing exit gate | `CLAUDE.md` |
| boundary 9 — measured examples | `ENGINEERING.md` + `tasks/M5/RULES.md` + `opening-3` as its twin |

### One reading recorded rather than made silently

The BLOCKED 2 ruling says "spec §8 amended in the same commit". **`spec/SPEC.md`
§8 is Mutation semantics and is not the output contract.** The contract is
`research/wire-routing.md` §8 — verified, not inferred, against the precedent
commit `dd4f659`, whose message says *"research/wire-routing.md §8 is amended in
this same commit"*, and against `report.rs`'s own module doc. Recorded in the
entry, because reading a ruling's citation as a slip is exactly the judgement
that must be visible.

---

## 3. Findings, attributed

### The freeze cannot be lifted from the place the procedure names — `lane-o1`

**Measured, not inferred**, and the measurement is the whole finding. The lane
removed `crates/kicli/src/route/report.rs` from **its worktree's**
`.claude/hooks/frozen-paths.txt`, and `Edit` on `report.rs` was still refused.
It then ran both copies of the identical script on identical stdin:

| script run | list read | exit |
|---|---|---|
| `bash .claude/hooks/frozen-surface.sh` (worktree copy) | worktree's, lifted | **0** — allowed |
| `bash /Users/james/code/kicli/.claude/hooks/frozen-surface.sh` (main copy) | main checkout's, intact | **2** — blocked |

`frozen-surface.sh` resolves its list as `"$(dirname "$0")/frozen-paths.txt"`,
and `.claude/settings.json` registers the hook by the **relative** command
`.claude/hooks/frozen-surface.sh`, which resolves against the session's project
root — the main checkout — never the worktree.

**The consequence is general and it is worse than the blocked task.** No lane
can execute a freeze lift; and **the lift a lane commits is invisible to the
hook that enforces it**. The version-controlled record and the enforced
mechanism read two different files. `CLAUDE.md` says "lifting a freeze IS the
ruling path" and names no owner who can walk it.

BLOCKED 1 of this session. See §6.

### Reachable, correct, untested — a third answer the record had no label for — `lane-o2`

James's ruling conditioned the action on reachability alone: *if any caller can
pass a right-to-left segment, file it as an M5 task with the measurement.*

**Reachability: YES.** `read_line` (`model/items.rs:756`) takes `from` = first
`(xy …)`, `to` = last, **in file order**; `read_wires_and_marks`
(`route/sheet.rs:218–224`) copies both verbatim, and is the only production
writer of `SheetGeometry::segments`. **14 of the 77 segments** in
`calibration.kicad_sch` arrive at `lay` reversed today, and **114 of 115 demo
schematics rewritten by `kicad-cli sch upgrade` itself** carry one. KiCad does
not normalise.

**Correctness: the guard is right.** The same `Segment` swapped end for end
builds an identical map, cell for cell.

**And the equality is not degenerate, which is the part that makes it
evidence.** With both M-1 guards replaced by `true` in a scratch copy, the same
comparison shows the maps differing in **8 of 9 cells** — the reversed segment's
obstacles moving from columns 14–22 to 22–30, mirrored about its start point —
and the fixture map going **877 → 890 occupied cells**.

**So the mutant is killable and no test kills it.** The router is not wrong; the
suite is thin. The lane refused both verdict forms the entry mandated and
recorded the refusal as PROPOSED, discharging both branches' obligations
regardless of the ruling. **The entry's own text was corrected**: its claim that
"equal maps under both orders would mean the guard is doing nothing" is
backwards — equal maps is what a *correct* guard produces.

**The orchestrator's defect, named:** the two mandated verdict forms encode a
false dichotomy. Reachability makes a mutant *killable*; it does not make the
code wrong. The ordinary outcome — reachable, correct, untested — had no label,
so a lane following the brief had to write something false. See PROPOSED 3.

**And its own instrument was falsified before it was trusted:** the lane's
fixture scanner reported **0 of 4 planted segments** on its first version. It
caught that with a presence control rather than shipping the zero as a result.

### The measured-examples mechanism found a real defect on its first application — `lane-o3`

**`AGENT.md`'s `wire draw` contract block claimed a three-segment route and
listed one of the three wire records**, and omitted the `wires added: 3
junctions added: 0` line entirely. Regenerated from a real run:

```
routed R1.1 -> R2.1   via 3 segments, 2 corners, 35.56mm
  cost 44 = length 28 + turns 12 + crossings 0 + text 0 + proximity 4
  wires added: 3   junctions added: 0
+ W 116592a7 50.80,45.72..50.80,50.80
+ W 7239e219 50.80,45.72..76.20,45.72
+ W 723e5aa0 76.20,45.72..76.20,50.80
```

An agent reading the old block and deleting "the wires the report named" would
have deleted one of three. **This is D3's class, alive and undetected, in the
document D3 was already fixed in.** The rule was adopted for exactly this and
paid on the first run.

**Five blocks regenerated with their commands recorded; three left alone with
the reason recorded** rather than hand-edited into agreement — which is the half
of the rule that is easy to skip.

### The check is not blind to a writer change, and says what it cannot see

The shared-ancestor trap was the entry's stated worry: anchoring the check to
the tool's own writer means a break in the writer moves both sides together.
**The lane established that it does not**, because `AGENT.md` is an
*independent control* — falsification rows F9–F12 break the writer alone, leave
the document alone, and **all four fail**.

What it cannot see is stated plainly in the entry: **it enforces agreement, not
provenance.** A hand-written example that happens to match what the writer
emits passes. That is an honest boundary rather than a gap, and naming it is the
behaviour the skill asks for.

**The derivation is anchored where it should be.** The mark set comes from
`Change::ALL` — a new constant in `delta.rs` with its own unit test asserting
the list is the enum and not a subset of it — and the handle/detail separator is
**measured off the writer by emitting a probe line**, not restated. The rustdoc
states the bound the compiler does not enforce.

### Three examples were invisible to the check until it looked at the walkthrough

The session walkthrough shows output as shell comments (`# + T da5aa983
"SPY"`). Stripping one leading `"# "` took the check from **17 to 20** documented
lines and **13 to 14** rebuilt. A check that had shipped without noticing would
have been the *reads nothing* kind in miniature — passing while silently
skipping three of the examples it claimed to cover.

### Two corrections to the orchestrator's own task text, measured and cited

- **`=` is not a `Change` glyph.** It is `Delta`'s unchanged tally. The entry
  said otherwise; the lane measured it.
- **`AGENT.md` line 183 is the layout view's wire summary, not a delta record.**
  So **ten** delta `W` lines stood at base, not the eleven the entry counted.

Both were the orchestrator's, both are cited in the entry under the standing
rule that task text yields to measured reality.

---

## 4. Reviewer rejections

**None.** Two reviews ran and both returned APPROVE. **Their quality is worth
recording rather than their verdicts, because both went past the standard the
definition sets.**

- **`opening-2`:** re-derived the "only production writer" claim by reading the
  code, wrote **its own** corpus scanner and landed on every headline number
  exactly across three corpora, and performed its own break-and-restore of both
  M-1 guards in a verified scratch copy — watching **its own independently
  written test** fail (`expected 9 occupied cells, got 1`) and then pass.
- **`opening-3`:** built the scenario itself with `kicli_probe`, ran the **real
  built binary** in a `git archive` scratch tree, and matched the regenerated
  block **byte for byte including all three handles** — so the headline defect
  is confirmed against the binary rather than against the entry. It reproduced
  the reads-nothing falsification, content-hash-verified the file back between
  steps, and **found one measured inaccuracy in the entry** (see §3).

Both stated plainly which claims they measured and which they took on the
entry's word. Both verdicts are recorded in their entries beside the tick, per
`CLAUDE.md`.

## 5. Dogfood

**No run this stop.** The gate is now standing from M5 (`CLAUDE.md`), and this
session ships no agent-facing command — `opening-1`, which would have changed
`wire connect`'s output, is parked.

---

## 6. PROPOSED items

**1. The frozen-surface hook is blind to the editing method this session
actually uses.** `.claude/settings.json` matches `Edit|MultiEdit|Write`. Bash is
not matched, and file changes made with `sed`, `python3` or a heredoc are never
seen by the hook. The project's **only** tool hook — the mechanism `CLAUDE.md`
calls "enforced" — is therefore bypassable by the ordinary editing route, with
no ill intent required.

*This orchestrator wrote every governing-document change this session through
Bash*, and none of those files is frozen, so nothing improper happened. That is
exactly why it is worth reporting: the gap was invisible because it never bit.

*Recommendation: accept, and choose between two honest positions.* Either extend
the matcher to `Bash` with command inspection — which is real work and will have
false negatives of its own — or **stop describing the hook as enforcement** and
call it what it is, an assistance against the accidental `Edit`. The present
state, prose claiming enforcement over a mechanism with an open door, is the
worst of the three. **Not applied** — hooks and `settings.json` change only by
ruling.

**2. `cargo` is not on `PATH` for a lane's shell, and no document says so.**
Raised verbatim by `lane-o2`: *"`cargo` is not on `PATH` for the lane's
non-login shell (`export PATH="$HOME/.cargo/bin:$PATH"` is needed on every Bash
call), which no brief or doc mentions."* The `tick-reviewer` definition has
carried this line for a milestone; `lane-implementer` and `chore-runner` have
not. Three of this session's briefs had to say it by hand.
*Recommendation: accept — one line in `lane-implementer.md` and
`chore-runner.md`, matching the one already in `tick-reviewer.md`.* **Not
applied**, agent definitions change by ruling.

**3. "Reachable, correct, untested" has no label, in the brief or in the
skill.** The orchestrator's own defect, named by `lane-o2` and confirmed by its
reviewer. The entry mandated `LIVE DEFECT` or `UNREACHABLE`; **reachability
makes a mutant killable, not the code wrong**, so the ordinary outcome had no
form and the lane had to deviate or write something false.

> **WORKFLOW NOTE, `lane-o2`, verbatim:** *"The entry's two mandated verdict
> forms encode a false dichotomy — they assume a reachable mutant implies
> incorrect code, but reachability only makes a mutant *killable*, so the
> ordinary outcome 'reachable, correct, untested' has no label and forces a lane
> to either deviate or write something false; the mutation-run skill's two
> triage classes have the same hole."*

**The second half is the important half**: `.claude/skills/mutation-run/SKILL.md`
has two triage classes, genuine and benign, and neither fits a survivor over
correct code that a test could kill. *Recommendation: accept — a third class,
and a brief pattern that offers three verdicts wherever it offers two.* **Not
applied**, skills change by ruling.

**4. The `falsification-control` restructure grew the file from 198 to 336
lines, and the doc-diet pressure is not resolved.** The restructure was the
right call and is what James ruled — six findings folded into one organisation
rather than six appends, and it now reads as one idea with four worked examples
rather than a pile. But **the diet finding wanted it shorter, and it is longer.**

The honest accounting: appending 3, 6, 7, 8, 11, 13, 15, 17, 22 and 23
separately would have been longer still, so the restructure paid. It did not
pay enough. *Recommendation for the advisor's triage: the next reduction is not
another restructure but a decision about **which worked examples earn their
place**, and that is a judgement the orchestrator should not make alone —
every one of them was paid for by an incident.* One candidate: the two T11
examples make the same point.

**5. The orchestrator has no rule against running the merged check on a live
tree, and tripped it.** The `dd6bd23` merged check failed `clean` — because the
orchestrator wrote `tasks/M5/opening-3-measured-examples.md` while the check was
running. Five of six gates green, all corpus and environment arms green, and one
phantom.

`tick-reviewer.md` carries this as an **unconditional** rule, promoted from a
conditional one precisely because *"the orchestrator writes the consolidated
report per tick, so the checkout is live whenever anyone else is working"*. The
rule names the orchestrator as the **cause** and binds only the reviewer.
*Recommendation: accept — the orchestrator's merged check runs on a quiescent
tree, which means the record commit precedes it, not the other way round.*
**Not applied**, agent definitions change by ruling.

**6. The obstacle walk's missing check: task or chore?** Recorded in full in
`tasks/M5/opening-2-…` beside the tick and as question 3 of `PLAN.md`. The
ruling's trigger fired and its premise did not. Filed as a task on the literal
words, with the reviewer's contrary reading quoted beside it. **One line either
way, and it is James's.**

**7. Dogfood defect 8 is confirmed environmental, from a second direction.** The
first run reported a `zoxide` shell-config warning on every invocation and the
triage recorded it as "not a kicli defect — the sandbox's shell". **The same
warning appeared in the orchestrator's own shell this session**, in the main
checkout, with no sandbox involved. *Recommendation: none needed — the triage
was right. Recorded because a closed finding that gets independent
corroboration is worth one line, and because the standing instruction for the
next run is "a clean shell environment" and this says that instruction is still
unmet.*

**8. Nothing in the repository reproduces the drawings the `AGENT.md` examples
were measured from.** Raised by `lane-o3`. The five regenerated blocks were
produced against drawings built by a **throwaway crate outside the repository**,
so the examples are now measured — and **not reproducible**. The next session
that touches one of those blocks cannot regenerate it without rebuilding the
scaffolding from scratch.

This is the measured-examples rule's own second half arriving with a cost
attached, on its first application. *Recommendation: accept as a chore — a
committed fixture or a probe recipe, so "regenerate it" is a command rather than
a project.* Until then the rule is satisfiable only by whoever still has the
scaffolding.

**9. The `opening-3` entry's scope and its completion check contradict each
other, and the orchestrator wrote both.** Raised verbatim by `lane-o3`:

> The entry's scope allows the check to go in "a new test file beside"
> `agent_doc.rs`, but its completion check is `cargo test --test agent_doc`,
> which would never run such a file — the two halves contradict, and a lane that
> took the offer would report green on a check that ran nothing.

**Nothing is wrong in the diff** — the lane did not take the offer. The defect is
the entry's, and it is the same family as PROPOSED 3: a brief that offers a
choice its own completion check cannot cover. *Recommendation: accept, as a line
in the orchestrator definition beside the enumeration-derived-scope rule — a
brief's completion check must be able to run everything its scope permits.*

**10. The orchestrator's shell working directory persisted into a lane
worktree, and a merge ran there.** Found by the orchestrator, in the act.
Reading `lane-o3`'s diff involved a `cd` into its worktree; the Bash tool's
working directory persists between calls, so the **next `git merge --no-ff
lane-o3`** executed inside `lane-o3` itself. It was a no-op — git refused
because the branch was checked out — and `git log --oneline -1` then printed the
lane's own head, **which reads exactly like a successful merge.**

**Nothing was damaged, and nothing was damaged for a reason that is not a rule:**
git's already-checked-out protection. Had the target been a branch not checked
out anywhere, the merge would have succeeded in the wrong tree and the main
checkout would have been silently short one merge.

*Recommendation: accept — orchestrator commands that act on the repository name
the checkout explicitly (`git -C <root>`), and never rely on the inherited
working directory. And a merge is confirmed by reading the merge commit, not by
reading `HEAD`.* It belongs in the orchestrator definition beside the
scope-verification step, which is the other place a merge is checked rather than
assumed. **Not applied** — agent definitions change by ruling.

---

## 7. BLOCKED items

### BLOCKED 1 — the freeze cannot be lifted from the place the procedure names

**Raised by `lane-o1`, parked and reported rather than worked around, which is
the rule working.** The measurement is in §3.

`CLAUDE.md` says: *"The frozen surface is enforced by a PreToolUse hook over
`.claude/hooks/frozen-paths.txt`. Lifting a freeze IS the ruling path; the
orchestrator is not exempt."* James ruled the freeze lifted for exactly the
joined-net field, with the lift and restore in one commit. **The procedure names
a step its addressee cannot take**: the hook reads the main checkout's list, so
a lane's lift changes nothing and a lane's restore proves nothing.

**Both readings, since a governing-document conflict is never resolved by
precedence:**

- *The record is the authority.* Then the committed `frozen-paths.txt` on the
  branch is what the freeze IS, the hook is a convenience that happens to read
  the wrong copy, and the fix is to make the hook derive its list from the
  edited path's own worktree root so the freeze travels with the branch.
- *The mechanism is the authority.* Then only the main checkout has a freeze,
  the lift is the orchestrator's pre-dispatch act, and every entry that says
  "remove the path from `frozen-paths.txt`" is telling a lane to perform
  theatre.

**Recommendation: the first reading — derive the list from the edited path's own
worktree root.** It is the one under which the version-controlled record and the
enforced mechanism are the same file, which is the property the freeze exists to
have. The second reading is workable and cheaper, and if it is chosen then
`CLAUDE.md`'s "the orchestrator is not exempt" needs re-wording, because under
it the orchestrator is the only party who can act at all.

**What waits on this ruling:** `opening-1` only. Its reconnaissance is banked in
its entry — the three sibling-key call sites, a degeneracy already present in
`claimed_net` (`.as_str()` cannot tell null from absent, so the null-arm
assertion should be **strengthened** rather than merely relocated), the fact
that **no test asserts the text line at all** so `AGENT.md` is its only reader,
the second `AGENT.md` block the change invalidates, and the one real design
decision (`perform()` renders internally and is shared with `wire draw`, so the
read-back must be threaded before the render on both `connect_wire` paths). The
re-dispatch is cheap.

**Nothing else waits.** The plan schedules around it.
---

## 8. Workflow retrospective

*Seven fixed areas, in order, per the `consolidated-report` skill as amended
this session. WORKFLOW NOTEs quoted verbatim under their areas.*

### 1. Score

- **Rulings applied: all of them.** BLOCKED 1, BLOCKED 2, PROPOSED 1–23, and
  boundary items 6–9. Landing map in §2.
- **Tasks:** 3 opened, **2 ticked** (`opening-2` and `opening-3`, both
  APPROVE), **1 parked** (`opening-1`, BLOCKED).
- **Lanes:** 3 dispatched, 3 base verifications performed and confirmed, **0
  fast-forwards needed, 0 scope deviations**.
- **Reviews:** 2 run, 2 APPROVE, **0 rejections**. Both found something the entry had wrong; neither found it disqualifying.
- **Commits on `main`:** 13, including 3 merges.
- **Gates:** one merged run with corpus and environment arms — five of six
  green, `clean` a phantom of the orchestrator's own making (§5, PROPOSED 5);
  corpus verified at 115 schematics, 0 off-stamp; **255 test-result lines, every
  one `0 ignored`**; netlist oracle green in both the fixture and corpus arms.
- **`tasks/M5/`:** 17 files, replacing one 346-line file.

### 2. Verification integrity

**The area with the most in it this stop, which is the right shape for a session
that spent itself on how checks are written.**

- **The measured-examples mechanism paid on its first application**, and paid in
  the document its own defect class had already been fixed in: `AGENT.md`'s
  `wire draw` block claimed a three-segment route and listed **one** wire record.
  A hand-written example is correct until the writer changes, and nothing
  notices — which was the entire argument for the rule, now with a number.
- **Three examples were nearly invisible to the check that covers them.** The
  session walkthrough shows output as shell comments, so stripping one leading
  `"# "` moved the counts from 17 to 20 documented lines and 13 to 14 rebuilt.
  Shipped as-is it would have been the *reads nothing* kind in miniature.
- **A shared-ancestor worry was tested rather than argued.** The `AGENT.md`
  check is anchored to the tool's own writer, which risks claim and control
  moving together; four falsification rows break the writer alone, leave the
  document alone, and all four fail. What it still cannot see is stated in the
  entry — **it enforces agreement, not provenance.**
- **A blind instrument was caught by its own presence control before it produced
  a result.** `lane-o2`'s fixture scanner reported **0 of 4 planted segments** on
  its first version. That is kind 1 — *reads nothing* — caught at the only point
  where catching it is free.
- **A degenerate equality was interrogated rather than accepted.** `lane-o2`'s
  central claim is that two orders build the *same* map. On its own that is the
  exact shape C8 was burned by. It is redeemed by a falsification that moves
  8 of 9 cells, and the entry states plainly that both sides descend from one
  `Segment` — the shared ancestor named rather than hidden.
- **The review re-derived rather than replayed.** It wrote its own scanner and
  its own test and landed on the entry's numbers exactly. It also said which
  claims it could *not* verify (the scanner's own falsification, whose scratch
  artefact was gone) instead of blurring them into the rest.
- **A verdict binary was refused, and the refusal was reviewed rather than
  waved through.** `lane-o2` returned neither mandated form; its reviewer ruled
  the refusal sound by name against `lane-implementer.md`'s "task text yields to
  measured reality". A lane that quotes the false sentence, cites the
  refutation, and files PROPOSED is doing the thing the tick review exists to
  make possible.
- **The one instrument this session did NOT verify is its own governing-document
  changes.** Nine documents were rewritten and nothing mechanical checks any of
  them. `ENGINEERING.md` says a rule that matters wants an executable twin, and
  of this session's rules exactly one got one (`opening-3`). That is honest to
  report and uncomfortable to read.

### 3. Record quality

- **The M5 directory is the boundary package's best return so far.** Three
  briefs this session cited an entry by *path* rather than by heading-and-range,
  and none of the three lanes had to search a 6,000-line file to find its task.
- **A ruling's own citation was wrong and was recorded rather than silently
  corrected.** "spec §8" means `research/wire-routing.md` §8; the resolution is
  in `opening-1`'s entry with the evidence (`dd4f659`, and `report.rs`'s module
  doc) rather than in the orchestrator's head.
- **Four live pointers to `tasks/M5.md` survived the migration** and were found
  by sweep, not by reading. Fixed in `23da909`. `tasks/reports/M4-phase3.md`'s
  two were deliberately left: a delivered report records the tree as it stood.
- **Three entries were corrected by the lanes that executed them**, in the
  record, beneath the claim rather than over it: `opening-2`'s "equal maps would
  mean the guard is doing nothing" is backwards; `opening-3` said `=` is a
  `Change` glyph (it is `Delta`'s unchanged tally) and counted eleven delta `W`
  lines where ten stood (line 183 is the layout view's summary). **All three
  were the orchestrator's, and all three were caught by measurement rather than
  by review.**
- **One entry contradicted itself and the lane said so rather than taking the
  offer:**

  > **WORKFLOW NOTE, `lane-o3`, verbatim:** *"The entry's scope allows the check
  > to go in 'a new test file beside' `agent_doc.rs`, but its completion check is
  > `cargo test --test agent_doc`, which would never run such a file — the two
  > halves contradict, and a lane that took the offer would report green on a
  > check that ran nothing. Separately, `cargo` is still not on `PATH` for a
  > non-login shell here and this is still in no document, a second session
  > running."*

  The first half is PROPOSED 9 and is the orchestrator's; the second half
  belongs to area 5 and is PROPOSED 2, now reported twice.

### 4. Coordination

- **Three worktrees created by hand at a commit the orchestrator chose; three
  lanes verified their base as their first action; three matched.** The manual
  flow behaved exactly as the checkpoint-2 ruling predicted, and the redundant
  check cost three commands.
- **Scope verification ran at every merge**, `git diff --stat` against the
  declared list, main checkout clean before each. **No lane wrote outside its
  scope.** The reversal trigger did not fire.
- **One sequencing decision was reversed mid-session and improved by it.**
  `opening-3` was sequenced after `opening-1` because both hold `AGENT.md`; when
  `opening-1` parked, `AGENT.md` freed and `opening-3` ran early — which puts
  the check *before* the change it must satisfy. The entry was amended rather
  than rewritten, so how the order came to be reversed is legible.
- **Lanes ran concurrently with the orchestrator's own record work**, on
  disjoint files, with no conflict — but see area 5 and PROPOSED 5: concurrency
  with the *orchestrator* is what produced the phantom gate failure.
- **One review was dispatched against a pin that went stale before it
  finished.** `lane-o3` amended `3171db0` into `72ce305` after dispatch. The
  orchestrator corrected it live and the reviewer re-verified the diff stat and
  both scratch trees' content hashes against the new head rather than accepting
  the correction — which is the definition's re-pin rule doing exactly its job.

  > **WORKFLOW NOTE, `opening-3`'s reviewer, verbatim:** *"The brief's initial
  > pin (`3171db0`) went stale mid-review when the lane amended its commit to
  > `72ce305` after dispatch but before the review's shell work finished —
  > required a live orchestrator correction and a full re-verification of the
  > diff stat and scratch-tree hashes against the new head. Separately, the
  > entry's own 'corroborated, left alone' claim about the delta-digest block
  > (`AGENT.md` ~211) is inaccurate — the reference test's line order differs
  > from the document's, not just the `Test:R`/`Device:R` token as claimed —
  > worth a follow-up correction to the entry text, though it doesn't touch the
  > diff's actual scope."*

  **The second half was acted on**: the correction is recorded beneath the claim
  in the entry, in commit `86f1ef9`.
- **The orchestrator merged from inside a lane worktree and the no-op looked
  like success.** Caught by reading the branch rather than the log line.
  PROPOSED 10, and it is the session's closest call.

### 5. Layer and tooling

**Earned their keep, named specifically:**

- `lane-implementer.md`'s *"Task text yields to measured reality, with the
  citation recorded in the entry"* — this is the line under which `lane-o2`'s
  refusal of a false binary was legitimate rather than insubordinate, and the
  reviewer cited it by name to approve.
- `tick-reviewer.md`'s *"You never run the full gate suite in the live
  checkout"* — the reviewer obeyed it and said so, and the orchestrator then
  demonstrated why it exists by breaking the equivalent rule it does not have.
- `CLAUDE.md`'s worktree-currency rule — three for three, cheaply.
- The **BLOCKED rule inside lanes**: `lane-o1` hit an impossible instruction and
  parked it with a measurement instead of finding a way round. There was a way
  round — the hook does not watch Bash — and it was not taken.

**Got in the way, or was wrong:**

- The **frozen-surface hook**, twice over: it cannot be lifted from the place
  the ruling names (BLOCKED 1), and it does not see the editing method this
  session actually used (PROPOSED 1).
- **The orchestrator's own brief for `opening-2`** encoded a false dichotomy.
  The lane and its reviewer both caught it; the brief was the defect.
- `cargo` off `PATH` in lane shells, documented in one agent definition of four
  (PROPOSED 2) — **reported by two independent lanes this session**, the second
  noting it is "a second session running". A friction reported twice and still
  undocumented is the cheapest thing on this list to fix.
- **The orchestrator's briefs, three times.** A false verdict binary
  (`opening-2`), a scope-versus-completion-check contradiction (`opening-3`),
  and two miscounted facts about `AGENT.md`. Every one was caught by the lane
  rather than by the orchestrator, which is the system working and is also the
  orchestrator being the weakest instrument in it this session.

### 6. Budget

- **Four subagents, ~478k subagent tokens.** The `opening-3` lane alone spent
  ~178k over 75 tool calls and 23 minutes — the most expensive single dispatch,
  and the only one that shipped code. The two measurement/reconnaissance
  lanes returned 386 and 159 lines of entry from ~115k tokens each — a good
  ratio, because both produced record rather than code.
- **The expensive thing was the gate suite**, run four times in full plus once
  with corpus and environment arms. Two commits exceeded a two-minute tool
  timeout and had to be re-run; the second attempt succeeded because it was
  given nine minutes rather than because anything changed.
- **What was NOT covered, stated so it does not read as coverage:** no mutation
  run (correctly — it is a milestone-close instrument, not an opening one); no
  dogfood run (nothing agent-facing shipped); **and no executable check over any
  of the nine governing documents this session rewrote.**

### 7. User signal

- **Every ruling in the opener is applied**, with the landing site for each in
  §2. Three shapings were followed as given: the four-kind restructure, 11
  subsuming the SHA collisions, and 8+22 as one rule.
- **Two rulings could not be executed as written**, and both are recorded rather
  than absorbed: BLOCKED 2's freeze procedure names a step its addressee cannot
  take (BLOCKED 1 above), and the `Obstacles::lay` triage's trigger fired while
  its premise did not (PROPOSED 6).
- **Going back to James:** one BLOCKED item, seven PROPOSED items, and the
  plan's five ruling requests — of which two block scheduling (the freeze-lift
  owner, and task-or-chore).
- **The plan is drafted and STOPPED at.** No M5 task is dispatched.

---

## 9. The doc-diet check

*Not owed until a milestone close, but the binding set moved 26 % in one session
and reporting that at the close would be reporting it late.*

| File | at `c2b4d94` | now | Δ |
|---|---|---|---|
| `CONSTITUTION.md` | 91 | 91 | — |
| `ENGINEERING.md` | 186 | 224 | +38 |
| `CLAUDE.md` | 143 | 167 | +24 |
| `falsification-control` | 198 | **336** | **+138** |
| `consolidated-report` | 31 | 85 | +54 |
| `task-entry-recording` | 69 | 69 | — |
| `mutation-run` | 146 | 146 | — |
| `oracle-check` | 29 | 29 | — |
| `orchestrator.md` | 69 | 92 | +23 |
| `lane-implementer.md` | 70 | 76 | +6 |
| `tick-reviewer.md` | 93 | 98 | +5 |
| **total** | **1,125** | **1,413** | **+288 (+26 %)** |

**The restructure paid and it did not pay enough.** Folding ten PROPOSED items
into one organisation is shorter than ten appends, and `falsification-control`
now reads as one idea with worked examples rather than a pile of rules. It is
still 138 lines longer, and `ENGINEERING.md` is explicit that the binding set's
readability outranks its completeness.

**The next reduction is a judgement about which worked examples earn their
place, and it is not the orchestrator's alone to make** — every one of them was
paid for by an incident, and deleting an example is deleting the evidence for
its rule. Raised as PROPOSED 4.

---

## 10. The stop

**The /goal condition is met.** The rulings are applied, the boundary package is
landed, the plan is drafted, and the two items that need James are BLOCKED 1 and
the plan's five ruling requests.

**M5 implementation has not begun and will not begin in this session.** No task
named in `tasks/M5/PLAN.md` is dispatched.

### What is on `main`

| | |
|---|---|
| commits | 13, `c2b4d94` → the report commit |
| lanes merged | 3 (`lane-o1` parked-and-merged for its record, `lane-o2`, `lane-o3`) |
| ticks | 2, both APPROVE |
| parked | 1, `opening-1`, BLOCKED on the freeze-lift mechanism |

### What James is asked for

1. **BLOCKED 1 — the freeze-lift mechanism.** Two readings, a recommendation,
   and `opening-1` waiting on it. §7.
2. **Ratify or re-cut `tasks/M5/PLAN.md`**, and its five ruling requests — of
   which the freeze-lift owner and task-or-chore block scheduling.
3. **Ten PROPOSED items**, §6. Four are the orchestrator's own defects
   (3, 5, 9, 10), which is the honest shape of this session.
