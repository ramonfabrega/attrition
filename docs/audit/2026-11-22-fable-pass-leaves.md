# The pass of 2026-11-22, second: two items held one leaf, eight pieces had no bay, and the sixth battery runs to 12,000

*2026-11-22, from 14:50 −05:00, the `steer` row on Fable 5.2, called by
the drain one landing short of the backstop. `commander` (Opus 5.5)
handed over at `e5fc6b3e0` after nineteen first-parent merges (items
4541, 4543, 4544, 4550, 4549, 4542, 4546, 4552, 4545, 4554, 4555, 4553,
4551, 4557, 4547, 4556, 4558, 4561 and 4560), gated whole
(`gate-drain-1122-2.log`). The charter was the handoff's: seat five
items and rule on the sixth battery. Decisions in entry 152. The pass
spawned no subagent and took no capture. The pass before it is
`2026-11-22-fable-pass-fold.md`.*

## For Ramon

- **The headline is 41 of 47, up from 39.** Two more games agree with
  the original on every state. Nineteen landings in eight hours, 7.54
  USD each on the lanes, down from 8.17.
- **The next unseen battery is recorded to 12,000 frames, not 6,000,
  and you confirmed it.** Every unseen game so far stops at 6,000,
  which is under seven minutes of play, and four batteries are whole
  inside that. The one long game we have, the person's, first differs
  at state 7,055, past the end of all forty. So the batteries have
  been measuring only the opening.
- **That battery's human plays from a script, on your word.** You
  allowed removing the seat too; I kept it, since a person's orders
  are the thing untested.
- **The person's game is moving slowly.** It gained 947 states over six
  landings, and less each time: 507, 259, 93, then 34. It has 13,989
  states to go. At this tranche's average that is about 89 landings and
  650 USD; at the last landing's pace it is several times that. You
  asked what to do: nothing yet. The sixth battery's first scores say
  whether that seat belongs on its games instead (entry 152 §5).
- **Two queued items were the same bug.** The person's game and one
  battery game differ on the same two fields. They now share one seat.
- **One fix replaces eight.** Three times a lane found a unit whose
  shots start from the wrong point and fixed that one unit. I listed
  every such unit in the games we hold: eight. One item builds them all.
- **The next battery is one small fix away.** One game of the fifth
  differs on a single field; when it lands the fifth reads 4 of 8 and
  the sixth is taken, with no pass needed.
- **The week** read 21 % at 15:02 −05:00, the second day of its window.
  Fable's week read 8 %.

## What was measured

