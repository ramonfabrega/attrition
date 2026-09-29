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

*2026-09-29, six landings since the nineteenth pass (DECISIONS 55):
East Indies 6151 → 6743 (1127, 1143, 1156); Great Sahara 6/5 →
1850/1850, long word 8 → 13182 (1133, 1147); chapter thirty-eight
closed (1131).*

- **Three lanes, on the user's word** (DECISIONS 55, amended): the
  second pair's — 1164, East Indies' 6743; **the third map's** — 1163,
  Great Sahara's frame 13182; the rules' — 1111, chapter thirty-nine.
  One capture lane still; a lane that waits on it says for how long.
- **The commander clears at the seam after every tenth landing**, the
  handoff written first.
- **Code is not fenced** (DECISIONS 55 §5): a brief reserves run and
  section numbers and says where the other lanes' words sit. **A brief
  is `python3 tools/brief.py <item> --kind residue|chapter`** plus a note.
- **The spawn follows the booking commit**; the gate runs beside it.
- **A floor test reports every pin that moved** (`Pins::hold()`,
  `pin_eq!`); a widening `WIDENINGS` names is held to it.
- **The user's**: whether phase 4 opens on the rules track alone. The
  disk: 64 GB free.
- **Fable backlog: 18 Loop items** (685, 1101, 1105, 1114, 1119, 1138, 1139, 1140, 1141, 1142, 1146, 1150, 1151, 1155, 1159, 1161, 1162, 1166).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 1850/1850 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w6743 of 18,140 · GreatLakes w5930 of 5,930
Third map: GreatSahara w13182 of 24,000
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · ch25 closed · ch26 closed · ch27 closed · ch28 closed · ch29 closed · ch30 closed · ch31 closed · ch32 closed · ch33 closed · ch34 closed · ch35 closed · ch36 closed · ch37 closed · ch38 closed · 1111 next
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked

**Opener: the commander resumes on three lanes — 1164, 1163 and 1111;
the twentieth pass at twenty landings from the nineteenth's commit.**

## The queue

In dependency order, headline-nearest first. **Three lanes, one to an open
word** (DECISIONS 41, 53, 55): the golden word for the rules, the third map's, and the newest pair's,
lower map first — East Indies (Great Lakes is closed at its end).
Take the first unstarted on a lane's own track unless a better order is obvious,
and say so; the backlog is `docs/PARKED.md`, back only when a score names it.

1163. **Great Sahara's word: frame 13182, ours 9 draws against 8**
    (1147), at index 3: ours `Leader::make_stuff+0x63d`, theirs
    `GameAccess::rnd+0x20 < Unit::do_job+0x67`, widened on run417 (blocks
    13177..13433), where the make list parts on 13181; between the
    captures, run416's 12784 (`1/2014`'s Militia at food and metal 16
    short) and 12945 (the AI citizens' `myhits`, `mylos`). Its own lane.
    No mechanism is named.

1164. **East Indies' second word: frame 6743, ours 4 draws against 8**
    (1156), at index 2: ours `Guy::set_anim+0x104b`, theirs
    `Guy::set_anim+0x97a < Guy::inc_time+0x271`, widened on run420
    (block 6744), where the citizen `1/33` aboard the barge `1/36` here
    and `1/38` there parts first, its `inside` apart since 6604 — the
    block's reading, not a mechanism. No mechanism is named.

1111. **Chapter thirty-nine: the spell issuer — no capture yet** (1131
    closed thirty-eight at 1762; CENSUS's blind list, row 7, the rest of
    the largest family a known staging reaches). `SpellType::cast_civilian`,
    `cast_to_arms` and `cast_create_decoy`, with a City and a General
    staged under the issuer (parked 1111). The emulator first; **run422**
    the capture, run423 at `cover=1`. GOLDEN §48.

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
