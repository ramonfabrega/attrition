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
dump's own fields. ~~**Chapters two to eight are a design and nothing more**~~
**Chapters two, four and five are captured and pinned** (run112, run132/133
and run127; items 415, 552 and 535), and none of their falsifiers fired;
**chapter seven is captured (run141/run142, item 578), closed at 1200, and
its premise did not survive** (§11); ~~**three, six and eight are a design
and nothing more**~~ **chapter three is captured (run145, closed at 900;
its restage run146, open) and seven-b too (run156/run157, item 628, its
premise killed by construction and pinned open as measured, §11); ~~six and
eight are a design and nothing more~~ **six is captured too (run168, item
648, its second falsifier fired, pinned open at ~~616~~ ~~700~~ and closed at 900 by item 652, §10), and its restage six-b (run175, item 651, its first falsifier fired on the target arm, pinned open at ~~632~~ and closed at 1250 by item 680); eight is a design
and nothing more** — no capture has been run for it, and each one's own falsifier section is written
precisely so that the first run can say it was wrong. The
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
pinned at 774 of 901 (item 445)~~ ~~Both chapters that are captured agree to
their traces' end, 900: chapter one on item 530, chapter two on item 523.~~
Chapters one, two and five agree to their traces' end, 900 (items 530, 523,
549); ~~chapter four is pinned at 1277, then 1416, of 1501 (items 552, 567)~~
chapter four agrees to its trace's end, 1500 (items 552, 567, 569); chapter
seven agrees to its end, 1200, on its first walk and on its control's (item
578); ~~chapter three is pinned at 621, then 633, of 901 (items 587, 590)~~
chapter three agrees to its end, 900 (items 587–603, 602), and its
restage (run146) ~~is pinned at 780 782 of 1001 (items 602, 616)~~
agrees to its end, 1000 (item 627); chapter eight ~~is pinned at 617 of
901 on its first walk (item 660)~~ agrees to its end, 900 (items 660, 664,
668), its
capture cut at 901 by the allied victory its own `ally` line causes (§12);
chapter six-b ~~is pinned at 632 of 1250 (item 651)~~ agrees to its end,
1250 (item 680); chapter nine, the first issuer chapter, ~~is pinned at
693 of 1100~~ agrees to its end, 1100 (item 676, §17). **A chapter whose
word is its trace's last block is closed**, and the handoff's `Golden:`
line says so — `chN closed` — and leads with the lowest *open* chapter's
word, which is the rules headline; `rondata::diff::floors` reads the line
that way (the eleventh pass, `docs/DECISIONS.md` 47). ~~What the line
should say once a second chapter pins is the commander's to rule~~; the
design's recommendation stands — the **lowest** chapter's word first,
because that is where the next cause lives. What a chapter's word means is
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
5. **Where each falsifier could first fire** — the frame and the record,
   checked against the staging *before* the run (parked 584, the twelfth
   pass). run145 staged §7 as written and two of its three falsifiers
   could not fire: the catapult was born packed and unpacked with nothing
   in view, and the chariots shot without moving. A chapter whose staging
   cannot reach a falsifier is restaged before it is pinned — run146 is
   the form, and `chapter3b.cmd` says per falsifier when it cannot fire —
   and a chapter whose *premise* a falsifier kills is closed on what it
   measured and restaged as a new chapter, never reopened (§11).
   **Two more things the staging reads before the run** (the thirteenth
   pass, parked 630 and 667): the premise names the function that would
   kill it, and the booking greps that function's writers and the dumps
   on disk — seven-b was booked on a block `Unit::init` had already
   closed for every computer unit, and its first falsifier fired by
   construction; and a falsifier whose premise rests on a loop cites the
   loop's bound from the listing or the PDB — chapter eight read
   `set_diplo`'s loop over leaders as every player where the bound is
   `0..7`, and `ally` ended the game.

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

**Lines** (`chapter3.cmd`): `!ai off` at 0; `age who=0 2` and `age who=1 2`
at 600 and 602 (~~`4`~~, which is Gunpowder: `age`'s level counts from one,
item 587) — `age` rather than `library`, so that chapter two owns the
`library` arm and a parting is attributable to one of them;
`add 3 chariot who=0 4,40` at 610, the leading count placing three separate
one-unit Chariots rather than a squad; `add hoplite who=1 12,40` at 615;
`add catapult who=0 4,41` at 620, eight tiles from the hoplites.

**The capture must dump** the same set as chapter two, over `[605, 900)`.

**What would falsify it.** The catapult firing at a target inside three
tiles, which would say the minimum is not enforced in the fight. Three
chariots whose per-frame displacement equals the hoplites', which would say
the line's speed is not read. A leading count of 3 producing nine units or
one, either of which would overturn `docs/INPUT.md` §11.5's reading that the
count is a count for a unit type and not for a building.

**Run, 2026-09-23, run145 (item 587). None of the three fired, and two
could not have.** `docs/RUNS.md` run145 has the receipt. The leading count
holds: three separate Chariots on 611. But the catapult is **born packed**
(`unit_masks 0x80000`), takes no attack order, starts its unpack (`spell
652`) on 696 and never fires a round. The chariots shoot the hoplites dead
without moving, and no hoplite comes within 3 tiles of the catapult. The
minimum-range and speed falsifiers are **untested** by run145, not held.

