# research/snapshots

Dated retrieval snapshots of **external sources that cannot be relied on to stay
put** — living web pages, Google Docs, wikis, blog posts, anything version-dated
rather than immutable. A source with a DOI, an ISBN or an archival permalink does
not need a snapshot; a page that can be edited under us does.

The point is that a citation must not drift. When this repository cites such a
source, the citation resolves to the file in this directory, not to the URL. A
reader checking a rule's provenance reads what we actually read, on the date we
read it, whatever the URL says today.

## The rules

1. **The reproduced body of a snapshot is never edited after it is taken.** Not
   to fix a typo, not to tidy formatting, not to correct the source's own errors.
   An edited snapshot is not a snapshot — it is a summary wearing a snapshot's
   filename, and it can no longer settle the question it exists to settle.
   *The one exception is narrow and points the same way:* an **integrity
   constant in the header** — a digest, a byte count, a stated normalisation —
   that is **demonstrably false** is corrected in place, with a note recording
   the wrong value, the right one, and how it was found. That is not editing the
   evidence; it is repairing the instrument that certifies it. A false digest
   does not sit there harmlessly, it *fires*, and it accuses the artefact of
   exactly the tampering the directory exists to rule out. The body's own
   immutability is what makes the repair safe: the correction must be
   accompanied by the `diff` showing the body did not move.
2. **A new retrieval is a new dated file, never an edit to an old one.** If the
   source changes, or a later retrieval is needed for any reason, add
   `<source>-<YYYY-MM-DD>.md` alongside the existing one and leave the old file
   exactly as it is. The pair of files *is* the evidence of drift, and that
   evidence is worth more than either file alone.
3. **Every snapshot carries a provenance header** stating: full title, author,
   the source's own version date if it has one, the source URL, the retrieval
   date, the exact method and HTTP status, the byte count, and a digest.
4. **Anything that is not the cited author's own words is fenced and labelled as
   such** — reader comments, editorial notes, third-party quotations. Mistaking a
   comment for the author's text is the specific failure these files prevent.
5. **Holes are stated in the file.** If part of the source was unreachable,
   rendered badly, or was lost by the export format, the snapshot says which part
   and why. A snapshot with a declared hole is honest; one with a silent hole is
   worse than no snapshot at all.
6. **Corrections to the CONTENT go outside the file.** If a snapshot is later
   found to misrepresent its source — a mis-transcription, a missing passage, a
   comment mistaken for the author's text — the correction is recorded in the
   citing document or the relevant task entry, never by touching the snapshot.
   Only the header-metadata case in rule 1 is repaired in place.
7. **A stated digest must be reproducible by a procedure the file itself
   gives.** State the commands, not just the constant. A digest a reader cannot
   regenerate is decoration; and an unrunnable procedure is how a stale constant
   survives review — see the header-correction note in the Greenberg snapshot,
   which is that failure caught at tick review.

## Naming

`<short-source-slug>-<YYYY-MM-DD>.md`, where the date is the **retrieval** date,
not the source's own version date. Where a byte-exact machine artefact is worth
keeping too, it sits beside the snapshot as
`<short-source-slug>-<YYYY-MM-DD>.source.<ext>` and its digest is recorded in the
snapshot's header.

## Contents

| File | Source | Retrieved |
|---|---|---|
| `greenberg-schematic-checklist-2026-08-22.md` | Andrew Greenberg, "Checklist for Schematics v2026-02-15" (Google Doc) | 2026-08-22 |
| `greenberg-schematic-checklist-2026-08-22.source.txt` | byte-exact plain-text export of the above (8,339 bytes), vendored so the snapshot's digests can be re-derived without a network fetch | 2026-08-22 |
