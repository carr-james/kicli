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

## The trigger, found in KiCad's source after the probes

**Added 2026-08-13 by the orchestrator, after the five probes above.** The
probes looked for a bus wired to a bus. That is not the trigger, which is why
none of them reproduced it.

`CONNECTION_GRAPH::processSubGraphs` (`eeschema/connection_graph.cpp:2581-2588`,
tag 10.0.5) opens with:

```cpp
// Handle buses that have been linked together somewhere by member (net) connections.
for( CONNECTION_SUBGRAPH* subgraph : m_driver_subgraphs )
{
    if( subgraph->m_bus_parents.size() < 2 )
        continue;
```

The subject is a **net**, not a bus. A net subgraph gains a bus parent at
`:2244-2251`, where a bus's member name matches a net subgraph's driver name:

```cpp
if( connection->IsBus() && candidate->m_driver_connection->IsNet() )
{
    subgraph->m_bus_neighbors[member].insert( candidate );
    candidate->m_bus_parents[member].insert( subgraph );
}
```

So the rule is:

> **One net that is a member of two or more bundles links those bundles'
> corresponding members.** Correspondence is `matchBusMember`: by index between
> two vectors, by local member name between two groups.

That is how `UART.RX` and `UART_TRG.RX` become one net in
`royalblue54L_feather`, and `VRAM31` and `DQ31` in `video`: a single net is a
child of both bundles, and every matching member pair is joined behind it.

The comment KiCad puts above the loop is worth keeping in mind — "This feels a
bit hacky, perhaps this algorithm should be revisited in the future" — because
it says the behaviour is emergent rather than designed, and a future release may
change it. The netlist oracle is what will notice if it does.

**Still not measured**: which name a net must carry to become a member-child of
a bundle — the full member name (`UART.RX`) or the local one (`RX`) — and what
`test_name` is built from at `:2200-2240`. That is one probe, now that the loop
to aim it at is known: give one wire a label that is a member of two bundles of
different names, and see whether KiCad joins their other members.

**What kicli must implement** to close the last three hierarchies: track, per
net, the bundles it is a member of; where a net has two or more, union the
corresponding members of those bundles. The member-matching half already exists
in the extractor for rule 6; what is missing is the bus-parent bookkeeping and
the linking pass.

## Probe six, and what the trigger still needs

**2026-08-14.** The source reading above gave a shape to test: one net that
answers to a member name of two bundles. The obvious drawing is a net carrying
two member labels, so that its subgraph has two drivers and `test_name`
(`connection_graph.cpp:2189`, `member->Name( true )`, the full member name)
matches through the `m_multiple_drivers` branch at `:2205`.

Drawn and measured: a wire with both `UART.RX` and `UART_TRG.RX` on it, and the
other member of each group on its own wire.

```
/UART.RX        R1.2
/UART.TX        R2.2
/UART_TRG.TX    R3.2
```

`UART.TX` and `UART_TRG.TX` stayed apart, so **that is not the trigger either**,
and `/UART_TRG.RX` did not even appear as a name: the shared net took one name
and dropped the other.

The reason is now obvious in hindsight and worth writing down, because it is the
thing all six probes have missed. `m_bus_parents` is populated at `:2251` only
when a **bus subgraph** exists whose member matches the net. A drawing with no
bus wire has no bus subgraph, so a net cannot have a bus parent, let alone two,
however many member labels it carries.

**The next probe, which is now fully specified:** two bus wires on one sheet,
one labelled `UART{RX TX}` and the other `UART_TRG{RX TX}`, each with its own
member nets drawn off bus entries, **and** one net carrying both `UART.RX` and
`UART_TRG.RX`. That gives two bus subgraphs, and one net that is a child of
both. If `UART.TX` and `UART_TRG.TX` then join, the rule is confirmed and the
implementation below is what closes it.

**An implementation was written against the source and then reverted**, because
it never fired on the corpus and no test could show it correct. Writing code for
a rule that has not been reproduced is the thing this project does not do. The
shape it took, for whoever picks this up: track for each net the bundle classes
it is a member of; where there are two or more, take each pair and union their
corresponding members, matching by index between two vectors and by local name
between two groups, per `matchBusMember` at `:3324`.
