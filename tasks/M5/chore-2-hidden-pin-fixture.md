# Chore — no fixture exercises a net with no visible pin

**Provenance: PROPOSED 12, raised by the `AGENT.md` lane (M4 T22) while
falsifying D4's law, promoted by James's ruling at the M4 close.**

## The gap

`listed + tallied + hidden = total` holds, but **`hidden` is 0 on all seven
fixture projects that have nets**, so dropping that term from the law leaves the
check green. **The law is verified in two terms of three.**

The lane established this as a **no-op break rather than assuming it** — which
is the discipline working, and is why this is filed as a gap in the instruments
rather than as a defect in the code.

## The chore

A fixture with an all-power-pin net, or an all-off-sheet one under `--sheet`.
Then the third term is exercised and the break stops being a no-op.

## Completion check

The existing law's check, plus a falsification row showing that removing the
`hidden` term now **fails** where it previously passed. That contrast is the
evidence, not the new fixture's presence.
