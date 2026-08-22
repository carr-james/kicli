# ERC consumption, and the 100× canary (Phase 1, T2)

**Provenance: `tasks/M5/PLAN.md` Phase 1, RATIFIED by James's ratification and
advisor rulings, M5 plan review.**

**Depends on T1.** The finding type T1 defines is what ERC's violations are
mapped *into*; writing this before that shape exists means writing it twice.

## What this task is, in one sentence

**kicli runs KiCad's ERC, translates its violations into kicli findings, and
never trusts the JSON's coordinates.** `spec/SPEC.md` §11.1: KiCad 10's ERC
implements 47 checks and **kicli's lint engine implements none of them.**

## Why the canary is the only check here that matters

`spec/SPEC.md` §14.2. KiCad 10.0.5's ERC JSON exporter builds its units provider
with `pcbIUScale` (1e6 IU/mm) instead of `schIUScale` (1e4), so schematic
coordinates come out **100× too small while labelled `"mm"`**. The text report is
correct. Root cause is `eeschema/erc/erc_report.cpp:161` vs `:63`
(`research/geometry.md` §3.5), and it is **still unfixed on `master`**.

**The canary exists to fail when KiCad changes its mind**, and that is its whole
purpose. §14.2, requirement 2, verbatim: *"A CANARY TEST that expects the bug: on
a committed fixture, assert `json.pos × 100 == text.pos` exactly. When upstream
fixes it, this test fails loudly and the workaround is removed, never
double-applied."*

**A silently-double-applied correction is the failure this task is guarding
against**, and it is a coordinate being wrong by 10,000× in a tool whose whole
job is where things are drawn. Weigh every design choice here against that.

## Goal state, as the checks that prove it

### 1. `kicad` owns the invocation. All of it.

`ENGINEERING.md`, Structure: *"`kicad` owns every invocation of an external
KiCad binary — discovery, the version check, the process seam and the exit-code
translation — because `lint`, `render` and `cli` all need it and none of them may
depend on another."* The module exists (`crates/kicli/src/kicad.rs`, with
`discovery.rs` and `runner.rs`); **ERC's invocation goes there, beside them, not
into `lint`.**

`lint` knows nothing of files on disk or `kicad-cli` (Dependency inversion). The
seam between them is data, and naming that seam well is most of this task.

### 2. The canary, per §14.2

- A **committed fixture** — not a fetched one, not a generated one. The
  measurement must be reproducible by anyone at any time, and it is asserting a
  property of a *specific* KiCad version.
- `json.pos × 100 == text.pos` **exactly**. Not within a tolerance: this is
  integer scale, and a tolerance would hide the fix when it lands.
- **The correction site carries a code comment naming `erc_report.cpp:161`**
  (§14.2, requirement 3). A future reader must be able to find the upstream line
  without finding this entry.
- §14.2 requirement 4: **no upstream bug report is filed** (Q35).

**Which report to read is a real decision and the spec leaves it open**: §14.2
requirement 1 permits either reading the text report, or applying a
sanity-checked ×100 correction to the JSON. The spec states the reason JSON is
otherwise preferable — *"each violation item carries the offending object's
UUID, which joins directly to kicli's handles"* (`research/kicad-cli.md` §3.1).
**Record which you chose and why**, because a later reader will otherwise assume
the choice was forced.

### 3. Absence of `kicad-cli` is a structured error and exit 6, not a panic

§14.1 and §6.1. The discovery order is §14.1's and already implemented; do not
re-invent it. Major version ≠ 10 is refused. **§6.2: kicli translates every one
of `kicad-cli`'s exit codes; raw pass-through is forbidden** — that table is
already spec'd, and `crates/kicli/src/cli/exit.rs` already has tests over the
code table. Read them first.

### 4. Severities are read-only

§14.3. ERC severities live in `.kicad_pro` (`erc.rule_severities`) and
`kicad-cli` can only filter what is reported. **kicli relabels them for its own
output and never edits `.kicad_pro`.** `kicli.toml`'s ERC severity mapping is a
*presentation* mapping, and the docs must say so — that sentence is §14.3's, and
it is a documentation obligation as much as a code one.

### 5. The two deliberate ERC exceptions are attributed, not duplicated

§11.1: `four_way_junction` → `KI-JCT-001` and `single_global_label` →
`KI-LBL-003`, because KiCad's default severity for both is `IGNORE` and an
untouched project would silently pass. **kicli attributes them clearly and does
not double-count when the project has ERC's version enabled.**

