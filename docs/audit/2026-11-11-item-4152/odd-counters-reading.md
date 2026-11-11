# Item 4152, family 15 (`odd-counters`) — first reading

Leaves (`LeaderData` offsets, all `int`): `missiles_used` (+0x7c0),
`repair_stamp` (+0x208), `units_bribed` (+0x8f4), `nukes_launched` (+0xa38).

Reader: Claude Opus 5.5 (`claude-opus-5-5`), wave 1 first reading,
2026-11-11. Sources: the decompile export (`~/ghidra-projects/decomp`) and
the listing of `riseofnations.exe` (`llvm-objdump`). Leaders array: slot `i`'s
`LeaderData` at `0xe3a390 + i*0x6eec`.

## Layout note (applies to every leaf)

`Leader` (0x6eec) holds `LeaderData` (0x6ee4) **at offset 0**; the extra 8
bytes are at the end. Proof: `Ammo::do_damage (00678060)` prints
`leaders.list[who].field_0x7c0` (decompile line 297) where the listing has
`incl 0xe3ab50(%eax)` at `67860e`, and `0xe3ab50 = 0xe3a390 + 0x7c0`; so a
`Leader`-typed `field_0xNN` is the `LeaderData` field at `+NN`, no bias.
The `+8` sibling offsets (`0x7c8`, `0x210`, `0x8fc`, `0xa40`) are other
leaves (`tech_cat_frame`, `aggression`, nothing, `frame_attacked`) and were
checked only to rule them out.

Nets run for all four leaves:

- export: `grep -rn -- '->leaf\b'` and `grep -rn 'field_0x<off>\b'` over
  `decomp/funcs`;
- listing: a full `llvm-objdump -d` of the executable, grepped for the
  absolute address `0xe3a390+off` (any use) and for every
  `mov/inc/dec/add/sub/and/or/xor/neg` with destination `0x<off>(%reg)`;
  each hit mapped to its function through `INDEX.tsv`. Register-relative
  hits whose base is not a `Leader*` (UI panes, `Objects`, `Constants`,
  `GameInfo` player records, stack frames `-0x<off>(%ebp)`) are excluded by
  name of the function and, where it mattered, by reading the base.
- bulk writers common to all four: `Leader::init (006e3930)` opens, when
  its second argument (`tribe`, the `+0xc` it stores) is negative, with a
  `memset` of `0x6929` bytes of the record (`6e398c`..`6e3995`), and
  `LeaderData::walk_data (006d6750)` streams every leaf in on a load
  (`Leaders::walk_data (006e38e0)`, `LoadGame::verify_load (005a39f0)`;
  its other callers are the checksum and the save, which read). Neither is
  an event; both are listed once here and not repeated per leaf.

## `repair_stamp` (+0x208)

### Writers

| function | site | store | value |
|---|---|---|---|
| `Unit::do_repair (005ee420)` | `5eea98` (`movl %edx, 0xe3a598(%ecx)`; read at `5eea82`) | decompile `:262` | `game->frame` |
| `Leader::init (006e3930)` | `6e3e07` (`movl %eax, 0x208(%ebx)`, `eax` zeroed at `6e3dce`) | decompile `:268` | 0 |

Export net: `->repair_stamp` only in `LeaderData::log_data (006e5110)`
(a read). `field_0x208` has 20 files; the two above are the only ones whose
base is `leaders.list[...]` or a `Leader*` `this`. Listing net: 44
register-relative stores at `0x208(`; the only `Leader*` base is
`Leader::init`'s `ebx`. `ScenarioRead::load_leaders_chunk (009acde0)`'s
`9acf92` store is into `GameInfo`'s player records (its decompile is
`(game->info).player[i]...` throughout), not `LeaderData`. Absolute
`0xe3a598`: the read and the write in `do_repair`, nothing else.

### Event, step, predicate

A **throttle stamp on the "cannot afford this repair" complaint**, written
for every leader, not only the console player's.

