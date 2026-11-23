# The pass of 2026-11-23: the seam counter names a first parting when it is run

*2026-11-23, from 07:20 −05:00, the `steer` row on Fable 5.2, called by
the backstop: twenty landings. `commander` (Opus 5.5) handed over at
`de60a62ee` after twenty first-parent merges (items 4574, 4572, 4581,
4580 twice, 4576, 4582, 4575, 4584, 4577, 4586, 4587, 4583, 4588, 4585,
4589, 4591, 4593, 4594 and 4595), gated whole (`gate-drain-1123.log`).
The charter was the handoff's: seat four, and read its `For the pass`
part. Decisions in entry 154. The pass spawned no subagent and took no
capture. The pass before it is `2026-11-22-fable-pass-rows.md`.*

## For Ramon

- **The headline is 47 of 55, up from 43 of 47.** Four more games agree
  with the original on every state. The share reads 85 % where it read
  91 %, because the new battery joined with seven unfinished games, as
  the last record said it would. Twenty landings in six hours forty,
  7.25 USD each on the lanes.
- **Sending seats to the game that goes wrong earliest worked.** A
  landing gained about 620 agreeing states, against about 200 the
  tranche before. One fix closed the first difference in three games.
- **The person's game moved without a seat.** A fix made for a battery
  game turned out to be its problem too: it now goes wrong at state
  7,502, from 7,407.
- **The list of known omissions you had counted is worth more than its
  first answer.** That answer was "none of them explains a game". I ran
  the counter again this morning and it names the next difference in
  one game outright: the note says what the original does, and the
  bytes agree. One of this tranche's fixes had also been written down
  as an omission, with no counter on it. So the counter runs at every
  pass now, and one item puts counters on the 71 notes still without.
- **Two lanes landed on a failing gate.** The failure was the
  frame-time test, slow because it ran beside the lane's own tests.
  It is not a real slowdown: all thirteen booking gates measured it
  green. Each lane argued the failure away in its notes, which the
  rules forbid. The gate now measures it the way the booking gate does.
- **Lanes still wait an hour a landing, 43 minutes of it on their own
  gate.** Twelve times a lane was sent round again because another
  landing touched the file they all re-pin, and four more times for a
  number no scoreboard line shows. One item makes those tests fail only
  on what the scoreboard books. I decided this as loop mechanics. What
  it gives up: a change that only alters a game after its first
  difference is printed, not failed. Say the word to keep the strict
  pins.
- **The next battery's human keeps playing.** It queues a Citizen and
  moves its scout every 600 frames, where the last one stopped giving
  orders at frame 90. Every first difference found in the last battery
  was a computer's decision, so a person's orders are still untested.
  It is recorded when the current battery reaches 4 of 8, likely on the
  next landing. Say the word to keep the old recipe.
- **The minimap matches the original on 2,808 of 3,600 cells**, from
  2,288 when it was first drawn. The unit and building dots are next.
  I drew and uploaded no pictures this pass.
- **Yours to decide:** 318 merged lane branches stand on origin. Seven
  duplicate probe files sit loose in `~/ron-data/lab-experiments/`,
  identical to the copies in their item's folder; say the word and
  they go.
- **The week** read 32 % at 07:45 −05:00 on its third day, ten points
  under the day's line. Fable's week read 12 % before this pass.

## What was measured

