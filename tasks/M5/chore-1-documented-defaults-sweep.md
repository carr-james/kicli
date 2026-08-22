# Chore — nothing checks a prose gloss against the value the code holds

**Provenance: PROPOSED 1, raised by the label-threshold lane (M4 T13's ruling
lane), promoted by James's ruling at the M4 close.**

## The gap

It is the finding that explains why the `label_threshold` defect survived an
entire milestone of green gates. `the_label_threshold_has_one_name.rs` sweeps
for the key's **name**; `agent_doc.rs` checks the key is **present**. **Neither
would have caught "30 G ≈ 381 mm."** The `config.rs` assertion pair added by
that lane guards the code side only — it holds the constant and the millimetre
form together — but nothing holds `spec/SPEC.md`, `research/*.md` and `AGENT.md`
to either of them.

## Why it is a chore and not a patch

**The class is wider than this one key** — every documented default has the same
exposure — so the honest form is a general sweep rather than a special case for
`label_threshold`. The lane did not build it **and was right not to**: the
ruling did not call for it and it would have widened a one-commit diff.

## The trap this chore is exposed to

The derivation rule, all three levels
(`.claude/skills/falsification-control/SKILL.md`). The set of "documented
defaults" must be **derived from an enumeration nobody in the loop wrote** — the
config type's own fields, not a list of key names typed into the sweep. A grep
whose pattern the author wrote is that author's vocabulary wearing a reference,
and this is precisely a check that classifies.

State the taxonomy of gloss forms the sweep covers, and assert that the taxonomy
is its boundary.

## Completion check

The sweep, with a presence control (it must fail if it reads nothing) and a
falsification showing it catches a gloss altered in each governed document.
