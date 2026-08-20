---
name: falsification-control
description: How to show a check is capable of failing before it counts. Use when writing or reviewing any test, sweep, or gate.
---

# Falsification control

A check that cannot fail is decoration. Before a check counts toward a tick,
it is shown capable of failing, and the falsification is recorded in the task
entry.

## Procedure

1. State the check against reality, not against the code's own structure — a
   test that restates the implementation passes just as happily when both are
   wrong.
2. Break the thing the check watches, in the source, deliberately.
3. Watch the check fail. If it stays green, the check is not watching what
   you thought.
4. Restore the source; record in the task entry WHAT was broken and WHICH
   assertion caught it.

**Record exactly what you removed, not only where the failure surfaced.** A row
naming a single line number for what was really a two-assertion removal reads as
precise and is not: the next reader removes that one line, watches a *different*
assertion catch the break, and has to work out whether the row is imprecise or
fabricated. Name the assertions by their message or their function, and if you
removed several to isolate one, say so. Provenance: the four-way (T12) review,
where exactly this cost the reviewer real time — it resolved as imprecision, but
only after the work of ruling out the alternative.

## Green after a deliberate break is a finding about the instrument

**Third amendment.** Step 3 says "if it stays green, the check is not watching
what you thought" — and the trap is that green *feels* like good news, so the row
gets skipped past and the table records a break that "did not apply". It is never
good news. **A break that leaves the check green means the instrument may be
blind, and the instrument is investigated before any conclusion is drawn from
that row.**

Two known cases, and telling them apart is the investigation:

1. **The break was a no-op** — nothing about behaviour actually changed, so
   nothing could have been caught. The row is real evidence about the code:
   something else already enforces the rule.
2. **The check does not watch what it claims.** The code was innocent and the
   control was blind. This is the dangerous one, because the check will keep
   passing forever and no one will look again.

Never record case 2 as case 1. "Removing it changes nothing" is the same
sentence for "this rule is redundant" and "my test cannot see this rule", and
those are opposite findings.

### Worked examples — the control was blind, twice, in one task

From the determinism property test (T11). Both breaks left green checks over
innocent code:

- **A shuffle that permuted nothing.** `reordered()` was broken to ignore its
  `order` argument, and the check passed — because the control compared the
  drawing against its own text, so it agreed on layout alone. Rewritten to
  compare against the same file through the same writer, unshuffled, the break
  fails. A determinism check that passes when nothing varies is the exact
  failure mode the check exists to prevent.
- **Every answer replaced by a constant.** This passed the shuffled arm, which
  carried no class counters. The baselines now must hold a shape route, an A\*
  route and a refusal, and the break fails both arms.

The tick reviewer re-made both breaks against the current control *and* against a
reconstruction of the old one, so what the record holds is a **contrast** rather
than a pass. That contrast is the evidence.

### Related precedent — case 1, held apart on purpose

The candidate shapes (T9) measured that with **both** `polyline` guards removed,
all five drawing checks still pass, and did **not** read that as the guards being
safe. It established the structural reason — a terminal's own body box covers
every neighbour of its own cell except the escape point, so the map blocks the
reversing leg anyway — and recommended keeping both rules while measuring them
in `route::shapes::tests` where they can be seen. That is case 1 diagnosed as
case 1, with the work shown.

## Environment variation is a break class

**Promoted from PROPOSED 9 by advisor ruling, checkpoint-2 review.** The
procedure above breaks the **source**. A check can be falsifiable against every
source break and still be asserting a property of the machine it ran on, because
nothing in steps 1–4 ever varies the machine.

So: **path, clock, locale and run order are break classes**, and they apply to
any check that consumes a **generated value** — an identifier, a timestamp, a
hash, a temporary directory, a sort over anything unordered.

The concrete rule, and it is cheap: **such a test runs once from a second
directory before it is reported green.** Rename the scratch directory, or run
the copy from a different absolute path, and run it again. One extra run.

The tell that you need this: the check's expected value was *produced by running
the code* rather than *derived from the contract*. A golden is the common case.

### Worked example — the T16 golden defect

