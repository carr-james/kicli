# The no-floating-point gate has a demonstrated false negative (M5)

**Provenance: found by `lane-ovl` while writing `KI-OVL-001`; the false-negative
half CONFIRMED BY CONSTRUCTION by its tick reviewer, 
`review-ovl`.** Filed by the orchestrator at the checkpoint-2 resumption.

**This is the most serious defect found this session, and it is not in a rule —
it is in the instrument that enforces the Constitution's core numeric rule.**

## The measurement

`crates/kicli/tests/the_linter_holds_no_floating_point.rs`'s `code_of` strips
string and comment content before scanning for floats. It handles `"…"`
(escaped), `'…'`, `//` and `/* */`. **It has no raw-string handling at all** — a
Rust `r#"…"#` is not understood.

**The false-positive half** is how it was found: `lane-ovl`'s rule file was the
first `r#"…"#` under `src/lint/`, and **its good state was failing the gate.**
Annoying, loud, harmless; the lane switched to an ordinary escaped string.

**The false-negative half is the defect, and the reviewer built it rather than
reasoning about it.** Placed under `src/lint/`:

```rust
pub const S: &str = r#"a"b"#;
pub fn smuggled() -> f64 { 3.14159_f64 }
```

```
cargo test -p kicli --test the_linter_holds_no_floating_point
→ ok
```

**The gate passed with a genuine, uncommented `f64` literal present.** The
unterminated-looking inner quote desynchronises the lexer's string tracking, so
everything after it is treated as string content and never scanned.

## Why this matters more than its size suggests

**Constitution §4:** detection is integer geometry only. Floating point appears
in `score()`'s final `exp` and nowhere else, with fixed rounding.

That is one of the project's four hard numeric commitments, and **this test is
the only thing enforcing it.** T3 shipped the score formula with *no floating
point at all* — not even the `exp` the Constitution permits — and this gate is
what keeps it that way.

**The failure mode is this project's own recorded lesson, arriving in the place
it is most expensive.** From `tasks/M5/PLAN.md`'s exit-criteria table:

> *a gate presented as measuring something it cannot fail on is worse than no
> gate, since it spends the credibility of a real one.*

And it is **the third classify-by-shape instrument defect in this record**:
`probe_harness_has_one_home` classifies by the literal string `mod support;`
(PROPOSED 9, still unsettled); `the_four_way_rule_has_one_home` matches *any*
path component named `src`, including a reviewer's mount path, and cost two
reviewers real time this session; and now a float sweep defeated by a quote
inside a raw string. **All three are hand-rolled lexers or textual matchers
standing in for a parse.**

## Goal state, as the checks that prove it

### 1. `code_of` understands raw strings

`r"…"`, `r#"…"#`, `r##"…"##` — arbitrary hash counts, per Rust's grammar. A
raw string's content is **not** code and must be stripped; the terminator is the
matching quote-plus-N-hashes, and **an inner `"` is content, not a terminator.**
Byte strings (`b"…"`, `br#"…"#`) and C strings (`c"…"`) exist too; decide whether
to handle them and **say which you chose and why.**

### 2. The reviewer's smuggling case is a committed check

**The exact construction above becomes a test fixture of the gate's own test.**
It is the one input that proves the repair, and it must fail before the repair
and pass after.

### 3. Both directions are shown

- **False negative closed**: a genuine `f64` after a raw string containing `"`
  is **caught**.
- **False positive closed**: a legitimate `r#"…"#` containing something that
  looks like a float — `r#"1.5"#` — does **not** fire.

Those are two different defects and one check cannot cover both.

### 4. The sibling sweeps are audited, not assumed

**`src/lint/` is swept by four tests.** If `code_of` is shared, or if the same
lexing shape was copied, the others have the same hole. **Enumerate every
textual code-scanning helper under `crates/kicli/tests/` and say, for each,
whether it can be defeated the same way.** Report; do not fix what is out of
scope.

### 5. The honest limit is stated in the gate's own rustdoc

A hand-rolled lexer standing in for a parse **will** have edge cases. Say so
where a reader of the gate will see it, and name what it does not understand.
That is the `CLAUDE.md` "guarded, not enforced" move applied to a gate rather
than to a hook, and it is the cheap half that stops the next reader
overtrusting it.

## Falsification obligation

Per `.claude/skills/falsification-control/SKILL.md`.

- **The repair is shown failing on the pre-repair code.** Run the new check
  against the current `code_of` and watch it go red; that is the whole proof
  the hole was real rather than theoretical.
- **The degenerate check, named in advance:** a test asserting *"the gate
  reports a violation"* passes if the gate reports a violation **for the wrong
  reason** — e.g. it fires on the raw string's own contents rather than on the
  `f64`. **Assert what was found and where**, not that something was found.
- **Beware the `--no-fail-fast` truncation** and **do not share a probe name**
  with another check in the same binary; both are in the skill.

## Scope

**IN**
- `crates/kicli/tests/the_linter_holds_no_floating_point.rs`
- new test files under `crates/kicli/tests/` if the repair wants its own
- this file, for the evidence, written AS YOU WORK

**OUT** — `crates/kicli/src/**` in its entirety. **This is a test-only repair.**
If the gate's repair reveals a real `f64` under `src/lint/`, **STOP AND REPORT
IT** rather than fixing it: that would be a Constitution §4 violation that has
been shipping invisibly, and it is not a chore's call.

The other three sweeps are **report-only** (goal state 4).

**If this scope proves wrong, the named goal state and its checks win over the
list.** Say so in your first paragraph.

## Completion check

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo xtask check
```

— **background it; it is ~2.5 minutes and more under contention, and it is also
the pre-commit hook.** Plus the smuggling check by name, shown failing against
the unrepaired `code_of`.

**Run the full gate before writing the falsification table, not only after.**
Promoted from `lane-ovl`'s WORKFLOW NOTE: a test file is reachable by targeted
`cargo test --test <name>` runs that **cannot see** clippy or the `src/lint/`
sweeps, and two of that lane's red gate runs were defects no targeted run could
have shown.
