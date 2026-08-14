# Eeschema does not notice that its open file changed

Established 2026-08-14 against KiCad **10.0.5**, by reading KiCad's own source
at that tag. kicli edits files on disk, and an editor holding the same file is
the obvious hazard; `spec/SPEC.md` §14.4 warned about it in words written from
assumption, which is not how anything else in this project is decided.

**Read this first: two of the four questions are answered from source, and two
still want a session in front of the running editor.** Every claim below says
which it is. The recipe at the end is what settles the rest, and it takes about
ten minutes.

## What was established, and how

The instrument is KiCad's source at tag `10.0.5`, read for three things: a file
watcher on the open schematic, a modification-time check, and a reload path.

**The control.** A search that finds nothing proves nothing about the search. The
same search, in the same tree, **does** find a watcher: `SCH_BASE_FRAME::
setSymWatcher` in `eeschema/sch_base_frame.cpp` builds a `wxFileSystemWatcher`
over the **symbol library** of the symbol being edited — the `.kicad_sym` file,
or the directory for a directory-based library. Its handler `OnSymChange`
debounces for a second, compares the stored timestamp against the file's, and
asks the user "The library containing the current symbol has changed. Do you
want to reload the library?". So the instrument can see a watcher when there is
one, and the negative results below are evidence rather than absence of effort.

### 1. Eeschema does not watch the schematic it has open — established

`eeschema/files-io.cpp` at 10.0.5 carries no file-system watcher over a
`.kicad_sch`, no comparison of a schematic's modification time, and no path that
reloads a schematic because the file changed. The only watcher in Eeschema is
the symbol-library one above, and it is scoped to `LIBRARY_TABLE_TYPE::SYMBOL`.

**Consequence.** An open Eeschema holds the schematic it read at open time. A
write by kicli, or by anything else, is invisible to it. There is no prompt,
because there is no mechanism that could raise one.

### 2. A save from Eeschema overwrites the external change — established by
### construction, not yet watched happening

It follows from (1): Eeschema writes the document it holds in memory, which was
read before kicli's write and knows nothing of it. Nothing reads the file again
in between. This is inference from (1) rather than an observation, and the recipe
below is what turns it into one.

### 3. Revert reloads from disk, and throws away every unsaved change —
### established

`ACTIONS::revert` in `common/tool/actions.cpp` is "Revert" / "Throw away
changes", and Eeschema puts it in the File menu (`eeschema/menubar.cpp`). Its
handler is `SCH_EDITOR_CONTROL::Revert` in
`eeschema/tools/sch_editor_control.cpp`. It:

1. asks "Revert '%s' (and all sub-sheets) to last version saved?";
2. sets `SetContentModified( false )` on **every screen of the hierarchy**, with
   the comment `do not prompt the user for changes`;
3. calls `ClearUndoRedoList()`;
4. re-opens the project with `OpenProjectFiles( …, KICTL_REVERT )`.

**Consequence, and it is the useful half of this note.** Revert is how a person
at an open editor picks up what kicli wrote. It is also the widest blast radius
in the file menu: it discards the unsaved work of **every sheet of the
hierarchy**, not only the sheet on screen, and it discards the undo history that
would let any of it be recovered. There is one confirmation dialog and no
per-sheet choice.

### 4. What a prompt looks like, and what happens with unsaved changes of the
### editor's own — **not established**

Source says no external-change prompt exists. It does not say what the editor
does at the moment of a save when the file underneath has moved, whether the
window title or the modified marker changes, or whether the platform's own file
handling (kicli replaces the file by `rename`, so the editor's original inode
survives until it closes it) produces any visible symptom. Those want the running
editor.

## The reproduction recipe

Ten minutes, KiCad 10.0.5, any project kicli can write.

```sh
cp -r crates/kicli/tests/fixtures/sch/nets /tmp/watch && cd /tmp/watch
open -a KiCad nets.kicad_pro     # then open the schematic editor
```

Then, in order, recording what the editor shows at each step:

1. **No unsaved changes in the editor.** Run
   `kicli label add --text WATCHED --at 30.48,88.9 -p /tmp/watch`.
   *Observe:* does the canvas change? does any dialog appear? does the title bar
   or the modified marker change? Wait a minute — the symbol-library watcher
   debounces for a second, so a schematic watcher, if one existed, would have
   fired by then.
2. **Then File → Revert.** *Observe:* the dialog's exact words, and whether the
   label kicli added is on the canvas afterwards.
3. **With unsaved changes in the editor.** Reopen, move a symbol in Eeschema and
   do not save. Run the same `kicli label add` with a different name.
   *Observe:* anything at all.
4. **Then File → Save from Eeschema.** *Observe:* whether the label kicli added
   survives in the file on disk. `grep WATCHED nets.kicad_sch` answers it.
5. **Then File → Revert with unsaved changes present.** *Observe:* the dialog,
   and whether the moved symbol is back where it started — and whether Undo can
   bring it forward again, which (3) above says it cannot.

Write each answer into this note, marked measured, with the date and the build.

## What kicli should say and do about it

- kicli's IPC probe (`spec/SPEC.md` §14.4) warns that a document is **open**. It
  cannot warn that the editor will overwrite the write, because nothing in the
  editor knows about it. The warning's job is to tell a person to press Revert.
- The advice for an agent working beside an open editor is therefore one line:
  **after kicli writes, the person at the editor must use File → Revert**, and
  Revert throws away every unsaved change in the whole hierarchy and the undo
  history with it. Save the editor's own work first.
- kicli replaces a file by writing a temporary beside it and renaming over the
  target. An editor that holds an open descriptor keeps reading the file it
  opened; the rename does not disturb it. That is another reason nothing in the
  editor notices.
