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
  run7's stream and it is what created object 2007. Replaying it would need
  `ConsoleWin::run_cmd`, which is not a simulation mechanic. Until then a
  recording of a cheat-staged run cannot be replayed faithfully, which is an
  argument for capturing future ground-truth runs without cheats.
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
