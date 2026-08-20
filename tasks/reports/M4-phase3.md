# M4 Phase 3 and the milestone close — consolidated session report

**STATUS: IN PROGRESS.**

Opened 2026-08-20. Phase 2 closed with all seventeen of its tasks merged and
ticked; its report is `tasks/reports/M4-phase2.md` and is preserved unaltered —
a report that rewrites its own history is not a record.

This session's scope, from James's opener: apply five rulings from the
checkpoint-2 review, then Phase 3 (the join, T18 and T19; the calibration gate,
T20; the loop, T21; the agent documentation, T22), then the chores, then the
milestone close — the `cargo-mutants` run, the four counts, the mutation-run
skill, and a doc-diet check.

Appended per tick, not written at the end.

---

## Rulings applied before any dispatch

Provenance for all five: **James's ruling (threshold) and advisor rulings,
checkpoint-2 review**, delivered in this session's opener. Recorded where each
one lands, at the time it was given, per CLAUDE.md's "a direct instruction from
James in a session IS a ruling".

### 1. The threshold BLOCKED item is ruled — option (a), 300 G = 381 mm

The BLOCKED item raised by the label proposal (T13) is closed. The default
becomes **300 G = 381 mm**; the 38.10 mm value descends from reading the
internal-unit constant `Iu(381_000)` with the units dropped.

Recorded in `tasks/M4.md` under T13, "RULED, 2026-08-20 — option (a)", beside
the BLOCKED entry it closes rather than replacing it. Recorded with it:
**`KI-LBL-001` (M5) scores against this value.**

Dispatched as the session's first commit, to a lane at `lane-threshold`. The
ruling required the constant, its assertion, and every gloss in one commit,
since a gloss corrected in a later commit is a window in which the record lies.

Result: **pending** — see the per-lane section below.

### 2. The manual worktree flow is the default dispatch mechanism

Decided on the experiment's evidence. The orchestrator runs `git worktree add`
at the brief's base and briefs a **non-isolated** lane into a pinned path.

This supersedes a rule that had been *rescinded as never executable* — under the
auto flow the dispatch mechanism created the worktree itself, at a commit the
orchestrator could not set. That diagnosis was correct about that mechanism; the
answer was to change the mechanism rather than weaken the rule. Both are kept in
`CLAUDE.md`, because the pair is the lesson.

Standing with it, all three parts recorded:

- the lane's first-action base verification is **retained** — now redundant with
  the mechanism, which is where a check is cheapest and its absence hardest to
  notice;
- **scope verification at every merge** is a standing step: `git diff --stat` of
  the lane branch against its declared scope, main checkout clean before the
  merge begins;
- **the reversal trigger**: a lane found outside its scope returns dispatch to
  the auto flow, pending a ruling.

Landed in `CLAUDE.md` (Parallel work), `.claude/agents/orchestrator.md`
(Dispatch) and `.claude/agents/lane-implementer.md` (base verification).

### 3. PROPOSED 9 promoted — environment variation is a break class

`falsification-control` now names **path, clock, locale and run order** as break
classes against any check consuming a generated value, with the concrete rule:
such a test **runs once from a second directory before it is reported green**.

Worked example added verbatim from the record: the T16 golden defect, which was
invisible to every one of fifteen source breaks — five of which failed those very
goldens — and visible to a directory rename. The distinction the skill now makes:
*a check can be falsifiable and environment-dependent at the same time, and the
procedure as written only tested the first.*

Landed in `.claude/skills/falsification-control/SKILL.md`.

### 4. PROPOSED 10 promoted — reviewers never run the full gate suite live

The `tick-reviewer` definition's rule is promoted from conditional ("while other
lanes are active") to **unconditional**. The condition was one the reviewer could
not reliably evaluate: a phantom `clean` failure is indistinguishable from a real
one from inside the review. Targeted `cargo test --test <name>` runs in the
verified scratch copy are what carry the evidence.

Landed in `.claude/agents/tick-reviewer.md`.

### 5. PROPOSED 13 promoted — the identifier seed goes project-relative

All three sites (`edit/label.rs:294`, `edit/wire.rs:382`, `edit/text.rs:98`) as
**one chore**, so the convention cannot end up split between verbs. Its first act
is the measurement of whether any committed fixture or golden depends on the
absolute-path values, **recorded before the change lands**.

Filed as **C8** in `tasks/M4.md`, with an owner and a time in the chore table.

---

## Per-lane record

*(appended per tick)*

---

## Findings

*(appended as measured)*

---

## PROPOSED items

*(in entry order)*

---

## BLOCKED items

*(none open at the time of writing)*

---

## Workflow retrospective

*(WORKFLOW NOTEs quoted verbatim, attributed, corrections recorded beside the
quote rather than folded into it)*
