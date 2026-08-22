# Tier 1 does not reduce the score (Phase 1, T4)

**Provenance: `tasks/M5/PLAN.md` Phase 1, RATIFIED by James's ratification and
advisor rulings, M5 plan review.**

**Depends on T1 and T3.** It is the property that binds the finding type to the
formula, and it cannot be checked before both exist.

## The rule, from `spec/SPEC.md` §11.5

> **Tier 1 findings do not reduce the score.** They set `"gate": "fail"`
> independently. A schematic can score 96 and still fail the gate; that is
> intentional and must be visible in the output.

And §11.2: `sch score --gate` **may require `kicad-cli`**, because half of Tier 1
is ERC-owned. Absence is a structured error and exit 6. §6.1: **gate failure is
exit 5**, and findings without `--gate` are **exit 0** — *"findings are data, not
failure."*

## Why this is a task and not a line of code

Because the property is easy to state, easy to implement, and easy to break
without noticing — the ordinary way a scorer grows is for a Tier 1 finding to
acquire a penalty "just so it shows up in the ordering". The plan's exit-criteria
table names what this gate must be able to fail on: **a Tier 1 finding moving the
score, or a Tier 2 finding failing the gate.** Both directions.

## Goal state, as the checks that prove it

### 1. The fixture exists, and it is the deliverable

The plan's completion check, verbatim: *"A file that scores 96 and fails the gate
exists as a fixture, and both facts are visible in one output."*

**All three clauses are load-bearing:**

- **It exists as a fixture** — committed, purpose-built, per Constitution §11 and
  `spec/SPEC.md` §18. Not constructed in a test body, because the next milestone
  should be able to point at it.
- **It scores high** — a genuinely well-drawn sheet, so the number is not an
  artefact. "96" is illustrative, not a target to engineer; a real high score is
  the point and a hand-tuned one is worthless.
- **Both facts are visible in ONE output.** An agent that has to run two commands
  to learn its build is broken will run one. Constitution §6 governs the shape:
  outputs are designed for LLM context budgets, and a view that floods is wrong
  whatever it contains.

### 2. Both directions are checked

- a Tier 1 finding **does not** change `raw_penalty` or the score;
- a Tier 2 finding **does not** set `gate: fail`.

**Two separate checks.** One check over a fixture carrying both kinds passes if
the implementation swaps them, and that is the mistake most worth catching.

### 3. Exit codes, per §6.1

- `sch score` with findings and no `--gate` → **exit 0**;
- `sch score --gate` with a Tier 1 finding → **exit 5**;
- `sch score --gate` with `kicad-cli` absent → **exit 6**, structured, naming the
  binary and an install hint (§14.1).

`crates/kicli/src/cli/exit.rs` already owns the table and has tests over it.
Read them; do not build a second one.

### 4. The output makes the separation legible without a footnote

This is where Constitution §6 and the north star meet. A reader — an LLM agent
under a context budget — must be able to see *at a glance* that the score is
high AND the gate failed, and not read the high score as "fine".

**That is a presentation judgement, and the entry records what you chose and
what you rejected.** If it feels like a value call rather than a formatting one,
it is: park it as PROPOSED against the north star (`RULES.md`) rather than
guessing. *"It must never reward a schematic that is impossible to read and
understand"* — and an output that lets a gate failure hide behind a 96 is a way
of rewarding one.

## Falsification obligation

Per `.claude/skills/falsification-control/SKILL.md`.

- **Both direction checks are shown failing**: give a Tier 1 rule a penalty and
  confirm the first goes red; make a Tier 2 finding set the gate and confirm the
  second does. Then remove both.
- **The high-scoring fixture is a degenerate-fixture candidate.** If it scores 96
  because no Tier 2 rule is implemented yet — which, in Phase 1, is exactly the
  situation — then the check is asserting nothing about tier separation at all.
  **State plainly what was actually firing when you measured 96.** If the honest
  answer is "nothing, because Phase 2 has not happened", say so and say what the
  check is therefore worth today, and what would strengthen it later.

