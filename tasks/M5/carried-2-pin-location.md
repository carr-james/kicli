# Carried in from M4 — D2, nothing read-only will tell an agent where a pin is ✅

*Migrated verbatim from the former `tasks/M5.md` at the M5 opening, by the
boundary-package ruling that gives M5 one file per task. The text below is the
record as M4 wrote it; nothing was re-argued in the move.*

**Provenance: the M4 dogfood run, defect 2, ratified in full by advisor ruling
2026-08-15 with the explicit promotion "D2 goes to the M5 planning list as a
task".** Full text in `tasks/dogfood.md`.

**What the dogfood agent actually hit.** It had to infer a pin offset from a
label kicli itself had placed, and learned its guess was wrong only from a
**write command's** output. Defect 6 of the same run is the same wound from the
other side: a wire vertex is refused rather than snapped — a ruled and correct
choice — but there is nothing to ask what *would* be accepted, so a first wire
onto a new symbol is trial and error.

**It is the largest of the nine defects that run found**, and it is a design
decision about the agent-facing surface rather than a patch: a read-only way to
ask where a symbol's pins are.

**The answer already exists internally and is simply unexposed** — the router
resolves pins in `route::terminal`, `Terminal::of_pin`. That is what makes this
cheap to build and expensive to design badly: the question is not how to compute
it but what an agent should be able to ask, and in what shape, under
Constitution §6's context budget.


---

# SCHEDULED — Phase 1, beside the spine

**Provenance: `tasks/M5/PLAN.md`, RATIFIED by James's ratification and advisor
rulings, M5 plan review.** The plan places this in Phase 1 *"as its own task"*.

**It is the one Phase 1 item that may run beside a spine task**, because its
file scope is disjoint from `lint/**` entirely. It does not block T1–T4 and they
do not block it.

## Why this is a design task with an implementation attached

The M4 record above says it: *"the question is not how to compute it but what an
agent should be able to ask, and in what shape."* `Terminal::of_pin` already
knows the answer. **Everything hard about this task is the surface.**

So the deliverable is not "a command that prints pin positions". It is **an
answer to a question an agent has, in a form an agent can act on** — and the
next thing that agent does with the answer is draw a wire to it.

## The two defects it must close, and they are one wound from two sides

- **D2**: an agent had to *infer* a pin offset from a label kicli itself had
  placed, and learned its guess was wrong only from a **write command's**
  output. Read-only questions should not be answered by write commands.
- **D6**: a wire vertex is **refused rather than snapped** — a ruled and correct
  choice — but **there is nothing to ask what *would* be accepted**, so a first
  wire onto a new symbol is trial and error.

**D6 is the sharper of the two and the easier to under-serve.** A command that
answers "where is pin 1" without answering "what may I connect to it" closes D2
and leaves D6 open, and the agent still guesses — one round-trip later than
before. Whatever you build, **check it against D6 explicitly** and say in the
entry whether it closes it.

## Constitution §6 governs the shape, and it has teeth here

*"Outputs are designed for LLM context budgets. A view that floods is wrong,
whatever it contains."* A 40-pin connector's every pin, printed by default,
is a flood. So the shape question includes **what you get without asking for
everything**, and `spec/SPEC.md` §7.4's budgets are the existing precedent —
`crates/kicli/tests/view_budgets.rs` is where budgets are already asserted.

Note that file currently carries two **dead** helper functions
(`connectivity_ceiling`, `layout_ceiling`, flagged by `cargo` as never used).
That is pre-existing and **not yours to clean** — report it, do not fix it.

## Where the answer already lives

`crates/kicli/src/route/terminal.rs`, `Terminal::of_pin`. `route` knows nothing
of files, the CLI or `kicad-cli` (`ENGINEERING.md`, Structure), and **it must
stay that way** — this task exposes what `route` computes; it does not move the
CLI into `route`.

`crates/kicli/tests/pin_positions.rs` and `route_terminals.rs` already measure
pin resolution. **Read both before adding a check**: whatever you build stands
on the same resolution they already exercise, and a new check that re-asserts
what they assert is a check with a shared ancestor.

