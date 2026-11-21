# The pass of 2026-11-21: every row not whole scored, the takes cut to the lines that part, and the sample's seat pointed at its score

*2026-11-21, from 07:38 −05:00, the `steer` row on Fable 5.2, called by
the backstop. `commander` (Opus 5.5) handed over at `5013bdf5e` after
twenty-two first-parent merges (items 4470 and 4478 to 4501, with 4483),
drained and gated whole (`gate-drain-1121b.log`). The charter was the
handoff's: score the held-out rows, the batteries and the sample on
disk, design the seats the lanes line left empty, and rule on the fold.
Decisions in entry 148. The pass spawned no subagent and took no
capture. The pass before it is `2026-11-20-fable-pass-reach.md`.*

## For Ramon

- **The fold is still yours, and it reads 13 of 15 today.** This is the
  fourth record to put it to you. Six of the seven and seven of the
  first eight are whole. I still recommend folding. All three batteries
  folded would read 23 of 31.
- **The newest unseen games came out much better.** The third battery
  read 5 of 8 whole on its first try, against 1 of 8 and 0 of 8 for the
  two before it. An unseen game now runs 88 % of its states before it
  first parts, against 53 % and 27 %.
- **That number flatters us, and you should know why.** Every one of the
  twenty-four held-out games has an idle human. The one game with a
  person's orders, the 2017 sample, parts a fifth of the way in (state
  4,248 of 21,044). A battery with seats and a team is yours to ask for.
- **The sample's score did not move for four landings**, while the lanes
  advanced a different number on the same game. I pointed the seat back
  at the score. The scoreboard prints both numbers now.
- **The disk is the next limit.** It read 67 GB at the pass's start and
  75 after cleaning, against a floor of 60. The captures the lanes keep
  were about 4 GB each. I cut them to the lines that actually differ.
  Nothing was deleted from the archive; that stays yours.
- **A fourth battery is named** (rows 15, 35, … 155), taken when the
  third reads 7 of 8.
- **The picture did not change this tranche.** Two landings read the
  tanks' dust and drew nothing. The eye's seat builds it now, from the
  packet.
- **The tranche:** 22 lane sessions, 127.03 USD, 5.77 a landing. A
  landing waited 49.3 minutes, down from 62.0. Three gates of 22 were
  red, each on the driver's own paperwork, 2.2 minutes lost in all.
- **The week:** it reset at 06:00 −05:00 today and read 1 % at the pass.
  Fable's week read 0 %.

## What was measured

