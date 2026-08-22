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

## The freeze, and who moves through it — RE-WORDED, and the owner is named

`crates/kicli/src/route/report.rs` is on the frozen surface, enforced by a
PreToolUse hook over `.claude/hooks/frozen-paths.txt`. **Lifting the freeze IS
the ruling path, and this is the ruling.**

**The lift is the ORCHESTRATOR's step, in the MAIN CHECKOUT.** Provenance:
James's ratification and advisor rulings, M5 plan review, question 2, ruling on
BLOCKED 1 below. Verbatim: *"the freeze lift is the ORCHESTRATOR's step, in the
main checkout — lift before dispatch, restore after merge, both committed with
the ruling's provenance."*

The rejected alternative and its reason are recorded because they are the
general lesson: teaching the hook to resolve its list from the **edited file's
own worktree** would let a lane lift its own freeze inside its own world. The
main-checkout resolution is **privilege separation**, and it is now deliberate.
`lane-o1`'s measurement was right in every particular; what it found was a
property rather than a bug.

### The procedure, as it is now performed

**Orchestrator, main checkout, before dispatch:**

1. Remove the path from `.claude/hooks/frozen-paths.txt`, recording above it
   what was lifted, for what single change, and by whose ruling. **Dedupe the
   "Lifted once, by advisor ruling 2026-08-15" comment block while there** — it
   stands twice, verbatim, at lines 4–7 and 8–11; one copy stands. Found by the
   orchestrator while deriving this brief, and it is in the file the
   orchestrator now owns for this cycle.
2. **Commit the lift**, with this ruling's provenance in the message.

**Lane, in its pinned worktree:**

3. Make the change, including the `research/wire-routing.md` §8 amendment.
   **The lane does not touch `.claude/hooks/frozen-paths.txt` at all** — it is
   out of the lane's scope, and the file the hook reads is not the lane's copy.

**Orchestrator, main checkout, after the merge:**

4. **Restore the path**, and commit, with the same provenance. The freeze
   continues; the next change needs its own ruling.

### The rule this supersedes, recorded rather than absorbed

The M4-close ruling that created this task said: *"Restore the path in the same
commit […] All of it is one commit. A commit where the freeze is lifted and not
restored must never exist on the branch."* **The new ruling necessarily creates
exactly that commit**, because the lift and the change are now performed by
different actors in different trees.

Later ruling, same author, aimed specifically at this mechanism — so it governs,
and the earlier words are superseded **on this one point** rather than in
general. This is recorded, not resolved silently, and it is flagged to James in
the session report: a superseded rule that nobody noticed being superseded is
how a rule quietly stops meaning anything.

**The window is narrowed to what it must be**: the lift commit touches
`frozen-paths.txt` and nothing else, names the single file and the single
change, and the restore is the first commit after the merge.

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

**OUT, and newly so:** `.claude/hooks/frozen-paths.txt`. It was on the IN list
when this entry expected the lane to lift its own freeze. Under question 2's
ruling the lift is the orchestrator's, in the main checkout, and **a lane
touching that file would be editing a copy no hook ever reads** — which is the
precise thing `lane-o1` measured.

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

---

## BLOCKED 1 — the freeze cannot be lifted from a lane worktree — **RULED, and the finding stands** ✅

**Ruled at the M5 plan review, question 2: the lane's fallback option 2 is
taken.** The lift moves to the orchestrator and the main checkout; the hook is
unchanged. The preferred option 1 is rejected on privilege separation — see the
re-worded procedure above. **The measurement below was correct in every
particular and is not amended**; the last line of its PROPOSED section — *"the
entry's procedure needs its owner named, because as written it names a step the
addressee cannot take"* — is what the ruling did.

**Recorded by lane `lane-o1`, at the moment of the claim, against base commit
`a1e25ad`. No code was written; the task is not started.**

