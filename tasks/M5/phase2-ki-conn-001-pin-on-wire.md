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

---

# Evidence and deliverable (lane b)

## Base verification (first action)

```
$ git -C .claude/worktrees/lane-b log --oneline -1
18ab930 merge: the Greenberg checklist, vendored so the citation cannot drift (M5)
$ git -C .claude/worktrees/lane-b status --porcelain
(empty)
```

Base matches the brief. No fast-forward needed.

## DISCLOSED scope deviation, stated first

**This lane edited one existing file: `crates/kicli/src/lint/drawing.rs`.**
**Thirty-six added lines, two additive methods, no signature changed and no
caller touched.** The task's Scope IN says *"new file, no edit to any existing
file"*; the same Scope says *"if this scope proves wrong, the named goal state
and its checks win."* The goal state is the rule, and the rule cannot exist
without this edit. The measurement that says so is the next section, and it was
taken before a line of the rule was written.

`crates/kicli/src/connectivity/` was **not** touched. What the extractor owes is
reported below rather than written.

## MEASURED: a rule could not reach connectivity at all

**The claim is not "the seam is awkward". It is that `KI-CONN-001` was not
expressible, and the compiler says so.** Two probes, run at `18ab930` before any
implementation, each reduced to the one expression the rule needs.

Probe 1 — can a rule reach the partition? A rule file whose `examine` opens with
`let nets: &Nets = drawing.nets();`:

```
error[E0599]: no method named `nets` found for reference `&Drawing<'_>` in the current scope
  --> crates/kicli/src/lint/rules/pin_on_wire.rs:20:35
