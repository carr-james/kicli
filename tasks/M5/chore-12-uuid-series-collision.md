# The uuid series is a global allocation recorded nowhere, and three lanes collided in it (M5)

**Provenance: the orchestrator, at the Phase 2 close. Found by the merged check
at `e06e117` — `main` is RED and this is the repair.**

**It is a gate failure found after a tick, and it is the first of this
milestone.** It is also **nobody's lane defect**: no lane could have seen it.

## The measurement

`cargo test -p kicli --test fixture_handles`:

```
every uuid atom of the committed fixture tree must answer to a handle of its
own, or no fixture can exercise a command addressed by one.
1395 atoms share 1388 handles. The sharers are: {
  "31000001": { erc/canary/canary.kicad_sch, sch/text_overlap/hidden.kicad_sch },
  "32000001": { erc/precision/precision.kicad_sch, sch/text_overlap/visible.kicad_sch },
  …
```

**Series in use, measured across the whole fixture tree:**

| Series | Taken by |
|---|---|
| `31`, `32` | **T2** — `erc/canary`, `erc/precision` |
| `31`, `32`, `33` | **`KI-TXT-001`** — `sch/text_overlap/*` |
| `33`, `34` | **`KI-HIER-001`** — `erc/hier/*` |

Highest in use: **`34`**. First free: **`35`**.

## Why no lane could have seen it

Three lanes ran **in parall**, each choosing a series from an allocation that is
**global across the whole fixture tree and written down nowhere.** The
`MANIFEST` header documents the `{series:02x}{n:06x}` *format* and names no
allocation, so the only way to find a free series is a grep over every fixture —
and a grep run in a lane worktree **cannot see a sibling lane's uncommitted
fixtures.** Each lane was correct against the tree it could observe.

**`lane-hier` hit the collision inside its own worktree** (its first choice,
`32`, collided with `erc/precision`, which had merged by then), renumbered to
`33`, and **reported the cause in its WORKFLOW NOTE**:

> *"the uuid series is a global allocation across the whole fixture tree that is
> recorded nowhere… the only way to find a free series is a grep over every
> fixture. One line in that header giving the highest series in use would have
> saved a full gate run, and **a lane whose `MANIFEST` scope is 'its own lines
> and nothing else' cannot add it.**"*

**It told the orchestrator exactly this, and the orchestrator merged the two
colliding lanes before acting on it.** That is the defect's real owner. The
narrow `MANIFEST` permission — *"your line and nothing else"* — was written to
stop lanes fighting over a hotspot, and it also stops the one actor who notices
the problem from fixing it.

## Goal state, as the checks that prove it

### 1. `fixture_handles` passes

`sch/text_overlap/`'s five fixtures move to free series. **`35` onward** — do not
reuse `31`–`34`. The other two families stay where they are: T2's merged first
and `KI-HIER-001`'s already renumbered once.

**One series per fixture**, as the existing families do, so a later reader can
tell which file an atom belongs to from the atom.

### 2. The fixtures stay what they were

Four properties, each with a check that already exists — **run all four**:

- **`round_trip`** — the files are prettifier fixed points. They must still be.
- **`invariants_pass_on_every_fixture`** — `angled.kicad_sch` was regridded by
  `KI-TXT-001` precisely because a label is a connection point. **Do not undo
  that.** Its labels sit at `(99.06,99.06)` and `(95.25,99.06)`, separation
  `38100` IU.
- **`lint_overlapping_text`** — 7 checks. The rule's findings name objects by
  handle, so **renumbering changes expected strings.** Update them, and say
  which.
- **`fixtures_match_manifest`** — paths unchanged, so this should not move.

**A uuid appears in more than one place in a file** (the symbol's own atom and
its instance path). **Renumber every occurrence**, and confirm by grep that the
old series is absent from the tree afterward.

### 3. The `MANIFEST` header records the allocation — this is the part that stops recurrence

**Add one line** naming the **highest series in use**, so the next lane can read
it instead of grepping, and so the next orchestrator knows what to bump.

**You are the one actor authorised to edit that header**; this brief is that
authorisation. Every lane's `MANIFEST` permission is deliberately *"its own
line and nothing else"*, which is why this could not be fixed from inside a
lane.

**Say in the entry what the line says and where you put it.** It is prose in a
data file, and a header that drifts from the data is worse than no header —
note whether anything checks it, and if nothing does, **say so plainly** rather
than implying the record is self-maintaining.

