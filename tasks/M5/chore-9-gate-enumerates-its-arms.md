# The gate says what it did NOT run (M5, PROPOSED 5 promoted)

**Provenance: James's ruling promoting the fourteen checkpoint-1 PROPOSED items
per their recorded recommendations, checkpoint 1 review.** PROPOSED 5 of
`tasks/reports/M5-checkpoint1.md`.

**Dispatched to `lane-implementer`, not `chore-runner`, deliberately.** The
chore-runner definition says *"never design work"*, and the shape of the fix is
a design decision — see "The trade, decided" below. Recorded so the routing is
visible rather than looking like a slip.

## The measurement

`CLAUDE.md` binds the orchestrator: *"The orchestrator runs the full check,
**corpus included**, at every lane merge."* **The command that reports on the
gates does not include it.** `xtask/src/main.rs`'s `GATES` table:

| Gate | Args |
|---|---|
| `clippy` | `clippy --all-targets --all-features -- -D warnings` |
| `test` | **`test`** — bare. No `--features corpus`, no `KICLI_TEST_KICAD_CLI` |

So the corpus code is **compiled** by clippy's `--all-features` and **never run**
by the test gate. The whole `mod corpus` block in `net_oracle.rs` sits behind
`#[cfg(feature = "corpus")]` and is simply **absent from the binary the `test`
gate builds**.

| Command | Binaries | Tests passed | Ignored |
|---|---|---|---|
| `cargo test -p kicli` (what the gate runs) | 71 | **529** | **0** |
| `cargo test -p kicli --features corpus` + `KICLI_TEST_KICAD_CLI=1` | 71 | **535** | **0** |

**Six tests** — not a chasm, and saying otherwise would be the same overclaim
this entry is about. But those six check kicli's answers **against KiCad
itself**, over 115 real schematics and 35 hierarchies, and they are the only
checks that can catch the extractor drifting from the thing it models.
`RULES.md` makes one of them a standing milestone gate in its own right.

**The sharper half is the `0 ignored` column, which is identical in both rows.**
The six missing tests do not appear as skipped, or ignored, or filtered. They
are **invisible rather than skipped**, and the bare run's summary is internally
consistent and completely silent about them.

**How close this came to being reported wrong**: the orchestrator ran
`cargo test -p kicli --test net_oracle`, got `2 passed; 0 failed` — **a green** —
and caught it only because the run finished in **0.00s**. 35 hierarchies
including one with 2,353 nets cannot be checked in no time. Run correctly it is
**five tests and sixteen seconds**, reporting `hierarchies matched: 35/35`.

**And note what was NOT broken.** The previous session's report records running
the corpus and environment arms by hand, so the practice has been correct
throughout. **That is exactly why this is worth fixing: the correctness depends
on the orchestrator remembering, and nothing fails if one forgets.**

## The trade, decided — and the deviation from the recommendation is deliberate

PROPOSED 5's recorded recommendation was *"the `test` gate gains a second arm,
`cargo test --features corpus`, reported as its own line in the summary"*, and
it named the cost as **James's, not the orchestrator's**: the corpus arm is
**~16 minutes** of wall clock against the bare gate's seconds, and
`cargo xtask check` **runs as a git pre-commit hook**. Making the arm
unconditional puts sixteen minutes on **every commit in the repository**.

**Decided: the second arm exists and is opt-in; the ENUMERATION is
unconditional.** That is the recommendation's second arm and its own named
"cheaper third option" taken together, and it is the only combination that
closes the invisibility without changing what committing costs.

*Labelled a deviation and flagged for James rather than absorbed, because the
recommendation's primary form was the unconditional arm.* If he wants it
unconditional, that is one boolean.

## Goal state, as the checks that prove it

### 1. A gate line has THREE verdicts, not two

`pass`, `FAIL`, and **`skip`** — and a `skip` line **names the reason and the
exact command that un-skips it**. This is the general rule already promoted into
`orchestrator.md`: *offer three verdicts wherever you offer two.* Today the
summary can only say `pass` or `FAIL`, so an arm that did not run has no way to
say so and is reported by silence.

### 2. `cargo xtask check` enumerates EVERY arm, including the ones it did not run

