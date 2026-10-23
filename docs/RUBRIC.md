# The rubric

`tools/rubric.py` is a steering pass's holistic scorecard of how close
this port is to a 1:1 simulation of the original: four tables, each from
data the tree or the machine's archive already holds, and one `Rubric:`
line at the end. It is a measurement tool — it changes no score, pins
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

## The headline

From the fifty-second tranche the queue's headline is table 3's count —
**unit types seen, of the skirmish types** — with the blind list's open
rows kept as a floor (`docs/DECISIONS.md` entry 89, amended with Ramon;
entry 90). **Seen means stood in a lockstep game's dump**: a `coverage`
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