The rules themselves are Phase 3's. **What T2 owes is the mechanism that makes
non-double-counting possible** — and a check that it works, because "does not
double-count" is precisely the kind of claim that is true until someone enables
a severity.

## Falsification obligation

Per `.claude/skills/falsification-control/SKILL.md`.

- **The canary is falsified by feeding it a correctly-scaled report and watching
  it fail.** Construct one; do not wait for KiCad to fix the bug. A canary never
  shown to fail is a canary nobody has checked is alive.
- **The exit-6 path is falsified by an absent binary**, and the check must
  distinguish "absent" from "present and broken" — they are different exit codes
  in §6.2's table and the same in a careless test.
- **The double-count check is a degenerate-equality candidate**: if both the ERC
  finding and the kicli finding are derived from the same mapping call, a break
  moves them together. State what the two sides derive from.

## Environment gate — and what that means for "done"

**This task's real checks need `kicad-cli`, so a lane cannot complete it alone.**
`CLAUDE.md`: corpus- and environment-gated checks in a lane worktree never count
toward done; only the orchestrator's merged run does. A lane may still run them
to MAKE a measurement its task owes — and this task owes several.

**Say in the entry which checks ran in the lane, with what `kicad-cli` version,
and which are waiting on the merged run.** Do not report an environment-gated
green as done.

## Scope

**IN**
- `crates/kicli/src/kicad.rs` and new files under `crates/kicli/src/kicad/`
- `crates/kicli/src/lint/` — only the seam that consumes ERC findings
- new test files under `crates/kicli/tests/`
- `crates/kicli/tests/fixtures/**` — new fixtures only
- this file, for the evidence, written AS YOU WORK

**MERGE HOTSPOTS — report, do not edit.** `Cargo.toml`, `crates/kicli/src/lib.rs`,
the fixture `MANIFEST`, `AGENT.md`, `spec/SPEC.md`, `crates/kicli/tests/command_surface.rs`,
`kicli.toml`'s `[rules]` table.

**OUT** — every other module, every other entry, `tasks/M5/PLAN.md`.

**If the enumeration above proves wrong, the named goal state and its checks win
over the list.** Say so in your first paragraph, name what you touched and why.

