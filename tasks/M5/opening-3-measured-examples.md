# Worked examples become measured output (opening 3)

**Provenance: advisor recommendation, James-approved, M4 close (boundary
package, item 9).** Verbatim:

> `AGENT.md` worked examples become measured output: every example block a
> session touches is regenerated from a real run of the built binary, and D3's
> defect class gets an executable twin — a check that `AGENT.md`'s W-record
> examples parse under the same reader the tool uses, extended as feasible.

The rule's prose half is already written, in `tasks/M5/RULES.md` under "Worked
examples in `AGENT.md` are measured output". **This task is the executable
twin**, and `ENGINEERING.md` is the reason it is a task rather than a paragraph:
*a rule that matters wants an executable twin: a gate, a lint, or a
workspace-reading test that fails when the rule is broken. Prose alone decays.*

## The defect class, so the check has something concrete to be about

Dogfood run 1, defect 3, verbatim from `tasks/dogfood.md`:

> **Wire-report coordinate format contradicts AGENT.md's own example.** The
> doc's worked example for `wire draw` shows:
> `+ W 3300f00e (50.80,50.80) -> (63.50,50.80)`
> with parenthesized points and a `->` arrow. What I actually got was:
> `+ W 906eceb2 180.34,41.91..180.34,46.99`
> — no parentheses, `..` instead of `->` […] I had to sit and reconcile these by
> hand; a first-time reader trusting the doc's format literally would misparse
> this line.

That defect was **fixed** — the current `AGENT.md` lines use `..` — and fixing
it closed nothing. **A hand-written example is correct until the writer changes,
and nothing notices.** Eleven `W` example lines stand in `AGENT.md` today
(lines 183, 496, 539, 540, 657–659, 679–681, 706).

## Goal state, as the checks that prove it

1. **A check that fails when an `AGENT.md` record example does not match what
   the tool writes.** The record writer is
   `crates/kicli/src/view/delta.rs` — `line`, `record_of`, and the `Change`
   glyphs `+ - ~ =`. The check must be anchored to **that** writer, not to a
   grammar you author beside it. A regular expression you wrote is your own
   vocabulary wearing a reference (`falsification-control`, the derivation
   rule); a check that round-trips a line through the tool's own emitter is not.
2. **"Extended as feasible" is yours to bound, and the bound is stated in the
   check's own rustdoc.** Say what the check covers, what it does not, and why
   the boundary sits there. A sweep must assert that its taxonomy is its
   boundary — the M4 handle chore was stopped on exactly this rule after three
   rejections, and its lesson is that extending a matcher without a fixed point
   is worse than naming where it stops.
3. **The check cannot pass by reading nothing.** Every absence check carries a
   presence control: assert that example lines **were found**, and that the
   count is what the document holds. A sweep that matched no files and passed is
   the most common blind instrument in this project's record.
4. **Every `W` example block in `AGENT.md` is regenerated from a real run of the
   built binary**, and the entry records the command that produced each. Where a
   block cannot be regenerated — it documents a case no fixture reaches —
   **say so in the entry and leave the block alone**; do not hand-edit it into
   agreement. A block that cannot be measured is a finding about the fixtures,
   and it goes in this entry for the plan to schedule.

## Falsification obligation

Per `.claude/skills/falsification-control/SKILL.md`. Two of the four blind kinds
bear directly:

- **Reads nothing.** See goal 3. Break the document's example lines (in your
  scratch copy) and watch the check fail; then break the *pattern* so it matches
  nothing, and watch it fail too. Both are required.
- **Shared ancestor.** Anchoring to the tool's own writer is the point of goal 1
  — but it means a break *in the writer* moves both sides together. State in the
  entry what this check can and cannot see, and if the answer is "it cannot see
  a writer change", say so plainly. That is an honest boundary, not a gap;
  claiming otherwise is the defect.

## Scope

**IN**
- `AGENT.md` — example blocks only
- `crates/kicli/tests/agent_doc.rs`, or a new test file beside it
- `crates/kicli/src/view/delta.rs` — **only** if the writer must expose
  something for a test to reach it, and then minimally, with the rustdoc saying
  why
- this file

**OUT** — every other source file, every other entry, `tasks/M5/PLAN.md`,
`crates/kicli/src/route/**`.

**If the enumeration above proves wrong, the goal state wins over the list.**
Say so in your first paragraph, name what you touched and why.

## Sequencing

**Originally: after the joined net's contract field
(`opening-1-joined-net-contract.md`) merged**, because that change regenerates
`AGENT.md` blocks of its own and `AGENT.md` is held by one lane at a time.

**Amended at the M5 opening, when opening-1 was PARKED** on the freeze-lift
mechanism BLOCKED item. `AGENT.md` has no other holder, so this task runs now,
and the order is the better one: **the check exists before the change that must
satisfy it.** When opening-1 is re-dispatched it inherits a live check over the
`W` examples — including the `wire draw` JSON block at `AGENT.md` lines 570–579,
which opening-1's reconnaissance identifies as a second block that change
invalidates.

Your brief names the base commit; verify it as your first action.

## Completion check

```sh
cargo test --test agent_doc
cargo xtask check
```
