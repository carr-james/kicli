# Worked examples become measured output (opening 3) ✅

**Provenance: advisor recommendation, James-approved, M4 close (boundary
package, item 9).** Verbatim:

> `AGENT.md` worked examples become measured output: every example block a
> session touches is regenerated from a real run of the built binary, and D3's
> defect class gets an executable twin — a check that `AGENT.md`'s W-record
> examples parse under the same reader the tool uses, extended as feasible.

The rule's prose half is already written, in `tasks/M5/RULES.md` under "Worked
examples in `AGENT.md` are measured output". **This task is the executable
twin**, and `ENGINEERING.md` is the reason it is a task rather than a paragraph:
*a rule that matters wants an executable twin: a gate, a lint, or a
workspace-reading test that fails when the rule is broken. Prose alone decays.*

## The defect class, so the check has something concrete to be about

Dogfood run 1, defect 3, verbatim from `tasks/dogfood.md`:

> **Wire-report coordinate format contradicts AGENT.md's own example.** The
> doc's worked example for `wire draw` shows:
> `+ W 3300f00e (50.80,50.80) -> (63.50,50.80)`
> with parenthesized points and a `->` arrow. What I actually got was:
> `+ W 906eceb2 180.34,41.91..180.34,46.99`
> — no parentheses, `..` instead of `->` […] I had to sit and reconcile these by
> hand; a first-time reader trusting the doc's format literally would misparse
> this line.

That defect was **fixed** — the current `AGENT.md` lines use `..` — and fixing
it closed nothing. **A hand-written example is correct until the writer changes,
and nothing notices.** Eleven `W` example lines stand in `AGENT.md` today
(lines 183, 496, 539, 540, 657–659, 679–681, 706).

## Goal state, as the checks that prove it

1. **A check that fails when an `AGENT.md` record example does not match what
   the tool writes.** The record writer is
   `crates/kicli/src/view/delta.rs` — `line`, `record_of`, and the `Change`
   glyphs `+ - ~ =`. The check must be anchored to **that** writer, not to a
   grammar you author beside it. A regular expression you wrote is your own
   vocabulary wearing a reference (`falsification-control`, the derivation
   rule); a check that round-trips a line through the tool's own emitter is not.
2. **"Extended as feasible" is yours to bound, and the bound is stated in the
   check's own rustdoc.** Say what the check covers, what it does not, and why
   the boundary sits there. A sweep must assert that its taxonomy is its
   boundary — the M4 handle chore was stopped on exactly this rule after three
   rejections, and its lesson is that extending a matcher without a fixed point
   is worse than naming where it stops.
3. **The check cannot pass by reading nothing.** Every absence check carries a
   presence control: assert that example lines **were found**, and that the
   count is what the document holds. A sweep that matched no files and passed is
   the most common blind instrument in this project's record.
4. **Every `W` example block in `AGENT.md` is regenerated from a real run of the
   built binary**, and the entry records the command that produced each. Where a
   block cannot be regenerated — it documents a case no fixture reaches —
   **say so in the entry and leave the block alone**; do not hand-edit it into
   agreement. A block that cannot be measured is a finding about the fixtures,
   and it goes in this entry for the plan to schedule.

## Falsification obligation

Per `.claude/skills/falsification-control/SKILL.md`. Two of the four blind kinds
bear directly:

- **Reads nothing.** See goal 3. Break the document's example lines (in your
  scratch copy) and watch the check fail; then break the *pattern* so it matches
  nothing, and watch it fail too. Both are required.
- **Shared ancestor.** Anchoring to the tool's own writer is the point of goal 1
  — but it means a break *in the writer* moves both sides together. State in the
  entry what this check can and cannot see, and if the answer is "it cannot see
  a writer change", say so plainly. That is an honest boundary, not a gap;
  claiming otherwise is the defect.

## Scope

**IN**
- `AGENT.md` — example blocks only
- `crates/kicli/tests/agent_doc.rs`, or a new test file beside it
- `crates/kicli/src/view/delta.rs` — **only** if the writer must expose
  something for a test to reach it, and then minimally, with the rustdoc saying
  why
- this file

**OUT** — every other source file, every other entry, `tasks/M5/PLAN.md`,
`crates/kicli/src/route/**`.

**If the enumeration above proves wrong, the goal state wins over the list.**
Say so in your first paragraph, name what you touched and why.

