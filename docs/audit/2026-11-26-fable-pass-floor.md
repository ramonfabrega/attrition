# The pass of 2026-11-26: a take is refused on its raw peak, a booking is one command, and the client's seat takes the top bar

*2026-11-26, from 18:25 −05:00, the `steer` row on Fable 5.2, the
backstop's opener. `commander` (Opus 5.5) handed over at `34175bb5d`
after twenty first-parent merges off the settlement of 2026-11-26
(items 4692, 4695, 4656, 4691, 4696, 4718, 4717, 4690, 4719, 4694,
4720, 4721, 4724, 4723, 4727, 4725, 4729, 4726, 4730 and 4731), gated
whole (`gate-backstop-1126.log`), with two lanes out. The pass merged
and booked both (4728, 4732). The charter was the handoff's message.
Decisions in entry 159. The pass spawned no subagent and took no
capture. The record before it is `2026-11-26-astra-settlement.md`; the
pass before it, `2026-11-25-fable-pass-takes.md`.*

## For Ramon

- **The headline stands at 52 of 63.** No game came whole in twenty-two
  landings. Thirteen of them sat on three games and every one fixed a
  real rule; together they bought 959 agreeing states, 74 each, against
  207 a tranche ago.
- **Why it slowed is where the three games stand, not how the lanes
  work.** All three part between states 3500 and 4300, where two armies
  first meet, and the next gap there is 3 to 65 states past the last.
  The thirteen rules have nothing in common with each other.
- **The eleven open games are 53,280 states short.** At this tranche's
  rate that is about 720 landings; at the five-tranche average, about
  230. I would not quote either as a date.
- **I found a wrong figure of my own from yesterday and it mattered.**
  I had written that a full-game capture costs 14 GB until a pass
  compresses it. It costs that only while the game plays; filing has
  compressed it for five days. The old rule would have refused the
  captures six of the eleven games now need. A capture is now refused
  by the tool on its real peak (45 GB free for a full game).
- **`commander` lost a queue item to one of its own booking scripts**;
  the ledger caught it thirteen minutes later. Booking is one command
  now, and it refuses an edit that touches any other item.
- **The client: the bottom-left of the HUD reads as the original's.**
  The top of the screen is still empty (resources, age banner,
  population), and so is the selected panel. Those are the client
  seat's next two items, ahead of the raised log. The pair:
  `https://cdn.ramonfabrega.com/attrition/pass-1126/`.
- **Yours to do:** 44 merged lane branches stand on origin again;
  Astra's two build trees hold 17 GB; the two `codex/*` branches that
  must not be merged still make every brief carry a waiver.
- **Six readings from Astra's night still stand on one reading each.**
  No score rests on one alone. I would run the blind second readings as
  one batch after the week resets on the 28th.
- **The week** read 75 % at 18:33 −05:00 on its sixth day, under the
  day's line of 84. Fable's read 50 % before this pass and 52 % at
  18:58, before its gate.

## What was measured

