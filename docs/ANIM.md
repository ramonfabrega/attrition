# The animation clock

*Established 2026-08-24 from the decompile (`tools/ghidra/`, the export
under `decomp/`), the executable's own `UnitAnimCat` table, the install's
`anim_graphics.xml`, and two `DUMP_ALL` oracles: run12 (frames 0–3) and
run13 (sim-frames 95–103), read with `tools/gamelog/anims.py`. Implemented
in `crates/sim/src/anim.rs`, wired through `lib.rs`'s tick and `orders.rs`'s
stands; the harness side is `crates/rondata/src/{gamelog,diff}.rs`. The
document says how each claim was established and what it has not.*

*Amended 2026-08-29 (item 71): the **lengths** are no longer a dump's — the
install's own `unit_graphics.xml` gives every player unit piece its whole
slot list, and §3.2 is the arithmetic that says which piece an entry is.
Which of this document's claims a diff backs, and which rest on a reading:
**diff-backed** are the categories, the wrap and its roll, the mirror, the
arrival stand, the four gaia-piece lengths, the whole player piece table
(88 rows, five dumps) and the carrying walk's four bits — every one of them
is asserted in `rondata::diff` or `sim::anim`'s tests against a capture.
**Reading-only** are the boat-crew offsets, the squad's synchronised group
idle, the attack and death animations, the age brackets above `AGE0`, and
`Guy::move`'s `des_angle != angle` arm; §9 lists them.*

*Amended 2026-09-03 (item 205): a moving frame asks for the walk **twice** —
`Unit::move_step`'s own `set_anim` before `set_new_location`, and then
`Guy::move`'s — and only the second takes an overrun length off the clock.
§4.9, and it is **diff-backed** against run73, this map's first `DUMP_ALL`
window: 4,869 `GUY` fields over sixteen frames.*

*Amended 2026-09-02 (item 173): the **count** is no longer a guess. §3.5 is
`crew_size + squad_size`, the `CREW_SIZE` column plus the literal 1 every
type gets, and it is **diff-backed** — asserted against the `GUY` blocks of
every unit of every `DUMP_ALL` capture on disk.*

Every figure (`Guy`) of every unit plays an animation, `cur_time` frames into
one of `end_time`. `Objects::inc_time` steps every clock once a frame, and a
clock that runs out restarts its animation through `Guy::set_anim` — which,
for an idle animation, **draws once from `game_random`** to pick the next
idle variant. That draw is why the clock is simulation state and not
rendering: on run12's frame 0 forty of the 120 draws are the animals' first
idle rolls, and on run13's sim-frame 100 twelve of the 18 are fish whose idle
animations ran out together (`docs/SYNC.md` §4, §4.1). This document is the
mechanism behind those counts.

## 1. The record — `GuyData` (`rise.pdb`, size 0xbc)

| offset | field | what |
|---|---|---|
| `+0x74` | `cur_time` (ulong) | frames into the current animation |
| `+0x78` | `end_time` (ulong) | the animation's length; **0** right after `init_real` |
| `+0x7c` | `last_time` (int) | `cur_time` before this frame's step; **−1** right after any `set_anim` |
| `+0x88` | `gpiece` | the graphic piece — the model whose animation packet the lengths come from |
| `+0x8c` | `o` | the unit's object number |
| `+0x9a` | `guy_flags` | bit 4 an attack-2 double step, bit 0x20 the two-variant idle, 0x40 a plane, 0x80 **a scholar** (`init_real:230–234`: `TypeIndex` 52/53, `SCHOLARS`/`SCHOLARSKOREAN` — the second reading's correction; the first reading called it a boat's crew) |
| `+0x9c` | `cur_anim` (char) | the `UnitAnim` slot (§2) |
| `+0x9d` | `stopped` | the body stood on its destination at the last `Guy::move` |
| `+0x9e` | `hold_attack`, `+0xa0` `queued_attack` | combat's deferred animations (§6) |
| `+0xa2` | `guy_num` | the member's index in its unit |

A `DUMP_ALL` dump prints all of it per guy per pass (`GuyData::log_data@
005de6c0`); `rondata::gamelog::Guy` reads `cur_time`, `end_time`, `last_time`,
`cur_anim`, `gpiece`, `guy_flags`, `stopped`, `guy_num`. Below `DUMP_ALL` the
per-frame `GUY` blocks are empty.

## 2. The slots and their categories

`UnitAnim` (`rise.pdb` type 0x46B1), 38 slots:

```
0 DEFAULT  1–3 IDLE1–3  4–6 GROUP_IDLE1–3  7 SLOG  8 WALK  9 JOG  10 ATTACKWALK
11–14 ATTACK1–3, ATTACKSPECIAL  15–20 the six deaths  21–22 TURN_LEFT/RIGHT
23–24 PACK/UNPACK  25 CHOP_WOOD  26 WALK_WITH_WOOD  27 DUMP_WOOD  28 WALK_TO_WOOD
29 MINE_ORE  30 WALK_WITH_ORE  31 DUMP_ORE  32 WALK_TO_ORE  33 BUILD  34 REPAIR
35 SOW  36 REAP  37 FARM
```

`UnitAnimCat` — 38 dwords at `.rdata+0x2f370` (`S_GDATA32` at `0002:193392`
in the PDB; file offset `0x6f2b70`), read straight out of the executable:

```
slot 0–6 → 0     7,8,9 → 8     10 → 10     11–14 → 12     15–20 → 15
21 → 21  22 → 22  23 → 23  24 → 24  25 → 25  26 → 8  27 → 27  28 → 8
29 → 29  30 → 8   31 → 31  32 → 8   33 → 33  34 → 34  35 → 35  36 → 36  37 → 37
```

So the seven idles are one category (`CHAR_DEFAULT`), the seven walks — the
three plain ones and the four carrying ones — are one (`CHAR_WALK`), the
three attacks are one (`CHAR_ATTACK2`), the deaths one, and every work
animation is its own. **The category, not the slot, decides every branch in
`set_anim` and `inc_time`.** `anim::CAT` is the table; `anim::category`.

This corrects a row of `docs/SYNC.md` §3.2: the `param_2 == 0xc` branch of
`set_anim` is not the sow, it is the **attack** category (0xc = 12) — the one
draw that picks `ATTACK1`/`ATTACK2`/`ATTACK3` at 30/40/30 percent when the
request carries `param_3`. The sow (35) is its own category and its
`set_anim` never draws (`do_gather@005ef2a0:407` passes `(CHAR_SOW, 0, 1)`,
not 0 as that table said; it makes no difference).

## 3. The lengths and the pieces are art

`set_anim` ends by writing `end_time = animmgr.frames[packet.action_ids[cur_anim]]`
(`set_anim:725–736`) — the animation file's frame count, per graphic piece
and slot — and **3** for a slot the packet lacks. The piece is
`GraphicPieces::get_unit_gpiece@0090c030(type, who, o, guy_num, packing)`:

- a gaia type (`TypeIndex` `0x192..0x19e`, the birds and the herd animals):
  `first_bird_piece − 0x4b6 + 3·type + (seed + o) % 3` with `seed` the map
  seed — three pieces per type, cycling by object number. Run12 (seed 7236):
  sheep `o` 0, 1, 2, 3 → 60063, 60064, 60065, 60063; fish `o` 4, 5, 6 →
  60073, 60074, 60072;
- a player's unit: by the type, the tribe's art set (`tribes[+0x68]`), the
  leader's age bracket (`(age ^ 0x62766 + …) / 3`, capped at 2), the crew
  index `guy_num`, and — unless the type's `+0x2b8 & 4` — the **gender bit
  `o & 1`** (`get_unit_gpiece:155`), with fall-backs down the age and the
  set. Run12's citizens: player 0's odd `o` → 6688, even → 352; player 1's
  odd → 6336, even → 0. The scouts: 371 + 13043 (the dog) for player 0,
  19 + 12691 for player 1.

The sim reads none of the art. It takes both as an input, [`anim::Art`]:
`lengths: (gpiece, slot) → frames` and `pieces: (owner, type, sub, guy_num)
→ gpiece` with `sub` the gender bit or the gaia variant. `rondata` fills the
lengths from every `GUY` block of every pass of a `DUMP_ALL` dump
(`Log::anim_lengths`; run13 shows 33 pairs, run12 a few more) and the pieces
from the start dump's guys; siblings pool their lengths. A piece and slot the
table lacks gets `end_time = UNKNOWN` (never wraps, never draws — §9) rather
than the original's 3, because an unobserved length and a missing slot
cannot be told apart from a dump.

What the dumps show (`anims.py --lengths`): the citizen's `DEFAULT` is 33
frames and `IDLE1` 232 on every citizen piece; the walks 15 on every citizen
and scout piece, 16 on the sheep; the chop 33, the sow 47; the scout's
`DEFAULT` 61 (piece 371) and `IDLE2` 41; the sheep's idles 90/109/250 by
piece; the fish's 101/116/170 by piece, the same for every variant.

### 3.1 The install says it outright, for the gaia types (2026-08-28)

A dump is a weak source for this: it shows a length only where something
played it, and it shows **nothing at all for owner 9** — gaia's bird, whose
animation therefore never wrapped and never drew (`docs/SYNC.md` §3.9). The
install has the whole table. `Data/unit_graphics.xml` is the packet's
`action_ids` — `<UNIT name="HERDSHEEP-TYPE1">` with an `<ANIM
name="CHAR_DEFAULT" file="Sheep Idle3"/>` per slot — `Data/anim_graphics.xml`
resolves each name to a `.bha`, and the file's own key times give the frame
count. `docs/FORMATS.md`, "The animation file (`.BHa`)", has the container
and the arithmetic; `rondata::artdata` is the reader.

[`Art::gaia_lengths`] carries the result, `(TypeIndex, variant, slot) →
frames`, and **takes precedence** over the dump's table for a `(type,
variant)` it mentions: that pair's slot list is complete, so a slot it omits
is the packet's own missing slot and gets [`anim::MISSING`] — the
original's 3 — rather than `UNKNOWN`. The `-TYPE<v>` suffix **is** the
variant `(seed + o) % 3` picks; that link is the only one neither file
states, and it is what the two oracles' agreement pins.

Only the gaia types are read. A player's unit needs
`GraphicPieces::get_unit_gpiece`'s tribe, age and gender walk to know which
`<UNIT>` it plays, and none of that is modelled — the dump stays its source.
`FARMPIG` and `FARMCHICKEN` are left out too: they name a `-TYPE0` and a
`-TYPE1` and no `-TYPE2`, so a third of the pasture would be a guess.

*The check.* `the_install_s_gaia_lengths_match_the_dump_s` asserts every
gaia length run12's dump shows against the file — thirteen rows, the three
sheep pieces' `DEFAULT` (90 / 109 / 250) and the three fish pieces' idles
(170 / 101 / 116) — and `cargo run -p rondata -- <install>` re-derives the
header arithmetic. Two independent oracles for one table.

### 3.2 And for a player's units, by the piece number itself (2026-08-29)

~~Only the gaia types are read.~~ All of them are. What §3.1 said needed
`get_unit_gpiece`'s tribe/age/gender walk needs only its **arithmetic**,
because a dump already gives the piece *number* and the walk's whole job is
to produce one.

`GraphicPieces::init_piece_ranges@008f70e0` sets the unit half of the pool
out in four nested strides, and they are literals in the executable:

| field | value | what one step is |
|---|---|---|
| `first_unit_piece` | 0 | |
| `num_unit_pieces` | `0x160` = 352 | one art style; also `BASE_GAIATYPES − 0x32`, the unit records |
| `num_unit_pieces_per_age` | `0x840` | six art styles |
| `num_unit_pieces_per_gender` | `0x18c0` | three age brackets |
| `num_unit_pieces_per_crew` | `0x3180` | two genders |
| `total_num_unit_pieces` | `0xc606` | four crews, **plus six**: the "over time" pieces `get_unit_gpiece` reaches by `total − 6 … total − 1` |

so `get_unit_gpiece@0090c030`'s sum is

```
piece = (TypeIndex − 0x32)
      + 0x160  · art_style     # the nation's UNIT_CONTINENT, 0..5
      + 0x840  · age_bracket   # age < 5 ? age / 3 : 2
      + 0x18c0 · gender        # o & 1, or `packed` — the same slot
      + 0x3180 · guy_num
```

and `unit_graphics.xml`'s names are that sum spelled out:
`{GRAPH}-{STYLE}-AGE{0|3|5}[-PACKED][-CREW{k}][-FEMALE]`, where `GRAPH` is
the rules row's own `GRAPH` column (`UnitType::init` keeps it at `+0x88`)
and the age **digit** is the age, not the bracket. The six styles are
`say_unit_art_style_name@006f02e0`'s six arms, which name six consecutive
`internal_strings.xml` entries: `Europe` (written `DEFAULT`), `Arab`,
`American`, `Asian`, `NA`, `India` — the values a nation file's
`<UNIT_CONTINENT>` carries. A `<UNIT>` whose style is none of those (the
handful of `MERCHANT-NEUROPE-`, `-KOREAN-`, `-IROQUOIS-`, `-COLONIAL-`,
`-EINDIAN-` entries) is a name `get_unit_gpiece` can never build and holds
no piece.

So the whole table is readable without modelling a tribe or an age:
`rondata::artdata::piece_lengths` walks every `<UNIT>` entry, inverts its
name, and files its `<ANIM>` rows under the piece each of that `GRAPH`'s
`TypeIndex`es lands on — **1,359 pieces**. [`Art::piece_lengths`] carries
them, and it **wins over the dump's table** wherever it speaks; a piece it
names at all has its whole slot list, so a slot it omits is the packet's own
and gets [`anim::MISSING`], three frames.

*The check.* `the_install_s_piece_lengths_match_the_dumps` takes the
`(gpiece, cur_anim) → end_time` rows the dumps print over six pieces of two
nations and asserts every one against the install — which checks the
addressing, where §3.1's check checked the `.bha` arithmetic. It had teeth
on its first run: fourteen rows failed, and both families were real. **Its
dump list is the assertion**, and it was five dumps and 88 rows until
2026-08-31; it is eight and 187 now (§3.3).

**The six that still fail are the mirror, and that is the second half of
the check.** A crew member past the squad's size copies guy 0's `cur_anim`
and keeps its **own** `end_time` (§5), so the pair a dump prints for a
scout's dog is one animation's slot beside another's length. All six are on
the two dogs, and each is a length the *same piece* carries at another slot,
which is what says they are the mirror rather than a mis-addressed piece.
**A dump is not a reliable source of lengths for a mirrored guy at all**, and
nothing before this had noticed.

**The other nine were `man_walk.bha`.** `key_times` had required the key
array to fill its chunk exactly. `AnimObj::load_hier@0054b700` requires no
such thing — it reads `header[1].size` keys from `header + 12` and never
compares the two — and `man_walk.bha` says thirty and carries thirty-one. So
every citizen's `CHAR_WALK` and `CHAR_JOG` had been dropped on the floor, and
with them `Man Ouch` and the deaths. The test is `28 + 36n ≤ 16 + size` now.

**And no unit packet in the shipped file names a `CHAR_GROUP_IDLE2`** — none
of 1,337 `<UNIT>` entries — so `set_anim`'s captain gate (§4.2, the one frame
in sixteen that skips the idle roll) can never fire. That was `Art::group_idle`,
empty for want of a dump; it is now a fact, and the gate reads the packet.

### 3.3 A non-looping animation drops its last key (2026-08-31)

`AnimMgr::force_load@0053ade0` does not convert the root node's last key
time. It converts the last key time **of a looping animation** and the
*second to last* of every other one:

```text
ms = key_times[n − 1]
if loopings[i] == 0 and ms != 0:
    ms = key_times[n − 2]            # and it is written back into the node
frames[i] = round(ms · 3 / 200)      # half up, as before
```

`loopings[i]` is the section the row sits in.
`GraphicPieces::init_anims_pool@008fca40` reads `anim_graphics.xml` in four
passes — `AnimMgr::add(name, 1)` under `<LOOPING>` and `add(name, 0)` under
`<NONLOOPING>`, `<BUILDING>` and `<PATH>` alike — and `AnimMgr::add@0053ac00`
writes `loopings[i] = param_2 != 0`. So **three of the four sections are
non-looping**, and a section this reader does not know must be too.

The shape says why: a looping animation's last key is the frame that returns
to the first and is part of the cycle; a non-looping one's is the pose it
ends on, and it is not played through. In the shipped data it is always
worth exactly one frame.

*What it moved.* `lumberjack_dump.bha` ends 2157, 2190, so
`CHAR_DUMP_WOOD` is `round(2157·3/200) = 32` and not 33 — and 32 is what
every `GUY` block in the corpus prints for `cur_anim 27`, 756 of them over
the four citizen pieces `0`, `352`, `6336`, `6688`. The crate had 33 for a
month because none of §3.2's five dumps prints a `cur_anim 27`; the list is
eight now — run25 the three attacks, both dumps and the ore half, run44 the
turns and pack/unpack, run27 `CHAR_WALK_WITH_ORE` — 187 rows over sixteen
slots, and it fails on the old arithmetic.

Every length the rule changes shrinks by one, and they are the non-looping
slots: the three attacks, the six deaths, the turns, pack/unpack and the two
dumps. Two of gaia's are in it — `HERDHORSES` and `HERDBISON`'s
`CHAR_DEATH_STAB2` and `CHAR_DEATH_SHOT1`, 28 → 27 — and no capture has
played either. The bird's 31 and 23 are looping and stand (`docs/SYNC.md`
§3.9).

### 3.4 And the walk that hands a piece out, so a unit trained mid-game has one (2026-09-01)

§3.2 inverted the `<UNIT>` names to place every piece; it did not *pick*
one, which a unit the opening dump does not hold needs. `Art::pieces` is
seeded from the start dump's `GUY` blocks, so a Fisherman the AI trains on
frame 4,376 carried **gpiece −1**: every length lookup missed, every
`end_time` was [`anim::UNKNOWN`], and no animation of its ever wrapped.

`GraphicPieces::get_unit_gpiece@0090c030(type, who, o, guy_num, packing, …)`
picks it, and the sum is §3.2's:

```
piece = (TypeIndex − 0x32)
      + 0x160  · style     # tribes[leader.tribe].unit_continent (+0x68)
      + 0x840  · bracket   # ages < 5 ? ages / 3 : 2
      + 0x18c0 · gender
      + 0x3180 · guy_num
