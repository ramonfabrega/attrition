# The golden record — the rules track's capture, chapter by chapter

`docs/DECISIONS.md` entry 41 §1 gave the rules track its own oracle and its
own word. This is the script: what the golden record is, the rules the cheat
channel imposes on it, and then a chapter at a time — the `.cmd` file's lines
with their frames, the records the capture must dump for the chapter to be
worth running, and **the records that would falsify it**.

**How this was established.** The channel's grammar and its reachability rule
are read from `ConsoleWin::run_cmd@007d6a70` and `ConsoleWin::parse_cmd@007d6470`
in the Ghidra export, and modelled in [`crate::golden`] under `docs/INPUT.md`
§11, where a run confirms four of the verbs. The map's own geometry — every
coordinate in every chapter below — is read off run105's `[Start Game]`
`WORLD` block rather than guessed: 3,600 cells, each with its terrain, its
region and its owner. Two constraints came from item 364 measuring them: the
bare `war` form is a no-op, and `age` leaves all four epochs Ancient.

**Confidence.** High for chapter one, which is staged, captured five times,
walked in the harness and pinned. High for the map facts in §4, which are a
dump's own fields. **Chapters two to eight are a design and nothing more** —
no capture has been run for any of them, and each one's own falsifier section
is written precisely so that the first run can say it was wrong. The
stageability of each is held by a test rather than by this prose
(`every_chapter_stages_what_it_says_it_stages`).

**What this does not establish** is in §16, and the shortest version is: the
channel cannot issue an order except one, so every chapter below is a
*staging* of state whose behaviour is the original's own automatic play. The
order family needs native issuers, and §13 says which.

## 1. What the golden record is

One staged game per chapter on **map 14 (Great Lakes) at seed 12345**, in the
Quick Battle lobby the whole capture tree already uses, with the Leader AI
silenced at frame 0 and everything else driven from a `rontrace.cmd` file.
The files are `tools/gamelog/golden/chapter<N>.cmd`; the interpreter is
[`crate::golden`]; the harness comparison is `crate::diff::golden`.

It exists because the AI track cannot be written ahead of. A long capture is
one trajectory: every rule verified on it was verified through the AI's own
orders, one decision at a time, and nobody can score a mechanic the AI has
not happened to exercise. A staged game is the opposite — the mechanic is
chosen, the frame is chosen, and the chapters are independent of one another,
so two rules items can run in two lanes on the same day.

**The chapters are independent launches, not segments of one game.** Entry 41
§1 says "one staged game, AI off, in chapters", and the independence it asks
for in the same sentence is what decides the reading: if chapter four were
frames 2700–3600 of one record, a parting in chapter two at frame 1000 would
block every chapter after it, and the parallel lanes the decision exists to
create would not exist. Each chapter therefore re-uses the *lobby* — the same
map, the same seed, the same 34 option fields — which is what keeps the setup
genuinely shared and `borrow_from_siblings` legitimate (`docs/INPUT.md` §11.7),
and diverges only in what its own script stages.

**The word.** ~~Chapter one's is pinned at 626 of 901~~ ~~Chapter one's is
pinned at 774 of 901 (item 445)~~ Both chapters that are captured agree to
their traces' end, 900: chapter one on item 530, chapter two on item 523.
The handoff's `Golden:` line carries both. What the line should say once a second chapter
pins is **the commander's to rule and not this document's to book**; the
design's recommendation is the AI track's own rule one level across —
the **lowest** chapter's word first, because that is where the next cause
lives, with the others listed behind it. What a chapter's word means is
fixed here either way: the first frame at which the harness's draw stream
parts from that chapter's own trace, with the value diff — `game_random`'s
word at the frame's entry — reported beside it, exactly as
`chapter_one_holds_to_the_golden_word` already does.

## 2. The channel's rules a chapter obeys

These are constraints on what a chapter can be, not notes
(`docs/ORACLE.md`, "The cheat channel" and "The channel's vocabulary";
`docs/INPUT.md` §11).

- **`<sim-frame> <text>`, one per line**, `#` a comment anywhere. Lines run
  in file order at the entry of `Game::do_frame` for their frame, before
  phase 1 — so a staged draw is the frame's *first*.
- **A frame below its predecessor's is clamped up**, not reordered. A chapter
  whose lines are out of order silently becomes a chapter whose lines are all
  on one frame.
- **`!` selects the console-only half** of `run_cmd`'s two disjoint switches:
  56 cases console-only, 45 chat-reachable, and a line on the wrong half
  reaches a case that is not there. `ai`, `quit`, `go`, `break`, `restart`
  and `ffwd`'s console twin are the ones a chapter meets; everything else a
  chapter stages is a bare line.
- **`no_mouse = 1`, so every placement carries an `x,y`.** `parse_cmd`
  refreshes `mouse_coord_x/y` only when `no_mouse` is zero, so an omitted
  coordinate does not fail — it silently uses a stale cursor. Chapter six's
  `bird` is the one line in the whole set that cannot carry one, and §10 says
  what that costs.
