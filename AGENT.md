# kicli for agents

kicli reads and edits KiCad 10 schematics from the command line. It exists so
that an agent can see a schematic without loading a 600 kB file into its
context, and can change one without rewriting the parts it did not touch.

This document is the command reference. It ships with the tool and is kept in
step with it by a test, so a command that is not written down here does not
exist.

**This build reads. It does not yet write.** Mutations, wire routing, the style
score and rendering arrive in later milestones. What follows is what works
today.

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
| `--sheet <PATH>` | One sheet path, instead of the whole project. |
| `--quiet`, `-q` | Results only. Progress notes are suppressed. |
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