```

with `ages` read out of `LeaderDataEncrypt::ages` (`+0xdc`) through its
`^ 0x62766`; `who == −1` means style 0 and bracket 0.

**The function does not return that sum. It walks down from it**, four
loops, each stepping the age bracket down to 0 and taking the first piece
the art pool actually has (`data_pieces[p] != 0`, after a `verify_load`):

| loop | style | gender |
|---|---|---|
| 1 | yes | yes |
| 2 | **no** | yes |
| 3 | yes | no |
| 4 | no | no |

— and `first_unit_piece`, zero, when none of the four finds anything. The
crew coordinate is in every one: `guy_num` is never dropped. Loops 1 and 2
are skipped unless the gender coordinate applies (`LAB_0090c2ff`: the type
is not `unit_flags2 & 4`, `o` is not −1, and `o & 1`).

**`packing` overrides that test, and the two are the same coordinate.**
Pass it non-zero and the gender loops run whatever the object number says —
`-PACKED` and `-FEMALE` are one slot, and no shipped entry carries both.
`Guy::update_gpiece@005d8530` passes `unit_masks & 0x80000`, so a type that
packs is born on its packed piece (`Unit::init` sets the bit at `:376` and
makes its guys at `:540`) and moves off it when `SpellType::cast_unpack`
clears the bit and calls `Unit::update_gpiece@005e2920`.

That swap is why the piece matters to the simulation rather than to a
renderer. `FISHERMEN-DEFAULT-AGE0-PACKED` names a `CHAR_UNPACK` and no
`CHAR_PACK`; `FISHERMEN-DEFAULT-AGE0` the reverse. The deploy's animation
lives on the piece the boat is *on* when it plays it, and its length
decides the frame the clock wraps and pays an idle roll.

The existence test is [`Art::piece_lengths`] — a piece the install's
`<UNIT>` entries name at all is one that loads — and the walk is
[`Sim::unit_gpiece`], used for every player unit wherever the install table
was read; gaia keeps the dump's table, its pieces coming off
`first_bird_piece`, a runtime pointer no file states.

*The check.* `the_walk_gives_every_dumped_guy_its_own_piece` runs the walk
against every `GUY` block carrying a `gpiece` in ten dumps of two maps —
126 guys over 12 pieces, four nations, both genders, both crews — and
asserts the number. East Indies' word 4988 → 5106.

SEAM: the merchant family. `TypeIndex` `0x3d`, `0x3e` and `0x190` reach six
"over time" pieces at `total_num_unit_pieces − 6 … − 1` before any of the
four loops, selected by the nation's **`build_continent`** (`+0x64`, not
`+0x68`) and the age bracket; those six are the `-NEUROPE-`, `-KOREAN-`,
`-IROQUOIS-`, `-COLONIAL-` and `-EINDIAN-` entries whose names the piece
arithmetic cannot build. The arm is unmodelled, and ~~no capture on disk
holds a merchant~~ **run54 holds one from frame 6353** — but the arm is
still not reached by it: `get_unit_gpiece@0090c030` takes the over-time
branch only when its `param_5`, the **packed** flag, is zero, and a
merchant walks packed. So the art a merchant uses on its way to a rare is
the ordinary arithmetic's `-PACKED` slot, which this crate computes, and
the seam bites only once one has deployed (`docs/MERCHANT.md` §7).
**run490 reaches it** (item 1281): East Indies' who=1 merchants `1/19` and
`1/59`, deployed, are re-pieced by the third age on frame 11328, and on
block 11329 both figures read piece 50689 (`total − 5`: the
`build_continent` 1 arm) where this crate derives 2123 and 14795. The arm
adds **no crew coordinate**, and the piece carries no track, so the crew
figure reads `track` 0 and stands on its leader (`1/19`'s `x[1]` 32256
against ours 32332). Neither merchant's clock parts through run490's
window. The arm's entry, read off the listing: the type is `0x3d`, `0x3e`
or 400 and not packed; the bracket is 1, or 0 with `build_continent` 6;
and `build_continent` is 1, 3 or at least 6. The piece is `total − 6` for
3, `− 5` for 1, `− 4 + (bracket ≠ 0)` for 6, `− 2` for 7, `− 1` for 8.
Which `<UNIT>` name each of the six slots loads, and so their lengths, is
the loader's, and it is not read.

**And a trained unit's tracked crew figure is seated at birth.**
`Unit::init@00612100:549` is `set_new_location(x, y, 1, 1)`, whose
`param_3 = 1` reaches `Guy::set_new_location@005d86f0` on every crew figure
with its own `des` — so the figure is *placed* on its track offset rather
than left to walk there. This crate seated only the figures a dump or a
disembark handed it until 2026-09-02; `Sim::init_guys` now ends where
`Unit::init` does. It matters through §4.8: `Guy::do_turn@005d97a0`
recurses into the **trackless** crew only, so an unseated figure asks for a
turn animation its art has not got and pays the idle roll beside its
driver. East Indies' Merchant turned twice a frame for it
(`docs/MERCHANT.md` §5).

## 3.5 How many figures a unit has: `CREW_SIZE + 1` (2026-09-02)

`Unit::init@00612100:471`–`508` grows the guy stack to `n`, pops a
`Recycler<Guy>::pop@0046de80` into every slot from the unit's **old**
`guy_mark` up to `n`, sets `guys.length = n`, returns every slot past `n`,
and then walks `0..length` giving each figure its `Guy::init_real@005db6b0`
— one `game_random` draw apiece (§4). `n` is two `UnitTypeData` fields:

```
n = crew_size (+0x30c) + squad_size (+0x304)
```

**`squad_size` is not a column.** `UnitType::init@0061ab50:723` writes the
literal `1` into `+0x304`, unconditionally, for every type, immediately
before it reads `UBER_SIZE` into `+0x308` and `CREW_SIZE` into `+0x30c` with
`get_text_num(…, −1)`. Nothing else in the image writes `+0x304`;
`sim::anim::SQUAD_SIZE` is that literal. So the count is `CREW_SIZE + 1`: 1
for a Citizen, 2 for a Scout (the dog), 3 for a Caravan, 4 for a Trebuchet.
All 364 `<UNIT>` records carry the column, so the −1 default is never taken
— 304 zeros, 29 ones, 20 twos, 11 threes.

`Unit::init` then copies `+0x304` into `UnitData::guy_mark`, which is why
every `UNITDATA` block prints `guy_mark 1` and why that field is **not** the
stack's length: it is what shrinks as figures die, and §4's and §5's loops —
`0..guy_mark` and `squad_size..num_guys` — are the whole stack while none has.

`crew_size` is disjoint from `uber_size`, loaded on the line between: all
109 types with `UBER_SIZE 3` (the foot infantry) have `CREW_SIZE 0`, and
every crewed type has `UBER_SIZE 1`. An uber squad is three chained
`UnitData` (`UnitData::curr_uber_size@0060a760` walks `o_up`/`o_down`); a
crew is extra `Guy`s inside one.

*Established* by the decompile for the arithmetic, `types.txt` for the
names, `unitrules.xml` for the column, and **a diff for the number**:
`every_dumped_unit_has_crew_size_plus_one_figures` asserts `CREW_SIZE + 1`
against the `GUY` blocks of every unit of every `DUMP_ALL` capture — 120
units, 20 of them two-figure. East Indies' long word 6164 → 6166.

SEAM: no capture holds a three- or four-figure unit — every crewed unit on
disk is a Scout or a General at `CREW_SIZE 1`, and the Caravan at East
Indies 6164 is in a trace, which counts draws and prints no `GUY` block.

## 3.6 A crew figure's packet is empty, and every slot of it costs a draw (2026-09-02)

`CARAVAN-DEFAULT-AGE0-CREW1` is a whole `<UNIT>` entry:

```xml
<UNIT name="CARAVAN-DEFAULT-AGE0-CREW1" model=".\art\artillery_crew_driver.bh3"
      texture=".\art\caravan0.tga" cache="1" scale="1" .../>
