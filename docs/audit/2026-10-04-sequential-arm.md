# The sequential arm — the steering session's ledger

Parked 1448, `docs/DECISIONS.md` entry 60 §2. One Opus 5.5 worker,
`--effort high`, lane `seq-opus` on branch `worktree-seq-opus`, base
`10d6f5e4`, spawned 2026-10-04 02:14Z under `AGENTS.md`, from item 1446.
The model is read from the lane's transcript (`claude-opus-5-5`, every
assistant record), never from the spawn flag. This file is the record
the twenty-fourth pass judges with the user: one section per landing,
written at the landing from the diff against the journal, and a closing
section at the arm's stop with `lore trace`'s totals. The estimate on
record (entry 60): ten items in under twelve hours, French East Indies
past 9,000, nothing closed, under 200 USD list.

## Landing 1 — item 1446, French East Indies 8182 → 8236

Commits `1634ab69` (the landing, 02:59Z), `c58bebd4` (the gate's
clippy verdict, `type_complexity` on a tuple alias — no logic change,
03:00Z) and `c3020401` (the gate's record, 03:11Z). The first full gate
stopped at clippy before any test step — the journal calls it a
failure, not a partial pass; the second, on `c58bebd4`, **exit 0, six
of six steps**, fixed 13, rondata 703, sim 1324, fixture audit 2532
requests and none missing, memcap peak 15023 MiB of 20 GiB. Pushed:
`origin/worktree-seq-opus` is `c3020401`, verified by fetch. The done
line reached this session at 03:12Z, one line, in the agreed shape.
Spawn to gated-and-pushed landing: **57 minutes**.

**Diff against the journal** (`docs/journal/2026-10-03-item-1446.md`,
141 lines; VISION §6.4, 90 lines): they agree. The journal's four
findings each have a code site — `Build::start`'s wonder call at the
tail of `start_building`, the two-plane write in
`update_local_seen_build`, the two arms of `update_all_seen`, and
`init_build`'s `check_ever_seen` — and each a test made to fail by the
mutation it names (`mutations.log`). The item's own evidence is the
strongest kind the method has: a prediction written before the capture
(block 7946, from `frame_started` 7941 and `frame & 7 == 1`), a capture
run to kill it (run613), and a value diff on the bytes the mechanism
hangs off (`ever_seen` 2 → 255 on both buildings on 7946, both met
bits on 7946 and no other block). It also dated the draw stream's
blindness — the `Build::start` call removed leaves the word at 8236 and
only the value rows part — which is the rule in `CLAUDE.md` measured
once more.

Three things to carry to the pass:

- **A strengthened pin, not a weakened one.** run240's world pin at
  17087 held 44 fog half-cells and nine danger half-cells as a known
  residue; both are now asserted to zero, the whole plane and both
  maps. One assertion tightened, none deleted, no new `allow` or
  `#[ignore]`.
- **The instrument gap is parked, not patched** (1450): `ever_seen` and
  `ever_seen_completed` stay in `UNCOMPARED_BY_THE_INSTRUMENT`; 1446
  compares them with its own helper on four East Indies windows only.
  Whether the shared widening takes them is the pass's.
- **Review debt, stated.** Three arms read off the listing; the start
  arm and the plane test have a capture behind them, the resync arm
  rests on run240 (a diff) and the `flags & 0x20` / `is_fort` gates on
  the reading alone. No blind reading, none claimed — the same debt as
  Codex's twenty (1447), written the same way.

**Bookkeeping the arm did itself**: queue rewritten (headline block,
opener, scoreboard `w8236`, item 1446 deleted, 1449 booked by frame and
draw delta with no mechanism), `FLOORS`/`WIDENINGS` re-pinned, run611
and run612 (Codex's, never entered) and run613 entered in `docs/RUNS.md`
with hashes, parked 1450 filed, backlog 21 → 22. The queue ledger and
the scoreboard parser pass on its tree, and the gate confirms it.

**Scored against Codex's tranche**: Codex averaged 46 minutes a landing
over twenty, gate included at the batch; this one took 57 from spawn to
a gated, pushed landing with the gate run per item. Journal 9 KB with
the gate section, in Codex's range. One red gate on a lint, recorded
and corrected, as Codex's one red gate (1444's batch) was.

## Landing 2 — item 1449, French East Indies 8236 → 8385

Commit `f533d1b5` at 03:49Z, **38 minutes after landing 1's push**;
the full gate on it started at 03:49Z and recorded in `99cb157d` at
04:00Z: **exit 0, six of six steps**, fixed 13, rondata 706, sim 1324,
fixture audit 2565 requests and none missing, memcap peak 15226 MiB.
Pushed: `origin/worktree-seq-opus` is `99cb157d`, verified by fetch;
the done line reached this session at 04:01Z in the agreed shape.
**49 minutes from landing 1's push to landing 2's**, gate included.
Three captures (run614–616), each booked with its killers before it
ran, each the same game as run600 by the RNG trace.

**Diff against the journal** (`docs/journal/2026-10-03-item-1449.md`,
108 lines; SYNC §3.28, AI §114): they agree, and the item is a model
of the frame-first rule. Booked by frame (8236, a collision of `1/41`
with `1/56`), it walked back three captures — `1/56`'s ring spot on
8156 with every input but one agreeing (run614), that input `1/51`'s
birth on 7963 already holding BUILD (run615), and the wonder's recruit
phase `(7962 + 2022) & 31 == 0` running before the birth here — to a
tick-order cause the tick's own comment had called "still ours": the
queue ran in a pass of its own after every building's `Wall::process`,
where the original runs `Build::do_queue` inside each building's
`Build::process`. The change is small and in the right place
(`process_queue(b)` right after `process_building(b)`;
`process_queues` kept under `#[cfg(test)]` for its one unit test), the
vslot `+0x1b4` is named from the Build vtable, and the mutation returns
the word to 8236 and fails all five pins.

To carry to the pass:

- **Five windows moved, none arrived, run613 holds** (156 → 127,
  147 → 121, 210 → 124, 153 → 122, 156 → 120; 117 → 117 because it ends
  before the birth), and both suites otherwise hold — a check that no
  other floor moved, run before the gate.
- **No sim unit test for the order itself**; the arm says so, and why
  (the recruiter's test needs the tech tree). The run614/615 pins on
  `1/51`, `1/56` and camp `1/2019` are the assertion. A later
  unit-level test of the per-building order is a reasonable ask.
- **Stated as unestablished**: the tower pass and the gather re-entry
  pass are still passes of their own; no capture separates them. The
  reading of `Build::process`'s order is backed by run615 for the queue
  and by nothing for the tower. No blind reading.

Bookkeeping done by the arm: queue (headline, opener, scoreboard
`w8385`, 1449 deleted, 1451 booked by frame and draw delta — the AI's
make list on run616 block 8385, parting since run610's 8185), pins and
`WIDENINGS` re-pinned, run614–616 entered in `docs/RUNS.md`, SYNC §3.3's
sentence amended in place to point at §3.28. The number 1450 is parked
1450 (landing 1); the arm's numbering is clean.

**Pace after two**: 57 and 49 minutes, spawn to pushed landing each;
two gated landings in 1 h 47 m. Codex's twenty took 15 h 20 m, 46 a
landing with the gate at the batch.

## Landing 3 — item 1451, French East Indies 8385 → 8840

Commits `a96211c3` (the landing) and `f9ff75c2` (a doc-comment line
clippy read as a list item), both 04:30Z, **30 minutes after landing
2's push**; the gate's record `32aa86a0` at 04:41Z: **exit 0, six of
six steps**, fixed 13, rondata 707, sim 1325, fixture audit 2582
requests and none missing, memcap peak 14846 MiB. Pushed:
`origin/worktree-seq-opus` is `32aa86a0`, verified by fetch; the done
line reached this session at 04:42Z in the agreed shape. **41 minutes
from landing 2's push to landing 3's.** The journal's gate section
owns a slip worth the pass's eye: `a96211c3` was committed after a
clippy run piped through `tail`, which hid its failure — the queue's
own "never piped, a pipe launders the exit" rule, met once more and
recorded by the arm itself. One capture (run617), the new word's
widening; the cause needed none.

**Diff against the journal** (`docs/journal/2026-10-03-item-1451.md`,
100 lines; AI §115, §24 amended): they agree, and this is the "grep
the disk before booking a capture" rule paying out whole. run612 had
been captured with `RON_LEADER_PROBE` over 8175..8240, so the
original's own offer on 8184 was on disk: the Farm for city 2 at
45156 against ours 52593. The arm did the arithmetic *before* opening
the listing — holding the base, 45156 needs multiplier 170, which is
384 before two ×2/3 steps where ours had 448, which fits `(k + X) × m0
/ k` with X = 2 for our 3 — wrote it down, and then found
`max(3 − largest_gather, ter)` in both arms of `create_buildings`, with
`compute_largest_gather` clamping the field to `[1, 2]` at load. The
crate's seam list had it at 0; the third reading (`create-buildings.md`
§3.2) had the formula right and the field's value wrong. The payoff
probe reproduces 45156 exactly. `crate::world::largest_gather` computes
it from `LANDS` rather than pinning 1, with a unit test made to fail by
dropping the clamp's floor.

To carry to the pass:

- **The first landing of the arm that moved other games.** Sixteen
  widening pins fell across Great Sahara, Toughest, the second pair
  and the French pair; the arm diffed their keys before and after:
  **69 keys leave, none arrive**. No floor moved; Toughest holds at
  12538. The third map's 7/6 offer residue on 15582 and 15982 is gone.
  One Toughest row moved without closing (the make head on 10785,
  51046 → 43828 against 44890) and stays pinned with its value.
- **Review debt: none claimed on the formula** — run612's probe and
  sixteen widenings back it, which is the first row in this ledger
  where a landing's arithmetic is diff-backed end to end. The clamp's
  upper bound (2) is reached by no shipped land, which is stated.
- **A seam that was a wrong constant.** `ai_build.rs`'s module header
  listed `largest_gather: 0` among what is "not modelled"; the field was
  not unmodelled, it was mis-valued, and the seam list did not say
  which. For the pass: whether `tools/seams.py`'s rows should carry
  "assumed value" against "not carried", since a reader of the list
  cannot tell them apart.

Bookkeeping done by the arm: queue (headline, opener, scoreboard
`w8840`, 1451 deleted, 1452 booked by frame — unit `1/60` the original
holds and we do not, run617 block 8838), sixteen pins and `WIDENINGS`
re-pinned, run617 entered in `docs/RUNS.md`, the module header struck
through in place. Its slips section records four tooling mistakes
caught by the tools' own assertions before any conclusion was drawn.

**Pace after three**: 57, 49 and 41 minutes, each spawn-or-push to
pushed landing; **455 frames in one landing**, the largest move on
this word since the pair stood up. Three gated landings in 2 h 27 m;
French East Indies 8182 → 8840 (+658).

## Landing 4 — item 1452, French East Indies 8840 → 9655

Commit `cbd08255` at 05:25Z, **44 minutes after landing 3's push**;
the gate's record `d8d99d75` at 05:39Z. **The first landing of the
arm not on a full green gate**: the full required-fixture gate on
`cbd08255` **exited 1** — fixed 13 and sim 1326 green, rondata 709 of
710, the one failure `coverage::every_key_the_dump_prints_is_read_or_
pinned` on run622's bare city-name line `Lyons`, which nothing reads;
2607 fixture requests, none missing; memcap peak 16690 MiB. The
correction adds `Lyons` to `UNREAD` beside Paris, Brest and Nantes
(items 1442/1443 did the same), one line and a comment, no logic; the
arm then re-ran the five coverage tests, clippy, fmt and the guards,
each exit 0, and wrote the result as **combined validation, "not a
claim that the original full gate passed"** — `AGENTS.md`'s clause,
applied as written. The steering session's read: the failure is the
coverage pin doing its job on a new window, the correction is a pin
and nothing else, and the clause fits; the pass should still note
that one of four landings rests on it. Pushed: `origin/worktree-seq-
opus` is `d8d99d75`, verified by fetch; the done line reached this
session at 05:40Z and named the combined validation in its first
clause. **58 minutes from landing 3's push to landing 4's.** Five
captures (run618–622), each with its hypotheses written before it ran;
the trail is the longest of the four so far and the cleanest to read.

**Diff against the journal** (`docs/journal/2026-10-04-item-1452.md`,
94 lines; COLLISION §24, VISION §6.4 noted): they agree. Booked by
frame (8838, a transport the original holds and we had already
emptied), the arm walked it back five captures and 2,700 frames: the
boarding identical (run618), the transport parting on 8651 against a
ship `1/16` (run619), that ship standing 11 units off since before
7351 and pushed on 6141 (run620, both ships parting by the same angle
with opposite signs), and the guard probe on 6141 (run621) showing the
**idle** ship calling `detect_boat_collision` at its own position with
`mates` clear, outside any move — `Unit::work`'s block before
`do_job`: pushed within four frames, the boat arm, a job that is not a
move. The change is that block, with the seven move-like jobs exempt
as the listing has them. The new word's widening (run622) exposed a
second, small write — `Wall::activate`'s `ever_seen_completed |= ally
mask`, the wonder reading 2 on the block it finishes — made in the same
landing with its own capture evidence.

To carry to the pass:

- **Every French window lost the same 11 keys — `1/16`'s — and none
  arrived**; run617 lost 77. The standing residue a reader of the
  French widenings had seen for ten items (a ship 11 units off) was
  this. No floor moved.
- **One unit test, and it is honest about its edge.** `a_pushed_ship_
  pushes_back_from_where_it_stands` fails without the block and tests
  the four-frame window on both sides; the move-job exemption is stated
  as read-only rather than tested through a moving ship's own step,
  "which would conflate the two pushes" — the right call.
- **Review debt, stated**: the push-back's siege, supply and hero arms
  and the move-job exemption are read, not exercised. No blind reading.
- **A slip the arm owns**: a comment-only edit to `orders.rs` while the
  full suites ran. No logic changed, and it says so; the frozen-tree
  rule was still crossed, and the pass should weigh whether the
  agreement's "do not edit the validated tree" needs the same teeth
  the gate has.

Bookkeeping done by the arm: queue (headline, opener, scoreboard
`w9655`, 1452 deleted, 1453 booked by frame — an air step's draw at
9655, `do_air_physics`, no flyer parting before 9657), pins and
`WIDENINGS` re-pinned, run618–622 entered in `docs/RUNS.md` (run619
as positions-only evidence, not a pinned window, and said so).

**Pace after four**: 57, 49, 41 and 58 minutes, each to a pushed
landing; **815 frames in one landing**, the largest yet. French East
Indies 8182 → 9655 (+1,473) in 3 h 25 m. Entry 60's estimate — ten in
under twelve hours, past 9,000 — is on pace at four, and the 9,000 is
passed.

## For the Loop, noticed by the steering session

- **`notify_when_idle` is a turn-end signal, not a landing signal.**
  The arm backgrounded run613's wait and ended its turn at 02:31Z, as
  the rules say; the idle notice came, and each re-arm that the 1412
  rule calls for fired at once with the same 02:31 stamp — three
  notices for one idle in two minutes, while `ccc list` read the lane
  `working` on the background task. A lane whose waits are backgrounded
  is idle to the harness for most of its life. This session's
  substitute is one `run_in_background` wait that exits when the lane's
  branch tip moves or the lane leaves `working`
  (`$CLAUDE_JOB_DIR/tmp/wait_seq_opus.py`), re-armed on its own exit.
  Parked briefly as 1449 (`e6598a2c`) and reverted (`133b9dd7`) because
  the arm books its own numbers and had taken 1449 for the next word;
  **the steering session books no number while the arm runs**. For the
  pass: amend the fan-out rule's subscription clause, and decide whether
  the branch-tip wait graduates into `tools/`.
- **The arm's evidence directory is denied to this session's reads**
  (`~/ron-data/lab-experiments/2026-10-03-item-1446-opus/`, the
  classifier's "Modify Shared Resources" on an `ls` and a `tail`). The
  gate verdict is therefore read from the arm's follow-up commit and
  its done line, not from the log.
