# Reading `Logs\gamelog.txt`

Scaffolding for the behavioural checks, not architecture. The real reader is
`crates/rondata/src/gamelog.rs`, which parses the same file into typed records
and is what the diff harness uses; these are for driving a check by hand
and reading the answer out in one line. See `docs/ORACLE.md`, "The detail level
is the knob", for what the log contains and how to make it contain it.

The readers default to this machine's corpus path:
`~/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/`. Its only
other copy is the R2 bucket `backup.sh` mirrors it into — `zsh
tools/gamelog/backup.sh sync` after a capture session, `pending` for what
that would send without sending it, `restore` on a new machine, `ls` to see
what is there. Ctrl-C is safe in every mode and a re-run resumes. It shows
a live byte counter on a terminal and one line per file everywhere else, so
a lane's tool result stays readable without a flag.

## `setlog.py` — write `gamelog.ini`

    setlog.py <DUMP_ALL> end:CAT=..,.. [start:..] [misc:..] [endgame:..]

Every category in a named section that is not listed is set to 0. The values
are **detail thresholds**, so a bare name means 1 and the level goes in the
argument:

    setlog.py 0 end:UNITS=3,BUILDS=7,CITIES=5,GUYS=2,DEATHS=1,LEADERS=9

`DUMP_ALL=1` dumps everything every frame and hangs the game; use it only with
`[Start Game]` and `InitialDump=1`, which is how the type tables and
`COMBATTABLE` were captured.

**`GROUPS` is a per-frame record and it does not need `DUMP_ALL`** — 684 KB
a frame and ~0.6 frames a second on run31, where `DUMP_ALL` is 80 MB and two
minutes. But it will come out empty unless `DEATHS` is **off**, and the two
cannot both be cheap. `GroupData::log_data` sets no type of its own, and
`dump_deaths` ends by calling `WorldData::log_data` twice, so the pool is
accepted against **`WORLD`**'s threshold rather than its own — and raising
`WORLD` far enough to let it through also dumps the whole per-cell map every
frame, three times over. `docs/ORACLE.md`, "The group pool is a per-frame
record", has the trap and the line that works.

## `gl.py` — look at the raw file

    gl.py frames                     count BEGIN FRAME, print first/last/size
    gl.py first <BLOCK> [n] [file]    the first `BEGIN <BLOCK>` and n lines, flattened
    gl.py nth <BLOCK> <k> [n] [file]  the k-th
    gl.py grep <regex> [max] [file]
    gl.py lines <a> <b> [file]

## `track.py` — one field per object per frame

    track.py KIND FIELDS [--where k=v,..] [--frames a-b] [--every n] [--changes]

`KIND` is the record's `BEGIN` name (`UNITDATA`, `BUILDDATA`, `CITYDATA`,
`LEADERDATA`); `FIELDS` a comma list, nested blocks prefixed (`guy.type`).
`--changes` prints an object only when its selected fields differ from its
previous frame, which is usually what a check wants:

    track.py BUILDDATA job_counter,helpers,constr_time --changes
    track.py UNITDATA damage,myhits,attrition,supply --where who=0 --changes

The parser tracks indentation to know which `BEGIN` block a key belongs to and
skips the repetitive array blocks (`BUILDQUEUE`, `STACK<TYPE>`).

## `attr.py` — attrition and supply, checked against the cadence

    attr.py [gamelog] [--who N] [--o N] [--from F] [--to F]

For every unit whose `attrition`/`supply`/`damage` moves: the period
timeline, every attrition tick (a rise of `damage*16 + damage_frac` by a
squad-size delta with no new `damage_frame`) and the phase-lock fit —
`(sim + o) % period == 0` with `sim = label − 1`, the offset-0 fit printed
as the control — and the combat rises separately. A drop in the total is an
object number recycled and restarts the history. Run16: 489/489
(`docs/ATTRITION.md`, last section).

## `steps.py` — the production AI's step machine

    steps.py [gamelog] [--who N] [--from F] [--to F] [--terse] [--all]
             [--names types.tsv]

