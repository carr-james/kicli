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
- The BLOCKED rule applies inside lanes: a subagent that hits a governing-
  document conflict parks it and reports to the orchestrator; the orchestrator
  parks it for James.