## Sequencing

**Originally: after the joined net's contract field
(`opening-1-joined-net-contract.md`) merged**, because that change regenerates
`AGENT.md` blocks of its own and `AGENT.md` is held by one lane at a time.

**Amended at the M5 opening, when opening-1 was PARKED** on the freeze-lift
mechanism BLOCKED item. `AGENT.md` has no other holder, so this task runs now,
and the order is the better one: **the check exists before the change that must
satisfy it.** When opening-1 is re-dispatched it inherits a live check over the
`W` examples — including the `wire draw` JSON block at `AGENT.md` lines 570–579,
which opening-1's reconnaissance identifies as a second block that change
invalidates.

Your brief names the base commit; verify it as your first action.

## Completion check

```sh
cargo test --test agent_doc
cargo xtask check
```

---

## What was built (lane `lane-o3`, base `9f4eb39`)

Base verification, first action: `git log --oneline -1` gave `9f4eb39` and
`git status --porcelain` was empty, matching the brief. No fast-forward needed.

### The executable twin, in two tests

Both live in `crates/kicli/tests/agent_doc.rs`, which is what the entry's
completion check runs.

**`agent_doc_record_examples_are_lines_the_writer_can_emit`** — the frame.
Every record line inside a fenced block is taken apart and the pieces are
handed back to `DeltaLine`'s own `Display`, which has to emit the documented
bytes. Nothing in the test states what a record line may look like:

- the **mark set** comes from `Change::ALL`, added to `crates/kicli/src/view/delta.rs`
  for this — the one source change, minimal, with the rustdoc saying why and
  naming its own bound (a new variant is not compiler-forced into `ALL`; what is
  forced is that `Change::mark`'s match stops compiling, so the author has to
  open that impl block, and a unit test beside it fails on a missing or repeated
  mark);
- the **separator** between handle and detail is *measured off the writer* by
  emitting a probe line with a one-character handle and a one-character detail
  and reading back what came out between them. That is why the one-space/
  two-space rule is enforced without being restated.

**`agent_doc_wire_and_text_examples_are_what_the_writer_emits`** — the content.
Each added or removed `W` and `T` line is **rebuilt**: a sheet is written
holding exactly the object the line describes, a snapshot is taken of it and of
the same sheet without it, and `Delta::between`'s own line has to equal the
documented line byte for byte. The separator `..`, the two decimal places, the
sorting of a segment's two ends and the record letter all come out of
`Doc::parse` → `Schematic::read` → `Snapshot::take` → `Delta::between` →
`Display`. Nothing about the printed line is predicted by the test.

### Where the check stops, and why (goal 2)

The bound is in each test's own rustdoc; this is the same statement in short.

- **The frame test does not read the content of a handle or a detail.** Dogfood
  D3's own line, `+ W 3300f00e (50.80,50.80) -> (63.50,50.80)`, *passes* it.
  D3 is caught one test down. The split is deliberate: the frame belongs to
  every record kind, the content can only be rebuilt for the kinds whose line
  carries enough to rebuild them.
- **The reconstruction covers added and removed `W` and `T` lines only.** The
  boundary is not the record letter, it is whether the line carries enough to
  rebuild the object. It does not for `S` (a symbol's summary is
  `<value> <lib_id>`, and either half may hold a space, so splitting one is a
  grammar rather than a reconstruction), nor for `L`, `F` or any `~` line (the
  detail is a pair of states, which one sheet with one item cannot express), nor
  for `H`/`P` (a second file). Naming where it stops is the result: extending it
  a kind at a time has no fixed point, and a matcher one kind behind reads as
  covering what it does not. This is the C1 lesson applied before the fact
  rather than after three rejections.
- **The five `T` kinds are indistinguishable**, and so are a wire and a bus. A
  `T` line is rebuilt as a local label; a global or hierarchical label, a
  netclass flag, free text and a text box all print the same bytes.
- **The record letter is unchecked on the lines that are not rebuilt.**
  `record_of` decides it from an `ObjectKind`, and a test that listed the kinds
  to ask it about would be listing them from memory — `ObjectKind` also lives in
  `snapshot.rs`, outside this task's scope. Where a line *is* rebuilt the letter
  comes out of `record_of` on a real comparison, and F12 below shows it fails
  when `record_of` changes.

