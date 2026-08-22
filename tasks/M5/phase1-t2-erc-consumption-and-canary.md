# ERC consumption, and the 100× canary (Phase 1, T2)

**Provenance: `tasks/M5/PLAN.md` Phase 1, RATIFIED by James's ratification and
advisor rulings, M5 plan review.**

**Depends on T1.** The finding type T1 defines is what ERC's violations are
mapped *into*; writing this before that shape exists means writing it twice.

## What this task is, in one sentence

**kicli runs KiCad's ERC, translates its violations into kicli findings, and
never trusts the JSON's coordinates.** `spec/SPEC.md` §11.1: KiCad 10's ERC
implements 47 checks and **kicli's lint engine implements none of them.**

## Why the canary is the only check here that matters

`spec/SPEC.md` §14.2. KiCad 10.0.5's ERC JSON exporter builds its units provider
with `pcbIUScale` (1e6 IU/mm) instead of `schIUScale` (1e4), so schematic
coordinates come out **100× too small while labelled `"mm"`**. The text report is
correct. Root cause is `eeschema/erc/erc_report.cpp:161` vs `:63`
(`research/geometry.md` §3.5), and it is **still unfixed on `master`**.

**The canary exists to fail when KiCad changes its mind**, and that is its whole
purpose. §14.2, requirement 2, verbatim: *"A CANARY TEST that expects the bug: on
a committed fixture, assert `json.pos × 100 == text.pos` exactly. When upstream
fixes it, this test fails loudly and the workaround is removed, never
double-applied."*

**A silently-double-applied correction is the failure this task is guarding
against**, and it is a coordinate being wrong by 10,000× in a tool whose whole
job is where things are drawn. Weigh every design choice here against that.

## Goal state, as the checks that prove it

### 1. `kicad` owns the invocation. All of it.

`ENGINEERING.md`, Structure: *"`kicad` owns every invocation of an external
KiCad binary — discovery, the version check, the process seam and the exit-code
translation — because `lint`, `render` and `cli` all need it and none of them may
depend on another."* The module exists (`crates/kicli/src/kicad.rs`, with
`discovery.rs` and `runner.rs`); **ERC's invocation goes there, beside them, not
into `lint`.**

`lint` knows nothing of files on disk or `kicad-cli` (Dependency inversion). The
seam between them is data, and naming that seam well is most of this task.

### 2. The canary, per §14.2

- A **committed fixture** — not a fetched one, not a generated one. The
  measurement must be reproducible by anyone at any time, and it is asserting a
  property of a *specific* KiCad version.
- `json.pos × 100 == text.pos` **exactly**. Not within a tolerance: this is
  integer scale, and a tolerance would hide the fix when it lands.
- **The correction site carries a code comment naming `erc_report.cpp:161`**
  (§14.2, requirement 3). A future reader must be able to find the upstream line
  without finding this entry.
- §14.2 requirement 4: **no upstream bug report is filed** (Q35).

**Which report to read is a real decision and the spec leaves it open**: §14.2
requirement 1 permits either reading the text report, or applying a
sanity-checked ×100 correction to the JSON. The spec states the reason JSON is
otherwise preferable — *"each violation item carries the offending object's
UUID, which joins directly to kicli's handles"* (`research/kicad-cli.md` §3.1).
**Record which you chose and why**, because a later reader will otherwise assume
the choice was forced.

### 3. Absence of `kicad-cli` is a structured error and exit 6, not a panic

§14.1 and §6.1. The discovery order is §14.1's and already implemented; do not
re-invent it. Major version ≠ 10 is refused. **§6.2: kicli translates every one
of `kicad-cli`'s exit codes; raw pass-through is forbidden** — that table is
already spec'd, and `crates/kicli/src/cli/exit.rs` already has tests over the
code table. Read them first.

### 4. Severities are read-only

§14.3. ERC severities live in `.kicad_pro` (`erc.rule_severities`) and
`kicad-cli` can only filter what is reported. **kicli relabels them for its own
output and never edits `.kicad_pro`.** `kicli.toml`'s ERC severity mapping is a
*presentation* mapping, and the docs must say so — that sentence is §14.3's, and
it is a documentation obligation as much as a code one.

### 5. The two deliberate ERC exceptions are attributed, not duplicated

§11.1: `four_way_junction` → `KI-JCT-001` and `single_global_label` →
`KI-LBL-003`, because KiCad's default severity for both is `IGNORE` and an
untouched project would silently pass. **kicli attributes them clearly and does
not double-count when the project has ERC's version enabled.**

The rules themselves are Phase 3's. **What T2 owes is the mechanism that makes
non-double-counting possible** — and a check that it works, because "does not
double-count" is precisely the kind of claim that is true until someone enables
a severity.

## Falsification obligation

Per `.claude/skills/falsification-control/SKILL.md`.

- **The canary is falsified by feeding it a correctly-scaled report and watching
  it fail.** Construct one; do not wait for KiCad to fix the bug. A canary never
  shown to fail is a canary nobody has checked is alive.
- **The exit-6 path is falsified by an absent binary**, and the check must
  distinguish "absent" from "present and broken" — they are different exit codes
  in §6.2's table and the same in a careless test.
- **The double-count check is a degenerate-equality candidate**: if both the ERC
  finding and the kicli finding are derived from the same mapping call, a break
  moves them together. State what the two sides derive from.

## Environment gate — and what that means for "done"

**This task's real checks need `kicad-cli`, so a lane cannot complete it alone.**
`CLAUDE.md`: corpus- and environment-gated checks in a lane worktree never count
toward done; only the orchestrator's merged run does. A lane may still run them
to MAKE a measurement its task owes — and this task owes several.

**Say in the entry which checks ran in the lane, with what `kicad-cli` version,
and which are waiting on the merged run.** Do not report an environment-gated
green as done.

## Scope

**IN**
- `crates/kicli/src/kicad.rs` and new files under `crates/kicli/src/kicad/`
- `crates/kicli/src/lint/` — only the seam that consumes ERC findings
- new test files under `crates/kicli/tests/`
- `crates/kicli/tests/fixtures/**` — new fixtures only
- this file, for the evidence, written AS YOU WORK

**MERGE HOTSPOTS — report, do not edit.** `Cargo.toml`, `crates/kicli/src/lib.rs`,
the fixture `MANIFEST`, `AGENT.md`, `spec/SPEC.md`, `crates/kicli/tests/command_surface.rs`,
`kicli.toml`'s `[rules]` table.

**OUT** — every other module, every other entry, `tasks/M5/PLAN.md`.

**If the enumeration above proves wrong, the named goal state and its checks win
over the list.** Say so in your first paragraph, name what you touched and why.

## Completion check

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo xtask check
```

plus, named because they are what this task is:

```sh
cargo test -p kicli --test command_kicad_gateway
```

and the canary test by name, with its falsification recorded.
