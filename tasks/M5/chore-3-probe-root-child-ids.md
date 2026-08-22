# Chore — `ROOT` and `CHILD` never received the per-series numbering

**Provenance: PROPOSED 16, found by D1 (the fixture-handle chore), promoted by
James's ruling at the M4 close.**

## The gap

M4's C5 gave probe objects `{series:02x}{n:06x}` identifiers.
`crates/kicli-probe/src/drawing.rs:15,18` hold `ROOT` and `CHILD` as **fixed
constants** that never received it. So a probe drawing **with a child sheet**
gives two objects the handle `00000000` — a collision inside a single drawing,
which is precisely what C5 set out to remove.

Small, mechanical, and **it will bite the first verb that addresses a child
sheet by handle**.

## Completion check

`cargo test --test fixture_handles` and the probe crate's own tests, plus a
check that no two objects of one probe drawing share a handle **when the drawing
has a child sheet** — the case that currently has no coverage.
