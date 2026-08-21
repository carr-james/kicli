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
kicli <noun> <verb> [<handle>] [flags]
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
  nets       0

sheets 2
  page 1  symbols 0  power 0  file healthy.kicad_sch  path /0000...0000
  page 2  symbols 0  power 0  name stage  file stage.kicad_sch  path /0000...0000/0000...0001
```

**`nets` counts every net in the whole project.** Every one: across all sheets,
power nets included, and including the single-pin nets that are one pin joined to
nothing. `sch view` counts a smaller number on purpose, and the two are
reconciled under the connectivity view below. If the two disagree by more than
that reconciliation, you are looking at different scopes, not at a bug.

**This command runs `kicad-cli`, and so does `project check`.** Before either of
them does, it prints a note on **standard error**:

```
kicli: asking /usr/bin/kicad-cli its version. The first KiCad run on a machine builds the font cache. It can take over 120 seconds.
```

It is a warning, not a report. kicli says what it is about to do before it
blocks, because an agent that sees nothing for two minutes decides kicli has
hung. **The note appears whenever `kicad-cli` is found; it does not mean the
cache was cold this time.** It never appears on standard output, so it cannot
corrupt a parse, and `--quiet` silences it.

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

`project check` runs `kicad-cli` when one is present, and prints the same
standard-error note before it does as `project info` does — see there for what
the note is and how to silence it. The first run on a machine can take over two
minutes; later runs take under a second.

### `kicli sch view`

The compact views. **This is what you act on.** Everything else kicli will
eventually draw is a picture of the same data; the view is the data.

| Flag | What it does |
|---|---|
| `--view connectivity\|layout\|delta` | Which view. Connectivity is the default. |
| `--include-power` | List power symbols, which are otherwise left out. |
| `--uuids` | Add the first eight characters of each object's identifier. |
| `--stats` | Report the size of the view in bytes. |
| `--against <NAME>` | The saved state the delta compares against. The default is `@last-write`. The other two views ignore it. |

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

**`nets=` here and `nets` in `project info` count different things, and they
reconcile.** `nets=` is how many `N` records follow — nets with at least one pin
this view is showing. Two kinds are left out of it:

- A net that is one pin joined to nothing is not listed at all. It is tallied
  instead, as `# 18 pin(s) join nothing`, because listing eighteen nets of one
  pin each would cost a fifth of the view to say nothing.
- A net with no pin this view is showing is silently absent — a net of only
  power pins when you did not pass `--include-power`, or a net drawn entirely on
  another sheet when you passed `--sheet`.

So for a **whole-project** view with nothing hidden, `nets=` plus the
`join nothing` tally is `project info`'s `nets`. The example above is a real
project: `14 + 18 = 32`, and `project info` on it prints `nets 32`. A
**per-sheet** view does not add up to the project figure and is not meant to —
that same project's root sheet gives `10 + 18 = 28` against a project total of
32, because the pins on the other two sheets are not the root sheet's to count.

#### The layout digest

Where things are drawn. No connections. Coordinates are millimetres to two
decimals.

```
page A4  sheet /0000...0000
L R1 50.80 50.80 0 - 2.03x7.62
F R1.Reference 0.00 0.00 0
T local NET_A 50.80 41.91 0
W 15 segments, 2 junctions, 1 crossings
```

| Record | Meaning |
|---|---|
| `L` | A symbol: reference, x, y, angle, mirror (`x`, `y` or `-`), body size |
| `F` | A field that has moved off the position its library gives it |
| `T` | A label or free text: kind, text, x, y, angle. The kind is `local`, `global`, `hierarchical`, `netclass` or `text` |
| `W` | The wire summary for the sheet |

A `T` kind is written with the word the command takes, so `T hierarchical IN …`
is added again by `label add --kind hierarchical`. `netclass` marks a netclass
flag and `text` marks free text; neither is made by `label add`.

`L` carries the angle and mirror exactly as the file writes them, because that
is what you pass back to a rotate command. `F` lists a field **only** when it has
moved, so on a tidy sheet it is empty and on an untidy one it is the list of
things to fix. A crossing is counted only where no junction sits on it: a
junctioned crossing is a connection.

