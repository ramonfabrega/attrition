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

*2026-10-15, **the thirty-fifth tranche, sixteen landings** since `dccda9f0`; three
lanes on DECISIONS 71, amended mid-tranche by Ramon: two kill rules tripped, the roster stands, the gap is a brief row.*

- **Words**: the peace pair (East Indies, the Lakota, `RUSH_RULES 10`, ends 6822) 1984 → 4544 in four landings
  (2384 diplomacy's peace arm, 2388 the pre-war gate, 2412 the silo's queue, 2423 the no-rush timer), the open word. Chapters 71 and 72
  closed (2436 The Despot, 2448 The Monarch); 69 and 70 built or surveyed, not closed. Census 894 → 965. Battery unmeasured since 576/6566/2974.
- **Red booking gates, each fixed by a second commit**: 2384's 144 `aggression` pins (re-pinned); 2400's
  `find_free` captain term broke the third pair's floor 17379 → 14461 (bisected, the call reverted, parked 2441).
- **Lanes: three** (DECISIONS 71, amended: two kill rules tripped, the roster stands, the gap fixed — Loop 2415): 2460 (AI §224,
  CX, runs 861–863), 2474 (GOLDEN §90, CU, run 860), 2486 (CW) live. Next 2495, run 864, AI §225, GOLDEN §91, park CY.
- **Fable backlog: 24 Loop items** (1570, 1725, 1900, 2003, 2146, 2240, 2405–2410, 2415, 2418, 2419, 2420, 2422, 2435, 2442, 2449, 2459, 2461, 2479, 2480).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 1850/1850 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w18140 of 18,140 · GreatLakes w5930 of 5,930
Third pair: EastIndiesFrench w17379 of 17,379 · GreatLakesFrench w5638 of 5,638
Coverage pair: EastIndiesPersianAllTech w4730 of 4,730
Successor pair: EastIndiesMongolAllTech w5056 of 5,056
Peace pair: EastIndiesLakotaPeace w4544 of 6,822
Census: simulation backed 965 of 3611
Third map: GreatSahara w24000 of 24,000 · GreatSaharaToughest w15432 of 15,432 · GreatSaharaPersianAllTech w4340 of 4,340 · GreatSaharaMongolAllTech w3268 of 3,268
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · ch25 closed · ch26 closed · ch27 closed · ch28 closed · ch29 closed · ch30 closed · ch31 closed · ch32 closed · ch33 closed · ch34 closed · ch35 closed · ch36 closed · ch37 closed · ch38 closed · ch39 closed · ch40 closed · ch41 closed · ch42 closed · ch43 closed · ch44 closed · ch45 closed · ch46 closed · ch47 closed · ch48 closed · ch49 closed · ch50 closed · ch51 closed · ch53 closed · ch55 closed · ch56 closed · ch57 closed · ch59 closed · ch61 closed · ch62 closed · ch63 closed · ch64 closed · ch65 closed · ch67 closed · ch68 closed · ch71 closed · ch72 closed
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked · GreatSahara 0 off, 0 unlinked

**Opener: the thirty-fifth tranche (commander): read this file, spawn three lanes on DECISIONS 71's roster — 2384
on `--model 'claude-opus-5-5[1m]' --effort medium`, 2389 and 2395 on `--model 'claude-sonnet-5-5[1m]' --effort
medium`, 2400 after 2395 on the same lane — arm `tools/lanewait.py` on every live lane, and run the chain; re-pin
`census::SIMULATION_BACKED` yourself in each sweep booking (Loop 2367); file every journal's "for the Loop" line at
its merge; at twenty landings write the handoff and clear `steer` into the thirty-fifth pass.**

## The queue

In dependency order, headline-nearest first. **The roster is Ramon's**:
2460 the AI word's (Opus), 2474 the rules track's and 2486
the sweep lane's (Sonnet) — lower map first — East Indies (Lakota), the peace pair's only map.
Take the first unstarted item unless a better order is clear, and say
why. The backlog is `docs/PARKED.md`, back only when a score names it.

2460. **The peace pair's word 4544: ours 145 draws, theirs 201, index
    0** (2423; ours `Leader::diplomacy+0x1168`, theirs
    `GameDaemon::calc_market+0x54`), 44 frames past the peace's end at
    4500. Inside run855 (blocks 4538..4794, 977 keys; block 4545 empty).
    On 4538 `leader:diplos` is ours 1 against theirs 0 both ways, and the
    ICBM `1/50` holds an order in theirs only; the cause lies in
    3552..4538, which no dump holds: `RON_DEBUG_LEADER` over 4499..4538
    names the frame the `diplos` were written. Runs 861–863. AI §224.

2474. **Chapter seventy-three: a wonder finished on a human**
    (DECISIONS 56 §3; 2448 closed seventy-two at 1770, The Monarch,
    GOLDEN §89). Parked 2438: `wonders_held` needs a city's wonder, and
    `finish`/`hurry` are unapplied verbs (GOLDEN §16); apply `finish`
    in the walk's interpreter, test first, to a Citizen-built wonder in
    a founded city (`small_city`), so AI §216's wonder arm is held on a
    human. Name the census rows first; run 860 only when no packet of
    the game is on disk. GOLDEN §90.

2486. **The sweep's seventy-eighth batch: ten never-backed simulation
    functions** (DECISIONS 63 (iv); the score `Census:` 965 of 3611):
    2481's shape; a word for a command issuer's SEH frame and failed-check
    feedback-log arm built first, a test made to fail first (parked 2482,
    Loop 2003); then the treaty family's seven and the three that read
    `game->info.player[console->play]` into the command (`_cannon_time`,
    `_player_speed`, `_chat_set`); skip operators (parked 1722). `--kind sweep`.

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
  constant is the worker's to re-pin; the line is the commander's**, and
  so is `census::SIMULATION_BACKED`, re-pinned at booking (Loop 2367).
  The chain — merge, book, spawn, gate, push, reap — is `CLAUDE.md`'s.
- **The floors and these lines move together** — `FLOORS`, `LONG_WORD_*`,
  `GOLDEN_WORD_*`, `ENDPOINTS`, `AI_WORDS` with `Scoreboard:`, `Long captures:`,
  `Golden:` (`<name> closed`, or `every chapter closed` first), `Endpoint <frame>:`,
  `Second pair:`, `Third map:`, `Census:`, `Fable backlog: N Loop items` and `lower map first — <map>` — read
  literally and **never wrapped**, or the count guard matches this line
  instead (twice, 09-19). The length guard counts every line to the next `## `.
- **A capture is a draw-stream trace first, detail on demand** (DECISIONS
  41): whole length at `cover=0`, then a windowed re-run sized to the word;
  after 363, the click-free lane unless it needs the mouse. Overlap the
  neighbours: six blocks each end (run83); `samegame.py` exits 0 when nothing
  is in common, so assert the count, not the verdict; **a detached capture
  waits on `waitrun.sh`** (565, 656). **A value question takes a packet at
  the block before the first print** (EMULATOR §8, 2303).
- **Grep for the derived quantity, the SHAPE and the GATE**: a term zero in every dump is
  zero-valued or switched off (117, 178); **a never-cleared field asserts a CHANGE, not a value** (`collide_frame`, run85).
- **The gate is `python3 tools/release_gate.py <install> --test-threads 4`**,
  to a file, never piped — a pipe launders the 137 — ending `Gate steps:`;
  **a worker's takes `--lane --tests`** (the brief's, 1623) and quotes `Lane verdict:` (969).
