# kicli

CLI tool giving LLM agents eyes and hands in KiCad 10 projects. Rust.

- CONSTITUTION.md is binding on all work. Read it before doing anything. If a
  task seems to require violating it, stop and ask instead of proceeding.
- spec/SPEC.md is the specification. research/ holds ground-truth docs.
- NEVER read Konnect source code (AGPL). Black-box observations only.
- Verify all KiCad facts against KiCad 10.0 documentation/behaviour, not
  training memory — formats changed between 7/8/9/10.
- ENGINEERING.md is binding on all code. The gates it lists are part of "done"
  for every task — run them before marking any task complete.
- When two governing documents conflict, do not resolve by precedence — mark
  the item BLOCKED with both readings and ask.
- Large builds start fresh sessions. Work is dispatched, merged and recorded at
  task grain; the session itself runs under /goal to a checkpoint stop. An
  interrupted session is wound down per the orchestrator definition's procedure
  — the task file is the handoff, updated before the stop, not after.

## Parallel work

- Subagents doing implementation work run with worktree isolation under
  .claude/worktrees/. One task lane per subagent, split along crate/module
  ownership boundaries — two subagents never own the same module.
- The orchestrator does not write code while subagents are active; it assigns,
  reviews, merges, and resolves.
- Merge hotspots (Cargo.toml, lib.rs module lists, xtask) are touched only by
  the orchestrator, or by exactly one designated lane.
- A lane is complete when its own `cargo xtask check` passes in its worktree;
  the milestone is complete only when the check passes on the merged result in
  the main checkout. The merged check is the orchestrator's job and is never
  skipped.
- Corpus-gated and environment-gated checks in a lane worktree never count
  toward done — only the orchestrator's merged run does. A lane may still run
  them to MAKE a measurement its task owes. The orchestrator runs the full
  check, corpus included, at every lane merge — not only at milestone end.
- A parked lane's uncommitted draft is reference, not resumption: a fresh
  implementer starts from the entry, and any line adopted from a draft passes
  the falsification discipline as if newly written.
- **Worktree currency is the lane's first action, and that is the mechanism —
  not a safety net.** Every brief names the base commit. The lane's **first**
  action verifies its worktree against it, fast-forwards only if the base is a
  descendant of the worktree's commit and the tree is clean, and stops and
  reports otherwise. The orchestrator **confirms the lane's base verification
  appears in its output before treating the work as started.** The rule as it
  acts lives in the `lane-implementer` definition; this is the cross-agent half.
  Three saves in three dispatches, which is why it is load-bearing rather than
  defensive: work built on a stale base is discovered at merge, the most
  expensive possible moment.
- **The manual worktree flow is the default dispatch mechanism.** Ruling:
  James, checkpoint-2 review, on the experiment's evidence. The orchestrator
  runs `git worktree add <pinned path> -b <lane branch> <base>` itself and
  briefs a **non-isolated** lane into that pinned path. Isolation is not
  requested from the dispatch mechanism; the worktree already exists, at a
  commit the orchestrator chose. Standing with it, all three parts:
  - the lane's first-action base verification is **retained** — the mechanism
    now agrees with the rule, which is the case where a redundant check is
    cheapest and its absence hardest to notice;
  - **scope verification is a standing step at every merge**, not a spot check:
    `git diff --stat` of the lane branch against its declared scope, and the
    main checkout clean before the merge begins;
  - **the reversal trigger is recorded, and it governs UNDISCLOSED scope
    excess**: a lane found to have written outside its declared scope *without
    saying so* returns dispatch to the auto flow, pending a ruling. The manual
    flow trades enforcement for control, and that trade is only sound while
    lanes stay inside their briefs **or say when they did not**.

    *Re-worded by James's ruling at the M4 close, on the trigger's first
    opportunity to fire.* The handle chore (C1) wrote to a file that was not on
    its brief's IN list — because the brief's scope list had been derived from
    an enumeration that undercounted, so the brief set a goal state its own list
    made unreachable. The lane took the named check over the derived list, wrote
    four lines, **reported the deviation in its first paragraph, and filed it in
    the entry as PROPOSED**. That is the control working, and reversing on it
    would have punished the behaviour the rule wants. The defect was the
    orchestrator's brief, and its fix is in the orchestrator definition: a brief
    that derives scope from an enumeration says which wins when the enumeration
    proves wrong.
- *Superseded, and recorded rather than deleted: "a lane worktree is created at,
  or reset to, the base as part of dispatch" was rescinded as **never
  executable** — the auto dispatch mechanism created the worktree itself, at a
  commit the orchestrator did not choose and could not set, at the moment the
  agent started.* That diagnosis was correct **about that mechanism**, and the
  answer was to change the mechanism rather than the rule. Kept because the pair
  is the lesson: a rule that cannot be performed reads like a rule that is being
  ignored, and the fix is sometimes a different mechanism rather than a weaker
  rule.
- The BLOCKED rule applies inside lanes: a subagent that hits a governing-
  document conflict parks it and reports to the orchestrator; the orchestrator
  parks it for James.

## Tick review

Adopted on advisor recommendation, M4 Phase 2 open.

- No task is ticked by the person who implemented it. Before any tick, a
  reviewer subagent with fresh context gets the task entry and the diff — never
  the implementer's narrative, because a narrative reviews itself.