| what | measured |
|---|---|
| the backstop's gate (`gate-backstop-1126.log`) | `gate exit 0` on `34175bb5d`, HEAD = origin/main = origin/worktree-replan-pdb; `Driver: 20 landings since the last pass (backstop 20) · context 276 k` |
| the merges (`git log --first-parent --merges 65b17a724..`) | 22, 12:04 to 18:36 −05:00; two of them the pass's (4728, 4732) |
| the lanes (`tables-1126-tranche.txt`) | 22 sessions over 22 items, 140.49 USD, 6.39 a landing; a session's clock 66.7 minutes, 15.1 working and 51.6 waiting, 27.8 on its own gate and 11.4 on a suite run; peaks 178 to 354 k, three past 300 (4691 at 323, 4723 at 354, 4732 at 315); the thirteen row landings 94.50 USD, the seam's 4.88, the client's eight 41.11; 1 capture section |
| the gates (`tables-1126-gates.txt`) | 12 to the backstop's, 361.8 minutes, eleven scoped and widened and one whole; one red, `gate-4724-4723` (one step, the pins: `queueledger.py` on item 4726); admission 28,422.5 s summed, longest 955.8 s; seven ended with the load past 16; the clock red beside load and green alone on six of eleven, 15.3 minutes |
| what a seat bought (the booking commits' figures) | thirteen landings on a held-out row, 959 states before a first parting: row 93 764 over five (606, 24, 65, 16, 53); row 133 119 over four (8, 5, 57, 49); row 83 76 over four (35, 3, 21, 17). The pivot's seam (4656) took its hook 235 → 0 and moved no column. No unseated row moved |
| the eleven games not whole (`score_recording` at `b34caacad`, `pass-1126/score-*.txt`) | every first parting equals its pin: row 53 leaders 3716 (run1835), row 88 builds 3717 (the pin; run1843 agrees to 3590), row 133 units 3739, row 83 units 3792, row 93 units, guys and world 4288, row 43 six lines 4341, row 48 ammo 5607, row 108 7421, row 13 7675, row 8 units 9896, the sample units and leaders 15666 (run1844) |
| the leaves on disk | row 133: units range 11916, record 122, `damage` 1 against 2, `healing` 5 against 0, `damage_frac` 0 against 4. Row 83: units range 8311, record 90, `orders.count` 1 against 0. Row 93: units range 5079 and guys range 1160, record 80, `x` 15682 against 15672, `y` 17607 against 17640; world range 5329, record 1340, `block.body` 0xe0 against 0xc0. Row 53: leaders range 221, record 1, `encrypt` 0xf6 against 0x92. Row 43: as the pass before, and its groups and leaders fields are row 83's of 3754. The sample: units range 4634, record 110, `damage` 0 against 1; leaders range 271, record 2, bytes 496 and 497, 0 against 0x3d31 (15665) |
| the states short of whole | 53,280 over the eleven games: row 53 8,285, row 88 8,284, row 93 7,713, row 48 6,394, the sample 5,378, row 108 4,580, row 13 4,326, row 133 2,346, row 83 2,209, row 8 2,105, row 43 1,660 |
| a take on disk (`du -sh` against `du -Ash`) | run1844 (never under a pass's compressor): 22 GB apparent, 3.8 GB on disk, its record in 20 pieces of 78 to 129 MB; run1799 13 and 1.6; run1843 3.5 and 0.36 |
| the disk (`df -g ~`) | 56 GB at 18:25 with four lane trees; 80 after three rows reaped; 85 after the debug profile's clean (12.5 GiB); 91 after the fourth row; 93 after the compressor |
| the archives (`archive_compress.py`, one tree at a time, its output outside the trees) | nothing to split; `scan: 0 damaged, 0 standing`; 91 → 93 GB |
| the seam table (`held_seams`, feature `seam-count`, seven tests, 250 s, 56 walks) | five green and two red on the fallback's pin, re-pinned `(133, 1)` and `(88, 2)` and run again, seven green; four hook entries no whole row shares, all 1,809 to 8,205 states past their rows' first partings (`pass-1126/seam-table.txt`) |
| the still at the tip (`the_minimap_frame_on_run1814b`, `RON_STILL_DIR`) against run1814b's photograph | by eye, below; the rows it pins stand: `RUN1814B_3294_LEFT_WINDOW` 61,397 of 61,614, `RUN1814B_3294_MINIMAP_FRAME` 31,065 of 32,483 |
| the suite re-timed (`suite_timings.py --stale-only`) | 9 tests timed, the tranche's leaf tests: 2,760 tests, 6,997 s one at a time |
| the rubric (`rubric.py --record`) | every figure the pass before's: ported 739, open 570, lockstep bytes 78.1 %, unit types seen 296 of 302 |
| the handoff's renderer rows against their pins | two stale: `_GROUND_STILL` read 1,104 where the pin is 1,105, `_RING` hue 21 where it is 20 (item 4690's named fall); corrected |
| the week (`lore limits`) | 74 % at 17:15 −05:00, 75 % at 18:33; Fable's 50 % at 17:39 |
| the rows and trees left | the old driver row reaped by `handover.py reap c9eee68f`; four lane rows reaped by `ccc rm`; no builder tree; 49 branches on origin, 44 of them merged lanes', three not in the trunk as before |

## Rulings

- **A ranges take is refused by the runner on its raw peak above 30 GB
  free** (entry 159 §2); the clause of 60 and the figure of 14 GB "until
  a pass" are withdrawn.
- **A booking's queue edit is `tools/book.py`** (§3).
- **Four seats** (§4, §7): the chain's on 4671 (row 53), parity seats
  on 4693 (row 88, its whole take) and 4735 (row 133), the client's on
  4737; the unseated in the guard's order, 4736 first.
- **Row 43's leaf is row 83's of 3754 again**, and its item says so
  (§4).
- **No seam is inside the mark** (§5).
- **Twenty-two parked findings** (§6): two booked inside items, five
  closed in the tranche, the rest stay in their journals.
- **The states before a first parting are the tranche's measure and
  not a forecast** (§1).
- **Marked rows:** no `FABLE:` marker and no second reading in the
  tranche; the settlement's six readings and three predicates are owed
  one batch.

## What the pass looked at

One still at `b34caacad`, run1814b's camera, frame 3294, with the
HUD's art, beside the photograph (`pass-1126/pair/`, uploaded). The
ground, the city, the farms, the groves and the fog read as they did a
pass ago. At the bottom left the stone bar, the pause button, the clock
(`Normal 00:03:39` in ours, `00:03:38` in theirs), the five pictured
slots among twenty and the minimap's diamond in its frame with four
round buttons stand where theirs do and look the same at this size. To
the right of the minimap theirs goes on: a red title `Small City --
Napata`, the city's picture, a star, six small figures, then a dark
panel under a crenellated strip to the right edge, with five small
buttons at its end. Ours ends at the minimap's frame and the ground
shows through. Across the top theirs has three boxes: resources at the
left (491, 418 and 172 with their rates), the age banner at the centre,
population at the right. Ours has nothing there. Theirs also carries
the name plate `Napata` under the city, a `MILITARY` tally at the right
edge, the raised log, and the idle citizens' button dark where ours is
lit. The white bird a pass ago is not in this still.

## The queue's items

| item | kind, mark | the score | first act | it stops and says so when |
|---|---|---|---|---|
| 4671 sixth, row 53 | rules, whole; the chain's | row 53's column from 3716 | the leaders leaf on run1835 | two landings running move no column |
| 4693 seventh, row 88 | rules, whole; a parity seat's | row 88's column from 3717 | the row's one whole take, `0xffbu` | two landings running move no column |
| 4735 sixth, row 133 | rules, whole; a parity seat's | row 133's column from 3739 | the units leaf on run1797 | two landings running move no column |
| 4736, 4733, 4571 rows 83, 93 and 43 | rules, whole; unseated | each row's column | the leaf on run1784, run1799, run1787 | two landings running move no column |
| 4615, 4616, 4579, 4617 rows 48, 108, 13 and 8 | rules, whole; unseated | each row's column | the row's one whole take | two landings running move no column |
| 4716 the sample | rules, whole; unseated | `Parity:`, units 15666 | player 4's detector bit against both ally masks | two landings move neither word |
| 4737 the top bar | render, scoped; the client's | a new row on run1814b | the three windows' origins and the leader record's figures | it has landed |
| 4738 the selected panel | render, scoped; behind 4737 | a new row; the minimap frame's 31,065 | the window's origin and crops | it has landed |
| 4734, 4722 the hoist, the ring's T | render, scoped; behind 4738 | a new row; pieces 480 → 482 | as booked | it has landed |

## Built

- `tools/book.py` and `tools/explore/test_book.py`: ten tests, seen red
  against the tranche's own end-match; in the offline step's list.
- `tools/explore/unattended_capture.py`: `ranges_peak_gb`, `free_gb`
  and `refused_ranges_disk`, asked before the lane is taken. One test,
  seen red.
- `crates/rondata/src/diff/held_seams.rs`: the fallback's pins,
  `(133, 1)` and `(88, 2)`.
- The paperwork: entry 159 and its index row, this record, the queue's
  handoff, lanes line and fifteen items, two clauses of `docs/LOOP.md`,
  the tracks' paragraph, the journal's entry, the suite's timings, the
  rubric's row.

## Not done, and said

- **The take's refusal sizes the peak by three takes' rate**, a
  constant; a record with more lines kept, or a longer game, is sized
  by the same figure.
- **A take that is not filed stays raw.** Nothing but the item's words
  makes a lane file it.
- **`book.py` does not know the seats' order or mint a number**; the
  guard and the ledger still read the file after it.
- **The forecast is not one.** What decides it is how many rule gaps
  the eleven games hold, and nothing on disk counts them.
- **Why this tranche's rows moved no other row** is not explained; a
  tranche ago two unseated rows moved on a seated row's rule.
- **The six readings that stand alone were not read again.**
- **The still was looked at and not measured** outside the two rows the
  test pins; the regions a pass ago measured were not measured again.
- **The seam counter does not print a table seam's key** (item 4656's
  note).
- **The clock's six reds beside load cost 15.3 minutes and were not
  changed**; nor was the lane's wait, the driver's own cost is not
  summed, and the blind list's one open row was not scanned.

## The gate

Whole, by `tools/gate_run.sh`, once, on the pass's commit; the commit
after it quotes the verdict here.
