# The rubric

`tools/rubric.py` is a steering pass's holistic scorecard of how close
this port is to a 1:1 simulation of the original: six tables, each from
data the tree or the machine's archive already holds, and one `Rubric:`
line at the end — the territory's three numbers first (table 0, Loop
3727), then bytes in lockstep, ages held and unit types seen. It is a measurement tool — it changes no score, pins
nothing, and no gate runs it. Its rules are the module docstring's; this
page says what each table means and what it does not.

## Lockstep: which games count

From the fifty-first pass (Ramon, 2026-10-22) tables 2, 3 and 4 read only
**lockstep games**: a game whose pinned word stands at the game's full
length — the draw stream this crate plays agrees with the original's on
every frame of it, the shape the scoreboard writes `w24000 of 24,000`.
`tools/rubric.py --sources` prints every pinned word's game, whole or open,
and the runs it resolves; nothing is listed by hand. The words are read
from the Rust source the handoff's lines are checked against:

- every `Lobby`/`Pair` literal under `crates/rondata/src/diff/` —
  `battery::BATTERY`, the word modules, the pairs — with its word the lower
  of `count` and `sequence`, its runs the long capture's and the start's
  when the start is the same lobby (a battery lobby may borrow another
  lobby's start, and that start is not its game);
- every `AI_WORDS` row whose word is not a struct's (the second and third
  pairs, Great Sahara and its Toughest game): the word const resolved, the
  run from the `endpoint` test's `run<N>_` prefix, else from the const's
  comment, "every frame of run<N>'s trace";
- the `LONG_WORD_*` consts, their run and length from the same comment
  (`0..23999`: 24,000 frames). `FLOORS` carries neither a run nor a length;
  its games are the long words'.

A `cover=1` trace in `blind::TRACES` joins a whole game as its **twin**
when its comment says it is `run<N>'s game on every frame` of one of that
game's runs (the rngcmp the landing quoted). A twin whose comment says
"… on every frame before <frame>" or "… game before <frame>" is a staged
take that parts there — console `add`s, a carrier staged from 600 — and
does not join: its dumps after the staging are not the game.

On ``1f8f04ef9``'s tree every one of the 49 pinned games is whole: 99 runs,
99 dumps and 99 traces on the archive of 2026-10-22, nine of the runs
`cover=1` twins (run1016–run1022, run1024, run1062). The scored captures
are `cover=0` — draw stream, setup and closing records — so the dumps
of a game are its start, its closing block and whatever its twin printed.

## 0. The territory

**What it measures** (Loop 3727; Ramon at the sixtieth pass's talk,
2026-10-27). Every simulation-layer function — the census's layer rule,
3,611 functions — classified by what this port has done with it, per PDB
source stem, in `tools/census.py --territory` (`census.territory`;
`--json` for the same as data):

- **ported**: backed, and `crates/sim` consumes it (`is_port_backed`: a
  coverage section diffs it, or the crate spells the `Class::method`
  outside `sweep/`);
- **replaced by design**: not ported, in one of the five layers the project
  replaced on purpose — **the command packaging and manager**
  (`commandpackage`, `commandmanager`), **the turn pump** (`turncontrol`,
  `timesync`), **the XML loaders** (the simulation-layer bodies that walk a
  parsed document into a table: `Type::load_cost`, `Tribe::parse`,
  `Map::load_map_data` and twelve more, by name), **the scripting VM** (in
  this layer only the map script's hands; the VM itself is the AI layer),
  **the order classes' plumbing** (`orders`: `walk_data`, `get_type`, the
  `get_*_order` casts, `copy_order`; an order's semantics live in
  `unit.cpp` and `groups.cpp`, untouched). **Every group took the mark on
  Ramon's word at the talk**, reversible per group if a port is wanted
  after all; `census.REPLACED_BY_DESIGN` is the list. The mark never
  touches a ported or an excluded function;
- **open**: neither ported, replaced nor excluded — a sweep-backed
  **layout** (the offset identity asserted by a test, no consumer), a
  **name** the crate defines without backing, or **unmarked**. This is the
  work left;
- **excluded**: dead weight, UI, operator, cut — the census's own marks.

On this tree: **ported 529 · replaced by design 318 · open 1305 of 3611**
(layout 903, name 4, unmarked 398; excluded 1459: dead 1160, UI 235,
operator 57, cut 7). The replaced groups: the command packaging and
manager 168 (layout 140, open 28), the order classes' plumbing 97 (94, 3),
the turn pump 35 (23, 12), the XML loaders 15 (6, 9), the scripting VM 3
(1, 2). The `Rubric:` line carries the three numbers first, pinned as
`census::RUBRIC_TERRITORY_*`; the stems with the most open functions
(`unit` 106, `groups` 99, `map` 79, `world` 73, `game` 59) are the table's
head.

