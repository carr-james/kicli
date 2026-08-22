# The obstacle walk's direction, measured (opening 2) ✅

**Provenance: James's ruling, M4 close review, on the mutation run's triage.**
Verbatim:

> Mutation survivors: the `Obstacles::lay` direction-guard survivor is triaged
> as a POSSIBLE LIVE DEFECT — before the M5 plan is drafted, measure whether
> callers normalise segment order; if any caller can pass a right-to-left
> segment, file it as an M5 task with the measurement, not a chore.

**This task is a measurement, not a fix.** Its deliverable is a number and a
verdict. It does not repair the router, it does not add the missing test as a
by-product, and it does not drive a survivor count to zero — the mutation-run
rule is explicit that fixing survivors is ruled work, and a count driven to zero
by unrecorded edits destroys the evidence the run exists to produce.

## The survivor

From the M4-close `cargo-mutants` run, quoted verbatim (group M-1 in
`mutation-survivors.md`, the highest-severity group of ten):

```
crates/kicli/src/route/obstacles.rs:354:33: replace match guard to.x.0 >= from.x.0 with true in Obstacles::lay
crates/kicli/src/route/obstacles.rs:356:31: replace match guard to.y.0 >= from.y.0 with true in Obstacles::lay
```

`Obstacles::lay` picks its walk direction from that guard and then walks
`steps + 1` cells **from `segment.from`**. `steps` is computed with `.abs()`, so
under the mutation the count stays right and the cells are wrong: a segment
drawn right-to-left or bottom-to-top lays its obstacles **mirrored about its
start point, on the far side of the wire**. Nothing else complains.

## The question, in the order it must be asked

The mutation surviving tells you *no test noticed*. It does **not** tell you the
code is wrong. Two facts are needed and they are different:

1. **Can a right-to-left or bottom-to-top `Segment` reach `Obstacles::lay`?**
   `lay` has exactly one caller — `Obstacles::build`, `obstacles.rs:254`,
   over `sheet.segments`. So the question is whether anything that constructs a
   `SheetGeometry`'s `segments` normalises `from`/`to`, or whether the file's
   own wire order survives to that point. KiCad writes `(xy …) (xy …)` in the
   order the wire was drawn, and a person draws right-to-left routinely.
2. **If one can, does the map then differ?** Establish it by construction, not
   by reading: build the same segment both ways round and compare the resulting
   maps. Equal maps under both orders would mean the guard is doing nothing —
   which is a finding about the code, and a different one.

**Answer 1 before 2.** If no caller can produce a reversed segment, the mutant
is unreachable and the verdict is "a test is missing, not a defect". If one can,
2 says whether the router has been building a wrong obstacle map.

## Falsification, and the trap this measurement is exposed to

Per `.claude/skills/falsification-control/SKILL.md`. Two shapes apply directly:

- **Degenerate equality.** If you compare the two orders' maps and they agree,
  ask what else would make them agree — a comparison that reads a field neither
  order changes, or a construction path that normalises before you look, both
  produce "equal" with nothing being tested. Say what the two sides were derived
  from and whether they share an ancestor.
- **The break that is a no-op.** If you reproduce the mutation by hand and the
  suite stays green, that is expected and is not the measurement. The
  measurement is whether a *reversed segment in a real drawing* changes the map.

## Scope

**IN**
- read anything under `crates/`
- this file, which is where the measurement is recorded
- a scratch reproduction **outside the repository** (`mktemp -d`), if you want
  to run the mutation by hand

**OUT** — every source file. **Commit no change to `crates/`.** If the
measurement shows a live defect, the output is a proposed task entry written
into this file, not a fix. If it shows the mutant unreachable, the output is the
same file saying so, with the missing test named for the plan to schedule.

## Evidence obligations

- Which construction sites feed `SheetGeometry::segments`, named by file and
  line, and what each does with `from`/`to` order.
- The constructed comparison from question 2, with the two maps' difference
  quoted, or the reason question 2 was not reached.
- Whether any existing fixture contains a right-to-left or bottom-to-top wire —
  measured over the fixture corpus, not assumed. This is what decides whether
  the missing test is "add an assertion" or "no fixture can reach it".
- The verdict, in one of exactly two forms:
  - **LIVE DEFECT** — with the M5 task entry drafted beneath it, per
    `.claude/skills/task-entry-recording/SKILL.md`; or
  - **UNREACHABLE** — with the reason stated as a property of the callers, and
    the check that would pin it named.

