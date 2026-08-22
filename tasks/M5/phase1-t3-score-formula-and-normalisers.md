# The score formula and the normalisers (Phase 1, T3)

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
