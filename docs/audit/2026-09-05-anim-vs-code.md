# ANIM.md versus `crates/sim` — 2026-09-05

Reader: Opus, docs-versus-code pass (not a blind reading: this reader saw the
document and the implementation, and nothing else — plus the decompile export
where a row turned on what the original actually does).

Checked: 134 rules across §1–§7 (the record's fields, the category table, the
length and piece arithmetic of §3–§3.6, `set_anim`'s five phases and its
thirteen-row caller table, §4.6–§4.10, `inc_time`'s step/wrap/mirror, §6's
wiring, §6.1's gaia bounds, §7's herd arithmetic). Rows below are the ones
where the code and the document disagree, or where one of them carries a rule
of the original the other does not.

Rows R1–R3 are document-versus-code proper. Rows R4–R9 are rules the original
has that **neither** the document nor the code carries — a docs-versus-code
pass finds them because the pass reads both sides against the same function,
and they are the kind of thing that becomes a divergence the moment a capture
reaches them. Each says whether anything on disk reaches it.

## Rows

### R1 — the animation clock does not step at all while `unit_masks2 & 0x10`, and the sim always steps it

| | |
|---|---|
| document | ANIM.md §5, "`step = 1  (2 under guy_flags & 4 with an ATTACK2 playing; 0 while unit_masks2 & 0x10)`" |
| code | `crates/sim/src/anim.rs:1151` (`fn guy_inc_time`) — `let step = 1u32;` |
| document says | a unit carrying `unit_masks2 & 0x10` steps its guys' clocks by **zero** that frame: `last_time = cur_time`, `cur_time` unchanged, and no wrap |
| code does | steps by one always. The one line of comment above it (`anim.rs:1149-1150`) accounts only for the `guy_flags & 4` doubling; the zero arm is not mentioned in the code at all, and §9's "the sim leaves all three off" covers `guy_flags` bits 0x2/0x4/0x20 — not this one |
| difference shows | `GUY` `cur_time` / `last_time` of a unit in melee, on the frame it holds: the sim's clock is one higher, and any wrap it owns lands one frame early — which is a `Guy::set_anim+0x97a < Guy::inc_time+0x271` draw on the wrong frame. Nearest capture with both combat and `GUYS`: `gamelog-run44-islands-turners.txt` (the 452 turn-animation guy-frames of §4.7 are the same window), and `gamelog-run17-combat.txt` / `gamelog-run25-islands-emergency-window.txt` |
| reached | reached (`Guy::inc_time@005d9e10` and `Unit::fight@005fd4d0` are both in the 667 entered; neither is in `blind-all-68traces.txt`) |

The decompile is unambiguous. `Guy::inc_time@005d9e10`, the head:

```c
iVar10 = 1;
if (((this->field_0x9a & 4) != 0) && (UnitAnimCat[(char)this->field_0x9c] == CHAR_ATTACK2)) {
  iVar10 = 2;
}
...
local_14 = 0;
if ((*(byte *)(iVar2 + 0x6c) & 0x10) == 0) {
  local_14 = iVar10;
}
```

`iVar2` is the `UnitData`, `+0x6c` is `unit_masks2` (`types.txt:43335`), and
`local_14` is what is added to `field_0x74` (`cur_time`) below. So the bit is
not a modifier on the step, it *is* the step: with it set the addend is zero.

The bit is live, and on a short cycle. Its only writer is
`Unit::fight@005fd4d0:1098` — `unit_masks2 |= 0x10` on the arm where the unit's
order is `ATTACK` and `field_0xae == 0` — and its only clearer is
`Unit::process@00610bc0:436`, `unit_masks2 &= 0xffffffef`, inside the
`inside_up < 0` tail. `Unit::process` is the unit loop and `Objects::inc_time`
is phase 7 of the same frame, so a unit that reached that arm of `fight` this
frame has its guys' clocks frozen for this frame's step and thawed at the top
of the next. `crates/sim` models neither side: `unit_masks2` appears in the
crate only as three stated SEAMs (`path.rs:204`, `path.rs:1186`,
`transport.rs:906`) and nothing reads a `0x10`.