**What it does not measure.** Ported is the census's `port` test: a
consumer exists, not that its every predicate was checked (a ported
function can still be wrong). Open counts a layout-only sweep as open
because an offset identity is not a port — the number the old `Census:`
line's `backed` flattered by 1,167. Bytes are the decompile file's size.

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
in `+N` only. Sorted by unbacked bytes, largest first. It does not read
the lockstep games: backed is a reading's and a sweep's word, not a run's.

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
`(dump, frame)` pairs of the **lockstep games' dumps** hold an active
human, an active computer, or either leader in that age. The age is each
`LEADERDATA` block's `ages_get()` — how many of the seven later ages the
leader owns (`docs/TECH.md`, "Per-player state"); `rondata::diff::leader`
reads it, `gamelog.rs`'s `LeaderDump` does not carry it. Active and human
are `leader_flags` bits 0x1 and 0x4, the flags written before the block
they belong to (`docs/ORACLE.md`). A dump below `LEADERS=2` prints no
`ages_get()` and contributes nothing. An age is **held** when a game this
port plays frame for frame put a leader in it.

**100 %** would be every age held for at least one frame of a lockstep
game.

**What it does not measure.** Frames, not play: a twin's hundred frames
in one age weigh more than a long capture dumped at its start and end.
`--traces` reads the pinned coverage traces' dumps instead — the
definition before the fifty-first pass, which counted the middle-age
starts (run1041, run1043) whose games no word scores.

## 3. By unit type

**What it measures.** Every `GUY` block's `type` line in the same
lockstep dumps, against the install's `unitrules.xml` (type id 50 + the
record, `docs/FORMATS.md`), with each never-seen type grouped by its
`DOMAIN` and the loaded `age` from the start-of-game types dump
(`gamelog-run3-fulldump-types.txt`; `-1` for the gaia animals, `?` when the
archive lacks it). A type is **seen** when any guy of it stood in a dump
of a lockstep game, start-of-game or per frame — on the map of a game this
port matched frame for frame.

**100 %** would be every skirmish unit type on the map of a lockstep game
at least once.

**What it does not measure.** Presence, not exercise: a type that stood
in a start dump has not fought, been trained or upgraded in a dump. The
twenty-odd nation variants of one unit (`GRAPH` differs, `NAME` repeats)
are separate types and are listed separately.

**Table 3b, what each type did** (Loop 3727; `tools/rubric_states.py`,
an Opus worker's at the sixtieth pass's sequel): in how many lockstep dumps
a unit of the type was **seen** (table 3's rule), **moved** (its
`x_internal`/`y_internal` differ between two unit snapshots with the same
`uid`), **attacked** (a victim's `damage_frame` with `damage_o`/`damage_who`
naming it — `do_damage`'s overkill record — or an `AMMO` record's shooter),
**died** — printed by **no** lockstep dump (`DeathObjData::log_data`'s
`DEATHS` record; the captures run `DEATHS=0`), so reported as not printed
with **vanished** inferred beside it (a unit with a `uid` absent from the
next whole snapshot; transports dissolving on landing are most of it) —
and **boarded** (`inside_up` ≥ 0, a transport or a garrison). On this tree:
**seen 229 · moved 199 · attacked 74 · died not printed (vanished 53) ·
boarded 114** of 302. Blows on buildings are not counted (`BUILDDATA`
prints no `damage_o`, so the siege lines read 0); the cache is
`~/ron-data/rubric-states/cache.json`, 25 s cold, 0.3 s warm. Its totals
line stands beside the `Rubric:` line and is not part of the pinned one.

## The denominator

Of `unitrules.xml`'s 364 records, **62 are out of a skirmish's
reach** and table 3 counts against the other 302. The criterion is the
record's own `TRIBE_MASK`, all zero, unless the loader files it as gaia:

