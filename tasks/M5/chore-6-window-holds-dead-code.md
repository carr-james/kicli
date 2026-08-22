# Chore — `Window::holds` is dead code

**Provenance: James's ruling at the M4 close, on the mutation run's triage.**
Verbatim: *"`Window::holds` is dead code — remove as a chore."*

## The evidence

From the M4-close `cargo-mutants` run, group M-7 in `mutation-survivors.md`:

```
crates/kicli/src/route/window.rs:105:9: replace Window::holds -> bool with false
crates/kicli/src/route/window.rs:105:9: replace Window::holds -> bool with true
```

**`Window::holds` has no callers at all** — which is why both `-> true` and
`-> false` survive. `clippy` does not flag it because the method is `pub`.

**A mutant surviving in both directions is the signature of dead code**, and
that lesson is worth more than the method is as code.

## Scope, exactly

Remove `Window::holds`. **The third mutant of group M-7 — `replace || with && in
Window::cell` at `window.rs:83:34` — is NOT in this chore**: it is an unpinned
guard on live code, and it stays filed in `mutation-survivors.md`. A chore that
quietly absorbed it would be the survivor-count-driven-to-zero failure the
mutation-run skill exists to prevent.

## Completion check

`cargo xtask check` — `#![deny(missing_docs)]` and the clippy gate are what
prove the removal is clean. If any caller turns out to exist, **stop and
report**: the ruling rests on there being none, and a chore-runner does not
overturn a ruling's premise.
