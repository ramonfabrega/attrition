# Blind second reading — the per-guy animation clock and its RNG draws

2026-08-24. Derived from the decompile export (`~/ghidra-projects/decomp/`), the
PDB type stream, the shipped executable's `.rdata`, `game/Data/anim_graphics.xml`
and the two `DUMP_ALL` gamelogs. Written without reading `docs/ANIM.md`,
`crates/sim/src/anim.rs`, the latest commit, or the animation paragraphs of
`docs/SYNC.md` §2 step 7 / §4.1 / §6.

Line numbers of the form `set_anim:NNN` refer to the exported decompiled C in
`funcs/<Class>/<name>@<addr>.c`. Where the decompiler printed something that
cannot be right, the verdict is taken from `llvm-objdump` and the instruction
address is given.

---

## 0. Field names and the objects involved

`GuyData` (`types.txt:43851`, size 0xbc), the fields this mechanic touches:

| off | name | note |
|-----|------|------|
| +0x08 | `type` | TypeIndex |
| +0x0c/+0x10/+0x14 | `x` / `y` / `z` | Coord |
| +0x18 | `angle` | |
| +0x5c/+0x60/+0x64 | `des_x` / `des_y` / `des_angle` | |
| +0x74 | `cur_time` | ulong |
| +0x78 | `end_time` | ulong |
| +0x7c | `last_time` | int |
| +0x84 | `avg_speed` | |
| +0x88 | `gpiece` | index into `graphic_pieces.data_pieces` |
| +0x8c | `o` | the owning unit's object number |
| +0x92/+0x94 | `off_x` / `off_y` | short, the formation offset |
| +0x9a | `guy_flags` | short |
| +0x9c | `cur_anim` | char — a `UnitAnim` slot |
| +0x9d | `stopped` | |
| +0x9e | `hold_attack` | |
| +0xa0 | `queued_attack` | |
| +0xa1 | `who` | char |
| +0xa2 | `guy_num` | char |

`Guy` extends `GuyData` with `GuyOut`; `+0xbc`, `+0xc0`, `+0xd0` are in the Out
half (render interpolation) and carry no simulation meaning.

On the unit side (`UnitData`, `types.txt:43325`), an `ObjectType*` sits at
`+0x18` and **an `ObjectType` stores its own `TypeIndex` at +0x4** — that is
what `*(int*)(ptype+4)` is everywhere below. Also used: `+0x68` `unit_masks`,
`+0x6c` `unit_masks2`, `+0x8e` `o_up`, `+0x90` `o_down`, `+0xb5` `guy_mark`,
`+0xe8` = `guys.length`, `+0xf4` = `guys.list`, `+0x104` `openlist`.

`AnimationPacket` (`types.txt:43676`) hangs off a graphic piece at `+0x54`;
`+0x14` is `action_ids.len` and `+0x20` is `action_ids.list` — one global
action id per `UnitAnim` slot. `AnimMgr` (`types.txt:58497`) holds the parallel
arrays `actions` / `times` / `frames` / `loopings` / `in_queue` /
`anim_file_names`, indexed by action id.

---

## 1. The `UnitAnim` enum and the `UnitAnimCat` table (question 5)

**Claim 1.1 — `UnitAnim` has 38 real values.** From the PDB: the global
`UnitAnimCat` is `S_GDATA32`, type `0x20E9D`, at section `0002:193392`;
`0x20E9D` is `LF_ARRAY` of size 152 with element type `0x46B1` = `LF_ENUM
UnitAnim`, field list `0x46B0`. 152/4 = **38 entries**, indices 0..37.

```
0  CHAR_DEFAULT        10 CHAR_ATTACKWALK      20 CHAR_DEATH_SPLODED2  30 CHAR_WALK_WITH_ORE
1  CHAR_IDLE1          11 CHAR_ATTACK1         21 CHAR_TURN_LEFT       31 CHAR_DUMP_ORE
2  CHAR_IDLE2          12 CHAR_ATTACK2         22 CHAR_TURN_RIGHT      32 CHAR_WALK_TO_ORE
3  CHAR_IDLE3          13 CHAR_ATTACK3         23 CHAR_PACK            33 CHAR_BUILD
4  CHAR_GROUP_IDLE1    14 CHAR_ATTACKSPECIAL   24 CHAR_UNPACK          34 CHAR_REPAIR
5  CHAR_GROUP_IDLE2    15 CHAR_DEATH_STAB1     25 CHAR_CHOP_WOOD       35 CHAR_SOW
6  CHAR_GROUP_IDLE3    16 CHAR_DEATH_STAB2     26 CHAR_WALK_WITH_WOOD  36 CHAR_REAP
7  CHAR_SLOG           17 CHAR_DEATH_SHOT1     27 CHAR_DUMP_WOOD       37 CHAR_FARM
8  CHAR_WALK           18 CHAR_DEATH_SHOT2     28 CHAR_WALK_TO_WOOD
9  CHAR_JOG            19 CHAR_DEATH_SPLODED1  29 CHAR_MINE_ORE
```
plus `NUM_PEASANT_ANIMS = 38` and `NUM_UNIT_ANIMS = 25`.

**Claim 1.2 — `UnitAnimCat` is a 38-entry `int` table in `.rdata`, and it is
this.** `.rdata` VA 0x6C5000 / raw 0x6C3800, so file offset
0x6C3800 + 193392 = 0x6F2B70:

```
idx : 0  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18 19
cat : 0  0  0  0  0  0  0  8  8  8 10 12 12 12 12 15 15 15 15 15
idx :20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37
cat :15 21 22 23 24 25  8 27  8 29  8 31  8 33 34 35 36 37
```

So there are eight live categories: **0** (default + the three idles + the
three group idles), **8** (SLOG/WALK/JOG *and* all four carry/approach
gathering walks 26/28/30/32), **10** (ATTACKWALK alone), **12** (all four
attacks), **15** (all six deaths), **21**, **22**, and then each gathering
*work* anim is its own category (23,24,25,27,29,31,33,34,35,36,37).

