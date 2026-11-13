# The pass of 2026-11-13, by supply: the gate binds the loop, a leaf leaves only where it moves, and step VII has no oracle yet

*2026-11-13, from 08:16 −05:00, the `steer` row on Fable 5.2. `commander` armed
the pass by supply at 01:30 with the tree banked at `9db514c56`; the arm did not
fire, and Ramon started the pass by hand (`docs/DECISIONS.md` entry 131 §8). The
charter: entry 130, Ramon's question on the clear at 230 k, parked 4187c and
4188a, the tranche's receipts, the design of the chain's step VII and items for
three lanes. Ramon was on the row. Decisions in entry 131. Two scouts and one
builder ran, each a `lean` agent on Opus 5.5; the scouts' reports are
`~/Library/Caches/attrition/pass-1113/deep-clear.md` and `vii-census.md`. The
pass before it is `2026-11-12-fable-pass-load.md`.*

## For Ramon

- **The handover is yours to pick** (entry 131 §8). Both failures are read:
  the cold wake was the resting row's own background command exiting an hour
  later; the lost arm is ccc declining to type a prompt onto a row that is
  already empty. I recommend spawning the other row at each handover and
  reaping the old one over fixing ccc's clear; a tree a row fixes neither.
- **The loop is bound by the booking gate.** A gate ran for 334.9 of the
  tranche's 413 minutes, one landing each. More workers add a queue. A gate
  now takes every landing merged when it starts.
- **Step 5's "eight of twelve lines" stands on one frame.** On the 107
  packets 147 of 1,058 lines agree and the leaders line on none, because 87
  of them sit on a start seated from a dump. The chain's next step walks
  those games from the lobby-built start, so the count is true at depth.
- **Step VII cannot be scored on anything on disk.** Its 21 leaves move in
  six dumps nobody walks. It takes one capture first, at `TEAM_STYLE` 0.
- **A hole in the release rule is closed**: a leaf could have left the
  held-out list by never being written. It must move now, or be shown
  frozen by a scan of the executable.
- **Clearing a lane at 230 k did not pay**: −3.49 USD over four lanes. The
  line is 400 k. What is dear is a read past the cache's hour.
