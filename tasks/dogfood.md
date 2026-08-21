# Dogfood defects

kicli's end user is an LLM agent, so an LLM agent tests it. A dogfood subagent
gets `AGENT.md`, the built binary and a short design brief — no source, no task
files, no spec — and attempts the brief cold. **Everything it fumbles is a
defect**: a command misused, a document misread, output that overflows or
confuses its context.

Defects are recorded here **verbatim**, then triaged like any finding — fixed,
PROPOSED, or recorded with the reason it stands. The verbatim rule is the point:
a defect summarised by the person who wrote the thing being tested is a defect
already half-explained away.

---

## Run 1 — 2026-08-15, M4 Phase 2 checkpoint 2. Dry run; gates nothing.

**Standing from M5; a dry run in M4**, per `CLAUDE.md`'s dogfood gate. This run
gates nothing, and every defect below is real anyway.

**Setup.** Sandbox at `/tmp/kicli-dogfood-XNUf9M`, outside the repository:
`AGENT.md`, the debug binary, and a copy of the `nets` fixture as `board/` — 21
symbols, 28 nets, one sheet. The agent was told to work only inside the sandbox
and specifically not to look for source, tests or spec.

**Brief.** Add a third resistor to `NET_A`, connected so it genuinely joins the
net "not merely so that it looks connected", verify it; then take it back out and
confirm the board is as before.

**Outcome: both halves achieved and verified.** The agent placed `R22`, drew a
wire from its pin, added a `NET_A` local label, and confirmed
`N NET_A=/NET_A: R1.1 R2.1 R22.1`. It then removed label, wire and symbol and
confirmed the board matched its original counts exactly. **It succeeded — and
found nine frictions doing it.** A successful run with a long friction list is
the expected shape of this exercise, not a contradiction.

### The defects, verbatim

