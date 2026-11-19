# The pass of 2026-11-19: the seventh game ratified, and a game is declared before it is played

*2026-11-19, from 09:13 −05:00, the `steer` row on Fable 5.2, called by
the handoff. `commander` (Opus 5.5) handed over at `1940ba9d5` after
thirteen first-parent merges of thirteen lanes (4396 to 4405, 4410 to
4412) and Codex's 4406 to 4409 fast-forwarded between them, drained and
gated whole (`gate-drain-1119.log`). The charter was the handoff's:
ratify item 4404's seventh game in parity's set and design four empty
seats. Decisions in entry 144. The pass spawned no subagent and took no
capture. The pass before it is `2026-11-18-fable-pass-milestone.md`.*

## For Ramon

- **The seventh game holds.** The small tribute game, which the original
  played until the French conquered the idle human, agrees with ours on
  all twelve checksums on all 10,929 states. I broke it from both sides
  and it went red on the right state each time.
- **One gap in it, now pinned.** The original prints no checksum after
  the tick that ends a game, so the conquest itself was compared nowhere.
  A new test holds the one number both sides give: the original stops
  after frame 10928 and ours is over on the next state, French win.
- **The rule I wrote yesterday for joining the set was flawed.** A game
  that joins only once it agrees can only raise the fraction. From now a
  game is named before the original plays it and stays in the count.
- **A new number is booked: parity held out.** Eight lobbies picked by a
  fixed rule, never tuned for, each recorded here and played by the
  original, scored on the first try. It is on its own line. **Yours:**
  whether it folds into the headline's set.
- **The lake has no seat.** Three takes of one frame agree on 3.8 % of
  it, so nothing there can be scored. That worker is on the packets.
- **A live game shows no city names.** I drew one. The third seat ports
  the founding name so the plate appears in play.
- **Seven lane rows are not reaped.** `ccc rm` was refused by the
  auto-mode classifier. All seven are merged; their trees hold 55 GB and
  the disk is at 98 GB free, from 183 yesterday. The line to run is in
  "Not done".
- **The week:** 74 % at 09:58 against today's line of 84; thirteen
  landings for 84.32 USD at list, 6.49 a landing. Fable's week 46 %.
- **The pictures:** `https://cdn.ramonfabrega.com/attrition/pass-1119/`,
  five files: the two pairs, the selected city four ways, and the live
  lobby at 6,000 with the capital and with a Citizen selected.

## What was measured

