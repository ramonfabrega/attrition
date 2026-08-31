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

*The check.* `the_install_s_piece_lengths_match_the_dumps` takes the 88
`(gpiece, cur_anim) → end_time` rows five dumps print over six pieces of two
nations and asserts every one against the install — which checks the
addressing, where §3.1's check checked the `.bha` arithmetic. It had teeth
on its first run: fourteen rows failed, and both families were real.

**The five that still fail are the mirror, and that is the second half of
the check.** A crew member past the squad's size copies guy 0's `cur_anim`
and keeps its **own** `end_time` (§5), so the pair a dump prints for a
scout's dog is one animation's slot beside another's length. All five are on
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
3. **The attack roll** (`:575–587`), category 12 with the third argument:
   one draw, `p < 30 → ATTACK1`, `p > 70 → ATTACK3`, else `ATTACK2`.
4. **The walk** (`:614–706`), category 8. **The slot asked for does not
   enter it**: the arm opens with the *category* — `CHAR_WALK` — as its
   answer, so every walk request is re-resolved from scratch and a
   `set_anim(CHAR_JOG)` is not a request for `CHAR_JOG`. From there:
   `SLOG` / `WALK` / `JOG` by the body's average speed against the type's
   base (`avg_speed / (moves · UNIT_MOVE_SPEED)` below `0.6f` slogs, above
   `1.1f` jogs — a float in the original, cross-multiplied here; the
   lengths are equal on every piece observed so the boundary is
   unobservable); an **owner-9** bird's is a coin instead (`% 100 > 49 →
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
   slot the packet lacks falls back to `CHAR_WALK` (`:596`).
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
| `Guy::move:59` (`+0x19f`), the frame after a walking guy stops with the plain `WALK` slot and nothing else changed it | `(DEFAULT, 0, 1)` | the arrival, when no order made the request first. **The one caller that reaches `Guy::set_anim` directly** rather than through `Unit::set_anim+0x56`, so its chain is a frame shorter and the trace's disambiguator sits at `up[0]` (`sim::anim::SITE_ARRIVE`, `docs/SYNC.md` §3.10) |
| `Guy::move:78` (`+0x14f`), the **turn arm** — a body standing on its unit whose angle has not reached `des_angle`, every frame it is still turning | `(CHAR_WALK, 0, 1)` | never itself, but it puts the guy back on the walk category, so the *next* frame's idle request rolls again. That is the second draw of an arrival pair (`docs/SYNC.md` §3.11). **Unmodelled**, and what it costs is there too |
| `Unit::move_step:281` (`+0x823`), a unit whose step is blocked, before the three give-up tests | `(DEFAULT, 0, 1)` | the same conditions as any idle request — three times on run14 (frames 122, 184, 256), and this crate takes the first two on the original's own frames (`sim::anim::SITE_BLOCKED`, `docs/COLLISION.md` §5, §8) |
| `Guy::inc_time` (§5) | the wrap | an idle running out |

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
then keep, so such a guy spends no arrival draw. `Guy::init_real@005db6b0:179`
sets the bit only for a guy whose piece names a turn animation: 273 of the
install's 1,359 unit pieces do, and **none of the eight a `DUMP_ALL` run's
guys carry** — 0, 19, 352, 371, 6336, 6688, 12691, 13043 — while gaia's
60063–60074 are not `<UNIT>` entries at all. Asserted in `rondata::diff`'s
`the_install_s_piece_lengths_match_the_dumps`, beside the `GROUP_IDLE2`
finding of §3.2, and unmodelled deliberately: the check that would make it
matter is a capture with a vehicle or a ship turning in place.