## Goal state, as the checks that prove it

1. **A read-only command answers where a symbol's pins are**, in both the terse
   text and JSON forms, per the project's established view conventions.
   **It writes nothing** — and that is worth an executable check, not a comment.
2. **The answer is sufficient to draw a wire without a failed attempt.** The
   check is end-to-end and is the task's real completion criterion: ask where the
   pin is, use the answer to place a wire, and **the wire is accepted first
   time**. A check that only compares numbers to a fixture does not test the
   thing the defect was about.
3. **It does not flood.** A budget assertion in the shape `view_budgets.rs`
   already uses, over a realistically large symbol.
4. **`AGENT.md` documents it, with a worked example regenerated from a real run
   of the built binary** — Constitution §10 (*a feature undocumented for agents
   is unfinished*) and M5 `RULES.md`'s measured-examples rule. `AGENT.md` is a
   merge hotspot **held by one lane at a time**; the orchestrator schedules it.

## Falsification obligation

Per `.claude/skills/falsification-control/SKILL.md`.

- **Goal-state 2 is the one that can be blind.** If the wire placement in the
  check derives its coordinates from the same call the command uses, the two
  sides share an ancestor and the check passes on a scorer that is uniformly
  wrong. **State what each side derives from.** The strong form goes through the
  *printed output*, parsed back, which is what an agent actually does.
- **The writes-nothing check is falsified by making it write**: touch the file
  in the command path and confirm red.

## Scope

**IN**
- a new read-only view/command surface, in the modules its shape requires
- `crates/kicli/src/route/terminal.rs` — **read; change only if the exposure
  genuinely requires it, and say so**
- new test files under `crates/kicli/tests/`
- this file, for the evidence, written AS YOU WORK

**MERGE HOTSPOTS — report, do not edit.** `Cargo.toml`, `crates/kicli/src/lib.rs`,
the fixture `MANIFEST`, `AGENT.md`, `spec/SPEC.md`, `crates/kicli/tests/command_surface.rs`.
`AGENT.md` and `command_surface.rs` both owe this command a line; **report what
they owe** and the orchestrator sequences it.

