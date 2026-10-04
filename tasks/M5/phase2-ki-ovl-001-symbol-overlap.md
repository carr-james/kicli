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

**No existing file was edited and no merge hotspot was touched.**
`git diff --name-status 25aea90 HEAD` reports two additions and this file, and
nothing else. **No fixture was committed, so no `MANIFEST` line is owed** —
every drawing is built by the probe harness into `CARGO_TARGET_TMPDIR`, which
also keeps the `clean` gate arm satisfied by construction. The registration
seam held exactly as T1's PASS said it would: `build.rs` found `overlap.rs` and
registered it with nothing else touched.

**None of the twenty probe directory names is `src`** — they are `x-edge-2` …
`x-edge2`, `y-edge-2` … `y-edge2`, `corner`, `corner-in`, `turned-over`,
`turned-clear`, `three`, `non-adjacent`, `power`, `gate`, `gate-clean` — so the
`the_four_way_rule_has_one_home` trap the brief named is avoided, and all twenty
are distinct, so no two checks of one binary share a name-keyed path.

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
at all, whatever the command line is. Three independent measurements of how far
the gap runs:

- **The lint engine has no caller outside the tests.**
  `grep -rn "of_every_rule\|Engine::of"` over `crates/` finds callers in
  `crates/kicli/tests/` only. `cli/check.rs` is the project health check, not
  the linter, and there is no `sch score` verb yet.
- **`kicli.toml`'s `[rules]` section is validated and not read.** *(Correcting
  my own earlier draft of this paragraph, which said there was no such section
  — measured against `src/model/config.rs` rather than assumed.)* The section
  exists at `config.rs:224-227` with `default_tier2_enabled`, `gate_on_tier1`
  and `consume_erc`, plus per-rule tables `[rules."KI-…-001"]` whose keys are
  `RULE_KEYS = ["enabled", "weight", "free_allowance"]` (`config.rs:233-234`).
  **`Config` has no `rules` field at all** (`config.rs:147-163`), so every one
  of those keys is type-checked on the way in and then discarded. The purpose
  is stated in that file's header — catch a typo in the milestone that writes
  the file, not the one that reads it.
- **There is no `allow` key, and the catalogue's own knob name is not there
  either.** `research/style-rules.md` §4 names `overlap.symbol = "error"`;
  `RULE_KEYS` has no `allow` and no `severity`. Adding either is a `config.rs`
  change and a `kicli.toml` change, both merge hotspots.

**The `--allow` mechanism is built and measured; the wiring is an edit in files
this lane may not touch.**

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

## 7. No fix command is emitted — **PROPOSED (lane ovl)**

`KI-CONN-001` offers `kicli junction add --at <p>` because the repair is
determinate: one junction at one point, and it joins every line through that
point. This rule's repair is **not** determinate, and the command it would use
exists — `kicli sym move <ref> --by x,y` (`cli/args.rs:484-485, 585`), so the
choice is deliberate rather than forced.

The least move that clears a pair **is** computable: four candidates, integer,
deterministic, pick the smallest. Two things make it the wrong thing to print.
It **may create a new overlap with a third symbol**, which the rule cannot see
from one pair; and **which of the two symbols should move is a judgement about
the drawing**, which the rule does not have. So `record` is used rather than
`record_with_fix`.

Recommendation: keep it. A fix that is wrong a fraction of the time trains an
agent worse than no fix at all, and Constitution §6's reader is an agent that
will run what it is given. Revisit trigger: a dogfood run where the absence of
a suggested move costs the agent a turn — that would be evidence, and this
paragraph is not.

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
only bites a rule that computes a box itself.