The consequence that matters for the sim: a lumberjack carrying wood
(`CHAR_WALK_WITH_WOOD`) is in the walk category, so it goes through the walk
cadence in §6, not the idle path.

---

## 2. Where the draws are (question 1)

`Guy::set_anim(this, anim, force, randomize)` — I name the second argument
`force` and the third `randomize`; the decompiler keeps both as `param_2` /
`param_3` and *reuses* `param_2` first as the category and then as the chosen
slot, which is the single biggest reading hazard in this function.

### 2.1 The early returns, before any draw

`set_anim:90–203`, in order:

1. **`anim == CHAR_ATTACKWALK`** (`:90`). If `cur_anim != 10`, then
   `unit->recharging (+0xae) < 6` → `recharging = 0`, else the request is
   downgraded to `CHAR_WALK`. Falls through to (3).
2. **`anim == CHAR_DEFAULT && force == 0`** (`:155`) — the interesting one,
   and the only way into `:156–202`:
   - `cat(cur_anim) == CHAR_DEFAULT` (`:157`): **return if `cur_time <
     end_time`**. Otherwise fall through to (3) and draw.
   - `cat(cur_anim) == CHAR_WALK` (`:162`): compare **`guy->des_x != guy->x -
     guy->off_x` or `guy->des_y != guy->y - guy->off_y`** — the guy's own
     desired position against its own position minus its own formation offset.
     If either differs (still travelling): return if `cur_time < end_time`,
     otherwise set `cur_time = 0`, zero every sibling's `+0xd0`, and **return
     without drawing**. If both are equal (arrived), fall through to (3).
   - `cur_anim == 21 or 22` and `des_angle != angle` (`:185`): the same
     no-draw handling.
3. `LAB_005da36f` (`:105`). If `who >= 8` **and** `anim == CHAR_WALK`, and
   `cat(cur_anim) == CHAR_WALK`, return if `cur_time < end_time`. Otherwise:
   - if `cat(anim) == CHAR_ATTACK2` and `(guy_flags & 0x40) == 0`
     (`:109`): if the guy is not in position (**same three comparisons**:
     `des_x != x - off_x`, `des_y != y - off_y`, `des_angle != angle`) then
     `hold_attack = randomize ? 1 : anim` and **return**; else if
     `cat(cur_anim) == CHAR_ATTACK2` then `queued_attack = randomize ? 1 :
     anim` and **return**.
   - if `anim` is `CHAR_TURN_LEFT/RIGHT` and the packet has no such animobj,
     the request becomes `CHAR_DEFAULT`.
4. `:216–226` — the same-category early-out, and it applies **only when
   `force == 0`**: if `cat(cur_anim) == cat(anim)`, and not (walk-category
   with `type != 0x192`), and `cur_time < end_time` → **return**.

**Claim 2.1** — the entry `end_time` is latched into a local at `:87` before
any of this, and every `cur_time < end_time` test above uses that latched
value. That matters because the tail of the function overwrites `end_time`.

### 2.2 The captain / group-idle gate

`set_anim:254–271`. Reached when `cat(anim) == 0`. The **group-idle** path is
taken only when **all five** hold:

- `is_captain(unit)` — devirtualised at `:262`; `UnitData::is_captain` is
  literally `return o_up < 0` (`is_captain@0046ceb0`), and the inlined form
  reads `(*(ushort*)(unit+0x8e)) >> 15`;
- `cur_anim` is **not** already in 4..6 (`:268`);
- **`guy->guy_num == 0`** (`:269`) — only the unit's first figure;
- `(frame + 0x2e + o) % 16 == 0` (`:255–258`, `& 0x8000000f` with the signed
  fix-up, so `(game->frame + 46 + o) % 16`);
- the packet exists and has an animobj for slot 5 (`CHAR_GROUP_IDLE2`).

That path (`:404–544`) runs a formation check over `o_down` chains and, if it
concludes, calls `Guy::set_anim(other_guy, CHAR_GROUP_IDLE1 or
CHAR_GROUP_IDLE3, 1, 1)` on every member's first guy. **It does not draw.**

### 2.3 The draw, and the `openlist` test

`set_anim:272–279`, verified in the listing at `0x5dac3e`:

```
5dac3e  movsbl 0xa1(%ebx),%eax        ; guy->who
5dac45  movswl 0x8c(%ebx),%ecx        ; guy->o
5dac55  movl   0xc0aec0(,%edx,4),%eax ; units[who].list
5dac5c  movl   (%eax,%ecx,4),%eax     ; Unit* u = units[who][o]
5dac5f  cmpl   $0x0,0x104(%eax)       ; u->openlist
5dac66  jne    5dac89                 ; -> roll = 0, NO DRAW
5dac68  movl   0xc06184,%ecx          ; GameAccess::game_random
5dac6e  pushl  $0xffff
5dac73  pushl  $0x0
5dac75  calll  0xa39d70               ; Random::get(0,0xffff)
5dac7a  movl   $0x64,%ecx ; cltd ; idivl %ecx   ; roll = r % 100
```

**Claim 2.3 — the draw is suppressed when the unit's pathfinder open list is
non-null.** `+0x104` is `Tree<PathNode*,int>* openlist`. A unit with a
suspended/live A* search takes `roll = 0` with no call to `game_random` at
all. This is a hard determinism dependency of the animation clock on the
pathfinder's state, and it is easy to miss.

### 2.4 Mapping the roll to a slot

`set_anim:281–320`, listing at `0x5dac8e`. The mapping runs **only when**
`cat(cur_anim_at_entry) == CHAR_DEFAULT` **and** `randomize != 0`
(`cmpl $0x0,-0x1c(%ebp); jne skip` / `cmpl $0x0,0x10(%ebp); je skip`). If
either fails the drawn value is discarded and the slot stays `CHAR_DEFAULT`
— **the draw still happened**.