In `Unit::do_repair`, after the repairer has passed the gates and the repair
amount for this frame is non-zero, the function walks the six goods
(`0..5`, those `type_avail` for the repairer's player). For each it works
out whether this step crosses a whole unit of that good's price — `due[g]`
is 1 when `(int)(hp_left / (good_hits / price))` is less than
`(int)((hp_left + amount) / (good_hits / price))`, **single-precision
float** quotients truncated (decompile `:247`–`:251`; the crate already
carries this as `Single` arithmetic). The first good whose stock
(`LeaderDataEncrypt` bucket, `^ 0x8221`) is below `due[g]`:

1. `kill_current_order(0)` on the repairer;
2. if `game->frame − repair_stamp ≤ 150` (`subl 0xe3a598(%ecx)`,
   `cmpl $0x96`, `jle` — a signed 32-bit wrapping difference; units: sim
   frames), return with nothing else;
3. otherwise `repair_stamp ← game->frame` (`5eea98`) — **before** the
   console test;
4. only if the repairer's player is `console->who`
   (`0xc06210 +0x298`) the red feedback message (`loc_str +0x11f6c`) and
   `S_INVALID_ORDER` (`push 0x40`) play.

Index: the repairer's own player, `(byte)unit+0x9`. Step: assignment, not
increment. Exempt: nothing by kind; a computer player's repairer stamps it
exactly as a human's. The goods not `type_avail` are skipped, and a good
with `price == 0` gives `due 0` and never trips (crate's reading; in the
original a zero price makes `local_1c == 0` skip the block, decompile
`:244`).

Because `Leader::init` sets it to **0** (unlike `ctw_hero_retreat_stamp`
beside it, set to −1000 at `6e3dd0`), no complaint can stamp before frame
151: the first shortfall at frame `f ≤ 150` returns at step 2.

### Reset

`Leader::init` only (`:268`, unconditional, top level). Not in
`Leader::reset_score (006e37f0)`, not in any per-frame fold.

### Readers

`Unit::do_repair` itself (the throttle) and `LeaderData::log_data` (the
dump). It decides only whether the message and sound play, which is
interface; but the stamp itself is in the checksum walk, so a sim that does
not carry it diverges on the leaders record.

### Crate site

`crates/sim/src/orders/jobs.rs`: `do_repair_steps` already has the shape —
`v.feedback_stamp()` / `v.set_feedback_stamp(frame)` around the
`frame.wrapping_sub(..) < 0x97` test (`jobs.rs:1913`–`1921`). The
`SimDoRepair` view stubs both: `feedback_stamp` returns `i32::MIN` and
`set_feedback_stamp` is a no-op (`jobs.rs:2105`–`2110`, a SEAM comment).
A builder stores a per-player `repair_stamp: i32` (initial 0) and makes the
view read and write it; `checksum/leaders.rs:74` already lists the leaf.
Side note for the builder: the SEAM comment says "a complaint is always
due", but `frame.wrapping_sub(i32::MIN)` is negative for every frame below
2³¹, so with the stub the throttle **always returns** — harmless today
because both arms are empty, wrong once one is not.

### Plain game?

Yes. Any repairer (Citizen on a damaged building; the AI repairs) whose
player is short of a good the repair charges reaches `5eea98`. No trigger,
CtW or multiplayer gate on the path.

### Confidence and not read

High for the writer list (both nets agree, two stores) and for the
predicate (decompile and listing agree on `> 150`, assignment, stamp before
console test). Not read: the gates earlier in `do_repair` (ally,
territory, active, under attack) beyond noting they precede it; the float
`due` arithmetic was not re-derived bit for bit here (the crate's
`do_repair_steps` already ports it under item 3346).

## `missiles_used` (+0x7c0)

### Writers

| function | site | store | value |
|---|---|---|---|
| `Ammo::do_damage (00678060)` | `67860e` (`incl 0xe3ab50(%eax)`, `eax = ammo.who·0x6eec`) | decompile `:297`–`:298` | `+1` |
| `Leader::init (006e3930)` | `6e3bbc` (`movl %ecx, 0x7c0(%ebx)`) | decompile `:164` | 0 |

Export net: `->missiles_used` only in readers (`get_support_count`,
`log_data`); `field_0x7c0` in exactly the two above. Listing net: absolute
`0xe3ab50` once (`67860e`); register-relative `0x7c0(` stores: 8, of which
only `Leader::init`'s has a `Leader*` base (the rest are
`IFaceRenderManager`, `AchieveChartPane`, `IFaceResources`,
`GraphicPieces`, `ChatSpace`, and a `Constants::log_data` stack slot).

### Event, step, predicate

**A missile that is not of the Nuclear Missile line detonates.** In
`Ammo::do_damage`, on the arm for a round with `splash_area != 0`
(`AmmoData +0x30`; the `== 0` arm is the plain hit):

