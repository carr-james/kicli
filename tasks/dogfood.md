# Dogfood defects

kicli's end user is an LLM agent, so an LLM agent tests it. A dogfood subagent
gets `AGENT.md`, the built binary and a short design brief — no source, no task
files, no spec — and attempts the brief cold. **Everything it fumbles is a
defect**: a command misused, a document misread, output that overflows or
confuses its context.

Defects are recorded here **verbatim**, then triaged like any finding — fixed,
PROPOSED, or recorded with the reason it stands. The verbatim rule is the point:
a defect summarised by the person who wrote the thing being tested is a defect
already half-explained away.

---

## Run 1 — 2026-08-15, M4 Phase 2 checkpoint 2. Dry run; gates nothing.

**Standing from M5; a dry run in M4**, per `CLAUDE.md`'s dogfood gate. This run
gates nothing, and every defect below is real anyway.

**Setup.** Sandbox at `/tmp/kicli-dogfood-XNUf9M`, outside the repository:
`AGENT.md`, the debug binary, and a copy of the `nets` fixture as `board/` — 21
symbols, 28 nets, one sheet. The agent was told to work only inside the sandbox
and specifically not to look for source, tests or spec.

**Brief.** Add a third resistor to `NET_A`, connected so it genuinely joins the
net "not merely so that it looks connected", verify it; then take it back out and
confirm the board is as before.

**Outcome: both halves achieved and verified.** The agent placed `R22`, drew a
wire from its pin, added a `NET_A` local label, and confirmed
`N NET_A=/NET_A: R1.1 R2.1 R22.1`. It then removed label, wire and symbol and
confirmed the board matched its original counts exactly. **It succeeded — and
found nine frictions doing it.** A successful run with a long friction list is
the expected shape of this exercise, not a contradiction.

### The defects, verbatim

> 1. **`--uuids` gave a useless handle on this board.** `kicli sch view --uuids`
>    documents: "Add the first eight characters of each object's identifier." In
>    this project every single object — 19 symbols, 2 power symbols, sheet ports
>    — printed the identical handle `@00000000`, e.g. `S R1 10k R @00000000` and
>    `S R22 10k R @00000000` right after placement. This is because every UUID in
>    the file is literally `00000000-0000-4000-8000-0300000000xx` (confirmed by
>    `grep '(uuid' board.kicad_sch`). This is a fixture artifact, not necessarily
>    a kicli bug — but it's exactly the kind of thing a real user hits: the doc's
>    own worked-example (`da5aa983`) implies distinct short prefixes are the
>    norm, and I could not use `--uuids` output at all to disambiguate or address
>    a specific object here. I had to fall back to full mutation-report handles
>    (`f8819360`, `906eceb2`) captured from write commands instead.
>
> 2. **No documented way to find a pin's drawn coordinates.** To wire a new
>    resistor into a net I needed to know where its pin actually sits, but
>    AGENT.md has no `sym pins`/`pin list`/`pin info` command and the layout view
>    (`L`, `F`, `T`, `W` records) never lists pin coordinates. I had to
>    reverse-engineer the offset by comparing R1's anchor (`50.80,50.80`) against
>    the label kicli itself had placed at R1's pin (`50.80,41.91` — an 8.89mm
>    offset) and assume R22, being the same symbol at the same angle, had the
>    same offset. This worked, but only because `--from-pin` internally resolved
>    the true point and reported it back to me in the wire-draw output
>    (`180.34,41.91..180.34,46.99`) — i.e., I only found out the guess was
>    slightly wrong (46.99 vs my assumed 50.80) *after* the fact, from a write
>    command's side effect, not from any read-only inspection command.
>
> 3. **Wire-report coordinate format contradicts AGENT.md's own example.** The
>    doc's worked example for `wire draw` shows:
>    `+ W 3300f00e (50.80,50.80) -> (63.50,50.80)`
>    with parenthesized points and a `->` arrow. What I actually got was:
>    `+ W 906eceb2 180.34,41.91..180.34,46.99`
>    — no parentheses, `..` instead of `->`, and (confusingly) the endpoints
>    appear to be in *reverse* order relative to the `routed` line directly above
>    it (`routed R22.1 -> 180.34,41.91`, i.e., from-pin to to-point), while the
>    `W` line reads `41.91..46.99`, i.e., to-point first, from-pin-resolved-point
>    second. I had to sit and reconcile these by hand; a first-time reader
>    trusting the doc's format literally would misparse this line or assume
>    something had gone wrong.
>
> 4. **`project info` and `sch view` report two different, undocumented "net
>    count" numbers, and nothing explains the relationship.** `project info` said
>    `nets 28`; `sch view` (connectivity) said
>    `# scope project sheets=1 sym=19 pwr=2 nets=10` alongside
>    `# 18 pin(s) join nothing`. Only by noticing `10 + 18 = 28` did I work out
>    that `project info`'s `nets` figure apparently includes every unconnected
>    pin as a singleton net, while `sch view`'s `nets=` counts only
>    named/multi-pin nets. AGENT.md documents neither figure's composition, and
>    nothing signposts that they're the same underlying concept counted two
>    different ways. This cost me a double-take and is exactly the kind of
>    surprise the brief asks me to flag.
>
> 5. **`sym delete`'s report doesn't say whether the shared symbol definition was
>    dropped.** AGENT.md says of `sym delete`: "The embedded definition stays if
>    another placement still uses it, and goes if none does." When I deleted R22
>    (whose `Test:R` definition was also used by R1 and R2), the entire output
>    was:
>    ```
>    - S R22 10k Test:R
>    checked: every invariant passed
>    ```
>    There is no line telling me which branch of that either/or happened. I have
>    to infer it did the right thing only because R1/R2 still render correctly,
>    not because the tool told me. For a case the docs specifically call out as a
>    two-way fork, I expected the report to say which way it went.
>
> 6. **The `--to-at` guess for `wire draw` is not snap-forgiving in the way other
>    verbs are.** AGENT.md is explicit that "a vertex off the grid is refused,
>    not snapped" for wires, unlike every other verb. I got lucky and picked a
>    coordinate that happened to be on-grid and axis-aligned with the resolved
>    pin, but the doc gives no way to *ask* what the legal/expected terminus is
>    before trying — I only find out post-hoc from the `routed`/`W` report, or
>    via a `blocked`/`invalid` refusal that costs a whole write attempt. This is
>    a documented design choice, not a bug, but combined with friction #2 (no
>    pin-location command) it means placing a first wire onto a new symbol is
>    trial-and-error by default.
>
> 7. **Brief ambiguity, not a tool defect, but worth recording since it changed
>    my actions:** the brief's step 2 header "Then take it back out" (referring
>    to "a third resistor" from step 1) is followed by "Remove the wiring you
>    added, and confirm the board is back to NET_A carrying what it carried
>    before" — which only mentions removing *wiring*, not the resistor symbol
>    itself. Read strictly, deleting the label and wire alone would satisfy the
>    literal instruction and leave `NET_A: R1.1 R2.1` exactly as before, while
>    leaving a dangling, fully-disconnected `R22` symbol on the sheet. I judged
>    that "take it back out" most naturally means removing the resistor too
>    (otherwise the board is not "back to what it carried before" — it carries
>    one extra unconnected part, and `project info`'s `nets` count would read 30
>    instead of 28). I deleted `R22` as well. This is my own judgement call under
>    an ambiguous instruction, flagged per the brief's own request to report
>    guesses.
>
> 8. **Minor: every invocation prints an unrelated `zoxide` shell-config warning
>    to stderr before any kicli output**, e.g.:
>    ```
>    zoxide: detected a possible configuration issue.
>    Please ensure that zoxide is initialized right at the end of your shell
>    configuration file...
>    ```
>    This is almost certainly local sandbox/shell noise rather than a kicli
>    defect, and I don't believe it originates from kicli itself — flagging only
>    because it appeared on every single command and had to be mentally filtered
>    out each time before reading real output, which is exactly the kind of
>    interleaved noise that would waste tokens/attention for an agent parsing
>    output programmatically if it were not filtered.
>
> 9. **The two-minute `kicad-cli` font-cache warm-up note is easy to miss and
>    fires unpredictably.** `project info` and `project check` both printed
>    `kicli: asking /opt/homebrew/bin/kicad-cli its version. The first KiCad run
>    on a machine builds the font cache. It can take over 120 seconds.` on every
>    run in my session (not just the first), which contradicts AGENT.md's
>    description under `project check`: "The first run on a machine can take over
>    two minutes; later runs take under a second." The message appeared
>    identically on repeated `project info`/`project check` calls late in my
>    session, not just the first one — either the "first run" caching AGENT.md
>    promises isn't happening for `project info` (only documented for
>    `project check`), or the warm-up note is printed regardless of whether the
>    cache was actually cold. I didn't measure wall-clock time closely enough to
>    be certain this is a real perf regression versus just a repeated-but-harmless
>    log line, so I'm flagging it as "message reappears where the doc implies it
>    shouldn't" rather than a confirmed timing bug.