## Falsification obligation

Per `.claude/skills/falsification-control/SKILL.md`.

- **`fixture_handles` is shown failing before the repair** — it already does, at
  `e06e117`. **Paste that output**, then paste it passing. The before-state is
  the proof the repair was needed and it is free.
- **Re-introduce one colliding atom and confirm `fixture_handles` goes red
  again**, so the check is shown alive against *this* collision rather than
  against the general property.
- **The degenerate risk, named:** a renumber that is internally consistent but
  changes what a fixture *means* passes `fixture_handles` and breaks the rule
  that reads it. `lint_overlapping_text`'s 7 checks are the control. **If any of
  them needed a change beyond a handle string, STOP AND REPORT** — that is not a
  renumber.

## Scope

**IN**
- `crates/kicli/tests/fixtures/sch/text_overlap/*.kicad_sch`
- `crates/kicli/tests/fixtures/MANIFEST` — **the header line, and nothing else;
  no record line changes, since no path moves**
- `crates/kicli/tests/lint_overlapping_text.rs` — **only** expected handle
  strings
- this file, for the evidence, written AS YOU WORK

**OUT** — every rule file, every other fixture family (`erc/**` especially),
every other test, every other entry. **If a rule's source needs changing, this
is not a renumber: stop and report.**

## Completion check

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo xtask check --corpus
```

**Background it** — `git commit` triggers it too, via the pre-commit hook, and a
foreground call is capped at 10 minutes. Expect `COMPLETE: all 8 arms passed`.
**That exact line is the deliverable**, because `main` is red until it appears.

---

## Evidence — `lane-uuid`, implementing, at base `c56f219`

Worktree verified as the first action: `c56f219`, `git status --porcelain`
empty. No fast-forward needed.

### The task text's series table yields to measured reality

**Measured** (`grep` of every uuid in `crates/kicli/tests/fixtures/`, mapped to
its file, at `c56f219`) — the table above is wrong in two ways, and the
corrections do not change the repair:

| Series | Taken by, measured | The table above said |
|---|---|---|
| `30` | `sch/text_overlap/angled.kicad_sch` **only** | not listed at all |
| `31` | `erc/canary` + `sch/text_overlap/hidden` | `31` for both — correct |
| `32` | `erc/precision` + `sch/text_overlap/visible` | correct |
| `33` | `erc/hier/{hier,matched,mismatched}` + `sch/text_overlap/one_pair` | correct |
| `34` | `sch/text_overlap/boundary.kicad_sch` **only** | credited to `KI-HIER-001` |

Two corrections:

1. **`KI-TXT-001` held five series, `30`–`34`, not `31`–`33`.** One per
   fixture, which is the convention the entry asks for and which that lane had
   already followed.
2. **`KI-HIER-001` holds only `33`** — all three of its files share it. It does
   **not** hold `34`; `34` was `boundary.kicad_sch`'s.

So **only three series actually collided** (`31`, `32`, `33`); `30` and `34`
were text_overlap's alone. **Highest in use `34` and first free `35` are both
correct as stated.** All five fixtures were still moved, per the entry's
instruction, because that makes the family one contiguous block and is what
makes the atom tell the reader its file.

### `fixture_handles` failing, before the repair

`cargo test -p kicli --test fixture_handles` at `c56f219`:

```
test every_committed_fixture_object_answers_to_a_handle_of_its_own ... FAILED
assertion `left == right` failed: every uuid atom of the committed fixture tree
must answer to a handle of its own, or no fixture can exercise a command
addressed by one. 1395 atoms share 1388 handles. The sharers are: {
    "31000001": { erc/canary/canary.kicad_sch, sch/text_overlap/hidden.kicad_sch },
    "32000001": { erc/precision/precision.kicad_sch, sch/text_overlap/visible.kicad_sch },
    "33000001".."33000005": { erc/hier/hier.kicad_sch, sch/text_overlap/one_pair.kicad_sch },
}
  left: 1388
 right: 1395
