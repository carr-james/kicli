# The score formula and the normalisers (Phase 1, T3) ✅

**Provenance: `tasks/M5/PLAN.md` Phase 1, RATIFIED by James's ratification and
advisor rulings, M5 plan review.**

**Depends on T1.** The formula sums over findings; the finding is T1's.

## The formula, from `spec/SPEC.md` §11.5, which is the contract

```
raw_penalty(sheet) = Σ_rules  w_r · n_r · norm_r
score(sheet)       = round( 100 · exp( −raw_penalty / K ) ),   K = 25
```

| Normaliser | Applies to | Definition |
|---|---|---|
| `per_object` | field / symbol / text rules | `1 / max(1, N_sym/20)` — 20 non-power symbols is the reference sheet |
| `per_wire` | crossings, doglegs | `1 / max(1, N_wire/10)` |
| `per_sheet` | flow, layout, docs | `1` |

`N_sym` **excludes power symbols**. **Project score = symbol-count-weighted mean
of sheet scores.**

## The rule that shapes every line of this task

**Constitution §4, and it is not negotiable**: detection is **integer geometry
only**. Floating point appears in `score()`'s **final `exp`** and nowhere else,
with fixed rounding.

Read that against the formula above and the difficulty is immediate: `norm_r` is
a *reciprocal*, `1 / max(1, N_sym/20)`. **A reciprocal is where a float sneaks
in**, and `raw_penalty` is a sum of products of reciprocals. Getting this right
is the task.

**The executable twin already exists for the router and you build its sibling**:
`crates/kicli/tests/the_router_holds_no_floating_point.rs`. Read it first. It is
the model, it is known to work, and M4's calibration lesson is that a gate
presented as measuring something it cannot fail on is worse than no gate.

## Goal state, as the checks that prove it

### 1. No floating point under `src/lint/` outside the final `exp`

The gate, per the plan's exit-criteria table: *"can fail on — any float under
`src/lint/` outside the final `exp`."*

**Show it failing.** Add a float somewhere under `lint/`, confirm red, remove
it. A sweep that has never been shown to catch anything is a sweep nobody has
checked is looking.

**And name the exemption precisely.** "The final `exp`" is one call site, not a
module. If your gate exempts a file or a function rather than an expression, say
so and say what that costs — a broad exemption is how the rule dies quietly.

### 2. Density normalisation actually normalises

§11.5's own justification is the check: *"a sheet with 4 symbols and one crossing
is worse than a sheet with 200 symbols and one crossing, so absolute counts will
not survive calibration."*

**So the check is that exact sentence, executable**: two fixtures, 4 symbols and
200 symbols, one crossing each, and the small one scores worse. Not "the
normaliser returns the expected number" — that is a check on arithmetic. This is
a check on the property the arithmetic exists for.

**The anti-vacuity control matters here**: confirm the two sheets score
*differently at all*, and that with normalisation disabled they score the same.
Otherwise the check passes on two fixtures that differ for some other reason.

### 3. Tier 1 does not enter `raw_penalty`

§11.5: *"Tier 1 findings do not reduce the score."* The full separation is T4's
task; **T3 owes the half that lives in the formula** — the sum runs over Tier 2
only. Say where you enforce it, because "the caller filters first" and "the sum
skips them" fail differently under a new caller.

### 4. Project score is the symbol-count-weighted mean of sheet scores

Note what it is **not**: the score of the concatenated findings, and not the
unweighted mean. A two-sheet project with a 10-symbol sheet at 40 and a
200-symbol sheet at 95 has a project score near 95, and a fixture asserting that
is worth more than a comment saying it.

### 5. Determinism, and it is bit-identical

§11.5: findings sorted by `(rule, sheet, x, y, uuid)` before output; **re-scoring
an unchanged file is bit-identical**. T1 owns the sort; **T3 owns that the
*number* is stable**, which is a different claim — `exp` and `round` are where a
platform difference would show up, and "fixed rounding" in §11.5 is the phrase
that makes that this task's problem.

## Weights are NOT tuned here

`RULES.md`, "The weights arrive already measured, and M5 owns them": **a weight
is moved in this milestone only with a measurement of that shape beside it**, and
Phase 4 is where those measurements get made. §11.6 is explicit that **weights
are not regression-fitted**, and that **`K` is frozen last**, by requiring set A
to land in 85–100 and worst-degraded set B in 30–50.