```
if (guy_flags & 0x20) {           # 5daca2
    roll >= 70 -> CHAR_IDLE1, else CHAR_DEFAULT
} else {
    roll <  70 -> CHAR_DEFAULT            # cmpl $0x46 ; jl
    roll <  83 -> CHAR_IDLE1              # cmpl $0x53 ; jl
    otherwise:
        if is_peasant(unit) and the unit's own tile is valid
           and (world.field_0x138[tile] & 3) != 0  -> CHAR_IDLE1
        else roll < 96 -> CHAR_IDLE2, roll >= 96 -> CHAR_IDLE3   # (0x5f < roll) + 2
}
```

**Claim 2.4 — the thresholds are 70 / 83 / 96.** `0..69 → 0`, `70..82 → 1`,
`83..95 → 2`, `96..99 → 3`.

Note this is **not** the same table as `Guy::init_real` uses (§5).

### 2.5 The scholar overlay

`set_anim:322–355`, gated on `guy_flags & 0x80`. `Guy::init_real:230–234` sets
that bit exactly when `ptype->type` is `0x34` or `0x35`, and the TypeIndex enum
gives `52 = SCHOLARS`, `53 = SCHOLARSKOREAN`. So **`guy_flags & 0x80` means
"scholar"**, not "animal" as the surrounding code might suggest. When a
scholar is inside a building (`inside_up != -1`), the drawn variant `v ∈ 0..3`
becomes:

- `v + 0x19` (25..28) if this unit is the building's `inside_down` head — the
  *teacher* slots;
- otherwise `v + 0x1d` (29..32) — the *student* slots; and if that would be 32,
  the `inside_down` chain is walked and, if another scholar in the building is
  already on slot 32, the slot is forced to 29.

`anim_graphics.xml` has exactly `Scholar Teach1..4` and `Scholar Student1..4`,
which is the independent confirmation.

`set_anim:207–219` also treats a scholar's slots > 24 as category 0 for both
the old and the new animation, which is what makes those eight slots behave
like idles.

### 2.6 The other two draw sites in `set_anim`

- `set_anim:575–587` (`0x5db22a`), category 12 (`CHAR_ATTACK2`): if
  `randomize != 0`, `roll % 100 < 30` → `CHAR_ATTACK1`, `> 70` →
  `CHAR_ATTACK3`, else keep the requested slot; falls back to `CHAR_ATTACK2`
  if the packet has no such animobj.
- `set_anim:619–624` (`0x5db346`), category 8: **`if (guy->who == 9)`** and the
  type is `0x192`/`0x193`/`0x194` (`BIRD`, and the two after it), draw and use
  `CHAR_JOG` when `roll % 100 > 49`. This is the only draw in the walk path.

**Claim 2.6** — for every other unit the walk cadence is a float comparison and
draws nothing: `f = avg_speed / (ptype[+0x2c0] * constants.unit_move_speed)`;
`f < 0.6` → `CHAR_SLOG`, `f > 1.1` → `CHAR_JOG`, else `CHAR_WALK`. The
constants are `0x3f19999a` and `0x3f8ccccd`. `unit_masks & 0x78000000` then
overrides the slot with 28/26/32/30 (wood-to/wood-with/ore-to/ore-with) in that
priority order.

---

## 3. `end_time`, and `cur_time` on a same-slot vs a new-slot request (q. 2)

**Claim 3.1 — `end_time` is the animation's frame count, from `AnimMgr`.**
`set_anim:721–736`, executed on every path that reaches the tail:

```
i = cur_anim
guy->+0xbc = guy->+0xc0 = cur_time                 # render interpolation
if (i < packet->action_ids.len
    && (id = packet->action_ids[i]) >= 0
    && id < animmgr.times.len) {
    AnimMgr::force_load(&animmgr, id, ..., 0)
    end_time = animmgr.frames[id]
} else end_time = 3
```

The dumps agree: gpiece 60072 slot 0 → 170, 60073 slot 0 → 101, 60074 slot 0 →
116, 60063 → 90, 60064 → 109, 60065 → 250, gpiece 371 slot 2 → 41, gpiece 13043
slot 2 → 61. `end_time` therefore depends on the guy's **own** graphic piece,
so two figures of one unit playing the same slot can have different lengths.

**Claim 3.2 — `last_time` is set to `-1` at `set_anim:253`**, before the
category branch and after all the early returns. A `last_time` of `-1` in a
dump is a reliable marker that `set_anim` got past its early-outs on that
frame. (Confirmed: run13 frame 101, the twelve wrapping fish all show
`last -1`; run12 frame 1, every animal shows `last 0` because
`Guy::inc_time` ran *after* `set_anim` and rewrote it.)

**Claim 3.3 — new slot ⇒ `cur_time = 0`.** Every path that writes a different
`cur_anim` (`:559–564` → `LAB_005db1b1`, `:594–612`, `:696–705`, `:707–711`)
ends with `*(undefined4*)&this->cur_time = 0` before jumping to the tail.

**Claim 3.4 — same slot ⇒ `cur_time -= end_time_at_entry`, clamped at zero.**
`:566–572` and `:713–717`: `cur_time = cur_time - min(cur_time,
end_time_at_entry)`. Since the same-slot case is normally reached only from
the wrap in `inc_time` (where `cur_time >= end_time`), this is the wrap
remainder.

**Claim 3.5 — the walk-category same-slot case does not touch `cur_time` at
all** if it is still short of `end_time`: `0x5db40f–0x5db417`, `jb 0x5db507`
jumps straight to the tail. Otherwise `+0xd0 = 0` and `cur_time -= end_time`.

**Claim 3.6 — the walk-category *different*-slot rescale is a no-op, and this
is a bug in the original.** The decompiler prints two `get_anim_time` calls
whose arguments look identical; the listing says they really are:

```
5db42e  movl $0x0,0xd0(%ebx)
5db438  testl %esi,%esi ; je 5db49c      ; no packet -> both times = 1
5db43c  pushl %ecx                        ; ecx is STILL guy->cur_anim from 5db404
5db43f  calll 0x918c40                    ; get_anim_time(packet, cur_anim) -> edi
5db444  movsbl 0x9c(%ebx),%ecx            ; guy->cur_anim AGAIN
5db44d  pushl %ecx
5db450  calll 0x918c40                    ; get_anim_time(packet, cur_anim) -> esi
5db459  movl $1,%eax ; cmovel %eax,%edi   ; edi = edi ? edi : 1
5db461  testl %esi,%esi ; jne .. ; movl %eax,%esi
5db46a  movb %al,0x9c(%ebx)               ; cur_anim = new slot (written AFTER both calls)
5db490  movl 0x74(%ebx),%eax ; xorl %edx,%edx
5db495  imull %esi,%eax ; divl %edi       ; cur_time = cur_time * t / t
```

