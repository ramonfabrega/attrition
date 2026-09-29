# The runs — the oracle's ledger, one section per capture

`docs/ORACLE.md` is the runbook: how the original runs here, what it can
dump, how a check is staged, and the traps. This file is the other half,
split out on 2026-09-18 by the fifth Fable pass: **the story of each
capture**, in the order the runs were taken, one `## runNN — …` section
each, exactly as they stood in ORACLE — nothing was rewritten in the move,
and a citation by heading (`docs/RUNS.md`, "run89") resolves here as it
did there. The machine-readable half of every run is its stanza in
`tools/gamelog/captures.txt`; the dumps and traces live outside the repo
(`docs/ORACLE.md`, "Off CrossOver", for where).

**How this file grows.** A capture lands its section here, dated, with
the runs it overlaps, the categories and window it was taken at, what it
answered and what it did not. A run that changes *how* a capture is
taken — a key, a trap, a permission — writes that into ORACLE's runbook
and cites its run here. The section is the unit the size guard reads.

---

## The attrition run (run16, 2026-08-24) — the first of the blind runs

The first run taken off the blind list. Same lobby and seed as run12–14
(frame-0 word `0x3bd39ae9`), the traced exe with `cover=1` and no window,
`[End Frame] UNITS=3 GUYS=1 DEATHS=1 LEADERS=3 BUILDS=1 CITIES=1 MISC=1`,
`[Start Game] WORLD=0`. Driven by an Opus agent from a written brief, one
hour of wall clock for 6,872 frames; the predictions were written before
the log was read and every one of them was observed (`docs/ATTRITION.md`
and `docs/SUPPLY.md`, their last sections; `tools/gamelog/attr.py` is the
reader). The archive is `gamelog-run16-attrition.txt` (1.0 GB) and
`rontrace-run16.log` (93 MB). The scenario, in cheat lines:

```
peace who=1                    diplos[1] 0 → 1 at label 341
add hoplite who=0 206,78       three records o 6–8, hits 120, on the AI's ground
add supply who=0 208,80        o 9, hits 90 — bleeds at 8 too
add scout who=0 204,76         takes nothing
war who=1                      diplos[1] → 0 at 2417; periods → 0 at each refresh
tech who=1 allegiance on       (landed on the fourth try — see below)
add hoplite who=1 42,146       the AI's squad on Nubian ground: attrition 0
tech who=0 allegiance on       → 48 within four frames
add hoplite who=0 206,78       → 24 (Allegiance + Oath of Fealty on who 1)
move 10 190,60                 → 0 at the next refresh
add supply who=0 209,91        the shelter: last tick 6278, wagon at 6289
```

**What the run corrected in this document and the recipe:**

- **`move` takes no `,who`** (the table above is fixed); `die` does.
- **`LEADERS=3` is the minimum that prints `diplos[]`** — `LeaderData::log_data`
  sets detail 3 just before the array — so a diplomacy read-back needs it;
  `LEADERS=1` stops after `score`. `att`/`anti_att` sit at detail 7 and
  the census at 9, which crawls; the tech grants have **no read-back at any
  level** (`tech … show` prints into the closed console window), so a
  tech's landing is verified through its effect on a unit's `attrition`.
- **`[End Frame] MISC=1` is what emits `BEGIN FRAME n`**; zero it and
  `gl.py frames`/`lastframe.py` have nothing to count. Keep it.
- **The chat box drops about four lines in ten.** Eleven of ~19 landed
  first time; one line needed four tries and one never arrived in three.
  Two modal stalls (the city rename dialog from a stray `Return`, and an
  empty chat box left open) each froze the sim for a minute — Cancel and
  Escape respectively. This is the case for the scripted cheat channel
  below.
- **A squad left inside the enemy's borders at war walks off to fight**
  (no order given) and is dead within ~2,000 frames; two squads were lost
  that way. A long observation wants peace, or the far corner of the
  enemy's territory.
- **Object numbers are recycled** from the lowest free slot: a dead
  squad's 6–8 went to the next scout (6) and the next squad (7, 8, 10 —
  9 being the wagon's until it died). Read a unit's history as
  (kind, who, o) *and* its `myhits`/damage continuity.
- The route out is the top-right HUD icon → Game Menu → Quit Game;
  Escape closes the chat box and clears the selection but does not open
  the menu here.
- Frames 10 and 11 draw 226 and 254 times on `game_random`: 220 + 248 of
  them are `PathFinder::calc_road_cost` under `astar_caravan_road` <
  `find_road` — the game planning a caravan road on the sim's stream, a
  per-frame source `docs/SYNC.md` had not seen (its §6).

**The blind list after run16:** 424 cited, **99 never run** (from 156),
with the whole of attrition and supply, combat's arithmetic, target
selection and the AI's C++ producers now entered by at least one trace.
Still blind as groups: the order commands other than move/gather/build and
their `Group::action_*`, the scenario host functions, ships and aircraft,
the hero-generals.

**The next improvement is the loop itself, not the mechanic.** An hour of
an agent typing into a chat box that drops lines is the cost of every
blind run, and it is avoidable: `rontrace.dll` already trampolines
`Game::do_frame`, `ConsoleWin::parse_cmd@007d6470(this, String *, int
from_chat, int no_mouse)` is what the chat box calls, `MiscAccess::console_win`
is the pointer at VA `0xE7FA84` (PDB `0003:2595460`), and
`String::String(wchar_t *)@00a1edd0` builds a const string without the
heap. A `rontrace.cmd` of `frame: line` entries run at the top of the frame
— inside the tick, not through the order stream, and while paused — makes
a run reproducible to the frame and unattended; a scheduled `quit`
(console-only half, `from_chat = 0`) closes it through the menu's path.
~~That is the next thing built.~~ **Built and validated the same night —
next section.**

## The combat run (run17, 2026-08-24) — the channel's first real run

Fourteen `rontrace.cmd` lines, no driver, 2,600 frames in fourteen
minutes: six duels on unowned mid-map land with `!ai off` at frame 100.
What it found is `docs/COMBAT.md` §16 — every unit-on-unit hit in the run
is one of the sizes the formula predicts, with the flank sectors confirmed
by damage; the run itself is in `docs/JOURNAL.md` under 2026-08-29,
"Lifted from ORACLE.md".

**Five staging facts from it, used by every scenario since.**

- A cheat-placed unit faces `0x55555555` = 120° (clockwise from north, y
  south) until it is ordered, so bearings are precomputed:
  `dx = 4 sin b`, `dy = −4 cos b`.
- **A Supply Wagon flees on sight**, so it is a one-hit target unless the
  attacker spawns within striking distance.
- **`!ai off` stops the leader's strategy, not the buildings' queues** —
  the AI kept training citizens after it.
- `add`'s `find_nearby_spot` can land a unit **eight tiles** from the
  asked one, and an object number cannot be predicted from a file, so give
  each trial its own spot rather than a `die` line.
- `move` from the channel is a **teleport** (`Unit::set_new_location`),
  not an order — the question of why `move 6 190,60` "did not work" was
  the wrong shape ("The channel's vocabulary", above).
## The producers' run (run18, 2026-08-25) — the script ends, the C++ takes over

The blind list's third entry: *a long game past the script, at `LEADERS=9`
around a sweep*, which `docs/AI.md` §12.1 item 4 had been asking for since
the producers were implemented — **no dump had ever shown a non-empty make
list**, because the script blocks the C++ steps for the whole opening.

Two stages, both this lobby and seed (frame-0 word `0x3bd39ae9`), both
driven entirely from `rontrace.cmd` with three lobby clicks:

- **run18a** — `5 !ffwd 30`, `24000 !quit`; `cover=1`, no trace window, and
  **`LogStartFrame=0 LogEndFrame=0` in `rise2.ini`**, whose gate `0 <= f < 0`
  is never true, so the per-frame dump is off entirely while the
  start-of-game dump still lands. 24,000 sim-frames, 1.6 MB of gamelog,
  26 MB of trace, **~45 seconds of game time**.
- **run18b** — the same game with `[End Frame] LEADERS=9 UNITS=3 BUILDS=7
  CITIES=5 GUYS=1 DEATHS=1 MISC=1` and the window `[6374, 6590)`, i.e. the
  sweep the script dies on *and* the next one; `cover=1 window=6374-6590`.
  217 blocks, 360 MB, ~9 minutes.

**What a run costs, measured** (and the reason the earlier estimates in
this section were wrong by an order of magnitude — they were the author's
wall clock, not the game's). Fast-forwarding with the dump off runs at
**~500 sim-frames a second**, about 35× real time: run18a's 24,000 frames
are 26.7 minutes of gameplay and took three quarters of a minute. A dump
block at `LEADERS=9 UNITS=3 BUILDS=7 CITIES=5 GUYS=1` costs **~2.4 seconds
and ~1.7 MB**, stable across runs (217 blocks in ~9 min, 19 in ~45 s). So
**a run's cost is `blocks × 2.4 s` and everything else rounds to zero** —
budget the window, not the frames, and put the frames you do not need
behind `!ffwd`.

The window's own semantics are run13's exactly — inclusive start, exclusive
end (run19 asked `[8174, 8192)` and got 8174…8191) — but **`!quit` emits one
or two ungated blocks of its own**, which is why run18a's "dump off" log is
not empty but holds frames 24000 and 24001.

**`ffwd` is the lever the combat run wanted.** `ConsoleWin::run_cmd`'s
`ffwd` case sets `game->fast_forward_frame = minute × 900` (bare `ffwd`
toggles 9,999,999); `TurnControl::check_new_frame_solo` skips the
wall-clock wait while it is non-zero and `Game::loop_render` draws one frame
in sixteen, clearing it once `fast_forward_frame <= frame`. It is a
*presentation* switch — nothing in the sim reads it — and with the per-frame
dump gated off it turns "3 frames a second" into **~500**. **The speed floor
was never the input or the renderer; it is the dump.** Window the dump and
fast-forward the rest.

**What the run establishes**, all of it in `docs/AI.md` §15:

- **The script ends at sim-frame 6376**, from `defensive`'s `case 29`:
  steps 28 (tower) and 29 both run in the one call, `research_tech_with_cost
  (who, "Classical Age")` queues the age — `num_queued` gains 544 while
  `ages_get()` is still 0 — and the call returns `SCRIPT_DONE`. Not the hang
  guard, and not the attacked-city bail-out.
- **The two ladders**, frame for frame in the dump's `production_step`:
  `1, 2, 3, 4, 5, 6, 7, 8, 0` over 6375…6383 on the sweep the script dies
  on, and `1, 3, 4, 5, 6, 7, 8, 0` over 6575…6582 on the next — one frame
  shorter, because step 1 entered with `prod_script_run` already 0 is
  promoted to 2 *and runs it* in the same call. Both are pinned
  (`ai_drive.rs`), and the first-entry frames in the trace agree:
  `production_ai_setup`/`market_speculation` 6377, `research_techs` 6379,
  `upgrade_units` 6380, `create_units` 6381, `create_buildings` 6382,
  `make_stuff` 6383, `produce_unit` 6582.
- **The make list is a ranked four plus seven category slots.** The dump
  shows entries at slots 0, 5 and 8 and nowhere else — `PEASANTS cat 5`,
  `TEMPLE cat 8` — which is `MakeList::make_me`'s tail writing each object
  into `list[cat]`, over a top-four insertion at slots 0–3. A new best
  **overwrites** slot 0 without shifting the old head down, so the citizen's
  rank-0 copy is simply lost when the temple outbids it and survives only in
  slot 5. `MakeList::clear()` at step 2 empties all eleven — visible at 6577.
- **Five expiry draws, and every one of them decides by `% 3 == 0`.** The
  trace records the seed before each `Random::get`, the dump records which
  slots survived, and the two together settle the arm `docs/AI.md` §2.6's
  prose had backwards: an ordinary building takes the **probabilistic**
  arm, not the unconditional one. Two sites, both new to the documents —
  `make_stuff+0x221` is the head's walk, `make_stuff+0x63d` the bought
  slot's.
- **`val /= 100` on a buy, observed**: the citizen bought at 6582 goes
  `714 → 7` in the same block and stays in the list.
- **The easy-difficulty stockpile clamp fires**: `bucket` 261/104/107 →
  37/43/77 across the setup step, on this Easiest lobby (`d = 0`,
  `m × 3/2`), with `econ`/`rate`/`worst_good` written there for the first
  time in the game.
- **A bought *slot* is not a bought *head*.** 6582 buys the citizen out of
  slot 5 and still disarms to step 0 — the second pass 9–11 is the head's
  alone.

**The blind list after run18: 424 cited, 89 never run** (from 95). The six
retired are the long game's own — `Army::find_target`, `Unit::unpack_merchant`,
`LeaderData::calc_rare`, `get_general_upgrade`, `HeroData::get_radius`,
`BuildTypeData::max_knowledge_gatherers`. What stays blind as groups: the
order commands other than move/gather/build and their `Group::action_*`,
the command processors, ships and aircraft, the scenario host functions
(`enable_production_ai` and the `disable_*` trio are scenario-only and
cannot be reached from a skirmish at all).

`tools/gamelog/steps.py` is the instrument — one block per frame with the
step fields, the goods picture, the queue, the eleven `MAKEOBJECT`s by slot
and the ten sites, `--terse` comparing only the step machine's own fields so
the ledger's per-frame tick does not print every block.

~~**Open:** `make_stuff` was reached twice and bought once; `produce_tech`
first runs at 8182 and `found_cities`' own purchases at 576, both outside
the window, so `research_techs`' and `found_cities`' *outputs* are still
unscored. A window around 8182 is the next one, and it is now cheap.~~

**run19 is that window, the same day** (`gamelog-run19-window-8174-8192.txt`,
`rontrace-run19.log`; `docs/AI.md` §15.6). One stage, `!ffwd 9` to frame
8100 and `[8174, 8192)` — 19 blocks, 32 MB, **a minute and a half of game
time**, which is the recipe above paying for itself. By 8182 the AI is in the Classical Age with a full make
list, and it shows what run18b structurally could not: the **second pass**
(steps 9, 10, 11, with `make_stuff` at step 8 called from
`production_ai+0x1fa` and at step 11 from **`+0x236`**), two purchases in
one `make_stuff` (`produce_tech` for the head and `produce_unit` for slot
1, both demoted 9,999,999 → 99,999), `research_techs` reaching the
**`9,999,999` overflow guard** in play and taking **no draw at all**, the
runners-up shifting through slots 1–3 with one falling off the end, and —
the best of them — a **scholar kept at a non-head slot and cleared at the
head on the same `% 3` residue**, which isolates the unconditional arm from
the probabilistic one on a single type. Nine expiry observations across the
two runs, nine agreeing.

**Open:** `found_cities`' own purchases are still unscored — they happen at
frame 576, inside the script's era, so the window that catches them also
catches the script calling `place_city_with_cost`, and the two callers have
to be told apart by the step in the dump.

## run20 and run21 — the islands map (2026-08-25)

The first runs off the profile's Great Lakes, and the first `DUMP_ALL`
capture read as the harness's own oracle rather than as a sibling.

**run20** (`gamelog-run20-islands-dumpall.txt`, 278 MB, `rontrace-run20.
log`): East Indies picked in the lobby's combo (`MAP_STYLE 18`, the WORLD
block's `map 18`, `sea_map 4`), run7's seed, and every logger at once —
`DUMP_ALL=1` with `InitialDump=1` (the start-of-game full dump: cells at
every level, the tile masks, the fog grids, `master_land_heights`, the
regions, the herds, the type tables), `check_all_level=14` with `[Misc
Logging] CHECKSUM=2` (the setup trace), `LogStartFrame=0 LogEndFrame=4`
(four `DUMP_ALL` frame blocks, `FRAME 1`–`4`), `rontrace.cfg` `cover=1
window=0-3`, and `rontrace.cmd` a single `4 !quit`. About seven minutes
wall clock: four for the start dump, ~110 s a frame block, and the quit's
own block. What that buys: a capture whose `Initial` carries its own
`checksums`, `heights`, `herds` and `frame_seeds`, so `build_sim` seeds
the personality, the stride and the slide from the run itself and the
`SITES` diff runs with no sibling at all — the shape every future
behavioural capture should take when the frames wanted are few.

**run21** (`gamelog-run21-islands-long.txt`, 1.7 MB, `rontrace-run21.log`,
29 MB): the same lobby, `DUMP_ALL=0`, `LogStartFrame=LogEndFrame=0`
(dump off), `cover=1`, `5 !ffwd 30` and `24000 !quit` — run18a's recipe on
the islands. Two minutes including the load. It is the trace that reaches
the AI's sea half (`docs/AI.md` §15.8: `check_transport` at 201, the docks
at 3579, `Army::do_transporting` at 14586).

**What the two runs settled** is in `docs/AI.md` §15.8 and the audit's
fourth pass: `world+0x34` is the style's `SEA_MAP` class and not a
landmass count; `land_key[]` is the static `BASELAND, SANDY, OCEAN, NONE`;
`is_ocean` is by cell kind; `was_seen` has a territory arm that the fog
grid alone does not explain; and B4-k's guard — the ten-record `SITES`
diff on run20 — passes and fails on demand.

**Three driving facts, each of which cost a relaunch or a wrong note:**

- **The traced process is `riseofnations_trace.exe`**, and
  `tools/gamelog/waitwin.sh` waited for `riseofnations.exe` forever; it
  matches both now.
- **`!quit` returns to the Game Over screen and the process stays up**;
  under `DUMP_ALL` the end-of-game dump keeps writing for a minute after
  it. Wait for `gamelog.txt` to stop growing, then kill.
- **Save to Profile does not survive a killed process** — see "The lobby
  is a file" above. The combo is read from a screenshot every launch.

## run22 — the first dock, under the window (2026-08-25)

The run21 lobby (East Indies from the combo, seed 12345, the profile's
Nubians) with the per-frame dump **gated to the frames that matter**: run21's
trace put `Dock::init` at sim-frame 3579, so `rise2.ini` `LogStartFrame=3579
LogEndFrame=3582`, `DUMP_ALL=1`, `InitialDump=1`, `[Start Game] WORLD=6`,
`rontrace.cfg` `cover=1 window=3579-3581`, `rontrace.cmd` `5 !ffwd 30` /
`3583 !quit`. **Eight minutes wall clock** end to end
(`gamelog-run22-islands-dock-window.txt`, 249 MB; `rontrace-run22.log`):
the start dump, the fast-forward to 3579 in seconds, three blocks of ~80 MB
at about two minutes each, and the quit's own two (3583, 3584). The staging
is one script, **`tools/gamelog/window.py stage 3579 3582`** / `restore`:
every ini edit, the cfg and the cmd in one call, and the reverse.

**What it holds.** Block 3579 (the end of sim-frame 3578): no active
`DOCK`, none of the AI's 14 units with `unit_masks & 0x800000`. Block 3580:
`DOCK dock 0, o 2010, reg 65, gull_o 15, who 1, dock_flags 1`; the building
`o 2010` of leader 1 at `(44160, 41856)`, `orig_type 432`; all 14 AI units
with the bit, the human's 6 without. (A `FRAME n` block under `DUMP_ALL`
carries two `FULL DUMP`s — the end of `n − 1` and the start of `n` — so a
count over the whole block doubles; the harness reads the first.) The
trace's frame 3579 repeats
run21's to the seed — draws 0 and 1 the gull's creation and
`Dock::init+0x125`. `docs/TRANSPORT.md` §10, §12; the assertion is
`rondata::diff::tests::run22_s_first_dock…`.

**Two things it settled that the reading had wrong or missing.** The dock's
`reg` is the **sea** (65): a dock's centre cell is water, so `Dock::init`'s
`reg < 0x40` guard never counts it into `reg_docks`. And **owner 9 is not
in a `DUMP_ALL` frame block**: the 208 `ANIMALDATA` records of block 3580
are all leader 8's; the gull (`gull_o 15`, who 9) is nowhere in the dump,
so its position and heading are the trace's to attest, not the log's.

**Driving notes.** `cliclick m:X,Y w:400 c:X,Y` fired every button this
run; the press-and-hold form (`dd`/`du`) registered as a hover on the
first Solo Game click. The window sat at `(760, 152)` again; the lobby
came up on the profile's Great Lakes and the combo pick was read back as
`MAP_STYLE 18` in the log's first lines. `!quit` from the cmd file left
the process at the Game Over screen writing blocks 3583–3584 for a minute;
a size poll on `gamelog.txt` (90 s unchanged) is the "done" signal, then
`pkill -f riseofnations_trace.exe`.

## runs 23–31 — the army's and the group's captures (2026-08-25/26)

Nine runs of the run21 lobby, staged from `docs/ARMY.md` §16 and
`docs/GROUPS.md` §11 and driven unattended by `tools/gamelog/runwin.sh N LO
HI TAG`. The story is in `docs/JOURNAL.md` under 2026-08-29, "Lifted from
ORACLE.md"; what belongs here is the inventory and the traps.

| run | file | what is in it |
| --- | --- | --- |
| 23 | `gamelog-run23-islands-war.txt` | a null result worth keeping: a Quick Battle **already starts at war**, so the `war` line changed nothing and all 24,001 per-frame words are run21's. `tools/gamelog/rngcmp.py A B` is the ten-second check that a scenario took at all |
| 24 | `gamelog-run24-islands-raid.txt` | the first traced game with combat in it: seven hoplites beside the AI's capital, which falls at 13125, and the AI is defeated at 16488 |
| 25–27 | `…-emergency-`, `…-findtarget-`, `…-defending-window.txt` | `DUMP_ALL` windows of run24's game at [12129, 12132), [12024, 12027) and [15100, 15103) — `Armies::emergency`, `find_target` with a live enemy, `do_defending` |
| 28 | `gamelog-run28-islands-engagement.txt` | `cover=1` with no dump at all: `Army::engagement@006f5160` entered at **15100**, the frame's coverage naming the chain down to `Group::action_attack`. Three minutes — the cheapest behavioural check there is |
| 29 | `gamelog-run29-islands-engagement-window.txt` | the same scenario windowed: its `ARMY` half is `docs/ARMY.md` §17 and its `UNITS=3` half `docs/GROUPS.md` §6.4 |
| 30 | `gamelog-run30-humangroup-nogroups.txt` | `GROUPS=1` under `[End Frame]` only, so **no `GROUPDATA` came out**; kept as the first dump holding `GroupMoveOrder` blocks |
| 31 | `gamelog-run31-humangroup.txt` | three human right-clicks on twelve selected units: forty frames carrying a `GroupMoveOrder` and a live group (`docs/GROUPS.md` §11, §12.1) |

**Four facts from them that every later run uses.**

- **`select` is scriptable, so a group capture needs one human click and
  not twelve.** `ConsoleWin::run_cmd`'s `select` case takes
  `[[ob#|type] [who] [+]]`: with a *type* it walks every object of that
  player and adds each match to its `SelectGroup`, and the trailing `+`
  suppresses the clear. `160 select slinger who=0` then `170 select
  hoplite who=0 +` puts twelve portraits in the tray. (The old note that
  `+` "is not reliable" was the **chat box** dropping lines, not the
  command.)
- **`rontrace.cmd` clamps a frame lower than the previous line's to it**,
  so a `!quit` written after a later-frame `add` runs at the *later*
  frame. Write the file in ascending frame order.
- **`!quit` at `HI + 1` does not end the process**: the game returns to the
  Game Over screen and runs on at fast-forward until it is killed, so a
  windowed run has more blocks than its window — the extra is a free
  capture of a later frame, labelled by the frame it was written at.
- **The East Indies lobby at seed 12345** (`MAP_STYLE 18`) starts the human
  on tiles 26–30 × 25–42 and the AI on 198–214 × 201–214.
  `tools/gamelog/live.sh` launches and **returns with the game running**,
  where `runwin.sh` waits for a `!quit`; `archive.sh N TAG` is the other
  end. At ~370 KB a frame the game runs about one frame every two seconds,
  which is what leaves a driver wall clock between frames.
## run32 — the road on fresh ground, and the heights of another game (2026-08-28)

The capture item 55 asked for, and it settled the item twice over.

**The recipe is `tools/gamelog/roadcapture.sh`**, end to end: it probes the
three macOS permissions, puts run10–14's game back (seed 12345 in `rise.ini`,
map style 14 "Great Lakes" in `check.ini` **and** in `PlayerProfile/Player.dat`
— the `<MULTI>` block is the one Quick Battle reads), stages the `DUMP_ALL`
window `[104, 109)` and a `rontrace.cmd` of four lines, drives the lobby and
archives. Nine minutes, 366 MB, five frame blocks.

```
5 !ffwd 30
100 add granary who=0 6,171
100 add smelter who=0 33,161
110 !quit
```

**The channel can place a finished building, and it plans its road there and
then.** `run_cmd`'s `add`, for a type index past the units, calls
`Objects::init_build` and then the object's vtable slot `+0x1a8`, which
`vtables.txt` names `Build::activate`; `Build::init` calls `find_city` on the
way in, so the building joins the human's city exactly as a built one does.
What no reading had noticed is what happens next: `Wall::start@0063e810`
passes **`REGEN_FORCE`** to `mask_me`, and `BuildType::mask_me`'s tail is
`place_roads` — so the ring and the road go down at *start*, and
`City::regen_roads`' flag is what makes it happen again later. Frame 100's
first 2,913 draws are `calc_road_cost`'s, before phase 1.

**The two searches are separable, and not by counting.** `Wall::activate`
plays a sound off a *different* generator, and the trace records every
generator's draws in file order — so the one non-sync record inside frame
100 splits the road draws into the Granary's **1,043** and the Smelter's
**1,870**. (Looking a word up in the trace does not: every draw's seed is
just the LCG advanced that far, so "where does our word appear" always
answers "at our own count".)

**The sites were chosen against the simulation.** Every tile 10–20 from p0's
centre whose whole footprint carries `CITY_RADIUS`, for every type that
`connects_to_roads` and that `place_building` accepts — the city already has
a Library and a Market, so those are refused as one-per-city and the
Granary, the Lumber Mill and the Smelter are not. It is worth doing: a
site that does not bind to a city plans no road at all.

**What it produced.**

| what | where |
| --- | --- |
| the fresh road, tile for tile | 62 tiles; `run32_s_two_fresh_roads_are_the_original_s_tile_for_tile` |
| four scheduled replans, node for node | 332, 231, 232, 60; `run32_s_scheduled_replans_cost_the_original_s_nodes` |
| the terraform's before and after | run13's `FRAME 100` heights against run32's `FRAME 104`: 128 corners, in the two footprints' boxes and nowhere else |
| the search reads the **pre**-terraform grid | 1,046 nodes against the original's 1,043 on the pre-terraform table, 967 on the post |

**And the thing it was not looking for.** The harness had been feeding the
road search `master_land_heights` from **run3** — the same seed, style and
size as run10–14 but `GAME_RULES 0` rather than 1, so its starting buildings
terraformed different ground and its grid differs on 237 corners spread from
(7, 83) to (230, 163). `borrow_from_siblings` took the first sibling with a
height table and run3 is first in the list. With the map's own heights
(run12's and run13's, which agree exactly), run14's frames 10 and 11 cost
**220** and **248** nodes — the original's, exactly, where they had been 208
and 222. That, not anything in the search, was item 55's six per cent.

## run33 — the long trace, and the proof that the instrument is free (2026-08-29)

run14 traced 284 of run10's 1,772 frames, and once the simulation's word
matched all 284 there was nothing on disk that could say where the two next
parted by *site*. run33 is the replacement: **run10's own game, traced,
1,850 frames**, and the recipe is `tools/gamelog/longtrace.sh` end to end —
the three permissions, `mapstyle.py` putting style 14 in `check.ini` and the
profile, seed 12345, run10's exact detail —

```
[Start Game] WORLD=6 TERRAIN=2 GOODS=3 UNITS=3 BUILDS=7 CITIES=5 GUYS=2 LEADERS=9 DEATHS=1
[End Frame]  MISC=1 UNITS=3 BUILDS=7 CITIES=5 GUYS=2 DEATHS=1 LEADERS=1
```

— `rise2.ini`'s frame window `[0, 1900)`, `rontrace.cfg` `cover=1` with no
window, and a two-line `rontrace.cmd` (`5 !ffwd 30`, `1850 !quit`).
**Fourteen minutes**, 284 MB of dump and 9.5 MB of trace, unattended.

**It is the same game, and that is now a measurement rather than a hope.**
`tools/gamelog/samegame.py` reduces each `BEGIN FRAME` block to a digest of
its indented lines and compares two dumps frame for frame; run33 against
run10 is **1,771 blocks in common and not one that differs**. So the traced
executable, the `int 3` on all 48,233 function entries, the cheat channel and
`!ffwd 30` are all invisible to the simulation over 1,771 frames — where
run18a had checked four — and run33 inherits run10's siblings, its
`build_sim` and its tests. (Calibrated on run10 against run14's gamelog:
284 identical blocks, the only difference being run14's truncated 285th.)

**What it says.** The simulation's per-frame draw **count** is the
original's through frame 306 and parts at **307**, twenty-three frames past
where run14's capture ran out. The original spends eleven draws there and
this simulation nine, and the two missing are one unit's non-flat gather —
a stand issued from `Unit::do_non_flat_gather+0x10f`, then `+0x54b`, the
gather's own roll. The site fires nineteen times in the whole run — 1, 168,
204, **307**, 465, 508, 565, 687, 780, 983, 985, 1030, 1091, 1393, 1498,
1544, 1590, 1643, 1781 — and this simulation makes the first three and
misses the fourth. Over the whole 1,850, **668 frames spend the
original's number of draws and 556 are its draws in its order**; both are
pinned in `run33_s_long_trace_says_where_the_word_parts`.

**And the thing it did not buy.** The blind list did not move — 617 cited,
**101 never entered**, with run33 in or out — though run33 entered 6,702
functions against run14's 6,585. A long run of the *same* no-input game
lights nothing new: the list is shrunk by scenarios, not by frames.

## run38 and run39 — the second map, and its first score (2026-08-29)

Phase 3's finish line names **two** maps, and every number in the harness
was Great Lakes. run38 and run39 are the other one: **East Indies**
(`MAP_STYLE 18`), seed 12345, and run10's rules otherwise (`MAP_SIZE 2`,
`GAME_RULES 1`, `REVEAL_MAP 1`) — the profile's lobby rather than
`check.ini`'s, which is the whole trick (see "The lobby is a file").
run38 is `tools/gamelog/startcapture.sh`'s `DUMP_ALL` start, two frames
and 151 MB; run39 is `longtrace.sh`'s 1,850 frames, 482 MB and an 11 MB
trace. Their traces' words agree on every frame they share, so they are
one game.

**run38 is the whole sibling list.** Its own `Initial` carries the
heights, the checksum trace, the herds and the frame seeds, so `build_sim`
stands the simulation up on a map it has never seen with nothing borrowed
— and **frame 0 is 175 draws against 175 on the first try**.

**The score, first time of asking: ticks 167, orders 167**; player 0 parts
at 219, player 1 at 168
(`run39_s_islands_game_is_the_second_map_s_score`). Great Lakes stands at
252 the same day, so the residue chased on the one map was not chased into
its shape. ~~What parts it first is an order-list **length**~~ — `1/4`
holds two orders on the original's frame 168 where this simulation holds
one, and its position parts on the same frame; `1/5` at 186 and `1/3` at
202 are the same disagreement — but **that is not what parts it first**,
see the next row.

## run39's trace, read at last — the second map's word (2026-08-30)

run39 shipped with an 11 MB `cover=1` trace and nothing read it: the map
was scored on its dump alone. Read against the harness frame for frame
(`run39_s_long_trace_says_where_the_second_map_s_word_parts`), **the word
parts at 19** — 148 frames before the order-list divergence above, so
run39's 167 is a number on a stream that is nobody's, and the row above is
struck for it.

The cause is the AI's **pasture**, which East Indies has and Great Lakes
does not. `docs/SYNC.md` §3.11 has all of it, including the coin's parity
argument (a pasture is one species, always) and the five position offsets
read back out of the trace's own seeds — **which is the first thing this
harness has taken from a trace rather than from a dump**, because owner 9
appears in no dump block at all.

The score beside the word is the **early window**, not a total: past the
parting a total is noise. Of the first 64 frames, **62 spend the
original's number of draws and 55 draw for draw** (2026-08-30; 49/47
before the pasture landed). What still parts at 19 is the *first* of the
two `Animal::do_idle` draws an arrival costs — movement's and animation's
residue, not the pasture's.

## run42 — `LEADERS=2`, and the pile is no longer unchecked

run39's lobby and seed exactly (East Indies, `MAP_STYLE 18`, seed 12345, the
profile's lobby with no `-config`), 900 frames, at run39's detail **plus
`LEADERS=2`**. The `LEADERDATA` block that carries `bucket`, `ages_get()`
and `epoch_get(scan)` is the encrypted one, and `LeaderData::log_data@006e5110`
announces it at detail **2** — the `this_00[1].handle = 2` before the
`LeaderDataEncrypt::log_data` call — not the **9** of the census. That is
the whole reason this capture is cheap: 245 MB and eleven minutes, where
`LEADERS=9` per frame is ten thousand lines a leader a frame and crawls.
`samegame.py --exclude LEADERDATA` against run39 is **900 frames in common
and not one that differs**, so it is run39's game and inherits its
siblings.

**It settles `docs/GOODY.md` §6's owed capture, and corrects one detail of
it.** The prediction was `bucket[2]` stepping by 50 on frame 867 with
`epoch_get(scan)` reading `0 0 0 1`.

- The step is there and it is the AI's: leader **1**'s `bucket[2]` goes
  **50 → 100** between the blocks labelled `FRAME 867` and `FRAME 868` — a
  block `FRAME n` is the end of sim-frame `n − 1`, so the pay lands on
  **sim-frame 867**, the predicted frame.
- **The trace names the cause rather than leaving it to be inferred.**
  `ObjectsData::find_goody_at` and `Unit::explore_goody` are entered on
  sim-frame 867 and on no other frame in the neighbourhood — 860, 863, 865,
  866, 868, 869, 872 and 880 all have neither. A box was opened on exactly
  the frame the pile moved.
- **`epoch_get(scan)` reads `0 1 0 1`, not `0 0 0 1`.** Civic is 1 as well
  as Science. It does not enter the formula — `epoch[3] × 25 + 25` reads
  Science alone, and `1 × 25 + 25 = 50` is the observed pay — so §6's
  arithmetic stands and only its stated vector was wrong.
- So the reading that mattered is **confirmed and its alternative refuted**:
  an `ages` reading would pay 25, because `ages` is 0 in an Ancient-age
  game, and the observed step is 50.

What run42 does **not** settle is which *good* a box picks: the lottery's
winner depends on the finder's buckets, and the frame's draw count would be
identical whichever good won (§6's last row). `bucket[0]` and `bucket[1]`
are visibly a different clock — the human's step by one every eleven and
fifteen frames respectively, all run long — so the record now on disk is
enough to separate the pile's income from the box's, which it was not
before.

## run43 — the terraform's own before and after, in one game

run32's scenario exactly — the same two enhancers on the same fresh ground
at the same frame, seed 12345 and map style 14 — with the `DUMP_ALL` window
opened four frames earlier: **[100, 108) rather than [104, 109)**. A block
`FRAME n` is the end of sim-frame `n − 1`, so `FRAME 100` is the grid before
either `add` lands and `FRAME 106` the grid after the Smelter's replan on
104 and the Granary's on 105.

**Why the four frames were worth a second capture.** `docs/ROADS.md` §7.1
reads the road search's grid as the pre-terraform one, and it had to reach
into **run13** for that grid — a different game, in which nothing is ever
placed. The two games are identical up to sim-frame 100 by construction, so
the substitution was almost certainly sound; "almost certainly" is what a
capture is for. run32's own window cannot supply it: `heightdiff.py` on
run32's `FRAME 104` against its `FRAME 108` moves **not one corner**, because
both are already post-terraform.

**What run43 says**, `tools/gamelog/heightdiff.py` on its own two frames:

- **128 corners move**, which is §7.1's number, now a single game's own
  difference rather than a cross-game one.
- They fall in **exactly two clusters of 64**, and nothing lies outside
  them: columns 3..10 × rows 168..175, and columns 30..37 × rows 158..165 —
  centres (6.5, 171.5) and (33.5, 161.5), for buildings placed at tiles
  **(6, 171)** and **(33, 161)**. So "the two footprints' boxes and nothing
  else" is exact, and each box is 8 × 8.
- **The corner grid is one corner per tile.** `master_land_heights` is
  `(4·xs + 1)²` for xs = 60 *cells* of four tiles — 58,081 corners, 241 a
  side, spanning 240 tiles. The 4 in the formula is cells-to-tiles, not
  tiles-to-corners, and run43's clusters are what says so: a building at
  tile 6 moves columns 3..10, and at tile 33 columns 30..37.

What this capture does **not** do is re-derive the two short node counts
(1,046 against the original's 1,043, and 1,460 against 1,870). Those are the
harness's arithmetic over the grid, not the dump's; what changes is that the
grid the harness should read them on is now this game's own, at a frame the
same file also carries the placement for.

**A trap this run cost, and the guard that caught it.** run43 was captured
twice. The first archive was 550 MB, the right map style, the right seed,
and a full window — and held **half its scenario**: zsh's `${(j:\n:)a}`
joins with a literal backslash-n rather than a newline, so the stanza's two
`add` lines reached `rontrace.cmd` as one, `ConsoleWin::parse_cmd` took the
Granary, returned 1 and dropped the rest. Nothing in the dump looked wrong.
`cmdsran.py` read the trace's `INFO cmd` records and said "3 lines parsed,
1 at frame 100, expected 2" — which is what it was written for one run
earlier, and it caught the bug on its first real outing. The same join had
silently emptied `rontrace.cfg`'s window line too. The fix is the `p` flag;
the incomplete archive was deleted rather than kept, because a
half-happened scenario that reads as valid is precisely run23's failure
mode.

## run44 — the turn override fires, and `guy_flags` has more writers than §9 has

`docs/ANIM.md` §4.6 calls `Guy::do_turn@005d97a0:15`'s override
unfalsifiable and owes it "a capture with a vehicle or a ship turning in
place". run44 fires it, and it needed no driver — which is a reading, not
luck. `Unit::move_step` passes the override flag on only its two
turn-in-place branches, but it is **not the only caller**: `Guy::move:109`
calls `turn_towards(this, des_angle, _, 1)` on the standing arm, guarded
only by `guy_flags & 2`, and `Guy::turn_towards@005d9720` hands its
argument straight to `do_turn`. So a turner unit **turning towards a
target** fires it, and a fight is enough.

run39's lobby, 700 frames, `GUYS=4` — `cur_anim` sits past the last of
`GuyData::log_data@005de6c0`'s three level announcements, so at run39's
`GUYS=2` a `GUY` block stops after `ox` and the question cannot be asked of
the file. Seven `add` lines from `rontrace.cmd`, all nine records accepted:
catapults and a trebuchet for the AI on the tiles beside its capital,
hoplites, pikemen and a catapult for the human among them.

**452 guy-frames play a turn animation**, in nine distinct
`(who, o, slot)` combinations, both `CHAR_TURN_LEFT` and
`CHAR_TURN_RIGHT`, on **both sides** — the human's pikemen from frame 166
and the AI's own catapults at 247 and 248, which the AI ordered unaided.
Every capture before this one has **zero**: run13, which does carry
clocks, has none in its whole window.

**And the flag byte says why, per type.** `guy_flags` in run44:

| value | bits | records |
| --- | --- | --- |
| 16 | 0x10 | 165,938 |
| 48 | 0x10 0x20 | 7,984 |
| 8 | 0x8 | 4,034 |
| 56 | 0x8 0x10 0x20 | 3,206 |
| 40 | 0x8 0x20 | 1,392 |
| 24 | 0x8 0x10 | 16 |

- **0x8 is exactly the three turner types** — 134 `PIKEMEN`, 265
  `CATAPULT`, 266 `TREBUCHET` — and no others, which is
  `Guy::init_real@005db6b0:179` setting the bit for a guy whose piece names
  a turn, observed rather than read. §4.6's "none of the eight a `DUMP_ALL`
  run's guys carry" is still true of those eight; it was a fact about which
  units had been captured.
- **0x20 is set on 12,582 records and it toggles within a type**: 50
  `PEASANTS` appears as both 16 (14,276) and 48 (1,534), 132 `HOPLITES` as
  16 (30) and 48 (6,450), 134 as 24 and 56, 265 as 8 and 40. So it is
  **state, not a per-piece init bit, and it has a writer.** `docs/ANIM.md`
  §9 lists 0x20 among three bits with "no writer found", says "none of the
  three is exercised", and leaves it off in the sim — where §9 also reads
  it as *collapsing the idle roll*, which is a draw.

  **This does not move either map's score today, and the reason is worth
  stating.** §9's "every guy in both dumps carries `guy_flags 16`" is still
  exactly true of the scored games: run13, which is run10's own game under
  `DUMP_ALL`, has **2,288 records and every one of them 16**, and run38,
  the islands start, has 1,180 and the same. Nothing in either turns 0x20
  on. run44 is simply the first capture that has **combat and guy-level
  detail at once** — the earlier fights (run17, run24) were taken without
  the clocks, and the earlier `GUYS=4` runs have no fight in them. So the
  bit is real, it is reachable, and it is waiting for the sim to arrive at
  the part of the game that turns it on; it is not a divergence in the
  1,850 frames anyone is scoring.
- **0x2 and 0x4 are still unobserved**, and 0x2 is a puzzle rather than an
  absence: `do_turn`'s first statement is
  `*(ushort *)&this->field_0x9a |= 2` whenever the angle actually changes,
  and 452 turn animations means that line ran. Either the dumped
  `guy_flags` is not the whole `ushort` at `+0x9a`, or something clears the
  bit before the frame ends. Unread here, and named rather than guessed.

Also worth keeping: 265 and 266 carry `8` and `40` — **without 0x10**,
which every other type in the file has. Whatever 0x10 is, the siege pieces
do not have it.

## run45 — the AI moves the mirror flag, and never lays a group move order

Item 23's driver-free shot, and a **negative result with a reason**, which
is worth more than the run it cost. run39's lobby, 900 frames, run31's group
detail; `groupfacing.py` over the whole archive:

| | |
| --- | --- |
| `GROUPDATA` blocks | 461,824 — 902 frames × the 512-slot pool |
| `group.facing` | 0 ×461,393, **1 ×431** |
| formations (`GROUPDATA.form`) | none ×460,923, **Line ×901**, nothing else |
| `GroupMoveOrder/MOVEORDER.facing` | **none at all** |
| other `MOVEORDER.facing` | −1 ×1,522, 0 ×699, 1 ×196 |

So the AI **does** form groups and its groups' mirror flag **does** flip —
431 records carry `facing 1`, which is `Unit::set_angle@00605400` toggling
it as a leader turns 90° or more off its heading. What the AI never does,
in 900 frames, is lay a **`GroupMoveOrder`**: not one unit in the file
carries one. `Unit::kill_current_order`'s hand-back reads the dying order's
own `MoveOrder +0x28`, so with no group move order there is nothing to hand
back and the XOR term cannot fire however long the run.

**What that settles.** The trace was right that the machinery runs —
`Group::action_move_near`, `Form::compute` and `GroupData::find_leader` are
all entered at frame 0 of run39 — and it was the wrong question to ask of
it. *Entering* `action_move_near` is not the same as a unit ending the frame
holding a `GroupMoveOrder` the dump can print. Item 23 needs a **human
right-click**, which is what run31 has and what no AI game supplies, and the
capture is run31's three clicks **plus a fourth**: `groupfacing.py` on run31
shows its group move orders carrying `facing 1` from frame 356 to 367 and
the log then closing, so the mirrored order it needs is already made and
simply never dies.

**Across every capture on disk, `GroupMoveOrder` is a human-click
artifact.** Counted rather than argued, over nine archives: run31 has
**945**, and run20, run13, run22, run25, run26, run27, run29 and run45 have
**none** — that set includes three `DUMP_ALL` windows taken *during* the
AI's own fighting (`Armies::emergency`, `find_target`, `do_defending`) and
900 frames of the group pool itself. So it is not that run45 was too short
or too peaceful: no AI in any captured situation has ever ended a frame with
a unit holding one, and run31, the one capture driven by right-clicks, is
the one that has them.

**And the Echelon half needs the mouse twice over.** Every group in run45 is
a **Line**, and every group in run31 is too. `docs/GROUPS.md` §6.4's slot
table only reads `reverse` on the Echelon rows, so the mirror is invisible
in the positions of a Line whatever the flag does. The console's 102
commands, re-derived from the user's own install by
`tools/gamelog/console.py`, contain **no formation verb at all** — the chat
half is `add`, `select`, `move`, `die`, `damage`, `tech` and the diplomacy
pokes — so a formation can only be set through the unit panel.

**A trap this run cost twice, now a check.** The first attempt dumped the
pool **once**, in the start block: 512 `GROUPDATA` records against run31's
111,616. `GROUPS=1` is necessary and not sufficient —
`GroupData::log_data@0045e1d0` calls neither `set_type` nor `set_detail`, so
its lines are accepted against whatever the previous dumper left, and
`dump_deaths@0092fd80` ends by calling `WorldData::log_data` twice, leaving
the type at `WORLD`; with `WORLD=0` under `[End Frame]` the whole pool fails
`check_accept` silently. **`DEATHS` off** under `[End Frame]` is the fix, and
it is written down in this file already — it cost run30 — which is the
argument for a guard over prose. `groupfacing.py` now fails on `≤ 512`
blocks and names the cause, so the next stanza to do it is told in a minute
rather than after a twenty-minute capture.

## run46 — the XOR term fires, and the formula is right

Item 23's event, and the first time `Unit::kill_current_order@005e2cb0`'s

    group.facing = order.facing XOR reversing(leader.angle - order.angle)

has run with `order.facing` **1** in any capture. run10's lobby, 900 frames
at run31's group detail, eight hoplites added and selected from the cheat
channel, and three right-clicks from `clickdriver.sh` — the camera alternating
between tiles (30, 167) and (6, 167) either side of the units, so every order
after the first is a ~180 degree turn and `reversing` is not left to luck.

| click | frame | its order appears | the order's angle | `order.facing` |
| --- | --- | --- | --- | --- |
| 1 | 213 | 216 | **+85.8°** | 0 |
| 2 | 333 | 336 | **−92.8°** | **1** |
| 3 | 453 | 456 | **+86.0°** | 0 |

Read it as two hand-backs, and both come out as the formula says:

- **click 1 → 2.** The leader turns +85.8° to −92.8°, which is 178.6° and
  inside the `reversing` window, so the toggle is 1. `Form::compute` sees
  `order.facing XOR toggle` = `0 XOR 1` = **1**, and click 2's order is laid
  out carrying `facing 1`. That is the mirrored layout run31 also reaches.
- **click 2 → 3, which is the one nobody had.** The dying order carries
  `facing 1`, the leader turns −92.8° to +86.0° — 178.8°, the window again,
  toggle 1 — and the hand-back is `1 XOR 1` = **0**. Click 3's order is laid
  out carrying `facing 0`, which is what the dump prints from frame 456.

The mirrored order lives on frames **336 to 455** and dies on the frame click
3's replaces it; 1,509 group move orders carry `facing 1` across those 120
frames, against **none** in run45 and none in any AI capture.

**What is still owed, and it is the other half of item 23.** Every group here
is a **Line** (`form 0`, 1,587 records, nothing else), and `docs/GROUPS.md`
§6.4's slot table only reads `reverse` on the Echelon rows. So the mirror's
*consequence for the positions* — the formation byte's sign — is still
unexercised: this run proves the flag is computed as stated and not what it
then does to a slot. A formation cannot be set from the console (its 102
commands have no such verb, `tools/gamelog/console.py`), so that half needs
the unit panel, which is a click on a button rather than on the map.

**Two instrument lessons, both of which cost a run.**

- **`!ffwd` stops the renderer.** run46's first attempt clicked three times
  and produced no order at all, and its screenshot showed the capital still
  selected — which read as `select hoplite who=0` having failed. It had not.
  run47's five screenshots, taken across ninety sim frames, came back
  **byte-for-byte identical** with the in-game clock at 00:00:00: the game
  was simulating and not drawing, so the driver was clicking at a picture
  minutes stale and no screenshot of that run was evidence of anything. With
  the fast-forward dropped the shots differ, the clock runs, and `select
  hoplite who=0` puts six hoplite portraits in the tray exactly as the
  recipe above says. `FFWD` is an input now; **every stanza with a `driver:`
  sets it empty.**
- **An accepted line is not a line that did something.** `cmdsran.py` reports
  what `ConsoleWin::parse_cmd` returned, and it returned 1 for the select
  that changed nothing. The tick means the channel took the line, and no
  more.

## run50 — the Echelon half, and the four doors that are shut

Item 23's remaining half, **not** obtained, and the value here is that the
search is now bounded rather than open. `docs/GROUPS.md` §6.4's slot table
reads `reverse` on the **Echelon** rows alone — Refused is `Y = Y0 - |X|`,
with no `reverse` in it, so the queue's "Refused or an Echelon" is really
Echelon only — and every group in every capture on disk, run46's included, is
a **Line**. So the mirror is confirmed as a computed flag (run46) and still
unobserved as a *displacement*.

What was tried, each with its evidence:

- **The console.** Its 102 commands, re-derived from the user's own install
  by `tools/gamelog/console.py`, contain no formation verb of any kind. The
  chat half is `add`, `select`, `move`, `die`, `damage`, `tech`, `resource`,
  the diplomacy pokes, `finish`, `hurry`, `pack`, `deploy` and `anim`.
- **The command card.** run48 photographed it with a group of hoplites
  selected: move, attack, auto-explore, board, stop, garrison, and fourteen
  empty cells. The compass-with-arrows that looked like a formation chooser
  is **Auto Explore** — the tooltip says so, and says its key is CTRL+E.
- **Military research**, the obvious gate, since RoN unlocks formations with
  it and every capture is an Ancient-age nation. run49 raised it (`tech who=0
  all on`, `military 5 0`) and photographed the same six buttons. Not the
  gate.
- **Binding the key.** This is the one that should have worked.
  `data/playerprofile.xml` is the keymap `KeyMap::init@007d5a90` loads, and it
  lists `FORM_LINE`, `FORM_REFUSED`, `FORM_ENVELOP`, `FORM_E_RIGHT` and
  `FORM_E_LEFT` as bindable actions **with no `<INPUT>` child on any of
  them** — the file has zero `<INPUT>` elements in total, so the formations
  ship unbound and that is why neither a key nor a button reaches them. The
  element's shape is fully recovered from
  `KeyMap::save_entry@007d4220` and `KeyMap::load_entry@007d43d0`, with every
  attribute name resolved out of `int_str_array` at stride 0x14 the way
  `console.py` reads it:

      <KEY enum="FORM_E_RIGHT" dependent="-1">
        <INPUT key="120" mouse="0" ctrl="0" shift="0" alt="0"/>
      </KEY>

  `key` is taken whole and then `ctrl` sets bit 0x20000, `shift` 0x10000 and
  `alt` 0x40000. `tools/gamelog/bindkey.py` writes exactly that into the
  **profile's** `<KEYS>` — `Player.dat`, which is user state and already
  edited with the game closed by `mapstyle.py`, never the install's shipped
  data — and `--restore` empties it again. run50 bound `FORM_E_RIGHT` to F9,
  pressed it twice (once with only the selection, once after the first march
  had made a group) and marched the group back and forth. **The formations in
  its dump are `Line x540` and nothing else.** The binding did not take, or
  the keystroke did not reach the action.

**Where the next attempt should start, and it is one question.** Is the
profile's `<KEYS>` read at all? `KeyMap::save_entry` writes an entry only when
its `dependent` is negative, so the profile is meant to hold the player's own
bindings — but the loader that would read them back is `KeyMap::load`'s
`String` overload at `007d39a0`, and **its caller has not been found**; the
only references the export shows outside `KeyMap` itself are unwind funclets.
Settle that and the rest follows: if the profile is read, the binding is
wrong in some detail; if it is not, the shipped `data/playerprofile.xml` is
the only keymap and the `<INPUT>` has to go there instead. The cheap
experiment either way is to rebind an action whose binding is **visible** —
`OPTION_AUTO_EXPLORE`, whose tooltip prints its key — and photograph the
tooltip: if it stops saying CTRL+E, the profile route works.

**What run50 is still good for**, and it is not a plain replication. It
reaches the mirrored layout on its own game — **870** group move orders
carrying `facing 1`, frames **337 to 395** — and then those orders **end with
no successor**: frame 396 holds none at all. So this order died by
*completing*, where run46's died by being *replaced*, and those are
`kill_current_order`'s two different ways in. Whether the hand-back's
arithmetic is the same on the completion path is **not** settled here: the
`GROUPDATA.facing` a frame prints is the whole 512-slot pool's, so the live
group's own value cannot be read off it, and run46's proof worked because the
*next* order's `facing` showed what `Form::compute` had been handed. With no
next order there is nothing to read it from. A capture that wants the
completion path needs a fourth click after the arrival.


## runs 53/54 — the same games, thirteen times as long, for eight minutes and 70 MB

Item 91's captures, and the first two stanzas to use `poll_max:`. One
24,000-frame run per map, the trace whole (`cover=1`) and the `[End Frame]`
detail cut to `MISC` alone, `[Start Game]` left at run10's exactly because
that block is what the harness stands the simulation up from.

| run | map | gamelog | trace | frames |
| --- | --- | --- | --- | --- |
| 53 | Great Lakes (14) | 10 MB | 25 MB | 24,001 |
| 54 | East Indies (18) | 10 MB | — | 24,001 |

**Both are the same games as run33 and run39**, and that is asserted rather
than assumed: `rngcmp.py` compares the `game_random` word of every `FRAME`
record and both pairs come back **0 differing over 1,851 overlapping
frames**. So run53 is a drop-in longer sibling of run10/run33 and run54 of
run38/run39.

**The measurement that matters here is the cost.** run33 is 1,850 frames and
took the better part of an hour; run53 is 24,000 and took **100 seconds**.
The sim is not the bottleneck and never was — unrendered under `!ffwd` it
runs at roughly 240 frames a second — it is the **per-frame dump**, 155 KB a
frame at run33's detail and rising with the roster. Cutting `[End Frame]` to
`MISC` removes the floor entirely. Two consequences worth writing down:

- **A trace-only capture of any length is nearly free.** Where a question is
  about the *stream* rather than a record, there is no reason to take a short
  one.
- **A full-detail dump over 24,000 frames would be hours and gigabytes**, and
  run53 is what says whether it is worth taking. It is not, yet: the word
  parts at **1802** on this capture exactly as it does on run33's 1,850, so a
  full-detail dump buys about thirty frames of new ground past run10's own
  1,772 and then twenty-two thousand frames of a stream that is nobody's.
  Size that capture to the word, and take it when the word has moved.

**The trap this pair found, before it cost anything.** `longtrace.sh`'s poll
loop was `for i in {1..160}` at twenty seconds — **53 minutes** — and its end
is not a graceful stop: it `pkill`s the game and archives whatever has been
written, which is a truncated capture that looks exactly like a finished one.
Every run to 52 fits inside the bound and none had reason to notice. `POLL_MAX`
is a hook now, `poll_max:` a stanza key, the default is unchanged at 160, and
the give-up path says out loud that its archive is truncated. `settle_min`
needed lowering for the same captures: a `MISC`-only per-frame block never
reaches the 10 MB the settle test defaults to, so the run would never have
been called finished.

## runs 51 and 52 — the profile IS read, and the key that answered it opened the chat box

The experiment run50's section asked for, run in two halves, and it closes the
`007d39a0` caller question from the behavioural side without reading another
line of the decompile.

**run51: the profile's `<KEYS>` is read.** `bindkey.py` wrote an `<INPUT>` for
`OPTION_AUTO_EXPLORE` — chosen because it is the one action whose binding is
**visible**, printed in its own tooltip — rebinding it from its shipped
CTRL+E to key 120. Hovering the button then photographs the answer:

| run | the same tooltip |
| --- | --- |
| run48, profile untouched | `Auto Explore: ON - ... (Hotkey: CTRL + E)` |
| run51, after `bindkey.py OPTION_AUTO_EXPLORE 120` | `Auto Explore: OFF - ... (Hotkey: F9)` |

So `KeyMap::load`'s `String` overload at `007d39a0` **does** run, whatever the
export shows about its callers, and `bindkey.py`'s element is right:
`<KEY enum=... dependent="-1"><INPUT key=... mouse=... ctrl=... shift=...
alt=.../></KEY>` in the profile's `<KEYS>`, exactly as
`KeyMap::save_entry@007d4220` writes it. **That is a tool the lane keeps**:
any action in `data/playerprofile.xml` can now be given a key with the game
closed, and the tooltip is how you check the game agrees.

**run52: and the keystroke arrives, but not at the action.** With the same
bind in place, hover, press, hover:

- before — `Auto Explore: OFF - Click to turn on Auto Explore. (Hotkey: F9)`
- `osascript ... key code 101`, the mouse parked off the card
- after — **the chat dialog is open** (`Chat`, `Chat Ally`, `Chat All`,
  `Close`), the game dimmed behind it, and Auto Explore still OFF.

So input reaches the game — something plainly happened — and it did not reach
the bound action. Two readings fit and this run does not separate them: either
F9 also opens chat and chat wins, or **macOS `key code 101` does not arrive as
F9 through CrossOver** and landed as the `Return` that opens the chat box (the
recipe's own table says the chat box is opened by `Return`). The second is the
likelier, and either way the lane's conclusion is the same.

**What this settles for item 23.** run50's `FORM_E_RIGHT`-on-F9 failure is no
longer a mystery and no longer evidence about formations at all: its two key
presses were opening a modal chat box, not asking for an Echelon. The bind
itself was fine. So the Echelon half is **not** blocked on the profile route,
which works; it is blocked on sending a key the game receives as the key it
was bound to.

**One operational trap, found by tripping it.** The game **rewrites
`Player.dat` when it quits**, binding included, so a `--restore` issued while
it is still running is undone a minute later. Restore after the process has
exited — which is where `runqueue.sh` leaves things — or the profile keeps a
binding nobody meant to leave behind.

**How the next attempt should send it.** Bind to a **plain letter** and send
it with `osascript ... keystroke "j"`, which is the path the recipe says
reaches the game, rather than a function key through `key code`. Then verify
before relying on it, in this order, because each step is cheap and the one
after is not: (1) the tooltip says the letter, so the game loaded the bind;
(2) pressing it toggles Auto Explore, so that letter arrives; (3) only then
rebind `FORM_E_RIGHT` to the same letter and look for `form 3` in the dump.
Steps 1 and 2 are a 250-frame run apiece and would have saved run50.

## run55 — the call proxies, and a function's own answer (2026-09-01)

The third instrument's third question. `tools/trace` could say **which
function drew** and **which functions ran**; it could not say **what a
function answered**, and no logger can: the dumps print state, a draw record
prints a seed, an `int 3` prints that something was entered. A function that
computes a number and hands it back leaves nothing behind.

`docs/PATHFINDER.md` §10 had recorded exactly that as a dead end — the
`PATHFINDER` gamelog category emits one line at map generation, the two
`dbg_*` printers are gated on a flag nothing writes, and
`PathFinderData::log_data` needs the `DUMP_ALL=1` that hangs the game. So
item 125's check was written down as an `int 3` on `calc_cost` and left.

**What it wanted was not a breakpoint but a proxy.** `rontrace.cfg`'s new
`callwin=LO-HI` replaces each listed function with a stub of its own
signature that logs the arguments, calls the original through the
displaced-prologue trampoline, and logs `eax`. The arguments live in the
proxy's own frame, so recursion and re-entrancy cost nothing, and the
callee-clean `ret <imm>` is copied from the listing. Two are proxied:
`PathFinder::astar_path@00683770`, whose entry and return **delimit one
search**, and `PathFinder::calc_cost@00684e50`. Without a `callwin` nothing
is patched at all, which is how every earlier capture stays reproducible.
`tools/trace/README.md` has the how-to and the three things a new site needs
from the listing.

**run55 is run39's game**: East Indies, `MAP_STYLE 18`, seed 12345, the
profile's lobby, run39's own `[End Frame]` detail, 1,500 frames, seventeen
minutes and 391 MB — with `cover=0` and `callwin=1460-1490`.
`tools/gamelog/rngcmp.py` says its `game_random` word is run39's on **all
1,501 overlapping frames, zero differing**, so the proxies cost the
simulation nothing and the capture inherits run39's siblings.

**What it found, in one reading.** Over the whole thirty-one-frame window
the game ran **one** search — the AI scout's, 110 `calc_cost` calls on
sim-frame **1476** — which made identification free. 103 of the 110 already
agreed with this crate's own answers for the same arguments. All seven that
did not were steps into the four cells under player 1's second city, and
they said the same thing twice: `+176` on each, and one refusal. `176` is
`20 × 9 − 4`, the terrain term for a cell nine of whose sixteen tiles are
built over, less the own-territory discount — so `WData.blocked` is a
**count of blocked tiles that a building raises**, not a property of the
map, and `World::set_blocked_at@006b4900` is its only writer. With that
kept, the two searches are identical call for call, and East Indies' whole
capture is matched: 1,851 ticks of 1,851, 1,850 order-frames of 1,850, no
player diverging anywhere.

**Two things worth carrying.**

- **The frame label bit again.** Every document before this one called the
  scout's search "frame 1477", from the dump block it lands in. The trace
  counts `Game::frame`, and the search is on **1476**. The first `calls`
  listing came back empty because of it.
- **A proxy is cheaper than it sounds and more general than it looks.** The
  encoding was verified against `llvm-mc --disassemble` before the game was
  ever launched, and a sixty-frame smoke run proved it before the
  seventeen-minute one. Any function whose *answer* is the question — a
  score, a predicate, a chosen index — is now one table row away.

## run56 — East Indies past its own word (2026-09-01)

The capture "The capture lane"'s standing rule owes: **when a map's word
crosses the newest full-detail capture it has, the next one is sized to the
word.** East Indies' word is run54's 2176 and its only full-detail run was
run39's 1,850, so every frame of item 85's divergence fell past the end of
the only dump that could show it.

run39's recipe unchanged and nothing else — East Indies, `MAP_STYLE 18`,
seed 12345, the profile's lobby with no `-config`, run39's `[Start Game]`
and `[End Frame]` detail, no input — carried to **3,000** frames. Thirty-six
minutes, **789 MB** of dump and 12 MB of trace, at about 1.5 sim-frames a
second; the per-frame block runs ~260 KB and rises with the roster. Only the
*length* changed, deliberately: `gatherers`, `gather_down` and
`non_flat_gather` are all already in run39's `BUILDDATA` at `BUILDS=7`, so
raising a category would have bought nothing and cost the sibling-hood that
lets both same-game tools speak.

**It is the same game twice over.** `rngcmp.py` against run54: **3,001
frames, zero differing**. `samegame.py` against run39: 1,850 frames in
common, **zero differing**. So it inherits run39's siblings and run54's word.

**What it settles.** The whole of `Build::find_gather_tiles`
(`docs/ECONOMY.md`, "The gather list, and its shuffle"). Frame 2176 is the
one camp the *game* builds in either capture — player 1's `o 2009`, a
Woodcutter's Camp at tile (198, 190), 48 tiles, `4 × 48 = 192` draws — and
its `BUILDDATA` record is what makes the shuffle's **order** checkable
rather than merely its count. Seed-anchored on the trace's own word at the
entry of 2176, this crate's camp comes back with the original's list entry
for entry.

**And what it named, and how long it lasted.** The gather half of the record
is now compared on every frame — and the *only* rows were that one camp and
the quit's own four. This simulation placed player 1's farm on frame 2, its
second city on 977 and its second farm on 1577, each at the original's own
tile and object number; then the original placed `o 2009` and this one placed
nothing. ~~The successor is an AI build decision.~~ **Closed the same day**
(`docs/AI.md` §19): `produce_building` scored a camp site by the forest tiles
in a one-tile ring rather than by what the site would gather, which is zero at
every site a camp can stand on. With `blocked_site`'s own out-parameter there
the camp goes up on frame 2176 at tile `(198, 190)` as `o 2009`, and run56's
gather comparison is 1,048,118 fields with nothing but the quit's four. East
Indies' long word went 2176 → **2665** on it.

## run57 — East Indies at 4,000 frames, and the capture the word did not need (2026-09-01)

run56's successor by the same standing rule that owed run56: East Indies'
word crossed the map's newest full-detail capture again — 2665 → **3021** on
item 128 — so the next one is sized to the word. run39's recipe unchanged and
nothing else (`MAP_STYLE 18`, seed 12345, the profile's lobby with no
`-config`, run39's `[Start Game]` and `[End Frame]` detail, no input),
carried to **4,000**. Forty-eight minutes, **1.07 GB** of dump and 12.8 MB of
trace, at about 1.4 sim-frames a second.

**It is the same game, twice over, and the tools said so before it was read.**
`rngcmp.py` against run54: **4,001 frames, zero differing**. `samegame.py`
against run56: 3,000 frames in common, **zero differing**. So it inherits
run39's siblings and run54's word, and it is a drop-in longer run56.

**And the frame it was taken for was answered without it.** Item 129's
divergence at 3021 is a blocked stand, and the first thing asked of run56 —
already on disk — was the collision block it is made of: 249,293 agreeing
unit-frames, zero disagreements. The seam turned out to be forty-four frames
upstream and in a record nothing had ever compared, `BUILDDATA`'s own
`x_internal`/`y_internal`: the AI's Dock `o 2010`, laid on frame 2977 two
cells south of the original's (`docs/AI.md` §20). The capture cost an hour of
screen and the answer cost a widening, which is the queue's own rule about
grepping the dump before booking a reading, one level up.

**What run57 does say, and it is worth having.** Past 3,000 the two games
have parted, and the parting is all one thing's consequence: seventeen units
first diverge from **2978** on, and the two later buildings — `1/2011` on
3177 (one tile of `y`) and `1/2012` on 3977 (eight tiles of `x`) — go up
after the citizen that builds them is already walking somewhere else. Nothing
in the extra thousand frames is independent evidence, which is exactly what
makes it useful: **when the dock lands, this is the capture that says what
is next**, and it needs no second run to do it.

**And it did, the same day.** The dock landed on 2026-09-01 (`docs/AI.md`
§21), and run57 became a test the hour after —
`diff::tests::run57_s_four_thousand_frames_stand_where_the_original_s_do`,
both position records over 4,000 frames. What it says now: **130,326
building fields with two buildings wrong** — `1/2011` on 3177, one cell east
in `x`, and `1/2012` on 3977, one tile — and **322,683 collision fields with
none wrong**, the fourteen units that ever leave the original's point all
leaving it at or after 3177. So the seventeen-unit consequence was the
dock's, as this section supposed, and what is left past 3,000 is one AI
placement and its own consequence. The word went 3021 → **3435** on the
same pair of fixes.

**And where its test stands after item 133** (2026-09-01). The building
half is **130,326 fields with one building wrong** — `1/2012` on 3977, one
tile of `y` — and the collision half **337,265 field-frames with none
wrong**, eleven units ever leaving the original's point, the first on 3582.
Both numbers moved on a change that has nothing to do with either: the
dock's gull, which took East Indies' word 3579 → 3608 and so moved where
the two streams part. The building assertion is now scoped to frames
**before the word** and the collision total is a floor rather than an
equality; see the marker below.

**And it answered a capture that had been booked against it** (2026-09-01).
`docs/TRANSPORT.md` §12's fourth check wanted a new `UNITS=3` window over
frames 3600–3640 to see the first boarding. run57 *is* that window: same
game, run39's detail, 4,000 frames. Blocks 3585–3609 hold the whole
mechanic — the AI scout `1/0` idle at `(40416, 34272)` through 3583, an
eleven-waypoint path and a `MOVE_TO` to `(35712, 25728)` on 3584, an order
list of `[CASTORDER spell 650 paid 0, MOVEORDER]` with a path top carrying
`flags 4` on 3608, and on 3609 the barge `1/14` at `(41112, 33695)`, guy
`type 320`, holding the scout's path with that flag cleared and the
scout's orders minus the cast, the scout itself `inside_up 14`. No screen
time; a `sed` range. The rule it illustrates is the queue's own, one level
up: **grep the dump before booking a capture, not only before booking a
reading.**

~~**`FABLE:` what may be asserted past the parting.**~~ **Ratified as
rescoped — and the ratchet declined** (Fable steering, 2026-09-01). The
re-read from the citations confirms the rescope is the rule run53/54's own
tests already state, applied to the one place it was missed. The ratchet is
declined on the evidence of the very next session: item 134 improved
fidelity — the route exact, the destination cell the original's — and the
past-the-word collision total *fell* 337,265 → 334,258 while the
off-position unit count rose eleven → fourteen. Past a parting the totals
move in both directions under unrelated *improvements*, so a ratchet fails
exactly when progress happens, and a guard whose failures teach
number-editing is not a guard. The standing rule for every score pinned
past a parting: **assert up to the word, print past it** — the printed
numbers stay visible telemetry, and the word itself is the only asserted
boundary. For one item this
capture's test asserted **zero** wrong building fields over all 4,000
frames, and that assertion was luck. `1/2012` on 3977 came back one tile
north the moment the word moved 3579 → 3608 on the dock's gull, a change
with nothing to do with it, and the choice was between reverting a
29-frame word gain and rescoping the test. **It was rescoped, in code** —
`build_bad` is filtered to `frame < LONG_WORD_EAST_INDIES` and `coll`
became a `>=` — so this is a code-changing verdict to re-read from its own
citations, not a proposal to weigh.

**Why a past-the-word assertion is luck, stated precisely, because the
obvious reason is the wrong one.** The harness does *not* free-run:
`Built::tick` installs the original's `game_random` word at the end of
every frame the dump carries a checksum for, and run57 is a per-frame full
dump, so both sides **start every frame on the same word**. What is not
reset is the position *within* a frame. From the first frame whose draw
sequence differs — 3608, `cast_transport` — this simulation spends a
different number of draws before the AI's own rolls, so every later draw in
that frame takes a value that is not the one the original took there, and
the state built from it persists. A placement 369 frames on is decided by a
roll that is nobody's. So the parting is not seed drift and cannot be fixed
by more re-seeding; it is the missing draws themselves, which is the main
loop's job and not a test's.

What a pass should weigh: **for** — run53's and run54's tests already say
out loud that past the parting "the totals there are coincidence that moves
with every unrelated change", and run57's was the one place that rule was
not applied, so this is consistency rather than new policy; the scope kept
is the half a shared stream backs, and both residues the test was written
around (`1/2011` on 3177) live inside it. **Against** — the strongest claim
this project has ever made about East Indies was "nothing is wrong in
either record over all 4,000 frames", it held for a day, and scoping to the
word means a real placement defect introduced past it now passes quietly;
worse, the scope *shrinks* whenever a capture is longer than the word,
which is every capture from here on. A third way nobody costed: keep the
whole range and make it a **ratchet** — assert `build_bad.len() <= 25` with
the offending building named — which fails on a new past-the-word defect
where a scope cut cannot, at the price of a number that has to be edited
every time the word moves. Whether a ratchet is a guard or a nuisance is
the question, and it applies to every score this repo pins past a parting.

~~**`FABLE:` the lane's own ordering rule, proposed and not adopted.**~~
**Adopted** (Fable steering, 2026-09-01) — the clause is in `CLAUDE.md`
beside "Grep the dump before booking a reading", worded to order the
*booking* and not the screen: an idle screen may still run the capture
lane, so the "against" (the lane's value is that it costs the loop
nothing) is preserved. The second instance sealed it: item 134's booked
`UNITS=3` window was answered outright by run57's blocks already on disk,
a `sed` range where an hour of screen was budgeted. This run
is the first evidence that "size the capture to the word" has the *order*
wrong rather than the size: the two records that answered item 129 were both
on disk, both parsed, and neither had ever been asserted, so the hour of
screen bought frames nobody needed yet. The proposal is a clause — **widen
every dumped record the mechanic touches before booking a capture, and take
the capture only for what no record on disk can answer** — and it belongs in
`CLAUDE.md`'s working agreement beside "Grep the dump before booking a
reading", which is the same rule one level down.

It is marked rather than written because it is a working-agreement change,
and those are the steering session's (`docs/DECISIONS.md` entry 22: Fable
writes `CLAUDE.md` and the queue). What a pass should weigh: **for** — item
129 is a clean instance, and item 128's own predecessor was found the same
way (`docs/QUEUE.md` item 87, the widening ledger, is the standing count of
records parsed and never compared); **against** — one instance is not a
rule, and the lane's value is that it runs *beside* the main loop and costs
it nothing, so an ordering clause that makes a capture wait on a widening
may spend the screen's idle hours rather than save them. run57 was not
wasted; it was second-best. Whether "second-best" is worth a rule is the
question.

## run58 — East Indies at 5,200 frames, and the lane running beside the work (2026-09-01)

run57's successor by the standing rule that owed it: East Indies' long word
had reached **4020** and run57's own dump stops at **4001**, so nothing on
disk reached the frame item 140 was about. run39's recipe unchanged
(`MAP_STYLE 18`, seed 12345, the profile's lobby with no `-config`, run39's
`[Start Game]` and `[End Frame]` detail, no input), carried to **5,200**.
Sixty-one minutes, **1.41 GB** of dump and 13.0 MB of trace, at about 1.4
sim-frames a second.

    POLL_MAX=260 zsh tools/gamelog/longtrace.sh 58 5200 islands-5k2 18

**It is the same game, twice over, and the tools said so before it was
read.** `rngcmp.py` against run54: **5,201 frames, zero differing**.
`samegame.py` against run57: 4,000 frames in common, **zero differing**. So
it inherits run39's siblings and run54's word, and it is a drop-in longer
run57.

**And the frame it was taken for was answered without it — while it ran.**
Item 140's divergence at 4020 was booked as the AI scout's second leg, and
it was the AI *citizen* `1/15`'s transport cast three hundred frames
downstream of two misread gates. What settled it was `rontrace-run54.log`
(the scout's own cast at 3608 spends two `Guy::set_anim` draws and 4020
spends one — one figure, so not the scout) and the decompile
(`Region::coast_here`, `Unit::move_step`), both of which cost minutes.
`docs/SYNC.md` §3.25.

**Which is the second capture in a row to be second-best, and the first to
say what the clause should be.** run57's `FABLE:` note above proposes
"widen every dumped record before booking a capture". run58 was booked
*correctly* by that clause — no record on disk reaches frame 4005 — and was
still not what answered the item. What both runs actually show is not an
ordering rule about the screen but one about the **model's attention**: the
capture lane is worth its wall-clock precisely because it does not consume
any, and the mistake would be to *wait* on it. run58 was launched in the
first five minutes of the session and read in the last ten; everything
between was reading and diffing. `CLAUDE.md` already says as much — "an
idle screen may still run the capture lane" — and this run is the evidence
for that half of the sentence rather than against it.

**What run58 says, and it is a test the same hour.**
`diff::tests::run58_s_five_thousand_frames_stand_where_the_original_s_do`:
**178,326 building fields, none wrong**, and **435,399 collision
field-frames** of which four are wrong — one unit-frame, `1/7` on 5084,
eight hundred frames past the word. Nineteen units first leave the
original's point between 4300 and 5085; **before the word only `1/13`
does**, at 3647, which is run57's own remaining residue (item 139).

**So its assertions are scoped to before the word, and this is the first
capture long enough for that to be the honest shape.** run57's test scopes
only its building half that way and says so under a `FABLE:` marker; run58
scopes all three — buildings, collision and the parting list — because past
4275 both sides are running on draws that are nobody's and a unit standing
somewhere else there is not a defect. The whole-capture numbers are printed
rather than pinned, so a reader can see the drift without the test pretending
to measure it. That widens the marked question rather than answering it: the
same third way is still available and still uncosted — a **ratchet on the
count** past the word instead of a cut to the range.

## run59 — the census window at East Indies' own word (2026-09-02)

The first `LEADERS=9` census that is not run10's game, and the first
resource level on disk past frame 800. East Indies' word is 5376 and the
frame is the AI's Market — eighty timber the original can pay and this crate
cannot — so the headline had become a number no capture measured.

    DETAIL_END="MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,DEATHS=1,LEADERS=9" \
    FRAME_WINDOW="5150 5400" SETTLE_MIN=250000000 POLL_MAX=200 \
    zsh tools/gamelog/longtrace.sh 59 5400 islands-census-5150 18

run58's recipe with two changes: `LEADERS=9` in the `[End Frame]` list, and
the **frame window** narrowing the per-frame dump to [5150, 5400).
`FRAME_WINDOW` is new — `censuswindow.sh` had the shape with run10's game
hardcoded, and `longtrace.sh` could only take the expensive `DUMP_ALL`
window, which at 250 frames would be tens of gigabytes.

**And the narrow window is what makes a late census cheap.** The frames
before 5,150 write no `[End Frame]` block at all, so the game reaches the
window in **under a minute** where run58, dumping every frame, took fifty.
Thirteen minutes and 446 MB, against run58's sixty-one minutes and 1.41 GB
for two hundred fewer frames.

One trap comes with it, and it is in the script's own comments now: the
start dump is ~150 MB and then **nothing grows until the window opens**, so
the poll's "stopped growing" test would call that quiet stretch a finished
run and kill the game two minutes in. `SETTLE_MIN` above the start dump's
own size is the guard.

**It is the same game.** `rngcmp.py rontrace-run54.log rontrace-run59.log`:
5,401 frames, **zero differing**. So it inherits run39's siblings and
run54's word, and its 251 blocks are a drop-in late window on run58's game.
`samegame.py` cannot speak here — no other capture dumps these frames.

**What it says** is in `docs/ECONOMY.md`, "The census at the word": 18,000
good-frames, 4,798 wrong in nine shapes, every one of them a standing level;
the AI is fifty timber short and its `leftover` agrees, so the fifty is a
lump; both players are short six timber gather slots and one wealth slot;
and the AI's food and wealth *rates* are wrong where the human's six are
exact. It also carries `production_step`, `script_step` and
`prod_script_run` on 250 frames 5,400 into the game — the AI's script is
still live, the machine never leaves step 1, and `economic.bhs` walks cases
23 → 15 → 18 (`docs/AI.md` §25).

## run60 — the census made cheap, and the whole curve at once (2026-09-02)

**The lesson is the recipe, not the run.** run59 narrowed an *expensive*
`[End Frame]` to a 250-frame window and cost thirteen minutes. run60 does the
opposite and it is better: keep the window open for all 5,400 frames and make
the **block** cheap instead.

    DETAIL_END="MISC,LEADERS=2" POLL_MAX=150 \
    zsh tools/gamelog/longtrace.sh 60 5400 islands-census-thin 18

`LEADERS=2` is where `LeaderData::log_data@006e5110` announces the encrypted
block — `bucket`, `leftover`, `resources`, `income`, `rate` and
`resource_cap`, per good — and `LEADERS=9` is where the ten-thousand-line
census sits. Dropping `UNITS`, `BUILDS`, `CITIES`, `GUYS` and `DEATHS` from
the end-frame list and keeping `LEADERS=2` leaves a per-frame block of a few
hundred lines:

| capture | frames | end-frame detail | wall clock | size |
| --- | --- | --- | --- | --- |
| run58 | 5,201 | run39's, `LEADERS=1` | 61 min | 1.41 GB |
| run59 | 250 of 5,400 | run39's, `LEADERS=9` | 13 min | 446 MB |
| **run60** | **5,400** | `MISC,LEADERS=2` | **under 5 min** | **67 MB** |

The start dump is unchanged (`DETAIL_START` defaults to run10's), so the
harness builds the same world from it and the run is a drop-in sibling.
`rngcmp.py rontrace-run59.log rontrace-run60.log` is 5,401 frames with
**zero differing**.

**What it says** is in `docs/ECONOMY.md`, "run60": 324,000 good-frames, and
the AI's whole bucket curve parts on exactly **three** frames in 5,400 —
frame 1 (item 156), 3579 and 4988. Those two were the dock's thirty wealth
and the goody box the thirty made pick the wrong good, and they were East
Indies' word.

**Two things this makes routine.**

- **A per-frame census is now cheaper than a window.** Where a question is
  about a *level* rather than a whole record, this is the capture to book —
  and its output is a curve, so the answer is a frame number rather than a
  standing gap.
- **Trace coverage fires once.** `report.py`'s `HIT` records mark a
  function's **first** entry, so a repeat of an event the trace already saw
  leaves no record at all. run60's box on 4988 is invisible to the trace for
  exactly that reason; the neighbouring `SpellType::cast_unpack` on 4988 is
  visible only because it had never run before. A coverage listing answers
  "has this ever run", never "did it run here".

## run61 — the record owner 9 never had (2026-09-02)

**A capture whose whole product is three call proxies.** run60's game
again — East Indies, seed 12345, map style 18, `[End Frame]` cut to
`MISC,LEADERS=2` — with `rontrace.cfg` carrying `callwin=0-5400` and three
new entries in `tracer.c`'s `CALLS`:

| proxy | what its record is |
| --- | --- |
| `Unit::do_air_physics@005e86d0(order, goal.x, goal.y)` | one flying unit's frame, **bracketed**: everything between its `CALL` and its `RET` is that unit's |
| `Unit::air_turn_speed@005ea390(sign, 0)` | the frame's turn rate, and so the bank angle — `bank_aircraft` is its only caller |
| `Unit::set_new_location@005f8d20(x, y, 0, 1)` | where the step landed |

    DETAIL_END="MISC,LEADERS=2" POLL_MAX=150 \
    TRACE_COVER=$'cover=1\ncallwin=0-5400' \
    zsh tools/gamelog/longtrace.sh 61 5400 islands-air 18

Under five minutes and 23 MB of trace, the same as run60. `rngcmp.py
rontrace-run60.log rontrace-run61.log`: **5,401 frames, zero differing** —
so a proxy costs the stream nothing and this is still run54's game.

**It is the oracle a whole class of question was waiting on.** A wild bird
belongs to owner 9, which no dump prints; before this, the only observable
its flight had was a single coin. run61 folds to **47,533 air frames** over
eleven flyers — goal, turn rate and landing position, per bird per frame —
and what it settled is in `docs/SYNC.md` §3.9, "The birth": `air.rs`
reproduces every wild bird exactly and the error was twenty-four position
units in the constructor. Both long words moved (East Indies 5437 → 5466,
Great Lakes 1802 → 2419).

**The general lesson is the bracket.** `set_new_location` is taken by every
unit that moves, so its records alone could not say which were a bird's;
proxying the *caller* as well makes the nesting the identity, and no guess
about `this` is needed. Any mechanic whose state is private to a class of
unit can be read this way — proxy the dispatcher and the mutator together —
and the cost is a five-minute capture rather than a reading.

## run62 — the road search's own prices (2026-09-02)

**run32's recipe unchanged, and three more proxies.** `roadcapture.sh` with
`RON_CALLWIN=99-101`: run10–14's game (Great Lakes, seed 12345), a Granary
dropped at tile (6, 171) and a Smelter at (33, 161) from the cheat channel
at sim-frame 100, the `DUMP_ALL` window still on [104, 109). Thirteen
minutes, 366 MB of dump, 9.4 MB of trace. `rngcmp.py` against run32: **111
frames, zero differing** — the third capture in a row where the proxies cost
the stream nothing.

| proxy | what its record is |
| --- | --- |
| `PathFinder::astar_caravan_road@00685990` | one road plan, entry to return |
| `PathFinderData::valid_roadcoord@00688740` | a candidate's **world coordinate**, and whether it was admitted |
| `PathFinder::calc_road_cost@00686300` | the node's **price** |

**The pairing is the instrument, not either half.** `calc_road_cost` takes a
pooled `PathNode *`, so its own record names an address out of
`Recycler<PathNode>::temp_pool` and no tile at all; the tile it prices is
the one the `valid_roadcoord` immediately before it admitted. That is
run61's lesson one map over — *proxy the dispatcher and the mutator
together* — and here the dispatcher is a predicate rather than a mutator.

Frame 100 carries 2 bracket calls, 3,028 gates and **2,913** prices, the
last exactly the frame's road-draw count.

**What it settled, inside an hour.** `docs/QUEUE.md` item 57 — two counts
that had stood 3 and 410 out since 2026-08-28, with every hypothesis a
reading could reach already ruled out by measurement. The first run said the
answer was the climb term and nothing else: the coordinates agreed for 19
nodes and eleven of the prices differed, **every difference a multiple of
three**. Three is `climb × 3`'s multiplier and nothing else in the cost is a
multiple of it. Two mechanics came out of that (`docs/ROADS.md` §7.3, §7.4)
and East Indies' word went **5466 → 5592**.

**The lesson, and it is the count's.** A count is not a sequence. Six
searches had matched the original's node *count* exactly, and the two that
did not had survived three readings, a height-grid chase and five
measurements — because nothing on the record could say *which node*. The
twenty-minute widening beats the reading again, and the shape of the
widening is now a table row.

## run63 — the site list either side of the word (2026-09-02)

**run58's recipe with the `[End Frame]` narrowed to a window, and
`LEADERS=9` in it.** East Indies, seed 12345, map style 18, the profile's
lobby, no input:

    DETAIL_END="MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,DEATHS=1,LEADERS=9" \
    FRAME_WINDOW="5430 5700" SETTLE_MIN=250000000 POLL_MAX=200 \
    zsh tools/gamelog/longtrace.sh 63 5700 islands-scoutwalk 18

Sixteen minutes, 482 MB of dump, 14 MB of trace, 271 frame blocks. `rngcmp.py
rontrace-run54.log rontrace-run63.log`: **5,701 frames, zero differing**, so
it is run54's game and run38's start dump stands it up.

| capture | frames dumped | end-frame detail | wall clock | size |
| --- | --- | --- | --- | --- |
| run58 | 5,201 | run39's, `LEADERS=1` | 61 min | 1.41 GB |
| run59 | 250 of 5,400 | run39's, `LEADERS=9` | 13 min | 446 MB |
| run60 | 5,400 | `MISC,LEADERS=2` | under 5 min | 67 MB |
| **run63** | **271 of 5,700** | **run39's, `LEADERS=9`** | **16 min** | **482 MB** |

**Why the window and not run60's thin per-frame block.** The question was
not a level but a **record**: which of `Leader::sites`' ten slots claimed
the region an AI citizen was standing in, and where every unit stood while
it did. That wants `UNITS=3` and `LEADERS=9` together, which is only
affordable over a few hundred frames. The frames before the window cost
nothing at all — the run reached 5,430 in under a minute, exactly as run59
reached 5,150.

**What it settled.** East Indies' word, 5592 → **5669**, and the item is
`docs/SCOUT.md` §11.1: `think_peasant`'s tail sends an idle AI worker off to
explore the instant it stands in a region none of its leader's ten sites
claims, and makes it wait six idle frames when one does. The original's list
gains the citizen's own cell on 5577 at `val 9728`; this crate scored every
cell of that region zero because `blocked_location` refused a first city
there with `COLONIZE 0x1c`. `COLONIZE_BONUS` is a technology's prerequisite
— `rules.xml`'s fourth `TECHBONUS`, Coinage — and the AI takes its Coinage
job on 5177, which is why the window is the earliest place on disk the
difference could have shown (`docs/CITIES.md` §2.6.1).

**Two lessons, and the second is the cheaper one.**

- **A caller offset in the trace is a predicate's answer.** The two
  `think_scout` call sites in `think_peasant` are `+0x2ac` and `+0x2ca`, and
  which one a frame spends says whether a site claimed that region —
  a fact about the AI's state read out of a *draw's return address*. Before
  booking the capture, that is what said the answer was the site list and
  not the walk.
- **Grep the disk first, and it half-answered this one for free.** run59's
  census has printed `Leader::sites` since 2026-09-02 and nothing had ever
  compared it: ten slots, six fields, 250 frames, sitting on disk. Fifteen
  minutes with it said the site *values* were wrong before the capture was
  booked, and that residue is now pinned rather than discovered twice.


## run64 — a caravan's road, and the world it reads (2026-09-02)

**run54's game with a `DUMP_ALL` window and the three road proxies.** East
Indies, seed 12345, map style 18, the profile's lobby, no input:

    DETAIL_END=MISC WINDOW="6164 6172" POLL_MAX=200 \
    TRACE_COVER=$'cover=1\nwindow=6163-6171\ncallwin=6163-6172' \
    zsh tools/gamelog/longtrace.sh 64 6180 islands-caravanroad 18

Twenty minutes, 563 MB of dump, 17 MB of trace, ten frame blocks.
`rngcmp.py rontrace-run54.log rontrace-run64.log`: **6,181 frames, zero
differing** — so it is run54's game, and a `DUMP_ALL` window and three
proxies together still cost the stream nothing. That is the fourth capture
in a row of which that is true, and it is now the assumption a capture is
designed on rather than a result each one re-earns.

**Why both halves.** The question was a road the crate spent no draws on at
all, and it needed two different things at once: the *sequence* — every
node the original priced, which only the `callwin` proxies carry — and the
*state the sequence reads*, which is the world at the frame before it. The
window is eight frames because that is the whole plan: `astar_caravan_road`
answers −1 on 6166–6169, parking its containers in the caravan each time,
and 1 on 6170.

| capture | frames dumped | end-frame detail | wall clock | size |
| --- | --- | --- | --- | --- |
| run62 | 5 of 110 | `DUMP_ALL` window | 13 min | 366 MB |
| **run64** | **10 of 6,181** | **`DUMP_ALL` window** | **20 min** | **563 MB** |

**What it settled**, in one afternoon: East Indies' word 6166 → **6169**,
and four separate things (`docs/CARAVAN.md`, `docs/ROADS.md` §7.4, §8).
The one worth naming here is the **height grid**, because it is the
capture-design lesson: a `DUMP_ALL` block carries
`master_land_heights` (`Log::frame_heights`), and comparing it whole said
in one run that 182 tiles of it were this crate's own — the AI's farms
terraforming ground the original leaves alone. Nothing shorter than the
whole record would have found it; the road only reached one of the 182.

**And a trap that cost the archive.** The capture was launched
`run_in_background` through `| head -20`, which closed the pipe after the
lobby lines and killed `longtrace.sh` mid-poll — the game ran on to its
own `!quit` and finished, but nothing archived it. A capture's driver must
not be piped into anything that exits early; redirect to a file.

## run65 — the caravan's turn out of its own city (2026-09-02)

**run54's game with an eighteen-frame `DUMP_ALL` window.** East Indies,
seed 12345, map style 18, the profile's lobby, no input:

    DETAIL_END=MISC WINDOW="6196 6214" POLL_MAX=250 \
    TRACE_COVER=$'cover=1\nwindow=6195-6212\ncallwin=6195-6213' \
    zsh tools/gamelog/longtrace.sh 65 6220 islands-caravanturn 18

Thirty-seven minutes, 1.19 GB of dump, 15 MB of trace, twenty frame
blocks. `rngcmp.py rontrace-run54.log rontrace-run65.log`: **6,221
frames, zero differing** — the fifth capture in a row for which a window,
a coverage window and the eight call proxies together cost the stream
nothing.

**The window is two frames wider than the question.** The queue asked for
`[6196, 6212)`; a `FRAME n` block is the end of sim-frame `n − 1`, and the
two sides' *first walking frames* are the whole point, so the window has
to reach past the later of them rather than stop on it. `[6196, 6214)`
puts both inside with a frame to spare, for 108 MB and four minutes.

**What it settled.** East Indies' word 6207 → **6353**, and the answer was
in `docs/MOVEMENT.md` rather than in `docs/CARAVAN.md`: `Unit::move_step`
asks `UnitData::invalid_loc` about any step that changes tile and drops
the step whole when it is refused. Both `docs/CARAVAN.md` §8's guesses —
the turn, and the detour `do_move` plans — were wrong, and the window
refuted them in one reading by showing the *same* detour node, the same
bearings and the same computed step on both sides.

**The capture-design lesson is the one field that carried it.** The unit's
`angle` and guy 0's `angle` are two different things — the heading
`set_angle` writes, and the facing `Guy::do_turn` chases it with — and a
reader that takes the first for the second concludes the original turns
instantly. The `GUY` block's own `last_speed`/`avg_speed` are what make
the turn rate checkable frame by frame, and they are why the eight
bearings could be shown to agree before the step was looked at.

**And a probe trap worth naming.** A flat key/value sweep of a `UNITDATA`
block reads `angle` three times — the unit's, the `MOVEORDER`'s and every
`GUY`'s — and the last one written wins. Nesting in these dumps is
**indentation**, and a probe that ignores it will silently answer with the
wrong field; the first reading of this capture did, and said the original
snapped its facing in one frame.

## run66 — the merchant's whole walk, and the cheap window's day (2026-09-02)

**run54's game with the cheap per-frame dump narrowed to `[6340, 6600)`.**
East Indies, seed 12345, map style 18, the profile's lobby, no input:

    DETAIL_END="MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,DEATHS=1,LEADERS=1" \
    FRAME_WINDOW="6340 6600" SETTLE_MIN=250000000 POLL_MAX=200 \
    zsh tools/gamelog/longtrace.sh 66 6620 islands-merchantwalk 18

**Four minutes and 89 MB**, 261 blocks. `rngcmp.py rontrace-run54.log
rontrace-run66.log`: **6,621 frames, zero differing** — the sixth capture in
a row for which a window costs the stream nothing.

**Compare the shapes.** run65 asked for eighteen frames at `DUMP_ALL` and
paid 37 minutes and 1.19 GB for them. run66 asked for **260** frames at
run39's `[End Frame]` detail and paid four minutes and 89 MB. The rule that
falls out is run60's, one level up: **narrow the window when the question is
a whole record, cheapen the block when the question is a field over time** —
and a *walk* is a field over time. Positions, angles, path stacks, order
stacks and both guys of every unit are all in the cheap block; only the
animation clocks are not.

**What it was booked for.** East Indies' word parted at 6570 on a collision
the original does not have, and nothing on disk covered the frame — run64
and run65's windows both end at 6221. The geometry was a knife edge (the
merchant clears the citizen by 160 units against a 144-unit block sum), so
reading was never going to settle it.

**What it settled** is `docs/COLLISION.md` §4.2's **fast path**: with
`nocoll` clear and a proposal exactly one cell away on one axis,
`CollCheck::collide_here` sweeps the leading edge alone, which is a strict
subset of the disc and therefore stops at a *different* first hit cell — and
the corner rule is decided on the cell. Word 6570 → **6571**, and the
merchant's whole walk, its collision, its centre snap and its recovery are
now an assertion (`run66_s_window_is_the_original_s_unit_for_unit`, 12,094
fields).

**Two capture-design notes.**

- **`SETTLE_MIN` is a ceiling as well as a floor.** run59's 250 MB was
  copied without thinking; run66's whole log is 89 MB, so the poll could
  never call it settled and would have run its full 200 polls — 66 minutes
  — after a four-minute capture. Set it above the *start dump* and below
  the finished log, not to the last run's number.
- **The dump's own `collide_o` is what named the cell.** The reading had
  three candidate colliders and no way to choose; `collide_o 11` on the
  frame after the collision picked one, and the geometry of that one is
  what the fast path had to explain. A record's own field beat two hours
  of listing.

## run67 — the whole `GuyData`, without `DUMP_ALL` (2026-09-02)

**run54's game with the cheap window narrowed to `[6545, 6605)` and `GUYS`
raised from 2 to 4.** East Indies, seed 12345, map style 18, the profile's
lobby, no input:

    DETAIL_END="MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,DEATHS=1,LEADERS=1" \
    FRAME_WINDOW="6545 6605" SETTLE_MIN=20000000 POLL_MAX=60 \
    zsh tools/gamelog/longtrace.sh 67 6620 islands-crewclocks 18

**Four minutes and 43 MB**, 61 blocks. `rngcmp.py rontrace-run54.log
rontrace-run67.log`: **6,621 frames, zero differing** — the seventh capture
in a row for which a window costs the stream nothing.

**The third capture shape, and it is a category rather than a window.** Every
`[End Frame]` category carries its own detail, and `GuyData::log_data`
(`005de6c0`) switches detail four times — the calls to the log's vslot
`0x28`. `GUYS=2` is the nine lines every capture since run10 has taken;
**`GUYS=4` is the whole record**: `des_x`, `des_y`, `des_angle`, `cur_time`,
`end_time`, `last_time`, `cur_anim`, `stopped`, `guy_flags`, `guy_num`,
`gpiece`, `track_dx` and `track_dy`. Until this run, a figure's clock had
only ever been read inside a `DUMP_ALL` window — and `DUMP_ALL` is what
run65 paid thirty-seven minutes and 1.19 GB for eighteen frames of. Sixty
frames of the same fields cost four minutes here.

So the rule now has three arms, not two: **narrow the window when the
question is a whole record; cheapen the block when the question is a field
over time; and raise one category's detail when the question is one
record's own fields.** The third is far cheaper than the first, and
`grep -n "0x28))(" ` over a record's `log_data` is how to find out whether
it is available.

**What it was booked for.** East Indies' word parted at 6571 on the merchant
crew's second `Unit::move_step+0x823` draw, and nothing on disk carried a
crew figure's `des` or its clock. The reading had run out: the geometry said
the figure was standing on its destination and therefore had to roll, and
the original did not.

**What it settled** is three findings that are one mechanism — the crew loop
`Guy::set_angle` and `Guy::set_new_location` share (`docs/MOVEMENT.md`, "Who
writes it, and when"): `Unit::set_angle` rewrites the crew's `des` from the
**heading** at the top of every `move_step`, which is what keeps a figure
off its destination on the frame a bearing moves; the cell-centre snap
*teleports* the crew rather than leaving it to walk; and the walk slot is
resolved from the **asked guy's own** average speed, so a tracked figure
jogs where its leader walks. Word 6571 → **6574**, and every `GuyData` field
of every figure over sixty frames is now an assertion
(`run67_s_window_is_every_figure_s_whole_record`, 13,545 fields).

**One capture-design note.** `SETTLE_MIN=20000000` was chosen from run66's
own numbers rather than copied: the start dump is 11 MB and a sixty-block
window at `GUYS=4` was never going to be under 20. The poll settled four
polls after the last frame, as designed. That is the second half of run66's
lesson working.

## run68 — the window either side of the word, and what the quit block is not (2026-09-03)

**run54's game with the cheap window over `[6595, 6730)` and `GUYS=4`** —
run67's recipe with the window moved and widened. East Indies, seed 12345,
map style 18, the profile's lobby, no input:

    DETAIL_END="MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,DEATHS=1,LEADERS=1" \
    FRAME_WINDOW="6595 6730" SETTLE_MIN=20000000 POLL_MAX=60 \
    zsh tools/gamelog/longtrace.sh 68 6745 islands-citizenword 18

**Five minutes and 83 MB**, 136 blocks. `rngcmp.py rontrace-run54.log
rontrace-run68.log`: **6,746 frames, zero differing** — the eighth capture in
a row for which a window costs the stream nothing.

**One capture for two items, which is the booking lesson.** Items 188 and
189 wanted `[6600, 6620]` and `[6710, 6720]`; 135 blocks covers both and
costs a minute more than sixty would. A window is priced by its *blocks*,
and the frames before it are free — so when two open items sit within a few
hundred frames of each other, the span between them is nearly free to buy.

**The quit block is not a frame state, and two captures say so.** The
`GameInfo closing` dump is written at shutdown and labelled with
`game->frame`, and until this run nothing could check it, because no capture
carried an *ordinary* block with the same label. run68 does. Dump against
dump, with no simulation involved:

| compared | units | differ |
| --- | --- | --- |
| run66's closing 6621 vs run68's ordinary **6620** | 131 | 5 |
| run66's closing 6621 vs run68's ordinary **6621** | 131 | **1** |
| run67's closing 6621 vs run68's ordinary **6621** | 131 | **1** |

The single disagreement is the same unit in both — `0/5`, the human's one
*moving* citizen, which the closing block holds at its 6620 position while
every other unit, five of them mid-step, is at 6621. Two independent
captures, one unit. So a closing block is block `n` for almost everything
and one tick behind for at least one unit. ~~And the harness scores no unit
position in it.~~ **It does, since 2026-09-07**: the lag is decidable,
because a torn unit is on the simulation's own `n - 1` position — see "The
shutdown dump" below, and `rondata::diff::compare_shutdown`. Why the quit
catches the unit it does is still unread; the fix does not need it.

It had cost something already: `run66_s_window_is_the_original_s_unit_for_unit`
excepted `0/5` by name for a day as queue item 188, on the strength of a
number the closing block had no business supplying.

**What else it settled.** The first field of the whole record to part is
`1/19`'s `orders_x/y` on block **6714** — a frame *ahead* of the draw
stream's own 6715, which is the argument for diffing fields and not only
draws. The merchant reaches its `CITRUS` and runs `find_merchant_spot`'s
ring, which no capture had ever reached (`docs/MERCHANT.md` §3). **Closed
the next day**: both sides pick the same tile, and what parted was the
*queue position* of the walk that ring orders — `unpack_merchant`'s tail
rotates it in front of the unpack cast (`docs/MERCHANT.md` §3.1). The
block's order list is what says so, and it says it three ways at once:
`orders_x/y`, `dest_angle` and the `MOVEORDER`'s own row. With it right
the window holds to **6718** and the word to 6739. And
`stance` is 1 on every unit here against 0 on every unit there, from the
window's first block — a field no capture had compared on a unit this crate
created, and the widening ledger (`docs/DATALAYER.md` §4) is what named it
as a single-capture field the day before the capture landed.

**122,752 fields over the window's first 123 blocks, zero differing**, with
`stance` and one unit's two path waypoints excepted by name
(`run68_s_window_is_every_unit_s_whole_record_to_the_word`). It was 118,948
over 119 while the merchant's arrival was open.

## run69 — Great Lakes past its own word, and the map that had been standing still (2026-09-03)

The capture lane's standing rule (`docs/DECISIONS.md` 29) owed this one two
days before it was taken: **when a map's word crosses the newest
full-detail capture it has, the next one is sized to the word.** Great
Lakes' word is run53's **2419** and its only full-detail run was run33's
1,850, so every frame of the parting fell past the end of the only dump
that could show it — the same shape that owed run56 on East Indies.

run33's recipe unchanged and nothing else: `MAP_STYLE 14`, seed 12345,
run10's `-config check.ini` lobby, run10's `[Start Game]` and `[End Frame]`
detail, no input, carried to **3,000** frames. Twenty minutes, **468 MB**
of dump and 10 MB of trace, at about 2.5 sim-frames a second. Only the
*length* changed, so it is a drop-in longer run33 and both same-game tools
speak.

**It is the same game twice over.** `rngcmp.py` against run53: **3,001
frames, zero differing**. `samegame.py` against run33: 1,850 frames in
common, **zero differing**. So it inherits run10's siblings and run53's
word.

**It was launched in the session's first five minutes and read in its
last** — the lane the 09-03 steering pass named, a background shell rather
than a second agent, and the diagnosis was done off run53's trace while it
ran.

**What it settles**, and it is the whole of item 193. The word parts at
2419 on one draw, the AI woodcutter `1/9`'s return-to-camp stand, and the
clock behind it is exact on both sides — so the frame is a **walk**, and
this capture is the first Great Lakes dump that carries the walk. `1/9`'s
`MOVEORDER` waypoints are the original's on every frame of the game until
**1993** and then run 48 short in x for two middle legs; it reaches its
tree on 2015 where the original reaches it on 2016
(`docs/PATHFINDER.md` §17, `docs/SYNC.md` §3.26).

**And what the widening said beside it.** Of the capture's fourteen units
that ever leave the original's point, thirteen part between 2467 and 2930
— all past the word, where both streams are on draws that are nobody's.
`1/9` parts 474 frames earlier than any of them. The collision block is
**228,821 field-frames with none wrong**; the buildings are 95,476 fields
with nothing wrong before the word and one row after it — `1/2010`'s
`y_internal`, this crate's four tiles south of the original's from 2577,
which is an AI placement past the parting and not this capture's business.

## run70 — the woodcutter's own search, and the cell the original refuses (2026-09-03)

The check `docs/PATHFINDER.md` §17 named and could not run off disk, and
the second capture to use the call proxies after run55. run69 had said that
Great Lakes' word was a **route** — the AI woodcutter `1/9` turning one
48-grid step early and reaching its tree a frame ahead of the original's —
and a day of reading said the search that builds it was right in every part
a decompile can check: `get_estimate@00688310` is `vector_dist × 10`, the
unit grid's `calc_cost@00684e50` is a flat 32/40 (the `param_6 == 0x30` arm
returns before every terrain term), `Tree::ordered_insert@004796f0` puts an
equal `value` left and `remove_current@00479770` keeps the in-order, and
the wheel is `pref + 1 … pref + 8`. Either the original refused a cell this
crate accepted, or something no reading had found.

**`calc_cost` is called only for a neighbour that passed `valid_ucoord`**,
so the proxy's argument list *is* the validity filter's answer, one row a
cell — and that is what makes a `callwin` the instrument here rather than a
breakpoint. run53's recipe with the trace cheap (`cover=0`) and the dump
thin (`end: MISC`), `callwin=1955-1985`, 2,000 frames: **three minutes and
10 MB**, against the twenty and 468 MB run69 cost. `rngcmp.py` against
run53: 2,001 frames, **zero differing**, so it is run53's game to the frame
and the window is the frame it claims to be.

**One cell.** Laid side by side, the original's twenty-one expansions and
this crate's twenty-two agree on every cell either probed but
`(849, 366)` — `(40776, 17592)`, the node this crate turned south onto,
which the original refuses from all three neighbours that reach it and
never prices at all. Nothing else: not a price, not an order, not a
direction.

The refusal turns on `(848, 367)`, a corner of the *standing* gatherer
`1/10`'s block that `1/9`'s own diagonal step had cleared eighty-one frames
earlier — the occupancy index is not refcounted and never has been. What
puts it back is `Guy::process@005e0230`: every guy standing still
(`avg_speed == 0`) re-marks its whole disc on the frames where
`(game->frame + o) % 64 == 0`, so `1/10` healed the hole on frame 1910 and
the original's search saw a wall where this crate saw a gap
(`docs/COLLISION.md` §2.2).

**What it bought.** Great Lakes' long word **2419 → 2808**; run69's
collision record 228,821 → **247,543 field-frames with none wrong**; its
buildings 650 rows wrong → **none, over the whole three thousand frames**;
eleven units ever off position instead of fourteen, and the earliest at
2804 instead of 1993. On East Indies the same one line closed item 191's
other half: run68's window compares `1/13`'s stack whole with the exception
deleted, and holds to **6730**, the last block that capture carries.

**Three minutes.** That is the number worth carrying beside run69's twenty
minutes: a question about *what a function answered* does not need a
full-detail dump at all, and the thin-`end:` recipe runs 2,000 frames in
the time a settle poll takes. The capture lane's cheapest instrument is the
one that had been used once.

## run72 — Great Lakes' word, node for node (2026-09-03)

run64's instrument one map over, and the capture that owed Great Lakes'
own word. `MAP_STYLE 14`, seed 12345, run10's `-config check.ini` lobby,
no input, to 4,810 frames, with a `DUMP_ALL` window on `[4800, 4806)` and
the three `docs/ROADS.md` §7.2 proxies over `[4799, 4807]`:

    DETAIL_END=MISC WINDOW="4800 4806" POLL_MAX=200 \
    TRACE_COVER=$'cover=1\nwindow=4799-4807\ncallwin=4799-4807' \
    zsh tools/gamelog/longtrace.sh 72 4810 greatlakes-marketroad 14

Fourteen minutes, 430 MB of dump, 12 MB of trace, eight frame blocks.
`rngcmp.py rontrace-run53.log rontrace-run72.log`: **4,811 frames, zero
differing** — the fifth capture in a row for which a window and three
proxies cost the stream nothing.

**Why it was booked.** Great Lakes' word is **4803** and its position
parting **4827**, and the queue's item 201 had booked the second: `1/15`
re-picking a different cell of its own farm. The re-pick is not wrong.
The frame's two `GameAccess::rnd(4)` draws read 26899 and 16738 — `% 4` is
(3, 2), which is the tile the original walks to — and this crate reads
different numbers because the *stream* parted twenty-four frames earlier.
Frame 4803 is **277 `PathFinder::calc_road_cost` draws against this
crate's 266**, one road search: player 1's Market `o 2015` finishing and
planning its road to London. Everything from 4809 on, 4827 included, is
downstream of it.

**What it settled, in one run, and the shape is run62's exactly.** A count
is not a sequence. The node records agree for **80** nodes and part on the
**81st** — the same tile, the same direction, priced 387 here against 27 —
and the block's `master_land_heights` said why before the sequence did:
**62 tiles** of the Market's own ground were still the map generator's
here where the original had already flattened them. The terraform belongs
to `Wall::init`, not `Wall::start` — the same frame for a building placed
and started at once, **226 frames apart** for one the AI builds
(`docs/ROADS.md` §7.6). With it moved, the height grid is the original's
on all **921,600** tiles.

**And what is left is a mechanic, not a residue.** Node 81's tile
`(223, 79)` is a road in the original and plain ground here, and it is not
the ring: this crate lays the Market's sixteen ring tiles exactly, and the
original lays a seventeenth. `World::set_road_at` ends in
`Roads::road_added` → `Roads::add_roads` → `Roads::set_diags`, the road
*mesh* builder, which fills the corner between a new road tile and one
already standing. `docs/ROADS.md` §9 is the reading, and this capture is
its oracle — already on disk.

## run73 — Great Lakes' first window, and the caravan that is born in it (2026-09-03)

The map that carries the headline had never had a `DUMP_ALL` window. Its
word was **5571** and the two draws that part it are a caravan's crew
figures wrapping an animation, so the question was a *clock* — and below
`DUMP_ALL` the per-frame `GUY` blocks are empty. run65 is the same question
on East Indies and answers a different configuration of it. Nothing on disk
could speak.

`MAP_STYLE 14`, seed 12345, run10's `-config check.ini` lobby, no input, to
5,590 frames, with a `DUMP_ALL` window on `[5564, 5580)` and the three
`docs/ROADS.md` §7.2 proxies over `[5563, 5581]`:

    DETAIL_END=MISC WINDOW="5564 5580" POLL_MAX=220 \
    TRACE_COVER=$'cover=1\nwindow=5563-5581\ncallwin=5563-5581' \
    zsh tools/gamelog/longtrace.sh 73 5590 greatlakes-caravanstart 14

Thirty-three minutes, **1.05 GB** of dump and 17 MB of trace, sixteen frame
blocks. `rngcmp.py rontrace-run53.log rontrace-run73.log`: **5,591 frames,
zero differing** — the sixth capture in a row for which a window and three
proxies cost the stream nothing.

**Why it was booked, and what the greps had already ruled out.** The
cadence said the original's caravan was walking where this crate was still
turning: crew wraps at 5569, **5572**, 5575 against 5569, **5571**, 5574,
and a standing caravan's crew wraps on alternate frames where a walking
one wraps every third (run65's 6202/6204/6206/6208 is the standing shape).
Two readings of `move_step` were tested off disk and both failed: the
near/far test measures the Manhattan distance to the **waypoint**
(`MoveOrder+0x2c`) and not to the order's destination — putting the
destination there collapsed the word from 5571 to **307** — and the
`unit_flags & 0x20` arm that skips the turn-in-place block is the
**helicopter** bit. run72's own block, already on disk, said the tiles were
not it either: `(227, 85)` is `0x6103` on both sides, a building, and the
caravan's own `(228, 85)` is `0x2113`, so both sides' marches are refused
at the same tile and both detour.

**What it settled, in one field.** Guy 0's clock is this crate's on every
frame of the window — the driver never differed. The crew's parts once, on
5571, the frame the caravan first walks: `cur_anim 8, cur_time 2,
last_time 1` against this crate's `cur_anim 0, cur_time 0, last_time −1`. A
`last_time` of 1 says the clock stood at **1** before that frame's step, and
the figure came off the mirror at **4** — the length, 3, taken off.
`Guy::set_anim`'s walk arm only subtracts for the slot already playing, so
one call could never produce a 1. There are two: `Unit::move_step` asks
every guy for `CHAR_WALK` immediately before `set_new_location`, and
`Guy::move` asks again in the body follow. §4.8 of `docs/ANIM.md` had cited
that call for a year of sessions and the implementation never made it
(`docs/ANIM.md` §4.9).

**And what the window measures beside it.** All **4,869** `GUY` fields of
every player unit over sixteen frames — `cur_anim`, `cur_time`, `end_time`,
`last_time`, `gpiece`, `stopped`, position and angle — are the original's,
with no unit of the dump this crate has none for. Without the call the
check fails on the window's *first* frame and on the human's units, not
only the caravan: **every guy that walks carried the wrong clock.** The
window also covers a unit's whole birth — `1/23` is trained on 5564, past
run71's 5,000 frames, so no capture had ever checked a mid-game unit's
first three frames. Great Lakes' word **5571 → 5573**.

**What it leaves loaded.** The frame past the new word is a **road**: the
caravan's own `Caravan::build_road`, 1,761 `PathFinder::calc_road_cost`
draws here against the original's 1,535. The `callwin` covering
`[5563, 5581]` carries `astar_caravan_road`, `valid_roadcoord` and
`calc_road_cost` for exactly that search, node for node — so the successor
item's oracle was taken by the same run, before the item existed.

## run74 — the cheap window, and the rare the harness never offered (2026-09-04)

Great Lakes' word was **5786** and the draw was a single
`Guy::set_anim+0x97a < Guy::do_turn+0x4a < Unit::move_step+0x389` — the
**far** turn-in-place arm, whose idle roll only a guy with `guy_flags & 8`
and no `CHAR_TURN_RIGHT` in its packet ever pays. It is the only turn draw
either side spends in the whole 5,800 frames. Six units were moving on that
frame and exactly one **packs**, so the unit was named before the run was
booked: the AI's Merchant `1/24`, `docs/ANIM.md` §4.8's own row.

**The window is the cheap one, and that is the point.** The question was a
position, a facing and a path stack, all of which `UNITS=3` writes, so
`frame_window` narrows run33's ordinary `[End Frame]` detail to
`[5700, 5800)` instead of turning `DUMP_ALL` on:

    DETAIL_END=MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,DEATHS=1,LEADERS=1 \
    FRAME_WINDOW="5700 5800" SETTLE_MIN=14000000 POLL_MAX=200 \
    TRACE_COVER=$'cover=1\nwindow=5699-5800\ncallwin=5699-5800' \
    zsh tools/gamelog/longtrace.sh 74 5800 greatlakes-merchantturn 14

**Three minutes and 31 MB** against run73's thirty-three and 1.05 GB, for a
window six times as long — the run-up costs nothing at all, so the game
reaches 5,700 in under a minute. 101 frame blocks.
`rngcmp.py rontrace-run53.log rontrace-run74.log`: **5,801 frames, zero
differing**. `settle_min` is the one number that needs care: this map's
start dump is 10.9 MB at this detail and nothing grows until the window, so
the default 10 MB floor would call the quiet run-up a settled run — 14 MB
sits above the start dump and below the finished file.

**What it settled, in one field.** The two merchants were walking to
**different rares**: the original's order is `MOVE_TO (40344, 14232)` on a
seven-node path north-east, this crate's `(31800, 21816)` on a sixteen-node
path across the map. `Unit::think_merchant` scores `LeaderData::new_rares`
with `base = 200 − 10 · position`, and the original's list held three goods
where this crate's held two — the missing one first, and so the winner. It
is the harness's, not the simulation's: `build_sim` *installs* the dump's
`seen2` grid, and `Sim::reveal_fog` is reached only from `World::set_seen`
answering that `seen2` **changed**, so every offer the original made before
the block was written was skipped, unrecoverably. `docs/ECONOMY.md`'s "The
rares a leader has seen" had said "recorded during `Setup`, before frame 0"
since 2026-09-02; nothing acted on it.
`Sim::seed_new_rares_from_fog` replays them. Great Lakes' word **5786 →
6080**, and the window's hundred blocks carry no order, path or angle
disagreement at all.

**The stall that cost four launches, and it was a permission after all.**
`wineserver`'s main thread sat in `open()`, 1770 samples of 1770, with the
game at 0.1 % CPU and `waitwin.sh` spinning — run72's "cold bottle"
signature. It was **TCC**: the bottle's
`drive_c/users/crossover/Documents` is a symlink to `~/Documents` and the
game opens `My Documents\My Games` at startup, so a Documents prompt raised
earlier in the session by an unrelated `ls` blocked every launch behind it.
`log show --predicate 'subsystem == "com.apple.TCC"'` showed **no denial**,
because an unanswered prompt neither denies nor returns — the tell is the
silence plus the blocked `open()`, not a `denied` line.

## run75 — the scout's walk down the river (2026-09-04)

Great Lakes' word was **6080** and the frame was 41 draws the original
spends none of: the AI scout `1/0` arrives at its explore target, goes idle
and runs the whole of `Unit::think_scout` where the original's is still
walking. The scout was not the mechanic, and nothing on disk could say what
was — run53's dump is checksums, its 179 `UNITDATA` records are all in the
start block, and the walk itself spends no draws at all, so both sides' 229
frames of it were invisible.

**run74's recipe, six hundred frames later and three times as long.** The
question was again a position and a path stack, so `UNITS=3` and the cheap
window rather than `DUMP_ALL`:

    DETAIL_END=MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,DEATHS=1,LEADERS=1 \
    FRAME_WINDOW="5845 6160" SETTLE_MIN=14000000 POLL_MAX=200 \
    TRACE_COVER=$'cover=1\ncallwin=5844-6160' \
    zsh tools/gamelog/longtrace.sh 75 6160 greatlakes-scoutwalk 14

**Five minutes and 75 MB**, 316 frame blocks.
`rngcmp.py rontrace-run53.log rontrace-run75.log`: **6,161 frames, zero
differing**.

**What it settled, in one field.** Both scouts take the same `EXPLORE_TO`
to the same cell on the same frame — 5851, `(40440, 30456)`, an eight-node
path the capture prints node for node identical — and the original's
arrives seventy-one frames later. The seventy-one frames are a **speed**:
`z_internal` reads 14 on 5948, **0** on every frame from 5949 to 6089 and
17 on 6090, and the per-frame step is 34 outside that span and **17**
inside it, on a `myspeed` of 34 throughout. That is
`UnitData::get_speed@00608720`'s land arm — a tile carrying `0x800`
(`WorldData::is_river`) halves a land unit's step while its own `z` is not
above zero — and `crates/sim` had none of that function's third layer.
With it the scout is on the original's point for all 315 frames of the
window and the word runs **6080 → 6151** (`docs/MOVEMENT.md`, "The river
halves a land unit's step").

**The launch is the driver's, not this process's.** `longtrace.sh` refused
at the permission probe — "Screen Recording is off" — which is a Claude
Code update having moved `~/.local/share/claude/versions/<VERSION>` out
from under the grant. `zsh tools/gamelog/viadriver.sh <script>` hands the
run to `~/bin/RonDriver.app`, a fixed path that holds all three
permanently, and it went first time. Re-granting the version path by hand
buys one session; the bundle is the standing answer.

## run77 — the frame that was not a birth, and the instrument that dates one (2026-09-04)

**What it is.** East Indies, seed 12345, 251 frame blocks over `[10150,
10400)` at run10's `[End Frame]` detail, 96,807,137 bytes, `cover=0`. Same
game as this map's own 24,000-frame run: `rngcmp.py rontrace-run54.log
rontrace-run77.log` → **differing frames: 0, identical frames: 10401**.

**What it was for, and why it missed.** Item 227 needs a squad born from a
building on a *second* map — one sample of two member offsets cannot tell a
formation from a search. The window was placed off **function coverage**:
run53 and run54 were both taken with `cover=1` under CrossOver, so each
carries about 6,900 records of "the frame this function was first entered
on", and the nine functions that first execute on Great Lakes 6612 —
`Unit::set_group`, `think_attack`, `find_melee_target`, `on_duty`,
`UnitData::get_activity`, `get_combat_stance`, `Army::add_group`, `member`,
`add_unit` — first execute on East Indies **10187**.

They do. It is still the wrong frame, because **`set_group` and `add_unit`
fire for a singleton group too**, so those nine date "a group was formed",
not "a squad was born". `FRAME 10188` holds 168 objects and so does 10187;
nothing is created. The only change of substance is unit `1/32` (guy 340,
180 hits, internal `(45192, 41880)`) gaining `flags & 0x8`, the captain bit,
and every grouped unit on the frame is alone in its group — `1/0` in 65,
`1/15` in 67, `1/31` in 64, `1/32` in 66. No Barracks among the 25
buildings, no Archers anywhere.

**The tell was in the same two lists.** `UnitData::get_captain` first runs on
Great Lakes at 6612 and on East Indies at **12794**. A singleton group never
asks for a captain, so the gap between those two numbers is exactly the
distinction the nine functions cannot draw.

**The instrument that does date a birth is the draw stream.** A creation
draws at `Guy::init_real+0x52 < Unit::init+0xb97 < Objects::init_unit+0xbd`,
and `report.py <log> when` counts one named chain per frame across a whole
24,000-frame trace — the verb went into the reader rather than a scratch
script because it reuses the same log parse and `INDEX.tsv` naming every
other verb uses. ~~`rontrace-run53.log` carries exactly three of them on
Great Lakes 6612, so a count of three is a three-unit squad, and East Indies'
is 15782.~~ **That inference is wrong, and run78 is what measured it — see
the next section.** The site fires once per **`Guy`**, and a unit has one or
more of them, so the per-frame count is guys created: an upper bound on
units, equal to units only for single-guy types. The verb dates a birth; it
does not count one.

Two rules earned their keep here and one was nearly broken. Grepping the
disk before booking a capture is what found 15782, and it cost a minute
against a fifteen-minute run. And a first reading of run54 said "no Barracks
on East Indies at frame 24,001" — **wrong**, because run54's `BUILDS` detail
never prints `otype` and every building came back `otype= -`. The
no-Barracks statement above is from run77's own `BUILDS=7` dump and is only
about `FRAME 10188`. A field that is not in the dump reads exactly like a
field that is zero.

## run78 — the counter that counts guys, and East Indies still has no squad birth (2026-09-04)

**What it is.** East Indies, seed 12345, 201 frame blocks over `[15700,
15900)`, 92,005,065 bytes, `cover=0`. Same game: `rngcmp.py
rontrace-run54.log rontrace-run78.log` → **differing frames: 0, identical
frames: 15951**.

**What it was for.** run77's section booked this window on a count of three
draws at `Guy::init_real < Unit::init < Objects::init_unit` on East Indies
15782, read as three units born — the analogue of Great Lakes 6612's Archer
squad. It is not.

**What the dump says, and it is unambiguous.** Across all 200 frames of the
window the object counts change **exactly once**: units 170 → 171 at `FRAME
15783`, animals and buildings never. Comparing the two frames by `(who, o,
uid)` — so a slot reused by a new unit could not hide — gives one new unit
and nothing gone: `1/60`, uid 91, `guy 353`, 109 hits, at internal
`(38040, 42408)`. **Three draws, one unit.**

**So the site is per-`Guy`, not per-unit**, and the count is guys created.
Great Lakes 6762 is the other half of the calibration: one draw there, and
run76's own window shows units 76 → 77 on `FRAME 6763`. One draw, one unit;
three draws, one unit. The multiplier is the unit's model count, so a `3` is
a three-Archer squad on one map and a single three-guy unit on the other.

**The chain matters as much as the site, and that was measured too.**
Matching `Guy::init_real` alone counts three draws on Great Lakes **6736**,
where the dump's unit, animal and building counts do not move at all — the
guy of an existing unit being re-initialised. Requiring
`< Unit::init < Objects::init_unit` drops 6736 and keeps 6612 and 6762, so
`when` takes the whole chain rather than a name. Two more sites were tried as
per-unit signals and neither survives: `Unit::set_anim < Unit::do_idle` fires
three times at Great Lakes 6612 and not at all at East Indies 15782, which
looks decisive until you count it over a whole run and find it firing on
frames with no birth at all.

**Where 227 stands after two captures.** Neither East Indies window holds a
multi-unit birth: run77's `[10150, 10400)` holds none at all, and run78's is
one single unit. The map's multi-member groups — `66` with `1/32` as captain
and `1/34`, `1/35` at `flags 73`, and `68` with nine units in a lattice
around internal `(34000, 41000)` — are on the map *before* 15782 and were
assembled, not born together. So the second sample the item needs is still
not taken, and the disk cannot say which of East Indies' remaining candidate
frames (6164, 6353, 6571, 17112, 17362, 17574, 17793, 18785, 20249) is a
squad rather than one multi-guy unit. Only a dump distinguishes them, which
is the cost the next booking has to weigh.

## run79 — two squads, one Barracks, three points (2026-09-06)

**What it is.** Great Lakes, seed 12345, 340 frame blocks over `[6910,
7250)`, 86 MB, `cover=0`. Same game: `rngcmp.py rontrace-run53.log
rontrace-run79.log` → **differing frames: 0, identical frames: 7301**.

**What it was for.** run77 and run78 each bet a capture on one East Indies
frame and each came back with no squad, because `report.py … when` counts
*guys*. This took Great Lakes' next **two** candidate threes in one window
instead of betting on one. Both are squads, so the window paid twice.

**Both births, off the block.** `tools/gamelog/births.py` — the census diff
that graduated with this run — over `[6910, 7250)` prints five lines and
nothing else:

| frame | what moved | who |
|---|---|---|
| **6994** | units 77 → 80 | `1/31` uid 49, `1/32` uid 50, `1/33` uid 51 — group 66 |
| 7183 | buildings 25 → 26 | `1/2018`, `orig_type 428`, at `(45120, 23424)`, 1200 hits |
| **7213** | units 80 → 83 | `1/34` uid 53, `1/35` uid 54, `1/36` uid 55 — group 67 |

**The trainer is a Barracks, and it is run76's own.** `orig_type 427` on
`1/2016` at `(45120, 25728)`. Across each birth that building alone moves a
queue field — `queued 3 → 2`, `queue[scan].job_counter` reset to 0 — and at
7213 `queue[scan].type` goes `177 → 132`, the item that has just left
followed by the next. No other building's queue moves on either frame. The
7183 building is a **Stable**, a coincidence of the window rather than a
trainer of anything in it.

**The three points are run76's three points, to the unit.** Both squads are
born at `(45144, 26424)`, `(45144, 26568)`, `(45288, 26520)` — the captain's
ring bearing-0 snap and the two member candidates `docs/CITIES.md` §6.5.1
derives, exactly. Three squads now, nine units, three points, on two
different unit types and 601 frames apart. The placement carries no
dependence on the frame, on the draw, on the unit, or on what else stands on
the map: it is a function of the trainer and the captain and nothing else.
All six are born at `angle 1431655765` — `Unit::init`'s `0x55555555`, still.

**The chain IS in the dump.** §6.5.1 says it is not, on the ground that the
log prints the `ObjectData` `up`/`down`/`down_who` rather than
`UnitData::o_up`/`o_down`. It prints **both**, under those exact names, at
`UNITS=3`: the `ObjectData` pair early in the record, and `o_up`/`o_down`
in the `UnitData` tail. The captain is the head of the list.

```
1/31  o_up -1  o_down 32     1/34  o_up -1  o_down 35
1/32  o_up 31  o_down 33     1/35  o_up 34  o_down 36
1/33  o_up 32  o_down -1     1/36  o_up 35  o_down -1
```

run76's own archive says the same of its squad — `1/27` `o_up -1`,
`o_down 28`, then 27/29, then 28/−1 — so the claim was falsifiable on disk
before this capture was booked. That is the "grep the dump before booking a
reading" rule, missed once.

**The type space, and it names every dumped object.** A dumped `guy` type
and a `BuildData::orig_type` are indices into **one** object-type space, and
its layout is fixed by arithmetic: buildings begin at 414, units are 364
records, so units begin at **50** and the 50 slots below them are
`resourcerules.xml`'s 50 records.

    resources 0–49        units 50–413 (= 50 + unit record)
    buildings 414–542 (= 414 + building record)

Every type in run79's window resolves under it, `HITS` for `HITS`: guy 50
Citizen 40 (×23), 59 Caravan 90, 61 Merchant 90, 69 Scout 50, **177
Longbowmen 88** — `UBER_SIZE 3`, `FROM` Bowmen — and player 8's 36 **Herd
Fish** and 4 **Herd Sheep** at 1 hit each, on a lakes map. The buildings
read Small City, Farm ×8, Woodcutter's Camp ×2, Library, Market, Barracks,
Tower, Stable — an ancient-age build with nothing left over.

**So run76's squad is Bowmen, not Archers.** Guy type **170** = unit record
120, `Bowmen`, 70 hits — which is what its units carry. `Archers` is record
121 and 80 hits. §6.5.1's label is wrong; its geometry is not. The
`type 21` that named them is the first `type` line in a *marching* unit's
record, which belongs to its order, not to its guy — `one.py` keeps the
first of a repeated key, and `births.py` reads the `GUY` sub-record instead.

**And the Barracks trained Bowmen at 6612 and Longbowmen at 6994.** Same
building, upgraded unit, identical member points — so the members' ring does
not move with the member's own type. The queue's next item at 7213 is guy
132, `Hoplites`, `UBER_SIZE 3`: a fourth squad from the same trainer is
already booked inside the archive.

**What it leaves for item 219.** The window holds the first squad's whole
march from 6994 to 7250 — 256 frames — and the second's first 37, from the
same building, on the map 219 already owns.

## run80 — the four territory seams at 24,000, and one of them fires (2026-09-06)

**What it is.** Great Lakes, seed 12345, 40 frame blocks over `[23960,
24000)` at `LEADERS=9`, 79 MB, `cover=0`. Same game: `rngcmp.py
rontrace-run53.log rontrace-run80.log` → **differing frames: 0, identical
frames: 24001**. The run-up to 23,960 took **forty seconds**: run53's stanza
warns this game is hours, and that was the `cover=1` int3 forest, not the
game.

**What it was for.** `docs/ATTRITION.md`, "Territory", names four inputs
inert on every capture so far. The disk was grepped before the booking and
refused to answer — the five archives that reach 24,000 carry thirteen
`orig_type` lines each, which is their start dump and nothing after it — so
this is the recon frame the item needed. **Not a coverage run**: no capture
on this machine shrinks `report.py … blind`, because coverage is exactly
what `cover=0` costs (226). What it does instead is the other route to the
same finish line — convert a reading-only claim into one a dump either
contains or refuses.

**The verdict, seam by seam.**

| seam | at 23,999 | why |
|---|---|---|
| gem rare | **FIRES** | player 1 has collected Gems |
| temple border techs | cannot have fired | no Temple exists |
| fort border techs | cannot have fired | no Fort exists, `fort_mark 0` |
| Colosseum / Eiffel | not built | but the AI does build wonders |
| AI handicap | **unreachable**, not merely zero | the branch is gated on a multiplayer-only flag — amended 2026-09-06, below |

**The gem fires, and the map says so by name.** `rares_collected[44]` on
player 1 is `{11, 13, 23, 27}`, and the array runs over resources 6–49 —
the six below it are the base goods, which `escrow[]`'s six entries
independently confirm. Offset by six the four are Amber, Tobacco, **Gems**
and Wool. **All four are on this map and none of the offset-0 readings are**
(Silk, Salt, Bison, Sugar), which settles the indexing: a `BEGIN GOOD`
record prints its resource **by name**, and the start dump's 35 goods are
Oil ×14, Fish ×12, then one each of Amber, Dye, Tobacco, Cotton, Wool,
**Gems**, Citrus, Aluminum, Rubber. One Gems on Great Lakes, and player 1
has it. So the term is live in a game the project already diffs, and
`territory` at 23,999 — **player 0 at 266, player 1 at 568** — is wrong
without it.

**The two building seams are blocked at their prerequisite.** Player 1's 27
buildings at 23,999 are Farm ×10, Small City ×2, Woodcutter's Camp ×2,
Barracks ×2, Stable ×2, University ×2, Mine ×2, Library, Market, Tower,
Senate and the **Pyramids**; player 0's five are Small City, Woodcutter's
Camp, Farm, Library, Market. No Temple (437), no Fort (443), no Fortress
(445), and `fort_mark 0`. `has_preq(TEMPLEBORDERS2..4)` and
`has_preq(FORTBORDERS2..4)` therefore cannot be true — not "did not happen
to fire", but could not. The Pyramids matter for the other pair: the AI
**does** build wonders on this map, so the Colosseum and Eiffel terms are
reachable in principle and simply are not reached by 24,000.

~~**The handicap is zero by the lobby, not by the frame.**~~ `handicap 0` on
both players here, and the same on East Indies 5379 (run59) and Great Lakes
779 (run41) — but **"a click, not a longer wait" was wrong, and so was the
whole of this paragraph's conclusion.** The allowance is not zero-valued, it
is **switched off**: see "117 is not a lobby click" below and
`docs/ATTRITION.md`, Territory.

**So 117 splits.** One seam is live and needs modelling now; two are blocked
behind buildings the AI does not build in 24,000 frames of this scenario;
one is blocked behind a lobby setting. Three of the four need a scripted
setup rather than a longer capture, which is a session's work and not the
screen's.

**Two things the dump gave for free.** A `BEGIN GOOD` record names its
resource, so any start dump lists the map's whole rare inventory without a
lookup. And at `LEADERS=9` a frame block carries more than the leaders: a
hand count taken off one is easy to get wrong.

> **Amended 2026-09-06 (capture lane, off run84's measurements).** The
> sentence that stood here — "a frame block writes each object **twice**, the
> full record then a seven-key stub of position alone" — is wrong in its
> cause, and the caution it gave was right for the wrong reason. Measured on
> this run's own block 23960: the depth-2 `UNITDATA` records are **87, with
> 87 distinct `(who, o)` and none duplicated**, and the smallest is 90
> fields. Nothing at the top level is written twice. What `LEADERS=9`
> actually adds is **inside** the census block, which at `LEADERS=1` has no
> nested records at all: each frame's four `LEADERDATA` blocks gain
> `DIPLOMACY` x32, `MAKEOBJECT` x22, `SITE` x20 and `PERSONALITY` x4. And a
> `BEGIN SITE` is **six** keys — `wx`, `wy`, `val`, `reg`, `dist`, `rank` —
> a scored candidate site, not a position stub; there are ten per *in-game*
> leader and none for the other two. So a naive `grep -c` over a whole block
> can pick up census entries alongside objects, which is the real trap.
>
> **run84 is the proof, and it is stronger than any count.** Its
> `samegame.py --exclude LEADERDATA` against run79 — the same game, the same
> detail but `LEADERS=1` — is **80 blocks in common and 0 differing**. If
> `LEADERS=9` doubled objects outside the census block, that check could not
> have passed. `births.py` remains unaffected either way.

**What it did not establish.** How much territory the gem adds. One frame
cannot separate the flat additions from each other, and the falsifier is a
capture either side of the Merchant reaching the Gems.

## run81 — the merchant's cast is 149 frames, and that is why nothing had seen an unpack (2026-09-06)

**What it is.** East Indies, seed 12345, run68's game and run68's detail with
the window moved to `[6730, 6800)` and `GOODS=3` added — 70 blocks, 49 MB,
`cover=0`, four minutes end to end. Same game: `rngcmp.py rontrace-run54.log
rontrace-run81.log` → **differing frames: 0, identical frames: 6816**.

**The disk was grepped first, exhaustively, and it refused.** `docs/MERCHANT.md`
§7 said no capture had seen a `MERCHANT` unpack; the writer of the bit is
`SpellType::cast_unpack@006709c0` (`*puVar1 & 0xfff7ffff`, so `0x80000`), which
makes the tell a single bit and the scan complete rather than clever. Of every
archive in `Logs/`, **ten** ever carry a unit with `0x80000` set at all —
run18b, 44, 58, 59, 63, 66, 67, 68, 76, 79 — and in none of them does any
`(who, o, uid)` gain or lose the bit between blocks. Not one pack, not one
unpack, in the project's whole history of captures.

**And this capture did not catch one either — which is its finding.** The
merchant `1/19` finishes its walk on block **6735**, standing on
(32280, 36888), which is `orders_x/y` exactly. On **6736** the cast starts,
and the block says so four ways at once:

| field | 6730–6735 | 6736 onward |
| --- | --- | --- |
| `cur_anim` | 8 | **24** |
| `end_time` | 15 | **149** |
| `stopped` | 0 | **1** |
| the `spell 656` order's `paid` | 0 | **1** |

`cur_time` then advances **exactly one a frame** — 1 on 6736, 64 on 6799, no
reset and no gap — so the cast ends at 6736 + 148 = **6884**, and
`unit_masks` is 9175050 on all seventy blocks because the bit cannot clear
before then. **The unpack is not an event on arrival; it is a 149-frame
animation the arrival starts**, which is the whole reason every window ever
aimed at this merchant has missed it. run68's window closed 149 frames early
and this one closed 85 early.

**So the frame is now derived, not guessed.** run77 and run78 each bet a
window on one frame off a coverage list and each came back empty; this
prediction rests on sixty-four measured samples of a counter the dump prints.
run82 is the capture that spends it.

**The order in the stack is `spell 656`, not the `0x28c` the decompiler
prints.** `Unit::unpack_merchant@006038e0` calls
`add_cast_order(this,-1,-1,-1,-1,0x28c,QUEUE_NEW,0)` — 652 — and the order
that lands in `1/19`'s stack is `type 14`, `spell **656**`, `ox -1`,
`whom -1`, `x -1`, `y -1`. One order, the whole window. Which of the two
numbers is the spell and which is something the decompiler has folded is not
settled here; the dump is the stronger witness and 656 is what it says.

**Four things the window gave for free.**

- **The `GOODS=3` category is nearly free and stable.** 66 `BEGIN GOOD`
  records a block, nine lines each, identical across all seventy — so the
  good is a per-frame record from here on at ~15 KB a block. The Citrus is
  `o 20`, `who 255`, at (31776, 37152), and its `ever_seen` is **2** where
  the start dump had 0.
- **The deploy spot is not the good's tile.** (32280, 36888) against the
  Citrus's (31776, 37152) is 2.6 tiles east and 1.4 north. A merchant stands
  *near* its rare, not on it.
- **Nothing else is in the two-by-two.** `cast_unpack` blocks
  `(x,y), (x-1,y), (x,y-1), (x-1,y-1)` off the deploy tile — here tiles
  (168,192), (167,192), (168,191), (167,191), which is x ∈ [32064, 32448)
  and y ∈ [36672, 37056). On block 6799 the only unit in that box is the
  merchant. So `docs/MERCHANT.md` §7's `detect_unit_collision` bullet is
  untouched by this capture: the footprint is empty either way, and the
  question needs a game where it is not.
- **`leader_flags` is a per-frame field at `LEADERS=1`.** Player 1's is
  33554439 = `0x2000007`, so the `0x2000000` `cast_unpack` ORs into the
  leader is **already set** through the whole window. It cannot be used as
  the tell for a deploy; the unit's own `unit_masks` can.

## run82 — the merchant unpacks, and it is the first one ever captured (2026-09-06)

**What it is.** run81's game and detail with the window at `[6860, 6930)` and
`LEADERS=9` added — 70 blocks, **151 MB**, `cover=0`, seven minutes. Same
game: `rngcmp.py rontrace-run54.log rontrace-run82.log` → **differing frames:
0, identical frames: 6946**. The stanza's own teeth came back
`blocks=70 packed first=1 last=0`, which is the sentence this run was booked
to make true.

**The unpack, frame by frame.** `1/19`, the AI's Merchant, on the Citrus:

| block | what changes |
| --- | --- |
| …6882 | `unit_masks 9175050`, `cur_anim 24`, `cur_time` 147 of 149, (32280, 36888), `flags 1`, `mylos 3` |
| **6883** | `unit_masks` → **8650762**; position and `orders_x/y` → **(32256, 36864)**; `mylos` 3 → **5** |
| 6884 | `cur_anim` 24 → **0** with `end_time` **3**; `flags` 1 → **9**; `idle` starts counting |
| **6888** | `rare` 0 → **26**, `good_obj` -1 → **10**; and the leader's income steps |

**Three things that is, exactly.**

- `9175050 - 8650762 = 524288`. The clear is `& ~0x80000` and **nothing
  else** — no other bit of `unit_masks` moves — which is
  `SpellType::cast_unpack@006709c0`'s `*puVar1 & 0xfff7ffff` checked against
  a dump rather than read.
- **The snap is exact and it is to the tile.** (32280, 36888) is tile
  168.125, 192.125; (32256, 36864) is **168.0, 192.0**, and 168 × 192 = 32256,
  192 × 192 = 36864. `cast_unpack` calls `set_new_location(TVar2 * 0xc0,
  TVar3 * 0xc0)` on the tile indices, so the merchant is **teleported** onto
  the cell corner on the frame the bit clears; `orders_x/y` are rewritten to
  match, and the walk it had is simply over.
- **The cast fires at `cur_time == end_time - 1`.** 148 against 149, and the
  animation is replaced rather than run out. A model that waits for
  `cur_time == end_time` is one frame late. run81 predicted 6884 off the
  linear counter and the answer is **6883**.

**The pay is on the gather clock, five frames behind the deploy.** `LEADERS=9`
rode along for this and it is the half nothing on disk had. Leader 1's whole
census differs in **14 of 11,796 fields** between 6882 and 6890:

| field | 6882 | 6890 | |
| --- | --- | --- | --- |
| `income` / `resources` | 1920 | **2080** | +160 |
| `income[1]` / `resources[1]` | 1280 | **1440** | +160 |
| `rares_collected[scan][20]` | 0 | **1** | |
| `gather_stamp` | 6767 | **6887** | +120 |
| `score` | 711 | 740 | |
| `bit_values`, `bucket`, `bucket[1]`, `leftover ×3` | | | |

- **Two slots gain 160 each, and that is `calc_rare` twice.**
  `docs/ECONOMY.md` step 6: `calc_rare` reads the good's **two**
  `(BONUS_TYPE, BONUS_NUM)` pairs out of `resourcerules.xml` and multiplies by
  sixteen. Citrus is ten and ten, so 160 into each of two resources — the
  arithmetic and the *pair* are both confirmed by one diff.
- **`rares_collected` is offset by six, independently.** The unit's `rare` is
  **26** and the leader's array moves at index **20**. run80 derived the −6
  offset from Great Lakes' Gems by name-matching the map's inventory; this is
  the same offset falling out of a single unit on a different map.
- **`gather_stamp` moves 6767 → 6887, exactly 120.** So the recompute is
  periodic and the deploy does not trigger it: the bit clears on 6883, the
  next stamp lands on 6887, and the unit's `rare`/`good_obj` and the leader's
  income both appear in block **6888**. `bucket` is *not* that clock — it
  steps every five or six frames throughout the window, 115 to 128.

**The deposit itself never changes.** All 66 `BEGIN GOOD` records are
byte-identical across all 70 blocks: the Citrus stays `o 20`, `who 255`,
`ever_seen 2`, `flags 1`, at (31776, 37152). Ownership of a rare lives on the
**unit** (`rare`, `good_obj`) and on the **leader** (`rares_collected`), never
on the good — `docs/ECONOMY.md`'s "the bonus is the merchant, not the
deposit", now diff-backed.

**And the closing block was telling the truth this time.** run68 quit at 6745
and its closing block put `1/19` on (32280, 36888) with `flags 1`; run81's
*ordinary* block 6746 says (32280, 36888) and `flags 1`. Same position, same
flags. The deploy was 137 frames away.

**What it does not answer.**

- **`gather_down` and `special` are still -1** on every block to 6929, 46
  frames past the unpack, where run76's long-deployed Great Lakes merchant
  carries `gather_down 18` and `special 6`. Both fill in later than this
  window reaches, and nothing on disk holds the frame they do.
- **`good_obj 10` is not the good's `o`.** The Citrus's `BEGIN GOOD`
  subobject is `o 20`; the unit's `good_obj` is 10. They are indices into
  different arrays and which is which is unread.
- **The two-by-two is unobservable.** `World::set_blocked_at` writes into the
  `WORLD` block, which is 600,601 lines in this game's own start dump and
  cannot ride a window. By arithmetic the tiles are (168,192), (167,192),
  (168,191), (167,191); no unit is in any of them, so
  `find_merchant_spot`'s missing `detect_unit_collision` test costs nothing
  here and is untested by this game.

## run83 — the last Great Lakes hole, and the deflection inside it (2026-09-06)

**What it is.** run53's game, `[6864, 6916)` at run76's and run79's detail —
53 blocks, **22 MB**, `cover=0`, four minutes end to end. It closes the only
stretch of either map's run-up that no archive held: run76 stops at block
6869, run79 starts at 6910, and **6870–6909 existed nowhere**.

**Three checks, and two of them are new in kind.** `rngcmp.py
rontrace-run53.log rontrace-run83.log` → **differing frames: 0** over 6,931.
Then `samegame.py` against **both** neighbours, on the twelve blocks of
deliberate overlap:

| against | common | differ |
| --- | --- | --- |
| run76 | **6 (6864..6869)** | **0** |
| run79 | **6 (6910..6915)** | **0** |

That is a *state* digest, not the LCG word, and it is the claim `rngcmp.py`
cannot make. A window is priced by its blocks and the frames before it are
free, so twelve blocks of overlap cost about seven seconds and turned "same
seed, therefore same game" into something that either matches or does not.
**Each check asserts the common count as well as the verdict**, because
`samegame.py` exits 0 when nothing differs *including when nothing is in
common* — a vacuous pass that a bare exit code would have hidden.

**The disk was grepped first**, every archive, for any block labelled
6850–6930. One candidate was not run76 or run79: **run16**
(`gamelog-run16-attrition.txt`, MAP_STYLE 14, seed 12345, blocks 1–6872),
which does hold 6870, 6871 and 6872. It is a different game —
`samegame.py` against run76 is **230 of 230 common frames differing** —
because it is the cheat-driven attrition run and parts from its first
`peace` at label 341; its detail is thinner too (`BUILDS=1 CITIES=1 GUYS=1`
against 7/5/2). The hole was 6870–6909 entire.

**What the original does in the forty frames.** Very little, and one thing
that matters. **No unit is born and none dies** — 77 units on all 53 blocks,
by `(who, o)`, no exceptions. Five units take a new order: `1/5` on 6865,
`1/15` on 6871, `0/5` on 6875, `0/4` on 6891, `1/17` on 6907. And there is
exactly **one blocked stand**, `1/29`'s on sim-frame **6892**:

| block | `1/29` | `collide` | `collide_o` / `collide_who` | `collide_frame` |
| --- | --- | --- | --- | --- |
| 6892 | (42388, 23858), stepping (−19, −18) | 0 | −1 / −1 | −1 |
| **6893** | **(42408, 23880)** — *backwards* | **1** | **17 / 1** | **6892** |
| 6894–6898 | 42382 → 42282, **y pinned at 23880** | 1 | −1 / −1 | 6892 |
| 6899 | (42264, 23880), diagonal resumes | 0 | −1 / −1 | 6892 |

**The blocker is `1/17`, and it never moves.** A citizen — `myspeed 25`
against the Archers' 26, `form 9` — parked at **(42360, 23736)** with
`orders_x/y` equal to its own position and `collide_frame 3831`, an ancient
stamp it does not touch. It registers nothing; the whole interaction is
written on the walker.

**And the shape is a deflection, not a stop.** `1/29` is pushed *back* one
step on the frame after the block, then **slides along the obstacle** — five
frames at exactly −26 in x with y constant to the unit — before resuming its
bearing. It never idles: `idle` is 0 throughout. Anything that models a
blocked step as "stand still and repath" gets six frames and about 1.6 tiles
wrong here.

**One field-lifetime fact a differ needs.** The three collision fields have
three different lives. `collide_o`/`collide_who` name the blocker for
**exactly one block** and are −1 the next; `collide` is a latch that stays 1
for six; `collide_frame` keeps the stamp permanently — `1/27` still carries
6861 and `1/28` 6862 fifty blocks later, and half the AI's units carry stamps
in the hundreds and thousands. A comparison that reads `collide_o` a frame
late sees −1 and calls it agreement.

**Why it was taken, and what it does not do.** It was booked when Great
Lakes' word stood at 6862 and the next divergence looked likely to land in
the gap. The word moved to **6982** while the stanza was being written, which
run79 already covers, so **this run moves no score**. What it does is retire
the last stretch whose lockstep rested on the draw stream alone: over
6870–6909 no record of any kind had ever been compared, and a state
divergence that did not perturb draws for seventy frames would have been
attributed to the economy at 6982 by everyone who looked. The forty frames
are now a diff like every other.

**Positions across the hole**, for whoever diffs it next:

| unit | 6870 | 6890 | 6909 |
| --- | --- | --- | --- |
| `1/13` citizen | (42744, 24504) | (42744, 24504) | (42744, 24504) |
| `1/17` citizen | (42360, 23736) | (42360, 23736) | (42324, 23702) |
| `1/27` Archer | (43040, 24552) | (43094, 24189) | (42729, 24106) |
| `1/28` Archer | (42622, 24647) | (42337, 24323) | (42129, 24036) |
| `1/29` Archer | (42806, 24254) | (42426, 23894) | (42153, 23848) |

All three hold `group 64` throughout; `1/13` and `1/17` are `form 9` and
ungrouped.

**Amended 2026-09-06 (capture lane), twice, and both are about what run76 and
run83 can be *used* as.**

**Neither is a goods oracle.** Both carry exactly **one** leader goods block —
24 `bucket` lines in the *initial* dump, the `[200, 200, 100, …]` grant — and
**no** frame block in either file carries one, against run80's 41 at
`LEADERS=9`. `loop-238` hit this looking for where two units of wealth were
banked and it is verified here by counting. A booking that assumes run76 or
run83 can be integrated forward for a leader's goods would waste its window.

**And the squad's own type changes inside run76's window.** run79's section
below says guy type 170, `Bowmen`, "is what its units carry"; that is true of
run76 up to **6736** and false after. On **frame 6737** all three of `1/27`,
`1/28` and `1/29` go **170 → 177** — `Longbowmen` — in one frame, keeping their
`(who, o)` and their `group 64`. So an upgrade in this engine is an **in-place
guy-type change on the standing unit**, not a modifier applied to a type, which
is the same shape `docs/DANGER.md` §8.1 finds on East Indies (`1/32`, type
340 → 341, and the danger map moves by exactly (110 − 100) / 2). run79's
"trained Bowmen at 6612 and Longbowmen at 6994" is about **births** and stands;
what it does not say is that the 6612 squad had already become Longbowmen by
6737.

## run84 — the make-list across 6982, and the rebuild nobody had seen (2026-09-06)

**What it is.** run53's game, `[6950, 7030)` at run79's detail with `LEADERS`
raised 1 → 9 and nothing else moved — 81 blocks, **145 MB**, `cover=0`, six
minutes. It is the first capture booked because the frames were **covered and
the detail was not**: run79's window is [6910, 7250) and holds 6982, but at
`LEADERS=1` it carries **22 `BEGIN MAKEOBJECT` records in the whole file** —
the start dump and nothing per-frame — against run80's 902 over 41 blocks,
exactly 22 a block.

**Three checks, and the overlap is now the whole window.**

| check | result |
| --- | --- |
| `rngcmp.py` vs `rontrace-run53.log` | **0 differing**, 7,046 identical |
| `samegame.py --exclude LEADERDATA` vs run79 | **80 in common (6950..7029), 0 differ** |
| frame blocks carrying a `MAKEOBJECT` | **80** (run79 gives **0**) |

The window sits **entirely inside run79's**, which takes run83's overlap
practice to its limit: 80 blocks of state agreement rather than six, for
nothing, because a window is priced by its blocks and the frames before it are
free. `LEADERS` is the only category that moved, so `--exclude LEADERDATA`
asks exactly the right question and the answer is that **everything the two
runs record in common is identical**. Checked before booking that the
exclusion suffices: `LEADERS=9` does *not* double the top-level object
records — run80's block 23960 has 127 `UNITDATA` and 127 distinct
`(who, o)` — so that stanza's "each object twice" note is about stubs inside
the census block, which the exclusion removes with it. The third check was
made to fail first: it prints 40 on run80 and **0** on run79, which is
precisely the gap that made this capture necessary.

**The make-list is rebuilt from empty, and 6982 is when the buildings land.**
Player 1's list has **11 slots**; live entries by `(type, val, city, cat)`:

| block | entries | what changed |
| --- | --- | --- |
| 6950–6976 | 7 | 2× 420 (`val 6075000`, city 1), 2× 428 (`val 5722784`, city 1), 437, 566, 573 |
| **6977** | **0** | the whole list is dropped |
| 6979 | 5 | +552, +2× 566 (`val 2100000`), +2× 573 (`val 165000`) |
| 6981 | 7 | +2× 82, +1× 132 (both `val 2445568`, city 0, cat 6); −566 |
| **6982** | **9** | **+2× 420 (`val 1518750`), +2× 428 (`val 5722784`)** — the city-1 buildings; −573, −82 |
| 6983 | 8 | **−1× 428** — consumed; 132's `val` collapses 2445568 → 24455 |

Three things fall out of that, and none of them was visible before:

- **The list is rebuilt, not amended.** It empties completely on 6977 and
  refills over the next six frames. Anything that models the make-list as an
  incremental queue is wrong in kind, not in degree.
- **`type=420`'s valuation drops by exactly four across the rebuild** —
  6075000 before, **1518750** after, and 6075000 / 4 = 1518750. The same
  entry, the same `city`, the same `cat`, a quarter of the price.
- **`type=437` is dropped and never returns.** run80 identified 437 as the
  **Temple**; it is live in every block from 6950 to 6976 and in none after.
  So the rebuild does not merely re-price the list, it changes its membership.

**And 6983 is where `make_stuff` chooses**: one `428` leaves the list, which
is the building the original actually buys. That is the row the divergence
becomes — the sim's `Leader::produce_building+0x1805` against the original's
`Leader::make_stuff+0x221` at frame 6982 stops being a draw-count difference
and becomes "the original's list held these nine entries at these valuations
and took a 428 on the next frame".

**What it does not establish.** ~~**The type numbers are not named here.**~~
**Named 2026-09-06 by `loop-238` through `rondata`'s table: 420 is the
University, 428 the Stable, 437 the Temple.** The buildings standing on this
map carry `orig_type` 414, 417, 418, 427, 435, 436 and 439; neither 420 nor 428
is built, which is consistent with their being *wanted*. ~~`val`'s units are
unread — the factor-of-four is exact and what it is a factor *of* is not.~~
**Also answered, and the caution was right to have: the factor of four is not
a re-price.** `Leader::check_income@006cc800` returns `0x100` for a type the
leader can afford one of and **`0x40`** for one it cannot while escrow is on,
and `Leader::create_buildings@006c1be0` multiplies the offer by that over 256.
So the University's `val` fell because the leader **stopped being able to
afford it**, not because the rebuild re-priced it — and "the price fell" would
have sent the item the wrong way. The Temple flag resolved the other way: the
membership change is real and this crate follows it. And `cat`
(4, 6, 7, 8, 9, 10) is taken as a category index on the evidence that `city`
is 1 for the two building entries, 0 for the cat-6 pair and −1 for cat 8–10;
nothing here proves that reading.

**One tooling note.** The launch line printed `ready (riseofnations.exe, …)`
where every previous run printed `riseofnations_trace.exe`. The traced
executable *was* what ran — `riseofnations_trace.exe -config check.ini`, and
the trace word matches run53 for all 7,046 frames. The window query had
matched a **concurrent repo worker's shell command**, which contained the
string `riseofnations.exe` in its gate line. Cosmetic here; a future session
reading that line as evidence the untraced binary launched would be wrong.

## run85 — East Indies' blocked stand at 7448, and it is an animal (2026-09-06)

**What it is.** run54's game, `[7400, 7480)` at run68/run81's East Indies
detail — 81 blocks, **54 MB**, `cover=0`, four minutes. East Indies' word
parts at 7448 on two `Guy::set_anim+0x97a` draws, the original's under
`Unit::move_step+0x823` and this crate's under `Guy::inc_time+0x271`.

**The disk was grepped first and refused completely.** Every archive, for any
block labelled 7200–7699 on any map: **exactly one has any, and it is run79,
which is Great Lakes**. East Indies' own coverage stops at run82's 6929 and
does not resume until run77's 10150, so 7448 sat in a **3,200-frame gap**.
There was no neighbour to anchor to, so run84's total-overlap check had
nothing to bite on here — a fact about this window, not a choice.

**Two checks.** `rngcmp.py rontrace-run54.log rontrace-run85.log` → **0
differing, 7,496 identical**. And the teeth: exactly **one** `collide_frame`
transition stamped in [7430, 7470] — `1/20`, **6803 → 7448, in block 7449**.

> That check took two drafts and the first would have passed vacuously.
> `collide_frame` is a **permanent stamp** (run83), so "some unit has a
> `collide_frame` in the band" is true of any window on this game — the draft
> passed on run83's own window for a band in which nothing happened. What says
> a stand happened *here* is a **transition**, and to a value inside the band.
> The final form finds run83's `1/29` and exits 0, finds nothing on the same
> window with a band of 6700–6800, and finds nothing at all on run81's quiet
> merchant walk.

**The event, frame by frame.** `1/20` is a lone unit — `group -1`, `myspeed
23` — walking south-west at (−13, −19) a frame, and it has **two guys**:

| block | position | `collide` / `collide_o` / `collide_who` | guy 0 | guy 1 |
| --- | --- | --- | --- | --- |
| 7448 | (29242, 24876) | 0 / −1 / −1 | anim **8**, 14/15 | anim **9**, 14/15 |
| **7449** | **(29256, 24888)** — back | **1 / 0 / 8** | **anim 0, 1/60, stopped** | **anim 0, 1/60, stopped** |
| 7450 | frozen | 1 / −1 / −1 | anim 8, 1/15 | anim 8, 1/15 |
| 7451–7453 | frozen | 1 / −1 / −1 | anim 7, 1/15 | anim 8, 2→3/15 |
| 7454 | (29233, 24888) | 1 / −1 / −1 | anim 7, 2/15 | anim 9, 5/15 |
| 7455–7458 | west, y pinned | 0 / −1 / −1 | anim 7 | anim 9 |

**The two draws are one per guy.** On 7449 the original sets **both** of
`1/20`'s guys to `anim 0`, `end_time` **60**, `stopped 1` — two
`Guy::set_anim` calls on the blocked frame, from `Unit::move_step`, which is
exactly the pair the parting names. This crate reaches `Guy::set_anim` from
`Guy::inc_time` instead: it advances the clock and never makes the blocked-step
animation change at all. The 60-frame animation is then **replaced after a
single frame** — 7450 has both guys back on `anim 8`, `1/15`.

**The blocker is an animal, and that is the difference from Great Lakes.**
`collide_o 0`, `collide_who 8` is `8/0`, an **`ANIMALDATA`** record: `myhits
1`, `myspeed 19`, stationary at **(29304, 24696)** with `orders_x/y` equal to
its own position for every block of the window. One tile south of the walker's
line. Set against run83's stand on the other map:

| | Great Lakes 6892 | East Indies 7448 |
| --- | --- | --- |
| walker | `1/29`, Archer, `group 64` | `1/20`, **lone**, `group -1` |
| blocker | standing **citizen** `1/17` | stationary **animal** `8/0` |
| response | pushed back, then slides at once | pushed back, **three frames frozen** on a changed animation, then slides |
| animation | (not dumped — run83 was `GUYS=2`) | both guys to `anim 0`/60, then 8, then 7 and 9 |

Same family — a step refused by a stationary obstacle — and a different
obstacle class, a different unit shape and a different recovery. Item 236
dissolved the squad that caused Great Lakes' member and did not move East
Indies; `1/20` is `group -1`, so there was never a squad here to dissolve.

**Two things worth carrying.** The two guys of one unit run **different
animations** — `anim 8` and `anim 9` before the block, `anim 7` and `anim 9`
after — so anything that keeps one animation per *unit* is already wrong on
this unit before the collision is reached. And a record's "unit-level"
`cur_anim` read by taking the first occurrence of the key is really **guy 0's**;
the second guy's is a separate `BEGIN GUY` further down.

**What it does not establish.** What `1/20` is — its type is unnamed here, as
420 and 428 were in run84, and naming it needs `rondata`'s type table. What
animations 0, 7, 8 and 9 are. Why the 60-frame animation is discarded after
one frame. And whether `collide_o`/`collide_who` naming an animal means the
collision test treats animals as units or as a separate pass — the dump shows
the outcome, not the search.

## run86 — the transport ride, and the one field the boat was born without (2026-09-06)

**What it is.** run54's game, `[6924, 7410)` at run85's detail exactly — 487
blocks, **272,628,481 bytes**, `cover=0`, thirteen minutes. Same game:
`rngcmp.py rontrace-run54.log rontrace-run86.log` → **differing frames: 0,
identical frames: 7,426**. East Indies' long word went **7448 → 7529** on
one line of it.

**The disk was grepped first and it answered a piece.** Every archive, for
any block labelled 6930–7399 on any map: four have one — run79, run83, run84
(all Great Lakes) and **run82, which is East Indies**. run82 quit at 6945/6946
and `GameLog::end_game`'s shutdown dump landed after those blocks, holding all
28 player units at **6946**. Nothing had ever parsed it, on that capture or any
other: it is written at `FRAME`'s own indent, so it is a *sibling* of the last
frame rather than a child, and the frame walk cannot reach a sibling.
`Log::final_state` is that reader, and
`run82_s_window_is_the_east_indies_ride_s_run_up` the diff — which also gave
run82's own 70 blocks their first comparison of any kind.

What 6946 settled before a frame was captured: **`1/20` is on the original's
own position there**, as it is on all seventy of run82's blocks. run85 opens at
7400 with it (36, 792) adrift, so the lag is not a run-up that drifted — every
unit of it is made in `[6947, 7399]`. The grep narrowed the band and
strengthened the booking rather than closing it.

**The window is the whole gap, not the two samples §13 booked.** §13 asked for
`[7080, 7140)` and `[7290, 7340)` — the cast, and a *guess* at the landfall,
which spends no draw and so has no frame anyone could name. One run over
`[6924, 7410)` is 486 blocks, ~258 MB and under ten minutes of dumping, and it
**butts against both neighbours**: six blocks over run82 at the bottom, ten
over run85 at the top. Both overlaps were checked, and the top one is byte for
byte — `samegame.py` against run85, **10 in common, 0 differing** — because the
detail was copied from the neighbour rather than chosen. After this, East
Indies has no uncompared frame below 7480.

> The guess would have missed. The eject is **7284**, six frames before
> §13's window opens. Taking the gap whole rather than sampling it is what
> caught that, and it cost one run instead of two.

**The ride, dated, off `births.py`:**

| frame | what |
| --- | --- |
| 7093 | `1/20` takes its last step and freezes at **(34530, 32268)** |
| **7094** | `1/22` is born — uid 39, guy 320, 50 hits, at **(34653, 32075)** |
| 7177 | a building, `1/2014`, `orig_type 439` — not the ride's |
| **7284** | the barge dies; `1/20` appears at **(30408, 28152)** and walks |

**And it is one field.** `1/22` prints **`myspeed 30`** on every block, and the
Transport Barge's `MOVES` is **25**. The difference is
`+WHALES_SHIPS_MOVE%` — the Whales rare, whose naval arm this crate has had
since run63 and which `docs/MOVEMENT.md` §1 measures at exactly "25 → 30 on a
Transport Barge". `Sim::cast_transport` set the boat's speed from
`unit_types[ty].moves` rather than `Unit::update_speed`'s cached value, so the
one place in the crate where a *boat* is born was the one place the bonus was
not applied. Every other spawn already went through `type_speed`.

The measurement is the step: the original walks the barge **(−27, −13)** a
frame, this crate walked **(−23, −11)** — magnitudes 30.0 and 25.5, the same
heading to three digits. Over 190 frames of open water that is 933 units, and
the passenger comes ashore that much late. With `type_speed` in that line,
**neither `1/22` nor `1/20` is ever off the original's position** across all
486 blocks: the birth spot, the whole crossing and `come_out`'s ring are exact,
and none of §13's three suspects was the cause.

**§13's second row came free with it.** A passenger's `x_internal`/`y_internal`
while aboard was unwitnessed on every dump in hand; here it is on 191 of them.
`1/20` prints **(34530, 32268)** — its boarding point — on every block from
7093 to 7283 inclusive, then the ring spot on 7284. `Sim::board` freezes the
passenger at exactly that point, and what was "believed harmless" is now what
the original does.

**What it does not answer.** Why `1/13` parts at 6938 — a citizen walking
south-west at `myspeed 25`, the window's only other divergence besides the
known `1/19` unpack constant, and no capture has covered its frame. What the
ring's own arithmetic is: (30408, 28152) against a boat last at (30521, 28243)
is one sample of `come_out`'s eight bearings, and one sample cannot tell a
bearing from a fallback. And the new word, **7529**, is a `Guy::set_anim` under
`Unit::do_non_flat_gather+0xb99` the original spends and this crate does not —
a gatherer's animation, and the next item.

## run87 — Great Lakes' word frame, and the stand is this crate's alone (2026-09-07)

**What it is.** run53's game, `[7244, 7520)` at run79's detail with `GUYS`
2 → 4 — **277 blocks, 120,956,729 bytes**, `cover=0`, six minutes. It is the
first Great Lakes dump ever taken within 200 frames of the map's own word,
which stands at **7455**: eight draws against seven, and at index 2 **this
crate** spends a `Guy::set_anim+0x97a < Unit::move_step+0x823` the original
does not. That is the reverse of run85's East Indies 7448, where the original
spent two such calls and this crate none.

**The disk was grepped first and it refused completely.** Every archive, for
any block labelled 7200–7699 on any map: exactly one has any — **run79**, and
its window is `[6910, 7250)`; run85's `[7400, 7480)` is East Indies. So Great
Lakes had no dump within 205 frames of 7455 and this was a capture rather
than a diff.

**Three checks, and the second of them found something.** `rngcmp.py
rontrace-run53.log rontrace-run87.log` → **0 differing, 7,536 identical**. The
teeth — the animation state must actually be per-frame, since the whole run is
bought for `cur_anim`/`end_time`/`stopped` — gave **276 frame blocks carrying a
`cur_anim`** where run79, same map, same era, `GUYS=2`, gives **0**.

And the overlap against run79's tail came back **6 in common (7244..7249),
differ 6** — six of six, on a run whose LCG word had just matched over 7,536
frames.

> **The exclusion was one record short, and the record was not the one that
> moved.** At `GUYS=4` an **`ANIMALDATA` prints three fields of its own** —
> `ox`, `whom`, `aid` — after its nested `UNITDATA` and *outside* any `GUY`
> block, so `--exclude GUY` cannot reach them. Across all six blocks the
> whole difference was **+120 lines and nothing else**: forty animals times
> three keys, none deleted, none changed. `--exclude ANIMALDATA` would have
> thrown away all forty animals to remove them, so `samegame.py` grew
> **`--drop KEY`** — a field, not a record — and the overlap is **6 in
> common, 0 differing**. Checked that the new flag did not blunt the tool:
> run16 against run76, a different game, still differs on all 230 common
> blocks with these same flags, and run79 against run83 still passes.
>
> Three fields for the widening ledger with it: nothing in the harness
> names `AnimalData`'s `ox`, `whom` or `aid`, and only a `GUYS ≥ 3` capture
> prints them.

**The original does not stand anything at 7455.** This crate's extra draw is
**`1/36`** — a Longbowman of run79's second squad, group 64 — stopped by
**`1/17`**, the standing citizen that blocked `1/29` on run83's 6892. The
original's `1/36` walks straight through:

| block | `1/36` | `collide` / `collide_o` / `collide_who` | `collide_frame` |
| --- | --- | --- | --- |
| 7453 | (42032, 23883) | 0 / −1 / −1 | −1 |
| 7454 | (42007, 23859) | 0 / −1 / −1 | −1 |
| **7455** | **(41982, 23836)** | **0 / −1 / −1** | **−1** |
| 7456 | (41957, 23813) | 0 / −1 / −1 | −1 |
| 7457 | (41931, 23791) | 0 / −1 / −1 | −1 |

— an unbroken (−25, −23) a frame, no stand anywhere in the window. And
`1/17` stands at **(41784, 23928)** for all sixty blocks of `[7420, 7480)`,
which is **exactly the point this crate has for it**. So neither the
blocker's position nor the collision predicate is the fault, and the reading
that would have blamed one is closed before it is written.

**The machinery is right wherever the positions agree.** The original's only
three `collide_frame` transitions in the window before the word are

| stamped | walker | blocker |
| --- | --- | --- |
| 7285 | `1/31` | `1/11` |
| 7287 | `1/7` | `1/21` |
| 7293 | `1/32` | `1/11` |

and this crate makes a stand on each of those three frames, on the same
walker, blamed on the same blocker. It also stands `1/34` on **7485** where
the original stands it on **7481** — a four-frame near miss on the same
event, which is the shape of a position gap rather than a rule gap.

**What is wrong is a position, and it parts 202 frames ahead of the draw
stream.** Driven through `run_traced`, so every unit of every block: over
12,154 unit fields, 12,108 order/path rows, 20,916 angles, 57,306 collision
fields, 21,856 building and 10,952 queue rows, the window parts at **7253**,
and only two units part at all before 7419.

- **`1/35` on 7253** — the earliest row on the map. Both sides are at
  (45704, 26155) on 7252; on 7253 the original steps **12** units in y to
  (45707, 26143) and this crate steps **25**, to (45710, 26130). It is a
  *step size*, not a heading: the original's walk down this leg alternates
  25 and 12 with repeats, and this crate takes the long step where the
  original takes the short one. By 7269 the two have different path
  lengths — **21 slots against 22** — and by 7270 different headings.
- **`1/26` from 7315** — a constant (24, 24), the same shape as `1/24` and
  `1/25`, both of which are older than either window and already excused.

**And then the army's own tick.** Block **7419** is sim-frame **7418**, which
is `7162 + 256` and `frame ≡ 250 (mod 256)` — the 256-frame group tick
`docs/ARMY.md` §5 names, and **the same tick item 261 fixed at 7162**. All six
of the AI's soldiers take a fresh `orders_x/orders_y` on it —

    1/31 → (36456, 22968)   1/34 → (36120, 23736)
    1/32 → (36408, 23064)   1/35 → (36072, 23880)
    1/33 → (36552, 22824)   1/36 → (36168, 23592)

— and four of them (`1/32`, `1/33`, `1/34`, `1/36`) come out of it on a
different path waypoint from this crate's, with **`1/32` and `1/33` on
exactly each other's** (`dest_x` 39838 and 39863, swapped). `1/36` is **94
units adrift** by 7455 — about three and a half frames of its 26-a-frame walk
— and that is what puts it inside `1/17`'s disc.

**The word's own frame is not comparable as a field**, which is why only the
draw stream ever saw it. The harness tests the collision record and the two
angles **only where the two positions agree** (`compare`, and deliberately —
counting a walker that is somewhere else colliding with something else would
measure the position gap twice). `1/36`'s position parts at 7420, so from
there on its `collide`, `collide_o`, `collide_who` and `collide_frame` leave
the diff entirely. A stand this crate invents at 7455 costs a draw and no
row.

**One cost worth recording.** run87 is 121 MB and `Log::parse` is still eager
(item 260), so the release diff suite's peak went to **15,376 MiB of the
20 GiB `memcap.sh` ceiling** with this test in it — the highest any capture
has pushed it. The next Great Lakes window of this size wants item 260 landed
first, or a raised cap.

**So the word does not move.** Nothing was fixed here; what the capture buys
is that Great Lakes 7455 stops being a collision question. The successors are
the two rows above — `1/35`'s step size on 7253, and the six-slot assignment
at the 7418 tick — and both are movement and formation, not collision.
`run87_s_window_is_great_lakes_word_frame` is the assertion, and its four
claims were each made to fail on purpose: pointing the `collide_frame`
reader at `1/32` (which the original *does* stand, 7293) rather than `1/36`,
moving `1/17`'s band to `[7500, 7520)` where it walks off on **7514**,
raising the parting floor to 7254, and dropping `1/26` from the pair of early
units. Two anti-vacuity guards are in the test permanently for the first two:
run85's first teeth check passed on a band where nothing happened, and an
*absent* `collide_frame` is what a mis-read file also looks like.

## run88 — East Indies' word is a gather countdown one tick long (2026-09-07)

**What it is.** run54's game, `[7474, 7800)` at run85's `[End Frame]` detail
**exactly** — **327 blocks, 186,777,592 bytes**, `cover=0`, ten minutes from
the start click to the last block. It is the first East Indies dump ever
taken at that map's own word, which stands at **7529**, and it closes the
last hole in entry 29's second counter: before it the map had **no dumped
frame above 7480 at all**.

**The disk was grepped first, and it found one frame.** Every archive, for
any block labelled 7480–7899 on any map: exactly two files have any —
**run87**, 7480..7519 with its shutdown block at 7536, which is Great Lakes,
and **run85**, a *single* block at **7496**, which is the
`GameLog::end_game` final state its `!quit` writes rather than a window. So
East Indies had one dumped frame in the whole band, 33 frames short of the
word, and this was a capture rather than a diff.

**Three checks, and the middle one cost nothing to make total.** `rngcmp.py
rontrace-run54.log rontrace-run88.log` → **0 differing, 7,816 identical**.
The overlap against run85's tail → **6 in common (7474..7479), 0 differing**
— and, unlike every overlap before it, **with no `--exclude` and no `--drop`
at all**: taking the neighbour's detail *exactly* rather than nearly is what
made it byte for byte, where run87's `GUYS` 2 → 4 needed three dropped
fields to say the same thing. The teeth → **326 frame blocks carrying a
`cur_anim`, 7474..7799**, which is the detail, the window's placement and
the absence of a `poll_max` truncation in one line.

> **The tooth run85 used was measured and refused.** run85 asserted a
> `collide_frame` transition because its eighty frames were bought for a
> stand. Over **run86's 486 East Indies frames there are zero** — measured
> before booking — so the same line over 326 frames would have been a
> hypothesis wearing a check's clothes, failing for a reason that is not the
> capture. What replaced it is the window itself, and it was made to fail
> first on real data both ways: it prints `6 (7474..7479)` and exits 1 on
> run85, `0 (0..0)` and exits 1 on run86, and with each file's own range and
> count it prints 80 and 486 and exits 0.

**The word is one field, and the field is off by one.** `1/13` is an AI
citizen gathering at `gather_down 12`, standing on its resource with
`orders_x/y` equal to its own position. Its `GATHERORDER`'s `wait` countdown
reads **`theirs + 1` on all 55 blocks** of the cycle that ends at the word:

| block | ours | theirs |
| --- | --- | --- |
| 7474 | 56 | 55 |
| … | … | … |
| 7528 | 2 | 1 |
| **7529** | **1** | **−1** |

— on 7529 the original's countdown has reached its end and the order is
done; this crate's still has a tick to run. That is the whole of it.

**And the draw stream says the same thing one frame apart.**

| frame | ours | theirs | first difference |
| --- | --- | --- | --- |
| 7528 | 32 | 32 | — |
| **7529** | **2** | **3** | theirs `Guy::set_anim+0x97a < Unit::do_non_flat_gather+0xb99` |
| **7530** | **2** | **1** | ours the same site |
| 7531 | 4 | 4 | — |

The gather animation the original asks for on 7529 is the one this crate
asks for on 7530. The word is not a missing call, a wrong predicate or a
unit in the wrong place: it is the same call, one tick late.

**The value diff is one 24-unit step, and it is the dump's own
coordinates.** The original's `1/13` takes its next job on block **7530** —
`orders_x/y` (40536, 37560) → (40344, 38520) — and steps on 7531:

| block | ours | theirs | delta |
| --- | --- | --- | --- |
| 7529 | (40536, 37560) | (40536, 37560) | — |
| 7530 | (40536, 37560) | (40536, 37560) | — |
| **7531** | **(40536, 37560)** | **(40560, 37560)** | **(−24, 0)** |
| 7532 | (40560, 37560) | (40584, 37560) | (−24, 0) |
| 7533 | (40584, 37560) | (40608, 37560) | (−24, 0) |

One step of its 24-a-frame walk, and it never gets it back.

**Nothing else parts at or below the word.** Over 9,156 unit fields, 9,128
order/path fields and 14,124 angles across 327 blocks, fifteen units are
ever off position and thirteen of them part at **7577 or later**. The two
that do not are `1/13` at 7531 and `1/19` — the unpacked Merchant standing
on its trade-post spot, a constant **(24, 24)** on every block, which is
run85's own excused row one window along (`docs/MERCHANT.md`).

**And the successor needs no screen.** The same `+1` runs unbroken through
**run86's** window from block **6985** — `(6985, 545, 544)` through
`(7409, 121, 120)`, 425 rows on a capture taken two days ago and never read
for this field. So the frame that seeds the off-by-one is inside a dump
already on disk and the next step is a **diff, not a capture**. Two things
are deliberately not claimed with it: `1/13`'s *position* parts at 6938 in
run86's window, which is earlier than 6985 and a separate row; and its
second cycle here (7721..7799) opens at `+1` and ends 47 apart, which is
downstream of the word and nobody's.

**One cost, and it is the one run87 warned about.** run88 is 187 MB against
run87's 121, and `Log::parse` is still eager (item 260): with both windows in
it the release diff suite's peak measured **15,479 MiB on one run and 16,169
MiB on the next**, of the 20 GiB `memcap.sh` ceiling — a 700 MiB swing at two
threads, so the headroom is now under a fifth and the number is not stable
enough to plan against. The next window of this size wants item 260 landed
first, or a raised cap; this is the second capture in two days to say so.

**So the word does not move.** Nothing was fixed here; what the capture
buys is that East Indies 7529 stops being a draw-stream report and becomes
a field with a value diff beside it.
`run88_s_window_is_east_indies_word_frame` is the assertion, and its five
claims were each made to fail on purpose before it was believed: the word's
own row moved to `(7529, 1, 0)`, `1/13` dropped from the parted set, the
non-`+1` guard moved to `(1, 7, 335, 334)`, the step moved to `(−25, 0)`,
and the original's draw count moved to four. All five failed; the guard that
stays in permanently is the third, because "every row is off by one" is also
what a reader that subtracts wrong looks like, and `1/7`'s own `wait` opens
at 375 against 334.

## run89 — Great Lakes' word is a bird's arrival stand (2026-09-07)

**What it is.** run53's game, `[7514, 7760)` at run87's `[End Frame]` detail
**exactly** — **247 blocks, 112,314,402 bytes**, `cover=0`, eight minutes from
the start click to the last block. It is the first Great Lakes dump ever taken
at that map's own word, which stands at **7584**. run87 covered `[7244, 7520)`
and closed 7455; 267's `compute_form` tail negation then moved the word 129
frames **past run87's own window**, so until this capture the headline map's
frame could be read as a draw stream and as nothing else.

**The disk was grepped first, and it found one block.** Every archive, for any
block labelled 7500–7899 on any map: exactly two files have any — **run87**,
7500..7519 with its `!quit` block at **7536**, which is Great Lakes but 48
frames short of the word and at the shutdown dump's detail rather than the
window's, and **run88**, which is East Indies. So nothing on this disk held
Great Lakes 7584 and this was a capture rather than a diff.

**Four checks, and the middle one was a prediction rather than a
convenience.** `rngcmp.py rontrace-run53.log rontrace-run89.log` → **0
differing, 7,776 identical**. The overlap against run87's tail → **6 in common
(7514..7519), 0 differing**, and — as run88 got against run85 — **with no
`--exclude` and no `--drop` at all**: taking the neighbour's detail *exactly*
rather than nearly is what makes six blocks byte for byte, where run87's own
`GUYS` 2 → 4 against run79 needed three dropped `ANIMALDATA` fields (item
268). The window was widened to start at **7514** rather than the brief's
7530 for exactly that: `[7530, …)` would have left a ten-frame hole above
run87's tail and bought nothing.

**Two teeth, both made to fail on real data before the run.** `cur_anim` over
the window pins `GUYS=4` — 246 blocks, 7514..7759 — and prints `6
(7514..7519)` / exit 1 on run87, `0 (0..0)` / exit 1 on run79 and run86, and
276 / 326 with run87's and run88's own ranges. `collide_frame` pins `UNITS=3`,
which the first line cannot: a stanza that kept `GUYS=4` and dropped `UNITS`
would pass it and lose the five collision fields, the order list and the path
stack. It fails the same two ways, run53's thin 24k dump carrying none at all.
**run85's `collide_frame` *transition* is not asserted** — run88's worker
measured it over run86 and refused it, and the same holds here.

**The word reproduces from this capture's own trace, not run53's.** Frames
7580–7583 agree draw for draw (12, 19, 7, 9); **7584 is 48 against 49**,
parting at index **24**:

| index | ours | theirs |
| --- | --- | --- |
| 21–23 | `Animal::think_bird` ×3 | the same |
| **24** | `Animal::think_bird+0x82` | **`Guy::set_anim+0x97a < Guy::move+0x19f`** |
| 25–27 | `+0xa6`, `+0x1f8`, `+0x82` | the bird triple, one late |

Removing that one entry makes the two frames equal, entry for entry, so the
whole of the word is **one missing draw** and not a reordering.

**It is an arrival stand, and it is inside the animal pass.** `Guy::move+0x19f`
is `docs/ANIM.md` §9's `Guy::move:59` row — the frame after a walking guy stops
with the plain `WALK` slot and nothing else has changed it, asking for the idle
and rolling for it (`sim::anim::SITE_ARRIVE`). It sits between the **fourth and
fifth** bird's `Animal::think_bird` triple, which is one of the gaia birds'
figures coming to rest. *Which* bird this capture cannot say: player 9's herd
is not in the dump at this detail — `borrow_pasture` takes it from the trace —
so naming it wants a capture that carries the herd.

**And the field it costs is on the same frame.** Nineteen draws later 7584
spends two `GameAccess::rnd+0x20 < Unit::do_job+0x67` on **`1/3`**, an AI
citizen whose `GATHERORDER` works building `2002` (`build_type 417`,
`been_there 1`, `goto_build 1`, `non_flat_gather 0`, `group −1`). Both sides
push it a fresh `MOVEORDER` on block **7585**, which is the state at the *end
of sim-frame 7584* — the word's own frame, in the block numbering run88's
`1/13` used — and they disagree only on where:

| field | ours | theirs |
| --- | --- | --- |
| `x`, `dest_x` | 42168 | **41976** |
| `y`, `dest_y` | 17400 | **17208** |
| `off_x` | 696 | **504** |
| `off_y` | 504 | **312** |

**(+192, +192) on all four** — one tile in each axis — and `x − off_x` = 41472,
`y − off_y` = 16896 on **both** sides. The base the spot is measured from
agrees and the tile chosen off it does not, which is what a stream one draw out
of step looks like rather than a second, independent fault. The position
follows on 7586:

| block | ours | theirs | delta |
| --- | --- | --- | --- |
| 7585 | (41592, 17592) | (41592, 17592) | — |
| **7586** | **(41615, 17585)** | **(41609, 17575)** | **(+6, +10)** |
| 7599 | (41914, 17485) | (41830, 17354) | (+84, +131) |

**Nothing else parts at or below the word.** Over 11,620 unit fields, 11,571
order/path rows, 18,322 angles, 59,626 collision fields, 134,670 gather,
20,068 building and 7,861 queue rows across 247 blocks, the only unit parting
at or under block 7585 is `1/3` — beside run87's own carried residue, all of it
older than this window and present on its first block: `1/23`'s order angle and
flags, `1/24`/`1/25`/`1/26` twenty-four units off their cell, and item 242's
group id. Above the word there are 7,493 rows and **none of them is asserted**:
past 7584 both sides run on streams that are nobody's (run57's rule).

**One row for the widening ledger.** The **`CITY` record parts on every block
of the window** — 18 fields, 246 of 246 blocks: player 0's city `2000` reads
zero here for `busy`, `gatherers`, `land`, `filled`, `peasant_dist`, three
`space` slots and four `ter` slots, and player 1's `2000`/`2007` differ on
`land`, `space[2]`, `trade_val` and `vans.length`. No window test on either map
asserts `city_diverged` — only the 24,001 endpoint does — so this has been
true and unread for as long as the record has been compared. It is printed by
the test and named here rather than assumed away.

**One cost, and it is the first capture since 260 landed.** run89 is 112 MB;
the release diff suite's peak with it in is **10,310 MiB of the 20 GiB
`memcap.sh` ceiling** at 244 tests, against **8,321** at 243 without it — so
this window costs about **2 GiB of peak**, a third of what run87's 121 MB cost
before the parse went lazy. The headroom warning run87 and run88 both ended on
is closed: it was 15,479–16,169 of 20 then and is a little over half the
ceiling now, and 2 GiB is the figure the next capture of this size should be
sized against.

**So the word does not move.** Nothing was fixed here; what the capture buys is
that Great Lakes 7584 stops being a draw-stream report and becomes one named
call site with a value diff beside it.

> ~~*Which* bird this capture cannot say~~ — **it can, by elimination, and
> the word moved the next day** (item 284, 2026-09-07). This dump prints all
> forty of gaia's owner-8 animals in full and not one of them is on
> `CHAR_WALK` on block 7584 — the **slot** `Guy::move@005d9240:57` tests for
> — while the thirteen owner-1 guys that are on it all carry `stopped 0`. So
> no dumped figure could have spent 7584's stand, and the draw's neighbours
> place it in the animal pass: it is a bird's. The mechanism is
> `Unit::do_air_physics`'s own `set_new_location(x, y, 0, 1)` — `param_3`
> zero, so the figure is *told* where to be and not put there, lags its unit
> by a step, and `des == pos` in `Guy::move` reads as *the bird did not move
> this frame*. With it, **7584 agrees 49 for 49 entry for entry**, `1/3`'s
> spot on block 7585 closes, nothing at all parts at or below that block but
> run87's carried residue, and the word moves to **7585** —
> `Leader::make_stuff+0x63d`, the AI's slot expiry. `docs/SYNC.md` §3.9,
> "The arrival stand".
`run89_s_window_is_great_lakes_word_frame` is the assertion, and its eight
claims were each made to fail on purpose before it was believed: the extra
draw's index moved 24 → 25, the counts to 49/49, the parted set to `1/4`, the
step to 191, the base to 41473, the original's own spot to 41977, the parting
floor to 7586, and the anti-vacuity range widened to include 7584 itself. All
eight failed. The guard that stays in permanently is the last one, because
"equal once one entry is dropped" says nothing on a frame pair that was never
equal, and a mis-read trace is also a stream that agrees with nothing.

> **And the word at 7585 was read from this same file the next day, without
> a capture** (item 287, 2026-09-07). Nine tenths of it was a grep. The
> extra draw is `production_ai+0x236`'s step-11 `make_stuff`, and its
> `+0x63d` is step 6's expiry over a slot it has just bought; the trace
> shows **no draw between it and the `+0x221` pair**, so the purchase was
> draw-free. Widening the queue found the record: block **7586** takes
> player 1's city building `2007` from `queued 0` to `queued 1` with one
> item `type 50, job_counter 100, cost[0] 43` — a **Citizen**, at the price
> this crate's own tables give the next one, and the only queue row that
> parts at or below that block. So the slot is 5, and every gate of
> `make_stuff` but one agrees: the gate that fails is step 6's good loop,
> `88 < 55 + 43 + 4` over food. Given food 98 on that frame — `need` falls
> as the purse rises, so 98 and not 102 is the least that buys — this
> crate's 7585 agrees **nine draws for nine, entry for entry**, the bird's
> own `Guy::move+0x19f` at index 4 included, which is downstream of the
> missing draw. **The word did not move**: it is the AI's *stockpile*, not
> its rules, and the ledger is written only at `LEADERS=9` — no Great Lakes
> capture on this disk carries it past setup, so nothing has ever compared
> it. `docs/AI.md` §32 has the bounds (98 ≤ food < 160 against this crate's
> 88), the two seams upstream of it, and the one thing the 24k trace
> settles for free: **no draw in it is made from `use_market`, `do_sell`,
> `do_buy`, `market_speculation` or `calc_market_prices`**, so the market's
> sell branch — the only one that rolls — never runs in this game.

## run90 — East Indies' word is a waypoint this crate walks to (2026-09-07)

**What it is.** run54's game, `[7790, 7900)` at run88's `[End Frame]` detail
**exactly** — **111 blocks, 71,645,977 bytes**, `cover=0`, six minutes from
the start click to the last block. It is the first dump of **any** frame in
`[7800, 7899]` on either map, which is where East Indies' word at 7806
actually turns: run88 stops at 7799 and its `!quit` block sits at 7816.

**The disk was grepped first and it found one file.** Every archive, for any
block labelled 7790-7899 on any map: only **run88**, and only its own tail
(7790..7799) plus that 7816 closing block — which item 276 had already read.
So the whole shuffle was undumped and this was a capture rather than a diff.

**Four checks, and the first attempt was a permission failure, not a
result.** Launched from the session's own process tree, `screencapture`
wrote nothing — Screen Recording is granted to `~/bin/RonDriver.app` and not
to the current Claude Code version's path — and `runqueue.sh` said so and
stopped. Re-launched through `tools/gamelog/viadriver.sh` it ran clean:
`rngcmp.py rontrace-run54.log rontrace-run90.log` → **0 differing, 7,916
identical**; the run88 overlap → **10 in common (7790..7799), 0 differing**,
with no `--exclude` and no `--drop`, which is what taking the neighbour's
detail exactly buys; and both teeth → **110 frame blocks, 7790..7899**,
`cur_anim` for `GUYS=4` and `collide_frame`'s *presence* for `UNITS=3`. Not
run85's `collide_frame` **transition**, which run88's worker measured at
zero over 486 East Indies frames and refused.

**This capture was booked to be refused, and the predictions were written
into its stanza before the run.** Item 276 had dated the word off the trace
and produced this crate's own `collide`/`pause`/`wait` rows for `1/6` and
`1/7`; the stanza carries them with a line each saying what a refusal would
mean. Two held exactly, two were refused, and the refusals are the finding.

**Held.** Both sides make the first collision on block 7803 — `1/6`
`collide_o 7`, `1/7` `collide 1 / collide_o 6` — and `1/7`'s `MOVEORDER
pause` reads **3, 3, 2, 1, 0** over 7803-7807 on both sides.

**Refused: the position parts two blocks below the word.** `1/6` is out on
**7805**, where the draw stream does not part until 7806:

| block | ours | theirs | delta |
| --- | --- | --- | --- |
| 7804 | (39729, 38802) | (39729, 38802) | — |
| **7805** | **(39720, 38808)** | **(39729, 38802)** | **(−9, +6)** |
| 7806 | (39720, 38808) | (39708, 38789) | (+12, +19) |
| 7807 | (39699, 38795) | (39708, 38789) | (−9, +6) |

**And the mechanism is the waypoint's arrival rule, not the response.** The
original pushes the *identical* sidestep — `(39720, 38808)` `flags 2` on
7803, `(39672, 38808)` on 7807, `(39624, 38808)` on 7811 — and its path
stack drops 5 → 4 on the **next** block with the unit at (39729, 38802),
which is not that point: one full step along the bearing and the waypoint is
abandoned. This crate keeps it and walks the remainder, a short (−9, +6)
step. The original's blocked cycle is **four** blocks; this crate's is
**five**; the extra frame per cycle is the word.

**Refused, second: the pause roll on 7810 is `1/7`'s.** §8.5 could not
attribute it. `1/7`'s `MOVEORDER` carries **`pause 8`** on block 7811 where
this crate sets §6 step 5's wait flag and rolls nothing — the
wait-versus-repath predicate is wrong. It is also the **first non-zero
`MOVEORDER pause` any dump on this disk has printed**: every one of run88's
1,467 is 0, which was `docs/COLLISION.md` §9's standing row. A rule comes
with it that no reading had stated — the countdown is **frozen while
`collide` is set**, holding 8 over eleven blocks (7811..7821) before ticking
down to 0 on 7829.

**The trace and the dump corroborate on the same three frames**, which a
closing block never could: the original's `SITE_BLOCKED` falls on sim-frames
7802, 7806, 7810, and the blocks whose `1/6` names `collide_o 7` are 7803,
7807, 7811 — the same three, four apart.

**One cost, and item 260 has closed the one run87 and run88 both warned
about.** run90 is 71 MB, and the release diff suite with this window in it
peaked at **11,015 MiB across the tree, 10,984 in the largest single
process, of the 20 GiB `memcap.sh` ceiling**, at 245 tests. That is at or
under the ~11.1 GiB the tree measured at 244 without it, so a 71 MB window's
cost sits **inside the run-to-run swing** and is not separable here — the
delta is not claimed. It is nowhere near the 15,479-16,169 of before item
260, and run89's ~2 GiB for 112 MB remains the only figure on this tree that
actually isolated a window.

**So the word does not move.** Nothing was fixed here; what the capture buys
is that East Indies 7806 stops being a draw-stream report and becomes a
field with a value diff beside it — **two frames earlier than the stream
noticed**, which is the standing rule in one row.
`run90_s_window_is_east_indies_shuffle` is the assertion, and its six claims
were each made to fail on purpose before being believed: the original's 7804
path length moved to 5, its `pause 8` to a 9, `1/6`'s 7805 row to
(39730, 38802), `1/7`'s first parted block to 7812, the parted set's `1/6`
to 7807, and the stand list's 7810 to 7811. All six failed.

**And the same day the reading landed and the word moved: 7806 → 7812**
(item 289, `docs/COLLISION.md` §8.7). The rule is `move_step`'s ordinary
post-step arrival test — Manhattan, against `UnitData::tolerance` — and
`resolve_unit_collision`'s sidestep push never writes that field, so the
waypoint is walked under the interrupted leg's 384 and retired from fifteen
units away. One deleted line. `1/6` parts at 7827 instead of 7805, its
collision fields and its whole order record agree over the cycle, and this
capture's own assertions turned over with it: the parted set, `1/6`'s and
`1/7`'s first rows, and a new pair pinning the four-block cycle from the
dump's side. run88's closing block loses `1/6` from its residue at the same
time.

**The capture is why the reading was cheap.** Every field the rule turns on
— the path stack, `dest`, the order's `dest_x/dest_y`, the position — is
printed on every block of the cycle at `UNITS=3`, so the 4 → 5 → 4 with the
unit *not on the point* is one record's own three rows and the decompile was
only asked to name the store that is missing. This is the "diff first, then
read what no run reaches" rule paying twice on one window: the capture
refused the reading it was booked to test, and then made the next reading a
twenty-minute one.

## run92 — Great Lakes' extra waypoint is `get_loc`'s second arm (2026-09-17)

**What it is.** run53's game, a **`GROUPS`** window over `[7668, 7690)` —
**23 blocks** (22 window plus the `!quit` block at 7701), **21,362,086
bytes**, `cover=0`, **four minutes** from the start click to the archive,
of which the 7,668-frame run-up was ninety seconds. It exists for one
field: the group's own `(ox, oy)` on the frame its leader plans.

**The disk was grepped first and came back empty.** Five Great Lakes
archives carry a `GROUPDATA` at all and the highest frame any of them
reaches is **5591** — run73's `DUMP_ALL` window; run34 stops at 6, run46
and run50 at 901, run72 at 4811. So the headline map had no group record
within two thousand frames of its word, and this was the rare item where
"grep the disk before booking a capture" argues *for* the capture.

**The detail is run45's, and the trap is the whole reason.**
`end: MISC=9,UNITS=9,GROUPS=9,GUYS=9,LEADERS=1` with **`DEATHS` off**:
`GroupData::log_data@0045e1d0` sets no type of its own, and
`dump_deaths@0092fd80` ends by calling `WorldData::log_data` twice, so with
`DEATHS` on the pool is accepted against `WORLD`'s threshold and vanishes
silently ("The group pool is a per-frame record", above). At `UNITS=9` the
window is nobody's sibling by detail, so there is no `samegame.py` here —
the same-game claim is the draw stream instead.

**Three checks, and one of them was wrong about its own capture.**
`rngcmp.py rontrace-run53.log rontrace-run92.log` → **0 differing, 7,701
identical**; the pool tooth → **22 blocks (7668..7689)**, made to fail
first and printing `0 (0..0)` on both run89 and run73. The third asserted
the window's last block at **7700** and the dump says **7701**: `!quit` at
frame `N` writes its block at `N + 1` (run89's `frames: 7775` closes at
7776), and the stanza's author read that as "fifteen past HI" rather than
`FRAMES + 1`. The count and the lower end were right; the prediction was
off by one and is corrected in the stanza rather than quietly fixed.

**And the field decided it in one read.** On block 7674 group `64` carries
`(ox, oy) = (36303, 23348)`; its leader `1/37` stands at `(43174, 24583)`,
**6981** away, so `get_loc`'s first arm is refused; the leader's current
move order's destination is `(36600, 23400)`, **301** from `(ox, oy)` and
inside the `0x180` the second gate allows. So the original's group location
is `(42877, 24531)` — cell **(55, 31)** against the leader's own **(56,
32)** — and the search starts one diagonal cell along, inside the very cell
this crate had to emit as a head waypoint. `docs/GROUPS.md` §12.5 has both
gates' operands off the listing; the word moved **7679 → 7930**.

## run94 — the AI scout's explore leg, and the danger term it turns on (2026-09-17)

**What it is.** run53's game, a `UNITS=3`/`GUYS=4` window over
**[7754, 8045)** — 292 blocks (291 window plus the `!quit` block at 8061),
**134,579,630 bytes**, `cover=0`, **nine minutes** from the start click to
the archive, of which the 7,754-frame run-up was about ninety seconds. It
exists for one unit's record: the AI's scout `1/0` over the frames it is
given an explore order and walks it.

**The disk was grepped first, and this time it argued for the capture.**
No archive on either map carries a block labelled 7937–8173: Great Lakes
runs run89 [7514, 7776), run93 [7929, 7936] at `MISC` alone, and then
nothing until run80's 23960; East Indies stops at run90's 7916.

**But run89 was read before this was booked, and it moved the item.** Its
247 blocks carry the same unit's whole record — position, order and the
`PATHDATA` stack — and `RON_DEBUG_ROWS=7514-7780` prints 1,080 diverging
rows over the window of which **not one is `1/0`**. So the scout's planner
was not wrong in general, which is what made 8030 a value question rather
than a patch. `report.py … when Unit::do_move` said the same from the
other side: the original's nearest `Unit::do_move` draws either side of
8030 are **7676 and 9443**.

**Three checks, all ok first time.** `rngcmp.py rontrace-run53.log
rontrace-run94.log` → **0 differing, 8,061 identical**; the run89 overlap
→ **6 in common (7754..7759), 0 differ**, with no `--exclude` and no
`--drop`, which is what taking the neighbour's detail exactly rather than
nearly buys; and the `to_x` tooth → **291 (7754..8044)**, made to fail
first on run89 (6) and on run53 (0).

**What it says.** Block 8002 has `1/0` taking an `EXPLORETOORDER` to
**`(4344, 32760)`** with a six-entry stack bottoming out on `(4320,
32736)`. This crate was sending it to `(2808, 31992)`: frame 8001 spends
the same **38 draws on both sides, entry for entry**, over five candidate
cells ringing the *human's* city, and the score that separates them adds
`danger[who][(y >> 1) * reg_xs + (x >> 1)] ` — a term
`Sim::scout_danger` returned a flat zero for, behind a doc comment
claiming the grid was keyed differently from the one this crate already
has. It is not (`docs/SCOUT.md` §8.2). Routed, the destination is the
original's and Great Lakes' word moves **8030 → 8031**.

**And the window dates its own successor.** `1/0` carries no row at all
below block 8002 — 248 blocks of exact agreement — its order parts on
8002 where the two `find_wpath` routes separate (the original swings a
cell west of the city's footprint, `(2040, 31224)`, where this crate keeps
to column `2808` and runs through it), and its **position** on 8014.
`run94_s_window_is_great_lakes_scout_repath` is the assertion and it pins
both dates.

## 178 needed no screen — the danger map's unit pass was on disk four times (2026-09-06)

**What it is.** Not a run. `docs/DANGER.md` §8 had the unit pass of
`GameDaemon::calc_danger` down as reading-only — `role & 0x10000`,
`(attack · 5) / 10` and the war gate — because "no capture has a military unit
on a frame divisible by 200", the rebuild boundary. The grep that was meant to
choose *which* capture to book closed the item instead.

**Two greps, and the first one confirmed the premise.** Every archive in
`Logs/` was scanned for `danger[who][scan]` and for the frame block it sits in:
71 files carry the map at all, 22 carry it per-frame, and of every block on
disk **exactly two are on a rebuild frame** — run65's East Indies 6200 and
run72's Great Lakes 4800. Neither has a single unit whose type carries the
military bit. So the premise held: a capture *was* owed, and the target was
picked — Great Lakes frame **7000**, where run79 already shows six Longbowmen
(guy type 177, `role 0x150c00`) in two clusters and the same 25 buildings
stand at the same positions, hit points and flags as at 6800 — so
`dump1 ≠ dump2` in that one block would be the unit pass and nothing else.

**Then the second grep made the capture unnecessary.** The dump does not have
to be *contemporaneous* to be decisive, because the two passes write different
shapes. The building pass writes a 3 × 3 of half-cells to **every** active
viewer, the owner included; the unit pass writes **one** half-cell and never
the owner's. So a half-cell with no building anywhere in its 3 × 3 is zero in
all eight rows from the building pass, and anything non-zero there is the unit
pass. Every combat window on disk has exactly two of them:

| archive | block | rebuilt at | half-cell | `danger[0]` | `danger[1]` |
| --- | --- | --- | --- | --- | --- |
| run26 | 12024 | 12000 | (27, 20) | **30** | 0 |
| run26 | 12024 | 12000 | (28, 21) | **212** | 0 |
| run29, run27 | 15100 | 15000 | (22, 28) | **30** | 0 |
| run29, run27 | 15100 | 15000 | (23, 29) | **217** | 0 |

Leader 1's units standing on them, and the type table in the same dump, give
`(attack · 5) / 10` halved once by `do_danger`'s enemy arm: 324 → 30,
334 → 132, 340 → 50, 341 → 55. **30 + 132 + 50 = 212** and
**30 + 132 + 55 = 217**, and the five between the two frames is `1/32`'s
upgrade from type 340 to 341 — (110 − 100) / 2. The whole of
`docs/DANGER.md` §8.1 falls out of that, negative side included: run26's 12024
has 45 half-cells outside every building's 3 × 3 holding only non-military
units, and all 45 are zero in all eight rows (run29's 15100 gives 44 of 44,
run25's 12129 45 of 45).

**And run25 is the control nobody designed.** Its block 12129 prints the same
two values at the same two half-cells with the units **already gone** — the map
is the 12000 rebuild's, 129 frames stale, and that is §2's "nothing decays
between rebuilds" as a dump rather than a reading.

**So no capture was booked.** The Great Lakes 7000 stanza was drafted, costed
(~154 MB: a 29.6 MB start dump plus two 61.6 MB `DUMP_ALL` blocks, on a
fifty-second run-up) and then not spent, because the only thing it would add
over the archives is a *contemporaneous* whole-map assertion, and the four
reading-only claims that remain — the peace arm, the garrisoned case, the
`LEADER_VALID`/`LEADER_ACTIVE` split, an upgrade inside `attack()` — are none
of them reachable by it. Two leaders at war with an army on the map is exactly
what run26 already is.

**What it left behind.** `tools/gamelog/danger.py`, the third reach for the
same probe shape and so a tool: `danger.py map FILE FRAME [--differ|--same]`
compares the two `FULL DUMP`s' maps in one block, and `danger.py units FILE
FRAME [--types A] [--least N]` lists the units whose `role` has the military
bit with the half-cell each indexes and what every row holds there. Both were
made to fail first, on real data, in both directions: `--differ` exits 1 on
run72's 4800, run65's 6200 and run29's 15100 and 0 on nothing yet; `--least 3`
exits 0 on run79's 7000 and run76's 6800 and 1 on run72's 4800 and run65's
6200. It reads the archive with **indentation intact** — `frame.py` strips it,
and a record then has to be closed by name rather than depth, which silently
hands the last `UNITDATA` of a dump the `BEGIN GUY` records of the top-level
guy pool and all 674 `GROUPDATA` after it. That bug inflated the first military
census of run29's 15100 by one unit with sixteen types.

**One thing the reading turned up that no dump can settle.** `calc_danger`'s
building pass walks its **viewer** loop to the end of the leaders array
(`local_28 < 0xe71af0`), where the clear loop and the whole unit pass walk
exactly eight slots. This game's leader 8 — Gaia, whose `leader_flags` is
`0x2000007`, both bits set — is therefore an active viewer of the building
pass and there is no `danger[8]`. Whether that writes past the array or the
loop bound is a decompiler artefact is a listing question, not a capture one.

## 117 is not a lobby click — the handicap branch no single-player game can take (2026-09-06)

**What it is.** Not a run, and the second item in a row the lane closed without
one. 117 was booked as the last of `docs/ATTRITION.md`'s four flat territory
terms a capture could still reach: `(handicap + 15) / 25`, inert in every dump
because `handicap` reads 0, and reachable — run80's section said — by changing
the lobby difficulty, "a click, not a longer wait". The brief asked, before
anything else, whether the lobby handicap is reachable through the driver at
all. It is not, and the reason is not the driver.

**The good news first: it is not a click at all.** `check.ini` already carries
`PLAYER0_HANDICAP=Standard` and `PLAYER1_HANDICAP=Standard`, and
`GameInfo::load_from_config@005d4da0` matches each by **exact name** against
`rules.xml`'s `handicaps` category — 21 entries, `Standard` then `Skill +1` to
`Skill +20`, `DATA` = index × 5 — and stores the matched **index** into that
player's slot. So the lobby half needs no UI path: it is a file key of the same
shape as `PLAYERn_TRIBE`.

**The bad news is a gate, and it is exhaustive.** `compute_reg_territory@006b0bb0`
line 255 reads the handicap only when `leader_flags & 4` (~~an AI~~ — **not
the computer-leader test**; item 437, `docs/COMBAT.md` §28.1, and what the
bit is was not established) **and** `Game::semaphore` bit 2 are both set:

```
if ((leader_flags & 4) == 0 || (game->semaphore.ptr[0] & 4) == 0) v = 0;
else                                                             v = get_handicap();
territory_bonus += (v + 15) / 25;
```

That bit is **set in exactly one function in the executable** —
`Game::run_gamespy@00587060`, the GameSpy multiplayer path — and it is
explicitly **cleared** by `Game::run_solo@00587830`, `Game::run_scenario@005860c0`,
`Game::run_editor@00586440` and `RecordGame::read_package@00952d90`. Checked by
grepping every write to `Game::semaphore`'s byte 0 across all 48k exported
functions, in both the inline (`*p = *p | 4`) and out-of-line
(`BitMask<256>::set`) forms: the console sets `game->semaphore` bits **1, 11
and 12** and never 2, and its two computed `BitMask<256>::set`/`toggle` calls
target `MiscAccess::scene->flags` and a leader's tech mask, not the game.

The gate is not local to territory. `LeaderData::get_handicap@006da740` has
exactly two live callers — `compute_reg_territory` and
`ObjectData::train_time@006508c0` — and **both** carry it, as does
`Game::init_teams@0058ae70`, the one other reader of the lobby handicap.
(`LeaderData::get_handicap_level@006d6740` returns the raw field and has no
callers at all.) So the whole handicap mechanism is multiplayer-only, and
`Game::run_solo` turns it off on the way into every skirmish this lane runs.

**Two things the reading corrected on the way past.**

- **The term is not `(handicap + 15) / 25` on the field.** It is
  `(get_handicap() + 15) / 25`, and `get_handicap` returns
  `handicaps.list[handicap].DATA` — index × 5 — so the allowance runs **0 to
  4**, where reading the raw field would cap it at 1. Exactly the "which array
  a level indexes" predicate `docs/audit/README.md` says is where the errors
  are.
- **`LeaderData::handicap` is not the lobby's `PLAYERn_HANDICAP`.**
  `Game::init_handicaps@0058abf0` recomputes it per leader as
  `clamp(strongest team's summed handicap − own team's, 0, 20) /
  max(num_teams − 1, 1)` — a catch-up deficit clamped to the list's own index
  range. So equal lobby handicaps leave every leader at 0 whatever the value,
  and it is the **weaker** side that would be paid: to hand the AI an
  allowance you raise `PLAYER0_HANDICAP`, the human's.

**What was not spent.** A run84-shaped capture was ready — Great Lakes,
`PLAYER0_HANDICAP=Skill +20`, `frame_window` [6950, 7030) at run84's own
`LEADERS=9` detail, with `rngcmp` against run53 and `samegame.py --exclude
LEADERDATA` against run84 as the falsifier that the handicap changed nothing.
It would confirm a negative the decompile already settles exhaustively, and the
brief's instruction was to stop and say so rather than work around it. **117 is
not a scripted-setup item either**, because `run_scenario` clears the same bit.
It is a multiplayer item, or nothing.

## run101–run105 — the golden record's first staged run (2026-09-18, item 363)

The rules track's first capture (`docs/DECISIONS.md` entry 41 §1): one staged
game on map 14 at seed 12345, the Leader AI silenced at frame 0, a late age, two
armies and a war, driven entirely from `tools/gamelog/golden/chapter1.cmd` and
launched click-free. **Five launches of one script**, outside the `Logs` archive
because §2 says regenerate rather than store — the tree is `~/ron-golden/`, the
runbook is `docs/ORACLE.md`, "The click-free lane needs a window", and every one
is reproducible from the commands below in about half a minute of wall clock.

| run | dir | window | detail | what it is |
|---|---|---|---|---|
| 101 | `g1` | `[880, 900)` | end + `LEADERS=2` | the digest: the whole 901-frame trace |
| 102 | `g2` | `[880, 900)` | the same | the second launch, for the identity check |
| 103 | `g3` | `[605, 900)` | the same | the detail on demand: the fight, whole |
| 104 | `c1` | `[605, 900)` | the same | **the control**: chapter one *without* `!ai off` |
| 105 | `g4` | `[605, 900)` | + the `[Start Game]` set | the parse subject for `rondata` |

All five: `success: true`, exit 0, 901 frames, closing dump at 901, `MAP_STYLE 14`
and seed 12345 read back from the dump's own `GAME INFO`, five settings files
restored and byte-verified. Launch-to-exit ran 24 s for a digest and 145–172 s
for a detailed window.

**Every command ran.** `cmdsran.py`'s question, answered from the trace: eight
`INFO cmd` records, every one returning 1 — `!ai off` at 0, `!ffwd 1` at 37,
`age who=0 8` at 600, `age who=1 8` at 602, `war` at 604, `add hoplite who=0
4,40` at 610, `who=1 5,40` at 615, `!quit` at 900. A `!` line logs half 0 and a
cheat line half 1, which is `run_cmd@007d6a70`'s two switches showing through.

### Auto-engage survives AI-off, and the control is what makes it a measurement

The reading said it would: in `Unit::think@005f6e40` the `think_attack` and
`add_attack_order` arms all sit **above** the line that reads `GameAccess::ai_off`
(`if ((leader_flags & 4) != 0 || ai_off != 0)`), so nothing about auto-engage is
downstream of the gate. The run says it does, and the value diff is the dump's
own, either side of the frame it moved:

```
frame 615   who 0  (888,7800) (1032,7800) (936,7944)   orders_x = x, whom -1, no ATTACKORDER
frame 616   who 0  unchanged
            who 1  (1368,7992) (1512,7992) (1416,8136) BEGIN ATTACKORDER, whom 0
frame 620   who 0  (840,7800) (1032,7800) (972,7988)   ATTACKORDER
            who 1  (1368,7992) (1431,7915) (1304,8116) ATTACKORDER
```

Three squads are born at 615 and engage on the frame they appear; by 620 all
six, **both owners**, carry an attack order and their coordinates have moved
toward each other. No order was ever issued by anyone — the channel has no order
verb, and `ai off` is the chapter's first line. By frame 800 two `DEATH_OBJS`,
by 899 three, and the roster is down from 58 to 55.

**What would have falsified it**: zero `ATTACKORDER` blocks on any frame after
615. What would have made it vacuous is `ai off` not taking, so run104 runs the
identical script with that one line deleted: **899 of 901 frames differ, first at
frame 2**, 2 identical. The line changes the simulation from the second frame,
which is what `issue_cheat_ai_toggle` travelling in the order stream looks like
(`Game::action_cheat_ai_toggle@005930c0`,
`CommandPackage::process_cheat_ai_toggle@00944d20`). Note that `ai off` does not
clear a leader's own computer bit — ~~who=1 still carries `leader_flags`
bit 2 at frame 899~~ — so the two are independent, exactly as the `||`
reads. **Corrected 2026-09-19, item 437** (`docs/COMBAT.md` §28.1): who=1
does **not** carry that bit. This capture's own dump prints
`leader_flags` — immediately *before* each `BEGIN LEADERDATA`, which is
why it was read as absent — and who=0, the **human**, is `0x00000007`
with bit 2 set while who=1, the **computer**, is `0x03000013` with it
clear. run112 and run105 agree exactly, so it is a lobby property and not
`ai off`'s doing. The independence of the two words stands; the reading of
which is which does not, and what the bit *is* was not established.

### Two launches are one game, and so is a re-run at three times the detail

`rngcmp.py` over the per-frame `game_random` word, the whole length:

```
run101 vs run102   901 frames in common, 0 differing, 901 identical
run101 vs run103   901 frames in common, 0 differing, 901 identical
run101 vs run105   901 frames in common, 0 differing, 901 identical
```

`samegame.py` over run101 and run102's dumps: **21 blocks in common, 0
differing** — frame 1 and 880..899; each file's 22nd block is its `!quit` block,
which the tool drops as the highest. The count is the claim, not the verdict:
`samegame.py` exits 0 when nothing is in common at all, so the number is what
says the comparison happened.

**What would have falsified it**: any differing frame, or a common count below
21 — and the count had to be re-derived rather than copied, because run99's
overlap was 9 where run100's was 10 for the same reason.

**run101 against run103 and run105 is `docs/DECISIONS.md` entry 41 §2 measured
rather than assumed.** run103 dumps 295 frames where run101 dumps 20, and run105
adds the whole `[Start Game]` set on top — 46 MB and 14 MB against run101's 2.5
— and all three are the same game frame for frame. The logging detail is not in
the simulation, which is what "regenerate rather than store" rests on: a window
can be moved and re-taken at any detail without asking whether the run it
describes is still the same run.

### A late-age dump parses, and the first thing it says is the divergence

`age who=0 8` lands `ages_get() 7` in `LEADERDATA` at detail 2 — the eighth and
last age, zero-indexed — with `epochs_get() 0`, so **`age` moves the age alone
and leaves all four epochs Ancient**; `library` is the lever that moves both, and
a chapter that wants a genuinely late-age leader wants it.

`cargo run -p rondata -- <install> --gamelog <run105> --diff 20` reads the whole
thing: 52 units, 13 buildings, 4 leaders, 2 cities out of the start block, 24
structural checks green, 296 frames, 3,600 world cells and 57,600 tile masks out
of `WORLD`, then 20 frames stepped and **252 unit-frames compared** with every
order list and path stack read. Nothing failed to parse.

Two things it reports that item 364 needs and neither is a parse failure:

- **A cheat-spawned unit is `unlinked`.** `unit 0/6`, `0/7`, `0/8` from frame
  611 and `1/6`, `1/7`, `1/8` from 616 — six units the harness has no unit for,
  because they appeared without a production order. That is what a staged spawn
  is, and the golden record's interpreter (item 364) is what gives them one.
- **The first order disagreement is the auto-engage itself**: `who 1 o 6, 7, 8:
  first order disagreement at frame 616 — Kind { ours: 7, theirs: 10 }`. Kind 10
  is the attack order the original gave them on the frame they were born; ours
  is 7. The chapter found its own first divergence without being asked to.

One structural check fails, and its premise rather than its subject is what is
wrong: *"every starting citizen's derived GATHER target matches the one the
original issued — no unit in the first logged frame holds a gather order, is the
dump below `UNITS=3`?"* The dump is at `UNITS=3`; the first logged frame is 605,
not 0. The check assumes a capture that opens on the citizens leaving the
capital, which a windowed golden record never does. It needs to know it is
looking at one — 364's, with the floors.

### Regenerating any of them

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/g1 \
    --map 14 --end-frame 900 --log-window 880 900 \
    --detail end:UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=2,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter1.cmd
```

run103/run105 differ only in `--log-window 605 900` and, for run105, a `--detail
start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1`.
run104 is run103 with the `0 !ai off` line stripped from the command file.
`viadriver.sh` is not optional: a launch from inside a session's own process
tree gets no window and dies in 3.8 s.

## run106 — the held-out third map, measured once (2026-09-18, item 363)

**Map 9, Himalayas.** Every scored capture since 08-24 has been Great Lakes or
East Indies, and every rule in the simulation was fitted against one of the
two. A third map nobody has debugged against is the only thing that says
whether those rules generalise or were tuned. `docs/DECISIONS.md` entry 41's
"the held-out third map's word, measured and never debugged against" did not
name one; this is the choice and the reasoning, so that a later pass can
disagree with the choice rather than re-derive it: **9 is the map least like
both**. 14 is land around lakes, 18 is islands — two water maps — and the
Himalayas is the terrain neither tests.

**One capture, one measurement, and this section is the whole of what was done
with it.** No item has been opened against it, nothing has been read into it,
and the number below was produced by a single invocation.

Seed 12345, 1,900 blocks over `[1, 1901]`, the `[Start Game]` set and the
frontier's `[End Frame]` detail, no cheats but the fast-forward, on the
click-free lane's own lobby. `MAP_STYLE 9` and the seed read back from the
dump's own `GAME INFO`; 189 MB; 522 s launch-to-exit; five settings files
restored.

```
cargo run -p rondata -- <install> --gamelog <run106> --trace <run106 trace> --diff

  1899 frames stepped, 28860 unit-frames compared
  ticks before divergence:        1
  ticks before an order diverges: 0
```

**Against floors of `EastIndies 1851/1850` and `GreatLakes 1772/1772`, the
held-out map is 1 and 0.** That is the number, and it should be read as the
generalisation gap rather than as a bug: nothing here was ever fitted to this
map, which is the entire point of holding it out.

What makes it informative rather than merely bad is what still holds:

- **The one check that derives rather than compares passes.** *"Every starting
  citizen's derived GATHER target matches the one the original issued — 10
  citizens, derived from §9.3 without reading the log"*: `0/1→2001 0/2→2001
  0/3→2002 0/4→2003 0/5→2004` and the same five for player 1. The opening
  economic assignment is computed from the map, not remembered from two maps,
  and on a third map it is exact for all ten.
- **Units track individually for hundreds of frames.** `0/2@501 0/1@461
  0/3@237 1/1@344 1/8@323 1/7@209`; the ticks figure is the *minimum* over
  sixteen units, and one unit at 2 is what sets it.
- **The earliest breaker is player 1's unit 0**, the AI's scout: a move
  destination `x` of 42,744 against the log's 39,672 **at frame 1**, and a
  `mylos` of 6 against 4 from frame 202 that persists for 1,698 frames. One
  unit, one wrong destination on the first frame, and the map's whole
  trajectory follows it.
- The 44,597 order disagreements are dominated by `gather` (23,472) and `move`
  (11,591) — the two families the existing maps' items have spent eight months
  on, which is the shape of a fit rather than of a missing mechanic.

**What this number is not.** It is on the click-free lane's own lobby
(`-automation +skipIntro`, the profile's match rules) rather than the scored
captures' `-config check.ini`, and its window opens at frame 1 rather than
matching run33/run39's shape. So it is comparable to 1,851 and 1,772 **in kind
and not in provenance**, and a later pass that wants them on one axis owes a
run33-shaped capture on map 9. Nine `unlinked` units from frames 1214, 1603
and 1838 are the AI's trained units, which the harness does not produce; 2,137
of the 28,860 unit-frames are theirs.

**A trap the lane shares with `captures.txt`'s `poll_max`, and it cost this
capture once.** `unattended_capture.py --timeout` defaults to **180 s**, and
the give-up is a **truncation, not a stop**: the first attempt came back with
593 of 1,900 blocks in a 62 MB file that looks entirely ordinary. The receipt
is what caught it — `success: false`, `TimeoutExpired`, and the settings
restored and byte-verified anyway — so read `receipt.json` before the dump.
Size the timeout from the *dumping*: this capture ran 3.6 blocks a second.

## run107 — the leader's ledger at Great Lakes' own word (2026-09-18, item 369)

**What it is.** run53's game, a `LEADERS=9` window over `[9170, 9200)` at
run97's detail exactly but for that one category, `!quit` at 9215,
`cover=0`, 68,290,869 bytes, **five and a half minutes** launch to
archive. `docs/AI.md` §44 is what it decided.

**Why it was owed, and the disk was grepped first.** Great Lakes' draw
sequence parts at **9182** and nothing anywhere on this disk carried the
AI's goods or its make list near it: the map's `LEADERS≥2` windows are
run84 `[6950, 7030)`, run91 `[7514, 7600)`, run19 `[8174, 8192)` and
run80 `[23960, 24000)` — a gap of eleven and a half thousand frames with
the word inside it. The two captures that *do* cover 9182, run97 and
run100, are both `LEADERS=1`, which stops after `score`. Item 368 had
already taken run19's window as far as it goes.

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run53.log` | **0 differing**, 9,216 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 14, seed 12345 |
| the window, block for block | **30 blocks, 9170..9199, no gap** |
| the raised category is in them | `PERSONALITY` ×4 and `MAKEOBJECT` ×22 on 9170, 9183 and 9199 |
| `samegame.py --exclude LEADERDATA` vs run97 | **30 in common, 0 differing** |

The third of those is the guard parked 373 asked for — the click-free
lane's give-up truncates rather than stops, and a short window reads as
completely ordinary — and this stanza asserts its own block count against
the window it asked for. It is the first capture in this file to do it.

**The detail is run97's but for `LEADERS`**, which is what run91 did to
run89: the overlap check then needs exactly one `--exclude` and no
`--drop`. The brief asked for `GUYS=2`; `GUYS=4` is run97's own and costs
about 0.4 MB a block more, which over thirty blocks buys an exact
same-game check against the capture that already covers these frames.

**What it says.** The original's `bucket` at 9182 is this crate's own
`73 84 35 111 71 0` and does not move across the frame, so both branches
`docs/AI.md` §43 named — which were both about the bucket — are refused.
The head is what differs: the original holds **Mercenaries** (573) at
`val 9,999,999` from block 9179 to the end of the window, where this
crate writes 6,600,000 on the same rebuild and loses the head to a
Scholar two frames later. `1/MAKE[0].val` on blocks 9179 and 9180 is the
whole parting, two frames wide.

**The comparison is `run107_s_window_is_the_leader_record_at_the_word`**:
60 blocks, 62,640 field-frames, 109 fields of residue pinned by name.
Made to fail on purpose by adding one to the comparison's own
`territory`, which puts a row on all 30 blocks of both leaders.

**What it also opened.** The ten `SITE` slots of player 1 part on every
block with the same ten sites in a different order — a ranking, not a
survey — and `tech_frame` and `tech_cat_frame[0..3]` are at nought here
against five stamped frames. Neither had ever been compared on this map.

## run108 — the search's own answer, through the call proxies (2026-09-18, item 386)

**What it is.** Chapter one again, on run101's setup exactly — map 14, seed
12345, the same `tools/gamelog/golden/chapter1.cmd`, `!quit` at 625 instead of
900 — with the trace's **call proxies** turned on over the engagement frame and
three new sites in the `CALLS` table. It is the first capture taken to read a
function's *answer* rather than the simulation's state, and it exists because
no dump could answer the question:

- `ObjectData::near_o`/`near_who` record the **nearest** candidate a search saw
  (`00649...`'s `if (dist < best) near_o = o`), not the one it returned;
- the `ATTACKORDER` that lands records only the winner.

So a search whose candidates tie — which is what `docs/COMBAT.md` §18 said
chapter one's is — leaves no trace of the tie, and any constant that produces
the winner fits. `find_nearby_target`'s entry and return **bracket** one search
the way `do_air_physics` brackets a bird's frame, and the two sites inside it
are the two numbers the ranking is built from.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/t1 \
    --map 14 --end-frame 625 --log-window 614 622 \
    --detail end:UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=2,DEATHS=1 \
    --cmd-file tools/gamelog/golden/chapter1.cmd \
    --callwin 614 618 --tracer-def RON_TARGET_PROBE
```

`--callwin` and `--tracer-def` are new pass-throughs (`live_session.stage`
wrote `callwin=0-END` unconditionally before, and `unattended_capture.py`
hard-coded `TRACER_DEFS`); the proxies log every call in their window, so a
probe that needs four frames now says so. `RON_TARGET_PROBE` adds
`Object::find_nearby_target@00648da0`, `ObjectData::attack_dist@006488f0` and
`Object::compare_target@0064e5c0` as sites 8, 9 and 10, and `tracer.c` refuses
a build that also defines `RON_TURN_PROBE`, which claims 8 and 9.

`success: true`, exit 0, 626 frames, `lifecycle_verified`, map 14 and seed
12345 read back, five settings files restored. Launch to exit 18.9 s.

### The run is run101's game, and the proxies do not perturb it

The frame-616 dump is g3's to the digit: the six hoplites at `(888, 7800)`,
`(1032, 7800)`, `(936, 7944)`, `(1368, 7992)`, `(1512, 7992)`, `(1416, 8136)`,
`1/6` alone carrying `near_o 7 near_who 0`, and all three of who=1 holding
`ATTACKORDER ox 7 whom 0 uid 14`. **What would have falsified it**: any of
those six coordinates moving, or `1/6`'s order naming another target.

### What it says — §18's tie is not a tie

`report.py rontrace.log calls 615`, the whole bracket:

```
attack_dist  this=0x1575d7c4  o=8 who=0 x=1368 y=7992 = 288
  attack_dist  this=0x1575d7c4  o=8 who=0 x=1368 y=7992 = 288
compare_target  this=0x1575d7c4  o=8 who=0 in_range=1 ai=1 =  2155
attack_dist  o=7 ... = 198   (nested: 198)   compare_target o=7 in_range=1 ai=1 = 10771
attack_dist  o=6 ... = 339   (nested: 339)   compare_target o=6 in_range=1 ai=1 =  2155
find_nearby_target  max_dist=4608  who_out=…  add_order=1  cavarch=0  flags=0 = 7
```

Five things, and four of them were open questions:

- **The candidate order is the cell's `down` chain**, `0/8`, `0/7`, `0/6` —
  §18's reading of the dump's `up`/`down`, confirmed from the other side.
- **`attack_dist` is 288, 198, 339**, exactly §13.1's formula at
  `block_radius + 0x18 = 72`. The ~84 per-side extent §18 wrote down as a
  falsifier is **refused**: the original's own numbers are the documented
  ones.
- **`max_dist` is 4608** — `unit_respond_range` (12) `× 0x180`, the floor item
  384 read off `find_melee_target` for a melee AGGRESSIVE unit carrying
  `unit_masks & 0x40000`. The radius arm is now diff-backed.
- **`in_range` is 1 for all three**, so §12.2's range gate takes the
  "deemed in range without testing" arm, as the decompile's `local_24 == 0 &&
  local_2c == 0` says for a non-guarding unit that is not STAND_GROUND.
- **And the values are not equal.** `0/7` scores 10771 where the two the
  attacker cannot reach score 2155 — a factor of 4.998. Each candidate's
  *second*, nested `attack_dist` is what makes it: `compare_target` calls
  `is_in_range` **itself** at `0064f1ed` and divides by five when its own test
  fails. §12.3 has carried that `/5` since the second reading; what nobody had
  read is that the `in_range` argument is a **permission to test**, not the
  verdict.

`ai=1` is who=1's computer bit, which `!ai off` does not clear (run104's note
above), so the original's absolute numbers run through §12.3's AI branches
(`v /= dmg`) and this crate's do not. The **factor of five** is what the
comparison rests on.

`compare_target` and `find_nearby_target` are entered once each in the window
and `attack_dist` 8 times, so the whole answer is 35 lines.

## The launch point (run109, 2026-09-19) — where an arrow actually starts

Item 396's capture, and the one `docs/COMBAT.md` §20.3 named. **Numbered 109
because 108 was already taken** by item 386's call-proxy capture above, which
runs through `tools/explore/golden_capture.sh` and so leaves no stanza in
`captures.txt` for the next booking to trip over; nothing was overwritten —
that lane writes under `~/ron-golden/`, this one into the bottle's `Logs`.

**What it is.** `AMMO=5` under `[End Frame]` over `[9420, 9480)` on run100's
Great Lakes game, everything else run100's detail exactly, so the overlap check
needs one `--exclude` and no `--drop`. **60 blocks, 9420..9479, no gap; 39 MB;
about three minutes.** Every check passed: identity against run53 (9,496
identical, 0 differing), `MAP_STYLE 14`, the window whole, the `AMMO` block
counts on their predicted boundaries, 183 `sx` lines, and the overlap against
run100 at **60 blocks, 0 differing**.

**The booking was wrong about the disk, and the grep that found it took a
minute.** The block is named `AMMO` — string-table index 127, `0x9ec / 20`
against `Data/internal_strings.xml` — not `AMMODATA`, which is what the earlier
sweep looked for. Under the right name **run17 already had 173 ammo records**
and run29 one. run17 is no substitute for this capture — its shooters are
Slingers and its `GUY` detail is 1, so nothing names the animation — but it
confirmed §20.3's rotate-by-facing model before this run cost anything: 24
shots collapsing onto three model-space vectors, each held to ±2.

**The predictions held, including the one that could fail.** The stanza said
the original's `total_time` on the four launches below the word would be
**27, 26, 26, 26** against this crate's 27, 27, 26, 27; it is. It said the
`AMMO` counts over 9420..9451 would step 0, 1, 2, 3, 4 on named boundaries;
they do. It said `|offset|` would land in [90, 130] on the Slinger transfer —
**it does not**: the Longbowman's radii are 64..82, below the Slinger's
96..125, so that hypothesis was wrong in magnitude while right in shape. The
bands §20.3 derived from the flight times held exactly, which is the part that
had to.

`docs/COMBAT.md` §22 is the measurement and the table; **the Great Lakes word
moved 9451 → 9510**.

## run110 — chapter one's group pool, and the emergency that never fired (2026-09-19, item 399)

**What it is.** Chapter one again, on run101's setup exactly — map 14, seed
12345, `tools/gamelog/golden/chapter1.cmd` — with a **`GROUPS`** window over
`[610, 630)`: 20 blocks, 2.5 MB, **42 seconds** end to end, `cover=0`. It
exists for two fields of one record: `GroupData::army` (`+0x8`) and
`order_num` (`+0x2c`).

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/g6 \
    --map 14 --end-frame 640 --log-window 610 630 \
    --detail end:MISC=9,UNITS=9,GROUPS=9,GUYS=9,LEADERS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file <abs path to>/tools/gamelog/golden/chapter1.cmd
```

`--cmd-file` must be **absolute**: `golden_capture.sh` does `cd
tools/explore` before exec'ing the runner, and a relative path resolves
against that. The first attempt died on it in a second, which is the cheap
failure.

**The disk was grepped first and came back empty.** Every chapter-one
capture on disk (`~/ron-golden/{g0..g4,c1,h1,t1,probe1}`, run101–run108)
runs `UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=2,DEATHS=1` and carries **no
`GROUPDATA` at all** — g4's block 618 holds ANIMALDATA, ATTACKORDER,
BUILDDATA, BUILDQUEUE, CITIES, CITY, EXPLORETOORDER, FRAME, GATHERORDER,
GUY, LEADERDATA, MOVEORDER, OBJECT, PATHDATA, STACK, SUBOBJECT,
TARGETORDER, UNITDATA, UNITORDER, WALLDATA, WORLD and nothing else. The
five archives that do carry a pool are other games.

**`ARMY` is not an `[End Frame]` category, and the item's brief was wrong
about that.** There is no `ARMY` key in `gamelog.ini`;
`GameLog::dump_armies@0092fc50` has **no caller** in the export; and
`GameLog::full_dump@00930380`'s per-key list has no armies index —
`ArmiesData::log_data` is reached only from `dump_all@0092f2d0`, i.e.
`DUMP_ALL=1`, run25–27's ~70 MB-a-frame shape. So this run took the `GROUPS`
half, which is cheap, and it settled the question without the `ARMY` half
being booked at all.

### The `GUYS` trap, which is `ORACLE.md`'s `DEATHS` trap one category along

The first attempt (`~/ron-golden/g5`, same command but `GUYS=2`) came back
with **zero** `GROUPDATA` records and everything else present. `DEATHS` was
already off, so the documented trap was not it.

`GroupData::log_data@0045e1d0` sets no type of its own and its lines are
accepted against whatever the previous dumper left. The previous dumper is
`dump_units` (`full_dump` index `0xd`; `0xf` WALLS, `0x10` AMMO and `0x11`
DEATHS are all 0), and the **last** record it emits is a `GUY` nested inside
the gaia players' `ANIMALDATA`. `GuyData::log_data@005de6c0` opens with
`set_type(0x14, 0)` and then walks `set_detail(1)`, `(2)`, `(3)`, **`(4)`** —
and `set_detail` is called whether or not the line is accepted, so it leaves
`current_detail` at **4**. `check_accept@009309a0` then drops the pool on
`details[End Frame][GUYS] < current_detail`, which at `GUYS=2` is `2 < 4`.

**So the rule is not "DEATHS off": it is that the category the pool inherits
must be set at or above that dumper's highest `set_detail`.** run31's and
run92's `GUYS=9` satisfied it by accident. `GUYS=9` here, and the pool came
out: **10,240 records**, 512 slots × 20 frames.

### The three checks, all green

- **Same game**: `rngcmp.py ~/ron-golden/g4/…/rontrace.log
  ~/ron-golden/g6/…/rontrace.log` → **641 frames in common, 0 differing,
  641 identical**. Logging takes no draws, and this is what says so.
- **The window landed whole** (parked 373): **20 blocks, 610..629**, no gap,
  plus block 1 and the `!quit` block at 641 (`FRAMES + 1`, as run92's
  correction has it).
- **The pool is in them**: 10,240 `GROUPDATA`, against the `≤ 512` that is
  the trap firing.

### What it says

Two live groups in the whole game, both who=1, unchanged across every block
616..629:

| slot | `who` | `num` | `army` | `order_num` | `form` | `form_num` | `stamp` | members |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 64 | 1 | 3 | **0** | **0** | −1 | 0 | 615 | `1/6`, `1/7`, `1/8` |
| 65 | 1 | 1 | −1 | 2 | 0 | 1 | 0 | `1/0`, the explorer |

The hoplites **are** in who=1's army 0, and the army issues **no order** in
the window — `order_num` is stepped by every `Group::action_*`. With the
army's own cadence frames at 508 and 764 (`docs/ARMY.md` §5), that means no
tick reached it at all, which is what `docs/COMBAT.md` §23 then explains:
`Armies::emergency` is the city alarm's and a unit taking a hit never
reaches it. `docs/ARMY.md` §15.8 carries the corrected predicate; the golden
word moved **624 → 621** and the value diff with it, four blocks of six
units now exact.

## run111 — the make list at the purchase (2026-09-19, item 414)

**What it is.** `LEADERS=9` under `[End Frame]` over `[9375, 9391)` on
run100's Great Lakes game, everything else run100's detail exactly — which
is **run107's `end:` line verbatim**, so the overlap check needs one
`--exclude` and no `--drop` and run107 is a second sibling to read it
against. **16 blocks, 9375..9390, no gap; 41.5 MB; about three and a half
minutes** end to end at `cover=0`.

It exists for one record on one frame: the leader's own `MAKEOBJECT` list
across `create_units`' re-offer on sim-frame 9379 (block 9381) and
`make_stuff`'s purchase on 9382 (block 9383). §47 could not name the slot
either side bought, because a purchase draws nothing and `LEADERS=1` prints
`who`, `tribe`, `score` and `leader_flags` and no list at all.

**All five checks passed on the first attempt through the driver:** rngcmp
against run53 **9,406 frames in common, 0 differing**; `MAP_STYLE 14` and
`(int)seed 12345`; the window whole at 16 blocks 9375..9390 (parked 373's
check); **352 `MAKEOBJECT`** in the window — 11 slots × 2 leaders × 16
blocks exactly, which is the detail guard and would be 0 at `LEADERS=1`;
and `samegame.py --exclude LEADERDATA` against run100 at **16 in common, 0
differing**.

**The first launch refused, and the tool named its own remedy.** `Screen
Recording is off — screencapture wrote nothing`: the grant belongs to
`/Users/rf-studio/bin/RonDriver.app` and not to the Claude Code bundle,
which macOS never evaluates. `zsh tools/gamelog/viadriver.sh
tools/gamelog/runqueue.sh - 414` ran clean. No human was ever in the loop,
which is the distinction between this and a refusal that has to end a turn.

**And the driver could not read `captures.txt` at all before this run.** A
stray `=======` at line 3387, left by merge `9ae8070` with no `<<<<<<<` or
`>>>>>>>` beside it and no duplicated stanza, made `runqueue.sh` exit with
`unknown key '======='` — so **every stanza after run107's was unreachable**,
run109's included. One line deleted.

**The predictions, and the headline one was wrong.** The stanza predicted
the original's Scholar would *stay* at `9999999`/`num 5` where this crate's
collapsed. Both collapse, on the same block: the original to **5,755,741**
on two slots, this crate to **45,568** on one. The agreement at block 9375
(two 9,999,999 batches of five, both sides) held as predicted, as did the
Citizen agreeing outright, the goods agreeing entering the frame, and
`scholars 5` throughout. `docs/AI.md` §48 is the measurement.

## run112 — chapter two, the ranged line and the ammunition (2026-09-19, item 415)

The golden record's second chapter (`docs/GOLDEN.md` §6), staged from
`tools/gamelog/golden/chapter2.cmd`: `!ai off` at 0, the Gunpowder age for
both players by `library` at 600 and 602, then a Bowmen squad at tile (4,40),
a Hoplite squad at (12,40) eight tiles away, and a Slinger squad at (4,43).
Chapter one's squads are born in contact and never shoot; this is the chapter
that makes the shooting half of `Unit::fight@005fd4d0` run first.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch2 \
    --map 14 --end-frame 900 --log-window 605 900 \
    --detail end:UNITS=3,GUYS=2,AMMO=5,DEATHS=1,LEADERS=2 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter2.cmd
```

`success: true`, exit 0, 901 frames, `MAP_STYLE 14` and seed 12345 read back,
five settings files restored, 144 s launch to exit, 34.7 MB of dump and 10 MB
of trace.

### The predictions, written into the `.cmd` file before the run

A golden chapter takes no `captures.txt` stanza — run101–run105 have none,
because this lane is `viadriver.sh golden_capture.sh` and not `runqueue.sh` —
so the stanza's `check:` lines live in the chapter file's header. All four
held, and they were checked before anything was read for interest:

| check | predicted | observed |
| --- | --- | --- |
| every staged line runs | eight `INFO cmd`, each returning 1 | eight, each 1 — the six staged plus `37 !ffwd 1` and `900 !quit` |
| the window is the window asked for | 605..899, no gap, plus the `!quit` block | **296 blocks**, 605..899 contiguous, 901 the quit block |
| the spawns | three figures per `add`, at 611, 616, 621 | 52 → 55 → 58 → 61 units on exactly those frames |
| the chapter's own falsifier | `AMMO` blocks after 610 | **373 blocks over 186 frames**, first at 645 |

The falsifier is the row that matters: *no* `AMMO` block would have said the
ranged arm was never entered and the bowmen had closed to contact like
hoplites. `AMMO` is off in every capture on this disk but run17 (173 blocks),
run109 (183) and run29 (1) — which is the grep that was run before the
capture was staged, and which also established that nothing new had to be
written to read the record: `crate::diff::ammo` already parses all
twenty-seven fields from item 402's widening of run109.

The other two falsifiers `docs/GOLDEN.md` §6 names did not fire either. No
`ATTACKORDER` on the Bowmen at 611, before the hoplites exist. And the first
arrows are the Bowmen `0/6` and `0/7` on `1/8` at 4.95 and 4.04 tiles —
inside a ten-tile reach, not beyond it.

### The run was taken twice, and the first take is why the guard exists

The first attempt gave `--detail end:` and `--detail misc:` and **no
`--detail start:`**, copying run101/run103's line rather than run105's. The
capture succeeded — 901 frames, every command run, the same 373 `AMMO`
blocks — and its `[Start Game]` block held **0 `LEADERDATA` and 0 `UNITDATA`
where run105 holds 4 and 52**. `borrow_from_siblings` lends a capture the map
it could not print for itself, so the harness stood *something* up, walked it,
and reported a golden word of **0**: 80 draws at frame 0 against the trace's
120, which is the setup's number and not the simulation's. It looks exactly
like a real word. The same shape as `crate::diff::endpoint`'s run28 note
(12 units of 71 in a region-less world) and as item 364's borrowed frame
stream.

`crate::diff::golden`'s `walk_chapter` now refuses a capture whose start
block carries no leaders and no units, and the refusal prints run105's
`start:` line as the remedy. Every chapter file's recipe carries that line
now; chapter one's says it is not optional.

A second, unintended benefit: the two launches are one game. Both takes give
873 identical numbers between them — the same eight commands, the same 296
blocks, the same 373 `AMMO` records over the same 186 frames beginning at
645, the same unit counts on the same frames — which is `docs/DECISIONS.md`
41 §2's "regenerate rather than store" measured again on a second script.

### What the walk says: the word is 616

`rngcmp.py` against g4: 901 frames in common, identical to **616** and
differing from 617 — which is chapter two's script diverging from chapter
one's at its first differing spawn, exactly where it should.

Staged into the harness, all six lines run, nine units, nothing carried and
not acted on. **The draw stream parts at 616** — ours 26 draws, theirs 25 —
and the extra one is `Unit::fight+0x9b0`, the one-in-five re-search of
`docs/COMBAT.md` §8.2 step 0, spent on the frame the hoplite squad appears.
The sequence parts on the same frame and the value word at 617.

**The value diff beside it, and it is not what the draw stream alone
suggested.** The dump holds all six combatants stock still on their birth
cells at 615–618 and the first arrow is not until 645, so nothing *moves*
across the parting. But the second `add` does not land where the original
lands it: `add hoplite who=1 12,40` asks for tile 12, internal 2400, the
original seats the squad at `(2424, 7800)`, `(2568, 7800)`, `(2472, 7944)`
— a clean `+24` on the captain — and this crate seats it at `(2284, …)`,
`(2428, …)`, `(2332, …)`, **uniformly 140 units west**. The Bowmen, the
chapter's first `add`, are exact.

That is `find_nearby_spot`'s ring on **clear ground**, which chapter one
could not test: chapter one's second `add` asks for a point one tile from
the first squad, so item 379 fitted the ring where the near ground was
already taken. Here the nearest other unit is seven tiles away and the
asked-for spot is empty. It is pinned rather than fixed — the function is on
every production path and item 379 said a change to it belongs in an item
that re-runs those pins — and it gives 616 a cause to test rather than a
mechanism to guess: ours stand 1252 units from the bowmen where the
original's stand 1392, 6.5 tiles against 7.25, so a re-search predicate
keyed on range is the first thing to look at.

## run114 — the AI's own dump, the offers read forwards (2026-09-21, the seventh pass)

**What it is.** run111's window again — `LEADERS=9` under `[End Frame]`
over `[9375, 9391)` on run100's Great Lakes game, run111's detail exactly —
with the tracer built `TRACER_DEFS=-DRON_LEADER_PROBE` and
`callwin=9376-9386`, so `Leader::create_units`, `MakeList::make_me` and
`Leader::make_this` are proxied over the make frames (`docs/AI.md` §52).
The first capture with the leader probe, and the item parked as 367 since
the fifth pass. 16 blocks, 9375..9390; through `viadriver.sh runqueue.sh`,
about four minutes; the plain tracer rebuilt into the install afterwards.

**Six checks, five passing and the sixth mine.** rngcmp against run53
**9,406 frames identical, 0 differing**; `MAP_STYLE 14`; the window whole
at 16 blocks; `samegame.py` against run111 **with `LEADERDATA` included**
at 16 in common and 0 differing — the proxies perturb nothing; and the
stanza's own falsifier, **four `make_me` on 9380 carrying exactly the four
values item 432 reconstructed** (869,565 · 4,891,136 · 234,782 ·
5,755,741). The check that failed counted proxied sites and expected 3;
the log says 11, because the eight base sites are always installed. The
stanza now says 11.

**The prediction that died was the order.** The stanza wrote Merchant
then Scholar for city 0, as item 432 had; the proxy prints Scholar then
Merchant. Both replay to block 9381 (`docs/AI.md` §52.3), so the record
never held the order and the reconstruction's clause was unsupported.

**And what nobody asked for.** The window holds the AI's other offers —
two building offers on 9378, two `cat 7` on 9379, eleven on 9381 — and
the purchase `make_this(slot 1)` on 9382 answering 0. All on the disk,
compared to nothing yet.

## run115 — the met bit's own frame, and the bracket closes on it (2026-09-21, item 390)

**What it is.** run53's game, a `LEADERS=9` window over `[7880, 8010)` at
**run94's detail exactly but for that one category**, `!quit` at 8025,
`cover=0`, 254,494,537 bytes, **eleven minutes** launch to archive
(13:04:54 to 13:15:34). `docs/VISION.md` §6.3 is what it decided.

**Why it was owed, and the grep moved a claim on the way.**
`docs/VISION.md` §6.2 and `docs/AI.md` §45 both describe the four windows
that bracket first contact as `LEADERS≥2`. They are not:
`LeaderData::log_data@006e5110` raises the detail to **3** at line 213,
immediately before the eight-iteration `diplos`/`treaties` loop, so the
pair needs `LEADERS≥3` — and all four of those windows are in fact
`LEADERS=9` (run19 is a `DUMP_ALL`). The sentence is harmless to their
conclusions and is exactly the sentence that decides whether a capture is
owed, so it was worth getting right before booking one.

**And the disk was grepped, twice.** run94 covers **`[7754, 8045)`** — the
whole interesting part of the gap — and at first grep appears to carry
`treaties`. It does not. run94 is `LEADERS=1`: of its 1,172 `LEADERDATA`
records, **32** hold a `treaties[scan]` line and all 32 are in the four
**start-dump** records, whose `start:` detail is run10's `LEADERS=9`. Its
1,168 per-frame records hold none. Every Great Lakes stanza with per-frame
`LEADERS≥3` is run80 `[23960, 24000)`, run84 `[6950, 7030)`, run91
`[7514, 7600)`, run107 `[9170, 9200)` and run111/run114 `[9375, 9391)` —
§45's gap, confirmed against `captures.txt` rather than against the
documents.

That same reading is what made run94 the right overlap partner: same game,
same detail but for the one raised category, so the check below compares
**all 130 blocks** rather than six at each end.

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run53.log` | **0 differing**, 8,026 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 14, seed 12345 |
| the window, block for block | **130 blocks, 7880..8009, no gap** |
| the raised category is in them | **4,160** `treaties[scan]` (8 × 4 leaders × 130); `LEADERS=1` gives 0 |
| `samegame.py --exclude LEADERDATA` vs run94 | **130 in common (7880..8009), 0 differing** |

All five first time. The fourth is the detail guard in its sharpest form
here: it is the *whole* difference between this capture and the one
already on disk, so a correctly-numbered window that came back without it
would have been run94 again at four times the price.

**What it says.** The original's met bit arrives on block **7945** and so
does this crate's — the same block, and on both leaders at once
(`1/treaties[0]` and `0/treaties[1]` alike), which is
`Leader::treaty_on@006e1190` oring into both slots as `docs/VISION.md`
§6.2 read it. The bracket was **(7616, 8174]**, 558 frames wide; it is a
block. `diplos` does not move on any of the 130 blocks of either leader —
2 on each leader's own slot, 0 on the cross slot — so the two are at war
from before the window and only *contact* happens inside it, and the bit
does not fall again through 8009.

**The comparison is `run115_s_window_is_the_met_bit_s_own_frame`**: 260
blocks, 272,480 field-frames, the flip triple pinned at `(7945, 7945,
7945)`, `diplos` asserted not to move, and 89 fields of residue pinned by
name. Made to fail on purpose by commenting out the `Sim::meet` call in
`Sim::check_ever_seen`'s tail — item 385's whole change — which reads
`(None, 7945, 7945)`.

**What it also opened.** `0/wars`, `0/active_wars` and
`0/active_wars_with` agree at nought for 121 blocks and part on **8001**,
where the original writes the **human** leader a war census — `wars 1`,
`active_wars 1`, `active_wars_with 2` — beside a single `production_step`
tick that falls back to 0 on 8002. Fifty-six blocks after first contact,
and the first direct evidence on this disk that the original runs its
census for a human at all; this crate leaves the human's census at zero
forever. `docs/AI.md` §43's `human` skip is what that belongs to.

The window also carries two more `LEADERDATA` records a block — leader
slots **8 and 9**, zero throughout — which is why the record count is four
a block rather than two, and why the `treaties[scan]` tooth above expects
4,160 rather than 2,080.

## run116 — the collision sweep, read from inside (2026-09-21, item 456)

**What it is.** run100's own game and detail — `MISC,UNITS=3,BUILDS=7,
CITIES=5,GUYS=4,DEATHS=1,LEADERS=1` — over `[10155, 10170)`, fifteen
blocks, with the tracer built `TRACER_DEFS=-DRON_COLLIDE_PROBE` and
`callwin=10158-10163`. The **first capture with the collide probe**
(`docs/COLLISION.md` §9): five proxies over
`Unit::detect_unit_collision@00617060`,
`CollCheck::collide_here@00682540`,
`UnitData::will_be_corner@00609fa0`, `UnitData::is_here@0060a0c0` and
`UnitData::is_corner@0060a040`, plus an INFO 15 identity record that
names each `UnitData *` in `(who, o)`. Through `viadriver.sh
runqueue.sh`, about three minutes; the plain tracer rebuilt into the
install afterwards.

**Half of it was already on the disk, and the stanza says so.** run100
covers `[9340, 10899)` of this very game at this exact detail, so the
word's whole *state* was on disk and item 448 had read it whole. What
no `gamelog.ini` line can put there is the **inside of the sweep**:
`collide_o` and `collide_who` are cleared by §5.4's snap arm two
instructions after the probe returns. That, and only that, is what this
run adds.

**Six checks, all green first time.**

| check | result |
| --- | --- |
| `rngcmp.py` vs `rontrace-run53.log` | **0 differing**, 10,186 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 14 |
| the window, block for block | **15 blocks, 10155..10169, no gap** |
| `samegame.py` vs run100, no `--exclude` | **15 in common (10155..10169), 0 differing** |
| proxied sites from the log's own `PROXIED` records | **13** (the eight base plus five) |
| the stanza's falsifier | **one `detect_unit_collision` on `1/38` on 10161, returning 1** |

The fourth is the perturbation check in its strongest available form —
the window lies *inside* run100's at the same detail, so nothing was
raised and the whole record is comparable rather than six blocks at
each end.

**What it says.** `1/38`'s refused step is §4.2's fast-path edge sweep
finding `(892, 471)`, `1/31` covering it, the exemption ladder
declining, and `will_be_corner 5` against `is_corner 0` — hard.
`docs/COLLISION.md` §9.2 has the eight rows beside this crate's, and
six of the eight already agreed.

**And a second record answered the question the first one raised.**
`PathFinder::astar_path@00683770` is one of the eight *base* proxies, so
it is in every log: on 10161 two searches return **−1** and resume on
10162. Their `stack` argument is `UnitData +0xb8`, so subtracting the
offset and reading the INFO 15 table names them — `1/31` and `1/32`.
That is `+0x104`, `openlist`, and it is the clause that refused the
step. The capture booked for one question answered it from a proxy
nobody added for it.

**What it moved.** Great Lakes' long word **10,161 → 10,232**.

## run118 — chapter two's guy clocks, at `GUYS=4` (2026-09-22, item 496)

**What it is.** run112's capture again — same lobby, same seed, same
`chapter2.cmd`, same window — with one field of the detail line changed:

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch2g4 \
    --map 14 --end-frame 900 --log-window 605 900 \
    --detail end:UNITS=3,GUYS=4,AMMO=5,DEATHS=1,LEADERS=2 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter2.cmd
```

**Why, and the grep that came first.** `GuyData::log_data@005de6c0`
announces four detail levels; `cur_time`, `end_time`, `last_time`,
`cur_anim`, `variation`, `hold_attack`, `queued_attack` and `guy_flags`
are all past the third, so at run112's `GUYS=2` a `GUY` block stops
after `ox` and the animation clock is not in the file. Item 496's whole
question is whether one bowman's clock stepped on frame 695. No capture
on this disk carries chapter two's guys at `GUYS=4` — run44 and run53
do, on other games with no bowmen — so the question could not be
answered from what was there.

**`success: false`, and the reason is the detail.** The runner's launch
timeout is 180 s and `GUYS=4` costs about four times `GUYS=2` a block:
**243 blocks of the window's 296**, 605..846, contiguous, 64 MB, and the
file stops mid-record inside 847. Everything item 496 reads is inside
it and nothing it reads is past 700, so the run answered what it was
booked for; a session that needs the rest of the window must raise the
timeout rather than re-run this line.

**And the 53 missing blocks are a truncation, not an absence.** The file
ends because the game was killed, so **nothing past 846 may be read as
evidence of anything** — not a record that stops appearing, not a unit
that stops being printed, not a death that never arrives. This repo has
paid for that shape three times (`crate::diff::endpoint`'s run28 note,
item 364's borrowed frame stream, run112's own first take, each of which
produced a number that looked exactly like a real one), and a partial
capture whose partiality is not written down is the cheapest way to buy
it a fourth. Every claim above is inside 605–846 and none reads past 700.

**What it says.** Block 696 — the state after frame 695, chapter two's
word:

```
  0/6  cur_anim 12  cur_time 29  end_time 30  last_time 29    ← last == cur
  0/7  cur_anim 0   cur_time 0   end_time 31  last_time -1  hold_attack 1
  0/8  cur_anim 0   cur_time 0   end_time 31  last_time -1  hold_attack 1
```

`last_time` is `cur_time` before this frame's step, so their difference
is the step the original took. Every other block of the window has
`last = cur − 1` on all three bowmen. On 695 `0/6` did not step, which is
`Guy::inc_time`'s zero arm under `unit_masks2 & 0x10` — and run112's own
`OBJECT` block, which prints that field at every detail level, carries
`unit_masks2 16` on `0/6` at exactly that block. `docs/COMBAT.md` §43.

**And it is the first capture that can price the level.** 45 MB at
`GUYS=2` against 64 MB for four fifths of the window at `GUYS=4` — about
2.4× the bytes a block, and enough slower to miss the timeout. A golden
chapter kept at `GUYS=4` for the whole window is a decision, not a free
upgrade.

## run119 — booked, and **not taken** (2026-09-22, item 502)

**What it would have been.** run118's line with `--timeout` raised from
its 180 s default, to land chapter two's whole 296-block window at
`GUYS=4` instead of the 243 blocks run118 got:

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch2g4full \
    --map 14 --end-frame 900 --log-window 605 900 --timeout 600 \
    --detail end:UNITS=3,GUYS=4,AMMO=5,DEATHS=1,LEADERS=2 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter2.cmd
```

It is recorded because the number was reserved and spent, and because a
capture not taken is only a result if the reason is on file. The line
above is tested — it launched, built, reached the lobby and was stopped
by hand 35 seconds in with `settings_restored: true` — so a session that
does want the 847–900 blocks can run it as written.

**Why it was dropped.** Item 502 was booked with this capture as part of
it, to settle §43.5's first open question: whether the captain's search
is what costs it the frame, which needs a window in which a *follower*
does the searching. Three things retired it before it was taken, and all
three came off the disk:

- The question was the wrong one. The followers do not search and do not
  lose a frame for a reason that has nothing to do with what a search
  costs: they take their captain's target **in place** at `Unit::fight`'s
  head, before the validity test (`docs/COMBAT.md` §44.2).
- Its falsifier was already in run112. An `add_attack_order` leaves
  `new_ord 1 ever_in_range 0` and an in-place retarget touches neither,
  and block 696 carries both cases side by side.
- The new word at **725** is inside run118's 605–846, so the landing's
  own successor is readable in the capture that exists.

What run119 would still buy is blocks 847–900 at `GUYS=4` and nothing
else; no open item reads them. Booked and dropped, rather than deferred
in silence.

**`--timeout` is the flag**, for the next session that needs it:
`unattended_capture.py --timeout <seconds>`, default 180, and it bounds
the *game*, not the build. run118 wrote 243 blocks in 180 s, so the whole
window wants about 600.
## run117 — the AI's ledger across two market rotations (2026-09-22, item 506)

**What it is.** run53's game and run53's `[End Frame]` detail — bare
`MISC` — with `LEADERS` raised from nothing to **9** over `[10375,
10620)`: 245 blocks, `!quit` at 10640, `cover=0`, 366,030,105 bytes,
**thirteen minutes** launch to archive. `docs/ECONOMY.md` §13 is what it
decided.

**Why it was owed, and the disk was grepped twice first.** Great Lakes'
draw sequence parted at **10582** on a market frame and nothing anywhere
on this disk carried the AI's goods or its make list within 1,400 frames
of it: the map's `LEADERS≥2` windows are run84 `[6950, 7030)`, run91
`[7514, 7600)`, run19 `[8174, 8192)`, run107 `[9170, 9200)` and run80
`[23960, 24000)` — a gap of 14,700 frames with the word inside it. The
two captures that cover 10582, run97 and run100, are both `LEADERS=1`.
And the widening ran **before** the booking, not after:
`run100_s_word_block_is_every_record_the_dump_carries` over 1,257 blocks
and 3.4M rows put **one** row on the word's block and nothing at all
between 10401 and the word, which is what said the cause had to be in a
record run100 does not print.

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run53.log` | **0 differing**, 10,641 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 14, seed 12345 |
| the window, block for block | **245 blocks, 10375..10619, no gap** |
| `LEADERDATA` records whole | **988** — 4 × 245 plus the start dump's 8 |
| the raised category is in them | `MAKEOBJECT` ×22 on block 10583 |

**The window straddles two rotations on purpose.** The AI's production
rotation runs every 200 frames and the draw streams agreed entry for
entry on 10381/10382/10383 and parted on 10582, so a window holding only
the parting could not have said whether the ledger was already off two
hundred frames earlier. It was: `bucket[2:wealth]` is 107 here against
110 at block 10375, and everything else agrees to the unit.

**What it says, and it is one frame nobody had named.** On sim-frame
**10576** — the rotation's `Setup` step, which spends **no draw at all**
— the original's `Leader::market_speculation` buys a hundred food for
**128** wealth: block 10575 `93 86 128 269 100 0` to block 10577
`194 86 0 269 100 0`. This crate's sell and buy passes were a declared
seam, so it bought nothing and entered `make_stuff` five frames later
with a purse the original does not have. The word, the sequence and the
count were all downstream of that. **10582 → 10817.**

**The stanza's three predictions were all true on the word's block and
none of them was the cause**, which is the entry worth keeping: (a) the
wealth is short — it is **0**, not "under 120"; (b) the timber is short —
refused, 87 on both; (c) the make list differs — it does, because the
purse does. A frame that agrees with every branch of a hypothesis is a
frame whose cause is upstream of all of them.

**And one check was written on a false premise.** The stanza asked
`samegame.py` for the overlap against run53 with `--exclude LEADERDATA`,
on the belief that run53 is a per-frame sibling. It is not: `samegame.py`
digests a block's **indented** lines and run53's bare `MISC` writes none,
so the file holds **one** frame block in 24,000 and the comparison
answered `frames: 1 and 245, 0 in common`. The check has been replaced in
`captures.txt` with a record count that can fail — a `MISC`-only capture
cannot be overlap-checked against anything, and `rngcmp`'s 10,641 frames
at 0 differing is the stronger same-game instrument anyway.

## run120 — the cap's writer, by per-frame coverage (2026-09-22, item 518)

**What it is.** run100's Great Lakes game again at run100's own detail
(`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,DEATHS=1,LEADERS=1`) over the frame
window `[10236, 10247)`, with `rontrace.cfg` `cover=1` and `window=10236-10244`
— per-frame function sets, nothing raised. `!quit` at 10250. **About four
minutes, 16 MB of dump and 17 MB of trace**, launched through
`viadriver.sh` with no human at the menu.

**Every check passed**: the draw stream against run53, 10,251 frames and 0
differing; `MAP_STYLE 14`; eleven blocks 10236..10246; the overlap with
run100, 11 blocks and 0 differing; a coverage set on all nine window frames.

**Why it was owed.** Great Lakes' `1/28` walks 26 until the raid's ungroup
on frame 10240 and 25 from 10241, and no member of group 65 can report on
either side. No category prints a group record or a caller, and no trace on
disk had a coverage window above 5,800 on this map.

**What it settled.** On frames 10240 and 10241 no `Group::normalize`,
`push_group`, `equals_group`, `Group::kill`, `Group::add` or
`kill_group_move` runs, and `Groups::process@006fa210` runs on every frame
— one pool slot per player a frame, slot `f mod 64`, and 10241's is
`group 65`. The stanza's R1–R4 died and R5's family was right.
`docs/GROUPS.md` §19; the Great Lakes word moved 10834 → **11185**.

## run123 — Great Lakes' market word, and the list slot under it (2026-09-22, item 520)

**What it is.** run100's Great Lakes game at run100's detail with
`LEADERS` raised to **9**: `MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,
DEATHS=1,LEADERS=9` over `[10760, 11460)`, plus `rontrace.cfg` `cover=1`
and `window=11176-11190`, the production rotation that parts. `!quit`
at 11470. **1,375,741,619 bytes of dump and 17 MB of trace, about 45
minutes** from launch at 17:26 to archive at 18:11. It was launched
through `viadriver.sh` with no human at the menu, and the dump ran at
~15 blocks, or ~28 MB, a minute: 1.95 MB a block net of the start dump,
close to the 2.1 sized.

**Why it was owed.** Great Lakes' word moved to 11185 on item 518, and
the highest block any dump reached was run100's 10899, at `LEADERS=1`,
which prints no goods. The map's last goods bucket or make list was
run117's block 10619. The lab's twelve map-14 captures all stop at 1401.

**Every check passed:**

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run53.log` | **0 differing**, 11,471 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 14 |
| the window, block for block | **700 blocks, 10760..11459, no gap** |
| overlap with run100, `--exclude LEADERDATA` | **140 in common (10760..10899), 0 differ** |
| the raised category is there | `MAKEOBJECT` ×22 on 10760, 11185 and 11459 |
| the coverage window | a set on all 15 frames 11176..11190 |

**What it settled.** The market frame's input is slot 1 of the AI's make
list. It is an emptied Merchant slot in the original and a Cataphract
here, so this crate's `use_market` counts wealth as short and spends a
third sell draw. The purse, the commerce level and the stock, the other
three readings the stanza named, all died on the word's own block.
`docs/ECONOMY.md` §14; `run123_s_word_frame_is_widened_whole`.


## run125 — Great Lakes' word 11531, the bowman's arrival (2026-09-22, item 533)

**What it is.** run123's Great Lakes game at run123's detail,
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,DEATHS=1,LEADERS=9`, over
`[11440, 11600)`, plus `rontrace.cfg` `cover=1` and
`window=11524-11536` over the arrival. `!quit` at 11610. **325,330,650
bytes of dump and 17.5 MB of trace, about 14 minutes** from launch at
19:14 to archive at 19:28. It was launched through `viadriver.sh` with no
human at the menu. The dump ran at ~18 blocks a minute, 2.0 MB a block,
against the 1.95 sized from run123.

**Why it was owed.** Item 327 moved Great Lakes' word to 11531, and
run123, the highest dump at any detail below run80's endpoint window,
ends on block 11459.

**Every check passed:**

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run53.log` | **0 differing**, 11,611 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 14 |
| the window, block for block | **160 blocks, 11440..11599, no gap** |
| overlap with run123, no `--exclude` | **20 in common (11440..11459), 0 differ** |
| the coverage window | a set on all 13 frames 11524..11536 |

**What it settled.** The word is army 1's bowman `1/34` reaching its
attack point on block 11531 here and 11533 in the original, on a lag of
one and a half steps. The lag was set between 11446 and 11462 by a
formation hop pushed a frame apart. `docs/GROUPS.md` §20;
`run125_s_word_frame_is_widened_whole`.
## run127 — chapter five, the water: the first ship on this disk (2026-09-22, item 535)

The golden record's fifth chapter (`docs/GOLDEN.md` §9), staged from
`tools/gamelog/golden/chapter5.cmd`: `!ai off` at 0, then `add trireme
who=0 60,180` at 610, `add trireme who=1 64,186` at 615 and `add fisher
who=0 61,184` at 620, all three inside sea region 70. Run112's command with
the chapter file swapped:

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch5 \
    --map 14 --end-frame 900 --log-window 605 900 \
    --detail end:UNITS=3,GUYS=2,AMMO=5,DEATHS=1,LEADERS=2 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter5.cmd
```

`success: true`, exit 0, 901 frames, `MAP_STYLE 14` and seed 12345 read
back, five settings files restored, **146 s launch to exit, 43.0 MB of dump
and 9.9 MB of trace** (76 MB on disk with the staged install's links). One
take. No lane lock was held when it launched.

**The grep before it.** No capture on this disk carries a warship or a
fishing boat. The nearest thing is a sea-domain unit: East Indies' transport
barge `1/22` on run54/run86, which is born by an embarking land army
(`docs/TRANSPORT.md` §6) and never fights.

### The predictions, written into the `.cmd` file before the run

Committed as `189925c` before the launch. All held:

| check | predicted | observed |
| --- | --- | --- |
| every staged line runs | six `INFO cmd`, each returning 1 | six, each 1: the four staged plus `37 !ffwd 1` and `900 !quit` |
| the window is the window asked for | 297 frame blocks: 1, 605..899, 901 | 297 blocks, gaps only at 605 and 901 |
| the spawns | one `UNITDATA` per `add` (both types `UBER_SIZE 1`) | 52 → 53 → 54 → 55 on 611, 616, 621 exactly |

### §9's three falsifiers, and none fired

- **The `add` did not refuse.** `0/6` is born on 611, `1/6` on 616 and
  `0/7` (the Fishermen) on 621, so `find_nearby_spot` with no Dock placed
  a hull.
- **No hull stands on land.** `0/6` is at (11640, 34680), cell (15, 45);
  `1/6` at (12408, 35832), cell (16, 46); `0/7` at (11832, 35448), cell
  (15, 46). The start block's `WORLD` holds all three as `OCEAN`, region
  70, owner −1. Each is its asked tile's centre plus the same 24 on both
  axes, the "clean +24" chapter two's first squad showed.
- **The ships shoot.** 506 `AMMO` blocks over 278 frames: 249 from `0/6`
  on `1/6` and 257 from `1/6` on `0/6`. The two triremes stand 7.2 tiles
  apart, inside `RANGE 0-9`. who=1's trireme turns on `0/6` on block 617,
  with `whom 0 ox 6`, `in_range 1` and `recharging 40`, and its first round
  is in the air on 622. `0/6`'s first round is on 640. By 899 each
  trireme has taken ~115 of its 180 hits. The Fishermen are untouched and
  nothing dies: no `DEATH` block in the window. Neither trireme moves after
  it is seated. The fishing boat drifts about 130 units north-west and then
  stands.

So chapter five is not the Dock chapter. `docs/GOLDEN.md` §9 carries what
this means for the later naval chapters.

### The word is 621

The first walk parted at 617, on who=1's trireme turning broadside
(`docs/COMBAT.md` §49). Item 535 landed that rule and the word went to
621, the first round's launch. See `GOLDEN_WORD_CHAPTER_FIVE`.

## run130 — Great Lakes' word 11757, army 2's march (2026-09-22, item 554)

**What it is.** run125's Great Lakes game at run125's detail,
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,DEATHS=1,LEADERS=9`, over
`[11560, 11800)`, plus `rontrace.cfg` `cover=1` and
`window=11750-11762` over the word. `!quit` at 11810. **480,987,532
bytes of dump and 17.6 MB of trace, about 18 minutes** from launch at
21:27 to archive at 21:45. It was launched through `viadriver.sh` with no
human at the menu, once att-552's chapter-four capture had released the
driver. The dump ran at ~15 blocks a minute, 2.0 MB a block, as sized
from run125.

**Why it was owed.** Item 545 moved Great Lakes' word to 11757, and
run125, the highest dump at any detail below run80's endpoint window,
ends on block 11599.

**Every check passed:**

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run53.log` | **0 differing**, 11,811 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 14 |
| the window, block for block | **240 blocks, 11560..11799, no gap** |
| overlap with run125, no `--exclude` | **40 in common (11560..11599), 0 differ** |
| the coverage window | a set on all 13 frames 11750..11762 |

**What it settled.** The word is army 2's `1/62` stopping against `1/23`
on 11758 in the original and a frame later here. It walks a collision
detour planned on 11689 around `1/64`, which stands 21/20 behind the
original's. That lag opens on 11514, the block after army 2's group order
of frame 11512 leaves the original's `1/64` in no pool group.
`docs/GROUPS.md` §22; `run130_s_word_frame_is_widened_whole` and
`run130_s_word_is_1_64_s_lag`.

## run134 — Great Lakes' army pool, the first `GROUPDATA` above 7689 (2026-09-22, item 557)

**What it is.** run125's Great Lakes game at run125's detail with
`GROUPS=1` added and `DEATHS` taken off:
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1` over
`[11416, 11520)`, plus `rontrace.cfg` `cover=1` and `window=11420-11514`.
The tracer reads one window, and this one is 95 frames, inside run74's
102. `!quit` at 11530. **230,346,455 bytes of dump and 18.9 MB of trace,
eleven minutes** from launch at 22:12 to archive at 22:23. It was
launched through `viadriver.sh` with no human at the menu, and the lane
was free.

`DEATHS` is off because of run92's trap. `dump_deaths` ends on two
`WorldData::log_data` calls that leave the log's type at `WORLD`, which
is 0 here. `GroupData::log_data` sets no type, so with `DEATHS` on, the
pool is dropped silently. No `DEATHOBJ` block appears in run130, so what
`DEATHS=1` printed at this detail was the two `WORLD` blocks.

**Why it was owed.** Great Lakes' word 11757 was `1/64`'s lag after
army 2's group order of frame 11512 (`docs/GROUPS.md` §22). The dump at
`UNITS=3` prints each unit's back-pointer and nothing of the pool.
No trace on this map had a coverage window between 11190 and 11524.

**Every check passed:**

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run53.log` | **0 differing**, 11,531 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 14 |
| the window, block for block | **104 blocks, 11416..11519, no gap** |
| a `GROUPDATA` on every window block | **104** |
| overlap with run125, `--exclude WORLD --exclude GROUPDATA --drop last_group` | **80 in common (11440..11519), 0 differ** |
| the coverage window | a set on all 95 frames 11420..11514 |

**What it settled.** On 11424 the squad `1/62`–`1/64` joins army 2's
`{1/60, 1/61}` as `1/64` alone, because `Group::add`'s own `get_num`
normalizes the small group at every step. On 11512 `Group::sort`
kills that stray follower, which clears its pointer, and re-adds the
squad whole without writing one. `docs/GROUPS.md` §23;
`run134_s_pool_list_is_the_original_s`. The word moved 11757 → 11806.

## run135 — Great Lakes' word 11806, the crossing of armies 1 and 2 (2026-09-22, item 560)

**What it is.** run134's line (run125's detail with `GROUPS=1` added and
`DEATHS` taken off), `MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`,
over `[11760, 11860)`, plus `rontrace.cfg` `cover=1` and
`window=11796-11816` over the word. `!quit` at 11870. **221,771,936 bytes
of dump and 17.8 MB of trace, about eleven minutes** from launch at 23:02
to archive at 23:13. It was launched through `viadriver.sh` with no human
at the menu, and the lane was free. The dump ran at ~15 blocks a minute,
2.2 MB a block, as sized from run134.

**Why it was owed.** Item 557 moved Great Lakes' word to 11806, and
run130, the highest dump at any detail below run80's endpoint window,
ends on block 11799.

**Every check passed:**

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run53.log` | **0 differing**, 11,871 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 14 |
| the window, block for block | **100 blocks, 11760..11859, no gap** |
| a `GROUPDATA` on every window block | **100** |
| overlap with run130, `--exclude WORLD --exclude GROUPDATA --drop last_group` | **40 in common (11760..11799), 0 differ** |
| the coverage window | a set on all 21 frames 11796..11816 |

**What it settled.** The word is army 1's `1/37`. The original's waits
on `1/64` from 11803 to 11809, a hard collision every frame. This
crate's carries the soft one-shot (`unit_masks & 0x100000`) out of frame
11804's sweep, which also found `1/64` hard, spends it on 11805 to step,
and stops on 11806: the extra `move_step+0x823` draw.
`docs/COLLISION.md` §12; `run135_s_word_frame_is_widened_whole`.

## run136 — Great Lakes' word 11903, `1/62`'s detour (2026-09-22, item 563)

**What it is.** run135's line, unchanged,
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, over
`[11840, 11960)`, plus `rontrace.cfg` `cover=1` and `window=11896-11910`
over the word. `!quit` at 11970. **264,051,273 bytes of dump and 17.8 MB
of trace, about twelve and a half minutes** from launch at 23:42 to
archive at 23:55. It was launched through `viadriver.sh` with no human at
the menu, after a background wait for att-552's chapter-four capture to
free the lane. It ran at ~15 blocks a minute and 2.2 MB a block, as sized
from run135. The leader detail stayed at 9, so the overlap check is a
plain `samegame` with nothing excluded.

**Why it was owed.** Item 560 moved Great Lakes' word to 11903, and
run135, the highest dump at any detail below run80's endpoint window,
ends on block 11859.

**Every check passed:**

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run53.log` | **0 differing**, 11,971 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 14 |
| the window, block for block | **120 blocks, 11840..11959, no gap** |
| a `GROUPDATA` on every window block | **120** |
| overlap with run135, nothing excluded | **20 in common (11840..11859), 0 differ** |
| the coverage window | a set on all 15 frames 11896..11910 |

**What it settled.** The word is `1/62`'s flag-2 detour around `1/27`,
planned on frame 11901 (block 11902) from the same point on both sides:
the original's goes north and ours goes south. The original's `1/64` then
waits hard on `1/62`, while ours steps and stops. The original's frame
11901 runs `PathFinder::find_upath` → `astar_path`.
`docs/PATHFINDER.md` §23; `run136_s_word_frame_is_widened_whole`.
## run132 — chapter four, the border: three levers, cell for cell (2026-09-23, item 552)

The golden record's fourth chapter (`docs/GOLDEN.md` §8), staged from
`tools/gamelog/golden/chapter4.cmd`: `!ai off` at 0, `add temple who=0
28,160` at 300, `tech who=0 religion on` at 400, `civic who=0 3` at 500,
`tech who=0 allegiance on` at 550, then the squad, the scout and the wagon
at 600, 605 and 1100. run127's command with the chapter file swapped and
the border's detail:

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch4b \
    --map 14 --end-frame 1500 --log-window 295 545 --timeout 3600 \
    --detail end:WORLD=6,BUILDS=7,CITIES=5,MISC=1 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter4.cmd
```

`success: true`, exit 0, 1501 frames, `MAP_STYLE 14` and seed 12345 read
back, five settings files restored. **2178 s from launch to exit, 830 MB
of dump and 11.9 MB of trace** (836 MB on disk). `WORLD` costs about
3.3 MB and **8.7 s a block**; the other 1250 frames run fast.

**The window is `[295, 545)`, not §8's `[295, 345)`**, because the narrow
one sees the Temple and neither other lever. `LeaderData::territory` is
printed only from `LEADERS=8`, so the cell count in `WORLD` is the only
cheap reading of a border.

**It took three launches.** The first, at `--timeout 1500`, was slow, not
blocked: it dumped 295..467 without a gap (577 MB) and the runner gave up
at 1500 s. A timeout writes no receipt, and a wait on the receipt alone
sat idle for 1.5 h; wait on the game's exit. That take is kept at
`~/ron-golden/ch4b-truncated`. The second stalled before its first frame:
0.2% CPU, a 544-byte trace, `wine.log` ending at MoltenVK's instance line.
Nothing was on the screen, and it was killed and the prefix cleared. The
third lost the lane lock by two seconds to att-563's run136 and was
relaunched when that capture exited.

**The grep before it.** No capture on this disk carries a Temple (run80:
none by 23,999 on Great Lakes) or a staged civic level. run16 is the only
one with attrition ticks, and it has no border lever.

### The predictions, written into the `.cmd` file before the run

Committed as `c4ba076`. All held:

| check | predicted | observed |
| --- | --- | --- |
| every staged line runs | ten `INFO cmd`, each returning 1 | ten, each 1 (`cmdsran.py`) |
| the window | 252 blocks: 1, 295..544, 1501 | 252, gaps only at 295 and 1501 |
| the Temple | `orig_type 437`, `who 0`, on cell (7,40), finished | block 301, (5376, 30720), `(int)construct_hits 1200` = `myhits 1200`, `frame_started 300` |
| Napata's temple bit | `city_flags` 18449 → 18577 on 301 | exactly; London's 16401 does not move |
| owner-0 cells | 266, then more after each lever | 266 → 296 (306–310) → 327 (406–411) → 445 (505–511) |
| owner-1 cells | 261 throughout | 261 on every block |

### §8's border falsifiers, and none fired

- **The Temple is a finished building, not a construction site.**
  `run_cmd` calls `Build::activate` straight after `init_build` for a line
  without `NEW`, so `finish` is not owed.
- **The count moves across 300**, +30.
- **It moves across 400 (+31) and 500 (+118).** Religion is a temple
  border level, and Civic 3 is a border term.

**Each lever lands five blocks late and over five blocks.** Changes start
on 306, 406 and 505 and end on 310, 411 and 511. That is the budgeted
recompute (`GameDaemon::check_borders`, 256 cells a frame) seen in the
dump. Every cell that changes goes from −1 to 0.

### What the harness made of it

`chapter_four_s_border_is_widened_cell_for_cell` compares all 3,600 cells
on every block (`who`, `who2`, `flags`, `blocked`, `solid`, `bad`), plus
the buildings and the cities. It found three defects, one per lever,
before it had any assertions:

- **The interpreter ordered the Temple rather than placing it.** Its
  building arm took `Sim::place_building`, which is `Group::action_build`:
  it charges the price and leaves an unstarted site, so the city's temple
  bit and the border never moved. The arm is `Objects::init_build` and
  then `Build::activate(0, 1, 0)`, whose arguments are read from the
  listing at `0x7e055b`.
- **The temple border level was a constant 1.** It is now `1` plus the
  highest `TEMPLEBORDERS2..4` held (bonuses 28–30: Religion, Monotheism,
  Existentialism). The fort level is wired the same way.
- **`civic` changed the level and nothing re-read it.** `Leader::set_epoch`
  raises a level through a whole `gain_tech` per step. The crate's
  `set_leader_epoch` skipped `gain_tech`'s tail, including the border
  resync.

With the three fixes the border agrees **cell for cell** on 295–300, from
310 to 400, from 411 to 500 and from 511 to 544. The only parting is the
sweep's own windows, where this crate recomputes wholesale on the line's
frame.

## run133 — chapter four, the bleed: the namesake scored (2026-09-23, item 552)

The same script as run132, windowed on the bleed:

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch4u \
    --map 14 --end-frame 1500 --log-window 595 1500 --timeout 3600 \
    --detail end:UNITS=3,GUYS=2,DEATHS=1,LEADERS=2 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter4.cmd
```

`success: true`, exit 0, 1501 frames, map and seed read back, one take:
**372 s from launch to exit, 120 MB of dump and 12.0 MB of trace**. Ten
`INFO cmd` records each returned 1. There are 907 blocks: 1, 595..1499
and 1501. **It is run132's game**: `chapter_four_s_two_captures_are_one_game`
finds all 15,099 draws identical on every frame. The capture adds `37
!ffwd 2` for a 1500-frame run, where chapters two and five had `!ffwd 1`;
both chapter-four captures carry the same line.

### The predictions and §8's bleed falsifiers

| check | predicted | observed |
| --- | --- | --- |
| births | squad 601, scout 606, wagon 1101 | `1/6..1/8` on 601, `1/9` on 606, `1/10` on 1101 |
| the period | 48 from each figure's first refresh | `1/8` 601, `1/7` 602, `1/6` 603: 48 |
| the tick | 6/16 on `(f + o) % 48` | 617/618/619, then every 48, to 1337: 6 + 0/16 |
| the scout | 0 throughout | 0 on every block |
| the wagon | 0 throughout, at war | 0 on every block |
| the shelter | no tick with the wagon within 14 tiles | every tick at ≥ 23 tiles landed; every tick at 11–13 tiles vetoed, `0x40000` on the tick frame |

**None fired.** The caveat did happen: the squad left its placement. It
marched about 30 tiles east on player 0's ground, and the wagon trailed
after it and came within reach only at 1385. No figure fought; `damage` is
attrition's alone.

### What the harness made of it

Two wiring defects, both in the namesake (`docs/ATTRITION.md`, "Golden
chapter four"): the strength was never written from the tech tree (period
0 on every figure), and the squad size was a stored 1 (16/16 a tick). With
both fixed, the bleed agrees tick for tick to 1337. The word is **1277**:
the original's squad takes a `GUARDORDER` and a move back beside its
wagon, and this crate's keeps its `AttackTo`.

## run138 — Great Lakes 11901, the search read from inside, with its blocks (2026-09-23, item 566)

**What it is.** run137's stanza, retaken: run136's line over
`[11896, 11905]`, with `cover=0` and `callwin=11900-11902`, on a
`RON_COLLIDE_PROBE` tracer that adds **INFO 16**, the live and copied
`CollBlock` of each probe's world cell. Only `rontrace.dll` was swapped.
The plain bytes were saved and hashed first, and were restored and passed
`shasum -c` after. It launched through `viadriver.sh` at 01:41 and was
archived by 01:44 (31,880,756 bytes of dump, 17,335,520 of trace). A
startup watchdog stood by for run137's fault, and it did not recur.

**Why it was owed.** Item 563 showed the word 11903 was `1/62`'s search
taking S, (810, 441), where the original's did not. No reading of the
functions on the path found a difference (`docs/PATHFINDER.md` §24.1).

**Every check passed:**

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run53.log` | **0 differing**, 11,916 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 14 |
| the window, block for block | **10 blocks, 11896..11905** |
| overlap with run136, nothing excluded | **10 in common, 0 differ** |
| proxied sites | 13 |
| `1/62`'s search probes on 11901 | 90 |
| INFO 16 block records on 11901 | 207, 94 of them copies |

**What it settled.** The original never probes S. Its memo already
refuses it:
- `1/27`'s `find_upath` pre-walked row 441 earlier on the same frame and
  returned before `kill_lists`;
- its five verdicts, refused by `1/62`'s own block, stayed in the
  pathfinder's memo;
- `1/62`'s search read them silently, for six cells of row 441.

The live block and the copy both agree with this crate's index.
`docs/PATHFINDER.md` §24; `run136_s_word_is_one_cell_of_1_62_s_search`.

## run139 — East Indies' word 9983, the make list over it (2026-09-23, item 576)

**What it is.** run99's game and line with `LEADERS` raised to **9**,
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,DEATHS=1,LEADERS=9`, over
`[9960, 10000)`, plus `rontrace.cfg` `cover=1` and `window=9974-9986` over
the production cycle. `!quit` at 10010. **92,056,165 bytes of dump and
17,987,840 of trace, about seven minutes** from launch at 03:59 to archive
at 04:06. It was launched through `viadriver.sh` with no human at the
menu; the lane lock was att-566's from 01:41, and its holder was dead.
2.0 MB a block net of the 11 MB start dump, as sized from run82.

**Why it was owed.** Item 573 moved East Indies' word to 9983, a
`make_stuff` that buys different things on the two sides. run99 carries
the word at `LEADERS=1`, whose seven-key stub prints no make list, no
stockpile and no production step. The `LEADERS=9` East Indies captures
are run59, run82 and run96; nothing prints the list anywhere in
[6930, 23960).

**Every check passed:**

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run54.log` | **0 differing**, 10,011 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 18 |
| the window, block for block | **40 blocks, 9960..9999, no gap** |
| a `MAKEOBJECT` on every window block | **40** |
| overlap with run99, the leader's kinds excluded | **40 in common (9960..9999), 0 differ** |
| the coverage window | a set on all 13 frames 9974..9986 |

**What it settled.** Both lists are empty on block 9979 and the steps
agree block for block. The lists part on block **9982**, which is
`create_units`' frame (sim-frame 9981, by the coverage; the dump's
`production_step` is the step after the one that ran). There the
original offers three ships at the Dock, types 340, 334 and 323, each
`val 9999999` and cat 6, and this crate offers none. The Mine enters this
crate's list on 9983 in the slot the original's ship holds. The sea
branch finds its dock with a search around the city, and Dock `1/2010`
belongs to no city. `run139_s_word_frame_is_widened_whole`; `docs/AI.md`
§57.
## run141 — chapter seven, the civilians with the AI off (2026-09-23, item 578)

The golden record's seventh chapter (`docs/GOLDEN.md` §11), staged from
`tools/gamelog/golden/chapter7.cmd`: `!ai off` at 0, `library who=0 2` at
600, then `add citizen`, `caravan`, `merchant`, `scholar` and `fur` for
player 0 at 610–630 beside Napata. The first `add` of any of the five in
any capture on this disk. run133's command with the chapter swapped:

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch7 \
    --map 14 --end-frame 1200 --log-window 605 1200 --timeout 3600 \
    --detail end:UNITS=3,GUYS=2,BUILDS=7,CITIES=5,GOODS=3,LEADERS=2 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter7.cmd
```

`success: true`, exit 0, 1201 frames, `MAP_STYLE 14` and seed 12345 read
back, five settings files restored. One take: **281 s from launch to exit,
112 MB of dump and 11 MB of trace**. Nine `INFO cmd` records each returned 1
(the capture adds `37 !ffwd 2` and `1200 !quit`). 597 blocks: 1, 605..1199
and 1201; block 1 is only the `process_cheat_ai_toggle` command record.
The lane lock was stale (att-576's pid gone) and was taken over.

### The predictions, written into the `.cmd` file before the run

Committed as `719065a`. The prediction held for the citizen and failed for
the caravan:

| check | predicted | observed |
| --- | --- | --- |
| births | `0/6..0/10` on 611, 616, 621, 626, 631 | exactly |
| guy types | 50, 59, 61, 52, 400 | exactly; one, three, two, one and three figures |
| the citizen | `GATHERORDER` from `think_peasant` at `idle` 12, both runs | block 763, `idle 12`, `ox 2001 build_type 418`, a timber slot on 801 (income 480 → 640) |
| the caravan | `TRADEORDER` from `think_caravan`, both runs | **no order**: `think_caravan` finds no city |
| merchant, scholar, fur | no order | none, to 1199 |

### §11's falsifiers

- **The first fires**: a `GATHERORDER` on the citizen in the AI-off run.
  It is not the gate misread. The order comes from `think_peasant`, above
  the `ai off` block, and run142 shows the same order on the same block.
- The second is run142's.

## run142 — chapter seven's control, the same with the AI on (2026-09-23, item 578)

run141's line with `~/ron-golden/ch7c` and `--cmd-file
tools/gamelog/golden/chapter7_control.cmd`. That file is `chapter7.cmd`
less `0 !ai off` and nothing else. `success: true`, exit 0, 1201 frames,
map and seed read back. One take: **331 s, 112 MB of dump and 11 MB of
trace**. Eight `INFO cmd` records each returned 1. There are 596 blocks,
because no toggle means no block 1. The tracer DLL is rebuilt per run
(`build_seconds` 24), and its hash differs from run141's.

- **§11's second falsifier does not fire**: the citizen `0/6` takes its
  `GATHERORDER` on 763 at `idle 12`, as in run141.
- **The five hold the same orders on every block of both captures**
  (`chapter_seven_s_civilians_act_alike_with_the_ai_off_and_on`). `ai off`
  takes nothing from a human's civilians (`docs/INPUT.md` §11.9).
- **The pair is one game through frame 0, 120 draws, and parts on frame 1**,
  12 draws against 54 (`chapter_seven_s_pair_is_one_game_until_the_gate`).
  In total, 13,434 draws against 13,504.
- **The harness walks both to 1200 without a parting.** Draws, sequence
  and values all agree, and this crate's five match the original's on 763,
  900 and 1199 in both runs.

## run143 — East Indies' word 10398, the Bark and 340 blocks past it (2026-09-23, item 588)

**What it is.** run99's game and line with `LEADERS` raised to **9**, as
run139 was: `MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,DEATHS=1,LEADERS=9`
over `[10380, 10740)`, plus `rontrace.cfg` `cover=1` and
`window=10394-10402` over the word. `!quit` at 10760. **752,295,186 bytes
of dump and 18,546,592 of trace, 27 minutes** from launch at 05:40 to
archive at 06:07. It ran through `viadriver.sh` with no human at the menu.
The lane lock was stale: its holder, pid 89520, was dead. That is 2.06 MB
a block net of the 11 MB start dump, against run139's 2.0.

**Why it was owed.** The word's block, 10399, is run99's last. No capture
on this disk printed 10399 onward, and run99 prints `LEADERS=1`'s stub.
The walk back on run99 alone had already named the Bark's first step
(`docs/journal/2026-09-23-item-588.md`). `LEADERS=9` was taken for the
frames past the word, not for the walk back.

**Every check passed:**

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run54.log` | **0 differing**, 10,761 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 18 |
| the window, block for block | **360 blocks, 10380..10739, no gap** |
| a `MAKEOBJECT` on every window block | **360** |
| overlap with run99, the leader's kinds excluded | **20 in common (10380..10399), 0 differ** |
| the coverage window | a set on all 9 frames 10394..10402 |

**What it settled.** R1 and R2 of the stanza hold. On the word's blocks,
run143's widening (`run143_s_word_frame_is_widened_whole`) prints the
same eight rows run99's does, all the Bark's and the seated Scholar
`1/24`'s, and the same three one-sided animation changes. Nothing else
parts on 10397..10401. The leader record is whole on every block: 758,160
rows, none unprinted.
## run144 — East Indies 10582, the first packet on this map (2026-09-23, item 597)

**What it is.** A `RON_STATE_FRAME=10582` packet on the click-free lane,
with run143's dump detail over [10578, 10586). `!quit` is at 10590.
`success: true`, exit 0, `MAP_STYLE 18` and seed 12345 read back, five
settings files restored. It took **68 s launch to exit and 76 s in all**,
with an 818,699,196-byte packet (174 ranges, 188 ms to copy) and 16 MB of
dump. The lane lock was stale: pid 44969 was dead.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-23-run144 \
    --map 18 --end-frame 10590 --timeout 2400 --log-window 10578 10586 \
    --detail end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,DEATHS=1,LEADERS=9 \
    --tracer-def RON_STATE_FRAME=10582 \
    --tracer-def 'RON_STATE_PLAN="<plan>/plan.h"'
```

The plan is the lab's Great Lakes plan, reused: it is PDB-bound and does
not depend on the frame, and the image hash matches. It is copied to
`~/ron-data/lab-experiments/2026-09-23-item-597/plan/`. The first launch
refused in a second, because the output directory already existed; the
runner wants a fresh one.

**Why 10582 and not 10581.** The word is a trace tick. Block N is the state
after tick N−1, and the lab's packet read logger 11,186 after trace 11,185.
Mine `1/2018` is first printed on run143's block 10583. So the packet
before the placement is logger frame 10582, which is after trace tick 10581.

**Every check passed:**

| check | result |
|---|---|
| `frame_snapshot.py` against the plan | frame 10582, trace 10581, the logger return's roots unchanged |
| `rngcmp.py` against `rontrace-run54.log` | **0 differing**, 10,591 identical |
| `samegame.py` against run143 | 8 in common (10578..10585), **0 differ** |
| `produce_building` on the packet | **7** `+0xc99` and **3** `+0x1805`, the original's own counts |

**What it settled.** At each of the five extra spiral cells the original's
`blocked_site` answers `NoMountain`, and `find_nearest` measures 1536 to a
solid mountain cell (`docs/AI.md` §59). The packet stays outside git at
`~/ron-data/lab-captures/2026-09-23-run144/map-18`, and its readings at
`~/ron-data/lab-experiments/2026-09-23-item-597/`.

## run145 — chapter three, the mounted and siege lines (2026-09-23, item 587)

The golden record's third chapter (`docs/GOLDEN.md` §7), staged from
`tools/gamelog/golden/chapter3.cmd`: `!ai off` at 0, `age who=0 2` and
`age who=1 2` at 600 and 602 (the Classical age; the file said `4`, which
is Gunpowder, until item 587), `add 3 chariot who=0 4,40` at 610, `add
hoplite who=1 12,40` at 615, `add catapult who=0 4,41` at 620. run141's
command with chapter two's dump set:

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch3 \
    --map 14 --end-frame 900 --log-window 605 900 --timeout 3600 \
    --detail end:UNITS=3,GUYS=2,AMMO=5,DEATHS=1,LEADERS=2 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter3.cmd
```

`success: true`, exit 0, 901 frames, `MAP_STYLE 14` and seed 12345 read
back, five settings files restored. One take: **144.5 s from launch to exit,
43 MB of dump and 10 MB of trace**. Eight `INFO cmd` records each returned 1.
297 blocks: 1, 605..899 and 901. The lane lock was stale (pid 3800 gone) and
was taken over.

**What the disk had before it** (`rg` over the `Logs` archive and
`~/ron-golden` for a `GUY` of type 195 or 265): no Chariot anywhere; Catapults
only in run44 (Islands, 700 frames, `GUYS=4`, no `AMMO`). This is the first
catapult and the first chariot in a capture with `AMMO` on.

### The predictions, written into the `.cmd` file before the run

Committed as `aeb4c70`.

| check | predicted | observed |
| --- | --- | --- |
| births | three separate Chariots `0/6..0/8` on 611, hoplites `1/6..1/8` on 616, catapult `0/9` on 621 | exactly; `o_up`/`o_down` −1 on each chariot, two type-195 figures each; three type-265 figures on the catapult |
| `myspeed` | 30, 25, 19 | exactly |
| the chariots fire | yes, orders near 635 | orders on 633, 634, 635; rounds from 651; the hoplites die from 678 |
| the catapult fires | yes, ordered near its birth | **no round, ever.** Born packed (`unit_masks 0x80000`), no order until a `CASTORDER spell 652` (`0x28c`, the unpack) on 696 at `idle 7`; unpacked (mask 0) by 899 |

### §7's falsifiers

- **The leading count: does not fire.** Three units, not nine or one.
- **The catapult's minimum range: cannot fire.** It never launches, and no
  hoplite comes within 3 tiles of it (closest 759 units, 3.95 tiles, on 670).
- **The chariots' speed: cannot fire.** The chariots never move. The
  hoplites stall about 3.3 tiles short of them from about 670 and are shot
  where they stand.

### The harness

The word is **621**, ours 7 draws against 6: `Unit::fight+0x9b0` on the
catapult `0/9`, which this crate orders to attack on its birth block while
packed. Values part on 622. `GOLDEN_WORD_CHAPTER_THREE` and
`chapter_three_s_word_frame_is_widened_whole`.

## run146 — chapter three restaged, the dead zone and the chase (2026-09-23, item 587)

`tools/gamelog/golden/chapter3b.cmd`, the commander's ruling after run145 left
two of §7's falsifiers untested. The unit types are the same, split into two
arenas 24 tiles apart. Arena A: `add catapult who=0 4,41` at 605, alone,
then `add hoplite who=1 12,41` at 770. Arena B: `add 3 chariot who=0 2,68`
at 610 and `add hoplite who=1 12,68` at 615, ten tiles apart. The `age`
lines and `!ai off` are run145's. The command is run145's with
`~/ron-golden/ch3b`, `--end-frame 1000 --log-window 605 1000` and the new
file.

`success: true`, exit 0, 1001 frames, map 14 and seed 12345 read back. One
take: **185.8 s, 55 MB of dump and 10 MB of trace**. Nine `INFO cmd` records
each returned 1. 397 blocks: 1, 605..999 and 1001. It waited on att-588's
`runqueue.sh`, whose RonDriver held the lane after run143's game had exited.
The first launch, at 06:07, was never serviced; the second, at 06:08:39,
ran.

### The predictions, written into the `.cmd` file before the run

Committed as `2a19c01`.

| check | predicted | observed |
| --- | --- | --- |
| births | catapult `0/6` on 606; Chariots `0/7..0/9` on 611; hoplites `1/6..1/8` on 616 and `1/9..1/11` on 771 | exactly |
| the unpack, with nothing in view | `CASTORDER spell 652` near 681 at `idle 7`, unpacked by ~761 | **699 at `idle 8`**, unpacked on 779: idle-driven as read, one grid step later than my arithmetic |
| the catapult fires | a round at arena A's hoplites in its 3–15 band | an attack order on `1/11` on 780, reload from 781, one round in flight by 798 |
| hoplites close inside 570 while it lives | yes | 173 blocks, 827–999, closest 339 units |
| a chariot walks ≥ 3 blocks | yes, near 633 | `0/9` five blocks from 634, `0/7` four from 635 |

### §7's falsifiers, all three reachable

- **The minimum range: does not fire.** One launch, at eight tiles. From 864
  the catapult takes an `ATTACKORDER` on a hoplite inside three tiles and
  drops it the next block, over and over, and fires nothing. Its `damage`
  climbs 12 → 72 by 999.
- **The speed: does not fire.** The chariots' longest one-block step is 33.2
  units (step² 1105), against the hoplites' 29.1 (848).
- **The leading count: does not fire.** Three Chariots, as in run145.

`chapter_three_s_falsifiers_are_the_dump_s` makes all three assertions. It
was made to fail once, by widening the dead zone to nine tiles.

### The harness

The word is **633**, ours 8 draws against 9. The original's chasing chariot
`0/8` spends one `Unit::fight+0x9b0`, then two `Guy::set_anim+0xf2f`, one
per figure as its walk starts. This crate spends a second re-search and
starts the walk on 634. Values part on 634. Pinned as
`GOLDEN_WORD_CHAPTER_THREE_RESTAGE` with
`chapter_three_s_restage_is_widened_whole`.

## run149 — East Indies' word 10782, past run143's last block (2026-09-23, item 608)

**What it is.** run143's game and line past its last block:
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,DEATHS=1,LEADERS=9` over
`[10730, 10880)`, plus `rontrace.cfg` `cover=1` and `window=10778-10786`
over the word. `!quit` at 10900. **321,162,982 bytes of dump and
18,694,976 of trace, 12 minutes** from launch at 09:00 to archive at 09:12.
It ran through `viadriver.sh` with no human at the menu. The lane lock was
stale: its holder, pid 21551, was dead. That is 2.07 MB a block net of the
11 MB start dump, against run143's 2.06.

**Why it was owed.** The word 10782 writes block 10783. No `MAP_STYLE 18`
dump on this disk carries a block in [10740, 10900]: run143's window ends
on 10739, and its next block is the `!quit` stub on 10761. `LEADERS=9`
was taken because the original's parting draw is `make_stuff+0x63d`, so
the make list is on the word.
## run147 — chapter three's pivot node, a packet at 684 (2026-09-23, item 603)

**What it is.** A `RON_STATE_FRAME=684` packet on chapter three's staging
(`chapter3.cmd`, run145's), with `misc:MISC=10` so that
`Guy::set_all_pivots` says the pivot node's vector, and `GUYS=4` for the
figures' turret fields. `!quit` is at 690. `success: true`, exit 0, 691
frames, `MAP_STYLE 14` and seed 12345 read back, five settings files
restored. **48 s launch to exit, 56 s in all**, with an 818,895,728-byte
packet and 13 MB of dump. The lane lock was stale (pid 332 dead).

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-23-run147 \
    --map 14 --end-frame 690 --log-window 680 690 --timeout 2400 \
    --detail end:UNITS=3,GUYS=4,AMMO=5,DEATHS=1,LEADERS=2 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1,MISC=10 \
    --cmd-file tools/gamelog/golden/chapter3.cmd \
    --tracer-def RON_STATE_FRAME=684 \
    --tracer-def 'RON_STATE_PLAN="<plan>/plan.h"'
```

The plan is run144's (`~/ron-data/lab-experiments/2026-09-23-item-597/plan/`).
The first launch refused before the game started: `live_session.py` took
single-digit detail levels only, and the say is at 10. `GameLog::details`
is a byte, so the runner now takes up to 255.

**What the disk could not answer.** No dump prints the pivot node. `AMMO`'s
`sx, sy` is the release node. Both earlier packets (run144, and the lab's
Great Lakes one) hold the Chariot's pieces loaded but with no `AttachPos`.

**Every check passed:**

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run54.log` | **0 differing**, 10,901 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 18 |
| the window, block for block | **150 blocks, 10730..10879, no gap** |
| a `MAKEOBJECT` on every window block | **150** |
| overlap with run143, every kind compared | **10 in common (10730..10739), 0 differ** |
| the coverage window | a set on all 9 frames 10778..10786 |

**What it settled.** The draw streams are one stream shifted. Both
sides spend `make_stuff+0x63d` at index 4, the original a second at 5,
and the same `Animal::do_idle+0x83` comes after. The widening
(`run149_s_word_frame_is_widened_whole`) names the make list on 10781.
The original offers the Citizen with `num 4`, and this crate with 1. That
count is sized by a census that parts on 10776: `reg_gather_slots` of
player 1's home region, 17 against 20, on the first census after the Mine
`1/2018` starts building. `docs/AI.md` §61.

With `count_gather_slots` counting unfinished buildings, the word moves to
10982, past this window. The widening keeps the move's value diff on
10776 and 10781..10783.

## run150 — the Mine's `gather_max` before it starts, a packet (2026-09-23, item 608)

**What it is.** A `RON_STATE_FRAME=10765` packet on the click-free lane,
with run149's dump detail over [10761, 10769). `!quit` is at 10775.
`success: true`, exit 0, `MAP_STYLE 18` and seed 12345 read back, five
settings files restored. **67 s launch to exit, 79 s in all.** The lane
lock was stale: its holder, pid 69563 (run149's), was dead. The plan is
597's, reused.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-23-run150 \
    --map 18 --end-frame 10775 --timeout 2400 --log-window 10761 10769 \
    --detail end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,DEATHS=1,LEADERS=9 \
    --tracer-def RON_STATE_FRAME=10765 \
    --tracer-def 'RON_STATE_PLAN="<plan>/plan.h"'
```

**Why 10765.** The Mine `1/2018` is placed on 10583 and `Wall::start`
runs on 10766 (`frame_started`). 10765 is after its placement and before
its start. The census counted it on the sweep that writes block 10776.

**What it settled.** The unstarted Mine (`flags 1`, `frame_started -1`)
holds **`gather_max 3`** and a 46-tile list, and it is city 1's chain's
tail. Leader 1's `reg_gather_slots[11]` reads 17, the previous sweep's. So
`count_gather_slots` counts it with no completion test (`docs/AI.md` §61).
The packet stays outside git at
`~/ron-data/lab-captures/2026-09-23-run150/map-18`.
| `frame_snapshot.py` against the plan | logger frame 684, the logger return's roots unchanged, receipt checked |
| `rngcmp.py` against run145's `rontrace.log` | **0 differing**, 691 identical |
| blocks 680–689 against run145 | every line run145 prints is there and equal; run147 only adds `GUYS=4` fields and the say lines |

**What it settled** (`docs/COMBAT.md` §54). The say line is `GUY
get_positiong -180 180 -102 -59 1512 7944` during tick 684: the Chariot's
node vector at 120°, against `1/7`. The packet's piece 145 holds node 4 at
`(0.0, 24.64, 11.55)`, and `guy_scale` is 4.8. `Unit::set_attack(0/8, 7,
1)` on the packet answers 1. `des_turret_angles[0]` reads `−496063829`
on 685. The packet stays outside git at
`~/ron-data/lab-captures/2026-09-23-run147/map-14`, and the oracle script and
its tables at `~/ron-data/lab-experiments/2026-09-23-item-603/`.

## run152 — East Indies' word 10982, past run149's last block (2026-09-23, item 613)

**What it is.** run149's game and line past its last block:
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,DEATHS=1,LEADERS=9` over
`[10870, 11040)`, plus `rontrace.cfg` `cover=1` and `window=10978-10986`
over the word. `!quit` at 11060. **362,519,989 bytes of dump and 18,841,632
of trace, 14 minutes** from launch at 09:51 to archive at 10:06. It ran
through `viadriver.sh` with no human at the menu. The lane lock was stale:
its holder, pid 18366, was dead. That is 2.07 MB a block net of the 11 MB
start dump, the same as run149.

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 613
```

**Why it was owed.** The word 10982 writes block 10983. No `MAP_STYLE 18`
dump on this disk carries a block in [10880, 11000]. run149's window ends
on 10879 and its next block is the `!quit` stub on 10901. run78 starts at
15700, run96 at 23960, and run54 and the 24k runs print only their end
frames. `LEADERS=9` was taken because the original's frame is player 1's
`make_stuff`.

**Every check passed:**

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run54.log` | **0 differing**, 11,061 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 18 |
| the window, block for block | **170 blocks, 10870..11039, no gap** |
| a `MAKEOBJECT` on every window block | **170** |
| overlap with run149, every kind compared | **10 in common (10870..10879), 0 differ** |
| the coverage window | a set on all 9 frames 10978..10986 |

**What it settled.** The make list parts first on 10981, the Light Horse's
offer: `check_income` answered 0x40 here against 0x100, because this crate
never took Horses' 15% off a Stable unit. With that in, the frame's spiral
scored seven friendless sites against four, because a range another Mine
gathers from is taken on the survey. `docs/AI.md` §62. The word moves to
11069, past this window. The widening,
`run152_s_word_frame_is_widened_whole`, keeps the move's value diff on
10981..10983.

## run156 — chapter seven-b, the computer's civilians with the AI off (2026-09-23, item 628)

`docs/GOLDEN.md` §11's restage, from `tools/gamelog/golden/chapter7b.cmd`:
`!ai off` at 0, `library who=1 2` at 600, then `add citizen`, `caravan`,
`merchant`, `scholar` and `fur` for player 1 at 610–630, on the clear
south-east side of London (tiles 228–232, 92–108). This is the first capture
on this disk to stage a who=1 unit on London's ground. It is run141's
command with the chapter swapped:

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch7b \
    --map 14 --end-frame 1200 --log-window 605 1200 --timeout 3600 \
    --detail end:UNITS=3,GUYS=2,BUILDS=7,CITIES=5,GOODS=3,LEADERS=2 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter7b.cmd
```

`success: true`, exit 0, 1201 frames, `MAP_STYLE 14` and seed 12345 read
back, five settings files restored. One take: **295 s from launch to exit,
106 MB of dump and 10 MB of trace**. All nine `INFO cmd` records returned 1.
There are 596 blocks: 1 and 605..1199. The `.cmd` predicted 597, but no
`!quit` block at 1201 was written this time. No test reads it.

### The predictions, committed before the run as `4fa528f`

| check | predicted | observed |
| --- | --- | --- |
| births | five who=1 `UNITDATA` on 611, 616, 621, 626, 631 | exactly, `1/6..1/10`; guy types 50, 59, 61, 52, 400 |
| `unit_masks & 0x40000` on all five | set, by `Unit::init:585` | set on every birth block |
| the citizen | `GATHERORDER` on its first think, 611 or 612 | **611**, its birth block |
| the caravan, the scholar | no order | none, to 1199 |
| the merchant, the fur trapper | lower confidence: may take an order in both runs | `MOVEORDER` on birth; `CASTORDER` on 900 and on 1151 |

### §11's falsifiers

- **The first fires, by construction**: the citizen's `GATHERORDER` on
  611, held and carried through 1199 (`unit_masks` `0x10000000` on 793).
- The second is run157's.
- **The third**: this crate parts at **1148**, the fur trapper's turn, 13
  draws against 14. It is not the seam. See `docs/GOLDEN.md` §11.

## run157 — chapter seven-b's control, the same with the AI on (2026-09-23, item 628)

run156's line with `~/ron-golden/ch7bc` and `--cmd-file
tools/gamelog/golden/chapter7b_control.cmd`, which is `chapter7b.cmd` less
`0 !ai off`.

**The first take stalled before the menu.** The game sat at 100% CPU for
nineteen minutes. `rontrace.log` stayed at 544 bytes: eight lines parsed at
attach, none run. The dump never left the profile. That is the click-free
lane's pre-menu failure, which `docs/lab/2026-09-09-autostart.md` records
twice. I stopped the game (`kill` of its own pid, never a Wine-wide kill).
The runner wrote a failed receipt (`ValueError: process failed: 1`,
`settings_restored: true`, five files) and restored the profile. The take
is kept aside as `~/ron-golden/ch7bc-stalled`.

**The second take**: `success: true`, exit 0, 1201 frames, map and seed read
back, five files restored, **335 s, 114 MB of dump and 11 MB of trace**. All
eight `INFO cmd` records returned 1. There are 595 blocks: 605..1199, with
no toggle block and no `!quit` block.

- **§11's second falsifier does not fire**: the citizen, `1/9` here (the AI
  trained three citizens before 605), holds its `GATHERORDER` on 611, as in
  run156.
- **The five act as in run156** except for the merchant's cast, on 887
  against 900
  (`chapter_seven_b_s_civilians_act_alike_with_the_ai_off_and_on`).
- **The pair is one game through frame 0 and parts on frame 1**, 12 draws
  against 54 (`chapter_seven_b_s_pair_is_one_game_until_the_gate`).
- This crate parts at **1036**, on who=1's own `1/1`, 8 draws against 7.
## run155 — East Indies' word 11069, past run152's last block (2026-09-23, item 620)

**What it is.** run152's game and line past its last block:
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,DEATHS=1,LEADERS=9` over
`[11030, 11280)`, plus `rontrace.cfg` `cover=1` and `window=11065-11073`
over the word. `!quit` at 11300. **529,523,354 bytes of dump and
19,032,928 of trace, 20 minutes** from launch at 12:43 to archive at
13:03. It ran through `viadriver.sh` with no human at the menu, and the
lane lock was free. That is 2.07 MB a block net of the 11 MB start dump,
the same as run152.

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 620
```

**Why it was owed.** The word 11069 writes block 11070. The only block any
`MAP_STYLE 18` dump on this disk has in [11040, 11100] is run152's
`!quit` stub on 11061. run152's window ends on 11039. run78 starts at
15700, run96 at 23960, and run54 and the 24k runs print only their end
frames. The original's first draw on the word is `Unit::do_move+0xe84 <
Unit::do_explore_to`, a walk's grid roll, and no record on disk says
whose walk it is.

**Every check passed:**

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run54.log` | **0 differing**, 11,301 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 18 |
| the window, block for block | **250 blocks, 11030..11279, no gap** |
| a `MAKEOBJECT` on every window block | **250** |
| overlap with run152, every kind compared | **10 in common (11030..11039), 0 differ** |
| the coverage window | a set on all 9 frames 11065..11073 |

**What it settled.** `run155_s_word_frame_is_widened_whole`; the item's
journal (`docs/journal/2026-09-23-item-620.md`) has the verdicts on the
three readings.

## run159 — East Indies' word 11590, past run155's last block (2026-09-23, item 629)

**What it is.** run155's game and line past its last block:
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,DEATHS=1,LEADERS=9` over
`[11270, 11900)`, plus `rontrace.cfg` `cover=1` and `window=11586-11594`
over the word. `!quit` at 11920. **1,324,073,337 bytes of dump and
19,374,848 of trace, 46 minutes** from launch at 13:54 to archive at
14:40. It ran through `viadriver.sh` with no human at the menu, after a
backgrounded wait on the lane lock, which att-628's run156 and run157
held. That is 2.08 MB a block net of the 11 MB start dump, the same as
run155.

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 629
```

**Why it was owed.** The word 11590 writes block 11591. No `MAP_STYLE
18` dump on this disk has a block in [11500, 11700]. run155's blocks end
on 11301 (its `!quit` stub), run78 starts at 15700 and run96 at 23960,
and run54 and the 24k runs print only their end frames. The draw stream
names the herd sheep `8/1`'s arrival idle, eight frames early here, and
no record on disk says where the original's sheep walks.

**Every check passed:**

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run54.log` | **0 differing**, 11,921 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 18 |
| the window, block for block | **630 blocks, 11270..11899, no gap** |
| a `MAKEOBJECT` on every window block | **630** |
| overlap with run155, every kind compared | **10 in common (11270..11279), 0 differ** |
| the coverage window | a set on all 9 frames 11586..11594 |

**What it settled.** `run159_s_word_frame_is_widened_whole`; the item's
journal (`docs/journal/2026-09-23-item-629.md`) has the verdicts on the
readings.

## run168 — chapter six, the air and the bird (2026-09-23, item 648)

`docs/GOLDEN.md` §10's first capture, from `tools/gamelog/golden/chapter6.cmd`:
`!ai off` at 0, `library who=0 6` and `library who=1 6` at 600 and 602,
`add fighter who=0 4,40` at 610, `add bomber who=1 12,40` at 615, and `bird`
at 700. The staging was read and committed before the run (`bf981e6`). It
is the first capture on this disk with an aircraft in it.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch6 \
    --map 14 --end-frame 900 --log-window 605 900 --timeout 3600 \
    --detail end:UNITS=3,GUYS=2,AMMO=5,DEATHS=1,LEADERS=2 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter6.cmd
```

`success: true`, exit 0, 901 frames, `MAP_STYLE 14` and seed 12345 read
back, five settings files restored. One take, **137 s from launch to exit,
42 MB of dump and 9.8 MB of trace**. The lane lock named att-571's
`longtrace.sh`, whose pid had exited, so the launch took it over. All eight
`INFO cmd` records returned 1. There are 297 blocks: 1, 605..899 and the
`!quit` block 901.

### The predictions, committed before the run

| check | predicted | observed |
| --- | --- | --- |
| the Fighter | a who=0 `UNITDATA` on 611 at (888, 7800) | `0/6` on 611 at (888, 7800) |
| the Bomber | a who=1 `UNITDATA` on 616 at (2424, 7800) | `1/6` on 616 at (2424, 7800), `unit_masks` `0x40000` |
| fuel | `mana_burn` one a frame from birth | 1 on the birth block, 289 and 284 on 899 |
| the bird's birth | one `Guy::init_real` draw, first on 700 | draw 0 of 700, `< Unit::init < Animal::init` |
| the bird's think | a seventh `think_bird` from 704 | six birds on 672 and 696, seven from 704 |
| the sampling | one pair short from 704 | 4 pairs on 672, 3 on 704, 736, 768 and 800 |
| the cursor at the corner | edge coins within the bird's first frames | `do_air_physics+0x639` on 750, 791 and 894 only: **not confirmed** |
| owner 9 in the dump | none | none |

### §10's falsifiers

- **The first does not fire**, on its restated form: the bird is born on
  700 and thinks from 704. The dump never prints it.
- **The second fires.** Neither aircraft moves. Both stand on their seats
  at `air_alt` 0 with an empty order stack from birth to 899, and no
  `AMMO` block is written in the whole capture. The Bomber sits in the AI's
  group 64 from its birth block, the Fighter in none.
- **The third does not fire**: `add` placed both aircraft with no base.

This crate parts at **616**, the Bomber's birth frame, 25 draws against 24:
a `Unit::fight+0x9b0` of `1/6`'s that the original does not spend. See
`docs/GOLDEN.md` §10.

## run163 — Great Lakes' word 12038, past run136's last block (2026-09-23, item 571)

**What it is.** run136's line, unchanged,
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, over
`[11950, 12400)`, plus `rontrace.cfg` `cover=1` and `window=12034-12042`
over the word. `!quit` at 12410. **961,670,782 bytes of dump and 17.9 MB
of trace, 34 minutes** from launch at 16:47 to archive at 17:21, through
`viadriver.sh` with no human at the menu. The lane lock was free.

**Why it was owed.** Item 566 moved Great Lakes' word to 12038. run136, the
only capture past 11859, ends on block 11959. The window runs 360 blocks
past the word because this crate's fix for block 11922, measured before
the run, moved the word to 12135.
## run169 — chapter six's cursor, a packet at logger frame 701 (2026-09-23, item 652)

**What it is.** run168's game to 712 with a `RON_STATE_FRAME=701` packet:
the state after trace tick 700, whose entry runs `bird`. The dump detail is
run168's over [699, 705), with no `[Start Game]` set, because nothing walks
this capture. `success: true`, exit 0, `MAP_STYLE 14` and seed 12345 read
back, five settings files restored. **17 s launch to exit, 25 s in all**,
with an 818,813,928-byte packet (173 ranges, 250 ms to copy). The lane
lock was stale: its holder, pid 2339 (run168's), was dead.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-23-run169 \
    --map 14 --end-frame 712 --timeout 2400 --log-window 699 705 \
    --detail end:UNITS=3,GUYS=2,AMMO=5,DEATHS=1,LEADERS=2 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter6.cmd \
    --tracer-def RON_STATE_FRAME=701 \
    --tracer-def 'RON_STATE_PLAN="<plan>/plan.h"'
```

The plan is 597's (`~/ron-data/lab-experiments/2026-09-23-item-597/plan/`).
It is PDB-bound and does not depend on the frame or the map.

**Why this and not the draw stream.** run168's trace puts the bird in the
corner and cannot say where in it. `Unit::init` snaps every cursor in
[0, 48)² onto the same seat, (24, 24), and only the unsnapped patrol point
differs. A walk with the cursor at (0, 0) agrees through the 750 and 791
edge coins and parts on 894, the third coin, two frames early.

**Every check passed:**

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run53.log` | **0 differing**, 12,411 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 14 |
| the window, block for block | **450 blocks, 11950..12399, no gap** |
| a `GROUPDATA` on every window block | **450** |
| overlap with run136, nothing excluded | **10 in common (11950..11959), 0 differ** |
| the coverage window | a set on all 9 frames 12034..12042 |

**What it settled.** The word's fifth draw is `1/62`'s arrival stand. On
block 12039 the original's `1/62` hard-collides with `1/64`
(`collide_o 64`) and goes WALK → DEFAULT. Ours is 49 units off its point,
and walks on. The chain runs back unbroken to block 11922, where who=1's
barracks research converts the nine walking type-82 figures, and the
original's `Guy::init_real` stands each one. `docs/ANIM.md` §11;
`run163_s_word_frame_is_widened_whole`. It is also the first window on
this map to print a grouped attack-move (`GROUPATTACKTOORDER`); the
coverage pin reads it.
| `frame_snapshot.py` against the plan | frame 701, trace 700, the roots unchanged |
| `rngcmp.py` against run168's `rontrace.log` | **0 differing**, 713 identical, 700–712 included |
| `samegame.py` against run168 | 7 blocks in common (1, 699..704), **0 differ** |

**What it settled.** Two reads, which agree:

- `MiscAccess::console_win` → the `ConsoleWin` (`ConsoleWin::vftable`
  `0xb51644` at +0, `coord_mode` 2). `mouse_coord_x/y` at +0x518/+0x51c
  read **(0, 6)**.
- Exactly one `AirPatrolOrder` is fresh: it has `cruising_alt` 0x640, as
  `add_air_patrol_order` writes it, where the six older birds' read 1800,
  the re-roll's. Its target is −1/−1 and it holds one waypoint, **(0, 6)**.
  The patrol sub-object is at the order's +0x10, the waypoint through
  `x_pos.data` at +0x24 and `y_pos.data` at +0x40.

Bird `9/6`'s unit seat reads (24, 24), the snap of (0, 6). A unit's
`+0x78` for a bird is its birth tile's centre and is not updated in flight
(the six older birds read their hatch cells' centres plus 24). With the
cursor at (0, 6), chapter six walks run168 to its end at 900 with no
draw parting (`docs/GOLDEN.md` §10). Whether the heap leaves (0, 6) on
every launch is parked 653; run169 is a second launch that left the same
value. The packet stays outside git at
`~/ron-data/lab-captures/2026-09-23-run169/map-14`, and the scratch reader
at `~/ron-data/lab-experiments/2026-09-23-item-652/pk.py`.

## run171 — chapter eight, the commanders and a declared war (2026-09-23, item 660)

`docs/GOLDEN.md` §12's first capture, from `tools/gamelog/golden/chapter8.cmd`:
`!ai off` at 0; `library who=0 2` and `library who=1 2` at 600 and 602; a
hoplite squad at `4,40` and a General at `5,40` for who=0 on 610 and 612; a
hoplite squad at `12,40` and a Spy at `12,41` for who=1 on 615 and 617;
then `peace 1`, `war 1` and `ally 1` on 700, 800 and 900. The staging was
read and committed before the run (`89e338e`). It is the first capture on
this disk with a General or a Spy in it, and the first golden capture at
`LEADERS=5`.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch8 \
    --map 14 --end-frame 1200 --log-window 605 1200 --timeout 3600 \
    --detail end:UNITS=3,GUYS=2,AMMO=5,DEATHS=1,LEADERS=5 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter8.cmd
```

One take, **168 s from launch to exit, 54 MB of dump and 10 MB of trace**.
The lane lock named a pid that had exited, so the launch took it over.
Eleven of the twelve `INFO cmd` records returned 1; `1200 !quit` never
ran. **`ally 1` ended the game**: both leaders carry the victory bit on
block 901 (`leader_flags` 7 → 167 and 19 → 179, `0x20 | 0x80`), the dump
closes after that block with `GameInfo closing`, and the trace's last frame
is 900. So the receipt says `success: false`, **"missing, repeated, or
unexpected simulation frames"**, and it means a game that ended itself, not
a lane that failed: 298 blocks, 1 and 605..901, all whole.

**Why**, read after the run. `Leader::set_diplo@006ec6a0` at level 2 counts
the live leaders allied to neither side and calls `Leader::victory` at
zero. Its loop runs up to `0xe71af0`, which is `leaders` (`0xe3a390`, the
PDB's `S_GDATA32` at `.data + 0x234390`) plus eight `Leader`s of `0x6eec`.
Gaia's leaders are 8 and 9, outside it. The staging read the loop as all
ten, and predicted the game would run on; it was wrong. **A two-player lobby
cannot hold an alliance and a game at once.**

### The predictions, committed before the run

| check | predicted | observed |
| --- | --- | --- |
| the squads' first orders | an `ATTACKORDER` on both by 636 | both on **635** |
| blows before the peace | a `damage` step on both sides before 700 | 0/7 from 660, 1/8 from 661, last on 699 |
| the General | stays inside six tiles of the fight | seated (1368, 7992); never moves, never struck |
| the Spy | "the capture's to say" | an `EXPLORETOORDER` on its birth block, 618, to (4344, 10488); away by 752 |
| `diplos[1]` of 0 and `diplos[0]` of 1 | 0 to 700, 1 on 701, 0 on 801, 2 on 901 | exactly so |
| no allied victory at 900 | the game runs on | **wrong**: the game ends |
| `ages_get()`/`epochs_get()` | 2 and 8 on both leaders | 2 and 8 |

### §12's falsifiers

- **The first does not fire**: the row moves on the block after each line.
- **The second does not fire**: every attack order on both squads is gone
  by 731, and no blow lands in (699, 901]. who=1's squad takes an
  `ATTACKTOORDER` on 765, in the peace, and none lands. After `war 1`,
  who=0's squad attacks on 827 and chases to x 3000 by 891 without a blow.
- **The third does not fire**: a plain blow on a who=0 hoplite, a
  General's 1.6 tiles off, is **2** (660, 663, 692, 695), and on a who=1
  hoplite it is **3** (661, 667, 693, 699). The 7-point blows are the same
  on both sides (665 and 697 on who=1, 672 on who=0). The rally armor
  shows in the blow and nowhere else; its size, one point where rules.xml's
  `GENERAL_RALLY_ARMOR` reads 2, is not established.

This crate parts at **617**, the Spy's birth frame, 9 draws against 80, at
draw 3: the original's `Unit::think_scout+0x941`. See `docs/GOLDEN.md` §12.

## run174 — Great Lakes' word 12429, past run163's last block (2026-09-23, item 669)

**What it is.** run163's line, unchanged,
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, over
`[12390, 12900)`, plus `rontrace.cfg` `cover=1` and `window=12425-12433`
over the word. `!quit` at 12910. **1,095,614,920 bytes of dump and 18.3 MB
of trace, 40 minutes** from launch at 21:14 to archive at 21:54, through
`viadriver.sh` with no human at the menu. The lane lock was stale: its
holder, pid 98400, was dead.

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 669
```

**Why it was owed.** Item 661 moved Great Lakes' word to 12429. run163, the
last capture on this map below run80's 23960, ends on block 12399, and no
`gamelog-*greatlakes*` holds block 12429 or 12430 (grepped before booking).
The window runs 469 blocks past the word so that a fix that moves it is
measured on the same capture.

**Every check passed:**

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run53.log` | **0 differing**, 12,911 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 14 |
| the window, block for block | **510 blocks, 12390..12899, no gap** |
| a `GROUPDATA` on every window block | **510** |
| overlap with run163, nothing excluded | **10 in common (12390..12399), 0 differ** |
| the coverage window | a set on all 9 frames 12425..12433 |

**What it settled.** The word's sixth draw is `1/67`'s stop. On block
12430 ours' `1/67` hard-collides with `1/68` (`collide_o 68`) and stops;
the original's has no collider and walks on. `1/68` has walked a leg the
original does not since 12422, because its tile plan on 12322 dropped the
group move's exact formation point that the original keeps
(`docs/AI.md` §64; `run174_s_word_frame_is_widened_whole`).

## run178 — Great Lakes' word 14382, past run174's last block (2026-09-24, item 678)

**What it is.** run174's line, unchanged,
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, over
`[12894, 14900)`, plus `rontrace.cfg` `cover=1` and `window=14378-14386`
over the word. `!quit` at 14910, through `viadriver.sh` with no human at
the menu.

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 678
```

**Why it was owed.** Item 678 moved Great Lakes' word 12897 → 14382 (the
pathfinder's block copies, `docs/PATHFINDER.md` §26). run174, the last
capture on this map below run80's 23960, ends on block 12899, and no
`gamelog-*greatlakes*` holds block 14382 or 14383 (grepped before booking,
the pattern checked against run163's 12180). The window overlaps run174 by
six blocks so the widening walks one chain from run123's 11400 to the
word, and runs 516 blocks past it so a fix that moves the word is measured
on the same capture.
## run175 — chapter six-b, the air line from a base (2026-09-23, item 651)

`docs/GOLDEN.md` §10's restage, from `tools/gamelog/golden/chapter6b.cmd`:
chapter six's lines with one lever added, `add airbase who=0 4,33` on 606
and `add airbase who=1 12,33` on 608, and the window run to 1250 so that
both tanks run dry inside it. The staging and the premise's killer were
read and committed before the run (`b098e75`). It is the first capture on
this disk with an Airbase in it.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch6b \
    --map 14 --end-frame 1250 --log-window 605 1250 --timeout 3600 \
    --detail end:UNITS=3,GUYS=2,AMMO=5,DEATHS=1,LEADERS=2,BUILDS=7 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter6b.cmd
```

`success: true`, exit 0, 1,251 frames, `MAP_STYLE 14` and seed 12345 read
back, five settings files restored. One take, **315 s from launch to exit,
112 MB of dump and 10.8 MB of trace**. The lane lock was free. All ten
`INFO cmd` records returned 1 (`cmdsran.py`). 647 blocks: 1, 605..1250
and the `!quit` block 1251. Waited on with
`WAITRUN_RUNNER=unattended_capture.py waitrun.sh <viadriver log>`, which
exits 2 on a golden lane by design: `golden_capture.sh` prints no
`runqueue.sh` banner, so the runner's exit is the signal and the receipt
is the verdict.

**The same game as run168 to frame 611**: `rngcmp.py` finds 612 frames
identical, 0..611, and the first differing is 612. The two `add airbase`
lines spend no draw.

### The predictions, committed before the run

| check | predicted | observed |
| --- | --- | --- |
| the Airbases | a finished building each, on 607 and 609 | who=0 `2007` at (864, 6432) on 607, who=1 `2006` at (2400, 6432) on 609; `myhits` 2400, `build_masks` 4232 (`0x1000 \| 0x88`), both to 1249 |
| the seats | run168's (888, 7800) and (2424, 7800) | exactly those, on 611 and 616 |
| the base link | `inside_up` −1, no air order, `air_alt` 0 | `inside_up` −1 and `air_alt` 0 on every block; no air order on either aircraft |
| an empty base | no draw under `do_launch`/`attempt_launch`, `launch_frames` 0 | none, 0, `inside_down` −1 throughout; no disband |
| the target arm | open: whether an aircraft takes a building was not read | **both do, on their birth blocks** |
| fuel | `mana_burn` stops at 400 on 1010 and at 600 on 1215 | exactly so, and nothing else changes; no `DEATH` block in the capture |

### §10's six-b falsifiers

- **The first fires, on the target arm.** On its birth block each aircraft
  holds an `ATTACKORDER` on the **enemy Airbase** — the Fighter `0/6` on
  who=1's `2006`, the Bomber `1/6` on who=0's `2007` — and on the next a
  `MOVEORDER` to an attack point beside it. Both walk there on the ground
  at `air_alt` 0: the Fighter from (888, 7800) to (600, 7944) by 632,
  after which its stack empties on 633 and it stands to 1249; the Bomber
  from (2424, 7800) to (1992, 7704) by 700, after which its stack
  alternates `ATTACK` and `ATTACK`+`MOVE` every frame to 1249 and it never
  moves again. No blow lands: both Airbases read `damage` 0 throughout.
  The base-link half is dead as predicted.
- **The second, third and fourth do not fire.**

This crate parts at **632**, the Fighter's arrival at its attack point,
34 draws against 24: ten `Unit::find_attack_pos+0xea9 < Unit::fight+0xcb4`
of `0/6`'s where the original drops the attack. See `docs/GOLDEN.md` §10.

**4,297,129,978 bytes of dump and 19.9 MB of trace, 2 h 24 min** from
launch at 09:27 to archive at 11:51. The lane lock was stale (its holder,
pid 41127, was dead) and was taken over. The window ran ~14 blocks a
minute, 2.0 MB a block.
## run177 — chapter six-b's word, a packet at logger frame 632 (2026-09-24, item 680)

**What it is.** run175's game to 640 with a `RON_STATE_FRAME=632` packet:
the state after trace tick 631, with the Fighter `0/6` at its attack point
and its `fight` still to run. The dump detail is run175's end set over
[630, 636), with no `[Start Game]` set. `success: true`, exit 0,
`MAP_STYLE 14` and seed 12345 read back, five settings files restored.
**18 s launch to exit, 26 s in all**, with an 819,338,272-byte packet (173
ranges, 259 ms to copy). The lane lock was stale: its holder, pid 62906
(run175's), was dead.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-24-run177 \
    --map 14 --end-frame 640 --timeout 2400 --log-window 630 636 \
    --detail end:UNITS=3,GUYS=2,AMMO=5,DEATHS=1,LEADERS=2,BUILDS=7 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file <exp>/chapter6b-to640.cmd \
    --tracer-def RON_STATE_FRAME=632 \
    --tracer-def 'RON_STATE_PLAN="<plan>/plan.h"'
```

The plan is 597's (`~/ron-data/lab-experiments/2026-09-23-item-597/plan/`).
`<exp>` is `~/ron-data/lab-experiments/2026-09-24-item-680/`, and its
`chapter6b-to640.cmd` is `chapter6b.cmd` without its `700 bird` line. The
runner refuses a line past `!quit`, and the bird comes after the packet.
Two launches refused before the game started. The first refused on that
line. The second refused because the first had left the empty output
directory behind (`mkdir exist_ok=False`), which was removed.

**Why a packet, and what the disk could not answer.** Which branch
`fight` takes on tick 632 is a value the original computes, and no dump
prints it. The trace names only the draws, and the original's 632 spends
none under `fight`.

**Every check passed:**

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run53.log` | **0 differing**, 14,911 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 14 |
| the window, block for block | **2,006 blocks, 12894..14899, no gap** |
| a `GROUPDATA` on every window block | **2,006** |
| overlap with run174, nothing excluded | **6 in common (12894..12899), 0 differ** |
| the coverage window | a set on all 9 frames 14378..14386 |

**What it settled.** The word's block 14383 is who=1 placing a building
apart. The new `1/2025` stands at (42624, 19776) in the original and at
(39552, 17472) here, and the citizens `1/6` and `1/7` trade the build
and walk orders between them. Under it, the leader's `SITE` table parts
on 12976 and 13176 (`docs/AI.md` §66.3;
`run178_s_word_frame_is_widened_whole`).
| `frame_snapshot.py` against the plan | frame 632, trace 631, the roots unchanged |
| `rngcmp.py` against run175's `rontrace.log` | **0 differing**, 641 identical |
| `samegame.py` against run175 | 7 blocks in common (1, 630..635), **0 differ** |

**What it settled** (`docs/COMBAT.md` §62). On the packet,
`find_attack_pos` from `fight`'s own call answers 1 with this crate's ten
`+0xea9` draws, so the word's booked arm is not the original's. `fight`
itself returns 0 drawless through `find_new_target` at `fight+0xa1f`: a
captain's attack on a building re-searches every frame, and at twelve
tiles the Fighter's search finds nothing. `Game::do_frame` does not run on
this packet: it stops at its nineteenth instruction on a stack read
outside the plan's ranges (`0x7a84642c`). The calls were made one
function at a time. The packet stays outside git at
`~/ron-data/lab-captures/2026-09-24-run177/map-14`, and the oracle
scripts (`fap.py`, `fight.py`) at `<exp>`.
## run166 — East Indies' word 13640, a narrow window over it (2026-09-24, item 643)

**What it is.** run159's line plus run178's pool,
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,DEATHS=1,LEADERS=9,GROUPS=1`, over
`[13580, 13700)`, plus `rontrace.cfg` `cover=1` and `window=13636-13644`
over the word. `!quit` at 13710, through `viadriver.sh` with no human at
the menu.

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 643
```

**Why it was owed.** Item 642 moved East Indies' word 11747 → 13640
(`Region::go_here` through `leader_reg_cities`, `docs/TRANSPORT.md` §9.4).
run159 ends on block 11899 and run78 starts at 15700, and no
`gamelog-*eastindies*` holds block 13640 or 13641 (grepped before booking,
the pattern checked against run159's 11591). **Sized to the word, not to
the gap**: 60 blocks under the word and 59 above. It shares no block with
run159, so its only same-game evidence is the draw stream against run54;
a backward window is owed only if 13580 already parts on a record the
word's cause could come from.

**Taken whole, without the pool.** 265 MB of dump and 20.9 MB of trace,
120 blocks 13580..13699, about 11 minutes from launch (12:19) to archive
(12:31), with no human at the menu. The lane lock was stale (pid 45812,
dead). Five of six checks are green: 13,711 frames identical to run54,
`MAP_STYLE 18`, the window whole, a `MAKEOBJECT` on every block, and
coverage on all 9 frames 13636..13644.

**The pool check failed: no `GROUPDATA` at all.** That is run30's trap,
repeated. `DEATHS=1` under `[End Frame]` leaves the logger's type at
`WORLD`, and the pool's lines then fail `check_accept` silently
(`docs/ORACLE.md`). run178's line had dropped `DEATHS` for that reason,
and this stanza kept it while adding `GROUPS=1`.
The check stays in the stanza, recorded as failed, and the widening reads
no pool.

**What it answered.** The Galley `1/32` (the Trireme, upgraded inside the
window on both sides) enters `CHAR_DEFAULT` on block 13638 with `end_time`
3 here and 20 there, and wraps again on 13640, one draw more than the
original spends. No record parts on 13580 that the cause could come from,
so no backward window (run182) is owed.
`run166_s_word_frame_is_widened_whole`.

## run180 — chapter nine, the move line, the first issuer chapter (2026-09-24, item 676)

`docs/GOLDEN.md` §17, from `tools/gamelog/golden/chapter9.cmd`: `!ai
off`, a Chariot `0/6` on 610 and a Hoplite squad `0/7`–`0/9` on 612, and
two **player orders** through the original's `CommandManager::issue_move_to`,
called from `rontrace.dll` by the new `@move` line — the chariot on 620,
the squad by its captain on 640. The staging, the premise's killer and the
falsifiers were committed before the run (`909df26`).

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch9 \
    --map 14 --end-frame 1100 --log-window 605 1100 --timeout 3600 \
    --detail end:UNITS=3,GUYS=2,DEATHS=1,LEADERS=2 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter9.cmd
```

`success: true`, exit 0, 1,101 frames, `MAP_STYLE 14` and seed 12345 read
back, five settings files restored. One take, **209 s launch to exit, 66 MB
of dump and 10.3 MB of trace**. The lane lock was stale when it launched:
its holder, pid 54101 (att-678's run178), had exited and its queue with it.
497 blocks: 1, 605..1100 and the `!quit` block 1101. The five `INFO cmd`
records returned 1 (`cmdsran.py`). Waited on with
`WAITRUN_RUNNER=unattended_capture.py waitrun.sh <viadriver log>`, which
exits 2 on a golden lane by design; the receipt is the verdict.

**The same game as run175 to frame 610**: `rngcmp.py` finds 0..610
identical and 611, the chariot's birth, first differing.

### The issuer's own records

| trace frame | record | read |
| --- | --- | --- |
| 620 | `INFO 18` | `0/6`, uid 13, at (3192, 7032) — this crate's seat |
| 620 | `INFO 17` | line 4, refusal 0, package 10 → 37 bytes, one object |
| 640 | `INFO 18` | `0/7`, uid 14, at (3192, 10104) — this crate's seat |
| 640 | `INFO 17` | line 5, refusal 0, package 10 → 37 bytes, one object |

The package already holds 10 bytes when the issuer runs: the turn's own
`camera` command. Each call adds the emulator's 27.

### The processed command, and §17's falsifiers

Between blocks 621 and 622 the dump prints `process_group, new 0 1 621`,
`process_move_to 12672 7296 2 0 0 1 0` and `process_move_to_2 -1 -1`
(`who`, `num`, frame; `to`, `queued`, `set_angle`, `angle`, `orders`,
`disembark`; `form`, `width`), and the same for the squad between 641 and
642. None of the four falsifiers fires.

| check | predicted | observed |
| --- | --- | --- |
| the issue | appended and processed on the next frame | as predicted, on 621 and 641 |
| the chariot, block 622 | one `MOVEORDER`, type 1, action bit, a plan of three or more | type 1, `flags 5`, an **11-entry** plan, `group 1`, `play 0` |
| the squad, block 642 | three type-19 orders, one id, one leader | `GroupMoveOrder` ×3, id **641000**, `oxx 7`, `form_id` 0–2, a 7-entry plan on the leader; `group 0`, `play 0` on each |
| the squad's arrival | this crate: blocks 938–941 on its slots | stacks empty on **938, 939, 941**, on (5112, 16488), (4992, 16512), (4872, 16536) — this crate's blocks and points |
| the chariot's arrival | this crate: block 990 | stack empty on **1040**, on (12672, 7296) |

**Where the chariot parts from this crate's reading.** The original's plan
runs **straight along row 9**, every waypoint at `y 7320`, x 4248 to 11160,
through the SANDY cells of region 65 that this crate's plan bends north
round (§17's "ground"), and it arrives fifty frames later than this crate's
longer route. See `docs/GOLDEN.md` §17.

## run189 — Great Lakes' word 14529, a packet at logger frame 14529 (2026-09-24, item 695)

**What it is.** The long game on the click-free lane to 14535 with a
`RON_STATE_FRAME=14529` packet — the state after trace tick 14528, with
tick 14529's `Unit::do_move` for the caravan `1/23` still ahead of it —
and a `WORLD=6` dump over [14528, 14531) beside run178's line, so the
tile masks come through the parser the harness already has.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-24-run189 \
    --map 14 --end-frame 14535 --timeout 3600 --log-window 14528 14531 \
    --detail end:MISC,WORLD=6,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1 \
    --tracer-def RON_STATE_FRAME=14529 \
    --tracer-def 'RON_STATE_PLAN="<plan>/plan.h"'
```

The plan is the lab's Great Lakes plan, reused as run144 reused it
(`~/ron-data/lab-experiments/2026-09-23-item-597/plan/`).

**Why a packet, and what the disk could not answer.** On tick 14529 the
original's `1/23` takes the waypoint (42272, 19424), flagged road
(`0x20`), and every entry of its path loses the flag: that is
`Unit::do_move@005f7b30:437`–`478`, which verifies the route only when
the tile is **not road** and is built on or blocked. Then 3,206 road
draws. Here the tile, (220, 101), is `0x190` on the same tick: placed on
by the Barracks `1/2025`, and **still road**. No capture on disk prints a
tile mask on this map past run72 — run178's line has no `WORLD` — so
whether the original's tile lost its road, and what took it, is a value
only the original holds.

`success: true`, exit 0, `MAP_STYLE 14` and seed 12345 read back, five
settings files restored. **85 s launch to exit, 122 s in all**, with an
842,030,040-byte packet and 16 MB of dump. The lane lock was stale: pid
83243 (att-693's run185) was dead. The dump is archived as
`gamelog-run189-greatlakes-roadword.txt`, and the trace as
`rontrace-run189.log`. The packet stays at
`~/ron-data/lab-captures/2026-09-24-run189/map-14`.

| check | result |
|---|---|
| `rngcmp.py` against `rontrace-run53.log` | **0 differing**, 14,536 identical |
| the window | `WORLD` on blocks 14528, 14529 and 14530, 57,600 tile masks each |
| the packet's `curscan` | cell (3, 15) = 903 = 14,529 × 7 mod 3,600 |

**What it settled** (`docs/ROADS.md` §10, `docs/AI.md` §68). (220, 101)
is `0x180` there: placed on, and not road. On block 14529 the surface
parted on 17 tiles and no others, the trade road (220, 98)..(216, 115).
On the packet those cells hold no road elements, and (220, 97)'s claims
N alone. That is `Roads::scan_and_kill_stray_roads`' erosion, and
`rondata::diff::world::tests::run189_s_world_has_the_original_s_roads_at_14529`
now pins the block with no surface residue.
## run184 — chapter ten, the patrol line (2026-09-24, item 693)

`docs/GOLDEN.md` §18, from `tools/gamelog/golden/chapter10.cmd`: `!ai
off`, chapter nine's Chariot `0/6` on 610, a Hoplite squad `0/7`–`0/9` on
612 east of the sand, and two **player patrols** through the original's
`CommandManager::issue_patrol`, called from `rontrace.dll` by the new
`@patrol` line — the chariot on 620, the squad by its captain on 640. The
staging, the premise's killer and the falsifiers were committed before the
run (`a4eed08`).

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch10 \
    --map 14 --end-frame 1250 --log-window 605 1250 --timeout 3600 \
    --detail end:UNITS=3,GUYS=2,DEATHS=1,LEADERS=2 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter10.cmd
```

`success: true`, exit 0, 1,251 frames, `MAP_STYLE 14` and seed 12345 read
back, five settings files restored. One take, **243 s launch to exit, 85 MB
of dump and 10.7 MB of trace**. The lane lock was stale when it launched:
its holder, pid 78882 (att-643's `longtrace.sh`), had exited. 647 blocks:
1, 605..1250 and the `!quit` block 1251. The five `INFO cmd` records
returned 1 (`cmdsran.py`). Waited on with `WAITRUN_RUNNER=unattended_capture.py
waitrun.sh <viadriver log>`, which reports no banner on a golden lane by
design; the receipt is the verdict. **The same game as run180 to frame
693**: `rngcmp.py` finds 0..693 identical — the hoplites' new seat draws
nothing — and 694, run180's chariot re-plan, first differing.

### The issuer's own records

| trace frame | record | read |
| --- | --- | --- |
| 620 | `INFO 18` | `0/6`, uid 13, at (3192, 7032) |
| 620 | `INFO 17` | line 4, refusal 0, package 10 → 25 bytes, one object |
| 640 | `INFO 18` | `0/7`, uid 14, at (10872, 7800) |
| 640 | `INFO 17` | line 5, refusal 0, package 10 → 25 bytes, one object |

Each call adds the emulator's 15 bytes to the turn's 10-byte `camera`.

### The processed command, and §18's falsifiers

Between blocks 621 and 622 the dump prints `process_group, new 0 1 621` and
`process_patrol 3456 11136 2` (`to_x`, `to_y`, `queued`), and the same for
the squad between 641 and 642. **None of the four falsifiers fires.**

| check | predicted | observed |
| --- | --- | --- |
| the issue | appended and processed on the next frame | as predicted, on 621 and 641 |
| the chariot's patrol, block 622 | one `GroupPatrolOrder`, points (3192, 7032) and (3480, 11160) | as predicted: flags 4, `waypoint 1`, id **621100**, `oxx 6`, `form_id 0` |
| the chariot's leg, block 622 | an `ATTACKTOORDER` above the patrol | type 2, `flags 1` (no action bit: `do_patrol` passes action 0), to (3480, 11160) |
| the squad's patrols, block 642 | three, one id, leader `0/7` | three, id **641000**, `oxx 7`, points (10872, 7800) and (11160, 11928), `waypoint 1` — and **`form_id 0` on all three**: `redo_patrol_order` has already rebuilt them with the leader's index |
| the squad's leg, block 642 | three `GroupAttackToOrder`s, one id | type 21 ×3, id **641001**, `form_id` 0–2, slots x 11160 / 11016 / 11304 on y 11928 |
| the turns | chariot near 770, 915, 1060, 1205; squad near 820, 1000, 1180 | chariot's leg empties on **761, 903, 1043, 1185** exactly on its points, and the next block holds the leg to the other point with `waypoint` stepped; the captain's on **812, 985, 1155**, new leg ids **812002, 985003, 1155004** |

**Two things the reading did not say, both visible in the table.** A turn
spends a block: on the block a leg empties, the patrol is alone at the
head, and `do_patrol` issues the next leg on the tick after. And the
squad's `GroupAttackToOrder`s degrade to plain `ATTACKTOORDER`s to their
slots ~14 blocks short of the point (798, 970, 1141: `ungroup_move_order`).
When the captain turns, `0/8` has not reached its slot yet: the halt drops
its leg and the new group leg replaces it.

## run185 — chapter ten under `cover=1`, hung before frame 0 (2026-09-24, item 693)

run184's script re-run with `--cover cover=1`, a log window of (605, 606)
and no `end:` detail, so that the census's entered column could see the
patrol's functions (`docs/CENSUS.md`; parked 692 asks whether an issuer
chapter should count). Launched at 13:17 through `golden_capture.sh`.
**It never reached frame 0.** `rontrace.log` holds 96 KB: the five hooks,
the eight `callwin` proxies, `INFO cover` (48,173 stubs built), `armed`,
`attach`, 2,984 `HIT` records of startup, and no `FRAME` record. Nothing
more was written, and the runner's 3,600 s timeout killed the game at
14:17, which released the lane lock. `waitrun.sh` exited 2, with no
banner. So **`cover=1` on the click-free golden lane**, with its `callwin`
proxies, is not a combination any capture had run before, and it hangs at
startup. run906's `cover=1` ran on the older lane with no proxies. Not
retried: the census's entered column is the Loop's question (692), and an
hour of the single lane was already spent.
## run202 — Great Lakes' word 15383, past run196's last block (2026-09-24, item 711)

**What it is.** run196's line, unchanged,
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, over
`[15227, 15441)`, plus `rontrace.cfg` `cover=1` and `window=15379-15387`
over the word. `!quit` at 15451, through `viadriver.sh` with no human at
the menu.

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 711
```

**Why it was owed.** Item 711 moved Great Lakes' word 15175 → 15383:
a unit's own mirror, `unit_masks & 2`, which `Unit::do_guard` reads off
its target (`docs/GROUPS.md` §25). run196 ends on block 15232, so the
word's block 15384 is on no disk. On 15383 ours spends one
`Guy::set_anim+0x97a < Unit::do_idle+0x7d` where the original spends
`Guy::set_anim+0x97a < Guy::inc_time+0x271`. **Sized to the word, not to
the gap** (Loop 687): six blocks shared with run196, the 151 up to the
word, its block, and 56 of runway above it. The readings and their kills
are in the stanza.

**Taken whole.** 475 MB of dump and 20.6 MB of trace, 214 blocks
15227..15440. About 20 minutes from launch (18:15) to archive (18:35),
with no human at the menu. The lane lock was stale (pid 26953, run196's,
dead). Waited on with `waitrun.sh`, exit 0.

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run53.log` | **0 differing**, 15,452 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 14 |
| the window, block for block | **214 blocks, 15227..15440, no gap** |
| a `GROUPDATA` on every window block | **214** |
| overlap with run196, nothing excluded | **6 in common (15227..15232), 0 differ** |
| the coverage window | a set on all 9 frames 15379..15387 |

**What it settled** (`run202_s_word_frame_is_widened_whole`).
- **R1 holds**: the checks above.
- **The word is a citizen's order.** On block 15383 the original's
  citizen `1/70` holds one order and ours holds none. On 15384 theirs is
  kind 3 and ours stands idle, which is the extra `Unit::do_idle` stand.
  Building `1/2022`'s gather list holds `1/70` here and `1/73` there. So
  **R3 holds**.
- **R2 is false, though its killer did not fire.** The first row to part
  is the escort's: The Despot `1/79`'s move `facing`, 1 here against 0
  there, on 15351, a value row that spends no draw. But the idler is
  `1/70`.
- The floor is run196's walk (422, nothing on run196's own blocks since
  item 711), then 11 keys on run202's own blocks up to the word, and 840
  to the window's end.

## run196 — Great Lakes' word 15175, past run192's last block (2026-09-24, item 706)

**What it is.** run192's line, unchanged,
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, over
`[15034, 15233)`, plus `rontrace.cfg` `cover=1` and `window=15171-15179`
over the word. `!quit` at 15243, through `viadriver.sh` with no human at
the menu.

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 706
```

**Why it was owed.** Item 706 moved Great Lakes' word 14982 → 15175:
the nation graft table and the Senate's government patriot
(`docs/TECH.md` §"The graft table", §"The government patriot"). run192
ends on block 15039, so the word's block 15176 is on no disk. On 15175
ours spends one `Guy::set_anim+0x97a < Unit::do_guard+0x7f4` more than
the original. **Sized to the word, not to the gap** (Loop 687): six
blocks shared with run192, the 136 up to the word, its block, and 56 of
runway above it. The readings and their kills are in the stanza.

**Taken whole.** 442 MB of dump and 20.5 MB of trace, 199 blocks
15034..15232. It took about 17 minutes from launch (17:03) to archive
(17:21), with no human at the menu. The lane lock was stale (pid 76328,
run192's, dead). Waited on with `waitrun.sh`, exit 0.

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run53.log` | **0 differing**, 15,244 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 14 |
| the window, block for block | **199 blocks, 15034..15232, no gap** |
| a `GROUPDATA` on every window block | **199** |
| overlap with run192, nothing excluded | **6 in common (15034..15039), 0 differ** |
| the coverage window | a set on all 9 frames 15171..15179 |

**What it settled** (`run196_s_word_frame_is_widened_whole`).
- **R1 holds**: the checks above.
- **R2 is killed.** The first parting on run196's own blocks is not the
  Despot. On 15095 the three free Longbowmen `1/76`–`1/78` take guard
  orders, and the posts are handed out the other way round: `1/77` guards
  (42696, 22392) here and (42984, 22344) there, and `1/78` the reverse.
  Their steps part from there. On 15151–15152 their order kinds part, 12
  here against 2 there, and `1/76` follows on 15180.
- **R3 is killed with it.** The Despot `1/79` parts nowhere before 15208.
- Nothing first-parts on the word's block, 15176. Its extra `do_guard`
  draw is the Longbowmen's.
- The floor is run192's 422, then 89 rows on run196's own blocks up to
  the word, all of them the three archers, and 732 keys to the window's end.

## run192 — Great Lakes' word 14982, past run178's last block (2026-09-24, item 698)

**What it is.** run178's line, unchanged,
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, over
`[14894, 15040)`, plus `rontrace.cfg` `cover=1` and `window=14978-14986`
over the word. `!quit` at 15050, through `viadriver.sh` with no human at
the menu.

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 698
```

**Why it was owed.** Item 698 moved Great Lakes' word 14650 → 14982 (the
suspended search's `GATHER` park, `docs/COLLISION.md` §15). run178 ends on
block 14899, and no `gamelog-*greatlakes*` holds block 14982 or 14983
(grepped before booking, the pattern checked against run163's 12180). On
14982 the original spends three `Guy::init_real+0x52` before the frame's
wraps, a three-figure birth, and this crate spends none. **Sized to the
word, not to the gap** (Loop 687): the 83 blocks up to it, six shared with
run178 so the widening walks one chain, and 56 of runway above it.

**Taken whole.** 326 MB of dump and 20.3 MB of trace, 146 blocks
14894..15039. It took about 14 minutes from launch (15:33) to archive
(15:47), with no human at the menu. The lane lock was stale (pid 66547,
dead). Waited on with `waitrun.sh`, exit 0.

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run53.log` | **0 differing**, 15,051 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 14 |
| the window, block for block | **146 blocks, 14894..15039, no gap** |
| a `GROUPDATA` on every window block | **146** |
| overlap with run178, nothing excluded | **6 in common (14894..14899), 0 differ** |
| the coverage window | a set on all 9 frames 14978..14986 |

**What it settled.** The word's block 14983 holds a birth, `1/79`. It
appears in the dump alone: three figures of guy type 352 at (44184,
24552), `myhits` 109, counted by who=1's `active` and `num_units[302]`.
Nothing parts on 14900..14945. The first parting under the word is on
14946. A who=1 Barracks' free train, `1/76`–`1/78`, is born on both sides,
as type 120 here and type 127 there, with `myhits` 70 against 88. No make
list or queue parts under the word
(`run192_s_word_frame_is_widened_whole`).

## run190 — chapter eleven, the guard line (2026-09-24, item 696)

`docs/GOLDEN.md` §19, `tools/gamelog/golden/chapter11.cmd`. The cast:
- chapter nine's Chariot `0/6` on 610;
- a Supply Wagon `0/7` on 612, two cells south;
- a Hoplite squad `0/8`–`0/10` on 614, on cell (9, 38) near who=0's
  `0/2001`.

Two **player guards** through `CommandManager::issue_guard`, called from
`rontrace.dll` by the new `@guard` line: the chariot on the wagon on 620,
and the squad on the building on 640. Then chapter nine's `@move` walks
the wagon on 720, and `1000 add chariot who=1 17,70` puts an enemy in
sight. The staging, the premise's killer and the falsifiers were
committed before the run (`c7d8f5c`).

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch11 \
    --map 14 --end-frame 1250 --log-window 605 1250 --timeout 3600 \
    --detail end:UNITS=3,GUYS=2,DEATHS=1,LEADERS=2 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter11.cmd
```

**The capture.** One take: **254 s launch to exit, 85 MB of dump and
10.8 MB of trace**.
- `success: true`, exit 0, 1,251 frames.
- `MAP_STYLE 14` and seed 12345 read back, and five settings files
  restored.
- The seven `INFO cmd` records returned 1 (`cmdsran.py`).

**The lane.** The lock was stale when it launched: its holder, pid 69380,
had exited, and no game or driver was running. `waitrun.sh` exits 2 at
once on a golden lane, because no `runqueue.sh` runs there. The wait was
on RonDriver's own exit, and the receipt is the verdict.

**The same game as run184 to frame 614**: `rngcmp.py` finds 0..614
identical, and 615, the tick after the squad's birth, first differing.

### The issuer's own records

| trace frame | record | read |
| --- | --- | --- |
| 620 | `INFO 18` | `0/6`, uid 13, at (3192, 7032) |
| 620 | `INFO 17` | line 5, refusal 0, package 10 → 28 bytes, one object |
| 640 | `INFO 18` | `0/8`, uid 15, at (7032, 29304) |
| 640 | `INFO 17` | line 6, refusal 0, package 10 → 28 bytes, one object |
| 720 | `INFO 18` | `0/7`, uid 14, at (3192, 8568) |
| 720 | `INFO 17` | line 7, refusal 0, package 10 → 37 bytes, one object |

Each guard adds the emulator's 18 bytes to the turn's 10-byte `camera`,
and the move adds chapter nine's 27.

### The processed commands, and §19's falsifiers

Between blocks 621 and 622 the dump prints `process_group, new 0 1 621`
and `process_guard 621`. The `Log::say` string carries the frame alone;
the `ox`/`whom`/`queued` line is the sync logger's, not the dump's. The
same appears between 641 and 642, and `process_move_to 3456 11904 2 0 0 1
0` between 721 and 722. **None of the five falsifiers fires.**

| check | predicted | observed |
| --- | --- | --- |
| the issue | appended, processed on the next frame | as predicted, on 621, 641 and 721 |
| the guard, block 622 | one `GUARDORDER` on `0/7`, the action bit, under a transit leg | type 12, `flags 4`, `ox 7 whom 0 uid 14`, **`dx 0 dy 372`**, post (3528, 8760), under an `ATTACKTOORDER` (`flags 1`, `timer 59`) to the post. This crate's value to the digit |
| the post | taken by ~660 | on (3528, 8760) with `idle 24` on 700, this crate's `idle` too |
| the moving charge | the post re-read as the wagon walks | the wagon's `MOVEORDER` from 722. Its stack empties on **857** on (3456, 11904); this crate said ~915. The guard's post steps (3288, 8952) → (3480, 9624) → … → (3480, 12216) on 856, one cell short, then (3480, 12264) on 890, where it stands from 900 |
| the engagement | an `ATTACKORDER` on `1/6` above the `GUARD` by 1027 | on **1011** exactly, the phase tick 1010. `1/6` holds its own `ATTACK` on `0/6` from 1001 |
| the building half | no order on `0/8`–`0/10` | none, on any of 635 blocks, and the three never move |

**What the reading did not say.** The fight is the chapter's surprise.
- **1021**: `1/6` takes an army `ATTACKTOORDER` and walks away.
- **1037**: the guard's `ATTACK` goes, and the `GUARD` stands alone.
- **~1050**: `1/6` comes back and shoots the guard from about 1,640
  units.
- **The guard never re-engages.** Its `idle` counts on every block, and
  its sixteen-frame search finds nothing.
- **The damage**: the guard landed one hit (`1/6` damage 21 on 1044) and
  took 20 on 1032, 1091 and 1115. It is gone from block 1141, and its
  `GUARD` goes with it.
- The wagon stands unguarded to 1249.

## run191 — chapter eleven's word, three bracket takes (2026-09-24, item 696)

**What it is.** run190's script cut at 730 (its `1000` line dropped: the
runner refuses a line past `!quit`), kept at
`~/ron-data/lab-experiments/2026-09-24-item-696/chapter11-to730.cmd`.
There are three takes, each a narrow `callwin` and a dump window, and no
`[Start Game]` set. Each is **the same game as run190**: `rngcmp.py` finds
731 of 731 frames identical. Each took 16–17 s launch to exit,
`success: true`, and each took the lane on a stale lock.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch11p \
    --map 14 --end-frame 730 --log-window 719 726 --timeout 2400 \
    --detail end:UNITS=3,GUYS=2,DEATHS=1,LEADERS=2 --detail misc:COMMANDMANAGER=1 \
    --callwin 719 723 --tracer-def RON_GUARD_PROBE \
    --cmd-file ~/ron-data/lab-experiments/2026-09-24-item-696/chapter11-to730.cmd
```

| take | output | `callwin` | tracer | what it settled |
| --- | --- | --- | --- | --- |
| 1 | `~/ron-golden/ch11p` | 719–723 | `RON_GUARD_PROBE` (five brackets) | tick 721's `set_new_location(0/6)`, the guard's step off its post, is nested in the **wagon's** `detect_unit_collision(3200, 8577, quick 0, boats 1)`. The guard's own `do_guard` calls nothing proxied (on-post) |
| 2 | `~/ron-golden/ch11p2` | 724–727 | the same, and `detect_boat_collision` as a sixth | the wagon's push answers **0** on ticks 725–727 (it refuses the walking guard), and its land scan then answers **0** too |
| 3 | `~/ron-golden/ch11p3` | 726 | `RON_COLLIDE_PROBE` | the wagon's scan on 726: `collide_here` hits (69, 183), `will_be_corner 5`, `is_here(0/6) = 1`, and **no `is_corner`**: the guard is a soft collider |

**Why a capture, and what the disk could not answer.** run190's dump
shows the guard moving on 722 with no order change and no draw. Nothing
it prints names the function that moved it, and the trace's proxy records
carry no caller. The brackets' nesting is the caller.

`RON_GUARD_PROBE` is new in `tools/trace/tracer.c`, sites 8–13, and each
`this` is named by an `INFO 15` as `RON_COLLIDE_PROBE` names them. Every
prologue and `ret` immediate was read off the listing. It refuses a build
beside the other probes that claim 8–13.

## run204 — chapter twelve, the follow line (2026-09-24, item 714)

`docs/GOLDEN.md` §20, `tools/gamelog/golden/chapter12.cmd`. The cast:
- pair A: chapter nine's Chariot `0/6` on 610, and a Supply Wagon `0/7`
  on 612, two cells south;
- pair B: a Hoplite squad `0/8`–`0/10` on 614, on cell (15, 14), and a
  Chariot `0/11` on 616, one cell south.

Two **player follows** through `CommandManager::issue_follow`, called from
`rontrace.dll` by the new `@follow` line: the chariot on the wagon on 620,
and the squad on the chariot on 640. Then chapter nine's `@move` walks each
leader and turns it: the wagon south on 700 and west on 960, the chariot
south on 720 and east on 880. The staging, the premise's killer and the
falsifiers were committed before the run (`26ae843`).

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch12 \
    --map 14 --end-frame 1150 --log-window 605 1150 --timeout 3600 \
    --detail end:UNITS=3,GUYS=2,DEATHS=1,LEADERS=2 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter12.cmd
```

**The capture.** One take, `cover=0`: **229 s launch to exit, 74 MB of
dump and 10.5 MB of trace**.
- `success: true`, exit 0, 1,151 frames.
- `MAP_STYLE 14` and seed 12345 read back, and five settings files
  restored.
- The seven `INFO cmd` records returned 1 (`cmdsran.py`).

**The lane.** The lock was stale when it launched: its holder, pid 33875
(att-711's `longtrace.sh`), had exited. `waitrun.sh` ran with
`WAITRUN_RUNNER=unattended_capture.py`, and exited 2 once the runner was
gone, since a golden lane prints no queue banner; the receipt is the
verdict.

**The same game as run190 to frame 616**: `rngcmp.py` finds 0..616
identical, and 617, the tick after pair B's chariot is born, first
differing.

### The issuer's own records

| trace frame | record | read |
| --- | --- | --- |
| 620 | `INFO 18` | `0/6`, uid 13, at (3192, 7032) |
| 620 | `INFO 17` | line 6, refusal 0, package 10 → 28 bytes, one object |
| 640 | `INFO 18` | `0/8`, uid 15, at (11640, 10872) |
| 640 | `INFO 17` | line 7, refusal 0, package 10 → 28 bytes, one object |
| 700 | `INFO 18` | `0/7`, uid 14, at (3192, 8568) |
| 700 | `INFO 17` | line 8, refusal 0, package 10 → 37 bytes |
| 720 | `INFO 18` | `0/11`, uid 18, at (11640, 11640) |
| 720 | `INFO 17` | line 9, refusal 0, package 10 → 37 bytes |
| 880 | `INFO 18` | `0/11`, uid 18, at (11904, 14976) |
| 880 | `INFO 17` | line 10, refusal 0, package 10 → 35 bytes: the 3-byte reuse |
| 960 | `INFO 18` | `0/7`, uid 14, at (3456, 13440) |
| 960 | `INFO 17` | line 11, refusal 0, package 10 → 37 bytes |

Each follow adds the emulator's 18 bytes to the turn's 10-byte `camera`.

### The processed commands, and §20's falsifiers

Between blocks 621 and 622 the dump prints `process_group, new 0 1 621`
and `process_follow 621`, and the same for 641. The moves print
`process_move_to` between F+1 and F+2; the 880 one follows
`process_group, repeat 0 0 881`. **None of the six falsifiers fires; the
fifth could not for pair B**, whose chariot turned before its sixty
blocks were up.

| check | predicted | observed |
| --- | --- | --- |
| the issue | appended, processed on the next frame | as predicted, on 621, 641, 701, 721, 881 and 961 |
| the orders, 622 and 642 | one `FOLLOWORDER` a unit on its leader, the action bit | `0/6`: `flags 4`, `ox 7 whom 0 uid 14`. `0/8`–`0/10`: `flags 4`, `ox 11 whom 0 uid 18`, three orders |
| standing | no leg before the leader walks | `0/6` stands 622–710 at d 1,536; the squad 642–721 at d 625–781 |
| trailing, pair A | a leg by 720 | first leg on **711** at d 1,737, the threshold 1,728 crossed. Then a leg every 6–9 blocks, each **one tile (192)** long, to the point 1,536 behind the wagon; every leg is issued at d 1,724–1,748 |
| trailing, pair B | a leg by 727 | `0/8` and `0/9` on **722**, `0/10` on 724, at d 784–797: the moving threshold 653, the doubling. Without it none would come before d 1,575 |
| rest, pair A | 60 blocks after the wagon's stack empties (901): FOLLOW alone, within 1,728 | on 961: FOLLOW alone, d 1,612 |
| rest, pair B | the same after 835 | not reachable: the chariot turned on 880 and `0/9` still held a leg then |
| the FOLLOW kept | on every block to 1149 | every stack of the four followers is the FOLLOW alone or the FOLLOW under one `MOVEORDER` leg, on every block |
| the turn | legs again after 962 (pair A) and 882 (pair B); within threshold on 1149 | `0/6` from 990, the squad from 882; on 1149, `0/6` stands at d 1,599 from the wagon and the hoplites at 1,416, 1,272 and 936 from `0/11`, each FOLLOW alone |

**The legs** are `MOVEORDER`s with `flags 1`: no action bit, and the
pathed bit `do_move` sets on the same frame.

**What the reading did not say.**
- **A faster follower hops.** The chariot's legs are one tile each, since
  the point 1,536 behind the wagon is only ~200 ahead of it when it
  crosses 1,728. Pair A's first walk is 24 legs.
- **The hoplites' legs are re-aimed.** While `0/11` stands, from 835 to
  880, `0/9`'s leg changes its destination on 861, 863, 865, 869 and 878,
  each a step or two off the last, at d 700–940. The standing threshold
  is 1,575, so no fresh `do_follow` leg is due. Which step re-aims it is
  not named.

## run208 — chapter thirteen, the garrison line (2026-09-24, item 718)

`docs/GOLDEN.md` §21, `tools/gamelog/golden/chapter13.cmd`. The cast:
- a Barracks `0/2007` (uid 13) on 606, centred on (2688, 14208);
- a Chariot `0/6` on 610, on cell (5, 15);
- a Hoplite squad `0/7`–`0/9` on 614, on cell (2, 22).

Two **player garrisons** through `CommandManager::issue_garrison`, called
from `rontrace.dll` by the new `@garrison` line: the chariot on 620 and the
squad on 640, both into the Barracks. Then the **Eject** through
`CommandManager::issue_eject_all` by the new `@eject` line on 900. The
staging, the premise's killer and the falsifiers were committed before
the run (`938a17a`).

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch13 \
    --map 14 --end-frame 1000 --log-window 605 1000 --timeout 3600 \
    --detail end:UNITS=3,GUYS=2,BUILDS=7,DEATHS=1,LEADERS=2 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter13.cmd
```

**The capture.** One take, `cover=0`: **184 s launch to exit, 72 MB of
dump and 10.1 MB of trace**.
- `success: true`, exit 0, 1,001 frames.
- `MAP_STYLE 14` and seed 12345 read back, and five settings files
  restored.
- The six `INFO cmd` records returned 1 (`cmdsran.py`).

**The lane.** The lock was stale: its holder, pid 9574 (run204's
unattended runner), had exited. `waitrun.sh` ran with
`WAITRUN_RUNNER=unattended_capture.py` and exited 2 once the runner was
gone, as on every golden lane; the receipt is the verdict.

**The same game as run204 to frame 612**: `rngcmp.py` finds 613 the first
differing frame. The Barracks on 606 takes no draw.

### The issuer's own records

| trace frame | record | read |
| --- | --- | --- |
| 620 | `INFO 18` | `0/6`, uid 14, at (3960, 11640) |
| 620 | `INFO 17` | line 5, refusal 0, package 10 → 28 bytes, one object |
| 640 | `INFO 18` | `0/7`, uid 15, at (2040, 17400) |
| 640 | `INFO 17` | line 6, refusal 0, package 10 → 28 bytes, one object |
| 900 | `INFO 18` | `0/2007`, uid 13, at (2688, 14208) |
| 900 | `INFO 17` | line 7, refusal 0, package 10 → 32 bytes, one object |

Each garrison adds the emulator's 18 bytes to the turn's 10-byte `camera`,
and the eject its 22.

### The processed commands, and §21's falsifiers

The dump prints `process_group, new 0 1 621` and `process_garrison 2007 0
2 621` between blocks 621 and 622, the same for 641, and `process_group,
new 0 1 901` and `process_eject_all 901` between 901 and 902. **None of
the six falsifiers fires.**

| check | predicted | observed |
| --- | --- | --- |
| the issue | appended, processed on the next frame | as predicted, on 621, 641 and 901 |
| the orders, 622 and 642 | one `GARRISONORDER` a unit on the Barracks, the action bit, `search 0` | four, each `flags 4, ox 2007 whom 0 uid 13, search 0`, at the bottom of the stack |
| the walk | a `MOVEORDER` leg above it, no action bit, 432–480 from (2688, 14208) on the unit's side | `0/6` to (2904, 13800), 461; `0/7` (2616, 14616), `0/8` (2808, 14616) and `0/9` (2280, 14328), each its own leg; `flags 0` on the block laid, `flags 1` from the next |
| the door | the chariot in near 708, the squad near 766, the squad whole | `0/6` in on **699**; `0/7`–`0/9` in together on **761**; `inside_up` 2007 ← 6 ← 7 ← 8 ← 9, the Barracks' `inside_down` 6; every stack empty inside |
| the eject | `0/6` out on 902 at (2712, 14904), `inside_down` 7; the squad out on 903, `inside_down` −1 | exactly so. The squad: `0/7` at (2424, 14808), `0/8` (2424, 14952), `0/9` (2424, 14616) |
| no orders after | empty stacks 902–999 | empty on every block |

**`build_masks`** reads 4096 before the eject, **20480** (`| 0x4000`) on
902 and 903, and 4096 again from 904: `process_ejection` clears the bit
on the frame after the last squad, when it finds the chain empty.

**What the reading did not say: every walk ended before its leg's point.**
- The chariot's last step is on 698, to (2990, 13689), about 140 short of
  (2904, 13800). It is inside on 699 without moving again.
- The squad's captain steps on 761 to (2612, 14651), about 35 short of
  (2616, 14616), and is inside the same block with both members. They had
  not moved on 761.

`do_garrison`'s door is `Object::adjacent_to`, and the reading put the
test only on a frame whose head is the GARRISON. Something cuts the leg
short while it is still the head. A hypothesis, not a finding: `Unit::
work`'s every-16-frames step, phased by `o` (`docs/ORDERS.md` §2.3 step
5). `(698 + 6)` and `(761 + 7)` are both multiples of 16. The widening
and this crate's own walk will say.

## run210 — chapter fourteen, the formation line (2026-09-25, item 723)

`docs/GOLDEN.md` §22, `tools/gamelog/golden/chapter14.cmd`. The cast, on
chapter thirteen's ground, is three squads:
- Hoplites `0/6`–`0/8` on 610;
- Hoplites `0/9`–`0/11` on 612;
- Slingers `0/12`–`0/14` on 614.

Two **player formation commands** go through `CommandManager::issue_form`,
called from `rontrace.dll` by the new `@form` line:
- Envelop (2) on 620, the group standing;
- Line (0) on 740, the group walking.

Between them, chapter nine's right-click `@move` on 700 sends the group
north to (2976, 12000). The staging, the premise's killer and the
falsifiers were committed before the run (`5e591b5`).

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch14 \
    --map 14 --end-frame 1150 --log-window 605 1150 --timeout 3600 \
    --detail end:UNITS=3,GUYS=2,BUILDS=7,LEADERS=2,GROUPS=1 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter14.cmd
```

**The capture.** One take at `cover=0`: **284 s from launch to exit, 97 MB
of dump and 10.4 MB of trace**.
- The receipt says `success: true` and exit 0, with 1,151 frames.
- `MAP_STYLE 14` and seed 12345 read back, and five settings files were
  restored.
- `cmdsran.py` has all six cheat lines returning 1.
- `waitrun.sh` exited 0 on the click-free lane's receipt.

**The lane.** att-722's run211 held it until 07:29. Its lock was left
stale: pid 22231 was dead and the runner had exited. This run launched
at 07:29 and exited at 07:34.

**It is the same game as run208 to frame 610.** `rngcmp.py` finds 611
frames identical. The first difference is at 611, the frame after both
scripts' first `add`.

**The pool did not come out.** There are 0 `GROUPDATA` records in the
dump. That is run30's and run166's family, but not their cause: `DEATHS`
was off, and `gamelog.ini` carried `GROUPS=1`. `GameLog::full_dump
@00930380` runs `dump_units` and then `dump_groups`, as it did in run178,
whose pool came out. The one difference in what reaches that point is
`GUYS=2` here against `GUYS=4` in run178. **A hypothesis, not a finding**:
the unit dump ends on a guy record, whose type and detail the pool's lines
inherit, and `GUYS=2` rejects them. The chapter's measures do not need the
pool, because each member's own order prints the slot (`x`/`y`, `off_x`/
`off_y`), `orig`, the group move's `id` and its leader `oxx`.

### The issuer's own records

| trace frame | record | read |
| --- | --- | --- |
| 620 | `INFO 18` | `0/6` uid 13 at (2424, 16248); `0/9` uid 16 at (3576, 16248); `0/12` uid 19 at (3000, 17016) |
| 620 | `INFO 17` | line 5, refusal 0, package 10 → 32 bytes, three objects |
| 700 | `INFO 18` | `0/6` (2314, 16436), `0/9` (2520, 16056), `0/12` (2088, 16056): already on their Envelop slots |
| 700 | `INFO 17` | line 6, refusal 0, package 10 → 35 bytes (the 3-byte reuse and the 22-byte `move_to`) |
| 740 | `INFO 17` | line 7, refusal 0, package 10 → 26 bytes (the reuse and the 13-byte `form`) |

The first `form` is the emulator's 22 bytes behind the turn's 10-byte
`camera`: a 9-byte `group` of three and the 13-byte command.

### The processed commands, and §22's falsifiers

The dump prints three commands between blocks:
- `process_group, new 0 3 621` and `process_form 2 0 2 621` between 621
  and 622;
- `process_group, repeat 0 0 701` and `process_move_to 2976 12000 2 0 0 1
  0` between 701 and 702;
- `process_group, repeat 0 0 741` and `process_form 0 0 2 741` between
  741 and 742.

| check | predicted | observed |
| --- | --- | --- |
| 1, the issue | appended, processed on the next frame | as predicted, on 621, 701 and 741 |
| 2, no `FormOrder` | no `FORMORDER` anywhere | **none in the whole dump** |
| 3, the byte | `form 2` on all nine on 622, `form 0` on 742 | exactly so; `form_mod 50`, `group 1` on each |
| 4, the re-form on the spot | one `GroupMoveOrder` a member, leader `0/6`, no action bit, to a slot round `0/6` | **a plain `MOVEORDER` (type 1) a member**, no action bit (`flags` 1 on six, 0 on `0/7`, `0/9`, `0/12`), `orig` = its own slot, no `GROUPORDER`; the slots lie round `0/6` at (2411, 16270): `0/6` → (2328, 16440), `0/9` → (2520, 16056), `0/12` → (2088, 16056). Every stack is empty on its slot by 684 (`0/6` 631 … `0/10` 684) |
| 5, the move reads the byte | Envelop `GroupMoveOrder`s with the action bit on 702 | nine `GroupMoveOrder`s (type 19), `flags 5`, `orig` (2976, 12000), id 701101, leader `oxx 6`, `form 2` |
| 6, halt and replay | on 742, one `GroupMoveOrder` a member, a new id, Line, `orig` 700's point, the action bit | exactly so: id **741102**, `flags 5`, `orig` (2976, 12000), `oxx 6`, `form 0`; nothing under it |
| 7, arrival in Line | stacks empty near 900 | empty from 905 (`0/6`) to 926 (`0/9`), round (2976, 12000) |

**Falsifier 4 fired in its letter.** The members hold plain moves on the
block the command was processed, not group moves. Everything else about
them is as read: the byte, no action bit, a slot round the leader, and
arrival. `Unit::do_group_move@005e79a0` ungroups a group move whose
point lies within `0x5ff` of the leader (`ungroup_move_order`, its tail).
A re-form on the spot always does. **A hypothesis, not a finding**: the
orders were laid grouped on 621 and ungrouped on their first step. This
crate models both `do_group_move` and the ungroup, so its walk will say.
## run211 — Great Lakes' word 15608, past run202's last block (2026-09-25, item 722)

**What it is.** run202's line, unchanged,
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, over
`[15435, 15860)`, plus `rontrace.cfg` `cover=1` and `window=15604-15612`
over the word. `!quit` at 15870, through `viadriver.sh` with no human at
the menu.

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 722
```

**Why it was owed.** Item 722 moved Great Lakes' word 15384 → 15608:
`CityData::num_wonders` counts a wonder site (`docs/AI.md` §70). run202
ends on block 15440, so the word's block 15609 is on no disk. On 15608
ours spends 40 draws against 38, parting at index 0: ours throws
`Army::find_target+0x410`, the difficulty gate's coin, where the
original's first draw is the per-candidate score `+0x7df`. **Sized with
250 blocks of runway** (DECISIONS 50 §7): six blocks shared with run202,
the 168 up to the word, its block, and 250 above it. The readings and
their kills are in the stanza.

**Taken whole.** 933.7 MB of dump and 20.9 MB of trace, 425 blocks
15435..15859. About 35 minutes from launch (06:54) to archive (07:29),
with no human at the menu. Waited on with `waitrun.sh`, exit 0.

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run53.log` | **0 differing**, 15,871 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 14 |
| the window, block for block | **425 blocks, 15435..15859, no gap** |
| a `GROUPDATA` on every window block | **425** |
| overlap with run202, nothing excluded | **6 in common (15435..15440), 0 differ** |
| the coverage window | a set on all 9 frames 15604..15612 |

**What it settled** (`run211_s_word_frame_is_widened_whole`,
`docs/AI.md` §70.5).
- **R1 holds**: the checks above.
- **R2 is not killed, and not shown.** Its killer did not fire: the
  escort `1/77`/`1/78`, army 3's, parts in position on 15607. But army 3
  is mustering, so the escort cannot be `find_aggressive_army`'s answer;
  armies 1 and 2 are the candidates, and none of their units parts before
  15609. The claim's own unit is an army's `pos` and its cell's owner,
  which the dump does not print — ~~a packet's question (run212)~~
  answered from the dump by item 729 (AI §70.5, §71.2): the gate before
  the coin is `frame_attacked`, which run211 prints every block.
- **R3 holds.** Nothing parts on 15441..15606; the first rows, on 15607,
  are the escort's (group 67, army 3), trading guard posts.
- The floor is run202's walk (401, nothing on run202's runway since item
  722), then 78 keys on run211's own blocks up to the word, and 1003 to
  the window's end.

## run218 — Great Lakes' word 16460, past run211's last block (2026-09-25, item 736)

**What it is.** run211's line, unchanged,
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, over
`[15854, 16712)`, plus `rontrace.cfg` `cover=1` and `window=16456-16464`
over the word. `!quit` at 16722, through `viadriver.sh` with no human at
the menu.

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 736
```

**Why it was owed.** Item 736 moved Great Lakes' word 15619 → 16460:
the siege arm's sub-group lays out on its own cleared `facing`
(`docs/GROUPS.md` §26). run211 ends on block 15859, so the word's block
16461 is on no disk. On 16460 ours spends 1 draw against 3, parting at
index 1: the original spends `Guy::set_anim+0x97a < Unit::move_step+0x823`,
a blocked step, which ours does not. **Sized with 250 blocks of runway**
(DECISIONS 50 §7): six blocks shared with run211, the 601 up to the word,
its block, and 250 above it. The readings and their kills are in the
stanza.

**Taken whole.** 1,870.9 MB of dump and 21.4 MB of trace, 858 blocks
15854..16711. About 64 minutes from launch (09:22) to archive (10:26),
with no human at the menu, while this lane worked. Waited on with
`waitrun.sh`, exit 0. The lane lock was stale (pid 74824).

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run53.log` | **0 differing**, 16,723 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 14 |
| the window, block for block | **858 blocks, 15854..16711, no gap** |
| a `GROUPDATA` on every window block | **858** |
| overlap with run211, nothing excluded | **6 in common (15854..15859), 0 differ** |
| the coverage window | a set on all 9 frames 16456..16464 |

**What it settled** (`run218_s_word_frame_is_widened_whole`,
`docs/GROUPS.md` §26.4).
- **R1 holds**: the checks above.
- **R2 is killed.** The blocked step on 16460 is not the escort's. On the
  word's block the original's `1/23`, a three-figure unit outside group
  67, carries `collide_o 79, collide_who 1`: it is blocked by The
  Despot. No escort member parts.
- **R3 is killed.** Nothing parts on 15860..16459. The first unit to
  part is `1/23`, on block 16460 (sim-frame 16459, a frame before the
  word): it stands stopped at (41632, 21466) in the original, and walks
  on here to (41632, 21440). It is not a member of group 67.
- The floor is run211's walk (398, nothing on run211's own blocks since
  item 736), then `1/23`'s 43 keys on 16460..16461, and 957 to the
  window's end. 355 rows stand on the word's block.
## run215 — chapter fifteen, the group attack (2026-09-25, item 731)

`docs/GOLDEN.md` §23, `tools/gamelog/golden/chapter15.cmd`. The cast is
two Hoplite squads, `0/6`–`0/8` on 610 and `0/9`–`0/11` on 612, and a
who=1 Chariot `1/6` on 614 at (3192, 12408). Three **player commands**
go through the DLL:
- a right-click `@move` on 620, north past the Chariot;
- `@attack` on 734, through `CommandManager::issue_attack`, the new verb;
- `@amove` on 860, `issue_move_to` with `ATTACK_TO`, the other new verb.

The staging, the premise's killer and the falsifiers were committed
before the run (`eff8b57`).

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch15 \
    --map 14 --end-frame 1250 --log-window 605 1250 --timeout 3600 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter15.cmd
```

**The capture.** One take at `cover=0`: **about 13 minutes from launch
(09:04) to exit (09:17), 269 MB of dump and 11.1 MB of trace**, 647
blocks.
- The receipt says `success: true` and exit 0, with 1,251 frames,
  `MAP_STYLE 14` and seed 12345.
- `cmdsran.py` has all six cheat lines returning 1.
- `waitrun.sh` exited 0 on the click-free lane's receipt.
- The lane lock was stale (pid 46167, 07:30).

**It is the same game as run210 to frame 614.** `rngcmp.py` finds 615
frames identical. The first difference is at 615, the frame after the
third `add`: a Chariot here, a Slinger there.

**The pool came out: 330,240 `GROUPDATA` records**, 512 on each of the
645 window blocks. The level was `GUYS=4` with neither `DEATHS` nor
`AMMO`, run178's. run210 asked at `GUYS=2` and got none. That makes
`GUYS=2` the likelier cause of run210's loss (parked 733), not a proof:
this line changed `GUYS` alone against run210's set, but the cast
differs too. The cost is the size: `GUYS=4` took the dump from run210's
97 MB to 269 MB over a window 100 blocks longer.

### The issuer's own records

| trace frame | record | read |
| --- | --- | --- |
| 620 | `INFO 18` | `0/6` uid 13 at (2424, 16248); `0/9` uid 16 at (3576, 16248) |
| 620 | `INFO 17` | line 5, refusal 0, package 10 → 39 bytes: a 7-byte `group` of two and the 22-byte `move_to` |
| 734 | `INFO 18` | `0/6` at (2983, 13858), `0/9` at (3414, 13866), walking |
| 734 | `INFO 17` | line 6, refusal 0, package 10 → 30 bytes: the 3-byte reuse and the **17-byte `attack`** |
| 860 | `INFO 17` | line 7, refusal 0, package 10 → 35 bytes: the reuse and a `move_to` |

### The processed commands, and §23's falsifiers

The dump prints three commands between blocks:
- `process_group, new 0 2 621` and `process_move_to 3192 10752 2 0 0 1
  0` between 621 and 622;
- `process_group, repeat 0 0 735` and **`process_attack 6 1 0 2 735`**
  between 735 and 736;
- `process_group, repeat 0 0 861` and `process_move_to 3192 7680 2 0 0
  2 0` between 861 and 862, `orders` 2.

| check | predicted | observed |
| --- | --- | --- |
| 1, the issue | three commands, each processed on the next frame | as predicted |
| 2, no `GroupAttackOrder` | none in either spelling | **none in the whole dump** |
| 3, six `AttackOrder`s | one a member on 736, `ox 6 whom 1`, `mandatory 1`, the action bit, no group move under it | exactly so: `uid 12`, `flags 20`, `new_ord 1`, each over a `MOVEORDER` approach leg (`flags 1`); the right-click's group move is gone |
| 4, the target held | no other target while `1/6` prints; dead by 900 | held; in range by 780 (`in_range 1`, `new_ord 0`); **`1/6` last prints on 808** |
| 5, stacks empty after | empty on 850 | empty on 850: `0/7`, `0/8` by 810, the rest by 850 |
| 6, six `GroupAttackToOrder`s | one a member on 862, `flags & 4`, `orig` (3192, 7680), one id, leader `oxx 6` | exactly so: type 21, `flags 5`, id **861102**; no member has `unit_masks & 4` |
| 7, arrival | stacks empty by 1200 on slots round the point | ungrouped to plain `ATTACKTOORDER`s (type 2) on 1060, empty from 1080 on (2976, 7670), (2856, 7656), (3096, 7656), (3384, 7704), (3240, 7704) and (3528, 7704) |

**No falsifier fired.** The six arrival points are the ones this crate's
scratch walk predicted, to the unit. On 736 every member stands where
the scratch walk put it on the tick before, (2999, 13821) for `0/6`.

## run221 — East Indies' word 15985, past run78's last block (2026-09-25, item 708)

**What it is.** run166's line without `DEATHS`, the pool's line (run178's
and run218's), `MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`,
over `[15894, 16237)`, at `cover=0`: a draw-stream trace first. `!quit`
at 16247, through `viadriver.sh` with no human at the menu.

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 708
```

**Why it was owed.** Item 706 moved East Indies' word 15782 → 15985 (The
Senator, `docs/TECH.md` §"The government patriot"). run78 ends on block
15899 and run96 starts at 23960, so the word's block 15986 is on no disk.
**The disk answered the who, not the why.** The draw stream (run54's
trace against ours) agrees through 15984; on 15985 both sides spend the
head's two expiry rolls (`make_stuff+0x221`), then the original alone
spends `make_stuff+0x63d`, a bought slot's expiry roll (`docs/AI.md`
§2.6 step 6), and then both spend `think_scout+0x941`. The scout's draw
is spent on both sides; the extra is who=1's slot loop. run78 is
`LEADERS=1`, so no block past run166's 13699 prints the make list, the
purse or the sites. **Sized with 250 blocks of runway** (DECISIONS 50
§7): six blocks shared with run78, the 86 up to the word, its block and
250 above it, 343 blocks. The readings and their kills are in the
stanza.

**Taken whole.** 801.6 MB of dump and 22.5 MB of trace, 343 blocks
15894..16236. About 29 minutes from launch (11:07) to archive (11:36),
with no human at the menu. Waited on with `waitrun.sh`, exit 0. The lane
lock was stale (pid 83878).

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run54.log` | **0 differing**, 16,248 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 18 |
| the window, block for block | **343 blocks, 15894..16236, no gap** |
| a `GROUPDATA` / a `MAKEOBJECT` on every window block | **343 / 343** |
| overlap with run78, on what both lines print | **6 in common (15894..15899), 0 differ** |

**What it settled** (`run221_s_word_frame_is_widened_whole`,
`docs/AI.md` §72).
- **R3 holds**: on 15985 the original's slot 5 is a citizen, `t 50`, `val
  130909`, bought and rolled on the word's frame; ours' slot 5 is empty.
  The offer parts on 15984 (`create_units`' frame) and the rates under it
  on 15977, 93 here against 118/110/100.
- The cause is on the window's first block: who=1's `resource_cap`, 2992
  here and 3792 there on every capped good. `calc_resource_caps` adds
  `REPUBLIC_COMMERCE_BONUS` (50) under Republic, which who=1 took on
  15782, and this crate had no republic term.
- R2 and R4 die on the stream. `1/31`'s record parts only on parked rows.
- With the term built the word moves **15985 → 16683**, past run221's
  last block. The floor goes 285/288/851 → 280/280/290.
## run219 — chapter sixteen, explore and flee (2026-09-25, item 738)

`docs/GOLDEN.md` §24, `tools/gamelog/golden/chapter16.cmd`. The cast is a
Chariot `0/6` on 610 at (2424, 11640) and a Hoplite squad `0/7`–`0/9`,
captain `0/7`, on 612 at (10104, 11640). Four **player commands** go
through `issue_move_to@00941720` from the DLL, with its trailing selector:
- `@explore` on 620, the chariot south past the goody box at cell (1, 19);
- `@explore` on 640, the squad south-east, short of the box at (16, 21);
- `@flee` on 900, the chariot back north;
- `@flee` on 1000, the squad back north-west.

The staging, the premise's killer and the falsifiers were committed
before the run (`a334f7d`, `3a71525`).

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch16 \
    --map 14 --end-frame 1250 --log-window 605 1250 --timeout 3600 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter16.cmd
```

**The capture.** One take at `cover=0`: **769 s from launch (10:27) to
exit (10:40), 263 MB of dump and 11.2 MB of trace**, 647 blocks.
- The receipt says `success: true` and exit 0, with 1,251 frames,
  `MAP_STYLE 14` and seed 12345.
- `cmdsran.py` has all five cheat lines returning 1.
- The lane waited on att-736's run218. The lock named it; `waitrun.sh`
  on its log, under a Monitor, released at 10:27.
- `waitrun.sh` exited 0 on the click-free lane's receipt.

**It is the same game as run215 to frame 610.** `rngcmp.py` finds 611
frames identical; the first difference is 611, the frame after the
chariot's `add`.

**The pool came out: 330,240 `GROUPDATA` records**, 512 on each of the
645 window blocks, at run215's levels.

### The issuer's own records

| trace frame | record | read |
| --- | --- | --- |
| 620 | `INFO 18` | `0/6` uid 13 at (2424, 11640) |
| 620 | `INFO 17` | line 4, refusal 0, package 10 → 37 bytes: a 5-byte `group` of one and the 22-byte `move_to` |
| 640 | `INFO 18` / `17` | `0/7` uid 14 at (10104, 11640); line 5, refusal 0, 10 → 37 |
| 900 | `INFO 18` / `17` | `0/6` at **(2400, 17280)**, the click; line 6, refusal 0, 10 → 37 |
| 1000 | `INFO 18` / `17` | `0/7` at **(12672, 14976)**, the click; line 7, refusal 0, 10 → 37 |

### The processed commands, and §24's falsifiers

Each command is processed on the next frame, a fresh `group` of one and
the `move_to` with its selector:
- `process_move_to 2400 17280 2 0 0 3 0` between 621 and 622;
- `process_move_to 12672 14976 2 0 0 3 0` between 641 and 642;
- `process_move_to 2400 11520 2 0 0 4 0` between 901 and 902;
- `process_move_to 10752 11520 2 0 0 4 0` between 1001 and 1002.

| check | predicted | observed |
| --- | --- | --- |
| 1, the issue | four commands, each processed on the next frame | as predicted |
| 2, the class | one `EXPLORETOORDER` (type 3) a member on 622 and 642, one `FLEETOORDER` (type 4) on 902 and 1002, `flags 5`, `orig` the click, no group order | exactly so; the squad's at their slots (12696, 15000), (12552, 15048), (12792, 14904); no `GROUPMOVEORDER` anywhere on the four |
| 3, the look | a box leg on `0/6` by 760, on the squad by 880 | `0/6` on **685**, an `EXPLORETOORDER` `flags 1` to (1176, 15000), `orig` (1152, 14976); the squad on **804**, to (12696, 16536), `orig` (12672, 16512) |
| 4, a figure's look, the walk dropped | none | every member of the squad takes the leg with its captain; each keeps its `flags 5` explore behind it; `0/9`'s leg is re-aimed on 819 and 834 |
| 5, the re-issue to the slot | the re-issued explores laid out afresh round the click | exactly so: each member's re-issued explore is its 642 order, point and `orig`, on 804 |
| 6, the box opened | an `explore_goody` draw on the frame the unit enters the cell | **730** for (1, 19) and **855** for (16, 21), three game draws each at `Unit::explore_goody+0x27c` |
| 7, arrival | stacks empty by 900, 1000, 1150 and 1250 | `0/6` on 838, standing on the click (2400, 17280); the squad on 936–947; `0/6`'s flee on 1098; the squad's on 1175–1183 |

**No falsifier fired.** Every frame above is this crate's scratch-walk
frame to within a few blocks, and both boxes open on the predicted
frames. The two places the scratch walk was wrong are both the re-issue's
**point**. `0/6` ends on its click, (2400, 17280), because the re-issued
explore plans to its `orig`, where the walk had (2424, 17304). The
squad's re-issue keeps its 642 slots round the click, where the walk
re-formed them round the leader's slot and put `0/8` on (12600, 15096).

## run223 — chapter seventeen, the flight line (2026-09-25, item 746)

`docs/GOLDEN.md` §25, `tools/gamelog/golden/chapter17.cmd`. The cast:
- who=0's Airbase `0/2007` on 606 at tile (60, 72);
- a Fighter `0/6` on 610;
- a Bomber pair `0/7`, `0/8` on 612 and 614;
- who=1's Barracks `1/2006` on 616, at (21120, 16512).

Four **player commands** go through `issue_flight@00941d40` from the DLL:
- `@strike` on 620, the pair on the ground at the Barracks;
- `@flight` on 640, the Fighter to its base;
- `@flight` on 660, the pair to the base;
- `@strike` on 664, the pair, flying, at the Barracks.

The chapter, its premise's killer and the falsifiers were committed before
the run (`637f623`).

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch17 \
    --map 14 --end-frame 1400 --log-window 605 1400 --timeout 3600 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter17.cmd
```

**The first take stalled.** It launched at 11:45 on a stale lane lock
(att-708's run221 had exited) and hung in DXVK's device setup:
- `wine.log` ends at MoltenVK's `VkInstance`, 173 lines against run219's
  812;
- a 544-byte trace and no `gamelog.txt`;
- killed at the 3600 s timeout, settings restored.

It is kept as `~/ron-golden/ch17-stalled`. The commander checked the
machine and asked for one relaunch, as run157's first take had stalled
the same way.

**The second take** was one take at `cover=0`: **942 s from launch
(13:39) to exit (13:55), 325 MB of dump and 11.1 MB of trace**, 795
blocks.
- The receipt says `success: true` and exit 0, with 1,401 frames,
  `MAP_STYLE 14` and seed 12345.
- `cmdsran.py` has all eight cheat lines returning 1.
- `zsh tools/gamelog/waitrun.sh` exited 0 on the click-free receipt. Run
  bare, the script fails with 126: it is committed 100644 (Loop 756).

**It is the same game as run219 to frame 610.** `rngcmp.py` finds 616
frames identical; the first difference is 611, the frame after the
Fighter's `add`.

**The pool came out: 407,040 `GROUPDATA` records**, 512 on each of the
795 window blocks.

### The issuer's own records

| trace frame | record | read |
| --- | --- | --- |
| 620 | `INFO 18` ×2 / `17` | `0/7` uid 15 at (10104, 16248), `0/8` uid 16 at (13176, 16248); line 8, refusal 0, 10 → 42 bytes: a 7-byte `group` of two and the 25-byte `flight` |
| 640 | `INFO 18` / `17` | `0/6` uid 14 at (11640, 16248); line 9, refusal 0, 10 → 40 |
| 660 | `INFO 18` ×2 / `17` | the pair on their pads; line 10, refusal 0, 10 → 42 |
| 664 | `INFO 18` ×2 / `17` | the pair **airborne**, (10222, 16312) and (13289, 16322); line 11, refusal 0, 10 → 38: the 3-byte reuse and the `flight` |

Each command is processed on the next frame: `process_group, new 0 2
621` and `process_flight 621`; `new 0 1 641`; `new 0 2 661`; `repeat 0
0 665`.

### §25's falsifiers

| check | predicted | observed |
| --- | --- | --- |
| 1, the issue | four commands, each processed on the next frame | as predicted |
| 2, an unbased strike | no order on `0/7`, `0/8` on 622 | **no order**: `action_flight` skipped both |
| 3, the class | one `STRAFEORDER` (16), target −1, `mandatory 1`, flags 4, `AIRORDER` `oxx 2007 whose 0 cruising_alt 1600 returning 1`, `xx/yy −1`: `0/6` on 642, the pair on 662 | **exactly so**, on all three |
| 4, the strike | block 666: the pair's strafe re-pointed at `1/2006`, `returning 0` | **fired**: on 666 each holds one **`AIRPATROLORDER` (17)** over (21120, 16512), the Barracks' point, home `0/2007`, `returning 0`, flags 0. Block 665 still shows the home-bound strafe |
| 5, flight | the pads left, an altitude above 0 | the Fighter's guy `z` 348 on 660 and 547 on 700; its `cruising_alt` redrawn, 1600 → 1300 on 643 and 1500 on 659. `UnitData::air_alt` prints 0 throughout |
| 6, the landing | a `SPECIALANIMORDER` on `0/2007`, then `inside_up` 2007 by 760 | **fired in its letter**: no `SPECIALANIMORDER` in the whole dump. `0/6` holds its strafe on 721 and is inside `0/2007` on **722**, stack empty; its `mana_burn` refills 2 a frame to 0 on 778 |
| 7, the strike reaches the point | damage on `1/2006` by 900 | the patrol pushes a `STRAFEORDER` on `1/2006` (`mandatory 0`, flags 0) on 777 (`0/8`) and 778 (`0/7`); damage from **822** (285 on 850, 571 on 900, 951 on 1000); the Barracks is gone on **1080**, and the strafe drops on 1081 |
| 8, home again | `returning` by 1230; the pair inside by 1400; three planes in the base | `returning 1` on **1212** and **1214**, as read; **fired on the landing**: both are still flying home on 1399, (12068, 15846) and (12277, 16312). Only `0/6` is inside at the end |
## run226 — Great Lakes' word 17099, past run218's last block (2026-09-25, item 742)

**What it is.** run218's line, unchanged,
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, over
`[16706, 17351)`, plus `rontrace.cfg` `cover=1` and `window=17095-17103`
over the word. `!quit` at 17361, through `viadriver.sh` with no human at
the menu.

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 742
```

**Why it was owed.** Item 742 moved Great Lakes' word 16460 → 17099:
`move_step`'s give-up takes its waypoint where the unit stands
(`docs/COLLISION.md` §17). run218 ends on block 16711, so the word's
block 17100 is on no disk. On 17099 ours spends 217 draws against 225,
parting at index 0: the original's first is `Guy::set_anim+0x97a <
Unit::move_step+0x823`, a blocked step, where ours spends
`Guy::set_anim+0x97a < Unit::do_move+0x11cf`. **Sized with 250 blocks
of runway** (DECISIONS 50 §7): six blocks shared with run218, the 388 up
to the word, its block, and 250 above it — 645 blocks. The readings and
their kills are in the stanza.

**Taken whole.** 1,410.7 MB of dump and 21.9 MB of trace, 645 blocks
16706..17350. About 50 minutes from launch (12:46, when att-746's chapter
seventeen freed the lane) to archive (13:36), with no human at the menu.
Waited on with `waitrun.sh`, which exited 0.

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run53.log` | **0 differing**, 17,362 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 14 |
| the window, block for block | **645 blocks, 16706..17350, no gap** |
| a `GROUPDATA` on every window block | **645** |
| overlap with run218, nothing excluded | **6 in common (16706..16711), 0 differ** |
| the coverage window | a set on all 9 frames 17095..17103 |

**What it settled** (`run226_s_word_frame_is_widened_whole`).
- **R1 holds**: the checks above.
- **R2 is killed.** `1/23` walks freely on block 17100 (`collide_frame
  16983`), and no unit carries `collide_frame 17099`: the word's blocked
  stand never reached `resolve`. The original's `1/5` stands stopped
  with `collide_o 3` there and walks on here.
- **R3 is killed.** Nothing parts on 16712..17085. On block 17086
  who=1's `resource_cap` for food and wealth is 4800 there and 4000 here,
  +50 a good, after a purchase both sides make on 17085 (`leftover`
  parts only on 17088). Four citizens are re-dealt on the same block:
  the original sends `1/70` and `1/72` to `1/2022`, ours `1/3` and
  `1/5`. Their walks part from 17087, and `1/9`'s from 17088.
- The floor is run218's walk (398), then 147 keys on 17086..17100, and
  1,382 to the window's end. 456 rows stand on the word's block.

## run227 — East Indies' word 16683, past run221's last block (2026-09-25, item 752)

**What it is.** run221's line, unchanged,
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, over
`[16230, 16935)`, at `cover=0`: a draw-stream trace first. `!quit` at
16945, through `viadriver.sh` with no human at the menu.

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 752
```

**Why it was owed.** Item 708 moved East Indies' word 15985 → 16683 (the
republic's commerce cap, `docs/AI.md` §72). run221 ends on block 16236, so
the word's block 16684 is on no disk. On 16683 ours spends 6 draws against
7, parting at index 5: the original spends six `Guy::set_anim+0x97a <
Guy::inc_time+0x271` idle wraps before `Farms::inc_time+0x1ae`, ours five
(`1/18`'s two crew figures, `1/19`, `1/20`, `1/54`). **The disk answered
who wraps here, not who wraps there**: the stream agrees to 16682, and
run221's last block holds no figure clock that parts. A `cover=0` trace
names no unit. **Sized with 250 blocks of runway** (DECISIONS 50 §7): seven
blocks shared with run221, the 447 up to the word, its block, and 250
above it, 705 blocks. The readings and their kills are in the stanza.

**Taken whole.** 1,645.7 MB of dump and 23.0 MB of trace, 705 blocks
16230..16934. About 57 minutes from launch (14:22) to the trace's close
(15:19), with no human at the menu. The lane lock was stale (pid 15110).
Waited on with `waitrun.sh`, exit 0.

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run54.log` | **0 differing**, 16,946 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 18 |
| the window, block for block | **705 blocks, 16230..16934, no gap** |
| a `GROUPDATA` / a `GUY` on every window block | **705 / 705** |
| overlap with run221, nothing excluded | **7 in common (16230..16236), 0 differ** |

**What it settled** (`run227_s_word_frame_is_widened_whole`,
`docs/AI.md` §73).
- **R2 holds**: the sixth wrap is who=1's idle citizen `1/46`, idle since
  16529 with a 123-frame `CHAR_IDLE`. It parts first on 16595: on tick
  16594, its seventh idle count, ours' `find_gather_spot` sends it to the
  woodcutter `1/2009`. The original's finds nothing and leaves it
  standing.
- R3 and R4 are killed: no figure is on one side only, and no farm or
  farmer parts under the word.
- The cause is on the window's first block: who=1's two cities count their
  gatherers 11 and 11 here against 10 and 12 there. `plan_strategy` counts
  a gatherer in its **building's** city (`6babfb`), and this crate counted
  it in the nearest. On 16594 the crossing rule read London 14 against 10
  here and 12 against 12 there.
- With the census fixed the word moves **16683 → 16982**, past run227's
  last block.

## run233 — East Indies' word 17189, past run227's last block (2026-09-25, item 767)

**What it is.** run227's line, unchanged,
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, over
`[16929, 17441)`, at `cover=0`: a draw-stream trace first. `!quit` at
17451, through `viadriver.sh` with no human at the menu.

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 767
```

**Why it was owed.** Item 767 moved East Indies' word 16982 → 17189 (the
British price of Taxation, `docs/AI.md` §74). run227 ends on block 16934,
so the word's block 17190 is on no disk. On 17189 ours spends 2 draws
against 1, parting at index 0: ours spends a blocked stand,
`Guy::set_anim+0x97a < Unit::move_step+0x823`, and the original spends its
own on 17190. **The disk answered whose stand it is here, not there**: a
scratch print named ours' `1/55`, blocked by `1/60`, and run227 parts on
neither to its last block. **Sized with 250 blocks of runway**
(DECISIONS 50 §7): six blocks shared with run227, the 255 up to the word,
its block, and 250 above it, 512 blocks. The readings and their kills are
in the stanza.

**Taken whole.** 1,211.8 MB of dump and 23.4 MB of trace, 512 blocks
16929..17440. About 42 minutes from launch (16:39) to the trace's close
(17:21), with no human at the menu. Waited on with `waitrun.sh`, exit 0.

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run54.log` | **0 differing**, 17,452 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 18 |
| the window, block for block | **512 blocks, 16929..17440, no gap** |
| a `GROUPDATA` / a `GUY` on every window block | **512 / 512** |
| overlap with run227, nothing excluded | **6 in common (16929..16934), 0 differ** |

**What it settled** (`run233_s_word_frame_is_widened_whole`,
`docs/AI.md` §74).
- **R2 holds**: the original's `1/55` takes the same stand by `1/60` a
  frame later, `collide_o 60` on block 17191.
- **R3 holds**: `1/55` parts before the word, first on 17182 on
  `half_step` (0 here, 1 there), then `last_speed` 30 against 15 on 17183.
  Ours took a whole step where the original took a half one. `1/60` parts
  on nothing but the pool's `group` (689).
- **R4's killer fires**: the first row on the window that is no floor
  family's is who=1's `scholars`, 13 against 14, from 16971. `1/58`'s path
  parts from 17147 and `1/57`'s `half_step` from 17161, both before the
  pair.
- The floor is 293 keys on the first block, 363 before the word's frame
  and 1,139 in all.

## run235 — chapter seventeen's bombs, at `AMMO=5` (2026-09-25, item 770)

**What it is.** run223's game again: the same lobby, seed,
`chapter17.cmd` and detail line, with `AMMO=5` added, to 1100 rather
than 1400. run223 dumps no `AMMO`, so its bombs could be read only by
their damage. The disk could not answer where a bomb leaves the plane,
and that is what the fringe hits on 842 and after turn on.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch17-ammo \
    --map 14 --end-frame 1100 --log-window 605 1100 --timeout 3600 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,AMMO=5,LEADERS=2,GROUPS=1 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter17.cmd
```

**Taken whole.** One take on the click-free lane: 588 s from launch
(17:41) to exit, 208 MB of dump and 10.4 MB of trace. The lane lock
named att-767's longtrace, whose pid had exited, so it was taken over as
the launch line does. It was waited on with `waitrun.sh`, exit 0.

| check | result |
|---|---|
| receipt | `success: true`, exit 0, 1,101 frames, seed 12345, `MAP_STYLE 14` |
| `rngcmp.py` vs run223's trace | **0 differing**, 1,101 identical |
| window blocks | 497 |
| live `AMMO` records on 800–1100 | 784, every one the Bomber pair's |

**What it settled** (`docs/ORDERS.md` §35).
- **Every bomb is `total_time 17`**, and `ex − sx`, `ey − sy` is one
  tile along the heading.
- **Two bays, 19 under the plane**, turned by a whole degree taken from
  the angle's top byte. That model reproduces all 49 release points.
- With the bays built, all 784 records agree field for field
  (`run235_s_bombs_are_the_original_s_record_for_record`), except the
  target a round in flight keeps past the Barracks' death on 1080.

## run241 — chapter eighteen, the build line (2026-09-25, item 779)

`docs/GOLDEN.md` §26, `tools/gamelog/golden/chapter18.cmd`. The cast:
four citizens of who=0 staged on 610–616, `0/6` alone and `0/7`–`0/9`
together. Two **player commands** go through `issue_build@00941c30`
from the DLL's `@build`:
- on 620, `0/6` drops a Barracks (427) at (7296, 34176);
- on 640, the three drop a Siege Factory (430) at (7296, 36864).

The chapter, its premise's killer and the falsifiers were committed before
the run (`c0a7a0a6`).

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch18 \
    --map 14 --end-frame 1450 --log-window 605 1450 --timeout 3600 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter18.cmd
```

**One take at `cover=0`**: 1,002 s from launch (19:12) to exit, 348 MB
of dump and 11.3 MB of trace, 847 blocks. The lane lock named pid 7640,
which had exited, so the launch took it over. It was waited on with
`zsh tools/gamelog/waitrun.sh`, exit 0.

| check | result |
|---|---|
| receipt | `success: true`, exit 0, 1,451 frames, seed 12345, `MAP_STYLE 14` |
| `cmdsran.py` | all eight cheat lines returned 1 |
| `rngcmp.py` vs run223's trace | the same game to 614; the first difference is 615, the frame after the third staged unit |
| `GROUPDATA` | **432,640 records**, 512 on each of the 845 window blocks: the pool printed |

### The issuer's own records

| trace frame | record | read |
| --- | --- | --- |
| 620 | `INFO 18` / `17` | `0/6` uid 13 at (4728, 35448); line 7, refusal 0, 10 → 40 bytes: a 5-byte `group` and the 25-byte `build` |
| 640 | `INFO 18` ×3 / `17` | `0/7` uid 14 at (4728, 37752), `0/8` uid 15 at (5112, 37752), `0/9` uid 16 at (5496, 37752); line 8, refusal 0, 10 → 44: a 9-byte `group` of three and the `build` |

Each command is processed on the next frame: `process_group, new 0 1 621`
and `process_build 7296 34176 7296 34176 427 2 621`; `new 0 3 641` and
`process_build 7296 36864 7296 36864 430 2 641`.

### §26's falsifiers

| check | predicted | observed |
| --- | --- | --- |
| 1, the issue | both processed on the next frame | as predicted |
| 2, the site and the price | `0/2007` `orig_type 427` on 622, timber −120; `0/2008` `orig_type 430` on 642, timber and metal −60 | **exactly so**: timber 241 → 121 on 622; 122 → 62 and metal 100 → 40 on 642; both sites `construct_hits 1`, `frame_started −1` |
| 3, the class | a `MOVEORDER` then a `BUILDORDER` (flags 4) on each | **the class holds**: `0/6` on 622, a `MOVEORDER` to (6840, 34440) and a `BUILDORDER` on `0/2007`, uid 17; `0/7`, `0/8`, `0/9` on 642, each a `MOVEORDER` to its own ring spot, (6840, 37032), (6840, 36792), (7128, 37320), and a `BUILDORDER` on `0/2008`. No `EXPLORETOORDER`. The move reads **flags 1**, not 0, on the processed block: its first step has already set the path bit. `0/8`'s later move shows the adder's 0 on the block it is laid (1097) and 1 the next |
| 4, the walk | constructing by 760 | the Siege Factory from **709** (`0/9` first), the Barracks from **721**; no re-swarm |
| 5, the rate | the Barracks by 1200, the Factory by 1000 | `construct_hits` 1200 on **948** (the Factory) and **1141** (the Barracks) |
| 6, letting go | no `BUILDORDER` past the last frame | the three's stacks are empty on 948, `0/6`'s on 1141 |

**After the builds.** The three stand idle from 948. On **1097** `0/8`
takes a `MOVEORDER` to (7176, 34632) and a `BUILDORDER` on `0/2007` with
**flags 0**, `find_build_spot`'s help. It is still walking when the
Barracks is finished on 1141, reaches its spot on ~1200 and stands idle.
No builder gathers to 1449.

**This crate's prediction held on every block read**: the ring spots, the
starts on 709 and 721, `construct_hits` block for block (53 and 116 on
740, 110 and 220 on 760), the completions, and the help on ~1097. The one
row it had wrong is the help's class: the prototype laid an
`EXPLORETOORDER` where the original lays a `MOVEORDER`, because this
crate's one-unit `swarm_around` never asks whose builder it is.
## run240 — Great Lakes' world at 17087, a packet at logger frame 17087 (2026-09-25, item 776)

**What it is.** The long game on the click-free lane to 17093. It carries
a `RON_STATE_FRAME=17087` packet: the state after trace tick 17086, with
tick 17087's world searches for `1/9` and `1/72` still ahead of it. It
also carries a `WORLD=6` dump over [17086, 17089) beside run226's line.
The trace's proxies were live all game (`callwin 0–17093`), so every
`calc_cost` and `astar_path` is on it.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-25-run240 \
    --map 14 --end-frame 17093 --timeout 3600 --log-window 17086 17089 \
    --detail end:MISC,WORLD=6,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1 \
    --tracer-def RON_STATE_FRAME=17087 \
    --tracer-def 'RON_STATE_PLAN="<plan>/plan.h"'
```

The plan is the lab's Great Lakes plan, reused as run189 reused it
(`~/ron-data/lab-experiments/2026-09-23-item-597/plan/`).

**Why a packet, and what the disk could not answer.** On tick 17087 the
original's `1/9` and `1/72` plan other world paths than ours to `1/2022`.
Whether they plan over another world (cells, tile masks, danger, fog)
or with another planner is a value the original holds. run226 prints no
`WORLD`. run189's `WORLD` (14529, every tile mask agreeing) is older than
the Pyramids' site (15382) and every building placed since.

**Taken whole.** `success: true`, exit 0, `MAP_STYLE 14` and seed 12345
read back, five settings files restored. **90 s launch to exit, 99 s in
all**, with an 859,770,264-byte packet, 16 MB of dump and 69 MB of trace.
The lane lock was stale: pid 62875 was dead. Waited on with `waitrun.sh`,
which exited 0. The dump is archived as
`gamelog-run240-greatlakes-worldword.txt`, and the trace as
`rontrace-run240.log`. The packet stays at
`~/ron-data/lab-captures/2026-09-25-run240/map-14`.

| check | result |
|---|---|
| `rngcmp.py` against `rontrace-run53.log` | **0 differing**, 17,094 identical |
| the window | `WORLD` on block 17087, 3,600 cells and 57,600 tile masks |
| the proxies | 740,390 calls; tick 17087 holds six world searches, 303 priced steps |

**What it settled** (`docs/PATHFINDER.md` §27,
`run240_s_world_at_17087_is_the_original_s`).
- **The world agrees where the searches look.** The only cell that
  parts is (2, 40). The tile masks part on `0x4` alone. who=1's danger
  agrees whole.
- **The planner parts on `1/9`'s first expansion.** The original's
  validity probe refused forest-flagged cells (56, 18) and (55, 19):
  `invalid_loc`'s cell arm under `valid_wcoord`'s flags.
- With the clause, all 303 priced steps agree, and the word moved
  17128 → 17181.
- The packet was not read: the dump and the trace answered every term.

## run245 — chapter nineteen, the cast line (2026-09-25, item 790)

`docs/GOLDEN.md` §27, `tools/gamelog/golden/chapter19.cmd`. The cast: a
who=1 Barracks staged on 606 and a who=0 Spy on 610; on 620 one **player
command** goes through `issue_spell@00941b80` from the DLL's `@spell`:
`0/6` casts the Informer (639) on `1/2006`, picked at (15360, 15360).

The chapter, its premise's killer and the falsifiers were committed before
the run (`fdaf52e3`).

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch19 \
    --map 14 --end-frame 1100 --log-window 605 1100 --timeout 3600 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter19.cmd
```

**One take at `cover=0`**: 540 s from launch (20:37) to exit, 201 MB of
dump and 10.2 MB of trace, 496 window blocks. The lane lock named pid
49990, run246's own game, which had exited, so the launch took it over.
It was waited on with `zsh tools/gamelog/waitrun.sh`, exit 0.

| check | result |
|---|---|
| receipt | `success: true`, exit 0, 1,101 frames, seed 12345, `MAP_STYLE 14` |
| `cmdsran.py` | all seven cheat lines returned 1 |
| `rngcmp.py` vs run246's trace | **0 differing** of 626 in common: the same game as the packet's to its end |
| `rngcmp.py` vs run241's trace | the same game to 612; the first difference is 613, three frames after the Spy |
| `GROUPDATA` | **253,440 records**, 511 on each of the 496 window blocks: the pool printed |

### The issuer's own records

| trace frame | record | read |
| --- | --- | --- |
| 620 | `INFO 18` / `17` | `0/6` uid 13 at (11640, 15864); line 6, refusal 0, 10 → 36 bytes: a 5-byte `group` and the 21-byte `spell` |

The command is processed on the next frame: `process_group, new 0 1 621`
and `process_spell 639 2006 1 621`.

### §27's falsifiers

| check | predicted | observed |
| --- | --- | --- |
| 1, the issue | processed on 621 | as predicted |
| 2, the class | a `MOVEORDER` and a `CASTORDER` (spell 639, `paid` 1) on 622 | **the class holds**: a `CASTORDER` (flags 4, `ox 2006 whom 1`, `x 15360 y 15360`, `paid 1`, `spell 639`) and a `MOVEORDER` to (14232, 15528). The dump **lists the cast first**, the order laid first; the move pushed QUEUE_FIRST is the head (`orders_x/y` its point). `cavarch_o 2006`, `cavarch_uid 12`, `cavarch_who 1` |
| 3, the price | `mana_burn` +500, no bucket down | `mana_burn` 489 → **988** on 622; who=0's buckets 254/241/113/100/100 on 621 and 622, and only rising after |
| 4, the walk | to (14232, 15528); `spell_time` rising in range, by 800 | the move ends on **755** at (14232, 15528) exactly, ~20 units a frame; `spell_time` 1 on **756** |
| 5, the cast | `infiltrated` 1 forty frames in; the stack empty; the Spy alive | `spell_time` 1 … 39 on 756–794; on **795** `1/2006` `infiltrated` 1 and `0/6`'s stack empty; the Spy stands there to 1099 |
| 6, the cloak | no 0x1000 or 0x10000; 0x20000, `visible` 0x2, `flags` 0x80 in range | `unit_masks` 8 → **0x20008** on 756, back to 8 on 795; never 0x1000 or 0x10000; `visible` 2 from 756 (cleared on 827, the unit's 32-frame tick); `flags` 1 → **0x81** from 756, 9 from 796 |
| 7, the mana | still while 0x20000, falling after | 854 from 756 to 795, 853 on 796 and one a frame after |

**No falsifier fired.** The emulator's first frame on the packet (the
order, the target fields, the price and the approach spot) is what the
capture holds on 622, value for value.

## run246 — chapter nineteen's staging, a packet at logger frame 619 (2026-09-25, item 790)

**What it is.** `chapter19.cmd`'s staging without its `@spell` line, to
625, with a `RON_STATE_FRAME=619` packet (814,570,380 bytes; the plan is
run144's). 59 s launch to exit, `success: true`, 626 frames. The lane lock
named pid 98880 (att-785's long trace), which had exited.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-25-run246 \
    --map 14 --end-frame 625 --log-window 605 625 --timeout 2400 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file <chapter19.cmd without the @spell line> \
    --tracer-def RON_STATE_FRAME=619 \
    --tracer-def 'RON_STATE_PLAN="<plan>/plan.h"'
```

**What the disk could not answer.** Which of the Spy's crafts the staged
Barracks is a valid target for, the craft's range and job time on it,
and what `action_spell` and the first `do_cast` write: no dump prints
`is_valid_target`, and no capture had issued a targeted craft.

**What it answered** (`tools/recomp/step4.py`'s machinery, scratch):
`is_castable` 3 for Bribe, Counterintelligence and the Informer;
`is_valid_target` 1 for the Informer alone; `get_range` 960, `get_job_time`
40; the Spy's `mana` 1000 and `mana_burn` 491. `Group::action_spell` and one
`Unit::do_cast` on a pool `Group` of `[6]`: a `CastOrder`, `paid` 1,
`mana_burn` 991, every bucket rewritten unchanged, and a `MOVEORDER` to
(14232, 15528). `docs/GOLDEN.md` §27.
## run243 — Great Lakes' word 20568, sized to the word (2026-09-25, item 785)

**What it is.** run226's line and lobby,
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, over
`[20500, 20819)`, with `rontrace.cfg` `cover=0`. `!quit` at 20829,
through `viadriver.sh` with no human at the menu.

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 785
```

**Why it was owed.** Item 785 moved Great Lakes' word 17181 → 20568:
`wonder_mark`'s writer (`docs/AI.md` §75). run226 ends on block 17350
and run80 starts on 23960, so no dump holds the word's block, 20569.
On 20568 ours spends 37 draws against 38, parting at index 31. Ours
spends `Guy::set_anim+0x97a < Guy::inc_time+0x271`, and the original
spends `< Unit::move_step+0x823`, a blocked step.

**Sized to the word, not to the gap** (DECISIONS 50 §7; the
commander's ruling on item 785). A capture contiguous with run226
would be ~3,470 blocks and ~4.5 hours. This one is 319 blocks: 68 into
the word, its block 20569, and 250 of runway above it. **No dump
compares a value over 17351..20499.** The draw stream agrees across
that range in count and sequence, and nothing else is known of it.
The same game is shown by the draw stream against run53, not by a dump
overlap. The readings and their killers are in the stanza.

**Taken whole.** 706.8 MB of dump and 23.9 MB of trace, 319 blocks
20500..20818. About 26 minutes from launch (19:56) to archive (20:22),
with no human at the menu. The lane lock was stale: pid 41224 was dead.
Waited on with `waitrun.sh`, which exited 0.

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run53.log` | **0 differing**, 20,830 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 14 |
| the window, block for block | **319 blocks, 20500..20818, no gap** |
| a `GROUPDATA` and a `GUY` on every window block | **319** each |

**What it settled** (`run243_s_word_frame_is_widened_whole`,
`docs/AI.md` §75.6).
- **R1 holds**: the checks above.
- **R2 holds.** On the word's block, `1/40` stands blocked by `8/0` in
  the original (`collide 1`, `stopped 1`) and walks here. Its own walk
  had already parted on run243's first block.
- **R3's killer fires.** 59 rows part on 20500 that were not standing on
  run226's last block. They are four who=1 walkers (`1/40`, `1/41`,
  `1/42`, `1/60`), `1/80`'s formation and `SITE[1].reg`. Where in
  17351..20499 any of them first parts is on no disk.

## run248 — East Indies' world at 17146, a packet at logger frame 17146 (2026-09-25, item 773)

**What it is.** The long game on the click-free lane to 17152. It carries
a `RON_STATE_FRAME=17146` packet: the state after trace tick 17145, with
tick 17146's world searches for army 0's column still ahead of it. It also
carries a `WORLD=6` dump over [17145, 17148) beside run233's line. The
trace's proxies were live all game (`callwin 0–17152`), so every
`calc_cost` is on it.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-25-run248 \
    --map 18 --end-frame 17152 --timeout 3600 --log-window 17145 17148 \
    --detail end:MISC,WORLD=6,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1 \
    --tracer-def RON_STATE_FRAME=17146 \
    --tracer-def 'RON_STATE_PLAN="<plan>/plan.h"'
```

The plan is 597's, reused as run150 and run240 reused it
(`~/ron-data/lab-experiments/2026-09-23-item-597/plan/`).

**Why a packet, and what the disk could not answer.** On tick 17146
the original's `1/58` plans another world path than ours. Whether the two
plan over another world or with another planner is a value the original
holds. No East Indies dump on disk carries a whole `WORLD` past its
start: each holds 21,600 `danger[who][scan]` lines, the start's three
scans. run233's trace, and run227's, are `cover=0`: the draw stream, with
no proxies.

**Taken whole.** `success: true`, exit 0, `MAP_STYLE 18` and seed 12345
read back, five settings files restored. **98 s launch to exit, 107 s in
all**, with an 858,021,144-byte packet, 17 MB of dump and 69 MB of trace.
The lane lock was stale: pid 62503 was dead. Waited on with `waitrun.sh`,
which exited 0. The dump is archived as
`gamelog-run248-eastindies-worldword.txt`, and the trace as
`rontrace-run248.log`. The packet stays at
`~/ron-data/lab-captures/2026-09-25-run248/map-18`.

| check | result |
|---|---|
| `rngcmp.py` against `rontrace-run54.log` | **0 differing**, 17,153 identical |
| the window | `WORLD` on blocks 17145..17147, 3,600 cells and 57,600 tile masks |
| the proxies | tick 17146 holds 28,828 priced steps |

**What it settled** (`docs/PATHFINDER.md` §28,
`run248_s_world_at_17146_is_the_original_s`).
- **The world agrees.** No cell and no danger value parts. The tile
  masks part on `0x4` alone (264). The fog parts on 16 half-cells of
  who=1's bit, and no search of the tick prices one.
- **The planner parts on the 32nd priced step**, `1/48`'s, 60 here
  against 116 there: who=1's danger / 8. `astar_path` sets `no_danger`
  under an `ATTACK_TO` order, and this crate did not.
- With the arm, all 28,828 steps agree, and the word moved 17189 → 17403.
- The packet was not read: the dump and the trace answered every term.

## run250 — chapter twenty's staging, to 646 (2026-09-26, item 803)

**What it is.** `chapter20.cmd`'s first eight lines — `!ai off`, `library
who=0 1`, a Dock, two Chariots, `@settransport 0 0 7` on 620 and both
moves onto lake 70 on 640 and 642 — to 646, dumped over `[605, 646)`.
92 s launch to exit, `success: true`, 647 frames, 20,992 `GROUPDATA`.
The lane lock named pid 97451, which had exited. No packet: every
predicate the staging asks is a field the dump prints.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-26-run250 \
    --map 14 --end-frame 646 --log-window 605 646 --timeout 2400 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file <chapter20.cmd's first eight lines>
```

**What the disk could not answer.** Whether `library who=0 1` and an
`add`ed Dock grant a human the transport level, whether the DLL's
`@settransport` clears the bit, and what plan each Chariot's move takes to
the water: no capture on disk had a human's Dock or the toggle.

**What it answered** (`docs/GOLDEN.md` §28): `leader_flags` 1799
— the three level bits — from 605; `0x800000` on every who=0 unit; `process_set_transport
621` and `0/7`'s `unit_masks` 8388608 → 0 on 622, stack empty; on 642
`0/6`'s six-entry plan with the embark (flags 4) at (8856, 34200); on 644
`0/7`'s six-entry plan onto the water with no embark and no pull-back.
This crate walks it whole: eight lines ran, nothing parts to 646.

## run249 — chapter twenty, the board line (2026-09-26, item 803)

**What it is.** `chapter20.cmd` whole — `!ai off`, `library who=0 1`, a
Dock, two Chariots, and six issuer lines: `@settransport 0 0 7` on 620,
both moves onto lake 70 on 640 and 642, `@settransport 0 1 7` on 800,
`0/7`'s move again on 820, and barge `0/8` to the east bank on 900 —
dumped over `[605, 1300)`, 695 blocks, at `cover=0`. The lane lock named
pid 73860, run250's own game, which had exited.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch20 \
    --map 14 --end-frame 1300 --log-window 605 1300 --timeout 3600 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter20.cmd
```

**Taken whole, one take.** 836 s launch to exit, 285.8 MB of dump and
10.8 MB of trace, `success: true`, 1301 frames. Waited on with
`waitrun.sh`, which exited 0.

| check | result |
|---|---|
| `rngcmp.py` vs run250's trace | **0 differing**, 647 identical |
| `MAP_STYLE` and seed from the receipt | 14, 12345 |
| `GROUPDATA` (the receipt's count) | **355,840**, 512 on each of 695 blocks |
| `process_set_transport` / `process_move_to` | 621, 801 / 641, 643, 821, 901 |
| `BOARDORDER` or `AWAITBOARDORDER` | **0** |

**What it settled** (`docs/GOLDEN.md` §28): no falsifier fired. `0/7`'s
bit off on 622 and on on 802; `0/6`'s Transport `CASTORDER` (650) on 703
and barge `0/8` on 704; `0/7` stopped at the shore on 719; its cast on 829
and barge `0/9` on 830; `0/6` ashore on 1160 with `0/8` gone. This crate
walks it whole — eleven lines, and no draw or `game_random` word parts to
1300.
## run251 — East Indies' word 17501, sized to the word (2026-09-26, item 800)

**What it is.** run233's line, unchanged,
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, over
`[17496, 17753)`, at `cover=0`: a draw-stream trace. `!quit` at 17763,
through `viadriver.sh` with no human at the menu.

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 800
```

**Why it was owed.** Item 800 moved East Indies' word 17403 → 17501 (the
anchor's sub-group sorts its own list, `docs/GROUPS.md` §27). run233 ends
on block 17440, so the word's block 17502 is on no disk. On 17501 ours
spends 13 draws against 11, parting at index 3. Ours spends
`Guy::set_anim+0x97a < Guy::move+0x19f` where the original spends
`< Unit::do_idle+0x7d`. **The disk answered whose step it is here, not
there**: a scratch print named ours' `1/57`, one of army 0's column,
which run233 parts on from 17405. **Sized to the word** (DECISIONS 50
§7): the six blocks before its block, the block, and 250 above it, 257
blocks. No capture shares a block with it, so **17441..17495 is compared
by no dump**, and the same game is the draw stream's word against run54.

**Taken whole.** 618.9 MB of dump and 23.7 MB of trace, 257 blocks
17496..17752. About 25 minutes from launch (03:10) to archive (03:35),
with no human at the menu. The first launch at 02:57 was dropped
silently: another lane's capture held RonDriver, and `viadriver.sh`'s
`open -a` does not start a second instance. It was relaunched when that
pid exited. Waited on with `waitrun.sh`, exit 0.

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run54.log` | **0 differing**, 17,764 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 18 |
| the window, block for block | **257 blocks, 17496..17752, no gap** |
| a `GROUPDATA` / a `GUY` on every window block | **257 / 257** |

**What it settled** (`run251_s_word_frame_is_widened_whole`,
`docs/GROUPS.md` §27.4):
- **R2 holds**: `1/57` parts on position from the window's first block,
  about 20 units off, carried in. On 17501 ours' stands with one path
  entry where the original's holds two and walks on. On the word's block
  the original's sets off (`cur_anim 8`) toward (31451, ·), where ours
  has taken a leg to (30683, ·).
- **R3's killer fires**: the column parts on 17496 on positions as well
  as on the ids.
- **R4's killer fires**: `9/9` and `9/15` agree on every gaia row to the
  word, so the extra clock is not an animal's own parting.
- Every army's list agrees on the word's blocks. The floor is 350 keys on
  the first block, 11 under the word, 361 on its block and 1,314 in all.

## run253 — East Indies' word 18182, sized to the word (2026-09-26, item 811)

**What it is.** run251's line, unchanged,
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, over
`[18177, 18434)`, at `cover=0`: a draw-stream trace. `!quit` at 18444,
through `viadriver.sh` with no human at the menu.

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 811
```

**Why it was owed.** Item 811 moved East Indies' word 17501 → 18182 (a
retarget forms the army twice, `docs/ARMY.md` §21). run251 ends on block
17752, so the word's block 18183 is on no disk: every `*eastindies*` and
`*islands*` dump was grepped by its block range first. On 18182 ours
spends 9 draws against 11, parting at index 0: ours spends
`Leader::make_stuff+0x221` where the original spends
`Leader::use_market+0x1ed`. **What the disk could not answer**: who=1's
make list and stockpile on the word's blocks. **Sized to the word**
(DECISIONS 50 §7): the six blocks before its block, the block, and 250
above it, 257 blocks. No capture shares a block with it, so
**17753..18176 is compared by no dump**, and the same game is the draw
stream's word against run54.

**Taken whole.** 621.4 MB of dump and 24.3 MB of trace, 257 blocks
18177..18433. About 24 minutes from launch (04:26) to archive (04:50),
with no human at the menu. Waited on with `waitrun.sh`, exit 0.

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run54.log` | **0 differing**, 18,445 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 18 |
| the window, block for block | **257 blocks, 18177..18433, no gap** |
| a `GROUPDATA` / a `GUY` on every window block | **257 / 257** |

**What it settled** (`run253_s_word_frame_is_widened_whole`,
`docs/ARMY.md` §21.4):
- **R2 holds**: who=1's make list parts first, on 18181. `MAKE[0]` is
  type 61 (cat 4, val 952380) here against 590 (cat 8, val 22784)
  there, and `MAKE[4]` holds a second 61 here against an empty slot.
- **R3's killer fires**: every stockpile row but the wealth floor's
  agrees on 18182. On 18183 the original has been to the market
  (`leftover[2:wealth]` 6862 → 560) and ours has queued at `1/2013`.
- No unit parts under the word. Past it, ours' `0/0` sets `cur_anim` 2
  on 18185 where the original's stays at 0.
- The floor is 310 keys on the first block, 18 under the word, 327 on
  its block and 977 in all.

## run256 — chapter twenty-one's staging, to 830, four takes (2026-09-26, item 813)

**What it is.** `chapter21.cmd`'s twelve lines — `!ai off`, `library` for
both, a who=0 Barracks, who=1 attackers, `peace 1` on 760, four citizens
and the two `@repair` lines on 780 and 800 — to 830, dumped over
`[605, 830)`. Four takes under one number, each a staging the one before
refused; only take 4 is the chapter's. Each ran on a free lane (the lock
was released by att-811's run253 before take 1), `success: true`.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-26-run256 \
    --map 14 --end-frame 830 --log-window 605 830 --timeout 2400 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file <chapter21.cmd's twelve lines>
```

**What the disk could not answer.** No dump on disk prints a human's
`REPAIRORDER` (every golden capture greps to none), whether the DLL's
`@repair` reaches `process_swarm_around`, or how a staged building takes
damage a player can then mend.

**The takes** (kept at `~/ron-data/lab-captures/2026-09-26-run256-take1`
to `-take3`; take 4 is `2026-09-26-run256`):

1. The Barracks on the neutral arena at tile (14, 74) and who=1 hoplites
   at (20, 74), `AMMO` not asked (312 s). No blow: a building on an
   unowned cell is no target (`docs/COMBAT.md` §12.2). The command was
   processed anyway: `process_swarm_around 2007 0 2 13 781`, and `0/6`
   under a `MOVEORDER` and a `REPAIRORDER` from 782.
2. The Barracks at tile (30, 170) in who=0's territory, hoplites at (35,
   170): they strike from 660, 4 a blow, `damage` 48 by 760. This crate
   parts on 636, the first step's walk-animation draw.
3. who=1 Bowmen at (34, 170), `library who=1 2`: they fire from where they
   stand, `damage` 3 by 744. This crate parts on 650, the first arrow a
   frame late: piece 120 had no measured bay.
4. The same with `library who=1 3`, which changed nothing the dump
   shows (the piece is 120 at both). Its twelve arrows measured piece
   120's three bays (`docs/GOLDEN.md` §29), and with them **this crate
   walks take 4 whole to 830**.

**What take 4 answered** (`docs/GOLDEN.md` §29): the Barracks `0/2007` at
(5760, 32640); arrows from 644, `damage` 0 → 3 (frac 12) by 744; the
Bowmen off from 765; `process_swarm_around 2007 0 2 13 781` and `… 801`;
on 782 `0/6`'s `unit_masks` 0 → 1034 and a `MOVEORDER` (flags 1) to (5736,
33096) with a `REPAIRORDER` (flags 4, `ox 2007`, `uid 13`) behind it; on
802 the same on `0/7`..`0/9`.

## run255 — chapter twenty-one, the repair line (2026-09-26, item 813)

**What it is.** `chapter21.cmd` whole — `!ai off`, `library` 2 and 3, a
who=0 Barracks, who=1 Bowmen, `peace 1` on 760, four citizens and two
issuer lines, `@repair 0 2007 0 6` on 780 and `@repair 0 2007 0 7 8 9` on
800 — dumped over `[605, 1300)`, 695 blocks, at `cover=0`. The lane was
free (run256's take 4 had exited).

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch21 \
    --map 14 --end-frame 1300 --log-window 605 1300 --timeout 3600 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter21.cmd
```

**One take.** 884 s launch to exit, 296 MB, `success: true`, 1301 frames.
`waitrun.sh` exited 0. **The pool printed**: 355,840 `GROUPDATA`, 512 on
each of the 695 blocks. It is the same game as run256's take 4 for the
831 frames they share (this crate walks both without a parting there).

**What it answered** (`docs/GOLDEN.md` §29; no falsifier fired):
`process_swarm_around 2007 0 2 13` on 781 and 801; on 782 `0/6` under a
`MOVEORDER` (flags 1) to (5736, 33096) and a `REPAIRORDER` (flags 4, `ox
2007`); on 802 `0/7`, `0/8` and `0/9` the same to (5688, 33048), (5880,
33048) and (5352, 32808); the Barracks 3 → 1 → 0 on 930 and 931, timber
261 → 260 on 931, `0/6`'s stack empty on 932; the trio's `REPAIRORDER`s
dying on 968, 969 and 1023 with nothing behind them; the citizens' own
`GATHERORDER`s on `0/2001` from 1083.

## run257 — East Indies' word 18938, sized to the word (2026-09-26, item 822)

**What it is.** run253's line, unchanged,
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, over
`[18933, 19190)`, at `cover=0`: a draw-stream trace. `!quit` at 19200,
through `viadriver.sh` with no human at the menu.

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 822
```

**Why it was owed.** Item 822 moved East Indies' word 18182 → 18938 (a
border fix zeroes `reg_known_rares` until the next census, `docs/AI.md`
§76). run253 ends on block 18433, so the word's block 18939 is on no
disk: every `*eastindies*` and `*islands*` dump was grepped by its block
range first. On 18938 ours spends 7 draws against 4, parting at index 1:
ours `Unit::do_guard+0x8fb`, the original `Guy::set_anim+0x104b`. **What
the disk could not answer**: which unit guards on 18938, and every unit,
figure and leader record on 18933..18939. **Sized to the word**
(DECISIONS 50 §7): six blocks before its block, the block, and 250 above
it, 257 blocks. No capture shares a block with it, so **18434..18932 is
compared by no dump**, and the same game is the draw stream's word
against run54.

**Taken whole.** 625.8 MB of dump and 24.9 MB of trace, 257 blocks
18933..19189. Launched 06:07 with the lane free (att-813's RonDriver had
exited), start clicked 06:08, archived 06:30. Waited on with
`waitrun.sh`, exit 0.

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run54.log` | **0 differing**, 19,201 identical, 19,201 in common |
| `MAP_STYLE` from the dump's `GAME INFO` | 18 |
| the window, block for block | **257 blocks, 18933..19189, no gap** |
| a `GROUPDATA` / a `GUY` on every window block | **257 / 257** |

**What it settled** (`run257_s_word_frame_is_widened_whole`,
`docs/AI.md` §76.7):
- **R1 holds**, by the checks above.
- **R2 holds, and on the word's own block.** Nothing parts on
  18934..18938. On 18939 four of who=1's units part whole: `1/67`,
  `1/68`, `1/70` and `1/71` stand under a `GUARD` (kind 12, one order,
  `tolerance` 144, stopped) here, and walk under an `ATTACK_TO` (kind 2,
  two orders, a one-leg path) there.
- **R3 is dead**: the four figures' clocks part, but with their orders,
  not alone.
- who=1's `known_rares` agrees on every block. The first block's floor
  (320 keys) adds who=1's `bucket[0:food]`, 114 against 214, from the gap.
- The floor is 320 keys on the first block, 103 under the word, 423 on
  its block and 1,265 in all.

## run261 — East Indies' gap over army 1's close (2026-09-26, item 829)

**What it is.** run253's line, unchanged,
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, over
`[18428, 18685)`, at `cover=0`: a draw-stream trace. `!quit` at 18695,
through `viadriver.sh` with no human at the menu.

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 829
```

**Why it was owed.** Item 829's first killer fired on run257's first
block. On 18933 `1/67`..`1/72` carry `group` −1 here against 71 there, a
row standing from the gap 18434..18932, which no dump compares (run253
ends on 18433). The disk answered what the original's group held on
18933: `[69, 72]` with `army −1`. **What it could not answer**: on which
tick army 1 closed, and whether its group survived the close. **Sized
over the gap** (the brief's step 6): six blocks before 18434 and 250 into
it, 257 blocks. Ours' close is tick 18682 (block 18683), inside. The
wagon's new army on tick 18692 is outside, and **18685..18932 stays
compared by no dump**.

**Taken whole.** 623.4 MB of dump and 24.5 MB of trace, 257 blocks
18428..18684. Launched 07:26, start clicked 07:27, archived 07:50.
Waited on with `waitrun.sh`, exit 0.

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run54.log` | **0 differing**, 18,696 identical, 18,696 in common |
| `MAP_STYLE` from the dump's `GAME INFO` | 18 |
| the window, block for block | **257 blocks, 18428..18684, no gap** |
| a `GROUPDATA` / a `GUY` on every window block | **257 / 257** |

**What it settled** (`run261_s_gap_is_widened_whole`, `docs/ARMY.md` §22):
- **R2 holds.** Group 71 lists all eighteen with `army 1` on 18682. On
  18683 it reads `army −1` with seventeen (`1/48` left through army 0's
  `push_group`), and army 0's new group 64 holds fifteen.
- **R3 holds**: the close is tick 18682, as ours' is.
- One key parts past the first block in 257 blocks: who=0's
  `production_step` on 18601. On 18683 army 0's list agrees, and every
  old-army `group` row is the same seat one id higher here (689). The
  floor is 318 keys on the first block, 1 past it, 316 on 18683 and 319
  in all.

## run265 — chapter twenty-two, the launch line (2026-09-26, item 836)

`docs/GOLDEN.md` §31, `tools/gamelog/golden/chapter22.cmd`: chapter
seventeen's cast and eleven lines, and **`766 @strike 0 2006 1 6`**, the
Fighter alone, inside its Airbase `0/2007`, at the Barracks `1/2006`,
with its tank at 24. The chapter and its falsifiers were committed
before the run (`4a28fa6a`). The lane lock named att-829's run261, whose
holder had exited; the launch took it over.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch22 \
    --map 14 --end-frame 1500 --log-window 605 1500 --timeout 3600 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter22.cmd
```

**One take** at `cover=0`: launched 08:26, **1,046 s** launch to exit,
**365 MB** of dump, 895 blocks. The receipt says `success: true`, exit
0, 1,501 frames, `MAP_STYLE 14`, seed 12345. `waitrun.sh` exited 0.
`cmdsran.py` has the nine console and chat lines returning 1.

| check | result |
|---|---|
| `rngcmp.py` vs run223's trace | the same game through 778 in its numbering, 784 frames identical; the first difference is its 779, the Fighter's first redraw (the harness's frame 778) |
| a `GROUPDATA` on every window block | **458,240**, 512 on each of the 895 |
| `SPECIALANIMORDER` anywhere | **none** |

### The issuer's own records

- Trace frame 766 (`0x2fe`): `INFO 18` for `0/6`, uid 14, at (11424,
  14005), inside; `INFO 17` line 12, refusal 0, package 10 → 40 bytes,
  one object.
- `process_group, new 0 1 767` and `process_flight 767`.

### §31's falsifiers

| check | predicted | observed |
| --- | --- | --- |
| 1, the issue | processed on 767 | as predicted |
| 2, the inside strike | block 768: one `STRAFEORDER` on `1/2006`, `mandatory 1`, flags 4, `AIRORDER oxx 2007 whose 0 cruising_alt 1600 returning 0`, `xx/yy` (21120, 16512), `inside_up 2007` | **exactly so** |
| 3, the tank's gate | out on 778, not 768 or 779 | **on 778**: `inside_up 2007` and the order on 768–777 while `mana_burn` falls 20 → 2; `inside_up −1` on 778, the block it first reads 0 |
| 4, the placement | 778: within 112 of (11424, 13920); `launch_frames` 0 on 778 and 1–15 on 779–793; no `SPECIALANIMORDER` | the figure at (11424, 13920), `last_z` 157 (the ground), `last_bank` and `last_pitch` 0.0, `angle 0`; the unit one step north at (11424, 13845); no `SPECIALANIMORDER`. **Fired on `launch_frames`**: 0 on 778 and **0 on every block after**, since the whole of `do_launch`, its counter included, is under `inside_down ≥ 0`, and the base is empty |
| 5, the strike | the strafe current from 778; a round of `0/6`'s on `1/2006` by 1178 | the strafe current 778–1080; **36 rounds** of `0/6`'s on `1/2006` from 924, a point of damage each; the Barracks gone on 1080 as in run223, and the strafe an `AIRPATROLORDER` from 1081 |
| 6, the tank | `mana_burn` 1 on 779, 400 on 1178, `returning 1` on 1178 | **exactly so**: the patrol's `returning` 1 on 1178 at (20182, 14744) |
| 7, home | inside `0/2007` by 1499 | **inside on 1385**, at (11426, 14018), its `AIRPATROLORDER` still on the stack; refilled 2 a block after it |

**Whether `Object::do_launch` executed**: yes, by the dump, not by a
coverage trace (run265 ran at `cover=0`, and no `cover=1` re-run was
taken, Loop 697): the base's `launch_frames` 15 → 0 and `inside_down` 6
→ −1 on 778, and the plane on the EXIT's own point on the same block.
The pair's tanks run dry on 1212 and 1214 as in run223, and both are
still flying home on 1480.

## run269 — East Indies' word 19413, sized to the word (2026-09-26, item 839)

**What it is.** run257's line, unchanged,
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, over
`[19408, 19665)`, at `cover=0`: a draw-stream trace. `!quit` at 19675,
through `viadriver.sh` with no human at the menu.

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 839
```

**Why it was owed.** Item 839 moved East Indies' word 19182 → 19413
(the British take the territory tax twice, `docs/ECONOMY.md` §16).
run257 ends on block 19189, so the word's block 19414 is on no disk:
every `*eastindies*` and `*islands*` dump was grepped by its block range
first. On 19413 ours spends 5 draws against 6, parting at index 3: the
original spends `Guy::set_anim+0x97a < Guy::do_turn+0x4a <
Guy::turn_towards+0x69`. **What the disk could not answer**: which
figure turns on 19413, and every unit, figure and leader record on
19408..19414. **Sized to the word** (DECISIONS 50 §7): six blocks before
its block, the block, and 250 above it, 257 blocks. No capture shares a
block with it, so **19190..19407 is compared by no dump**. The same game
is shown by the draw stream against run54.

**Taken whole.** 628.5 MB of dump and 25.3 MB of trace, 257 blocks
19408..19664. Launched 09:16. The lane lock was stale and no RonDriver
was running. Archived 09:40. It was waited on with `waitrun.sh`, which
exited 0.

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run54.log` | **0 differing**, 19,676 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 18 |
| the window, block for block | **257 blocks, 19408..19664, no gap** |
| a `GROUPDATA` / a `GUY` on every window block | **257 / 257** |

**What it settled** (`run269_s_word_frame_is_widened_whole`,
`docs/ECONOMY.md` §16.4):
- **R1 holds**, by the checks above.
- **R2 and R3**, on the word's own block. Nothing parts on
  19409..19413. On 19414, `1/77` (a two-figure unit inside `1/78`)
  differs on its second figure: `angle` −541917184 here against
  −901447680 there, and its pace (`avg_speed` 15/16 against 11/12,
  `last_speed[1]` 1 against 0). `1/78`'s `form` reads −1 against 0.
  Its orders and target agree, so R2 is dead. The figure's heading
  parts before any clock does, which is R3's shape but on the angle
  rather than the clock.
- The first block's floor (318 keys) carries two groups from the gap:
  who=1's twenty-nine citizens at `hits_left` 40 against 50, and
  `leftover[2:wealth]` eight sixteenths over. The floor is 318 on the
  first block, 5 under the word, 323 on its block and 1,433 in all.

## run277 — East Indies' word 20007, sized to the word (2026-09-26, item 857)

**What it is.** run269's line, unchanged,
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, over
`[20002, 20259)`, at `cover=0`: a draw-stream trace. `!quit` at 20269,
through `viadriver.sh` with no human at the menu.

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 857
```

**Why it was owed.** Item 857 moved East Indies' word 19509 → 20007
(the blocker probe is the quick form, `docs/COLLISION.md` §18, and the
trace names `Unit::do_guard+0x8fb`). run269 ends on block 19664, so the
word's block 20008 is on no disk: every `*eastindies*` and `*islands*`
dump was grepped by its block range first. On 20007 ours spends 8 draws
against 7, parting at index 0 on `1/67`'s `Guy::set_anim+0x97a <
Unit::move_step+0x823`. **What the disk could not answer**: why `1/67`
stands blocked on 20007, and every unit, figure and leader record on
20002..20008. **Sized to the word** (DECISIONS 50 §7): six blocks
before its block, the block, and 250 above it, 257 blocks. No capture
shares a block with it, so **19665..20001 is compared by no dump**. The
same game is shown by the draw stream against run54.

**Taken whole.** 627.4 MB of dump and 25.7 MB of trace, 257 blocks
20002..20258. Launched 11:23. The lane lock was stale and no RonDriver
was running. Archived 11:46. It was waited on with `waitrun.sh`, which
exited 0.

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run54.log` | **0 differing**, 20,270 identical (rngcmp numbers one ahead of the harness, parked 845) |
| `MAP_STYLE` from the dump's `GAME INFO` | 18 |
| the window, block for block | **257 blocks, 20002..20258, no gap** |
| a `GROUPDATA` / a `GUY` on every window block | **257 / 257** |

**What it settled** (`run277_s_word_frame_is_widened_whole`):
- **R1 holds**, by the checks above.
- **R3 holds, from the gap.** The first block's floor is 352 keys, and
  run269's standing set (`form`, `group`, who=1's citizens' hits, the
  leader rows) is 319 of them. The rest is new since 19664: who=1's army
  group, 68 there and 71 here, holds a group move whose slots `1/65` and
  `1/66` hold the other way round (`off` (504, 648) there against (696,
  456) here, and the reverse; `facing` 0 against 1 on `1/64`..`1/66`),
  so `1/65` walks to (35838, 42137) there and (36017, 41910) here.
- **R2 holds** on the word's block. `1/65`'s walk animation is 7 here
  against 8 there from 20005. On 20008 the original's `1/65` takes a half
  step and its `1/67` walks on to (34821, 40956), while ours is refused
  by `1/65` (`collide_o` 65) and stands.
- The floor is 352/21/371/948.

## run289 — East Indies' word 20782, sized to the word (2026-09-26, item 880)

**What it is.** run277's line, unchanged,
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, over
`[20777, 21034)`, at `cover=0`: a draw-stream trace. `!quit` at 21044,
through `viadriver.sh` with no human at the menu.

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 880
```

**Why it was owed.** Item 880 moved East Indies' word 20007 → 20782
(`Group::kill` clears an emptied pool record, and an army's group is
its slot's record, `docs/GROUPS.md` §30). run277 ends on block 20258,
so the word's block 20783 is on no disk: every `*eastindies*` and
`*islands*` dump was grepped by its block range first. On 20782 ours
spends 8 draws against 1, parting at index 0: ours
`Leader::use_market+0x1ed`, the original `Farms::inc_time+0x1ae`.
**What the disk could not answer**: why who=1's leader reaches
`use_market` on 20782, and every leader, unit, figure and pool record on
20777..20783. **Sized to the word** (DECISIONS 50 §7): six blocks before
its block, the block, and 250 above it, 257 blocks. No capture shares a
block with it, so **20259..20776 is compared by no dump**. The brief's
other booking for run289, a `GROUPS=1` capture over 6216..15893, was not
needed: the slot records agree on every dump without it (§30).

**Taken whole.** 630.5 MB of dump and 26.3 MB of trace, 257 blocks
20777..21033. Launched 17:32. The lane lock was stale (its pid gone) and
no RonDriver was running. Archived 17:57. It was waited on with
`waitrun.sh`, which exited 0.

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run54.log` | **0 differing**, 21,045 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 18 |
| the window, block for block | **257 blocks, 20777..21033, no gap** |
| a `GROUPDATA` / a `GUY` on every window block | **257 / 257** |

**What it settled** (`run289_s_word_frame_is_widened_whole`):
- **R1 holds**, by the checks above.
- **The first keys to part under the word are who=1's make list on
  20782**: `MAKE[0]`, `[1]` and `[8]` name city 1 here, valued 486, 398
  and 486, and are empty there. On 20783 ours places `1/2030`, which the
  original does not, sends `1/12` to build it, and spends timber, metal
  and wealth. Nothing parts on 20778..20781.
- **R2 stands on one row**: who=1's only resource row parting before
  the word is the standing `leftover[2:wealth]`, 2108 here against 2100
  (parked 851). **R3**: no clock row parts. No mechanism is named.
- The floor is 284/42/326/1,222.

## run281 — chapter twenty-three, the repeat line (2026-09-26, item 867)

`docs/GOLDEN.md` §32, `tools/gamelog/golden/chapter23.cmd`: chapter
twenty-two's cast and twelve lines, and **`1440 @buildmask 0 128 2007`**,
the repeat button on the Airbase `0/2007` alone, through the DLL's new
verb 17 (`CommandManager::issue_buildmask@00941f80`). By 1440 the Fighter
`0/6` has been inside since 1385 with its kept patrol; the Bombers `0/7`
and `0/8` are flying home. The chapter and its five falsifiers were
committed before the run (`fcb23eae`). No `RonDriver` was running and the
lane was free.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch23 \
    --map 14 --end-frame 1840 --log-window 605 1840 --timeout 3600 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter23.cmd
```

**One take** at `cover=0`: launched 12:39, **1,505 s** launch to exit,
**498 MB** of dump and 12 MB of trace, 1,235 blocks. The receipt says
`success: true`, exit 0, 1,841 frames, `MAP_STYLE 14`, seed 12345.
`waitrun.sh` exited 0. `cmdsran.py` has the eight console and chat lines
returning 1. The runner staged `37 !ffwd 3` where run265's staged 2; the
draw stream is the same game (below).

| check | result |
|---|---|
| `rngcmp.py` vs run265's trace | **0 differing, 1,501 identical**: the toggle, the landings and the kill draw nothing |
| a `GROUPDATA` on every window block | **632,320**, 512 on each of the 1,235 |
| `SPECIALANIMORDER` anywhere | **none** |

### The issuer's own records

- Trace frame 1440: `INFO 18` for `0/2007`, uid 13, at (11616, 13920);
  `INFO 17` line 13, refusal 0, package 10 → 24 bytes: the 5-byte
  `group` and the 9-byte `buildmask`, the emulator's 14.
- `process_group, new 0 1 1441` and `process_buildmask 1441`.

### §32's falsifiers

| check | predicted | observed |
| --- | --- | --- |
| 1, the issue | processed on 1441 | as predicted |
| 2, the toggle | `build_masks` 4104 on 1442 and to the end | **4232 → 4104 on 1442**, 4104 to 1839 |
| 3, a landing off the base | `0/7` inside on 1489 with no order; `0/8` likewise | **`0/7` inside on 1489, `orders []`**, `mana_burn` 600; **`0/8` inside on 1513**, `orders []`; neither has an order to its end |
| 4, the waiting patrol | `0/6`'s patrol kept to 1584, gone on 1585, `inside_up 2007` | **exactly so**: `AIRPATROLORDER` flags 0 on 1385–1584, `orders []` on 1585 (the block `mana_burn` first reads 0), still inside |
| 5, nothing launches | no aircraft out after 1442, no `SPECIALANIMORDER` | `inside_down` 6 on every block from 1385; `launch_frames` saturated at 15; `0/7` full on 1789 and `0/8` on 1813, and neither leaves |

**The pool.** The building group takes who=0's **slot 0** on 1442 (`num
1`, `buildings 1`, `stamp 1441`) and holds it to 1839. Slots 1 (`[6]`,
767) and 2 (`[7, 8]`, 661) are chapter twenty-two's, unchanged.

## run285 — chapter twenty-four, the queue line (2026-09-26, item 877)

`docs/GOLDEN.md` §33, `tools/gamelog/golden/chapter24.cmd`: chapter
thirteen's cast to 619 (a Barracks `0/2007` for who=0, a Chariot, a
Hoplite squad, `!ai off`), then **`620 @queueup 0 132 1 2007`** and
**`640 @queueup 0 170 1 2007`** (Hoplites, then Bowmen, through the DLL's
new verb 18, `CommandManager::issue_queue_up@00941be0`), **`900 @buildmask
0 64 2007`** (the infinite-queue button, between the two finishes) and
**`1300 @buildmask 0 64 2007`** (the same button on the empty queue). The
chapter and its eight falsifiers were committed before the run
(`a14e1e37`). No `RonDriver` was running and the lane was free.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch24 \
    --map 14 --end-frame 1560 --log-window 605 1560 --timeout 3600 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter24.cmd
```

**One take** at `cover=0`: launched 14:01, **1,164 s** in all (1,141
launch to exit), **403 MB** of dump and 11 MB of trace, 955 blocks. The
receipt says `success: true`, exit 0, 1,561 frames, `MAP_STYLE 14`, seed
12345. `waitrun.sh` exited 0. `cmdsran.py` has the six console and chat
lines returning 1.

| check | result |
|---|---|
| `rngcmp.py` vs run208's trace | the same game on its first 640 frames (the first frame it prints differing is 641; it numbers one ahead, parked 845): chapter thirteen's garrison is not in this script |
| a `GROUPDATA` on every window block | **488,960**, 512 on each of the 955 |

### The issuers' own records

- `INFO 18` for `0/2007`, uid 13, at (2688, 14208), on each of 620, 640,
  900 and 1300.
- `INFO 17`, refusal 0 on all four: the package 10 → 24 bytes on 620 (a
  fresh 5-byte `group` and the 9-byte `queue_up`, the emulator's 14) and
  10 → 22 on 640, 900 and 1300 (the 3-byte reuse and 9).
- `process_group, new 0 1 621` and `process_queue_up 132 1 621`;
  `process_group, repeat 0 0 641` and `process_queue_up 170 1 641`;
  `process_group, repeat 0 0 901` and `process_buildmask 901`; the same on
  1301.

### §33's falsifiers

| check | predicted (this crate's walk) | observed |
| --- | --- | --- |
| 1, the issues | processed on 621, 641, 901, 1301 | as predicted |
| 2, the queue-up | `[132]` at 100 on 622, `[132, 170]` on 642 | **as predicted**; food 253 → 203, timber 239 → 203 on 622; timber → 161, wealth 113 → 61 on 642 |
| 3, a finish with the bit clear | `[170]` alone on 848 | **`[170]` alone on 856**, `build_masks` 4096: the Hoplites ran to 23310 (a final step of 10) on 855 |
| 4, the toggle | 4160 on 902 | **4096 → 4160 on 902** |
| 5, a finish with the bit set | `[170]` at 0 on 1052, paid again, 4160 | **on 1060**: the Bowmen ran to 20280 on 1059; `[170]` at 0, timber 189 → 143 and wealth 70 → 14, **4160** |
| 6, a refused re-queue | `[]` and 4096 on 1264, wealth unspent | **on 1272**: the second Bowmen ran to 21030 on 1271; `[]`, **4096**, wealth 19 unspent |
| 7, the gate on an empty queue | 4096 on 1302 | **4096** |
| 8, the finish trains | a squad born on each finish | **three Hoplites `0/10`–`0/12` on 856, three Bowmen `0/13`–`0/15` on 1060 and `0/16`–`0/18` on 1272**, each squad in a pool group (0, 2, 1) |

**No falsifier fired.** The reading's three arms hold as read. The
staging walk had each finish 8 blocks early; ~~a clock this crate runs
short~~ with the commands entered the crate agrees on every block
(`docs/GOLDEN.md` §33), so the 8 blocks were the scratch walk's own. The
second Bowmen's target is 750 above the first's.

## run292 — chapter twenty-five, the cancel line (2026-09-26, item 884)

`docs/GOLDEN.md` §34, `tools/gamelog/golden/chapter25.cmd`: chapter
thirteen's cast to 619 (a Barracks `0/2007` for who=0, a Chariot, a
Hoplite squad, `!ai off`), then **`620 @queueup 0 132 2 2007`**, **`640
@queueup 0 170 1 2007`**, **`700 @unqueue 0 0 2007`** (arm a), **`760
@unqueue 0 0 2007`** (arm b), **`800 @buildmask 0 64 2007`**, **`840
@unqueue 0 -1 2007`** (arm c), **`980 @queueup 0 132 1 2007`**, **`990
@queueup 0 170 1 2007`** and **`1000 @unqueue 0 -1 2007`** (arm d), the
cancels through the DLL's new verb 19, `CommandManager::issue_unqueue@
00942c40`. The chapter and its nine falsifiers were committed before the
run (`5fd7e0cc`). No `RonDriver` was running and the lane was free.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch25 \
    --map 14 --end-frame 1466 --log-window 605 1466 --timeout 3600 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter25.cmd
```

**One take** at `cover=0`: launched 18:38, **1,099 s** in all (1,078
launch to exit), **359 MB** of dump and 11 MB of trace, 861 blocks. The
receipt says `success: true`, exit 0, 1,467 frames, `MAP_STYLE 14`, seed
12345. `waitrun.sh` exited 0. `cmdsran.py` has the six console and chat
lines returning 1.

| check | result |
|---|---|
| `rngcmp.py` vs run285's trace | the same game to 855 (the first frame it prints differing is 856, one ahead, parked 845): run285's first Hoplite is born there and this run cancelled it |
| a `GROUPDATA` on every window block | **440,832**, 512 on each of the 861 |

### The issuers' own records

- `INFO 18` for `0/2007`, uid 13, on each of the nine issuer frames.
- `INFO 17`, refusal 0 on all nine. The package grows 10 → 25 on 700,
  760, 840 and 1000: **15 bytes, no group**, the emulator's `unqueue`. It
  grows 10 → 24 on 620 (a fresh group and the `queue_up`) and 10 → 22 on
  640, 800, 980 and 990.
- `process_queue_up 132 2 621`, `170 1 641`, `132 1 981` and `170 1 991`;
  `process_buildmask 801`; **`process_unqueue 2007 0 701`, `2007 0 761`,
  `2007 -1 841`, `2007 -1 1001`** (`o`, `p`, frame).

### §34's falsifiers

| check | predicted (this crate's walk) | observed |
| --- | --- | --- |
| 2, the queue-up | `[132 at 100, 132]` paid 51/38 and 53/41 on 622; `[132, 132, 170]` on 642 | **as predicted**; food 254 → 150, timber 241 → 162 on 622; timber → 117, wealth → 58 on 642 |
| 3, a cancel inside a run | `[132 at 8100, 170]`, food +53, timber +41 on 702 | **as predicted**: food 157 → 210, timber 121 → 162 |
| 4, a cancel across two types | `[170 at 100]`, food +51, timber +38 on 762 | **as predicted**: the head at 13900 on 760 gone; food 216 → 267, timber 166 → 204 |
| 6, a single cancel on 4160 | 4160 on 802; 4096 and `[170 at 8100]`, nothing refunded, on 842 | **as predicted** |
| 7, the bit gone at the finish | `[]`, 4096, no re-queue on 965 | **as predicted**: the Bowmen at 20200 on 963, `[]` on 965 |
| 8, −1 is the last slot | `[132 at 2100]`, timber +46, wealth +56 on 1002 | **as predicted**: timber 133 → 179, wealth 10 → 66 |
| 9, the trains | the Hoplites' last run out on 1216 | `[132 at 23310]` on 1215, `[]` on 1216 |

**No falsifier fired**, and every predicted value is on its predicted
block: the staging walk went through the commands' own entries (parked
885). Falsifiers 1 and 5 and the births are read in `docs/GOLDEN.md` §34.

## run296 — chapter twenty-six, the research line (2026-09-26, item 883)

`docs/GOLDEN.md` §35, `tools/gamelog/golden/chapter26.cmd`: chapter
thirteen's cast to 619 (a Barracks `0/2007` for who=0, a Chariot, a
Hoplite squad, `!ai off`; who=0's own Library is `0/2005`), then
**`620 @queueup 0 572 2 2005`** (The Art of War, `num` 2), **`640 @queueup
0 572 1 2005`**, **`650 @queueup 0 551 1 2005`** (Written Word), **`850
@queueup 0 572 1 2005`** (held), **`860 @queueup 0 558 1 2005`** (Barter),
**`870 @queueup 0 132 1 2007`**, **`1040 @unqueue 0 0 2005`** and **`1060
@queueup 0 558 1 2005`**: a technology through the DLL's verb 18,
`CommandManager::issue_queue_up@00941be0`, which passes the type through.
The chapter and its eleven falsifiers were committed before the run
(`2e3bb156`). No `RonDriver` was running and the lane was free.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch26 \
    --map 14 --end-frame 1492 --log-window 605 1492 --timeout 3600 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter26.cmd
```

**One take** at `cover=0`: launched 20:08, **1,120 s** in all (1,098
launch to exit), **365 MB** of dump and 11 MB of trace, 887 blocks. The
receipt says `success: true`, exit 0, 1,493 frames, `MAP_STYLE 14`, seed
12345, 454,144 `GROUPDATA`. `waitrun.sh` exited 0. `cmdsran.py` has the
six console and chat lines returning 1.

| check | result |
|---|---|
| `rngcmp.py` vs run292's trace | the same game to 964: run292's Bowmen are born on 965, and this run trains nothing before 1106 |

### The issuers' own records

- `INFO 18` for `0/2005`, uid 5, on the seven Library frames, and for
  `0/2007`, uid 13, on 870.
- `INFO 17`, refusal 0 on all eight. The package grows 10 → 24 on 620 and
  870 (a fresh group and the `queue_up`), 10 → 22 on 640, 650, 850, 860
  (the group reused), and 10 → 25 on 1040 (the cancel, no group) and 10 →
  24 on 1060.
- `process_queue_up 572 2 621`, `572 1 641`, `551 1 651`, `572 1 851`,
  `558 1 861`, `132 1 871` and `558 1 1061`; `process_unqueue 2005 0 1041`.

### §35's falsifiers

| check | predicted (this crate's walk) | observed |
| --- | --- | --- |
| 2, one entry | `[572 at 100]`, 120 food on 622 | **as predicted**: food 254 → 134, `queued` 1 |
| 3, the gate | unchanged on 642 | **as predicted**: `[572 at 2100]`, food 136 unspent |
| 4, the second pass | `[572, 551 at 0]`, 120 timber 50 wealth on 652 | **as predicted**: timber 123, wealth 64 |
| 5, the finish | the entry out, epochs 1, Military 1, no unit on 822 | **as predicted**: `[572 at 20000]` on 821, `[551 at 0]` on 822; 56 units throughout |
| 6, held | unchanged on 852 | **as predicted**: `[551 at 3000]`, food 155 unspent |
| 7, the re-price | `[558 at 0]` 54/54, +6/+6 on 1023 | **as predicted**: food 59 → 65, timber 49 → 55 |
| 8, the cascade's count | `discovered_get()` 2 on 822, 3 on 1023 | **2 and 3**: Boadicea is counted |
| 9, the cancel | `[]`, +54/+54 on 1042 | **as predicted**: food 67 → 121, timber 57 → 111 |
| 10, pressed again | `[558 at 100]` 54/54 on 1062 | **as predicted**: food 68, timber 58 |
| 11, the unit and the last finish | the Hoplites out on 1106; Barter out on 1242 | **as predicted**: 56 → 59 units on 1106; epochs 3, `epoch_get` 1 0 1 1 on 1242 |

**No falsifier fired**, and every predicted value is on its predicted
block: the staging walk went through the command's own entry (parked
885).
## run294 — Great Lakes' departure, 19840..19999, a bisection of the gap (2026-09-26, item 795)

**What it is.** run243's line and lobby,
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, over
`[19840, 20000)`, at `cover=0`: a draw-stream trace. `!quit` at 20010,
through `viadriver.sh` with no human at the menu.

```
RON_LANE_WAIT=3600 zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 795
```

**Why it was owed.** Great Lakes' word was 20568 (item 785): the
original's `1/40` stands blocked by `8/0` and ours, three frames behind on
the same path, walks. The lag was whole on run243's first block, 20500.
Every `*greatlakes*` dump was grepped by its block range first (run226
16706..17350, run240 17086..17094, run243 20500..20818, run80 from
23960), and none holds 17351..20499. **What the disk did answer**: every
walker's last collision stamp and `1/40`'s `last` are ours, and on ours
the four leave the far point together on 19853..19883, a group move led
by `1/40`. **What it could not answer**: when the original's group move
ends, where each walker stands on the departure, and whether its `1/40`
walks the detour ours took after an ungroup on 19892. **Sized to the
departure, not to the gap**: 160 blocks, from the attack's end to eighty
blocks of plain walk. 17351..19839 is compared by no dump.

**Taken whole.** 360.7 MB of dump and 23.4 MB of trace, 160 blocks
19840..19999. Launched 19:00 with the lane lock stale (pid 71074 gone)
and no RonDriver running, archived 19:14. Waited on with `waitrun.sh`,
which exited 0.

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run53.log` | **0 differing**, 20,011 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 14 |
| the window, block for block | **160 blocks, 19840..19999, no gap** |
| a `GROUPDATA` and a `GUY` on every window block | **160** each |

**What it settled** (`run294_s_departure_is_widened_whole`,
`docs/GROUPS.md` §32).
- **R1 holds**: the checks above.
- **R2 holds.** On block 19893 the original's `1/40` still carries its
  group move (`MOVEORDER` with its `GROUPORDER`, `orig` (44851, 22480)).
- **R3 holds.** On block 19999 the original's `1/40` stands at
  (5462, 30256), and ours (before the fix) at (5254, 30259), eight frames
  behind: the loss is the departure's.
- **R4's killer fires.** The original's `1/40` passes (3732, 31380)
  itself, on block 19903: the detour is the original's first leg. The
  first parting is `1/40`'s waypoint on block 19876, (3912, 30984) here
  against (3732, 31380) there, with both stacks' tops at the detour. The
  cause is `do_move`'s TAKE, which reads the stack again after the line
  is verified (§32.3). With it built, nothing on `1/40`, `1/41` or `1/42`
  parts on run294 or run243.

## run304 — chapter twenty-eight, two buildings under one command (2026-09-26, item 888)

`docs/GOLDEN.md` §37, `tools/gamelog/golden/chapter28.cmd`: chapter
thirteen's cast with a second who=0 Barracks, **`608 add barracks who=0
22,74`** (`0/2008`), `!ai off`; then **`620 @queueup 0 170 1 2007`**,
**`640 @queueup 0 132 3 2007 2008`**, **`700 @buildmask 0 64 2008`**,
**`720 @buildmask 0 64 2008 2007`**, and **`740`** and **`760 @queueup 0
132 1 2007 2008`**, both refused on price. The chapter and its eleven
falsifiers were committed before the run (`d824e53d`). No `RonDriver`
was running and the lane was free.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch28 \
    --map 14 --end-frame 1580 --log-window 605 1580 --timeout 3600 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter28.cmd
```

**One take** at `cover=0`: launched 23:20, **1,231 s** in all (1,208
launch to exit), **428 MB** of dump and 12 MB of trace, 975 blocks. The
receipt says `success: true`, exit 0, 1,581 frames, `MAP_STYLE 14`, seed
12345, 499,200 `GROUPDATA`. `waitrun.sh` exited 0. `cmdsran.py` has the
seven console and chat lines returning 1, both `add barracks` among them.

| check | result |
|---|---|
| `rngcmp.py` vs run285's trace | the same game to 824: the second Barracks spends no draw, and the Bowmen's birth on 825 parts first |

### The issuers' own records

- `INFO 18` for `0/2007` (uid 13) and `0/2008` (uid 14), in the command's
  order, on every line.
- `INFO 17`, refusal 0 on all six. The package grows 10 → 24 on 620 and
  700 (a one-building group and the command), 10 → 26 on 640, 720 and 740
  (a group of two), and 10 → 22 on 760 (the three-byte reuse: the
  selection's ordered compare matched 740's).
- `process_group, new 0 1 621`, `new 0 2 641`, `new 0 1 701`, `new 0 2
  721`, `new 0 2 741`, **`repeat 0 0 761`**; `process_queue_up 170 1`,
  `132 3`, `132 1`, `132 1`, and `process_buildmask` on 701 and 721.

### §37's falsifiers

| check | predicted (this crate's walk, the seat by prototype) | observed |
| --- | --- | --- |
| 0, the staging | `0/2008` at (4224, 14208), 4096, on 610 | **as predicted** |
| 2, the single press | `0/2007` `[170 at 100]` 41/51; 254/200/62 on 622 | **as predicted** |
| 3, the sort and passes | `0/2008` 53/41 and 60/50, `0/2007` 56/45 and 65/56; 22/9 on 642 | **as predicted**: four entries, the fourth by 7 timber |
| 4, the toggle on one | `0/2008` 4160 on 702 | **as predicted** |
| 5, the toggle on two | 4096 and 4096 on 722 | **as predicted** |
| 6, the group of two | slot 0 `[2007, 2008]` stamp 641, slot 1 `[2007]` 621, on 642 | **as predicted**: `buildings 1`, `num` 2, `speed` 0 |
| 7, the next seat | slot 1 `[2008]` 701 on 702 | **as predicted** |
| 8, the order kept | slot 0 `[2008, 2007]` 721 on 722 | **as predicted** |
| 9, order is identity | slot 1 `[2007, 2008]` 741 on 742 | **as predicted** |
| 10, an equal group | slot 1 stamp 741 on 762 | **as predicted** |
| 11, the births | `group` 0, 1, 2, 3, 4 on 825, 876, 1067, 1126, 1324 | **as predicted**: stamps 824, 875, 1066, 1125, 1323 |

**No falsifier fired**, and every predicted value is on its predicted
block, the prototype's pool among them.

## run300 — chapter twenty-seven, the upgrade line (2026-09-26, item 901)

`docs/GOLDEN.md` §36, `tools/gamelog/golden/chapter27.cmd`: chapter
thirteen's cast to 614 and a Slinger squad `0/10` on 616, `!ai off`; who=0
(Nubian) staged Classical by **`600 age who=0 2`** and The Art of War by
**`602 military who=0 1`**; then **`620 @queueup 0 83 2 2007`**
(Javelineers, `num` 2), **`640 @queueup 0 83 1 2007`**, **`650 @queueup 0
82 1 2007`** (Slingers), **`1000 @queueup 0 82 1 2007`** and **`1010
@queueup 0 83 1 2007`**: a unit research through the DLL's verb 18. The
chapter and its ten falsifiers were committed before the run
(`35ba55a7`). No `RonDriver` was running and the lane was free.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch27 \
    --map 14 --end-frame 1560 --log-window 605 1560 --timeout 3600 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter27.cmd
```

**One take** at `cover=0`: launched 21:47, **1,214 s** in all (1,191
launch to exit), **404 MB** of dump and 12 MB of trace, 955 blocks. The
receipt says `success: true`, exit 0, 1,561 frames, `MAP_STYLE 14`, seed
12345, 488,960 `GROUPDATA`. `waitrun.sh` exited 0. `cmdsran.py` has the
nine console and chat lines returning 1, the two cheats among them.

| check | result |
|---|---|
| `rngcmp.py` vs run296's trace | the same game to 616: the cheats at 600 and 602 spend no draw, and the Slinger squad's birth on 617 parts first |

### The issuers' own records

- `INFO 18` for `0/2007`, uid 13, on all five frames.
- `INFO 17`, refusal 0 on all five. The package grows 10 → 24 on 620 (a
  fresh group and the `queue_up`) and 10 → 22 on 640, 650, 1000 and 1010
  (the group reused).
- `process_queue_up 83 2 621`, `83 1 641`, `82 1 651`, `82 1 1001` and
  `83 1 1011`.

### §36's falsifiers

| check | predicted (this crate's walk, the queue loop by prototype) | observed |
| --- | --- | --- |
| 0, the staging | 605: ages 2, epochs 1, `epoch[0]` 1, discovered 8, knowledge and metal 100 | **as predicted** |
| 2, arm a | `[83 at 100]` at `0/2007`, 80/80 once, on 622 | **as predicted**: food 254 → 174, timber 240 → 161 |
| 3, the gate | unchanged on 642 | **as predicted**: `[83 at 2100]`, 176/162 unspent |
| 4, arm c | `[83, 82 at 0]`, 46/46 on 652 | **as predicted**: food 131, timber 117 |
| 5, the finish | 922: the research out, `[83 at 0]` with 46/46 kept, buckets untouched | **as predicted**: `[83 at 30000, 82 at 0]` on 921, `[83 at 0]` 46/46 on 922, 155/135 both |
| 6, the reach | `0/10`..`0/12` type 83 on 922, `0/7`..`0/9` still 132 | **as predicted**: `myhits` 85 → 95; discovered 8 → 9 |
| 7, arm e | unchanged on 1002 | **as predicted**: 162/140 unspent |
| 8, arm f | `[83, 83 at 0]`, the second 46/46, on 1012 | **as predicted**: food 117, timber 95 |
| 9, the births | a Javelineers squad on 1111, the second on 1307 | **as predicted**: `0/13`..`0/15` on 1111, `0/16`..`0/18` on 1307, all type 83 |

**No falsifier fired**, and every predicted value is on its predicted
block, the prototype's among them.
## run299 — East Indies' word 23182, sized to the word (2026-09-26, item 890)

**What it is.** run289's line, unchanged,
`MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, over
`[23177, 23434)`, at `cover=0`: a draw-stream trace. `!quit` at 23444,
through `viadriver.sh` with no human at the menu.

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 890
```

**Why it was owed.** Item 890 moved East Indies' word 20782 → 23182
(`get_cost`'s wonder arm counts every wonder held or sited,
`docs/COSTS.md`, "A wonder is ramped by every wonder"). run289 ends on
block 21045 and run96 starts at 23960, so the word's block 23183 is on no
disk: every `*eastindies*` and `*islands*` dump was grepped by its first
and last block first. On 23182 ours spends 49 draws against 48, parting at
index 46: ours `Leader::produce_building+0x1805`, the original
`Leader::make_stuff+0x63d`. **What the disk could not answer**: why ours
draws one more placement jitter on 23182, and every leader, building,
unit and pool record on 23177..23183. **Sized to the word** (DECISIONS 50
§7): six blocks before its block, the block, and 250 above it, 257
blocks. **21046..23176 is compared by no dump.**

**Taken whole.** 631.5 MB of dump and 28.5 MB of trace, 257 blocks
23177..23433. Launched 20:35. The lane lock was stale (pid 87908 gone)
and no RonDriver was running. Archived 20:58. It was waited on with
`waitrun.sh`, which exited 0.

| check | result |
|---|---|
| `rngcmp.py` vs `rontrace-run54.log` | **0 differing**, 23,445 identical |
| `MAP_STYLE` from the dump's `GAME INFO` | 18 |
| the window, block for block | **257 blocks, 23177..23433, no gap** |
| a `GROUPDATA` / a `GUY` on every window block | **257 / 257** |

**What it settled** (`run299_s_word_frame_is_widened_whole`):
- **R1 holds**, by the checks above.
- **R2 holds, on a field and not the list's type**: who=1's `MAKE[0]` and
  `MAKE[4]` (the Farm, 417, 129,600, in Norwich) part on block 23182 on
  `escrow`, 0 here against 1 there. Built by item 890 (`docs/AI.md`
  §77.5): 1 on both.
- **R3 holds, narrowed**: with the list agreeing, both sides place the
  head's Farm `1/2031` at (34944, 38016) and walk slot 4's Farm through
  the same spiral. At its best site's 2×2 jitter, corner (184, 196), this
  crate finds four clear sub-positions and the original three. Both place
  `1/2032` at (35904, 38016); one expiry draw shifts, and on 23183 slot 4's
  `t` is 417 here against −1 there.
- **R4 is killed**: the only stock row parting is 851's standing
  `leftover[2:wealth]`, 4385 against 4377 on 23182.
- The floor is 283/5/288/1,045.

## run308 — chapter twenty-nine, the repeat launch (2026-09-27, item 915)

`docs/GOLDEN.md` §38, `tools/gamelog/golden/chapter29.cmd`: chapter
twenty-two's twelve lines, with **no** repeat toggle, and **`1540 @queueup
0 287 1 2007`**, one Biplane at the Airbase through the DLL's verb 18. The
chapter and its seven falsifiers were committed before the run
(`6a150069`). No `RonDriver` was running and the lane was free.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch29 \
    --map 14 --end-frame 2070 --log-window 605 2070 --timeout 5400 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter29.cmd
```

**One take** at `cover=0`: launched 00:53, **1,647 s** in all (1,617
launch to exit), **590 MB** of dump and 13 MB of trace, 1,465 blocks. The
receipt says `success: true`, exit 0, 2,071 frames, `MAP_STYLE 14`, seed
12345, 750,080 `GROUPDATA` (512 a block). `waitrun.sh` exited 0.

| check | result |
|---|---|
| `rngcmp.py` vs run265's trace | the same game on all 1,501 frames the two share |
| `rngcmp.py` vs run281's trace | the same game to 1585; the harness word 1586: run281's toggle on 1440 spends no draw, and run308's relaunch of `0/6` does |

### The issuer's own records

- `INFO 18` for `0/2007` (uid 13) on trace frame 1540 (`0x604`).
- `INFO 17` on 1540: the package grows 10 → 24 (a fresh one-building
  group and the `queue_up`).
- `process_group, new 0 1 1541` and `process_queue_up 287 1 1541`.

### §38's falsifiers

| check | predicted (this crate's walk) | observed |
| --- | --- | --- |
| 1, the issue | processed on 1541 | **as predicted** |
| 2, the press | `0/2007` one Biplane; metal 134 → 49, oil 100 → 15 on 1542 | **as predicted**: `queued` 1, the entry's `cost` 85/85, goods 2 and 5 |
| 3, the first relaunch | `0/6` out on 1585 at (11424, 13845), its `AIRPATROLORDER` flags 0, `returning` 0; counter 0 | **as predicted**: `inside_up` −1, `mana_burn` 0, `cruising_alt` 1400; the base's `inside_down` 6 → 7 |
| 4, the trained aircraft | the reading: inside, no order, `mana_burn` 0; this crate: out on (11424, 13920) | **the reading**: `0/9` born on 1746, `inside_up 8` (behind `0/8` in the chain), no order, `mana_burn` 0, at (11640, 13944), and so to 2069 (`inside_up 2007` once it heads the chain); `queued` 0 on 1747 |
| 5, the later relaunches | `0/7` on 1789 and `0/8` on 1813, each with its patrol, flags 0 | **as predicted**: out at (11424, 13860), the base's `inside_down` 7 → 8 → 9 |
| 6, the counter | 1..15 over 1814..1828, then 15 | **as predicted**: the chain holds `0/9` alone, and the counter saturates |
| 7, the tank flown out | `0/6` `mana_burn` 400, `returning` 1 on 1985 | **as predicted** |

**Falsifier 4's first two readings split, and the listing's won**: the
trained aircraft never comes out of an Airbase with no gather point
(`Build::train@0062f9b0`'s `CARRY_AIR` arm); parked 843's EXIT, which this
crate takes, is killed. **The others did not fire**: the launch half of
`do_launch`'s repeat arm (parked 876) flies each kept patrol out on its
first full tank. The Biplane's point is the base's plus (24, 24), not the
base's own.

## run312 — chapter thirty, the gather point (2026-09-27, item 928)

`docs/GOLDEN.md` §39, `tools/gamelog/golden/chapter30.cmd`: chapter
twenty-eight's cast, five `@gatherpoint` lines through the DLL's new verb
20 and three `@queueup` lines. The chapter and its six falsifiers were
committed before the run (`237108d1`, `089103c3`). The lane was taken
after att-923's run314 exited (its `waitrun.sh` exited 0 at 02:36); no
`RonDriver` was running at the launch.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch30 \
    --map 14 --end-frame 1450 --log-window 605 1450 --timeout 3600 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter30.cmd
```

**One take** at `cover=0`: launched 02:36, **1,031 s** in all (1,010
launch to exit), **367 MB** of dump and 11 MB of trace, 845 blocks. The
receipt says `success: true`, exit 0, 1,451 frames, `MAP_STYLE 14`, seed
12345, 432,640 `GROUPDATA` (512 a block). `waitrun.sh` exited 0. **The
DLL is the lane's own**: `unattended_capture.py` built it with
`-DRON_AUTOSTART` from this branch's `tools/trace/tracer.c` into the
output directory (sha256 `49ca567d…c582c1`); `game/rontrace.dll` was
neither read nor written.

| check | result |
|---|---|
| `rngcmp.py` vs run304's trace | the same game to 759; the harness word 760: the Citizen's first step, a move run304 never makes. No press spends a draw |

### The issuer's own records

- `INFO 17` refusal 0 on all eight lines. Each `@gatherpoint` grew the
  package 10 → 32 (a fresh one-building group and the 17-byte
  `gather_point`), and the Clear 10 → 30 (the three-byte reuse), as the
  emulator had it.
- `INFO 18` names `0/2007` (uid 13), `0/2000` (uid 0) and `0/2008` (uid
  14) at their points.
- `process_group, new 0 1 F` and `process_gather_point x y action 0 F` on
  617, 651, 701, 901 and 1101, with the lines' own arguments.

### §39's falsifiers

| check | predicted (the reading; this crate's walk in brackets) | observed |
| --- | --- | --- |
| 1, the issues | processed on 617, 651, 701, 901, 1101 | **as predicted** |
| 2, the lists | 618 2007 (1344, 12096, 0); 652 2000 (4224, 28608, 0), the Woodcutter's; 702 2008 (−1, −1, 0); 902 2007 (4224, 14208, 1) alone; 1102 2007 `length 0` | **as predicted**, every block. `metric` is 0 on every node, the node's key, not the action |
| 3, the first Hoplites (856) | `GROUPATTACKTOORDER`s under one group toward (1344, 12096), the captain out toward the point [south ring, no order] | **the reading**: `0/11` at (2328, 13656), `0/12` (2328, 13800), `0/13` (2232, 13512), north-west of 2007; each a `GROUPATTACKTOORDER` over a `MOVEORDER`, `group` 0, `orders_x/y` (1368, 12120); `angle` −385482752. An `ATTACKTOORDER` from 920, no order from 936, standing at (1368, 12120), (1512, 12024), (1224, 12216) |
| 4, 2008's Hoplites (953) | inside 2008, no order [out on the south ring] | **the reading**: `0/14` `inside_up 2008`, `0/15` and `0/16` behind it, no order, at 2008's point plus (24, 24); so to 1449 |
| 5, the Bowmen (1060) | a `GARRISONORDER` into 2008 [out at (2424, 14808), no order] | **the reading**: `0/17` out east at (3384, 14232), `0/18` (3336, 14520), `0/19` (3528, 14424), each a `GARRISONORDER` over a `TARGETORDER`, `group` 2, `angle` 1247346688; a `MOVEORDER` under it from 1061; inside 2008 on **1079**, behind `0/16` in the chain |
| 6, the Citizen (760) | a `MOVEORDER` toward (4224, 28608), no gather order [south of the City, no order, a `GATHERORDER` of its own on 919] | **the reading**: `0/10` out north-east at (3576, 29976), a `MOVEORDER`, `orders_x/y` (4248, 28632), `angle` 314048512; there by 880 and idle; a `GATHERORDER` of its own from **984** |

**No falsifier fired.** Every reading held: the command writes a
one-point list, replaced by the next press and emptied by the Clear; a
City's forest click takes its Woodcutter's point; a trainer's click on
itself keeps its trained squad inside; a squad is sent out toward the
point as one group, attack-moving; a squad whose point is a friendly
building garrisons it; and a citizen whose point is a building of its
own at action 0 walks there with a plain move and gathers later of its
own accord.
## run314 — chapter twenty-three under `cover=1` on the queue lane, and the issuers it refused (2026-09-27, item 923)

**What it is.** The blind list's first targeted capture (`docs/CENSUS.md`,
"The blind list, ranked"): `~/ron-golden/ch23`'s staged `rontrace.cmd`
line for line as `cmd:` lines, `cover=1`, no `callwin`, map 14 at seed
12345 with `cfg: -` (the golden record's lobby, not `check.ini`'s),
`end: MISC,UNITS=3,BUILDS=7,AMMO=5` over the whole run, `!quit` at 1840,
through `viadriver.sh … runqueue.sh - 314` with no human at the menu. The
stanza is in `tools/gamelog/captures.txt`; `ffwd: -` with the chapter's
own `37 !ffwd 3` as a line, because `longtrace.sh` writes its `FFWD`
before the stanza's lines and the channel would clamp `0 !ai off` to 37.

**Take one** (02:14) died before frame 0: `wine: Unhandled page fault on
read access to 00004ECD` — `docs/ORACLE.md`'s bop signature — after 2,101
`HIT`s, on the install's 2026-09-23 `rontrace.dll`, which predates the
chapter's verbs and had run `cover=1` whole at run226. `waitwin.sh` then
waits forever on a process that is gone, so the runner was stopped by
hand and `window.py restore` run; the trace and wine log are kept as
`rontrace-run314-take1-fault.log` and `wine-run314-take1-fault.log`. The
queue lane loads whatever DLL `build.sh` last left in `game/` and never
builds one; a plain build of the current `tracer.c` went in before take
two (the old one beside it, `rontrace.dll.2026-09-23-0102.bak`).

**Take two** (02:22–02:36) is the capture: 249 MB of dump, 9.7 MB of
trace, 1,841 blocks, `MAP_STYLE 14`, seed 12345, 6,649 functions entered.

| check | result |
|---|---|
| `cmdsran.py`: every cheat line on its frame | 9 of 9 ran, each returned 1 |
| `rngcmp.py` against ch23's `cover=0` trace | **0 differing to 642**, first differing 643 (harness word 642), 654 identical of 1,841 |
| `issue_flight` and `issue_buildmask` entered | **neither** |

**Every `@` line was refused, refusal 2.** Six `INFO 17` records, on 620,
640, 660, 664, 766 and 1440, each `b = 0x2000n`: the line index with
refusal 2 in the high half — the issuer's prologue is not the shipped one
(`tools/trace/tracer.c`, `issue_line`). The console-player check comes
first and passed on all six. `cover=1` writes a five-byte `jmp` to each
listed function's stub at attach, so every `CommandManager::issue_*`'s
first bytes are the instrument's, and the guard that protects a call
into an unexpected build refuses them all. **No issuer chapter can be
captured with coverage on any lane until the guard reads the displaced
bytes from the stub** (or the issuers leave the coverage table). The game
is the golden run's until the first order the refusal withheld: the 620
strike took no order in the golden runs either (`docs/GOLDEN.md` §25), and the
640 flight is the first that did.

**What it entered that nothing else does**: `Object::do_launch`, first on
606 (the staged Airbase), and `Leader::set_age`, on 600 (the `library`
cheat). The blind list 228 → 227; the census's entered column 7,664 →
7,666 over the corpus. `rondata::blind::TRACES` carries run314.

## run316 — British Isles with a Large Town, quit at frame 5: the blind list's row 4 (2026-09-27, item 935)

**What it is.** One launch for row 4 of `docs/CENSUS.md`'s ranked blind
list, on the queue lane at `cover=1`, through `viadriver.sh … runqueue.sh -
316` with no human at the menu: map style 22 (British Isles) through the
profile (`cfg: -`), the profile's `STARTING_TOWN` set to 3 (Large Town)
through the stanza's new `profile:` key, seed 12345, `ffwd: -`,
`end: MISC`, run10's start detail, `!quit` at 5. The reading behind the
staging is the census's "row 4 read to its callers": only two of the
nine functions are reachable from a solo lobby at all, and one lobby sets
both. The DLL was item 923's plain build of `tracer.c`, sha256
`17c5f215d8ea2775f020e3acc4a1b9de5e07bce83264f8afea837dec6cc798e2`
(parked 936).

**Taken** 03:04:57–03:08 in one take: 11.0 MB of gamelog, 9.1 MB of
trace, six `FRAME` records (frames 0–5), 6,518 `HIT`s. The run's own
`GAME INFO` reads `MAP_STYLE 22` and `STARTING_TOWN 3`, so both lobby
fields took; `longtrace.sh` copied `Player.dat` back afterwards
(`STARTING_TOWN 2`), and `mapstyle.py 14` put the profile's style back to
the 14 it was found at.

| check | result |
|---|---|
| `MAP_STYLE 22` in `GAME INFO` | yes |
| `STARTING_TOWN 3` in `GAME INFO` | yes |
| `Map::place_start_in_region` and `Setup::large_city_buildings` entered | **both**, before frame 0 |

The queue summary said `checks FAILED` on the take: the two `GAME INFO`
checks as first written anchored `grep -q "… 22$"`, and the gamelog's
lines end in `\r`. Both values were right on the page; the checks now
strip the `\r` and read the field with `awk`, pass on this archive, and
exit 1 when the expected value is wrong.

**What it entered that nothing else does**: `Setup::large_city_buildings`,
`Map::place_start_in_region`, `MapBritishIsles::make_continents` and
`Map::is_near_ocean`. The blind list 227 → 225; the census's entered column
7,666 → 7,670. `rondata::blind::TRACES` carries run316.

## run338 — chapter thirty-one, the gather point's other arms (2026-09-27, item 955)

`docs/GOLDEN.md` §40, `tools/gamelog/golden/chapter31.cmd`: chapter
thirty's cast and a second Chariot, six `@gatherpoint` lines, one
`@gatherpointadd` (verb 20 with `add_to_end` 1, item 955's), three
`@queueup` lines and one `@build`. The chapter and its seven falsifiers
were committed before the run (`494a883e`). The lane was taken after
att-934's queue (run319–run337) exited (its `waitrun.sh` exited 0 at
08:10); the lock named a dead pid and no `RonDriver` was running at the
launch.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch31 \
    --map 14 --end-frame 1400 --log-window 605 1400 --timeout 3600 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter31.cmd
```

**One take** at `cover=0`: launched 08:11, **1,287 s** in all (966
launch to exit, one relaunch at 300 s), **351 MB** of dump and 11 MB of
trace, 795 blocks. The receipt says `success: true`, exit 0, 1,401
frames, `MAP_STYLE 14`, seed 12345, 407,040 `GROUPDATA`. `waitrun.sh`
exited 0. **The DLL is the lane's own**, built from this branch's
`tools/trace/tracer.c` into the output directory: **sha256
`93ba3be03f28acecc7472418dc2948b1187169deaaf240283938fbc79957a98d`**
(parked 936); `game/rontrace.dll` was neither read nor written.

| check | result |
|---|---|
| `rngcmp.py` vs run312's trace | the same game to 615; the harness word 615, the second Chariot's `add` |

### §40's falsifiers

| check | predicted (the reading; the other in brackets) | observed |
| --- | --- | --- |
| 1, the issues | processed on 617, 621, 701, 703, 741, 841; the build on 791 | **as predicted**: `process_gather_point` with each line's arguments, 703's `add_to_end` 1; `process_build 7296 34176 7296 34176 521 2 791`; `process_queue_up` on 619, 623, 711 |
| 2, the lists and the site | 618, 622, 702, **704 two points in order**, 742, 842; the Lookout on 792 | **as predicted**, every block: 2008 `(5184, 12672, 0), (4992, 11136, 0)` from 704, printed head first; `0/2009` a Lookout (521), unfinished, from 792 |
| 3, the seeker (956) | exempt: `0/11` (4104, 31032), `0/12` (3576, 29928), `0/16` (3912, 31416) [counted: (3960, 31368), (3864, 30168), (3624, 31656)] | **fired, on all three**: `0/11` at (3960, 31368) on 718, `0/12` at (3864, 30168) on 824, `0/16` at (3624, 31656) on 938. The seeker is counted |
| 4, the gather arm (948) | a `GATHERORDER` on `0/2001` [none] | **the reading**, one shape off: `0/12`'s stack is a `GATHERORDER` alone, `ox 2001`, flags 4, `goto_build 1`, `orders_x/y` its own point. This crate puts a `MOVEORDER` to (4104, 28824) ahead of it |
| 5, the build arm (948) | a `BuildOrder` on `0/2009` [none] | **the reading**, the same shape off: `0/16`'s stack is a `BUILDORDER` alone, `ox 2009`, flags 0, `orders_x/y` its own point |
| 6, the third re-seat (957) | `0/13` at (2376, 14808) [(2472, 14856)] | **the reading**: `0/13` at (2376, 14808), `0/14` (2472, 15000), `0/15` (2328, 15000); a `GROUPATTACKTOORDER` each, group 2. **The target is not the prototype's**: `orders_x/y` (2424, 15672), `0/14`'s (2280, 15672), against (2232, 15528) and (2088, 15480) |
| 7, the two points (946) | two orders [one] | **the reading**: `0/17`..`0/19` each a `GROUPMOVEORDER` to (5208, 12696), (5352, 12792), (5064, 12600), then a `GROUPATTACKTOORDER` to (5016, 11160), (5160, 11160), (4872, 11160); group ids 952000 and 952001; `0/17` at (5016, 11160) on 1100 |

**Falsifier 3 fired**: `FILTER_ALL` counts the seeker, as the listing
reads it (`docs/GOLDEN.md` §40); item 945's exemption is dead. Every
other reading held, with two values the prototype did not predict: a
citizen's gather and build orders stand alone, and the squad's target
beside a unit is a squad placement's.
## run318 — chapter twenty-three under `cover=1`, the issuer guard fixed: every issuer called, 0 differing (2026-09-27, item 934)

**What it is.** run314's stanza line for line, on a DLL whose `@` issuer
guard reads the displaced bytes from the coverage table when the live
`jmp` lands on the entry's own stub (`tools/trace/issue_guard.h`;
`docs/CENSUS.md`, "item 934"). It is a plain build of `c37a99f5`'s
`tracer.c`, sha256
`b451aeb6d5c1a322033b84d900bfa4082d7afa0522e33a62119de5ec50e93dca`, and
the previous build is kept beside it as
`rontrace.dll.2026-09-27-0222-plain.bak` (parked 936). The lane lock was
stale (pid 727, run316's). The dead-take recovery (parked 937) was ready
and not needed: kill runqueue, longtrace, waitwin and the game, run
`window.py restore`, keep the trace as `-take1-fault`, retake once.

**Taken** 04:25:46–04:39 in one take: 247.6 MB of gamelog, 9.7 MB of
trace, 1,841 blocks.

| check | result |
|---|---|
| `cmdsran.py`: every cheat line on its frame | 9 of 9 |
| `issuesmatch.py` against run281 | the six `@` lines issued, refusal 0, each record as golden (package 10 → 42, 40, 42, 38, 40, 24) |
| `rngcmp.py` against run281 | **0 differing, 1,841 identical** |
| `issue_flight`, `issue_buildmask` entered | on 620 and 1440 |

**What it entered that nothing else does**: 15 of the blind list, the
flight line and the repeat mask (`docs/CENSUS.md`). The fix leaves the game
alone. `rondata::blind::TRACES` carries run318.

## run319 — chapter ten under `cover=1`, the golden game with coverage (2026-09-27, item 934)

`~/ron-golden/ch10/map-14/rontrace.cmd` line for line on run318's DLL and
lobby, `end: MISC,UNITS=3,BUILDS=7,AMMO=5,GROUPS=1`, `!quit` at 1250. It ran
in the one runner that took run319–run337 from 04:40, finished 04:50, and
was one take: 166.9 MB of gamelog. All four checks passed:
`cmdsran`, `issuesmatch` against run184 (2 `@` records, none refused),
`rngcmp` against run184 (**0 differing, 1,251 identical**), and each
issuer the script calls entered. New on the blind list's side: the patrol line, ten of the 77 and nine first (`issue_patrol` among them, DLL-only).
`rondata::blind::TRACES` carries run319.

## run320 — chapter eleven under `cover=1`, the golden game with coverage (2026-09-27, item 934)

`~/ron-golden/ch11/map-14/rontrace.cmd` line for line on run318's DLL and
lobby, `end: MISC,UNITS=3,BUILDS=7,AMMO=5,GROUPS=1`, `!quit` at 1250. It ran
in the one runner that took run319–run337 from 04:40, finished 04:59, and
was one take: 176.5 MB of gamelog. All four checks passed:
`cmdsran`, `issuesmatch` against run190 (3 `@` records, none refused),
`rngcmp` against run190 (**0 differing, 1,251 identical**), and each
issuer the script calls entered. New on the blind list's side: `issue_guard` (DLL-only) and `process_guard`.
`rondata::blind::TRACES` carries run320.

## run321 — chapter twelve under `cover=1`, the golden game with coverage (2026-09-27, item 934)

`~/ron-golden/ch12/map-14/rontrace.cmd` line for line on run318's DLL and
lobby, `end: MISC,UNITS=3,BUILDS=7,AMMO=5,GROUPS=1`, `!quit` at 1150. It ran
in the one runner that took run319–run337 from 04:40, finished 05:08, and
was one take: 154.1 MB of gamelog. All four checks passed:
`cmdsran`, `issuesmatch` against run204 (6 `@` records, none refused),
`rngcmp` against run204 (**0 differing, 1,151 identical**), and each
issuer the script calls entered. New on the blind list's side: the follow line, six (`issue_follow` DLL-only).
`rondata::blind::TRACES` carries run321.

## run322 — chapter thirteen under `cover=1`, the golden game with coverage (2026-09-27, item 934)

`~/ron-golden/ch13/map-14/rontrace.cmd` line for line on run318's DLL and
lobby, `end: MISC,UNITS=3,BUILDS=7,AMMO=5,GROUPS=1`, `!quit` at 1000. It ran
in the one runner that took run319–run337 from 04:40, finished 05:17, and
was one take: 135.3 MB of gamelog. All four checks passed:
`cmdsran`, `issuesmatch` against run208 (3 `@` records, none refused),
`rngcmp` against run208 (**0 differing, 1,001 identical**), and each
issuer the script calls entered. New on the blind list's side: the garrison line and eject, twelve of the 77 and eleven first (`issue_garrison` DLL-only).
`rondata::blind::TRACES` carries run322.

## run323 — chapter fourteen under `cover=1`, the golden game with coverage (2026-09-27, item 934)

`~/ron-golden/ch14/map-14/rontrace.cmd` line for line on run318's DLL and
lobby, `end: MISC,UNITS=3,BUILDS=7,AMMO=5,GROUPS=1`, `!quit` at 1150. It ran
in the one runner that took run319–run337 from 04:40, finished 05:27, and
was one take: 156.7 MB of gamelog. All four checks passed:
`cmdsran`, `issuesmatch` against run210 (3 `@` records, none refused),
`rngcmp` against run210 (**0 differing, 1,151 identical**), and each
issuer the script calls entered. New on the blind list's side: `issue_form` (DLL-only), `process_form`, `action_form`.
`rondata::blind::TRACES` carries run323.

## run324 — chapter fifteen under `cover=1`, the golden game with coverage (2026-09-27, item 934)

`~/ron-golden/ch15/map-14/rontrace.cmd` line for line on run318's DLL and
lobby, `end: MISC,UNITS=3,BUILDS=7,AMMO=5,GROUPS=1`, `!quit` at 1250. It ran
in the one runner that took run319–run337 from 04:40, finished 05:37, and
was one take: 170.0 MB of gamelog. All four checks passed:
`cmdsran`, `issuesmatch` against run215 (3 `@` records, none refused),
`rngcmp` against run215 (**0 differing, 1,251 identical**), and each
issuer the script calls entered. New on the blind list's side: `issue_attack`, `process_attack`.
`rondata::blind::TRACES` carries run324.

## run325 — chapter sixteen under `cover=1`, the golden game with coverage (2026-09-27, item 934)

`~/ron-golden/ch16/map-14/rontrace.cmd` line for line on run318's DLL and
lobby, `end: MISC,UNITS=3,BUILDS=7,AMMO=5,GROUPS=1`, `!quit` at 1250. It ran
in the one runner that took run319–run337 from 04:40, finished 05:47, and
was one take: 165.3 MB of gamelog. All four checks passed:
`cmdsran`, `issuesmatch` against run219 (4 `@` records, none refused),
`rngcmp` against run219 (**0 differing, 1,251 identical**), and each
issuer the script calls entered. New on the blind list's side: nothing no other trace enters.
`rondata::blind::TRACES` carries run325.

## run326 — chapter seventeen under `cover=1`, the golden game with coverage (2026-09-27, item 934)

`~/ron-golden/ch17/map-14/rontrace.cmd` line for line on run318's DLL and
lobby, `end: MISC,UNITS=3,BUILDS=7,AMMO=5,GROUPS=1`, `!quit` at 1400. It ran
in the one runner that took run319–run337 from 04:40, finished 06:00, and
was one take: 217.2 MB of gamelog. All four checks passed:
`cmdsran`, `issuesmatch` against run223 (4 `@` records, none refused),
`rngcmp` against run223 (**0 differing, 1,401 identical**), and each
issuer the script calls entered. New on the blind list's side: nothing run318 does not (the flight line again).
`rondata::blind::TRACES` carries run326.

## run327 — chapter eighteen under `cover=1`, the golden game with coverage (2026-09-27, item 934)

`~/ron-golden/ch18/map-14/rontrace.cmd` line for line on run318's DLL and
lobby, `end: MISC,UNITS=3,BUILDS=7,AMMO=5,GROUPS=1`, `!quit` at 1450. It ran
in the one runner that took run319–run337 from 04:40, finished 06:11, and
was one take: 198.6 MB of gamelog. All four checks passed:
`cmdsran`, `issuesmatch` against run241 (2 `@` records, none refused),
`rngcmp` against run241 (**0 differing, 1,451 identical**), and each
issuer the script calls entered. New on the blind list's side: `issue_build`, `process_build`, `action_build`, `GroupData::validate_build`.
`rondata::blind::TRACES` carries run327.

## run328 — chapter nineteen under `cover=1`, the golden game with coverage (2026-09-27, item 934)

`~/ron-golden/ch19/map-14/rontrace.cmd` line for line on run318's DLL and
lobby, `end: MISC,UNITS=3,BUILDS=7,AMMO=5,GROUPS=1`, `!quit` at 1100. It ran
in the one runner that took run319–run337 from 04:40, finished 06:20, and
was one take: 146.1 MB of gamelog. All four checks passed:
`cmdsran`, `issuesmatch` against run245 (1 `@` record, none refused),
`rngcmp` against run245 (**0 differing, 1,101 identical**), and each
issuer the script calls entered. New on the blind list's side: the cast line, five (`cast_double_agent` among them).
`rondata::blind::TRACES` carries run328.

## run329 — chapter twenty under `cover=1`, the golden game with coverage (2026-09-27, item 934)

`~/ron-golden/ch20/map-14/rontrace.cmd` line for line on run318's DLL and
lobby, `end: MISC,UNITS=3,BUILDS=7,AMMO=5,GROUPS=1`, `!quit` at 1300. It ran
in the one runner that took run319–run337 from 04:40, finished 06:31, and
was one take: 174.0 MB of gamelog. All four checks passed:
`cmdsran`, `issuesmatch` against run249 (6 `@` records, none refused),
`rngcmp` against run249 (**0 differing, 1,301 identical**), and each
issuer the script calls entered. New on the blind list's side: `issue_set_transport` (DLL-only), `process_set_transport`, `action_set_transport`.
`rondata::blind::TRACES` carries run329.

## run330 — chapter twenty-one under `cover=1`, the golden game with coverage (2026-09-27, item 934)

`~/ron-golden/ch21/map-14/rontrace.cmd` line for line on run318's DLL and
lobby, `end: MISC,UNITS=3,BUILDS=7,AMMO=5,GROUPS=1`, `!quit` at 1300. It ran
in the one runner that took run319–run337 from 04:40, finished 06:42, and
was one take: 189.4 MB of gamelog. All four checks passed:
`cmdsran`, `issuesmatch` against run255 (2 `@` records, none refused),
`rngcmp` against run255 (**0 differing, 1,301 identical**), and each
issuer the script calls entered. New on the blind list's side: the repair line, five (`issue_swarm_around`, `do_repair`, `RepairOrder::log_data` among them).
`rondata::blind::TRACES` carries run330.

## run331 — chapter twenty-four under `cover=1`, the golden game with coverage (2026-09-27, item 934)

`~/ron-golden/ch24/map-14/rontrace.cmd` line for line on run318's DLL and
lobby, `end: MISC,UNITS=3,BUILDS=7,AMMO=5,GROUPS=1`, `!quit` at 1560. It ran
in the one runner that took run319–run337 from 04:40, finished 06:54, and
was one take: 214.3 MB of gamelog. All four checks passed:
`cmdsran`, `issuesmatch` against run285 (4 `@` records, none refused),
`rngcmp` against run285 (**0 differing, 1,561 identical**), and each
issuer the script calls entered. New on the blind list's side: `issue_queue_up`, `process_queue_up`, `BuildData::can_infinite`.
`rondata::blind::TRACES` carries run331.

## run332 — chapter twenty-five under `cover=1`, the golden game with coverage (2026-09-27, item 934)

`~/ron-golden/ch25/map-14/rontrace.cmd` line for line on run318's DLL and
lobby, `end: MISC,UNITS=3,BUILDS=7,AMMO=5,GROUPS=1`, `!quit` at 1466. It ran
in the one runner that took run319–run337 from 04:40, finished 07:05, and
was one take: 199.1 MB of gamelog. All four checks passed:
`cmdsran`, `issuesmatch` against run292 (9 `@` records, none refused),
`rngcmp` against run292 (**0 differing, 1,467 identical**), and each
issuer the script calls entered. New on the blind list's side: the building unqueue, four (`issue_unqueue` DLL-only, `Build::action_unqueue`, `unpay_cost`).
`rondata::blind::TRACES` carries run332.

## run333 — chapter twenty-six under `cover=1`, the golden game with coverage (2026-09-27, item 934)

`~/ron-golden/ch26/map-14/rontrace.cmd` line for line on run318's DLL and
lobby, `end: MISC,UNITS=3,BUILDS=7,AMMO=5,GROUPS=1`, `!quit` at 1492. It ran
in the one runner that took run319–run337 from 04:40, finished 07:17, and
was one take: 200.5 MB of gamelog. All four checks passed:
`cmdsran`, `issuesmatch` against run296 (8 `@` records, none refused),
`rngcmp` against run296 (**0 differing, 1,493 identical**), and each
issuer the script calls entered. New on the blind list's side: nothing run332 does not.
`rondata::blind::TRACES` carries run333.

## run334 — chapter twenty-seven under `cover=1`, the golden game with coverage (2026-09-27, item 934)

`~/ron-golden/ch27/map-14/rontrace.cmd` line for line on run318's DLL and
lobby, `end: MISC,UNITS=3,BUILDS=7,AMMO=5,GROUPS=1`, `!quit` at 1560. It ran
in the one runner that took run319–run337 from 04:40, finished 07:29, and
was one take: 214.8 MB of gamelog. All four checks passed:
`cmdsran`, `issuesmatch` against run300 (5 `@` records, none refused),
`rngcmp` against run300 (**0 differing, 1,561 identical**), and each
issuer the script calls entered. New on the blind list's side: `Leader::track_queued`.
`rondata::blind::TRACES` carries run334.

## run335 — chapter twenty-eight under `cover=1`, the golden game with coverage (2026-09-27, item 934)

`~/ron-golden/ch28/map-14/rontrace.cmd` line for line on run318's DLL and
lobby, `end: MISC,UNITS=3,BUILDS=7,AMMO=5,GROUPS=1`, `!quit` at 1580. It ran
in the one runner that took run319–run337 from 04:40, finished 07:41, and
was one take: 227.0 MB of gamelog. All four checks passed:
`cmdsran`, `issuesmatch` against run304 (6 `@` records, none refused),
`rngcmp` against run304 (**0 differing, 1,581 identical**), and each
issuer the script calls entered. New on the blind list's side: nothing earlier runs do not.
`rondata::blind::TRACES` carries run335.

## run336 — chapter twenty-nine under `cover=1`, the golden game with coverage (2026-09-27, item 934)

`~/ron-golden/ch29/map-14/rontrace.cmd` line for line on run318's DLL and
lobby, `end: MISC,UNITS=3,BUILDS=7,AMMO=5,GROUPS=1`, `!quit` at 2070. It ran
in the one runner that took run319–run337 from 04:40, finished 07:58, and
was one take: 307.8 MB of gamelog. All four checks passed:
`cmdsran`, `issuesmatch` against run308 (6 `@` records, none refused),
`rngcmp` against run308 (**0 differing, 2,071 identical**), and each
issuer the script calls entered. New on the blind list's side: nothing run318 does not.
`rondata::blind::TRACES` carries run336.

## run337 — chapter thirty under `cover=1`, the golden game with coverage (2026-09-27, item 934)

`~/ron-golden/ch30/map-14/rontrace.cmd` line for line on run318's DLL and
lobby, `end: MISC,UNITS=3,BUILDS=7,AMMO=5,GROUPS=1`, `!quit` at 1450. It ran
in the one runner that took run319–run337 from 04:40, finished 08:10, and
was one take: 205.9 MB of gamelog. All four checks passed:
`cmdsran`, `issuesmatch` against run312 (9 `@` records, none refused),
`rngcmp` against run312 (**0 differing, 1,451 identical**), and each
issuer the script calls entered. New on the blind list's side: the gather point, four (`issue_gather_point`, `process_gather_point`, `action_gather_point`, `Build::add_gather_point`), and `BuildData::num_gather` and the gather-point list's `add`, but not `get_first_gather`.
`rondata::blind::TRACES` carries run337.

## run340 — a staged economy for the blind list's row 6: two of six entered (2026-09-27, item 959)

**What it is.** One launch for row 6 of `docs/CENSUS.md`'s ranked blind
list, on the queue lane at `cover=1`, through `viadriver.sh … runqueue.sh -
959`: the golden lobby (`cfg: -`, Great Lakes, seed 12345), `!ai off` and
`!ffwd 2` as the chapters run them, `!quit` at 2400, `end:
MISC,UNITS=3,BUILDS=7,CITIES=5,LEADERS=2,DEATHS=1`. The staging is the
census's "row 6 read to its callers": a Small City for who=1 at tile
(216,112), a Lumber Mill in Napata, Angkor Wat for who=0, a caravan for
who=1 (o 6), three who=0 Hoplites twenty tiles off its road, chapter
thirty's gather point on Napata, `die 8,0` on 700, `@attack 0 6 1 6` on
900, and `die 6,1` on 2000 as the kill's backstop. The DLL was run318's,
sha256 `b451aeb6d5c1a322033b84d900bfa4082d7afa0522e33a62119de5ec50e93dca`
(parked 936).

**Taken** 08:59:24–09:17 in one take: 357.4 MB of gamelog, 10.3 MB of
trace. Every chat line ran and returned 1, and both `@` lines issued with
refusal 0 (no golden twin, so the stanza reads `INFO 17` itself; the
check fails on run314's six refusals).

| function | entered | why, if not |
|---|---|---|
| `CityData::lumber_level` | 604 | |
| `Caravans::close_caravan` | 2000 | by the backstop `die`, not the attack |
| `LeaderData::resource_cap_add` | no | Angkor Wat (o 2008) landed at `city -1`, 23 tiles from Napata; `has_wonder` counts a wonder only in a city (`+0x72`) |
| `Build::replace_gather` | no | hoplite 8 died on 700, but a squad member's `Unit::close` skips the hotkey arm (`field_0x8e >= 0`); `HotKeyGroups::kill` first ran on 2000 |
| `Caravans::new_danger`, `Caravan::restart_trade_route` | no | the caravan routed (`do_trade`, `build_road` on 611), but hoplite 6's order stack is empty on 902: `valid_target_const` asks the target's `is_seen` for who=0, and the caravan was out of sight; nothing was damaged |

`rondata::blind::TRACES` carries run340.

## run341 — run340 retaken with its three misses fixed: four of six (2026-09-27, item 959)

**What it is.** run340's stanza with three changes, each one of run340's
reasons: Angkor Wat at corner (12,168), inside Napata's 20-tile radius;
`die 5,0` (a starting citizen, no squad) for `die 8,0`; and `explore all`
on 890, which sets `reveal_map` 3 so `UnitData::is_seen` answers yes. The
same DLL.

**Taken** 09:22:09–09:40 in one take: 355.9 MB of gamelog, 10.3 MB of
trace; every asserted chat line returned 1, both `@` lines issued with
refusal 0.

| function | entered |
|---|---|
| `LeaderData::resource_cap_add` | 606 |
| `CityData::lumber_level` | 604 |
| `Build::replace_gather` | 700 |
| `Object::take_damage` | 1167, the hoplites on the caravan |
| `Caravans::close_caravan` | 1458, the attack's kill |
| `Caravans::new_danger`, `Caravan::restart_trade_route` | **no** |

The listing's reading of why (`00652e97..00652f41`): on the caravan
owner's own or an ally's ground, `take_damage` calls `new_danger` only when
the object its arguments 7 and 8 name answers vtable `+0x1c` non-zero, which
`Build` and `Wall` do and `Unit` never does; the hoplites hit the caravan
on London's ground. run342 tried a Tower, and that reading did not hold
(below; `docs/CARAVAN.md` §10.4, amended).
`rondata::blind::TRACES` carries run341; the blind list 151 → 147 with
both.

## run342 — run341 and a Tower beside the caravan's road: `new_danger` still not entered (2026-09-27, item 959)

**What it is.** run341's stanza and one line, `add tower who=0 216,104` on
880, two to four tiles west of the road between London and the Small City;
a Tower has one base arrow, range 10. The number was reserved for 947 and
moved by the commander. The same DLL.

**Taken** 09:44:01–10:01 in one take; every asserted chat line returned 1,
both `@` lines issued with refusal 0. The Tower shot the caravan on
London's ground: its `damage` goes 0 → 12 on 913 and climbs by the Tower's
12 a shot, the hoplites joining from ~1030, until it dies on 1209
(`Caravans::close_caravan`, `Object::take_damage` first on 903). **Neither
`Caravans::new_danger` nor `Caravan::restart_trade_route` was entered**, so
the second arm's attacker is not simply the object that fired, and what
`Object::do_damage` passes as arguments 7 and 8 for an arrow is not
established; the first arm, a hit on unowned or enemy ground, is untried.
run342 enters nothing run341 does not; `rondata::blind::TRACES` carries it
so the pin records the attempt.


## run353 — Towers kill the caravan on its owner's road: `new_danger` and `restart_trade_route` entered (2026-09-27, item 965)

**What it is.** run342's staging without the lines its row had already
entered (the Lumber Mill, Angkor Wat, the gather point, `die 5,0`) and
without the hoplite and its `@attack`, so no Unit can deal the kill.
`take_damage` reaches the `new_danger` gate only on a killing blow
(`docs/CARAVAN.md` §10.4), and run342's was hoplite `0/7`'s. Four
`add tower who=0` lines on 880–886 at (216,104), (226,100), (226,106)
and (216,96), and `explore all` on 890. It has no `@` line, so run318's
DLL was served unstamped, sha256 `b451aeb6…3dca`. It ran at `cover=1`
on the queue lane.

**Taken** 17:39–17:52 in one take, after att-979's queue exited 0 on
`waitrun.sh`, with no `RonDriver` running and 28 GB free. Every chat line
returned 1. The dump is 240 MB and the trace 9.9 MB. The Towers are
`0/2007`–`0/2010`. Three of them fire (`2007`, `2008`, `2009`, a volley
every 31 frames from 904) and `0/2010` at (216,96) does not. They take
the caravan `1/6` to 12 on 919, 47 on 959 and 83 on 981, with
`damage_o` naming each Tower in turn. It stands at 82 of 90 on block
1007, and `DEATH_OBJS` prints it with `first_frame 1007`. No player-0
unit fires in 900–1010. **`Object::take_damage` is first entered on 902;
`Caravans::new_danger`, `Caravan::restart_trade_route` and
`Caravans::close_caravan` on 1007.** `rondata::blind::TRACES` carries it,
and the blind list goes 149 → 147.


## run344 — chapter thirty-two, an Airbase's gather point (2026-09-27, item 947)

`docs/GOLDEN.md` §41, `tools/gamelog/golden/chapter32.cmd`: chapter
twenty-nine's thirteen lines and three `@gatherpoint` lines on the Airbase
`0/2007` — P1 (11520, 7680) on 1600, P2 (5760, 5760) appended on 1700
(`@gatherpointadd`), the Clear on 1850. The chapter and its eight
falsifiers were committed before the run (`3996cc6b`). Launched after
att-971's queue exited (`waitrun.sh` 0 on its viadriver log), with no
`RonDriver` running and the profile's `DIFFICULTY` 0 in both blocks.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch32 \
    --map 14 --end-frame 2360 --log-window 605 2360 --timeout 5400 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter32.cmd
```

**One take** at `cover=0`: launched 14:55, **2,142 s** in all (2,111
launch to exit), **705 MB** of dump and 13 MB of trace, 1,755 blocks. The
receipt says `success: true`, exit 0, 2,361 frames, `MAP_STYLE 14`, seed
12345, 898,560 `GROUPDATA`, settings restored. The DLL is the lane's own
build from this branch's `tracer.c`, sha256 `8678f03f…d240`. `waitrun.sh`
exited 0. The dump's `GAME INFO` reads `DIFFICULTY 0`.

| check | result |
|---|---|
| `issuesmatch.py --none-refused … 9` | every `@` line issued, none refused |
| `rngcmp.py` vs run308's trace | the same game to 1751 (2,071 frames in common, 318 differ); the harness word 1751 — run308's Biplane stays in, run344's launches on 1747 |
| `process_gather_point` | `11520 7680 0 0` on 1601, `5760 5760 0 1` on 1701, `-1 -1 0 0` on 1851 |

### §41's falsifiers

| check | predicted (this crate's prototype) | observed |
| --- | --- | --- |
| 1, the issues | processed on 1601, 1701, 1851 | **as predicted** |
| 2, the lists | `[P1]` on 1602, `[P1, P2]` on 1702, empty on 1852 | **as predicted** |
| 3, the NEW press (1602) | `0/6`, `0/7`, `0/8` each one `AIRPATROLORDER` over P1, flags 4, `cruising_alt` 1600 | **as predicted**: `0/6` flying (1700 on 1601, 1600 on 1602, the strafe home's), `0/7` inside, `0/8` behind it (`inside_up 7`) |
| 4, the LAST press (1702) | each `[P1, P2]`, waypoint 0; `0/6`'s height its 1701 value | **as predicted**: both arrays length 2 on all three; `0/6` 1800 on 1701 and 1702 |
| 5, the trained plane | `0/9` born inside on `[P1, P2]`, flags 4; out on 1747 | **as predicted**: born on 1746 at (11640, 13944), `inside_up 8`, the patrol whole; on the map on 1747 at (11424, 13866) |
| 6, the walk | `0/6`'s waypoint 1 on 1741 | **as predicted**: 0 to 1740, 1 from 1741, turning on P2 |
| 7, the Clear (1852) | each plane a `STRAFEORDER`, `ox` −1, `returning` 1, flags 0, `mandatory` 0, `oxx` 2007 | **as predicted**, all four |
| 8, the landings | inside with no order: `0/8` 2034, `0/7` 2073, `0/6` 2102, `0/9` 2106 | **as predicted**, each on its block |

**No falsifier fired.** Every value the prototype predicted is the
original's on its block.
## run346 — run54's game at the lobby's top difficulty: the second pair's East Indies (2026-09-27, item 971)

**What it is.** The second pair's first capture (DECISIONS 53 §2): run54's
stanza with two lines moved, `cover=0` and `profile: DIFFICULTY=5`, on the
queue lane through `viadriver.sh … runqueue.sh - 971`. East Indies drops
`-config`, so the lobby is the profile's `<GAME_INFO>` block (`docs/AI.md`
§80.1); `rules.xml`'s six `difficulties` put Toughest at 5. The DLL was
run318's, sha256
`b451aeb6d5c1a322033b84d900bfa4082d7afa0522e33a62119de5ec50e93dca`,
unstamped (no `@` line, so `stamp.py` passes it).

**Taken** 14:22–14:26 in one take: 11.1 MB of gamelog, 38.3 MB of trace.
**The read-back**: the dump's `GAME INFO` block is run54's with one line
apart, `DIFFICULTY 5` against 0; the two `PLAYER` blocks are run54's line
for line (tribes 4 and 11, the per-player `diff 2`). **The game ended at
frame 18141**, not 24,000: the idle human is `defeated_by 1` on the closing
block, and the trace's last frame is 18140. The profile read `DIFFICULTY 0`
in both blocks before the take and again after it (the `PROFILE` hook's
copy-back; att-947's golden lane shares the file).

**Against run54**, as a measurement: `rngcmp.py` has 18,141 frames in
common and 18,138 differing, the first on frame 1 — frame 0 spends seven
draws more (182 against 175), so every later frame starts on another seed.
The harness's word was **0** (`diff::second`): ours 175 draws on frame 0,
parting at index 24, where the original spends `Unit::think_spellcaster+
0x413 < Unit::think_scout+0x7c`; it is **10** with that coin built
(`docs/AI.md` §80.5).

## run347 — run53's game at the lobby's top difficulty: the second pair's Great Lakes, in two takes (2026-09-27, item 971)

**What it is.** run53's stanza with `cover=0` and the difficulty, on the
queue lane beside run346, same DLL (run318's, sha256 `b451aeb6…3dca`).

**The first take refused the lever** (14:26–14:30): it kept `-config
check.ini` and set `difficulties=Toughest` there (the new `checkini:`
key, `tools/gamelog/checkini.py`) and `DIFFICULTY=5` in the profile, and
the dump read back **`DIFFICULTY 0`** — the stanza's own check failed.
`rngcmp.py` has it as run53's stream on all 24,001 frames, 0 differing. It
is kept as `gamelog-run347-greatlakes-take1-diff0-refused.txt` and
`rontrace-run347-take1-diff0.log`; `check.ini` and the profile were copied
back (`difficulties=Easiest`, `DIFFICULTY 0` in both blocks).

**The second take** (14:31–14:35) drops `-config` (`cfg: -`), as East
Indies always has: 10.8 MB of gamelog, 13.4 MB of trace. **The read-back**:
`GAME INFO` is run53's with one line apart, `DIFFICULTY 5` against 0; run53's
and run54's blocks differ only in `MAP_STYLE`, player blocks included, so
the profile's lobby is run53's but for the difficulty. **The game ended at
frame 5931**: the idle human `defeated_by 1`. The profile read `DIFFICULTY
0` in both blocks after the take.

**Against run53**, as a measurement: 5,931 frames in common, 5,930
differing, the first on frame 1 (frame 0 spends 121 draws against 120).
The harness's word was **0** (the same coin as run346's), and is **1**
with it built (`docs/AI.md` §80.5).

## run349 — run346's game at run299's detail over blocks 1..250: the second pair's East Indies word widened (2026-09-27, item 971)

**What it is.** The widening's capture for a frame-0 word: run346's lobby
(`cfg: -`, `profile: DIFFICULTY=5`, the default start detail), `end:
MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1` over
`frame_window: 0 251`, `cover=0`, `!quit` at 262. The word's frame 0 writes
block 1, so no block stands before it and 250 stand after.

**Taken** 14:36–14:55, one take, queued behind run347 by the runner reading
`captures.txt` as it went: 520.1 MB of gamelog, 9.9 MB of trace. `rngcmp.py`
against run346: 263 frames in common, **0 differing**; 250 window blocks;
`DIFFICULTY 5`, `MAP_STYLE 18`. The profile was copied back.

**What it holds**: `run349_s_word_frame_is_widened_whole` (`diff::second`),
from run346's start. Before the coin, 130 keys first parted on block 1 —
the AI scout `1/0`'s 28 rows among them; with it, 119, none of the scout's
before block 97 (`docs/AI.md` §80.5). East Indies' next word, frame 10, is
inside the window.

## run350 — run347's game at run349's detail over blocks 1..250: the second pair's Great Lakes word widened (2026-09-27, item 971)

**What it is.** run349's shape on run347's second take's lobby (`cfg: -`,
Great Lakes, `profile: DIFFICULTY=5`): `frame_window: 0 251`, `cover=0`,
`!quit` at 262. Taken for the lower word after `think_spellcaster`'s coin
was built — Great Lakes 1, whose frame writes block 2. Same DLL
(`b451aeb6…3dca`). Queued behind att-947's chapter thirty-two, which held
the lane 14:55–15:30, waited on with `waitrun.sh` on that lane's log.

**Taken** 15:31–15:49, one take: 466.3 MB of gamelog, 8.6 MB of trace.
`rngcmp.py` against run347: 263 frames in common, **0 differing**; 250
window blocks; `DIFFICULTY 5`. The profile read `DIFFICULTY 0` in both
blocks after it. Free disk 32 → 31 GB.

**What it holds**: `run350_s_word_frame_is_widened_whole`
(`diff::second`), from run347's start. Block 1 stands (58 keys, the first
pair's families); the word's block 2 parts on 130, who=1's opening
(`docs/AI.md` §80.5).

## run352 — run346's game at run349's detail over blocks 1571..1827: the second pair's East Indies word 1576 widened (2026-09-27, item 979)

**What it is.** run349's shape on run346's lobby (`cfg: -`, East Indies,
`profile: DIFFICULTY=5`), with `frame_window: 1571 1828`, `cover=0` and
`!quit` at 1840. It was taken after item 979's lobby gate moved East
Indies' word 10 → 1576 (`docs/AI.md` §81). The word's frame writes block
1577, with six blocks before it and 250 after. Same DLL (`b451aeb6…3dca`).
The lane lock was stale (run348's game had exited), and nothing else was
running.

**Taken** 16:59–17:19, one take: 542.5 MB of gamelog, 11.1 MB of trace.
- `rngcmp.py` against run346: 1841 frames in common, **0 differing**.
- 257 window blocks; `DIFFICULTY 5`, `MAP_STYLE 18`.
- Free disk 30 → 28 GB, across both captures.

**What it holds**: `run352_s_word_frame_is_widened_whole`
(`diff::second`), walked from run346's start.
- Block 1571 stands on 83 keys.
- Block 1576, before the word, parts on 5: `reg_land[11]`, and `2007`'s
  `city:space`.
- The word's block 1577 parts on 115: the new building `1/2009` (its
  gather tiles, damage 1 against 0, city −1 against 1) and citizen `1/2`'s
  walk.

## run355 — run347's game at run350's detail over blocks 3771..4027: the second pair's Great Lakes word 3776 widened (2026-09-27, item 979)

**What it is.** run350's shape on run347's lobby, with `frame_window:
3771 4028` and `!quit` at 4040. Run number given by the commander. It was
taken straight after run352 in the same queue.

**Taken** 17:20–17:39, one take: 509.3 MB of gamelog, 10.9 MB of trace.
- `rngcmp.py` against run347: 4041 frames in common, **0 differing**.
- 257 window blocks; `DIFFICULTY 5`, `MAP_STYLE 14`.

**What it holds**: `run355_s_word_frame_is_widened_whole`.
- Block 3771 stands on 78 keys, and nothing parts on 3772..3776.
- The word's block 3777 parts on 127: the new building `1/2010` (its
  gather tiles, damage 1 against 0), `2007`'s `city_down`, the city's
  `gatherers` 7 against 8, and citizens `1/6` and `1/28`.
## run357 — run346's game at run352's detail over blocks 5601..5857: the second pair's East Indies word 5606 widened (2026-09-27, item 989)

**What it is.** run352's shape on run346's lobby (`cfg: -`, East Indies,
`profile: DIFFICULTY=5`), with `frame_window: 5601 5858`, `cover=0` and
`!quit` at 5870. It was taken after item 989's close tail moved East
Indies' word 1576 → 5606 (`docs/AI.md` §82). The word's frame writes
block 5607, with six blocks before it and 250 after. Same DLL
(`b451aeb6…3dca`). The lane lock was stale (att-965's run354 game had
exited), and nothing else was running.

**Taken** 18:30–18:53 through `runqueue.sh - 989` under `viadriver.sh`,
waited on with `waitrun.sh`, one take: 576.0 MB of gamelog, 16.4 MB of
trace.
- `rngcmp.py` against run346: 5871 frames in common, **0 differing**.
- 257 window blocks; `DIFFICULTY 5`, `MAP_STYLE 18`.
- Free disk 30 → 27 GB, across both captures.

**What it holds**: `run357_s_word_frame_is_widened_whole`
(`diff::second`), walked from run346's start.
- Block 5601 stands on 159 keys.
- Block 5605, before the word, parts on one: `1/14`'s `order:move.dest`,
  0 against 1.
- The word's block 5607 parts on one more: `1/14`'s path.

## run356 — run347's game at run355's detail over blocks 4550..4806: the second pair's Great Lakes word 4555 widened (2026-09-27, item 989)

**What it is.** run355's shape on run347's lobby, with `frame_window:
4550 4807` and `!quit` at 4820. The number was reserved to item 989 for a
packet; no value question needed one, so it is this window. Taken
straight after run357 in the same queue.

**Taken** 18:53–19:12, one take: 517.4 MB of gamelog, 11.4 MB of trace.
- `rngcmp.py` against run347: 4821 frames in common, **0 differing**.
- 257 window blocks; `DIFFICULTY 5`, `MAP_STYLE 14`.

**What it holds**: `run356_s_word_frame_is_widened_whole`.
- Block 4550 stands on 86 keys.
- Block 4555, before the word, parts on 6, all `1/21`'s: order kind 10
  and two orders here against a move and one there.
- The word's block 4556 parts on 9: `1/21`'s walk.

## run348 — the held-out map measured again, in run33's shape (2026-09-27, item 972)

**Map 9, Himalayas, measured once more and read no further.** DECISIONS
53 §4 asked for this capture because run106's 1 was comparable to the
floors in kind and not in provenance (parked 372): it ran on the
click-free lane's own lobby and its window opened at frame 1. run106's
dump is not on disk any more, so the harness is the only thing this run
is set against. As with run106, no item is opened on it and nothing was
debugged against it. The number below comes from one invocation on
`4bf865b7`.

**How it was taken.** `longtrace.sh` through `runqueue.sh - 972` under
`viadriver.sh`, the stanza in `tools/gamelog/captures.txt`. Seed 12345,
run10's detail exactly (`[Start Game] WORLD=6 TERRAIN=2 GOODS=3 UNITS=3
BUILDS=7 CITIES=5 GUYS=2 LEADERS=9 DEATHS=1`, `[End Frame] MISC=1 UNITS=3
BUILDS=7 CITIES=5 GUYS=2 DEATHS=1 LEADERS=1`), the cheap window `[0,
1900)`, `5 !ffwd 30`, `1850 !quit`, `cover=1`, no input. All of these are
`longtrace.sh`'s defaults, so the stanza overrides only the map (`mapstyle:
9`, `cfg: -`) and `poll_max`. It ran in one take, from 16:27:25 to 16:38,
with the start clicked at 16:28:20 and the dump settled at 16:38:04.
The dump is 183,935,994 bytes with 1,851 blocks and the trace is
11,920,800 bytes. The DLL is sha256 b451aeb6…3dca, run318's, the build
run341 and run342 ran; `stamp.py` found no stamp and had nothing to refuse
on a stanza with no `@`. The disk had 31 GB free before the launch and 32
GB after (`df -g ~`, 97 per cent both times; other lanes write to it too).
The profile and `check.ini` were copied before the launch and put back
after it, once the lane was idle. `mapstyle.py` writes the profile's style
and nothing restores it by itself.

**The lobby is run39's, and `-config` could not have taken it.** run33's
literal lobby path, `-config check.ini`, pins the style at 14 whatever any
file says (`docs/ORACLE.md`, "The lobby is a file"), so on that path a
capture can only ever be Great Lakes. The lever that does take a map is
the profile's `<SETTINGS><MAP_STYLE>`, read by the lobby without
`-config`. That is the East Indies floor's own path (run39), and it is
`longtrace.sh`'s default for any style but 14. `DIFFICULTY` was 0 in
both of the profile's lobby blocks before the launch. The dump's own
`GAME INFO` confirms it: its 85 lines, both players, tribes and leaders
included, are **run39's line for line except `MAP_STYLE 9`**. Against
run33 they differ in `MAP_STYLE` and in two `Player` name lines. Seed
12345 and `DIFFICULTY 0` both read back.

**How this differs from run106 in provenance.** The lobby is the scored
runs' (run39's), where run106's was the click-free lane's (`-automation
+skipIntro`). The window and length are run33's and run39's, `[0, 1900)`
to `!quit` at 1850, where run106 ran 1,900 blocks over `[1, 1901]`. The
capture is `longtrace.sh`'s, at run33's detail and with a `cover=1`
trace. **One difference to the scored runs remains, and it is on the
harness side, not the capture's.** East Indies is scored with run38, a
`DUMP_ALL` start dump, as its sibling for the height table, and Great
Lakes with run12 and run13. Map 9 has no sibling, so the harness stands
this world up flat (`note: world: … no height table (flat)`). Himalayas
is the map where that matters most. Nothing here says whether it
decides the frame-1 parting, and by rule nothing was run to find out.

```
rondata <install> --gamelog gamelog-run348-himalayas-longtrace.txt \
        --trace rontrace-run348.log --diff        # on 4bf865b7

  note: player 1: personality rolled from the trace's Leader::init state 0xf4984e3b, landing on 0xcbe2f7af as the original did
  note: rng: seeded 0x477d3618 from the trace's last checkpoint (game.cpp 5024, CHECKSUM 1031)
  1851 frames stepped, 27723 unit-frames compared, 2122 unit-frames the sim has no unit for
  ticks before divergence: 1
  [ok] every starting citizen's derived GATHER target matches the one the original issued
       10 citizens, derived from §9.3 without reading the log: 0/1→2001 0/2→2001 0/3→2002 0/4→2003 0/5→2004 1/1→2001 1/2→2001 1/3→2002 1/4→2003 1/5→2004
  units that diverge, as who/o@frame — everything else tracked to the end: 0/1@490 0/2@432 0/3@237 0/4@103 0/5@103 1/0@2 1/1@2 1/2@2 1/3@103 1/4@103 1/5@103 1/6@114 1/7@209 1/8@323 1/9@1497 1/10@1626
  mylos: 27723 unit-frames compared, 2231 disagreements
    who 1 o 0: first at frame 202, ours 6, the log has 4 (1650 frames)
  order lists: 27723 unit-frames compared, 41488 order disagreements, 14065 path-stack disagreements
  ticks before an order diverges: 0
  by kind: coll 232, flags 5, gather 20606, kind 4251, length 3508, move 11821, path-field 236, path-length 4922, path-to 8907, target 1065
  who 1 o 0: first order disagreement at frame 1 — Move { field: "x", ours: 44280, theirs: 39672 }
  player 0: first divergence at frame 103 — unit o 4 ours (6552, 29064) theirs (6565, 29044)
  player 1: first divergence at frame 2 — unit o 0 ours (41913, 15772) theirs (41883, 15802)
```

**The number: ticks 1, orders 0**, against the floors `EastIndies
1851/1850` and `GreatLakes 1772/1772`. That is run106's number again.
**The first breaker is player 1's unit 0, the AI's scout: its move order's
`x` is 44,280 here against the log's 39,672 on frame 1**, and its position
parts on frame 2 (ours (41913, 15772), theirs (41883, 15802)). The log's
39,672 is run106's value too. This crate's side was 42,744 there. The
derived opening GATHER targets are again exact for all ten citizens. Nine
`unlinked` units from frames 1214, 1603 and 1838 are the AI's trained
units, 2,122 of the unit-frames.

**It parts inside its first 1,850 frames, on its first.** DECISIONS 53
§4 makes that fact the booking of a third *scored* map at the next pass:
a different map, with this one staying held out. The booking is the
pass's.

## run358 — chapter thirty-three, an Airbase's launch issuers (2026-09-27, item 976)

`docs/GOLDEN.md` §42, `tools/gamelog/golden/chapter33.cmd`: chapter
thirty-two's sixteen lines, an enemy Barracks by the Airbase (2200), and
five presses on `0/2007` — `@launchstrike` on it (2260), `@launchpatrol`
to P1 (2280), `@launchpatrolall` to P2 (2295), `@gatherpoint` of action 3
on the Citizen `0/1` (2305) and `@gatherpointadd` to P3 (2335). The
chapter and its nine falsifiers were committed before the run
(`cb60facd`). Launched at 19:13 after att-989's queue exited (its lock's
pid dead, no `RonDriver`), with the profile's `DIFFICULTY` 0 in both
blocks.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch33 \
    --map 14 --end-frame 2740 --log-window 605 2740 --timeout 6600 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter33.cmd
```

**One take** at `cover=0`: **2,344 s** in all (2,306 launch to exit),
**858 MB** of dump and 14 MB of trace, 2,135 blocks. The receipt says
`success: true`, exit 0, 2,741 frames, `MAP_STYLE 14`, seed 12345,
1,093,120 `GROUPDATA`, settings restored. The install's DLL was built from
this branch's `tracer.c` at `cb60facd` (sha256 `2518ed09…8adb`, stamped);
the lane's own build of the same source, the one that ran, is
`d8eeb6b7…e81b`. `waitrun.sh` exited 0. The dump's `GAME INFO` reads
`DIFFICULTY 0`.

| check | result |
|---|---|
| `issuesmatch.py --none-refused … 14` | every `@` line issued, none refused; each launch line grew the package by 28 bytes |
| `rngcmp.py` vs run344's trace | the same game to 2287 (2,361 frames in common, 73 differ) |
| the processed commands | `process_flight` on 2261; `process_launch_patrol 13440 9600 2 0 0 0` on 2281 and `7680 11520 1 1 0 0` on 2296; `process_gather_point 1 0 3 0` on 2306 and `9600 7680 0 1` on 2336 |

### §42's falsifiers

| check | predicted (this crate's prototype) | observed |
| --- | --- | --- |
| 1, the issues | processed on 2261, 2281, 2296, 2306, 2336 | **as predicted** |
| 2, the target's number | `1/2006` at (72, 78) | **as predicted**, from 2202 |
| 3, the strike (2262) | `0/8` a `STRAFEORDER` on `1/2006`, `mandatory` 1, flags 4, and out | **as predicted**: `returning` 0, `oxx` 2007, `xx/yy` (13824, 14976), on the map on 2262 |
| 4, the plain patrol (2282) | `0/9` over P1, flags 4, and out | **as predicted**: `0/7` fuelled and inside, `0/6` at `mana_burn` 40, untouched |
| 5, the shift-click (2297) | `0/7` and `0/6` over P2, flags 4 | **as predicted**: `0/6` at `mana_burn` 10 took it; out on 2306 and later |
| 6, action 3 (2307) | all four a `STRAFEORDER` on `0/1`, `mandatory` 1; `0/6` flags 4 and inside, the others 0 | **as predicted**; `xx/yy` (4104, 28392) |
| 7, the escort (2308..2336) | `xx/yy` on `0/1`, each plane flying | the strikes stand to 2336 with `xx/yy` on `0/1`, which does not move: the point is not split from a frozen one here; the positions are the widening's |
| 8, the append (2337) | each one `AIRPATROLORDER` over P3 alone, flags 4 | **as predicted**: the arrays of length 1, the strike gone on all four |
| 9, the arrivals | over P3 by 2739 | within `0x240` on 2409 (`0/6`), 2421 (`0/7`), 2470 (`0/9`), 2489 (`0/8`) |

**No falsifier fired.** Every order the prototype predicted is the
original's on its block.

## run362 — chapter thirty-four, the launch commands' other arms (2026-09-27, item 1009)

`docs/GOLDEN.md` §43, `tools/gamelog/golden/chapter34.cmd`: chapter
thirty-three's seventeen lines to its enemy Barracks (2200), a second
Airbase `0/2008` (2210), and nine presses — `@launchpatrolctrl` (2260),
`@launchstrikealt` (2275), `@launchmove` onto `0/2008` (2305),
`@launchstrikectrl` (2312), `[P1, A3, P2]` by `@gatherpoint` and two
`@gatherpointadd` (2320, 2335, 2350), the Clear (2400), and
`@launchpatrolalt` at `0/2008` (2600). The chapter and its twelve
falsifiers were committed before the run (`2ba9e3b1`). Launched 21:26:54
on the click-free lane: no `RonDriver`, the lane lock's pid (14816,
run358's) dead, 26 GB free.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch34 \
    --map 14 --end-frame 2850 --log-window 605 2850 --timeout 6600 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter34.cmd
```

**One take** at `cover=0`: **2,770 s** in all (2,731 launch to exit),
**906 MB** of dump and 14.5 MB of trace, 2,245 blocks. The receipt says
`success: true`, exit 0, 2,851 frames, `MAP_STYLE 14`, seed 12345,
1,149,440 `GROUPDATA`, settings restored. The lane's DLL was built from
this branch's `tracer.c` at `2ba9e3b1` (stamp `source ed4390b6…c4c5`, the
committed file's sha256; `rontrace.dll` `6ccb99b6…bc6c`). `waitrun.sh`
exited 0.

| check | result |
|---|---|
| `issuesmatch.py --none-refused … 18` | every `@` line issued, none refused; the nine new each a group and a command (28 bytes, 30 for the fresh group of `0/2008`) |
| `rngcmp.py` vs run358's trace | the same game to 2279 (2,741 frames in common, 460 differ, the first 2280) |
| the processed commands | `process_launch_patrol 13440 9600 2 0 1 0` on 2261 (ctrl) and `5760 13440 2 0 0 1` on 2601 (alt); `process_flight` on 2276, 2306 and 2313; `process_gather_point` on 2321, 2336, 2351 and 2401 |

### §43's falsifiers

| check | predicted (this crate's walk) | observed |
| --- | --- | --- |
| 1, the issues | processed on 2261, 2276, 2306, 2313, 2321, 2336, 2351, 2401, 2601 | **as predicted** |
| 2, the second base | `0/2008` at (8544, 14688) | **as predicted**; the 2600 press names it (uid 18) |
| 3, ctrl on the patrol (2262) | `0/8` over (13440, 9600), out | **as predicted**; `0/9` full inside, untouched |
| 4, alt on the strike (2277) | `0/9` a `STRAFEORDER` on `1/2006`, `mandatory` 1 | **as predicted**: `xx/yy` (13824, 14976); `0/7` full inside, untouched |
| 5, `MOVE_TO` (2307) | `0/7` a flight home to `0/2008`, out; `0/6` untouched | **as predicted**: `oxx` 2008, `returning` 1, `mandatory` 1, no target; `0/6` full inside with no order |
| 6, ctrl on the strike (2314) | nothing | **as predicted**: `0/6` inside, no order |
| 7, the ground point (2322) | `0/6`, `0/8`, `0/9` over P1, `0/7` not | **as predicted**; `0/6` out on 2323 |
| 8, A3 (2337) | three `STRAFEORDER`s on `0/1`, `mandatory` 1 | **as predicted**, `xx/yy` (4104, 28392) |
| 9, P2 (2352) | every stack as on 2351 | **as predicted**: the three strikes stand, one order each |
| 10, the Clear (2402) | three strafes home, `mandatory` 0; `0/7` untouched | **as predicted** |
| 11, alt at `0/2008` (2602) | nothing | **as predicted**: `0/7` inside `0/2008`, no order |
| 12, the arrival | `0/7` in `0/2008` on 2474 | **as predicted**: on the map on 2473, inside on 2474, full on 2558 |

**No falsifier fired.** Every order the walk predicted is the original's
on its block.
## run360 — Great Lakes at Toughest, a packet at tick 4506: the AI scout's choice (2026-09-27, item 1014)

**What it is.** A `RON_STATE_FRAME=4506` packet on the click-free lane over
run347's game (Great Lakes, `DIFFICULTY 5`). Blocks 4500–4511 are dumped at
run356's detail, and `!quit` is at 4515.
- It is the first second-pair capture on this lane. The lane gained
  `--profile KEY=N` for it (`unattended_capture.py`), the queue lane's
  `profile:` key. It writes both of `Player.dat`'s game blocks after
  staging, and the staged `PlayerProfile` backup restores them.
- The plan is item 597's, reused.
- The lane lock was stale: pid 84639 was dead, and nothing else ran.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-27-run360 \
    --map 14 --end-frame 4515 --timeout 2400 --log-window 4500 4512 \
    --detail end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1 \
    --tracer-def RON_STATE_FRAME=4506 \
    --tracer-def 'RON_STATE_PLAN="<plan>/plan.h"' \
    --profile DIFFICULTY=5
```

**Taken** 22:59–23:00, one take. `success: true`, exit 0, 67 s launch to
exit and 77 s in all. The packet is 821,504,932 bytes, and the take is
862 MB with its dump. Free disk went 24 → 22 GB.

| check | result |
|---|---|
| `rngcmp.py` against `rontrace-run347.log` | **0 differing**, 4,516 identical |
| `GAME INFO` | `DIFFICULTY 5`, `MAP_STYLE 14`, seed 12345 read back |
| `frame_snapshot.py` against the plan | logger return's roots unchanged, receipt checked |
| settings | five files restored, and `DIFFICULTY 0` back in the profile |

**Why 4506.** The AI scout `1/0` chooses its explore target during tick
4506, and block 4507 is the first to print it. Per `docs/EMULATOR.md` §8,
the packet for that decision is logger frame 4506.

**The booking cites what the disk could not answer.** run347 prints no
unit records. run355 ends at 4027 and run356 opens at 4550, and neither
prints the danger map or `treaties`. Every term that the disk did print
agreed: the scout's position, the buildings' `ever_seen`, and the 4400
rebuild's inputs. On those terms the original's formula picks ours.

**What it settled** (`docs/SCOUT.md` §8.3):
- Block 4506 has the scout at (4320, 28896), and block 4507 sends it to
  (2808, 32760).
- The packet's `danger[1]` equals this crate's around the human's city.
- `think_scout`, run on unit `1/0` under unicorn, scores the human's cells
  without the stranger's ×2, because `treaties[0] & 3` is set: the two
  leaders met on 4456.
- The probe enters `think_scout` directly, because `Game::do_frame`'s first
  call on this packet goes through a renderer object the packet does not
  hold.

The packet stays outside git at
`~/ron-data/lab-captures/2026-09-27-run360/map-14`. The probe and its output
are at `~/ron-data/lab-experiments/2026-09-27-item-1014/`.

## run364 — chapter thirty-one under `cover=1`, the golden game with coverage (2026-09-28, item 1011)

`~/ron-golden/ch31/map-14/rontrace.cmd` line for line on the queue lane,
the golden lobby (`cfg: -`), the chapter's own `!ffwd`,
`end: MISC,UNITS=3,BUILDS=7,AMMO=5,GROUPS=1`, `!quit` at 1400, on the DLL
built from att-1011's tree (`tracer.c` `ed4390b6…c4c5`, 1009's;
`rontrace.dll` `d654cdb2…599d`, stamped; the coverage table and trace
executable byte-identical to the build before). It ran in the one runner
that took run364–run367 from 23:33 (`runqueue.sh - 1011`), started 23:34,
settled 23:44, one take: 201.6 MB of gamelog. All four checks passed:
`cmdsran`, `issuesmatch` against run338 (10 `@` records, none refused),
`rngcmp` against run338 (**0 differing, 1,401 identical**), and each
issuer the script calls entered: `issue_gather_point` (616), `issue_queue_up` (618), `issue_build` (790). New on the blind list's side:
nothing the other traces do not. `rondata::blind::TRACES` carries run364.

## run365 — chapter thirty-two under `cover=1`, the golden game with coverage (2026-09-28, item 1011)

`~/ron-golden/ch32/map-14/rontrace.cmd` line for line on the queue lane,
the golden lobby (`cfg: -`), the chapter's own `!ffwd`,
`end: MISC,UNITS=3,BUILDS=7,AMMO=5,GROUPS=1`, `!quit` at 2360, on the DLL
built from att-1011's tree (`tracer.c` `ed4390b6…c4c5`, 1009's;
`rontrace.dll` `d654cdb2…599d`, stamped; the coverage table and trace
executable byte-identical to the build before). It ran in the one runner
that took run364–run367 from 23:33 (`runqueue.sh - 1011`), started 23:45,
settled 00:03, one take: 346.0 MB of gamelog. All four checks passed:
`cmdsran`, `issuesmatch` against run344 (9 `@` records, none refused),
`rngcmp` against run344 (**0 differing, 2,361 identical**), and each
issuer the script calls entered: `issue_flight` (620), `issue_queue_up` (1540), `issue_gather_point` (1600). New on the blind list's side:
nothing the other traces do not. `rondata::blind::TRACES` carries run365.

## run366 — chapter thirty-three under `cover=1`, the golden game with coverage (2026-09-28, item 1011)

`~/ron-golden/ch33/map-14/rontrace.cmd` line for line on the queue lane,
the golden lobby (`cfg: -`), the chapter's own `!ffwd`,
`end: MISC,UNITS=3,BUILDS=7,AMMO=5,GROUPS=1`, `!quit` at 2740, on the DLL
built from att-1011's tree (`tracer.c` `ed4390b6…c4c5`, 1009's;
`rontrace.dll` `d654cdb2…599d`, stamped; the coverage table and trace
executable byte-identical to the build before). It ran in the one runner
that took run364–run367 from 23:33 (`runqueue.sh - 1011`), started 00:04,
settled 00:24, one take: 399.5 MB of gamelog. All four checks passed:
`cmdsran`, `issuesmatch` against run358 (14 `@` records, none refused),
`rngcmp` against run358 (**0 differing, 2,741 identical**), and each
issuer the script calls entered: `issue_flight` (620), `issue_queue_up` (1540), `issue_gather_point` (1600), `issue_launch_patrol` (2280). New on the blind list's side:
`Group::action_launch_flight` (2260), `Group::action_launch_patrol` and `CommandManager::issue_launch_patrol` (2280), off `NEVER`, and five uncited (`docs/CENSUS.md`, "item 1011"). `rondata::blind::TRACES` carries run366.

## run367 — chapter thirty-four under `cover=1`, the golden game with coverage (2026-09-28, item 1011)

`~/ron-golden/ch34/map-14/rontrace.cmd` line for line on the queue lane,
the golden lobby (`cfg: -`), the chapter's own `!ffwd`,
`end: MISC,UNITS=3,BUILDS=7,AMMO=5,GROUPS=1`, `!quit` at 2850, on the DLL
built from att-1011's tree (`tracer.c` `ed4390b6…c4c5`, 1009's;
`rontrace.dll` `d654cdb2…599d`, stamped; the coverage table and trace
executable byte-identical to the build before). It ran in the one runner
that took run364–run367 from 23:33 (`runqueue.sh - 1011`), started 00:25,
settled 00:44, one take: 418.4 MB of gamelog. All four checks passed:
`cmdsran`, `issuesmatch` against run362 (18 `@` records, none refused),
`rngcmp` against run362 (**0 differing, 2,851 identical**), and each
issuer the script calls entered: `issue_flight` (620), `issue_queue_up` (1540), `issue_gather_point` (1600), `issue_launch_patrol` (2260). New on the blind list's side:
nothing run366 does not. `rondata::blind::TRACES` carries run367.

## run371 — chapter thirty-five, the Helicopter's and missiles' launch arms (2026-09-28, item 1019)

`docs/GOLDEN.md` §44, `tools/gamelog/golden/chapter35.cmd`: chapter
thirty-four's eighteen lines to its second Airbase (2210), then
`resource who=0 all +500` (2215), a Missile Silo `0/2009` (2220), two V2
Rockets queued there (2225) and two Helicopters at `0/2008` (2240),
`@launchpatrol` at the silo (2440), a point at each base (2460, 2500),
`@launchmove` and `@launchstrike` from the silo (2650, 2670), the Clear
at `0/2008` (2800), and `@launchstrike` and `@launchpatrol` there (3000,
3010). The chapter and its thirteen falsifiers were committed before the
run (`55087201`).

**Two takes at `cover=1` hung before frame 0** (01:54 and 03:08, the
click-free lane): 96 KB of trace, no `FRAME` record, run185's hang. The
runner's 300 s stall guard killed each healthy first launch — its
`wine-stalled.log` is run362's whole 29,306 bytes — and the relaunch sat
at the menu. **81 minutes of the lane.** Both takes are kept aside as
`~/ron-golden/ch35.dead-20260928-0154` and `-0308`.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch35 \
    --map 14 --end-frame 3260 --log-window 605 3260 --timeout 7200 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter35.cmd
```

**The take** at `cover=0`, 03:15:41: **3,256 s** in all (3,214 launch to
exit), **1.08 GB** of dump and 15.1 MB of trace, 2,655 blocks. The
receipt says `success: true`, exit 0, 3,261 frames, `MAP_STYLE 14`, seed
12345, 1,359,360 `GROUPDATA`, settings restored. The lane's DLL was built
from this branch's `tracer.c` (stamp `source ed4390b6…c4c5`, `defs
-DRON_AUTOSTART`; `rontrace.dll` `6bbc15b5…0990`). `waitrun.sh` exited 0.
19 GB free after.

| check | result |
|---|---|
| `issuesmatch.py --none-refused … 19` | every `@` line issued, none refused |
| `rngcmp.py` vs run362's trace | the same game to 2279 (2,851 frames in common, 569 differ, the first 2280: run362's own presses) |
| the processed commands | `process_queue_up 313 2` on 2226 and `310 2` on 2241; `process_launch_patrol 7680 11520 2 0 0 0` on 2441 and `3840 13440 2 0 0 0` on 3011; `process_gather_point` on 2461, 2501 and 2801; `process_flight` on 2651, 2671 and 3001 |

### §44's falsifiers

| check | predicted | observed |
| --- | --- | --- |
| 1, the issues | processed on the frame after each press | **as predicted** |
| 2, the resource line (2215) | six buckets +500 | **as predicted**: no bucket parts before the queue's charge |
| 3, the silo | `0/2009` on 2221 | **as predicted**, at (9984, 12288) |
| — the silo's queue (2227) | two V2s queued (the walk) | **one**: `queued 1`, one price paid; the second `@queueup` refused. No V2 `0/12` is born, so the Helicopters are `0/11` and `0/12` |
| 4, `0/11`'s birth (2445) | out, two draws, no order | **as predicted**: out on 2446 at (8355, 14690), `idle`; the draws `5e59ef`, `5e5a0f` |
| 5, the silo's patrol (2442) | nothing | **as predicted** |
| 6, P_h (2502) | `0/11` untouched | **as predicted** |
| 7, the second V2 (2635) | no order | **cannot fire**: no second V2 |
| 8, the silo's `MOVE_TO` (2652) | nothing | **as predicted** (`0/10` alone inside) |
| 9, the second Helicopter (2659) | an `AIRPATROLORDER` over P_h, flags 4 | **fired**: a `MOVEORDER` (`MOVE_TO`) to (5784, 12312), P_h's cell centre, flags 4 — `add_air_patrol_order`'s Helicopter arm (`5e4363`), read after the commit |
| 10, the silo's strike (2672) | `0/10` a `STRAFEORDER`, flags 4 | **fired on the kind**: an `AIRATTACKGROUNDORDER`, flags 4, home `0/2009` — `add_strafe_order`'s missile head, read after the commit; `recharging` 30 on 2672, 1 on 2701, and `0/10` gone on 2702 with its round in flight |
| 11, the Clear (2802) | `0/12` a strafe home | **fired**: nothing — `0/12`, at its point from 2740 with no order, is homed nowhere |
| 12, the strike at `0/2008` (3002) | nothing | **as predicted** (the base empty) |
| 13, the patrol at `0/2008` (3012) | `0/12` a `MOVE_TO` | **fired**: nothing, the base empty |

`0/11` drifts from (8355, 14690) to (8465, 14784) by 2701, idle. The
V2's round is a spline shot (`traj 2`, `total_time` 120) from (9974,
12347) at 466 onto 1/2006's point.
## run368 — Great Lakes at Toughest, a packet at logger frame 4688: the raider's re-search (2026-09-28, item 1028)

**What it is.** A `RON_STATE_FRAME=4688` packet on the click-free lane over
run347's game (Great Lakes, `DIFFICULTY 5`). Blocks 4684–4691 are dumped
at run356's detail, and `!quit` is at 4697. The plan is item 597's, as for
run360. The lane lock was stale: att-1011's runner had exited, and nothing
else ran.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-28-run368 \
    --map 14 --end-frame 4697 --timeout 2400 --log-window 4684 4692 \
    --detail end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1 \
    --tracer-def RON_STATE_FRAME=4688 \
    --tracer-def 'RON_STATE_PLAN="<plan>/plan.h"' \
    --profile DIFFICULTY=5
```

**Taken** 01:03, one take. `success: true`, exit 0, 55 s launch to exit
and 64 s in all. The packet is 807,996,408 bytes.

| check | result |
|---|---|
| `rngcmp.py` against `rontrace-run347.log` | **0 differing**, 4,698 identical |
| `GAME INFO` | `DIFFICULTY 5`, `MAP_STYLE 14`, seed 12345 read back |
| settings | five files restored |

**Why 4688, and not the booked 4687.** The word is trace tick 4688, and
block 4689 is the first to print its strike. `docs/EMULATOR.md` §8's rule
(the twelfth pass) is to take the word itself as a logger frame. The
booking's "a packet at N is after N−1's decision" is the rule that pass
struck. A packet at 4687 would have left tick 4687 to emulate, and
`Game::do_frame` faults on a packet (parked 1026).

**The booking cites what the disk could not answer.** No dump prints
`targeted` (`ObjectData +0x3d`), and 1023 grepped for it.

**What it settled** (`docs/COMBAT.md` §68.1, §68.3):
- `targeted` is 0 on the scout `0/0`, the citizens and every building,
  the city included, where ours read 31 on the city.
- `find_melee_target`, entered directly on `1/24`, answers `0/3`.
- Every candidate's shaped distance equals ours, and the values do not:
  the RAID arm.

## run369 — Great Lakes at Toughest, a packet at logger frame 4605: the army's hand-off (2026-09-28, item 1028)

**What it is.** A `RON_STATE_FRAME=4605` packet over run347's game, blocks
4601–4608 dumped, `!quit` at 4614. It is the booking's second packet, for
parked 1015. It was taken at 4605 rather than 4606 because the question
became the look's hand-off on tick 4605 itself.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-28-run369 \
    --map 14 --end-frame 4614 --timeout 2400 --log-window 4601 4609 \
    --detail end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1 \
    --tracer-def RON_STATE_FRAME=4605 \
    --tracer-def 'RON_STATE_PLAN="<plan>/plan.h"' \
    --profile DIFFICULTY=5
```

**Taken** 01:17, one take, exit 0. The packet is 821,648,320 bytes.
`rngcmp.py` against `rontrace-run347.log` shows **0 differing** of 4,615,
and `DIFFICULTY 5` read back. Free disk went 23 → 22 GB across the two.

**The booking cites what the disk could not answer.** With §68.1 built,
the walk fell back to 4605: our members took `0/2001`, where every member
of group 65 holds the city on block 4606. The dump prints the result,
not the searches that made it.

**What it settled** (`docs/COMBAT.md` §68.2):
- A word-2 search from `1/9`, `1/13` or `1/21` answers `0/2001`.
- `1/15`'s look finds the city, and `Group::action_attack` gives all
  eighteen members `add_attack_order(2000)` from `+0xc44`, with no search.

Both packets stay outside git under `~/ron-data/lab-captures/`. The probes
(`targeted_probe.py`, `melee_probe.py`) and their outputs are at
`~/ron-data/lab-experiments/2026-09-28-item-1028/`.

## run373 — run347's game at run356's detail over blocks 4841..5097: the second pair's Great Lakes word 4846 widened (2026-09-28, item 1040)

**What it is.** run356's shape on run347's lobby, with `frame_window:
4841 5098` and `!quit` at 5110: six blocks before the word 4846's block
4847 and 250 after it. The stanza is in `tools/gamelog/captures.txt`.

**The booking cites what the disk could not answer.** The word had moved
past run356's last block (its dump stops at 4821, its window at 4806), and
run347's own dump prints no per-frame record there.

**Taken** 04:36–04:56 through `runqueue.sh - 1040` under `viadriver.sh`,
one take, after att-1019's run372 released the lane. 519.7 MB of gamelog,
11.6 MB of trace. Free disk was 18 GB before and after (`df -g ~`).
- `rngcmp.py` against run347: 5,111 frames in common, **0 differing**.
- 257 window blocks; `DIFFICULTY 5`, `MAP_STYLE 14`.

**What it holds**: `run373_s_word_frame_is_widened_whole`
(`docs/COMBAT.md` §70.6–§70.8).
- Block 4841 stands on 111 keys. Among them are the citizens' damage (more
  here than there since run356's window closed) and the capital's
  `order:target` (parked 1003).
- Block 4847 parted on the Hoplite `1/19`'s facing. It closed under the
  side arm (§70.6).
- Block 4853 parted on `1/24`'s clock. It closed under the frozen mark
  (§70.7).
- The word moved to 4877 inside the window. Block 4861, frame 4860 (the
  army's tick), is the first to part after that, on 121 keys.

## run381 — Great Sahara's `DUMP_ALL` start, in run38's shape (2026-09-28, item 1066)

**The third scored map's sibling** (DECISIONS 54 §3). This is run38's
recipe with the map moved:
`zsh tools/gamelog/viadriver.sh tools/gamelog/startcapture.sh 381 7
greatsahara-start`. It took two `DUMP_ALL` frame blocks, `cover=1
window=0-1`, `2 !quit`, and the profile's lobby (no `-config`, which pins
the style at 14). It ran in one take: launched at 09:49:26, the start
clicked at 09:50:22, settled at 09:56. The dump is 149,587,702 bytes
(run38's is 151 MB) and the trace 8,280,576 bytes over six frames. The DLL
is sha256 `d654cdb2…599d`, a later build than run348's `b451aeb6…3dca`,
and the same one run382 and run383 ran. The lane lock was stale (att-1040's
pid 37944, dead) and no `RonDriver` was running. The disk had 70 GB free.
`waitrun.sh` under `WAITRUN_RUNNER=gamelog/startcapture.sh` exits 2 on
this script, which prints no banner, so the log's tail is the verdict: it
read `MAP_STYLE 7` and `seed 12345`.

**`GAME INFO` is run39's 85 lines, line for line, except `MAP_STYLE 7`**.
`DIFFICULTY 0` reads back. The profile and `check.ini` were copied before
the launch and put back after it once the lane was idle, and the profile
read `MAP_STYLE 14` and `DIFFICULTY 0` in both blocks again.

**It stands the map up alone.** `rondata <install> --gamelog
gamelog-run381-greatsahara-start.txt --trace rontrace-run381.log --diff`
loads 3,600 cells, two regions, nine placed mountain ranges, two herds and
24 goods from the head, and rolls the AI's personality where the original
did. It reads **`rng: frame 0: ours 99 draws, the original's 99`**, three
frames with no unit parting. What the map holds (all land, west against
east) is in `docs/AI.md` §83.2.

## run382 — Great Sahara in run33's shape, the third map's score (2026-09-28, item 1066)

This is run348's stanza with the map moved, 9 → 7, through
`viadriver.sh tools/gamelog/runqueue.sh - 1066`. It used run10's
detail, the window `[0, 1900)`, `5 !ffwd 30`, `1850 !quit` and
`cover=1`, on the queue lane (1047). It ran in one take from 10:00 (the
start clicked at 10:00:52) to 10:10. The dump is 176,714,287 bytes with
1,851 blocks, and the trace 8,804,416 bytes. `GAME INFO` is run39's except
`MAP_STYLE 7`.

| check | result |
|---|---|
| `MAP_STYLE 7`, `seed 12345`, `DIFFICULTY 0`, blocks ≥ 1850 | pass (1,851) |
| `rngcmp.py` against run381 | 6 in common, 0 differ |
| `samegame.py` against run381, the count | **2 in common**, 2 differ (run381's blocks are `DUMP_ALL`; run38 against run39 reads the same) |

The queue called the take "checks FAILED" on the last line. That was the
stanza's fault, not the capture's: the runner's pipe status carried
`samegame.py`'s exit 1 through `tee`, so the line asserted the verdict it
was written not to assert. The stanza now reads `(… || true) | tee`.

**The number: ticks 6, orders 5.** The first word is 8.

```
rondata <install> --gamelog gamelog-run382-greatsahara-longtrace.txt \
        --sibling gamelog-run381-greatsahara-start.txt \
        --trace rontrace-run382.log --diff        # on 11d1dbaa

  note: rng: frame 0: ours 99 draws, the original's 99 — installed 0xa1258b91
  1851 frames stepped, 28000 unit-frames compared, 0 unit-frames the sim has no unit for
  ticks before divergence: 6
  ticks before an order diverges: 5
  who 1 o 2: first order disagreement at frame 6 — Move { field: "dest_x", ours: 38232, theirs: 38328 }
  player 0: first divergence at frame 103 — unit o 3 ours (4996, 30617) theirs (5009, 30576)
  player 1: first divergence at frame 7 — unit o 2 ours (38352, 15814) theirs (38328, 15768)
```

The widening over its blocks 1..259, and the first parting by unit and
field, are in `docs/AI.md` §83.5.

## run383 — Great Sahara's draw stream to 24,000, `cover=0` (2026-09-28, item 1066)

This is run347's stanza with the map moved and no `profile:` line, so the
profile's `DIFFICULTY 0` stands. It used `end: MISC`, run10's `start:`,
`cover=0` and `cfg: -`, and ran in the same queue as run382 from 10:10
(the start clicked at 10:11:19) to 10:14. The dump is 10,552,505 bytes and
the trace 22,691,712. `GAME INFO` is run39's except `MAP_STYLE 7`.
`rngcmp.py` against run382 reads 1,851 in common and 0 differ, so the long
capture is the score capture's game.

**Where it ends, and why: at 24,000, on the `!quit`.** The trace holds a
`FRAME` record for all 24,001 frames. The dump's last block, 24001, is the
shutdown dump, and both leaders read `defeated_by -1`, with scores 421 for
the human and 2,495 for the AI. Nobody won an Easiest game against an idle
human in twenty-seven minutes of game time.

**The long word is 8**, the same frame as run382's
(`run383_s_long_trace_says_where_the_third_map_s_word_parts`): ours 6
draws against 7, parting at index 0, `Farms::inc_time+0x1ae` against
`Guy::set_anim+0x97a < Unit::move_step+0x823`.

## run396 — run347's game at run373's detail over blocks 5100..5356: the second pair's Great Lakes word 5105 widened (2026-09-28, item 1074)

**What it is.** run373's shape on the click-free lane: run347's lobby
(`DIFFICULTY 5`, map 14, seed 12345), `cover=0`, the detail
`end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, blocks
5100..5356, and `!quit` at 5369. That is six blocks before the word 5105's
block 5106 and 250 after it.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-28-run396 \
    --map 14 --end-frame 5369 --timeout 3000 --log-window 5100 5357 \
    --detail end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1 \
    --profile DIFFICULTY=5
```

**The booking cites what the disk could not answer.** Item 1074 moved the
word from 5075 to 5105, past run373's last block (5097), and run347's own
dump prints no per-frame record there.

**Taken** 14:10–14:27, one take, through `viadriver.sh`, waited on with
`waitrun.sh`. `success: true`, exit 0, 5,370 frames, 990 s from launch to
exit. The lane lock was stale (pid 6486, exited) and no `RonDriver` or
wine was running. The dump is 509,028,415 bytes and the trace 31,951,776.
They were moved into `Logs` as `gamelog-run396-greatlakes-toughest-5105.txt`
and `rontrace-run396.log`. Free disk was 66 GB before and 65 after.

| check | result |
|---|---|
| `rngcmp.py` against `rontrace-run347.log` | **0 differing**, 5,370 identical |
| `GAME INFO` | `DIFFICULTY 5`, `MAP_STYLE 14` |
| window blocks 5100..5356 | 257 |

**What it holds**: `run396_s_word_frame_is_widened_whole`
(`docs/COMBAT.md` §77.6). It parts on 1,253 keys, 139 of them standing on
its first block (among them `1/21`'s chase goal, parted since run373's
5091, and `1/13`'s, parked 1089). Block 5105 parts only on `1/21`'s
order: the original has ended its chase, and ours has not. The compared
pin's walk (`second::great_lakes_word_window`) reads this capture from
item 1074 on.
## run390 — chapter thirty-six, the missile's other arms (2026-09-28, item 1078)

`docs/GOLDEN.md` §45, `tools/gamelog/golden/chapter36.cmd`: chapter
thirty-five whole, then `resource who=0 all +500` (2705), Missile Silos
`0/2010` and `0/2011` (2710, 2712), a V2 queued at each (2720, 2725),
who=1's Barracks `1/2007` in its own land and `1/2008` in no one's (2730,
2735), a who=0 Elite Special Forces between them (2740), and
`@launchstrike` from `0/2010` on `1/2007` (3019), again on `1/2008`
during the countdown (3029), `tech who=1 missile_shield on` (3070), and
`@launchstrike` from `0/2011` on `1/2008` (3080). The chapter and its
eight falsifiers were committed before the run (`d9dfa274`).

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch36 \
    --map 14 --end-frame 3420 --log-window 605 3420 --timeout 7200 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter36.cmd
```

**The take** at `cover=0` on the click-free lane, 12:49: **3,278 s** in all
(3,225 launch to exit), **1.17 GB** of dump and 15.5 MB of trace, 2,815
blocks. The receipt says `success: true`, exit 0, 3,421 frames, `MAP_STYLE
14`, seed 12345, 1,441,280 `GROUPDATA`, settings restored. `waitrun.sh`
exited 0. 65 GB free after. The lane's lock was stale (both holders
gone) and was taken over. No `cover=1` twin: no `NEVER` row is on the
staging's path, so run391 is unused, and so is run392.

| check | result |
|---|---|
| `issuesmatch.py --none-refused … 24` | every `@` line issued, none refused |
| `rngcmp.py` vs run371's trace | the same game to 2740 (3,261 frames in common, the first differing 2741: the spotter's birth) |
| the processed commands | `process_queue_up 313 1` on 2721 and 2726; `process_flight` on 3020, 3030 and 3081 |

### §45's falsifiers

| check | predicted | observed |
| --- | --- | --- |
| 1, the issues | processed on the frame after each press | **as predicted** |
| 2, the staging | the silos, the Barracks, `0/14` on 2926 and `0/15` on 2931 | **as predicted**, at the walked points |
| 3, the strike (3021) | `0/14` an `AIRATTACKGROUNDORDER` to (38016, 18816), flags 4; `recharging` 30 | **as predicted**, `oxx` 2010 |
| 4, the re-press (3031) | `0/14`'s `att` unchanged | **as predicted** |
| 5, the redraw (trace 3050) | `do_air_physics+0xba`, then `Ammo::init+0xae8`, `+0xb25` | **as predicted**: rolls 17247, 40766, 9105; `sz` 530 |
| 6, the shield at the order (3082) | `0/15` inside, no order | **as predicted**, to the end |
| 7, the shield at the blast (3170) | `1/2007` at `damage` 0, the round gone | **as predicted**; 3169's one `Ammo::do_damage` draw is the sound generator's |
| 8, the tech line (3072) | no bucket of who=1's moves | **as predicted** |

## run397 — chapter thirty-seven, the nuke (2026-09-28, item 1091)

`docs/GOLDEN.md` §46, `tools/gamelog/golden/chapter37.cmd`: a cast of its
own on the golden start — a Missile Silo `0/2007` (610), the nuke's
research there (615) and its train (2300), who=1's Barracks T `1/2006` at
ground zero and F `1/2007` at a building distance of 1824 (2940, 2942),
four of who=0's Elite Special Forces at 518, 1039, 1804 and 1870 from T
(2950..2956), `@launchstrike` on T (3050) and `tech who=1 missile_shield
on` in the countdown (3060). The chapter and its ten falsifiers were
committed before the run (`298cf771`).

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch37 \
    --map 14 --end-frame 3490 --log-window 605 3490 --timeout 9000 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter37.cmd
```

**The take** at `cover=0` on the click-free lane, 15:09: **3,380 s** in all
(3,335 launch to exit), **1.12 GB** of dump and 15.2 MB of trace, 2,885
blocks. The receipt says `success: true`, exit 0, 3,491 frames,
`MAP_STYLE 14`, seed 12345, 1,477,120 `GROUPDATA`, settings restored.
`waitrun.sh` exited 0. The lane's lock was stale. 64 GB free after.

| check | result |
|---|---|
| `issuesmatch.py --none-refused … 3` | three `@` lines issued, none refused (615, 2300, 3050) |
| `rngcmp.py` vs run390's trace | the same game to 610; the first differing frame 611, the silo's |

### §46's falsifiers

| check | predicted | observed |
| --- | --- | --- |
| 1, the issues | processed on 616, 2301 and 3051 | **as predicted** |
| 2, the research (617, 2237) | the entry laid, then gone with nothing placed | **as predicted**: `queued` 1 on 617, 0 on 2237 |
| 3, the train (3022) | `0/10` inside `0/2007` | **as predicted**, seated +24, +24 |
| 4, the strike (3052) | one `AIRATTACKGROUNDORDER` on T's point; `recharging` 30 | **as predicted** |
| 5, the shield after the press | the launch on 3081 | **as predicted** |
| 6, the launch (3082) | `visible` −1; the round on (23040, 34560), no draw; 120 frames | **as predicted**: `visible` 0 on 3081, −1 on 3082; `ez` 403 |
| 7, the landing (3201) | T gone, F and the probes untouched; `nuke_stamp` 3200, `nukes_used` 1 | **as predicted**; the leader half **could not fire**: `LEADERS=2` prints neither |
| 8, the ring | `0/6` on 3217, `0/7` on 3226, `0/8` on 3239, once each | **as predicted**: `0/6`, `0/7` killed, `0/8` at 56 and 4/16 |
| 9, the edge | `0/9` untouched | **as predicted**, to 3489 |
| 10, F | struck on 3239 alone, standing | **as predicted**: 491 and 2/16 on 3240 |

## run398 — chapter thirty-seven at `cover=1`, to 3210 (2026-09-28, item 1091)

`captures.txt`'s stanza: `chapter37.cmd` line for line with the chapter's
own `!ffwd`, the golden lobby (`cfg: -`), `cover=1`, on the queue lane,
ending ten frames past the landing. The click-free lane cannot take
`cover=1` (run185; 1011's split).

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 398
```

**The take**, 16:17 → 16:39, `captured, checks ok`: 433 MB of gamelog,
10.0 MB of trace. Every check passed:

| check | result |
|---|---|
| `issuesmatch.py` vs run397 | three issue records, none refused; golden 3 |
| `rngcmp.py` vs run397 | 3,211 frames identical, none differing |
| `report.py … functions` | `Nuke::add_nuke` entered on 3200 |

**The blind list 145 → 144**: `Nuke::add_nuke@0092ba30` off `NEVER`, the
one row the staging named; `LeaderData::can_nuke` (3050) and
`Nuke::been_damaged_before` (3200), newly cited by §46 and PRODUCTION,
are entered by it too. `rontrace-run398.log` joins `blind::TRACES`.
## run400 — Great Lakes at Toughest, a packet at logger frame 5066: `1/13`'s `find_new_target` (2026-09-28, item 1089)

**What it is.** A `RON_STATE_FRAME=5066` packet on the click-free lane over
run347's game (Great Lakes, `DIFFICULTY 5`). Blocks 5062–5069 are dumped at
run368's detail, and `!quit` is at 5075. It uses the plan of item 597, as
run368 did. It launched when att-1091's run397 released the lane
(`waitrun.sh` on that run's log, exit 0).

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-28-run400 \
    --map 14 --end-frame 5075 --timeout 2400 --log-window 5062 5070 \
    --detail end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1 \
    --tracer-def RON_STATE_FRAME=5066 \
    --tracer-def 'RON_STATE_PLAN="<plan>/plan.h"' \
    --profile DIFFICULTY=5
```

**Taken** 16:06, one take. `success: true`, exit 0, 54 s launch to exit
and 62 s in all. The packet is 807,820,156 bytes. Free disk was 64 GB
before.

| check | result |
|---|---|
| `rngcmp.py` against `rontrace-run347.log` | **0 differing**, 5,076 identical |
| `GAME INFO` | `DIFFICULTY 5`, `MAP_STYLE 14`, seed 12345 read back |
| settings | five files restored |

**Why 5066, and not the word's 5105.** A probe of a decision runs inside
the decision. The word is 5105, but the decision it follows from is
`1/13`'s `find_new_target` on trace tick 5066, which writes block 5067. A
packet at logger frame 5066 is the state before that tick. By 5105, `0/5`'s
`targeted` would have decayed and been bumped again many times. This
departs from the letter of `docs/EMULATOR.md` §8 ("the word itself"), and
the commander approved it before the launch.

**The booking cites what the disk could not answer.** No dump prints
`targeted` (`ObjectData +0x3d`), and it was the one input under which ours'
ranking could have preferred `0/4`. The dump prints the order the search
added, not the search.

**What it settled** (`docs/COMBAT.md` §79.2):
- `targeted` is 2 on `0/5` and 0 on `0/4`, `0/0`, `0/1`, `1/13`, `1/21`
  and `1/22`, the same as ours.
- `find_melee_target(−1, &whom, 0, 1, 0)`, entered directly on `1/13`
  (`13d5982c`), calls `add_attack_order(4, 0, …)` from
  `find_melee_target+0x1c5` and answers `o 4 whom 0`. It never calls
  `find_nearby_target`: the squad head handed `1/13` its captain `1/12`'s
  target.

The packet stays outside git under `~/ron-data/lab-captures/`. The probes
(`targeted_probe.py`, and `melee_probe.py` with a `P4` argument), copied
from item 1028's, and their outputs are at
`~/ron-data/lab-experiments/2026-09-28-item-1089/`. The copied probes
first reproduced item 1028's `1/24` list on run368's packet exactly.

## run404 — chapter thirty-eight, the air line under fire (2026-09-28, item 1102)

`docs/GOLDEN.md` §47, `tools/gamelog/golden/chapter38.cmd`: a cast of its
own on the golden start — an Airbase `0/2007` and two Bombers `0/6`,
`0/7` (606..612), who=1's Barracks T `1/2006`, a Radar Air Defense
`1/2007`, an Infantry squad `1/6`..`1/8` and an Anti-Aircraft Battery
`1/9` (616..622), `@flight` home (640) and `@strike` on T in flight (644).
The chapter and its eight falsifiers were committed before the run
(`9954ebe7`).

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch38 \
    --map 14 --end-frame 1762 --log-window 605 1762 --timeout 5400 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5,DEATHS=1 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter38.cmd
```

**The take** at `cover=0` on the click-free lane, 17:40: **895 s** in all
(878 launch to exit), **321 MB** of dump and 12.1 MB of trace, 1,157
blocks, exit 0, 1,763 frames, settings restored. **The receipt says
`success: false`**: `GROUPS was asked for and no GROUPDATA block was
printed` — the first capture to ask for `DEATHS=1` beside `GROUPS=1`
(§47, GOLDEN §16). `waitrun.sh` exited 2 on it. The lane's lock was
stale. 63 GB free after.

| check | result |
|---|---|
| issues | two `@` lines, processed on 641 and 645 |
| `GROUPDATA` | none on any block; every other record whole |
| `DEATH` | none on any block (the only deaths are the two planes) |

### §47's falsifiers

| check | predicted | observed |
| --- | --- | --- |
| 1, the issues | processed on 641, 645 | **as predicted** |
| 2, the flight | strafes home, then patrols over T | **as predicted** (no widening row) |
| 3, the roll's site | one draw a flak round, `+0x432`/`+0x463` | **as predicted**: 43 draws, 31 and 13, none of the other arm; 963's `+0x463` at 2,279 from T not read |
| 4, the threshold | `flags & 0x10` iff `r % 100` ≥ the shooter's figure | **as predicted** on all 38 matched rounds; 5 draws print no round |
| 5, a miss | does not wound | **as predicted** |
| 6, the Infantry | no round at a high Bomber | **could not fire**: no round of theirs at a Bomber |
| 7, the crash | one draw at `init_crash+0x305`, the round, no death | **as predicted**: 1006 and 1019, `rolling` −1 and −2, no `DEATH` |
| 8, inside | no round | **as predicted** |

## run405 — chapter thirty-eight at `cover=1`, to 1030 (2026-09-28, item 1102)

`captures.txt`'s stanza: `chapter38.cmd` line for line with the chapter's
own `!ffwd 2`, the golden lobby (`cfg: -`), `cover=1`, on the queue lane,
ending eleven frames past the second crash, and the end detail without
`DEATHS` (run398's).

```
zsh tools/gamelog/viadriver.sh tools/gamelog/runqueue.sh - 405
```

**The take**, 18:00 → 18:11, `captured, checks ok`: 181 MB of gamelog,
9.1 MB of trace.

| check | result |
|---|---|
| `issuesmatch.py` vs run404 | two issue records, none refused; golden 2 |
| `rngcmp.py` vs run404 | 1,031 frames identical, none differing |
| `report.py … functions` | `UnitData::is_flying_low` and `is_flying_high` entered on 730, `Ammo::init_crash` on 1006 |

**The blind list 144 → 141**: the three rows the staging named.
`rontrace-run405.log` joins `blind::TRACES`.
## run403 — run347's game at run396's detail over blocks 5925..5930: the second pair's Great Lakes word at the game's end (2026-09-28, item 1099)

**What it is.** run396's shape on the click-free lane: run347's lobby
(`DIFFICULTY 5`, map 14, seed 12345), `cover=0`, the detail
`end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, blocks
5925..5930, and `!quit` at 5930. That is the last six blocks before the
game's own end: item 1099 moved Great Lakes' word to 5930, run347's last
traced frame, where the AI takes the human's city.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-28-run403 \
    --map 14 --end-frame 5930 --timeout 3000 --log-window 5925 5931 \
    --detail end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1 \
    --profile DIFFICULTY=5
```

**The booking cites what the disk could not answer.** run396 ends at
block 5356, and run347's own dump prints no per-frame record past its
start: nothing on disk held the blocks before the game's end at detail.
The word's own block, 5931, is run347's closing whole-map state, which the
disk did hold and which `walk_second` now scores.

**Taken** 17:35–17:36, one take, through `viadriver.sh`, waited on with
`waitrun.sh`. Two launches before it refused before staging anything: the
first on its window (`--log-window 5925 5932` is past `!quit` + 1), the
second on the empty output directory the first had made, which was
removed with `rmdir`. The lane lock was stale (att-1091's pids, exited)
and no `RonDriver` was running. `success: true`, exit 0, 5,931 frames, 48
s from launch to exit (the fast-forward). The dump is 11,994,340 bytes and
the trace 33,994,528; they were moved into `Logs` as
`gamelog-run403-greatlakes-toughest-5930.txt` and `rontrace-run403.log`.
Free disk was 63 GB before and after.

| check | result |
|---|---|
| `rngcmp.py` against `rontrace-run347.log` | **0 differing**, 5,931 identical |
| `GAME INFO` | `DIFFICULTY 5`, `MAP_STYLE 14` |
| window blocks 5925..5930 | 6 |
| the quit block, 5931 | `GameInfo closing` and no record: the tracer's quit, not the game's end |

**What it holds**: `run403_s_word_frame_is_widened_whole`
(`docs/COMBAT.md` §80.6). It parts on 112 keys, every one standing on its
first block and every family among them standing on run396's first block
too; nothing parts on 5926..5930, and no position. The compared pin's walk
(`second::great_lakes_word_window`) read its last four blocks from item
1099 until item 1106 moved it to East Indies' word
(`second::east_indies_word_window`, run357).

## run407 — East Indies at Toughest, a packet at logger frame 5518: `1/14`'s colonist gate and the border pass (2026-09-28, item 1106)

**What it is.** A `RON_STATE_FRAME=5518` packet on the click-free lane over
run346's game (East Indies, `DIFFICULTY 5`), with blocks 5514..5521 dumped
at run400's detail and `!quit` at 5527. It uses item 597's plan, as run400
did.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-28-run407 \
    --map 18 --end-frame 5527 --timeout 2400 --log-window 5514 5522 \
    --detail end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1 \
    --tracer-def RON_STATE_FRAME=5518 \
    --tracer-def 'RON_STATE_PLAN="<plan>/plan.h"' \
    --profile DIFFICULTY=5
```

**Taken** 18:38, one take. `success: true`, exit 0, 59 s from launch to exit
and 69 s in all. The packet is 827,988,952 bytes. Free disk was 62 GB
before. The lane lock was stale: att-1102's `longtrace.sh` pids had exited,
and no game or driver was running.

| check | result |
|---|---|
| `rngcmp.py` against `rontrace-run346.log` | **0 differing**, 5,528 identical |
| `GAME INFO` | `MAP_STYLE 18`, seed 12345 read back; lobby `DIFFICULTY=5` |
| settings | five files restored |

**Why 5518.** Ours seats the citizen `1/14` in pool group 67 on tick 5518,
the first frame it is idle after it finishes the city `1/2017` (tick 5517).
The original's group is stamped 5521. A packet at logger frame 5518 is the
state before ours' deciding tick.

**The booking cites what the disk could not answer.** No dump prints
`1/14` between run352's window (to 1827) and run357's (from 5601), and no
dump prints the world's per-cell owner byte or a region's border resume
index. Those are the colonist gate's input (`think_civilian_transport`'s
`world +0x134`, `+0xf`) and the pass that writes it.

**What it settled** (`docs/AI.md` §84):
- The dump window: `1/2017` finishes on block 5518 on both sides
  (`job_counter` 59900 → 0, flags 35 → 39). `1/14` has no order from 5518
  and goes idle on 5519 (`idle` 1, 2, 2 on 5519..5521), still in group 66.
  who=1's `city_num` 3 and `xport_peasants` 0 hold. `reg_cities[5]` is
  already 1 on 5518. `reg_terr` empties on 5519 and region 5's reads 34 on
  5521.
- The packet (`borders_probe.py`): `1/14`'s cell (48, 29) is **unowned**
  (−1, −1). Every land region's resume index is **0**: regions 1..12 of 260,
  153, 139, 182, 151, 96, 100, 12, 125, 151, 269 and 31 cells. The cell is
  region 5's list position 112. At 256 cells a frame the pass reaches it
  on tick 5521, and after tick 5520 region 5's index is 34, which is the
  dump's `reg_terr[5]`.

The packet stays outside git under `~/ron-data/lab-captures/`. The probe
and its output are at `~/ron-data/lab-experiments/2026-09-28-item-1106/`.


## run413 — East Indies at Toughest, a packet at logger frame 5776: the Caravan `1/33`'s parked road search (2026-09-28, item 1115)

**What it is.** A `RON_STATE_FRAME=5776` packet on the click-free lane over
run346's game (East Indies, `DIFFICULTY 5`), with blocks 5772..5778 dumped
at run407's detail and `!quit` at 5785.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-28-run413 \
    --map 18 --end-frame 5785 --timeout 2400 --log-window 5772 5778 \
    --detail end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1 \
    --tracer-def RON_STATE_FRAME=5776 \
    --tracer-def 'RON_STATE_PLAN="/Users/rf-studio/ron-data/lab-experiments/2026-09-23-item-597/plan/plan.h"' \
    --profile DIFFICULTY=5
```

**Taken** 20:28–20:29, one take, waited on with `waitrun.sh`. `success:
true`, exit 0, 54 s from launch to exit. The packet is 811,948,992 bytes. The
lane lock was stale (run407's pid), no driver was running, and 60 GB were
free.

| check | result |
|---|---|
| `rngcmp.py` against `rontrace-run346.log` | **0 differing**, 5,786 identical |
| receipt | `map_style 18`, seed 12345, lobby `DIFFICULTY=5`, five files restored |

**Why 5776.** With `do_trade`'s region tests built, ours' road search kept
the original's per-frame draw counts on 5773..5775 and parted on 5776. The
draw labels are all one site, so they cannot say which nodes a frame
priced. A packet at logger 5776 is the state after three search frames.

**The booking cites what the disk could not answer.** No dump prints a
parked search: the `CARAVAN` block prints the laid road only, and run346's
trace proxies no `calc_road_cost`.

**What it settled** (`docs/CARAVAN.md` §11.2, `docs/AI.md` §85):
- `road_probe.py` reads `caravans` (VA `0xE3A290`), who=1's slot 1 (`city2`
  0, `city3` 2, `o` 33, `making_road` 1) and walks the open `Tree` and the
  closed `BRTree`. They hold 158 open and 1,279 closed nodes.
- Against ours after tick 5775: 171 nodes apart with the surcharge on every
  sea tile, and **1,437 against 1,437, none apart**, with the parent test.

## run414 — run346's game at run357's detail over blocks 5970..6226: the second pair's East Indies word 5975 widened (2026-09-28, item 1115)

**What it is.** run357's shape on the click-free lane: run346's lobby, `cover=0`,
the detail `end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`,
blocks 5970..6226, `!quit` at 6240. The word's frame 5975 writes block
5976: six blocks before it and 250 after.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-28-run414 \
    --map 18 --end-frame 6240 --timeout 4000 --log-window 5970 6227 \
    --detail end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1 \
    --profile DIFFICULTY=5
```

**The booking cites what the disk could not answer.** run357 ends at block
5857. No dump prints the word's blocks.

**Taken** 20:35–20:54, one take, waited on with `waitrun.sh`. `success:
true`, exit 0, 1,153 s from launch to exit. The dump (577.3 MB) and the
trace (48.3 MB) were moved into `Logs` as
`gamelog-run414-islands-toughest-5975.txt` and `rontrace-run414.log`.

| check | result |
|---|---|
| `rngcmp.py` against `rontrace-run346.log` | **0 differing**, 6,241 identical |
| window blocks | 257, 5970..6226 |
| receipt | `map_style 18`, seed 12345, lobby `DIFFICULTY=5`, five files restored |

**What it holds**: `run414_s_word_frame_is_widened_whole` (`diff::second`),
walked from run346's start. Block 5970 stands on 156 keys, all standing
families. The word's block 5976 parts on 5: the newborn `1/40`'s figure
clocks. The compared pin walks its 5974..5978, and the coverage driver
drives 5975..5978.

## run415 — East Indies at Toughest, a packet at logger frame 5975: the sea scout's region scan (2026-09-28, item 1115)

**What it is.** run413's shape at `RON_STATE_FRAME=5975`, blocks 5972..5977,
`!quit` at 5980. The number was the commander's, on request.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-28-run415 \
    --map 18 --end-frame 5980 --timeout 2400 --log-window 5972 5978 \
    --detail end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1 \
    --tracer-def RON_STATE_FRAME=5975 \
    --tracer-def 'RON_STATE_PLAN="/Users/rf-studio/ron-data/lab-experiments/2026-09-23-item-597/plan/plan.h"' \
    --profile DIFFICULTY=5
```

**Taken** 20:58–20:59, one take. `success: true`, exit 0, 56 s. `rngcmp.py`
against run346: **0 differing**, 5,981 identical.

**Why 5975.** The scan is tick 5975's, and a packet at logger 5975 is the
state before it. **The booking cites what the disk could not answer**: the
scan's refusals are fog and `invalid_loc`, and no dump prints `seen2`.

**What it settled** (`docs/AI.md` §85.5): `fog_probe.py` reads `WorldData
+0x160`, 120 × 120. Against ours after tick 5974, three half-cells are seen
there and not here: (45, 73), already on run407, and **(90, 80) and
(91, 81)**. The last is the probe point of cell (45, 40), the scan's extra
accepted cell. run413 holds both unseen at logger 5776.

The packets stay outside git under `~/ron-data/lab-captures/`. The probes
and their outputs are at `~/ron-data/lab-experiments/2026-09-28-item-1115/`.

## run416 — run383's game at run414's detail over blocks 12778..13034: Great Sahara's long word 12783 widened (2026-09-29, item 1133)

**What it is.** run414's shape on the click-free lane with the third map's
lobby: `--map 7`, `--profile DIFFICULTY=0` (the first pair's Easiest),
seed 12345, `cover=0`, the detail
`end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, blocks
12778..13034, `!quit` at 13048. The word's frame 12783 writes block 12784:
six blocks before it and 250 after.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-29-run416 \
    --map 7 --end-frame 13048 --timeout 9000 --log-window 12778 13035 \
    --detail end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1 \
    --profile DIFFICULTY=0
```

**The booking cites what the disk could not answer.** run382 ends at block
1851 and run383 prints no record past its start. No dump prints the
word's blocks.

**Taken** 06:25–06:42, the second take, waited on with `waitrun.sh`. The
first take (06:24) died 3.5 s after launch, before frame 0: `wine:
Unhandled page fault on read access to 00004ECD at address 7BF21139`, and
the receipt's `extra lifecycle or fault records`. Its directory is kept
as `~/ron-data/lab-captures/2026-09-29-run416-fault1`. The second take:
`success: true`, exit 0, 999 s from launch to exit, 13,049 frames, map 7
and seed 12345 verified, five files restored. The dump (493.8 MB, 258
blocks: the window and the shutdown block 13049) and the trace (42.4 MB)
were moved into `Logs` as `gamelog-run416-greatsahara-12783.txt` and
`rontrace-run416.log`.

| check | result |
|---|---|
| `rngcmp.py` against `rontrace-run383.log` | **0 differing**, 13,049 identical |
| window blocks | 257, 12778..13034 |
| `GAME INFO` | `MAP_STYLE 7`, `DIFFICULTY 0`, seed 12345 |

**What it holds**: `run416_s_word_frame_is_widened_whole` (`diff::third`),
walked from run383's start with run381's head. Block 12778 stands on 133
keys, all standing families. The first parting past it is the AI's make
list on 12780 (`MAKE[1].val` 9999999 against 1632000), and the word's
block 12784 parts on 10 keys. The compared pin walks 12782..12786 and the
coverage driver drives 12782..12786.
## run419 — run346's game at run414's detail over blocks 6316..6572: the second pair's East Indies word 6321 widened (2026-09-29, item 1127)

**What it is.** run414's shape on the click-free lane: run346's lobby,
`cover=0`, the detail `end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`,
blocks 6316..6572, `!quit` at 6586. The word's frame 6321 writes block
6322: six blocks before it and 250 after.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-29-run419 \
    --map 18 --end-frame 6586 --timeout 4000 --log-window 6316 6573 \
    --detail end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1 \
    --profile DIFFICULTY=5
```

**The booking cites what the disk could not answer.** run414 ends at block
6226. No dump prints the word's blocks.

**Taken** 05:55–06:15, one take, waited on with `waitrun.sh`. `success:
true`, exit 0, 1,186 s from launch to exit. The dump (580.0 MB) and the
trace (49.7 MB) were moved into `Logs` as
`gamelog-run419-islands-toughest-6321.txt` and `rontrace-run419.log`.

| check | result |
|---|---|
| `rngcmp.py` against `rontrace-run346.log` | **0 differing**, 6,587 identical |
| window blocks | 257, 6316..6572 |
| receipt | `map_style 18`, seed 12345, lobby `DIFFICULTY=5`, five files restored |

**What it holds**: `run419_s_word_frame_is_widened_whole` (`diff::second`),
walked from run346's start. Block 6316 stands on 169 keys, from the gap
6227..6315. Among them the barge `1/42`, carrying `1/32`, sails apart. The
word's block 6322 parts on 15 keys: the original has landed `1/32` and
closed the barge, and ours still carries both (`docs/AI.md` §87.2). The
compared pin walks its 6320..6324, and the coverage driver drives
6321..6324.

## run424 — chapter thirty-nine's staging, a packet at logger frame 619 (2026-09-29, item 1111)

**What it is.** `chapter39.cmd`'s births without its `@spell` lines and,
as taken, without its `tech` line, to 625, with a `RON_STATE_FRAME=619`
packet (814,832,508 bytes; run144's plan). 60 s launch to exit, `success:
true`, 626 frames. The lane lock was stale.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-29-run424 \
    --map 14 --end-frame 625 --log-window 605 625 --timeout 2400 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file <chapter39.cmd's births> \
    --tracer-def RON_STATE_FRAME=619 \
    --tracer-def 'RON_STATE_PLAN="/Users/rf-studio/ron-data/lab-experiments/2026-09-23-item-597/plan/plan.h"'
```

**What the disk could not answer.** Whether a Citizen of the golden start
may take To Arms under `library who=0 1`, what each cast writes, and what
the tech verb's type parse admits: no dump prints `is_castable` or
`has_tech`, and no capture had issued an untargeted craft.

**What it answered** (`tools/recomp/step4.py`'s machinery, scratch):
`is_castable(To Arms)` 0 on both Citizens with `has_preq(MILITIA)` 1 and
`has_tech` 0; after `Leader::gain_tech(MILITIA)`, 3. The console `tech`
verb's category string, `tubs`. `cast_civilian` on the Militia: `rare` 0 →
50, `set_type(50, 0)`, one `Random::get(0, 0xffff)`, `form` 9;
`cast_to_arms`: `rare` 50, `set_type(66, 0)`; `cast_create_decoy`: the
Slingers' copy at (7800, 34296) first, then a Steam stats call the stubs do
not answer. `docs/GOLDEN.md` §48.

## run422 — chapter thirty-nine, the spell issuer's untargeted crafts (2026-09-29, item 1111)

`docs/GOLDEN.md` §48, `tools/gamelog/golden/chapter39.cmd`: a cast of its
own on the golden start — `library who=0 1`, `tech who=0 militia on`, two
Citizens, a Militia, a General, a Hoplite and a Slinger squad (604..616);
To Arms on each Citizen (620, 640), Civilian on the Militia and on the
first Citizen (660, 700), Create Decoys (720).

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch39 \
    --map 14 --end-frame 1100 --log-window 605 1100 --timeout 3600 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter39.cmd
```

**Three takes** at `cover=0` on the click-free lane, each `success: true`:
- 07:59, deleted: `rontrace.cmd` held only `!ffwd` and `!quit`, since the
  script's lines were written only inside its header comments.
- 08:10, deleted: `tech who=0 militia` only reports without `on`
  (`run_cmd`'s `parse_binary`); both To Arms presses issued, none refused
  at the command, and neither laid an order.
- **08:28, kept**: 651 s launch to exit, **224 MB**, 1,101 frames, the
  pool printed.

| check | result |
|---|---|
| `cmdsran.py` | every chat line returned 1 |
| `issuesmatch.py --none-refused 5` | five `@` lines, none refused |

### §48's falsifiers

| check | predicted | observed |
| --- | --- | --- |
| 1, the issues | 621, 641, 661, 701, 721 | **as predicted** |
| 2, the order | one `CASTORDER`, no move | **as predicted** on 622 (`spell_time` 1) |
| 3, To Arms | a Militia, `rare` 50, `form` 0 by 627 | **626** and B **646** |
| 4, C's Civilian | a Citizen, `rare` 50 | **666** |
| 5, A's Civilian | `rare` still 50 | **706** |
| 6, G's order | `mana_burn` +1,000, no bucket down | **as predicted** on 722, recovering one a frame |
| 7, the decoys | two squads, Slingers first | **821**: `0/16`..`18`, `0/19`..`21` |
| 8, the age | +1 a frame | **as predicted** (1 on 821) |

Every Citizen of who=0 is at 50 hits and LOS 4 from 605, the block after
the `tech` line: `Object::update_hits`'s and `Unit::update_los`'s Militia
arm.

## run423 — chapter thirty-nine at `cover=1`, to 830 (2026-09-29, item 1111)

run422's script line for line on the queue lane, `cover=1`, to 830
(`tools/gamelog/captures.txt`, `run: 423`), 08:55–09:04:

| check | result |
|---|---|
| `issuesmatch.py` vs run422 | five issue records, none refused; golden 5 |
| `rngcmp.py` vs run422 | 831 frames identical, none differing |
| `report.py … functions` | `cast_to_arms` f625, `cast_civilian` f665, `cast_create_decoy` f820 |

The three casts leave `NEVER`, 142 → 139 (`crates/rondata/src/blind.rs`).
## run417 — run383's game at run414's detail over blocks 13177..13433: Great Sahara's long word 13182 widened (2026-09-29, item 1147)

**What it is.** run416's shape on the click-free lane: `--map 7`,
`--profile DIFFICULTY=0`, seed 12345, `cover=0`, the detail
`end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, blocks
13177..13433, and `!quit` at 13447. The word's frame 13182 writes block
13183, with six blocks before it and 250 after.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-29-run417 \
    --map 7 --end-frame 13447 --timeout 9000 --log-window 13177 13434 \
    --detail end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1 \
    --profile DIFFICULTY=0
```

**The booking cites what the disk could not answer.** run416 ends at block
13034 and run383 prints no record past its start, so no dump prints the
word's blocks.

**Taken** 07:28–07:46 in one take, waited on with `waitrun.sh` (exit 0).
The receipt: `success: true`, exit 0, 1,081 s from launch to exit, 13,448
frames, map 7 and seed 12345 verified, five files restored. The lane lock
was stale (its pid had gone and no wine was running). The dump (502.0 MB,
257 blocks) and the trace (43.3 MB) were moved into `Logs` as
`gamelog-run417-greatsahara-13182.txt` and `rontrace-run417.log`.

| check | result |
|---|---|
| `rngcmp.py` against `rontrace-run383.log` | **0 differing**, 13,448 identical |
| window blocks | 257, 13177..13433 |
| `GAME INFO` | `MAP_STYLE 7`, `DIFFICULTY 0`, seed 12345 |

**What it holds**: `run417_s_word_frame_is_widened_whole` (`diff::third`),
walked from run383's start with run381's head. Block 13177 stands on 238
keys, the make list parts on 13181, and the word's block 13183 parts on 18
keys (`docs/AI.md` §91.5). The compared pin and the coverage driver walk
13181..13185.

## run420 — run346's game at run419's detail over blocks 6604..6860: the second pair's East Indies word 6609 widened (2026-09-29, item 1143)

**What it is.** run419's shape on the click-free lane: run346's lobby,
`cover=0`, the detail `end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`,
blocks 6604..6860, `!quit` at 6874. The word's frame 6609 writes block
6610: six blocks before it and 250 after.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-29-run420-take2 \
    --map 18 --end-frame 6874 --timeout 4000 --log-window 6604 6861 \
    --detail end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1 \
    --profile DIFFICULTY=5
```

**The booking cites what the disk could not answer.** run419 ends at block
6572. No dump prints the word's blocks.

**Taken** in two takes. The first (06:49, `2026-09-29-run420`) exited 9
2.4 s after launch, before the menu — the pre-menu failure
`docs/lab/2026-09-09-autostart.md` records — and its receipt says `success:
false`; it is kept. The second (06:50–07:12, `2026-09-29-run420-take2`)
was waited on with `waitrun.sh`: `success: true`, exit 0, 1,301 s from
launch to exit. The dump (580.6 MB) and the trace (52.4 MB) were moved into
`Logs` as `gamelog-run420-islands-toughest-6609.txt` and
`rontrace-run420.log`.

| check | result |
|---|---|
| `rngcmp.py` against `rontrace-run346.log` | **0 differing**, 6,875 identical |
| window blocks | 257, 6604..6860 |
| receipt | `map_style 18`, seed 12345, lobby `DIFFICULTY=5`, five files restored |

**What it holds**: `run420_s_word_frame_is_widened_whole` (`diff::second`),
walked from run346's start. Block 6604 stands on 263 keys, from the gap
6573..6603. Among them `1/28` walks apart. The word's block 6610 parts on
13 keys: the original stands `1/28` against the animal `8/3`, and ours
walks on (`docs/AI.md` §90.3). The compared pin walks its 6608..6612, and
the coverage driver drives 6609..6612.

## run421 — run346's game at run420's detail over blocks 6567..6610: the gap before the second pair's East Indies word 6609 (2026-09-29, item 1156)

**What it is.** run420's shape on the click-free lane: run346's lobby,
`cover=0`, the detail `end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`,
blocks 6567..6610, `!quit` at 6624.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-29-run421 \
    --map 18 --end-frame 6624 --timeout 4000 --log-window 6567 6611 \
    --detail end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1 \
    --profile DIFFICULTY=5
```

**The booking cites what the disk could not answer.** run419 ends at block
6572 and run420 starts at 6604; `1/28` agrees on the one and is apart on
the other, so the frame it parts on and the order it took were in the gap
6573..6603 that no dump covers.

**Taken** 08:22–08:26, after waiting 530 s for item 1111's capture to free
the lane; waited on with `waitrun.sh`: `success: true`, exit 0, 225 s from
launch to exit. The dump (99.4 MB) and the trace (49.8 MB) were moved into
`Logs` as `gamelog-run421-islands-toughest-gap6573.txt` and
`rontrace-run421.log`.

| check | result |
|---|---|
| `rngcmp.py` against `rontrace-run346.log` | **0 differing**, 6,625 identical |
| window blocks | 44, 6567..6610 |
| receipt | `map_style 18`, seed 12345, lobby `DIFFICULTY=5`, five files restored |

**What it holds**: `run421_s_gap_is_walked_whole` (`diff::second`), walked
from run346's start. On block 6577 the original's `1/28` holds `orders_x/y`
(26616,22776), its leg to (26592,22752), cell (34,29); the old tree sent it
to (28920,25080), and since item 1156 it agrees (`docs/AI.md` §93,
`docs/SCOUT.md` §8.1).

## run430 — chapter forty, the casts' other arms (2026-09-29, item 1167)

`docs/GOLDEN.md` §49, `tools/gamelog/golden/chapter40.cmd`: a cast of its
own on the golden start — `library who=0 1`, `peace 1`, To Arms on a
Citizen before `tech who=0 militia on` (606) and on a Citizen D added on
who=1's land (612); four Militia (614..620); `@repair`, `@build` and
`@gather` on three of them (630, 640, 650), `@garrison` of the fourth into
the City (660); D walked off who=1's land (700), To Arms and Civilian on it
(800, 840); `@alarm` on the City twice (880, 900). `@alarm` and `@gather`
are the DLL's new verbs 24 and 25, built from item 1167's tree.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch40 \
    --map 14 --end-frame 1150 --log-window 605 1150 --timeout 3600 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=2,GROUPS=1 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter40.cmd
```

10:20–10:33, 739 s launch to exit, `success: true`, 1,151 frames, 231 MB;
the lane lock was stale. Eleven issue records, none refused.

**What the disk could not answer.** No capture wounds a converting unit,
presses To Arms off the Militia line or on another's land, issues a
Militia into a repair, a build or a gather, or presses the City's alarm.
The staging's predicates were printed fields on run422's dump — the City's
`city_flags` 18449 (bit 0 set, 0x40 clear), who=0's timber 242 on 640,
who=1's land at cells (45..46, 26) — or answered by run422 (a Militia takes
Civilian), so no packet was taken.

**What it holds**: `chapter_forty_holds_to_the_golden_word` (1150, the
capture's end), `chapter_forty_s_word_frame_is_widened_whole` and
`run430_s_casts_are_the_original_s_field_for_field` (`diff::golden`).

## run431 — chapter forty at `cover=1`, to 910 (2026-09-29, item 1167)

run430's script line for line on the queue lane, `cover=1`, to 910
(`tools/gamelog/captures.txt`, `run: 431`), 10:50–10:59. The first launch
refused before the game: the install's DLL predated the two verbs, and
`runqueue.sh`'s tracer guard said so; `tools/trace/build.sh` rebuilt it
from item 1167's tree.

| check | result |
|---|---|
| `issuesmatch.py` vs run430 | eleven issue records, none refused; golden 11 |
| `rngcmp.py` vs run430 | 911 frames identical, none differing |
| `report.py … functions` | `Group::action_gather` f650, `Group::action_alarm` f880 |

The two issuers leave `NEVER`, and `CommandManager::issue_gather` with them
(`ENTERED_BY_THE_DLL_ONLY`): 141 → 138 (`crates/rondata/src/blind.rs`).
Run 432, reserved for a packet, was not taken.
## run425 — run346's game at run420's detail over blocks 7377..7633: the second pair's East Indies word 7382 widened (2026-09-29, item 1164)

**What it is.** run420's shape on the click-free lane: run346's lobby,
`cover=0`, the detail `end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`,
blocks 7377..7633, `!quit` at 7647. The word's frame 7382 writes block
7383: six blocks before it and 250 after.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-29-run425 \
    --map 18 --end-frame 7647 --timeout 4000 --log-window 7377 7634 \
    --detail end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1 \
    --profile DIFFICULTY=5
```

**The booking cites what the disk could not answer.** run420 ends at block
6860; no dump of run346's game prints the word's blocks at detail.

**Taken** 09:17–09:39 in one take, the lane lock stale (its holders
exited). It was waited on with `waitrun.sh`: `success: true`, exit 0,
1,270 s from launch to exit. The dump (589.6 MB) and the trace (69.9 MB)
were moved into `Logs` as `gamelog-run425-islands-toughest-7382.txt` and
`rontrace-run425.log`.

| check | result |
|---|---|
| `rngcmp.py` against `rontrace-run346.log` | **0 differing**, 7,648 identical |
| window blocks | 257, 7377..7633 |
| receipt | `map_style 18`, seed 12345, lobby `DIFFICULTY=5`, five files restored |
| `GAME INFO` | `DIFFICULTY 5`, `MAP_STYLE 18` |

**What it holds**: `run425_s_word_frame_is_widened_whole` (`diff::second`),
walked from run346's start. Block 7377 stands on 238 keys, from the gap
6861..7376. Player 1's make list parts first: `MAKE[].val` on 7379 (slot 0
ours 1800000 against 4194000), its `city` on 7381, and on the word's block
7383 its buckets and `1/12`'s order. The window parts on 1,182 keys. The
coverage driver drives 7382..7385.
## run418 — run383's game at run414's detail over blocks 14582..14838: Great Sahara's long word 14587 widened (2026-09-29, item 1163)

**What it is.** run417's shape on the click-free lane: `--map 7`,
`--profile DIFFICULTY=0`, seed 12345, `cover=0`, the detail
`end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, blocks
14582..14838, and `!quit` at 14852. The word's frame 14587 writes block
14588, with six blocks before it and 250 after.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-29-run418 \
    --map 7 --end-frame 14852 --timeout 11000 --log-window 14582 14839 \
    --detail end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1 \
    --profile DIFFICULTY=0
```

**The booking cites what the disk could not answer.** run417 ends at block
13433 and run383 prints no record past its start, so no dump prints the
word's blocks.

**Taken** 09:39–09:58 in one take, after a 15-minute wait behind item
1164's run425 on the capture lane. It was waited on with `waitrun.sh`
(exit 0). The receipt: `success: true`, exit 0, 1,064 s from launch to
exit, 14,853 frames, map 7 and seed 12345 verified, five files restored.
The dump (529.1 MB, 257 blocks) and the trace (49.2 MB) were moved into
`Logs` as `gamelog-run418-greatsahara-14587.txt` and `rontrace-run418.log`.

| check | result |
|---|---|
| `rngcmp.py` against `rontrace-run383.log` | **0 differing**, 14,853 identical |
| window blocks | 257, 14582..14838 |
| `GAME INFO` | `MAP_STYLE 7`, `DIFFICULTY 0`, seed 12345 |

**What it holds**: `run418_s_word_frame_is_widened_whole` (`diff::third`),
walked from run383's start with run381's head. On the tree merged with
item 1111, block 14582 stands on 155 keys, and the word's block 14588
parts on 6, led by `1/52`'s collision with `1/30` in the original
(`docs/AI.md` §94.6). The compared pin and the coverage driver walk
14586..14590.

## run426 — run383's game at run414's detail over blocks 15581..15837: Great Sahara's long word 15586 widened (2026-09-29, item 1171)

**What it is.** run418's shape on the click-free lane: `--map 7`,
`--profile DIFFICULTY=0`, seed 12345, `cover=0`, the detail
`end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1`, blocks
15581..15837, and `!quit` at 15851. The word's frame 15586 writes block
15587, with six blocks before it and 250 after.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-data/lab-captures/2026-09-29-run426 \
    --map 7 --end-frame 15851 --timeout 12000 --log-window 15581 15838 \
    --detail end:MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,LEADERS=9,GROUPS=1 \
    --profile DIFFICULTY=0
```

**The booking cites what the disk could not answer.** run418 ends at block
14838 and run383 prints no record past its start, so no dump prints the
word's blocks.

**Taken** 11:03–11:24 in one take. The lane lock named item 1167's
`longtrace.sh`, whose two pids had both exited, so the launch took it over
and there was no wait. It was waited on with `waitrun.sh` (exit 0). The
receipt: `success: true`, exit 0, 1,220 s from launch to exit, 15,852
frames, map 7 and seed 12345 verified, five files restored. The dump
(532.8 MB, 257 blocks and the closing one) and the trace (54.6 MB) were
moved into `Logs` as `gamelog-run426-greatsahara-15586.txt` and
`rontrace-run426.log`.

| check | result |
|---|---|
| `rngcmp.py` against `rontrace-run383.log` | **0 differing**, 15,852 identical |
| window blocks | 257, 15581..15837 |
| `GAME INFO` | `MAP_STYLE 7`, `DIFFICULTY 0`, seed 12345 |

**What it holds**: `run426_s_word_frame_is_widened_whole` (`diff::third`),
walked from run383's start with run381's head. Block 15581 stands on 159
keys, and the word's block 15587 parts on `1/29`'s figure clock, after its
head order parted on 15585 (`docs/AI.md` §97.5). The compared pin and the
coverage driver walk 15585..15589.
## run436 — chapter forty-one's first take: `be 0` on the issuing frame (2026-09-29, item 1182)

`docs/GOLDEN.md` §50. The chapter's first script:
- who=1's Airbase, Biplane, Barracks and Citizen;
- who=0's trireme, Hoplites and a Catapult;
- `be 1` / `@flight 1 2006 1 6` / `be 0`, all on 620;
- the trireme's and the Catapult's `@attack` on the Barracks;
- `be 1` / `@repair 1 2007 1 7` / `be 0` on 720.

It ran on the click-free lane at `cover=0`, `--end-frame 1600`, with
`GROUPS=1` in its `end:` detail. It took 1,601 frames and wrote 286 MB of
dump and 11 MB of trace. The verifier then refused the take, because
`GROUPS` was asked for and no `GROUPDATA` block printed (parked 735). The
dump and the trace are whole, and they are kept at
`~/ron-golden/ch41-run436`.

**What it holds.** The DLL issued both who=1 commands with no refusal,
and the dump logs `process_flight 621` and `process_swarm_around 2007 1 2
13 721`. Yet the Biplane holds no order on any block to 1600, and the
Citizen never takes the repair. `CommandManager::process_turn` stamps the
package with `console->play` as the pump walks it, and by then `be 0` has
put the seat back. `CommandPackage::process_group` then drops a group
that is not that player's. This is what
`run436_s_other_seat_s_commands_are_dropped_at_the_pump` (`diff::golden`)
walks, with run436's own lines.

## run437 — chapter forty-one, CENSUS row 7's last seven (2026-09-29, item 1182)

`docs/GOLDEN.md` §50, `tools/gamelog/golden/chapter41.cmd`: run436's cast
without the Catapult. `be 0` comes a frame after each who=1 command (621,
901), and the repair is at 900.

```
zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch41 \
    --map 14 --end-frame 1770 --log-window 605 1770 --timeout 3600 \
    --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=2,AMMO=5,DEATHS=1 \
    --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
    --detail misc:COMMANDMANAGER=1 \
    --cmd-file tools/gamelog/golden/chapter41.cmd
```

13:42–13:57, 888 s from launch to exit, `success: true`, 1,771 frames,
321 MB. It waited fifteen minutes for the lane, which another lane's
run442 held from 13:22 to 13:42.

**What the disk could not answer.** No capture on this disk sends a
computer's plane on a sortie, or strikes a building while an enemy of
the attacker repairs it. `get_speed` and `is_siege` are `NEVER` rows.

**What it holds**:
- `chapter_forty_one_holds_to_the_golden_word` (839);
- `chapter_forty_one_s_word_frame_is_widened_whole` (513 rows);
- the coverage driver's windows (`diff::coverage`).

## run438 — chapter forty-one at `cover=1`, to 1140 (2026-09-29, item 1182)

run437's script line for line on the queue lane, `cover=1`, to 1140
(`tools/gamelog/captures.txt`, `run: 438`), 14:06–14:16. `be` is a chat
command the DLL hands to `parse_cmd`, so the DLL needed no rebuild.

| check | result |
|---|---|
| `issuesmatch.py` vs run437 | every record as in the golden run, none refused |
| `rngcmp.py` vs run437 | 1,141 frames identical, none differing |
| `report.py … functions` | `ObjectData::is_siege` f1088, `UnitData::get_speed` f778 |

The two leave `NEVER`: 138 → 136 (`crates/rondata/src/blind.rs`).
