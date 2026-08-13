# The schematic IPC API: what is served, and when

Measured 2026-08-13 against KiCad 10.0.5 as installed, and against `master` at
commit `1d34496` (2026-08-13, `KICAD_SEMANTIC_VERSION 10.99.0`, i.e. the KiCad
11 development line).

**Headline: `research/ipc-api.md` §4.4 and `spec/SPEC.md` §13 are wrong in one
important respect.** They say there is no schematic IPC API in KiCad 10.0.5,
because `schematic_commands.proto` contains no messages. The proto file is
indeed empty of messages, but the conclusion does not follow: eeschema
**registers an API handler and serves the generic editor command set today**.
What 10.0.5 lacks is not the plumbing but the type vocabulary.

## What eeschema serves in 10.0.5

`eeschema/sch_edit_frame.cpp:461` builds an `API_HANDLER_SCH` and registers it
with the API server. That handler registers `GetOpenDocuments` itself and
inherits the rest from `API_HANDLER_EDITOR`
(`common/api/api_handler_editor.cpp:34-39`):

| Command | Served in 10.0.5 |
|---|---|
| `GetOpenDocuments` | yes |
| `BeginCommit`, `EndCommit` | yes |
| `CreateItems`, `UpdateItems`, `DeleteItems` | yes |
| `HitTest` | yes |

Confirmed in the **shipped binary**, not only in source. `strings` over
`/Applications/KiCad/KiCad.app/Contents/PlugIns/_eeschema.kiface` finds
`GetOpenDocuments`, `CreateItems`, `UpdateItems`, `DeleteItems`, `BeginCommit`
and `GetSelection`, and finds no `GetSchematicHierarchy`, `GetSchematicNetlist`
or `SaveDocument`.

**The limit is the type vocabulary.** `api/proto/schematic/schematic_types.proto`
in 10.0.5 holds six messages: `Line`, `Text`, `LocalLabel`, `GlobalLabel`,
`HierarchicalLabel`, `DirectiveLabel`. There is no symbol, field, junction,
sheet or pin message, so there is nothing to send for those. The item factory
behind `CreateItems`
(`eeschema/api/api_sch_utils.cpp:43`) can build twenty-odd item types including
`SCH_SYMBOL_T`, but a client has no wire format for them.

So the honest statement for 10.0.5 is: **wires, text and the four label kinds
are reachable over IPC; symbols, fields and junctions are not.**

## What master adds

| | 10.0.5 | master (`1d34496`) |
|---|---|---|
| `schematic_commands.proto` | 0 messages | 4: `GetSchematicHierarchy`, `SchematicHierarchyResponse`, `GetSchematicNetlist`, `SchematicNetlistResponse` |
| `schematic_types.proto` | 6 messages | 30+, including `SchematicSymbol`, `SchematicField`, `Junction`, `SheetSymbol`, `SheetPin`, `SchematicPin`, `NoConnectMarker`, `BusEntry`, `SchematicSymbolTransform`, `PinMap` |
| `schematic_jobs.proto` | absent | present, 200 lines |
| Handler registrations | 1 own + 6 inherited | 16 own + 8 inherited |

Master's `API_HANDLER_SCH` adds `SaveDocument`, `SaveCopyOfDocument`,
`GetItems`, `GetItemsById`, `GetSelection`, `AddToSelection`,
`RemoveFromSelection`, `ClearSelection`, `GetPageSettings`, `SetPageSettings`,
`GetSchematicHierarchy`, `GetSchematicNetlist`, and six job runners
(`RunSchematicJobExportSvg`, `…Dxf`, `…Pdf`, `…Ps`, `…Netlist`, `…BOM`).

**This covers mutation, not only inspection.** `CreateItems`/`UpdateItems`/
`DeleteItems` are served in both versions and are wrapped in
`BeginCommit`/`EndCommit`, so a client edits inside one undo step. Master
additionally lets a client save the document it edited.

## Served, or schema only?

Served, in both versions, for the commands listed above — this is not a case of
protos existing ahead of an implementation. The evidence is the handler
registration in the editor frame plus the symbols in the shipped binary.

**Not verified: a live call.** No KiCad 11 nightly is installed here, and kicli
has no IPC client until M9, so nothing in this note was proved by connecting to
a running editor. Everything above is read from source and from the shipped
binary's symbol table.

## Stability caveats

From KiCad's own developer documentation
([IPC API](https://dev-docs.kicad.org/en/apis-and-binding/ipc-api/),
[For KiCad Developers](https://dev-docs.kicad.org/en/apis-and-binding/ipc-api/for-kicad-developers/)):

- "new versions of KiCad may introduce new messages and fields, but will not
  modify the meaning of existing messages and fields"
- deprecated items are supported "at least one major version of KiCad after the
  deprecation is announced"
- for contributors: "Never rename, re-order, or change the functional meaning of
  existing fields in any Protobuf messages"

The page still describes the API as under development with the plan of
stabilising the first version for KiCad 9.0, which has passed; the wording has
not kept up with the releases. Treat the compatibility promise as real and the
"stable since" claim as unmaintained prose.

## Earliest plausible release

The schematic-specific commands and the full type vocabulary are on the KiCad 11
development line (`10.99.0`) as of 2026-08-13. **KiCad 11.0 is the earliest
release that carries them.** A backport to a 10.0.x point release is implausible
on KiCad's own rules, which forbid removing or changing fields in a bugfix
release and would not add a command surface there either.

The narrower statement — wires, text and labels over IPC — is true of **10.0.5
today**.

## What to re-check, and how

When a KiCad 11 nightly or release is available:

1. `strings` the installed `_eeschema.kiface` for `GetSchematicHierarchy`,
   `GetSchematicNetlist`, `SaveDocument` and `GetItems`. Absent means the protos
   shipped without the handler.
2. Start it with the API server enabled (`api.enable_server` in
   `kicad_common.json`), open a schematic, and make one real call —
   `GetOpenDocuments` first, since kicli's open-document probe (§14.4) already
   wants it, then `GetSchematicNetlist`.
3. Compare `GetSchematicNetlist`'s partition against
   `kicad-cli sch export netlist` on the same file. If they agree, kicli gains a
   second oracle that needs no subprocess.
4. Re-read `schematic_types.proto` for `SchematicSymbol`'s transform and
   instance fields, which is where a live editing mode would stand or fall:
   a symbol on a twice-placed sheet has two references, and the wire format has
   to carry both.
5. Check whether `SetPageSettings` and `SaveDocument` require the document to be
   the active one, which decides whether a headless-ish workflow is possible.

## What this changes for kicli

Nothing in v1's plan, and two things in its wording.

`spec/SPEC.md` §13's "There is no schematic IPC API in KiCad 10.0.5 — schematic
work is file-based or it does not happen" is too strong; the accurate version is
that the schematic **type vocabulary** is too thin in 10.0.5 to carry the items
kicli edits. The conclusion for v1 is unchanged: kicli edits files, because
symbols and fields are exactly what it edits and those are the types 10.0.5 does
not have.

§14.4's open-document probe is on firmer ground than the spec implies:
`GetOpenDocuments` is served by eeschema in 10.0.5, so the probe is expected to
work rather than hoped to.