### Triage — RATIFIED IN FULL by advisor ruling, 2026-08-15

The triage below was written by the orchestrator and **ratified in full**. Two
items were promoted at ratification: **D1 runs as a chore after the checkpoint**,
golden changes included as part of the change; **D2 goes to the M5 planning list
as a task**, noting that the answer already exists internally in
`route::terminal` (`Terminal::of_pin`) and is merely unexposed. D3–D6 proceed as
filed. Two standing instructions for the next run: **a clean shell environment**,
and **the brief-writer owns the brief-ambiguity lesson** — an ambiguous brief
spends the run on the brief rather than on the tool.

### Triage

Every defect gets one of three outcomes: fixed, PROPOSED, or recorded with the
reason it stands. Nothing is closed by being explained.

| # | Verdict | Where it goes |
|---|---|---|
| 1 | **Real, and already half-known.** | C5's second half — see D1 below |
| 2 | **Real, and the largest of the nine.** | D2, PROPOSED as a task |
| 3 | **Real defect, introduced by the verb surface (T16) today.** | D3 — **FIXED**, incidentally, by the label proposal (T13) |
| 4 | **Real documentation defect.** | D4, chore |
| 5 | **Real, small, and the doc invites it.** | D5, chore |
| 6 | Consequence of a ruled design choice, sharpened by #2 | folded into D2 |
| 7 | **Not a tool defect — the orchestrator's brief was ambiguous.** | recorded, stands |
| 8 | **Not a kicli defect — the sandbox's shell.** | recorded, stands |
| 9 | **Real, and it is two questions.** | D6, chore for the doc half |

**D1 — every fixture object still answers to one handle.** Defect 1 is the
**second half of C5**, which fixed the probe crate and explicitly scoped the
committed fixtures out, naming them "a known second half rather than swept in
silently". This run is that half arriving with a cost attached: the agent could
not use `--uuids` at all and fell back to reading handles out of write-command
reports. **PROPOSED: do the fixture half.** Recommendation: accept, as a chore
after the checkpoint — it moves goldens, which is why C5 held it back, and the
golden change is part of the change. Note the agent correctly diagnosed it as a
fixture artifact rather than a tool bug, which is the diagnosis C5 already
recorded.

#### D1 — Done, 2026-08-21

Lane `lane-d1`, base `42d5201`.