1. the shooter's type has the missile bit, `ObjectTypeData::obj_masks &
   0x8000000` (`678200`; the crate's `mask::MISSILE`, `1 << 27`); without
   it the code jumps straight to `hit_target` at `678614`, past the
   counter;
2. `is(0x13b /* NUCLEARMISSILE */, 0)` on the shooter's type is taken
   (`678212`..`67823e`, kept at `-0x1c(%ebp)`);
3. the territory owner of the landing tile is found — the tile under the
   point; on ocean, the first land tile of a 120-entry offset ring
   (`0xadcaf4`/`0xadc404`) whose owner is `≥ 0`, else −1;
4. **shield**: if that owner is `≥ 0`, holds `MISSILE_DEFENSE_BONUS`
   (`has_preq`, TypeIndex 693), and is not the shooter's player
   (`ammo.who`), the round is closed with messages and nothing counted
   (`goto LAB_0067845d`). Note the test is `!=`, not "enemy": a missile
   into an ally's shielded land is stopped too;
5. if step 2 answered non-zero (`cmpl $0x0, -0x1c(%ebp)` at `67846d`), the
   nuke arm runs (`nuke_stamp`, `nukes_used`) and returns;
6. otherwise `S_NUKE_HIT` plays (`push 0x50`, `678602`) and
   **`missiles_used[ammo.who] += 1`** (`67860e`), then `hit_target` and
   the splash continue as for any splash round.

The decompile prints step 5's comparison as `!= GAMELOG_STARTING_INDEX`:
that is the enum's name for 0, a misnamed literal; the listing compares
with `$0x0`.

Index: `AmmoData::who` (`+0x3c`), the shooter's player at launch. Step:
+1 per detonating round. Exempt: rounds whose shooter lacks the missile
bit; the Nuclear Missile line (step 5); rounds stopped by a shield; the
outer guards of the function (`o ≥ 0`, `who ≥ 0`, graphic piece type
`!= 9`, decompile `:84`). In the shipped tree the types this leaves are the
V2 Rocket (0x139) and Cruise Missile (0x13a) lines — which is exactly what
`get_support_count` pairs it with — but which types carry bit 27 is data,
not code, and was not read here.

### Reset

`Leader::init` (`:164`, top level, unconditional). Not in `reset_score`.
No other writer: it only grows.

### Readers

`LeaderData::get_support_count (006da110)` adds it to the count for type
`0x139` or `0x13a` (decompile `:37`–`:39`); that count is called four times
from `TypeData::get_cost (00664090)`, so **each missile that lands makes the
next V2/Cruise Missile of that player dearer, permanently** — a reader that
changes a decision (price, therefore AI affordability and the escrow).
`LeaderData::log_data` dumps it.

### Crate site

`crates/sim/src/combat/fight/projectiles.rs`, `Sim::land`: the shield test
(`projectiles.rs:604`–`614`) and the nuke arm (`:617`–`619`) are already
there; the counter goes on the fall-through after `if self.is_nuke(..)`,
guarded by `self.profile(p.shooter).has(mask::MISSILE)`, indexed by
`p.owner`. Storage beside `self.combat.nukes.used`
(`crates/sim/src/combat/nuke.rs`, `nuke_land`). The reader:
`crates/sim/src/economy/pricing.rs:281`–`300`, `nuke_ramp_extra`, whose doc
says outright that the `missiles_used` term for `0x139`/`0x13a` is not
modelled. `checksum/leaders.rs:141` lists the leaf.

Two divergences of the crate's `land` from the original's order, noticed
in passing and **not** part of this leaf's evidence: (a) the crate runs
`hit_target`/`check_hit` before the shield and nuke tests for every splash
round, where the original reaches `hit_target` (`678616`) only after the
missile handling; (b) the ocean case of step 3 is a SEAM in the crate
(`projectiles.rs:607`).

### Plain game?

Yes, given the age: a skirmish that reaches the Industrial (V2) or Modern
(Cruise Missile) age, with either side building missiles, reaches
`67860e`. The AI builds missiles. No trigger/CtW/multiplayer gate.

### Confidence and not read

High for the writer list and the step. Medium-high for the full predicate:
the ocean ring's table length (120 entries, `local_18 < 0x1e0` bytes) and
exact order were read from the decompile only, not the listing; which types
carry `obj_masks` bit 27 was not read (data). `LAB_0067845d` (decompile
`:379`) is `close(this); return 0` — the round dies with no damage.

