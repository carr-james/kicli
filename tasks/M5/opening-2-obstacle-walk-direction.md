# The obstacle walk's direction, measured (opening 2)

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