```

`Rule::examine` receives a `Drawing` and nothing else. `Drawing` carries the
doc, the typed objects, the embedded library and the sheet path, and — by its
own documented design — *"no file name and no way to reach the disk"*. Net
membership is reachable only through `connectivity::extract(&Hierarchy)`, and
`Hierarchy::load` takes a `&Path`. **So before this task, no rule of any kind
could ask a connectivity question.**

Probe 2 — given a `Nets` in hand, can it name the net a wire segment is on?

```
error[E0599]: no method named `net_of_line` found for reference `&'a Nets` in the current scope
help: there is a method `net_of` with a similar name, but with different arguments
```

`Nets`' whole surface is `warnings()`, `nets()` and `net_of(reference, number)`,
and a `Net` lists `NetPin`s — reference designator, pin number, sheet, symbol,
`power`, `on_board`. **There is no wire in it.** `connectivity::graph::Node`
carries no item identity for `NodeKind::Line` at all, so the partition cannot be
asked about a wire even from inside the module. `KI-CONN-001`'s test is *the
pin's net against the wire's net*, so this is the second half of the gap.

**One thing the record already sanctioned this direction**:
`the_linter_holds_no_write_path.rs` lists `connectivity` in `CRATE_PATHS`, the
whitelist of modules the linter may name. The linter was already permitted to
read the partition; it had no way to be handed one.

## What was built

| Path | What |
|---|---|
| `crates/kicli/src/lint/rules/pin_on_wire.rs` | **new** — the rule, its four unit checks |
| `crates/kicli/tests/lint_pin_on_wire.rs` | **new** — seven end-to-end checks |
| `crates/kicli/src/lint/drawing.rs` | **edited, +36 lines** — `with_nets`, `nets` |

**No fixture was committed and no `MANIFEST` line was added.** The brief granted
the narrow permission and it was not needed: `kicli_probe::drawing::Probe` is the
project's existing way to build *"the smallest schematic that shows one rule"*,
and `Probe::partition` asserts kicli's partition against `kicad-cli`'s on the
same file when the tool is present. A probe therefore gets the KiCad
verification a committed fixture would get, with no manifest record to keep in
step. **`kicad-cli` 10.0.5 is installed in this worktree's environment**, so
every probe below was measured against KiCad rather than reasoned about.

## The rule's two sides, which is this task's load-bearing claim

**Neither side is the rule's own arithmetic. Both are the extractor's, and they
are two different questions rather than one answer compared with itself.**

- **The pin's side**: the net the partition lists that pin on, looked up by
  sheet path, reference designator and pin number.
- **The wire's side**: the net of a pin sitting at an **end** of the wire, or of
  a wire joined to it end to end. A wire's connection points are its two ends
  (`SCH_LINE::GetConnectionPoints`), so a pin at an end is on that wire's net by
  the extractor's first merge rule — exactly, and with no inference of this
  file's own.

They share an ancestor: one `extract` call. **What stands beside them is that
the signal is their *difference*, and no break moves a difference the way it
moves an equality.** Collapse the partition and both sides become one net and
the rule goes silent; shatter it and every pin is its own net and the rule
reports everywhere. That is measured, not argued — breaks B2, B3, B4 and B7
below each move one side and each is caught.

**And the checks' two sides share nothing at all.** On the committed fixture the
expectation is computed from `sch/nets/nets.netlist` — the bytes `kicad-cli sch
export netlist` wrote, carried in `MANIFEST` with `kicad-cli` provenance —
asking KiCad's partition which pins share a net. kicli contributes only pin
*positions*, through its reader, which is a different subsystem from the
union-find and not the one the trap concerns. On a probe the expectation is the
drawing's construction, and `Probe::partition` refuses to return at all if KiCad
disagrees about that drawing's connectivity.

**The `opening-1` break, run verbatim.** That entry predicted this trap and it
fired once already: *"a break replacing the net with the constant `SIG_A` left
the check green, because the constant was that fixture's own answer."* Break B3
is that break — the pin's net replaced by a constant — and **six of seven checks
go red**.

## Falsification table

Every break was applied to the shipped code, the suite run, and the code
restored. `✓` means the check went red under that break.

| # | The break | What went red |
|---|---|---|
| B1 | the endpoint exclusion deleted from `is_interior` | **2 of 4 unit checks**; **0 of 7 end-to-end checks** — see below |
| B2 | the net comparison deleted, so every interior pin is reported | 5 of 7 ✓ |
| B3 | **the `opening-1` break**: the pin's net replaced by a constant | 6 of 7 ✓ |
| B4 | no wire ever found crossed, so the rule reports nothing | 6 of 7 ✓ |
| B5 | the fix hint suggests moving the symbol instead | 1 of 7 ✓ (the one that reads it) |
| B6 | **the plausible wrong rule**: "fire when no junction is at the point" | 2 of 7 ✓ — and they are exactly the two negative checks written for it |
| B7 | `Drawing::with_nets` ignores the partition it is handed | 6 of 7 ✓ |

### B1 is the finding of this table, and it is recorded rather than hidden

**Deleting the 1 IU endpoint exclusion left all seven end-to-end checks green.**
That is not a hole in the checks; it is a fact about the published rule. A pin
at a wire's **end** is on that wire's net by the extractor's first merge rule,
so the net comparison suppresses it whether or not the exclusion is there. **The
exclusion is redundant with the union-find phrasing**, and no drawing can show
the difference.

It is kept, because it states the rule's own definition and covers the case
where the two mechanisms could disagree. It is now checked where deleting it is
visible: four unit checks in the rule file exercise `is_interior` directly — an
end (no), one internal unit along (yes), one unit short of the far end (yes),
one unit off the line (no), a zero-length wire (no), and a diagonal. Re-running
B1 against them turns **2 of 4 red**, and the integration suite stays green,
which is the measurement pasted rather than the claim asserted.

### The reading of "1 IU", recorded because the rule's two sentences differ

`research/style-rules.md` §4 says *"not within 1 IU of either endpoint"*; this
entry's own falsification obligation says *"a pin exactly at an endpoint (no
finding), and one 1 IU inside the interior (finding)."* Under the first reading
a point 1 IU along is excluded; under the second it fires. **The second is the
more specific statement and it is what is implemented**: the exclusion is the
endpoint itself and nothing wider. Every schematic coordinate is an integer
number of internal units, so there is no point between "at the end" and "one
unit along" for the two readings to disagree about — the phrase "1 IU" is
saying the zone has no tolerance, not that it has width.

## The fix hint, produced and then proved by a real run

`fix` is a suggested command and scoring never mutates, so the string was made
by the rule and then **run verbatim by the built binary on a copy of the
fixture**, and KiCad asked what changed.

The rule's string, read out of the finding: `kicli junction add --at 38.1,88.9`

**The first draft said `kicli sch junction add`, which the binary does not
accept.** `AGENT.md` §`kicli junction add` and `kicli junction add --help` both
say the noun is top level. Corrected from the tool rather than from memory —
this is exactly what the measured-examples rule exists to catch.

```
########## the suggested command, run verbatim by the built binary
$ kicli junction add --at 38.1,88.9
+ W 0540399e 38.10,88.90
checked: every invariant passed
exit=0

########## KiCad's netlist BEFORE               (kicad-cli 10.0.5)
    Net-(R12-Pad2)          = R12.2 R13.1
    unconnected-(R11-Pad1)  = R11.1

