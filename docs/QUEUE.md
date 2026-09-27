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

*2026-09-27, the commander after the seventeenth pass (DECISIONS 53):
**the AI's word is the second pair's** — run54's and run53's games at the
lobby's top difficulty, Toughest, both ending in the idle human's defeat.
The first pair stays closed at 24,000 on both maps, endpoints 0 off;
chapters one to thirty-two are closed. The blind list is counter two, on
the staging lane, and not a headline.*

- **Since the pass** (count from 0b401602: **4**): 947 closed chapter
  thirty-two; 971 took the second pair and built its first coin; 972
  measured the held-out map once, **1 and 0**, so the next pass books a
  third scored map; 979 found the harness borrowing an Easiest game's
  frame words and gated it on `GAMEINFO`: **East Indies 10 → 1576, Great
  Lakes 1 → 3776**. 965 is live.
- **The user's**: the archive's disk has **20 GB free** and the tranche
  wrote 10.2 GB; and whether phase 4 opens on the rules track alone.
- **Fable backlog: 22 Loop items** (677, 685, 745, 775, 799, 838, 852, 894, 900, 927, 953, 954, 960, 973, 974, 975, 983, 984, 985, 986, 987, 992).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w1576 of 18,140 · GreatLakes w3776 of 5,930
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · ch25 closed · ch26 closed · ch27 closed · ch28 closed · ch29 closed · ch30 closed · ch31 closed · ch32 closed · 965 next
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked

**Opener: the commander, two lanes — 989 on the AI lane (East Indies'
1576, the second pair's lower word) and 965 on the rules lane; after 965
the rules lane takes the next chapter without a capture (GOLDEN §13).
The count runs from 0b401602; the eighteenth pass at twenty.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41, 53): the golden word for the rules, the newest pair's word for the AI,
lower map first — East Indies (a guard reads this line). Take the first
unstarted on either track unless a better order is obvious, and say so; the
backlog is `docs/PARKED.md`, and an item returns only when a score names it.

989. **East Indies' second word: frame 1576, ours 272 draws against 216**
    (979). Parting at index 192: ours `Build::find_gather_tiles+0x10a`,
    theirs `Animal::think_bird+0x82`, run346 at Toughest; widened on
    run352. Great Lakes' 3776 parts on the same two sites (223 against
    217 at index 180, widened on run355). That the two are one cause is
    979's reading, a hypothesis.

965. **The blind list's row 6, its last gate** (959): `new_danger` and
    `restart_trade_route` are one gate in `Object::take_damage`
    (00652e97..00652f41). run342's Tower hit the caravan on London's
    ground and entered neither; the unowned or enemy-ground arm is
    untried. What `do_damage` and the arrow path pass as arguments 7 and
    8, under the emulator first (CARAVAN §10.4); a staged run on the queue
    lane only if it cannot say.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch, headline
  first and whether it moved; delete finished items — their story is in
  `docs/journal/`. Shorten only what you touch; never widen a line to beat a count.
- **A residue item is booked by its frame and its draw delta** (DECISIONS
  42); a mechanism in its title is the previous item's hypothesis, not a build order.
- **A finding parks by default.** `docs/PARKED.md` takes what names no
  score; this file takes only what names a headline's frame, a floor, or
  a takes-chain to one. **Neither headline slot is ever empty**: no item
  on the AI word's frame means the widening of that frame; no rules item
  means the next chapter without a capture (GOLDEN §13, §14; the run
  number is minted at booking); **a worker parks what names the
  headline's frame and the commander books it at the merge.**
- **A worker never books a number and never edits this file or
  `docs/JOURNAL.md`.** It reports; the commander books and writes these
  lines; the story is `docs/journal/<date>-item-<N>.md`. **A pinned
  constant is the worker's to re-pin; the line is the commander's.** The
  chain — merge, book, gate, push, reap, spawn — is `CLAUDE.md`'s.
- **The floors and these lines move together** — `FLOORS`, `LONG_WORD_*`,
  `GOLDEN_WORD_*`, `ENDPOINTS` with `Scoreboard:`, `Long captures:`,
  `Golden:` (`<name> closed`, or `every chapter closed` first), `Endpoint <frame>:`,
  `Fable backlog: N Loop items` and `lower map first — <map>` — read
  literally and **never wrapped**, or the count guard matches this line
  instead (twice, 09-19). The length guard counts every line to the next `## `.
- **A capture is a draw-stream trace first, detail on demand** (DECISIONS
  41): whole length at `cover=0`, then a windowed re-run sized to the word;
  after 363, the click-free lane unless it needs the mouse. Overlap the
  neighbours: six blocks each end (run83); `samegame.py` exits 0 when
  nothing is in common, so assert the count, not the verdict; **a detached
  capture waits on `tools/gamelog/waitrun.sh`** (565, 656). **A value question
  takes a packet at the word's own frame, not a detail capture** (EMULATOR §8).
- **Grep for the derived quantity, the SHAPE and the GATE**: a term zero in
  every dump is zero-valued or switched off (117, 178); **a never-cleared
  field asserts a CHANGE, not a value** (`collide_frame`, run85).
- **The gate is `python3 tools/release_gate.py <install> --test-threads 4`**,
  to a file, never piped — a pipe launders the 137 — ending `Gate steps:`;
  **a worker's takes `--lane`** and its landing quotes `Lane verdict:` (969).
