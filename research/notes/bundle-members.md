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

## Open, and the reason the corpus is not yet exact

**Two bundles of different names, wired together, share their members.** In
`royalblue54L_feather` the root sheet wires the port `UART{TX, RX}` of one
child to the port `UART_TRG{TX, RX}` of two others, and KiCad puts `UART.RX`
and `UART_TRG.RX` on one net. `demos/video` does the same at scale:
`VRAM[0..31]` and `DQ[0..31]` are wired together on the root sheet, and
`VRAM31` and `DQ31` are one net.

Five probes failed to reproduce it, and each is recorded here so the next
attempt does not repeat them. Each has a root sheet wiring the ports of two
children together:

1. two vectors, `AA[0..3]` and `BB[0..3]`, with `AA1` and `BB1` on the two
   children — two nets;
2. the same with bus entries joining each named wire to its bundle — two nets;
3. two groups, `AA{P Q}` and `BB{P Q}`, with `AA.P` and `BB.P` — two nets;
4. the same with bus entries — two nets;
5. probe 1 with a label `AA[0..3]` on the root's bundle — `AA1` moved to the
   root's sheet path, so the propagation did reach the like-named child, and
   `BB1` did not move at all.

Probe 5 is the informative one: the propagation happens, and it stops at the
bundle whose name differs.

KiCad's own answer is in `CONNECTION_GRAPH::matchBusMember`
(`eeschema/connection_graph.cpp`), which is worth quoting for whoever picks
this up:

```cpp
if( aBusConnection->Type() == CONNECTION_TYPE::BUS )
    // Vector bus: compare against index, because we allow the name to be different
    ... bus_member->VectorIndex() == aSearch->VectorIndex()
else
    // Group bus ... compare names, because for bus groups we expect the naming
    // to be consistent across all usages
    ... bus_member->LocalName() == aSearch->LocalName()
```

So the correspondence is by **vector index** between two vectors and by
**local member name** between two groups. What the probes have not found is
the condition under which `propagateToNeighbors` carries one bundle's
connection onto another bundle of a different name. Until that is measured,
kicli joins members only where the names agree, and three demo hierarchies —
`royalblue54L_feather`, `video` and `vme-wren` — differ from KiCad by exactly
the nets that cross from one bundle name to another.

## Reproduction

```sh
# A root sheet and one child, the bundle AN[0..7] between them, and the name
# AN0 on a wire of each sheet; ZZ9 on a wire of each sheet as the control.
kicad-cli sch export netlist -o probe.net probe.kicad_sch
grep -A8 '"/child/AN0"' probe.net   # two pins
grep -A4 '"/ZZ9"'       probe.net   # one pin
```

`cargo test --test net_probe_rules` builds that drawing and holds the rule in
place. With `KICLI_TEST_KICAD_CLI` set it exports the netlist as well, so the
expectation above is checked against the tool rather than remembered.
