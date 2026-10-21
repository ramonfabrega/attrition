# The rubric

`tools/rubric.py` is a steering pass's holistic scorecard of how much of
the original's simulation this port has verified: three tables, each from
data the tree or the machine's archive already holds, and one `Rubric:`
line at the end. It is a measurement tool — it changes no score, pins
nothing, and no gate runs it. Its rules are the module docstring's; this
page says what each table means and what it does not.

## 1. By mechanic

**What it measures.** The PDB's source files are the exhaustive mechanic
list. Every function the census files under the simulation layer
(`tools/census.py --layers`, its rule 5) is counted under its source file,
a header grouped with the `.cpp` of the same stem. Per group: functions,
backed (by doc and by sweep, exactly the census's count — the table's
totals equal the `Census:` line), and the unbacked rest split by the mark
`census.py --unbacked` gives — dead weight, `ui`, `operator`, `cut`,
carried (the crate defines the method's snake-case name) and unmarked (it
defines no such function). Bytes are the census's proxy, the decompile
file's size. The docs column names the mechanic documents citing any of
the group's functions by address, most cites first; ledgers (queue,
parked, journal, runs, census, decisions, loop) and subdirectories count
in `+N` only. Sorted by unbacked bytes, largest first.

**100 %** is a group where every function is backed or marked dead
weight, `ui` or `operator`. A `cut` body is *not* exempt here: the brief
named the three marks, and the seven `cut` names would move one group at
most.

**What it does not measure.** Backed is the census's floor — a function
a coverage section or a sweep cites — never a claim that its predicates
were checked. A group at 100 % can still be wrong; one at 0 % can be
dead in every lobby this project plays. Bytes weigh decompiled C, not
machine code.

## 2. By age

**What it measures.** Per age 0–7 (Ancient … Information), how many
`(dump, frame)` pairs of the pinned captures hold an active human, an
active computer, or either leader in that age. The pinned captures are
`rondata::blind::TRACES`' runs; each trace's `gamelog-<run>-*.txt` dumps
are read from the archive (`$RON_GAMELOG_DIR`). The age is each
`LEADERDATA` block's `ages_get()` — how many of the seven later ages the
leader owns (`docs/TECH.md`, "Per-player state"); `rondata::diff::leader`
reads it, `gamelog.rs`'s `LeaderDump` does not carry it. Active and human
are `leader_flags` bits 0x1 and 0x4, the flags written before the block
they belong to (`docs/ORACLE.md`). A dump below `LEADERS=2` prints no
`ages_get()` and contributes nothing.

**100 %** would be every age held for at least one frame.

**What it does not measure.** Frames, not play: a window capture's
hundred frames in one age weigh more than a long capture dumped sparsely.
It says nothing about the dumps the diff harness pins (`rondata::diff`),
only the traces' own — the two sets overlap but are not the same.

## 3. By unit type

**What it measures.** Every `GUY` block's `type` line in the same dumps,
against the 364 records of the install's `unitrules.xml` (type id 50 + the
record, `docs/FORMATS.md`), with each never-seen type grouped by its
`DOMAIN` and the loaded `age` from the start-of-game types dump
(`gamelog-run3-fulldump-types.txt`; `-1` for the gaia animals, `?` when the
archive lacks it). A type is **seen** when any guy of it appears in any
pinned dump, start-of-game or per frame.

**100 %** would be every unit type on the map at least once.

**What it does not measure.** Presence, not exercise: a type that stood
in a start dump has not fought, been trained or upgraded in a dump. The
twenty-odd nation variants of one unit (`GRAPH` differs, `NAME` repeats)
are separate types and are listed separately.

## The first run

On `9bbda88d9`'s tree and the archive of 2026-10-21, 58 dumps (14.9 GB)
read in 89 s; `rontrace-run15.log` is pinned and has no gamelog dump. The
three tables' summary lines and the `Rubric:` line, verbatim:

```
groups 112 · at 100 % 36 (marked *) · functions backed 1696 of 3611 · bytes backed 4317313 of 11392782 (37.9%) · unmarked 452 (the crate defines no function of the name)
leader records with an age: 67431 in 58 dumps · ages never held: 2,3,4,5,6,7
seen 30 of 364 · never 334 · ids outside the unit range seen: 0
Rubric: mechanics 36 of 112 source groups at 100 % (backed 1696 of 3611 fns, 37.9% of bytes) · ages held 2 of 8 (never: 2,3,4,5,6,7) · unit types seen 30 of 364 (334 never)
```

Read plainly: every pinned frame is Ancient or Classical, and no capture
has a leader past the Classical age; thirty unit types have ever stood in
a pinned dump. The first table's 36 groups at 100 % hold 407 functions,
131 backed and 276 dead weight, `ui` or operators; four of them have no
backed function at all, and `gameaccess` (114) is the only one past 22.
