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
3408c1bae7a3117ee77afdfffb2464a4918df144b09ff61c24ebd5ab8c8ee691  crates/kicli/tests/lint_grid_covers_every_connectable_class.rs
```

Good state committed as `83365bf` before any deliberate break.

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
2. **The join is positional.** KiCad reports millimetres to four decimals
   (`@(51.435 mm, 46.99 mm)`) and this rule reports `Point` in internal units at
   the same coordinate; the oracle check in this lane's test file already
   compares the two by formatting the finding's `Iu` and matching KiCad's report
   text, and they agree exactly. No tolerance is needed on either side.
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
