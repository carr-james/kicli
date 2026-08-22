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

---

## Implementation record — lane `lane-o1b`, base `d4c0eb8`

**Base verified as the lane's first action.** `git log --oneline -1` answered
`d4c0eb8 freeze: lifted for exactly the joined-net contract field (M5
opening-1)` and `git status --porcelain` was empty. No fast-forward was needed.
The freeze lift was already in the main checkout, and `.claude/hooks/
frozen-paths.txt` was **not touched by this lane**: `git diff --stat` names no
such path. The hook did not refuse a single edit to `report.rs`, which is the
other half of `BLOCKED 1`'s finding measured from the far side — with the main
checkout's list lifted, the main checkout's script allows.

### The field, and what it is called

`Report::joined_net: Option<String>`, rendered as the JSON key `"joined_net"`
and as the text line `joined: net NAME`.

**PROPOSED — the key is `joined_net` and not `net`.** The sibling key it
replaces was called `net`, so keeping that name would have been the smaller
diff. Three reasons for the rename, and a ruling may reverse it:

1. Every other field of this contract maps 1:1 onto its JSON key
   (`blocked_by`, `alternatives_considered`, `length_mm`). Naming the Rust
   field `joined_net` and the key `net` would be the only split in the module,
   and `contract.rs`'s third stated rule is that the two forms use one
   vocabulary.
2. `net` already means something else one level down — `crossings[].net` — and
   the two are different questions. `joined_net` cannot be misread for it.
3. The move breaks every existing reader anyway (the key changes container),
   so the rename costs a reader nothing extra and buys the disambiguation.

### The shared renderer — the one real design decision, and what was chosen

`perform()` renders inside itself and is shared by `connect_wire`'s
`--auto-labels` arm and `draw_wire`'s. The net is read back **off disk after the
commit**, so it cannot be computed by the caller before the call.

**Chosen: `perform` gained a parameter, `joins: Option<[&End; 2]>` — "the two
ends whose net to report".** `connect_wire` passes `Some([&request.from, &far])`;
`draw_wire` passes `None`. `perform` fills `route.joined_net` after its commit
and before its render.

Rejected, with reasons:

- **Return the unrendered `Report` and render in each caller.** It would put
  the mutation report, the note and the text splice in two places, which is the
  second-answer-waiting-to-disagree shape the contract module exists to avoid.
- **A closure `&dyn Fn(&Path) -> Option<String>`.** Same effect, more
  machinery, and it hides which ends are being asked about at the call site.

**`wire draw` therefore reports `joined_net: null` always, and this is a
contract statement rather than an omission.** `wire draw` takes the corners it
was given; it is not asked to join two ends and does not answer for what it
happened to connect. Recorded in `research/wire-routing.md` §8, in `AGENT.md`,
and in the field's own rustdoc.

### Goal-state item 4 — the text form, MEASURED rather than argued

**The position did not move, and no `AGENT.md` text block needed regenerating.**
This was measured, not read. The pre-change binary was built from `HEAD`
(`d4c0eb8`) into a scratch directory by the `git archive` recipe in
`falsification-control`, and both binaries were run against byte-identical
copies of the same probe drawing:

| arm | command (both binaries) | result |
|---|---|---|
| `wire connect`, routed | `kicli -q wire connect --from-pin R1.1 --to-pin R2.1 -p <copy>` | **byte-identical**, `shasum 86562b5ec652b05434fe6af17080f41be28600f2` on both |
| `wire connect`, proposal | same, with `label_threshold = "1G"` | byte-identical |
| `wire connect --auto-labels` | same, plus `--auto-labels` | byte-identical (leads `joined: net R1_1`) |
| `wire draw` | `kicli -q wire draw --from-pin R1.1 --to-pin R2.1 --via 50.8,45.72 --via 76.2,45.72 -p <copy>` | byte-identical |
| `wire draw --auto-labels` | same, plus `--auto-labels` | byte-identical |

The JSON diffs over the same runs are exactly the intended change and nothing
else — `wire connect`: `-  "net": "SIG_A"` at the top level, `+ "joined_net":
"SIG_A"` inside `"wire"`; `wire draw` and the proposal arm: `+ "joined_net":
null` inside `"wire"` (and `- "net": null` at the top level for the proposal).

