# The pass of 2026-11-15: the driver's two lines are asked at the gate, a wake is a spawn, the day's line carries no margin, and a session's booking gate runs through the script

*2026-11-15, from 20:07 −05:00, the `steer` row on Fable 5.2, called on
Ramon's word and by the twenty-landing backstop. `commander` (Opus 5.5)
handed over at `b4f919fb6` after fifty merges in two sittings: 13:27 to
20:15 on 2026-11-14, and one unbroken session from a cron wake at 06:07 to
18:01 on 2026-11-15. The charter was five findings the row itself raised
reading the day (the driver never cleared, the backstop passed in silence,
the day's line read with the wall's margin, the morning wake, Fable barely
used) and the handoff's five. Decisions in entry 135. The pass cut two
in-session builders on Opus 5.5 and took no capture. The pass before it is
`2026-11-14-fable-pass-token-bound.md`.*

## For Ramon

- **The day:** 33 merges in 11.9 h for 9 points (14 → 23 %), 279 USD at list (lanes 147,
  builders 96, the driver 36): 0.76 a point-hour, so fourteen a day is 18.5 h of this roster.
- **Fable, in the order I would buy it:** (1) the backstop's pass, skipped for fifty landings
  and now refused by the gate; two a day at this pace, about a point each (`BACKSTOP = 40`
  makes it one). (2) A Fable builder on a leaders leaf two Opus builders have parked, about a
  point a step against a third; the driver may already call it and never has. (3) Nothing
  else: lanes, readers and the driver stay Opus (the driver is 13 % of the spend).
- **Decided, yours to reverse:** the gate refuses a scoped gate past twenty landings and the
  four seats, any gate past 450 k, and a session's bare gate; no brief at twenty; the wake
  is a handover; the day's line has no margin. Entry 135.
- **Yours:** 57 merged branches on origin of 61; the README's line 12 ("no open-source Rise
  of Nations engine existed"); and ten minutes playing the client, which no counter replaces.

## What was measured

