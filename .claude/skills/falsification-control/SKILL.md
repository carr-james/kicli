---
name: falsification-control
description: How to show a check is capable of failing before it counts. Use when writing or reviewing any test, sweep, or gate.
---

# Falsification control

A check that cannot fail is decoration. Before a check counts toward a tick, it
is shown capable of failing, and the falsification is recorded in the task entry.

*Restructured at the M5 opening, by James's ruling on the M4-close batch, which
folded four separate findings into one organisation and kept every worked
example. The reason is in the finding that prompted it: these are not four rules
but **one idea at four levels** — an instrument is blind in the dimension its
author was not thinking in — and stating it once with the examples is shorter
than four appends, and truer.*

## Procedure

1. State the check against reality, not against the code's own structure — a
   test that restates the implementation passes just as happily when both are
   wrong.
2. Break the thing the check watches, in the source, deliberately.
3. Watch the check fail. If it stays green, **see "Green is a finding" below.**
4. Restore the source; record in the task entry WHAT was broken and WHICH
   assertion caught it.

**Record exactly what you removed, not only where the failure surfaced.** A row
naming a single line number for what was really a two-assertion removal reads as
precise and is not: the next reader removes that one line, watches a *different*
assertion catch the break, and has to work out whether the row is imprecise or
fabricated. Name the assertions by their message or their function, and if you
removed several to isolate one, say so. Provenance: the four-way (T12) review,
where exactly this cost the reviewer real time — it resolved as imprecision, but
only after the work of ruling out the alternative.

## Green is a finding about the instrument

Step 3's trap is that green *feels* like good news, so the row gets skipped past
and the table records a break that "did not apply". **It is never good news.** A
break that leaves the check green means the instrument may be blind, and the
instrument is investigated before any conclusion is drawn from that row.

Two cases, and telling them apart is the investigation:

1. **The break was a no-op** — nothing about behaviour actually changed, so
   nothing could have been caught. The row is real evidence about the code:
   something else already enforces the rule.
2. **The check does not watch what it claims.** The code was innocent and the
   control was blind. This is the dangerous one, because the check will keep
   passing forever and no one will look again.

**Never record case 2 as case 1.** "Removing it changes nothing" is the same
sentence for "this rule is redundant" and "my test cannot see this rule", and
those are opposite findings.

### Case 1 diagnosed as case 1, with the work shown

The candidate shapes (T9) measured that with **both** `polyline` guards removed,
all five drawing checks still pass, and did **not** read that as the guards being
safe. It established the structural reason — a terminal's own body box covers
every neighbour of its own cell except the escape point, so the map blocks the
reversing leg anyway — and recommended keeping both rules while measuring them
in `route::shapes::tests` where they can be seen.

## The four kinds of blind instrument

Each row is a way a check can be watching, and be watched failing, and still see
nothing. Each has a diagnostic question you can ask before you write the check.

| Kind | The check is blind because | Ask | Found by |
|---|---|---|---|
| **reads nothing** | the sweep matched no files and passed | *what proves it found something?* | D1, and two others |
| **author's vocabulary** | it recognises the spellings its author thought of | *who wrote the list I am matching against?* | C1 |
| **shared ancestor** | its control is computed by the code under test | *does my control share an ancestor with the thing it controls?* | T19, T21 |
| **degenerate equality** | the two sides agree because neither does anything | *what else would make these equal?* | C8 |

### 1. Reads nothing — every absence check carries a presence control

A sweep asserting the absence of a retired name must also assert that the
canonical name **is** found, so a sweep that read nothing cannot pass (M4 T5).
The same shape catches vacuous arithmetic: a falsification in the cost model
(M4 T8) fired the **anti-vacuity control** rather than an arithmetic assertion —
corners counted as zero were caught by the check that the check itself saw work.

### 2. The author's vocabulary — the derivation rule, at three levels

Paid for three times, in three consecutive rejections of one chore's regression
guard (C1, the eight-character handle rule). Each fix moved the boundary without
changing its kind:

| Rejection | The evasion | What it was blind to |
|---|---|---|
| 1 | a parameter named `id`; a method on a type named `Ident` | it classified by **name**, against a closed word list |
| 2 | `format!("{:.8}", uuid)` | it enumerated `str`/`String`/`Iterator` **methods**; precision is not a method |
| 3 | `chars().take(0x8)` | it matches the **decimal spelling** of a mechanism, not the **value** 8 |

The rule in its final form, assembled from the lane's own words and one
reviewer's extension:

1. **Derive the vocabulary from an enumeration nobody in the loop wrote — and a
   citation is not a derivation.** A grep whose pattern you authored is still
   your vocabulary wearing a reference. The mechanical test: *can a reader re-run
   the derivation and get the list, without trusting anyone's judgement about
   what belongs in it?*
2. **Choosing which enumerations to run is itself authorship.** A sweep must
   state the taxonomy of mechanisms it covers and assert that the taxonomy is
   its boundary, because an enumeration can be exhaustive within a category
   while the category was chosen from memory.
