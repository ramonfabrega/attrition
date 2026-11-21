# The pass of 2026-11-20, second: the estimate read by reach, the rows nearest whole seated, and parity's count parsed

*2026-11-20, from 21:05 −05:00, the `steer` row on Fable 5.2, called by
the backstop. `commander` (Opus 5.5) handed over at `d516f0746` after
twenty-three first-parent merges (items 4455 to 4477, with 4431), drained
and gated whole (`gate-drain-1120b.log`). The charter was the handoff's:
score the held-out rows and the sample on disk, design the seats the
lanes line left empty, and rule on the fold. Decisions in entry 147. The
pass spawned no subagent and took no capture. The pass before it is
`2026-11-20-fable-pass-leaves.md`.*

## For Ramon

- **The fold is still yours, and it reads 11 of 15 today.** This is the
  third record to put it to you. Six of the seven and five of the first
  eight are whole. I still recommend folding the first eight.
- **Parity held out went 3 → 5 of 8, and a sixth is one line away.** Row
  60 agrees on eleven lines to its end and on the world line for 3,984 of
  4,075 states. The builder's seat is on it.
- **The second battery read 1 of 8 as found, against the first's 0 of
  8.** That looks like no progress, and it is not the whole story. An
  unseen game now runs 53 % of its states before it first parts, against
  27 % a day earlier. Eight games against eight is a small sample. The
  scoreboard carries that figure from now on.
- **A third battery is named** (rows 5, 25, … 145), taken when the second
  reads 4 of 8. The lobby page can set a team, so a battery with seats
  and a team is yours to ask for.
- **The sample moved 43 states in six landings** (3878 → 3921 of 21,044,
  40.76 USD). All six were one event, the capture of a capital. The six
  before moved it 1,178. It keeps its seat; the road is long and uneven.
- **The picture did not change this tranche.** The "fading wrecks" were
  three live tanks under a haze. The render seat reads what they are.
- **The tranche:** 23 lane sessions, 147.66 USD, 6.42 a landing, no red
  gate of 18. A landing waited 62.0 minutes, down from 71.4.
- **The week:** 93 % at the pass's start against the stop, 99, on its
  last day; it resets at 06:00 −05:00 on 2026-11-21. Fable's week 54 %.
  The opener tells the commander to cut nothing at 96 or more and wake
  at 06:07.

## What was measured

