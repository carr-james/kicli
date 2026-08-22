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
| the joined net's contract field (`opening-1-joined-net-contract.md`) | `lane-o1b` | the field moved into the frozen route contract, `with_net` removed, `research/wire-routing.md` §8 amended, five goldens migrated; **two blind instruments found by breaking things** | entry "Tick — APPROVE"; lane `6a9e6a4`; merge `e4449ed`; freeze restored next commit | **APPROVE** |
| **the rule identity and registration seam** (`phase1-t1-rule-identity-and-registration.md`) | `lane-t1` | **the seam, and its verdict: PASS.** A `build.rs` generates the module list and registry from the rules directory — no new dependency, `Cargo.toml` untouched. Plus five new checks and a measured `fmt` gate loss with its repair | entry "Tick — APPROVE"; lane `749eaae`; merge `ee08396` | **APPROVE** |
| the seed catalogue and the ground-name list (`phase1-t5-seed-catalogue-and-ground-names.md`) | `lane-t5` | two PROPOSED answers with sources, **awaiting James** — plus two BLOCKED items and a correction to the orchestrator's brief | entry "Tick — APPROVE"; lane `088df33`; merge `0333151` | **APPROVE** |
| the obstacle walk's missing check (`chore-7-obstacle-walk-check.md`) | `lane-c7` | one test in `route_obstacles.rs`, **no source change** — the third triage class's first worked example | entry "Tick — APPROVE"; lane `94eae37`; merge `5fede1b` | **APPROVE** |

---

## 2. Findings, attributed

### A gate classifies by NAME rather than by content, and made a lane triplicate 70 lines — `lane-t1`

**WORKFLOW NOTE, `lane-t1`, verbatim:**

> *"`probe_harness_has_one_home` refuses any test file containing the literal `mod support;`, which forbids a shared test-helper module by name rather than by content — I followed the convention and triplicated ~70 lines of drawing builder across three test files rather than rename around a textual gate; if shared test helpers are wanted, that gate needs re-scoping by ruling, not by evasion. Second: the brief's scope list omitted `crates/kicli/build.rs` while the task entry's own mechanism table named a build script as a candidate, so the list and the goal state disagreed from the start — a brief that derives scope from an enumeration should say which wins, and this one did, which is the only reason it cost nothing."*

**The first half is this session's sharpest layer finding, because the project
already wrote down the lesson and then shipped a gate that violates it.**

`tasks/M5/carried-4-handle-lint.md` is three rejections' worth of evidence about
exactly this failure. Its own rejection table, rejection 1:

> *the evasion: a parameter named `id`; a method on a type named `Ident` — what
> it was blind to: **it classified by name, against a closed word list.***

`probe_harness_has_one_home` classifies by name against a closed word list of
one: the literal string `mod support;`. **A lane that wanted a shared helper
could satisfy the gate by calling it `mod helpers;`** — the gate's entire force
is over authors who follow the convention it names.

**And note what the lane did with that.** It could have renamed and passed. It
**triplicated ~70 lines instead, and reported the gate**. That is the behaviour
the project wants and it should not be punished by silence: *"a textual gate
needs re-scoping by ruling, not by evasion."*

**The cost is real and is now in the tree**: ~140 duplicated lines of drawing
builder across three test files, introduced not because duplication was cheaper
than the wrong abstraction — `ENGINEERING.md`'s actual rule — but because a gate
forbade the right one by spelling.

*Filed as PROPOSED 9.* And the irony is the argument: **`carried-4` was cut from
M5 this morning on the grounds that "its lesson is available without it".** This
is the lesson arriving, in a live gate, on the same day — which is evidence for
the ruling rather than against it, but only if the lesson is actually applied
somewhere rather than merely available.

### `inventory` and `linkme` were rejected on a measurement, not a preference — `lane-t1`

Worth recording because the brief told the lane to **stop and ask** before
adding a dependency, and it did not need to:

> **a distributed slice does not remove the per-rule `mod` line.**

So `inventory`/`linkme` would have bought a Constitution §9 licence question and
a new dependency **for nothing** — the `mod` line, which is the entire thing the
mechanical check counts, survives them. The build script removes it; the crates
do not. The lane did not stop to ask because the answer had become a fact rather
than a judgement, and that is the right reason not to escalate.

### Two blind instruments in the existing suite, found by breaking things — `lane-o1b`

