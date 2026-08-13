# A hidden power input reaches its rail with no wire drawn

Measured 2026-08-13 against KiCad 10.0.5, while making kicli's net partition
equal KiCad's on the demo corpus. The demo that exposed it is
`demos/interf_u`, and `demos/sonde xilinx` shows the same shape.

## The shape in the corpus

`interf_u.kicad_sch` places `U9`, a `4003APG120` whose supply pins are hidden:

```
(pin power_in line (at -7.62 68.58 270) (length 0) (hide yes) (name "VCC" ...)
    (number "C4" ...))
(pin power_in line (at -10.16 -68.58 90) (length 0) (hide yes) (name "GND" ...)
    (number "K11" ...))
```

Nothing is drawn at those points, so kicli left every one of them on a net of
its own: nine single-pin nets off `VCC`, seven off `GND`. KiCad puts them all
on the rails. This is the old convention that keeps a schematic readable — a
74-series part with fourteen signal pins and no visible supply.

## What was measured

One scratch schematic, two clusters, each a power symbol wired to a resistor,
and one ordinary part with a power input of the pin name to match.

| Cluster | The part's power input | `kicad-cli sch export netlist` says |
|---|---|---|
| a | `U1` pin 9, `power_in`, **hidden**, named `VHID` | `VHID` = `R1.1 U1.9` — **the pin is on the rail** |
| b | `U2` pin 9, `power_in`, **visible**, named `VVIS` | `VVIS` = `R2.1`, and `unconnected-(U2-VVIS-Pad9)` = `U2.9` |

Cluster (b) is the control, and it is the whole of the difference: the same
pin, of the same electrical type, with the same name, connects when the editor
hides it and does not connect when it draws it.

## The rule, as KiCad implements it

From `SCH_PIN::IsGlobalPower` (`eeschema/sch_pin.cpp`):

```cpp
if( GetType() != ELECTRICAL_PINTYPE::PT_POWER_IN )  return false;
if( parent->IsGlobalPower() )                       return true;
if( parent->IsLocalPower() )                        return false;
// Legacy support: invisible power-in pins on non-power symbols act as global power
return !IsVisible();
```

and from `SCH_PIN::GetDefaultNetName`, which names such a pin. The name is the
**symbol's value** when the parent is a power symbol, and the **pin's own
name** otherwise. Both go through `EscapeString( …, CTX_NETNAME )`, so they
share one namespace with global labels
(`CONNECTION_GRAPH::collectAllDriverValues` files both under
`m_global_label_cache`).

## What it means for kicli

A pin names a net when it is a power input and either its symbol is a power
symbol, which names by value, or the pin is hidden, which names by pin name.
Those names merge project-wide. A hidden power pin is still an ordinary
symbol's pin, so a netlist lists it: `U9.C4` is a node, unlike `#PWR01.1`.

**Not measured, and therefore not implemented:** `(power local)`, which KiCad
scopes to one sheet. No symbol in the demo corpus uses it.

`spec/SPEC.md` §7.1 and `research/representation.md` §3.2 carry the rule.

## Reproduction

```sh
# Two ordinary parts, each with one power_in pin and no wire on it:
#   a: U1 pin 9 hidden, named VHID, against a VHID power symbol
#   b: U2 pin 9 visible, named VVIS, against a VVIS power symbol
kicad-cli sch export netlist -o probe.net probe.kicad_sch
grep -A8 '"VHID"' probe.net   # two pins: the hidden pin joined the rail
grep -A4 '"VVIS"' probe.net   # one pin: the visible pin did not
```

`cargo test --test net_probe_rules` builds that drawing and holds the rule in
place. With `KICLI_TEST_KICAD_CLI` set it exports the netlist as well, so the
expectation above is checked against the tool rather than remembered.
