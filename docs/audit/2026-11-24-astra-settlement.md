# The settlement of 2026-11-24: Astra's direct branch taken whole, the seam pins moved under it, and the disk

*2026-11-24, from 22:10 −05:00, the `steer` row on Fable 5.2, with Ramon
on the row. Not a pass: no lane ran between the pass of 2026-11-23,
third (`2026-11-23-fable-pass-bays.md`) and this, and `commander` had
not started. What it did is what DECISIONS 142 §2 says a direct branch
gets: fast-forwarded, gated whole here, pushed. It spawned no subagent
and took no capture. No decision entry: nothing was decided that an
entry does not already hold.*

## For Ramon

- **Taken whole.** `codex/nov24-handoff`, `codex/coverage-scout` and
  `codex/sample-merchant` are one chain of eighteen commits off
  `dc14bfc51`, items 4638 to 4648 and the sample's 4569. Fast-forwarded
  to `05ba34976`, gated whole, pushed; main is the same commit.
- **The headline stands at 50 of 63.** The 2017 sample's first parting
  went from leaders on 7502 to units on 10653 of 21,044, its draws
  7613 → 11488. The fifth's row 43 went 3833 → 3899.
- **No second reading is owed.** Every score rests on a diff against
  the sample's own takes, seen red under a mutation. Two readings stand
  alone and no score rests on either (below).
- **The seam pins had drifted and no gate builds them.** The fallback
  fires on five rows now, not four; the probe names an eighth piece.
  Items 4608 and 4618 carry the new figures.
- **Disk: 73 → 134 GB free**, nothing of the archive deleted. The
  archive's fold (29 GB) and what is outside this project still wait on
  your word; the list is `2026-11-23-pass-second/disk-report.md` under
  `~/ron-data/lab-experiments/`.
- **Astra stays off while `commander` runs** (your word on the row);
  item 4649, the sample's next, stays in the lanes' order.
- **The week** read 48 % at 22:35 −05:00, Fable's 29 %; the reset is
  2026-11-28, 06:00 −05:00.

## What the branch moved

Read whole, journal by journal, and the simulation's diff line by line.

| item | what | the sample's first parting | draws |
|---|---|---|---|
| 4638 | the booking clock under load: a bounded retry, the whole-host lease given priority | | |
| 4569 | Diamonds raises the commerce caps before the flat additions | 7502 → 7539 | 7613 |
| 4639 | the Scout's auto-explore bit and the cities' `scouted` word | 7539 → 7612 | 7613 |
| 4640 | unit and figure fields read together; `--library-slots 12` offered | | |
| 4641 | a trainee does not rally on a unit its owner cannot see | 7612 → 7935 | 7949 |
| 4642 | old building damage does not summon the army; the current alert does | 7935 → 8523 | 8534 |
| 4643 | the selected group's spell is dispatched and clears its formation | 8523 → 9234 | 9507 |
| 4644 | a unit selection is recorded in its hot-key slot | 9234 → 9506 | 9507 |
| 4645 | a figure regrown outdoors takes a move order | 9506 → 9602 | 9602 |
| 4646 | a factory's rally on an object, and the target's hot keys | 9602 → 10207 | 10207 |
| 4647 | a computer's garrison leaves an all-clear city on its player's phase | 10207 → 10261 | 11488 |
| 4648 | the gathering list keeps its allocated slots | 10261 → 10653 | 11488 |

Item 4642 also moved the fifth's row 43, 3833 → 3899, where builds
alone part: building 2010, `damage` ours 9, theirs 11 (run1787, range
1754). No other row's first parting moved. Past their first partings
some rows' agreeing counts moved both ways (row 53 items −866, row 83
items −1,124 across the batch); the journals carry each.

**The readings that stand alone**, each named so in its document, and
no score on either: where Diamonds sits against the nation percentages
and the wonders' additions (`docs/ECONOMY.md`; the recording fixes it
against Republic alone), and the regrown figure's search range, its
starting bearing and its fallback on exhaustion (item 4645; the
recording exercises a successful search at the centre).

