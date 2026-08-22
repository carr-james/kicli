# SNAPSHOT — Checklist for Schematics v2026-02-15 (Andrew Greenberg)

**This file is a dated retrieval snapshot of an external source. It is never
edited after the retrieval date in its name.** It exists because the source is a
Google Doc: a live, version-dated, editable page, not an archival artefact. It
can change under us without notice and carries no immutable identifier. Any
citation of Greenberg in this repository resolves to *this file*, not to the
URL, so that the citation cannot drift.

## Provenance

| Field | Value |
|---|---|
| Title | Checklist for Schematics v2026-02-15 |
| Author | Andrew Greenberg — <http://github.com/andrewgreenberg> |
| Document's own version date | **2026-02-15** (stated in the title; the document carries no other version marker) |
| Source URL | <https://docs.google.com/document/d/1gCPILcrdGZJjRzIDSL-b3ezVReeK5S-7raeub1RohyE/> |
| Retrieved | **2026-08-22** by lane `lane-snap` |
| Retrieved by | `curl -sSL` (no auth, no cookies, public anonymous fetch) |

### Retrieval 1 — plain text (the primary artefact)

| Field | Value |
|---|---|
| URL | `https://docs.google.com/document/d/1gCPILcrdGZJjRzIDSL-b3ezVReeK5S-7raeub1RohyE/export?format=txt` |
| Method | `curl -sS -L` |
| HTTP status | **200** (after redirect to `doc-0g-64-docstext.googleusercontent.com`) |
| Bytes | **8,339** |
| MD5 | `e04ed36a3066374a7e02e4224adfc109` |
| Encoding | UTF-8 **with BOM**, **CRLF** line terminators |
| Vendored byte-for-byte as | `greenberg-schematic-checklist-2026-08-22.source.txt` |

### Retrieval 2 — HTML (supplementary, to recover what the text export drops)

| Field | Value |
|---|---|
| URL | `https://docs.google.com/document/d/1gCPILcrdGZJjRzIDSL-b3ezVReeK5S-7raeub1RohyE/export?format=html` |
| Method | `curl -sS -L` |
| HTTP status | **200** |
| Bytes | **20,782** |
| MD5 | `2ebb2f516ca7092a79c1fb24ac99a4cb` |

The HTML export was taken **only** to recover hyperlink targets and character
emphasis, both of which the plain-text export discards. It contributes nothing
to the rule text. What it recovered is recorded in "Rendering losses", below,
and is marked there as reconstruction rather than as part of the verbatim body.

## Fidelity control against the previous retrieval

`lane-t5` retrieved this same document on 2026-08-22 and recorded: HTTP 200,
8,339 bytes as plain text. **The byte count and content match exactly** — this
retrieval is 8,339 bytes with the digest above. The document has not drifted
between the two retrievals.

`lane-t5` additionally recorded "**55 checklist items in 8 groups**". **That
count is wrong, and the discrepancy is in the counting, not in the document** —
the bytes are identical, so there is nothing for a drift to have changed. Counted
structurally from this retrieval:

- **9 group headings**, not 8.
- **47 checklist items**, not 55.
- 56 bullet lines in total (9 headings + 47 items).

The `55`/`8` pair is consistent with a single off-by-one: 56 bullets minus the
first heading is 55, and 9 headings minus the first is 8, i.e. the first group
heading ("Visual Design Best Practices") was absorbed into the document title and
then its subordinate bullets were counted as items. The group headings are
independently distinguishable from items in the HTML export, where **all nine are
bold and no item is bold except two** (see "Rendering losses"). Per-group counts:

| # | Group | Items |
|---|---|---|
| 1 | Visual Design Best Practices | 12 |
| 2 | Schematic Symbols | 3 |
| 3 | Part values | 5 |
| 4 | Circuit Gotchas | 8 |
| 5 | Design for Test | 4 |
| 6 | Design for Fail | 5 |
| 7 | Electrical Rule checks | 2 |
| 8 | Bill of Material (BOM) Integration | 3 |
| 9 | "You're Almost Done" checks | 5 |
| | **Total** | **47** |

## What is and is not the author's

The document ends with ten reader comments, anchored `[a]`–`[j]`. **These are
third-party comments, not Greenberg's rules.** They quote and link other authors'
material and in places disagree with the checklist. They are reproduced below
under their own heading, fenced separately, because confusing a reader comment
for the author's rule is precisely the failure this snapshot exists to prevent.

The anchors `[a]`–`[j]` also appear **inline in the body text** at the points
they annotate (e.g. `Checklist[a][b]`, `your user[h][i]`). They are part of the
retrieved bytes and are left in place unaltered; they are comment markers, not
the author's words.

## Rendering losses, stated

The plain-text export is lossy in three ways. Each is stated here rather than
silently repaired, and each recovery below comes from Retrieval 2.

