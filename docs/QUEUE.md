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

*2026-09-28, the eighteenth Fable pass (`docs/audit/2026-09-28-fable-pass-18.md`,
DECISIONS 54): **no score moved in the pass, and none was meant to.**
The AI's word is the second pair's, at the lobby's top difficulty, and
it is in the first war; the first pair stays closed at 24,000,
endpoints 0 off. Every chapter is closed, thirty-five at 3260
(1050). Two lanes run; the count is at three (1061, 1050, 1066).*

- **The tranche, ruled**: twenty landings, 36.6 USD each. The war moves
  Great Lakes forty-one frames and one rule of combat a landing, and
  its game ends on 5,930: **the pass reads landings to that end**.
- **The instrument follows the word** (1061, GROUPS §33): the second
  pair's widening compares the group record and the attack row, and
  the compared pin walks the pair's Great Lakes word's window.
- **A third map is scored** (1066, AI §83): Great Sahara, all land,
  its word 8; no item opens on it until a pass says so (DECISIONS 54 §3).
- **The user's**: whether phase 4 opens on the rules track alone. The
  disk is not: 73 GB free, after 55 GB of debug build.
- **Fable backlog: 14 Loop items** (677, 685, 894, 900, 927, 953, 954, 960, 973, 1067, 1071, 1076, 1079, 1080).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 6/5 w8
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w5606 of 18,140 · GreatLakes w4978 of 5,930
Third map: GreatSahara w8 of 24,000
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · ch25 closed · ch26 closed · ch27 closed · ch28 closed · ch29 closed · ch30 closed · ch31 closed · ch32 closed · ch33 closed · ch34 closed · ch35 closed · 1077 next
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked

**Opener: the commander resumes — 1072 and 1077 run; East Indies'
5606 is booked when either frees.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41, 53): the golden word for the rules, the newest pair's word for the AI,
lower map first — Great Lakes (a guard reads this line). Take the first
unstarted on either track unless a better order is obvious, and say so; the
backlog is `docs/PARKED.md`, and an item returns only when a score names it.

1072. **Great Lakes' second word: frame 4978, ours 9 draws against 8**
    (1061), parting at index 1: ours `Unit::close+0xcb6`, theirs
    `Farms::inc_time+0x1ae`. Widened on run373 (block 4979): citizen
    `0/2` dies on ours alone, damage 42 against 37, the gap standing
    from 4841; the original's falls a point on 4859 and 4904 and ours
    does not. No mechanism is named. East Indies' 5606 follows.

1077. **Chapter thirty-five's floor: the V2's blast on 2821** (1050
    closed the chapter at 3260, PRODUCTION "The missile's launch and
    round"). Value-only, no draw parts: the dump's Barracks `1/2006` is
    gone on 2821, ours stands at 400 damage of 1200. `Ammo::do_damage`'s
    missile arm is read to its shield only; what destroys the building
    is `Object::do_damage`'s. `fight.rs`, `combat.rs`.

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
