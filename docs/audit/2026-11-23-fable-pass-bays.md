# The pass of 2026-11-23, third: a named fix is seated first, the near-end rank goes, and the compressor takes one tree at a time

*2026-11-23, from 22:35 −05:00, the `steer` row on Fable 5.2, the
drain's opener. `commander` (Opus 5.5) handed over at `86a8d3da3` after
nineteen first-parent merges (items 4613, 4610, 4612, 4614, 4620, 4621,
4622, 4619, 4624, 4623, 4627, 4625, 4626, 4628, 4629, 4631, 4632, 4630
and 4611), gated whole (`gate-drain-1123d.log`), with no lane out. The
charter was the handoff's `For the pass` part. Decisions in entry 156.
The pass spawned no subagent and took no capture. The pass before it
is `2026-11-23-fable-pass-fog.md`.*

## For Ramon

- **The headline is 50 of 63, from 48 of 55.** The new battery's eight
  games joined the count, one of them whole as found and one made
  whole. Nineteen landings in six hours, 6.76 USD each.
- **A landing on a game bought 43 agreeing states, against 163 and
  621 the two tranches before.** Thirteen landings made one game whole.
- **Three of those thirteen redid work the last pass had already
  listed.** It had printed five unit models with no launch point for
  their projectiles and filed them as one job, which never got a seat.
  Three lanes each found one of the five again, each with its own
  capture. Those three bought more than half the tranche's states. A
  job a tool has already named now goes first.
- **My bet from the last pass lost.** I seated a game that went wrong
  292 states before its end, betting it would finish in two landings.
  It took four. The rule is withdrawn, as it said it would be.
- **I damaged four archive files and repaired them.** The archive
  compression I run at the start of every pass zeroed four trace logs.
  The cause was last pass's change that stores each capture once, under
  two names: the compressor mishandles a file reached by both. Nothing
  was lost. The data was still in each file, and three of the four
  match their recorded checksums byte for byte; the fourth has no
  recorded checksum and plays back whole. The compression now runs
  through a tool that cannot do this and checks afterwards.
- **The client's seat draws the capital's ground next**: the Farm's
  wheat plots and the paving under the city's buildings, which the
  original shows and ours does not. The pair:
  `https://cdn.ramonfabrega.com/attrition/pass-1123c/`. Say the word
  to put the seat on a parity game instead.
- **Yours to decide, as before:** the 19 GiB of captures stored twice
  from before the change, and the merged lane branches on origin.
- **The week** read 44 % at 22:38 −05:00 on its third day. Fable's
  week read 21 % before this pass.

## What was measured

