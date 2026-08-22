# The instrument that actually calibrates the weights

**Provenance: PROPOSED 9, raised by the re-route calibration gate (M4 T20),
promoted by James's ruling at the M4 close.** The gate's own entry is at
`tasks/M4.md`, "T20 — The re-route calibration gate, at 15 %"; the finding and
its numbers are in `tasks/reports/M4-phase3.md`.

## The finding, which is worth more than the gate

> **This gate measures agreement, not calibration.**

Both sides are costed with the same weights **while the router optimises exactly
that objective**. So no perturbation of any weight moves either sheet outside
±15 %: the lane swept `w_near` 0→20000, `w_turn` 0→10000, `w_cross` 1→100000,
`w_len`, `w_text`, `margin` and `u_max`, and **the fixture reads +0.0 % under
every one of them.** A gate that cannot fail on a wrong weight is not measuring
the weights.

The gate is still worth having — it catches a router that disagrees with a
person about *shape*, and it caught real defects in its own construction. What
was wrong was the milestone's presentation of it as calibration.

## The two parts James ruled

1. **Re-word M4's exit criteria `calibration` row to say what the gate
   measures.** Done at the M5 opening; see `tasks/M4.md`, "Milestone exit
   criteria".
2. **Carry the perturbation sweep into M5 as the instrument that actually
   calibrates**, since M5 owns the weights and inherits these numbers.

## The numbers M5 inherits

The lane also ran **the sweep that does answer the question** — the router
chooses under perturbed weights, then both drawings are scored under the
defaults. Its result:

- `w_near`, `w_turn`, `w_len` and `margin` are each shown to be **doing work**,
  and the defaults sit at a **local optimum**;
- **`w_turn = 6` is confirmed better than 0 and *not* better than 60**;
- **`w_cross` and `w_text` are exercised by neither sheet.**

**The defaults are under-determined rather than wrong.** That is the sentence
that governs M5's freedom with them: a weight may move, with a measurement of
this shape beside it, and `w_cross`/`w_text` cannot be judged at all until a
sheet exercises them.

## What this becomes as a task

Not yet drafted as one; it is proposed in `PLAN.md`. Its shape is: promote the
lane's sweep from a one-off measurement into an instrument that lives in the
repository, with a corpus that exercises `w_cross` and `w_text`, feeding
`spec/SPEC.md` §11.6's calibration properties rather than §9's gate.