**The measurement, taken before anything was changed** (worktree at `42d5201`,
clean). Counted with
`grep -rhoE 'uuid "[^"]*"' crates/kicli/tests/fixtures/sch/` and, for the
identifier-shaped strings that are not `uuid` atoms — sheet instance `path`
fields, netlist `tstamps`, ERC JSON — with
`grep -rhoE '[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}'`.

| scope | `uuid` atoms | distinct 8-char handles |
|---|---|---|
| `tests/fixtures/sch/` **before** | 354 | 204 |
| `tests/fixtures/sch/` **after** | 354 | 354 |
| `tests/fixtures/` (whole tree, all identifier-shaped strings) **before** | 1307 | 203 |
| `tests/fixtures/` (whole tree, all identifier-shaped strings) **after** | 1307 | 1307 |

**C5's "151 atoms, 1 prefix" was verified rather than inherited, and it has
moved.** The tree has gained fixtures since C5 was written. Of the 354 `uuid`
atoms now under `sch/`, **203 are in `sch/routing/calibration.kicad_sch` and
already carry distinct handles** — that file was written by the probe crate
*after* C5's fix, so it is C5's fix visible in a committed artefact. The
remaining **151 atoms across the other nine `sch/` files all shared the handle
`00000000`**, which is C5's number exactly.

**Files affected — the blast radius as a fact.** 1105 of the 1307 distinct
identifier-shaped strings in the fixture tree carried the colliding `00000000`
prefix. They live in these files:

| fixture group | identifiers remapped | files |
|---|---|---|
| misc `sch/` | 41 | `sch/future_version.kicad_sch`, `sch/item_zoo.kicad_sch`, `sch/lib_name_redirect.kicad_sch`, `sch/multi_instance/channel.kicad_sch`, `sch/multi_instance/multi_instance.kicad_sch`, `sch/unreadable_coordinate.kicad_sch`, `sch/v9_legacy.kicad_sch` |
| `geometry/orientations` | 25 | `.kicad_sch`, `.expected`, `.erc.json` |
| `geometry/asymmetric` | 41 | `.kicad_sch`, `.expected`, `.erc.json` |
| `sch/nets` | 110 | `nets.kicad_sch`, `nets_channel.kicad_sch`, `nets.kicad_pro`, `nets.netlist` |
| `project/healthy` | 4 | `healthy.kicad_sch`, `stage.kicad_sch` |
| `project/broken` | 6 | `broken.kicad_sch`, `future.kicad_sch`, `commented.kicad_sch` |
| `project/cycle` | 4 | `cycle.kicad_sch`, `inner.kicad_sch` |
| `text/calibration` | 873 | `calibration.kicad_sch` |
| `sch/routing/calibration` | 1 | `calibration.kicad_sch` — its **root sheet** only |

**The last row is the one worth reading.** `sch/routing/calibration.kicad_sch`
was probe-written and its 202 object identifiers are already distinct, but its
**root sheet uuid is not**: the probe crate hard-codes
`const ROOT: &str = "00000000-0000-4000-8000-999999999999"` and
`const CHILD: &str = "00000000-0000-4000-8000-cccccccccccc"`
(`crates/kicli-probe/src/drawing.rs:15,18`), and C5's `{series:02x}{n:06x}`
change did not reach them. **Carried, not fixed here:** the probe crate is out
of this lane's scope, and a probe drawing *with a child sheet* gives its root
and its child sheet symbol the two distinct-but-both-`00000000`-prefixed
handles `00000000` — a collision **inside a single drawing**, which is exactly
what C5 set out to remove. Recorded as of `42d5201`; see the PROPOSED item at
the end of this entry.

**The rule the fixtures now follow, stated so a later fixture can follow it.**
Each identifier **keeps its old body** and gains a new leading field
`{series:02x}{n:06x}` — the format C5 gave the probe crate
(`crates/kicli-probe/src/drawing.rs`), so the fixtures and the instrument now
number handles the same way. `series` is the fixture group's existing series
byte **raised by 0x10**, which reserves `0x00`–`0x0f` for probe drawings (the
probe numbers its series from 1), and `n` counts that group's identifiers in
ascending order of the identifier they already had. `13000001-0000-4000-8000-030000000000`
is the nets fixture's root: handle `13000001`, body unchanged. Keeping the body
is what makes the diff readable — every changed line differs in its first eight
characters and nowhere else.

**Applied as one deterministic substitution over 1104 identifiers**, to the
fixture tree, the goldens and the test sources together — not by regenerating
fixtures and refreshing goldens from output. That ordering is the evidence: the
suite was green afterwards **without a single further edit**, which says the
only thing that changed anywhere was identifiers. A golden refreshed from output
could not have shown that.

##### The goldens that moved, and why

Four of the eleven. Each line of each diff is a leading-field substitution and
nothing else.

| golden | why it moved |
|---|---|
| `view_connectivity.golden` | prints the sheet path of each of the three sheets of `sch/nets`; three paths, five identifier occurrences |
| `view_layout.golden` | the same three sheet paths, as its `page` headers |
| `project_info_healthy.golden` | prints the sheet paths of `project/healthy`; two paths, three occurrences |
| `project_info_broken.golden` | prints the sheet paths of `project/broken`; three paths, five occurrences |

**Seven goldens did not move, and one group of them was checked rather than
assumed.** `project_check_healthy.golden` and `project_check_broken.golden`
print no identifier at all. All five `wire_contract_*.golden` are normalised by
`without_generated_identifiers` — the fix for the M4 T16 defect, where goldens
asserted identifiers derived from a seed containing the checkout's absolute
path. **The brief flagged a move in those as unexpected. They did not move**,
which is that normalisation doing its job over a change of exactly the kind it
was built for.