**The check is kept anyway, and break B6 is why it earns its place.** B6 drops
the rotation at the rule's own call site — `drawn.angle = Angle(0);
drawn.mirror = None;`, which is exactly what a later author who reached for
`definition.units_for(...)` directly would produce — and
`a_turned_body_is_compared_where_it_is_actually_drawn` is the **only** check in
the suite that fails. Both of its arms fail, because the two anchors are chosen
so the turned and the unturned hypothesis give **opposite** answers: on one the
rule stops reporting an overlap that is there, and on the other it starts
reporting one that is not.

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

**Method.** Twelve breaks, each made against the **committed** good state —
`1d437e3` — and restored with `git checkout --`, with the restored file's
**content hash** checked after every row rather than trusting the command's exit
code. Content hash of the rule file in its good state:
`shasum crates/kicli/src/lint/rules/overlap.rs` =
`c10701b4582cd5427b6b62141fe29a72e4775146`. Evidence is anchored to that hash
rather than to a commit SHA, per the falsification-control skill: these commits
will be merged forward.

Each row ran `cargo test -p kicli --no-fail-fast --lib --test lint_symbol_overlap`.
The reasoning for choosing those two targets was: no other target can see a
*behavioural* change in this rule — `lint_findings_are_bit_identical`,
`lint_scores_are_bit_identical` and `lint_findings_sort_by_their_key` run
`specimens::all()` rather than the crate rules; `lint_gate_separates_the_tiers`
builds explicit rule lists; `lint_pin_on_wire` filters to `KI-CONN-001`; and
`lint_rules_register_from_their_own_files` asserts nothing about what a crate
rule reports (its own comment says so).

**That reasoning was verified rather than trusted, and it was WRONG — see
"the two-target claim was short by one" below.** B3 was re-run over the whole
suite and found a tenth catcher outside both targets.

| # | What was broken, exactly | The defect it models | Caught by |
|---|---|---|---|
| **B1** | `shared_area`: the **x** half of the region guard, `start.x >= end.x` → `start.x > end.x`. The y half left alone. | a shared edge on x fires — the one-character defect | **6** — `a_box_of_no_size_shares_nothing_even_with_itself` **(unit)**; `a_shared_edge_is_not_a_shared_area_and_one_unit_in_is` **(unit)**; `the_lookalike_disagrees_exactly_where_a_box_has_no_size` **(unit)**; `a_shared_edge_does_not_fire_and_one_internal_unit_of_overlap_does`; `exactly_the_overlapping_pair_is_named`; `one_overlapping_pair_fails_the_gate_whatever_the_sheet_holds` |
| **B2** | `shared_area`: the **y** half only, `start.y >= end.y` → `start.y > end.y`. | the same defect on the other axis | **4** — `a_box_of_no_size_shares_nothing_even_with_itself` **(unit)**; `a_shared_edge_is_not_a_shared_area_and_one_unit_in_is` **(unit)**; `the_lookalike_disagrees_exactly_where_a_box_has_no_size` **(unit)**; `the_same_boundary_holds_on_the_other_axis` |
| **B3** | `shared_area`: **both** halves relaxed to `>`. | `<=` everywhere: every edge and corner touch fires | **9** — `a_box_of_no_size_shares_nothing_even_with_itself` **(unit)**; `a_corner_touch_shares_nothing` **(unit)**; `a_shared_edge_is_not_a_shared_area_and_one_unit_in_is` **(unit)**; `the_lookalike_disagrees_exactly_where_a_box_has_no_size` **(unit)**; `a_corner_touch_is_not_an_overlap`; `a_shared_edge_does_not_fire_and_one_internal_unit_of_overlap_does`; `exactly_the_overlapping_pair_is_named`; `one_overlapping_pair_fails_the_gate_whatever_the_sheet_holds`; `the_same_boundary_holds_on_the_other_axis` |
| **B4** | `bodies_of`: `if symbol.is_power() { continue; }` inserted after the definition guard. | power symbols exempted — the quiet 'fix' the catalogue forbids | **1** — `power_symbols_are_not_exempt` |
| **B5** | `bodies_of`: `symbol_boxes(...).body` → `symbol_boxes(...).full`. | the `KI-TXT-001` box used instead of this rule's | **7** — `a_corner_touch_is_not_an_overlap`; `a_pair_that_is_not_adjacent_in_file_order_is_still_found`; `a_shared_edge_does_not_fire_and_one_internal_unit_of_overlap_does`; `a_turned_body_is_compared_where_it_is_actually_drawn`; `exactly_the_overlapping_pair_is_named`; `one_overlapping_pair_fails_the_gate_whatever_the_sheet_holds`; `the_same_boundary_holds_on_the_other_axis` |
| **B6** | `bodies_of`: `let drawn` → `let mut drawn`, then `drawn.angle = Angle(0); drawn.mirror = None;`. | the body measured **unrotated** — the rotated-symbol trap | **1** — `a_turned_body_is_compared_where_it_is_actually_drawn` |
| **B7** | `examine`: `bodies.iter().skip(place + 1)` → `…skip(place + 1).take(1)`. | nearest neighbour only, instead of every pair | **1** — `a_pair_that_is_not_adjacent_in_file_order_is_still_found` |
| **B8** | `Allowed::permits`: the second disjunct — the `pair.one == two && pair.two == one` half — removed, leaving only `pair.one == one && pair.two == two`. | the allow pair becomes **ordered** | **2** — `an_allow_entry_exempts_only_the_pair_it_names` **(unit)**; `an_allow_entry_is_two_names_and_one_colon` **(unit)** |
| **B9** | `examine`: the whole three-line guard `if self.allowed.permits(&one.name, &two.name) { continue; }` removed. | the allow list never consulted at all | **1** — `an_allow_entry_exempts_only_the_pair_it_names` **(unit)** |
| **B10** | `tier`: `Tier::One` → `Tier::Two`. | the rule stops blocking | **1** — `one_overlapping_pair_fails_the_gate_whatever_the_sheet_holds` |
| **B11** | `saturation`: `Saturation::NEVER` → `Saturation::of(Counted::Symbols)`. | the false, inert denominator of §3 | **1** — `one_overlapping_pair_fails_the_gate_whatever_the_sheet_holds` |
| **B12** | `examine`: `shared.centre()` → `Point::default()` as the finding's position. | the marker moved off the overlap to the origin | **1** — `exactly_the_overlapping_pair_is_named` |

**Twelve breaks, twelve caught, no green row.** The restored file's content
hash was `c10701b4582cd5427b6b62141fe29a72e4775146` after **every** row, checked
by `shasum` rather than inferred from `git checkout --`'s exit code.

**The file has moved since, and the table was re-anchored rather than left to
rot.** The raw-string fix below changes `overlap.rs` to
`7715747ab06d4e1f2eff20137c18af3ee475bd6c`. The edit is confined to the spelling
of the `TWO_OVERLAPPING` fixture literal — no break in the table touches that
constant, and the rule's own code is unchanged — so the twelve rows stand. Not
left as an inference: **B8 and B9, the two rows whose only catchers consume that
fixture, were re-run at `7715747a…` and returned the identical catcher lists**
(`an_allow_entry_exempts_only_the_pair_it_names` and
`an_allow_entry_is_two_names_and_one_colon` for B8; the first alone for B9).

### The two-target claim was short by one, and the tenth catcher was a red gate

**This is the most important row in the entry, and it is not in the table.**

Re-running B3 over the whole suite returned **ten** catchers, not nine. The
tenth was `no_floating_point_appears_under_the_linter`, in
`crates/kicli/tests/the_linter_holds_no_floating_point.rs` — a target my
reasoning had ruled out because it is a *textual* sweep and B3 changes no
arithmetic. The reasoning was right about B3 and wrong about the gate:
**`overlap.rs` was failing that sweep in its GOOD state, and had been since the
allow-list check was added.** Measured both ways, at the same content hash the
whole table was made against:

```
$ shasum crates/kicli/src/lint/rules/overlap.rs
c10701b4582cd5427b6b62141fe29a72e4775146     # the good state
$ cargo test -p kicli --test the_linter_holds_no_floating_point
no_floating_point_appears_under_the_linter ... FAILED
the linter's arithmetic must be exact: ["overlap.rs: 000000000000A4Probe",
  "overlap.rs: 000000000001ReferenceR1ValueSLABprobe",
  "overlap.rs: 000000000002ReferenceR2ValueSLABprobe"]
