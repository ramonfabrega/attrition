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

*2026-09-30, fourteen landings since the twenty-first
Fable pass (the pass commit `d4aeabbe`, DECISIONS 57). East Indies 11637 →
15862 (1302, 1326, 1341, 1351), Great Sahara at Toughest 8856 → 10144 (1305,
1318, 1332, 1338, 1346, 1354), chapter forty-seven opened at 712 and closed at 1400
(1310, 1323, 1330, 1350). Every booking gate green; one went to origin red
before its exit was read (Loop 1345), fixed by `5098706e`.*

- **Lanes live: 1358, 1362, 1365**, one to a word, each cut off a
  booking commit; `ccc clear --then` was consumed unfired twice more.
- **Three Loop items share one shape** (1337, 1340 with three reaches,
  1318/1332/1338): the answer was a seam on the first parted field's
  writer, not on the word's chain — a checklist row is due.
- **The user's**: the classifier's refusals (1315); phase 4 on the rules
  track alone; the disk, 217 GiB free; parked 1141 and 1142.
- **Fable backlog: 36 Loop items** (1105, 1119, 1138, 1139, 1141, 1142, 1199, 1225, 1242, 1247, 1259, 1263, 1273, 1274, 1296, 1301, 1308, 1309, 1314, 1315, 1316, 1317, 1322, 1325, 1329, 1331, 1337, 1340, 1344, 1345, 1349, 1353, 1357, 1361, 1364, 1369).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 1850/1850 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w15862 of 18,140 · GreatLakes w5930 of 5,930
Third map: GreatSahara w24000 of 24,000 · GreatSaharaToughest w10144 of 15,432
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · ch25 closed · ch26 closed · ch27 closed · ch28 closed · ch29 closed · ch30 closed · ch31 closed · ch32 closed · ch33 closed · ch34 closed · ch35 closed · ch36 closed · ch37 closed · ch38 closed · ch39 closed · ch40 closed · ch41 closed · ch42 closed · ch43 closed · ch44 closed · ch45 closed · ch46 closed · ch47 closed
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked · GreatSahara 0 off, 0 unlinked

**Opener: the commander, on Opus — lanes live on 1358, 1362 and 1365;
merge each landing and refill its lane; the count is at fourteen.**

## The queue

In dependency order, headline-nearest first. **Three lanes, one to an open
word** (DECISIONS 41, 53, 55): the golden word for the rules, the third map's, and the newest pair's,
lower map first — East Indies (Great Lakes is closed at its end).
Take the first unstarted on a lane's own track unless a better order is obvious,
and say so; the backlog is `docs/PARKED.md`, back only when a score names it.

1362. **East Indies' sixth word: frame 15862, ours 11 draws against 7**
    (1351), at index 0: ours spends four `Army::find_target+0x7df`
    before the original's first, `Guy::set_anim+0x97a <
    Animal::do_idle+0x19` (seed `5faaa95a`); widened on run544 (block
    15863): who=1's group 70 on 15859 (the original's fifteen slots and
    ours none), group 65's `held` ours alone; on 15863 `1/30` and four
    more hold an order there and none here. No mechanism is named.

1365. **Great Sahara at Toughest's word: frame 10144, ours 42 draws
    against 42** (1354), at index 33: the original spends
    `Guy::init_real+0x52 < Unit::init+0xb97 < Objects::init_unit+0xbd`
    (seed `8530fb47`) — `1/89`'s birth from `1/2022`, ours still queued
    — where ours spends `1/44`'s `Guy::set_anim+0x97a <
    Guy::inc_time+0x271`; widened on run547 (block 10145). No mechanism
    is named.

1358. **Chapter forty-eight: three arms the unit tests or a widening
    hold and no walk** (DECISIONS 56 §3), 1350's M2–M4: the carry in a
    site's lost progress (the widening holds it; the walk's site dies on
    902 against 901 with no draw moved); an aircraft striking a site;
    and building attrition on a site (`docs/CITIES.md` §9.5). Whether
    one staging reaches each is the item's to establish first.

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
