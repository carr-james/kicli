# M5 — Scoring: the plan

> **RATIFIED, with one amendment.** Provenance: James's ratification and advisor
> rulings, M5 plan review. The four-phase shape stands; **Phase 3 opens at TWO
> lanes**, widening mid-phase only on evidence of frictionless first merges (see
> Phase 3 below). All five questions at the foot of this file are ruled, each
> recorded at its question.
>
> Written by the orchestrator at the M5 opening. Provenance for every carried
> item is in the file named beside it; provenance for the plan's *shape* is
> `spec/SPEC.md` §11 and §19, and `research/style-rules.md` §4 and §6, which
> §11.4 makes the canonical catalogue.
>
> **The milestone's north star is `RULES.md`'s, and it governs every value-level
> call in this plan**: *"The tool must validate the important aspects of quality
> schematics. It must never reward a schematic that is impossible to read and
> understand."*

## Goal of M5

**kicli says how good a drawing is, deterministically, and says why.**
`sch score` runs the lint engine, consumes ERC where ERC owns the check, and
reports findings in `spec/SPEC.md` §11.3's format with a score per §11.5.
`sch score --gate` fails on Tier 1 independently of the score. A schematic can
score 96 and still fail the gate, and that must be visible in the output.

M4 built a router that draws a wire the way a person would. M5 is the milestone
that can say whether the drawing is any good — which is also the first milestone
whose output is a **judgement** rather than a fact, and that changes what the
checks have to do. A gate that cannot fail on a wrong weight is the M4
calibration lesson, and this milestone is made of such gates.

## What M5 is NOT

- **Not rendering.** `sch watch`, SVG, overlays are M6.
- **Not a rule invented here.** `research/style-rules.md` §4 is the catalogue;
  §11.4 retires the pre-research draft with no reconciliation work owed.
  A rule this milestone wants and the catalogue lacks is a BLOCKED item.
- **Not an LLM in the loop, ever.** Constitution §4: geometry and heuristics
  only, no network, no vision model. Rendering exists so an agent can look;
  what it sees never feeds the score.
- **Not a re-implementation of ERC.** §11.1: KiCad 10's ERC implements 47
  checks and **kicli's lint engine implements none of them**, with exactly two
  deliberate exceptions (`four_way_junction` → `KI-JCT-001`,
  `single_global_label` → `KI-LBL-003`) because KiCad's default severity for
  both is `IGNORE` and an untouched project would silently pass.

---

## The design question that decides whether this milestone can parallelise

**Settle it in Phase 1 or the lane table below is fiction.**

`ENGINEERING.md` requires that *adding a new lint rule must not require editing
unrelated modules*. There are **six Tier 1 rules and twenty-two Tier 2 rules**.
If rules register themselves by a mechanism that touches one shared list, that
list is a merge hotspot that every lane in Phases 2 and 3 must queue on, and the
milestone serialises whatever the table says.

So the first task is not a rule. It is the **rule identity and registration
seam**, and its completion check is mechanical: *adding a rule touches exactly
one new file and no existing one.* If that check cannot be made to pass, the
lane table shrinks to two lanes and the plan is re-cut — better to learn that in
Phase 1 than at the third merge conflict.

---

## Phase 0 — the opening (RULED, in progress)

| Task | State |
|---|---|
| the joined net's contract field (`opening-1-joined-net-contract.md`) | **UNBLOCKED and scheduled.** The freeze lift is the **orchestrator's** step in the main checkout — question 2, ruled. The entry's procedure is re-worded to name that owner; the banked reconnaissance stands. |
| the obstacle walk's direction (`opening-2-obstacle-walk-direction.md`) | **MEASURED.** Reachable, guard correct, check missing. See below. |
| worked examples become measured output (`opening-3-measured-examples.md`) | **DONE**, ticked APPROVE, merged `f69bad6`. It found a live defect on its first application. |

### What the obstacle measurement returned, and the one place it needs James

The ruling read: *if any caller can pass a right-to-left segment, file it as an
M5 task with the measurement, **not a chore**.*

