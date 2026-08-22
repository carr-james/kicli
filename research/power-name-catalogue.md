# The power-name catalogue — RATIFIED

**Status: RATIFIED. Provenance: James's ruling on BLOCKED 1, checkpoint 1
review** — *"T5's measured ground catalogue (KiCad 10.0.5's library, twelve
symbols, the Earth case ruling) is RATIFIED and supersedes both document lists.
`spec/SPEC.md` §11.4 and `research/style-rules.md` §4 are corrected to cite it
in the same commit. `KI-FLOW-001/002` stand on the ratified catalogue."*

**This file is the single canonical source for the power-direction name lists.**
`spec/SPEC.md` §11.4 and `research/style-rules.md` §4 cite it and state no list
of their own. If you are about to write a ground name into either of those
files, you are about to recreate the defect this file exists to close.

## Why the catalogue is a file rather than a paragraph

Two governing documents carried **different** lists, and `spec/SPEC.md` §11.4
declared `research/style-rules.md` §4 canonical *while stating a different list
in its own text*. There was no reading under which both were satisfied. That was
BLOCKED 1, raised by `lane-t5` at the moment of the claim and correctly not
resolved by precedence, per `CLAUDE.md`.

The ruling did not pick a winner between them. **It ratified a third thing that
was measured** — read out of KiCad 10.0.5's own library rather than recalled —
which makes both previous lists obsolete rather than making one of them lose.
One place, two citations. Whoever changes this file changes the rule.

## The measurement this catalogue rests on

**Measured by `lane-t5`, 2026-08-22, from KiCad 10.0.5's `power.kicad_sym`, and
falsified by rendering with `kicad-cli` rather than by re-reading the parse.**
Full evidence: `tasks/M5/phase1-t5-seed-catalogue-and-ground-names.md` §E2.

101 stock power symbols. Their intrinsic direction, taken from the pin angle
after transform:

| Pin angle | Body sits | Count | Symbols |
|---|---|---|---|
| `90` | above the connection point — points **up** | **89** | every positive supply, **and every negative one**, and `VSS`, `VSSA`, `VEE` |
| `270` | below the connection point — points **down** | **12** | `Earth`, `Earth_Clean`, `Earth_Protective`, `GND`, `GND1`, `GND2`, `GND3`, `GNDA`, `GNDD`, `GNDPWR`, `GNDREF`, `GNDS` |

**KiCad distinguishes a negative supply from a positive one by FILL, not by
direction.** Only the twelve `GND*`/`Earth*` symbols point down.

### The consequence, which is why this was the highest-value change and not a tidiness item

`KI-FLOW-001` defines **positive as the complement of the ground set**. So a
name missing from the ground list is **not** "a power symbol whose direction is
never checked" — it is a power symbol **checked backwards**, and it produces a
**false finding on a correctly drawn schematic**.

The list in force before this ratification recognised **three** of the twelve.
`GNDREF`, drawn pointing down exactly as KiCad ships it, would have been
reported as a mis-oriented positive supply. **Nine names, nine guaranteed false
findings**, which is the north star's expensive error rather than its cheap one:

> It must never reward a schematic that is impossible to read and understand.

A linter that invents findings on correct drawings is not reading the drawing.

## The catalogue

### Ground set — must point down

```
GND, GND1, GND2, GND3, GNDA, GNDD, GNDPWR, GNDREF, GNDS,
Earth, Earth_Clean, Earth_Protective,
AGND, DGND, VSS, VSSA, 0V
```

The twelve KiCad ships pointing down, plus five conventional names users author
by hand. **`AGND`, `DGND` and `0V` are kept although KiCad ships none of them**:
they cost nothing, they are widely hand-authored, and by the paragraph above a
missing name is worse than a spurious one.

### Matching is CASE-INSENSITIVE, and `Earth` is the spelling — RULED

The superseded §11.4 wrote `EARTH`; KiCad ships `Earth`. Under case-sensitive
matching that is a tenth false finding. **Neither previous document said
anything about case at all** — the behaviour would have fallen out of whichever
comparison an implementer happened to type.

**Ratified: matching is case-insensitive, and the catalogue's own spelling is
KiCad's.** Recorded as a decision rather than left to fall out of code, because
it was reached by nobody deciding it, which is how it stayed invisible.

### Negative set — a rule, not a list

**Value begins with `-`.** Plus **`VEE`** by name.

Measured over-catch: **zero, twice.** Of the 101 stock power symbols, 18 begin
with `-` and all 18 are genuine negative supplies. Of the 93 power-symbol
`Value`s across KiCad's 19 template schematics, none begins with `-`.

