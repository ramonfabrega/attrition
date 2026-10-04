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
