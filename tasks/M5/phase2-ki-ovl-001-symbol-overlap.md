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

---

# Evidence and deliverable (lane ovl)

**Lane `lane-ovl`, branch `lane-ovl`, base `25aea90` (verified clean at first
action, no fast-forward needed).** Written as the work was done.

## Deliverable

| File | State |
|---|---|
| `crates/kicli/src/lint/rules/overlap.rs` | **new**, the rule and its unit checks |
| `crates/kicli/tests/lint_symbol_overlap.rs` | **new**, the written-file checks |
| this file, below the brief | the evidence |

**No existing file was edited, and no fixture was committed**, so no `MANIFEST`
line is owed. Every drawing is built by the probe harness into
`CARGO_TARGET_TMPDIR`. The registration seam held exactly as T1's PASS said it
would: `build.rs` found `overlap.rs` and registered it with nothing else
touched.

## 1. The tier, argued from the north star

`lane-t5`'s measurement is accepted and declared rather than papered over: this
rule **rests on nothing published**, its nearest candidate ([S] Sutherland §10,
*"Don't try to squash things into the smallest possible space"*) is about
density and **is not this rule**, and its Tier 1 has **no source support at
all** (`tasks/M5/phase1-t5-seed-catalogue-and-ground-names.md` P1.1, Tier 1
table row 2, and its counts: one of the four rules resting on nothing). The
argument below is therefore made from `RULES.md`'s north star, which `RULES.md`
says is the sentence a tier is argued from. **The argument can be made**, and it
is three sentences:

1. A pair of overlapping bodies **hides drawing from the reader** — at the
   shared area, at least one of the two symbols' graphics or pins is not there
   to be seen — so a sheet holding one cannot be read as drawn, which is the
   north star's second sentence's subject exactly.
2. **Tier 2 cannot carry that claim**, because a normalised rule's cost is
   capped at `w · reference` however often it fires: the measurement BLOCKED 3
   recorded for crossings — *a sheet where every wire crosses another scores
   **67**, at ten wires and at ten thousand* — applies unchanged to a sheet
   whose every symbol sits on another, and a score in that band is a reward.
3. **Tier 1 is the only tier whose mechanism matches the claim**, because one
   occurrence fails the gate and the verdict never depends on how much of the
   sheet was lost; and the detection is **exact** — integer box intersection,
   no tolerance anywhere — so the gate fires only where drawing really is
   hidden and never on a sheet that is merely tight.

Sentence 3 is the one that earns the tier, and it is also the reason the
catalogue's `--allow` decision is correct and was not re-opened: a **tolerance**
inside a blocking gate would be unarguable from that sentence, because the
drawing a tolerance passes is still a drawing a reader cannot read. A **named
pair** is different in kind — an author saying *this one is on purpose*, and
saying which one. The argument is reproduced in the rule file's module header so
it travels with the code.

## 2. Which box, and the `KI-TXT-001` crossing hazard

**The BODY box**, read as `symbol_boxes(...).body`. The citation is
`spec/SPEC.md` §8, which assigns the two boxes by rule family in one sentence:
*"**body box** (graphics + pins, no text) for overlap rules, and **full box**
(∪ visible field boxes) for text-collision rules"*. `KI-TXT-001` is the sibling
that takes `full`. Crossing them would make this rule fire on two symbols whose
*reference designators* sit near each other while nothing drawn overlaps at all
— plausible and wrong. Stated in the rule file's header with the same citation.

**Rotation is not re-derived.** `crates/kicli/src/geometry/symbol_box.rs`
already applies `Transform::from_file(angle, mirror)` with the mirror composed
second, and transforms only the two corners, which is correct for the eight
orientations a symbol can take. The rule calls that primitive and adds nothing.
`a_turned_body_is_compared_where_it_is_actually_drawn` is what holds it down.

## 3. Saturation: `Saturation::NEVER`, declared with three reasons

`crates/kicli/src/lint/gate.rs` read. The declaration is `Saturation::NEVER`,
and — as `RULES.md` anticipates and `KI-CONN-001` found first — **for a
blocking rule the declaration cannot change an outcome**. Two further reasons
are specific to this rule and are reported rather than shrugged at:

- **This rule counts PAIRS, and `Counted` has no pair denominator.** A finding
  is one pair; a sheet of `n` symbols holds `n(n-1)/2` of them. A pair count
  against `Counted::Symbols` is a ratio of two different things, and can exceed
  one.
- **`Counted::Symbols` excludes objects this rule counts.**
  `Density::symbols()` is documented as *"power symbols excluded"*
  (`src/lint/score.rs:65-68`), and this rule **includes** power symbols. On a
  sheet of nothing but overlapping power symbols the total is **0**, which
  `Saturation::is_reached` reads as never-saturating anyway — so declaring
  `Counted::Symbols` would be a false declaration that happens to be inert.
  That is worse than declaring nothing. **Measured, not argued**:
  `power_symbols_are_not_exempt` asserts one finding on a sheet whose
  `Density::of(&drawing).symbols()` is `0` and whose symbol count is `2`.

**PROPOSED (lane ovl).** `Counted` needs a third symbol denominator — *all*
symbols, power included — before any rule that counts power symbols is scored.
Recommendation: leave it until a Tier 2 rule needs it; `gate.rs` is not this
lane's file and the change decides nothing today. Revisit trigger: the first
Tier 2 rule whose findings include power symbols.

## 4. The defect my own first version had, caught before any deliberate break

Recorded because it is the "plausible lookalike" the brief warned about, and
because it was found by a check rather than by reading.

The obvious form of the test is **four comparisons between the two boxes' own
corners** (`one.start.x < two.end.x && …`). It agrees with the published rule
on every pair of boxes that both have a size. **It disagrees on a box that has
none**: `symbol_boxes` falls back to `Rect::around(anchor)` for a symbol that
draws no graphics and no visible pin, and a zero-size box sitting *strictly
inside* another box passes all four comparisons while sharing an area of
nothing. It was reported as a blocking overlap of `0 by 0 mm`.

Caught by `a_box_of_no_size_shares_nothing_even_with_itself` on the first run of
the unit checks, before any break was made. The fix states "≥ 1 IU" **once, on
the shared region**: compute the region, return it only if it has at least one
internal unit on each axis. The two forms are not two spellings of one rule —
firing on a shared **point** while refusing a shared **edge** would be
incoherent, since the edge is the larger set. Both forms are kept side by side
in `the_lookalike_disagrees_exactly_where_a_box_has_no_size`, which asserts they
agree on eight non-degenerate pairs and disagree on the two degenerate ones.

## 5. The seam: registration held, the capability seam did not — twice

T1's PASS verdict held for **registration**: one new file, no edit to any
existing file, `build.rs` generated the module list and the registry. Two
capability gaps were measured, and neither was worked around.

**(a) No rule can hold runtime configuration, so `--allow` cannot be wired.**
`Rule::examine` receives a `Drawing` and nothing else, and `Engine::of` takes
`Vec<&'static dyn Rule>` — so a value a command line chose cannot reach a rule
at all. Independently, the lint engine **has no command surface yet**:
`grep -rn "of_every_rule\|Engine::of"` over `crates/` finds callers in
`crates/kicli/tests/` only, `kicli.toml` has no `[rules]` section
(`src/model/config.rs:188-200` lists every section), and `cli/check.rs` is the
project health check rather than the linter. **The `--allow` mechanism is built
and measured; the wiring is one edit in files this lane may not touch.**

**(b) A rule's own type is unreachable from `tests/`.** The seam puts a rule
file behind a private `mod` the build script writes, so `tests/` can reach
`&'static dyn Rule` and nothing more. A *configured* rule therefore cannot be
constructed from an integration test, which is why
`an_allow_entry_exempts_only_the_pair_it_names` is a unit check inside
`overlap.rs`. And the probe harness cannot be used from there either:
`tests/the_linter_holds_no_write_path`'s `USE_ROOTS` whitelist
(`crate`, `std`, `core`, `self`, `super`, `kicli_sexpr`) excludes `kicli_probe`.
So that one check stands on a **hand-built** schematic fragment — permitted by
`ENGINEERING.md`'s stated exception, *"only where no drawable request can
distinguish the behaviour"*, because no drawable request can reach the type at
all. The fragment is kept to the minimum, and the boundary it stands on is
measured through written files in `tests/lint_symbol_overlap.rs`.