#### The delta

What has touched the file **since kicli last wrote it**. It is not a replay of
what your own last command changed: that command already reported it, and
nothing derives it again. Right after any kicli write, this view is empty. That
is the design, not a fault.

```
# delta @last-write -> current  scope=sheet  sheet=/0000...0000  compared=values
~ L R1  moved  (50.80,50.80) -> (50.80,63.50)
~ S R2.Value  "1k" -> "2k2"
+ S R42 10k Device:R
- S R7 4k7 Device:R
= 231 objects unchanged
```

| Header field | What it says |
|---|---|
| `delta A -> B` | The saved state, and the file as it is now. |
| `scope` | `sheet`, or `sheet-summary` when the lines do not fit the budget. |
| `sheet` | The sheet path the saved state covers. One state covers one sheet. |
| `compared` | `values` when the saved state carries the old positions and values, so a line can print them. `hashes` when it carries only hashes and names, which is what a state written by an older kicli holds. |

`+` is a new object, `-` is one that is gone, and `~` is one that moved or whose
content changed. The record letter is the one the other views print, so `L` is a
placement, `S` a symbol or a field, `T` a label or free text, and `W` a wire.

`--against <NAME>` compares against a saved state of another name.
`--include-power` and `--uuids` do not change a delta, and kicli says so on
standard error rather than ignoring them silently.

**A project kicli has never written has no saved state.** That is an error
naming `@last-write`, and exit 1. kicli does not tell you that nothing has
touched a file it has never seen.

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

**A positional argument is always a handle.** It names something that is already
in the drawing, and the verb acts on that object: `sym move R12`, `field hide
R12 --name Value`, `label delete da5aa983`, `net rename NET_A --to NET_Z`.
Everything a command makes or sets is a named flag.

A verb that makes a new object therefore takes **no** positional at all. The new
object's own text goes in a flag: `label add --text SPY`, `text add --text
"note"`, `sym place --lib-id Device:R`. This is why `label delete SPY` and
`label add --text SPY` are different shapes — the first addresses a label that
exists, the second makes one — and it is what stops a typo in a name from
reading as a handle.

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
placement still uses it, and goes if none does. The report says which way it
went, as a note named `definition-kept` or `definition-removed`, so you never
have to read the file to find out.

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

### `kicli wire draw`

```
kicli wire draw --from-pin <REF.PIN> | --from-port <NAME> | --from-at <X,Y>
                --to-pin   <REF.PIN> | --to-port   <NAME> | --to-at   <X,Y>
                [--via <X,Y>]... [--auto-labels]
```

You give the corners. kicli does **no searching**: it checks that what you asked
for is drawable and refuses rather than drawing something illegal. Give `--via`
once per corner, in the order the wire meets them.

Each end takes one of three forms, because a port name can look like anything
and kicli will not guess which kind a word is. Name exactly one form per end.

Four rules decide, and every refusal names the vertex it is about. Every vertex
sits on the grid. Every segment runs along one axis. Nothing a wire may not
cross is in the way. And the wire leaves each end the way that end must be
left — a wire leaves a pin along the pin's own direction, so the first corner
after a pin is straight out from it.

**A vertex off the grid is refused, not snapped.** Every other verb snaps a
position you give it. A polyline is not a position: moving one corner changes
the shape of the wire you asked for, and can turn a legal path into one that
runs along another net.

The result is the route contract, then the usual mutation report:

```
routed R1.1 -> R2.1   via 3 segments, 2 corners, 35.56mm
  cost 44 = length 28 + turns 12 + crossings 0 + text 0 + proximity 4
