# A label on a wire's interior connects, unless something else is there too

Measured 2026-08-13 against KiCad 10.0.5, while building the connectivity
extractor. This is the companion to
[`pin-on-wire-interior.md`](pin-on-wire-interior.md) and points the other way: a
**pin** on a wire's interior does not connect, and a **label** on a wire's
interior does — under conditions that are worth writing down, because one of
them produces a net that looks connected and is not.

## What was measured

One scratch schematic, three independent clusters, compared through
`kicad-cli sch export netlist`. Every cluster has a wire from `(25.4, y)` to
`(50.8, y)` with a resistor pin at each end, so the wire's own net is visible.

| Cluster | What sits at `(38.1, y)`, mid-wire | KiCad's netlist |
|---|---|---|
| a | a local label `ALONE` | `/ALONE` = R1.2 R2.1 — **the label joined the wire and named its net** |
| b | a local label `WITHPIN` **and** a resistor pin | `/WITHPIN` = **R5.1 only**, and the wire is a separate net R3.2 R4.1 |
| d | the endpoint of another wire, carrying a resistor | wire net = R6.2 R7.1; **the arriving wire and its resistor are unconnected** |

Cluster (b) is the one to remember. The label and the pin form a net of their
own, and the wire they both sit on is not in it. On screen the pin, the label
and the wire all meet at one point. Electrically the wire is somewhere else.

## The rule, as KiCad implements it

From `CONNECTION_GRAPH::updateItemConnectivity` and
`SCH_LABEL_BASE::UpdateDanglingState`, and consistent with everything measured:

1. A label on **two or more** lines joins all of them. A label dropped on an
   unjunctioned crossing therefore merges both wires.
2. A label on **exactly one** line joins it **only if** no pin, other label,
   sheet pin or no-connect shares its anchor. Cluster (b) is this rule denying
   the join.
3. A **pin** on a line's interior never joins it. A junction is required.
4. A **wire endpoint** on another wire's interior never joins it. A junction is
   required.
5. A junction joins every line that passes through its point, as a midpoint or
   as an end.

## What kicli does

The extractor implements exactly this, and the netlist oracle holds it in
place: kicli's partition equals KiCad's on every fixture, net for net. The
committed connectivity fixture depends on rule 1 already — its `D0` and `D1`
labels sit mid-wire and name those nets.

`spec/SPEC.md` §7.1 and `research/representation.md` §3.2 are amended to match.
The general principle James ruled stands behind both amendments: connectivity is
whatever KiCad 10.0.5's netlister does, the rules only describe it, and a
disagreement is settled by measurement and written down with its evidence.

Rule 2 is also a lint finding waiting to happen, and `KI-CONN-001` already
covers it: geometric coincidence without electrical merge. A pin sharing an
anchor with a label mid-wire is caught by the same test that catches a bare pin
mid-wire, because in both cases the pin's net is not the wire's net.

## Reproduction

```sh
# The three clusters, in one file:
#   a: wire (25.4,25.4)..(50.8,25.4), label ALONE at (38.1,25.4)
#   b: wire (25.4,50.8)..(50.8,50.8), label WITHPIN and R5 pin 1 at (38.1,50.8)
#   d: wire (25.4,76.2)..(50.8,76.2), R8 wired up to (38.1,76.2) from below
kicad-cli sch export netlist -o labels.net labels.kicad_sch
grep -A4 'ALONE'   labels.net    # two pins: the label joined the wire
grep -A4 'WITHPIN' labels.net    # one pin: the label and the pin, not the wire
```
