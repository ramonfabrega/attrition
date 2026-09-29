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

*2026-09-29, the commander after the eighteenth pass: **twenty landings
from `7e9025da`, and the steering pass is due.** Both headlines moved;
every floor held on every gate; the first pair stays closed at 24,000,
endpoints 0 off. No lane is live.*

- **The AI track**: Great Lakes 4924 → 5930, closed at its game's end,
  endpoint 0 off, in seven landings (1061 … 1099) — one tranche where
  DECISIONS 54 priced two. East Indies 5606 → 6151 in four (1106 … 1120).
- **The rules track**: chapters thirty-five to thirty-seven closed (the
  V2, the missile's arms, the nuke); thirty-eight, the air line under
  fire, open at 878 after five landings; the blind list 143 → 141.
- **A third map is scored** (1066, AI §83): Great Sahara, word 8.
- **For the pass**: the default-map guard asks for a closed map (1121);
  some twenty spot grants let two lanes share `fight.rs`, `orders.rs`, `anim.rs`.
- **The user's**: whether phase 4 opens on the rules track alone. The
  disk: 56 GB free, from 73 at the tranche's start.
- **Fable backlog: 31 Loop items** (677, 685, 894, 900, 927, 953, 954, 960, 973, 1067, 1071, 1076, 1079, 1080, 1083, 1085, 1088, 1090, 1094, 1097, 1101, 1105, 1108, 1114, 1116, 1119, 1121, 1123, 1126, 1130, 1132).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 6/5 w8
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w6151 of 18,140 · GreatLakes w5930 of 5,930
Third map: GreatSahara w8 of 24,000
Golden: ch38 w878 of 1763 · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · ch25 closed · ch26 closed · ch27 closed · ch28 closed · ch29 closed · ch30 closed · ch31 closed · ch32 closed · ch33 closed · ch34 closed · ch35 closed · ch36 closed · ch37 closed · 1131 next
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked

**Opener: the nineteenth Fable pass, over landings 1061 to 1113; then
the commander resumes — 1127 on the AI lane, 1131 on the rules lane.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41, 53): the golden word for the rules, the newest pair's word for the AI,
lower map first — Great Lakes, closed at its end, so East Indies (1121).
Take the first unstarted on either track unless a better order is obvious,
and say so; the backlog is `docs/PARKED.md`, back only when a score names it.

1127. **East Indies' second word: frame 6151, ours 9 draws against 8**
    (1120), at index 0: ours `Unit::move_step+0x823`, theirs
    `Unit::do_non_flat_gather+0x10f`, widened on run414 (block 6152),
    where only the Caravan `1/15` parts: ours stands it against `1/33`
    — the block's reading, not a mechanism. No mechanism is named.

1131. **Chapter thirty-eight's word: frame 878, ours 7 draws against 5**
    (1113: the follower scatter and the pack built): `0/7` is shot down
    in ours. Walked back: 837 the Radar's `attack_ox`, 820 `0/7`'s
    patrol leg, 784 its hit a frame late — parked 1125's release
    vectors, and 1118's building `near_o` may be the 837 row. No
    mechanism is named.

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
