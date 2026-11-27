# The pass of 2026-11-27: a row's leaf is read with a kept printer, a row that holds sums is taken by the pass, and one seat carries two rows

*2026-11-27, from 03:38 −05:00, the `steer` row on Fable 5.2, the
backstop's opener. `commander` (Opus 5.5) handed over at `aad846fa0`
after twenty-three first-parent merges off the pass of 2026-11-26
(items 4737, 4735, 4671, 4739, 4693, 4742, 4743, 4740, 4738, 4741 with
4744, 4748, 4746, 4750, 4749, 4752, 4747, 4751, 4754, 4753, 4736, 4745
and 4755, and one re-pin's branch), gated whole
(`gate-backstop-1127.log`), with no lane out. The charter was the
handoff's `For the pass`. Decisions in entry 160. The pass spawned no
subagent and took four captures, run1848 to run1851. The pass before
it is `2026-11-26-fable-pass-floor.md`.*

## For Ramon

- **The headline stands at 52 of 63.** No game came whole in
  twenty-three landings. The thirteen that sat on open games bought
  2,945 agreeing states, 227 each against 74 a tranche ago. The eleven
  open games are 50,335 states short.
- **A fix now helps more than one game.** Two of the thirteen moved
  other games as well, 433 states between them. A tranche ago none
  did. Late-game combat rules are shared, and I would watch this
  number.
- **Two open games stop on the same thing**, a projectile launched one
  unit off. One seat takes both.
- **I took the four full-game captures myself**, while I wrote. The
  game lane is idle during a pass and a capture needs nothing of ours.
  Every open game now has its full capture on disk, and no lane starts
  an item with a quarter of an hour of game.
- **The lanes were copying a 259-line probe from one landing to the
  next and reverting it each time**, sixteen journals' worth. It is a
  kept command now. Two more hand jobs went the same way: writing a
  game's pin, and reading a projectile's fields.
- **One red gate, 37 minutes.** Two fixes crossed at a merge. A guard
  for it would cost more than it saves, so I built none.
- **The client reads as the original across the whole frame now.**
  What is left to the eye is three lines of text at the right
  (`MILITARY`, `Scout: 1`, `Cities: 1`) and two buttons; those are the
  client seat's next two items. The pair:
  `https://cdn.ramonfabrega.com/attrition/pass-1127/`.
- **Yours to do, as before:** 46 merged lane branches stand on origin;
  Astra's two build trees; the two `codex/*` branches that must not be
  merged still make every brief carry a waiver.
- **I ran out of my own context before my gate.** The pass's work is
  committed and the reflex is green, but the whole gate runs on the
  next row, first thing. I read my depth once and too late; the tools
  were worth building, and I should have gated before the captures.
- **The week** read 80 % at 02:17 −05:00, with the reset at 06:00 on
  the 28th. The tranche cost 155.08 USD at list, the driver's 16.86 of
  it, and took five points in eight hours. Fable's read 52 % before
  this pass.

## What was measured

| what | measured |
|---|---|
| the backstop's gate (`gate-backstop-1127.log`) | `gate exit 0` on `aad846fa0`, HEAD = origin/main = origin/worktree-replan-pdb; `Driver: 23 landings since the last pass (backstop 20) · context 337 k` |
| the merges (`git log --first-parent --merges 6726efdc5..`) | 23, 20:03 to 03:04 −05:00: twenty-two lanes' and one re-pin's branch |
| the lanes (`tables-1127-tranche.txt`) | 23 sessions over 23 items, 138.22 USD, 6.01 a landing; a session's clock 62.5 minutes, 13.9 working and 48.6 waiting, 27.0 on its own gate and 13.0 on a suite run; peaks 149 to 317 k, two past 300 (4740 at 312, 4741 at 317); the fourteen row sessions 97.12 USD, the client's nine 41.10; 2 capture sections; the driver's own 16.86 (`lore loop commander --since`) |
| the gates (`tables-1127-gates.txt`) | 13 before the backstop's, 356.9 minutes, all scoped, twelve widened; one red, `gate-4749` (the release step, `the_sixth_held_out_games_score_as_pinned`); admission 27,183.6 s summed, longest 1,187.6 s; four ended with the load past 16 |
| what a seat bought (the booking commits' figures) | thirteen landings on a held-out row, 2,945 states before a first parting: row 53 1,559 (219, 157, 650, 281, and 16 and 236 on other rows' landings); row 88 851 (449, 197, 21, 3, and 181 on another's); row 133 320 (51, 12, 230, 27); row 83 215. 433 of the 2,945 on a row other than the landing's own (items 4746 and 4755) |
| the eleven games not whole (`score_recording` at `aad846fa0`, `pass-1127/score-*.txt`) | every first parting equals its pin: row 83 ammo 4007, row 133 units and guys 4059, row 93 units, guys and world 4288, row 43 six lines 4341, row 88 five lines 4568, row 53 deaths 5275, row 48 ammo 5607, row 108 7421, row 13 7675, row 8 units 9896, the sample units and leaders 15666 |
| the states short of whole | 50,335 over the eleven games: row 93 7,713, row 88 7,433, row 53 6,726, row 48 6,394, the sample 5,378, row 108 4,580, row 13 4,326, row 8 2,105, row 133 2,026, row 83 1,994, row 43 1,660 |
| the leaves on disk | row 83: ammo range 10, record 3, `launch_y` 27377 against 27376. Row 48 (run1848): ammo record 0, `launch_x` 15972 against 15971, record 4's `sz` 588 against 585. Row 133: units range 8918 and guys range 1905, record 50's guy, `node_flags` 15 against 0. Row 93: units range 5079, record 80, `x` 15682 against 15672, `y` 17607 against 17640. Row 88: units range 5331, record 38, `down` 2014 against −1, `order.type` 26 against 7. Row 53: deaths range 3, record 1, eight bytes. Row 43 and the sample: as the pass before. Rows 108, 13 and 8: on run1849 to run1851, in `docs/RUNS.md` |
| the four takes (`pass-1127/takes.log`, `docs/RUNS.md`) | run1848, row 48: 862.1 s, 14,906,950,332 bytes raw, 1.76 GiB filed, first parting 5607 as run1806's sums; run1849 to run1851, rows 108, 13 and 8, in the same script, their figures in `docs/RUNS.md` |
| the drift (`held_out_games_score_as_pinned --nocapture`, 354 s, nine tests green) | eighteen lines on three rows: row 8 nine, row 48 six, row 93 three; re-pinned by `tools/repin_held.py --write` |
| the seam table (`held_seams`, feature `seam-count`, seven tests, 266 s, 56 walks; `tools/seam_table.py`) | five green and two red on the fallback's pin, re-pinned to none on the sixth and the seventh and run again; two hooks no whole row shares, 2,715 and 6,169 states past their rows' first partings |
| the blind list's open row (`tools/trace/report.py <exe> refs`) | `Cities::close_city`: one `call rel32`, from `Build::close`; alive, stays open |
| the disk (`df -g ~`) | 77 GB at 03:40; 83 after the debug profile's clean (12.3 GiB); 86 after the compressor; 71 with run1848 raw and 83 with it filed |
| the archives (`archive_compress.py`, one tree at a time, its output outside the trees) | `Processed 94420 entries`; `scan: 0 damaged, 0 standing`; 83 → 86 GB |
| the still at the tip (`the_minimap_frame_on_run1814b`, `RON_STILL_DIR`) against run1814b's photograph | by eye, below; the rows it pins stand |
| the suite re-timed (`suite_timings.py --stale-only`) | 16 tests timed, the tranche's leaf tests: 2,776 tests, 7,040 s one at a time; the pass's own four tests are untimed, so a lane's gate runs them |
| the rubric (`rubric.py --record`) | every figure the pass before's: ported 739, open 570, lockstep bytes 78.1 %, unit types seen 296 of 302 |
| the week (`lore limits`) | 80 % at 02:17 −05:00; Fable's 52 % then |
| the rows and trees left | the old driver row reaped by `handover.py reap 1790bee2`; no lane row and no builder tree stood; 74 branches on origin, 46 of them merged lanes', three not in the trunk as before |

## Rulings

- **The range printer, the ammo's names, the re-pin and the seam table
  are kept tools** (entry 160 §3), and a pass re-pins the drift at its
  tip.
- **A pass takes the whole take of every open row that holds sums**
  (§4); no item opens on a take.
- **Four seats** (§5, §8): the chain's on 4759 (rows 83 and 48, one
  word), parity seats on 4758 (row 133) and 4733 (row 93), the
  client's on 4761; the unseated in the guard's order, 4571 first.
- **Nothing is built for a pin a merge moves on another row** (§2):
  the price of the guard is more than the red's.
- **The fallback fires in no walk, and no seam is inside the reach**
  (§6).
- **The blind list's row is alive and stays open** (§6).
- **Twenty-nine parked findings** (§7): two booked inside item 4760,
  ten closed in the tranche, the rest stay in their journals.
- **Marked rows:** no `FABLE:` marker and no second reading in the
  tranche; the settlement's six readings and three predicates are owed
  one batch.

## What the pass looked at

One still at `aad846fa0`, run1814b's camera, frame 3294, with the
HUD's art, beside the photograph (`pass-1127/pair/`, uploaded). The
ground, the city, the farms, the groves, the fog and the bottom left
read as they did a pass ago. Across the top ours now has the three
boxes theirs has: the resources at the left with their rates, the
banner `Nubians: Ancient Age` at the centre, the population at the
right. Right of the minimap ours has the red title `Small City --
Napata`, the city's picture, the star, the six figures in their backs,
the dark panel under the crenellated strip to the right edge and the
five small buttons at its end. At this size I could not tell the two
apart there. The figures are a tick apart: 492, 419 and 173 in ours
where theirs reads 491, 418 and 172, and the clock 00:03:39 against
00:03:38. Theirs writes three lines above the bar's right end, in
white with a shadow, `MILITARY`, `Scout: 1`, `Cities: 1`; ours writes
nothing there. Theirs has the idle citizens' button dark and ours has
it lit. A crow stands over the grass left of the farms in theirs and
not in ours. The name plate `Napata` is in the photograph and not in
this still, which the test draws without names; the live client draws
it.

## The queue's items

| item | kind, mark | the score | first act | it stops and says so when |
|---|---|---|---|---|
| 4759 fifth, row 83, and seventh, row 48 | rules, whole; the chain's | both rows' columns from 4007 and 5607 | `print_ranges` on run1784 and run1848 | two landings running move no column |
| 4758 sixth, row 133 | rules, whole; a parity seat's | row 133's column from 4059 | `print_ranges` on run1797 | two landings running move no column |
| 4733 sixth, row 93 | rules, whole; a parity seat's | row 93's column from 4288 | `print_ranges` on run1799 | two landings running move no column |
| 4571, 4756, 4757 rows 43, 88 and 53 | rules, whole; unseated | each row's column | the leaf on run1787, run1846, run1847 | two landings running move no column |
| 4615, 4616, 4579, 4617 rows 48, 108, 13 and 8 | rules, whole; unseated | each row's column | the leaf on run1848 to run1851 | two landings running move no column |
| 4716 the sample | rules, whole; unseated | `Parity:`, units 15666 | player 4's detector bit against both ally masks | two landings move neither word |
| 4761 the military tally | render, scoped; the client's | a new row on run1814b | the window that draws it and its counts | it has landed |
| 4760 the display and idle citizens' buttons | render, scoped; behind 4761 | `_SELECTED` 241,422 and `_MINIMAP_FRAME` 31,065 | as booked | each has landed |
| 4734, 4722 the hoist, the ring's T | render, scoped; behind 4760 | a new row; pieces 480 → 482 | as booked | it has landed |

## Built

- `crates/rondata/src/checksum/playback/window.rs` and
  `crates/rondata/examples/print_ranges.rs`: the range printer. Two
  tests, each seen red.
- `sim::checksum::AMMO_WORDS` and `checksum::fields::ammo_range_fields`:
  an ammo range named by the writer's own table. Two tests.
- `tools/repin_held.py` and `tools/test_repin_held.py`: eight tests,
  seen red; in the offline step's list.
- `tools/seam_table.py`: the seam table from `held_seams`' files.
- `crates/rondata/src/diff/held_out.rs`: the drift of rows 93, 48 and
  8 re-pinned. `held_seams.rs`: the fallback's pins, none.
- The paperwork: entry 160 and its index row, this record, the queue's
  handoff, lanes line and fifteen items, `docs/LOOP.md`'s first acts 9
  and 10 and a clause of the lanes, `docs/RECGAME.md` §15, four
  sections of `docs/RUNS.md`, the tracks' paragraph, the journal's
  entry, the suite's timings, the rubric's row.

## Not done, and said

- **The printer's pairing is a heuristic**: where two sides hold
  ranges of different lengths and no four agree within forty, it pairs
  them as they stand and the lengths say so.
- **Nothing makes a lane use the printer** but its item's words; the
  brief's frame stands eight characters under its ceiling for a
  chapter item and took no row.
- **`repin_held.py` finds a row by its battery's residue and its
  place**; an eighth battery needs its table's name to be the one the
  tool expects.
- **The ammo's names are ours**, the sim's fields, not the type
  record's; the 24 words agree with the original's bytes on every
  state the lines agree on.
- **Why a rule crossed rows in this tranche and not the last** is not
  explained beyond what the two rules are.
- **The six readings that stand alone were not read again.**
- **The still was looked at and not measured** outside the rows the
  test pins.
- **The seam counter does not print a table seam's key** (item 4656's
  note), and the lane's wait and the clock's reds beside load were
  measured and not changed.
- **Row 108's take drops its deaths line**, whole today, as the
  scorer's mask named it; rows 13 and 8 were taken after that was
  seen and keep every line but walls.

## The gate

**Not run by the pass.** `tools/gate_run.sh` refused it: the row read
454 k at its gate, past the 450 k ceiling (`gate-pass-1127.log`, `gate
exit 94`). The pass had read its own depth once, at 441 k, an hour
in, and went on building. The reflex, `tools/guard.sh`, is green on
the pass's commits, with the offline tests of its tools and the tests
it added; the release suite has not run on them. The tip goes to
`commander` by `handover.py to commander --ungated`, and its whole
gate is that row's first act after the reap (`docs/LOOP.md`, the
driver's two lines). A red there is the pass's, on a tree no lane has
left yet.