## Completion check

This task adds no check, so it has no `cargo` completion check of its own, and
saying so is part of the entry rather than a gap in it. What proves it complete
is that the two questions above are answered **by measurement recorded in this
file**, each with what was run and what it showed.

Run `cargo xtask check` before you finish anyway, to establish that you left the
tree as you found it.

---

# The measurement

Run in the lane worktree `.claude/worktrees/lane-o2` at base `a1e25ad`, base
verified as the lane's first action (`git log --oneline -1` → `a1e25ad`,
`git status --porcelain` → empty). **No file under `crates/` was changed.**

Two instruments, both outside the repository, in `mktemp -d` at
`/tmp/lane-o2-WEuTrg`:

- `scan_wires.py` — a text scan of `.kicad_sch` files, reading `from` as the
  **first** `(xy …)` under a `(pts …)` and `to` as the **last**, which is what
  `read_line` does (`crates/kicli/src/model/items.rs:756`).
- `repo/crates/kicli/tests/scratch_direction.rs` — the same questions asked
  through the real reader and the real `Obstacles::build`, in an `rsync` copy of
  this worktree (`target/` and `.git` excluded) so that the mutation could be
  applied to a source tree that is not this one.

## Question 1 — can a right-to-left or bottom-to-top `Segment` reach `Obstacles::lay`?

**Answer: yes, and one does on every route request over the router's own
fixture.**

### Every construction site of a `SheetGeometry`'s `segments`, and what it does with the order

There is exactly one production site, and it copies the file's order verbatim.

| Site | What it does with `from`/`to` |
| --- | --- |
| `crates/kicli/src/route/sheet.rs:218-224` — `SheetObjects::read_wires_and_marks` | `from: line.from, to: line.to`. **No normalisation.** The only production writer of `SheetObjects::segments` (`sheet.rs:109`), which `geometry()` lends out at `sheet.rs:251`. |
| `crates/kicli/src/model/items.rs:756-779` — `read_line`, the upstream of the above | `from: ends.first()`, `to: ends.last()` over the `(xy …)` children of `(pts …)`, in file order. **No normalisation**, and `Schematic::lines()` (`items.rs:613`) yields them "in file order" too. |
| `crates/kicli/src/route/search.rs:507` | `SheetGeometry::default()` — empty. Not a source of segments. |
| `crates/kicli/src/route/search.rs:530-532` | Unit-test literal, one left-to-right segment. |
| `crates/kicli/src/route/obstacles.rs:468-470` | Unit-test literal. |
| `crates/kicli/tests/route_calibration.rs:711-715` | `base.segments.to_vec()` then `extend_from_slice(laid)`; `laid` is built at `route_calibration.rs:1149` from the router's own output. Copies, does not reorder. |

The two production callers of `Obstacles::build` are
`crates/kicli/src/edit/wire.rs:465` and `crates/kicli/src/edit/wire.rs:685`,
both `Obstacles::build(window, &objects.geometry())` over a `SheetObjects` from
`SheetObjects::read`. **There is no normalising step anywhere on that path.**
`Segment`'s fields are plain `pub` `Point`s (`obstacles.rs:165-176`) with no
constructor to hide one in.

### Measured through the real reader, not inferred

`cargo test -p kicli --test scratch_direction -- --nocapture`, test
`q1_reader_keeps_file_order`, three probe drawings differing only in the order
the two `(xy …)` are written:

```
Q1 forward: from=96.52,101.6 to=106.68,101.6  to.x<from.x=false  to.y<from.y=false
Q1 reversed: from=106.68,101.6 to=96.52,101.6  to.x<from.x=true  to.y<from.y=false
Q1 up: from=101.6,106.68 to=101.6,96.52  to.x<from.x=false  to.y<from.y=true
```

The reversed drawing's `Segment` arrives at `lay` with `to.x < from.x`. The
guard at `obstacles.rs:354` is the only thing standing between it and a
mirrored walk.

### And on the router's own fixture

Test `q1_fixture_segments_reaching_lay`, over
`crates/kicli/tests/fixtures/sch/routing/calibration.kicad_sch` loaded through
`Hierarchy::load` → `SheetObjects::read` → `geometry()`:

```
Q1 fixture routing/calibration.kicad_sch: 77 segments reach lay --
right-to-left 12, bottom-to-top 2, other 63
```