> 1. **`--uuids` gave a useless handle on this board.** `kicli sch view --uuids`
>    documents: "Add the first eight characters of each object's identifier." In
>    this project every single object — 19 symbols, 2 power symbols, sheet ports
>    — printed the identical handle `@00000000`, e.g. `S R1 10k R @00000000` and
>    `S R22 10k R @00000000` right after placement. This is because every UUID in
>    the file is literally `00000000-0000-4000-8000-0300000000xx` (confirmed by
>    `grep '(uuid' board.kicad_sch`). This is a fixture artifact, not necessarily
>    a kicli bug — but it's exactly the kind of thing a real user hits: the doc's
>    own worked-example (`da5aa983`) implies distinct short prefixes are the
>    norm, and I could not use `--uuids` output at all to disambiguate or address
>    a specific object here. I had to fall back to full mutation-report handles
>    (`f8819360`, `906eceb2`) captured from write commands instead.
>
> 2. **No documented way to find a pin's drawn coordinates.** To wire a new
>    resistor into a net I needed to know where its pin actually sits, but
>    AGENT.md has no `sym pins`/`pin list`/`pin info` command and the layout view
>    (`L`, `F`, `T`, `W` records) never lists pin coordinates. I had to
>    reverse-engineer the offset by comparing R1's anchor (`50.80,50.80`) against
>    the label kicli itself had placed at R1's pin (`50.80,41.91` — an 8.89mm
>    offset) and assume R22, being the same symbol at the same angle, had the
>    same offset. This worked, but only because `--from-pin` internally resolved
>    the true point and reported it back to me in the wire-draw output
>    (`180.34,41.91..180.34,46.99`) — i.e., I only found out the guess was
>    slightly wrong (46.99 vs my assumed 50.80) *after* the fact, from a write
>    command's side effect, not from any read-only inspection command.
>
> 3. **Wire-report coordinate format contradicts AGENT.md's own example.** The
>    doc's worked example for `wire draw` shows:
>    `+ W 3300f00e (50.80,50.80) -> (63.50,50.80)`
>    with parenthesized points and a `->` arrow. What I actually got was:
>    `+ W 906eceb2 180.34,41.91..180.34,46.99`
>    — no parentheses, `..` instead of `->`, and (confusingly) the endpoints
>    appear to be in *reverse* order relative to the `routed` line directly above
>    it (`routed R22.1 -> 180.34,41.91`, i.e., from-pin to to-point), while the
>    `W` line reads `41.91..46.99`, i.e., to-point first, from-pin-resolved-point
>    second. I had to sit and reconcile these by hand; a first-time reader
>    trusting the doc's format literally would misparse this line or assume
>    something had gone wrong.
>
> 4. **`project info` and `sch view` report two different, undocumented "net
>    count" numbers, and nothing explains the relationship.** `project info` said
>    `nets 28`; `sch view` (connectivity) said
>    `# scope project sheets=1 sym=19 pwr=2 nets=10` alongside
>    `# 18 pin(s) join nothing`. Only by noticing `10 + 18 = 28` did I work out
>    that `project info`'s `nets` figure apparently includes every unconnected
>    pin as a singleton net, while `sch view`'s `nets=` counts only
>    named/multi-pin nets. AGENT.md documents neither figure's composition, and
>    nothing signposts that they're the same underlying concept counted two
>    different ways. This cost me a double-take and is exactly the kind of
>    surprise the brief asks me to flag.
>
> 5. **`sym delete`'s report doesn't say whether the shared symbol definition was
>    dropped.** AGENT.md says of `sym delete`: "The embedded definition stays if
>    another placement still uses it, and goes if none does." When I deleted R22
>    (whose `Test:R` definition was also used by R1 and R2), the entire output
>    was:
>    ```
>    - S R22 10k Test:R
>    checked: every invariant passed
>    ```
>    There is no line telling me which branch of that either/or happened. I have
>    to infer it did the right thing only because R1/R2 still render correctly,
>    not because the tool told me. For a case the docs specifically call out as a
>    two-way fork, I expected the report to say which way it went.
>
> 6. **The `--to-at` guess for `wire draw` is not snap-forgiving in the way other
>    verbs are.** AGENT.md is explicit that "a vertex off the grid is refused,
>    not snapped" for wires, unlike every other verb. I got lucky and picked a
>    coordinate that happened to be on-grid and axis-aligned with the resolved
>    pin, but the doc gives no way to *ask* what the legal/expected terminus is
>    before trying — I only find out post-hoc from the `routed`/`W` report, or
>    via a `blocked`/`invalid` refusal that costs a whole write attempt. This is
>    a documented design choice, not a bug, but combined with friction #2 (no
>    pin-location command) it means placing a first wire onto a new symbol is
>    trial-and-error by default.
>
> 7. **Brief ambiguity, not a tool defect, but worth recording since it changed
>    my actions:** the brief's step 2 header "Then take it back out" (referring
>    to "a third resistor" from step 1) is followed by "Remove the wiring you
>    added, and confirm the board is back to NET_A carrying what it carried
>    before" — which only mentions removing *wiring*, not the resistor symbol
>    itself. Read strictly, deleting the label and wire alone would satisfy the
>    literal instruction and leave `NET_A: R1.1 R2.1` exactly as before, while
>    leaving a dangling, fully-disconnected `R22` symbol on the sheet. I judged
>    that "take it back out" most naturally means removing the resistor too
>    (otherwise the board is not "back to what it carried before" — it carries
>    one extra unconnected part, and `project info`'s `nets` count would read 30
>    instead of 28). I deleted `R22` as well. This is my own judgement call under
>    an ambiguous instruction, flagged per the brief's own request to report
>    guesses.
>
> 8. **Minor: every invocation prints an unrelated `zoxide` shell-config warning
>    to stderr before any kicli output**, e.g.:
>    ```
>    zoxide: detected a possible configuration issue.
>    Please ensure that zoxide is initialized right at the end of your shell
>    configuration file...
>    ```
>    This is almost certainly local sandbox/shell noise rather than a kicli
>    defect, and I don't believe it originates from kicli itself — flagging only
>    because it appeared on every single command and had to be mentally filtered
>    out each time before reading real output, which is exactly the kind of
>    interleaved noise that would waste tokens/attention for an agent parsing
>    output programmatically if it were not filtered.
>
> 9. **The two-minute `kicad-cli` font-cache warm-up note is easy to miss and
>    fires unpredictably.** `project info` and `project check` both printed
>    `kicli: asking /opt/homebrew/bin/kicad-cli its version. The first KiCad run
>    on a machine builds the font cache. It can take over 120 seconds.` on every
>    run in my session (not just the first), which contradicts AGENT.md's
>    description under `project check`: "The first run on a machine can take over
>    two minutes; later runs take under a second." The message appeared
>    identically on repeated `project info`/`project check` calls late in my
>    session, not just the first one — either the "first run" caching AGENT.md
>    promises isn't happening for `project info` (only documented for
>    `project check`), or the warm-up note is printed regardless of whether the
>    cache was actually cold. I didn't measure wall-clock time closely enough to
>    be certain this is a real perf regression versus just a repeated-but-harmless
>    log line, so I'm flagging it as "message reappears where the doc implies it
>    shouldn't" rather than a confirmed timing bug.

