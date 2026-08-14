# A bundle carries its members to every sheet it reaches

Measured 2026-08-13 against KiCad 10.0.5, while making kicli's net partition
equal KiCad's on the demo corpus. The demos that exposed it are
`demos/kit-dev-coldfire-xilinx_5213` and `demos/royalblue54L_feather`.

## The shape in the corpus

`kit-dev-coldfire-xilinx_5213` runs a bundle `AN[0..7]` from one sheet to
another. On `in_out_conn` a wire named `AN0` reaches the connector; on the
processor sheet a wire named `AN0` reaches `U102` pin 43. No wire joins the
two, and neither does a label of a kind that crosses sheets. KiCad puts both
pins on one net, `/AN0`. kicli had them on two.

`royalblue54L_feather` does the same with groups: `ANALOG{A[0..5]}` carries
`ANALOG.A0` to `ANALOG.A5`, and `I2C{SCL, SDA}` carries `I2C.SCL` and
`I2C.SDA`.

## What was measured

A root sheet and one child. The bundle leaves the root through the child's
port. Each sheet carries two named wires with a resistor on each.

| Cluster | The name on both sheets | `kicad-cli sch export netlist` says |
|---|---|---|
| a | `AN0`, a member of the bundle `AN[0..7]` | `/child/AN0` = `R1.1 R2.1` — **one net across the two sheets** |
| b | `ZZ9`, which no bundle carries | `/ZZ9` = `R5.1` and `/child/ZZ9` = `R6.1` — two nets |

Cluster (b) is the control: an equal name on two sheets is two nets, as a
local label always is. The bundle is the whole of the difference.

Two more probes fix the shape of the rule:

- **No wire is needed.** In cluster (a) neither named wire touches the bundle.
  A second probe with bus entries joining the wires to the bundle gives the
  same partition.
- **A group names its members with a stop.** `ANALOG{A[0..5]}` carries
  `ANALOG.A0`, measured on the same two-sheet drawing.

## The rule, as KiCad implements it

`CONNECTION_GRAPH::processSubGraphs` (`eeschema/connection_graph.cpp`) walks
each bundle's members and links every same-sheet net subgraph whose driver
name is one of them:

```cpp
if( member->IsBus() )
    connections_to_check.insert( end, member->Members().begin(), member->Members().end() );
...
subgraph->m_bus_neighbors[member].insert( candidate );
```

`propagateToNeighbors` then carries the member along the bundle, and the
bundle reaches the child sheet through its port. The member names come from
`SCH_CONNECTION::ConfigureFromLabel` with `NET_SETTINGS::ParseBusVector` and
`ParseBusGroup`.

## What it means for kicli

kicli merges every bundle item into classes as it does any other item, then,
for each class, expands the member names of every bundle name in it and joins
the like-named nets on every sheet that class reaches.


## Two bundles on one bus carry each other's members

**Measured 2026-08-14 against KiCad 10.0.5.** This is the rule five earlier
probes looked for and did not find. It is simpler than the source reading
suggested:

> **Where one bus carries two bundle names, their corresponding members are one
> net.** No wire joins the member nets, and no net has to carry both names.

That is how `UART.RX` and `UART_TRG.RX` become one net in
`royalblue54L_feather`: the root sheet joins the port `UART{TX, RX}` of one
child to the port `UART_TRG{TX, RX}` of two others with a bus, so one bus
carries both names.

Correspondence was measured member by member, and it is not what the names
suggest:

| Two bundles | Joined |
|---|---|
| `UART{TX, RX, CTS}`, `UART_TRG{TX, RX}` | `.TX` to `.TX`, `.RX` to `.RX`; `UART.CTS` alone |
| `AA{P, Q}`, `BB{Q, Z}` | `AA.Q` to `BB.Q` only |
| `AA[0..2]`, `BB[5..6]` | `AA0` to `BB5`, `AA1` to `BB6`; `AA2` alone |
| `ANALOG{A[0..1]}`, `BB[0..1]` | `ANALOG.A0` to `BB0`, `ANALOG.A1` to `BB1` |

So a group member corresponds by its own name without the group's, and a vector
member by **its place in the range, not the number written in its name**:
`AA[0..2]` against `BB[5..6]` joins `AA0` to `BB5`. A group whose member is
itself a vector holds vector members, so `ANALOG{A[0..1]}` corresponds by place.
This is `CONNECTION_GRAPH::matchBusMember` (`eeschema/connection_graph.cpp`,
tag 10.0.5), which compares `VectorIndex()` for a vector and `LocalName()`
otherwise.

**One measured case is deliberately not implemented, and is reported instead.**
A vector against a plain group, `AA[0..1]` against `BB{P, Q}`, puts `AA0`,
`BB.P` and `BB.Q` all on one net and leaves `AA1` alone. That follows from
comparing a vector index against members that have none, and it is degenerate
rather than designed — KiCad's own comment above the loop is "This feels a bit
hacky, perhaps this algorithm should be revisited in the future". kicli
corresponds only between two vectors or two groups, so it differs from KiCad on
that drawing.

Differing is acceptable; differing in silence is not. A net list that is wrong
and confident reads exactly like one that is right, so kicli **detects the shape
it declines to reproduce**: a bus carrying a bundle whose members match by place
and another whose members match by name raises the `mixed-bundle-kinds` warning,
which names both bundles and says the nets may differ from KiCad's. The warning
rides on `Nets::warnings`, and the connectivity view prints it as a `W` record
above the nets it qualifies, because a warning that arrives after the data it
qualifies has already been skipped.

"Degenerate" is a claim about real drawings, so it is checked against them:
`no_corpus_hierarchy_mixes_bundle_kinds` walks all 35 demo hierarchies and
asserts none raises the warning. It passes. If a demo ever draws one, the shape
is not degenerate, the rule has to be implemented, and that test fails to say
so.

