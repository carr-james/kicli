---
name: oracle-check
description: The ask-KiCad-about-the-written-file procedure. Use for any task that changes connectivity.
---

# Oracle check

Standing policy (M3, carried into M4): a check that matters asks KiCad about
the file kicli wrote, not about kicli's arithmetic. Every task that changes
connectivity carries an oracle check.

## Procedure

1. kicli writes the file through the mutation path (atomic write, invariant
   pass).
2. KiCad reads it back: `kicad-cli` netlist export on the written file.
3. Compare KiCad's netlist against the connectivity kicli claimed. Exact
   match on hermetic fixtures; the demo-corpus oracle holds at 35/35 and a
   change that moves that number is seen, not absorbed.
4. Environment-gated: oracle checks run under `KICLI_TEST_KICAD_CLI=1` and do
   not run in lane worktrees — the orchestrator runs the full check, corpus
   included, at every lane merge (CLAUDE.md, Parallel work).

## The distinction that keeps this honest

Established-from-source is not measured-against-the-tool. A rule read out of
KiCad's source (e.g. a sheet pin's angle names an edge) is recorded as such
until a probe measures it against the running tool; the entry that first
exercises it owes the probe.