- **The field.** `TRIBE_MASK` is 24 bits, one per nation, MSB-first
  (`docs/FORMATS.md`). `LeaderData::tribe_can_type(t)` is 4 when the
  player's tribe bit is set in `t`'s mask and 0 otherwise, and
  `LeaderData::get_graft(t)` is `t` only when `tribe_can_type(t)`, else the
  tribe's graft for it; the graft table is written only by records into
  the slots of nations in their own mask (`docs/TECH.md`, "Nation
  variants", `UnitType::init`'s passes 2 and 3). A zero mask is a type no
  nation trains and no graft names.
- **The exception.** The gaia animals (402–413: the birds, the herds, the
  fish and whales) have a zero mask too and are on every skirmish map; the
  types dump loads them at `age` −1, the gaia marker table 3 already
  prints, and they stay in the denominator.
- **Not a name.** Unique units carry their nation's bit (the skirmish
  Immortals 162–165 have the Persians'), and two `CTWE`/`CTWW` records
  (252, 253, Main Battle Tank) carry three nations' bits and stay in; the
  rule is the field, never the graph's suffix.

The excluded 62: the `CTWE`/`CTWW` variants of the General, Assault
Infantry, Advanced Machine Gun, Anti-Tank Missile, Armored Cavalry and
Anti-Aircraft Missile (55, 56, 105, 106, 128, 129, 142, 143, 221, 222, 285,
286); the named heroes 358–375 and Darius' War Chariot 376; the scenario
records 377–400 (the Guerilla Fighters, Bandeirantes, Red Coats, SAS, the
barbarian Archers, Javelineers and Phalanx, Greek Mercenaries, Imperial
Guard, the Fur Trapper and the rest); the Immortal Bowmen line 184–186,
the Conquistadors 201–202 (`WHERE disable`), `ABRAMDES` 255 and the
Siegeship 345. `tools/rubric.py --tables 3` prints the list and an
"excluded and seen" line, which reads **none** on the lockstep dumps
(99 of them, 2026-10-22); the pinned coverage traces' (`--traces`), where
staged takes and the chat cheat `add` stand units on the map, were not
read for it on this pass.

**What it has not established.** A zero-mask type can still reach a
skirmish by a route other than training: a script (`ScenarioFuncSet`), a
cheat, a power that spawns a unit, or "No Unique Units" and barbarian
tribes, which `Types::finalize_grafting` hands a unique unit's *graft* —
a base record, never a zero-mask one, on this file. None of those is a
lobby this project plays. The check that would falsify the criterion is
the "excluded and seen" line on a lockstep game.

## 4. Bytes in lockstep

**What it measures.** Of the simulation layer's functions (the census's
`--layers` rule; 3,611 functions, 11,392,782 bytes), how many and how many
bytes a lockstep game's trace entered: `census.entered_in |
census.hooked_in` over the whole games' `rontrace-run<N>.log` files —
exactly the census's `entered` column and its reader, on these logs only.
It is the share of the simulation's code the original executed inside a
game this port matched frame for frame. Printed in tenths of a percent,
floored; the `Rubric:` line carries it.

**100 %** would be every simulation function entered in a lockstep game.

**What it does not measure.** The scored long captures are `cover=0`:
their traces carry the draw stream and the five trampolined hooks
(`FRAME`, the two `get`s, `rand_real`, `reseed`), never a function's
entry, so the number is carried by the `cover=1` twins — a twin of every
whole game is what moves it. Entered is not checked: a function the
original ran inside a lockstep game is one whose *effects* agreed with
ours on the draw stream, never one whose every field a dump compared.

## 5. Entered × age

**What it measures** (Loop 3727, with table 0). Of each stem's **open**
functions, how many no lockstep trace with coverage entered in a game
that held each age — the ages table 2 reads off the dumps' `ages_get()`
per run (`rubric.ages_by_run`), the coverage `census.entered_in` per trace
(`entered_by_trace`: a trace counts only when it entered a function; a
`cover=0` trace holds the five hooks and nothing else and places
nothing). The header row says how many traces with coverage each age has;
`never` is the open functions no trace entered at any age. The **emptiest
cell** — the stem and age with the most never-entered open functions, ties
to the later age — is what the next capture stages (Loop 3729, the melee;
`docs/GOLDEN.md` §155), and the braces' refill reads this matrix rather
than the age buckets alone.

On this tree (the sixtieth pass's sequel, 235 lockstep traces, 21 with
coverage: the long words run53 and run54, the battery twins run1016–1024
and run1062, the pair twins run1176–1179, 1187, 1188, 1190, 1191, 1215 and
1216): traces with coverage by age **Ancient 20 · Classical 1 · Medieval 2
· Gunpowder 0 · Enlightenment 3 · Industrial 2 · Modern 2 · Information
7**; never at any age 616 of the 1305 open (`groups` 76, `map` 43, `world`
41, `unit` 39, `game` 31); **the emptiest cell is `unit` at Gunpowder, 106
— the whole stem, because no coverage trace's game held Gunpowder**, and
after it every stem at Gunpowder, then `groups` at Classical (78). The
`unit` functions never entered at any age are the melee's targets:
`do_board`, `do_await_board`, `do_garrison`, `do_group_attack`,
`do_flee_to`, `do_form_change`, `think_hero`, `think_supply`,
`check_meet_ship`, `air_alt_speed`, `kill_group_order` among the 39.
run1176 carries coverage and no age (its dumps hold no leader record) and
places nothing.

**What it does not measure.** Entered is not compared (table 4's caveat);
an age column rests on the few twins that hold it, so a column with one
trace says what one game reached; and every run holds Ancient because the
start block's leaders read `ages_get()` 0 before the lobby's age applies,
which is why the Ancient column equals `never`.

## The headline

From the fifty-second tranche the queue's headline is table 3's count —
**unit types seen, of the skirmish types** — with the blind list's open
rows kept as a floor (`docs/DECISIONS.md` entry 89, amended with Ramon;
entry 90). **From the sixty-first the `Rubric:` line opens with table 0's
three numbers** — `ported P · replaced by design R · open O of 3611` — and
table 1's mechanics summary stays in the table and the ledger (Loop 3727,
DECISIONS 100); the braces' refill reads table 5's matrix, not the age
buckets alone. **Seen means stood in a lockstep game's dump**: a `coverage`
item that takes a lobby for the units it puts on the map moves the count
only once that lobby's game is a whole word — its own `cover=0` capture to
the game's end, walked to its length, and its `cover=1` twin pinned in
`blind::TRACES` with the comment "`run<N>'s` game on every frame". The
worker runs `tools/rubric.py --tables 3` on its tree after the pin, names
the types new to the ledger in its journal and re-pins
`census::RUBRIC_UNIT_TYPES_SEEN`; the commander writes the queue's
`Rubric:` line at the booking, as it writes `Blind:`; the pass's
`--record` row stays the pass's.

