# The pass of 2026-11-22, third: the seats go by first parting, and the sixth's rows take them

*2026-11-22, from 22:35 −05:00, the `steer` row on Fable 5.2, called by
a drain: a seat freed with no designed item to take it. `commander`
(Opus 5.5) handed over at `cf483b9c6` after ten first-parent merges
(items 4563, 4559, 4562, 4565, 4567, 4568, 4566, 4564, 4548 and 4570),
gated whole (`gate-drain-1122-3.log`). The charter was the handoff's:
read the sixth battery's found column for the sample's seat (entry 152
§5) and seat five items. Decisions in entry 153. The pass spawned no
subagent and took no capture. The pass before it is
`2026-11-22-fable-pass-leaves.md`.*

## For Ramon

- **The headline is 43 of 47, up from 41.** Two more games agree with
  the original on every state. Ten landings in five hours, 8.84 USD
  each on the lanes, up from 7.54.
- **The person's game no longer has a seat reserved for it.** You asked
  what to do about its pace. Measured over 29 landings: work on a game
  that goes wrong early gained about 730 agreeing states a landing;
  work deep in a game gained about 190, on the person's game and on
  battery games alike. So seats now go to whichever game goes wrong
  earliest. The person's game goes wrong at state 7,407, later than
  nine of the ten others, so it waits. Its line will stand still
  meanwhile. Say the word and it takes the fourth seat back.
- **The new battery did not answer the question it was built for.** It
  was meant to show whether the person's game is hard because it is
  long or because a person played it. Six of its eight games go wrong
  in their first four minutes, so only two got far enough to say: one
  agrees for all 12,000 frames, one goes wrong at 7,675.
- **The headline's share will drop when that battery joins it.** By
  your fold rule it joins at the first fix on one of its games, as 44
  of 55. That is 80 % where today reads 91 %. Nothing got worse; seven
  unfinished games enter the count.
- **One fix replaces eleven again.** The new battery's games field
  eleven units whose shots start from the wrong point. One item builds
  them all.
- **Three of its games look like one bug.** They are the three where
  both computers ally against the human, and each goes wrong on the
  same two records at once. They share one seat until shown otherwise.
- **Two games ran at once during the battery's recording**, against the
  one-at-a-time rule. A waiting script gave up a second too early.
  Fixed and tested. Both recordings were good.
- **Lanes wait an hour on their own gate**, up from 46 minutes. Seven
  of eight had to re-run because another lane's landing touched the
  file they all re-pin, seventeen times in all. I changed nothing: the
  re-runs are also why no booking gate was red. A fix exists (pin less
  per game) and wants a daytime item, on your word.
- **The week** read 26 % at 23:02 −05:00, the second day of its window,
  two points under the day's line. Fable's week read 11 %.

## What was measured