The entry's freeze procedure step 1 — "remove the path from
`.claude/hooks/frozen-paths.txt`" — was performed in the lane worktree
`.claude/worktrees/lane-o1`, with the duplicated 2026-08-15 comment block
deduped and this task's own lift note added beneath it. `Edit` on
`crates/kicli/src/route/report.rs` was then **still refused** by the PreToolUse
hook:

```
PreToolUse:Edit hook error: [.claude/hooks/frozen-surface.sh]:
Frozen surface: crates/kicli/src/route/report.rs — changes require a ruling;
park as BLOCKED.
```

### The measurement, not the inference

`.claude/hooks/frozen-surface.sh` resolves its list relative to the script:

```sh
list="$(dirname "$0")/frozen-paths.txt"
```

`.claude/settings.json` registers the hook by the **relative** command
`.claude/hooks/frozen-surface.sh`, which resolves against the session's project
root — the **main checkout**, `/Users/james/code/kicli` — and never against the
worktree the lane was briefed into. So `$0` is the main checkout's script and
the list read is the main checkout's list.

Both copies of the script were run by hand on the same input, from the lane
worktree, with the lane's list lifted:

| script run | list it read | exit |
|---|---|---|
| `bash .claude/hooks/frozen-surface.sh` (worktree copy) | worktree's, freeze lifted | **0** — allowed |
| `bash /Users/james/code/kicli/.claude/hooks/frozen-surface.sh` (main copy) | main checkout's, freeze intact | **2** — blocked |

Same script text, same stdin, opposite answers. The variable is which
`frozen-paths.txt` `dirname "$0"` names, and it is always the main checkout's.

### Why this is BLOCKED rather than worked around

Three governing statements meet here and the lane may not pick between them:

- `CLAUDE.md`, the agentic layer: "Lifting a freeze IS the ruling path; the
  orchestrator is not exempt." The ruling path is an edit to a
  version-controlled file — and the file version control holds for this branch
  is the worktree's.
- `CLAUDE.md`, parallel work: the lane's pinned worktree "is your whole world";
  a lane does not write to the main checkout. The only file that would actually
  move the hook is outside the lane's world.
- The lane's brief: "Do not work around the hook by any other route; if the
  hook still blocks you after the path is removed, stop and report it, because
  that is a finding about the hook."

The consequence is general and not about this task: **no lane can execute a
freeze lift, and the lift a lane commits is invisible to the hook that enforces
it.** The version-controlled record and the enforced mechanism read two
different files. This is the shape `CLAUDE.md` already has a precedent for —
"a rule that cannot be performed reads like a rule that is being ignored, and
the fix is sometimes a different mechanism rather than a weaker rule".

### PROPOSED — resolve the mechanism, not this task

Recommendation, for a ruling; the lane does not choose:

1. **Preferred: make the hook read the list that belongs to the file it is
   judging.** Derive the list from the edited path's own worktree —
   `git -C "$(dirname "$path")" rev-parse --show-toplevel` — and read
   `<that root>/.claude/hooks/frozen-paths.txt`. The freeze then travels with
   the branch, the lift is enforced where it is committed, and a lane can
   perform the ruling path it is briefed to perform. This also removes the
   present oddity that the freeze only catches a worktree file at all through
   the hook's absolute-suffix arm rather than through its repo-relative one.
2. **Fallback, if the mechanism stands as it is:** the freeze lift is the
   **orchestrator's** step, not the lane's — the orchestrator lifts in the main
   checkout before dispatch and restores after merge, and the lane's brief says
   so instead of instructing the lane to lift. This keeps the hook simple and
   makes the entry's four-step procedure executable by someone.

Either way the entry's "Lifting the freeze IS the ruling path" procedure needs
its owner named, because as written it names a step the addressee cannot take.

### State left behind

`.claude/hooks/frozen-paths.txt` was **restored to its committed content** and
verified identical to the main checkout's copy (`md5` `8bd8f80c…` on both). The
worktree carries no code change and no lift: `git status --porcelain` is empty
but for this entry. **The duplicated comment block at lines 8–11 therefore
still stands** — it was ruled into this task's single commit, and that commit
does not exist.

## Reconnaissance, banked so the re-dispatch is cheap

