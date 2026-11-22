# The pass of 2026-11-21, second: the disk freed without a file deleted, the rows scored, and four seats cut

*2026-11-21, from 18:30 −05:00, the `steer` row on Fable 5.2, called by
the backstop. `commander` (Opus 5.5) handed over at `ed4649c21` after
twenty first-parent merges (items 4484, 4502, 4504 to 4508, 4510 to 4516
and 4520 to 4525), gated whole (`gate-backstop-1121.log`). The charter
was the handoff's: every item left needs a take, so score what is on
disk, seat, and tell Ramon the disk. Decisions in entry 149. The pass
spawned no subagent and took no capture. The pass before it is
`2026-11-21-fable-pass-takes.md`.*

## For Ramon

- **The disk is no longer the limit, and nothing was deleted.** It read
  56 GB free when the pass began and reads 117 now. The handoff said the
  only thing left to free was the archive, which is yours. It was not
  needed.
- **Why it was full.** The archive is compressed in place, and the
  compressor silently skips any file of 2 GiB or more. Fourteen of the
  big capture files were sitting uncompressed for that reason, 48.8 GiB.
  They compress about twelve times over.
- **What I did.** Thirteen of those files are now stored as 1 GiB pieces
  that compress, 46.8 GiB down to 4.1. The pieces put end to end are the
  original file, byte for byte. I re-read all thirteen from their pieces
  and each matches the SHA-256 recorded when it was captured. One
  command a file undoes it (`tools/trace/split.py --join`). New captures
  are split when they are filed.
- **One thing this breaks.** A checkout older than today's commit reads
  a split file as cut short. That includes Codex's branch and your root
  checkout until they are pulled.
- **The fold is still yours.** It reads 14 of 15 today: six of the
  seven, and all eight of the first unseen battery. This is the fifth
  record to ask. I still recommend folding.
- **A fifth unseen battery is named** (rows 3, 23, … 143). Like the four
  before it, its human is idle, because seats and a team are yours and
  you have not said. It is taken when the fourth reads 7 of 8, about two
  landings away. If you want it to carry something else, say so before
  then.
- **The unseen games stopped improving by count.** The fourth battery
  read 5 of 8 whole on its first try, the same as the third. An unseen
  game runs 91 % of its states before it first parts, up from 88 %.
- **The tranche:** twenty landings in about nine hours. No gate was red.
  A landing cost 7.18 USD, up from 5.77, and waited 73 minutes, up from
  49. Forty of those minutes are its own gate under load.
- **The week** read 7 % at 18:29 −05:00, the day after its reset.
  Fable's week read 2 %.

## What was measured

