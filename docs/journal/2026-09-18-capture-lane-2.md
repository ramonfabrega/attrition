# 2026-09-18 — the capture lane, second run: both frontiers extended, and a watchdog that killed a finished run

Both maps' words had outrun the disk. Great Lakes still had 215 frames of
runway inside run97, but the last three items moved +322, +178 and +149, so
the map was one item from having none. East Indies had none at all: run98
stopped at 8789 and the word was 9711, so **eight items had advanced on the
draw stream alone**, and a draw stream can agree on a wrong destination for a
long time.

Two windows, extending each map's existing coverage rather than duplicating
it. Both landed whole, on the first attempt, with every check passing.

## The tranche

| run | map | window | bytes | blocks | identity | overlap |
|---|---|---|---|---|---|---|
| 99 | East Indies | `[8780, 10400)` | 934,558,773 | **1,620**, 8780..10399 | run54: 0 / 10,411 | run98: **9**, 0 differing |
| 100 | Great Lakes | `[9340, 10900)` | 770,701,478 | **1,560**, 9340..10899 | run53: 0 / 10,911 | run97: **10**, 0 differing |

Values at the frontier, which is what the lane exists for:

- **East Indies `to_x` on 9711 — the word itself — 34 records**, with 24 on
  10000 and 33 on 10300.
- **Great Lakes `to_x` on 9340 (148), 10000 (82) and 10800 (251).** The far
  end of the runway is dumped too, so an item landing at 10,800 already has a
  value diff waiting.

Both maps read back `MAP_STYLE` and seed from the dump's own `GAME INFO`
rather than from the write that set them — 18 and 14, seed 12345 on both.

## The overlap is 9 on one map and 10 on the other, and that is the finding

`samegame.py` drops **each file's highest block** before comparing, because a
run ends by quitting and the block the quit interrupts is written short. The
two archives being extended do not end the same way:

- run97's highest block is its **9361 `!quit` block**, so the drop takes 9361
  and leaves 9349 in the compared set. A window opening at 9340 overlaps in
  **10** blocks.
- run98's highest block is **8789 — the one its death cut mid-record**. There
  is no quit block above it, so the drop takes 8789 itself and the compared
  set stops at **8788**. A window opening at 8780 overlaps in **9**.

Assuming the two were equal would have produced a `check:` asserting 10 on
East Indies, which would have failed on a capture that was perfectly correct —
and the temptation then is to edit the number until it passes, which is how an
overlap assertion stops meaning anything. Both were re-derived from the
archives before the windows were written. This is the predecessor's
exclusive-`HI` trap one step along: the same arithmetic, a different edge.

## The failure was mine, and it killed a run that had already finished

The lane's three hard limits — free disk below 15 GiB, a run's output past
6 GiB, one run past 90 minutes — were enforced by a watchdog rather than
watched, and all three branches were made to fire and kill a live process
before the first capture started. That part held.

A **fourth** guard did not. Last night's lane lost eighty minutes to a hang
whose signature is "the game is alive at high CPU and no start dump was ever
written", so this one added: if `gamelog.txt` is under 1 MB at t+8m, kill. It
reads the **live** log's size — and `longtrace.sh`'s last act is to `mv` that
file onto the archive name. **So a finished run reads zero bytes, exactly like
a run whose lobby clicks never landed.** It fired 43 minutes into a complete
run99 and took `runqueue` down during its check phase.

Nothing was lost: the archive was already renamed, the INIs already restored,
and the five checks ran by hand and all passed. But the shape is worth naming,
because it is the general case of which last night's `samegame.py` trap was an
instance — **a signal that is absent for two opposite reasons cannot be a
guard**, and the way to find that out is to ask what the signal reads at every
phase, not only at the one the guard was written for.

Two fixes, both in the driver with the incident named in the comment:

- The guard **latches**. Once a start dump has been seen it is off for good.
- The driver distinguishes the **two phases of a queue**. A capture owns the
  screen; its checks own a core and run `rngcmp` and `samegame` over a gigabyte
  for minutes. Once the archive exists and no game process survives, limit
  enforcement stops and the checks get a bounded grace. run100 exercised this —
  "capture done at 36 min; checks running", then a clean queue summary — and
  its checks were run by the queue itself rather than by hand.

## The first invocation refused, and it was not a permission problem

`runqueue.sh` called directly from this session's process tree died in under a
second: "Screen Recording is off — `screencapture` wrote nothing." That is the
`viadriver.sh` path, not a revoked grant — the capture lane runs through
`~/bin/RonDriver.app`, which is the fixed path holding all three permissions
across Claude Code updates. The tool's own error names the fix. Relaunched
correctly and it ran.

Worth writing down only because it costs a minute when recognised and an hour
when mistaken for the TCC failure it is worded like.

## Sizing, and a rate that is flattening

Both windows were sized from the siblings' own per-block rates, net of the
start dump, with the roster's growth extrapolated forward. Both came in under:

| map | predicted | actual | per block |
|---|---|---|---|
| East Indies | 1.05 GB at 650 KB/block | **0.93 GB** | 570 KB |
| Great Lakes | 0.81 GB at 520 KB/block | **0.77 GB** | 485 KB |

The linear extrapolation from run98 and run97 overshoots — East Indies was
551 KB a block at 7880..8789 and is 570 at 8780..10399, a far slower climb
than the gap between the two maps' rates would suggest. A future window on
either map can be sized at its predecessor's measured rate plus about 5%,
rather than the 20% assumed here.

Neither run came near the 6 GiB abort, and free disk moved 32 → **31.13 GiB**:
**1.62 GiB** for two captures and their traces.

## Timing and the machine

run99 took ~41 minutes for 1,620 blocks, run100 ~35 for 1,560 — both inside
`poll_max: 240` (80 minutes), which was chosen to give way *before* the lane's
own 90-minute stop rather than after it, so a slow run truncates loudly in the
tool rather than being killed from outside. 105 polls of 240 on run99.

The box was quiet throughout: load 2.5-3.1, frontmost `iTerm2` before each
launch and at every check. No contention, and no repeat of run98's death — one
more data point behind run97's verdict that the truncation was a loaded
machine rather than the window size.

## The courtesy, and its limit

`MUSIC_VOL`, `SOUND_VOL` and `TAUNT_VOL` were set to 0 in `Player.dat` for the
run, on the same backup-and-restore path the profile's `MAP_STYLE` already
takes. **No tooling changed** — muting as a feature of
`unattended_capture.py` stays parked as 343 and belongs to the steering pass.

East Indies ran **first** and Great Lakes second, so the profile's `MAP_STYLE`
would end back at the 14 it was found at. It did, and the restore was verified
as the whole file rather than the three fields: `Player.dat` is byte-identical
to its backup by sha256. No stray `gamelog.txt`, no surviving game, wine or
driver process.