Two things the count does not say, each a decision rather than a
measurement, and each Ramon's: whether a type the chat cheat `add` puts on
the map counts (a staged take parts its twin at the `add`, so under the
lockstep definition it does not); and whether "seen" should become
"trained" (a `Leader::create_units` for the type in a lockstep dump) once
presence is cheap. The Conquer-the-World and scenario-only ceiling is
answered by "The denominator" above.

The human is tribe 4 in every capture (`docs/NATIONS.md`): the runner's
`--ai-tribe` sets the first computer slot only, so a nation's line is seen
when a computer plays it.

## The first runs

On `9bbda88d9`'s tree and the archive of 2026-10-21, 58 dumps (14.9 GB)
read in 89 s under the traces definition:

```
groups 112 · at 100 % 36 (marked *) · functions backed 1696 of 3611 · bytes backed 4317313 of 11392782 (37.9%) · unmarked 452 (the crate defines no function of the name)
leader records with an age: 67431 in 58 dumps · ages never held: 2,3,4,5,6,7
seen 30 of 364 · never 334 · ids outside the unit range seen: 0
Rubric: mechanics 36 of 112 source groups at 100 % (backed 1696 of 3611 fns, 37.9% of bytes) · ages held 2 of 8 (never: 2,3,4,5,6,7) · unit types seen 30 of 364 (334 never)
```

The fifty-first pass's run under the lockstep definition (`--record "the
fifty-first pass, lockstep"`, the ledger's first row of the new columns):

```
groups 112 · at 100 % 36 (marked *) · functions backed 1696 of 3611 · bytes backed 4317313 of 11392782 (37.9%) · unmarked 452 (the crate defines no function of the name)
lockstep entered 2227 of 3611 simulation functions · 8777717 of 11392782 bytes (77.0%) · 99 traces
leader records with an age: 10696 in 99 dumps · ages never held: 6
seen 128 of 302 skirmish types (62 Conquer-the-World/scenario-only excluded by TRIBE_MASK 0, not gaia) · never 174 · ids outside the unit range seen: 0
excluded and seen (would falsify the criterion): none
Rubric: mechanics 36 of 112 source groups at 100 % (backed 1696 of 3611 fns, 37.9% of bytes) · lockstep bytes 77.0 % (2227 of 3611 fns) · ages held 7 of 8 (never: 6; lockstep) · unit types seen 128 of 302 (174 never; 62 excluded; lockstep)

rubric: the fifty-first pass, lockstep against the fifty-first pass
what                                         now        before
source groups at 100 %                        36            36
of groups                                    112           112
functions backed                            1696          1696
of functions                                3611          3611
bytes backed, %                             37.9          37.9
lockstep bytes, %                           77.0             -
lockstep functions entered                  2227             -
of simulation functions                     3611             -
dumps read from                         lockstep             -
ages held, of 8                                7             8
ages never held                                6          none
unit types seen                              128            74
of unit types                                302           364
dumps read                                    99           120
```

Read plainly: every pinned word is whole, so the lockstep games are 49 and the
dumps 99; the cover=0 closing blocks print every unit at the game's end,
which is why 128 types stand in them against the traces' 74. No lockstep
game holds a leader in the Modern age (6); the traces definition held all
eight, from captures whose games no word scores.