```

**Root cause.** The sweep's `code_of` strips comments and string literals so
that prose about floating point and a decimal inside a message are not read as
arithmetic. It understands `"…"` with `\` escapes, line comments, block
comments and character literals. It does **not** understand a Rust **raw string
literal**, `r#"…"#` — and the `TWO_OVERLAPPING` schematic fragment I wrote for
the allow-list check was **the first raw string anywhere under `src/lint/`**
(`grep -rn 'r#"' crates/kicli/src/lint/` returned exactly one line, mine).
The lexer opened a string at the fragment's first `"` and closed it at the
next, which inverts inside-and-outside for the rest of the literal; schematic
text was then read as code, `a_number` took `000000000000` and swallowed the
trailing name characters `A4Probe`, and `is_a_float` flagged it because the
result contains an `e`.

**Fix, in my own file.** `TWO_OVERLAPPING` is now an ordinary string literal
with `\"` escapes and `\n\` continuations — which is the house pattern
already used by `geometry/symbol_box.rs`'s `PIN_SOURCE` — and all four arms of
the sweep pass. `the_linter_holds_no_floating_point.rs` is an **existing** file
and not this lane's to edit, so the sweep itself is untouched and reported
instead.

**PROPOSED (lane ovl) — the sharper half, which is a gate defect rather than my
inconvenience.** The failure I hit is the *safe* direction: a false positive,
loud, on a file holding no float. The same inversion runs the other way. A raw
string anywhere under `src/lint/` shifts the lexer's inside/outside state for
everything after it, so **real arithmetic following a raw string can be stripped
as though it were string content — and a genuine `f64` would then pass the
gate silently.** The sweep's own rustdoc states one boundary (*"a textual sweep
cannot see a float that arrives as another module's return type"*) and does not
state this one. Recommendation: `code_of` learns raw strings — `r`, then zero or
more `#`, then `"`, closing on `"` followed by the same number of `#` — and
`the_sweep_permits_what_integer_arithmetic_looks_like` gains a raw-string row
plus a row with a float *after* a raw string, which is the case that must go
red. Until then the honest statement is that **`src/lint/` must hold no raw
string**, and this entry is the only place that is written down. Revisit
trigger: the next rule author who wants an inline s-expression fixture.

