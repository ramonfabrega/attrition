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

## Landing 5 — item 1453, French East Indies 9655 → 9777

Commit `4f60d52c` at 06:06Z, **27 minutes after landing 4's push**;
the gate's record `5fc69d45` at 06:17Z: **exit 0, six of six steps**,
fixed 13, rondata 712, sim 1326, fixture audit 2629 requests and none
missing, memcap peak 16433 MiB. Pushed: `origin/worktree-seq-opus` is
`5fc69d45`, verified by fetch; the done line reached this session at
06:18Z in the agreed shape. **38 minutes from landing 4's push to
landing 5's.** One capture (run623), the new word's widening with the
leader probe; the cause needed none — the disk answered again.

**Diff against the journal** (`docs/journal/2026-10-04-item-1453.md`,
78 lines; SYNC §3.29): they agree. The word was the flyer's edge coin
(`do_air_physics+0x639` under `do_strafe`), and run622's own call
window already proxied every flyer's step: the ten wild birds matched
frame for frame, the two **gulls** flew in the original and stood
still here — a seam both `orders.rs` and `anim.rs` had stated. The
change is two measured steps, the first its own mutation record:
flight alone toward the dock (gull 1 lands ~1000 units off, the coin a
frame early — the killer written beforehand fired), then the birth
snap (`Unit::init`'s tile snap, which the wild bird already took), on
which both gulls match the original **exactly** on every proxied frame
on disk, ~3,900 frames after birth included. The gull's figure skip
was lifted, moved nothing, and stays with its comment amended.

To carry to the pass:

- **A seam closed by the disk alone**: the third of five landings
  whose cause needed no capture. `tools/seams.py`'s list shrinks by
  one named seam (the gull's flight) and gains a smaller one (the
  `StrafeOrder` itself is not carried; the goal is read off the dock
  slot) — the arm wrote the new `SEAM` where the old one stood.
- **Six keys leave run622's widening (102 → 96), none arrive**, and
  one older pin (`a_dock_with_a_gull_type_draws_twice`) moves from the
  unsnapped birth to the snapped one — a pin corrected, with the
  reason in the test. No floor moved.
- **The open window is checked only to the word's own block**, because
  past 9777 the original's purchase stands alone "by design" — said in
  the journal, which is the honest shape, and the next item is that
  purchase.
- **No review-debt line in this journal.** The flight is diff-backed
  on two windows; the figure arm is measured as unobservable (nothing
  dumps owner 9's figures). The pass should ask whether "no debt" is
  the arm's claim or an omission; the specification section is where
  the answer belongs.

Bookkeeping done by the arm: queue (headline, opener, scoreboard
`w9777`, 1453 deleted, 1454 booked by frame — the original buys
building `1/2035` on 9778 and we buy nothing, the offers already on
disk in run623's probe), pins and `WIDENINGS` re-pinned, run623 in
`docs/RUNS.md`.

**Pace after five**: 57, 49, 41, 58 and 38 minutes, each to a pushed
landing; five gated landings in **4 h 03 m**, French East Indies
8182 → 9777 (+1,595). Entry 60's five-or-fewer threshold — "the
architecture was not the variable" — is passed, and the pace is
twice its estimate's.

## Landing 6 — item 1454, French East Indies 9777 → 10131

Commit `52503b86` at 07:05Z, **47 minutes after landing 5's push**;
the gate's record `3fb1953b` at 07:16Z. **The second landing on
combined validation**: the full gate on `52503b86` **exited 1** — five
of six steps run, fixed 13 and sim 1328 green, rondata 713 of 714, the
one failure `blind::the_blind_list_is_the_pinned_residue`, because the
new TECH section cites `Wonder::init@0073c5e0`, the second writer of
`Game::wonders`, which no kept trace enters; 2650 fixture requests,
none missing; memcap peak 16666 MiB. The correction pins the address
in `blind::NEVER` (141 → 142) with its line of comment, no logic; the
arm re-ran the eight `blind::` tests, clippy, fmt and the guards, each
exit 0, and wrote it as combined validation, not a full-gate pass. The
steering session's read: as landing 4's — the guard did its job on a
new citation, the correction is a pin, the clause fits. Pushed:
`origin/worktree-seq-opus` is `3fb1953b`, verified by fetch; the done
line reached this session at 07:17Z, shorter than the agreed shape but
carrying every field. **58 minutes from landing 5's push to landing
6's.** One capture (run624), the new word's widening with the leader
probe; the cause needed none — run623's probe held the original's
offers and the purchase.

**Diff against the journal** (`docs/journal/2026-10-04-item-1454.md`,
91 lines; AI §116, COSTS and TECH each a new section): they agree.
Three terms, each measured before the next and each its own mutation:
the city limit (the crate's Pyramids arm read `Nation::pyramids`, which
nothing in a game sets, where `get_city_limit` reads `has_wonder(0x20e)`
— four of four against the original's five), the price (`get_cost`'s
city tail, `PYRAMIDS_CITY_DISCOUNT` after the ramp, 210 → 140 on both
buckets, with the Bantu term ahead of it and each truncating alone),
and `already_built` (a built wonder leaves every player's list —
parked 797, read on 2026-09-25 and never built, now built with its
killer written first). The limit alone moved nothing; the price took
the word to 9781; `already_built` to 10131. The crate carries
`Game::wonders` as `wonders_built`, set where the census notes an
activated wonder, read by `type_avail`.

To carry to the pass:

- **A parked reading became a build on a word's frame**, the way the
  parked file is meant to work: 797 struck through in place and pointed
  at its item.
- **Two tuning slots added** (`BANTU_CITY_COST` 75,
  `PYRAMIDS_CITY_DISCOUNT` 33) and `ron_slots` widened 345 → 347, so
  `rondata`'s drift check re-derives them from the install; the Bantu
  term is a reading alone and the journal says so.
- **run623's widening falls 186 → 97, and the 97 are its first block's
  standing keys** — nothing parts on 9773..9784. The purchase test
  asserts the fall of each bucket by 140 rather than the food value,
  because food stood three high from before the window; the reason is
  in the test.
- **A census residue parked with its cheapest dating** (1456: the
  capital's `filled` one high on every French window from 6136, moving
  no draw yet) — a finding that names no score, parked rather than
  chased, with the capture that would date it named.
- **Three slips owned**, one of them the yield rule: "one backgrounded
  `sleep 1` was a yield, which `CLAUDE.md` bans; it carried nothing."
  The arm reports its own infractions of the parent agreement without
  being asked.

Bookkeeping done by the arm: queue (headline, opener, scoreboard
`w10131`, 1454 deleted, 1455 booked by frame — the state parted before
run624's first block, 10126, to be dated in 9784..10126), parked 797
struck and 1456 filed, pins and `WIDENINGS` re-pinned, run624 in
`docs/RUNS.md`.

**Pace after six**: 57, 49, 41, 58, 38 and 58 minutes, each to a
pushed landing; six landings in **5 h 02 m**, four on a full green
gate and two on combined validation. French East Indies 8182 → 10131
(+1,949).

## Landing 7 — item 1455, French East Indies 10131 → 10802

Commit `ed43fd38` at 08:27Z, **71 minutes after landing 6's push** —
the longest interval so far, on five captures (run625–629); the gate's
record `bebcf5ab` at 08:38Z: **exit 0, six of six steps**, fixed 13,
rondata 717, sim 1330, fixture audit 2675 requests and none missing,
memcap peak 15760 MiB. Pushed: `origin/worktree-seq-opus` is
`bebcf5ab`, verified by fetch; the done line reached this session at
08:39Z, the short form again with every field. **82 minutes from
landing 6's push to landing 7's.**

**Diff against the journal** (`docs/journal/2026-10-04-item-1455.md`,
95 lines; COSTS and MOVEMENT each a new section): they agree. Two
causes, found in series, each dated by its own capture. The booked
parting (food 3 high, a second Hoplite) went back to one event on
block 7783 — found by a 600-frame buckets-only capture, `LEADERS=2`,
19 MB (run626), which is the cheap shape the queue's rules ask for —
and run627's widening named it: the three Citizens queued at the
capital cost 52/53/54 there and 51/52/53 here, because `get_cost`'s
unit ramp counts the Militia line into a Citizen's count and the
original had a Militia queued. With that built, `1/58` still parted;
run628 (positions only) dated it to a collision with a Supply Wagon
born at speed 30 there and 25 here — `Unit::update_speed`'s trainer
arm, a French unit of the Siege Factory line a fifth faster. Each is
its own mutation: no Militia ramp → 10182, no French move → 10131.

To carry to the pass:

- **The second landing to move other games**: 24 widening pins fell,
  including Great Lakes (first pair) runs 163, 174, 243 and one
  standing row on each of ten Great Lakes pins 178–226, diffed before
  and after — keys leave, **none arrive**. The harness's own floor
  comments carry the item's line in the established form. No floor
  moved; sim 1328 of 1328.
- **A new sim field is derived, not kept**: `scholar_militia`
  (`LeaderData +0x9f0`, `track_unit_type`'s count) is computed from the
  live militia whose former type is a scholar rather than carried as
  state. Stated in the code; reading-only, and the journal says so.
- **Two tuning slots** (`FRENCH_SIEGE_MOVE` 20, `VERSAILLES_UNITS_MOVE`
  25), `ron_slots` 347 → 349; `wonder::VERSAILLES` added. The
  Versailles term is read, not exercised.
- **Review debt, stated**: the Scholar's term, `scholar_militia` and
  Versailles' speed are a reading alone. No blind reading.
- **A slip owned, and the right one**: "I started the suites once with
  a shell `&` instead of a background task, so no notice would have
  come. I killed that run and restarted it." The arm knows which wait
  the harness can see.

Bookkeeping done by the arm: queue (headline, opener, scoreboard
`w10802`, 1455 deleted, 1457 booked by frame — a group the crate holds
on run629's first block that the original's pool does not, to be dated
in 10139..10796), parked amended in place (a `caras` note under an
existing row; the French move struck from a "not built" line), 24 pins
and `WIDENINGS` re-pinned, run625–629 in `docs/RUNS.md` (626 and 628 as
evidence only, said so). **1456 was booked and 1457 follows it**: the
arm skipped no number and reused none.

**Pace after seven**: 57, 49, 41, 58, 38, 58 and 82 minutes, each to
a pushed landing; seven landings in **6 h 24 m**, five on a full green
gate and two on combined validation. French East Indies 8182 → 10802
(+2,620); the estimate's twelve hours holds at this pace with three to
go.

## Landing 8 — item 1457, French East Indies 10802 → 10802 (no score moved)

Commit `bea585ca` at 09:50Z, **72 minutes after landing 7's push**;
the gate's record `2234a796` at 10:01Z: **exit 0, six of six steps**,
fixed 13, rondata 718, sim 1332, fixture audit 2679 requests and none
missing, memcap peak 15961 MiB. Pushed: `origin/worktree-seq-opus` is
`2234a796`, verified by fetch; the done line reached this session at
10:02Z and said "no score moved" in its first clause. **83 minutes
from landing 7's push to landing 8's.** One capture (run630), the
parting's own window with the leader probe, booked with its killer
first.

**Diff against the journal** (`docs/journal/2026-10-04-item-1457.md`,
82 lines; CITIES §6.5.3, GROUPS §36): they agree, and the landing's
first line says what it is — **"this landing moved no score"** — which
is the honest shape the agreement asks for. The booked parting (a
squad slot the crate held and the original's pool did not) came apart
into three rows on block 10766, each read from the decompile and each
taking its own rows off run630: a member's `come_out` runs the same
tail as the captain's (a computer's unit gets `close_orders`,
`clear_partial_path`, `update_action`, so its `orders_x/y` are its new
place); a stack group's point is `Group::clear`'s (0, 0), not the
record default's (−1, −1); and `equals_group` normalizes the last
pushed slot on every push — a thing the seam table had said was not
modelled, now struck in place. The word did not move; the parting under
it moved from 10797 to 10766 and is booked again as 1458 on the same
frame with the new first row named.

To carry to the pass:

- **The largest re-pin of the arm**: 85 widening tests across every
  game, 110 pins in six files, **~440 keys leave and four arrive** (slot
  speeds on golden chapter forty and run421, parked 1459 with the
  reading of why). The deletions in `second.rs` are lists of expected
  parting rows shrinking, not assertions removed — 36 `assert`/`pin`
  lines out, 39 in, across `rondata`. No floor moved; sim 1330 of 1330.
- **A re-pin tool failure, caught and owned**: the first script put a
  comment beside the wrong pin because its own replacements had shifted
  the line numbers; the arm restored the files and re-ran in two passes
  (values by position, comments by description). The same class of
  slip as landing 7's; the pass should ask whether the re-pin belongs
  in `tools/` now — a probe shape reached for a third time.
- **Review debt, stated**: the human and plane arms of the come-out
  tail and `get_num`'s normalize arm are a reading alone. No blind
  reading.
- **A no-score landing that was worth landing**: three mechanisms the
  decompile names, each with a unit test and a mutation, and the
  widening residue across the corpus down by a tenth. The queue rule
  "a session that moved no score says so in the handoff" is met in the
  queue's own headline block.

Bookkeeping done by the arm: queue (headline says 1457 moved no score,
opener, scoreboard unchanged at `w10802`, 1457 deleted, 1458 booked by
frame with the new first parting), parked 1459 filed, 110 pins
re-pinned, run630 in `docs/RUNS.md`, the seam table amended in place.

**Pace after eight**: 57, 49, 41, 58, 38, 58, 82 and 83 minutes, each
to a pushed landing — the last two slower than the first six, on items
whose trail is longer and whose re-pins are wider. Eight landings in
**7 h 47 m**, six on a full green gate and two on combined validation;
French East Indies 8182 → 10802 (+2,620); two to go on the arm's own
stop rule.

## Landing 9 — item 1458, French East Indies 10802 → 11582

Commit `abea5824` at 10:42Z, **40 minutes after landing 8's push**;
the gate's record `07465ac1` at 10:53Z: **exit 0, six of six steps**,
fixed 13, rondata 719, sim 1333, fixture audit 2696 requests and none
missing, memcap peak 15711 MiB. Pushed: `origin/worktree-seq-opus` is
`07465ac1`, verified by fetch; the done line reached this session at
10:54Z. **52 minutes from landing 8's push to landing 9's.** One
capture (run631), the new word's widening with the leader probe; the
cause needed none — run630, the previous item's capture, still held
both partings under the word.

**Diff against the journal** (`docs/journal/2026-10-04-item-1458.md`,
75 lines; GROUPS §37): they agree. Two rows, each read from the
decompile and each its own mutation. `path_recursion` on a newborn
captain: nothing searched for it, but its object number had belonged
to a dead unit, and `UnitData::UnitData` zeroes the field where
`Unit::init` never writes it — so a recycled slot keeps its last
occupant's count, and `add_unit` now carries it. And the member `1/81`:
§6.6 step 4's **different-region** arm, which the arm had set aside on
the previous item as a candidate (a computer's unit skips the
same-region arm) and came back to — the slot's `get_tregion` is the
coastal-water `region2` read, the member is re-placed around slot 0's
point by the *type's* sweep (`not_o`/`not_who` −1), and the table is
written back so §6.7's path reads the same offset. Asked as the unit,
the sweep found a spot 144 east; asked as the type, as the listing
does, slot 0's point. run630's widening fell 167 → 161 → 155 → 101,
one step per row.

To carry to the pass:

- **64 widening pins moved; 454 keys leave and 5 arrive** (run99's
  first-block standing rows, two families that stand elsewhere already;
  named, not parked — the pass should ask whether they should be).
  Golden chapters 43 and 46 each lose one `WANT` line
  (`path_recursion` on `0/10`), the two deletions in `golden.rs`. No
  floor moved.
- **A field that survives its slot.** The journal's last line is the
  right open question: "No other `UnitData` field was surveyed for the
  same survival." That survey is a reading on `UnitData::UnitData`
  against `Unit::init`, cheap and bounded, and a candidate item.
- **One mechanism has no unit test** — the region arm's check is
  run630's pin and the word; the arm says so.
- **Review debt, stated**: step 4's barge and passenger footprints and
  its same-region arm are a reading alone (and `SEAM`ed in the code).
  No blind reading.
- **The re-pin script is now a recurring cost**: refused eight pins on
  this item, mis-anchored comments on the last, lost to the formatter
  on the two before. Four landings running; this is the "third time"
  rule and then some, and the pass should book the tool.

Bookkeeping done by the arm: queue (headline, opener, scoreboard
`w11582`, 1458 deleted, 1460 booked by frame — the original's make
list headed on 11579 by an entry we lack, t=575 at 9999999), 64 pins
and `WIDENINGS` re-pinned, two golden `WANT` lines removed, run631 in
`docs/RUNS.md`, the seam table amended. **1459 is parked 1459**
(landing 8); the arm's numbering is still clean.

**Pace after nine**: 57, 49, 41, 58, 38, 58, 82, 83 and 52 minutes,
each to a pushed landing; nine landings in **8 h 39 m**, seven on a
full green gate and two on combined validation; French East Indies
8182 → 11582 (+3,400). One to go on the arm's own stop rule, and the
estimate's twelve hours holds with room.

## Landing 10 — item 1460, French East Indies 11582 → 12794

Commit `a7551ad8` at 11:45Z, **51 minutes after landing 9's push**; the
full gate on it started 11:45Z and was running when this row was
written. Three captures (run632–634) and **a packet** (run633, the
lab's first use by the arm): `step4.py` ran `Leader::research_techs`
on the original's own state at 11578.

**Diff against the journal** (`docs/journal/2026-10-04-item-1460.md`,
97 lines; COSTS, "A French siege unit costs less"): they agree. Three
hypotheses with killers, two killed by their captures: H6 (a purchase
in 10810..11382) by run632, both sides offering Conscription at the
same value to 11382; H7 (an unprinted `tech_value` factor) by the
packet, which showed Conscription reaching the crate's own weights,
category and shortage values and never the original's 3491100 — so
the difference lay downstream, in affordability, 15 timber and 15
metal short on run631's standing rows. H8 dated the shortfall to the
Trebuchet's research queued at 94/94 here and 79/79 there, and the
listing has it: `get_cost`'s pre-ramp nation tail, `FRENCH_SIEGE_COST`
15 % off a French unit of the Siege Factory line, on the train arm and
the research arm alike. The arithmetic is shown both ways (70 → 59 →
88 → 79 with the arm; 70 → 105 → 94 without), and the arm's own slip —
it first worked the price through the train arm and the number did not
reproduce — was caught by a modifiers print before anything landed.
`trainer_where` is now shared with landing 7's speed arm.

To carry to the pass:

- **The packet earned its place**: a value question — does the
  original's `research_techs` reach 3491100 from these inputs — was
  answered on the original's state rather than by a reading or a
  detail capture, exactly as `CLAUDE.md`'s lab paragraph asks. First
  use of `step4.py` in the arm's ten; the rule held.
- **96 keys leave run631 and none arrive**; only that pin moved; no
  floor moved; sim 1334 of 1334. The widest single move of the arm,
  **1,212 frames**.
- **One tuning slot** (`FRENCH_SIEGE_COST` 15), `ron_slots` 349 → 350.
  `FRENCH_SPECIAL_COST` ships as 0 and is not modelled; the other
  nations' arms of the same tail are read and not carried — the review
  debt line says so. No blind reading.
- **The arm stopped itself.** The queue's handoff reads "the arm has
  stopped at its tenth landing" and the opener hands 1461 to "the next
  session … as the steering session decides" — the stop rule met as
  written, the direction question left where it belongs.

Bookkeeping done by the arm: queue (headline, opener rewritten for the
stop, scoreboard `w12794`, 1460 deleted, 1461 booked by frame —
run634 parted by its first block, 12789, on `num_units[270]/[271]`,
a dozen units' orders 24 short, three second guys elsewhere), pins and
`WIDENINGS` re-pinned, run632–634 in `docs/RUNS.md` with the packet.

**Pace after ten landing commits**: 57, 49, 41, 58, 38, 58, 82, 83,
52 and 51 + gate minutes. French East Indies **8182 → 12794 (+4,612)
in 9 h 31 m** to the tenth landing commit.

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
- **Two of six full gates were red on a paperwork pin alone** — the
  coverage pin on a new window's unread key (landing 4) and the blind
  list on a newly cited never-executed function (landing 6) — each
  ~11 minutes of gate to learn what `coverage::` and `blind::` would
  have said in seconds. `tools/guard.sh` runs the paperwork guards but
  not those two release-profile tests. For the pass: whether the
  reflex before a landing commit should run `coverage::` and `blind::`
  in release on the item's windows, so the full gate is red only on
  what it alone can see.
- **The arm's evidence directory is denied to this session's reads**
  (`~/ron-data/lab-experiments/2026-10-03-item-1446-opus/`, the
  classifier's "Modify Shared Resources" on an `ls` and a `tail`). The
  gate verdict is therefore read from the arm's follow-up commit and
  its done line, not from the log.