**This is reading, not work.** Every line below was read out of the tree at
`a1e25ad` and **none of it has been falsified**. Per `CLAUDE.md`, a line adopted
from here passes the falsification discipline as if newly written; it carries no
evidence standing whatever it asserts.

- **Where the sibling key is asserted.** Three call sites, all through two
  helpers, and both helpers read a **top-level** key:
  - `crates/kicli/tests/edit_wire_connect.rs:87`, `Connected::claimed_net` —
    `self.object()["net"].as_str()`. Used at lines 349, 582 and 786.
  - `crates/kicli/tests/wire_loop.rs:424` and `:449` — `pins["net"]` and
    `to_net["net"]`, both reached directly rather than through that file's
    `wire()` helper at `:247`.
  - Nothing else in `crates/kicli/tests/**` reads a top-level `net`; the only
    other `["net"]` reads are `crossings[].net` inside `contract.rs`.
- **`claimed_net` is degenerate on the null arm as written.** `.as_str()`
  answers `None` for a null value **and** for an absent key, so
  `edit_wire_connect.rs:582`'s "nothing was written, so nothing was joined"
  assertion cannot tell the contract's every-key-at-every-status rule from a
  dropped key. Whoever moves it should assert presence and null separately —
  that is a check the migration can strengthen rather than merely relocate.
- **No check asserts the text line.** `grep -n "joined: net"` over `crates/`
  finds only the producer, `cli/edit/wire.rs:260`. The three readers are all in
  `AGENT.md` (lines 605, 652, 675). So goal-state item 4's "position may move"
  is unconstrained by any test, and `AGENT.md` is the only thing that would
  notice — which is exactly why `RULES.md` says regenerate rather than hand-edit.
- **A position that changes nothing is available.** `cli/edit/wire.rs:260`
  prepends `joined: net {name}\n` to the whole result text, ahead of
  `contract::render`'s own first line. If `contract::text` emits the same line
  first, the bytes are unchanged. Worth weighing against putting it in
  `detail()`, which would indent it two spaces and change what `AGENT.md` shows.
- **`wire draw` gains a null key it does not have today.** `with_net` is called
  only from `connect_wire`'s `joined()` and from `unwritten()`, so no `wire
  draw` answer carries a top-level `net` at all. Moving the field into the route
  contract puts `"net": null` into every `wire draw` answer as well, because the
  renderer is shared and its rule is every key at every status. That invalidates
  a second `AGENT.md` block — the `wire draw` JSON example at lines 570–579 —
  which is in scope as "an example block this change invalidates".
- **`command_surface.rs:403` is the guard that the text line stays off `wire
  draw`**: it asserts the printed output `starts_with("routed R1.1 -> R2.1 …")`.
  It is not a check to update; it is a check that will catch the mistake.
- **The write ordering moves.** The net is read back off disk, so
  `Report::joined_net` must be filled **before** `contract::render`, whereas
  today `joined()` runs after it. `connect_wire` has two such paths: the
  `draw_plan` path at `wire.rs:167–173`, and the `--auto-labels` path at
  `:163`, where the render happens **inside** `perform()` — and `perform()` is
  shared with `draw_wire`, which must keep reporting nothing. That shared
  renderer is the one real design decision in this task.
- **`spec/SPEC.md` §9 looks not to need its sentence.** §9 states the crossing
  seam ("A reported crossing names the wire; the net is attributed at the seam")
  and says nothing whatever about the net a connection joined, so §8 gaining the
  field contradicts nothing there. Silence is not contradiction — but this was
  read, not tested, and the entry's condition is "only if they would otherwise
  disagree".
- **Oracle standing.** This change is report-only: it moves where a net name is
  printed and does not touch `crate::connectivity` or any write path. The
  connectivity claims around it are already oracle-gated in
  `edit_wire_connect.rs` (`oracle(...)` arms at lines 412, 454, 493 and 791) and
  keep running unchanged. Environment-gated, so it is the orchestrator's merged
  run that counts.
