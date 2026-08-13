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
cargo deny check           # licence allowlist (MIT/Apache-2.0/BSD; no MPL, no GPL) + advisories
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

## Structure (SOLID, translated)

- **Single responsibility** holds as-is: one module = one concern. The crate
  splits along the spec's seams: `sexpr` (tokens/tree/prettify), `model`
  (typed schematic objects), `geometry`, `connectivity`, `lint`, `render`,
  `libraries`, `pcb`, `cli`. `cli` depends on everything; nothing depends on
  `cli`.
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
  Constitution; GPL corpus fetched by xtask stays in target/). Assert exact
  output; update goldens only in a dedicated commit that says why.
- **Integration tests** (few): `tests/` running the compiled binary end-to-end
  (`assert_cmd`), including the kicad-cli paths, skipped gracefully when
  kicad-cli is absent.
- **Doctests**: every public API example in rustdoc compiles and runs — free
  integration of docs and tests.

Write the test before the implementation for each task. A bug fix starts with
a failing test reproducing it.

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
