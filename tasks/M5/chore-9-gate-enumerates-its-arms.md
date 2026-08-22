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

---

# Evidence and deliverable (lane gate)

Written as the work was done. Lane branch `lane-gate`, base `047783b`
(verified as the lane's first action: `git log --oneline -1` reported
`047783b tasks: PROPOSED 5 becomes an entry — the gate says what it did NOT run
(M5)` and `git status --porcelain` reported nothing).

## What was built

`xtask/src/gate.rs` is new. It holds `ARMS`, the whole enumeration, and the
summary that renders it. `xtask/src/main.rs` runs the arms and records one
verdict per arm. `xtask/src/corpus.rs` gained `root()`, `which()` and
`fetched()`.

**The enumeration is now the single table `ARMS`, and it drives both the run
and the summary.** The old shape had two lists: `GATES` and, chained on by hand
in two places, `CLEAN`. A `skip` verdict living in that shape would have needed
the same hand-chaining a third and fourth time, and a list maintained in four
places is the drift this entry is about.

Eight arms, in run order: `fmt`, `clippy`, `test`, `doc`, `deny`, `corpus`,
`kicad-cli`, `clean`. The corpus arm runs before `clean` so the working-tree
comparison covers it.

**PROPOSED: `all gates passed` is deleted rather than reserved for the full
run.** Goal state 4 requires a headline that is false when an arm was skipped.
The headline is now one of three, and the two non-green forms name counts:

- `COMPLETE: all 8 arms passed. This was a full run.`
- `INCOMPLETE: N of 8 arms did not run. M passed, 0 failed.`
- `FAILED: N of 8 arms failed. M passed, K skipped.`

Recommendation: keep the deletion. Nothing in the repository consumed the old
string — `grep` finds it only in task files and reports, never in a script, a
hook or a CI job. Reserving it for the full-green case would have worked too;
deleting it means no reader can ever have seen it over a partial run.

## Both summaries, verbatim

### `cargo xtask check` — the corpus arm enumerated and skipped

Exit code 0.

```
=== summary ===
  pass  fmt        cargo fmt --check
  pass  clippy     cargo clippy --all-targets --all-features -- -D warnings
  pass  test       cargo test
  pass  doc        cargo doc --no-deps
  pass  deny       cargo deny check
  skip  corpus     cargo test --features corpus
                   did not run: --corpus was not given, so the `corpus` feature stayed off
                   runs with: cargo xtask check --corpus
  skip  kicad-cli  KICLI_TEST_KICAD_CLI=1 cargo test --features corpus
                   did not run: --corpus was not given, so KICLI_TEST_KICAD_CLI stayed off
                   runs with: cargo xtask check --corpus

INCOMPLETE: 2 of 8 arms did not run. 6 passed, 0 failed.
This was not a full run. The skip lines above say what runs each one.
```

Every arm prints its command whether it ran or not, so "what would a full run
have been?" is answerable from this block alone. That question was the finding.

### `cargo xtask check --corpus` — the corpus arm run

Exit code 0. `kicad-cli` was on `PATH` (`/opt/homebrew/bin/kicad-cli`), so the
oracle arm rode the corpus run.

```
=== summary ===
  pass  fmt        cargo fmt --check
  pass  clippy     cargo clippy --all-targets --all-features -- -D warnings
  pass  test       cargo test
  pass  doc        cargo doc --no-deps
  pass  deny       cargo deny check
  pass  corpus     cargo test --features corpus
  pass  kicad-cli  KICLI_TEST_KICAD_CLI=1 cargo test --features corpus
  pass  clean      git status --porcelain, before the arms and after them

COMPLETE: all 8 arms passed. This was a full run.
```

**The two summaries differ in a way a reader cannot miss**: two `skip` marks
against two `pass` marks, six explanatory lines present against absent, and a
headline that says `INCOMPLETE … This was not a full run` against one that says
`COMPLETE … This was a full run`. Neither can be mistaken for the other at a
glance, which was the requirement.

**What the corpus arm actually added, measured from the two transcripts.** Both
arms build the same 97 test targets. The bare `test` arm ran **719 passed, 0
failed, 2 ignored**; the `corpus` arm ran **730 passed, 0 failed, 2 ignored**.

*Task text yields to measured reality, and two numbers in "The measurement"
above are refined by this run rather than contradicted.* That section counted
**six** tests, from `cargo test -p kicli` alone. Across the workspace the
difference is **eleven**, because `kicli-sexpr` carries a `corpus` feature of
its own. The seven corpus-gated checks that ran by name:

```
test corpus::every_corpus_hierarchy_loads ... ok
test corpus::no_corpus_hierarchy_mixes_bundle_kinds ... ok
test corpus::netlist_partition_matches_kicad_corpus ... ok
test corpus::views_stay_within_byte_ceilings_corpus ... ok
test emit_reproduces_input_bytes_corpus ... ok
test prettify_reproduces_kicad_layout_corpus ... ok
test reparse_preserves_tree_corpus ... ok
```

**And the `ignored` column is 2 in both**, unchanged by eleven extra checks —
the entry's sharper half, re-measured at workspace scale after the fix. The
count of ignored tests still cannot tell a reader what did not run. Only the
summary can, which is why the summary is where the fix went.

**Cost, measured rather than estimated.** The warm run above took **524 s**
end to end, eight arms included. The entry's "~16 minutes" is the right order
and the shape is worth recording: `--features corpus` is a different Cargo
fingerprint, so the **first** `--corpus` run on any checkout pays a full rebuild
of all 97 test targets before a single corpus check runs. A reader who types
`--corpus` for the first time should expect the rebuild, not just the tests.

### An unplanned demonstration of the third headline

The first `--corpus` attempt reported, verbatim:

```
  FAIL  clean      git status --porcelain, before the arms and after them

FAILED: 1 of 8 arms failed. 7 passed, 0 skipped.
```

Cause: this evidence section was being appended **while the run was in
progress**, so the tree the `clean` arm compared before and after was not the
same tree. The `clean` arm is doing exactly its job, and the run was repeated
with the tree untouched to produce the summary above.

Recorded rather than quietly discarded for two reasons. It is the `FAILED`
headline shown by a real run rather than by a unit test, which no other
evidence here provides. And it is an operational fact the next person needs:
**`cargo xtask check` cannot be run while anything edits the working tree** —
including an agent writing up its own task entry. That was true before this
change and is not new; it is merely easy to walk into.

## The enumeration is counted by something that is not the enumeration

The obvious check — "assert every arm in `ARMS` appears in the summary" —
**cannot fail**, because `ARMS` drives the summary. Removing an arm removes it
from both sides and the check stays green. That is the shared-ancestor kind of
blind instrument, wearing an enumeration's clothes.

So the expectation is derived from three files this module does not own:

| Check | Derived from | Owned by |
|---|---|---|
| `every_documented_gate_is_an_arm` | the fenced gate block under `## Machine-enforced gates` in `ENGINEERING.md` | a governing document, out of this lane's scope |
| `every_declared_feature_is_an_arm` | the `[features]` sections of `crates/*/Cargo.toml` | the crate authors |
| `every_test_environment_variable_is_an_arm` | every `KICLI_TEST_*` name read anywhere under `crates/**/*.rs` | the test authors |

Each carries a presence control, because a sweep that read nothing passes
otherwise: the first asserts it parsed at least five gate lines, the other two
assert they found at least one name. Break 9 in the table below is that control
firing.

**The boundary, stated rather than implied.** The `clean` arm has no
independent source: the working-tree comparison is xtask's own invention and no
file outside `xtask/` names it. It is covered instead by
`every_kind_of_arm_is_enumerated`, which walks the `Run` enum — an enumeration
the compiler maintains — and demands an arm for each variant. That is weaker
than the other three, and saying so beats an honest-looking check that is one
category behind.

Note what the `ENGINEERING.md` check also buys, which was not asked for: it
compares the **exact command**, so an arm whose arguments drift from the
documented contract fails too. Break 5 is that case.

## Falsification table

Good state committed first, at `12943a8`, and anchored by content hash rather
than by that SHA: `995db3b715ae24e84ff046600ba3d6c9401fc9f5  xtask/src/gate.rs`.
Every break was applied to that state, and the file was restored and
re-checksummed against that hash **before** each break and after the last.

Every row is a whole-workspace `cargo test --workspace --no-fail-fast` run, so
the caught-by lists are complete rather than truncated at the first failing
target. **In every row only the `xtask` binary's checks fired**, which is the
expected result — `gate.rs` compiles into no other target — recorded because
"expected" is not the same as "measured".