| what | measured |
|---|---|
| the drain's gate (`gate-drain-1122-3.log`) | `gate exit 0` on `cf483b9c6`, the tip, HEAD = origin/main = origin/worktree-replan-pdb; whole, 9 of 9 steps, 29.4 minutes, release 25.0; `Fixture audit: 203656 requests, 0 missing fixtures`; `Driver: 10 landings since the last pass (backstop 20) · context 262 k`; `Trunk: origin/main is origin/worktree-replan-pdb` |
| the merges (`git log --first-parent --merges 0a1dd89ea..cf483b9c6`) | 10, 16:46 to 22:03 −05:00 |
| the lanes (`tools/tranche.py --base 0a1dd89ea`, after `lore index`) | 8 sessions over 8 items, 70.75 USD, 8.84 a landing; a session's clock 135.3 minutes, 19.9 working and 115.4 waiting, 60.3 on its own gate and 31.9 on a suite run; peaks 225 to 348 k, three past 300 (4548, 4565, 4566); items 4559 and 4567 were in-session builders and have no session; 9 captures over 8 items |
| the gates (`tools/gates.py … --since gate-pass-1122b.log`) | 11, 299.2 minutes: 9 scoped (9 widened) and the drain's whole gate green, 0 red; `gate-book-4559` refused before it ran, exit 92, main 15 commits behind the pushed tip; admission 19,480.7 s summed, longest 824.2 s; 1 ended with the load past 16 |
| why a lane waited on its gate (the eight journals' gate sections) | `Base moved` refusals: 4562 two, 4565 three, 4568 two, 4566 two, 4564 three, 4548 two, 4570 three, 4563 none; seventeen, each naming `held_out.rs` touched on both sides; 4548's third came after all seven steps had run. Two runs lost to `docs/RECGAME.md` §12's ceiling (4563 at 16,459 of 16,000 bytes, 4568 at 16,100) |
| the `Parity:` line and the fifth's at each booking, two tranches (`git show <booking>:docs/QUEUE.md`) | the sample's first parting moved at 4543, 4549, 4552, 4555, 4557, 4560 and 4565 and at no other booking: 6108, 6114, 6162, 6669, 6928, 7021, 7055, 7407. The fifth's states before a first parting: 27,316 of 43,524 at `25a90ab76`, 35,364 at `0a1dd89ea`, 38,006 at `cf483b9c6`; 4565's booking moved row 3 by 920 and row 43 by 143 |
| what a seat bought | the sample: 7 landings, 1,299 states, 186 a landing, 57.34 USD, 8.19 a landing, no game whole. The fifth's rows, the tranche before: 11 landings, 8,048 states, 732 a landing, 53.93 USD over seven priced, row 123 whole. The fifth's rows, this tranche: 8 landings, 1,579 states, 197 a landing, 46.81 USD over six priced, rows 63 and 103 whole. Row 83's last three landings: 286, 11 and 6 states |
| the eleven games not whole (`score_recording` at the tip) | entry 153 §3's table; exit 1 on each, a measured difference, 8 to 60 seconds; every report equals its pin; under `~/ron-data/lab-experiments/2026-11-22-pass-third/` |
| the sixth's found column, read | first partings 7675, 2411, 3358, 1563, 2342, 2259, 956, whole; median 2,376; 32,565 of 66,044 states; 24,891 of 41,960 inside the first 6,001; 7,674 of 24,084 past them, rows 13 and 153 alone |
| the fallback of `launch_point` (a scratch print, reverted; the sixth's and the fifth's score tests and `sample::`) | 0 rounds on the fifth and on the sample's fourteen tests; 3,673 on the sixth from eleven pieces and 102 keys: 67 (row 73, 296 rounds, first on frame 2689), 140 (rows 113 and 33, 53 and 27, frames 3047 and 3372), 243, 242, 233, 222 and 1131 (row 93: 1,520, 1,310, 123, 57 and 155; frames 4749, 4828, 4894, 8297 and 8406), 1133, 234, 206 and 1485 (row 133: 1, 52, 15 and 64; frames 3570, 3896, 4024 and 4317); none on rows 13, 53 and 153 |
| the probe's second run | exit 101 and 0 rounds: the frame's print did not build, an `Option` it could not format |
| the archives (`split.py --archive`, then `afsctool` over both) | 38 records, nothing to split; four files skipped over the compressor's limit, the same four as the pass before; 96 GB free before and after |
| the disk (`df -g ~`) | 96 GB at the start; 102 after `cargo clean --profile dev` (9.9 GiB); 101 at the record |
| the week (`lore limits`) | 25 % at 20:30 −05:00 and 26 % at 23:02, reset at 06:00 −05:00 on 2026-11-21; Fable's week 9 %, then 11 % |
| the suite re-timed (`suite_timings.py --stale-only`) | 18 tests timed: 2,662 tests, 6,495 s one at a time; the sixth's score test is 189.8 s |
| the rubric (`rubric.py --record`) | every figure the pass before's: ported 739, open 570, lockstep bytes 78.1 %, unit types seen 296 of 302 |
| the rows and trees left | no lane row stood; the old driver row reaped by `handover.py reap ca2ed1e4`; `reap_agents.py` found no builder tree; no Codex branch ahead of HEAD |

## Rulings

- **The found column fires neither branch of entry 152 §5** (entry 153
  §1). Six rows part before 3,400; two reach past 6,000 in agreement.
- **The seats go by first parting, and the sample holds none of its
  own** (§2). One line from whole first, then the lowest first parting.
  Item 4569 stays written and is seated before a hand-over when no
  row's item stands.
- **Four seats** (§3): the chain's builder on item 4574, parity seats
  on row 3 (4572) and row 133 (4575), the fourth on rows 73, 33 and 113
  (4576). Behind them, in order: 4577, 4578, 4573, 4571, 4569, 4579.
- **Rows 73, 33 and 113 take one seat** (§4), on one take of row 73.
- **The eleven pieces are built by class** (§5), in one act.
- **The sixth folds at its first leaf test** (§3): 44 of 55 on today's
  pins. Entry 150 §1 stands as written.
- **A seventh battery is named** (§6): rows 8 to 148, taken when the
  sixth reads 4 of 8 now or on Ramon's word.
- **The sample's stop rule stands** (§2). Every landing moved the line.
- **The lanes' refusals on a moved base stand** (§8). The pins are not
  thinned by this pass.
- **Seven parked findings stay in their journals** (§9); 4566a is
  closed by the waiter's fix.
- **Marked rows:** no `FABLE:` marker and no second reading in the
  tranche.

## What the pass looked at

Nothing was drawn. The tranche has no render landing: all ten are
rules items. The pairs of 2026-11-20 were not redrawn or uploaded.

## The queue's items

| item | kind, mark | the score | first act | it stops and says so when |
|---|---|---|---|---|
| 4574 the sixth's eleven pieces with no bay | rules, whole; the chain's | what moves in the sixth's `moved` columns | a test that lists the 102 keys, red; the `<RESTRICTION>` lookup, then 4559's recipe | it has landed its one act |
| 4572 fifth, row 3 | rules, whole; a parity seat's | the headline's 43 → 44 of 47 | the one leaf on run1796, world `mask`; no take | two landings running move no column |
| 4575 sixth, row 133 | rules, whole; a parity seat's | row 133's column from 956 | a take, `0xffbu`, `--allow-early-end` | two landings running move no column |
| 4576 sixth, rows 73, 33, 113 | rules, whole; the fourth seat's | the three rows' columns from 1563, 2411, 2259 | a take of row 73, `0xffbu`, `--allow-early-end` | two landings running move no column |
| 4577 sixth, row 93 | rules, whole; unseated | row 93's column from 2342 | a take, `0xffbu`, to 12,000 | two landings running move no column |
| 4578 sixth, row 53 | rules, whole; unseated | row 53's column from 3358 | a take, `0xffbu`, to 12,000 | two landings running move no column |
| 4573 fifth, row 83 | rules, whole; unseated | row 83's column from 3662 | the leaf on run1784; no take | two landings running move no column |
| 4571 fifth, row 43 | rules, whole; unseated | row 43's column from 3833 | the leaf on run1787; no take | two landings running move no column |
| 4569 the sample | rules, whole; unseated | `Parity:`, leaders 7407 | the leaf on run1786; no take | two landings running leave the first parting and the draw word as they are |
| 4579 sixth, row 13 | rules, whole; unseated | row 13's column from 7675 | a take, `0xfebu`, to 12,000 | two landings running move no column |

## Built

- `tools/gamelog/waitrun.sh` gives a launch whose log shows no line
  past its own header `WAITRUN_GRACE` seconds, 120 by default, before
  it calls a missing runner dead. Two tests in
  `tools/explore/test_waitrun.py`;
  `test_a_launch_whose_runner_has_not_started_is_waited_for` seen red
  on the script as handed over (exit 2 inside three seconds). The
  first draft took the runner's `Staged:` line for the start and
  turned `test_a_log_with_no_banner_is_still_a_dead_runner` into a
  timeout; the start is any line past the header now, and the
  thirty-nine tests of the three launcher files pass.
- The paperwork: entry 153 and its index row, this record, the queue's
  handoff, lanes line and ten items, the loop's two clauses, the
  tracks' paragraph, the journal's entry, the suite's timings, the
  rubric's row.

## Not done, and said

- **That one mechanism parts rows 73, 33 and 113 is not shown.** Three
  takes of sums name the same two lines on the first state; no field
  and no function was read.
- **The eleven pieces are numbers, not names**, as the eight were. The
  item's first step names them and their tables.
- **Every one of the eleven fires after its row's first parting, on
  our walk.** A piece the original fields past that state and ours
  never does is not listed, and none of the eleven is known to be a
  row's first leaf.
- **No guard holds the fallback at zero**, as before. It has now been
  a scratch print twice, and both times one run of it printed a zero
  that was not a zero. The patch is kept; a third use graduates it.
- **What the ruling on the sample gives up is not measured.** Nineteen
  row landings moved its first parting by nothing. Whether the sixth's
  rows will is unknown, and the rate of 732 was read on the fifth's
  rows, not the sixth's.
- **Whether the sample's trouble is depth or a person's orders is
  still open.** Row 13's leaf on 7675 would say, and its item sits
  last by its first parting.
- **The seventh battery's script is not designed.** A human who keeps
  issuing orders past frame 90 is named as the limit and nothing more.
- **The lanes' wait on their own gate is explained and not mended.**
  Thinning what a row pins is a change to `held_out.rs` and its tests.
- **The seam scanner still reads the names it is given.** The lanes
  line asks the driver's note to name the field; nothing enforces it.
- **`tools/ctx.py` still reads the driver's context from a builder**,
  and the readers of `leaders.defeated` are not swept, as before.
- **The four files over the compressor's limit stay raw**, as before.
- **The driver's own cost is not summed**, as before.
- **The blind list was not scanned**: open 0.
- **The tracks' paragraph for the pass of 2026-11-15 was retired** to
  keep the section under its ceiling; its story is the journal's.

## The gate

Whole, by `tools/gate_run.sh`, once, on the pass's commit. The commit
after it quotes the log here.
