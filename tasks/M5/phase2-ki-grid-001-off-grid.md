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
