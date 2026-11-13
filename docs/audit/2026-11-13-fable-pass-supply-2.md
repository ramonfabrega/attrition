# The second pass of 2026-11-13, by supply: the draws are blind to state, the gate is cut by rule, and the chain is ordered

*2026-11-13, from 13:33 −05:00, the `steer` row on Fable 5.2. `commander` handed
over at `331d85b01` with one designed item on the queue and the chain's next
step unordered (`docs/DECISIONS.md` entry 126 §4). The charter: the staged
accept of item 4207, supply for three lanes, a capture filed mid-gate, a
literal the merge pins missed; then, by a message from the `lore` row it said
was at Ramon's request, the gate's cost, what parity means and a function
census. Ramon was on the row from about 14:00. Decisions in entry 132. Six
scouts, two builders and a spike ran, each a `lean` agent on Opus 5.5; their reports are
under `~/Library/Caches/attrition/pass-1113b/`. The pass before it is
`2026-11-13-fable-pass-supply.md`.*

## For Ramon

Ramon was on the row; what he decided is entry 132 §10 and §11. What stands
for him to read:

- **The words are whole on draws, and draws are blind to state.** Two recorded
  files three units apart in one Scout's target agree on all 24,001
  random-number words; on 41 packets in draw lockstep the units, builds and
  guys lines agree on 0. **Parity is now the checksum's** (his word): zero
  mismatched frames on the twelve state lines over a declared set. It reads
  0 games today, and step H (item 4214) is what lets it be counted.
- **The gate is cut by rule**: a lane's gate runs its filters, a booking gate
  is scoped (20.0 minutes by the median, from 33.0). Neither is measured yet
  under the new rules; the next pass reads both.
- **The suite's form is item 4216, a fifth lane.** Measured: 136 games walked
  once in 30 s on 14 threads; their modules' 1,051 tests cost 2,247 s. The
  estimate over the whole suite is 2.3 to 2.5 times less CPU, not measured
  end to end. **The lane's model is his**: the pass's take is Fable 5.2.
- **A sixth worker** has one independent piece to take, the literal pins as
  data, once 4216's second stage lands.
- **The XML and the decompiler rules stand** ("both seem fine as is").
- **`fseventsd`** still holds a core and 27.9 GiB, 29 days up, swap 15.1 of
  16 GB. `sudo killall fseventsd`, or a restart, is his.
