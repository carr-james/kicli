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
