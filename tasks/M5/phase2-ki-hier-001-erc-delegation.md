# `KI-HIER-001` — sheet pin / hierarchical label mismatch (Phase 2, lane B)

**Provenance: `tasks/M5/PLAN.md` Phase 2, RATIFIED at the M5 plan review.**
**Depends on T1, T3, T4 — and on T2, which builds the ERC consumption seam.
Do not dispatch this before T2 merges.**

## This task is a DELEGATION, not a detector

`research/style-rules.md` §4:

> **T** 1. **Delegated to ERC** (`hier_label_mismatch`). Listed here only so the
> catalogue is complete; kicli reports ERC's finding and gates on it.

`spec/SPEC.md` §11.1: **KiCad 10's ERC implements 47 checks and kicli's lint
engine implements none of them.** `ENGINEERING.md` and the plan's exit-criteria
table both name **"ERC layering"** as a gate that can fail on *"kicli
implementing an ERC check, or double-counting one it reports"*.

**So the deliverable is the attribution and the gate, and the thing that must be
provably absent is a detector.** If you find yourself reading sheet pins and
comparing them to hierarchical labels, **stop** — that is the failure this task
exists to prevent, and it will pass every test you write for it.

**This rule's tier is one of only four in the catalogue with source support**:
ERC's own default severity for `hier_label_mismatch` is **ERROR**. Record that —
it means kicli is agreeing with KiCad rather than asserting over it.

## Goal state, as the checks that prove it

### 1. The finding is ERC's, and says so

An agent reading the output must be able to tell **which tool found this**, and
must not be sent to kicli's source to understand a check kicli does not
implement. §14.3: kicli **relabels** ERC severities for its own output and
**never edits `.kicad_pro`** — the `kicli.toml` ERC severity mapping is a
**presentation** mapping and the docs must say so.

### 2. It gates

Tier 1. A `hier_label_mismatch` sets `gate: fail`, using T4's mechanism. **It
must not move the score** — that is T4's first direction check, and this rule is
the first real rule to exercise it.

### 3. `kicad-cli` absence is exit 6, not a silent pass

`spec/SPEC.md` §11.2: `sch score --gate` **may require `kicad-cli`, because half
of Tier 1 is ERC-owned** — and this rule is that half. §6.1: absence is a
structured error and **exit 6**.

**The dangerous behaviour is the quiet one:** a `--gate` run with no `kicad-cli`
that reports `gate: pass` has told an agent its build is fine **on the strength
of a check that never ran.** That is this session's `xtask` finding in a
different costume, and it is worse here because it is the shipped product.
**Write that check first.**

### 4. No double-counting, and no second detector

Use T2's mechanism. **Do not build a second one.** If T2's mechanism does not
reach this case, **report it rather than working around it.**

### 5. Saturation declared

Declare it per T4, and note in the entry that a delegated rule's denominator is
a genuine question — say what you chose and why.

## Falsification obligation

- **A fixture with a real mismatch fires**, and the finding is attributed to ERC.
  Build it and **confirm KiCad agrees** by running `kicad-cli sch erc` on it —
  `ENGINEERING.md`: fixture expectations are verified against KiCad, never
  hand-asserted.
- **A matching fixture does not fire** — the presence control for an absence
  claim.
- **The absent-`kicad-cli` path is shown**, and it must be distinguishable from
  "present and broken": those are **different exit codes in §6.2's table and the
  same in a careless test.**
- **The no-detector claim needs a check that can fail.** This is the hard one and
  it is the point of the task: state how you would know if a future edit added a
  detector here. The `ERC layering` gate in the plan's exit-criteria table is
  where that check belongs. **If you cannot make it fail on anything, say so
  plainly** — a gate that cannot fail is worse than none, because it spends the
  credibility of a real one.

## Environment gate — what "done" means here

**This task's real checks need `kicad-cli`.** `CLAUDE.md`: environment-gated
checks in a lane worktree **never count toward done**; only the orchestrator's
merged run does. A lane may still run them to MAKE a measurement, and this task
owes several. **Say which checks ran in the lane, with what `kicad-cli` version,
and which wait on the merged run. Do not report an environment-gated green as
done.**

## Scope

**IN**
- `crates/kicli/src/lint/rules/hier.rs` — **new file, no edit to any existing
  file**
- new test files and new fixtures under `crates/kicli/tests/`, **plus the
  `MANIFEST` line for each fixture** — that line and nothing else
- this file, for the evidence, written AS YOU WORK

