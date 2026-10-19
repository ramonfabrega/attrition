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

*2026-10-19, **the forty-fifth tranche (commander, Opus): 10 of 20 landed from `43e161a7`**, three lanes (DECISIONS
83); the commander cleared at this seam. Six battery words closed; 3304, 3320 swept. Last whole gate `6ecf90d2`.*

- **Words**: every pair closed. Battery closed whole: Iroquois 12658 (3306), Romans 18620 (3314, 3322), Spanish 22700
  (3318), Greeks 24000 (3326), Japanese 24000 (3331), Dutch 24000 (3329, 3333). Open: the Egyptians 2974 → 15982 on
  3332's branch, the Indians 221 (3335). Battery on `987c4c33`: 6563, 2974, **24000**, 9130, **14991**, 1567, 9185,
  9382 (the Chinese and Turks above their pins, 3326b, the pass's).
- **Lanes: three live**: 3332 Opus (att-3332, runs 966, 967, AI §269, KQ; unpushed, owes its gate), 3335 Opus (runs
  969, 970, AI §271, KS), 3330 Sonnet (KO). Next 3336, run 971. Pools: GOLDEN §144–148, AI §272–276, park JK, KT–KV.
- **Fable backlog: 15 Loop items** (1570, 2003, 3013, 3118, 3143, 3188, 3237, 3240, 3311, 3312, 3313, 3323, 3327, 3328,
  3334).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 1850/1850 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w18140 of 18,140 · GreatLakes w5930 of 5,930
Third pair: EastIndiesFrench w17379 of 17,379 · GreatLakesFrench w5638 of 5,638
Coverage pair: EastIndiesPersianAllTech w4730 of 4,730
Successor pair: EastIndiesMongolAllTech w5056 of 5,056
Peace pair: EastIndiesLakotaPeace w6822 of 6,822
Battery word: EastIndiesDutchTier4 w24000 of 24,000 · GreatSaharaJapaneseTier1 w24000 of 24,000 · GreatSaharaGreekTier1 w24000 of 24,000 · GreatSaharaSpanishTier3 w22700 of 22,700 · GreatLakesIroquoisTier3 w12658 of 12,658 · EastIndiesRomanTier2 w18620 of 18,620 · EastIndiesGermanTier3 w13519 of 13,519
Census: simulation backed 1566 of 3611 · port 411 · layout 1155
Third map: GreatSahara w24000 of 24,000 · GreatSaharaToughest w15432 of 15,432 · GreatSaharaPersianAllTech w4340 of 4,340 · GreatSaharaMongolAllTech w3268 of 3,268
Golden: every chapter closed · 99 closed
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked · GreatSahara 0 off, 0 unlinked

**Opener: continue the forty-fifth tranche from docs/QUEUE.md's handoff — commander, Opus: read `docs/LOOP.md` first;
10 of 20 landed from `43e161a7`; re-arm `tools/lanewait.py` on att-3332, att-3335, att-3330 and carry the chain (refill
a battery word on Opus 5.5 medium by `battery_measure`'s lowest open lobby, the sweep on Sonnet 5.5 medium); twenty landings.**

## The queue

In dependency order, headline-nearest first. **The roster is Ramon's** (the forty-fourth steer, DECISIONS 83):
3332 and 3335 the battery words' (Opus), 3330 the sweep's (Sonnet). **A battery word's lane lands wide**: every fix its
widening exposes to the next parting, verified, in one gate. The battery word is the AI's — lower map first — closed until 3332's pin lands (`AI_WORDS`
holds seven closed games); open: the Egyptians on Great Sahara (3332) and the Indians at 221 on East Indies (3335). Take the first unstarted item unless a better order is clear, and say why. The backlog is `docs/PARKED.md`.

3332. **The battery's lowest open game, `GreatSaharaEgyptianTier4` at 2974 of 20380** (entry 72, by the measure on
    `987c4c33`, the Dutch at 1567 being 3329's): ours 9 draws against theirs 12 at index 2 — ours `Guy::set_anim+0x97a
    < Guy::do_turn+0x4a < Unit::move_step+0x389`, theirs `Guy::set_anim+0x104b`. Promoted by the recipe —
    `diff::egyptian_word` and its `AI_WORDS` pin, the lobby leaving `BATTERY`, a replacement minted (run 966) —
    **its widening first**: grep the lobby's start and long dumps for 2974, a Great Sahara window capture at detail
    only for what they cannot answer (run 967), both directions. Lands wide. AI §269, park KQ.

3335. **The battery's lowest open game, `EastIndiesIndianTier2` at 221** (entry 72: 3333 closed the Dutch at 24000,
    and the lowest lobby left is the one 3329 minted, the Indians at Difficulty 2, 221 on its first try): promoted by
    the recipe — `diff::indian_word` and its `AI_WORDS` pin, the lobby leaving `BATTERY`, a replacement minted (run
    969) — **its widening first**: grep the lobby's start and long dumps for 221, an East Indies window capture at
    detail only for what they cannot answer (run 970), both directions. Lands wide. AI §271, park KS.

3330. **The nations seat's next shape batch** (DECISIONS 63 (iv), Loop 3188; the score `Census:` port 411; 3320 built
    five of six, with sim fixes the sweeps forced in Ambush's time, `find_gather_spot`'s return and the decoy limits,
    parked 3320a–h): `Group::action_queue_up`, `UnitData::calc_gather`, `Unit::think_spellcaster`,
    `TechType::compute_ai_values`, `SpellType::cast_transport`, `Unit::go_around_building` (`census.py --unbacked`,
    the `port` mark) — each built against the crate's own method where it has one, or parked naming the generator
    flag it could not spell. `--kind sweep`.

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
  so are `census::SIMULATION_BACKED` and `SIMULATION_PORT_BACKED`, re-pinned at booking (Loop 2367, 3093).
  The chain — merge, book, spawn, gate, push, reap — is `CLAUDE.md`'s.
- **The floors and these lines move together** — `FLOORS`, `LONG_WORD_*`,
  `GOLDEN_WORD_*`, `ENDPOINTS`, `AI_WORDS` with `Scoreboard:`, `Long captures:`,
  `Golden:` (`<name> closed`, or `every chapter closed` first), `Endpoint <frame>:`,
  `Second pair:`, `Third map:`, `Census:`, `Fable backlog: N Loop items` and `lower map first — <map>` — read
  literally and **never wrapped**, or the count guard matches this line
  instead (twice, 09-19). The length guard counts every line to the next `## `.
- **A capture is a draw-stream trace first, detail on demand** (DECISIONS 41): whole length at `cover=0`, then a
  windowed re-run sized to the word, **from the last dumped block when a gap leaves it near** (3155); after 363, the
  click-free lane unless it needs the mouse. Overlap the neighbours: six blocks each end (run83); `samegame.py` exits
  0 when nothing is in common, so assert the count, not the verdict; **a detached capture waits on `waitrun.sh`**
  (565, 656). **A value question takes a packet at the block before the first print** (EMULATOR §8, 2303).
- **Grep for the derived quantity, the SHAPE and the GATE**: a term zero in every dump is
  zero-valued or switched off (117, 178); **a never-cleared field asserts a CHANGE, not a value** (`collide_frame`, run85).
- **The gate is `python3 tools/release_gate.py <install> --test-threads 8 [--scope <tip>]`** (DECISIONS 72), to
  a file, never piped — a pipe launders the 137 — ending `Gate steps:`, `Admission wait:` and `Host load:`; **whole
  (no `--scope`) every third landing whose merge widened, and at the drain** (DECISIONS 79); **a worker's takes
  `--lane --tests --test-threads 4`** (the brief's, 1623), quotes `Lane verdict:` (969), and widens on a branch that
  moves a pin (2415).
