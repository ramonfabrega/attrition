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

*2026-09-23, an Opus 5.5 commander after the eleventh pass: 17 landings,
15 of them moved a word. East Indies **9711 → 10982**; the rules track
closed chapters four (1500), seven (1200) and three (900), and its
restage stands at 782. Great Lakes held at 12038 throughout.*

- **Two bookings were wrong about the disk or the design**: 573's capture
  was already run99 (Loop 575), and chapters seven and three were staged
  so their falsifiers could not fire (Loop 584, both instances).
- **Chapter seven's premise fell** (INPUT §11.9): a human's Citizen
  gathers under `!ai off`. The commander pinned it closed on its
  agreement; restaging is the pass's.
- **592 and 597 moved no word**; 604 then did, 10582 → 10782, from the
  templates 597's packet named, and 608 → 10982 by a second packet.
- **Fable backlog: 10 Loop items** (313, 527, 574, 575, 583, 584, 596, 598, 605, 612).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w10982 of 24,000 · GreatLakes w12038 of 24,000
Golden: ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch7 closed · restage w782 of 1000 · 621 next
Endpoint 24001: EastIndies 64 off, 0 unlinked · GreatLakes 45 off, 0 unlinked

**Opener: 18 landings since the eleventh pass (573 to 616 by the log).
att-613 is live on the AI track. A commander takes 621 on the rules
track, then 571; workers spawn with `claude-opus-5-5[1m]`. The count is
by the log from the pass and stops at twenty.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden word for the rules, the long word for the AI, and
lower map first — East Indies (a guard reads this line). Take the first
unstarted on either track unless a better order is obvious, and say so; the
backlog is `docs/PARKED.md`, and an item returns only when a score names it.

613. **East Indies' word is 10982, past every capture** (608 moved it
    10782 → 10982: `City::count_gather_slots` counts unfinished gather
    buildings, AI §61, by run149 and run150's packet). On 10982 the
    original places a gather building: 680 `find_gather_tiles+0x10a`, and
    `produce_building` 4×`+0xc99`, 4×`+0x1805`, against ours 7× and 1×.
    run149 ends at 10879, so the `WIDENINGS` row names this item: capture
    **run152** over the word, overlapping run149, then widen it whole.

571. **Army 1's squad stops on block 11922 in the original and walks on
    here** (566 moved the word 11903 → 12038 by carrying the pathfinder's
    validity memo across searches, PATHFINDER §24). Nine figures, `1/37`–
    `1/42` and `1/62`–`1/64`: `stopped 1`, speed 0 against walking;
    positions agree, no draw; 29 rows pinned in run136's widening. Then
    12038's capture and widening past 11959: ours 4 draws against 5,
    `move_step+0x823`. No mechanism. If PRs #7/#8 land first, a packet at
    11921 with the nine guys' move path run on it is an optional second run.

621. **Chapter three's restage word is 782: an `ATTACKGROUNDORDER`**
    (616 moved run146 780 → 782: `SpellType::cast_unpack` lights the whole
    fog disc, COMBAT §56). On 782 ours 7 draws against 5, parting at draw
    0; values part on 781 on `0/6` alone. The original pushes an
    `ATTACKGROUNDORDER` (`fight`'s siege arm and `do_attack_ground`,
    §56.3), which this crate does not carry, and **the harness needs a
    reader for that order first**. The rules headline. No mechanism.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch, headline
  first and whether it moved; delete finished items — their story is in
  `docs/journal/`. Shorten only what you touch; never widen a line to beat a count.
- **A residue item is booked by its frame and its draw delta** (DECISIONS
  42); a mechanism in its title is the previous item's hypothesis.
- **A finding parks by default.** `docs/PARKED.md` takes what names no
  score; this file takes only what names a headline's frame, a floor, or
  a takes-chain to one. **Neither headline slot is ever empty**: no item
  on the AI word's frame means the widening of that frame; no rules item
  means the next chapter without a capture (GOLDEN §14; the run number is
  minted at booking); **a worker parks what names the headline's frame
  and the commander books it at the merge.**
- **A worker never books a number and never edits this file or
  `docs/JOURNAL.md`.** It reports; the commander books and writes these
  lines; the story is `docs/journal/<date>-item-<N>.md`. **A pinned
  constant is the worker's to re-pin; the line is the commander's.** The
  chain — merge, book, gate, push, reap, spawn — is `CLAUDE.md`'s.
- **The floors and these lines move together** — `FLOORS`, `LONG_WORD_*`,
  `GOLDEN_WORD_*`, `ENDPOINTS` with `Scoreboard:`, `Long captures:`,
  `Golden:` (a closed chapter reads `chN closed`), `Endpoint <frame>:`,
  `Fable backlog: N Loop items` and `lower map first — <map>` — read
  literally and **never wrapped**, or the count guard matches this line
  instead (twice, 09-19). The length guard counts every line to the next `## `.
- **A capture is a draw-stream trace first, detail on demand** (DECISIONS
  41): whole length at `cover=0`, then a windowed re-run sized to the word;
  after 363, the click-free lane unless it needs the mouse. Overlap the
  neighbours: six blocks each end (run83); `samegame.py` exits 0 when
  nothing is in common, so assert the count, not the verdict; **a wait
  keys on the runner's exit, never on a file** (565). **A value question
  takes a packet at the frame before, not a detail capture** (EMULATOR §8).
- **Grep for the derived quantity, the SHAPE and the GATE**: a term zero in
  every dump is zero-valued or switched off (117, 178); **a never-cleared
  field asserts a CHANGE, not a value** (`collide_frame`, run85).
- **The gate is `python3 tools/release_gate.py <install> --test-threads 4`**,
  to a file, never piped — a pipe launders the 137. A clear is free when
  every landed branch is merged, gated, pushed and reaped and none is in flight.
