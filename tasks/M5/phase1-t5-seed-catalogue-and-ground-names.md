# The seed catalogue and the ground-name list, researched (Phase 1, T5) ✅

**Provenance: James's ratification and advisor rulings, M5 plan review,
question 4.** Verbatim:

> Q1 (seed catalogue) and Q5 (ground-name list) become one bounded Phase 1
> research task whose deliverable is PROPOSED answers with sources, awaiting
> James's ratification at the checkpoint — the linter's notion of "good" is his
> to sign.

The questions are `research/style-rules.md` §8, Q1 and Q5. The same ruling
**closed Q2** (the Greenberg video is skipped; the published text sources govern)
and **confirmed Q3 and Q4** as already answered by `spec/SPEC.md` §11.2 and
§11.5. Those three need no work; cite them and move on.

## What makes this one task rather than two

Both questions ask the same thing in two places: **on whose authority does kicli
call a drawing good?** Q1 asks it of the rule catalogue, Q5 of one name list
inside it. Neither is a fact about KiCad that a measurement settles, which is
why the deliverable is a proposal and the ratification is James's.

**The milestone's north star is the sentence these answers serve** (`RULES.md`):
*"The tool must validate the important aspects of quality schematics. It must
never reward a schematic that is impossible to read and understand."* Where a
source is silent or two sources disagree, that sentence is the tie-breaker to
argue against — **in writing, in your proposal**, not silently.

## This task writes no source file

**It contends with nothing.** It runs alongside T1–T4. It touches no crate, no
fixture, no spec section. Its whole output is this entry's proposal section.

## Q1 — the seed catalogue

`research/style-rules.md` §8: *"`research/schematic-lint-rule-catalogue.md` is
not in the repo or its history. If you have it, send it and I will reconcile rule
IDs and any Tier 1/2 assignments you had already made, rather than imposing the
IDs invented here."*

**Half of this question only James can answer** — whether that document exists.
Do not try to find it; **confirm its absence and stop there**: `git log --all
--diff-filter=A -- '*catalogue*'` and a search of the working tree, both pasted.

**The half you CAN answer, and it is the useful half:** the rule IDs and the
Tier 1/2 assignments in `research/style-rules.md` §4 were **invented by this
project**. Establish what each rests on.

Deliver a table over the catalogue's rules — the six Tier 1 and the
twenty-two Tier 2 named in `spec/SPEC.md` §11.4 — with, for each:

- the **published source** that supports it, cited to a URL and a retrieval
  date, or **"no source — invented here"**, which is a legitimate and important
  answer;
- whether the source supports its **tier** (blocking vs scored) or only its
  existence. These are different claims and the catalogue conflates them.

**One source needs its own paragraph.** §9 records that Olin Lathrop's canonical
answer *"returned 403 to automated fetch, so the rule content here comes from
widely-reproduced summaries of it and should be spot-checked against the
original."* **That spot-check is part of this task.** If the original is still
unreachable, say so and say which rules therefore rest on summaries — that is a
provenance fact James should have before he signs.

**Then state the reconciliation cost**, concretely: if James does produce the
seed catalogue, how much work is it to reconcile? Which rules would be
re-numbered, and does any Phase 2 or 3 lane depend on the IDs being stable?
The answer decides whether Phase 2 can start before Q1 is fully closed.

## Q5 — the ground and negative-supply name list

`spec/SPEC.md` §11.4 carries the defaults:

- positive `{+12V, +5V, +3V3, …}`
- ground/negative `{GND, -12V, AGND, DGND, VSS, VEE, GNDA, GNDD, 0V, EARTH}`
- plus **"value starts with `-`" as negative**
- per-project override via `kicli.toml`

`KI-FLOW-001` and `KI-FLOW-002` stand on these lists. Deliver:

- **Whether the defaults cover standard Eurorack**, which §11.4 says they aim
  at, cited to a published Eurorack convention rather than to memory —
  Doepfer's A-100 bus specification is the obvious primary source.
- **What is missing, and what is wrong to include.** Both directions matter:
  a name absent from the list is a power symbol whose direction is never
  checked; a name wrongly in it is a **false finding on a correct drawing**, and
  the north star's second half makes the false finding the more expensive error
  in a tool whose findings an agent acts on.
- **The `+3V3` question, explicitly**: KiCad's own library uses several
  spellings for the same rail (`+3V3`, `+3.3V`). A list that catches one and
  misses the other is worse than a shorter list, because it looks complete.
  Say which spellings KiCad 10's own power symbol library actually ships —
  **read the library, do not recall it.**
- **What the `-` prefix rule catches and what it over-catches.** It is the only
  *rule* among the *lists*, so it is the only one that can be wrong in an
  unbounded way. A net legitimately named with a leading `-` is the falsifying
  case; say whether one exists in KiCad's demos.

## The form of the deliverable

Per `.claude/skills/task-entry-recording/SKILL.md`, **as PROPOSED entries in
this file** — one per question, each with:

- the proposed answer, stated so James can say yes or no to it without
  reconstructing the reasoning;
- the sources, cited to URL and retrieval date, with **"no source" said plainly
  where that is the truth**;
- the recommendation, and what it costs to leave the question open — because
  what James is deciding at the checkpoint is partly whether Phase 2 can start
  without this.

**Do not write the answers into `spec/SPEC.md` or `research/style-rules.md`.**
They are ratified first. A proposal that has already edited the spec is not a
proposal.

## Rules that bind this task specifically

- **NEVER read Konnect source (AGPL).** `CLAUDE.md`. Black-box only.
- **Verify KiCad facts against KiCad 10.0 documentation or behaviour, not
  training memory.** Formats and libraries changed between 7/8/9/10, and a
  power-symbol name list recalled from memory is exactly the failure this rule
  exists for. KiCad's own GPL source, fonts and demo files may be read freely
  (Constitution §9).
- **The Greenberg video is not consulted** — Q2, closed by James's standing
  round-6 ruling. Published text only. This binds `KI-DOC-001…004`'s row of the
  Q1 table too.
- **A source that could not be fetched is reported as unfetched.** Not
  paraphrased from memory of it, and not quietly replaced by a summary without
  saying so — that substitution already happened once in `research/style-rules.md`
  §9 and is the reason this task exists in the shape it does.

## Scope

**IN** — this file only. Reading: anything in the repository, KiCad 10's own
source and libraries, and published web sources.

**OUT** — `spec/SPEC.md`, `research/**`, every crate, every other entry.

## Completion check

**This task adds no code, so it has no `cargo` check.** It is complete when this
file carries two PROPOSED entries, one per question, each with its sources and
its recommendation, and the absence check for the seed catalogue pasted verbatim.

Per Constitution §11 every task names an executable completion check; **this one
names its check as the record's own completeness**, and says so rather than
inventing a `cargo` command that would pass whatever the file said. Recorded as
a deliberate exception, in the same form `opening-2` used for the same reason.

---

# Evidence and deliverable (lane t5)

Written as the work happened, per `CLAUDE.md`'s standing rule. Base verified as
the first action: `git log --oneline -1` → `d4c0eb8 freeze: lifted for exactly
the joined-net contract field (M5 opening-1)`, `git status --porcelain` → empty,
branch `lane-t5`. Matches the brief's named base; no fast-forward needed.

## E1 — the seed-catalogue absence check, pasted verbatim

**Provenance: measured, this lane, 2026-08-22.** Run in the lane worktree at
`d4c0eb8`. `--all` covers every branch in the repository, which is the twenty
lane branches plus `main` plus `origin/main`.

