# A label names a net by its escaped text

Measured 2026-08-13 against KiCad 10.0.5, while making kicli's net partition
equal KiCad's on the demo corpus. The demo that exposed it is
`demos/pic_programmer`, and `demos/openair-max` shows it as well.

## The shape in the corpus

`pic_programmer.kicad_sch` carries two local labels on one sheet:

```
(label "VPP{slash}MCLR" (at 209.55 40.64 180) ...)
(label "VPP/MCLR"       (at 213.36 88.9 0)   ...)
```

KiCad puts both on one net, `/VPP{slash}MCLR` = `P2.1 P3.1 R18.2 U5.4 U6.4`.
kicli had them on two nets, and worse: it read the `{` of `{slash}` as the
start of a bus group, so that label carried a bundle and joined no single net
at all. `R18.2` was left on a net of its own.

## What was measured

One scratch schematic, three pairs of wires, one local label on each wire.

| Pair | Labels | `kicad-cli sch export netlist` says |
|---|---|---|
| a | `AA/BB` and `AA{slash}BB` | `/AA{slash}BB` = `R1.1 R2.1` — **one net** |
| b | `CC-DD` and `CC_DD` | `/CC-DD` = `R3.1` and `/CC_DD` = `R4.1` — **two nets** |
| c | `EE/FF` and `EE/FF` | `/EE{slash}FF` = `R5.1 R6.1` — one net, and not a bundle |

Pair (b) is the control. The escape is not a general normalisation of
punctuation; it is one table, applied to one character.

## The rule, as KiCad implements it

Two halves, both in the KiCad source at tag 10.0.5.

1. **The name a label drives is its text, escaped.**
   `CONNECTION_SUBGRAPH::driverName` returns
   `EscapeString( label->GetShownText(...), CTX_NETNAME )`
   (`eeschema/connection_graph.cpp`). In that context `EscapeString`
   (`common/string_utils.cpp`) replaces `/` with `{slash}` and drops line
   breaks. It changes nothing else. A sheet pin and a power symbol's value go
   through the same call, so the whole namespace is escaped text.
   `/` is the character that separates a sheet path from a net name, which is
   why it is the one that may not stand.
2. **Whether a name is a bundle is decided after unescaping.**
   `SCH_CONNECTION::IsBusLabel` calls `UnescapeString` and only then looks for
   a vector, `D[0..7]`, or a group, `{A B}`. `VPP{slash}MCLR` unescapes to
   `VPP/MCLR`, which is neither. `UnescapeString` also leaves a brace that
   follows `$`, `~`, `^` or `_` alone, because that brace draws formatting
   such as an overbar: `~{RESET}` is one net, not a group.

## What it means for kicli

kicli compares escaped names and tests bundle-ness on unescaped ones. The
display name stays the label's own text, because that is what an agent reads
and types; `kicad_name` carries the escaped form, because that is what ERC and
the editor show.

`spec/SPEC.md` §7.1 and `research/representation.md` §3.2 carry the rule.

## Reproduction

```sh
# Three pairs of wires, one local label each:
#   a: AA/BB and AA{slash}BB     b: CC-DD and CC_DD     c: EE/FF twice
kicad-cli sch export netlist -o probe.net probe.kicad_sch
grep -A8 'AA{slash}BB' probe.net   # two pins: the two labels are one net
grep -A4 'CC-DD'       probe.net   # one pin: the two labels are two nets
```

`cargo test --test net_probe_rules` builds that drawing and holds the rule in
place. With `KICLI_TEST_KICAD_CLI` set it exports the netlist as well, so the
expectation above is checked against the tool rather than remembered.
