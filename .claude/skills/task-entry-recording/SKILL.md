---
name: task-entry-recording
description: The task entry format - provenance labels, PROPOSED and BLOCKED forms, evidence structure. Use whenever writing to a task file.
---

# Task entry recording

The task file is the only memory that survives a session. Review happens in
batches from the record, so every entry must support a retroactive ruling
from the record alone.

## Provenance labels

Written at the moment of the claim, never backfilled by outcome.

- A decision James or the advisor made: cite the ruling.
- A call you made yourself: **PROPOSED**, with your recommendation and the
  evidence. A ruling promotes or reverses it later. Never pre-record a
  self-made call as resolved.
- A question you cannot decide: **BLOCKED**, with the options and a
  recommendation.

## Evidence

- Measurements over assertions: what was run, what it showed.
- Task text yields to measured reality — when you correct task text, the
  citation goes in the entry (e.g. T6 corrected the escape sign against
  KiCad's own Device:R geometry and SCH_PIN::GetPinRoot, cited).
- Every check's falsification is recorded (see falsification-control).
- Carried gaps are recorded in the task files that owe them at the moment
  they are identified, not when they bite.

## Form

Good: "PROPOSED: bus laid down as a foreign wire, entries left out
(diagonal, no lattice representation). Recommendation: conservative —
a route must never cross a bus. Revisit trigger: calibration gate reports
routes threading close to entries."

Bad: "Handled bus edge case." — no provenance, no evidence, no way to rule
on it from the record.

## Naming

A task is named by its role, with the number in parentheses: "the
calibration gate (T20)". Numbers drift as a milestone is edited.