```
$ git log --all --oneline --diff-filter=A --name-only -- "*catalogue*" "*catalog*"
1112a87 rulings: the M5 plan is ratified, and all five of its questions are answered (M5)
tasks/M5/phase1-t5-seed-catalogue-and-ground-names.md

$ git log --all --oneline --diff-filter=A --name-only -- "*schematic-lint*"
(no output)

$ find . -path ./.git -prune -o -iname "*catalog*" -print
./tasks/M5/phase1-t5-seed-catalogue-and-ground-names.md

$ grep -rIl "schematic-lint-rule-catalogue" . --exclude-dir=.git
research/SUMMARY.md
research/RESEARCH-TASKS.md
research/style-rules.md
tasks/M5/phase1-t5-seed-catalogue-and-ground-names.md

$ git stash list
(no output)
```

**Reading.** The single history hit is **this task's own file**, whose name
contains the word "catalogue" — not the sought document. No path matching
`*catalog*` or `*schematic-lint*` has **ever** been added on any branch. The four
grep hits are all *references to* the missing document (`research/SUMMARY.md`
and `research/RESEARCH-TASKS.md` list it as a research input;
`research/style-rules.md` opens by recording its absence). The working tree
contains no such file and no stash holds one.

`research/style-rules.md`'s own claim — checked at `d2f3e93` — is **confirmed at
`d4c0eb8` and widened to all branches.** The document does not exist in this
repository. Whether it exists outside it is James's to answer, and per the task
entry this lane stops here rather than hunting for it.

## E2 — KiCad 10.0.5's power symbol library, read rather than recalled

**Provenance: measured, this lane, 2026-08-22.** Source of truth:
`/Applications/KiCad/KiCad.app/Contents/SharedSupport/symbols/power.kicad_sym`,
from the installed KiCad whose `kicad-cli version` reports **10.0.5**. The
library file's own header is `(version 20251024) (generator_version "10.0")`.
**101 symbols.**

### E2.1 Every name the library ships

```
-10V -12V -12VA -15V -24V -2V5 -36V -3V3 -48V -5V -5VA -6V -8V -9V -9VA
-BATT -VDC -VSW
+10V +12C +12L +12LF +12P +12V +12VA +15V +1V0 +1V1 +1V2 +1V35 +1V5 +1V8
+24V +28V +2V5 +2V8 +3.3V +3.3VA +3.3VADC +3.3VDAC +3.3VP +36V +3V0 +3V3
+3V8 +48V +4V +5C +5F +5P +5V +5VA +5VD +5VL +5VP +6V +7.5V +8V +9V +9VA
+BATT +VDC +VSW
AC Earth Earth_Clean Earth_Protective GND GND1 GND2 GND3 GNDA GNDD GNDPWR
GNDREF GNDS HT LINE NEUT PRI_HI PRI_LO PRI_MID PWR_FLAG VAA VAC VBUS VCC
VCCQ VCOM VD VDC VDD VDDA VDDF Vdrive VEE VMEM VPP VS VSS VSSA
```

### E2.2 The intrinsic direction of each, measured from the pin angle

Every one of the 101 symbols has **exactly one pin**. The pin angle takes
exactly **two** values across the whole library:

| Library pin angle | Body drawn | Count | Which symbols |
|---|---|---|---|
| `270` | **below** the connection point (points **down** on screen) | **12** | `Earth`, `Earth_Clean`, `Earth_Protective`, `GND`, `GND1`, `GND2`, `GND3`, `GNDA`, `GNDD`, `GNDPWR`, `GNDREF`, `GNDS` |
| `90` | **above** the connection point (points **up** on screen) | **89** | everything else — **including every `-…V` negative supply, and `VSS`, `VSSA`, `VEE`** |

**The headline measurement: KiCad 10.0.5 draws every negative supply symbol
pointing UP, identically to a positive supply.** Only the `GND*`/`Earth*` family
points down. The library distinguishes negative from positive by **fill**, not
by direction — `+5V` draws an outline arrow, `-12V` a filled one.

### E2.3 Falsification of the reading — rendered by KiCad itself

The direction claim above is derived from a coordinate convention, so it was
**checked against KiCad's own renderer rather than left as a derivation.** A
12-symbol schematic was built placing the stock symbols at rotation 0 and
plotted with `kicad-cli sch export pdf` (KiCad 10.0.5). The render shows, left
to right: `+5V ↑`, `+3V3 ↑`, `+3.3V ↑`, `-12V ↑`(filled), `VSS ↑`, `VEE ↑`,
`GND ↓`, `GNDA ↓`, `Earth ↓`, `GNDPWR ↓`, `VSSA ↑`, `-5V ↑`.

**This is the falsifying control for E2.2**: had the coordinate convention been
inverted, the render would have shown `GND` up and `+5V` down. It shows the
opposite, so the reading is confirmed by the tool rather than by argument. The
generated schematic is scratch (outside the repo) and is reproducible from the
recipe above; it is deliberately **not** added as a fixture, since this task's
scope is one file.

### E2.4 What KI-FLOW-001 would do to the stock library, counted

Applying `spec/SPEC.md` §11.4's ground/negative set and `style-rules.md` §4's
classifier (ground = set membership or `^-?V?SS$`; negative = leading `-`;
positive = everything else) to all 101 stock symbols **placed unrotated**:

| Class | Count | Outcome |
|---|---|---|
| Classified ground/negative, drawn **up** | **20** | **false finding** — 18 `-…V` symbols, plus `VSS`, `VEE` |
| Classified positive, drawn **down** | **9** | **false finding** — `Earth`, `Earth_Clean`, `Earth_Protective`, `GND1`, `GND2`, `GND3`, `GNDPWR`, `GNDREF`, `GNDS` |
| Classified ground, drawn down | 3 | correct — `GND`, `GNDA`, `GNDD` |
| Classified positive, drawn up | 69 | correct |

**29 of KiCad 10.0.5's 101 stock power symbols would produce a KI-FLOW-001
finding when placed straight out of the library with no rotation**, and only
**3 of the 12** symbols KiCad actually draws pointing down are recognised as
grounds.

*The four counts were derived by hand and then re-derived by script over the
library file — classifier fed the **union** of `spec/SPEC.md` §11.4's and
`style-rules.md` §4's ground sets, which is the most generous reading available
to the rule and therefore the lower bound on the error. Script output:
`20 / 9 / 3 / 69, TOTAL FALSE FINDINGS: 29`. Hand and script agree; had they
not, the script would have won.* The `Earth` row assumes case-sensitive matching, since §11.4 writes
`EARTH` and the library ships `Earth`; that is a decision nobody has made yet
and it is called out in P2 below.

## E3 — The Lathrop spot-check, which §9 said was owed

**Provenance: measured, this lane, 2026-08-22.**

### E3.1 The original is still unreachable, from two independent clients

```
$ WebFetch https://electrical.codidact.com/posts/278601
The server returned HTTP 403 Forbidden.

$ curl -A "Mozilla/5.0 (Macintosh; …) Chrome/120.0 Safari/537.36" -o /dev/null -w "%{http_code}"
https://electrical.codidact.com/posts/278601      403
https://electrical.codidact.com/questions/278600  403
https://electrical.codidact.com/api/posts/278601  403
https://electrical.codidact.com/                  403
```

The block is **host-wide**, not path-specific — the site root 403s too. So
`research/style-rules.md` §9's note is still accurate about the live URL, and
retrieval date 2026-08-22.

### E3.2 But the spot-check was completed anyway, against an archived original

The Internet Archive holds a snapshot, and **it is the original answer text, not
a summary**:

- <http://web.archive.org/web/20241209044413/https://electrical.codidact.com/posts/278601>
  — snapshot dated **2024-12-09**, retrieved **2026-08-22**, 16,871 characters
  of answer body extracted.

**This closes §9's open provenance item.** The rules below are now quoted from
Lathrop's own words rather than from third-party summaries of them. What follows
is what the original actually says — and it does **not** match the summary in
three places.

### E3.3 What the original supports, verbatim

