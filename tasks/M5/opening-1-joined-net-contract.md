# The joined net's contract field (opening 1)

**Provenance: James's ruling on BLOCKED 2, M4 close review.** Verbatim:

> BLOCKED 2 ruled per the adjusted-field precedent: the freeze is lifted for
> exactly the joined-net field — `route/report.rs` gains it, spec §8 amended in
> the same commit, freeze restored in that commit — and T18/T19's sibling-key
> surface migrates to the contract field in the same change, their checks
> updated.

The block it rules is recorded in full at `tasks/reports/M4-phase3.md`, "BLOCKED
2 — the joined net has no unfrozen home".

## Which §8

The ruling says "spec §8". **`spec/SPEC.md` §8 is Mutation semantics and is not
the output contract.** The output contract is `research/wire-routing.md` §8, and
that is the document the precedent actually amended — verified, not inferred:
commit `dd4f659`'s message says *"research/wire-routing.md §8 is amended in this
same commit, per the standing rule that the spec and the frozen contract must
not disagree"*, and `crates/kicli/src/route/report.rs`'s own module doc says
*"Every field of `research/wire-routing.md` §8 is here"*.

**So: `research/wire-routing.md` §8 is the document to amend.** Recorded here
rather than resolved silently, because reading a ruling's citation as a slip is
exactly the kind of judgement that must be visible to whoever reviews it.
`spec/SPEC.md` §9 states the wiring contract in summary and gains a sentence
only if it would otherwise contradict §8.

## Why the field belongs in the contract

`kicli wire connect` must report **which net it joined** — T18's own check
requires that the report's claimed net be the extractor's net. Today the CLI
layer attaches it as a top-level JSON key **beside** the route object
(`crates/kicli/src/cli/edit/wire.rs`, `with_net`), and that function's own
rustdoc records why:

> The key is a sibling of the noun's own rather than a field inside the route
> contract: `crate::route::report` is frozen […] What net a connection produced
> is the command's answer, not the router's.

The first clause is the reason being removed by this ruling. **The second clause
is not a problem and has a precedent inside the same file**: `Crossing::net` is
already a contract field the router never fills — the search knows which wire
sits on a cell and never learns net names, and the command layer attributes it
at the seam (`research/wire-routing.md` §8; `spec/SPEC.md` §9, "A reported
crossing names the wire; the net is attributed at the seam"). The joined net is
attributed at the same seam, by the same layer, into the same contract.

The cost of leaving it a sibling is stated in the BLOCKED item: the route
contract would not describe the whole of a route's result, **and M5's linter
reads that contract**.

## Goal state, as the checks that prove it

1. `crates/kicli/src/route/report.rs` carries the field. `Report::of` initialises
   it, and the "nothing that can be worked out from another field is stored"
   rule in that module's doc still holds — the net is read from the written
   file, so it is not derivable from anything else in the report.
2. The renderer prints it in **both** forms from one place:
   `crates/kicli/src/cli/edit/wire/contract.rs`. The module's three stated rules
   govern it — **the JSON carries every key at every status** (null when nothing
   was joined, never absent), and **the text prints a line only when it has
   something to say**.
3. `with_net` and its callers in `crates/kicli/src/cli/edit/wire.rs` are gone,
   not merely bypassed. A second way to emit the key is a second answer waiting
   to disagree with the first.
4. **The text form does not change what an agent reads.** The `joined: net NAME`
   line keeps its wording. Its *position* may move to sit with the rest of the
   route's detail; if it does, say so, because `AGENT.md` shows it.
5. Every check T18 and T19 named still passes, and the ones that assert the
   sibling key are **updated to assert the contract field** — updated, not
   deleted. Name each one in the entry.
6. `research/wire-routing.md` §8 is amended in the same commit: the text form
   and the JSON both gain the field, with the null-when-nothing-joined rule
   stated.
7. `AGENT.md`: any example block showing `wire connect` output is **regenerated
   from a real run of the built binary** (M5 `RULES.md`, "Worked examples in
   `AGENT.md` are measured output"). Do not hand-edit an example to match what
   you believe the code now prints.

## The freeze, and how to move through it

`crates/kicli/src/route/report.rs` is on the frozen surface, enforced by a
PreToolUse hook over `.claude/hooks/frozen-paths.txt`. **Lifting the freeze IS
the ruling path, and this is the ruling.** The procedure is the precedent's:

1. Remove the path from `.claude/hooks/frozen-paths.txt`, recording above it
   what was lifted, for what single change, and by whose ruling.
2. Make the change, including the §8 amendment.
3. **Restore the path in the same commit.** The freeze continues; the next
   change needs its own ruling.
4. All of it is **one commit**. A commit where the freeze is lifted and not
   restored must never exist on the branch.

**Also fix, in the same commit:** the "Lifted once, by advisor ruling
2026-08-15" comment block in that file is **duplicated verbatim** (lines 4–7 and
8–11). One copy stands, and this task's own note is added beneath it. Found by
the orchestrator while deriving this brief; it is four lines and in the file
being edited, so leaving it would be tidier to report than to fix.

## Falsification obligation

Per `.claude/skills/falsification-control/SKILL.md`, and note the shape this
task is exposed to. A check asserting that the reported net **equals** the
extractor's net is a **degenerate-equality** candidate: ask what else would make
the two sides equal. If both are computed by the same call on the same seam, a
break moves them together and the check cannot see it. State in the entry what
the two sides are derived from, and whether they share an ancestor.

## Scope

**IN**
- `crates/kicli/src/route/report.rs`
- `crates/kicli/src/cli/edit/wire/contract.rs`
- `crates/kicli/src/cli/edit/wire.rs`
- `crates/kicli/tests/**` — only the files whose checks assert the joined net
- `research/wire-routing.md` (§8 only)
- `.claude/hooks/frozen-paths.txt`
- `AGENT.md` — only example blocks this change invalidates
- `spec/SPEC.md` §9 — one sentence, only if §8 and §9 would otherwise disagree
- this file

**OUT** — everything else, and in particular `crates/kicli/src/route/**` other
than `report.rs`, every other task's entry, and `tasks/M5/PLAN.md`.

**If the enumeration above proves wrong, the goal state wins over the list.**
Say so in your first paragraph, name what you touched and why. Provenance for
that instruction: PROPOSED 5, M4 close, promoted — a brief that derives scope
from an enumeration must state which wins when the enumeration is wrong, and
this one derives from a `grep` the orchestrator wrote.

## Evidence obligations

- The falsification table, per the skill.
- Which checks were updated and what each asserted before and after.
- The regenerated `AGENT.md` blocks, with the command that produced each.
- The one-commit freeze cycle, shown: `git show --stat` of the commit,
  demonstrating `frozen-paths.txt` net-unchanged-but-for-the-note.

## Completion check

```sh
cargo xtask check
cargo test
```

plus, named explicitly because they are the ones this change can break:

```sh
cargo test --test agent_doc
cargo test --test command_surface
```