- The tranche before the pass: ten landings in 3.9 hours on six booking
  gates, six lanes at 64.59 USD (10.77 a landing). The pass moved no score.

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
| the suite by phase and shape (scout: 51 tests alone, the timings file by family and by game) | by family, in the file's seconds: words 2,081 over 946 tests, golden 974, map generation 526, harness 471, audit 252, coverage 221, packets 129. By the game a test replays: 814 kept games, 1,650 tests, 4,316 s, each game's longest test alone 2,363. A window: standing up 0.34 to 0.64 s, ticking 0.2 to 3.6, decoding 0.2 to 0.39, comparing 0.09 to 0.21. **Where the tree shares**: one test walks 436 windows over 123 games, 202 resumed from a kept prefix, in 159.8 s of wall and 604.7 CPU-seconds, and their 589 standalone twins cost 1,272 s; another walks 110 whole games in 294.6 thread-seconds where the 773 tests on those games cost 1,784. Each of 129 map-generation tests reads the preambles of 1,203 dumps again, about 3.2 s. 1,480 of 3,578 pins hold a literal in the test's body. **The file is stale**: 2,171 of its 2,360 rows are of 2026-11-09 and the sampled big ones ran at 0.01 to 0.45 of their row |
| the floor (a spike: `diff::corpus_floor::every_word_walked_once`, branch `worktree-agent-a5aaf4d6276762f1d`, tip `9e5aee8e0`, unmerged) | 136 scoreboard games, 1,201,580 frames, each stood up and walked once by the floors' own functions, every word at its pinned length: 82.4 s of wall and 324.8 CPU-seconds on 4 threads, 43.0 and 334.3 on 8, 29.6 and 372.1 on 14; peak footprint 3.3, 4.5 and 5.5 GiB; the host's load 6 to 22 beside it. The 130 tests that walk those games to their floor cost 291 s one at a time; all 1,051 tests of their 126 modules 2,247. Standing up is 104 s of 381 (the install's load 46.7, 0.34 s a game); ticking and the draw comparison 72 %. Not walked: the first pair's three words and the two 24,000-frame captures |
| the window (`lore limits`) | the week 33 % at 18:01Z, 35 % at 18:57Z and 36 % at 19:25Z, Fable's 21 %, 23 % and 24 %; both close 2026-11-14 06:00 −05:00; the stop 95 % |
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
- **No verdict cache** (§8). **The suite's form is one walk a game** (§11).
- **The builder's two calls on the lane gate stand**: the floors and census
  pins run in every lane gate, and a lock or toolchain change keeps the
  client's targets.
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
| 4216 one walk a game | structure, whole | `Suite:` stand-ups from zero in a whole gate, and its test CPU-seconds | 1,650 tests stand 814 games up from frame 0 |

Order: the builder on the chain. The start lane: 4208, 4209, 4210, 4211,
4204. The render lane: 4212, 4213. The third lane: 4214, 4215. The fifth
lane: 4216.

## One walk a game: the stages of item 4216

The form: a kept game is stood up and walked once in a process, every check
on it rides that walk, and a check added to a game already walked costs a
probe. The gate's unit today is one process a test, chosen on 2026-10-17 for
latency and a memory ceiling a test (`docs/TEST_PERFORMANCE.md`, the isolated
runner); no decision makes the process boundary a rule of correctness.

| stage | what | its number | first act | it stops and says so when |
|---|---|---|---|---|
| 0 | the spike's instrument merged as an ignored test, on the differ ledger and the shape pins | the floor: 136 games, 325 to 372 CPU-seconds | the test green on the merged tree | — |
| 1 | the runner's group unit: the tests of one game, in ascending frame order, in one process on test-named threads; one group for map generation | stand-ups from zero in a whole gate; map generation's 526 s | the count of tests run equal to the count listed, seen red on a group that drops one | the fixture audit stops matching test for test, or a group's memory passes the ceiling |
| 2 | a standalone window resumes the group's kept walk, through the scope the coverage test already uses | the words' 2,247 s; stand-ups from zero toward the 814 games | the audit's, a chapter's and a phase-traced walk each diffed against the plain walk on one game, frame for frame | a walk differs: that shape keeps its own stand-up and is named |
| 3 | takes of one lobby and seed keyed as one game | stand-ups toward the distinct games | `rngcmp` over two takes' common frames, 0 differing, on every pair folded | a pair differs: it stays two games |
| 4 | the coverage and ages tests observe the shared walks | their 406 and 335 s in a gate | the recorder fed from a group's walk on one module, its record equal to the driver's | the recorder needs a walk the tests do not make |
| 5 | the 1,480 literal pins as rows | none on the clock; a re-pin becomes a line of data | one module's pins moved, its re-pin seen as a one-line diff | — |

What the lane is told: a game's checks share a walk, so a panic mid-walk
must still report every check it did not reach as not run, never as green;
a test that changes the walk (`harness::checkpoint`'s interventions, the
built-start packets) stays alone; the ground guard reads the thread's name
(`Drop for Built`), so a group keeps test-named threads; the install's load
is 0.34 s a game and is taken once a process. **A whole re-time on a quiet
host is its baseline**, taken first (`tools/suite_timings.py <install> --out`).

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
- **The lane gate's own budget** (an Opus 5.5 builder, branch
  `worktree-agent-a24fc4dd1e7538004`, tip `17e990f92`, three commits). A
  lane's gate skips every timed test its filters and the paperwork do not
  name (`LANE_OWN_BUDGET_SECONDS` 0; the scoped booking gate keeps one
  second); it keeps `diff::floors::` and the census pin, which two older
  rules need in every lane gate. A `start` brief names filters (`diff::start
  lobby:: minted:: checksum_lines::packets:: mapgen:: census:: blind::`) and
  no longer gates whole. A lane leaves the client crate's targets to the
  booking gate unless it touches `crates/client/` or `crates/render/`: the
  clock test ran 8 to 362 s on four lane gates and is behind two of the
  seven red logs. **Red first**: `lane_skips` on the old code skipped 1,340
  of 2,171 rows and none under a second; the client test read `'client' not
  found in ['sim']`. 128 tools tests and the 994 of the offline step green.
- **The documents**: entry 132; `docs/LOOP.md`'s roster, booking gate,
  corpus, chain and pass clauses; `docs/RUNS.md`'s head; the queue; the
  tracks page; one sentence of `CLAUDE.md` on a worker's gate, on Ramon's
  word ("yes you may edit").

## Not done, and said