3. **A taxonomy of mechanisms is not enough if the matcher recognises spellings
   rather than meanings.** That is where a textual instrument stops and a
   semantic one has to start.

**And knowing where it stops is a result.** C1's sweep was stopped deliberately,
with its claim narrowed to exactly what it enforces and the boundary written
into the check's own rustdoc, because extending it had no fixed point — after
the radices come integer suffixes, then `4 + 4`, then a `const` one line above
the call. **A textual matcher cannot decide a value.** Naming the boundary beats
an honest-looking matcher that is one spelling behind.

### 3. The shared ancestor — a control computed by the code under test

The break changed behaviour, the check was watching, and the check still passed
— because both of its controls were computed by the same code the break was in,
so they moved together. A dearer route was produced and both sides of the
comparison got dearer (T19; T21 found the same shape in
`crates/kicli/tests/mutation_loop.rs`, where `is_canonical()` and `emit()` are
compared through one `prettify`).

A control sharing an ancestor can only detect breaks that affect the two sides
**differently**. The fix is a control derived independently: asking the router
for each candidate point by a different route; re-saving the file through
`kicad-cli sch upgrade --force` in the same run rather than through the same
prettifier.

**Two worked failures from the determinism property test (T11)**, both of which
left green checks over innocent code:

- **A shuffle that permuted nothing.** `reordered()` was broken to ignore its
  `order` argument, and the check passed — because the control compared the
  drawing against its own text, so it agreed on layout alone. Rewritten to
  compare against the same file through the same writer, unshuffled, the break
  fails. A determinism check that passes when nothing varies is the exact
  failure mode the check exists to prevent.
- **Every answer replaced by a constant.** This passed the shuffled arm, which
  carried no class counters. The baselines now must hold a shape route, an A\*
  route and a refusal, and the break fails both arms.

The tick reviewer re-made both breaks against the current control *and* against
a reconstruction of the old one, so what the record holds is a **contrast**
rather than a pass. That contrast is the evidence.

### 4. Degenerate equality — what else would make them equal?

**When a check asserts two values are EQUAL, ask what else would make them
equal. The answer is usually "the code doing nothing."**

C8's case: a `document_name` that returns a **constant** makes two checkouts
produce identical identifiers, which is exactly what the check asserts. The
check was watching, the break changed behaviour, and it passed — because a
constant is checkout-independent in the most trivial possible way. An equality
check needs a second arm that a do-nothing implementation fails: assert the two
sides differ where they must differ, not only that they agree where they must
agree.

## The fifth dimension: the environment is a break class

**Promoted from PROPOSED 9 by advisor ruling, M4 checkpoint-2 review.** The
procedure breaks the **source**. A check can be falsifiable against every source
break and still be asserting a property of the machine it ran on, because
nothing in steps 1–4 ever varies the machine. It is the same idea one level out:
the author was not thinking about the environment, so the instrument is blind
there.

**Path, clock, locale and run order are break classes**, and they apply to any
check that consumes a **generated value** — an identifier, a timestamp, a hash,
a temporary directory, a sort over anything unordered.

**The tell that you need this:** the check's expected value was *produced by
running the code* rather than *derived from the contract*. A golden is the
common case.

### The second-directory run, and how to do it

**The rule:** such a test runs once from a second directory before it is
reported green. One extra run.

**The procedure**, because every lane that hits this re-derives it (D1). Inside
a git worktree the clean way is to take the commit out of git rather than to
copy a live tree:

```sh
scratch="$(mktemp -d)"                 # never a fixed path; never one you did not create
git archive HEAD | tar -x -C "$scratch"
( cd "$scratch" && cargo test --test <name> )
```

`git archive` gives you the commit, not the working tree, so the second run
cannot be contaminated by an uncommitted break you forgot to restore. Renaming
the scratch directory and running again is the cheaper variant when the test
takes its own scratch path as input.

### Worked example — the T16 golden defect

Two `routed` goldens passed every gate in the lane worktree and failed the
moment the orchestrator ran them after merge. The identifiers in them are a
SHA-256 of a seed built from the drawing's **absolute path**, so the goldens
asserted a property of the worktree they were written in.

The falsification table for that task had **fifteen rows**, and rows 2, 3, 4, 5
and 14 all broke the renderer and all failed these same two goldens. The goldens
*were* shown capable of failing. That is necessary and not sufficient:

> a check can be falsifiable and environment-dependent at the same time, and the
> procedure as written only tests the first. **Every break was made in the
> source; none was made in the environment.**

The reproduction is the whole rule in one line: changing **only** the probe's
scratch directory name made the pre-fix commit fail those two tests and no
others — `fa9bd366…`/`6ebfadf1…`/`9b63e57e…` under the changed path against
`ebb43fde…`/`d42bd368…`/`85a91ae2…` under the original.

**And note which failure mode this is.** The values were *stable* per checkout
and *wrong* everywhere but one — worse than random, because random fails loudly
on the second run, and this failed only on somebody else's machine.