**The weakened-check scan.** `#[ignore]` 17 before and after; clippy
`allow` 154 before and after; one assertion removed, the clock's old
wait, and 74 added; two `seam!` hooks removed with the code that closed
them (`unit_unseen`, `regrow_outside`). No float in `sim` (the guard
ran in the gate).

## The gate

Whole, by `tools/gate_run.sh`, once, on Astra's tip before the push.

- **`gate-pass-1124-astra.log`: `gate exit 0`** on `05ba34976`, 9 of 9
  steps: offline 1.0, clippy 0.2, guard 1.7, merge pins 1.1, release
  23.3, clock 0.1. `Clock: green beside load`. Admission 9.7 s over
  1,608 leases; load 4.1 at its start and 9.9 at its end. `Fixture
  audit: 207950 requests, 0 missing fixtures`. `Driver: 0 landings
  since the last pass`. The oracle step removed 130 stale generations
  (Astra's parser change had written a new one).
- The record's commit changes the queue, this file, the suite's
  timings (17 new tests timed) and `held_seams::FALLBACK`, which no
  gate builds; the paperwork guard and the seam tests ran on it.

## The seam table

`held_seams` under `seam-count`, 56 walks, 260 s, on `05ba34976`: four
green and three red on the fallback's pin, as below. Re-pinned and run
again, 257 s: seven green (`held-seams-3.log` under
`~/ron-data/lab-experiments/2026-11-24-settlement/`).

| hook | at the pass of 2026-11-23, third | on `05ba34976` |
|---|---|---|
| `launch::fallback` | 1,796 rounds: row 88 991, row 133 642, row 148 143, row 48 20 | 2,169: row 88 1,392, row 133 563, row 148 143, row 48 21, **the fifth's row 83 50 from state 5714** |
| `attack_ground_aim` | rows 93 (486), 88 (135), 133 (575), 53 (1,012), 83 (291) | rows 93 (270), 88 (2,689), 133 (580), 53 (555), 83 (165, first on 4853) and **the seventh's row 48 (3, on 11998)** |
| `pivot::offset_at_unpinned` | 1,385 on rows 48 and 133 | 1,523: row 48 850, row 133 673 |

Every entry is past its row's first parting but row 93's
`attack_ground_aim` (2962, three states before 2965), and no whole row
enters any of the three. Each row's walk past its parting is its own
game, so these counts move with any fix upstream.

**The bay probe** (a print on the fallback, the fifth's, sixth's and
seventh's score tests, taken out after): 50, 563 and 1,556 lines, the
counter's totals. The seven pieces and 55 keys of the last table stand;
an eighth is new, piece 238 on row 83, twelve keys (animation 12, six
start times, nodes 0 and 1). The keys are in `bay-probe-fifth.out`,
`-sixth.out` and `-seventh.out` under the settlement's directory.

**The first seam run died on the compressor.** It was started beside
`tools/archive_compress.py` with its log under `~/ron-data`; the
compressor rewrote the open log and the run exited 120 with two lines
written. Run again after the compressor ended. A log a running command
writes belongs outside the trees being compressed.

## The disk

| what | free after |
|---|---|
| as found | 73 GB |
| Astra's `target/` in its worktree, on Ramon's word | 98 |
| the gate's oracle step, 130 stale generations | 107 |
| the gate tree's debug build, both trees' incremental caches, the integration tree's debug profile (8.8 GiB), the diff-failure exports | 128 |
| the archives compressed, one tree at a time: `scan: 0 damaged, 0 standing` | 136 |

Five files stay raw over the compressor's limit: the four of the last
pass and one ranges record under
`lab-experiments/2026-11-18-codex-combat-replays/`, 2.1 GB, which no
test reads. A tranche's four lane trees take about 24 GB back.

## What was not done

- **No pass ran**: no rubric row, no gates table, no blind scan, no
  still drawn. The next pass's base for its tables is the pass of
  2026-11-23, third.
- **The thirteen open games were not scored again by leaf**; Astra's
  held-out runs report every first parting but row 43's unmoved, and
  the gate's pins hold them.
- **Astra's spend is not in `lore`**; the branch has no price here.
