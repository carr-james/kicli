# The obstacle walk is direction-blind, and asserted to be (chore 7)

**Provenance: James's ratification and advisor rulings, M5 plan review,
question 3.** Verbatim:

> the obstacle-walk check is a CHORE — the measurement dissolved the
> possible-defect proxy my ruling's words conditioned on; task text yields to
> measured reality, rulings included.

The measurement that dissolved it is `tasks/M5/opening-2-obstacle-walk-direction.md`,
executed under James's earlier M4-close ruling on the mutation run's triage. That
earlier ruling said *"if any caller can pass a right-to-left segment, file it as
an M5 task with the measurement, **not a chore**"* — its trigger fired
(reachability holds) and its premise did not (the guard is correct). The
orchestrator filed it as a task on the literal words and put the discrepancy to
James; **this ruling is James reversing his own earlier words on measured
evidence**, which is why the sentence above is quoted rather than paraphrased.

**The general half of the same ruling is in the layer**, not here:
`.claude/skills/mutation-run/SKILL.md` gains a third triage class, *reachable but
correct — the check is the deliverable*. This chore is that class's first
worked example.

**Not a defect report — the router is correct.** This is a check for a correct
behaviour nothing asserts, whose absence let two mutants live.

## What is missing

`Obstacles::lay` (`crates/kicli/src/route/obstacles.rs:339`) picks its walk
direction from two match guards at `:354` and `:356`, and walks `steps + 1`
cells from `segment.from` with `steps` computed under `.abs()`. Nothing in the
suite asserts that the cells it reaches do not depend on which end of the wire
the file happens to write first. Both guards are `cargo-mutants` survivors from
the M4-close run, group M-1 of `mutation-survivors.md`:

```
crates/kicli/src/route/obstacles.rs:354:33: replace match guard to.x.0 >= from.x.0 with true in Obstacles::lay
crates/kicli/src/route/obstacles.rs:356:31: replace match guard to.y.0 >= from.y.0 with true in Obstacles::lay
```

**Reachability is measured, not assumed**: `read_line` takes `from` = first
`(xy …)` and `to` = last, in file order; `read_wires_and_marks` copies both
verbatim; **14 of 77 segments** in the calibration fixture arrive reversed, and
114 of 115 demo schematics *rewritten by `kicad-cli` itself* carry one. KiCad
does not normalise. Full working: `opening-2`, Question 1.

## Goal state, as the check that proves it

One test in `crates/kicli/tests/route_obstacles.rs` — the file that already owns
*"what a route meets on the way, measured on drawings rather than on lists"*.
Built from drawings and not from a hand-made list, per that file's own standing
rule and the `falsification-control` skill's special case:

- two probe drawings identical **but for the written order of one horizontal
  wire's two `(xy …)`**, and two more for a vertical wire;
- each read through `SheetObjects::read` and `Obstacles::build` over the same
  `Window`;
- assert the two maps hold the **same occupied cells with the same features** —
  not merely that the endpoints are covered, which the mutation also satisfies;
- **an anti-vacuity control**: assert the map is non-empty and spans the expected
  span in cells, so a comparison of two empty maps cannot pass.

## Falsification obligation, with the expected result already measured

Per `.claude/skills/falsification-control/SKILL.md`. **Replace either guard with
`true` and the check must fail.** Measured in `opening-2`'s scratch copy: with
both replaced, the reversed segment's nine cells move from columns 14–22 to
columns 22–30, **8 of 9 differing**, and the fixture map goes 877 → 890 occupied
cells. A check that stays green under that replacement is not watching `lay`.

**Record the falsification run in this entry**, with the diff of the guard you
broke and the assertion that caught it. Do not commit the broken guard.

**The degenerate-equality warning applies and is why the anti-vacuity control is
not optional**: this check asserts two maps are *equal*, and an equality passes
for a comparison that reads a field neither order changes just as happily as for
a correct one. `opening-2` interrogated the same equality and its working is at
that entry's "Interrogating the equality" section — read it before writing the
assertion, not after.

## Scope

**IN**
- `crates/kicli/tests/route_obstacles.rs`
- this file, for the evidence

**OUT** — every source file. **No source change**: the behaviour under test is
already correct, and a chore that edits `obstacles.rs` has stopped being this
chore. `tasks/M5/opening-2-obstacle-walk-direction.md` is also OUT — it is ticked
and closed; its measurement is quoted here, not amended there.

**If the enumeration above proves wrong, the named check wins over the list.**
Say so in your first paragraph and name what you touched and why.

## Completion check

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo test -p kicli --test route_obstacles
```

passes, **and** the falsification above is recorded in this entry with its
measured result.

## Cost

One test function. No source change, no fixture change, no new dependency.
