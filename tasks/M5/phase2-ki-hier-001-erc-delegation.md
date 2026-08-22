# `KI-HIER-001` — sheet pin / hierarchical label mismatch (Phase 2, lane B)

**Provenance: `tasks/M5/PLAN.md` Phase 2, RATIFIED at the M5 plan review.**
**Depends on T1, T3, T4 — and on T2, which builds the ERC consumption seam.
Do not dispatch this before T2 merges.**

## This task is a DELEGATION, not a detector

`research/style-rules.md` §4:

> **T** 1. **Delegated to ERC** (`hier_label_mismatch`). Listed here only so the
> catalogue is complete; kicli reports ERC's finding and gates on it.

`spec/SPEC.md` §11.1: **KiCad 10's ERC implements 47 checks and kicli's lint
engine implements none of them.** `ENGINEERING.md` and the plan's exit-criteria
table both name **"ERC layering"** as a gate that can fail on *"kicli
implementing an ERC check, or double-counting one it reports"*.

**So the deliverable is the attribution and the gate, and the thing that must be
provably absent is a detector.** If you find yourself reading sheet pins and
comparing them to hierarchical labels, **stop** — that is the failure this task
exists to prevent, and it will pass every test you write for it.

**This rule's tier is one of only four in the catalogue with source support**:
ERC's own default severity for `hier_label_mismatch` is **ERROR**. Record that —
it means kicli is agreeing with KiCad rather than asserting over it.

## Goal state, as the checks that prove it

### 1. The finding is ERC's, and says so

An agent reading the output must be able to tell **which tool found this**, and
must not be sent to kicli's source to understand a check kicli does not
implement. §14.3: kicli **relabels** ERC severities for its own output and
**never edits `.kicad_pro`** — the `kicli.toml` ERC severity mapping is a
**presentation** mapping and the docs must say so.

### 2. It gates

Tier 1. A `hier_label_mismatch` sets `gate: fail`, using T4's mechanism. **It
must not move the score** — that is T4's first direction check, and this rule is
the first real rule to exercise it.

### 3. `kicad-cli` absence is exit 6, not a silent pass

`spec/SPEC.md` §11.2: `sch score --gate` **may require `kicad-cli`, because half
of Tier 1 is ERC-owned** — and this rule is that half. §6.1: absence is a
structured error and **exit 6**.

**The dangerous behaviour is the quiet one:** a `--gate` run with no `kicad-cli`
that reports `gate: pass` has told an agent its build is fine **on the strength
of a check that never ran.** That is this session's `xtask` finding in a
different costume, and it is worse here because it is the shipped product.
**Write that check first.**

### 4. No double-counting, and no second detector

Use T2's mechanism. **Do not build a second one.** If T2's mechanism does not
reach this case, **report it rather than working around it.**

### 5. Saturation declared

Declare it per T4, and note in the entry that a delegated rule's denominator is
a genuine question — say what you chose and why.

## Falsification obligation

- **A fixture with a real mismatch fires**, and the finding is attributed to ERC.
  Build it and **confirm KiCad agrees** by running `kicad-cli sch erc` on it —
  `ENGINEERING.md`: fixture expectations are verified against KiCad, never
  hand-asserted.
- **A matching fixture does not fire** — the presence control for an absence
  claim.
- **The absent-`kicad-cli` path is shown**, and it must be distinguishable from
  "present and broken": those are **different exit codes in §6.2's table and the
  same in a careless test.**
- **The no-detector claim needs a check that can fail.** This is the hard one and
  it is the point of the task: state how you would know if a future edit added a
  detector here. The `ERC layering` gate in the plan's exit-criteria table is
  where that check belongs. **If you cannot make it fail on anything, say so
  plainly** — a gate that cannot fail is worse than none, because it spends the
  credibility of a real one.

## Environment gate — what "done" means here

**This task's real checks need `kicad-cli`.** `CLAUDE.md`: environment-gated
checks in a lane worktree **never count toward done**; only the orchestrator's
merged run does. A lane may still run them to MAKE a measurement, and this task
owes several. **Say which checks ran in the lane, with what `kicad-cli` version,
and which wait on the merged run. Do not report an environment-gated green as
done.**

## Scope

**IN**
- `crates/kicli/src/lint/rules/hier.rs` — **new file, no edit to any existing
  file**
- new test files and new fixtures under `crates/kicli/tests/`, **plus the
  `MANIFEST` line for each fixture** — that line and nothing else
- this file, for the evidence, written AS YOU WORK

**MERGE HOTSPOTS — report, do not edit.** As `phase2-ki-conn-001-pin-on-wire.md`.

**OUT** — every other rule file, `crates/kicli/src/kicad/**` (T2's, and merged
before you start — **read it, do not change it**), every other entry.

**If this scope proves wrong, the named goal state and its checks win.** Say so
in your first paragraph.

## Completion check

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo xtask check
cargo test -p kicli --test rule_files_are_formatted
cargo test -p kicli --test command_kicad_gateway
```

plus the ERC-dependent checks by name, with the `kicad-cli` version recorded and
each shown capable of failing.
