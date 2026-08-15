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
