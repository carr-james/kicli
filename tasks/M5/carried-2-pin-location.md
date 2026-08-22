# Carried in from M4 — D2, nothing read-only will tell an agent where a pin is

*Migrated verbatim from the former `tasks/M5.md` at the M5 opening, by the
boundary-package ruling that gives M5 one file per task. The text below is the
record as M4 wrote it; nothing was re-argued in the move.*

**Provenance: the M4 dogfood run, defect 2, ratified in full by advisor ruling
2026-08-15 with the explicit promotion "D2 goes to the M5 planning list as a
task".** Full text in `tasks/dogfood.md`.

**What the dogfood agent actually hit.** It had to infer a pin offset from a
label kicli itself had placed, and learned its guess was wrong only from a
**write command's** output. Defect 6 of the same run is the same wound from the
other side: a wire vertex is refused rather than snapped — a ruled and correct
choice — but there is nothing to ask what *would* be accepted, so a first wire
onto a new symbol is trial and error.

**It is the largest of the nine defects that run found**, and it is a design
decision about the agent-facing surface rather than a patch: a read-only way to
ask where a symbol's pins are.

**The answer already exists internally and is simply unexposed** — the router
resolves pins in `route::terminal`, `Terminal::of_pin`. That is what makes this
cheap to build and expensive to design badly: the question is not how to compute
it but what an agent should be able to ask, and in what shape, under
Constitution §6's context budget.


---

# SCHEDULED — Phase 1, beside the spine

**Provenance: `tasks/M5/PLAN.md`, RATIFIED by James's ratification and advisor
rulings, M5 plan review.** The plan places this in Phase 1 *"as its own task"*.

**It is the one Phase 1 item that may run beside a spine task**, because its
file scope is disjoint from `lint/**` entirely. It does not block T1–T4 and they
do not block it.

## Why this is a design task with an implementation attached

The M4 record above says it: *"the question is not how to compute it but what an
agent should be able to ask, and in what shape."* `Terminal::of_pin` already
knows the answer. **Everything hard about this task is the surface.**

So the deliverable is not "a command that prints pin positions". It is **an
answer to a question an agent has, in a form an agent can act on** — and the
next thing that agent does with the answer is draw a wire to it.

## The two defects it must close, and they are one wound from two sides

- **D2**: an agent had to *infer* a pin offset from a label kicli itself had
  placed, and learned its guess was wrong only from a **write command's**
  output. Read-only questions should not be answered by write commands.
- **D6**: a wire vertex is **refused rather than snapped** — a ruled and correct
  choice — but **there is nothing to ask what *would* be accepted**, so a first
  wire onto a new symbol is trial and error.

**D6 is the sharper of the two and the easier to under-serve.** A command that
answers "where is pin 1" without answering "what may I connect to it" closes D2
and leaves D6 open, and the agent still guesses — one round-trip later than
before. Whatever you build, **check it against D6 explicitly** and say in the
entry whether it closes it.

## Constitution §6 governs the shape, and it has teeth here

*"Outputs are designed for LLM context budgets. A view that floods is wrong,
whatever it contains."* A 40-pin connector's every pin, printed by default,
is a flood. So the shape question includes **what you get without asking for
everything**, and `spec/SPEC.md` §7.4's budgets are the existing precedent —
`crates/kicli/tests/view_budgets.rs` is where budgets are already asserted.

Note that file currently carries two **dead** helper functions
(`connectivity_ceiling`, `layout_ceiling`, flagged by `cargo` as never used).
That is pre-existing and **not yours to clean** — report it, do not fix it.

## Where the answer already lives

`crates/kicli/src/route/terminal.rs`, `Terminal::of_pin`. `route` knows nothing
of files, the CLI or `kicad-cli` (`ENGINEERING.md`, Structure), and **it must
stay that way** — this task exposes what `route` computes; it does not move the
CLI into `route`.

`crates/kicli/tests/pin_positions.rs` and `route_terminals.rs` already measure
pin resolution. **Read both before adding a check**: whatever you build stands
on the same resolution they already exercise, and a new check that re-asserts
what they assert is a check with a shared ancestor.

## Goal state, as the checks that prove it

1. **A read-only command answers where a symbol's pins are**, in both the terse
   text and JSON forms, per the project's established view conventions.
   **It writes nothing** — and that is worth an executable check, not a comment.
2. **The answer is sufficient to draw a wire without a failed attempt.** The
   check is end-to-end and is the task's real completion criterion: ask where the
   pin is, use the answer to place a wire, and **the wire is accepted first
   time**. A check that only compares numbers to a fixture does not test the
   thing the defect was about.
3. **It does not flood.** A budget assertion in the shape `view_budgets.rs`
   already uses, over a realistically large symbol.
4. **`AGENT.md` documents it, with a worked example regenerated from a real run
   of the built binary** — Constitution §10 (*a feature undocumented for agents
   is unfinished*) and M5 `RULES.md`'s measured-examples rule. `AGENT.md` is a
   merge hotspot **held by one lane at a time**; the orchestrator schedules it.

## Falsification obligation

Per `.claude/skills/falsification-control/SKILL.md`.

- **Goal-state 2 is the one that can be blind.** If the wire placement in the
  check derives its coordinates from the same call the command uses, the two
  sides share an ancestor and the check passes on a scorer that is uniformly
  wrong. **State what each side derives from.** The strong form goes through the
  *printed output*, parsed back, which is what an agent actually does.
- **The writes-nothing check is falsified by making it write**: touch the file
  in the command path and confirm red.

## Scope

**IN**
- a new read-only view/command surface, in the modules its shape requires
- `crates/kicli/src/route/terminal.rs` — **read; change only if the exposure
  genuinely requires it, and say so**
- new test files under `crates/kicli/tests/`
- this file, for the evidence, written AS YOU WORK

**MERGE HOTSPOTS — report, do not edit.** `Cargo.toml`, `crates/kicli/src/lib.rs`,
the fixture `MANIFEST`, `AGENT.md`, `spec/SPEC.md`, `crates/kicli/tests/command_surface.rs`.
`AGENT.md` and `command_surface.rs` both owe this command a line; **report what
they owe** and the orchestrator sequences it.

**OUT** — `crates/kicli/src/lint/**` (Phase 1's spine lanes own it), every other
entry, `tasks/M5/PLAN.md`, the two dead helpers in `view_budgets.rs`.

**If the enumeration above proves wrong, the named goal state and its checks win
over the list.** Say so in your first paragraph, name what you touched and why.

## The decision this task must NOT make alone

**What the command is called and what it answers by default is a surface
decision.** If it feels like a value call — and D6 makes it one, because
"what may I connect to" is a different question from "where is it" — **park it
as PROPOSED** with the options and a recommendation, per `RULES.md`'s north
star rule. It is cheap to reverse a name now and expensive after `AGENT.md`
ships it.

## Completion check

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo xtask check
cargo test --test agent_doc
cargo test --test command_surface
```

plus the end-to-end check of goal-state 2 by name, with its falsification
recorded.