Both calls pass the old `cur_anim`, and `cur_anim` is only written at
`0x5db46a`, after both. So `cur_time = cur_time * t / t = cur_time`: switching
between two walk-category slots (WALK→JOG, WALK→WALK_WITH_WOOD, …) **preserves
`cur_time` unchanged**, including when it already exceeds the new slot's
`end_time`. The intended `cur_time * len_new / len_old` never happens.

---

## 4. `Guy::inc_time` (question 3)

**Claim 4.1 — the step is 1, or 2 while attacking with `guy_flags & 4`, or 0
when the unit is frozen.** `inc_time:33–43`:

```
step = 1
if ((guy_flags & 4) && cat(cur_anim) == CHAR_ATTACK2) step = 2
if (unit->unit_masks2 & 0x10) step = 0
```

**Claim 4.2 — the gate.** `inc_time:44–45`:

```
if (guy_num < ptype->squad_size(+0x304) || cat(cur_anim) == CHAR_WALK)
    normal stepping
else
    mirror guys[0]
```

**Claim 4.3 — normal stepping.**

```
last_time = cur_time
cur_time  = cur_time + step
while (end_time <= cur_time) {          # unsigned
    packet = graphic_pieces.data_pieces[gpiece]->+0x54
    if (cur_anim < packet->action_ids.len
        && (id = packet->action_ids[cur_anim]) >= 0
        && id < animmgr.loopings.len
        && animmgr.loopings[id] != 0)
        set_anim(this, cur_anim, 0, 1)                 # A: restart the SAME slot
    else if (cat(cur_anim) != CHAR_ATTACK2)
        set_anim(this, cur_anim == CHAR_ATTACKWALK ? CHAR_WALK : CHAR_DEFAULT, 0, 1)   # B
    else {                                             # C
        set_anim(this, CHAR_DEFAULT, 0, 0)
        if (!is_hero(unit) && queued_attack) {
            q = queued_attack; queued_attack = 0
            if (q < 2) set_anim(this, CHAR_ATTACK1, 0, 1)
            else       set_anim(this, q, 0, 0)
        }
    }
}
```

Branch **A** is the looping restart; **B** falls to `CHAR_DEFAULT` (or
`CHAR_WALK` for the one `CHAR_ATTACKWALK` case); **C** is the attack case, and
note it passes `randomize = 0`, so a finished attack drops to plain
`CHAR_DEFAULT` *and still burns a draw* (the roll is discarded per §2.4).

After the loop, `inc_time:126–135` flushes a pending `queued_attack` when the
guy is not attacking and `cur_anim != 8`, and `:136–166` copies the leader's
`cur_anim`/`cur_time` onto every guy in `[squad_size, num_guys)` while the
leader is attacking. `:191–193` calls `GuyOut::graph_inc_frame` only for
`who < 8` — the renderer half, skipped for the two nature players.

**Claim 4.4 — a guy with `guy_num >= squad_size` mirrors `guys[0]`, and its
`last_time` is never written.** `inc_time:168–190`:

```
if (cur_anim != guys[0]->cur_anim) +0xd0 = 0
cur_anim = guys[0]->cur_anim
cur_time = guys[0]->cur_time
```

It does not touch `end_time` (so the mirrored guy keeps whatever length its own
`set_anim` last computed for its own graphic piece) and it does not touch
`last_time`. That is visible in the dump: run12's scout companion `0/0#1`
shows `last -1` on frames 1–4 while the leader `0/0#0` shows `0,1,2,3`.

---

## 5. `Guy::init_real` — the creation draw

**Claim 5.1 — one draw per figure at creation, with a *different* table from
§2.4.** `init_real:38–56`: `cur_anim = 0`, then `roll = get(0,0xffff) % 100`:

```
roll <  70 -> CHAR_DEFAULT
roll <  80 -> CHAR_IDLE1
roll <  90 -> CHAR_IDLE2
roll < 100 -> CHAR_IDLE3
```

**70 / 80 / 90**, not 70/83/96. (The decompile prints an unreachable
`goto LAB_005db722` inside the `roll < 0x46` arm; that arm can only fall to
`LAB_005db740`, leaving `cur_anim = 0`.)

**Claim 5.2 — a fresh guy has `cur_time = 0`, `end_time = 0`, `last_time =
-1`** (`init_real:71–73`), and `init_real` never calls `set_anim`. So the
guy's `end_time` is 0 until the first `set_anim`, and the very first
`inc_time` is guaranteed to enter the wrap loop. That is the second of the
"two creation draws" seen for a newly trained unit.

`init_real:104–111` then validates the drawn slot against the packet and falls
back to `cur_anim = 0` if the action is missing — that check does **not**
draw again.

Both `DUMP_ALL` start-of-game dumps show exactly this state: every guy at
`0/0 last -1` with a non-uniform `cur_anim` in 0..3.

---

## 6. `Objects::inc_time` and `Unit::inc_time` (question 4)

**Claim 6.1 — no rotation. Players 0..9 in ascending order, objects in
ascending order, two bands.** `inc_time@0065db70`, listing at `0x65db70`:

```
5db87  movl $0xe3a390,%edi          ; &leaders
5dbff  addl $0x6eec,%edi            ; sizeof(Leader)
5dc11  cmpl $0xe7f8c8,%edi ; jl     ; (0xE7F8C8-0xE3A390)/0x6EEC = 10 exactly
```

For each `who` in 0..9 with `leaders[who].flags & 1`:

- objects `0 .. objects.+0x15c[who]`: if `obj->+0x8 & 1`, call vtable `+0xa0`
  then vtable `+0x154` — `Unit::inc_time` then `Unit::execute_events`
  (`vtables.txt`, `Unit::vftable @ 00b417d0`);
- objects `2000 .. objects.+0x184[who]` (`movl $0x7d0,%esi` at `0x65dbdb`): if
  `obj->+0x8 & 1`, call vtable `+0xa0` only. This is the buildings/sites band
  — for a `Build` the `+0xa0` slot is `Window::on_bring_to_top`, i.e. a no-op
  as far as guys are concerned, so **buildings step no animation clocks here**.

Then, in this order: goods (`+0x28` for non-plain `Good`s), ammo
(`Ammo::inc_time`), death objects (`DeathObj::inc_time`), **`Farms::inc_time`**,
`Doober::inc_time`, `Surf::inc_time`. The farms' per-frame draws are therefore
emitted from inside this same function, *after* every guy's clock.

**Claim 6.2 — `Unit::inc_time`'s gate.** `inc_time@00610b40:8–9`:

```
if (o_up < 0 || ptype->type == 52 || ptype->type == 53)
```

i.e. **the unit is a captain (not attached under another unit), or it is a
`SCHOLARS` / `SCHOLARSKOREAN`**. Everything else skips its guys entirely.
Inside, the loops are `for i in 0..guy_mark` and `for i in
ptype->squad_size .. guys.length` — guys in `[guy_mark, squad_size)` are never
stepped. `Unit::set_anim@00616f40` walks exactly the same two ranges and
additionally clears each guy's `hold_attack` when `guys[0]` is not attacking.

---

## 7. `get_unit_gpiece` (question 6)

`get_unit_gpiece(this, type, who, o, crew, packing, queue_only, age_bonus)` —
the argument list is fixed by the only sim-side caller, `Guy::update_gpiece`,
which passes `(ptype->type, guy->who, guy->o, guy->guy_num, packing, 0, 0)`.

**Claim 7.1 — the gaia branch.** `:36–49`: if `type - 402 < 12` (i.e.
`BASE_GAIATYPES=402 .. HERDPEACOCK=413`),

```
v = (gamec.info.seed + o) % 3
for (; v >= 0; v--) {
    piece = first_bird_piece - 0x4b6 + type*3 + v
    if (loaded) verify_load; if (data_pieces[piece]) return piece
}
```

so a gaia unit's figure gets one of three variant pieces chosen by
`(map seed + object number) mod 3`, walking down to variant 0 as a fallback.
The dumps confirm the shape exactly: `HERDSHEEP` (408) → 60063/60064/60065 and
`HERDFISH` (411) → 60072/60073/60074, with `408*3` and `411*3` differing by 9 =
60072−60063, and the variant cycling with `o mod 3` in both runs.

**Claim 7.2 — the player branch.** `:91–103`:

```
art_age = (leaders[who].+0x6eb8 ^ 0x62766) + age_bonus
art_age = art_age < 5 ? art_age / 3 : 2          # 0,1,2
tribe_art = tribes[leaders[who].+0xc].+0x68
```

and then the piece index is built from

```
first_unit_piece + (type - 0x32)
  + num_unit_pieces_per_age  * art_age
  + num_unit_pieces          * tribe_art
  + num_unit_pieces_per_crew * crew
  + num_unit_pieces_per_gender
```

with four successive fallback loops that drop, in order, the gender term, the
tribe term, and then walk `art_age` down to 0; the final fallback is
`first_unit_piece`. There is a special case first (`:104–140`) for types
`MERCHANT` (61), `MERCHANTDUTCH` (62) and `FURTRAPPER` (400) when `packing ==
0`, which picks one of the six pieces at `total_num_unit_pieces - 6 .. -1`
according to `tribes[..].+0x64`.

The `crew` argument being `guy_num` is why the several figures of one unit can
draw different pieces (the scout at `0/0` has pieces 371 and 13043).

---

## 8. `AnimMgr::add`, `init_anims_pool`, and the looping flag (question 7)

**Claim 8.1 — `AnimMgr::add(name, looping)` appends one entry to six parallel
arrays**: `actions[i] = 0`, `times[i] = 0`, `frames[i] = 0`,
`anim_file_names[i] = name`, **`loopings[i] = (looping != 0)`**, `in_queue[i] =
0`. `frames[]` and `times[]` are filled by `force_load` from the `.bha`.

**Claim 8.2 — the XML is `game/Data/anim_graphics.xml`**, structured
`ROOT / ANIMATIONS / {LOOPING, NONLOOPING, BUILDING, PATH} / ANIM[@name,@file]`.
`init_anims_pool` reads the four sections in that order and calls
`AnimMgr::add(file, 1)` for the first and `AnimMgr::add(file, 0)` for the other
three (`:168`, `:242`, `:334`, `:423`). The action id is the position in the
concatenated list, and `GraphicPieces::anim_names[]` is the parallel name
array a graphic piece resolves against.

Counts in the shipped file: LOOPING 932, NONLOOPING 1037, BUILDING 102, PATH 2
= **2073 actions**, so `loopings[id] != 0` ⟺ `id < 932`.

**Claim 8.3 — the looping flag changes no draw for a category-0 animation.**
Take a guy whose `cur_anim` is an idle slot and whose clock has run out.
Branch A calls `set_anim(cur_anim, 0, 1)`; branch B calls
`set_anim(CHAR_DEFAULT, 0, 1)`. Both requests have `cat == 0`; the
same-category early-out at `set_anim:216` cannot fire because we are here
exactly because `cur_time >= end_time`; and the drawn value, not the requested
slot, decides the result (§2.4). So the number of draws and the resulting slot
are identical either way. The only observable difference is bookkeeping when
the drawn slot happens to equal `cur_anim`: branch A can reach the same-slot
`cur_time -= end_time` remainder, branch B (requesting slot 0) reaches it only
when `cur_anim` was already 0.

In the shipped data every peasant/animal idle (`Peasant Default/Idle1..3`,
`Scarecrow0 Default`, the `Scout0 Idle*`, …) is in the LOOPING section anyway.

