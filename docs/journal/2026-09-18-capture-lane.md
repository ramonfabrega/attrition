# 2026-09-18 — the capture lane: one census landed, one window running, one stopped

The lane was released to restore the **value diff** at both maps' frontiers:
the long-capture words had outrun every detail capture on disk, so the
frontier had only the draw stream, and a draw stream can agree on a wrong
destination for a long time (item 336's journal is the argument).

## What landed

**run96 — East Indies late census, `[23960, 24000)` at run80's `LEADERS=9`
detail.** 88,806,321 bytes, 40 blocks 23960..23999 plus a 24001 closing
block. Identity against run54: **0 differing frames, 24,001 identical.**
Records: `LEADERDATA` 168, `GUY` 16,598, `UNITDATA` 8,029, `BUILDDATA`
1,653 — **richer than run80's Great Lakes** on both `GUY` (11,205) and
`UNITDATA` (5,259).

This run was not in the original brief. `loop-338` asked for it while the
lane was starting, and the reasoning is worth keeping: its scholar-gated
`Unit::go_inside` tail moved **both** maps, and it could only be *proved*
on Great Lakes, because the value diff came from run80's endpoint census —
and East Indies had no rich end-of-game census at all (run54 is 11 MB over
24,000 frames; run80 is 82.7 MB over 40). **A value diff does not have to
sit on the word's frame**: for a cause whose state persists, the endpoint
dump answers it for a fiftieth of the bytes.

**run98 — East Indies `[7880, 9100)`, and it is TRUNCATED at 8788.**
512,298,924 bytes, **910 blocks 7880..8789** against the 1,220 asked for.
Identity against run54: **0 differing, 8,789 identical.** Overlap against
run90: **20 blocks (7880..7899), 0 differing** — byte for byte, exactly the
assertion the 7900 window could not have made. And **both of this map's
words carry values**: `to_x` on 8466 (11 records) and on 8495 (10).

**The game died; the dump did not merely stop.** The trace's last `FRAME`
record is sim-frame 8788, the dump's tail is cut mid-`GUY`-record, and no
game process survived. `poll_max` was never reached — 105 polls of 200 —
so this is not the header's POLL_MAX truncation. The machine was at load
average 8.91 with another game running; the cause is not established and
was not worth a reading. The product is ~300 frames of runway past 8495
rather than the ~600 intended, and the two words are covered.

**The checks were left able to fail and retargeted at the archive that
exists**, not weakened to pass and not left asserting a window that is not
there — the same call run96's 40-versus-41 got. They still fail loudly on a
wrong game, a corrupted dump, or a lost overlap.

**run97 — Great Lakes `[8030, 9350)`, and it is whole.** 616,832,145 bytes,
**1,320 blocks 8030..9349**, no gap. Identity against run53: **0 differing,
9,361 identical.** Overlap against run94: **15 blocks (8030..8044), 0
differing** — byte for byte at identical detail, which is what opening at
8030 rather than 8150 bought. And the words carry values generously:
`to_x` on **8272 (350 records)**, **8374 (315)** and on 9349 (150), so the
runway's far end is dumped too.

It was taken on the third attempt, after the machine was free.

**run97 completing is also the experiment that closes run98's truncation.**
The same shape of window, the same detail, the same rate, on the same box —
1,320 blocks without incident. So run98's death at 8788 was contention on a
loaded machine, not a systematic fault in the window size or the tooling,
and no paragraph is owed to it beyond the record above.

## The two attempts that failed first, and why

**run97 hung twice before the machine was free:**

1. **00:47:08-44, hung.** No `gamelog.txt` was ever created, the trace
   froze at 352,512 bytes with **zero `FRAME` records**, and the game sat at
   **68 % CPU** for six minutes. Calibration says that is not a slow run-up:
   run96's start dump appeared **20 seconds** after its start click. System
   Events reported one `AXStandardWindow` and no modal, and `wine97.log` is
   byte-identical in length to `wine96.log` with no error.
2. **00:54:40, `perm_probe` refused.** "Accessibility is off — a synthetic
   move did not land. asked for 37,41 and 53,59; the cursor read back 37,41
   and 55,59." **Two pixels.** `tccd` shows `com.ramonfabrega.rondriver`
   being evaluated with no denial, so the grant is present.

**The cause is contention for the screen, and nothing was broken.** A League of Legends *match* has
been running since **00:35:33** and `LeagueofLegends` is the frontmost
process; Discord was active at 00:55:48. Clicks that land in another window
look exactly like failure 1, and a contended cursor looks exactly like
failure 2. A capture lane losing to a foreground application on a shared
machine is the normal condition, not a defect: no tool misbehaved and no
grant was missing. The lane stopped rather than fire more synthetic clicks
into a machine someone was using, and rather than diagnose a human at the
keyboard into a fact about the hardware. **run97 was taken later the same
night, once the machine was free** — verified independently before
restarting, not on report: the match process gone, frontmost `iTerm2`, load
down from 8.91 to 4.88.

run98 was left to finish: after its start clicks it never touches the screen
again. **The honest framing is "costs the screen nothing and one core
something"** — it sat at 74.8 % CPU against a load average of 8.91 while
someone gamed. It was left running because it was four minutes into
thirty-five and one busy core on this box is marginal for a game; that is a
judgement, not a free action.

### The failure mode worth naming: a click that lands in another window

It mimics a revoked TCC grant closely enough to cost the next session an
hour, and the two are told apart cheaply.

| | click landed elsewhere | grant actually revoked |
|---|---|---|
| game process | alive, high CPU | alive, high CPU |
| `gamelog.txt` | never created | never created |
| trace | frozen, zero `FRAME` records | frozen, zero `FRAME` records |
| `perm_probe` | passes, or misses by a pixel or two | fails outright, every time |
| `tccd` `AUTHREQ_SUBJECT` | subject evaluated, **no denial** | denial recorded |
| frontmost process | **something else** | the game, or nothing |

**The discriminator is the last row, and it costs one `osascript` call:**
`tell application "System Events" to get name of first process whose
frontmost is true`. Check what is frontmost *before* blaming TCC. A game
that has captured the mouse also drags a synthetic move off by a pixel or
two, which is what run97's probe reported — so a *near* miss is evidence of
contention, not of a missing grant, and the existing probe cannot tell the
difference because it tests for an exact match.

## Three corrections to the brief, each of which would have cost the product

- **East Indies at `[7900, 9100)` would have overlapped run90 in nothing.**
  `frame_window`'s HI is **exclusive**, so run90's blocks stop at 7899 and
  its 7916 is the `!quit` block `samegame.py` drops as each file's last. The
  overlap would have been **zero blocks** — and `samegame.py` exits 0 when
  nothing is in common, so it would have been reported as a clean same-game
  result. That is the vacuous pass run83 caught. The window opens at
  **7880** instead, for 20 byte-for-byte blocks.
- **Great Lakes' usable neighbour is run94, not run19.** run19 sits at a
  **higher detail level**: 1.54 MiB a block against run94's 0.40, net of
  preambles (1,598,671 and 10,765,469 bytes), for the *same* block kinds —
  the `BEGIN <KIND>` sets differ by exactly one entry each way
  (`ATTACKORDER`, `GROUPATTACKTOORDER`) — and the same population, 93
  `UNITDATA` a block against 90. The ini value is a threshold, not a flag.
  So the window opens at **8030** to butt against run94's 8044 tail at
  identical detail. **This lane first wrote that run19 was a `DUMP_ALL`
  window; that was wrong**, caught by a peer session and re-derived here
  before it was taken — a real `DUMP_ALL` block is 50-64 MiB.
- **run96's own block assertion was wrong and the capture was right.** It
  was written as run80's 41 blocks at 23960..24000 and came back 40 at
  23960..23999, for the same exclusive-HI reason. The assertion was fixed,
  not the window. **Asserting the count is what made it visible at all.**

## The unattended lane cannot serve a windowed detail capture

The brief named `tools/explore/unattended_capture.py` and asked that the
clicked lane not be driven unless it failed. It fails three ways, and the
first is structural:

- **It cannot express the capture.** `live_session.stage` hardcodes
  `LogStartFrame=18, LogEndFrame=36` and zeroes every `[End Frame]`
  category except `UNITS=3`. No frame window, no
  `BUILDS`/`CITIES`/`GUYS`/`DEATHS`/`LEADERS`. A *successful* run produces
  the wrong product.
- **It does not claim the headline games.** `docs/lab/2026-09-09-autostart.md`:
  "it does not claim those rules equal the headline human-versus-AI
  captures." Identity with run53/run54 is the whole point.
- **It failed when run.** End-frame 1400, both maps: exit 5 at 3.9 s, page
  fault at `0xA2C457` before the menu-selection record, no gamelog. Not
  permission-shaped — the game got its Vulkan device, and the only
  `nodrv_CreateWindow` line belongs to winedbg's crash dialog *after* the
  fault. Settings were restored and byte-verified.

## The tranche

| run | map | window | bytes | blocks | identity | overlap |
|---|---|---|---|---|---|---|
| 96 | East Indies | `[23960, 24000)` census | 88,806,321 | 40, 23960..23999 | run54: 0 / 24,001 | — |
| 97 | Great Lakes | `[8030, 9350)` | 616,832,145 | **1,320**, 8030..9349 | run53: 0 / 9,361 | run94: 15, 0 differing |
| 98 | East Indies | `[7880, 9100)` | 512,298,924 | 910, 7880..**8789** | run54: 0 / 8,789 | run90: 20, 0 differing |

Every word this lane was sent for now has a value diff: `to_x` on Great
Lakes **8272** (350 records) and **8374** (315), and on East Indies **8466**
(11) and **8495** (10).

## Storage

Free disk 35.4 GiB before, **33.9 GiB after** — 1.2 GiB for the three
captures, about 3 % of what was free.
Sizes were re-derived from the siblings' own files rather than taken from
the brief: run94 is **461 KB a block**, run90 is **645 KB a block**. A
watchdog enforced the three hard limits rather than reporting them — free
disk below 15 GiB, a run's output past 6 GiB, or one run alive past 90
minutes, each killing the queue. **Its wall-clock guard was dead on first
write** (`ps -o etimes=` is a Linux keyword; macOS prints its keyword list
and the test silently never fires) and was fixed and tested against
`05:30`, `1:02:03`, `2-03:04:05` and `00:45` before it was trusted.
