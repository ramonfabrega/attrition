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