| Claim in `style-rules.md` | Lathrop's own words | Verdict |
|---|---|---|
| positive supplies up, grounds down | *"Power connections should go up to positive voltages and down to negative voltages."* / *"Try to put positive power pins at the top, negative power pins (usually grounds) at the bottom"* | **supported, and it covers negative supplies explicitly** |
| inputs left, outputs right | *"inputs at left, and outputs at right"*; *"logical flow left to right"* | supported |
| no 4-way junctions | *"try to keep junctions to Ts, not 4-way crosses… The way to do that is to never have a 4-way junction."* | supported |
| minimise crossings | *"Spend some time with placement reducing wire crossings and the like."* | supported |
| consistent symbol orientation | **see E3.4 — not what he says** | **contradicted** |

### E3.4 Three corrections the spot-check forces

**(a) `KI-SYM-001` cites Lathrop for something he argues against.** The rule is
"inconsistent orientation for two-terminal parts", sourced to "Olin (consistent
orientation)". The original names rotated two-terminal parts as **normal
practice**: *"Some parts are commonly placed in different orientations,
horizontal and vertical in the case of resistors."* His demand is not that they
face the same way; it is that **the text be fixed after rotating**: *"If you
rotate a stock part, move the text around afterward so that it is easily
readable, clearly belongs to that part, and doesn't collide with other parts of
the drawing."* That sentence is `KI-FLD-001`'s rule, not `KI-SYM-001`'s.
`KI-SYM-001` is **invented here** and its citation should be struck.

**(b) `KI-FLOW-002` is missing an exemption its own source states.** *"One
notable exception to this is feedback signals. By their very nature, they feed
'back' from downstream to upstream, so they should be shown sending information
opposite of the main flow."* The catalogue's left-to-right rule records no
feedback exemption.

**(c) `KI-LBL-001` cites Lathrop for the opposite of his position.** The rule
penalises a long connection drawn as a wire instead of a label. Lathrop's
section is headed *"Direct connections, within reason"* and prefers the wire:
*"a messy rats nest of wires is worse than a few carefully chosen 'air wires'"*
— air wires are his **fallback**, not his preference. His labelling rule applies
to nets **already** broken into segments: *"If a net is broken up into visually
unconnected segments, then you absolutely have to let people know."*

### E3.5 Rules the original supports that the catalogue left uncited

The spot-check also **gains** sources. These §4 rules carry no `Source` line
today and can now have one:

- `KI-FLD-002` (missing designator) — *"Use component designators… make sure to
  add component designators."*
- `KI-FLD-001` (field placement) — *"Clean up text placement. Schematic programs
  generally plunk down part names and values based on a generic part
  definition… Fix it. That's part of the job of drawing a schematic."*
- `KI-LAY-001/002` (page utilisation, overcrowding) — the whole *"Design for
  regular size paper"* section: *"design your schematic so that individual
  sheets are nicely readable on a single normal page"*, *"Think of pages in
  schematics like paragraphs in a narrative."*
