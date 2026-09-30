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

*2026-09-30, the commander after the twentieth pass: three landings.
**East Indies' second word 8907 → 10183** (1214); **chapter forty-three
opens at 1356** (1223, a squad boarding its captain's barge); 1222
measured the held-out map: **Himalayas 1851/1850**, for the record only.*

- **Three lanes live: 1228, 1221, 1235.** 1228 reports 10185 and 1221
  Great Sahara at Toughest's first word 5376, neither landed.
- **The commander clears at the seam after every tenth landing**, the
  handoff written first.
- **A brief is `python3 tools/brief.py <item> --kind residue|chapter`**
  and a note of what the item alone needs: never the sections or the
  other lanes' words, which the brief carries already.
- **The spawn follows the booking commit**; the gate runs beside it. **A
  gate that is running is not stopped for a base that moved.**
- **A chapter is three arms of one staging at most, booked by its
  functions**, from the arms a landing parks as held by no walk.
- **The user's**: whether phase 4 opens on the rules track alone; **the
  disk, 53 GiB free at 11.2 GB a tranche**; parked 1141 and 1142.
- **Fable backlog: 16 Loop items** (1101, 1105, 1114, 1119, 1138, 1139, 1140, 1141, 1142, 1199, 1225, 1226, 1227, 1233, 1234, 1240).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 1850/1850 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w10183 of 18,140 · GreatLakes w5930 of 5,930
Third map: GreatSahara w24000 of 24,000
Golden: ch43 w1356 of 2,201 · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · ch25 closed · ch26 closed · ch27 closed · ch28 closed · ch29 closed · ch30 closed · ch31 closed · ch32 closed · ch33 closed · ch34 closed · ch35 closed · ch36 closed · ch37 closed · ch38 closed · ch39 closed · ch40 closed · ch41 closed · ch42 closed
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked · GreatSahara 0 off, 0 unlinked

**Opener: the commander resumes — three lanes live on 1228, 1221 and
1235; three landings counted from the twentieth pass's commit.**

## The queue

In dependency order, headline-nearest first. **Three lanes, one to an open
word** (DECISIONS 41, 53, 55): the golden word for the rules, the third map's, and the newest pair's,
lower map first — East Indies (Great Lakes is closed at its end).
Take the first unstarted on a lane's own track unless a better order is obvious,
and say so; the backlog is `docs/PARKED.md`, back only when a score names it.

1228. **East Indies' second word: frame 10183, ours 24 draws against 9**
    (1214), at index 0: ours `Leader::create_units+0x642`, theirs
    `Guy::set_anim+0x97a < do_cast`, widened on run462 (block 10184),
    which parts on nine keys — who=1's `MAKE[0].t` (228 against 597)
    among them, the make list parting from 10181; 182 keys stand on
    block 10178, who=1's `num_units[268]` (0 against 1) among them. No
    mechanism is named.

1221. **Great Sahara at Toughest: the closed map in the second pair's
    lobby** (DECISIONS 56 §1): a `DUMP_ALL` start sibling, a dump of
    1,850 frames and a draw-stream trace at `cover=0` to 24,000 or the
    game's end, the difficulty read back from `GAME INFO`; its row joins
    `AI_WORDS` on the `Third map` line as `GreatSaharaToughest`, and its
    first word is pinned with its widening. No mechanism is named. The
    third lane's.

1235. **Chapter forty-three's word: frame 1356, ours 8 draws against 6**
    (1223), parting at index 2: ours `0/8`'s `Unit::do_move+0xe84` and
    `Guy::set_anim+0x97a < Guy::move+0x19f`, the original neither;
    widened on run466 (to 1607). Block 1357: `inside` of `0/8` and
    `0/9` ours −1 against 10, and `0/9` holds its move. The item
    before's hypothesis: `do_cast`'s captain check, a `SEAMS:` block.

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
  chain — merge, book, spawn, gate, push, reap — is `CLAUDE.md`'s.
- **The floors and these lines move together** — `FLOORS`, `LONG_WORD_*`,
  `GOLDEN_WORD_*`, `ENDPOINTS`, `AI_WORDS` with `Scoreboard:`, `Long captures:`,
  `Golden:` (`<name> closed`, or `every chapter closed` first), `Endpoint <frame>:`,
  `Second pair:`, `Third map:`, `Fable backlog: N Loop items` and `lower map first — <map>` — read
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
