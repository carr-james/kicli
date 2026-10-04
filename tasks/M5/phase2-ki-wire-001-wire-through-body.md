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

---

# Evidence and deliverable (lane wire)

**Scope as taken.** No deviation. One new file
`crates/kicli/src/lint/rules/wire_body.rs`, one new test file
`crates/kicli/tests/lint_wire_through_body.rs`, and this section. **No existing
file was edited, and no fixture was committed**, so there is no `MANIFEST` line
to add — see "Why no committed fixture" below. The seam held: the rule
registered itself with no edit to `lib.rs`, `registry.rs`, `Cargo.toml` or
`build.rs`.

**Base.** `25aea90`, clean, verified as the first action.

## What was built

| | |
|---|---|
| rule | `crates/kicli/src/lint/rules/wire_body.rs` — `WireCrossesBody`, `KI-WIRE-001`, Tier 1 |
| checks | 13 in-file unit checks; 6 in `crates/kicli/tests/lint_wire_through_body.rs`, one of them `kicad-cli`-gated |
| the seam | **held**. The build registered the rule from the directory; no existing file was edited. `cargo test -p kicli --test lint_rules_register_from_their_own_files` green, and the generated `lint_rules.rs` lists `wire_body` beside `pin_on_wire`. |

## Goal state 1 — the exclusion, and the check that proves it exists

**The check is `a_wire_ending_on_an_inset_pin_is_not_a_crossing` (in-file) and
`a_wire_that_terminates_on_a_pin_is_not_a_crossing` (integration).**

The check is only worth anything if the clipped part has **real length** at a
legitimate pin, and that needed a symbol built for it. **Measured, and it
changes how the brief's claim should be read:**

> A pin's connection point lies **on** the body box boundary whenever it is the
> outermost contributor along its own axis — and because `symbol_boxes` builds
> the body box as the union of the graphics **and the pin segments**, that is
> the ordinary case. A wire running to such a pin therefore clips to a **single
> point**, and the *length* half of the rule excuses it. The exclusion is never
> exercised.

So "without the exclusion this rule fires on every correctly drawn schematic"
is **not literally true of this box definition and this strict `> 0` test**, and
saying so is cheaper than letting a reviewer discover it. Two confirmations
that it is the box rather than the exclusion doing the work in the symmetric
case:

- `lint_rules_register_from_their_own_files`'s specimen drawing has seven wires
  running into `PAIR` symbols whose pins are all extremal. **It reports
  nothing**, and that test was already green before and after this rule landed.
- the in-file check `a_wire_that_touches_the_body_at_one_point_is_not_a_crossing`
  shows the same arithmetic directly.

**The exclusion is load-bearing for the asymmetric family**, which is real and
not rare: a symbol with two pins on one side at **different insets**. The outer
pin sets the box edge; the inner one sits 5.08 mm inside it, and a wire ending
on the inner pin clips to 5.08 mm of positive length. The test symbol `INSET` is
built to be exactly that, and it is documented in the test file's header with a
diagram. Without the proximity half, that correct drawing is a Tier 1 block.

**Both halves are therefore independently falsifiable**, which is the design
point and is the reason the fixtures are shaped the way they are: break B1
(exclusion removed) is caught only by the inset-pin cases; break B2 (length test
removed) is caught only by the corner graze. Neither break hides behind the
other.

## Goal state 2 — integer geometry

No floating point. `cargo test -p kicli --test the_linter_holds_no_floating_point`
green, and that sweep exempts nothing and reads rule files.

The clip is **Liang–Barsky with the parameter comparisons as cross
multiplications**. The clipped endpoints are **rational** where a diagonal wire
meets the box, and they are never evaluated: `near()` multiplies the offset
through by the denominator and compares squared distances in `i128`. Two
in-file checks measure the exactness rather than assuming it —
`the_clip_is_exact_on_a_diagonal_that_divides_by_nothing_whole` (dx 3, dy 7) and
`a_rational_endpoint_is_measured_without_being_evaluated` (an entry point at two
sevenths along).

### What `geometry/` owes, reported and not done

`crates/kicli/src/geometry/` carries `Rect` (`contains`, `union`, `inflate`,
`corners`, `transformed`, `centre`) and `grid::on_segment`. It carries **no
segment/box clip**, and — relevant to the parallel lane — **no rect/rect
intersection** either.

**Nothing was duplicated: there was no copy to duplicate.** The clip is new
code. It is private to `wire_body.rs` because `geometry/` is out of this lane's
scope and adding to it is a shared-file edit, which `tasks/M5/RULES.md` and this
entry's own scope list make the orchestrator's to sequence.

**What should move, named precisely:** `Along` (the positive-denominator
rational), `clip()` and `near()` — 61 lines at the bottom of `wire_body.rs`,
plus the seven in-file checks that measure them. If `KI-OVL-001` lands needing
rect/rect intersection, `geometry/` owes two primitives rather than one and this
file's three items should go with the pair.