### What it can and cannot see (goal: the shared-ancestor question, answered plainly)

The control here is **`AGENT.md` itself** — hand-maintained text that no part of
kicli computes — so the usual shared-ancestor trap does not apply, and the
measurements say so: F9, F10, F11 and F12 all break the **writer** alone, leave
the document alone, and all four fail. The check is not blind to a writer change.

What it cannot see:

1. **A writer change mirrored into the document in the same commit.** The check
   asserts *agreement*, not *provenance*. It cannot tell a block regenerated
   from a real run from one hand-edited to be self-consistent. That is what the
   `RULES.md` rule asks of the person editing the block, and it is not decidable
   from the bytes.
2. **Whether a documented value came from a run at all.** A `+ W` line with
   invented but well-formed coordinates passes. The check enforces the format
   the tool writes, not the history of the number.
3. **The layout view's `W` line** (`W 15 segments, 2 junctions, 1 crossings`,
   `AGENT.md` line 183). See the correction below — it is a different writer.

### Correction to the task text, measured

- The entry says the `Change` glyphs are `+ - ~ =`. Measured: `Change::mark()`
  produces `+`, `-` and `~` only; `=` is written by `Delta`'s own `Display` as
  the unchanged tally (`= {n} objects unchanged`) and belongs to no `Change`.
  The check's mark set is therefore three glyphs, derived from `Change::ALL`.
- The entry counts "eleven `W` example lines … (lines 183, 496, 539, 540,
  657–659, 679–681, 706)". Line 183 is **not** a delta record line: it is the
  layout digest's wire summary, written by the layout view, not by
  `view/delta.rs`. Ten delta `W` lines stood at the base commit, not eleven, and
  line 183 is outside what a check anchored to the delta writer can reach.
- The document held **17** record example lines at the base commit and **20**
  after the extension below, of which **14** are rebuilt. Both numbers are
  asserted as floors, so removing an example fails and adding one does not.

### An extension the work turned up

The session walkthrough (`AGENT.md` ~line 737 onward) shows each command's
answer as a **shell comment** under the command — `# + T da5aa983 "SPY"`. Three
record examples sat there unchecked because of two characters. `record_examples`
now strips one leading `"# "` before looking for the frame, which brought the
count from 17 to 20 and the rebuilt count from 13 to 14.

### Falsification (per `.claude/skills/falsification-control/SKILL.md`)

Good state committed **before** any break, per Rule 1. Content hashes of the
good state, which is what the rows below were measured against:

- `crates/kicli/tests/agent_doc.rs` — `0dc68a675585e0c18e417f53a218fe8be8a99124`
- `crates/kicli/src/view/delta.rs` — `711d73f86e2fffaa64f53ea22f6caaf3c91f721d`
- `AGENT.md` — `ab2686640e82f429bd21654dd4e7e1e087fb9bf3`
- `crates/kicli/src/view/snapshot.rs` — `c5b554f5c07cfcc137c8307a01bdf37513e06d1c`

Each row restores the file afterwards and re-checksums it; every restore matched
the hash above. `frame` is
`agent_doc_record_examples_are_lines_the_writer_can_emit`, `rebuild` is
`agent_doc_wire_and_text_examples_are_what_the_writer_emits`. **No row came back
green**, so there is no case-1/case-2 diagnosis to make.