The document states the rule correctly and does not flag it as unmodelled.
This is the ANIM row of the shape the lane is looking for.

### R2 — §4.2's group-idle gate names three of the original's five conditions; the code has all five

| | |
|---|---|
| document | ANIM.md §4.2, "a captain (`o_up < 0`, every standalone unit) whose piece has a `GROUP_IDLE2` animation skips the roll one frame in sixteen, `(frame + 0x2e + o) & 15 == 0`" |
| code | `crates/sim/src/anim.rs:919-923` (`fn guy_set_anim`) |
| document says | three conditions: captain, the sixteen-frame phase, and the packet naming `CHAR_GROUP_IDLE2` |
| code does | five: those three, plus `g == 0` and "the guy is not already playing a `GROUP_IDLE`". Here the **code is right and the document is short** |
| difference shows | nothing — the gate can never fire (§3.2: no shipped `<UNIT>` entry names a `CHAR_GROUP_IDLE2`), so no capture separates the two readings. Dead path |
| reached | reached (`Guy::set_anim@005da300` is entered; this arm of it is not) |

`Guy::set_anim@005da300`, the `param_2 == 0` arm, is a single disjunction whose
*true* branch is the roll:

```c
if ((((uVar13 == 0) || (('\x03' < (char)this->field_0x9c && ((char)this->field_0x9c < '\a'))))
    || (this->field_0xa2 != '\0')) ||
   (((uVar28 != 0 || (local_14 == (AnimationPacket *)0x0)) ||
    (pAVar14 = AnimationPacket::get_animobj(local_14,5), pAVar14 == (AnimObj *)0x0)))) {
  ... the roll ...
}
```

`uVar13` is `is_captain`, `uVar28` is `(frame + 0x2e + o) & 15`, `field_0xa2` is
`guy_num`, `get_animobj(packet, 5)` is `CHAR_GROUP_IDLE2` — and the two the
document leaves out are `'\x03' < cur_anim && cur_anim < '\a'` (slots 4–6, the
group idles) and `guy_num != 0`. The code carries both:
`!(GROUP_IDLE1..=GROUP_IDLE3).contains(&guy.anim)` and `g == 0`. Amend the
document, not the code.

### R3 — the gaia-walker early return is in the code and in no section of ANIM.md

| | |
|---|---|
| document | ANIM.md §4 step 1 lists the idle-category return, the walk-category `des` return with its rewind, and the general same-category return. It does not name this one anywhere in §4 or §9 |
| code | `crates/sim/src/anim.rs:883-886` (`fn guy_set_anim`) — `else if who >= 8 && anim == WALK && cur_cat == 8 && guy.cur_time < guy.end_time { return; }` |
| document says | (nothing) |
| code does | an owner-8-or-9 unit asked for `CHAR_WALK` while already on the walk category and inside its length returns before every later test — which is what stops a gaia walker's coin being re-thrown outside a wrap |
| difference shows | a bird's `Guy::set_anim+0x104b` coin count. Without the return a bird would spend a wing-beat coin on every walk request rather than only on a wrap, so the falsifier is the per-frame draw count in `rontrace-run14.log` around frame 96 (§6's bird wrap) — but the code has it and the original has it, so nothing on disk *differs*. The exposure is a later editor deleting an undocumented branch |
| reached | reached (`Guy::set_anim@005da300`) |

The original, at the head of the function, straddling `LAB_005da36f`:

```c
if (((char)this->field_0xa1 < '\b') || (param_1 != CHAR_WALK)) {
  ... the attack-2 deferral, then the turn-slot rewrite ...
}
else if (UnitAnimCat[(char)this->field_0x9c] == CHAR_WALK) {
  pGVar25 = *(GameLog **)&this->field_0x74;
joined_r0x005da3d2:
  if (pGVar25 < pGVar22) { return; }     /* cur_time < end_time */
}
```

