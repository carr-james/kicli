# R2 — Rust S-expression strategy

Status: **bespoke parser confirmed**, and the design is simpler than expected
because of one empirically established fact:

> For every file KiCad 10 writes, the byte layout is a pure function of the
> token stream. Whitespace carries no information, and `prettify(flatten(bytes))
> == bytes` for 100 % of the test corpus (§2).

That means kicli does **not** need a whitespace-preserving concrete syntax tree
in the rust-analyzer/`rowan` sense. It needs a *token-preserving* tree plus a
faithful port of KiCad's pretty-printer.

Prerequisite reading: [`sch-format.md`](sch-format.md) §2 (lexical layer).

---

## ⚠ Contradictions and cautions for `spec/SPEC.md`

1. **SPEC §15 M1 says "lossless CST".** Refine to: *token-preserving syntax tree
   + exact re-emitter*, with raw-span fallback for non-canonical input (§4.4).
   The distinction matters: a full CST (whitespace as tree nodes) is more code,
   more memory, and buys nothing for KiCad-authored files.

2. **Fixtures cannot be KiCad's demo files.** SPEC D18 / Constitution §11 says
   fixtures are purpose-built and in-repo. KiCad's `demos/` and `qa/data/` are
   GPL-licensed project files; they are excellent as an *external* corpus but
   must not be vendored into an MIT/Apache repo. Recommendation: a
   `cargo xtask corpus` that clones KiCad at a pinned tag into `target/` and
   runs the round-trip property over it, kept out of the default `cargo test`
   (Q3).

