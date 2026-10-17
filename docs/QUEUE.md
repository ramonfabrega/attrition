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

*2026-10-17, **the thirty-ninth pass (Fable), steered with Ramon: five lanes, the battery a gate floor, five lobbies to mint,
the nations audited** (DECISIONS 77). Twenty landings from `31eb3a05` in 3 h 53 (21 in 4 h 44 with the drain's), 23.6 min
waiting a landing (35.2), every gate green, two `DEEP`, no kill rule, no merge backed out; 13 Loop items built or closed,
seven stay; `docs/audit/2026-10-17-fable-pass-39.md`.*

- **Words**: every pair closed (the peace pair at 6822, 2864). The AI word is the battery's `EastIndiesGermanTier3`
  (DECISIONS 72): 576 → 5577 (2931) → 6585 (2956) → 12815 (2974), as 2997; it stays on its lobby until it closes (DECISIONS 77 (vii)).
  Chapters 86–92 closed; 93 is 3018. Census 1224 → 1418 of 3611. Battery eight lobbies (2975), 0 to 9130, median 1008, a gate floor.
- **Lanes: fifteen landings** (2976 NATIONS.md; 2959, 2966, 2990, 3004, 3008, 3014, 3024, 3030, 3038, 3044 Census → 1418;
  2971, 2994 ch91–92; 2974 6585 → 12815; 2975 five lobbies). 2997 the AI word's (Opus 5.5 medium, run 912); 3018 rules (run
  913), 3049 and 3064 harvest, 3058 shape (Sonnet 5.5 medium); twenty with these five, so no more refills. Next 3065, run 914 (Astra 900–909). Pools: GOLDEN §118–120, AI §245–247, park HG–HJ.
- **Fable backlog: 24 Loop items** (1570, 1725, 2003, 2899, 2948, 2958, 2973, 2983, 2984, 2987, 2989, 3002, 3003, 3007, 3012, 3013, 3022, 3023, 3037, 3043, 3048, 3057, 3062, 3063).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 1850/1850 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w18140 of 18,140 · GreatLakes w5930 of 5,930
Third pair: EastIndiesFrench w17379 of 17,379 · GreatLakesFrench w5638 of 5,638
Coverage pair: EastIndiesPersianAllTech w4730 of 4,730
Successor pair: EastIndiesMongolAllTech w5056 of 5,056
Peace pair: EastIndiesLakotaPeace w6822 of 6,822
Battery word: EastIndiesGermanTier3 w12815 of 13,519
Census: simulation backed 1418 of 3611
Third map: GreatSahara w24000 of 24,000 · GreatSaharaToughest w15432 of 15,432 · GreatSaharaPersianAllTech w4340 of 4,340 · GreatSaharaMongolAllTech w3268 of 3,268
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · ch25 closed · ch26 closed · ch27 closed · ch28 closed · ch29 closed · ch30 closed · ch31 closed · ch32 closed · ch33 closed · ch34 closed · ch35 closed · ch36 closed · ch37 closed · ch38 closed · ch39 closed · ch40 closed · ch41 closed · ch42 closed · ch43 closed · ch44 closed · ch45 closed · ch46 closed · ch47 closed · ch48 closed · ch49 closed · ch50 closed · ch51 closed · ch53 closed · ch55 closed · ch56 closed · ch57 closed · ch59 closed · ch61 closed · ch62 closed · ch63 closed · ch64 closed · ch65 closed · ch67 closed · ch68 closed · ch71 closed · ch72 closed · ch73 closed · ch74 closed · ch75 closed · ch76 closed · ch77 closed · ch78 closed · ch79 closed · ch80 closed · ch85 closed · ch86 closed · ch87 closed · ch88 closed · ch89 closed · ch90 closed · ch91 closed · ch92 closed
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked · GreatSahara 0 off, 0 unlinked

