# The instance record says which unit a symbol draws

Measured 2026-08-13 against KiCad 10.0.5, while making kicli's net partition
equal KiCad's on the demo corpus. The demo that exposed it is
`demos/jetson-agx-thor-baseboard`.

## The shape in the corpus

`usb_debug_pd.kicad_sch` places `U30`, a three-unit `TPS65988DHRSHR`, twice.
One of the two placements is written this way:

```
(symbol (lib_id "antmicroInterfaceControllers:TPS65988DHRSHR")
    (at 256.54 66.04 0) (mirror y) (unit 1) ...
    (instances (project "…" (path "/…" (reference "U30") (unit 2)))))
```

The `(unit 1)` beside the `lib_id` and the `(unit 2)` in the instance record
disagree. kicli read the first, so it drew unit 1's pins — `U30.5`, `U30.9` —
where KiCad draws unit 2's — `U30.13`, `U30.24`. Every unit of that part has a
pin at the same place, so the pins landed on the right nets under the wrong
numbers, and eight nets differed.

This is the same trap as the reference designator, which the item model
already handles: the field beside the symbol is a cache of whichever sheet was
loaded last, and the instance record is the truth.

## What was measured

One scratch schematic, two placements of a two-unit part. Unit 1 has pins 1
and 2, unit 2 has pins 3 and 4, and each unit's first pin is at the same place,
so one wire reaches whichever unit is drawn.

| Cluster | Cached unit | Instance unit | `kicad-cli sch export netlist` says |
|---|---|---|---|
| a | 1 | 2 | `/TOPNET` = `R1.1 U1.3`, and `unconnected-(U1B-D-Pad4)` = `U1.4` |
| b | 2 | 2 | `/CTRLNET` = `R2.1 U2.3` |

Cluster (a) settles it: the instance wins. Pin 1 and pin 2 are not in the
netlist at all, so unit 1 is not drawn.

## The rule, as KiCad implements it

`SCH_SYMBOL::GetUnitSelection( const SCH_SHEET_PATH* )` reads the instance
record for the sheet path and falls back to the cached unit only when the path
has no record. Everything that draws or nets the symbol asks for the unit that
way, exactly as `SCH_SYMBOL::GetRef` asks for the reference designator.

## What it means for kicli

The connectivity graph resolves a symbol's pins against the unit its own
placement records, and falls back to the cached unit when the placement has no
record. The pins of one placement of one sheet are therefore the pins KiCad
draws there.

**A note for whoever owns `geometry`:** `resolve_pins` takes the unit from
`Symbol::unit`, the cached field. Connectivity works around that by resolving
against a copy of the symbol carrying the placement's unit. A `unit` parameter
on `resolve_pins`, or a `Symbol::unit_on(path)` beside `reference_on`, would
put the rule where the other instance-aware readers can find it.

`spec/SPEC.md` §7.1 and `research/representation.md` §3.2 carry the rule.

## Reproduction

```sh
# One two-unit part, placed twice:
#   a: (unit 1) beside the lib_id, (unit 2) in the instance record
#   b: (unit 2) in both
kicad-cli sch export netlist -o probe.net probe.kicad_sch
grep -A8 TOPNET  probe.net    # U30 pin 3, so unit 2 is drawn
grep -c '"1"'    probe.net    # unit 1's pins are absent
```

`cargo test --test net_probe_rules` builds that drawing and holds the rule in
place. With `KICLI_TEST_KICAD_CLI` set it exports the netlist as well, so the
expectation above is checked against the tool rather than remembered.
