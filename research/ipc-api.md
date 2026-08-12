# R5 — KiCad IPC API from Rust

Status: protocol, transport and command inventory verified against KiCad
**10.0.5** sources and against the **MIT-licensed** official Python client
(`kicad-python`/kipy), which is the reference implementation the research brief
points at. No Konnect source was consulted.

Two findings dominate:

1. **There is no schematic IPC API in KiCad 10.0.5** — `schematic_commands.proto`
   contains no messages at all (§4.4). kicli's schematic work must be file-based,
   which is what SPEC already assumes; this closes the question.
2. **KiCad's `.proto` files are GPL-3.0-or-later**, which is a live licensing
   question for an MIT/Apache tool (§6) — and the two existing Rust crates for
   this API resolved it in opposite directions.

---

## ⚠ Contradictions and cautions for `spec/SPEC.md`

1. **SPEC §11's PCB ops require KiCad to be running with the API server
   enabled**, and there is no headless fallback. That is a materially different
   UX from every other kicli command. `kicli pcb …` must detect the absence of
   the socket and say precisely what to do, and `AGENT.md` must set the
   expectation.

2. **Licensing (Constitution §9).** Generating Rust types from KiCad's GPL-3
   `.proto` files and shipping them in an MIT/Apache binary is not obviously
   clean. Options in §6; a decision is needed before M9 starts, not during it.

3. **SPEC §11 says "coarse footprint placement by refdes to coords" — the API
   supports it, but only inside a commit**: `BeginCommit` / `EndCommit` group
   changes into one undo step (§4.2). Without that, a placement run leaves the
   user with N undo steps. Should be in the spec.

---

## 1. Enabling the API

Server setting, `~/Library/Preferences/kicad/10.0/kicad_common.json` (macOS) /
`~/.config/kicad/10.0/kicad_common.json` (Linux):

```json
"api": {
  "enable_server": true,
  "interpreter_path": "/Applications/KiCad/KiCad.app/Contents/Frameworks/Python.framework/Versions/Current/bin/python3"
}
```

Backed by `common/settings/common_settings.cpp:477` (`api.enable_server`) and
`include/settings/common_settings.h:206-209`. In the GUI this is
Preferences → Plugins → "Enable KiCad API".

**Verified on this machine: `enable_server` is already `true`.** (It is a
prerequisite of other API clients, so a user who has ever run one will have it
on. A fresh install will not.)

KiCad must be **running**, with the board open, for board commands to work.

---

## 2. Transport

From kipy (`kipy/client.py:47`, `kipy/kicad.py:49-63`), MIT:

| Aspect | Value |
|---|---|
| Library | **nng** (nanomsg-next-gen), protocol **REQ0/REP0** (`pynng.Req0`) |
| Address | `ipc://` Unix domain socket |
| Default path (Linux/macOS) | `ipc:///tmp/kicad/api.sock` |
| Default path (Windows) | `ipc://%TEMP%\kicad\api.sock` |
| Flatpak path | `ipc://$HOME/.var/app/org.kicad.KiCad/cache/tmp/kicad/api.sock` (probed for existence first) |
| Override | `KICAD_API_SOCKET` env var |
| Framing | **none needed** — nng messages are datagram-framed; send one serialised `ApiRequest`, receive one `ApiResponse` |
| Timeouts | send and receive timeouts set on the socket (kipy default is per-client) |
| Reconnect | the client dials on first use and re-dials after a close |

The absence of manual length-prefix framing is worth stating explicitly, because
it is the thing people get wrong when reimplementing: **do not write a
varint/length header.** nng owns the message boundary.

---

## 3. Protocol envelope

`api/proto/common/envelope.proto` (KiCad 10.0.5):

```proto
message ApiRequestHeader  { string kicad_token = 1; string client_name = 2; }
message ApiRequest        { ApiRequestHeader header = 1; google.protobuf.Any message = 2; }
message ApiResponseHeader { string kicad_token = 1; }
message ApiResponse       { ApiResponseHeader header = 1; ApiResponseStatus status = 2; … }
```

Status codes (same file):

