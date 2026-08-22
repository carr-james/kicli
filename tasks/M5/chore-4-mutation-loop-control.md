# Chore — `mutation_loop.rs`'s P1/P2 assertions share an ancestor

**Provenance: PROPOSED 18, found by the round-trip lane (M4 T21) while fixing
its own, promoted by James's ruling at the M4 close.**

## The gap

The three P1/P2 lines in `crates/kicli/tests/mutation_loop.rs` compare
`is_canonical()` against `emit()`, **both routed through one `prettify`**, so a
break in the prettifier moves claim and control together and the check cannot
see it. This is the **shared-ancestor** kind
(`.claude/skills/falsification-control/SKILL.md`). T21 recorded it rather than
fixing it, being another file.

**This is not hypothetical**: T21 demonstrated a genuine P1 violation that its
own uncorrected check passed.

## The chore

Give it the control T21 built: compare against `kicad-cli sch upgrade --force`
re-saving the same file **in the same run**. An independently derived control
is the only thing that can see a break in the shared ancestor.

## Completion check

The corrected assertions, plus the falsification that proves the point: break
`prettify`, and watch the corrected check fail where the old one passed.
Environment-gated, so it counts only on the orchestrator's merged run.