########## KiCad's netlist AFTER
    Net-(R11-Pad1)          = R11.1 R12.2 R13.1

########## and the rule, re-run on the repaired file
    KI-CONN-001 findings after the suggested command: 0
```

**The fix hint is provably the fix**, by KiCad's own answer and not by kicli's.

## Saturation, declared — and the declaration is that it is meaningless here

`Saturation::NEVER`, with the reasoning in the rule's own file. Two reasons, and
the brief invited the first:

1. **A blocking rule fails the gate on its first finding.** `Gate::of` branches
   on `Tier::One` and never reads a Tier 1 rule's saturation at all. One pin in
   a thousand or every pin on the sheet gives the same verdict. Saturation
   exists to catch a *normalised* rule whose cost is capped however often it
   fires; a blocking rule has no cost to cap. **So for a Tier 1 rule the
   declaration is genuinely empty**, and that is reported rather than dressed up.
2. **`Counted` cannot express what this rule counts.** It offers `Symbols`,
   `Wires` and `Nothing`. This rule counts **pin connection points**. Declaring
   `Wires` would be a false denominator, so it declares nothing.

## The label-plus-pin case: checked, not assumed

The goal state claims it needs no code of its own. **Measured** —
`a_pin_sharing_a_mid_wire_anchor_with_a_label_is_reported` builds the cluster
from `research/notes/label-on-wire-interior.md` (a wire with a resistor pin at
each end, a mid-span pin, and a local label sharing that anchor), and the rule
reports it. Not one line of this rule mentions a label. The claim holds.

## Two probe premises were wrong, and KiCad corrected both

Recorded because the corrections are the evidence that the probes were measured
rather than reasoned.

1. The name-merge probe first used a local label `GND` and a ground symbol
   placed the ordinary way. It fired, which looked like a false positive.
   `kicad-cli` on that exact file wrote `(name "/GND")` with only the two
   resistor pins on it. **`Placed::new` values a placement after its reference
   designator**, so the "ground symbol" was driving a net called `#PWR01` — a
   different drawing that happened to look like the intended one. kicli was
   right and the probe was wrong.
2. Given a `Value` of `GND`, the same drawing has KiCad writing `(name "GND")`
   with the power symbol joined, so the ground symbol **does** merge with a
   hand-written local label of the same name on the same sheet. The check now
   asserts silence there, and it is a second negative case reached through a
   different merge — a name meeting a name, rather than geometry.

## Where this rule under-reports, and why that is the safe direction

The wire's net is known only when the partition lists a pin at an end of the
wire's end-to-end chain. Three cases give no answer, and in each the rule
reports **nothing**:

- a wire whose chain reaches no listed pin — `names.rs::draft` returns `None`
  for a class with no pins, so such a net is not in `Nets` at all;
- a **sheet pin**. `draft` reads a `NodeKind::SheetPin` as a *name driver* only
  and never builds a `NetPin` from one, so the partition cannot say which net a
  sheet pin is on. The published rule says sheet pins are covered the same way;
  **they are not covered, and this is the gap.** It is reported below as
  something the extractor owes rather than asserted by a check, because a check
  that asserts a limitation is a check that resists its own repair;
- a symbol with no library definition, or none with an instance record on this
  sheet path. Both are the same omission a netlist makes.

Every one of these is a **missed finding and never a false one**. That is
deliberate: a pin at a wire's end is an ordinary connection, and a Tier 1 rule
that blocked a build on one would be the `KI-FLOW-001` backwards-list mistake in
a worse place.

## The oracle: all five arms ran, 35 of 35

**This task touches connectivity's *interpretation*, not its computation.** Not
one line of `crates/kicli/src/connectivity/` was changed, so the partition this
rule reads is byte-for-byte the partition the milestone already gates on. The
oracle was run anyway, per `.claude/skills/oracle-check/SKILL.md`, because a
rule that stands on the extractor is a reason to re-measure the extractor rather
than an excuse not to.

`kicad-cli` **10.0.5** is installed at `/opt/homebrew/bin/kicad-cli`. The first
run left three arms skipped for want of the corpus, so `cargo xtask corpus` was
run and the oracle re-run with every arm live:

