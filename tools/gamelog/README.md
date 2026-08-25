# Reading `Logs\gamelog.txt`

Scaffolding for the behavioural checks, not architecture. The real reader is
`crates/rondata/src/gamelog.rs`, which parses the same file into typed records
and is what the diff harness uses; these are for driving a check by hand
and reading the answer out in one line. See `docs/ORACLE.md`, "The detail level
is the knob", for what the log contains and how to make it contain it.

The readers default to this machine's bottle path:
`~/Library/Application Support/CrossOver/Bottles/ron/drive_c/users/crossover/AppData/Roaming/Microsoft Games/Rise of Nations/`.

## `setlog.py` — write `gamelog.ini`

    setlog.py <DUMP_ALL> end:CAT=..,.. [start:..] [misc:..] [endgame:..]

Every category in a named section that is not listed is set to 0. The values
are **detail thresholds**, so the useful line for a check is

    setlog.py 0 end:UNITS,BUILDS,CITIES,DEATHS,LEADERS

followed by raising the ones that matter (`UNITS=3`, `BUILDS=6`, `CITIES=5` —
the script writes 1, edit up). `DUMP_ALL=1` dumps everything every frame and
hangs the game; use it only with `[Start Game]` and `InitialDump=1`, which is
how the type tables and `COMBATTABLE` were captured.

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

## One frame at a time

`track.py` follows a field across a run. When the question is instead "what is
on the map *now*", cut the last frame out of the growing log and look at it:

    lastframe.py                     the last complete frame → $TMPDIR/ron-last.txt
    objs.py                          one line per object: kind, o, who, tile, hits, type
    one.py BUILDDATA 0 2008 flags job_counter

`one.py` exits non-zero when the object is not in the frame, which is the
answer to "was the site disbanded?". Object numbers are **per player** — units
from 0, buildings from 2000 — so an object is (kind, who, o), never o alone.

## The sync stream

    rngtrace.py [gamelog] [cap]   every `game_random seed` the log carries,
                                  with the draws between consecutive records

The setup path's `say_checksum` records (`check_all_level=14` in `rise.ini`,
`[Misc Logging] CHECKSUM=2`) and, under `DUMP_ALL`, the per-frame ones;
the draw count is a forward walk of the 32-bit LCG. `docs/ORACLE.md`, "The
setup path's checksum trace is the RNG state".

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

There is no multi-select and none is needed: orders are per unit and persist,
so `rclick.sh` run once per unit puts several on one job. The rest of what
bites — the window's size and position, the modal rename dialog, the
console's effect on the mouse, screenshots of the whole desktop — is one list
in `docs/ORACLE.md`, "Traps that cost a run each".

## Reading the decompile

`rd.sh <Class/method@addr> [from] [to]` prints a function from the Ghidra
export at `~/ghidra-projects/decomp/funcs/` with the `String` constructor
boilerplate stripped, which is most of the noise in this binary.