**14 of the 77 segments the calibration gate lays on every run are reversed.**
The mutant is not merely reachable; it is reached, today, by a fixture already
in the tree.

## Does any existing fixture contain a right-to-left or bottom-to-top wire?

**Measured, not assumed. Yes — 21 of 104 segments, across 5 files.**

`python3 scan_wires.py crates/kicli/tests/fixtures crates/kicli-sexpr/tests/fixtures`:

```
files scanned: 29
wire/bus segments: 104
  forward: 83
  right-to-left: 15
  bottom-to-top: 6
  diagonal: 0
files containing a reversed segment: 5
```

The five, with their per-file counts (right-to-left / bottom-to-top / segments):

| Fixture | RTL | BTT | segments |
| --- | --- | --- | --- |
| `crates/kicli/tests/fixtures/sch/routing/calibration.kicad_sch` | 12 | 2 | 77 |
| `crates/kicli/tests/fixtures/sch/nets/nets.kicad_sch` | 2 | 2 | 16 |
| `crates/kicli/tests/fixtures/sch/nets/nets_channel.kicad_sch` | 1 | 0 | 4 |
| `crates/kicli/tests/fixtures/sch/item_zoo.kicad_sch` | 0 | 1 | 3 |
| `crates/kicli-sexpr/tests/fixtures/sch/all_items.kicad_sch` | 0 | 1 | 3 |

First examples:

```
crates/kicli/tests/fixtures/sch/routing/calibration.kicad_sch:425  (wire (pts (xy 27.94 20.32) (xy 17.78 20.32)))   right-to-left
crates/kicli/tests/fixtures/sch/routing/calibration.kicad_sch:575  (wire (pts (xy 101.6 66.04) (xy 101.6 55.88)))   bottom-to-top
crates/kicli/tests/fixtures/sch/nets/nets.kicad_sch:494            (wire (pts (xy 38.1 132.08) (xy 30.48 132.08)))  right-to-left
```

So the missing test is **"add an assertion"**, not "no fixture can reach it".

**Two instruments agree.** The text scan reports 12 right-to-left and 2
bottom-to-top for `routing/calibration.kicad_sch`; the Rust reader reports the
same 12 and 2 out of 77. A regex over the file and kicli's own parser were
derived independently and landed on the same numbers.

### The probe drawings the suite builds, too

Classifying every `probe.wire((…))` call site in `crates/`: **46 forward, 11
reversed, 0 diagonal.** The reversed ones include three inside route tests that
do build an obstacle map — `crates/kicli/tests/route_four_way.rs:91`, `:332`
(bottom-to-top) and `:333` (right-to-left) — and eight in `edit_wire_connect.rs`
and `edit_wire_delete.rs`, which reach `Obstacles::build` through
`crates/kicli/src/edit/wire.rs`. **The mutant is executed by the existing suite
and survives it anyway**: it is reached and unasserted, not unreached.

### KiCad's own files, as ground truth rather than training memory

`target/corpus/demos` is the demo tree **after every schematic has been rewritten
by `kicad-cli sch upgrade --force`** (`xtask/src/corpus.rs:87-127`), so what is
in it is what KiCad 10 itself writes:

```
files scanned: 115
wire/bus segments: 21684
  forward: 16650
  right-to-left: 2828
  bottom-to-top: 2026
  diagonal: 180
files containing a reversed segment: 114
```

**114 of 115 files KiCad's own writer produced carry a reversed segment**, 22 %
of all segments. KiCad does not normalise wire point order on save, and any
claim that a caller could rely on it doing so is refuted by KiCad's output.
(The never-canonicalised `target/corpus/qa` tree agrees: 294 files, 9,492
segments, 1,023 right-to-left, 713 bottom-to-top.)

## Question 2 — does the obstacle map then actually differ?

**Answer: no. Under the shipped guard the two orders produce the identical map,
cell for cell. That is the guard working, not the guard idling — established by
removing it and watching the maps come apart.**

### The constructed comparison

Test `q2_same_segment_both_ways`. **What each side was derived from, and whether
they share an ancestor** — they do, deliberately:

- a probe drawing is written with its wire's points in right-to-left order and
  loaded through `Hierarchy::load` → `SheetObjects::read` → `geometry()`;
- `as_read` is `geometry().segments[0]` — the real `Segment` the real reader
  produced;
