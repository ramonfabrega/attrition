# The second pass of 2026-11-13, by supply: the draws are blind to state, the gate is cut by rule, and the chain is ordered

*2026-11-13, from 13:33 −05:00, the `steer` row on Fable 5.2. `commander` handed
over at `331d85b01` with one designed item on the queue and the chain's next
step unordered (`docs/DECISIONS.md` entry 126 §4). The charter: the staged
accept of item 4207, supply for three lanes, a capture filed mid-gate, a
literal the merge pins missed; then, by a message from the `lore` row it said
was at Ramon's request, the gate's cost, what parity means and a function
census. Ramon was on the row from about 14:00. Decisions in entry 132. Five
scouts and two builders ran, each a `lean` agent on Opus 5.5; their reports are
under `~/Library/Caches/attrition/pass-1113b/`. The pass before it is
`2026-11-13-fable-pass-supply.md`.*

## For Ramon

- **The words are whole on draws, and draws are blind to state.** Two recorded
  files three units apart in one Scout's target agree on all 24,001
  random-number words, and our walk under the wrong file holds 24,000 frames.
  On 41 packets in draw lockstep the units, builds and guys lines agree on 0.
- **Parity as zero checksum mismatches over a declared set reads 0 games
  today**, and nothing on disk can score it: the original computes its
  fifteen checksums every turn and prints them only in a dump window. Step H
  (item 4214) hooks them every frame. **Recommended: adopt that definition
  for the rules' finish line once H prints its first number**; the draw
  floors stay as the gate.