##### The one fixture left alone, which is a finding

`sch/routing/calibration.kicad_sch` was remapped and then **put back**. Its 202
object identifiers were already distinct — it is C5's fix visible in a committed
artefact — but its **root sheet uuid** is `00000000-0000-4000-8000-999999999999`,
which is the probe crate's `ROOT` constant, and
`route_calibration.rs::the_calibration_fixture_is_what_the_recipe_builds`
**byte-compares the committed fixture against what the probe recipe builds**.
Remapping the root broke that test — measured, not predicted:

```
test the_calibration_fixture_is_what_the_recipe_builds ... FAILED
route_calibration.rs:1584: the committed calibration fixture is not what the recipe builds
```

Changing the root therefore requires changing `crates/kicli-probe/src/drawing.rs`,
which this lane is scoped out of. **Reverting costs nothing here**: with every
other fixture remapped, `00000000` is now carried by exactly one object in the
whole tree, so it is distinct and the goal state holds. The residual is real but
it is the probe crate's, and it is filed below rather than reached for.

##### PROPOSED — the probe crate's two constants are C5's remaining half

`crates/kicli-probe/src/drawing.rs:15,18` hold
`const ROOT: &str = "00000000-0000-4000-8000-999999999999"` and
`const CHILD: &str = "00000000-0000-4000-8000-cccccccccccc"`. C5's
`{series:02x}{n:06x}` change reached `Probe::uuid` and not these. Consequence,
stated as the measurement it is: **a probe drawing with a child sheet gives two
objects the same handle `00000000`** — its root sheet and its child sheet
symbol — which is a collision *inside a single drawing*, the exact defect C5
exists to remove. No test addresses either by handle today, so nothing is
silently passing, which is also exactly what C5 said about the state it found.
**Recommendation: accept as a chore** — give the two constants leading fields in
the probe's own reserved range (`0x00`–`0x0f`), regenerate
`sch/routing/calibration.kicad_sch` through the recipe, and let
`the_calibration_fixture_is_what_the_recipe_builds` be the check. Not done here
because `crates/kicli-probe/**` is out of this lane's scope and reaching into it
is how a chore becomes a merge conflict. Recorded as of `42d5201`.

##### Scope: this went wider than the brief's IN list, and here is the reason

The brief scoped IN `crates/kicli/tests/fixtures/sch/**`. The work covered
**`crates/kicli/tests/fixtures/**`** — `geometry/`, `project/` and `text/` as
well. Two reasons, and the second is not an argument:

1. the named check is stated over "the committed fixture tree", and
   `geometry/orientations.kicad_sch`, `project/healthy/healthy.kicad_sch` and
   `text/calibration.kicad_sch` are committed schematic fixtures whose objects
   all answered to `00000000` too;
2. **the brief's own list of goldens expected to move includes
   `project_info_*`, which is driven by `fixtures/project/**` and by nothing
   under `sch/`.** The scope list and the golden list disagreed with each other;
   the brief's rule is that the check wins.

Also outside the letter of the brief: **twenty-three occurrences across nine test
sources** under `crates/kicli/tests/` name a fixture identifier in a `const` or a
literal (`edit_field_placement.rs`, `edit_field_reference.rs`,
`edit_field_visibility.rs`, `edit_mark.rs`, `edit_text.rs`, `invariants.rs`,
`item_model.rs`, `project_commands.rs`, `snapshot_hashes.rs`). They took the same
substitution; leaving them would have been leaving the suite broken.

**Not touched, and named rather than implied:** `crates/kicli/src/**`,
`crates/kicli-probe/**`, and `crates/kicli-sexpr/tests/fixtures/**`. The last is
a deliberate exclusion, not an oversight — those are byte-fidelity fixtures for
the s-expression parser, they belong to another crate's root and another
crate's `MANIFEST`, and a handle means nothing to a layer that does not know
what a schematic is.

##### The checks

New file `crates/kicli/tests/fixture_handles.rs`, three tests.

- **`every_committed_fixture_object_answers_to_a_handle_of_its_own`** — the
  arithmetic. Over the whole committed fixture tree: the number of distinct
  handles equals the number of `uuid` atoms (1307 = 1307), and, as a wider
  second assertion, equals the number of distinct identifier-shaped strings
  *anywhere* in the tree — a sheet instance `path`, a netlist `tstamps` and an
  ERC report's JSON all name objects, and a handle that collides there collides
  just as badly. Every handle comes from `Uuid::short`, never from a private
  cut, so `the_handle_has_one_name` gains no new accounted-for site.
- **`a_fixture_object_is_addressed_by_its_handle_and_found`** — the capability.
  For **every** identifier-carrying object of `sch/nets/nets.kicad_sch` and
  `sch/item_zoo.kicad_sch`, the handle a view would print is handed to
  `cli::edit::address::item` and must return that same object. Two fixtures
  because C5's own first pass passed a single-sheet control and failed a
  multi-sheet one.
- **`a_handle_no_fixture_object_carries_is_refused`** — the other half of the
  capability, so addressing succeeds because the object is there rather than
  because the resolver accepts anything.

##### Falsification

Good state committed at `6ae0c21` before any break. Evidence anchored to
`shasum` rather than to commits: the restored good state is
`4313c4f5998142379e7917b18ac45318ac8ebaea  crates/kicli/tests/fixture_handles.rs`
and `0d1191bcb4171ee7e336aa9506b0194dcadb1cc4  crates/kicli/tests/fixtures/sch/nets/nets.kicad_sch`,
verified after each restore.

