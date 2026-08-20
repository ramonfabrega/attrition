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