| what | measured |
|---|---|
| the drain's gate (`gate-drain-1119.log`) | `gate exit 0` on `1940ba9d5`, the tip, HEAD = origin/main; whole, 9 of 9 steps, 24.5 minutes, release 20.7; no `skipping:` line; `Suite: 1413 processes for 2542 tests` |
| the merges (`git log --first-parent --merges 997c01993..1940ba9d5`) | 13, of thirteen lanes; Codex's four items are fast-forwards between them |
| what they moved | nine moved a score (4396, 4397, 4400, 4401, 4404, 4405, 4410, 4411, 4412); 4398, 4402 and 4403 minted a row or named a leaf on their first act; 4399 stopped with nothing |
| the lanes (`tools/tranche.py --base 997c01993`, after `lore index`) | 13 sessions, 84.32 USD, 6.49 a landing; a session's clock 63.5 minutes, 17.6 working and 45.9 waiting, 32.7 on its own gate; two past 300 k (4397 at 319 k, 4400 at 306 k); 5 captures |
| the gates (`tools/gates.py … --since gate-pass-1118b.log`) | 12, 235.7 minutes: 3 whole, 9 scoped (7 widened), 1 red (`gate-4402-4405`, two minutes, green on its second); admission 8,024.4 s summed, longest 581.1 s; the whole gate 30.1 minutes on 2026-11-18 and 24.5 after Codex's 4407 to 4409 |
| the seventh game, un-mutated (`the_demand_s_game_to_its_end_on_run1691`) | green, 54.3 s: 10,929 states, twelve lines at 10,929 each, no first parting, ours not over to 10928 |
| theirs bent (a copy of run1691's take, one bit of the ammo sum on state 10177) | refused: `checksum set total disagrees at trace frame 10176 ordinal 0`. With the set's total moved to match: **red**, ammo 10,928 of 10,929, first parting 10177, ours 1314070668, theirs 1314070669 |
| ours bent (`tools/mutate.py`, the cadence's read of `tribute_demanded` removed) | **held**, exit 101: leaders 9,985 of 10,929, first parting 9985, ours 2254783755, theirs 705512842 |
| the three functions, at the address | `Ammo::hit_target@00678f90`, `Ammo::check_hit@00678d90` and `Ammo::do_damage@00678060` read in the export: each writer of `whom`/`ox` is as `docs/COMBAT.md` §46.9 says; `check_hit` also writes a building it finds |
| the end tick (`the_demand_s_game_ends_on_the_tick_the_original_stops_on`, new) | green, 1.0 s: theirs' last frame 10928, ours over on state 10929, `defeated_by` 1, `won` true. With `check_victory` held off to 10940: **held**, ours never over to 10993, `won` false |
| the week (`lore limits`, 09:58) | 74 % (the day's line 84), from 67 at the pass before; 25.32 USD a point all-in; Fable's week 46 %, from 44 at the pass's start |
| the suite re-timed (`suite_timings.py --stale-only`) | 9 tests timed: 2,542 tests, 5,492 s one at a time |
| the disk (`df -g ~`) | 97 GB at the start, under the last pass's 183; 102 after `afsctool` over both archives; 98 at the record. The `att-*` trees hold 55 GB, Codex's tree 14, the gate tree 28, this tree 27 |
| the stills (`RON_STILL_DIR`; `the_ground_on_the_client_s_still`, the three `selection::` tests; `client --lobby --frame 6000 --select …`) | run1564 and run1559b at their photographs' cameras, the selected city's band, plate and bar, and the minted lobby at 6,000, under `~/ron-data/lab-experiments/2026-11-19-pass/` |
| the rubric (`rubric.py --record`) | every figure the pass before's: 737 ported, 331 replaced, 572 open, 1,457 of 3,611 backed, 44 of 112 groups whole, ages 8 of 8, unit types 296 of 302, lockstep bytes 78.1 % |
| the rows and trees left | seven merged lane rows and eight `att-*` trees: `ccc rm` refused by the classifier. `reap_agents.py --yes` printed nothing. The old driver row reaped by `handover.py reap 7abde85d` |

## Rulings

- **Item 4404's game is ratified, parity's seventh** (entry 144 §1), with
  its end tick pinned and what follows that tick named as uncompared.
- **A game is declared before the original plays it** (§2); parity held
  out is booked on its own line (item 4413).
- **The suite's seat stays on the sample and takes its ranges once** (§3,
  item 4414).
- **The lake gives up its seat** (§4); its worker takes the packets'
  guys line (item 4415), the units line booked behind (item 4417).
- **The third seat names a city** (§5, item 4416).
- **Marked rows:** no `FABLE:` marker and no second reading in the
  tranche.

## What the pass looked at

The two pairs at the tip, the selected city at 1:1 four ways (theirs,
ours' band, ours' plate, ours' bar), and the minted lobby at 6,000.

- **The city (run1559b).** The band, the plate and the bar are drawn
  where theirs are. The band is wider than theirs and sits outside the
  paved base, on the grass; theirs is a thin line on the base's edge. The
  plate's name is a blocky stand-in, capitals spaced apart, where theirs
  is a serif face. Theirs still has what ours lacks: trampled marks on
  the grass to the west, a pale beam over the flag-bearer, and two
  vehicles drawn faint where ours are solid.
- **The lake (run1564).** As the pass before saw it. Ours has crisp
  white arcs at the west and east rims and a paler body; theirs has a
  soft band inside the south shore and bright flecks in it.
- **The live lobby at 6,000.** The capital, its buildings, the Citizens,
  the trees and the mountain are drawn and lit. Selected, the capital has
  its band and no plate. The HUD is text: the resources on one line, the
  queue's bar, eleven buttons.

## The queue's items

| item | kind, mark | the score | first act | it stops and says so when |
|---|---|---|---|---|
| 4413 parity held out | rules, whole; the chain's | a new line: of eight lobbies named before any take, the ones whole on twelve lines on the first try, and each one's first parting | the eight named from the rule, row 0 recorded by the client and played by the original for 6,000 frames | the original refuses a file the client recorded on a lobby, or the lane is not free for an hour |
| 4414 the sample, from units 532 | rules, whole; the suite's | `SAMPLE_LINES`' lowest first parting, 532 | units `905 6 guys.length` (4 against 1) in run1697's window, on disk; then one ranges take to 3,100 | two acts move no line's first parting |
| 4415 the packets' guys line | rules, whole; the render seat's | `PACKETS_SCORE` lines 690 of 1,053 | `cur_time` (byte 108, first parting on 17 lines, ours one behind): the writer by a `step4.py` watch on a packet | two acts turn no line |
| 4416 a city's founding name | render, scoped, touches `crates/sim`; the third seat's | a row minted first: each city's name on the packets against ours; then the plate in a live game; then the band's red on theirs' 1,477 | `CityData +0x90` read off one packet beside ours' `None` | the snapshot tools cannot reach a city's record |
| 4417 the packets' units line | rules, whole; unseated | `PACKETS_SCORE` lines | a census of `id..march` (44 lines) and `collide_frame..play` (42) by first differing leaf | — |

## Built

- `the_demand_s_game_ends_on_the_tick_the_original_stops_on` and
  `RUN1691_END` in `crates/rondata/src/diff/start_walk.rs`, seen red.
- `tools/repin_packets.py` and `tools/test_repin_packets.py` (six tests,
  in the offline step; a copy with the count bent failed four). On the
  real table it reads 690 of 1,053, the pinned score.
- The frame's re-pin row and the checklist's, with the two flags a
  packets census needs.
- The paperwork: entry 144, this record, `docs/LEADERS.md` §8.6, the
  queue's handoff and five items, the tracks' paragraph, the journal's
  entry, the suite's timings, the rubric's row.

## Not done, and said

- **The lane rows were not reaped.** The line, for whoever may run it:
  `for r in ac14173f 91372c1f ea4796b5 26458a36 eb95a135 63e33b1b
  885d7daf; do ccc rm $r; done`. `att-4385`'s tree has no row and stands
  too.
- **Parity's count is still prose**, as at the two passes before. Item
  4413's line is the first that will be parsed.
- **The walls line has no emitter**, and no game on disk holds a wall.
  The held-out lobbies may be the first to build one.
- **The driver's own cost is not summed**: `lore loop commander` names
  the row as it stands since the wake (5.97 USD), not the evening's.
- **The blind list was not scanned**: open 0.
- **The debug profile was not cleaned**: the reflex needed it before the
  gate, and it is 10 GB of the 55 the lanes' trees hold.

## The gate

Whole, by `tools/gate_run.sh`; its verdict is quoted in the commit after
this one.
