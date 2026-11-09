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
*2026-11-08, **the seventy-fourth steering pass, with Ramon on the row: the finish line is written (`docs/TRACKS.md`), phase 4 is open, and the seventy-fifth tranche is the structure tranche, 19 of 20 landed from `3e4ea444a`** (DECISIONS 114). Two Opus 5.5 lanes at `--effort medium`, the count fixed on Ramon's word: lane A runs `docs/STRUCTURE.md` §5's moves in order, alone on the sim side; lane B runs the renderer's two sim-free items (`docs/RENDERER.md` §6 items 2 and 4). The record is `docs/audit/2026-11-08-fable-pass-74.md`.*
- **The rule of every structure landing is no behaviour change**: every floor below unchanged, the soak digest unchanged, the census reading the same; the gate is the whole release gate, no `--scope`. **The rules track is whole**: every word stands at its length, chapter 134 closed at 1,400 by 4007 (`valid_target_const`'s air ladder now asks the target's own `is_flying_high`/`is_flying_low`; 1/6's orders equal on 969 and 970, empty on 971 in both); chapter 135 (3994, run1552: Delete on a finished and an unfinished building, the Disband clock's research arm, the Helicopter skip in `get_max_range` now held by a walk) closed at 1250 on its first take; chapter 136 (3995, run1553: `think_carry_idle` and the barge's no-army arm wired — at sea it walks to the owner's nearest city, so adjudication 4 row 50's eject reading was wrong — and the barge formed once, ch114's seven rows gone) closed at 900 on its first take; the pass's three shelved items (4007, 3994, 3995) re-booked from `docs/PARKED.md` at 4017's booking, the structure's code moves all landed (4008–4017; §5 move 10, the README, was 4021); lower map first — closed.
- **The structure's number is the `Structure:` line**, `tools/shape.py`'s counts (item 4008 built it; `rondata::diff::floors::STRUCTURE` pins the exact counts): each may only fall — `largest` rose 13437 → 13442 at 4012 by `orders/mod.rs` declaring its five siblings, a named rise, retired at 4013 (13336); 4015's four cuts took it to 5437, where two constant tables lead (`data/launch.rs` 5437, `data/pivot.rs` 4517, left uncut on the brief's word — the ≤ 4000 target for tables goes to the pass) and `cities/city.rs` 4113 is the one production file above 4,000. `fields` 92 → 21 at 4016: a five-field `tables` bundle (`nation` went to `cities/`'s state, since `refresh_nation_powers` rewrites it on a city gained or lost) and twelve per-directory states; the per-player record was not built. 4017's doors: `sim::observe`, `sim::command` (40 methods) and `sim::setup` (47) — `rondata`'s 161 direct method calls now 74 `&self` reads plus `Sim::new` and `form_update_positions`, its 74 flat paths 0, the shim deleted; `sweep-lines` 17255 → 17286 on the full paths' rewraps, a named rise the line's 17.3k does not show. The renderer's is `docs/RENDERER.md` §5.4's K of M, **K 32 of M 60** at 4020 (M: 39 pieces the runner's scenes play on GreatLakesAztecModern plus 21 building types its dumps hold; K: 32 of the 39 decode — six gaia pieces no `<UNIT>` names and one citizen crew row short — and the building half has no decoder yet); the scene equals the harness's on 3390 of 3390 ticks, 35 of 35 logged frames (`free::tests` `SCENE_TICKS_AGREEING`), 306 of 306 dumped `gpiece`s agree; the clock's first readings 431 ms to frame zero and 325 MiB peak against the harness's 540 ms and 458 MiB; at 4026 the draw list stands **frames in register 33 of 35** (`FRAMES_IN_REGISTER`, figures 3840 of 3842) and **K 29 of M 59** (31 of 38 pieces decoding, 36 of 38 in register; piece 265 left M with the dead units `observe` now drops; `FigureScene` gained `z`), the clock 434 ms and 325 MiB; at 4027 **frames in register 35 of 35** (figures 3842 of 3842: two units pushed by a later unit's collision left guy 0 behind, and `observe` now reads guy 0's point off `Movement::body`) and **K 31 of M 59** (`RENDER_K (31, 31, 38, 38)`), every decoded piece in register — the camera stands in register 240 of 240 sweep states and 394 of 394 packet points (4018, `crates/render/tests/camera_sweep.rs` `IN_REGISTER`), and 1,439 of 1,452 pieces decode (4019, `rondata::artdata::PIECES_DECODING`; the 13 are three `.bha` the install does not ship and one `.bh3` with a node past its vertices); its clock (seconds to lobby and to frame zero, slowest-percentile frame ms, peak RSS) from the first client item.
- **Fable backlog: 14 Loop items** (4025, Ramon's: the crate mixes `mod.rs` and `foo.rs` + `foo/` layouts — one convention, one landing, a clippy lint, before or right after 4015; 4024, `seams.py` blames without `-C`; 4022, 4023 (the frame spells `ccc update att-<n>`; ccc takes `lane-<N>`), `brief.py` gains the `structure` and `render` kinds — until then the commander briefs a structure item from `docs/STRUCTURE.md` §1 and §5 and a render item from `docs/RENDERER.md` §6 with the frame's shared sections; 3997; 3848, 3821, 3799, 3382, 3350, 3237, 3188, 3143, 3118).
- **The blind list: open 0** (never 331 · residue 331). The territory 549 · 318 · 1286; the census and the per-group rulings are a lane's filler from lore's scoring (DECISIONS 114 §6), after the structure.
- **Lanes: lane A empty — its list is spent; lane-4028 (B, park DL), the tranche's twentieth; run 1558 returned unused, 1559 held.** `ccc spawn --name lane-<N> --worktree=att-<N> --model 'claude-opus-5-5[1m]' --effort medium`; a structure lane's gate is `python3 tools/release_gate.py <install> --test-threads 4 --lane --whole` — the whole suite, since a move touches every path; a render lane's is `--lane --tests render` once the crate exists, the fast suite before. Next 4029, run 1559. Pools: AI §417–§419; GOLDEN §195–§198; MAPGEN §35–§37; park DO–DR free (DL held). Disk: 28 GB free after the pass's acts, `~/ron-golden` still compressing; the spawn floor is 10 GB.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 1850/1850 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w18140 of 18,140 · GreatLakes w5930 of 5,930
Third pair: EastIndiesFrench w17379 of 17,379 · GreatLakesFrench w5638 of 5,638
Coverage pair: EastIndiesPersianAllTech w4730 of 4,730
Successor pair: EastIndiesMongolAllTech w5056 of 5,056
Peace pair: EastIndiesLakotaPeace w6822 of 6,822
Late ages: EastIndiesGermanAllTech w4175 of 4,175 · GreatSaharaAztecAllTech w4565 of 4,565 · EastIndiesRussianModern w5841 of 5,841 · GreatSaharaAmericanModern w4548 of 4,548 · EastIndiesJapaneseIndustrial w4547 of 4,547 · GreatSaharaChineseIndustrial w4467 of 4,467 · EastIndiesBritishEnlightenment w5522 of 5,522 · GreatSaharaTurkishEnlightenment w3812 of 3,812 · EastIndiesPersianMedieval w4759 of 4,759 · EastIndiesGreekClassical w8126 of 8,126 · EastIndiesKoreanAncient w5639 of 5,639 · GreatSaharaRomanClassical w7130 of 7,130 · GreatSaharaMongolClassical w9884 of 9,884 · GreatSaharaChineseMedieval w4520 of 4,520 · EastIndiesLakotaMedieval w4973 of 4,973 · EastIndiesBantuAncient w6210 of 6,210 · GreatSaharaJapaneseGunpowder w3673 of 3,673 · GreatSaharaIroquoisAncient w8420 of 8,420 · EastIndiesEgyptianClassical w6748 of 6,748 · GreatSaharaSpanishClassical w8565 of 8,565 · EastIndiesMayaAncient w5935 of 5,935 · GreatSaharaGermanAncient w5314 of 5,314 · GreatSaharaKoreanClassical w8777 of 8,777 · EastIndiesJapaneseAncient w6126 of 6,126 · GreatSaharaEgyptianGunpowder w4454 of 4,454 · EastIndiesAztecAncient w9258 of 9,258 · GreatSaharaRussianMedieval w4541 of 4,541 · EastIndiesIncaClassical w6073 of 6,073 · GreatSaharaFrenchClassical w6106 of 6,106 · EastIndiesDutchMedieval w7095 of 7,095 · EastIndiesRomanMedieval w5979 of 5,979 · EastIndiesMongolGunpowder w5399 of 5,399 · GreatSaharaBantuMedieval w4914 of 4,914 · EastIndiesChineseGunpowder w5568 of 5,568 · GreatSaharaGreekGunpowder w4074 of 4,074 · EastIndiesSpanishGunpowder w6128 of 6,128 · GreatSaharaLakotaGunpowder w4750 of 4,750 · EastIndiesIroquoisEnlightenment w5128 of 5,128 · GreatSaharaAztecMedieval w4602 of 4,602 · GreatSaharaIncaEnlightenment w4045 of 4,045 · EastIndiesIndianEnlightenment w5000 of 5,000 · EastIndiesAmericanEnlightenment w6669 of 6,669 · EastIndiesBritishClassical w8980 of 8,980 · GreatLakesPersianClassical w11633 of 11,633 · GreatLakesJapaneseMedieval w5140 of 5,140 · GreatLakesKoreanMedieval w4559 of 4,559 · GreatLakesGermanIndustrial w4772 of 4,772 · GreatLakesRomanGunpowder w4062 of 4,062 · EastIndiesFrenchEnlightenment w4940 of 4,940 · GreatLakesMongolEnlightenment w4949 of 4,949 · GreatSaharaBritishIndustrial w4767 of 4,767 · GreatSaharaAmericanIndustrial w5861 of 5,861 · GreatLakesRussianGunpowder w5341 of 5,341 · GreatLakesGreekEnlightenment w4705 of 4,705 · GreatLakesLakotaEnlightenment w4673 of 4,673 · GreatLakesChineseEnlightenment w3836 of 3,836 · EastIndiesJapaneseEnlightenment w4701 of 4,701 · GreatLakesSpanishIndustrial w4200 of 4,200 · EastIndiesPersianGunpowder w5673 of 5,673 · GreatLakesIncaIndustrial w4471 of 4,471 · EastIndiesEgyptianIndustrial w5689 of 5,689 · GreatLakesBritishMedieval w5282 of 5,282 · EastIndiesMongolMedieval w5592 of 5,592 · EastIndiesGreekMedieval w5942 of 5,942 · GreatLakesFrenchMedieval w4664 of 4,664 · EastIndiesSpanishModern w4081 of 4,081 · GreatLakesGermanModern w5304 of 5,304 · GreatLakesEgyptianInformation w3797 of 3,797 · EastIndiesRussianInformation w6706 of 6,706 · GreatLakesJapaneseModern w3866 of 3,866 · GreatLakesBantuModern w4395 of 4,395 · GreatLakesEgyptianModern w3878 of 3,878 · GreatLakesAmericanInformation w4320 of 4,320 · GreatLakesMayaInformation w3330 of 3,330 · GreatLakesAztecModern w3389 of 3,389 · EastIndiesChineseModern w4130 of 4,130 · GreatLakesBritishModern w3793 of 3,793 · EastIndiesRomanModern w5234 of 5,234 · EastIndiesTurkishInformation w5036 of 5,036 · GreatSaharaFrenchInformation w3191 of 3,191 · GreatLakesSpanishInformation w3500 of 3,500 · GreatLakesGermanIndustrialModerate w4784 of 4,784
Battery word: GreatLakesSpanishTier4 w14068 of 14,068 · GreatLakesAmericanTier4 w6152 of 6,152 · GreatLakesTurkishTier4 w15503 of 15,503 · EastIndiesSpanishTier3 w24000 of 24,000 · EastIndiesIncaTier3 w24000 of 24,000 · GreatSaharaFrenchTier4 w16764 of 16,764 · GreatLakesPersianTier2 w7105 of 7,105 · GreatSaharaAztecTier2 w17554 of 17,554 · EastIndiesAmericanTier1 w24000 of 24,000 · GreatSaharaKoreanTier3 w15944 of 15,944 · GreatSaharaDutchTier3 w16109 of 16,109 · GreatSaharaIroquoisTier3 w16371 of 16,371 · GreatSaharaGermanTier3 w19483 of 19,483 · GreatSaharaIncaTier3 w15632 of 15,632 · GreatSaharaIndianTier3 w16564 of 16,564 · GreatLakesLakotaTier2 w7902 of 7,902 · GreatSaharaBantuTier4 w23627 of 23,627 · GreatLakesRussianTier2 w7303 of 7,303 · EastIndiesIndianTier2 w24000 of 24,000 · GreatSaharaMayaTier4 w13086 of 13,086 · GreatSaharaEgyptianTier4 w20380 of 20,380 · EastIndiesDutchTier4 w24000 of 24,000 · GreatSaharaJapaneseTier1 w24000 of 24,000 · GreatSaharaGreekTier1 w24000 of 24,000 · GreatSaharaSpanishTier3 w22700 of 22,700 · GreatLakesIroquoisTier3 w12658 of 12,658 · EastIndiesRomanTier2 w18620 of 18,620 · EastIndiesGermanTier3 w13519 of 13,519
Melee: GreatSaharaGunpowderMelee w2600 of 2,600 0 off · GreatSaharaClassicalMelee w2600 of 2,600 0 off · GreatSaharaInformationMelee w2600 of 2,600 0 off · EastIndiesInformationMelee w2600 of 2,600 0 off · EastIndiesAirbaseMelee w2600 of 2,600 0 off · GreatSaharaCorpseMelee w2600 of 2,600 0 off
Mapgen: checksum 496 of 496 · GreatSahara 8 of 8 phases · GreatLakes 8 of 8 phases · EastIndies 8 of 8 phases · SouthwestMesa 8 of 8 phases · OldWorld 8 of 8 phases · EastMeetsWest 8 of 8 phases · AmazonBasin 8 of 8 phases · Himalayas 8 of 8 phases · AfricanWateringHole 8 of 8 phases · AustralianOutback 8 of 8 phases · Mediterranean 8 of 8 phases · NewWorld 8 of 8 phases · SeaPower 8 of 8 phases · NileDelta 8 of 8 phases · ColonialPowers 8 of 8 phases
Census: simulation backed 1448 of 3611 · port 589 · layout 859
Third map: GreatSahara w24000 of 24,000 · GreatSaharaToughest w15432 of 15,432 · GreatSaharaPersianAllTech w4340 of 4,340 · GreatSaharaMongolAllTech w3268 of 3,268
Golden: every chapter closed · 130 closed
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked · GreatSahara 0 off, 0 unlinked
Blind: never 331 · residue 331 · open 0
Rubric: ported 549 · replaced by design 318 · open 1286 of 3611 · lockstep bytes 78.1 % · ages held 8 of 8 · unit types seen 296 of 302
Held-out: 8 of 9 whole · shortest 6147
Structure: flat 7 · test-tails 0k · sweep-lines 17.3k · names 15 · fields 21 · largest 5437