One block per frame out of a `LEADERS=9` dump: `production_step`,
`script_step`, `prod_script_run`, the goods picture `production_ai_setup`
writes (`econ`, `rate`, `shortages`, `worst_good`/`best_good`, the six
buckets, incomes and caps), the census headline, `num_queued`, the eleven
`MAKEOBJECT`s **by slot number** and the ten sites. `--terse` compares only
the step machine's own fields, which is what you want — the ledger's
buckets tick every frame and would otherwise print every block. `--names`
takes an `index<TAB>name` table so `t` reads `50=PEASANTS`; build one from
the Ghidra export's `enums/TypeIndex.txt`.

The slot numbers are the point: the make list is a ranked four (0–3) over
one-per-category slots (`list[cat]`), so entries at 0, 5 and 8 with nothing
between them is `MakeList::make_me` behaving exactly as read. `docs/AI.md`
§15 and `docs/RUNS.md`, "The producers' run", are what this was written
for.

## One frame at a time

`track.py` follows a field across a run. When the question is instead "what is
on the map *now*", cut the last frame out of the growing log and look at it:

    lastframe.py                     the last complete frame → $TMPDIR/ron-last.txt
    objs.py                          one line per object: kind, o, who, tile, hits, type
    groups.py                        the live GROUPDATA records: members, off, curr, angles
    one.py BUILDDATA 0 2008 flags job_counter

`one.py` exits non-zero when the object is not in the frame, which is the
answer to "was the site disbanded?". Object numbers are **per player** — units
from 0, buildings from 2000 — so an object is (kind, who, o), never o alone.

Those four take a frame cut by `frame.py` or `lastframe.py`, which **strip the
indentation**. `danger.py` reads the archive itself and keeps it, because the
dump nests by depth and the danger map's question is about whole records:

    danger.py map   FILE FRAME [--differ|--same]   the map in each FULL DUMP
    danger.py units FILE FRAME [--types A] [--least N]

A `DUMP_ALL` block carries the world twice — `GameLog::begin_frame` before the
frame ran and `GameLog::end_frame` after — so `map --differ` asks whether the
rebuild between them changed anything, and exits non-zero when it did not.
`units` lists the units whose type has the military bit `role & 0x10000` with
the half-cell each indexes and what every leader's row holds there; `--types`
borrows the `UNITTYPE` table from a `DUMP_ALL` archive, which a cheap window
has none of. `docs/DANGER.md` §2 and §8.1.

## The sync stream

    rngtrace.py [gamelog] [cap]   every `game_random seed` the log carries,
                                  with the draws between consecutive records

The setup path's `say_checksum` records (`check_all_level=14` in `rise.ini`,
`[Misc Logging] CHECKSUM=2`) and, under `DUMP_ALL`, the per-frame ones;
the draw count is a forward walk of the 32-bit LCG. `docs/ORACLE.md`, "The
setup path's checksum trace is the RNG state".

**The draw's site, not its outcome.** `tools/trace/` (its own README) is
the in-process trace: every `Random::get` with its caller and the frame,
and function coverage of the whole executable. `report.py <log> draws 0`
names frame 0's 120 draws in order; `report.py <log> blind docs/` lists
the functions the documents cite that no traced run has entered.
`docs/ORACLE.md`, "The draw-site trace and function coverage".

**A run a person has to touch.** `live.sh N` launches the traced exe with
whatever the inis and `rontrace.cmd` already say, drives the lobby, and **returns with the game running** — where `runwin.sh` waits for a
`!quit` and archives. It exists for the one capture the cheat channel cannot
make on its own: a right-click on a multi-unit selection, which has to come
from `cliclick` (`docs/GROUPS.md` §6.4). `archive.sh N TAG` is the other end —
kill, name the log, restore the window, print the map style and the frame
count. The selection itself *is* scriptable: `select <type> who=0` from the
channel, and `select <type> who=0 +` to append.