**A third thing the full gate caught that no targeted run could.** The `clippy`
arm failed on `clippy::struct_field_names`: the rule's private
`struct Body { name, object, body }` has a field named exactly its own struct,
and `-D warnings` makes that an error. Renamed to `Compared`; the field, the
function `bodies_of` and every falsification citation are unchanged, and
`cargo clippy --all-targets --all-features -- -D warnings` is clean. Noted
because it is the **same class** as the raw-string failure above: a rule file
is invisible to `cargo fmt`, and it turns out the author's habit of running
`cargo test -p kicli --test <name>` makes it effectively invisible to `clippy`
too. **The only instrument that sees a rule file whole is `cargo xtask check`,
and a lane that runs targeted tests to go fast will hand over a red branch.**

**And one process finding, recorded because it nearly cost the whole table.**
The commit `e2f594b` was made with `git add -A` **while the full-suite B3 run
was mid-flight**, so it captured the applied break: `git show e2f594b --
crates/kicli/src/lint/rules/overlap.rs` is the one-line `>=` → `>` edit. It was
caught by the harness's own control — the restored-state content hash printed
`a8d543b5…` for that row where every other row printed `c10701b4…` — and
reverted by `git checkout 1d437e3 -- crates/kicli/src/lint/rules/overlap.rs`,
verified back to `c10701b4…`, in the commit that follows. The
falsification-control skill warns that *"git will not hold your good state"* in
the direction **restore-loses-work**; this is the mirror case,
**commit-captures-break**, which it does not name. Recommendation: add the
mirror to that skill — *never stage while a break harness is running; a
`git add -A` is as destructive as a `git checkout --` when something else owns
the tree* — and keep the harness's per-row hash print, which is what made it a
five-minute fix instead of a reviewer's finding.

### Four things the table says that prose would have hidden

**B4 and B6 are caught by exactly one check each, and that is the argument for
those two checks existing.** Nothing else in the suite sees a power exemption,
and nothing else sees a body measured unrotated. A rule file reviewed by reading
would have both defects available to a later editor with no instrument pointing
at them.

**`power_symbols_are_not_exempt` survived B5**, and that is a limit of that
check rather than a surprise: it asserts that a finding *exists* and names the
two power symbols, and the full box is *larger* than the body box, so a break
that enlarges every box leaves it green. Recorded rather than smoothed over —
the check's claim is "power symbols are included", and it does not and should
not also claim which box was used. B5's seven catchers are where that claim
lives.

**B1 against B2 is the axis contrast.** B1 fires the x sweep and leaves the y
sweep green; B2 does the reverse. A single square fixture would have been caught
by neither in a way that distinguished them, and a rule relaxed on one axis only
is the shape a careless edit makes.

**B7 measured the design reasoning that produced
`a_pair_that_is_not_adjacent_in_file_order_is_still_found`, and confirmed it.**
The check was written *before* B7 ran, on the prediction that
`exactly_the_overlapping_pair_is_named` — three symbols in a row, one pair
sharing an area, one pair merely meeting — **cannot** tell a full pair walk from
a nearest-neighbour one, because the only overlapping pair in it is adjacent.
B7 (`.take(1)`) is caught by the new check and by **nothing else in the suite**,
including that one. Had the check not been added, B7 would have been a green row
and the entry would have had to record a blind instrument.

**B10 and B11 land on the same single check, which is the honest cost of a tier
that is one line.** `one_overlapping_pair_fails_the_gate_whatever_the_sheet_holds`
is the only thing standing between `Tier::One` and `Tier::Two`, and the only
thing standing between `Saturation::NEVER` and the false denominator §3 argues
against. Both are one-token edits in a file `cargo fmt --check` cannot see. That
check is therefore load-bearing out of proportion to its length, and it is named
here so a later reader does not treat it as a formality.
