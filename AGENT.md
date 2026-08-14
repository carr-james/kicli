# kicli for agents

kicli reads and edits KiCad 10 schematics from the command line. It exists so
that an agent can see a schematic without loading a 600 kB file into its
context, and can change one without rewriting the parts it did not touch.

This document is the command reference. It ships with the tool and is kept in
step with it by a test, so a command that is not written down here does not
exist.

**This build reads and writes.** Wire routing, the style score and rendering
arrive later. What follows is what works today.

Licence: **GPL-3.0-or-later**. If you need to embed schematic tooling in a
closed product, use [`kicad-tools`](https://github.com/rjwalters/kicad-tools)
(MIT, Python, broader surface) instead.

## The shape of a command

```
kicli <noun> <verb> [flags]
```

Global flags, accepted before or after the verb:

| Flag | What it does |
|---|---|
| `--output text\|json` | Terse lines, or one JSON object with the same content. Text is the default. |
| `--project <DIR>`, `-p` | The project directory. The default is the working directory. |
| `--sheet <PATH>` | One sheet path, instead of the whole project. A command that writes edits that placement's file; without it, the root sheet. |
| `--quiet`, `-q` | Results only. Progress notes are suppressed. |
| `--allow-comment-loss` | Write a file that carries `#` comments, dropping them as KiCad would. Only commands that write are affected. |
| `--version` | Print the version and exit. |

Findings are data, not failure. A command that reports twenty problems still
exits 0. Read the exit code to learn whether kicli *ran*, and read the output to
learn what it *found*.

## Commands

### `kicli project info`

What the project is: its files, its sheet tree with page numbers and sheet
paths, how many symbols each sheet holds, the format stamp of every file, and
whether `kicad-cli` is available.

```
project healthy
  file       healthy.kicad_pro
  root       healthy.kicad_sch
  kicad-cli  10.0.5
  alias      ADDR = A0 A1

sheets 2
  page 1  symbols 0  power 0  file healthy.kicad_sch  path /0000...0000
  page 2  symbols 0  power 0  name stage  file stage.kicad_sch  path /0000...0000/0000...0001
```

### `kicli project check`

What is wrong with the project, as a list of findings. Every file parses and
round-trips; every sheet names a file that is there; the sheet tree has no
cycle; every symbol's instance data covers the sheet paths it appears on; every
format stamp is one kicli will write; a file carrying `#` comments is named,
because writing it would drop them.

```
findings 3
  sheet-file-missing  broken.kicad_sch  sheet absent names absent.kicad_sch, which is not there
  version-ceiling     future.kicad_sch  the format stamp 20260803 is above the ceiling 20260306, ...
  refuse-to-write     commented.kicad_sch  the file carries 1 comment(s), which writing would drop; ...
```

The command also names the checks it does **not** yet make, rather than passing
silently: library resolution is not covered in this build.

`project check` warms KiCad's font cache when `kicad-cli` is present and says so
first. The first run on a machine can take over two minutes; later runs take
under a second.

### `kicli sch view`

The compact views. **This is what you act on.** Everything else kicli will
eventually draw is a picture of the same data; the view is the data.

| Flag | What it does |
|---|---|
| `--view connectivity\|layout` | Which view. Connectivity is the default. |
| `--include-power` | List power symbols, which are otherwise left out. |
| `--uuids` | Add the first eight characters of each object's identifier. |
| `--stats` | Report the size of the view in bytes. |

Every line starts with a one-letter record type, so you can filter with `grep`
and never need a parser.

#### The connectivity view

What is joined to what. No coordinates at all.

```
# scope project  sheets=3 sym=23 pwr=4 nets=14
sheet /0000...0000 / sym=19 pwr=2
S R1 10k R
H channel_a: IN(i)
P IN(i)
# N name[=kicad-name]: pins
N GND*: R1.2 R100.2 R2.2 R200.2
N #n1=Net-(R10-Pad1): R10.1 R7.2 R8.1 R9.2
# 18 pin(s) join nothing; sch erc lists them
```

| Record | Meaning |
|---|---|
| `# scope` | What this view covers: `project`, one `sheet`, or an `index` |
| `sheet` | One placement of one sheet, with its counts |
| `S` | A symbol: reference, value, symbol name |
| `H` | A child sheet's ports, with directions `i o b t p` |
| `P` | This sheet's own hierarchical labels |
| `N` | A net: name, then its pins |

Reading a net record:

- `N GND*` — the `*` means the net is drawn on more than one sheet, so a
  per-sheet view shows only part of it.
- `N #n1=Net-(R10-Pad1)` — `#n1` is kicli's **handle**, stable under unrelated
  edits. After the `=` is KiCad's own name, which changes when an unrelated
  symbol is renumbered. Use the handle to address the net; use KiCad's name to
  correlate with the rule check and with what the editor shows.
- `N channel_a/IN` — a name that two nets would otherwise share is qualified by
  the sheet it comes from. A hierarchical label is local to its placement, so a
  sheet placed twice gives two different nets with the same drawn name.

#### The layout digest

Where things are drawn. No connections. Coordinates are millimetres to two
decimals.

```
page A4  sheet /0000...0000
L R1 50.80 50.80 0 - 2.03x7.62
F R1.Reference 0.00 0.00 0
T label NET_A 50.80 41.91 0
W 15 segments, 2 junctions, 1 crossings
```

| Record | Meaning |
|---|---|
| `L` | A symbol: reference, x, y, angle, mirror (`x`, `y` or `-`), body size |
| `F` | A field that has moved off the position its library gives it |
| `T` | A label or free text: kind, text, x, y, angle |
| `W` | The wire summary for the sheet |

`L` carries the angle and mirror exactly as the file writes them, because that
is what you pass back to a rotate command. `F` lists a field **only** when it has
moved, so on a tidy sheet it is empty and on an untidy one it is the list of
things to fix. A crossing is counted only where no junction sits on it: a
junctioned crossing is a connection.

#### Scope and budget

A view covers the whole project by default. If that would be larger than
`view.max_bytes` (32 KB by default), you get an index instead — one line per
sheet, and instructions for asking for less. The first line always says which
of the two you are holding.

```
# scope index  sheets=3  full=41000B budget=32768B
# ask for one sheet with --sheet <path>, or raise view.max_bytes
I /0000...0000 / sym=19 pwr=2 nets=12
```

## The commands that write

Every command below changes one file and reports what it changed. The report is
a delta fragment, so "what changed" reads the same whether you asked or caused
it.

**One command is one write.** The write is atomic: kicli writes a temporary file
beside the target, re-parses the bytes it wrote, and renames only if they read
back correctly. A failure leaves the original byte-identical and exits 3.

**Every write runs four invariants first.** The output re-parses; every
identifier a file references still exists; connectable geometry is on the grid;
no instance data is orphaned. A failure writes nothing and exits 3. The report
names each invariant and whether it passed, so a caller never has to assume.

### How to name an object

| You type | It means |
|---|---|
| `R12` | The symbol whose reference designator is `R12` **on this sheet path** |
| `da5aa983` | The object whose identifier starts with these characters |
| `da5aa983-…-…` | The whole identifier |

Get handles from `sch view --uuids`. Eight characters are the minimum, and they
do not always identify: if two objects share them, kicli refuses and lists both.
Name more of the identifier.

### Units, and the grid

Positions and sizes are **millimetres** at the command line: `--at 50.8,88.9`,
`--size 25.4x12.7`. That is the unit the views print, so a value read out of a
view goes straight back in.

Connectable geometry snaps to the grid: symbol anchors, label anchors and
junctions. The report says when it snapped. `--off-grid` places the anchor
exactly and reports an `off-grid` note instead, so you feel the exception.

Field text and graphic text are **exempt**. KiCad's own autoplacement lands
fields on arbitrary units, so snapping them would fight the editor.

### `kicli sym place`

Place a symbol, with its definition and its instance data.

| Flag | What it does |
|---|---|
| `--lib-id <ID>` | The library identifier to record, such as `Device:R`. Required. |
| `--from <FILE>` | A `.kicad_sym` file holding the definition. |
| `--at <X,Y>` | Where the anchor goes. Required. |
| `--reference <REF>` | The reference designator the placement carries. Required. |
| `--value <TEXT>` | The value to write, when it is not the library's own. |
| `--angle 0\|90\|180\|270` | The angle to place it at. |
| `--mirror x\|y` | The axis to mirror about, applied after the angle. |
| `--unit <N>` | Which unit of a multi-unit part to draw. |
| `--body-style <N>` | 1 is normal, 2 is the De Morgan alternative. |
| `--off-grid` | Place the anchor exactly where asked. |

The definition is **copied into the sheet**. KiCad draws that copy, so a
placement without it draws as a placeholder. kicli takes the copy from the
`.kicad_sym` file you name, or from a file of this project that already embeds
that identifier. This build does **not** search the library tables: without
either source, it refuses and says so.

A sheet placed twice gets **two instance entries**, one per sheet path, both
carrying the reference you asked for. Use `sym set-field` to change one.

### `kicli sym move`

```
kicli sym move <TARGET> --to <X,Y> | --by <DX,DY> [--off-grid] [--keep-field-positions]
```

Move to a position, or by an offset. Exactly one of the two.

**The fields move with the symbol** and keep their own angles.
`--keep-field-positions` leaves them where they are.

### `kicli sym rotate`

```
kicli sym rotate <TARGET> --to 0|90|180|270 [--keep-field-positions]
```

Turn to an absolute angle. The anchor does not move. Each field's position turns
about the anchor; its own angle does not change.

### `kicli sym mirror`

```
kicli sym mirror <TARGET> --axis x|y [--keep-field-positions]
```

Reflect about an axis through the anchor. The written orientation is normalised,
exactly as KiCad's editor writes it.

### `kicli sym delete`

```
kicli sym delete <TARGET>
```

The instance data goes with the symbol. The embedded definition stays if another
placement still uses it, and goes if none does.

### `kicli sym set-field`

```
kicli sym set-field <TARGET> --name <FIELD> --value <TEXT>
```

Setting `Reference` is the case the format punishes. The truth lives in
`instances → project → path → reference`; the property on the symbol is a cache
of whichever sheet was loaded last. kicli moves both, and **only for the sheet
path you are on**. The other placements of a twice-placed sheet keep theirs.

### `kicli field move`, `kicli field rotate`, `kicli field justify`

```
kicli field move   <OWNER> --name <FIELD> --to <X,Y>
kicli field rotate <OWNER> --name <FIELD> --to 0|90|180|270
kicli field justify <OWNER> --name <FIELD> [--horizontal left|center|right] [--vertical top|center|bottom]
```

`justify` needs at least one axis. An axis you do not name keeps what it has.

Every one of these clears `fields_autoplaced`. Without that, KiCad places the
fields again on its next open and your work disappears.

### `kicli field show`, `kicli field hide`

```
kicli field show <OWNER> --name <FIELD>
kicli field hide <OWNER> --name <FIELD>
```

Not just symbols. A sheet owns `Sheetname` and `Sheetfile`; a global label owns
`Intersheetrefs`; a netclass flag owns `Netclass` and `Component Class`. kicli
writes the `hide` form the file's own format stamp uses.

### `kicli text add`, `kicli text move`, `kicli text edit`, `kicli text delete`

```
kicli text add    --text <TEXT> --at <X,Y> [--angle 0|90|180|270] [--size <WxH>]
kicli text move   <TARGET> --to <X,Y>
kicli text edit   <TARGET> --text <TEXT> | --size <WxH>
kicli text delete <TARGET>
```

`--size` makes a text box. Without it you get free text. `text edit` takes one
of `--text` and `--size`, because one command is one write and one delta.

### `kicli label add`, `kicli label move`, `kicli label delete`

```
kicli label add    --text <NAME> --at <X,Y> [--kind local|global|hierarchical] [--angle 0|90|180|270] [--shape input|output|bidirectional|tri-state|passive]
kicli label move   <TARGET> --to <X,Y>
kicli label delete <TARGET>
```

**Adding a label changes the netlist**, so the report names the net it joined or
made, with that net's pins:

```
+ T da5aa983 "SPY"
checked: every invariant passed
net SPY: R12.2 R13.1 (was #n3)
```

A label on a wire's interior joins that wire — unless a pin or another label
shares its anchor. kicli reports that case as a note rather than leaving you to
find out from a netlist.

A hierarchical label is the **child half** of a sheet port. kicli does not add
the parent's sheet pin, and says so in a note.

### `kicli junction add`, `kicli junction delete`

```
kicli junction add    --at <X,Y> | --pin <REF.PIN>
kicli junction delete --at <X,Y> | --pin <REF.PIN>
```

`--pin` is the point where that pin connects, worked out on this sheet path.

**A junction where four wire ends already meet is refused.** That is a defect a
reader cannot resolve, and KiCad's own check ignores it by default. The refusal
names the four wires, exits 1, and writes nothing. Move one wire end by a grid
step, then add the junction.

### `kicli noconnect add`, `kicli noconnect delete`

```
kicli noconnect add    --pin <REF.PIN>
kicli noconnect delete --pin <REF.PIN>
```

**A no-connect on a pin something already joins is refused.** It would
contradict the drawing. The refusal says what the pin is joined to, exits 1, and
writes nothing.

### `kicli net rename`

```
kicli net rename <FROM> --to <NAME>
```

kicli does not write the project file, so a net's name is only what its labels
say. A rename renames **every** label of that net, across every sheet the net
reaches, plus the sheet pins that meet them and the `Value` of the power symbols
on it. Every file is checked before any file is written.

A net with no label has no name to change. The refusal exits 1 and points you at
`label add`.

## The loop: look, edit, verify

```sh
# 1. Look. The view is the data.
kicli sch view --uuids
# S R12 10k R
# N #n3: R12.2 R13.1

# 2. Edit. Each command reports what it changed and what it checked.
kicli sym place --lib-id Test:R --from parts.kicad_sym \
                --at 200.66,100.33 --reference R99 --value 4k7
# + S R99 4k7 Test:R
# checked: every invariant passed

kicli field move R99 --name Value --to 203.2,102.87
# ~ F R99.Value  moved  (0.00,0.00) -> (2.54,2.54)
# checked: every invariant passed

kicli label add --text SPY --at 30.48,88.9
# + T da5aa983 "SPY"
# checked: every invariant passed
# net SPY: R12.2 R13.1 (was #n3)

# 3. Verify. The view says the same thing the deltas did.
kicli sch view | grep -E 'R99|SPY'
# S R99 4k7 R
# N SPY: R12.2 R13.1
```

With `--output json`, every one of those commands returns one object:

```json
{
  "file": "board.kicad_sch",
  "changed": [{ "change": "+", "record": "S", "handle": "R99", "detail": "4k7 Test:R" }],
  "unchanged": 162,
  "invariants": [
    { "name": "reparses", "passed": true, "faults": [] },
    { "name": "references-resolve", "passed": true, "faults": [] },
    { "name": "geometry-on-grid", "passed": true, "faults": [] },
    { "name": "instances-resolve", "passed": true, "faults": [] }
  ],
  "reformatted": false,
  "symbol": { "uuid": "…", "reference": "R99", "lib_id": "Test:R", "sheet_paths": ["/…"] },
  "notes": [{ "name": "snapped-to-grid", "message": "…" }]
}
```

`changed` is the delta. `invariants` is what kicli checked afterwards.
`reformatted` says whether the file arrived non-canonical and was laid out again
as KiCad's next save would. The key named after the noun — `symbol`, `field`,
`text`, `label`, `junction`, `no_connect` — carries the handles you need to
address what was just made.

## Exit codes

| Code | Name | Meaning |
|---|---|---|
| 0 | success | the command did what was asked; findings are data, not failure |
| 1 | operation | a well-formed request that kicli could not complete |
| 2 | usage | kicli did not understand the flags or the arguments |
| 3 | verification | a mutation failed its own checks and was rolled back |
| 4 | file | a file did not read or parse, or kicli refused to write it |
| 5 | gate | a gate found the findings it was told to fail on |
| 6 | tool | a required external tool is missing or is the wrong version |

### `kicad-cli`'s codes are translated, never passed through

`kicad-cli` gives the same numbers different meanings, so kicli never lets one
of its codes reach you. If you see one of kicli's codes, it means what the table
above says.

| `kicad-cli` | kicli | Why |
|---|---|---|
| 0 | continue | — |
| 1 `ERR_ARGS` | 1 | kicli built a bad command line: a kicli bug, reported with the invocation |
| 2 `ERR_UNKNOWN` | 1 | |
| 3 `ERR_INVALID_INPUT_FILE` | 4 | |
| 5 `ERR_RC_VIOLATIONS` | 1 | should never happen: kicli reads the report instead of asking for this code |
| 6 `ERR_JOBS_RUN_FAILED` | 1 | |
| binary absent | 6 | with the places kicli looked and an install hint |
| major version not 10 | 6 | the file format and the report schema both move between major versions |

## Configuration

`kicli.toml`, in the project directory. **An unknown key is an error**, not a
warning, so a misspelled setting tells you rather than doing nothing.

```toml
[grid]     step = "50mil"       # also "1.27mm" or "8G", whole grid steps
[view]     max_bytes = 32768
[formats]  max_schematic_version = 20260306
[tools]    kicad_cli_path = "/opt/kicad/bin/kicad-cli"
```

## Two things that will bite you

**A pin touching a wire is not connected to it.** In KiCad 10.0.5 a pin whose
connection point lands on a wire's interior does *not* join that wire; a
junction is required. A label in the same place *does* join it — unless a pin
shares the anchor, in which case the label and the pin form their own net and
the wire is left out. All three cases draw identically. kicli follows KiCad
exactly, so its net list is KiCad's net list, and the case that looks connected
and is not will be a blocking finding when the style rules land.

**A reference designator belongs to a sheet path, not to a symbol.** A sheet
placed twice has one symbol object and two references. Views show the reference
for the placement they are describing, and `--sheet` picks the placement.