- **`fseventsd` has held a core and 28.4 GiB for 29 days up**, swap 15.1 of
  16 GB. It tops every gate's receipt, quiet ones too. `sudo killall
  fseventsd`, or a restart, is yours.
- The tranche: eleven landings in 6.9 hours, 85.87 USD of lanes (10.73 a
  landing); the week 25 % at 04:46Z with a day left. The pass changed four
  documents and the memcap fixture, and moved no score.

## What was measured

| what | measured |
|---|---|
| the tranche (`tranche.py 4197 4192 4190 4198 4193 4195 4191 4194`, after `lore index`; `--base 2fcd8fc1d` printed no rows) | 8 lane landings over 12 sessions, 85.87 USD, 10.73 a landing; mean session 69.0 minutes clock, 18.9 working, 50.0 waiting (its gate 39.0, the suite 6.4); every session peaked between 246 k and 316 k but the two second sessions that only gated (126 k, 133 k); two takes of the original. The chain's P, VI and 4189 were in-session builders |
| the gates (`gates.py <commander's tmp> --since gate-pass-1112d.log`) | 11 logs: 7 whole, 4 scoped and widened; 3 red (gate-4192 on the memcap fixture; gate-4187 at its offline step on the doors pin, 1.8 minutes; gate-bank-1113 on `free::ages`); 334.9 minutes; admission 26,415 s summed, the longest wait 665.8 s. A whole booking gate beside the lanes 33.1 to 44.9 minutes, alone 29.1 and 30.0 |
| the gate's share of the clock | first gate's start 18:35, last gate's end 01:28: 413 minutes, a gate running for 334.9 of them (81 %), one landing a gate |
| host load at each gate's end (16 cores) | 14.7, 7.4, 8.5, 50.2, 18.7, 11.6, 8.3, 12.9, 5.0, 4.7; two past the cores (4190's end at 50.2; 4198's start at 67.7 with a capture and a build beside it) |
| `Host top:` | `fseventsd` between 98 and 176 % on every line of every gate, 100 % at a load of 3.0. Now: 28.4 GiB resident, 29 days 4 hours of process time, swap 15.1 of 16 GB used |
| the disk step (`gc-oracle` lines) | 0 stale generations removed at every gate; kept in use 130, 260, 387, 514, 644, 774, 774, 774 over 132 caches: about six sets, 9 GiB at 1.5 GiB a set. The disk 224 → 187 → 200 GB over the tranche: the fall is lanes' and builders' trees, returned at their reap |
| the clear at 230 k (scout, 1,310 requests of 12 sessions, lore's list prices) | ramp 2.51, 0.45, 3.44, 1.37 USD (4190, 4192, 4193, 4198); saving 1.17, 1.72, 0.78, 0.60; net −3.49; on a model charging only the context carried, +0.24. Mean USD a request by context: 0.049 at 100–150 k, 0.064, 0.071, 0.076, 0.083 at 300 k+. No request missed the cache; the longest gap 51.7 minutes. On the four uncleared lanes 209 requests above 230 k cost 15.08, of which 3.54 to 4.82 is the context; waiting turns 0.75 |
| step VII's leaves (scout: `rg` over 1,180 gamelogs less the battery's 18; item 4188's `leaves4.tsv`) | 21 of 21 in `LEADERS_HELD_OUT`, no writer in `crates/sim`; 15 printed, `theirs` parses none; nonzero in six dumps (run1077, run1078, run1086 and twins), none walked, none with a packet or a recording; on 226 packet leaders 20 read 0 and `hire_who` −1 |
| the window (`lore limits`) | the whole week 25 % at 04:46Z, the Fable week 13 % at 02:43Z, both closing 2026-11-14 06:00 −05:00 |
| the rubric (`rubric.py --record`) | the territory 733 · 332 · 574; simulation backed 1,456; lockstep bytes 78.1 %, ages 8 of 8, unit types 296 of 302: every figure the pass before's |
| the disk (`df -g ~`) | 186 GB free at the start, 209 after `afsctool` over both archives (1,249,010 entries) |

## What the pass looked at

The client's still from the gated binary at `9db514c56`, at the three cameras
of the pass before, each beside its photograph; the stills are under
`~/ron-data/lab-experiments/2026-11-13-pass/`. By eye, no boxes taken:

- **run1559b and run1570, the city.** The paved base is drawn and is
  smaller than theirs: theirs has a pale outer ring and a red line round it
  (4199). Theirs burns on four buildings, black smoke and flame; ours draws
  neither, and it is now the largest gap in the world (4201). Theirs has
  the city's plate and a cone of light at the general. Two of theirs's
  tanks stand in a brown haze, faded; ours are opaque and a brighter green.
  Muzzle flashes at three riflemen. Small dark marks on the grass left of
  the city in both photographs, in the same places: unread, no item.
- **The same frames, the field.** Trees and their shadows agree. On
  run1570 our mountain is darker and browner than theirs. Beyond the map's
  edge ours is black, theirs dark olive.
- **run1564, the lake.** The land's fade under water landed and the picture
  is still wrong in the ways a row should name: the `OCEAN` cells are a
  light-blue polygon with tile-stepped edges inside a darker ring; the
  beach is a sawtooth a tile wide and three times theirs's width; no
  breaker, no surface texture; the fish are white on the surface (4200).
  Theirs also shows an oil patch on the grass and a border line at the
  right edge; ours draws neither, and no item names them.
- **The interface** is not in a `--still`: the HUD of 4197 and 4198 is the
  window's, so the largest difference on every photograph is unchanged in
  what a row can read.

## Rulings

- **4187c: frozen by construction** (entry 131 §1). `score_explored` and
  `score_pop` leave when the scan pins their writers as two constant
  stores and a neighbour on each side has left by moving. `last_spoke`
  takes the same rule.
- **Entry 128 §3's agreement clause takes a mover** (§1). The leaves
  released before it on a reset value are run through the scan.
- **4188a is the chain's step S, first** (§3).
- **The clear at 230 k is withdrawn for 400** (§6), on the measurement.
  The commander's own move to 400 mid-tranche was right.
- **The disk step's twelve hours stand.** It kept six sets, about 9 GiB,
  where entry 129 §2 said a few; the disk's fall was trees, and the reap
  returned them.
- **4198's fix is ratified**: a nested gate test took the whole-lane lock
  and stalled every whole lane gate; the test no longer takes it.
- **The memcap fixture** went red twice at loads of 14 to 26 on 16 cores,
  printing nothing of the wrapper's. The host is not an explanation at
  those loads. A builder took it during the pass ("Built").
- **The fifth worker is not added** (§5): the gate binds.
- **Entry 130 §3 is withdrawn** (§8); its §1 stands.

## The chain's order, each step with its first act and its kill

| step | what | score | first act | it stops and says so when |
|---|---|---|---|---|
| **S** | each packet's game stood up from the lobby-built start where `Start:` holds its lobby, in place of the dump's seat (parked 4188a) | `Checksum: lines 147 of 1,058`; the leaves parting outside the list (`cities_built` 224, `aggression` 216, `start_x.num` 106) | one word whose lobby the start track holds, walked from the built start to its packet's frame, the twelve lines beside the seated walk's | the built start does not hold that word's lockstep: the word stays seated, is named, and the count is taken over the rest |
| **VII-a** | one take, click-free, on a lobby our walk holds at `TEAM_STYLE` 0 (run1078's is the coverage pair's): meet, ally, a resource taunt, a declaration while allied, a peace accepted, a second declaration, a short proposal; level-9 records round each line, a packet after the last; then our walk of the script | the game's word, whole to its last line; the six dumps' movers reproduced (`dow[1]` 0 → 1 the frame after the declaration) | run1078's own dump against the coverage pair's walk with its script: how far the walk holds a style-0 game before any take | the walk parts at a computer's diplomatic decision the crate lacks: that decision is the next step, read first, and VII-b waits |
| **VII-b** | the leaves that moved on VII-a's game, held and compared: `dow`, `blacken`, `broke_alliance`, `made_peace`, `tribute_stamp`, `gift_stamp`, `tribute_sent`, `tribute_received`, `last_taunt`, `taunt_frame`, the three `incoming_taunt*` it can reach and `incoming` | `LEADERS_HELD_OUT` 47 by each leaf that moves there | `theirs` parsing the fifteen printed keys, seen parting against a stub | a leaf's rule is one §8.6 leaves unread (`counteroffer`, `good_deeds`, `tribute_received`'s scale): it stays held and is named |
| the frozen scan | the executable's stores to a leaf's displacement, as an install-backed test | `LEADERS_HELD_OUT` by `score_explored`, `score_pop`, `last_spoke`; the earlier quiet releases checked | the scan on `score_buildings`, which has live writers, seen to find them | a displacement under `+0x200`, which cannot be told from another record's (`docs/LEADERS.md`, the shared facts) |
| after | the residue of III, IV and VI: parked 4176a–c, 4177a–c, 4187a, 4187b; then VII-c with a reading and a game of three leaders | the 26 leaves outside §7 to §9 | the commander's, as each lands | |

## The lanes' items, and what each can fail on

| item | kind, gate | the score | how it fails today |
|---|---|---|---|
| 4199 the asphalt ring | render, whole | the ring's tiles on two stills | the client has no roads |
| 4196 Southwest Mesa, British Isles | start, whole | `Start: 143 of 161` | 18 lobbies part |
| 4204 the quit's `clear_orders` | rules, whole | run667's 45 unit rows and 4 Mill rows, run609's 4, `cant_reach` 5 → 0 | carried at the frame's head it moved 54 closing states the wrong way |
| 4200 the lake's water | render, scoped | `RUN1564_3279_GROUND_STILL` 1,024 of 1,920; luma 1,318 | 896 tiles part; `OCEAN` (6, 155, 236) against (47, 113, 158) |
| 4201 fire and smoke | render, scoped | emitter anchors, ours of theirs within a tile on two photographs | none is drawn |
| 4202 a press that follows or guards | input, scoped | `INPUT_FOLLOWED` n of n; the fog arm's press | an attack is minted for both; no `is_seen` test |
| 4203 the HUD's clock, a wounded selection, the ring | render's brief, scoped | `Hud:` the clock on three photographs; the panel's `damage` on a wounded capture | the bar prints the frame; `damage` parts on no frame |

Order: the builder on S, VII-a, VII-b. The whole slot: 4199, 4196, 4204. The
scoped render lane: 4200, 4201. The input lane: 4202, 4203.

## What the pass ran, to check a ruling before writing it

- ccc's `performClear` (`Sources/ccc/PaneController.swift`, line 683 of the
  installed 0.1.40 build 262's source): the guard that returns `typed /clear
  on <ref>; the screen never changed` before the prompt is typed. `ccc clear
  steer --status` printed that line for the arm of 06:29Z.
- The resting row's transcript at 2026-11-13T00:00:16Z: a
  `task-notification` for its own background command, "Serve a headless ccc
  pane for the commander row", then a request with 31,008 tokens read from
  the cache and 312,347 written.
- `ps` and `sysctl` on the host for `fseventsd`.
- Not run: any take, any packet walk, the scan. The census's counts are the
  scout's, measured or cited as its report marks them.

## Built

BUILT_PLACEHOLDER

## Not done, and said

- **No handover mechanism is built**: Ramon's pick (entry 131 §8).
- **No second reading** was run; no marked row stood for this pass.
- **Minting a `Declare` or a `Chat` through `Command`** is not booked. It is
  the milestone's own carrier for diplomacy, and VII-a's script does not
  need it; it follows VII-b.
- **The dark marks on the grass, the oil patch, the border line, the tanks'
  haze, the plate and the cone** have no item: none has a row ruled.
- **The lane's own gate** is 39.0 of a session's 50.0 waiting minutes, and
  the pass changed only the booking gate's rule.
- **`tranche.py --base`** printed no rows for this tranche's `Book item N`
  commits; the table was made by item. Not fixed.

## The gate

GATE_PLACEHOLDER
