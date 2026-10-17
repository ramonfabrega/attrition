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

*2026-10-15, **the thirty-seventh pass (steer, Fable) after the thirty-seventh tranche: 23 landings from `23e2df12`** (DECISIONS
74), four lanes (Opus 5.5 medium on the AI word, Sonnet 5.5 medium on the rules track and two sweeps): 4.22 USD and **36.5 minutes
waiting a landing** against 3.92 / 27.0 — a fifth broken, so **the fourth lane goes** (entry 61 §5); every gate green, no kill rule, one `DEEP`.*

- **Words**: the peace pair (East Indies, the Lakota, `RUSH_RULES 10`, ends 6822) 5564 → 5982 (2780: the scout's fog read takes the Space Program), the open word.
  Chapter 80 closed (1204, run878: a fight blows a stealth unit), 81 compared the blown bits (2798); 69 built, 70 surveyed. Census 1068 → 1164, `port` 171 → 156. Battery 576 / 6566
  / 2974; **when the peace pair closes, the AI word's successor is its lowest lobby** (DECISIONS 72).
- **Gates (DECISIONS 72, 73)**: the booking gate `--scope <last whole gate's tip> --test-threads 8`, whole every third landing
  and at the drain; a whole gate refuses a stale `agent-*` worktree (Loop 2724). The sweep's per-function check runs in debug (2787).
- **Lanes: four, live from `a4ff472f`** (DECISIONS 75: the lane entry 74 took goes back for the trial): 2779 (AI §234,
  EZ, run 879), 2804 (GOLDEN §105, FD, run 880), 2801 (FC) and 2793 (EY) on the two sweep lanes; 2783, 2780, 2782, 2785, 2797, 2798 landed. Next 2805, run 881 (Astra 900–909). Pools: GOLDEN §106–109, AI §235–239, park FE–FI.
- **Fable backlog: 13 Loop items** (1570, 1725, 2003, 2518, 2654, 2750, 2763, 2769, 2775, 2788, 2792, 2796, 2799).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 1850/1850 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w18140 of 18,140 · GreatLakes w5930 of 5,930
Third pair: EastIndiesFrench w17379 of 17,379 · GreatLakesFrench w5638 of 5,638
Coverage pair: EastIndiesPersianAllTech w4730 of 4,730
Successor pair: EastIndiesMongolAllTech w5056 of 5,056
Peace pair: EastIndiesLakotaPeace w5982 of 6,822
Census: simulation backed 1164 of 3611
Third map: GreatSahara w24000 of 24,000 · GreatSaharaToughest w15432 of 15,432 · GreatSaharaPersianAllTech w4340 of 4,340 · GreatSaharaMongolAllTech w3268 of 3,268
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · ch25 closed · ch26 closed · ch27 closed · ch28 closed · ch29 closed · ch30 closed · ch31 closed · ch32 closed · ch33 closed · ch34 closed · ch35 closed · ch36 closed · ch37 closed · ch38 closed · ch39 closed · ch40 closed · ch41 closed · ch42 closed · ch43 closed · ch44 closed · ch45 closed · ch46 closed · ch47 closed · ch48 closed · ch49 closed · ch50 closed · ch51 closed · ch53 closed · ch55 closed · ch56 closed · ch57 closed · ch59 closed · ch61 closed · ch62 closed · ch63 closed · ch64 closed · ch65 closed · ch67 closed · ch68 closed · ch71 closed · ch72 closed · ch73 closed · ch74 closed · ch75 closed · ch76 closed · ch77 closed · ch78 closed · ch79 closed · ch80 closed
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked · GreatSahara 0 off, 0 unlinked

**Opener: the thirty-eighth tranche (commander, Opus): four lanes from the trial's gate commit (DECISIONS 75) — 2780 on
the AI word (Opus 5.5 `--effort medium`), 2785 on the rules track, 2782 and 2783 on the two sweep lanes (Sonnet 5.5
`--effort medium`); the whole gate green at the trial's commit; count landings from that commit, stop at twenty, and clear
`steer` into the thirty-eighth pass with the handoff written and every gate's `Admission wait:` line in it.**

## Astra's branch, on trial for the thirty-eighth tranche

PR #10 (`codex/astra-windowed-reader`, 72 commits) fast-forwarded on 2026-10-17 after the steer's review (DECISIONS 75;
the branch's own ledger moved to `journal/2026-10-17-pr10-adoption.md`, its design is `TEST_PERFORMANCE.md`). Kept whole:
the windowed capture reads, the three on-disk caches beside the captures, the test-body and sweep splits, the isolated
rondata runner. Adapted: the host budget is 32 slots / 96 GiB, the reflex guard reserves 4 GiB, an unknown test 8 GiB,
the whole gate runs the caches' verifiers, and `touch ~/Library/Caches/attrition/admission-off` turns admission off for
every worktree at once. **The trial's two numbers**, read at the thirty-eighth pass: waiting a landing (`tools/tranche.py`)
against 36.5 on four lanes, and the gates' `Admission wait:` line — a rise with the admission wait near zero is the box's
cost and the fourth lane goes again; a rise with it high is the admission's and the marker is the answer.
Whole gate green on `946393a4`: 6 of 6, 422 s at eight threads, rondata 900 in 275 s with the verifiers on (128 s without, at
four, on the branch's tip), 4080 fixture requests and none missing, admission wait 3.9 s over 917 leases.

## The queue

In dependency order, headline-nearest first. **The roster is Ramon's**:
2779 the AI word's (Opus), 2804 the rules track's, 2801 and 2793 the
two sweep lanes' (Sonnet) — lower map first — East Indies (Lakota), the
peace pair's only map. Take the first unstarted item unless a better
order is clear, and say why. The backlog is `docs/PARKED.md`.

2779. **The peace pair's word 5982: ours 37 draws, theirs 287, index
    5** (2780; ours `Leader::produce_building+0x1805`, theirs `Build::
    find_gather_tiles+0x10a`). No dump of this lobby holds 5982 (run867
    ends 5657, run847 is checksums and the close), so run 879 first:
    run867's recipe, window 5976..6232 (`--end-frame 6238`), `BUILDS=7`
    in the detail; then the widening, every record on 5982, both
    directions, `WIDENINGS` naming it. AI §234.

2804. **Chapter eighty-two: `do_cast`'s blow held on a stealth caster**
    (DECISIONS 56 §3; 2798 compared the blown and fought bits, both
    clears diff-backed). `do_cast@005ebfe0` writes `0x11000` too and
    this crate's casters do not (parked 2760): stage a stealth unit that
    casts in a who=1 scout's sight, one that does not as control, and
    compare `unit_masks` across the cast. Census rows first; run 880
    only when no packet answers. GOLDEN §105.

2801. **The sweep's hundred-and-eleventh batch: ten never-backed
    simulation functions, a shape the parked rows name first** (DECISIONS
    63 (iv); the score `Census:` 1164 of 3611): a `--jump` over an
    indexed jump's arms (`Search::valid_filter`, 2800), then 2791's
    import clock (`timeGetTime`) and register arguments — each shape
    built with a test made to fail first, then what it and slot EY hold,
    each checked against `census.py --never`. `--kind sweep`.

2793. **The sweep's hundred-and-ninth batch: ten never-backed simulation
    functions a built shape already holds** (DECISIONS 63 (iv), 73; the
    score `Census:` 1151 of 3611): 2783's harvest (ten of ten), off
    `tools/census.py --unbacked`, the `port`-marked rows first, each
    generated by `sweep_gen.py` as it stands — no generator change; a
    function no built shape holds parks under slot EY for 2782's lane.
    The two sweep lanes share `census.rs`'s pin, the commander's at
    each booking (Loop 2367). `--kind sweep`.

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
