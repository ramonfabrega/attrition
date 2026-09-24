# The recorded order stream, into the harness

**What this establishes.** A recorded game and a per-frame gamelog of *the
same run* are now both on disk, and the recording's commands drive the
harness's simulation. `docs/RECGAME.md` §5 left "a recording made by *this*
install of a fixed-seed gamelog run" open; `docs/DATALAYER.md` called the
order stream "the single thing the diff's score cannot move without". This
closes both.

**Confidence.** High for the pairing and the wire (the recording decodes to
exact size, and its embedded 493×493 combat table equals the one composed
from this install, cell for cell). High for the frame convention, which is
measured rather than assumed. Honest and partial for the *mapping*: two of
the eleven command kinds in the sample reach the simulation today, and the
other nine are named seams, counted in the report rather than dropped.

**The one thing this changes about the plan** is in §1, and it is not small.

---

## 1. The AI is not in the stream, and cannot be

Every class in the shipped executable that calls `CommandManager::issue_*` or
`add_command` is a user-interface class or the turn pump itself:

```
ChatBox  Console  ConsoleWin  EndGameWin  GroupOut  IFaceDiplo  IFaceDiploNeg
LeaderOptionOut  LeaderOut  Options  Popup  UnitOut  ScenarioFuncSet
+ CommandManager, TurnControl, Game
```

**No AI class appears.** The corroborating detail is
`CommandManager::issue_cheat_ai_toggle`: turning the AI on or off is itself a
*replicated command*, which is only necessary if the AI runs identically on
every client from the same simulation state. This is the ordinary lockstep
arrangement — orders are replicated, the AI is re-derived — and it means:

- **A recording replays the AI by re-simulating it.** The stream contains the
  AI's *effects* nowhere and its *decisions* nowhere.
- **A recorded-game diff of any match containing an AI player therefore needs
  the AI implemented.** `CLAUDE.md`'s queue puts AI last "because it is the
  least oracled". That reason still holds, but the queue's implication — that
  the diff's score can be driven up without it — is wrong. Player 0's units
  can be driven from the stream; player 1's cannot, ever, by any recording.
- An AI-versus-AI recording is close to empty: a camera command a frame and
  nothing else. This was checked before it cost a session.

What a recording *is* good for is exactly the human half, and that is what
this document wires in.

## 2. The paired run

`gamelog-run7-ancient-nubian-orders.txt` (260 MB, 1,732 frames) and
`Playback - 2026.08.24 10'15'53 (Mon).rcx` (66 KB, 1,732 packages), captured
2026-08-24. Same lobby as run6 — `check.ini`, seed 12345, Nubians, Easiest,
Small (2/3-player), Small Town, Ancient Age — so run7 differs from run6 in
exactly one variable: it has input in it.

**A recording holds one package per frame**, so equal counts is the cheapest
possible guard that a gamelog and a recording describe the same game, and the
harness checks it before feeding anything in.

### 2.1 Recording is a profile preference, and it was already on