```

Self-closing. **Sixty of the shipped file's 1,435 entries have no `<ANIM>`
child at all, and every one of them is a `-CREW{k}`.** The piece is real —
it names a model, so `GraphicPieces` loads it and `data_pieces[p] != 0` —
and its `AnimationPacket` is *empty*. Three things follow, and they are one
mechanism:

- **Every slot is three frames.** `AnimationPacket::get_game_frames@00918cc0`
  answers `3` for a slot the packet does not name ([`anim::MISSING`]), so a
  crew figure's `end_time` is 3 whatever it plays. Run64's caravan prints
  exactly that: `g1 piece 12681 anim 8 t 2/3`, beside its driver's `27`.
- **No slot of it loops.** `Guy::inc_time`'s test is
  `slot < packet->count && packet->ids[slot] >= 0 && loopings[id]` — three
  conjuncts, and the first two fail before the flag is read. So the wrap
  takes the *other* arm, `set_anim(cur_anim == 10 ? WALK : DEFAULT, 0, 1)`,
  and a `DEFAULT` request is §4's idle roll: **one draw**. For a
  category-0 animation the two arms make the same call, which is why the
  empty packet was invisible until one walked.
- **And a walk request resolves to `CHAR_WALK`.** The slot the speed picks
  is checked against the *asked guy's own* packet (`set_anim:596`), so a
  caravan whose driver slogs has two crew figures walking beside it —
  run64's frames 6168 and 6170, `g0 anim 7` against `g1 anim 8`.

Put together, a walking crew figure is a **three-frame metronome**: three
frames of walk, a wrap that falls to the idle and rolls, and `Guy::move`
putting it back on the walk the next frame. Two figures, so **two draws
every third frame, for as long as the unit lives** — 11,872 of them over
run54's last 17,800 frames, on 5,936 frames of which exactly two apiece.
East Indies' word had parted on the first of them.

The figure is *carried*, not walked: its `<UNIT>` names no `trackoffset`,
so `Guy::update_gpiece` leaves `track_dx`/`track_dy` zero and the guy has no
body of its own (§6). One consequence is its own: `Guy::do_turn@005d97a0`
writes guy 0's new angle, hands it to every crew figure as `des_angle`
through `Guy::set_angle`, then restores **guy 0's** `des_angle` from the
local it saved — and recurses into the untracked crew with the same pair.
So an untracked crew figure ends every turn with `angle == des_angle` and is
never owed one: on the frame a caravan starts moving, its driver takes
`Guy::move`'s turning arm and its crew takes the standing arm and is left
for the mirror. Run64's 6167, `g1 last_time −1, stopped 1`.

*Established* by `unit_graphics.xml` for the entries, the decompile for the
three tests, and **a diff for all of it**:
`run64_s_window_clocks_are_the_original_s` puts every `GUY` block's
`cur_anim`, `cur_time`, `end_time`, `last_time`, `gpiece`, `stopped`,
position and angle against this crate's over the eight frames of run64's
`DUMP_ALL` window — 2,061 fields, of which the only ones that part are the
caravan's own walk (`docs/QUEUE.md` item 177, thirty-four fields, pinned).
East Indies' long word 6169 → 6189.

SEAM: `unit_anims` had dropped an entry with no `<ANIM>` row since it was
written, so `Art::piece_lengths` did not hold the piece,
`get_unit_gpiece`'s existence walk missed it and fell to
`first_unit_piece`, and every crew figure in the game played **the
citizen's** art — looping, so it never wrapped and never drew. The count is
1,440 pieces now, not 1,359.

## 4. `Guy::set_anim@005da300` — the draw

Every request goes through the early returns, then the apply. The paths a
unit on open ground reaches, in the order the function tests them:

1. **The early returns** (`:155–224`). An idle request (`DEFAULT`, second
   argument 0) on a guy whose current category is idle returns while
   `cur_time < end_time` — the request is a no-op until the animation runs
   out. The same test covers **any** category that matches the current one,
   with the walk category excepted (it re-resolves its slot every time) —
   and `TypeIndex::BIRD` excepted from that exception (`:219`), so a bird
   inside its wing beat is left alone. On a **walking** guy it compares the guy's destination with its own
   position (`des_x != x − off_x`): a body still on its way returns (and, if
   the walk cycle has run out, rewinds `cur_time` to 0 — no draw); a body
   that has **arrived** falls through to the roll. That is the arrival draw:
   run13's sheep 0 at sim-frame 101, `SLOG 11/16 → DEFAULT 0/90`, one draw.
   Any other request whose category equals the current one and whose clock
   has not run out returns too, unless the second argument forces it.
2. **The idle roll** (`:254–320`), when the requested category is 0. First
   the group-idle gate: a captain (`o_up < 0`, every standalone unit) whose
   piece has a `GROUP_IDLE2` animation skips the roll one frame in sixteen,
   `(frame + 0x2e + o) & 15 == 0`, and takes `DEFAULT` — **and no shipped
   unit packet has one** (§3.2), so it never fires. Otherwise, **if the
   unit has no suspended search (`openlist == 0`), one draw**, `p = rand %
   100`. The variant is chosen only when the guy was already idle and the
   request carries its third argument (`local_20 == DEFAULT && param_3`):
   `p ≤ 69 → DEFAULT`, `≤ 82 → IDLE1`, `≤ 95 → IDLE2`, else `IDLE3` — with
   `guy_flags & 0x20` collapsing everything above 69 to `IDLE1`, and a
   peasant standing on a tile whose mask has `& 3` taking `IDLE1` for
   anything above 82. A guy arriving from a walk or a work animation draws
   but takes `DEFAULT` regardless. A variant the packet lacks falls back to
   `DEFAULT` (`:546–554`).
3. **The attack roll** (`:575–587`, the address `+0xf2f`), category 12 with
   the third argument: one draw, `p < 30 → ATTACK1`, `p > 70 → ATTACK3`,
   else `ATTACK2`. A rolled slot the packet does not name becomes
   `ATTACK2` (§4.13). **Nothing reaches it from the swing** — an attack asked
   of a guy still walking or still turning is deferred into `GuyData +0x9e`
   above these arms and paid by `Guy::move` on a later frame, which is
   where every attack roll on disk comes from: §6.2.
4. **The walk** (`:614–706`), category 8. **The slot asked for does not
   enter it**: the arm opens with the *category* — `CHAR_WALK` — as its
   answer, so every walk request is re-resolved from scratch and a
   `set_anim(CHAR_JOG)` is not a request for `CHAR_JOG`. From there:
   `SLOG` / `WALK` / `JOG` by the body's average speed against the type's
   base (`avg_speed / (moves · UNIT_MOVE_SPEED)` below `0.6f` slogs, above
   `1.1f` jogs — a float in the original, cross-multiplied here; the
   lengths are equal on every piece observed so the boundary is
   unobservable). **It is the asked guy's own average** — `this->field_0x84`
   at `005db438`, not guy 0's — and that is what makes a *tracked* crew
   figure jog beside a walking leader: `Guy::move`'s tracked branch pays it
   `(get_speed · 11) / 8` a frame to keep station, eleven eighths of the
   leader's base (`docs/MOVEMENT.md`, "`off_x` and `off_y`"'s section, last
   paragraph). run67's merchant crew is `cur_anim 9` against its driver's 8
   on all sixty blocks, and the cost of reading guy 0's was a draw on every
   arrival, because §4.6's arrival test is on the **slot**; an **owner-9** bird's is a coin instead (`% 100 > 49 →
   JOG`, `docs/SYNC.md` §3.9); then the carrying walks override, from
   `unit_masks & 0x78000000` rather than from what the caller named —
   `0x10000000` `WALK_TO_WOOD`, then `0x8000000` `WALK_WITH_WOOD`,
   `0x40000000` `WALK_TO_ORE`, `0x20000000` `WALK_WITH_ORE`, tested in that
   order (`005db61f`–`005db665`).

   **The mask is the whole of it, and it is not the gather order.**
   `Unit::do_non_flat_gather` is its only writer: it clears the nibble at
   the head of its arrived half (`:96`) and sets one bit on each walk it
   issues — `WITH` on the walk back to the camp (`:150`/`:154`), `TO` on
   the approach to a tile (`:376`) and on the walk out a tile choice ends
   with (`:694`/`:700`). `Unit::think` clears it for a citizen (`:82`),
   `kill_current_order` for anybody (`:107`). So the citizen's **first**
   walk to its camp — issued by `find_gather_spot`, before
   `do_non_flat_gather` has ever run for it — is the plain `CHAR_WALK`, and
   it ends in the arrival stand below. This crate derived the slot from the
   order's `goto_build` until 2026-08-29, which made that walk a carrying
   one and cost the stand; it was invisible while the carrying slots had no
   length, because the packet fallback turned every one of them back into
   `CHAR_WALK`. A
   slot the packet lacks falls back to `CHAR_WALK` (`:596`) — **the asked
   guy's own packet**, which is how a caravan's driver slogs while its two
   crew figures walk (§3.6).
   A walk already playing keeps
   its time, and so does a walk-to-walk slot change: ~~rescales `cur_time
   · len_new / len_old`~~ the rescale at `:691` passes the **old** slot to
   both `get_anim_time` calls (`ecx` is loaded at `0x5db404` and the new
   slot is only written at `0x5db46a`, after both calls — the second
   reading's finding, verified in the listing), so it is `cur_time · t /
   t`; a time past the new slot's end stands until the next step wraps it.
5. **The apply** (`:559–573`, `:707–717`): a new slot starts at `cur_time =
   0`; the same slot keeps what ran past its end, `cur_time −= min(cur_time,
   end_time)`. Then `last_time = −1` (set at `:253`) and `end_time` from the
   table.
   **The walk category's same-slot arm is not that one** (2026-09-02).
   `:667` is `if (cur_time < len) goto <past the write>` — a walk still
   inside its animation keeps the clock it has untouched, and only one
   that has run past takes `cur_time −= len`. The other arm's
   `cur − min(cur, len)` is the same thing for an overrun and **zero**
   for a clock still running, and applying it to a walk froze every
   walking guy at `cur_time == 1`: `Guy::move` asks for the walk again on
   every frame and `Guy::inc_time` stepped it straight back. No guy
   walking for longer than its cycle could then wrap, and a wrap is a
   draw. Found from run64's frame 6169, where the original spends two
   `Guy::set_anim+0x97a < Guy::inc_time+0x271` this crate spent none of
   (`docs/CARAVAN.md` §4.1); the two are still missing for a second
   reason and `docs/QUEUE.md` item 176 is that.

`Unit::set_anim@00616f40` runs `Guy::set_anim` for guys `0..guy_mark` and
`squad_size..num_guys` — every member, so a scout's dog rolls its own
variant on the unit's idle request (§5 says why that never shows).

**The callers on open ground**, and which of them draw:

| caller | request | draws when |
|---|---|---|
| `Unit::do_idle@0060dcd0` (no order) | `(DEFAULT, 0, 1)` | the guy's idle ran out (never — phase 7 caught it first), or the body just arrived from a walk |
| `Animal::do_idle@005d7460`, every idle frame | `(DEFAULT, 0, 1)` | as above — the sheep's arrival. Its own four draws follow, on the frame the idle's last tick falls: the wander coin at `+0x83` (three in ten), then the direction (`& 7`) and the two step counts (`& 3`) at `+0x1a4`, `+0x1d4`, `+0x212` — `docs/SYNC.md` §3.10 and §7 |
| `Unit::do_non_flat_gather@005f0170:512` (`+0x10f`), the tile choice; `:135` (`+0xb99`), the return to camp; `:275` (`+0xfd4`), the tile approach | `(DEFAULT, 0, 1)` | ~~the woodcutters' four draws at frame 0 (§8)~~ **not at frame 0 on any traced map** — run14 and run20 have no `do_non_flat_gather` draw there at all, and the four are wraps (§5). Run21 reaches `+0x10f` at frame 381, `+0xb99` at 526 and `+0xfd4` at 23,299 |
| `do_non_flat_gather:130`, `:267`, `:271`, `:412`, `:416` | `CHOP_WOOD`, `MINE_ORE`, `DUMP_WOOD`, `DUMP_ORE` | never — their own categories. **There is no `CHAR_DEFAULT` beside `:412`/`:416`**: the camp-arrival branch is those two and nothing else (`5f0b5e`–`5f0b89`), which is what the sim had wrong until 2026-08-26 (`docs/SYNC.md` §6) |
| `do_gather:407`, `:473`, `:479`, `:486` | `SOW`, `REAP` | never |
| `Unit::do_build@005eebf0:step 4`, every frame an adjacent builder builds | `CHAR_SOW` on a `FARM` (`is(0x1a1)`), else `CHAR_BUILD` | never — its own category. **What it costs is the arrival stand it prevents**: §4.6 |
| `Unit::do_repair@005ee420:1`, ahead of every gate | `CHAR_REPAIR` | never, and unconditionally — even on the frame the order dies |
| `Guy::move@005d9240:86`, `Unit::move_step@005faf30:304` | the walk | never (a bird's coin aside) |
| `Guy::move:59` (`+0x19f`), the frame after a walking guy stops with the plain `WALK` slot and nothing else changed it | `(DEFAULT, 0, 1)` | the arrival, when no order made the request first. **The one caller that reaches `Guy::set_anim` directly** rather than through `Unit::set_anim+0x56`, so its chain is a frame shorter and the trace's disambiguator sits at `up[0]` (`sim::anim::SITE_ARRIVE`, `docs/SYNC.md` §3.10). **A crew figure that was teleported never gets here**: §4.10 |
| `Guy::move:78` (`+0x14f`), the **turn arm** — a body standing on its unit whose angle has not reached `des_angle`, every frame it is still turning | `(CHAR_WALK, 0, 1)` | never itself, but it puts the guy back on the walk category, so the *next* frame's idle request rolls again. That is the second draw of an arrival pair (`docs/SYNC.md` §3.11). ~~**Unmodelled**, and what it costs is there too~~ **Modelled 2026-08-31, §4.7**: what it cost was a frame of every work animation an order sets on the frame it turns |
| `Unit::move_step:148` (`+0x3b6`) and `:170` (`+0x389`), the two **turn-in-place** arms, and `Guy::turn_towards+0x69` under `Guy::move:109` | `(CHAR_TURN_LEFT/RIGHT, 0, 1)`, on `guy_flags & 8` | when the piece has no such slot: the request becomes `CHAR_DEFAULT` and a guy on the walk category rolls — §4.8, the AI's fishing boat |
| `Unit::move_step:378` (`+0x4e2`), a unit whose **snap onto its waypoint** is blocked — `move_step`'s other arm, which resolves nothing and consumes the waypoint where it stands | `(DEFAULT, 0, 1)` | the same conditions as any idle request; Great Lakes 9134 is the first frame of either map to be read against it (`sim::anim::SITE_SNAP_BLOCKED`, `docs/COLLISION.md` §5.4, §8.9) |
| `Unit::move_step:281` (`+0x823`), a unit whose step is blocked, before the three give-up tests | `(DEFAULT, 0, 1)` | the same conditions as any idle request — three times on run14 (frames 122, 184, 256), and this crate takes the first two on the original's own frames (`sim::anim::SITE_BLOCKED`, `docs/COLLISION.md` §5, §8) |
| `Unit::go_inside@0061a2e0:+0x280`, a **scholar** being seated on its container | `(DEFAULT, 1, 1)` | **always** — the `force` skips every early return, so it is one draw a figure, once a scholar. The fourteenth caller of this address, and the last to be found: nothing on either map reaches it before Great Lakes 8272, and it fires on fourteen frames of run53 and fourteen of run54 (`sim::anim::SITE_GO_INSIDE`, `docs/CITIES.md` §6.5.2) |
| `Guy::inc_time` (§5) | the wrap | an idle running out |

## 4.6–4.8 What a turn costs the animation

Three findings that share a mechanism: an order, a step or an idle frame
moves the unit's *heading* without its *facing*, and `Guy::move` spends the
frame turning instead of playing what was asked for. They keep their `4.x`
numbers — the code cites them — and sit under a heading of their own because
§4 had outgrown what a session reads.

### 4.6 The work animation is what keeps a worker off the arrival stand

The arrival row above turns on `field_0x9c == 8` — the **slot**, `CHAR_WALK`,
not the category (`Guy::move:57`; `set_anim` indexes `UnitAnimCat` by the same
byte, so it is `cur_anim`). A guy that reaches its destination and is still on
that slot the next frame spends an idle roll. A worker never is, because its
order put it on a work animation first, and that is one draw a frame's stream
either has or does not.

Diff-backed 2026-08-28 (item 59), on run14's frame 18. Player 1's citizen
`uid 7` reaches its build site on sim-frame 16, `Unit::do_build` turns it on
17 (`angle -136249344 → -292028416`, the dump's own two frames) and this
crate — which had step 4's `set_anim` missing — spent a
`Guy::set_anim+0x97a < Guy::move+0x19f` at index 0 of frame 18 that the
original does not. With the work animation in, run14's traced stream goes
from parting at frame 18 to parting at **99**, and from 173 of 284 frames
matching to **219**.

The order the original writes them in is worth keeping: `do_build` sets the
animation at step 4 and *then* faces the site (`set_angle(…, 0)` — the heading
only, no snap), so on the frame the site is first faced `Guy::move` takes its
`des_angle != angle` arm instead, which puts the guy back on `CHAR_WALK` and
clears `field_0x9d`; the *next* frame's `do_build` puts it on `CHAR_BUILD`
again, before `Guy::move`'s arrival test reads the slot. Either way the test
never sees a walk, which is why no capture has an arrival stand for a builder.
That arm is the table's new row above, and the builder is the one caller whose
answer does not depend on modelling it.

### 4.7 An order that turns loses that frame's animation (2026-08-31)

§4.6's last paragraph read the rule and left it unmodelled, because for a
builder it cancels. For everybody else it does not, and it is worth a frame
of every work animation.

`Unit::set_angle@00605400` writes the unit's own `+0x50` and then
`Guy::set_angle@005d9010`, whose **third argument is a snap flag**: with it
zero the guy takes `des_angle` (`+0x64`) alone and its own `angle`
(`+0x18`) stands, to be turned by `Guy::move`. Every `set_angle` in the
ordinary order path passes **0** — `do_non_flat_gather:208` and `:402`,
`do_build:155`, `do_gather:382`, `fight:725`, `move_step`'s three,
`do_group_attack`'s two, `do_attack_ground`, `do_form_change`, `check_idle`.
The flag is set in five places this crate does not reach: `do_move`'s
re-face, `go_inside`, `do_spec_anim`, `dbg_jump_to_action` and the scenario
loader.

So on the frame an order turns its unit, `Guy::move` finds the body **at its
destination and owed a turn** and takes the turn arm: `set_anim(CHAR_WALK,
0, 1)`, on top of whatever the order asked for a moment earlier. The work
animation takes hold on the *next* frame, when the angle has arrived — for a
foot type that is one frame, because a standing body's `last_speed` is zero
and `GuyData::turn_speed` returns instant for it (`docs/SYNC.md` §3.11).

*The record* is run44's citizen `0/2`, piece 352, frame 542 — the order ran
its at-camp branch and asked for `CHAR_DUMP_WOOD`, and the turn arm overwrote
it with the carrying walk in the same frame, so the dump landed a frame late
(`docs/JOURNAL.md`, 2026-08-31). This crate's `Movement::set_facing` snapped
facing, heading and `des_angle` together, so the arm never fired and the dump
landed a frame *early*; `Movement::set_heading` is the zero-flag call and
every order site uses it now.

*What is not established.* That every one of the twelve zero-flag call sites
above wants this in *this* crate: the evidence is run44's camp arrival, and
the other five sites the crate reaches (`do_build`, the oil well's stand,
`fight`, the reload turn, the tile arrival) were changed with it because the
original's argument is the same. None of them moved a number either way on
the two traced maps.

**The arm itself, modelled 2026-08-30** (`Guy::move@005d9240:73–89`,
`crates/sim/src/anim.rs`'s `guys_follow`). A body standing on its unit with
`des_angle != angle` is put back on `CHAR_WALK` — the plain one; the carrying
walk is resolved in the *moving* half — and marked unstopped, so the next
frame's `do_idle` sees the walk category and a stopped body and rolls again.
That second roll is the arrival's second draw, run39's frames 19 and 20
(`docs/SYNC.md` §3.11). Its only guard is a **sea** unit (`type+0x218 == 1`)
or a `SPECIAL_ANIM` order, the second of which this crate does not model at
all.

**And `Guy::do_turn@005d97a0:15`'s override could not fire on any
capture — until run44, below.**
With `guy_flags & 8` the turn replaces that walk with
`CHAR_TURN_LEFT`/`CHAR_TURN_RIGHT`, which `Guy::move`'s own slot exclusions
then keep, so such a guy spends no arrival draw. ~~`Guy::init_real@005db6b0:179`
sets the bit only for a guy whose piece names a turn animation~~ — **`:179`
is one of two writers, and §4.8 is the other**: a type that packs carries the
bit with no turn animation at all, and pays an idle roll for it. Of the
animation half: 273 of the install's 1,359 unit pieces name a turn, and
**none of the eight a `DUMP_ALL` run's guys carry** — 0, 19, 352, 371, 6336,
6688, 12691, 13043 — while gaia's 60063–60074 are not `<UNIT>` entries at
all. Asserted in `rondata::diff`'s
`the_install_s_piece_lengths_match_the_dumps`, beside the `GROUP_IDLE2`
finding of §3.2.

**Fired 2026-08-31** (`docs/ORACLE.md`, run44): 452 guy-frames play
`CHAR_TURN_LEFT`/`CHAR_TURN_RIGHT`, nine `(who, o, slot)` combinations
(`tools/gamelog/turnanim.py`), once a capture had combat and `GUYS=4` at
once — `Guy::move:109`'s standing arm hands the override through
`Guy::turn_towards@005d9720`, so a turner turning towards a target is
enough. The types that *play* one are 134, 265 and 266, **not** the set
carrying `guy_flags & 8` (§4.8); the scored games have none of them, so
playing one is still unmodelled and moves no score.

### 4.8 The turning stand — a turn animation the boat does not have (2026-09-01)

§4.7 read `Guy::do_turn`'s override and left it unmodelled, on the strength
of "none of the eight pieces a `DUMP_ALL` run's guys carry names a turn
animation". That is still true, and the conclusion drawn from it was still
wrong: **`guy_flags & 8` does not mean the guy has a turn animation.**

*The bit has two writers, and they are independent.*
`Guy::init_real@005db6b0` sets it at `:179` when the piece's packet names
`CHAR_TURN_RIGHT` — slot 22, `action_ids[0x16]`, and only that one — with a
file that loads; and again at `:215`, which is the `else` of `(type+0x2b8 &
4) == 0`, so **every type that packs gets the bit whatever its art says**.
The same flag is `needs_packing`, the one `get_unit_gpiece:155` reads to
suppress the gender bit (§3).

*What the packing writer costs.* `Guy::set_anim@005da300:225–252` opens the
non-idle path by testing the request: a `CHAR_TURN_LEFT`/`CHAR_TURN_RIGHT`
whose packet has no `get_animobj` for that exact slot is rewritten to
**`CHAR_DEFAULT`** — past the `param_1 == CHAR_DEFAULT` early returns, which
are the `else` arm above it and are not re-entered. So the request lands in
the common tail as an idle request, and a guy whose current category is the
**walk** — which a sailing boat's is, from `Unit::move_step:304`'s
`set_anim(CHAR_WALK, 0, 1)` — falls straight through to the roll and **spends
a draw**. Its variant is `CHAR_DEFAULT` regardless (§4.2: the variant is
picked only when the guy was already idle), and the next frame of the same
turn returns early on the clock, so it is one draw an episode, not one a
frame.

*The record.* run26's `DUMP_ALL` window states the flag by type, which is
what makes this diff-backed rather than a reading:

| type | packs | names a turn | `guy_flags` |
|---|---|---|---|
| `FISHERMEN` (317), `MERCHANT` (61) | yes | no | **8** |
| `CATAPULT` (265), `TREBUCHET` (266) | yes | yes | **8** |
| `PIKEMEN` (134) | no | yes | **24** (`8 \| 0x10`) |
| `TRIREME` (340), `GALLEY` (341), `DROMON` (324) | no | no | **0** |

`MERCHANT` is the row that settles it: it packs (`trader_id`, `docs/AI.md`'s
`flags2`), names no turn animation in `unit_graphics.xml`, and carries the
bit anyway.

*Where the override is asked for.* Four callers pass `do_turn` a non-zero
fifth argument, and three of them fire in a traced game (run54 / run53
counts):

| caller | `via` | fires |
|---|---|---|
| `Unit::move_step:148`, the **near** arm — within a tile, or `TURN_FIRST`, and owing any turn | `move_step+0x3b6` | 52 / 75 |
| `Unit::move_step:170`, the **far** arm — 45° owed (80° for a ship two tiles out) | `move_step+0x389` | 17 / 23 |
| `Guy::turn_towards:41`, which `Guy::move:109`'s standing arm calls while `guy_flags & 2` is clear | `turn_towards+0x69` | 8 / 6 |
| `Unit::detect_boat_collision@005fa8b0:267` | — | 0 / 0 |

**The near arm is the later address.** The compiler laid the source's first
arm out second — `005fb24c` and `005fb256` both jump *forward* to it — so
`move_step+0x3b6` is the near call and `+0x389` the far one, the opposite of
what the decompile's line order suggests. Reading it the other way round
costs a frame: the count agrees and the label does not.
`crate::anim::SITE_TURN_NEAR`, `SITE_TURN_FAR`, `SITE_TURN_STAND`.

*What it moved.* East Indies' word **4945 → 4950**; every other capture is
unchanged, which is the expected shape — the only unit in these games that
asks for an animation it lacks is the AI's fishing boat.

*And the tracked crew figure pays it too* (2026-09-04, item 212). The row
above is read as guy 0's because `Guy::move:109` is the caller and `do_turn`
recurses only into the **trackless** crew. Both halves are true and the
conclusion was short: `Guy::process → Guy::move` runs for **every** guy, so a
*tracked* figure standing on its offset with `des_angle != angle` reaches the
standing arm on its own account, and `guy_flags & 8` is the **type's**
(`init_real:215`). A merchant's driver catching its leader up after a turn
therefore asks for a `CHAR_TURN_RIGHT` the art has not got, and rolls — once
an episode, on the frame its walk ends. The gate cannot close on it: bit 2's
only writer is `do_turn:15`, and `do_turn:37` recurses from `+0x304` into the
crew with *no* track.

*The record* is run18b, which is run53's own game (`rngcmp`: 6,601 frames,
zero differing) and whose `[6374, 6590)` window carries `UNITS=3 GUYS=1`. At
Great Lakes 6463 the Merchant `1/26`'s second figure holds `(39471, 18769)`
across the frame and its angle goes `-1153564672 → -1605566464`, its
leader's. Great Lakes **6463 → 6582**, East Indies **6739 → 7448**;
`crates/sim`'s `a_crew_figure_of_a_packing_type_pays_the_turning_stand` is
the mechanic with a non-packing control that spends nothing.

*And a fourth chain*, the crew's through `do_turn`'s recursion, is §4.14
(item 628).

*What is not established.* `Unit::detect_boat_collision`'s call is not
modelled and no capture reaches it. The set `do_turn` recurses into is read
here as "guy 0 and the trackless crew", which is the set `Guy::move`'s follow
walks; the original starts its loop at the type's `+0x304` rather than at 1,
and no capture has a unit whose guys differ between the two readings. ~~And the
bit is **derived** here rather than stored at `init_real`: neither the type
nor the piece changes under a guy, so the answer is the one `init_real` would
have written — but a piece that changes on packing (`get_unit_gpiece`'s fifth
argument) would break that, and nothing checks it.~~ Stored now, as
`Guy::flag_piece` (item 1281): an age re-pieces without `init_real`
(`docs/TECH.md`, "The piece moves with the age").

### 4.9 A moving frame asks for the walk **twice** (2026-09-03)

§4.8 already cited the call — "`Unit::move_step:304`'s `set_anim(CHAR_WALK,
0, 1)`" — and this crate never made it. It is not decoration:

```
move_step:304  (the partial step)   set_anim(this, local_2c, 0, 1)
move_step:355  (the Manhattan snap) set_anim(this, UVar17,   0, 1)
                                    set_new_location(...)
```

`local_2c` is `CHAR_WALK`, or `CHAR_ATTACKWALK` when the unit has a target
(`move_step:66`, `:99`); `UVar17` is `local_2c` unless the waypoint offsets
were both zero at the top of the function and `UnitData+0xd8 < 2`, when it is
`CHAR_DEFAULT`. Both sit **past** the two turn-in-place arms, which
`return 1` at `005fb2e1` and `005fb2b4`, so a frame spent turning makes no
such call. Then `Guy::move`'s own animation half runs later in the same frame
(§4, `Sim::guys_follow`) and asks for the walk **again**.

*Why two calls are not one.* `Guy::set_anim`'s walk arm splits on whether the
slot it resolved is the one already playing:

```
param_2 = the resolved walk slot          # speed, then the gather mask,
                                          # then `CHAR_WALK` if the packet
                                          # does not name it (§3.6)
if cur_anim == param_2:                   # the SAME slot
    if cur_time < len: keep               # :667
    else:              cur_time -= len
else:                                     # a slot CHANGE
    cur_time = cur_time * get_anim_time(cur_anim) / get_anim_time(cur_anim)