**MERGE HOTSPOTS — report, do not edit.** As `phase2-ki-conn-001-pin-on-wire.md`.

**OUT** — every other rule file, `crates/kicli/src/kicad/**` (T2's, and merged
before you start — **read it, do not change it**), every other entry.

**If this scope proves wrong, the named goal state and its checks win.** Say so
in your first paragraph.

## Completion check

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo xtask check
cargo test -p kicli --test rule_files_are_formatted
cargo test -p kicli --test command_kicad_gateway
```

plus the ERC-dependent checks by name, with the `kicad-cli` version recorded and
each shown capable of failing.

---

# Evidence and deliverable (lane hier)

**Base commit `8c3ec73`, worktree `.claude/worktrees/lane-hier`, verified clean
as the lane's first action.** `git log --oneline -1` printed
`8c3ec73 tasks: the ERC partial-overlap gap, and a cross-review disagreement
resolved (M5)` and `git status --porcelain` printed nothing. No fast-forward was
needed. `kicad-cli` on this machine is **10.0.5**, at `/opt/homebrew/bin/kicad-cli`.

**Scope: as the entry's IN list, with nothing outside it.** One new rule file,
two new test files, six new fixture files and six `MANIFEST` lines, and this
section. No merge hotspot was touched; `crates/kicli/src/lint/erc.rs` and
`drawing.rs` were read and not changed.

## The deliverable

`crates/kicli/src/lint/rules/hier.rs` — `KI-HIER-001`, Tier 1, weight
`Penalty::ZERO` by the trait's default, `Saturation::NEVER`. Its `examine` is:

```
if drawing.rule_check().covers(CHECK) {
    delegate(CHECK, drawing, found);
}
```

and no third line. **T2's seam reached this case exactly as designed** and
nothing was built beside it: the file names `RuleCheck::covers` and
`erc::delegate` and no other mechanism.

**One deviation from the seam's published shape, and it is cosmetic.** The seam
writes `delegate(…); return;` because a rule with its own geometry continues
after the branch. This rule has none, so the `return` would be the final
statement of the function and `clippy::needless_return` refuses it. Measured:
`cargo clippy -p kicli --lib` is clean as written. The rustdoc on `examine`
records the deviation and the reason, so a reader comparing the file against the
seam's documentation does not read it as a divergence of substance.

## The tier's source, which is the thing this rule has and its five siblings do not

**Measured, not cited.** `tests/fixtures/erc/hier/hier.kicad_pro` sets
`"rule_severities": {}` — the project asks for nothing. `kicad-cli sch erc` on
that project reports both `hier_label_mismatch` violations at
`"severity": "error"`, and `hier_label_mismatch` is **absent** from the report's
`ignored_checks` list (which does hold `single_global_label` and
`four_way_junction`, KiCad's own two `IGNORE` defaults, exactly as §11.1 says).

So Tier 1 here is **kicli agreeing with KiCad's own default severity**, not
kicli ruling over it. That is recorded in the rule file's module documentation
and asserted by
`lint_hier_label_mismatch::kicad_reports_the_mismatch_twice_and_says_nothing_about_the_matched_pair`.
`RULES.md` checkpoint 1: *"a Tier 1 rule whose entry does not argue its tier from
the north star is incomplete"* — the argument is in the rule file's module
documentation and is the north star's second sentence directly: a sheet pin with
no label behind it is a wire the drawing shows going somewhere and the netlist
does not, so a reader who follows the drawing is misled and a reader who follows
the netlist cannot see the drawing's claim at all.

## The fixture gap T2 recorded, closed — and the route

T2 measured that `hier_label_mismatch` appears in **no** committed report. It
does now.

**New fixture: `crates/kicli/tests/fixtures/erc/hier/`.** One project, three
sheets, carrying **both** cases:

| File | What it is |
|---|---|
| `hier.kicad_sch` | root; two sheet symbols, pin `READY` on `matched`, pin `BUSY` on `mismatched` |
| `matched.kicad_sch` | hierarchical label `READY` — **the presence control** |
| `mismatched.kicad_sch` | hierarchical label `WRONG` |
| `hier.kicad_pro` | `"rule_severities": {}` — the project asks for nothing |
| `hier.erc.json`, `hier.erc.txt` | `kicad-cli` 10.0.5's own two reports of one run |

**Route: hand-written s-expressions, canonicalised by nothing.** The probe route
was available — `kicli_probe::drawing::Probe::sheet_of_size` writes sheet ports
and `LabelKind::Hierarchical` writes hierarchical labels, so this is **not**
`lane-txt`'s situation, where the probe structurally could not write the shape.
It was not taken, and the reason is recorded rather than assumed: the task owes a
**committed** fixture (T2's own note: *"an end-to-end check over KiCad's own
bytes would need a new fixture"*), and the probe writes into scratch. A probe
check would also have had to place two children under one parent, which
`sheet_of_size` hardcodes as `(page "2")` for both.

**`kicad-cli sch upgrade` was NOT the canonicalising route and could not be**,
which is a measurement worth recording for the next fixture author: run on all
three files it printed *"Schematic file was not updated"* three times and changed
no byte, because the files already carry the current `(version 20260306)`.
`lane-txt`'s route — committed fixtures plus `kicad-cli sch upgrade` — therefore
does **not** generalise to a fixture written at the current version. The three
schematics were written in KiCad's own layout (modelled on
`project/healthy/healthy.kicad_sch`, which is `kicad-cli` provenance) and are
**fixed points of kicli's prettifier on the first attempt**: `cargo test -p kicli
--test round_trip` passes all four checks with them marked `canonical yes`, which
includes `prettify_reproduces_kicad_layout` and `emit_reproduces_input_bytes`.
Provenance in the `MANIFEST` is `hand` for the three schematics and the project
file — as `erc/canary/canary.kicad_sch` is — and `kicad-cli` for the two reports.

**`MANIFEST`: six lines appended and nothing else**, one per new fixture, as the
scope requires. `cargo test -p kicli --test fixtures_match_manifest` passes;
`fixtures_match_manifest` asserts the two directions, so a fixture without its
line would have failed.

### What KiCad itself says, which is the fixture's whole expectation

`ENGINEERING.md`: fixture expectations are verified against KiCad, never
hand-asserted. **The measurement was made before any assertion was written** —
the candidate was built in `/tmp/kicli-scratch/hier/build`, `kicad-cli sch erc`
was run on it, and only then was anything committed. `kicad-cli` 10.0.5, over the
committed bytes:

```
***** Sheet /
[hier_label_mismatch]: Sheet pin BUSY has no matching hierarchical label inside the sheet
    ; error
    @(101.60 mm, 31.75 mm): Hierarchical Sheet Pin 'BUSY'
