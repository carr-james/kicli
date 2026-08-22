# `KI-TXT-001` — overlapping text (Phase 2, lane A)

**Provenance: `tasks/M5/PLAN.md` Phase 2, RATIFIED at the M5 plan review.**
**Depends on T1, T3, T4.**

> **LANE-TABLE GAP, recorded rather than absorbed.** `PLAN.md`'s Phase 2 table
> lists **six** Tier 1 rules including `KI-TXT-001`, and the **lane table assigns
> only five of them**: lane A owns `grid.rs`, `overlap.rs`, `wire_body.rs`; lane
> B owns `pin_on_wire.rs`, `hier.rs`. **`text.rs` is named in neither Phase 2
> lane** — it appears only in Phase 3's lane D. **Assigned to lane A** by the
> orchestrator at checkpoint 2, because this rule stands on the geometry module
> and the two-box model, which is what lane A already is. Phase 3's lane D
> inherits and extends the file, which is safe because the phases are
> sequential. *Filed as a PROPOSED item: the lane table undercounts Phase 2 by
> one rule, and an enumeration that undercounts is the exact defect the M4 handle
> chore's ruling is about.*

## The rule, from `research/style-rules.md` §4

> **Detect**: for all pairs of visible text objects `(a,b)` (fields, labels, free
> text, sheet names, pin names/numbers): **oriented-box** intersection with area
> **> 20 %** of `min(area(a), area(b))`.
>
> **Note**: uses oriented boxes, not AABBs — schematic text is routinely at 90°.

**Why it blocks, and why it is the rule with the strongest claim to Tier 1 of
any in this phase:** *"this is the failure mode that motivated kicli"* —
`spec/SPEC.md` D3. Two strings drawn on top of each other cannot be read, which
is the north star's second sentence stated as a geometry problem.

**Sources**, measured by `lane-t5`: Sutherland §10 *"Don't run wires through
text"* plus his worked example *"R4's reference designator overlaps the `FB_OUT`
net label"*; Lathrop *"doesn't collide with other parts of the drawing"*. **This
rule's existence is well supported. Its tier is not** — argue it from the north
star in this entry, per `RULES.md`.

## Goal state, as the checks that prove it

### 1. Oriented boxes. Not AABBs. This is the whole task.

An axis-aligned approximation of a 90° label is **wrong in both directions**: it
reports overlaps that do not exist and misses ones that do. **Schematic text is
routinely at 90°**, so the wrong version fails on ordinary drawings rather than
on exotic ones.

**Build a fixture with a 90° label adjacent to a horizontal one, positioned so
that the AABBs intersect and the oriented boxes do not.** Assert **no finding**.
That single check is the difference between this rule and a plausible-looking
thing that is not this rule.

### 2. The 20 % ratio, in integers

Constitution §4 — *"detection is integer geometry only"*. An area ratio is a
division, and **it must not become floating point**: compare
`20 · area(∩) > 20 % · …` by cross-multiplication —
`100 · area(∩) > 20 · min(area(a), area(b))` — which is exact.

Oriented-box intersection area in integers is the hard part of this task.
**`crates/kicli/src/geometry/` is where the project's integer primitives live.
Read it first.** If a primitive is genuinely missing, **report it — do not
duplicate one into the rule file.**

### 3. The FULL box, and this is where the lane's two rules must not cross

`spec/SPEC.md` §8's two-box model. **This rule uses the full box (∪ visible field
boxes); `KI-OVL-001`, in this same lane, uses the body box.** Getting them
crossed makes both rules wrong in ways that look plausible on a screenshot.
**State which box you used and where it comes from**, in both entries.

### 4. "Visible" is load-bearing

A hidden field has no drawn text and **cannot overlap anything**. `edit::field`
already knows how visibility is stored (and that older formats hide the flag
inside the effects). **A fixture with two hidden fields at identical positions
must produce no finding.** This is an absence check and therefore carries a
presence control: the same two fields, made visible, must fire.

### 5. Text metrics come from the port, not from a guess

The text-metrics table is derived by `cargo xtask text-metrics`. **A box computed
from a character count is not a box.** Use the port.

### 6. Saturation declared

This rule counts **visible text objects**. Declare its denominator and fraction
per T4.

## Falsification obligation

- **The 90°/AABB divergence check** — goal state 1. Show it failing by
  substituting an AABB implementation, then restore.
- **The ratio boundary**: a pair overlapping at exactly 20 % must not fire; at
  20 % + 1 IU² it must. That is one comparison and it is the rule's whole
  tolerance.
- **The hidden-field pair, with its presence control** — goal state 4.
- **Degenerate-fixture warning:** a sheet where every text object overlaps every
  other proves nothing about pair selection. Include a sheet where exactly one
  pair overlaps and assert **which** pair is named.

## Scope

**IN**
- `crates/kicli/src/lint/rules/text.rs` — **new file, no edit to any existing
  file.**
- new test files and new fixtures under `crates/kicli/tests/`
- **the fixture `MANIFEST` line for any fixture you commit.** `fixtures_match_manifest`
  asserts every fixture is recorded, so a fixture without its line fails your own
  check. The `MANIFEST` is otherwise a merge hotspot: **add your line and
  nothing else, and say so.**
- this file, for the evidence, written AS YOU WORK

**MERGE HOTSPOTS — report, do not edit.** `Cargo.toml`, `lib.rs`, `build.rs`,
`AGENT.md`, `spec/SPEC.md`, `tests/command_surface.rs`, `kicli.toml`'s `[rules]`.

**OUT** — every other rule file, every other entry,
`crates/kicli/src/geometry/` (report what it owes).

**If this scope proves wrong, the named goal state and its checks win.** Say so
in your first paragraph.

## Completion check

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo xtask check
cargo test -p kicli --test rule_files_are_formatted
```
