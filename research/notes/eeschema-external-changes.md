# Eeschema does not notice that its open file changed

Established against KiCad **10.0.5** (`kicad-cli --version`, and
`KiCad.app` `CFBundleShortVersionString`, both 10.0.5 on the machine that ran
the session). Two halves: KiCad's own source read at that tag, and a session in
front of the running editor on 2026-08-14. kicli edits files on disk, and an
editor holding the same file is the obvious hazard; `spec/SPEC.md` §14.4 warned
about it in words written from assumption, which is not how anything else in
this project is decided.

**All four questions are answered.** Every claim below says whether it was read
from source or watched happening, and the two that are both say so.

**The short version.** An open Eeschema never learns that its file changed. It
does not warn when kicli writes, and it does not warn when its own save
overwrites that write. The loss is silent at both ends and is found afterwards,
by looking at the file. `File → Revert` is the only way the write reaches the
screen, and it discards every unsaved change of the whole hierarchy and the undo
history with it.

## What was established, and how

The source instrument is KiCad's tree at tag `10.0.5`, read for three things: a
file watcher on the open schematic, a modification-time check, and a reload
path. The GUI instrument is the editor itself, on the build named above.

**The source control.** A search that finds nothing proves nothing about the
search. The same search, in the same tree, **does** find a watcher:
`SCH_BASE_FRAME::setSymWatcher` in `eeschema/sch_base_frame.cpp` builds a
`wxFileSystemWatcher` over the **symbol library** of the symbol being edited —
the `.kicad_sym` file, or the directory for a directory-based library. Its
handler `OnSymChange` debounces for a second, compares the stored timestamp
against the file's, and asks the user "The library containing the current symbol
has changed. Do you want to reload the library?". So the instrument can see a
watcher when there is one, and the negative results below are evidence rather
than absence of effort.

**The GUI control.** `kicli project check` on the copied project reported no
findings before the session started, so the project was healthy and every change
below is one the session made.

### 1. Eeschema does not watch the schematic it has open — established from source

`eeschema/files-io.cpp` at 10.0.5 carries no file-system watcher over a
`.kicad_sch`, no comparison of a schematic's modification time, and no path that
reloads a schematic because the file changed. The only watcher in Eeschema is
the symbol-library one above, and it is scoped to `LIBRARY_TABLE_TYPE::SYMBOL`.

**Consequence.** An open Eeschema holds the schematic it read at open time. A
write by kicli, or by anything else, is invisible to it. There is no prompt,
because there is no mechanism that could raise one.

### 2. A save from Eeschema overwrites the external change — measured

With the schematic open in Eeschema, kicli added a label, and the file on disk
carried it:

```
> kicli label add --text PROBE_A --at 100,100
+ T a2da39ad "PROBE_A"
checked: every invariant passed
no net with a pin changed
note: The anchor moved onto the grid, from 100,100 to 100.33,100.33.
note: No wire passes through this anchor. The label names nothing yet.

> grep -c PROBE_A main-board.kicad_sch
1
```

Then a symbol was nudged in the open editor and the file was saved from
Eeschema:

```
> grep -c PROBE_A main-board.kicad_sch
0
```

**The label is gone.** Eeschema wrote the document it read when it opened the
file, which knew nothing of kicli's write. This is what (1) predicts, and it is
now watched happening rather than inferred.

### 3. Revert reloads from disk, and throws away every unsaved change — established from source and measured

`ACTIONS::revert` in `common/tool/actions.cpp` is "Revert" / "Throw away
changes", and Eeschema puts it in the File menu (`eeschema/menubar.cpp`). Its
handler is `SCH_EDITOR_CONTROL::Revert` in
`eeschema/tools/sch_editor_control.cpp`. It:

1. asks "Revert '%s' (and all sub-sheets) to last version saved?";
2. sets `SetContentModified( false )` on **every screen of the hierarchy**, with
   the comment `do not prompt the user for changes`;
3. calls `ClearUndoRedoList()`;
4. re-opens the project with `OpenProjectFiles( …, KICTL_REVERT )`.

In the editor, with a second kicli write on disk and a small unsaved edit of the
editor's own:

```
> kicli label add --text PROBE_B --at 120,100
+ T 2352b703 "PROBE_B"
checked: every invariant passed
no net with a pin changed
note: The anchor moved onto the grid, from 120,100 to 119.38,100.33.
note: No wire passes through this anchor. The label names nothing yet.
```

`File → Revert` raised one confirmation dialog, naming the file and **all
sub-sheets**, as the source string above says. After pressing Yes:

- **`PROBE_B` is on the canvas.** Revert is how an external write reaches the
  screen.
- **Undo does nothing.** The undo list is cleared, so the editor's own unsaved
  edit — which the reload from disk replaced — cannot be brought back.

### 4. What the editor shows at the moment of the write, and at the moment of the save — measured: nothing

No dialog, and no observed symptom, at either moment. Nothing appeared when
kicli wrote the file underneath the open editor, and nothing appeared when the
editor saved over that write. kicli replaces a file by writing a temporary
beside it and renaming over the target, so the editor's original descriptor
stays valid and even the platform gives it nothing to notice.

**This is the finding that matters.** The hazard is not that the editor warns
badly. It is that the whole sequence — write, overwrite, loss — is silent, and
the only evidence is the file afterwards.

## The reproduction recipe

Ten minutes, KiCad 10.0.5, any project kicli can write.

```sh
cp -r crates/kicli/tests/fixtures/sch/nets /tmp/watch && cd /tmp/watch
kicli project check -p /tmp/watch      # the control: no findings before the session
open -a KiCad nets.kicad_pro           # then open the schematic editor
```

Then, in order, writing down what the editor shows at each step:

1. **No unsaved changes in the editor.** Run
   `kicli label add --text PROBE_A --at 30.48,88.9 -p /tmp/watch`, and
   `grep -c PROBE_A nets.kicad_sch` to confirm the file carries it. *Observe:*
   the canvas, any dialog, the title bar and the modified marker. Wait a minute —
   the symbol-library watcher debounces for a second, so a schematic watcher, if
   one existed, would have fired by then.
2. **Move a symbol in Eeschema and save.** *Observe:* any dialog, then
   `grep -c PROBE_A nets.kicad_sch`. Zero is the overwrite.
3. **Write again from kicli** with a different label name, make a small edit in
   the editor and do not save it, then **File → Revert**. *Observe:* the dialog's
   words, whether the new label is on the canvas, and whether Undo brings the
   editor's own edit back.

## What kicli should say and do about it

- kicli's IPC probe (`spec/SPEC.md` §14.4) warns that a document is **open**. It
  cannot warn that the editor will overwrite the write, because nothing in the
  editor knows about it. The warning's job is to tell a person to press Revert.
- The advice for an agent working beside an open editor is therefore one line:
  **after kicli writes, the person at the editor must use File → Revert**, and
  Revert throws away every unsaved change in the whole hierarchy and the undo
  history with it. Save the editor's own work first.
- Until that Revert happens, any save from the editor silently discards kicli's
  write. There is no warning at either end and no marker to look for, so an
  agent that cannot see the screen should treat an open document as a write it
  may have to make again, and should say so.
- kicli replaces a file by writing a temporary beside it and renaming over the
  target. An editor that holds an open descriptor keeps reading the file it
  opened; the rename does not disturb it. That is another reason nothing in the
  editor notices.