### A property the change EXPOSED, which is a finding

`wire_output_contract.rs`'s `the_status_word_starts_the_first_line_of_every_form`
claimed the status word starts the first line of every form. **That claim was
already false for `wire connect` and the check could not see it**: the joined
line was prepended by the command layer, above the whole result, and the
renderer — which is all that test drives — never produced it. Now that the line
is a contract field the exception is measurable, so it is stated and measured:
`only_the_joined_net_may_come_before_the_status_word` asserts the first line is
exactly `joined: net SIG_A`, that the status word starts the very next line, and
that the same report with `joined_net = None` leads with the status word and is
exactly one line shorter.

This is the `author's vocabulary` kind from `falsification-control`: an
instrument blind in the dimension its author was not driving.

### `command_surface.rs:403` — confirmed, not changed

`printed.starts_with("routed R1.1 -> R2.1   via 3 segments, 2 corners,
35.56mm\n")` still passes: `wire draw` reports no net, so no line is printed.
It was left exactly as it was, and it is the guard that catches the mistake of
reporting a net from the shared renderer for the wrong verb.
`route_labels.rs:396`'s `starts_with("labels U1.1 -> U2.2\n")` is the same guard
on the `wire draw --auto-labels` arm, and is also unchanged and passing.

### Checks migrated, with what each asserted before and after

| check | before | after |
|---|---|---|
| `edit_wire_connect.rs` `Connected::claimed_net` (helper for three assertions) | `self.object()["net"].as_str()` — the top-level sibling key | asserts `wire()["joined_net"]` is **present**, then reads it. Presence and value are separate, because `as_str()` answers `None` for a null and for an absent key alike |
| `edit_wire_connect.rs` `a_route_joins_the_two_pins_it_names` | `claimed_net() == Some(found)` where `found = net_name_of(sheet,"R1","1")` | unchanged in form; the shared-ancestor analysis is now written into the test beside it (below) |
| `edit_wire_connect.rs` `a_connection_over_the_threshold_is_proposed_and_not_drawn` | `claimed_net() == None` — could not tell null from absent | `claimed_net() == None` **through the presence assertion**, plus `wire()["joined_net"] == Value::Null` |
| `edit_wire_connect.rs` `connecting_to_a_net_takes_the_nearest_terminal` (line ~786) | `claimed_net() == Some("SIG")` | same literal, through the migrated helper |
| `wire_loop.rs` step 2 | `pins["net"].as_str() == Some(named)` — top-level | `joined_net(&pins) == Some(named)`, a new helper that asserts key presence first |
| `wire_loop.rs` step 3 | `to_net["net"].as_str() == Some("SIG")` | `joined_net(&to_net) == Some("SIG")` |
| `wire_output_contract.rs` `KEYS` | 15 keys, no joined net | 16 keys, `"joined_net"` among them. The list is written out rather than derived, so a dropped key fails at every status |
| the five `wire_contract_*.golden` files | no joined net | each gains `"joined_net": null` |
| `wire_output_contract.rs` `the_status_word_starts_the_first_line_of_every_form` | unchanged | unchanged; joined by `only_the_joined_net_may_come_before_the_status_word` |

Added, not migrated: `contract.rs`'s two renderer tests, `report.rs`'s
no-wire-no-net assertion, `command_surface.rs`'s `wire draw` presence-and-null
pair, and `wire_contract_routed_joined.golden` — because a field seen only as
`null` in every golden has never been seen rendered, which is the reason that
file's own module doc gives for the moved-terminal golden.

### The degenerate-equality question, answered

`a_route_joins_the_two_pins_it_names` compares the reported net with the
extractor's. **The two sides share an ancestor.** The command computes its
answer with `Hierarchy::load` + `connectivity::extract` + `net_of` in its own
process; `net_name_of` computes the control with `Hierarchy::load` +
`connectivity::extract` + `net_of` in the test's process. A break inside the
extractor moves both and the equality sees nothing.

**What stops that mattering is the literal on the line after it**:
`assert_eq!(found, "SIG_A")`. `SIG_A` is not the extractor's answer — it is the
text of the label `two_resistors_one_named` writes into the drawing — so it
fails on exactly the breaks the equality cannot see. The analysis is now written
into the test as a comment, so the next reader does not have to redo it.

