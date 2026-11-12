# The pass of 2026-11-12, by supply: the disk, the held leaves, and the lanes' next items

*2026-11-12 10:38 to about 13:30 −05:00, the `steer` row on Fable 5.2, armed
by `commander` on supply (`docs/DECISIONS.md` entry 126 §4) with the tree
banked at `d64f135cc`. The charter: design two or three lane items with
scores that can fail, rule on the leaves that part only beside a parent and
on the `clock` test going red under a lane's draws, keep step V of item 4159
as designed, and say whether item 4034 gets a brief kind. Ramon was on the
row for most of it. Decisions in entry 128. The pass before it is
`2026-11-12-fable-pass-render.md`.*

## For Ramon

- **The disk was the real blocker, and it is closed by a tool.** It read 20
  GB at the start and 243 after: 10,725 stale decoded-cache generations
  removed from the two archives, 40.4 GiB of build output in merged builder
  trees, then the 23 trees themselves. The gate runner now does both after
  every green gate, so neither piles up again.
- **The leaders line was releasing a third of what it built.** Steps II to
  IV built 43 leaves and released 19; the rest part only where a value
  they are computed from already parts. They now leave on a check that can
  fail (the transfer rule, entry 128 §3), and the chain's next step is that
  harness, ahead of step V.
- **Four workers stand, on your word, on trial**: the builder and three
  lanes, the third on items that never run the timed suite. The next pass
  confirms it or pulls it back.
- **Seven lane items are booked** (4178 to 4184): the lake drawn as water,
  two steps of the mouse, the shaders out of `gpu.rs`, shadows and the
  city's base, rounds and corpses, the lobby as a plain page.
- **The tranche**: six lane landings, 64.21 USD at list, 10.70 a landing;
  two builder steps at 88 and 107 minutes. The week read 11 % at 16:05Z
  with 43 hours left: about a point an hour on three workers.
- **Nothing waits on you.** The original's interface as a port is yours to
  reopen once the plain page exists.

## What was measured

| what | measured |
|---|---|
| the tranche (`tranche.py --base 27b21c8f6`, after `lore index`) | 6 lane landings, 64.21 USD over 6 sessions, 10.70 a landing; mean session 54.0 minutes clock, 25.3 working, 28.6 waiting (gate 24.4); five deep (4170 at 316 k, 4171 at 312 k, 4173 at 381 k, 4174 at 356 k, 4175 at 405 k); no take of the original. Items 4176 and 4177 were in-session builders, 88 and 107 minutes, and have no lane session |
| the gates (`gates.py … --since gate-pass-1112.log`) | 8 gates, every one whole, 7 green and 1 red on `clock` and re-run green; 167.2 minutes; admission 6,703.7 s summed, the longest wait 529.5 s (4170's, whose gate waited 3,786.7 s in all) |
| the window (`lore limits`) | the whole week 10 % at 14:30Z and 11 % at 16:05Z, the Fable week 5 % at 14:04Z, both closing 2026-11-14 06:00 −05:00; 5 % at 10:29Z, so about one point an hour |
| the rubric (`rubric.py --record`) | the territory 731 · 332 · 576 from 669 · 336 · 1066 (item 4170); simulation backed 1,456 from 1,448; lockstep bytes 78.1 %, ages 8 of 8, unit types 296 of 302, unmoved |
| the disk | 20 GB free at the start; the dev profile 3.4 GiB; seven merged builder trees' `target/` 40.4 GiB; `rondata --gc-oracle` on both archives, 132 caches and 10,698 generations in `~/ron-golden`, 1 and 27 in `~/ron-data`; 243 GB free after. One generation is 1.51 GiB and 91,739 files over the archive, and a parser edit strands one |

`afsctool` was not run: the last pass stopped it at 51 % after 55 minutes,
and with 243 GB free the compression of the captures added since is not what
binds. It is the next pass's if `df` reads under 100.

## What the pass looked at

The client's still (`client --frame 3279 --still`, the tip's release build)
at both photographs' cameras, each set beside its photograph. The stills are
under `~/ron-data/lab-experiments/2026-11-12-pass-supply/`, outside the
repository.