test result: FAILED. 2 passed; 1 failed
```

**Seven shared handles across three series.** `erc/hier` contributed five of
the seven, which is why the collision cost more than the three-series count
suggests.

### The renumber

One series per fixture, `35` onward, alphabetical order preserved so the shift
is a uniform `+5`:

| Fixture | Was | Now |
|---|---|---|
| `angled.kicad_sch` | `30` | **`35`** |
| `hidden.kicad_sch` | `31` | **`36`** |
| `visible.kicad_sch` | `32` | **`37`** |
| `one_pair.kicad_sch` | `33` | **`38`** |
| `boundary.kicad_sch` | `34` | **`39`** |

**`erc/**` was not touched.** `git diff --stat` covers only the five
`sch/text_overlap/*.kicad_sch` files, `MANIFEST` and this entry.

**The series appears TWICE in each of these uuids** — the handle prefix and the
body's leading two hex (`31000001-0000-4000-8000-310000000001`), a convention
`KI-TXT-001` used and the `erc/**` families do not. **Both positions were
renumbered**, because the entry requires the old series to be *absent* from
`sch/text_overlap/` afterwards and a body left at `31` would have kept it
present. Confirmed by grep: the only series now present under
`sch/text_overlap/` are `35 36 37 38 39`.

Applied by `/tmp/kicli-scratch/uuid/renumber.py`, which **asserts rather than
assumes** four things per file, and would have refused the write on any of
them: every uuid in the file carries the old series in both positions; the uuid
count is unchanged; **no old-series occurrence survives**; and —

```python
assert UUID.sub('<U>', src) == UUID.sub('<U>', out), f"{p.name}: non-uuid bytes changed"
```

— **every byte that is not inside a uuid is identical.** That assertion is the
structural proof this was a renumber and not an edit: no coordinate, string,
style or angle in any of the five files could have moved. It is why
`angled.kicad_sch`'s deliberate regrid survives *by construction* rather than
by luck. Re-verified directly anyway: its labels are still at
`(99.06 99.06 0)` and `(95.25 99.06 90)`, separation `3.81` mm = `38100` IU.

Audit of the whole working diff: 44 changed lines, **every one of them a
`(uuid "…")` or `(path "…")` line.**

### The four properties, after

| Check | Result |
|---|---|
| `fixture_handles` | **3 passed, 0 failed** (was 1 failed) |
| `round_trip` | **4 passed, 0 failed** — still prettifier fixed points |
| `invariants_pass_on_every_fixture` | **4 passed, 0 failed** — regrid intact |
| `lint_overlapping_text` | **7 passed, 0 failed** |
| `fixtures_match_manifest` | **1 passed** — no path moved, as predicted |

### `lint_overlapping_text` needed NO change — the entry's prediction was wrong

**The entry says renumbering changes this file's expected strings. It does not,
and the measurement is that the file was never edited.**

`lint_overlapping_text.rs` contains **no handle literal at all**. Every handle
it asserts on is read off the fixture at run time — `written()` collects
`uuid.short()` into `Written::handle`, and the assertions compare against
`drawn[0].handle`, `drawn[3].handle`, `below[2].handle`. Confirmed by grep: no
8-hex-character string literal and no uuid literal anywhere in the file.

So **zero expected strings changed**, and the 7 checks were a *clean*
control on the degenerate risk rather than a control that had to be adjusted to
keep passing — which is the stronger position of the two. Had the renumber
altered what any fixture means, these 7 would have failed on unchanged source.
They passed on unchanged source.

**The entry's "STOP AND REPORT if any needs a change beyond a handle string"
condition was not reached**, because no check needed any change whatever.

Two literal uuid references to the *neighbouring* families exist elsewhere and
are **untouched and still correct**, which is the control that `erc/**` was not
disturbed:
- `crates/kicli/tests/lint_hier_label_mismatch.rs:263` — `33000002-0000-4000-8000-000000000001`, which is `erc/hier`'s (body `000000000001`), never `one_pair`'s (body `330000000002`).
- `crates/kicli/src/lint/erc.rs:344` — `31000010-…`, `erc/canary`'s.

### Falsification — the check is shown alive against *this* collision

Per `.claude/skills/falsification-control/SKILL.md`. **One atom** of
`hidden.kicad_sch` put back onto the colliding series — `36000001` →
`31000001`, the exact sharer the before-state named:

```
test every_committed_fixture_object_answers_to_a_handle_of_its_own ... FAILED
1395 atoms share 1394 handles. The sharers are: {
    "31000001": {
        ("31000001-0000-4000-8000-000000000001", "erc/canary/canary.kicad_sch"),
        ("31000001-0000-4000-8000-360000000001", "sch/text_overlap/hidden.kicad_sch"),
    },
}
```

**Red on a single colliding atom, naming both sharers and both files.** The
check is therefore alive against this collision specifically, not merely
against the general property. Fixture restored from a byte copy and
`fixture_handles` re-run: **3 passed, 0 failed**, and the series present under
`sch/text_overlap/` are again exactly `35 36 37 38 39`.

### The `MANIFEST` header line, and what checks it — nothing

Added to the header's fixture-handle-chore paragraph in
`crates/kicli/tests/fixtures/MANIFEST`, immediately after the line
*"Nothing else about any file moved."* **14 added lines, all comments, zero
deletions, no record line touched** — confirmed by counting the non-`#`
additions in `git diff`, which is 0.

The load-bearing sentence:

> `THE SERIES ALLOCATION IS GLOBAL ACROSS THIS WHOLE TREE, AND THIS LINE IS ITS`
> `ONLY RECORD. Highest series in use: 0x39. A new fixture takes 0x3a, and the`
> `commit that adds it raises this line.`

`0x39` was **measured, not asserted**: the series in use across
`crates/kicli/tests/fixtures/` after the repair are `00 01 10 11 12 13 14 15 16
17 18 19 1a 1b 1c 20 31 32 33 35 36 37 38 39`, highest `39`.

**Nothing checks this number, and the header says so in its own text.**
`fixtures_match_manifest` skips every line beginning with `#` (`read_manifest`,
`crates/kicli/tests/fixtures_match_manifest.rs:30`), so the header is prose no
test reads. A stale number here is caught only by `fixture_handles` going red
*after* a colliding fixture is already committed — which is precisely how this
chore came to exist. **The record does not maintain itself**, and the line is
an aid to the next lane's choice rather than a guard on it.

`0x30` and `0x34` are now **free holes** below the high-water mark. The header
records that too, so a later reader does not mistake the gap for a mistake.

**PROPOSED: the honest guard is a check, not a comment, and it was left
undone deliberately.** A test that parses `Highest series in use: 0x([0-9a-f]+)`
out of the header and asserts it equals the measured maximum would convert this
line from prose into an enforced invariant, and would fail the moment a lane
adds a series without raising it. That is a new check in
`crates/kicli/tests/`, which is outside this chore's scope (`lint_overlapping_
text.rs`, expected handle strings only). Recommendation: **add it**, as a
one-test chore — the comment form fails exactly where this milestone already
got hurt, at the moment a parallel lane does the wrong thing silently. Noted
here rather than acted on, because a lane inventing scope is the failure mode
this project's merge discipline exists to prevent.

### Completion check

`cargo xtask check --corpus` in the lane worktree. The first run **refused**,
which is the designed behaviour and worth recording: the worktree had no
corpus, and the flag that asks for one does not silently get none —

```
xtask: --corpus asks for the corpus and …/target/corpus/demos is not there, so
the tests cannot read the canonicalised demo schematics.
xtask: run `cargo xtask corpus` first, then run this again.
xtask: no arm ran.
```

Corpus fetched by the documented path (`cargo xtask corpus`, 115 sch and 19 pcb
files canonicalised) **in this worktree**, rather than copied from the main
checkout's existing 347 MB copy, so the artifact under test was produced by the
path the gate documents. Re-run:

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

**The `kicad-cli` arm passed**, so KiCad itself re-read the renumbered fixtures
and the regenerated oracles still match the committed bytes — the strongest
available evidence that the renumber changed identity and nothing else.

Per CLAUDE.md this lane-worktree corpus run **does not count toward done**; the
orchestrator's merged run is the one that does.

### Commits

- `05f0115` — the repair: five fixtures renumbered, the `MANIFEST` header's
  allocation line, and the evidence above. Branch `lane-uuid`, base `c56f219`.

**Operational note for the orchestrator's merge:** the pre-commit hook runs
`cargo xtask check` **without** `--corpus`, so the commit's own gate reports
`INCOMPLETE: 2 of 8 arms did not run. 6 passed, 0 failed.` That is the hook's
normal behaviour and not a failure. The 8-arm `COMPLETE` above came from an
explicit `cargo xtask check --corpus` run on byte-identical content — the
working tree was clean at commit time and `git status --porcelain` is empty
after it.
