---
name: consolidated-report
description: Structure of the session's consolidated report. Orchestrator use, appended per tick, delivered at every /goal stop.
---

# Consolidated report

Maintained AS YOU GO — append per tick, not at the end. At any /goal stop
(phase complete or BLOCKED), the report is what James pastes to the advisor.

## Structure

1. **Per tick, appended at the tick**: task (named by role, number in
   parentheses), lane, what landed, evidence locations (entry section,
   commits), reviewer verdict.
2. **Findings** attributed to their lane, with measurements and citations —
   not summaries of them.
3. **Reviewer rejections** and their resolutions.
4. **Dogfood defect list** when a run happened.
5. **PROPOSED items** gathered with evidence and recommendation, in entry
   order.
6. **BLOCKED items** with options and a recommendation.
7. **Workflow retrospective**, in seven fixed areas, in this order.

## The retrospective's seven areas

**Provenance: advisor recommendation, James-approved at the M4 close.** The
retrospective used to be one open question, and an open question is answered
from whatever is nearest the top of the writer's mind. Fixed areas are read in
batches by someone comparing this stop against the last one, so the areas stay
in this order and none is dropped.

**"Nothing this stop" is a complete and valid entry.** It is not a failure to
report and it is not padding to avoid. A few lines per area is the norm; an area
grows only when something actually happened in it.

**Every subagent WORKFLOW NOTE is quoted VERBATIM, attributed to its task, under
the area it belongs to.** Never editorialised into a summary — the raw answers
are the data. A correction to a note goes *beside* the quote, never folded into
it. A note that spans two areas is quoted once and cross-referenced.

1. **Score.** What landed: tasks ticked, rejections and their resolutions, gates
   run and their results, lanes merged, what was parked and why. The countable
   part, so a reader can tell scale from prose.
2. **Verification integrity.** Did the checks check? Blind instruments found and
   which of the four kinds each was; falsifications that came back green and how
   they were diagnosed; controls that shared an ancestor with what they
   controlled; any check that passed and should not have. **This is the area
   that most rewards bad news** — a stop with nothing here after a session of
   new checks is a claim, not an observation.
3. **Record quality.** Could a ruling be made from the record alone? Provenance
   labels written at the moment of the claim or backfilled; claims that went
   stale between writing and reading; entries that were hard to brief from
   because they were vague. Name the entry.
4. **Coordination.** Lanes, bases, merges, scope. Base-currency saves and
   misses; scope verification at each merge; sequencing decisions and whether
   they held; anything two lanes collided over.
5. **Layer and tooling.** Agent definitions, skills, hooks, `settings.json`, the
   harness itself. Which rules and prompt lines earned their keep, and which you
   worked around, ignored, or found ambiguous — **name the specific line**. A
   rule that was never load-bearing is worth reporting as such.
6. **Budget.** Context and token cost, wall-clock, what was expensive and what
   it bought. Work that was truncated, sampled or capped, and what was therefore
   NOT covered — a silent cap reads as full coverage.
7. **User signal.** What James or the advisor asked for, and how it landed.
   Rulings applied and anything they did not cover. Items going back for a
   ruling, and what each one costs to leave open.

This section is diagnostic, not graded. "Everything was fine" is a suspicious
answer.

## At a milestone close, three more

Appended after the seven areas, at a close only:

- **The four counts** — reversed PROPOSED items, re-litigated decisions, gate
  failures found after a tick, and BLOCKED items that were decidable from the
  record. Each with the items behind the number, because a bare zero is
  indistinguishable from a number nobody computed.
- **The mutation run's results** — the counts, the two triage classes, and every
  survivor filed rather than fixed. See the `mutation-run` skill.
- **The doc-diet check** — the binding set's line counts, what grew, and whether
  a growth is an append that should have been a restructure.
  `ENGINEERING.md` makes the binding set's readability outrank its
  completeness, and only a per-milestone look enforces that.
