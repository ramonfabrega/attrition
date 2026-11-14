# The pass of 2026-11-14: the loop is bound by test CPU, a lane's gate reads the queue's mark, the diplomacy's second reading is adjudicated, and the documents are compacted

*2026-11-14, from 04:20 −05:00, the `steer` row on Fable 5.2. `commander` handed
over at `e8b6a8359` after the tranche of `docs/DECISIONS.md` entry 132 (28
landings, 15:18 to about 04:40). The charter: Fable's docs-compaction proposal
(`2026-11-14-docs-compaction.md`), the blind second reading of the diplomacy,
the queue (item 4159's residue, 4224, 4230's draft, the render lane's next), and
three measurements: what a lane gate and a scoped booking gate cost under entry
132, how many booking reds fell on tests a lane no longer runs, and the week's
pace by worker count. Decisions in entry 133. Three scouts, an adjudicator and
four builders ran, each a `lean` agent on Opus 5.5 (the model each report names
for itself); the scouts' reports are under `~/Library/Caches/attrition/pass-1114/`.
The pass before it is `2026-11-13-fable-pass-supply-2.md`.*

## For Ramon

- **More workers did not land more.** 4.7 workers on average landed 2.24 an
  hour at 41.1 USD an hour; 3.6 landed 2.20 at 35.2. A booking gate ran 73 % of
  every hour, and a lane waits 68 of its 90 minutes, 34 of them on its own gate.
  The twenty slots of the host budget bind before the cores do.
- **Your question, more lanes at quiet times:** no hour was quiet. A fifth or
  sixth worker adds landings only where it reserves no slots: a reading, a docs
  lane, a capture. The pass recommends four gating workers stay, and the extra
  seats go to those kinds. The count is yours.
- **The "8 points an hour" was not attrition.** At 01:00 to 03:00 the week moved
  7 and 7 points; 61 % of that was one `lore` row and its 46 subagents. Over the
  tranche attrition spent 475 USD and other projects' rows 466. A point costs
  19.2 USD all-in, not the 13.5 `lore limits` prints (it leaves subagents out).
  At that mix next week's 99 % arrives in 23 to 27 hours, and Fable's own week
  in about 23, which stops this row.
- **A landing costs** 10.1 USD on an Opus lane, 24.1 on a Fable lane, 21.8 from
  an in-session builder. Item 4224's remainder is mechanical; the pass's take is
  Opus. You asked that an Opus driver not constrain a Fable lane: the brief now
  takes its gate from the queue's mark, not from the driver's note.
- **The compactor.** Its return was true first reads, not bytes: about 3 % of a
  lane's bill at most. What it found by reading (an opener 21 commits stale,
  eight stale counts in one bullet, a brief that stopped lanes at 300 k) is
  mostly checkable by script. The pass took the proposal nearly whole and
  recommends it run at a drain as an Opus lane with the audit's scripts, not as
  a standing Fable row.
