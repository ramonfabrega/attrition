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

*2026-09-22, the tenth chain. Both words moved: Great Lakes **10234 →
10237** (478), chapter two **645 → 680** (479).*

- **`BuildDump::build_masks` was never parsed on any capture ever
  taken** (478): read off BUILDDATA where the record writes it at
  **WALLDATA's** indent, so `None` everywhere while `crate::ledger`
  called it uncompared for weeks and **four items chased the frame it
  explains**. Now compared on every linked building-frame (+63k on
  run57, none wrong), indent guarded against a decoy. Sixth
  instrument defect, and the largest.
- **The word was a road replan, not a caravan** (478, ROADS §1.2):
  Farm `0/2004` dies on 10230, `remove_from_city` calls
  `City::regen_roads`, and **two** buildings replan on their own
  `(frame+o)%16` slots — `0/2006` on 10234, `0/2005` on 10235. We had
  only `Build::activate`'s call. One line. Draws 6/204 → 204/204.
- **run117 was not needed and not taken.** 475's "not on this disk"
  was wrong — run53 *was* `cover=1`, 6,936 functions. Thirteen
  seconds. §18.5 struck.
- **The sequence word is blocked on a name, not a behaviour**: 10237
  parts on a *label*, `Ammo::do_damage+0xc59`, absent from
  `trace::SITES`; naming it reaches the count word, 10244 — that is 483.
  Six open, 68 parked. **Fable backlog: 2 Loop items** (313, 480).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w9711 of 24,000 · GreatLakes w10237 of 24,000
Golden: w626 of 901 (ch1) · ch2 w680 · 481 next
Endpoint 24001: EastIndies 63 off, 11 unlinked · GreatLakes 51 off, 7 unlinked

**Opener: 481 is live; 483 takes the freed lane. run117 is released,
unused. The loop stands at 14 of 20.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden record's word for the rules, the long captures' word for
the AI, lower map first — Great Lakes. Take the first unstarted on either
track unless a better order is obvious, and say so. Numbers are stable;
the backlog is `docs/PARKED.md`, and an item returns only when a score names it.

483. **`Ammo::do_damage@00678060+0xc59` is unnamed in
    `trace::SITES`** — the AI headline's own blocker. Great Lakes'
    count word is 10244 and its sequence word 10237; 10237 and 10242
    spend the **same** draws on both sides and part on a label, ours
    an unattributed `projectiles` phase. **Until the site is named
    the sequence cannot pass 10237 however the simulation behaves.**
    A `SITES` entry plus a `SITE_*` const in `sim::fight`. Reported by
    478, not taken. COMBAT §39 reserved.

476. **f10234's three value rows**: `0/5 order:length` 2/1,
    `0/5 orders.len` 2/1, `0/2001 gather:gather_down[-1]` 5/2. The
    word's own frame, value side. ORDERS §18.3.

477. **f10235's `1/28 pos`**, ours (4801,30175) theirs (4800,30175),
    with `g.x[0]` and `g.des_x[0]` the same — one world unit in x,
    said three times. ORDERS §18.3.

481. **All three hoplites hold `0/11` where we hold `0/10`, on
    block 671** — the rules headline's own frame after 479 moved the
    word to 680 and re-pinned the window to [606, 684). `pos 1/6`
    ours (1623,8037) theirs (1608,8040); `order 1/6`, `1/7`, `1/8`
    Target (0,10)/(0,11); `1/7` Move x 1512/1128, y 8328/8424,
    PathLength 0/5; `672 angle 1/6` Heading -1320157184/-1605566464.
    479 reads it as **§33's shape mirrored onto who=1** — that is its
    hypothesis, not this item's. Re-measure first.

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
