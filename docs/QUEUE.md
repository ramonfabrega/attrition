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

*2026-09-18/19, two lanes all day, twelve landings. **Great Lakes 9182 →
9451 (+269), golden 617 → 618** — both words moved. Nine booked mechanisms
were wrong, the frames right every time.*

- **An arrow is launched by the attack ANIMATION's event track**, not by
  `Unit::fight` (389, `docs/COMBAT.md` §9.0) — a **data** coupling, not a
  renderer one, and the document says so. `ms × 3 / 200` **truncated**,
  measured not chosen: every product lands at remainder 198/199 of 200, so
  truncation is 4/4 against run53 and rounding 0/4. 9415 → 9451. **394.**
- **The met bit is set by the fog** (385) closed a chain six items long
  and moved the word 9182 → 9415: `MAKE[0].val` → `MAKE[0].t` →
  `use_market`'s `need`, `active_wars` behind `treaties[i] & 1`.
- **The golden word moves 617 → 618** (391): `Unit::target_opportunity`
  walks `o_up` to the victim's **captain**, and the ARMY — never the
  group — walks a struck unit off its seat. 615–617 are the original's;
  **392** has 618. `0xf6` uncoded since the second reading (356) and
  `in_range` inverted were one constant at two sites; §18's unproven ~84
  extent was **refused on measurement**, so keeping the fit out was right.
- **The namesake has never fired in a scored capture** (382): zero
  non-exempt attrition outcomes over 24,000 frames — the Temple chapter is
  the only route. **The held-out map answers 1 tick, 0 orders** (run106),
  a fit to two trajectories: **the pass's number**. Six open, 46 parked; **Fable backlog: ten Loop items** (251, 335, 343, 356, 321, 313, 367, 374, 375, 377).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w9711 of 24,000 · GreatLakes w9451 of 24,000
Golden: w618 of 901 · chapter one pinned · 392 next
Endpoint 24001: EastIndies 64 off, 5 unlinked · GreatLakes 43 off, 13 unlinked

**Opener: 394 on the word's new frame — the first arrow landing at 9451,
which no document explains — beside 392, in flight on the golden 618.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden record's word for the rules, the long captures' word for
the AI, lower map first — Great Lakes. Take the first unstarted on either
track unless a better order is obvious, and say so. Numbers are stable;
the backlog is `docs/PARKED.md`, and an item returns only when a score names it.

394. **9451 is the first arrow landing** (the AI headline's frame, where
    389 just moved the word): five draws against seven, the original
    spending two at `Object::take_damage@00652020+0xe1`. **The document
    does not explain this frame** — `docs/COMBAT.md` §9.5's reading (a
    fort/temple/town's first wound, a siege attacker's flock) covers
    neither `0/2004` nor `1/29`. run100's `BUILDDATA 0/2004` is `damage 0`
    through 9451 and `damage 1` at 9452: the game's first hit point lost.

390. **A `LEADERS≥2` window anywhere in (7616, 8174]** (capture lane;
    takes-chain to the met bit): nothing on disk dumps a leader between
    7600 and 8174, so this crate's CONTACT flip frame **7944 is checked
    against a bracket, not a frame**. The cheapest capture left on the
    mechanism 385 just landed. Assert the block count against the window
    asked for — run97's truncation is why (373).

392. **§17's ring is what stands at 618** (the rules headline's frame,
    where 391 just moved the word): one draw, `0/6`'s second
    `Unit::fight+0x9b0`. The original spends 617 in `docs/COMBAT.md` §17's
    ring — `find_attack_pos` → (1080,8280) — and walks a five-node chase
    from 618; ours answers nothing, falls back to the target's own point
    and loses the move inside the frame.

365. **`docs/GOLDEN.md`, the script itself** (a reading and a design, Opus):
    chapters from `docs/CENSUS.md`'s order family and `docs/COMMANDS.md`
    §3 — every order class, each unit line, an age jump between chapters,
    the Temple chapter for attrition, a war — each chapter naming the
    records that would falsify it. **Two constraints from 364**: the bare
    `war` form is a no-op (it prints the diplomacy table; chapter one's
    squads engage because a Quick Battle already starts at war), and `age`
    leaves all four epochs Ancient, so a late-age chapter wants `library`.

370. **Type the ladder's twenty-five extras** (names rung C's pinned
    `extra` floor, 13→25 under 362's batch fix; rung B +13 the same way,
    compared/off/unlinked unmoved). All scholars and caravans means the
    batch overshoots where the word is long past; a mixed bag is sibling
    divergence on borrowed captures 7,590 and 8,700 frames past the word.
    **No dump can answer it** — an `extra` is ours and not theirs — so
    `docs/journal/2026-09-18-item-362.md` names the one line inside
    `the_east_indies_ladder_is_pinned` that types all twenty-five.

342. **Host choice seats the scholar on the wrong university** — 338's
    residue, re-pinned three times, and now `1/55`. **Re-measure before
    diagnosing**: the vector `(768, 9984)` is the claim, never the count.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch, headline
  first and whether it moved; delete finished items — their story is in
  `docs/journal/`. Shorten only what you are touching, never another
  author's item, and never widen a line to beat a count.
- **A finding parks by default.** `docs/PARKED.md` takes what names no
  score; this file takes only what names a headline's frame, a floor, or
  a takes-chain to one. **Neither headline slot is ever empty**: no item
  on the AI word's frame means the widening of that frame; no rules item
  means the next unpinned chapter of the golden record. Loop items —
  tooling, guards, these rules — are the steering pass's, never a worker's.
- **A worker never books a number and never edits this file or
  `docs/JOURNAL.md`.** It reports; the commander books and writes the
  `Scoreboard:` line; the worker's story is `docs/journal/<date>-item-<N>.md`.
- **A number is measured on the tip** — after the worker's last `ccc
  update`, or the report says which tree. 301's +381 was attributed to the
  wrong half against a stale base and would have landed wired.
- **Merge, gate, push, reap — one chain.** A reap left for "before the
  next spawn" was forgotten twice in one session.
- **A floor that moves** moves `FLOORS`, the assert reading it, and the
  `Scoreboard:` line together; the guard parses the line against the floors.
  The `Golden:` line is the same shape for the rules track.
- **A capture is a draw-stream trace first, detail on demand** (DECISIONS
  41): whole length at `cover=0`, then a windowed re-run sized to the word;
  after 363, the click-free lane unless it needs the mouse. Overlap the
  neighbours on purpose: six blocks each end (run83); `samegame.py` exits
  0 when nothing is in common, so assert the count, not the verdict.
- **Grep for the derived quantity, the SHAPE and the GATE — not only for
  coverage of a frame.** A term zero in every dump is either zero-valued or
  switched off: 117 and 178 were the second.
- **A check on a never-cleared field asserts a CHANGE, not a value** —
  `collide_frame` is a permanent stamp (run85). Test both directions first.
- **The gate is `python3 tools/release_gate.py <install> --test-threads 4`**,
  to a file, never piped — a pipe launders the 137. A commander's clear is
  free when every landed branch is merged, gated, pushed and reaped,
  nothing is in flight, and both headlines are measured. **Before a blind
  fan-out**, grep `CLAUDE.md` and the memory index: a subagent inherits both.
