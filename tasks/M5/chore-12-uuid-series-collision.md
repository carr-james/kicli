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