~~**The word is 621**, the catapult's birth frame. This crate orders the
packed catapult to attack and spends `Unit::fight+0x9b0`'s re-search draw,
7 against the original's 6; values part on 622.~~ Item 590 took
`Unit::think_attack`'s packed-unit arm (`docs/COMBAT.md` §51): a human's
packed siege engine returns before the target search and unpacks at `idle
7` on the auto-attack's 32-frame phase. The catapult now agrees with the
dump on every row of run145. ~~**The word is 633**, 8 draws against 9, and
values part on 634~~ (`GOLDEN_WORD_CHAPTER_THREE`,
`chapter_three_s_word_frame_is_widened_whole`, which widens run145 whole
and adds the `GUY` record's position and facing). Item 595 took the
chariot's pivot (`docs/COMBAT.md` §52): a type whose `<RESTRICTION>`
pivot bears within ±45° shoots on its own heading, and its swing rolls in
`fight`'s frame. `0/8` now agrees on every row of 633 and 634. ~~**The word
is 682**, 8 draws against 7: this crate kills the hoplite `1/7` there,
where the dump's dies on 704. It is downstream of the first value
parting, **635**, where the chariot `0/6` targets `1/7` here and `1/8` in
the dump. No mechanism is named for 635.~~ Item 601 read two things in
`get_damage` off the listing (`docs/COMBAT.md` §53). The flank reduction
is keyed on the target's mask, so a chariot flanks hoplites at the whole
bonus and `0/6` takes `1/8` on 635. The ranking skips overkill, so `0/7`
takes the wounded `1/7` on 685. ~~**The word is 684**, 14 draws against
15. `0/8` turns to `1/7`, 50° off its heading, where the original swings
on its heading. That reads as the pivot node's offset (item 603).~~ Item
603 measured the node (`docs/COMBAT.md` §54): the pivot bears from the
unit's point plus the node's vector, `(−102, −59)` for the Chariot at
120°, and `1/7` is 42° off from there. ~~**The word is 706**, 18 draws
against 20: a round landing with no live target, whose ±20 scatter
(`Ammo::do_damage+0xc59`/`+0xc7e`) the original spends and this crate
does not. Values part on 707 on rounds only. The first value parting is
**651**, the chariots' first rounds leaving from the unit's square
(item 602's shape), and 706's rounds carry it.~~ Item 602 found where a
pivot piece's round leaves (`docs/COMBAT.md` §55): through
`get_position`'s pivot branch, the release node carried by the pivot node
and rotated by the facing **and the turret's angle**, which this crate
now carries. **run145 holds to 900, the capture's end: word, sequence
and values. Chapter three is closed on run145.** What the widening still
parts on, with no draw after it, is three families of the `AMMO` record:
a round's target, cleared here on its target's death where the original
keeps it to its due frame; a round's pool slot; and one turned
chariot's launch point, a unit off on 729 and 753 (§55.5). The
`rolling` row 595 and 603 read at 651 was the instrument: it compared
flag 4 with `AmmoData +0x5`, a byte `Ammo::init` zeroes (§55.1). The widening now also compares
each figure's aim and the `ATTACKORDER`'s own row (`in_range`,
`new_ord` and the rest).

**Restaged, run146 (`chapter3b.cmd`, the same item): all three reachable,
none fired.** The same three unit types go into two arenas. A catapult born
alone unpacks on idle (`Unit::think_attack`'s human threshold, `idle 7`)
and is ready before its hoplites exist. Chariots born ten tiles from their
hoplites have to walk. The catapult fires once at eight tiles, then
refuses the hoplites inside three for 173 blocks, dropping each attack
order the block after it takes it. The chariots walk at up to 33.2 units a
block against the hoplites' 29.1 (`chapter_three_s_falsifiers_are_the_dump_s`).
run146's first parting is **633** (`GOLDEN_WORD_CHAPTER_THREE_RESTAGE`).
~~The chase's first frame: the original's chariot starts its walk
animation where this crate re-searches.~~ Since item 590, run145 parts on
633 in the same shape: the chariot `0/8`'s one `Unit::fight+0x9b0`, then
two `Guy::set_anim+0xf2f` in the original and a second re-search here.
In neither capture does the dump's `0/8` move or turn from 630 to 720.
Its figure takes `ox 8 whom 1` and its reload starts (`recharging 25`) on
634, and this crate's `0/8` has turned by then. So the pair is an attack
swing started without a turn, not a walk (`docs/COMBAT.md` §51.3). ~~**The
chapter's word is 621**, the lower of the two.~~ ~~Both captures' words are
633.~~ Item 595's pivot moved run146 to **664**
(`GOLDEN_WORD_CHAPTER_THREE_RESTAGE`), 31 draws against 30. The chariot
`0/9`'s horse rolls a fresh swing where the original's rolls nothing,
because this crate's crew figure has stood on a walk slot since its first
swing. The first value parting is **651**: `0/8`'s first arrow leaves
from the unit's square, not the archer's release node. The two words have
moved apart. run145 is at 682 and run146 at 664, the lower
(`docs/COMBAT.md` §52.4). Item 602 moved run146 **664 → 780** with two
things: the release through the turret, and the crew swinging with its
leader (`docs/ANIM.md` §5.2): figure 0, stepping into the attack
category, puts its crew on its own slot and clock, so a crew that fell to
a walk slot while figure 0's swing waited for a turn does not stay there.
~~**The word is 780**, 5 draws against 6: the catapult `0/6`, unpacked on
779, takes an `ATTACKORDER` on 780 (`Unit::fight+0x9b0`) and turns to it
on 781 in the original; this crate's stays idle. No mechanism is named.~~
Item 616 found why (`docs/COMBAT.md` §56): `cast_unpack` lights the whole
disc at the new line of sight, and this crate's kept the packed four-tile
disc, so its 780 search could not see hoplites seven tiles off. The
catapult now takes the same hoplite on 780 and turns on 781 as the dump's
does. **The word is 782**, 7 draws against 5
(`GOLDEN_WORD_CHAPTER_THREE_RESTAGE`). Values part on 781 on `0/6` alone:
the original pushes an `ATTACKGROUNDORDER` over the attack at the
target's point (`Unit::fight`'s siege arm and `Unit::do_attack_ground`,
§56.3), which this crate ~~does not carry~~ carries since item 621
(`docs/COMBAT.md` §57). Every `0/6` value row the order made on 781–783
is gone, and the order's life agrees to 864, its round included. ~~**The
word stays 782**, and it is not the order: this crate's two extra draws
are the catapult's crew starting a walk on the turn in place, where the
original's crew stand on their slots and mirror figure 0's `TURN_LEFT`
(§57.6; run44's `0/15` has the clocks).~~ Item 625 found why
(`docs/COMBAT.md` §58): `Guy::move`'s moving arm asks for no walk for an
unpacked packer, so the crew, pulled round to their rotated slots,
keep theirs and take figure 0's turn through the mirror. The crew stand
on the dump's points and play figure 0's slot on 781–785, and the
round's landing on 798 reads the dump's. ~~**The word is 792**, 28 draws
against 29 at draw 24: the original's arena-A hoplites take an attack
on the catapult on 792 and one spends `Unit::fight+0x9b0`; this crate's
take it on 795. The idle's +1 every sixteen frames is phased by the
object number, and this crate numbers those hoplites 6–8 where the dump
numbers them 9–11.~~ Item 617 widened the numbers
(`chapter_three_s_restage_numbers_its_objects`). The dump culls nothing:
the dead hoplites' `DEATH_OBJS` live to 999 on both sides. What parted
on 771 was the allocator. `Objects::find_free` skips a dead number that
its death object still holds (`docs/COMBAT.md` §59), so arena A's
hoplites are 9–11 on both sides now, and reach `idle 4` on the dump's
frames. ~~**The word is 865**, 5 draws against 4, and values part on 866:
the catapult's fresh attack after its reload, which is §57.6's third
residue (621's park).~~ Item 627 found two things (`docs/COMBAT.md` §60).
An unpacked packer's idle search must reach what it takes
(`find_nearby_target`'s `local_24`), so on 864 the catapult takes none of
the hoplites standing inside its minimum. A packer also re-searches before
it chases (`fight:1051`), so each hoplite that hits it after that is
attacked for one block and let go, and the catapult never moves.
**run146 holds to 1000, the capture's end: word, sequence and values.
The restage is closed** (`GOLDEN_WORD_CHAPTER_THREE_RESTAGE`,
`chapter_three_s_catapult_after_its_reload`). §7's minimum-range
falsifier is diff-backed now on both of the catapult's paths: the idle
search and the retaliation. What the widening still parts on, with no draw
after it, is three families: a round's target (736, run145's family), a
round's pool slot (798) and the scout `1/0`'s move facing (847).

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

**Run, 2026-09-23 (item 552): run132 the border, run133 the bleed, and
none of the five falsifiers fired.** The two captures are one game, draw
for draw on all 1501 frames. The border window is `[295, 545)`, not
`[295, 345)`: the narrow one sees the Temple and neither other lever, and
`LeaderData::territory` is printed only from `LEADERS=8`, so the `WORLD`
cell count is the only cheap reading.

- **The Temple lands finished**: `(int)construct_hits 1200` = `myhits`, on
  cell (7, 40), block 301. `run_cmd` calls `Build::activate(0, 1, 0)`
  after `init_build` for a line without `NEW`, so `finish` is not owed.
  Napata's `city_flags` gains `0x80` on the same block.
- **Every lever moves the border.** Owner-0 cells go 266 → 296 (Temple),
  → 327 (Religion) and → 445 (Civic 3). Each change starts five blocks
  after its line and spreads over five blocks, the budgeted sweep.
- **The squad bleeds at 48**, 6/16 per figure per tick, on the 48-grid
  from its first refresh. The **scout** and the **wagon** read 0 on every
  block.
- **The wagon shelters, but only once it is in reach.** The squad did not
  stand where it was placed. By 1101 it had marched about 30 tiles east,
  still on player 0's ground, and the wagon trailed after it. Every tick
  with the wagon 23 tiles or more away landed (1145 … 1337). Every tick
  due with it at 11–13 tiles was vetoed, with `unit_masks2`'s `0x40000` on
  that figure's own tick frame (1385 on). The prediction's condition,
  "with the wagon within 14 tiles", was not met until then, so the
  falsifier stayed quiet, and the radius is bracketed rather than hit.

**What the harness met**, five defects, each a widening row:
`docs/RUNS.md` run132 and run133, and `GOLDEN_WORD_CHAPTER_FOUR`. The
chapter's word ~~is **1277 of 1501**~~ was 1277: on it the original gives
the squad a `GUARDORDER` and a move back beside the wagon, and this crate's
squad kept marching.

**Item 567 moved it 1277 → 1416.** The guard is the army's own: on its
256-frame tick, `action_siege_attack_to` anchors a siegeless army on its
Supply Wagon and `action_guard` gives the rest an escort's post. This
crate had the anchor and not the guard (`docs/ORDERS.md` §24). The bleed
and the shelter now agree on every block to 1500. **1416** is the wagon's
frame: the original's wagon holds `pause 15`, a collision wait with the
escort at its heels, and this crate's wagon, which has walked its own line
since 1102, holds 0. The delta is +1, 32 draws against 31.

**Item 569 moved it 1416 → 1500, and chapter four is closed.** Both
causes were the wagon's own. This chapter's Supply Wagon is the first
unarmed attack-mover in the corpus, and it reached two seams this crate
had named and never built. Its `find_wpath` plans as an army through
`is_supply`, which put its route round the `0x200` cells, one leg longer
(`docs/PATHFINDER.md` §25). Every fifteen frames `do_attack_to_pause`
stops it for fifteen frames when its escort's captain is within `0x600`,
and it stands each waiting frame out (`docs/ORDERS.md` §24.9). The draw
count, the site sequence and every dumped value agree on every block to
1500. The only rows left are the capture's standing ones and three
residues under 1174, none of which spends a draw: the scout's explore
`facing` (parked 275), the group id's numbering, and the scout's second
figure on 1172.

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
nothing under the word parts. ~~On 739 this crate ends an attack the
original does not: `1/6`'s `CHAR_ATTACK3` runs three frames.~~

**The word is 900, run127's end,** since item 549 (`docs/ANIM.md` §4.13).
The Trireme's packet has no `CHAR_ATTACK3`, and `Guy::set_anim` plays
`CHAR_ATTACK2` for an attack slot the packet lacks. No draw parts on any
frame of chapter five, and all 506 rounds agree on every field. Under the
word only the standing rows remain, plus `1/0`'s explore-order `facing` on
847, the non-scoring formation mirror.

## 10. Chapter six — the air, and the one command that issues an order

**Premise.** Three of the six order classes no document cites are the air
ones. And there is one door into the order family the channel already has.

**`bird` is the only console command in the executable that issues an
order.** `ConsoleWin::run_cmd@007d6a70`'s case 82 — the table index of
`bird`, whose help text is "Drop a Wild Bird at Mouse" — calls
`Objects::init_unit@0065e0c0` ~~for gaia type 9~~ **for owner 9 and type
`0x192`, `BIRD`** (the listing pushes both, 0x7df1b1 and 0x7df1b6; item
648) and then `Unit::add_air_patrol_order@005e4350` on it, on the same
point. It is the *only* `add_*_order`
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

**What would falsify it.** ~~No `AIRPATROLORDER` block after frame 700~~ **No
bird born on 700 in the draw stream** — the reading of case 82 is wrong, or
the bird did not survive its placement. (No dump prints owner 9, so the
block could never have been written: item 648.) An
aircraft record whose position never changes, which would say an air unit
staged outside an airbase is inert. Or the `add` refusing the Fighter
outright, which would say the air line needs a base the way chapter five may
show a hull needs a Dock.

**Where each could first fire, read before the run (item 648,
`chapter6.cmd`'s header, committed as `bf981e6`).**

- **The cursor is heap.** `console_win` is an uninitialised `malloc` of the object's size
  (`System::init@00599700`). Its `mouse_coord_x/y` are written only by
  `parse_cmd` with `no_mouse` 0 and by `CommandPackage::process_console_cmd`,
  and the channel reaches neither. The prediction was zero, which puts the
  bird at (24, 24) with its patrol point on the corner. ~~zero~~ **Measured
  (0, 6)** (item 652, run169's packet at logger frame 701): the
  `ConsoleWin`'s own `mouse_coord_x/y`, and the one fresh
  `AirPatrolOrder`'s waypoint, agree. The seat is (24, 24) as predicted,
  since `Unit::init` snaps any cursor in the corner tile there. The patrol
  point is six units down the west edge. A walk with the cursor at (0, 0)
  lands the 750 and 791 edge coins and parts on 894, two frames before the
  bird's third.
- **The first falsifier fires on the trace's frame 700**, not in the dump.
- **The second is an absence** on the aircraft's `UNITDATA`, 611 and 616
  to 899.
- **The third fires on 611 or 616.** `add` has no base or age test, and
  `UnitType::find_nearby_spot@0061de70` skips the terrain test for the air
  domain.
- **`!ai off` does not reach the Bomber.** It is AI-driven (`0x40000`,
  `docs/INPUT.md` §11.10), so its idle `Unit::think_attack@005f5a80` has the
  army and city arms the human's Fighter lacks. A move of the Bomber was to
  be read off its order stack.

**Run 2026-09-23 as run168 (item 648): the second falsifier fired.**

- **Neither aircraft moves.** The Fighter `0/6` stands on its seat, (888,
  7800), and the Bomber `1/6` on (2424, 7800), from birth to 899. Both
  are at `air_alt` 0 with an empty order stack, and no `AMMO` block is
  written in the whole capture. They are eight tiles apart and inside
  each other's line of sight.
- **What each does do.** Each burns fuel, `mana_burn` one a frame, as
  `Unit::process@00610bc0` does for an air unit with `inside_up < 0`, and
  its `idle` climbs. The Bomber sits in the AI's group 64 from its birth
  block.
- **The first and third did not fire.** The bird is born on 700 (draw 0,
  `Guy::init_real < Unit::init < Animal::init`). A seventh `think_bird`
  runs from 704, and the sampling is a pair short from 704 (4 → 3). The
  `add` placed both aircraft with no base.

**So the premise that "what closes the gap is the air line's own
movement" is killed.** An aircraft the channel stages outside a base is
inert on this lobby, human's or computer's. What the chapter measures is
**an unbased aircraft's idle upkeep**, and **the bird the channel orders**,
which only the draw stream sees. It is pinned open as measured, as seven-b
was, and an air line that moves needs a restage with a base.

**Where this crate parts: `GOLDEN_WORD_CHAPTER_SIX` = ~~616~~ ~~700~~
900, closed.** 900 is run168's trace end, and no draw parts on any frame.

- **Item 652 moved it 700 → 900** by staging `bird`: `init_unit(9, BIRD)`
  and the air patrol at the channel's cursor, through the sampling's own
  entry point (`Sim::spawn_bird_at`, `docs/SYNC.md` §3.9). The cursor is
  **(0, 6)**, measured on run169's packet (above). The bird's birth draw
  on 700 puts the AI scout's `think_scout` roll back on the original's
  draw. Its edge coins land on 750, 791 and 894, the three the capture
  has. **At (0, 0) the walk parts on 894**, the third coin, two frames
  early, and that is the check that can fail: the widening passes at
  either cursor, because nothing prints owner 9. The value diff is run168
  whole, 605 to 899: past the first block's standing rows only the two
  aircraft's `form` on their birth blocks part, and the 701 rows (the
  scout's move order and path) are gone.
- **Item 650 moved it 616 → 700.** Neither aircraft's idle search takes
  the other in the original. `Object::poor_target`'s plane arm refuses a
  plane to a searcher without `ANTI_AIR`, and to one with it beyond its
  own `max_range`. `valid_target_const`'s air ladder refuses a `FLY_HIGH`
  0 searcher a plane with no air order (`docs/COMBAT.md` §61).
- **616 was 648's word**, the Bomber's birth frame: a `Unit::fight+0x9b0`
  the original does not spend, and an `ATTACK` order where the original
  holds none (`docs/journal/2026-09-23-item-648.md`).

Both leaders agree on every `LEADERS=2` row of every block, so `library
6`'s `gain_tech` tail agrees through the Modern age (`docs/INPUT.md`
§11.11). The chapter's premise stays killed: an air line that moves needs
a based restage (parked 651).

### Chapter six-b — the air line from a base (item 651)

**Premise, as booked.** A based aircraft moves where an unbased one did
not. **Lines** (`chapter6b.cmd`): chapter six's, with one lever added —
`add airbase who=0 4,33` on 606 and `add airbase who=1 12,33` on 608, each
centred seven tiles north of its own aircraft's seat — and a window to
1250, so that both aircraft run out of fuel inside it. **The capture must
dump** chapter six's `end:` set plus `BUILDS=7`, over `[605, 1250)`.

**The reading kills the base link before the run** (the file's header
has every citation). An aircraft moves only on an air order, and run168's
had none. The base's three writers of one — `Object::do_launch@0064f3b0`,
`Object::attempt_launch@00643a10` and `Build::train@0062f9b0`'s gather arm
— each walk the base's `inside_down` chain, and **`do_launch`'s whole body
sits under `-1 < inside_down`**, so an empty base spends nothing. What
fills the chain is `Unit::go_inside@0061a2e0`, and `add` never calls it:
`Build::train` seats its trainee inside the base, and `add` is
`find_nearby_spot` and `init_unit` and nothing after. No caller of
`go_inside` is reachable by a channel line under `!ai off`. **So the
premise's base-link half is predicted dead by construction**, as seven-b's
was (§11). What this chapter can still teach is below.

**The Airbase is also a target, and that half is open.** who=1's Bomber
is AI-driven, so its idle search reaches 24 tiles
(`UNIT_RESPOND_RANGE × 0x180`), and who=0's Airbase is ~10.8 tiles away.
Whether a `FLY_HIGH 0` plane on the ground takes a building is not read
(COMBAT §61 covers a plane target only). The order stack separates the
halves: a target on the enemy Airbase is the target arm; `inside_up ≥ 0`
or a targetless air order is the base arm.

**What would falsify it, and where each could first fire** (§3, point 5):

1. **An aircraft moves, takes an order, or goes inside**: the Fighter's
   `UNITDATA` from block 611, the Bomber's from 616, to 1249. Predicted
   to fire on the target arm only, if at all.
2. **The Airbases displace the seats**: blocks 611 and 616, a seat other
   than run168's (888, 7800) and (2424, 7800). Predicted not: neither
   footprint (5 × 9 tiles, rows 29–37) reaches row 40.
3. **An empty tank does something**: `mana_burn` reaches `mana()` on
   block 1010 for the Fighter (400) and 1215 for the Bomber (600). A
   death, a move or an order from then on fires it. Predicted not:
   `Unit::check_fuel@005e9be0`, the one reader that acts on an empty tank,
   runs only under an air order.
4. **An empty base spends something**: from 607 and 609, any draw under
   `do_launch` or `attempt_launch`, a `launch_frames` above 0, or who=1's
   Airbase disbanded by `Leader::check_orphaned_buildings@006c9f20` (its
   disband arm needs `build_masks & 1`, written only by
   `Unit::resolve_block@005fccc0`). Predicted not.

**Run 2026-09-23 as run175 (item 651): the first falsifier fired, on the
target arm.**

- **Each aircraft takes the enemy Airbase.** On its birth block the
  Fighter `0/6` holds an `ATTACKORDER` on who=1's Airbase and the Bomber
  `1/6` one on who=0's, and on the next each adds a `MOVEORDER` to an
  attack point beside it. Both **walk there on the ground**, `air_alt` 0:
  the Fighter reaches (600, 7944) on 632 and its stack is empty from 633;
  the Bomber reaches (1992, 7704) on 700 and holds `ATTACK` with a
  one-frame `MOVE` on every other frame to 1249, never moving again.
  Neither Airbase takes a point of damage.
- **The base link is dead, as read.** `inside_up` −1 and `air_alt` 0 on
  every block of both; each Airbase's `inside_down` −1 and
  `launch_frames` 0 throughout; no air order anywhere but the bird's.
- **The second, third and fourth did not fire.** The seats are run168's;
  `mana_burn` stops at 400 on 1010 and 600 on 1215 and nothing else
  changes; no draw under `do_launch` or `attempt_launch`, no disband.

**So the chapter measured an aircraft's ground attack on a building, not
an air line.** An aircraft outside a base is not inert: it is a ground
unit that its own idle search sends at the nearest enemy building it may
take, and run168 simply had none in reach. What flies — an air order on a
player's aircraft — still needs a writer the channel does not reach: a
trained aircraft inside its base, or an issuer (§13).

**Where this crate parted: `GOLDEN_WORD_CHAPTER_SIX_B` = 632, open (item 651); closed at 1250 by item 680, below.**
Frames 605–631 agree: this crate takes the same Airbase on the same birth
block and walks the same path to the same attack point. On 632, the
Fighter's arrival, ours spends 34 draws against 24, parting at draw 18 on
ten `Unit::find_attack_pos+0xea9 < Unit::fight+0xcb4` of `0/6`'s: at its
point and out of range, it asks for a new attack position and gets one.
The original spends none and drops the attack, so its stack is empty on
633 — `Unit::fight@005fd4d0:1059`'s arm, where `find_attack_pos` answering
0 runs `find_new_target` and kills an attack on the same target. ~~**Which
exit of `find_attack_pos@00601280` answers 0, drawless, for an air-domain
attacker on a building is not read**~~ — no exit does: on the word's
packet it answers 1 with this crate's ten draws, and the original never
calls it (item 680, below) (957 lines, the ranged arm under
`type +0x2c8 & 0x400`), and that is what moving the word needs. The value
diff is `chapter_six_b_s_word_frame_is_widened_whole` over (605, 635):
past the first block's 13 standing rows, each aircraft's `form` on its
birth block, each one's `order:target` there (the harness's: `build_ids`
names no building staged after `BEGIN GAME`; with a fallback to the
building's own `(owner, index)` both rows go), and the Fighter's
`orders.len`, `order:length` and `dest_angle` on 633 and `idle` on 634.
The Bomber agrees on every block to 634.

**Closed at 1250 by item 680** (`docs/COMBAT.md` §62; run177's packet at
logger frame 632, `docs/RUNS.md`). The word's premise was wrong in its
arm: `fight@005fd4d0:1059` never runs on 632. Run on the packet,
`find_attack_pos` from `fight`'s own call answers **1**, out (600, 7944),
with exactly this crate's ten `+0xea9` draws. `fight` itself, run on the
packet with `do_attack`'s arguments, returns 0 with **no draw**, never
enters `find_attack_pos`, and calls `find_new_target` from `fight+0xa1f`.
That is the captain arm `LAB_005fddf7`: a non-mandatory attack by a
captain whose target is **not a unit** re-runs the idle search on every
frame (`005fdeb4` → `005fdf50` → `005fdeea`), with no roll and no
`poor_target`. The Fighter's search at its attack point reaches twelve
tiles (`UNIT_RESPOND_RANGE × 0xc0`, a human's), and the Airbase is about
12.2 off, so the search finds nothing and the attack dies where the
Fighter stands. From its seat, 10.6 tiles off, the same search had found
it. This crate carried the arm for a unit target only (§8.2 step 0).
With it built for a building, the walk agrees to run175's end: sequence
1250, no value part. The Bomber's alternating `ATTACK` and one-frame
`MOVE` from 700 to 1249 (parked 682) agrees on every block too: its
AI-driven search reaches 24 tiles and re-finds who=0's Airbase each frame,
and the chase that follows moves it nowhere. The widening,
`chapter_six_b_s_word_frame_is_widened_whole`, now spans run175 whole
(605 to 1249). Past the first
block's standing rows only each aircraft's `form` and the harness's
`order:target` part on their birth blocks, 611 and 616, and one row that
is neither aircraft's: the scout `1/0`'s non-scoring `order:move.facing`
from 991 (parked 275), pinned by value as chapter five pins it on 847.

## 11. Chapter seven — the civilians, and what AI-off takes away

**The premise below was falsified by its own capture, and the chapter is
closed at 1200 on what it does measure** (item 578, run141 and run142).
`ai off` takes nothing from a human's civilians: the citizen takes the same
`GATHERORDER` on the same block with the cheat on and off, from
`think_peasant` above the cheat's block, and the other four stand in both
(`docs/INPUT.md` §11.9). So this chapter measures **a human's idle civilian
behaviour** — a gather issued at `think_peasant`'s wait, walked and
delivered, and four units that take no order — **not the gate**. That
record agrees with the original to the end of both captures:
`GOLDEN_WORD_CHAPTER_SEVEN`, `chapter_seven_holds_to_the_golden_word` and
`chapter_seven_s_control_holds_to_the_golden_word`, widened whole by
`chapter_seven_s_word_frame_is_widened_whole`. The design as written
follows, unchanged, because its falsifier is what fired.

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

**Run 2026-09-23 as run141 and run142 (item 578): the first falsifier
fired, and the premise is wrong.** The citizen takes a `GATHERORDER` on
block 763 in *both* captures, from `think_peasant` above the `ai off`
block, and the other four take no order in either; the five's orders are
identical block for block. For who=0 the block is entered whether the
cheat is on or not, because the human carries `leader_flags & 4` (item
437), so `ai off` takes nothing from a human's civilians
(`docs/INPUT.md` §11.9). The second falsifier did not fire. The harness
agrees with both captures to their end, 1200, and the commander ruled it
pinned closed as it stands; restaging it on the computer's civilians, where
`ai off` does bite, is parked for the steering pass.

**Restaged by the twelfth pass as chapter seven-b (item 628; run156 and
run157, §14).** The cheat's block decides something only for a leader
without `leader_flags & 4`, so the same five civilians are staged for
**who=1**, the computer, once with `!ai off` and once without, the lines
otherwise chapter seven's. What it tests is what every other chapter's
who=1 targets stand on and none has measured on a civilian: that the cheat
closes `Unit::think`'s tail for a computer's units, and that this crate's
one-term stand-in — `ai_off && !ai_driven(owner)`, `docs/INPUT.md` §11.9's
seam — closes and opens the same tail. **Its falsifiers, and where each
could first fire** (§3, point 5): a `GATHERORDER`, `TRADEORDER` or carry
on any of the five in the AI-off run at or after the citizen's
`think_peasant` wait (block ~763 on run141's cadence) — the block does not
close the tail; **no** order on the citizen in the control by the same
block — the AI's economy never reached it and the pair is vacuous; a
draw-stream parting under the word in this crate's walk of either capture
— the seam is real and this is its frame. **The closed chapters did not
lean on the fallen premise**: their who=1 units are soldiers, whose
auto-attack arm runs above the block (item 590), and each closed in
lockstep to its trace's end.

**Run 2026-09-23 as run156 and run157 (item 628): seven-b's premise fell by
construction, as `chapter7b.cmd` predicted before either run.**
`Unit::init@00612100:585` sets `unit_masks & 0x40000` for every unit of a
leader whose `flags & 0xc` is not 4, which is who=1. So the block's one exit,
`think:264`, is never taken for a computer's unit, and its arms need bit 4
(`docs/INPUT.md` §11.10). The records that show it are the five's
`UNITDATA`. Every one carries `unit_masks & 0x40000` on its birth block in
both captures, as do run141's who=1 units and run146's cheat-`add`ed
hoplites. The citizen's order stack holds a `GATHERORDER` on its birth
block, 611, under `!ai off`, and that is **the first falsifier firing**. The
control's citizen holds the same order on the same block, so **the second
does not fire**. The caravan and scholar never take an order. The merchant
and fur trapper move on birth and cast later: the fur trapper on 1151 in
both, the merchant on 900 under the cheat and 887 without it. The pair is
one game through frame 0 and parts on frame 1, 12 draws against 54. **So
this chapter measures a computer's idle civilians, not the gate**, and the
`|| ai_off` term decides nothing for a unit born to its current owner on
this lobby, human or computer (`docs/INPUT.md` §11.10).

**The third falsifier, where this crate parts.** Both captures part in the
walk, and neither parting is the seam. The one-term stand-in and the
original both leave who=1's tail open.

- **run156, `GOLDEN_WORD_CHAPTER_SEVEN_B` = ~~1148~~ 1200 (closed, item
  629, below)**: 13 draws against 14 at
  draw 6. The original's fur trapper `1/10` spends two turn draws, guy 0
  under `move_step+0x3b6` and its crew under `do_turn+0xe5`; this crate
  spends one `Guy::inc_time` stand. Its move was handed another `dest_y` on
  1091, 14408 against 14804. The merchant `1/8` stands 24 units off the
  original's when its cast ends on 1070. The sequence parted first at 672,
  on the same unit's crew turn, where the trace could not name the
  original's chain. Item 628 named it (`docs/ANIM.md` §4.14), and the
  sequence moved to the word.
- **run157, `GOLDEN_WORD_CHAPTER_SEVEN_B_CONTROL` = ~~1036~~ ~~1176~~ ~~1187~~ 1200
  (items 632 and 644, below)**: 8 draws against
  7 at draw 2. who=1's own `1/1` takes an idle roll here where the
  original's walks. From 990 the original holds it in group 65 with three
  orders and `form_mod` 50, and this crate holds group 64, one order and
  no formation. It is the AI's, not one of the five.

Both are widened whole over their windows in
`chapter_seven_b_s_word_frame_is_widened_whole`, with every parted row
pinned by block and key. ~~Two more rows stand from the first block. who=1's
`resource_cap` is 1392 here against the original's 2992, because
`library who=1 2` raises the original's cap and not this crate's (run141,
without the line, prints 1392).~~ Gone on item 644, below. The scout
`1/0`'s `facing` stands from 847, as in chapter seven. No mechanism is named for either word.

**Seven-b closed at 1200 on item 629.** The 24 units were the mechanism.
`SpellType::cast_unpack`'s merchant arm seats a Merchant on its tile corner
and blocks the two-by-two under it, and this crate had it as a seam
(`docs/MERCHANT.md` §3.2). With the arm built, the merchant `1/8` agrees
from 1070 and the fur trapper's `dest_y` agrees from 1091. run156 agrees
draw for draw and value for value to the end of its trace, and 22 rows
under the old word go. The one left is the scout's `facing` from 847,
parked 635. The control's 1036 does not move.

**The control moved 1036 → 1176 on item 632.** The group was the goody
box's, not an army's. On 990 the fifteen-frame look halts `1/1` on its way
to a site with a one-member group's `QUEUE_FIRST`. The original's
`Group::finish_insert` re-issues the copied `BUILDORDER` behind the box's
walk, as `action_swarm_around(…, QUEUE_LAST, BUILD_AT, 1)`, and this crate
dropped it. The group's width twin, `form_mod` 50, is written on a citizen
too (`docs/GROUPS.md` §24). With both built, `1/1` walks on to its site on
1037 as the original's does. On 990 only the group id stands, 64 against
65, a slot permutation that starts with the scout's standing row. On its
own tree the fix stopped at 1148, run156's word, and item 629's merchant
arm clears that frame here too. The word is now **1176**, the computer's
`Leader::produce_building`: 69 draws against 249, parting at draw 34,
where the original spends a run of `Build::find_gather_tiles+0x10a`. On
1177 the original's gatherer `1/7` is sent toward a site and this crate's
keeps gathering. The scout's explore path parts from 1077, value only. No
mechanism is named for 1176.

**The control moved 1176 → 1187 on item 644: the `library` line itself.**

- **What the 180 draws are.** They are not the Farm's. The Farm `1/2008`
  is draws 0–33, and it agrees. The 180 are a Woodcutter's Camp's shuffle,
  45 tiles. `place_woodcutter` places the camp beside the unfinished Small
  City `1/2007`, finds it under five workers, and destroys it within the
  same frame. So no dump block prints its `1/2009`, and only `1/7`'s
  `BUILDORDER` names it.
- **Why this crate did not buy it.** It had 23 food against a price of 70;
  the original had 98. `library who=1 2` raises the levels through a whole
  `gain_tech` per step, and the Classical age's step pays knowledge and
  metal 100 each. This crate's handler skipped that tail. So on 1018 a
  goody pile of 75 went to metal here, the good at 0, and to food there
  (`docs/INPUT.md` §11.11).
- **What else the handler fixes.** The same section makes the caps read
  the Commerce level live, as `calc_resource_caps` does on every frame,
  which was parked 633. The first block's knowledge, metal and
  `resource_cap` rows go in both captures of the pair, and in chapter
  seven's and chapter three's.
- **Why `1/7` walks on.** Once the camp is bought, the original's `1/7`
  keeps its walk and `BUILDORDER` on the dead camp to the capture's end.
  `Unit::work`'s target test is the `uid`, and a closed building keeps
  its uid until its number is reused. This crate had it as the alive bit.
- **The value diff.** On 1177 `1/7` stands at (40271, 18360) on both sides,
  holding explore-to (40248, 23160) and the `BUILDORDER` on `1/2009`.
  Nothing new parts on 1177..1188.
- **The word.** It is now **1187**: the scout `1/0`, ours 10 draws against
  9, parting at draw 0 on `Unit::do_move+0xe84`. The scout's explore path
  has parted since 1077, value only. ~~No mechanism is named for 1187.~~
  Named and built on item 647, below.

**The control closed at 1200 on item 647: the fog under a city its owner
placed out of sight.** The scout's path was built on frame 1076, and
run157's own `rontrace.cfg` proxies `calc_cost` across the whole capture.
So the search that built it was on disk, and no capture was needed. Nine
of the steps both sides priced answered differently, every one into the
south half of the Small City `1/2007`'s footprint: 304/312 in the
original, which is seen ground carrying its blocked tiles, and 1/9 here,
which is unseen ground to a scout. `Wall::start` ors the owner's bit into
`seen2` over the footprint, and this crate did not, because the owner's
line of sight was assumed to cover it. The AI placed this city on 1069
out of its own sight. With the write, the search is 945 steps on both
sides and every one is priced alike. **The value diff**: on 1077 `1/0`
holds 48 path nodes on both sides, `path[41]` (35832, 24312) through
`path[47]` (40440, 25080). On 1116 it stands at (39984, 25092) with
`dest_y` 25080, and on 1138 its `dest_x` is 38136, both sides each
time. **run157 agrees draw for draw and value for value to the end of
its trace, and that closes the last open golden chapter.** Under the
widening, only `1/1`'s group id on 990 stands (`docs/GROUPS.md` §24).
`docs/SCOUT.md` §14 has the readings and what killed each, and
`docs/VISION.md` §6.1 has the write.

## 12. Chapter eight — the commanders, and a war that is declared

**Captured as run171 (item 660), pinned open at ~~617~~ ~~659~~ (item
664), and closed at **900 of 901** (item 668); its third change ended the
game.** The staging was read and committed first
(`chapter8.cmd`'s header, `89e338e`), the capture's section is
`docs/RUNS.md` run171, and no falsifier fired. What the chapter measured:

- **The row moves on the block after each line**: `diplos[1]` of who=0 and
  `diplos[0]` of who=1 read 1 on 701, 0 on 801 and 2 on 901, both sides
  written by the one call.
- **The peace stops the fight.** Both squads take their `ATTACKORDER` on
  635 and trade blows from 660 to 699; every attack order is gone by 731
  and no blow lands after 699. After `war 1` who=0's squad attacks again
  on 827 and chases without a blow by 900.
- **The General's aura shows in the blow**: a plain blow on a who=0
  hoplite is 2 hits, on a who=1 hoplite 3; the 7-point blows match. No
  dumped field carries the aura, which `UnitData::armor@00610160` computes
  on read. The General itself never moves and is never struck.
- **The Spy explores**: an `EXPLORETOORDER` on its birth block, 618, to
  (4344, 10488), group 66 — `!ai off` leaves a computer's Spy its
  `think_scout` (`docs/INPUT.md` §11.10's tail). It is the region scan,
  one `+0x941` and seventy `+0xaba` on 617, and since item 664 this
  crate draws them all (`docs/SCOUT.md` §15).
- **`ally` in a two-player lobby is an allied victory.**
  `Leader::set_diplo@006ec6a0` counts the live leaders allied to neither
  side over `leaders.list[0..8]` — the bound `0xe71af0` is eight
  `Leader`s past `leaders` — and gaia's leaders are 8 and 9, so the count
  is zero and `Leader::victory` runs. Both leaders carry `0x20 | 0x80` on
  901 and the game closes: the trace ends on 900. **The premise's third
  change is dead**: a fight across an alliance needs a third live player,
  which this lobby has not got. The staging read the loop as all ten
  leaders and predicted the game would run on.

~~**The word is 617**, the Spy's birth frame: 9 draws against 80, parting at
draw 3 on the original's `Unit::think_scout+0x941`.~~ **Item 664 moved it
617 → 659**: `ObjectData::is(SPY, 0)` had been a seam answering false, so
the Spy never reached `think_scout`. With the lineage test its birth
think spends the original's 71 draws, and on 618 it holds the original's
order, path, group and `form_mod` (`docs/SCOUT.md` §15).

~~**The word is 659**, the first blow's frame: 9 draws against 10, parting
at draw 2 on the original's `Unit::fight+0x9b0`, the one-in-five
re-search roll. The value diff, in run171's own coordinates: who=1's
hoplite `1/6` is chasing the General `0/9`, and on block 658 it stands at
(1780, 7844) on both sides. On 659 the original's has stopped there,
`collide_o 7` (who=0's hoplite `0/7`), with its move dropped and only its
attack order left. This crate's has stepped back to (1800, 7848) with
`collide 1` and still walks for the General. On 660 the original's
targets `0/7` and lands the first blow, and both leaders' `treaties[·]`
read 3 where this crate's read 0. No mechanism is named
(`docs/DECISIONS.md` 42); `GOLDEN_WORD_CHAPTER_EIGHT` carries the value
diff and `chapter_eight_s_word_frame_is_widened_whole` the blocks, now
605–660.~~ **Item 668 closed it, 659 → 900.** `1/6`'s stop was
`resolve_unit_collision`'s enemy ladder, arm C. A captain whose target is
out of range, bumped by an enemy that is not its target, drops its walk
and its attack. It then takes what it can strike from where it stands,
through `find_new_target(this, NULL, 1)`, while its leader's `retargets`
is under ten (`docs/COLLISION.md` §14). With it, block 659 is run171's,
and every draw agrees to the trace's last frame. The widening is run171
whole. Two rows stand on 660, and neither spends a draw: `0/7` takes 2
hits and `damage_frac` 5 from the first blow in run171 and 3 here, which
is the rally armor (parked 666). And both leaders' `treaties[·]` read 3
there and 0 here.

**Premise.** Chapter one's `war` is the bare form and changes nothing. This
is the chapter that moves the diplomacy state, three times, with a fight
running across each change: peace at 700, war at 800, ~~alliance at 900~~
alliance at 900, which ends a two-player game (above). And
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
aura is not modelled where this document assumes it is. ~~No field~~ The
aura writes no field by construction, so the falsifier was restated before
the run on the blow a who=0 hoplite takes inside six tiles of the General
(`chapter8.cmd`, check 4), and run171 answers it: 2 against 3.

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
| GroupAttackOrder | ~~`issue_attack` on a multi-unit group~~ **no issuer makes one** (`docs/ORDERS.md` §7.9): `CommandManager::issue_attack@009415e0` → `Group::action_attack@00712490` gives each member its own `AttackOrder`; only `copy_order` and the save loader build a `GroupAttackOrder` | 15, the group attack (§23), which measures the absence |
| GroupAttackToOrder | `issue_move_to` with `ATTACK_TO`, a ctrl+right-click on the ground (`WorldMap::on_right_up@008c7050:199`); also the AI's armies and the patrol's legs | 10 (the patrol's legs); 15, the group attack (§23) |
| MoveOrder, GroupMoveOrder | `CommandManager::issue_move_to@00941720` | 9, the first issuer chapter (§17; the lab validated the issuer) |
| ExploreToOrder, FleeToOrder | the same entry point, trailing selector: `issue_move_to` copies the `orders` byte, and `add_move_facing_order@005e55c0` picks the class at process time; a squad gets one a member and no group order | 16, explore and flee (§24) |
| PatrolOrder, GroupPatrolOrder | `CommandManager::issue_patrol@00941800`; `PatrolOrder` is never constructed alone (`docs/ORDERS.md` §7.7) | 10, the patrol line (§18) |
| AirPatrolOrder | **channel** (`bird`), and `CommandManager::issue_launch_patrol@00941860`; and a player's strike at a target it cannot see (`Unit::do_strafe@005eab00`) | 6; 17, the flight line (§25) |
| AirOrder | ~~`CommandManager::issue_flight@00941d40`~~ **no class of its own**: `AirOrder` is a base, printed as the `AIRORDER` block of a `STRAFEORDER` or an `AIRPATROLORDER` (`docs/ORDERS.md` §32) | 17, the flight line (§25), under both |
| AirAttackGroundOrder, AttackGroundOrder | `CommandManager::issue_attack_ground@009417a0`; AttackGroundOrder also auto, `Unit::fight`'s siege arm (`docs/COMBAT.md` §57) | 3 (the restage) |
| GuardOrder | `CommandManager::issue_guard@00941ed0`; also an army's escort (`docs/ORDERS.md` §24) | 4 (the AI's escort); 11, the guard line (§19) |
| FollowOrder | `CommandManager::issue_follow@00941e70` | 12, the follow line (§20) |
| FormOrder | **no issuer makes one**: `CommandManager::issue_form@00941580` → `Group::action_form@00707220` writes each member's `form` and lays out a group move; only `copy_order` and the save loader build a `FormOrder` | 14, the formation line (§22), which measures the absence |
| GarrisonOrder | `CommandManager::issue_garrison@00941a70`; the way out is `CommandManager::issue_eject_all@00941ca0` | 13, the garrison line (§21) |
| GatherOrder | auto (starting citizens), `CommandManager::issue_gather@00941a20` | 7 (the control) |
| BuildOrder | `CommandManager::issue_build@00941c30` → `Group::action_build@00707510` → `Group::action_swarm_around@0070fbe0`: a `MOVEORDER` for a human (an `EXPLORETOORDER` for a computer) and the `BuildOrder` behind it; the AI's own builders come through its planner | 18, the build line (§26) |
| TradeOrder | auto (a Caravan under AI), `CommandManager::issue_trade@00941960` | 7 (the control) |
| CastOrder | `CommandManager::issue_spell@00941b80` → `Group::action_spell@006fe1a0` → `Unit::add_cast_order@005e4a60`; also auto (`think_fish`, the unpacks, the transport; `think_spellcaster`'s Counterintelligence) | 19, the cast line (§27): the Informer on an enemy Barracks, `do_cast`'s targeted arm |
| RepairOrder | ~~the `repair` command type has no `CommandManager` issuer in the export~~ **a player's repair is `swarm_around`** (§29): `Console::execute_at_cursor@007c6630` and `Options::picked_spot@00721c40` → `CommandManager::issue_swarm_around@009416b0` with `REPAIR` → `Group::action_swarm_around@0070fbe0` → `Unit::add_repair_order@005e4ff0`, a `MOVEORDER` and the `RepairOrder` behind it; the `repair` command (`process_repair@00948cb0` → `Group::action_repair@007020c0`) is issued by nothing; also auto, a computer's (`do_gather`, `Build::process`) | 21, the repair line (§29) |
| BoardOrder, AwaitBoardOrder | ~~`CommandManager::issue_set_transport@00941910`; `board_ship` has no issuer~~ **no issuer makes one** (§28): `issue_set_transport@00941910` → `Group::action_set_transport@007024b0` is the auto-transport toggle and lays no order; the one adder of each, `Unit::add_board_order@005e4d10` and `Unit::add_await_board_order@005e4c80`, is reached only from `Group::action_board_ship@00700010` (the never-issued `board_ship`, and `finish_insert`'s replay) and `Unit::check_meet_ship@00604550` under a `BoardOrder`'s own step. What boards is the Transport `CastOrder` | 20, the board line (§28), which measures the absence |
| StrafeOrder | ~~no command type of its own; a mounted or air attack on the move~~ **`CommandManager::issue_flight@00941d40`** → `Group::action_flight@006fb260` → `Unit::add_strafe_order@005e48c0`: a flight home, `returning 1`, and a strike re-pointing one in flight; an unseen target turns it into an `AirPatrolOrder` over its point (`docs/ORDERS.md` §32) | 17, the flight line (§25) |
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

**Booked** (the thirteenth pass, DECISIONS 49): with every chapter above
closed, the rules track's chapters are issuer chapters from here, in this
table's order by what a failure would teach. **Chapter nine is the move
line** — item 676, `issue_move_to` on one unit and on a squad over land
with a world plan, the lab's L15 shape, the issuer under the emulator
before the pair; its section is §17, the worker's. The census's
order-family row (`docs/CENSUS.md`) is the counter, and it is expected to
move for the first time since it was built.

**Read again by the fifteenth pass (2026-09-25)**, with chapters fourteen
to nineteen closed and the order row at 60 cited: the table needs no
reorder. Of its three `unresolved` rows one has an issuer —
`issue_set_transport` (item 803, chapter twenty, §28) — and the other
two, `RepairOrder` and `SpecialAnimOrder`, have none, so after 803 they
are readings before they are chapters: what builds each, from the
export, and whether the DLL can reach it. `AirAttackGroundOrder`,
`AirPatrolOrder` and `TradeOrder` are the cited-zero rows a run enters
and no document names — the blind list's, not a chapter's.

## 14. The running order, and the run numbers

~~Reserved for this design: **run112–run119**.~~ The reservation was spent:
run116–run119 went to Great Lakes items on 2026-09-22, and the chapters
below without a run take their number at booking (the eleventh pass).

| run | chapter | window | why this order |
|---|---|---|---|
| 112 | two, the ranged line | `[605, 900)` | **run 2026-09-19, word 616**; the cheapest chapter that adds a record the tree has never dumped (`AMMO`) |
| ~~113~~ 127 | five, the water | `[605, 900)` | the likeliest to fail, 24 s to find out, and it gates any work on a Dock — **run 2026-09-22 as run127 (item 535), word 621; no falsifier fired** |
| ~~114~~ 132 | four, the border | ~~`[295, 345)`~~ `[295, 545)` | the namesake; `WORLD=6` narrow — **run 2026-09-23 as run132 (item 552), 830 MB, 2178 s; no falsifier fired** |
| ~~115~~ 133 | four, the bleed | `[595, 1500)` | the same script, a second window — **run133, 120 MB, 372 s; word 1277**, closed at 1500 by item 569 |
| ~~116~~ 141 | seven, the civilians | `[605, 1200)` | tests the premise every other chapter stands on — **run 2026-09-23 as run141 (item 578), 112 MB, 281 s; the first falsifier fired; closed at 1200** |
| ~~117~~ 142 | seven, the control | `[605, 1200)` | `!ai off` deleted; without it 116 measures nothing — **run142, 112 MB, 331 s; the five act exactly as in run141** |
| ~~118~~ 145 | three, the mounted and siege lines | `[605, 900)` | **run 2026-09-23 as run145 (item 587), 43 MB, 145 s; word 621; no falsifier fired, two could not** |
| 146 | three, restaged in two arenas | `[605, 1000)` | **run146 (item 587), 55 MB, 186 s; first parting 633; all three falsifiers reachable, none fired** |
| 156 | seven-b, the computer's civilians | `[605, 1200)` | the same five for who=1 with `!ai off`, where the cheat's block decides (item 628, the twelfth pass) — **run 2026-09-23 (item 628), 106 MB, 295 s; the first falsifier fired by construction; word 1148** |
| 157 | seven-b, the control | `[605, 1200)` | `!ai off` deleted; the AI's economy should reach the citizen, or the pair is vacuous — **run157, 114 MB, 335 s on the second take (the first stalled before the menu); the citizen gathers on 611 as in run156; word 1036** |
| ~~119~~ 168 | six, the air and the bird | `[605, 900)` | the one new order class — **run 2026-09-23 as run168 (item 648), 42 MB, 137 s; the second falsifier fired: an unbased aircraft is inert; word ~~616~~ ~~700~~ (item 650), closed at 900 (item 652; the cursor from run169's packet)** |
| 175 | six-b, the air line from a base | `[605, 1250)` | run168's second falsifier killed chapter six's premise; one Airbase a side, and both tanks run dry inside the window — **run 2026-09-23 (item 651), 112 MB, 315 s; the first falsifier fired on the target arm: each aircraft walks at the enemy Airbase; the base link is dead; word ~~632~~, closed at 1250 (item 680; a building target's re-search, from run177's packet)** |
| 180 | nine, the move line | `[605, 1100)` | the first issuer chapter (DECISIONS 49): two player orders through `issue_move_to` from the DLL, a lone Chariot and a squad of three Hoplites (§17) — **run 2026-09-24 (item 676), 66 MB, 209 s; no falsifier fired: both commands processed on the next frame, a plain move and three `GroupMoveOrder`s, all four arrive; the chariot's plan runs straight through the sand; word ~~693~~, closed at 1100 (item 676: a human's fog arm and `find_wpath` pop)** |
| 184 | ten, the patrol line | `[605, 1250)` | an issuer the AI never uses (parked 692): two player patrols through `issue_patrol` from the DLL, chapter nine's chariot and a squad east of the sand (§18) — **run 2026-09-24 (item 693), 85 MB, 243 s; no falsifier fired: one `GroupPatrolOrder` a unit, attack-move legs from the leader, the chariot turning on 761, 903, 1043, 1185 and the squad on 812, 985, 1155; word ~~640~~, closed at 1250 (item 693: the ground patrol, built)** |
| 190 | eleven, the guard line | `[605, 1250)` | an issuer the AI never uses from a command: a chariot guarding a wagon that walks, then an enemy in range, and a squad guarding a building, which the reading says gives no order (§19) — **run 2026-09-24 (item 696), 85 MB, 254 s; no falsifier fired: one `GUARDORDER` on the wagon at (0, 372), the post re-read as it walks, the guard's attack above its guard on 1011, and no order on the squad; then the guard drops its attack when the enemy walks off, never re-engages, and dies on 1141; word 724, then 734 (item 696: a supply wagon's land push, and an escort's soft row, from run191's brackets), then 1036 (item 703: a pushed unit's disc follows its figure, COLLISION §16); closed at 1250 (item 713: the danger flag is guy 0's, ANIM §13)** |
| 204 | twelve, the follow line | `[605, 1150)` | an issuer the AI never uses: a chariot following a supply wagon, and a squad following a chariot, each leader walking, stopping and turning (§20) — **run 2026-09-24 (item 714), 74 MB, 229 s; no falsifier fired: one `FOLLOWORDER` a unit, the chariot trailing in one-tile legs at the 1,728 threshold, the hoplites' first legs at d ≈ 790 while their leader walks (the doubling); the fifth could not fire for the squad; word ~~717~~, closed at 1150 (item 714: the follow, built)** |
| 208 | thirteen, the garrison line | `[605, 1000)` | an issuer the AI never uses from a command: a chariot and a squad garrisoning one Barracks, then the building's eject (§21) — **run 2026-09-24 (item 718), 72 MB, 184 s; no falsifier fired: one `GARRISONORDER` a unit under a plain leg, the chariot in on 699 and the squad whole on 761, the chariot out on 902 and the squad on 903; each walk ended short of its leg; word ~~640~~, closed at 1000 (item 718: the command and the Eject entered, the door on the review, the chain's kill, the exit's angles)** |
| 210 | fourteen, the formation line | `[605, 1150)` | an issuer the AI never uses: a three-squad group told Envelop standing and Line on the move through `issue_form` from the DLL, with `GROUPS=1` for the pool (§22) — **run 2026-09-25 (item 723), 97 MB, 284 s; the pool did not come out; falsifier 4 fired in its letter: the re-form on the spot is a plain `MOVEORDER` a member, not a `GroupMoveOrder`; no `FORMORDER` anywhere; the walking group halted and replayed in Line as read, arriving 905–926; word ~~631~~, closed at 1150 (item 723: the command entered, an equal group's record kept by `push_group`, the replay to `orig`)** |
| 215 | fifteen, the group attack | `[605, 1250)` | a player's attack on an enemy and an attack-move on the ground, `issue_attack` and `issue_move_to(ATTACK_TO)` through the DLL's `@attack` and `@amove`, with `GROUPS=1` at `GUYS=4` for the pool (§23) — **run 2026-09-25 (item 731), 269 MB, ~13 min; no falsifier fired: six `AttackOrder`s, `mandatory 1`, on 736 and no `GroupAttackOrder` anywhere; `1/6` last prints on 808; six `GroupAttackToOrder`s on 862, arriving on the predicted points from 1080; the pool printed; word ~~753~~, closed at 1250 (item 731: the attack command entered, a fresh slot's stamp)** |
| 219 | sixteen, explore and flee | `[605, 1250)` | the move issuer's trailing selector through the DLL's `@explore` and `@flee`, on a Chariot and a Hoplite squad, each explorer passing a goody box, with `GROUPS=1` at `GUYS=4` for the pool (§24) — **run 2026-09-25 (item 738), 263 MB, 769 s; no falsifier fired: one `EXPLORETOORDER` or `FLEETOORDER` a member and no group order; both explorers take a box leg (685, 804) and open the box (730, 855); the re-issue goes to the click; the pool printed** |
| 223 | seventeen, the flight line | `[605, 1400)` | `issue_flight` through the DLL's `@flight` and `@strike` on a Fighter and a Bomber pair from a staged Airbase, a strike from the ground first, with `GROUPS=1` at `GUYS=4` for the pool (§25) — **run 2026-09-25 (item 746), 325 MB, 942 s on the second take (the first stalled in DXVK's device setup); the pool printed; the strike on the ground took no order; three `STRAFEORDER`s home, `returning 1`, as read; falsifier 4 fired: the flying pair's strike became an `AIRPATROLORDER` over the unseen Barracks' point; the Fighter inside its base on 722; the Barracks bombed from 822 and destroyed on 1080; the pair still flying home at 1399** |
| 241 | eighteen, the build line | `[605, 1450)` | `issue_build` through the DLL's `@build` on a lone citizen (a Barracks) and a group of three (a Siege Factory), with `GROUPS=1` at `GUYS=4` for the pool (§26) — **run 2026-09-25 (item 779), 348 MB, 1,002 s; the pool printed; no falsifier fired: a `MOVEORDER` and a `BUILDORDER` (flags 4) a citizen on 622 and 642, both sites paid; built from 709 and 721, finished on 948 and 1141; `0/8` helps with a `MOVEORDER` on 1097; word ~~642~~, closed at 1450 (item 779: the build command entered, a human's approach a move)** |
| 245 | nineteen, the cast line | `[605, 1100)` | `issue_spell` through the DLL's `@spell` on a Spy of who=0: the Informer on a staged who=1 Barracks, with `GROUPS=1` at `GUYS=4` for the pool; the staging's predicates first read on run246's packet at 619 (§27) — **run 2026-09-25 (item 790), 201 MB, 540 s; the pool printed; no falsifier fired: a `CASTORDER` (flags 4, `paid` 1) and a `MOVEORDER` to (14232, 15528) on 622, `mana_burn` +500 and no bucket down; the Spy in range on 756, `spell_time` 1…39, the Barracks `infiltrated` on 795; the mana still from 756 to 795** |
| 249 | twenty, the board line | `[605, 1300)` | `issue_set_transport` through the DLL's `@settransport` on a Chariot beside a flagged one, both moved onto lake 70 behind a staged Dock, with `GROUPS=1` at `GUYS=4` for the pool; the staging's predicates first read on run250, to 646 (§28) — **run 2026-09-26 (item 803), 286 MB, 836 s; the pool printed; no falsifier fired: `0/7`'s bit off on 622 and on on 802, no order from either toggle; `0/6`'s Transport `CASTORDER` on 703 and barge `0/8` on 704; `0/7` stopped at the shore on 719; its cast on 829 and barge `0/9` on 830; `0/6` ashore on 1160; no `BOARDORDER` or `AWAITBOARDORDER` on any block** |
| 255 | twenty-one, the repair line | `[605, 1300)` | `issue_swarm_around` with `REPAIR` through the DLL's `@repair` on a lone citizen and a trio, at a who=0 Barracks who=1's Bowmen damaged before a peace, with `GROUPS=1` at `GUYS=4` and `AMMO=5`; the staging walked first on run256, to 830, four takes (§29) — **run 2026-09-26 (item 813), 296 MB, 884 s; the pool printed; no falsifier fired: `0/6`'s `MOVEORDER` and `REPAIRORDER` (flags 4) on 782, the trio's on 802 at three spots; the Barracks 3 → 0 on 930–931; the trio's orders dying on arrival on 968, 969 and 1023, each on this crate's predicted block** |
| 171 | eight, the commanders and a declared war | `[605, 1200)` | the diplomacy moved three times — **run 2026-09-23 (item 660), 54 MB, 168 s; no falsifier fired; `ally` ended the game on 900, so the capture is 605..901; word 617** |

~~Chapter eight and any further detail window need numbers beyond the
reservation.~~ Any further detail window takes its number at booking.
The order above is by **what a failure would teach**, not by
chapter number: 113 and 116 are placed early because each can invalidate work
that would otherwise be done on top of it.

## 15. Coverage — what a diff backs, and what rests on a reading

**Diff-backed**, by chapter one's own capture and the harness walk
(`crate::diff::golden`): the script format and frame clamping; `ai off`'s
three readers and its cost from frame 1; `add`'s unit arm, its count, its
three figures and its seating; the tile arm of `parse_coord`; the bare
diplomacy form's inertness; and the claim that two launches of one script
are one game at any detail. And by chapter seven's pair (item 578): that
`ai off` leaves a human's civilians exactly as it finds them, and that the
two scripts are one game through frame 0 and part on frame 1.

**Held by a test rather than a capture**: that every file in
`tools/gamelog/golden/` parses, that every line is on the right half of
`run_cmd`'s two switches, and that the verbs the interpreter will not act on
are exactly the ones `CHAPTER_DEBT` names — currently chapter one's bare
`war` ~~and chapter six's `bird`~~ (staged since item 652). It fails when a new chapter reaches for a
verb the harness drops, which is the failure mode this design is most likely
to produce.

**Reading alone, and a capture would settle each**: the entire content of §6
through §12 — every premise, every record list and every falsifier. The
addresses in §10 and §13 are read from the export and checked against its
index by `docs_guard`, which is not the same as a run entering them. The map
facts in §4 are a dump's own fields and are the strongest thing here short
of chapter one itself.

## 16. What is not established

- ~~**No chapter but the first has been run.** Everything in §6 through §12 is
  a design. The falsifiers exist so that the first run of each can say the
  design was wrong, and on the evidence of the last two months the honest
  expectation is that two or three of them will.~~ Chapters one to five
  and seven have run and closed (§14); one premise fell (§11) and one
  staging was restaged (§7). ~~Six and eight have not run~~ Six ran as
  run168 (item 648) and its premise fell (§10); ~~eight has not run~~ eight
  ran as run171 (item 660) and its third change ended the game (§12);
  seven-b ran as run156/run157 (item 628, §11).
- **Whether the cheat's block closes a computer civilian's tail**, and
  whether this crate's one-term stand-in does the same — §11's restage,
  chapter seven-b (item 628). The human pair could not measure it.
- **The rally armor's size.** run171's plain blow on a who=0 hoplite
  beside the General is one hit smaller than on who=1's, where rules.xml's
  `GENERAL_RALLY_ARMOR` reads 2 and `UnitData::armor` multiplies it by
  `get_general_upgrade + 1`. Which term makes it one is not read (§12).
- **A fight across an alliance.** `ally` ends a two-player Quick Battle
  (§12); a chapter that wants one needs a third live player in the lobby.
- **Whether `library <n>` moves the age as well as the epochs.** `age who=0 8`
  was measured to set the age and leave all four epochs Ancient; the inverse
  — that `library` carries both — is `docs/RUNS.md`'s note and not a
  measurement, and chapters two, six, seven and eight rest on it. The first
  of them to run settles it, and the check is one `LEADERDATA` line:
  `ages_get()` beside `epochs_get()`.
- ~~**Whether a cheat-placed building is finished or a site.** §8.~~
  Finished: run132's Temple (§8, item 552).
- **Whether `find_nearby_spot` filters by domain.** §9 — the whole of
  chapter five turns on it, and nothing on disk answers it.
- **Where `bird` lands.** §10. ~~`ConsoleWin`'s initial `mouse_coord_x/y` was
  not read~~ They are an uninitialised heap block's (item 648): the channel
  never writes them, and the case applies no `WorldData::restrict`. run168
  confirms the bird is born, on 700, but no dump prints owner 9. The
  prediction of zero, the world's corner, is unconfirmed: the bird's edge
  coins come on 750, 791 and 894, not in its first frames. ~~Which heap value
  it is, and whether it is the same on every launch, is not established.~~
  **(0, 6)**, read off run169's packet (item 652). The corner was right, and
  the three coins are the staged bird's own, after a loop out and back.
  **Still not established**: whether every launch leaves (0, 6). run168
  and run169 are two launches that did, the first by its walk and the
  second by its packet (parked 653).
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

## 17. Chapter nine — the move line, the first issuer chapter (item 676)

**Premise.** A player's move command, issued through the original's own
issuer, reaches the order family and is walked: a lone unit takes a plain
`MoveOrder` with the action bit and a world plan, a squad of three takes
three `GroupMoveOrder`s under one id and one leader, and both arrive. Every
order in chapters one to eight was one the original's own automatic play
issued (§13); these two are a player's. `tools/gamelog/golden/chapter9.cmd`
has the whole reading with its citations; this section is its summary.

**The channel's third half.** A line whose text starts with `@` is not
handed to `parse_cmd`. `rontrace.dll` (`tools/trace/tracer.c`,
`issue_line`) builds a `GroupOut` — `num` at `+0xc`, `who` at `+0x4a`, the
objects at `+0x8cc`, the three fields `CommandPackage::add_group@0094bb60`
reads — and calls `CommandManager::issue_move_to@00941720` with a plain
right-click's arguments: `QUEUE_NEW`, no angle, `MOVE_TO`, form and width
−1, no disembark (`WorldMap::on_right_up@008c7050:203`). It checks the
issuer's prologue, that `who` is the console's player, that every object is
a live captain of `who` with that id, and that the package has room, and
refuses by an `INFO` 17 record otherwise. The lab's probes (L15) passed
`orders 0, form 0, width 0`, which is not a click.

**The issuer, under the emulator first** (`tools/emu/callfn.py`'s machine,
item 676's scratch fixture: `command_oracle.py`'s widened to who=0's
objects 6–10). With the chapter's two calls, `issue_move_to` writes the
local `CommandPackage` and the four selection caches `CommandPackage::
last_who_sent`, `last_num_sent`, `last_objects_sent[0]` and
`last_uids_sent[0]`, and nothing else — no unit, no order, no draw
(`add_group`'s padding roll is under `semaphore & 4`, the network bit). Each
call appends 27 bytes: `group` num 1 who 0 [o] and the 22-byte `move_to`.
The squad's call names **one** object, its captain. A second call on the
same selection appends the three-byte `num 0` reuse, and a non-captain in
the list is dropped. **What the emulator cannot reach** is everything the
chapter measures, which happens at process time: `process_group@0094a0c0`
walking the captain's `o_down` chain into a `Group`, `Groups::push_group`,
`Group::action_move_to` and the plan. That needs the game.

**Lines.** `0 !ai off`; `610 add chariot who=0 16,36` — a Chariot is
`UBER_SIZE` 1, the lone unit, `0/6`; `612 add hoplite who=0 16,52` —
Hoplites are `UBER_SIZE` 3, captain `0/7` with `0/8` and `0/9` on its
`o_down` chain, as run105 threads chapter one's; `620 @move 0 12672 7296 6`,
the chariot from cell (4, 9) to cell (16, 9)'s centre; `640 @move 0 4992
16512 7`, the squad by its captain from cell (4, 13) to cell (6, 21)'s
centre. Ancient, one lever a line.

**The frame convention.** The call runs where a cheat runs, at `do_frame`'s
entry on trace frame F, but only appends: `TurnControl::do_frame_solo@
009556a0` calls `CommandManager::process_turn` before the next `do_frame`,
so the command is processed after logger block F+1 is written and before
tick F+1, and its order is first on block **F+2** — 622 and 642. The
harness runs an `@` line before the tick of F+1, ahead of any cheat staged
there ([`crate::golden::Script::apply`]), through
[`crate::input::group_move_to`], the `group` + `move_to` entry named against
`docs/COMMANDS.md` §3: `Group::add` each listed captain with its squad,
`push_group(…, 1)`, `group_action_move_to(…, action = 1)`.

**The capture must dump** `end:UNITS=3,GUYS=2,DEATHS=1,LEADERS=2` and
`misc:COMMANDMANAGER=1` over `[605, 1100)` — `COMMANDMANAGER=1` is where
`process_group` and `process_move_to` log the command they walk — beside
run105's `start:` set.

**The premise's killer, and its writers** (§3, point 5). It is
`CommandManager::check_accept_issue@00940a70` answering 0, which appends
nothing. It answers 1 in a solo game unless `use_mp_playback` is set —
written by `CommandManager::CommandManager@00943440` (0) and
`CommandManager::set_mp_playback@0093ee70`, a replay's — or the semaphore
has `0x10` (playback) or `4`, whose one writer is `Game::run_gamespy@
00587060`. The second is `process_group`'s player test, which hands a group
not of `info.player[package.play].who` to `is_team` and drops it; the DLL
issues only for the console's player. The third is `add_group`'s captain
filter, which the DLL's refusal 3 stands in front of. What decides plain
move against formation is `Group::action_move_near@00704990`'s per-member
test (`docs/ORDERS.md` §8.2), whose loop is bounded by `group.num`, at most
128 and here one and three.

**The ground.** A SANDY lake, region 65, lies at cells x 7–11, y 7–17
(run175's start `WORLD`), between the chariot and its point, so the
chariot's plan must bend round its north end; `find_wpath@00688fc0` plans
on the world grid for any start and goal three or more cells apart by
Manhattan, and these are 12 and 10. The squad walks the lake's west shore.
Neither walk enters a goody box's cell, the one way a walker opens one
(`Unit::set_new_location@005f8d20`, `docs/GOODY.md` §2).

**What this crate predicts, walked before the run** from run175's frame 0,
which every chapter shares to 599: the chariot a plain `MOVE_TO` with an
11-entry plan round the lake, on (12672, 7296) on block 990; the squad three
`GroupMoveOrder`s, one id, leader `0/7`, a 7-entry plan, the figures on
their slots on blocks 938–941.

**What would falsify it, and where each could first fire.**

1. **The issuer does not reach the pump**: an `INFO` 17 with a refusal on
   trace frame 620 or 640, or no `COMMANDMANAGER` group and move text
   between blocks 621 and 622 (641 and 642).
2. **The chariot's order is not a plain move with a plan**: block 622,
   `0/6`'s stack — anything but one `MOVEORDER` of kind 1 with the action
   bit, or a path of fewer than three entries.
3. **The squad's orders are not one formation**: block 642, `0/7`–`0/9` —
   anything but three kind-19 orders with one id and one leader.
4. **A walk does not arrive**: the block where a stack empties with the
   chariot more than a tile from (12672, 7296) or a figure more than two
   cells from (4992, 16512), or 1099 with either still walking.

Predicted: none fires. If one does, it closes the chapter on what it
measured and restages as a new one, as §11 and §10 did.

**Run 2026-09-24 as run180 (item 676): no falsifier fired.**

- **Both commands reach the pump.** `INFO` 17 on 620 and 640, refusal 0,
  the package 10 → 37 bytes (its 10 are the turn's `camera`). Between
  blocks 621 and 622 the dump prints `process_group, new 0 1 621`,
  `process_move_to 12672 7296 2 0 0 1 0` and `process_move_to_2 -1 -1`,
  and the same for the squad between 641 and 642. `process_group` sets
  `play 0` on each commanded unit.
- **The chariot**: one `MOVEORDER`, `flags 5`, an 11-entry plan, on 622.
  **The squad**: three `GroupMoveOrder`s, id 641000, leader `0/7`,
  `form_id` 0–2, on 642.
- **All four arrive.** The squad's stacks empty on 938, 939 and 941 on
  (5112, 16488), (4992, 16512) and (4872, 16536), exactly this crate's
  blocks and points. The chariot's empties on 1040 on (12672, 7296).
- **What the reading missed: the lake is no obstacle to the plan.** The
  original's plan runs straight along row 9 (`y 7320`), through the sand.
  A human leader's world search takes a cell whose four fog half-cells it
  has never seen as valid — `UnitData::invalid_loc@00607c30`'s `param_4`
  arm, which `valid_wcoord` enables past a node's second step — and on 620
  player 0 has seen none of the sand. The chariot walks east until the sand
  is in sight. On 693, a cell short of it, the straight line fails, it
  spends `do_move`'s grid draw, and it re-plans round the north end. That
  is the bend this crate took on 621.

**Where this crate parted: `GOLDEN_WORD_CHAPTER_NINE` = 693, open; closed at 1100 by the same item, below.** On 693
the original spends 7 draws against this crate's 6, parting at draw 0 on
`Unit::do_move+0xe84`, the chariot's re-plan. This crate's `invalid_loc`
carries the fog arm as a seam ("a no-op with no fog model"), so its plan
went round the lake from the start and there is nothing to refuse. The
value diff is `chapter_nine_s_word_frame_is_widened_whole` over (605, 696).
Past the first block's 26 standing rows — `form`, `filled_gather_slots`
and `build:extra`, chapter six's family — it pins:

- each staged unit's `form` on its birth block;
- the chariot's plan slots 2–10, `dest_y`, heading and position on 622
  (648–649 a waypoint on);
- the squad's `order:group.id` on 642, 641000 against 647600. The original's
  `group.id` is the pool slot, 0 here (the chariot's push took slot 1
  first); this crate's pushed group reads `64 +` its index in `Sim::pushed`.

The squad's walk, slots and plan agree to the word. The coverage driver
takes the word's five blocks, and they add one unread key,
`GroupMoveOrder/GROUPORDER/UNITORDER` `flags`, the attack-move's second copy.

**Closed at 1100 by the same item** (item 676; `docs/PATHFINDER.md`
§3's human variant, now built). Two human-only arms of the pathfinder were
missing, and the chariot's walk needs both:

- **`invalid_loc`'s fog arm.** With `fog_relax` set and a human leader, a
  tile whose cell's four fog half-cells are all unseen is valid before its
  terrain is read (`00607c9e`–`00607d4c`). `valid_wcoord` sets it past a
  node's second step, so the plan on 621 runs straight through the sand.
  With this arm alone every 622 row goes, and the word moves 693 → 694:
  the re-plan comes a frame late.
- **`find_wpath`'s human variant** (`00689109`–`006892e4`). A human's goal
  is **popped** for the entry under it while its cell's centre tile is in
  another region from the start's, or its centre half-cell is unseen,
  stopping on a final entry. Only a seen goal, or a sea unit's, goes on to
  the AI's pull-back walk. On 693 the waypoint is sand. The AI's walk had
  dragged it onto the shore and returned with the stack unchanged, which
  counts as a refusal. The human's pops past the sand and the unseen land
  beyond it to the order's own goal, and the search lays the 11-entry
  route round the north end.

With both, the walk agrees to run180's end: sequence 1100, no value part.
`chapter_nine_s_word_frame_is_widened_whole` spans run180 whole, 605 to
1099; 1100 is the `!quit` frame and prints no block. Past the first block's
26 standing rows it pins:

- the staged units' `form` on their birth blocks;
- the squad's `order:group.id` on 642, a pushed group's id, which is a value
  and no step reads it;
- the scout `1/0`'s non-scoring `order:move.facing` from 847 (parked 275),
  the row chapter five pins on 847.

Every other chapter and both long words hold. Two unit tests in `sim::path`
each fail with their arm switched off.

## 18. Chapter ten — the patrol line, an issuer the AI never uses (item 693)

**Premise.** A player's patrol command, issued through the original's own
issuer, gives each commanded unit a `GroupPatrolOrder` between where its
group stands and the click, and the group's leader walks it as alternating
attack-moves, turning at each end, with every member re-ordered at each
turn. Chapter nine's move orders were ones the AI already issues, so the
census's order row could not see them (parked 692); no AI class calls
`Group::action_patrol`, so this chapter's orders are the record's alone.
`tools/gamelog/golden/chapter10.cmd` has the reading with its citations.

**The issuer, under the emulator first** (item 693's scratch script on
`tools/explore/command_oracle.py`'s fixture, widened to who=0's objects
6–10 as live captains). `CommandManager::issue_patrol@00941800(group, x, y,
QUEUE_NEW)` appends **15 bytes** a call: the 5-byte `group` (num 1, who 0,
the one object) and a 10-byte `patrol`, type `0x0a`, `[to_x][to_y][queued
i8]` (`docs/COMMANDS.md` §3). It writes the package's size and data and the
selection caches `CommandPackage::last_who_sent` and `last_num_sent`
directly, and `last_objects_sent` and `last_uids_sent` through the imported
memcpy, and nothing else — no unit, no order, no draw. The same selection
again appends the 3-byte `num 0` reuse; `queued` rides through as passed;
`use_mp_playback`, `semaphore & 0x10` and `semaphore & 4` each append
nothing. **What the emulator cannot reach** is everything below:
`CommandPackage::process_patrol@00949380` → `Group::action_patrol@007030c0`
→ `Unit::add_patrol_order@005e4560`, and each leg's `Unit::do_patrol@
005f1910`. The DLL's `@patrol` verb is `@move`'s with that issuer and its
own prologue check (`sub esp, 0x10`); a patrol click passes QUEUE_NEW
(`WorldMap::on_right_up@008c7050:206`).

**The reading.**

- **The order.** `action_patrol` takes the group's location
  (`GroupData::get_loc@0070e030`: the leader's position, unless the group
  stands within `0x180` of its own order point) as the first point and the
  click as the second, each snapped to the 48-unit grid
  (`div_3_table[v >> 4] × 0x30 + 0x18`). Every member that is on the map,
  not a plane and not `UnitData::is_busy@0060a370` gets one
  `GroupPatrolOrder` (type 22) at QUEUE_NEW: the two points, `waypoint 0`,
  the action bit, `id = (group.id + frame × 10) × 100 + order_num`, the
  leader's object and who, and the member's index. **There is no plain
  `PatrolOrder`**: `add_patrol_order` asks for `GROUP_PATROL` alone
  (`docs/ORDERS.md` §7.7), and a lone unit's group is pushed too, because
  `process_group` forces `Groups::push_group@0070f9e0`, which then writes
  the unit's `group` (`+0x80`). The chariot is the leader of a group of one.
- **A leg.** With the patrol at the head, `do_patrol` on the **leader**
  (`order.leader == o && order.who == who`) steps `waypoint` modulo the
  point count and calls `Group::action_move_to(group, point, QUEUE_FIRST,
  ATTACK_TO)`. A patrol is a loop of attack-moves. A **follower** with the
  patrol at its head only idles (`set_anim(CHAR_DEFAULT)`).
- **A group's QUEUE_FIRST re-issues the patrol** (`docs/ORDERS.md` §8.2,
  this crate's `group_action_move_to` §17). `Group::set_up_insert@0070e520`
  copies the leader's action-bit orders aside, every member is halted, the
  leg is issued at QUEUE_NEW, and `Group::finish_insert@0070e620` re-issues
  the copies at QUEUE_LAST. Its case `0x16` is `Group::redo_patrol_order@
  00706d90`: every member that is on the map, not a plane and not type flag
  `0x8000000` gets a fresh `add_patrol_order` with the leader's two points,
  id and form index, and the leader's `waypoint` copied in. So after every
  turn each unit's stack is [the leg, the patrol]. For the chariot the leg
  is an `ATTACKTOORDER`, because a group of one takes
  `add_move_facing_order`. For the squad it is three `GroupAttackToOrder`s
  under the leg's own id and one leader.

**Lines.** `0 !ai off`; `610 add chariot who=0 16,36`, chapter nine's
chariot `0/6` on cell (4, 9); `612 add hoplite who=0 56,40`, the squad
`0/7`–`0/9` on cell (14, 10), east of the sand; `620 @patrol 0 3456 11136 6`,
to cell (4, 14)'s centre; `640 @patrol 0 11136 11904 7`, the squad by its
captain to cell (14, 15)'s centre. Ancient, one lever a line. The frame
convention is §17's: a call on trace frame F is processed before tick F+1,
so the patrol is first on block F+2, and a leader's first `do_patrol` runs
on tick F+1.

**The capture must dump** `end:UNITS=3,GUYS=2,DEATHS=1,LEADERS=2` and
`misc:COMMANDMANAGER=1` over `[605, 1250)`, beside run105's `start:` set.
The window holds two turns for each walk by speed alone (below).

**The premise's killer, and its writers** (§3, point 5).

- `check_accept_issue` and `process_group`'s player test are chapter
  nine's, with the same writers (§17).
- The new killer is **`action_patrol`'s early returns**:
  - `GroupData::buildings` (`+0x49`) set. Its one writer is
    `Group::add@00714350`, from the added object's `is_building`, and the
    DLL names only unit captains.
  - No leader, or `get_loc` answering 1.
  - A plane leader or any air member (`count(COUNT_DOMAIN, 4)`), which
    goes to `action_air_patrol` instead.
  - Per member, `is_busy`: a head cast order, or a unit entering or
    exiting. It skips that member.
- **The loops' bounds.** The member loop is bounded by `group.num`
  (`+0xc`, below `0x80` in `Group::add`), here 1 and 3. `do_patrol`'s step
  runs against `x_pos.length`, which `add_patrol_order` sets to exactly 2;
  `redo_patrol_order` extends it only past 2, and nothing here makes a
  third point.

**The ground.** Both walks are on BASELAND of region 1 (run180's start
`WORLD`): the chariot on column x 4, y 9–14, west of the sand strip of
region 65 (x 7–11); the squad on column x 14, y 10–15, east of it. No goody
box is on either line — the nearest, (16, 21), is five cells off the
squad's. The nearest animals, at cells (26, 12) and (27, 16), are twelve
cells east of the squad, and who=1 is ~35 cells off with its AI off.

**What would falsify it, and where each could first fire.**

1. **The issuer does not reach the pump**: an `INFO` 17 with a refusal on
   trace frame 620 or 640, or no `COMMANDMANAGER` `process_group` and
   `process_patrol` text between blocks 621 and 622 (641 and 642).
2. **The patrol is not one `GroupPatrolOrder` a unit between the group's
   place and the click**: block 622 for `0/6`, block 642 for `0/7`–`0/9`.
   It fires on any of these:
   - no type-22 order;
   - a `PATROLORDER` alone;
   - points other than the leader's snapped seat and the snapped click,
     (3480, 11160) for the chariot and (11160, 11928) for the squad;
   - for the squad, three orders that do not share one id and leader
     `0/7`.
3. **The legs are not the leader's attack-moves**: block 622 (642). It
   fires if `0/6` has no `ATTACKTOORDER` above its patrol with `waypoint
   1`, or if the squad has anything but three `GroupAttackToOrder`s under
   one id above its patrols.
4. **The patrol does not turn.** It fires on the block where the chariot's
   (the captain's) leg empties within a tile (two cells) of its point, if
   the next block holds no new leg to the other point with `waypoint`
   stepped. It also fires on block 1249 if either walk has turned fewer than
   twice. Predicted by speed alone — run180's chariot at ~29 and captain at
   ~23 internal units a frame, legs of ~4,100 and ~3,850 — the chariot
   turns near 770, 915, 1060 and 1205, the squad near 820, 1000 and 1180.

Predicted: none fires. **This crate cannot take the command yet**: it has
no ground patrol at all — no `GroupPatrolOrder`, no `do_patrol`, and
`finish_insert`'s patrol case is a named seam (`group_action_move_to`). If
the capture agrees with the reading, the first landing is the patrol
command's entry into the sim, built the way `crate::input::group_move_to`
was.

**Run 2026-09-24 as run184 (item 693): no falsifier fired.**

- **Both commands reach the pump.** `INFO` 17 on 620 and 640, refusal 0,
  the package 10 → 25 bytes. Between blocks 621 and 622 the dump prints
  `process_group, new 0 1 621` and `process_patrol 3456 11136 2`, and the
  same for the squad between 641 and 642.
- **One `GroupPatrolOrder` a unit, as read.** On 622 the chariot's has
  points (3192, 7032) and (3480, 11160), `waypoint 1`, id 621100 and
  `oxx 6`. On 642 the squad's three have id 641000, `oxx 7`, points
  (10872, 7800) and (11160, 11928), and **`form_id 0` on all three**:
  `redo_patrol_order` has already rebuilt them with the leader's index,
  on the leader's first `do_patrol`.
- **The legs.** On 622 an `ATTACKTOORDER` sits above the chariot's patrol,
  with `flags 1`: `do_patrol` passes action 0, so the leg carries no action
  bit. On 642 three `GroupAttackToOrder`s sit above the squad's patrols,
  id 641001, `form_id` 0–2.
- **The turns.** The chariot's leg empties on 761, 903, 1043 and 1185,
  exactly on its points. The captain's empties on 812, 985 and 1155. Each
  turn spends one block with the patrol alone at the head, and the next
  leg comes on the tick after. The squad's legs degrade to plain
  `ATTACKTOORDER`s ~14 blocks short of each point (`ungroup_move_order`),
  and `0/8` is still short of its slot when the captain turns: the halt
  drops its leg.

**Where this crate parted: `GOLDEN_WORD_CHAPTER_TEN` = 640, open; closed at 1250 by the same item, below.** The
harness skips both `@patrol` lines, because the command has no entry into
this simulation. On 640 this crate spends 37 draws against 36, parting at
draw 30: an extra `Guy::set_anim+0x97a < Guy::inc_time+0x271`, the idle
chariot's animation, where the original's chariot is walking.
`chapter_ten_s_word_frame_is_widened_whole` covers (605, 642). Past the
first block's 26 standing rows and the four births' `form`, it pins:

- the chariot's missing patrol on 622 (`orders.len` 0 against 2, no
  `group`, no path, `orders_x/y`, `dest_angle`);
- its standing still from 623;
- the squad's missing patrols on 642.

The coverage driver takes 638..644. It pins `GroupPatrolOrder`'s
`PATROLORDER` keys and the `process_patrol` text as unread, owed by this
item's build.

**Closed at 1250 by the same item** (item 693; `docs/ORDERS.md` §27, the
ground patrol, built). The command's entry is `rondata::input::group_patrol`,
built the way `group_move_to` was: the group, forced into the pool, then
`Sim::group_action_patrol`. Then the order itself:

- **`Body::Patrol`**, the `GroupPatrolOrder` with its two snapped points,
  added by `add_patrol_order`;
- **`do_patrol`**: the leader steps and issues the attack-move leg at
  `QUEUE_FIRST`, and a follower idles;
- **`redo_patrol_order`**, `finish_insert`'s case `0x16`, which had been a
  named seam in `group_action_move_to`: every member's patrol is rebuilt
  with the leader's step and `form_id`.

With these the walk agrees to run184's end: sequence 1250, no value part,
the chariot's four turns and the squad's three block for block.
`chapter_ten_s_word_frame_is_widened_whole` spans run184 whole, 605 to
1249; 1250 is the `!quit` frame and prints no block. Past the first
block's 26 standing rows it pins:

- the staged units' `form` on their birth blocks;
- the patrol ids on 622 and 642 and the squad's first-leg
  `GroupAttackToOrder` id on 642. These are a pushed group's id (parked
  689): this crate's `64 +` index against the original's pool slot. It is
  a value no step reads, declared non-scoring as chapter nine's is.
- the scout `1/0`'s `facing` from 847 (parked 275).

Two unit tests in `sim::group` pin the order and the rebuild. The second
fails with `redo_patrol_order` switched off, and again with the member's
own index in place of the leader's `form_id`. The coverage driver reads
`PATROLORDER`'s arrays and `waypoint` now; each array's `size`,
`increment` and `flags` stay pinned as unread.

## 19. Chapter eleven — the guard line, an issuer the AI never uses (item 696)

**Premise.** A player's guard command, issued through the original's own
issuer on a **unit**, gives the guard one `GuardOrder` on its charge at an
escort slot's offset. The guard takes that post, keeps it while the charge
walks, and engages an enemy that comes into its respond radius while the
charge stands, with the attack stacked above the guard. The AI issues
`Group::action_guard` only for an army's wagon escort (`docs/ORDERS.md`
§24), never from a command. So chapter four's escort is the only
`GUARDORDER` on disk, and no dump prints `process_guard`.
`tools/gamelog/golden/chapter11.cmd` has the reading with its citations.

**The booked second half does not stand.** The booking asked for a squad
guarding a building, and read it as entering `GuardOrder`. The reading
says it enters nothing. `action_guard@006fcd30`'s first test on the charge
is object vslot `+0x8` (`call *0x8(%eax)` at `006fce46`), then `+0xbc`.
The PDB's `SubObjectData` method list names `+0x8` **`is_valid_unit`**:
the export's folded name, `SubObjectData::is_active`, is the body's.
`Build::vftable@00b42174` has `Window::get_button`, a folded `return 0`,
there, so a building charge returns before any order. The emulator
confirms it (below). The half is kept, staged as booked, as a test of
that reading: its prediction is **no order**.

**The issuer, under the emulator first.** Item 696's scratch scripts run
on `tools/explore/command_oracle.py`'s fixture, widened to who=0's objects
6–10 as live captains.

- **`CommandManager::issue_guard@00941ed0(group, ox, whom, QUEUE_NEW)`
  appends 18 bytes.** That is the 5-byte `group` (num 1, who 0, the one
  object) and a 13-byte `guard`: type `0x1f`, then `[ox i32][whom
  i32][queued i32]` (`docs/COMMANDS.md` §3).
- **It writes** the package's size and data and the four selection caches
  (`CommandPackage::last_who_sent`, `last_num_sent`, `last_objects_sent`
  and `last_uids_sent`), and nothing else: no unit, no order, no draw.
  The same selection again appends the 3-byte reuse; `queued` rides
  through as passed; `use_mp_playback`, `semaphore & 0x10` and
  `semaphore & 4` each append nothing.
- **`Group::action_guard` entered on a building charge** (`Build::vftable`
  at `0/2001`, the group `[0/8]`) runs `Group::clear`,
  `Group::action_begin@00714100` and the folded `return 0`, then
  returns. Its one write is the group's `disband`. A unit charge
  (`Unit::vftable`) passes to `is_on_map`.
- **What the emulator cannot reach**: the unit half's process time.
  That is `CommandPackage::process_guard@009478a0` →
  `action_guard(g, ox, whom, queued, 0)` → the escort and
  `Group::compute_form@00707c80` → `Unit::add_guard_order@005e3e40`, and
  each frame's `Unit::do_guard@005e5c70`.

The DLL's `@guard <who> <ox> <whom> <o>…` is `@patrol`'s with this
issuer: the same prologue (`sub esp, 0x10`), and QUEUE_NEW, which is
what `Options::picked_spot@00721c40` passes through
`GroupOut::issue_guard@007088e0` for an unmodified pick.

**The reading, for a unit charge** (`docs/ORDERS.md` §7.5 and §24.3–24.5
have it with its citations; this crate built it for the AI's escort).

- **The order.** `action_guard` replaces the charge by its captain,
  builds the escort, lays it out with `compute_form` at the charge
  (formation `leader_flags >> 2 & 1`, 1 for a human, width `0x32`, the
  guard flag), and gives each member `add_guard_order(charge, whom, dx,
  dy, queued)`, `dx/dy` its slot's offset. The command passes siege
  filter 0.
- **The post, each frame** (`do_guard`). The post is the charge's
  position plus `(dx, dy)` rotated by its heading, snapped to the 48-unit
  cell. Off that cell, a transit `ATTACKTOORDER` goes on at QUEUE_FIRST
  with `timer 0x1e × max(1, cells)`, and `do_move` runs the same frame.
  If that leg ends at once, `retry = Random::get % 3 + 6`, the one draw.
- **While the charge moves** (`is_moving`), both periodic arms are
  skipped, and the post is re-read each frame the `retry` allows.
- **While it stands**: on `(o + frame + 8) % 16 == 0` the guard calls
  `Unit::find_melee_target@005ff9c0(−1, 0, 0, 1, 0)`. That is the
  ordinary respond radius, `UNIT_RESPOND_RANGE` × `0xc0` for a human's
  melee type, not `UNIT_GUARD_RESPOND_RANGE`. Its issue argument 1
  stacks the attack at QUEUE_FIRST above a `GUARD`. On
  `(o + frame) % 16 == 0` it counts `idle`.

**Lines.**

- `0 !ai off`.
- `610 add chariot who=0 16,36`: the guard, `0/6`, on cell (4, 9).
- `612 add supply who=0 16,44`: the charge, a Supply Wagon, `UBER_SIZE`
  1 and no attack, `0/7`, on cell (4, 11).
- `614 add hoplite who=0 36,152`: the squad `0/8`–`0/10`, on cell (9, 38),
  four cells east of who=0's `0/2001` (orig type 418, at (4224, 28608)).
- `620 @guard 0 7 0 6`: the chariot guards the wagon.
- `640 @guard 0 2001 0 8`: the squad, by its captain, guards the
  building.
- `720 @move 0 3456 11904 7`: the charge walks to cell (4, 15)'s centre,
  down the baseland column x 4, as in chapter ten.
- `1000 add chariot who=1 17,70`: the enemy, who=1's next unit, `1/6`.
  It is about 1,300 units south of the guard's second post, inside both
  chariots' sight (`LOS` 9 tiles) and the guard's respond radius.

A call on trace frame F is on block F+2, and the guard's first
`do_guard` runs on tick F+1 (§17's convention).

**The capture must dump** `end:UNITS=3,GUYS=2,DEATHS=1,LEADERS=2` and
`misc:COMMANDMANAGER=1` over `[605, 1250)`, beside run105's `start:` set.
The `GUARDORDER` record prints `ox`, `whom`, `uid`, `dx`, `dy`,
`guard_x`, `guard_y`, `idle` and `retry` (run133).

**The premise's killer, and its writers** (§3, point 5).

- `check_accept_issue` and `process_group`'s player test are chapter
  nine's, with the same writers (§17).
- **`action_guard`'s early returns**, each ahead of any order:
  - `GroupData::buildings` (`+0x49`), whose one writer is
    `Group::add@00714350`; the DLL names only unit captains.
  - The charge failing `is_valid_unit` or `is_on_map`. For the building
    half this fires by construction. For the wagon it cannot: a cheat's
    unit is on the map.
  - `LeaderData::is_ally@006edb50(who, whom)`: 0 and 0.
  - "Invalid order" (`ox < 2000` past `whom`'s unit count), an
    `Error::report` that would stop the game. `0/7` is inside the count.
  - An empty escort (`get_num_cap`). The chariot is active, on the
    map, no plane, and of the wagon's domain.
- **Nothing sweeps or breaks a cycle here.** The human sweep adds units
  already guarding the charge (`update_guard_order@005e3220`), and at 620
  there are none. The cycle break clears a charge that guards a member,
  and the wagon has no order.
- **The loops' bounds.** The member loop and the slot loop are bounded
  by `group.num` (`+0xc`, below `0x80` in `Group::add`), here 1. The
  sweep is bounded by who=0's unit count (`objects +0x15c`). `do_guard`
  has no loop but `find_nearby_spot`'s, reached only on an invalid post.
- **`do_guard`'s own kill**: a charge whose object slot is freed. The
  enemy's target is the dump's to say. If it is the wagon (90 hits), the
  guard's order ends with it, and that is a finding, not a falsifier.

**The ground.** All of the unit half is on BASELAND of region 1, on the
column x 3–5, y 9–18 (run184's start `WORLD`). That is west of chapter
nine's sand, region 65, at x 7–11. The squad's cell (9, 38) is baseland
with no feature. No goody box and no animal is within five cells of
either.

**What this crate predicts, walked before the run** from run184's frame
0 (chapters share frames to 609):

- **The guard.** On 622, `0/6` has a `GUARD` on the wagon with offset
  (0, 372) under a transit leg. It is on its post, (3528, 8760), by
  ~660.
- **The walk.** The wagon's `MOVE_TO` is first on 722, and it arrives on
  (3456, 11904) at ~915. The chariot re-posts on the way and stands on
  (3480, 12264) from ~955.
- **The fight.** `1/6` fires on the guard from 1004. The guard's
  `ATTACK` on `1/6` sits above its `GUARD` from block 1011, its phase
  tick 1010.
- **The squad** holds nothing to 1249.

**What would falsify it, and where each could first fire.**

1. **The issuer does not reach the pump.** Trace frames 620, 640 and
   720: an `INFO` 17 with a refusal. Or: no `COMMANDMANAGER`
   `process_group` and guard text between blocks 621 and 622 (641 and
   642), or no move text between 721 and 722.
2. **The guard is not one `GuardOrder` on its charge.** Block 622,
   `0/6`: no type-12 order; a `whom`/`ox` other than 0/7; or the action
   bit clear. The offset is a value row, and it is not a falsifier:
   (0, 372) is this crate's `compute_form`, never measured for a
   one-unit escort.
3. **The guard does not keep its post on a moving charge.** On the block
   the wagon's stack empties (this crate: ~915), and on block 1000, it
   fires if either holds:
   - `0/6` no longer holds its `GUARD` on `0/7`;
   - its `guard_x/guard_y` is more than one 48-unit cell from the wagon's
     position plus the offset rotated by the wagon's heading, snapped to
     its cell.

   On 1000 it also fires if `0/6` is not on its post's cell.
4. **The guard does not engage.** `1/6` is first on block 1001. It fires
   if no `ATTACKORDER` on `1/6` sits above `0/6`'s `GUARDORDER` by block
   1027 (the guard's phase ticks are 1010 and 1026), or if the `GUARD`
   is gone from under it.
5. **The building half gives an order.** Block 642 or any block to 1249:
   any order on `0/8`–`0/10`.

Predicted: none fires. **This crate can take the command.** The harness
reads `@guard` through `rondata::input::group_guard`, which pushes the
group as `process_group` forces. Then it runs
`sim::Sim::group_action_guard` with the siege filter off for a unit
charge, and does nothing for a charge that is no unit, which is the
`is_valid_unit` exit. The AI's escort built the rest (item 567). Its
named seams are the human sweep, the QUEUE_FIRST insert and the idle-arm
engagement's own `find_melee_target` arguments; this staging reaches only
the last.

**Run 2026-09-24 as run190 (item 696): no falsifier fired.**

- **All three commands reach the pump.** `INFO` 17 on 620, 640 and 720,
  refusal 0, the package 10 → 28, 28 and 37 bytes. The dump prints
  `process_group` and `process_guard <frame>` between blocks 621/622 and
  641/642, and the move between 721/722.
- **The guard, block 622**: one `GUARDORDER` on `0/7`, `flags 4`, offset
  **(0, 372)**, post (3528, 8760), under a transit `ATTACKTOORDER`
  (`timer 59`). This crate's value to the digit, and `idle 24` on 700 too.
- **The walk.** The wagon's stack empties on 857, 58 blocks before this
  crate's walk predicted. The guard's post steps behind it, one cell short
  on 856, and it stands on (3480, 12264) from 890.
- **The engagement.** An `ATTACKORDER` on `1/6` is above the `GUARD` on
  **1011**, the phase tick 1010, as predicted.
- **The building half.** No order on `0/8`–`0/10` on any block: the
  `is_valid_unit` exit, measured.
- **What the reading did not say: the fight.** `1/6` walks off on an army
  `ATTACKTOORDER` on 1021, and the guard's `ATTACK` goes on 1037. `1/6`
  comes back and shoots it from ~1,640 units, and the guard never
  re-engages. It lands one hit, takes three, and is gone from 1141, and
  the wagon stands unguarded.

**Where this crate parted: `GOLDEN_WORD_CHAPTER_ELEVEN` = 724, then 734
by the same item, then 1036 (item 703), then 1133 (item 707), then 1139
(item 709), then 1250 (item 713), closed.** On 724 this crate spends 14 draws against 11,
three extra `Guy::set_anim+0x97a < Unit::move_step+0x823`. The value parts
on **722**: on tick 721, the first after the wagon's move, the original's
guard steps (21, 1) off its post and names the wagon in `collide_o`, with
no order added and no draw. `chapter_eleven_s_word_frame_is_widened_whole`
pins it.

**run191, three 17 s takes with the new `RON_GUARD_PROBE` brackets**
(`do_guard`, `do_move`, `move_step`, `resolve_unit_collision`,
`detect_unit_collision`, `detect_boat_collision`), each the same game as
run190 frame for frame, named the two mechanisms:

- **The wagon pushes its guard.** `set_new_location(0/6)` on tick 721 is
  nested in the *wagon's* `detect_unit_collision(…, boats 1)`: a supply
  wagon takes `detect_boat_collision`'s push as a ship does. This crate
  carried only the sea half (`docs/COLLISION.md` §13.5). Built → **726**.
- **An escort never blocks its charge.** On tick 726 both sides' push
  refuses the walking guard and falls to the land scan. The original's
  scan reaches `is_here` on the guard and never `is_corner`: §4.3's row
  "its action is `GUARD` on me" is soft, and this crate had no such row.
  Built → **734**.

**734 → 1036 (item 703), open.** On tick 733 the original's guard is
refused its own step and then pushed by the wagon. The collision disc
follows guy 0, not the unit (`docs/COLLISION.md` §16): the guard's
figures, and so its bits, still stood on the cell tick 732's push left
it. One of those bits is the wagon's by `is_here`, so the hit is hard.
The same push leaves the chariot's trackless crew on its destination and
guy 0 off it, so the blocked stand on 734 rolls one idle. The widening
needed no probe take. Every row from 736 to 1036 then agrees.

**On 1036** ours spends `Unit::fight+0x9b0` where the original spends
only the birds. On block 1037 the original's guard holds its `GUARD`
alone, with the attack on `1/6` gone and `recharging 0`. This crate's
keeps the `ATTACK` and fires. `1/6` is about 1,810 units off. No
mechanism is named.

**1036 → 1133 (item 707), open: a guard's attack is leashed to its
post** (`docs/COMBAT.md` §63).
- **The arm.** When a unit's activity is a `GUARD`, `fight` asks
  `check_target` with its guarding argument once `valid_target` passes.
  That refuses a target further than `UNIT_GUARD_RESPOND_RANGE × 2 × 0x60`
  = 1,536 from the post.
- **Why 1036.** The reload gate sits above the arm, so it is first asked
  on tick 1036. `1/6` is ≈1,780 off by then.
- **The search.** The guard's own search is leashed too, so it adds
  nothing and writes `near` −1. That is the widening's new `near` row, the
  only record that said a search ran.
- **Parked 705 closes with it.** The same leash keeps the guard's
  sixteen-frame search from re-engaging `1/6` at ≈1,635.
- **The retaliation.** A human's `GUARD` also holds its retaliation
  (`target_opportunity`'s action gate), which closed a transient on 1091.

Every row from 736 to 1133 agrees.

**On 1133** ours spent `Unit::fight+0x9b0` where the original spends
only the farms: on block 1134 the original's `1/6` had dropped its
`ATTACK` on the guard, drawlessly.

**1133 → 1139 (item 709), open: the hundredth-frame resync forgets**
(`docs/VISION.md` §10). `update_all_seen` on 1133 clears `seen`, and the
guard's `visible` byte, 0 since 1051, no longer relights its cell for
who=1, so `valid_target` fails. A building target is seen through its
`ever_seen` byte, which keeps Great Lakes from falling. Every row from
736 to 1141 agrees.

**1139 → 1250 (item 713), closed: the danger flag is guy 0's**
(`docs/ANIM.md` §13). On 1139 the original's guard re-rolls its stand
(`Unit::do_guard+0x7f4`). `set_in_danger` flags figures `0 .. guy_mark`
alone, and this crate flagged the crew too. So on tick 1100 its crew
took `IDLE1` where the original's took `IDLE2`, whose 71 frames ran out
for 1139. Nothing parts on any frame of run190.

## 20. Chapter twelve — the follow line, an issuer the AI never uses (item 714)

**Premise.** A player's follow command, issued through the original's own
issuer, gives each commanded unit one `FollowOrder` on its leader. The
follower stands while it is within its standoff, trails a walking leader
by `MOVE_TO` legs to the standoff point, comes to rest when the leader
stops, and trails again when the leader turns. No AI class reaches
`Group::action_follow@006fd510`, so no dump on disk holds a `FOLLOWORDER`.
`tools/gamelog/golden/chapter12.cmd` has the reading with its citations.

**The issuer, under the emulator first.** Item 714's scratch script runs
on `tools/explore/command_oracle.py`'s fixture, widened to who=0's objects
6–10 as live captains.

- **`CommandManager::issue_follow@00941e70(group, ox, whom, QUEUE_NEW)`
  appends 18 bytes.** That is the 5-byte `group` (num 1, who 0, the one
  object) and a 13-byte `follow`: type `0x1e`, then `[ox i32][whom
  i32][queued i32]` (`docs/COMMANDS.md` §3). Two captains make a 7-byte
  group.
- **It writes** the package's size and data and the four selection caches,
  `CommandPackage::last_who_sent` and `last_num_sent` directly and
  `last_objects_sent` and `last_uids_sent` through the imported memcpy.
  Nothing else: no unit, no order, no draw. The same selection again
  appends the 3-byte reuse; `queued` rides through as passed;
  `use_mp_playback`, `semaphore & 0x10` and `semaphore & 4` each append
  nothing.
- **What the emulator cannot reach**: `CommandPackage::process_follow@
  009479c0` → `action_follow(g, ox, whom, queued)` →
  `Unit::add_follow_order@005e3f60`, and each frame's `Unit::do_follow@
  005e65d0`.

The DLL's `@follow <who> <ox> <whom> <o>…` is `@guard`'s with this issuer:
the same prologue (`sub esp, 0x10`), and QUEUE_NEW, which is what
`Options::picked_spot@00721c40` passes through `GroupOut::issue_follow@
00708980` for an unmodified pick of a map unit.

**The reading** (the listing, `005e65d0`–`005e6b7c`; `docs/ORDERS.md` §7.6
has the older summary).

- **The order.** `action_follow` gives every member that passes
  `is_valid_unit`, `is_on_map` and not `is_plane`, and is not the leader's
  own `get_captain` of the same player, one `add_follow_order(ox, whom,
  queued)`. That is a FOLLOW (type 11) with `ox/whom/uid` the leader and
  the action bit, and `oxx/whose/uid2` the leader again unless it is
  inside a container. The dump prints `flags`, `ox`, `whom` and `uid`.
- **The standoff, each frame.** `d = vector_dist` from the follower to the
  leader. From the follower's `los` (`UnitData::los@006100c0`, the unit's
  `+0x3c`, `mylos` in the dump):
  - `k = los × 0x60` when the follower is the faster
    (`UnitData::speed@0060aae0`), else `los × 0x300 / 5`;
  - `k` doubles while the leader `is_moving`;
  - `s = clamp(los × 0x180 − k, 0x180, 0x600)`.
- **Standing.** `d ≤ s + 0xc0`: `set_anim(CHAR_DEFAULT, 0, 1)`, nothing
  else.
- **Trailing.** Farther, the target point is `s` from the leader toward
  the follower (`find_angle`, `project`). `find_nearby_spot` from it, with
  `FILTER_NOT_ME`; failing that the point `s` behind the leader's heading;
  then a ring `s..s + 0xc0` round the leader; then the leader's own place.
  A `MOVE_TO` leg goes on at QUEUE_FIRST without the action bit, facing the
  leader's heading, and `do_move` runs the same frame. The leg has no timer,
  so it walks to its end before the FOLLOW is read again.

**The cast's numbers** (run190's dump: a chariot's `mylos` is 9 at about 29
units a frame, a supply wagon's 4 at 25, a hoplite's 6 at 23).
- **Pair A**, a Chariot after a Supply Wagon. It is the faster, so `s` is
  `0x600` whether the wagon stands or walks (clamped). It trails at 1,536
  and moves once `d` passes 1,728.
- **Pair B**, three Hoplites after a Chariot. They are the slower, so `s`
  is 1,383 (threshold 1,575) while the chariot stands and **461**
  (threshold 653) while it walks. That is the doubling, and this pair is
  staged to see it.

**Lines.**

- `0 !ai off`.
- `610 add chariot who=0 16,36`: `0/6`, on cell (4, 9).
- `612 add supply who=0 16,44`: its leader, `0/7`, on cell (4, 11), 1,536
  away.
- `614 add hoplite who=0 60,56`: the squad `0/8`–`0/10`, on cell (15, 14).
- `616 add chariot who=0 60,60`: its leader, `0/11`, on cell (15, 15), one
  cell south.
- `620 @follow 0 7 0 6` and `640 @follow 0 11 0 8`.
- `700 @move 0 3456 13440 7`: the wagon south to cell (4, 17)'s centre.
- `720 @move 0 11904 14976 11`: the chariot south to cell (15, 19)'s.
- `880 @move 0 14976 14976 11`: the chariot turns east, to cell (19, 19).
- `960 @move 0 1152 13440 7`: the wagon turns west, to cell (1, 17).

A call on trace frame F is on block F+2 (§17's convention).

**The capture must dump** `end:UNITS=3,GUYS=2,DEATHS=1,LEADERS=2` and
`misc:COMMANDMANAGER=1` over `[605, 1150)`, beside run105's `start:` set.

**The premise's killer, and its writers** (§3, point 5).

- `check_accept_issue` and `process_group`'s player test are chapter
  nine's, with the same writers (§17).
- **`action_follow`'s early returns**, each ahead of any order:
  - `GroupData::buildings` (`+0x49`), whose one writer is
    `Group::add@00714350`; the DLL names only unit captains.
  - `ox` or `whom` negative.
  - The leader failing `is_valid_unit` or `is_on_map`, or `is_plane`. A
    cheat's wagon and chariot pass all three.
  - Per member, the same three, and the member being the leader's captain
    with `who == whom`. `0/6` against `0/7` and `0/8`–`0/10` against
    `0/11` are not. **There is no `is_ally` test**: a player may follow an
    enemy.
- **`do_follow`'s kill.** It kills the order when the leader fails
  `UnitData::is_seen@00607a60` by the follower's player. That answers 1
  for any unit of that player (its stealth block ends in `is_detected`,
  which answers 1 for the owner, and the fog test is skipped), so it
  cannot fire here. It also kills when the leader is neither valid on the
  map nor inside a container.
- **The loops' bounds.** The member loop is bounded by `group.num`
  (`+0xc`, below `0x80` in `Group::add`), here 1 and 3. The scenario sweep
  ahead of it runs only under `ScenarioData::ignore_orders`, whose writers
  are all `ScenarioFuncSet`'s. `do_follow` has no loop but
  `find_nearby_spot`'s.

**The ground** (run190's start `WORLD`). Pair A walks BASELAND on column
x 4, y 9–17, then row y 17, x 1–4. Pair B walks column x 15, y 14–19,
then row y 19, x 15–19. Chapter nine's sand is x 7–11. The nearest goody
boxes, (1, 19) and (16, 21), are two cells off each line.

**What would falsify it, and where each could first fire.** Predicted by
speed alone: the wagon's stack empties near 897 and again near 1054, the
chariot's near 828 and 988.

1. **The issuer does not reach the pump.** Trace frames 620 and 640: an
   `INFO` 17 with a refusal. Or no `COMMANDMANAGER` `process_group` and
   follow text between blocks 621 and 622 (641 and 642), or no move text
   for 700, 720, 880 and 960 between F+1 and F+2.
2. **Not one `FollowOrder` a unit on its leader.** Block 622 for `0/6`,
   block 642 for `0/8`–`0/10`: no type-11 order; an `ox/whom` other than
   7/0 (11/0); a `uid` other than the leader's; or the action bit clear.
3. **A follower within its standoff does not stand.** Blocks 622–701 for
   `0/6` and 642–721 for the squad: a leg above the FOLLOW, or a step.
4. **A follower does not trail a walking leader.**
   - `0/6`: no `MOVEORDER` leg above its FOLLOW by block 720 (the wagon
     passes 1,728 near 712), or a leg with no FOLLOW under it.
   - The squad: no leg by block 727. With the doubling the legs come as
     soon as the chariot walks, since `d` ≈ 768 > 653. Without it they come
     no sooner than `d` > 1,575, near 750.
5. **A follower does not come to rest.** Sixty blocks after its leader's
   stack empties, the follower has a leg, or stands farther than its
   standing threshold (1,728 and 1,575) from its leader. Or, on any block
   to 1149, a follower's FOLLOW is gone.
6. **A follower does not trail again after the turn.** No leg on `0/6`
   after block 962, or on the squad after 882. Or, on block 1149, `0/6`
   farther than 1,728 from the wagon, or a hoplite farther than 1,575 from
   `0/11`.

Predicted: none fires. **This crate cannot take the command yet**: it has
no FOLLOW, and the harness skips both `@follow` lines (a named seam in
`crate::golden`). If the capture agrees with the reading, the first
landing is the follow command's entry into the sim, built the way
`crate::input::group_patrol` was.

**Run 2026-09-24 as run204 (item 714): no falsifier fired; the fifth
could not for the squad**, whose chariot turned on 880, before its sixty
blocks were up.

- **Both follows reach the pump.** `INFO` 17 on 620 and 640, refusal 0,
  the package 10 → 28 bytes. The dump prints `process_group` and
  `process_follow 621` (641) between blocks 621/622 (641/642).
- **One `FOLLOWORDER` a unit, as read.** On 622 `0/6`'s has `flags 4`,
  `ox 7 whom 0 uid 14`. On 642 each of `0/8`–`0/10` has `flags 4`, `ox 11
  whom 0 uid 18`.
- **Standing.** `0/6` stands 622–710 at d 1,536, and the squad 642–721
  at d 625–781.
- **Trailing, pair A.** The first leg comes on **711** at d 1,737, the
  1,728 threshold crossed. From then on there is a leg every 6–9 blocks,
  each issued at d 1,724–1,748 and each **one tile** long. A faster
  follower hops, since the point 1,536 behind the wagon is only ~200
  ahead of it when it crosses the threshold.
- **Trailing, pair B.** `0/8` and `0/9` take legs on **722** and `0/10`
  on 724, at d 784–797. That is the moving threshold 653: **the doubling,
  measured.** Without it the first leg would wait for d > 1,575.
- **Rest and turn.** On 961 `0/6` holds its FOLLOW alone at d 1,612. It
  trails again from 990, and the squad from 882. On 1149 every follower
  holds its FOLLOW alone, within its threshold.
- **The legs** are `MOVEORDER`s with `flags 1`: no action bit, and the
  pathed bit `do_move` sets on the same frame. Every stack of the four
  followers, on every block, is the FOLLOW alone or the FOLLOW under one
  leg.
- **What the reading did not say.** While `0/11` stands, from 835 to 880,
  `0/9`'s leg changes its destination five times at d 700–940, below the
  standing threshold. No fresh `do_follow` leg is due there, and which step
  re-aims it is not named.

**Where this crate parted: `GOLDEN_WORD_CHAPTER_TWELVE` = 717, open;
closed at 1150 by the same item, below.** The
harness skips both `@follow` lines. The sequence parts first on 703, at an
equal count: the original's chariot stands under `do_follow` (`set_anim`,
`5dac7a`) where this crate's idles through `Unit::do_idle`. On **717** this
crate spends 6 draws against 8, parting at draw 0.
`chapter_twelve_s_word_frame_is_widened_whole` covers (605, 719). Past the
first block's 26 standing rows, it pins:

- the staged units' `form` on their birth blocks;
- the missing follows on 622 and 642: `orders.len` 0 against 1, no
  `group`, and `idle` counting;
- the wagon's move group on 702, `group 2` against this crate's 1: the
  follows took pool slots 1 and 0 first (parked 689's family);
- the chariot's first leg on 711: its path, `orders_y`, `dest_angle`,
  heading and position.

The coverage driver takes 620..624, 640..644, 709..713 and 715..719. It
pins the `process_follow` text as unread.

**Closed at 1150 by the same item** (item 714; `docs/ORDERS.md` §28, the
follow, built). The command's entry is `rondata::input::group_follow`,
built the way `group_guard` was: the group, forced into the pool, then
`Sim::group_action_follow`, one `Body::Follow` a member on the leader.
`Sim::do_follow` stands within `s + 0xc0` and otherwise lays a `MOVE_TO`
leg to the standoff point and steps it the same frame. With these the
walk agrees to run204's end: sequence 1150, no value part.
`chapter_twelve_s_word_frame_is_widened_whole` spans run204 whole, 605
to 1150. Past the first block's 26 standing rows it pins only the staged
units' `form` on their birth blocks and the scout `1/0`'s `facing` from
847 (parked 275). Every row the first pin carried goes, the pushed
groups' ids on 622, 642 and 702 among them. Two unit tests in
`sim::group` pin the order and the standoff. The first fails with the
leader's-own-captain exclusion removed, the second with the doubling
removed.

## 21. Chapter thirteen — the garrison line, an issuer the AI never uses (item 718)

**Premise.** A player's garrison command, issued through the original's
own issuer, gives each commanded unit one `GarrisonOrder` on the building.
Each unit walks to the building's approach ring by a plain `MOVE_TO` leg
above that order, and the first of a squad to reach the door takes the
whole squad inside, off the map, with its orders gone. The building's
eject puts the squads back on the map one a frame, first in first out, on
the exit ring south of it. No capture on disk holds a player's garrison
command or an eject: every garrison the long captures hold is the AI's or
a trained unit's. `tools/gamelog/golden/chapter13.cmd` has the reading
with its citations.

**The issuers, under the emulator first.** Item 718's scratch script runs
on `tools/explore/command_oracle.py`'s fixture, widened to who=0's
objects 6–10 as live units and a `Build` at 2003.

- **`CommandManager::issue_garrison@00941a70(group, ox, whom,
  QUEUE_NEW)` appends 18 bytes**: the 5-byte `group` and a 13-byte
  `garrison`, type `0x14`, `[ox i32][whom i32][queued i32]`
  (`docs/COMMANDS.md` §3). It is `issue_follow`'s shape to the byte.
- **`CommandManager::issue_eject_all@00941ca0(group, 0, −1, −1, −1)`
  appends 22 bytes**: a `group` naming the building and a 17-byte
  `eject_all`, type `0x1a`, `[back_to_work][who][eject_o][eject_who]`.
  `CommandPackage::add_group@0094bb60` takes a non-unit object whole,
  since its vslot `+0x18` answers 0 and the captain test is skipped.
- **Each writes** the package's size and data and the four selection
  caches, and nothing else. The same selection again appends the 3-byte
  reuse; `queued` and `back_to_work` ride through as passed;
  `use_mp_playback`, `semaphore & 0x10` and `semaphore & 4` each append
  nothing.
- **What the emulator cannot reach**: `process_garrison@00948760` →
  `Group::action_garrison@00700490` → `Unit::add_garrison_order@005e4080`;
  each frame's `Unit::do_garrison@005e6b80`, `go_inside@0061a2e0` and
  `kill_garrison_order@005e2bd0`; `process_eject_all@00947fe0` →
  `Group::action_eject_all@00710b40` → `Object::eject_contents@0064cd20`;
  and `Build::process_ejection@006201e0` → `Unit::come_out@00617c10`.

The DLL's `@garrison <who> <ox> <whom> <o>…` is `@follow`'s with this
issuer; `@eject <who> <b>…` checks its objects as the player's live
buildings on `Build::vftable`, and passes what the Eject button does
(`Options::exec@007188c0` option `0x1f` → `Options::do_eject_all@0071c470`
→ `GroupOut::issue_eject_all@00708b90`, whose own-player arm writes
`back_to_work 0` and three −1s).

**The reading** (each function's decompile; `docs/ORDERS.md` §5.7–§5.8
and `docs/CITIES.md` §6 had the summaries).

- **The order.** `action_garrison`, with QUEUE_NEW, on an own building
  that is active, has a limit and is not unassimilated, gives every
  member that is active, on the map, not air and not entering or exiting,
  **and whose type `can_garrison` the building's**, one
  `add_garrison_order(b, whom, search 0, QUEUE_NEW, action 1)`. A member
  that cannot garrison gets no order. The dump prints `flags`, `ox`,
  `whom`, `uid` and `search`.
- **The walk.** `do_garrison`, not `Object::adjacent_to` the building
  (vslot `+0x170`: `attack_dist < 0x60`), takes `find_nearby_spot` on
  the ring `min(x_size, y_size) × 0x60 + 0x30` round the building's
  centre, biased toward the unit, `FILTER_NOT_ME`, retried relaxed, and
  adds `add_move_order(spot, MOVE_TO, 0, QUEUE_FIRST, action 0)`. The
  GARRISON stays under the leg. The leg is stepped from the next frame
  (`docs/ORDERS.md` §2.3).
- **The door.** Adjacent, with room (`num_inside(0) + control_cost ≤
  limit`) and the building's cell nobody's, its owner's or an ally's:
  `go_inside(get_captain(), b, who, 0)` takes the whole squad, appended
  at the chain's bottom, and `kill_garrison_order(captain)` walks the
  `o_down` chain. On each unit whose action is a GARRISON it `repath`s,
  which pops the leading moves, and kills the order.
- **The way out.** `action_eject_all` with `who < 0` gives each building
  that is alive, holds a squad and is no hangar `eject_contents(0, −1, 0,
  1)`. For a building on the map, outside the editor, that **defers**:
  `build_masks |= 0x4000`. `Build::process` then runs `process_ejection`
  every frame: `come_out(captain of the head, 1)`, **one squad a frame,
  from the head**. The captain lands on the ring `(x_size + y_size) ×
  0x30 + UNIT_TRAIN_DISTANCE` (672 .. 864), swept from due south, and each
  member round its captain (`docs/CITIES.md` §6.5.1).

**The cast.** A Barracks (4 × 4, `GARRISON_MAX` 10) trains Hoplites
(`can_garrison`'s own-trainer arm) and not Chariots. The Chariot enters by
the sibling arm: `where` Stable admits a Barracks under
`Game::get_patch_version@00595260 > 3`, which this build's own
`info.version` takes (`docs/RECGAME.md` §5). Both have POP 1, so the two
squads fill 2 of 10. Neither is a worker or a packing type, so each takes
the plain QUEUE_NEW arm.

**Lines.**

- `0 !ai off`.
- `606 add barracks who=0 14,74`: `0/2007`, uid 13, centred on the tile
  corner (2688, 14208) as chapter four's Temple was; its footprint is
  cell (3, 18).
- `610 add chariot who=0 20,60`: `0/6`, on cell (5, 15), ~2,880 north-east
  of the door.
- `614 add hoplite who=0 10,90`: the squad `0/7`–`0/9`, on cell (2, 22),
  ~3,250 south of it.
- `620 @garrison 0 2007 0 6` and `640 @garrison 0 2007 0 7`.
- `900 @eject 0 2007`.

A call on trace frame F is on block F+2 (§17's convention).

**The capture must dump** `end:UNITS=3,GUYS=2,BUILDS=7,DEATHS=1,LEADERS=2`
and `misc:COMMANDMANAGER=1` over `[605, 1000)`, beside run105's `start:`
set. `BUILDS=7` is for the Barracks' own chain head, `inside_down`.

**The premise's killer, and its writers** (§3, point 5).

- `check_accept_issue` and `process_group`'s player test are chapter
  nine's (§17).
- **`action_garrison`'s gates**: the owner test, the building's
  `is_active`, a zero limit, `is_unassimilated`, and per member
  `can_garrison`. The chariot's half rests on `get_patch_version`, whose
  one input, `GameInfo.version`, the lobby writes.
- **The editor arm**: QUEUE_NEW under `Game::semaphore` bit `0xb` is an
  instant `go_inside` and no order. Its writers are `Game::Game`, which
  zeroes it, and `ConsoleWin::run_cmd`'s editor toggle, which sets and
  resets it. No line here reaches it.
- **`do_garrison`'s kills**: the building inactive (vslots `+0xc`,
  `+0x4c`), foreign, limitless or refused by `can_garrison`; no spot
  relaxed; full; or standing on an enemy's territory. The start `WORLD`
  has `who −1` on every cell x 0–6, y 14–22, and no line moves a border.
- **The eject's**: `action_eject_all` needs `who < 0`, or `eject_who <
  0` with `who` the group's. `eject_contents` defers only for a building
  on the map outside the editor. `come_out` keeps a unit inside when even
  the relaxed ring finds nothing, and the ground south of the door is
  open BASELAND.
- **The loops' bounds.** `action_garrison`'s member loop is bounded by
  `group.num` (`+0xc`, below `0x80` in `Group::add`), here 1 and 3;
  `action_eject_all`'s by its group's `num`, 1. The scenario sweeps run
  only under `ScenarioData::ignore_orders`, whose writers are
  `ScenarioFuncSet`'s. `go_inside`, `kill_garrison_order` and
  `come_out`'s recursion walk the captain's `o_down` chain, `uber_size`
  long (3 and 1). `process_ejection` has no loop, so one squad a frame.

**The ground** (run204's start `WORLD`, the same map and seed): cells
x 0–6, y 14–22 are BASELAND with no border, west of chapter nine's sand
(x 7–11). The chariot walks south-west across cells (5, 15)–(4, 17), the
squad north across (2, 22)–(3, 19).

**What would falsify it, and where each could first fire.**

1. **The issuer does not reach the pump.** Trace frames 620, 640 and 900:
   an `INFO` 17 with a refusal. Or no `COMMANDMANAGER` `process_group`
   and `process_garrison` text between blocks 621/622 (641/642), or no
   `process_eject_all` between 901/902.
2. **Not one `GarrisonOrder` a unit on the building.** Block 622 for
   `0/6`, block 642 for each of `0/7`–`0/9`: no type-26 order at the
   bottom of the stack; `ox/whom` other than 2007/0; a `uid` other than
   13; the action bit clear; `search` other than 0.
3. **A unit does not walk to the door.** Block 622 (`0/6`) and 642 (the
   squad): no `MOVEORDER` leg above the GARRISON, without the action bit,
   aimed 432–480 from (2688, 14208) on the unit's side. Or the GARRISON
   gone while the unit is on the map.
4. **The door does not take the squad whole.** By speed the chariot is
   in near block 708 and the squad near 766. It fires on any member
   inside on a block its captain is not; a unit inside with an order
   left; an `inside_up` chain other than 2007 ← 6 ← 7 ← 8 ← 9 with the
   Barracks' `inside_down` 6; or a unit still on the map at block 850.
5. **The eject is not one squad a frame, first in first out.** Block
   902: `0/6` on the map on the 672–864 ring south of the Barracks
   (bearing 0 at 672 snaps to (2712, 14904)), the squad inside, and
   `inside_down` 7. Block 903: the squad out and `inside_down` −1. It
   fires on both out on one block, the squad first, or anyone out before
   902 or after 903.
6. **They come out with orders.** Any of the four with a non-empty stack
   on blocks 902–999: `back_to_work` is 0, a cheat's Barracks has no rally
   point, and a human's unit keeps what it had, which was nothing.

Predicted: none fires. **This crate cannot take either command yet.** It
has the GARRISON order, `do_garrison`, `go_inside`, `come_out` and the
deferred ejection, all written from the reading and none diffed against a
player's command. The harness skips all three `@` lines as a named seam
in `crate::golden`. If the capture agrees with the reading, the first
landing is the two commands' entry into the sim, built the way
`crate::input::group_follow` was.

**Run 2026-09-24 as run208 (item 718): no falsifier fired**
(`docs/RUNS.md` has the tables).

- **All three commands reach the pump.** `INFO` 17 on 620, 640 and 900,
  refusal 0. The dump prints `process_garrison 2007 0 2` on 621 and 641,
  and `process_eject_all 901`.
- **One `GARRISONORDER` a unit, as read.** On 622 and 642 each of the
  four has `flags 4, ox 2007 whom 0 uid 13, search 0` under its own
  `MOVEORDER` leg, `flags 0`. The chariot's point is 461 from the centre.
- **The door takes the squad whole.** `0/6` is inside on **699** and
  `0/7`–`0/9` together on **761**. The chain is 2007 ← 6 ← 7 ← 8 ← 9
  with `inside_down` 6, and every stack is empty inside.
- **One squad a frame, first in first out.** `0/6` is out on 902 at
  (2712, 14904) with `inside_down` 7, and the squad on 903. `build_masks`
  carries `0x4000` on 902 and 903. Every stack stays empty to 999.
- **What the reading did not say.** Each walk ended short of its leg: the
  chariot's last step on 698 left it ~140 from its point, and the
  captain's on 761 ~35. The door is `adjacent_to`, tested with the leg
  still at the head. Which step does it is not named. `Unit::work`'s
  every-16-frames step, phased by `o`, fits both frames and is only a
  hypothesis.

**Where this crate parted: `GOLDEN_WORD_CHAPTER_THIRTEEN` = 640, open.**
The harness skips both `@garrison` lines and the `@eject`. On 622 the
original's chariot holds its GARRISON under a plain leg and is its pushed
group's (`group 1`), where this crate's holds nothing and counts `idle`. On
623 it has pathed (`tolerance 384`) and stepped. On **640** this crate
spends 37 draws against 36, parting at draw 30 on the standing chariot's
idle roll (`Guy::set_anim+0x97a < Guy::inc_time+0x271`).
`chapter_thirteen_s_word_frame_is_widened_whole` covers (605, 642): past
the first block's 13 standing rows it pins the staged units' `form` and
the chariot's 622–623 rows, and the Barracks agrees on every block. The
coverage driver takes 620..624, 638..644, 697..701, 759..763 and
900..904, and pins `process_garrison`, `process_eject_all` and the
order's `search` as unread.

**Closed at 1000 by the same item** (item 718; `docs/ORDERS.md` §29).
`rondata::input::group_garrison` pushes the group and calls
`Sim::group_action_garrison`; `group_eject_all` calls
`Sim::action_eject_all`. That alone moved the word to 903, and the
widening named four things under it, each fixed:

- `check_target_path`'s GARRISON arm (the door on the review, 699 and
  761);
- the door as `adjacent_to`;
- `kill_garrison_order`'s walk down the chain;
- `come_out`'s angles.

With these the walk agrees to run208's end: sequence 1000, no value part.
The widening over run208 whole leaves the standing `form`s, parked 275's
scout row on 847, and the squad's `group` on 903. `come_out` pushes a
squad that leaves a building into a fresh pool slot, behind the eject's
own building group, and this crate carries neither (parked 689's family).
A test in `sim::cities_tests` pins the command, the whole-squad door and
the exit's angles. It fails with the one-unit kill, and again with the
angles reset.

## 22. Chapter fourteen — the formation line, an issuer the AI never uses (item 723)

**Premise.** A player's formation command, issued through the original's
own issuer, **makes no order of its own**. It writes the formation into
every member's `UnitData::form` (`+0xaa`) and lays out a group move in it.
A group that stands re-forms on the spot, round its leader, with orders
that carry no action bit. A group that walks is halted and its move
replayed to the same point in the new formation, with the action bit. The
booked premise was that the command enters `FormOrder` (§13's table). The
reading kills that before the run, and this chapter measures the
absence. No capture on disk holds a formation command: the console has no
formation verb (`docs/RUNS.md` run45), so every group on disk is the AI's
or a right-click's. `tools/gamelog/golden/chapter14.cmd` has the reading
with its citations.

**The issuer, under the emulator first.** A scratch script ran on
`tools/explore/command_oracle.py`'s fixture, widened to who=0's objects
6–10 as live captains.

- **`CommandManager::issue_form@00941580(group, form, rotate,
  QUEUE_NEW)` appends 18 bytes**: the 5-byte `group` and a 13-byte
  `form`, type `0x03`, `[form i32][rotate i32][queued i32]`
  (`docs/COMMANDS.md` §3). Its prologue is `issue_follow`'s to the byte.
- **It writes** the package's size and data and the four selection caches,
  and nothing else. The same selection again appends the 3-byte reuse.
  `form −2, rotate 0x40000000, queued 1` and `form −1` ride through as
  passed, and a non-captain is dropped. `use_mp_playback`, `semaphore &
  0x10` and `semaphore & 4` each append nothing.
- **Its callers.** `Options::do_formation@007215b0` passes the button's
  formation, `rotate` 0 and the queue from the shift and alt keys (option
  `0x14` passes −3, "the previous one"). `Options::do_rotate@0071dfd0`
  passes −2 and an angle. The DLL's `@form <who> <form> <rotate> <o>…`
  passes QUEUE_NEW, a button with no key held.
- **What the emulator cannot reach**: `process_form@00949d90` →
  `Group::action_form@00707220`, and under it `set_up_insert@0070e520`,
  `action_halt@0070d0c0`, `GroupData::get_loc_to@0070c5d0`,
  `action_move_to@0070fba0` → `action_move_near@00704990` →
  `Form::compute@0072e8e0`, and `finish_insert@0070e620`.

**The reading.**

- **No `FormOrder`.** Nothing in the export calls
  `OrdersMemManager::get_obj(CHANGE_FORM)`. `get_new_order@00730550`
  builds one only for the save loader (`OrderList::walk_data`,
  `get_new_data`) and for `copy_order@0072f900`, which copies an order that
  already exists. `finish_insert`'s case `0x12` would replay one, and none
  exists to replay.
- **The gates.** `action_form` returns early for a group that is off the map
  or holds buildings, or that is empty, leaderless or led by a plane.
- **With an explicit formation and QUEUE_NEW**, the group's `form`
  (`+0x10`) is set to −1. Then:
  1. `set_up_insert` copies the leader's action-bit orders aside.
  2. `action_halt(0)`.
  3. The function recurses with `param_5 = 1`: QUEUE_NEW when nothing was
     copied, QUEUE_FIRST when something was.
  4. `finish_insert`.
- **The recursion** writes `unit +0xaa = form` on every member that is a
  unit and no plane. That includes figures, and it does not exempt
  citizens, which a move does (`docs/GROUPS.md` §6.6).
- **Standing** (QUEUE_NEW): `get_loc_to` gives the leader's final point,
  where it stands when it has no orders. Then `action_move_to(that,
  QUEUE_LAST, set_angle 0, angle 0, MOVE_TO, action 0, form −1, width −1,
  0)`, and `form −1` reads the byte just written through `get_form`. With
  a zero delta, the formation's bearing is the leader's heading less its
  slot byte (`docs/GROUPS.md` §6.3).
- **Walking** (QUEUE_FIRST): nothing more happens in the recursion.
  `finish_insert`'s case `0x13` replays the leader's copied
  `GroupMoveOrder`: `action_move_near(orig_x, orig_y, tolerance,
  QUEUE_LAST, set_angle 1, the order's angle, MOVE_TO, action 1, form −1,
  …)`. `orig_x`/`orig_y` are `+0x44`/`+0x48`, which
  `Unit::add_group_move_order@005e4710` sets to the group's anchor, not
  to the leader's slot.

**The cast.** Three squads on chapter thirteen's ground (cells x 0–6,
y 14–22: BASELAND, no border), no Barracks:

- two Hoplite squads, captains `0/6` and `0/9` (`FORM_CAT_FOOT`);
- a Slinger squad, captain `0/12` (`FORM_CAT_FOOT_RANGED`).

`find_leader` takes `0/6`, the first of the lowest category. Formation 2 is
Envelop and 0 is Line, both buttons.

**Lines.**

- `0 !ai off`.
- `610 add hoplite who=0 12,84`, `612 add hoplite who=0 18,84` and
  `614 add slinger who=0 15,88`.
- `620 @form 0 2 0 6 9 12`: Envelop, standing.
- `700 @move 0 2976 12000 6 9 12`: north, ~4,200 to cell (3, 15).
- `740 @form 0 0 0 6 9 12`: Line, on the move.

A call on trace frame F is on block F+2 (§17).

**The capture must dump** `end:UNITS=3,GUYS=2,BUILDS=7,LEADERS=2,GROUPS=1`
and `misc:COMMANDMANAGER=1` over `[605, 1150)`, beside run105's `start:`
set. `GROUPS=1` puts the 512-slot pool in every block, for `GROUPDATA`'s
`form`, `off`, `curr` and `angles`. `DEATHS` is off under `[End Frame]`:
`dump_deaths` would leave the logger's type at `WORLD` and drop the pool
(run166's trap). With it off, `dump_units` runs just before `dump_groups`
(`GameLog::full_dump@00930380`), as in run178's line.

**The premise's killer, and its writers** (§3, point 5).

- **A `FORMORDER` anywhere**, on any unit on any block. Its only builders are
  `copy_order`, which `set_up_insert` alone calls, on an order that exists,
  and the save loader.
- **The `form` byte.** Its writers are:
  - `UnitData::UnitData@00606670` (−1);
  - `action_form`;
  - `action_move_near`'s step 1, per member of every group move;
  - `SpellType::cast_civilian@006704a0` and `cast_to_arms@00670880`.

  No line here casts, and `!ai off` issues no group move.
- `check_accept_issue` and `process_group`'s player test are chapter
  nine's (§17). `action_form`'s gates are above. The leader is a Hoplite.
- **The loops' bounds.**
  - `action_form`'s member loop and `get_form`'s run to `group.num`
    (`+0xc`, below `0x80` in `Group::add`): here nine, with the figures.
  - `set_up_insert` walks the leader's order list to its tail, and
    `finish_insert` walks the copies until the list is empty.
  - `Form::compute` walks the members.
  - `get_form_option`'s `% 5` and −3 arms are not reached: both formations
    are explicit.

**What would falsify it, and where each could first fire.**

1. **The issuer does not reach the pump.** Trace frames 620, 700 and 740:
   an `INFO` 17 with a refusal. Refusal 3 would name a wrong captain id,
   12 above all. Or no `COMMANDMANAGER` `process_form 2 0 2` between
   blocks 621 and 622, or no `process_form 0 0 2` between 741 and 742.
2. **A `FormOrder` is made.** Any `FORMORDER` block, from 622 on.
3. **The byte is not written.** Block 622: any of `0/6`–`0/14` with
   `UNITDATA form` other than 2. Block 742: other than 0.
4. **The standing group does not re-form on the spot.** Block 622: a
   member whose stack is not one `GroupMoveOrder` (type 19) leader `0/6`,
   under one id, without the action bit, aimed at a slot round `0/6`'s own
   position. The group's `GROUPDATA` `form` other than 2. Or any stack
   empty with the member off its slot.
5. **The move does not read the byte.** Block 702: the `GroupMoveOrder`s
   laid out in anything but Envelop (`GROUPDATA form` 2), or without the
   action bit.
6. **The walking group is not halted and replayed.** Block 742: a member
   whose stack is not exactly one `GroupMoveOrder` under a new id minted
   on 741. It fires on an order without the action bit, on one laid out
   in anything but Line, and on an anchor other than 700's point
   (2976, 12000), such as the leader's old slot. It also fires on the
   Envelop order still under the new one.
7. **They do not arrive in Line.** By speed, near block 900. It fires on a
   member still walking on 1100, or standing off its Line slot.

Predicted: 2 does not fire, and the rest do not either. **This crate cannot
take the command yet.** It carries `get_form`, `get_form_option`,
`get_loc_to`, the slot table, the halt and `QUEUE_FIRST`'s insert dance.
Two of them are not the original's here:
- Its replay of a copied move goes to the order's `dest`, the leader's
  slot, not its `orig`.
- Its moves take `get_form` whatever the caller passes.

The harness skips both `@form` lines as a named seam in `crate::golden`.
So the word should part on 622, where the original's group walks to its
Envelop slots and this crate's stands.

**Run 2026-09-25 as run210 (item 723)** (`docs/RUNS.md` has the tables).

- **Falsifiers 1, 2, 3, 5, 6 and 7 did not fire.**
  - All three commands reached the pump: `process_form 2 0 2 621`, the
    move on 701, and `process_form 0 0 2 741`.
  - **The dump holds no `FORMORDER` at all.** The kill of the booked
    premise stands, measured.
  - All nine members read `form 2` on 622 and `form 0` on 742.
  - On 702 the move laid Envelop `GroupMoveOrder`s with the action bit.
  - On 742 the walking group was halted and its move replayed. Each
    member holds one `GroupMoveOrder`, `flags 5`, under the new id 741102,
    with `orig` 700's point (2976, 12000) and leader `oxx 6`, in Line, and
    nothing under it.
  - They arrive 905–926.
- **Falsifier 4 fired in its letter.** On 622 each member holds a plain
  `MOVEORDER` (type 1), not a `GroupMoveOrder`. Its point is its Envelop
  slot round `0/6`'s position, its `orig` that same slot, and it carries
  no action bit. Every stack is empty on its slot by 684. The rest of the
  claim held. `Unit::do_group_move@005e79a0` ungroups a group move whose
  point lies within `0x5ff` of the leader, and a re-form on the spot
  always does. That the orders were laid grouped on 621 and ungrouped on
  their first step is **a hypothesis**. This crate models both, and its
  walk decides.
- **The pool did not come out**: 0 `GROUPDATA` records, though `DEATHS`
  was off and the ini carried `GROUPS=1`. The hypothesis is that `GUYS=2`
  rejects the lines, where run178's `GUYS=4` passed them (`docs/RUNS.md`,
  run210). The measures above come from each member's own order.

**Where this crate parted: `GOLDEN_WORD_CHAPTER_FOURTEEN` = 631, open.**
The harness skipped both `@form` lines. On 622 the original's nine hold
`form 2`, `form_mod 50`, `group 1` and a plain move each, where this
crate's hold nothing. On **631** this crate spends 9 draws against 10,
parting at draw 3: the original's leader `0/6`, stopped on its slot,
rolls an idle (`Guy::set_anim+0x97a < Unit::do_idle+0x7d`). The widening
over (605, 633) had 13 standing rows on 605, each unit's birth `form`,
and the nine on 622–626. The coverage pin failed first on `process_form`
and on the right-click's `process_move_to` lines. Its driver takes 620..624,
629..633, 700..704, 740..744 and 903..907.

**Closed at 1150 by the same item** (item 723; `docs/ORDERS.md` §30).
- **The entry.** `rondata::input::group_form` pushes the group and calls
  `Sim::group_action_form`. That alone moved the word to 740. The
  standing re-form now agrees value for value, so the plain moves need
  no ungroup hypothesis: this crate lays them the same way.
- **The equal group's slot and record.** `push_group` keeps them for the
  player's last pushed slot, so the right-click on 701 mirrors Envelop as
  the original's does. That moved the word 740 → 764.
- **The replay to `orig`.** `finish_insert` replays the copied group move
  to `orig`, the group's point, not the leader's slot. That moved the
  word 764 → 1150.

Sequence 1150, and no value part. The widening over run210 whole leaves:
- the standing `form`s;
- the group move's `id` on 702, 701101 against 707501, which is this
  crate's numbering of a pushed group (parked 689).

The equal-group fix took parked 275's scout row off ten other chapters'
widenings and the two controls.

## 23. Chapter fifteen — the group attack, an issuer the AI rarely takes whole (item 731)

**Premise.** A player's attack on an enemy unit, issued through the
original's own issuer, **makes no group order**. Each member, figures
too, gets its own `AttackOrder` (type 10) on the target, `mandatory` 1,
`new_ord` 1, with the action bit, and walks, fights and stops on its own.
A player's attack-move on the ground **is** a group order: each member
of a land group gets a `GroupAttackToOrder` (type 21). The booked premise
was that `issue_attack` enters both classes (§13's table). The reading
kills half of it before the run: `issue_attack` takes a target, never a
point, and no issuer builds a `GroupAttackOrder`. No capture on disk
holds a player's attack command: the console has no attack verb, and
every attack on disk is the AI's `engagement` (`mandatory` 0) or an
auto-engage. `tools/gamelog/golden/chapter15.cmd` has the reading with
its citations.

**The issuer, under the emulator first**, on
`tools/explore/command_oracle.py`'s fixture widened to who=0's objects
6–14, with 6, 9 and 12 as captains (a scratch script in the job's tmp dir).

- **`CommandManager::issue_attack@009415e0(group, ox, whom, ignore,
  queued)` appends 26 bytes** for three captains: the 9-byte `group` and
  a 17-byte `attack`, type `0x04`, `[ox][whom][ignore][queued]`
  (`docs/COMMANDS.md` §3).
- **It writes** the package and the selection caches, and nothing else.
  The same selection again appends the 3-byte reuse. `ignore 7` and every
  queue ride through as passed, and a non-captain is dropped.
- **A negative `ox` or `whom` appends nothing and writes nothing**: the
  issuer tests both before `check_accept_issue`. `use_mp_playback`,
  `semaphore & 0x10` and `semaphore & 4` each append nothing.
- **The ground point is not this issuer's.** A ctrl+right-click is
  `issue_move_to@00941720(…, ATTACK_TO, −1, −1, 0)`
  (`WorldMap::on_right_up@008c7050:199`), and the emulator appends its
  22-byte `move_to` with the `orders` byte 2.
- **The DLL's two new verbs.** `@attack <who> <ox> <whom> <o>…` passes
  `ignore` 0 and QUEUE_NEW, the right-click's bytes
  (`Console::execute_at_cursor@007c6630:2741` through
  `GroupOut::issue_attack@0070b060`). `@amove` is `@move` with ATTACK_TO.
  Both compile `-Werror`, plain and under `RON_AUTOSTART`.
- **What the emulator cannot reach**: `process_attack@00949c30` →
  `Group::action_attack@00712490` → `Unit::add_attack_order@005e5410`,
  and `process_move_to@009497c0` → `action_move_near@00704990` →
  `Unit::add_group_move_order@005e4710`.

**The reading.**

- **No `GroupAttackOrder`.** Nothing asks `OrdersMemManager::get_obj`
  for `GROUP_ATTACK` (20). `get_new_order@00730550:193` builds one only
  for the save loader and for `copy_order@0072f900`, which copies an
  order that exists (`docs/ORDERS.md` §7.9). A record would print as
  `GroupAttackOrder`, mixed case (`GroupAttackOrder::log_data@00485140`),
  and `get_type@00485320` answers 20.
- **The attack** (`docs/ORDERS.md` §8.5, `docs/GROUPS.md` §10).
  `process_attack` hands the command on when the target's object is
  active and calls `action_attack(g, ox, whom, mandatory 1, queued,
  ignore)`. For a unit group on the map, a unit target and QUEUE_NEW:
  1. The leader, out of range, asks `find_attack_pos` once
     (`action_attack+0x41a`).
  2. Three passes, one a domain. Each member that is not already on a
     mandatory attack of this target gets `add_attack_order(ox, whom,
     QUEUE_NEW, mandatory 1, action 1)`.
  3. QUEUE_NEW closes the member's orders, so the right-click's group
     move goes. `mandatory` 1 skips the `find_melee_target` retarget,
     which is the AI's arm.
- **The attack-move.** `action_move_near`'s step 6 (`docs/GROUPS.md`
  §6.6) gives each member of a land group of two or more a group move,
  unless it is modern infantry, a scout, sea, `unit_masks & 4` or form 9.
  `add_group_move_order` asks `get_obj(GROUP_ATTACK_TO)` when its kind
  is 2. Its step, `Unit::do_group_attack_to@005e74e0`, is
  `do_group_move` plus a `find_melee_target(−1, …)` look every
  fifteenth frame, `(o + frame) % 15 == 0`.

**The cast**, on chapter thirteen's ground and chapter eleven's column
north of it (cells x 3–5, y 9–22, BASELAND, no border):

- two Hoplite squads, captains `0/6` (the leader) and `0/9`;
- a Chariot of who=1, `1/6`, at (3192, 12408), about 3,840 north.

A Hoplite sees about 1,150 and searches 2,304 (`unit_respond_range` 12
× `0xc0`). The Chariot sees about 1,730 and outranges it. **No ground
lets who=0 see the Chariot while nobody engages**, and a click can name
only a seen target: `execute_at_cursor` filters on `FILTER_SEEN`, and
`fight`'s `valid_target` drops an unseen one on its first step. So the
group walks in under a right-click, through the Chariot's spot. The
move's action bit keeps it from answering fire, and the attack comes
fifteen frames after this crate first sees the target.

**Lines.**

- `0 !ai off`.
- `610 add hoplite who=0 12,84`, `612 add hoplite who=0 18,84`, `614 add
  chariot who=1 16,64`.
- `620 @move 0 3192 10752 6 9`: north, past the target.
- `734 @attack 0 6 1 6 9`: the right-click on `1/6`.
- `860 @amove 0 3192 7680 6 9`: the attack-move north, on the ground.

A call on trace frame F is on block F+2 (§17).

**The capture must dump** `end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1`
and `misc:COMMANDMANAGER=1` over `[605, 1250)`, beside run105's `start:`
set. These are run178's levels, the one line whose pool printed.
`GroupData::log_data@0045e1d0` sets no logger type of its own, so the
pool passes `GameLog::check_accept@009309a0` only on the type and detail
the dumper before it left. run210 asked at `GUYS=2` and got none (parked
733). `DEATHS` and `AMMO` are off, so the target's death is read off
`UNITS`.

**The premise's killer, and its writers** (§3, point 5).

- **A `GroupAttackOrder` record anywhere**, in either spelling. Its only
  builder, `get_new_order`, is reached by `copy_order`, which
  `set_up_insert` alone calls on an order that exists, and by the save
  loader.
- **The `mandatory` byte**, `AttackOrder +0x1c`. Its writer on this path
  is `add_attack_order`'s `param_4`, which `action_attack` passes as its
  own `param_3` and `process_attack` passes as 1.
- **The kind 2.** `process_move_to` passes the packet's `orders` byte to
  `action_move_to`, and `add_group_move_order` tests `param_10 == 2`.
- `check_accept_issue` and `process_group`'s player test are chapter
  nine's (§17).
- **The loops' bounds.** `action_attack`'s member loop runs to
  `group.num` (`+0xc`, below `0x80` in `Group::add`), here six, inside
  `local_14 < 3` domain passes. `action_move_near`'s runs to the same
  `num`.

**What would falsify it, and where each could first fire.**

1. **The issuer does not reach the pump.** Trace frames 620, 734 and
   860: an `INFO` 17 with a refusal. Or no `COMMANDMANAGER`
   `process_attack` naming `6 1 0 2` between blocks 735 and 736, or no
   `process_move_to 3192 7680 2 0 0 2 0` between 861 and 862.
2. **A `GroupAttackOrder` is made.** Any `GroupAttackOrder` or
   `GROUPATTACKORDER` block, from 736 on.
3. **The attack is not six `AttackOrder`s.** Block 736: any of `0/6`–`0/11`
   without an `ATTACKORDER` on `ox 6 whom 1` with `1/6`'s `uid`,
   `mandatory 1` and the action bit (`flags & 4`), or with the
   right-click's `GROUPMOVEORDER` still under it.
4. **The approach does not hold its target.** Any `ATTACKORDER` on
   another object while `1/6` prints, or `1/6` still printing on block
   900.
5. **The death leaves them ordered.** Block 850: a member whose stack is
   not empty. A resumed right-click leg would fire here.
6. **The attack-move is not six `GroupAttackToOrder`s.** Block 862: a
   member without one `GROUPATTACKTOORDER` (type 21) with `flags & 4`,
   `orig` (3192, 7680), one id and leader `oxx 6`, unless its
   `unit_masks & 4` holds (step 6's exemption, a plain `ATTACKTOORDER`).
7. **They do not arrive.** Block 1200: a member still ordered, or
   standing more than a tile from its slot round (3192, 7680).

**This crate's prediction**, walked from run210's start with `@attack`
through `crate::input::group_attack` (a scratch walk; run210 is the same
game to 610):

- The group walks from 622 and sees `1/6` from 719. The Chariot shoots
  from 728.
- On 736, each member holds an `ATTACK` on `1/6` under a `MOVE_TO`
  approach leg.
- Contact on about 775. `1/6` falls to 19 hits by 780 and dies by 810.
  The stacks are empty by 840.
- On 862, six `GroupAttackToOrder`s. They ungroup to plain
  `ATTACKTOORDER`s near the point about 1060 and stand by 1080.

Predicted: none fires. **The harness skips `@attack`**, a named seam in
`crate::golden` until the floor is pinned. `crate::input::group_attack`
is built, and the scratch walk used it. So the word should part on 736,
where the original's six charge and this crate's walk on.

**Run 2026-09-25 as run215 (item 731)** (`docs/RUNS.md` has the tables).

- **No falsifier fired.** All three commands reached the pump on the next
  frame, among them `process_attack 6 1 0 2 735`.
- **The dump holds no `GroupAttackOrder`**, in either spelling. The kill of
  the booked premise's first half stands, measured.
- On 736 each member holds one `ATTACKORDER` on `ox 6 whom 1`, `uid 12`,
  `mandatory 1`, `new_ord 1`, `flags 20`, over a `MOVEORDER` approach
  leg. The right-click's group move is gone. `0/6` stands on (2999,
  13821), this crate's point.
- In range by 780. **`1/6` last prints on 808.** The stacks empty by 850.
- On 862 each member holds one `GROUPATTACKTOORDER`, `flags 5`, id
  861102, `orig` (3192, 7680), leader `oxx 6`. None has `unit_masks & 4`.
- They ungroup to plain `ATTACKTOORDER`s on 1060 and stand from 1080 on
  exactly the six points this crate predicted.
- **The pool printed**: 330,240 `GROUPDATA` at `GUYS=4`, where run210's
  `GUYS=2` printed none.

**Where this crate parted: `GOLDEN_WORD_CHAPTER_FIFTEEN` = 753, open.**
The harness skipped `@attack`, so this crate's six walked on under the
right-click where the original's charged from 736. The charge spends no
draw until **753**: 6 draws against 7, parting at draw 0 on the
original's `Guy::set_anim+0x97a < Unit::move_step+0x823`.

- **The widening** over (605, 755) holds the standing rows on 605, the
  births' `form`, the right-click's group `id` on 622 (parked 689), and
  on 736 105 keys of the six, every one of them the skipped attack's.
  It failed first on an empty pin.
- **The pool's widening**, `widen_pool`, is the first on a golden
  capture: who=0's 64 slots, both directions, on what each holds. Slot 1
  is the pushed selection from 621. Its **`stamp` (`+0x14`) is 621 in
  the dump and 0 here**: this crate's pushed record never writes it. The
  rest is the skipped attack's (`order_num`, `facing` on 736, `curr` on
  738).
- **The coverage pin** failed first on `process_attack`, pinned unread
  beside its siblings. Its driver takes 620..624, 734..738, 751..755,
  807..811, 860..864 and 1058..1062.

**Closed at 1250 by the same item.**
- **The entry.** `rondata::input::group_attack` pushes the group and
  calls `Sim::group_action_attack` with `mandatory` 1. That alone moved
  the word 753 → 1250, run215's trace end: sequence 1250, no value part.
  The attack-move needed nothing new: `input::group_move_to` already took
  `orders` 2 into `GroupAttackToOrder`s, and they agree value for value
  from 862 to the arrival.
- **A fresh slot's `stamp`.** `Groups::copy_group@006fa690` writes `+0x14
  = game->frame` on the slot `push_group` copies into. This crate's
  record started at 0. It is stamped now, and the pool widening parts
  nowhere on run215's 645 blocks. Nothing here reads a pushed `stamp`,
  so no draw moved.

The widening over run215 whole leaves:
- the births' `form`;
- the group move's `id` from 622 (parked 689);
- one instrument row, `1/6`'s death object `extra` on 809. `DEATHS` was
  off for the pool, and `DEATH_OBJS` prints a record per corpse and
  nothing else, so the dump cannot say the list was not asked for.

## 24. Chapter sixteen — explore and flee, the move issuer's trailing selector (item 738)

**Premise.** `CommandManager::issue_move_to@00941720`'s `orders` byte,
the trailing selector, set to `EXPLORE_TO` (3) or `FLEE_TO` (4), makes
**one `ExploreToOrder` (type 3) or one `FleeToOrder` (type 4) a member,
each at its own formation slot, and no group order**. Chapters fourteen and
fifteen died on "no issuer builds one", so the booking asked the
emulator first. The issuer builds nothing at all, and the class is chosen
at process time by `Unit::add_move_facing_order@005e55c0`'s switch on its
kind. The explore's one behaviour of its own is the goody look, and the
staging puts a box in reach of each explorer. `tools/gamelog/golden/
chapter16.cmd` has the reading with its citations.

**The issuer, under the emulator first**, on
`tools/explore/command_oracle.py`'s fixture widened to who=0's objects
6–14, with 6, 9 and 12 as captains (a scratch script in the job's tmp dir).

- **`orders` 0 to 5 append the same 27 bytes**, a 5-byte `group` of one
  and the 22-byte `move_to`, differing in the `orders` byte alone. The
  issuer copies the byte and tests nothing.
- **It writes** the package and the selection caches, and nothing else,
  as in §17 and §23. The same selection again appends the 3-byte reuse; a
  non-captain is dropped; `queued` 0, 1 and 2 ride through;
  `use_mp_playback`, `semaphore & 0x10` and `semaphore & 4` each append
  nothing.
- **What the emulator cannot reach** is everything that makes a class:
  `process_move_to@009497c0` → `Group::action_move_to@0070fba0` →
  `action_move_near@00704990` → `add_move_facing_order@005e55c0`.
- **The DLL's two new verbs.** `@explore` and `@flee` are `@move` with
  `orders` 3 and 4. `@explore` is the Explore button's pick on the ground
  (`Options::picked_spot@00721c40:905`). The Flee button (`:946`) passes
  `FLEE_TO` to a friendly building's point and then a `QUEUE_LAST`
  `issue_garrison` of it; `@flee` issues the move alone. Both compile
  `-Werror`, plain and under `RON_AUTOSTART`.

**The reading.**

- **The class.** `add_move_facing_order` maps kind 2 → `get_obj(ATTACK_TO)`,
  3 → `get_obj(EXPLORE_TO)`, 4 → `get_obj(FLEE_TO)`, anything else
  `MOVE_TO`. Its kind is `action_move_near`'s `param_7`, which is
  `process_move_to`'s `orders` byte. Both classes are `MoveOrder`s with no
  fields of their own (`docs/ORDERS.md` §4.1). `do_flee_to@005f2480` is
  `do_move`.
- **No group.** `action_move_near:758` hands a member to
  `add_group_move_order` only for `MOVE_TO` or `ATTACK_TO`. A squad told
  to explore or flee gets a plain order a member at its slot, pathed, with
  the action bit, and `orig` the click.
- **The explore's look.** `do_explore_to@005f24a0` is `do_move` and, every
  fifteenth frame (`(o + frame) % 15 == 0`), on a captain whose head is
  still this order, `find_goody_box@005f2540`: a 49-cell sweep for a seen
  box in the unit's region (`docs/GOODY.md` §7). A hit calls
  `get_goody_box@005f7690`: a one-member group of the captain, which drags
  its figures (`docs/GROUPS.md` §4.1), pushed, and `action_move_to(box
  cell centre, QUEUE_FIRST, EXPLORE_TO, action 0)`. The group's
  `QUEUE_FIRST` copies the leader's action-flagged orders aside, halts,
  issues the box leg `QUEUE_NEW`, and re-issues each copy through
  `Group::finish_insert@0070e620` case 3. That call is
  `action_move_near(orig_x, orig_y, QUEUE_LAST, set_angle 1, the copy's
  angle, EXPLORE_TO, action 1)`: **to the copy's click, not its slot**.
- **The look repeats.** The box leg has no action bit, so `update_action`
  walks past it, and `orders_x/y` stays the re-issued explore's point.
  `find_goody_box`'s "already going there" test compares the box against
  `orders_x/y`, so the look re-issues the leg on every fifteenth frame
  until the unit stands in the box's cell. Entering it opens the box
  (`Unit::set_new_location` → `explore_goody`, one draw a candidate good,
  `docs/GOODY.md` §2–§3).
- **The flee's readers** are `UnitData::is_fleeing@0046efa0`,
  `order_type() == FLEE_TO`. `Unit::target_opportunity@005fffc0` returns
  on it, `PathFinder::calc_cost@00684e50` triples a seen world or tile
  step's extra on it, and `Unit::resolve_unit_collision@005f9d30:380`
  does not wait for a blocker that holds one. None is staged to fire:
  no enemy is in the window, and the flee paths cross no seen cell whose
  extra is above 0.

**The cast**, on chapter nine's and ten's ground (BASELAND, no border).
Two goody boxes are in reach, read off run215's start `WORLD` (22 on the
map): cells (1, 19) and (16, 21).
- a Chariot `0/6`, seated at tile (12, 60), cell (3, 15);
- a Hoplite squad `0/7`–`0/9`, captain `0/7`, seated at tile (52, 60),
  cell (13, 15).

**Lines.**
- `0 !ai off`; `610 add chariot who=0 12,60`; `612 add hoplite who=0 52,60`.
- `620 @explore 0 2400 17280 6`: south, past the box at (1, 19).
- `640 @explore 0 12672 14976 7`: south-east, two cells short of the box
  at (16, 21), which the captain sees on the way.
- `900 @flee 0 2400 11520 6`: the chariot back north.
- `1000 @flee 0 10752 11520 7`: the squad back north-west.

A call on trace frame F is on block F+2 (§17).

**The capture must dump** `end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1`
and `misc:COMMANDMANAGER=1` over `[605, 1250)`, beside run105's `start:`
set: run215's levels, the line whose pool printed (parked 733).

**The premise's killer, and its writers** (§3, point 5).
- **A `MOVEORDER` (type 1) or any group order where an explore or a flee
  is predicted.** The dump names the class by its record's label,
  `EXPLORETOORDER` (`ExploreToOrder::log_data@00483000`) or `FLEETOORDER`
  (`FleeToOrder::log_data@00482eb0`), and its `type` line (`FleeToOrder::
  get_type@00482f30` answers 4). The type's writer on this path is
  `add_move_facing_order`'s switch. Its two other `EXPLORE_TO` writers
  are not reached: the `QUEUE_LAST` conversion of a `role & 0x10` type's
  `MOVE_TO` (the call is `QUEUE_NEW`, and neither type is a scout), and
  `Unit::work`'s step 5 on `unit_masks & 0x4000000`, which the `QUEUE_NEW`
  arm clears (`005e568d`).
- **The loops' bounds.** `action_move_near`'s member loop runs to
  `group.num`: 1 for the chariot, 3 for the squad. `find_goody_box` sweeps
  `0xc4 / 4` = 49 cells. `explore_goody` walks goods 0..6.

**What would falsify it, and where each could first fire.**
1. **The issue does not reach the pump.** Trace frames 620, 640, 900 and
   1000: an `INFO 17` with a refusal. Or no `process_move_to` with
   `orders` 3 between 621 and 622 and between 641 and 642, and with 4
   between 901 and 902 and between 1001 and 1002.
2. **The class is not the selector's.** Block 622: `0/6` without one
   `EXPLORETOORDER` (type 3), `flags 5`, `orig` (2400, 17280). Block 642:
   any of `0/7`–`0/9` without one at its own slot, or with a
   `GROUPMOVEORDER`. Blocks 902 and 1002: the same for `FLEETOORDER`
   (type 4).
3. **The look does not fire.** No box leg (an `EXPLORETOORDER`, `flags 1`,
   to (1176, 15000)) in front of `0/6`'s re-issued explore by block 760;
   none on the squad by block 880. Its first possible block is the first
   fifteenth frame with the box seen and in the sweep.
4. **The look is a figure's, or it drops the walk.** A box leg on a
   follower that its captain does not share; or no re-issued `flags 5`
   explore behind the leg.
5. **The re-issue goes to the slot.** Behind the squad's box legs, a
   re-issued explore whose `orig` is not the click (12672, 14976), or
   whose points are not the slots laid out afresh round the click.
6. **The box is not opened.** No `Unit::explore_goody` draw on the frame
   `0/6` first stands in cell (1, 19), or the frame a squad member first
   stands in (16, 21).
7. **They do not arrive.** A stack not empty on block 900 (`0/6`), 1000
   (the squad), 1150 (`0/6`'s flee) or 1250 (the squad's flee).

**This crate's prediction**, walked from run215's start with the harness's
new `@explore` and `@flee` (a scratch walk; run215 is the same game to
610). The harness takes both verbs through `crate::input::group_move_to`
with `orders` 3 and 4, which the crate already carried.
- **The chariot**: an `EXPLORE_TO` to (2424, 17304) on 622. The box leg
  to (1176, 15000) on 685, re-issued each fifteenth frame. The box opens
  on 731 and the chariot stands on its centre by 748. It re-walks the
  explore and stands on (2424, 17304) on 839.
- **The squad**: three `EXPLORE_TO`s on 642, to (12696, 15000), (12552,
  15048) and (12792, 14904), and no group. All three get the box leg on
  804, the figures with their captain. The box opens on 856, and they
  stand by 950.
- **The flees**: a `FLEE_TO` on 902 and three on 1002. They stand on
  1095 and by 1190.

**Where it should part.** The crate's `group_finish_insert` re-issues a
plain move to its snapped `dest`, since the crate does not carry a plain
move's `orig` (a named seam). For `0/6` the two are the same point. For
the squad the group re-forms round the leader's slot, (12696, 15000),
where the reading says the click. That puts `0/8` on (12600, 15096)
against (12552, 15048), so the first parting expected is the squad's
walk back from its box, after 856. The crate's `calc_cost` triples a
flee's extra on `flags & 2`, a bit nothing writes (`docs/ORDERS.md`
§1.3), not on `FLEE_TO`. The scratch walk is the same with the predicate
corrected, so this staging cannot reach it (falsifier 7 cannot see it).

**Run 2026-09-25 as run219 (item 738)** (`docs/RUNS.md` has the tables).
- **No falsifier fired.** All four commands reached the pump on the next
  frame with their selector, `process_move_to … 3 0` and `… 4 0`.
- **The premise holds, measured.** One `EXPLORETOORDER` on 622 and 642
  and one `FLEETOORDER` on 902 and 1002 a member, `flags 5`, `orig` the
  click. The squad's are at their own slots, and there is no group order
  on any of the four.
- **The look**, on the frames this crate predicted. `0/6` takes its box
  leg on 685 and opens the box on 730; the squad takes its leg on 804,
  every member with its captain, and opens the box on 855. Each opening
  is three `explore_goody` draws. The leg is re-aimed every fifteenth
  frame while it walks (`0/9` on 819 and 834).
- **The re-issue goes to the click.** Behind each box leg the re-issued
  explore carries the member's first point and `orig`. The squad's slots
  are laid out afresh round (12672, 14976), and `0/6` ends on (2400,
  17280), its `orig`, where the plan's goal is.
- The squad stands on 936–947; the flees on 1098 and 1175–1183.
- **The pool printed**: 330,240 `GROUPDATA` at `GUYS=4`.

**Where this crate parted: `GOLDEN_WORD_CHAPTER_SIXTEEN` = 838, open.**
The harness took both selectors from the start, and every class, box leg
and box opening agreed on its frame. The parting was the one predicted,
the re-issue's point. On 838 the original's chariot stands on its click
and goes idle, two `Guy::set_anim+0x97a < Unit::do_idle+0x7d` rolls,
7 draws against 5 at draw 0; this crate's walked on to the snap.
- **The widening** over (605, 840) names it first on 685: `0/6`'s plan
  goal, (2424, 17304) here against (2400, 17280). From it follow the
  waypoint on 779, a one-unit drift on 793 and 797, and the arrival. On
  804 the squad's replay was laid out round the leader's slot, `0/8` to
  (12600, 15096) against (12552, 15048), and 819's re-aim carried it.
  71 keys, and the births' `form`.
- **The pool's widening** adds the replay's group point, `ox/oy` on the
  box leg's slot, and `speed`/`new_speed` on 685 and 804.
  `Groups::get_open_slot@006fa460` asks each live slot it walks past for
  `get_num`, which normalizes a seated group of fewer than four and
  re-seats its speed; this crate does not model the slot allocation.
- **The coverage pin** needed nothing new; its driver takes 622, 642,
  685, 730, 804, 838, 902 and 1002.

**Closed at 1250 by the same item** (`docs/ORDERS.md` §31). A plain
move's `orig` is carried, the click from `action_move_near`'s plain arm,
and `finish_insert` replays to it. That alone moved the word 838 → 1250,
run219's trace end: sequence 1250, no value part. The widening over run219
whole leaves the births' `form` and the pool's six `speed` rows.

**Not reached by this staging**: the flee's three readers. The crate's
`calc_cost` triples a flee's extra on `flags & 2`, a bit nothing writes,
where the original reads `order_type() == FLEE_TO`; the scratch walk with
it corrected was identical, because neither flee path crosses a seen
cell whose extra is above 0. A flee over seen rough ground, or past an
enemy's danger, would reach it.

## 25. Chapter seventeen — the flight line, an issuer the AI rarely takes (item 746)

**Premise.** `CommandManager::issue_flight@00941d40` makes **one
`StrafeOrder` (type 16) an aircraft, never a bare `AirOrder`**. `AirOrder`
has no `OrderIndex` of its own. It is a base of `StrafeOrder : AttackOrder,
AirOrder`, of `AirPatrolOrder` and of `AirAttackGroundOrder`, and the
dump prints it as the `AIRORDER` block inside a `STRAFEORDER`
(`StrafeOrder::log_data@0047fd80`, `AirOrder::log_data@0047fa40`).
- A flight to one's own Airbase (`MOVE_TO`) is a target-less strafe home,
  `returning 1`.
- A flight at an enemy (`ATTACK`) re-points a strafe already flying.
- **An aircraft `add` placed on the ground takes no strike at all.**

Three of the last four premises died on the class, so the emulator went
first. `tools/gamelog/golden/chapter17.cmd` has the reading, citation by
citation.

**The issuer, under the emulator first**, on
`tools/explore/command_oracle.py`'s fixture widened to who=0's objects 6
and 7 as captains (a scratch script in the job's tmp dir).
- **One call appends a 25-byte `flight`** (type 0x1c, `[ox][whom][shift]
  [ctrl][alt][orders]`) behind a fresh `group`: 5 bytes for one aircraft
  (30 in all), 7 for the pair (32). A repeated selection is the 3-byte
  reuse (28).
- `MOVE_TO` (1) and `ATTACK` (10) differ in the `orders` word alone. The
  issuer tests neither the target nor the aircraft, and a target of −1
  rides through.
- A non-captain is dropped. `use_mp_playback`, `semaphore & 0x10` and
  `semaphore & 4` each append nothing.
- **It writes** the package and the selection caches and nothing else,
  as in §17 and §24: no order, no unit, no draw.
- **What the emulator cannot reach** is the class:
  `CommandPackage::process_flight@00947db0` → `Group::action_flight@
  006fb260` → `Unit::add_strafe_order@005e48c0`, at process time.
- **The DLL's two new verbs.** `@flight` and `@strike` call `issue_flight`
  with `MOVE_TO` and `ATTACK`. These are what `Console::execute_at_cursor@
  007c6630:2849` and `:2835` pass through `GroupOut::issue_flight@
  00708b10`: a right-click on one's own base, and a right-click on an
  enemy. Both compile `-Werror`, plain and under `RON_AUTOSTART`.

**The reading** (`chapter17.cmd` has it whole).
- **`action_flight`'s member loop** skips a member that already stands in
  the base.
  - A member already on a `STRAFE` has that order re-pointed. `ATTACK`
    writes the target, and with fuel left `returning 0`, `mandatory 1`
    and the action bit.
  - Any other member needs to be "inside" for `ATTACK`: the home of an
    `AIR_PATROL` or `AIR_ATTACK_GROUND`, or else
    `ObjectData::get_inside@00651a80`, which is −1 for an aircraft `add`
    placed. `MOVE_TO` does not need it.
- **The home.** `MOVE_TO` gives `add_strafe_order(−1, −1, base, who, 1,
  QUEUE_NEW, 1)`: a strafe with no target, `returning 1`, `cruising_alt`
  0x640, home `oxx/whose` the base.
- **The flight** is `Unit::do_strafe@005eab00` → `Unit::do_air_physics@
  005e86d0`.
  - On every eighth frame, `(o + frame) & 7 == 0`, a **non-bomber**
    redraws `cruising_alt` from `Random::get(0, 0xffff)`. A Bomber
    (`is(BOMBER)`) does not.
  - `Unit::check_fuel@005e9be0` aims a returning plane at its base.
  - `Unit::land_plane@005e9950` clears the strafe and adds a
    `SpecialAnimOrder` (type 25) on the base, which ends in `go_inside`.
- **The tank.** `mana_burn` climbs one a frame outside and refills 2
  inside. `check_fuel` sets `returning` when the tank is empty: the
  Bombers on **1212** and **1214**.
- **An idle plane in its base stays there.** `Object::do_launch@0064f3b0`
  launches only a plane with an order and a full tank.

**The cast**, on chapter nine's open ground north of Napata (BASELAND,
owner −1, no border). No enemy is within twelve tiles of the pad.
- who=0's Airbase at tile (60, 72), **`0/2007`**: the id run175 gave
  who=0's first staged building.
- a Fighter `0/6` on the pad at tile (60, 84);
- a Bomber pair, `0/7` at (52, 84) and `0/8` at (68, 84);
- who=1's Barracks at tile (110, 86), **`1/2006`**, ~42 tiles from the pad.

**Lines.**
- `0 !ai off`; `600 library who=0 6` for the Modern age.
- `606 add airbase who=0 60,72`; `610 add fighter`, `612` and `614 add
  bomber`, all `who=0`; `616 add barracks who=1 110,86`.
- `620 @strike 0 2006 1 7 8`: the pair on the ground, at the enemy.
- `640 @flight 0 2007 0 6`: the Fighter home to its base.
- `660 @flight 0 2007 0 7 8`: the pair home.
- `664 @strike 0 2006 1 7 8`: the pair, now flying, at the enemy.

A call on trace frame F is on block F+2 (§17).

**The capture must dump** `end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1`
and `misc:COMMANDMANAGER=1` over `[605, 1400)`, beside run105's `start:`
set: run215's levels, the line whose pool printed (parked 733). The window
reaches the pair's landing after their tanks run dry.

**The premise's killer, and its writers** (§3, point 5).
- **The order stack of each commanded aircraft on its processed block**:
  anything but one `STRAFEORDER` where one is predicted, or any order where
  none is.
- The class's writer on this path is `add_strafe_order`'s
  `get_obj(STRAFE)`. Its target and `returning` are rewritten by
  `action_flight`'s STRAFE arm, by `do_strafe` and by `check_fuel`.
  - `do_strafe` turns a dead target with a valid point into an
    `AirPatrolOrder` over it, and re-targets a live one `QUEUE_FIRST` by
    `find_new_bomber_target` every sixteenth frame.
  - The one other air strike, `add_air_attack_ground_order`, needs a
    missile's `0x8000000` and is not reached.
- **The loops' bounds.** `action_flight`'s member loop runs to
  `group.num`: 1 and 2. `check_fuel`'s base search runs over `objects`
  2000..`[who]+0x184` and 0..`[who]+0x15c`. `do_launch` walks its chain
  to the first −1.

**What would falsify it, and where each could first fire.**
1. **The issue does not reach the pump.** Trace frames 620, 640, 660 and
   664: an `INFO 17` with a refusal, or no processed `flight` on the next
   frame (`COMMANDMANAGER`).
2. **An unbased aircraft takes a strike.** Block 622: `0/7` or `0/8` with
   any order.
3. **The class is not a strafe, or not home.** Block 642: `0/6` without
   exactly one `STRAFEORDER` (type 16), target −1, `mandatory 1`, the
   action bit, and an `AIRORDER` naming `0/2007` with `returning 1`.
   Block 662: the same for `0/7` and `0/8`.
4. **The strike is a new order, or not the target's.** Block 666: either
   bomber with a second order, a target other than `1/2006`, `returning`
   not 0, or a home other than `0/2007`.
5. **The aircraft does not fly.** From 642 and 662: a pad never left, or
   an air altitude that stays 0.
6. **The landing is not the base's.** No `SPECIALANIMORDER` on `0/2007`
   before `inside_up` reads 2007; `0/6` not inside by block 760.
7. **The strike does not reach the point.** No damage on `1/2006` by
   block 900.
8. **They do not come home.** `returning` not set on the pair by 1230;
   `0/7` and `0/8` not inside by 1400; fewer than three planes inside
   `0/2007` at the end.

**Where it should part.** This crate does not enter the flight command:
the harness skips `@flight` and `@strike` by name. The Fighter's first
`cruising_alt` draw would be block 642, `(6 + 642) & 7 == 0`, if
`do_strafe` runs on the processed frame. So the first parting expected
is 642: a draw the original spends and this crate does not, with the
Fighter's order stack beside it.

**Run 2026-09-25 as run223 (item 746)** (`docs/RUNS.md` has the tables;
the second take, since the first stalled in DXVK's device setup).
- **The premise's class holds.** All four commands reached the pump on
  the next frame (`process_flight 621`, `641`, `661`, `665`). On 642 and
  662 each aircraft holds exactly one `STRAFEORDER` (16): target −1,
  `mandatory 1`, flags 4, `AIRORDER` `oxx 2007 whose 0 cruising_alt 1600
  returning 1`, `xx/yy −1`. **No bare `AirOrder`, anywhere**; the class
  that carries it is the strafe.
- **The strike from the ground took nothing** (falsifier 2 did not fire):
  no order on either bomber on 622.
- **Falsifier 4 fired: the flying strike became a patrol.** On 666 each
  bomber holds one **`AIRPATROLORDER` (17)** over (21120, 16512), the
  Barracks' point, home `0/2007`, `returning 0`, flags 0. The STRAFE arm
  did re-point the order, since the patrol carries its point.
  `do_strafe`'s first step then found `valid_target` false and took its
  dead-target arm: `kill_current_order` and `add_air_patrol_order(xx,
  yy, home)`. The Barracks is unseen by who=0 on 665. **So a player's
  strike at an enemy it cannot see is a patrol over the point, and the
  patrol's own search takes the target once seen**: a `STRAFEORDER` on
  `1/2006`, `mandatory 0`, no action bit, current in front of the patrol
  on 777 (`0/8`) and 778 (`0/7`).
- **At the point.** Bombs land from 822 (damage 285 on 850, 951 on
  1000); the Barracks is gone on **1080**, the strafe drops on 1081 and
  the patrol resumes.
- **Home again.** The Fighter flies 642–721 (guy `z` to ~550;
  `cruising_alt` redrawn 1600 → 1300 on 643, 1500 on 659) and is inside
  `0/2007` on **722** with an empty stack, refuelled 2 a frame to 0 by
  778. **No `SPECIALANIMORDER` is ever dumped** (falsifier 6 fired in
  its letter): the landing's animation starts and ends inside one frame.
  The pair's tanks run dry on 1212 and 1214 (`returning 1`, as read),
  and both are **still flying home on 1399** (falsifier 8 fired on the
  landing). The base's count at the end is one, `0/6`.

**Where this crate parted: `GOLDEN_WORD_CHAPTER_SEVENTEEN` = 642, open.**
As predicted, 642 is the first frame of the Fighter's strafe home. The
original spends its `cruising_alt` redraw, `Random::get` at
`Unit::do_air_physics+0xba` (`0x5e878a`): 7 draws against 5 + 1, parting
at draw 0. The first pin had the harness skip both verbs by name. Its
widening over (605, 645) named the pushed selections, the pair's `group`
on 622, and the Fighter standing with no order on 642.
- **The command entered** (item 746, `docs/ORDERS.md` §32):
  `input::group_flight` → `Sim::group_action_flight`, and the strafe's
  row compared.
- On the widening, now (605, 667), every `STRAFEORDER` row agrees on 642
  and 662, the pushes with them.
- **What stands is the flight**:
  - each aircraft's first step on its processed block;
  - the Fighter's redraw on 643 (1600 → 1300);
  - the strike turned `AIRPATROLORDER` on 666, a class this crate has no
    body for;
  - past the word, the one-draw shift in the citizens' and the scout's
    rows;
  - in the pool, the pushed groups' point, (0, 0) there and (−1, −1)
    here, and item 738's `get_num` speeds.

**What moving the word takes is the flight**, bigger than an item and
listed in `docs/ORDERS.md` §32:
- the `cruising_alt` draw with the climb (`pitch_aircraft`) and the step,
  built onto the bird's bank in `crate::air`;
- `check_fuel`'s approach, `land_plane` and `go_inside` for a plane, and
  the tank;
- the `AirPatrolOrder` a dead or unseen target becomes, with the patrol's
  search and its bombing.

A draw alone would move the word with the positions already parted, which
is the trap DECISIONS 42 names, so it was not taken.

**The flight home, flown: `GOLDEN_WORD_CHAPTER_SEVENTEEN` = 805, open**
(item 759, `docs/ORDERS.md` §33). `do_strafe` takes a flight home into
`do_air_physics`: the redraw, `check_fuel`'s approach, the returning
bank, `pitch_aircraft`, the step and `land_plane`. The figure's bank,
pitch and altitude are parsed and compared now.
- **The value diff on the old word's frame, 642**: every aircraft
  agrees whole. `0/6` stands at (11664, 16262), heading 1419725301,
  bank 10, pitch 2, altitude 2, path point (11424, 15089), on both
  sides. `0/7` and `0/8` are still on their pads, (10104, 16248) and
  (13176, 16248), altitude 0 on both.
- Every redraw 642–714 agrees, as does the figure draw under
  `do_air_physics+0x683` on 705. Its whole flight agrees to its landing inside `0/2007` on 722,
  and inside after it. The pair's flight home agrees on 662–665. The
  citizens' one-draw shift past the old word is gone.
- **The word is the first bomb**, 805. The original spends
  `Guy::set_anim+0xf2f < Unit::set_anim+0x56 < Unit::do_strafe+0x9d0`,
  5 draws against 4 at draw 0: the patrol's strafe on `1/2006` going to
  `CHAR_ATTACK2`.
- **The first value parting is 666, before the word**: the pair's strike
  turned `AIRPATROLORDER`, which this crate has no body for (§32 piece
  5). Its flight spends no draw until the bomb, so the stream agrees
  across a stretch where the pair's points do not. The widening, now
  (605, 807), names every row from 666. **666 is the chapter's next
  frame.**

**The pair's patrol, flown: `GOLDEN_WORD_CHAPTER_SEVENTEEN` = 821, open**
(item 763, `docs/ORDERS.md` §34). `do_strafe` turns the strike on the
unseen Barracks into an `AirPatrolOrder` over its point and `work` flies
it in the same frame; the patrol flies `do_air_physics`' non-returning
arms; a plane's step lights the fog, so the Barracks is first seen
between the searches on 760 and 776; the search pushes the strike
`QUEUE_FIRST`, with no `update_action`; `0/8` releases on 805.
- **The value diff on 666, both sides**: `0/7` at (10319, 16351),
  heading 1315604981, `z` 36; `0/8` at (13370, 16389), heading
  1547706549, `z` 36; each one `AIRPATROLORDER` over (21120, 16512),
  home `0/2007`, flags 0.
- **On the old word's frame, 805** (block 806), both sides: `0/8` at
  (20573, 16573), `z` 1632, `recharging 31`, `cur_anim 12`, strike in
  front of patrol; `0/7` at (17838, 16573), `z` 1674, still closing.
- **The word is the bomb's landing**, 821: the original spends
  `Object::take_damage+0xe1 < Object::do_damage < Ammo::do_damage`, 5
  draws against 4 at draw 0. **The first value parting is its own
  block**, 822: the Barracks' `damage` 45 (`damage_frac` 14) there, 0
  here — compared by the widening since this item. The round is the
  release animation's event, which `crate::anim` fires for an `ATTACK`
  or `ATTACK_GROUND` front order only.

**The strafe's round: `GOLDEN_WORD_CHAPTER_SEVENTEEN` = 1400, closed**
(item 770, `docs/ORDERS.md` §35). A strafe with a target releases: the
bomb leaves one of the Bomber's two bays at its altitude less 19, lands
a tile ahead along the heading with no draw, and falls in 17 frames.
The stream agrees to run223's end, 1400, with no value part.
- **The value diff on the old word's frame**, block 822, both sides:
  the Barracks at `damage 45`, `damage_frac 14`. Its damage agrees on
  every block to its death on 1080, every fringe hit included.
- **The widening is run223 whole**, (605, 1401). What stands past the
  births is **the tank** (parked 765): `returning` is 1 there and 0 here
  on `0/7` from 1212 and on `0/8` from 1214, and each plane's flight
  home after it. No draw follows from it to 1400.
- **run235** is this game again at `AMMO=5`, to 1100. All 49 bombs
  agree field for field on 800–1100, except the target a round in
  flight keeps when the Barracks dies (chapter three's family).

## 26. Chapter eighteen — the build line, an issuer the AI takes through its own planner (item 779)

**Premise.** `CommandManager::issue_build@00941c30` on a human's citizens
places **one site, paid once**, and gives **each citizen a `MOVEORDER`
then a `BuildOrder` (type 6, flags 4) on it** — the approach is a plain
move for a human, where the AI's is an `EXPLORETOORDER`. The AI's
citizens reach `BuildOrder` through its planner (`Leader::
create_buildings`, `crate::ai_place`) and never through this command, so
this crate carries the construction and not the command. Three of the
last six premises died on the class, so the emulator went first.
`tools/gamelog/golden/chapter18.cmd` has the reading, citation by
citation.

**The issuer, under the emulator first**, on
`tools/explore/command_oracle.py`'s fixture widened to who=0's objects
6–9 as captains and 5 as a non-captain (a scratch script in the job's
tmp dir).
- **One call appends a 25-byte `build`** (type 0x19, `[x][y][x2][y2]
  [type][queued]`, each as passed) behind a fresh `group`: 5 bytes for
  one citizen (30 in all), 9 for three (34). A repeated selection is the
  3-byte reuse (28); a non-captain is dropped.
- `use_mp_playback`, `semaphore & 0x10` and `semaphore & 4` each append
  nothing. The issuer tests neither the point, the type nor the
  builders.
- **It writes** the package and the selection caches (`last_who_sent`,
  `last_num_sent`, the objects and uids lists) and nothing else, as in
  §17 and §25: no site, no price, no order, no draw.
- **What the emulator cannot reach** is everything the chapter measures:
  `CommandPackage::process_build@00948110` → `Group::action_build@
  00707510` → `Group::action_swarm_around@0070fbe0`, at process time, on
  the game's state.
- **The DLL's new verb.** `@build <who> <x> <y> <type> <o>…` calls
  `issue_build(group, x, y, x, y, type, QUEUE_NEW)`, what
  `Options::picked_spot@00721c40:531` passes through
  `GroupOut::issue_build@00708c60` for an unmodified drop with no drag.
  `action_build` reads only the first point. It compiles `-Werror`, plain
  and under `RON_AUTOSTART`.

**The reading** (`chapter18.cmd` has it whole).
- **The site.** `action_build` needs the group on the map,
  `GroupData::validate_build@00708620` (snap, `blocked_site` clear, the
  price affordable), `num_valid > 0`, the city limit for a city and
  `type_avail(type, 1) == 4`. Its `PathData` stack holds one point and
  is popped once: `snap_center`, `blocked_site`, the price paid, and
  `Objects::init_build(who, type, x, y, 0, −1)`.
- **The builders.** `action_swarm_around(site, who, QUEUE_NEW, BUILD_AT,
  1)` walks the members twice (land, sea; air never) to `group.num`. Each
  citizen gets a ring spot, `find_nearby_spot(site, R)` with `R =
  min(xs, ys) × 0x60 + 0x30`, nudged 0x30 off the site. Then
  `add_move_facing_order(spot, facing, local_40, 0, QUEUE_NEW, 0, …)` and
  `add_build_order@005e5210(site, who, QUEUE_LAST, 1)`. For `BUILD_AT`,
  `local_40 = ~(leader_flags >> 1) & 2 | 1`: **1, `MOVE_TO`, for a
  human** (`leader_flags & 4`, `LeaderData::is_human`'s whole body, read inline) and 3,
  `EXPLORE_TO`, for a computer.
- **The walk and the build** are `Unit::do_build@005eebf0` (`docs/
  ORDERS.md` §5.2): not adjacent, re-swarm at `QUEUE_FIRST` with the same
  `local_40`; adjacent, `CHAR_BUILD` and `ACCEL_CONSTRUCT` a frame into
  `Wall::do_construct`.
- **Finished**, a non-gather building's builder goes to
  `Unit::build_done@00603bf0` (§5.5): a human of stance 1 tries
  `find_build_spot`, then `find_gather_spot`.

**The cast**, on who=0's open ground south-east of Napata (run223's
start `WORLD`: cells x 5–10, y 44–50, BASELAND owned by 0).
`library who=0 2` holds The Art of War, so both types are available; on
619 who=0 holds 240 timber, 113 wealth and 100 metal.
- `0/6` at tile (24, 184); `0/7`, `0/8`, `0/9` at (24, 196), (26, 196)
  and (28, 196).
- `0/6` drops a **Barracks** (427, 120 timber) at (7296, 34176):
  **`0/2007`**.
- The three drop a **Siege Factory** (430, 60 timber and 60 metal) at
  (7296, 36864): **`0/2008`**. A second Barracks would be 145 timber
  against 122 in hand.

**Lines.** `0 !ai off`; `600 library who=0 2`; `610`–`616 add citizen
who=0`; `620 @build 0 7296 34176 427 6`; `640 @build 0 7296 36864 430 7
8 9`. A call on trace frame F is on block F+2 (§17).

**The capture must dump** `end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1`
and `misc:COMMANDMANAGER=1` over `[605, 1450)`, beside run105's `start:`
set: run223's levels, the line whose pool printed (parked 733).

**The premise's killer, and its writers** (§3, point 5).
- **Each commanded citizen's order stack on its processed block**:
  anything but a `MOVEORDER` then a `BUILDORDER` (flags 4) on the new
  site.
- The class's one constructor on this path is `add_build_order`'s
  `get_obj(BUILD_AT)`. Its other callers are `check_build_order`,
  `Wall::process`'s AI recruiter and `come_out`, none reached by a staged
  human. The approach's class is `add_move_facing_order`'s switch on
  `local_40`. `do_build` and `check_build_order` kill the order.
- **The loops' bounds.** `action_build`'s stack holds one push;
  `action_swarm_around` makes two passes over `group.num`, 1 and 3;
  `find_nearby_spot`'s ring runs to `R`, 432 for a 4×4.

**What would falsify it, and where each could first fire.**
1. **The issue does not reach the pump.** Trace frames 620 and 640: an
   `INFO 17` with a refusal, or no `process_build` on 621 and 641.
2. **No site, or not paid.** Block 622: no `BUILDDATA` `0/2007` with
   `orig_type 427`, unstarted, or timber not down by 120. Block 642: no
   `0/2008` `orig_type 430`, or timber and metal not down by 60 each.
3. **The class is not a build, or the approach not a move.** Block 622
   for `0/6`, 642 for `0/7`–`0/9`: anything but a `MOVEORDER` (flags 0)
   then a `BUILDORDER` (flags 4) on the site. An `EXPLORETOORDER` says
   `local_40` is not the human flag.
4. **The walk does not end at the site.** No builder constructing
   (`frame_started` set, `construct_hits` rising) by block 760, or a
   second approach (a `QUEUE_FIRST` re-swarm) on any.
5. **The rate is not one builder's, or three's.** The Barracks under one
   not finished by 1200; the Siege Factory under three not by 1000.
6. **The builders do not let go.** A builder still holding its
   `BUILDORDER` past its site's last frame of construction.

**This crate's prediction**, from a scratch walk of the script stood up
on run223's start with a prototype of the command's entry: both sites
paid and placed on the processed frames; `0/6` constructing from ~735,
the Barracks finished ~1160 at one builder's rate; the three
constructing from ~720 and the Siege Factory finished ~955; then `0/8`
takes the Barracks by `find_build_spot` on ~1090.

**Where it should part.** This crate does not enter the build command:
the harness skips `@build` by name. So the first parting expected is
621: the command's processing, with `0/6`'s first step of the walk the
original spends and this crate does not, and on block 622 the site, the
price and the two orders.

**Run 2026-09-25 as run241 (item 779)** (`docs/RUNS.md` has the tables).
- **No falsifier fired, and the premise's class holds.** Both commands
  reached the pump on the next frame. On 622 `0/6` holds a `MOVEORDER`
  to its ring spot (6840, 34440) and a `BUILDORDER` (flags 4) on
  `0/2007`, and timber is down 120. On 642 each of the three holds a
  `MOVEORDER` to its own spot and a `BUILDORDER` on `0/2008`, and
  timber and metal are down 60 each. No `EXPLORETOORDER` anywhere on a
  builder: `local_40` is the human flag. Falsifier 3's "flags 0" is the
  adder's; on the processed block the move already carries the path
  bit of its first step.
- **The build.** The three start on 709 and finish the Siege Factory on
  **948**; `0/6` starts on 721 and finishes the Barracks on **1141**.
  Each builder's stack is empty on its site's last frame.
- **After.** On **1097** `0/8`, idle since 948, takes `find_build_spot`'s
  help: a `MOVEORDER` and a `BUILDORDER` with flags 0 on `0/2007`. It is
  still walking on 1141 and stands idle from ~1200.
- **The scratch walk's prediction held on every block read** except the
  help's class, an `EXPLORETOORDER` there: this crate's one-unit
  `swarm_around` never asks whose builder it is.

**Where this crate parted: `GOLDEN_WORD_CHAPTER_EIGHTEEN` = 642, open.**
The harness skips both `@build` lines, as §25's first pin skipped its
verbs. Nothing spends a draw on the lone builder's walk until 642, where
the original spends 6 draws against 7 here, parting at draw 0 on an
idle citizen's roll this crate spends (`Guy::set_anim+0x97a <
Guy::inc_time+0x271`) and the walking one does not. The widening over
(605, 645), both directions, names what stands: the births' `form`; on
622 the Barracks, which the dump holds alone, the 120 timber, and `0/6`
idle with no order; on 642 the Siege Factory, its 60 metal, and the
three idle; in the pool, each command's pushed selection.

**The build command entered: `GOLDEN_WORD_CHAPTER_EIGHTEEN` = 1450,
closed** (item 779, `docs/ORDERS.md` §36). `input::group_build` →
`Sim::group_action_build` places and pays once and swarms at
`QUEUE_NEW`, a `MOVEORDER` for a human's builder; the one-unit swarm
asks whose builder it is. The stream agrees to run241's end, 1450, with
no value part.
- **The value diff on the old word's frame**, block 642, both sides:
  `0/7` at (4751, 37744), `0/8` at (5136, 37748) and `0/9` at (5520,
  37746), each on its first step to (6840, 37032), (6840, 36792) and
  (7128, 37320) with a `BUILDORDER` (flags 4) on `0/2008`; timber 62,
  metal 40.
- **The widening is run241 whole**, (605, 1451). Past the births only
  the pool's `ox`/`oy` on each pushed selection stands: (0, 0) there and
  (−1, −1) here, as on §25's flight groups, a point no build reads.

## 27. Chapter nineteen — the cast line, a Spy's Informer on an enemy building (item 790)

**Premise.** `CommandManager::issue_spell@00941b80` on a human's Spy,
casting the **Informer** (`0x27f`) on an enemy Barracks, lays **a
`CastOrder` (type 14, flags 4)** through `Group::action_spell@006fe1a0` →
`Unit::add_cast_order@005e4a60`, and `Unit::do_cast@005ebfe0`'s
**targeted arm** — which no capture has reached (`docs/ORDERS.md` §6.9,
"What is not established") — pays it once in **mana**, walks the Spy to a
ring on the target with a `MOVEORDER` ahead of the cast, faces and holds
it for the craft's forty frames, and sets the target's `infiltrated`. This
crate kills a targeted order on its first frame. Four of the last seven
premises died on the class, so the emulator went first, twice.
`tools/gamelog/golden/chapter19.cmd` has the reading, citation by citation.

**The issuer, under the emulator**, on `tools/explore/command_oracle.py`'s
fixture (a scratch script in the job's tmp dir): one call appends a
21-byte `spell` (type 0x17, `[ox][whom][type][x][y]`, each as passed)
behind a fresh `group`, 26 bytes in all, or the 3-byte reuse (24).
`use_mp_playback`, `semaphore & 0x10` and `semaphore & 4` each append
nothing. It writes the package and the selection caches: no order, no
price, no draw, as in §17, §25 and §26.

**The cast's predicates, under the emulator on the original's own
state** (run246: a `RON_STATE_FRAME=619` packet of this script without its
`@spell` line, 59 s; `tools/recomp/step4.py`'s machinery, scratch).
Without the DLL's verb, it answers which crafts the Spy may cast on the
Barracks:

| craft | `is_castable(·, 0/6, 0, 0)` | `is_valid_target(·, 0, 2006, 1)` | `get_range` | `get_job_time` |
|---|---|---|---|---|
| Bribe `0x275` | 3 | 0 | 192 | 100 |
| Counterintelligence `0x277` | 3 | 0 | 192 | 38 |
| Informer `0x27f` | 3 | **1** | **960** | **40** |

`can_pay_cost` answers 10 for each; the Spy holds `mana` 1000 and
`mana_burn` 491 (born at `mana / 2`, `Unit::init@00612100`, and one back a
frame). Then **`Group::action_spell` and one `Unit::do_cast`, run on the
packet** with a pool `Group` of `[6]`: a **`CastOrder` (type 14)**, the
Spy's target fields `+0xa2` 2006, `+0xa8` 1, `+0xa6` its uid, `spell_time`
0; then `paid` 1, `mana_burn` 991, `Type::pay_cost@006681f0` rewriting
every bucket of who=0 **unchanged** (254 food, 240 timber, 113 wealth, 100
metal, stored `^ 0x8221`) although the row prints `COST 20g/20w`, and a
**`MOVEORDER` pushed ahead of the cast** to **(14232, 15528)**. What the
emulator cannot reach is the frames after the first: the walk, the
in-range half and the cast.

**The reading** (`chapter19.cmd` has it whole).
- **The order.** `action_spell` runs `validate_spell` again; the craft
  carries `m` (0x1000), so the member with the most `mana_left` casts;
  `mana_left + pending ≥ MANA` (500); `can_pay_cost`; `is_valid_target`;
  the target onto the unit; `add_cast_order(ox, whom, x, y, 0x27f,
  QUEUE_NEW, 1)` — no `g` (0x40), so not QUEUE_FIRST.
- **The walk.** `do_cast`'s targeted arm pays once, re-reads the target,
  and out of `get_range + radius` (960 + 384 for a 4×4) queues
  `add_move_order(spot, QUEUE_FIRST, tolerance 1344)` to a
  `find_nearby_spot` ring at 1,152–1,296.
- **In range.** The target must answer `is_seen(0, 0)`; the Spy takes
  `flags |= 0x80` and `visible |= 1 << 1`, faces the target, re-sets
  `CHAR_ATTACKWALK` (0xa) every frame and `unit_masks |= 0x20000` once;
  the craft's `l` (0x800) keeps the cloak. On the fortieth in-range
  frame `SpellType::cast` → `cast_double_agent@00673a80`: the target's
  `infiltrated |= 1`, its `update_seen(0)`, then `kill_current_order`,
  which clears 0x20000. Mana recovers one a frame only while
  `unit_masks & 0x2a000` is clear (`Unit::process@00610bc0`).

**The cast**, on neutral open ground (chapter seventeen's arena, cells
x 12–23, y 17–22 of run241's start `WORLD`), both sides Medieval by
`library`: a who=1 **Barracks** at tile (80, 80), **`1/2006`**, at
(15360, 15360); a who=0 **Spy** at tile (60, 82), **`0/6`**, at (11640,
15864); and on 620 the Informer, picked at the Barracks' own point.

**Lines.** `0 !ai off`; `600 library who=0 2`; `602 library who=1 2`;
`606 add barracks who=1 80,80`; `610 add spy who=0 60,82`; `620 @spell 0
639 2006 1 15360 15360 6`. A call on trace frame F is on block F+2 (§17).

**The capture must dump** `end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1`
and `misc:COMMANDMANAGER=1` over `[605, 1100)`, beside run105's `start:`
set.

**The premise's killer, and its writers** (§3, point 5).
- **The Spy's order stack on block 622**: anything but a `MOVEORDER` then
  a `CASTORDER` (spell 639, `paid` 1).
- The class's constructor on this path is `add_cast_order`'s
  `get_obj(CAST_SPELL)`. Its other callers are `think_spellcaster@
  005f27a0` — a human's QUEUE_FIRST Counterintelligence on a valid target
  in range, and none is staged: no enemy Spy, no infiltrated object of
  who=0's — `think_fish` and the unpack and transport arms, none reached
  by a staged Spy. `do_cast` kills the order (a target no longer valid, an
  unseen target in range, the cast), as does any QUEUE_NEW order.
- **The loops' bounds.** `action_spell`'s member loops run over
  `group.num`, 1; `find_nearby_spot`'s ring to 1,296.

**What would falsify it, and where each could first fire.**
1. **The issue does not reach the pump.** Trace frame 620: an `INFO 17`
   with a refusal; or no `process_spell` on 621.
2. **The class is not a cast.** Block 622: `0/6` holding anything but a
   `MOVEORDER` then a `CASTORDER` (spell 639, `paid` 1, x 15360, y 15360,
   on `2006`/1). No order says `action_spell` refused.
3. **The price is not the mana.** Block 622: `mana_burn` not up by 500
   (about 989), or a bucket of who=0 down — a 20-wealth, 20-timber drop
   says `get_cost` charges the row's `COST`.
4. **The walk is not to the ring.** Block 622: the `MOVEORDER` not to
   (14232, 15528); or `spell_time` rising before the Spy is within 1,344
   of the Barracks, or not rising by block 800.
5. **The cast does not land, or costs the Spy.** Forty frames after the
   first in-range frame: `1/2006`'s `infiltrated` not 1, the Spy's stack
   not empty, or the Spy gone.
6. **The cloak breaks.** Any block: `unit_masks` taking 0x1000 or
   0x10000. The first in-range block: no 0x20000, `visible` not taking
   0x2, `flags` not taking 0x80; or 0x20000 still set after the cast.
7. **The mana does not wait.** `mana_burn` falling while 0x20000 is set,
   or not falling after the cast.

**This crate's prediction.** The emulator's frame on the packet stands in
for the scratch walk (parked 794): it is the original's own `action_spell`
and first `do_cast` on this staging, and it found neither a refusal nor a
price. From the Spy's `MOVES 21` (about 28 units a frame) the walk of
~2,600 units ends near block 715, the cast near 755.

**Where it should part.** This crate does not enter the spell command: the
harness skips `@spell` by name. So the first parting expected is 621–622:
the command's processing, the order, the price and the Spy's first step,
which the original spends and this crate does not.

**Run 2026-09-25 as run245 (item 790)** (`docs/RUNS.md` has the tables).
- **No falsifier fired, and the premise's class holds.** The command
  reached the pump on 621 (`process_spell 639 2006 1`). On 622 `0/6`
  holds a `CASTORDER` (flags 4, `ox 2006 whom 1`, `x 15360 y 15360`,
  `paid 1`, `spell 639`) and, at its head, a `MOVEORDER` to (14232,
  15528) — the emulator's spot. The dump lists the cast first, the order
  laid first. `cavarch_o/uid/who` read 2006, 12, 1.
- **The price is the mana.** `mana_burn` 489 → 988 on 622; no bucket of
  who=0 falls.
- **The walk and the cast.** The Spy walks ~20 units a frame and stands
  on the spot on **755**; `spell_time` climbs 1 … 39 on 756–794, and on
  **795** `1/2006`'s `infiltrated` is 1 and the Spy's stack is empty. It
  stands idle there to 1099.
- **In range.** `unit_masks` takes 0x20000 on 756 and loses it on 795;
  never 0x1000 or 0x10000 — the craft's `l` keeps the cloak. `visible`
  takes 0x2 on 756 and loses it on 827; `flags` takes 0x80 on 756 and
  loses it on 796. `mana_burn` stands at 854 from 756 to 795 and
  recovers from 796.
- **The prediction** from the Spy's `MOVES` was a walk to ~715 and a cast
  near 755; the Spy walks slower, 20 units a frame, and both came forty
  frames later.

**Where this crate parted: `GOLDEN_WORD_CHAPTER_NINETEEN` = 669, open.**
The harness skipped the `@spell` line. Nothing spends a draw on the Spy's
walk until 669, where the original spends 7 draws against 9 here, parting
at draw 0 on an idle Spy's roll this crate spends (`Guy::set_anim+0x97a
< Guy::inc_time+0x271`) and the walking one does not. The widening over
(605, 672), both directions, names what stands: the Spy's birth `form`;
on 622 its stack, empty here, with the action point and group; on 623 its
first step; in the pool, the command's pushed selection.

**The spell command entered: `GOLDEN_WORD_CHAPTER_NINETEEN` = 1100,
closed** (item 790, `docs/ORDERS.md` §37). `input::group_spell` →
`Sim::group_action_spell` lays the cast; `crate::cast` walks, holds and
casts it. The stream agrees to run245's end, 1100.
- **The value diff on the old word's frame**, block 669, both sides: `0/6`
  at (12551, 15616) on its walk to (14232, 15528), a `CASTORDER` (flags 4,
  `paid` 1) behind the `MOVEORDER`, `mana_burn` 941.
- **One row fell to a reading on the way**: on 756 the figure's `stopped`
  was a frame late here. `Unit::set_angle(angle, target, 1)`'s third
  argument writes guy 0's `angle` and `last_angle` outright
  (`Guy::set_angle@005d9010`), so the figure stands facing the target on
  its first frame in range.
- **The widening is run245 whole**, (605, 1101): past the births nothing
  stands but the pool's `ox`/`oy` on the pushed selection, (0, 0) there and
  (−1, −1) here, as on §25's and §26's. `run245_s_cast_is_the_original_s_
  field_for_field` reads the cast's own fields raw on every block —
  `mana_burn`, `spell_time`, `cavarch_o`/`cavarch_who`, the started bit,
  `visible`, the Barracks' `infiltrated`, the order's target, point, craft
  and `paid` — 46,540 rows, none parted.

## 28. Chapter twenty — the board line: the transport toggle, and the move it gates (item 803)

**Premise.** The player's transport command, issued through the original's
own issuer, **is a toggle and makes no order**. It sets or clears each
member's auto-transport bit (`unit_masks & 0x800000`, `docs/TRANSPORT.md`
§3.4), and that bit alone decides what a move across water does at the
shore: with it, the step onto the water becomes a **Transport cast**
(`CASTORDER`, craft 650) and a barge; without it, the unit stops on the
shore. The booked premise was §13's row, `BoardOrder` and
`AwaitBoardOrder` through `issue_set_transport`. The reading kills that
before the run: neither class is built by anything a game reaches, and
this chapter measures their absence beside the toggle's effect.
`tools/gamelog/golden/chapter20.cmd` has the reading with its citations.

**The issuer, under the emulator first.** A scratch script ran on
`tools/explore/command_oracle.py`'s fixture.

- **`CommandManager::issue_set_transport@00941910(group, flag)` appends
  10 bytes**: the 5-byte `group` and a 5-byte `set_transport`, type
  `0x0e`, `[flag i32]` (`docs/COMMANDS.md` §3), flag 1 and 0 as passed.
  The same selection again appends the 3-byte reuse (8 bytes).
- **`GroupOut::issue_set_transport@0070ad10`**, the UI's wrapper, appends
  the same 10 bytes, and nothing under `semaphore & 0x10`; the manager's
  form appends nothing under `use_mp_playback`.
- **It writes** the package's size and data and the selection caches, and
  nothing else: no unit, no order, no draw. Its prologue is `55 8b ec 83
  ec 08 b9 60 ff e8 00`, `sub esp, 8` for the 5-byte command.
- **Its callers.** `Options::do_transport@0071c500` passes
  `!GroupData::can_transport()` for the transport button, so a press flips
  the selection. `Options::picked_spot@00721c40`'s `OPTION_DISEMBARK`
  passes 1 and then an `issue_move_to` with `disembark 1`. The DLL's
  `@settransport <who> <flag> <o>…` passes the flag it is given.
- **What the emulator cannot reach**: `process_set_transport@00948f60` →
  `Group::action_set_transport@007024b0`, read instead. `action_begin`
  (it zeroes the group's `+0x28`); then, for a group that is not
  buildings, the flag is forced to 0 when the leader's level is 0; then
  each of the group's `num` members that is active and
  `can_ever_transport` has `0x800000` set or cleared. The loop's bound
  is the group's `+0xc`, one member here.

**The reading.**

- **No `BoardOrder`, no `AwaitBoardOrder`.** `OrdersMemManager::get_obj
  (BOARD_SHIP)` has one caller, `Unit::add_board_order@005e4d10`, whose one
  caller is `Group::action_board_ship@00700010`. That is reached from
  `CommandPackage::process_board_ship@00948e00`, the `board_ship` command
  (0x0f), which no call site in the export issues (`docs/COMMANDS.md` §6's
  sweep of every `add_command` caller), and from `Group::finish_insert@
  0070e620`'s case 8, which replays a `BoardOrder` that already exists.
  `get_obj(AWAIT_BOARD)` has one caller, `Unit::add_await_board_order@
  005e4c80`; its callers are `action_board_ship` and `Unit::
  check_meet_ship@00604550`, which only `Unit::do_board@005ed1f0` — a
  `BoardOrder`'s own step — calls. `copy_order@0072f900` and
  `get_new_order@00730550` copy or load an order that exists.
- **The disk agrees.** No dump on disk prints either label: the 149
  gamelogs of the `Logs` archive (50 GB, both long maps' AI transport rides
  among them — run86 prints 41 `CASTORDER`s and no `BOARDORDER`) and the
  60 golden and lab captures.
- **What boards is a cast** (`docs/TRANSPORT.md` §6.1). A step of a unit
  that `can_transport` onto a water tile is converted by
  `Unit::set_new_location@005f8d20` into `add_cast_order(−1, −1, −1, −1,
  0x28a, QUEUE_FIRST, 0)` — the Transport craft, flags 0 — and the unit's
  frame ends; the next frame `Unit::do_cast` → `SpellType::cast_transport
  @00670db0` makes the barge, which takes the unit's orders and its path.
- **Without the bit** the land arm of `UnitData::invalid_loc@00607c30`
  refuses the water tile, and nothing converts the step.

**The staging, walked before the capture** (run250, this file's first
eight lines to 646; `docs/RUNS.md` run250). Every predicate the premise
rests on is a field the dump prints:

- **The level**: `leader_flags` 1799 (the three level bits `0x100`, `0x200`, `0x400` set) from 605. `library who=0 1`
  gains Written Word, `TRANSPORT_BONUS`'s prerequisite
  (`docs/TRANSPORT.md` §13), and the Dock on 604 (`orig_type 432`) is the
  dock `Leader::check_transport` counts.
- **The bits**: every who=0 unit carries `0x800000` on 605; `0/6` and
  `0/7` are born with it (`Unit::init`).
- **The toggle**: `process_set_transport 621`; `0/7`'s `unit_masks`
  8388608 → 0 on 622, its stack empty before and after.
- **The shore the move reaches.** On 642 `0/6` holds a `MOVEORDER` (flags
  5) to (11904, 34944) and a six-entry plan whose (8856, 34200) — cell
  (11, 44), the lake's first water cell — carries flags 4, the embark. On
  644 `0/7` holds the same order to (11904, 36480) and a six-entry plan
  **straight onto the water with no flags-4 entry**: the world search did
  not pull the goal back to land for the unit without the bit, so what it
  does at the waterline is the step's to decide, not the plan's.
- **This crate walks run250 whole**: all eight lines, and no draw or
  `game_random` word parts to 646.

**The cast.** Two Chariots on the west shore of lake 70 (§4: sea region
70; land to x 10 on rows 44–47, water from x 11), a Dock on its north-west
shore, and the toggle on `0/7` alone.

**Lines.**

- `0 !ai off`; `600 library who=0 1`; `604 add dock who=0 53,153`.
- `610 add chariot who=0 34,177` (`0/6`) and `612 add chariot who=0
  34,189` (`0/7`).
- `620 @settransport 0 0 7`: `0/7`'s bit off.
- `640 @move 0 11904 34944 6` and `642 @move 0 11904 36480 7`: both onto
  the lake's deep cells (15, 45) and (15, 47).
- `800 @settransport 0 1 7` and `820 @move 0 11904 36480 7`: `0/7`'s bit
  on, and the same move again.
- `900 @move 0 19584 34944 8`: `0/8` — the barge this crate expects
  `0/6`'s cast to make — to dry ground on the east shore, cell (25, 45).

A call on trace frame F is on block F+2 (§17).

**The capture must dump** `end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1`
and `misc:COMMANDMANAGER=1` over `[605, 1300)`, 695 blocks, beside
run105's `start:` set: chapter nineteen's line. **This crate's prediction**,
a scratch walk forward from run250's start: `0/6` lays the cast on 702 at
(8428, 34200) and is inside barge `0/8` on 703, which reaches the point on
855; `0/7`'s path runs out on 718 at (8265, 35365), its stack empty, no
barge; `0/7` re-flagged on 801 casts on 828 and is inside `0/9` on 829;
`0/8` takes the 900 move and puts `0/6` ashore at (18168, 34152) on 1159,
dying there. The first parting is expected between 702 and 718; 250
blocks past 718 is 968, and the window runs to the disembark plus 140.

**The premise's killer, and its writers** (§3, point 5).

- **A `BOARDORDER` or `AWAITBOARDORDER`** on any unit on any block. Its
  builders are the two adders above and nothing else.
- **`0/7`'s bit on 622 and on 802** — the claim's own unit and field, not
  the first row (711). The bit's writers are `Unit::init`,
  `Leader::check_transport` (at a dock's activation or loss and at every
  `gain_tech`: none after 604 here, with the AI off), `action_set_
  transport`, `cast_transport` (the barge's) and `eject_contents` (the
  passenger's, back), and `ScenarioFuncSet::force_transport_ability`.
- **The killer splits the readings** (parked 789): the table's reading
  puts a `BOARDORDER` on `0/6` or `0/7`; this chapter's puts a `CASTORDER`
  on `0/6` and nothing on `0/7`; a third — the toggle as a no-op on a
  unit that already holds the bit — leaves `0/7`'s bit set on 622.

**What would falsify it**, and where each would first fire.

1. **The issue does not reach the pump.** Trace frame 620 or 800: an
   `INFO 17` with a refusal; or no `process_set_transport` on 621 or 801
   (`COMMANDMANAGER`).
2. **The toggle is not the bit.** Block 622: `0/7`'s `unit_masks` still
   holding `0x800000`; block 802: without it.
3. **The toggle builds an order.** `0/7`'s stack not empty on 622 or 802;
   or falsifier-killer one, any block.
4. **The unflagged unit boards.** `0/7` holding a `CASTORDER`, or a who=0
   unit born, before 800 — first on the block its step reaches the
   waterline (this crate: 716–718).
5. **The flagged unit does not board by a cast.** `0/6`'s stack, on the
   block before its step crosses (this crate: 702), not a `CASTORDER`
   (spell 650, flags 0) ahead of the `MOVEORDER`; or, the next block, no
   `0/8` of the barge type holding that `MOVEORDER`, with `0/6` off the
   map.
6. **The re-flagged unit does not board.** After 820, `0/7` stopping at
   the shore again with no `CASTORDER` (this crate: 828).
7. **The disembark.** `0/8` not putting `0/6` ashore on the east bank, or
   surviving it (this crate: 1159); or the 900 line refused (`INFO 17`,
   refusal 3: `0/8` is not the barge).

**run249, and which falsifier fired: none** (`docs/RUNS.md` run249; 836
s, the same game as run250 for the 647 frames they share, 355,840
`GROUPDATA`).

- **The toggle.** `process_set_transport 621` and `801`. `0/7`'s
  `unit_masks` 8388608 → 0 on 622 and 0 → 8388608 on 802, its stack empty
  on both. No order anywhere from either line.
- **The flagged Chariot boards by a cast.** `0/6` walks its plan to
  (8428, 34200); on 703 it holds the `MOVEORDER` and a `CASTORDER` —
  spell 650, `UNITORDER` flags 0, `paid` 0, no target — and on 704 it is
  inside `0/8` (`inside_up 8`), off the map, and `0/8` holds the
  `MOVEORDER` and four of the path's entries, born at (8588, 34379). `0/8`
  reaches (11904, 34944) on 856.
- **The unflagged Chariot stops at the shore.** `0/7` walks its six-entry
  plan to (8265, 35365) — cell (10, 46), the last land cell — and its path
  and stack are empty on 719. No cast, no new unit.
- **Re-flagged, it boards.** On 822 `0/7` holds the 820 move and a fresh
  four-entry plan; on 829 the `CASTORDER` at (8440, 35478); on 830 it is
  inside `0/9`, which reaches (11904, 36480) on 967.
- **The disembark.** The 900 line names `0/8`, the barge (`process_group
  … 901`). It sails to the east bank; on 1160 `0/6` is ashore at (18168,
  34152) holding the `MOVEORDER` to (19608, 34968) and `0/8` is gone. `0/6`
  arrives on 1222.
- **The absence.** No `BOARDORDER` or `AWAITBOARDORDER` on any of the 695
  blocks. §13's row is struck through to this section.

This crate's prediction, written before the run, is the original's to the
block: 703, 704, 719, 802, 829, 830, 856, 967, 1160.


**The word: `GOLDEN_WORD_CHAPTER_TWENTY` = 1300, closed on the first
walk.** The command's entry, `input::group_set_transport` →
`Sim::set_transport` (`docs/TRANSPORT.md` §15), landed with the chapter
before the capture, and no draw count, draw sequence or `game_random`
word parts on any of the 695 blocks.
- **The value diff under a closed word** (DECISIONS 51.2). The widening
  over (605, 1301) parted first on block 605: the staged Dock `0/2007` at
  (10176, 29376) here and (10752, 29760) there. That is `snap_center`'s
  dock arm, now built (`Sim::snap_center_placed`). On block 1160 it parted
  on `0/6`'s `orders_x/y`, (19608, 34968) here and (8428, 34200) there,
  and then its `dest_angle`, 671481856 here and 1073741824 there — the
  disembark's `update_action` and `come_out`'s angle, now built. All three
  agree now.
- **The toggle's own field**: nothing in the widening compared it —
  `widen_block` reads two other bits of `unit_masks` — so
  `run249_s_transport_is_the_original_s_unit_for_unit` reads it raw on
  every block, beside the leader's level, the casts and the absent
  `BOARDORDER`: 12,036 rows, none parted.
- **What stands.**
  - The births' `form`.
  - The passengers' figures' `avg_speed` on the boarding frame: 20 here
    against 15 there on 704, and 18 against 13 on 830. The original ages
    it once more before the figures freeze inside.
  - On the disembarked `0/6`: its figures' animation on 1162 (`cur_anim`
    7 against 8) and its facing on 1165.
  - In the pool, the toggle's pushed selection's `ox`/`oy` and the 642
    move's group `speed`.

## 29. Chapter twenty-one — the repair line: a right-click on a damaged building (item 813)

**The reading first** (the fifteenth pass: after §28, §13's `RepairOrder`
row was a reading before it was a chapter). The row said the `repair`
command type has no `CommandManager` issuer. That is true, and it is not
the route: **a player's repair is the `swarm_around` command** (type
`0x06`, 17 bytes, `docs/COMMANDS.md` §3) with `orders` `REPAIR` (13), and
the DLL can call its issuer.

**What builds a `RepairOrder`.** `OrdersMemManager::get_obj(REPAIR)` has
one caller, `Unit::add_repair_order@005e4ff0`. Its five callers are the
whole list, and each is a route:

| route | caller of `add_repair_order` | reached from | whose |
|---|---|---|---|
| the right-click | `Group::action_swarm_around@0070fbe0`, its `REPAIR` arm, `QUEUE_LAST`, the command's action bit | `CommandPackage::process_swarm_around@00949970` ← `CommandManager::issue_swarm_around@009416b0` ← `GroupOut::issue_swarm_around@0070afc0` ← `Console::execute_at_cursor@007c6630` (a right-click on a damaged, finished building of one's own or an ally's) and `Options::picked_spot@00721c40` (the Repair pick) | a player's, **the chapter's** |
| the re-entry | the same arm | `Unit::do_repair@005ee420`: a repairer not adjacent kills its order and swarms again at `QUEUE_FIRST` (a halt, the arm at `QUEUE_NEW`, `finish_insert`); `Unit::find_repair_spot@00604320` for a computer's | the order's own step |
| the replay | the same arm | `Group::finish_insert@0070e620`, cases 6 and `0xd` | a halted group's |
| the `repair` command | `Group::action_repair@007020c0` | `CommandPackage::process_repair@00948cb0` (command `0x10`, which **nothing issues**: `docs/COMMANDS.md` §6's sweep of every `add_command` caller) and `ScenarioData::issue_order@00996ac0` (the trigger system, cut from v1) | nobody a game reaches |
| the gatherer's | `Unit::do_gather@005ef2a0` | a computer-driven gatherer (`unit_masks & 0x40000`), difficulty above 1, one frame in 256 (`(o + frame + who) & 0xff == 0`), its building damaged and its city's flag 2 clear, at `QUEUE_NEW` without the action bit | a computer's |
| the building's call | `Build::process@0061edf0` | a building damaged past half its hits, its owner not human (`leader_flags & 4` clear), difficulty above 1, no enemy capturing it: the idle friendly `PEASANTS` (`0x32`) `find_unit` returns within `0x1e00`, at `QUEUE_NEW` | a computer's |
| the rally | `Unit::come_out@00617c10` | a unit coming out of a building whose destination object is a building with damage (`+0x24`), `QUEUE_LAST`, no action bit: the gather point's arm, read no further than the call | no command |

`Group::action_repair` is a citizen-only walk (`PEASANTS`/`PEASANTSKOREAN`)
gated on `Region::is_coast`, with `add_repair_order` at `QUEUE_NEW` or
`QUEUE_LAST`; no game reaches it. `ScenarioFuncSet::citizen_repair_order@
009f85b0` calls `action_swarm_around` for the trigger system and is cut
with it.

**The issuer's two callers pass the same bytes.** `execute_at_cursor`
picks `REPAIR` when the building under the cursor is finished (vslot
`0x4c`; unfinished is its `BUILD_AT` arm) and damaged (`+0x24`), on a cell
whose owner is nobody, the player or an ally; `picked_spot` asks the
same of its pick. The queue is `QUEUE_NEW` unmodified, `QUEUE_LAST` with shift,
`QUEUE_FIRST` with shift and alt (`GetKeyState(0x10)`, `(0x12)`). A
`BUILD_AT` swarm is the same issuer on an unfinished building, which is
how a player sends citizens to help a site.

**What the arm does with `REPAIR`**, the same member arm as §26's build
(`docs/ORDERS.md` §5.4), with four differences read off the function:

- **the approach is a `MOVE_TO` for everyone.** `local_40` is 1 and is
  rewritten only on the `BUILD_AT` arm (`~(leader_flags >> 1) & 2 | 1`).
  So **parked 791 is confirmed by the reading**: a computer's `REPAIR`
  swarm walks under a `MOVEORDER`, where this crate's single-unit
  `Sim::swarm_around` gives it an `EXPLORETOORDER`;
- no gather filter (`local_30` is 0 unless `BUILD_AT` on a gather
  building);
- no farm halving of the ring, and **no `0x30` nudge**: the `REPAIR` arm
  jumps past the second `find_nearby_spot` to the queue test;
- `add_repair_order(o, who, QUEUE_LAST, action)` behind the approach,
  with the `CIVILIANSPELL` (`0x293`) cast ahead of it for a member that can
  cast it; no `build_masks` write.

**`add_repair_order` itself** (`005e4ff0`): at `QUEUE_NEW` it clears
`unit_masks & 0x4000000`, zeroes `+0xc0`, closes the orders, clears the
partial path and updates the action; it always sets `unit_masks & 0x400`
(the builder bit); it writes the target's `o`, `who` and `uid` into the
order, the action bit into its flags, and sets the auto-transport bit
`0x800000` when the target is in another region the unit's transport
level can reach; then it adds the order at the end of the list (there is
no `QUEUE_FIRST` tail) and updates the action.

**The record the chapter reads.** `RepairOrder::log_data@00482720`
prints `REPAIRORDER` and then `TargetOrder`'s fields — the target's `ox`,
`whom` and `uid` — inside `UnitOrder`'s `flags`; `RepairOrder::get_type@
004827a0` returns `REPAIR` (13), the `type` a repairer's action shows; and
`RepairOrder::RepairOrder@00482330` leaves the target at (−1, −1) with
`uid` `0xffff` until `add_repair_order` writes it.

**So the DLL can reach it.** `issue_swarm_around`'s prologue is
`issue_attack`'s to the byte (`55 8b ec 83 ec 14 56 8b 75 0c c6`: it
tests `ox` and `whom` before `check_accept_issue`), and the DLL gained
`@repair <who> <ox> <whom> <o>…`, which passes `QUEUE_NEW` and `REPAIR`.

**The issuer, under the emulator.** A scratch script ran `issue_swarm_around
(group, 2006, 0, QUEUE_NEW, REPAIR)` on `tools/explore/command_oracle.py`'s
fixture.

- **One call appends 22 bytes**: the 5-byte `group` and a 17-byte
  `swarm_around`, type `0x06`, `[ox i32][whom i32][queued i32][orders
  i32]`, every field as passed. The same selection again appends the
  3-byte reuse (20 bytes); `BUILD_AT` and `QUEUE_LAST` change only their
  own words.
- **It refuses** a negative `ox` or `whom` (the listing's two `js` before
  `check_accept_issue`), and appends nothing under `use_mp_playback` or
  `semaphore & 0x10`.
- **It writes** the package's size and data and the selection caches:
  no unit, no order, no draw.
- **What the emulator cannot reach**: `process_swarm_around@00949970`,
  which logs `process_swarm_around <ox> <whom> <queued> <orders> <frame>`
  and, for a target that is `(−1, −1)` or active, calls
  `action_swarm_around(ox, whom, queued, orders, 1)`, read above; and
  `Unit::do_repair`'s step, `docs/ORDERS.md` §5.6 and `docs/CITIES.md`
  §9.3.

**The premise.** A player's right-click repair **lays two orders a
citizen**: a `MOVEORDER` to a free spot on the ring round the building,
and a `REPAIRORDER` behind it with the action bit (flags 4). The citizen
walks the ring, mends the damage at `do_repair`'s rate, and then, the
building whole, **drops the order**; a citizen that reaches a building
already whole drops it on arrival. No `repair` command, no
`Group::action_repair`, no `EXPLORETOORDER`. The killer is the claim's
own unit, the citizen's stack on the block after the command, and it
splits the readings (parked 789): the table's reading (`action_repair`)
lays a lone `REPAIRORDER`; the build line's shape for a computer lays an
`EXPLORETOORDER` approach; this chapter's lays a `MOVEORDER` and a
`REPAIRORDER`. **Parked 791 is not tested here**: the player is human, and
a human's approach is a `MOVE_TO` under both readings of `local_40`.

**The staging, walked before the capture** (run256, `chapter21.cmd`'s
twelve lines to 830, four takes; `docs/RUNS.md` run256). A building must
be damaged first, and the damage came from three staging mistakes.

- **Take 1**, the Barracks on the neutral arena: the hoplites never struck
  it. `Object::check_target` refuses a building that neither
  `build_flags & 0x10` (buildable outside a city) nor an owned cell
  allows (`docs/COMBAT.md` §12.2). The `@repair` line was processed all
  the same: `process_swarm_around 2007 0 2 13 781`, and `0/6` on 782
  under a `MOVEORDER` and a `REPAIRORDER`.
- **Take 2**, in who=0's territory beside Napata: the hoplites strike, 4 a
  blow, `damage` 48 by 760. This crate parts on 636, a walk animation's
  draw on the first step toward the building (`Guy::set_anim` <
  `Unit::move_step`), in `movement.rs` and `anim.rs`, fenced and not the
  chapter's. So the attackers became archers who stand.
- **Takes 3 and 4**, who=1's Bowmen at three tiles: they fire from where
  they stand and the Barracks takes `damage` 3 (frac 12) by 744. This
  crate parted on 650, the first arrow a frame late: who=1's Bowman is
  **piece 120**, whose release bays nobody had measured, so its arrows
  left the figure's own point, 86 units behind the bow hand. Take 4's
  twelve arrows measure all three of its releases (`sim::launch`'s
  `BAYS`: `(120, ATTACK1, 12)`, `(120, ATTACK2, 9)`, `(120, ATTACK3,
  15)`, each the centroid of the integer region that reproduces every
  launch point of its key exactly, `dz` 163, 164, 164), and
  `launch::tests::run256_s_bowman_arrows_leave_from_the_measured_bays`
  pins the seven distinct points and their `sz`. 472's nodes, the
  Nubian bowman's, put the same arrows one to four units out.
- **With the bays, this crate walks take 4 whole**: no draw count,
  sequence or `game_random` word parts to 830.

What take 4 printed, each a field the chapter reads: the Barracks
`0/2007` at (5760, 32640), `myhits` 1200; `peace 1` and the Bowmen walking
off from 765, their action `type` 2 (`ATTACK_TO`); on 782 `0/6`'s `unit_masks` 0 →
1034 (the `0x400` builder bit) and its stack a `MOVEORDER` (flags 1) to
(5736, 33096), the ring's spot on the Barracks' south face, with the
`REPAIRORDER` (flags 4, `ox 2007`, `whom 0`, `uid 13`) behind it; and on
802 the same on `0/7`, `0/8` and `0/9`.

**This crate's prediction**, a scratch walk forward from take 4's start:
`0/6` reaches its spot and its `MOVEORDER` pops on 929; the Barracks goes
3 → 1 → 0 on 930 and 931; `0/6`'s stack is empty on 932. `0/7` and `0/8`
arrive on 967 and 968 and `0/9`, from the far side, on 1022; each drops
its `REPAIRORDER` the block it arrives (968, 969, 1023) with nothing
behind it. From 1083 the idle citizens take gather orders of their own.

**The cast.** Chapter eighteen's citizens and chapter two's archers, in
who=0's territory beside Napata (§4's free ground).

**Lines.**

- `0 !ai off`; `600 library who=0 2`; `602 library who=1 3`.
- `606 add barracks who=0 30,170`: `0/2007` at (5760, 32640).
- `610 add bowmen who=1 34,170`: `1/6`, `1/7`, `1/8`.
- `760 peace 1`: the Bowmen stop.
- `770`–`776 add citizen who=0`: `0/6` at tile (28, 190), `0/7`..`0/9` at
  (26, 192), (28, 192), (30, 192).
- `780 @repair 0 2007 0 6`; `800 @repair 0 2007 0 7 8 9`.

A call on trace frame F is on block F+2 (§17).

**The capture must dump** `end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,
AMMO=5` and `misc:COMMANDMANAGER=1` over `[605, 1300)`, 695 blocks, beside
run105's `start:` set. This crate's last event is `0/9`'s order dying on
1023, and 250 blocks of runway past it end at 1273.

**The premise's killer, and its writers** (§3, point 5). The citizen's
stack on 782 and 802. `REPAIRORDER`'s one adder is `add_repair_order`, and
of its five callers (the table above) only the swarm's arm can reach a
who=0 citizen here: `do_gather`'s and `Build::process`'s need a computer,
`come_out`'s a unit leaving a building, and `action_repair` the command
nothing issues. `unit_masks & 0x400`'s writers include
`add_repair_order` and `add_build_order`; no build is staged.

**What would falsify it**, and where each would first fire.

1. **The issue does not reach the pump.** Trace frame 780 or 800: an
   `INFO 17` with a refusal; or no `process_swarm_around 2007 0 2 13` on
   781 or 801 (`COMMANDMANAGER`).
2. **The command is not the swarm.** Block 782 (802 for the trio): the
   citizen's stack not a `MOVEORDER` to a ring spot with a `REPAIRORDER`
   (flags 4, `ox 2007`) behind it — a lone `REPAIRORDER`, an
   `EXPLORETOORDER` approach, or a `BUILDORDER` each fires it; or
   `unit_masks & 0x400` clear on 782.
3. **The repair does not happen.** `0/2007`'s `damage` still above 0 ten
   blocks after `0/6` stands at its spot (this crate: 929; 3 → 1 → 0 on
   930–931).
4. **The late repair lives on.** `0/7`, `0/8` or `0/9` still holding a
   `REPAIRORDER` the block after it arrives at the whole Barracks (this
   crate: 968, 969, 1023), or taking a `GATHERORDER` at it (a Barracks is
   no gather building).
5. **The three spots are not three.** Two of `0/7`..`0/9`'s `MOVEORDER`
   points equal on 802: `find_nearby_spot`'s occupancy test is what
   spreads them (`docs/ORDERS.md` §5.4).

**run255, and which falsifier fired: none** (`docs/RUNS.md` run255; 884
s, the same game as run256's take 4 for the 831 frames they share,
355,840 `GROUPDATA`).

- **The swarm.** `process_swarm_around 2007 0 2 13` on 781 and 801. On
  782 `0/6`'s `unit_masks` 0 → 1034 and its stack a `MOVEORDER` (flags 1)
  to (5736, 33096) with the `REPAIRORDER` (flags 4, `ox 2007`, `whom 0`)
  behind it. On 802 the trio's the same, to (5688, 33048), (5880, 33048)
  and (5352, 32808): three spots.
- **The repair.** `0/6` stands at its spot on 929; the Barracks' `damage`
  goes 3 → 1 on 930 and 1 → 0 on 931, with timber 261 → 260 on 931, the
  price; `0/6`'s stack is empty on 932.
- **The late orders.** `0/7` and `0/8` reach a whole Barracks on 967 and
  968 and `0/9` on 1022; each `REPAIRORDER` is gone the next block (968,
  969, 1023), with nothing behind it.
- **After.** The four take `GATHERORDER`s on `0/2001` of their own on
  1083, 1129, 1130 and 1176.

This crate's prediction, written before the run, is the original's to the
block: 782, 802, 929, 930, 931, 932, 968, 969, 1023, 1083.


**The word: `GOLDEN_WORD_CHAPTER_TWENTY_ONE` = 1141, open, on the first
walk.** The command's entry — `input::group_swarm_around` →
`Sim::group_swarm_around` (`crates/sim/src/swarm.rs`) →
`group_action_swarm_around` — and piece 120's bays landed with the chapter
before the capture, and no draw count, draw sequence or `game_random`
word parts through the whole repair line: the arrows, the peace, both
swarms, the repair and the three late orders. The word is past it. On
1141 the original spends two walk-step animation draws (`Guy::set_anim` <
`Unit::move_step`), `0/8`'s and `0/7`'s, and this crate spends `0/8`'s
alone.

- **The value diff at the word** (DECISIONS 51.2). The first record
  that parts on a citizen is **`0/7`'s `MOVEORDER` on 1131**, the approach
  to its idle gather on `0/2001`: `x` 4104 here and 4296 there, `off_x`
  264 and 456, `angle` −237961216 and −211681280. The original sends
  `0/6`, `0/8` and `0/7` all to (4296, 28824); this crate spreads `0/7` a
  tile west. Its path parts on 1132, and on 1141 its step draws there
  and not here. That is the gather approach (`gather.rs`), not this
  item's module.
- **The widening** over (605, 1301), both directions, with `AMMO`: the
  births' `form`; chapter two's `order:target` on the Bowmen's attack
  (635); chapter eight's `group.id` on their walk-off (1021); the idle
  gathers' `group` (1083, 1129, 1130); and from 1131 `0/7`'s approach and
  what follows it. **Every arrow agrees**, launch, landing and clock, on
  every block. In the pool: the pushed selections' `ox`/`oy` on 782 and
  802 (chapter seventeen's family) and slot 1's `speed`/`new_speed` on 802
  (chapter twenty's).
- **The repair's own fields**, read raw in
  `run255_s_repair_is_the_original_s_unit_for_unit`, because the widening
  compares neither: `unit_masks & 0x400` (`widen_block` reads other bits)
  and the `REPAIRORDER`'s target (`build_ids` names only the start dump's
  buildings, so `target_ids` leaves a staged building's target
  uncompared). 14,623 rows, 704 repair-order frames. They part on four
  rows only: **the builder bit outlives the repair here**. The original
  clears `0x400` in `Unit::add_gather_order@0061a5c0` and in
  `Unit::think_peasant@005f5760`'s failed build arm; this crate's
  `was_builder` is never cleared, so on each citizen's first gather (1083,
  1129, 1130, 1176) ours is 1 and theirs 0. `orders.rs`, fenced. The test
  failed first on the target, which read −1 through `build_ids`.
- **What is not compared**: the Barracks' `helpers` is 0 on every block
  of both sides (the step's own counter, reset before the dump), and
  stays pinned unread.
