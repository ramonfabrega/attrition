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

*2026-09-30, the commander after the twentieth pass: nineteen landings.
**East Indies 8907 → 11549**; **Great Sahara at Toughest opens at 5376,
now 8786**; **chapters forty-three to forty-five opened and closed**
(2200, 1450, 1650); **forty-six opens at 1902**. 1222: the held-out map, 1851/1850, for the record.*

- **Three lanes live: 1297, 1293, 1291**, one to each open word.
- **The commander clears at the seam after every tenth landing**; the
  twentieth is the pass's.
- **A brief is `python3 tools/brief.py <item> --kind residue|chapter`**
  and a note of what the item alone needs.
- **Two lanes that touch `coverage.rs` conflict at the merge**: the
  second takes `ccc update` and gates again (three this run).
- **A chapter is three arms of one staging at most**, from the arms a
  landing parks as held by no walk (DECISIONS 56 §3).
- **The user's**: whether phase 4 opens on the rules track alone; **the
  disk, 46 GiB free**; parked 1141 and 1142.
- **Fable backlog: 30 Loop items** (1101, 1105, 1114, 1119, 1138, 1139, 1140, 1141, 1142, 1199, 1225, 1226, 1227, 1233, 1234, 1240, 1242, 1247, 1250, 1253, 1256, 1259, 1263, 1267, 1273, 1274, 1285, 1289, 1296, 1301).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 1850/1850 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w11549 of 18,140 · GreatLakes w5930 of 5,930
Third map: GreatSahara w24000 of 24,000 · GreatSaharaToughest w8786 of 15,432
Golden: ch46 w1902 of 2,201 · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · ch25 closed · ch26 closed · ch27 closed · ch28 closed · ch29 closed · ch30 closed · ch31 closed · ch32 closed · ch33 closed · ch34 closed · ch35 closed · ch36 closed · ch37 closed · ch38 closed · ch39 closed · ch40 closed · ch41 closed · ch42 closed · ch43 closed · ch44 closed · ch45 closed
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked · GreatSahara 0 off, 0 unlinked

**Opener: the commander resumes — three lanes live on 1297, 1293 and
1291; nineteen landings counted from the twentieth pass's commit.**

## The queue

In dependency order, headline-nearest first. **Three lanes, one to an open
word** (DECISIONS 41, 53, 55): the golden word for the rules, the third map's, and the newest pair's,
lower map first — East Indies (Great Lakes is closed at its end).
Take the first unstarted on a lane's own track unless a better order is obvious,
and say so; the backlog is `docs/PARKED.md`, back only when a score names it.

1297. **East Indies' second word: frame 11549, ours 11 draws against 4**
    (1281), at index 0: ours `Unit::think_scout+0x941` (then six
    `+0xaba`, all `1/35`'s) where the original spends `Guy::set_anim
    +0x97a < Unit::do_guard+0x7f4`; widened on run490 (block 11550):
    `1/35`'s `order:kind` ours 3 against 2, and its path. Standing from
    11323: who=1's `scouts` 0 against 3. No mechanism is named.

1293. **Great Sahara at Toughest's word: frame 8786, ours 10 draws
    against 49** (1286), at index 1: the original spends
    `Unit::do_group_move+0xb03 < Unit::do_group_attack_to+0x11` where
    ours spends a second `Guy::set_anim+0x97a < Unit::move_step+0x823`;
    widened on run500 (block 8787), which parts on no key first. Standing
    from 8781: who=1's `order:group.id` on `1/73`..`1/75` 8698116 against
    8704516, a dozen units' orders parted in 8434..8780. No mechanism.

1291. **Chapter forty-six's word: frame 1902, ours 3 draws against 2**
    (1278), at index 0: ours `0/8`'s `Guy::set_anim+0x97a <
    Guy::move+0x19f`, theirs `Farms::inc_time+0x1ae`; run496. Block
    1902: the original's three riders hold one `GroupMoveOrder` each
    (group 1901300), ours 3, 2 and 1; `0/9` at x 14856 against 14904.
    The squad's landing (`disembark_squad`). No mechanism is named.

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
