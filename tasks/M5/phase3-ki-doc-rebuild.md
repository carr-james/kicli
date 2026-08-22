# The KI-DOC family, rebuilt from the primary source (Phase 3)

**Provenance: James's ruling on BLOCKED 2, checkpoint 1 review**, verbatim:

> option 1, scoped small: the `KI-DOC-001…004` family rebuilds from the
> published checklist text as a Phase 3 task. The retrieval is SNAPSHOTTED into
> the repo (dated, with source URL) so the citation cannot drift; the video
> remains skipped per James's standing ruling, whose own stated condition this
> fulfils. Draft the task entry now; it dispatches in Phase 3.

**Status: DRAFTED, NOT DISPATCHED.** This entry is written at checkpoint 2 and
dispatches in Phase 3, into lane **D** (geometry, text, fields, docs, DNP), which
owns `lint/rules/docs.rs` per the lane table.

## Why the family is being rebuilt rather than written

**A ruling whose stated condition has been met went back to James, and he ruled
on it.** `research/style-rules.md` §8's Q2 said, in full:

> *"This catalogue used the published summaries […] not the video. **If his
> checklist is published in a citable form, it should be the primary source for
> the KI-DOC-\* family.** Want me to work through the video?"*

`lane-t5` found the checklist published as citable text. **The video is still
skipped and nothing about that is in question** — the ruling that skipped it is
James's, it was re-confirmed at the M5 plan review, and consulting a published
text document is *inside* it rather than an exception to it.

**What changed is the source's quality, and it is measurable.** The
published-summaries route produced **two of the six unsupported citations** T5
found across the whole 28-rule catalogue, which makes this family's sourcing
measurably the weakest in it. The Hackaday write-up §4 was built from is
**1,450 characters** — a table of contents, not a checklist.

## The snapshot is the citation. Do not fetch the URL.

`research/snapshots/greenberg-schematic-checklist-2026-08-22.md`, with the
byte-exact export beside it at
`research/snapshots/greenberg-schematic-checklist-2026-08-22.source.txt`.

Vendored by `lane-snap` at checkpoint 2, **because a Google Doc is not an
archival citation** — it is version-dated rather than immutable and can change
under us. That was the honest counter recorded against the ruling's own
recommendation, and the snapshot is the answer to it.

**Retrieval facts**: `…/export?format=txt`, `curl -sS -L`, anonymous, **HTTP
200**, **8,339 bytes**, MD5 `e04ed36a3066374a7e02e4224adfc109`, UTF-8 with BOM,
CRLF, no trailing newline. Document's own version date: **2026-02-15**.

**Cite the snapshot path, not the URL.** A citation to a live Google Doc is
exactly the drift the ruling spent a lane preventing.

## What the source actually contains — 47 items in 9 groups

*Corrected at checkpoint 2. `lane-t5` reported **55 items in 8 groups**; the
bytes matched exactly, so the discrepancy was a **counting error, not drift** —
consistent with one group heading being absorbed into the title. Corroborated
independently: all nine group headings are bold in the HTML export.*

The five groups bearing on this family, and the items that do:

| Group | Item |
|---|---|
| 1. Visual Design Best Practices | *"Functional blocks are clearly labeled…"* (whitespace or a box) |
| | *"Functional blocks have text that describes what they do and their requirements (e.g., Vbatt to 3.3 V @ 1 A switching power supply)"* |
| | *"It's clear where your power is coming from and what the power requirements are (V/I)"* |
| | *"All connectors have text that describes where they go, and describes signals (voltage, current, names)"* |
| 8. BOM Integration | *"Add 'MFR' (Manufacturer) and 'MPN' (Manufacturer's Part Number) to **all components** as attributes."* |
| | *"**Bonus points** for adding a distributor, distributor part number, description, and datasheet link."* |
| | *"Add required cable or required accessory part numbers as text, or add as Do Not Place (DNP) components if you want them to show up in the BOM."* |
| 9. You're Almost Done | *"If this is a revision, record changes to the schematic in a table or in nearby documentation."* |
| | *"Update your schematic version and/or date."* |

## Goal state, as the checks that prove it

### 1. Each of the four rules is rebuilt against the snapshot, and each cites it

`KI-DOC-001` MFR/MPN, `KI-DOC-002` datasheet, `KI-DOC-003` title block,
`KI-DOC-004` sheet notes. Every citation names the snapshot path and quotes the
item verbatim. **A rule whose quote is not in the snapshot does not ship.**

### 2. Three source facts change what the rules may require