```

The rescale's two `get_anim_time` calls are both passed the *old* slot —
`cur_anim` is written after them — so a change is `cur_time · t / t` and
**keeps the clock, however far past the end it is** (`docs/audit/
2026-08-24-anim.md`). Only the same slot subtracts. So the pair is: the first
call changes the slot and keeps the clock, the second finds the slot already
playing and takes the length off.

*Where it shows.* For guy 0 it never does — a walking guy's clock is inside
its length, so both calls keep it. It is the **crew figure** whose packet is
empty (§3.6) that reads the difference, because the mirror (§5) hands it guy
0's clock, which is a driver's 27-frame length against its own 3:

| Great Lakes frame | 5570 | 5571 | 5572 |
|---|---|---|---|
| the original | `a7 4/3` (mirror) | `a8 2/3 last 1` | `a0 0/3` — wrap |
| one call only | `a7 4/3` (mirror) | `a0 0/3` — wrap | `a8 1/3` |

5571 is the frame the caravan first walks. With one call the figure came off
the mirror at 4, kept it, stepped to 5 and wrapped a frame early — an idle
roll the original does not spend, twice over for two figures, and Great Lakes'
word parted on exactly those two draws. With both, the change keeps 4, the
same-slot call takes 3 off, the step makes it 2, and the wrap falls on 5572
where the original's does.

*Established* by run73 — this map's first `DUMP_ALL` window, `[5564, 5580)` —
and it is **diff-backed**:
`rondata::diff::tests::run73_s_window_clocks_are_the_original_s` puts every
`GUY` block's `cur_anim`, `cur_time`, `end_time`, `last_time`, `gpiece`,
`stopped`, position and angle against this crate's over the window's sixteen
frames, **4,869 fields**, and none of them parts. Without the call it fails on
the window's *first* frame and on the human's units, not only the caravan:
every guy that walks carries the wrong clock. Great Lakes' word **5571 →
5573**.

*What is not established.* The snap arm's `CHAR_DEFAULT` case — both waypoint
offsets zero and `UnitData+0xd8 < 2` — is not modelled: `do_move`'s own
"already there" test takes that case a step earlier here, so the arm is
unreachable, and `+0xd8` is unread. It would be a **draw** if it were
reachable. `CHAR_ATTACKWALK` is not passed either; no capture has a unit
stepping with a target.

## 4.10 A teleported crew figure pays no arrival stand (2026-09-04)

A tracked crew figure jogs (§4.3), so the arrival test — `cur_anim == 8`,
the **slot** — normally fails for it. It does not fail forever. The moment
its leader stops, the figure is still walking; the frame its body reaches
`des` with its angle not yet on `des_angle`, `Guy::move`'s standing arm puts
it back on `CHAR_WALK` (§4, step 1's third row), and `set_anim` re-resolves
the walk from an `avg_speed` that has begun to decay. One frame later it is
settled and `stopped`, and the frame after **that** is the arrival draw.

Great Lakes' Merchant `1/24` walked that whole sequence in this crate and
the original never started it, because the original's figure is **not
walking**: `Unit::do_cast` re-seats a rare collector on the first frame of
its unpack, and the snap flag teleports every tracked figure onto its
offset (`docs/ORDERS.md` §6.9 step 2). run75's block 6145 has the figure at
`(40706, 14716)` with its driver's angle, four frames before it could have
walked there; this crate had it at `(40631, 14634)` and jogging.

So the draw at 6151 was never a bug in `set_anim` at all — it was a body in
the wrong place, and the whole of it is `crates/sim/src/collide.rs`'s
`set_new_location` no longer returning early when the point is unchanged.
Great Lakes' word 6151 → **6463** (item 210).

The numbers are worth keeping, because they say how *narrow* the slot test
is here. The figure's steady walking `avg_speed` is 28 against a base of
23 — `280 > 253`, a jog. Its last step onto `des` is a partial one, so
`last_speed` is `vector_dist(12, 13) = 18` rather than the full 31, and
`(28·3 + 18) / 4` is **25**: `250 < 253`, a walk, by three parts in a
thousand. One unit of `avg_speed` either way and the arrival test would
have failed on its own.

## 4.11 A seated scholar teaches: the idle roll's variant is an **offset** (2026-09-18)

`Guy::init_real@005db6b0`'s last statement, before it returns, is one test
and one bit:

```c
iVar6 = *(int *)(*(int *)(units[who][o] + 0x18) + 4);   // UnitTypeData +0x4
if ((iVar6 == 0x34) || (iVar6 == 0x35)) {
  guy_flags |= 0x80;                                    // GuyData +0x9a
}
```

`0x34`/`0x35` is `ObjectData::is_scholar@0046d330` inline, so **the bit is
the Scholar and nothing else**, set once per figure at birth and never
cleared.

`Guy::set_anim@005da300`'s `CHAR_DEFAULT` arm reads it. After the variant
roll has chosen `v ∈ {0, 1, 2, 3}` (§4.1) and before the packet check that
falls an absent slot back to `CHAR_DEFAULT` (`:546`), the arm is:

```c
v = param_2;                                  // the variant just chosen
if (guy_flags & 0x80) {
  if (units[who][o]->inside_up != -1) {       // UnitData +0x82
    host_o = ObjectData::get_inside(unit, &host_who);   // walks inside_up
    param_2 = v + 0x19;                                 //  25..28
    if (objects[host_who][host_o]->inside_down != guy->o) {   // ObjectData +0x28
      param_2 = v + 0x1d;                               //  29..32
      if (param_2 == 0x20) {
        // walk the host's inside chain; a scholar already on 0x20
        // takes this one back to 0x1d
      }
    }
  }
}
```

So **the variant is not a slot for a scholar inside something — it is an
offset**, and the four idle variants become the four `Scholar Teach` files.
`SCHOLAR-DEFAULT-AGE0`'s `unit_graphics.xml` entry names them
`CHAR_CHOP_WOOD`..`CHAR_WALK_TO_WOOD`, slots 25–28, `Scholar Teach1`
through `Scholar Teach4`, and `rondata::artdata` reads them **30, 100, 103
and 105** frames long against `CHAR_IDLE1`'s 232. Slots 29–32 are the
second set.

`ObjectData::get_inside@00651a80` walks `inside_up` while the thing above is
a unit and stops at the container, so the host is the **building** whatever
the chain's depth. `Object::insert_inside@00647e90` writes each new object
into `inside_bottom`'s `inside_down`, so the chain is appended at the bottom
and its head is the **first** unit to have entered: the head takes the
`+0x19` set and everyone under it the `+0x1d` set. A teacher and its
students, as the animation names say.

### And `ObjectData::is_peasant` is a type, not a job

`ObjectData::is_peasant@0046d310`'s whole body is `UnitTypeData +0x4 in
{0x32, 0x33}` — the two **Citizen** types, which is `rondata::load`'s
[`Worker::Citizen`] exactly. This crate read it as *any* worker, so a
Scholar answered it `1` and took §4.1's peasant-on-a-masked-tile collapse
to `IDLE1`. It stands on its university's own tile, whose mask carries `&
3`, so the collapse always fired.

### The two together are Great Lakes 8374

The map's first scholar `1/44` is seated on 8272 (§`docs/CITIES.md` §6.5.2).
The seating's roll draws **787**, `787 % 100 = 87`, which is `83..95` →
variant **2**; the collapse made it 1. It heads its university's chain, so
the slot is `2 + 0x19` = **27**, whose length on `SCHOLAR`'s piece is
**103** — and 8272 + 103 − 1 is exactly 8374, the frame the original spends
a `Guy::set_anim+0x97a < Guy::inc_time+0x271 < Unit::inc_time+0x3e` wrap
this crate did not. With the collapse but without the offset the slot is
`IDLE1`, 232 frames; with the offset but without the variant fix it is slot
26, 100 frames and 8371. Only both give 8374. Great Lakes **8374 → 8382**.

**The value diff is the other map's.** Nothing on Great Lakes dumps a guy
clock after 8043, but run98 dumps East Indies over `[7879, 8788]` and that
window spans **8466**, its own first scholar's seating. `1/22`'s `GUY`
block reads `cur_anim 25, cur_time 1, end_time 30` on the frame it sits
down — slot 25 is `variant 0 + 0x19` — and this crate now reads the same
clock on every frame of the window
(`rondata::diff`, `run98_s_window_clocks_are_the_original_s`, 84,888
fields below the word). The offset is therefore read off the original's own
dump and not inferred from one arithmetic coincidence.

**What this does not establish.**

- ~~**The `+0x1d` set's tie-break.** `param_2 == 0x20` walks the host's chain
  looking for a scholar already playing slot `0x20` and takes `0x1d`
  instead; the decompiler's aliasing does not settle whether the walk
  starts at the head or at the head's successor, and this crate tests
  "any other scholar in the chain". It fires only for variant 3 on a
  non-head scholar. The falsifier is a `GUYS` window over a university
  holding **two or more** scholars: run98's holds one.~~
  **Settled by §4.12** (item 497, 2026-09-22): the walk starts at the
  host's own `inside_down` and so examines **every** member including the
  guy asking, "any other scholar" was the wrong half of the reading, and
  run100's two chains are the falsifier the bullet asked for. Great Lakes
  **10303 → 10582**.
- **The chain's head, for a scholar inside a *unit*.** `inside_up` points
  at a boat as readily as at a building (`docs/TRANSPORT.md` §6); this
  crate keeps a boat's passengers as a filter rather than in entry order,
  so it answers "not the head" there. No capture has a scholar aboard
  anything.
- **`guy_flags & 0x80` survives a type change.** `Guy::init_real` sets it
  from the type the figure was *born* with; this crate reads the unit's
  current `Worker`. `Unit::set_type` re-runs `Guy::init_real` for the
  figures it grows (§3.5) but the existing ones keep their bit, so a
  scholar upgraded into something else would still take the offset in the
  original and not here. Nothing upgrades a scholar in any capture.

## 4.12 The `0x20` tie-break counts the guy asking (item 497, 2026-09-22)

§4.11 left one line of `Guy::set_anim@005da300` unsettled and named the
capture that would settle it. run100 is that capture, it has been on disk
since the day it was taken, and the answer was thirty blocks under a word
nobody had looked there for.

### 4.12.1 What the walk actually visits

The arm, at `set_anim:332`, in the decompiler's own registers — `iVar15`
is an object index times four and `pGVar22` the `objects[who]` base, so
the pair together is one object and the loop rewrites both:

```c
sVar2 = host->inside_down;                          // +0x28
if (sVar2 != guy->o && (param_2 = v + 0x1d, param_2 == 0x20)) {
  while (sVar2 != -1) {
    cur  = objects[who][o];                         // the pair, this round
    o    = cur->inside_down;                        // +0x28 — the next member
    who  = cur->inside_down_who;                    // +0x3e
    if (objects[who][o]->vtbl[0x18]()                       // it is a unit
        && objects[who][o]->type->+4 in {0x34, 0x35}        // it is a scholar
        && units[who][o]->guys[0].cur_anim == 0x20) {       // +0xf4 → [0] → +0x9c
      param_2 = 0x1d; break;
    }
    sVar2 = objects[who][o]->inside_down;
  }
}
```

The pair entering the loop is the **host** — the building, not the first
scholar — so the first object examined is `host->inside_down`, the chain's
head, and each round afterwards examines the member below the last. The
walk therefore visits every member of the chain from the head to the tail,
and the guy that asked is one of them: the arm is only reached when
`host->inside_down != guy->o`, which is exactly the statement that it is
somewhere below the head.

**And its `cur_anim` is still the slot it is leaving.** `set_anim` writes
the new slot a hundred lines further down, past the packet check; the walk
reads `guys[0].cur_anim` off the live record. So the rule the arm actually
states is:

> A student re-rolling variant 3 **while already on `0x20`** always takes
> `0x1d`. The fourth student slot can only ever be entered from one of the
> other three.

This crate read the walk as "some *other* scholar in the chain", which is
the one reading under which it never fires on a chain with a single
student — and on run100 that is the only kind of chain that matters.

### 4.12.2 run100's two chains are the falsifier §4.11 asked for

§4.11 named the falsifier as "a `GUYS` window over a university holding
two or more scholars". run100 holds **ten** seated scholars in two chains,
and it has printed all ten on every block of the widening's window —
10,304 `GUY` records — since the capture was taken:

| host | chain, in `inside_down` order | slots on block 10273 |
| --- | --- | --- |
| `1/2019` | `1/44` → `1/51` → `1/56` | 25, **32**, 29 |
| `1/2020` | `1/45` → `1/48` → `1/49` → `1/50` → `1/52` → `1/53` → `1/55` | 25, 31, 30, 30, 31, 29, 29 |

The heads take the `+0x19` set and everything under them the `+0x1d` set,
which is §4.11 confirmed on ten figures rather than the one run98 has.
What is new is `1/51`: it is the **only** member of its chain on `0x20`,
and on block **10274** its 118-frame slot runs out, `Guy::inc_time` wraps
it, and the original comes back with **29**, `cur_time 0`, `end_time 30`.

Under "some other scholar" there is nobody to find and the answer is
`0x20`. Under "every member" the walk finds the requester itself, still
showing `0x20`, and takes `0x1d`. The dump says `0x1d`. One clause.

The variant is not in question either, and that is what makes the block a
proof rather than an arithmetic coincidence. The draw streams are in
lockstep at 10274 — the word was 279 frames further on — so both sides
rolled the **same** number and this crate's arithmetic turned it into
`3 + 0x1d = 0x20`. The original had the same 3 and produced 29. Variant 0
would also produce 29, but variant 0 is not what the shared stream rolled.

### 4.12.3 Why it was worth 279 frames

`1/51`'s two clocks were all the booking named: on the word's own block,
**10304**, `g.cur_time` ours 30 against 0 and `g.last_time` ours 29
against −1. Both are consequences, and the sign is the tell — `cur_time 0,
last_time −1` is `set_anim`'s own write on a wrap, so the original wrapped
on 10304 and this crate did not. It could not: on `1/51`'s own piece the
second teach set's fourth slot is **118** frames and its first is **30**
— the dump's own `end_time`s, not a table this crate supplies — so from
the shared start at 10274 the original's clock ran out at 10274 + 30 =
10304 exactly while this crate's had ninety left to run.

That is also the whole of the draw delta the item was booked as. The
original spends four draws on 10303 and this crate three; the extra at
index 3 is `Guy::set_anim+0x97a < Guy::inc_time+0x271`, which is the wrap
calling the roll. Neither the clock nor `inc_time` was ever wrong.

**What one clause moved** (measured on run53 and run100 at the same tip):

| | before | after |
| --- | --- | --- |
| Great Lakes long word, sequence / count | 10303 / 10304 | **10582** / **10583** |
| run100 widening over `[9340, 10316]` | 352 | **253** |
| blocks 10274 and 10304, keys parted | 2 / 2 | **0** / **0** |
| Great Lakes endpoint `off` / `unlinked` / `build_diverged` | 49 / 8 / 9 | **55** / **5** / **10** |
| East Indies endpoint `off` / `unlinked` / `build_diverged` | 63 / 11 / 28 | **59** / **12** / **30** |
| East Indies ladder B `off` / `extra` | 48 / 11 | **45** / **6** |
| scholar seatings below Great Lakes' word | 10 | **11** |
| market draws below Great Lakes' word | 10 | **11** |

East Indies' own word does not move (9711 either side) and neither does
the golden record's; the endpoints move on both maps because a chain that
entered the wrong teach slot held its figure three wraps' worth of draws
out of the stream, and every map with a university has one. Those numbers
are 13,400 frames past their words, where the stream is nobody's
(`docs/DECISIONS.md` 36).

### 4.12.4 How it was established, and what it has not

**Diff-backed**, and the assertions are where the claim lives:

- `rondata::diff`'s `run100_s_word_block_is_every_record_the_dump_carries`
  pins blocks **10274** and **10304** empty, beside item 494's 10294 and
  10295. 10274 is the cause and 10304 the symptom, and pinning both is
  deliberate: a tie-break that stopped counting the requester would fail
  on the cause, not thirty blocks downstream of it.
- `sim::anim::tests::the_scholar_s_0x20_tie_break_counts_the_guy_asking`
  asserts the four variants on a student already on `0x20`, the same
  student on another slot (where `0x20` still stands), a third member
  taking it away again, and the head's `+0x19` set. **Made to fail on
  purpose** with `c != u` back in the predicate: the first assertion reads
  **32** where the dump reads **29**, which is run100's `1/51` on 10274 to
  the digit.
- The same widening carries every other `GUY` field of all ten scholars
  across 1,257 blocks, so a chain that took the right slot for the wrong
  reason would part somewhere in `cur_time`, `end_time` or `gpiece`.

**This was a reading-only claim that a run has now closed** — §4.11 said
so itself, in the sentence that named the falsifier — and the run that
closed it needed no capture: run100 had printed the evidence on every
block since it was taken.

**What is not established:**

- **The chain's head, for a scholar inside a *unit*.** Unchanged from
  §4.11: `inside_up` points at a boat as readily as at a building, this
  crate keeps a boat's passengers as a filter rather than in entry order,
  and no capture has a scholar aboard anything. What this item did change
  is that the tie-break now fires there too, on the same "every member"
  rule; before, `scholar_student_slot` read `units[u].inside`, which is
  `None` for a passenger, and so could not fire at all.
- **Whether the break's *position* in the chain matters.** The original
  stops at the first member it finds on `0x20` and this crate asks
  `any()`. The two differ only if a later member could change the answer,
  and the answer is a single `0x1d`, so they cannot — but the loop's order
  is asserted nowhere and a future arm that returned the member rather
  than the slot would need it.
- **`guy_flags & 4` is still unobserved on this disk.** All ten seated
  scholars carry **144** on all 10,304 of the window's records — `0x80`,
  the scholar bit, plus `0x10` — so this window adds no sighting of 0x4
  to §9's list. `unit_masks2` is **0** on `1/51` throughout, which rules
  out `inc_time`'s zero-step arm here from the dump's side as well as
  from the sign of the draw.

## 4.13 An attack slot the packet lacks plays `CHAR_ATTACK2` (item 549, 2026-09-22)

Golden chapter five's word stood at **739** (`docs/GOLDEN.md` §9). On 739
this crate spent `Guy::set_anim+0x97a < Guy::inc_time+0x1ed`, 7 draws
against 6. No record parted on 739 or 740; the first dumped difference was
`1/6 ammo[0]`, which the dump held alone on 742.

**The site.** `Guy::inc_time+0x1ed` is the attack-category arm of §5's
wrap loop: a guy whose slot is in category 12 and whose `cur_time` has
reached `end_time` asks for `set_anim(CHAR_DEFAULT, 0, 0)`. That lands on
§4's idle roll, `+0x97a`. Its inputs are the category of `cur_anim`, the
step, and `end_time`. `set_anim`'s tail writes `end_time`: the packet's
frame count for the slot, or **3** when the packet has no id for it
(§3.2's [`anim::MISSING`]). This crate's `1/6` was in `CHAR_ATTACK3`
(`anim 13`) with `end_time 3`. It rolled on 737 and ran out on 739.

**The reading, and what would have killed it.** The Trireme's
`unit_graphics.xml` packet names `CHAR_ATTACK1` and `CHAR_ATTACK2`, both
`Trireme Attack1`, and no `CHAR_ATTACK3`. Three things could have killed
the reading that the original never plays a missing attack slot. The
original's round on 742 could have been off a full-length swing's release
frame: it is 737 + 5, `Trireme Attack1`'s first `<RELEASEEVENT>` at
`400 / 67` (`docs/COMBAT.md` §50.1). The fix could have left a draw parted
on 739, which would mean the slot mattered for more than its length. Or it
could have moved a record under the old word. None of the three happened.

**The rule** is the listing's, at `0x5db25f`–`0x5db279`, after the roll:

```text
5db262  push esi                    ; the slot: rolled, or the request when p3 == 0
5db263  call AnimationPacket::get_animobj@00918be0
5db274  mov  ecx, 0xc
5db279  cmove edx, ecx              ; no AnimObj → CHAR_ATTACK2
```

`get_animobj(slot)` is null when the slot is at or past the packet's
count, its id is negative, or the id is at or past `animmgr`'s. Those are
§5's index tests, which [`Sim::packet_has`] already carries. The fallback
covers the request as well as the roll. A `p3 == 0` request for a missing
attack slot also plays `CHAR_ATTACK2`. The middle band's `ATTACK2` is the
*category* `set_anim` wrote into its second argument's stack slot at
`0x5da637`, not the caller's slot. This crate already had it that way.

The two other arms already had their fallbacks: the idle roll's to
`CHAR_DEFAULT` (`:546–554`) and the walk's to `CHAR_WALK` (`:596`). The
attack arm had none until now. `Sim::guy_set_anim`'s attack arm is the
rule, and `a_rolled_attack_the_packet_lacks_plays_attack2` pins it. The
test failed first without the fallback, with `(ATTACK3, 3)`.

**What moved.** The word went **739 → 900**, run127's trace end. No draw
parts on any frame of chapter five. The widening's map went from 224 keys
to 30. Those are the 29 standing rows on the capture's first blocks
(`form`, `build:extra`, the gather slots), plus `1/0`'s explore-order
`facing` on 847. That field is declared non-scoring
(`OrderMismatch::scores`, parked 275), and Great Lakes' `1/0` carries the
same row on 8002. **The value diff on 742**: `1/6 ammo[0]` is in both airs,
`sx, sy` (12445, 35807), `ex, ey` (11673, 34704), `total_time` 13, `sz`
88, `v1z` 61.399521. All 506 of run127's live rounds agree on every
compared field. With the fallback removed the word falls back to 739.

**How it was established.** Listing-backed (the `cmove`) and diff-backed
(the word, the 742 round and the 506 rounds). The length is art this crate
already reads: `Art::piece_lengths` names the packet's slots, so no new
data entered the sim.

**What is not established.**

- **A unit whose packet lacks `CHAR_ATTACK2` too.** It gets `0xc`, which
  the tail then finds missing, so `end_time` is 3. No shipped attacker is
  known to be in that state, and no capture reaches one.
- **An unknown piece.** A piece neither the install nor a dump describes
  reads as lacking every slot, so its attack roll now plays `CHAR_ATTACK2`
  with an [`anim::UNKNOWN`] length. That is the idle arm's precedent. It
  matters only for a unit whose piece this crate cannot name, and §3.4
  says that no longer happens for a player's unit.
- **Which attackers this reaches.** Every packet that names fewer than
  three attack slots. The Trireme is the first on this disk. No census
  of the others has been taken.

## 4.14 The crew's turn is a fourth chain: `do_turn`'s recursion (item 628, 2026-09-23)

A trackless crew figure's turn draw is `do_turn`'s from inside `do_turn`: `:38` recurses
from `+0xe5`, so the chain is `set_anim+0x97a < do_turn+0x4a < do_turn+0xe5`,
whichever of the three rows above turned guy 0. `rondata::trace`'s table had
no entry for it and printed the original's draw as a bare `5dac7a`, while this
crate labelled every guy of the turn with guy 0's site. The count agreed and
the label could not. Golden chapter seven-b's fur trapper `1/10` turning on
672 is the first record of it (run156: guy 0 under `move_step+0x389`, the
crew under `do_turn+0xe5`). `crate::anim::SITE_TURN_CREW` names it on both
sides, and `Sim::do_turn_anim` marks it for each crew figure and restores the
caller's label after. Only labels change, never a draw. run156's sequence
parting moves 672 → 1148; every other capture is unchanged.

## 5. `Guy::inc_time@005d9e10` — the step and the wrap

Phase 7 of the frame (`docs/SYNC.md` §2): `Objects::inc_time` walks leaders
0–9 **in order, no rotation**, each leader's live units in object order (then
its buildings), calling `Unit::inc_time@00610b40` → `Guy::inc_time` for guys
`0..guy_mark` and `squad_size..num_guys`. `Unit::inc_time` runs only while
`inside_up < 0` (a garrisoned unit's clocks stop — `UnitData+0x82` in the
PDB; the second reading read `o_up`, which is `+0x8e`, and the dump refutes
it: the sheep step with `up 1`, woodcutter `0/2` stands with `up −1`) or
for a **scholar** (`TypeIndex` 52/53), whose teach/student slots play
inside the university. Per guy:

```
step = 1  (2 under guy_flags & 4 with an ATTACK2 playing; 0 while unit_masks2 & 0x10)
if guy_num < squad_size or cat(cur_anim) == WALK:
    last_time = cur_time; cur_time += step
    while cur_time >= end_time:
        if loopings[action(cur_anim)]:  set_anim(cur_anim, 0, 1)        # restart
        elif cat != ATTACK2:            set_anim(cur_anim == 10 ? WALK : DEFAULT, 0, 1)
        else:                           set_anim(DEFAULT, 0, 0) + the queued attack