## Completion check

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo xtask check
```

plus, named because they are what this task is:

```sh
cargo test -p kicli --test command_kicad_gateway
```

and the canary test by name, with its falsification recorded.

---

# Evidence and deliverable (lane t2)

**Base: `18ab930`, worktree clean at first action.** Verified with
`git log --oneline -1` and `git status --porcelain` before anything else; no
fast-forward was needed.

## Scope, as written

Inside the brief's IN list, with two disclosures:

- **`crates/kicli/src/lint.rs` was edited** — two lines declaring `pub mod erc;`
  and re-exporting `RuleCheck`/`KicadSeverity`, plus one documentation
  paragraph. The brief's IN list says `crates/kicli/src/lint/`, and `lint.rs`
  sits beside that directory rather than in it, so the edit is disclosed rather
  than assumed. It is not on the merge-hotspot list and no Phase 2 rule lane
  touches it: the M5 checkpoint-1 seam verdict records that adding a rule is one
  new file under `src/lint/rules/` **and no edit to any existing file**.
- **Eight lines were added to `crates/kicli/tests/fixtures/MANIFEST`**, one per
  new fixture, and nothing else in that file was touched. The brief granted this
  narrowly; the entry's own hotspot list does not, so it is named here.

Nothing else. No `Cargo.toml`, `lib.rs`, `build.rs`, `AGENT.md`, `spec/SPEC.md`,
`tests/command_surface.rs`, `kicli.toml`, `cli/`, or `lint/rules/`.

## The decision §14.2 leaves open, and what settled it

**kicli reads the JSON report and applies the ×100 correction. The text report
is read on the same run as a control, and never as a position.**

The spec permits either. The reason recorded in `spec/SPEC.md` §14.2 is the
identifiers — the JSON's items carry the offending object's UUID and the text
report's carry none, so a delegated finding read from the text report could not
be joined to a kicli handle and `KI-HIER-001` could not name an object. That
alone was already decisive.

**A second reason was measured during this task and is stronger, because it is
about the numbers themselves.** KiCad 10.0.5's text report rounds a millimetre
reading to **three decimals** (then strips trailing zeros to a floor of two).
The schematic's own resolution is 1e-4 mm. So the text report **loses the fourth
decimal of every coordinate**, while the JSON — divided by `pcbIUScale`, 1e6 —
carries the internal units exactly.

Measured on a probe built from `geometry/orientations.kicad_sch` with R1 moved,
`kicad-cli` 10.0.5, and then committed as `tests/fixtures/erc/precision`:

| placed at | text report | JSON × 100 |
|---|---|---|
| 25.4321 | `25.432` | 25.4321 |
| 25.4329 | `25.433` | 25.4329 |
| 25.4325 | `25.433` | 25.4325 |
| 25.0001 | `25.00` | 25.0001 |
| 100.9999 | `101.00` | 100.9999 |

The check that carries this into the suite is
`erc_canary::the_text_report_loses_the_fourth_decimal`, over the committed
fixture: the corrected JSON reads `Iu(254_321)` and the text report reads
`Iu(254_320)`.

**Consequence for §14.2 requirement 2's word "exactly".** `json.pos × 100 ==
text.pos` is exact only where the coordinate fits three decimals. The canary
asserts it exactly over the three committed pairs whose coordinates do
(`erc/canary`, `geometry/orientations`, `geometry/asymmetric` — 46 coordinates),
and `erc/precision` is deliberately excluded from that arm with the measurement
above as the reason. **The task text yields to measured reality here** and the
exclusion is written into the test file's own module documentation, not only
into this entry.

## A second half of the bug, not previously recorded

**The 100× scale error is in the JSON's item *descriptions* too**, because the
same wrong units provider formats them. `kicad-cli` 10.0.5 writes, of one wire,
in one run:

```
erc.txt : @(63.50 mm, 50.80 mm): Horizontal Wire, length 12.70 mm    <- correct
erc.json: "description": "Horizontal Wire, length 0.1270 mm"         <- 100x small
```

`research/geometry.md` §3.5 records the coordinate half only. kicli therefore
takes each item's **description from the text report** and its **identifier,
check key and severity from the JSON**. Asserted in
`erc_canary::the_committed_reports_are_still_the_ones_kicli_reads`, in both
directions: the text report's `length 12.70 mm` is present and the JSON's
`length 0.1270 mm` is absent.

*(`research/geometry.md` is outside this lane's scope, so the finding is
recorded here rather than appended there. It is one paragraph of §3.5 for
whoever owns that file next.)*

## The seam, for the two Phase 2 lanes waiting on it

`kicli::lint::erc`, pure data, no I/O. `crates/kicli/src/kicad/erc.rs` produces
it; `lint` never learns a process ran (`ENGINEERING.md`, dependency inversion).

```rust
kicli::lint::erc::RuleCheck        // everything KiCad's ERC said, or NOT_RUN
kicli::lint::erc::Violation        // check, severity, description, sheet, items
kicli::lint::erc::Item             // uuid, description, at (corrected Point)
kicli::lint::erc::KicadSeverity    // Error | Warning | Excluded, + relabelled()
```

On `Drawing`: `.with_rule_check(&RuleCheck) -> Drawing` and
`.rule_check() -> &RuleCheck`. A drawing built without one answers
`RuleCheck::NOT_RUN`.

A rule author writes exactly this and needs to read nothing else:

```rust
const CHECK: &str = "hier_label_mismatch";   // or "endpoint_off_grid", ...

