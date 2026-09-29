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

*2026-09-29, nineteen landings since the nineteenth pass: East Indies
6151 → 8820 (1127, 1143, 1156, 1164, 1174, 1185, 1191); Great Sahara
6/5 → 1850/1850, long word 8 → 17623 (1133, 1147, 1163, 1171, 1177,
1189, 1194); chapters thirty-eight to forty-one closed (… 1182, 1200).*

- **Three lanes, on the user's word** (DECISIONS 55, amended): the
  second pair's — 1197, East Indies' 8820; **the third map's** — 1206,
  Great Sahara's frame 17623; the rules' — 1209, chapter forty-two.
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
- **Fable backlog: 30 Loop items** (685, 1101, 1105, 1114, 1119, 1138, 1139, 1140, 1141, 1142, 1146, 1150, 1151, 1155, 1159, 1161, 1162, 1166, 1170, 1173, 1176, 1180, 1181, 1187, 1188, 1193, 1199, 1205, 1208, 1213).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 1850/1850 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w8820 of 18,140 · GreatLakes w5930 of 5,930
Third map: GreatSahara w17623 of 24,000
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · ch25 closed · ch26 closed · ch27 closed · ch28 closed · ch29 closed · ch30 closed · ch31 closed · ch32 closed · ch33 closed · ch34 closed · ch35 closed · ch36 closed · ch37 closed · ch38 closed · ch39 closed · ch40 closed · ch41 closed · 1209 next
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked

**Opener: the commander resumes on three live lanes, `att-<item>` —
1197, 1206 and 1209; the twentieth pass at twenty landings, one to go.**

## The queue

In dependency order, headline-nearest first. **Three lanes, one to an open
word** (DECISIONS 41, 53, 55): the golden word for the rules, the third map's, and the newest pair's,
lower map first — East Indies (Great Lakes is closed at its end).
Take the first unstarted on a lane's own track unless a better order is obvious,
and say so; the backlog is `docs/PARKED.md`, back only when a score names it.

1206. **Great Sahara's word: frame 17623, ours 7 draws against 8**
    (1194), at index 2: ours `Guy::set_anim+0x97a < Guy::inc_time+0x271`,
    theirs `Guy::set_anim+0x97a < Unit::do_idle+0x7d`, widened on run449
    (blocks 17618..17874), where `1/52` is idle there and walking here on
    17624 and `1/46`, `1/51` hold an order the original has finished;
    block 17618 already stands on four of who=1's soldiers' positions,
    parted in the gap 16933..17617 no dump covers. No mechanism is named.

1197. **East Indies' second word: frame 8820, ours 2 draws against 3**
    (1191), at index 2: theirs a third `Guy::set_anim+0x97a <
    Guy::inc_time+0x271 < Unit::inc_time+0x3e`, ours none, widened on
    run445 (block 8821, quiet beyond the standing keys through 8822);
    block 8815 stands on 504 keys, among them the figure clocks of `1/68`
    and `1/70`, two of the four figures the original wraps on 8820. No
    mechanism is named.

1209. **Chapter forty-two: a Gate across a path — no capture yet**
    (1200 closed forty-one at 1770; CENSUS row 7's last two).
    `Unit::resolve_block`, which `do_move` asks where
    `PathFinder::astar_path` flags a passable building tile: a wall with a
    Gate across an enemy's path, both sides' walkers. Which arms a staging reaches is the item's
    first finding. The emulator first; **run460** the capture, run461 at
    `cover=1`. GOLDEN §51.

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
