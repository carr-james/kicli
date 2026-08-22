# The `sch score` command surface (Phase 2, orchestrator-held)

**Filed by the orchestrator at checkpoint 2, from `lane-t4`'s report.**
**Provenance: measured — `lane-t4`, WORKFLOW NOTE, verbatim:**

> *"the brief named 'the `sch score` surface's gate reporting' as in-scope while
> `AGENT.md` and `command_surface.rs` are hotspots, and those three cannot be
> split without a red lane branch — so no CLI surface could be built at all, and
> the brief should have said so rather than leaving the lane to derive it."*

## The gap, and it is in the PLAN rather than in any lane

**`sch score` is M5's deliverable command and `PLAN.md` schedules it nowhere.**
The plan's Goal of M5 opens with it — *"`sch score` runs the lint engine,
consumes ERC where ERC owns the check, and reports findings in §11.3's format
with a score per §11.5"* — and every phase in the plan builds a *part* of it.
Phase 1 built the engine's spine, Phase 2 builds the rules, Phase 4 calibrates.
**No task builds the command.**

It went unnoticed because each individual brief could truthfully say *"the CLI
surface is a merge hotspot, report what it owes"*, and every lane did report it.
**A hotspot that every lane defers and no task owns is a hotspot that never gets
built.** `lane-t4` is where that became visible, because it was the first task
whose goal state genuinely needed the command to exist.

**This is a real instance of the rule promoted at this checkpoint** — *a brief's
completion check and its scope must be satisfiable together* — arriving from the
third direction: not check-forbids-scope, not scope-forbids-check, but **a
deliverable that every scope defers.**

## Why it is orchestrator-held

`AGENT.md` is **held by one lane at a time** and `tests/command_surface.rs` is a
merge hotspot. Both must move together with the command: `agent_doc_covers_every_command`
**fails the moment a verb exists undocumented**, so a lane that adds the verb and
not the document leaves a red branch, and a lane that adds neither cannot build
the surface. **The three are one atomic change**, which is precisely why they
belong to one holder.

## Sequencing — and this task is NOT dispatched at checkpoint 2

It needs the rules to exist, or it ships a command that scores nothing and the
worked examples in `AGENT.md` are measured from an empty run. Per the
measured-examples rule (`RULES.md`), **every example block a session touches is
regenerated from a real run of the built binary** — so a surface built before the
rules produces documentation that is correct and worthless, and will be
regenerated anyway.

**Dispatch at the end of Phase 2, after the six rules merge, as a single lane
holding `AGENT.md`.** Recorded here so the gap cannot close by being forgotten a
second time.

## Goal state, as the checks that prove it

### 1. The command exists and the four surfaces agree

`sch score` and `sch score --gate`, present in: the CLI parser,
`tests/command_surface.rs`, `AGENT.md`, and the exit-code table. **The existing
checks already enforce the agreement** — `agent_doc_covers_every_command`,
`agent_doc_covers_every_verb_flag`, `every_verb_parses`,
`every_exit_code_has_exactly_one_name`. **Read them before writing; they define
what "on the surface" means here and they will fail loudly if a fifth surface is
missed.**

### 2. Exit codes, per §6.1

- `sch score` with findings, no `--gate` → **exit 0**. *"Findings are data, not
  failure."*
- `sch score --gate` with a Tier 1 or saturating finding → **exit 5**
- `sch score --gate` with `kicad-cli` absent → **exit 6**, structured, naming the
  binary and an install hint (§14.1)

`crates/kicli/src/cli/exit.rs` owns the table and T4 added `ExitCode::for_gate`.
**Reuse it. Do not build a second table.**

### 3. Constitution §6 governs the view, and this is the command that floods

Findings are read by an **LLM agent under a context budget**. A view that floods
is wrong whatever it contains. `KI-DOC-001` alone can fire once per component.
The project already has `view_budgets.rs` — **this command gets a budget check
like every other view, and the fallback-to-summary behaviour the delta view
already models is the precedent to follow rather than reinvent.**

### 4. The score and the gate are visible in ONE output

T4's fixture `crates/kicli/tests/fixtures/sch/score/high_and_blocked.kicad_sch`
scores high and fails the gate. **That is this command's acceptance case**, and
T4 already measured what it prints. **An agent that must run two commands to
learn its build is broken will run one.**

### 5. Every `AGENT.md` block is regenerated from a real run

Not edited by hand. `opening-3`'s check enforces it, and **it caught a live
defect on the run that introduced it.** Paste the producing commands.

## Falsification obligation

- **Each exit code shown**, and 5 distinguished from 6 — they are different rows
  of §6.2's table and the same in a careless test.
- **The budget check shown failing** on a sheet that floods.
- **The `--gate`-without-`kicad-cli` case must not report `gate: pass`.** See
  `phase2-ki-hier-001-erc-delegation.md` goal state 3: a gate that reports pass
  on a check that never ran is the worst output this milestone can produce.

## Scope

**IN** — the CLI verb and its view; `AGENT.md` (**this lane holds it**);
`tests/command_surface.rs`; new tests and fixtures with their `MANIFEST` lines;
this file.

**OUT** — every rule file, `crates/kicli/src/lint/**` beyond what the command
calls, every other entry.

## Completion check

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo xtask check
cargo test -p kicli --test agent_doc
cargo test -p kicli --test command_surface
cargo test -p kicli --test view_budgets
```