**Pending its tick review at the time of writing.** Both are claims about checks
that were already in the repository and already green.

**1. `wire connect --auto-labels` had no behavioural check at all.** Break B8 —
deleting the assignment in `perform` — left **every check in the repository
green**. The lane added `a_performed_proposal_reports_the_net_its_labels_made`
in `edit_wire_connect.rs`, which is now the only thing that catches it.

That arm is not obscure: it is the path where kicli invents label names and
reports the net they made. It has been shipping unmeasured.

**2. `the_status_word_starts_the_first_line_of_every_form` was asserting
something already false, and could not see it.** The joined line was **prepended
by the command layer, above the renderer the test drives** — so the test's claim
about "every form" was true of everything it could reach and false of `wire
connect` output as an agent actually receives it. Now stated and measured by
`only_the_joined_net_may_come_before_the_status_word`.

**This is the four-kinds-of-blindness idea in the `falsification-control` skill,
arriving as a *layer* mismatch**: the instrument was pointed one level below the
thing it named. A test that drives a renderer cannot make a claim about a
command's output, and nothing in its name said so.

### The degenerate-equality trap the entry predicted in advance actually fired — `lane-o1b`

**This is the falsification discipline working exactly as designed, and it is
worth recording as such**, because most entries in this report are cases where
the discipline caught something *after* the fact.

The `opening-1` entry, written **before the work**, warned:

> *"A check asserting that the reported net **equals** the extractor's net is a
> **degenerate-equality** candidate: ask what else would make the two sides
> equal. If both are computed by the same call on the same seam, a break moves
> them together and the check cannot see it."*

**It fired.** Under break B3 — replacing the net with a constant `"SIG_A"` —
`a_route_joins_the_two_pins_it_names` stayed **green**, because the constant is
that fixture's own answer.

The lane's defence is that the equality is sound anyway, *because three literals
on three other drawings stand beside it*, and it classifies the result as the
skill's case 2 rather than case 1. **That defence is the load-bearing claim of
the whole task and the tick reviewer was briefed to attack it specifically** —
if those three checks share a fixture family or a generator, the suite is blind
to a constant and the defence is a rationalisation.

**Either way the prediction paid.** An entry that names the trap in advance turns
a review from a search into a test.

### The rule catalogue's provenance is much thinner than the catalogue looks — `lane-t5`

**Pending its tick review at the time of writing.** The measurements, as the
lane reports them, over all 28 rules in `spec/SPEC.md` §11.4:

| | Count |
|---|---|
| rules resting on **nothing published** | **4** |
| rules carrying a citation **the source does not support** | **6** |
| — of those, **inverted**: the cited source recommends what the rule penalises | **2** (`KI-DNP-001`, `KI-SYM-001`) |
| rules **gaining** a published source they never had | 10 |
| rules with **no source support for their TIER** | **24 of 28** |

**The two inverted citations are the finding.** The sharpest, quoted by the lane
from Lathrop: *"Some parts are commonly placed in different orientations,
horizontal and vertical in the case of resistors."* `KI-SYM-001` cites him for
penalising exactly that.

**And `research/schematic-lint-rule-catalogue.md` has never existed on any
branch in any commit** — the only history hit is this task's own filename. So
the IDs and tiers are wholly this project's invention, which Q1 suspected and
nobody had checked.

**The tier column is the one that should worry Phase 3**, not the source column.
A rule with no published source is a rule this project chose to have, which is
legitimate and merely undeclared. **A tier with no support is a claim about
whether a drawing is *shippable*** — Tier 1 blocks a build — and 24 of 28 are
asserted rather than argued. `RULES.md`'s north star is the sentence they should
be argued from, and nothing currently does.

*Lane's recommendation: Phase 2 may start without Q1; the answer is owed before
Phase 3, where every unsourced rule and all 22 invented weights live.*

### The ground-name list is not incomplete, it is inverted — and this corrects the orchestrator's own brief

**`lane-t5`, measured from KiCad 10.0.5's library and confirmed by rendering
with `kicad-cli`.** Not recalled — which was the brief's binding condition and
the one most likely to be quietly broken.

**The measurement:** every stock negative supply, **plus `VSS`, `VSSA` and
`VEE`, is drawn pointing UP — identically to `+5V`.** KiCad distinguishes
negative from positive **by fill, not by direction.** Only the twelve
`GND*`/`Earth*` symbols point down, and the current list recognises **three** of
them.