- `swapped` is `Segment { from: as_read.to, to: as_read.from, ..as_read.clone() }`
  — **the same handle, the same `own_net`, the same two endpoints**, differing
  from `as_read` in nothing but which end is called `from`;
- each is passed alone to `Obstacles::build` with the **same** `Window`, so
  there is no step between the swap and the map that could normalise either.

The comparison is over every occupied cell of each map, printed as
`(column,row) [features]`:

```
Q2 as_read : from=106.68,101.6 to=96.52,101.6
Q2 swapped : from=96.52,101.6 to=106.68,101.6
Q2 as_read map (9 cells):  (14,18)…(22,18)  ForeignWire { handle: "01000001", axis: Horizontal }
Q2 swapped map (9 cells):  (14,18)…(22,18)  ForeignWire { handle: "01000001", axis: Horizontal }
Q2 VERDICT: maps equal = true; only in as_read = []; only in swapped = []
```

### Interrogating the equality, per the entry's degenerate-equality warning

Two explanations for "equal" had to be ruled out, and the falsification below
rules out both at once:

- *a comparison reading a field neither order changes* — refuted: with the guard
  removed, the very same comparison reports 8 of 9 cells differing on each side.
  It reads exactly the field at issue.
- *a construction path that normalises before you look* — refuted: with the
  guard removed, `as_read` and `swapped` land on different columns. Nothing
  upstream had flattened the order; the guard was doing the work.

### Falsification: what was broken, and which assertion caught it

Both M-1 mutants applied together, in the scratch copy only
(`/tmp/lane-o2-WEuTrg/repo/crates/kicli/src/route/obstacles.rs:353-358`), the
guards replaced exactly as `cargo-mutants` writes them:

```
-            Axis::Horizontal if to.x.0 >= from.x.0 => Heading::PlusX,
+            Axis::Horizontal if true => Heading::PlusX,
-            Axis::Vertical if to.y.0 >= from.y.0 => Heading::PlusY,
+            Axis::Vertical if true => Heading::PlusY,
```

The comparison in `q2_same_segment_both_ways` — the `only_a` / `only_b`
difference lists and `a == b` — then reports:

```
Q2 as_read map (9 cells):  (22,18)…(30,18)
Q2 swapped map (9 cells):  (14,18)…(22,18)
Q2 VERDICT: maps equal = false;
  only in as_read = (23,18)…(30,18)   [8 cells]
  only in swapped = (14,18)…(21,18)   [8 cells]
```

The reversed segment's obstacles move from columns 14–22 to columns 22–30:
**mirrored about its start point, on the far side of the wire**, exactly as this
entry predicted. Source restored from `/tmp/lane-o2-WEuTrg/obstacles.orig.rs`
and re-checked (`grep` shows `if to.x.0 >= from.x.0` back at line 354).

### The consequence on the router's own fixture

Test `q2_fixture_map_fingerprint` lays all 77 segments of
`routing/calibration.kicad_sch` onto a page-sized window and counts occupied
cells (FNV-1a over the printed cells as a fingerprint):

| | occupied cells | fingerprint |
| --- | --- | --- |
| shipped guard | 877 | `3ed02579113d81fb` |
| both guards mutated | 890 | `81322a1d46169d22` |

**The obstacle map of a fixture already in the tree changes under the mutation.**
Reachability is not theoretical and the consequence is not zero-sized; only the
absence of an assertion lets it through.

### The break that is a no-op — confirmed, and recorded as not the measurement

With both mutants applied, `cargo test -p kicli` in the scratch copy is **green
on every target** (164 unit tests plus 30 integration targets, 0 failed). Per
this entry's own warning and the third amendment of `falsification-control`,
that is expected, is *case 2* of that skill (the controls do not watch this
rule), and is **not** the measurement. The measurement is the constructed
comparison above, which is *case 2 diagnosed*: the code was innocent and the
suite was blind.

## Correction to this entry's own reasoning

**Task text yields to measured reality.** The section "The question, in the
order it must be asked" says:

> Equal maps under both orders would mean the guard is doing nothing — which is
> a finding about the code, and a different one.

That reading is wrong, and the measurement is the citation. Equal maps under
both orders is precisely what a **correct** guard produces: laying a wire A→B
and B→A must cover the same cells, and the guard is what makes that true. The
guard is emphatically not doing nothing — remove it and 8 of 9 cells move. The
entry's binary ("unreachable ⟹ missing test; reachable ⟹ wrong map") assumed
reachability implies incorrectness. Reachability implies the mutant is
**killable**, which is a fact about the suite, not about the router.

