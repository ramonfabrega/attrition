# The pass of 2026-11-18, second: the milestone closes, and parity is the headline

*2026-11-18, from 17:26 −05:00, the `steer` row on Fable 5.2, called by
the handoff. `commander` (Opus 5.5) handed over at `1e0e1fdfe` after
sixteen first-parent merges of fourteen lanes (4382 to 4395), drained and
gated whole (`gate-drain-1118b.log`). The charter was the handoff's:
ratify the milestone's step 5, say what its close makes the headline, and
place four empty seats. Decisions in entry 143. The pass spawned no
subagent and took no capture. The pass before it is
`2026-11-18-fable-pass-astra.md`.*

## For Ramon

- **The milestone is closed.** A game started here, with orders minted
  here, is played by the original and agrees with it: on every draw of
  24,000 frames, and on all twelve state checksums on every one of those
  frames. Step 5 was the last piece and I ratified it after breaking it on
  purpose: with the last held field computed wrong, the comparison goes
  red on the exact frame.
- **What I could not confirm, said plainly.** That field has one writer, one
  eraser and one reader. The game that released it exercises the writer
  only. The other two rest on two independent readings from a week ago.
- **The headline is now parity**, the number you named on 2026-11-13: 5 of
  6 games agree on every checksum. The sixth is the 2017 recording of a
  person against four computers.
- **Our sim says the tribute game ends.** Left alone, the French declare
  war on frame 7937 and conquer the idle human on 10929. The next item has
  the original play that file to its end. If it agrees, it is the first
  whole game against the computer, start to defeat, in the set.
- **The lake cost three lanes and moved nothing.** The original's lake
  differs between two takes of the same frame, so its streaks cannot be
  scored against a photograph. The seat now scores only what both takes
  agree on.
- **A selected city shows no health bar, no name plate and no red outline
  in ours.** I looked at the pair. A seat takes those, since playing comes
  before the picture.
- **The week:** 67 % at 17:26 against today's line of 70; the tranche took
  six points for fourteen landings, 99.33 USD at list. Fable's week 39 %.
- **The pictures:** `https://cdn.ramonfabrega.com/attrition/pass-1118b/`,
  five files: the three pairs, the city at 2:1 and the lake on two takes
  over ours. The upload ran unrefused.
- **Yours:** whether the tribute game joins parity's set as a seventh once
  it is played whole (I ruled yes, yours to overturn); ten minutes playing
  the client, still the best measure nobody has taken.

## What was measured