### Exempt by name — `PWR_FLAG`

It is an ERC annotation, not a supply, and its direction carries no meaning. It
ships drawn pointing up, so it does not fire today — **it would fire the moment
anyone rotated one, and the finding would be nonsense.**

### There is no positive set, and that is the point

**Struck from `spec/SPEC.md` §11.4 by this ratification.** `KI-FLOW-001` never
consults a positive list: anything not in the ground set and not beginning with
`-` is positive. A list that looks authoritative and is never read is how the
next reader adds `+3.3V` to it, believes a gap is closed, and ships a rule whose
behaviour did not change.

**The `+3V3` question is therefore moot for the rule and was load-bearing only
for the prose.** KiCad 10.0.5 ships `+3V3` *and* `+3.3V`, alongside `+3V0`,
`+3V8`, `+1V35`, `+7.5V`, `+3.3VA`, `+3.3VADC`, `+3.3VDAC`, `+3.3VP`; its own
templates use `+3V3` 17 times and `+3.3V` 4 times; and Sutherland names six
spellings of that one rail and recommends internal consistency rather than a
canonical spelling. **The spelling space is open and the list cannot be
completed** — which costs nothing, because both spellings classify correctly
today, and so would `3V3`, `+3v3`, and every spelling nobody has thought of.

## The precondition, which is the thing to defend

**The classifier applies to a power symbol's `Value`. It is never applied to net
names.** Ratified from P2.5.

Measured over-catch of the `-` prefix rule is zero — **but the falsifying case
exists in KiCad's own shipped content**: `API_Series-500.kicad_sch` carries the
net labels `-IN+4`, `-IN-2` and `-OUT`. Op-amp pin names, not supplies. They are
out of reach *only* because the rule reads a power symbol's `Value` and those
are `label` items.

> "Safe given precondition P" is a different claim from "safe", and it stays
> true only while P does.

**`KI-FLOW-001`'s author owes a falsification case built from those three
names**: a sheet carrying a net labelled `-OUT` must produce **no**
`KI-FLOW-001` finding. That check fails if anyone ever widens the classifier to
net names, which is the plausible future mistake — and the shorter rule with the
longer list is the trade that buys a bounded risk for an unbounded one.

## The conflict between the canon and KiCad's library is DELIBERATE

**Ratified from P2.6. Recorded here, where the rule lives, so that a later lane
or a dogfood run cannot meet this finding, read it as a false positive, and
"fix" it.**

Three published sources say negative supplies point down:

- **Greenberg:** *"Positive supplies point up, ground and negative supplies
  point down. **Always**."*
- **Lathrop:** *"Power connections should go up to positive voltages and down to
  negative voltages."*
- **Sutherland:** *"If you've got a bipolar supply, try to draw the negative rail
  at the bottom and positive rail at the top."*

**KiCad draws all of them pointing up.** So a user who places KiCad's stock
`-12V` symbol without rotating it produces a drawing that violates all three
conventions, and `KI-FLOW-001` will say so. **That finding is correct**, and it
will fire on the out-of-the-box behaviour of KiCad's own library: 20 of 101
stock symbols are in that class.

The north star's first half is the argument for keeping it — a `-12V` flag drawn
identically to a `+12V` flag is genuinely ambiguous to a reader, which is the
readability harm this milestone exists to catch.

## Project override

`kicli.toml`'s `flow.ground_names` overrides the ground set per project, per
`spec/SPEC.md` §15. The override replaces the list; it does not extend it. The
case-insensitivity and the `PWR_FLAG` exemption are not overridable, because
neither is a project's judgement to make.

## What supersedes what

| Superseded | Where it was | Now |
|---|---|---|
| `{GND, -12V, AGND, DGND, VSS, VEE, GNDA, GNDD, 0V, EARTH}` | `spec/SPEC.md` §11.4 | replaced by a citation to this file. `-12V` was redundant with its own leading-`-` rule; `EARTH` was a false finding waiting on case |
| `GND, GNDA, GNDD, AGND, DGND, VSS, 0V, EARTH` + `^-?V?SS$` | `research/style-rules.md` §4 | replaced by a citation to this file. The regex matched `VSS`, `SS`, `-VSS`, `-SS` and **not** `VSSA` — so `VSSA` was classified positive and passed **by coincidence**. Here it is named and passes on purpose. Coincidence is not a test |
| the positive list `{+12V, +5V, +3V3, …}` | `spec/SPEC.md` §11.4 | **struck.** Unreachable code in prose form |