## Goal state 3 — which box, and where it comes from

The **body** box: `geometry::symbol_box::SymbolBoxes::body`, documented there as
*"the graphics and the pins and no text"*. `spec/SPEC.md` §8's two-box model.
`KI-TXT-001` uses `full`; this rule and `KI-OVL-001` use `body`.

**Measured rather than asserted.** Break B4 swaps `.body` for `.full` and the
legitimate-connection check fails, because the probe's visible fields sit at an
absolute `(at 0 0)` and the full box therefore reaches the page origin. A rule
written against the full box would block the drawing; this one does not.

The body box is also independently calibrated, which is what makes every "no
finding" check in this task non-vacuous:
`the_body_box_this_rule_reads_is_where_the_library_puts_it` asserts
`symbol_boxes(...).body` against a box derived **by hand** from the library
coordinates the test file writes, `86.36,96.52..109.22,106.68`, and the three
pin positions the same way. Neither side of that comparison is the rule.

## Goal state 4 — saturation declared

```rust
fn saturation(&self) -> Saturation {
    Saturation::of(Counted::Wires)
}
```

**Denominator: the sheet's wire segments. Share: 1/2, the provisional standard
from `gate.rs`'s `SHARE`.**

The declaration is **exactly true**, which is why it is made rather than
defaulted: the rule reports **at most once per wire** — one finding per wire,
naming the first body it crosses in file order, because the repair is one
re-route and Constitution §6 forbids a view that floods. So the count and the
denominator are the same objects and `count <= total` always. A sheet on which
every wire crosses a body gives the whole share, which is the `per_wire` shape
`tasks/M5/RULES.md` records the property for.

**It changes no outcome today, and that is asserted rather than claimed.**
`the_gate_fails_on_the_first_crossing_whatever_share_was_declared` builds two
drawings — one wire of four crossing (well under half) and two of two (the whole
share) — and shows `Gate::of` returning `Blocker::Blocking` in **both**, because
`Gate::of` never asks a Tier 1 rule what share it covered.

**Why not `Saturation::NEVER`, which is what `KI-CONN-001` declared.** That
author's reasoning holds and is agreed with: for a blocking rule the declaration
is inert. The difference is what the two rules count. `KI-CONN-001` counts **pin
connection points**, which `Counted` cannot name, so a denominator there would
have been a **false** declaration and `NEVER` was the honest one. Here `Wires`
is exactly right, so `NEVER` would be the false one — and the declaration
becomes live the moment a tier review moves this rule, which Phase 3 is
chartered to do.

## The tier argument, from the north star

**Required because `tasks/M5/RULES.md` requires it, and the provenance is
thinner than the rule's text suggests. The gap first:**

- `lane-t5` measured this rule's support as **partial**. The cited support is
  Sutherland §8, *"Route out of pins, not across pins"*. **That sentence is
  about pins, not bodies.** It supports the sub-case where the crossing wire is
  drawn over the symbol's other pins; it says nothing about a wire across body
  graphics away from any pin.
- **The TIER has no source support at all** — one of the 24 of 28
  `tasks/M5/RULES.md` records. Stated plainly: **no published source in this
  catalogue says a wire across a symbol body makes a drawing unshippable.** What
  follows is an argument, not a citation.

**The north star**: *"The tool must validate the important aspects of quality
schematics. It must never reward a schematic that is impossible to read and
understand."*

1. **The second sentence is about reading, and this defect attacks reading in
   two directions at once.** It *adds* a connection that is not there — the wire
   visually touches the symbol's outline and runs over its pins, so a reader
   taking the drawing at face value infers a merge the netlist does not have.
   And it *removes* information that is there — the wire is drawn over pins, pin
   numbers and pin names, so the reader cannot see what the symbol's pins are.
   A symbol whose pins cannot be read cannot be checked at all.
2. So **the drawing means something other than what it shows**, which
   `spec/SPEC.md` D3 names as the class motivating kicli. "Impossible to read
   and understand" is met in the strong sense: not ugly, but **actively
   misleading**. It is the same family as `KI-CONN-001`, which is Tier 1 and
   whose tier *does* have published support (Lathrop on the junction dot:
   *"It's a rule. We don't care whether you think it's silly."*).
3. **Tier 2 would defeat the north star's second sentence by arithmetic.**
   A Tier 2 rule is scored and normalised, and `tasks/M5/RULES.md` records the
   measurement: a sheet where every wire crosses another **scores 67 at ten
   wires and at ten thousand**. A sheet whose wires are drawn through its
   symbols would score well and pass. The BLOCKED 3 saturation property is a
   backstop for that, but it is a *share* test — and **one wire drawn through
   one IC already makes that IC unreadable**. Tier 1 is the mechanism that says
   one is enough.

