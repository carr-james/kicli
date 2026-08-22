# Carried in from M4 C1 — the handle rule needs a lint, not a sweep

*Migrated verbatim from the former `tasks/M5.md` at the M5 opening, by the
boundary-package ruling that gives M5 one file per task. The text below is the
record as M4 wrote it; nothing was re-argued in the move.*

**Not a deferral. It is the conclusion of three rejected reviews, and the
conclusion is a fact about instruments rather than about the chore.**

M4's C1 folded five private copies of the eight-character handle rule into one
(`Uuid::short`), keeping a separately-named `short_key` for keys that are not
identifiers. **That half is done and was confirmed by three independent
reviewers.** What is carried here is the *guard*.

The chore's regression guard is a textual sweep over `crates/`, and it was
rejected three times, each reviewer finding a real gap one level below the last:

| Rejection | The evasion | What it was blind to |
|---|---|---|
| 1 | a parameter named `id`; a method on a type named `Ident` | it classified by **name**, against a closed word list |
| 2 | `format!("{:.8}", uuid)` | it enumerated `str`/`String`/`Iterator` **methods**; precision is not a method |
| 3 | `chars().take(0x8)` | it matches the **decimal spelling** of a mechanism, not the **value** 8 |

Each fix moved the boundary without changing its kind, and the third makes the
reason plain:

> **A textual matcher cannot decide a value.**

Extending it further has no fixed point — after the radices come integer
suffixes, then `4 + 4`, then a `const` one line above the call. The sweep was
therefore **stopped deliberately**, with its claim narrowed to exactly what it
enforces and its boundary written into the check's own rustdoc.

**What M5 inherits is the instrument, not the rule.** The honest form of this
claim is a lint over MIR, where `take(0x8)` and `take(8)` are the same node and
a `const` is already folded. That is real work with a real dependency
(`cargo-dylint` or an equivalent, and a licence check under Constitution §9), and
it is why it was not attempted inside a chore.

**The general lesson, in the words the lane reached and a reviewer then extended,
because it applies to every check this project writes:**

1. Derive the vocabulary from an enumeration nobody in the loop wrote — *and a
   citation is not a derivation; a grep whose pattern you authored is still your
   vocabulary wearing a reference.*
2. **Choosing which enumerations to run is itself authorship.** A sweep must state
   the taxonomy of mechanisms it covers and assert that the taxonomy is its
   boundary, since an enumeration can be exhaustive within a category while the
   category was chosen from memory.
3. **A taxonomy of mechanisms is not enough if the matcher recognises spellings
   rather than meanings.**

Recorded here rather than only in the M4 report because it is the kind of finding
that has to be in front of whoever writes M5's rule catalogue, which is a whole
milestone of checks that classify.