**OUT** — `crates/kicli/src/lint/**` (Phase 1's spine lanes own it), every other
entry, `tasks/M5/PLAN.md`, the two dead helpers in `view_budgets.rs`.

**If the enumeration above proves wrong, the named goal state and its checks win
over the list.** Say so in your first paragraph, name what you touched and why.

## The decision this task must NOT make alone

**What the command is called and what it answers by default is a surface
decision.** If it feels like a value call — and D6 makes it one, because
"what may I connect to" is a different question from "where is it" — **park it
as PROPOSED** with the options and a recommendation, per `RULES.md`'s north
star rule. It is cheap to reverse a name now and expensive after `AGENT.md`
ships it.

## Completion check

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo xtask check
cargo test --test agent_doc
cargo test --test command_surface
```

plus the end-to-end check of goal-state 2 by name, with its falsification
recorded.

---

# IN PROGRESS — the `pin` lane's record

**Lane `pin`, branch `lane-pin`, base `a8f2057`. Base verified as the lane's
first action: `git log --oneline -1` reported `a8f2057 freeze: restored, and the
window is closed (M5 opening-1)` and `git status --porcelain` was empty.**

## The surface, and why this shape

The task says the deliverable is *"an answer to a question an agent has, in a
form an agent can act on"*, and that *"the next thing that agent does with the
answer is draw a wire to it"*. Every choice below is made from that sentence.

**The command is `kicli sch pins <TARGET>`.**

- `TARGET` is `REF` — every pin of that placed symbol — or `REF.PIN` — one pin.
  The same `REF.PIN` form `wire draw --from-pin` and `junction add --pin`
  already take, so an address read out of one command goes straight into
  another.
- It hangs off `sch`, whose own help line is *"Read the schematics"* and whose
  only other verb (`view`) is read-only. `sym` was the other candidate and was
  rejected: every verb of `sym` writes, and a read-only verb there would make
  "the `sym` noun writes" false.
- **A target is required.** There is no whole-project form. That is the first
  and largest piece of flood control — see below.

**What each pin line answers is D2 *and* D6 in one record**, which is the thing
the task warns can be under-served:

| Field | The question it answers |
|---|---|
| `at` | D2 — where is the pin (the point `--from-pin` resolves to) |
| heading + escape point | D6 — **what may I connect to it**: the direction a wire must leave in, and the first point it may reach. `wire draw --from-pin REF.PIN --to-at <escape>` is a legal one-segment stub by construction. |
| `off-grid` | D6 — a wire may not start here at all; the router refuses rather than snapping (`Terminal::is_on_grid`) |
| `blocked=<handle>` | D6 — the escape point is barred, so *nothing* may connect to this pin until the obstruction moves |
| `crowded` | D6 — three wire ends already meet here, so a route's own end would be the fourth and `Approach` would offset it (`spec/SPEC.md` §9 Q2) |
| `free` / `net=<name>` | what is already joined here |
| `hidden` | the pin is drawn by nothing and still connects |

**The escape point is printed in the exact token `--to-at` parses** — `x,y`,
`Point`'s own `Display`, trailing zeros trimmed — rather than the layout view's
space-separated two-decimal form. The agent copies the token and passes it. That
is what makes goal-state 2's check a real end-to-end test rather than a
comparison of numbers.

## PROPOSED — the name, the record letter, and the default

Parked rather than guessed, per this task's *"the decision this task must NOT
make alone"* and `RULES.md`'s north star rule. **It is cheap to reverse now and
expensive after `AGENT.md` ships it.** Implemented as the recommendation so
there is something measurable; every part of it is one edit to reverse.

**1. The verb name.** Recommendation: **`sch pins`**.
- *`sch pins`* — read-only noun, plural verb, reads as "the schematic's pins".
- *`sym pins`* — pins belong to a symbol, but every other `sym` verb writes.
- *`pin list` / `pin show`* — a new noun, which implies write verbs later that
  this project has no plan for.
- *`sch view --view pins --of REF`* — no view takes a target today, and one
  symbol's pins are not a view of the drawing.

**2. The record letter is `P`, and `P` is already taken.** The connectivity
view uses `P` for *this sheet's own hierarchical labels*. The collision is
inside one tool's output vocabulary, and dogfood defect 3 is the class of thing
that costs. Recommendation: **keep `P`** — the two never appear in one output,
`P` is the only mnemonic letter free of a worse collision, and the view names
its own record table on its second line. The alternative is `E` (the *end* a
wire may take), which collides with nothing and means nothing to a reader.

**3. What it answers by default: every pin, or only the free ones?** D6 makes
this a value call. Recommendation: **every pin by default, `--free` to filter**.
A pin already on a net is still a legal wire target — that is how a net grows —
so hiding it by default would answer a narrower question than the one asked.
`--free` is there because on a part-wired 40-pin connector the free pins are the
actionable list.

## Flood control — Constitution §6, three layers

1. **A target is required.** The enumeration is bounded by one symbol rather
   than by the project.
2. **`--free`** narrows a part-wired connector to the actionable pins.
3. **`view.max_bytes` with a summary fallback**, in the shape `scope.rs`
   already uses for the other views: over budget, the listing is replaced by
   counts plus the three ways to get the records back.

Measurements and the budget assertion are recorded below as they are made.

## Measured correction — `blocked` had a false positive, and the tool told me

**The first implementation judged the escape step with `Routed { wires: &[] }`,
which is what `edit::wire::draw` uses.** On `tests/fixtures/sch/nets` it printed

```
P 1 ~ passive 43.18,107.95 -x 41.91,107.95 net=D0 blocked=1300004c
```

for `R20.1`. `1300004c` is the wire already on `R20.1`'s own net. Run against
the same drawing:

```
$ kicli wire connect --from-pin R20.1 --to-pin R21.2 -p <copy of the fixture>
joined: net D0
routed R20.1 -> R21.2   via 5 segments, 4 corners, 35.56mm
+ W 0706b1b2 41.91,107.95..43.18,107.95
...
exit=0
```

The router went straight out through the point the view called barred, because
`edit::wire::plan` gives the search `wires: &aim.own` — `wires_through` of the
route's own ends — so a wire at the pin **ends** a route rather than blocking
it. **A view that predicts a refusal the tool does not make is worse than one
that predicts nothing**, so `blockages` now owns the same wires, and the
false positive is gone from the same command.

**A second measurement, which decided what `blocked` may claim.**
`Tally::of_path` exempts the *arriving* step from blocking
(`route/cost.rs:181-190`, `let arriving = ends_the_route && step == steps`), so

```
$ kicli wire draw --from-pin R20.1 --to-at 41.91,107.95 -p <copy>
routed R20.1 -> 41.91,107.95   via 1 segments, 0 corners, 1.27mm
exit=0
```

is accepted **even when the escape point is blocked**. So `blocked` is a claim
about a route that *continues past* the escape point — `wire connect`, and a
longer `wire draw` — and not about the one-segment stub. That distinction is
written into the record's documentation rather than left for a reader to
discover the way I did.

**Scope note, declared rather than quietly taken.** Reaching that answer needed
two things outside the brief's IN list, both additive, both reported here:

- `crates/kicli/src/edit/wire.rs` — `wires_through` changed from `fn` to
  `pub fn` with the reason in its doc comment. No behaviour change; it is the
  one implementation of "which wires are already at this point", and the
  alternative was a second copy of it in the view.
- `crates/kicli/src/connectivity.rs` — a new `Net::joins_nothing`, and
  `view/connectivity.rs` changed to call it instead of spelling the predicate
  inline. Without it the pin view reported `net=#n15` for a pin the
  connectivity view counts under *"18 pin(s) join nothing"* — two views
  disagreeing about one pin. This **removes** a duplication rather than adding
  one.

Also touched, because a new module cannot be reached otherwise:
`crates/kicli/src/view.rs` and `crates/kicli/src/cli.rs` (one module line each,
plus the `sch` noun's dispatch arm), and `crates/kicli/src/cli/args.rs` and
`crates/kicli/src/cli/view.rs` (`SchVerb::View`'s inline fields became
`ViewArgs`, on the `WireVerb::Draw(DrawArgs)` precedent, so the enum can carry a
second verb).

## `AGENT.md` is OWED, and here is the measured block that pays it

**Not written by this lane, by the brief's instruction**: `AGENT.md` is a merge
hotspot held by one lane at a time and the orchestrator schedules it. What
follows is the whole of what it owes.

**Where it goes:** immediately before the `## The commands that write` heading,
after the `sch view` section. That places the read-only commands together.

**Every example below was regenerated from a real run of the built binary**, per
`RULES.md`'s measured-examples rule, and pasted unedited. The producing commands:

```sh
# The first two blocks. The project is a copy of
# crates/kicli/tests/fixtures/sch/nets, and the second command writes, so it is
# run on the copy.
kicli sch pins R20 --quiet
kicli wire draw --from-pin R20.2 --to-at 52.07,107.95 --quiet

# The third block. The project is the forty-pin connector that
# tests/pin_view_budgets.rs writes to
# target/tmp/pin-view-budgets/pin_budget_fallback/probe.kicad_sch, copied into a
# directory holding a kicli.toml of `[view]\nmax_bytes = 200`.
kicli sch pins J1 --quiet
```

**Verified, not assumed.** The block below was pasted into `AGENT.md`,
`cargo test --test agent_doc` was run — **10 passed, 0 failed** — and the file
was then restored with `git checkout -- AGENT.md`, which `git status --porcelain
AGENT.md` confirms is clean. Without the block the same run is **8 passed, 2
failed**: `agent_doc_covers_every_command` ("AGENT.md has no heading naming
`kicli sch pins`") and `agent_doc_covers_every_verb_flag` ("AGENT.md does not
document --free of `sch pins`"). **So this lane's branch is red on `agent_doc`
by construction, and goes green the moment the block below lands.**

### The block, verbatim

### `kicli sch pins`

Where a symbol's pins are, and what may be connected to each one. **Read-only:
it writes nothing.** Ask it before drawing a first wire onto a part, rather than
inferring an offset and learning from a write command that the guess was wrong.

```
kicli sch pins R20
```

```
# pins R20 Test:R  sheet=/13000001-0000-4000-8000-030000000000  at=46.99,107.95 angle=90 mirror=- grid=1.27  scope=symbol
# P num name type at heading escape state
P 1 ~ passive 43.18,107.95 -x 41.91,107.95 net=D0
P 2 ~ passive 50.8,107.95 +x 52.07,107.95 free
```

The target is `REF` for every pin of a symbol, `REF.PIN` for one of them, or a
symbol's identifier. **A target is required.** There is no whole-project form: a
project's every pin is a flood, and `sch view` already lists the symbols.

| Flag | What it does |
|---|---|
| `--free` | List only the pins nothing is joined to yet. |
| `--stats` | Report the size of the answer in bytes. |

| Field | Meaning |
|---|---|
| `num` | The pin number. `R20.2` is how every other command addresses it. |
| `name` | The library's name for the pin. `~` means it has none. |
| `type` | The electrical type: `passive`, `power_in`, `output`, and the rest. |
| `at` | Where the pin connects — the point `--from-pin` resolves to. |
| `heading` | The direction a wire must leave in: `+x`, `-x`, `+y`, `-y`. `*` is any. |
| `escape` | **The first point a wire from this pin may reach.** |
| `state` | One or more of the words below, always in this order. |

| State word | Meaning |
|---|---|
| `free` | Nothing is joined to this pin. |
| `net=NAME` | The pin is already on that net. |
| `off-grid` | The pin is not on the placement grid, so **no wire may start here at all**. kicli refuses rather than moving somebody's pin. |
| `blocked=HANDLE` | Something is one step out, so a route cannot get **past** the escape point. A one-segment wire *to* the escape point is still accepted; a `wire connect` through it is not. |
| `crowded` | Three wire ends already meet here. A fourth is refused, so `wire connect` offsets its end by one grid step and reports the adjustment. |
| `hidden` | The pin is drawn by nothing. A hidden power pin still connects. |

**The escape point is the answer to "what will you accept?"** It is written in
the same `x,y` form `--to-at` takes, so it goes straight back into a command
with no arithmetic in between:

```
kicli wire draw --from-pin R20.2 --to-at 52.07,107.95
```

```
routed R20.2 -> 52.07,107.95   via 1 segments, 0 corners, 1.27mm
  cost 3 = length 1 + turns 0 + crossings 0 + text 0 + proximity 2
  wires added: 1   junctions added: 0
+ W 905a7dc4 50.80,107.95..52.07,107.95
checked: every invariant passed
```

A symbol with more pins than `view.max_bytes` allows answers with counts
instead of records, and says every way to get the records back: name one pin as
`REF.N`, narrow with `--free`, or raise the budget.

```
# pins J1 Probe:CONN  sheet=/00000000-0000-4000-8000-999999999999  at=101.6,101.6 angle=0 mirror=- grid=1.27  scope=symbol-summary  full=2127B budget=200B
# pins=40 listed=40 free=40 reachable=40
# name one pin as J1.N to see it, narrow with --free, or raise view.max_bytes
```

## `command_surface.rs` owes nothing — measured, correcting the brief

The brief says *"`AGENT.md` and `command_surface.rs` both owe this command a
line"*. Measured: `cargo test --test command_surface` is **22 passed, 0 failed**
with `sch pins` on the surface and `command_surface.rs` untouched. Three of its
sweeps already cover the new verb without naming it — `every_verb_parses` runs
`kicli sch pins --help`, `a_positional_argument_always_names_something_that_exists`
requires the positional's value name to be one of `TARGET`, `OWNER`, `FROM`
(it is `TARGET`), and `a_verb_that_makes_an_object_takes_no_positional` is
satisfied because `sch pins` makes nothing. **Nothing is owed there**; if the
orchestrator wants a named check it is an addition rather than a repair.

## Does it close D6? Yes, and here is the argument stated so it can be attacked

The task warns that *"a command that answers 'where is pin 1' without answering
'what may I connect to it' closes D2 and leaves D6 open"*. D6's words are:
*"the doc gives no way to ask what the legal/expected terminus is before trying
— I only find out post-hoc from the `routed`/`W` report, or via a
`blocked`/`invalid` refusal that costs a whole write attempt."*

**The escape point is the legal expected terminus, printed before trying.** It
is a legal one-segment wire from the pin by construction: the escape rule says a
route must take one grid step along the terminal's own direction before it may
turn, so a route consisting of exactly that step honours the rule with nothing
left to check. `tests/pin_view.rs::the_printed_escape_point_is_accepted_by_wire_draw_first_time`
buys the claim end-to-end and through the printed characters.

The three refusals a first wire can buy are each named **before** the write:

| What cost a write attempt | The record word that now precedes it |
|---|---|
| a vertex off the grid, refused not snapped | `off-grid` |
| `blocked` — the way out is barred | `blocked=HANDLE` |
| the four-way rule offsetting the terminus | `crowded` |

**Where it stops, stated rather than left to be discovered.** `blocked` is a
claim about a route that continues *past* the escape point; the one-segment stub
to the escape point is accepted regardless, because `Tally::of_path` exempts the
arriving step. That boundary is measured (above), documented in the record's own
rustdoc, and written into the `AGENT.md` block. **A view that claimed more than
that would be the same defect in the other direction.**

## Falsification table

Per `.claude/skills/falsification-control/SKILL.md`. **Good state committed
before the first break** (`1407855`, and anchored by content:
`crates/kicli/src/view/pins.rs` = `2209a08eba54ad2a74b27fc85a2254cf03494a12`,
`crates/kicli/src/cli/pins.rs` = `2081b70650370a774742331503725936ed2644e1`).
Every restore was checksummed back to those hashes before the next break.

**Run with `cargo test --no-fail-fast` over all three new targets**, because
cargo stops after the first failing target and a caught-by list built without it
under-counts.

| # | What was broken, in the source | Caught by |
|---|---|---|
| 1 | `escape_at: terminal.escape_point(grid)` → `terminal.at` (the escape point is the pin) | `pin_view::the_printed_escape_point_is_accepted_by_wire_draw_first_time` (1 of 15 failed) |
| 2 | the escape heading reversed **and** `escape_at` stepped the other way, so the point lands inside the symbol | the same check, and only it |
| 3 | `blockages`'s `entering(..).blocked_by` post-composed with `.and(None)` — nothing is ever barred | `pin_view::a_pin_whose_escape_is_barred_says_what_bars_it_and_the_router_agrees`, on `the record names what is in the way` |
| 4 | `on_grid: terminal.is_on_grid(grid)` → `true` | `pin_view::an_off_grid_pin_is_named_off_grid_and_no_wire_may_start_there` |
| 5 | `crowded: !has_room(..)` → `false` | `pin_view::a_pin_three_wire_ends_already_meet_at_is_called_crowded` |
| 6 | the `net` lookup post-composed with `.and(None)` — every pin reads `free` | `pin_view::a_pin_already_on_a_net_names_it_and_free_lists_only_the_others` |
| 7 | `render`'s budget test short-circuited with `\|\| true` — the fallback never fires | `pin_view_budgets::a_budget_smaller_than_the_records_falls_back_to_counts` **and** `::the_json_form_carries_the_listing_it_was_rendered_at` (2 of 5) |
| 8 | `only_free`'s `retain(\|pin\| pin.net.is_none())` → `retain(\|_\| true)` | `pin_view::a_pin_already_on_a_net_names_it_and_free_lists_only_the_others`, on the `--free` half |
| 9 | `cli::pins::pins` made to `std::fs::write` the root sheet | `the_pin_view_writes_nothing::asking_where_the_pins_are_leaves_the_project_byte_identical`, plus 5 of 8 in `pin_view` |
| 10 | `word()`'s `if text.is_empty() { "~" }` → `text` — an empty field shortens the record | the unit test `view::pins::tests::a_record_field_is_never_empty`, **and** 3 of 8 in `pin_view` (the shifted field is read as a heading) |

### Break 6 is the row that earned its place

**With `net` forced to `None`, every check then in the file stayed green.** A
view that called every pin free would have shipped. `a_pin_already_on_a_net_names_it_and_free_lists_only_the_others`
was written for that hole — and deliberately covers the state word and `--free`
together, because they are one fact read twice and a break that flips one must
not be able to hide in the other. That check is what turns break 6 and break 8
from silent into caught.

### Controls, against the four blind kinds

- **Shared ancestor.** The end-to-end check's asking side is the binary's
  standard output parsed as characters; its acting side is a second process
  whose acceptance is decided in `Tally::of_path` and `escapes_are_honoured`,
  which the view never calls. The residual shared ancestor is
  `geometry::pins::resolve_pins`, which is why
  `the_same_command_with_the_escape_point_moved_off_grid_is_refused` exists:
  the same command, the same drawing, the printed token moved half a
  millimetre, must be refused. Without it the check would pass on any on-grid
  point at all.
- **Degenerate equality.** `a_budget_smaller_than_the_records_falls_back_to_counts`
  ends by asserting that the budget the answer *fits* does **not** fall back, so
  a view that always summarised fails it. `the_ceiling_is_capable_of_failing` is
  the same idea for the ceiling: a one-byte-per-pin formula must refuse the
  answer, so a run in which the ceiling assertion could not bite fails.
- **Reads nothing.** `the_pin_view_writes_nothing` asserts each run answered
  something before comparing directories, and carries
  `the_check_above_would_notice_a_command_that_writes` — a `wire draw` over the
  same project by the same walk, asserting the root sheet's own bytes changed.
- **Environment.** `git archive HEAD | tar -x -C "$(mktemp -d)"` and the three
  targets run from `/var/folders/.../tmp.IyPsVs6O3X`: **8 + 5 + 2 passed, 0
  failed.** Nothing here asserts a generated identifier, but the byte counts in
  `pin_view_budgets` were produced by running the code, which is the tell.

## Measurements

**Flood control, `tests/pin_view_budgets.rs`.** Published ceiling
`pins_ceiling(pins) = 256 + 96 * pins`. Measured over a one-column connector
built through the probe:

| Pins | Bytes | Ceiling | Fill |
|---|---|---|---|
| 2 | 261 | 448 | 58 % |
| 8 | 547 | 1024 | 53 % |
| 40 | 2127 | 4096 | 51 % |
| 100 | 6427 | 9856 | 65 % |

At the default `view.max_bytes` of 32 768 the fallback fires at roughly 340
pins, which is a large BGA and not a connector. **A forty-pin connector costs
2 127 bytes and is asked for by name** — the caller named `J1`, and there is no
form of this command that answers about a project.

**Gates, on the lane worktree, `cargo xtask check`:**

- **without** the `AGENT.md` block: `fmt` pass, `clippy` pass, **`test` FAIL**,
  `doc` pass, `deny` pass, `clean` pass — 1 of 6 failed, and the only failing
  target is `agent_doc` (8 passed, 2 failed).
- **with** the block pasted in: **all six gates passed.** Then reverted.

`cargo test --test command_surface`: 22 passed, 0 failed.
`cargo test --no-fail-fast` over the workspace: every target green except
`agent_doc`.

**Commits were made with `KICLI_SKIP_HOOK=1`, and that is reported here rather
than hidden**, per the hook's own instruction. The reason is the `agent_doc`
red above and nothing else: the pre-commit hook runs `cargo xtask check`, which
cannot pass while the block this lane was told not to write is missing.

## Carried gap — a hazard inherited rather than introduced

A pin **name** holding a space would shorten nothing but would shift the
positional parse of a `P` record, exactly as a symbol **value** holding a space
already shifts an `S` record of the connectivity view. The empty case is closed
(`word()` writes `~`, break 10). The space case is **not**, in either view, and
is not this lane's to close: fixing it in one view and not the other would leave
one tool with two grammars. Recorded here so the next reader of either view
finds it written down.


---

## Tick — APPROVE, 2026-08-22

**Reviewer verdict: APPROVE.** Lane `lane-pin`, commit `ac23bbf`, base
`a8f2057`, merged to `main` as `d031bed`. Pinned at start and re-checked at
finish, unchanged.

**Method**: three independent scratch copies via `git archive lane-pin | tar -x`,
each verified against `git ls-tree -r --name-only lane-pin` (**340 files,
identical listing**) before reading anything.

### What the reviewer re-measured rather than read

| Claim | Reviewer's own result |
|---|---|
| **the router is reused, not reimplemented** | read `view/pins.rs`: it calls `Terminal::of_pin`, `Obstacles::build(...).entering(...)`, `route::terminal::has_room`. **No parallel geometry.** |
| **writes-nothing, falsified** | patched `cli/pins.rs` to add a `std::fs::write` into the read path; `asking_where_the_pins_are_leaves_the_project_byte_identical` failed exactly as claimed (`left: Some(6), right: Some(37325)`), while the control still passed |
| **goal-state 2 round-trips through printed TEXT** | `the_printed_escape_point_is_accepted_by_wire_draw_first_time` spawns `kicli sch pins`, parses the escape token out of **stdout as text**, then spawns a **second** `kicli wire draw` process with that literal string. **Two processes, no shared in-process ancestor.** The companion control (`…moved_off_grid_is_refused`) guards against a degenerate pass. |
| **the measured false positive** | built the binary and ran `wire connect --from-pin R20.1 --to-pin R21.2` on the fixture — it routed straight through `41.91,107.95` with no complaint, and `sch pins R20` now reports `net=D0` rather than `blocked` |
| **the budget ceiling** | `pins_ceiling(pins) = 256 + 96·pins` is asserted, **is shown capable of failing** (`the_ceiling_is_capable_of_failing`, a 1-byte/pin formula the real answer must exceed), and the fallback is exercised at multiple budgets including the boundary |
| **`command_surface.rs` owes nothing** | ran it on the clean lane tree: **22 passed, 0 failed**, file untouched — the entry's correction to the brief is right |

### Break 6, and the reviewer went past the brief on it

Forcing `net` to `None` failed **only** the new check
(`a_pin_already_on_a_net_names_it_and_free_lists_only_the_others`), 1 of 15. The
reviewer then **removed that test function and re-ran with the break still
applied**: **all 14 remaining checks stayed green.**

**That is a stronger result than the brief asked for.** Showing the new check
fails proves the check works; showing the suite passes *without* it proves **the
hole was real.** Those are different claims and only the second justifies the
check's existence.

### The sanctioned red, closed

The reviewer confirmed the failure on the clean lane tree — **8 passed, 2
failed**, with the exact messages the entry quotes — and `AGENT.md` untouched
(`grep -n "sch pins" AGENT.md` → nothing). **Not weighed against the tick**, per
its brief.

**Paid at merge by the orchestrator, who owns `AGENT.md` as a merge hotspot.**
The entry's measured block was applied verbatim at the position it names, and
`cargo test -p kicli --test agent_doc` now reports **10 passed, 0 failed** —
matching the entry's own prediction exactly.

### Scope

Exactly the declared set. `git diff a8f2057..lane-pin -- AGENT.md spec/SPEC.md
Cargo.toml lib.rs build.rs command_surface.rs 'crates/kicli/src/lint/**'`
returns **empty** — **no collision with the parallel `lint/` lane**, which was
the live risk in this dispatch. The three additive deviations are each declared
with a stated reason and each is mechanically necessary for the declared
surface.

> **WORKFLOW NOTE, the pin lane's reviewer, verbatim:** *"Everything needed for
> review was present and consistent — entry, diff, and pre-declared RED/skip-hook
> rationale for `agent_doc` all matched what re-measurement showed on a fresh
> archive; no friction to report this cycle."*
