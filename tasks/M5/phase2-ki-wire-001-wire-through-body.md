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
| checks | 13 in-file unit checks; 8 in `crates/kicli/tests/lint_wire_through_body.rs`, one of them `kicad-cli`-gated |
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

**Good state committed first**, per the `falsification-control` skill: commit
`7991647`, gate green (6 of 8 arms pass; `corpus` and `kicad-cli` skip, which is
the correct bare-run output). **That commit is deliberately NOT amended** — the
two checks added after the sweep land in a second commit instead, so the SHA
this section names stays valid. The primary anchor is still the **content
hash**, because a hash survives a merge-forward and a SHA does not:

```
sha256 crates/kicli/src/lint/rules/wire_body.rs
  8dbe2ddce57bcf81c77817d4b994a6c5f87dbac2e6b4bc8d269c9ac01f4144bd
```

Every break was applied to the rule file, every run used `--no-fail-fast`, and
after every break the file was restored with `git checkout --` on **that one
pathspec** and the sha256 re-compared with the good state. **All fifteen
restores matched.** Raw logs: `/tmp/kicli-scratch/wire/falsification/`.

### The target set, and why it is narrower than the whole suite — measured

`cargo test -p kicli --no-fail-fast` runs **659 checks** and, under the
contention measured during this lane (three other lanes running their own full
suites concurrently; `ps` showed six concurrent `cargo test` invocations), one
pass took over twenty minutes. So thirteen targets were used instead of fifty:

- derived, not chosen: `grep -l "kicli::lint" crates/kicli/tests/*.rs` ∪
  `grep -l "src/lint" crates/kicli/tests/*.rs`, plus `--lib`. A target that
  neither names the lint engine nor reads `src/lint` as text cannot observe a
  change in a lint rule. The derivation is in
  `/tmp/kicli-scratch/wire/lint-targets.txt` and is re-runnable;
- **validated by measurement, not by that argument.** Break B1 was run **twice**
  — once over all 659 checks and once over the narrowed 260 — and the catcher
  lists are **identical, six names each**. The narrowing loses nothing on the
  break that matters most.

**No break caused a compile failure.** Checked rather than assumed:
`grep -lE "could not compile|^error\[E"` over all fifteen logs matches nothing.
(The first sweep's own summary prints `COMPILE ERROR` on every row — that flag
is a **false positive** in my runner, which matched cargo's own
`error: test failed, to rerun pass …` line. The flag is wrong; the runs are
sound.)

### The table

| # | What was broken, exactly | Caught by |
|---|---|---|
| **B1** | **the proximity half, entire**: `!body.pins.iter().any(\|pin\| near(from, to, lo, *pin) \|\| near(from, to, hi, *pin))` replaced by `true` | **6** — `a_wire_ending_on_an_inset_pin_is_not_a_crossing`, `the_exclusion_reads_this_symbols_pins_and_no_others`, `the_known_under_report_is_observed_rather_than_unobserved`, `a_wire_that_terminates_on_a_pin_is_not_a_crossing`, `a_wire_drawn_through_a_body_names_that_wire_and_that_symbol`, `the_gate_fails_on_the_first_crossing_whatever_share_was_declared`. Identical over 659 checks and over 260. |
| **B2** | **the length half**: the three lines `if !lo.is_before(hi) { return false; }` removed — the `if`, the `return` and the closing brace, named in full because one line number would misdescribe it | **2** — `a_wire_that_touches_the_body_at_one_point_is_not_a_crossing`, `a_wire_that_touches_the_body_at_one_point_only_is_not_a_crossing` |
| | **B1 and B2 have no catcher in common.** That is the independence claim of the two halves, measured rather than argued. | |
| **B3** | the zero-length guard `if from == to { return false; }` removed (three lines) | **1** — `a_wire_of_no_length_is_never_a_crossing`, and it is the **only** catcher. No fixture carries a zero-length wire, which is exactly why the unit check is there. |
| **B4** | `.body` replaced by `.full` in `bodies_of` | **3** — `text_outside_the_body_does_not_make_a_crossing`, `a_wire_drawn_through_a_body_names_that_wire_and_that_symbol`, `a_wire_across_two_bodies_is_one_finding_that_counts_the_others`. **See the measured gap below: on the first pass this break had one catcher, and that catcher was a position assertion.** |
| **B5** | the clip's sign convention: `if p < 0` became `if p > 0` in the Liang–Barsky loop — the sign-inversion control, so no check below is a restatement of the implementation | **10**, including every clip unit check and both integration crossing checks |
| **B6** | `NEAR_A_PIN` widened from `1` to `100_000` (10 mm) | **7** — including `an_endpoint_is_near_a_pin_at_one_unit_and_not_at_two` and the crossing check, which stops firing because the crossing gets excused |
| **B7** | `NEAR_A_PIN` tightened from `1` to `0` | **1** — `an_endpoint_is_near_a_pin_at_one_unit_and_not_at_two`. The boundary is at exactly one internal unit and only that check can see it, because a pin that coincides with the endpoint is excluded at either value. |
| **B8** | `Saturation::of(Counted::Wires)` replaced by `Saturation::NEVER` | **1** — `the_gate_fails_on_the_first_crossing_whatever_share_was_declared`. The declaration is asserted, not merely written. |
| **B9** | `Tier::One` replaced by `Tier::Two` | **2** — the crossing check's tier assertion and the gate check (`Blocker::Blocking` becomes `Saturated` or absent) |
| **B10** | `wires_of`'s bus filter removed: `Item::Line(line) if matches!(line.kind, LineKind::Wire)` became `Item::Line(line)` | **1** — `a_wire_drawn_through_a_body_names_that_wire_and_that_symbol`, whose drawing carries a bus clean through the body. Without that bus this break would have been green, and the gap would have been invisible. |
| **B11** | only one endpoint of the clipped part checked: `\|\| near(from, to, hi, *pin)` removed | **6**, including `every_hidden_rule_file_is_formatted` — incidental, because the break shortened a line rustfmt then wanted to rejoin. Noted so the next reader does not take it for a behavioural catcher. |
| **B12** | `let others = crossed.len() - 1;` became `let others = 0;` | **1** — `a_wire_across_two_bodies_is_one_finding_that_counts_the_others` |
| **B13** | `crossed.first()` became `crossed.last()` | **1** — the same check, on the `objects` assertion. "First in file order" is asserted rather than incidental. |

