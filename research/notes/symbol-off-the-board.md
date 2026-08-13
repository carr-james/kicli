# A symbol off the board is in no net list

Measured 2026-08-13 against KiCad 10.0.5, while making kicli's net partition
equal KiCad's on the demo corpus. The demo that exposed it is
`demos/cm5_minima/CM5_MINIMA_3`.

## The shape in the corpus

`IO.kicad_sch` and `PCIe-M2.kicad_sch` carry three symbols kicli put on nets
that KiCad leaves them off: `SW601`, `R601` and `TP701`. Each is written this
way:

```
(symbol (lib_id "CM5IO:SW_Push") (at 33.02 72.39 90) (mirror x) (unit 1)
    (body_style 1) (exclude_from_sim no) (in_bom no) (on_board no)
    (in_pos_files yes) (dnp yes) ...)
```

They are fitting options: drawn, wired, and not built. KiCad's netlist has no
node for them anywhere — not even the one-pin net of an unwired pin.

## What was measured

One scratch schematic, three clusters. Each is one labelled wire with two
resistors on it, and the left resistor of each carries one attribute.

| Cluster | The left resistor | `kicad-cli sch export netlist` says |
|---|---|---|
| a | `(on_board no)` | `/OFFBOARD` = `R2.1` — **R1 is in no net, and neither is its other pin** |
| b | `(dnp yes)` | `/NOTFITTED` = `R3.1 R4.1` — both listed |
| c | `(in_bom no)` | `/NOTINBOM` = `R5.1 R6.1` — both listed |

Clusters (b) and (c) are the controls, and they matter: the three symbols in
the corpus carry all three attributes at once, so only a probe separates them.
`on_board` is the one that decides.

## The rule, as KiCad implements it

`NETLIST_EXPORTER_XML::makeListOfNets` drops the node:

```cpp
if( forBoard && ( sheet.GetExcludedFromBoard() || symbol->ResolveExcludedFromBoard() ) )
    continue;
```

`forBoard` is set for the netlist a board reads, which is what
`kicad-cli sch export netlist` writes. `ResolveExcludedFromBoard` is
`(on_board no)` on the symbol, or on a rule area that covers it.
`dnp` says the part is not fitted, which is a different thing: an unfitted
part still has a footprint and still needs its pads connected.

The rule is about the listing and not about the merge. The pin stays in the
connection graph, so it still names and still joins.

## What it means for kicli

`NetPin` carries `on_board`, beside the `power` flag that works the same way:
the pin stays in the net, so an agent can see what the drawing joins, and the
comparison against a netlist leaves it out.

**Not measured:** a sheet marked excluded from the board, and rule areas. No
demo uses either, so kicli reads the symbol attribute only.

`spec/SPEC.md` §7.1 and `research/representation.md` §3.2 carry the rule.

## Reproduction

```sh
# Three labelled wires, two resistors each, the left one carrying:
#   a: (on_board no)   b: (dnp yes)   c: (in_bom no)
kicad-cli sch export netlist -o probe.net probe.kicad_sch
grep -A4  OFFBOARD  probe.net   # one pin
grep -A8  NOTFITTED probe.net   # two pins
grep -A8  NOTINBOM  probe.net   # two pins
grep -c   '"R1"'    probe.net   # zero
```

`cargo test --test net_probe_rules` builds that drawing and holds the rule in
place. With `KICLI_TEST_KICAD_CLI` set it exports the netlist as well, so the
expectation above is checked against the tool rather than remembered.