| # | What was broken | Caught by |
|---|---|---|
| 1 | the `doc` arm deleted from `ARMS` | `every_documented_gate_is_an_arm`: *"ENGINEERING.md lists `cargo doc --no-deps` and no arm runs it"* |
| 2 | the `corpus` arm deleted from `ARMS` | `every_declared_feature_is_an_arm`; `every_kind_of_arm_is_enumerated`; `a_skip_line_names_its_reason_and_the_command_that_runs_it`; `a_skipped_arm_still_states_what_a_full_run_would_have_been` |
| 3 | the `kicad-cli` arm deleted from `ARMS` | `every_test_environment_variable_is_an_arm`; `every_kind_of_arm_is_enumerated`; `a_skip_line_names_its_reason_and_the_command_that_runs_it` |
| 4 | the `clean` arm deleted from `ARMS` | `every_kind_of_arm_is_enumerated`: *"the run can perform a Tree arm and no arm of that kind is enumerated"*; `an_arm_with_no_verdict_is_reported_as_a_bug` |
| 5 | the `test` arm kept, its `command` changed to `cargo test --lib` | `every_documented_gate_is_an_arm` |
| 6 | `write_arms` renders a `Pass` arm's mark as `skip` | `each_arm_reports_its_own_verdict` — **and nothing else** |
| 7 | the headline forced to `COMPLETE` whatever the tally | `one_skipped_arm_makes_the_run_incomplete`; `one_failed_arm_makes_the_run_failed`; `an_arm_with_no_verdict_is_reported_as_a_bug` |
| 8 | `no_kicad_cli` returns the same `why` text as `not_requested` | `a_skip_line_names_its_reason_and_the_command_that_runs_it` |
| 9 | the `ENGINEERING.md` parser pointed at a heading that does not exist, so it reads nothing | `every_documented_gate_is_an_arm`, at its **presence control** rather than at a per-gate assertion |
| 10 | the `runs with:` line dropped from every skip block | `a_skip_line_names_its_reason_and_the_command_that_runs_it` |

Rows 1 to 5 are the enumeration shown failing, which was the obligation. Rows 4
and 5 also record two things worth naming: deleting the `clean` arm is caught by
**two** checks and by neither of the two derived from repository files, which is
the boundary above measured rather than asserted; and an arm can be present and
still wrong, which row 5 is.

## The degenerate check, named and avoided

The obligation names it: *a test asserting the summary contains the word `skip`
passes if every line says `skip`.* Break 6 is exactly that renderer — every
`Pass` arm printed with a `skip` mark — and it was run to find out which checks
survive it:

- `text.contains("skip")` — **would pass.** Every line says skip.
- `every_arm_appears_exactly_once` — **passed.** It matches on the name column
  and never reads the mark.
- `a_full_run_is_the_only_run_called_complete` — **passed.** The headline is
  computed from the tally, not from the rendered marks, so the two are
  independent and the break moved only one of them.
- `each_arm_reports_its_own_verdict` — **failed**, and it was the only one.

What it catches that a substring match does not: it gives **every arm a
different verdict in one summary**, then reads each arm's own line back and
compares the mark against the verdict that arm was given. So it fails a
renderer that ignores its input, a renderer that prints one blanket mark, and a
renderer that puts one arm's verdict on another arm's line. A substring match
sees none of the three, because all three still contain the word.

The complement is deliberate: `a_full_run_is_the_only_run_called_complete`,
`one_skipped_arm_makes_the_run_incomplete` and `one_failed_arm_makes_the_run_failed`
watch the headline, and break 6 leaving them green is the evidence that the mark
and the headline are two independent claims rather than one claim asserted twice.

## The probe-name hazard, checked

The falsification skill's second new line — *two checks in one test binary must
never share a probe name, because the binary runs them in parallel against a
name-keyed path* — has no probe drawings here, but it has the same shape: the
three `corpus::tests` checks each create a scratch directory and each empties it
first. They take **distinct** names — `corpus-empty`, `corpus-partial`,
`corpus-whole`, all under `target/xtask-scratch/` — so the parallel run cannot
have one check delete another's directory. The nineteen checks in the binary
have nineteen distinct names.

## `--corpus` on an unfetched corpus, shown failing

The lane worktree had no `target/corpus` at all, so this is the natural state
rather than a contrivance. Verbatim, exit code **1**, and **no arm ran**:

```
xtask: --corpus asks for the corpus and /Users/james/code/kicli/.claude/worktrees/lane-gate/xtask/../target/corpus/demos is not there, so the tests cannot read the canonicalised demo schematics.
xtask: run `cargo xtask corpus` first, then run this again.
xtask: no arm ran.
```

The refusal is up front, before `fmt` runs, because the alternative is telling
the reader after two and a half minutes of gates they did not ask about.

