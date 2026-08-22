---
name: mutation-run
description: The milestone-close cargo-mutants procedure - scoping, the four counts, the three triage classes, and why a timeout is not a result. Use at a milestone close, never per-commit.
---

# The mutation run

A batch verification run at a milestone close. **Hours-scale. Not a per-commit
gate, and it never becomes one** — a gate a developer dreads is a gate a
developer skips.

## Why it exists

It mechanises the falsification rule **after the fact**.

Hand falsification proves a check can fail **at birth**: the implementer breaks
the code and watches the check go red. Nothing keeps proving it afterwards. A
surviving mutant is a check whose coverage **decayed** since — the code changed
underneath it, or a refactor made it vacuous, and nobody was watching.

The two instruments answer different questions and neither replaces the other.

## Procedure

1. **Only after every task of the milestone is ticked and the gates are green.**
   Nothing about it runs before then.
2. `cargo install cargo-mutants`. **Read `cargo mutants --help` for the current
   file-filtering options** rather than anyone's memory of them; they change.
3. Scope it. M4's ruling scoped it to `crates/kicli/src/route/`. Widening is a
   ruling, not a default.
4. `cargo mutants -f '<glob>' -j <n> --output <dir>`.
5. **Check `--list` first.** It tells you the mutant count before you spend hours
   on it, and it tells you whether your glob caught the files you meant.
6. Triage every survivor. Report the four counts.

## THE RULE THIS RUN EXISTS TO TEACH: run it on a quiet machine

**A `cargo-mutants` timeout is not a result. It is the absence of one.**

`cargo-mutants` sets its per-mutant timeout from a **baseline test run measured
on whatever machine load exists at the time.** If you start the batch while
anything else is building, the baseline is inflated, the timeout is set from it,
and then every mutant test competes with that same load. Mutants that would have
been *caught* — or *missed* — time out instead, and a timeout tells you **nothing
about the mutant's true outcome**.

### The worked example, and it is the whole reason this file exists

The M4 run, started while lanes were still building:

| | Loaded machine | Quiet machine |
|---|---|---|
| baseline test | — (auto-timeout **191s**) | **35s** |
| result | 303 caught, **0 missed**, 150 timeouts | — |

**"0 missed" was false.** Re-running the 150 timed-out mutants on the idle
machine at `--timeout 900` produced **48 survivors**, and **zero** load-induced
timeouts. Only **3** of the original 150 were genuine hangs.

Had the run been banked as it stood, the milestone would have recorded a perfect
mutation score over its router while **42 genuine coverage gaps went unfiled** —
including one that is arguably a live defect.

**So:**

- **Run it on an otherwise idle machine.** Nothing else building, no lanes, no
  gate runs, no editor indexing.
- **If the run reports any timeouts, it is not finished.** Re-run exactly those
  mutants with a generous `--timeout` on a quiet machine before reporting counts.
- **A timeout that survives a generous timeout on a quiet machine is a genuine
  hang** — a mutated loop bound — and is legitimately *detected*, since no test
  suite passes with it.

Re-running only the timed-out subset: build a regex of their `file:line:col`
prefixes from `timeout.txt` and pass it to `--re`.

```sh
cut -d: -f1-3 mutants.out/timeout.txt | sort -u \
  | sed 's/\./\\./g' | paste -sd'|' - > /tmp/re.txt
cargo mutants -f '<glob>' --re "$(cat /tmp/re.txt)" --timeout 900 -j 3
```

## The four counts

**generated, killed, survived-genuine, survived-benign.**

The third triage class below (**reachable but correct**) counts under
`survived-genuine`, because a check is owed. Say how many of the genuine
survivors are of that kind, since the two need different work: one is a bug plus
a check, the other is a check alone.

Report unviable and timeout separately — they are not survivors and they are not
kills. **Never fold a timeout into "killed" to make the arithmetic close.**

## The three triage classes

### Genuine coverage gap

The mutant changes behaviour and no test notices. **File each as a chore or a
task, with the mutant quoted verbatim.**

> **Do not silently iterate the tests to zero survivors during the close.**
> Fixing them is ruled work, not close-out tidying. A survivor count driven to
> zero by unrecorded test edits destroys the very evidence the run exists to
> produce.