- **A bare coordinate is a TILE**, `n × 192 + 96`, not a world cell
  (`docs/INPUT.md` §11.3). Four tiles to a cell. This is the single most
  expensive thing to get wrong, and it has already been got wrong once: see
  §5.
- **A bare number is not always a player** (§11.2). `age 3` is player 0's
  third age because `age`'s first token is parsed with a negative default;
  `ally 1` is player 1 because the diplomacy verbs' default is the console's
  own player. When in doubt write `who=`.
- **`restart` wedges the game**, so a chapter is a launch. One seed, one
  script, one process.
- **No console command issues an order — with exactly one exception.** `bird`
  is it, and §10 is the chapter that uses it.

## 3. The chapter form

Every chapter names four things, and the fourth is the one that makes it a
chapter rather than a scene:

1. **Its premise** — the mechanic it puts under the light, in a sentence.
2. **Its cheat lines**, with their frames, in `tools/gamelog/golden/`.
3. **The records the capture must dump** — the `gamelog.ini` keys under
   `[End Frame]`, at the detail levels the record needs, and the window.
   **The `[Start Game]` set is not one of a chapter's choices**: every
   capture the harness walks needs run105's
   `start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1`,
   because that block is what the simulation is stood up from. Without it
   the dump still parses, `borrow_from_siblings` still lends it a map, the
   walk still runs — and the word it reports is the setup's rather than the
   simulation's. run112's first take was that, and `walk_chapter` refuses
   such a capture by name now.
4. **What would falsify it.** A chapter that cannot fail is not a chapter. A
   staged scene always produces *a* dump; the question is whether the dump
   can disagree with the reading that staged it.

**A chapter is a script; a run is a window on it.** `docs/DECISIONS.md` 41 §2
— digest first, detail on demand — is what makes that affordable: the first
run of a chapter is the whole trace at a cheap detail, and each question after
that is a re-run windowed to the frames that answer it. Two launches of one
script are the same game frame for frame (run101 against run103 and run105,
at three times the detail), so a window can be moved and re-taken without
asking whether it still describes the same run.

**Each chapter changes one lever.** Chapter two moves the age with `library`
and chapter three with `age`, so a parting in one is attributable; chapter
four stages its three border levers on three different frames for the same
reason. A chapter that changes two things at one frame has bought one
observation and spent two.

**The ages across the set**, which is entry 41 §1's "an age jump between
chapters": Ancient in chapters four and five, Classical in three, Medieval in
seven and eight, Gunpowder in two, Modern in six, Information in one. The
chapters between them carry the fields that only a late age writes, and
`library` rather than `age` wherever the epoch levels matter — because `age`
alone leaves all four epochs Ancient, which run101–run105 measured.

**What a chapter costs.** A digest of 900 frames is about 24 s of wall clock
and ~35 MB; a detailed window of ~300 frames is 145–172 s and 45–90 MB
(`docs/RUNS.md` run101–run105). `WORLD=6` is the expensive key — 3,600 cells
a frame — and chapter four is the only one that needs it, windowed to fifty
frames.

## 4. The map, read off the disk

Every coordinate in every chapter below comes from run105's own `[Start Game]`
`WORLD` block — 3,600 cells, printed in row-major order with **x fastest**,
each carrying its terrain, its region, its owner and whether it is blocked.
The ordering is not assumed: player 0's owned cells centre on (5.7, 39.3) and
player 1's on (53.4, 20.7), which are their two capitals to within a cell.

A world cell is **768 internal units** and **four tiles** on a side; the map
is 60 cells, so 240 tiles, so 46,080 units.

| what | cell | tile | internal |
|---|---|---|---|
| Napata, player 0's capital | (4, 40) | (16, 160) | (3168, 30816) |
| London, player 1's capital | (55, 21) | (220, 84) | (42336, 16224) |
| player 0's territory at frame 0 | x 0–14, y 29–50, 266 cells | | |
| player 1's territory at frame 0 | x 45–59, y 10–31, 261 cells | | |
| the neutral arena (chapters 1, 2, 3, 6, 8) | (1, 10) and neighbours | 4–15, 40–43 | |
| free ground beside Napata | (6, 40), (7, 40), (8, 42), (6, 44) | 24–35, 160–179 | |

**Great Lakes has four of them**, and they are sea regions in the dump's own
numbering: 66 at cells x 23–31, y 11–17; 67 at 43–46, 13–23, the one beside
London; 69 at 38–48, 37–49, the largest at 69 cells; and **70 at 13–20,
40–49**, the one beside Napata, whose deepest cells — water on every side
within two — are (15, 45), (15, 46) and (16, 46), tiles x 60–67, y 180–187.
164 of the 3,600 cells are ocean. That is what makes chapter five possible at
all, and it cost a grep rather than a capture.