**This is sharper than `KI-CONN-001`'s report and in the same place.** That lane
found `Rule::examine` needed data beyond one sheet; this one finds it needs
nothing from the sheet and still cannot be *configured*. **`lint/drawing.rs`
was not touched** — a pure-geometry rule needs nothing from it, as the brief
predicted.

**PROPOSED (lane ovl), for the orchestrator.** Three candidate shapes for the
`--allow` wiring, none implemented: a second argument to `Rule::examine`; a
field on `Drawing`; or a `Vec<Box<dyn Rule>>` in `Engine`. All three are merge
hotspots. Recommendation: decide it with the `sch score` command that will
first need it, not before — a knob with no caller is a knob with no measured
requirement.

## 6. What `geometry/` owes, not duplicated

`grep -r intersect crates/` finds **nothing**: there is no box-intersection
primitive in `crates/kicli/src/geometry/`, and `Rect` has `union`, `contains`,
`inflate` and `transformed` but no `intersection`. `geometry/` is out of this
lane's scope, so `shared_area` is **local to the rule file** and is this
project's first copy. `ENGINEERING.md`'s DRY section licenses that — *"a little
duplication is cheaper than the wrong abstraction. Three strikes before
abstracting"* — and the second strike is close: `KI-WIRE-001` (`lane-wire`)
needs segment-against-box, and `KI-TXT-001` needs oriented-box intersection.

**PROPOSED (lane ovl).** `Rect::intersection` belongs in
`crates/kicli/src/geometry.rs`, with the "≥ 1 IU on each axis" semantics and
the degenerate-box case from §4 above, once a second rule needs it.
Recommendation: whoever writes the second one lifts `shared_area` rather than
copying it, and deletes this copy in the same commit. Revisit trigger: the
`KI-WIRE-001` or `KI-TXT-001` merge.

## 7. No fix command is emitted, deliberately

`KI-CONN-001` offers `kicli junction add --at <p>` because the repair is
determinate. This rule's repair is not: the least move that clears a pair is
computable (four candidates, integer, deterministic) and **may create a new
overlap with a third symbol**, and which of the two symbols should move is a
judgement about the drawing. `record` is used rather than `record_with_fix`.
A fix that is wrong a fraction of the time trains an agent worse than no fix
at all, and the dogfood gate is where that cost shows up.

## 8. Checks, and the falsification of each

Every check below was run with `--no-fail-fast`. **No two checks in this binary
share a probe name** — each is keyed by its own string (`x-edge-2` … `gate-clean`),
which the pair-name sweep varies per step.

### The checks

In `crates/kicli/src/lint/rules/overlap.rs`, `mod tests` (8):

| Check | What it holds down |
|---|---|
| `a_shared_edge_is_not_a_shared_area_and_one_unit_in_is` | the boundary, both sides, **on each axis separately** |
| `a_corner_touch_shares_nothing` | a corner touch, and one unit in on both axes |
| `a_box_of_no_size_shares_nothing_even_with_itself` | the degenerate box, on each axis |
| `the_lookalike_disagrees_exactly_where_a_box_has_no_size` | §4's two forms, side by side |
| `a_box_inside_another_shares_its_whole_self` | containment, and order-independence |
| `an_allow_entry_is_two_names_and_one_colon` | the parse, and that the pair is unordered |
| `an_unreadable_allow_entry_is_refused_rather_than_skipped` | six malformed entries, each refused |
| `an_allow_entry_exempts_only_the_pair_it_names` | `examine` consults the list; a list of other pairs is not an off switch |

In `crates/kicli/tests/lint_symbol_overlap.rs` (8):