fn examine(&self, drawing: &Drawing<'_>, found: &mut Findings<'_>) {
    if drawing.rule_check().covers(CHECK) {
        kicli::lint::erc::delegate(CHECK, drawing, found);
        return;
    }
    // KiCad did not look, so kicli must. Own geometry here.
}
```

`KI-HIER-001` is the first two lines and no third: it is entirely a delegation.

**`covers()` answers `false` for two different situations on purpose** — the
project ignored the check, and nobody ran the check at all. Both leave the
occurrence unreported by KiCad, so both must fall through to kicli's own
detection. Folding "never ran" into "covered" would make every delegating rule
fall silent on a machine with no KiCad install, which is the failure
`command_erc_gateway::a_rule_check_nobody_ran_sends_a_rule_to_its_own_detection`
exists to catch.

## The correction site

`crates/kicli/src/kicad/erc.rs`, `fn corrected`. It carries the citation §14.2
requirement 3 asks for — `eeschema/erc/erc_report.cpp:161` against `:63`, and
`include/base_units.h:72,111-114` — and states the correction as what it is: the
number was produced by dividing schematic internal units by `pcbIUScale` (1e6),
so multiplying by 1e6 recovers them. That is ×100 relative to `schIUScale`
(1e4), and the factor is derived from the two constants rather than written as a
literal 100.

**No coordinate passes through a float** (Constitution §4). `corrected` parses
the number's own decimal text with exact integer arithmetic and returns nothing
— never a rounded value — for anything it cannot read exactly (more than six
fraction digits, an exponent, a non-number). `serde_json`'s `Number::to_string`
is the text it reads.

## The run-time sanity check, and why it is not the canary

§14.2 requirement 1's word is "**sanity-checked** ×100 correction", and §14.2's
stated failure is a correction **double-applied**. A test alone catches that in
CI and not on the machine of somebody running a fixed KiCad, so
`kicad::erc::scale_verdict` re-decides the relation from the two reports **on
every run**:

| verdict | meaning | what `read_reports` does |
|---|---|---|
| `HundredTimesSmall` | corrected == text, within the text report's own step | proceed |
| `Fixed` | the JSON already agrees with the text report | **refuse**, naming `erc_report.cpp:161` and saying to remove the correction |
| `Indeterminate` | no item tells the two readings apart (every coordinate is the origin, or there are none) | proceed; the correction is a no-op on zero |
| `Unrelated` | neither reading fits | **refuse** |

The refusal is `CliFailure::Failed`, which the **existing** table in
`cli/exit.rs` already maps to `ExitCode::Operation`. No second table was built
and `cli/exit.rs` was not touched (§6.2).

**The canary's tolerance and the run-time check's are deliberately different,
and both follow from the same measurement.** The canary is exact, because §14.2
says a tolerance would hide the fix. The run-time check carries ±5 internal
units, because that is half the text report's own 0.001 mm step, measured above;
a control tighter than the instrument it reads would refuse every drawing with a
four-decimal coordinate — which is exactly the drawings `KI-GRID-001` exists to
find. A 100× error is off by ~10^4 units, so the band costs the check nothing.

## Severities are read-only (§14.3)

- kicli reads what KiCad reported and relabels it for kicli's own output:
  `KicadSeverity::relabelled`. The rustdoc on that method and on the
  `lint::erc` module both say, in §14.3's own words, that `kicli.toml`'s ERC
  severity mapping is a **presentation** mapping.
- kicli never writes `.kicad_pro`. Measured two ways:
  `command_erc_gateway::the_project_severities_are_read_and_never_written` runs
  the whole gateway over a copy of the fixture project and compares
  `canary.kicad_pro` and `canary.kicad_sch` byte for byte afterwards, **with a
  presence control** — the same check asserts that the project file really did
  turn `four_way_junction` on and that KiCad's default really did leave
  `simulation_model_issue` off, so the comparison is not over a file nothing
  read.
- Separately measured against the real binary: `kicad-cli sch erc` itself leaves
  `.kicad_pro` byte-identical and writes a `.kicad_prl` instead. That is KiCad's
  file, not the project settings §14.3 protects, and it is why the fixture is
  copied out of the tree before any real run.

## §11.1's two exceptions: attributed, not duplicated

`four_way_junction` → `KI-JCT-001` and `single_global_label` → `KI-LBL-003`.
The rules are Phase 3's; the **mechanism** is above, and the check that it works
is `command_erc_gateway::a_delegating_rule_reports_once_however_the_project_is_configured`.

**What the two sides derive from** (the brief flagged this as a
degenerate-equality candidate, correctly):

- the **attributed** count comes from the JSON `kicad-cli` 10.0.5 wrote, through
  `read_json` → `delegate`;
- the **detected** count comes from kicli's own parse of the `.kicad_sch`,
  through `Hierarchy::load` → `Drawing::schematic()`.

Their only common ancestor is the fixture itself, so a break in either moves one
and not the other. The check asserts more than the equality, precisely because
an equality alone would pass on a rule that found nothing:

- one finding in each configuration, **not two** (the double count) and **not
  zero** (the vacuous pass);
- the attributed finding's message says `reported by KiCad's ERC` and the
  detected one's does not;
- their `objects` lists differ — KiCad names the wires it joined, kicli names
  the junction it drew;