### Triage — RATIFIED IN FULL by advisor ruling, 2026-08-15

The triage below was written by the orchestrator and **ratified in full**. Two
items were promoted at ratification: **D1 runs as a chore after the checkpoint**,
golden changes included as part of the change; **D2 goes to the M5 planning list
as a task**, noting that the answer already exists internally in
`route::terminal` (`Terminal::of_pin`) and is merely unexposed. D3–D6 proceed as
filed. Two standing instructions for the next run: **a clean shell environment**,
and **the brief-writer owns the brief-ambiguity lesson** — an ambiguous brief
spends the run on the brief rather than on the tool.

### Triage

Every defect gets one of three outcomes: fixed, PROPOSED, or recorded with the
reason it stands. Nothing is closed by being explained.

| # | Verdict | Where it goes |
|---|---|---|
| 1 | **Real, and already half-known.** | C5's second half — see D1 below |
| 2 | **Real, and the largest of the nine.** | D2, PROPOSED as a task |
| 3 | **Real defect, introduced by the verb surface (T16) today.** | D3 — **FIXED**, incidentally, by the label proposal (T13) |
| 4 | **Real documentation defect.** | D4, chore |
| 5 | **Real, small, and the doc invites it.** | D5, chore |
| 6 | Consequence of a ruled design choice, sharpened by #2 | folded into D2 |
| 7 | **Not a tool defect — the orchestrator's brief was ambiguous.** | recorded, stands |
| 8 | **Not a kicli defect — the sandbox's shell.** | recorded, stands |
| 9 | **Real, and it is two questions.** | D6, chore for the doc half |

**D1 — every fixture object still answers to one handle.** Defect 1 is the
**second half of C5**, which fixed the probe crate and explicitly scoped the
committed fixtures out, naming them "a known second half rather than swept in
silently". This run is that half arriving with a cost attached: the agent could
not use `--uuids` at all and fell back to reading handles out of write-command
reports. **PROPOSED: do the fixture half.** Recommendation: accept, as a chore
after the checkpoint — it moves goldens, which is why C5 held it back, and the
golden change is part of the change. Note the agent correctly diagnosed it as a
fixture artifact rather than a tool bug, which is the diagnosis C5 already
recorded.

**D2 — nothing read-only will tell an agent where a pin is.** The agent had to
infer a pin offset from a label kicli itself had placed, and learned its guess
was wrong only from a **write command's** output. Defect 6 is the same wound: a
wire vertex is refused rather than snapped — a ruled and correct choice — but
there is no way to ask what would be accepted, so a first wire onto a new symbol
is trial and error. **PROPOSED: a read-only way to ask where a symbol's pins
are**, as a task, not a chore. Recommendation: accept for M5 planning. This is a
design decision about the agent-facing surface and it deserves an entry rather
than a patch; note the router already resolves pins internally (`route::terminal`,
`Terminal::of_pin`), so the answer exists and is simply not exposed.

