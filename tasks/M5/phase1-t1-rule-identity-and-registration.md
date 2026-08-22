# The finding, the rule identity, and the registration seam (Phase 1, T1)

**Provenance: `tasks/M5/PLAN.md` Phase 1, RATIFIED by James's ratification and
advisor rulings, M5 plan review.** The plan's own words: *"the first task is not
a rule. It is the rule identity and registration seam."*

**This task's verdict gates the milestone.** `PLAN.md`'s design question is
settled here or its lane table is fiction, and the ruling adds a second gate on
top: **no Phase 2 lane is dispatched until James ratifies this verdict at the
checkpoint.**

## Why this is first, and why its answer may be "no"

`ENGINEERING.md` (Structure, Open/closed): *"adding a new lint rule or a new
schematic object type must not require editing unrelated modules."* There are
**six Tier 1 rules and twenty-two Tier 2 rules** (`spec/SPEC.md` §11.4). If
rules register through one shared list, that list is a merge hotspot every lane
in Phases 2 and 3 queues on, and the milestone serialises whatever the lane
table says.

**A FAIL here is a finding, not a setback, and it is explicitly permitted.** The
plan: *"If that check cannot be made to pass, the lane table shrinks to two lanes
and the plan is re-cut — better to learn that in Phase 1 than at the third merge
conflict."* **Do not bend the check to reach PASS.** A seam reported as passing
that a Phase 2 lane then finds requires touching `lint/mod.rs` costs far more
than an honest FAIL costs now.

## Goal state, as the checks that prove it

### 1. The finding type, per §11.3, exactly

```json
{ "rule": "KI-FLOW-001", "tier": 2, "severity": "warning",
  "sheet": "/Power", "pos": {"x": 123.19, "y": 45.72},
  "objects": ["uuid…"], "message": "Power symbol +3V3 points down",
  "fix": "kicli sym rotate <uuid> --to 0", "penalty": 3.0 }
```

- `fix` is a **suggested command**. §11.3: *"kicli never mutates during
  scoring."* Nothing in `lint` may hold a write path, and that is worth an
  executable twin of its own if one is cheap.
- `tier` is 1 or 2. **Tier 3 is cut from scoring entirely** (§11.4) — it is not
  a variant to carry "for completeness".
- `pos` carries the finding's position in the same units the views print, and
  the number in it is **not** how detection was done. Constitution §4: detection
  is integer geometry only.

### 2. Findings sort by `(rule, sheet, x, y, uuid)` before output

§11.5, Determinism. This is a total order and it must actually be total —
two findings identical in all five terms are the same finding, and if that is
not true, say so rather than adding a sixth term silently.

### 3. Re-scoring an unchanged file is bit-identical

§11.5. The existing precedents are `the_router_iterates_no_hash_map.rs` and
`route_determinism.rs`; **read both before writing this one.** A determinism
check that runs the same call twice in one process is weaker than one that
crosses a process boundary, and the router's tests already know that.

### 4. THE MECHANICAL CHECK — adding a rule touches one new file and no existing one

**This is the task's verdict, and it is answered by measurement, not argument.**

The check is executable and it is the deliverable:

- add a throwaway rule as **one new file** under `lint/rules/`;
- **`git status --porcelain` shows exactly one added path and no modified
  path**;
- the rule's findings appear in the engine's output — a registration that
  compiles but never runs is a PASS on the letter and a FAIL on the point;
- remove it, and the output returns to what it was.

**Both arms are required.** A check that only counts files can pass while the
rule does nothing; a check that only looks at output can pass while the rule
list is hand-edited. State in the entry what each arm would miss without the
other.

**Candidate mechanisms, none of them ruled, all of them constrained:**

| Mechanism | The constraint on it |
|---|---|
| a distributed-slice crate (`inventory`, `linkme`) | **new dependency ⟹ Constitution §9 licence check**, and the check is part of the task, not a footnote. GPL-3-compatible only. |
| a `build.rs` that globs `lint/rules/*.rs` and generates the `mod` list and the registry | no new dependency; generated code must still be readable and the generation must be visible in the check |
| `include!` of a generated registry | same, and it interacts with `#![deny(missing_docs)]` |
| accept that `mod` lines are unavoidable and report FAIL | **a legitimate outcome**, and the plan already names the consequence |

**ENGINEERING.md's DRY caveat governs whichever you pick**: *"Never macro-away
duplication that a plain function can remove."* A macro that exists to defeat a
counting check rather than to remove duplication is the wrong answer even when
the counter goes green.

### 5. `lint` knows nothing of the CLI, files on disk, or `kicad-cli`

`ENGINEERING.md`, Dependency inversion. Pure functions over the model wherever
possible.

## Falsification obligation

Per `.claude/skills/falsification-control/SKILL.md`. **Three shapes apply and
each has a specific trap here:**

- **The mechanical check is the one most likely to be blind.** Show it failing:
  make a rule that requires editing an existing file and confirm the check goes
  red. A green counter over an unexercised path is the exact failure mode this
  whole task exists to avoid.
- **The determinism check is a degenerate-equality candidate.** Two scores
  computed by the same call in the same process share every ancestor. State what
  the two sides are derived from. If they share one, the check watches nothing.
- **The sort check is vacuous on a one-finding fixture.** It needs findings that
  actually contend on each term — same rule different sheet, same sheet
  different x, and so on down to the uuid tiebreak.

## Scope

**IN**
- `crates/kicli/src/lint.rs` and everything new under `crates/kicli/src/lint/`
- new test files under `crates/kicli/tests/` for the checks above
- `crates/kicli/tests/fixtures/**` — new fixtures only, **and see the note on
  `MANIFEST` below**
- this file, for the evidence, which you write AS YOU WORK

**MERGE HOTSPOTS — report, do not edit.** `Cargo.toml`, `crates/kicli/src/lib.rs`,
the fixture `MANIFEST`, `AGENT.md`, `spec/SPEC.md`, `crates/kicli/tests/command_surface.rs`.
`lib.rs` already declares `pub mod lint;`, so you should not need it. **If your
mechanism needs `Cargo.toml`** — a distributed-slice crate does — **stop and
report before adding it**, because that is a dependency and a §9 licence check,
and the orchestrator owns that file.

**OUT** — every other module, every other task's entry, `tasks/M5/PLAN.md`.

**If the enumeration above proves wrong, the named goal state and its checks win
over the list.** Say so in your first paragraph, name what you touched and why.
Provenance: PROPOSED 5, M4 close, promoted.

## Evidence obligations

- **The mechanical check's verdict, PASS or FAIL, stated in one line at the top
  of your evidence section**, with the `git status --porcelain` output that
  produced it, pasted verbatim.
- If FAIL: **what the minimum edit to an existing file is**, precisely — which
  file, how many lines, and whether it is mechanical or a judgement. The re-cut
  depends on the difference between "one `mod` line per rule" and "a match arm
  per rule".
- The falsification table, per the skill.
- The mechanism you chose and the ones you rejected, with the reason each was
  rejected — this is the record James ratifies from.

## Completion check

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo xtask check
```

plus the mechanical check itself, run and pasted:

```sh
# with a throwaway rule file added under lint/rules/
git status --porcelain
```

**A FAIL on the mechanical check does not fail this task.** The task is done
when the seam exists, the checks above pass, and the verdict is recorded with
its evidence.
