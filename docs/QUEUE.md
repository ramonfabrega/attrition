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

- **Lanes live: 1302, 1318, 1323.** Landed: 1305 (Great Sahara at
  Toughest 8856 → 9323), 1310 (chapter forty-seven opens at 712).
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
- **Fable backlog: 24 Loop items** (1105, 1119, 1138, 1139, 1141, 1142, 1199, 1225, 1242, 1247, 1259, 1263, 1273, 1274, 1296, 1301, 1308, 1309, 1314, 1315, 1316, 1317, 1322, 1325).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 1850/1850 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w11637 of 18,140 · GreatLakes w5930 of 5,930
Third map: GreatSahara w24000 of 24,000 · GreatSaharaToughest w9323 of 15,432
Golden: ch47 w712 of 1,401 · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · ch25 closed · ch26 closed · ch27 closed · ch28 closed · ch29 closed · ch30 closed · ch31 closed · ch32 closed · ch33 closed · ch34 closed · ch35 closed · ch36 closed · ch37 closed · ch38 closed · ch39 closed · ch40 closed · ch41 closed · ch42 closed · ch43 closed · ch44 closed · ch45 closed · ch46 closed
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked · GreatSahara 0 off, 0 unlinked

**Opener: the commander, on Opus — lanes live on 1302, 1318 and 1323;
merge each landing and refill its lane; the count is at two.**

## The queue

In dependency order, headline-nearest first. **Three lanes, one to an open
word** (DECISIONS 41, 53, 55): the golden word for the rules, the third map's, and the newest pair's,
lower map first — East Indies (Great Lakes is closed at its end).
Take the first unstarted on a lane's own track unless a better order is obvious,
and say so; the backlog is `docs/PARKED.md`, back only when a score names it.

1302. **East Indies' second word: frame 11637, ours 2 draws against 38**
    (1297), at index 1: the original spends eighteen `Guy::init_real
    +0x52 < Unit::init+0xb97 < Objects::init_unit+0xbd` and seventeen
    idle stands; widened on run506 (block 11638): who=1's eighteen
    Peltasts `1/93`, `1/104`..`1/120` are the dump's alone, and group 70
    lists 37 against 19. No mechanism is named.

1318. **Great Sahara at Toughest's word: frame 9323, ours 7 draws
    against 5** (1305), at index 1: ours spends the AI scout `1/0`'s two
    `Guy::set_anim+0x97a < Unit::move_step+0x823`, the original
    `Guy::set_anim+0x97a < Guy::inc_time+0x271`; widened on run511
    (block 9324): 571 keys stand on 9318 from the gap 9038..9317, 28 of
    who=1's army positions among them. No mechanism is named.

1323. **Chapter forty-seven's word: frame 712, ours 35 draws against
    31** (1310), at index 25: ours spends four draws in the buildings
    phase — T `0/2008`'s round while it is a site — where the original's
    next is `Farms::inc_time+0x1ae` (seed `0x8fad03fd`); widened on
    run514 (block 713): `1/6`'s `hits:damage` ours 8 against 0,
    `hits_left` 112 against 120. No mechanism is named.

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