| # | What was broken | frame | rebuild | Which assertion caught it |
|---|---|---|---|---|
| F1 | `AGENT.md:496` restored to dogfood D3's own bytes, `+ W 3300f00e (50.80,45.72) -> (50.80,50.80)` | pass | **fail** | the byte comparison in `rebuild`: `document: … (50.80,45.72) -> (50.80,50.80)` against `kicli: … 50.80,45.72..50.80,50.80`. **The frame test passing here is the boundary claim above, measured rather than asserted.** |
| F2 | `AGENT.md:212`, one space after a `~` handle instead of two | **fail** | **fail** | `decompose`'s separator error, "the writer puts 2 space(s) between a `~` line's handle and its detail" |
| F3 | `AGENT.md:428` given the mark `*`, which no `Change` makes | **fail** | **fail** | the presence controls — the line stops being recognised, so the count falls to 16 and the rebuilt count to 12. This is the row that shows why the counts are asserted |
| F4 | `AGENT.md:706`, the two ends of the segment swapped | pass | **fail** | the byte comparison; `line_object` sorts a segment's ends |
| F5 | `AGENT.md:496`, `50.8` for `50.80` | pass | **fail** | the byte comparison; `millimetres` writes two decimals |
| F6 | `AGENT.md:428`, the `T` detail stripped of its quotes | pass | **fail** | the `T` branch's "detail is not a quoted string" panic |
| F7 | `record_examples`' frame rule changed to match nothing (`Some(' ')` → `Some('!')`) | **fail** | **fail** | both presence controls, at 0 examples and 0 rebuilds. This is the **reads-nothing** pair's second half |
| F8 | the rebuild loop's record filter changed from `'W' \| 'T'` to `'Q'` | pass | **fail** | the rebuilt-count control, at 0. Separating the two counts is what makes this visible: the frame test still found all 20 |
| F9 | `snapshot.rs`, `line_object`'s segment summary changed from `{}..{}` to `({}) -> ({})` — **the writer, not the document** | pass | **fail** | the byte comparison, in the opposite direction from F1: `document: … 50.80,45.72..50.80,50.80` against `kicli: … (50.80,45.72) -> (50.80,50.80)` |
| F10 | `delta.rs`, `DeltaLine::fmt`'s `~` separator cut from two spaces to one | **fail** | **fail** | **not** the separator assertion — the probe faithfully reported one space, the strip succeeded, and the *leading-whitespace* assertion fired: "the detail ` moved  …` begins with whitespace, which no producer in delta.rs's `detail()` can emit". That second assertion exists for exactly this, and this row is why it is not decoration |
| F11 | `delta.rs`, `Change::mark` gives `Added` the mark `@` | **fail** | **fail** | both presence controls — the document's `+` lines stop being recognised, count 4 and rebuilt 1 |
| F12 | `delta.rs`, `record_of` prints `X` rather than `W` for a wire | pass | **fail** | the byte comparison: `+ X 3300f00e …` against the document's `+ W …`. The record letter **is** checked where a line is rebuilt |

**Re-verified after the document was regenerated**, because F1–F12 were
measured against `AGENT.md` at `ab26866…` and the regeneration below replaced
the very lines four of them broke. Four rows re-run against the final tree
(`AGENT.md` `3b92e79e140eea677c5bf16975a9347c157cba93`,
`crates/kicli/tests/agent_doc.rs` `a57a7d36a8ebae89a5e38d7209a6a60acc9fd004`),
all four still fail:

| # | What was broken, on the final tree | frame | rebuild | Caught by |
|---|---|---|---|---|
| R1 | the regenerated `+ W 116592a7 …` given D3's parentheses and arrow | pass | **fail** | the byte comparison — F1 re-made on the line that replaced F1's line |
| R2 | `AGENT.md:212`, one space after a `~` handle | **fail** | **fail** | the separator error |
| R3 | the walkthrough's **commented** `# + T da5aa983 "SPY"` given two spaces after its handle | **fail** | **fail** | the leading-whitespace assertion, at `AGENT.md` line 750. This is the row that shows the `"# "` extension is load-bearing: before it, this line was invisible |
| R4 | `snapshot.rs`, `line_object`'s segment separator | pass | **fail** | the byte comparison, writer side |

Two removals were needed for F10 to be attributable: the break is one line in
`DeltaLine::fmt`, and the assertion that caught it is a different one from the
assertion the break was aimed at. Recorded as such rather than as "the separator
check caught it".

**The environment as a break class.** The tests read `AGENT.md` through
`CARGO_MANIFEST_DIR` and consume no generated value — the identifiers they build
are derived from the document's own handles, not minted. Run anyway from a
second directory per the fifth-dimension rule:
`scratch="$(mktemp -d)"; git archive HEAD | tar -x -C "$scratch"; (cd "$scratch" && cargo test --test agent_doc)`
— 10 passed, 0 failed.

### The blocks, regenerated from real runs (goal 4)

The drawings are built with `kicli_probe::Probe` by a throwaway crate outside the
repository (`…/scratchpad/mkdraw`), because no committed fixture reaches these
scenarios; the drawings themselves are therefore **not** committed. The
binary is `target/debug/kicli` at `a97b978ef0ce8bfce819f79f1220f453b125ed4a`.
Every command below was run with `-p <project directory>`.

