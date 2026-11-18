# The pass of 2026-11-18: Codex's night, parity's set, and a cheaper game for the demand

*2026-11-18, from 10:46 −05:00, the `steer` row on Fable 5.2, called by
the backstop. `commander` (Opus 5.5) handed over at `cce4d958b` after
twenty first-parent merges (nineteen lanes' and Codex's branch), drained
and gated whole (`gate-drain-1118.log`). The charter was the handoff's:
rule on Codex's `codex/live-game-recording`, on 4379's named fall, and on
the four designed items. Decisions in entry 142. The pass spawned no
subagent and took no capture. Ramon was on the row and asked whether
Codex's night was merged, whether work was done twice, and what to
improve. The pass before it is `2026-11-17-fable-pass-empty-lines.md`.*

## For Ramon

- **Codex's night is merged, and it did well.** `commander` merged it at
  07:36, ninety minutes after waking, and the drain's whole gate is green
  on it. Our client now records a live game, the original plays that file
  back, and all twelve state checksums agree on every frame of four such
  games, two of them with fighting in them. I added those four to parity's
  set: **parity reads 5 of 6 games.** The sixth is the 2017 sample.
- **One thing was built twice, and one morning was untidy.** Codex and a
  lane each wrote a routine that walks a recording against the original's
  checksums. No game mechanic was built twice. The untidiness: the driver
  woke with Codex's branch sitting 23 commits ahead, spawned lanes first,
  reused three of Codex's item numbers and merged later with a conflict.
  `brief.py` now refuses to brief a lane while a Codex branch is ahead.
- **The last leaf of step 5 has cost 89.72 USD over ten lanes and is still
  404 frames short.** The last four lanes bought 55 frames. I changed its
  next act: search our own sim for a small game where the computer demands
  tribute early, have the original play that game, and compare the
  checksums. If the search finds none, the old walk resumes.
- **The 2017 sample fails parity on its very first frame** on four lines.
  The suite's lane had been chasing a scout at frame 2,892 instead. Its
  next act is the first frame.
- **The border colour is better, and its score fell because the score was
  measuring the wrong thing.** I looked at the pair; the new band is the
  right lavender. The next item fixes the measure.
- **The week:** 61 % at 10:46 against today's line of 70. The tranche took
  nine points for twenty landings; the lanes cost 131.52 USD at list, 6.92
  each. Fable's week reads 33 %.
- **Blocked on you:** the picture upload was refused by the permission
  classifier. The three pictures are in
  `~/ron-data/lab-experiments/2026-11-18-pass/upload/`; the command is
  `share ~/ron-data/lab-experiments/2026-11-18-pass/upload attrition/pass-1118/`.
- **Yours, carried:** ten minutes playing the client; the merged branches
  on origin; the README's line 12; Codex's worktrees hold 17 GB.

## What was measured

