# The pass of 2026-11-17, second: parity's three empty lines, the sample under the hook, and a pin every lane gate runs

*2026-11-17, from 15:57 −05:00, the `steer` row on Fable 5.2, called by
the backstop. `commander` (Opus 5.5) handed over at `88e7c2451` after
nineteen landings on twenty first-parent merges, 10:09 to 15:26, drained
and gated whole (`gate-drain-1117.log`). The charter was the handoff's:
score the nineteen, rule whether the minted game is parity's 1 of 1,
ratify the four designed items and refill the seats. Decisions in entry
140. The pass spawned no subagent and took no capture. Ramon came onto the
row during it. The pass before it is `2026-11-17-fable-pass-eye.md`.*

## For Ramon

- **Parity's first game is one check away, and I did not call it.** Nine
  of the twelve checksums are compared on the minted game and agree on all
  24,000 frames. The other three (walls, ammo, deaths) are not compared on
  our side. I read the original's own per-frame trace: all three are the
  empty value on every frame, so that game has no wall, no shot and no
  death in it. Item 4355 counts ours on every frame. If it reads zero
  throughout, parity is 1 of 1, on one game with no combat in it.
- **The milestone's step 5 still waits on one leaf**, `tribute_demanded`.
  Six lanes and 53.53 USD have gone into the walk that reaches it; it
  stands at frame 5622 of 6,081, stuck on how an army musters. The next
  act is a snapshot of the original at 5621. Its stop rule stands: two
  acts that move nothing, and it comes back to a pass.
- **The week:** 52 % at 15:57 against today's line of 56. This tranche
  took eight points for nineteen landings, 0.42 of a point each, and
  153.14 USD at list. Fable's week reads 25 %.
- **What you decided on the row** (entry 140 §6): parity waits on 4355;
  Astra comes in when `commander` pauses at the day's line and is settled
  by a fast-forward and a whole gate; the render seat finishes the foam
  strokes and then leaves the lake for what shows in a played game, the
  borders first.
- **The pictures:** `https://cdn.ramonfabrega.com/attrition/pass-1117/`,
  eight of them, the lake drawn at this tip and the cities and the lobby
  from this morning. Every pass uploads its own from now on, under the
  same names, so two folders are a before and after. The folder is public
  to anyone with the link and expires in about thirty days.
- **Yours, carried:** ten minutes playing the client; the merged branches
  on origin; the README's line 12.

## What was measured