## Why five probes missed it: the probes were wrong, not the rule

Probes 1 to 5 drew the right shape — a root sheet wiring the ports of two
children together — and measured two nets. The drawing was not what they
thought. A sheet pin carries an angle, and the angle decides which edge of the
sheet symbol the pin binds to: 0 is the right edge, 180 the left. The probe
harness wrote 0 for every pin. A pin written at the left edge with angle 0 is
moved by KiCad to the edge its angle names, which takes it off the bus drawn to
meet it, so the second child was never on the bus at all.

The control that found it: two children carrying the **same** bundle name, whose
members must join and did not. A probe whose control fails is measuring its own
defect. `two_bundles_on_one_bus_join_their_corresponding_members` in
`net_probe_rules.rs` now draws both angles, and `Probe::sheet_named` documents
which edge each one binds to.

The source reading that produced probes 5 and 6 — that the trigger is a net with
two or more bus parents — is not wrong, but it is the mechanism one level down.
A net reached by two bundles on one bus has two bus parents by construction. Six
probes aimed at reproducing the mechanism directly; the rule above is what the
drawing has to show.

## The namespace a bundle names its members in

**Measured 2026-08-14.** The second rule, which `video` and `vme-wren` needed:

> **A bundle label names its members in the namespace of the sheet it is drawn
> on**, exactly as any other label does. Two bundles labelled on one sheet share
> every member whose name they both carry, though no bus joins them.

`DQ[0..2]` and `DQ[0..1]`, each leaving the root sheet on a bus of its own that
the other never touches, put `DQ0` and `DQ1` on one net each. The control is the
same drawing with the two bundles on two child sheets instead: an equal member
name is then two nets, one per namespace.

**A sheet pin names nothing on the sheet it is drawn on.** Its name is the
child's hierarchical label, so it speaks for the child's namespace and not the
parent's. That distinction is load-bearing rather than pedantic: reading a sheet
pin as naming the parent joins every sub-range that feeds a like-named port, and
those are a different net per port. The control that catches it is one child file
placed twice, whose two placements must stay apart.

This is what `video` draws — `DQ[0..31]`, `DQ[0..15]` and `DQ[0..7]` on three
buses that never touch, all labelled on the root — and what `vme-wren` draws at
depth. `vme-wren`'s `pp_driver_32x` labels `PP_OUT[0..31]` on one bus and
`PP_OUT[24..31]` on another that only reaches a bank's port; the two never touch
and are one namespace apart, so `PP_OUT31` is one net across them, and `J6.32`
on the root joins it.

An earlier reading of this rule keyed the namespace on the bus's strongest
driver instead of on each label's own sheet. It closed `video` and left
`vme-wren` short, because a bus that reaches the root takes the root's scope
while the sub-range beside it keeps the child's, and two bundles drawn side by
side then never met. Keying each label where it is drawn is both simpler and
what KiCad does, and it removed the driver-priority machinery entirely.

**With both rules, all 35 corpus hierarchies match KiCad exactly.**

## What `vme-wren` needed, and the probe that lied about it

`vme-wren` was the last hierarchy to match, and its difference had a very clean
shape: **96 nets, each missing exactly one pin** — connector and FPGA pins such
as `J6.32`, `JFP2.4` and `IC14.B1`, each left on a net of its own. The cause was
the namespace rule above, and nothing else.

Its chain renames at every port: a leaf's port is `PP_OUT[0..1]`, its bank
carries `PP_OUT[0..7]`, the bus feeding that bank is labelled `PP_OUT[24..31]`,
and the root carries `PP_OUT[0..31]`. `FP_IO` is worse: the root labels two
buses `FP_IO[0..31]` and feeds one of them into a port called `IO[0..31]`, so
the member names differ on the two sides of one port.

### A probe measured its own defect, and how that was caught

Before the rule was found, a probe reported that kicli could not carry a bundle
member through a port at all, and that it merged members no geometry related.
Both were the instrument.

The probe generator wrote coordinates straight from floating point, so
`38.1 + 12.7 * 3.0` reached the file as `76.19999999999999`. KiCad reads that;
kicli's number reader rejects more than four decimals and the caller reads the
rejection as zero. Phantom items collected at the origin, every label lay on
them, and nets that shared nothing were joined. Rounded to the four decimals
KiCad itself writes, the same drawing passes.

Two things came out of it. `Probe::check_precision` now refuses to write a
drawing carrying a number KiCad would not write, so no probe can make this
mistake quietly again. And a real kicli defect is recorded: **a coordinate the
reader cannot parse becomes zero rather than an error**, which is the
wrong-and-confident failure this project exists to avoid. That one is not a
bundle rule and belongs in its own task.

The lesson is the one the sheet-pin angle already taught: a probe needs a
control that fails loudly, and an instrument needs checking before its readings
are believed. Both controls are committed —
`one_child_placed_twice_carries_each_sub_range_to_its_own_placement` and
`two_bundles_in_different_scopes_keep_their_members_apart`.

## Reproduction

```sh
# A root sheet and one child, the bundle AN[0..7] between them, and the name
# AN0 on a wire of each sheet; ZZ9 on a wire of each sheet as the control.
kicad-cli sch export netlist -o probe.net probe.kicad_sch
grep -A8 '"/child/AN0"' probe.net   # two pins
grep -A4 '"/ZZ9"'       probe.net   # one pin
```

`cargo test --test net_probe_rules` builds every drawing above and holds the
rules in place. With `KICLI_TEST_KICAD_CLI` set it exports each one with
`kicad-cli` as well, so the expectations are checked against the tool rather
than remembered.