- both are reported under `KI-JCT-001`, which is the attribution half.

The fixture supports the claim rather than assuming it: `erc/canary` ships a
`.kicad_pro` that turns both §11.1 checks **on**, and `erc/precision` ships one
that leaves them at KiCad's default. Both configurations are therefore committed
KiCad output, and the second is the evidence for §11.1's own sentence — its
`ignored_checks` lists `Global label only appears once in the schematic` and
`Four connection points are joined together`, so an untouched project does
silently pass.

## The committed fixtures

Eight files, all named in `MANIFEST`. Neither fetched nor generated at test
time.

| file | what it is |
|---|---|
| `erc/canary/canary.kicad_sch` | 2 unconnected symbols, a four-way junction, one global label; every coordinate at two decimals |
| `erc/canary/canary.kicad_pro` | turns `four_way_junction` and `single_global_label` on, `lib_symbol_issues` off |
| `erc/canary/canary.erc.json` | `kicad-cli` 10.0.5, `--format json --severity-all` |
| `erc/canary/canary.erc.txt` | the same run, `--format report --units mm --severity-all` |
| `erc/precision/precision.kicad_sch` | one symbol at `25.4321 mm` |
| `erc/precision/precision.kicad_pro` | KiCad's default severities, `lib_symbol_issues` off |
| `erc/precision/precision.erc.json` / `.erc.txt` | the same pair |

The schematics are hand-built and then made a fixed point of kicli's own
prettifier, so `round_trip::emit_reproduces_input_bytes` and
`prettify_reproduces_kicad_layout` both hold over them; provenance in `MANIFEST`
is `hand` for those and `kicad-cli` for the four reports, which are KiCad's
bytes. `lib_symbol_issues` is turned off in both projects on purpose: it fires
on the machine's symbol-library configuration rather than on the drawing, and a
fixture that varies with the reader's KiCad install is not a fixture.

The canary fixture carries **11 violations over 6 check types and 13 items**,
including `pin_not_connected`, `wire_dangling`, `unconnected_wire_endpoint`,
`isolated_pin_label`, `four_way_junction` and `single_global_label`; the
precision fixture adds `endpoint_off_grid`, which is the check `KI-GRID-001`
delegates to.

## Which checks ran here, and which wait on the merged run

`kicad-cli` on this machine is **10.0.5** (`/opt/homebrew/bin/kicad-cli` and
`/Applications/KiCad/KiCad.app/Contents/MacOS/kicad-cli`, both 10.0.5).

**Ran in the lane worktree, no KiCad needed** — these are the ones that carry
the task's claims, and they run on any machine:

- `cargo test -p kicli --test erc_canary` — 6 of 7 checks
- `cargo test -p kicli --test command_erc_gateway` — all 6
- `cargo test -p kicli --lib` — the `kicad::erc` and `lint::erc` unit checks

**Environment-gated, run here to MAKE the measurement, counting toward no
tick**: `erc_canary::the_committed_reports_are_current`, under
`KICLI_TEST_KICAD_CLI=1`. It runs `kicli::kicad::erc::rule_check` against the
real binary through kicli's own gateway and compares both fresh reports against
the committed ones, dates aside. **Result: identical, against `kicad-cli`
10.0.5.** Recorded output: `measured against kicad-cli 10.0.5`, then
`test the_committed_reports_are_current ... ok`.

**Waiting on the orchestrator's merged run**: that check, plus the full
`cargo xtask check` with the corpus. `CLAUDE.md`: an environment-gated green
inside a lane worktree counts toward no tick.

## What the merge hotspots owe

- **`crates/kicli/src/cli/`, `tests/command_surface.rs`, `AGENT.md`** — `kicli
  sch erc` is **not yet a command**. T2 built the gateway and the seam; nothing
  wires them to a verb, and the exit-code table's ERC rows are exercised through
  `ExitCode::for_tool_failure` rather than through a command. A later task owes
  the verb, its `command_surface` row, and its `AGENT.md` block — and
  `RULES.md`'s standing rule applies: the `AGENT.md` example must be regenerated
  from a real run.
- **`kicli.toml`'s `[erc] severity_map`** is already declared in
  `model/config.rs`'s `SECTIONS` and is parsed into no field. §14.3's
  documentation obligation is met in rustdoc; the *reading* of the map is owed
  by whoever ships the `sch erc` verb, and it must stay presentation-only.