`field_0xa1` is the owner. The crate's placement (inside the `else` of
`anim == DEFAULT && !force`) is equivalent, because the branch requires
`param_1 == CHAR_WALK` and `CHAR_WALK != CHAR_DEFAULT` sends every such call
to `LAB_005da36f` anyway. The only account of it in the repository is a
sentence inside `anim.rs`'s `BIRD_TYPE` doc-comment (`anim.rs:159-162`),
which is not where §4's readers look. The specification should carry it as a
fourth early return.

### R4 — a guy playing a turn animation has its own early return, in neither the document nor the code

| | |
|---|---|
| document | ANIM.md §4 step 1, which enumerates the early returns and stops at the walk category |
| code | `crates/sim/src/anim.rs:870-882` — the `anim == DEFAULT && !force` block has arms for `cur_cat == 0` and `cur_cat == 8` and no third |
| document says | (nothing) |
| code does | (nothing) — an idle request on a guy currently on `CHAR_TURN_LEFT`/`CHAR_TURN_RIGHT` with an unsettled angle falls through to the roll and **spends a draw** |
| difference shows | a `Guy::set_anim+0x97a` draw the original does not spend, on any frame a guy that actually *plays* a turn animation is asked to idle. `gamelog-run44-islands-turners.txt` is the capture: §4.7 counts 452 guy-frames of `CHAR_TURN_LEFT`/`CHAR_TURN_RIGHT` there over nine `(who, o, slot)` combinations, types 134/265/266 |
| reached | reached (`Guy::set_anim@005da300`); the arm itself needs a guy on slot 21/22, which run44 has and no *scored* capture does |

The third arm of the `CHAR_DEFAULT`-with-`param_2 == 0` block, immediately after
the walk one:

```c
else if (((cVar26 == '\x15') || (ppvVar10 = &local_10, cVar26 == '\x16')) &&
        (ppvVar10 = &local_10, *(int *)&this->field_0x64 != *(int *)&this->field_0x18)) {
  if (*(GameLog **)&this->field_0x74 < pGVar22) { return; }
  iVar12 = 0;
  *(undefined4 *)&this->field_0x74 = 0;
  ...
  return;
}
```

`cVar26` is `cur_anim`, `0x15`/`0x16` are `CHAR_TURN_LEFT`/`CHAR_TURN_RIGHT`,
and `field_0x64 != field_0x18` is `des_angle != angle`. It is exactly the
walking guy's shape one arm above it — return while the clock is inside its
length, and rewind `cur_time` to zero without a draw once it has run out —
with "still owed a turn" standing in for "still on its way". §4.7 already
observes that `Guy::move`'s slot exclusions keep such a guy on its turn
animation; this is the other half of the same protection, in `set_anim`, and
it is the one that keeps the draw off the stream.

Harmless today only because the crate never puts a guy on slot 21/22: §4.8's
rewrite (`anim.rs:895-900`) turns a turn request into `CHAR_DEFAULT` for every
type whose art lacks the slot, and the types whose art has it (134, 265, 266)
are absent from both scored maps. It becomes a live divergence with the first
scored capture that has a Pikeman, a Catapult or a Trebuchet in it.

### R5 — the attack roll's own packet fallback is in neither

| | |
|---|---|
| document | ANIM.md §4 step 3, "category 12 with the third argument: one draw, `p < 30 → ATTACK1`, `p > 70 → ATTACK3`, else `ATTACK2`" |
| code | `crates/sim/src/anim.rs:938-952` (`fn guy_set_anim`) |
| document says | the three thresholds and nothing after them |
| code does | the three thresholds and nothing after them |
| difference shows | the `GUY` `cur_anim` of an attacking unit whose piece names `CHAR_ATTACK2` but not `CHAR_ATTACK1`/`CHAR_ATTACK3` — the original plays `ATTACK2`, this crate would play the missing slot and take `MISSING`'s three frames for its `end_time`, wrapping 3 frames in instead of the animation's length. No capture reaches it: the harness's runs have no fights (§9) |
| reached | reached (`Guy::set_anim@005da300`); the attack arm needs a fight |

The original, immediately after the roll:

```c
pAVar14 = AnimationPacket::get_animobj(local_14,param_2);
iVar12 = param_2;
if (pAVar14 == (AnimObj *)0x0) {
  iVar12 = 0xc;                    /* CHAR_ATTACK2 */
}
if ((char)this->field_0x9c == iVar12) goto LAB_005db1fc;
```

So the attack arm has the same shape as the idle roll's `:546` fallback and the
walk's `:596` — a slot the packet lacks falls back to the **category's own**
slot — and the document states that rule for the other two arms and not for
this one. Worth a line in §4 step 3 when the attacks are modelled; the
sentence is already written twice elsewhere in the document.

### R6 — `non_looping` calls `CHAR_ATTACKWALK` non-looping, and no document says so

| | |
|---|---|
| document | ANIM.md §5, "a non-looping one (`Lumberjack Dump`, `Miner Dump`, the attacks, the deaths, pack/unpack) falls to `DEFAULT` and draws" |
| code | `crates/sim/src/anim.rs:216` — `(ATTACKWALK..=24).contains(&anim) \|\| anim == DUMP_WOOD \|\| anim == DUMP_ORE` |
| document says | six families, none of them slot 10 |
| code does | the range starts at `ATTACKWALK` = 10, so slot 10 is non-looping. The function's **own** doc-comment (`anim.rs:202-204`) lists "the attacks, deaths, turns, pack/unpack and the two dumps" and does not name it either, so the code disagrees with its own docstring as well as with §5 |
| difference shows | nothing observable. Both wrap arms end in a call that cannot draw for slot 10 — looping would be `set_anim(ATTACKWALK, 0, 1)` (category 10, its own, no roll) and non-looping is `set_anim(WALK, 0, 1)` (category 8, no roll) — so only the resulting `cur_anim` differs, and no capture has a unit on `CHAR_ATTACKWALK` (§4.9: the slot is never passed by this crate) |
| reached | reached (`Guy::inc_time@005d9e10`); the slot is not |

UNSURE: whether the original's `Man Attack Walk` files sit under `<LOOPING>`.
`rondata::artdata` reads the flag per animation *file* from
`anim_graphics.xml`'s own sections (`artdata.rs:463`), so the answer is on
disk and costs a grep — and §9 already says `non_looping` "should be a
per-piece table beside `Art::piece_lengths` rather than a rule". Settling it
would close this row either way; the range's lower bound reads like it was
chosen to start at the attacks and picked up slot 10 by arithmetic.

### R7 — the idle roll's variant fallback has a second trigger, `unit_masks & 0x2000000`, in neither

| | |
|---|---|
| document | ANIM.md §4 step 2, "A variant the packet lacks falls back to `DEFAULT` (`:546–554`)" |
| code | `crates/sim/src/anim.rs:934-936` — `if v != DEFAULT && guy.gpiece >= 0 && !self.packet_has(u, guy.gpiece, v) { v = DEFAULT; }` |
| document says | one trigger: the packet does not name the slot |
| code does | the same one trigger |
| difference shows | the `GUY` `cur_anim` of a unit carrying `unit_masks & 0x2000000` after any idle roll above 69 — the original takes `CHAR_DEFAULT`, this crate takes the rolled variant and runs a different length, so the *next* wrap (and its draw) lands on a different frame. The draw itself is already spent, so the stream does not part on that frame — the `GUY` record does. No capture is known to carry the bit; a `UNITS=3` dump prints `unit_masks`, so `grep` over the corpus would settle whether one exists |
| reached | reached (`Guy::set_anim@005da300`, the idle arm, on every idle roll) |

`LAB_005daba0` is one disjunction with six terms, and the sixth is not about
the packet at all:

```c
if (((((local_14 == 0) || ((local_14->action_ids)._padding_ <= param_2))
     || (iVar12 = *(int *)((local_14->action_ids)._padding_ + param_2 * 4), iVar12 < 0)) ||
    ((animmgr.actions._padding_ <= iVar12 ||
     (AnimMgr::force_load(&animmgr,iVar12,...), *(int *)(...) == 0)))) ||
   ((*(uint *)(... unit ... + 0x68) & 0x2000000) != 0))
goto LAB_005db0d5;                                 /* iVar12 = 0 — CHAR_DEFAULT */
```