- `KI-CONN-001` (pin on a wire's interior) — *"Dots connect, crosses don't. Draw
  a dot at every junction… the only way to know whether they are connected is
  whether the little junction dot is present."* This is exactly the rule's
  rationale, and it was previously carried on in-repo evidence alone.

### E3.6 Where the original distinguishes tier, it does so explicitly

Two places, and they are the only direct evidence any source gives about
**tier** rather than existence:

- junction dots are **hard**: *"It's a rule. We don't care whether you think
  it's silly or not. That's how it's done."*
- four-way junctions are **soft**: *"This isn't as hard a rule, but stuff
  happens."* — which **confirms `KI-JCT-001` at Tier 2** and would make Tier 1
  wrong.

### E3.7 One canon rule the catalogue does not implement at all

*"Upper case symbol names — Use all caps for net names and pin names."*
Recorded, not proposed: adding a rule is a scoping decision, not this task's.

## E4 — Greenberg: the checklist IS published in citable text, and it is not the video

**Provenance: measured, this lane, 2026-08-22.**

### E4.1 The Hackaday write-up is a table of contents, not a checklist

Fetched directly (HTTP 200, 2026-08-22) and the **article body is 1,450
characters**. In full, what it attributes to the talk is: *"visual design best
practices; using schematic symbols rather than packages; nominating part values;
specific types of circuit gotchas; Design for Test; Design for Fail; electric
rule checks (ERC); manufacturer (MFR), part number (MPN), and datasheet
annotations for Bill of Materials (BOM); and things to check at the end of a
design iteration, including updating the date and version number."

That is the **entire** rule content of the source `research/style-rules.md` §1
credits with nine specific rules. It does **not** mention four-way junctions,
explicit-over-label connections, mono-PDF legibility, DNP minimisation, purpose
notes, voltage ranges, or "one page, one idea".

### E4.2 The article links to the checklist itself — Q2's condition is met

`research/style-rules.md` §8 Q2 asked: *"If his checklist is published in a
citable form, it should be the primary source for the KI-DOC-* family."*
**It is.** The Hackaday article links the word "checklist" to a public document:

- Andrew Greenberg, **"Checklist for Schematics v2026-02-15"**,
  <https://docs.google.com/document/d/1gCPILcrdGZJjRzIDSL-b3ezVReeK5S-7raeub1RohyE/>
  — retrieved **2026-08-22** as plain text (HTTP 200, 8,339 bytes), 55 checklist
  items in 8 groups.

**This is a published text source, not the video**, so consulting it is inside
James's standing round-6 ruling rather than an exception to it — the ruling's
words are *"the video is skipped, the published text sources govern"*, and Q2
named this document as the one to prefer if it existed.

### E4.3 What the checklist actually says, against what §4 attributes to him

| §4's attribution | The published checklist | Verdict |
|---|---|---|
| power direction (`KI-FLOW-001`) | *"Positive supplies point up, ground and negative supplies point down. **Always**."* | **supported, emphatically** |
| MFR/MPN (`KI-DOC-001`) | *"Add 'MFR' (Manufacturer) and 'MPN' (Manufacturer's Part Number) to all components as attributes."* | supported |
| datasheet (`KI-DOC-002`) | *"**Bonus points** for adding a distributor, distributor part number, description, and datasheet link."* | **supported only as optional** — which justifies the low weight and rules out requiring it |
| version/date (`KI-DOC-003`) | *"Update your schematic version and/or date."* | supports `rev`+`date`; **`title` has no source** |
| purpose notes, voltage ranges (`KI-DOC-004`) | *"Functional blocks have text that describes what they do and their requirements (e.g., Vbatt to 3.3 V @ 1 A switching power supply)"*; *"It's clear where your power is coming from and what the power requirements are (V/I)"* | **supported more strongly than §4's `corroboration: weak` assumed** |
| left-to-right flow (`KI-FLOW-002`) | *"All symbols are schematic symbols, not packages (inputs on left, outputs on right…)"*; *"Data flow (inputs, outputs, requirements) are clear and labeled."* | supported |
| named significant nets (`KI-LBL-002`) | *"All important nets are descriptively named."* | supported |
| crossings (`KI-XING-001`) | *"Route wires at a consistent distance from each other and avoid crossing net wires as possible."* | supported |
| avoid 4-way junctions (`KI-JCT-001`) | **absent** | **not supported — strike the Greenberg citation** |
| explicit connections over label-only (`KI-LBL-001`) | **absent, and inverted**: *"Net 'stubs' … use an 'off-sheet' or 'Global' type of label with the correct In/Out/Bidirectional flag shape and cross reference info"* | **not supported — the checklist mandates labels** |
| mono-PDF legibility (`KI-TXT-002`) | **absent** | **not supported — strike the citation** |
| minimise DNP / just-in-case parts (`KI-DNP-001`) | **absent, and inverted**: *"Add required cable or required accessory part numbers as text, or add as Do Not Place (DNP) components if you want them to show up in the BOM."* | **not supported — the cited source recommends the opposite** |
| "one page, one idea" (`KI-LAY-002`) | **absent** | **not supported — strike the citation** |

**Five of the thirteen attributions to Greenberg are unsupported by his own
published checklist, and two of those five are inverted** — the checklist
recommends the thing the rule penalises. This is the concrete cost of §9's
substitution of summaries for a source, and it is why the spot-check was owed.

### E4.4 A caution on the document's comments

The checklist carries reader comments (`[a]`–`[j]`) quoting third-party
material. **Those comments are not Greenberg's rules** and are not cited as his
anywhere below. One of them, however, surfaced a source worth having on its own
terms — E5.

## E5 — A fourth published source, and it covers what "invented here" covered

**Provenance: measured, this lane, 2026-08-22.**

- Graham Sutherland, **"Creating high quality electronics schematics"**,
  <https://blog.poly.nomial.co.uk/2025-08-10-creating-high-quality-electronics-schematics.html>
  — dated **2025-08-10**, retrieved **2026-08-22** (HTTP 200). Fourteen numbered
  rules with worked bad/good examples.

It matters because it names, in a citable published source, several rules the
catalogue currently carries as derived or uncited:

| Rule | Sutherland's words |
|---|---|
| `KI-GRID-001` | §1 *"Stick to the grid… Put your parts on the grid, draw your wires on the grid. The grid should ideally be set to the same size as the pin pitch on your symbols."* |
| `KI-RTE-001` | §2 *"Avoid small and unnecessary turns… no unnecessary offsets, zigzags, or loops."* |
| `KI-RTE-002` | §6 *"Avoid spaghetti. Don't route wires in a big tangled mess… or wires looping around through your schematic."* |
| `KI-LBL-001` | §6 *"Use net label to avoid needing to route wires to lots of different locations across your schematic."* — the **distance** argument the rule actually makes |
| `KI-LBL-002` | §4 *"an autogenerated net name like `NetU4_7`"* |
| `KI-TXT-001/002` | §10 *"Give yourself space… **Don't run wires through text**"*, and the worked example *"R4's reference designator overlaps the `FB_OUT` net label"* |
| `KI-LAY-001` | §10 *"If you find yourself running out of space, consider changing your default sheet size to something larger."* |
| `KI-LAY-002` | *"Split bigger schematics up into labelled blocks… You can also move to a multi-sheet schematic."* |
| `KI-LAY-003` | §2 *"Align things nicely."*; §11 *"Be logical about where you place things."* |
| `KI-FLOW-001` | §5 *"Your power ports should face upwards. Your grounds should point downwards."* and *"If you've got a bipolar supply, try to draw the **negative rail at the bottom** and positive rail at the top."* |

**And it answers the `+3V3` question head-on** (§14, *"Decide on a net naming
convention and stick to it"*), naming six spellings of one rail:

> `3V3` `+3V3` `3v3` `+3v3` `3.3V` `+3.3V`

with the recommendation being *consistency*, not a canonical spelling. **No
published source names a canonical spelling for the 3.3 V rail.** That is the
finding, and it is the reason a list of positive-rail spellings cannot be
completed by adding one more entry.

## E6 — What KiCad's own shipped schematics do

**Provenance: measured, this lane, 2026-08-22.** The macOS KiCad 10.0.5 package
ships **no `demos/` directory** (`find /Applications/KiCad -type d -name demos`
returns nothing). The nearest KiCad-authored corpus in the install is
`SharedSupport/template` — **19 schematics, 20 projects**, and that is what the
"is there a legitimate leading-`-` name in KiCad's own drawings" question was
answered against. Stated plainly because it is a substitution of corpus, and
substituting a source without saying so is the failure this whole task exists to
correct.

### E6.1 Both `3V3` spellings ship, in KiCad's own schematics

**93 power-symbol placements across the 19 template schematics.** Counted by
`Value`, which is what `KI-FLOW-001` classifies on and what KiCad uses as the
net name:

```
GND 34   +3V3 17   +5V 14   GNDD 8   VCC 6   +3.3V 4   GNDA 2
VBUS 2   VDDA 2    +1V8 1    +BATT 1  VDC 1   VDD 1
```

**`+3V3` (17) and `+3.3V` (4) are both live in KiCad's own work**, split across
twelve files: `+3.3V` in `Arduino_Mega`, `Arduino_Uno`, `STM_Nucleo64_Morpho` by
`lib_id`, `+3V3` elsewhere. **A list carrying one spelling and not the other
mis-handles KiCad's own templates either way round.**

### E6.2 The `Value` and the `lib_id` disagree, 14 times out of 93

Every one of the fourteen is `lib_id "power:+3.3V"` placed with
`Value "+3V3"` — in `Arduino_Micro`, `Arduino_Nano`, `BeagleBone-Black-Cape`,
`RaspberryPi-HAT`, `RaspberryPi-uHAT`, `TI-LaunchPad-BoosterPack-20pin`,
`TI-LaunchPad-BoosterPack-40pin` and `stm32f100-discovery-shield`. Confirmed by
rendering `stm32f100-discovery-shield.kicad_sch` with `kicad-cli sch export pdf`
and reading the plotted label: the symbol drawn from `power:+3.3V` prints
**`+3V3`**.

**Consequence for whoever implements `KI-FLOW-001`: classify on `Value`, never
on `lib_id`.** Reading `lib_id` would give the wrong name 15 % of the time on
KiCad's own templates. `style-rules.md` §4 already says `Value`; this is the
measurement that makes it a load-bearing sentence rather than an incidental one.

### E6.3 KiCad's own templates rotate power symbols

8 of the 93 placements are not at rotation 0: four `GND` at 270 and one `+3.3V`
at 90 in `stm32f100-discovery-shield`, one `+5V` at 90 there, and one `+3.3V` at
270 in each of `RaspberryPi-HAT` and `RaspberryPi-uHAT`. The render of
`stm32f100-discovery-shield` shows those symbols pointing **sideways (left)**,
not up or down. **`KI-FLOW-001` would emit 8 findings on KiCad's own shipped
templates.** Those are arguably true findings by all three published sources —
recorded so that the number is known before it surprises someone, not as an
objection to the rule.

### E6.4 The leading-`-` falsifying case exists, and is out of the rule's reach

Scanning every `label`, `global_label`, `hierarchical_label` and `text` in the
19 templates: **366 distinct strings, of which exactly 3 begin with `-`** — all
in `API_Series-500.kicad_sch`, all op-amp signal names:

```
-IN+4     -IN-2     -OUT
```

**So the answer to "is there a net legitimately named with a leading `-` in
KiCad's own drawings" is YES, three of them.** They do not fire today for one
reason only: `KI-FLOW-001` classifies the **`Value` of a power symbol**, and
these are net labels. **Zero of the 93 power-symbol `Value`s in the templates
begins with `-`, and zero of the 18 leading-`-` names in the stock library is
anything other than a genuine negative supply — so the measured over-catch of
the `-` rule is zero, conditional entirely on that precondition holding.**
That makes the precondition the thing to test, not the name list.

---

# P1 — PROPOSED: Q1, the seed catalogue

**PROPOSED (lane t5, 2026-08-22).** This is a proposal, not a decision.
`spec/SPEC.md` and `research/style-rules.md` are **untouched** by this lane, per
the task entry: *"A proposal that has already edited the spec is not a
proposal."*

## The proposed answer, in three sentences

1. **`research/schematic-lint-rule-catalogue.md` does not exist in this
   repository, on any branch, at any point in its history** (E1). Whether it
   exists outside the repository is the half only James can answer, and this
   lane stopped there as instructed.
2. **The rule IDs and the Tier 1/2 assignments are this project's own
   invention**, and the table below records, per rule, what each actually rests
   on — including **four rules that rest on nothing published**, and **six whose
   cited source is wrong, two of them inverted**.
3. **Recommendation: do not hold Phase 2 for this.** Close Q1 either way at the
   checkpoint; if the seed catalogue exists, hand it over **before Phase 3, not
   before Phase 2**, for the reason in P1.4.

## P1.1 The provenance table — all 28 rules

Legend for **Tier**: does the source support the rule's *tier* (blocking vs
scored), or only its *existence*? The catalogue conflates these, and separating
them is most of this table's value. Retrieval date for every URL: **2026-08-22**.

Sources, cited once:
- **[L]** Olin Lathrop, Codidact answer, <https://electrical.codidact.com/posts/278601> — **live URL 403s host-wide**; read from the Internet Archive snapshot of **2024-12-09**, <http://web.archive.org/web/20241209044413/https://electrical.codidact.com/posts/278601> (E3).
- **[G]** Andrew Greenberg, "Checklist for Schematics v2026-02-15", <https://docs.google.com/document/d/1gCPILcrdGZJjRzIDSL-b3ezVReeK5S-7raeub1RohyE/> (E4).
- **[S]** Graham Sutherland, "Creating high quality electronics schematics", 2025-08-10, <https://blog.poly.nomial.co.uk/2025-08-10-creating-high-quality-electronics-schematics.html> (E5).
- **[E]** KiCad 10.0.5 ERC, `eeschema/erc/erc_item.cpp`, `erc_settings.cpp:95-124`.
- **[K]** KiCad 10.0.5 itself — libraries, templates, renderer (E2, E6).
- **[R]** in-repo: Constitution, SPEC, `research/notes/**`. Not a published source.

### Tier 1 — the six blocking rules

| Rule | Source for its **existence** | Source for its **tier** |
|---|---|---|
| `KI-GRID-001` off-grid | **[S]** §1 *"Stick to the grid… draw your wires on the grid"*; **[E]** `endpoint_off_grid` (wire endpoints only) | **none.** [E]'s own default severity is **WARNING**, not error. Tier 1 is **[R]** Constitution §7's correctness argument, and is kicli's own call |
| `KI-OVL-001` bodies overlap | **weak.** No source states it. Nearest is **[S]** §10 *"Don't try to squash things into the smallest possible space"* | none |
| `KI-WIRE-001` wire through body | **partial.** **[S]** §8 *"Route out of pins, not across pins"* — about pins, not bodies | none |
| `KI-TXT-001` overlapping text | **[S]** §10 *"Don't run wires through text"* + the worked example *"R4's reference designator overlaps the `FB_OUT` net label"*; **[L]** *"doesn't collide with other parts of the drawing"* | **none.** Tier 1 rests on **[R]** SPEC D3 |
| `KI-CONN-001` pin on wire interior | **[L]** *"Dots connect, crosses don't… the only way to know whether they are connected is whether the little junction dot is present"*; **[R]** `notes/pin-on-wire-interior.md` | **SUPPORTED.** **[L]** marks it a hard rule: *"It's a rule. We don't care whether you think it's silly or not. That's how it's done."* |
| `KI-HIER-001` hier label mismatch | **[E]** `hier_label_mismatch` | **SUPPORTED** — [E]'s default severity is ERROR |

### Tier 2 — the twenty-two scored rules

| Rule | Source for its **existence** | Source for its **tier** |
|---|---|---|
| `KI-FLOW-001` power direction | **[G]** *"Positive supplies point up, ground and negative supplies point down. **Always**."*; **[L]** *"Power connections should go up to positive voltages and down to negative voltages"*; **[S]** §5 | **sources disagree.** [G]'s "Always" argues Tier 1; [L] and [S] phrase it as guidance. Tier 2 defensible, **not** source-mandated |
| `KI-FLOW-002` L-to-R flow | **[L]** *"logical flow left to right"*, *"inputs at left, and outputs at right"*; **[G]**; **[S]** §5 | none. **And [L] states an exemption the rule lacks** — see P1.2(b) |
| `KI-XING-001` crossings | **[L]** *"reducing wire crossings"*; **[G]** *"avoid crossing net wires as possible"*; **[S]** §6 | **SUPPORTED as soft** — **[L]** *"It is impossible to come up with a universal rule here"*. Tier 2 is right |
| `KI-JCT-001` four-way junction | **[L]** *"never have a 4-way junction"*; **[E]** (default `IGNORE`) | **SUPPORTED as soft** — **[L]** *"This isn't as hard a rule, but stuff happens."* Tier 2 is right, Tier 1 would be wrong. **§4's "Greenberg" citation is unsupported** |
| `KI-RTE-001` doglegs | **[S]** §2 *"Avoid small and unnecessary turns… no unnecessary offsets, zigzags, or loops"* — §4 says "derived"; **it now has a source** | none |
| `KI-RTE-002` length ratio | **[S]** §6 *"Avoid spaghetti… wires looping around through your schematic"* — §4 says "derived"; **it now has a source** | none. The `1.6` constant is invented |
| `KI-LBL-001` long wire, no label | **[S]** §6 *"Use net label to avoid needing to route wires to lots of different locations across your schematic"* | none. **Both cited sources fail** — see P1.2(c) |
| `KI-LBL-002` auto-generated name | **[L]** *"they default to some gobbledygook unless you explicitly set them"*; **[G]** *"All important nets are descriptively named"*; **[S]** §4 | none |
| `KI-LBL-003` lonely global label | **[E]** `single_global_label` | **argued against.** [E]'s default is `IGNORE`; scoring it is kicli's own call (§2.2). **And [G] mandates Global labels on stubs** — a near-miss, see P1.2(d) |
| `KI-TXT-002` text over wire | **[S]** §10 *"Don't run wires through text"*; **[L]** *"doesn't collide"* | none. **§4's "mono-PDF legibility item from Greenberg" does not exist** |
| `KI-TXT-003` inconsistent text sizes | **NO SOURCE — invented here.** Not in [L], [G], [S], [E] or [K] | none |
| `KI-FLD-001` field placement | **[L]**, the whole *"Clean up text placement"* section — **§4 cites nothing today** | **[L]**'s language is Tier 1 in force: *"There is no excuse."* In tension with Tier 2 |
| `KI-FLD-002` hidden designator/value | **[L]** *"Use component designators… make sure to add component designators"* — **§4 cites nothing today** | none |
| `KI-DOC-001` MFR/MPN | **[G]** *"Add 'MFR'… and 'MPN'… to **all components** as attributes"* | [G] is a checklist, so pass/fail in form. **Grouping by part class is kicli's own Constitution §6 mitigation, not [G]'s** — say so where it is written down |
| `KI-DOC-002` datasheet | **[G]**, but as **optional**: *"**Bonus points** for adding a distributor… and datasheet link"* | **supports the low weight and rules out requiring it** |
| `KI-DOC-003` title block | **[G]** *"Update your schematic version and/or date"* → `rev`, `date`. **`title` has NO source** | none |
| `KI-DOC-004` no sheet notes | **[G]** ×3: *"Functional blocks have text that describes what they do and their requirements"*, *"It's clear where your power is coming from and what the power requirements are (V/I)"*, *"All connectors have text that describes where they go"*; **[S]** §13 | none. **Stronger than §4's `corroboration: weak`, which was assigned when only the Hackaday summary was in hand.** The weak claim that survives is the *mechanical proxy* — presence of text ≠ presence of explanation |
| `KI-LAY-001` page utilisation | **[L]** *"Design for regular size paper"*; **[S]** §10 — **§4 cites nothing today** | none. The `0.35`/`0.92` constants are invented |
| `KI-LAY-002` overcrowding | **[L]** *"Think of pages in schematics like paragraphs in a narrative"*; **[S]** *"Split bigger schematics up into labelled blocks"* | none. **§4's "Greenberg's 'one page, one idea' framing" is not in [G]**. The `60` constant is invented, as §4 already says |
| `KI-LAY-003` symbol alignment | **[S]** §2 *"Align things nicely"*, §11 *"Be logical about where you place things"* — **§4 cites nothing today** | none |
| `KI-DNP-001` DNP clutter | **NO SOURCE — invented here, and the cited source says the opposite**: **[G]** *"…or add as Do Not Place (DNP) components if you want them to show up in the BOM"* | none |
| `KI-SYM-001` two-terminal orientation | **NO SOURCE — invented here, and [L] contradicts it**: *"Some parts are commonly placed in different orientations, horizontal and vertical in the case of resistors."* | none |

### The counts

- **28 rules. 4 have no published source at all**: `KI-TXT-003`, `KI-DNP-001`,
  `KI-SYM-001`, and `KI-OVL-001` (nearest-neighbour only).
- **6 carry a citation that the source does not support**: `KI-JCT-001`
  (Greenberg half), `KI-LBL-001` (both), `KI-TXT-002` (Greenberg),
  `KI-LAY-002` (Greenberg), `KI-DNP-001` (**inverted**), `KI-SYM-001`
  (**contradicted**).
- **10 gain a published source they did not have**: `KI-GRID-001`,
  `KI-TXT-001`, `KI-CONN-001`, `KI-RTE-001`, `KI-RTE-002`, `KI-FLD-001`,
  `KI-FLD-002`, `KI-LAY-001`, `KI-LAY-002`, `KI-LAY-003`.
- **24 of 28 have no source support for their TIER.** Four do: `KI-HIER-001`
  and `KI-CONN-001` for Tier 1, `KI-JCT-001` and `KI-XING-001` for Tier 2.
  **Tier assignment is this project's judgement almost everywhere**, which is
  precisely the thing James said is his to sign.

## P1.2 Four corrections the sources force

- **(a) `KI-SYM-001` should lose its citation.** [L] names rotated resistors as
  normal practice; what he demands is that the *text* be fixed after rotating,
  which is `KI-FLD-001`'s rule. Detail and quotes in E3.4(a).
- **(b) `KI-FLOW-002` is missing an exemption its own source states.** [L]:
  *"One notable exception to this is feedback signals… they should be shown
  sending information opposite of the main flow."* A feedback path is exactly
  the shape the rule's `x̄_in − x̄_out` term punishes.
- **(c) `KI-LBL-001`'s two citations both fail**, and its real source is [S] §6.
  Detail in E3.4(c) and E4.3.
- **(d) `KI-LBL-003` has a near-miss with [G].** [G] *requires* a Global label
  on a single-pin stub, with cross-reference. The rule is safe **only because it
  tests "once overall", not "on one sheet"** — an implementation that counted
  sheets would fire on exactly the pattern [G] prescribes. Worth a falsification
  case when Phase 3 writes it.

**None of these are edits this lane makes.** They are edits to `research/**` and
`spec/SPEC.md`, both OUT of scope, and all of them are ratification material.

## P1.3 The Lathrop provenance fact James should have before signing

`research/style-rules.md` §9 said the canonical answer *"returned 403 to
automated fetch, so the rule content here comes from widely-reproduced summaries
of it and should be spot-checked against the original."*

**The live URL still 403s — host-wide, from two independent clients (E3.1). The
spot-check was nevertheless completed, against the Internet Archive's
2024-12-09 snapshot of the original answer (E3.2).** So:

- **No rule below now rests on a summary of Lathrop.** Every [L] citation in
  P1.1 is quoted from the archived original.
- **The summaries were right about four claims and wrong about one** — the
  "consistent symbol orientation" claim, which is the one `KI-SYM-001` was built
  on (E3.3, E3.4a).
- **The Greenberg summaries were the worse of the two**, and nobody had asked
  about them: five of thirteen attributions unsupported, two inverted (E4.3).

**Recommendation: §9's caution is discharged and should be replaced by the
archive citation** — and the same paragraph should record that the Greenberg
rows are now backed by the checklist document, not by the Hackaday write-up.

## P1.4 The reconciliation cost, concretely

**If James produces the seed catalogue, what does it cost?**

**Renaming is cheap and stays cheap.** The IDs are opaque labels; no detection
logic branches on one.

*Corrected against measurement, at the moment of the claim: this paragraph first
asserted "no rule exists in code at all". That was wrong, and the grep says so.*
As of `d4c0eb8` no rule is **implemented** — `crates/kicli/src/lint.rs` is a
6-line doc comment and there is no `lint/` directory — but the ID strings
already appear in code in two places:

```
$ grep -rIn 'KI-[A-Z]' crates/
crates/kicli/src/model/config.rs:234:  /// The keys a per-rule table holds, as in `[rules."KI-XING-001"]`.
crates/kicli/src/model/config.rs:393:  // [rules."KI-XING-001"] holds one table per rule.
crates/kicli/src/model/config.rs:668:  "[rules.\"KI-XING-001\"]\nenabled = true\nweight = 1.0\nfree_allowance = 2\n",
crates/kicli/src/model/config.rs:677:  Config::parse("[rules.\"KI-XING-001\"]\nwieght = 1.0\n")
crates/kicli/src/model/invariant.rs:30:   /// … under the blocking rule `KI-GRID-001`, because a
crates/kicli/src/model/invariant.rs:217:  /// A symbol's resolved pin positions are the lint's, under `KI-GRID-001`.
```

Four are doc comments and two are **live test strings** in `config.rs`. So a
re-numbering today is two documents, a plan table, and **two source files** —
still trivial, but not zero, and the difference is exactly the kind of thing a
sweep misses.

After Phase 2 and 3 it also touches, per the seam `phase1-t1` is deciding
(*"adding a rule touches exactly one new file"*): one file name per rule under
`lint/rules/`, one ID constant inside each, the `[rules."KI-…"]` config keys, and
every test snapshot carrying the ID string. **28 mechanical renames** — bounded,
but only if the ID has a single source of truth per rule.

- **PROPOSED, addressed to the orchestrator rather than to James:** ask
  `phase1-t1` to make each rule's ID **one constant**, with the file name derived
  from or checked against it. That is the difference between a re-numbering that
  is a rename and one that is a sweep, and it costs T1 nothing to decide now.
  This lane did **not** contact T1 or touch its entry.

**Re-numbering is not the risk. Re-tiering is.** A seed catalogue that moved a
rule between Tier 1 and Tier 2 would move it **between phases and between
lanes** — Phase 2 builds Tier 1, Phase 3 Tier 2, and the Phase 3 lane cut groups
by shared definition. That is a re-plan, not a rename. And P1.1 shows the tier
assignments are the *least* evidenced thing in the catalogue: 24 of 28 have no
source support for their tier at all.

**Does any Phase 2 or 3 lane depend on the IDs being stable? No — and the
asymmetry is what decides the recommendation:**

- **Phase 2 does not depend on Q1.** Its six rules are the ones a seed catalogue
  could least disturb: five are kicli-native geometry and connectivity checks
  whose existence rests on the Constitution and in-repo measurement, and the
  sixth is delegated to ERC entirely. §11.1 makes them kicli's reason to exist,
  so no external catalogue removes them. **Phase 2 may start before Q1 closes.**
- **Phase 3 does depend on it**, because Tier 2 is where every invented rule and
  every invented weight sits — `KI-TXT-003`, `KI-DNP-001` and `KI-SYM-001` have
  no source, and all 22 weights are priors.

**Cost of leaving Q1 open:** near zero through Phase 2; real from Phase 3
onward, where it is the risk of building three rules that a catalogue would
delete and of numbering 22 that it would renumber. **Cost of closing it: one
sentence from James** — "I have it" or "I don't". If he does not, Q1 closes
permanently, nothing reconciles, and P1.1 becomes the catalogue's provenance
record on its own.

---

# P2 — PROPOSED: Q5, the ground and negative-supply name list

**PROPOSED (lane t5, 2026-08-22).** A proposal, not a decision. Neither
`spec/SPEC.md` nor `research/style-rules.md` was edited by this lane.

## The proposed answer, in four sentences

1. **The task entry's own framing of the risk is wrong, and the algorithm says
   so**: because "positive" is defined as *the complement of the ground set*, a
   name missing from the ground list is **not** "a power symbol whose direction
   is never checked" — it is a power symbol **checked backwards**, which
   produces a **guaranteed false finding on a correctly drawn schematic**.
2. **The current list misses 9 of the 12 names KiCad 10.0.5 actually draws
   pointing down**, so it is wrong in exactly that way, nine times over (E2).
3. **The `+3V3` question is moot for `KI-FLOW-001` and load-bearing for
   `spec/SPEC.md`**: the positive list is never consulted by the classifier, yet
   §11.4 presents it as though it were.
4. **The `-` prefix rule's measured over-catch is zero** — but only because the
   rule reads a power symbol's `Value`, and KiCad's own templates contain three
   legitimate leading-`-` **net labels** that would fire the moment anything
   applied the classifier to net names (E6.4).

## P2.1 The correction that reorders everything else

`style-rules.md` §4's `KI-FLOW-001` defines **two** sets — ground and negative —
and no positive set. Positive is the complement:

> Classify `s` as *positive* or *ground* from its `Value` (ground set: …;
> negative set: value starts with `-`). Finding if a **positive** symbol's pin
> does not point up **or** a ground symbol's pin does not point down.

**Task text yields to measured reality.** The entry states the stakes as
*"a name absent from the list is a power symbol whose direction is never
checked"*. Under the algorithm as written, an absent name falls into the
complement, is classified **positive**, and is then required to point **up** —
so `GNDREF`, drawn correctly pointing down as KiCad ships it, produces a
finding. The failure mode is not a silent gap. **It is a false finding, on a
correct drawing, and it is the expensive error the north star's second half
warns about.** Completing the ground list is therefore the highest-value change
here, not a tidiness item.

## P2.2 Recommended lists

**Ground set — must point down.** The twelve names KiCad 10.0.5 actually draws
downward (E2.2), plus the conventional names users author themselves. Additions
to §11.4 in **bold**:

```
GND, GND1, GND2, GND3, GNDA, GNDD, GNDPWR, GNDREF, GNDS,
Earth, Earth_Clean, Earth_Protective,
AGND, DGND, VSS, VSSA, 0V
```

- **`GND1`, `GND2`, `GND3`, `GNDPWR`, `GNDREF`, `GNDS`, `Earth_Clean`,
  `Earth_Protective`, `VSSA` are new**, and each is a symbol KiCad ships drawn
  pointing down that today would be flagged as a mis-oriented positive supply.
- **`EARTH` should become `Earth`, and matching should be case-insensitive.**
  §11.4 writes `EARTH`; KiCad ships `Earth`. Under case-sensitive matching that
  is a tenth false finding. **This is a decision nobody has made** — §4 and
  §11.4 are both silent on case — and it should be made explicitly rather than
  fall out of whichever comparison an implementer types.
- **`AGND`, `DGND`, `0V` are kept although KiCad ships none of them**: they cost
  nothing, they are widely authored by hand, and by P2.1 a missing name is worse
  than a spurious one.

**Negative set — leading `-`, kept as a rule.** Plus `VEE`. Measured over-catch
zero across the 101 stock symbols and the 93 template placements (E6.4).

**Exemption — `PWR_FLAG`.** It is an ERC annotation, not a supply; its direction
carries no meaning. It is drawn pointing up, so it does not fire today, but it
would fire the moment anyone rotated one, and the finding would be nonsense.
Not in either list today, and it should be exempted by name.

**Positive set — delete it from §11.4, or mark it non-normative.** It is
unreachable code in prose form. See P2.3.

## P2.3 The `+3V3` question, answered from the library rather than from memory

**What KiCad 10.0.5 ships (E2.1):** *both*, and more than both —
`+3V3` **and** `+3.3V`, alongside `+3V0`, `+3V8`, `+1V35`, `+7.5V`,
`+3.3VA`, `+3.3VADC`, `+3.3VDAC`, `+3.3VP`. **What KiCad's own templates use
(E6.1):** both, `+3V3` 17 times and `+3.3V` 4 times across twelve files. **What
the published sources say (E5):** Sutherland names *six* spellings of this one
rail — `3V3`, `+3V3`, `3v3`, `+3v3`, `3.3V`, `+3.3V` — and recommends internal
consistency, **not** a canonical spelling. No source names one.

**So the answer is not "add `+3.3V` to the list".** The list cannot be
completed: the spelling space is open, and §11.4's trailing `…` is the tell.

**And it does not need to be**, which is the useful half: `KI-FLOW-001` never
reads the positive list. Anything not in the ground set and not leading with `-`
is positive. Both `+3V3` and `+3.3V` classify correctly **today**, and so would
`3V3`, `+3v3` and every spelling nobody has thought of.

**PROPOSED: strike the positive list from `spec/SPEC.md` §11.4, or mark it
explicitly non-normative** — *"illustrative; the classifier does not consult
it"*. Leaving a list that looks authoritative and is never read is how the next
reader adds `+3.3V` to it, believes the gap is closed, and ships a rule whose
behaviour did not change. It also matters because §11.4's list is what
`PLAN.md`'s Phase 3 lane-1 table points at (*"the power-direction name lists
(§11.4 Q21)"*), so a lane will read it expecting it to be load-bearing.

## P2.4 BLOCKED — the two governing documents already carry different lists

**BLOCKED, per `CLAUDE.md`: "when two governing documents conflict, do not
resolve by precedence — mark the item BLOCKED with both readings and ask."**

- **`spec/SPEC.md` §11.4 reads:** ground/negative =
  `{GND, -12V, AGND, DGND, VSS, VEE, GNDA, GNDD, 0V, EARTH}`.
- **`research/style-rules.md` §4 reads:** ground = `GND, GNDA, GNDD, AGND, DGND,
  VSS, 0V, EARTH` **and anything matching `^-?V?SS$`**; negative = leading `-`.

They differ in three ways: §11.4 has **`VEE`** and §4 does not; §4 has the
**`^-?V?SS$` regex** and §11.4 does not; §11.4 lists **`-12V`** literally,
redundant with its own leading-`-` rule. And the conflict is self-referential —
**§11.4 declares §4 canonical** (*"`research/style-rules.md` §4 is the canonical
catalogue"*) while stating a different list in its own text.

**Not resolved by precedence here.** Recommendation: **James's ratification of
P2.2 settles it directly**, because signing one list is what makes the other a
copy — which is the same signature this task was created to obtain, so the
BLOCKED item costs nothing extra to clear. Whoever applies the ratification
writes the list in **one** place and makes the other cite it.

*Aside, for whoever implements it: the `^-?V?SS$` regex matches `VSS`, `SS`,
`-VSS`, `-SS` and does **not** match `VSSA` — so `VSSA` is currently classified
positive and, being drawn up, passes by coincidence. Under P2.2's list it is
named explicitly and passes on purpose. Coincidence is not a test.*

## P2.5 What the `-` prefix rule over-catches, and which way I lean

**Measured over-catch: zero, twice.** Of the 101 stock power symbols, 18 begin
with `-` and **all 18 are genuine negative supplies**. Of the 93 power-symbol
`Value`s in KiCad's 19 template schematics, **none** begins with `-`.

**But the falsifying case exists** (E6.4): `API_Series-500.kicad_sch`, shipped by
KiCad, carries the net labels `-IN+4`, `-IN-2` and `-OUT` — op-amp inverting
input and output names, not supplies. They are out of reach **only** because
`KI-FLOW-001` classifies a power symbol's `Value` and these are `label` items.

**So the thing to defend is the precondition, not the list.** PROPOSED: write
"the classifier applies to power-symbol `Value`s and never to net names" into
the rule as an explicit precondition, and give it a falsification case built
from those three names — a sheet carrying a net labelled `-OUT` must produce
**no** `KI-FLOW-001` finding. That check fails if anyone ever widens the
classifier to net names, which is the plausible future mistake.

**Which way I lean, and why.** The north star's second half makes the false
finding the more expensive error, and I lean toward **the shorter rule and the
longer list**: keep the `-` prefix rule unchanged (its measured over-catch is
zero and its under-catch would be 18 stock symbols), and **spend the effort on
completing the ground list**, because P2.1 shows an omission there is not a
silent skip but an inverted check. Nine false findings on KiCad's own symbols is
a concrete, measured cost today; the `-` rule's cost is hypothetical and fenced
by a precondition that can be tested.

## P2.6 The one genuine conflict, which is James's to sign

**Three published sources say negative supplies point down. KiCad's own library
draws them pointing up.**

- **[G]** *"Positive supplies point up, ground and negative supplies point down.
  **Always**."*
- **[L]** *"Power connections should go up to positive voltages and down to
  negative voltages."*
- **[S]** *"If you've got a bipolar supply, try to draw the negative rail at the
  bottom and positive rail at the top."*
- **[K]** every one of the 18 stock `-…V` symbols, plus `VSS`, `VSSA` and `VEE`,
  is drawn with its body **above** the connection point — pointing **up**,
  identically to `+5V`. KiCad distinguishes negative from positive by **fill**,
  not by direction (E2.2, E2.3).

**This is not a defect in either.** It means a user who places KiCad's stock
`-12V` symbol without rotating it produces a drawing that violates all three
published conventions, and `KI-FLOW-001` will say so. **That finding is
correct.** But it will fire on the out-of-the-box behaviour of KiCad's own
library, and 20 of 101 stock symbols are in that class (E2.4).

**PROPOSED: keep the rule, and write the conflict down where the rule lives.**
The canon is unanimous and the north star's first half — *"validate the
important aspects of quality schematics"* — is exactly the argument for keeping
it: a `-12V` flag drawn identically to a `+12V` flag is genuinely ambiguous to a
reader, which is the readability harm the milestone exists to catch. What must
not happen is that a later lane, or a dogfood run, meets this finding, reads it
as a false positive, and "fixes" it. Recording it as deliberate is what stops
that.

**Alternative, if James disagrees:** apply the direction check only to the
ground set and score the negative set at a lower weight or behind an opt-in.
**Not recommended** — it discards a rule three sources state and one states with
the word "Always" — but it is the decision that is actually available, and it is
his.

## P2.7 What it costs to leave Q5 open

**More than Q1 costs, and it lands earlier than the phase order suggests.**

- **`KI-FLOW-001/002` are Phase 3, lane 1**, and `PLAN.md` groups them as *"flow
  and direction — shares the power-direction name lists (§11.4 Q21)"*. The lists
  **are** that lane's shared definition, so the lane cannot start without them.
- **Phase 2 does not touch these lists at all.** None of the six Tier 1 rules
  reads a power name. **So Q5 does not gate Phase 2 either.**
- **But the checkpoint is the cheap moment**, because P2.4's document conflict
  is live *now*: two governing documents carry different lists, and any lane
  reading either one before ratification builds on a list that ratification may
  change. That is not a Phase 3 cost, it is a cost to anyone who reads §11.4
  meanwhile.

**Recommendation: Phase 2 may start without either Q1 or Q5.** Both answers are
owed **before Phase 3 opens**, and Q5 is the one with a dated cost, because
`KI-FLOW-001` is in Phase 3's opening two-lane cut.

---

## Completion check — a deliberate Constitution §11 exception

**This task adds no code and names no `cargo` command**, in the same form
`opening-2` used for the same reason. Constitution §11 requires every task to
name an executable completion check; **this one names its check as the record's
own completeness**, because inventing a `cargo` command here would produce a
command that passes whatever this file said, which is worse than no command.

Complete when this file carries:

- [x] the seed-catalogue absence check, pasted verbatim — **E1**
- [x] a PROPOSED entry for Q1, with sources (URL + retrieval date), a
      recommendation, and the cost of leaving it open — **P1**
- [x] a PROPOSED entry for Q5, likewise — **P2**
- [x] the Lathrop spot-check reported as fetched or unfetched, plainly — **E3**:
      **the live URL is still unfetchable and is reported as such; the check was
      completed against an archived original, and which rules rested on
      summaries is stated**
- [x] `spec/SPEC.md` and `research/**` untouched — verified by
      `git diff --stat`, recorded at the commit

**No new automated check was added, so `falsification-control` has no check to
falsify.** The one measurement-shaped claim in this entry — that KiCad draws
grounds down and negative supplies up — **was** given a falsifying control and
it is recorded at **E2.3**: the reading was re-derived by rendering with
KiCad's own plotter, where an inverted coordinate convention would have shown
`GND` up and `+5V` down. It showed neither.


---

## Tick — APPROVE, 2026-08-22

**Reviewer verdict: APPROVE.** Lane `lane-t5`, commit `088df33`, base `d4c0eb8`,
merged to `main` as `0333151`. Recorded beside the tick. The reviewer had the
entry and the diff, never the lane's narrative.

**The reviewer re-derived rather than re-read**, which is the standard this
entry needed, because a research deliverable's failure mode is a number that
traces to nothing. Independently reproduced, from the sources rather than from
this file:

| Claim | How the reviewer checked it |
|---|---|
| the seed catalogue never existed | re-ran `git log --all --diff-filter=A -- "*catalogue*"` — identical output |
| KiCad's power library | **parsed `power.kicad_sym` itself: 101 symbols, name-for-name identical; 89 up / 12 down; the twelve-name list character-for-character identical** |
| the templates corpus | parsed all 19 template schematics: 93 power placements, the value distribution, **14 `lib_id`/`Value` mismatches in the exact eight files named**, 8 non-zero rotations at the named angles |
| the three leading-`-` labels | found in exactly `API_Series-500.kicad_sch`, as claimed |
| Lathrop | live URL 403s (confirmed); archive snapshot returns 200 at the cited URL; **every attributed sentence verbatim present in the text the reviewer fetched itself** |
| Greenberg | fetched the doc — **8,339 bytes, exact match**; the DNP sentence present verbatim, confirming the inversion |
| Sutherland | the six `3V3` spellings verbatim present |
| the 24-of-28 tier count | **counted the SUPPORTED tier markers by hand — exactly 4** (`KI-HIER-001`, `KI-CONN-001`, `KI-JCT-001`, `KI-XING-001`), giving 24 by subtraction |

**The tier count was checked for the specific way it could have been fake.** The
brief warned that a table collapsing *tier* support into *existence* support
would make the number mean something else. The reviewer confirmed the table
genuinely distinguishes them — `KI-FLD-001` has an existence source while its
tier is marked *"in tension"*, not supported.

**On the one measurement-shaped claim**, the reviewer notes it went further than
this entry did: *"the one 'measurement-shaped' claim (KiCad direction
convention) got an explicit falsifying control at E2.3, and I additionally
re-derived the same result directly from the library file myself, which is
stronger than the render the entry describes."*

**Scope: exact.** One file. `git diff d4c0eb8..lane-t5 -- spec/ research/`
returns **zero lines** — both OUT paths untouched.

**Not weighed, correctly:** the BLOCKED item and the Greenberg-video finding are
James's, and the reviewer left both alone per its brief. It confirmed only that
the video was not consulted, which is the standing ruling.