**Opener: the seventy-fourth steering pass (steer, Fable) — the seventy-third tranche drained at twenty from `3f3a403b5`, every chapter closed but 134 (w969 of 1,400); Ramon joins on this row, so open with his piled-up asks (`docs/RENDERER.md` §7.2, the two census defects, the per-group rulings) and where the project goes next; run `lore index`, read "The drain" above, then rule on the lane rule's reading (49.3 minutes waiting a landing), on what feeds the lanes with three items left (4007, 3994, 3995), on the disk (the spawn floor hit three times) and on Loop 4006; write the base as this file's `from` line, book the first items, and arm the commander.**

**Opener: continue the seventy-fifth tranche (commander, Opus) from docs/QUEUE.md's handoff: the structure tranche, two Opus 5.5 lanes at medium from the base this file's `from` line names — lane A on 4008 then `docs/STRUCTURE.md` §5 in order, lane B on 4018 then 4019; every structure landing's gate is the whole release gate with every floor unchanged; the chain is docs/LOOP.md's; at twenty landings, or when both lists are spent, write the handoff and clear the steer row.**

## The queue

In dependency order; **the roster is Ramon's** (DECISIONS 84–93, 114 §3). The shapes are DECISIONS 90–94 and 100 §3, and from this
pass a `structure` item (`docs/STRUCTURE.md` §1: a move, split, rename or re-export with no behaviour change, gated whole) and a
`render` item (`docs/RENDERER.md` §6). A residue or chapter item re-books from `docs/PARKED.md` only once the structure has landed.