**Where the argument is weakest, stated rather than hidden.** The published
detection cannot tell "crosses a body and hides a pin" from "crosses a body
through empty graphics". It blocks both, and only the first is covered by (1)'s
first limb. A reviewer who wants Tier 1 to rest **only** on the misleading case
would need a narrower detection — one that requires the clipped part to overlay
a pin or its text — and that is a different rule. **Filed as PROPOSED 2 below
rather than decided here**, because narrowing a Tier 1 detection is a
value-level judgement and `tasks/M5/RULES.md` parks those.

## Decisions recorded, each with its check

| Case | Decision | Check |
|---|---|---|
| wire grazes the body **corner** (clipped length zero) | **no finding** — the published rule says *length > 0*, and a single touching point is not a crossing | `a_wire_that_touches_the_body_at_one_point_is_not_a_crossing` (in-file), `a_wire_that_touches_the_body_at_one_point_only_is_not_a_crossing` (integration, through a real corner at `109.22,106.68`) |
| wire ends **exactly on** a pin that sits on the box edge | **no finding** — and both halves of the rule are at their limits there: the clipped part is a single point **and** that point is the pin | the same two checks, plus `a_wire_ending_on_an_inset_pin_is_not_a_crossing` for the contrast |
| wire runs **along** the body box edge | **finding** — the box is a bounding box, so this is a wire laid over the symbol's outline or along its pin stubs; the published arithmetic reports it and that is the right answer | `a_wire_along_the_body_edge_is_a_crossing` |
| wire of **zero length** inside a body | **no finding** — nothing to clip. Needs its own guard: `clip` of a degenerate segment returns the whole parameter range, so the arithmetic alone would report it | `a_wire_of_no_length_is_never_a_crossing` (break B3 is caught **only** by this check) |
| a **bus** across a body | **no finding** — the published knob is `wire.through_symbol` and the rule is named for a wire | the crossing drawing carries a bus clean through the body and the count stays at one; break B10 (read buses as wires) is caught there |
| wire crosses **several** bodies | **one finding**, naming the first in file order, with *"and N others"* | the crossing check asserts the message says nothing about others when it crosses one |
| **hidden** pins in the exclusion list | **kept** — *"a pin of `s`"*, and a hidden power pin connects. It can suppress a crossing, which is a missed finding and never a false one | documented in `bodies_of`; not covered by a check, and that is recorded below |

## PROPOSED items

**PROPOSED 1 — the formal restatement is weaker than the sentence it
formalises, and it is a false negative on a real drawing.**

The informal half of the published rule excludes *"the ≤ 1 G **stub** at each
end"*. The formal half discards the **finding** when either endpoint of the
clipped part is within 1 IU of a pin. Those are different: a wire that enters at
a pin and leaves through the **far side** of the body has one endpoint at a pin,
so the formal rule reports nothing about a wire drawn straight across the whole
symbol. Implemented as published, and the gap is made **observable** rather than
left to be discovered: `the_known_under_report_is_observed_rather_than_unobserved`
asserts the silence, so when the formula is narrowed that check goes red and
names the decision. The narrowing would be to exclude the stub rather than the
finding — require the clipped part, **after** the near-pin stub is taken off
each end, still to have positive length.

**PROPOSED 2 — Tier 1 blocks the aesthetic case along with the misleading one.**
As the tier argument's weakness above. A narrower detection would require the
clipped part to overlay a pin or its text.

**PROPOSED 3 — `geometry/` owes the segment/box clip**, as the section above
names it. Not a defect in this file; a placement question only the orchestrator
can sequence.

## The oracle, and why this rule still has one

The `oracle-check` skill scopes the ask-KiCad procedure to **connectivity-
touching** work. This rule reads no partition — `Drawing::nets()` is never
called — so it touches no connectivity and the skill does not require one.

**It has one anyway, and it is the one that matters here.** The exclusion is
measured against pin **positions**, and every other check takes those from
kicli's own geometry module — the same module the rule calls.
`kicad_puts_the_pins_where_the_exclusion_expects_them` runs
`kicad-cli sch erc` over the probe drawing and compares KiCad's reported pin
positions with the **hand-derived** constants in the test file. Neither side is
kicli's.

**Measured, green**: under `KICLI_TEST_KICAD_CLI=1` with
`/opt/homebrew/bin/kicad-cli`, KiCad reports `U1` pins 1/2/3 at exactly
`91.44,99.06`, `86.36,104.14` and `109.22,101.6`. Per `CLAUDE.md` this
environment-gated run **does not count toward done from inside a lane
worktree** — it was run to make the measurement the task owes.

## Why no committed fixture, and therefore no `MANIFEST` line

Every drawing is built by the probe harness. The `falsification-control` skill:
*"build fixtures through the harness wherever a real request can reach the
behaviour"* — a hand-built fixture encodes the same assumptions as the code that
reads it. The harness reaches all of this behaviour, and it also lets the symbol
be shaped for the one case that matters (`INSET`'s two different pin insets),
which no committed fixture in the tree carries. So nothing was added under
`tests/fixtures/**` and `fixtures_match_manifest` needed no line.

## Falsification

*(filled from the runs — see the table below)*