The second do-nothing arm the skill asks for is already in the file and across
it: `connecting_to_a_net_takes_the_nearest_terminal` asserts `Some("SIG")` and `wire_loop.rs`
step 3 asserts `Some("SIG")` against a different drawing, so a `joined_net` that
returned a constant fails one of the three.

### `AGENT.md`, regenerated and measured

Two blocks were invalidated; the command that produced each is recorded.

1. **The `wire draw` JSON example** (the `{ "status": "routed", "from": "R1.1"
   … }` block). Regenerated from:

   ```sh
   kicli -q --output json wire draw --from-pin R1.1 --to-pin R2.1 \
     --via 50.8,45.72 --via 76.2,45.72 -p <copy of the two-resistor drawing>
   ```

   The block was then **verified mechanically** against that run: parsed, the
   elided `added.wires` substituted, and compared — `keys equal: True`,
   `values equal: True`. The block's only change is the added
   `"joined_net": null`.

2. **The prose above the `joined: net #n5` sample**, which said the key was "a
   top-level `"net"` key beside `"wire"`". Rewritten to name `"joined_net"`
   inside `"wire"`, with the three null cases stated.

**The two worked `wire connect` blocks (`joined: net #n1`, R30/R31) were NOT
regenerated, and that is a measurement rather than an omission.** Their drawing
is not reproducible from anything in the repository — the gap
`tasks/M5/chore-8-agent-example-recipe.md` records. It did not have to be: the
text form was proved byte-identical across the change on five arms (above), so
nothing those blocks show has moved. **Reported as live evidence for chore-8**:
the only reason this task could regenerate the `wire draw` block at all is that
its drawing was recoverable by hand from `command_surface.rs`'s fixture, and the
recipe had to be reconstructed by reading test source. The recipe used, in full,
is the probe drawing `R1 @ (50.8, 54.61)`, `R2 @ (76.2, 54.61)`, both `R` with
pins `1` and `2` — which reproduces the committed example's three wire handles
`116592a7`, `7239e219`, `723e5aa0` exactly, so the reconstruction is confirmed
rather than assumed.

### `spec/SPEC.md` §9 — deliberately NOT amended

**PROPOSED.** The entry's condition is "one sentence, only if §8 and §9 would
otherwise disagree". Read in full: §9 states the algorithm, the obstacles, the
cost model, the four-way rule, the label fallback, the status set and the
crossing seam. **It says nothing whatever about the net a connection joined**,
so §8 gaining the field contradicts no sentence there, and the condition is not
met. Recorded rather than acted on, with the argument for the other reading so a
ruling can take it: §9's opening sentence — *"returns the route plus a cost
breakdown"* — could be read as an enumeration that is now short by one, and §9
already carries one sentence about a caller-attributed field (the crossing net),
so a reader could infer from §9 that the crossing net is the only one. One
sentence beside that paragraph would close it.

### Oracle standing — verified, not inherited

This change is report-only: it moves where a net name is printed and touches no
write path and nothing in `crate::connectivity`. `git diff --stat` names no file
under `crates/kicli/src/connectivity/` or `crates/kicli/src/edit/`. The oracle
arms that already gate these claims are unchanged and still present —
`oracle(...)` at `edit_wire_connect.rs` lines 412, 454, 493 and 791 as banked,
re-measured after the edit as lines 433, 475, 514 and 822. Environment-gated, so
the orchestrator's merged run is what counts.

### Reconnaissance, checked against the tree

The banked reconnaissance was read as reference and every line relied on was
re-measured at `d4c0eb8`. All of it held, with one correction:

- **Correct**: the three `claimed_net` call sites; `wire_loop.rs:424`/`:449`;
  no check asserts the text line (only `AGENT.md` reads it); `command_surface.rs`
  is the `wire draw` guard; the write ordering; `spec/SPEC.md` §9's silence.
- **Correct, and now strengthened rather than relocated**: `claimed_net` was
  degenerate on the null arm.
- **Incomplete**: it did not name `wire_output_contract.rs`, which asserts the
  contract's key set against a literal list and holds five goldens. That file
  did not assert the joined net before this change and does now, so it was in
  scope under "files whose checks assert the joined net" — it just could not be
  found by grepping for the old key.

### Falsification table