## Verdict

**REACHABLE, AND THE GUARD IS CORRECT — what is missing is the check, and it is
writable from a fixture already in the tree.**

**PROPOSED — this is a third form, not one of the two the entry offers, and the
deviation is deliberate.** Neither offered label can be written without asserting
something the measurement contradicts:

- **UNREACHABLE** is false as a property of the callers. `read_line` and
  `read_wires_and_marks` copy the file's order verbatim, KiCad writes reversed
  segments in 114 of 115 of its own demo schematics, and 14 of the 77 segments
  in `routing/calibration.kicad_sch` reach `lay` reversed today.
- **LIVE DEFECT** is false as a property of the router. Both orders build the
  identical map; the router has never built a wrong obstacle map from a reversed
  wire.

Recommendation: record the verdict in this form and add "reachable but correct —
the check is the deliverable" as a third triage class for the mutation run,
since a survivor over *correct* code on *reachable* input is the ordinary case
and the run's two classes cannot name it. Reversal is cheap: nothing was changed
in `crates/`.

**Both offered forms' obligations are discharged regardless of the ruling.** The
reason as a property of the callers is in Question 1's table; the check that
would pin it is drafted immediately below and is the same artefact the LIVE
DEFECT branch would have demanded.

James's ruling that created this task is satisfied on its own terms: it
conditions the action on reachability alone — *"if any caller can pass a
right-to-left segment, file it as an M5 task with the measurement, not a
chore"* — and reachability is measured YES.

## The M5 task entry this measurement files

Per `.claude/skills/task-entry-recording/SKILL.md`. **Not dispatched**: `M5`'s
`RULES.md` reserves dispatch to ratified plan items and the three `opening-*`
entries, so this is a filed candidate awaiting ratification.

---

### The obstacle walk is direction-blind, and asserted to be (`chore-N-obstacle-walk-direction.md`)

**Provenance: filed by the measurement in
`tasks/M5/opening-2-obstacle-walk-direction.md`, under James's M4-close ruling
on the mutation run's triage.** Not a defect report — the router is correct. A
check for a correct behaviour nothing asserts, whose absence let two mutants
live.

**What is missing.** `Obstacles::lay` (`crates/kicli/src/route/obstacles.rs:339`)
picks its walk direction from two match guards at `:354` and `:356`, and walks
`steps + 1` cells from `segment.from` with `steps` computed under `.abs()`.
Nothing in the suite asserts that the cells it reaches do not depend on which
end of the wire the file happens to write first. Both guards are `cargo-mutants`
survivors from the M4-close run, group M-1 of `mutation-survivors.md`.

**Scope.** `crates/kicli/tests/route_obstacles.rs` only — the file that already
owns "what a route meets on the way, measured on drawings rather than on lists".
No source change: the behaviour under test is already correct.

**The check.** One test, built from drawings and not from a hand-made list, per
that file's own standing rule and the `falsification-control` special case:

- two probe drawings identical but for the written order of one horizontal
  wire's two `(xy …)`, and two more for a vertical wire;
- each read through `SheetObjects::read` and `Obstacles::build` over the same
  `Window`;
- assert the two maps hold the **same occupied cells with the same features** —
  not merely that the endpoints are covered, which the mutation also satisfies;
- and an anti-vacuity control: assert the map is non-empty and spans the
  expected span in cells, so a comparison of two empty maps cannot pass.

**Falsification it must show, and the expected result, already measured here.**
Replace either guard with `true` and the check must fail. Measured in this
task's scratch copy: with both replaced, the reversed segment's nine cells move
from columns 14–22 to columns 22–30, 8 of 9 differing. A check that stays green
under that replacement is not watching `lay`.

**Completion check.**
`cargo test -p kicli --test route_obstacles` passes, and the falsification above
is recorded in the entry.

**Cost.** One test function. No source change, no fixture change, no new
dependency.

---

## This task's completion check

**This task adds no check, so it has no `cargo` completion check of its own.**
That is stated here rather than left as a gap, per the entry's own instruction.
What proves it complete is that Questions 1 and 2 are answered above by
measurement, each with what was run and what it showed.