| Check | What it holds down |
|---|---|
| `a_shared_edge_does_not_fire_and_one_internal_unit_of_overlap_does` | the x boundary, **swept** across five anchors one internal unit apart |
| `the_same_boundary_holds_on_the_other_axis` | the same sweep on y; a rule strict on one axis passes the first and fails this |
| `a_corner_touch_is_not_an_overlap` | the corner case through a written file |
| `a_turned_body_is_compared_where_it_is_actually_drawn` | one rotated overlapping pair and one rotated clear pair, **each chosen so the turned and unturned boxes give opposite answers** |
| `exactly_the_overlapping_pair_is_named` | three symbols, one pair sharing 1 IU and one pair merely meeting; **which** pair is named, that the third is not, and that the marker sits in the shared area |
| `a_pair_that_is_not_adjacent_in_file_order_is_still_found` | the overlapping pair is the **first and the third**, so a nearest-neighbour walk misses it. **Added while designing break B7**, on the reasoning that the three-in-a-row fixture gives the same answer under `take(1)`; B7's row below is the measurement of that reasoning |
| `power_symbols_are_not_exempt` | a power pair reported on a sheet whose `Density::symbols()` is `0` |
| `one_overlapping_pair_fails_the_gate_whatever_the_sheet_holds` | tier, severity, zero penalty, `Saturation::NEVER` read off a real finding, and the gate failing identically at 2 and at 2,000 symbols |

### The derivation is itself asserted, which is the control that matters

A single "these two touch" fixture would pass just as happily if the body box
were somewhere else entirely and the two boxes happened to miss — the
**reads-nothing** kind of blind instrument wearing a fixture's clothes. So
`tests/lint_symbol_overlap.rs` carries `slab_body()`, which computes the body
box from **the numbers this file writes into the drawing** (`WIDE`, `TALL`, the
anchor, and the quarter-turn matrix `(x, y) -> (y, -x)` cited to
`libs/kimath/src/transform.cpp`) and calls nothing in kicli. `share_an_area()`
restates the published rule in the test's own arithmetic. Every boundary check
asserts the derivation's answer **and then** the rule's, and the two rotated
cases additionally assert that the turned and unturned hypotheses **disagree**.
A body box of a different size or in a different place moves the sweep's
transition and fails it.

### The rotated-symbol trap: what was found

**Nothing was wrong, and the reason is worth recording rather than filing as a
no-op.** `geometry::symbol_box::symbol_boxes` already transforms the body's two
corners by `Transform::from_file(angle, mirror)` before offsetting to the
anchor, so a rule that calls the primitive gets the rotation for free; the trap
only bites a rule that computes a box itself. The two rotated pairs are kept
anyway, because they are what would catch a later author who reached for
`definition.units_for(...)` directly: with the anchors chosen as they are, the
turned and the unturned hypothesis give **opposite** answers on both drawings,
so such an author fails both checks rather than half of one.

### No oracle check is owed, and the derivation's own provenance

This rule **changes no connectivity and writes no file**: it never calls
`Drawing::nets()`, and `tests/the_linter_holds_no_write_path` is the standing
enforcement that it cannot write. So the oracle-check skill's trigger does not
fire.

**But the skill's second rule does apply** — *"established-from-source is not
measured-against-the-tool"* — to the derivation
`tests/lint_symbol_overlap.rs::slab_body()` rests on. Its two load-bearing
facts are **already measured against KiCad**, and the citation is in the
repository:

- `crates/kicli/tests/fixtures/geometry/asymmetric.expected` states
  `abs_pin = symbol.at + M · (lib_pin.x, -lib_pin.y)` — the library Y negation
  and the quarter-turn matrix, composed in that order — for an **asymmetric**
  four-pin part at all four rotations and both mirrors, with
  `MANIFEST` provenance `kicad-cli`;
- `crates/kicli/tests/fixture_oracles.rs::predicted_pin_positions_match_the_rule_check`
  asserts kicli's predicted positions against **KiCad's own ERC report** on
  that fixture and on `orientations`.

**Recorded gap, not hidden.** The last step of the derivation — that a
`rectangle` shape is negated and transformed by the *same* path a pin's `at` is
— is read from source rather than measured: both go through
`model::library::named_point` and `Rect::transformed`, which is one function
each, and **`kicad-cli` exposes no body box at all**, so no oracle of this
project's established shape can close it. The substitute is the boundary
**sweep**: it asserts the body box is where the derivation says it is to within
one internal unit, and a rectangle negated the other way would move the
transition by `2 · TALL` and fail.

### Falsification table

Breaks were made against the committed good state and restored with
`git checkout --`. Content hash of the rule file in its good state:
`shasum crates/kicli/src/lint/rules/overlap.rs` =
`PLACEHOLDER_HASH`.

PLACEHOLDER_TABLE