**D3 — FIXED, before the chore was ever run.** The label proposal (T13) was
editing `AGENT.md` for `--auto-labels`, hit the same wrong examples, and corrected
them as a disclosed incidental change in the same commit — its tick reviewer
confirmed the correction and judged it in scope. **Verified after the fact: all
four `W` examples in `AGENT.md` now carry the `..` form the tool actually
produces** (`:455`, `:494`, `:495`, `:554`). The chore below is retained as the
record of what the defect was and how it was found, not as outstanding work.

**D3 — `AGENT.md` documents a wire delta format the tool does not produce.**
**Verified at source before filing**, not taken on the agent's word.
`AGENT.md:455` and `:515` show `+ W 3300f00e (50.80,50.80) -> (63.50,50.80)`;
`crates/kicli/src/view/snapshot.rs:781` formats it `format!("{}..{}", …)`, and
that line predates M4 — it is the delta format M3 shipped. So the examples the
verb surface (T16) added today describe a format **kicli has never produced**.

This one is worth more than its size. **The tick review for T16 approved on a
diff and a check set; this defect lives in the gap between a document and the
tool's actual behaviour, which is precisely what a check set does not cover and
what a cold reader hits first.** `agent_doc_covers_every_command` asserts a
mention (chore C7), and even a fixed version asserting a heading would not have
caught it. **Chore, chore-runner eligible**: correct both examples against
measured output. The second half of the agent's report — that the `W` line's
endpoints read in the line record's order rather than the request's order — is
**also true and also undocumented**, and the corrected example should say so.

**D4 — two net counts, one concept, no signpost.** `project info` says `nets 28`;
`sch view` says `nets=10` with `18 pin(s) join nothing`. The agent worked out
`10 + 18 = 28` unaided, which is the good outcome; the bad one is that it had to.
**Chore**: document what each figure counts, at both places.

#### D4 — Done, 2026-08-21

**Measured first, written second.** Both numbers were read out of the code and
then out of a running view, before any prose was written.

- **`project info`'s `nets N`** is `nets.nets().len()`
  (`crates/kicli/src/cli/project.rs`, `write_nets`), the whole partition from
  `connectivity::extract`: **every net of the whole project**, all sheets, power
  nets included, single-pin unconnected nets included. Nothing is filtered.
- **`sch view --view connectivity`'s `nets=N`** is not a count of nets at all —
  it is the number of `N` records the view emitted, counted back out of its own
  text (`crates/kicli/src/view/connectivity.rs:96`). A net is left out of it two
  ways: one with exactly one visible pin whose KiCad name starts `unconnected-`
  is **tallied** as `# N pin(s) join nothing` instead of listed
  (`connectivity.rs:271-277`), and one with **no** visible pin at all — every pin
  a power pin without `--include-power`, or every pin on another sheet under
  `--sheet` — is dropped in silence (`connectivity.rs:270`).

**So the dogfood agent's `10 + 18 = 28` is right, and is not a general law.** The
exact statement is `listed + tallied + hidden = total`, and it is the third term
that the agent's arithmetic did not have to reckon with. Measured on the `nets`
fixture, whole project: `14 + 18 + 0 = 32`, and `project info` on it reports
`nets 32`. Measured on the **root sheet** of that same fixture: `10 + 18 = 28`
against a project total of 32 — the per-sheet view does not reconcile with the
project figure and must not be documented as if it did. **This is why the chore
said to measure before writing: the obvious signpost, "they add up", is false
under `--sheet`, and `--sheet` is exactly what an agent reaches for on a large
project.**

**Where the signpost went.** Beside both numbers, in the document the end user
reads:

- `AGENT.md`, `### \`kicli project info\`` — and the sample output there was
  **missing the `nets` line entirely**, which is its own small part of why there
  was no signpost to find. Added, from the golden
  (`crates/kicli/tests/project_info_healthy.golden:6`), plus a paragraph saying
  the count is whole-project and pointing at the reconciliation.
- `AGENT.md`, `#### The connectivity view` — the reconciliation itself, both
  omission rules, the worked numbers above, and the per-sheet warning.
- `crates/kicli/src/cli/project.rs`, rustdoc on `write_nets`, for the next person
  who changes the number rather than reads it.