1. **Hyperlink targets are dropped**, leaving bare link text. Two body links are
   affected. Recovered from the HTML export:
   - "…usually >= 2x working voltage, also see **this**…" — the word "this"
     linked to <https://github.com/CDFER/Ceramic-Capacitor-Derating>
   - "See also **Checklist for PCB Layout**" — linked to
     <https://docs.google.com/document/d/1rTR0l9Wx3Xt49cGbwR0zIWP6VN6aWhkTXeV7VV9wQNA/>
     (a *sibling* checklist for PCB layout, a separate document, **not**
     retrieved or snapshotted here)
2. **Character emphasis is dropped.** In the source, bold marks the nine group
   headings and **exactly two checklist items**, which are therefore the author's
   own emphasised rules:
   - "No unapproved errors OR warnings in the ERC."
   - "Your schematic is peer reviewed by at least one person not involved in the design."

   No other body text is bold or italic. In particular the words "Always" and
   "Bonus points" are **not** emphasised in the source, notwithstanding any
   downstream quotation that bolds them.
3. **Typographic quotes and one stray character are preserved as retrieved.** The
   source uses curly quotes throughout, contains an unbalanced closing quote in
   the functional-blocks item (`... switching power supply")`), and has doubled
   full stops on two items. These are the document's own; they are not
   transcription errors and have not been corrected.

No part of the document was unreachable and no part rendered badly beyond the
three losses above. There are no images, tables or drawings in the source.

---

# VERBATIM BODY — Andrew Greenberg's checklist

Reproduced byte-for-byte from Retrieval 1. Three display normalisations, and no
others: the UTF-8 BOM is stripped, CRLF line endings become LF, and a single
trailing newline is added (the source file ends without one). With those three
reversed, the text below hashes to `87d0784e22f4360e0654dfef71f6d42f`, identical
to the retrieval. **Nothing below is paraphrased, summarised, reordered or
corrected.**

```text
Checklist[a][b] for Schematics v2026-02-15
Andrew Greenberg - http://github.com/andrewgreenberg


* Visual Design Best Practices[c][d][e][f][g]
   * Power supplies use supply symbols (not wires) with useful names.
   * Positive supplies point up, ground and negative supplies point down. Always.
   * All important nets are descriptively named.
   * Net “stubs” (nets visually connected to only one pin) use an “off-sheet” or “Global” type of label with the correct In/Out/Bidirectional flag shape and cross reference info (sheet / location if possible).
   * Functional blocks are clearly labeled (plenty of whitespace around it, or maybe even a box).
   * Functional blocks have text that describes what they do and their requirements (e.g., Vbatt to 3.3 V @ 1 A switching power supply”).
   * There's a frame around the schematic.
   * It's clear where your power is coming from and what the power requirements are (V/I).
   * Data flow (inputs, outputs, requirements) are clear and labeled.
   * All connectors have text that describes where they go, and describes signals (voltage, current, names)..
   * Route wires at a consistent distance from each other and avoid crossing net wires as possible.
   * Groups of nets above about ≥ 4 nets collected into buses.
* Schematic Symbols
   * All symbols are schematic symbols, not packages (inputs on left, outputs on right, power on top and bottom).
   * Pins have correct electrical rule check (ERC) direction (inputs, outputs, passives, etc).
   * Components with symbolic shapes use those shapes (e.g, opamps are triangles).
* Part values
   * Capacitors have the appropriate voltage (usually ≥ 2x working voltage, also see this) and specify dielectric type if necessary.
   * Special case capacitors marked with power and tolerance.
   * Power dissipation checked on all resistors.
   * Special case resistors marked with power and tolerance.
   * Layout features that are circuit elements (e.g., copper inductor) are labeled in the schematic.
* Circuit Gotchas
   * MOSFETs oriented correctly WRT the body diode (!), with note if intentionally forward conducting.
   * Check IC part numbers reflect the correct package type.
   * Small, low ESR (e.g., ceramic) bypass capacitors on all IC supplies (check datasheet for values).
   * Check voltage inputs and outputs match across power domains (e.g., 5V to 3.3V).
   * Check that powered-off  domains are not phantom powered by their inputs from other circuits (including test circuits, like UARTs).
   * Check for UART TX/RX swaps (TX to RX, RX to TX) and name with source, e.g. MCU_TX.
   * Check for pull-up/down resistors on open collector/drain outputs (e.g.,  I2C lines).
   * Check for required pull-up/down resistors to set nets in a default state at power up.
* Design for Test
   * Place pads or connector for programming your ICs.
   * Place test points on critical signals, especially power and ground and reset lines.
   * Check the PCB side (front/back) for  SMT test points, or use through-hole test points. Double check the board side works for  your programmer / bed-of-nails tester / programming jig.
   * Add debugging hardware (e.g., LEDs, UART connectors, jumpers, scope probe points, etc).
* Design for Fail
   * Group components in separable modularly powered blocks and use zero ohm resistors or cuttable jumpers to disconnect (especially for switching power supplies!)
   * Unused pins (especially GPIO) should  go to usable test points. Consider adding some random  pull-up and pull-down resistors connected to a test point on the board, too.
   * Consider somehow encoding your PCB hardware revision in hardware (GPIO pullups, etc).
   * UART (serial port) TX/RX are always mixed up, triple-check them. Also consider cuttable jumpers here.
   * Consider over-voltage/ polarity input protection if you or your user[h][i] can screw this up.
* Electrical Rule checks
   * No unapproved errors OR warnings in the ERC.
   * All important excluded errors/warnings have a comment on why they’re approved.
* Bill of Material (BOM) Integration
   * Add “MFR” (Manufacturer) and “MPN” (Manufacturer’s Part Number) to all components as attributes.
   * Bonus points for adding a distributor, distributor part number, description, and datasheet link.
   * Add required cable or required accessory part numbers as text, or add as Do Not Place (DNP) components if you want them to show up in the BOM..
* “You’re Almost Done” checks
   * Your schematic is peer reviewed by at least one person not involved in the design.
   * Check that your specialized parts are in stock at a distributor.
   * Re-run ERC and double check your approved errors, looking for accidentally approved errors.
   * If this is a revision, record changes to the schematic in a table or in nearby documentation.
   * Update your schematic version and/or date [j].


See also Checklist for PCB Layout


Please comment on this checklist using Google Doc Comments!!

```