| # | what was broken | result |
|---|---|---|
| 1 | Two fixture objects made to share a handle: in `nets.kicad_sch`, symbol `13000006-…-030000000004`'s leading field set to `13000005`, colliding with the pin `13000005-…-030000000005`. | **caught** by the atom assertion of `every_committed_fixture_object_…`: "1307 atoms share 1306 handles", naming both sharers. Capability test **green** — see below. |
| 2 | Two **top-level** objects made to share a handle: wire `13000009-…-030000000008` → `13000008`, colliding with wire `13000008-…-030000000007`. | **caught by both**. The capability test reports kicli's own refusal: `13000008 names 2 objects of this sheet`. |
| 3a | The sweep made blind — `walk` returns before reading any entry — **and every presence control removed** (the `files.len() >= 20` assertion, the `NAMED` loop, and both anti-vacuity floors). | **PASSED, green, on zero files.** This is the blind check, demonstrated rather than asserted. |
| 3b | The identical blind `walk`, controls **restored**. | **caught**: "the fixture tree was read: 0 files under …/tests/fixtures". The contrast between 3a and 3b is what shows the control is load-bearing and not decoration. |
| 4 | The sweep pointed at the **wrong root** — `tests/` instead of `tests/fixtures`, so the `files.len() >= 20` floor still clears. | **caught** by the `NAMED` control: `sch/nets/nets.kicad_sch was read and holds the identifiers D1 measured`. A file-count floor alone would have waved this through. |
| 5 | **The defect itself.** The pre-D1 fixture tree (`git show 42d5201:…`) restored under the new tests, in a second directory. | **both checks caught it**: "1307 atoms share 203 handles", and the capability test carrying kicli's own words — `00000000 names 51 objects of this sheet: …`. That message is the dogfood defect verbatim. |

**Break 1 left the capability test green, and that was investigated rather than
recorded as a pass.** It is falsification-control case 1, not case 2: the two
identifiers made to collide were a **symbol** and a **symbol pin**, and
`address::item` searches `schematic.items` — the top-level objects — so no
ambiguity existed in the set that check addresses. Break 2 is the control that
tells the two cases apart: the same break made between two **top-level** wires
turns the capability test red immediately. Case 2 was ruled out by measurement,
not by reasoning about it.

##### Environment variation — the break class the source breaks cannot reach

Per the skill's own worked example (the T16 golden defect, where identifiers
were a hash of a seed containing the checkout's absolute path), the whole suite
was **run once from a second absolute path**: the tracked tree was extracted
with `git archive` into
`…/scratchpad/second-directory` and run there with `KICLI_TEST_KICAD_CLI=1`.

- **599 tests pass**, the same set as in the lane worktree.
- The fixture tree's digest is **byte-identical** in both directories, before
  and after the run: `e363e3c5b1a5dd4675cdf522c017c644f44ddc24`.
- Distinct handles: **1307 in both**.

A committed fixture cannot depend on where the checkout is, and now that is
measured rather than assumed.

##### The oracle check

Connectivity-touching, so it is owed. Identifiers are what a sheet instance path
and a netlist `tstamps` are made of, so **KiCad was asked about the rewritten
files rather than told about them**. With `KICLI_TEST_KICAD_CLI=1`, the whole
workspace suite passes — including the tests that regenerate an oracle with
`kicad-cli` and compare it to the committed bytes:

- `oracles_are_current` — the geometry ERC reports, regenerated by KiCad;
- `netlist_oracle_is_current` and `netlist_partition_matches_kicad` — `sch/nets`;
- `the_calibration_oracle_is_current` and
  `the_calibration_fixture_partitions_as_kicad_says` — `sch/routing`;
- `the_calibration_fixture_is_what_the_recipe_builds`;
- `a_canonical_file_writes_without_reformatting` and
  `prettify_reproduces_kicad_layout` — so `canonical yes` in the `MANIFEST`
  still holds for every record that claims it.

**This is why the `MANIFEST`'s `kicad-cli` provenance still stands after a hand
substitution**, and the `MANIFEST` now says so in a comment rather than leaving
a reader to wonder: KiCad re-derives these bytes and agrees with them.

##### Check evidence

- `cargo test --test fixture_handles`: **3 passed**.
- `cargo test --workspace`: **pass**, no test edited beyond the identifier
  substitution.
- `KICLI_TEST_KICAD_CLI=1 cargo test --workspace`: **pass**, 599 ok. Environment-
  gated, so it counts as the measurement this entry owes and **not** toward done
  — the orchestrator's merged run is what counts.
- `cargo xtask check`: **all six gates pass** (fmt, clippy, test, doc, deny,
  clean), in the lane worktree.

**D2 — nothing read-only will tell an agent where a pin is.** The agent had to
infer a pin offset from a label kicli itself had placed, and learned its guess
was wrong only from a **write command's** output. Defect 6 is the same wound: a
wire vertex is refused rather than snapped — a ruled and correct choice — but
there is no way to ask what would be accepted, so a first wire onto a new symbol
is trial and error. **PROPOSED: a read-only way to ask where a symbol's pins
are**, as a task, not a chore. Recommendation: accept for M5 planning. This is a
design decision about the agent-facing surface and it deserves an entry rather
than a patch; note the router already resolves pins internally (`route::terminal`,
`Terminal::of_pin`), so the answer exists and is simply not exposed.

**D2 — CARRIED INTO M5, 2026-08-21.** The ratified promotion is executed: the
entry now also lives at `tasks/M5.md`. Closed for M4 by being carried, not by
being done — it is a design decision about the agent-facing surface, and M4 ships
no read-only pin query. Defect 6 travels with it, being the same wound from the
other side.