**The consequence reorders the whole question, and the orchestrator got it
backwards.** The T5 brief and entry both framed the stakes as: *"a name absent
from the list is a power symbol whose direction is never checked."*

> **WORKFLOW NOTE, `lane-t5`, verbatim:** *"The brief and entry both framed Q5's
> stakes as "a name absent from the list is a power symbol whose direction is
> never checked" — but the specified classifier defines positive as the
> *complement* of the ground set, so an absent name is checked *backwards*,
> producing a false finding rather than a silent skip; the brief's own
> north-star reasoning therefore pointed at the wrong half of the problem.
> Separately, the brief said to check "KiCad's demos" for a legitimate
> leading-`-` net, but the macOS KiCad 10.0.5 package ships no `demos/`
> directory at all — a brief naming a corpus should name one the lane can
> confirm exists, or say what to substitute."*

**Both halves accepted; the first is the important one and the defect is the
orchestrator's.** The brief reasoned from the north star to "a missing name is
an unchecked symbol", and **because positive is defined as the complement of the
ground set, a missing name is an INVERTED check** — a guaranteed false finding
on a correct drawing. That is the north star's *expensive* error, not its cheap
one, and the brief's own reasoning walked past it. **Completing the ground list
is therefore the highest-value change in Q5 rather than a tidiness item**: nine
additions plus a case ruling on `Earth`.

**The second half is a plain brief defect and is corrected for future briefs**:
the brief said "KiCad's demos" without a path. macOS KiCad 10.0.5 ships no
`demos/` directory; **the corpus this repository actually uses lives at
`target/corpus/demos`, fetched by `cargo xtask corpus`** — which the
orchestrator knew and did not write down. *A brief naming a corpus names a path
the lane can confirm exists.*

### The `-` prefix rule over-catches zero times, and the falsifying case exists anyway — `lane-t5`

Measured twice: **0 of 18** stock leading-`-` symbols are non-supplies; **0 of
93** template power `Value`s start with `-`.

**But the falsifying case is in KiCad's own shipped content**:
`API_Series-500.kicad_sch` carries net labels `-IN+4`, `-IN-2`, `-OUT`. It is
out of reach **only because the rule reads a power symbol's `Value`** — so, as
the lane puts it, *the precondition is the thing to test, not the list.*

**That is the right shape of answer and it is worth naming as a pattern**: "safe
given precondition P" is a different claim from "safe", and it stays true only
while P does. The lane leans toward **the shorter rule and the longer list**,
which trades an unbounded risk for a bounded one.

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