| what | measured |
|---|---|
| the drain's gate (`gate-drain-1121b.log`) | `gate exit 0` on `5013bdf5e`, the tip, HEAD = origin/main = origin/worktree-replan-pdb; whole, 9 of 9 steps, 25.2 minutes, release 21.4; `Driver: 22 landings since the last pass (backstop 20)`, context 231 k |
| the merges (`git log --first-parent --merges fcf68acc0..5013bdf5e`) | 22 |
| the lanes (`tools/tranche.py --base fcf68acc0`, after `lore index`) | 22 sessions over 22 items, 127.03 USD, 5.77 a landing; a session's clock 65.1 minutes, 15.8 working and 49.3 waiting, 30.5 on its own gate and 6.8 on a suite run; peaks 161 to 276 k, none past 300; 12 captures |
| the gates (`tools/gates.py … --since gate-pass-1120c.log`) | 22, 408.9 minutes: 2 whole, 20 scoped (18 widened), 3 red; admission 26,305.2 s summed, longest 997.5 s; 6 ended with the load past 16 |
| the three reds | `gate-book-4470` and `gate-book-4498`, each stopped at the first step: the queue referred to a number never booked; `gate-book-4496`, at the guard: the scoreboard and the endpoint lines against their pins. 0.0, 0.0 and 2.2 minutes; each booking was green on its next gate |
| the seven rows not whole (`score_recording` at the tip, 9 to 16 s each) | entry 148 §3's two tables; exit 1 on each, a measured difference |
| the three batteries (`held_out`'s parsed lines, read) | as found 0, 1 and 5 of 8 whole, 64, 65 and 90 of 96 lines, reach 27, 53 and 88 %; now 7, 4 and 6 of 8, 95, 82 and 92 lines, reach 93, 68 and 92 % |
| the sample (`SAMPLE_LINES`, `SAMPLE_WALKED`, read; green on the drain's gate) | lowest first parting units 4248, then builds and leaders 4321, ammo 4462, world 5256, guys 5273; the draw word 5273; the newest ranges take, run1724, ends on 4130 |
| the takes on disk (`du -sh ~/ron-golden/run17*`) | eleven that hold a held-out row or the sample, 37.9 GB between them, 2.2 to 5.2 each; `~/ron-golden` 106 GB, `~/ron-data` 40 |
| the disk (`df -g ~`) | 67 GB at the start, under the last pass's 102; 73 after `cargo clean --profile dev` (12.5 GiB); 75 after `afsctool` over both archives |
| the week (`lore limits`, 07:40 −05:00) | 1 %, reset at 06:00 −05:00 on 2026-11-21; Fable's week 0 % |
| the suite re-timed (`suite_timings.py --stale-only`) | 13 tests timed: 2,584 tests, 5,932 s one at a time |
| the rubric (`rubric.py --record`) | every figure the pass before's: ported 738, replaced 331, open 571 |
| the rows and trees left | none: no lane row stood, `reap_agents.py --yes` printed nothing, the old driver row reaped by `handover.py reap 5ba5462e`; Codex's branch is not ahead of HEAD |

## Rulings

- **The fold is Ramon's; recommended, 13 of 15 folded** (entry 148 §1).
- **The fourth battery is rows 15 to 155**, named before any take, taken
  when the third reads 7 of 8 (§2; item 4510).
- **Four seats** (§3): the chain's on row 40, a parity seat on row 105,
  a parity seat on the sample's ranges, the eye's on the dust. Seven
  items wait behind them, row 5 first.
- **A row's take keeps the lines its score parts on** (§4). The scorer
  prints the arguments.
- **The sample's seat reads the `Parity:` line** (§5): the ranges past
  the draw word first, then the word's next act. A landing that moves
  neither the lowest first parting nor the word counts toward the stop.
- **The dust is renderer state, taken from the packet for a still**
  (§6). The haze's line ended by the stop rule after 4482 and 4490, and
  the pass seats a different item: a build, where the two before read.
- **The eleven parked findings stay in their journals** (§7). None names
  a score.
- **Marked rows:** no `FABLE:` marker and no second reading in the
  tranche.

## What the pass looked at

The pair the pass of 2026-11-20 drew (`pair-1559b-half.png`,
`crop-1559b-units-1x.png`), again. The tranche's one change under
`crates/client` and `crates/render` is the haze row's reader
(`silhouette.rs`, 148 lines), so the still at the tip is that one, and
nothing was redrawn or uploaded.

- **The tanks.** As the record before said: three hazy with tan dust in
  theirs, one crisp, and all four crisp in ours. Item 4482 named the
  haze, and item 4490 showed why our walk cannot time it.
- **The city, selected in theirs.** A pale yellow beam stands at the
  plaza's lower corner in theirs and ours draws none. Ours' still has no
  selection, so it may be the selected city's marker. It is not
  identified, and no row counts it.
- **Two riflemen's muzzle flash, the framed portrait and the name
  plate**: as the record before said, and still not identified. Item
  4506's first act prints the packet's emitters for the two riflemen.

## The queue's items

| item | kind, mark | the score | first act | it stops and says so when |
|---|---|---|---|---|
| 4484 held out, row 40 | rules, whole; the chain's | `Parity held out:` 7 → 8 of 8 | one take of the units line, run1723, `RNG_MASK=0x1u` | two landings running move no column |
| 4502 third, row 105 | rules, whole; a parity seat's | the third's column, three lines | the leaf of units 2983, on run1752's disk | two landings running move no column |
| 4505 the sample, ranges to 4500 | rules, whole; a parity seat's | `Parity:`, units 4248 | one take of four lines, run1726, `RNG_MASK=0x8bu` | two landings running leave `Parity:` as it is; 4501 was the first |
| 4506 the dust from the packet | render, scoped; the eye's | `RUN1559B_3279_TANKS_HAZE` 1 → 4 of 4 | the packet's emitter lists, tanks and riflemen | one landing leaves the row at 1 of 4 |
| 4507 third, row 5 | rules, whole; unseated | the third's 6 → 7 of 8 | one take of the ammo line, run1753, `RNG_MASK=0x8u` | two landings running move no column |
| 4504 second, row 10 | rules, whole; unseated | row 10's column, five lines | the leaf of units 1747, on run1739's disk | two landings running move no column |
| 4508 second, row 130 | rules, whole; unseated | row 130's column, two lines | score it again, then a take, run1754, `RNG_MASK=0x81u` | two landings running move no column |
| 4509 second, row 30 | rules, whole; unseated, after 4504's world leaf | row 30's column, two lines | score it again, then a take, run1755, `RNG_MASK=0x801u` | two landings running move no column |
| 4485 second, row 110 | rules, whole; unseated | row 110's column, five lines | a take, run1742, `RNG_MASK=0x88bu` | two landings running move no column |
| 4503 the sample's draw word | rules, whole; behind 4505 in its seat | `Parity:`, draws 5273 | ours' path stack for `1/22` on 5272 | two landings running leave `Parity:` as it is |
| 4510 the fourth battery | rules, whole; at 7 of 8 on the third | a fourth found column, with its reach | the declaration, one commit before any take | — |

## Built

- `playback::joined_take` and the scorer's last line, `one take for
  every parted line: …`, printed when a take holds sums and lines part.
  `a_joined_take_keeps_the_parted_lines_and_no_others` holds it, seen red
  on a mask of the lines that are whole. Run on the five rows with no
  ranges it printed `0x1u`, `0x8u`, `0x81u`, `0x801u` and `0x88bu`.
- `Parity:` carries the draw word (`· draws 5273`, off `SAMPLE_WALKED`).
  `the_handoff_s_parity_line_is_the_pinned_sample` was red on the drain's
  queue until the line carried it.
- The paperwork: entry 148 and its index row, this record, the queue's
  handoff and eleven items, the loop's three clauses, `docs/METHOD.md`'s
  two notes, `docs/RECGAME.md` §10's line and §11.1's paragraph, the
  tracks' paragraph, the journal's entry, the suite's timings, the
  rubric's row.

## Not done, and said

- **No leaf is read for rows 40, 5, 130, 30 or 110.** None has a ranges
  take; five items take one each.
- **A masked take's size is not measured.** Seven earlier takes used a
  narrow mask, and none of them is one of these rows. Item 4484's take is
  the first, and its `df -g ~` before and after is the measure.
- **Why row 10's ammo line is unemitted on 5178 is not read**; the scorer
  says ours holds no line there and theirs three ranges.
- **The beam, the flash and the portrait are not identified.**
- **The sample's seat is re-pointed, not sped up.** Whether the four
  lines below the word are cheap leaves, as the held-out rows' were, is
  item 4505's to find.
- **The idle human is named, not mended.** No battery with a person's
  orders is designed here; seats and a team are Ramon's.
- **The driver's own cost is not summed**, as before.
- **The blind list was not scanned**: open 0.
- **The pairs were not redrawn or uploaded**; the picture is the pass of
  2026-11-20's.
- **The guards caught the pass three times**: an item naming a function
  the export does not hold, the handoff 553 bytes over its bound, and
  entry 148 without an index row. All three were mended before the gate.

## The gate

Whole, by `tools/gate_run.sh`.

- **`gate-pass-1121.log`: `gate exit 0`** on `04f7466b0`, the pass's
  commit, 9 of 9 steps: offline 1.0, guard 0.6, merge pins 1.1, release
  22.1, clock 0.1. `Clock: green beside load`. Admission 11.1 s over
  1,493 leases; load 2.0 at its start and 7.7 at its end. `Fixture audit:
  198791 requests, 0 missing fixtures`. `Rows: one driver row at most:
  steer 7e53811d (working)`; `Driver: 0 landings since the last pass
  (backstop 20) · context 269 k`. `Builders: 0 reaped`; 78 GB free after
  it.
- The paperwork guards had run on the tree before the commit, red three
  times first ("Not done, and said").
