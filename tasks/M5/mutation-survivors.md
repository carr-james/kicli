# Carried in from M4 — the mutation run's survivors

*Migrated verbatim from the former `tasks/M5.md` at the M5 opening, by the
boundary-package ruling that gives M5 one file per task. The text below is the
record as M4 wrote it; nothing was re-argued in the move.*

**Provenance: the standing `cargo-mutants` run at the M4 close, 2026-08-21,
scoped to `crates/kicli/src/route/` by James's ruling.** Procedure and the
run's own methodology lesson are in `.claude/skills/mutation-run/SKILL.md`.

**Counts: 488 generated, 402 killed, 48 survived (42 genuine, 6 benign), 35
unviable, 3 genuine hangs.**

**These are filed, not fixed.** The rule is explicit: *"Do not silently iterate
the tests to zero survivors during the close. Fixing them is ruled work, not
close-out tidying; a survivor count driven to zero by unrecorded test edits
destroys the very evidence the run exists to produce."*

Each mutant below is quoted **verbatim** from the run. They are grouped by
mechanism so that one piece of work closes each group — a group is a candidate
chore, not ten of them.

**Triaged at the M4 close by James's ruling, and two groups have moved:**

| Group | Ruling |
|---|---|
| **M-1** | POSSIBLE LIVE DEFECT. Measure whether callers normalise segment order **before the M5 plan is drafted**; if any caller can pass a right-to-left segment, file it as an M5 task with the measurement, **not a chore**. That measurement is `opening-2-obstacle-walk-direction.md`. |
| **M-7** | `Window::holds` is dead code — **remove as a chore**: `chore-6-window-holds-dead-code.md`. The unpinned `Window::cell` guard in the same group stays filed here. |
| the rest | stay filed as grouped below. |

**One caution for whoever picks these up.** A survivor means *no test noticed*.
It does **not** mean the code is wrong. Two of these groups are near-certainly
tests that were never written rather than defects; **M-1 is the one that may be a
live defect**, and it is the one to look at first.

## M-1 — the obstacle walk's direction is unpinned — 2 mutants

```
crates/kicli/src/route/obstacles.rs:354:33: replace match guard to.x.0 >= from.x.0 with true in Obstacles::lay
crates/kicli/src/route/obstacles.rs:356:31: replace match guard to.y.0 >= from.y.0 with true in Obstacles::lay
```

**Severity: highest of the ten, and the only one that may be a live defect.**
`Obstacles::lay` picks its walk direction from that guard. Forced to `true`, the
walk always heads `PlusX`/`PlusY` — so **a segment drawn right-to-left or
bottom-to-top lays its obstacles mirrored about its start point, on the wrong
side of the wire.** `steps` is computed with `.abs()`, so the count stays right
and nothing else complains.

The question to answer first is not "which test is missing" but **"do callers
normalise segment order?"** If they do, this is unreachable and the fix is a
test plus a note. If they do not, the router has been building a wrong obstacle
map for some segments and routing through wires it cannot see.
## M-2 — A* optimality under a perturbed heuristic is unasserted — 11 mutants

```
crates/kicli/src/route/search.rs:328:9: replace Field<'_>::heuristic -> i64 with -1
crates/kicli/src/route/search.rs:328:9: replace Field<'_>::heuristic -> i64 with 0
crates/kicli/src/route/search.rs:328:9: replace Field<'_>::heuristic -> i64 with 1
crates/kicli/src/route/search.rs:337:45: replace - with + in Field<'_>::estimate
crates/kicli/src/route/search.rs:337:9: replace Field<'_>::estimate -> i64 with -1
crates/kicli/src/route/search.rs:337:9: replace Field<'_>::estimate -> i64 with 0
crates/kicli/src/route/search.rs:337:9: replace Field<'_>::estimate -> i64 with 1
crates/kicli/src/route/search.rs:338:42: replace - with + in Field<'_>::estimate
crates/kicli/src/route/search.rs:339:47: replace * with + in Field<'_>::estimate
crates/kicli/src/route/search.rs:339:59: replace + with - in Field<'_>::estimate
crates/kicli/src/route/search.rs:341:22: replace += with -= in Field<'_>::estimate
```

Constant and perturbed heuristics survive. A **constant** heuristic is admissible
and A* still returns an optimal path, so those are near-benign; the **arithmetic**
mutations of `estimate` are not — they can over-estimate, which breaks
admissibility and permits a suboptimal route.

Nothing asserts optimality under a perturbed heuristic. The check the group wants
is the one T19 built for its own multi-source claim: **the route's cost must be no
worse than a route obtained another way.**
## M-3 — `turns_again`'s direction table is unpinned — 12 mutants