**Reachability is YES**, measured: `read_line` takes `from` = first `(xy …)`,
`to` = last, in file order; `read_wires_and_marks` copies both verbatim; **14 of
77 segments** in the calibration fixture arrive reversed, and 114 of 115 demo
schematics *rewritten by `kicad-cli` itself* carry one — KiCad does not
normalise.

**But the guard is correct**: the same segment swapped end for end builds an
identical map, cell for cell — and that equality is not degenerate, because with
both guards forced to `true` the maps differ in 8 of 9 cells and the fixture map
goes 877 → 890 occupied cells.

**So the ruling's trigger fired and its premise did not.** The condition was
reachability, and reachability holds; the reason was a possible live defect, and
there is none. Followed literally, it is **an M5 task**; read for its purpose, it
is a one-test chore. **Filed as a task, per the ruling's own words** — the
orchestrator does not reverse James — with this discrepancy recorded rather than
absorbed. It is one line of the plan to move it either way.

**RULED: it is a CHORE.** Provenance: James's ratification and advisor rulings,
M5 plan review, question 3. Verbatim: *"the measurement dissolved the
possible-defect proxy my ruling's words conditioned on; task text yields to
measured reality, rulings included."* The line is moved, and the entry is
`chore-7-obstacle-walk-check.md`.

**The general lesson was ruled into the layer at the same time**: the
`mutation-run` skill gains a **third triage class** — *reachable but correct: the
check is the deliverable* — because the two it had (genuine coverage gap, benign
survivor) could not name this survivor, which is the ordinary case rather than an
exotic one. That is `lane-o2`'s finding (PROPOSED 3), promoted.

### What the measured-examples check returned

**The rule paid on the run that introduced it.** `AGENT.md`'s `wire draw` block
claimed a **three-segment** route and listed **one** of the three wire records,
omitting `wires added: 3   junctions added: 0` entirely. An agent following the
document's own instruction to delete "the wires the report named" would have
deleted one of three and left two behind.

That is dogfood defect 3's class — a hand-written example that was correct until
the writer changed — **alive and undetected in the document D3 was already fixed
in**. Confirmed by the reviewer against the built binary, byte for byte.

**Two consequences for this plan.**

- **A chore is owed** (PROPOSED 8 of the session report): every regenerated block
  stands on a drawing built by a throwaway crate **outside the repository**, so
  the examples are measured and **not reproducible**. Until a fixture or probe
  recipe is committed, "regenerate it" is a project rather than a command. **M5
  will touch `AGENT.md` heavily** — `sch score` is a new command surface — so
  this lands before Phase 2, not after.
- **The check enforces agreement, not provenance.** A hand-written example that
  happens to match what the writer emits still passes. That boundary is stated
  in the check's own rustdoc and is the honest limit of a control anchored to
  the thing it controls.

---

## Phase 1 — the engine's spine (sequential, orchestrator, no parallel lanes)

Nothing here is a rule; everything here is what every rule stands on. Sequential
because each task fixes a shape the next is written against, and because getting
these wrong is the expensive kind of wrong.

| # | Task | Completion check measures |
|---|---|---|
| **T1** | **The finding, the rule identity, and the registration seam.** `KI-…` code, tier, severity, penalty, position, objects, `fix` string, per §11.3. | Adding a rule touches **one new file and no existing one**. Findings sort by `(rule, sheet, x, y, uuid)`. Re-scoring an unchanged file is **bit-identical**. |
| **T2** | **ERC consumption, and the 100× canary.** `kicad` owns the invocation (`ENGINEERING.md` Structure); §14.2 requires the canary because KiCad builds the ERC JSON units provider with the PCB scale. §14.3: severities are read-only. | The canary **fails** when fed an ERC report at the wrong scale — the only check here that matters is the one that catches KiCad changing its mind. Absence of `kicad-cli` is a structured error and exit 6, not a panic. |
| **T3** | **The score formula and the normalisers.** §11.5: `raw_penalty = Σ w_r·n_r·norm_r`, `score = round(100·exp(−raw/K))`, `K = 25`; `per_object`, `per_wire`, `per_sheet`. Project score is the symbol-count-weighted mean. | **No floating point outside the final `exp`** — an executable twin like M4's `no floating point` gate. A sheet with 4 symbols and one crossing scores worse than one with 200 symbols and one crossing. |
| **T4** | **Tier 1 does not reduce the score.** It sets `"gate": "fail"` independently. | A file that scores 96 and fails the gate exists as a fixture, and both facts are visible in one output. |