- **D9, yours:** `CLAUDE.md` calls phase 3 done on lockstep by draws; entry 132
  §10 made parity by the checksum the rules' finish line, and it reads 0 of 1
  games. A clause the pass would add to phase 3: "counted in draws; parity by
  the twelve state checksums is the open count (`docs/TRACKS.md`)". Also yours
  in that file: S10 (the recipe is ORACLE's fourth section, not its last) and
  S17 ("before a commit" against "commits before it gates").
- **Decided, yours to reverse:** the two never-empty headline slots of entry 41
  are retired (D5); a briefed lane reads the handoff to its `Scoreboard:` line
  (D7); the section ceiling stays at 16,000 with the pressure taken off (D1).
- **28 merged branches stand on origin** of 43; the delete is yours (entry 129 §9).
- The pass moved no score. One lane (4230) was re-armed and is out.

## What was measured

| what | measured |
|---|---|
| the booking gates (`gates.py ~/Library/Caches/attrition/gates --since gate-pass-1113d.log`) | 19 gates for 28 landings: 1 whole, 18 scoped and every one widened, 3 red, 528.0 minutes; admission 45,354 s summed, the longest wait 788 s; six ended past the cores |
| a scoped booking gate (scout: the logs' receipts) | total median 28.4 minutes (16.6 to 43.4), release median 17.9 (12.3 to 29.3). Release before item 4216: 25.5 to 29.3; from 4216 to 4220: median 23.0; from gate-4220 on: median 16.2, nine of ten at 12.3 to 17.4. No landing moved it alone: 4216 halved the leases (about 2,395 → 1,281) and release fell to 23; after 22:41 the suite's test CPU-seconds held (median 4,326 → 4,191) and the host's load at a gate's start fell 31.4 → 8.1. Release tracks admission wait (r = 0.83) |
| the whole gate on a quiet host (`gate-drain-1114.log`) | release 23.6 minutes at a load of 4 to 7: `Suite: 1268 processes for 2397 tests: 145 groups hold 1274, 1123 ran alone; 9474 test CPU-seconds`; admission 7.6 s. At eight threads 9,474 CPU-seconds is 19.7 minutes: the gate is CPU, not waiting |
| a lane's own gate (scout: 66 report directories under `$TMPDIR/attrition-gate-*`, 34 that reached release) | release median 15.8 minutes (2.8 to 26.0), total 21.2; 808.6 minutes summed. Start lanes on their filters with the sim crate: 4,087 to 4,251 tests, release median 15.4. Render and input lanes: 299 to 543 tests, median 11.6. **Six lanes skipped nothing** (4218, 4222, 4226, 4204's second, 4228, 4229: 6,142 to 6,211 tests, median 18.9 over twelve gates): `tools/brief.py` left `--no-widen` off every `residue` brief. The three `--whole` structure gates did not wait on the lock |
| the booking reds against what the lane ran (scout: each lane gate's `release-tests.log`) | **2 of 3 on tests the landing lane did not run** (4208's three `free::` tests, 4204's eight `widened_whole` and run-up tests). Only 4204's was let through by entry 132 §6: the old one-second rule would have run three of its eight and skipped all of 4208's (1.27 to 79 s). It cost 35.2 booking minutes and a 20.9-minute lane gate. The third was the clock test at a load of 23 on 16 cores |
| the trade of §6 (scout) | no lane's release step fell measurably; by the timings file the skip is worth 263 to 364 s a lane gate, 21 to 133 lane-minutes over the tranche, in parallel, against 35.2 serial booking minutes |
| the pace (scout: `~/.lore/limits.jsonl`, `lore.db` with subagent transcripts, `tranche.py`) | the week 38 % → 86 % over the tranche, 941.6 USD at list: 19.2 a point. Attrition 475: Opus lanes 201.7, builders 137.1, Fable lanes 91.3, commander 34.6, steer 10.6. Workers never passed six, and that for half an hour; 3.8 on average at 02:00 to 03:00. The driver worked 96.7 of 783 minutes, peak context 549 k |
| the suite re-timed (`suite_timings.py --stale-only`) | 40 stale tests timed: 2,397 tests, 4,910 s one at a time. The longest: the coverage census 216 s, the checksum frames' walk 144, every lobby's page 92, the ages' draw list 91 |
| the disk (`df -g ~`) | 153 GB at the start (the pass before left 228), 196 after `afsctool` over both archives and `cargo clean --profile dev` (86,464 files, 42.5 GiB) |
| the rubric (`rubric.py --record`) | unchanged from the pass before: the tranche moved no rubric pin (`Rubric:` on the queue) |
| the window (`lore limits`) | the week 86 % at 03:50, 88 % at 04:50; Fable's 83 % and 84 %; both close 06:00 −05:00; the stop 99 % |

## What the pass looked at

The client's still from `e8b6a8359` at the three cameras, beside each photograph
(ours left), under `~/ron-data/lab-experiments/2026-11-14-pass/`; theirs for
run1564 is the sRGB-matched grab the row reads (item 4223). By eye.

- **The land, on all three.** Yellow-green as theirs, where the pass before saw
  orange-brown: item 4212's light and tint.
- **run1564, the lake. Still the worst picture.** Ours is an even saturated
  cyan with a soft pale rim. Theirs is darker, mottled water with a white surf
  line, whitecaps and the fish herd's dark blot. `RUN1564_3279_GROUND_STILL`
  reads ocean 412 of 437 on it. The 25 parted tiles, listed by the scout: four
  under the herd's blot, nine in the centre where theirs has small figures, six
  in the whitecap field, six scattered. The lit floor's composite (parked 4227c)
  reads 14 lower on hue. A hue row cannot see texture; item 4232 opens with one
  that can.
- **run1559b and run1570, the city.** Theirs burns with black plumes on three
  buildings and shows the cone of light; ours has small flame sprites, no smoke,
  no cone. No row scores a fire (item 4233). The mountain is paler than theirs.
- Against the pass before's stills: 1,715,270, 2,039,624 and 1,891,230 pixels
  changed of 2,073,600.

## Rulings

- **The second reading is adjudicated** (entry 133 §1;
  `2026-11-14-leaders-diplomacy-adjudication.md`). Five points by the pass on
  the listing, 47 claims by a third reader: 38 agree, 5 true where the document
  was silent, 3 corrections, no marker open. No leaf and no sim value moved.
  The proposal's draw at `6c0980` is parked in `docs/LEADERS.md` §7.6.
- **A lane's gate widens by the queue's mark** (§2), built.
- **Item 4224 is the gate's CPU-seconds** (§3): the remainder of "one walk a
  game" is kept, scored on a scoped gate's `Suite:` line, and opens with the
  table by family. 4220 bought 3.7 %.
- **The third lane follows parity's first partings** (§4): 4230, then 4231 on
  the groups line's 2746.
- **The render lane takes the lake's water, then the fire's smoke** (§5), each
  opening with a row that can see what the eye sees.
- **The docs compaction is taken** (§6), below.
- **The opener names no item between passes; the two never-empty slots are
  retired** (§7).
- **Loop 4051 and 4055 are closed**: `tools/lanewait.py`'s header is the
  authority on the waiter, and the block that retold it is retired whole.
- **`TEST_PERFORMANCE.md`'s "independent review remains owed"** is ruled once:
  the trial's verdict (entry 76) and a month of gates stand for it.

## The docs compaction, row by row

| row | ruling |
|---|---|
| G1 | taken: 272 numbered `###` parts in 46 sections of 16 files are `##` sections; seven `OVER` rows deleted, one added (`GROUPS.md` §6.4, 16,448 B) |
| R1 | taken: a scoreboard line whose every pinned row is whole may read `every word whole · <rows>`; the ten lines went from 5,738 B to 355. `Long captures:` gained pinned rows and a length check it never had |
| R2, R3 | taken in the rewrite: the handoff 15,983 → 8,993 B. The pilot's rules cite `docs/LOOP.md`; the renderer's bullet states today's pins without their histories |
| R0, D7 | a briefed lane reads the handoff to its `Scoreboard:` line |
| R4 to R10, R13 | taken: `LOOP.md` 37,586 → 32,260 B, `METHOD.md` 22,127 → 18,514, `TRACKS.md` 11,817 → 8,857; every moved clause is verbatim in `2026-11-09-loop-retired-clauses.md` (15.2 KB appended) |
| S1, S16 | the index is two sections; six statuses corrected (59, 71, 72, 100, 116, 121). Left: 57, 64, 99 and twelve rows whose entries carry an "Amended" note |
| S2, S4 to S8, S11, S12, S19, S20 to S31 | taken. S24 found four stale counts; the scout found eight |
| S9 | ruled (entry 133 §7) |
| S13, S14 | taken: twelve dead section cites mended, none left unresolved; four remain in `GOLDEN.md` and `JOURNAL.md`, which the builder was not to touch |
| S15 | none to lower after G1 |
| S29, G13 | taken and widened to §4, §14, §15, §17: 38 `file:line` cites into the crate became names; 43 remain in `LEADERS.md` |
| G8, G11 | a "What stands" head on `ORACLE.md` and `TEST_PERFORMANCE.md` |
| S3, D5 | decided: the never-empty slots are retired |
| S10, S17, S21's table row, R11, D9 | `CLAUDE.md`: Ramon's |
| D2, D4, D6, G2 to G7, G9 | left as they are |
| D3, G3 | not done: an index by topic is a pass of its own |
| D8 | not built: a pass-time listing of the parked rows nothing else names (46 in five days) |
| R12 | the memory index: not touched |

## The queue's items

| item | kind, mark | the score | first act | it stops and says so when |
|---|---|---|---|---|
| 4159 the residue | the chain's builders | `LEADERS_HELD_OUT` 27 | parked 4176a–c, 4177b, 4177c, 4159ra, 4207b, in that order | as each row says |
| 4230 parity's units | rules, scoped | units' first parting frame, 3166 (3596 on the draft) | out: merge item 4229, re-measure, gate | the parting is the caravan's copy alone |
| 4231 parity's groups | rules, scoped | groups' first parting frame, 2746 | a packet at 2746, the line's ranges against the hook's | the first parted range is a member's own |
| 4232 the lake reads as water | render, scoped | a texture row on the ocean tiles, seen red first; sandy 442 of 772 | the row | the composite's row does not rise with the lit floor |
| 4233 the fire's smoke | render, scoped | a row of the plume's dark pixels, seen at 0 | the row | the plume's draws are in no trace on disk |
| 4224 the gate's CPU-seconds | structure, whole | a scoped gate's test CPU-seconds, about 4,200 | the CPU-seconds by family from one gate's per-test file | no family holds a fifth |

## Built

- **The widening by mark** (`tools/brief.py`, `queue_mark`; the pass). A test
  first, red on the old composer: a whole-marked item on a `chapter` brief
  carried `--no-widen`.
- **The waiter's header** takes the `DEEP` answer and the `Left to do` marker,
  which lived only in the block that left `docs/LOOP.md`.
- **G1** (builder, `d168d63e3`), **R1** (builder, `567d12e46`; red first on three
  tests, then on the real queue with a row count of 81, a missing close and a
  word lowered by one), **the loop's files** (builder, `f4eb770e7`), **the
  specifications** (builder, `cb4c4d42c`).
- **The adjudication's amendments**: twelve edits to `docs/LEADERS.md` §7.2,
  §7.5, §7.6 and §8.2, three seam comments in `crates/sim/src/leaders/`.

## Not done, and said

- **No guard reads a count in the handoff's prose against its pin.** The
  renderer's bullet now states pins without histories, and a pass checks them;
  eight had drifted in two days. A reader keyed on a backticked constant and a
  count is the guard this wants.
- **§9 of `docs/LEADERS.md`, the taunts, is outside the second reading.** It
  rests on item 4152's pair and adjudication; the adjudication record lists
  what in §7 to §9 is still on one reading.
- **Lane-gate CPU is not measurable**: those logs are not kept past the
  report directory, and no sampler reads the host between gates.
- **The stub pool** (D6) stays on `AI.md` and `GOLDEN.md`, where no lane lands.
- **`lore`'s repository copy has no Fable 5.2 rate** (the installed build does);
  told here, not fixed: it is another project's.

## The gate

Whole, on the commit that carries this record, by `tools/gate_run.sh`:
`~/Library/Caches/attrition/gates/gate-pass-1114.log`. The record is written
before the gate runs; its verdict is that log's last line, and neither the push
nor the handover happens on anything but `gate exit 0`.