---

## 9. `Animal::do_idle` (question 8)

`do_idle@005d7460`. Order of operations:

1. `Unit::set_anim(this, CHAR_DEFAULT, 0, 1)` — this reaches each guy's
   `Guy::set_anim` and is the animal's idle draw (§2). **This is the draw the
   sync stream sees for the forty animals on frame 1.**
2. `collide = 0`.
3. Return if `ptype->+0x218 != 0`; if `herd (+0x86) < 0`, tail-call
   `think_farm_animal` and return.
4. `g = guys[0]`; the wander gate.

**Claim 9.1 — the wander gate is `cur_time == end_time - 1` and nothing
else.** The decompiler prints a tautology (`cur_anim > 0 || cur_anim < 4`);
the listing says it really is one:

```
5d74b2  movl 0x78(%ecx),%eax ; decl %eax
5d74b6  cmpl %eax,0x74(%ecx) ; jne skip     ; cur_time == end_time - 1
5d74bf  movb 0x9c(%ecx),%al
5d74c5  cmpb $0x1,%al ; jge  proceed        ; cur_anim >= 1 -> proceed
5d74c9  cmpb $0x3,%al ; jg   skip           ; unreachable for cur_anim <= 0
5d74d1  proceed
```

For `cur_anim <= 0` the second test can never be taken, so the intended
"only while playing an idle variant" restriction is dead code as compiled.

**Claim 9.2 — one draw for the 30 % gate, then three more if the animal is
near its herd point.**

```
draw #1: if (get(0,0xffff) % 10 >= 3) return                # 5d74de..5d74ee
h  = herds.list[herd]                                        # HerdData: cx,cy,wx,wy
tx = ((h.wx + 2*h.cx) * 0x300 + 0x480) / 3
ty = ((h.wy + 2*h.cy) * 0x300 + 0x480) / 3
if (vector_dist(animal, (tx,ty)) < 0x181) {
    draw #2: dir = get(0,0xffff) % 8
    draw #3: nx  = (get(0,0xffff) % 4 + 1) * move_x[dir+1] * 0x30 + animal.x
    draw #4: ny  = (get(0,0xffff) % 4 + 1) * move_y[dir+1] * 0x30 + animal.y
    if (world.is_valid(nx,ny) && !detect_unit_collision(...))
        add_move_order(nx, ny, 1, 0, QUEUE_NEW, ...)
} else {
    find_nearby_spot(ptype, tx, ty, &sx, &sy, 0xc0, -1, 0, 0x55555555, FILTER_NOT_ME, o, who, ...)
    on success add_move_order(sx, sy, ...)                    # no further draw here
}
```

`0x300` = 768 units per world cell, `0x480/3` = 384 = half a cell, so the
target is the point one third of the way from `(cx,cy)` toward `(wx,wy)`,
cell-centred, computed with a single integer divide at the end. `0x30` = 48
units per step and the step count is 1..4 on each axis **independently**.

Note the sequencing: on the frame an animal is created, step 1's `set_anim`
gives it `cur_time = 0` and `end_time = len`, so `cur_time == end_time - 1`
cannot hold and the wander draws do not fire.

---

### 9b. `Unit::do_idle`, the player-side counterpart

`do_idle@0060dcd0` is the same shape, minus the wandering. It stashes and
restores `unit_masks2 & 0x8000` around the body, and calls
`set_anim(CHAR_DEFAULT, 0, 1)` **unless** `cat(guys[0]->cur_anim) ==
CHAR_ATTACK2` *and* `is_hero(unit)` (devirtualised: `ptype->+0x2b8 & 0x20`) —
i.e. a hero mid-swing is not interrupted. Then `collide = 0`, `check_idle`,
`think`. So an idle player unit makes the same one-draw-per-figure request an
animal does.

---

## 10. The arrival (question 9)

**Claim 10.1** — the comparisons at `set_anim:111–115` and `:163–166` are all
on the **guy's own** fields: `guy->des_x` vs `guy->x - guy->off_x`,
`guy->des_y` vs `guy->y - guy->off_y`, and (at `:111–115` only)
`guy->des_angle` vs `guy->angle`. `off_x`/`off_y` are the figure's formation
offset inside the unit; `Guy::move:222–225` subtracts them before stepping and
`set_new_location` re-adds them, so the invariant is `guy.x = position +
off_x` and "arrived" is `des_x == x - off_x`.

**Claim 10.2 — a walking guy asked for `CHAR_DEFAULT` draws only once it has
arrived.** While `des != x - off`, `set_anim:167–182` returns after at most
setting `cur_time = 0` — no draw. Once `des == x - off`, the request falls
through to the category-0 path and draws.

`Guy::move:52–74` is the caller that produces those requests on arrival:
when `des_x == x && des_y == y && des_angle == angle` and `hold_attack == 0`,
it calls `set_anim(CHAR_DEFAULT, 0, 1)` if `cur_anim == CHAR_WALK` and
`stopped != 0`, then sets `stopped = 1`. (`Guy::move:52–53` compares `des_x`
against `x` *without* the offset, unlike `set_anim`; I could not reconcile the
two and flag it in §13.)

---

## 11. Oracle check (a): run13 sim-frame 100, the twelve fish

`gamelog-run13-window-95-105.txt`. The per-pass words are
F99 `0x259a53dd`, F100 `0x60032f25`, F101 `0xa45fecaf`. Advancing the LCG
(`s = s*0x19660D + 0x3C6EF35F`, `r = ((s & 0xFFFF) * 0xFFFF) >> 16`):
`0x259a53dd` + 8 draws = `0x60032f25`, and `0x60032f25` + 18 draws =
`0xa45fecaf` — so the F99→F100 transition burned 8 draws and F100→F101 burned
18, matching the recorded counts.

Twelve `HERDFISH` (type 411) on graphic piece 60073 have `end_time 101`; they
are objects 4, 7, 10, 13, 16, 19, 22, 25, 28, 31, 34, 37 of player 8. At the
F100 pass all twelve read `cur_time 100/101`; at F101 all twelve read
`cur_time 0/101` with `last -1`.