`+0x68` is `unit_masks`. So the rule is "the packet lacks it, **or** the file
would not load, **or** the unit carries `0x2000000`" — and the crate's
`packet_has` is only the first two terms. UNSURE what `0x2000000` means; the
same bit is read a second time in this function at `005db0d5`'s log line, which
suggests a "no animations" object flag rather than a gameplay one.

### R8 — a scholar's same-category test remaps every slot above `CHAR_UNPACK` to the idle, in neither

| | |
|---|---|
| document | ANIM.md §1 names `guy_flags` bit 0x80 as the scholar bit; §5 spares a scholar from the garrison gate. Neither section, nor §4, states what the bit does inside `set_anim` |
| code | `crates/sim/src/anim.rs:864` (`let cur_cat = category(guy.anim);`) and `:901` (`let target_cat = category(anim);`) — no remap |
| document says | (nothing) |
| code does | (nothing) |
| difference shows | a scholar's `GUY` `cur_time`: with the remap, a scholar asked for one work slot while playing another (both above 0x18) takes the same-category early return and **keeps its clock**; without it the clock is reset to zero and the animation restarts. The falsifier is a `GUYS=4` window over a university with a scholar inside it — §5 already notes a scholar's clocks keep running while garrisoned. No capture on disk has one |
| reached | reached (`Guy::set_anim@005da300`); the remap needs `guy_flags & 0x80` |

```c
uVar21 = *(ushort *)&this->field_0x9a & 0x80;
local_20 = UnitAnimCat[(char)this->field_0x9c];
if (uVar21 != 0) {
  local_30 = (GameLog *)0x0;
  if ('\x18' < (char)this->field_0x9c) { local_20 = CHAR_DEFAULT; }
}
param_2 = UnitAnimCat[param_1];
if (iVar12 == 0) {
  if ((uVar21 != 0) && (0x18 < (int)param_1)) { param_2 = CHAR_DEFAULT; }
  if ((local_20 == param_2) && (((local_20 != CHAR_WALK || (*(int *)&this->field_0x8 == 0x192))
      && (*(GameLog **)&this->field_0x74 < pGVar22)))) { return; }
}
```

Both sides of the comparison are remapped, current and requested, so for a
scholar every slot from `CHAR_CHOP_WOOD` (25) up collapses into one category
for the purpose of the "already playing" test. `local_20` is also what the
variant gate reads (`local_20 == CHAR_DEFAULT && param_3`), so the remap
decides whether a scholar's roll picks a variant at all. The crate reads
`category()` on both sides with no scholar arm.

### R9 — `set_anim`'s `CHAR_ATTACKWALK` head arm and its `UnitData+0xae` counter, in neither

| | |
|---|---|
| document | ANIM.md §4.9, "`CHAR_ATTACKWALK` is not passed either; no capture has a unit stepping with a target" — which disclaims *passing* it, not the arm that receives it |
| code | `crates/sim/src/anim.rs:861-900` — `guy_set_anim`'s head has no `ATTACKWALK` arm |
| document says | (nothing about the arm) |
| code does | (nothing) |
| difference shows | a `UNITDATA` field and a slot, not a draw: the original's arm zeroes `UnitData+0xae` (or rewrites the request to `CHAR_WALK` once the counter has reached six) on the frame a moving unit first acquires a target. `Guy::move@005d9240:110` is the caller that produces it — `cavarch_o >= 0 && type+0x2b4 & 0x200000` — so the capture that would show it is any trace with a mounted/chariot type stepping while it has a target; none of the scored games has one |
| reached | reached (`Guy::set_anim@005da300`); the arm is not |

```c
if (param_1 == CHAR_ATTACKWALK) {
  if (this->field_0x9c != '\n') {
    iVar15 = ... the UnitData ...;
    if (*(byte *)(iVar15 + 0xae) < 6) { *(undefined1 *)(iVar15 + 0xae) = 0; }
    else { param_1 = CHAR_WALK; }
  }
  goto LAB_005da36f;
}
```

