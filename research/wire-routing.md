# R9 — Orthogonal schematic wire routing

Status: algorithm and cost model specified; all defaults are grounded in
**measured statistics from real schematics** (§2), not guesses.

Prerequisite reading: [`geometry.md`](geometry.md) (pin positions, body boxes),
[`style-rules.md`](style-rules.md) §4 (the routing rules the score will judge
this router's output by).

---

## ⚠ Contradictions and cautions for `spec/SPEC.md`

1. **SPEC §7's label threshold and R8's `KI-LBL-001` must be the same knob.** If
   the router emits a wire at 250 mm and the linter penalises wires over 381 mm,
   the tool argues with itself. One config key,
   `routing.label_threshold`, read by both.

2. **SPEC §4 lists `wire connect A B` and `wire draw` but not "route to a net".**
   In practice the common agent request is "connect `U1.7` to `+3V3`", where the
   target is a *net*, not a point. §5.3 specifies it; SPEC's command surface
   should gain `kicli wire connect <pin> <net>`.

3. **Determinism needs stating as a testable property, not an aspiration**
   (Constitution §4). §7 gives the exact tie-break chain and the property test.

---

## 1. Problem statement

Given a source terminal and a target terminal on one sheet, produce an ordered
list of grid points forming an orthogonal polyline such that:

- every vertex is on the placement grid `G` (default 12700 IU = 50 mil);
- every segment is axis-aligned;
- the path does not pass through symbol bodies;
- the path does not run *along* another net's wire (which would read as a
  connection);
- crossings of other nets are allowed but costed;
- the result is identical for identical inputs, on any platform, forever.

Then serialise as `(wire (pts (xy …) (xy …)) …)` records, one per segment
(`sch-format.md` §3.1 — KiCad wires are always two-point).

---

## 2. What real schematics look like (measured)

Over four representative demo sheets (46–234 symbols, 1,267 wire segments
total), using the R10 extractor:

| Property | Measured |
|---|---|
| segments that are axis-aligned | **99.5 %** (6 diagonals in 1,267) |
| wire endpoints on the 50 mil grid | **100.0 %** |
| segment length: p10 / p50 / p90 / p99 / max | 1.27 / 6.35 / 20.32 / 33.02 / 71.12 mm |
| segments ≤ 1 grid step | 256 / 1,267 (20 %) |
| segments ≤ 2 grid steps | 448 / 1,267 (35 %) |

Crossings without a junction, per sheet:

| Sheet | wires | junctions | crossings | crossings per 10 wires |
|---|---|---|---|---|
| `ampli_ht` (hand-drawn, tidy) | 81 | 13 | 1 | **0.12** |
| `One-Air-Max` | 517 | 77 | 21 | **0.41** |
| `in_out_conn` (busy) | 323 | 29 | 43 | **1.33** |

Three conclusions that shape the design:

1. **The grid assumption is not an approximation — it is what schematics are.**
   100 % of endpoints are on grid, so a lattice router is exact, not lossy.
2. **Routes are short.** Half of all segments are ≤ 6.35 mm; a router optimised
   for long-haul paths would be solving the wrong problem. The
   candidate-shape fast path (§4) will handle the overwhelming majority.
3. **0.1–1.3 crossings per 10 wires is the real-world range**, which both
   calibrates R8's normaliser and tells the router that crossings must be
   discouraged but not forbidden.

---

## 3. The routing graph

### 3.1 Lattice

Nodes: grid points within the routing window — the bounding box of the two
terminals inflated by `routing.margin` (default 8 G ≈ 101.6 mm), clipped to the
page. For a full A3 page at 50 mil the lattice is 331 × 234 = 77,454 nodes;
with turn-aware states (node × 4 directions) that is ~310 k states, which A*
explores a small fraction of. Performance is a non-issue; correctness and
determinism are the whole game.

### 3.2 Obstacles and their treatment

| Feature | Treatment |
|---|---|
| symbol body box (`body(s)`, `geometry.md` §6) | **hard block**, except the grid point at a target pin |
| pin of another symbol | hard block, plus a 1 G no-enter halo along the pin's own direction |
| existing wire of **another** net, crossing perpendicular | allowed, cost `w_cross` |
| existing wire of another net, **collinear/overlapping** | **hard block** — it would render as a connection |
| existing wire of the **same** net | free; entering it terminates the route (and creates a junction, §5.4) |
| junction, no-connect | hard block unless it is the target |
| label / text bounding box (`tbox`) | soft, cost `w_text` per grid step inside |
| sheet body box | hard block, sheet pins are terminals |
| page border margin | hard block |