| what | measured |
|---|---|
| the merges since the pass before (`git log --first-parent --merges f7d7960b9..88e7c2451`) | 20, all lanes': 4334, 4335, 4337, 4331, 4338, 4333, 4339, 4342, 4344, 4340, 4343, 4343 again (its mend), 4336, 4345, 4346, 4341, 4347, 4349, 4348, 4350; nineteen landings |
| what they moved | 15 moved a score, 4 moved none (4335, 4331, 4342, 4350) |
| by seat, lanes at list | the suite's (4333, 4340, 4341, 4350) 38.20 USD, `LEADERS_AGREE` 23,965 → 24,000 of 24,000 and `RUN1646_WORD` 5496 → 5622; the chain's (4331, 4336, 4347) 28.34, the sample's start 17,163 → 18,315 of 18,315 and `SAMPLE_WALKED` 0 → 444 of 21,044; the third (4334, 4337, 4339, 4343, 4346, 4349) 36.44, the HUD's panel 103 of 103, its pages, the standard panel 398 of 398 and the picks 24 of 24; render (4335, 4338, 4342, 4344, 4345, 4348) 36.49, `GROUND_STILL` 1,798 → 1,855 of 1,921 and the fish 0 → 302 of 304 |
| the demand's leaf, by item | 4316, 4322, 4325 (23.73 USD, 2082 → 5496), 4331 (10.83, none), 4341 (6.44, 5496 → 5622), 4350 (12.53, none): 53.53 USD, 459 frames short |
| the lanes (`tools/tranche.py --base f7d7960b9`, after `lore index`) | 19 sessions, 139.47 USD, 7.34 a landing; a session's clock 56.2 minutes, 15.9 working and 40.3 waiting, 27.9 of them on its own gate; two past 300 k (4331 at 320 k, 4350 at 345 k); 5 captures |
| the driver (`lore loop commander --since` the last pass's tip) | 13.67 USD on one row (`c63ebe67`), twenty lanes spawned, no in-session builder; lanes and driver 153.14 USD |
| the gates (`tools/gates.py ~/Library/Caches/attrition/gates --since gate-pass-1117b.log`) | 14, 255.2 minutes: 1 whole, 13 scoped (11 widened), 2 red (`gate-1117o`, `gate-1117p`); admission 6,552.9 s summed, longest 422.3 s; 2 ended past the cores |
| the drain's gate (`gate-drain-1117.log`) | `gate exit 0` on `88e7c2451`, 9 of 9 steps, 27.8 minutes, release 23.2; `Clock: green beside load`; `Driver: 20 landings since the last pass (backstop 20)`, context 327 k; HEAD, `origin/worktree-replan-pdb` and `origin/main` all `88e7c2451` |
| the week (`lore limits`, 15:57) | 52 % (the day's line 56, the fourth day begun), from 44 at the pass before; 26.99 USD a point all-in; Fable's week 25 % |
| the hook's trace of the minted game (`rontrace-run1578.log`, the first set of each frame, read by a scratch script on `rondata::trace::checksums`' layout) | 24,004 sets on 24,001 frames, −1 to 23,999; walls, ammo and deaths each read 1 on all 24,001; units, builds, guys and leaders take 24,001 distinct values, world 23,496, groups 4,006, cities 78, items 41, goods 9; rules 1 |
| the stills (`RON_STILL_DIR`, `the_ground_on_the_client_s_still`) | run1564 and run1559b at their photographs' cameras, the pair and three crops of run1564, under `~/ron-data/lab-experiments/2026-11-17-pass-2/` |
| the suite re-timed (`suite_timings.py --stale-only`) | 6 untimed tests timed (the sample's walk 14.43 s, 1,299 MiB; the rest under a second): 2,494 tests, 5,042 s one at a time |
| the disk (`df -g ~`) | 192 GB at the start, under the last pass's 207, so `cargo clean --profile dev` (13.1 GiB): 200; 202 after `afsctool` over both archives |
| the rubric (`rubric.py --record`) | every figure the pass before's: 736 · 331 · 572, 44 of 112 groups whole, 1,456 of 3,611 backed, ages 8 of 8, unit types 296 of 302, lockstep bytes 78.1 % |
| the rows and trees left | no lane row (`ccc list`); `reap_agents.py --yes` printed nothing; the old driver row reaped by `handover.py reap c63ebe67` |

## Rulings

- **The minted game is not yet parity's 1 of 1** (entry 140 §1): three
  lines are uncompared, theirs are empty on every frame, and item 4355
  counts ours.
- **The four designed items are ratified** (§2): 4354 and 4352 as
  designed, 4351 with the hook's take as its second act, 4353 on the
  strokes alone.
- **The render seat leaves the lake after the strokes, and a pass uploads
  its pairs** (§6, with Ramon).
- **The packets' wiring pin runs in every lane gate** (§3).
- **The render seat's five lines stay in their journals** (§4): the
  frame's render section is full.
- **Marked rows:** no `FABLE:` marker and no second reading in the
  tranche; nothing to ratify.

## What the pass looked at

run1564's photograph over the client's still at the tip
(`pair-1564-half.png`), and three crops, theirs left and ours right.

- **The south-east water** (`crop-1564-southeast.png`, 1:1). Theirs has
  bold, opaque white strokes far out from the shore. Ours has fine lace
  near the shore and none far out. It is the largest difference on the
  frame.
- **The west rim** (`crop-1564-west.png`, 1:1). Ours draws thick white
  crescents along the rim. Theirs, where the chat's text lets it show, has
  thin lines. Ours also draws a bird over the lake that the photograph
  does not have.
- **The fish and the blot** (`crop-1564-herd-2x.png`, 2:1). The fish are
  dark, under the water and where theirs are. Theirs have a faint splash
  beside one group. The blot is flat dark blue in ours and mottled in
  theirs, and the lake's body is paler and flatter in ours.
- run1559b's still was drawn and not paired; nothing landed on it in the
  tranche.

## The queue's items

| item | kind, mark | the score | first act | it stops and says so when |
|---|---|---|---|---|
| 4354 the demand's walk from 5622 | rules, whole; the chain's | `LEADERS_HELD_OUT` 1 → 0; `RUN1646_WORD` 5622 → 6,081 | a packet at 5621: army 2/1's state and the guards' head orders | two acts move the word by nothing |
| 4355 parity's three unemitted lines | rules, scoped; the suite's, first | parity 0 → 1 of 1 games; a row minted at 0, the frames of 24,000 on which ours holds no wall, shot or death record | the count on our walk, seen red on a walk that holds a shot | ours holds one where theirs holds none (a parting, named by its frame) |
| 4351 the sample's walk from 444 | rules, whole; the suite's, after 4355 | `SAMPLE_WALKED` 444 of 21,044; then the sample's row by state | the scout's order on 92; then the playback under the hook | two acts move the word by nothing; the tracer is refused |
| 4352 the picks played by the original | input, scoped; the third seat | a pick's file played, each effect equal on both sides | a file minted by the picks on the click-free lane | the playback is refused |
| 4353 the broad strokes, the lake's last | render, scoped | the band's q99, 153 against 200 | `card_pass.py` at a stroke's pixel | no draw on the card pass puts the stroke there |

## Built

- `tools/release_gate.py`: `every_packet_is_wired_to_its_game_or_named`
  joins `LANE_PINS`. Red first:
  `test_a_lane_gate_keeps_the_packets_wiring_pin` failed on the lane's
  zero budget skipping the pin, and passes with it kept; the suite's 102
  tests pass.
- The paperwork: entry 140, this record, the queue's handoff and five
  items, the tracks' paragraph, the journal's entry, the suite's timings,
  the rubric's row, and the upload clause in `docs/LOOP.md`.

## Not done, and said

- **Nothing was built for item 4355.** The pass read theirs and booked
  ours: which of the sim's lists are the sets the three walkers walk is
  the item's reading.
- **No guard against an unkeyed listing of the archives** (`gate-1117o`'s
  cause). The mend is in place; a source scan was not written.
- **No row was added to the brief's frame** for the render seat's five
  lines. A clause for the expose white and the cover split measured 2,416
  characters against the section's 2,400 with another citation shortened,
  and a test holds that citation.
- **The blind list was not scanned again**: open 10, the input track's
  issuers.
- **run1559b and run1570 were not looked at again.**
- **Carried:** a red clock beside load is unrun; whether a builder's
  handback survives its driver's clear; `tools/tranche.py` tables no
  in-session builder; a lane's wait on `ScheduleWakeup` (4298b) is not
  ruled on.

## The gate

Whole, by `tools/gate_run.sh`, on `b0ce6b16d`.

- **`gate-pass-1117c.log`: `gate exit 0`**, 9 of 9 steps: offline 2.0,
  guard 0.2, merge pins 1.4, release 22.7, clock 0.1. `Clock: green
  beside load`. `Suite: 1365 processes for 2494 tests: 145 groups hold
  1274, 1220 ran alone; 9814 test CPU-seconds`; admission 8.5 s over
  1,402 leases; load 2.9 at its start and 11.7 at its end. `Rows: one
  driver row at most: steer 5a7589e8 (working)`; `Driver: 0 landings
  since the last pass (backstop 20) · context 271 k`. `Builders: 0
  reaped`; 215 GB free after it.

The guard went red once on the pass's own paperwork before that commit,
as built: the queue's handoff stood at 16,199 bytes against its 16,000
(`a_section_over_the_ceiling_may_not_grow`), and was cut. What Ramon
decided on the row, this section and the upload clause came after that
gate, and are gated scoped over their commit (`gate-pass-1117d.log`).