| what | measured |
|---|---|
| the drain's gate (`gate-drain-1123.log`) | `gate exit 0` on `de60a62ee`, the tip, HEAD = origin/main = origin/worktree-replan-pdb; whole, 9 of 9 steps, 26.7 minutes, release 22.9; `Driver: 20 landings since the last pass (backstop 20): backstop reached`, context 318 k; `Trunk: origin/main is origin/worktree-replan-pdb` |
| the merges (`git log --first-parent --merges 93932dfd0..de60a62ee`) | 20, 00:37 to 07:14 −05:00 |
| the lanes (`tools/tranche.py --base 93932dfd0`, after `lore index`) | 17 sessions over 17 items, 123.26 USD, 7.25 a landing; the eleven sim lanes 86.76, 7.89 a landing; the six client lanes 36.50, 6.08; a session's clock 80.6 minutes, 16.7 working and 63.9 waiting, 43.1 on its own gate and 11.8 on a suite run; peaks 180 to 351 k, two past 300 (4585 at 313, 4587 at 351); items 4574 and 4580 were in-session builders and have no session; 5 captures |
| the gates (`tools/gates.py … --since gate-pass-1122c.log`) | 13, 355.6 minutes: 12 scoped (12 widened) and the drain's whole gate green, 0 red; admission 24,255.9 s summed, longest 762.2 s; 4 ended with the load past 16; `Clock: green beside load` on all 13 |
| why a lane waited on its gate (the journals' gate sections) | `Base moved` refusals naming `held_out.rs` on both sides: 4572 one, 4575 three, 4577 one, 4583 two, 4589 one, 4591 one, 4594 two, 4595 one; twelve over eight of eleven sim lanes. Reds on the merged tree for a count past another row's first parting or the sample's held frames: 4577, 4583, 4585, 4594. Updates that conflicted on `docs/RECGAME.md` §13's one paragraph: 4575, 4583 twice, 4591, 4595; two merges backed out (4575, 4591). Client lanes: none of either |
| the lane gates' clock (the client lanes' journals) | 4581: gate exit 1, p99 28.41 ms at host load 24.6, then 27.30 at 15.7; alone 20.56 and 20.75 at 19.5 and 15.2. 4582: gate exit 1, 25.84 at 47.1, then 20.68 at 15.4; alone 15.50 at 16.6. 4584, 4586, 4588, 4593: green |
| what a seat bought (the queue's lines at each booking; the journals) | twelve landings on a held-out row: 7,449 states before a first parting (the fifth's row 3 1,011; the sixth 6,438: row 33 1,533, row 73 1,875, row 93 344, row 113 1,736, row 133 950), 621 a landing; item 4583 alone 3,927; no first parting moved at 4574, 4576, 4587. The sample: leaders 7407 → 7502, draws 7410 → 7613, at item 4585 |
| the eight games not whole (`score_recording` at the tip) | entry 154 §3's table; exit 1 on each, a measured difference, 13 to 57 seconds; every report equals its pin; under `~/ron-data/lab-experiments/2026-11-23-pass/` |
| the seam table (`held_seams`, feature `seam-count`, six tests, 206 s) | first run, on the tip: five green, the sixth red, `left: Some([(133, 16)])`, `right: Some([(133, 21)])`. Row 133: `sim::orders::jobs::computer_repair_explore` entered once, first on state 1906, the row's first parting, marked `=`; no other parted row enters a hook on or before its first parting that a whole row does not also enter, but the two item 4580 named on the fifth's rows 43 and 83. The first run wrote into item 4580's directory, over its tables |
| the notes behind the tranche's leaves (item 4580's `classification/`) | the President's heal (4577): note `s400`, class P, no hook. The repair swarm (row 133's 1906): note `s309`, class F, hooked. The tower arm's comment (4585): not among the 436 |
| the archives (`split.py --archive`, then `afsctool` over both) | nothing to split; four files skipped over the compressor's limit, the same four |
| the disk (`df -g ~`) | 88 GB at the start; 93 after `cargo clean --profile dev` (10.8 GiB); 96 after the compression; 93 at the record |
| the week (`lore limits`) | 32 % at 07:45 −05:00, reset at 06:00 −05:00 on 2026-11-21; Fable's week 12 % at 06:14 |
| the suite re-timed (`suite_timings.py --stale-only`) | 7 tests timed: 2,669 tests, 6,503 s one at a time |
| the rubric (`rubric.py --record`) | every figure the pass before's: ported 739, open 570, lockstep bytes 78.1 %, unit types seen 296 of 302 |
| the rows and trees left | no lane row stood; the old driver row reaped by `handover.py reap 0ebb62fb`, and its job directory, with the handoff's two tables, went with it; `reap_agents.py` found no builder tree; no Codex branch ahead of HEAD; 318 merged `worktree-att-*` branches on origin |
| item 4593's loose copies | seven files, each `cmp`-equal to its copy in `2026-11-23-item-4593/`; left in place |

## Rulings

- **The seats go by first parting, still** (entry 154 §1).
- **The seam counter is the pass's tenth act** (§2), written to the
  pass's own directory, its pin re-pinned there.
- **Item 4597 opens on the seam's fix**, and **item 4599 hooks the P
  notes** (§2). The readers' table stands as a sort and not as a
  reading: it was four Sonnet readings.
- **Rows 93 and 83 take one seat**, item 4590's (§3). Item 4573 rides
  it.
- **A lane's clock runs after its release step, never drains, and its
  red is the booking gate's** (§4).
- **The held-out tests fail on whole lines and first partings alone**
  (§5; item 4598). The pins stay.
- **A sixth row writes under its own heading in RECGAME §13** (§6).
- **The seventh's human orders through the game** (§7; item 4600). The
  trigger is entry 153 §6's.
- **Four seats** (§10): the chain's on 4598, then 4599, then 4590;
  parity seats on 4592 and 4597; the client's on 4596.
- **Twenty-six parked findings**: five closed (§9), the rest stay in
  their journals.
- **Marked rows:** no `FABLE:` marker and no second reading in the
  tranche.

## What the pass looked at

Five of the twenty landings drew the minimap. The pass designed no
render step: item 4596 is the commander's, off item 4593's reading. So
it drew no still and uploaded no pair. What it read is each lane's own
"Looked at" paragraph, which sets the photograph's crop beside the
still at every landing: the lakes, forests, ranges, borders and view
rectangle sit where theirs do; the dots, the ping rings, grey patches
and cyan specks in the lakes are what ours lacks.

## The queue's items