| Code | Meaning |
|---|---|
| `AS_UNKNOWN` 0 | — |
| `AS_OK` 1 | success |
| `AS_TIMEOUT` 2 | request timed out |
| `AS_BAD_REQUEST` 3 | invalid/illegal parameters |
| `AS_NOT_READY` 4 | KiCad started recently, not ready |
| `AS_UNHANDLED` 5 | not handled by KiCad |
| `AS_TOKEN_MISMATCH` 6 | `kicad_token` did not match |
| `AS_BUSY` 7 | KiCad busy, cannot accept commands |
| `AS_UNIMPLEMENTED` 8 | call not yet implemented |

Key mechanics:

- The payload is a `google.protobuf.Any`, so dispatch is by **type URL**. A Rust
  client needs the same type-URL strings KiCad registers
  (`type.googleapis.com/kiapi.board.commands.GetNets`, etc.).
- **Token**: `KICAD_API_TOKEN` env var, or empty. kipy sends an empty token on
  the first request and adopts the token returned in the response header
  (`client.py`, the `if self._kicad_token == "":` branch). kicli should do the
  same and then send it on every subsequent request, or it will start getting
  `AS_TOKEN_MISMATCH` when another client connects.
- `AS_NOT_READY` and `AS_BUSY` are **normal** and must be retried with backoff,
  not surfaced as errors. This is the main robustness difference between a demo
  client and a usable one.

---

## 4. Command inventory (KiCad 10.0.5)

12 `.proto` files under `api/proto/`. Message counts below are from the tag.

### 4.1 `common/commands/base_commands.proto`

`GetVersion` / `GetVersionResponse`, `Ping`, `GetKiCadBinaryPath` / `PathResponse`,
**`GetTextExtents`**, `TextOrTextBox`, **`GetTextAsShapes`** / `TextWithShapes` /
`GetTextAsShapesResponse`, `GetPluginSettingsPath`, `StringResponse`.

> **`GetTextExtents` is a gift for R7.** It asks KiCad itself for the extents of
> a given text with given attributes. That makes it a second, independent oracle
> for the font-metrics table (`geometry.md` §5.4, `kicad-cli.md` §5.5) — build
> the table offline from SVG `textLength`, then validate it against
> `GetTextExtents` on a machine with KiCad running. It cannot be a *runtime*
> dependency (Constitution §4 requires deterministic offline scoring), but it is
> an excellent test fixture generator.

### 4.2 `common/commands/editor_commands.proto` — the generic CRUD layer

`RefreshEditor`, `GetOpenDocuments`, `SaveDocument`, `SaveCopyOfDocument`,
`RevertDocument`, `RunAction`, **`BeginCommit` / `EndCommit`**,
**`CreateItems` / `GetItems` / `GetItemsById` / `UpdateItems` / `DeleteItems`**
(each with per-item result status), `GetBoundingBox`, `GetSelection`,
`AddToSelection` / `RemoveFromSelection` / `ClearSelection`, `HitTest`,
`GetTitleBlockInfo` / `SetTitleBlockInfo`, `SaveDocumentToString`,
`SaveSelectionToString`, `ParseAndCreateItemsFromString`.

This is where all of kicli's PCB work happens: items are created/updated/deleted
generically, typed by their `Any` payload.

`BeginCommit`/`EndCommit` wrap a batch into one KiCad undo step — **mandatory**
for kicli's PCB commands so the user gets one undoable operation per kicli
invocation.

### 4.3 `board/*.proto`

- `board_commands.proto` — 37 messages: stackup get/update, enabled/visible
  layers, active layer, board origin, layer names, **`GetNets` / `GetItemsByNet`
  / `GetItemsByNetClass` / `GetConnectedItems` / `GetNetClassForNets`**,
  `RefillZones`, `GetPadShapeAsPolygon`, `CheckPadstackPresenceOnLayers`,
  `InjectDrcError`, editor appearance settings, `InteractiveMoveItems`.