### The measured gap in my own instrument, and the repair

**B4's first run had exactly one catcher, and it was the finding's reported
position** — the box centre moved, so the `pos` assertion failed. The
behaviourally important half was **not** caught: with the full box, the three
legitimate wires are still excused, because the exclusion absorbs them. A
body-versus-full confusion that produced a *false finding* would have gone
unobserved.

That is the fourth kind of blind instrument — the check agreed for a reason
other than the one claimed — and it was found only because the break was made.
Two checks were added in response and B4 was re-run:

- `text_outside_the_body_does_not_make_a_crossing` — a wire across the symbol's
  **text** and nowhere near its body reports nothing, with a control asserting
  that the two boxes really do differ in that drawing and that the wire really
  is inside one and outside the other;
- `a_wire_across_two_bodies_is_one_finding_that_counts_the_others` — which also
  closed a second gap: the **per-wire** decision and the "first in file order"
  choice were not asserted anywhere, and B12 and B13 exist because of it.

### The environment break — the fifth dimension

My checks consume **probe-generated identifiers**: `wire_between` and
`symbol_called` take uuids from the loaded drawing and compare them with the
uuids in the finding. That is a generated value, so the second-directory run is
owed.

```sh
scratch="$(mktemp -d /tmp/kicli-scratch/wire/elsewhere-XXXXXX)"
git archive HEAD | tar -x -C "$scratch"
( cd "$scratch" && KICLI_TEST_KICAD_CLI=1 cargo test -p kicli \
    --no-fail-fast --lib lint::registry::wire_body --test lint_wire_through_body )
```

**Green from the second directory**: 13 unit checks and all integration checks,
the `kicad-cli` oracle included. Nothing here asserts a property of the
worktree it was written in.

*The scratch directory is deliberately **not** named `src`: the recorded trap is
that `the_four_way_rule_has_one_home.rs` matches any path component called
`src`, and the false failure is indistinguishable from a real regression.*

### Probe-name collision

Nine `Probe::new` names in the binary, **nine distinct**
(`calibration`, `connected`, `crossing`, `grazing`, `gate-one-of-four`,
`gate-all`, `pin-oracle`, `text-not-body`, `two-bodies`). Checked rather than
assumed, because `cargo test` runs one binary's checks in parallel and the probe
harness writes to a name-keyed path.

### What is NOT covered by a check, said plainly

- **Hidden pins in the exclusion list.** `bodies_of` keeps them, deliberately,
  and no fixture carries a hidden pin inside a body box. The effect is a
  suppressed finding, never a false one, so the gap is in the quiet direction —
  but it is a gap and not a measurement.
- **A symbol the file embeds no definition for.** Skipped by `bodies_of`'s
  `let Some(definition) = …` and not exercised.

## Completion check

```
cargo xtask check                                    # 6 pass, 2 skip (corpus, kicad-cli)
cargo test -p kicli --test rule_files_are_formatted  # 1 passed
```

Per `CLAUDE.md`, the two skipped arms are **environment- and corpus-gated and do
not count toward done from inside a lane worktree**; the `kicad-cli` oracle was
run by hand, under `KICLI_TEST_KICAD_CLI=1`, to make the measurement this task
owes, and is green.

## What landed, and the anchors

| | |
|---|---|
| base | `25aea90`, verified clean as the lane's first action |
| `7991647` | the rule, the checks and this evidence — the **good state** every break was made against and restored to |
| `febda2d` | the two checks break B4 asked for (`text_outside_the_body_does_not_make_a_crossing`, `a_wire_across_two_bodies_is_one_finding_that_counts_the_others`) and the falsification section above |
| the commit after it | this closing section, which cannot name its own hash. `git log --oneline 25aea90..` is the whole list. |
| gate | `cargo xtask check` on `febda2d`: **6 pass, 0 fail**, `corpus` and `kicad-cli` skip |

Content hashes of the final state, for evidence that survives a merge-forward:

```
sha256 crates/kicli/src/lint/rules/wire_body.rs
  8dbe2ddce57bcf81c77817d4b994a6c5f87dbac2e6b4bc8d269c9ac01f4144bd
sha256 crates/kicli/tests/lint_wire_through_body.rs
  424c60fc6a314dcf72e3e4e4c07829ae540bd3d92d35651851a3e7e266076acc
```

**Scope, re-measured at hand-off rather than re-read** (`git diff --stat
25aea90..HEAD`): three files, **1,524 insertions, 0 deletions, 0 existing files
touched**. No merge-forward happened during this lane, so no scope or state
claim above has been across one. Working tree clean.