***** Sheet /matched/
***** Sheet /mismatched/
[hier_label_mismatch]: Hierarchical label WRONG has no matching sheet pin in the parent sheet
    ; error
    @(50.80 mm, 50.80 mm): Hierarchical Label 'WRONG'
```

Three facts in that, each load-bearing:

1. **Both directions fire** — the pin with no label, and the label with no pin —
   and KiCad reports them **on two different sheets**, so the delegation's sheet
   filter (`delegate` reads `of_check(check, drawing.path())`) is exercised
   rather than assumed. One finding lands on the root placement and one on
   `/mismatched/`.
2. **The matched pair is silent, in the same run.** `matched` carries pin `READY`
   and label `READY` and KiCad says nothing about it, while the same report
   names `Hierarchical Sheet Pin 'READY'` and `Hierarchical Label 'READY'` under
   other checks — so the silence is a verdict and not a sheet KiCad never opened.
   **The presence control is intrinsic to the fixture rather than a second
   project**, which is the stronger form: a run that stopped reading after the
   first sheet could not produce this pair of answers.
3. **Severity `error` by KiCad's default**, over a project that sets none. That
   is the tier's source, above.

The oracle in the test is read from the **text** report by a line-oriented reader
of this lane's own (`mismatches_kicad_reported`), while the rule's input is the
**JSON** report through `kicad::erc::read_reports`. The two share the fixture's
bytes and nothing above them, so the comparison is not one parse against itself —
the sharper form of the degenerate-equality trap for a rule whose only input is
the report it is measured against.

## The hazard: what this rule does when `covers()` is false because nothing ran

**It reports nothing, the gate says `pass`, and that is the worst output this
milestone can produce.** `a_broken_drawing_passes_the_gate_when_nobody_ran_the_rule_check`
is that sentence as a check, over the committed fixture — the same bytes KiCad
calls two errors — and it is the executable equivalent of `KI-CONN-001`'s
`a_drawing_with_no_partition_attached_cannot_answer_and_says_nothing`.

The check measures four readings of one drawing:

| Reading | `has_run()` | `KI-HIER-001` findings | Gate |
|---|---|---|---|
| `RuleCheck::NOT_RUN` — nobody asked KiCad | `false` | 0 | **pass** |
| `RuleCheck::ran(vec![], vec![])` — KiCad ran, found nothing | `true` | 0 | pass |
| `ran(violations, ignored: [hier_label_mismatch])` | `true` | 0 | pass |
| the committed reading | `true` | **2** | **fail** |

**Rows 1 and 2 are byte-identical in the findings and byte-identical in
`Report::text()`**, and the check asserts that equality rather than inferring it.
That is the finding, and it is what makes the next paragraph a conclusion rather
than an opinion.

### The honest answer cannot live in a rule file, and here is what the surface owes

**PROPOSED.** The repair belongs to the `sch score` command surface
(`tasks/M5/phase2-sch-score-command-surface.md`), which does not exist yet, and
this lane cannot write it: that entry is out of scope and `sch score` is not
built. Recommendation, with the reason:

- **A rule must not invent a finding here.** `examine` records what is wrong with
  a *drawing*, and "the drawing was never examined" is not a fact about the
  drawing. A rule that fired on `NOT_RUN` would put a Tier 1 blocker on every
  correct hierarchy on every machine with no KiCad install, and §6.1 already says
  what the absence of `kicad-cli` is: **a structured error and exit 6**, not a
  finding.
- **So the surface must ask, before it prints a verdict.**
  `RuleCheck::has_run()` is public and already separates rows 1 and 2 of the
  table above. `sch score --gate` over a project with no `kicad-cli` must exit 6
  (§11.2, §6.1) and must **not** print `gate: pass`.
- **Row 3 is a third situation and needs its own answer, which is not exit 6.**
  The tool was there and the *project* turned the check off. The information
  needed is also already on the reading — `RuleCheck::ignored()` — and the
  surface's choice between "report it and pass" and "refuse to give a verdict"
  is a value-level judgement. **Parked against the north star rather than
  guessed.** Recommendation: name the un-run check in the `--gate` output, so an
  agent reading `gate: pass` can see which half of Tier 1 was not examined.
  `covers()` folding rows 1 and 3 together is correct for the seam and is
  precisely why the surface, not the rule, has to separate them.

`PLAN.md`'s *"ERC layering"* gate names *"double-counting"* as the other failure.
Not reachable for this rule: it has no own detector to double with, which
`the_hier_rule_adds_no_detector` is the standing proof of.

## Is the no-detector claim checkable? Yes, and here is where it stops

`crates/kicli/tests/the_hier_rule_adds_no_detector.rs`, three checks and two
halves.

**How a future edit that added a detector would be caught.** A detector has to
reach the drawing's geometry, and every road in runs through one of
`Drawing`'s accessors — `schematic()`, `doc()`, `library()`, `nets()`,
`definition_of` — or through `crate::model`, `crate::geometry` or
`crate::connectivity`. `path()` is excluded because `delegate` needs it and it
carries no geometry; `resolve_pins` is listed because it is the next thing a
pin-reading rule reaches for. The sweep strips comments first, so the rule file's
own documentation can explain at length that it never calls `drawing.schematic()`
without failing the check that says so.

**Second half, because the first is evadable by moving the code.** A detector
written in another module and reported under `KI-HIER-001` satisfies the file
sweep, so the **code** is swept for across every source of the crate with
comments stripped, and must appear in exactly one file.

**Where it stops, stated rather than papered over.** This is a textual
instrument. It cannot see a detector that reaches geometry through a helper that
names none of those paths — and what bounds that hole is not this sweep but the
companion `the_linter_holds_no_write_path`, whose whitelist is five modules of
this crate long. The derivation rule applies and is satisfied in the weak form
only: the accessor list is **derived from `Drawing`'s own `impl` block** rather
than from this lane's memory, and the boundary is written into the check's own
rustdoc. **PROPOSED, and the honest reading: the first half's vocabulary is
derived; the second half is a mechanism rather than a vocabulary, which is why it
is there.**

## Saturation: declared `NEVER`, and this rule's situation is its own

`Saturation::NEVER`, and **inert twice over** — which is the same declaration
`KI-CONN-001`, `KI-OVL-001` and `KI-TXT-001` made and *not* the same reasoning:

1. `Gate::of` never asks a Tier 1 rule what share it covered. A blocking rule
   fails on its first finding, so the share changes no verdict. This half is
   shared with the other three Tier 1 rules.
2. **This rule's denominator does not exist in `Counted`, and the reason is
   specific to it.** Its findings are **pairs across two sheets**: one sheet pin
   on the parent against one hierarchical label in the child. `Counted` offers
   the symbols or the wires of **one** sheet placement. Declaring either would be
   a false denominator, so it declares nothing and says why in the rule file.

**BLOCKED 3's "scores 67" number is NOT borrowed, and recomputing it is why.**
That number is for a rule that fires **once per counted object** on one sheet:
`n ≤ total`, normalised, capped at `w · reference`. This rule's count is bounded
by neither the parent's objects nor the child's — a sheet symbol with `k` pins
and a child with `m` labels, all disagreeing, yields up to `k + m` findings
**split across two different placements**, so no single sheet's denominator
bounds it. It is `KI-TXT-001`'s situation rather than `KI-WIRE-001`'s, and
sharper: the denominator is not merely the wrong size, it is on the wrong sheet.
The citation is therefore **omitted**, per the brief's discriminator.

## The full gate, run BEFORE the falsification table, and what it caught

**The first gated commit attempt FAILED on two of eight arms**, and neither
failure was visible to any targeted `cargo test --test <name>` run:

1. **`fmt`** — four diffs in `crates/kicli/tests/lint_hier_label_mismatch.rs`.
   Note the direction, which is the *inverse* of the registration seam's known
   cost: a rule file is invisible to `cargo fmt` and a **test** file is not, so a
   lane that formats only its rule file is still red.
2. **`test`** — `fixture_handles::every_committed_fixture_object_answers_to_a_handle_of_its_own`:
   the fixture's first uuid series, `32`, **collided with
   `erc/precision/precision.kicad_sch` on three handles** (`32000001`,
   `32000010`, `32000011`). Renumbered to series `33`, chosen by enumerating the
   series actually in use across the whole fixture tree —
   `00 01 10 11 12 13 14 15 16 17 18 19 1a 1b 1c 20 31 32` — rather than by
   picking one. The ERC oracles were regenerated from the renumbered bytes,
   because the JSON carries every object's identifier.

   **PROPOSED, carried for the next fixture author.** The uuid *series* is a
   global allocation across the whole fixture tree and is recorded nowhere: the
   `MANIFEST` header documents the `{series:02x}{n:06x}` **format** and names no
   allocation, so the only way to find a free series is a grep over every
   fixture. One line in that header giving the highest series in use would have
   saved a full gate run. Not written here, because this lane's `MANIFEST` scope
   is **its six records and nothing else** — recommendation: add it at whatever
   task next edits that header.

**The second gated commit passed**, and its tree is the good state whose content
hashes are listed below. (No SHA is cited for it: the lane's single commit has
been amended four times since, so a SHA written here would already be wrong —
which is the defect `falsification-control` names and which this paragraph is
the correction of, recorded rather than quietly fixed.)

```
pass fmt   pass clippy   pass test   pass doc   pass deny   pass clean
skip corpus      (--corpus was not given)
skip kicad-cli   (--corpus was not given)
```

`6 passed, 0 failed, 2 skipped.` The gates run as the pre-commit hook, so the
commit **is** the full run. 727 checks pass at the baseline.

## Falsification

Procedure per `.claude/skills/falsification-control/SKILL.md`. **The good state
was committed first**, because `git checkout --` restores the last
commit and two of the five files touched below were brand new. Every row ran
`cargo test -p kicli --no-fail-fast` over the **whole** crate, so no catcher list
is truncated by cargo stopping at the first failing target. Rows 8 and 9 ran with
`KICLI_TEST_KICAD_CLI=1`, because what they falsify is an environment-gated
check.

**Content hashes of the good state** (`shasum -a 256`, run from the worktree
root — the command matters, because a bare digest read under a different command
looks like a different tree):

```
ca03b4bafa23607c330b536f9852357188a59b394a294a1ae42b91935316d46c  crates/kicli/src/lint/rules/hier.rs
c2105197820f3c8e7b076aaca1fd6ce278fa6d622b04c0e380f465197010b6e0  crates/kicli/tests/lint_hier_label_mismatch.rs
ceccab18a05b64ea3714927b6db7098f66eaa98d377eb5037b5cf8ebe8a74c2a  crates/kicli/tests/the_hier_rule_adds_no_detector.rs
3b6b2b2718a10fae1ea473d67b26c8efed14dbe78e6aa34d734058dedad499cd  crates/kicli/tests/fixtures/erc/hier/hier.erc.json
87e7718bc6facba472d2a5621764b5a2ba66734add21dc3ca0e7df7a2e75d2b5  crates/kicli/tests/fixtures/erc/hier/hier.erc.txt
71c4bc1f396ee7e67b46bbfec1fa4714704d56b778d54fb4146a5d822628f50d  crates/kicli/tests/fixtures/erc/hier/hier.kicad_pro
f4f128739dc1866548c028f58f2b95062b07b52ca10566e810c49b06b6af8463  crates/kicli/tests/fixtures/erc/hier/hier.kicad_sch
2cc5602c5f865396fef31fda5175ea4ef6bc5e2fc8fb5135ffb936c0a55ec6ff  crates/kicli/tests/fixtures/erc/hier/matched.kicad_sch
9a2dbccfa1aed95e6870e8b55cdffbaac5d7bf7bded1a1704b3c4084725a1f44  crates/kicli/tests/fixtures/erc/hier/mismatched.kicad_sch
```

Each row's restore was verified by re-hashing all five mutable files against
that list; **no row reported a restore failure**, and the tree is clean at the
good-state hashes above.

| Row | What was broken, exactly | ok / FAILED | Which checks caught it |
|---|---|---|---|
| 0 | nothing — the baseline | 727 / 0 | — |
| 1 | the one line `delegate(CHECK, drawing, found);` deleted from `examine`, leaving the branch empty | 722 / 5 | `the_rule_reports_kicads_findings_on_the_sheets_kicad_named`, `a_broken_drawing_passes_the_gate_when_nobody_ran_the_rule_check` (its *"the control: it does fire"* arm), `it_fails_the_gate_and_moves_no_score`, `the_rule_reads_the_rule_check_and_nothing_else` (the `delegate(` presence control), `every_hidden_rule_file_is_formatted` |
| 2 | the seam's question inverted: `covers(CHECK)` → `!…covers(CHECK)` | 724 / 3 | `a_broken_drawing_passes_the_gate_when_nobody_ran_the_rule_check`, `the_rule_reports_kicads_findings_on_the_sheets_kicad_named`, `it_fails_the_gate_and_moves_no_score` |
| 3 | `const CHECK` changed to another real KiCad check, `"label_dangling"` | 724 / 3 | `the_rule_reads_the_rule_check_and_nothing_else` (the `hier_label_mismatch` presence control), `the_rule_reports_kicads_findings_on_the_sheets_kicad_named`, `a_broken_drawing_passes_the_gate_when_nobody_ran_the_rule_check` |
| 4 | `Tier::One` → `Tier::Two` in `tier()` | 724 / 3 | `it_fails_the_gate_and_moves_no_score`, `the_rule_reports_kicads_findings_on_the_sheets_kicad_named` (`assert_eq!(finding.tier, Tier::One)`), `the_rule_reads_the_rule_check_and_nothing_else` |
| 5 | a `weight()` of three points added to the blocking rule | **727 / 0** | **none — see the diagnosis below** |
| 6 | `for _item in &drawing.schematic().items {}` inserted as the first line of `examine` | 726 / **1** | `the_rule_reads_the_rule_check_and_nothing_else`, **and nothing else in 727 checks** |
| 7 | the sweep pointed at a rule that really does hold a detector: `RULE_FILE` → `src/lint/rules/grid.rs`, `CODE` → `KI-GRID-001` | 725 / 2 | `the_rule_reads_the_rule_check_and_nothing_else`, `the_rules_code_is_claimed_by_exactly_one_file` |
| 8 | one character of the committed text report: `Sheet pin BUSY` → `Sheet pin BUSZ` (armed) | 724 / 3 | `kicad_reports_the_mismatch_twice_and_says_nothing_about_the_matched_pair`, `the_rule_reports_kicads_findings_on_the_sheets_kicad_named`, `the_committed_reports_are_current` |
| 9 | the drawing **repaired** — `mismatched.kicad_sch`'s label `"WRONG"` → `"READY"` — with the committed reports left stale (armed) | 726 / **1** | `the_committed_reports_are_current`, **and nothing else** |
| 10 | `ScoredTwin::weight` → `Penalty::ZERO`, emptying the scorer's anti-vacuity control | 726 / 1 | `it_fails_the_gate_and_moves_no_score` (its `assert_ne!` arm) |

### Row 6 is the task's own warning, measured

The entry warns that a detector *"will pass every test you write for it"*. Row 6
is that sentence as a number: adding the literal first line of a detector to this
rule left **726 of 727 checks green**, because the rule's behaviour on the
fixture is identical either way. The one check that went red is the source
sweep. **That is the whole argument for the sweep existing**, and it is a
measurement rather than a plausibility argument.

### Row 5 is green, and it is case 1 — with the work shown and the boundary named

**Not recorded as "the break did not apply."** A weight of three points on a
Tier 1 rule changes nothing, and the structural reason is in the scorer rather
than in the rule: `RawPenalty::of` skips blocking findings **in the sum itself**,
and its own rustdoc says why — *"Blocking findings are skipped here, in the sum,
rather than by the caller. A caller that filtered would leave the next caller
free not to, and a blocking finding must never move the score whoever asks."*
So the declared weight is unreachable, and nothing could have caught it.

**The finding about this lane's own check, stated rather than buried.**
`it_fails_the_gate_and_moves_no_score`'s `raw() == RawPenalty::ZERO` arm
therefore measures **the scorer's refusal**, not **this rule's declaration**, and
cannot tell the two apart. It is still the right end-to-end assertion — it is the
property an agent reads — but the declaration's own check is T4's
`lint_gate_separates_the_tiers`, not this one. Row 10 is the complement: it shows
the same test function's anti-vacuity arm is live, so the zero above is not a
scorer that returns zero for everything.

### Row 9 is a finding about the fixture's guard, not only about the check

Repairing the drawing while leaving the committed reports claiming a mismatch
left **every unarmed check green**, because they all read the committed reports.
So the agreement between this fixture's schematics and its committed oracles is
guarded by **exactly one** check, and that check is environment-gated — which
means, per `CLAUDE.md`, by the **orchestrator's merged `cargo xtask check
--corpus` run and by nothing in a lane**. The same is true of `erc/canary` and
`erc/precision`, so this is not a new hole; it is one that had not been measured.
Recorded at the moment it was identified.

### The environment break class

`the_committed_reports_are_current` consumes a **generated value** — two reports
`kicad-cli` writes into `CARGO_TARGET_TMPDIR` — so the fifth dimension applies.
Run once from a second directory, taken out of git rather than copied, armed:

```sh
S=$(mktemp -d /tmp/kicli-scratch/hier/elsewhere.XXXXXX)
git archive HEAD | tar -x -C "$S"
( cd "$S" && KICLI_TEST_KICAD_CLI=1 cargo test -p kicli --no-fail-fast \
    --test lint_hier_label_mismatch --test the_hier_rule_adds_no_detector \
    --test round_trip --test fixtures_match_manifest --test fixture_handles \
    --test erc_canary )
```

`/tmp/kicli-scratch/hier/elsewhere.i02zJE`, **all 24 checks green**, printing
`measured against kicad-cli 10.0.5`. So nothing here asserts a property of the
worktree it was written in. The directory was deliberately **not** named `src`:
`the_four_way_rule_has_one_home` matches any path component called `src`,
including a mount path, and two reviewers have lost time to that false failure.

## Which checks ran here, and which wait on the merged run

`kicad-cli` on this machine is **10.0.5** (`/opt/homebrew/bin/kicad-cli`,
confirmed with `kicad-cli version`).

**Ran in the lane and counting toward done** — no KiCad needed: `cargo xtask
check`, 6 of 8 arms, green as the pre-commit hook of the lane's own commit; the
two named
completion checks `rule_files_are_formatted` and `command_kicad_gateway`, both
inside that run and both inside the 727-check baseline; and the eleven-row
falsification sweep.

**Ran in the lane to MAKE a measurement, counting toward no tick:**
`lint_hier_label_mismatch::the_committed_reports_are_current` under
`KICLI_TEST_KICAD_CLI=1`. It printed `measured against kicad-cli 10.0.5` and
passed in **1.55 s against 0.00 s unarmed**, the difference being the two real
`kicad-cli` runs. **Result: the committed reports are byte-identical, dates
aside, to what `kicad-cli` 10.0.5 writes today over the committed schematics.**
Rows 8 and 9 show the same check capable of failing, and row 9 shows it failing
for the reason that matters — the drawing and the reports disagreeing.

**Waiting on the orchestrator's merged run**: that check, and `cargo xtask check
--corpus` in the main checkout. `CLAUDE.md`: an environment-gated green inside a
lane worktree counts toward no tick, and **this entry claims none**.

## What the merge hotspots owe

- **`crates/kicli/src/lint/erc.rs`** — nothing. Read in full, not changed. The
  seam reached this case exactly as T2 designed it, and the module header's
  sentence *"A third, `hier_label_mismatch`, is what `KI-HIER-001` is"* is now
  true of a shipped rule rather than of a plan. **PROPOSED, one line:** the
  header's worked example could cite `rules/hier.rs` as the delegation's live
  instance, since the example is currently a `KI-JCT-001` sketch that no shipped
  rule matches.
- **`crates/kicli/src/lint/drawing.rs`** — nothing. Read, not changed.
- **`kicli.toml` `[rules]`** — `research/style-rules.md` gives every other rule a
  **knob** (`connection.pin_on_wire = "error"` and so on) and gives this one
  **none**, because §4 lists it only for catalogue completeness. **PROPOSED:
  leave it without a knob.** §14.3 is the reason and it is a strong one: a knob
  here would read as a severity control over an ERC check, and kicli's ERC
  severity mapping is a **presentation** mapping that never touches
  `.kicad_pro`. A knob that looked like it turned a KiCad check off, and did
  not, is worse than no knob. Recommendation: if a knob is wanted, it belongs to
  the ERC severity *presentation* table and not to `[rules]`.
- **`spec/SPEC.md`** — nothing owed by this rule. §11.1, §11.2, §14.1 and §14.3
  all read correctly against what was built.
- **`tests/command_surface.rs`, `AGENT.md`** — nothing yet, and something soon:
  both describe commands, and the `--gate` behaviour this rule's hazard section
  specifies is the `sch score` surface's to build. Named there, not here.

## Scope, re-measured at hand-off rather than re-read

`git diff --stat 8c3ec73 HEAD`, against the base this lane was briefed on:

```
 crates/kicli/src/lint/rules/hier.rs                | 123 +
 crates/kicli/tests/fixtures/MANIFEST               |   6 +
 crates/kicli/tests/fixtures/erc/hier/hier.erc.json | 139 +
 crates/kicli/tests/fixtures/erc/hier/hier.erc.txt  |  34 +
 crates/kicli/tests/fixtures/erc/hier/hier.kicad_pro|  11 +
 crates/kicli/tests/fixtures/erc/hier/hier.kicad_sch| 125 +
 crates/kicli/tests/fixtures/erc/hier/matched.kicad_sch    | 19 +
 crates/kicli/tests/fixtures/erc/hier/mismatched.kicad_sch | 19 +
 crates/kicli/tests/lint_hier_label_mismatch.rs     | 455 +
 crates/kicli/tests/the_hier_rule_adds_no_detector.rs | 266 +
 tasks/M5/phase2-ki-hier-001-erc-delegation.md      | 446 +
 11 files changed, 1643 insertions(+)
```

Insertions only, no deletions, **no merge hotspot**, and the six `MANIFEST`
lines are the only change to that file. No merge-forward was performed: `main`
did not move under this lane, so no scope claim here has crossed one. The lane
branch is `lane-hier`, and it carries exactly **one** commit. That commit's SHA
is deliberately not written here, because every append to this entry amends it;
the durable anchors are the nine content hashes above, which cover the rule, the
two test files and all six fixtures and are unchanged by any edit to this entry.

## One environment artefact, with a root cause this time

T2 recorded that freshly linked test binaries in a lane worktree *"stalled at
0 % CPU for minutes"* under concurrent cargo runs and called it a macOS
execution stall. **It recurred here, three times during the gated-commit runs,
and `sample` says what it is:**

```
Call graph:
    1771 Thread_246935824: Main Thread
      1771 _dyld_start  (in dyld) + 0  [0x1028989c0]
Physical footprint: 96K
```

The process is stuck **inside `dyld`, before `main`** — it is not a test
hanging, it is a binary that never starts. (`fseventsd` was at 85.8 % CPU
throughout, and the main checkout's `cargo xtask check --corpus` was stalled the
same way at 0 % CPU for over an hour.) Two consequences worth carrying:

- **It cannot be mistaken for a check failure.** A killed target is reported by
  cargo as `process didn't exit successfully … (signal: 9, SIGKILL: kill)`, never
  as `test X ... FAILED`. Verified on the one attempt that was killed: zero
  `FAILED` lines, one `signal: 9`.
- **The stall is per-exec, not per-binary.** Each killed binary ran normally on
  the next attempt, so the remedy is kill-and-retry rather than investigation of
  the test. A watchdog that kills any `target/debug/deps/` process seen at 0.0 %
  CPU in **two consecutive 20 s sweeps**, wrapped in a retry loop around `git
  commit --amend`, took the final run to green on its first attempt with no kill
  needed.

  **Carried note, because the next lane will write this watchdog too:** macOS
  `ps` has **no `etimes` keyword**, so the obvious "kill anything idle longer
  than N seconds" watchdog silently matches nothing and waits forever. It cost
  one ten-minute stall here. The two-consecutive-sweeps form needs no elapsed
  time at all.