**T3 ships `K = 25` and the catalogue's stated weights as starting points, and
says in the code that they are starting points.** A weight changed here to make a
fixture read nicely is the exact failure Phase 4 exists to prevent, and it would
be invisible.

## The north star, and why it bites this task specifically

`RULES.md`: *"It must never reward a schematic that is impossible to read and
understand."*

`exp(−raw/K)` is **asymptotic**: it approaches zero and never reaches it, so
every schematic scores above zero however bad it is. Worse, density
normalisation **divides penalties by symbol count**, so a large unreadable sheet
is penalised less per finding than a small one.

**Those two facts together are a mechanism by which a truly unreadable drawing
could score respectably**, which is what the north star forbids. It is Phase 4's
job to measure whether it happens, and **T3's job to state the exposure in the
entry** rather than let Phase 4 discover it. If the arithmetic shows it, that is
a PROPOSED item against the north star — not a value call for a lane to make.

## Falsification obligation

Per `.claude/skills/falsification-control/SKILL.md`. Three named traps:

- **The no-float gate**: shown failing, per above. This one is not optional.
- **The density check is degenerate if both fixtures come from the same
  generator with the same defaults** — they would differ in symbol count and in
  nothing else only if you engineered that, so confirm you did.
- **The determinism check must cross a process boundary**, or it compares two
  results of one call and passes on a scorer that reads a global.

## Scope

**IN**
- `crates/kicli/src/lint/score.rs` and what it needs under `crates/kicli/src/lint/`
- new test files under `crates/kicli/tests/`
- `crates/kicli/tests/fixtures/**` — new fixtures only
- this file, for the evidence, written AS YOU WORK

**MERGE HOTSPOTS — report, do not edit.** `Cargo.toml`, `crates/kicli/src/lib.rs`,
the fixture `MANIFEST`, `AGENT.md`, `spec/SPEC.md`, `crates/kicli/tests/command_surface.rs`,
`kicli.toml`'s `[rules]` table.

**OUT** — every other module, every other entry, `tasks/M5/PLAN.md`. **And the
weights and `K`**, which are `RULES.md`-governed and Phase 4's.

**If the enumeration above proves wrong, the named goal state and its checks win
over the list.** Say so in your first paragraph, name what you touched and why.

