# `KI-CONN-001` — a pin touches a wire but is not connected (Phase 2, lane B)

**Provenance: `tasks/M5/PLAN.md` Phase 2, RATIFIED at the M5 plan review.**
**Depends on T1, T3, T4.**

## Why this is the most valuable rule in the milestone

`research/style-rules.md` §4, and it is worth quoting at length because it is
the argument for the whole project:

> A pin whose connection point lands on a wire's interior with no junction there
> reads, on screen and to a reviewer, **exactly like a connection**. KiCad
> 10.0.5's netlister does not merge it, so the board is wired differently from
> the way the schematic reads. **This is the most expensive class of schematic
> defect: it survives review.**

**And it is the one Tier 1 rule whose TIER has published support.** `lane-t5`
measured 24 of 28 rules as having no source for their tier. This one does —
Lathrop, on the junction dot: *"It's a rule. We don't care whether you think
it's silly or not. That's how it's done."* Record that; it is rare.

**Overlap with ERC: none.** KiCad's 47 checks have nothing for this — the
netlister simply reports two nets, which from its point of view is not a
violation. *"That makes it kicli's to catch, and it is the clearest example so
far of what 'where things are drawn' adds to electrical correctness."*

## The rule

> **Detect**: a byproduct of the corrected extractor, needing **no new
> geometry**. For every pin connection point `p` and wire segment `w` where `p`
> lies on the **interior** of `w` (not within 1 IU of either endpoint): finding
> when `p` and `w` are **in different nets after union-find**. Geometric
> coincidence without electrical merge is the whole test. **Sheet pins are
> covered the same way.**

**Read `research/notes/pin-on-wire-interior.md` before writing anything.** The
behaviour is measured there against KiCad 10.0.5 in **both directions**.

**It also catches the label-plus-pin case** and needs no extra detection for it:
a pin sharing a mid-wire anchor with a label forms a net with the label and
leaves the wire out, which draws as a connection and is not one
(`research/notes/label-on-wire-interior.md`). **The pin's net is not the wire's
net, which is already the test.** Say in the entry that you checked this rather
than assuming it.

## Goal state, as the checks that prove it

### 1. The fixture already exists, and that is a gift with a catch

`crates/kicli/tests/fixtures/sch/nets/nets.kicad_sch` **carries one cluster of
each kind**, so this rule has a positive and a negative case from the day it is
written.

**The catch, and it is the trap of this task:** a rule tested only against a
fixture built for the *extractor* may be measuring the extractor's answer rather
than the rule's. **Build at least one fixture of your own**, and say what it
tests that the existing one does not.

### 2. The interior test excludes endpoints by 1 IU

Not by tolerance-in-spirit: **1 IU exactly**, in integers. A pin **at** a wire
endpoint is an ordinary connection and must never fire. That is the false-
positive direction and it is the expensive one.

### 3. `--gate`'s answer must be the netlist's answer

This rule stands on union-find. **The oracle is the control**: the netlist
oracle is a standing milestone gate at 35/35 and this rule reads the same
connectivity. Per `.claude/skills/oracle-check/SKILL.md`, **connectivity-touching
work carries an oracle check.** This task touches connectivity's *interpretation*
rather than its computation — **say which, and run the oracle anyway.**

### 4. The fix hint is exact and is not a layout decision

> `add a junction at <x>,<y>` — **that is the one-item change that makes the
> drawing mean what it looks like.** The alternative, moving the symbol off the
> wire, is a layout decision and is **not** suggested automatically.

`fix` is a *suggested command* and **kicli never mutates during scoring**
(`spec/SPEC.md` §11.3). The command must be one the binary actually accepts —
per the measured-examples rule, **produce it from a real run**, do not compose it
by hand.

### 5. Saturation declared

This rule counts **pin connection points**. Declare its denominator and fraction
per T4.

## Falsification obligation

- **Both directions on the shipped fixture**: the connected cluster produces
  **no** finding; the coincident-but-unmerged cluster produces one, naming the
  pin and the wire.
- **The endpoint boundary**: a pin exactly at an endpoint (no finding), and one
  1 IU inside the interior (finding).
- **Degenerate-equality warning, and it is live here.** A check asserting the
  rule's net equals the extractor's net is comparing a thing with itself if both
  come from the same union-find call on the same seam. `opening-1` predicted
  exactly this trap in advance and **it fired** — a break replacing the net with
  a constant left the check green, because the constant was that fixture's own
  answer. **State what your two sides derive from**, and if they share an
  ancestor, say what stands beside them.
- **The label-plus-pin case is shown firing** without any code written for it —
  goal state's claim. If it does not, that is the finding.

## Scope

**IN**
- `crates/kicli/src/lint/rules/pin_on_wire.rs` — **new file, no edit to any
  existing file.**
- new test files and new fixtures under `crates/kicli/tests/`, **plus the
  `MANIFEST` line for each fixture you commit** — add your line and nothing else,
  and say so
- this file, for the evidence, written AS YOU WORK

**MERGE HOTSPOTS — report, do not edit.** `Cargo.toml`, `lib.rs`, `build.rs`,
`AGENT.md`, `spec/SPEC.md`, `tests/command_surface.rs`, `kicli.toml`'s `[rules]`.

**OUT** — every other rule file, every other entry, `crates/kicli/src/connectivity/`
(**report** anything the extractor owes; a change there is the orchestrator's to
sequence, because the oracle stands on it).

**If this scope proves wrong, the named goal state and its checks win.** Say so
in your first paragraph.

## Completion check

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo xtask check
cargo test -p kicli --test rule_files_are_formatted
```

plus the oracle, which is environment- and corpus-gated and therefore **does not
count toward done from inside the lane** — run it to make the measurement this
task owes, and say which arms ran with what:

```sh
cargo test -p kicli --features corpus --test net_oracle -- --nocapture
```