```
$ KICLI_TEST_KICAD_CLI=/opt/homebrew/bin/kicad-cli \
    cargo test -p kicli --features corpus --test net_oracle -- --nocapture
test netlist_partition_matches_kicad ... ok
test netlist_oracle_is_current ... ok
test corpus::every_corpus_hierarchy_loads ... ok
test corpus::no_corpus_hierarchy_mixes_bundle_kinds ... ok
test corpus::netlist_partition_matches_kicad_corpus ... ok
hierarchies matched: 35/35
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**Five arms of five, and 35 of 35.** `netlist_oracle_is_current` regenerated the
committed netlist with `kicad-cli` and found it not stale, so the committed
oracle this task's expectation is derived from is current as of this run.

Per `CLAUDE.md`, a corpus- and environment-gated arm **does not count toward
done from inside a lane**. This is the measurement the task owed, not a claim
that the lane is green on it; the merged run is the orchestrator's.

## The corpus sweep, and its control — which is the more interesting number

The rule was run over all 35 corpus projects.

```
CORPUS SWEEP: 35 projects, 0 KI-CONN-001 findings
```

**A zero says nothing without a control**, so the coincidences the sweep had to
judge were counted:

```
CONTROL over 35 corpus projects:
  pin connection points examined            22072
  pins lying ON a wire, ends included       18544
  pins on a wire's INTERIOR (ends excluded) 0
```

Three things follow, and the third is the one worth keeping.

1. **No false positives on 35 drawings KiCad itself ships**, over 22,072 pin
   connection points.
2. **The zero findings is vacuous as evidence about the net comparison**: the
   geometry never arose. Not one pin in the corpus sits on a wire's interior,
   which is unsurprising — these are drawings a tool laid out, and the defect
   this rule catches is one a human makes. The rule's net comparison is
   therefore exercised by the committed fixture and the four probes, and by
   nothing in the corpus. Said plainly rather than left for a reader to notice.
3. **It is not vacuous about the geometry, and it explains B1 with numbers.**
   18,544 of those 22,072 pins lie *on* a wire — they are the ordinary
   connections every schematic is made of. The endpoint exclusion is the whole
   of the difference between 18,544 candidates and 0. And the net comparison
   would have suppressed all 18,544 anyway, because every one of them really is
   on its wire's net. **Two independent mechanisms reach the same answer on
   18,544 real cases, which is exactly why deleting either one is invisible from
   outside.** That is the B1 result, quantified.

## What the extractor owes, reported rather than written

`crates/kicli/src/connectivity/` is OUT of this lane's scope, *"because the
netlist oracle stands on it."* It was not touched. Three debts, in the order a
follow-up should take them.

### 1. `Nets` cannot say which net a non-pin item is on

`Nets` publishes `warnings()`, `nets()` and `net_of(reference, number)`, and a
`Net` lists `NetPin`s. **A wire, a junction, a label and a sheet pin are not in
it at all.** `KI-CONN-001`'s published test is *the pin's net against the wire's
net*, so the rule has to reach the wire's net the long way round — through a pin
at an end of the wire's end-to-end chain — and that is where all three of its
under-reporting cases come from.

**The narrow repair, and it needs no item identity.** `graph::Node` already
carries `points` and `segment`. A net could publish the connection points it
occupies on each sheet, which makes the query

```rust
Nets::net_at(sheet: &SheetPath, point: Point) -> Option<&Net>
```

and that answers **both** sides of this rule exactly: a wire's net is
`net_at(sheet, wire.from)`, because a wire's connection points are its own ends.
It also removes the end-to-end chain walk from the rule file entirely, which is
the only piece of connectivity reasoning this lane had to write.

The wider repair — item identity on `NodeKind::Line`, so a net lists the wires
on it — is a bigger change and this rule does not need it.

### 2. A sheet pin is not on any net the partition publishes

`names.rs::draft` reads a `NodeKind::SheetPin` as a **name driver** only and
never builds a `NetPin` from one, and `draft` returns `None` for a class with no
pins at all. So the partition cannot be asked which net a sheet pin is on.

**The published rule says sheet pins are covered the same way. They are not.**
This is the one goal-state clause this task does not meet, it is stated here
rather than glossed, and `net_at` above would close it in the same change.

### 3. `Counted` has no denominator for pin connection points

`crate::lint::gate::Counted` offers `Symbols`, `Wires` and `Nothing`. A rule that
counts pin connection points cannot declare its denominator honestly. It does
not bite here — a Tier 1 rule's saturation is never read — but the first Tier 2
rule that counts pins will meet it. `gate.rs` is T4's file, not this lane's.

## What the merge hotspots owe

None was touched. `Cargo.toml`, `lib.rs`, `build.rs`, `spec/SPEC.md`,
`tests/command_surface.rs` and `kicli.toml` need nothing from this task — the
registration seam held exactly as T1's PASS said it would, and `lib.rs` already
declares `pub mod lint;`.

**`AGENT.md` owes nothing yet and will owe something soon.** There is no `sch
score` verb, so there is no worked example to regenerate. When
`phase2-sch-score-command-surface.md` builds it, **that lane must attach the
partition to every `Drawing` it builds** — `Drawing::read` alone leaves this
rule silent, and a gate that reports `pass` on a check that never ran is the
worst output this milestone can produce, in that entry's own words. The hazard
is executable: `a_drawing_with_no_partition_attached_cannot_answer_and_says_nothing`
in `tests/lint_pin_on_wire.rs` states it as a measured property.

## Provenance of the tier, recorded because it is rare

**`KI-CONN-001` is one of only four rules in the 28-rule catalogue whose TIER has
published support.** `lane-t5` measured 24 of 28 as asserted rather than argued.
This one is argued: Olin Lathrop, on the junction dot — *"It's a rule. We don't
care whether you think it's silly or not. That's how it's done."* Recorded here
and in the rule file's own header, per the brief.

**Overlap with ERC: none**, and that is the point of the rule. KiCad's 47 checks
have nothing for it; the netlister reports two nets, which from its point of
view is not a violation. Confirmed by the measurement above: `kicad-cli` on the
committed fixture reports `Net-(R12-Pad2)` and `unconnected-(R11-Pad1)` without
complaint, and the drawing shows one connection.

## The `Drawing` edit, in full

Thirty-six added lines, of which twenty are documentation, in
`crates/kicli/src/lint/drawing.rs`:

```rust
pub struct Drawing<'a> { …, nets: Option<&'a Nets> }