- **The gate is cut by rule, not made instant.** A lane's gate runs its
  filters alone (about 6 minutes by the scout's estimate, from 11 to 12.6)
  and a booking gate is scoped (20.0 minutes by the median, from 33.0).
  Reusing verdicts would hit 7 of 60 landings. The suite's own form, each
  game replayed once and tests as slices, cuts at most 24 %. **Recommended:
  measure one tranche under the new rules before spending on it.**
- **The XML and the decompiler rules: no change recommended.** The install's
  XML is the input today. `lore` read 561 modules flagged `ltcg`; its own
  reading, untested, is that byte-matching decompilation is impractical.
- **A census of all 48,233 functions exists** (`tools/census.py --layers`); a
  one-state table is 60 lines and moves no score. The AI layer's 1,661
  functions have no state at all.
- **A fifth worker**: after one tranche's numbers. `fseventsd` is still at a
  core and 27.9 GiB, 29 days up, swap 15.1 of 16 GB.
- The tranche: ten landings in 3.9 hours on six booking gates, six lanes at
  64.59 USD (10.77 a landing). The pass moved no score.

## What was measured

| what | measured |
|---|---|
| the tranche's lanes (`tranche.py 4196 4199 4200 4201 4202 4203`, after `lore index`) | 6 lane landings, 64.59 USD, 10.77 a landing; mean session 70.0 minutes clock, 22.8 working, 47.3 waiting (its gate 32.1, other 4.5, capture 4.0, message 3.6, suite 3.1); peaks 279 k to 391 k; one capture. 4199 waited 70.2 minutes on its gate, 4202 41.2, 4203 36.4. The chain's S, VII-a and VII-a′ were in-session builders |
| the booking gates (`gates.py ~/Library/Caches/attrition/gates --since gate-pass-1113b.log`) | 6 gates for ten landings: 3 whole, 3 scoped and widened, 2 red, 170.8 minutes; admission 9,579 s summed, the longest wait 431.4 s; two ended past the cores. gate-4205 red on `the_checksum_line_groups_its_bytes` (23.6 minutes), gate-4205b red on the two capture tests (24.3). The tranche before: eleven gates for eleven landings, 334.9 minutes |
| every gate log on disk (scout; 43 logs, 2026-11-11 22:26 to 11-13 13:31) | 25 whole, 18 scoped (9 widened), 7 red, 945.2 gate-minutes. Release step by mode: whole median 27.9 (the gate 33.0), scoped widened 15.5 (20.0), scoped not widened 6.5 (8.6); the other six steps 2.5 to 4 minutes together |
| the reds (scout: each red log read, the test's source read) | 9 red verdicts: 3 tools tests at the offline step, 2 the client's clock test under load, 1 the literal, 2 the capture pair, 1 log with no receipt. **A timed test a scoped gate would have skipped: 0.** The whole gate's cache verifiers: 0. Thirty-nine hours of history |
| what landings touch (scout: the last 60 bookings' merges) | 30 `crates/sim`, 17 `rondata` code, 1 `rondata` pins alone, 11 client or render alone, 1 documents and tools. 50 of 60 widen a scoped gate, 8 of the last 8. **The test binaries unchanged: 7 of 60, 0 of the last 8** |
| the suite inside a whole gate (gate-4207's own per-test file) | `rondata`: 2,350 tests, 1,653 s of wall, 8,977 test-seconds, 11,408 CPU-seconds. Four tests over a minute hold 918 s: `diff::coverage`'s field census 406, `free::ages` 335, the Great Lakes widening 110, every lobby's start 67. 1,436 tests each replay one game; 894 distinct games; **156 games are replayed from frame 0 by two or more tests**, 6,579 s where each game's longest test alone is 4,442 (24 %) |
| the suite alone (`suite_timings.py --stale-only`, this pass) | 189 tests untimed since 2026-11-09, timed and merged: 2,360 tests, 5,465 s one at a time on a quiet host, 925 under a second. Inside the gate the same tests summed 8,977 s |
| one test, quiet (scout: `check.sh` with `/usr/bin/time -l` as cargo's runner) | a paperwork pin 0.03 s; the harness lobby's start 0.72 s, 15.6 G instructions, 439 MiB; a 9,632-frame word 1.76 s, 31.0 G, 484 MiB (8.20 s inside gate-4207, 4.7 times); a 24,000-frame word 3.46 s, 74.9 G, 1.41 GiB; a packet test 8.02 s, 165 G. The word's split: standing up 32 %, ticking 40 %, comparing 25 % |
| a lane's own gate (scout: the report directories of 4200, 4201, 4203) | release 11.0 to 12.6 minutes, all of it the `rondata` library's wall (618 to 701 s): the filters 61 tests and 645 to 740 test-seconds, the paperwork 11 to 18, and about 920 tests listed under a second that ran at 1.6 to 2.0 s each, 1,258 to 1,690 test-seconds. The build 1.8 to 7 s. `free::ages` 313 to 371 s bounds it. Lanes 4199 and 4202 gated whole and sat 21.1 and 18.0 minutes on the whole-lane lock |
| the checksum oracle (scout; the call sites checked by the pass in the export) | `check_all` is called from `process_turn` on every turn, twice in playback; printed only in a dump window; no recording carries it; playback's own desync check compares recorded slots with each other. Our sim plays one recording of the original's (run7); the order pump maps 10 of 82 command variants; the client writes a recording only inside tests. A 24,000-frame playback takes about 28.5 minutes on the lane |
| the diplomacy ports (scout: `wc -l` over the export, run1575's record) | the declaration 317 lines for 3 leaves; the taunt's answer 674 for 4, and 5 more on the packet; the offers arm's accept 750 more for 1; found short 250 more for 1. Every mover of the first two is on a block at or before 1002 |
| the census (scout: `census.py --layers`, `--territory`, a cross-tab) | simulation 3,611, AI 1,661, engine 9,375, interface 3,303, no line record 30,283. Assignable by script: ported and swept 625, ported 108, out of scope 43,454, cited outside the simulation layer 653, unexplored 3,393. 569 of the 574 open simulation functions carry a reader's ruling of 2026-11-06 |
| the window (`lore limits`) | the week 33 % at 18:01Z and 35 % at 18:57Z, Fable's 21 % and 23 %; both close 2026-11-14 06:00 −05:00; the stop 95 % |
| the disk (`df -g ~`) | 224 GB at the start, 228 after `afsctool` over both archives |

## What the pass looked at

The client's still from the tree at `331d85b01` at the three cameras of the
pass before, beside its photograph; under
`~/ron-data/lab-experiments/2026-11-13-pass-b/`. By eye, no boxes taken.

- **The field, on all three.** Ours is orange-brown in broad patches where
  theirs is yellow-green grass with faint paths. `RUN1559B_3279_GROUND_STILL`
  reads 1,104 of 1,109 on that picture. A row that reads whole on a picture
  the eye says is off (item 4212).
- **run1559b and run1570, the city.** The paved base and its ring are drawn.
  Theirs burns with black smoke on four buildings; at this size nothing of
  ours reads as fire. The plate, the cone of light and the tanks' haze are
  as the pass before found them.
- **run1564, the lake.** Still the worst picture: a light-blue tile-stepped
  polygon inside a darker ring, a wide orange beach; theirs is textured water
  with whitecaps and a narrow pale beach (item 4213).
- Against the morning's stills 17,667, 295,242 and 18,711 pixels changed of
  2,073,600.

## Rulings

- **The staged accept** (entry 132 §1): run1575 is 1150 staged and 1099
  computed, held unstaged through block 1026. A human's command read off the
  record is an input; a computer's answer read off it is not a port.
- **The chain's order** (§2), below.
- **A tree's corpus is its ledger's** (§3). The rule is a guard, built.
- **`diff::floors` is a merge pin** (§4).
- **Booking gates are scoped; the whole gate at a drain and at a pass** (§5).
- **A lane's gate runs what its brief names; no lane gates whole but a
  `structure` lane** (§6). The fourth parked item of 4198, the memcap guard
  under load, needs no ruling: no red on disk names it since the fixture's fix.
- **No verdict cache** (§8).
- **4199b's ratio waits on 4212**: 0.89 on all three channels is what one
  transform of the photograph looks like, and 4212's first act measures it.
- **The census is not booked.** It moves no number; the tool exists.

## The chain's order, each step with its first act and its kill

| step | what | score | first act | it stops and says so when |
|---|---|---|---|---|
| **VII-b1** | the declaration: `action_declare`, `pay_dow`, `afford_dow`; the carrier is the record's `process_declare` line, which the take already parses | `LEADERS_HELD_OUT` 47 by `dow`, `blacken`, `broke_alliance`; the wealth parting of 640 on block 987 | `theirs` parsing the printed keys, each seen parting against a stub | `pay_dow`'s price needs a craft's cost the crate does not load: the cost is read first |
| **VII-b2** | the taunt's answer: `process_taunt`'s goods arm, `action_respond`'s commit and exchange, the tribute scale; run1575's packet at 1090 wired | 47 by `gift_stamp`, `tribute_stamp`, `last_taunt`, `taunt_frame` on block 967, and by `incoming`, `incoming_taunt`, `incoming_taunt_frame`, `tribute_sent`, `tribute_received` on the packet; the food of the gift | the script's 905 line minted as the taunt; the two tribute constants loaded from the rules | the walk parts once `is_tribute_period` is live in the army's targeting: the parting is the step's first number, named |
| **the frozen scan** | a scanner of the executable's stores by displacement, as an install-backed test | 47 by `score_explored`, `score_pop`, `last_spoke`; the earlier quiet releases checked | the scan on `dow`, which has live writers, seen to find them | a displacement under `+0x200`, which cannot be told from another record's |
| **VII-c** | the offers arm's accept and `action_respond`'s treaty arm | 47 by `made_peace`; run1575 **1150 computed**, the staging retired (parked 4207a) | the accept of block 1027 decided by the arm with the record read removed, seen red with the arm's answer forced to refuse | the shortfall formula `docs/LEADERS.md` §8.6 leaves unread decides it: read first |
| **VII-d** | found short: the alliance gates and the counteroffer | 47 by `counteroffer` | the proposal of block 1051 answered short | `num_allies` and `num_team_members` are not in the crate: they are the step's first build |
| after | the residue of III, IV and VI (parked 4176a–c, 4177a–c, 4187a, 4187b, 4207b); VII-c's leaves that no game on disk moves | the leaves outside §7 to §9 | the commander's, as each lands | |

Not scored on run1575 at all, by the scout's read of its record: `hire_stamp`
and `hire_who` (no attack deal), `tribute_demanded` (frame 3600 at the
earliest), `tributes` (0 on every block), `good_deeds` (a paid deal),
`incoming_taunt_who` (the sender is who 0, the reset value).

## Readings a step settles before it builds

Each is the sizing scout's, from the export and the record, unadjudicated.

- **`docs/LEADERS.md` §8.2 misses a write**: `action_respond`'s commit sets the
  proposer's `agendas` bit 4 for the target. It is the parting on the human's
  `agendas[1]` at block 967 (theirs 4, ours 0), and it is why the French pass
  runs on 1027. VII-b2 ports it.
- **The shortfall's selector**: in the listing the branch is on
  `aggression[k]`, one store, and the French `aggression[0]` reads 0 on blocks
  1026 and 1051; §7.2 assigns the formula to "treaty is 2". The listing
  (`~/Library/Caches/attrition/pass-1113b/diplomacy.lst`) settles it before
  VII-d.
- **The record prints the per-target deal record and nothing compares it**:
  `dip.treaty` moves on blocks 1022, 1028 and 1042, outside the checksum's
  range. Comparing it is a free value diff on `action_treaty` and `clear_all`.
- **The counteroffer's ask is message text**: its state writes are
  `counteroffer` and `clear_agree` on both sides; no `dip.offers` moves on 1052.

## The lanes' items, and what each can fail on

| item | kind | the score | how it fails today |
|---|---|---|---|
| 4208 East Indies' frame 2 | start, whole | 38 packets onto the built start; `Checksum: lines 150 of 1,058` | every East Indies word's built walk parts on frame 2 |
| 4209 a cell's `down` after its goody box | rules, whole | run1186b's world line; the first parting on 39 of 41 built packets | ours −3, theirs −1 |
| 4210 the late-age Great Lakes fog | start | 8 packets onto the built start | four fog cells part on frame 0 |
| 4211 the last three lobbies | start | `Start: 158 of 161` | two styles have no mask port |
| 4204 the quit's `clear_orders` | rules, whole | run667's 45 unit rows and 4 Mill rows, run609's 4 | as designed by the pass before |
| 4212 the field's colour | render | a hue row on the grass tiles, seen under its bar | the row that exists reads 1,104 of 1,109 on a browner picture |
| 4213 the lake | render | `RUN1564_3279_GROUND_STILL` 1,028 of 1,920 | 892 tiles part |
| 4214 step H | input, with the capture section | `Checksum: frames`, per line the first parting frame of 24,000 | no per-frame checksum of the original exists |
| 4215 the page starts any lobby | lobby | `Lobby:` the page starts N of 161 | the client composes Great Lakes alone |

Order: the builder on the chain. The start lane: 4208, 4209, 4210, 4211,
4204. The render lane: 4212, 4213. The third lane: 4214, 4215.

## Built

- **`rondata::corpus`** (an Opus 5.5 builder, branch
  `worktree-agent-a1ab037132175f084`, tip `5859788cf`). `Ledger` reads
  `docs/RUNS.md`'s headings; `in_tree` keeps a name with no run token, a run
  below 1546, and a run the ledger heads. Routed through it: the packets
  census, the transfer corpus, the lobbies on disk and map generation's
  traces. **Made to fail**: on the old code, a mirror of the 109 packet names
  plus `run9999` under `RON_GOLDEN_DIR` failed `the packets on disk are the
  census's`; on the new it passes and prints `unfiled on disk, not in this
  tree's docs/RUNS.md: run9999`. Six unit tests on authored directories. No
  pin moved: every capture on disk from run 1546 on has its heading. The
  builder's threshold count: 67 gamelog run tokens have no heading, the
  highest run1042, reading every token in a heading; 103, the highest
  run1545b, reading the first alone.
- **`the_checksum_line_groups_its_bytes`** asserts that no bare number of four
  digits stands in the line and one grouped number does; a second test shows
  it fail on an ungrouped line. Same builder.
- **`tools/handover.py reap`** polls a fresh roster after `forget`: the reap
  of `e9aff36b` read a roster a tick old, saw the row it had just removed
  and printed `forget kept the row; ccc archive: exit 1`. The row was gone.
- **The suite re-timed** (`docs/audit/2026-10-15-suite-timings.txt`).
- **The lane gate's own budget** and the `start` kind off the whole gate:
  below, under "The gate", with its builder's tip.
- **The documents**: entry 132; `docs/LOOP.md`'s roster, booking gate,
  corpus, chain and pass clauses; `docs/RUNS.md`'s head; the queue; the
  tracks page; one sentence of `CLAUDE.md` on a worker's gate, on Ramon's
  word ("yes you may edit").

## Not done, and said

- **No second reading** was run; no marked row stood for this pass.
- **The recordings are not on the ledger.** `RECORDINGS_ON_DISK` lists
  `Playback - <date>.rcx` files with no run token; a capture that leaves one
  in `Logs\` would turn another tree red. No capture did this tranche.
- **The suite's form is not changed**: 156 games are still replayed by two or
  more tests each.
- **The timings file still ages a test a landing.** The pass re-times it; a
  whole gate holds every test's seconds and writes them nowhere a lane reads.
- **Minting a `Declare` through `Command`** is not booked; it follows VII-b1,
  which builds what it would do on our side.
- **`docs/audit/census-backed.txt` is stale** (dated 2026-10-21, 1,696 rows
  against 1,456 backed), and the census files map generation's and the
  checksum's classes under `engine`. Not fixed.
- **`tranche.py --base`** still prints no rows for `Book N` subjects without
  the word "item" (41 of the last 60 bookings). Not fixed.

## The gate