| what | measured |
|---|---|
| the drain's gate (`gate-drain-1118b.log`) | `gate exit 0` on `1e0e1fdfe`, the tip, HEAD = origin/main; whole, 9 of 9 steps, 29.2 minutes, release 24.1; 0 `skipping:` lines; the `FAILED` lines are the gate's own fixtures; `Suite: 1404 processes for 2533 tests` |
| the merges (`git log --first-parent --merges 8bf3adde2..1e0e1fdfe`) | 16, of fourteen lanes, 13:29 to 16:52 |
| what they moved | eight moved a score (4383, 4385, 4387's game, 4388, 4391, 4392, 4394, 4395); six moved none (4382, 4384, 4386, 4389, 4390, 4393) |
| the lanes (`tools/tranche.py --base 8bf3adde2`, after `lore index`) | 14 sessions, 89.38 USD, 6.38 a landing; a session's clock 52.8 minutes, 16.5 working and 36.3 waiting, 25.0 on its own gate; two past 300 k (4394 at 303 k, 4395 at 323 k); 6 captures |
| the driver (`lore loop commander --since` the last pass) | 9.95 USD; with the lanes, 99.33 |
| the gates (`tools/gates.py … --since gate-pass-1118.log`) | 9, 192.6 minutes: 1 whole, 8 scoped (7 widened), 1 red (`gate-4388-4391`, a client pin 4391's walk moved, mended in its second commit); admission 9,501.4 s summed, longest 659.0 s |
| step 5's tests in that gate | `the_leaders_line_is_the_original_s_range_by_range`, `the_ported_lines_are_checksum_state`, `our_lines_part_from_the_hook_s_on_their_first_frame` and the four `the_demand_s_` tests all ran and passed |
| the pass's mutations (`tools/mutate.py`, on `1c1b11207`, against `the_demand_s_game_is_whole_on_run1683`) | un-mutated: green, 28.8 s. The demand's store written 0: **held**, leaders parts on 6081, ours 2843059440, theirs 411149553. `action_respond`'s clear removed: **failed nothing**. The cadence's read removed: **failed nothing** |
| the demand's game past its take (a scratch walk in our sim, reverted) | `tribute_demanded[0]` 0 → 1 on 6081; war both ways on 7937; the human `defeated_by` 1, type 0, and the French won, on 10929; 0.5 s |
| the week (`lore limits`, 17:26) | 67 % (the day's line 70), from 61 at the pass before; 25.94 USD a point all-in; Fable's week 39 % |
| the suite re-timed (`suite_timings.py --stale-only`) | 12 tests timed: 2,533 tests, 5,433 s one at a time; the slowest new one `the_demand_s_game_is_whole_on_run1683`, 31.0 s |
| the disk (`df -g ~`) | 168 GB at the start, under the last pass's 194; 170 after `afsctool` over both archives |
| the stills (`RON_STILL_DIR`, the three tests of the pass before) | run1564, run1559b and run1665's arms at their photographs' cameras; three half-size pairs, a 2:1 crop of run1559b's city and the lake on two takes over ours, under `~/ron-data/lab-experiments/2026-11-18-pass-2/` |
| the rubric (`rubric.py --record`) | territory ported 736 → 737 and simulation backed 1,456 → 1,457 of 3,611 (item 4394's `update_all_seen`); every other figure the pass before's: 331 replaced, 572 open, 44 of 112 groups whole, ages 8 of 8, unit types 296 of 302, lockstep bytes 78.1 %; `RUBRIC_TERRITORY_PORTED` and the queue's `Rubric:` line re-pinned |
| the rows and trees left | no lane row (`ccc list`); `reap_agents.py --yes` printed nothing; the old driver row reaped by `handover.py reap 236ed36b`; the `att-4385` tree is merged and stands for the gate's disk step |

## Rulings

- **Step 5 is ratified and the milestone is closed** (entry 143 §1), with
  three limits named.
- **The headline is parity on the declared set** (§2).
- **The chain's seat plays the demand's game to its end** (§3, item 4396).
- **The suite's seat stays on the sample, ascending by state** (§4, item
  4397), and the packets' lines are booked behind it (item 4400).
- **The third seat draws what a selected city shows** (§5, item 4398).
- **The render seat scores the lake on what two takes agree on, and no
  row is minted on its streaks** (§6, item 4399).
- **Marked rows:** no `FABLE:` marker and no second reading in the
  tranche.

## What the pass looked at

The three pairs at the tip, the city at 2:1, and the lake from both takes
of the original over ours (`lake-two-takes-and-ours.png`).

- **The lake.** Both takes of theirs carry a wide, soft band of foam
  flecks inside the whole south shore and a darker, mottled body. Ours
  draws crisp white arcs at the west and east rims, which neither take
  has, a thin south band, a paler body, and a bird. The streaks differ
  between the two takes; the rim, the band's width and the body's tone do
  not.
- **The city (run1559b).** The land, the trees, the mountain, the
  buildings and the fire read alike. Theirs has what ours lacks: a red
  line round the selected city's plaza, a name plate under it, a health
  bar over one building, red trim along three roof edges, trampled marks
  on the grass to the west, and two vehicles drawn faint where ours are
  solid.
- **The border (run1665)** was drawn and not read again: 4385 landed its
  row by value.

## The queue's items

| item | kind, mark | the score | first act | it stops and says so when |
|---|---|---|---|---|
| 4396 the demand's game to its end | rules, whole; the chain's | the demand's game's states whole, 6,401 → its end; the leaf's clear and reader seen red | the same lobby minted to 11,200 frames, played by the original under the hook, scored on twelve lines | the playback is refused, or two acts move the first parting by nothing |
| 4397 the sample, ascending | rules, whole; the suite's | `SAMPLE_RANGES` state 0: 3 → 0; then `SAMPLE_LINES`' lowest first parting | builds' `z` on two Farms, then leader 2's `oil_patches` order | two acts move no line's first parting |
| 4398 what a selected city shows | render, scoped; the third seat's | a row minted on run1559b's photograph: the plaza's outline, the bar, the plate | the still drawn with `--select` at the photograph's camera and the row minted at what it reads | the marks are drawn by an `iface*` window on the cut list |
| 4399 the lake on what two takes agree | render, scoped | a row over the lake's pixels where run1564 and run1688 agree | the stable set minted with no capture, seen red on a perturbed side | the stable set is under a quarter of the lake |
| 4400 the packets' lines | rules, whole; unseated | `PACKETS_SCORE` lines 608 of 1,053 | a census of the 445 parting lines by first differing leaf | — |

## Built

- Two stale comments mended in `crates/sim/src/leaders/` (the leaf "is
  held", the demand arm "is not carried"). No behaviour changed.
- The paperwork: entry 143, this record, `docs/LEADERS.md` §8.6, the
  queue's handoff and five items, the tracks' paragraph, the journal's
  entry, the suite's timings.

## Not done, and said

- **No guard was built.** One thing was built twice again (4388 and 4392,
  `Leader::init`'s row on unseated slots); the lane gate's `Base moved:`
  refusal caught it after the build. A rule written as prose would not
  hold, and the pass found no cheap check that would have.
- **The walls line has no emitter.** `empty_walk` refuses a walk that
  holds a wall, and no game on disk holds one.
- **Parity's count is still prose**, as at the pass before.
- **The two walkers of a recording are still two.**
- **The blind list was not scanned**: open 0.
- **The debug profile was not cleaned**: the reflex needed it before the
  gate.