**Fired 2026-08-31** (`docs/ORACLE.md`, run44): 452 guy-frames play
`CHAR_TURN_LEFT`/`CHAR_TURN_RIGHT`, nine `(who, o, slot)` combinations,
both sides, once a capture had combat and `GUYS=4` at once — and no driver
was needed, because `move_step`'s turn-in-place branches are not the only
path: `Guy::move:109`'s standing arm hands the override through
`Guy::turn_towards@005d9720` to `do_turn`, so a turner unit turning
towards a target is enough, and a fight supplies one. The paragraph above
stays true of the *scored* games — run13's 2,288 guy records and run38's
1,180 all carry 16 — so the override stays unmodelled and moves no score;
what changed is that the row is diff-backed (`tools/gamelog/turnanim.py`
counts the 452) rather than unfalsifiable, and `guy_flags & 8` is
observed as exactly the three turner types 134, 265 and 266.

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
- **A walk or a work animation running out is silent.** `Man Walk`, `Farmer
  Sow`, `Farmer Reap`, `Lumberjack Chop3/Carry`, `Miner Dig3/Walk` are
  looping and restart through the walk branch or the same-category apply,
  neither of which draws; a non-looping one (`Lumberjack Dump`, `Miner
  Dump`, the attacks, the deaths, pack/unpack) falls to `DEFAULT` and draws
  — `anim::non_looping` is the rule by slot, from the XML's names (§9).
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

## 6. What the sim does with it

`crates/sim/src/anim.rs`, in the order the frame reaches it:

- **`Sim::guy_set_anim`** — §4 whole, minus the boat-crew offsets and the
  squad's group-idle synchronisation (a one-guy unit has no partner). The
  draw is `rng.roll() % 100`; the `openlist` gate is always open (the sim
  keeps no suspended search, `docs/PATHFINDER.md`).
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
  hides it) and the every-frame re-roll of a dog under a shorter idle.
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
  0x10 is, the siege pieces lack it.
- **`Guy::move:52` tests `des_x == x` without the formation offset** while
  `set_anim:163` tests `des_x == x − off_x`; the same for guy 0 (`off` 0)
  and for every unit in the dumps. Unsettled for a formation with offsets.
- ~~**Lengths the dumps have not shown**, for a **player's** unit:
  `DUMP_WOOD`, `REAP`, `FARM`, the scout's `IDLE1/3`, most citizen variants
  on most pieces.~~ **Settled 2026-08-29, §3.2**: the whole table is
  `unit_graphics.xml`'s, filed under the piece
  `GraphicPieces::init_piece_ranges`' four strides say each `<UNIT>` entry
  is. The scout's `IDLE1` is 76 frames, and its absence was item 71 — the
  roll fell back to `CHAR_DEFAULT` and the clock wrapped fifteen frames
  early. What is left open is the **age brackets**: no capture ages a
  player up, so only `AGE0`'s pieces have ever been read back, and the
  `0x840` stride is arithmetic rather than an observation.
- **The loop flags by slot** (`anim::non_looping`): from the XML's names,
  not from the packets' slot-to-file mapping. Only the dumps and the
  attacks depend on it.
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
  offsets, the turn animations, the squad's synchronised group idle** —
  read as far as the table in §4 and not modelled: the harness's runs have
  no fights.
- ~~**The build and repair animations' lengths** are not in any dump for a
  player's citizen, so their clocks take `UNKNOWN` and never wrap.~~ The
  install has them (§3.2): both are `Construction Saw`, fourteen frames on
  the citizen pieces. A wrap of either re-requests its own category and
  draws nothing, so nothing observable turned on it either way.
- **`Guy::move`'s `des_angle != angle` arm** — the walk a guy plays standing
  still while it turns, and the `field_0x218 == 1` / `SPECIAL_ANIM` exemption
  in front of it (§4.6) — is read and not modelled: this crate's `do_build`
  faces the site with the snap, so the arm's frame does not arise. It would
  for any caller that moves the heading without the facing.
- **`num_guys` per type**: one guy per spawned unit; the start dump's units
  carry as many as it prints.
- ~~**`think_farm_animal`**~~ — read, `docs/SYNC.md` §3.6; what is still
  open there is the animals' positions, and their lengths are the pasture
  pair §3.1 leaves out. ~~**The birds after creation**~~ — `think_bird` and
  `Guy::set_anim`'s bird arm are read and modelled (`docs/SYNC.md` §3.9);
  `do_air_physics`'s **flight** is not.
- ~~**The 4-draw tail of frame 0** is still not a wrap (`docs/SYNC.md` §6).~~
  It is a wrap, at 110–113 rather than at the end (the trace, above); the
  farms are the tail.

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
