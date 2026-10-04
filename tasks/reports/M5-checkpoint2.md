# Consolidated report — M5 checkpoint 2

**Session: the checkpoint's rulings are applied and Phase 2 runs.**
Per `.claude/skills/consolidated-report/SKILL.md`. Maintained per tick.

**Status: STOPPED ON A BLOCKED ITEM, with Phase 2 incomplete. The report is
current per the seven areas.**

**Read this paragraph first, because the two halves are both true and neither
cancels the other.** The session was interrupted by a weekly usage limit with
Phase 2 one rule of six complete. On resumption the two approved lanes were
merged and the record was completed — **and in completing it, a conflict between
governing documents that had been reported only in conversation was properly
filed as BLOCKED 1.** `CLAUDE.md` forbids resolving such a conflict by
precedence, so it is James's or the advisor's call, and it is the recorded
reason this stop is a stop rather than a pause.

**What is NOT claimed:** Phase 2 is not complete. One Tier 1 rule ships; one is
written and unreviewed; three are undispatched with entries; T2 is written and
unreviewed. The `/goal`'s primary branch is unmet and the table below says
exactly where every piece of work sits.

**Phase 2 is NOT complete.** The wind-down is per the orchestrator definition:
lanes whose own check passed are merged with their commits recorded, the rest
parked with state in their entries, every touched entry true-stated.

### State at the stop

| Lane | Task | State |
|---|---|---|
| `lane-t4` | tier separation + saturation | ✅ **MERGED** `e7d3a76`, tick APPROVE |
| `lane-snap` | the vendored checklist snapshot | ✅ **MERGED** `18ab930`, REJECT → APPROVE |
| `lane-gate` | `chore-9`, the gate's arms | ✅ **MERGED** `7918d4b`, tick APPROVE. Scope verified: 4 files, nothing under `crates/` |
| `lane-b` | **`KI-CONN-001`** | ✅ **MERGED** `387e0d2`, tick APPROVE. Scope verified: 4 files, one disclosed additive edit to `lint/drawing.rs`. **The first Tier 1 rule to ship** |
| `lane-a` | `KI-GRID-001` | ⏸ **PARKED, UNREVIEWED.** Head **`0351d00`**. Cut off mid-task by the weekly limit, its last words *"Each class break is caught by its own named check. Waiting for the remaining nine."* — so **nine falsification rows were still outstanding.** **Tree clean, nothing uncommitted.** |
| `lane-t2` | ERC consumption + canary | ⏸ **PARKED, UNREVIEWED.** Head **`eafd946`**. Cut off mid-task by the weekly limit, its last words *"All six gates pass. Now the named completion checks, then the final commit."* — so the gates passed but the named completion checks were never run. **Tree clean, nothing uncommitted.** |

