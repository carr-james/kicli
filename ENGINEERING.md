# kicli Engineering Standards

Binding on all code, like the Constitution. Most rules are machine-enforced;
the gates below are part of "done" for every task. When a rule here conflicts
with making the code simpler, prefer simpler and say so in the commit message.

## Machine-enforced gates (run all of them; all must pass)

```
cargo fmt --check          # formatting is rustfmt's opinion, not yours
cargo clippy --all-targets --all-features -- -D warnings
cargo test                 # includes doctests
cargo doc --no-deps        # must build clean; missing_docs is denied
cargo deny check           # licence allowlist (see Dependencies) + advisories
```

Crate lints (in lib.rs):

```rust
#![deny(missing_docs)]
#![deny(unsafe_code)]        // this problem domain never needs unsafe
#![warn(clippy::pedantic)]   // allow specific pedantic lints only with a
                             // comment saying why
```

No `#[allow(...)]` without an adjacent comment justifying it. No `unwrap()` or
`expect()` outside tests; library code returns `Result` with typed errors
(`thiserror`), the CLI layer renders them (`anyhow` acceptable there only).
`panic!` is a bug.

## The gate and the lane branch

**Ruled by James at the M4 close, on BLOCKED 1** — the second reading, written
down, because two lanes had by then invented the same deviation independently
and a conflict re-decided by each implementer is not a rule.

**The gate governs every commit that reaches `main`. A lane branch may be
transiently red only under a written sanction stating its cause and its end
condition; the merge must be green.**

The sanction's form is fixed, and both clauses were paid for:

- **A sanction is stated by cause, never by test name.** "The only permitted
  failure is `agent_doc_covers_every_command`" went stale the moment the task
  added a CLI flag and `agent_doc_covers_every_verb_flag` failed for the
  identical cause. Write the cause: "any `agent_doc` failure naming an
  undocumented `kicli wire` verb or flag."
- **Every sanction carries its exit procedure.** When the ending condition is
  met — the blocking lane merges, the documentation lands — **say in the entry
  which commits fell either side of it.** A sanction that ends mid-lane, with
  nothing telling the lane to notice, leaves a branch whose commits were checked
  under two different regimes and no record of where the line was.

The conflict this resolves was real, not pedantic: `AGENT.md` belongs to one
lane at a time, and `agent_doc_covers_every_command` fails the moment a verb
exists and is undocumented, so under the other reading the lane implementing the
verb could not make a single commit. The other reading is not absurd — it
forbids something else instead, namely documentation that describes a verb the
binary does not have — and that is why this needed a ruling rather than a
judgement.

## Rules earn their place

A rule that matters wants an executable twin: a gate, a lint, or a
workspace-reading test that fails when the rule is broken. Prose alone decays.
When a new rule is proposed, name its enforcement — or the incident that
demonstrated the need. A rule with neither does not go in. The same test
applies to growth of this document and the Constitution: the binding set stays
small enough to read in full at the start of every session, and that property
outranks completeness.

## Structure (SOLID, translated)

- **Single responsibility** holds as-is: one module = one concern. The
  workspace is four crates: `kicli-sexpr` (tokens/tree/prettify — minimal
  dependencies, no knowledge of schematics), `kicli` (modules: `model`,
  `geometry`, `connectivity`, `route`, `view`, `lint`, `render`, `libraries`,
  `kicad`, `pcb`, `cli`), `kicli-probe` (the test instruments: probe drawings and oracle
  readers), and `xtask`. `cli` depends on everything; nothing depends on `cli`;
  `kicli-sexpr` depends on nothing of ours. Crate boundaries enforce the
  dependency direction — do not merge them for convenience.
- **One edge is a cycle, and it is dev-only.** `kicli-probe` depends on `kicli`,
  because a probe measures what the extractor reads; `kicli` dev-depends on
  `kicli-probe`, because its tests are what probe. Cargo resolves a dev-only
  cycle. The edge back must stay under `[dev-dependencies]`, or a test
  instrument ships inside the binary — `probe_crate_is_dev_only` is the
  enforcement, because the licence gate cannot see a dev-dependency.
- **Two of those modules exist because the spec needs them, and the list above
  is not closed.** `view` owns the compact text and JSON representations, which
  are a separate concern from `render`: views are the truth an agent acts on,
  renders are passive pictures. `kicad` owns every invocation of an external
  KiCad binary — discovery, the version check, the process seam and the
  exit-code translation — because `lint`, `render` and `cli` all need it and
  none of them may depend on another. A later milestone may add a module the
  same way: state the concern, keep the dependency direction, and amend this
  list in the same change. `route` was added that way: it turns two terminals
  and a sheet's geometry into an ordered list of grid points and the cost of
  reaching them, and it knows nothing of files, the CLI or `kicad-cli`, so the
  search is as cheap to test as arithmetic.
- **Interfaces → traits.** Depend on traits or generics at seams that need
  substitution (e.g. the process-runner behind kicad-cli invocation, so tests
  can fake it). Do NOT introduce a trait with a single implementation and no
  test need — that is Java reflex, not design.
- **No inheritance exists.** Model alternatives as `enum`s (closed sets, get
  exhaustive `match` checking — prefer this) or trait objects (open sets —
  rare here).
- **Open/closed, Rust form:** adding a new lint rule or a new schematic object
  type must not require editing unrelated modules. Exhaustive matches on core
  enums are GOOD — the compiler then lists every site a new variant must
  handle.