- `board_types.proto` — `BoardLayer` enum, `Track`, `Arc`, `Via`, `PadStack`
  (with full drill/plating/solder-mask/paste modelling), `Pad`, `Zone`,
  **`BoardGraphicShape`**, `BoardText`, `BoardTextBox`, `Barcode`, …
- `board.proto` — board/document level messages.
- `common/types/base_types.proto` — `Vector2`, `Box2`, `Angle`, `PolyLine`,
  `PolygonWithHoles`, `PolySet`, `Text`, `TextBox`, `TextAttributes`, `KIID`,
  `DocumentSpecifier`, `SheetPath`, `LockedState`, `ItemHeader`, …

### 4.4 Schematic: **empty**

`api/proto/schematic/schematic_commands.proto` at tag `10.0.5` contains only:

```proto
syntax = "proto3";
package kiapi.schematic.types;
```

— **no messages.** `schematic_types.proto` defines a few types
(`SchematicLayer`, `Line`, `Text`, `LocalLabel`, `GlobalLabel`,
`HierarchicalLabel`, `DirectiveLabel`) but there are no commands to use them
with.

**Conclusion: KiCad 10.0.5 has no schematic IPC API.** Anything that edits
schematics through KiCad today is either driving the GUI or writing files.
This validates kicli's whole architecture — files are the only way — and it also
means kicli will never conflict with a running Eeschema *except* over file
freshness (Q3).

---

## 5. Mapping SPEC §11's PCB operations

| SPEC op | API route |
|---|---|
| rectangular/rounded board outline on Edge.Cuts | `CreateItems` with `BoardGraphicShape` items (segments/arcs, or a `PolyLine`) on `BL_Edge_Cuts` |
| N fiducials | `CreateItems` with footprint instances from a library; needs the footprint's `LibraryIdentifier` |
| CNC flip-registration holes (parametric: diameter, count, axis) | either footprints with NPTH pads, or `PadStack` items with `DrillProperties`; footprints are more idiomatic and survive DRC better |
| coarse footprint placement by refdes | `GetItems` (footprints) → match refdes → `UpdateItems` with new `Vector2` positions, inside a commit |
| verification after each op | `GetBoundingBox`, `GetItems`, and `RefillZones` where copper is affected |

All of these are parametric and deterministic, as SPEC requires — the agent
supplies numbers, kicli composes messages.

---

## 6. Licensing — the decision that must be made before M9

KiCad's `.proto` files carry the standard KiCad header:

> This program is free software: you can redistribute it and/or modify it under
> the terms of the **GNU General Public License** … version 3 … or (at your
> option) any later version.

Generating Rust types from them produces derived files. Constitution §9 requires
kicli to be MIT OR Apache-2.0 with compatible dependencies.

The ecosystem has already split on this:

| Project | Licence | Approach |
|---|---|---|
| `kicad-python` (kipy), official | **MIT** | generates Python bindings at build time from the GPL protos (submodule) |
| `kicad-api-rs` 0.1.0 | **GPL-3.0-or-later** | took the conservative reading |
| `kicad-ipc-rs` 0.5.1 | **MIT** | ships *checked-in generated prost code* (`src/proto/generated/kiapi.*.rs`, 228 KB) with no `.proto` files in the crate |

Options for kicli, in order of preference:

1. **Depend on `kicad-ipc-rs` (MIT).** It advertises 100 % coverage of KiCad
   v10.0.1's 59 commands, async-first with a blocking wrapper, `nng` as an
   optional dependency, and "zero protobuf dependencies for consumers". The
   licensing exposure then sits with that crate, and kicli's PCB module becomes
   a thin mapping layer. Cheapest path by a wide margin. Risk: a 519-download,
   single-maintainer crate on kicli's critical path for M9 — but M9 is
   explicitly last, and vendoring later is always possible.
2. **Ask the KiCad developers to dual-license the `.proto` files** (or confirm
   that the API definitions are intended to be freely implementable). Given that
   the official Python client is MIT, this is likely a formality, and it is the
   clean answer.
3. **Generate from the protos ourselves and ship the generated code**, matching
   what `kicad-ipc-rs` does. Same exposure, more work.