3. **SPEC's round-trip gate needs the comment and non-canonical cases spelled
   out.** KiCad's lexer accepts `#`-prefixed line comments
   (`common/dsnlexer.cpp:575-582`) and silently drops them on save; the
   prettifier would mangle them. Files can also arrive non-canonical (all 36
   library tables in KiCad's own demos are, §2.3). Constitution §1 says kicli
   must refuse to modify what it cannot round-trip — §4.4 proposes the concrete
   policy, but the *choice* is James's (Q1).

---

## 1. Requirements, derived

From R1, an emitter must reproduce, exactly:

| # | Requirement | Source |
|---|---|---|
| R-1 | Token text preserved verbatim (no number re-formatting of untouched values) | `sch-format.md` §2.3 |
| R-2 | Layout produced by `KICAD_FORMAT::Prettify`, three modes | `common/io/kicad/kicad_io_utils.cpp:97` |
| R-3 | String escaping: exactly `\n \r \\ \"` | `common/richio.cpp:468` |
| R-4 | Numbers written as `int32` IU → fixed-point mm, trailing zeros stripped | `common/eda_units.cpp:194` |
| R-5 | Booleans as `yes`/`no`; legacy bare-token form accepted on read | `sch-format.md` §2.4 |
| R-6 | Trailing newline, exactly one | `kicad_io_utils.cpp:336` |
| R-7 | UTF-8 throughout, no BOM | `Quotew` → `utf8_str()`, `richio.cpp:507` |

---

## 2. The central experiment: layout is derivable

### 2.1 Recipe (R2-A)

1. Build the canonical corpus (recipe C in `sch-format.md` §0.2) — 115
   `.kicad_sch` files at `(version 20260306)`.
2. Port `KICAD_FORMAT::Prettify` (NORMAL mode) literally — ~90 lines.
3. `flatten(src)`: delete every whitespace run outside quotes, replacing it with
   a single space. This destroys all layout information while preserving tokens.
4. Assert `prettify(flatten(src)) == src`.

### 2.2 Results

| Corpus | Files | `prettify(flatten(x)) == x` |
|---|---|---|
| KiCad 10.0.5 demos, `.kicad_sch` (canonicalised, recipe C) | 115 | **115 / 115** |
| KiCad 10.0.5 demos, `.kicad_pcb` (sample) | 6 | **6 / 6** |
| Shipped `Device.kicad_sym` (KiCad 10.0.5 install) | 1 | **1 / 1** |
| Demos' `sym-lib-table` / `fp-lib-table` (LIBRARY_TABLE mode) | 36 | 1 / 36 — see §2.3 |

The Python reference port used for this is ~90 lines and is the specification
kicli's Rust emitter must match; it should be committed alongside the Rust
implementation as a cross-check oracle.

Two corollaries:

- **Byte-identical output is achievable without storing whitespace.** Keep the
  list structure and each atom's exact source text; re-emit through the
  prettifier.
- **`.kicad_pcb` uses the same machinery**, so one emitter serves the PCB phase
  too (SPEC §11), should kicli ever write board files directly.

### 2.3 Where it fails, and why that is informative

35 of 36 library tables in KiCad's own demos do **not** round-trip through the
prettifier, because they were written by KiCad ≤ 7 and never re-saved:

```
(sym_lib_table
  (version 7)
  (lib (name "CM5IO")(type "KiCad")(uri "${KIPRJMOD}/CM5IO.kicad_sym")(options "")(descr ""))
)
```

Two-space indentation, and no space between sibling lists. KiCad 10 writes tabs
and `(lib (name "…") (type "…") …)`. So the moment kicli edits a project's
`sym-lib-table` (R4 vendoring!) the whole file reformats. That is what KiCad
itself does, but it should be an explicit, documented consequence rather than a
surprise in a git diff.

**This is the general shape of the problem: files not written by KiCad 8+ are
not canonical.** Same applies to a hand-edited `.kicad_sch`.

### 2.4 The three prettifier modes

| Mode | Used for | Extra rule |
|---|---|---|
| `NORMAL` | `.kicad_sch`, `.kicad_pcb`, `.kicad_sym` | — |
| `COMPACT_TEXT_PROPERTIES` | clipboard, local history, **and all normal saves when the advanced config `CompactSave` is true** | keeps `font stroke fill teardrop offset rotate scale` lists on one line |
| `LIBRARY_TABLE` | `sym-lib-table`, `fp-lib-table` | one `lib` row per line |

`CompactSave` defaults to `false` (`common/advanced_config.cpp:276`) but is a
user-settable advanced key (`common/advanced_config.cpp:77`, applied in
`common/richio.cpp:575`). **kicli must implement all three modes** and must
choose its output mode by *detecting the input file's mode*, not by assuming
NORMAL — otherwise editing one field in a `CompactSave` user's schematic
reformats the entire file.

Detection is cheap: run the file through both prettifiers and see which one is a
fixed point.

---

## 3. Crate survey

**Caveat on method:** this environment has no Rust toolchain (`cargo` absent),
so no candidate was executed. The table is from crates.io metadata (fetched
2026-08-12 via the crates.io API) and published documentation. Verdicts rest on
documented data models, which is sufficient to rule crates in or out on the
*token-preservation* criterion, but any crate short-listed for real use should
be exercised against recipe R2-A before adoption.

### 3.1 General s-expression crates

| Crate | Latest | Licence | Data model | Verdict |
|---|---|---|---|---|
| `lexpr` 0.2.7 | 2023-03 | MIT OR Apache-2.0 | Lisp `Value`; numbers → `i64`/`u64`/`f64` | ✗ numeric atoms lose exact source text; Lisp reader semantics (dotted pairs, `#t`, chars) don't match KiCad's dialect |
| `sexp` 1.1.4 | 2016-10 | MIT | `Atom::{S,I,F}` | ✗ same numeric-lossiness; unmaintained |
| `sise` 0.8.0 | 2022-03 | MIT OR Apache-2.0 | `Node::{Atom(String), List(Vec)}` — **atoms stay strings** | ~ closest general fit; still no layout, and SISE's own quoting rules ≠ KiCad's |
| `rsexp` 0.2.3 | 2022-01 | MIT/Apache | OCaml-style sexp, `Atom(Vec<u8>)` | ~ byte-atom model is fine; OCaml escaping conventions differ |
| `anysexpr` 0.4.0 | 2023-05 | MIT OR Apache-2.0 | multi-dialect reader/formatter, keeps comments | ~ interesting for comment handling; dialect mismatch, low adoption |
| `ssexp` 0.6.0 | 2026-06 | MIT OR Apache-2.0 | general parser | ~ actively maintained; still a general dialect |
| `symbolic_expressions` 5.0.3 | 2017-10 | MIT | `Sexp::{String,List,Empty}` | ~ written *for* kicad-parse-gen; atoms are strings; dormant 9 years |

### 3.2 KiCad-specific crates

| Crate | Latest | Licence | Notes |
|---|---|---|---|
| `kicad_parse_gen` 7.0.2 | 2018-01 | MIT | Targets the KiCad 4/5 era; predates the s-expression schematic format entirely (v6+). Dead end for KiCad 10, but its sibling `symbolic_expressions` shows the "atoms as strings" approach. |
| `serde_kicad_sexpr` 0.1.0 | 2022-01 | Apache-2.0 OR LGPL-3.0 | "KiCAD v6 S-Expression Format". Serde-based ⇒ strongly typed AST ⇒ unknown tokens are lost. Dormant. |
| `via-kicad-sexp` 0.1.1 | 2026-07 | **MPL-2.0** | Newest KiCad sexpr crate ("parser and renderer for via"). MPL-2.0 is file-level copyleft; Constitution §9 restricts dependencies to MIT/Apache/BSD-compatible, so this needs an explicit exception to use. |
| `kicad-ipc-rs` 0.5.1 | 2026-05 | (see R5) | IPC API client, not a file parser. Relevant to R5. |
| `kicad-api-rs` 0.1.0 | 2025-06 | (see R5) | Ditto. |

### 3.3 Verdict

**Refuted: no existing crate is usable as the core.** The blocking reasons, in
order:

1. **Serde/AST crates lose unknown tokens.** KiCad adds tokens every point
   release (`sch-format.md` §5.7 lists 13 in 10.0.x alone). A typed AST that
   drops what it doesn't know violates Constitution §1 on the first KiCad
   update. Any design where "parse → struct → serialise" is the only path is
   wrong for this problem.
2. **Numeric atoms must not be interpreted at parse time.** Crates that parse
   `41.91` into `f64` cannot guarantee it comes back as `41.91`.
3. **No crate implements KiCad's prettifier**, which is where byte-identity
   actually comes from.

**Confirmed: bespoke parser + emitter.** It is small — the grammar is
`atom | "(" atom* ")"` with one string form — and the value is entirely in the
fidelity guarantees, which is exactly the part a general crate cannot provide.

Dependencies worth taking anyway: `proptest` (property tests), `arbitrary`
(fuzz), `memchr` (scanning), and `logos` **only if** hand-rolling the lexer
proves slow (it will not — the grammar has five token classes).

---

## 4. Design

### 4.1 Lexer

Five token classes; single pass over `&[u8]`, UTF-8 validated once up front.

```
LParen        "("
RParen        ")"
QuotedString  '"' ( '\\' . | [^"\\] )* '"'
BareAtom      [^ \t\r\n()"]+            (numbers, keywords, symbols — undifferentiated)
Comment       '#' … EOL                 (only when '#' is the first non-blank on the line)
```

Notes:

- `BareAtom` deliberately does **not** classify numbers. Classification is a
  *query* on the node (`as_i32_iu()`, `as_f64()`), never a parse-time
  transformation.
- The comment rule mirrors `common/dsnlexer.cpp:575-582` (first non-blank char
  of a line). Comments are retained in the tree as trivia nodes so kicli can
  detect them (§4.4), even though KiCad discards them.
- No escape processing at lex time: the quoted string's raw span is kept, and
  unescaping is a query.

### 4.2 Tree

```rust
pub struct Doc {
    src:   Arc<str>,      // whole original file, kept for span slicing
    nodes: Vec<Node>,     // arena; index-based, no Rc cycles
    root:  NodeId,
    mode:  FormatMode,    // detected: Normal | CompactTextProperties | LibraryTable
    canonical: bool,      // prettify(src) == src  → byte-identity achievable
}

pub enum Node {
    List { head: Option<NodeId>, children: Vec<NodeId>, span: Span },
    Atom { kind: AtomKind, span: Span, edited: Option<Box<str>> },
    Comment { span: Span },
}

pub enum AtomKind { Bare, Quoted }
```

Key properties:

- **`span` into the original source is the source of truth for unedited
  atoms.** `edited` is `None` until kicli changes a value; emission prefers
  `edited`, else `&src[span]`. This is what makes "kicli never rewrites a token
  it did not modify" (`sch-format.md` §5.4) structurally true rather than a
  convention.
- **Arena + `NodeId`** keeps mutation cheap and lets handles (UUID → node) be
  plain indices, which the addressing model (SPEC D13) needs.
- The `head` is the first child when it is a bare atom — memoised so
  `list.head_is("symbol")` is O(1). Every KiCad list is head-tagged.

### 4.3 Emitter

Two-stage, mirroring KiCad exactly:

```
stage 1  walk tree → flat token stream, single space between tokens
stage 2  prettify(mode)  → final bytes
```

Stage 2 is a literal port of `kicad_io_utils.cpp:97-339`, including the three
mode flags, `xySpecialCaseColumnLimit = 99`, `consecutiveTokenWrapThreshold =
72`, the backslash-parity quote tracking, and the final `'\n'`. §2.2 shows this
is sufficient.

Value writers (only used for *edited* atoms):

```rust
fn fmt_iu(v: i32) -> String     // int IU → mm, fixed 4dp, strip trailing zeros then '.'
fn fmt_angle(deg: f64) -> String// {:.10g} equivalent
fn quote(s: &str) -> String     // escape only \n \r \\ \"
```

`fmt_iu` avoids float formatting entirely and is provably identical to KiCad's
`{:.10g}` output for all `int32` inputs (`sch-format.md` §2.3).

### 4.4 Policy for non-canonical and commented files

Proposed (needs James's sign-off, Q1):

| Input | kicli behaviour |
|---|---|
| Canonical (`prettify(src) == src` in some mode) | Normal path. Output byte-identical except edited subtrees. |
| Non-canonical, **no comments** | Edit and emit canonically, with a warning in the structured output: `"reformatted": true, "reason": "input was not in KiCad canonical form"`. Semantics preserved; the diff is large but is exactly what KiCad's next save would do anyway. |
| Contains `#` comments | Refuse to write by default (Constitution §1); `--allow-comment-loss` to proceed, which drops them exactly as KiCad would. |
| Unknown newer `(version …)` | Refuse to write (`sch-format.md` Q3). |

`canonical` is computed once at load; it costs one prettify pass (~ms for a 4 MB
file).

---

## 5. Round-trip property-test harness

Constitution §11 requires an executable check per task. Four layers:

### L1 — Corpus round-trip (the workhorse)

```
for f in corpus:
    doc = parse(f)
    assert emit(doc) == bytes(f)          # P1: byte identity, no edits
```

Corpora, in increasing severity:

| Corpus | Size | Provenance |
|---|---|---|
| `fixtures/` purpose-built | tens | in-repo, hand-authored + KiCad-canonicalised (Constitution §11, SPEC D18) |
| KiCad demos at pinned tag | 115 `.kicad_sch`, 36 lib tables, ~10 `.kicad_pcb` | external, fetched by `cargo xtask corpus` |
| KiCad `qa/data` at pinned tag | 295 `.kicad_sch` incl. deliberately weird ones | external, same |

The `qa/data` set is especially valuable: it contains the files KiCad's own
regression tests use, i.e. the pathological ones. Note that many are older
format versions — P1 applies only to files that are already canonical for their
version; for the rest, assert P2 only.

### L2 — Semantic round-trip

```
assert parse(emit(parse(f))) ≡ parse(f)     # P2: structural equality of the tree
                                            #     modulo whitespace and atom spans
```

This is the property that must hold for *every* input, including non-canonical
ones, and is the honest floor Constitution §1 describes.

### L3 — Mutation locality

```
doc = parse(f); doc.edit(one_atom);
diff = line_diff(bytes(f), emit(doc))
assert diff.changed_lines <= K              # P3: an edit touches only its own region
```

P3 is what actually protects the user's git history, and it is the property most
likely to regress silently. `K` should be small (≤ 3 for a field-value change)
and asserted per mutation command.

### L4 — Generative

`proptest` strategy that builds random *valid* schematic trees (random nesting,
random atoms including nasty strings: embedded quotes, backslashes, newlines,
UTF-8, empty strings, `~`, `#`-leading), then:

```
assert parse(emit(t)) ≡ t
assert prettify(prettify(x)) == prettify(x)   # idempotence
```

Plus a fuzz target (`cargo-fuzz` + `arbitrary`) asserting *no panic* on
arbitrary bytes — the parser will be fed agent-generated files.

### L5 — Oracle cross-check (optional but cheap)

When `kicad-cli` is on `PATH`, run `kicad-cli sch upgrade --force` on a copy of
kicli's output and assert the result is unchanged (i.e. kicli's output is
already what KiCad would write). This catches divergence from KiCad's evolving
canonical form without kicli having to guess. Gate it behind
`KICLI_TEST_KICAD_CLI=1` so the default test run stays hermetic.

**Verified during this research** that `kicad-cli sch upgrade --force` is
idempotent (running it twice produces byte-identical files), which is what makes
L5 a valid oracle.

---

## 6. Performance notes

Largest demo sheet is ~4 MB (`CM5.kicad_sch` and friends: 19 279 wires across
the corpus). A single-pass byte lexer plus arena tree is comfortably in the
low-milliseconds range; the prettify pass is a second linear scan. No
performance-driven design compromises are needed. Memory: the arena plus the
original source is ~3-4× file size, fine at these sizes. Avoid `String` per atom
(spans only) and this stays true for the largest realistic project.

---

## 7. Open questions for James

- **Q1 — Non-canonical and commented input policy.** Confirm the §4.4 table,
  especially: is silent reformatting of a non-canonical file acceptable (with a
  flag in the JSON output), or must kicli refuse?

- **Q2 — Emit mode selection.** Confirm: detect the input's prettifier mode and
  preserve it (so `CompactSave` users' files stay compact), rather than always
  writing NORMAL.

- **Q3 — External corpus.** Approve `cargo xtask corpus` fetching KiCad's demos
  and `qa/data` at a pinned tag into `target/` for round-trip testing, keeping
  GPL files out of the repo, with the in-repo `fixtures/` remaining the gate for
  the default `cargo test`.

- **Q4 — MPL-2.0.** `via-kicad-sexp` is the only actively maintained KiCad
  s-expression crate and is MPL-2.0. We are not proposing to use it, but confirm
  the general rule: MPL-2.0 dependencies are out under Constitution §9.

---

## 8. Sources

- KiCad 10.0.5 source, tag `10.0.5`: `common/io/kicad/kicad_io_utils.cpp`,
  `common/richio.cpp`, `common/dsnlexer.cpp`, `common/advanced_config.cpp`,
  `common/eda_units.cpp`.
- crates.io API, fetched 2026-08-12: `https://crates.io/api/v1/crates/<name>`
  and `?q=kicad`, `?q=s-expression`.
- Crate repositories linked in the tables above.
- Experiment R2-A: prettifier port + corpus comparison, this document §2.