Obstacle lookup is a hash grid keyed by grid cell, built once per route request
in O(objects); queries are O(1).

### 3.3 Pin escape

A route leaving a pin **must** first step `≥ 1 G` in the pin's own direction
(`geometry.md` §3.3). Wires approaching a pin from the side look wrong and, for
pins whose graphic style has a marker (clock, inverted), overlap the marker.
Same rule at the target. This is a hard constraint, not a cost.

---

## 4. Algorithm: shapes first, A* second

### 4.1 Why not A* alone

A* with a corner penalty produces *a* minimal route, but among equal-cost routes
it picks whichever the tie-break happens to favour, and those are often not the
route a human would draw. Enumerating the canonical shapes first gives natural
results and makes 90 %+ of requests O(1).

### 4.2 Candidate shapes, in evaluation order

Let `S` = source escape point, `T` = target escape point.

1. **I** — straight: only if `S.x == T.x` or `S.y == T.y`.
2. **L** — two variants: horizontal-then-vertical, vertical-then-horizontal.
3. **Z** — three segments with one free coordinate `m`:
   - vertical-mid: `S → (m, S.y) → (m, T.y) → T`, `m` over grid columns in
     `[min(S.x,T.x), max(S.x,T.x)]`
   - horizontal-mid: symmetric.
   Evaluate all candidate `m` (bounded by the span, typically < 30).
4. **U** — four segments, needed when both terminals face the same way; free
   coordinate is the "outward" offset, searched outward from 1 G to
   `routing.u_max` (default 6 G).

Each candidate is validated against §3.2 and costed by §6. If any candidate is
feasible, take the cheapest (ties broken by §7).

### 4.3 A* fallback

If no shape is feasible (dense sheet, obstacles in the way), run A* over
turn-aware states `(x, y, direction)`:

- `g` = accumulated cost (§6);
- `h` = Manhattan distance × `w_len` + (`w_turn` if a turn is unavoidable), which
  is admissible because every cost term is non-negative and `w_len` is the
  per-unit-length term;
- expansion order fixed: `+x, −x, +y, −y` (never data-dependent);
- the priority queue orders by `(f, g, x, y, dir)` — a total order, so no ties
  are ever resolved by heap internals.

### 4.4 Escalation

If A* fails (no path), the router does **not** give up silently. It reports
`"status": "blocked"` with the blocking objects' handles, and — if the terminals
are further apart than `routing.label_threshold` — offers the label fallback
(§5.5) as the suggested action. An agent that gets "blocked" plus a list of what
blocked it can act; an agent that gets "failed" cannot.

---

## 5. Request forms

### 5.1 `wire connect <pin> <pin>`

The base case above.

### 5.2 `wire draw <pin> <x,y> <x,y> … <pin>`

Explicit vertices; kicli validates (orthogonal, on grid, no hard-obstacle
violations) and refuses with a structured error rather than drawing something
illegal. No search.

### 5.3 `wire connect <pin> <net>`

Target set = every grid point on the net's existing wires, plus its pins. A*
runs with a multi-source backward heuristic (distance to the nearest target
point). The shape fast path is tried against the *nearest* target point first.
This is the common agent request and should be first-class.

### 5.4 Junctions

If a route terminates on the interior of an existing same-net wire, kicli emits
a `junction` at that point. If it terminates at an existing endpoint, it does
not (KiCad renders a corner). Terminating such that four wire ends meet at one
point is **refused** — it creates the four-way junction that R8 `KI-JCT-001`
penalises; the router offsets by 1 G instead and reports that it did.

### 5.5 Label fallback

When the best route's path length exceeds `routing.label_threshold` (default
300 G = 381 mm, shared with `KI-LBL-001`) or A* reports blocked, the router
proposes paired labels instead:

```
status: labels
reason: path_length 447.04mm > threshold 381.00mm
action: add local label "SPI_SCK" at U1.14 (+2 G along pin direction)
        add local label "SPI_SCK" at U7.3  (+2 G along pin direction)
```

It **proposes**; it does not silently do it. `--auto-labels` performs it, and the
output says so. The name is derived from the net's existing name, or from
`<source-ref>_<pin-name>` when the net is unnamed.

