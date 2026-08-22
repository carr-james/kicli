# Chore — `blocked` has no committed fixture

**Provenance: PROPOSED 20, raised by the round-trip lane's reviewer (M4 T22),
promoted by James's ruling at the M4 close.**

## The gap

`blocked` is the least-exercised status. The reviewer **could not construct a
live `blocked` refusal within budget**, because the router routes around small
enclosures too readily. That difficulty is the router working as designed — but
it leaves `blocked` verified by **a unit test's mapping table rather than by a
real refusal**.

It is the status an agent most needs to trust, because it is the one that says
"no route exists".

## The chore

A committed walled-in fixture that produces a real `blocked` result, and a check
that reads it.

## The trap

If the fixture is hand-built, it encodes the same assumptions as the code that
reads it. `.claude/skills/falsification-control/SKILL.md`'s hand-built-fixture
rule permits that only where no drawable request can distinguish the behaviour —
so **first establish whether a drawable request can reach a genuine refusal**,
and record the answer either way. The reviewer's failure to build one under
budget is evidence, not a conclusion.

## Completion check

A test that asserts a `blocked` report from a committed fixture, with its
`blocked_by` list naming the obstacles, plus the falsification that shows the
check fails when the wall is removed.