| what | measured |
|---|---|
| the drain's gate (`gate-drain-1120b.log`) | `gate exit 0` on `d516f0746`, the tip, HEAD = origin/main = origin/worktree-replan-pdb; whole, 9 of 9 steps, 24.6 minutes, release 21.1; `Driver: 23 landings since the last pass (backstop 20)`, context 313 k |
| the merges (`git log --first-parent --merges 3d376d942..d516f0746`) | 23 |
| the lanes (`tools/tranche.py --base 3d376d942`, after `lore index`) | 23 sessions over 23 items, 147.66 USD, 6.42 a landing; a session's clock 79.2 minutes, 17.2 working and 62.0 waiting, 36.2 on its own gate and 14.8 on a suite run; peaks 156 to 331 k, one past 300 (4463, 13.36 USD); 9 captures. The handoff's 139.46 USD over 22 was pasted before 4477's session closed |
| the gates (`tools/gates.py … --since gate-pass-1120.log`) | 18, 434.3 minutes: 1 whole, 17 scoped (16 widened), 0 red; admission 31,965.7 s summed, longest 774.2 s; 5 ended with the load past 16 |
| the eight now (`held_out::SCORES`, read) | rows 0, 20, 80, 120 and 140 whole; row 60 eleven lines of twelve; rows 40 and 100 nine each |
| rows 60 and 100 by leaf (`score_recording` at the tip on run1722 and run1721) | entry 147 §3's table; each line's agreeing count and first parting is the pinned `moved` column; exit 1 on each, a measured difference |
| the second eight now (`held_out::SCORES_SECOND`, read) | rows 50 and 150 whole; rows 70 and 90 eleven and ten lines, a line and a state short; rows 30, 110, 10 and 130 eight, six, five and five. None has a ranges take |
| the sample (`SAMPLE_LINES`, `SAMPLE_RANGES_1724`, read; green on the drain's gate) | lowest first parting units 3921 (`2190 11 guys.length`); eight lines part between 3921 and 3981, cities whole to 4130, goods 4171, items 5235, no wall |
| reach (`held_out::queue_line`) | first battery 27 % as found, 92 % now; second 53 % as found, 57 % now |
| the week (`lore limits`, 21:07) | 93 % (the stop 99, the last day); Fable's week 54 % (its stop 90) |
| the suite re-timed (`suite_timings.py --stale-only`) | 16 tests timed: 2,571 tests, 5,847 s one at a time |
| the disk (`df -g ~`) | 85 GB at the start, under the last pass's 127; 94 after `cargo clean --profile dev` (13.3 GiB); 96 after `afsctool` over both archives |
| the rubric (`rubric.py --record`) | every figure the pass before's: ported 738, replaced 331, open 571 |
| the rows and trees left | none: no lane row stood, `reap_agents.py --yes` printed nothing, the old driver row reaped by `handover.py reap 6c4d1009`; Codex's branch is not ahead of HEAD |

## Rulings

- **The fold is Ramon's; recommended, 11 of 15 folded** (entry 147 §1).
- **The estimate is read by reach beside games and lines, and the lines
  carry it** (§2).
- **The third battery is rows 5 to 145**, named before any take (§2;
  item 4487).
- **Four seats** (§3): the chain's on row 60, a parity seat on the second
  battery's row 90, a parity seat on the sample, the eye's on the hazy
  tanks. Six items wait behind them, row 70 first.
- **4458a, 4462a and 4462b are items 4482, 4483 and 4485** (§4). 4472a
  was killed by item 4475. 4464c and 4476a stay parked.
- **The sample keeps its seat** (§6).
- **Marked rows:** no `FABLE:` marker and no second reading in the
  tranche.

## What the pass looked at

The pair the pass before drew (`crop-1559b-units-1x.png`), again. No
commit of the tranche touches `crates/client` or `crates/render`, so the
still at the tip is that one, and nothing was redrawn or uploaded.

- **The tanks.** Theirs draws the two at the upper left washed and partly
  see-through, and the one at the lower right with tan dust at its base.
  The tank between them is crisp. Ours draws all four crisp. Item 4458
  showed the three are alive and at full health.
- **Two riflemen** in theirs carry an orange muzzle flash; ours none.
- **A framed portrait** stands over a figure near the upper-left tanks in
  theirs, and a name plate at the top-left corner. Ours draws neither on
  this frame. The portrait is not identified, and no record before this
  one names it.

## The queue's items

| item | kind, mark | the score | first act | it stops and says so when |
|---|---|---|---|---|
| 4480 held out, row 60 | rules, whole; the chain's | `Parity held out:` 5 → 6 of 8 | both sides round the `seen3` disc on 3983 and 3984, from run1722 | two landings running move no row's column |
| 4470 second, row 90 | rules, whole; a parity seat's | the second's column, one state on two lines | a ranges take, run1738 | two landings running move no column, or the disk reads under 60 GB |
| 4479 the sample, units 3921 | rules, whole; a parity seat's | `Parity:`, the sample's lowest first parting | unit 2190's figures on 3920 and 3921 | two landings running move no first parting |
| 4482 three tanks under a haze | render, scoped; the eye's | a row minted on the three tanks' pixels, `1/15` its control | the row, then the packet's `UnitData` and `Underlays` | the packet holds nothing apart between them |
| 4486 second, row 70 | rules, whole; unseated | the second's column, one line, 87 states | a ranges take, run1741 | two landings running move no column |
| 4478 held out, row 100 | rules, whole; unseated | row 100's column | deaths record 0 on 2755 and 2756 | two landings running move no column |
| 4483 second, rows 10 and 130 | rules, whole; unseated | two rows' columns | a ranges take of row 10, run1739 | two landings running move no column |
| 4484 held out, row 40 | rules, whole; unseated | row 40's column | a ranges take, run1723 | two landings running move no column |
| 4485 second, row 110 | rules, whole; unseated | row 110's column | a ranges take, run1742 | two landings running move no column |
| 4481 the packets' groups line | rules, whole; unseated | `PACKETS_SCORE` 853 of 1,053 | the information melee rows on the merged tree | two landings running turn no line |
| 4487 the third battery | rules, whole; at 4 of 8 on the second | a third found column, with its reach | the declaration, one commit before any take | — |

## Built

- `Parity:` on the scoreboard, held to `SAMPLE_LINES` by
  `the_handoff_s_parity_line_is_the_pinned_sample` in
  `crates/rondata/src/diff/sample.rs`, seen red on the queue without the
  line. It is in `release_gate.py`'s `COMMANDERS_LINES`.
  `HANDOFF_LINES` 33 → 34.
- The held-out lines' reach, in `held_out::queue_line`; both parsed lines
  were red until the queue carried it.
- The paperwork: entry 147, this record, the queue's handoff and eleven
  items, the loop's two clauses, `docs/RECGAME.md` §11's paragraph, the
  tracks' paragraph, the journal's entry, the suite's timings, the
  rubric's row.

## Not done, and said

- **No leaf is read for any row of the second battery, or for row 40.**
  None has a ranges take; five items take one each.
- **Row 30 of the second battery has no item.** It parts world on 1662,
  leaders on 1776 and units and guys on 2334.
- **The haze and the portrait are not identified.**
- **The sample's rate is named, not mended.** No faster instrument is
  proposed here.
- **Why a landing waits 62 minutes is as the record before said**: its
  own gate, 36.2 minutes, and a suite run, 14.8.
- **The driver's own cost is not summed**, as before.
- **The blind list was not scanned**: open 0.
- **The pairs were not redrawn or uploaded**; the picture is the pass
  before's.

## The gate

Whole, by `tools/gate_run.sh`, once; quoted here by the commit after it.