| what | measured |
|---|---|
| the merges since the pass before (`git log --first-parent --merges b0ce6b16d..cce4d958b`) | 20: lanes 4353, 4355, 4352, 4356, 4351, 4354, 4357 (2026-11-17, 17:26 to 18:55), 4361, 4360, 4358, Codex's branch (07:36), 4374, 4375, 4376 (worktrees `att-4362` to `att-4364`), 4359, 4378, 4377, 4381, 4380, 4379 |
| what they moved | 14 lanes moved a score; 5 moved none (4353, 4356, 4360, 4374, 4380); Codex's twelve items moved the packet lines 499 → 608 of 1,053 and minted four whole games |
| the lanes (`tools/tranche.py --base b0ce6b16d`, after `lore index`; the three renumbered ones from `lore loop commander`) | 19 sessions, 131.52 USD (111.82 over sixteen, and 8.07, 5.35, 6.28), 6.92 a landing; of the sixteen a session's clock was 52.9 minutes, 17.7 working and 35.1 waiting, 25.8 of them on its own gate; one past 300 k (4359 at 342 k, 172.7 minutes, 90.6 at its gate); 10 captures |
| the driver (`lore loop commander --since` the last pass) | the morning's row 8.90 USD, twelve lanes spawned; the evening's row was not re-read |
| the demand's leaf, by item | to 4350: 53.53 USD, 5622; then 4354 (9.78, 5622 → 5668), 4360 (9.17, none), 4374 (8.07, none), 4377 (9.17, 5668 → 5677): 89.72 USD over ten lanes, 404 frames short of 6,081 |
| the gates (`tools/gates.py ~/Library/Caches/attrition/gates --since gate-pass-1117d.log`) | 13, 310.1 minutes: 3 whole, 10 scoped (9 widened), 0 red; admission 12,849.8 s summed, longest 763.2 s; the whole gate on Codex's merge (`gate-1118c`) 52.8 minutes beside load |
| the drain's gate (`gate-drain-1118.log`, read for every line that is not a passing test) | `gate exit 0` on `cce4d958b`, 9 of 9 steps, 30.0 minutes, release 25.4; 7,729 passing lines; the `FAILED` lines are the gate's own fixtures (`test_release_gate`, `test_isolated_tests`); 0 `skipping:` lines; Codex's eight `checksum::playback` tests ran; `Driver: 20 landings since the last pass (backstop 20)`, context 257 k |
| Codex's branch (`git log 405a7658f..26264ca35`, its twelve journals read whole) | 23 commits, 20:22 to 04:27, 79 files, +3,656 −693; seven whole gates of its own green, four red and mended on the branch; evidence under seven `~/ron-data/lab-experiments/2026-11-1[78]-codex-*` directories, 3.3 GB |
| the demand on Codex's games (a scratch test on `recording_start`, reverted) | `tribute_demanded` is 0 on every state of the long window (9,476), window A (11,721) and window B (14,438) |
| Codex's scorer on the 2017 sample (`score_recording`) | refused: "missing the recording's closing package"; the two walkers were not cross-run |
| the week (`lore limits`, 10:46) | 61 % (the day's line 70, the fifth day begun), from 52 at the pass before; 26.37 USD a point all-in; Fable's week 33 % |
| the stills (`RON_STILL_DIR`; `the_ground_on_the_client_s_still`, `the_settled_border_band_on_run1665`, `the_border_band_on_run1564`) | run1564, run1559b and run1665's four arms at their photographs' cameras; two half-size pairs and the band's crop under `~/ron-data/lab-experiments/2026-11-18-pass/` |
| the suite re-timed (`suite_timings.py --stale-only`) | 27 tests timed: 2,521 tests, 5,391 s one at a time; the slowest new one `diff::sample::tests::the_sample_s_lines_against_the_hook`, 148.69 s, 631 MiB |
| the disk (`df -g ~`) | 158 GB at the start, under the last pass's 215; 171 after `afsctool` over both archives; 178 after `cargo clean --profile dev` (10.9 GiB) |
| the rubric (`rubric.py --record`) | every figure the pass before's: 736 · 331 · 572, 44 of 112 groups whole, 1,456 of 3,611 backed, ages 8 of 8, unit types 296 of 302, lockstep bytes 78.1 % |
| the rows and trees left | no lane row (`ccc list`); `reap_agents.py --yes` printed nothing; the merged, clean `att-4356` tree removed; the old driver row reaped by `handover.py reap 45a273b0` |

## Rulings

- **Codex's branch stands as merged; no second reading is owed; parity's
  set is six games and five agree** (entry 142 §1).
- **A driver settles Codex's branch before it briefs a lane** (§2), with
  the guard.
- **The demand is tried on a game the walk holds** (§3).
- **The sample is read by its lines, from the start** (§4).
- **4379's fall stands as named; a row's footprint is never a colour; the
  queue's `RUN1564_3279_BORDER` is corrected 48 → 1** (§5).
- **The four designed items are ratified**, three with new first acts
  (§6).
- **Marked rows:** no `FABLE:` marker and no second reading in the
  tranche; nothing to ratify.

## What the pass looked at

run1665's photograph over the client's still at the tip
(`pair-1665-half.png`), the band at 4:1 (`crop-1665-band-4x.png`: theirs,
the LDR blend, the HDR blend), and run1564's pair (`pair-1564-half.png`).

- **The band** is a strip of about 130 by 80 pixels at the frame's east
  edge. The HDR blend carries theirs' lavender; the LDR one is greyer and
  darker. Theirs has a soft owner edge and a faint second line under it;
  ours steps by the pixel and has no second line.
- **On run1564** theirs draws the morph's curved arc at the east edge and
  ours a short straight strip, as item 4356 found.
- **What no row counts, on both pairs:** the lake. Theirs is dark and
  mottled with white streaks across its whole body; ours is pale and
  smooth, with heavy white crescents along the west and east rims that
  theirs does not have, and a bird theirs does not have. The land, the
  sand, the blot and the cracked patch read alike. The seat left the lake
  by Ramon's word (entry 140 §6); this is what it left.
- run1559b's still was drawn and not paired; nothing landed on it.

## The queue's items

| item | kind, mark | the score | first act | it stops and says so when |
|---|---|---|---|---|
| 4382 the demand on a game our walk holds | rules, whole; the chain's | `LEADERS_HELD_OUT` 1 → 0 | a sweep in our own sim for a lobby whose computer demands early; then that game minted, played by the original and scored | no lobby demands in ours (then the walk from 5677, two acts that move nothing) |
| 4384 the sample's lines at the start | rules, whole; the suite's | `SAMPLE_LINES`' first partings: builds 0, leaders 0, world 0, goods 1, units 92 | a ranges take of the playback's states 0 to 100, each line's first differing range named | two acts move no line's first parting and the word by nothing |
| 4383 the Repair pick's `REPAIR` played | input, scoped; the third seat | the arm's effect equal on both sides | an attack package that wounds a building of slot 0's, then the pick | the playback is refused |
| 4385 a border row that reads by value | render, scoped | `RUN1665_3279_BORDER` 138 of 262, `RUN1564_3279_BORDER` 1 of 1,440, both re-minted | the footprint from our strip's geometry, a pixel counted within a stated distance of theirs, seen red on a perturbed side | two acts move the row by nothing |

## Built

- `tools/handover.py`: `astra_ahead` and `astra_line`; `tools/brief.py`
  refuses a brief while a `codex/*` branch is ahead of HEAD. Red first:
  the two new tests in `tools/explore/test_handover.py` failed on the
  missing functions and pass with them; the 66 tests of the two files
  pass; on this tree `astra_ahead()` reads `[]`.
- The paperwork: entry 142, this record, the queue's handoff and four
  items, the tracks' paragraph, the journal's entry, the suite's timings,
  the rubric's row, the driver's clause in `docs/LOOP.md`, and
  `docs/CHECKSUM.md` §10's list of what rests on one reading.

## Not done, and said

- **The pictures were not uploaded.** The classifier refused `share`; the
  command is under "For Ramon".
- **The two walkers were not merged or cross-run.** Codex's refuses a
  recording with no closing package, which the 2017 sample is. One walker
  over both kinds of file is a small structure item nobody has booked.
- **Parity's count is prose.** No constant pins "5 of 6"; each game has
  its own test, and the queue's sentence is not parsed by a guard.
- **The sweep of item 4382 was not run.** The pass showed only that the
  leaf does not move on the three games on disk.
- **The evening driver's cost was not re-read**, so the tranche's all-in
  figure is the lanes' and the morning's driver only.
- **The crops of the pass before were not redrawn**: their boxes were not
  recorded. This pass's crop box is x 1790 to 1920, y 450 to 530.
- **The blind list was not scanned**: open 0.
- **Carried:** a red clock beside load is unrun; `tools/tranche.py`
  tables no in-session builder and no renumbered lane.

## The gate

Whole, by `tools/gate_run.sh`; quoted below once it has run.