else:
    cur_anim = guys[0].cur_anim; cur_time = guys[0].cur_time            # the mirror
```

**Neither of the two step modifiers is implemented** — `Sim::guy_inc_time`
has `let step = 1u32;` — and that is not a detail (item 496, 2026-09-22).
`unit_masks2` is not a field of any struct in `crates/sim` at all, so the
zero arm is unrepresentable rather than merely unwritten, and it is the
whole of the golden record's **chapter-two word at 695**: this crate's
three bowmen end an attack animation together and the original ends two,
because the original's `0/6` does not step its clock. run118 — chapter two
re-taken at `GUYS=4` — prints the step itself on that block, `last_time 29`
against `cur_time 29` where every other block of the window has
`last = cur − 1`, and run112's `OBJECT` block carries `unit_masks2 16` on
the same unit on the same block. `docs/COMBAT.md` §43.

**The state that claim was in is itself the finding**, and §9.1 already
said so without anyone reading it that way: "the test is green because it
asserts the original's rule, not this crate's behaviour". For seventeen
days the document was right, the assertion was green, the dump carried the
field — and *nothing was looking at the difference*, because
`rondata::diff` compares no animation clock on a frame a unit is in melee.
A rule can be **corpus-backed on the original** and **reading-only against
this crate** at the same time, and it is the second half that decides
whether a divergence can be seen. That is a coverage fact rather than an
implementation backlog, and `docs/audit/README.md` wants claims in that
state named rather than counted as covered.

**Where the bit has and has not been seen.** Six unit-frames of run112
carry it, the first of them chapter two's word; run17 has 35 and run44
26, all stepping zero. On **Great Lakes it is zero across run100's whole
window** — measured on `1/51` by the lane on item 497, which ruled the arm
out of 10304 from the dump's side independently of any argument about
which side the extra draw fell on. So the freeze is a fight-frame
phenomenon of the golden record's captures and nothing on the AI headline
has reached it yet.

Three things follow.

- **A category-0 animation running out always draws**, looping or not: both
  restarts are idle requests whose clock has run out, and both take §4's
  roll. `anim_graphics.xml` lists `Peasant Default`, `Peasant Idle1–3`,
  `Scout0 Default/Idle1–3` and the dog's under `<LOOPING>`; it changes
  nothing. Run13's sim-frame 100: twelve fish on piece 60073 (length 101)
  reach 101 together in `o` order (4, 7, …, 37) and roll `2,0,0,1,0,0,0,0,0,
  0,0,0` from `0x60032f25`, which are the dump's new `cur_anim`s, each with
  `cur_time 0` and `last_time −1`. Pinned: `anim::tests::run13_s_fish_wrap_
  together`.
- **A walk or a work animation running out is silent — unless the packet
  does not name it.** The flag `Guy::inc_time` reads is `loopings[packet->
  ids[slot]]`, and both of the indices in front of it are tested first, so a
  slot the piece's packet lacks is non-looping whatever its name says: that
  is the whole of §3.6, and it is what makes a crew figure's walk pay a roll
  every third frame. Where the packet *does* name it: `Man Walk`, `Farmer
  Sow`, `Farmer Reap`, `Lumberjack Chop3/Carry`, `Miner Dig3/Walk` are
  looping and restart through the walk branch or the same-category apply,
  neither of which draws; a non-looping one (`Lumberjack Dump`, `Miner
  Dump`, the attacks, the deaths, pack/unpack) falls to `DEFAULT` and draws
  — `anim::non_looping` is the rule by slot, from the XML's names (§9), and
  `Sim::packet_has` is the pair of index tests in front of it. **A
  scholar is the exception to the whole bullet: its teach slot is
  looping, the restart is taken, and `set_anim` turns that restart into
  an idle request anyway, because for a scholar every slot from `0x19`
  up *is* the idle category — §5.1.**
- **The mirror.** A member past the squad's size that is not walking copies
  guy 0's slot and time and never steps or draws itself: the scout's dog.
  Run13's sim-frame 101: the human scout's two guys go `60/61 → 0/61` on
  **one** draw, the dog's `last_time` stays −1 throughout. Pinned:
  `anim::tests::the_mirror`. The dog's `end_time` is its own, from its own
  `set_anim`s — **and that settles that the dog rolls on the unit's idle
  request** (the second reading's inference): run12's `0/0#1` ends frame 0
  at `end_time 61` and `1/0#1` at 41, not the 0 `init_real` left, and only
  `set_anim`'s tail writes `end_time`, past a roll that nothing gated at
  frame 0. So `Unit::set_anim` costs one draw per figure, the dog's result
  is overwritten by the mirror the same frame, and frame 0's draw 21 is the
  human scout's dog (`docs/SYNC.md` §4). The consequence by the code: a
  dog whose own `end_time` is shorter than guy 0's running idle finds its
  mirrored `cur_time` past it and re-rolls on every idle frame. ~~Unobserved
  so far~~ — **observed, run33's frames 346–350** (2026-08-29): the human
  scout's guy 0 takes `CHAR_IDLE1` on frame 284, 76 frames long, and from
  the frame its mirrored `cur_time` passes the dog's own 61 the dog spends
  one `Unit::do_idle+0x7d` roll a frame until a variant long enough comes
  up — five of them, and then nothing. The simulation reproduces the five
  once the lengths come from the install (§3.2).