| `AGENT.md` block | Command | What changed |
|---|---|---|
| `wire draw`, the route contract (~493) | `kicli wire draw --from-pin R1.1 --to-pin R2.1 --via 50.8,45.72 --via 76.2,45.72` on two resistors with pin 1 at `50.80,50.80` and `76.20,50.80` | **A real defect.** The block showed **one** `+ W` line for a three-segment route and omitted `wires added: 3   junctions added: 0` entirely — a row its own contract table promises. Now all three segments and the count |
| `wire draw --auto-labels` (~536) | `kicli wire draw --from-pin U1.1 --to-pin U2.2 --auto-labels` on two parts whose pin 1 is named `SCK`, U1 pin 1 at `12.70,199.39`, U2 pin 2 at `285.75,10.16` | Handles only. `path length 462.28mm`, the threshold, both label positions, both stub segments and the note came back identical — the block was right |
| `wire connect`, worked, the dear route (~654) | `kicli wire connect --from-pin R30.1 --to-pin R31.1` on R30 pin 1 at `127,173.99`, R31 pin 1 at `215.90,173.99`, with another net's wire at `x = 170.18` | Handles, the crossing wire's handle, the order of the three `+ W` lines, and `joined: net #n5` → `#n1`. `93.98mm`, `cost 110 = 74 + 12 + 20 + 0 + 4` and `crossings: 1 (at 170.18,171.45 …)` reproduced exactly |
| `wire connect`, worked, after the move (~677) | `kicli wire delete` ×3, `kicli sym move R31 --to 152.4,177.8`, then the same connect | Handles, order, `#n5` → `#n1`. `30.48mm` and `cost 40 = 24 + 12 + 0 + 0 + 4` reproduced exactly, so the prose's "110 to 40" stands |
| `wire delete` (~706) | `kicli wire delete 01000004` on two wires meeting at a junction | Both handles. The stranded-junction note's wording, position and "1 wire end(s)" reproduced exactly |
| `label add` (~427) | `kicli label add --text SPY --at 114.3,63.5` on a drawing whose third net is `R12.2 R13.1` | Handle only. `net SPY: R12.2 R13.1 (was #n3)` reproduced exactly, `#n3` included |
| `wire draw --output json`, the `wire` object (~570) | `kicli wire draw … --output json`, same drawing as the first row | Nothing. Every key, every value and the key order match the real object; the block is the `wire` sub-object pretty-printed with `…` for the three uuids, and that is what the run returns |

**Measured, and the reason a printed line was left out of every block above.**
Each run also printed `the file was laid out again, as KiCad's next save would`.
That line is conditional on the file arriving non-canonical, which a
probe-written drawing does and a KiCad-written one does not: running the same
verb a second time, on the file kicli had just written, does **not** print it.
So it is an artefact of the instrument, not of the command, and no block gained
it. `AGENT.md` documents the flag it corresponds to (`reformatted`) already.

### Blocks NOT regenerated, and why (goal 4's second half)

- **The delta digest (`AGENT.md` ~211).** Reaching it from the binary needs a
  project kicli has written, then touched by something else, with a symbol
  moved, a value edited, one symbol removed and one added. No committed fixture
  reaches that, and building it means hand-editing a `.kicad_sch` between two
  runs. **The block is corroborated rather than regenerated**:
  `crates/kicli/tests/delta_view.rs::delta_distinguishes_moved_from_edited`
  asserts these same four lines from a real `Delta::between`, differing only in
  `Test:R` where the document writes `Device:R`. Left alone.

  > **Corrected by the tick reviewer, and the correction goes beneath the claim
  > rather than over it.** The reviewer ran that test and diffed its output
  > against the document: **the ORDER of the trailing three lines also differs**
  > — the document writes `~S`, `+S`, `-S`; the test emits `+S`, `-S`, `~S`. So
  > "differing only in `Test:R`" is an overstatement, measured. It does not
  > touch this task's goal states, because the block is outside the diff, was
  > explicitly not regenerated, and is outside the new check's rebuild boundary
  > — but "corroborated" is a weaker claim than this paragraph made it sound,
  > and the difference is exactly the kind a later reader would take on trust.
  > **The block is corroborated in content and not in order.**