**Sea is never owned**: every cell of every sea region carries owner and
runner-up 255 after the land pass (`docs/ATTRITION.md`, "The shape of
it"), so a ship on a lake is on nobody's ground and chapter five cannot
double as an attrition chapter.

## 5. Chapter one — two armies, a late age, and no Leader AI

**Landed** (item 364). `tools/gamelog/golden/chapter1.cmd`, six staged lines,
901 frames, captured five times as run101–run105 and walked in
`crate::diff::golden`. **The word is 774 of 901** (item 445; it stood at
626 from item 405). Its widening is
`chapter_one_s_word_frame_is_widened_whole`, over `[605, 779)`, with the
figure's clock borrowed from run110 (`GUYS=9`, `[610, 630)`), because
run105 is a `GUYS=2` capture and prints no clock (`docs/COMBAT.md` §48.1).

```
0    !ai off
600  age who=0 8
602  age who=1 8
604  war
610  add hoplite who=0 4,40
615  add hoplite who=1 5,40
```

Records: `end:UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=2,DEATHS=1` and
`misc:COMMANDMANAGER=1`, digest window `[880, 900)` and detail `[605, 900)`.

What it established, and what a later chapter must not re-derive: `ai off` is
the whole of the difference from frame 1 (run104 is the control with the line
deleted, and its frame 1 is this crate's own draw for draw); auto-engage
survives AI-off, so three squads born at 615 carry an `ATTACKORDER` by 616
with no order ever issued; `add hoplite` is three units, not one; and a squad
is seated by `find_nearby_spot` around its captain, all six coordinates pinned.

**Two corrections this chapter carries, both worth stating because the file
now says so in its own header.** The `604 war` line is the **bare** form and
does nothing; the squads engage because a Quick Battle already starts at war.
And the squads are **not near player 0's capital** — the first draft of the
file's comment said they were, on the world-arm reading of `parse_coord`.
`4,40` is a tile pair, so it is world cell (1, 10): unowned `BASELAND`,
region 1, thirty cells from Napata. Nothing in the chapter depends on it —
the arena is clear ground either way, which is why the error survived five
captures — but every chapter that *does* depend on where it stands takes its
coordinates from §4 instead.

## 6. Chapter two — the ranged line, and the ammunition

**Premise.** Chapter one's squads are born in contact and never shoot. Put a
ranged squad eight tiles from a melee squad and the shooting half of
`Unit::fight@005fd4d0` runs first: target acquisition at range, a projectile,
`Objects::add_ammo`, a reload, and the closing of the gap while all of it
happens.

**Lines** (`chapter2.cmd`): `!ai off` at 0; `library who=0 3` and
`library who=1 3` at 600 and 602; `add bowmen who=0 4,40` at 610;
`add hoplite who=1 12,40` at 615; `add slinger who=0 4,43` at 620. Bowmen
reach ten tiles and Slingers six, each a squad of three; Hoplites reach zero.

**The capture must dump** `end:UNITS=3,GUYS=2,AMMO=5,DEATHS=1,LEADERS=2` and
`misc:COMMANDMANAGER=1` over `[605, 900)`. **`AMMO` is the key that makes
this chapter and it is off in every capture the tree has** except run109;
without it the projectile is invisible and the chapter is chapter one with
more units.

**What would falsify it.** No `AMMO` block on any frame after 610 — the
ranged arm was never entered and the bowmen closed to contact like hoplites.
Or `AMMO` blocks from the Slingers at a range above six, or from the Bowmen
above ten, which would say the `RANGE` field is not the reach the fight
tests. Or an `ATTACKORDER` on the Bowmen at frame 611, before the hoplites
exist, which would say auto-engage is not choosing its target the way
chapter one's melee squads do.

**Run, 2026-09-19, run112 (item 415). None of the three fired, and the
word is 616.** `docs/RUNS.md` run112 has the whole receipt; the four
predictions written into the file's header before the run all held — eight
`INFO cmd` records each returning 1, 296 window blocks over 605..899 with
no gap, nine units born three at a time at 611, 616 and 621, and **373
`AMMO` blocks over 186 frames** beginning at 645. The first arrows are the
Bowmen `0/6` and `0/7` on `1/8` at 4.95 and 4.04 tiles, inside reach; no
`ATTACKORDER` on the Bowmen at 611.

~~The draw stream parts at **616**, ours 26 draws against the original's
25, the extra one `Unit::fight+0x9b0` — the one-in-five re-search — on the
frame the hoplite squad appears.~~ **624 since item 447, 2026-09-21**
(`docs/COMBAT.md` §31): the spurious 616 order was a target the original
could not see, and `UnitData::is_seen` is the fifth test of
`ObjectData::valid_target_const` that this crate had no term for. The
stream now parts at **624**, ours 25 draws against the original's 26 —
the missing one a `Unit::move_step+0x823` on the slinger squad — and the
first value disagreement is 625. `GOLDEN_WORD_CHAPTER_TWO` pins it, and
`chapter_two_s_word_frame_is_widened_whole` is its widening: the values
part two frames earlier, at **622**, on where the slingers plan their
chase.

**900 since item 523, 2026-09-22, the end of the trace.** The word
walked 624 → 762 over items 447 to 495 (`docs/COMBAT.md` §31–§46). Item 523
closed the rest: the captain a dying head hands its squad to, the bearing a
shot strikes at, and the lead it is aimed with (§47). No draw parts on any
frame of chapter two and no word does. The widening covers the whole
capture, and what it still carries are two value residues that spend no
draw (`damage 1/6` at 680 and ten `g.gpiece` rows at 606), plus `near_o` on
two bowmen from 847.

~~**And the value diff found what the draw stream did not.** … 140 units
west … `find_nearby_spot`'s ring on clear ground …~~ **Withdrawn
2026-09-19, item 441** (`docs/COMBAT.md` §29.2). Widening every record
over 612–640 shows the seating **exact** at 616 and 617 and the positions
parting only from 618, after which this crate's squad walks west at 28
units a frame while the original's never moves: `622 − 617` is five frames
and `5 × 28` is 140. The number was a real measurement read at the wrong
frame, and the ring was never involved.

**What the widening puts in its place is narrower.** who=0's nine units
never diverge at all, in any field, on any frame; nothing else in the dump
does either. The entire divergence is who=1's three hoplites, beginning
exactly at 616, where this crate holds one order and the original holds
none — then two at 617, a path at 618, and the march. One order, and
everything after it downstream.

**What stands at 616, after item 426 widened it.** The original issues
**no** attack order on that frame at all: its first is at **621**, to the
slingers on their own birth frame targeting `ox 8 whom 1`, and the bowmen
and the hoplites both take theirs at **635**. This crate matches two of
those three squads exactly and ~~gives the hoplite squad an order on its
birth frame, 616~~ — which was the whole of the word, because the spurious
order put the captain into `do_attack` and `Unit::fight`'s one-in-five
re-search spent the twenty-sixth draw. **Item 447 retired that order**
and did not replace it with the dump's 635: the original's `1/6` accepts
its target on `ObjectData::visible`, which this crate does not model, so
the hoplite squad now takes no order at all in the chapter
(`docs/COMBAT.md` §31.5, §31.7).

**Two named mechanisms were ruled out by measurement**, and that is the
item's product as much as the frame is. The 140-unit seating above is
**not** the cause: seated on the dump's own cells the squad still engages
at 616. `find_melee_target`'s `0x40000` arm is **not** the cause either —
who=1's hoplites do carry `unit_masks 262144` where who=0's carry 0, so it
fires, but disabling it changes nothing, because the plain
`unit_respond_range` floor already reaches twelve tiles and the bowmen are
7.25 away. That arithmetic exonerates the radius as such: the original
floors a melee searcher the same way, so had its captain searched it would
have found them too. ~~**It did not search.**~~ **Corrected 2026-09-21,
item 443** (`docs/COMBAT.md` §30): it did search, and refused what it
found. `near_o` is written above the range test, so it is the search's
footprint and not the order's; and the bowmen at 616 sit in the *same*
object-grid cell as the slinger the same captain accepts at 635, from a
seat it never leaves — so no radius can separate the two frames, and what
is left is a target-acceptance predicate. And the slingers are the
control that makes this a measurement rather than a story — they engage on
their birth frame at **8.6** tiles, further than the 7.25 the hoplites do
not engage at, so no distance threshold can separate the two.

~~The remaining suspect is `think`'s own auto-attack gate firing on a
cheat-spawned captain's first frame~~ — **ruled out 2026-09-21, item 443**
(`docs/COMBAT.md` §30.5): the slinger captain's own birth frame is on
neither of `think`'s grids, so its birth-frame search can only have come
through the `idle == 1` arm, and the hoplite captain reaches
`think_attack` at 616 by the same arm. The suspects that survive are
`Object::valid_target@00648ba0` and `Object::poor_target@0064a270`'s
type-record arm, and both are written down as hypotheses
(`docs/DECISIONS.md` 42).

**Two things the chapter cost that were not in the design.** The first take
omitted `--detail start:` and reported a golden word of **0** that was the
setup's number rather than the simulation's; `walk_chapter` now refuses a
capture whose start block carries no leaders and no units, and §3 below says
the set is not optional. And a relative `--cmd-file` could not work at all,
because `golden_capture.sh` cds into `tools/explore` before exec'ing the
runner — fixed in the script rather than in the recipe, so the recipe stays
the one a reader would type.

## 7. Chapter three — the mounted and siege lines

**Premise.** Two things chapter two cannot separate: a per-line movement
speed, and a shot a unit *refuses* to take. A Catapult's range is **3 to 15
tiles** — it has a minimum — so a chapter in which the enemy closes past
three tiles is the only kind that can show the refusal.

**Lines** (`chapter3.cmd`): `!ai off` at 0; `age who=0 4` and `age who=1 4`
at 600 and 602 — `age` rather than `library`, so that chapter two owns the
`library` arm and a parting is attributable to one of them;
`add 3 chariot who=0 4,40` at 610, the leading count placing three separate
one-figure units rather than a squad; `add hoplite who=1 12,40` at 615;
`add catapult who=0 4,41` at 620, eight tiles from the hoplites.

**The capture must dump** the same set as chapter two, over `[605, 900)`.

**What would falsify it.** The catapult firing at a target inside three
tiles, which would say the minimum is not enforced in the fight. Three
chariots whose per-frame displacement equals the hoplites', which would say
the line's speed is not read. A leading count of 3 producing nine units or
one, either of which would overturn `docs/INPUT.md` §11.5's reading that the
count is a count for a unit type and not for a building.

## 8. Chapter four — the Temple, the border, and the bleed

**Premise.** The namesake, end to end, on ground the dump can already name.
Three border levers on three separate frames, then a hostile squad standing
inside player 0's own territory with an attrition strength granted, a scout
beside it as the exemption control, and a Supply Wagon arriving five hundred
frames later to cancel the tick.

**Lines** (`chapter4.cmd`): `!ai off` at 0; `add temple who=0 28,160` at 300
— cell (7, 40), free ground beside Napata, and the **building arm of `add`,
which no capture has ever exercised**; `tech who=0 religion on` at 400, the
first temple border level, which the install's own `rules.xml` hangs
"Temples increase city effect on National Borders" on; `civic who=0 3` at
500, the independent civic term; `tech who=0 allegiance on` at 550, without
which the period is the sentinel 0 and nothing bleeds at all;
`add hoplite who=1 24,176` at 600 and `add scout who=1 27,176` at 605, both
at cell (6, 44), well inside player 0's border; `add supply who=1 25,178` at
1100.

**The capture must dump, in two windows.** The border wants
`end:WORLD=6,BUILDS=7,CITIES=5,MISC=1` over `[295, 345)` — `WORLD` is 3,600
cells a frame and cannot be windowed wide, and `GameDaemon::check_borders@00732060`
spends a shared budget of 256 cells a frame, so a recompute takes on the
order of fifteen frames to settle and the window must outlast it. The bleed
wants `end:UNITS=3,GUYS=2,DEATHS=1,LEADERS=2` over `[595, 1500)`, where
`attrition` and `damage` are per-unit fields and the wagon's shelter shows as
`unit_masks2` carrying the supply flag on the tick frames.

**What would falsify it**, five ways, which is why this is the chapter worth
running first after chapter two:

- The count of cells with owner 0 in the `WORLD` block **unchanged** across
  frame 300 — the Temple placed and the border indifferent. (Counting cells
  rather than measuring the reach is deliberate: it does not depend on
  knowing `TEMPLE_UPGRADE_TERR`'s value.)
- The count unchanged across 400 or 500 — the tech, or the civic level, not
  a border term after all.
- `attrition 0` on the hoplite squad after 550, which would overturn
  `docs/ATTRITION.md`'s "one step is 48".
- A **nonzero** `attrition` on the scout, which would overturn the
  `is_special` exemption run16 observed.
- `damage` still climbing on the squad after 1100, which would overturn
  supply as the counter — or, the other way, `damage` frozen *before* 1100,
  which would say something other than the wagon was sheltering it.

**What it will cost if the Temple lands as a construction site rather than a
finished building.** `Objects::init_build` is what the building arm calls and
whether it finishes the building is unread; if the dump shows an unstarted
site, the lever is `finish`, which is in the chat half of the vocabulary and
**not** in the interpreter's set. That is a takes-chain the first run will
price, not a reason to delay it.

## 9. Chapter five — the water

**Premise.** No capture on disk has ever carried a ship. run16's coverage
note names `ObjectData::in_a_ship` and `num_aircraft_here` as never entered
"because no ship, aircraft, or hero-general was in the game", and every
capture since has been the same two land games. Great Lakes has four lakes
(§4) and the channel can put a hull on one.

**Lines** (`chapter5.cmd`): `!ai off` at 0 and nothing else but three
spawns, because the Trireme and the Fishermen need no tech at all and a
chapter that changes no age lever cannot part on an age field.
`add trireme who=0 60,180` at 610 and `add trireme who=1 64,186` at 615 —
tiles inside sea region 70's deepest cells, about five tiles apart, inside
the Trireme's nine-tile reach — and `add fisher who=0 61,184` at 620, a hull
that fights nothing.

**The capture must dump** `end:UNITS=3,GUYS=2,AMMO=5,DEATHS=1,LEADERS=2` and
`misc:COMMANDMANAGER=1` over `[605, 900)`.

**What would falsify it.** No `UNITDATA` record for the triremes at all — the
`add` refused, which is the likeliest outcome and a real result: it would
say `UnitType::find_nearby_spot`'s filter is what decides a hull's ground and
the channel cannot satisfy it without a Dock. A trireme record standing on a
land cell — the search does not filter by domain, and every naval chapter
after this one can place freely. Or two hulls in the water and no `AMMO`
block, which would say a ship's fight is not the land fight's shooting arm.

~~**This is the chapter most likely to fail, and it is the cheapest to run**: a
digest is 24 s. It should be run before any effort is spent on a Dock.~~
**Run as run127 (item 535), and none of the three fired** (`docs/RUNS.md`
run127). The `add` placed all three hulls with no Dock. Each stands on an
`OCEAN` cell of region 70, on its asked tile's centre plus 24 on both axes.
The two triremes fight: 506 `AMMO` blocks, 249 and 257 a side. So a naval
chapter after this one can place hulls through the channel. What the
channel cannot tell a later chapter is how the spot search treats *dry*
ground asked for a hull, because every asked point here was already water.
The lines' "about five tiles apart" is 7.2 (dx 4, dy 6), still inside the
nine-tile reach.

~~**The word is 621**~~ (`GOLDEN_WORD_CHAPTER_FIVE`). The first walk parted
at 617, on who=1's trireme turning **broadside** to its target, and item
535 landed the rule (`docs/COMBAT.md` §49). On 621 the original launches
that trireme's first round, whose landing scatter is two draws (8 against
6), a frame before this crate does. The widening is
`chapter_five_s_word_frame_is_widened_whole`.

**The word is 664** since item 542 (`docs/COMBAT.md` §50). The round's
frame was the release event's divisor, `starttime / 67` and not `× 3 /
200`. Its point was node 0 on the keel, and the widening, now comparing
the whole `AMMO` record, found its height: a sea figure stands at `z` 0.
The first two volleys of both ships now agree on every field. ~~On 664 the
original spends a `Guy::set_anim` for the fisher `0/7`, whose birth cast
ends there. This crate never gave it the cast.~~

**The word is 739** since item 543 (`docs/ORDERS.md` §23). The fisher's
cast was its deploy, and a *human's* boat is given it by `Unit::think`'s
rare-collector arm, which sits above `ai off`'s exit. The birth orders,
the walk, the forty-frame cast and the deploy on 665 now agree, and
nothing under the word parts. On 739 this crate ends an attack the
original does not: `1/6`'s `CHAR_ATTACK3` runs three frames.

## 10. Chapter six — the air, and the one command that issues an order

**Premise.** Three of the six order classes no document cites are the air
ones. And there is one door into the order family the channel already has.

**`bird` is the only console command in the executable that issues an
order.** `ConsoleWin::run_cmd@007d6a70`'s case 82 — the table index of
`bird`, whose help text is "Drop a Wild Bird at Mouse" — calls
`Objects::init_unit@0065e0c0` for gaia type 9 and then
`Unit::add_air_patrol_order@005e4350` on it. It is the *only* `add_*_order`
call in the whole of `run_cmd`; the other two order-adjacent cases, `pack`
and `deploy`, call `Unit::clear_orders` and then `SpellType::cast_pack@00670be0`,
which is a state poke and not an order, and `anim` reaches
`Guy::set_anim@005da300` directly.

**That was a correction to `docs/ORACLE.md`**, whose "The channel's
vocabulary" section said "No console command issues an order at all", to the
same document's "unreachable from the channel by construction", and to
`docs/INPUT.md` §11, which restated it. All three are amended in place as of
item 365, and chapter one's own header with them.

**Lines** (`chapter6.cmd`): `!ai off` at 0; `library who=0 6` and
`library who=1 6` at 600 and 602, the Modern age, where the Fighter and the
Bomber are and whose stats are epoch-fed, so `library` and not `age`;
`add fighter who=0 4,40` at 610 and `add bomber who=1 12,40` at 615, eight
tiles apart, outside both reaches, so what closes the gap is the air line's
own movement; and `bird` alone at 700.

**`bird` is last, and on its own frame, for a reason.** Its case reads
`mouse_coord_x/y` with no coordinate argument and **no** `WorldData::restrict`
— which `nuke`'s neighbouring case does have — and with `no_mouse = 1` those
are whatever `ConsoleWin`'s constructor left. The expectation is the world's
corner; if it is something else, every other record in the chapter is
already on disk before the line runs.

**The capture must dump** `end:UNITS=3,GUYS=2,AMMO=5,DEATHS=1,LEADERS=2` and
`misc:COMMANDMANAGER=1` over `[605, 900)`.

**What would falsify it.** No `AIRPATROLORDER` block after frame 700 — the
reading of case 82 is wrong, or the bird did not survive its placement. An
aircraft record whose position never changes, which would say an air unit
staged outside an airbase is inert. Or the `add` refusing the Fighter
outright, which would say the air line needs a base the way chapter five may
show a hull needs a Dock.

## 11. Chapter seven — the civilians, and what AI-off takes away

**Premise, which is a prediction and not a setup.** `Unit::think@005f6e40`'s
tail is gated by `if ((leader_flags & 4) != 0 || ai_off != 0)`, and for a
human leader with the cheat on, the block's one unconditional statement
returns for any unit without the AI-driven mask (`docs/INPUT.md` §11.4). So
five civilians staged on their owner's ground with `!ai off` should do
**nothing** — no gather, no trade, no carry. Every other chapter in this set
relies on that gate holding; this is the one that tests it.

**Lines** (`chapter7.cmd`): `!ai off` at 0; `library who=0 2` at 600;
`add citizen`, `add caravan`, `add merchant`, `add scholar` and `add fur` for
player 0 at 610–630, on free owned ground beside Napata (§4).

**It is a pair.** The same file with the `0 !ai off` line deleted is the
control, exactly as run104 is chapter one's, and there the same five should
act. A chapter that only shows nothing happening has not measured anything.

**The capture must dump**
`end:UNITS=3,GUYS=2,BUILDS=7,CITIES=5,GOODS=3,LEADERS=2` and
`misc:COMMANDMANAGER=1` over `[605, 1200)`. `GOODS` is what says whether a
gather ever delivered.

**What would falsify it.** A `GATHERORDER` or a `TRADEORDER` on any of the
five in the AI-off run — the gate does not do what §11.4 reads it as doing,
and every chapter in this set is standing on a wrong premise. Or **no**
order on any of the five in the *control*, which would say the units are
inert for some reason that has nothing to do with the gate and the
comparison is vacuous. The pair is what separates those.

## 12. Chapter eight — the commanders, and a war that is declared

**Premise.** Chapter one's `war` is the bare form and changes nothing. This
is the chapter that moves the diplomacy state, three times, with a fight
running across each change: peace at 700, war at 800, alliance at 900. And
it carries the Command line, which nothing has captured.

**Lines** (`chapter8.cmd`): `!ai off` at 0; `library who=0 2` and
`library who=1 2` at 600 and 602, the Medieval age, where the General and
the Spy's prerequisite sits; `add hoplite who=0 4,40` and
`add general who=0 5,40` at 610 and 612; `add hoplite who=1 12,40` and
`add spy who=1 12,41` at 615 and 617; then `peace 1`, `war 1`, `ally 1`. The
targets are bare numbers and they are **players**, because the diplomacy
verbs' `parse_who` default is the console's own player and not −1.

**The capture must dump**
`end:UNITS=3,GUYS=2,AMMO=5,DEATHS=1,LEADERS=5` and `misc:COMMANDMANAGER=1`
over `[605, 1200)`. **`LEADERS=5` rather than 2**: the diplomacy row is what
this chapter reads, and `LEADERS=2` stops above it.

**What would falsify it.** `diplos[1]` unchanged in the frame after 700, 800
or 900 — the targeted form does no more than the bare one, and the whole
reading of the three diplomacy cases in `docs/INPUT.md` §11.6 is wrong. Two squads
still trading damage at frame 750, after the peace — the fight does not read
the diplomacy state on the frame it changes, which would be a finding about
`Unit::fight`'s target validity rather than about the channel. A General
whose presence moves no field on the squad beside it, which would say the
aura is not modelled where this document assumes it is.

## 13. The order family, and what each chapter reaches

`docs/CENSUS.md`'s order family is 31 classes, 410 functions, 44 cited and
205 entered, with six classes carrying no citation at all. The honest
statement about the chapters above is that **they reach the order family
almost entirely by accident**: the channel stages state, and every order in
a golden capture is one the original's own automatic play issued. The table
is what says which is which.

`auto` — a chapter's own units reach it with no issuer, through
`Unit::think@005f6e40`'s auto-engage or the animals' and starting citizens'
own behaviour. `channel` — `bird`, the one exception (§10). `issuer` — needs
a native issuer called from the tracer DLL, and names the one
`obsoletescriptfuncs.txt` shows the shape of. `unresolved` — no command type
in `docs/COMMANDS.md` §3 maps to it cleanly and the reading is owed.

| class | how | chapter |
|---|---|---|
| AttackOrder, AttackToOrder | auto, and `CommandManager::issue_attack@009415e0` | 1, 2, 3, 5, 6, 8 |
| TargetOrder | auto (the search step) | 1 and every combat chapter |
| GroupAttackOrder, GroupAttackToOrder | `issue_attack` on a multi-unit group | — |
| MoveOrder, GroupMoveOrder | `CommandManager::issue_move_to@00941720` | — (the lab has validated this one) |
| ExploreToOrder, FleeToOrder | the same entry point, trailing selector | — |
| PatrolOrder, GroupPatrolOrder | `CommandManager::issue_patrol@00941800` | — |
| AirPatrolOrder | **channel** (`bird`), and `CommandManager::issue_launch_patrol@00941860` | 6 |
| AirOrder | `CommandManager::issue_flight@00941d40` | — |
| AirAttackGroundOrder, AttackGroundOrder | `CommandManager::issue_attack_ground@009417a0` | — |
| GuardOrder | `CommandManager::issue_guard@00941ed0` | — |
| FollowOrder | `CommandManager::issue_follow@00941e70` | — |
| FormOrder | `CommandManager::issue_form@00941580` | — |
| GarrisonOrder | `CommandManager::issue_garrison@00941a70` | — |
| GatherOrder | auto (starting citizens), `CommandManager::issue_gather@00941a20` | 7 (the control) |
| BuildOrder | `CommandManager::issue_build@00941c30` | — |
| TradeOrder | auto (a Caravan under AI), `CommandManager::issue_trade@00941960` | 7 (the control) |
| CastOrder | `CommandManager::issue_spell@00941b80` | 8 stages the Spy; the cast needs the issuer |
| RepairOrder | the `repair` command type has no `CommandManager` issuer in the export | unresolved |
| BoardOrder, AwaitBoardOrder | `CommandManager::issue_set_transport@00941910`; `board_ship` has no issuer | unresolved |
| StrafeOrder | no command type of its own; a mounted or air attack on the move | unresolved |
| SpecialAnimOrder | **not** `anim`, which pokes `Guy::set_anim@005da300` | unresolved |
| UnitOrder, GroupOrder, ThinkOrder | base classes, entered by everything | all |

**The conclusion this table is for.** Seven of the eight chapters below
chapter one add no order class the tree does not already enter; chapter six
adds one. Order coverage is a **separate axis** from the chapters, and its
unit of work is a native issuer validated the way the lab validated
`issue_move_to` — under the emulator first, then in a live pair. Entry 41 §5
says so and this is the list it was waiting for. The chapters are still
worth running: what they buy is the *unit lines* and the *ages*, which no
issuer buys, and each of them gives the eventual issuer a staged cast to
order about.

## 14. The running order, and the run numbers

Reserved for this design: **run112–run119**.

| run | chapter | window | why this order |
|---|---|---|---|
| 112 | two, the ranged line | `[605, 900)` | **run 2026-09-19, word 616**; the cheapest chapter that adds a record the tree has never dumped (`AMMO`) |
| ~~113~~ 127 | five, the water | `[605, 900)` | the likeliest to fail, 24 s to find out, and it gates any work on a Dock — **run 2026-09-22 as run127 (item 535), word 621; no falsifier fired** |
| 114 | four, the border | `[295, 345)` | the namesake; `WORLD=6` narrow |
| 115 | four, the bleed | `[595, 1500)` | the same script, a second window |
| 116 | seven, the civilians | `[605, 1200)` | tests the premise every other chapter stands on |
| 117 | seven, the control | `[605, 1200)` | `!ai off` deleted; without it 116 measures nothing |
| 118 | three, the mounted and siege lines | `[605, 900)` | |
| 119 | six, the air and the bird | `[605, 900)` | the one new order class |

Chapter eight and any further detail window need numbers beyond the
reservation. The order above is by **what a failure would teach**, not by
chapter number: 113 and 116 are placed early because each can invalidate work
that would otherwise be done on top of it.

## 15. Coverage — what a diff backs, and what rests on a reading

**Diff-backed**, by chapter one's own capture and the harness walk
(`crate::diff::golden`): the script format and frame clamping; `ai off`'s
three readers and its cost from frame 1; `add`'s unit arm, its count, its
three figures and its seating; the tile arm of `parse_coord`; the bare
diplomacy form's inertness; and the claim that two launches of one script
are one game at any detail.

**Held by a test rather than a capture**: that every file in
`tools/gamelog/golden/` parses, that every line is on the right half of
`run_cmd`'s two switches, and that the verbs the interpreter will not act on
are exactly the ones `CHAPTER_DEBT` names — currently chapter one's bare
`war` and chapter six's `bird`. It fails when a new chapter reaches for a
verb the harness drops, which is the failure mode this design is most likely
to produce.

**Reading alone, and a capture would settle each**: the entire content of §6
through §12 — every premise, every record list and every falsifier. The
addresses in §10 and §13 are read from the export and checked against its
index by `docs_guard`, which is not the same as a run entering them. The map
facts in §4 are a dump's own fields and are the strongest thing here short
of chapter one itself.

## 16. What is not established

- **No chapter but the first has been run.** Everything in §6 through §12 is
  a design. The falsifiers exist so that the first run of each can say the
  design was wrong, and on the evidence of the last two months the honest
  expectation is that two or three of them will.
- **Whether `library <n>` moves the age as well as the epochs.** `age who=0 8`
  was measured to set the age and leave all four epochs Ancient; the inverse
  — that `library` carries both — is `docs/RUNS.md`'s note and not a
  measurement, and chapters two, six, seven and eight rest on it. The first
  of them to run settles it, and the check is one `LEADERDATA` line:
  `ages_get()` beside `epochs_get()`.
- **Whether a cheat-placed building is finished or a site.** §8.
- **Whether `find_nearby_spot` filters by domain.** §9 — the whole of
  chapter five turns on it, and nothing on disk answers it.
- **Where `bird` lands.** §10. `ConsoleWin`'s initial `mouse_coord_x/y` was
  not read; the case applies no `WorldData::restrict`.
- **How the golden word composes across chapters.** §1 states the design's
  recommendation and says it is the commander's ruling. Until a second
  chapter pins, the handoff's `Golden:` line is chapter one's word and the
  question does not arise.
- **`meet` and `unmeet`** are in the chat half of the vocabulary and would
  give the CONTACT bit a staged frame rather than a bracket — which is what
  a capture-lane item has been paying for. They are not in the interpreter's
  cheat set and no chapter uses them; that is a takes-chain worth pricing,
  not an omission this document is defending.
- **Seven verbs remain parsed and not applied** (`docs/INPUT.md` §11.8):
  `die`, `damage`, `craft`, `move`, `resource`, `finish`, `hurry`. Between
  them and `select`, which is what the first four of them act on, there is a
  second family of chapters — a unit brought to an exact hit total, a
  building finished on a chosen frame, a resource given — that this design
  does not reach and that costs interpreter work rather than a capture.
