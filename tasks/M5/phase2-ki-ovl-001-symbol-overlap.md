# `KI-OVL-001` — symbol bodies overlap (Phase 2, lane A)

**Provenance: `tasks/M5/PLAN.md` Phase 2, RATIFIED at the M5 plan review.**
**Depends on T1, T3, T4.**

## The rule, from `research/style-rules.md` §4

> **Detect**: for all pairs `s≠t`: `body(s) ∩ body(t) ≠ ∅` (exact box
> intersection, ≥ 1 IU). Power symbols included.

**Why it blocks**: unreadable, and usually a placement bug. The north star's
second sentence is the argument — two symbols drawn on top of each other is the
paradigm case of a drawing that cannot be read.

## The one thing this rule's provenance says, and it should be in the entry

**`lane-t5` measured this rule as resting on nothing published.** Its nearest
source is Sutherland §10, *"Don't try to squash things into the smallest
possible space"*, which is not this rule.

That is legitimate — **a rule with no published source is a rule this project
chose to have** — but it is currently *undeclared*, and its **Tier 1 has no
source support at all**, which is a claim that a drawing is unshippable.
**`RULES.md` says the north star is the sentence a tier is argued from.** Argue
it, in three sentences, in this entry. If you cannot, that is a finding worth
more than the rule.

## Goal state, as the checks that prove it

### 1. Exact integer box intersection, ≥ 1 IU

Constitution §4. **Touching is not overlapping**: boxes that share an edge
exactly must **not** fire. That boundary is the whole rule and it is one
`>` versus `>=`.

**Write the touching case as a check.** It is the single most likely defect and
it is invisible to any fixture built by eye, because a one-IU gap and a one-IU
overlap look identical on screen.

### 2. `body(s)` is the BODY box, not the full box

`spec/SPEC.md` §8's two-box model. The body box excludes field text. **`KI-TXT-001`
is the rule that uses the full box** — and it is being written in this same lane,
so the two boxes will be within arm's reach of each other. **Getting these
crossed makes both rules wrong in ways that look plausible.** State in the entry
which box you used and where it comes from.

### 3. Power symbols are included

Stated in the catalogue and easy to "fix" away, because power symbols cluster.
**A check asserts they are included**, so a later reader cannot quietly exempt
them.

### 4. The `--allow` list, not a soft rule

The catalogue: *"deliberately overlapping decorative symbols are rare enough to
justify an explicit `--allow` list rather than a soft rule."* **The design
decision is already made — implement it, do not re-open it.** A soft rule would
put a tolerance in a Tier 1 gate, which is exactly the thing that cannot be
argued from the north star.

### 5. Saturation declared

This rule counts **symbols**. Declare the denominator and the fraction per T4.

## Falsification obligation

- **Overlap fires; touching does not; a 1-IU gap does not.** Three fixtures,
  three checks, and the second and third are the ones that matter.
- **A rotated symbol's body box is the trap.** A symbol at 90° whose box is
  computed unrotated overlaps things it does not touch. **Build one rotated
  overlapping pair and one rotated non-overlapping pair**, and say what you
  found. `edit::symbol` already knows how transforms work — read it, do not
  re-derive it.
- **Degenerate-fixture warning:** a fixture where every pair overlaps proves
  nothing about pair selection. Include a sheet with three symbols where exactly
  one pair overlaps, and assert **which** pair is named.

## Scope

**IN**
- `crates/kicli/src/lint/rules/overlap.rs` — **new file, no edit to any existing
  file.** If you need to edit one, stop and report — that is a finding about the
  seam.
- new test files and new fixtures under `crates/kicli/tests/`
- this file, for the evidence, written AS YOU WORK

**MERGE HOTSPOTS — report, do not edit.** As `phase2-ki-grid-001-off-grid.md`.

**OUT** — every other rule file, every other entry.

**If this scope proves wrong, the named goal state and its checks win.** Say so
in your first paragraph.

## Completion check

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo xtask check
cargo test -p kicli --test rule_files_are_formatted
```

The second is named because **`cargo fmt --check` cannot see your rule file** —
`rustfmt` does not follow the generated registry's `include!`.