The view's own printer is in `crates/kicli/src/view/connectivity.rs`, which
another lane held open at the time; its rustdoc was left alone deliberately
rather than risk the conflict, and the omission rules are commented there
already.

**Check:** `crates/kicli/tests/net_counts_reconcile.rs`, two tests.
`the_whole_project_view_accounts_for_every_net` walks every `.kicad_sch` under
`tests/fixtures`, renders the default whole-project view, reads `listed` and
`tallied` back out of the view's **own text** rather than recomputing them, and
asserts `listed + tallied + hidden == total`. Reading the text matters: a
recomputation would be a second implementation of kicli agreeing with the first.
`a_per_sheet_view_does_not_account_for_the_project` pins the other half — if a
per-sheet view ever did add up, the document's warning would be wrong and this
is what would notice.

**Falsification.** Six breaks against the checkpointed good state; `shasum -a
256` after each restore returned `net_counts_reconcile.rs` to `f6f1f431…` every
time.

| # | The break | Result |
|---|---|---|
| A | The `tallied` term dropped from the identity | **FAILED** on `geometry/asymmetric.kicad_sch`: lists 0, tallies 32 |
| B | The `listed` term dropped | **FAILED** on `sch/item_zoo.kicad_sch`: lists 1, tallies 1, total 2 |
| C | The tally reader made blind — always returns 0 | **FAILED**, and it is the reader being wrong rather than the tool |
| D | The fixture root pointed at a directory that does not exist | **FAILED**: "only 0 fixture projects have nets, which is too few to have checked anything" |
| E | The per-sheet claim inverted, `<` to `>=` | **FAILED**: "the root sheet accounts for 28 of 32 nets" |
| F | The `hidden` term dropped | **ok** — see below |

**Break F is a no-op break, not a blind check, and the difference was
established rather than assumed.** `hidden` is **0 on every fixture**, measured
across all seven projects that have nets, so removing the term changes no
arithmetic anywhere and the identity is arithmetically identical with and
without it. The term is read from the code (`connectivity.rs:270`), not from a
fixture. **PROPOSED: no fixture exercises a net with no visible pin** — a net of
only power pins under the default view, or a whole net off-sheet under
`--sheet`. Recommendation: leave it. The term is correct, cheap, and self-
documenting, and manufacturing a fixture to exercise it belongs to whoever owns
`tests/fixtures/` rather than to a documentation chore. Recorded so that a later
reader does not mistake break F for evidence the term is tested.

**`cargo xtask check`: all six gates pass.** `cargo test` green throughout;
nothing kicli prints was changed, so no golden moved.

**D5 — `sym delete` reports a two-way fork without saying which way it went.**
`AGENT.md` specifically calls out that the embedded definition "stays if another
placement still uses it, and goes if none does", and the report says neither.
**Chore**: say which. The doc raising the question is what makes the silence a
defect.

**D6 — the font-cache note fires where the document implies it should not.** Two
questions, and the agent was right to separate them. The **documentation** half
is a chore: `AGENT.md` describes the warm-up under `project check` only, while
`project info` also invokes `kicad-cli`. The **timing** half — whether the cache
is genuinely cold each time or the note prints unconditionally — is a
measurement nobody has made, and the agent said so rather than guessing.
Recorded as unmeasured.

**7 — recorded, stands, and it is the orchestrator's defect not the tool's.** The
brief said "take it back out" and then described removing only the wiring. The
agent read the ambiguity, chose the reading that made the verification claim
true, and flagged the guess. That is the behaviour the exercise wants. **The
lesson is for whoever writes the next dogfood brief**: a brief that is ambiguous
spends the run's attention on the brief rather than on the tool.

**8 — recorded, stands, not a kicli defect.** The `zoxide` warning comes from the
sandbox's shell initialisation, inherited from the environment the orchestrator
prepared. It is noise the orchestrator introduced. Worth keeping in the record
for one reason the agent gives well: it is "exactly the kind of interleaved noise
that would waste tokens/attention for an agent parsing output programmatically".
**Next run's sandbox should start from a clean shell environment**, so the
exercise measures kicli's output and not the harness's.
