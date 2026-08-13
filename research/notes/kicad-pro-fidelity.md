# `.kicad_pro` round-trip fidelity

Measured by `cargo test -p kicli kicad_pro_fidelity_report`. Each file is
read with an order-preserving JSON reader and written back with
`serde_json::to_string_pretty`, then compared byte for byte.

| verdict | files |
|---|---|
| byte-identical | 4 |

Files measured: 4.

- `byte-identical` first seen in `broken.kicad_pro`