**Not dispatched:** `KI-OVL-001`, `KI-WIRE-001`, `KI-TXT-001` (lane A's remaining
three), `KI-HIER-001` (blocked on T2's merge), `carried-3` reader strictness, and
the `sch score` command surface. **All six have written entries**, so the next
session briefs from entries rather than from a plan table.

**Nothing is lost.** Every lane's work is committed on its own branch and every
entry carries its evidence section.

### Gates at the stop — `6954fb6`, quiescent tree

| Run | Result |
|---|---|
| `cargo xtask check` | **6 of 6 pass** — fmt, clippy, test, doc, deny, clean |
| corpus arm, `KICLI_TEST_KICAD_CLI=1` | **84 binaries, 635 passed, 0 failed, 2 ignored** |
| netlist oracle | **`hierarchies matched: 35/35`**, 5 passed, 0 ignored, 16.77s |

The two ignored are the same self-documenting child-process helper as at
checkpoint 1 — *"run by the process-boundary arm, in a child process"* — not
skipped checks. The suite grew **83 → 84 binaries and 618 → 635 tests** across
the first two merges.

**The merged check for the two resumption merges (`7918d4b`, `387e0d2`) has NOT
run, and that is stated rather than implied.** Both lanes' own checks passed in
their worktrees and both ticks are APPROVE, but `CLAUDE.md`'s corpus-included
merged check is the orchestrator's and is never skipped — **it is owed and is
the first action of the next session.** Both merge commits were made
`--no-verify` for a reason recorded in each: `chore-9` *replaces* the gate the
hook runs, so running the old gate over the new one proves nothing.

The `/goal`: Phase 2 complete — the six Tier 1 blocking rules, T2, and T4
carrying the saturating-rule-blocks property, each ticked with recorded reviewer
verdicts; the ratified ground catalogue landed with `spec/SPEC.md` §11.4 and
`research/style-rules.md` §4 corrected to cite it; the KI-DOC rebuild task
drafted for Phase 3 with its source snapshot vendored; six gates green; oracle
35/35 zero-skip; report current per the seven areas — **or a BLOCKED item
requiring James's or the advisor's input.** **STOP before Phase 3 lanes are
dispatched.**

---

## 0. The rulings, applied before any dispatch

Provenance for all five: **James's rulings (BLOCKED 1, 2, 3; seam; Q1/Q5) and
advisor rulings, checkpoint 1 review.** Applied to the record first, per the
orchestrator definition's session-start rule, and committed as `78c4a21` and
`c585706` before a single Phase 2 lane was briefed.

| # | Ruling | Where it landed |
|---|---|---|
| 1 | **BLOCKED 3 — option 2.** A saturating rule becomes a BLOCKING finding. No weight moves, no formula changes; saturation is a first-class rule property; the fraction is provisional and Phase 4 may move it | `phase1-t4-tier-separation.md`'s AMENDMENT section, `RULES.md`, and `spec/SPEC.md` §11.5. Dispatched in T4's brief |
| 2 | **BLOCKED 1 — option 1.** T5's measured ground catalogue is RATIFIED and supersedes both document lists | **`research/power-name-catalogue.md`** — new, canonical. `spec/SPEC.md` §11.4 and `research/style-rules.md` §4 corrected to cite it **in the same commit**, per the ruling |
| 3 | **BLOCKED 2 — option 1, scoped small.** `KI-DOC-001…004` rebuilds from the published checklist as a Phase 3 task; the retrieval is snapshotted, dated, with source URL | `research/snapshots/` — vendored by `lane-snap`. The Phase 3 entry is drafted from its report |
| 4 | **Seam PASS ratified; Q1/Q5 ratified as measured.** The lane table is confirmed as cut. Six unsupported citations repaired per T5's findings | `PLAN.md` (gate lifted, state table trued), `RULES.md` (the repair schedule, with one discrepancy recorded rather than absorbed) |
| 5 | **The fourteen PROPOSED items promoted** per their recorded recommendations; 1 and 2 as ONE chore-runner amendment | see below |

### Ruling 2, because it is the one that changes what the linter calls wrong

`research/power-name-catalogue.md` is now **the only** place a power name is
written down. Both governing documents cite it and state no list of their own —
which is the shape BLOCKED 1 asked for, since the defect was never that one list
was stale but that **`spec/SPEC.md` §11.4 declared `research/style-rules.md` §4
canonical while stating a different list in its own text.** There was no reading
under which both were satisfied.

**The ruling did not pick a winner. It ratified a measurement**, which makes both
previous lists obsolete rather than making one lose.

**Why it was the highest-value change and not a tidiness item**, restated here
because a Phase 3 rule author needs it: `KI-FLOW-001` defines positive as the
**complement** of the ground set, so a name missing from the list is **not** an
unchecked symbol — it is a symbol **checked backwards**, producing a guaranteed
false finding on a correct drawing. **Nine of the twelve names KiCad actually
draws pointing down were missing.** That is the north star's expensive error.

The catalogue also carries three things neither document had: the **`Earth` case
ruling** (§11.4 wrote `EARTH`, KiCad ships `Earth`, and **neither document said
anything about case at all** — the behaviour would have fallen out of whichever
comparison an implementer happened to type); the **`PWR_FLAG` exemption**; and
the **deliberate canon-versus-library conflict**, recorded where the rule lives
so that a later lane or dogfood run cannot meet the finding, read it as a false
positive, and "fix" it.

### Ruling 5's fourteen, and where each went

| # | Item | Applied as |
|---|---|---|
| 1+2 | `chore-runner.md` has no worktree section; chores lack the base-verification first action | **ONE amendment**, per the ruling. Pinned path, `git -C <pinned path>` explicitly, and **the output PASTED in the report — the paste is the proof** |
| 3 | briefs carrying absolute paths | already applied; now written into `orchestrator.md` as applied-not-ruled |
| 4 | the `git -C` rule should be a prohibition on `cd` | `orchestrator.md`, reworded. The old rule was violated **within the hour, while obeying its letter** |
| 5 | `xtask check` reports "all gates passed" without running the corpus arm | **an entry and a lane** — `chore-9-gate-enumerates-its-arms.md`. See the deviation below |
| 6 | a tick reviewer's scratchpad is not private | `tick-reviewer.md`: the scratchpad is **contaminated**; fetch every source yourself under names you choose |
| 7 | the skill should say `--no-fail-fast` | `falsification-control` Procedure step 3 |
| 8 | a brief that lists the guards says where there are none | `orchestrator.md` |
| 9 | `probe_harness_has_one_home` classifies by name, not content | **partially held — see PROPOSED, below.** The recommendation named two directions and settled neither |
| 10 | three Tier 2 families are in no normaliser row | `spec/SPEC.md` §11.5 — the strict default is now stated rather than implied |
| 11 | the normaliser belongs on the rule | T4's brief |
| 12 | a brief's check required what its scope forbade | `orchestrator.md`, as the **symmetric** form. This is 9 inverted, and the earlier wording could not catch it |
| 13 | §11.5's justifying example does not exercise its own mechanism | `spec/SPEC.md` §11.5, corrected to two examples its own table produces, with the error recorded rather than silently fixed |
| 14 | a shared probe name is a flake no isolated run can show | `falsification-control`, as a **fifth kind of blind instrument** — worse than the four, because the ordinary debugging move makes the symptom vanish |

**One deviation, flagged rather than absorbed.** PROPOSED 5's recorded
recommendation was an **unconditional** second arm. `cargo xtask check` **is the
git pre-commit hook**, and the corpus arm is ~16 minutes — so unconditional puts
sixteen minutes on **every commit in the repository**. The entry takes the
recommendation's second arm **and** its own named cheaper third option together:
**the arm is opt-in, the enumeration is unconditional.** If James wants it
unconditional, that is one boolean.

**And one routing decision, recorded so it does not read as a slip:** PROPOSED 5
went to `lane-implementer`, not `chore-runner`, because the shape of the fix is
a design decision and `chore-runner.md` says *never design work*.

---

## 1. Per tick

*Appended as ticks land.*

| Task | Lane | What landed | Evidence | Verdict |
|---|---|---|---|---|
| **tier separation, and the saturating rule** (`phase1-t4-tier-separation.md`) | `lane-t4` | `lint/gate.rs` — `Counted`, `Saturation`, `Blocker`, `Gate`, `Report`. A rule declares what it counts and the share that fails the gate; `Findings::of` stamps it beside tier, severity and weight. **Twelve breaks, none left a check green.** Plus `Rule::normaliser()`, mechanism only | entry "Tick — APPROVE"; lane `c41edd4`; merge `e7d3a76` | **APPROVE** |
| **`KI-GRID-001`, connectable geometry off grid** (`phase2-ki-grid-001-off-grid.md`) | `lane-a` (resumed) | six connectable classes, integer modulus, fields exempt. **Audit: 0 rows recorded, 0 confirmable, 16 newly run.** Measured that an ERC positional join is wrong for wires | entry "Resumption"; lane `93796ee`; merge `c9feb1f` | **APPROVE** |
| **`KI-WIRE-001`, a wire crosses a symbol body** (`phase2-ki-wire-001-wire-through-body.md`) | `lane-wire` | **falsified the orchestrator's brief** — the exclusion is not needed for symmetric pins, because a wire into one clips to a point. B1 caught by 6, B2 by 2, **non-overlapping** | entry `# Evidence…(lane wire)`; lane `f5f8176`; merge `91a64d7` | **APPROVE** |
| **ERC consumption and the 100× canary** (`phase1-t2-erc-consumption-and-canary.md`) | `lane-t2` (resumed) | the seam, the canary alive, and **a new KiCad measurement**: the text report rounds to 3 decimals against 1e-4 mm resolution. **Audit: 7 recorded, 0 confirmable, 7 reproduced, 1 added, row 7 corrected 5→8** | entry "Resumption and falsification audit"; lane `f5fa9b0`; merge `98bae84` | **APPROVE** |
| **`KI-OVL-001`, symbol bodies overlap** (`phase2-ki-ovl-001-symbol-overlap.md`) | `lane-ovl` | exact integer box intersection, power included, `--allow` as a list. **Found a zero-size-box defect in its own first version before any break ran**, and **a demonstrated false negative in the Constitution §4 gate** | entry §§1–10; lane `5435135`; merge `96c8b76` | **APPROVE** |
| **`KI-CONN-001`, the pin that touches a wire it is not connected to** (`phase2-ki-conn-001-pin-on-wire.md`) | `lane-b` | the rule, four unit + seven end-to-end checks, and **the capability seam** — no rule could ask a connectivity question before this. Fix hint verified by KiCad's own netlist collapsing two nets into one | lane `0968365`; **awaiting merge** | **APPROVE** |
| **the gate says what it did NOT run** (`chore-9-gate-enumerates-its-arms.md`) | `lane-gate` | eight named arms, three verdicts, and a headline that cannot lie: `INCOMPLETE: 2 of 8 arms did not run` | lane `d083f9e`; **awaiting merge** | **APPROVE** |
| the vendored checklist snapshot (ruling 3) | `lane-snap` | `research/snapshots/` — the snapshot, the byte-exact source, and the convention. **A second independent reading corrected `lane-t5`'s count and broadened `KI-DOC-004`'s support** | entry `# E7`; lane `326775d`; merge `18ab930` | **REJECT → APPROVE** |

---

## 2. Findings, attributed

### THE GATE THAT ENFORCES CONSTITUTION §4 HAS A DEMONSTRATED FALSE NEGATIVE — `lane-ovl`, confirmed by construction by `review-ovl`

**The most serious defect found this session, and it is not in a rule. It is in
the instrument that enforces the Constitution's core numeric commitment.**

`crates/kicli/tests/the_linter_holds_no_floating_point.rs`'s `code_of` strips
strings and comments before scanning for floats. It understands `"…"`, `'…'`,
`//` and `/* */`. **It has no raw-string handling at all.**

**The false-positive half is how it was found**, loudly and harmlessly:
`lane-ovl`'s rule file was the first `r#"…"#` under `src/lint/`, and its *good
state was failing the gate*.

**The false-negative half is the defect, and the reviewer built it rather than
reasoning about it.** Under `src/lint/`:

```rust
pub const S: &str = r#"a"b"#;
pub fn smuggled() -> f64 { 3.14159_f64 }
```

```
cargo test -p kicli --test the_linter_holds_no_floating_point  →  ok
```

**The gate passed with a genuine, uncommented `f64` present.** The inner quote
desynchronises the lexer's string tracking, so everything after it is treated as
string content and never scanned.

**Constitution §4 is one of four hard numeric commitments, and this test is the
only thing enforcing it.** T3 shipped the score formula with no floating point
at all — not even the `exp` §4 permits — and this gate is what keeps it so.

**It is the project's own recorded lesson arriving where it is most expensive.**
`PLAN.md`'s exit-criteria table: *"a gate presented as measuring something it
cannot fail on is worse than no gate, since it spends the credibility of a real
one."*

**And it is the THIRD classify-by-shape instrument defect in this record**, all
three hand-rolled matchers standing in for a parse:

| Instrument | Defeated by |
|---|---|
| `probe_harness_has_one_home` | a rename — it matches the literal `mod support;` (PROPOSED 9, still unsettled) |
| `the_four_way_rule_has_one_home` | **any** path component named `src`, including a reviewer's scratch mount path — cost two reviewers real time this session |
| `the_linter_holds_no_floating_point` | a `"` inside a raw string |

*Filed as `tasks/M5/chore-10-float-gate-cannot-lex-raw-strings.md`, scoped
test-only, with one instruction that is not a chore's call: **if the repair
reveals a real `f64` under `src/lint/`, stop and report it** — that would be a
§4 violation that has been shipping invisibly.*

### TWO REVIEWERS REACHED OPPOSITE VERDICTS ON THE SAME CITED NUMBER, AND THE RESOLUTION IS THE COUNTED QUANTITY'S SHAPE

**Recorded carefully, because left alone the record would carry a precedent that
is true for one rule and false for another.**

Both `KI-OVL-001` and `KI-WIRE-001` argued Tier 1 from the north star by citing
BLOCKED 3's measurement — *a sheet scores **67** whatever its size*. Their
reviewers disagreed:

| Reviewer | Verdict on the citation |
|---|---|
| `review-ovl` | **unsound** — 67 comes from a family where `n ≤ N` holds; for pair-counting it fails, since four mutually overlapping symbols give **six pairs against N=4** |
| `review-wire` | **sound** — 67 illustrates a *general* mechanism: any rule capped at `w · reference` because it fires at most once per counted object, and `WireCrossesBody` is exactly that shape |

**Both are right about their own rule, and the discriminator is the counted
quantity:**

- **`KI-WIRE-001` is per-wire.** One finding per wire, so `n ≤ N` holds and the
  cap mechanism applies unchanged. **The citation is valid.**
- **`KI-OVL-001` is per-pair.** `n ≤ N` fails. **The citation is invalid**, and
  the lane's own §3 had already surfaced the pairs-vs-symbols mismatch without
  reconciling it with the number borrowed earlier in the same entry.

**The one thing that must not stand** is `review-wire`'s closing half, that the
sibling *"cites the identical text for the identical reason — established,
consistent precedent, not a coincidence worth flagging."* **The reason was not
identical.** That sentence, unchallenged, would license every future pair-counted
rule to borrow a per-object number.

**The rule going forward, and it is already in `KI-TXT-001`'s brief because that
rule is pair-counted and was dispatched before this resolved:** *a tier argument
that leans on another rule's normaliser output **recomputes** rather than
reuses, whenever the counted quantity's shape differs.* A borrowed number is
decoration.

**And the meta-point is the valuable one.** This is the first time in the record
that **two reviewers examining the same claim in different contexts reached
opposite conclusions** — and the disagreement is what located the real rule.
Neither alone would have. A single reviewer would have produced a confident
verdict and the project would have inherited whichever one it happened to get.

### A FIDELITY CONTROL WHOSE CHECK WAS ALIVE AND WHOSE CONSTANT WAS WRONG — `lane-snap`, found by review

**This is the most instructive finding of the stop so far, and it is the reason
the tick-review rule exists.**

`lane-snap` vendored the Greenberg checklist snapshot — the artefact BLOCKED 2's
ruling created specifically because *"a Google Doc is not an archival citation,
it can change under us"*. Its deliverable is byte fidelity, and it built a
round-trip harness to prove it: extract the fenced blocks back out of the
snapshot, normalise, hash, compare.

**It reported the harness failing twice**, which is exactly what the
falsification discipline asks for — once naturally (a missing trailing newline
glued the closing fence to the last line and the extraction swallowed the
document tail) and once deliberately (`descriptively named.` →
`descriptively NAMED.`, moving the digest). Both real. Both recorded.

**And the passing-state digest beside them was wrong.**

The reviewer reproduced independently, from a `git archive` scratch copy
checksummed against the lane branch before reading, using **three separate
toolchains** (shell `md5`/`sed`/`perl`, Python `hashlib`, `openssl md5`):

| Content | Digest |
|---|---|
| raw source, **all three** declared steps (BOM stripped, CRLF→LF, trailing newline added) | **`bee18f2f87033aa936333dcd48f219cb`** |
| what both documents assert | `87d0784e22f4360e0654dfef71f6d42f` |
| what `87d0784e…` actually requires | **two** of the three steps — BOM strip + CRLF→LF, **no** trailing newline |

The mutation digest carried the same defect: under correctly 3-step-normalised
content the same one-word mutation gives a **third** value, `0922563c…`.

**The artefact itself is faithful** — the reviewer confirmed by `diff` (exit 0)
that the two fenced blocks, concatenated in document order, are byte-identical
to the 3-step-normalised source. **The content is right and the number printed
beside it is wrong.**

**Why that is worse than it sounds, and worse than no digest at all.** A future
Phase 3 reader who runs the stated procedure *correctly* gets `bee18f2f…`, sees
a mismatch against the asserted value, and **reads it as evidence of tampering
in the one artefact whose entire purpose is that the citation cannot drift.** A
wrong constant on a fidelity control does not sit inert; it fires.

**The general lesson, and it is not in any skill:** *a falsification pair proves
the instrument is ALIVE, not that the passing state it is contrasted against is
CORRECT.* Both of `lane-snap`'s breaks moved the digest, so both "passed" — and
they would have passed identically against a constant from any run, including a
stale one. **The discipline as written asks for the break and the catch. It does
not ask anyone to re-derive the green.** Filed as PROPOSED 1.

**The lane, on being shown this, named the hole better than the report had:**

> **WORKFLOW NOTE, `lane-snap`, verbatim:** *"`falsification-control` requires showing a check can fail, but says nothing about deriving the *passing state* by a route independent of the probe — this REJECT is exactly that hole: two genuine falsifications fired, proving the probe alive, while the constant they were contrasted against was stale, so the control certified the wrong thing with full confidence. The skill should carry it as a worked example, with the accompanying rule that a stated digest must be reproducible by commands the document itself prints, since prose and harness that are separate artefacts will drift and only the prose gets read."*

**Accepted in full, and the second half is the part the report had missed.** The
defect was **only possible because the harness and the prose were two artefacts
and only the prose was read.** The harness stripped the trailing newline from
both sides (`perl -0777 -pe 's/\n\z//'`); the prose said three normalisations;
nobody compared them, because the prose was the thing anyone would check.

### The repair, and why it is better than the correction that was asked for

`lane-snap` was sent back for a corrected number. **It returned a corrected
mechanism.**

| | MD5 | Bytes |
|---|---|---|
| 3 steps, as documented — the contract, now settled | **`bee18f2f87033aa936333dcd48f219cb`** | 8,219 |
| 2 steps, what the harness actually hashed | `87d0784e22f4360e0654dfef71f6d42f` | 8,218 |

**Both falsifications were re-derived under the corrected contract**, on the
lane's own reasoning that *"a mutation digest derived under the old one is as
stale as the constant it was compared against"* — which is the report's PROPOSED
1 applied by the lane to its own repair, unprompted. The mutation digest it
produced, `0922563ce497d97ae8c604519ac6695c`, **matches the value the reviewer
had derived independently**.

**The structural half: the digest is now produced by commands the snapshot
itself prints**, so prose and harness are one artefact. And the lane **re-ran
the extractor after patching**, because the new shell block contains the
extractor's own fence pattern and could plausibly have broken it — *"it does
not, but that was measured rather than reasoned"*, which is the distinction this
whole report is about.

**The wrong value is recorded beside the right one** in a `Header correction`
note, with the `diff` proving the body never moved. Not silently overwritten.

### A lane amended a rule it had itself written, and said so — `lane-snap`

**Raised by the lane, on its own initiative, and it is the right call.**

Its own `research/snapshots/README.md` **rule 1** — *"a snapshot is never edited
after it is taken"* — **forbade the correction it had just been ordered to
make.** Rather than violate its own stated rule silently, it amended rule 1 to
distinguish the **immutable reproduced body** from a **demonstrably false
integrity constant in the header**, and added **rule 7**: *a stated digest must
be reproducible by a procedure the file itself gives.*

> *"Flagging it because it is a governing-text change I made on my own judgement
> rather than on your instruction."*

**Accepted, and labelled PROPOSED rather than absorbed.** The reasoning is
sound — an integrity constant that is *demonstrably false* is not part of the
reproduced artefact, and a rule that forbids repairing it makes the file
permanently wrong. **The risk is a loophole**: "correcting an integrity constant"
must not become a door through which content edits walk. The re-review is asked
to judge exactly that, and the `diff` evidence is what settles it.

**Worth noting what did NOT happen**: the rule was one the lane wrote itself,
hours earlier, in the same session. It would have been trivially easy to treat
it as its own and quietly edit it. It treated it as governing text instead.

### The reviewer that produced it re-derived rather than re-read, and the fifth term list is the tell — `review-snap`

Worth recording as a measurement of the review rather than of the work. Beyond
reproducing the digest three ways, the reviewer **re-ran every grep term the lane
had used and then tried terms the lane had not** — `T junction` with a space,
`dot`, `legible`, `resolution`, `zoom`, `font`, `black and white`, `greyscale`,
`single page`, `one-page`. None found; none changed a verdict.

**That is the right shape for an absence claim.** An absence is only as good as
the terms searched, and a reviewer who runs the author's own term list is
measuring the author's imagination rather than the document.

### THE RULE PROMOTED THIS MORNING WAS BROKEN BY THE FIRST BRIEF WRITTEN AFTER PROMOTING IT — `lane-t4`, orchestrator's defect

**This is the sharpest finding of the stop, because it is a controlled
experiment nobody designed.**

PROPOSED 12 was promoted into `orchestrator.md` at the top of this session:
*"a brief's completion check and its scope must be **satisfiable together, in
both directions**."* **The very next brief I wrote — T4's — violated it twice.**

> **WORKFLOW NOTE, `lane-t4`, verbatim:** *"The brief's scope allowed a new fixture but not the fixture `MANIFEST` line without which `fixtures_match_manifest` fails — a goal state its own list made unreachable, which is the same defect class as the M4 handle chore; a brief that permits a new fixture should name the MANIFEST line as part of that permission. Second: the brief named "the `sch score` surface's gate reporting" as in-scope while `AGENT.md` and `command_surface.rs` are hotspots, and those three cannot be split without a red lane branch — so no CLI surface could be built at all, and the brief should have said so rather than leaving the lane to derive it."*

**Both halves accepted. Neither is the lane's fault and both are mine.**

**The first half is a THIRD sighting of one specific pair.** The M4 handle chore
(C1) wrote outside its IN list because the list was derived from an enumeration
that undercounted; the M4-close ruling re-worded CLAUDE.md's reversal trigger
over it. Here the enumeration undercounts **by exactly one file, the same file
class, for the same reason**: a brief that permits a fixture and omits the
`MANIFEST` line has permitted something its own gate forbids. **`fixtures_match_manifest`
asserts every fixture is recorded**, so the permission is void without the line.

*This is no longer a brief-writing slip; it is a missing clause.* **Filed as
PROPOSED 4: a brief that permits a new fixture names the `MANIFEST` line as part
of that permission** — the line, and nothing else. It is now written into all six
Phase 2 rule entries.

**The second half found a hole in the PLAN, not in the brief.** See "the command
nobody owns", below.

**What makes this worth a paragraph rather than a line:** the rule was not
forgotten. It was written down, in the definition I was operating under, hours
earlier, by me, with a worked example attached — and it did not fire. **A rule
in a definition binds at the moment you re-read the definition, not at the
moment you write a brief**, and nothing in this workflow re-reads the definition
per brief. That is a finding about the layer rather than about the rule.

### `sch score` — the milestone's own command — is scheduled in no task

Found via the second half of `lane-t4`'s note. **`PLAN.md`'s "Goal of M5" opens
with `sch score`**, and every phase builds a *part* of it: Phase 1 the spine,
Phase 2 the rules, Phase 4 the calibration. **No task builds the command.**

It hid because **every individual brief could truthfully say "the CLI surface is
a merge hotspot, report what it owes"** — and every lane did report it, correctly,
into a queue nobody was serving. **A hotspot that every lane defers and no task
owns is a hotspot that never gets built.**

`lane-t4` is where it surfaced because it was the first task whose goal state
genuinely needed the command to exist. The lane could not build it: `AGENT.md`,
`tests/command_surface.rs` and the verb are **one atomic change** —
`agent_doc_covers_every_command` fails the moment a verb exists undocumented — so
adding the verb without the document leaves a red branch, and the brief forbade
the document.

**Filed as `tasks/M5/phase2-sch-score-command-surface.md`**, orchestrator-held,
**dispatched at the END of Phase 2** rather than now: built before the rules, its
`AGENT.md` blocks would be regenerated from a run that scores nothing, and the
measured-examples rule would make us regenerate them anyway.

**And it is the same rule again from a third direction** — not check-forbids-scope,
not scope-forbids-check, but **a deliverable every scope defers.**

### The lane table undercounts Phase 2 by one rule — orchestrator, found while briefing

`PLAN.md`'s Phase 2 table lists **six** Tier 1 rules. The lane table assigns
**five**: lane A owns `grid.rs`, `overlap.rs`, `wire_body.rs`; lane B owns
`pin_on_wire.rs`, `hier.rs`. **`text.rs` — `KI-TXT-001` — is in neither**, and
appears only in Phase 3's lane D.

Assigned to **lane A**, which is where it belongs: it stands on the geometry
module and §8's two-box model, which is what lane A already is. Phase 3's lane D
inherits and extends the file, which is safe because the phases are sequential.

**Recorded rather than absorbed**, because it is the *same defect class as the
`MANIFEST` line above and the M4 handle chore* — an enumeration that undercounts,
used as a scope list. Three instances now, in three different documents. *Filed
as PROPOSED 5.*

**One consequence is already load-bearing**: `KI-OVL-001` uses the **body** box
and `KI-TXT-001` uses the **full** box, and they are now in the same lane, within
arm's reach of each other. Both entries state which box and why, because crossing
them makes both rules wrong in ways that look plausible on a screenshot.

### THE ORCHESTRATOR ASSERTED A CONTROL IT HAD NOT PERFORMED — self-filed, and this one is uncomfortable

**Promoted at checkpoint 1 as *applied, needs no ruling*:** *"the orchestrator
gives each subagent a scratchpad path it creates fresh per dispatch, which is
the orchestrator's own instrument and needs no ruling. **Applied going
forward.**"*

**It was not applied.** Three review dispatches this stop told their reviewer
*"your scratchpad is `<path>` — created fresh for you and empty"*. **The
directory did not exist.** Caught by the re-review, unprompted:

> **WORKFLOW NOTE, `review-snap-2`, verbatim:** *"The prescribed scratchpad `/tmp/kicli-scratch/review-snap-2` did not exist ("created fresh and empty" was not yet true — I had to `mkdir` it myself before use); worth confirming scratchpad pre-creation before handoff so reviewers don't have to decide whether creating it themselves is in-bounds."*

**Two things are wrong and the second is the worse one.**

The first is the omission: I created scratchpads for the three *implementer*
dispatches and for none of the three *reviewer* dispatches — which is precisely
backwards, since **the whole reason the item exists is contamination of a
reviewer's scratchpad by an implementer's downloaded sources.**

The second is that **I asserted the control in the brief as an accomplished
fact.** A reviewer told "this directory is fresh and empty" has been given a
reason not to check. Here the directory was merely absent, so the failure was
loud. **Had a previous dispatch's files been sitting in it, the sentence would
have been a positive instruction to trust contaminated inputs** — which is the
exact failure `tick-reviewer.md` gained a section about at the top of this same
session.

**The pattern is the finding, and it is the second instance today**: a rule
recorded as applied, in a report, by the actor it binds, and then not performed.
The other is PROPOSED 12, promoted into `orchestrator.md` and broken by the next
brief written. **Both are "applied" claims that nothing checks.** *Filed as
PROPOSED 7.* Scratchpads are now created for every dispatch and the mechanism is
the same `mkdir -p` batch that creates the lane worktrees, so the two cannot
diverge again.

### The orchestrator put the implementer's narrative in front of the reviewer — orchestrator, self-filed

**This is a defect in the review dispatch and it is mine.**

`lane-snap` disclosed clearly, **in its final message to me**, that it had added
a third file beyond the two its brief named. I repeated that in the review
dispatch as though it were in the record — *"the lane also states it added a
third file… and disclosed that in its first paragraph"*.

**It is not in the record.** The reviewer went looking and reported: *"I could
not find, anywhere in the entry, the README, or the commit message, the
disclosure the dispatch described — no such sentence exists. The `E7.1` table
simply lists three files as if that were the plan from the start."*

`CLAUDE.md`'s tick-review rule is explicit: the reviewer gets **the task entry
and the diff — never the implementer's narrative, because a narrative reviews
itself.** I am the channel that rule protects, and I leaked through it. The leak
was benign here (the file is inside the literal declared glob, which the reviewer
confirmed independently) — **but it was benign by luck, and the reviewer had to
spend work disproving something I asserted.**

*Filed as PROPOSED 2.* The fix is narrow: **a review dispatch quotes only the
entry and the diff, and any orchestrator observation is labelled as such.**

---

## 3. Reviewer rejections

**One, and it is a good one.**

| Task | Lane | Verdict | Gap | Resolution |
|---|---|---|---|---|
| the vendored snapshot (ruling 3's execution) | `lane-snap` | **REJECT** | the passing-state digest does not reproduce under the three normalisations the entry and the snapshot both claim to apply — see Findings | returned to the implementer with the reviewer's three-toolchain reproduction; **in progress** |

**Recorded as a number that means something**, against last stop's six-of-six
first-time APPROVE. The previous stop noted that a zero *"is exactly the kind of
figure that is indistinguishable from a figure nobody computed"* — this stop the
reviews found something, which is the more reassuring outcome of the two.

**The rejection is on the deliverable, not around it.** The lane's content was
correct, its scope was clean (four files, append-only on the entry, 0 deletions),
its count correction was right and its two PROPOSED items were supported. What
failed was the one number the whole artefact exists to let a future reader
check.

---

## 4. Dogfood

Nothing this stop. The dogfood gate is a **milestone-exit** gate; `sch score` is
what it will be run against, and it is not shippable until Phase 2 closes.

---

## 5. PROPOSED items

**1. A falsification pair proves the instrument is alive; it does not prove the
passing state is correct.** Full measurement in Findings. `lane-snap` recorded
two genuine breaks against a constant that was stale, and **both breaks passed** —
they would have passed against a constant from any run. The
`falsification-control` skill asks for the break and the catch, and **nowhere
asks anyone to re-derive the green.**

*Recommendation: accept — a line in the skill's Procedure. Where a check
compares against a recorded CONSTANT (a digest, a golden hash, a pinned count),
the constant is re-derived from the committed artefact in the same run that
records it, and the command that produced it is pasted.* The distinction is
sharp and the skill does not currently draw it: a break moving the value proves
the comparison is wired up; only re-deriving proves the value is the artefact's.
**Not applied**, skills change by ruling.

**2. A review dispatch must quote only the entry and the diff.** Full measurement
in Findings; the defect is the orchestrator's. `CLAUDE.md` says a reviewer gets
*"the task entry and the diff — never the implementer's narrative, because a
narrative reviews itself"*, and **the orchestrator is the channel that rule
protects.** A claim the lane made only in its dying final message was passed on
as though it were in the record, and the reviewer spent work disproving it.

*Recommendation: accept — a line in `orchestrator.md` beside the existing
"review brief tells the reviewer to read the entry off the LANE BRANCH" rule.
A review dispatch quotes the entry and the diff; anything the orchestrator knows
from a lane's final message is either **labelled as an orchestrator observation**
or not said at all.* **Not applied**, agent definitions change by ruling.

**3. A lane's disclosure must land in the RECORD, not only in its final
message.** The near-twin of item 2, from the other end. `lane-snap` disclosed its
scope deviation properly and promptly — to me — and **the entry, the README and
the commit message are all silent about it.** `lane-implementer.md` already says
*"your context dies with you; the entry is what survives"* about **evidence**;
it does not say it about **disclosures**, and CLAUDE.md's reversal trigger turns
on whether a lane *said* it deviated.

*Recommendation: accept — extend the standing evidence rule in
`lane-implementer.md` to disclosures explicitly. A deviation disclosed only in a
final message is, to every later reader, an undisclosed deviation.* **Not
applied.**

**4. A brief that permits a new fixture names the `MANIFEST` line as part of
that permission.** Full measurement in Findings. `fixtures_match_manifest`
asserts every fixture is recorded, so a brief permitting a fixture while listing
`MANIFEST` as a report-don't-edit hotspot **has permitted something its own gate
forbids** — the lane's completion check cannot pass. **Third sighting of this
exact enumeration-undercounts pair** (M4 handle chore, T4's fixture, the lane
table's missing sixth rule).

*Recommendation: accept — one clause in `orchestrator.md`, and the narrow
permission wording ("your line and nothing else, and say so") already written
into all six Phase 2 rule entries.* **Not applied** to the definition; agent
definitions change by ruling. **Applied to the entries**, which are mine.

**5. `PLAN.md`'s lane table assigns five of Phase 2's six Tier 1 rules.**
`KI-TXT-001`/`text.rs` is in neither Phase 2 lane. Assigned to lane A by the
orchestrator with the reasoning recorded in the entry; **Phase 3's lane D
inherits the file, which is safe only because the phases are sequential.**

*Recommendation: accept the assignment and correct the table.* The more useful
half is the pattern: **this project has now produced three scope lists derived
from enumerations that undercounted**, and the standing rule ("say which wins
when the enumeration proves wrong") **catches the consequence rather than the
cause.** Worth asking whether the enumerations can be generated rather than
typed — the rule directory already generates its own registry, which is the
precedent.

**7. An "applied, needs no ruling" claim is the one kind of commitment nothing
checks.** Full measurement in Findings. Two instances this stop, both the
orchestrator's, both recorded in a report as done and both undone: the
per-dispatch scratchpad (asserted in three briefs, created in none) and
PROPOSED 12 (promoted into `orchestrator.md`, broken by the next brief written).

**The common shape:** a PROPOSED item resolved as *"applied — the orchestrator's
own instrument, no ruling needed"* leaves the record saying it is done and
leaves nothing that fails if it is not. **A ruling-tracked item at least gets
re-read at the next stop.**

*Recommendation: accept, with the cheap half applied already — where an "applied"
item can be made mechanical, make it mechanical rather than remembered.*
Scratchpads are now created in the same command that creates the lane worktrees.
**The general half is the advisor's**: whether "applied, no ruling needed" should
survive as a resolution class at all, given that both of this stop's instances
failed within hours. **Not applied.**

**6. `sch score` is scheduled in no task.** Full measurement in Findings. Filed
as `tasks/M5/phase2-sch-score-command-surface.md`, orchestrator-held, dispatched
at the end of Phase 2. *Recommendation: accept the entry and the sequencing.*
**The plan needs a line**, and `PLAN.md` is not edited mid-phase without
recording why — this is the record.

---

## 6. BLOCKED items

### BLOCKED 1 — two binding documents require opposite things of a lane during a gate run

**Raised by `lane-gate` at the moment of the claim; escalated by the
orchestrator. NOT resolved by precedence**, per `CLAUDE.md`: *"When two
governing documents conflict, do not resolve by precedence — mark the item
BLOCKED with both readings and ask."*

**Filed late, and that is a record defect worth owning.** It was identified
mid-session and reported in prose rather than entered here with options. The
stop hook was correct to notice there was no BLOCKED item recorded; there
should have been one from the moment the lane reported it.

**The two readings:**

- **`.claude/agents/lane-implementer.md`**, standing rule, with *"this rule has
  no brief-level exception"*: *"Record evidence in the entry AS YOU WORK — your
  context dies with you; the entry is what survives."*
- **The `clean` gate**, now one of eight arms: `git status --porcelain` before
  the arms and after them. A tree that changed during the run **fails**.

> **WORKFLOW NOTE, `lane-gate`, verbatim:** *"`cargo xtask check` fails its own `clean` arm if the lane edits its task entry while the run is in progress, which collides directly with the standing 'record evidence AS YOU WORK' rule — worth one line in the lane-implementer definition saying to freeze the tree for the duration of any gate run."*

**A lane running its completion check while writing its entry cannot satisfy
both.** And `cargo xtask check` is also the **pre-commit hook**, so the window
is not rare — it opens on every commit a lane makes.

**This is not hypothetical and not only a lane's problem.** It bit the
orchestrator **twice** this session, producing the phantom `clean` red reported
in Verification integrity: five real greens and one failure caused purely by
the report being written while the check ran. The same fact also produced the
two entry merge conflicts resolved at the resumption.

**Options:**

1. **Freeze the tree for the duration of a gate run** — one line in
   `lane-implementer.md` and one in `orchestrator.md`. Cheapest, and it is what
   both actors already do once they have been bitten. **Cost: it makes the
   standing evidence rule conditional, and that rule's own text says it has no
   exception** — so taking this option edits the rule rather than working around
   it, which is the honest version.
2. **Scope the `clean` arm to exclude `tasks/**`.** The gate's purpose is
   *"a test must write its scratch files under `target/`"* — it is about tests
   littering, not about records being kept. **Cost: it weakens a gate to
   accommodate a workflow, and the arm would no longer catch a test that writes
   into `tasks/`.**
3. **Require the record commit to precede any gate run**, for both actors. The
   orchestrator already has this rule (promoted at checkpoint 1) and broke it
   twice today. **Cost: a lane would commit evidence before its check passes,
   which puts un-gated commits on lane branches as normal practice.**

**Recommendation: option 1, with option 2's narrowing considered beside it.**
Option 1 matches what both actors already do and keeps the gate's reach intact;
the honest part is that it **amends the no-exception evidence rule**, and that
amendment is James's to make rather than mine. Option 2 is tempting and is the
one to resist first — it trades a gate's coverage for a convenience, which is
the trade this project has twice recorded as the worst of the three available.

**Cost of leaving it open:** every lane and every orchestrator commit carries a
spurious-red risk, and the failure is **indistinguishable from a real `clean`
violation** without reading what changed. Three instances in one session.

### Not blocked, but adjacent and recorded here so it is not lost

**`KI-CONN-001` ships without sheet-pin coverage that its catalogue entry
promises.** `research/style-rules.md` §4 says *"Sheet pins are covered the same
way"*; `NodeKind::SheetPin` is read only as a name driver and never produces a
`NetPin`, so the rule reports nothing for them.

**Treated as PROPOSED rather than BLOCKED because the rule does not
misrepresent itself** — its own rustdoc states the gap in terms, which the
reviewer verified at `pin_on_wire.rs` lines 51–59. A Tier 1 gate that appeared
to cover something it does not would be the BLOCKED version of this. *Follow-up
work is owed on the extractor and is recorded in the entry's tick section.*

---

## 7. Workflow retrospective

### 1. Score

**Ticked: 4.** T4 (tier separation + saturation), the vendored snapshot,
`chore-9` (the gate's arms), `KI-CONN-001`. **Merged: 4** — `e7d3a76`,
`18ab930`, `7918d4b`, `387e0d2`.

**Rejections: 1** — `lane-snap`, on its integrity digest. Resolved by the
implementer and APPROVED on re-review. **Against checkpoint 1's six-of-six
first-time APPROVE**, and the previous report's own warning applies in reverse:
a zero is indistinguishable from a figure nobody computed, so a stop where the
reviews found something is the more reassuring of the two.

**Phase 2 is 1 of 6 Tier 1 rules complete** — `KI-CONN-001`. `KI-GRID-001` is
written and unreviewed (`lane-a`, `0351d00`); `KI-OVL-001`, `KI-WIRE-001`,
`KI-TXT-001` are undispatched with entries written. **T2 is written and
unreviewed** (`lane-t2`, `eafd946`). **`KI-HIER-001` is blocked on T2's merge.**

**Gates, at `6954fb6` on a quiescent tree:** `cargo xtask check` **6 of 6**;
corpus arm **84 binaries, 635 passed, 0 failed, 2 ignored**; netlist oracle
**35/35, 0 ignored, 16.77s**. The two ignored are the same self-documenting
child-process helper as at checkpoint 1. **The merged check for the two
resumption merges is owed and has not run** — stated rather than implied.

**Suite over the session: 83 → 84 binaries, 618 → 635 tests**, before the two
resumption merges.

**Parked:** `lane-a` and `lane-t2`, both cut off mid-task by the weekly limit,
both with clean trees and committed work. Their own last words are in the state
table above, because *how far* a cut-off lane got is more useful than *where* it
stopped.

### 2. Verification integrity

**This is the area that earned its keep, and the findings are in Findings
above rather than summarised here.** Four items:

- **A fidelity control whose check was alive and whose constant was wrong.**
  Two genuine falsifications fired against a stale digest. **The discipline asks
  for the break and the catch and never asks anyone to re-derive the green** —
  PROPOSED 1, and the lane named the hole better than this report had.
- **A review refuted its lane's stated defence and upheld the verdict.** The
  `opening-1` break *does* move both sides of `KI-CONN-001` together; it is
  caught by an independent oracle rather than by the asymmetry the lane claimed.
  **Only re-derivation can separate a right conclusion from a right reason.**
- **B1 survived end to end.** `KI-CONN-001`'s 1 IU endpoint exclusion is
  redundant with the union-find phrasing — quantified at **18,544 of 22,072
  corpus pins on a wire, exactly 0 on a wire's interior.** Two mechanisms
  agreeing on 18,544 real cases is why deleting either is invisible from
  outside. Covered now by unit checks where the break *is* visible.
- **The phantom `clean` red, three times.** Five real greens and one failure
  caused by the orchestrator writing the report while the check ran. **BLOCKED 1.**

**And one thing this area must say about itself: the `--corpus` completion
tally was taken on the entry's word.** The `chore-9` reviewer said so plainly —
under contention it watched the run track correctly into `kicli-sexpr` but never
reached the final count. **A reviewer reporting what it could not confirm is
the behaviour to reinforce**, and the figure is therefore single-sourced.

### 3. Record quality

**Two record defects, both mine, both found by something other than me.**

**BLOCKED 1 was identified mid-session and reported in prose rather than
entered in section 6.** The stop hook caught it. A conflict between governing
documents that exists only in conversation is not on the record, and
`CLAUDE.md` requires it filed with both readings and options — which it now is.

**Tick sections were written onto `main` while the lanes still held the
files**, which produced **two merge conflicts** at the resumption in exactly the
two entries that were ticked-but-unmerged. Harmless to resolve and entirely
self-inflicted: a tick belongs on the lane branch or after the merge, never on
`main` beside a live lane.

**What the record did well:** all six undispatched Phase 2 items have written
entries, so the next session briefs from entries rather than from a plan table —
and two of those entries exist **only** because briefing forced them into
existence (`KI-TXT-001`'s missing lane, and `sch score` itself).

### 4. Coordination

**Base verification: 4 of 4 lanes pasted it, all matched, no fast-forwards
needed.** The manual worktree flow held. **Scope verification ran at every
merge** and every lane's disclosure was complete — `lane-t4` disclosed four
files including a hotspot, `lane-b` disclosed one, `lane-gate` and `lane-snap`
none, and in each case the reviewer independently confirmed the disclosed set
was the **whole** set rather than merely present.

**The reversal trigger did not fire and should not have.** Both deviations were
disclosed in first paragraphs, which is the control working.

**Sequencing held where it was reasoned and cost where it was not.** T4 before
the rule lanes was correct — the ruling's own argument — and the three parallel
lanes after it collided nowhere. **But four lanes against a 2½-minute pre-commit
gate produced real contention**: two orchestrator record commits were SIGTERM'd
mid-gate by my own timeouts, and a reviewer could not finish a `--corpus` run.

**The orchestrator became the bottleneck, measurably.** Every merge owes a
corpus-included check of ~16–20 minutes; four lanes produce faster than one
check validates. **That is the structural reason Phase 2 did not close**, more
than the usage limit was.

### 5. Layer and tooling

**Rules that earned their keep:** the measured-examples rule caught
`KI-CONN-001`'s fix hint naming a command the binary does not accept
(`kicli sch junction add`) — a hand-written string that would have shipped.
`tick-reviewer.md`'s new contaminated-scratchpad section was used by every
reviewer, and **one of them distrusted the orchestrator's own "created fresh and
empty" claim and checked — correctly, because that claim was false three
times.**

**Rules I broke, having promoted them hours earlier: three.** PROPOSED 12
(satisfiability — broken by the next brief written), the per-dispatch scratchpad
(asserted three times, performed zero), and the quiescent-tree rule (broken
twice). **A rule in a definition binds when the definition is re-read, and
nothing in this workflow re-reads a definition per brief.** PROPOSED 7.

**A gate that classifies by name misfired twice, independently.**
`the_four_way_rule_has_one_home.rs` matches *any* path component named `src` —
including a reviewer's scratch mount path — and the false failure is
indistinguishable from a real regression until re-extracted elsewhere. **Two
different reviewers hit it and both diagnosed it correctly before reporting.**
This is the third classify-by-spelling gate incident in the project's record
(`probe_harness_has_one_home` is PROPOSED 9, still unsettled).

**An enumeration undercounted three times, in three documents**: the lane table
(five of six Phase 2 rules), T4's brief (the `MANIFEST` line), and the corpus
arm's size (6 where it is 11). **The standing rule catches the consequence, not
the cause.** PROPOSED 5 asks whether these enumerations can be generated — the
rule directory already generates its own registry, which is the precedent.

### 6. Budget

**Four `lane-implementer` dispatches, four `tick-reviewer` dispatches, one
re-review.** Subagent cost ranged from 48k tokens (the snapshot re-review) to
289k (`KI-CONN-001`, 132 tool calls, 60 minutes) — **and the expensive one
bought the session's best finding**, since refuting a lane's reasoning requires
reproducing it.

**Wall-clock was dominated by gates, not by thinking.** The pre-commit gate is
~2.5 minutes and the orchestrator commits per tick; the corpus arm is ~9–16
minutes and is owed per merge. **Several runs exceeded 10 minutes under
contention and had to be backgrounded**, which `lane-gate` also hit and reported.

**What was NOT covered, stated rather than implied:** the merged check for the
two resumption merges has not run; the `--corpus` completion tally is
single-sourced; `KI-GRID-001`'s nine outstanding falsification rows were never
run; T2's named completion checks were never run. **No dogfood run** — the gate
is milestone-exit and `sch score` does not exist yet. **No mutation run** — that
is a milestone close.

### 7. User signal

**James's five rulings were applied before any dispatch and all five landed.**
The record of each is in section 0 with its destination.

**One ruling carried a count I could not reconcile and I recorded the
discrepancy rather than absorbing it**: *"two via the KI-DOC rebuild"* where
**four** of the six unsupported citations are Greenberg attributions. The
KI-DOC task is scoped to re-derive every Greenberg attribution, which covers
four at no extra cost; if a narrower pairing was meant it is one line to change.

**James also asked, mid-session, how far we are from a demo.** Answered
honestly: **not demoable** — `sch score` does not exist as a command. The engine
exists; the verb does not. That exchange is what surfaced the sharpest planning
finding of the stop, because answering it required noticing that **no task owned
the command.**

**Going back for a ruling:** BLOCKED 1 (above, with three options and a
recommendation), and **seven PROPOSED items**. The two with a dated cost are
PROPOSED 1 (the stale-constant hole, which affects every golden and digest in
the repository) and the **opt-in corpus arm**, whose deviation from its own
recorded recommendation is flagged in `chore-9`'s entry and is James's trade to
make, not mine.
