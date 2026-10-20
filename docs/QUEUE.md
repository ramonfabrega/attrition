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

*2026-10-19, **the forty-fifth pass (steer, Fable): the forty-fifth tranche measured — 20 landings on three lanes from
`43e161a7` in 16 h 55, fourteen battery games closed, 14.79 USD a landing and 26 a game** (DECISIONS 84, its record
`docs/audit/2026-10-19-fable-pass-45.md`); the roster is Ramon's (DECISIONS 83): two Opus 5.5 medium lanes on battery
words, landing wide, two Sonnet 5.5 medium sweep lanes (his word at this steer, entry 84); the base is this pass's last commit.*

- **Words**: every pair closed; fourteen battery words closed (`AI_WORDS` holds thirteen). Battery on `ec062f94`: 6071
  (the Lakota, 3345), 13422 (the French, 3347), 24000, 9130, 14991, 9185, 7782, 9382 — median 9284 (4770). Census 1580, port 436.
- **Lanes: none live**; unspawned: 3345 AI (runs 979, 980, AI §277, LA), 3347 AI (runs 981, 982, AI §278, LC), 3346
  the nations seat (LB), 3348 the second sweep (LD). Next 3353, run 983. Pools: GOLDEN §144–148, AI §279–282, park LE–LF.
- **Fable backlog: 11 Loop items** (1570, 2003, 3013, 3118, 3143, 3188, 3237, 3349, 3350, 3351, 3352); DECISIONS 84.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 1850/1850 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w18140 of 18,140 · GreatLakes w5930 of 5,930
Third pair: EastIndiesFrench w17379 of 17,379 · GreatLakesFrench w5638 of 5,638
Coverage pair: EastIndiesPersianAllTech w4730 of 4,730
Successor pair: EastIndiesMongolAllTech w5056 of 5,056
Peace pair: EastIndiesLakotaPeace w6822 of 6,822
Battery word: GreatSaharaBantuTier4 w23627 of 23,627 · GreatLakesRussianTier2 w7303 of 7,303 · EastIndiesIndianTier2 w24000 of 24,000 · GreatSaharaMayaTier4 w13086 of 13,086 · GreatSaharaEgyptianTier4 w20380 of 20,380 · EastIndiesDutchTier4 w24000 of 24,000 · GreatSaharaJapaneseTier1 w24000 of 24,000 · GreatSaharaGreekTier1 w24000 of 24,000 · GreatSaharaSpanishTier3 w22700 of 22,700 · GreatLakesIroquoisTier3 w12658 of 12,658 · EastIndiesRomanTier2 w18620 of 18,620 · EastIndiesGermanTier3 w13519 of 13,519
Census: simulation backed 1580 of 3611 · port 436 · layout 1144
Third map: GreatSahara w24000 of 24,000 · GreatSaharaToughest w15432 of 15,432 · GreatSaharaPersianAllTech w4340 of 4,340 · GreatSaharaMongolAllTech w3268 of 3,268
Golden: every chapter closed · 99 closed
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked · GreatSahara 0 off, 0 unlinked

**Opener: continue the forty-sixth tranche from docs/QUEUE.md's handoff — commander, Opus: read docs/LOOP.md
first; spawn the four unspawned items (3345 and 3347 on `claude-opus-5-5[1m]` `--effort medium`, landing wide;
3346 and 3348 on `claude-sonnet-5-5[1m]` `--effort medium`, `--no-widen`, each sweep brief naming the other's
worktree for `--claimed`), `python3 tools/lanewait.py` on the four and carry the chain — a battery word's lane
refills by `battery_measure`'s lowest open lobby under entry 72's rules as amended (DECISIONS 84; the booking says
which rule minted the replacement), a sweep lane by the next six `port`-marked rows; twenty landings, then clear `steer`.**

## The queue

In dependency order, headline-nearest first. **The roster is Ramon's** (DECISIONS 83, 84): 3345 and 3347 the battery
words' (Opus), 3346 and 3348 the sweeps' (Sonnet). **A battery word's lane lands wide**: every fix its widening
exposes to the next parting, verified, in one gate. The battery word is the AI's — lower map first — closed until 3345's pin lands (`AI_WORDS` holds
thirteen); open: none until 3345 and 3347 promote the Lakota and the French. An open word is carried to its end before
the next lobby is promoted (3329 → 3333). Take the first unstarted item unless a better order is clear, and say why.

3345. **The battery's lowest open game, `GreatLakesLakotaTier2` at 6071 of 7902** (entry 72: 3343 closed the Bantu at
    23627; the Lakota minted by 3342 under rule (2) of entry 72 as amended — DECISIONS 84, Loop 3344 — 6071 on its
    first try, by the measure on `ec062f94`): ours 2 draws against theirs 3 at index 0 — ours `Ammo::init+0xcd9`,
    theirs `Unit::do_move+0xe84`. Promoted by the recipe — `diff::lakota_word` and its `AI_WORDS` pin, the lobby
    leaving `BATTERY`, a replacement minted (run 979, by entry 72's rules in order, the booking saying which) — **its
    widening first**: grep the lobby's start and long dumps (runs 975, 975b) for 6071, a Great Lakes window capture at
    detail only for what they cannot answer (run 980; run75's 5845..6160 is another lobby's game), both directions.
    Lands wide. AI §277, park LA.

3347. **The battery's second open game, `GreatSaharaFrenchTier4` at 13422 of 16764** (entry 72 as amended by 84: minted
    by 3343 under rule (2) — a nation held only by closed games on another map; 13422 on its first try, by the measure
    on `ec062f94`): ours 14 draws against theirs 19 at index 6 — ours `Guy::set_anim+0x97a < Guy::inc_time+0x271`,
    theirs `Guy::init_real+0x52`. Promoted by the recipe — `diff::french_word` and its `AI_WORDS` pin, the lobby
    leaving `BATTERY`, a replacement minted (run 981, by entry 72's rules in order, the booking saying which) — **its
    widening first**: grep the lobby's start and long dumps (runs 977, 977b) for 13422, a Great Sahara window capture
    at detail only for what they cannot answer (run 982), both directions. Lands wide. AI §278, park LC.

3346. **The nations seat's next shape batch** (DECISIONS 63 (iv), Loop 3188; the score `Census:` port 436; 3339 built
    four of six and parked `find_build_spot` and `think_civilian_transport` a second time, 3339a–c, beside 3337a–b's
    two — four shapes now their own rows): `Wall::process`, `Unit::think`, `Unit::do_repair`,
    `Unit::think_carry_civilian`, `Object::disband`, `BuildTypeData::blocked_tcoord` (`census.py --unbacked`, the
    `port` mark) — each built against the crate's own method where it has one, or parked naming the generator flag it
    could not spell. `--kind sweep`, `--no-widen`, park LB.

3348. **The second sweep lane's first shape batch** (DECISIONS 84 (i), Ramon's word; the score `Census:` port 436;
    the next six `port`-marked rows after 3346's on `census.py --unbacked --all --top 4000`): `Unit::do_group_move`,
    `Roads::add_roads`, `Unit::find_path`, `Unit::work`, `Army::find_muster_spot`, `Unit::target_opportunity` — each built
    against the crate's own method where it has one, or parked naming the generator flag it could not spell; the brief
    names 3346's worktree for `--claimed` (Loop 2926) so the two sweeps never build one function. `--kind sweep`,
    `--no-widen`, park LD.

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