### Reachable but correct — the check is the deliverable

**Added by James's ruling, M5 plan review, question 3.** The mutant's input is
reachable, the map or the answer really does change under it, **and the code as
written is right**. Nothing is broken. What is missing is a test.

**This is the ordinary case, not an exotic one**, and the skill had no name for
it. The gap was found by `lane-o2` while executing a ruling whose two mandated
verdicts were `LIVE DEFECT` and `UNREACHABLE`:

> **WORKFLOW NOTE, `lane-o2`, verbatim:** *"The entry's two mandated verdict
> forms encode a false dichotomy — they assume a reachable mutant implies
> incorrect code, but reachability only makes a mutant *killable*, so the
> ordinary outcome 'reachable, correct, untested' has no label and forces a lane
> to either deviate or write something false; the mutation-run skill's two
> triage classes have the same hole."*

**Reachability makes a mutant killable. It does not make the code wrong.** Those
are two measurements and they are asked in that order.

The worked example is the M4 run's best survivor, `Obstacles::lay`'s direction
guard. Reachability: **YES** — 14 of 77 segments in the calibration fixture
arrive reversed, and 114 of 115 demo schematics rewritten by `kicad-cli` itself
carry one. Correctness: **also yes** — the same segment swapped end for end
builds an identical map, cell for cell, and that equality is **not degenerate**,
because with both guards forced to `true` the maps differ in 8 of 9 cells and the
fixture map goes 877 → 890 occupied cells. Verdict: reachable, correct, untested.
Filed as `chore-7-obstacle-walk-check.md` — **one test, no source change.**

**File it like a genuine gap**, with the mutant quoted verbatim and the
reachability and correctness measurements beside it. The deliverable is the
check. Do not write the check during the close; that is the same "do not iterate
to zero survivors" rule, and it applies here in full.

**And the brief-writing half, which is the general lesson**: *a brief that
mandates two verdicts, where a third outcome is possible, forces a lane to
deviate or to write something false.* Offer the third wherever you offer two.

### Benign survivor

Message wording, logging, **genuinely equivalent code** — recorded with the
reason it stands.

**Be strict about this class, and note the boundary it now shares.** "The
consequence is small" is not equivalence, and neither is "the code is correct" —
correct-but-killable belongs in the class above, where it owes a test. Only put a
mutant here when you can state *why no input distinguishes it*. In the
M4 run, only 6 of 48 qualified:

- `Heading::between`'s `dy > 0` → `dy >= 0`, twice — the `(0, 0)` arm above it
  already matched, so the operand cannot be zero and the two forms are the same
  predicate.
- `keep_smallest`'s `<` → `<=` — on equal strings both assign the same value.
- `outlines`' `grid.0 > 0` → `>= 0` — a divide-by-zero guard on a value that is
  never zero.
- `Frontier::is_stale -> false` and its `<` → `>` — **equivalent by a measurement
  already recorded in the source**: the module documents that refusing every
  re-offer outright changes no answer, so skipping the staleness check costs time
  and not correctness.

Everything else was filed, **including mutants whose practical risk is low**. A
generous benign class is how a mutation run talks itself into a good score.

## What a good survivor tells you

The M4 run's most valuable survivor:

```
crates/kicli/src/route/obstacles.rs:354:33:
  replace match guard to.x.0 >= from.x.0 with true in Obstacles::lay
```

`Obstacles::lay` picks its walk direction from that guard. Forced to `true`, the
walk always heads `PlusX` — so a segment drawn right-to-left lays its obstacles
**mirrored about its start point, on the wrong side of the wire.** `steps` is
computed with `.abs()`, so the count stays right and nothing else complains.

No test noticed, because every fixture happens to draw its wires in the positive
direction. That is not a test that decayed; it is a case nobody thought of — which
is the other thing a mutation run finds.

The second most valuable was `Window::holds -> true` **and** `-> false` both
surviving. Both survive because **nothing calls it**. `clippy` does not flag it:
the method is `pub`. A mutant surviving in both directions is the signature of
dead code.
