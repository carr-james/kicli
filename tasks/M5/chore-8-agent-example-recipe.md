# The `AGENT.md` examples have no recipe (chore 8)

**Provenance: PROPOSED 8 of the M5 opening report, raised by `lane-o3`,
promoted by James's ratification and advisor rulings, M5 plan review, item 7.**

**Scheduled: before Phase 2.** `sch score` is a new agent-facing command surface
and M5 will touch `AGENT.md` heavily. Until this lands, the measured-examples
rule is satisfiable only by whoever still has the scaffolding.

## The finding

M5's `RULES.md` requires every `AGENT.md` example block a session touches to be
**regenerated from a real run of the built binary**. `opening-3` built the check
that enforces it and regenerated five blocks — and every one of those blocks
stands on drawings built by a **throwaway crate outside the repository**.

So the examples are now *measured* and **not reproducible**. The next session
that touches one of those blocks cannot regenerate it without rebuilding the
scaffolding from scratch. **The rule arrived with a cost attached, on its first
application**, and that cost is a project where the rule assumes a command.

## Goal state, as the check that proves it

**One command, runnable from a clean checkout, reproduces the drawings the
`AGENT.md` example blocks are measured from.** The check is that the command
exists, runs in CI's reach, and that its output feeds the same comparison
`opening-3`'s check already makes.

**The likely home is the existing probe harness, not a new one.** The repository
already has one — `crates/kicli/tests/probe_harness_has_one_home.rs` and
`probe_crate_is_dev_only.rs` guard exactly that, so a second scaffolding route
would break a check that already exists. **Read those two files first**; if the
harness will not carry this, that is a finding and this chore stops, because a
second home for probe drawings is the thing the project has already ruled
against.

## Where this stops being a chore

**If the answer requires choosing what an example *should* show** — which
drawing, which command, which flags — that is design, and this stops being a
chore and becomes a PROPOSED item for the orchestrator. The chore is *making the
existing five blocks reproducible*, not deciding what a sixth should contain.

## Falsification obligation

Per `.claude/skills/falsification-control/SKILL.md`, and this one has a specific
trap: a recipe that regenerates a drawing which then produces *different* output
from what `AGENT.md` shows is a **failure**, not a discrepancy to reconcile by
editing the document. Show the recipe capable of catching that: perturb one
drawing, confirm the comparison fails, restore.

**And the boundary already stated in `opening-3`'s rustdoc still holds**: the
check enforces *agreement*, not *provenance*. A hand-written example that
happens to match still passes. This chore narrows that gap by making the
provenance cheap to re-establish; it does not close it, and claiming otherwise
in the entry would be the same overclaim.

## Scope

**IN**
- the probe harness's own files, per what those two guard tests permit
- `crates/kicli/tests/agent_doc.rs` — only if the recipe must be reachable from
  the existing check
- this file, for the evidence

**OUT** — `AGENT.md` itself. **This chore changes no example text.** If a block
turns out to be wrong, that is a finding to report, not a fix to make here: the
document is a merge hotspot held by one lane at a time.

**If the enumeration above proves wrong, the named goal state wins over the
list.** Say so in your first paragraph and name what you touched and why.

## Completion check

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo test -p kicli --test agent_doc
cargo test -p kicli --test probe_harness_has_one_home
cargo test -p kicli --test probe_crate_is_dev_only
```

plus the recipe command itself, run from the checkout and named in this entry.