On run1559b's frame the picture is close, where the pass before found it
washed and flat: the trees are textured, the figures carry armbands and
turret marks, the mountain is pale rock. What the eye still finds, none of it
under a row: the city stands on bare grass where the photograph has a paved
base with a red border; nothing casts a dark patch, not a tree, a figure or a
building; the land beyond the map's edge is black where the original's is a
dark olive; there is no smoke, fire, muzzle flash or dust; the tanks are a
brighter green. On run1564's frame the lake is unchanged: the floor's texture
tiling in cyan, a hard sawtooth beach a tile wide, and a few figures drawn on
the water where the photograph has dark shapes under it.

## Rulings

- **An inherited parting is not agreement; a leaf is released on
  transfer** (entry 128 §3). 4167's, 4176's and 4177's "For the Loop" lines
  each asked whether a leaf that parts only beside a held parent may count
  as agreeing. It may not on the story alone. It may when the crate's own
  step, fed the original's printed inputs on one record, gives the
  original's printed leaf on the next, over every consecutive pair on
  disk. 24 of the 73 leaves out are built and held this way. `4176b`,
  `4177a` and `4177b` part on frames of their own and are findings.
- **The chain's next step is the transfer harness, then V.** V is not
  redesigned. The harness goes first because it sets the rule every later
  step's leaves are released by, and because at most 24 leaves ride on it
  against V's nine.
- **Item 4034 gets a brief kind**, `start`, gated whole. It is taken when
  no scoped item stands, one such lane at a time beside the chain: Ramon's
  condition on the fourth worker is that it does not run the full suite.
- **The input gets one too**, `input`: the method of items 4145 and 4149
  as rows, with the capture section, gated scoped.
- **The `clock` red was not our draws.** The handoff named a lane's stills
  on the same card. A builder built the lock against that and measured it:
  no difference (see "Built"). The cause of gate-4171's p99 75.10 ms is
  open. The test now measures a break twice before it fails (entry 128 §8).
- **The cache collector and the builder trees are the gate's to clear**
  (entry 128 §1), on Ramon's word. A capture is still removed only by a
  list he approves (entry 84 (iv)).
- **The worktree stays.** It holds under 1 GB without its build.
- **The parked file needs no work**: entry 125 removed it this morning.
  What still only grows is the per-item journals (1,476 files, 12 MB), the
  chronicle (1.2 MB) and the decisions (788 KB); no session reads one whole.
- **The shaders are strings in `gpu.rs`** and were booked nowhere; item
  4181, on Ramon's question.

## The items, and what each can fail on

| item | kind | the score | how it fails today |
|---|---|---|---|
| 4159 step T, the transfer harness | chain | `LEADERS_HELD_OUT` 73 → 49 at best | a leaf whose step does not reproduce the original's next record; a bent step, seen red |
| 4178 the water and the shore | render | `RUN1564_3279_GROUND_STILL` 1,237 of 1,920; its luma twin 791 | water and shore 527 of 1,209 |
| 4179 a right press on an object | input | `INPUT_CLICKED` up by its orders; the original plays the file | no arm mints a gather, an attack or a garrison |
| 4180 a dragged box and a group | input | the same, on 4179's script | no drag, one object selected |
| 4181 the shaders leave `gpu.rs` | render | none; both stills byte-equal | a changed character moves a still |
| 4182 what stands under things | render | a row over the plaza's and the shadows' tiles, rule before count | bare grass; no shadow drawn |
| 4183 the rounds and corpses | render | a still row on a new photograph | `render::draw` lists neither |
| 4184 the lobby, a plain page | lobby | lobbies whose `GAMEINFO` the page reproduces, N of 161 | no page: 0 |
| 4034 the start block's width | start | `Start: N of 161` | 1 of 161 |

