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
| SpecialAnimOrder | ~~**not** `anim`, which pokes `Guy::set_anim@005da300`~~ **auto, and no issuer makes one** (§30): its one adder, `Unit::add_spec_anim_order@005e4160`, is called by `Unit::land_plane@005e9950` (type 0, a plane into its base) and `Unit::come_out@00617c10` (type 1, every unit out of a building on a frame past 0), and `do_spec_anim@005e5880` kills each in the call that adds it, so no dump prints one; ~~the launch arm (type 1 at an `AIRBASE`) is parked 761's route and no game reaches it~~ the launch arm (type 1 at an `AIRBASE`) is `Object::do_launch@0064f3b0`'s, entered by run265 (§31) | 13 (the eject, 902–903), 17 (the landing, 722) and 22 (the launch, 778), and every trained unit (§30) |
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
export, and whether the DLL can reach it. **Both are read**: a player's
repair is `swarm_around` (§29, a chapter), and a `SpecialAnimOrder` is
the engine's own, entered by chapters thirteen and seventeen (§30, no
chapter). `AirAttackGroundOrder`,
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
| 255 | twenty-one, the repair line | `[605, 1300)` | `issue_swarm_around` with `REPAIR` through the DLL's `@repair` on a lone citizen and a trio, at a who=0 Barracks who=1's Bowmen damaged before a peace, with `GROUPS=1` at `GUYS=4` and `AMMO=5`; the staging walked first on run256, to 830, four takes (§29) — **run 2026-09-26 (item 813), 296 MB, 884 s; the pool printed; no falsifier fired: `0/6`'s `MOVEORDER` and `REPAIRORDER` (flags 4) on 782, the trio's on 802 at three spots; the Barracks 3 → 0 on 930–931; the trio's orders dying on arrival on 968, 969 and 1023, each on this crate's predicted block; word 1141 (item 813: `0/7`'s camp approach a tile west), closed at 1300 (item 824: a human's found gather drops its group, ORDERS §5.9)** |
| 265 | twenty-two, the launch line | `[605, 1500)` | a strike from inside a base: chapter seventeen whole and `@strike` on the Fighter `0/6` inside `0/2007` on 766, its tank at 24, with `GROUPS=1` at `GUYS=4` and `AMMO=5`; the staging's predicates read off run223 (§31) — **run 2026-09-26 (item 836), 365 MB, 1,046 s; the pool printed; one clause of falsifier 4 fired: `launch_frames` stays 0 once the base is empty; the strike on 768, the launch on 778 (the block the tank first reads 0) onto (11424, 13920), 36 rounds from 924, `returning` on 1178, inside again on 1385; word ~~778~~, ~~923, open~~ (item 836: the launch line built, ORDERS §38), then **1500, closed** (item 842: the strafer's half altitude and exact round, ORDERS §39)** |
| 281 | twenty-three, the repeat line | `[605, 1840)` | the repeat button through the DLL's `@buildmask` on the Airbase `0/2007` on 1440, between chapter twenty-two's landings: `0/6` inside with its kept patrol, `0/7` still flying home; with `GROUPS=1` at `GUYS=4` and `AMMO=5`; the staging's predicates read off run265 (§32) — **run 2026-09-26 (item 867), 498 MB, 1,505 s; the pool printed; no falsifier fired: `build_masks` 4232 → 4104 on 1442; `0/7` inside with no order on 1489 and `0/8` on 1513; `0/6`'s kept patrol killed on 1585, its tank's first 0, and `0/6` still inside; nothing launches to 1839; the building group in pool slot 0** |
| 285 | twenty-four, the queue line | `[605, 1560)` | the infinite-queue button through the DLL's `@buildmask` with 0x40 on a Barracks `0/2007` on 900, between the Hoplites' finish and the Bowmen's, both queued through the DLL's new `@queueup`; a second press on the empty queue on 1300; with `GROUPS=1` at `GUYS=4`; the staging walked by this crate on run208's start (§33) — **run 2026-09-26 (item 877), 403 MB, 1,164 s; the pool printed; no falsifier fired: `[132]` on 622 and `[132, 170]` on 642; the Hoplites out on 856 with nothing re-queued; `build_masks` 4096 → 4160 on 902; the Bowmen out on 1060 and re-queued at the end, paid again, the bit kept; out again on 1272, the re-queue refused on 19 wealth, the queue empty and the bit off; 4096 after the second press on 1302; a squad trained on each finish; word ~~855, open~~, closed at 1560 (item 877: the queue commands entered); the squads' `group` and the pool's lists agree (item 882: `come_out`'s push and the command's building group, §33)** |
| 292 | twenty-five, the cancel line | `[605, 1466)` | the player's cancel through the DLL's new `@unqueue` on a Barracks `0/2007`: slot 0 of a Hoplite run with the head in progress (700), slot 0 of `[132*, 170]` (760), a single cancel on an infinite queue (840, after `@buildmask` 0x40 on 800), and −1 on `[132*, 170]` (1000); with `GROUPS=1` at `GUYS=4`; the staging walked by this crate through the commands' entries on run285's start (§34) — **run 2026-09-26 (item 884), 359 MB, 1,099 s; no falsifier fired, every value on its predicted block: arm a removed the run's second (+53/+41) on 702 and kept the head at 8100; arm b the head (+51/+38) on 762; arm c cleared the bit and removed nothing on 842; no re-queue at the Bowmen's finish on 965; −1 removed the last (+46/+56) on 1002; no cancel seats a slot; word ~~855, open~~, closed at 1466 (item 884: the cancel entered)** |
| 296 | twenty-six, the research line | `[605, 1492)` | a technology through the player's `@queueup` on who=0's Library `0/2005`: The Art of War with `num` 2 on the idle Library (620), again while it researches (640), Written Word on the busy Library (650), The Art of War held (850), Barter behind Written Word (860), Hoplites at the Barracks (870), a cancel of the re-priced Barter (1040) and Barter again (1060); with `GROUPS=1` at `GUYS=4`; the staging walked by this crate through the command's entry on run292's start (§35) — **run 2026-09-26 (item 883), 365 MB, 1,120 s; no falsifier fired, every value on its predicted block: one entry and 120 food on 622, nothing on the second press (642) or the held one (852), Written Word behind the busy head on 652, The Art of War out on 822 (discovered 2), Barter re-priced to 54/54 on 1023 (discovered 3: Boadicea counted), +54/+54 on the cancel (1042), 54/54 again on 1062, Barter out on 1242; word 1492, closed on the first walk (a research spends no draw) and after the build, whose value rows it closed (item 883: the research arm entered)** |
| 300 | twenty-seven, the upgrade line | `[605, 1560)` | a unit upgrade through `@queueup` at who=0's Barracks, staged Classical (§36) |
| 304 | twenty-eight, two buildings under one command | `[605, 1580)` | `@queueup` and `@buildmask` on two Barracks: the sort and passes, the toggle on `[on, off]`, the building group of two (§37) |
| 308 | twenty-nine | `[605, 2070)` | §38 |
| 171 | eight, the commanders and a declared war | `[605, 1200)` | the diplomacy moved three times — **run 2026-09-23 (item 660), 54 MB, 168 s; no falsifier fired; `ally` ended the game on 900, so the capture is 605..901; word 617** |

Chapter thirty onward: §14a, where this table continues (parked 932).

## 14a. The running order, continued

§14's table is at the section ceiling (parked 932); its rows from chapter
thirty on are here, in the same columns. Any further detail window takes
its number at booking. The order of both tables is by **what a failure
would teach**, not by chapter number: 113 and 116 are placed early because
each can invalidate work that would otherwise be done on top of it.