`Game::run@00584590` +257 gates the recorder on
`player_profile.prefs & 0x1000000`, which `OptionsWinGame::do_save` writes and
`do_load` reads — the **"Record games"** checkbox on Options → Game. It was
already ticked on this install, so **every session since 2026-08-20 has been
recording**: fourteen `.rcx` files were sitting in
`~/Documents/My Games/Rise of Nations/Recorded Games/` before this run.
(`PlayerProfile::get_record_game_directory` builds that path under
`CSIDL_PERSONAL`; CrossOver maps it to the Mac's `~/Documents`.)

### 2.2 gzip is the "was it quit cleanly" fingerprint

`RecordGame::close@00953990` calls `finalize`, and `Game::run` calls it at
+1157, after `end_game_close` — the normal end-of-game path. `finalize` is
the only place the file is repacked as gzip; until then it is written raw
(`docs/RECGAME.md`, second reading). So sniffing two bytes classifies a
recording:

| first two bytes | meaning |
|---|---|
| `1f 8b` | `finalize` ran — the game was quit through the menu |
| anything else | raw — the process was killed |

Of the fifteen recordings on this machine, the two gzipped ones are exactly
the two runs that were quit through the in-game menu, and the thirteen raw
ones are exactly the runs that were `pkill`ed. That is the second reading's
correction confirmed against fifteen files at once, for free.

Raw files still read: they are missing only the repack, not the content.

## 3. The frame convention, measured

The gamelog's `FRAME n` is the state at the **end** of frame `n`, numbered
from 1. A recording's packages are numbered from 0. A command in the package
for recording frame `f` is processed by the game frame the gamelog reports as
`FRAME f + 1`.

Measured, not assumed: the recording's `MoveTo` for unit `0/1` is on frame
376, and the un-fed harness's first order disagreement for that unit is at
gamelog frame **377**. Same for the scout — command on 476, disagreement on
477 — and for `0/2`, 396 and 397.

So `Stream::apply(f)` is called immediately before the tick that produces
`FRAME f + 1`.

## 4. The selection is state

A `group` command sets the issuing player's selection, and the unit-targeted
commands that follow act on it. `num = 0` — an empty `objects` — means "the
same selection as this player's previous group command"
(`docs/COMMANDS.md` §3). That was derived from the wire; run7 exercises it:

```
frame 1064 play 0  Group { who: 0, objects: [2000] }
frame 1064 play 0  QueueUp { unit: 50, num: 1 }
frame 1068 play 0  Group { who: 0, objects: [] }      ← "same as last"
frame 1068 play 0  QueueUp { unit: 50, num: 1 }
```

so the selection has to be carried across frames exactly as the engine
carries it, and `crate::input::Stream` does.

Object numbers are per player — units from 0, buildings from 2000 — so a
selection entry is only meaningful together with the issuer's `who`.

## 5. What run7's stream contains

46 input commands over 31 frames, after dropping the two per-frame
housekeeping kinds (`Camera`, one a frame; `PlayerSpeed`, the turn pump's):

| kind | n | into the simulation? |
|---|---|---|
| `Group` | 15 | yes — sets the selection |
| `Chat` | 13 | no — a cheat line; its effect is `ConsoleWin::run_cmd`'s, not an order's |
| `MoveTo` | 9 | **yes — nine move orders** |
| `QueueUp` | 3 | no — production is not wired into the harness |
| `SwarmAround` | 2 | no — the target site was created mid-run by a cheat and is not in the simulation |
| `Buy`, `Sell` | 1 each | no — the market is not modelled |
| `GatherPoint` | 1 | no — rally points are not modelled |
| `LeaderOptions` | 1 | no — not mapped |

Each "no" is counted by name and reason in the report. A command the harness
carries and ignores is a named gap, never a silent drop.

### 5.1 Three things the sample confirms that no earlier one could

- **`QueueUp.unit` is the `Guy` kind index.** The two Citizens are
  `QueueUp { unit: 50 }` and the Caravan is `{ unit: 59 }`; the gamelog's own
  records for the units that appeared are `guy=50` and `guy=59`. Two
  independent readings of the same field, agreeing.
- **`Group { objects: [] }` occurs in the wild** — §4.
- **`SwarmAround.ox` is an object number in the target owner's numbering**,
  with `whom` the owner: `SwarmAround { ox: 2007, whom: 0 }` names exactly
  the site the gamelog reports as `BUILDDATA o=2007 who=0`.
- **`MoveTo.to_x/to_y` are internal units**, the simulation's own space:
  `to_x: 3373, to_y: 34075` is 17.57, 177.5 tiles at 192 units a tile, and
  the gamelog puts that unit at tile (17.6, 177.5) after the order.

## 6. What feeding it did

Same run, same dump, with and without the stream:

| unit | un-fed | fed |
|---|---|---|
| `0/0` (scout) | frame 477, `Length { ours: 0, theirs: 1 }` | frame 477, `PathTo { ours: (936, 32184), theirs: (955, 32167) }` |
| `0/1` | frame 377, `Kind { ours: 7, theirs: 1 }` | frame 377, `PathLength { ours: 10, theirs: 7 }` |
| `0/2` | frame 397, `Kind { ours: 7, theirs: 1 }` | frame 397, `PathTo { ours: (4296, 33480), theirs: (4290, 33465) }` |

The order's *existence and kind* now agree where they did not: `ours: 0`
became a real order, and `Kind 7` (GATHER) became the move the original
issued. What is left on those units is path-level, and §7 is why.

Two figures the single score does not show: order disagreements fell
14,888 → 14,085, and path-stack disagreements rose 5,316 → 12,464. The rise
is not a regression — it is surface that did not exist before, because a unit
with no order has no path to disagree about.

`ticks before divergence` is still 1, and will stay 1 while player 1 is an
AI: §1.

## 7. A finding this run produced, not yet acted on

**The path stack's goal is the un-snapped click, not the snapped
destination.**

The scout at frame 477, straight out of the dump:

```
MOVEORDER   x 936   y 32184        the destination, snapped to its 48-unit cell
            orig_x 955  orig_y 32167    the raw click
STACK<TYPE> length 2
  PATHDATA[0]  to_x 955   to_y 32167   tolerance 0    flags 1
  PATHDATA[1]  to_x 2472  to_y 32184   tolerance 384  flags 0
```

`MoveOrder.dest` is snapped and **the harness computes it exactly right**
(955 → 19×48+24 = 936). But the bottom of the path stack holds `orig` — the
raw click — and the harness pushes the snapped `dest` there instead.

This fits what is already written down. `docs/ORDERS.md` §714 has the snap,
§725 has `orig_x/orig_y` as "the un-snapped point the caller asked for
(`action_move_near` passes the click; `add_move_order` passes −1, −1)", and
§8.2 has the command path handing `orig = click` with `pathed = 1`. So the
rule is plausibly *"the path goal is `orig` when the caller supplied one,
the snapped `dest` otherwise"* — and the un-fed baseline supports it, having
only 40 `path-to` disagreements in 20,784 unit-frames, because internal
moves pass no `orig`.

**Not implemented, deliberately.** It is a change to the pathfinder, which
has its own document, its own audit, and pins on run6 (two woodcutters, 432
and 427 frames). It belongs in a focused pass that re-runs those pins, not
in the session that happened to notice it. §725's parenthetical also wants
correcting: for a command-issued move `orig` is the click, not −1/−1.

## 8. What is not established

- **Nine of the eleven command kinds.** `QueueUp` needs production in the
  harness; `SwarmAround`/`Repair` need mid-run sites; `Buy`/`Sell` need the
  market; `GatherPoint` needs rally points; `LeaderOptions` is unmapped. Each
  is counted in the report, so the gap is measured rather than assumed.
- **`Chat` as a cheat line.** In solo a cheat travels the order stream and is
  therefore *reproducible from the recording* — `cheat add NEW tower` is in
  run7's stream and it is what created object 2007. ~~Replaying it would need
  `ConsoleWin::run_cmd`, which is not a simulation mechanic. Until then a
  recording of a cheat-staged run cannot be replayed faithfully, which is an
  argument for capturing future ground-truth runs without cheats.~~ Answered
  by §11 (item 364): a listed set of cheats **is** modelled now, and the
  rules track stages its ground truth with them on purpose
  (`docs/DECISIONS.md` entry 41 §3). What §11 does not do is read them back
  out of a `.rcx` — the golden record's cheats come from the script file, and
  a `Chat` command in a recording is still counted and skipped.
- **Multiplayer.** `decode_mp`'s seed-keyed XOR is still unexercised: every
  recording on this machine is single-player and plain
  (`docs/COMMANDS.md` §5).
- **`queued`'s group semantics.** The wire's value is mapped straight onto
  `QueuePos` (`docs/ORDERS.md` §1.5), but `Group::action_swarm_around` and
  `action_move_near` implement `QUEUE_FIRST` for a *group* differently —
  detach, halt, recurse with `QUEUE_NEW`, re-append. Every `MoveTo` in run7
  is `queued: 2` (`QUEUE_NEW`, a plain click), so the sample does not
  exercise the difference.
- **The formation split.** §8.2 of `docs/ORDERS.md` sends a group of two or
  more land units to `add_group_move_order` instead. Run7 selects one unit at
  a time, so every move here took the lone-unit path and nothing about
  `GroupMoveOrder` is tested.
- **Whether the harness's `play` is the right key.** Non-`Group` commands are
  attributed to the package's `play`; `Group` carries its own `who`. In
  single player they are both 0 and the sample cannot distinguish them.

## 9. Where the artifacts are

Outside the repo, per `CLAUDE.md` — nothing from the install enters it.

| what | where |
|---|---|
| `gamelog-run7-ancient-nubian-orders.txt` | the bottle's `Logs\`, as the other dumps (`$RON_GAMELOG_DIR`) |
| `Playback - 2026.08.24 10'15'53 (Mon).rcx` | the bottle's `Logs\`, kept beside its own dump (`$RON_RECGAME_DIR`) |

**A recording is kept with the other captures, not where the game writes it.**
The original writes to `PlayerProfile::get_record_game_directory`, which
CrossOver maps to the Mac's `~/Documents` — and macOS gates that directory
behind a consent dialog. The first `open` under it blocks until a human at the
machine clicks Allow, so a background or SSH run sits at 0 % CPU looking hung
for as long as it is left to (2026-08-28: ~30 minutes of one session). The
corpus therefore lives in the bottle's `Logs\` with the dumps and traces, and
`diff::tests::recording` looks there first; `~/Documents` stays as the
fallback, which is where a *fresh* capture is found before it is archived.

`rondata <install> --recgame <file> --gamelog <dump> --diff` runs the whole
thing, and `--recgame` alone now prints the input as a transcript — every
command with its frame and player, which is the readable form of "what did
the players do".

## 10. A correction to `docs/ORACLE.md` on the way

The stored screen↔world anchor is wrong for the current window placement.
`cheat camera X,Y` centres tile `(X, Y)` at the viewport centre, and
`docs/ORACLE.md` records that centre as desktop **(1719, 574)**. Re-measured
2026-08-24 by the documented probe — `cheat add NEW tower` at the cursor,
then reading the site's tile back out of the dump — it is **(1720, 620)**: a
tower placed at (1720, 620) landed on tile (16, 160), which is the tile
`cheat camera 16,160` had just centred. The x is right and the y is 46 out.
The window moves between launches (`docs/ORACLE.md`, "Traps that cost a run
each"), so the lesson is the one already written down — re-probe rather than
trust a stored anchor — and the probe takes one cheat line.

## 11. The staged channel, interpreted (2026-09-18, item 364)

`docs/DECISIONS.md` entry 41 §3 **reverses §8's last bullet for the rules
track**: ~~a recording of a cheat-staged run cannot be replayed faithfully,
which is an argument for capturing future ground-truth runs without
cheats~~ — a cheat is now a *modelled input* from a small, listed set, and
this section is the model. The orders still come from the `.rcx` through
[`crate::input`]; the cheats come from the `rontrace.cmd` script through
`crate::golden`, and the two never overlap, because ~~no console command
issues an order at all~~ **exactly one console command issues an order —
`bird`, table case 82, calling `Unit::add_air_patrol_order@005e4350`, and no
chapter of the record uses it for anything else** (`docs/ORACLE.md`, "The
channel's vocabulary"; `docs/GOLDEN.md` §13, item 365).

**How this was established.** `ConsoleWin::run_cmd@007d6a70`'s cases read in
the Ghidra export — `ai` 0xc, `ally`/`peace`/`war` 0x2c–0x2e,
`human`/`computer` 0x31/0x32, `age` and the four epoch verbs and `library`
0x37–0x3c, `add`/`insert` 0x4d/0x4e — with `ConsoleWin::parse_who@007e3ad0`,
`ConsoleWin::parse_coord@007e5390`, `ConsoleWin::parse_type@007e4050` and
`Objects::init_unit@0065e0c0` for the argument grammar and the spawn.
`tools/gamelog/console.py table` re-derives the 102-entry command table from
the install. **Diff-backed**: chapter one staged into the harness and walked
against its own trace for 900 frames (`crates/rondata/src/diff/golden.rs`).

**Confidence.** High for the four verbs chapter one uses, each of which a
run confirms. Read-only for `tech`, the epoch verbs, `human`/`computer` and
the building arm of `add` — implemented, unit-tested, and **not** yet
exercised by a capture. Everything else in the channel is refused by name.

### 11.1 The script

`<sim-frame> <text>`, `#` a comment anywhere on the line, `!` selecting the
console-only half of `run_cmd`'s two disjoint switches, lines in file order,
**a frame below its predecessor's clamped up**. `rontrace.dll` hands each
line to `ConsoleWin::parse_cmd(·, from_chat, no_mouse = 1)` at
`Game::do_frame`'s entry, before phase 1 — so a staged draw is the frame's
*first*, which is what `Script::stage` reproduces by holding the marks
across `Sim::tick`'s own clear. Item 364 measured the difference: chapter
one's `add hoplite` spends three `Guy::init_real+0x52` draws, and marked the
ordinary way they were the frame's first three and invisible to the fold.

The two halves are **disjoint** — 56 console-only cases, 45 chat-reachable —
so a line on the wrong side reaches a case that is not there. The
interpreter refuses it rather than running it anyway.

### 11.2 `parse_who`: a bare number is not always a player

`parse_who(arg, default)` computes `param_2 = (default < 0) ? 0 : 1`; a
`who=` prefix sets it to 1 and is stripped; and the digit arm at the tail is
reached **only when it is 1** — `if (param_2 == 0) goto <return the
default>` stands immediately above it. So:

| call site | default | `age 3` reads as |
|---|---|---|
| `age`'s first token | `-1` | the **level**, for player 0 |
| `age`'s token after the level | `console->who` | a **player** |
| `ally`/`peace`/`war`'s target | `console->who` | a **player** |
| `add`'s `who=` slot | `-1` | not a player at all |

`age who=1 8` and `age 8 1` both set player 1's eighth age; `age 3` sets
player 0's third. The original also matches the eight player names, the
eight colour names and `gaia`; the interpreter says the number.

### 11.3 `parse_coord`: a bare number is a **tile**, and the runbook is wrong

`parse_coord@007e5390` has three arms: a leading `c` (or a bare digit under
`coord_mode == 3`) is a raw internal coordinate; a leading `t` (or a bare
digit under `coord_mode == 2`) is **`n × 0xc0 + 0x60`**, a tile centred; and
anything else is `n × 0x300 + 0x180`, a world cell centred.

**This install takes the tile arm**, and it is measured rather than read:
run101–run105 staged `add hoplite who=0 4,40` and the dump puts the squad's
head at `(888, 7800)`, which is `(4 × 192 + 96, 40 × 192 + 96)` plus
`find_nearby_spot`'s own offset. The world arm would have asked for
`(3456, 31104)` — eighteen tiles away, on the other side of the map.

That **corrects `docs/ORACLE.md`**'s stored `internal = arg × 768 + half a
footprint` for the staged channel, and it closes the open question
`docs/journal/2026-09-18-item-363.md` left ("the coordinate argument did not
calibrate the way the runbook says"): the runbook records the *world* arm and
the channel reads the tile one.

### 11.4 `ai off` is one flag with three readers

`run_cmd` case 0xc calls `CommandManager::issue_cheat_ai_toggle`, so the
cheat travels the order stream and `Game::action_cheat_ai_toggle@005930c0`
does the whole of the work: `ai_off = !ai_off`, once, in a solo game. Its
readers in the simulation's own territory are three:

- **`Leader::production_ai@006c1960:15`** bails to the switch's `default`
  arm — the step machine cleared, and return. It does **not** stop
  `plan_strategy`'s sweep, so `census`, `check_orphaned_buildings` and
  `compute_sites` keep their draws; that is why the golden record's frame 0
  is unchanged by the line and its frame 1 is not.
- **`Unit::think@005f6e40:205`** opens
  `if ((leader_flags & 4) != 0 || ai_off != 0)`, whose only unconditional
  statement is `if ((unit_masks & 0x40000) == 0) goto <return>`.
  ~~The arms inside are a computer leader's alone~~ — **`leader_flags & 4`
  is not the computer-leader test** (2026-09-19, item 437;
  `docs/COMBAT.md` §28.1). Measured identically on run112, run105 and the
  AI-on control run104, so it is a lobby property: **who=0, the human,
  carries the bit set** (`0x00000007`) and **who=1, the computer, has it
  clear** (`0x03000013`). What the bit *is* was **not** established. The
  claims below rest on the old reading and are left standing and flagged
  rather than dropped; each is owed a re-check. So for a human leader with
  the cheat on the block is exactly one thing: a unit that is not AI-driven loses the
  whole tail — `think_fish`, `think_merchant`, `think_scout`, `think_carry`,
  the army join. **Everything above the line is untouched**, which is what
  run101–run105 measured as "auto-engage survives AI-off".
- **`Leader::diplomacy@006bc950`**, which this crate does not model.

The cost of the line, measured on the golden record: with the AI on, frame 1
is 54 draws — eight `MathUtilFuncSet::rand_int` and thirty-six
`Leader::produce_building` — and with it off, 12. Frames 0 and 1 of run104
(the control, the same script with `0 !ai off` deleted) are identical to this
crate's own, draw for draw.

SEAM: the toggle is a *replicated command*, so the original flips the flag
when the turn pump walks the package; the interpreter flips it at the line's
own frame. No capture can separate the two — frame 0 is identical either way.

### 11.5 `add`: a squad is three units, and the count is not a count

`run_cmd` case 0x4d/0x4e reads `[#] typename [who=RED] [x,y]`, where the
leading count is a count only when it parses as one (`String::number`'s 0
falls back to 1) and is capped at 300. Then, once per count:

- a **unit** type takes `UnitType::find_nearby_spot(x, y, 0, 0xc00, 0,
  0x55555555, FILTER_NOT_ME, −1, −1, …)` and then
  `Objects::init_unit(who, type, spot, −1, −1, −1)`;
- a **building** type takes `Objects::init_build` and **breaks out of the
  count loop**, so a leading count places one building, not `num`.

`Objects::init_unit` is itself a loop over `UnitTypeData::uber_size`, each
pass a whole `Unit::init` with its own `Guy::init_real` draw, the units
threaded `o_down`/`o_up` — which the dump prints as `down`/`up`, and
run101–run105's three hoplites read `6 → 7 → 8` exactly so. So one `add
hoplite` line is **three units**, and `docs/ORACLE.md`'s "the leading count
is not a count" is the same fact seen from outside.

`parse_type` underscores-to-spaces the token and then walks the unit table
before the building table, asking `String::ignore` and retrying against the
name with its spaces purged. It is a **prefix** match, not an equality:
`add hoplite` names the type whose name is `Hoplites`.

~~SEAM: the original seats the squad's members with `find_nearby_spot` around
the captain before `init_unit` returns; [`sim::Sim::init_unit`] leaves them
on the captain's point until a formation or `come_out` moves them.~~
**Closed 2026-09-18, item 379** — `docs/ANIM.md` §6.3. The seating is
modelled and all six of chapter one's coordinates are pinned:
`(888, 7800)`, `(1032, 7800)`, `(936, 7944)` for `who=0` and
`(1368, 7992)`, `(1512, 7992)`, `(1416, 8136)` for `who=1`. The search
takes no draw, so the stream never knew the difference — but the second
`add`'s **own** spot did: with the first squad stacked the near ground
stays free and this crate put `who=1`'s captain at `(1080, 7800)`,
eighteen tiles from the original's.

### 11.6 `ally`/`peace`/`war`, and the bare form that does nothing

Cases 0x2c–0x2e are one body. With **no further token** it prints the
diplomacy table and changes nothing; with `all` it calls
`Leader::set_diplo(console->who, ·, level)` on every other live leader; with
a target it calls it once. The level is `ally` 2, `peace` 1, `war` 0.

**Chapter one's `604 war` is the bare form**, so it is a no-op — and the two
players were already at war from the lobby (`docs/ARMY.md` §16.3: "a Quick
Battle already starts at war"). The squads engage on the frame they are born
because of that, not because of the line.

**The targeted form, measured** (run171, item 660, `docs/GOLDEN.md` §12).
`peace 1`, `war 1` and `ally 1` move `diplos[1]` of who=0 and `diplos[0]`
of who=1 together, on the block after the line: `Leader::set_diplo@006ec6a0`
writes both slots. **`ally` with the lobby's only other player is an allied
victory**: at level 2 the call counts the live leaders allied to neither
side over `leaders.list[0..8]` (its bound `0xe71af0` is eight `Leader`s
past `leaders`), gaia's leaders 8 and 9 are outside it, and at zero it calls
`Leader::victory`, which ends the game. This crate's `Sim::set_diplo`
writes the row and not the victory; nothing staged past a two-player `ally`
has a frame to compare.

### 11.7 What the golden record's harness must refuse

The golden record's `GAMEINFO` is **byte-identical to run11's** — map 14,
seed 12345, size 2, every one of the 34 option fields — and its setup
checksum word is the same `0x3bd39ae9`, because the script's first line runs
at frame 0's `do_frame` entry, *after* `Game::init`. Two consequences:

- The **setup** is genuinely shared, so `borrow_from_siblings` handing the
  golden capture run11's checksum trace is right, and it is what seeds the
  stream entering frame 0. `the_golden_record_s_setup_is_run11_s` checks the
  borrowed word against the run's own trace rather than assuming it.
- The **frames** are not shared, and nothing in the borrow can tell:
  `frame_seeds`/`frame_guys` would hand the staged run fourteen of run12 and
  run13's per-frame words, which `Built::tick` then *installs*. The caller
  refuses them by hand. Item 364's first measurement was an artefact of
  exactly that — a "word" of 7 that was the correction, not the simulation.

No pinned number rests on it: five Great Lakes captures take the same
fourteen and run53's word is 9182 either way
(`refusing_the_borrowed_frame_stream_does_not_move_great_lakes_word`); East
Indies' setup word is `793793043` and it takes none.

### 11.8 What is not established

- **The residue at the word.** ~~Chapter one's golden word is **617** and one
  draw stands in the way: `Guy::set_anim@005da300+0xf2f`, the variant roll
  `set_anim` takes when the animation is `0xc` and a variant is asked for —
  the attack animation, on the frame the auto-engaged squads first swing.
  This crate does not spend it.~~ **Read 2026-09-18, item 379**
  (`docs/ANIM.md` §6.2): the roll is real and now modelled, but it is not
  the swing's — `Unit::fight` defers it into `GuyData +0x9e` and
  `Guy::move+0x166` pays it a frame later. The word **did not move**. With
  the squads seated where the original seats them (§11.5 above) the residue
  at 617 is no longer the roll at all: it is `Unit::fight+0x9b0` and the
  roll together, because this crate's squads do not engage on frame 616 the
  way the original's do. That is the next rules item, and it is a combat
  one. The value diff parts one frame later, at 618.
- **Seven verbs are parsed and not applied.** `die`, `damage`, `craft`,
  `move`, `resource`, `finish`, `hurry` — each is refused by name and
  counted, never silently dropped. `move` in particular is a *teleport*
  (`Unit::find_nearby_spot` then `Unit::set_new_location`), not an order.
- **The building arm of `add` is unexercised.** No chapter places one yet.
- **`coord_mode` is inferred from behaviour, not read.** `ConsoleWin::init`
  takes it from a preference whose default the decompile prints as 0, and
  the tile arm is what the run does. Which preference key, and whether a
  fresh profile would read differently, is unsettled — so a chapter that
  needs an exact point should say `c<internal>` and not rely on the mode.
- **`parse_type`'s match is a prefix here** on the strength of one
  observation (`hoplite` → `Hoplites`) plus `parse_who`'s shape. The
  `String::ignore` third argument was not resolved in the decompile.

### 11.9 What `ai off` leaves a human's civilians: all of it (2026-09-23, item 578)

**Measured, both directions, on a pair.** `docs/GOLDEN.md` §11 staged five
civilians for player 0 — Citizen, Caravan, Merchant, Scholar, Fur Trapper
(`0/6..0/10`, born on blocks 611–631) — once with `0 !ai off` (run141) and
once without it (run142, `chapter7_control.cmd`). **The five hold the same
order on every block of both captures**
(`chapter_seven_s_civilians_act_alike_with_the_ai_off_and_on`):

- the **citizen** takes a `GATHERORDER` on block 763 with `idle 12`, on the
  building `2001` (`build_type 418`). It walks and gets a timber slot on 801
  (`filled_gather_slots` 2 → 3, income 480 → 640), so the gather delivered;
- the **caravan, merchant, scholar and fur trapper** take no order at all,
  and are still standing at 1199 with `idle` 39, 56, 38 and 54.

**Why, read before the run and held by it.** Two facts, both in §11.4's
own function:

- **The citizen's arm is above the `ai off` block.** `Unit::think@005f6e40`
  runs `think_peasant` for a worker (TypeIndex 0x32–0x35) and
  `think_caravan` for a caravan at `:154`–`:165`, under only
  `leader_flags & 2`, before `:205`'s `if ((leader_flags & 4) != 0 ||
  ai_off != 0)`. `idle 12` is `think_peasant`'s human wait (item 494).
- **For the human the block is entered either way.** Item 437 measured
  `leader_flags & 4` set on who=0 (`0x00000007`) and clear on who=1, so for
  player 0 the `|| ai_off` term never decides anything: a human unit without
  `unit_masks & 0x40000` loses the tail whether the cheat is on or off.

So **`ai off` changes nothing a human's units do in `Unit::think`**; its
effect there is on the computer's units (bit 4 clear), and elsewhere on the
production AI (`:15` above). The draw streams show it: the pair shares
frame 0's 120 draws and parts on frame 1, 12 against 54 — the production
AI, as chapter one against run104
(`chapter_seven_s_pair_is_one_game_until_the_gate`).

**The harness agrees with both captures to their end, 1200**, draw for draw
and word for word, and it is not a blind agreement: this crate's `0/6`
takes its gather on 763 at `idle 12` and stands at the original's
(4920, 29016) on 900 and 1199, and its other four stand with the original's `idle`
counts at 1199, in both runs.

**What this overturns.** §11's premise — that the cheat leaves a human's
civilians inert — was written on the reading of bit 4 that item 437
retired. Its first falsifier ("a `GATHERORDER` or a `TRADEORDER` on any of
the five in the AI-off run") **fires**, on the citizen; its second ("no
order on any of the five in the control") does not. The gate is doing
what the corrected reading says; the chapter's premise is what was wrong.

SEAM, unexercised: this crate folds the block's predicate into `ai_off &&
!ai_driven(owner)` (`crates/sim/src/orders.rs`), so with the cheat **off**
it lets a human's units into the tail, where the original's exit still
closes it for any unit without `0x40000`. Nothing gave these five an order
in this crate's walk of run142 either, so no dump parts on it yet.

## 11.10 What `ai off` leaves a computer's civilians: all of it, by construction (2026-09-23, item 628)

**Read before the run, and measured on a pair.** `docs/GOLDEN.md` §11's
restage, chapter seven-b, staged §11.9's five civilians for **who=1**, once
with `0 !ai off` (run156) and once without (run157). It predicted in
`chapter7b.cmd`, committed before either capture, that the cheat's block
takes nothing from them either. Both captures held that.

**Why the block cannot close the tail for a computer's unit.** Three reads,
two of them on the dump:

- **`Unit::init@00612100:585` sets `unit_masks |= 0x40000` whenever
  `(leader_flags & 0xc) != 4`.** who=1's flags are `0x03000013`, so every
  unit the computer owns carries the bit from birth. The dump prints it
  every time it could. run141's who=1 units on 700 carry 262152 and
  262154. run146's cheat-`add`ed who=1 hoplites carry 331790. Here all five
  of the civilians carry it on their birth blocks, in both captures
  (`chapter_seven_b_s_civilians_act_alike_with_the_ai_off_and_on`).
- **So `Unit::think@005f6e40:206` is entered and its exit never taken.**
  For who=1 bit 4 is clear, so the block is entered only with `ai_off`.
  Its arms are guarded by `uVar4 != 0` (bit 4) and do not run. Its one
  unconditional statement is `:264`, `if ((unit_masks & 0x40000) == 0)
  return`, which is false for every unit of who=1. The tail runs exactly as
  without the cheat.
- **The citizen's arm is above the block anyway.** `think_peasant` runs at
  `:158` under `leader_flags & 2`, which both leaders carry. For an
  AI-driven unit its wait is `idle >= 1`, not the human's 12
  (`think_peasant@005f5760:16`–`:40`), and its search range is unlimited
  (`:91`–`:96`).

**What the pair measured.** In both captures:

- the **citizen** holds a `GATHERORDER` on its birth block, 611;
- the **caravan** and **scholar** never take an order;
- the **merchant** and **fur trapper** move on their birth blocks and later
  cast. The fur trapper casts on 1151 in both runs. The merchant casts on
  900 under the cheat and on 887 without it, the one difference in time
  among the five.

That is `docs/GOLDEN.md` §11's first falsifier firing, as predicted, and
its second not firing. The two captures are one game through frame 0 and
part on frame 1, 12 draws against 54: the production AI, as in chapter
seven's pair.

**What follows.** On this lobby the `|| ai_off` term of `:206` decides
nothing for **any** unit born to its current owner. A human's units have
bit 4, so they enter the block either way (§11.9). A computer's units carry
`0x40000`, so they leave it either way. The term can decide only for a unit
with `0x40000` clear under a leader with bit 4 clear. That is a unit born
to a plain human and later owned by the computer (conversion or capture),
or one whose leader's flags change after its birth. No `add` can stage it.
In the simulation's own territory, `ai off` reaches the units only through
`Leader::production_ai` (§11.4) and `Leader::diplomacy`.

**This crate's stand-in, read again.** `orders.rs` folds the block into
`ai_off && !ai_driven(owner)`, derived as `(x || ai_off) && !x` with `x =
ai_driven`. That reads bit 4 as "a computer leader", the reading item 437
retired. Under the corrected reading, `leader_flags & 4` is `!ai_driven` and
`unit_masks & 0x40000` is `ai_driven` (for a unit born to its owner, with bit
8's seam), so the original's predicate is `(!x || ai_off) && !x`, which is
`!x`. **The cheat drops out.** For who=1 the stand-in and the original agree
(the tail is open under both). That is why neither capture parts on it, and
building the block "for who=1" would take nothing. For who=0 without the
cheat the stand-in opens the tail where the original closes it. That is
§11.9's SEAM, still unexercised; the fix is the one-term `!ai_driven(owner)`
and wants its own gate against the long captures, whose human is who=0.

**Where each capture parts in this crate, booked by frame and draw delta**
(`docs/DECISIONS.md` 42). run156 parts at 1148: the fur trapper `1/10`'s
turn, 13 draws against 14. run157 parts at 1036: who=1's own `1/1`, 8
against 7. Neither is the seam (`docs/GOLDEN.md` §11).

## 11.11 `library`, `age` and the epoch verbs raise levels through a whole `gain_tech` (2026-09-23, item 644)

**The handler.** `ConsoleWin::run_cmd@007d6a70` case 0x3c, `library`, runs
`Leader::set_epoch` on the four lines and then `Leader::set_age`. `age` is
`set_age` alone, and the four epoch verbs are `set_epoch` alone. Both
setters share one body (`006d25a0`, `006d26f0`):

1. Lose every level of the row at or above the target, highest first.
2. Raise each level below it that is not held, through **`Leader::gain_tech(t,
   0, 0, ·, 1)`**. That is the whole call research makes, tail included.
3. Lose any age, unit or building whose prerequisites no longer hold. Then
   `reset_obs_flags`, `calc_unit_stats` and `calc_wall_stats`.

This crate had step 2 as the tree's bit-and-cascade alone
(`TechTree::set_age`/`set_epoch`). So none of `gain_tech`'s tail ran for a
staged level:

- the goods loop that pays a good's starting amount when its first
  prerequisite arrives (`Sim::pay_arriving_goods`; the Classical age pays
  knowledge and metal, 100 each);
- the economy flag `0x2000000` (`gain_tech:319`);
- the Science re-pricing, the unit upgrades and the age's snap.

Item 552 had wired the border half on the epoch verbs, and `age` had
none of it. **`Sim::set_leader_levels` now does all three steps**, with
step 2 through `Sim::gain_tech`.

**What it moved, diff-backed.**

- **Chapter seven-b's control, run157, 1176 → 1187.** On 1018 a goody pile
  of 75 (`docs/GOODY.md` §3) went to metal here, the good at 0, and to food
  there. The lottery draws the same four times on both sides, and only its
  inputs differed. On 1176 `place_woodcutter` could not afford its
  Woodcutter's Camp, at 23 food against 70, where the original had 98.
  (`docs/GOLDEN.md` §11.)
- **The standing first-block rows of every chapter that stages a level
  go.** That is knowledge and metal, 0 against 100:
  - chapter seven's who=0 (`library who=0 2`), 29 → 27 and 34 → 32;
  - chapter seven-b's who=1, together with the caps below, 34 → 27 and
    39 → 32;
  - chapter three's both players (`age who=N 2`), 30 → 26, and its
    restage the same.

**The caps read the Commerce level live** (parked 633).
`Leader::gather@006ce280:58` calls `calc_resource_caps` on every frame,
outside the reassembly gate, and `calc_resource_caps@006ce900` reads the
level off `LeaderDataEncrypt + 0xf0` directly. This crate read
`holdings.commerce`, which only a reassembly writes. So `library who=1 2`
showed 1392 on run157's 605 where the original showed 2992. The income
loop now writes the level into the holdings every frame before the caps
are taken.

- The five `resource_cap` rows on run157's and run156's first block go.
- Off the golden record, Great Lakes' AI timber accumulator agrees where
  it stood off by that lag:
  - run136's `bucket[1:timber]` and `leftover[1:timber]`, 250 → 248 under
    the word;
  - `leftover[1:timber]` in the leader windows of run107, run111 and
    run117.
- Both long words hold (East Indies 13640 after item 642, Great Lakes 12038), and so does
  every East Indies widening.

**What is not established.**

- The loss half (step 1) is unexercised: no staged line lowers a level.
- The Greeks' re-grant arm and the oil grant (Industrial) are reached by no
  capture.
- The camp's own purchase is diff-backed only through the draw stream: its
  180 shuffle draws and `1/7`'s orders, because the camp never reaches a
  dump block.
