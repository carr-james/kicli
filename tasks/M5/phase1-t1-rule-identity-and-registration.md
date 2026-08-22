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

---

# Evidence (the implementing lane, `lane-t1`)

## The mechanical check's verdict

**PASS.** Adding a rule is one new file under `crates/kicli/src/lint/rules/`
and no edit to any existing file. Both required arms hold: the file count, and
the rule actually running.

The check as run, on a clean tree at `c64852e`, verbatim.

```
########## 1. BASELINE: git status --porcelain on a clean tree
[end of output]

########## 2. ADD ONE NEW FILE: crates/kicli/src/lint/rules/throwaway.rs

########## 3. git status --porcelain, with the rule file added and NOTHING else touched
?? crates/kicli/src/lint/rules/
[end of output]
----- the same, listing untracked files rather than their directory
?? crates/kicli/src/lint/rules/throwaway.rs
[end of output]

########## 4. does the rule RUN? (no other file edited, no rebuild forced)
running 1 test
registered files: ["throwaway"]
registered rules: [RuleId("KI-TEMP-001")]
finding: KI-TEMP-001 tier=2 warning /00000000-0000-4000-8000-999999999999 0,0 [] "the sheet /00000000-0000-4000-8000-999999999999 was examined" fix=None penalty=1.0
finding: KI-TEMP-001 tier=2 warning /00000000-0000-4000-8000-999999999999/00000000-0000-4000-8000-cccccccccccc 0,0 [] "the sheet /00000000-0000-4000-8000-999999999999/00000000-0000-4000-8000-cccccccccccc was examined" fix=None penalty=1.0
test the_crate_rules_are_listed_with_everything_they_report ... ok

########## 5. the whole seam test binary, with the throwaway rule present
running 6 tests
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

########## 6. REMOVE the file
----- git status --porcelain
[end of output]
----- the output returns
running 1 test
registered files: []
registered rules: []
test the_crate_rules_are_listed_with_everything_they_report ... ok
```

The command in step 4 is the observation window and it is committed:

```sh
cargo test -p kicli --test lint_rules_register_from_their_own_files \
    -- --nocapture the_crate_rules
```

**What each arm would miss without the other.** The file-count arm alone passes
on a registry that compiles and never runs — a `mod` list generated correctly
into a registry nothing reads reports zero findings and shows a clean
`git status`. The output arm alone passes on a hand-edited registry — a rule
whose findings appear because somebody added a line to `lint/rules.rs` runs
perfectly and dirties one tracked file. Step 3 and step 4 are the two halves,
and step 6 is what stops a check that only ever grows.

**The rebuild is part of the answer and was measured, not assumed.** Step 4 ran
`cargo test` with no other change and no `touch`; cargo re-ran the build script
and recompiled. The build script emits `cargo:rerun-if-changed` for `src/lint`
as well as for `src/lint/rules`, because the rules directory does not exist in a
fresh checkout — git does not track an empty directory — and a watch on a path
that is not there is not a watch on the thing that creates it.

## The mechanism chosen, and the ones rejected

**Chosen: a build script that reads the rule directory and writes the module
list and the registry into `OUT_DIR`.** `crates/kicli/build.rs`. No new
dependency, so no Constitution §9 licence question. One generator function, run
over two directories: the crate's own `src/lint/rules/`, and
`crates/kicli/tests/specimen_rules/`, whose registry the seam test measures. The
test therefore measures the mechanism that ships rather than a copy of it.

Generated per directory: one `#[path = "…"] mod` declaration per file, a
`BY_FILE` table pairing each file's name with the rules it declares, `all()` and
`files()`. A rule file declares `pub static RULES`, so one file may hold a family
of rules that share a definition — which is what `lint/rules/labels.rs` will need
for `KI-LBL-001/002`.

| Rejected | Why |
|---|---|
| a distributed-slice crate (`inventory`, `linkme`) | **It does not solve the problem.** Rust compiles no file that no `mod` declaration names, so a distributed slice still needs the per-rule `mod` line the check is about. It would have bought a new dependency and a §9 licence check for nothing. Not raised with the orchestrator, because the answer did not depend on the licence. |
| `include!` of a generated registry **into the module tree** | This is what was built, and the entry lists it separately from the build script only because the two are usually named as alternatives. The interaction with `#![deny(missing_docs)]` is real and is handled: the generator writes a doc comment above every public item it emits, which was found by the compiler refusing the first draft. |
| a hand-written `mod` list in `lint/rules.rs` | The FAIL branch. Costed below rather than guessed at. |
| a macro that expands to the registry | `ENGINEERING.md`'s DRY caveat forbids it in as many words, and it would not have worked anyway: a macro cannot enumerate a directory. |

## What the FAIL branch would have cost, measured rather than argued

Recorded because the plan's re-cut turns on it, and because a PASS that hides
the alternative's price is worth less at a checkpoint.

**The minimum edit would have been one line per rule file** — `mod grid;` in a
`crates/kicli/src/lint/rules.rs` holding nothing else, kept in alphabetical
order. **Mechanical, not a judgement.** No match arm, no registration call, no
weight table: the `BY_FILE` table the generator writes could have been written by
hand from the same `mod` list, so a rule author would have edited exactly one
line in exactly one existing file.

That is the cheap end of the range the plan feared, not the expensive end.

## The cost this mechanism does carry, measured

**`cargo fmt --check` cannot see a rule file, and `cargo fmt --check` is one of
the six gates.** `rustfmt` walks the module tree from the crate root and does not
follow an `include!`, so the `#[path]` declarations inside the generated file are
invisible to it. Measured on a scratch crate before any of this was written:

```
=== cargo fmt --check ===        (over a deliberately mangled rule file)
fmt exit=0
       0 /tmp/fmtout             (no diff reported at all)
=== rustfmt --check directly on it ===
rustfmt exit=1
```

`cargo clippy --all-targets` **is not** affected — it lints a rule file
normally, measured in the same scratch crate (`dead_code` reported against
`src/rules/alpha.rs:9`).

The refund is `crates/kicli/tests/rule_files_are_formatted.rs`, which hands the
same files to `rustfmt` directly. `rustfmt` is a pinned component in
`rust-toolchain.toml`, so its absence is a broken environment rather than a
reason to skip.

**PROPOSED (lane-t1): the loss of a gate's reach is worth recording at the
checkpoint even though it is repaired.** Recommendation: accept, because the
repair is executable, runs inside `cargo test`, and is shown capable of failing.
The alternative reading is that a seam which blinds one of six gates is a seam
that should not ship; that reading is available to James and the measurement
above is what it would rest on.

## Scope: one file outside the brief's IN list

`crates/kicli/build.rs` is new and was not on the brief's IN list. It is not a
merge hotspot either — it did not exist. The task entry names a build script as
a candidate mechanism in its own table, so the goal state contemplated it and
the enumeration under-counted; the brief's own rule is that the named goal state
wins over the list. `Cargo.toml` is **not** touched: cargo detects `build.rs` at
the package root with no manifest key.

Nothing else outside the list was touched. `crates/kicli/src/lib.rs` already
declared `pub mod lint;` and was not edited.