4028. **The decode half of K** (a `render` item; RENDERER §5.4 condition 1, §6 item 4): the seven of M's 38 pieces that do not
decode (six gaia pieces no `<UNIT>` names, the crew row 38322) resolved through the original's loader, and M's 21 building types
resolved and decoded under the self-checking readers, with what a decoded building earns without a dumped frame ruled per §5.4.
Moves K of M. Lane B.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch, headline first and whether it moved; delete
  finished items — their story is in `docs/journal/`. Shorten only what you touch; never widen a line to beat a count.
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
  so are `census::SIMULATION_BACKED` and `SIMULATION_PORT_BACKED`, re-pinned at booking (Loop 2367, 3093).
  The chain — merge, book, spawn, gate, push, reap — is `CLAUDE.md`'s.
- **The floors and these lines move together** — `FLOORS`, `LONG_WORD_*`,
  `GOLDEN_WORD_*`, `ENDPOINTS`, `AI_WORDS` with `Scoreboard:`, `Long captures:`,
  `Golden:` (`<name> closed`, or `every chapter closed` first), `Endpoint <frame>:`,
  `Second pair:`, `Third map:`, `Census:`, `Blind:`, `Rubric:` (the worker re-pins `RUBRIC_UNIT_TYPES_SEEN`, the commander writes it; DECISIONS 90),
  `Melee:` (`… of <length> <n> off`, pinned by `*_MELEE_CLOSING_OFF`, Loop 3842), `Fable backlog: N Loop items` and `lower map first — <map>` — read
  literally and **never wrapped**, or the count guard matches this line
  instead (twice, 09-19). The length guard counts every line to the next `## `.