---

## 6. Cost model

```
cost = w_len   · length_in_grid_steps
     + w_turn  · corners
     + w_cross · crossings_of_other_nets
     + w_text  · grid_steps_inside_text_boxes
     + w_near  · grid_steps_within_1G_of_a_symbol_body
     + w_offgrid_penalty (∞ — hard constraint, listed for completeness)
```

Defaults, chosen so that the router's preferences match R8's penalties (a router
that optimises a different objective than the linter scores is a bug):

| Term | Default | Rationale |
|---|---|---|
| `w_len` | 1 per grid step | the base unit |
| `w_turn` | 6 | measured median segment is 5 grid steps, so a corner must cost more than a modest detour or the router zig-zags |
| `w_cross` | 20 | crossings are the most visible defect (R8 `KI-XING-001`); 20 means "detour up to 20 grid steps to avoid one crossing" |
| `w_text` | 12 per step | routing through a label is nearly as bad as a crossing |
| `w_near` | 2 per step | mild preference for breathing room around symbols |

All integers — the cost is an `i64`, so there is no floating-point
non-determinism anywhere in the router (Constitution §4).

These weights are a *starting point tied to measured data* (§2), and should be
re-checked once the R8 calibration corpus exists: routing a known-good sheet's
nets from scratch should reproduce something close to the original routing.
That is a strong, cheap test — Q3.

---

## 7. Determinism

Guarantees, in order of application:

1. Candidate shapes are enumerated in the fixed order I, L(h-first), L(v-first),
   Z(vertical-mid, `m` ascending), Z(horizontal-mid, `m` ascending), U(offset
   ascending).
2. Cheapest wins.
3. Ties broken by: fewer corners → smaller total length → lexicographically
   smallest vertex sequence (as a list of `(x, y)` integer pairs).
4. A* uses a total order on `(f, g, x, y, dir)`; no `HashMap` iteration ever
   affects a decision (use `BTreeMap`/sorted vectors in any path that feeds the
   search).

**Property test**: for every fixture and every pair of terminals, routing 100
times (and across a shuffled input item order, since `sch-format.md` §1.1 says
KiCad reorders items) yields byte-identical output. This test would have caught
every non-determinism bug I have seen in this class of tool.

---

## 8. Output contract

Text:

```
routed U1.14 -> R7.1   via 3 segments, 2 corners, 38.10mm
  cost 62 = len 30 + turns 12 + crossings 20 + text 0 + near 0
  crossings: 1 (net GND at 152.40,88.90 on wire da5aa983)
  adjusted: R7.1 by 0.00,1.27mm (four-way)
  wires added: 3   junctions added: 1
```

The `adjusted` line is omitted entirely when nothing was adjusted.

JSON (`--output json`):

```json
{ "status": "routed",
  "from": "U1.14", "to": "R7.1",
  "path": [[139.7,88.9],[152.4,88.9],[152.4,101.6],[165.1,101.6]],
  "segments": 3, "corners": 2, "length_mm": 38.10,
  "cost": { "total": 62, "length": 30, "turns": 12, "crossings": 20,
            "text": 0, "proximity": 0 },
  "crossings": [ { "wire": "da5aa983", "net": "GND", "at": [152.4,88.9] } ],
  "adjusted": [ { "terminal": "R7.1", "by": [0.0,1.27], "why": "four-way" } ],
  "added": { "wires": ["uuid…","uuid…","uuid…"], "junctions": ["uuid…"] },
  "joined_net": null,
  "alternatives_considered": 7 }
```

`status` ∈ `routed | labels | blocked | invalid`. The cost breakdown is the point
of the whole exercise: the agent can see *why* a route is bad and decide to move
a symbol instead of accepting it — which is the loop Constitution §3 is asking
for.

**A terminal the router moved is reported structurally, not in prose.** Ruled
2026-08-15, on the frozen-surface question the four-way avoidance task raised.
§9's Q2 rules that a route which would make a fourth wire end meet at one point
is refused and offset by 1 G, "reporting the adjustment" — and this contract had
nowhere to report it but `reason`, which is one English sentence for a person.
An agent cannot branch on English; branching on the cost breakdown rather than
parsing prose is the whole point of this contract. So `adjusted` is a list, empty
when nothing moved, of `{ terminal, by, why }`:

- `terminal` names itself exactly as `from` and `to` do, so the caller can tell
  which end moved without comparing coordinates;