Good state committed before the first break, per `falsification-control` rule 1.
Anchored to content hashes, not SHAs: `crates/kicli/src/cli/edit/wire/contract.rs`
`8aac8d457bd9a46d3192a06d3b204c7d4130661a`,
`crates/kicli/src/cli/edit/wire.rs` `78f744c3c3b6d08f5de7aa3237338b7f1260a5a2`,
`crates/kicli/tests/edit_wire_connect.rs`
`7d5858a2d454fd80815b9be669a35ac95913f6ca`. Every restore was checksummed
against those before the next break; `cargo xtask check` is green on the
restored tree.

Run with `cargo test --no-fail-fast` — without it cargo stops after the first
failing target and the table under-counts.

| # | what was broken, exactly | what caught it |
|---|---|---|
| B1 | `contract.rs::to_json`: the whole line `"joined_net": report.joined_net,` deleted, so the key never appears | **15**, over five binaries: `every_status_answers_with_the_same_key_set`; all six golden checks; `contract.rs`'s two renderer tests; `a_route_joins_the_two_pins_it_names`, `a_connection_over_the_threshold_is_proposed_and_not_drawn`, `a_performed_proposal_reports_the_net_its_labels_made`, `connecting_to_a_net_takes_the_nearest_terminal`; `an_agent_wires_a_sheet_and_kicad_agrees`; `a_drawn_wire_answers_in_the_contract_and_in_the_mutation_result` |
| B2 | same line, value replaced by `serde_json::Value::Null` — **the do-nothing arm**: key present, never a name | **6**: `a_joined_net_matches_the_golden`; `contract.rs::a_joined_net_leads_the_text_and_names_itself_in_json`; `a_route_joins_the_two_pins_it_names`; `a_performed_proposal_reports_the_net_its_labels_made`; `connecting_to_a_net_takes_the_nearest_terminal`; `an_agent_wires_a_sheet_and_kicad_agrees`. **The presence assertions do not fire**, which is why presence and value are asserted separately rather than through one `as_str()` |
| B3 | two changes together in `contract.rs`: `to_json`'s value → the constant `Some("SIG_A".to_owned())`, and `text`'s conditional `writeln!` → an unconditional `writeln!(out, "joined: net SIG_A")` | **15**, including `only_the_joined_net_may_come_before_the_status_word`, `the_status_word_starts_the_first_line_of_every_form`, `an_empty_report_still_names_both_ends`, `auto_labels_writes_the_pair_and_says_so`, `command_surface`'s presence-and-null pair, and every null golden. **See the row below it: two checks stayed green and that is the finding** |
| B4 | `contract.rs::text`: the `if let Some(net) … writeln!` block deleted; JSON untouched | **3**: `a_joined_net_matches_the_golden`, `contract.rs::a_joined_net_leads_the_text_and_names_itself_in_json`, `only_the_joined_net_may_come_before_the_status_word` |
| B5 | `contract.rs::text`: the same block moved to **after** `headline` — exactly the position change `AGENT.md` would notice | the same **3**. This is goal-state item 4's guard, and it is the reason no `AGENT.md` text block had to be regenerated |
| B6 | `wire.rs::draw_wire`: `perform(…, None)` → `perform(…, Some([&request.from, &request.to]))` — `wire draw` reporting a net through the shared renderer | **1**: `route_labels::auto_labels_writes_the_pair_and_says_so`, on `starts_with("labels U1.1 -> U2.2\n")`. `command_surface.rs`'s `starts_with("routed …")` did **not** fire, and that is correct rather than a hole: it drives the plain `wire draw` path, which never enters `perform` |
| B7 | `wire.rs::connect_wire`: `drawn.report.joined_net = joined_net(…)` → `let _ = joined_net(…)` | **3**: `a_route_joins_the_two_pins_it_names`, `connecting_to_a_net_takes_the_nearest_terminal`, `an_agent_wires_a_sheet_and_kicad_agrees` |
| B8 | `wire.rs::perform`: `route.joined_net = joins.and_then(…)` → `let _ = joins.and_then(…)` — the `wire connect --auto-labels` arm | **1, and only 1**: `a_performed_proposal_reports_the_net_its_labels_made`, added by this task. See below |

#### B3's two green checks, diagnosed rather than skipped