Unconditionally, with no flag. The summary must let a reader answer **"what
would a full run have been?"** from the output alone. That question is the whole
finding: *nothing anywhere states what a full run would have been.*

At minimum the corpus arm and the `kicad-cli` environment arm each get a line,
marked `skip` with the reason and the command, whenever they did not run.

### 3. `cargo xtask check --corpus` runs them

- the corpus arm: `cargo test --features corpus`;
- with `KICLI_TEST_KICAD_CLI=1` **when `kicad-cli` is discoverable**, and a
  `skip` line naming its absence when it is not. **The environment arm cannot be
  made unconditional** — `kicad-cli` genuinely may be absent, which is what
  `Kicad::found_or_skip` exists for. What it can do is **say so**.
- If the corpus is not fetched, `--corpus` **fails loudly with the fetch
  command**, rather than passing on nothing. A flag that asks for the corpus and
  silently gets no corpus is this same defect with a new name.

### 4. `all gates passed` never appears over a run with a skipped arm

It is true of the gates that ran, and a reader has no way to see which did not.
Whatever replaces it must be false when something was skipped. **This is the
exact failure M4's calibration row taught** — *"a gate presented as measuring
something it cannot fail on is worse than no gate, since it spends the
credibility of a real one"* — with the twist that here **the gate is real and the
summary line over it is what overclaims.**

## Falsification obligation

Per `.claude/skills/falsification-control/SKILL.md`. Each of these is shown, and
the result recorded here:

- **The enumeration is shown failing**: remove an arm from the enumeration and
  confirm a check goes red. An enumeration nothing counts is a list that drifts.
- **The `skip` verdict is shown**: run `check` with no corpus fetched and paste
  the summary. Then run it with the corpus and paste that summary. The two must
  differ in a way a reader cannot miss.
- **`--corpus` on an unfetched corpus is shown failing**, per goal state 3.
- **Beware the degenerate check**: a test asserting the summary contains the word
  `skip` passes if every line says `skip`. State what your check would catch that
  a substring match would not.

## Scope

**IN**
- `xtask/**` — this is the one lane designated to touch `xtask`, which is
  otherwise a merge hotspot. That designation is this brief.
- new test files under `xtask/` for the checks above
- this file, for the evidence, written AS YOU WORK

**OUT** — `crates/**` in its entirety (two lanes are live there), every other
task entry, `ENGINEERING.md`, `CLAUDE.md`. **If the documentation needs to change
to match, say what and where; the orchestrator writes it.**

**If this scope proves wrong, the named goal state and its checks win over the
list.** Say so in your first paragraph, name what you touched and why.

## Completion check

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo xtask check
```

— which is now partly the thing under test, so it is **not sufficient on its
own**. Also required, pasted into this entry:

```sh
cargo xtask check          # summary with the corpus arm enumerated and skipped
cargo xtask check --corpus # summary with it run
```

both summaries, verbatim, plus the falsification results above.

## Tick — APPROVE (awaiting merge at the checkpoint-2 stop)

Lane `d083f9e`, base `047783b`. **Merged: no** — session stopped on a usage limit.
Scope verified: 4 files, nothing under `crates/`.

The reviewer reproduced seven falsification rows including break 6, the named
degenerate one, and confirmed **only `each_arm_reports_its_own_verdict`** catches
it. It confirmed the entry's correction of the orchestrator's own number: the
corpus arm adds **11** checks workspace-wide, not 6, because `kicli-sexpr`
carries its own `corpus` feature — and `ignored` is **2 in both runs**, so that
column still cannot tell a reader what did not run.

**Two under-claims found, both in the safe direction**: the entry says nineteen
checks where the binary has **21**, and its "verbatim" bare-run transcript
**dropped the `clean` arm's pass line**. A transcript labelled verbatim that is
not one is worth fixing in a task whose whole subject is a summary that said
something untrue about itself.

**Documentation owed, the orchestrator's:** `ENGINEERING.md`'s gate fence
(missing `clean`, `corpus`, `kicad-cli` — **and it is now an INPUT to
`every_documented_gate_is_an_arm`, compared line-for-line, so an innocuous doc
edit can break the build**), `.githooks/pre-commit` line 2, `CLAUDE.md`'s
*"corpus included"*, and `RULES.md`'s copy of it.
