# A net lists a pin once, however many units draw it

Measured 2026-08-13 against KiCad 10.0.5, while making kicli's net partition
equal KiCad's on the demo corpus. The demo that exposed it is
`demos/multichannel/multichannel_mixer`, where kicli listed `IC1.4` twice on the
`V-` net; `demos/complex_hierarchy` and `demos/pic_programmer` show the same
shape, the last of them four times over on `U2.14`.

## The shape in the corpus

A symbol library may put a pin in **unit 0**, which means "common to every
unit". `TL072CD` in the multichannel demo does this with its two supply pins:

```
(symbol "TL072CD_0_1"
    (pin power_in line (at 5.08 -15.24 90) ... (number "4" ...))
    (pin power_in line (at 5.08 5.08 270) ... (number "8" ...))
)
```

The sheet places unit 1 at one position and unit 2 at another. Both placements
draw pins 4 and 8, at two different points, and both points are wired to the
same rail. kicli therefore found two pin items called `IC1.4` and listed both.
KiCad lists one.

## What was measured

One scratch schematic, two clusters, each with a two-unit part whose pin 9
lives in unit 0.

| Cluster | Geometry | `kicad-cli sch export netlist` says |
|---|---|---|
| U1 | unit 1 at `(50.8, 50.8)`, unit 2 at `(50.8, 76.2)`; **both copies of pin 9 wired together**, with `R1` | `/SHAREDNET` = `R1.1 U1.9` — **one entry**, though two pins reach the net |
| U2 | the same part, the same two units, **each copy of pin 9 on a net of its own** | `/SPLITA` = `R2.1 U2.9` and `/SPLITB` = `R3.1 U2.9` — **the same pin number on both nets** |

The second cluster is the control, and it settles what the rule is not. The
pin is not owned by one unit, and it is not dropped from the second net it
reaches. Each net lists it once.

## The rule, as KiCad implements it

From `NETLIST_EXPORTER_XML::makeListOfNets`
(`eeschema/netlist_exporters/netlist_exporter_xml.cpp`), after the nodes of one
net are sorted:

```cpp
// Some duplicates can exist, for example on multi-unit parts with duplicated
// pins across units.  If the user connects the pins on each unit, they will
// appear on separate subgraphs.  Remove those here:
alg::remove_duplicates( net_record->m_Nodes, ...refA == refB && numberA == numberB );
```

The removal is per net record. That is exactly what the two clusters show.

## What it means for kicli

The rule is about the listing and not about the merge: the two pin items stay
two items in the connection graph, and they name and join whatever they touch.
Only the pin list of a net collapses them. kicli therefore sorts a net's pins
and drops a later pin with the same reference designator and pin number.

`spec/SPEC.md` §7.1 and `research/representation.md` §3.2 carry the rule.

## Reproduction

```sh
# Two units of one part, pin 9 in unit 0 of the library symbol:
#   U1: both copies of pin 9 wired to one net, with R1
#   U2: the copies wired to two nets, with R2 and R3
kicad-cli sch export netlist -o probe.net probe.kicad_sch
grep -A6 SHAREDNET probe.net   # one U1 pin 9
grep -A6 SPLITA    probe.net   # U2 pin 9 here
grep -A6 SPLITB    probe.net   # and U2 pin 9 here as well
```

`cargo test --test net_probe_rules` builds that drawing and holds the rule in
place. With `KICLI_TEST_KICAD_CLI` set it exports the netlist as well, so the
expectation above is checked against the tool rather than remembered.