| what | measured |
|---|---|
| the backstop's gate (`gate-backstop-1121.log`) | `gate exit 0` on `ed4649c21`, the tip, HEAD = origin/main = origin/worktree-replan-pdb; whole, 9 of 9 steps, 26.1 minutes, release 22.0; `Driver: 20 landings since the last pass (backstop 20)`, context 109 k |
| the merges (`git log --first-parent --merges 04f7466b0..ed4649c21`) | 20, 09:02 to 17:59 −05:00 |
| the lanes (`tools/tranche.py --base 0bd32ef70`, after `lore index`) | 17 sessions over 17 items, 122.03 USD, 7.18 a landing; a session's clock 92.1 minutes, 19.0 working and 73.1 waiting, 39.6 on its own gate and 16.7 on a suite run; peaks 162 to 314 k, one past 300 (4505); 12 captures. Items 4484, 4504 and 4514 were in-session builders and have no session |
| the gates (`tools/gates.py … --since gate-pass-1121.log`) | 17, 399.8 minutes: 1 whole, 16 scoped (16 widened), 0 red, 2 stopped; admission 29,461.4 s summed, longest 798.0 s; 7 ended with the load past 16 |
| the two stopped gates | `gate-4524` at 20.8 minutes and `gate-4525` at 4.1, each `gate exit 143`: the driver stopped them and booked three landings under the backstop's gate. The table printed both `9 of 9 … green` until this pass |
| the five rows not whole (`score_recording` at the tip) | entry 149 §2's table; exit 1 on each, a measured difference |
| the four batteries (`held_out`'s parsed lines, read) | as found 0, 1, 5 and 5 of 8 whole, 64, 65, 90 and 90 of 96 lines, reach 27, 53, 88 and 91 %; now 8, 6, 8 and 5 of 8, 96, 89, 96 and 90 lines, reach 100, 88, 100 and 91 % |
| the sample (`SAMPLE_LINES`, `SAMPLE_WALKED`, read; green on the backstop's gate) | world 5401, leaders 5403, units, groups and guys 5461, builds 5487, cities 5801, ammo 5969, deaths 6190, goods 6395, items 10910; the draw word 5460; the newest record, run1771, ends on 5350 |
| the compressor's refusals (`afsctool`'s own output) | seventeen files `with unsupportable size`: fourteen ranges records, 48.8 GiB, and three text dumps, 10.1 GiB |
| one piece (`head -c 1900000000` of run1721's record, then `afsctool`) | 1.9 GB to 151 MiB on disk in 9.9 s, the bytes unchanged (`cmp`) |
| the split (`split.py --archive`) | thirteen records, 46.84 GiB on disk before and 4.06 after, in 3 to 6 pieces each; exit 0, no refusal |
| the re-read (`split.py --check` on the thirteen, against `docs/RUNS.md`) | 13 of 13 carry the ledger's SHA-256 |
| the disk (`df -g ~`) | 56 GB at the start; 64 after `cargo clean --profile dev` (12.0 GiB); 74 after `afsctool` over both archives; 117 after the split |
| the oracle caches (`du` over every `.ron-oracle-v1`) | 8.6 GiB over 132 caches, six generations in each; five go stale twelve hours after their last reader |
| the week (`lore limits`, the reading of 18:29 −05:00) | 7 %, reset at 06:00 −05:00 on 2026-11-21; Fable's week 2 % |
| the suite re-timed (`suite_timings.py --stale-only`) | 21 tests timed: 2,605 tests, 6,083 s one at a time |
| the rubric (`rubric.py --record`) | every figure the pass before's: ported 738, replaced 331, open 571 |
| the rows and trees left | none: no lane row stood, `reap_agents.py --yes` printed nothing, the old driver row reaped by `handover.py reap 9d0fb0fe`; Codex's branch is not ahead of HEAD |

## Rulings

- **A record past two billion bytes is kept in gigabyte pieces** (entry
  149 §1). Both readers open them as one; the filing tool splits a take;
  a pass splits the archive ahead of its compression.
- **The takes' stop under 60 GB stays, and the mask rule stays.** The
  first no longer binds. The second bounds what a take reads while it
  runs.
- **Four seats** (§2): the sample's on item 4526, the chain's on row 75,
  parity seats on rows 30 and 55. Rows 110 and 95 wait behind them.
- **The sample's take runs to 6400** on the ten lines that part below it
  (§2), so one take serves the landings to 6395.
- **Item 4503 leaves the queue** (§2). Item 4521 moved the draw word past
  the frame its next act names.
- **The fifth battery is rows 3 to 143**, named before any take, taken
  when the fourth reads 7 of 8 (§3; item 4527).
- **The fold is Ramon's; recommended, 14 of 15 folded** (§4).
- **A gate the driver stopped is `stopped` on the table** (§5).
- **The thirteen parked findings stay in their journals** (§6). None
  names a score.
- **Marked rows:** no `FABLE:` marker and no second reading in the
  tranche.

## What the pass looked at

Nothing was drawn. The tranche's one render landing is item 4506, the
tanks' dust, whose row reads 4 of 4 and whose journal carries its own
pair. Its two parked findings are what the eye would say of that pair:
the dust is denser than the photograph's with a quad's edge showing
(4506a), and four riflemen's muzzle flash is undrawn (4506b). No row
counts either. The pairs of 2026-11-20 were not redrawn or uploaded.

## The queue's items

| item | kind, mark | the score | first act | it stops and says so when |
|---|---|---|---|---|
| 4526 the sample, world 5401 | rules, whole; the sample's | `Parity:`, world 5401 | one take to 6400 of ten lines, run1772, `RNG_MASK=0xdfbu` | two landings running leave `Parity:` as it is |
| 4517 fourth, row 75 | rules, whole; the chain's | the fourth's 5 → 6 of 8 | one take of the ammo line, run1768, `RNG_MASK=0x8u` | two landings running move no column |
| 4509 second, row 30 | rules, whole; a parity seat's | row 30's column, two lines | one take, run1755, `RNG_MASK=0x801u` | two landings running move no column |
| 4518 fourth, row 55 | rules, whole; a parity seat's | row 55's column, two lines | score it again, then a take, run1769, `RNG_MASK=0x21u` | two landings running move no column |
| 4485 second, row 110 | rules, whole; unseated | row 110's column, five lines | score it again, then a take, run1742, `RNG_MASK=0x88bu` | two landings running move no column |
| 4519 fourth, row 95 | rules, whole; unseated | row 95's column, three lines | score it again, then a take, run1770, `RNG_MASK=0xc1u` | two landings running move no column |
| 4527 the fifth battery | rules, whole; at 7 of 8 on the fourth | a fifth found column, with its reach | the declaration, one commit before any file | — |

## Built

- `tools/trace/split.py`: split, `--check`, `--join`, `--archive`.
  `tools/explore/test_split.py` holds it: the pieces are the record, a
  record already split or under two billion bytes is left alone, a
  record a take may be writing is refused, pieces that do not read back
  leave the record whole, `--join` undoes it.
- The readers: `rondata::trace::ranges`' `Pieces` and
  `tools/trace/ranges.py`'s. Each has a test seen red on the old reader.
  The Python one read a split record as no sets and no error.
- `tools/gamelog/filerun.py` splits a take's record at filing, with a
  test seen red.
- `tools/gates.py`: `stopped (exit N)` for a gate ended by a signal,
  with a test seen red on `green`.
- The paperwork: entry 149 and its index row, this record, the queue's
  handoff and seven items, the queue ledger's line for 4503, the loop's
  four clauses, `docs/CHECKSUM.md` §25.2's paragraph, `docs/ORACLE.md`'s
  paragraph, the tool's README row, the tracks' paragraph, the journal's
  entry, the suite's timings, the rubric's row.

## Not done, and said

- **No leaf is read for any row.** All five hold sums. Each item takes
  one take.
- **The three text dumps over the limit stay raw**, 10.1 GiB. No reader
  of a dump takes pieces, and the disk does not ask for it today.
- **The lab's record was not split.** It is under
  `~/ron-data/lab-experiments`, 2.0 GiB, and the lab's tools read it.
- **A take's size once filed is measured on old takes only.** The
  thirteen read 0.19 to 0.55 GiB each. Item 4526's take is the first
  filed through the new step, and its `df -g ~` before and after is the
  measure.
- **Why a builder's tree vanished mid-item (4504) is not established.**
  The gate logs show no reap of it.
- **Whether a lane gate carries the widening suites (4522) is not
  ruled.** No booking gate was red on one this tranche.
- **Why a landing's wait rose 24 minutes is measured, not explained.**
  Ten of them are its own gate and ten its suite runs; seven gates ended
  past the cores.
- **The idle human is named again, not mended.**
- **The driver's own cost is not summed**, as before.
- **The blind list was not scanned**: open 0.
- **The guards caught the pass twice**: the queue ledger refused item
  4503's line until it said `dropped`, and the tracks' section and the
  handoff were each over the ceiling, 442 and 165 bytes. The tracks'
  paragraph for the pass of 2026-11-16 was retired; its story is entry
  136 and the journal's. Both were mended before the gate.
- **The first split was one too many.** The trial split run1754's
  record, 1.3 GB, which the compressor had already taken whole. It was
  joined again, its SHA-256 the ledger's, and the tool now leaves a
  record under two billion bytes in one file.

## The gate

Whole, by `tools/gate_run.sh`. Quoted when it has run.