- The reviewer answers three questions: does the recorded evidence support the
  tick, is every new check shown capable of failing, and does the diff exceed
  the entry's stated scope? It returns APPROVE or REJECT with the gap.
- A rejection goes back to the implementer. Two rejections on one task escalate
  to a PROPOSED item in the session report. The verdict is recorded in the task
  entry beside the tick.

## Dogfood gate

Adopted on advisor recommendation, M4 Phase 2 open, as a dry run that gated
nothing. **Standing from M5 as a milestone-exit gate** — advisor
recommendation, James-approved at the M4 close.

- kicli's end user is an LLM agent, so an LLM agent tests it. A dogfood subagent
  gets AGENT.md, the built binary and a short design brief — no source, no task
  files, no spec — and attempts the brief cold.
- Everything it fumbles is a defect: a command misused, a document misread,
  output that overflows or confuses its context. Defects are recorded verbatim
  in tasks/dogfood.md, then triaged like any finding — fixed, PROPOSED, or
  recorded with the reason it stands.
- **A milestone that ships agent-facing commands is not done until a run has
  attempted them cold and its defect list is triaged.** One run per milestone
  minimum. The first run found nine defects, one of which (no read-only way to
  ask where a pin is) is a design item no gate this project already had would
  have surfaced — which is the argument for the gate.
- An occasional **haiku-model run is permitted as a stress variant**: a weaker
  reader finds documentation that only works for a strong one.
- **The brief-writer owns brief ambiguity.** An ambiguous brief spends the run
  on the brief rather than on the tool; run 1's defect 7 is that lesson and it
  cost a ninth of the run.

## The agentic layer

Adopted on advisor recommendation via PR, M4 Phase 2.

- A rule lives where its scope is. Role-scoped rules live in that role's
  `.claude/agents/` definition. Cross-agent policy lives here. Repeated
  procedures with steps and worked examples live in `.claude/skills/`.
  Mechanical prohibitions live in hooks. A new incident adds a worked example
  to the relevant skill, not a rule to this file, unless the lesson is
  genuinely cross-agent policy.
- Agent definitions, skills, hooks and `.claude/settings.json` are
  version-controlled working practice, changed only via ruling — like any
  other governing document.
- Implementation is dispatched to the `lane-implementer` agent; ticks go
  through the `tick-reviewer` agent; dogfood runs use the `dogfooder` agent
  with a sandbox directory prepared outside the repo (tool lists restrict
  tools, not paths — the dogfooder's isolation is sandbox-plus-instruction,
  and its transcript is spot-checkable); check-guarded chores go to
  `chore-runner`.
- If an implementer disputes a REJECT, the orchestrator re-runs the review
  once with the main model before the two-rejection escalation counts.
- **A direct instruction from James in a session IS a ruling.** Record it with
  that provenance where it lands, in the entry or document it governs, at the
  time it is given. The advisor reviews rulings for conflict with recorded
  principle and raises any conflict to James; the advisor does not reverse him.
- The frozen surface is **guarded, not enforced**, by a PreToolUse hook over
  `.claude/hooks/frozen-paths.txt`. Lifting a freeze IS the ruling path; the
  orchestrator is not exempt. This is the only tool hook; further hooks
  require a triggering incident.

  **"Guarded" is the honest word and the change is deliberate.** Promoted from
  PROPOSED 1 at the M5 opening. `.claude/settings.json` matches
  `Edit|MultiEdit|Write`; **Bash is not matched**, so a change made with `sed`,
  `python3` or a heredoc is never seen. The mechanism is an assistance against
  the accidental `Edit`, and calling it enforcement claimed a door that is open.
  Nothing improper had happened — the gap was invisible because it never bit,
  which is exactly why it was worth reporting. The alternative honest position,
  extending the matcher to `Bash` with command inspection, **remains open and is
  real work with false negatives of its own**; this wording change is the cheap,
  reversible half and is labelled PROPOSED-promoted rather than final.

  **The rule the hook cannot enforce is enforced by ownership instead**: the
  lift is the orchestrator's step in the main checkout, and the hook's
  main-checkout list resolution is privilege separation rather than a bug. See
  `tasks/M5/PLAN.md` question 2.
- **The freeze lift is the orchestrator's, in the main checkout** — lift before
  dispatch, restore after merge, both committed with the ruling's provenance.
  Ruling: James, M5 plan review, question 2. A lane cannot perform a lift: the
  hook resolves its list relative to the main checkout, so the lift a lane
  commits is invisible to the hook that enforces it. Measured by `lane-o1`,
  which parked the task rather than working around the hook. **This supersedes,
  on this one point only, the M4-close ruling's "all of it is one commit"** —
  two actors in two trees cannot share a commit. The lift commit touches
  `frozen-paths.txt` and nothing else, and the restore is the first commit after
  the merge.
- Subagents do not spawn subagents (`.claude/settings.json` caps depth at 1,
  concurrency at 4): every layer is a summarisation hop that loses evidence.
- The gates run as a git pre-commit hook. One-time setup per checkout:
  `git config core.hooksPath .githooks` (worktrees share it).
- Workflow-retrospective findings are triaged by the advisor at each session
  stop into amendments to agent definitions and skills. Milestone
  retrospectives report four counts: reversed PROPOSED items, re-litigated
  decisions, gate failures found after a tick, and BLOCKED items that were
  decidable from the record.
