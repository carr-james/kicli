# Carried in from M4 C4 — a sheet pin whose angle disagrees with its position

*Migrated verbatim from the former `tasks/M5.md` at the M5 opening, by the
boundary-package ruling that gives M5 one file per task. The text below is the
record as M4 wrote it; nothing was re-argued in the move.*

**Decided in M4, not deferred by drift.** The full entry, with the measurement,
stays at `tasks/M4.md` under "C4 — A sheet pin whose angle disagrees with its
position is nobody's"; what follows is why it is here.

**The gap, measured** (M4 T14's third arm, kicad-cli 10.0.5): KiCad puts a
port's connection point on the edge its angle names, keeping the along-edge
coordinate. `SheetPin::at` is the file's value. On a file KiCad wrote the two
agree and nothing is owed. On a hand-written or generated file where they
disagree, `Terminal::of_sheet_pin` gives the router a terminal at a place **KiCad
does not connect**, and a wire drawn to it is dangling in the tool while kicli
reports it routed.

**Why M5 and not M4.** The two open questions are both M5's by subject: whether
the disagreement earns a `KI-…` code is a `spec/SPEC.md` §11.4 decision, and
§11.4 is what M5 builds. **M4 scores no drawing at all**, so there was nothing in
that milestone to attach it to — naming an M4 task would have meant opening a
scoring surface the milestone deliberately had none of, and deciding the larger
question by default in the milestone least equipped to.

**The recommendation carried with it, unchanged since T14: report the
disagreement, do not correct it silently.** A router that moved the port would be
editing the drawing to suit itself. The alternative reading — that the reader
should hand back the position KiCad uses — is defensible, and that it is
defensible is what makes this a decision rather than a typo.

**Not chore-runner eligible.** Detecting the disagreement is mechanical; choosing
between reporting and correcting is not, and neither is deciding whether it earns
a rule code or stays a router-level refusal.

The two checks it owes are written in the M4 entry, including the useful detail
that **the disagreeing drawing already exists as a recipe** — it is the reflected
arm of `crates/kicli/tests/edit_wire_sheet_pin.rs`.
