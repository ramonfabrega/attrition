# The pass of 2026-11-12, by load: the fourth worker, the sweep that ate live caches, and the lanes' next items

*2026-11-12 16:50 to about 18:30 −05:00, the `steer` row on Fable 5.2, armed
by `commander` on supply (`docs/DECISIONS.md` entry 126 §4) with the tree
banked at `47ccea057`. The charter: confirm or pull back the fourth worker
against what the tranche measured, rule on the disk step's two failures and
the two timing tests that went red under load, read Ramon's wgpu note, and
design lane items with scores that can fail from the parked rows. Ramon was
on the row. Decisions in entry 129. The pass before it is
`2026-11-12-fable-pass-supply.md`.*

## For Ramon

- **Four workers: keep them.** Eleven landings in four and a half hours
  against eight on three, at 9.44 USD a lane landing against 10.70. The
  cost was two false reds and a lane's wait nearly doubling. Your
  condition did not hold as prose (four of nine lane landings ran the
  whole suite), so it is a mechanism now: one whole lane gate at a time,
  host-wide, by a lock.
- **The last pass's own tool was working against the lanes.** The disk
  sweep deleted every decoded cache but the gated build's, including the
  ones in-flight workers were reading: 130 to 239 generations a gate,
  rebuilt cold. The design had priced that as one cold read; it recurred
  at every gate. How much of the host's load it made is not measured. It
  keeps a cache used within twelve hours now.
- **The two timing tests wait for the host** before they fail, and a gate's
  receipt names the five busiest commands, so the next tranche says what
  the load is made of. Nobody knows yet.
- **The week closes near 60 % at this pace.** 17 % at 21:20Z, 37 hours
  left, 1.2 points an hour. The host binds, not the window; I would not add
  a fifth worker before the receipts say what loads it.
- **The milestone's checksum has an oracle nobody had used: 108 packets
  on disk**, taken for other questions, on three maps at frames 91 to
  17,146. The pair stood on frame 91 alone. Comparing every state line on
  each is the chain's next step.
- **No fifth worker yet, on your word**: the passes go deeper first, and
  the next one reads the load receipts and takes it up again.
- **A plain HUD is booked** (item 4197), first on the input lane, with
  its specification written by the pass (`docs/HUD.md`). The interface is
  the largest difference between our picture and theirs, and two orders
  hide behind keys.
- **Run by you during the pass: 65 merged branches deleted from origin.** `git branch -r --merged
  origin/main | grep -E 'origin/worktree-(att|agent)-' | sed 's#origin/##'
  | xargs -n 20 git push origin --delete`
- **The wgpu note: nothing to adopt**, one gap fixed (the shader test
  covered five of seven modules).

## What was measured

