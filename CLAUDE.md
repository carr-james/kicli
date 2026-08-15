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
- Large builds start fresh sessions. Sessions end at task boundaries; stopping
  early to hand over a clean state beats pushing through — the task file is
  the handoff, and it is updated before the session ends, not after.

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
- Corpus-gated and environment-gated checks do not run in lane worktrees; the
  orchestrator runs the full check, corpus included, at every lane merge — not
  only at milestone end.
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

Adopted on advisor recommendation, M4 Phase 2 open. Standing from M5; a dry run
in M4, where it gates nothing.

- kicli's end user is an LLM agent, so an LLM agent tests it. A dogfood subagent
  gets AGENT.md, the built binary and a short design brief — no source, no task
  files, no spec — and attempts the brief cold.
- Everything it fumbles is a defect: a command misused, a document misread,
  output that overflows or confuses its context. Defects are recorded verbatim
  in tasks/dogfood.md, then triaged like any finding — fixed, PROPOSED, or
  recorded with the reason it stands.