**Opener: continue the fortieth tranche from `docs/QUEUE.md`'s handoff (commander, Opus): fifteen landings booked; five
lanes live — `att-2997` (Opus 5.5), `att-3018`, `att-3049`, `att-3058`, `att-3064` (Sonnet 5.5); arm `lanewait.py` on them;
merge, book, refill while landings + live lanes < 20; the whole gate every third landing; stop at twenty.**

## The queue

In dependency order, headline-nearest first. **The roster is Ramon's**:
2997 the AI word's (Opus); 3018, 3049, 3058 and 3064 (Sonnet). The battery word is the AI's — lower map first — East Indies (Germans), at 12815. Take the first unstarted item unless a better
order is clear, and say why. The backlog is `docs/PARKED.md`.

2997. **The battery word, 12815 of 13519 (`EastIndiesGermanTier3`, East Indies):
    ours 9 draws against theirs 8** (2974 moved it from 6585 with the
    Germans' Fire Raft costs and speed; run910 widens it, 967 keys): at
    index 1 ours draws `Unit::fight+0x9b0` where theirs draws
    `Guy::set_anim+0x97a < Guy::inc_time+0x271`. Positions part already
    on 12809 (`0/0` (4680, 9362) against (4674, 9396)), so the state first
    parts in 6837..12808, which no dump holds: a windowed capture or a
    bisection, run 912. Unstarted. AI §244.

3018. **Chapter ninety-three: Sabotage on a vehicle** (DECISIONS 56 §3;
    2994 closed chapter 92 on run911 to 1200, Sabotage's building arm
    razing a Barracks, parked 3015–3017): `cast_sabotage`'s vehicle arm
    (`dtype` 3, the boom) is built and never staged; a who=1 vehicle
    staged out of a Stockade's border (a lone Commando bleeds dead
    mid-cast, parked 3017), run 913. Unstarted. GOLDEN §117.

3058. **The sweep's next shape batch: `Unit::update_armor` and 3030's
    tails** (DECISIONS 63 (iv); the score `Census:` 1407 of 3611; parked
    3050–3056, 3019, 3020): `Unit::update_armor` (parked twice: `--answer-by`
    from the stub's `ecx` with `--link` on `+0x8e`), `Tribes::find`'s
    `_wcsicmp` arm (an `iat:` stub, 3051), `Build::swap_team` and
    `SubObject`'s sibling (3055), `AmmoData::walk_data`, `Game::walk_data`,
    `TurnControl::end_cannon_time` (3056). Unstarted. `--kind sweep`.

3049. **A harvest batch beside 3044: ten never-backed simulation functions
    a built shape holds** (DECISIONS 61 §5; the score `Census:` 1399 of
    3611; parked 3045–3047): as 3044, `--claimed` naming 3044's and the
    shape lane's trees; never a `process_*`. Unstarted. `--kind sweep`.

3064. **A harvest batch beside 3049: ten never-backed simulation functions
    a built shape holds** (DECISIONS 61 §5; the score `Census:` 1418 of
    3611; parked 3059–3061): as 3049, `--claimed` naming 3049's and the
    shape lane's trees; never a `process_*`. Unstarted. `--kind sweep`.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch, headline
  first and whether it moved; delete finished items — their story is in
  `docs/journal/`. Shorten only what you touch; never widen a line to beat a count.
- **A residue item is booked by its frame and its draw delta** (DECISIONS 42); a mechanism in its title is the previous
  item's hypothesis, not a build order, and an absence it cites ("ours has no X") names the probe that showed it (Loop 2704).
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
- **The gate is `python3 tools/release_gate.py <install> --test-threads 8 [--scope <tip>]`** (DECISIONS 72), to
  a file, never piped — a pipe launders the 137 — ending `Gate steps:`; **a worker's takes `--lane --tests
  --test-threads 4`** (the brief's, 1623), quotes `Lane verdict:` (969), and widens on a branch that moves a pin (2415).
