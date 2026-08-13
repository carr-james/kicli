# One sheet is one namespace

Measured 2026-08-13 against KiCad 10.0.5, while making kicli's net partition
equal KiCad's on the demo corpus. The demo that exposed it is
`demos/cm5_minima/CM5_MINIMA_3`.

## The shape in the corpus

`IO.kicad_sch` carries a local label `SCL` on the wire that reaches `U1` pin 4,
and, elsewhere on the same sheet, a hierarchical label `SCL` that leaves
through the sheet pin above. kicli treated the two kinds as two namespaces, so
`U1.4` sat on a net of its own while the rest of `/IO/SCL` reached the parent.
KiCad puts all five pins on one net: `/IO/SCL` = `D502.1 J502.2 Module301.56
R502.2 U1.4`. `SDA` and `U1.6` are the same shape.

## What was measured

One scratch schematic, four pairs of strands, each strand a wire with a
resistor at one end and one naming item at the other.

| Pair | The two names | `kicad-cli sch export netlist` says |
|---|---|---|
| a | local label `LOC`, hierarchical label `LOC` | `/LOC` = `R1.1 R2.1` — one net |
| b | local label `GLB`, global label `GLB` | `GLB` = `R3.1 R4.1` — one net, and global |
| c | local label `PWRX`, power symbol valued `PWRX` | `PWRX` = `R5.1 R6.1` — one net |
| d | hierarchical label `HGL`, global label `HGL` | `HGL` = `R7.1 R8.1` — one net |

Every pair merges. The kinds do not partition the namespace; the sheet does.
The net's **name** still says which kind won — `/LOC` carries the sheet path,
`GLB` and `PWRX` do not — but the partition is one net in all four.

## The rule, as KiCad implements it

`CONNECTION_GRAPH::processSubGraphs` (`eeschema/connection_graph.cpp`) walks
the strongly driven subgraphs of one sheet and absorbs every candidate whose
driver name matches, where the candidates are

```cpp
std::copy_if( m_sheet_to_subgraphs_map[ subgraph->m_sheet ].begin(), ... )
```

— the same sheet, and no other. A strong driver is anything of priority
`HIER_LABEL` or above (`CONNECTION_SUBGRAPH::ResolveDrivers`): a hierarchical
label, a local label, a power pin, a global label. A sheet pin is weaker and is
skipped as a merge candidate. Across sheets, `m_global_label_cache` carries the
global kinds — a global label and a global power pin — by name alone.

## What it means for kicli

kicli merges by name in two maps: one keyed by sheet and name, holding every
naming item on that sheet, and one keyed by name alone, holding the global
kinds. A hierarchical label still meets the like-named pin of the sheet symbol
that draws its placement, which is the other half of the hierarchy.

The scoping still holds: two local labels of one text on two sheets are two
nets, and a sheet placed twice gets one net per placement.

`spec/SPEC.md` §7.1 and `research/representation.md` §3.2 carry the rule.

## Reproduction

```sh
# Four pairs of strands on one sheet, each pair carrying one name twice:
#   a: local + hierarchical   b: local + global
#   c: local + power symbol   d: hierarchical + global
kicad-cli sch export netlist -o probe.net probe.kicad_sch
grep -A8 '"/LOC"' probe.net    # two pins
grep -A8 '"PWRX"' probe.net    # two pins
```

`cargo test --test net_probe_rules` builds that drawing and holds the rule in
place. With `KICLI_TEST_KICAD_CLI` set it exports the netlist as well, so the
expectation above is checked against the tool rather than remembered.