+ W 3300f00e 50.80,45.72..50.80,50.80
checked: every invariant passed
```

| Line | What it says |
|---|---|
| the first | the status, the two ends, and what the route is: segments, corners, length |
| `cost` | the total, then the five parts it is the sum of |
| `crossings` | how many other nets the route crosses, each with its wire and, when kicli knows it, its net. Absent when there are none |
| `adjusted` | a terminal kicli had to move, how far **by**, and why. Absent when it moved none |
| `wires added` | what reached the file. Absent when nothing did |

**Read the cost, not just the total.** The parts are there so you can decide to
move a symbol instead of accepting a bad route. `turns 12` on a two-corner route
is normal; `crossings 20` is one net cut, and a cut net is usually worth a move
rather than a wire. **The decision is the point of the breakdown**, and
`kicli wire connect` below works one all the way through.

The status is one of four words. `routed` and `labels` are answers and print the
contract above. `blocked` and `invalid` are refusals: they print one
`kicli: ...` line instead — `{"error": {"kind": "operation", ...}}` in JSON —
and write nothing.

| Status | Exit | What it means | What you do next |
|---|---|---|---|
| `routed` | 0 | a wire was drawn | read the `cost` parts. Accept it, or change the drawing and route again |
| `labels` | 0 | a pair of labels is **proposed** and nothing was written. A proposal is a result, not a failure | run the same command with `--auto-labels`, or choose the path yourself with `wire draw` |
| `blocked` | 1 | the request was drawable and the way was barred. The message names what barred it | move what it names, or join the ends with `--auto-labels` |
| `invalid` | 1 | the request itself was not drawable: a diagonal, a vertex off the grid | fix the request. Nothing in the drawing has to change |

**`--auto-labels` writes a pair of labels instead of a long wire.** A connection
longer than `routing.label_threshold` reads better as two labels than as a wire
across the sheet. Give the flag and kicli writes the pair; leave it off and
kicli draws exactly what you asked for, however long it is.

```
kicli wire draw --from-pin U1.1 --to-pin U2.2 --auto-labels
```
```
labels U1.1 -> U2.2
  reason: path length 462.28mm is over the threshold 381.00mm
  labels: "U1_SCK" at 12.70,196.85 and 285.75,12.70
  wires added: 2   junctions added: 0
+ W d58d24fb 285.75,10.16..285.75,12.70
+ W f4ba12b5 12.70,196.85..12.70,199.39
+ T 43aab6e9 "U1_SCK"
+ T 479f4a3c "U1_SCK"
checked: every invariant passed
note: auto-labels  kicli wrote the label "U1_SCK" at each end instead of a wire. Each label sits on a short stub from its own pin. Nothing joins the two ends but the name they share.
```

Four things to know about it.

The name is the net's own name when the drawing gives the pin one, and
`<reference>_<pin name>` when it does not. A pin with no name of its own is
named by its number instead.

Each label sits **two grid steps along its own pin's direction**, on a short
stub drawn from the pin to it. The stub is what makes the label the pin's: a
label standing off a pin with nothing between them names a net that pin is not
on. `wires added` counts those stubs, and no wire joins the two ends.

The flag needs a pin at one end, because that is where an unnamed net's name
comes from. A request with no pin at either end is drawn rather than labelled.

The length judged is the path you gave. Give no `--via` and it is the Manhattan
distance between the two ends, which no orthogonal wire could beat — so
`--auto-labels` with two pins and no corners is the whole request.

With `--output json`, the contract is under the `wire` key of the mutation
result, and every key is there whatever the status — an empty list where nothing
happened, `null` where there is no value. You never have to ask which keys came
back.

```json
{ "status": "routed", "from": "R1.1", "to": "R2.1",
  "path": [[50.8,50.8],[50.8,45.72],[76.2,45.72],[76.2,50.8]],
  "segments": 3, "corners": 2, "length_mm": 35.56,
  "cost": { "total": 44, "length": 28, "turns": 12, "crossings": 0,
            "text": 0, "proximity": 4 },
  "crossings": [], "adjusted": [],
  "added": { "wires": ["…","…","…"], "junctions": [] },
  "labels": null, "blocked_by": [], "reason": null,
  "alternatives_considered": 0 }
```

`adjusted` carries `{ terminal, by, why }`. **`by` is a displacement, not a
position** — where the terminal ended up is the matching end of `path`, and the
point you asked for is that end less `by`. `why` is a closed set, currently the
single word `four-way`, so you can switch on it and never parse English.

### `kicli wire connect`

```
kicli wire connect --from-pin <REF.PIN> | --from-port <NAME> | --from-at <X,Y>
                   --to-pin <REF.PIN> | --to-port <NAME> | --to-at <X,Y> | --to-net <NET>
                   [--auto-labels]
