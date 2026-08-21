# Reading `Logs\gamelog.txt`

Scaffolding for the behavioural checks, not architecture. The real reader is
`crates/rondata/src/gamelog.rs`, which parses the same file into typed records
and is what the diff harness uses; these three are for driving a check by hand
and reading the answer out in one line. See `docs/ORACLE.md`, "The detail level
is the knob", for what the log contains and how to make it contain it.

All three default to this machine's bottle path:
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

## Driving the game

`cheat.sh X Y "<rest of the cheat line>"` aims the mouse at `X,Y` (the tile
`add`/`move` will use) and sends `cheat <rest>` through the chat box; pass
`- -` to leave the mouse alone. `waitwin.sh [out.png]` blocks until the game
window exists, focuses it, and writes a screenshot plus a downscaled copy.
`shot.sh` takes a screenshot, whole-desktop or `-r X Y W H`, and downscales it.
`click.sh X Y` left-clicks; `rclick.sh O X Y` selects object `O` and
right-clicks, which is how a unit is given an order.

`aim.py A B C D --from cx,cy tx,ty ...` turns a tile into the desktop point to
click. The projection is linear in `u = wx − wy` and `v = wx + wy`; `A` and `B`
come from the zoom, `C` and `D` from the camera. Calibrate once with two
right-click anchors, then re-fit `C`/`D` after any camera move with a single
`cheat add NEW tower` probe — a tower's even footprint makes its logged
`x_internal / 192` the cursor's tile exactly. `docs/ORACLE.md`, "A scripted
placement test", is the whole recipe, including the tower-before-city control
that tells a terrain refusal from a rule refusal.

Two things about input that cost a run each, both in `docs/ORACLE.md`:
**keys go through `osascript … keystroke`, clicks through `cliclick`** (the
other way round does not reach the game), and **a fast `cliclick c:` often
does not register** — press and release with a hold instead:

    cliclick dd:X,Y w:250 du:X,Y      # or:  cliclick m:X,Y w:400 c:X,Y

`cliclick` will not box-select several units however it is driven, so any
check needing more than one unit under one order needs a human to drag the
box. Also: the window is 1920×1080 inside whatever the desktop is, and its
position changes between launches — locate the buttons from a screenshot
rather than trusting stored coordinates.

## Reading the decompile

`rd.sh <Class/method@addr> [from] [to]` prints a function from the Ghidra
export at `~/ghidra-projects/decomp/funcs/` with the `String` constructor
boilerplate stripped, which is most of the noise in this binary.
