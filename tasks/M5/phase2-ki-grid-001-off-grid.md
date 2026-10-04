# `KI-GRID-001` — connectable geometry off grid (Phase 2, lane A)

**Provenance: `tasks/M5/PLAN.md` Phase 2, RATIFIED at the M5 plan review; the
seam verdict and lane table ratified at the checkpoint 1 review.**

**Depends on T1 (the seam), T3 (the formula) and T4 (tier separation and the
saturation property).** A Tier 1 rule written before T4 would need the
saturation property retrofitted, which is the thing the BLOCKED 3 ruling
sequenced T4 first to avoid.

## The rule, from `research/style-rules.md` §4

> **Detect**: for every pin `p` of every symbol, every wire endpoint, junction,
> no-connect, label anchor and sheet pin position `q`:
> `q.x mod G ≠ 0 ∨ q.y mod G ≠ 0` → finding.

**Why it blocks**: Constitution §7. Off-grid pins **silently fail to connect**.
The drawing looks right and the netlist is wrong, which is the class of defect
this whole project exists to catch.

## Goal state, as the checks that prove it

### 1. Every connectable class is covered, and text is exempt

Covered: symbol pins, wire endpoints, junctions, no-connects, label anchors,
sheet pin positions.

**Exempt: field and graphic text positions.** `research/geometry.md`
Contradiction 2 — **KiCad's own autoplacement lands fields on arbitrary IU**, so
a blanket rule fails KiCad's own output. A rule that fires on a file KiCad just
wrote is a rule nobody will keep enabled.

**The exemption is a check, not a comment.** Write a fixture whose fields are
off grid and whose connectable geometry is on it, and assert **no finding**.
That check fails the day someone widens the rule, which is the plausible future
mistake.

### 2. Integer arithmetic only

Constitution §4: detection is integer geometry. `G` is 50 mil = 12700 IU
(`spec/SPEC.md` §5.2). A modulus is exact; **there is no tolerance and no
rounding here**, and introducing one would make the rule's own boundary a
judgement.

### 3. ERC's overlap is reported, not double-counted

ERC's `endpoint_off_grid` covers **wire endpoints only**, as a **warning**.
`KI-GRID-001` additionally covers pins, labels and sheet pins, and **blocks**.

**Report ERC's finding when present; do not double-count.** T2 builds the
mechanism that makes non-double-counting possible — read what it built and use
it rather than inventing a second one. If T2's mechanism does not reach this
case, **say so; do not work around it.**

### 4. The saturation property is declared

Per the BLOCKED 3 ruling and T4's implementation. This rule counts **connectable
points**. Declare its denominator and its fraction, and say in the entry what
you declared and why. See `RULES.md`.

### 5. The knobs exist and are read

`grid = "50mil"`, `grid.exempt_text = true`, per `kicli.toml`'s `[rules]` table.
**That table is a merge hotspot — report what it owes, do not edit it.**

## Falsification obligation

Per `.claude/skills/falsification-control/SKILL.md`, re-read: it gained two lines
at this checkpoint (`--no-fail-fast`, and the shared-probe-name flake).

- **Each covered class is shown firing independently.** One fixture with an
  off-grid pin, one with an off-grid junction, and so on. **A single fixture
  carrying one off-grid object of every class is a degenerate fixture**: it
  passes if the rule only checks pins, because *something* was found.
- **The text exemption is shown holding** — goal state 1.
- **The most likely blind instrument, named in advance:** a check that asserts
  "N findings" over a fixture. If the rule silently stops examining a class, the
  count changes and the check fails — good — but if the rule *double-reports*
  one class the count also changes, and the two are indistinguishable. **Assert
  on the findings' positions and rules, not on their number.**

## Scope

**IN**
- `crates/kicli/src/lint/rules/grid.rs` — **new file, and no edit to any
  existing file.** That is the seam T1 built and this rule is its first real
  customer. **If you find yourself needing to edit an existing file, stop and
  report it — that is a finding about the seam, not a detail.**
- new test files and new fixtures under `crates/kicli/tests/`
- this file, for the evidence, written AS YOU WORK

**MERGE HOTSPOTS — report, do not edit.** `Cargo.toml`, `lib.rs`,
`crates/kicli/build.rs`, the fixture `MANIFEST`, `AGENT.md`, `spec/SPEC.md`,
`tests/command_surface.rs`, `kicli.toml`'s `[rules]` table.