**D3 — FIXED, before the chore was ever run.** The label proposal (T13) was
editing `AGENT.md` for `--auto-labels`, hit the same wrong examples, and corrected
them as a disclosed incidental change in the same commit — its tick reviewer
confirmed the correction and judged it in scope. **Verified after the fact: all
four `W` examples in `AGENT.md` now carry the `..` form the tool actually
produces** (`:455`, `:494`, `:495`, `:554`). The chore below is retained as the
record of what the defect was and how it was found, not as outstanding work.

**D3 — `AGENT.md` documents a wire delta format the tool does not produce.**
**Verified at source before filing**, not taken on the agent's word.
`AGENT.md:455` and `:515` show `+ W 3300f00e (50.80,50.80) -> (63.50,50.80)`;
`crates/kicli/src/view/snapshot.rs:781` formats it `format!("{}..{}", …)`, and
that line predates M4 — it is the delta format M3 shipped. So the examples the
verb surface (T16) added today describe a format **kicli has never produced**.

This one is worth more than its size. **The tick review for T16 approved on a
diff and a check set; this defect lives in the gap between a document and the
tool's actual behaviour, which is precisely what a check set does not cover and
what a cold reader hits first.** `agent_doc_covers_every_command` asserts a
mention (chore C7), and even a fixed version asserting a heading would not have
caught it. **Chore, chore-runner eligible**: correct both examples against
measured output. The second half of the agent's report — that the `W` line's
endpoints read in the line record's order rather than the request's order — is
**also true and also undocumented**, and the corrected example should say so.

**D4 — two net counts, one concept, no signpost.** `project info` says `nets 28`;
`sch view` says `nets=10` with `18 pin(s) join nothing`. The agent worked out
`10 + 18 = 28` unaided, which is the good outcome; the bad one is that it had to.
**Chore**: document what each figure counts, at both places.

#### D4 — Done, 2026-08-21

**Measured first, written second.** Both numbers were read out of the code and
then out of a running view, before any prose was written.

- **`project info`'s `nets N`** is `nets.nets().len()`
  (`crates/kicli/src/cli/project.rs`, `write_nets`), the whole partition from
  `connectivity::extract`: **every net of the whole project**, all sheets, power
  nets included, single-pin unconnected nets included. Nothing is filtered.
- **`sch view --view connectivity`'s `nets=N`** is not a count of nets at all —
  it is the number of `N` records the view emitted, counted back out of its own
  text (`crates/kicli/src/view/connectivity.rs:96`). A net is left out of it two
  ways: one with exactly one visible pin whose KiCad name starts `unconnected-`
  is **tallied** as `# N pin(s) join nothing` instead of listed
  (`connectivity.rs:271-274`), and one with **no** visible pin at all — every pin
  a power pin without `--include-power`, or every pin on another sheet under
  `--sheet` — is dropped in silence (`connectivity.rs:265-267`).

**So the dogfood agent's `10 + 18 = 28` is right, and is not a general law.** The
exact statement is `listed + tallied + hidden = total`, and it is the third term
that the agent's arithmetic did not have to reckon with. Measured on the `nets`
fixture, whole project: `14 + 18 + 0 = 32`, and `project info` on it reports
`nets 32`. Measured on the **root sheet** of that same fixture: `10 + 18 = 28`
against a project total of 32 — the per-sheet view does not reconcile with the
project figure and must not be documented as if it did. **This is why the chore
said to measure before writing: the obvious signpost, "they add up", is false
under `--sheet`, and `--sheet` is exactly what an agent reaches for on a large
project.**

**Where the signpost went.** Beside both numbers, in the document the end user
reads:

- `AGENT.md`, the `kicli project info` section — and the sample output there was
  **missing the `nets` line entirely**, which is its own small part of why there
  was no signpost to find. Added, from the golden
  (`crates/kicli/tests/project_info_healthy.golden:6`), plus a paragraph saying
  the count is whole-project and pointing at the reconciliation.
- `AGENT.md`, `#### The connectivity view` — the reconciliation itself, both
  omission rules, the worked numbers above, and the per-sheet warning.
- `crates/kicli/src/cli/project.rs`, rustdoc on `write_nets`, for the next person
  who changes the number rather than reads it.

The view's own printer is in `crates/kicli/src/view/connectivity.rs`, which
another lane held open at the time; its rustdoc was left alone deliberately
rather than risk the conflict, and the omission rules are commented there
already.

**Check:** `crates/kicli/tests/net_counts_reconcile.rs`, two tests.
`the_whole_project_view_accounts_for_every_net` walks every `.kicad_sch` under
`tests/fixtures`, renders the default whole-project view, reads `listed` and
`tallied` back out of the view's **own text** rather than recomputing them, and
asserts `listed + tallied + hidden == total`. Reading the text matters: a
recomputation would be a second implementation of kicli agreeing with the first.
`a_per_sheet_view_does_not_account_for_the_project` pins the other half — if a
per-sheet view ever did add up, the document's warning would be wrong and this
is what would notice.

**Falsification.** Six breaks against the checkpointed good state; `shasum -a
256` after each restore returned `net_counts_reconcile.rs` to `f6f1f431…` every
time.

| # | The break | Result |
|---|---|---|
| A | The `tallied` term dropped from the identity | **FAILED** on `geometry/asymmetric.kicad_sch`: lists 0, tallies 32 |
| B | The `listed` term dropped | **FAILED** on `sch/item_zoo.kicad_sch`: lists 1, tallies 1, total 2 |
| C | The tally reader made blind — always returns 0 | **FAILED**, and it is the reader being wrong rather than the tool |
| D | The fixture root pointed at a directory that does not exist | **FAILED**: "only 0 fixture projects have nets, which is too few to have checked anything" |
| E | The per-sheet claim inverted, `<` to `>=` | **FAILED**: "the root sheet accounts for 28 of 32 nets" |
| F | The `hidden` term dropped | **ok** — see below |

