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
