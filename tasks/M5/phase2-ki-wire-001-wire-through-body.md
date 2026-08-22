# `KI-WIRE-001` — a wire crosses a symbol body (Phase 2, lane A)

**Provenance: `tasks/M5/PLAN.md` Phase 2, RATIFIED at the M5 plan review.**
**Depends on T1, T3, T4.**

## The rule, from `research/style-rules.md` §4

> **Detect**: segment/box intersection between `seg(w)` and `body(s)`, excluding
> the ≤ 1 G stub at each end where the wire legitimately meets a pin of `s`.
> Formally: clip `seg(w)` to `body(s)`; finding if the clipped length > 0 and
> neither endpoint of the clipped part is within 1 IU of a pin of `s`.

**Why it blocks**: it **reads as a connection that isn't one**, and it hides
pins. Same family as `KI-CONN-001` — the drawing means something other than what
it shows — which is the class `spec/SPEC.md` D3 names as motivating kicli.

**Provenance note for the entry**: `lane-t5` measured this rule's support as
**partial**. Sutherland §8 says *"Route out of pins, not across pins"* — which is
about **pins**, not bodies. Declare the gap; the tier has no source support and
`RULES.md` requires it be argued from the north star.

## Goal state, as the checks that prove it

### 1. The exclusion is the rule, not a detail

**A wire that legitimately terminates on a pin necessarily enters the body box**,
because a pin sits on the body's edge and the wire runs to it. **Without the
exclusion this rule fires on every correctly drawn schematic in existence.**

So the clipped-length test and the pin-proximity test are **both** load-bearing,
and the pin-proximity half is the one a naive implementation omits. Write the
"correctly connected symbol" fixture **first** and assert **no finding**. If that
check does not exist, the rule is not implemented.

### 2. Integer geometry, exactly

Constitution §4. Clipping a segment to a box is a Liang–Barsky/Cohen–Sutherland
shape and it is **doable in integers** — the parameter comparisons become
cross-multiplications. **No floating point.** `crates/kicli/src/geometry/` already
carries the project's integer primitives; read them before writing a new one.
`ENGINEERING.md`: a second copy of a geometric primitive is a defect, not a
convenience.

### 3. `body(s)`, not the full box

As `KI-OVL-001`. §8's two-box model. State which you used.

### 4. Saturation declared

This rule counts **wires**. Declare the denominator and fraction per T4. Note
this is a `per_wire`-shaped rule and therefore one where the BLOCKED 3
arithmetic actually bites: *a sheet where every wire crosses a body*.

## Falsification obligation

- **The legitimate-connection case is shown NOT firing** — goal state 1, and it
  is worth more than every other check in this task.
- **A wire passing clean through a body fires**, and the finding names the wire
  and the symbol.
- **The boundary: a wire ending exactly ON the body edge at a pin.** Both the
  clipped length and the 1-IU proximity are at their limits there. Build it.
- **A wire that grazes the body corner** — clipped length effectively zero.
  Decide, record the decision, and check it. Do not let it fall out of the
  arithmetic unobserved.
- **Blind-instrument warning:** if your check asserts "one finding on this
  fixture", it passes when the rule fires for the *wrong reason* — e.g. because
  the exclusion is broken and a different wire fired. **Assert which wire and
  which symbol.**

## Scope

**IN**
- `crates/kicli/src/lint/rules/wire_body.rs` — **new file, no edit to any
  existing file.** Exception, and it is the likely one: if `geometry/` genuinely
  lacks a primitive you need, **say so and report it** rather than duplicating
  one into your rule file. Adding to `geometry/` is a shared-file edit and it is
  the orchestrator's to sequence.
- new test files and new fixtures under `crates/kicli/tests/`
- this file, for the evidence, written AS YOU WORK

**MERGE HOTSPOTS — report, do not edit.** As `phase2-ki-grid-001-off-grid.md`.

**OUT** — every other rule file, every other entry, `crates/kicli/src/geometry/`
(report what it owes).

**If this scope proves wrong, the named goal state and its checks win.** Say so
in your first paragraph.

## Completion check

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo xtask check
cargo test -p kicli --test rule_files_are_formatted
```
