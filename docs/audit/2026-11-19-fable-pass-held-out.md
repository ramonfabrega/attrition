# The pass of 2026-11-19, second: parity held out ratified at 0 of 8, and the lanes follow the eight

*2026-11-19, from 18:06 −05:00, the `steer` row on Fable 5.2, called by
the backstop. `commander` (Opus 5.5) handed over at `4623a2e03` after
twenty first-parent merges of sixteen lanes and the chain's builder
(4413 to 4429), drained and gated whole (`gate-drain-1119c.log`). The
charter was the handoff's: rule on item 4413's held-out line and design
the seats the lanes line left empty. Decisions in entry 145. The pass
spawned no subagent and took no capture. The pass before it is
`2026-11-19-fable-pass-seventh.md`.*

## For Ramon

- **On games nobody tuned for, parity reads 0 of 8.** The line is real:
  the eight were named before the first take, the scores re-ran as
  pinned, and the test fails when a score is missing. It is on the
  scoreboard as a parsed line now.
- **Four of the eight are close.** The Ancient starts agree on ten of
  twelve checksums for all 6,000 frames. One line, units, parts and then
  agrees again. If one cause is behind three of them, three games turn
  whole on one fix. The builder's seat is on that.
- **The other four part early and stay parted.** They start in a later
  age with large stockpiles, and the original ends each within 5,000
  frames. In our walk of one, the computer is burning the idle human's
  capital by 3000.
- **A held-out game stops being held out once a lane reads it.** The
  first score stays as the estimate. I named a second eight now (rows
  10, 30, … 150), to be taken when this eight reads 4 whole.
- **Yours:** whether the eight fold into the headline, which would read
  6 of 15. And whether the second eight add seats and a team: of 718
  captures on disk, 688 seat two players and 4 have a team, and the ally
  bug item 4429 found in attrition was reached only through those 4.
- **The client opens every map on Great Lakes' camera.** On East Indies
  that is open sea with your capital off the screen. The eye's seat
  takes it.
- **The handoff's cost line was wrong**, from a bug in my own table
  tool. Right: sixteen lanes, 98.53 USD, 6.16 a landing.
- **The week:** 80 % at 18:30 against today's line of 84. Fable's week
  49 %, from 47 at the pass's start.
- **The pictures:** `https://cdn.ramonfabrega.com/attrition/pass-1119b/`,
  seven files: the two pairs, the selected city at 1:1, and four lobbies
  as the client opens them.

## What was measured

