# The seed catalogue and the ground-name list, researched (Phase 1, T5)

**Provenance: James's ratification and advisor rulings, M5 plan review,
question 4.** Verbatim:

> Q1 (seed catalogue) and Q5 (ground-name list) become one bounded Phase 1
> research task whose deliverable is PROPOSED answers with sources, awaiting
> James's ratification at the checkpoint — the linter's notion of "good" is his
> to sign.

The questions are `research/style-rules.md` §8, Q1 and Q5. The same ruling
**closed Q2** (the Greenberg video is skipped; the published text sources govern)
and **confirmed Q3 and Q4** as already answered by `spec/SPEC.md` §11.2 and
§11.5. Those three need no work; cite them and move on.

## What makes this one task rather than two

Both questions ask the same thing in two places: **on whose authority does kicli
call a drawing good?** Q1 asks it of the rule catalogue, Q5 of one name list
inside it. Neither is a fact about KiCad that a measurement settles, which is
why the deliverable is a proposal and the ratification is James's.

**The milestone's north star is the sentence these answers serve** (`RULES.md`):
*"The tool must validate the important aspects of quality schematics. It must
never reward a schematic that is impossible to read and understand."* Where a
source is silent or two sources disagree, that sentence is the tie-breaker to
argue against — **in writing, in your proposal**, not silently.

## This task writes no source file

**It contends with nothing.** It runs alongside T1–T4. It touches no crate, no
fixture, no spec section. Its whole output is this entry's proposal section.

## Q1 — the seed catalogue

`research/style-rules.md` §8: *"`research/schematic-lint-rule-catalogue.md` is
not in the repo or its history. If you have it, send it and I will reconcile rule
IDs and any Tier 1/2 assignments you had already made, rather than imposing the
IDs invented here."*

**Half of this question only James can answer** — whether that document exists.
Do not try to find it; **confirm its absence and stop there**: `git log --all
--diff-filter=A -- '*catalogue*'` and a search of the working tree, both pasted.

**The half you CAN answer, and it is the useful half:** the rule IDs and the
Tier 1/2 assignments in `research/style-rules.md` §4 were **invented by this
project**. Establish what each rests on.

Deliver a table over the catalogue's rules — the six Tier 1 and the
twenty-two Tier 2 named in `spec/SPEC.md` §11.4 — with, for each:

- the **published source** that supports it, cited to a URL and a retrieval
  date, or **"no source — invented here"**, which is a legitimate and important
  answer;
- whether the source supports its **tier** (blocking vs scored) or only its
  existence. These are different claims and the catalogue conflates them.

**One source needs its own paragraph.** §9 records that Olin Lathrop's canonical
answer *"returned 403 to automated fetch, so the rule content here comes from
widely-reproduced summaries of it and should be spot-checked against the
original."* **That spot-check is part of this task.** If the original is still
unreachable, say so and say which rules therefore rest on summaries — that is a
provenance fact James should have before he signs.

**Then state the reconciliation cost**, concretely: if James does produce the
seed catalogue, how much work is it to reconcile? Which rules would be
re-numbered, and does any Phase 2 or 3 lane depend on the IDs being stable?
The answer decides whether Phase 2 can start before Q1 is fully closed.

## Q5 — the ground and negative-supply name list

`spec/SPEC.md` §11.4 carries the defaults:

- positive `{+12V, +5V, +3V3, …}`
- ground/negative `{GND, -12V, AGND, DGND, VSS, VEE, GNDA, GNDD, 0V, EARTH}`
- plus **"value starts with `-`" as negative**
- per-project override via `kicli.toml`

`KI-FLOW-001` and `KI-FLOW-002` stand on these lists. Deliver:

- **Whether the defaults cover standard Eurorack**, which §11.4 says they aim
  at, cited to a published Eurorack convention rather than to memory —
  Doepfer's A-100 bus specification is the obvious primary source.
- **What is missing, and what is wrong to include.** Both directions matter:
  a name absent from the list is a power symbol whose direction is never
  checked; a name wrongly in it is a **false finding on a correct drawing**, and
  the north star's second half makes the false finding the more expensive error
  in a tool whose findings an agent acts on.
- **The `+3V3` question, explicitly**: KiCad's own library uses several
  spellings for the same rail (`+3V3`, `+3.3V`). A list that catches one and
  misses the other is worse than a shorter list, because it looks complete.
  Say which spellings KiCad 10's own power symbol library actually ships —
  **read the library, do not recall it.**
- **What the `-` prefix rule catches and what it over-catches.** It is the only
  *rule* among the *lists*, so it is the only one that can be wrong in an
  unbounded way. A net legitimately named with a leading `-` is the falsifying
  case; say whether one exists in KiCad's demos.

## The form of the deliverable

Per `.claude/skills/task-entry-recording/SKILL.md`, **as PROPOSED entries in
this file** — one per question, each with:

- the proposed answer, stated so James can say yes or no to it without
  reconstructing the reasoning;
- the sources, cited to URL and retrieval date, with **"no source" said plainly
  where that is the truth**;
- the recommendation, and what it costs to leave the question open — because
  what James is deciding at the checkpoint is partly whether Phase 2 can start
  without this.

**Do not write the answers into `spec/SPEC.md` or `research/style-rules.md`.**
They are ratified first. A proposal that has already edited the spec is not a
proposal.

## Rules that bind this task specifically

- **NEVER read Konnect source (AGPL).** `CLAUDE.md`. Black-box only.
- **Verify KiCad facts against KiCad 10.0 documentation or behaviour, not
  training memory.** Formats and libraries changed between 7/8/9/10, and a
  power-symbol name list recalled from memory is exactly the failure this rule
  exists for. KiCad's own GPL source, fonts and demo files may be read freely
  (Constitution §9).
- **The Greenberg video is not consulted** — Q2, closed by James's standing
  round-6 ruling. Published text only. This binds `KI-DOC-001…004`'s row of the
  Q1 table too.
- **A source that could not be fetched is reported as unfetched.** Not
  paraphrased from memory of it, and not quietly replaced by a summary without
  saying so — that substitution already happened once in `research/style-rules.md`
  §9 and is the reason this task exists in the shape it does.

## Scope

**IN** — this file only. Reading: anything in the repository, KiCad 10's own
source and libraries, and published web sources.

**OUT** — `spec/SPEC.md`, `research/**`, every crate, every other entry.

## Completion check

**This task adds no code, so it has no `cargo` check.** It is complete when this
file carries two PROPOSED entries, one per question, each with its sources and
its recommendation, and the absence check for the seed catalogue pasted verbatim.

Per Constitution §11 every task names an executable completion check; **this one
names its check as the record's own completeness**, and says so rather than
inventing a `cargo` command that would pass whatever the file said. Recorded as
a deliberate exception, in the same form `opening-2` used for the same reason.