```

kicli chooses the path here: the silhouettes a person would draw first, and a
search when none of them fits. There is no `--via` — the corners are the
router's to pick, and if you want to pick them yourself the verb is `wire draw`.
The answer is the same route contract, with the same four statuses and the same
cost breakdown, so everything above applies.

One extra line sits on top of it, and in JSON it is a top-level `"net"` key
beside `"wire"` — `null` when nothing was joined:

```
joined: net #n5
```

That name is **read back out of the file kicli has just written**, not predicted
from the route, so it is what the drawing now says.

**A successful connect writes immediately.** There is no dry run: if you decide
against the route, delete the wires the report just named you.

**`--to-net` joins a whole net**, by its drawn name or by the `#n3` handle
`sch view` gives an unnamed one. kicli routes to the cheapest point of the net —
any grid point of its wires, or any of its pins — and `to` says which point it
joined:

```
routed R11.2 -> #n3@35.56,88.9   via 3 segments, 2 corners, 12.70mm
```

A handle names a position in the view you just read, not a conductor: joining a
net renumbers them, and the `#n3` above comes back as `#n2` once the write lands.
Re-read the view rather than reusing a handle. Quote it, too — `#` opens a
comment in most shells. A name that answers for **more than one** net, which the
same local label text on two sheets does, is refused with both nets and their
pins listed. There is no `--from-net`: a route leaves one point, and which point
of a net it should leave is not a question the escape rule can answer.

**A route that ends on the interior of a wire gets a junction; one that ends
where a wire already ends does not**, because KiCad draws a corner there.
`junctions added` says which happened.

**`--auto-labels` covers one case more here than under `wire draw`.** kicli
proposes a pair of labels when the cheapest route is longer than
`routing.label_threshold`, *and* when no route joins the two ends at all. The
second kind says `reason: no route reaches ...` and adds a `blocked by:` line
naming every obstacle it met, which is the list to move something out of.

#### Worked: read the cost, then change the drawing

`R30` and `R31` are 88.9mm apart with another net's wire between them.

```sh
kicli wire connect --from-pin R30.1 --to-pin R31.1
```
```
joined: net #n5
routed R30.1 -> R31.1   via 3 segments, 2 corners, 93.98mm
  cost 110 = length 74 + turns 12 + crossings 20 + text 0 + proximity 4
  crossings: 1 (at 170.18,171.45 on wire e58e0c77)
  wires added: 3   junctions added: 0
+ W 2fbd461d 215.90,171.45..215.90,173.99
+ W a3506aad 127.00,171.45..215.90,171.45
+ W f1a99ad0 127.00,171.45..127.00,173.99
checked: every invariant passed
```

Cost 110, and the parts say where it went: 74 of it is length and 20 is one
crossing. Nothing about that is a failure — it is an invitation to move `R31`
instead. Delete the three wires the report named, move, and route again:

```sh
kicli wire delete 2fbd461d
kicli wire delete a3506aad
kicli wire delete f1a99ad0
kicli sym move R31 --to 152.4,177.8
kicli wire connect --from-pin R30.1 --to-pin R31.1
```
```
joined: net #n5
routed R30.1 -> R31.1   via 3 segments, 2 corners, 30.48mm
  cost 40 = length 24 + turns 12 + crossings 0 + text 0 + proximity 4
  wires added: 3   junctions added: 0
+ W 2fbd461d 152.40,171.45..152.40,173.99
+ W a3506aad 127.00,171.45..152.40,171.45
+ W f1a99ad0 127.00,171.45..127.00,173.99
checked: every invariant passed
```

110 to 40, and the crossing is gone. Only the last command's answer is shown;
the three deletes and the move each print their own one-line delta and
`checked: every invariant passed`.

**Delete the wires first, and mean it.** `sym move` moves the symbol and its
fields, and **not** the wires that met its pins: a symbol moved out from under a
route leaves the route behind, joined to nothing, and the net it was on quietly
disappears from `sch view`. That is why the undo comes before the move.