`falsification-control` says green is never good news. Two checks stayed green
under B3 and they are different cases.

- **`a_route_joins_the_two_pins_it_names` — case 2, the dangerous one, and it is
  the degenerate-equality prediction confirmed by measurement.** That check
  compares the reported net with `net_name_of(sheet,"R1","1")`, and the constant
  the break substituted is `SIG_A`, which is that fixture's own answer. A
  reported-equals-extractor check on one fixture cannot see a constant. What
  does see it: `connecting_to_a_net_takes_the_nearest_terminal` and `wire_loop` step 3, which
  assert the literal `"SIG"` on a different drawing, and
  `a_performed_proposal_reports_the_net_its_labels_made`, which asserts `R1_1`
  on a third. **The equality is only sound because those three literals stand
  beside it**, and that is now written into the test as a comment.
- **`a_joined_net_matches_the_golden` — case 1, a genuine no-op.** The new
  golden's own value is `SIG_A`, so substituting the constant `SIG_A` changed
  nothing that golden asserts. It fires on B1, B2, B4 and B5, so it is not a
  blind instrument; it is simply the one case B3 could not move.

#### B8 is the reason a check was added rather than only migrated

**`wire connect --auto-labels` had no behavioural check at all** before this
task — `route_labels.rs` drives `wire draw --auto-labels`, and
`command_surface.rs` only reads the two `--help` texts. The migration put a
decision on that arm (`perform`'s `joins` parameter), and B8 measured that
deleting it left **every check in the repository green**. That is why
`a_performed_proposal_reports_the_net_its_labels_made` exists, and its own
falsification is B8: with it, the break fails; without it, the break is
invisible.

#### The environment break class

The new golden `wire_contract_routed_joined.golden` derives from a probe drawing
and therefore consumes generated identifiers — the class `falsification-control`
says is not covered by source breaks. Second-directory run, by the skill's own
recipe (`git archive HEAD | tar -x -C "$(mktemp -d)"`), of
`wire_output_contract`, `edit_wire_connect` and `command_surface`: **11, 15 and
22 tests, all passing**, under a scratch path this worktree never saw. The
existing `without_generated_identifiers` normalisation is what makes that true,
and the new golden inherits it unchanged.

#### What is NOT watched by a committed check, said plainly

**The `AGENT.md` JSON example block.** `agent_doc.rs` compares the document's
command surface against `clap`, not its example output, so a wrong example
passes every gate. This block was verified once, mechanically, against a real
run (`keys equal: True`, `values equal: True`) — but that verification is a
measurement in this entry and not a check in the tree. It is precisely what
`tasks/M5/chore-8-agent-example-recipe.md` is for, and this is evidence for it.

### Completion check, run in the lane worktree

```
cargo xtask check   ->  fmt pass, clippy pass, test pass, doc pass,
                        deny pass, clean pass — all gates passed
cargo test --test agent_doc        ->  10 passed; 0 failed
cargo test --test command_surface  ->  22 passed; 0 failed
```

Corpus- and environment-gated arms do not count from inside a lane worktree;
the orchestrator's merged run is what counts. The oracle arms in
`edit_wire_connect.rs` were exercised here without `KICLI_TEST_KICAD_CLI`, so
they skipped, and `the_oracle_says_when_it_did_not_run` is the check that says
so rather than letting a silent run read as a passing one.

### Scope, measured at hand-off

`git diff --stat d4c0eb8 HEAD` names 16 files, every one on the brief's IN list:
`AGENT.md`, `research/wire-routing.md`, `crates/kicli/src/route/report.rs`,
`crates/kicli/src/cli/edit/wire.rs`, `crates/kicli/src/cli/edit/wire/contract.rs`,
six files and five goldens under `crates/kicli/tests/`, one new golden, and this
entry. **`.claude/hooks/frozen-paths.txt` does not appear**, which is the
freeze-cycle evidence this task owes from the lane's side: the lift is
`d4c0eb8`, the change is on top of it, and the restore is the orchestrator's
first commit after the merge. `spec/SPEC.md` does not appear either, for the
reason recorded above.

The scope enumeration did **not** prove wrong: nothing outside the IN list was
needed. One file the enumeration did not anticipate — `wire_output_contract.rs`
— is inside it, under `crates/kicli/tests/**`.