4. **Hand-write the wire messages** from the protocol description. Only the
   handful of messages kicli needs would be required; it is defensible (a wire
   format is an interface) but it is also the most work and the most fragile.

**Recommendation: (1) now, (2) in parallel.** Q1.

---

## 7. Rust stack

| Need | Crate | Licence | Note |
|---|---|---|---|
| protobuf | `prost` 0.14.4 | Apache-2.0 | if generating ourselves |
| nng transport | `nng` 1.0.1 | MIT | last released 2021 — old but the protocol is stable; `nng-c` 1.11.1 (BSL-1.0) is a maintained alternative |
| whole client | `kicad-ipc-rs` 0.5.1 | MIT | option 1 above |

`tonic`/gRPC is **not** applicable — this is nng REQ/REP with `Any` payloads,
not gRPC.

Note `nng`'s age (2021). If it proves unmaintained against current Rust, the
fallbacks are `nng-c` (BSL-1.0, permissive) or implementing the REQ0 wire
protocol directly over a Unix socket — REQ0 framing is simple, but that is a
last resort.

---

## 8. Connection lifecycle for kicli

```
kicli pcb <op>
 ├─ resolve socket path (KICAD_API_SOCKET → platform default → flatpak probe)
 ├─ if absent → exit 1 with: "KiCad is not running with the API enabled.
 │              Start KiCad, open <board>, and enable Preferences → Plugins →
 │              Enable KiCad API (api.enable_server in kicad_common.json)."
 ├─ dial, Ping, GetVersion → assert major version 10, else refuse
 ├─ GetOpenDocuments → assert the intended board is open; refuse if not
 ├─ BeginCommit
 ├─ … CreateItems / UpdateItems / DeleteItems …
 ├─ verify (GetItems / GetBoundingBox), then EndCommit
 └─ report structured result (Constitution §5)
```

Retry `AS_NOT_READY` and `AS_BUSY` with bounded exponential backoff; treat
`AS_UNIMPLEMENTED` as a hard, clearly-labelled failure — it means the KiCad
version is older than kicli expects.

---

## 9. Open questions for James

- **Q1 — Licensing route.** Depend on `kicad-ipc-rs` (MIT) for M9, and
  separately ask the KiCad devs to clarify/dual-license the `.proto` files?
  Or generate ourselves and accept the same posture `kicad-ipc-rs` took?

- **Q2 — `GetTextExtents` as a test oracle.** Approve using a running KiCad to
  *generate and validate* the font-metrics fixture (offline at runtime, as
  Constitution §4 requires)? It would make `geometry.md` §5 exact rather than
  measured-and-hoped.

- **Q3 — File freshness vs a running KiCad.** kicli edits `.kicad_sch` on disk;
  if Eeschema has the file open, it will not see the change and may overwrite it.
  Should `kicli sch …` detect a running KiCad with that document open (via
  `GetOpenDocuments`, which needs the socket) and warn? It is the one place the
  IPC API is useful to the *schematic* side.

- **Q4 — Minimum KiCad version for `pcb` commands.** `kicad-ipc-rs` targets
  10.0.1; this machine runs 10.0.5. Pin a minimum of 10.0.0 and check
  `GetVersion` at connect?

---

## 10. Sources

- KiCad 10.0.5, tag `10.0.5`: `api/proto/**` (12 files),
  `common/settings/common_settings.cpp:474-477`,
  `include/settings/common_settings.h:206-272`.
- `kicad-python` (kipy) 0.8.0.dev0, **MIT**: `kipy/client.py`, `kipy/kicad.py`,
  `build.py` — the reference implementation named in the research brief.
- crates.io API, fetched 2026-08-12, for `kicad-ipc-rs`, `kicad-api-rs`,
  `prost`, `nng`, `nng-c`; plus the `kicad-ipc-rs` 0.5.1 crate archive
  (`src/proto/generated/`, `LICENSE`, `README.md`).
- This machine: `~/Library/Preferences/kicad/10.0/kicad_common.json`.
- **No Konnect source was read** (Constitution §9, CLAUDE.md).
