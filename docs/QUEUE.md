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

*2026-09-30, the twenty-first Fable pass (`docs/audit/2026-09-30-fable-pass-21.md`,
DECISIONS 57): **the count starts at zero from its commit.** The tranche
before: East Indies 8907 → 11637, Great Sahara at Toughest 5376 → 8856,
Himalayas 1851/1850 held out and in lockstep, chapters forty-three to
forty-six closed; 13.7 USD a landing, 142 minutes of which 108 waiting.*

- **Lanes live: 1326, 1330, 1338.** Landed: 1305, 1318, 1332 (Toughest
  → 9764), 1310, 1323 (ch47 → 838), 1302 (East Indies → 12582).
- **The pass built**: the lane held through the click-free runner's
  restore (1234), `seams.py` reading comments, `SEAMS:` blocks and lists
  (1253, 1240), `standing.py` reading a chapter's firsts (1250), the
  frame's ceiling (1226), `tools/tranche.py` for a tranche's waiting.
- **Two lanes lost 258 minutes to a classifier outage and the stop
  rule** (parked 1315): a `no verdict (error)` is backed off on a
  Monitor, not stopped on — `CLAUDE.md` and the frame say so now; **the arm of `ccc clear --then` is the turn's last act and the
  turn ends** — the last commander cancelled its own three (1314).
- **The user's**: the classifier's refusals (1315); phase 4 on the rules
  track alone; **the disk, 40 GiB free, 13 written a tranche**; parked
  1141 and 1142.
- **Fable backlog: 28 Loop items** (1105, 1119, 1138, 1139, 1141, 1142, 1199, 1225, 1242, 1247, 1259, 1263, 1273, 1274, 1296, 1301, 1308, 1309, 1314, 1315, 1316, 1317, 1322, 1325, 1329, 1331, 1337, 1340).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 1850/1850 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w12582 of 18,140 · GreatLakes w5930 of 5,930
Third map: GreatSahara w24000 of 24,000 · GreatSaharaToughest w9764 of 15,432
Golden: ch47 w838 of 1,401 · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · ch25 closed · ch26 closed · ch27 closed · ch28 closed · ch29 closed · ch30 closed · ch31 closed · ch32 closed · ch33 closed · ch34 closed · ch35 closed · ch36 closed · ch37 closed · ch38 closed · ch39 closed · ch40 closed · ch41 closed · ch42 closed · ch43 closed · ch44 closed · ch45 closed · ch46 closed
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked · GreatSahara 0 off, 0 unlinked

**Opener: the commander, on Opus — lanes live on 1326, 1330 and 1338;
merge each landing and refill its lane; the count is at six.**

## The queue

In dependency order, headline-nearest first. **Three lanes, one to an open
word** (DECISIONS 41, 53, 55): the golden word for the rules, the third map's, and the newest pair's,
lower map first — East Indies (Great Lakes is closed at its end).
Take the first unstarted on a lane's own track unless a better order is obvious,
and say so; the backlog is `docs/PARKED.md`, back only when a score names it.

1326. **East Indies' third word: frame 12582, ours 9 draws against 10**
    (1302), at index 4: the original spends `Leader::make_stuff+0x63d`,
    the expire-slot roll; widened on run508 (block 12583): block 12581
    holds who=1's `MAKE[5]` Citizen order (`t` 50, `num` 4, `city` 2)
    the dump's alone; 1,817 keys part. No mechanism is named.

1338. **Great Sahara at Toughest's word: frame 9764, ours 10 draws
    against 11** (1332), at index 4: the original spends a fourth
    `Guy::set_anim+0x97a < Unit::set_anim+0xb6 < Unit::do_idle+0x7d` on
    the Catapult `1/84` (seed `c84aaf40`); widened on run529 (block
    9765). No mechanism is named.

1330. **Chapter forty-seven's word: frame 838, ours 6 draws against 7**
    (1323), at index 1: the original spends `Guy::set_anim+0x97a <
    Unit::do_idle+0x7d` (seed `0x3de49d86`); on block 838 `0/9`'s
    `orders.len` ours 1 against 0, `orders_x/y` (3864, 36888) against
    (3840, 36864). No mechanism is named.

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