The fix was not to freeze the identifiers: `matches_golden` normalises each
**distinct** identifier-shaped string to `<id-1>`, `<id-2>` … in first-appearance
order, so count and ordering are still asserted, and the real values keep their
own check on shape and distinctness so the normalisation hides nothing.

## Git will not hold your good state, and your evidence rots by default

Three collisions between this skill and commit discipline were reported
independently in one session, plus a fourth from a reviewer. They are one thing:
**the tree moves under you, and a claim written once is a claim about a tree
that no longer exists.** Ruled at the M4 close as two rules.

### Rule 1 — commit the good state, and anchor evidence to content

**Git can only restore committed state.** Step 4 assumes the source can be put
back, and `git checkout -- <path>` puts back **the last commit**, not the thing
you had a moment ago. So: **before any deliberate break, the good state is
committed** — new file or tracked file alike.

The rule is not about whether git knows the **file**, and not about whether you
committed **once**. It is about whether git knows **the state you want back, at
the moment you break something.** After that first commit, any *further*
improvement is uncommitted too, and the next `git checkout --` silently
discards it. A lane lost a strengthened sweep exactly this way and had to
re-apply it.

**A tracked file reads as safe and is not.** `git checkout --` on it succeeds,
exits 0, and takes your uncommitted work with it. That is the trap: the command
does exactly what it promises, and what it promises is not what you wanted.

Two adjacent traps. `git checkout --` with several pathspecs restores
**nothing** when one of them fails, so a multi-path restore that includes a new
file silently leaves every break in place. And a restore is not complete because
the command exited 0 — checksum the restored file against the good state before
the next break.

**`--amend` is the corollary, and it is licensed.** A brief that requires one
commit and a skill that requires the good state committed first are compatible
**only** by amending. Neither document used to say so, and a lane that did not
think of it had to choose which rule to break. Amend freely; the one-commit
requirement is about what lands, not about how many times you typed `git
commit`.

**Which is why falsification evidence anchors to CONTENT HASHES, not commit
SHAs.** Cite `shasum` of the file in the state you are describing. **A content
hash survives amending, rebasing and merging `main` forward; a SHA does not** —
and in a session where every lane merged forward at least once and one did so
six times, SHA-anchored evidence rots by default rather than by accident. The
`AGENT.md` lane cited its good-state commit's SHA in the entry, then folded the
entry in by `--amend`, and destroyed the SHA the entry named.

#### Worked example — a brand-new file

The `wire draw` (T14) implementer's note, verbatim:

> Second: `crates/kicli/src/edit/wire.rs` is untracked until the first commit, so
> `git checkout --` cannot restore it — falsification runs on a brand-new file
> need a commit of the good state first, which the falsification-control skill
> does not mention.

#### Worked example — a tracked file, which is the one that surprises people

From the orchestrator's own report, on the contract amendment `dd4f659`:

> The first falsification break above was made against `report.rs` while the
> whole contract change was **uncommitted**.
> `git checkout -- crates/kicli/src/route/report.rs` duly restored the file to
> `HEAD` — and took the entire change with it, not just the break. The file was
> tracked, so the amendment as written did not cover it; the failure is
> identical.

Cost: four edits re-applied. In a lane it would have been a task's work.

### Rule 2 — evidence that names a moving target is re-verified at the last merge-forward

**A claim about a moving target is pinned or re-verified after every
merge-forward, and never trusted across one.** Ruled as one rule at the M4
close, covering a family that was assembled without being named:

| The claim rots because | Reported by |
|---|---|
| it cites a commit SHA, and amend/rebase/merge changes it | the `AGENT.md` lane |
| it assumes the base stops moving after a resume | the handle chore |
| it states a diff scope, and merging forward widens it | T21's reviewer |
| it names a condition that ends mid-lane | T21 |

The measured case: T21's entry claimed `crates/kicli/tests/fixtures/**` had a
zero diff from its base. Measured at review: **11,059 lines**, entirely from a
fixture chore T21 had merged forward and disclosed merging. **The claim was true
when written and went stale when `main` moved.**

So, in practice: merge `main` forward whenever it moves under you and say how
many times you did; and at the last merge-forward before hand-off, **re-measure
every scope or state claim your entry makes** rather than re-reading it.

## Special case: hand-built fixtures

Hand-built test geometry is permitted only where no drawable request can
distinguish the behaviour (ruled, M4 T7). A hand-made fixture encodes the same
assumptions as the code that reads it, so agreement proves nothing — build
fixtures through the harness wherever a real request can reach the behaviour.
It is the shared-ancestor kind, wearing a fixture's clothes.

## One more from the record, kept because it is the cheapest example

**Sign inversion (M4 T6).** The escape rule was stated against the symbol's body
box, not against the code's sign. The implementer then inverted the sign in the
source and watched the test fail — the control that distinguishes a test of the
rule from a restatement of the implementation. A restatement would have passed
with every route leaving its pins through the symbol.