### `kicli wire delete`

```
kicli wire delete <TARGET>
```

Removes the one segment you named and **nothing else**. It does not tidy up
after itself: a junction the removal leaves sitting on fewer than three wire
ends is still legal, and taking it away is a second decision that is yours.
kicli reports every such junction as a note and leaves it in the file.

```
- W 3300f00e 50.80,50.80..63.50,50.80
checked: every invariant passed
note: stranded-junction  the junction 01000003 at (63.5,50.8) now joins 1 wire end(s), and is still there. Run junction delete --at 63.5,50.8 to take it away.
```

A bus is refused rather than deleted: a bundle carries several nets, and
removing one is not this verb's decision.

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

### "What changed?" is two questions

Read this before you go looking for a command that replays your own edits. There
isn't one, and there should not be.

| Question | Answered by | Empty when |
|---|---|---|
| What did **this command** change? | the result the command just printed | never |
| What has touched the file **since kicli last wrote it**? | `kicli sch view --view delta` | right after any kicli command, **by design** |

Every mutating command reports its own changes. **Keep that output** if you need
it later; nothing re-derives it for you.

`.kicli/@last-write` records the file **after** kicli's write, so a comparison
against it is empty until something *else* touches the file — a person editing in
KiCad beside you, another tool, a checkout or a merge. That is what it is for.
Making it replay your own edits would make it a copy of the report you already
have, and would leave you no way to notice the person editing beside you.

`kicli sch view --view delta` prints that comparison. See **The delta** above.

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
warning, so a misspelled setting tells you rather than doing nothing. kicli
validates every section it knows, including ones no command reads yet, so a typo
is an error the moment you write it.

```toml
[grid]     step = "50mil"       # also "1.27mm" or "8G", whole grid steps
[view]     max_bytes = 32768
[formats]  max_schematic_version = 20260306
[tools]    kicad_cli_path = "/opt/kicad/bin/kicad-cli"
```

### `[routing]`

What a wire costs, and how far kicli looks. The commands that read it are
`kicli wire draw`, `kicli wire connect` and `kicli wire delete`.

```toml
[routing]
label_threshold = "300G"  # above this, a pair of labels is proposed, not a wire
margin          = "8G"    # how far outside the wire kicli looks for obstacles
u_max           = "6G"    # how far outward a U-shaped route may reach
w_len           = 1       # the cost of one grid step of wire: the base unit
w_turn          = 6       # the cost of one corner
w_cross         = 20      # the cost of crossing another net
w_text          = 12      # the cost of one grid step inside a label or text box
w_near          = 2       # the cost of one grid step beside a symbol body
```

Distances take `"300G"` (whole grid steps), `"1.27mm"` or `"50mil"`. Weights are
whole numbers and **none of them may be negative** — a negative term would make
a longer route cheaper.

The weights are what the `cost` line in a route's report is built from, so
`w_turn = 6` is why a corner shows as `turns 6` per corner. Raise `w_cross` to
make kicli detour further to avoid cutting another net; raise `w_turn` to make
it prefer a longer straight run.

**`label_threshold` is one knob read twice.** The router decides with it and the
long-wire style rule judges with it. Changing it changes both, on purpose: a
tool that draws at one distance and complains at another argues with itself.

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

## Working beside an open editor

**Eeschema does not notice that its file changed.** KiCad 10.0.5 puts no watcher
on the schematic it has open. No prompt appears when you write, and a save from
Eeschema overwrites what kicli wrote — also with no prompt — because the editor
is writing the document it read when it opened the file. Both silences are
measured in the running editor.

Tell the person at the editor to use **File → Revert**. That reloads from disk
and is how your edit reaches their screen. Warn them first: Revert discards every
unsaved change in the **whole hierarchy**, not only the sheet on screen, and it
clears the undo history, so nothing it throws away can be recovered. They should
save their own work before you write.

Until that Revert happens, treat your write as one you may have to make again:
any save from the editor discards it, nothing warns anybody, and you cannot see
the screen. Say so when you report the write.

The evidence, the transcript and the recipe are in
`research/notes/eeschema-external-changes.md`.