**Staging a window run in one call.** `window.py stage LO HI` writes every
setting a `DUMP_ALL` window needs — `gamelog.ini` (`DUMP_ALL=1`, `[Start
Game] WORLD=6`), `rise.ini` (`InitialDump=1`), `rise2.ini` (the two frame
keys), `rontrace.cfg` and a `rontrace.cmd` that fast-forwards to the window
and quits after it — and `window.py restore` undoes them. Run22
(`docs/ORACLE.md`) is its first use: eight minutes for a three-block window
at frame 3579. `window.py frames LO HI` is the **cheap** window: the two
`rise2.ini` keys and the start dump alone, leaving the detail levels to
`setlog.py`. A per-frame dump at the `[End Frame]` thresholds costs
hundreds of KB a frame rather than `DUMP_ALL`'s 80 MB, so that window can be
hundreds of frames wide rather than three, and run31 used it.

**A `DUMP_ALL` window.** `LogStartFrame` / `LogEndFrame` under
`[RISE OF NATIONS]` in **`rise2.ini`** (not `gamelog.ini`'s `Checksum
Dump`/`Checksum Break`, which are a one-shot dump and an `int 3` keyed on the
checksum record index) gate the whole per-frame `full_dump`: inclusive start,
exclusive end, default −1 for "every frame", and the keys are never written
back so you add them by hand. `LogStartFrame=95 LogEndFrame=105` with
`DUMP_ALL=1` gives ten frame blocks at ~61 MB each and nothing for frames
0–94 — `gamelog-run13-window-95-105.txt`. A block `FRAME n` holds the end of
sim-frame n−1 then the start of sim-frame n, so `[a, b)` measures the draws
of sim-frames a … b−2. `docs/ORACLE.md`, "The frame window is real".

**Reading a window.** The dump nests by indentation (no `END` lines), and
the `STACK<TYPE>` block does not indent its contents; these four read it:

    passes.py window.txt out.json     every DUMP_ALL pass: frame, seed, each
                                      unit's position/orders/guys, the buildings
    framediff.py out.json [frame ...] per sim-frame: the draw count and every
                                      unit whose state changed (cur_time ticks
                                      filtered out)
    draws.py <seed-hex> N [a-b ...]   the LCG from a word, each draw with its
                                      %100 (idle variant), %1000, %5, %4, %3
    farms.py window.txt block ...     the `Farms` list at a block: owner, empties,
                                      every non-empty cell (print index is the
                                      column-major position)
    anims.py dump.txt [who/o ...]     the animation clock (`docs/ANIM.md`): per
                                      pass, each named unit's guys — piece, slot,
                                      cur/end time, last_time, flags — and the
                                      pass's seed
    anims.py dump.txt --lengths       every (gpiece, slot) → end_time the dump shows
    anims.py dump.txt --wraps         every clock reset between two passes: the
                                      same slot again (a silent restart or an idle
                                      re-roll) or a new one

A pass is one `full_dump` — the start-of-game one, then the end of
sim-frame n−1 nested under `FRAME n` and the start of sim-frame n after it
(the same state, printed twice); `anims.py` labels each `F<frame>p<pass>`.

The method is `docs/SYNC.md` §4.1: `framediff` says what changed on a
frame, `draws` says what every draw would have read, and a state change
that only one offset reproduces places the draw. Run13's frames 99–103
were attributed this way in an afternoon.

## Combat

    hits.py frames.txt        every damage increment, with the flank geometry
    trace.py frames.txt 1 48  one unit's damage/frac/hits/angle, changes only
    heading.py frames.txt 0 8 is `angle` the direction of travel?

`hits.py` prints the increment in **sixteenths** (`damage*16 + damage_frac`),
the victim's `angle`, the attacker named by `damage_o`/`damage_who`, and
`d = victim.angle − find_angle(victim − attacker)` in degrees, which is the
quantity `docs/COMBAT.md` §6 step 19 branches on. The attribution is only as
good as `damage_o`, and **`damage_o` does not update per hit** — a tower's
arrows arrive labelled with whichever unit last set the field — so a
measurement wants exactly one damage source on the target.

## Driving the game

`cheat.sh X Y "<rest of the cheat line>"` aims the mouse at `X,Y` (the tile
`add`/`move` will use) and sends `cheat <rest>` through the chat box; pass
`- -` to leave the mouse alone. `waitwin.sh [out.png]` blocks until the game
window exists, focuses it, and writes a screenshot plus a downscaled copy.
`shot.sh` takes a screenshot, whole-desktop or `-r X Y W H`, and downscales it.
`click.sh X Y` left-clicks; `rclick.sh O X Y` selects object `O` and
right-clicks, which is how a unit is given an order.

`con.sh X Y "<command>"` sends a line to the `~` console instead of the chat
box — no `cheat ` prefix, and it reaches the console-only commands (`ai off`,
`human <who>`, `coord`, `pause`, `break`, `ffwd`, `quit`). `?` there lists the
whole command table; `docs/ORACLE.md` transcribes it.

**Prefer tile coordinates to aiming.** With `Console Coord Mode=2` in
`rise2.ini` (or `coord t` once in the console) `add` reads its `x,y` as tiles,
so `cheat add NEW tower 56,156` needs no mouse at all, and `cheat add 1 citizen
32,141` puts a builder exactly where it is wanted. What still needs the mouse
is the *order* — `cheat camera 32,137` then a right-click at the viewport
centre, desktop `(1719, 574)` on this machine's window.

`aim.py A B C D --from cx,cy tx,ty ...` turns a tile into the desktop point to
click, for the cases tile coordinates cannot cover. The projection is linear in `u = wx − wy` and `v = wx + wy`; `A` and `B`
come from the zoom, `C` and `D` from the camera. Calibrate once with two
right-click anchors, then re-fit `C`/`D` after any camera move with a single
`cheat add NEW tower` probe — a tower's even footprint makes its logged
`x_internal / 192` the cursor's tile exactly. `docs/ORACLE.md`, "A scripted
placement test", is the whole recipe, including the tower-before-city control
that tells a terrain refusal from a rule refusal.

Input, in one line: **keys go through `osascript … keystroke`, the mouse
through `cliclick`** — System Events clicks do not reach the game — and a fast
`cliclick c:` often does not register, so press and release with a hold:

    cliclick dd:X,Y w:250 du:X,Y      # or:  cliclick m:X,Y w:400 c:X,Y

**The lobby's buttons are not in one place, and `lobby.sh` is where they
live.** This machine has two desktops — 3440×1440 with the main monitor on,
1920×1080 without — and the game opens windowed at (760, 152) on the first
and full-screen at (0, 0) on the second, so a script carrying one desktop's
coordinates clicks on nothing on the other and sits on the Main Menu until
it is killed (run32, ten minutes). Every drive script now sources it:

    source "$W/tools/gamelog/lobby.sh"
    lobby_init || exit 1          # measures the screen, picks the table
    lobby_click solo 4 "$T/solo.png"
    lobby_click quick 8 "$T/quick.png"
    lobby_start 20 "$T/"          # two presses on the wide desktop, one here

`lobby_shot FILE` takes the game's own region on whichever desktop is up. A
width neither table knows is an **error**, not a guess: screenshot the
lobby, downscale it with `sips -Z 900`, read the button off the image and
multiply by `width / 900`, then add a row. The Map Style combo is measured
on the wide desktop only, and needs no measuring on the other — the style
can be set in `PlayerProfile/Player.dat`'s `<MULTI>` block and `check.ini`
with the game closed, which is faster and cannot mis-click
(`roadcapture.sh`).

There is no multi-select and none is needed: orders are per unit and persist,
so `rclick.sh` run once per unit puts several on one job. The rest of what
bites — the window's size and position, the modal rename dialog, the
console's effect on the mouse, screenshots of the whole desktop — is one list
in `docs/ORACLE.md`, "Traps that cost a run each".

## Reading the decompile

`rd.sh <Class/method@addr> [from] [to]` prints a function from the Ghidra
export at `~/ghidra-projects/decomp/funcs/` with the `String` constructor
boilerplate stripped, which is most of the noise in this binary.