- **A capture is a draw-stream trace first, detail on demand** (DECISIONS 41): whole length at `cover=0`, then a
  windowed re-run sized to the word, **from the last dumped block when a gap leaves it near** (3155); after 363, the
  click-free lane unless it needs the mouse. Overlap the neighbours: six blocks each end (run83); `samegame.py` exits
  0 on nothing in common, so assert the count; **a detached capture waits on `waitrun.sh`** (565, 656); **a `cover=1`
  twin passes `--twin <its sibling's dir>`** (3928). **A value question takes a packet before the first print** (2303).
- **Grep for the derived quantity, the SHAPE and the GATE**: a term zero in every dump is
  zero-valued or switched off (117, 178); **a never-cleared field asserts a CHANGE, not a value** (`collide_frame`, run85).
- **The gate is `python3 tools/release_gate.py <install> --test-threads 8 [--scope <tip>]`** (DECISIONS 72), to
  a file, never piped — a pipe launders the 137 — ending `Gate steps:`, `Admission wait:` and `Host load:`; **whole
  (no `--scope`) every third landing whose merge widened, and at the drain** (DECISIONS 79); **a worker's takes
  `--lane --tests --test-threads 4`** (the brief's, 1623), quotes `Lane verdict:` (969), and widens on a branch that
  moves a pin (2415).