That last point is the most likely blind instrument in this task and it is
predictable in advance, which is why it is written down before the work starts
rather than found in review.

## Scope

**IN**
- `crates/kicli/src/lint/` — the tier separation and the gate result
- `crates/kicli/src/cli/` — only the `sch score` surface's gate reporting
- new test files under `crates/kicli/tests/`
- `crates/kicli/tests/fixtures/**` — new fixtures only
- this file, for the evidence, written AS YOU WORK

**MERGE HOTSPOTS — report, do not edit.** `Cargo.toml`, `crates/kicli/src/lib.rs`,
the fixture `MANIFEST`, `AGENT.md`, `spec/SPEC.md`, `crates/kicli/tests/command_surface.rs`.
**`command_surface.rs` and `AGENT.md` will both eventually need this command
written down** — Constitution §10, *"a feature undocumented for agents is
unfinished"* — and both are the orchestrator's to schedule. Report what they owe.

**OUT** — every other module, every other entry, `tasks/M5/PLAN.md`.

**If the enumeration above proves wrong, the named goal state and its checks win
over the list.** Say so in your first paragraph, name what you touched and why.

## Completion check

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo xtask check
```

plus both direction checks by name, each shown failing under its own injected
break, and the fixture committed with its measured score recorded in this entry.

---

# AMENDMENT — the two items the checkpoint added, and one of them is a ruling

**Provenance: James's rulings (BLOCKED 1, 2, 3; seam; Q1/Q5) and advisor
rulings, checkpoint 1 review.** Recorded before dispatch, per the orchestrator
definition's session-start rule.

The Phase 1 state table in `PLAN.md` said two items go into this brief when it
is dispatched. Both are below. The first is a **ruling** and it enlarges the
task; the second is a **promoted PROPOSED item** and it moves a function.

## A. BLOCKED 3 is ruled: a saturating rule becomes a BLOCKING finding

**RULED — option 2, verbatim from the checkpoint:**

> a rule that saturates (fires at or above the saturation fraction of the
> objects it counts) becomes a BLOCKING finding, using §11.5's existing tier
> mechanism. No weight moves; no formula changes. T4 builds saturation as a
> first-class rule property every Tier 1 author declares; the saturation
> fraction gets a provisional value recorded as provisional, with Phase 4
> explicitly authorised to move it on measurement. Provenance: the north star's
> own sentence — a drawing that fails the gate is not rewarded whatever it
> scores.

**What it is answering** is `lane-t3`'s measurement, made with the shipped
implementation and reproduced in the checkpoint 1 report:

| Drawing | Raw penalty | Score |
|---|---|---|
| 10 wires, **every wire crosses another** | 10.0 | **67** |
| 200 wires, every wire crosses another | 10.0 | **67** |
| 10 000 wires, every wire crosses another | 10.0 | **67** |

A normalised rule that can fire at most once per object it counts has `n ≤ N`,
so its contribution is capped at `w · reference` **whatever `N` is** — and the
cap does not fall as the drawing grows. A sheet on which every wire crosses
another scores 67 at ten wires and at ten thousand. **67 is not a punishment,
and that sheet is not readable.** The north star forbids exactly this.

### What this adds to T4, precisely

T4 was "Tier 1 does not reduce the score". It is now **that, plus the other
half of the same question: what else makes a drawing fail the gate.** The ruling
puts both on one mechanism, which is why it lands here rather than in a rule.

**1. `Rule::saturation()` — a first-class property, declared, not inferred.**

A rule declares two things the scorer cannot work out for itself:

- **what it counts** — the denominator: wires, non-power symbols, sheets, or
  nothing;
- **the fraction of that denominator at which it saturates.**

Both belong on the rule for the same reason the normaliser does (item B below,
and T1's generated registry before it): **the knowledge is the rule's and the
scorer does not have it.** `KI-DNP-001` is the standing proof — it counts
symbols and is still right at `per_sheet`, because *its own detection already
divides by symbol count*.

**2. The provisional fraction is `1/2`, and it is recorded AS provisional.**

Directed by the orchestrator so a lane does not make a value call the ruling
already reserved for Phase 4. The reasoning, so Phase 4 can argue with it:
at `n = N/2` a normalised rule is already within a factor of two of a ceiling it
can never exceed, so half is the coarsest line that is defensible without a
measurement — and the ruling explicitly authorises Phase 4 to move it.

**Obligation: record what the score would be at the provisional fraction** on
the three measured rows above, so Phase 4 inherits numbers rather than a
threshold with no context. A provisional value with no measured consequence is
indistinguishable from a guess, and this project has an entry about that.

**3. Every rule author declares it, starting with Phase 2's six.**

This is the ruling's stated reason for putting the property in T4 and not later:
*"Building T4 first turns a property into a retrofit across every Tier 1 rule."*
Phase 2's six Tier 1 rules are the first authors after this task, and they will
declare it.

> **NOTE, orchestrator, recorded rather than resolved.** The ruling says *"every
> Tier 1 author declares"*, and **the mechanism only changes an outcome for
> Tier 2**: a Tier 1 finding already fails the gate on its first occurrence, so
> saturating cannot make it block any harder. Two readings are available — the
> property is uniform on the trait and Tier 1 authors declare it so Phase 3
> inherits a trait that already has it (which is what this brief implements), or
> the declaration is meant only where it changes an outcome. **Implement the
> first**; it is the reading that matches the ruling's own stated reason, and
> the difference between them is one default. **Say in the entry which arm your
> implementation would need if the second reading were meant.**

**4. Both new directions are checked, and both are shown failing.**

Beyond the two direction checks this entry already names:

- a Tier 2 rule firing **at or above** its saturation fraction **sets
  `gate: fail`**;
- a Tier 2 rule firing **below** it does **not** — and still scores normally.

The second is the one that matters, in the same way the existing pair matters:
one check over a fixture carrying both passes if the implementation swaps them.

**5. The output shows WHY the gate failed, and saturation is a different why.**

Item 4 of the goal state above governs. A gate failure that reads identically
whether it came from a Tier 1 finding or from a saturating Tier 2 rule sends an
agent to the wrong place. **The saturating case names the rule, the count, the
denominator and the fraction** — `n of N` is the whole explanation and it costs
one line.

## B. PROPOSED 11, promoted: the normaliser moves onto the rule

**`Rule::normaliser()`, with a default, stamped onto the finding by
`Findings::of` exactly as tier, severity and weight already are.**

`Normaliser::of(RuleId)` reads the family out of the rule code, which works only
because §11.5's published table is written in the same vocabulary the codes use.
**It is already wrong for two catalogue rules whose nature disagrees with their
family:**

| Rule | Its own definition | Family gives it | Nature suggests |
|---|---|---|---|
| `KI-LAY-003` | *"W 1 per unaligned **symbol**"* | `per_sheet` | `per_object` |
| `KI-JCT-001` | four-way junction, a **wire** feature | `per_sheet` | `per_wire` |

`KI-LAY-003` un-normalised costs one point per unaligned symbol **with no
ceiling**, so a 200-symbol sheet with every symbol unaligned reaches 200 raw
points and **scores 0**, while every other symbol-shaped rule on that sheet is
divided by ten.

**T3 correctly did not take this** — it changes `rule.rs` and `finding.rs`,
which are T1's files and this task's. T3 pinned current behaviour with
`each_family_takes_the_normaliser_the_catalogue_gives_it` **so the change is
visible when made**; expect that check to need updating and say what you changed
it to and why.

**Do not silently re-family `KI-LAY-003` or `KI-JCT-001`.** Those two rules are
Phase 3's and neither exists yet. Move the *mechanism*; leave the two values as
the family table gives them today, and record that the mechanism now makes the
correction a one-line change in the rule's own file when its author writes it.
Changing a published normaliser is a §11.5 matter and §11.5 is a merge hotspot.

## C. What did not change

Scope, completion check, and the falsification obligation above stand as
written, with the additions in A.4 and A.2. **The degenerate-fixture warning is
now doubly live**: a saturation check on a Phase 1 tree has no Tier 2 rule to
saturate, so say plainly what you constructed and what it is worth today.