- **The layout digest (`AGENT.md` ~176).** A different writer, outside this
  task's anchor. Left alone.
- **The session walkthrough's commented output (`AGENT.md` ~737).** A narrative
  over a project that does not exist — `parts.kicad_sym`, `R99`, and a `SPY` net
  on `R12`/`R13` at a third position. Its three record lines are now *checked*
  (the `# ` extension above) but its handles are not measured. Left alone.
- **PROPOSED — the fixtures owe these scenarios.** Every regeneration above
  stands on a drawing built by a throwaway crate outside the repository, so
  **nothing in the repository can reproduce these blocks.** The next person to
  edit one of these blocks starts from scratch. Recommendation: a fixture or a
  probe recipe, committed, that builds the five drawings, so `AGENT.md`'s worked
  examples have a reproducible source. Not done here: it lands in
  `tests/fixtures/MANIFEST` or in `kicli-probe`, both outside this task's scope.
  Evidence for the size of it: the five drawings are ~60 lines of `Probe` calls.

### Scope

Written: `AGENT.md` (example blocks only), `crates/kicli/tests/agent_doc.rs`,
`crates/kicli/src/view/delta.rs` (`Change::ALL` and one unit test), this file.
Nothing else. No merge hotspot touched.

### Completion check

`cargo test --test agent_doc` — 10 passed, 0 failed.
`cargo xtask check` — all six gates pass (fmt, clippy, test, doc, deny, clean).
Corpus and environment gates do not count from a lane worktree; the
orchestrator's merged run is the one that does.

---

## Tick — APPROVE, 2026-08-22

**Reviewer verdict: APPROVE.** Merged `f69bad6`; lane `lane-o3`, base `9f4eb39`,
one commit `72ce305`, four files, all inside this entry's IN list.

**What the reviewer measured itself:**

- **The headline defect, against the binary rather than against this entry.** It
  built the scenario with `kicli_probe::Probe` — two resistors, pin 1 at
  `50.80,50.80` and `76.20,50.80` — and ran the real built binary's `wire draw`
  in a `git archive` scratch tree. The output matched the regenerated block
  **byte for byte, including the three handles** `116592a7`, `7239e219`,
  `723e5aa0`, the `wires added: 3   junctions added: 0` line and the cost
  breakdown. The old block was a genuine documentation defect, confirmed.
- **The counts, by an independent re-implementation** of the frame rule in
  Python: 22 documented lines and 16 rebuilt on the final document, against the
  20/14 floors this entry asserts — **honest floors, not overcounts**, the
  difference being the two `+ W` lines the regeneration itself added after the
  constants were set. Removing the `"# "` strip drops the count by exactly 3, as
  claimed.
- **The reads-nothing falsification (F7), reproduced.** Changing
  `record_examples`'s separator match from `Some(' ')` to `Some('!')` gave
  `0 record example line(s)` and `0 example(s) were rebuilt`, **both presence
  controls firing**. The file was content-hash-verified back to the lane's state
  before the next step.
- **The shared-ancestor boundary, tested and found honest.** Restoring the
  `wire draw` block to its pre-fix single-line form left **both** tests passing —
  because a self-consistent single record for a single object is legitimately
  reconstructable. That is precisely the limit this entry's rustdoc states:
  *agreement is not provenance.* The reviewer recorded it as the named boundary
  rather than as a gap.

**On `Change::ALL` — derivation or citation?** The reviewer's answer, recorded
because it is the question the derivation rule exists to ask: it is **not a pure
derivation**. A variant could be added to the enum, handled in `mark()` under
compiler force, and never added to `ALL` under no force at all. **The rustdoc
says exactly that**, so the bound is honest rather than oversold — and an honest
bound is what the skill asks for when a stronger instrument is not available.

**Scope:** `AGENT.md` confined to lines inside fenced example blocks, no prose
outside them; `delta.rs` exactly `Change::ALL` plus one unit test with the
rustdoc saying why; both new tests inside `agent_doc.rs`, so the completion
check does exercise everything the scope permits. No merge hotspot touched.

**The reviewer's own friction, recorded because it was the orchestrator's:** the
brief's pin went stale mid-review when this lane amended `3171db0` into
`72ce305`, and the orchestrator had to correct it live. The reviewer re-verified
the diff stat and both scratch trees' content hashes against the new head rather
than assuming the correction.
