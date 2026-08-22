# Carried in from M4 — D2, nothing read-only will tell an agent where a pin is

*Migrated verbatim from the former `tasks/M5.md` at the M5 opening, by the
boundary-package ruling that gives M5 one file per task. The text below is the
record as M4 wrote it; nothing was re-argued in the move.*

**Provenance: the M4 dogfood run, defect 2, ratified in full by advisor ruling
2026-08-15 with the explicit promotion "D2 goes to the M5 planning list as a
task".** Full text in `tasks/dogfood.md`.

**What the dogfood agent actually hit.** It had to infer a pin offset from a
label kicli itself had placed, and learned its guess was wrong only from a
**write command's** output. Defect 6 of the same run is the same wound from the
other side: a wire vertex is refused rather than snapped — a ruled and correct
choice — but there is nothing to ask what *would* be accepted, so a first wire
onto a new symbol is trial and error.

**It is the largest of the nine defects that run found**, and it is a design
decision about the agent-facing surface rather than a patch: a read-only way to
ask where a symbol's pins are.

**The answer already exists internally and is simply unexposed** — the router
resolves pins in `route::terminal`, `Terminal::of_pin`. That is what makes this
cheap to build and expensive to design badly: the question is not how to compute
it but what an agent should be able to ask, and in what shape, under
Constitution §6's context budget.
