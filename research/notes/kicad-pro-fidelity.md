# `.kicad_pro` round-trip fidelity

Measured by `cargo test -p kicli kicad_pro_fidelity_report`. Each file is
read with an order-preserving JSON reader and written back with
`serde_json::to_string_pretty`, then compared byte for byte.

| verdict | files |
|---|---|
| byte-identical | 34 |
| reformats-numbers | 3 |

Files measured: 37.

- `byte-identical` first seen in `CM5_MINIMA_3.kicad_pro`
- `reformats-numbers` first seen in `Q17ng.kicad_pro`