~~The four woodcutters' frame 0 on run12 — their draws at 36, 37, 46, 47 are
unit-phase stands (§4's camp stand; a same-slot `set_anim` leaves `cur_time
0`, which a phase-7 wrap could not), yet their clocks show `0/232, last −1`
at the frame's end while the farmers beside them show `1/47, last 0`: phase
7 skipped them that one frame. Neither gate fits: their `inside_up` is −1
and `unit_masks2` 0 at every pass, their object flags 1. Open (§9); it
moves those four wraps by one frame.~~

**Wrong, and closed 2026-08-26 (`docs/SYNC.md` §6).** They are **wraps**,
and nothing skipped phase 7. The camp stand this attributed them to does
not exist in the original — `do_non_flat_gather`'s camp-arrival branch is
a two-way `CHAR_DUMP_WOOD` / `CHAR_DUMP_ORE` and no third `set_anim` — so
run12's frame 0 has no `do_non_flat_gather` draw at all, and neither does
run20's. The reasoning that ruled a wrap out was the mistake: a wrap
whose roll returns the slot already running takes the "same animation"
apply, which leaves `cur_time` where it was rather than at 0, so
`0/232, last −1` is a wrap that *changed* slot and `1/47, last 0` an
ordinary step. `rondata::diff`'s whole-frame check reproduces all four in
place on both traced maps.

## 5.1 A scholar's every slot from `0x19` up is `CHAR_DEFAULT` (2026-09-18)

`Guy::set_anim`'s own remap, four instructions the decompiler renders as two
unrelated `param_2`s. The listing at `0x5da5ef`:

```text
5da602  local_20 = UnitAnimCat[cur_anim]      ; the CURRENT category
5da613  cx = guy_flags & 0x80                 ; the scholar bit (§4.11)
5da617  je   ..                               ; not a scholar: keep it
5da619  cmp  cur_anim, 0x19
5da624  cmovge local_20, 0                    ; a slot 25+ IS CHAR_DEFAULT
5da62b  edx = UnitAnimCat[param_1]            ; the REQUEST's category
5da632  test edi, edi                         ; edi is param_2, from 5da329
5da63a  jne  ..                               ; a forced call keeps it
5da643  cmp  param_1, 0x19
5da646  cmovge edx, 0                         ; so does the request
5da649  [ebp+0xc] = edx
5da64c  cmp  local_20, edx                    ; the same-category early return
5da709  test edx, edx                         ; and the idle arm is entered on edx
```

`edi` is **`param_2`**, loaded at the prologue's `0x5da329`, and not `param_3`:
the same register gates the same-category early return, which is `!force` in
this crate. So the *request's* category is remapped on every unforced call —
which every one of `Guy::inc_time`'s wrap calls is, since they pass
`set_anim(slot, 0, 1)`.

**Two consequences, and each was a map's word.**

- **A wrap of a teach slot is an idle request.** `Guy::inc_time`'s looping
  arm calls `set_anim(cur_anim, 0, 1)`; the remap makes `edx` zero, and
  `0x5da709` then enters the `CHAR_DEFAULT` arm — which **draws** at
  `+0x97a` and rolls a fresh variant. Nothing about the looping flag is
  involved: `loopings[Scholar Teach1]` really is 1. The `<LOOPING>` section
  says so, and `AnimMgr::force_load@0053ade0`'s last-key rule (§3.3) is the
  independent witness — slot 27 is 103 frames under a 1 and 102 under a 0,
  and `1/44`'s `GUY` block on run97's block 8273 reads `end_time 103`.
- **The variant bands apply to it**, because `local_20` is `CHAR_DEFAULT`
  too and `set_anim:281`'s gate is `local_20 == CHAR_DEFAULT && param_3 != 0`.
  A crate reading the raw category 25 discards the roll and lands on
  `0 + 0x19` every time.

**What made it visible was a value diff, not the reading.** run97 is Great
Lakes' first value window — `[8029, 9348]` in sim-frames, 1,320 blocks with
`GUYS` detail — and `1/44`'s slot across it goes

```
27 → 25 → 25 → 28 → 27 → 25 → 25 → 25 → 27 → 25 → 25 … 25 → 26 → 26
```

at 8272, 8374, 8404, 8434, 8539, 8642, 8672, 8702, 8732, 8835, … , 9165,
9265. A `set_anim(same, 0, 1)` restart cannot change a slot at all, so the
restart had to be producing a roll. The wrap cadence is the slot's own
length — 30, 100, 103, 105 frames for `Scholar Teach1..4` — which is the
arithmetic §4.11 used at 8374 and now holds twenty-two times over.

**What it moved.** Great Lakes **8404 → 8582** and East Indies **8495 →
9711**; 8404 and 8495 are each map's seated scholar's *first* wrap
(8374 + 30 and 8466 + 30 − 1). This is the second item running whose one
change moved both maps, and again neither map's brief named the other.

**The value diff.** `run97_s_window_clocks_are_the_original_s` compares
**127,324** clock fields below the word — the first frame-by-frame reading
of run97 by anything — and `run98_s_window_clocks_are_the_original_s`
**126,508**, its whole window now below East Indies' word. Both hold every
`cur_anim`, `cur_time`, `end_time` and `last_time` of every player figure.

**What this does not establish.**

- **The remap's upper bound.** `cmovge` fires for every slot from `0x19` to
  37, so a scholar asked for `CHAR_BUILD` or `CHAR_SOW` on an unforced call
  would idle instead. Nothing asks a scholar for either, and no capture
  could refuse it.
- **A *forced* request for a teach slot.** `0x5da63a` leaves the request's
  category alone when `param_2 != 0`; `local_20` is still remapped, so such
  a call would take the non-idle arms with an idle `local_20`. No call site
  in the traced games passes one.
- **Whether the bit survives a type change** — §4.11's third seam, unchanged:
  this crate reads the unit's current `Worker`, the original the type the
  figure was born with.
- **The walk-slot residue this now uncovers.** run97's 127,324 fields part
  on **4,615** of them, all from block 8443 on, all on player 1's army, and
  every `cur_anim` among them is one of `SLOG`/`WALK`/`JOG` disagreeing with
  another — §4.3's speed test, which costs no draw because the three share a
  category. It is counted and floored by the test rather than listed, and it
  is the successor item: the falsifier is run97 itself, block 8443, `1/27`.

## 5.2 The crew swings with its leader (item 602, 2026-09-23)

§5's sketch has two arms, and the listing has a third. At the foot of
`Guy::inc_time@005d9e10`, past the wrap loop and the queued attack, a
figure with `guy_num == 0` whose slot is now in the attack category
(`UnitAnimCat[cur_anim] == CHAR_ATTACK2`) walks its unit's crew, from
`squad_size` (`UnitType +0x304`) to the guy count (`UnitData +0xe8`). For
each crew figure it:

- calls `Guy::set_new_location(crew, crew.des, 1)`: the body onto its own
  destination, `z` from the ground, `last_x/y/z` with it, and **not** the
  facing (only guy 0's own crew loop turns a crew);
- copies figure 0's `cur_anim` and `cur_time` into it (and clears the
  renderer's `+0xd0` when the slot changes).

The crew's own mirror (§5's `else` arm) copies only a
crew figure that is not walking. So the two differ exactly when figure 0
swings while its crew is on a walk slot, and golden chapter three's
restage is that case. On 639 the chariot `0/9`'s figure 0 deferred its
swing for its turn (§6.2) while its crew rolled at once; the crew's mirror
then put the crew on figure 0's slog. When figure 0 paid the swing, this
crate left the crew walking, stepping its own clock. On 664, `fight`'s
next swing found it not attacking and it rolled a fresh one
(`Unit::set_anim+0xb6`) where the original's crew, copied into the
attack, queued it. `Sim::guy_inc_time` now has the arm, and run146's word
moves **664 → 780** with it.

**Diff-backed**: run146's draw stream, draw for draw, from 639 to 780,
through every chariot swing of the restage, crew rolls included. The
crew's position after the snap is `GUYS=2`'s `g.x`/`g.y`, compared on
every block by the chapter's widening and quiet to 780. **Not
established**: the `+0xd0` clear (a `GuyOut` field, not sim state), and a
crew of more than one figure, which no capture reaches.

## 6. What the sim does with it

`crates/sim/src/anim.rs`, in the order the frame reaches it:

- **`Sim::guy_set_anim`** — §4 whole, minus the boat-crew offsets and the
  squad's group-idle synchronisation (a one-guy unit has no partner). The
  draw is `rng.roll() % 100`; ~~the `openlist` gate is always open (the sim
  keeps no suspended search, `docs/PATHFINDER.md`)~~ the `openlist` gate
  reads the unit's suspended search (§14).
- **The lengths.** `Sim::slot_length` answers "how many frames" and
  `Sim::packet_has` answers "does the packet name this slot at all" — two
  questions the crate ran together until 2026-08-29, when the install's
  table made them differ (§3.2). The first returns [`anim::MISSING`] for a
  slot a known piece omits; the second is what the three fallbacks read —
  `init_real`'s variant, the idle roll's variant, and the walk's `:596`.
- **`Sim::set_default_anim`** at the callers in §4's table: `do_idle`
  (`orders.rs`), the wood machine's camp stand, tile choice and return, and
  the frame-after-arrival in `guys_follow`. The work animations at the
  chop, the dump, the sow and the reap — and, since item 59, at
  `Sim::do_build` (`CHAR_BUILD`, or `CHAR_SOW` on a farm) and
  `Sim::do_repair` (`CHAR_REPAIR`, ahead of its gates), which is what keeps
  a worker off the arrival stand (§4.6). The walk in **`guys_follow`**, run
  from `process_movement` with the body as it stood before the follow: a
  body away from the unit starts the walk (`Guy::move:86`; `move_step:304`
  is the same frame), one standing on it since last frame goes idle if it is
  still on the plain `WALK` slot (`Guy::move:59`) — the carrying walk for a
  gather's transit legs from `gather_walk`, which reads `Unit::carry`, the
  `unit_masks & 0x78000000` nibble `do_non_flat_gather` writes (§4.4).
- **`Sim::guys_inc_time`** in the tick between `process_gaia` and the ammo,
  §5 whole: leader order, the mirror, the bounded wrap loop. ~~A unit created
  this frame is skipped (`born == frame`): the original spends a trained
  unit's first frame inside its building (`inside_up ≥ 0`).~~ **Wrong, and
  it was a compensation for the tick's loop order (2026-08-30,
  `docs/SYNC.md` §3.16): a unit created this frame is *not* skipped.** A
  trained unit is created in `Objects::process_all`'s **second** loop —
  after every unit — so the original never reaches it in that frame's unit
  loop, and `Objects::inc_time` finds its `Guy::init_real` clock at
  `cur_time 0, end_time 0` and wraps it. run13's `1/6` ends sim-frame 99 at
  `0/232, last −1`, which is what the wrap's `set_anim` leaves; its two
  draws are `Guy::init_real+0x52` and
  `Guy::set_anim+0x97a < Guy::inc_time+0x271`, and run33's trace carries
  them in that order.
- **`Sim::init_guys`** at every spawn (`Trained`, `produce`):
  `Guy::init_real@005db6b0` — `cur_time 0, end_time 0, last_time −1`, one
  draw `% 100` → `DEFAULT` below 70, then `IDLE1`, `IDLE2`, `IDLE3` by tens
  (a variant the packet lacks → `DEFAULT`). The first idle request then
  finds `end_time 0` and rolls again: the trained citizen's two draws at
  sim-frame 99 (`docs/SYNC.md` §4.1). One guy per unit until the loader
  carries `num_guys`.
- **Gaia's units.** Owners 8 and 9 are units of the sim now (`marks` has
  ten slots; the tick's loop gives them the order step and the body follow
  and nothing a player owns), and **`Sim::animal_idle`** is `Animal::do_idle`:
  the idle request, then — for a land animal with a herd — when its idle is
  on its last frame (`cur_time == end_time − 1`), one draw `% 10 < 3` to
  wander: within `0x181` of the herd centre (§7) three more draws pick a
  compass step and two magnitudes (`(k + 1) · move · 0x30` on each axis)
  and issue a move; further out `find_nearby_spot(centre, 0xc0)` issues one
  without a draw. The `detect_unit_collision` before the near move is a
  seam. ~~A herdless animal's `think_farm_animal` (every 128 frames, one
  draw `& 7` on a farm) is unread.~~ **Read, 2026-08-26: `docs/SYNC.md`
  §3.6** — a herdless animal is a *pasture's*, one of five owner-9 animals
  a `farm_type == 1` farm carries, and `do_idle` hands it to
  `think_farm_animal` with no clock gate at all. The fish never reach any
  of it: their domain is not land.

`docs/SYNC.md`'s per-frame counts move with it (its §5). Run10 with run11,
run3 and run13 as siblings (the pinned set, `diff::tests::run13_s_window_
counts_…`): **sim-frames 98, 99, 100, 102 and 103 match exactly** — 99 gains
the trained citizen's two creation draws (8/8), 100 the twelve fish wraps
(18/18) — 101 is 20/21 with the scout's wrap in and the sheep's arrival
out (the sim has no wandering sheep to arrive), 97 is 6/7 and 96 26/28 (the
AI scout's re-target draws, `think_scout`), 95 6/23 (its scan). With run12
as a sibling too, **frame 0 goes 48 → 96 of 120** (the forty animals, the
two scouts' four and the four woodcutters; the 24 left are the human scout's
scan, the AI scout's explore path and the 4-draw tail — none of them
clocks), and then to **120 of 120** with `think_scout` and the camp-stand
removal (`docs/SYNC.md` §4.2, §6), draw for draw; frame 1 holds at 54/54, frame 2 at 6/6 — and every window frame
reads one higher than above, because run12's frame-1 clocks put the AI
scout's dog under a shorter idle than guy 0's, which the sim's standing
scout then re-rolls every frame (§9, the dog). The forty first idles of
run12's frame 0 and their forty variants are pinned in `anim::tests::run12_
s_forty_first_idles`, against the LCG from draw 48 and the dump's
`cur_anim`s; the human scout's own first roll is the original's draw 20,
`p = 91 → IDLE2`.

### 6.1 Gaia is outside every object search

Established 2026-08-26, from the decompile alone, after the first fuzzed
seed panicked the harness on it (`Sim::is_enemy` with `who 8` against a
two-player diplomacy table).

`Leaders::list` is **`Leader[10]`**, `sizeof(Leader) = 0x6eec` — eight
players, then 8 and 9 for gaia's animals and gaia's birds. The
player-facing world stops at eight, and the original says so in two
independent places:

- **`ObjectsData::find_unit@0065ca80`.** Its linear branch walks the
  per-leader object lists with the cursor stepping `0x6eec` while
  `< 0x37760` — exactly `8 × 0x6eec`, so leader slots 0–7 and no more. Its
  by-cell branch reads the leader out of the cell's object chain and guards
  `if ((int)leader < 8)` before it will call `Search::valid_search`. Both
  halves stop at eight.
- **`ObjectData::valid_target_const@006472c0`.** Its **first line** is
  `if (param_1 < 0 || param_2 < 0 || 7 < param_2) return 0`, where
  `param_2` is the target's leader. This runs *before*
  `LeaderData::is_enemy`.

So no search returns a gaia unit, no attacker may target one, and
`Ammo::check_hit@00678d90` — which is a `find_unit` — cannot land a shot on
one. In RoN the animals are scenery, and this is where that is written
down. `Object::find_nearby_target@00648da0` is the case that matters most,
because it walks the world's per-cell object chains with **no leader bound
of its own**: `valid_target` is the only thing keeping gaia out of the ring
search.

**The bound is on the target, not the attacker.** `Object::valid_target`
dispatches `valid_target_const` through the *attacker's* vtable, and an
animal's — `AnimalData::valid_target_const@005d8120` — is the whole of two
lines: the target's `flags & 1`, no leader test and no diplomacy. So the
asymmetry is real in the original: nothing may target an animal, and an
animal may target anything. It does not arise here, because
`Sim::animal_idle` is the whole of an animal's behaviour and it only
wanders; the sim applies the player rules to a gaia attacker, which is a
divergence no path reaches.

**The second bound is also why the first is never tested.**
`LeaderData::diplos` is `int[8]` (`+0x74`, with `treaties` at `+0x94`), so
`is_enemy(8)` would read `treaties[0]` — the original has no answer for a
leader outside the table, and never asks for one.

In the sim: `world::PLAYER_SLOTS = 8`, which `Unit::is_gaia` is defined
against; `Sim::valid_target` carries `valid_target_const`'s first line; and
the four `find_unit`-modelled scans (`Sim::nearest_enemy_attacker` and the
chain scan beside it in `army.rs`, `check_hit` in `fight.rs`, the capture
re-test in `city.rs`) skip `is_gaia()` units. `Sim::is_enemy` and
`Sim::is_ally` are total in both arguments as well, the way
`Sim::at_war_with` already was — a guard rather than a model, since the
answer the original would give is a read past the end of an array.

**Not established.** `Build::check_capture@006276a0`'s tally walks the cell
chains with no leader bound and writes `local_8c[leader]`, a local array
that a gaia object would overrun; whether its filter (index 8 in
`Search::valid_filter`'s jump table, the block at `0067de47`, which turns
on a type field `+0x1e8`) excludes animals for another reason is unread.
The sim skips gaia there, which cannot be wrong in effect but is not
derived. *Capture:* an animal inside a capture radius during a contested
capture, with `CITIES=5` and `UNITS=3` over the window.

## 6.2 An attack is **deferred**, not played, and that is where the draw is (2026-09-18)

Item 379, from the listing and the golden record's own trace. The whole of
what `docs/QUEUE.md` called "the variant roll `set_anim` takes when the
animation is `0xc`": the roll is §4.3's, but **`Unit::fight` never spends
it** ~~for any unit~~ — for any unit that turns to shoot. A unit whose
pivot bears does not turn, and its swing rolls in `fight`'s own frame
under `Unit::set_anim+0x56` and `+0xb6` (item 595, `docs/COMBAT.md` §52;
run145's chariot on 633). All thirteen of the golden record's attack rolls, and all 196 of
run53's, come from `Guy::move` or `Guy::inc_time`.

**The address.** The attack roll is `Guy::set_anim@005da300+0xf2f`
(`005db22a`'s `Random::get(0, 0xffff)`, `% 100`, `< 30 → CHAR_ATTACK1`,
`> 70 → CHAR_ATTACK3`, else `CHAR_ATTACK2`) — a different block of the same
function from the idle roll's `+0x97a`, so a draw here is named by its own
offset first and by its caller second. Until this item the trace printed a
bare `5db22f` for it, which is a comparison that cannot fail.

**The deferral** (`005da38a`–`005da3ba`, the head of `LAB_005da36f`). Before
anything else an attack request is tested:

```text
if (UnitAnimCat[param_1] == CHAR_ATTACK2 && (guy_flags & 0x40) == 0) {
    v = param_1;
    if (des != pos || des_angle != angle) {        // still walking or turning
        if (param_3) v = 1;
        guy->+0x9e = v;  return;                   // owe it
    }
    if (UnitAnimCat[cur_anim] == CHAR_ATTACK2) {
        if (param_3) v = 1;
        guy->+0xa0 = v;  return;                   // queue it behind this one
    }
}
```

The stored byte is **`1` when the request carried its third argument and the
slot itself otherwise**, which is how the consumption knows whether to roll.
`guy_flags & 0x40` is `Guy::init_real@005db6b0:228`'s bit — a genuine plane
(`domain == 2 && !(unit_flags & 0x20)`) — and a plane is the one thing that
plays its attack where it is asked.

**`Unit::fight` always defers on the swing frame**, because two statements
above the request it writes the attack angle into **every guy's**
`des_angle` (`005fed61`'s loop, `+0x64`). So the figure is owed a turn by
construction and the roll is never `fight`'s.

**What `fight` asks for** (`005fee2d`–`005feec1`). Four arms reach
`Unit::set_anim`, and **only the last carries the third argument**, so only
the last can ever roll:

| arm | request | draws |
|---|---|---|
| `unit_flags & 0x2000000` — `z`, "Unit rocks left/right when it attacks (attack1 is left, attack2 is right)", the eighteen ship types | `CHAR_ATTACK2` when the direct angle to the target is at or past the angle being attacked on, else `CHAR_ATTACK1`; `param_3 = 0` | never |
| the **target** is `TypeIndex::PATROLBOAT` (`0x185`) | the same pair, by the target's own domain; `param_3 = 0` | never |
| `is(IMMORTALS)` (`0xa2`) within `0xc0` | `CHAR_ATTACKSPECIAL`, `param_3 = 0`, and its own per-figure `do_damage` | never |
| everything else | `CHAR_ATTACK1`, `param_3 = 1` | **on the frame the debt is paid** |

**The three payments.**

- **`Guy::move+0x166`** (`005d9381`), the *settled* arm — body on its
  destination, angle reached. `+0x9e > 1` replays that slot with no third
  argument; `+0x9e == 1` asks `set_anim(CHAR_ATTACK1, 0, 1)` and **rolls**.
  Then `set_all_pivots`, `+0x9e = 0`, `stopped = 1`. It is tested **before**
  the arrival stand, so a guy that owes an attack never pays §4's idle.
- **`Guy::move+0xe3`** (`005d92db`), the arm still owed its turn. Same two
  calls, guarded by the **two turn slots only** (`CHAR_TURN_LEFT/RIGHT`;
  `CHAR_ATTACKWALK` is *not* in this guard, unlike the walk below it) — and
  **`+0x9e` is not cleared**. The call re-enters `set_anim`, finds the angle
  still unsettled, stores the same byte back and returns without drawing, so
  the debt survives every turning frame until the angle lands. Clearing it
  here spends the debt on a call that cannot pay it, which is exactly the
  one draw the golden record is missing at 617.
- **`Guy::inc_time+0x271`** (`005da081`) and **`+0x357`** (`005da162`), the
  **queued** attack. `inc_time`'s wrap has two call sites, not one: `+0x271`
  is the shared tail every looping restart *and* every queued attack goes
  through (`CHAR_ATTACK2` as the slot it names), and `+0x1ed` (`005d9ffd`)
  is the `set_anim(CHAR_DEFAULT, 0, 0)` an attack running out takes on its
  way to the queue — **third argument zero**, so §4.2's variant bands do not
  apply to it. The in-loop payment is gated on the unit **not** being a hero
  (`unit_flags2 & 0x20`); `+0x357`, past the loop, takes any guy no longer on
  an attack and not on `CHAR_WALK`, and names `CHAR_ATTACK1`.

**And any unit-level request off an attack cancels the debt.**
`Unit::set_anim@00616f40`'s body reads **guy 0's** current slot on every
pass and writes `0` into that guy's `+0x9e` whenever the category is not
`CHAR_ATTACK2`. So a squad member that walks on the frame it swings never
plays its attack: the walk `Unit::move_step` asks for wipes it. On the
golden record's frame 616 all three of who=1's hoplites swing and two of
them move; only `1/6`, which stands still, still owes the swing at 617, and
its payment is the frame's eighth draw.

**Diff-backed**: the site names at golden 617 (`Guy::set_anim+0xf2f <
Guy::move+0x166`) and the run53 walk, which does not move either word.
**Not diff-backed**: the `PATROLBOAT` and `IMMORTALS` arms are unmodelled
(one target type, three elephant types — `k`, "Unit has melee AND ranged
attacks"), and the `z` arm's comparison is against `fight`'s own attack
angle, which the **sideways** flag (`g`, `unit_flags & 0x40`, "most ships")
offsets by a quarter turn either way; this crate models no such offset, so
the difference is always zero and a ship always rocks the one way. Neither
arm draws, so the cost of all three is which slot plays.

**No `GUY` record prints `+0x9e` or `+0xa0`**, at any detail level, so a
capture stood up mid-fight starts both at zero and may miss one deferred
swing.

**And the swing carries the shot with it** (item 389, `docs/COMBAT.md`
§9.0). This section is about which animation plays and when its variant
roll is spent; the *arrow* rides on the same deferral, one layer further
out. `Unit::fight` launches nothing for a unit: once the attack animation
is playing, `Unit::execute_events@0060edc0` → `Guy::execute_events
@005d99c0` → `GraphicEvents::execute_game_events@008e48e0` walks the
slot's own event list every frame and adds one `Ammo` for each
`<RELEASEEVENT>` frame the clock has just crossed. So §6.2's debt decides
when the swing *starts* and the event track decides when the arrow
*leaves* — on Great Lakes the two are ten frames apart, and the second was
worth thirty-six frames of the word.

## 6.3 The squad is seated around its captain, and the positions are the oracle (2026-09-18)

`Objects::init_unit@0065e0c0` does not leave a squad on one point.
Every member is born on the requested point and then moved —
`UnitType::find_nearby_spot` over the ring `[size · 0x30,
size · 0x60 + 0xc0]`, step `−1` (an eighth of the span), the bias angle the
unit's own `+0x50` (`0x55555555` at birth), `FILTER_NOT_ME` with the
member's own `o`/`who` — and then `Unit::set_new_location(spot, 1, 1)`.
`size` is the type's `+0x248`, `docs/COLLISION.md`'s `coll_size`.

**The search takes no draw**, which is why the stream never knew the
difference and item 364 could leave it open (`docs/INPUT.md` §11.5's SEAM).
The *positions* knew, and the golden record prints all six:

| line | captain | second | third |
|---|---|---|---|
| `add hoplite who=0 4,40` | `(888, 7800)` | `(1032, 7800)` | `(936, 7944)` |
| `add hoplite who=1 5,40` | `(1368, 7992)` | `(1512, 7992)` | `(1416, 8136)` |

The **second captain's** is the one that matters, and it is what makes this
a test rather than a cosmetic fix. The two `add` lines ask for points one
tile apart. With the first squad stacked on its captain the near ground
stays free and the second `add`'s own `find_nearby_spot` lands at
`(1080, 7800)`; with it spread over three points the search is pushed out to
`(1368, 7992)` — **eighteen tiles** from where a stacked crate puts it, and
the original's own answer.
`rondata::diff::golden::chapter_one_s_two_squads_are_seated_where_the_dump_says`
pins all six.

## 6.4 What §6.2 and §6.3 have not established

- **The first member's own seating.** The original runs the same block for
  member 0, whose `get_captain` is its own index, and would search a ring
  around itself with `min = size · 0x30` — which cannot return the point it
  is standing on. The dump says the captain does not move, so this crate
  takes the arm for members 1.. only and the first member's outcome is
  unread. *Falsifiable by*: a `GUYS=2` capture of an `add` whose captain's
  spot is itself crowded.
- **The cell precondition.** The move is guarded by
  `div_3_table[requested >> 4] == div_3_table[captain >> 4]` on each axis —
  the requested point and the captain's own must share a `0x30` cell. It
  cannot fail here, because this crate's callers place the captain on the
  requested point itself; a path that does not would need it.
- **Whether `+0xa0` is ever reached on either map.** The queue is modelled
  because not modelling it turns a silent return into a draw, but no capture
  on disk spends `Guy::set_anim+0xf2f` from `Guy::inc_time+0x357`, and the
  `+0x271` rolls run53 does spend are all past its word.
- **`Unit::fight`'s `param_5`.** `do_attack` passes 0 and the animation
  block runs; `do_group_attack` passes 1 and a group's own field, either of
  which skips it. This crate models no `fight` call from a group.

## 7. The animals' herd centre

`Animal::do_idle:37–39`: with the herd record `(cx, cy, wx, wy)`, the
centre it measures from is `((wx + 2·cx) · 0x300 + 0x480) / 3` on each axis
— a third of the way from the home cell's centre to the wander centre's, in
position units (`anim::herd_centre`). The herd's own walk every 64 frames is
`gaia.rs`.

**The wander centre jitters about the home cell; it does not walk**
(2026-08-31, item 112). `Herd::process@00741760` writes
`wx = cx − 1 + p % 3` and `wy = cy − 1 + p % 3`: it loads `HerdData
+0x0/+0x4` and stores into `+0x8/+0xc`, so however many times a herd is
processed its centre stays inside the nine cells around home. This crate
read the destination as the source and random-walked it, which is
indistinguishable until a herd's *second* walk — and on run33 only herd 0
gets one, on frame 832, because the cadence is `(frame >> 6) % 13`. One
cell of `wy` moves the far wander's whole ring 240 units: run33's sheep
`8/3` was sent to `(17688, 26616)` where the original sends it to
`(17688, 26328)`, a spot our ring did not contain at any of its nine
radii. Diff-backed: `docs/SYNC.md` §3.2, and
`rondata::diff::run33_s_herd_centre_jitters_about_its_home_cell` holds
`8/3`'s walk to the dump step for step and the whole capture's 74,040
animal-frames beside it. Which herd an animal belongs to is `UnitData+0x86` for an
`Animal`, which the dump does not print; the harness assigns the herd of the
animal's own type — one sheep herd on this lobby — and the fish, which are
`HERDFISH` in twelve schools, never wander.

**The far branch's bearing is a literal** (2026-08-31, item 102). Beyond
`0x181` of that centre the wander is `UnitType::find_nearby_spot(centre,
0xc0, −1, 0, …)` with no draw, and the angle its thirty-one bearings sweep
out from (§10 of `docs/ORDERS.md`) is the constant **`0x55555555`** —
`Unit::init`'s untouched-angle value, 120°, passed as the ninth argument
where `do_gather`, `do_build` and every other call site passes a real
heading. It is not the animal's facing and not the bearing to the centre.
So every far wander a herd makes fans out from the same direction, and two
animals on opposite sides of the centre try the same spots in the same
order. Diff-backed: `docs/SYNC.md` §3.19, and
`rondata::diff::a_far_wander_sweeps_from_the_literal_bearing` holds run39's
`8/3` to the dump step for step.

## 8. The harness

`rondata --diff`:

- reads every guy's clock out of a `DUMP_ALL` dump's `GUY` blocks, at the
  start (`Initial::units[].guys`) and at the end of every traced frame
  (`Initial::frame_guys`, engine frame `n − 1` under `FRAME n`, beside the
  word `frame_seeds` reads from the same block), and the lengths from all
  of it (`Initial::anim_lengths`). `ANIMALDATA` → `UNITDATA` is read
  through, so gaia's animals are in `units` now;
- **borrows** what a dump lacks from its siblings (`borrow_from_siblings`):
  the start units' clocks by `(who, o)` from any sibling on the same setup
  stream, the animals wholesale when a `UNITS`-level dump has none, the
  traced frames' clocks with their words, and every sibling's lengths;
- stands the sim up with the art (`Art::lengths`, `Art::pieces` from the
  start guys, `Art::gaia_types` from the loader, the map seed) and each
  unit's guys (`set_guy`), gaia's units included with their herd;
- and at every traced frame's end, after installing the word, **installs the
  clocks** — except on a unit that is walking on one side only (the AI
  scout: the sim has no `think_scout`), which keeps its own; the note names
  the count. A walking clock on a standing unit would read every idle
  request as an arrival, one draw a frame.

`tools/gamelog/anims.py` is the instrument: the per-pass table for named
units, `--lengths` for the table, `--wraps` for every clock reset between
two passes.

## 9. What is not established

- ~~**The woodcutters' un-stepped frame 0** (§5): which gate skipped their
  `Guy::inc_time` on run12's frame 0 and not the farmers'. `inside_up`,
  `unit_masks2 & 0x10` and the object flags are ruled out by the dump. The
  sim steps them; their first wrap (232 frames) lands one frame early.
  **Reframed by the draw-site trace (run14, `docs/ORACLE.md`):** frame 0's
  draws 110–113 are four `Guy::set_anim` calls from **`Guy::inc_time+0x271`**
  — the wrap path — before the six farm draws that end the frame. A wrap
  resets `cur_time` to 0, which is what "un-stepped" looks like in the dump;
  so the likeliest reading is that no gate skipped anything and the four
  guys *wrapped* at frame 0. What has to be reconciled is the variants
  those four draws read (`2,0,0,0` under §6's threshold mapping) against
  the woodcutters' idle slots at the end of the frame (`1,0,0,0`) — either
  the four are not all woodcutters, or the roll-to-slot mapping is not what
  §3 says for a `set_anim(CHAR_DEFAULT, 0, 1)` from `inc_time`. A `GUYS=4`
  frame-0 capture names the four; the trace has fixed where to look.~~
  **Settled 2026-08-26 without the capture** (§5, `docs/SYNC.md` §6): no
  gate skipped anything, the four are wraps, and the reason the sim did
  not make them was a stand of its own invention resetting their clocks.
  Both traced maps' frame 0 now matches the trace draw for draw, wraps in
  place.
- ~~**Whether a scout's dog draws on the unit's idle request.**~~ Settled by
  the second reading from the dump's own `end_time` (§5); the sim's reading
  stands. What is unobserved is the drawn *value* for a dog (the mirror
  hides it). ~~And the every-frame re-roll of a dog under a shorter idle.~~
  **Observed, and it is the headline** (2026-09-01): the re-roll is not a
  clock at all, it is the dog's **own body**. run56's `GUY` records carry a
  position and an angle per guy, and the scout's dog walks four frames past
  the unit's arrival on its own — so `set_anim`'s walking-guy early return
  (§4.1, `des != x − off_x`) holds it silent on exactly the frames this
  crate re-rolls it, guy 0's body being the only one the crate has.
  **Closed 2026-09-01**: the crew guy has its own body now, and the word
  moved 2665 → 3021. `docs/MOVEMENT.md`, "The follower's destination".
- **`guy_flags` bits 0x2, 0x4 and 0x20**: 0x4 doubles the attack step,
  0x20 collapses the idle roll, 0x2 skips `turn_towards`. Every guy in both
  *scored* dumps carries `guy_flags 16` — run13's 2,288 records and run38's
  1,180, every one — so the sim leaves all three off and no score turns on
  them. But ~~no writer found~~ **0x20 is exercised and written** (run44,
  `docs/ORACLE.md`, 2026-08-31): 12,582 records across five types including
  `PEASANTS`, toggling within a type — state, not a per-piece init bit; the
  writer itself is still unread. 0x2 is now a puzzle rather than an absence:
  `do_turn`'s first statement sets `|= 2` on `+0x9a` whenever the angle
  changes, and run44's 452 turn animations mean it ran with no record
  showing the bit — either the dumped `guy_flags` is not the whole `ushort`
  at `+0x9a`, or something clears it before the frame ends. And 265/266
  carry `8`/`40` **without 0x10**, which every other type has; whatever
  0x10 is, the siege pieces lack it. **Item 497 adds run100's scholars and
  no sighting**: the ten seated figures of Great Lakes' two university
  chains carry `guy_flags 144` — `0x80` plus `0x10` — on every block of
  `[9340, 10595]`, so 0x4 remains unobserved in every capture on disk
  (§4.12.4).
- ~~**`Guy::move:52` tests `des_x == x` without the formation offset** while
  `set_anim:163` tests `des_x == x − off_x`.~~ **Settled 2026-09-01 by a
  grep, not a capture**: `GuyData::off_x`/`off_y` (`+0x92`/`+0x94`) are
  written **once in the whole executable**, by `Guy::clear@005db590:49`, as
  one `undefined4` of zero. They are not the formation's offsets — those are
  `Form::off_x[]` (`docs/GROUPS.md` §6) and live on the group. So both tests
  are `des == pos` and the two lines agree by construction.
  `docs/MOVEMENT.md`, "`off_x` and `off_y` are always zero".
- ~~**Lengths the dumps have not shown**, for a **player's** unit:
  `DUMP_WOOD`, `REAP`, `FARM`, the scout's `IDLE1/3`, most citizen variants
  on most pieces.~~ **Settled 2026-08-29, §3.2**: the whole table is
  `unit_graphics.xml`'s, filed under the piece
  `GraphicPieces::init_piece_ranges`' four strides say each `<UNIT>` entry
  is. The scout's `IDLE1` is 76 frames, and its absence was item 71 — the
  roll fell back to `CHAR_DEFAULT` and the clock wrapped fifteen frames
  early. What is left open is the **age brackets**: no capture ages a
  player up, so only `AGE0`'s pieces have ever been read back, and the
  `0x840` stride is arithmetic rather than an observation. ~~And picking
  the piece is not modelled at all, so a type absent from the opening dump
  has none.~~ **Settled 2026-09-01, §3.4**: `get_unit_gpiece`'s four walks
  are modelled and checked against every dumped guy's own `gpiece`. What
  the check cannot reach is the same age bracket — every row it asserts is
  bracket 0 — and the merchant family's six "over time" pieces, which no
  capture holds.
- **The loop flags by slot** (`anim::non_looping`): from the XML's names,
  not from the packets' slot-to-file mapping. Only the dumps and the
  attacks depend on it. **And the mapping is now readable** (§3.3): the
  flag is `anim_graphics.xml`'s own section, per animation *file*, and
  `rondata::artdata` reads it for the lengths. By slot it is not a
  constant — 42 `<UNIT>` entries give `CHAR_DUMP_WOOD` a non-looping file
  and 38 give it a looping one, the same for `CHAR_DUMP_ORE` (28 / 38),
  and two entries even have a non-looping `CHAR_WALK`. So `non_looping`
  should be a per-piece table beside [`Art::piece_lengths`] rather than a
  rule; every piece a capture has reached agrees with the rule, which is
  why nothing has failed on it. **Half of it landed 2026-09-02** and it
  was not the halves the rule differs on: the two index tests *in front*
  of the flag — `slot < packet->count` and `ids[slot] >= 0` — are
  `Sim::packet_has` now, so a slot the packet does not name is non-looping
  however the rule reads it (§3.6). What is still a rule rather than a
  table is the flag itself, for the slots a packet *does* name, and the
  `CHAR_DUMP_WOOD` split above is what a table would settle.
- **A walking guy's clock.** run44's citizen counts its carrying walk
  1…14 while it moves and restarts it on the frame it arrives; this crate
  re-issues the walk from `guys_follow` every frame, so the clock sits at
  1. `Guy::move`'s moving arm does call `set_anim(UVar4, 0, 1)` every
  frame — behind a guard this crate has not read, `(type+0x2b8 & 4) == 0
  || unit_masks & 0x80000` — and a walk request never takes the
  same-category early return, so on the reading it should zero the clock
  and it does not. Nothing observable turns on it: a looping walk's wrap
  re-requests its own category and draws nothing for anyone but a bird,
  and a bird never reaches `guys_follow` (§6). It would matter to the
  first mechanic that reads a walk's `cur_time`.
- **The walk's speed ratio** is `f32` in the original; cross-multiplied
  here. ~~Unobservable while every piece's three walks share a length.~~
  **Observable since 2026-08-29** — the install gives the scout's
  `CHAR_JOG` 12 frames against `CHAR_WALK`'s 15, and the female citizen's
  11 against 15 — so a wrong side of the `0.6f`/`1.1f` boundary now costs a
  wrap on a different frame. Nothing has failed on it yet; a capture that
  fails will name the boundary.
- ~~**The group-idle gate**: `Art::group_idle` is empty until a dump shows a
  captain skipping a roll~~ — **settled 2026-08-29, §3.2, and it can never
  fire**: no `<UNIT>` entry in the shipped `unit_graphics.xml` names a
  `CHAR_GROUP_IDLE2` at all. The animals' forty first idles at frame 0
  already pointed at it — `o` 2, 18 and 34 were on the gate frame and rolled
  anyway — and the install says it outright.
- **The attack animations, the deaths, pack/unpack, the boat crews'
  offsets, ~~the turn animations~~, the squad's synchronised group idle** —
  read as far as the table in §4 and not modelled: the harness's runs have
  no fights. The turn animations' **request** is modelled since 2026-09-01
  (§4.8), because the fishing boat asks for one and pays a draw when it
  cannot play it; a guy that actually *plays* `CHAR_TURN_LEFT`/`RIGHT` still
  is not modelled, and no scored capture has one.
- ~~**The build and repair animations' lengths** are not in any dump for a
  player's citizen, so their clocks take `UNKNOWN` and never wrap.~~ The
  install has them (§3.2): both are `Construction Saw`, fourteen frames on
  the citizen pieces. A wrap of either re-requests its own category and
  draws nothing, so nothing observable turned on it either way.
- ~~**`Guy::move`'s `des_angle != angle` arm** — the walk a guy plays standing
  still while it turns, and the `field_0x218 == 1` / `SPECIAL_ANIM` exemption
  in front of it (§4.6) — is read and not modelled: this crate's `do_build`
  faces the site with the snap, so the arm's frame does not arise. It would
  for any caller that moves the heading without the facing.~~ **Modelled
  2026-08-31, §4.7**: every order in the ordinary path passes `set_angle`'s
  snap flag as **zero**, so the arm's frame arises at all of them, and the
  crate's `Movement::set_heading` is that call. What it cost was a frame of
  every work animation an order sets on the frame it turns — East Indies'
  word `1570 → 1647` with §3.3. The `SPECIAL_ANIM` half of the exemption is
  still unmodelled (`docs/ORDERS.md` §3); the sea half is.
- **`num_guys` per type**: one guy per spawned unit; the start dump's units
  carry as many as it prints.
- ~~**`think_farm_animal`**~~ — read, `docs/SYNC.md` §3.6; what is still
  open there is the animals' positions, and their lengths are the pasture
  pair §3.1 leaves out. ~~**The birds after creation**~~ — `think_bird` and
  `Guy::set_anim`'s bird arm are read and modelled (`docs/SYNC.md` §3.9);
  ~~`do_air_physics`'s **flight** is not~~ — modelled 2026-09-02, and its
  figure since 2026-09-07: **§4.6's arrival row fires for a bird**, on
  every frame its step is refused and `WorldData::restrict` hands back the
  point it stands on. `Unit::do_air_physics` ends in `set_new_location(x,
  y, 0, 1)` and the zero is `param_3`, so the figure is told where to be
  and not put there; it lags a step, and `des == pos` in `Guy::move` reads
  as *the bird did not move*. Great Lakes' word 7584 → 7585
  (`docs/SYNC.md` §3.9, "The arrival stand"; item 284). What is still
  unmodelled is `Unit::do_strafe`, so the dock's **gull** does not fly and
  its figure is left out of the follow.
- ~~**The 4-draw tail of frame 0** is still not a wrap (`docs/SYNC.md` §6).~~
  It is a wrap, at 110–113 rather than at the end (the trace, above); the
  farms are the tail.

## 9.1 The docs-versus-code pass, 2026-09-05

`docs/audit/2026-09-05-anim-vs-code.md` (Opus reader, Opus adjudication).
134 stated rules of §1–§7 traced to the code that implements them; nine
disagree, of which only R1 is adjudicated so far.

**§5's zero step is real, reached, and unimplemented — and it is now
asserted from the original's own output.** §5 states the step as `1` (`2`
under `guy_flags & 4` with an `ATTACK2` playing; **`0` while `unit_masks2 &
0x10`**). `Sim::guy_inc_time` steps by one always, and its comment accounts
only for the doubling; `unit_masks2` is not a field of any struct in
`crates/sim` at all, so the freeze is not merely unimplemented but
unrepresentable. The bit is `Unit::fight@005fd4d0`'s, set on the arm where a
unit is swinging and cleared by `Unit::process@00610bc0` — a one-frame
freeze on a unit in melee.

Nothing had to be inferred to check it. `GuyData::log_data@005de6c0` prints
`last_time` beside `cur_time`, and `last_time` is `cur_time` *before this
frame's step*, so their difference is the step the original actually took,
per figure, per frame, already on disk — the field was there all along
(`CLAUDE.md`, "grep the dump before booking a reading").
`rondata::diff::tests::the_frozen_frame_s_figures_do_not_step_their_clocks`
walks every capture that prints both:

> 26 figure-frames with `unit_masks2 & 0x10`, steps `{0: 26}`; without it
> `{1: 89522}`

Twenty-six for twenty-six at zero and 89,522 for 89,522 at one, over
`gamelog-run44-islands-turners.txt` and its four siblings;
`gamelog-run17-combat.txt` carries 35 more unit-frames of the bit, with
`GUY` blocks too short to price. Both halves are asserted, so a corpus in
which nothing steps at all cannot satisfy it.

The test is **green** because it asserts the original's rule, not this
crate's behaviour: `rondata::diff` compares no animation clock on a frame a
unit is in melee, so no diff can fail on the gap today. The arm belongs in
`guy_inc_time`; until it lands the rule cannot drift, and the day it lands
this is the check that says what it should do.

## 9.2 The other eight rows of that pass

All confirmed, none struck, and the shape is unlike the wave's other seven
documents. Only three of the nine rows are document-versus-code in the
ordinary sense: §4.2's group-idle gate names three conditions where the code
and the original both have five (the document is short, and the gate is dead
in every capture); the gaia-walker early return — `who >= 8`, `CHAR_WALK`,
inside its length — is implemented and appears in no section here, only in
the function's own doc-comment; and R1's freeze, above.

**Five are rules of the original that neither this document nor the code
carries**, which a document-only or a code-only reading could not have
surfaced: a guy on `CHAR_TURN_LEFT`/`RIGHT` with `des_angle != angle` has
its own early return and no-draw rewind — run44 has 452 such guy-frames, so
this one is *reached* and merely unmeasured; the attack roll's packet
fallback to `CHAR_ATTACK2`; `CHAR_ATTACKWALK`'s non-looping mark (left
`UNSURE:` — `anim_graphics.xml` settles it with a grep); the idle variant's
sixth fallback term, `unit_masks & 0x2000000`; the scholar's
`guy_flags & 0x80` remap of every slot above `CHAR_UNPACK`, on both sides of
the same-category test; and `set_anim`'s `CHAR_ATTACKWALK` head arm with its
`UnitData+0xae` counter — the same counter §5's freeze reads. §4.9 disclaims
*passing* that slot, not receiving it.

## 10. Second reading — landed

The blind reader's report is `docs/audit/2026-08-24-anim-reading.md`
(Opus, 2026-08-24, from the decompile, the PDB, the executable's `.rdata`,
the XML and both dumps, without this document or `anim.rs`); the
adjudication, every claim taken back to the decompile or the listing, is
`docs/audit/2026-08-24-anim.md`. It re-derived the structure identically
— the categories, the early returns, the `openlist` gate, the thresholds,
the wrap loop, the mirror, the gaia piece, the XML — and matched the
oracle 52/52 the same way. Three things changed here because of it, each
marked inline above: the walk-to-walk rescale is an identity (§4.4;
`anim.rs` no longer rescales), `guy_flags & 0x80` and the `0x34/0x35`
classes are scholars, not boats (§1, §5; the garrison gate spares them),
and the dog's own roll is settled from `end_time` rather than left to the
trace (§5, §9). One of its claims was refuted: `Unit::inc_time`'s gate is
`inside_up` (`+0x82`), not `o_up` (`+0x8e`), by the PDB and by the dump.

## 11. A converted figure stands: `init_real`'s tail on a kept guy (item 571, 2026-09-23)

`Unit::set_type@00612fa0` gives every kept guy `Guy::init_real(guy, 1)`
(`:187`, `:201`). §4.9 and `Sim::reinit_guys` modelled the clock half of
that call: the variant roll, `cur_time` and `end_time` 0, and `last_time`
−1. **The rest of `init_real@005db6b0` stands the figure still.** Past the
clock it writes:

| offset | field | value |
|---|---|---|
| `+0x9d`/`+0x9e` (one short) | `stopped`, `hold_attack` | 1, 0 |
| `+0xa0` | `queued_attack` | 0 |
| `+0x80` | `last_speed` | 0 |
| `+0x84` | `avg_speed` | 0 |

After that, the `SET_TYPE_NORMAL` tail's `Guy::set_new_location(guy, x, y,
1)` writes only position and height. So a figure converted while it walks
reads `stopped 1` and speed 0 for one block. It walks on from the next,
with its running average starting again from zero, and until it has sped
up it turns at once (MOVEMENT, "The body step").

**How it was established.** It came from run136's dump, before any
reading. On block 11922 of Great Lakes, nine player-1 figures read
`stopped 1` with both speeds 0, while their positions kept advancing on
both sides. What they share is their type: they are every figure walking
on 11921 with guy type 132, and on 11922 it reads 133. Player 1's leader
record moves on the same block: `discovered_get()` 11 → 12, and
`barracks_queued` 1 → 0. That is `Leader::gain_tech`'s conversion loop
(TECH §7), and this crate makes it on the same frame. The listing then
named the fields. `Guy::fresh`, the new-guy arm, already wrote all of them;
the kept arm did not.

**What it moved.** Great Lakes' long word, **12038 → 12135**. Without the
stand, `1/62` turned slowly on 11922 (a step of 12 against the original's
27), parted on 11923, and walked past the arrival stand against `1/64`
that the original's makes on 12038, the draw
`Guy::set_anim+0x97a < Unit::move_step+0x823`. Block 11922 goes from 29
rows parted to 2, and the nine agree from 11923 to the word.

**What is not established.**
- **`1/41`'s idle variant on 11922**: ours `IDLE1` over 64, the
  original's `IDLE2` over 48, on that block alone. The draw counts agree
  and `init_variant` is `init_real`'s ladder, so the two sides gave
  `1/41` different rolls. Which roll went to which unit is on no disk:
  run136's call records end at 11910.
- **SEAM**: `init_real`'s turret zeroing (`+0x20..+0x3c`), its
  `reset_pivots`, and its `+0x54`/`+0x58` track zeroing, which
  `set_type`'s `update_gpiece` may rewrite. No converted type on a
  measured frame has a pivot or a tracked crew.

**Coverage.** *Diff-backed*: the four fields on nine figures, block
11922 (`run136_s_word_frame_is_widened_whole`,
`run163_s_word_frame_is_widened_whole`); the word, from run53. *Guard*:
`a_figure_converted_mid_walk_stands_with_its_speeds_zeroed`, made to
fail without the reset. *Listing-backed*: the offsets, against the
decompile of `init_real` and the §1 table, which names `+0x9d` and
`+0x84`.

## 12. An animation's name is found case-folded (item 643, 2026-09-24)

A `<UNIT>`'s `<ANIM name="CHAR_…" file="…"/>` names its animation by the
`name` of an `anim_graphics.xml` row, and the two files do not always
agree on case. `GraphicPieces::decipher_animation@008face0` resolves the
name. It walks `anim_names` in load order, the four sections of
`init_anims_pool@008fca40` one after another, and returns the **first**
entry that compares equal under **`_wcsicmp`**. A name that finds nothing
returns −1, and the slot is the packet's missing slot: `MISSING`'s three
frames (§3.1).

`artdata::anim_files` used a case-sensitive map in which a later
duplicate overwrote an earlier one. It now keys on the name trimmed and
ASCII case-folded (`anim_key`), and the first row wins.

**What it changes.** 46 of the 1,894 names `unit_graphics.xml` cites
resolve only case-folded, and every one of them read as a missing slot:
`Galley Default`, `Sloop Default`, `Transport default`, `Macemen Default`,
the `longbow`, `balamob` and `bombvessel_*` sets, `Chicken Death`, `Pig
Death`, and others. The file's one duplicate name, `Caravan3 Wagon Walk`,
is byte-identical both times, so first-wins changes nothing today.

**How it was established.** From run166's dump, before any reading. On
block 13638 of East Indies, the Galley `1/32` enters `CHAR_DEFAULT` with
`end_time` 20 there and 3 here. It is a Trireme, upgraded to TypeIndex
341 / piece 291 inside the window on both sides. It wraps again on frame
13640, and that was the long word's extra draw. The two XML rows then
named the case, and the decompile named the comparison.

**What it moved.** East Indies' long word, **13640 → 15782**. run166's
floor goes 295/308/495 → 295/307/307, and nothing parts from the word to
the window's end. On the way the draw sequence parted on 15378, on a
research draw that was the same draw on both sides but unnamed in ours;
it is now `ai_research::SITE_GOV_ROLL` (and the coin `SITE_GOV_COIN`).
Great Lakes holds at 14382.

**What is not established.**
- **The other 45 names on a dump.** Only the Galley's slot is diff-backed.
  The rest follow from the same comparison and the same reader, and no
  capture has printed any of them.
- **Non-ASCII case.** `_wcsicmp` folds by locale, and this crate folds
  ASCII. No shipped name has a non-ASCII letter.

**Coverage.** *Diff-backed*: piece 291's `CHAR_DEFAULT` at 20 frames,
block 13638 (`run166_s_word_frame_is_widened_whole`), and the word, from
run54. *Guard*: `the_install_s_piece_lengths_match_the_dumps` pins piece
291's slot at 20, and it was made to fail against the case-sensitive
key. *Listing-backed*: the first-match walk and `_wcsicmp`, from the
decompile of `decipher_animation`.

## 13. The danger flag is guy 0's (item 713, 2026-09-24)

`guy_flags & 0x20` narrows §4's idle roll to `DEFAULT` or `IDLE1`, with
everything above 69 collapsing to `IDLE1`. **Only guy 0 carries it.**

- **One writer.** The `orw $0x20` onto a guy's `+0x9a` at `5fd01e` is
  the only instruction in the image that sets the bit. It sits inside
  `Unit::set_in_danger@005fcfb0`, in a loop over figures `0 .. guy_mark`
  (`+0xb5`), which is 1 on every dumped unit (§3.5).
- **The other bytes are not it.** Every other store to `+0x9a` either
  zeroes it (`Guy::clear`, `Guy::init_real`) or sets another bit
  (`init_real`'s `0x100`, `8`, `0x10`, `0x40` and `0x80`, and
  `do_turn`'s `2`). `Unit::update_speed`'s `+0x9a` is the unit's own
  `myspeed`, a different record.
- **The clear is per guy.** `Guy::process@005e0230:120` drops the bit
  when `(frame + o) % 64 == 0` and that guy's own `avg_speed` (`+0x84`)
  is 0. `Unit::process@00610bc0:540–551` runs it on every figure.

So a crew figure past the squad rolls all four variants whenever it is
asked, and a hit on its unit does not change that.

This crate kept one flag per unit (`Unit::guy_flag_0x20`), which every
figure read. It is now read for guys below `SQUAD_SIZE` only, in
`Sim::guy_set_anim`. The clear stays keyed to guy 0, because no other
guy has a flag to drop.

**How it was established.** From the draw stream and this crate's own
clocks, before any reading and with no packet (run200 was not taken).

On chapter eleven's 1139 (run190), the original spends one
`Guy::set_anim+0x97a < Unit::do_guard+0x7f4` more than this crate. Both
sides reach `do_guard`'s stand on the post:

- on tick 1139 neither sixteen-frame arm returns (`o` = 6);
- the guard is on its cell;
- its heading is its facing.

`RON_DEBUG_UNIT` put this crate's chariot at guy 0 `IDLE1` 72/80 and crew
guy 1 at 72/81, with the crew's clock mirrored and its `end_time` its own
(§5). A scratch print of the rolls gave the crew's re-rolls on ticks
1098–1100 under the mirror, with p6 and then **p91**. With the flag,
p91 is `IDLE1` at 81 frames. Without it, p91 is `IDLE2`, and piece 12817's
`IDLE2` is 71 frames. 71 runs out on block 1138, and tick 1138 is
`do_guard`'s search arm (`(6 + 1138 + 8) % 16 == 0`), which returns
before the stand. So the re-roll falls on 1139, which is the original's
draw. The guard was flagged by the hit on 1091. Guy 0's own roll on tick
1066, p97, is `IDLE1` on both sides only because guy 0 does carry the
flag.

**What it moved.** Golden chapter eleven, **1139 → 1250, closed**. Before
the fix, run190's whole-run widening parts from 1156 on (`1/4`'s move,
then `0/4` on 1172 and `1/3` on 1188). With it, no row parts past 1001.

**What is not established.**
- **`guy_mark` above 1.** Every dump prints `guy_mark 1`, and this crate
  reads the bound as `SQUAD_SIZE`.
- **The crew's own clear.** It is unobservable while no crew figure can
  hold the bit.

**Coverage.** *Diff-backed*: the crew's `IDLE2` on tick 1100, through
the re-roll on 1139 and run190's whole trace
(`chapter_eleven_holds_to_the_golden_word`,
`chapter_eleven_s_word_frame_is_widened_whole`). *Unit test*:
`a_crew_figure_rolls_its_idle_without_the_danger_flag`, made to fail by
reading the flag for every figure. *Listing-backed*: the single writer
and its loop bound, and the per-guy clear.

## 14. A unit holding a suspended search idles without a draw (item 837, 2026-09-26)

§4's idle roll draws only while the unit's `openlist` (`UnitData
+0x104`) is null. The listing at `5dac5f` is `cmpl $0x0, 0x104(%eax)`;
its `jne` goes to `5dac89`, which zeroes the roll with no call. Below
the compare, `5dac75` calls `Random::get`, whose return
address `5dac7a` is `set_anim+0x97a`, the idle roll's draw site. So a
unit that holds a suspended search takes the variant ladder with `p =
0`: `CHAR_DEFAULT`, or slot `0x19` for a scholar, and spends nothing.

`openlist` is non-null exactly while a search is suspended. `Unit::init`
and the `UnitData` constructor zero it. `astar_path`'s suspend block hands
the open list to the unit, and `Unit::clear_partial_path@005e3920` frees
it (`docs/PATHFINDER.md` §18.3, which counts the same writers). In this
crate that state is `Unit::search`, wired since 2026-09-17. This arm was
written before then, and it read the stash as always empty.

**How it was established.** From the draw stream and run257's dump,
before any reading. On East Indies' 18999 this crate spent 7 draws
against the original's 6, with five `Guy::set_anim+0x97a <
Guy::inc_time+0x271` against four (`RON_DEBUG_SITES`). Ours' five
belong to `1/19`, `1/20`, `1/29`, `1/67` and `1/69`, and run257's block
19000 shows all five wrapping on the original's side too (`cur_time 0`,
`last_time −1`). So one wrap there drew nothing. Of the five, only `1/67`
carries a suspend's stamps on that block: `start_dist 480`,
`collide_frame 18969`, `collide 31`, with a move order still at its
head. Its guy 0 reads `cur_anim 0`, `end_time 31` on both sides. The
decompile of `set_anim` named the field, and the listing named the
address.

**What it moved.** East Indies' long word, **18999 → 19182**, inside
run257. With a scratch print on the gate, it fired on one unit in
18990..19000, `1/67` on 18999. Great Lakes holds at 20568. On run257,
`1/9`'s `cur_anim` 36 → 8 on 19002 had changed on the original's side
alone, and now changes on both. The walk's floor goes 322/1/323/863 →
322/9/355/387.

**What is not established.**
- **The other arms of `set_anim`.** The attack roll (`+0xf2f`) and
  `init_real`'s variant have no `openlist` test in the export. Only the
  idle roll's gate is read here.
- **A suspended scholar.** The ladder with `p = 0` gives slot `0x19` by
  the code. No capture has one.

**Coverage.** *Diff-backed*: the draw count on 18999, from run54's
trace, and `1/67`'s stamps and clock on block 19000
(`run257_s_word_frame_is_widened_whole`). *Unit test*:
`a_suspended_search_idles_without_a_draw`, made to fail without the
gate. *Listing-backed*: the compare and its branch at `5dac5f`.

## 15. A unit that goes inside in its own work still takes its figures' frame (item 850, 2026-09-26)

`Unit::process@00610bc0` tests `inside_up < 0` (`+0x82`) once, on entry.
Under it, it runs the work, the vtable's `+0x188`, and then
`Guy::process` on every figure: first the squad (`+0xb5` figures), then
the crew from the type's `+0x304` up to `+0xe8` (lines 538–549 of the
export). **Nothing re-tests inside between the work and the figures.** A
unit whose own work puts it inside a transport or a building still has
its figures moved once on that frame. After that the inside arm (`else`)
writes only `last_z` from `z`.

This crate gave that frame to an aircraft alone. The SEAM in
`Sim::process_unit` said a squad that garrisons was owed it by the
reading, and that no capture had measured it. Now it is given to every
unit that went inside in its own work (`inside` or `inside_unit` set by
the work's end). What the frame does is `Guy::move`'s standing arm on a
body already at its `des`. It writes `last_speed` 0, ages `avg_speed`
(`(avg × 3 + 0) / 4`), and turns a figure owed a turn toward
`des_angle`: `Guy::turn_towards → do_turn`, whose `set_anim` falls back
to the idle roll, the draw (§4.8).

**How it was established.** From the draw stream, then the dump, then
the decompile. On East Indies' 19413 this crate spent 5 draws against
the original's 6, parting at index 3. Both spent `1/77`'s two
`Guy::set_anim+0x97a < do_cast` and the barge's `Guy::init_real+0x52`.
The original then spent `Guy::set_anim+0x97a < Guy::do_turn+0x4a <
Guy::turn_towards+0x69` (`RON_DEBUG_SITES`). `1/77` is a Merchant (type
61, two figures) casting its barge `1/78` at the shore (`docs/TRANSPORT.md`
§6). run269's block 19414 has its second figure standing on its `des`
(28037, 34244) with `last_speed` 0 and `angle` = `des_angle` −901447680,
where it read −541917184 and `last_speed` 1 on 19413. Both figures' `avg_speed`
go 15 → 11 and 16 → 12. Ours printed the 19413 values unchanged: its
figures had not moved at all (`RON_DEBUG_UNIT`).

**What it moved.** East Indies' long word, **19413 → 19509**, inside
run269. The four figure rows on 19414 agree. Great Lakes holds at 20568.
Chapter twenty (run249), where the Chariots `0/6` and `0/7` board their
barges on 704 and 830, loses four standing rows. `0/6`'s `avg_speed`,
20 against 15, had stood 704..1169, and now stands only on 1160..1169,
where our disembark seats the figures on a zero average and the original
brings out the 15 it froze with. `0/7`'s, 18 against 13, had stood
830..1299, and is gone. The older East Indies widenings lose keys the
same way (item 850's journal).

**What is not established.**
- **A garrison on its arrival frame.** The same gate now gives it the
  frame by the reading. No widening names a garrisoning squad's figures
  on that frame.
- **The disembark's figure bodies.** run249's `0/6` comes out of its
  barge on 1160 with `avg_speed` 15 on both figures. Ours seats them at
  0 (`Sim::seat_guys`' `Body::at`). The original's `Guy::set_new_location(…,
  1)` is not read here.

**Coverage.** *Diff-backed*: the draw count on 19413, from run54's
trace, and `1/77`'s figures on block 19414
(`run269_s_word_frame_is_widened_whole`), with chapter twenty's rows on
704 and 830 (`chapter_twenty_s_word_frame_is_widened_whole`). *Unit
test*: `a_walker_that_boards_its_boat_moves_its_figures_that_frame`,
made to fail on the aircraft-only gate. *Decompile-backed*: the entry's
single `inside_up` test and the figure loops after `+0x188`.