Two `routed` goldens passed every gate in the lane worktree and failed the
moment the orchestrator ran them after merge. The identifiers in them are a
SHA-256 of a seed built from the drawing's **absolute path**, so the goldens
asserted a property of the worktree they were written in.

The falsification table for that task had **fifteen rows**, and rows 2, 3, 4, 5
and 14 all broke the renderer and all failed these same two goldens. The goldens
*were* shown capable of failing. That is necessary and not sufficient:

> a check can be falsifiable and environment-dependent at the same time, and the
> procedure as written only tests the first. **Every break was made in the
> source; none was made in the environment.**

The reproduction is the whole rule in one line: changing **only** the probe's
scratch directory name made the pre-fix commit fail those two tests and no
others. Under the changed path the written identifiers were `fa9bd366…`,
`6ebfadf1…`, `9b63e57e…`; under the original, `ebb43fde…`, `d42bd368…`,
`85a91ae2…`.

**And note which failure mode this is.** The values were *stable* per checkout
and *wrong* everywhere but one — worse than random, because random fails loudly
on the second run, and this failed only on somebody else's machine.

The fix was not to freeze the identifiers: `matches_golden` normalises each
**distinct** identifier-shaped string to `<id-1>`, `<id-2>` … in first-appearance
order, so count and ordering are still asserted, and the real values keep their
own check on shape and distinctness so the normalisation hides nothing.

## Commit the good state before you break anything

**Git can only restore committed state.** Step 4 assumes the source can be put
back, and `git checkout -- <path>` puts back **the last commit**, not the thing
you had a moment ago. So: **before any deliberate break, the good state is
committed** — whether the file is new or tracked with uncommitted changes.

**A tracked file reads as safe and is not.** `git checkout --` on it succeeds,
exits 0, and takes your uncommitted work with it. That is the trap: the command
does exactly what it promises, and what it promises is not what you wanted.

Two adjacent traps from the same batch. `git checkout --` with several pathspecs
restores **nothing** when one of them fails, so a multi-path restore that
includes a new file silently leaves every break in place. And a restore is not
complete because the command exited 0 — checksum the restored file against the
good state before the next break.

*(Amended twice. First at the M4 checkpoint-1 review, covering brand-new files;
then widened immediately after the incident below showed the same failure on a
tracked one. The rule is not about whether git knows the **file** — it is about
whether git knows the **state you want back**.)*

### Worked example — a brand-new file

The `wire draw` (T14) implementer's note, verbatim:

> Second: `crates/kicli/src/edit/wire.rs` is untracked until the first commit, so
> `git checkout --` cannot restore it — falsification runs on a brand-new file
> need a commit of the good state first, which the falsification-control skill
> does not mention.

### Worked example — a tracked file, which is the one that surprises people

From the orchestrator's own report, on the contract amendment `dd4f659`:

> The first falsification break above was made against `report.rs` while the
> whole contract change was **uncommitted**.
> `git checkout -- crates/kicli/src/route/report.rs` duly restored the file to
> `HEAD` — and took the entire change with it, not just the break. The file was
> tracked, so the amendment as written did not cover it; the failure is
> identical.

Cost: four edits re-applied. In a lane it would have been a task's work.

## Worked examples from the record

- **Sign inversion (M4 T6).** The escape rule was stated against the symbol's
  body box, not against the code's sign. The implementer then inverted the
  sign in the source and watched the test fail — the control that
  distinguishes a test of the rule from a restatement of the implementation.
  A restatement would have passed with every route leaving its pins through
  the symbol.
- **Sweep with a found-something control (M4 T5).** A sweep asserting the
  absence of a retired name also asserts that the canonical name IS found —
  so a sweep that read nothing cannot pass. Every absence check carries a
  presence control.
- **Anti-vacuity trip (M4 T8).** A falsification fired the anti-vacuity
  control rather than an arithmetic assertion — corners counted as zero were
  caught by the check that the check itself saw work. Controls before
  conclusions.

## Special case: hand-built fixtures

Hand-built test geometry is permitted only where no drawable request can
distinguish the behaviour (ruled, M4 T7). A hand-made fixture encodes the
same assumptions as the code that reads it, so agreement proves nothing —
build fixtures through the harness wherever a real request can reach the
behaviour.
