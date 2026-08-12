# R4 — Library resolution and vendoring mechanics

Status: resolution rules verified against KiCad 10.0.5 source and against the
**actual configuration on this machine**, including James's existing Eurorack
shared-library submodule layout, which turns out to already implement most of
SPEC D8's convention (§5).

Prerequisite reading: [`sch-format.md`](sch-format.md) §4 (`lib_symbols`),
[`sexpr-strategy.md`](sexpr-strategy.md) §2.3 (library tables are not in
canonical format in the wild).

---

## ⚠ Contradictions and cautions for `spec/SPEC.md`

1. **SPEC D8 says the shared library lives at "`libs/parts`, configurable". The
   existing convention on this machine is different**: submodule at
   `hardware/shared`, referenced as `${KIPRJMOD}/../shared/symbols/…` from each
   board's project directory (§5). Either change the default to match reality or
   accept a config value everywhere; changing the default costs nothing and
   avoids a permanent mismatch with your own repos. Q1.

2. **Vendoring is not "copy the symbol".** A correct vendor operation touches up
   to seven things (§4.2), and getting any one wrong leaves a project that
   *opens fine* and *fails at fabrication*. This must be one atomic transaction
   with the Constitution §5 verification treatment.

3. **`sym-lib-table` files in the wild are not in KiCad 10 canonical format**
   (35 of 36 in KiCad's own demos, `sexpr-strategy.md` §2.3). kicli's first
   vendoring write will reformat the whole file. Acceptable, but it must be
   stated in the command output, not discovered in a diff.

4. **SPEC §8 has no story for the embedded `lib_symbols` cache**, which is the
   thing that actually determines what KiCad draws. Vendoring must update it
   too, or the schematic keeps rendering the old symbol (§3).

---

## 1. The resolution chain

To turn `(lib_id "Eurorack Common:R_DIN0207")` into a symbol definition, KiCad:

1. Splits at the **first** `:` → nickname `Eurorack Common`, item name
   `R_DIN0207`.
2. Looks the nickname up in the **project** `sym-lib-table` (a file named
   `sym-lib-table` in the project directory).
3. Falls back to the **global** `sym-lib-table`.
4. Expands `${VAR}` in the row's `uri` against the environment map.
5. Loads that library file and finds the item by name.

For footprints the same applies with `fp-lib-table` and the footprint's
`lib_id`; for 3D models the path inside the `.kicad_mod` is expanded the same
way.

**Nickname shadowing**: a project row with the same nickname as a global row
wins. This is the mechanism vendoring uses, and also the mechanism by which two
projects can disagree about what `Device:R` means.

### 1.1 Global table locations (verified on this machine)

| Platform | Path |
|---|---|
| macOS | `~/Library/Preferences/kicad/10.0/sym-lib-table`, `…/fp-lib-table` |
| Linux | `~/.config/kicad/10.0/…` (same file names) |

The `10.0` component is the KiCad major.minor — KiCad 9's tables sit alongside in
`…/kicad/9.0/`, unmigrated. Verified: this machine has both `9.0/` and `10.0/`
directories.

Sample row from the shipped global table:

```
(sym_lib_table
	(version 7)
	(lib (name "4xxx") (type "KiCad") (uri "${KICAD10_SYMBOL_DIR}/4xxx.kicad_sym") (options "") (descr "4xxx series symbols"))
```

### 1.2 Environment variables

Defined by KiCad (`common/env_vars.cpp:38-46`, `:125-140`), all **version
suffixed** via `ENV_VAR::GetVersionedEnvVarName` → `KICAD{major}_{BASE}`
(`env_vars.cpp:78-83`):

```
KICAD10_SYMBOL_DIR    KICAD10_FOOTPRINT_DIR    KICAD10_3DMODEL_DIR
KICAD10_TEMPLATE_DIR  KICAD10_3RD_PARTY        KIPRJMOD
```

`KIPRJMOD` is the absolute path of the **project directory** of the currently
loaded project (`common/env_vars.cpp:143-146`). It is what makes project-relative
library references portable, and it is the only one kicli should ever write into
a table it generates.

User-defined variables live in
`~/Library/Preferences/kicad/10.0/kicad_common.json` under
`environment.vars`. Verified on this machine:

```json
{ "vars": { "EURORACK_LIB": "/Users/james/code/eurorack/eurorack-common-library" } }
```

**Version-drift hazard (verified, real):** the shared Eurorack footprints
reference

```
(model "${KICAD9_3DMODEL_DIR}/LED_THT.3dshapes/LED_D3.0mm.step"
```

KiCad 10 defines `KICAD10_3DMODEL_DIR`, not `KICAD9_3DMODEL_DIR`. There *is* a
versioned-fallback helper (`ENV_VAR::GetVersionedEnvVarValue`, which accepts any
`KICAD<n>_<BASE>` present in the map) but grepping its call sites at tag
`10.0.5` shows it is used only for `3RD_PARTY` paths
(`scripting/python_scripting.cpp:557`, `kicad/kicad.cpp:237`) — **not** in the
generic path expander. So those model paths resolve only if the user still has
`KICAD9_3DMODEL_DIR` defined. This is exactly the kind of silent rot
`kicli project check` should report. Q2.

---

## 2. Library file formats

| Table | File | Format |
|---|---|---|
| symbols | `sym-lib-table` | s-expression, `(sym_lib_table (version 7) (lib (name …) (type …) (uri …) (options …) (descr …)) …)` |
| footprints | `fp-lib-table` | same shape, `fp_lib_table` |
| design blocks | `design-block-lib-table` | same shape (KiCad 9+) |

Rows are written one per line by `FORMAT_MODE::LIBRARY_TABLE`
(`sexpr-strategy.md` §2.4); older files use two-space indentation and no space
between sibling lists. `type` is `"KiCad"` for native libraries (`.kicad_sym`,
`.pretty` directories); other values (`Legacy`, `Database`, `HTTP`) exist and
kicli must round-trip them even though v1 only vendors `KiCad` type.

A `.kicad_sym` file is `(kicad_symbol_lib (version …) (generator …) (symbol …)*)`
where each `symbol` uses **the same serialiser as the schematic's `lib_symbols`**
(`sch-format.md` §4) — one parser covers both. Verified: the shipped
`Device.kicad_sym` on this machine is `(version 20251024)`, matching
`SEXPR_SYMBOL_LIB_FILE_VERSION` at tag `10.0.5`.

---

## 3. Embedded cache vs external library

Every `.kicad_sch` embeds a copy of each symbol it uses in `lib_symbols`
(`sch-format.md` §4). The relationship:

| Question | Answer |
|---|---|
| What does KiCad draw? | **The embedded copy.** |
| What does ERC check against? | The embedded copy, plus a comparison against the external library, reported as `lib_symbol_mismatch` / `lib_symbol_issues` (warnings by default) |
| When is the embedded copy refreshed? | Only on an explicit "Update Symbols from Library" action |
| What if the external library is missing entirely? | The schematic still opens and renders correctly |

Consequences for kicli:

- **Reading**: resolve `lib_id` against the embedded cache first; the external
  library is only needed for `parts search` and for vendoring.
- **Writing a new symbol placement**: kicli must copy the definition from the
  external library into `lib_symbols` — otherwise the symbol renders as a
  placeholder. This is a mandatory step of `sym place`, not an optimisation.
- **`lib_name` redirection** (`sch-format.md` §3.4): when the embedded key
  differs from the `lib_id`, the symbol carries `(lib_name "…")` and *that* is
  the cache key. Any code resolving by `lib_id` alone is wrong.
- kicli should surface embedded-vs-external drift in `project check`, because it
  silently changes what gets fabricated after someone "updates from library".

---

## 4. Vendoring

### 4.1 The two directions (SPEC D8)

- **`--into project`** — copy a part from wherever it lives into a project-local
  library, so the project is self-contained and immune to upstream changes.
- **`--into shared`** — promote a part from a project into the shared submodule,
  so other projects can use it.

Both are the same mechanical operation with different source/target.

### 4.2 What a correct vendor operation must rewrite

For one symbol, in this order (all-or-nothing, temp files + rename, per
Constitution §5):

| # | Action | Failure if skipped |
|---|---|---|
| 1 | Copy the `symbol` block from source `.kicad_sym` into target `.kicad_sym` (creating it with the right `(version …)` header if new) | nothing to resolve |
| 2 | Copy the referenced **footprint** `.kicad_mod` into the target `.pretty` directory | board fails to load footprint |
| 3 | Copy the footprint's **3D model** files, and **rewrite the `(model "…")` paths** in the copied `.kicad_mod` to the new location | 3D view breaks; STEP export loses bodies |
| 4 | Add/patch the `sym-lib-table` row for the target nickname | `lib_id` unresolvable |
| 5 | Add/patch the `fp-lib-table` row | footprint unresolvable |
| 6 | Rewrite `(lib_id "old:Name")` → `(lib_id "new:Name")` on every affected symbol **in every sheet** of the project | still points at the old library |
| 7 | Rewrite the **embedded `lib_symbols` key** (and any `(lib_name …)`) to match, or refresh the embedded copy from the new source | schematic renders the stale symbol (§3) |

Plus: the symbol's `Footprint` **field value** on each instance contains
`libnick:FootprintName` and must be rewritten when the footprint nickname
changes (this is a *field* edit on both the instance and, if present, the
library symbol's default). Missing this is the most common vendoring bug — the
schematic looks right and the netlist points at a footprint library that is no
longer in the table.

### 4.3 Verification after vendoring

kicli should assert, before committing the write:

1. Every `lib_id` in every sheet resolves through the *new* tables.
2. Every `Footprint` field value resolves through `fp-lib-table`.
3. Every `(model …)` path in every copied footprint resolves (after env
   expansion) to an existing file.
4. The embedded `lib_symbols` entry is byte-identical to the target library's
   symbol (modulo the name key).
5. Re-parse of every written file succeeds and round-trips.

Steps 1–3 are also exactly what `kicli project check` should run standalone —
they catch the version-drift case in §1.2 and the "submodule not initialised"
case in §6.

---

## 5. The shared-library convention (as it actually exists)

From James's `quad-vca-mixer` repository on this machine:

```
quad-vca-mixer/
  .gitmodules            → [submodule "hardware/shared"]
                             path = hardware/shared
                             url  = https://github.com/carr-james/eurorack-common-library.git
  hardware/
    shared/              ← the submodule
      symbols/eurorack-common.kicad_sym
      footprints/…       (.pretty directories)
      3dmodels/          (.wrl and .step)
      spice/
      design-rules/
    control-board/
      control-board.kicad_pro / .kicad_sch / .kicad_pcb
      sym-lib-table      → (lib (name "Eurorack Common") (type "KiCad")
                                (uri "${KIPRJMOD}/../shared/symbols/eurorack-common.kicad_sym"))
      fp-lib-table       → (lib (name "Eurorack Common") (type "KiCad")
                                (uri "${KIPRJMOD}/../shared/footprints/eurorack-common.pretty"))
    bottom-board/  middle-board/  front-panel/     ← same pattern
```

and schematics reference it as `(lib_id "Eurorack Common:R_DIN0207")`.

This is a good convention and kicli should adopt it as the default:

- one submodule per repository, shared by several board projects;
- referenced via `${KIPRJMOD}/../shared/…` so it works from any board directory
  and survives cloning;
- symbols/footprints/3dmodels as sibling directories with matching base names.

**Two things to fix or at least flag:**

1. **The nickname contains a space** (`Eurorack Common`). Legal, but it forces
   quoting everywhere, and `lib_id` strings with spaces are a common source of
   breakage in third-party tooling. kicli should warn on space-containing
   nicknames and offer `--rename-nickname` as a project-wide rewrite. Q3.
2. **`${KICAD9_3DMODEL_DIR}` references** (§1.2) should be migrated.

Derived defaults for `kicli.toml`:

```toml
[libraries]
shared_path   = "../shared"          # relative to the project dir (${KIPRJMOD})
shared_nick   = "Eurorack Common"
symbols_dir   = "symbols"
footprints_dir= "footprints"
models_dir    = "3dmodels"
```

---

## 6. Failure modes

| Mode | Symptom | Detection |
|---|---|---|
| Submodule not initialised | every shared `lib_id` unresolvable; schematic still renders (embedded cache) but ERC warns | `uri` path does not exist → `project check` |
| Nickname collision (project row shadows global) | a symbol silently resolves to a different part | compare project + global nickname sets |
| Nickname with spaces | third-party tool breakage, quoting bugs | lint on table load |
| Version drift in env vars (`KICAD9_*`) | 3D models missing; STEP export incomplete | expand every `${…}` and check existence |
| Embedded cache drift | KiCad draws the old symbol; "update from library" silently changes the netlist later | compare embedded vs external, per symbol |
| Project moved/renamed | `${KIPRJMOD}`-relative URIs still fine; **absolute** URIs break | flag absolute URIs in tables |
| Two projects vendoring the same part into `shared` with different content | last writer wins; silent divergence | hash the symbol on vendor-up; refuse on conflict unless `--force` |
| Footprint field points at a nickname not in `fp-lib-table` | board load error at PCB time, long after the schematic edit | check all `Footprint` field values against the table |
| Legacy `.lib`/`.dcm` libraries (`type "Legacy"`) | KiCad 10 can still read some; kicli should not try to write them | refuse with a clear message |

---

## 7. Implications for kicli's design

1. A `LibraryResolver` that owns: the two tables (project + global), the env map
   (including `kicad_common.json` user vars), and a cache of loaded
   `.kicad_sym` files. Every `lib_id` → definition goes through it.
2. Table writing reuses the s-expression emitter in `LIBRARY_TABLE` mode
   (`sexpr-strategy.md` §2.4) and reports reformatting.
3. `kicli parts search` reads the shared library directly — no KiCad needed —
   and indexes on name, value, keywords, footprint, description.
4. `kicli lib vendor` is a transaction over multiple files. Since there is no
   undo (SPEC D14), the implementation must stage every write to temp files,
   verify §4.3, and only then rename — and on failure remove the temps and touch
   nothing.
5. `kicli project check` runs §4.3's assertions plus §6's detections. This is
   probably the single most useful command for James day-to-day, and it needs
   nothing but the parser.

---

## 8. Open questions for James

- **Q1 — Default shared path.** Adopt `../shared` + nickname from the existing
  Eurorack layout as the default (instead of SPEC's `libs/parts`)?

- **Q2 — Env-var migration.** Should `kicli project check` *report*
  `${KICAD9_*}` references, or should `kicli lib migrate-envvars` rewrite them to
  `${KICAD10_*}`? (Reporting is safe; rewriting touches the shared submodule,
  which other projects depend on.)

- **Q3 — Nickname with a space.** Warn only, or offer the project-wide rename?
  A rename touches every `lib_id` in every sheet plus both tables — mechanical,
  but it is a big diff.

- **Q4 — Vendor-up conflict policy.** When `--into shared` would overwrite an
  existing part with different content: refuse, refuse-unless-`--force`, or
  auto-suffix the name (`R_DIN0207_2`)? Recommendation: refuse with a diff
  summary.

- **Q5 — Scope of "copy the 3D model".** Model files can be tens of MB. Copy,
  symlink, or reference the upstream path? Recommendation: copy for
  `--into project` (self-containment is the point), reference for
  `--into shared`.

---

## 9. Sources

- KiCad 10.0.5 source, tag `10.0.5`: `common/env_vars.cpp:38-146`,
  `common/filename_resolver.cpp`, `common/libraries/library_table.cpp`,
  `eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_lib_cache.cpp`.
- This machine: `~/Library/Preferences/kicad/10.0/{sym-lib-table, fp-lib-table,
  kicad_common.json}`; `/Applications/KiCad/KiCad.app/Contents/SharedSupport/symbols/`.
- `/Users/james/code/eurorack/quad-vca-mixer/` — `.gitmodules`, `hardware/shared/`,
  and the per-board `sym-lib-table` / `fp-lib-table` quoted in §5.