No oracle check applies: nothing here touches connectivity, and no file is
written by kicli in the course of it.

`cargo xtask check` was run in this worktree to establish that the tree was left
as it was found. **All six gates pass, exit 0** — `fmt`, `clippy`, `test`,
`doc`, `deny`, `clean`, with `clean` confirming "the gates changed no file
outside target/". The measurement lives in a copy outside the repository, and
the only file this task writes is this one; `git status --porcelain` shows
`M tasks/M5/opening-2-obstacle-walk-direction.md` and nothing else.

Corpus- and environment-gated checks do not count toward done from inside a lane
worktree (`CLAUDE.md`, Parallel work); the corpus numbers above were read from
the orchestrator's fetched corpus at `target/corpus` to **make** a measurement
this task owes, which is the permitted use.

## Where the instruments are

Outside the repository, in `/tmp/lane-o2-WEuTrg` (a `mktemp -d`, not preserved
across machines — the commands are the record):

- `scan_wires.py`, falsified before use against a hand-built file carrying one
  forward, one right-to-left, one bottom-to-top and one diagonal segment; the
  first version reported **0 segments of 4**, which is why the falsification is
  load-bearing rather than ceremonial. The defect was a `(pts …)` regex that
  stopped at the first inner `)`. Fixed, re-run, reported exactly the planted
  1/1/1/1.
- `repo/`, an `rsync` copy of this worktree at `a1e25ad` with
  `crates/kicli/tests/scratch_direction.rs` added, run as
  `cargo test -p kicli --test scratch_direction -- --nocapture`.

---

## Tick — APPROVE, 2026-08-22

**Reviewer verdict: APPROVE.** Recorded here beside the tick, per `CLAUDE.md`'s
tick-review rule. Merged `4b2f2d0`; lane `lane-o2`, base `a1e25ad`, one commit
`0f6a87e`, one file, **no change under `crates/`**, confirmed at review start
and again at the verdict.

**What the reviewer measured itself rather than taking on this entry's word**,
because that distinction is the review's value:

- **the "only production writer" claim** — re-derived by reading the code, and
  by `grep -rn "Segment {"` across `crates/`: one non-test production site
  (`sheet.rs:220`), the others an integration test and the struct definition;
  `search.rs:507/528` and `obstacles.rs:466` are inside `#[cfg(test)]`.
- **the corpus numbers** — with its **own** scanner, not this lane's. It got
  **77 segments, 12 right-to-left, 2 bottom-to-top** in the calibration fixture,
  and **115 files, 21,684 segments, 114 files carrying a reversed segment** over
  the demo corpus. Exact match on every headline number.
- **the equality and its falsification** — it wrote its own integration test, ran
  it against an `rsync -a` scratch copy verified byte-identical before reading,
  then replaced **both** M-1 guards with `true` **in that scratch copy only** and
  watched **its own test fail** (`expected 9 occupied cells, got 1`), then
  restored and watched it pass. A first-hand FAIL, not a narrative one.

**Taken on this entry's word:** the scanner's own falsification story (0 of 4
planted segments before its fix — the scratch artefact was ephemeral and gone),
and the six-gate claim, per the standing rule never to run the full gate suite
in a live checkout.

**On the refused verdict binary, the reviewer judged the refusal SOUND**, under
the standing rule that *task text yields to measured reality, with the citation
recorded in the entry*: the entry's own premise — "equal maps under both orders
would mean the guard is doing nothing" — is refuted by the entry's own
falsification, and the lane quoted the false sentence, cited the refutation, and
used PROPOSED rather than silently overriding. Not a BLOCKED case: BLOCKED is
for conflicts between governing documents, not between an entry's assumed
taxonomy and its own measured outcome.

### The one open point, recorded from both sides

James's ruling says *"file it as an M5 task with the measurement, **not a
chore**"*. The trigger it names — reachability — **fired**. The reason it gives —
a possible live defect — **did not hold**.

- **The orchestrator filed it as a TASK in `PLAN.md`**, on the ruling's literal
  words. The orchestrator does not reverse James.
- **The reviewer's reading, recorded rather than folded away:** *"given the
  measurement shows no source defect, 'a check-guarded chore' is RULES.md's own
  correct taxonomy for exactly this shape of work, and it is filed
  PROPOSED/unratified either way, leaving James the actual decision."*

It is one line of `PLAN.md` to move it. It is question 3 of the plan's ruling
requests.