**OUT** — every other rule file (three other rules are live), every other entry.

**If this scope proves wrong, the named goal state and its checks win over the
list.** Say so in your first paragraph, name what you touched and why.

## Completion check

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo xtask check
```

plus the per-class checks by name, each shown failing under its own break, and
the exemption check recorded with its falsification.

**Note:** `cargo fmt --check` **cannot see your rule file** — `rustfmt` does not
follow the `include!` the generated registry uses. Your file is formatted by
`crates/kicli/tests/rule_files_are_formatted.rs` **and by nothing else.** Run it.

---

# Evidence and deliverable (lane a)

**Base `18ab930`, worktree `.claude/worktrees/lane-a`, branch `lane-a`.**
Verified as the lane's first action: `git log --oneline -1` gave `18ab930
merge: the Greenberg checklist, vendored so the citation cannot drift (M5)`
and `git status --porcelain` was empty. No fast-forward was needed.

## Resumption: the base moved, and what that invalidated

**The first implementer was cut off by a usage limit with the falsification
table unwritten.** Its last recorded words were *"Each class break is caught by
its own named check. Waiting for the remaining nine."* The orchestrator then
merged `main` (`25aea90`) forward, so the lane resumed at a different base.

**Base re-verified as the resuming lane's first action: `git log --oneline -1`
gave `98ccadb merge main forward into lane-a (resumption after the usage
limit)` and `git status --porcelain` was empty.** No fast-forward was needed.
`KI-CONN-001` (`pin_on_wire.rs`) and the eight-arm `xtask` gate arrived with
that merge; `crates/kicli/src/lint/rules/` now holds two files, and the
registration check below is unaffected because it filters the registry by rule
id rather than asserting the registry's length.

**Two claims above went stale across the merge-forward and are repaired here
rather than silently, per the `falsification-control` skill's Rule 2.**

1. **The recorded content hash of the test file was wrong.** The entry recorded
   `3408c1ba…`, which was the file as of commit `f952966`. Commit `0351d00`
   then added 66 lines to it (the committed-drawing check and the measured wire
   caveat) **after** the entry was written, so the hash named a state that no
   longer existed. The two hashes below are re-measured at `98ccadb`.
2. **The "no fixture was added and no `MANIFEST` line was needed" claim was
   re-measured, and it holds.** `git diff --stat 25aea90..HEAD` is three files:
   the rule, its test, and this entry. Both fixtures the test reads
   (`sch/item_zoo.kicad_sch`, `sch/score/high_and_blocked.kicad_sch`) were
   already committed and already carry `MANIFEST` lines 64 and 70, both with
   `kicad-cli` provenance. **No `MANIFEST` line is owed by this lane.**

### The falsification audit, stated as numbers

**Rows the entry recorded: 0.** There was no falsification table in this
section at all — the predecessor's breaks were run but never written down, so
**nothing was confirmable from the record and every row below was run by the
resuming lane.** Rows confirmed from existing evidence: **0**. Rows newly run:
**14**, one per check, listed in the table below.

This is recorded as a finding and not as bookkeeping: the entry read as a
finished, fluent piece of evidence while the single thing that licenses a tick
was absent, and an absent table is harder to notice than a wrong one.

## What was written

| File | State |
|---|---|
| `crates/kicli/src/lint/rules/grid.rs` | new |
| `crates/kicli/tests/lint_grid_covers_every_connectable_class.rs` | new |

**Nothing else.** No existing file was edited, no fixture was added and no
`MANIFEST` line was needed. Content hashes of the good state, per the
falsification skill (a hash survives an amend, a SHA does not):

```
5196ef96be9eedd2103315e43aff00ad407a0af6bdb86bd5df2790ea36a13b62  crates/kicli/src/lint/rules/grid.rs
1ed192e96a9c1b2a371b87ba898136c5a9bf66a7424417a288d1a8645e0561f0  crates/kicli/tests/lint_grid_covers_every_connectable_class.rs
```

The rule's hash is unchanged since `83365bf`; the test file's is re-measured
after `0351d00`, which is the repair described under "Resumption" above.

Good state committed before any deliberate break — `83365bf` for the rule and
`0351d00` for the test file, both reachable from the resumption base `98ccadb`.
**Every break below was applied to the rule file from a pristine copy of the
hash above and restored by overwriting from that copy, with the restored file's
hash re-checked against `5196ef96…` after each row** — not by `git checkout --`,
whose multi-path and uncommitted-work traps the skill records.

## The seam, measured by its first customer

**The seam's verdict holds. `KI-GRID-001` cost one new file and no edit to any
existing file.** `crates/kicli/src/lint/rules/` did not exist; creating it and
dropping `grid.rs` in was the whole registration. `build.rs` already watches
`src/lint` for exactly this — *"a rule directory that does not exist yet has no
mtime to watch, and the first rule file added creates it"* — so the first build
after the file appeared picked it up with no `touch` and no `cargo clean`.

Two costs the seam carries, both already recorded at the checkpoint and both
confirmed here rather than discovered:

- `cargo fmt --check` cannot see the rule file. It is formatted by `rustfmt
  --edition 2024` run directly, and held by `cargo test -p kicli --test
  rule_files_are_formatted`, which passes.
- The rule file is reached through a generated `include!`, so a rule author who
  forgets `pub static RULES` gets a link error rather than a missing rule.
  `the_rule_is_registered_from_its_own_file` asserts the registry names the file
  `grid` and holds exactly one rule with this code, which is the presence
  control every other check in the file stands on.

## What the rule covers

Six connectable classes, from `research/style-rules.md` §4: symbol pins (through
`resolve_pins`, so the **resolved** position rather than the placement anchor),
wire endpoints (both ends, separately), junctions, no-connect markers, label
anchors of every label kind, and sheet pin positions. Arithmetic is
`Point::is_on_grid`, an integer modulus against `GRID = Iu(12_700)`. No
tolerance, no rounding, no float — `the_linter_holds_no_floating_point` sweeps
`src/lint/**` including rule files and passes.

Field and graphic text positions are never read. The exemption is therefore by
construction in the rule and by check in the test, which is the order that
matters: the check fails the day someone widens the rule.

## Task text corrected against measured reality

**`research/style-rules.md` §4 and this entry's own "Overlap with ERC" say
KiCad's `endpoint_off_grid` covers "wire endpoints only". Measured against KiCad
10.0.5 it covers symbol pins as well.** KiCad's own message says so:

```
[endpoint_off_grid]: Symbol pin or wire end off connection grid
    ; warning
    @(51.435 mm, 46.99 mm): Symbol R1 Pin 1 [Passive, Line]
```

The measurement, one drawing per class, `kicad-cli sch erc --severity-all`
against `kicad-cli --version` = `10.0.5`, counting `endpoint_off_grid` lines:

| Displaced class | KiCad reports | `KI-GRID-001` reports |
|---|---|---|
| symbol pin | **yes** | yes |
| wire endpoint | **yes** | yes |
| junction | no | yes |
| no-connect | no | yes |
| label anchor | no | yes |
| sheet pin | no | yes |
| symbol anchor, pins on grid | no | no |
| off-grid free text | no | no |

**The overlap T2's mechanism has to cover is two classes, not one.** This is
recorded here rather than fixed in `research/style-rules.md`, which is not this
lane's file. Carried gap: §4's "Overlap with ERC" line for `KI-GRID-001` is
wrong as written and owes a one-line repair.

The last two rows are the two independent confirmations worth having. KiCad
agrees that an off-grid *placement anchor* whose pins land on the grid is not an
off-grid condition — so the rule reading resolved pin positions is not a
kicli convention, it is what the tool does. And KiCad reports nothing about the
off-grid free text every drawing here carries, which is Contradiction 2 from the
other side.

## What the non-double-counting seam will owe (T2 was not merged at this base)

The rule reports its own findings and knows nothing of ERC. Nothing here stubs
or anticipates T2's mechanism, per the brief. What that mechanism will need from
this rule, stated so T2 does not have to re-derive it:

1. **The overlap is `endpoint_off_grid` against `KI-GRID-001`, on two classes**
   — symbol pins and wire endpoints — not one. Suppressing on the strength of
   §4's "wire endpoints only" would leave every off-grid pin double-reported.
2. **A positional join works for pins and FAILS for wires. This is the single
   most important line in this section, and the entry previously said the
   opposite.**

   **Repaired by the resuming lane, on its own re-measurement.** The sentence
   here used to read *"The join is positional … they agree exactly. No tolerance
   is needed on either side."* That was written at `f952966` and **commit
   `0351d00` then measured the counter-case into the test file without this
   paragraph being updated**, so the entry contradicted its own test. The
   resuming lane re-ran the measurement by hand rather than inferring it from a
   passing assertion:

   ```
   # the drawing: one wire, on-grid end to off-grid end
   (wire (pts (xy 25.4 25.4) (xy 38.735 25.4)) …)

   # KiCad 10.0.5's own report for it
   [endpoint_off_grid]: Symbol pin or wire end off connection grid
       ; warning
       @(25.40 mm, 25.40 mm): Horizontal Wire, length 13.335 mm
   ```

   **The off-grid end is `38.735`. KiCad names `25.40` — the wire's anchor,
   which is ON the grid.** `KI-GRID-001` names `38.735`, because it reports the
   point that actually fails to connect.

   For a **pin** the two instruments agree to the digit: KiCad's
   `@(51.435 mm, 46.99 mm)` is this rule's `Point(514_350, 469_900)`, and the
   test asserts that by formatting the finding's own `Iu` and matching KiCad's
   report text.

   **So a T2 seam that suppresses `KI-GRID-001` wherever ERC reported
   `endpoint_off_grid` at the same position would suppress correctly for pins
   and not at all for wires** — every off-grid wire end would be reported twice.
   A seam that instead suppresses per *object* (the wire's uuid) rather than per
   *point* gets both classes right. **The join T2 needs is by object, not by
   coordinate**, and that is a measured conclusion rather than a preference.
3. **Severity differs and that is not a conflict.** KiCad's is a warning, this
   one is `Tier::One`/`Severity::Error`. A merged report that keeps the blocking
   verdict and drops the duplicate line is the shape the catalogue asks for.

## Saturation: declared as NEVER, and why that is a finding rather than a shrug

**The rule declares `Saturation::NEVER`, which is the trait default, and it does
so because the denominator this rule needs does not exist.**

Per the BLOCKED 3 ruling every rule author declares what the rule counts. This
rule counts **connectable points**. `Counted` (`crates/kicli/src/lint/gate.rs`)
holds `Symbols`, `Wires` and `Nothing`, and `Density` carries a symbol count and
a wire count and nothing else. There is no honest declaration available:

- `Counted::Symbols` would divide a count of *points* by a count of *symbols*.
  A sheet of two symbols and ten off-grid wire endpoints would read "10 of 2".
- `Counted::Wires` is wrong the same way, and worse for the pin class.
- `Counted::Nothing` is documented as *"a rule that reports at most once about
  the whole sheet"*, which this rule is not either.

So `NEVER` is the least-wrong of three wrong answers, and it is recorded as such
rather than presented as a considered choice.

**PROPOSED (lane a): `Counted` owes a `ConnectablePoints` variant, and `Density`
owes the count behind it.** Adding it means editing `gate.rs` and `score.rs`,
both existing files and both outside this lane's scope, so it is reported and
not done. Recommendation: Phase 3 or Phase 4 adds the variant when the first
**Tier 2** rule needs it. Revisit trigger: any Tier 2 rule whose findings are
per connectable point.

**And the note `RULES.md` invited: yes, declaring saturation for a blocking rule
is inert, and it is inert by construction rather than by accident.** `Gate::of`
reads `if rule.tier == Tier::One { Blocking } else if saturation.is_reached(..)`
— a Tier 1 rule's saturation is never consulted on any input. So no value this
rule could declare changes any outcome, and the missing denominator above costs
nothing *today*. It is still worth the variant later, because the first Tier 2
rule that counts points will hit it for real and will hit it as a retrofit,
which is exactly what sequencing T4 first was meant to avoid.

## PROPOSED (lane a): three connectable things the published rule does not name

Recorded rather than decided, because widening a blocking rule is a value-level
judgement and `RULES.md` parks those against the north star.

1. **Bus endpoints.** `research/style-rules.md` §4 says "wire endpoint"; the
   rule implements `LineKind::Wire` and skips `LineKind::Bus`. A bus end off the
   grid fails to connect in exactly the way a wire end does.
2. **Bus entries.** `Item::BusEntry` carries a position and joins a wire to a
   bus. Not named in §4, not covered.
3. **The sheet symbol's own corner** is deliberately *not* covered — it is a
   body corner, not a connection point. Only `sheet.pins` are read. This one is
   a decision, not a gap.

Recommendation: cover buses and bus entries, in the same rule, when the
catalogue text is next amended. Both are one line each and both are the same
defect class. Revisit trigger: a dogfood run or a corpus sheet where a bus is
drawn off grid.

## Falsification: sixteen breaks, and which named check caught each

**Run by the resuming lane. Every row below was executed; none is planned.**
Fourteen breaks are in the rule, two are in the test file's own presence-control
helpers. Each was applied to a pristine copy of the good state, run, then
restored by overwrite with the restored file's hash re-checked — the hashes
printed by every row are in `/tmp/kicli-scratch/a2/rows/*.summary.txt`.

### How the suite was run, and the one deviation from the skill's letter

The skill says run the whole suite with `--no-fail-fast`, because cargo stops
after the first failing **target** and silently truncates the caught-by list.
**Rows 1–4 were run exactly that way** — `cargo test -p kicli --no-fail-fast`
over every target.

Then four other lanes began running full suites in parallel on the same machine
and a single row went from 100 seconds to 13 minutes, with every `cargo`
process blocked rather than computing. **The remaining rows were run against a
reduced target list instead**, and the reduction was *validated rather than
assumed*: rows 1 and 2 were re-run under the reduced list and **reproduced their
whole-crate caught-by lists exactly**. The list is every target that touches the
registry or the lint engine (`grep -ln 'lint::registry\|registry::all\|Engine::of\|Report::of'`)
plus every `src/lint/**` sweep and `rule_files_are_formatted` — fourteen
targets, named in `/tmp/kicli-scratch/a2/runrow2.sh`.

**`--no-fail-fast` was passed on every run.** The deviation is the target *set*,
not the flag, and it is recorded because an unvalidated reduction would be
exactly the silent truncation the skill's line exists to prevent.

`KICLI_TEST_KICAD_CLI=1` was set on every row, so `kicad_agrees_where_its_own_check_reaches`
appears in the lists below. **It is environment-gated and does not count toward
this lane being done** — it is listed because leaving a real catcher out of a
caught-by list is the same under-count the skill warns about.

### The table

| # | What was broken, in `grid.rs` unless noted | Caught by |
|---|---|---|
| 1 | the whole `Item::Symbol` arm removed (pins never read) | `an_off_grid_pin_is_found`, `one_off_grid_pin_fails_the_gate_and_leaves_the_score_alone`, `the_rule_finds_what_kicad_found_in_a_committed_drawing`, `kicad_agrees_…` |
| 2 | the whole `Item::Line`/`LineKind::Wire` arm removed | `an_off_grid_wire_endpoint_is_found`, `the_rule_finds_what_kicad_found_in_a_committed_drawing`, `kicad_agrees_…` |
| 3 | the `Item::Junction` arm removed | `an_off_grid_junction_is_found`, `off_grid_text_is_exempt_at_a_coordinate_the_rule_does_report`, `kicad_agrees_…` |
| 4 | the `Item::NoConnect` arm removed | `an_off_grid_no_connect_is_found`, `kicad_agrees_…` |
| 5 | the `Item::Label` arm removed | `an_off_grid_label_anchor_is_found`, `kicad_agrees_…` |
| 6 | the `Item::Sheet` arm removed (sheet pins never read) | `an_off_grid_sheet_pin_is_found`, `kicad_agrees_…` |
| 7 | **widened to graphic text** — an `Item::Text` arm added before `_ => {}` | all six class checks, `a_drawing_on_the_grid_reports_nothing`, `a_symbol_anchor_off_the_grid_is_not_itself_a_finding`, `off_grid_text_is_exempt_…`, `one_off_grid_pin_fails_the_gate_…`, `kicad_agrees_…` — **eleven** |
| 8 | the pin loop replaced by a test of `symbol.at`, the **placement anchor** | `an_off_grid_pin_is_found`, `a_symbol_anchor_off_the_grid_is_not_itself_a_finding`, `one_off_grid_pin_fails_the_gate_…`, `the_rule_finds_what_kicad_found_…`, `kicad_agrees_…` |
| 9 | `for end in [line.from, line.to]` → `[line.from]` (reported per wire, not per end) | `an_off_grid_wire_endpoint_is_found`, `the_rule_finds_what_kicad_found_…`, `kicad_agrees_…` |
| 10 | the junction **double-reported** — a second identical `found.record` in the same arm | `an_off_grid_junction_is_found`, `off_grid_text_is_exempt_…`, `kicad_agrees_…` |
| 11 | `pub static RULES` emptied — the rule compiles and never registers | `the_rule_is_registered_from_its_own_file` **and ten others** |
| 12 | `Tier::One` → `Tier::Two` | `the_rule_is_registered_from_its_own_file`, `one_off_grid_pin_fails_the_gate_and_leaves_the_score_alone` |
| 13 | **a half-step tolerance** — `is_on_grid()` replaced at all six call sites by `near_grid`, which admits any point within 6 350 IU of a grid line (Constitution §4's forbidden rounding) | all six class checks, `off_grid_text_is_exempt_…`, `one_off_grid_pin_fails_the_gate_…`, `the_rule_finds_what_kicad_found_…`, `kicad_agrees_…` — **ten** |
| 14 | **widened to fields** — symbol `fields` positions reported alongside pins | `kicads_own_field_autoplacement_is_exempt`, **and nothing else** (see the finding below) |
| 15 | in the **test file**: `exempt_positions` short-circuited to return an empty `Vec` | `kicads_own_field_autoplacement_is_exempt` (its presence control 1) |
| 16 | in the **test file**: `connectable_classes` short-circuited to return `0` | `kicads_own_field_autoplacement_is_exempt` (its presence control 2) |

**Every one of the fourteen checks in the file appears in at least one row.**
The six class breaks (1–6) each fail **their own** class check and no other
class's, which is the independence the entry claimed and had not shown: a
single fixture carrying one off-grid object of every class could not produce
that pattern.

Rows 15 and 16 exist because `kicads_own_field_autoplacement_is_exempt` asserts
an **absence**, and an absence check that read nothing would pass. Both of its
presence controls were shown to fail when blinded, so neither is decoration.

### Row 13 did not compile on the first attempt, and that is recorded

The first form of the tolerance break used `i64` for the half-step constant;
`Iu` is `pub struct Iu(pub i32)`, so it failed with two `E0308`s and the row
produced **no** caught-by list. A break that does not compile is not a
falsification — it was repaired to `i32` and re-run, and the row above is the
re-run. Recorded because "0 targets failed" and "the break did not compile"
look identical in a summary, and the first reads as a blind instrument.

## Finding: the field exemption rests on ONE fixture, and the probe drawings add nothing to it

**Measured, row 14.** Widening the rule to symbol field positions was caught by
`kicads_own_field_autoplacement_is_exempt` **and by no other check** — not by
`a_drawing_on_the_grid_reports_nothing`, and not by any of the eight drawings
this test file builds through the probe.

The reason is that the exemption has **two halves that are protected very
differently**:

- **graphic text** (`Item::Text`) is protected by the standing control in every
  probe-built drawing, which carries one off-grid free text. Row 7 shows that
  widening to text fails **eleven** checks at once.
- **fields** (`Field::at` on symbols, sheets and labels) are protected by
  `sch/item_zoo.kicad_sch` **alone**, because the probe places symbols whose
  fields happen to land on the grid, so no probe drawing can distinguish a rule
  that reads fields from one that does not.

That is a single point of failure for half of the exemption, and it is a fixture
rather than a check: if `item_zoo.kicad_sch` were ever regenerated with its
fields on grid, row 14's break would become invisible and nothing would go red.

**PROPOSED (lane a): the probe owes a way to place a symbol with a deliberately
off-grid field**, so the field half of the exemption gets the same standing
control the text half already has. Not done here: it means editing
`crates/kicli-probe`, which is outside this lane's scope. Revisit trigger: any
regeneration of `sch/item_zoo.kicad_sch`, which would silently remove the only
instrument covering the field half.

## Finding: nothing generic catches a rule file that registers no rules

**Measured, row 11.** Emptying `pub static RULES` was caught by eleven checks —
**all eleven in this lane's own test file**, and *none* in
`crates/kicli/tests/lint_rules_register_from_their_own_files.rs`, the test that
owns the seam.

That test has two arms and neither reaches this break. Its **directory arm**
compares the rule directory on disk with the generated list, and `grid.rs` is
still on disk and still in the list when its `RULES` is empty. Its **engine
arm** runs rules over a drawing, but the rules it runs are the specimens in
`tests/specimen_rules/`, not the crate's own. So *"a real rule file whose
`RULES` is empty"* is a state the seam's own test accepts.

Today that costs nothing, because `the_rule_is_registered_from_its_own_file`
exists in this file and the equivalent exists for `KI-CONN-001`. The cost is
that it is **per-rule discipline rather than a property of the seam**: the
first rule author who forgets the check gets a rule that compiles, registers
its file, runs nothing, and is reported by no gate.

**PROPOSED (lane a): the seam test's engine arm should assert that every file
in the generated registry contributes at least one rule.** One assertion, in
`lint_rules_register_from_their_own_files.rs`, which is not this lane's file —
so it is reported, not done. Revisit trigger: the third rule file, or any rule
whose own test file omits a registration check.

## Completion check, as run

```
cargo test -p kicli --test rule_files_are_formatted
  test every_hidden_rule_file_is_formatted ... ok

cargo xtask check
  pass  fmt / clippy / test / doc / deny / clean
  skip  corpus      (--corpus not given)
  skip  kicad-cli   (--corpus not given)
  INCOMPLETE: 2 of 8 arms did not run. 6 passed, 0 failed.
```

**The two skips are the opt-in arms and are expected, not a failure.** Per
`CLAUDE.md`, corpus- and environment-gated arms never count toward a lane being
done; the orchestrator's merged run with `--corpus` is what settles them.

The formatting test is listed separately because it is the **only** thing that
formats this rule file — `cargo fmt --check` passes without ever reading it,
which is the seam cost ratified at checkpoint 1.

### Scope, re-measured at hand-off rather than re-read

`git diff --stat 25aea90..HEAD` — three files, every one of them IN scope:

| File | Lines | Scope |
|---|---|---|
| `crates/kicli/src/lint/rules/grid.rs` | +126 | IN |
| `crates/kicli/tests/lint_grid_covers_every_connectable_class.rs` | +620 | IN |
| `tasks/M5/phase2-ki-grid-001-off-grid.md` | this entry | IN |

**No merge hotspot was edited.** `Cargo.toml`, `lib.rs`, `build.rs`,
`AGENT.md`, `spec/SPEC.md`, `tests/command_surface.rs`, `kicli.toml`'s
`[rules]` table, the fixture `MANIFEST` and `crates/kicli/src/lint/drawing.rs`
are all untouched — confirmed by the diff above being exactly three paths.

**`drawing.rs` was not needed.** It is named a hotspot in practice because
`Drawing` is all `Rule::examine` receives, but this rule needed only
`schematic().items` and `definition_of`, both of which it already exposes.

### What the hotspots owe

| Hotspot | What it owes | Why this lane could not do it |
|---|---|---|
| `kicli.toml` `[rules]` | `grid = "50mil"` and `grid.exempt_text = true`. The defaults are already correct — `GRID = Iu(12_700)` **is** 50 mil and text **is** exempt — so this is a missing knob, not a wrong value. | the `[rules]` table is a merge hotspot |
| `research/style-rules.md` §4 | a one-line repair: the "Overlap with ERC" line says `endpoint_off_grid` covers *wire endpoints only*; measured against KiCad 10.0.5 it covers **symbol pins as well**. | not this lane's file |
| `lint_rules_register_from_their_own_files.rs` | one assertion that every file in the generated registry contributes at least one rule (see the finding above) | not this lane's file |
| `crates/kicli-probe` | a way to place a symbol with an off-grid **field**, so the field half of the text exemption gets a standing control (see the finding above) | not this lane's file |
| `crates/kicli/src/lint/gate.rs`, `score.rs` | a `Counted::ConnectablePoints` variant and the count behind it | both existing files, outside scope |

## Carried gaps

- **The knobs are not read.** §4 names `grid = "50mil"` and
  `grid.exempt_text = true`; `kicli.toml`'s `[rules]` table is a merge hotspot
  and was not touched. `Point::is_on_grid` is hardcoded to `GRID = Iu(12_700)`,
  which *is* 50 mil, so the default is right and only the knob is missing. Owed
  by whoever owns the `[rules]` table.
- **A finding carries no `fix` command.** `Findings::record_with_fix` exists and
  is unused here, because no command on the current surface moves a pin, a
  junction or a sheet pin onto the grid, and a suggested command that does not
  exist is worse than none. Owed when such a command exists.
- **A placement whose definition the file does not embed contributes no pin
  findings.** `drawing.definition_of` returns `None` and the symbol is skipped.
  Every `kicad_sch` KiCad writes embeds `lib_symbols`, so this is a hole only a
  hand-broken file reaches, but it is a silent skip and it is recorded as one.
