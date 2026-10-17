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

*2026-10-17, **the thirty-eighth tranche (commander, Opus): 11 landings from `a4ff472f`**, every booking gate green
after its second commit at most (one: the Golden line, Loop 2799); last whole gate `05718f24`, last gate `6d760eba`; two `DEEP`
lanes cleared (2779, 2801), no kill rule. 2793 waited 1h50 on a subagent that ended its turn (Loop 2820), told to land without it.*

- **Words**: the peace pair (East Indies, the Lakota, `RUSH_RULES 10`, ends 6822) 5564 → 5982 → 5985 (2780: the scout's fog read takes the Space Program; 2779: Capitalism's oil), the open word.
  Chapter 80 closed (1204, run878: a fight blows a stealth unit), 81 compared the blown bits (2798), 82 built `do_cast`'s blow from the listing (2804), 83 carried Bribe's target arm (2812); 69 built, 70 surveyed. Census 1068 → 1181, `port` 171 → 156. Battery 576 / 6566
  / 2974; **when the peace pair closes, the AI word's successor is its lowest lobby** (DECISIONS 72).
- **Gates (DECISIONS 72, 73)**: the booking gate `--scope <last whole gate's tip> --test-threads 8`, whole every third landing
  and at the drain; a whole gate refuses a stale `agent-*` worktree (Loop 2724). The sweep's per-function check runs in debug (2787).
- **Lanes: four, live from `a4ff472f`** (DECISIONS 75: the lane entry 74 took goes back for the trial): 2808 (AI §235,
  FF), 2823 (GOLDEN §107, FI, run 880), 2818 (FH) and 2793 (EY) on the two sweep lanes; 2783, 2780, 2782, 2785, 2797, 2798, 2801, 2779, 2804, 2807, 2812 landed. Next 2825, run 881 (Astra 900–909). Pools: GOLDEN §108–113, AI §236–239, park FJ–FO.
- **Fable backlog: 17 Loop items** (1570, 1725, 2003, 2518, 2654, 2750, 2763, 2769, 2775, 2788, 2792, 2796, 2799, 2809, 2819, 2820, 2824).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 1850/1850 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w18140 of 18,140 · GreatLakes w5930 of 5,930
Third pair: EastIndiesFrench w17379 of 17,379 · GreatLakesFrench w5638 of 5,638
Coverage pair: EastIndiesPersianAllTech w4730 of 4,730
Successor pair: EastIndiesMongolAllTech w5056 of 5,056
Peace pair: EastIndiesLakotaPeace w5985 of 6,822
Census: simulation backed 1181 of 3611
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
**The tranche's admission ledger** (ten landings): lane gates 2.1–12.5 s each over 380–919 leases, longest 1.2 s
(2785's first gate 74 s, one 65 s lease); booking gates 41.0 s (`2280648b`, longest 32.6), 18.9 (whole), 3.5, 2.3, 10.4
(whole), 2.1, 7.0, 10.6 (whole), 2.8; the one admission failure was the probe race `a8883529` fixed (exit 125, Loop
2788); the marker was not used.
Whole gate green on `946393a4`: 6 of 6, 422 s at eight threads, rondata 900 in 275 s with the verifiers on (128 s without, at
four, on the branch's tip), 4080 fixture requests and none missing, admission wait 3.9 s over 917 leases.

## The queue

In dependency order, headline-nearest first. **The roster is Ramon's**:
2808 the AI word's (Opus), 2823 the rules track's, 2818 and 2793 the
two sweep lanes' (Sonnet) — lower map first — East Indies (Lakota), the
peace pair's only map. Take the first unstarted item unless a better
order is clear, and say why. The backlog is `docs/PARKED.md`.

2808. **The peace pair's word 5985: ours 23 draws, theirs 24, index
    19** (2779; ours `Guy::set_anim+0x97a < Guy::inc_time+0x271`, theirs
    `Guy::set_anim+0x104b`). East Indies run879 (5976..6232) holds it, so no capture
    first: the widening is every record on 5985, both directions,
    `WIDENINGS` naming it, then the Transport Freighter `1/225` (idle in
    ours from 5976, 2779's hypothesis) walked back. AI §235.

2823. **Chapter eighty-four: Bribe's conversion built, then the cloaked
    Spy's Bribe staged** (DECISIONS 56 §3; 2812 carried Bribe's target
    arm in `cast.rs`; Sabotage's caster is a Commando, not
    cloaked). Build `cast_bribe`'s unit arm and `Unit::swap_team@006118d0`
    with the leader counter at `+0x8f4` (parked 2821), then stage a
    cloaked Spy bribing in a who=1 scout's sight, one out of sight as
    control, and compare `unit_masks` and the owner across the cast.
    Census rows first; run 880 only when no packet answers. GOLDEN §107.

2818. **The sweep's hundred-and-thirteenth batch: ten never-backed
    simulation functions, a shape the parked rows name first** (DECISIONS
    63 (iv); the score `Census:` 1181 of 3611): a hooked callee's answer
    in `(eax, ecx)` (2816), then the register pair the `project`
    overloads take (2817) — each shape built with a test made to fail
    first, then what it and slot EY hold, a function with a simulation
    effect before a renderer body (Loop 2819), each checked against
    `census.py --never`. `--kind sweep`.

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