- **`KI-DOC-002` is optional at source.** *"**Bonus points** for adding a
  distributor… and datasheet link."* This **supports the low weight and rules
  out requiring it.** A rule that fails a drawing for a missing datasheet link
  is not implementing this source.
- **`KI-DOC-003`'s `title` has NO source.** The item supports `rev` and `date`.
  A second item — *"If this is a revision, record changes to the schematic in a
  table or in nearby documentation"* — supports `rev` further and still says
  nothing about `title`. **Either find a source for `title` or drop it from the
  rule and say so.** Do not carry it because the field exists.
- **`KI-DOC-004` is supported by FOUR items, not two** — stronger than §4's
  `corroboration: weak`, which was assigned when only the Hackaday summary was
  in hand. **The weak claim that survives is the mechanical proxy**: presence of
  text ≠ presence of explanation, and no measurement can close that gap. Say so
  in the rule's own rustdoc; it is the honest limit.

### 3. Four citation repairs land with this task

Per `RULES.md`'s repair schedule. **Each repair is recorded at the moment it is
made** — a citation quietly deleted is indistinguishable from one that was never
there.

| Rule | Repair |
|---|---|
| `KI-JCT-001` | strike the Greenberg citation — four-way junctions are **absent** from the checklist (0 hits for `junction`, `4-way`, `four-way`, `T-junction`) |
| `KI-TXT-002` | strike it — the "mono-PDF legibility item from Greenberg" **does not exist** (0 hits for `PDF`, `mono`, `legib`) |
| `KI-LAY-002` | strike it — *"one page, one idea"* is **not in Greenberg** (0 hits for `one page`, `one idea`) |
| `KI-DNP-001` | strike it — **and see PROPOSED A below, which says the stronger claim overshoots** |

### 4. Constitution §6 governs the output, and this family is the one that floods

`KI-DOC-001` fires **per component with a missing attribute**. On a 200-symbol
sheet that is up to 200 findings from one rule. **Grouping by part class is
kicli's own §6 mitigation and NOT the source's** — say so where it is written
down, because a reader will otherwise assume the grouping is Greenberg's.

## Two PROPOSED items raised by `lane-snap`, to be decided in this task

**A. `KI-DNP-001`'s "recommends the opposite" overshoots, and the correction is
more useful than the strike.** The checklist endorses DNP for **one narrow
purpose** — getting *required* cables and accessories into the BOM — not for
speculative or just-in-case parts, which is what the rule penalises. So the
citation should be **struck** (it does not support the rule), **but** the
required-accessory case should be treated as a **candidate exemption**: a
false positive this rule would produce on a drawing following the checklist.
*Recommendation: accept both halves. The exemption is the half with a check
attached.*

**B. `KI-FLOW-002`'s support is weaker than E4.3 recorded.** *"All symbols are
schematic symbols, not packages (inputs on left, outputs on right…)"* is about
**pin arrangement within a symbol**, not sheet-level flow; and *"Data flow…
clear and labeled"* requires **clarity, not a direction**. *Recommendation:
downgrade the Greenberg citation to weak/partial rather than strike it.* This
does **not** affect the five-unsupported headline, and `KI-FLOW-002` is lane C's
rule — so this item **travels to lane C**, not to this task.

## Falsification obligation

Per `.claude/skills/falsification-control/SKILL.md`.

- **Every rule is shown firing and not firing** on committed fixtures — a
  documentation rule is an absence check, and **an absence check carries a
  presence control** or it is the skill's blind instrument #1.
- **`KI-DOC-002` is shown NOT failing a drawing** that omits the datasheet link,
  because the source makes it optional. That check is the executable form of
  goal state 2 and it fails if anyone later promotes the rule.
- **The snapshot's own fidelity is already checked** by the round-trip
  `lane-snap` built; do not rebuild it. **Do confirm it still passes** before
  citing the snapshot — that is one command and it is what makes the citation
  worth more than the URL.

## Scope

**IN**
- `crates/kicli/src/lint/rules/docs.rs` — new, lane D's file
- new test files and new fixtures under `crates/kicli/tests/`
- `research/style-rules.md` §4's four `KI-DOC-*` rules **and the four citation
  repairs in goal state 3** — coordinate with the orchestrator, since three of
  the four repairs are to rules lane D does not own
- this file, for the evidence, written AS YOU WORK

**OUT** — `research/snapshots/**` (**a snapshot is never edited after it is
taken**; a new retrieval is a new dated file), every merge hotspot, every other
entry.

## Completion check

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo xtask check
```

plus each of the four rules' checks by name, each shown capable of failing, and
the `KI-DOC-002`-is-optional check recorded with its falsification.