| what | measured |
|---|---|
| the drain's gate (`gate-drain-1123d.log`) | `gate exit 0` on `86a8d3da3`, HEAD = origin/main = origin/worktree-replan-pdb; whole, 9 of 9 steps, 27.5 minutes, release 23.5; `Driver: 19 landings since the last pass (backstop 20) · context 283 k`; 80 GB free after it |
| the merges (`git log --first-parent --merges e23a6e911..86a8d3da3`) | 19, 16:12 to 22:06 −05:00 |
| the lanes (`tools/tranche.py --base e23a6e911`; `tables-1123d-tranche.txt`) | 19 sessions over 19 items, 128.48 USD, 6.76 a landing; a session's clock 66.7 minutes, 13.8 working and 52.9 waiting, 32.4 on its own gate and 10.6 on a suite run; peaks 167 to 386 k, three past 300 (4614 at 322, 4619 at 386, 4624 at 316); 8 capture sections |
| the gates (`tools/gates.py … --since gate-pass-1123b.log`) | 9, 267.8 minutes: eight scoped, all widened, and the drain's whole gate; one red, `gate-4629`, 32.7 minutes, on `the_seventh_held_out_games_score_as_pinned` reading run1810 in flight; admission 16,401.1 s summed, longest 975.5 s; one ended with the load past 16 |
| what a seat bought (the bookings' own lines) | thirteen landings on a held-out row: 553 states before a first parting (row 28 292 over four and whole, row 88 161 over four, row 133 100 over five), 43 a landing; 313 of the 553 on three landings that built `BAYS` rows (4625 92, 4626 140, 4630 81) |
| the thirteen games not whole (`score_recording` at `86a8d3da3`) | exit 1 on each, a measured difference, 9 to 53 seconds; every report's first parting equals its pin: row 133 builds 2451, row 88 units 2902, row 93 units 2965, row 128 leaders 3338, row 53 units and guys 3358, row 83 3662, row 43 3833, row 48 ammo 5607, row 108 7421, the sample leaders 7502, row 13 7675, row 8 units 9896, row 148 units 10879. Row 128 holds eleven lines whole to 12,001 and leaders on 8,154. Row 88's first run answered exit 2, `incomplete trace header or record`, on a trace log reading zero bytes (below), and exit 1 after the repair; under `~/ron-data/lab-experiments/2026-11-23-pass-third/` |
| the seam table (`held_seams`, feature `seam-count`, seven tests, 255 s, 56 walks) | five green; the sixth red, `left: Some([(133, 642)])`, `right: Some([(133, 455)])`; the seventh red, `left: Some([(48, 20), (88, 991), (148, 143)])`, `right: Some([(28, 41), (48, 20), (88, 2092)])`. Entry 156 §3's table: `attack_ground_aim` on five open rows and no whole row, `launch::fallback` on four, `pivot::offset_at_unpinned` on two. Run again on the re-pinned figures (`held-seams-2.log`, 254 s): seven green |
| the bay probe (a print on `launch_point`'s fallback, the sixth's and the seventh's score tests) | exit 0, 414 s, both tests green with the print in; 1,796 lines, the counter's own total: piece 128 on row 88 (six keys, 991), 37 on row 148 (three, 143), 10 on row 48 (one, 20), and 206, 1133, 1485 and 1499 on row 133 (two, thirty, ten and three keys, 642); seven pieces, 55 keys, in `bay-probe.out`; the print taken out after, `crates/sim` clean |
| the archives (`split.py --archive`, then `afsctool` over both trees in one run) | nothing to split; four files skipped over the compressor's limit, the same four; five trace logs printed `Resource temporarily unavailable`; 107 → 111 GB |
| the four trace logs (`stat`, `xattr`, then `os.chflags`) | run1813, run1815, run1818 and run1819's `rontrace.log`, each with a second name under `Logs`: size 0, 33.2, 13.7, 11.4 and 14.4 MB of blocks, `com.apple.decmpfs` 16 bytes (`fpmc`, type 12, declaring 246,192,512, 100,517,856, 81,561,344 and 99,457,184 bytes), the flag off. With the flag set each reads to its declared length; the first three match the SHA-256 in `docs/RUNS.md`; run1819 scores 2,383 of 2,383 states, exit 0. The forks as found are under the pass's directory, `fork-backup/` |
| the race, made again (24 pairs of 44 MB under two names, `afsctool -c -f -T LZFSE -J8` on all 48) | 17 zero-length with the flag off, 7 whole, 7 lines of `Resource temporarily unavailable`; three pairs compressed a directory at a time: whole |
| the scan after (`archive_compress.py`'s, 334,847 files) | 0 damaged |
| the week (`lore limits`) | 44 % at 22:38 −05:00, reset at 06:00 −05:00 on 2026-11-21; Fable's week 21 % |
| the suite re-timed (`suite_timings.py --stale-only`) | 7 tests timed: 2,689 tests, 6,738 s one at a time |
| the rubric (`rubric.py --record`) | every figure the pass before's: ported 739, open 570, lockstep bytes 78.1 %, unit types seen 296 of 302 |
| the rows and trees left | the old driver row reaped by `handover.py reap f0c56367`; no lane row stood; `reap_agents.py` found no builder tree; three branches on origin not in HEAD (`rescue-234`, `worktree-att-1565-opus`, `worktree-att-4537`), as before |

## Rulings

- **An item whose first act an instrument has already named takes a
  seat before any row's next word** (entry 156 §2): item 4608 on the
  seam five open rows enter, item 4618 on the pieces with no bay.
- **The near-end rank goes** (§4): row 28 came whole on its fourth
  landing where the clause bet two.
- **A take in flight is not scored** (§5): a trace with no receipt, on
  a row with no pin, is skipped and said.
- **The compressor takes one tree at a time**, and the pass scans
  after it (§6).
- **The client's seat takes the capital's ground** (§7; item 4636).
- **A battery folds on its declared rows** (§8); `Parity:` stands at
  50 of 63.
- **Four seats**: the chain's on 4635 (row 128, one line from whole),
  parity seats on 4608 and 4618, the client's on 4636.
- **Nine parked findings**: two are item 4636's, two were closed in
  the tranche, the rest stay in their journals (§10).
- **Marked rows:** no `FABLE:` marker and no second reading in the
  tranche.

## What the pass looked at

Five of the nineteen landings drew the fog (4612, 4621, 4624, 4627,
4629). The pass set run1814b's photograph beside item 4629's still of
the same camera and frame (`run1814b-theirs.png`, `run1814b-ours.png`
under the pass's directory, and uploaded). No client source has
changed since that still. The fog's soft edge falls where theirs
does, on the lake, the grass and the mountain, and the minimap is
black but for the capital's patch on both. On the lit ground: theirs
has a block of wheat plots south-west of the city with three Citizens
on them, and ours has the three Citizens on bare grass; theirs has a
sandy plate under each building and ours has grass to the walls;
theirs has the city's name plate and ours none; the trees in the dark
at the lower left are dark in theirs and lit in ours. The still
carries no HUD, so the bar and the panels were not judged.

## The queue's items

| item | kind, mark | the score | first act | it stops and says so when |
|---|---|---|---|---|
| 4635 seventh, row 128 | rules, whole; the chain's | the seventh 2 → 3 of 8, `Parity:` 51 | one take, `0x80u`, to 12,000 | two landings running move no column |
| 4608 sixth, row 93 | rules, whole; a parity seat's | row 93's column from 2965; rows 88, 133, 53 and 83 scored again | the seam's fix: an attack-ground order with no aim | two landings running move no column |
| 4618 the bays | rules, whole; a parity seat's | `held_seams::FALLBACK` 1,796 rounds on four rows toward none | the probe's keys built in one act | it has landed its two acts |
| 4636 the capital's ground | render, scoped; the client's | a new row on run1814b's photograph | a reading off the packet's draws | the packet holds no draw for either |
| 4634 sixth, row 133 | rules, whole; unseated, first | row 133's column from 2451 | the leaf on run1797 | two landings running move no column |
| 4633 seventh, row 88 | rules, whole; unseated | row 88's column from 2902 | the leaf on run1815 | two landings running move no column |
| 4609 sixth, row 53 | rules, whole; unseated | row 53's column from 3358 | a packet at 3356 | two landings running move no column |
| 4573, 4571 fifth, rows 83 and 43 | rules, whole; unseated | each row's column | the leaf on run1784, run1787 | two landings running move no column |
| 4615, 4616, 4617, 4637 seventh, rows 48, 108, 8, 148 | rules, whole; unseated | each row's column | a take, `0x8u`, `0x63u`, `0x1u`, `0x1u` | two landings running move no column |
| 4569 the sample | rules, whole; unseated | `Parity:`, leaders 7502 | the leaf on run1786 | two landings running leave the first parting and the draw word |
| 4579 sixth, row 13 | rules, whole; unseated | row 13's column from 7675 | a take, `0xfebu` | two landings running move no column |

## Built

- `tools/archive_compress.py`: split, one compressor run a tree, a
  scan of both for a file that reads empty and holds blocks, and
  `--repair`. Four tests in `tools/explore/test_archive_compress.py`;
  two seen red with both trees in one run (`one tree a run`, and the
  scan finding `take5.log` = `filed5.log`).
- `crates/rondata/src/diff/held_out.rs`: `in_flight`, and the scorer
  skips an unpinned row whose take has no receipt. One test, seen red
  on the predicate reversed (`no receipt yet`), in the timings now.
- `crates/rondata/src/diff/held_seams.rs`: the fallback's pins read
  `(133, 642)` and `(48, 20), (88, 991), (148, 143)`.
- The paperwork: entry 156 and its index row, this record, the queue's
  handoff, lanes line and fifteen items, the loop's clauses (the first
  act's tool, the named fix's seat, the near-end rank struck), the
  tracks' paragraph, the journal's entry, the suite's timings, the
  rubric's row.

## Not done, and said

- **No seam is shown to be a row's cause.** `attack_ground_aim` is
  entered three states before row 93's parting and past the other
  four rows'. Item 4614's was the same evidence and moved five lines
  without moving its row's first parting.
- **The in-flight skip has not met a take in flight.** Its test builds
  a directory. A take that dies without a receipt on an unpinned row
  is skipped too, and only the fixture audit's count says so.
- **The compressor's race was made again on scratch files, not on the
  archive.** That the archive's four came from this pass's run and not
  an earlier one rests on the files having scored that afternoon.
- **The capital's ground is a look, not a reading.** Nobody has read
  what lays the plots or the plates.
- **Nothing of rows 8, 48, 108, 148 or 13 is read.** Their takes hold
  sums.
- **No gate builds the seam feature**, so its pins hold only until
  the next landing moves a row.
- **No still was drawn at the tip.** The pair is item 4629's still and
  the photograph on file.
- **`tools/ctx.py` still reads the driver's context from a builder**,
  the four files over the compressor's limit stay raw, the driver's
  own cost is not summed, and the blind list was not scanned (open 0),
  all as before.

## The gate

Whole, by `tools/gate_run.sh`, once.

- **`gate-pass-1123c.log`: `gate exit 0`** on `e2259006b`, the pass's
  commit, 9 of 9 steps: offline 1.0, clippy 0.1, guard 0.6, merge pins
  1.1, release 23.3, clock 0.1. `Clock: green beside load`. Admission
  13.7 s over 1,591 leases; load 3.6 at its start and 8.5 at its end.
  `Fixture audit: 207068 requests, 0 missing fixtures`, where the pass
  before read two: run1810's and run1811's traces are taken. `Rows: one
  driver row at most: steer be2dae47 (working)`; `Driver: 0 landings
  since the last pass (backstop 20) · context 300 k`; `Trunk:
  origin/main is origin/worktree-replan-pdb`. `Builders: 0 reaped`; 117
  GB free by its count, 109 by `df -g ~`.
- The commit that quotes this is documents only and ran no whole gate;
  the reflex, `tools/guard.sh`, was green on it.