- **No second reading** was run; no marked row stood for this pass.
- **The recordings are not on the ledger.** `RECORDINGS_ON_DISK` lists
  `Playback - <date>.rcx` files with no run token; a capture that leaves one
  in `Logs\` would turn another tree red. No capture did this tranche.
- **The suite's form is not changed**; it is item 4216. The spike's branch is
  pushed and not merged.
- **The timings file is re-timed for its 189 new tests alone.** Its other
  2,171 rows are four days old and read high. A whole re-time takes about
  ninety minutes one test at a time and is 4216's first act. A whole gate
  holds every test's seconds and writes them nowhere a lane reads.
- **A lane's gate and a scoped booking gate are not measured under the new
  rules.** No lane ran in the pass.
- **Minting a `Declare` through `Command`** is not booked; it follows VII-b1,
  which builds what it would do on our side.
- **`docs/audit/census-backed.txt` is stale** (dated 2026-10-21, 1,696 rows
  against 1,456 backed), and the census files map generation's and the
  checksum's classes under `engine`. Not fixed.
- **`tranche.py --base`** still prints no rows for `Book N` subjects without
  the word "item" (41 of the last 60 bookings). Not fixed.

## After the commit: four notes from `lore`, weighed while the gate ran

`lore` answered the pass's rulings with four notes, none a ruling. Each is
weighed here; this section and nothing else was added after the gated commit.

- **The 4.7 times inside a gate is not the background clamp.** `lore`'s
  hypothesis: macOS confines a background-policy process to the efficiency
  cores. Measured on this host (M4 Max, 12 performance and 4 efficiency
  cores), the 9,632-frame word test alone: 1.98 s of wall and 5.73 G cycles
  as it runs; 1.31 s and 5.31 G under `nice -n 10`, which is what
  `tools/memcap.sh` gives a test; **6.46 s and 8.72 G cycles under `taskpolicy
  -b`, 4.9 times**, on the same 30 G instructions. So the clamp would
  explain the gate's figure exactly, and it is not there: the gate's own
  test processes, sampled three times as its guard step ran, read priority
  31 to 46 at nice 15, and the host's load read 6.5, 34.0 and 64.5 over
  those fifty seconds on 16 cores. The slowdown is more runnable threads
  than cores. What runs them at a gate's start, past the budget's twenty
  slots, is not read; item 4216's first stage reads it, because a group
  runner changes it. One more sample, eight minutes into this pass's own
  gate with no lane out: the load read 89.5, one `rondata` process held 7.2
  cores, six `python3` processes 4.9 between them, and `fseventsd` one.
- **The dev profile carries full debug info**: `Cargo.toml` sets only
  `[profile.release]`. `target/debug` is 15 GiB in the integration tree and
  10 in the gate's. `lore` cites `debug = "line-tables-only"` with the
  dependencies' off. Not measured here. It rides with item 4216's stage 0:
  the bytes of `target/debug` and a clean debug build's seconds, before and
  after, and a panic's backtrace still naming its line.
- **The widening rule as data.** Fifty of sixty landings widen a scoped
  gate, and seventeen of those touched `rondata` code that was mostly a
  test module of its own. A leaf test module that nothing else names
  cannot move another module's pin; a script can tell one. Not built:
  item 4216 shrinks what a widening costs, and the next pass reads whether
  the rule is still worth narrowing.
- **The documents and the tools as one CLI with an index** is a pass of its
  own and is not this one's.

## The gate

Whole, on the commit that carries this record, by `tools/gate_run.sh`, started
at 14:45 −05:00: `~/Library/Caches/attrition/gates/gate-pass-1113c.log`. The
record is written before the gate runs, so its verdict is that log's last
line, and neither the push nor the handover happens on anything but `gate
exit 0`. Before it: `tools/guard.sh --built` green on the documents after
three reds it forced (an index row for entry 132, the queue's header a line
over its pin, two items that read as capture bookings with no map); the 128
tools tests and the 994 of the offline step green on the lane gate's
changes; the corpus builder's 56 tests green on its branch.

**The receipt, added with the last section**: `gate exit 0` on `8a0b2a0e7`, 7
of 7 steps, release 27.5 minutes (offline 1.9, guard 1.0), admission 25.0 s
over 2,392 leases, load 3.1 at its start and 6.5 at its end, `Rows: one
driver row at most: steer 2186a1cf (working)`, 231 GB free. No lane and no
builder ran beside it, and its release step took what the whole gates beside
four workers took (the median 27.9): the gate's minutes are its own, not the
lanes'. The section above and the queue's handoff lines are gated scoped
over that commit (`gate-pass-1113d.log`).