**Break F is a no-op break, not a blind check, and the difference was
established rather than assumed.** `hidden` is **0 on every fixture**, measured
across all seven projects that have nets, so removing the term changes no
arithmetic anywhere and the identity is arithmetically identical with and
without it. The term is read from the code (`connectivity.rs:265-267`), not from a
fixture. **PROPOSED: no fixture exercises a net with no visible pin** — a net of
only power pins under the default view, or a whole net off-sheet under
`--sheet`. Recommendation: leave it. The term is correct, cheap, and self-
documenting, and manufacturing a fixture to exercise it belongs to whoever owns
`tests/fixtures/` rather than to a documentation chore. Recorded so that a later
reader does not mistake break F for evidence the term is tested.

**Confirmed end-to-end at the command line**, added while measuring D6 on the
same machine and recorded here because it belongs to this defect. On a copy of
the `nets` fixture, with a real `kicad-cli` present:

```
$ kicli --project <dir> project info   # stdout
  nets       32
$ kicli --project <dir> sch view       # stdout
# scope project  sheets=3 sym=23 pwr=4 nets=14
# 18 pin(s) join nothing; sch erc lists them
```

`14 + 18 = 32`, from the binary rather than from the library, which is the form
an agent actually meets and the form `AGENT.md` now documents.

**`cargo xtask check`: all six gates pass.** `cargo test` green throughout;
nothing kicli prints was changed, so no golden moved.

**D5 — `sym delete` reports a two-way fork without saying which way it went.**
`AGENT.md` specifically calls out that the embedded definition "stays if another
placement still uses it, and goes if none does", and the report says neither.
**Chore**: say which. The doc raising the question is what makes the silence a
defect.

#### D5 — Done, 2026-08-21

**The mechanism the codebase already had.** `delete_symbol` returned
`Edited { symbol, findings }` with `findings: Vec::new()`, and every other
`sym` verb renders its findings through `notes_of` into `note: <name>  <text>`
and the JSON `notes` array. The fork now returns exactly one of two new
`Finding` variants — `DefinitionKept` and `DefinitionRemoved` — and
`cli/edit/symbol.rs`'s `delete` passes `&notes_of(&edited.findings)` where it
passed `&[]`. No parallel channel was invented, and no new output key exists:
`notes` is the key `AGENT.md:659` already documents. One short line, per
Constitution §6; both output forms, per §8.

**Deliberate: the degenerate third case reports `DefinitionRemoved`, not a
third variant.** A sheet that embeds no definition under the key takes the
`!still_drawn` branch and removes nothing. The finding states the outcome — no
definition for that key remains on the sheet — which is true whether one was
removed or none was there. A third variant would be untested surface, since a
fixture for it would have to be built and `tests/fixtures/` was held by another
lane this session. Recorded in `delete_symbol`'s doc comment.

**The checks, derived from the defect rather than from the entry.**
`crates/kicli/tests/edit_symbol_delete_definition.rs`, three checks, all through
the compiled binary. `sch/nets/nets.kicad_sch` draws `Test:GND` through exactly
two placements, `#PWR01` and `#PWR02`, so deleting them in order gives the kept
arm and then the gone arm off one committed fixture. Each arm asserts the file
as well as the sentence — `embeds_definition()` greps the written sheet — so the
note is a report of what happened and not a constant. A third check pins the
fixture shape, because two arms are only two arms while that definition has two
placements.

Observed, `kicli --project <scratch> sym delete`, both arms in both forms:

```
- S #PWR01 GND Test:GND
checked: every invariant passed
note: definition-kept  another placement still draws Test:GND, so its embedded definition stays

- S #PWR02 GND Test:GND
checked: every invariant passed
note: definition-removed  no placement draws Test:GND now, so its embedded definition is gone
```

```json
[{"name": "definition-kept", "message": "another placement still draws Test:GND, so its embedded definition stays"}]
[{"name": "definition-removed", "message": "no placement draws Test:GND now, so its embedded definition is gone"}]
```

**Falsification, per `.claude/skills/falsification-control/`.** The good state
was committed first, and each break is anchored to the `shasum` of the file it
broke rather than to a commit SHA. Good state:
`6653ef1cf14bd865cda842ce3404851bc5b26242  crates/kicli/src/edit/symbol.rs`,
`9dbddbd75f58a0a1f78f24d1c3c6144d9529b01e  crates/kicli/src/cli/edit/symbol.rs`,
`c03c20697cfc3a39cafacad5dc73a4ad84fdfbc0  crates/kicli/tests/edit_symbol_delete_definition.rs`
— all three green, and all three restored to those hashes afterwards.

| Break | Broken file `shasum` | Result |
|---|---|---|
| Report the opposite fork, removal left correct | `91e6c41d…` (`edit/symbol.rs`) | both checks FAIL at arm one (lines 126, 163) |
| **Anti-vacuity A**: always report `DefinitionKept` | `8d21a64c…` (`edit/symbol.rs`) | both checks FAIL at **arm two** (lines 141, 180); `left: ["definition-kept"] right: ["definition-removed"]` |
| **Anti-vacuity B**: always report `DefinitionRemoved` | `55f56e5e…` (`edit/symbol.rs`) | both checks FAIL at **arm one**; `left: ["definition-removed"] right: ["definition-kept"]` |
| CLI wiring reverted to `&[]` | `2855434b…` (`cli/edit/symbol.rs`) | both checks FAIL; `left: [] right: ["definition-kept"]` |
| Fixture guard pointed at `Test:R` | — | guard FAILS, `left: 19 right: 2` |

The two anti-vacuity controls are the pair that matters: A fails only the gone
arm and B fails only the kept arm, so **no constant answer passes both**. The
first break alone would not have shown that, because a check that panics at arm
one never reaches arm two — which is why B is recorded separately rather than
treated as the same break inverted.

