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

**One measured case is deliberately not implemented.** A vector against a plain
group, `AA[0..1]` against `BB{P, Q}`, puts `AA0`, `BB.P` and `BB.Q` all on one
net and leaves `AA1` alone. That follows from comparing a vector index against
members that have none, and it is degenerate rather than designed — KiCad's own
comment above the loop is "This feels a bit hacky, perhaps this algorithm should
be revisited in the future". kicli corresponds only between two vectors or two
groups, so it differs from KiCad on that drawing. No corpus project draws one.

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

## Still open: `video` and `vme-wren`

With the rule above, 33 of the 35 corpus hierarchies match KiCad exactly, up
from 32. `video` and `vme-wren` remain, and `video`'s cause is now measured and
is a **different rule**:

> **A bundle names its members at its own sheet path.** Two bundles on one
> sheet whose members expand to equal names put those members on one net, with
> no bus between the two bundles at all.

Measured on two buses that never touch: `DQ[0..2]` and `DQ[0..1]`, each leaving
the root sheet to a child of its own, put `DQ0` and `DQ1` on one net each,
named `/DQ0` and `/DQ1` at the root path. The control, `AA[0..1]` against
`BB[0..1]` drawn the same way, stays apart. That is the shape `video` draws:
its root sheet carries `DQ[0..31]`, `DQ[0..15]` and `DQ[0..7]` on three
separate buses, and KiCad puts all their `DQ0` on one net.

kicli joins like-named member nets only across the sheets one bus class reaches,
so it splits that net three ways. Closing it needs the half that is **not yet
measured**: which sheet path a bundle's members take when the bundle spans
several sheets. `royalblue54L_feather` names the merged net
`/Connectors/UART.RX`, a child path and not the root, so the answer is driver
priority rather than "the sheet the label is drawn on". That is the next probe,
and it should carry a control that fails loudly, as this one now does.

`vme-wren`'s difference is not yet attributed. Its missing joins are whole
connector pins (`J6.32`, `JFP2.4`, `IC14.B1`) that kicli leaves on nets of their
own, which does not look like either rule above.

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
