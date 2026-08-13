# Two bus entries that meet do not join

Measured 2026-08-13 against KiCad 10.0.5, while making kicli's net partition
equal KiCad's on the demo corpus. The demo that exposed it is
`demos/kit-dev-coldfire-xilinx_5213`.

## The shape in the corpus

`in_out_conn.kicad_sch` fans a bundle out to a connector. Two of the entries
are drawn towards each other:

```
(bus_entry (at 353.06 48.26) (size 2.54 2.54))     ; ends at (355.6, 50.8)
(bus_entry (at 353.06 53.34) (size 2.54 -2.54))    ; ends at (355.6, 50.8)
```

Both bus ends land on the same point of the bundle. kicli joined items that
share a point, so it joined the two entries, and through them the wire named
`AN0` to the wire named `AN2`. Two nets became one, and the same happened to
`IRQ-5` and `IRQ-7`.

## What was measured

One scratch schematic, two clusters, each with two entries drawn towards one
point.

| Cluster | Where the two entries meet | `kicad-cli sch export netlist` says |
|---|---|---|
| a | on a bundle labelled `AN[0..7]` | `/AN0` = `R1.1` and `/AN2` = `R2.1` — **two nets** |
| b | in free space, with no bundle there | `/BB0` = `R3.1` and `/BB2` = `R4.1` — **two nets** |

Cluster (b) is the control, and it moves the rule: the bundle is not the
reason. Two bus entries never join, wherever they meet.

## The rule, as KiCad implements it

`SCH_BUS_WIRE_ENTRY::ConnectionPropagatesTo` (`eeschema/sch_bus_entry.cpp`)
refuses four partners:

```cpp
// Don't generate connections between bus entries and buses, since there is
// a connectivity change at that point (e.g. A[7..0] to A7)
...
// Same for bus junctions
...
// Don't generate connections between bus entries and bus labels that happen
// to land at the same point on the bus wire as this bus entry
...
// Don't generate connections between two bus-wire entries
if( aItem->Type() == SCH_BUS_WIRE_ENTRY_T )
    return false;
```

An entry carries one member of the bundle. Everything it meets at its bus end
carries the whole bundle, or another member, so joining any of them shorts two
nets. `CONNECTION_GRAPH::updateItemConnectivity` adds one more guard of the
same kind: where a bus passes, an entry connects only to items on the bus
layer.

## What it means for kicli

At a shared point, kicli joins the items that are not bus entries as before,
and then joins each bus entry to one of them — never to another entry, and
never to a junction where a bundle passes. The refusal of a bus, and of a bus
label, already followed from the rule that a bundle and a net never join.

`spec/SPEC.md` §7.1 and `research/representation.md` §3.2 carry the rule.

## Reproduction

```sh
# Two pairs of bus entries drawn towards one point:
#   a: the point is on a bundle labelled AN[0..7]
#   b: the point is in free space
kicad-cli sch export netlist -o probe.net probe.kicad_sch
grep -A4 '"/AN0"' probe.net    # one pin
grep -A4 '"/BB0"' probe.net    # one pin
```

`cargo test --test net_probe_rules` builds that drawing and holds the rule in
place. With `KICLI_TEST_KICAD_CLI` set it exports the netlist as well, so the
expectation above is checked against the tool rather than remembered.
