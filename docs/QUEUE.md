# The queue

Where the work stands and what comes next — the file a fresh session reads
first, and the one a subagent never sees. `CLAUDE.md` is the rules; this is
the state; `docs/journal/` is the story; `docs/PARKED.md` is the backlog,
read when a wave is composed and never at boot.

This file **deletes**; `queueledger.py` fails the build on a number that
leaves in silence or is referred to and never booked; `docs_guard.rs` bounds
it by items (18 at 8 lines, the handoff 32). **A finding that names no
score parks**: only the headline's frame, a floor, or a takes-chain books here.

## Where things stand

*2026-09-21, the tenth chain. Chapter two **637 → 645** (472); the AI
word holds at 10234 and 475 has named what holds it.*

- **The AI word is a road search this crate never runs** (475, ORDERS
  §18.2). At f10234 the two sides agree on draws 0-3 and part at draw
  4: the original's is the first of 198 at `calc_road_cost+0x46 <
  astar_caravan_road+0x52b < find_road+0x3a8`, ours is `set_anim`. One
  road search over 10234 (198) + 10235 (73), then nothing for 2,300
  frames. **478 is the headline item**; 476 and 477 are its frame's
  value rows. A *missing* behaviour, not a divergent one.
- **475's own first reading was wrong and it retracted it**: `FLOORS[1]
  .word = 1850` is the *score* run's word, and `unwrap_or(last)` makes
  it mean "never parted inside it". Great Lakes' draw word is 10234 and
  the stream is aligned. The trap is written into §18.2. What was false
  is one clause of §17.5 — the owner of the draws, not the count.
- **The attribution audit came back clean**: 464, 465 and 471 all
  *cited* it, none rested on it, and 465 steered away from it
  explicitly. Nothing pinned depends on it (§18.4).
- **A chase ends on a clock** (472, COMBAT §36): `Unit::work:440` on
  `(o+frame) % 16 == 0`. `visible` exact 5 → **8 of 9**, re-pinned
  upward; widening map over `[606, 641)` 13 → **0**. 473 landed free.
  Four open, 67 parked. **Fable backlog: 2 Loop items** (313, 480).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w9711 of 24,000 · GreatLakes w10234 of 24,000
Golden: w626 of 901 (ch1) · ch2 w645 · 479 next
Endpoint 24001: EastIndies 64 off, 10 unlinked · GreatLakes 44 off, 11 unlinked

**Opener: 479 is live; 478 takes the freed lane with run117 approved.
The loop stands at 10 of 20.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden record's word for the rules, the long captures' word for
the AI, lower map first — Great Lakes. Take the first unstarted on either
track unless a better order is obvious, and say so. Numbers are stable;
the backlog is `docs/PARKED.md`, and an item returns only when a score names it.

478. **`calc_road_cost` draws ours 0 theirs 198 on f10234** — the AI
    headline's own parting, first parting draw index 4, call chain
    `astar_caravan_road < find_road`. The original runs a road search
    here and this crate runs none. **Which of `find_road`'s three
    callers** (`Caravan::process`, `Caravan::build_road`,
    `BuildType::place_roads`) is **not on disk** — run53's `cover` was
    off and nothing proxies a call. **run117 is reserved and the
    capture is approved.** ORDERS §18. No mechanism named.

476. **f10234's three value rows**: `0/5 order:length` 2/1,
    `0/5 orders.len` 2/1, `0/2001 gather:gather_down[-1]` 5/2. The
    word's own frame, value side. ORDERS §18.3.

477. **f10235's `1/28 pos`**, ours (4801,30175) theirs (4800,30175),
    with `g.x[0]` and `g.des_x[0]` the same — one world unit in x,
    said three times. ORDERS §18.3.

479. **`0/9`'s three rows on block 645** (the rules headline's own
    frame). `pos` ours (942,8098) theirs (912,8096); `order` Length
    2/1, Kind 1/10, PathLength 3/0; `0/10` and `0/11` both take
    Target (1,8) where the dump takes (1,6). 472's reading — **its
    hypothesis, not this item's** — is `do_move`'s captain arm
    (`o_up < 0`) taking an incumbent from `near_o`/`near_who` into
    `change_target`. COMBAT §36.6 has the `in_range 1 /
    ever_in_range 0` fingerprint. Re-measure first.

342. **Host choice seats the scholar on the wrong university** — 338's
    residue, re-pinned three times, and now `1/55`. **Re-measure before
    diagnosing**: the vector `(768, 9984)` is the claim, never the count.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch, headline
  first and whether it moved; delete finished items — their story is in
  `docs/journal/`. Shorten only what you are touching, never another
  author's item, and never widen a line to beat a count.
- **A residue item is booked by its frame and its draw delta** (DECISIONS
  42); a mechanism in its title is the previous item's hypothesis.
- **A finding parks by default.** `docs/PARKED.md` takes what names no
  score; this file takes only what names a headline's frame, a floor, or
  a takes-chain to one. **Neither headline slot is ever empty**: no item
  on the AI word's frame means the widening of that frame; no rules item
  means the next unpinned chapter of the golden record. Loop items —
  tooling, guards, these rules — are the steering pass's, never a worker's.
- **A worker never books a number and never edits this file or
  `docs/JOURNAL.md`.** It reports; the commander books and writes these
  lines; the story is `docs/journal/<date>-item-<N>.md`. **A pinned
  constant is the worker's to re-pin; the line is the commander's.** The
  chain — merge, book, gate, push, reap — is `CLAUDE.md`'s. **A number is
  measured on the tip**, after the last `ccc update`, or names its tree.
- **The floors and these lines move together** — `FLOORS`, `LONG_WORD_*`,
  `GOLDEN_WORD_*`, `ENDPOINTS` with `Scoreboard:`, `Long captures:`,
  `Golden:`, `Endpoint <frame>:`, `Fable backlog: N Loop items` — read
  literally and **never wrapped**, or the count guard matches this line
  instead (twice, 09-19). The length guard counts every line to the next `## `.
- **A capture is a draw-stream trace first, detail on demand** (DECISIONS
  41): whole length at `cover=0`, then a windowed re-run sized to the word;
  after 363, the click-free lane unless it needs the mouse. Overlap the
  neighbours: six blocks each end (run83); `samegame.py` exits 0 when
  nothing is in common, so assert the count, not the verdict.
- **Grep for the derived quantity, the SHAPE and the GATE**: a term zero in
  every dump is either zero-valued or switched off (117, 178). **A
  never-cleared field asserts a CHANGE, not a value** (`collide_frame`, run85).
- **The gate is `python3 tools/release_gate.py <install> --test-threads 4`**,
  to a file, never piped — a pipe launders the 137. A commander's clear is
  free when every landed branch is merged, gated, pushed and reaped,
  nothing is in flight, and both headlines are measured. **Before a blind
  fan-out**, grep `CLAUDE.md` and the memory index: a subagent inherits both.