Order: the builder on T; lanes on 4178, 4179 and 4181; 4182 and 4183 after
4178 on the render lane; 4180 after 4179; 4184 after 4181; 4034 when no
scoped item stands.

## Built

Three builders, `lean` agents on Opus 5.5 in worktrees of their own, each
red first; the pass merged each `--no-ff`.

- **`tools/brief.py`: the `start` and `input` kinds** (`6faa574f8`). A start
  brief carries the counter, the on-disk oracle, the quiet-field warning,
  cheapest first and a whole gate; an input brief carries the method of
  `docs/CLIENT_INPUT.md` in its six steps, with the capture section and a
  scoped gate on `input::`, `minted::`, `recgame::`, `coverage::` and
  `blind::`. Four tests in `tools/explore/test_brief.py`, each seen red.
  The builder corrected two of the pass's rows against the tree: `script::`
  matches no test, and `INPUT_PLAYED` is a number in the document whose pin
  is `CLICKED_WALKED`.
- **`tools/disk.py`, under `tools/gate_run.sh`** (`836212980`, `d0edd6969`).
  After a gate that exits 0: a builder's `agent-*` tree whose tip is an
  ancestor of the gated commit and off its first-parent line is reaped with
  its branch when clean, and loses its `target/` alone when dirty; then
  `rondata --gc-oracle` on both archives, on the binary the gate's survey
  already built. A freshly cut builder sits on the first-parent line and is
  kept; one in flight is no ancestor and is kept; a red gate runs neither
  step, since a just-merged builder may still be sent its failing test. The
  log's last line stays `gate exit <code>` and a failed step never changes
  it. Seventeen tests on a fixture repository with real `git`, seven
  mutations each seen red. It found `tools/gates.py` reading a gate's scope
  from the log's first three lines, and fixed it to count from `Gate pid:`.
- **A host-wide GPU lane, built and not merged** (branch
  `worktree-agent-a321dca8c2323aa9d`, `539da46a0`, `117bc85b4`). An
  off-screen device held an advisory lock on
  `~/Library/Caches/attrition/gpu.lock`, shared inside a process and
  exclusive between processes; three tests, each seen red; the stills
  logged waits of 2.0 to 2.3 s for the clock's session, so it serialised.
  Then the measurement the brief asked for, the clock test's p99 over three
  or four sessions a row:

  | beside the clock | with the lane | with the kill switch |
  |---|---|---|
  | one `--still` loop | 4.75–5.16 ms | 4.83–5.06 ms |
  | the client's test binary looping, four threads | 4.94–5.63 ms | 4.96–5.67 ms |
  | four `--still` loops | 5.04–5.26 ms | 5.58–5.89 ms |

  Against a ceiling of 20 and the red gate's 75.10 the lock buys half a
  millisecond at most. A mechanism its own measurement kills is not
  merged; the branch stands for whoever finds the cause.
- **`crates/client/tests/clock.rs` measures a break twice** (the pass's own
  edit). Seen red with the p99 ceiling bent to 1 ms: "breaks its ceilings
  twice running: first 4.74 ms; then 10.01 ms". Green on the pinned 20.

## Not done, and said

- **No sim or render code was touched by the pass itself.**
- **The finer gate rule is still unbuilt** (the pass before left it): which
  of `rondata`'s `artdata`, `trace` and `free` and `render::clock` can move
  a timed test. Every gate of this tranche ran whole. With three lanes on
  scoped items it is the first thing the next pass measures.
- **What a person needs on screen to play** past the plain page — a build
  menu, the resources, a selection's panel — is undesigned. Items 4179 and
  4180 mint orders from the mouse; nothing yet shows a person what they
  can order.
- **What broke the clock in gate-4171 is not found.** Not our stills. The
  candidates nobody measured: the gate's other tests on the processor, memory
  with the disk at a few GB free, the original drawing under Wine during a
  capture, another application.
- **`afsctool`** was skipped, as said above.

## The gate

Scoped to `d64f135cc`, on the pass's commit; its log is
`~/Library/Caches/attrition/gates/gate-pass-1112b.log`.