## Completion check

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo xtask check
```

plus the no-float gate by name, run and shown failing on an injected float.

---

# Evidence (T3 implementer, written as the work was done)

## What was built, and where

| File | What it holds |
|---|---|
| `crates/kicli/src/lint/score.rs` | `Density`, `Normaliser`, `RawPenalty`, `SheetScore`, `score_of`, `project_score`, and the fixed point decay. New. |
| `crates/kicli/src/lint.rs` | `pub mod score;` and the re-export line beside the existing ones. Two lines. |
| `crates/kicli/tests/the_linter_holds_no_floating_point.rs` | The gate. New. |
| `crates/kicli/tests/lint_score_normalises_by_density.rs` | The density property, its two anti-vacuity arms, the blocking-tier arm and the project mean. New. |
| `crates/kicli/tests/lint_scores_are_bit_identical.rs` | The number's determinism, across a process boundary. New. |

**No merge hotspot was touched.** `Cargo.toml`, `crates/kicli/src/lib.rs`,
`crates/kicli/build.rs`, the fixture `MANIFEST`, `AGENT.md`, `spec/SPEC.md`,
`crates/kicli/tests/command_surface.rs` and `kicli.toml` are unchanged; so is
`crates/kicli/src/lint/rules/`, which does not yet exist. No fixture file was
added: both drawings are written by the probe at run time, into
`CARGO_TARGET_TMPDIR`, so `tests/fixtures/**` has a zero diff.

## Deviation, disclosed: there is no floating point at all, not even in the decay

The published rule permits floating point in the score's **final exponential**
and forbids it everywhere else. **This implementation does not take that
permission.** The decay is evaluated in fixed point, from the exponential series
summed in `u128` at a scale of `10^17`, and rounded once at the end.

The reason is that the permission is what would have to be exempted from the
gate. "The final `exp`" is one expression, and a sweep cannot exempt one
expression — it can exempt a file, a function or a name, and each of those is a
hole a later author can widen without anybody reading a diff noticing. **Taking
no float at all costs one function (`grown_by`, 12 lines) and buys a gate with
zero exemptions.** The prohibition is then satisfied more strictly than it is
written, not less.

The rounding is decided rather than inherited: halves away from zero, in
integer arithmetic, in `rounded()`. `exp(−t)` for a non-zero rational `t` is
transcendental, so `100·exp(−t)` is never exactly at a rounding boundary and the
choice can never be observed. It is written down because "fixed rounding" has to
mean something a second implementer could reproduce.

### The fixed point decay, and why it is exact enough

`raw_penalty` is held in **billionths of a point** in `RawPenalty`. A rule's
weight arrives in thousandths (T1's `Penalty`), and each normaliser is an exact
ratio — `20/max(20, N_sym)`, `10/max(10, N_wire)`, or `1/1` — so the sum is
integers over three denominators. **One division per normaliser, not one per
finding**: three truncations in the whole sheet, each below one billionth of a
point.

`score_of` then converts once to the series scale, sums the series, and takes
the reciprocal. Above six decay constants of penalty it returns zero, because
`100·exp(−6)` is `0.248` and rounds to nothing; the decay is asymptotic and
never reaches zero, so the cut is placed where rounding makes it unobservable.

**Measured against an independent oracle.** `100·exp(−raw/25)` computed in
double precision by Python, against the fixed point implementation, at raw
penalties of 0, 1, 5, 25, 50, 100 and 149 points: `100, 96, 82, 37, 14, 2, 0` on
both sides, no disagreement. Recorded in `the_decay_matches_the_published_function`.
`the_decay_never_rises` walks 0 to 199 points and asserts the sequence is
monotone and ends at zero.

## Goal state item by item

### 1. No floating point under `src/lint/`, with no exemption

`cargo test --test the_linter_holds_no_floating_point`, four checks. The sweep
strips comments **and string literals** first — `finding.rs` already asserts
`Penalty::points(3).text() == "3.0"`, and a sweep that read string literals
would fail on prose from the first minute and be relaxed by the next author.

Two mechanisms are refused, and the second is the one the router's sibling sweep
does not have:

- the type names `f32` and `f64`, whole-word;
- **float literals**, in the three shapes the Rust grammar gives them: a decimal
  point, an exponent, or an `f32`/`f64` suffix. `let scale = 1.0;` names no
  type, so a sweep that matched only type names would not see it.

**The boundary is stated, per the derivation rule.** A textual sweep cannot see
a float that arrives as another module's return type without ever being named or
cast. What bounds that hole is the companion sweep
`the_linter_holds_no_write_path`, which whitelists the five modules of this crate
the linter may name at all — so the surface a float could arrive through is
short enough to read. That sentence is in the sweep's own module docs, not only
here.

### 2. Density normalisation actually normalises

`cargo test --test lint_score_normalises_by_density`, five checks.

Both drawings come from **one generator with one parameter**, `sheet_of(name,
symbols)`: the same symbol definition, one wire per symbol, and the same two
crossing wires at the same coordinates on every sheet it writes. The engineered
difference is the count and the wire count that follows it, as a real drawing's
does. The generator's own control is in `the_two_sheets`, which asserts the
densities read back from the written files are `4/6` and `200/202` — **measured
from the file by `Density::of`, not asserted by the test**.

| Check | Sparse (4 symbols) | Crowded (200 symbols) |
|---|---|---|
| one crossing, `KI-XING-001`, weight 3 | **89** | **99** |
| one stray field, `KI-FLD-001`, weight 3 | **89** | **99** |
| one flow finding, `KI-FLOW-001`, weight 3 — no normaliser | 89 | 89 |

The third row is the **anti-vacuity control the task named**: the same weight,
on the same two drawings, under a rule that divides by nothing, scores the same
on both. The difference in rows one and two therefore comes from the normaliser
and not from something else about the two files. The second arm — that the two
sheets differ *at all* — is `assert_ne!(small.score(), large.score())` in the
crossing check.

Both normalisers are exercised, because a crossing divides by the wire count and
a field divides by the symbol count, and a check that only ever exercised one
would leave half the arithmetic unmeasured.

### 3. Tier 1 does not enter `raw_penalty`, and the sum is where that lives

`RawPenalty::weight_of` filters on `finding.tier == Tier::Two`. **The sum skips
them; no caller filters.** The distinction is the one the task asked for: a
caller-side filter is a convention the next caller need not keep, and a blocking
finding must never move the score whoever asks.

It is measured by handing the scorer a blocking finding **directly**, which a
caller that filtered first would never do:
`a_blocking_finding_leaves_both_sheets_untouched` (integration, both densities)
and `a_blocking_finding_never_moves_the_score` (unit, 50 points of blocking
weight, raw stays `ZERO`, score stays 100).

### 4. Project score is the symbol-count-weighted mean

`project_score` weighs each sheet by `density().symbols()`. The task's own worked
example is the fixture: a 10-symbol sheet at 40 and a 200-symbol sheet at 95 give
**92**, and the check also asserts the answer is **not 68**, which is the
unweighted mean of the same two sheets. In the integration file the two real
drawings at 89 (4 symbols) and 99 (200 symbols) give **99**, not the unweighted
94.

Degenerate input, **PROPOSED**: a project whose sheets hold no non-power symbols
at all has nothing to weigh by. `project_score` falls back to the unweighted
mean, and a project of no sheets scores 100. Recommendation: keep both; the
unweighted mean is the continuous limit of the weighted one and neither case can
arise from a drawing that holds a part. Recorded because it is a decision, not
because it is doubtful.

### 5. The number is stable, across a process boundary

`cargo test --test lint_scores_are_bit_identical`, four checks plus the child.
Modelled on T1's `lint_findings_are_bit_identical`, and it crosses the same
boundary: two copies of one drawing at two paths, two child processes, each with
its own working directory, `TZ` (`UTC` against `Pacific/Auckland`) and `LC_ALL`
(`C` against `en_GB.UTF-8`). What is compared is the children's bytes.

The report prints every number that goes into a score — symbols, wires, the raw
penalty text and the score, per sheet, then the project score — so a report that
agreed while a density differed would not compare equal.

Two controls against a true-for-the-wrong-reason pass:

- **not vacuous**: at least one sheet line must not end `score=100`. Every sheet
  scoring the best there is would be the same bytes for any drawing at all.
- **not blind**: the same binary over a **different** drawing (3 symbols against
  40) must print different bytes.

The specimen rules are one family, so their findings would all take one
normaliser. `under_every_normaliser` copies each finding under a crossing code
and a field code as well as its own, and
`every_normaliser_takes_part_in_the_report` asserts the three codes take the
three different normalisers. The tier is copied with the finding, so a blocking
specimen finding stays blocking under a crossing code — which exercises the tier
filter across the boundary too.

## Weights and `K` were not tuned

`DECAY_POINTS = 25`, and its rustdoc says in full sentences that it is a
starting point frozen last by calibration, and that nothing in the module may be
tuned to make one drawing read nicely. **No rule weight is in this module at
all**: a weight arrives on the finding, from the rule, and the rules are Phase 2
and 3. The weights used in the checks above are the checks' own numbers, chosen
to make the arithmetic legible, not defaults shipped anywhere.

`REFERENCE_SYMBOLS = 20` and `REFERENCE_WIRES = 10` are the published
normaliser definitions, not weights.

## PROPOSED: the normaliser table does not cover every family

The published table assigns `per_object` to "field / symbol / text" rules,
`per_wire` to "crossings, doglegs" and `per_sheet` to "flow, layout, docs".
Mapped onto the catalogue's codes that gives `FLD`, `SYM`, `TXT` → `per_object`;
`XING`, `RTE` → `per_wire`; `FLOW`, `LAY`, `DOC` → `per_sheet`.

**Three Tier 2 families are named nowhere in the table**: `JCT`, `LBL` and
`DNP`. They default to `per_sheet`, which divides by nothing.

Recommendation: keep the strict default. An unlisted rule keeping its whole
weight can only make a bad drawing score worse, which is the direction the north
star points; the other default would quietly reduce penalties nobody decided to
reduce. `an_unlisted_family_keeps_its_whole_weight` measures it on both
densities so the default is visible rather than implied.

## PROPOSED: the normaliser is derived from the rule's code, and three rules disagree with their code

`Normaliser::of(RuleId)` reads the family out of the code. That works because
the published table is stated in the same vocabulary the codes use — and it is
wrong for at least two catalogue rules whose **nature** disagrees with their
**family**. The third row is here because it looked like a third and is not,
which is the reason the table is worth reading rather than skimming:

| Rule | Its own definition | Family gives it | Nature suggests |
|---|---|---|---|
| `KI-LAY-003` | "W 1 per unaligned **symbol**" | `per_sheet` | `per_object` |
| `KI-JCT-001` | four-way junction, a **wire** feature | `per_sheet` | `per_wire` |
| `KI-DNP-001` | "W 1 per DNP part beyond the allowance", allowance `max(2, 0.05·N_sym)` | `per_sheet` | `per_sheet`, on inspection |

`KI-DNP-001` counts symbols and is still right at `per_sheet`, because its own
detection already divides by the symbol count: its allowance is
`max(2, 0.05·N_sym)`. Normalising it again would divide by the symbol count
twice. **That is the argument against a table in the scorer in one line** — the
answer depends on what the rule's detection already does, which is knowledge the
rule has and the scorer does not.

The consequence is not small. `KI-LAY-003` un-normalised costs one point per
unaligned symbol with no ceiling, so a 200-symbol sheet with every symbol
unaligned reaches 200 raw points and scores 0, while every other symbol-shaped
rule on the same sheet is divided by ten.

Recommendation, and **it is a seam change rather than a table edit**: the rule
should declare its own normaliser, as it declares its own tier and weight — a
`Rule::normaliser()` with a default, stamped onto the finding by `Findings::of`
exactly as the tier and the weight already are. A new rule then carries its
normaliser in its own file, and no central table has to be edited by two authors
at once, which is the same argument the generated registry already won.

**Not taken in this task** because `rule.rs` and `finding.rs` are T1's files and
T4 is in them this dispatch; a lane adding a field to `Finding` while another
lane separates the tiers in it is a merge collision for no gain in a phase where
no rule exists yet. The family table is behaviourally identical for every rule
the table does name, and `each_family_takes_the_normaliser_the_catalogue_gives_it`
pins what it does today so the change is visible when it is made.

## PROPOSED, against the north star: a large drawing has a floor it cannot fall through

**The north star**: *"It must never reward a schematic that is impossible to read
and understand."* The task asked for the exposure to be stated with real numbers
rather than discovered in Phase 4. It is real, it is arithmetic rather than
speculative, and it is now executable in
`a_normalised_rule_cannot_take_off_more_than_its_ceiling`.

**The mechanism.** A normalised rule contributes `w · n · reference / max(reference,
N)`. If the rule can fire at most once per object it counts — a crossing per wire,
a bad field per symbol — then `n ≤ N`, and the contribution is **capped at `w ·
reference`**, whatever `N` is. The cap does not fall as the drawing grows. It is
the same at ten thousand wires as at ten.

**Measured**, with the implementation, at weight 1:

| Drawing | Raw penalty | Score |
|---|---|---|
| 10 wires, **every wire crosses another** | 10.0 | **67** |
| 200 wires, **every wire crosses another** | 10.0 | **67** |
| 10 000 wires, **every wire crosses another** | 10.0 | **67** |
| 20 symbols, **every symbol's fields wrong** | 20.0 | **45** |
| 200 symbols, **every symbol's fields wrong** | 20.0 | **45** |
| 10 000 symbols, **every symbol's fields wrong** | 20.0 | **45** |

A sheet on which **every single wire crosses another with no junction** — which
is not a readable drawing by any standard — scores **67** from that rule, and
scores 67 whether it holds ten wires or ten thousand. The catalogue gives
`KI-XING-001` a weight of 1, so 67 is the number that rule alone can reach today.

**The second half is sharper, and it is an inversion.** Because `per_sheet` rules
are not normalised at all, they overtake the normalised ones as a sheet grows. On
a 202-wire sheet, twenty unresolved crossings cost `1 × 20 × 10 / 202 = 0.99`
raw points. An incomplete title block (`KI-DOC-003`, weight 2 per sheet) costs 2.
**A missing title block is scored at twice the cost of twenty wires that cross
without junctions.** That is a defensible normaliser producing an indefensible
comparison, and no single weight is wrong for it to happen.

**Recommendation, and the value call is deliberately not made here.** `K` and the
weights are `RULES.md`-governed and Phase 4's, and tuning either to hide this
would be exactly the invisible failure Phase 4 exists to prevent. Three options,
for a ruling:

1. **Leave it, and let Phase 4 measure it.** The ceilings only bind when a rule
   fires on nearly every object, and set B may never reach that.
2. **Give the gate the job the score cannot do.** A saturating rule — a rule
   firing on more than some fraction of the objects it counts — becomes a
   blocking finding rather than a scored one. The north star's second sentence
   is about *rewarding*, and a drawing that fails the gate is not rewarded
   whatever it scores. This is the recommendation.
3. **Cap the normaliser's reach**, so `norm` never falls below some floor. This
   is the option that changes the published formula, and it should not be taken
   without a measurement.

Option 2 is recommended because it needs no weight to move and no formula to
change: it is a tier decision, which is T4's subject, and it uses a mechanism the
specification already has.

## Falsification

Every run is the **whole suite**, `cargo test --no-fail-fast -p kicli`, because
cargo stops at the first failing target and a caught-by list built from a
stopping run under-counts. The good state was committed before the first break;
each break was applied by patch, run, reverted with `git checkout --`, and the
restored file checksummed against `b84b8744e0bb1d9eef349e31c004b8d7d9b7f4ec`
every time.

Good state, at hand-off:

```
b84b8744e0bb1d9eef349e31c004b8d7d9b7f4ec  crates/kicli/src/lint/score.rs
af2f05575d2a6ad5075179127d21f7c985ec9270  crates/kicli/src/lint.rs
c025d6e0b08cfab3b14dc3db8b922c82586229a7  crates/kicli/tests/the_linter_holds_no_floating_point.rs
0e5e58f4041f8c6105e355f4ce6f16332e3d95e8  crates/kicli/tests/lint_scores_are_bit_identical.rs
```

`crates/kicli/tests/lint_score_normalises_by_density.rs` was strengthened twice
during this table; its final checksum is recorded under B11.

| # | What was broken | Caught by |
|---|---|---|
| B1 | `score_of`: the four lines `exponent`/`grown`/`scaled`/`rounded` replaced by an `f64` `.exp()` | `no_floating_point_appears_under_the_linter`, **and nothing else** |
| B1b | `rounded`: `let half = 0.5;` added — a float literal naming no type | `no_floating_point_appears_under_the_linter` |
| B2 | `Normaliser::ratio`: `PerObject` and `PerWire` both return `(1, 1)` | `a_crossing_weighs_less_on_a_crowded_sheet`, `a_normalised_rule_cannot_take_off_more_than_its_ceiling`, `a_sparse_sheet_with_one_crossing_scores_worse_than_a_crowded_one`, `a_sparse_sheet_with_one_stray_field_scores_worse_than_a_crowded_one`, `a_project_leans_towards_the_sheet_that_holds_the_symbols` |
| B3 | `RawPenalty::weight_of`: the line `.filter(\|finding\| finding.tier == Tier::Two)` removed | `a_blocking_finding_never_moves_the_score`, `a_blocking_finding_leaves_both_sheets_untouched` |
| B4 | `project_score`: the `symbols` closure returns `1` for every sheet | `a_project_weighs_each_sheet_by_its_symbols`, `a_project_leans_towards_the_sheet_that_holds_the_symbols`, the `project_score` doctest |
| B5 | `score_of` returns `100` always | 11 checks, including both determinism arms |
| B6 | `Density::of` returns the default and `RawPenalty::of` returns a constant 3 points | 8 checks, including `a_different_drawing_scores_different_bytes` **alone** in its binary |
| B7 | `const TERMS: u128 = 64` → `3`, so the series is cut short | `the_decay_matches_the_published_function`, `a_project_weighs_each_sheet_by_its_symbols`, the `project_score` doctest |
| B8 | `rounded`: `(numerator * 2 + denominator) / (denominator * 2)` → `numerator / denominator` | 7 checks |
| B9 | `REFERENCE_SYMBOLS` 20 → 40 and `REFERENCE_WIRES` 10 → 20 | `a_sparse_sheet_with_one_stray_field_scores_worse_than_a_crowded_one`, `a_normalised_rule_cannot_take_off_more_than_its_ceiling` |
| B10 | `Density::of`: the guard `if !symbol.is_power()` removed | **green at first — see below**; after the fix, `a_power_symbol_does_not_make_a_sheet_look_crowded` |
| B11 | `Density::of`: `matches!(line.kind, LineKind::Wire)` removed, so a bundle counts as a wire | **green at first — see below**; after the fix, 5 checks |

### B1 is the row that says why the gate exists

The float exponential produced **exactly the same scores** as the fixed point
one on this machine, so all 183 other checks stayed green. **No behavioural
check can see a float that happens to round the same way here** — that is the
whole failure mode, and the sweep is the only instrument that catches it. B1b
is the second half of the same argument: `let half = 0.5;` names no type, so
the type-name arm alone would have let it through and the literal arm is
load-bearing rather than belt-and-braces.

### B2's control is supposed to survive B2

`with_no_normaliser_the_two_sheets_score_the_same` stayed **green** under B2,
and that is correct rather than a hole: it is the anti-vacuity control, and it
asserts an *equality* that a break making everything equal cannot disturb. The
four checks that assert the *inequality* all fired. Recorded because a green row
beside four red ones reads like a gap until the reason is written down.

### B9's surviving checks are directional by design

Doubling both reference counts leaves the sparse sheet un-normalised and the
crowded sheet still divided, so the *direction* of every inequality survives.
Only the two checks that pin exact numbers fired. That is case 1 — the break
changed behaviour, and the checks that survived are the ones that never claimed
to measure the constants.

### B10 and B11 were green, and both were case 2

**Neither was a no-op, and neither was recorded as one.** In both, the code was
innocent and the instrument was blind:

- **B10**: no drawing in any check held a power symbol, so "`N_sym` excludes
  power symbols" was never watched by anything. Fixed by
  `a_power_symbol_does_not_make_a_sheet_look_crowded`, which places six parts
  and six power symbols and **carries its own control**: it asserts twelve
  symbols were placed and six of them carry power, so a fixture that quietly
  stopped holding power symbols would fail rather than pass.
- **B11**: no drawing held a bundle, so "a bundle is not a wire" was never
  watched. Fixed in the generator rather than in one check — every sheet
  `sheet_of` writes now holds one bundle, which turns the existing wire-count
  assertions in `the_two_sheets` into the control.

The contrast is the evidence, and it was measured both ways: B10 and B11 each
ran green against the committed good state, then red against the strengthened
one. Final checksum of the strengthened file:
`24bba8177ced5f2bc032f06fb5beed3e405e9a26`.

### The environment break class

The checks here consume generated values: drawings the probe writes under
`CARGO_TARGET_TMPDIR`, at paths that differ per checkout. The published lesson
is that such a check can be falsifiable against every source break and still be
asserting a property of the directory it ran in.

Run from a second directory, taken out of git rather than copied:

```sh
scratch="$(mktemp -d)"
git archive HEAD | tar -x -C "$scratch"
( cd "$scratch" && cargo test --test lint_score_normalises_by_density \
    --test lint_scores_are_bit_identical --test the_linter_holds_no_floating_point )
```

All three binaries green: 6, 4 (1 ignored, the child), and 4.

### A flake this table found, and the reason it was found

The five density checks originally shared **one pair** of probe drawings.
`cargo test` runs the checks of one binary in parallel, so several threads
wrote and read `density-many/probe.kicad_sch` at once. The **first isolated run
was green.** The first whole-suite run was not, and three separate runs failed
differently:

```
the drawing loads: Unreadable { ... reason: "unclosed list opened at byte 30" }
the drawing loads: Unreadable { ... reason: "no s-expression found" }
the drawing loads: Unreadable { ... reason: "unclosed list opened at byte 244" }
```

Each check now writes its own pair, named after the check. Recorded because the
green isolated run is the trap: a lane that ran only its own test would have
shipped a flake, and what caught it was running the whole suite.

## Completion check

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo xtask check
```

**All six gates pass** in the lane worktree: `fmt`, `clippy`, `test`, `doc`,
`deny`, `clean`. The corpus and environment gated arms are not counted from
here, per the milestone's rules; the orchestrator's merged run is the one that
counts.

The gate named in the task, by name:

```sh
cargo test --test the_linter_holds_no_floating_point
```

Four checks, and shown failing on an injected float twice, once through the type
name and once through a literal that names no type (B1, B1b above).

## Carried, for whoever picks up Phase 2

- The seam a rule needs is `Rule::normaliser()`, per the second PROPOSED above.
  Until it exists, a new rule's normaliser comes from the family in its code, so
  **a rule in a new family divides by nothing** — the strict default, but a
  surprise if it is not expected.
- `RawPenalty` is in **billionths** of a point and `Penalty` is in
  **thousandths**. The two units are one multiplication apart, in
  `BILLIONTHS_IN_A_THOUSANDTH`, and the difference exists because a normaliser
  divides.
- The scorer reads a `Drawing` for the density and a `&[Finding]` for the rest.
  It does not run the engine, so a caller decides which rules ran.


---

## Tick — APPROVE, 2026-08-22

**Reviewer verdict: APPROVE.** Lane `lane-t3`, commit `3e469e6`, base `a8f2057`,
merged to `main` as `327c033`. Pinned at start and finish, unchanged.

### The arithmetic, verified against an independent oracle at 176,200 points

**This entry ships no floating point at all — not even in the final `exp`, which
Constitution §4 would have permitted.** That made the arithmetic the review's
entire burden, because a hand-rolled fixed-point exponential that is subtly
wrong produces plausible scores that are simply not `100·exp(−raw/25)`.

The reviewer built a **60-digit-precision `mpmath` oracle** of
`round(100·exp(−raw/25))`, half-away-from-zero, and a faithful re-implementation
of the shipped `score_of` / `grown_by` / `rounded`, and compared them over:

- every integer raw penalty **0–199**;
- a **0.001-point sweep across 0–150**;
- **20,000 random fractional points** across 0–200;
- **near-zero billionths** (0–2000);
- a window either side of the `EXHAUSTED` cutoff at 150.0.

**Zero disagreements.**

**Overflow was checked rather than assumed**: worst case in `grown_by` is
`term × exponent ≈ 6×10³⁴` against `u128::MAX ≈ 3.4×10³⁸`, and `RawPenalty::of`
would need **~2.6×10²⁶ findings** to overflow.

### The gate has zero exemptions, confirmed by reading it

The reviewer read `the_linter_holds_no_floating_point.rs` in full: **no
`#[allow]`, no file carve-out, no name carve-out** — a whole-word type-name
sweep plus a grammar-complete float-literal sweep over every `.rs` file found by
recursive walk of `src/lint`.

**That matters in this repository specifically**, because another gate was found
this same session classifying by spelling rather than by content
(`probe_harness_has_one_home`). This one does not.

### Four breaks reproduced by the reviewer in its own scratch copy

| Break | Result |
|---|---|
| a float literal `let half = 0.5;` in `rounded` | `no_floating_point_appears_under_the_linter` **failed, naming the exact literal**; the other three in that binary stayed green |
| **B10** — power-symbol guard removed | exactly one check failed (`a_power_symbol_does_not_make_a_sheet_look_crowded`); the other five stayed green |
| **B11** — bundle-as-wire guard removed | exactly the five checks the entry names failed |
| **B3** — the `Tier::Two` filter removed | both named checks failed, with the exact raw-penalty deltas |

### The flake was checked for, not taken on trust

The reviewer confirmed **every probe name in the new files is distinct** —
`density-few-{check}`/`density-many-{check}` keyed by caller, `density-power`,
`density-power-control`, `scored-first/second/sparse/crowded/repeated` — and
**ran the density suite three times back to back, all green.**

### Determinism, weights, and the project score

- The determinism check **crosses a real process boundary**
  (`Command::new(current_exe)` with distinct working directory, `TZ` and
  `LC_ALL`, comparing child stdout bytes), with **both** anti-vacuity controls
  present (`not vacuous`, `not blind`) and passing.
- `cargo test --doc -p kicli` passes, including `project_score`'s doctest
  asserting **40/95/92 weighted — not the unweighted 68.**
- **No rule weight exists in this module at all**, and `K = 25` and the
  reference counts are documented in code as starting points. `RULES.md`'s
  weights discipline is held rather than merely respected.

### Scope

`crates/kicli/src/lint.rs` (module declaration and re-export, **5 lines**,
confirmed by direct diff), `score.rs`, three new test files, and this entry.
**No merge hotspot, no `rules/`, no `src/view/` or `src/cli/`** — the last
mattering because a parallel lane owned those this dispatch. `RuleId::family()`,
which `score.rs` uses, **pre-exists at base `a8f2057`**: T1's file was read, not
written.

> **WORKFLOW NOTE, T3's reviewer, verbatim:** *"The review brief quoted a
> "WORKFLOW NOTE" (crossing-takes-per_wire-not-per_object) as if it should be
> found in the entry; it is not there — it lives in
> `tasks/reports/M5-checkpoint1.md` from a prior review pass and is already filed
> as PROPOSED 13. Future T3-style briefs should say which document a quoted
> WORKFLOW NOTE comes from, since the entry alone (correctly) only carries the
> substance, not the verbatim quote."*

**Accepted, and it is the orchestrator's defect** — a brief that quotes a note
names the document the note lives in. The lane's entry was right to carry the
substance without the quote; the report is where verbatim notes live.