```
crates/kicli/src/route/search.rs:358:20: replace match guard dx > 0 with true in turns_again
crates/kicli/src/route/search.rs:358:23: replace > with >= in turns_again
crates/kicli/src/route/search.rs:359:20: replace match guard dx < 0 with false in turns_again
crates/kicli/src/route/search.rs:359:20: replace match guard dx < 0 with true in turns_again
crates/kicli/src/route/search.rs:359:23: replace < with <= in turns_again
crates/kicli/src/route/search.rs:359:23: replace < with == in turns_again
crates/kicli/src/route/search.rs:359:23: replace < with > in turns_again
crates/kicli/src/route/search.rs:360:20: replace match guard dy > 0 with false in turns_again
crates/kicli/src/route/search.rs:360:20: replace match guard dy > 0 with true in turns_again
crates/kicli/src/route/search.rs:360:23: replace > with < in turns_again
crates/kicli/src/route/search.rs:360:23: replace > with == in turns_again
crates/kicli/src/route/search.rs:360:23: replace > with >= in turns_again
```
## M-4 — the cost tally in `route_of` is unpinned — 3 mutants

```
crates/kicli/src/route/search.rs:465:25: replace += with *= in route_of
crates/kicli/src/route/search.rs:465:25: replace += with -= in route_of
crates/kicli/src/route/search.rs:466:26: replace += with *= in route_of
```
## M-5 — `alternatives_considered` is reported and never asserted — 2 mutants

```
crates/kicli/src/route/search.rs:247:9: replace Search::expanded -> u32 with 0
crates/kicli/src/route/search.rs:247:9: replace Search::expanded -> u32 with 1
```

`Search::expanded` feeds `alternatives_considered` in the route report — a number
this project **reasons with**: T18's tick reviewer used it (39 against 52) as its
evidence that a break was masked rather than blind. Three call sites read it; none
asserts a value.

A number that carries an argument and that nothing pins is exactly the shape of
the threshold gloss this milestone opened with.
## M-6 — the crowded-point test is unpinned — 3 mutants

```
crates/kicli/src/route/terminal.rs:358:39: replace < with <= in has_room
crates/kicli/src/route/terminal.rs:358:5: replace has_room -> bool with true
crates/kicli/src/route/terminal.rs:412:59: replace < with <= in clear_of_four_way
```
## M-7 — `Window::holds` is dead code, and `Window::cell`'s guard is unpinned — 3 mutants

```
crates/kicli/src/route/window.rs:105:9: replace Window::holds -> bool with false
crates/kicli/src/route/window.rs:105:9: replace Window::holds -> bool with true
crates/kicli/src/route/window.rs:83:34: replace || with && in Window::cell
```

**`Window::holds` has no callers at all** — which is why both `-> true` and
`-> false` survive. `clippy` does not flag it because the method is `pub`.

**A mutant surviving in both directions is the signature of dead code**, and that
is worth more as a lesson than the method is as code. Decide whether it earns its
place; if it does, give it a caller and a check.
## M-8 — the detour geometry is unpinned — 2 mutants

```
crates/kicli/src/route/shapes.rs:288:32: replace == with != in detour
crates/kicli/src/route/shapes.rs:291:38: replace - with / in detour
```
## M-9 — `keep_smallest`'s tie-break direction is unpinned — 1 mutant

```
crates/kicli/src/route/obstacles.rs:416:50: replace < with > in keep_smallest
```
## M-10 — `Frontier::admit`'s bookkeeping is unpinned — 3 mutants

```
crates/kicli/src/route/search.rs:282:13: replace + with * in Search::advance
crates/kicli/src/route/search.rs:285:18: replace += with *= in Search::advance
crates/kicli/src/route/search.rs:404:18: replace + with - in Frontier::admit
```

## The six benign survivors, with the reason each stands

```
crates/kicli/src/route/terminal.rs:81:27: replace > with >= in Heading::between
crates/kicli/src/route/terminal.rs:83:27: replace > with >= in Heading::between
```
The `(0, 0)` arm above these already matched, so the operand cannot be zero and
`> 0` and `>= 0` are the same predicate on every reachable input.

```
crates/kicli/src/route/obstacles.rs:416:50: replace < with <= in keep_smallest
```
On equal strings both forms assign the same value.

```
crates/kicli/src/route/shapes.rs:259:29: replace > with >= in outlines
```
A divide-by-zero guard on a grid step that is never zero.

```
crates/kicli/src/route/search.rs:420:9: replace Frontier::is_stale -> bool with false
crates/kicli/src/route/search.rs:422:39: replace < with > in Frontier::is_stale
```
**Equivalent by a measurement already recorded in the source.** `Frontier::admit`
documents: *"Measured, 2026-08-15 — refusing every re-offer outright changes no
answer this milestone's checks can see."* Skipping the staleness check costs
time, not correctness.

**Nothing else was placed in this class**, including mutants whose practical risk
is plainly low. *A generous benign class is how a mutation run talks itself into a
good score.*

## The three genuine hangs, which are detections rather than survivors

```
crates/kicli/src/route/search.rs:281:13: replace + with - in Search::advance
crates/kicli/src/route/search.rs:282:13: replace + with - in Search::advance
crates/kicli/src/route/search.rs:391:59: replace <= with > in Frontier::admit
```
Still timing out at 900s on an idle machine. A mutated loop or frontier bound
that never terminates is *detected* — no test suite passes with it — but it is
recorded apart from the kills, because the evidence is different in kind: nothing
asserted a wrong value, the run simply never ended.
