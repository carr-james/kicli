# A pin on a wire's interior does not connect in KiCad 10.0.5

Measured 2026-08-13 against KiCad 10.0.5, while building the M2 connectivity
fixture. **This contradicts `research/representation.md` §3.2 rule 2 and
`spec/SPEC.md` §7.1 rule (2)**, both of which say that a pin lying on a wire's
interior merges with that wire. It does not. A junction is required.

## What was measured

`crates/kicli/tests/fixtures/sch/nets/nets.kicad_sch` carries the same shape
twice, with one difference.

| Cluster | Geometry | Junction | `kicad-cli sch export netlist` says |
|---|---|---|---|
| R11, R12, R13 | wire `(25.4,88.9)..(50.8,88.9)`; R12 pin 2 and R13 pin 1 at its ends; **R11 pin 1 at `(38.1,88.9)`, mid-span** | no | `Net-(R12-Pad2)` = R12.2 R13.1, and `unconnected-(R11-Pad1)` = R11.1 |
| R15, R16, R17 | wire `(25.4,76.2)..(50.8,76.2)`; R16 pin 2 and R17 pin 1 at its ends; **R15 pin 1 at `(38.1,76.2)`, mid-span** | yes, at `(38.1,76.2)` | `Net-(R15-Pad1)` = R15.1 R16.2 R17.1 |

The two clusters are identical apart from the junction, so the junction is the
whole of the difference. A control run confirmed it from the other direction:
adding a junction at `(38.1,88.9)` to the first cluster turns
`Net-(R12-Pad2)` into `R11.1 R12.2 R13.1`.

## What it means for kicli

`spec/SPEC.md` §7.1 makes the netlist comparison against `kicad-cli` a merge
gate. An extractor that implements rule 2 as written would merge R11 and fail
that gate on this fixture. Reality wins over both documents, so the rule needs
restating. Suggested wording, pending James's ruling:

> (2) a **junction** on a segment interior merges the items that meet there; a
> pin or sheet pin that merely lies on a segment interior does **not** merge.
> Two wires crossing without a junction do not merge either.

The junction case itself is unaffected — a junction on a wire's interior merges,
which the R7..R10 crossing cluster in the same fixture confirms.

Worth noting for the linter (M5): a mid-span pin with no junction looks
connected on screen and is not. That is a readability defect KiCad's own ERC
does not report as such, and it is the kind of thing kicli exists to catch.

## Reproduction

```sh
cd crates/kicli/tests/fixtures/sch/nets
kicad-cli sch export netlist -o /tmp/nets.netlist nets.kicad_sch
grep -A6 'Net-(R12-Pad2)' /tmp/nets.netlist    # two pins, not three
grep -A8 'Net-(R15-Pad1)' /tmp/nets.netlist    # three pins
```