**9. `probe_harness_has_one_home` classifies by name, not by content, and it
cost 140 duplicated lines this stop.** The gate refuses any test file containing
the literal `mod support;`. It is evaded by renaming and binding only on authors
who follow its own convention — which is precisely the failure
`carried-4-handle-lint.md` documents across three rejections (*"it classified by
name, against a closed word list"*).

`lane-t1` followed the convention rather than evading it, triplicating ~70 lines
of drawing builder across three test files, and reported the gate instead of
routing around it.

*Recommendation: accept as a ruling item, and note that the lane's choice —
comply and report, rather than rename and pass — is the behaviour to reinforce.*
Two directions are available and they are genuinely different: **re-scope the
gate to classify by what a file does rather than by what it spells**, or
**declare that shared test helpers are forbidden on purpose** and accept the
duplication as the price, saying so in the gate's own rustdoc. The present
state — a rule whose stated intent is defeated by a rename — is the worst of the
three, which is the same shape as PROPOSED 1's hook.

**8. A brief that lists the guards says where there are none.** `lane-o1b`'s
brief named `command_surface.rs:403` as the guard that would catch a mistake on
one arm, and was silent about the `--auto-labels` arm having no behavioural
check at all — while the change put a live decision on that arm. The lane found
it by breaking the code (B8) and watching the whole repository stay green.
*Recommendation: accept, as a line in `orchestrator.md` beside the
completion-check rule — when a brief enumerates the checks that protect a
change, it names the parts of the change nothing protects.* **Not applied.**

**7. `.claude/skills/falsification-control/SKILL.md` should say
`cargo test --no-fail-fast`.** Full measurement in Verification integrity,
above. Cargo stops after the first failing target, so a break's caught-by list
silently truncates — B1 read as 2 checks instead of 15. The skill mandates
recording which assertion caught a break and never says how to run the suite to
see all of them. The error is conservative, which is why it has never failed
loudly. *Recommendation: accept — one line in the skill's Procedure, step 3.*
**Not applied**, skills change by ruling.

**6. A tick reviewer's scratchpad is not private, and it contained the
implementer's own downloaded sources.** Full measurement in Verification
integrity, above. The reviewer of T5 was given a scratchpad already holding
`lathrop.txt` and `hackaday.html` — the files the T5 *implementer* had fetched
while writing the entry under review.

**The danger is precise**: a reviewer verifying "is this quote really in
Lathrop?" by opening a `lathrop.txt` it did not download is checking the entry
against the entry's own working copy. It would look identical to a real
verification and would confirm anything the implementer had already convinced
itself of. **The whole point of a fresh-context reviewer is defeated by a shared
filesystem.**

*Recommendation: accept, as a line in `tick-reviewer.md`* — a reviewer treats
its scratchpad as **contaminated**, fetches every external source itself under
filenames it chooses, and never reads a file it did not create. It cost this
reviewer nothing to do; it noticed unprompted, and the next one may not.

Worth pairing with a second, cheaper measure: **the orchestrator gives each
subagent a scratchpad path it creates fresh per dispatch**, which is the
orchestrator's own instrument and needs no ruling. Applied going forward.

**Not applied** to `tick-reviewer.md` — agent definitions change by ruling.

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

### BLOCKED 1 — the spec and the research doc carry different ground lists, and the spec declares the research doc canonical

**Raised by `lane-t5`, at the moment of the claim. Correctly NOT resolved by
precedence**, per `CLAUDE.md`: *"When two governing documents conflict, do not
resolve by precedence — mark the item BLOCKED with both readings and ask."*

**The conflict:**

- `spec/SPEC.md` §11.4 states the ground/negative-supply default list.
- `research/style-rules.md` §4 states a **different** one — it carries `VEE`, a
  `^-?V?SS$` regex, and a redundant literal `-12V`.
- **§11.4 itself declares §4 the canonical catalogue.**

So the spec says "the other document is canonical" and then disagrees with it,
which means **there is no reading under which both are satisfied** — the defect
is not that one is stale, but that the spec's own pointer contradicts the spec's
own content. `KI-FLOW-001` and `KI-FLOW-002` stand on whichever wins.

**Why this is genuinely blocked rather than a PROPOSED item**: picking either
list is a value-level scoring call — it decides which drawings the linter calls
wrong — and `RULES.md`'s north star rule says such calls are *parked against
that sentence, not guessed*. It is also not cheap to reverse: once Phase 3
writes `KI-FLOW-001` against a list, changing the list changes findings on every
sheet already scored.

**Options:**

1. **Ratify the T5 proposal, which supersedes both.** The lane's measured list
   (twelve `GND*`/`Earth*` symbols, nine additions to the current three, plus a
   case ruling on `Earth`) is derived from KiCad 10.0.5's actual library rather
   than from either document, and adopting it makes both existing lists
   obsolete rather than making one win.
2. Declare §11.4's list canonical and correct §4 to match.
3. Declare §4 canonical, per §11.4's own pointer, and correct §11.4 to match.

**Recommendation: option 1.** It is the only one that resolves the conflict with
a *measurement* rather than a choice between two unsourced lists, and — as the
lane notes — **James's ratification of the proposed list clears this at no extra
cost**, because he is being asked to rule on that list anyway. Options 2 and 3
each pick a winner between two documents neither of which was measured against
KiCad.

**Cost of leaving it open:** Phase 3's flow-and-direction rules cannot be written.
Phase 2 is unaffected.

### BLOCKED 2 — Q2's closing condition has been met, and the ruling that closed it named that condition

**Raised by `lane-t5` as an incidental find; escalated by the orchestrator,
because it reopens a standing ruling and that is not the orchestrator's to do.**

James's standing round-6 ruling closed Q2: **the Greenberg video is skipped; the
text sources govern.** It was re-confirmed at this session's plan review, and
`lane-t5` complied — **the video was not consulted.**

But `research/style-rules.md` §8's Q2 states the condition in full:

> *"This catalogue used the published summaries […] not the video. **If his
> checklist is published in a citable form, it should be the primary source for
> the KI-DOC-\* family.** Want me to work through the video?"*

**`lane-t5` found the checklist published as citable text** —
`docs.google.com/document/d/1gCPILcrdGZJjRzIDSL-b3ezVReeK5S-7raeub1RohyE/`,
version-dated 2026-02-15.

**So the ruling's premise has changed, and in the direction the ruling
anticipated.** The video is still skipped and nothing about that is in question;
the question is whether `KI-DOC-001…004` should now be **rebuilt from a primary
text source** that did not exist in citable form when the family was written
from summaries.

**This is the same shape as the obstacle-walk ruling James reversed this
morning** — *"task text yields to measured reality, rulings included"* — which
is why it is filed rather than absorbed: the orchestrator does not reverse
James, and a ruling whose stated condition has been met is exactly the case that
goes back to him.

**Options:**

1. **Rebuild `KI-DOC-001…004` from the primary text**, as a Phase 3 task with
   its own entry. The four documentation rules are among the 28 whose provenance
   T5 just measured, so this would also close part of Q1.
2. **Leave the family on summaries** and record that the primary source was
   found and deliberately not used, with the reason.
3. Defer to the Phase 3 lane that writes the family, as a PROPOSED item.

**Recommendation: option 1, scoped small.** The published-summaries route is
what produced two of the six unsupported citations T5 found, so the family's
current sourcing is measurably the weakest in the catalogue. **But note the
honest counter**: a Google Doc is not an archival citation, it can change under
us, and it is version-dated rather than immutable — so option 1 should carry a
retrieval snapshot, not a bare URL.

**Cost of leaving it open:** none until Phase 3. The four rules are not in
Phase 2.

---

## 7. Workflow retrospective

### 1. Score

**Ticked:** 4 — chore 7, T5, T1, opening-1. **All four APPROVE first time. No
rejections, and therefore no escalations.**

**And all four reviewers re-derived rather than re-read**, which is the number
worth carrying out of this stop. Three took `git archive` into a `mktemp -d` and
verified fidelity by file count before reading anything; one re-parsed KiCad's
own library rather than checking the entry's arithmetic; one re-fetched every
external source rather than trusting a scratchpad that turned out to contain the
implementer's copies. **A tick review that only reads the entry could not have
produced any of the confirmations in this report.**

**The freeze cycle completed**: lift `d4c0eb8` → merge `e4449ed` → restore
`a8f2057`. Three commits wide, which is what the mechanism costs now the lift
and the change belong to different actors in different trees.

**Phase 0 is closed.** All three `opening-*` tasks are done and ticked.

**Merged:** 4 lanes — `lane-c7` → `5fede1b`, `lane-t5` → `0333151`,
`lane-t1` → `ee08396`, `lane-o1b` → `e4449ed`. Scope verified before the merge
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

**At the `lane-t1` and `lane-o1b` merges (`ee08396`, `e4449ed`)**, on a
quiescent tree after record commit `a8f2057`: `cargo xtask check` **6 of 6**;
corpus arm **77 binaries, 569 passed, 0 failed, 1 ignored**.

**The one ignored test is legitimate and says so itself**:
`the_report_a_child_process_prints`, ignored with the reason *"run by the
process-boundary arm, in a child process"* — it is the helper T1's determinism
check spawns, not a check that was skipped. Named here because this report
elsewhere argues that a skipped check must be distinguishable from a passing
one, and this is what doing it properly looks like: **the ignore reason states
who runs it instead.**

The suite grew **71 → 77 binaries and 535 → 569 tests** across the two merges. That pair is the session's real
code-weight — the seam plus five new lint checks, and the frozen contract's new
field with five migrated goldens.

**At the `lane-t5` merge (`0333151`)**, on a quiescent tree after record commit
`a1d31ef`: `cargo xtask check` **6 of 6**; corpus arm **71 binaries, 535 passed,
0 failed, 0 ignored**. That merge changed one markdown file and nothing else —
the corpus arm was run anyway, because `CLAUDE.md` says *at every lane merge*
and *never skipped*, and a rule with a "when it obviously matters" exemption is
a rule the orchestrator decides case by case. Cost: ~16 minutes of background
wall-clock against zero risk. Recorded so the judgement is visible rather than
assumed.

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

### A falsification table built with plain `cargo test` under-counts, and the skill does not say so

**WORKFLOW NOTE, `lane-o1b`, verbatim:**

> *"The brief's completion check says `cargo test`, but a falsification table needs `cargo test --no-fail-fast` — cargo stops after the first failing target, so a break's caught-by list silently under-counts (B1 read as 2 checks instead of 15 on the first attempt). Second: the brief named `command_surface.rs:403` as a guard to confirm, but did not say that `wire connect --auto-labels` has no behavioural check whatsoever — the migration puts a live decision on that arm, and a brief that lists the guards should say where there are none."*

**Both halves accepted, and the first is the more valuable finding of the two.**

**`cargo test` stops after the first failing target.** So when a lane breaks
something and records *which checks caught it*, the list it can see is truncated
at the first failing binary — and the truncation is silent. **B1 read as 2
checks instead of 15.**

**This is not a brief defect, it is a skill defect.**
`.claude/skills/falsification-control/SKILL.md` tells every lane in this project
to record "WHAT was broken and WHICH assertion caught it", and the caught-by
list is the evidence a reviewer checks. **The skill never says how to run the
suite to get a complete one.** Every falsification table in this project's
history was potentially built through this instrument, and an under-counted
caught-by list is *conservative* — it under-claims coverage rather than
over-claiming it — which is why nothing ever failed because of it and why nobody
noticed.

**It still matters**, because the table is also how a reviewer judges whether a
check is the *only* thing watching a behaviour. "Caught by 2 checks" and "caught
by 15" support very different conclusions about what is safe to change.

*Filed as PROPOSED 7 — one line in the `falsification-control` skill's
Procedure.* **Not applied**, skills change by ruling.

**The second half is the orchestrator's own defect and generalises**: the brief
listed the guards that would catch a mistake on the `wire draw` arm, and said
nothing about the `--auto-labels` arm having **no** guard at all — while the
change put a live decision on exactly that arm. *A brief that lists the guards
says where there are none.* That is the same family as the two-verdict brief and
the scope-versus-completion-check defect: **the brief's silence read as
coverage.** Applied to future briefs immediately; filed as PROPOSED 8 for the
orchestrator definition.

### A reviewer's "private" scratchpad already held the implementer's artefacts

**WORKFLOW NOTE, T5's tick reviewer, verbatim:**

> *"The scratchpad directory handed to me (`.../3d3843d1-.../scratchpad`) was **not actually empty/private** — it already contained files (`hackaday.html`, `poly.html/.txt`, `lathrop.txt`, `extract.py`, and several subdirectories) I did not create, apparently left by earlier work under the same session ID. This is exactly the shared-scratchpad collision the review protocol warns about; I did not rely on any pre-existing file as evidence and re-fetched everything myself under filenames I chose, but the harness's claim that this directory is "session-specific... and can generally be used without permission prompts" did not hold in practice and should not be trusted at face value on future reviews."*

**This is a verification-integrity finding, not a tidiness one, and it is the
most serious thing in this area.** Look at the filenames: `hackaday.html`,
`lathrop.txt` — **those are the T5 implementer's own downloaded sources.** The
reviewer whose job is to check T5's claims against the world was handed a
directory pre-loaded with the implementer's copy of that world.

**A reviewer that had read `lathrop.txt` instead of fetching Lathrop would have
"independently confirmed" the entry against the entry's own working files.**
That is a shared-ancestor failure — the exact class the
`falsification-control` skill exists for — arriving through the *harness* rather
than through a test's construction, where no rule in this project was looking
for it.

**The control held, and it held because the reviewer was suspicious rather than
because anything stopped it.** It re-fetched every source itself under filenames
it chose. Nothing in the tick-review brief, the `tick-reviewer` definition, or
the skills told it to; it noticed the directory was dirty and drew the right
conclusion unprompted.

*Filed as PROPOSED 6.* **This is the strongest candidate in this session for a
rule rather than a note**, because the failure is invisible when it happens: a
review that quotes the implementer's cached copy looks exactly like a review
that fetched the source.

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

**A corpus run was reported FAILED by the orchestrator's own shell, and it had
passed.** The command ended `…; grep -cE 'test result: FAILED' "$f"`, and
**`grep -c` exits 1 when it finds zero matches** — so the pipeline's exit code
reported the *absence* of failures as a failure. `cargo`'s own `exit=0` was
sitting in the same output file the whole time.

**Nothing was wrong and thirty seconds were spent establishing that.** It is
recorded because of what it is an instance of: this stop's recurring theme is
instruments that answer a different question than the one asked, and the
orchestrator built one, in the act of checking for exactly that class of
problem. *A "no failures found" check whose success path exits non-zero is the
same defect as a summary that cannot say "did not run".*

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
