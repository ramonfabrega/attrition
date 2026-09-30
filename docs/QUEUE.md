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

*2026-09-30, the commander after the twentieth pass: six landings.
**East Indies' second word 8907 → 10185** (1214, 1228); **chapter forty-three
opens at 1356, then 1552** (1223, 1235); **Great Sahara at Toughest
opens at 5376 of 15,432** (1221; run469's 1850/1850 is its test's, not a
floor). 1222: the held-out map, **Himalayas 1851/1850**, for the record.*

- **Three lanes live: 1243, 1241, 1248**, one to each open word.
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
- **Fable backlog: 19 Loop items** (1101, 1105, 1114, 1119, 1138, 1139, 1140, 1141, 1142, 1199, 1225, 1226, 1227, 1233, 1234, 1240, 1242, 1247, 1250).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 1850/1850 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w10185 of 18,140 · GreatLakes w5930 of 5,930
Third map: GreatSahara w24000 of 24,000 · GreatSaharaToughest w5376 of 15,432
Golden: ch43 w1552 of 2,201 · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · ch25 closed · ch26 closed · ch27 closed · ch28 closed · ch29 closed · ch30 closed · ch31 closed · ch32 closed · ch33 closed · ch34 closed · ch35 closed · ch36 closed · ch37 closed · ch38 closed · ch39 closed · ch40 closed · ch41 closed · ch42 closed
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked · GreatSahara 0 off, 0 unlinked

**Opener: the commander resumes — three lanes live on 1243, 1241 and
1248; six landings counted from the twentieth pass's commit.**

## The queue

In dependency order, headline-nearest first. **Three lanes, one to an open
word** (DECISIONS 41, 53, 55): the golden word for the rules, the third map's, and the newest pair's,
lower map first — East Indies (Great Lakes is closed at its end).
Take the first unstarted on a lane's own track unless a better order is obvious,
and say so; the backlog is `docs/PARKED.md`, back only when a score names it.

1243. **East Indies' second word: frame 10185, ours 9 draws against 10**
    (1228), at index 1: ours `Leader::make_stuff+0x221` where the
    original spends a second `Leader::use_market+0x1ed` (then
    `produce_building+0x1805` twice); widened on run462 (block 10186),
    which parts on 38 keys — who=1's `bucket[2:wealth]` 57 against 7, the
    Senate's foundation `1/2026` the dump's alone. `MAKE[0].val` on 10185
    1,200,000 against 4,800,000; `num_queued[84]` stands from 10178.

1241. **Great Sahara at Toughest's first word: frame 5376, ours 45
    draws against 40** (1221), at index 1: four
    `Leader::produce_building+0x1805` against one, five
    `Guy::set_anim+0x104b` against three; widened on run471 (5371..5627).
    Block 5377: who=1's new Granary site `1/2023` at ours (40608, 19680),
    theirs (41184, 15072). Standing from 5371: `SITE[i].reg` 1 against 0,
    who=1's `bucket[0:food]` 381 against 417. No mechanism is named.

1248. **Chapter forty-three's word: frame 1552, ours 31 draws against
    30** (1235), parting at index 27: ours who=1 `1/2`'s
    `Unit::do_non_flat_gather+0xcc3`, where the original draws `9/6`'s
    `Guy::set_anim+0x104b` and gathers on 1553; widened on run466 to
    1803. Block 1553: `1/2`'s `order:gather.wait` 324 against −1. The
    item before's hypothesis: the dead Citizen `1/6` left on `1/2001`'s
    gather list (`gather_down[-1]` 6 against 2 from 1488).

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