| run | chapter | window | why this order |
|---|---|---|---|
| 312 | thirty, the gather point | `[605, 1450)` | `@gatherpoint` (the DLL's new verb 20) on two Barracks and the City: 2007's point on the ground before its Hoplites, moved onto 2008 between the Hoplites' finish and the Bowmen's, and cleared; 2008's on itself before its Hoplites; the City's on a forest before its Citizen; with `GROUPS=1` at `GUYS=4`; the staging walked by this crate on run308's start (§39) — **run 2026-09-27 (item 928), 367 MB, 1,010 s; the pool printed; no falsifier fired: every list on its block, the City's point snapped to its Woodcutter's; the Hoplites attack-moved to 2007's point from 856, 2008's stayed in, the Bowmen garrisoned 2008 by 1079, the Citizen walked to the Woodcutter from 760**; item 945 built the Bowmen's re-seat, 105 → 35 rows |
| 338 | thirty-one, the gather point's other arms | `[605, 1400)` | chapter thirty's cast and a second Chariot: the City's point on the ground, then on its Woodcutter (action 1), then on a Lookout site, each before one of three Citizens (956's seeker on three units, 948's gather and build arms); 2007's point on the Chariot before its Hoplites (957's third re-seat); 2008's two points, the second by `@gatherpointadd` (verb 20 with `add_to_end` 1), before its Hoplites (946); the staging walked by this crate on run312's start (§40) — **run 2026-09-27 (item 955), 351 MB, 966 s; falsifier 3 fired on all three Citizens: `FILTER_ALL` counts the seeker; every other reading held; the word 740 → 1400, closed** |
| 344 | thirty-two, an Airbase's gather point | `[605, 2360)` | chapter twenty-nine's cast and `@gatherpoint` on the Airbase `0/2007` three times: P1 with `0/6` flying and `0/7`, `0/8` inside (1600), P2 appended before the Biplane's birth (1700), and the Clear with all four flying (1850); with `GROUPS=1` at `GUYS=4` and `AMMO=5`; the staging walked by this crate on run308's start (§41) |

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
scout row on 847, and ~~the squad's `group` on 903. `come_out` pushes a
squad that leaves a building into a fresh pool slot, behind the eject's
own building group, and this crate carries neither (parked 689's
family)~~ — **both carried since item 882** (§33): the eject seats
`[2007]` in slot 2 and `come_out` pushes the squad into slot 3, and
`0/7`–`0/9` read group 3 on 903 on both sides.
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
- **The widening is run223 whole**, (605, 1401). ~~What stands past the
  births is **the tank** (parked 765): `returning` is 1 there and 0 here
  on `0/7` from 1212 and on `0/8` from 1214, and each plane's flight
  home after it. No draw follows from it to 1400.~~ The tank is built
  (item 836, `docs/ORDERS.md` §38.1): both turn for home on 1212 and
  1214 on both sides, and only the births stand.
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
twelve lines to 830, four takes, each in `docs/RUNS.md` run256). A
building must be damaged first. On the neutral arena a building is no
target (`Object::check_target`, `docs/COMBAT.md` §12.2); in who=0's
territory a walking attacker parts this crate on a fenced walk draw
(636); so the attackers are who=1's Bowmen, who stand and shoot. Their
piece, **120**, had no measured release bay, and take 4's twelve arrows
measure its three (`sim::launch`'s `BAYS`, pinned by
`launch::tests::run256_s_bowman_arrows_leave_from_the_measured_bays`;
472's nodes put the same arrows one to four units out). With the bays,
**this crate walks take 4 whole**: no draw count, sequence or
`game_random` word parts to 830.

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


**The word: `GOLDEN_WORD_CHAPTER_TWENTY_ONE` = 1300, closed** (item 824;
~~1141, open, on the first walk~~, item 813). The command's entry —
`input::group_swarm_around` → `Sim::group_swarm_around`
(`crates/sim/src/swarm.rs`) → `group_action_swarm_around` — and piece
120's bays landed with the chapter, and nothing of the repair line parts.
813's word was past it: on 1141 the original spent `0/7`'s and `0/8`'s
walk-step draws (`Guy::set_anim` < `Unit::move_step`), and this crate
`0/8`'s alone.

- **The value diff at 1141** (DECISIONS 51.2): `0/7`'s camp-approach
  `MOVEORDER` on 1131, `x` 4104 here and 4296 there (`off_x` 264/456 and
  `angle` follow from it). Its one writer is `do_non_flat_gather`'s
  `find_nearby_spot`, and `find_ordered_collision` refused it 4296 through
  its **own-group arm**: `0/8`, sent there on 1129, shared `0/7`'s group.
  The input parted first: `group` on 1083 `0/6` (1 here, −1 there),
  1129 `0/8` and 1130 `0/7` (0 here, −1 there). `Unit::think_peasant@
  005f5760` writes it: on a found gather, a human's `+0x80` gets −1
  (`5f5900`–`5f590c`; `docs/ORDERS.md` §5.9). This crate did not.
  `was_builder` (parked 825) is not read on this path.
- **The widening** over (605, 1301), both directions, with `AMMO`: the
  births' `form`, `order:target` (635), `group.id` (1021), and in the
  pool `ox`/`oy` on 782 and 802 and slot 1's `speed` on 802. Every row
  from 1083 on is gone. **Every arrow agrees**.
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

## 30. The special-anim line — a reading, and no chapter (item 832)

**The reading first** (the fifteenth pass: §13's last `unresolved` row,
`SpecialAnimOrder`, was a reading before it was a chapter). **No command
builds one, and two closed chapters already enter both of its live
arms.** The class is the engine's one-call wrapper round a unit entering
or leaving a building: each is added and killed inside one call, so no
dump prints one, and a chapter would measure only what chapters thirteen
and seventeen already measure.

**What builds one.** The constructor,
`SpecialAnimOrder::SpecialAnimOrder@00484f10`, has one call site,
`get_new_order@00730550` (`73095b`), the order pool's factory. `get_obj(SPECIAL_ANIM)` has one caller,
`Unit::add_spec_anim_order@005e4160`, and the listing calls that twice,
at `5e9aca` and `61a00d`; the export agrees. `copy_order@0072f900`
(`Group::set_up_insert`, the save walk) copies one, and none is ever in a
list to copy.

The adder, from the listing (`5e4160`–`5e41b9`): it writes `type`
(`+0x8`), `data1` (`+0x14`), `data2` (`+0x18`) and the action bit, and
**`LinkListBase::add@0046d5a0` inserts at the head**, so the new order is
the current one. Then `clear_partial_path`, the cursor, `update_action`.
Its fourth argument is never read (`retl $0x10`). The caller writes
`data3` to `whom` through `update_order` after it (`SpecialAnimOrder`,
`types.txt`).

| route | `type` (the PDB's `SpecialType`) | caller, call site | what reaches it | run by | entered |
|---|---|---|---|---|---|
| the landing | 0 `SPECIAL_ENTER`: `data1` the base's vslot `+0x34`, `data2` 3 for a helicopter (the type's `+0x2b4` bit `& 0x20`) else 1, `data3`/`data4` the base's `o`/`who` | `Unit::land_plane@005e9950`, `5e9aca` | `Unit::do_air_physics@005e86d0`: a returning plane within a step and a half of its point, a flight home (`issue_flight@00941d40`, §25) | the `work` at `land_plane`'s tail (vslot `+0x188`) | **chapter seventeen**: run223, block 722, the Fighter `0/6` |
| the exit | 1 `SPECIAL_EXIT`: `data1` the host's `+0x34`, `data2` 0, `data3`/`data4` the host's point, `ox`/`whom` the host | `Unit::come_out@00617c10`, `61a00d` | every `come_out` that reaches its tail (`619fe2`) while the host's `+0x34` is non-zero and `Game::frame` is not 0: `Build::train@0062f9b0` (every trained unit), `Build::process_ejection@006201e0` and `Object::eject_contents@0064cd20` (the eject), `Object::do_launch@0064f3b0` (a plane leaving its base), `CommandPackage::process_come_out@009465d0`, `do_cast`'s unload, the deaths that empty a container | `do_spec_anim` called directly, `61a0b1` | **every long capture**: run53 from frame 99 and run54 from 274 (coverage traces; the frame `Build::train` is first entered); **chapter thirteen**, run208 902 and 903, by reading |
| the default | 2 `SPECIAL_UNIT`, what `SpecialAnimOrder::clear@00484ec0` and the constructor write | none | nothing passes 2 | `do_spec_anim` returns at once | never |

Setup's births (`Setup::build_units`, `place_unit`) call `come_out` on
frame 0, and the gate refuses them.

**No issuer builds one.** Every command that reaches the adder does so
through an arm that belongs to another command: a flight home
(`issue_flight`, §25), the eject (`issue_eject_all@00941ca0`, §21),
`issue_come_out@00942c90` (one unit leaving what it is in: the garrison's
other arm, parked 724), and a strike from inside a base (parked 761).
The console's `anim` pokes `Guy::set_anim@005da300` and builds none. So
the row is **auto**.

**What `Unit::do_spec_anim@005e5880` does** (listing `5e5880`–`5e5bc4`):
- `type` 2 returns.
- It writes `frames` 10 and `started` 1, then asks whether `started`
  is below a 0 or a 1 (`jl` at `5e5906`). **The progress arm is never
  taken** (`5e5ada`–`5e5bbf`, which would move the unit a step a frame
  and count `started` up), so every order ends in its first call.
- **ENTER**: if the base is active (vslot `+0x10`) or an
  `AIRCRAFTCARRIER`, then `go_inside(data3, data4, 0)` and the kill;
  otherwise the kill and a death.
- **EXIT**: unless the host (`ox`, `whom`) is an `AIRBASE`, the order is
  killed and nothing else. At an Airbase, a launch:
  - the unit faces 0;
  - it is placed at the base's point less `0xc0` in `x`. A helicopter
    instead draws two offsets, `Random::get(0, 0xffff) % 11` each, for
    `x` in −197..−187 and `y` in −5..5, and flies 200 above the ground;
  - its figure's bank and pitch (`+0x44`, `+0x4c`) are zeroed, the old
    kept (`+0x48`, `+0x50`);
  - the kill, then vslot `+0x188`.
- **The kill** (`Unit::kill_current_order@005e2cb0`):
  - `unit_masks &= ~0x20000`;
  - for an EXIT, while playing and not loading, the host's `launching`
    list (`ObjectData +0x44`, which only `do_launch` fills) drops the
    unit;
  - `remove_current`, `give_obj`, `update_action`.

So away from an Airbase an EXIT leaves nothing on the unit but an
`update_action` and the clear of a caster's bit.

**What enters it, and what compares it.** No `SPECIALANIMORDER` block is
in any of the 43 golden dumps or the 155 logged runs on disk (a grep,
2026-09-26), as the reading predicts.
- **ENTER, chapter seventeen, block 722.** This crate folds it into
  `crate::air`'s `land_plane` → `go_inside` (`docs/ORDERS.md` §33.5).
  The widening compares its effect on 722: `0/6`'s order stack empty
  (`compare_orders`), the figure's standing arm (`last_speed` 0,
  `stopped` 1), and `last_z = z` from 723. The container link itself,
  `up`/`up_who`, no site compares (`diff::coverage`'s `UnitDump` row).
- **EXIT, chapter thirteen, blocks 902 and 903**, and every long
  capture's trained units. `crate::garrison`'s `come_out` lays no order.
  What the arm leaves is its `update_action`, which `come_out` calls, and
  the `0x20000` clear, which a unit coming out never needs. The effect is
  compared by the ejected and born units' `action` and orders on their
  first block; chapter thirteen closed at 1000 with them agreeing. The
  `0x20000` bit is compared on chapter nineteen alone.
- **A block that printed one would fail a standing guard.**
  `rondata::diff::coverage`'s `every_key_the_dump_prints_is_read_or_pinned`
  drives ch17's 720–724 and ch13's 900–904, and nothing reads the class's
  keys (`started`, `frames`, `data1`…).

~~**The arm no game reaches is EXIT at an Airbase: the launch.** None of
the three coverage traces enters `Object::do_launch`, and no chapter
launches a plane (§25: an idle plane in its base stays there).~~ **run265
enters it** (§31, item 836): chapter seventeen's Fighter, struck out of
its base on 766, launches on 778 onto the base's point less `0xc0`; the
dump shows it, since the run is `cover=0`. Its route
is parked 761's, a strike from inside a base, which this crate does not
build (`docs/ORDERS.md` §32's SEAMs).

**What a staging would take.** Chapter seventeen's cast to 722, and the
Fighter's tank refilled inside (2 a frame, `Unit::process@00610bc0`).
Then `@strike` on `0/6` alone: `do_launch` would `come_out` it through
the arm above, onto the base's point less `0xc0`, facing 0. A Fighter
draws nothing there; a helicopter would draw twice. It needs 761 and the
tank (765) built first, and it names no score until then.

**What is not established.**
- ~~**The launch arm is entered by no run.**~~ run265's 778 (§31): the
  placement, `last_bank`/`last_pitch` 0.0 and the figure on the ground,
  diff-backed once the chapter's widening compares it.
- **Chapter thirteen's EXIT rests on the reading.** No draw is made off
  an Airbase, so the draw stream cannot see it, and run208 ran at
  `cover=0`. The train route is backed by the coverage traces.
- **The vslot `+0x34` gate.** The folded COMDAT names it
  `WallOut::get_gpiece@006424d0` (`render_gpiece`, `+0x68`). A host
  without a piece skips the order, and nothing else changes.
- **`data1` and `data2` are never read by `do_spec_anim`.** What they
  cue is the renderer's.

## 31. Chapter twenty-two — the launch line: a strike from inside a base (item 836)

**Premise.** A player's strike on an aircraft standing in its own Airbase
is **a `StrafeOrder` on the target, laid while the plane is inside, and
the base launches it only on a full tank**: `Object::do_launch@0064f3b0`
puts it out through `Unit::come_out@00617c10`'s tail, `do_spec_anim`'s EXIT
at an `AIRBASE` (§30), onto the base's point less `0xc0`, and it flies the
strike in the same call. This is the arm of `SpecialAnimOrder` no traced
game reaches (§30), and parked 761's route and 765's tank are its build.

**The issuer, under the emulator first** (a scratch script on
`tools/explore/command_oracle.py`'s fixture). `issue_flight(group, 2006,
1, ATTACK, 0, 0, 0)` appends the same 30 bytes whether the captain stands
on the map (`inside_up` −1), inside `0/2007` at `mana_burn` 24, or inside
full: the 5-byte `group` and the 25-byte `flight` (0x1c), as §25 found.
It writes no unit; the reuse is 28 bytes; `use_mp_playback`, `semaphore &
0x10` and `semaphore & 4` append nothing. **What the emulator cannot
reach** is all of the chapter: `Group::action_flight@006fb260`'s inside
arm at process time, and `do_launch` in the base's `Build::process`.

**The staging, read off run223** (chapter seventeen's capture, the same
game to 764; parked 821: every predicate is a printed field, so no
staging run was taken and run266 was not used).
- `0/6` is inside `0/2007` from block 722 (`inside_up 2007`) at (11424,
  14005). The base stands at (11616, 13920) with `inside_down 6`,
  `launch_frames 15` and `build_masks` 4232, whose bit 8 is what makes
  `Build::process@0061edf0` call `do_launch`.
- **The tank** (`mana_burn`): +1 a block on the map from the `add` (1 on
  611), 112 on 722, −2 a block inside, 24 on 766, **0 first on 778**.
- **The Barracks `1/2006`**: `ever_seen` 2 from 618 and **3 from 762**
  (who=0's bit, set while the pair patrol); damage from 822; gone on 1080.
- **The reach**: `vector_dist(9504, 2592)` = 9857, against `mana` 400
  (the `FIGHTER` row, type 289) times `get_speed` 75.

**The reading.**
- **`action_flight`'s inside arm** (listing `6fbbb0`–`6fbea0`): a member
  not on a strafe takes its "inside" from `ObjectData::get_inside@00651a80`,
  here 2007. An `ATTACK` then needs, in order:
  - `Object::valid_target@00648ba0`: `ObjectData::valid_target_const@006472c0`
    asks the target's `BuildData::is_seen@0062e1a0`, the `ever_seen`
    byte; an air-domain attacker is not refused a building (the
    `ANTI_AIR` refusal is for a land or building attacker), and a
    Barracks is not capture-eligible;
  - the target's leader without `MISSILE_DEFENSE_BONUS` (`has_preq`,
    0x2b5), unless the target is one's own;
  - the reach: `vector_dist(base − target) ≤ UnitData::mana@00609a50 ×
    get_speed(x, y, 1)` (vslot `0x17c`, its three pushes at `6fbcd2`);
  - not a `NUCLEARMISSILE` (`is(0x13b)`), then `is_ally ||
    war_allowed`.
  It gives `add_strafe_order(2006, 1, 2007, 0, 1, QUEUE_NEW, 1)`: a
  strafe on the Barracks, `mandatory 1`, flags 4, home the base,
  `returning 0`, `xx/yy` the target's point.
- **A plane inside runs no `work`**: `Unit::process@00610bc0` takes its
  inside arm (the tank's refill and `last_z = z`) and returns.
- **`do_launch`** (the decompile, and the listing where it names a slot):
  `launch_frames` (`+0x41`) is counted up and, once at
  `FRAMES_BETWEEN_LAUNCHES` (15, the PE's `.data` at `0xc06248`),
  saturates there while the chain is walked from `inside_down`:
  - a plane with no order, or with `mana_burn ≠ 0`, is passed over;
  - one whose current order has the action bit (or a base whose vslot
    `0xf0` says so) stays: a targeted strafe is killed only if its target
    is invalid **and** its point is off the world; it joins `launching`
    (`+0x44`), and the first of the call is `come_out(0)` and
    `launch_frames` 0;
  - one without is `kill_current_order`ed and dropped from `launching`.
  Units are processed before buildings (`Objects::process_all`, SYNC
  §3.2), so the launch shares the block the tank first reads 0.
- **`come_out`** places the plane off the building's ring, spares a
  fixed-wing plane its orders (`close_orders` is for a non-air type or a
  helicopter), draws nothing for a unit without `unit_masks & 0x40000`,
  and hands its tail to `do_spec_anim`: the plane faces 0, is placed at
  (11424, 13920), its figure's bank and pitch zeroed; the kill drops it
  from `launching`; then `work`, and `do_strafe` flies the first step.
- **The tank again**: +1 a block from 779; 400 on 1178, when
  `Unit::check_fuel@005e9be0` sets `returning 1`; the flight home and a
  second `land_plane`.

**The cast** is chapter seventeen's (§25). **Lines**: `chapter17.cmd`'s
eleven, and **`766 @strike 0 2006 1 6`**: the Fighter alone, inside, at
the Barracks, with its tank at 24.

**The capture must dump**
`end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5` and
`misc:COMMANDMANAGER=1` over `[605, 1500)`, beside run105's `start:` set:
run255's line. 895 blocks; the parting is expected on 768 and 778, so
the runway past it is 722 blocks, and the last falsifier is the second
landing, near 1330 by the reading.

**The premise's killer, and its writers** (§3, point 5): `0/6`'s order
stack and `inside_up` on blocks 768–778. The class's one adder for a
plane inside is `action_flight`'s inside arm; `do_launch` kills an
unflagged order and an invalid strike; `come_out`'s `close_orders`
spares a fixed-wing plane. **The loops**: `action_flight`'s members to
`group.num` (1); `do_launch`'s chain from `inside_down` to the first −1
(`0/6` alone).

**What would falsify it, and where each could first fire.**
1. **The issue does not reach the pump.** Trace frame 766: an `INFO 17`
   with a refusal, or no `process_flight` on 767.
2. **The inside strike is refused.** Block 768: `0/6` with no order, or
   anything but one `STRAFEORDER` on `1/2006`, `mandatory 1`, flags 4,
   `AIRORDER oxx 2007 whose 0 cruising_alt 1600 returning 0`, `xx/yy`
   (21120, 16512), with `0/6` still `inside_up 2007`.
3. **The tank does not gate the launch.** It splits three readings:
   `0/6` on the map on 768 (no gate), on 778 (the reading), or on 779
   (the gate a frame behind).
4. **The launch is not the EXIT's placement.** Block 778: `0/6` more
   than a step and a half (112) from (11424, 13920); `0/2007`'s
   `launch_frames` not 0 on 778 and 1 to 15 on 779 to 793; any
   `SPECIALANIMORDER` on any block.
5. **The strike does not fly at its target.** From 778: `0/6`'s current
   order not its `STRAFEORDER` on `1/2006` while the Barracks stands and
   the tank lasts; no round of `0/6`'s on `1/2006` (`AMMO`) by 1178.
6. **The tank is not the reading's.** `0/6`'s `mana_burn` not 1 on 779
   and 400 on 1178, or `returning` not 1 on 1178.
7. **It does not come home.** `0/6` not inside `0/2007` again by 1499.

Falsifiers 2 to 4 test the premise's own unit, `0/6`'s stack and
`inside_up` on its own blocks (711), and 3 splits the readings (parked
789).

**Where it should part.** This crate gives a member inside a base no
strike (`docs/ORDERS.md` §32's SEAM), carries no `launch_frames` and
builds no `do_launch`. So the first value parting expected is **768**,
`0/6`'s stack, and the first draw parting **778**: the Fighter's
`cruising_alt` redraw at `Unit::do_air_physics+0xba`, `(6 + 778) & 7 ==
0`, a draw the original spends and this crate does not.

**Run 2026-09-26 as run265 (item 836)** (`docs/RUNS.md` has the
tables). One take, `cover=0`, the same game as run223 to 778.
- **The premise holds.** `process_flight 767`; on 768 `0/6` holds one
  `STRAFEORDER` on `1/2006` (`mandatory 1`, flags 4, `AIRORDER oxx 2007
  whose 0 cruising_alt 1600 returning 0`, `xx/yy` (21120, 16512)) and is
  still inside (falsifier 2 did not fire).
- **The tank gates the launch** (falsifier 3 split the readings): inside
  with its order on 768–777 while `mana_burn` falls 20 → 2, and out on
  **778**, the block it first reads 0 — not 768, not 779.
- **The EXIT's placement** (falsifier 4): the figure on (11424, 13920),
  `last_z` 157, `last_bank` and `last_pitch` 0.0, `angle 0`; the unit one
  step north at (11424, 13845); no `SPECIALANIMORDER` on any block.
  **Its `launch_frames` clause fired**: the base reads 0 on 778 and on
  every block after, because the whole of `do_launch`, the counter
  included, sits under `inside_down ≥ 0` and the base is empty. The
  reading had put the increment outside that test.
- **The strike, the tank and home** (5 to 7 did not fire): 36 rounds of
  `0/6`'s on `1/2006` from 924, a point of damage each; the Barracks
  gone on 1080 as in run223, and the strafe an `AIRPATROLORDER` from
  1081 (§25's dead-target arm); `mana_burn` 1 on 779 and 400 on 1178,
  with `returning 1` that block; **inside `0/2007` again on 1385**, the
  patrol still on its stack.
- **`Object::do_launch` executed**, by the dump (the base's
  `launch_frames` 15 → 0 and `inside_down` 6 → −1 on 778); run265's trace
  is `cover=0`, so no coverage names it.

**Where this crate parted: `GOLDEN_WORD_CHAPTER_TWENTY_TWO` = 778, open**,
as predicted: the original spends the launched Fighter's `cruising_alt`
redraw (`Unit::do_air_physics+0xba`), 5 draws against 4 at draw 0. **The
value diff**: on 768 `0/6`'s stack, 0 orders here against the original's
`STRAFEORDER` on `1/2006` laid inside; on 778 the launch's 13 keys. The
widening is run265 whole, (605, 1501), in `WIDENINGS`, with five windows
in the coverage driver (768, 778, 924, 1178, 1385).

**The launch line built: `GOLDEN_WORD_CHAPTER_TWENTY_TWO` = 923, open**
(item 836, `docs/ORDERS.md` §38; parked 761 and 765).
- **The value diff on the old word's frame, 778, both sides**: `0/6` out
  of `0/2007`, its figure on (11424, 13920) at `z`/`last_z` 157 before
  the step, `last_bank` and `last_pitch` 0.0, `avg_speed` 25, heading 0,
  the unit at (11424, 13845); the base's `launch_frames` 0. On 768 both
  hold the `STRAFEORDER` on `1/2006` inside; the tank agrees on every
  block of every aircraft (`run265_s_launch_is_the_original_s_field_for_field`,
  39,793 rows), and chapter seventeen's pair now turn home on 1212 and
  1214 as the original's do.
- **The word is the Fighter's first attack**, 923: the original spends
  `Guy::set_anim+0xf2f < Unit::set_anim+0x56`, 6 draws against 5 at
  draw 0; on 924 its `0/6` has `cur_anim` 12 and `recharging` 31, ours
  neither.
- **The first value parting is 798**, before the word: the climb out of
  the base, `0/6`'s `pitch` 40.0 here against 38.0 there (bits 1109393408
  / 1108869120), `last_speed` 38 against 40, its point a unit or two off;
  its heading parts on 813 (1390007340 / 1391742680). ~~**798 is the
  chapter's next frame**, in `pitch_aircraft`'s non-returning arm.~~
  Upstream of the word, by item 842 (below).

**The strafer built: `GOLDEN_WORD_CHAPTER_TWENTY_TWO` = 1500, closed**
(item 842, `docs/ORDERS.md` §39).
- **Whose sixth draw on 923**: the original's. Its Fighter plays
  `CHAR_ATTACK2` from `do_strafe+0x9d0` (`Guy::set_anim+0xf2f`); ours was
  still 387 units short, at (19523, 15869) against (19910, 16016), and
  released on 929.
- **The chain from 798.** The Fighter's type carries flag `w`
  (`unit_flags & 0x400000`, "strafes targets"). On a strike at a
  standing ground target within 90° of the nose, `pitch_aircraft` wants
  **half** its `cruising_alt` over the ground ahead (`0x5e90ff`–`0x5e910c`).
  - On 797 both sides stand at `z` 677, pitch 40.0, `cruising_alt`
    1600, heading 361709932.
  - On 798 the original's rate is `((800 + 191 − 677) / 25 · 20) / 7` =
    34, so its pitch falls 40 → 38. Ours wanted 1791, rate 125, and held
    40.
  - Ours then climbed longer under the pitch speed cut (`last_speed` 38
    against 40), fell behind, and released six frames late.
- **The value diff on the word's frames, both sides, after the build**:
  - on 798, `0/6`'s `pitch` 38.0 (bits 1108869120), `z` 724,
    `last_speed` 40, at (11504, 12621);
  - on 924, `cur_anim` 12, `end_time` 30, `recharging` 31, at (19979,
    16045);
  - on 923, the draws `Guy::set_anim+0xf2f`, then five
    `Farms::inc_time`.
  - With the half alone, 923 parted at draw 1: ours then spent two
    `Ammo::init` scatter draws (`+0xcd9`, `+0xd0b`) that the original
    does not. A strafer's round is exact (`0x67c33a`).
- **The stream agrees to run265's end.** What the widening still parts:
  - **The Barracks' `damage` on 928**, ours 571 + 8/16 against 572 + 0.
    The original fires **two** rounds a release event, one per gun
    node. It walks each landing `(cur/end − 0.3)·6·192` along the
    heading and 48 to the node's side. Ours fires one, on the target.
    Parked, for `anim.rs`'s release walk (§39.3).
  - ~~**The landed plane's patrol**, `0/6` on 1385 and `0/7` on 1489: one
    order there, none here. `land_plane` keeps the order under a home
    whose `build_masks & 0x80` is set (`has_repeat_air`), which run265's
    Airbase carries (4232). `Building` carries no `build_masks`. Parked.~~
    Closed by item 854 (ORDERS §40): `Building::repeat_air`, set by
    `Build::init` for every `can_carry(AIR)` building. Both patrols now
    agree, flags 0.

## 32. Chapter twenty-three — the repeat line: an Airbase's repeat toggled off between two landings (item 867)

**Premise.** The player's repeat button on an Airbase **toggles its
`build_masks & 0x80`**, off here (under the emulator,
`Group::action_buildmask` takes 4232 → 4104 and 4104 → 4232, whatever
`set` says), and a base without it keeps no air order, by two arms: **a
plane that lands there loses its orders at the landing**
(`Unit::land_plane@005e9950`'s clear arm, `docs/ORDERS.md` §40.1), and **a
plane already inside with an unflagged order loses it at its full tank**
(`Object::do_launch@0064f3b0`'s kill, §38.3), and stays inside. The one
in-game writer of the bit is this button:
`Options::set_air_repeat@0071c740` → `GroupOut::issue_buildmask@00708820`
→ `CommandManager::issue_buildmask@00941f80` →
`CommandPackage::process_buildmask@00947680` →
`Group::action_buildmask@006fc9a0` (and `WallData::valid_buildmask@0063e2a0`).

**The booking's premise was one arm of this.** It read the unflagged
patrol off a non-repeating base as killed at a full tank. That is
`do_launch`'s arm, and it needs a plane that is inside with its order when
the bit goes; a plane that lands after never reaches it, because
`land_plane` closes its orders first. So the toggle is placed **between**
chapter twenty-two's two landings, and one lever reaches both arms: `0/6`
landed on 1385 under the bit with its patrol kept, and `0/7` lands on 1489
without it.

**The issuer, under the emulator first** (a scratch script on
`tools/explore/command_oracle.py`'s fixture, with a building on
`Build::vftable` in who=0's registry).
- `issue_buildmask(group [b], 0x80, set)` appends 14 bytes: the 5-byte
  `group` and a 9-byte `buildmask`, type 0x21, `[mask i32][set i32]`. The
  reuse is 12. **The wire's `set` is 1 whatever the third argument**:
  `941fa6` stores the constant, and the listing never reads `[ebp+0x10]`.
  A unit and a building in one selection both enter the `group`. It
  writes no object.
- **`action_buildmask` toggles; it does not set.** On a group whose
  `buildings` byte (`+0x49`) is 1, with `can_carry(AIR)`
  (`ObjectData::can_carry@00646c40`, hooked) answering 1: 4232 → 4104 and
  4104 → 4232, **with `set` 1 or 0** (the listing never reads
  `[ebp+0xc]`). Each member `valid_buildmask` admits is set if it lacks the
  bit and every member before it was set; otherwise it is cleared, and so
  is every member after it. With `buildings` 0, with `can_carry(AIR)` 0,
  or with the mask 0x08, it writes nothing.
- **What the emulator could not reach**: `process_group`'s `Group::add@
  00714350` (which sets `buildings` from the member's vslot `0x1c`) and its
  `Groups::push_group@0070f9e0` of the building group, and
  `process_buildmask`'s logging, all at process time.

**The staging, read off run265** (chapter twenty-two's capture, the same
game to 1500; parked 821: every predicate is a printed field, so no
staging run was taken and run282 was not used).
- `0/2007`: `build_masks` 4232 on every block.
- `0/6`: inside `0/2007` from 1385 at `mana_burn` 400, **286 on 1442**,
  −2 a block, so **0 first on 1585**. One `AIRPATROLORDER`, flags 0, `oxx`
  2007, point (21120, 16512), `cruising_alt` 1400, `returning` 0: kept by
  `land_plane` under the bit.
- `0/7`, `0/8`: on the map on 1442, each an `AIRPATROLORDER` home with
  `returning 1` and `mana_burn` 600, the Bomber's cap. `0/7` is inside on
  1489 in run265 (then 600, full near 1789); `0/8` is not inside by 1499.

**The reading.**
- **The toggle.** `process_group` builds and pushes a group of the one
  building (force 1). `process_buildmask` logs `unitmask: 128 set: 1` and
  calls `action_buildmask(0x80, 1)`: the Airbase can carry aircraft, its
  bit is set, so it is cleared. `build_masks` 4104 from 1442.
- **The pool.** `push_group` takes a fresh who=0 slot for the building
  group (`Groups::get_open_slot@006fa460`), and `GROUPDATA` prints it with
  `buildings 1` and `[2007]`. `get_open_slot` counts a slot whose
  `buildings` byte is set as open, so the next push may take it back. No
  push follows here.
- **`land_plane` off a non-repeating base** (`005e9a43`, vslot `0xf0`
  clear): `unit_masks &= ~0x4000000`, the path emptied, `close_orders`,
  `clear_partial_path`, `update_action`; then the `SpecialAnimOrder` and
  `go_inside` as always. `0/7` is inside on 1489 with no order, and `0/8`
  likewise on its landing.
- **`do_launch` at a full tank**: once `launch_frames` is saturated, a
  plane in the chain with an order and `mana_burn` 0 is launched only if
  `has_repeat_air() || flags & 4`; otherwise `kill_current_order` and out
  of `launching`. `0/6`'s patrol has flags 0, so it goes on 1585 (units
  are processed before buildings, SYNC §3.2), and `0/6` stays inside with
  no order. `0/7` at its full tank has none to fly.

**The cast** is chapter twenty-two's (§31). **Lines**: `chapter22.cmd`'s
twelve, and **`1440 @buildmask 0 128 2007`**: the repeat button on the
Airbase alone. The DLL's `@buildmask <who> <mask> <b>…` is new
(`tools/trace/tracer.c`, verb 17): the registry check is `@eject`'s (a
live `Build` of `who` with that id), the prologue `55 8b ec 83 ec 0c b9
60 ff e8 00`, the command 9 bytes, and `set` 1.

**The capture must dump** run265's line,
`end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5` and
`misc:COMMANDMANAGER=1`, over **`[605, 1840)`**: 1,235 blocks. The
toggle is on 1442 and the last falsifier is `0/6`'s full tank on 1585,
so the runway past it is **255 blocks**; `0/7`'s full tank near 1789 is
inside the window.

**The premise's killer, and its writers** (§3, point 5): `0/7`'s order
stack on its landing block and `0/6`'s on 1585. The writers: `land_plane`'s
clear arm and its keep (`flags &= ~4`); `do_launch`'s kill and its launch
(`come_out`); no command after 1440. **The writers of `build_masks &
0x80`**, grepped by `+0x60` in every spelling (`build_masks`,
`field_0x60`, `(x + 0x60)`; parked 869): `Build::init@00629740:280` (`|=
0x88`), `action_buildmask`, `ScenarioFuncSet::repeat_orders_enable@
009f99e0` and `_disable@009f9ad0`, `UnitBalance::next@009b8ac0`, the
zeroing `Wall::Wall@0063e390`, `Wall::init@0063e9b0` and
`BuildData::BuildData@0062f370`, and `Wall::swap_team@00640c00`'s copy.
`Wall::process@00640450`'s whole-word store at line 52 keeps the bit
(it touches `0x10` and `0x20`). Only `action_buildmask` runs in the window.
**The loops**: `action_buildmask`'s members to `group.num` (1);
`do_launch`'s chain from `inside_down` to the first −1 (`0/6`, then `0/7`
and `0/8` as they land).

**What would falsify it, and where each could first fire.**
1. **The issue does not reach the pump.** Trace frame 1440: an `INFO 17`
   with a refusal, or no `process_buildmask unitmask: 128` on 1441.
2. **The toggle is not the reading's.** `0/2007`'s `build_masks` on 1442.
   It splits three readings: 4232 (no toggle: `set` read as a value, or
   the base refused), 4104 (the reading), or any other value (another
   bit). And it must stay 4104 to the end.
3. **A landing off a non-repeating base keeps its order.** `0/7`, inside
   on 1489. It splits three readings: no order (the reading); its
   `AIRPATROLORDER` kept with flags 0 and killed at its full tank (the
   booking's reading); or kept and relaunched then (the bit is no gate).
   The same for `0/8` at its landing.
4. **The waiting patrol is not killed at its tank.** `0/6`'s patrol, flags
   0. It splits four readings: gone on 1442 (the toggle kills it); kept to
   1584 and gone on 1585 with `0/6` still `inside_up 2007` (the reading);
   launched on 1585 (`inside_up` −1, on the EXIT's point); or kept past
   1585.
5. **Something launches.** Any aircraft out of `0/2007` after 1442, or any
   `SPECIALANIMORDER` on any block.

Falsifiers 3 and 4 test the premise's own unit, each plane's stack on its
own block (711), and each splits the readings (parked 789).

**Where it should part.** This crate has no entry for the command, so
without it the first value parting is **1442**, the harness's
`0/2007 build:repeat_air` (ours 1, theirs 0), and then **1489**, `0/7`'s
stack (ours the kept patrol, theirs none). The pool gains a slot this
crate does not seat. On 1585 this crate's `do_launch` already kills an
unflagged order whatever the bit (§40.3), so `0/6` agrees there. No
draw is expected to part: neither `close_orders` nor `kill_current_order`
draws, and no plane flies. With the command entered, the reading says
this crate agrees on every plane.

**Run 2026-09-26 as run281 (item 867)** (`docs/RUNS.md` has the tables).
One take, `cover=0`, the same game as run265 on all 1,501 frames the two
share (`rngcmp.py`: 0 differing).
- **The issue** (1 did not fire): `INFO 17` refusal 0, the package 10 →
  24 bytes; `process_group, new 0 1 1441`, `process_buildmask 1441`.
- **The toggle** (2 did not fire; it split the readings to the second):
  `build_masks` 4232 → **4104 on 1442**, and 4104 to the end.
- **The landings** (3 did not fire): `0/7` inside on **1489** with no
  order, and `0/8` inside on **1513** with none. `land_plane`'s clear arm;
  the booking's reading (kept, then killed at the tank) is killed.
- **The waiting patrol** (4 did not fire): `0/6`'s `AIRPATROLORDER`, flags
  0, on every block 1385–1584; on **1585**, the block its `mana_burn`
  first reads 0, no order and still `inside_up 2007`. `do_launch`'s kill.
- **Nothing launches** (5 did not fire): `inside_down` 6 on every block
  from 1385; `0/7` full on 1789 and `0/8` on 1813, neither out; no
  `SPECIALANIMORDER`.
- **The pool**: the building group takes who=0's slot 0 on 1442 (`num 1`,
  `buildings 1`, `stamp 1441`) and keeps it to 1839.

**Where this crate parted: `GOLDEN_WORD_CHAPTER_TWENTY_THREE` = 1840,
closed, on the first walk**, as predicted: with the `@buildmask` line
skipped, nothing draws. The widening, run281 whole, (605, 1841), in
`WIDENINGS`, with five windows in the coverage driver (1442, 1489, 1513,
1585, 1837) and `process_buildmask` pinned there. **The value partings**,
both sides:
- **1442**, `0/2007`'s `build_masks & 0x80`: ours 1 (`repeat_air`),
  theirs 0 (4104);
- **1489**, `0/7`'s stack: ours its `AIRPATROLORDER` (flags 0), theirs
  none;
- **1513**, `0/8`'s stack: the same.
- 1585 agreed already: this crate's `do_launch` killed an unflagged order
  whatever the bit.

**The command entered** (item 867, `docs/ORDERS.md` §41): `Sim::
action_buildmask`, the toggle, from `input::group_buildmask`; and
`do_launch`'s repeat arm, `has_repeat_air() || flags & 4`. **The value
diff after, both sides**: on 1442 `0/2007`'s bit is 0 on both (theirs
4104); on 1489 `0/7` is inside `0/2007` with no order on both, and on
1513 `0/8` likewise; on 1585 `0/6` is inside with no order and
`mana_burn` 0 on both. The widening goes **14 → 9 rows**: what stands is
chapter twenty-two's births and chapter one's clocks (611–655). The pool
keeps chapter twenty-two's ten rows ~~and one more, **1442 slot 0 held**
(theirs `[2007]`): this crate's pool holds units only. Its reader is
`Groups::get_open_slot@006fa460`, which counts the slot as open to the
next push, and no push follows; no later slot parts by it~~ — **the
building group is seated since item 882** (`process_group`'s push at the
command, §33), and slot 0 agrees on 1442. The word stays **1840,
closed**.

## 33. Chapter twenty-four — the queue line: a Barracks' infinite queue toggled on between two finishes (item 877)

**Premise.** The player's infinite-queue button on a production building
**toggles its `build_masks & 0x40`**, and only while a **train job** is
queued there: `WallData::valid_buildmask@0063e2a0` admits 0x40 when
`BuildData::can_infinite@0062d4d0` finds a unit type (0x32..0x19d) with
its availability bit set in the queue of a training building
(`BuildTypeData::is_training_building`, `build_flags & 0x80000000`).
With the bit set, **a finished train job re-queues itself at the end of
the queue, paid again**, and the bit survives the queue emptying under it;
a re-queue the stockpile refuses leaves the bit off. This is
`Build::do_queue@0061e410`'s arm at `61ec24` (`docs/PRODUCTION.md`,
"Completion"): the word is read after `finished` answers > 0 and before
`unqueue(i, 0)`, which clears 0x40 when `queued` reaches 0
(`Build::unqueue@006207c0`); then, for a train job with the bit read,
`&= ~0x40`, `Build::queue_up@00620f40(type, 0)`, and `|= 0x40` on
success. The in-game writers of the bit are this button —
`Options::exec@007188c0`'s option 0x40 → `GroupOut::issue_buildmask@
00708820` → `CommandManager::issue_buildmask@00941f80` →
`CommandPackage::process_buildmask@00947680` → `Group::action_buildmask@
006fc9a0` — and the queue's own clears (`unqueue`, `clean_queue@00620b60`,
`action_unqueue@00620280`). No AI function reads or writes it.

**A claim checked, not a premise**: the booking's reading, that `0x40`
is the player's repeat production and that a finished unit re-queues
itself under it, is the listing's; the emulator adds the gate. The
premise names three arms of the completion, and **the toggle is placed
between two finishes** (parked 879) so one capture buys all three: a
finish with the bit clear (the Hoplites, 848), a finish with it set and
the re-queue paid (the Bowmen, 1052), and a finish with it set and the
re-queue refused (the Bowmen again, 1264). A fourth line buys the gate's
refusal: the same button on an empty queue (1302).

**The issuers, under the emulator first** (a scratch script on
`tools/explore/command_oracle.py`'s fixture, with a `Build` on
`Build::vftable` in who=0's registry, its `BuildTypeData`, the type list
and the leader's bitmask synthesized).
- `CommandManager::issue_queue_up@00941be0(group [b], type, num)`
  appends the 5-byte `group` and a 9-byte `queue_up` (type 0x18,
  `[type i32][num i32]`), 14 bytes, each as passed. It tests nothing of
  the type. It writes no object.
- `issue_buildmask(group [b], 0x40, set)`: 14 bytes with `set` 1,
  whatever the third argument (§32 again).
- **`valid_buildmask`'s answer, by state** (the masks 0x40, 0x80, 0xc0,
  0x08, 0x100):

  | the building | 0x40 | 0x80 |
  | --- | --- | --- |
  | not a training building, a train job queued | 0 | 0 (1 with `can_carry(AIR)`) |
  | training, queue empty | 0 | 0 |
  | training, one research entry (the bit clear) | 0 | 0 |
  | training, one train job (the bit set) | **1** | 0 |
  | training, a research entry then a train job | **1** | 0 |
  | training, one tech entry (type 0x250, its bit set) | 0 | 0 |

  0xc0 answers as 0x40 or 0x80 does; 0x08 and 0x100 are never admitted.
- **`action_buildmask(0x40)` toggles**: on an admitted Barracks 4104 →
  4168 and 4168 → 4104 with `set` 1 or 0; 4296 → 4232 (0x80 untouched).
  With only a research entry, or an empty queue with the bit set (4168),
  it writes nothing.
- **`Build::unqueue(0, 0)`**: a one-entry queue with 0x40 empties and the
  bit clears (4168 → 4104); on a two-entry queue the bit stays.
- **What the emulator could not reach**: `do_queue`'s completion arm (it
  trains a unit), read from the listing (`61ec17` `finished`, `61ec24`
  the word, `61ec35` `unqueue(i, 0)`, `61ec4f` the clear, `61ec59`
  `queue_up(type, 0)`, `61ec66` the set); `Group::action_queue_up@
  006fdbb0` (it sorts the members by `queued`, then calls
  `Build::queue_up(type, 1)` on each active, finished member `num` times,
  a missile silo asking `can_carry(type)` first); and `process_group`'s
  push of each building group, at process time.

**The readers of `build_masks & 0x40`**, by `+0x60` in every spelling
(parked 869): `do_queue`, `unqueue`, `clean_queue`, `action_unqueue`
(the player's cancel clears the bit and, for a single cancel, returns
without removing the entry), `ScenarioFuncSet::toggle_infinite_queue@
009f45e0` (out of v1), and the interface's `IFaceSelected::update_queue`,
`IFaceOptions::setup_button`, `GroupData::get_infinite_queue` and
`GroupOut::issue_queue_up`'s queue-full message. **None is the AI's.**
The other `&= ~0x40` stores in `come_out`, `action_alarm`, `Wall::start_me` and
`BuildType::mask_me` are a city flag or a tile mask.

**The cast** is chapter thirteen's to 619 (run208 is this game there):
`!ai off`, who=0's Barracks `0/2007` at tile (14, 74), `build_masks` 4096,
a Chariot and a Hoplite squad. The Barracks trains Scouts, Slingers,
Hoplites (132) and Bowmen (170), each with its availability bit set, so
every entry here is a train job. **Lines**: `620 @queueup 0 132 1 2007`;
`640 @queueup 0 170 1 2007`; `900 @buildmask 0 64 2007`; `1300
@buildmask 0 64 2007`. The DLL's `@queueup <who> <type> <num> <b>…` is new
(`tools/trace/tracer.c`, verb 18): `@buildmask`'s registry check, the
prologue `55 8b ec 83 ec 0c b9 60 ff e8 00`, the command 9 bytes. An `@`
line on trace frame F is on block F+2 (§17).

**The staging, walked by this crate on run208's start** (a scratch walk:
`Sim::queue_up` by hand on the processed frames, the re-queue emulated;
run286 was not used). Every value below is this crate's:
- 619: who=0 holds 253 food, 240 timber, 113 wealth; pop 8 of 25.
- **622**: `[132]`, `job_counter` 100; food 204, timber 204.
- **642**: `[132, 170]`; wealth 63.
- **848**: the Hoplites out, a Hoplite trained; `[170]` at 0; no re-queue.
- **902**: `build_masks` 4160.
- **1052**: the Bowmen out, a Bowman trained; `[170]` at 0, paid 46
  timber and 56 wealth out of 72; `build_masks` 4160.
- **1264**: the Bowmen out again; 24 wealth against 56: refused; the
  queue empty; `build_masks` 4096.
- **1302**: the second press on the empty queue: 4096.

The margins are what the staging rests on: the toggle is 54 blocks after
the Hoplites' finish and 150 before the Bowmen's (a train job's clock is
~226 blocks here and is compared on every capture), and the first re-queue
has 16 wealth over, the second 32 short.

**The capture must dump** `end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1`
and `misc:COMMANDMANAGER=1` over **`[605, 1560)`**: 955 blocks. The last
falsifier is 1302's, so the runway past it is **258 blocks**.

**The premise's killer, and its writers** (§3, point 5): `0/2007`'s
`build_masks` and `BUILDQUEUE` on 848, 902, 1052, 1264 and 1302, and
who=0's `bucket`s there. The writers of the bit are listed above; of the
queue, `queue_up` (the command's and the re-queue's), `unqueue` (the
finish's), and nothing else in the window: no cancel, no capture, no
defeat. **The loops**: `action_buildmask`'s and `action_queue_up`'s
members to `group.num` (1); `num` 1; `can_infinite`'s entries to
`queued` (1 or 2).

**What would falsify it, and where each could first fire.**
1. **The issues do not reach the pump.** Trace frames 620, 640, 900,
   1300: an `INFO 17` with a refusal; or no `process_queue_up type: 132
   num: 1` on 621, `170` on 641, no `process_buildmask unitmask: 64` on
   901 and 1301.
2. **The queue-up is not the reading's.** `0/2007`'s `BUILDQUEUE` on 622
   and 642: anything but `[132]` then `[132, 170]`, each paid once
   (`queue[k].cost`), `queue[0].job_counter` 100 on 622.
3. **A finish with the bit clear re-queues** (arm 1). The block the
   Hoplites' entry leaves (848 by this crate): `[170]` alone. It splits
   two readings: a 132 behind the 170 (every train job repeats, or the
   bit is read as set), or none (the reading).
4. **The toggle is not the gate's.** `build_masks` on 902. It splits
   three readings: 4096 (refused: the Bowmen not a train job, or
   `can_infinite` not the gate), 4160 (the reading), or another value.
5. **A finish with the bit set does not re-queue, or loses the bit**
   (arm 2). The block the Bowmen's entry leaves (1052 by this crate): it
   splits three readings: `[]` and 4096 (the bit is no gate), `[170]` at
   0, paid again, and 4096 (the re-queue made but `unqueue`'s empty clear
   left standing), `[170]` at 0, paid again, and 4160 (the reading). A
   re-queue at the front against the end cannot be split here: the queue
   is empty when it is made.
6. **A refused re-queue keeps the bit** (arm 3). The block the second
   Bowmen's entry leaves (1264): `[]` with 4096 and wealth unspent (the
   reading); `[]` with 4160 (the bit kept on a refusal); or `[170]` (the
   price not asked).
7. **The gate admits an empty queue.** `build_masks` on 1302: 4096 (the
   reading) or 4160.
8. **The finish does not train.** A new who=0 Hoplite on 848 and a new
   Bowman on 1052 and 1264, each out of the Barracks. The arm
   independent of the bit: trained whether or not re-queued.

Falsifiers 3, 5 and 6 test the premise's own unit, one entry's finish on
its own block (711), and each splits the readings (parked 789).

**Where it should part.** This crate enters neither command: the harness
skips `@queueup` and `@buildmask` 0x40 writes nothing
(`Sim::action_buildmask` carries 0x80 alone). So the first value parting
is **622**, `0/2007`'s `queued` (ours 0, theirs 1) and who=0's food and
timber; the draw stream should first part at the Hoplite's training
(~848), a birth this crate does not make. The pool gains a building group
at each command, which this crate's pool of units does not seat (§32's
standing row).

**Run 2026-09-26 as run285 (item 877)** (`docs/RUNS.md` has the tables).
One take, `cover=0`, the same game as run208 to 640.
- **The issues** (1 did not fire): `INFO 17` refusal 0 on all four;
  `process_queue_up 132 1 621`, `170 1 641`, `process_buildmask` on 901
  and 1301.
- **The queue-up** (2 did not fire): `[132]` at 100 on 622, `[132, 170]`
  on 642, each paid once.
- **The finish with the bit clear** (3 did not fire): the Hoplites run to
  23310 on 855 and leave on **856**; `[170]` alone, 4096.
- **The toggle** (4 did not fire; it split the readings to the second):
  4096 → **4160 on 902**.
- **The finish with the bit set** (5 did not fire; it split the three
  readings to the third): the Bowmen run to 20280 on 1059; on **1060**
  `[170]` at 0, timber 189 → 143 and wealth 70 → 14, **4160**. The re-queue
  survived `unqueue`'s empty clear.
- **The refused re-queue** (6 did not fire): the second Bowmen run to
  21030 on 1271; on **1272** `[]`, **4096**, wealth 19 unspent.
- **The gate on an empty queue** (7 did not fire): 4096 on 1302.
- **The trains** (8 did not fire): three Hoplites on 856 and three Bowmen
  on 1060 and on 1272, each squad in a pool group.

The staging walk had every finish 8 blocks early (848, 1052, 1264). ~~A
clock this crate runs short, which the widening names.~~ **Not this
crate's clock**: with the commands entered, the queue agrees with run285
on every block (below), so the 8 blocks were the scratch walk's own — it
queued by hand outside the command and did not stage chapter thirteen's
lines past 619 the same way; which difference it was is not established.
The arms did not depend on it: the toggle stood 46 blocks after the
Hoplites' finish and 158 before the Bowmen's.

**Where this crate parted: `GOLDEN_WORD_CHAPTER_TWENTY_FOUR` = 855,
open, on the first walk**, with both `@queueup` lines skipped and
`@buildmask` 0x40 writing nothing: on 855 the original's Hoplites finish
and train a squad this crate never queued, and the draws part there. The
widening, (605, 858), in `WIDENINGS`, with three windows in the coverage
driver (622, 642, 855) and `process_queue_up` pinned there. **The value
partings**, both sides:
- **622**, `0/2007`'s `queued`: ours 0, theirs 1; who=0's food and
  timber, ours 254 and 241, theirs 203 and 203;
- **642**, wealth: ours 114, theirs 61;
- **856**, `0/10`–`0/12`, the trained Hoplites: the dump holds them alone.
The pool: the building group `[2007]` in who=0's slot 1 on 622, and the
trained squad `[10, 11, 12]` in slot 0 on 856, neither seated here.

**The commands entered: `GOLDEN_WORD_CHAPTER_TWENTY_FOUR` = 1560,
closed** (item 877, `docs/PRODUCTION.md` "The infinite queue"):
`input::group_queue_up` → `Sim::action_queue_up`; the 0x40 arm of
`Sim::action_buildmask`, gated by `Sim::can_infinite`; `Queue::infinite`,
cleared by `Queue::unqueue` on the empty queue; and `do_queue`'s re-queue,
the bit read before the unqueue and `Sim::requeue_infinite` after the
train. The stream agrees to run285's end. **The value diff, both sides**,
on the blocks the reading named:
- **622**: `0/2007` `[132]` at 100; food 203, timber 203 on both (ours
  was 254 and 241 before the build).
- **642**: `[132, 170]`; wealth 61 on both (ours was 114).
- **856**, the old word's block: `[170]` at 0 and 4096 on both; the
  Hoplites `0/10`–`0/12` at (2712, 14904), (2712, 15048), (2856, 15000)
  on both (the dump held them alone before).
- **902**: 4160 on both.
- **1060**: `[170]` at 0, timber 143, wealth 14, 4160 on both; the Bowmen
  `0/13`–`0/15` born.
- **1272**: `[]`, 4096, wealth 19 on both; `0/16`–`0/18` born.
- **1302**: 4096 on both.

The widening is **run285 whole**, (605, 1561), with eight windows in the
coverage driver (622, 642, 855, 902, 1060, 1272, 1302, 1557). What stands,
34 rows: chapter thirteen's births' `form` (611, 615), and on each trained
squad (856, 1060, 1272) `Unit::come_out@00617c10`'s human arm — a squad
type (`+0x308` > 1) coming out of a building is `Group::add`ed and
`Groups::push_group`ed, so the original's members read `group` (the pool
slot: 0, 2, 1), `form` 0, and the followers' `orders_x/y` (2712, 14232)
against the Barracks' point (2688, 14208) here. This crate's pool does not
seat that push. The stream agrees to 1560 with it standing; the named
reader of `orders_x` is `Unit::check_target_path@005e22d0`, and nothing
targets these squads. **The pool**, four rows, each a push not seated
here: the building group `[2007]` in slot 1 on 622 and slot 3 on 1302,
the squads in slot 0 on 856 and slot 2 on 1060. The third squad takes
slot 1 back on 1272 (`Groups::get_open_slot@006fa460` counts the
building group's slot as open), which is why the second press's building
group lands in slot 3.

**Mutations**, each restored from git and `touch`ed: the bit read after
the unqueue fails `a_finished_train_job_requeues_under_the_bit` and drops
the word to 1271 (the second Bowmen never re-queued); `can_infinite`
answering for any queue fails `the_infinite_button_needs_a_train_job`
and parts the widening on **1302**, `0/2007 build:infinite_queue` ours 1
theirs 0 — the compare is read.

**The pool push built** (item 882, `docs/GROUPS.md` §31): `come_out`
pushes every trained squad, a computer's too (the booking's "human arm"
was 877's hypothesis, and the listing killed it), and the command's
`process_group` seats the Barracks' building group. The squads read
`group` 0, 2 and 1 and every slot's list and `stamp` agree. **34 → 25
rows, and the pool 4 → 8**: the 25 are `Unit::init@00612100`'s (`form`,
and the followers' tile-centred `orders_x/y`, parked 646). The 8 are
**a divergence this landing introduces**, a row for att-880 and not
built: each slot's `ox`/`oy` on its first seat (622, 856, 1060, 1302),
`push_group`'s record `o`, −1 here against 0 there; before the push the
slots were `held` rows and the record went uncompared. Its reader is a
layout's `o`, and no group in run285 is laid out. The word stays
**1560, closed**.

## 34. Chapter twenty-five — the cancel line: a Barracks' queue cancelled inside a run, across two types, on an infinite queue, and from the end (item 884)

**Premise.** The player's cancel, `CommandPackage::process_unqueue@
009466f0` → `Build::action_unqueue@00620280(p)` → `Build::unqueue@
006207c0(i, 1)`, **refunds the removed entry's recorded price** (`Build::
unpay_cost@006206e0`, its three `(good, cost)` pairs) and **walks forward
over a run of the entry's type**, so a cancel of the head of a run removes
the run's last and the head keeps its progress (`docs/PRODUCTION.md`,
"Cancelling refunds exactly what was paid"). `p` is a selector: a slot, or
a negative read as the last slot (−5 five, −10 all). **With `build_masks
& 0x40` set, a cancel first clears the bit and, for `p ≥ −1`, removes
nothing** (877's reading, `docs/PRODUCTION.md` "The infinite queue").
Each of these is a claim to check: the premise names four arms, and the
cast puts one cancel on each (parked 879).

**The cancel, under the emulator first** (a scratch script on
`tools/emu/callfn.py`'s machine: a `Build` on `Build::vftable` with its
queue, the leader's `num_queued`, tallies and stockpile, the type list's
`is_unit_type`/`is_age_type`/`is_epoch_type` slots, the options record,
and a console on another player; and `tools/explore/command_oracle.py`'s
fixture for the issuer). The listing was read for each arm (`6203c1`,
`6203ca`, `620411`, `62045d`, and `unqueue`'s walk at `6208b9`..
`62090c`), because five functions in a row have hit the decompiler's
dropped-register trap: the decompile prints all three of `action_unqueue`'s
`unqueue` calls with no arguments.
- `CommandManager::issue_unqueue@00942c40(b, p)` appends **15 bytes and
  no `group`**: `30 [who i32][o i32][p i32][uid i16]`, for p 0, −1, 2, −5
  and −10 alike. `BuildOut::issue_unqueue@00629e70` is the same behind a
  `semaphore & 0x10` gate; `Options::exec@007188c0`'s option 0xa6 calls it
  once per selected building, `p` the option's `object`, −5 with one
  modifier and −10 (`~PAPYRUS`) with the other.
- `unqueue(0, 1)` on `[132*, 132, 170]` (the head at 5000) removes **slot
  1**: `[132 at 5000, 170]`, and the stockpile takes back slot 1's recorded
  pairs. On `[132*, 132, 132]` it removes slot 2. On `[132*, 170]` it
  removes the head, and the Bowmen move up at 0. `unqueue(0, 0)`
  (completion) on the run removes the head with no walk and no refund.
  An entry recorded at 60 food 40 timber refunds 60 and 40, whatever the
  price now is.
- `action_unqueue(p)`, no bit: p 0 and 1 are `unqueue(p, 1)`; p −1, −2
  and 2 on a three-entry queue remove the last; −5 removes
  `min(queued, 5)` from the end (seven entries leave two, the head's 5000
  kept); −10 empties it.
- **With 0x40**: p 0 and −1 write the bit alone (4160 → 4096) and
  nothing else; −2 clears it and removes the last; −5 and −10 clear it and
  remove. On an empty queue with the bit set, nothing is written: the
  bit stays (`620353` tests `queued` first).
- **What each removal writes**: `queued` (+0x82), the removed entry's
  `job_counter` (0, before the tail is copied down), `num_queued[type]` at
  `leader + 0x5a22` (not below 0), the AI's per-building tallies
  (`docs/PRODUCTION.md`'s AI counters, by the unit's trainer; not below
  0), `ages_queued`/`epochs_queued` for an age or epoch, the leader's
  stockpile (encrypted, as every bucket is), and `options->rebuild`.
- **What it could not reach**: the forward to the first library (the
  synthesized building answers `is(LIBRARY)` 0) and its `DISBAND` (0x29a)
  exception; the console player's feedback and `S_INFINITE_QUEUE_OFF`;
  `process_unqueue`'s unit arm (`Unit::action_unqueue@005e1f20`).

**The writers of the fields**, by offset in every spelling (parked 869,
823). `queued`: `Build::queue_up@00620f40`, `unqueue`, `clean_queue@
00620b60`, `Build::close@00628980`, `Build::new_library@00627fe0` and the
`BuildData` constructor — six; in this window only the commands' `queue_up`
and `unqueue` (the cancels', and `do_queue@0061e410`'s completion) run.
`unqueue`'s callers: `do_queue` (twice, no refund), `action_unqueue`,
`clean_queue`, `Build::activate@00623e20` (three sites, with refund) and
the scenario editor — **none is the AI's**. `unpay_cost`'s: `unqueue` and
`clean_queue`. The bit's writers are §33's.

**The cast** is chapter thirteen's to 619 (run285 is this game to 640 but
for the second Hoplite): `!ai off`, who=0's Barracks `0/2007`, 4096.
**Lines**: `620 @queueup 0 132 2`; `640 @queueup 0 170 1`; `700 @unqueue 0
0`; `760 @unqueue 0 0`; `800 @buildmask 0 64`; `840 @unqueue 0 −1`; `980
@queueup 0 132 1`; `990 @queueup 0 170 1`; `1000 @unqueue 0 −1`, each on
2007. The DLL's `@unqueue <who> <p> <b>…` is new (`tools/trace/tracer.c`,
verb 19): the prologue `55 8b ec 83 ec 10 b9 60 ff e8 00`, 15 bytes a
building. An `@` line on trace frame F is on block F+2 (§17).

**The staging, walked by this crate through the commands' own entries**
(`input::group_queue_up`, `input::unqueue`, `input::group_buildmask`, on
run285's start; parked 885: a walk outside the entry was 8 blocks off in
§33). run293 was not used. Every value below is this crate's:
- **622**: `[132 at 100, 132]`, paid 51 food 38 timber, then 53 and 41.
- **642**: `[132, 132, 170]`, the Bowmen 46 timber 56 wealth.
- **702** (arm a): `[132 at 8100, 170]`; food +53, timber +41.
- **762** (arm b): `[170 at 100]`; food +51, timber +38.
- **802**: `build_masks` 4160. **842** (arm c): 4096, `[170 at 8100]`, no
  refund.
- **965**: the Bowmen out, `[]`, no re-queue.
- **982**: `[132 at 100]`; **992**: `[132, 170]`, wealth 66 → 10.
- **1002** (arm d): `[132 at 2100]`; timber +46, wealth +56.
- **1216**: the Hoplites out.

The two Hoplites' prices differ (51/38 against 53/41), so arm a's refund
says which entry went; a recomputed price would be a third Hoplite's.

**The capture must dump** `end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1`
and `misc:COMMANDMANAGER=1` over **`[605, 1466)`**: 861 blocks. The last
falsifier is 1216's, so the runway past it is **250 blocks**.

**The premise's killer, and its writers** (§3, point 5): `0/2007`'s
`BUILDQUEUE` and `build_masks`, and who=0's `bucket`s, on 702, 762, 842,
965 and 1002. The queue's writers in the window are the listed ones. **The
loops**: `unqueue`'s walk to `queued` (at most 3 here); `action_unqueue`'s
none (every `p` here is ≥ −1); `action_queue_up`'s `num` (2 on 620).

**What would falsify it, and where each could first fire.**
1. **The issues do not reach the pump.** Trace frames 620–1000: an `INFO
   17` with a refusal; or no `process_unqueue` on 701, 761, 841 and 1001
   with `o` 2007 and `p` 0, 0, −1, −1.
2. **The queue-up is not the reading's.** `BUILDQUEUE` on 622: anything
   but two 132 entries, each paid once; on 642 `[132, 132, 170]`.
3. **A cancel inside a run** (arm a). 702, `BUILDQUEUE` and the buckets.
   It splits three readings: `[132 at ~8100, 170]` with +53/+41 (the walk:
   the run's last goes, the reading); `[132 at 0, 170]` with +51/+38 (no
   walk: the head goes); the head kept with another refund (a recomputed
   price).
4. **A cancel across two types** (arm b). 762: `[170]` with +51/+38 (the
   head goes with its progress, the reading); `[132 at ~13900]` with
   +46/+56 (a walk that crosses types).
5. **The cancel carries no group.** The pool on 702, 762, 842 and 1002:
   no slot seated or re-stamped by a cancel (802's `@buildmask` seats the
   building group; a cancel's `process_unqueue` has no `process_group`).
6. **A single cancel on an infinite queue** (arm c). 842, `build_masks`
   and `BUILDQUEUE`: 4096 with `[170 at ~8100]` and nothing refunded (the
   reading); 4096 with `[]` and +46/+56 (the cancel removes too); 4160
   with `[]` (it removes and keeps the bit); 4160 and `[170]` (nothing).
7. **The bit is gone at the finish.** The Bowmen's finish (965 by this
   crate): `[]` and 4096 (the reading), or `[170 at 0]` paid again and
   4160 (the bit survived the cancel).
8. **−1 is the last slot** (arm d). 1002: `[132 at ~2100]` with +46
   timber and +56 wealth (the reading); `[170 at 0]` with +53/+41 (−1 read
   as slot 0); `[132, 170]` unchanged (−1 read as no slot).
9. **The trains.** A Bowmen squad out of the Barracks on the Bowmen's
   finish and a Hoplite squad on the last Hoplite's (1216); no Hoplite
   trained from the first run (both cancelled).

Falsifiers 3, 4, 6 and 8 test the claim's own unit, one entry on the
block its cancel is processed (711), and each splits the readings (789).

**Where it should part.** The floor is measured with `@unqueue` skipped
(`crate::golden`'s SEAM): the first value parting is **702**, `0/2007`'s
`queued` (ours 3, theirs 2) and who=0's food and timber, and the draw
stream should part where a squad is trained that this crate does not
train, or not trained where it does: the uncancelled Hoplite's finish
(~855 here, a birth the original never makes).

**Run 2026-09-26 as run292 (item 884)** (`docs/RUNS.md` has the tables).
One take, `cover=0`, the same game as run285 to 855. **No falsifier
fired, and every value the staging walk named is on its block**:
- **The issues** (1): `INFO 17` refusal 0 on all nine; the package grows
  15 bytes on each cancel, no group; `process_unqueue 2007 0 701`, `0
  761`, `-1 841`, `-1 1001`.
- **The queue-up** (2): `[132 at 100, 132]` paid 51/38 and 53/41 on 622;
  `[132, 132, 170]` on 642.
- **Arm a** (3, the first of three readings): on 702 `[132 at 8100,
  170]`, food 157 → 210 and timber 121 → 162, the run's second Hoplite's
  recorded 53/41.
- **Arm b** (4, the first of two): on 762 `[170 at 100]`, food +51 and
  timber +38, the head at 13900 gone.
- **The pool** (5): no slot parts on a cancel's block (the widening below).
- **Arm c** (6, the first of four): 4160 on 802; on 842 4096, `[170 at
  8100]`, nothing refunded.
- **The bit at the finish** (7): the Bowmen at 20200 on 963; `[]` and 4096
  on 965, no re-queue.
- **Arm d** (8, the first of three): on 1002 `[132 at 2100]`, timber 133 →
  179 and wealth 10 → 66.
- **The trains** (9): the last Hoplite at 23310 on 1215 and out on 1216;
  no Hoplite born on 856, where run285's is.

**Where this crate parted: `GOLDEN_WORD_CHAPTER_TWENTY_FIVE` = 855, open,
on the first walk**, with the four `@unqueue` lines skipped: the head
Hoplite the original cancelled on 761 finishes here and trains a squad
the original never makes. The widening, (605, 858), in `WIDENINGS`, with
three windows in the coverage driver (702, 842, 855) and
`process_unqueue` pinned there. **The first parting's field list**, 702:
`0/2007`'s `queued` (ours 3, theirs 2), who=0's food (157, 210) and timber
(121, 162); on 842 `build:infinite_queue` (1, 0); on 856 the Hoplites
`0/10`–`0/12`, held here alone. The block's standing rows are chapter
thirteen's births' `form` (611, 615; parked 646). The writers on 702:
`queued`'s six are counted above and only `unqueue` runs there; a bucket's
are the gather income, `pay_cost` and `unpay_cost`, and the income agrees
on every block to 701 — so all three rows are the cancel's. **The pool**:
622's `slot 1` `ox`/`oy` (−1 here, 0 there), `push_group`'s record `o`,
§33's standing row (parked 887; its reader is a layout's `o`, and nothing
is laid out); and `856 slot 0 held`, the squad this crate trains alone.

**The cancel entered: `GOLDEN_WORD_CHAPTER_TWENTY_FIVE` = 1466, closed**
(item 884, `docs/PRODUCTION.md` "The player's cancel"): `crate::golden`'s
`@unqueue` goes through `input::unqueue` → `Sim::action_unqueue`. The
stream agrees to run292's end. **The value diff, both sides**, on the
blocks the reading named (ours before the build in brackets):
- **702**: `[132 at 8100, 170]`; food 210, timber 162 (157, 121, `[132,
  132, 170]`).
- **762**: `[170 at 100]`; food 267, timber 204.
- **842**: 4096, `[170 at 8100]`; no refund (the bit 1 here before).
- **965**: `[]`, 4096; the Bowmen `0/10`–`0/12` born on both (before, the
  Hoplites on 856, held here alone).
- **1002**: `[132 at 2100]`; timber 179, wealth 66.
- **1216**: `[]`; the Hoplites `0/13`–`0/15` born on both.

The widening is **run292 whole**, (605, 1467), with nine windows in the
coverage driver (702, 762, 842, 855, 965, 982, 1002, 1216, 1463). **What
stands, 18 rows**, all `Unit::init@00612100`'s (parked 646), §33's shape:
the births' `form` (611, 615, 965, 1216) and the followers' tile-centred
`orders_x/y` (965, 1216). **The pool, 8 rows**: each slot's `ox`/`oy` on
its first seat (622, 965, 1216), `push_group`'s record `o` (parked 887);
and **982 slot 0's `speed`/`new_speed`, ours 0 and theirs 26** — a row the
build exposes, and not this item's to build. The 981 `@queueup`'s
building-group push runs `Groups::get_open_slot@006fa460`, which calls
vslot 8 (`Group::get_num_cap@007145c0`) on every slot whose stamp is not
newer than the best so far, and `get_num_cap` `normalize`s a group of
fewer than four, so the Bowmen squad (num 3, stamp 964) takes its
leader's 26. This crate's `open_slot` has no such side effect. It is the
pool lane's code. Its reader is `UnitData::get_speed@00608720`'s group
cap on a moving member, and the squad does not move in run292. Chapter
twenty-four's 901 press hid the same call: its squad had taken 25 on 897
from `Groups::process@006fa210`'s cursor.

**Mutations**, each restored from git and `touch`ed, on the built tree:

| mutation | the widening parts on |
| --- | --- |
| the bit's early return dropped | **842**: `queued` 0 against 1, timber 255/209, wealth 118/62 |
| a negative `p` read as slot 0 | **1002**: the Hoplite refunded in the Bowmen's place, food 288/235; the word falls to 1212 |
| `unqueue`'s walk dropped | **702**: the head removed, `job_counter` 100 against 8100, food 208/210 |

The unit test `the_player_s_cancel_reads_its_selector` fails under the
first two as well.

## 35. Chapter twenty-six — the research line: a technology through the player's command, pressed again, behind a busy Library, held, re-priced, cancelled and pressed again (item 883)

**Premise.** The player's queue-up of a technology, `CommandPackage::
process_queue_up@00948230` → `Group::action_queue_up@006fdbb0(t, num)`,
takes **the research arm** (`docs/PRODUCTION.md`, "The player's
research"): `LeaderData::researching@006db510(t, −1, 0, 0)` refuses the
command whole when the tech is queued at any of the player's buildings;
otherwise two passes over the members sorted by `queued` — the first
offering the job only to a member with an empty queue, the second to any
— and **the first `Build::queue_up@00620f40(t, 1)` that answers 0 ends
it**, so a research lands once whatever `num` says. `queue_up` then asks
`BuildData::can_make@0062db10`, whose tech arm refuses a tech already held
(`has_tech`, the leader's `tech` bit) or researching. The finish is
`Leader::gain_tech@006dcb60`; a Science epoch's gain re-prices the first
Library's other tech entries in place (`Build::refund_cost@00620490`), and
a cancel refunds the recorded pairs (§34). 877 read the arm and did not
build it; each clause is a claim, and the cast puts one press on each.

**Under the emulator first** (a scratch script on `tools/emu/callfn.py`'s
machine: a `Group` on `Group::vftable`, `Build`s on `Build::vftable` with
their queues in who=0's registry, every slot 2000..mark an object, a
`TypeData` per index on `TechType::vftable`, the leader's inline bits, a
console on another player; `Build::queue_up` stubbed, each call recorded
with its building, type and flag and answered by a per-building policy).
The listing was read first: `researching`'s arguments at `6fde26`..
`6fde39`, the pass counter in `num`'s slot at `6fe08a`, the empty-queue
test at `6fe135`, the return on `queue_up == 0` at `6fe16b`.

| the group, the job | `queue_up` calls (building: answer) | laid |
| --- | --- | --- |
| an idle Library, a tech, `num` 2 (and `num` 0) | L: 0 | one entry |
| [an idle Barracks refusing, a busy Library] | B: 1, B: 1, L: 0 | behind the busy one |
| [a busy Library, an idle one] | L2: 0 | on the idle one |
| both busy, 2 and 1 queued | L2: 0 | the least queued |
| an idle member refusing, a busy one | L: 1, L: 1, L2: 0 | behind the busy one |
| every member refusing | B: 1, B: 1, L: 1 | nothing |
| the tech in the Library's queue, or another building's outside the group | none | nothing |
| the tech held and queued nowhere | L: 0 (the stub) | `queue_up`'s to refuse |
| a member alive and not finished (`field_0x8 & 4` clear) | none | nothing |
| a unit type, its bit clear, two Barracks, `num` 2 | B: 0 | one entry |
| the same, its bit set | B, B2, B, B2 | `num` × members |

`researching`: 1 for a tech queued at an active building, 0 at an inactive
one; for a unit type, an entry of the same type (a Hoplites entry answers
for Hoplites). The arm writes only the group's `+0x28` (`action_begin`).
**What the emulator could not reach**: `queue_up`'s gates and price (its
body reaches `get_cost`), the forward to the first Library, `can_make`'s
held and researching refusals (read from the decompile), the console's
`add_feedback`, and the finish.

**The writers and readers**, by address and offset in every spelling
(parked 869, 823). `researching` has 35 call sites in 21 functions: the
simulation's `can_make`, `action_queue_up`, `get_cost` (the finals' ramp)
and `SpellTypeData::is_castable`; the AI's `Leader::` `compute_research_
score`, `compute_unit_upgrades_score`, `create_units`, `diplomacy`,
`plan_strategy`, `produce_tech`, `tech_avail` and `upgrade_units`; the
interface's `BuildOut::make_options` (8), `GroupOut::issue_queue_up`,
`IFaceSelected::draw_build_text`, `Options::describe` and
`do_cycle_research`, `StatWin`'s two; the scenario's `researching_tech_at`;
and its own wrapper. The bit (`LeaderData::tech`, `docs/TECH.md`)
is written by `Leader::gain_tech` (its own bit, and a unit's predecessors),
`Leader::lose_tech`, `Leader::init` and `ConsoleWin::run_cmd` — in this
window, `gain_tech` alone.

**The cast** is chapter thirteen's to 619 (run292 is this game there):
`!ai off`, who=0's Barracks `0/2007`, and who=0's own Library `0/2005`, in
its city from the start (`get_first_library`, one library city). **Lines**:
`620 @queueup 0 572 2 2005` (The Art of War); `640 @queueup 0 572 1 2005`;
`650 @queueup 0 551 1 2005` (Written Word); `850 @queueup 0 572 1 2005`;
`860 @queueup 0 558 1 2005` (Barter); `870 @queueup 0 132 1 2007`; `1040
@unqueue 0 0 2005`; `1060 @queueup 0 558 1 2005`. **Timber is the budget**
(240 at 619, about 0.07 a block): The Art of War is food-only so that the
second press (642) and the held press (852) are affordable by 16 and 35
food, and neither refusal can be the price's.

**The staging, walked by this crate through the command's entry**
(`input::group_queue_up` → `Sim::action_queue_research`, on run292's
start; run297 was not used). Every value is this crate's:
- **622**: `0/2005` `[572 at 100]`, 120 food; food 134.
- **642**: unchanged. **652**: `[572, 551 at 0]`, 120 timber 50 wealth.
- **822**: The Art of War out: `epochs_get()` 1, Military 1,
  `discovered_get()` 2 (the Bark and the Trireme, `gain_tech`'s units
  cascade); `[551 at 0]`; no unit trained.
- **852**: unchanged. **862**: `[551 at 4000, 558 at 0]`, 60 food 60
  timber. **872**: `0/2007` `[132 at 100]`.
- **1023**: Written Word out: epochs 2, Science 1, discovered 3
  (Boadicea); `[558 at 0]` recorded 54/54, food and timber +6.
- **1042**: `[]`, food and timber +54. **1062**: `[558 at 100]`, 54/54.
- **1106**: the Hoplites out. **1242**: Barter out: epochs 3, Commerce 1.

**The capture must dump** `end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1`
and `misc:COMMANDMANAGER=1` over **`[605, 1492)`**: 887 blocks, **250 of
runway** past 1242. **`LEADERS=2`, not the bitmask's level**: a kept dump
(run292) prints `tech`, `tech_at_start` and `obs_flags` only at 9, as
`bit_values` (each byte's decimal, concatenated), and 9 is ~350 KB a
leader, four a block — about a gigabyte over this window. No key prints
`researching`; it is a function. Level 2 prints `ages_get()`,
`epochs_get()`, `discovered_get()` and `epoch_get(scan)`, the counters
`gain_tech`'s step 2 writes beside the bit, and the held press (852) reads
the bit itself.

**The premise's killer, and its writers** (§3, point 5): `0/2005`'s
`BUILDQUEUE` and who=0's `bucket`s and tech counters on 622, 642, 652,
822, 852, 1023, 1042 and 1062. The queue's writers in the window are
`queue_up`, `unqueue` (the finishes' and the cancel's) and `refund_cost`
(the re-price); nothing closes, captures or defeats. **The loops**: the
passes, two; the members, one; `researching`'s buildings to who=0's
`objects` mark and each queue to its `queued`.

**What would falsify it, and where each could first fire.**
1. **The issues do not reach the pump.** An `INFO 17` with a refusal; or
   no `process_queue_up` with type 572 num 2 on 621, 572 1 on 641, 551 on
   651, 572 on 851, 558 on 861, 132 on 871, 558 on 1061, and no
   `process_unqueue 2005 0` on 1041.
2. **Arm a, one entry** (622): `[572 at 100]` paid 120 food once (the
   reading); two entries and 240 food (`num` read, as the unit arm does);
   none.
3. **Arm b, the gate** (642): one entry, food unspent (the reading); a
   second 572 paid (no `researching`).
4. **Arm c, the second pass** (652): `[572, 551 at 0]` paid 120/50 (the
   reading); unchanged (a busy member never offered).
5. **The finish** (822): the entry gone, epochs 1 and Military 1, no unit
   born (the reading); a trained unit or no count (a research read as a
   train job).
6. **Arm d, held** (852): unchanged, food unspent (the reading); a second
   572 paid (`has_tech` not asked).
7. **Arm e, the re-price** (1023): `[558 at 0]` recorded 54/54 and +6/+6
   (the reading); 60/60 and nothing refunded.
8. **The cascade's count** (822, 1023): `discovered_get()` 2 then 3 (this
   crate); 2 then 2 (Boadicea, flags `lmh`, a hero `gain_tech`'s units
   cascade skips); anything else.
9. **Arm f, the cancel** (1042): `[]` and +54/+54 (the re-priced record,
   the reading); +60/+60 (the price at queue time); nothing removed.
10. **Arm g, pressed again** (1062): `[558 at 100]` paid 54/54 (the gate
    cleared and `calc_science_discount`, the reading); 60/60 (no
    discount); nothing (`researching` left set by the cancel).
11. **The unit beside, and the last finish**: the Hoplites out of the
    Barracks on 1106; Barter out on 1242, epochs 3 and Commerce 1.

Falsifiers 2, 3, 4, 6, 7, 9 and 10 test the claim's own unit, one entry on
the block its command or finish is processed (711), and each splits the
readings (789).

**Where it should part.** The floor is measured with a technology's
`@queueup` skipped (`crate::golden`'s SEAM): the first value parting is
**622**, `0/2005`'s `queued` (ours 0, theirs 1) and who=0's food (254
against 134). No draw is known to follow a research, so the draw stream
may not part at all: the Hoplites are queued on both sides.

**Run 2026-09-26 as run296 (item 883)** (`docs/RUNS.md` has the tables).
One take, `cover=0`, the same game as run292 to 964. **No falsifier
fired, and every value the staging walk named is on its block**: the
issues processed on 621, 641, 651, 851, 861, 871, 1041 and 1061 with
refusal 0 (1); `[572 at 100]` and 120 food once (2); nothing on the second
press (3) and on the held one (6), with 136 and 155 food unspent; Written
Word behind the busy head (4); The Art of War out on 822 with epochs 1,
Military 1 and no unit (5); Barter re-priced to 54/54 with +6/+6 on 1023
(7); `discovered_get()` 2 then 3 — Boadicea is counted (8, the first of
its readings); +54/+54 on the cancel (9); Barter again at 54/54 (10); the
Hoplites out on 1106 and Barter's finish on 1242 (11).

**Where this crate parted: `GOLDEN_WORD_CHAPTER_TWENTY_SIX` = 1492,
closed, on the first walk**, with a technology's `@queueup` skipped: a
research spends no draw, and the Hoplites are queued on both sides. The
floor is the widening's, (605, 1493) in `WIDENINGS`, with ten windows in
the coverage driver, and `ages_get()`, `epochs_get()` and
`discovered_get()` read by the leader diff since this item (they left the
coverage pin). **The first parting's field list**, 622: `0/2005`'s
`queued` (ours 0, theirs 1) and who=0's food (254, 134); then 652's timber
and wealth; 822's `epochs`, `epoch[0]` and `discovered`; 1023's
`epoch[3]`; 1024's `0/0` `mylos` (4, 6); 1242's `epoch[2]`; 1243's
`resource_cap` (1120, 1600). The writers on 622: `queued`'s are
`queue_up` and `unqueue`, and only the command's `queue_up` runs; a
bucket's are the gather income, `pay_cost` and `unpay_cost`, and the
income agrees on every block to 621 — so both rows are the research's.
The pool: the Library's building group `[2005]` in slot 1 from 622 there,
not seated here, so the Barracks' group takes slot 1 on 872 here and slot
0 there; and the first seats' record `o` (parked 887). The standing rows
are `Unit::init`'s (parked 646).

**The research entered: `GOLDEN_WORD_CHAPTER_TWENTY_SIX` = 1492, closed**
(item 883, `docs/PRODUCTION.md` "The player's research"): `crate::golden`'s
technology `@queueup` goes through `input::group_queue_up` →
`Sim::action_queue_research`. **The value diff, both sides**, on the
blocks the reading named (ours before the build in brackets): **622**
`[572 at 100]`, food 134 (`[]`, 254); **652** `[572, 551 at 0]`, timber
123 and wealth 64 (243, 114); **822** epochs 1, `epoch[0]` 1, discovered 2
(0, 0, 0); **852** unchanged, food 155; **1023** `[558 at 0]` 54/54, food
65, `epoch[3]` 1, discovered 3; **1042** `[]`, food 121 and timber 111;
**1062** `[558 at 100]` 54/54; **1242** epochs 3, `epoch[2]` 1 (0);
**1243** `resource_cap` 1600 (1120). **What stands, 12 rows**: eleven
`Unit::init@00612100`'s (parked 646; 611, 615 and the Hoplites on 1106),
and **1023 `0/0` `mylos`**, ours 6 and theirs 4 — Written Word's
`SCIENCE_LOS`, computed on read here and cached by `Unit::update_los@
0060e4d0` there, refreshed a block after the gain: `docs/VISION.md`'s open
question on the cached `mylos` (run10's Scout, the same shape). The first
walk had it on 1024 (4 against 6); it is named, not built. **The pool**:
the four first seats' record `o` (parked 887).

**Mutations**, each restored from git and `touch`ed, on the built tree:

| mutation | fails |
| --- | --- |
| `researching`'s gate dropped (both the arm's and `can_make`'s) | the unit test; the widening on **642**, a second entry and food 16 against 136 |
| the second pass dropped | the unit test; the widening on **652**, timber 243 against 123 |
| `can_make`'s held refusal dropped | the unit test; the widening on **852**, food 35 against 155; the word falls to 1105 |
| `num` read, the research laid `num` times | **nothing**: the second lay meets `researching` |

The last is a falsifier the staging could not split: 2's "two entries"
reading is foreclosed by the gate for one member, so `num` is backed by
the emulator's `num 0` row alone.

## 36. Chapter twenty-seven — the upgrade line: a unit upgrade through the player's command, pressed again, the old type queued behind it, the finish that converts both, and the two presses after (item 901)

**Premise.** A unit type whose availability bit is clear is a research:
`Group::action_queue_up@006fdbb0` takes its research arm behind
`LeaderData::researching@006db510` (§35), `Build::queue_up@00620f40`
prices it through `TypeData::get_cost@00664090`'s research arm (the
premiums, the refit over the line, the military discount; `docs/AI.md`
§56) and lays it at the building its `WHERE` names. Its finish,
`Build::finished@00628490` → `Leader::gain_tech@006dcb60(t, x, y, 1, 1)`,
sets the bit and runs the **unit arm** (`docs/TECH.md` step 7): every
standing unit of the line is `set_type`d; **every queued entry of the line
is re-targeted in place** — `BuildQueue::set_queue@006309f0(i, t, NULL,
1)`, the type written, the progress and the recorded price kept, nothing
paid or refunded; and the predecessors get the `tech` and `obs_flags`
bits, so the old type can no longer be queued and the new one is a train
job. 883 built the command's arm and left it unstaged; the queue loop is
read here and **not built** in this crate (`Sim::upgrade_units_to`'s
`SEAM:` names only the carrier arm beside it). Each clause is a claim, and
the cast puts one press or one finish on each.

**Under the emulator first** (a scratch script on `tools/emu/callfn.py`'s
machine, out of git): `gain_tech`'s unit arm, `6dd999`..`6e05d9`, entered
with a synthesized frame (`t`, `t × 4`, `upgrade_units`, the building
counter at 2000) on a synthesized player — units on `0xc0aec0`'s table,
buildings from 2000 with their `BuildData` and queues, unit type records
carrying `from`, `jump` and `uber_size`. The three loops run as shipped,
with the real `BuildQueueData::get_queue@00630670` and `set_queue`;
`get_graft` (identity: the Slinger line has no graft for the Nubians),
`Objects::operator[]`, `Leader::track_queued@006e0f30`, `BitMask::set`,
`total_damage` and every virtual slot are stubbed and recorded. The
listing was read first; the building counter is `0x44(%ebp)`, written 2000
at `6dd62f` on every path that reaches the arm.

| the player, the gain | what is written |
| --- | --- |
| (a) Phalanx; two Hoplites (a captain, a member), a Bowmen, a Phalanx, an inactive Hoplites | `set_type(Phalanx, 0)` on both active Hoplites; `tech` and `obs` bits on Hoplites |
| (b) Phalanx; Barracks `[Phalanx at 10000, Hoplites at 0 51/38]`, a second `[Hoplites at 3000 51/38, Bowmen]`, a Library `[tech]` | both Hoplites entries → Phalanx, **counter and pairs kept**; `track_queued(Hoplites, −1)`, `(Phalanx, +1)` each; the research entry, the Bowmen and the tech left |
| (c) Phalanx; the same upgrade queued at a second Barracks, and Pikemen behind it | nothing re-targeted: an entry of `t` itself, and one above it, are left |
| (d) the building 2000 inactive, holding Hoplites | its entry left; 2001's re-targeted |
| (e) `upgrade_units` 0 | no conversion, no re-target; the bits |
| (f) Pikemen; Hoplites and Phalanx standing and queued | both converted and both re-targeted; **the Hoplites entry, a match by the `jump` chain, calls `track_queued(Pikemen, −1)`** — the walker, not the entry's type (`6ddcf4`, `push esi`, and the decompile's `TVar16` agree) |

**What the emulator could not reach**: the price and the research time
(`get_cost`, `train_time`), `finished`'s own path, `set_type`'s body (its
guys, `docs/TECH.md` "The conversion, landed"), the console's feedback,
and the object loop's carrier arm (`is(0x134)`, `is(0x15f)`).

**The writers and readers**, by offset and by address in every spelling
(823, 869). A queue entry's `type` (`QueueItem +0x4`) is written by
`set_queue` — from `queue_up` (the price, counter 0) and from `gain_tech`
(no price, counter kept) — and by `un_queue`'s shift. `num_queued`
(`LeaderData +0x5a22`, also spelt as the absolute `0xe3fdb2` plus the leader's stride) is written by
`queue_up`, `unqueue`, `clean_queue`, `Wall::activate`, `close`,
`increment_stats` and `decrement_stats`, `Unit::process`,
`Unit::action_unqueue`, `Group::action_spell`, the AI's
`create_buildings`, `Leader::init`, and `track_queued`, whose two callers
are `Unit::close` and `gain_tech`; in this window, `queue_up`, the
finishes' `unqueue` and `gain_tech`. No key prints it at `LEADERS=2`;
the train price at 1012 reads it (the ramp's count). `obs_flags`
(`+0x6cf4`): `gain_tech`, `reset_obs_flags`, `Leader::init`, `close`,
`ConsoleWin::run_cmd` and `Options::exec`; no key prints it, and the
press at 1000 reads it. **`BuildData::can_make@0062db10` asks
`researching` for a technology only**: a unit research is refused there
only by `type_avail`, so the second press (642) is the command's gate
alone — unlike §35's, which `can_make` also refused.

**The cast** is chapter thirteen's to 619 with a Slinger squad beside it
(`616 add slinger who=0 12,86`, `0/10`), and who=0 (Nubian) staged into
the Classical age and The Art of War **by two cheats before the window**
— `600 age who=0 2` and `602 military who=0 1`, each a whole `gain_tech`
(`docs/INPUT.md` §11.11); the line is Slingers (82) → Javelineers (83),
priced in food and timber, which the Classical age leaves at 254 and 240.
The Hoplites `0/7` are a line the gain does not touch. **Lines**: `620
@queueup 0 83 2 2007`; `640 @queueup 0 83 1 2007`; `650 @queueup 0 82 1
2007`; `1000 @queueup 0 82 1 2007`; `1010 @queueup 0 83 1 2007`.

**The staging, walked by this crate through the command's entry**
(`input::group_queue_up`, on run296's start, which is this game to 599;
the queue loop by a prototype for the values it writes; run301 was not
used). **The staging is walked before any word** (828): 605 reads
ages 2, epochs 1, `epoch[0]` 1, discovered 8, knowledge and metal 100.
Then: **617** `0/10`; **622** `[83 at 100]` 80 food 80 timber, once, food
174 timber 161; **642** unchanged; **652** `[83, 82 at 0]`, 46/46; **922**
the gain, discovered 9, `0/10`..`0/12` Javelineers, `[83 at 100]` with
46/46 kept; **1002** unchanged; **1012** `[83, 83 at 0]` at 46/46;
**1111** a Javelineers squad out; **1307** the second.

**The gates between each reading and its block** (903): at 622 "`num`
read" needs a second 80/80, and food 174 and timber 161 pay it; at 642
"no gate" needs the same, 176 and 162; at 1002 "obsolete not asked" needs
46/46 and 162/140 pay it; at 1012 the research reading's 80/80 is paid by
163/141. **The lineage arm of `researching` cannot be split here**: a type
that `is` Javelineers with its bit clear is Elite Javelineers, which
`type_avail` refuses at Classical whatever the gate does.

**The capture must dump** `end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1`
and `misc:COMMANDMANAGER=1` over **`[605, 1560)`**: 955 blocks, **253 of
runway** past 1307. `BUILDS=7` prints every slot's `type`, `job_counter`,
`cost[]` and `good[]`; `LEADERS=2` the counters and the buckets; the bits
are printed only at 9, and the presses at 1000 and 1010 read them.

**The premise's killer, and its writers** (§3, point 5): `0/2007`'s
`BUILDQUEUE` and who=0's `bucket`s on 622, 642, 652, 922, 1002 and 1012,
and `0/10`..`0/12`'s `type` on 922. The queue's writers in the window are
`queue_up`, the finishes' `unqueue` and `gain_tech`'s `set_queue`; the
units' type, `set_type` alone. **The loops**: the passes, two; the
members, one; `gain_tech`'s objects to who=0's unit mark, its buildings
2000 to the building mark, each queue to its `queued`.

**What would falsify it, and where each could first fire.**
0. **The staging** (605): ages 2, epochs 1, `epoch[0]` 1, discovered 8,
   knowledge and metal 100 (this crate); anything else is the cheats', and
   the chapter reads its arms from there.
1. **The issues do not reach the pump.** An `INFO 17` with a refusal; or
   no `process_queue_up` with type 83 num 2 on 621, 83 on 641, 82 on 651,
   82 on 1001 and 83 on 1011.
2. **Arm a** (622): `[83 at 100]` at `0/2007`, 80/80 once (the reading);
   two entries and 160/160 (`num` read); the entry at the Library `0/2005`
   (a research forwarded as a technology is); none.
3. **Arm b, the gate** (642): unchanged (the reading); a second 83 at
   80/80 (no `researching` in the command).
4. **Arm c** (652): `[83, 82 at 0]` and 46/46 (the reading); refused (a
   train job of the line refused while its upgrade researches).
5. **Arm d, the finish** (922): the research entry gone; `0/10`..`0/12`
   type 83; **the Slingers entry `[83]` with its `job_counter` and its
   46/46 kept, buckets untouched** (the reading); `[82]` left (no queue
   loop, this crate before its build); the entry gone and +46/+46 (a
   refund); `[83]` at a new price (re-queued).
6. **The conversion's reach** (922): `0/7`..`0/9` still Hoplites (the
   reading); converted (a match by anything but the line).
7. **Arm e** (1002): unchanged, 46/46 unspent (the reading, `obs_flags`
   asked); `[83, 82]` paid 46/46 (the old type still trainable).
8. **Arm f** (1012): a second `[83]` at the train price 46/46 (the
   reading); 80/80 (a research again); nothing (the research arm's gate on
   a type now held).
9. **The births**: a Javelineers squad on 1111 (the reading) or a Slinger
   squad on 1103 (5's second reading); the second Javelineers on 1307.

Falsifiers 2, 3, 5, 7 and 8 test the claim's own unit — one entry or one
unit on the block its command or finish is processed (711) — and each
splits the readings (789).

**Run 2026-09-26 as run300 (item 901)** (`docs/RUNS.md` has the tables).
One take, `cover=0`, the same game as run296 to 616. **No falsifier
fired, and every value the staging walk named is on its block**: 605's
staging (0); the issues on 621, 641, 651, 1001 and 1011 with refusal 0
(1); `[83 at 100]` and 80/80 once (2); nothing on the second press, 176
and 162 unspent (3); the Slingers behind at 46/46 (4); on 922 the research
out and **the Slingers entry `[83 at 0]` with its 46/46 kept and nothing
refunded** (5), `0/10`..`0/12` Javelineers and the Hoplites untouched
(6); nothing on the Slingers after the gain (7); a second Javelineers at
the train price 46/46 (8); the squads on 1111 and 1307 (9).

**Where it should part.** The floor is measured on this crate as it
stands, with no queue loop: **922**, `0/2007`'s `queue[1].type` (ours 82,
theirs 83 by the reading), then 1103's Slinger squad against 1111's
Javelineers — the draw stream's first parting, the births' `Guy::init`.

**Where this crate parted: `GOLDEN_WORD_CHAPTER_TWENTY_SEVEN` = 1102,
open, on the first walk**: the Slingers entry the original re-targeted
trains a Slinger squad here on 1102, the births' draws. The floor is the
widening's over (605, 1105) in `WIDENINGS`, with seven windows in the
coverage driver. **The first parting's field list**, 922: `0/2007`'s
`queue[0].type`, ours 82 and theirs 83 — the slot is 0 once the finish's
`unqueue` has removed the research, which both sides run; the only other
writer of the field on 922 is `gain_tech`'s `set_queue`, and only the
original runs it. Then 1103: `0/13`..`0/15` this crate's alone, and
`queued` 1 against 2. The births' `form` are `Unit::init`'s (parked
646). The pool: the first seat's record `o` on 622 (parked 887), and the
extra squad's seat on 1103.

**The queue loop entered: `GOLDEN_WORD_CHAPTER_TWENTY_SEVEN` = 1560,
closed** (item 901, `docs/TECH.md` "The queue loop",
`Sim::retarget_queued_to`). **The value diff, both sides**, on the blocks
the reading named (ours before the build in brackets): **922**
`0/2007`'s `[83 at 0]` 46/46 (`[82 at 0]` 46/46), food 155 and timber 135
both; **1103** `queued` 2 (1) and no squad (`0/13`..`0/15` Slingers);
**1111** `0/13`..`0/15` type 83, `myhits` 95 (none); **1307**
`0/16`..`0/18` type 83. Every other queue, bucket, counter and unit
type in run300 agrees on every block. **What stands, 21 rows**,
all `Unit::init@00612100`'s (parked 646): the births' `form` (611, 615,
617, 1111, 1307) and the followers' `orders_x/y`. **The pool**: the first
seats' record `o` on 622 and 1111 (parked 887).

**Mutations**, each restored from git and `touch`ed, on the built tree:

| mutation | fails |
| --- | --- |
| the queue loop dropped | the unit test; the widening on **922**, `queue[0].type` 82 against 83; the word falls to 1102 |
| a refund on the re-target | the unit test; the widening on **922**, food 201 against 155 and timber 181 against 135 |
| the progress reset | the unit test alone: the entry sat at 0 behind the research |
| the `jump` match decrementing the entry's type | the unit test alone: Slingers → Javelineers is a `from` match |

The last two are the emulator's rows b and f, and no capture on disk
splits them.

## 37. Chapter twenty-eight — two buildings under one command: the queue-up's sort and passes, the infinite toggle on two, and the building group of two in the pool (item 888)

**Premise.** Every issuer chapter before this one selected one building.
With two, three mechanisms read one way and not yet staged:
`Group::action_queue_up@006fdbb0` **sorts the members once by `queued`,
least first, and then lays `num` passes, one entry a member a pass**
(`docs/PRODUCTION.md` "The command on a selection of buildings");
`Group::action_buildmask@006fc9a0` **sets a member's bit only while it
and every admitted member before it lacked it** (§32); and
`CommandPackage::process_group@0094a0c0`'s push **seats the two as one
record, `buildings 1`, in the command's order, and a command whose group
equals `last_group`'s record in the same order seats nothing**
(`Groups::push_group@0070f9e0`, `Group::equals_group@00708000`). The
last is the one this crate does not carry: `Sim::push_command_buildings`
seats a single building only (its `SEAM:`, `docs/GROUPS.md` §31.4). Each
clause is a claim, and the cast puts one press on each.

**Under the emulator first** (a scratch script on `tools/emu/callfn.py`'s
machine, out of git): real `Build::vftable` and `Group::vftable` objects
in who=0's registry, the type list and the leader's bits synthesized,
`Build::queue_up@00620f40`, `LeaderData::researching@006db510` and the
type record's `is` stubbed and recorded; `Group::add@00714350`,
`push_group`, `equals_group`, `Groups::get_open_slot@006fa460` and
`Groups::copy_group@006fa690` run as shipped on a synthesized pool. The
listing was read first: the sort is `6fdc6b`..`6fdd86` (the pivot's
`+0x82` against each later member's, `jbe` at `6fdd4b`: a swap only on
strictly less, unsigned), the pivot tested alive (vslot `0xc`, `+8 & 1`)
and finished (vslot `0x4c`, `+8 & 4`) before its inner loop; the passes
are `6fdf20`..`6fe06f`, `[ebp+0xc]` counted down, each member tested
alive, finished and vslot `0x20`, a missile silo asking `can_carry`,
and the call at `6fe056` whose answer is not read.

| the group, the press | what `queue_up` is called on, in order |
| --- | --- |
| (a) two empty, `num` 1 | 2007, 2008 |
| (a) two empty, `num` 3 | 2007, 2008, 2007, 2008, 2007, 2008 |
| (b) `[2007 q2, 2008 q0]`, `num` 1 / 3 | 2008, 2007 / 2008, 2007 × 3 |
| (b) `[2007 q1, 2008 q1]` | 2007, 2008: no swap on equal |
| (c) the first not finished, `num` 1 | 2008 alone |
| (c) `[2007 q2, 2008 not finished q0, 2009 q1]` | 2009, 2007: the unfinished member is swapped to the head, and never called |
| `num` 0, `num` −1 | nothing |
| a call refused (`queue_up` answers 1) | the passes go on |
| the research arm (the bit clear), `[q2, q1]` / `[q0, q0]` | once: 2008 / 2007 |

`action_queue_up` itself writes nothing on a member (every building write
is `queue_up`'s) and the group's `+0x28` (`Group::action_begin`).
`action_buildmask(0x40)` on two admitted members: `[on, off]` → `[off,
off]`, `[off, on]` → `[on, off]`, `[off, off]` → `[on, on]`, `[on, on]` →
`[off, off]`. The push of `[2007, 2008]`: one record, `buildings 1`,
`num` 2, the list as added, `stamp` the frame, `speed` 0, and **nothing
written on either building** (the member walk writes a unit's `+0x80`
alone); the same group again: nothing; `[2008, 2007]` after it: a new
seat (`equals_group` compares the lists in order); a duplicate in the
command is dropped by `GroupData::member@0070f8f0`. **The `SEAM:` against
the listing's loops** (910): `process_group`'s loop over the command's
`num` objects adds every live one, whatever it is, and pushes once after
the loop (`0x94a6cf`): the crate's single-building guard is the only
difference. **What the emulator could not reach**: `queue_up`'s price and
room (the crate's, diff-backed by run285 and run296), `process_group`'s
own body (its `SyncLogger` and log), a finished member (`+8 & 4`) no
cheat can clear, and a silo.

**The writers and readers, counted** (823, 869). `queued` (`BuildData
+0x82`): written by `Build::queue_up` (+1), `Build::unqueue@006207c0` and
`Build::clean_queue@00620b60` (−1), `Build::new_library` (a copy) and the
constructor; in this window, the presses' `queue_up` and the finishes'
`unqueue`. `last_group` (`Groups +0x1c`, `0xe85f2c`): written by
`push_group` alone (and the constructor and the save), read by it and by
`get_open_slot`. A record's kind (`+0x49`): `Group::add` and
`copy_group`. `build_masks` (`WallData +0x60`): §33's list.

**The cast** is chapter thirteen's with a second Barracks: `606 add
barracks who=0 14,74` (`0/2007`), **`608 add barracks who=0 22,74`**
(`0/2008`, eight tiles east; `Objects::init_build` asks no site), `610
add chariot`, `614 add hoplite`. **Lines**: `620 @queueup 0 170 1 2007`
(Bowmen at 2007 alone); `640 @queueup 0 132 3 2007 2008` (Hoplites,
`num` 3, the busier listed first); `700 @buildmask 0 64 2008`; `720
@buildmask 0 64 2008 2007`; `740 @queueup 0 132 1 2007 2008` and `760`
the same, both refused on price. `CommandPackage::add_group@0094bb60`'s
reuse test is an ordered compare of the selection, so 740 sends a new
group and 760 the three-byte reuse.

**The staging, walked by this crate through the commands' entries**
(`input::group_queue_up`, `input::group_buildmask`, on run285's start,
which is this game to 607; the two-building seat by a prototype for the
pool's values; run305 was not used). Every value is this crate's:
- **610**: `0/2008` at (4224, 14208), `build_masks` 4096.
- **622**: `0/2007` `[170 at 100]`, 41 timber 51 wealth; food 254, timber
  200, wealth 62.
- **642**: `0/2008` `[132 at 100 53/41, 132 60/50]`; `0/2007` `[170, 132
  56/45, 132 65/56]`; food 22, timber 9. The fifth call's 70/62 and the
  sixth are refused.
- **702**: `0/2008` 4160. **722**: 4096 and 4096.
- **742**, **762**: the queues unchanged; food 31 and 33, timber 16 and 17
  against the Hoplites' ~70/62.
- the pool (who=0): **622** slot 1 `[2007]`, stamp 621; **642** slot 0
  `[2007, 2008]`, 641; **702** slot 1 `[2008]`, 701; **722** slot 0
  `[2008, 2007]`, 721; **742** slot 1 `[2007, 2008]`, 741; **762** the
  same.
- the births: **825** the Bowmen `0/10`..`0/12` out of 2007, slot 0 (the
  building group `[2008, 2007]`'s, open and not `last_group`'s); **876**
  the Hoplites `0/13`..`0/15` out of 2008, slot 1; **1067** `0/16`.. slot
  2; **1126** `0/19`.. slot 3; **1324** `0/22`.. slot 4.

**The gates between each reading and its block** (903). At 642 every
alternative below is price alone: the four entries cost 234 food and 182
timber in any order, and the room is 20 a Barracks. **The fourth entry
has 7 timber over** (63 against 56): a higher price there makes it three
entries, and the readings still split on the first and third (both
2008's in the reading). At 722 both members are admitted (`can_infinite`:
2007 holds the Bowmen, 2008 two Hoplites, all train jobs). At 742 and 762
the push is `process_group`'s, before `action_queue_up`, so the price's
refusal does not gate it.

**The capture must dump** `end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1`
and `misc:COMMANDMANAGER=1` over **`[605, 1580)`**: 975 blocks, **256 of
runway** past 1324. `GROUPS=1` prints `GROUPDATA`, the pool; `BUILDS=7`
every entry's `type`, `job_counter`, `cost[]` and `good[]` and
`build_masks`; `LEADERS=2` the buckets.

**The premise's killer, and its writers** (§3, point 5): `0/2007`'s and
`0/2008`'s `BUILDQUEUE` and who=0's `bucket`s on 622 and 642, their
`build_masks` on 702 and 722, who=0's `GROUPDATA` slots 0 and 1 on 622,
642, 702, 722, 742 and 762, and the trained units' `group` on 825 and
876. The queue's writers in the window are the presses' `queue_up` and
the finishes' `unqueue`; the bit's, `action_buildmask` (no finish meets a
set bit in the reading); the pool's, `push_group` from `process_group`
and `come_out`. **The loops**: the sort to `num − 1`; the passes, 3; the
members, 2; `process_group`'s objects, 1 or 2.

**What would falsify it, and where each could first fire.**
0. **The staging** (610): `0/2008` a Barracks at (4224, 14208), 4096.
1. **The issues do not reach the pump.** An `INFO 17` with a refusal on
   620, 640, 700, 720, 740 or 760; or no `process_queue_up` 170 1 on 621,
   132 3 on 641, 132 1 on 741 and 761, no `process_buildmask` 64 on 701
   and 721, and no `process_group` with two objects on 641, 721 and 741
   and the reuse on 761.
2. **The single press** (622): `0/2007` `[170 at 100]` at 41/51, once.
3. **The sort and the passes** (642), each entry's building and recorded
   price — the claim's own unit (711): `0/2008` 53/41 and 60/50, `0/2007`
   56/45 and 65/56 (**the reading**: sorted, one a member a pass);
   `0/2007` 53/41 and 60/50, `0/2008` 56/45 and 65/56 (no sort); `0/2008`
   53/41, 56/45 and 60/50, `0/2007` 65/56 (sorted, `num` a member at a
   time); `0/2007` three and `0/2008` one (neither); three entries in all
   and food 87 (`num` over the group); every entry at one building (one
   member).
4. **The toggle on one** (702): `0/2008` 4160, `0/2007` 4096.
5. **The toggle on two** (722), `[on, off]`: 4096 and 4096 (**the
   reading**: the first member's clear clears the rest); `0/2008` 4096 and
   `0/2007` 4160 (each member toggled on its own); 4160 and 4160 (the
   command's `set` read).
6. **The building group of two** (642): who=0's slot 0 `buildings 1`,
   `num` 2, `[2007, 2008]`, stamp 641, and slot 1 `[2007]` from 621 (**the
   reading**); no two-member record (a single building seated, or none);
   `[2007]` alone (the first member only).
7. **The next seat** (702): slot 1 `[2008]`, stamp 701 (the reading); slot
   0 (the two-member record not seated, this crate before its build).
8. **The order kept** (722): slot 0 `[2008, 2007]`, stamp 721 (the
   reading); `[2007, 2008]` (the list sorted).
9. **Order is identity** (742): slot 1 `[2007, 2008]`, stamp 741, slot 0
   `[2008, 2007]` kept (the reading); no seat, slot 1 `[2008]` 701 (an
   unordered compare).
10. **An equal group seats nothing** (762): slot 1 stamp 741 (the
    reading); slot 0 `[2007, 2008]`, stamp 761 (every command seats).
11. **The births take the building groups' slots** (825, 876): `0/10`..
    `0/12` `group` 0 and `0/13`..`0/15` `group` 1 (the reading); 1 and 0
    (the two-member records not seated); then 2, 3 and 4 on 1067, 1126
    and 1324.

Falsifiers 3, 5, 6, 9 and 10 test the claim's own unit — an entry, a
member's bit, a slot's record — on the block its command is processed
(711), and each splits the readings (789).

**Where it should part.** The floor is measured on this crate as it
stands, with the two-building seat not built: **642**, who=0's slot 0
held there and empty here; then 702's `[2008]` in slot 0 here against
slot 1, 722's and 742's records, and 825's and 876's `group`, 1 and 0
here against 0 and 1. The queues and the bits should agree on every
block: `Sim::action_queue_up`'s sort and passes and
`Sim::action_buildmask`'s rule are built from the reading. The draw
stream need not part: a slot number spends no draw.

**Run 2026-09-26 as run304 (item 888)** (`docs/RUNS.md` has the tables).
One take, `cover=0`, the same game as run285 to 824. **No falsifier
fired, and every value the staging walk named is on its block**: the
issues on 621, 641, 701, 721, 741 and 761 with refusal 0, 761 the
selection's reuse (1); 622's single press (2); on 642 **`0/2008` 53/41
and 60/50, `0/2007` 56/45 and 65/56** — sorted once, one a member a pass
(3); 4160 on 702 (4) and **4096 and 4096 on 722** (5); the pool **slot 0
`[2007, 2008]` stamp 641** (6), `[2008]` in slot 1 on 702 (7), `[2008,
2007]` in slot 0 on 722 (8), `[2007, 2008]` in slot 1 on 742 (9), stamp
741 kept on 762 (10); and the births' `group` 0 to 4 (11).

**Where this crate parted: `GOLDEN_WORD_CHAPTER_TWENTY_EIGHT` = 1580,
closed, on the first walk** — a pool slot spends no draw. The widening,
run304 whole in `WIDENINGS`, with eight windows in the coverage driver.
**Every queue, bucket and `build_masks` agrees on every block**: the sort
and passes and the toggle's rule were built from the reading. **The
first parting's field list**, 642: who=0's slot 0 `held`, theirs `[2007,
2008]` and ours empty. Its writer on 642 is `process_group`'s
`push_group` alone (no birth, no other command), and only the original
seats a group of two. Then every seat after it one slot off (702's
`[2008]` in slot 0 here, 742's `num`) and, the units' half, **825's
`group`**, `0/10`..`0/12` 1 here against 0, and 876's `0/13`..`0/15` 0
against 1: `come_out`'s push takes the first open slot, and the slots
the two-member records hold there are empty here. **The writers of each
standing field, counted before booking** (889): `group` (`+0x80`) —
`push_group`'s member walk and `get_open_slot`'s clear, both sides run
both, and the slot they are handed is the pool's; `form` and the
followers' `orders_x/y` on every birth — `Unit::init@00612100` (parked
646), 30 of the 45 rows; the pool's `speed`/`new_speed` on 833 and 834 —
`normalize`'s leader speed on the squads' slots, one slot off; each
first seat's `ox`/`oy` — `push_group`'s record `o` (parked 887).

**The seat built: `GOLDEN_WORD_CHAPTER_TWENTY_EIGHT` stays 1580,
closed** (item 888, `docs/PRODUCTION.md` "The command on a selection of
buildings", `Sim::push_buildings_group`). **The value diff, both sides**
(ours before the build in brackets): **642** who=0's slot 0 `buildings
1`, `num` 2, `[2007, 2008]`, stamp 641 (empty); **702** slot 1 `[2008]`
701 (in slot 0, and slot 1 still `[2007]` 621); **722** slot 0 `[2008,
2007]` 721; **742** slot 1 `[2007, 2008]` 741 (`num` 1); **762** the
same, the stamp kept; **825** `0/10`..`0/12` `group` 0 (1); **876**
`0/13`..`0/15` `group` 1 (0), and the squads' `speed` 26 and 25 on their
slots from 833 and 834. **What stands: 39 rows**, all
`Unit::init@00612100`'s (parked 646) — `form` on every birth and the
followers' `orders_x/y` — and **10 pool rows**, each first seat's
`ox`/`oy` (parked 887).

**Mutations**, each restored from git and `touch`ed, on the built tree:

| mutation | fails |
| --- | --- |
| a single building seated (the old guard) | the unit test; the widening on **642**, slot 0 `held`, and on 825's `group` |
| the equality unordered | the unit test; the widening on **742**, slot 1 `[2008]` against `[2007, 2008]` |
| a building listed twice kept twice | the unit test alone: no capture lists one twice |
| `action_queue_up`'s sort dropped | `a_press_on_two_buildings_lays_one_entry_a_member_a_pass_from_the_least_queued`; the widening on **642**, `0/2008`'s `queue[1].cost` 65 against 60 |
| `num` laid a member at a time | the same unit test; the widening on **642**, `queued` 3 against 2 |
| the toggle read per member | `the_repeat_button_toggles_off_the_first_member`; the widening on **722**, `0/2007` `infinite_queue` 1 against 0 |

The sort and the passes had no unit test before this item: the first
run of those two mutations failed run304's widening alone, and the unit
test was written then.

## 38. Chapter twenty-nine — the repeat launch: a landed patrol relaunched under the repeat bit, and an aircraft trained at the Airbase (item 915)

**Premise.** Two Airbase arms are built from readings and no capture
measures them (parked 876 and 843):
- **`Object::do_launch@0064f3b0`'s repeat arm, the launch half.** A plane
  inside a base whose `build_masks & 0x80` is set, holding the unflagged
  patrol `Unit::land_plane@005e9950` kept (`docs/ORDERS.md` §40), is
  **launched** at its full tank, not killed: `has_repeat_air() || flags &
  4` (§41.2). Chapter twenty-three measured the kill; nothing has flown
  the launch.
- **An aircraft trained at an Airbase.** Parked 843 read it as leaving
  through the same EXIT (`come_out`'s tail, §31), and this crate trains it
  so. **The listing says it does not come out at all**:
  `Build::train@0062f9b0` tests the *trainer's* type `+0x1e4 & 0x200`
  (`CARRY_AIR`, `J`, carried by the Airbase and the Missile Silo alone)
  at `62fac0`, and on that arm calls no `Unit::come_out@00617c10`. With
  no gather point (`+0xcc` 0, `62fadf`) it asks only `is(HELICOPTER
  0x136, 0)` (`62ff59`), which a plane answers 0, and leaves: the
  plane stays inside, with no order, in the base's chain. With a gather
  point it gets `add_air_patrol_order(point, base, who, 1)` — the action
  bit — and waits for `do_launch` like any other. No command in any
  capture sets a gather point (its writers are `Group::action_gather_point`
  from the player's `issue_gather_point`, `action_city_gather`, and the
  scenario functions), so every building on disk has none.

Both are claims to check, not premises, and the cast puts one event on
each: chapter twenty-two's game with **no toggle** — every Airbase repeats
from `Build::init` — so its three kept patrols meet their full tanks under
the bit, and one aircraft trained at the Airbase **between** the first
relaunch and the second (879), so the relaunches are measured before and
after it joins the chain.

**Under the emulator first** (two scratch scripts on `tools/emu/callfn.py`'s
machine, out of git).
- **`do_launch`**, entered unchanged on a synthesized base on
  `Build::vftable` (vslots `0xf0` `WallData::has_repeat_air@00472410`,
  `0x18` and `0x20` run as shipped; the order accessors, `come_out`,
  `kill_current_order` and the `launching` array hooked and recorded):

  | the base, the chain | what it wrote |
  | --- | --- |
  | 4232, one patrol, flags 0, `mana_burn` 0, counter 15 | the patrol's `returning` (`+0x3c`) 0, `launching += 6`, `come_out(0)`, counter 0 |
  | 4104, the same | `kill_current_order(0)`, `launching -= 6`, counter 15 |
  | 4232, `mana_burn` 2 | nothing; counter 15 |
  | 4232, counter 14 | counter 15, no walk |
  | 4232, `[trained (no order), patrol]` | the trained plane passed over, the patrol launched |
  | 4232, `[patrol, trained]` | the patrol launched; **the walk ends** |
  | 4232, two full patrols | the first launched; **the second not walked** |
  | 4104, two full unflagged patrols | both killed |
  | 4104, one patrol flags 4 | launched |
  | 4232, empty | nothing: the counter stands |

  **The walk ends at a launch.** The chain's next link is read from the
  launched plane's own `inside_down` (`64f806`) *after* `come_out`, and
  `Object::remove_from_inside` stores −1 there. With the hook leaving the
  link, the second full patrol joins `launching` — this crate's loop,
  which walks its garrison list to the end.
- **`Build::train`**, entered unchanged on a synthesized trainer
  (`init_unit`, `go_inside`, `set_stance`, `come_out`,
  `add_air_patrol_order` and the gather list hooked; the unit and the
  type through stub vtables), `train(BIPLANE)` at 2007:

  | the trainer | what it called |
  | --- | --- |
  | `CARRY_AIR`, no gather point | `init_unit` at the base's point, `go_inside(2007)`, `options.rebuild = 1`, `is(0x136, 0)`, `unit_masks &= ~0x4000000`, `is(0x15f, 1)`: **no `come_out`** |
  | `CARRY_AIR`, one gather point | the same, and `add_air_patrol_order(x, y, 2007, 0, 1)` |
  | no `CARRY_AIR` | `come_out(0)` |

- **What the emulator could not reach**: `come_out`'s own body (it was
  hooked), so the EXIT a relaunch takes is §31's, diff-backed by run265;
  `Unit::init`'s birth values; the tank's refill inside; and a gather
  point's order walking out under `do_launch`.

**The booking's mutations** (918), on the tree as it stands, each restored
from git and `touch`ed:
- `do_launch`'s bit read as never set: `a_full_tank_launches_an_unflagged_patrol_only_under_the_bit`
  fails. **The launch half has a unit test.**
- the patrol's `returning` not cleared at the launch: **no test fails** —
  `land_plane` has already cleared it on every path a test takes.
- `come_out`'s EXIT at an Airbase removed: `a_strike_from_inside_waits_for_the_tank_and_leaves_on_the_exit`
  fails. **No test trains an aircraft at an Airbase.**

**The writers and readers, counted by offset** (823, 869):
- `launch_frames` (`ObjectData +0x41`, a `char`): `Object::Object@
  00646e80`'s zero and `do_launch`'s three stores; its only reader is
  `do_launch` (and `log_data`).
- `mana_burn` (`+0x96`): read by `do_launch` at `64f4b5`; the tank's
  writers are §31's.
- the gather point count (`BuildData +0xcc`): `Build::add_gather_point`
  and `Build::clear_gather`, reached only from the player's
  `issue_gather_point`, `Options::do_clear_gather`, `action_city_gather`
  and the scenario functions.
- the trainer's `CARRY_AIR`: the type record, loaded (`GJ` for Airbase
  and Missile Silo in `buildingrules.xml`).

**The cast** is chapter twenty-two's (§31), and **the lines** are
`chapter22.cmd`'s twelve and **`1540 @queueup 0 287 1 2007`**: one
**Biplane** at the Airbase. The Fighter (289) needs the Modern Age, which
`library who=0 6` does not give — this crate's `queue_up` answers
`CantTrain` for it and `Ok` for the Biplane, and so should the original
(its `PREQ0` is Industrial Age). The Biplane is a plane (`is(0x136)` 0),
and the trainer's type decides the arm, not the unit's.

**The staging, walked by this crate through the command's own entry**
(`input::group_queue_up` on run281's start, which is this game to 1440;
run309 was not used). Every value is this crate's:
- **1385**, **1489**, **1513**: `0/6`, `0/7`, `0/8` inside `0/2007`, each
  with its `AIRPATROLORDER`, flags 0 (§40's keep).
- **1542**: the Biplane queued at 2007, **85 metal and 85 oil** (134 → 49,
  100 → 15).
- **1585**: `0/6` out at (11424, 13845), `mana_burn` 0, its patrol kept,
  flags 0; the base's counter 0.
- **1746**: the Biplane born, `0/9`. **This crate brings it out** onto
  (11424, 13920) with no order, and it burns +1 a block from there.
- **1789**: `0/7` out; **1813**: `0/8` out.
- **1985**: `0/6`'s `mana_burn` 400, its tank empty.

**The readings, and the gates between each and its block** (903).
- The relaunches (1585, 1789, 1813): **launched** (the reading),
  **killed** (the bit no gate, parked 844's first reading), or **kept
  inside** (only the action bit launches). Nothing gates them but the
  tank: each block is its plane's first `mana_burn` 0, and the counter is
  saturated from 1400.
- The trained aircraft (1746): **inside with no order** (the listing),
  **out through the EXIT** (843, this crate), or **out on the ring** (the
  ordinary `come_out`). The gates before 1746: the price (85 and 85 of
  134 and 100, paid on 1542 in this crate), the Industrial Age (the
  Biplane's `PREQ0`), the population (one), and the job's pace — a
  different pace moves the block, never the arm.
- After 1813, the base's counter: under the reading the chain still holds
  the Biplane, so `launch_frames` runs 1..15 over 1814..1828 and stands at
  15; with the Biplane out, the base is empty and the counter stands at 0.

**The capture must dump** run281's line,
`end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5` and
`misc:COMMANDMANAGER=1`, over **`[605, 2070)`**: 1,465 blocks, **257 of
runway** past the last relaunch on 1813, with `0/6`'s empty tank on 1985
inside it. `AMMO=5` because the relaunched Bombers patrol a point with
who=1's town in reach. `BUILDS=7` prints `launch_frames`, `inside_down`,
the queue and `build_masks`; `UNITS=3` each plane's `inside_up`, order
stack and `mana_burn`; `LEADERS=2` the buckets; `GROUPS=1` the pool, where
the command's building group is seated.

**The premise's killer, and its writers** (§3, point 5): each plane's
stack and `inside_up` on its own relaunch block, and the Biplane's on
1746. The writers: `do_launch`'s launch and kill, `land_plane`'s keep,
`Build::train`'s arm, and `come_out`. **The loops**: `do_launch`'s chain
from `inside_down`, which **ends at the first launch** (above); the
queue-up's members, 1.

**What would falsify it, and where each could first fire.**
1. **The issue does not reach the pump.** Trace frame 1540: an `INFO 17`
   with a refusal, or no `process_queue_up` 287 1 on 1541.
2. **The press** (1542): `0/2007`'s `BUILDQUEUE` one Biplane, and who=0's
   metal and oil down 85 each. A refusal (no entry) kills the second half
   and the chapter is restaged.
3. **The first relaunch** (1585), `0/6`'s own stack and `inside_up`:
   `inside_up` −1 on the EXIT's point (11424, 13920 less a step), its
   `AIRPATROLORDER` kept with flags 0 and `returning` 0, `0/2007`'s
   `launch_frames` 0 (**the reading**); inside with no order (killed);
   inside with the patrol past 1585 (kept).
4. **The trained aircraft** (1746), the Biplane's own record:
   `inside_up 2007`, no order, `mana_burn` 0, on the base's chain behind
   `0/8` (**the reading**); `inside_up` −1 on (11424, 13920) (the EXIT,
   843); `inside_up` −1 on the ring due south of the base (the ordinary
   exit). And under the reading it stays so to 2069: no order, never
   launched.
5. **The second and third relaunches** (1789, 1813): `0/7` and `0/8`
   each out on its first `mana_burn` 0, with its patrol and flags 0 —
   with the Biplane behind each in the chain.
6. **The counter with the Biplane inside** (1814..1829): `0/2007`'s
   `launch_frames` 1, 2, … 15 and then 15 (the reading); 0 on every block
   (the base empty).
7. **The relaunched patrol flies its tank out** (1985): `0/6`'s
   `mana_burn` 400 and `returning` 1 (`check_fuel`, §31).

Falsifiers 3, 4 and 5 test each plane's own stack on its own block (711),
and 3 and 4 each split three readings (789). **Parked 844's neighbour —
a landed patrol with no action bit killed at the next full tank — is not
reached**: the bit is never cleared here; chapter twenty-three measured
that kill.

**Where it should part.** This crate trains the Biplane out through the
EXIT, so the first value parting is expected on **1746**: the Biplane's
`inside_up` (−1 here against 2007) and its point, then its `mana_burn`
(+1 a block here, 0 there), and from 1814 the base's `launch_frames` (0
here). The relaunches on 1585, 1789 and 1813 should agree: the launch
half is built. Whether the draw stream parts at 1746 depends on what a
plane on the map with no order spends; the walk will say.

**Run 2026-09-27 as run308 (item 915)** (`docs/RUNS.md` has the tables).
One take, `cover=0`, the same game as run265 on all 1,501 frames the two
share. **Falsifier 4 split its readings for the listing**: the Biplane
`0/9` is born on 1746 **inside `0/2007`**, `inside_up 8` (behind `0/8`),
with no order and `mana_burn` 0, at (11640, 13944), and so to 2069 —
parked 843's EXIT, which this crate takes, is killed. **No other
falsifier fired**: the press on 1541 (metal and oil −85, 1542); `0/6`
relaunched on 1585 onto (11424, 13845) with its patrol, flags 0; `0/7` on
1789 and `0/8` on 1813; `launch_frames` 1..15 over 1814..1828 with the
Biplane alone inside; `0/6` `returning` on 1985 at `mana_burn` 400.

**Where this crate parted: `GOLDEN_WORD_CHAPTER_TWENTY_NINE` = 1745,
open, on the first walk**: ours 5 draws against 4, ours alone `Guy::
set_anim+0x97a < Unit::do_idle+0x7d` — the Biplane idling on the map.
The widening, run308 whole, (605, 2071): through 1745 only chapter
twenty-two's standing rows (611–655), so **the three relaunches agree
whole**. **The first parting's field list**, 1746, `0/9` alone: `inside`
(ours −1, theirs 2007), `idle` (1, 0), `pos`, `orders_x/y` and the
figure's `x`/`y` ((11424, 13920), (11640, 13944)), `heading`,
`dest_angle` and the figure's angles (0, 0x55555555), `z` and `last_z`
(157, 0), the figure's clock, and `form` (parked 646). Their writers:
`inside`, `idle`, the angles and `z` — `come_out`'s EXIT, here alone;
`pos` — the EXIT here, and `Unit::init`'s seat there. The pool agrees
but for chapter twenty-two's ten rows: the press's `[2007]` is seated on
1542 by both.

**The arm built: `GOLDEN_WORD_CHAPTER_TWENTY_NINE` = 2070, closed** (item
915, `docs/PRODUCTION.md` "The trained aircraft", `Sim::build_train`'s
hangar test; and `do_launch`'s walk ending at a launch, `docs/ORDERS.md`
§41.3, which no capture reaches). **The value diff on 1746, both sides**
(ours before the build in brackets): `0/9` `inside` 2007 (−1), `idle` 0
(1), `heading` and `dest_angle` 0x55555555 (0), `z` 0 (157), the figure's
clock 0/0/−1 (1/8/0), no order, `mana_burn` 0 (0, then +1 a block); every
row after 1746 agrees — 1814..1828's counter among them — and the draw
stream to run308's end. The widening goes **262 → 17 rows**: chapter
twenty-two's nine, and the Biplane's `form` and point on 1746, ours
(11616, 13920) the base's own against (11640, 13944) — `Unit::init@
00612100`'s seat (parked 646), its one writer. `GROUND_INEXACT` 39 → 38:
the Biplane's EXIT read is gone. **Mutations**, each restored from git and
`touch`ed: the hangar arm dropped fails the new unit test and re-parts
run308 on 1745; the walk carried on after a launch fails
`a_launch_ends_the_walk_along_the_base_s_chain` alone.

## 39. Chapter thirty — the gather point: a Barracks' rally on the ground, on another building and on itself, a City's on a forest, and the Clear (item 928)

**Premise.** The player's rally point has no issuer in any capture and no
model in this crate: `input::Stream` skips a recorded `GatherPoint`, and
every building on disk has an empty list. Item 915 read three of its
readers — `Build::train@0062f9b0`'s two `CARRY_AIR` arms and
`Unit::come_out@00617c10`'s routing — and this chapter puts the command
and the ground arms under the light. What the reading says, as claims to
check:
- **The command writes a list on the building.** `CommandPackage::
  process_gather_point@00948510` hands `(x, y, action, add_to_end)` to
  `Group::action_gather_point@006ff1b0`, which gives each admitted member
  one `GatherPoint {x, y, action}` through `Build::add_gather_point@
  00622e70` (`BuildData +0xb8`, the count at `+0xc8`), the list cleared
  first unless `add_to_end`; `−1` in either coordinate is the Clear:
  `Build::clear_gather@00623180`. The dump prints it (`BuildData::
  log_data`'s last list: `length`, then per point `type`, `metric` and a
  `GATHERPOINT` with `x`, `y` and `action`).
- **A unit trained under a point leaves toward it.** `come_out`'s
  gather-point block (`6181a3`..`618377`) finds a free spot within `0x600`
  of the list's head point (`UnitType::find_nearby_spot@0061de70`) and
  stores `find_angle(spot − building)` in the unit's `angle` (`+0x50`) and
  in the exit sweep's bias (`[esp+0x2c]`, south otherwise, `docs/CITIES.md`
  §6.5.1).
- **Then it is sent there, and what it is sent to do depends on what is
  at the point.** With one point, the last is the first (`618c8d`): no
  building and no unit there, a military unit made at a Barracks, Stable
  or Dock (`TypeData::where`, `+0x40`) whose stance is not 5 takes
  `ATTACK_TO`, anything else `MOVE_TO`; a squad with a pushed group
  (882's push) goes through `Group::action_move_to` (`QUEUE_LAST`), a lone
  unit through `add_move_facing_order`. A friendly building with room and
  `action` ≠ 0 is garrisoned (`add_garrison_order` down the chain); a
  citizen at its own unfinished building builds, at a damaged one
  repairs, at a gather building with `action` ≠ 0 gathers.
- **A point on the trainer itself keeps the unit in.** The click on a
  building tile the member covers stores (−1, −1, 0) (`GatherPoint::
  is_inside`), and `Build::train`'s non-`CARRY_AIR` arm asks
  `BuildData::gather_inside@0046f180` before `come_out`: a unit with room
  (`num_inside ≤ get_garrison_limit`, 10 when the limit is 0) stays.
- **A City's point on a forest snaps to its Woodcutter.** When every
  member is a City centre (`COUNT_TYPE` VILLAGE `0x19e`), a forest tile
  (`TData & 0x30 == 0x30`) asks `ObjectsData::find_building` for a
  friendly Woodcutter within `0x600` and takes that building's point, the
  action unchanged (0).

**Under the emulator first** (a scratch script on `tools/emu/callfn.py`'s
machine through `command_oracle.py`'s fixture, out of git; the objects,
their types and the world synthesized, the member's vtable and its type's
stubbed; `add_gather_point` and `clear_gather` unchanged, `malloc` and
`free` adapted):

| the call | what it wrote |
| --- | --- |
| `issue_gather_point(group, 11424, 13920, 0, 0)` | 22 bytes: a fresh one-building `group`, then `16 [x][y][0][0]`; every other row 20 bytes behind the three-byte reuse, each field as passed, −1, −1 too |
| a point on `[Barracks]` | one `GatherPoint` (11424, 13920, 0), the list's count 1 |
| another point | the list cleared, the new one alone |
| two with `add_to_end` | appended in order: three points |
| a friendly object, action 1 | the point stored with action 1 |
| (−1, −1, 0, 0) | the list empty |
| a point on `[Barracks, University, Woodcutter, Senate, City]` | the Barracks, the Senate and the City take it; the University (named) and the Woodcutter (no `0x80000000`, no garrison limit) do not |
| a point past the world's edge | clamped to (width·0x300 − 1) |
| a building tile the Barracks itself covers | (−1, −1, 0): "inside" |
| the same on the Senate | the list cleared |
| a forest tile, `[City]`, no Woodcutter found | the point as given; with one found, the Woodcutter's own point, `ping_target` |
| a mountain tile (`& 3 == 2`), `[City]` | `find_building` for a Mine, the same snap |
| a forest tile, `[City, Barracks]` | no snap: not every member is a City |
| an "inside" head, then a point with `add_to_end` | the list replaced, not appended |

**What the emulator could not reach**: `come_out`'s two gather-point
blocks, whose `find_nearby_spot`, `find_any_building_at`,
`find_unit_with_radius` and orders read the world and the unit;
`Build::train`'s "inside" test with a real `num_inside`; the Airbase's
arm of `add_gather_point` (`build_masks & 8`), which re-orders the base's
planes; and a list of more than one point walked by a trained unit.

**The writers and readers, counted by offset** (823, 869): the list
(`BuildData +0xb8..+0xcc`, the count `+0xc8`, the tail `+0xcc`) is written
by `add_gather_point` and `clear_gather` alone, and they are reached from
`Group::action_gather_point`, `Group::action_city_gather` and the
scenario functions. Its readers: `come_out` (the head, `num_gather`, the
walk), `Build::train` (`+0xcc` at `62fadf`, `gather_inside`),
`BuildData::gather_inside`, `num_gather`, `log_data`, and the interface.

**The cast and the lines** are `chapter30.cmd`'s: chapter twenty-eight's
cast (two who=0 Barracks, `0/2007` at (2688, 14208) and `0/2008` at
(4224, 14208)), who=0's City `0/2000` at (3168, 30816) and Woodcutter
`0/2001` at (4224, 28608) from the map, and nine lines. **One capture buys
every arm** (879): each press precedes the unit its arm needs, and the
press that moves 2007's point falls between its two finishes.

**The staging, walked by this crate through the commands' own entries**
(`@gatherpoint` skipped as not modelled; run308's start, which is this
game to 605; run313 was not used). The frames are this crate's:
- presses processed on 617, 651, 701, 901 and 1101;
- 622: 2007 `[132]`; 642: `[132, 170]`; 662: 2000 `[Citizen]`; 712: 2008
  `[132]`;
- 760: the Citizen `0/10` out south of the City at (3192, 31800), no
  order, and a `GATHERORDER` of its own on 919;
- 856: the Hoplites `0/11`..`0/13` out on 2007's south ring, (2712,
  14904), no order;
- 953: the Hoplites `0/14`..`0/16` out on 2008's south ring, no order;
- 1060: the Bowmen `0/17`..`0/19` out at (2424, 14808), no order.
The tiles, by this crate's `TData` mask: (7, 63), 2007's point, plain
land; (20, 146) forest (`& 0x30 == 0x30`); (22, 74), 2008's centre, a
building (`& 3 == 3`).

**The readings, and the gates between each and its block** (903):
- **The list** (618, 652, 702, 902, 1102): written as read, or not at
  all. No gate but the issue: the DLL's refusal names itself.
- **The first Hoplites** (856): out toward the point and sent to it
  under `ATTACK_TO` as a group (**the reading**); sent under `MOVE_TO`;
  out south with no order (**this crate**). The gates before 856: the
  price and the pace, which chapter twenty-four measured on this block.
- **2008's Hoplites** (953): **inside 2008** (the reading, the "inside"
  point); out and sent back at 2008's point; out south with no order
  (this crate). Gates: the second price (53 food, 41 timber of 130 and
  121), the population (15 of 25), and the garrison limit (0 → 10).
- **The Bowmen** (1060): out toward 2008 with a `GARRISONORDER` a member
  into 2008 (**the reading**, `action` 1); sent at 2008's point under
  `ATTACK_TO` (the building arm not taken); out south with no order (this
  crate). Gates: 2008 holding its three Hoplites under a limit of 10, and
  `can_garrison` of Bowmen in a Barracks (chapter thirteen garrisoned a
  squad in one).
- **The Citizen** (760): out toward the Woodcutter and sent there with a
  plain move, no gather order (**the reading**: the snapped point keeps
  action 0, and the gather arm wants ≠ 0); a `GATHERORDER` at the
  Woodcutter from `come_out` (the snap read as a click on the building);
  out south with no order (this crate), gathering from 919. Gates: the
  snap needs the Woodcutter within `0x600` of (3936, 28128): 560.

**The capture must dump** `end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1`
and `misc:COMMANDMANAGER=1` over **`[605, 1450)`**: 845 blocks, **300 of
runway** past the Bowmen's garrison, predicted by 1150. `BUILDS=7`
prints the list; `UNITS=3` each unit's `inside_up`, order stack and
`angle`; `GROUPS=1` the pool, where each press's building group and each
squad's push are seated; `COMMANDMANAGER=1` the `process_gather_point`
line.

**The premise's killer, and its writers** (§3, point 5): each trained
unit's own `inside_up`, point and order stack on its birth block, and
each building's list on the press's block. The writers: the list's two
above; a unit's `inside_up` and point, `come_out` (and nothing on the
"inside" arm); its orders, `come_out`'s routing, `do_idle`'s auto-gather
for a citizen, and 882's push for the group. **The loops**:
`action_gather_point`'s over the group's members, `0..num`;
`come_out`'s over the points, `0..num_gather`, one here.

**What would falsify it, and where each could first fire.**
1. **An issue does not reach the pump.** Trace frames 616, 650, 700, 900,
   1100: an `INFO 17` with a refusal, or no `process_gather_point` on
   617, 651, 701, 901, 1101.
2. **The list** on 618 (2007: 1344, 12096, 0), 652 (2000: **4224,
   28608, 0**, the Woodcutter's point; 3936, 28128 if the snap is not
   taken), 702 (2008: −1, −1, 0), 902 (2007: 4224, 14208, 1, one point,
   the first replaced), 1102 (2007: `length 0`).
3. **The first Hoplites** (856), `0/11`'s own stack and point: a
   `GROUPATTACKTOORDER` on each of `0/11`..`0/13` under one group id,
   toward (1344, 12096), the captain out on 2007's ring on the bearing to
   the point (the reading); a `GROUPMOVEORDER` (MOVE_TO); no order on the
   south ring (this crate). And the captain's `angle` the bearing.
4. **2008's Hoplites** (953), `0/14`'s own record: `inside_up 2008`, no
   order (the reading), and so to the window's end; out.
5. **The Bowmen** (1060), `0/17`'s own stack: a `GARRISONORDER` on
   2008 (the reading), then inside 2008 by ~1150; a group order at
   2008's point; no order.
6. **The Citizen** (760), `0/10`'s own stack: a `MOVEORDER` toward
   (4224, 28608) and no `GATHERORDER` on its birth block (the reading); a
   `GATHERORDER` on 760; no order on the south ring.

Falsifiers 3 to 6 test each unit's own record on its own block (711),
and each splits three readings (789). **Not reached**: a list of more
than one point, the Airbase's arm, an enemy at the point (action 2), a
citizen at a building it can build or repair, and the Senate's clear on
itself.

**Where it should part.** This crate models none of it, so the first
value parting is expected on **618** (2007's list) and the first draw
parting on **856**, where the Hoplites' exit is swept from another
bearing and their orders move them.

**Run 2026-09-27 as run312 (item 928)** (`docs/RUNS.md` has the tables).
One take, `cover=0`, the same game as run304 to 759. **No falsifier
fired; every reading held.** The five presses were processed on 617, 651,
701, 901 and 1101, and the lists read as predicted on 618, 652, 702, 902
and 1102: the City's forest click took its Woodcutter's point (4224,
28608), 2008's click on itself stored (−1, −1, 0), the press on 900
replaced 2007's point, and the Clear emptied it. The first Hoplites came
out north-west toward (1344, 12096) on 856, each with a
`GROUPATTACKTOORDER` under group 0, and stood there from 936. 2008's
Hoplites stayed inside 2008 from 953. The Bowmen came out east on 1060,
each with a `GARRISONORDER` on 2008, and were inside on 1079. The Citizen
came out north-east on 760 with a plain `MOVEORDER` to (4248, 28632) and
no gather order, and took a `GATHERORDER` of its own on 984.

**Where this crate parted: `GOLDEN_WORD_CHAPTER_THIRTY` = 822, open, on
the first walk** (`@gatherpoint` skipped): theirs 7 draws against ours 6,
theirs alone `Guy::set_anim+0x97a < Unit::do_idle+0x7d`, the Citizen
`0/10` idling at its Woodcutter. The widening, run312 whole, (605, 1451),
now reads the gather list (`BuildDump::gather`, `build:gather_len` and
`gather[k].x/y/action`), and pins 696 rows and 29 pool rows. **The first
parting's field list**, 618, `0/2007` alone: `build:gather_len`, ours 0
against 1. Its writers are `add_gather_point` and `clear_gather`, and
here neither runs. Then 652 (`0/2000`) and 702 (`0/2008`), the same
field. On 760, `0/10`: `pos` (3192, 31800) against (3576, 29976), the
figure's point and angles, `heading` and `dest_angle` (0x55555555
against 314048512), `orders.len` 0 against 1, `orders_x/y` against
(4248, 28632), and `form` (parked 646). Their writers: `come_out`'s
gather-point block (the bearing), its routing (the `MOVEORDER`), and
`Unit::init` (`form`). The pool: each press's building group, seated
there by `process_group` on 618, 652 and 702 and absent here, and every
seat after it one stamp or one slot off.

**The gather point built: `GOLDEN_WORD_CHAPTER_THIRTY` = 1450, closed**
(item 928, `docs/PRODUCTION.md` "The gather point": `Sim::action_gather_point`,
`come_out`'s gather block and routing, and `Build::train`'s "inside" arm).
**The value diff, both sides** (ours before the build in brackets):
- 618: 2007's list (1344, 12096, 0) (empty); 652: 2000's (4224, 28608,
  0), the snap (empty); 702: 2008's (−1, −1, 0) (empty); 902: 2007's
  (4224, 14208, 1); 1102: `length 0`;
- 760: `0/10` at (3576, 29976) ((3192, 31800)), `heading` 314048512
  (0x55555555), one `MOVEORDER`, `orders_x/y` (4248, 28632) (none);
- 856: `0/11` at (2328, 13656) ((2712, 14904)), `heading` −385482752, a
  `GROUPATTACKTOORDER` each of three, `orders_x/y` (1368, 12120), the
  pool's slot 0 laid out (`form_num` 3, `o_angle` −381943808), and the
  members' `mirror` 1 (0);
- 953: `0/14`..`0/16` `inside 2008`, `group` −1 (out on the south ring,
  `group` 1).

The widening goes **696 → 105 rows** and the pool **29 → 6**. What stands:
- the births' `form` and 953's seat at the building's point, ours
  (4224, 14208) against (4248, 14232) (`Unit::init@00612100`, parked 646);
- 856's group move id, ours 861600 against 855000: `64 +` a pushed index
  here, `who·64 +` the slot there (parked 676's first);
- ~~1060..1079, the Bowmen's exit~~: built below (item 945);
- the pool's first seats' `ox`/`oy` (parked 887).

**Mutations**, each committed first, then restored from git and
`touch`ed:

| mutation | fails |
| --- | --- |
| the gather block's bias dropped | the ground-point and lone-unit tests; the word 822 |
| the routing dropped | the ground-point, lone-unit and garrison tests; the word 822 |
| the "inside" arm dropped | `a_squad_trained_under_its_trainer_s_own_point_stays_inside`; the word 952 |
| the garrison arm dropped | the garrison test; the word 1090 |
| `ATTACK_TO` read as `MOVE_TO` | the ground-point test; the widening |
| the City's snap dropped | the word 822; then, written for it, `a_city_s_rally_on_a_forest_takes_its_woodcutter_s_point` |
| the member's mirror flip dropped | the widening; then, written for it, `a_member_turned_past_ninety_degrees_to_its_captain_s_bearing_flips_its_mirror` |

**The re-seat built (item 945): 105 → 35 rows**, the word at 1450. Not a
refused ring: the routing's building arm sweeps the captain round 2007's
ring again from the bearing to 2008 itself, due east (`docs/PRODUCTION.md`
"The gather point"). **The value diff, `0/17`** (ours before in
brackets): 1060 `pos`, `g.x/y`, `g.des_x/y`, `orders_x/y` (3384, 14232)
both ((3336, 14376)); 1061 `dest_angle` and the move's `angle`
0x40000000 both (943259648); 1079 `inside` 2008 for all three (−1). The
members were already equal. What stands on 1060: three `form` rows (646).

## 40. Chapter thirty-one — the gather point's other arms: a lone unit under a ground point, a squad under a point on a unit, a list of two points, and a citizen's gather and build arms (item 955)

**Premise.** §39 closed the gather point on the arms run312 reached, and
left five that are read or built and that no capture measures (parked
946, 947, 948, 956, 957). This chapter stages four of them in one capture,
each arm between two staged events (879), as claims to check:
- **956, whether `FILTER_ALL` exempts the seeker.** The routing's re-seats
  sweep the trainer's ring under `FILTER_ALL`, which takes
  `find_nearby_spot`'s general path, `ObjectsData::find_unit_with_radius@
  00659890`. **The listing says it does not exempt it**: the filter is
  tested at `659a1a` and a zero skips `Search::valid_filter`, the only
  reader of `not_o`/`not_who`; and the search argument is never read, since
  `Search::valid_search` is called with selector 0 (`6599d4`..`6599db`),
  which answers 1. So every player's live, on-map unit is a candidate,
  the seeker too. This crate built the seeker exempt (item 945,
  `orders::Coll::All`). The killer: a lone unit whose re-seat's first
  candidate is the spot it already stands on.
- **957, the third re-seat.** At a last point with no building,
  `find_unit_with_radius(point, ·, who, 1, ·, FILTER_SEEN, who, 0)`
  (`619ab2`..`619ac2`) looks for a unit whose body covers the point. If
  it finds one, the captain is swept round the trainer's land ring from
  `find_angle(found − trainer)` (the register pair at `619b6b`/`619b92`)
  under `FILTER_ALL`, and `set_new_location(·, 1, 1)` puts it there
  (`619be7`..`619c08`). An armed captain and an enemy found take attack
  orders down the squad; anything else falls to the move arm.
- **946, a list of two points.** `Build::add_gather_point` under
  `QUEUE_LAST` appends at the tail, and the head stays the first point.
  `come_out`'s routing walks the list from the head (`618c10`..`61918f`).
  Each point after the head is replaced by its own free spot,
  `find_nearby_spot(point, 0, 0x600, 0, 0x55555555, FILTER_NOT_ME)`
  (`618c3d`..`618c83`), and a point with none is passed over. Every point
  but the last is a waypoint: a squad with a group is sent there by
  `Group::action_move_to(T, QUEUE_LAST, 1, find_angle(T − prev), MOVE_TO,
  1, …)`, with `T` the squad's own free spot round it (`619091`,
  `6190ce`); a lone unit by `add_move_facing_order` to its cell
  (`619129`). The origin `prev` starts at the exit point and becomes each
  waypoint's target (`61912e`..`61913e`). The last point is §39's
  routing: its own free spot, its kind, and the squad's angle from `prev`
  (`619e67`).
- **948, a citizen's gather and build arms.** A Citizen trained under a
  point on its own finished gather building with `action` ≠ 0 takes
  `add_gather_order(·, QUEUE_LAST, 1)`, and under a point on its own
  unfinished building `add_build_order(·, QUEUE_LAST, 0)`; both end the
  routing before any move. This crate built both from the reading, and
  **no unit test fails without them**: the three citizen arms removed,
  1,088 of 1,088 sim tests pass (the brief's mutation, 918).

**Not staged: 947, the Airbase's arm**, and the repair arm. Every air
chapter stands at `library who=0 6` and this cast is Ancient, so the
Airbase would be a second lever. The repair arm needs a damaged building,
which only combat stages (chapter twenty-one's). Both are named as their
own chapters in item 955's journal.

**Under the emulator first** (a scratch script on `tools/emu/callfn.py`'s
machine, out of git; `malloc`, `free` and `memcpy` adapted). `Build::
add_gather_point@00622e70`, `clear_gather@00623180`, `BuildData::
gather_inside@0046f180`, `get_first_gather@0046f140` and `num_gather@
0046f0a0` ran unchanged:

| the calls | what they wrote |
| --- | --- |
| p1 NEW, then p2 and p3 LAST | the list [p1, p2, p3]; `get_first_gather` and `gather_inside` read p1 |
| the save walk (`walk_data@004708a0`) | p3, p2, p1: tail first |
| inside NEW, then p LAST | [inside, p], `gather_inside` 1 (`action_gather_point` never does this: an inside head turns `add_to_end` off) |
| p NEW, then inside LAST | [p, inside], `gather_inside` 0 |
| a hangar (`build_masks & 8`, `is(0x1bf)`), a plane homed there, p1 NEW then p2 LAST | each press rebuilds the plane's patrol from the whole list: `add_air_patrol_order(p1, base, who, 1, ·)`, p2 appended to the patrol's x/y arrays; the QueuePos passed is the list's head node, a heap address |
| the same, action 3 | `add_strafe_order(o, who, base, who, 1, QUEUE_NEW, 0)` |
| QUEUE_NEW's clear on a hangar, the plane flying | `add_strafe_order(-1, -1, base, who, 0, QUEUE_NEW, 0)`: home |

`LinkListBase::add@00470c60` pushes in front, and `add_gather_point`
then steps the head on by one (`622f03`), which is the append.

**What the emulator could not reach**: `come_out`'s routing, which reads
the world, the unit and the list at process time. The listing settled
its registers, above. **Every `SEAM:` checked against the loops** (910):
the routing's loop is bounded by `num_gather` (`[esp+0x10]`,
`619187`..`61918f`) and walks `next` from the head (`619171`..`619184`),
so a waypoint is every point but the last, in list order. The move's
target is the free spot's cell, not the point's: the gather block writes
the spot to `[esp+0x44]/[esp+0x40]` and the raw head to `[esp+0x38]/
[esp+0x3c]` (`61823b`..`618279`), and every later point's spot goes to
the same pair.

**The writers and readers, counted by offset** (823, 869): the list
(`+0xb8..+0xcc`) has §39's two writers. Its readers gain the routing's
waypoint loop (`+0xcc` the head, `+0xc4` the walk) and `add_gather_point`'s
own hangar loop. A unit's point is written here by `come_out`'s exit and
its three re-seats (`set_new_location`), and its orders by the routing.

**The cast and the lines** are `chapter31.cmd`'s: chapter thirty's cast,
a second Chariot `0/10` idle at (2040, 15480), 1,427 south-west of 2007,
and eleven lines. The City's point on open ground (4608, 31104) before
three Citizens; its point moved onto its Woodcutter (action 1) between
the first and the second; a Lookout site placed by the first Citizen, and
the City's point moved onto it between the second and the third. 2007's
point on the Chariot before its Hoplites. 2008's two points, the second
by `@gatherpointadd`, before its Hoplites.

**The staging, walked by this crate through the commands' own entries**
on run312's start (the same game to 605; run339 was not used). A
prototype of 946's and 957's arms (a patch, reverted before the capture)
and the seeker counted (956's other reading) gave the readings' values:
- presses processed on 617, 621, 701, 703, 741 and 841, the build on 791;
- Citizens born 718, 824 and 938; 2007's Hoplites 858; 2008's 953;
- the Lookout `0/2009` at (7296, 34176) from 792, unfinished to ~1150.

**The readings, and the gates between each and its block** (903):
- **The seeker** (956), on three units, each its own killer:
  - `0/11` on 718 (ground point): (4104, 31032) exempt, **(3960, 31368)
    counted**;
  - `0/12` on 824 (the Woodcutter): (3576, 29928) exempt, **(3864,
    30168)** counted;
  - `0/16` on 938 (the site): (3912, 31416) exempt, **(3624, 31656)**
    counted.
  Gates: each exit ring clear of every other unit, and each press on its
  block.
- **The gather arm** (948, `0/12` on 824): a `MOVEORDER` to (4104,
  28824) then a `GATHERORDER` on `0/2001`, flags 4 (the reading and this
  crate); a `MOVEORDER` alone (the arm not taken). Gates: 742's list
  `action 1`; the Woodcutter finished and its own.
- **The build arm** (948, `0/16` on 938): a `MOVEORDER` to (7032, 34056)
  then a `BuildOrder` on `0/2009`, flags 0 (the reading and this crate);
  a `MOVEORDER` alone. Gates: the site placed on 791 (the Lookout
  available and paid; if the original refuses it, falsifier 2 fires on
  792 first) and still unfinished on 938.
- **The third re-seat** (957, `0/13` on 858): **the captain at (2376,
  14808)**, re-seated from the bearing to `0/10` (the reading); at its
  exit (2472, 14856) (this crate, which has no third re-seat). The
  members at (2472, 15000) and (2328, 15000) either way, and a
  `GROUPATTACKTOORDER` each, to (2232, 15528), (2088, 15480) and (2376,
  15576). Gates: `0/10` idle at (2040, 15480) on 857, and the Hoplites'
  pace (chapter thirty's, two blocks later).
- **The two points** (946, `0/17` on 953): **a `GROUPMOVEORDER` to
  (5208, 12696) then a `GROUPATTACKTOORDER` to (5016, 11160)**, the
  members' (5352, 12792)/(5160, 11160) and (5064, 12600)/(4872, 11160)
  (the reading); one `GROUPATTACKTOORDER` to (5208, 12696) (this crate:
  the head's spot, the last's kind); one to the second point's spot only
  (the tail read as the head). Gates: 704's list, both points in order.

**The capture must dump** `end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1`
and `misc:COMMANDMANAGER=1` over **`[605, 1400)`**: 795 blocks, **250 of
runway** past the Lookout's finish near 1150, the last staged event.

**The premise's killer, and its writers** (§3, point 5): each trained
unit's own point and order stack on its birth block, and each list on
its press's block. The writers: `come_out`'s exit and re-seats for the
point; the routing for the orders; §39's two for the list. **The loops**:
the routing's over `0..num_gather`, from the head; `action_gather_point`'s
over the group's members; `find_unit_with_radius`'s over the
`circle_radius[(r + 0x2ff) / 0x300]` blocks, or all eight players' units.

**What would falsify it, and where each could first fire.**
1. **An issue does not reach the pump.** Trace frames 616, 620, 700, 702,
   740, 790, 840: an `INFO 17` with a refusal, or no
   `process_gather_point` on 617, 621, 701, 703, 741, 841 and no
   `process_build` on 791.
2. **The lists and the site.** 618, 2000: (4608, 31104, 0); 622, 2007:
   (2040, 15480, 1); 702, 2008: (5184, 12672, 0); **704, 2008: `length
   2`, (5184, 12672, 0) then (4992, 11136, 0)** (the append; one point,
   or the second alone, kills it); 742, 2000: (4224, 28608, 1); 792:
   `0/2009`, a Lookout, unfinished; 842, 2000: (7296, 34176, 1).
3. **The seeker** (956): `0/11`'s point on 718, `0/12`'s on 824, `0/16`'s
   on 938, each against the two positions above. Any one at its counted
   position kills the exemption.
4. **The gather arm** (948): `0/12`'s own stack on 824, a
   `GATHERORDER` on `0/2001` or none.
5. **The build arm** (948): `0/16`'s own stack on 938, a `BuildOrder` on
   `0/2009` or none.
6. **The third re-seat** (957): `0/13`'s own point on 858, (2376, 14808)
   or (2472, 14856), with `0/14` and `0/15` at their exits.
7. **The two points** (946): `0/17`'s own stack on 953, two orders
   (`GROUPMOVEORDER`, then `GROUPATTACKTOORDER`) or one, and its point by
   ~1100: (5016, 11160) or (5208, 12696).

Falsifiers 3 to 7 test each unit's own record on its own birth block
(711), and each splits its readings (789). **Not reached**: the Airbase
(947); the repair arm; an enemy found at a ground point (the attack
arms); a waypoint with `action` ≠ 0; a point with no free spot.

**Where it should part.** This crate has the list and the gather and
build arms, and not 946's waypoint or 957's re-seat. The first value
parting is expected on **858** (`0/13`'s point), or on **718** if the
seeker is counted; the first draw parting where either captain's walk
starts from another cell.

**Run 2026-09-27 as run338 (item 955)** (`docs/RUNS.md` has the table).
One take, `cover=0`, the same game as run312 to 615. The presses were
processed on their blocks, 703's with `add_to_end` 1, and every list and
the site read as predicted; 2008's list is two points from 704, head
first. **Falsifier 3 fired, on all three Citizens**: `0/11` on 718 at
(3960, 31368), `0/12` on 824 at (3864, 30168) and `0/16` on 938 at
(3624, 31656), each the counted position. **`FILTER_ALL` counts the
seeker**, as the listing reads it; item 945's exemption was the untested
assumption, and it is dead. Every other reading held:
- 946: `0/17`..`0/19` on 953, each a `GROUPMOVEORDER` to (5208, 12696),
  (5352, 12792), (5064, 12600), then a `GROUPATTACKTOORDER` to (5016,
  11160), (5160, 11160), (4872, 11160); at the second point by 1100;
- 957: `0/13` on 858 at (2376, 14808), the third re-seat's point, its
  members at their exits;
- 948: `0/12`'s `GATHERORDER` on `0/2001` (flags 4) and `0/16`'s
  `BuildOrder` on `0/2009` (flags 0).

**Two values the prototype did not predict**, each a mechanism to build:
- each citizen arm's order **stands alone** (`orders_x/y` the Citizen's
  own point), where this crate puts a `MOVEORDER` ahead of it;
- `0/13`'s group target is (2424, 15672), `0/14`'s (2280, 15672), against
  the prototype's (2232, 15528) and (2088, 15480): beside the Chariot, the
  squad's target sweep refuses candidates the unit's own does not.

**Where this crate parted: `GOLDEN_WORD_CHAPTER_THIRTY_ONE` = 740, open,
on the first walk**: theirs 12 draws against ours 10, ours alone
`Guy::set_anim+0x97a < Unit::do_idle+0x7d`, the Citizen `0/11` idle at
its point here and still walking there. The widening, run338 whole, (605,
1401), pins 811 rows and 13 pool rows. **The first parting's field
list**, 718, `0/11` alone: `pos` (4104, 31032) against (3960, 31368),
`g.x/y[0]`, `g.des_x/y[0]`, `dest_angle` and the move's `angle`
1191706624 against 837877760, and `form` (parked 646). Their writer is
the lone arm's re-seat, `set_new_location`, whose sweep counts the seeker
there.

**The arms built: `GOLDEN_WORD_CHAPTER_THIRTY_ONE` = 1400, closed**
(item 955, `docs/PRODUCTION.md` "The gather point"). **The value diff,
both sides** (ours before the build in brackets):
- 718: `0/11` at (3960, 31368) ((4104, 31032)); 824: `0/12` at (3864,
  30168), `orders_x/y` its own point ((3576, 29928)); 938: `0/16` at
  (3624, 31656) ((3912, 31416)). The seeker counted: 740 → 884;
- 858: `0/13` at (2376, 14808) ((2472, 14856)), the third re-seat;
  `orders_x/y` (2424, 15672) ((2232, 15528)), the squad placement's
  target and its default span (884 → 892 → 1400); `dest_angle`
  2112749568 (−1911619584), the angle from the re-seat's spot;
- 953: `0/17`'s stack two orders, a `GROUPMOVEORDER` (kind 19) to (5208,
  12696) then a `GROUPATTACKTOORDER` to (5016, 11160) (one, kind 21, to
  (5208, 12696)).

The widening goes **811 → 15 rows** and the pool **13 → 4**. What stands:
- the births' `form` (parked 646);
- the group moves' ids on 858 and 953, ours `64 +` a pushed index and
  theirs `who·64 +` the slot (parked 676's first);
- the pool's first seats' `ox`/`oy` (parked 887);
- **1144, the builder `0/11`'s `group`**, ours 1 against −1, on the block
  its finished `BuildOrder` gives way to a `GATHERORDER`: the build
  line's, not the gather point's. It stood on the first walk (1134).
  Named, not built.

Chapter thirty holds at 1450 with its 35 rows and 6 pool rows: its
Citizen `0/10` ends at its exit point with the seeker counted too.

**Mutations**, each on the committed build (`62468432`), restored from
git and `touch`ed after:

| mutation | fails |
| --- | --- |
| the seeker exempt again | `a_lone_unit_under_a_ground_point_is_re_seated_past_the_spot_it_stands_on`; the word and the widening |
| the waypoints dropped | `a_squad_trained_under_two_points_moves_to_the_first_then_attack_moves_to_the_second`; the word and the widening |
| the third re-seat dropped | `a_squad_trained_under_a_point_on_a_unit_is_re_seated_on_the_bearing_to_that_unit`; the word and the widening |
| the citizen arms dropped | `a_citizen_under_a_point_on_its_own_building_gathers_there_or_builds_it`; the word and the widening |
| the squad's default span dropped | the word and the widening only |
| the squad's target a unit placement | the word and the widening only |
| the re-seat's spot not the leg's origin | the widening only (858's angle rows) |

Chapter thirty held under every one.

## 41. Chapter thirty-two — an Airbase's gather point: the planes re-ordered on each press, a second point appended, a plane trained under the list, and the Clear (item 947)

**Premise.** §39 and §40 closed the gather point everywhere but the
hangar, whose arms are read and not built, and which no capture reaches
(parked 947, §40's "Not staged"). As claims to check:
- **Every press re-orders the base's planes.** `Build::add_gather_point@
  00622e70` under `build_masks & 8` and `is(0x1bf)` walks the owner's live
  units whose domain is air (`type +0x218 == 2`) and whose
  `UnitData::home_base@00609dc0` is the base — the building a unit is in,
  or its air order's home — and rebuilds each one's orders from the whole
  list: the first point an `add_air_patrol_order(point, base, 1)`, the
  action bit, each later one appended to that patrol's arrays, an action-3
  point a strike. The old air order's `cruising_alt` and `sharp_turn` go
  into the new patrol when either is non-zero. `QUEUE_NEW` first runs
  `Build::clear_gather@00623180`'s hangar half: a plane on the map is sent
  home (`add_strafe_order(−1, −1, base, 0, QUEUE_NEW, 0)`), one inside
  loses its orders and leaves `launching`.
- **A plane trained under the list takes it.** `Build::train@0062f9b0`'s
  `CARRY_AIR` arm with points (`62fadf`..`62fbfb`) gives the order-less
  unit the same patrol, action bit set, and leaves it inside for
  `Object::do_launch@0064f3b0`.
- **The patrol walks its points.** `Unit::do_air_patrol@005ea620` flies
  at `x_pos[waypoint]` and steps the waypoint on within `0x240`; alone at
  the last point it circles there (ORDERS §34.5). This crate's patrol
  holds one point.

**Under the emulator first** (a scratch script on `tools/emu/callfn.py`'s
machine, out of git): `add_gather_point`, `clear_gather`, `LinkListBase::
add`, `make_valid` and `Build::train` ran unchanged; `home_base`,
`get_order`, `update_order`, the order adders and `clear_orders` hooked
on a model of each plane. Airbase `2007` of who=0, `build_masks` 0x1088:

| the call | what it wrote |
| --- | --- |
| p1 NEW; `0/6` flying on a patrol (1400, −1), `0/7` inside on one | `0/6`: the strafe home, then `add_air_patrol_order(p1, 2007, 0, 1, head node)`; `0/7`: `clear_orders`, then the same. Both (1600, 0): the strafe's heights, and none for `0/7` |
| p2 LAST | each patrol rebuilt `[p1, p2]`, waypoint 0; a plane's (1400, −1) and (1600, 1) carried; (0, 0) leaves the adder's 1600 |
| p3 LAST, action 1 | `[p1, p2, p3]`: a point's action is not read |
| (o 12, who 0) NEW, action 3 | each plane: the strafe home, then `add_strafe_order(12, 0, 2007, 0, 1, QUEUE_NEW, 0)` |
| `clear_gather` | the plane flying: the strafe home; inside: `clear_orders` |
| 0x1080, or `is(0x1bf)` 0 | nothing |
| `train(287)` under `[p1]`, `[p1, p2]`, `[(12, 0, 3), p2]` | `go_inside`, no `come_out`; `add_air_patrol_order(first, 2007, 0, 1, ·)`, the rest appended; (12, 0) flown to as a point |

**The listing settled the rest** (910, 964, 933):
- `add_air_patrol_order` is `ret 0x18` and never reads `[ebp+0x1c]`: the
  head node passed as its `QueuePos` (`62308d`) is inert.
- The plane loop is bounded by `objects +0x15c + 4·who` over
  `0xc0aec0 + 0x1c·who` (`622f5d`..`623166`); the list walk runs from the
  head to `+0xc4 == +0xcc` (`623029`..`623133`). The heights are read at
  `622fde`..`623026` and written at `62313d`..`62314e`.
- `train`'s loop re-reads `num_gather` each pass (`62fbf4`) and
  `seek_index`es each point; the order count `+0xd8` picks the adder.
- **`do_launch`'s store at `64f6a0` is the patrol's `waypoint`**, `+0x3c`
  in the PDB's `PatrolOrder` field list (`x_pos` 4, `y_pos` 32,
  `waypoint` 60). §38's table and `air.rs` have it as `returning`, which
  is `+0x58`. Only a relaunch from a waypoint past 0 tells them apart.
- Action 3 through the command reaches the Airbase by
  `action_gather_point`'s own loop, then `Group::action_flight` on the
  building group, whose `+0x49` arm is `Group::action_launch_flight@
  006fbfb0`, an issuer this crate does not carry. Not emulated, not staged.

**The booking's mutation** (918): the Airbase skip under action 3 in
`Sim::action_gather_point` dropped, **1,092 of 1,092 sim tests pass**. No
other hangar arm is carried to mutate.

**The writers and readers, counted by offset** (823, 869):
- the list (`+0xb8..+0xcc`): §39's two writers; its readers gain the
  hangar loop and `train`'s `CARRY_AIR` loop.
- a patrol's arrays (`+0x4`, `+0x20`): the adder, the hangar loop,
  `train`'s loop, `Group::action_air_patrol`'s append and `Unit::close`'s
  carrier shift; read by `do_air_patrol`, `do_strafe` (the last point)
  and `think_bird`.
- `waypoint` (`+0x3c`): the adder (0), `do_air_patrol` (past the length
  0, and the step), `do_launch` (0).
- `cruising_alt`/`sharp_turn` (`+0x4c`/`+0x50`): the adder (0x640; not
  `sharp_turn`), the hangar copy, a non-bomber's redraw every eighth
  frame, and the edge coin.

**What the emulator could not reach**: the patrol's walk in flight;
`action_launch_flight`; and whether the trained plane's launch shares its
birth block (the queue's `train` against `do_launch` in one building's
process).

**The cast and the lines** are `chapter32.cmd`'s: chapter twenty-nine's
thirteen, then `1600 @gatherpoint 0 11520 7680 0 2007` (P1, tile (60,
40)), `1700 @gatherpointadd 0 5760 5760 0 2007` (P2, (30, 30)) and `1850
@gatherpoint 0 -1 -1 0 2007`, the Clear. Both points are open ground,
a hundred and fifty tiles and more from who=1's town, so no bomber's search finds a
building near the last point. One lever a press, each between two staged
events (879): P1 with `0/6` flying and `0/7`, `0/8` inside; P2 before the
Biplane's birth; the Clear with all four flying.

**The staging, walked by this crate through the commands' own entries**
(`input::group_gather_point` on run308's start, the same game to 2070;
run345 was not used). Unbuilt, the list is written and no plane is
touched. A prototype of the arms (a patch, reverted before the capture)
gives the reading's values:
- **1602**: `0/6`, `0/7`, `0/8` each an `AIRPATROLORDER` over P1, flags 4,
  waypoint 0, `oxx` 2007, `cruising_alt` 1600;
- **1702**: each `[P1, P2]`; `0/6`'s `cruising_alt` its own of 1701
  (1800 here);
- **1741**: `0/6` within `0x240` of P1, waypoint 1;
- **1746**: the Biplane `0/9` born inside on `[P1, P2]`, flags 4; out on
  **1747** by this crate's order;
- **1789**, **1813**: `0/7`, `0/8` launched on the patrol;
- **1852**: all four a `STRAFEORDER`, no target, `returning` 1, flags 0;
- **2034**, **2073**, **2102**, **2106**: `0/8`, `0/7`, `0/6`, `0/9`
  inside with no order.

**The readings, and the gates between each and its block** (903, 968):
- **The NEW press** (1602): the reading above; **the list alone** (this
  crate: each plane's patrol over (21120, 16512), flags 0, unchanged);
  **the clear half alone** (`0/6` a strafe home, `0/7`, `0/8` no order).
  Gates: the press on 1601; `0/6` on the map (from 1585) and the two
  inside (to 1789, 1813); each homed at 2007 (its chain, or its patrol's
  `oxx`); the Airbase's 0x1088.
- **The LAST press** (1702): the arrays `[P1, P2]` on all three (the
  reading), or untouched. `0/6`'s height copied (its 1701 value) or the
  adder's 1600. Gate: `0/6`'s `cruising_alt`, redrawn every eighth frame
  (1300..1900 by hundreds on this crate's walk), is not 1600 on 1701.
- **The trained plane** (1746): an `AIRPATROLORDER [P1, P2]`, flags 4
  (the reading); no order (run308, and this crate). Launched on 1746 or
  1747. Gates: the price and age as §38; the counter saturated from 1600.
- **The walk** (~1741): `0/6`'s waypoint 1 then its heading on P2 (the
  reading); circling P1 at waypoint 0. Gate: its reaching P1 before the
  Clear.
- **The Clear** (1852): the strafe home on each plane (the reading); the
  patrols kept (the list alone). Gate: all four on the map.

**The capture must dump** `end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5`
and `misc:COMMANDMANAGER=1` over **`[605, 2360)`**: 1,755 blocks, **254
of runway** past the last landing near 2106. `UNITS=3` prints each
plane's stack, the patrol's arrays and `waypoint`; `BUILDS=7` the list,
`launch_frames` and the chain.

**The premise's killer, and its writers** (§3, point 5): each plane's own
stack on 1602, 1702, 1746 and 1852. The writers: the hangar loop,
`clear_gather`'s half, `train`'s arm, `do_air_patrol`'s step, `do_launch`,
`land_plane`. **The loops**: the planes by index, bounded above; the
list from its head; `train`'s over `num_gather`.

**What would falsify it, and where each could first fire.**
1. **An issue does not reach the pump.** Trace frames 1600, 1700, 1850:
   an `INFO 17` with a refusal, or no `process_gather_point` on 1601,
   1701, 1851.
2. **The lists.** 2007's on 1602 `[(11520, 7680, 0)]`; on 1702 `length 2`,
   P1 then P2; on 1852 empty.
3. **The NEW press** (1602), each plane's own stack: `0/6`, `0/7`, `0/8`
   one `AIRPATROLORDER` over P1, flags 4, `cruising_alt` 1600 (the
   reading); the old patrol, flags 0 (the list alone); a strafe home or
   no order (the clear half alone).
4. **The LAST press** (1702): each patrol's arrays `[P1, P2]`, waypoint
   0; `0/6`'s `cruising_alt` its 1701 value, or 1600.
5. **The trained plane** (1746): `0/9`'s stack `[P1, P2]`, flags 4, or
   empty; its `inside_up` −1 first on 1746 or on 1747.
6. **The walk**: `0/6`'s `waypoint` 1 near P1 and its heading on P2.
7. **The Clear** (1852): each plane one `STRAFEORDER`, `ox` −1,
   `returning` 1, flags 0, `oxx` 2007; or its patrol.
8. **The landings**: each plane inside 2007 with no order by 2359.

Falsifiers 3 to 7 test each plane's own stack on its own block (711),
and 3, 4 and 5 each split two readings or three (789). **Not reached**:
action 3 and `action_launch_flight`; a relaunch from a waypoint past 0
(the `waypoint` store); a helicopter's or a missile's arm in `train`.

**Where it should part.** Unbuilt, the first value parting is **1602**,
the three stacks; the first draw parting where a plane's flight starts
from another point.