Noted here rather than left implicit because §4.9's disclaimer reads as
covering the whole of `CHAR_ATTACKWALK`, and it covers only the caller. When
the attack walk is modelled, the arm is a second thing to model and `+0xae`
is a field neither `docs/ANIM.md` §1 nor `crates/sim` carries. (`+0xae` is
also read by `Unit::fight@005fd4d0:1097`, `field_0xae == '\0'`, which is R1's
gate — the two are the same counter.)

## What was checked and found faithful

Everything below was read on both sides and agrees. A later pass need not
re-derive it.

**§1, the record.** `cur_time`, `end_time`, `last_time`, `cur_anim`, `gpiece`,
`stopped`, `guy_num` (the crate's stack index) are all `anim::Guy`'s fields
with the documented meanings and the documented initial values
(`anim.rs:220-300`); `guy_flags & 0x20` is `Unit::guy_flag_0x20`
(`lib.rs:265`), off everywhere, as §9 says.

**§2, the categories.** `anim::CAT` (`anim.rs:184-187`) is the executable's 38
dwords slot for slot, including the four carrying walks at 8 and the three
attacks at 12; `anim::category` (`:190`) clamps out of range to 0.

**§3–§3.4, the lengths and the pieces.** `Art`'s five tables and the
precedence between them (`anim.rs:598-614` `slot_length`, `:627-637`
`packet_has`): `gaia_lengths` first, then `piece_lengths`, then the dump's
`lengths`; `MISSING` = 3 where the slot list is known, `UNKNOWN` where it is
not. The four strides (`anim.rs:425-435`) are the executable's literals; the
sum and the four existence walks of `get_unit_gpiece` are
`Sim::unit_gpiece` (`anim.rs:489-523`) with the documented loop order, the
documented gender/`packed` coordinate collapse, and `FIRST_UNIT_PIECE` when
nothing is found; `Sim::piece_of` (`:535-554`) keeps gaia on the dump's table
and the merchant "over time" arm unmodelled as the SEAM says;
`Sim::update_gpiece` (`:567-578`) recomputes without touching the clock;
`init_guys` ends in `seat_guys` (`:686`), which is §3.4's last paragraph.

**§3.2/§3.3, the install's arithmetic** (in `crates/rondata/src/artdata.rs`,
not `crates/sim`): `key_times`' relaxed bound is `28 + n*36 > 16 + size`
rejected (`artdata.rs:562`), exactly the document's `28 + 36n ≤ 16 + size`;
`game_frames` (`artdata.rs:590-600`) drops a non-looping animation's last key
for the second-to-last when it is non-zero and there are at least two, then
rounds `ms · 3 / 200` half up; the looping flag comes from the XML section
(`artdata.rs:463`), `<LOOPING>` alone being looping.

**§3.5, the count.** `SQUAD_SIZE = 1` (`anim.rs:39`) and
`crew_size + SQUAD_SIZE` in both `init_guys` (`:656-658`) and `reinit_guys`
(`:710-711`).