| item | kind, mark | the score | first act | it stops and says so when |
|---|---|---|---|---|
| 4598 the held-out pins thinned | rules, whole; the chain's | none of its own; the lanes' gate minutes | a pin bent past a first parting passes and prints, one bent on it fails | it has landed its act |
| 4592 sixth, row 113 | rules, whole; a parity seat's | the sixth 3 → 4 of 8, the headline's forty-eighth | a cities take, `0x100u`, `--allow-early-end` | two landings running move no column |
| 4597 sixth, row 133 | rules, whole; a parity seat's | row 133's column from 1906 | the seam's fix, the long words measured | two landings running move no column |
| 4596 the blips | render, scoped; the client's | `RUN1564_3279_MINIMAP` from 2,808 | the packet's 118 quads | the commander's |
| 4599 the P notes hooked | rules, whole; the chain's, after 4598 | none of its own; a list by first parting | each predicate against the code | the depth rule |
| 4590 rows 93 and 83 | rules, whole; unseated, first | both rows' columns from 2686 and 3662 | a packet at row 93's 2685 | row 93's second landing that moves no column |
| 4578 sixth, row 53 | rules, whole; unseated | row 53's column from 3358 | a take, `0xffbu` | two landings running move no column |
| 4573 fifth, row 83 | rules, whole; rides 4590 | row 83's column from 3662 | the leaf on run1784 | two landings running move no column |
| 4571 fifth, row 43 | rules, whole; unseated | row 43's column from 3833 | the leaf on run1787 | two landings running move no column |
| 4569 the sample | rules, whole; unseated | `Parity:`, leaders 7502 | the leaf on run1786, `encrypt` 3200 against 3440 | two landings running leave the first parting and the draw word |
| 4579 sixth, row 13 | rules, whole; unseated | row 13's column from 7675 | a take, `0xfebu` | two landings running move no column |
| 4600 the seventh battery | rules, whole; on its trigger | a new line, as found | the declaration, then row 8 alone | the original refuses row 8's file |

## Built

- `tools/release_gate.py`: a lane that keeps the client crate skips
  the clock in its release step and runs it after, under the two-slot
  lease; a red there prints and is left to the booking gate. Three
  tests, `LaneClock`, in `tools/explore/test_release_gate.py`, seen red
  on the gate as handed over (`['guard', 'release']` where `['release',
  'clock']` was asked); three older tests that pinned the lane's seven
  steps now pin eight. 107 tests pass.
- `crates/rondata/src/diff/held_seams.rs`: the tables go to
  `RON_SEAMS_DIR` when it is set; the fallback's pin reads `(133, 16)`.
  Seen red at 21 on the tip.
- `docs/RECGAME.md` §13: a heading for each open row.
- The paperwork: entry 154 and its index row, this record, the queue's
  handoff, lanes line and twelve items, the loop's clauses (the lane's
  clock, the tenth act, the row headings, the seventh's recipe, where
  a handoff's tables go), the tracks' paragraph, the journal's entry,
  the suite's timings, the rubric's row.

## Not done, and said

- **The seam's fix is not built.** The note, the hook's one entry and
  the leaf agree on state 1906. Whether the fix moves row 133 past it,
  and what it does to the two long words the note says it was held
  for, is item 4597's to measure.
- **That one term parts rows 93 and 83 is not shown.** Two leaves have
  one shape. No function was read for it.
- **Row 113's cause is arithmetic on two sums**, not a reading.
- **The thinned assertion is designed and not built.** How many of the
  twelve refusals it removes depends on its second half, which changes
  a refusal that has kept every booking gate green for two tranches.
- **The seventh's script is a rule on paper.** Whether the original
  plays a file whose orders name a dead unit is what row 8 alone is
  for.
- **The 210 notes classed as unreachable were read by Sonnet alone.**
  Nobody has checked that class, and one comment outside the 436 was
  wrong in exactly that way.
- **The first run of the seam table overwrote item 4580's tables** with
  the tip's. Its classification and candidates files are untouched.
  The journal's own table is in git.
- **The lane's clock, as rebuilt, has not run in a real lane gate.**
  The tests drive the gate's steps with a fake runner.
- **No still was drawn and nothing was uploaded.**
- **The tracks' paragraph for the pass of 2026-11-18 was retired** to
  keep the section under its ceiling; its story is the journal's.
- **`tools/ctx.py` still reads the driver's context from a builder**,
  the four files over the compressor's limit stay raw, the driver's own
  cost is not summed, and the blind list was not scanned (open 0), all
  as before.

## The gate

Whole, by `tools/gate_run.sh`, once.

- **`gate-pass-1123.log`: `gate exit 0`** on `ebc1b9d00`, the pass's
  commit, 9 of 9 steps: offline 1.0, guard 0.2, merge pins 1.1, release
  22.3, clock 0.1. `Clock: green beside load`. Admission 13.6 s over
  1,571 leases; load 3.2 at its start and 8.5 at its end. `Fixture
  audit: 204394 requests, 0 missing fixtures`. `Rows: one driver row at
  most: steer f42ce00d (working)`; `Driver: 0 landings since the last
  pass (backstop 20) · context 393 k`; `Trunk: origin/main is
  origin/worktree-replan-pdb`. `Builders: 0 reaped`; 98 GB free after
  it.
- The commit that quotes it is documents only and ran no whole gate;
  the reflex, `tools/guard.sh`, was green on it.
