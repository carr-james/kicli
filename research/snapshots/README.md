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

1. **A snapshot is never edited after it is taken.** Not to fix a typo, not to
   tidy formatting, not to correct the source's own errors. An edited snapshot is
   not a snapshot — it is a summary wearing a snapshot's filename, and it can no
   longer settle the question it exists to settle.
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
6. **Corrections go outside the file.** If a snapshot is later found to
   misrepresent its source, the correction is recorded in the citing document or
   the relevant task entry — never by touching the snapshot.

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
| `greenberg-schematic-checklist-2026-08-22.source.txt` | byte-exact plain-text export of the above (8,339 bytes) | 2026-08-22 |