- **`spec/SPEC.md` §14.2** could record the two measurements above — the text
  report's three-decimal rounding, and the description half of the scale bug.
  Not this lane's file.
- **`research/geometry.md` §3.5** owes the description half of the bug, as
  above.

## Falsification

Per `.claude/skills/falsification-control/SKILL.md`. **Every break was made in
the source (or the fixture), the whole `kicli` suite was run with
`--no-fail-fast`, and the good state was committed first** so `git checkout --`
could restore it. Restoration was verified by content hash after each row, not
by the command's exit code.

**Good state, by content hash** (a hash survives amend and rebase; a commit SHA
does not): `crates/kicli/src/kicad/erc.rs`
`cc13ec6aa080a082262bf3ab68bd118c9eda63fa`, `crates/kicli/src/lint/erc.rs`
`b03b9a2221f1d9a38c2a495bcb3be9c6a3d98b9a`, `crates/kicli/tests/erc_canary.rs`
`5add137fe9fa0a987a6b65f1ce34ac8f5c538b1f`,
`crates/kicli/tests/command_erc_gateway.rs`
`3779c8857b76ff92dd487332ba965ff55deb9911`,
`crates/kicli/tests/fixtures/erc/canary/canary.erc.json`
`86668eaab52c1ea435219e50e44615dc00f6a6f4`.

**Control, unbroken, `cargo test -p kicli --no-fail-fast`: 86 test targets, 656
checks, zero failures.** Without this the caught-by counts below would have no
denominator.

| # | What was broken | Caught by |
|---|---|---|
| 1 | **The correction removed.** `JSON_UNITS_PER_MM` 1e6 → 1e4, so the JSON's number is read at the schematic scale and no correction happens. | **11** — the `corrected` doctest; `kicad::erc::tests::the_correction_is_exactly_the_ratio_of_the_two_scales`; 6 of the canary's 7 (`the_rule_check_json_reports_coordinates_a_hundred_times_small`, `a_correctly_scaled_report_is_not_read_as_the_bug`, `the_committed_reports_are_still_the_ones_kicli_reads`, `the_text_report_loses_the_fourth_decimal`, `the_two_reports_enumerate_the_same_items_in_the_same_order`, `a_number_kicli_cannot_read_exactly_is_never_rounded`); and 3 of the gateway's. |
| 2 | **The correction applied twice.** `JSON_UNITS_PER_MM` 1e6 → 1e8. **This is §14.2's named failure**, the coordinate wrong by 10,000×. | **12** — every catcher of row 1, plus `kicad::erc::tests::a_number_that_cannot_be_read_exactly_is_refused`. |
| 3 | **`covers` folds "nobody ran it" into "KiCad has it covered."** The `self.ran &&` term removed. | **4** — `lint::erc::tests::a_check_nobody_ran_is_not_a_check_that_passed`; `command_erc_gateway::a_rule_check_nobody_ran_sends_a_rule_to_its_own_detection`; **and both doctests**, the module's worked example and `RuleCheck::covers`'s own. |
| 4 | **`delegate` records each violation twice** — the double count itself. | **1** — `a_delegating_rule_reports_once_however_the_project_is_configured`. |
| 5 | **The gateway swallows a runner error** and returns `RuleCheck::NOT_RUN` instead of failing, so an unusable binary stops being exit 6. | **1** — `an_absent_binary_is_exit_six_and_a_broken_run_is_not`. |
| 6 | **`described_by_the_text_report` made a no-op**, so the JSON's own scale-corrupted descriptions survive. | **1** — `the_committed_reports_are_still_the_ones_kicli_reads`. |
| 7 | **A committed report goes stale.** One coordinate of `canary.erc.json` changed from `0.254` to `0.255`. Run under `KICLI_TEST_KICAD_CLI=1`. | **5**, including **`the_committed_reports_are_current`** — which regenerated both reports from the real `kicad-cli` 10.0.5 and saw the difference. This is the row that shows the environment-gated check is alive. |

**Rows 4, 5 and 6 have exactly one catcher each, and that is worth recording
rather than glossing.** Each is a single point of detection: nothing else in
the suite would go red. That is why each of those three checks carries several
assertions rather than one equality — see in particular row 4's, whose two
sides are named above and whose message and object-list assertions are what
stop it being one number compared with itself.

**No row came back green.** The skill's "green is a finding" case did not arise,
so no instrument needed investigating.