| what | measured |
|---|---|
| the merges since the pass before (`git rev-list --first-parent --merges 51aca3ff9..b4f919fb6`) | 50: 17 on 2026-11-14 from 13:27, 33 on 2026-11-15; the twentieth at 07:02 on 2026-11-15 (item 4263) |
| the lanes (`tools/tranche.py --base 51aca3ff9`, after `lore index`, once it read `Booking`) | 40 landings, 266.97 USD, 6.67 a landing (9.68 the tranche before); a lane's clock 56.3 minutes, 20.2 working and 36.1 waiting, 16.1 of them on its own gate; three past 300 k (4253 302, 4259 320, 4273 302; six of thirteen before, to 422); 22 captures |
| the builders (`lore spawns --session 49009261`) | 9 on 2026-11-15, all Opus 5.5, 96.29 USD, 10.70 each; 24 to 104 minutes, 1,529 requests; one in flight 498 of the 618 minutes from the first cut to the last handback |
| the driver (`lore trace 49009261 --steps`) | 437 requests, 36.22 USD; context 81 k at the wake (50 k written cold) → 536 k, 38 k an hour; 400 k crossed at 13:10:49, eleven seconds after a handback and 12.5 minutes before the next cut; 133 requests past it, 15.04 USD |
| the gates (`tools/gates.py ~/Library/Caches/attrition/gates --since gate-pass-1114f.log`) | 50, 712.5 minutes: 0 whole, 50 scoped (all widened), 10 red; admission 31,734 s summed, the longest wait 680.6 s; 11 ended past the cores |
| the reds | two the clock test beside load (23.5 and 15.2 minutes); one on 47 widenings a `--no-widen` lane left (14.5); one on the census pin at a builder's merge (12.8) and one on a packet's wiring (13.2); five on the booking's own paperwork (fmt, the structure pin, the handoff's section bound and two of its lines), 0 to 2.7 minutes each |
| how the gates were started | 16 through `tools/gate_run.sh` on 2026-11-14; 34 bare on 2026-11-15 (`release_gate.py … --scope ebfa81aab` in the integration tree, the queue's maintenance line): no `Gate tree:`, no `Rows:`, no `Disk:` line in any |
| the week (`lore limits`) | 7 % at the pass before, 14 % at 06:09 and 23 % at 18:01 on 2026-11-15, 24 % during this pass; 29.15 USD a point all-in; Fable's week 7 % |
| the suite re-timed (`suite_timings.py --stale-only`) | 59 stale tests timed: 2,464 tests, 4,973 s one at a time |
| the widenings (the timings file) | 613 tests ending `_widened_whole` or `_walked_whole`, 1,363 s one at a time; the two families kept before were 405 and 919 s |
| the disk (`df -g ~`) | 120 GB at the start (228 at the pass before's gate); 141 after `afsctool` over both archives, 154 after `cargo clean --profile dev` (24.6 GiB), 199 after nine merged builder trees went |
| the rubric (`rubric.py --record`) | the territory 736 · 331 · 572 (734 · 332 · 573 before), re-pinned; 44 of 112 groups whole, 1,456 of 3,611 backed, ages 8 of 8, unit types 296 of 302, lockstep bytes 78.1 %, all unchanged |
| the rows and trees left | no lane row; nine `agent-*` trees, seven under a lock naming the reaped driver's process |
| the blind list's nine open rows (`tools/trace/report.py <exe> refs`) | eight called; the group's stance issuer has no caller and no copy of its address in the executable's bytes |
| origin's branches | 61, 57 merged into `main` |

## Rulings

- **The backstop is counted and refused** (entry 135 §1). Built.
- **The driver's line is asked at the gate; past 350 k no builder is cut**
  (§2). Built, but for the cut itself, which is an Agent call no script
  sees: the 450 k refusal is what stands behind it.
- **A session's booking gate runs through `tools/gate_run.sh`** (§3).
  Built. This is the finding the charter did not carry, and the other two
  checks rest on it.
- **A wake is a spawn** (§4). Built: `handover.py wake`.
- **The day's line carries no margin** (§5). A clause; no guard, since
  cutting under the line is not a thing a tool refuses.
- **Every widening by suffix** (§6), the handoff's first. Built.
- **The handoff's second, `repin.py --write` dropping comments**: carried
  by two passes on "the chain's next builder", which nobody booked. Cut to
  a builder by this pass, which found the defect was another; see "Built".
- **The handoff's third, the clock test beside lanes**: its own rule calls
  the host loaded only above its cores (16) and it broke at 13.5 to 15.6;
  alone at 7.5 it reads p99 14 ms. A threshold moved to a load nobody
  measured would be a guess. Cut to a builder: the booking gate measures
  it alone under the whole host budget.
- **The handoff's fourth, what builders and lane gates miss.** Of ten reds
  five were the booking's own paperwork and cost ten minutes in all: the
  gate's cheap steps already run first, and nothing is built for them. The
  two that cost thirteen minutes each are the pins a merge moves, which
  `docs/LOOP.md` asks the driver to run by hand. Cut to a builder: the
  booking gate runs them first.
- **The handoff's fifth, a docs-only lane's exit 2** (4281). The same
  builder.
- **A lane's depth plan is confirmed** (§8), and a builder's brief takes
  the row.
- **The whole gate "at the drain"** ran at neither bank. It is left as
  written: with the backstop refused, a drain hands to a pass, and the
  pass gates whole.
- **The journals' "for the Loop" lines.** Twenty of the tranche's journals
  carry one. They are method for the item's own kind and stay where they
  are: check slot 0 holds an order's actor before booking a playback
  (4264, repeated by 4267); a `@dumpall` on frame N is a frame late
  (4270); a packet for a round goes on a frame the round is in flight
  (4277); name a parted range from the original's call site (4266); diff
  a draw's inputs against the packet before its pixels (4275, 4271); a
  booking that names "the nearest row" lists the row's parted objects
  (4245, taken: item 4284's and 4283's lines name their rows' objects).
  Two were fixed by their lanes (4268's ledger now reads the client crate;
  4279's cache keys are held apart by a test).
- **Marked rows:** the tranche took no second reading and left no `FABLE:`
  marker; nothing to ratify.

## The queue's items

Supply stands for the three seats. What the pass adds is the input seat's
step after its census ends at 25 of 26 kinds, so that an undesigned step
does not call the next pass.

| item | kind, mark | the score | first act | it stops and says so when |
|---|---|---|---|---|
| 4159 the leaders line | the chain's builders | `LEADERS_HELD_OUT` 10; `score` 7 tests | 4159r12a (run1184, ch77) | as each row says |
| 4284 the mountain's last six tiles | render, scoped | under a mountain, 100 of 106 | 4272a, the six tiles named and measured | as its line says |
| 4283 the AI toggle, disband, unitmask | input, scoped | `INPUT_KINDS` 22 of 26 | the console's case, read first | as its line says |
| 4285 a person's whole game, re-minted | input, scoped | recordings whose every order the client's layer mints byte for byte, of the 50 on disk; first counted by its first act | the census: per recording, the orders minted of its total, and the arm each miss takes | the misses are one kind's arms no button reaches, which is 4282a's and parks there |

## Built

By the pass, in this session, each red first.

- **The driver's lines** (`tools/handover.py`: `landings_since_pass`,
  `brief_line`, `landings_line`, `depth_line`, `check --scoped --tip`,
  `to --ungated`, `wake`; `tools/gate_run.sh`, exits 93 and 94;
  `tools/brief.py`). Red: nine tests on functions that did not exist; then
  the live tree, `50 landings since the last pass`, a brief refused with
  exit 2 and a scoped check with exit 3.
- **The bare gate** (`tools/release_gate.py`, `refuse_a_bare_booking_gate`,
  `GATE_RUN`). Red: no such function; then this session's own bare gate,
  refused.
- **Every widening** (`SIM_WIDENINGS`). Red: a goods widening in a
  sim-touching `--no-widen` lane's skips.
- **The tranche's table** (`tools/tranche.py`, `BOOK`, `ITEM`). Red: none
  of today's five subject forms read.
- **A stale lock** (`tools/reap_agents.py`, `stale_lock`). Red: a tree
  locked by pid 999999 refused; then the seven real trees removed.

By two in-session builders on Opus 5.5, merged by the pass:

- **The merge pins first** (`tools/release_gate.py`, `MERGE_PINS`, a
  `merge pins` step ahead of a booking gate's release step; 108.6
  test-seconds by the timings file, 90.8 of them the ages pin). Red: no
  such step.
- **The clock last and alone** (`CLOCK_TEST`, `clock_command`; a `clock`
  step after the release step under `tools/resource_budget.py --drain`, a
  lease that keeps each slot as it frees until it holds the whole host).
  Red: the gate's last steps were the guard and the release. A lane gate
  that touches the client still runs the clock in its own parallel step.
- **A filter split across left-out crates** (`left_only_filters`). Red:
  item 4281's own error, `the word's filters matched no test that ran`.
- **`repin.py --write`** keeps a comment inside a replaced want, writes an
  emptied list of strings as `Vec::<String>::new()` and never rewrites the
  pins self-test's literals. Red on each. **A trailing comment beside a
  want did not reproduce**: the lane's discarded commit holds its 22
  comments on the added lines, and its count was a grep over the removed
  ones. So the handoff's second finding and item 4258's journal name the
  wrong defect; what was lost was the comments inside a replaced list.
- **`cant_reach_pins.py --write`** edits its table in place and keeps every
  comment (`tools/explore/test_cant_reach_pins.py`, new, registered). Its
  parser had read one of the real table's two rows.

The builders: 14.0 and 22.7 minutes, 167 k and 287 k tokens. Their reports
name what they saw and did not touch; the pass carries three of those
below.

## Not done, and said

- **The drain's wait has no cap.** The clock's lease waits for the longest
  admitted test already running, and lanes' admissions wait behind it.
  With admission switched off nothing is drained. Unmeasured until a
  booking gate runs beside lanes.
- **The clock still sees load the budget does not hold** (a build, wine),
  and its step is outside the gate's fixture audit.
- **`Vec::<String>::new()` is assumed** for an emptied list of strings; six
  wants in the tree are `Vec::<&String>`, which the log cannot tell apart.
- **No look.** The pass designed no render step and drew no still.
- **The cut past 350 k is prose.** An in-session builder is an Agent call;
  only the 450 k refusal can fail.
- **Whether a builder's handback survives its driver's clear** is untested.
- **No guard reads a count in the handoff's prose against its pin**
  (carried from two passes).
- **The clock between loads of 7.5 and 13.5** is unmeasured.
- **Item 4285 is a direction, not a reading**: the pass did not open
  `docs/CLIENT_INPUT.md`'s sections to check that every kind's "every one
  on disk, typed back" test exists; its first act is the check.

## The gate

Whole, by `tools/gate_run.sh`, the first gate to run the merge pins and the
drained clock for real.

GATEVERDICT