The first twelve draws from `0x60032f25`, mapped through the §2.4 table:

| draw | roll % 100 | slot | dump `cur_anim` at F101 (o) |
|---|---|---|---|
| 0 | 95 | 2 | 2 (o=4) |
| 1 | 46 | 0 | 0 (o=7) |
| 2 | 65 | 0 | 0 (o=10) |
| 3 | 76 | 1 | 1 (o=13) |
| 4 | 3 | 0 | 0 (o=16) |
| 5 | 66 | 0 | 0 (o=19) |
| 6 | 37 | 0 | 0 (o=22) |
| 7 | 48 | 0 | 0 (o=25) |
| 8 | 47 | 0 | 0 (o=28) |
| 9 | 54 | 0 | 0 (o=31) |
| 10 | 57 | 0 | 0 (o=34) |
| 11 | 24 | 0 | 0 (o=37) |

**12 of 12 correct.** This pins, in one shot: the 70/83/96 thresholds; one
draw per wrapping figure; the ascending object order within a player; that the
wraps are the *first* draws of the frame (the remaining six are `Farms::inc_time`,
which §6.1 shows is called at the end of the same `Objects::inc_time`); and
that the previous slot (three of the twelve were on non-zero idles at F100:
o=19 and o=22 and o=25 on slot 2, o=31 and o=37 on slot 1) does not bias the
result — the roll alone decides.

---

## 12. Oracle check (b): run12 frame 0, the forty animals

`gamelog-run12-dumpall-seeds.txt`, frame-0 word `0x3bd39ae9`. Forty gaia units
`8/0 .. 8/39`, one guy each. At the start dump every one is `0/0 last -1` with
`cur_anim` from `init_real`; at the F1 pass every one is `cur_time 1`,
`last 0`, with a fresh `end_time`.

**That `last 0` fixes the order within the frame**: `set_anim` writes
`last_time = -1` and `Guy::inc_time` writes `last_time = cur_time` *before*
incrementing. `last 0` with `cur_time 1` is only reachable if
`Animal::do_idle`'s `set_anim` ran first (leaving `cur_time 0`, `end_time` set,
`last -1`) and `Guy::inc_time` ran second. The reverse order would leave
`last -1`.

Draws 48..87 from `0x3bd39ae9`, mapped through §2.4 and laid against the F1
`cur_anim` of objects 0..39:

```
draw   48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63 64 65 66 67
pred    0  0  0  0  0  0  1  0  0  0  0  0  0  0  0  0  0  0  2  2
dump    0  0  0  0  0  0  1  0  0  0  0  0  0  0  0  0  0  0  2  2
o       0  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18 19

draw   68 69 70 71 72 73 74 75 76 77 78 79 80 81 82 83 84 85 86 87
pred    3  0  2  1  0  2  0  0  0  0  0  1  1  3  0  3  2  1  0  0
dump    3  0  2  1  0  2  0  0  0  0  0  1  1  3  0  3  2  1  0  0
o      20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39
```

**40 of 40 correct.** So on frame 1 each animal spends exactly one draw, via
`Animal::do_idle` → `Unit::set_anim` → `Guy::set_anim`, in ascending object
order; the wander gate (§9.2) does not fire because `set_anim` has just reset
the clock.

---

## 13. Oracle check (c): the two-guy scouts

`0/0` and `1/0` are `SCOUT` (type 69) with two figures each, `#0` on a normal
unit piece and `#1` on a piece above 12000 (the companion). run12:

```
      0/0#0                    0/0#1                  1/0#0                  1/0#1
F0    anim 0   0/0   last -1   anim 0   0/0   last -1 anim 2  0/0  last -1  anim 1  0/0  last -1
F1    anim 2   1/41  last  0   anim 2   1/61  last -1 anim 3  1/190 last 0  anim 3  1/41  last -1
F2    anim 2   2/41  last  1   anim 2   2/61  last -1 anim 7  1/15  last 0  anim 7  1/15  last  0
F3    anim 2   3/41  last  2   anim 2   3/61  last -1 anim 7  2/15  last 1  anim 7  2/15  last  1
F4    anim 2   4/41  last  3   anim 2   4/61  last -1 anim 7  3/15  last 2  anim 7  3/15  last  2
```

This is claim 4.4 end to end. A scout's `squad_size` is 1, so `#1` has
`guy_num >= squad_size`:

- at **F0** the two figures have *independent* `cur_anim` (`1/0` shows 2 and
  1) — `init_real` draws once per figure, before any mirroring;
- at **F1** `#1`'s `cur_anim` and `cur_time` equal `#0`'s, because
  `Unit::inc_time` steps `guys[0]` first and `#1` then copies it; `#1`'s
  `end_time` is its **own** (61 vs 41, 41 vs 190) because `Unit::set_anim`
  called `Guy::set_anim` on it separately and the tail computed `end_time`
  from `#1`'s own graphic piece;
- `#1`'s `last_time` stays `-1` on F1–F4 for `0/0`, because the mirror branch
  never writes it;
- at **F2** player 1's scout starts moving: both figures go to `CHAR_SLOG`
  (7), `cat(7) == CHAR_WALK`, so `#1` now takes the *normal* branch — and its
  `last_time` starts tracking (0, 1, 2). That is claim 4.2's `|| cat(cur_anim)
  == CHAR_WALK` disjunct observed directly.

Player 0's scout never moves in these four frames and simply counts 1→4 of 41.

**Claim 13.1 — the companion figure does draw its own roll, and the roll is
then thrown away by the mirror.** `#1`'s `end_time` at F1 is 61 (`0/0`) and 41
(`1/0`), not the 0 that `init_real` left. Only `Guy::set_anim`'s tail writes
`end_time`, and the only route to it for `#1` is `Unit::set_anim`'s second loop
over `[squad_size, num_guys)` — the mirror branch of `Guy::inc_time` never
writes `end_time`, and `#1` cannot reach the wrap because it takes the mirror
branch. So `Guy::set_anim` ran on `#1` with a category-0 request, which by §2.3
draws. `Unit::set_anim` therefore costs **one draw per figure**, not one per
unit, and the extra figures' rolls are invisible in the dump because
`Guy::inc_time` overwrites `cur_anim` from `guys[0]` immediately afterwards.

