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