- **Dependency inversion:** `lint` and `geometry` know nothing about the CLI,
  files on disk, or kicad-cli. Pure functions over the model wherever
  possible — they are the easiest code to test and the hardest to break.

## DRY, with the Rust caveat

Extract shared logic when it is the SAME concept, not merely similar text.
A little duplication is cheaper than the wrong abstraction. Three strikes
before abstracting is a fine heuristic. Never macro-away duplication that a
plain function can remove.

## Testing pyramid

- **Unit tests** (most): `#[cfg(test)] mod tests` beside the code. Pure
  functions in `geometry`/`lint`/`connectivity` should make these trivial.
- **Property tests** (the load-bearing layer for THIS project): `proptest` for
  round-trip laws, formatter equivalence, router determinism. Constitution §1
  lives here.
- **Golden-file tests**: fixtures in `tests/fixtures/` (purpose-built, per
  Constitution; KiCad's corpus fetched by xtask stays in target/, now to keep
  the repository small rather than for licence reasons). Assert exact output;
  update goldens only in a dedicated commit that says why. Fixture expectations
  are verified against KiCad (oracle records), never hand-asserted — a fixture
  written from the same assumption as the code tests nothing.
- **Integration tests** (few): `tests/` running the compiled binary end-to-end
  (`assert_cmd`), including the kicad-cli paths, skipped gracefully when
  kicad-cli is absent.
- **Doctests**: every public API example in rustdoc compiles and runs — free
  integration of docs and tests.
- **Worked examples in agent-facing documentation are measured output.** Every
  example block a change touches is regenerated from a real run of the built
  binary, never hand-edited into agreement with what the code is believed to
  print. This is the golden-file rule applied to prose, and for the same reason:
  an example written from the same assumption as the code demonstrates nothing.
  Provenance: the first dogfood run's defect 3, where `AGENT.md` showed a wire
  record in a format the tool had stopped writing and the reader misparsed it.

Write the test before the implementation for each task. A bug fix starts with
a failing test reproducing it.

## Controls before conclusions

A probe or experiment that tests external behaviour (KiCad, kicad-cli, any
oracle) is itself an instrument, and instruments fail. Before concluding
anything from a probe that did NOT produce the expected behaviour, run a
control: a variant that MUST produce it, built on ground already known good.
If the control also fails, the harness is broken — fix the instrument before
re-testing the hypothesis. Record the control alongside the probe; a negative
result without a passing control is not evidence and does not merge into a
research note as a finding.

## Self-documenting code

- Names carry the meaning; comments carry the WHY (invariants, KiCad quirks
  with source links — e.g. the ERC JSON canary — and justification for any
  `#[allow]`).
- Every public item has rustdoc (enforced). Module-level `//!` docs state the
  module's single responsibility in one paragraph.
- Prefer newtypes over primitives for domain values: `Iu(i32)` not bare `i32`,
  `SheetPath`, `LibId`, `NetId`. Illegal states unrepresentable beats runtime
  validation.
- Functions small enough to read without scrolling; clippy's complexity lints
  are the backstop.

## Code is plan-free

Code, tests, fixtures, and rustdoc never reference project-management
artefacts: no milestone numbers, task IDs, spec/constitution section numbers,
phase names, or "TODO(M4)". Names describe behaviour (`roundtrip_byte_identity`,
not `t7_roundtrip`); comments state the invariant or reason in full sentences
rather than citing an internal document ("ERC JSON coordinates are 100× too
small because KiCad builds the units provider with the PCB scale — see
erc_report.cpp:161", not "see SUMMARY.md finding 5"). External references
(KiCad source lines, upstream issues, format documentation) are encouraged;
internal plan references are forbidden. Commit messages are the ONE place task
IDs belong — they are project history, not code.

Deferred work in code is either a tracked issue referenced by URL/number or it
is deleted; bare TODOs do not merge.

## Prose: Simplified Technical English

All rustdoc, error messages, CLI help, and markdown: STE rules. Short
sentences (≤ 20 words). One instruction per sentence. Active voice.
Imperative mood for instructions. One term per concept — the spec's
vocabulary (symbol, field, instance, sheet path, IU) is the only vocabulary;
never introduce synonyms.

## Commits

One task per commit, message `area: what changed` with the task ID, body says
why when it is not obvious. The gates above pass at every commit, not just at
milestone end.

## Dependencies

Adding a crate requires: licence on the allowlist (cargo-deny enforces),
justification in the commit message, and preference for boring/std-adjacent
choices. The dependency budget is small on purpose — this tool must build
from source trivially for years.

**The licence allowlist governs the shipped artefact**: normal and build
dependencies, which must be GPL-3-compatible. Permissive licences (MIT,
Apache-2.0, BSD, ISC, Zlib) qualify, and so do MPL-2.0, LGPL and GPL itself,
because GPL-3 absorbs them. AGPL does not qualify and Constitution §9 excludes
it. Dev-dependencies are exempt, because a test helper is not distributed and
its licence attaches to nothing we ship. This is also what cargo-deny does:
verified on this workspace, a crate whose licence is off the allowlist is
rejected as a normal dependency and passes as a dev-dependency.

**Advisories cover everything**, dev-dependencies included — also verified: a
crate with an open RUSTSEC advisory fails `cargo deny check advisories` when
added as a dev-dependency. A known-vulnerable test helper is still a problem on
a developer's machine.