`corpus::fetched` requires **three** directories, not one, because three
different test files read three different roots: `demos` (the netlist oracle,
the `kicad_pro` fidelity sweep, the calibration fixtures and the s-expression
corpus), `qa` (the s-expression corpus's regression arm) and `kicad/demos` (the
view budget sweep). A check that looked only at `demos` would pass a corpus
that silently starves two of those three.

That requirement is itself falsified, by
`corpus::tests::every_directory_the_tests_read_is_required`: it builds the
corpus three times, each time with a different one of the three missing, and
demands the answer name the missing one. A one-directory check passes two of
those three cases. `a_corpus_holding_all_three_directories_is_fetched` is the
presence control beside it — without it, "not fetched" could be the function's
only answer and the other two checks would still be green.

## Disclosure: how the corpus reached this worktree

The lane worktree's `target/corpus` holds **symlinks** to the main checkout's
already-fetched `demos`, `qa` and `kicad`, plus a **real, local** `oracle`
directory. Fetching properly is a 1.3 GB clone of KiCad plus a canonicalisation
pass, for a corpus this lane only reads.

The one directory the corpus tests **write** to is `<corpus>/oracle`
(`kicli-sexpr`'s `output_matches_kicad_writer` writes its scratch copies there),
and that one is local, so this run wrote nothing into the main checkout.
`target/` is untracked, so nothing about this appears in the diff.

## Documentation this now owes, and where — the orchestrator writes it

Nothing outside `xtask/**` was touched. Four places say something that this
change makes wrong or incomplete:

**1. `ENGINEERING.md`, "Machine-enforced gates (run all of them; all must
pass)".** The fence lists five cargo commands. Three arms are missing from it:
`clean`, `corpus` and `kicad-cli`. `every_documented_gate_is_an_arm` is
therefore **one-directional** — every documented gate must be an arm; not every
arm must be documented — and that direction is the only one that can be true
today.

**Read this before editing that fence: the fence is now an input to a check.**
`every_documented_gate_is_an_arm` parses it and compares each line against an
arm's `command` string exactly. A new line that does not match an arm fails the
`test` arm. That coupling is the point — the document and the gate cannot drift
— but it means the fence is no longer free-form prose.

**PROPOSED, two options, recommendation second.**

- *Option A — prose, no fence change.* Add a paragraph after the fence: the
  five above always run; `cargo xtask check` also compares the working tree
  before and after; two further arms, the corpus tests and the `kicad-cli`
  oracle, are opt-in behind `cargo xtask check --corpus`; every arm is listed in
  the summary whether it ran or not. The check stays one-directional.
- *Option B — extend the fence and make the check bidirectional.* Add
  `cargo test --features corpus` and
  `KICLI_TEST_KICAD_CLI=1 cargo test --features corpus` to the fence, and then
  assert both directions in one added loop.

**Recommendation: A.** Option B cannot cover the `clean` arm without putting
`git status --porcelain, before the arms and after them` — which is not a
command anybody types — into a fence of commands, and a fence that is partly
commands and partly descriptions is worse documentation than a paragraph.

**2. `.githooks/pre-commit`, line 2**: *"Gates as physics: the six-gate suite
runs before every commit."* Six is now the count of arms that **run** by
default, out of eight that are **reported**. Suggested wording: *"the gate
suite runs before every commit; the two opt-in arms are reported as skipped."*

**3. `CLAUDE.md`, "Parallel work"**: *"The orchestrator runs the full check,
**corpus included**, at every lane merge."* That sentence now has an executable
form and should name it — `cargo xtask check --corpus` — which is the whole
point of the change. The adjacent line, *"A lane is complete when its own
`cargo xtask check` passes in its worktree"*, is still exactly right and should
stay as it is: a lane's run is meant to be the one that skips those arms.

**4. `tasks/M5/RULES.md`, "Rules for this milestone"** restates the `CLAUDE.md`
sentence and takes the same repair.

Not owed a change, listed so a reader does not go looking: the many task entries
recording *"all six gates pass"* are history, true when written.

## PROPOSED: `--corpus` runs the suite twice, and that is deliberate

Under `--corpus` the workspace suite runs once bare (the `test` arm) and once
with `--features corpus` and `KICLI_TEST_KICAD_CLI=1` (the `corpus` arm). The
second is not a superset in the sense that matters: the bare arm is the only
run that proves the **hermetic** configuration — no feature, no KiCad install —
still passes, and that configuration is what the pre-commit hook and every
other machine actually run.

Recommendation: keep both. The cost is paid only by the person who typed
`--corpus`, and collapsing them would mean the hermetic run is never measured
on the machine that has KiCad installed.
