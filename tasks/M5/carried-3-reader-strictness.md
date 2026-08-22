# Carried in from M4 — the reader question C3 measured

*Migrated verbatim from the former `tasks/M5.md` at the M5 opening, by the
boundary-package ruling that gives M5 one file per task. The text below is the
record as M4 wrote it; nothing was re-argued in the move.*

**Not a chore. It is a defect with a number on it, and it was measured at the M4
close rather than argued about.**

M4's C3 made a probe helper's label shape a type, so no caller can write a bare
`input` where a `(shape input)` list belongs. In doing so it settled the question
the entry had explicitly deferred — *is kicli's lenient reader, which accepted
the bare token, itself a defect?*

**Measured on one fixture with a single `(shape input)` replaced by a bare
`input`:**

| | |
|---|---|
| KiCad reads | **32 nets** from the file KiCad wrote, **36** from the bare-token file — `/CH_A_IN`, `/CH_B_IN`, `/channel_a/CHAN_LOCAL`, `/channel_b/CHAN_LOCAL` all gone |
| kicli reads | **identically.** `kicli sch view --view connectivity` on both projects diffs to *nothing at all* — same `nets=14`, same 47 lines, **no warning** |

**So kicli silently reports connectivity KiCad does not agree with.** That is a
Constitution §1 and §3 matter — a file kicli cannot read the way KiCad does is
a file kicli should refuse or flag, not quietly reinterpret — and it belongs to
whichever M5 item owns the reader's strictness. It is recorded here so that the
measurement is not lost between milestones, since the M4 chore that produced it
was scoped to the probe API and correctly did not act on it.
