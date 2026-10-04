# ERC partial overlap has no mechanism, and `KI-GRID-001` is in it (M5)

**Provenance: the orchestrator, at the checkpoint-2 resumption, from two
measurements that only meet when both are read together.** Neither lane could
have found this alone, and neither is at fault.

## The two halves

**`KI-GRID-001`'s lane measured** that a positional join against ERC is wrong
for wires. For `(xy 25.4 25.4) → (xy 38.735 25.4)`, where **`38.735` is the
off-grid end**, KiCad's `endpoint_off_grid` names **`@(25.40 mm, 25.40 mm)`** —
the wire's **on-grid anchor**. Its reviewer reproduced this against `kicad-cli`
10.0.5 bit-for-bit, and the pin case too, where KiCad names the offending
coordinate exactly. Conclusion: **the join must be by object, not by
coordinate.**

**T2's reviewer then established** that T2's seam performs **no per-item join at
all.** `covers()` is a per-check-key boolean; `delegate()` records KiCad's
violations wholesale. The rustdoc states the discipline:

> *"A rule calls this instead of looking, never as well as looking."*

**That discipline is better than a join** — it makes double-counting
structurally impossible rather than arithmetically avoided, and it is the right
default for the two deliberate exceptions T2 was built for (`four_way_junction`
→ `KI-JCT-001`, `single_global_label` → `KI-LBL-003`), where ERC's coverage and
kicli's rule are the **same** check.

## The gap

**`KI-GRID-001` is the first rule where ERC's coverage is a STRICT SUBSET of the
rule's, and the seam has no form for that.**

`research/style-rules.md` §4, verbatim:

> **Overlap with ERC**: ERC's `endpoint_off_grid` covers *wire endpoints* only,
> as a warning. `KI-GRID-001` additionally covers pins, labels and sheet pins,
> and is blocking. **Report ERC's finding when present; do not double-count.**

So the rule must **both** delegate (for wire endpoints, where ERC looked) **and**
look (for pins, labels and sheet pins, where ERC did not) — on the same drawing,
in the same pass. That is exactly what *"instead of looking, never as well as
looking"* forbids.

**Neither lane did anything wrong.** T2's scope was the mechanism and its two
same-check exceptions; `KI-GRID-001`'s scope put the seam's design out of reach
and it correctly **reported what the seam would owe it** rather than inventing a
second mechanism. Both ticks are APPROVE and both are sound. **The gap is in the
seam between two correct deliverables, which is the kind only the orchestrator
is positioned to see.**

## What is true in the tree today, and why this is not urgent

**`KI-GRID-001` as merged does not double-count, because nothing joins at all** —
it detects all six classes itself and ERC's finding is not consumed by it. So
the shipped behaviour is **complete detection with an unattributed overlap**, not
a double count.

**The visible consequence is attribution, not arithmetic**: on a sheet whose wire
endpoint is off grid, ERC says so as a *warning* and `KI-GRID-001` says so as a
*blocking* finding, and nothing tells a reader these are the same object seen
twice. Under Constitution §6 that is a context-budget cost and a confusing one;
it is not a wrong answer.

**No CLI verb consumes the seam yet** — T2's reviewer confirmed no `--gate` or
`sch erc` wiring exists. So this must be settled **before `sch score` ships**,
which is the entry `phase2-sch-score-command-surface.md` and the last task of
Phase 2.

## Goal state

### 1. The seam gains a third form, or the rule gains an exemption — and that is a DESIGN decision

Two shapes are available and they are genuinely different:

- **`delegate_partial(CHECK, drawing, found, |item| …)`** — the rule delegates
  the subset ERC covers, keyed **by object UUID**, and looks only at the
  complement. Keeps one finding per object. **Cost:** every partial-overlap rule
  must state its complement correctly, and a wrong complement is a silent gap
  rather than a double count — the more expensive error.
- **Attribution only** — `KI-GRID-001` keeps full detection and the finding
  carries *"KiCad's ERC also reports this as `endpoint_off_grid`"*. **Cost:**
  nothing dedupes across tools, so a reader still sees two lines; but the rule's
  own answer is never incomplete.

**Recommendation: the second, with the first recorded as the shape to take if a
later rule needs real deduplication.** The north star's expensive error is the
false or missing finding, not the duplicated one — and `KI-GRID-001`'s own
measurement is the argument: **a positional join would have suppressed nothing
for wires and could suppress wrongly for pins**, so a join that is hard to get
right is worse here than an attribution that is impossible to get wrong.

**This is a design call on a published contract (§4's "do not double-count"), so
it is PROPOSED rather than decided.** If the advisor or James prefers the first
shape, it is T2's seam that grows, not the rule.

### 2. Whichever is chosen, the object key is UUID

Settled by measurement and not re-openable: **a positional key is wrong for
wires.** T2 already parses `Item.uuid` and populates `Violation::objects()`
precisely so a consumer can join by object, and its module docs name UUID as the
correct key. **Any implementation that joins by coordinate is a defect on
arrival.**

### 3. The check is the attribution, and it must be able to fail

A check asserting *"the finding mentions ERC"* passes on a hard-coded string.
**Assert that the attribution names the same OBJECT ERC named** — which is the
only claim with content, and the only one a positional key would break.

## Falsification obligation

Per `.claude/skills/falsification-control/SKILL.md`.

- **The wire case is the falsifying case and it already exists**: the fixture
  behind `KI-GRID-001`'s measurement. A check that passes on the pin case and
  fails on the wire case is the one worth having, because **the pin case passes
  under a positional key by coincidence** and coincidence is not a test.
- **`the_linter_holds_no_floating_point` cannot lex raw strings** (`chore-10`) —
  use ordinary escaped strings.
- **Do not edit any tracked file while a gate run or commit is in flight** —
  BLOCKED 1; the `clean` arm snapshots the tree before and after.

## Scope

**IN** — `crates/kicli/src/lint/erc.rs` (the seam) **or**
`crates/kicli/src/lint/rules/grid.rs` (the rule), **whichever the ruling
selects — not both**; new tests under `crates/kicli/tests/`; this file.

**OUT** — every other rule file, `spec/SPEC.md` and `research/style-rules.md`
(**report** the amendment owed; §4's *"do not double-count"* sentence may need
re-wording and both are the orchestrator's), the fixture `MANIFEST` beyond a
line for a fixture you commit.

## Completion check

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo xtask check          # background it; it is also the pre-commit hook
cargo test -p kicli --test rule_files_are_formatted
```

plus the attribution check by name, shown failing under a positional key.