| what | measured |
|---|---|
| the tranche (`tranche.py --base 471755ade`, after `lore index`) | 9 lane landings, 84.98 USD over 9 sessions, 9.44 a landing; mean session 74.0 minutes clock, 21.5 working, 52.5 waiting (gate 31.4, the suite 8.8, a capture 2.7); six deep (4178 at 342 k, 4179 at 317 k, 4180 at 345 k, 4182 at 435 k, 4184 at 326 k, 4186 at 318 k); three takes of the original. Steps T and V were in-session builders, about 35 and 190 minutes |
| against the tranche before | 6 lane landings, 10.70 USD a landing, 54.0 minutes clock, 28.6 waiting (gate 24.4); 8 landings in about four and a half hours on three workers against 11 on four |
| the gates (`gates.py … --since gate-pass-1112b.log`) | 9 logs: 8 whole, 1 scoped; 2 red (gate-4179 on `clock`; gate-4185 at its offline step on the `DOORS` pin, 1.6 minutes), 1 without a receipt; 252.0 minutes; admission 18,932 s summed, the longest single wait 835.6 s. A whole booking gate beside the lanes took 34.6 to 60.8 minutes; alone (`gate-bank-1112.log`) 30.5, with 21.5 s of admission |
| who gated whole | four of nine lane landings: 4034 and 4186 by kind, 4179 and 4180 because each added a field to `crates/sim/src/observe.rs`; 4180 ran three whole lane gates |
| the disk step (the gates' own `gc-oracle` lines) | generations removed by each green gate: 0 (4181), failed (4159T), 239 (4184, then failed on `~/ron-data`), 132 (4034), 131 (4180), 131 (bank), over 133 caches. Each 130 is one whole generation that was not the gated build's: 1.5 GiB and about 90 k files |
| the window (`lore limits`) | the whole week 17 % at 21:20Z from 11 % at 16:05Z, the Fable week 10 %, both closing 2026-11-14 06:00 −05:00 |
| the rubric (`rubric.py --record`) | the territory 733 · 332 · 574 from 731 · 332 · 576 (item 4186's two); simulation backed 1,456; lockstep bytes 78.1 %, ages 8 of 8, unit types 296 of 302, unmoved |
| the disk (`df -g ~`) | 236 GB free at the start; no `afsctool`, since it reads over 100 |

## What the pass looked at

The client's still from the gated binary at `47ccea057`, at three cameras,
each beside its photograph: run1559b's frame 3279, run1564's (`--look
36960,38496`), run1570's frame 3331 (`--look 3168,30816`). The stills are
under `~/ron-data/lab-experiments/2026-11-12-pass-load/`, outside the
repository. By eye, no boxes taken:

- **The interface.** Every photograph has the resource bar, the message
  log, the minimap and the selection's panel; ours has three lines of
  debug text. It is the largest difference on all three and no item named
  it (4197).
- **run1559b and run1570, the city.** Ours stands on bare grass where
  theirs has the grey paved base with its red rim (4190); theirs burns on
  four buildings, smoke and fire, and ours draws neither; each of our
  building sprites has a black wedge at its left (4182e); theirs has the
  city's name plate and a pale cone of light at the general.
- **The same frames, the field.** The trees' shadows are there now, a
  little lighter than theirs. Our tanks are a brighter green and raise no
  dust. The pale dirt under the left stand of trees is stronger in ours.
  Beyond the map's edge ours is black and theirs dark olive.
- **run1564, the lake.** It reads as water now. Three things are wrong:
  a deep-blue polygon with hard, tile-stepped edges in the middle of a
  smooth gradient, where theirs is one painted surface with white
  breakers; the beach is still a sawtooth a tile wide; the fish are white
  figures on the surface where theirs are dark shapes under it (4192).

## Rulings

- **The fourth worker is confirmed** (entry 129 §1), with one whole lane
  at a time, kept by a lock in the lane gate and by a `whole` or `scoped`
  mark on every queued item.
- **The disk step's failures were the sweep racing the caches it should
  not have been removing** (§2). Entry 128 §1 stands for stale
  generations and merged trees; a generation in use is neither.
- **The clock and the memcap fixture were right about the host and wrong
  to fail on it** (§3). Neither is loosened: a ceiling is still a ceiling,
  and a host that never quiets is still a red, named so nobody diagnoses
  the client.
- **Parked 4185a is the chain's** (item 4189), since it re-pins
  `LEADERS_HELD_OUT` beside the builder's own steps.
- **Step P goes ahead of step VI** (§5, item 4188).
- **Parked 4183b is settled inside 4191**, first: the row's rule names the
  photographed frame before any window is counted.
- **Parked 4034a is the start lane's to settle** with the dump in hand
  (item 4196); the rule it applies is that a lobby counts only if its dump
  can fail the comparison.
- **4185's note for the Loop** (the probe that printed every compared row
  of the nine leaves) graduates when a third step reaches for it; step P
  is the second.
- **The remote deletes are not a blocker and not a session's to route
  around.**

## The items, and what each can fail on

| item | kind, gate | the score | how it fails today |
|---|---|---|---|
| 4188 step P, the checksum's lines on every packet | chain, whole | a `Checksum:` clause, lines agreeing of 12 a packet over the packets our walk holds; `LEADERS_HELD_OUT` by each unprinted leaf | only frame 91 is compared |
| 4187 step VI | chain, whole | `LEADERS_HELD_OUT` 61 by up to 18 | the leaves are held |
| 4189 the quit on a closing block | chain, whole | `LEADERS_HELD_OUT` by `defeat_type`, `victory_type` | 0 or 5 against 6; 2 against 0 |
| 4190 the paved base | render, whole | a row over the plaza's tiles on two stills | bare grass |
| 4191 the corpse and the round | render, whole | `RUN1570_DEAD`, pinned at none | neither is drawn |
| 4192 the lake's shore and floor | render, scoped | `RUN1564_3279_GROUND_STILL` 1,373 of 1,920; luma 1,001 | the hard polygon, the sawtooth |
| 4193 the figures' shadows | render, scoped | `RUN1559B_3279_SHADOW`'s figure tiles, 28 of 37 | drawn as read, 5 of 37 |
| 4194 one unit's press | input, scoped | the re-minted clicked file played in lockstep | the layer mints the minimap's fields |
| 4195 the attack, played | input, scoped | `INPUT_ATTACKED` n of n; the target's hits equal | no file carries one |
| 4196 Southwest Mesa, British Isles | start, whole | `Start: 143 of 161` | 18 lobbies part |
| 4197 the HUD's menu and panel | render's brief, scoped | `Hud:` the clicked file from button presses, k of k; the panel's numbers the dump's | no button |
| 4198 the HUD's bar | render's brief, whole | `Hud:` each number the `LEADERDATA` record's | the door carries no leader state |

Order: the builder on 4188, 4187, 4189, then VII. The whole slot: 4190,
4198, 4191, 4196. The scoped render lane: 4192, 4193. The input lane: 4197, 4195,
then 4194.

## What the pass ran, to check an item before booking it

`uv run --locked --project tools/emu python tools/recomp/checksum.py
<exe> ~/ron-golden/run1570/map-14/frame-snapshot.bin`: logger frame 3331,
`check_all` returned in 10.4 s, 273,201 ranges and 1,572,230 bytes, the
stream's sums equal to the original's on every line. Units 4,240 ranges
and 75,779 bytes, builds 1,890, deaths 4 ranges and 150 bytes, groups 556,
guys 1,112, leaders 271 and 58,446, cities 30, items 75, goods 180, world
8,294 and 310,369; walls and ammo empty. `find ~/ron-golden ~/ron-data
-maxdepth 4 -name frame-snapshot.bin` lists 108: 14 on map 14 beside the
four of this week, the rest on maps 18 and 7; `docs/RUNS.md` names 158
`RON_STATE_FRAME` lines. run1564's packet is frame 3294, run1559b's own
frame with the camera moved, so those two must give one leaders line.
Not run: any packet but run1570's, and our side of any.

## Built

Three builders and a reader, `lean` agents on Opus 5.5, the builders in
worktrees of their own and each red first; the pass merged each `--no-ff`
and removed its tree.

- **The sweep keeps a generation in use** (`eef0f93d4`, merge
  `6f7d0c895`). `rondata::diff::oracle::gc` moved to `oracle/gc.rs`. A
  reader touches `<generation>/.in-use` once a process; a foreign
  generation is kept when the newer of that mark and the directory's
  mtime is under twelve hours old (`IN_USE`). A failed removal is
  recorded and the walk goes on; the line reads `gc-oracle <dir>: N
  cache(s), M stale generation(s) removed, K in use kept, F failed; kept
  …` and a failure still exits 1 after everything else was swept. Seen
  red: with the old rule a fresh foreign generation was removed, `(4, 0)`
  for `(2, 2)`; with a failure made to abort, `(2, 4, 0)` for `(3, 5, 0)`.
  **The generation key moved** (`2ef74e85cc1efd2d` to
  `b97e57e18ede5933`), since `oracle.rs` is a schema source; the new file
  is not.
- **The timed tests, the receipt and the lock** (`c14670e75` to
  `d0f1ef25c`, merge `bc71a9403`).
  - `crates/client/tests/clock.rs`: the wait and the wording. Quiet, p99
    6.2 ms at a load of 4.8. Seen red twice: the ceiling bent to 1 ms
    ("breaks its ceilings twice running … (host load 10.87 on 16
    cores)"), and the load reader faked at 99 with the bound bent to 10 s
    ("the host stayed loaded: 99.00 on 16 cores for 10 s; unmeasured,
    re-run").
  - `crates/sim/src/memcap_guard.rs`: the child prints `held` and sleeps a
    day; memcap must fire within 120 s of `held`. Green in 2 s. Seen red
    with `memcap.sh`'s kill commented out: failed at 120 s, no stray
    child.
  - `tools/release_gate.py`: `Host top: <command> <summed %CPU>, … at
    end; … at start`, after `Host load:`; entries split on `, `, since a
    macOS command name can hold a space.
  - `whole_lane_lock`: only `--lane --whole` takes it; the kill switch
    skips it. A live run printed `Whole lane: waiting on pid 60951 item
    4186 started 2026-11-12T17:09:08-0500`, then `Whole lane: waited 2 s`.
- **One shader list** (`fba887624`, merge `8f6821730`).
  `gpu::shader_sources()` holds the seven assembled sources; `Gpu::new`
  and the naga test both walk it, a guard reads `crates/client/shaders/`
  for a file in no source, and the shade template's holes have their
  count. Seen red on the old list: "validated, built / left: 5 / right:
  7", and `not in any validated source: ["decal.wgsl", "shade.wgsl"]`.
  The client's tests went 80 to 83; no still's pin moved.
- **The reading of the wgpu examples** (no code;
  `~/ron-data/lab-experiments/2026-11-12-pass-load/wgpu.md`). Verdicts in
  entry 129 §8. Its measurements: `wgpu` 30.0.1 pinned; the frame is
  `Rgba8Unorm` headless and `Bgra8Unorm` in a window, the shadows' keep
  map `R16Float`; the client's 79 tests ran 30.3 s and the clock 6.1 s on
  the bank's gate, in the lane beside a library of 1,556.9 s.

## Not done, and said

- **No sim rule or render code was touched by the pass itself**; the
  builders touched a test fixture under `crates/sim`, so the pass's gate
  is whole.
- **What the load was made of is not found.** The budget admits twenty
  slots and lanes read 46 to 51. The candidates nobody measured: compiles
  in fresh worktrees, cold decodes after the sweep, the original under
  Wine during three captures, something that is not ours.
- **The finer gate rule is unbuilt for a third pass**: which paths can
  move a timed test. A field added to `observe.rs` gates whole today.
- **Smoke and fire, the name plate, the general's cone, dust, the
  off-map colour** are on the photographs and under no row.
- **The leaders line's end is not designed past step P**: what is left
  when VI and VII land depends on what the packets release.
- **The fifth worker.** A reader row on second readings and the census
  filler would spend Opus without loading the gate; Ramon's word is that
  the passes go deeper first.
- **`afsctool`** was skipped, as said above.

## The gate

Whole, on the pass's commit; its log is
`~/Library/Caches/attrition/gates/gate-pass-1112c.log`.