| what | measured |
|---|---|
| the drain's gate (`gate-drain-1119c.log`) | `gate exit 0` on `4623a2e03`, the tip, HEAD = origin/main; whole, 9 of 9 steps, 26.6 minutes, release 23.6; `Fixture audit: 198531 requests, 0 missing fixtures`; `Driver: 20 landings since the last pass (backstop 20)`, context 341 k |
| the merges (`git log --first-parent --merges beee13ad6..4623a2e03`) | 20, of sixteen lanes and the chain's builder (4413, 4416 and 4418 merged twice) |
| the lanes (`tools/tranche.py --base beee13ad6`, after `lore index`, the parser mended) | 16 sessions, 98.53 USD, 6.16 a landing; a session's clock 63.6 minutes, 15.2 working and 48.4 waiting, 27.1 on its own gate; peaks 133 to 281 k, none past 300; 4413's builder is in no lane well and is the driver's window |
| the same table before the mend | 98.39 USD over 16 rows, two of them `1716` and `2921` (wells from months ago), 4419 and 4429 absent |
| the gates (`tools/gates.py … --since gate-pass-1119.log`) | 15, 290.5 minutes: 2 whole, 13 scoped (13 widened), 4 red (`gate-4416`, `gate-4415`, `gate-4418`, `gate-drain-1119b`); admission 15,242.8 s summed, longest 604.9 s |
| the declaration (`git log -1 8b3538f49`; `docs/RUNS.md` run1700) | committed 10:45:54 −05:00, three files, no take; run1700 launched 10:48:02; entry 144 §2's rule committed 10:03:25 |
| the eight, re-run (`the_held_out_games_score_as_pinned`, alone, `--nocapture`) | green, 206.2 s; each row's states, agreeing counts, first partings and `over: None` as pinned |
| the comparison failing (`gate-drain-1119b.log`) | red on that test with run1707 taken and `SCORES[7]` unpinned |
| the skip, before (reading `paths`) | a missing `.rcx` or a missing run directory filed no fixture request |
| the skip, after (`HOME` at an empty directory, `RON_FIXTURE_AUDIT_DIR`) | the test still passes and prints eight `skipping:` lines; the audit counts 16 requests, 16 missing |
| `skipping:` on gate logs (`grep -l` over 258 logs) | one log holds one; a passing test's stderr is not on a gate log |
| the new line's guard (`the_handoff_s_parity_held_out_line_is_the_pinned_scores`) | **red** on the queue without the line; green with it |
| the lobbies on disk (718 captures with a lobby block) | 688 seat two players, 26 three, 4 five; 4 share a team |
| the eight's rules (each take's `gamelog_p.txt`) | rows 0, 20, 120, 140: start age 0, end age 7, resources 1; rows 40, 60, 80, 100: start age = end age (2, 3, 4, 7), resources 7, difficulty 5; all two seats |
| the week (`lore limits`, 18:30) | 80 % (the day's line 84), from 79 at the pass's start; Fable's week 49 %, from 47 |
| the suite re-timed (`suite_timings.py --stale-only`) | 6 tests timed: 2,548 tests, 5,720 s one at a time |
| the disk (`df -g ~`) | 133 GB at the start, over the last pass's 98, so no profile cleaned; 134 after `afsctool` over both archives; 133 at the record |
| the rubric (`rubric.py --record`) | ported 737 → 738, open 572 → 571 (item 4415's `Unit::set_new_location`); every other figure the pass before's |
| the rows and trees left | none: the driver's `ccc rm` took all 17, `reap_agents.py --yes` printed nothing, the old driver row reaped by `handover.py reap 105996c9` |

## Rulings

- **Parity held out is ratified at 0 of 8 as found** (entry 145 §1),
  parsed on the scoreboard, its test routed through the fixture audit.
- **A held-out game is spent once a lane reads it** (§2). The found
  column is the estimate. A second battery is named: rows 10, 30, 50,
  70, 90, 110, 130, 150, taken at 4 of 8 now or on Ramon's word.
- **The lanes follow the eight, each game's ranges taken once** (§3).
- **A lane lands one act, and the driver counts the stop** (§4).
- **`skrifa` is earned**, in the client alone (§7).
- **The namesake's four other constant predicates book no item** (§7).
- **Marked rows:** no `FABLE:` marker and no second reading in the
  tranche.

## What the pass looked at

Four lobbies as the client opens them, with slot 0's capital selected,
and the two photographs' pairs at the tip.

- **The minted lobby at 6,000 (Great Lakes).** The capital is in view
  with its band and its plate, which reads "Napata" with the star. The
  morning's pass saw no plate here.
- **Row 100 at 3,000 (Great Lakes, Information).** The capital is
  burning, two smoke plumes over it, the population reads 0 of 25, and
  the computer's vehicles are at the bottom edge.
- **Rows 40 and 120 (East Indies).** Open sea, a strip of shore and a
  stand of trees. No city is on the screen. The camera line reads the
  same on all four stills: `pinned_camera` is one constant.
- **The city at 1:1 (run1559b).** The plate matches theirs in face and
  size. Theirs' roofs and awnings carry red and ours are bare stone.
  Theirs still has the trampled marks on the grass.
- **The lake (run1564).** As the two passes before saw it.

## The queue's items

| item | kind, mark | the score | first act | it stops and says so when |
|---|---|---|---|---|
| 4435 held out, the Ancient starts | rules, whole; the chain's | `Parity held out:` now column: rows 0, 20, 120, 140 | one ranges take of row 120 to its end (run1716), then units 2694 by leaf | two landings running move no row's column, or the disk reads under 60 GB |
| 4436 held out, the later starts | rules, whole; a parity seat's | the same line: rows 40, 60, 80, 100 | one ranges take of row 80 (run1720), then the world line on state 0 by leaf | two landings running move no row's column |
| 4432 the sample, from units 2700 | rules, whole; a parity seat's | `SAMPLE_LINES`' lowest first parting | unit o 27's move order, on disk | two landings running move no first parting |
| 4437 the opening camera | render, scoped; the eye's | a row minted first: slot 0's capital in the frame at the opening camera, of 161 lobbies | the row at its found value | no packet on disk holds a camera the game set itself |
| 4430, 4431 the packets' lines | rules, whole; unseated | `PACKETS_SCORE` 771 of 1,053 | as booked by the driver | two landings running turn no line |
| 4433 a second city's plate | render, scoped; unseated | as booked | one photograph, run1715 | — |
| 4438 the buildings' team colour | render, scoped; unseated | a row minted on run1559b's photograph | theirs' red in the capital's footprint against ours | the sprite carries no mask the client can read |

Item 4434 left the queue: the pass wrote its paragraph and its line.

## Built

- `the_handoff_s_parity_held_out_line_is_the_pinned_scores` and
  `queue_line` in `crates/rondata/src/diff/held_out.rs`, seen red; the
  test's name in `release_gate.py`'s `COMMANDERS_LINES`.
- `held_out::tests::paths` through `testenv::experiment` for both the
  recording and the take.
- `tools/tranche.py`'s `outside_parentheses`, and
  `test_a_booking_s_colon_inside_a_parenthesis_ends_nothing` (red on the
  old parser).
- `HANDOFF_LINES` 32 → 33 in `docs_guard.rs`.
- The paperwork: entry 145, this record, `docs/RECGAME.md` §10.3's
  results, the queue's handoff and four items, two clauses in
  `docs/LOOP.md`, the tracks' paragraph, the journal's entry, the suite's
  timings, the rubric's row and its two pins.

## Not done, and said

- **No parting of the eight names its leaf.** The takes hold sums. Items
  4435 and 4436 take the ranges.
- **Why the original ended the four later starts is not read from
  theirs.** The burning capital is our walk.
- **Parity's own count, 6 of 7, is still prose**, as at the three passes
  before. The held-out line is parsed; the declared set's is not.
- **The second battery's seats and teams are not designed.** Whether the
  lobby page can set a team was not read.
- **The walls line has no emitter.** Ours is the empty walk, and it
  agrees on every state of all eight, so none of the eight holds a wall.
- **The driver's own cost is not summed**, as before.
- **The blind list was not scanned**: open 0.

## The gate

Whole, by `tools/gate_run.sh`, twice.

- **`gate-pass-1119b.log`: `gate exit 1`** on `3774bc2d6`, in its offline
  step after one minute.
  `test_the_commander_s_lines_are_tests_that_exist` reads four source
  files for every name in `COMMANDERS_LINES`, and the new line's test
  lives in `diff/held_out.rs`, which was not one of them. The pass had
  run the paperwork guard and the release filters and not the offline
  suite. Both directions of that check read the held-out module now.
- **`gate-pass-1119c.log`: `gate exit 0`** on `e5f6fc88d`, 9 of 9 steps:
  offline 1.0, guard 1.0, merge pins 1.1, release 22.1, clock 0.1.
  `Clock: green beside load`. Admission 10.2 s over 1,457 leases; load
  2.2 at its start and 8.8 at its end. The four `held_out::tests` ran and
  passed, the scores' in 244.8 CPU-seconds. `Fixture audit: 198539
  requests, 0 missing fixtures`: eight more than the drain's, the eight
  recordings. `Rows: one driver row at
  most: steer 429c9837 (working)`; `Driver: 0 landings since the last
  pass (backstop 20) · context 368 k`. `Builders: 0 reaped`; 143 GB free
  after it.