---

# READER COMMENTS `[a]`–`[j]` — NOT the author's rules

**Everything in this section is written by readers of the document, not by
Andrew Greenberg, and none of it may be cited as his.** It is retained because
it is part of the retrieved artefact and because it names other sources. It is
reproduced verbatim from the same retrieval.

```text

[a]the checklist is great, and we have your video + presentation to go with it, but I guess a 'working document/page/blog/presentation' is needed properly explaining the rationale, e.g. https://blog.poly.nomial.co.uk/2025-08-10-creating-high-quality-electronics-schematics.html (but kept up-to-date)
[b]i just also realized it's two checklists in one. For one, it's a checklist on 'improving readability' as per your presentation at KiCon. The other is 'general design guidelines' which are in itself important, but a different topic maybe?
[c]Next, someone else made a blogpost about this as well https://blog.poly.nomial.co.uk/2025-08-10-creating-high-quality-electronics-schematics.html


where the following points make a lot of sense too, which helps with the visual bits (maybe less on the reading bit ...)


1. Stick to the grid


2. Avoid small and unnecessary turns


3. No random angles


6. Avoid spaghetti


7. Symbols are logical, footprints are physical (you have this, but I think the important point is, that its often preferably to make your package match its use-case e.g. if your uart is on pins 2 and 6, re-organize these pins on your package to have them together. imo kicad should become better in this, but this might be a hard problem)


8. Route out of pins, not across pins


9. Label the purpose of switches, LEDs, and connectors (generally goes with your add lots of comments, but this makes it more concrete (for these parts))


10. Give yourself space -> Don't run wires through text (you cover the space part)


11. Be logical about where you place things


Btw, not a fan of
14. Decide on a net naming convention and stick to it


In that yes, of course he's right, but everybody has their own convention. Since this is about KiCAD, stick with the KiCAD convention ;)
[d]My own question as a 'novice' PCB designer. Should decoupling capacitors that are close to the IC be connected to the symbol power inputs directly, or placed elsewhere on the schematic separately? On one hand it shows intent clearer (these caps should be close to the IC), but on the other it can be a bit messier in the schematic
[e]Olliver S. shared a nice link to a list of guidelines. #11 addresses your question. 
https://blog.poly.nomial.co.uk/2025-08-10-creating-high-quality-electronics-schematics.html
[f]I missed that, thanks Brandon!
[g]I think we can do better! While all mentioned points are important!


First, we should use (SI) version of RKM https://en.wikipedia.org/wiki/RKM_code for reference designators (SI support was later added avoiding the painful eyesore of '4K7 by using 4k7, which types easier to boot).
[h]Further more, on refdes's while I get that on the other side of the pond the squigly resistor symbol is preferred, but the square euro one has as an advantage you can put the refdes in the box. this works well up until R999 (from a size point of view) What's your opinion on that?
[i]That's fine, I don't have a big opinion either way. I joke about "freedom symbols" but both have advantages.
[j]Here is where I prefer automation. I use some tools https://gitlab.com/ci-includes/kici (made by me) to do a lot of things via 'a software developer flow' e.g. commits, tags, generation etc.
```

---

*End of snapshot. Retrieved 2026-08-22 by `lane-snap`. Do not edit this file; a
new retrieval is a new dated file.*