### 13a. A second reading of the same frame: units that never tick

run12's F1 also shows `0/1#0` and `0/2#0` (and `1/1`, `1/2`) at `cur_time 0`
with `last -1` and a fresh non-zero `end_time`, while their siblings `0/3..0/5`
are at `cur_time 1` with `last 0`. The first group had `set_anim` run on them
and `Guy::inc_time` never run; the second group had both. That is claim 6.2
observed: `Unit::inc_time` is entered only for a captain (`o_up < 0`) or a
scholar, so a citizen sitting under another object gets its animation *set* by
the order/idle path but its clock never *stepped*.

(The four sowing citizens `0/3..0/5`, `1/3..1/5` are on `cur_anim 35`
= `CHAR_SOW`, category 35, so their `set_anim` never entered the idle branch
and cost no draw — the category table in §1 is what decides that.)

**Frame 0→1 in total burns 120 draws** (`0x3bd39ae9` advanced 120 times is
exactly `0xb6194ba1`, the F1 word). 40 of them are the animals at offsets
48–87 and 8 more are the category-0 player figures (`0/0#0`, `0/0#1`, `0/1`,
`0/2`, `1/0#0`, `1/0#1`, `1/1`, `1/2`); those 8 are **not** contiguous — I
searched all 200 offsets for the observed variant sequence `2,·,1,0,3,·,0,0`
in either player order and there is no match — so other draw sites (the
scouts' scan, unit creation, the map/AI setup) are interleaved with them.

---

## 14. Summary of what draws, per figure, per frame

| site | condition | draws |
|---|---|---|
| `Guy::init_real` | figure created | 1 (table 70/80/90) |
| `Guy::inc_time` wrap → `set_anim`, cat 0 | `cur_time >= end_time`, `unit->openlist == 0`, not the group-idle frame | 1 (table 70/83/96) |
| same, group-idle frame | captain, `guy_num == 0`, `cur_anim ∉ 4..6`, `(frame+46+o)%16 == 0`, slot 5 exists | 0 |
| same, `openlist != 0` | pathfinder search live | 0 |
| `Unit::do_idle` / `Animal::do_idle` → `set_anim(CHAR_DEFAULT,0,1)` | `cat(cur_anim)==0` and `cur_time >= end_time`; or arrived from a walk; else nothing | 1 or 0 |
| `set_anim` cat 12 with `randomize` | attack request | 1 |
| `set_anim` cat 8, `who == 9`, type 402/403/404 | bird walking | 1 |
| `Animal::do_idle` wander gate | `guys[0].cur_time == end_time - 1` | 1, plus 3 more if within 385 of the herd point |

---

## Where I am unsure

1. **`guy_flags` bits I could not source.** `init_real` sets 0x8, 0x10, 0x40
   (plane), 0x80 (scholar) and 0x100. I did not find the writers of **0x4**
   (the double step in `inc_time`), **0x20** (the reduced idle table in
   §2.4) or **0x2** (skips `turn_towards` in `Guy::move`). Every guy in both
   dumps carries `guy_flags 16`, so none of the three is exercised there. The
   cities audit's rule applies: grep the writers before trusting the reading.
2. **`Guy::move:52` vs `set_anim:163`.** `Guy::move`'s arrival test compares
   `des_x == x` while `set_anim`'s compares `des_x == x - off_x`. Either
   `off_x` is zero for every unit that reaches `Guy::move`'s branch, or one of
   the two is a latent bug. I did not chase `set_new_location` far enough to
   settle it, and no dump in hand has a formation with non-zero offsets
   arriving.
3. **~~`end_time` of a mirrored figure~~** — settled in claim 13.1: `#1`'s
   non-zero `end_time` at F1 can only have come from its own `Guy::set_anim`,
   so `Unit::set_anim` costs one draw per figure. What I still have not
   *directly* observed is the drawn value for a mirrored figure, because
   `Guy::inc_time` overwrites `cur_anim` in the same frame; the inference is
   from `end_time` alone.
4. **Where `Objects::inc_time` sits in `Game::do_frame`.** I read the function
   and its internal order (guys → goods → ammo → deaths → farms → doobers →
   surf) but not `do_frame`, so I can only say the twelve wraps precede the six
   farm draws *within* `Objects::inc_time`, which the run13 window confirms.
   Whether any earlier phase of the frame can draw before it, I did not
   establish.
5. **When `openlist` is non-null across a frame boundary.** The suppression in
   claim 2.3 is certain in the listing; how often it fires depends on the
   pathfinder's suspend/restore, which I did not read. If a unit can hold a
   suspended search across frames, its figures silently stop consuming draws,
   and any sim that always draws will desync.
6. **The float in the walk cadence** (§2.6, 0.6 and 1.1 against
   `avg_speed / (ptype[+0x2c0] * unit_move_speed)`) is a genuine `f32` divide
   in the original. It picks SLOG/WALK/JOG, which changes `end_time`, which
   changes *when* the next draw happens. It needs an exact rational in the
   port and I have not checked whether the ratio can land close enough to the
   thresholds for rounding to matter.
7. **`world.field_0x138[tile] & 3`** in the peasant override (§2.4) — I read it
   as a per-tile mask lookup via `div_3_table` but did not identify the mask
   bits. It only affects rolls in 83..99 for peasants.
8. **`ptype->+0x218`** in `Animal::do_idle:27` and `+0x2c0` in the walk cadence
   are unnamed in the export (`UnitTypeData` is all padding in Ghidra); I did
   not resolve them out of the PDB.
9. **Frame labelling.** I have treated the dumps' `FRAME n` passes as
   *states* and worked only with the transitions between consecutive passes,
   because the begin- and end-of-frame passes in run13 are byte-identical for
   the guys and report the same word. Whether the log's `FRAME 100` is
   sim-frame 100 or 101 I did not establish, and nothing above depends on it.