**§3.6, the empty crew packet.** All three consequences are in: three frames
per slot through `MISSING`, the two index tests in front of the loop flag
(`guy_inc_time`'s `self.packet_has(...) && !non_looping(...)`, `anim.rs:1177`),
and the walk resolving against **the asked guy's own** packet
(`walk_variant`, `anim.rs:1056-1063`).

**§4 step 1, the early returns.** The idle-category clock test
(`anim.rs:870-874`), the walk-category `des` test with its no-draw rewind
(`:875-882`), and the general same-category return with the walk exception and
the `BIRD_TYPE` exception to the exception (`:907-913`) all match
`005da300`'s structure. `body_at_des` (`:851-857`) drops the `off_x`/`off_y`
subtraction on §9's settled grounds.

**§4 step 2, the idle roll.** The unconditional draw, the variant gate
`cur_cat == 0 && p3`, `idle_variant`'s 69/82/95 thresholds with the
`guy_flags & 0x20` and peasant-on-masked-tile collapses (`anim.rs:378-388`,
`:929-932`) — all as the decompile has them, including that the draw is spent
even when the variant is discarded. The `openlist` gate is deliberately always
open and §6 says so.

**§4 step 3, the attack thresholds.** `< 30 → ATTACK1`, `> 70 → ATTACK3`, else
`ATTACK2` (`anim.rs:941-949`) is `0x1e`/`0x46` in the original. (Its packet
fallback is R5.)

**§4 step 4, the walk.** The category — not the asked slot — opens the arm;
the owner-9 bird coin at `% 100 > 0x31` for types `0x192..0x194`
(`anim.rs:1013-1019`); the speed test against **this guy's own** `avg_speed`
cross-multiplied as `avg*10 < base*6` / `> base*11` against the original's
`0.6f` / `1.1f` (`anim.rs:1033-1042`, and `UNIT_MOVE_SPEED = 1` in
`combat.rs:23` makes `moves · UNIT_MOVE_SPEED` the crate's `moves`); the
gather-mask override in the original's own order — `TO_WOOD`, `WITH_WOOD`,
`TO_ORE`, `WITH_ORE` — read from the carry nibble and not from the order
(`orders.rs:1334-1348`); the `:596` fallback to `CHAR_WALK` against the asked
guy's packet.

**§4 step 5, the apply.** All four arms match `005db1fc`–`005db507`: a
walk-to-walk slot change keeps the clock (the rescale is `t/t`, and the
listing agrees — both `get_anim_time` calls take the old `field_0x9c`, written
only afterwards); a new animation zeroes it; a same-slot walk keeps it while
inside its length and subtracts otherwise; everything else takes
`cur − min(cur, end)`. `last_time = -1` and the `end_time` write close the
function in the same order.

**§4's caller table**, all thirteen rows, each at the cited site:
`Unit::do_idle` (`orders.rs:1315-1316`), `Animal::do_idle`
(`anim.rs:1303-1305`), the three `do_non_flat_gather` stands
(`orders.rs:4254`, `:4350`, `:4450`, under the three distinct site labels),
the chop/mine/dump/sow/reap work animations (`orders.rs:3412`, `:4341`,
`:4410`, `:4552`, `:4599`, `:4604`, `:4609`), `do_build`'s `CHAR_SOW`-on-a-farm
/ `CHAR_BUILD` (`orders.rs:3406-3412`), `do_repair`'s unconditional
`CHAR_REPAIR` (`orders.rs:3771`), the walk from `Guy::move` and `move_step`,
the arrival stand (`anim.rs:1254-1257`), the turn arm (`:1277-1280`), the two
turn-in-place arms and `turn_towards` (`orders.rs:2803-2808`, `lib.rs:3085`,
`lib.rs:3207`), the blocked stand (`orders.rs:2848-2849`), and the wrap.

**§4.6.** `do_build` sets the work animation *before* `set_heading`
(`orders.rs:3412` then `:3415-3417`), which is the order the document says to
keep.

**§4.7.** `Movement::set_heading` is the zero-snap-flag call at every order
site; the turn arm is `guy_follow_anim`'s at-des-unsettled branch with the sea
guard (`anim.rs:1273-1281`) and `SPECIAL_ANIM` disclaimed; `guy_turns`
(`anim.rs:762-771`) derives `guy_flags & 8` from **both** writers — the type
packing, and the piece naming `CHAR_TURN_RIGHT`.

**§4.8.** The turn-request rewrite to `CHAR_DEFAULT` when the packet lacks the
slot (`anim.rs:895-900`), placed past the `CHAR_DEFAULT` early returns exactly
as the original places it; the three modelled call sites with `SITE_TURN_NEAR`
carrying `+0x3b6` and `SITE_TURN_FAR` `+0x389` (the document's near/far
address swap, `anim.rs:123-126`); the tracked crew figure paying it on its own
account (`lib.rs:3199-3208`, `guy_do_turn_anim`).

**§4.9.** `move_step`'s own `set_anim(CHAR_WALK, 0, 1)` before
`set_new_location`, gated on not having turned in place
(`orders.rs:2935-2936`), and `guys_follow`'s second request in the same frame.

**§4.10.** `guy_follow_anim` reads the arrival on the **slot** (`anim == WALK`)
and not the category (`anim.rs:1254`); the seating is `seat_guys`.

**§5.** The leader-order walk 0–9 with units in object order and no rotation
(`anim.rs:1101-1133`); the `inside_up` gate with the scholar exemption
(`:1125-1127`); the mirror for `g >= SQUAD_SIZE && category != 8` copying guy
0's slot *and* clock and stepping nothing (`:1142-1148`); the three wrap arms
with the `cur_anim == 10 ? WALK : DEFAULT` ternary and the `p3 = 0` attack arm
(`:1177-1188`); the loop, bounded where the original's is unbounded, which is
the documented deliberate divergence. (The step is R1; the queued-attack tail
and the `ATTACK2` crew broadcast after the loop are unmodelled, and §9's "the
attack animations … read as far as the table in §4 and not modelled" covers
them.)

**§6.** `guys_inc_time` sits between `process_gaia` and the ammo
(`lib.rs:2871-2878`); `init_guys`' one `init_variant` draw per figure with the
70/80/90 bands and the packet fallback; a unit created this frame is *not*
skipped; `animal_idle` is `Animal::do_idle` with the `cur_time == end_time − 1`
gate, the `% 10 < 3` coin, the three near-wander draws and the collision test,
and the far branch's literal bearing.

**§6.1.** `PLAYER_SLOTS = 8`, `Unit::is_gaia` (`anim.rs:1429-1431`), and the
gaia skips in the four `find_unit`-modelled scans.

**§7.** `herd_centre` is `((w + 2c)·0x300 + 0x480)/3` per axis (`anim.rs:408`),
`WANDER_NEAR = 0x181` (`:416`), and the far sweep's `Angle::INITIAL`
(`anim.rs:1362`).


## Adjudication — 2026-09-05, Opus

### R1 is confirmed, reached, and now asserted

The code claim holds: `anim.rs:1151` is `let step = 1u32;` with no arm for
the bit, and the two lines of comment above it account only for the
`guy_flags & 4` doubling. `unit_masks2` is not a field of any struct in
`crates/sim` at all — it appears there only in SEAM comments for other bits
(`0x2000`, `0x4000`, `0x200`) — so the freeze is not merely unimplemented,
it is unrepresentable.

**And a capture reaches it.** The question the row left open was whether any
dump on disk has a unit in melee inside a `DUMP_ALL` window, and the answer
is yes: `gamelog-run17-combat.txt` prints 35 unit-frames with `unit_masks2
16` and `gamelog-run44-islands-turners.txt` prints 26, of which run44's
carry the whole `GUY` block. Nothing had to be inferred to check the rule —
`GuyData::log_data` prints `last_time` beside `cur_time`, and `last_time` is
`cur_time` *before this frame's step*, so their difference is the step the
original actually took, per figure, per frame, already on disk.

`rondata::diff::tests::the_frozen_frame_s_figures_do_not_step_their_clocks`
walks every capture that prints both and reports:

> 26 figure-frames with `unit_masks2 & 0x10`, steps `{0: 26}`; without it
> `{1: 89522}`

Twenty-six for twenty-six at a step of zero, and 89,522 for 89,522 at a step
of one. §5's parenthesis is no longer a reading: it is the original's own
output, and the walk asserts both halves so a corpus in which nothing steps
at all cannot satisfy it.

**Why the test is green rather than red.** It asserts the *original's* rule,
not the simulation's behaviour. `crate::diff` compares no animation clock on
a frame a unit is in melee, so a diff cannot fail on the gap today, and a
red test on this branch would block the merge train for a fix this lane is
not allowed to write. The one-line arm in `Sim::guy_inc_time` is the main
lane's; the day it lands, this is the check that says what it should do, and
until then the rule cannot drift.

The remaining eight rows are not yet adjudicated.