**`cargo xtask check`: all six gates pass** in the lane worktree (fmt, clippy,
test, doc, deny, clean), and again through the pre-commit hook. **No golden
moved** — no golden covers `sym delete`, and the gate confirms it.

**Owed, not done: `AGENT.md`.** Constitution §8 requires agent docs to move with
a command-surface change in the same change. This adds no noun, verb or flag —
it fills the documented `notes` array — so the §8 trigger is arguable, but the
`sym delete` section at `AGENT.md:358-365` states the fork and should now add
that the report names which way it went. `AGENT.md` was **out of this lane's
scope by the brief**, held by another lane this session, so this is reported as
work owed rather than done. **PROPOSED: a one-line addition to the `sym delete`
section.** Recommendation: fold into whichever lane next holds `AGENT.md`; it is
one sentence and needs no measurement.

**D6 — the font-cache note fires where the document implies it should not.** Two
questions, and the agent was right to separate them. The **documentation** half
is a chore: `AGENT.md` describes the warm-up under `project check` only, while
`project info` also invokes `kicad-cli`. The **timing** half — whether the cache
is genuinely cold each time or the note prints unconditionally — is a
measurement nobody has made, and the agent said so rather than guessing.
Recorded as unmeasured.

#### D6 — Done (documentation half), 2026-08-21

**Verified at source, then at runtime.** The note comes from
`crates/kicli/src/cli/tools.rs`, `probe`, printed through `Reporter::note` —
which writes to **standard error** with a `kicli: ` prefix and is silenced by
`--quiet` (`cli/output.rs:82-86`). `probe` has exactly **two** callers:
`cli/check.rs:65` (`project check`) and `cli/project.rs:65` (`project info`).
The document named only the first. The agent was right.

Observed black-box, `kicli --project <dir> project info`, stderr separated from
stdout:

```
kicli: asking /opt/homebrew/bin/kicad-cli its version. The first KiCad run on a machine builds the font cache. It can take over 120 seconds.
```

**The document now says where the note can appear.** `AGENT.md`'s
the `kicli project info` section carries the explanation — the note's own words, that
it goes to standard error and cannot corrupt a parse, that `--quiet` silences
it, and that it is a warning about what kicli is about to do rather than a
report of what happened. The `kicli project check` section names `kicad-cli`, says it
prints the same note, and points at the other section. The old sentence's claim
that `project check` "warms KiCad's font cache" is also gone: kicli warms
nothing, it asks `kicad-cli --version` and that ask is what may build the cache.

**Check:** `agent_doc_warns_about_the_kicad_cli_wait_in_both_places`, in
`crates/kicli/tests/agent_doc.rs`. Both sections must name `kicad-cli`, the
`project info` section must carry the font-cache explanation, and the
`project check` section must point at it.

**Falsification** — against the checkpointed good state; `shasum -a 256`
returned `AGENT.md` to `dece3025…` after every restore.

| # | The break | Result |
|---|---|---|
| G | The note deleted from the `project info` section — D6's original defect, put back | **FAILED**: "`project info` runs kicad-cli and blocks on it exactly as `project check` does" |
| H | The cross-reference removed from `project check` | **FAILED**: "it has to say where to look" |
| I | `kicad-cli` no longer named in the `project check` section | **FAILED**: "its section has to name it" |

**The timing half — measured incidentally, recorded, NOT acted on.** The lane
brief reserved this and the reservation is respected: nothing below changed any
code or any timing claim in `AGENT.md`.

- **The note prints unconditionally.** Two `project info` runs seconds apart on
  a machine whose cache was necessarily warm by the second: **the note printed
  both times, identically.** It fires whenever `Discovery::locate()` finds the
  binary, before `kicad-cli --version` is run; kicli never inspects the cache
  and has no way to. So of the agent's two candidate explanations, "the note
  prints unconditionally" is **confirmed** and "the cache is genuinely cold each
  time" is **refuted**.
- **A warm run costs 0.12s.** `kicli -q project info` on the `nets` fixture,
  three consecutive runs, `real 0.12` each. This **corroborates** `AGENT.md`'s
  standing "later runs take under a second", which was previously unmeasured;
  the sentence is left exactly as it was, now with a measurement behind it.
- **Still unmeasured:** the cold-cache cost. Nobody has run this on a machine
  with no KiCad font cache, so the "over two minutes" figure remains inherited
  rather than observed. Recorded so it is not mistaken for measured.

**PROPOSED:** the note is a warning that fires on every run of two commands, and
an agent that runs `project info` in a loop reads it every time. Recommendation:
leave it. It costs one stderr line, `--quiet` removes it, and the failure it
prevents — an agent concluding kicli has hung — is far more expensive than the
line. Raised only because the measurement above makes the tradeoff explicit for
the first time; not a defect, and not this chore's to change.

**7 — recorded, stands, and it is the orchestrator's defect not the tool's.** The
brief said "take it back out" and then described removing only the wiring. The
agent read the ambiguity, chose the reading that made the verification claim
true, and flagged the guess. That is the behaviour the exercise wants. **The
lesson is for whoever writes the next dogfood brief**: a brief that is ambiguous
spends the run's attention on the brief rather than on the tool.

**8 — recorded, stands, not a kicli defect.** The `zoxide` warning comes from the
sandbox's shell initialisation, inherited from the environment the orchestrator
prepared. It is noise the orchestrator introduced. Worth keeping in the record
for one reason the agent gives well: it is "exactly the kind of interleaved noise
that would waste tokens/attention for an agent parsing output programmatically".
**Next run's sandbox should start from a clean shell environment**, so the
exercise measures kicli's output and not the harness's.