- `by` is a **displacement**, not a position. Where the terminal ended up is the
  corresponding end of `path`, and this contract does not store what it can work
  out — the requested point is that end less `by`;
- `why` is a **closed set**, currently the single value `four-way`. A new reason
  is a new value and a compile error at every match on it, which is what makes it
  safe for an agent to switch on. It is never free text.

`reason` continues to say the same thing in English for a person, and carries no
load an agent must parse.

**The net a connection joined is a field of this contract, and it is attributed
at the same seam.** Ruled 2026-08-22 — James's ruling on BLOCKED 2 at the M4
close, `tasks/M5/opening-1-joined-net-contract.md`. `wire connect` must answer
*which net are these two ends on now*, and until this ruling the command layer
answered it in a key beside the contract rather than inside it, so the contract
did not describe the whole of a route's result. `joined_net` is now that answer:

```
joined: net SIG_A
routed R1.1 -> R2.1   via 3 segments, 2 corners, 43.18mm
  cost 70 = length 34 + turns 12 + crossings 20 + text 0 + proximity 4
```

- **It is read back out of the written file, never predicted.** What the two
  ends are on is a property of the drawing kicli has just written, not of the
  arithmetic that produced it, so it is taken from the file. It is therefore
  not derivable from any other field here.
- **In JSON it is `null` when nothing was joined, and the key is never absent** —
  the same every-key-at-every-status rule the rest of this contract keeps. Three
  cases give a null: a proposal, which wrote nothing to join anything into; a
  connection between two ends that name no pin; and `wire draw`, which takes the
  corners it was given and is not asked to join two ends.
- **The text line is printed only when there is a name to print**, which is the
  rule the `adjusted` line already follows. It sits **above** the status line —
  the one thing in this contract that does — because it is the answer to the
  question `wire connect` was asked, and because that is where the command layer
  printed it before it was a field here.

**A crossing names the wire it crossed, and the net is attributed at the seam.**
The first draft of this contract reported a crossing as `{ net, at }`. The
search cannot answer that: the obstacle map knows the **wire** on a cell, and
whose net a wire carries is connectivity's answer, not geometry's. So `wire` is
the search's own truth and is always present; `net` is filled by the caller on
the same seam that sorts a wire into the route's own or another's, and is `null`
when the caller did not attribute it. The text form omits the `net` clause in
that case and prints the wire alone. Having the router ask the extractor instead
would put connectivity inside the search, which is the thing the caller-supplied
net list exists to prevent.

---

## 9. Scope boundaries for v1

| In | Out |
|---|---|
| two-terminal routes, pin↔pin, pin↔net | full net re-routing / global optimisation |
| junction creation and four-way avoidance | bus routing and bus entries (needs its own model) |
| label fallback proposal | automatic net naming policy beyond the derived default |
| single sheet | cross-sheet routing (that is what hierarchical labels are for) |

Rip-up-and-reroute is explicitly out: SPEC D14 says there is no undo, and a
router that rewrites existing wires would violate the "every mutation is small
and reviewable" spirit of Constitution §5.

---

## 10. Open questions for James

- **Q1 — `w_turn = 6`.** This makes the router prefer one long detour over two
  corners. Sanity-check against your taste on a real board before it is frozen;
  it is the single most visible parameter.

- **Q2 — Four-way junction policy.** Proposed: refuse and offset by 1 G,
  reporting it. Alternative: allow with a warning. Recommendation is refuse,
  because R8 penalises it and the router should not create work for the linter.

- **Q3 — Router-vs-corpus calibration.** Approve the test "re-route every net of
  a known-good sheet from scratch; assert total cost is within X % of the
  original"? It is the only objective measure of whether the weights are right.
  Needs you to pick X (suggest 15 %).

- **Q4 — `routing.margin` default (8 G).** Larger margins allow prettier detours
  at some search cost. Fine as is?

---

## 11. Reproduction

| § | How |
|---|---|
| §2 wire statistics | `exp/r10/extract.py` over the recipe-C corpus; count orthogonality, grid alignment, segment lengths |
| §2 crossings | segment-pair intersection test excluding junction points, per sheet |

Sources: measured from the KiCad 10.0.5 demo corpus (recipe C,
`sch-format.md` §0.2); wire record grammar from `sch-format.md` §3.1; obstacle
geometry from `geometry.md` §6.