pub fn with_nets(self, nets: &'a Nets) -> Self
pub fn nets(&self) -> Option<&'a Nets>
```

**Additive on purpose.** `Drawing::read` keeps its signature and its six
existing call sites — all of them in `lint_*` test binaries owned by T1, T3 and
T4, and one of them in a file a live lane may also be editing. Changing the
constructor would have put this lane inside three other tasks' files for no gain.

**An option rather than an empty partition**, so a rule must handle the absence
rather than mistake it for an answer. The module header's promise is kept: a
drawing still carries no file name and no way to reach the disk, and `Nets` is a
value the caller computed. `the_linter_holds_no_write_path.rs` already lists
`connectivity` in `CRATE_PATHS` — the linter was permitted to read a partition
before this task and simply had no way to be handed one. That check, and
`the_linter_holds_no_floating_point`, `the_linter_iterates_no_hash_map` and
`rule_files_are_formatted`, all pass on the result.

## Every check this task added, and the break that kills it

| Check | Break that kills it |
|---|---|
| `an_end_is_not_the_interior_and_one_unit_along_from_it_is` | B1 |
| `a_point_off_the_wire_is_not_on_its_interior` | B1 (partly), and any change to `on_segment` |
| `a_wire_of_no_length_has_no_interior` | B1 |
| `a_diagonal_wire_has_an_interior_too` | B4-class changes to the predicate |
| `the_committed_fixture_answers_both_directions_as_kicad_does` | B2, B3, B4, B7 |
| `the_finding_names_the_pin_the_wire_and_the_junction_that_repairs_it` | B2, B3, B4, **B5**, B7 |
| `a_drawing_with_no_partition_attached_cannot_answer_and_says_nothing` | B2, B3, B4, B7 |
| `the_endpoint_boundary_is_exact_from_both_sides` | B3, B4, B7 |
| `a_pin_sharing_a_mid_wire_anchor_with_a_label_is_reported` | B3, B4, B7 |
| `a_mid_wire_pin_already_on_the_wires_net_by_name_is_not_reported` | B2, B3, B4, **B6**, B7 |
| `a_ground_symbol_on_a_locally_labelled_wire_joins_it_and_is_not_reported` | B2, **B6** |

Every check has at least one break that turns it red, and the two breaks that
matter most — B3, the `opening-1` degenerate-equality break, and B6, the
plausible wrong rule — are each caught by checks written for them.

## Two notes from the gate itself

### The seam's known cost was paid, and its repair fired on real work

`cargo xtask check` failed the first time on **two** formatting complaints, and
they came from two different mechanisms:

```
=== fmt ===
Diff in crates/kicli/tests/lint_pin_on_wire.rs:164:
=== test ===
test every_hidden_rule_file_is_formatted ... FAILED
```

`cargo fmt --check` saw the **test** file and was blind to the **rule** file, as
the ratified seam verdict said it would be. `rule_files_are_formatted.rs` is the
only thing that saw `pin_on_wire.rs`, and it caught it. **The repair check is
not theoretical: it fired on the first rule ever written through this seam.**
Both were repaired with `cargo fmt` and a direct `rustfmt` on the rule file.

### One correctness change made after the first break series, and the series re-run

`first_wire_crossed` originally took the first wire in file order whose interior
the pin lands on, and compared nets afterwards. On a pin sitting where two wires
cross, a wire it *is* connected to could hide one it is not, purely by file
order. It is now `wire_it_is_not_on`, which asks both conditions together.

**The whole break series was re-run against the final code**, and every row of
the table above reproduced exactly — B1 at 2 of 4 unit checks and 0 of 7
end-to-end, B2 5, B3 6, B4 6, B5 1, B6 2, B7 6. The table is the second run's
numbers, not the first's.

## Completion check, run and pasted

```
$ export PATH="$HOME/.cargo/bin:$PATH"
$ cargo xtask check
=== fmt ===
=== clippy ===
=== test ===
=== doc ===
=== deny ===
=== clean ===
=== summary ===
all gates passed
exit=0

$ cargo test -p kicli --test rule_files_are_formatted
test every_hidden_rule_file_is_formatted ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**The corpus is present in this worktree** (`cargo xtask corpus` was run to make
the oracle's measurement), so this gate ran with the corpus arms live rather
than skipped. It still does not count toward done from inside the lane; the
merged run in the main checkout is the orchestrator's.

## Goal state, clause by clause

| Clause | State |
|---|---|
| 1. the shipped fixture, both directions, plus a fixture of my own | **met** — and the expectation comes from KiCad's netlist, not from kicli. Four probes rather than a committed fixture, so no `MANIFEST` line was needed |
| 2. the interior test excludes endpoints by 1 IU, in integers | **met** — and the exclusion's redundancy with the net test is measured and recorded rather than left implicit |
| 3. the gate's answer is the netlist's answer; the oracle run | **met** — 5 arms of 5, 35 of 35, `kicad-cli` 10.0.5. Interpretation, not computation: `connectivity/` untouched |
| 4. the fix hint is exact, and not a layout decision | **met** — produced by the rule, run verbatim by the built binary, and KiCad's netlist shows the repair. No move is ever suggested |
| 5. saturation declared | **met** — `NEVER`, with the reason it is meaningless for a Tier 1 rule stated rather than dressed up, and the `Counted` gap reported |
| the label-plus-pin case fires with no code for it | **met, and checked rather than assumed** |
| sheet pins covered the same way | **NOT met.** The partition does not list a sheet pin. Reported above as the extractor's debt; this is the one clause outstanding |
## Tick — APPROVE

Lane `0968365`, base `18ab930`. Merged on the resumption after the usage limit.

**The reviewer refuted the lane's reasoning and upheld its verdict**, which is
the most useful outcome a review can have. The lane defended its
degenerate-equality risk with *"the signal is their difference, and no break
moves a difference the way it moves an equality."* **That is false**, and the
reviewer showed it: `chain_nets` derives the wire's net from the same
`Located.net` field the pin side reads, so the `opening-1` break collapses
**both** sides. It is caught anyway — **because the checks' expectations come
from an independent oracle (KiCad's own netlist via `Probe::partition`), not
from a second reading of kicli's union-find.** That is the real defence and it
is stronger than the one it replaces.

Reproduced by the reviewer: `opening-1` break **6 of 7** red; B2 **5 of 7**; B1
(endpoint exclusion deleted) **0 of 7** integration and **2 of 4** unit; B7
(hazard) **6 of 7** including the hazard check itself. Fix hint round-tripped
end to end against `kicad-cli`.

**Owed, and NOT closed by this tick:**
1. **Sheet pins are not covered** — `NodeKind::SheetPin` is a name driver only
   and never produces a `NetPin`. The catalogue says *"sheet pins are covered
   the same way"*; they are not. **The rule's rustdoc states the gap rather than
   claiming coverage**, which is why this is follow-up rather than a rejection.
2. **`Nets::net_at(sheet, point)`** — buildable from `Node.points` alone.
3. **`Counted` has no denominator for pin connection points.**
4. **The `sch score` lane must attach the partition to every `Drawing`**, or
   this rule is silent and `--gate` passes on a check that never ran. The hazard
   is executable.
