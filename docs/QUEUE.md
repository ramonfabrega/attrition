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

*2026-09-22, the tenth chain. Great Lakes **10233 → 10244**, chapter
two **637 → 683** — and chapter two's earliest parting is **656**.*

- **A correction I owe (484).** I booked `[606, 684)` as "at nought,
  78 frames". It was clean only because the field was not compared.
  `crate::diff::compare` carried **no hit-point row at all**, and
  `damage_frac` was never parsed on a unit — read for a building
  since item 394, for a figure never. The map goes three → **eight**
  first-partings and the earliest is **656**, 27 blocks under the
  word. 478's shape one record over, twice in one chain.
- **Every value we hold is the dump's own previous one** — the same
  ladder, one arrival late, from the first wound: `1/8`'s `damage`
  0/8 at 656, 8/17 at 657, 17/25 at 660, 25/34 at 682. **`extra 1/8`
  at 684 is the end of that lag, not a fact of its own**, so 485's
  frame is 656. Word 683 and `visible` 9 of 9 both hold.
- **The AI word is 10244**, sequence and count together after 483
  named `Ammo::do_damage+0xc59`; its guard caught a site address
  transposed since item 394. 487 is live on 10245's `1/27` rows.
- **Nine instrument defects this chain**, all found by widening. 488
  parks the sharpest: one window wrong three ways, the third — a
  field absent from the comparator — unreachable by any widening.
  Seven open, 70 parked. **Fable backlog: 3 Loop items** (313, 480, 488).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w9711 of 24,000 · GreatLakes w10244 of 24,000
Golden: w626 of 901 (ch1) · ch2 w683 · 484 next
Endpoint 24001: EastIndies 63 off, 11 unlinked · GreatLakes 51 off, 7 unlinked

**Opener: 487 is live; 485 takes the freed lane. The loop stands at
19 of 20 — the next landing is the steering pass's.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden record's word for the rules, the long captures' word for
the AI, lower map first — Great Lakes. Take the first unstarted on either
track unless a better order is obvious, and say so. Numbers are stable;
the backlog is `docs/PARKED.md`, and an item returns only when a score names it.

487. **10245's eleven rows, all `1/27`** (the AI headline's own
    frame). `collide` 2/1, `collide_o` 29/-1, `collide_who` 1/-1,
    `g.stopped[0]` 1/0, `path:length` 49/43; the word's extra draw is
    ours `set_anim+0x97a < move_step+0x823`, the blocked stand,
    against theirs `Guy::inc_time+0x271`. 483's reading — **its
    hypothesis, not this item's** — is that `1/29` drops its move on
    10241 (`order:kind` 10/1) and stands 28 units short in `x`.
    Re-measure first. COMBAT §39.2.1.

476. **f10234's three value rows**: `0/5 order:length` 2/1,
    `0/5 orders.len` 2/1, `0/2001 gather:gather_down[-1]` 5/2. The
    word's own frame, value side. ORDERS §18.3.

477. **f10235's `1/28 pos`**, ours (4801,30175) theirs (4800,30175),
    with `g.x[0]` and `g.des_x[0]` the same — one world unit in x,
    said three times. ORDERS §18.3.

485. **`1/8`'s wound ladder from block 656** (the rules headline's
    own frame, under the word at 683). `damage` ours 0 theirs 8 at
    656, 8/17 at 657, 17/25 at 660, 25/34 at 682, then `extra 1/8` —
    the death — at 684. **We are one arrival late, not arithmetically
    wrong.** 484's reading, its hypothesis not this item's:
    `Object::take_damage` divides the squad's `myhits` by `uber_size`
    on the way in (§7.3) where `Sim::take_damage` uses `u.health`
    outright. COMBAT §40.6. 484 has landed; this is unblocked.

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