| what | measured |
|---|---|
| the drain's gate (`gate-drain-1122-2.log`) | `gate exit 0` on `e5fc6b3e0`, the tip, HEAD = origin/main = origin/worktree-replan-pdb; whole, 9 of 9 steps, 25.6 minutes, release 22.0; `Driver: 19 landings since the last pass (backstop 20)`, context 372 k; `Trunk: origin/main is origin/worktree-replan-pdb` |
| the merges (`git log --first-parent --merges 25a90ab76..e5fc6b3e0`) | 19, 06:31 to 14:23 −05:00 |
| the lanes (`tools/tranche.py --base 25a90ab76`, after `lore index`) | 13 sessions over 13 items, 97.97 USD, 7.54 a landing; a session's clock 102.6 minutes, 19.7 working and 82.9 waiting, 46.4 on its own gate and 15.5 on a suite run; peaks 186 to 313 k, three past 300 (4542, 4549, 4553); items 4541, 4550, 4546, 4551, 4547 and 4561 were in-session builders and have no session |
| the gates (`tools/gates.py … --since gate-pass-1122.log`) | 16, 378.0 minutes: 1 whole, 15 scoped (15 widened), 3 red; admission 29,320.4 s summed, longest 1,266.3 s; 4 ended with the load past 16 |
| the three red gates | `gate-book-4541-4543`, 7 of 9 steps, the census pin (`SIMULATION_PORT_BACKED` 738 → 739), green as `…-4543b`; `gate-book-4550`, 6 of 9, the handoff 16,004 of 16,000 bytes, green as `…-4550b`; `gate-book-4553-4551`, 4 of 9, `cargo fmt` on two files, green as `…-4551b` |
| the `Parity:` line at each booking (`git show <booking>:docs/QUEUE.md`) | the sample's first parting units 6108, units 6114 (4543), builds 6162 (4549), units 6669 (4552), 6928 (4555), 7021 (4557), 7055 (4560); draws 6114, 6114, 7021, 7021, 7021, 7021, 7105; the count 39, 40 (4550) and 41 of 47 (4546) |
| the fifth's line at each booking | 2 → 3 of 8 whole (4546); lines 32 → 41 → 48 → 53 of 96; reach 62 → 81 % |
| the five rows not whole (`score_recording` at the tip) | entry 152 §1's table; exit 1 on each, a measured difference; the reports are under `~/ron-data/lab-experiments/2026-11-22-pass-second/` |
| the fallback of `launch_point` (a scratch print, reverted; the five batteries' score tests and the sample's walk) | 0 rounds on the first four batteries; 1,011 on the fifth from pieces 1528, 1224, 73, 231 and 198; 223 on the sample from 921, 1193 and 148; 55 keys in all; with the frame printed, piece 1224 first on frame 2859 (row 103), 198 on 3091, 1528 on 3360 (row 43), 231 on 3997 and 73 on 4252 (row 83); the sample's none before state 8000 (`RON_SAMPLE_WALK` at 7053, 7057, 7200 and 8000) |
| the probe's first run | six test runs, `running 0 tests` each, 0 rounds: the filter was `--exact` on a short path |
| the archives (`split.py --archive`, then `afsctool` over both) | 37 records, nothing to split; four files skipped over the compressor's limit: the three text dumps as before, and one ranges record of 2,148,618,992 bytes under `lab-experiments/2026-11-18-codex-combat-replays/`, which the splitter does not walk; 96 GB free to 98 |
| the disk (`df -g ~`) | 96 GB at the start; 98 after the archives; 105 after `cargo clean --profile dev` (11.7 GiB) |
| the week (`lore limits`) | 20 % at 14:52 −05:00 and 21 % at 15:02, reset at 06:00 −05:00 on 2026-11-21; Fable's week 7 %, then 8 % |
| the suite re-timed (`suite_timings.py --stale-only`) | 15 tests timed: 2,644 tests, 6,290 s one at a time |
| the rubric (`rubric.py --record`) | ported 738 → 739, open 571 → 570 (item 4541's `is_jammed`); every other figure the pass before's |
| the rows and trees left | no lane row stood; the old driver row reaped by `handover.py reap 0c7ea2a7`; no builder tree; no Codex branch ahead of HEAD |

## Rulings

- **Four seats** (entry 152 §1): the sample's on item 4565, parity
  seats on rows 63 (4563) and 83 (4562), the chain's builder on item
  4559. Behind them, in order: 4566, 4564, 4548.
- **Items 4565 and 4564 name one leaf and take one seat** (§2). Item
  4565 carries both scores; 4564 is scored again at its booking.
- **The bays are built by class** (§3): eight pieces in one act. Row
  103's first parting is confirmed as one of them by frame.
- **The sixth battery keeps its trigger, 4 of 8** (§4). Its item is
  written, so the driver seats it with no pass.
- **The sixth is recorded to 12,000** (§4), with each row's reach in
  its first 6,001 states beside the whole. Ramon confirmed it.
- **The sixth's human is scripted** (§4; Ramon, on the row). The item's
  first act reads whether the minting serves a held-out lobby.
- **The sample's seat is re-decided on the sixth's found column** (§5).
- **The sample's stop rule stands** (§5). Every landing moved the line.
- **The reflex asks the format** (§6).
- **The fourteen parked findings stay in their journals** (§8). None
  names a score.
- **Marked rows:** no `FABLE:` marker and no second reading in the
  tranche.

## What the pass looked at

Nothing was drawn. The tranche has no render landing: all nineteen are
rules items. The pairs of 2026-11-20 were not redrawn or uploaded.

## The queue's items

| item | kind, mark | the score | first act | it stops and says so when |
|---|---|---|---|---|
| 4565 the sample, units 7055, with row 43's 3299 | rules, whole; the sample's | `Parity:`, units 7055; row 43's column | the leaf on run1787's shorter walk, then run1786; no take | two landings running leave the first parting and the draw word as they are |
| 4563 fifth, row 63 | rules, whole; a parity seat's | the fifth's 3 → 4 of 8 | the one leaf on run1782, no take | two landings running move no column |
| 4562 fifth, row 83 | rules, whole; a parity seat's | row 83's column, eleven lines | the leaf on run1784, no take | two landings running move no column |
| 4559 fifth, row 103, and eight bays | rules, whole; the chain's | row 103's column; what else moves | a test that lists the eight pieces, red; then 4558's recipe on each | two landings running move no column |
| 4566 the sixth battery, to 12,000, a scripted human | rules, whole; unseated until the fifth reads 4 of 8 | none; a found column | whether the minting serves a held-out lobby; then the declaration, before any file | it is pinned |
| 4564 fifth, row 43 | rules, whole; unseated behind 4565 | row 43's column | scored again at 4565's booking | two landings running move no column |
| 4548 fifth, row 3 | rules, whole; unseated | row 3's column, eleven lines | a take, `0xffbu`, to 6000 | two landings running move no column |

## Built

- `tools/guard.sh` runs `cargo fmt --check` after the source pins and
  before its first build. `test_brief.TheReflexReadsTheQueue.
  test_the_reflex_checks_the_format_before_it_builds`, seen red on the
  script as handed over; the script seen to exit 1 in 4.6 s on one
  misformatted line.
- `census::RUBRIC_TERRITORY_PORTED` 738 → 739 and `_OPEN` 571 → 570,
  from the recorded row; the queue's `Rubric:` line with them.
- The paperwork: entry 152 and its index row, this record, the queue's
  handoff, lanes line and seven items, the loop's three clauses, the
  tracks' paragraph, the journal's entry, the suite's timings, the
  rubric's row.

## Not done, and said

- **That one mechanism parts both the sample and row 43 is not shown.**
  The two reports name the same fields; no function was read.
- **The eight pieces are numbers, not names.** Only 148 (the Royal
  Mameluke, 4159r5a) and none of the others was looked up. Whether any
  is a pivot piece that wants `pivot::RELEASES` rows is the item's.
- **The probe saw only the games on disk, on our walk.** A piece the
  original fields past our first parting and ours never does is not
  listed. Seven of the eight fire after their game's first parting.
- **No guard holds the fallback at zero.** The item lands the list as
  its red test; a standing guard over every game is not built.
- **Whether an idle human's game lasts 12,000 frames is unread.** Two
  rows of the fifth were ended by the original before 6,000, both with
  the computers allied against the human.
- **Whether a scripted human can be recorded on a held-out lobby is
  still unread**, as at the pass before, and Ramon has now asked for
  one. It is item 4566's first act.
- **The sample's rate is two figures, not a forecast.** Nothing says
  which of 158 and 34 states a landing the next ten will resemble.
- **Why a lane waits 46 minutes on its own gate is measured, not
  explained**, as before; it fell from 54.8.
- **`tools/ctx.py` still reads the driver's context from a builder**,
  and the readers of `leaders.defeated` are not swept, as before.
- **The frame took no row.** The rule of §2 and §3 is in the loop's
  ninth act, which the pass reads, and in the items; a worker's brief
  does not carry it.
- **The four files over the compressor's limit stay raw**: three text
  dumps as before, and a lab ranges record of 2.0 GiB the splitter does
  not walk.
- **The driver's own cost is not summed**, as before.
- **The blind list was not scanned**: open 0.
- **The tracks' paragraphs for the passes of 2026-11-17 and
  2026-11-17, second, were retired** to keep the section under its
  ceiling; their story is entries 139 and 140 and the journal's.

## The gate

Whole, by `tools/gate_run.sh`, once.

- **`gate-pass-1122b.log`: `gate exit 0`** on `a182ae5ad`, the pass's
  commit, 9 of 9 steps: offline 1.0, guard 0.5, merge pins 1.1, release
  21.9, clock 0.1. `Clock: green beside load`. Admission 12.7 s over
  1,551 leases; load 3.0 at its start and 7.9 at its end. `Fixture
  audit: 202284 requests, 0 missing fixtures`. `Rows: one driver row at
  most: steer 327c5727 (working)`; `Driver: 0 landings since the last
  pass (backstop 20) · context 288 k`; `Trunk: origin/main is
  origin/worktree-replan-pdb`. `Builders: 0 reaped`; 110 GB free after
  it.
- **Ramon's two answers came while that gate ran**, and the commit
  that quotes it carries them: documents only (entry 152 §4 and §5,
  item 4566, the handoff's lead, one clause of the loop, the tracks'
  paragraph, the journal, this record). It ran no whole gate; the
  reflex, `tools/guard.sh`, was green on it.