| **T5** | **The seed catalogue and the ground-name list, researched.** Question 4, ruled: Q1 and Q5 of `research/style-rules.md` §8 become **one bounded research task** (`phase1-t5-seed-catalogue-and-ground-names.md`). | Its deliverable is **PROPOSED answers with sources**, awaiting James's ratification at the checkpoint — *the linter's notion of "good" is his to sign*. It runs alongside T1–T4 where it does not contend: it writes no source file. |
| **carried-2** | **Where a pin is, read-only** (`carried-2-pin-location.md`). Dogfood D2, the largest of the first run's nine defects. | Scheduled after the seam merges. Disjoint from `lint/**`, so it is the one Phase 1 item that may run beside a spine task. |

**Dependency:** everything. Phases 2 and 3 do not start until Phase 1 merges.

**And Phase 2 is gated on more than the merge.** Ruling, M5 plan review: the
**seam verdict** (T1's mechanical check, PASS or FAIL) and the **T5 research
proposals** both go to James at the checkpoint, and **no Phase 2 lane is
dispatched until he ratifies both.** A FAIL on T1 re-cuts the lane table and this
plan, and that is a finding rather than a setback — see the design question
above.

---

## Phase 2 — Tier 1, the blocking rules (lanes)

Six rules, one of them delegated. These decide whether `--gate` passes, so they
are the ones an agent's build breaks on.

| Rule | What it detects | Notes |
|---|---|---|
| `KI-GRID-001` | connectable geometry off grid | Constitution §7. **Field and graphic text are exempt** — KiCad's own autoplacement lands fields on arbitrary IU, so a blanket rule fails KiCad's own output. ERC's `endpoint_off_grid` covers wire endpoints only, as a warning; report it when present, do not double-count. |
| `KI-OVL-001` | symbol bodies overlap | exact box intersection ≥ 1 IU, power symbols included; `--allow` list rather than a soft rule |
| `KI-WIRE-001` | wire crosses a symbol body | |
| `KI-TXT-001` | overlapping text | needs the **full box** (∪ visible field boxes), not the body box — §8's two-box model |
| `KI-CONN-001` | a pin touches a wire's interior with no junction | looks connected and is not; `research/notes/pin-on-wire-interior.md` |
| `KI-HIER-001` | delegated to ERC entirely | its task is the delegation and the attribution, not a detector |

**The carried item that lands here:** the reader-strictness defect
(`carried-3-reader-strictness.md`). kicli reads a bare `input` where
`(shape input)` belongs and reports **14 nets where KiCad reads 36 and 32**,
with **no warning and a byte-identical view**. That is a Constitution §1 and §3
matter and it belongs to whichever Tier 1 rule owns the reader's strictness —
most likely a refusal rather than a finding, which is a decision, not a patch.

---

## Phase 3 — Tier 2, the scored rules (lanes)

Twenty-two rules. Grouped by the *definitions* they share rather than by
alphabet, because that is what decides lane boundaries.

### AMENDMENT — Phase 3 opens at TWO lanes

**Provenance: James's ratification and advisor rulings, M5 plan review.** The
plan asked (question 1) whether Phase 3's six lanes were worth the coordination
against two larger ones. **Ruled: open at two, widen mid-phase only on evidence
of frictionless first merges.**

The groups below are **not** six lanes on day one. They are the definition-
sharing map, and the map is what tells you how to cut two lanes and where the
seams would be if the phase widens. The opening cut and the widening rule:

- **Lane 1 — nets, labels and flow**: the significant-net rules, the label rule,
  and flow and direction. They share the significant-net definition and the
  power-name lists, so splitting them is splitting a definition.
- **Lane 2 — geometry, text, fields, docs and DNP**: everything standing on the
  geometry module and the two-box model, plus the two independents.
- **The widening evidence is the first merges**, and it is a measurement, not a
  feeling: a first merge in each lane that is **clean, in scope, and green on
  the orchestrator's full check with no re-dispatch** is what buys a third and
  fourth lane. A rejected tick, an out-of-scope diff, or a merge conflict in
  either lane and the phase stays at two.

Recorded because the reason is the interesting part: **six lanes is a bet that
the T1 seam holds**, and the seam is exactly what Phase 1 is measuring. Two
lanes is the shape that is right whether the seam passes or fails, and widening
is cheap in a way that un-widening is not.

| Group | Rules | Shares |
|---|---|---|
| **significant-net rules** | `KI-LBL-001/002`, `KI-RTE-001/002` | the "significant net" definition: ≥ 3 pins, or a bbox diagonal ≥ 20 G, or a user-authored label, or a power net |
| **label rules** | `KI-LBL-003` | ERC's `single_global_label`, defaulted to `IGNORE` |
| **flow and direction** | `KI-FLOW-001/002` | the power-direction name lists (§11.4 Q21), project-overridable |
| **geometry and layout** | `KI-XING-001`, `KI-JCT-001`, `KI-LAY-001…003`, `KI-SYM-001` | the geometry module; `KI-JCT-001` is ERC's `four_way_junction`, defaulted to `IGNORE` |
| **text and fields** | `KI-TXT-002/003`, `KI-FLD-001/002` | the text-metrics port and the two-box model |
| **documentation** | `KI-DOC-001…004` | **published text sources only — the KiCon talk video is not consulted (Q22)** |
| **DNP** | `KI-DNP-001` | |

### `KI-LBL-001` is the one with a rope tied to M4

Its scope, per `spec/SPEC.md` §11.4 and `research/style-rules.md` §4:

- **Detect**: a net whose wire path's bounding-box diagonal ≥ `label_threshold`
  (default **300 G = 381 mm**) and which carries **no label**. Tier 2, weight 2.
- **The knob is `routing.label_threshold`, and it is the ONLY one.** C14
  resolved this rule's threshold and the router's into one key; `spec/SPEC.md`
  §15 carries it, and **neither side may grow one of its own**. A router that
  emits paired labels at 250 mm while the linter penalises above 381 mm would
  argue with itself.
- `research/style-rules.md` §4 calls the knob `labels.distance_threshold`;
  **that name is stale** and M4's T5 corrected it. The spec's name wins.
- The default was 30 G until James's ruling of the M4 checkpoint-2 review; the
  "≈ 381 mm" gloss came from reading `Iu(381_000)` as millimetres. **The length
  stood and the grid-step count moved.** This rule is the reason that mattered.

**Whoever writes this rule reads `carried-4-handle-lint.md` first.** Not for its
subject but for its finding: M5 is *a whole milestone of checks that classify*,
and that entry is three rejections' worth of evidence about what a classifier
can and cannot decide.

---

## Phase 4 — calibration (sequential, and it needs James for one step)

`spec/SPEC.md` §11.6 and `research/style-rules.md` §6.

| Set | Contents | Where from |
|---|---|---|
| **A — good** | 8–12 known-good external sheets plus KiCad's own demos | fetched, **not vendored** |
| **B — bad** | **programmatic degradations of A**, generated in-repo | purpose-built, belongs in `fixtures/` |
| **C — agent output** | sheets an agent produced driving kicli with no style feedback | generated; the dogfood gate can produce these |

Four properties, and note which need a human:

1. **Monotonicity** — `score(A_i) > score(B_i,k)`, decreasing as degradations
   stack. **No human labels needed**, because each bad sheet is a known
   perturbation of a good one. This is the property that makes the whole method
   work.
2. **Rule isolation** — a degradation changes only the penalties of the rules it
   targets. Catches accidental coupling between rules.
3. **Human agreement** — Kendall's τ ≥ 0.7 over ~20 pairs **James ranks**. The
   only step needing a human; ~15 minutes.
4. **Stability** — re-scoring, and scoring after a no-op mutation, is
   bit-identical.

**Weights are not regression-fitted.** Run properties 1–2 and **fix rules, not
weights**, when they fail. Only then run property 3, and change **at most one
weight per iteration**, recording the reason. `K` is frozen **last**, by
requiring set A to land in 85–100 and worst-degraded set B in 30–50.

**And this is where `carried-5-calibration-sweep.md` lands.** M4's gate measures
agreement, not calibration; the sweep that does calibrate was run once and its
numbers are recorded — `w_turn = 6` is better than 0 and **not** better than 60,
and **`w_cross` and `w_text` are exercised by neither sheet**. Set B is the
answer to that last one: a degradation that makes crossings and text collisions
happen is exactly the corpus those two weights have never had.

---

## The carried items, scheduled

| Item | Lands in | Why there |
|---|---|---|
| `carried-1-sheet-pin-angle.md` — a sheet pin whose angle disagrees with its position | **Phase 2**, beside `KI-CONN-001` | whether it earns a `KI-…` code is a §11.4 decision, and §11.4 is what M5 builds. **Recommendation unchanged since M4 T14: report the disagreement, do not correct it silently.** The disagreeing drawing already exists as a recipe — the reflected arm of `edit_wire_sheet_pin.rs`. |
| `carried-2-pin-location.md` — D2, nothing read-only tells an agent where a pin is | **Phase 1**, as its own task | the largest of the first dogfood run's nine defects, and a **design decision about the agent-facing surface**. The answer already exists internally in `route::terminal`, `Terminal::of_pin`, merely unexposed — which is what makes it cheap to build and expensive to design badly. Constitution §6 governs the shape. Defect 6 is the same wound from the other side and folds in. |
| `carried-3-reader-strictness.md` | **Phase 2** | see above |
| `carried-4-handle-lint.md` — the handle rule wants a lint over MIR | **OUT of M5. Backlogged.** | **Ruled, question 5**: a new dependency, a Constitution §9 licence check, **its lesson available without it**, and **no contribution to schematic readability, which is the milestone's goal**. The last clause is the one that decided it and is the north star doing its work. **Its lesson is still owed in Phase 3** — whoever writes a classifying rule reads the entry — and that is now the entry's only claim on this milestone. |
| `carried-5-calibration-sweep.md` | **Phase 4** | see above |

## The chores, and where each fits

Six are filed. None is design work; each has a check that guards it.

| Chore | Fits |
|---|---|
| `chore-1-documented-defaults-sweep.md` — nothing checks a prose gloss against the value the code holds | **before Phase 3.** M5 documents ~28 new defaults. This is the class that let "30 G ≈ 381 mm" survive a milestone of green gates, and it gets much larger this milestone. |
| `chore-2-hidden-pin-fixture.md` — no fixture has a net with no visible pin | **Phase 1**, with T1 — a law verified in two of three terms |
| `chore-3-probe-root-child-ids.md` — `ROOT`/`CHILD` handle collision | any time; bites the first verb addressing a child sheet by handle |
| `chore-4-mutation-loop-control.md` — P1/P2 assertions share an ancestor | any time; the control T21 already built |
| `chore-5-blocked-fixture.md` — `blocked` has no committed fixture | any time |
| `chore-6-window-holds-dead-code.md` — remove `Window::holds` | any time; ruled |
| `chore-7-obstacle-walk-check.md` — the obstacle walk's missing check (from `opening-2`) | **RULED a chore**, question 3. Phase 0; one test in `route_obstacles.rs`, no source change. |
| `chore-8-agent-example-recipe.md` — nothing in the repo reproduces the drawings the `AGENT.md` examples were measured from | **before Phase 2**, per PROPOSED 8, promoted. `sch score` is a new command surface and M5 will touch `AGENT.md` heavily; until this lands, "regenerate it" is a project rather than a command. |

---

## Lane table — PROPOSED, and conditional on T1

Conventions from M4: one lane per subagent, split along module ownership; two
lanes never own the same module; merge hotspots are the orchestrator's.

| Lane | Owns | Phase |
|---|---|---|
| **spine** | `crates/kicli/src/lint/{finding,rule,score}.rs`, `crates/kicli/src/kicad/erc.rs` | 1 (sequential, one lane at a time) |
| **pin location** | `crates/kicli/src/route/terminal.rs` (read), the new read-only view and its CLI seam | 1, beside the spine — disjoint from `lint/**` |
| **A — blocking geometry** | `lint/rules/grid.rs`, `overlap.rs`, `wire_body.rs` | 2 |
| **B — blocking connectivity** | `lint/rules/pin_on_wire.rs`, `hier.rs`, the reader-strictness decision | 2 |
| **C — nets, labels and flow** | `lint/rules/labels.rs`, `route_quality.rs`, `flow.rs` | 3, **opening lane 1** |
| **D — geometry, text, fields, docs, DNP** | `lint/rules/layout.rs`, `junction.rs`, `text.rs`, `fields.rs`, `docs.rs`, `dnp.rs` | 3, **opening lane 2** |
| **calibration** | `crates/kicli/tests/score_calibration.rs`, `xtask` corpus additions | 4 |

**Phase 3's C and D are the two-lane amendment's opening cut.** The six-lane
version of this table is retired rather than deleted: if the first merges in C
and D come back clean, in scope and green with no re-dispatch, C splits into
significant-nets and flow, and D splits into geometry, text-and-fields, and
documentation, along the group boundaries in the Phase 3 table. **The split
seams are pre-drawn precisely so widening is a decision rather than a re-plan.**

**Merge hotspots, orchestrator-only:** `Cargo.toml`, `lib.rs`, `lint/mod.rs`
(**if T1 fails to remove it as one**), `AGENT.md`, `tests/command_surface.rs`,
fixture `MANIFEST`, `spec/SPEC.md`, `kicli.toml`'s `[rules]` table.

**`AGENT.md` is held by one lane at a time**, as in M4 — and now under the
measured-examples rule, so a lane holding it regenerates its blocks from a real
run.

---

## What M5's exit criteria will measure

Written as **what each gate can fail on**, because M4's calibration row is the
lesson: a gate presented as measuring something it cannot fail on is worse than
no gate, since it spends the credibility of a real one.

| Gate | Can fail on |
|---|---|
| **determinism** | re-scoring an unchanged file producing a different byte; a finding order that depends on hash-map iteration |
| **no floating point** | any float under `src/lint/` outside the final `exp` |
| **one knob** | `KI-LBL-001` or the router growing a second threshold key |
| **ERC layering** | kicli implementing an ERC check, or double-counting one it reports |
| **ERC canary** | an ERC report at the wrong unit scale being consumed as if correct |
| **tier separation** | a Tier 1 finding moving the score, or a Tier 2 finding failing the gate |
| **monotonicity** | any degraded sheet scoring ≥ its source |
| **rule isolation** | a degradation moving a rule it does not target |
| **human agreement** | Kendall's τ < 0.7 over James's ~20 ranked pairs |
| **stability** | a no-op mutation changing the score |
| **budget** | a findings view that floods an agent's context on a realistic sheet (Constitution §6, §7.4) |
| **netlist oracle** | still 35 of 35 demo hierarchies — the scorer stands on the extractor |
| **dogfood** | now a **gate**: a naive-agent run has attempted the new commands cold and its defects are triaged |
| **mutation** | `cargo-mutants` over `crates/kicli/src/lint/`, every survivor triaged and recorded, none silently fixed |

---

## What James was asked to rule — ALL FIVE RULED

**Provenance for every ruling below: James's ratification and advisor rulings,
M5 plan review.** Kept as questions with their answers beneath, rather than
rewritten into statements, because the question is what makes the answer
readable — and because a plan that shows what it did not know is a better
record than one that only shows what it decided.

### 1. Ratify or re-cut the plan; are Phase 3's six lanes worth it?

**RATIFIED, with one amendment: Phase 3 opens at TWO lanes**, widening
mid-phase only on evidence of frictionless first merges. The four-phase shape
stands unchanged. The amendment, the opening cut and the widening evidence are
recorded at Phase 3 above, and the lane table carries the pre-drawn split seams.

### 2. The freeze-lift mechanism (`opening-1`'s BLOCKED item)

**RULED: the freeze lift is the ORCHESTRATOR's step, in the main checkout** —
lift before dispatch, restore after merge, **both committed with this ruling's
provenance**. This is the entry's fallback option 2, taken.

**Option 1 — teaching the hook to read the edited file's own worktree list — is
REJECTED, and the reason is the general one**: a hook that resolved its list
from the edited file's worktree would let a lane lift its own freeze inside its
own world. The main-checkout resolution is **privilege separation**, and it is
now deliberate rather than incidental. `lane-o1`'s measurement stands and was
correct in every particular; what it found was a property, not a bug.

**Consequence recorded rather than absorbed**: the M4-close ruling that created
`opening-1` said *"Restore the path in the same commit […] A commit where the
freeze is lifted and not restored must never exist on the branch."* This ruling
says lift-before-dispatch and restore-after-merge, which **necessarily creates
that commit**. Later ruling, same author, addressed specifically at this
mechanism — so it governs, and the earlier words are superseded on this one
point rather than in general. The window is narrowed to what it must be: the
lift commit names the single file and the single change, the restore is the
first commit after the merge, and both carry this provenance. **Flagged to James
in the report's User signal area**, because a superseded rule that nobody
noticed being superseded is how a rule quietly stops meaning anything.

### 3. The obstacle walk's missing check: task or chore?

**RULED: CHORE.** Verbatim: *"the measurement dissolved the possible-defect
proxy my ruling's words conditioned on; task text yields to measured reality,
rulings included."* Filed as `chore-7-obstacle-walk-check.md`. The general half
of the ruling went into the `mutation-run` skill as a **third triage class** —
*reachable but correct: the check is the deliverable* — which is PROPOSED 3
(`lane-o2`'s finding) promoted in the same breath.

The sentence worth keeping: **task text yields to measured reality, rulings
included.** The project already bound lanes to that rule; this is the first time
it was applied to a ruling, by the person who made it.

### 4. `research/style-rules.md` §8's five open questions

- **Q3 (ERC coupling for `--gate`) and Q4 (the score shape and `K = 25`):
  CONFIRMED as answered** by `spec/SPEC.md` §11.2 and §11.5. No work owed beyond
  citing them; T2 and T3 build to those sections.
- **Q2 (the Greenberg video): CLOSED by citation** of James's standing round-6
  ruling — the video is skipped and **the text sources govern**. `KI-DOC-001…004`
  is written from published text only, and the Phase 3 table's Q22 note already
  said so.
- **Q1 (the seed catalogue) and Q5 (the ground-name list): ONE bounded Phase 1
  research task**, `phase1-t5-seed-catalogue-and-ground-names.md`. Its
  deliverable is **PROPOSED answers with sources, awaiting James's ratification
  at the checkpoint** — *the linter's notion of "good" is his to sign*. That
  clause is why this is not a lane decision: Q1 and Q5 are where the tool's
  values are written down, and the north star is the sentence they answer to.

### 5. Is `carried-4`'s MIR lint in this milestone at all?

**RULED: OUT of M5, backlogged**, for four stated reasons: a new dependency, a
Constitution §9 licence check, its lesson available without it, and **no
contribution to schematic readability, which is the milestone's goal**. The
entry records the backlog with those reasons; its lesson is still owed to
whoever writes a classifying rule in Phase 3, and that is now its only claim on
this milestone.
