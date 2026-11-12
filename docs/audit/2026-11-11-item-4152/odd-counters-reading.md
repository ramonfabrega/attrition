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

## `units_bribed` (+0x8f4)

### Writers

| function | site | store | value |
|---|---|---|---|
| `SpellType::cast_bribe (00672d00)` | `67306e` (`incl 0xe3ac84(%eax)`, `eax = -0x18(%ebp) = caster_player·0x6eec`, set at `672d81`..`672d8d`) | decompile `:387`–`:388` (`LAB_0067306a`, `:386`) | `+1` |
| `Leader::reset_score (006e37f0)` | `6e38bf` (`movl %eax, 0x8f4(%ecx)`) | decompile `:48` | 0 |

Export net: `->units_bribed` has no hit at all (not even `log_data`);
`field_0x8f4` hits the two above and one reader,
`AchieveChartPane::set_chart_data (007f2c40)`. Listing net: absolute
`0xe3ac84` once (`67306e`); register-relative `0x8f4(` stores: 8, of which
only `reset_score`'s base is a `Leader*` (`GameSpyGroup`, `Constants`,
`ConquestGame`, `IFaceMainBase` and a stack slot are the rest).
`Leader::init` has no direct store; it zeroes the leaf by calling
`reset_score` (`6e3b1a`), and by its `memset` when `tribe < 0`.

### Event, step, predicate

**A Spy's Bribe takes a unit.** `cast_bribe(caster_id, caster_player,
target_id, target_player)`:

1. `caster_player == target_player`: a different arm (the spell's vslot
   `0xd8`, and the caster's `+0x96` reduced by the spell's `+0x1d0`); no
   count.
2. Refused (`unpay_cast_costs`, no count) when the target's player is the
   caster's ally — the inlined `is_ally`: `target_player ==
   leaders[caster].who`, or `diplos[target] == 2` both ways (`+0x74`) —
   **and** either `game->info.team_style` (`game +0x24`) is 0, 8 or 11, or
   the target leader's `leader_flags & 2` (`672d81`..`672dd9`).
3. Refused when the target object is a unit (vslot `0x18`, a folded
   predicate) and `ObjectData::has_general(target, 0, 0x165 /* THECEO */)`
   is `≥ 0` — the unit is a CEO itself, or stands within its owner's
   CEO's radius (`ObjectData::has_general (00646b00)` → `HeroesData::find_hero`).
4. A non-unit target (building) goes to the building arm (city
   assimilation via `Cities::capture_city`, or a building's
   `swap_team`): **no count** on any building path, whether it succeeds or
   is refused.
5. A unit target that passed 2–3 meets **no further test** (no enemy test,
   no territory test — those are the building arm's): the console messages
   and sounds (`S_SPY_OURS_DONE_BRIBE` / `S_SPY_THEIRS_DONE_BRIBE`) are
   chosen by who the console is, every branch falls to `LAB_0067306a`,
   **`units_bribed[caster_player] += 1`**, then the target's
   `Unit::swap_team` (vslot `0x94`, named from `vtables.txt` for
   `Unit::vftable`).

Index: the caster's player (`param_2`). Step: +1, counted **before**
`swap_team`, so a swap that builds nothing still counts. Exempt: building
targets; own targets; allied targets under the team styles named;
CEO-shielded units. Whether the order-time validity test
(`is_valid_target`) has already refused other kinds is that function's
business, not this counter's; `cast_bribe` itself does not re-test them.

### Reset

`Leader::reset_score` (decompile `:48`), called from `Leader::init`
(`6e3b1a`) and from `Setup::build_game (005ac190)` (`5ad1e7`, decompile
`:731`) once for every leader whose `leader_flags & 1` is set, after the
build. So: zero at game start, never zeroed mid-game.

### Readers

`AchieveChartPane::set_chart_data` (the end-of-game/achievements chart,
decompile `:236`–`:237`), and the checksum walk. No sim decision reads it.
`LeaderData::log_data` does **not** print it (no `->units_bribed`, no
`+0x8f4` in its body), so the per-frame dump cannot show it; the checksum
is the only oracle.

### Crate site

Already carried: `crates/sim/src/spells/bribe.rs:42`,
`Sim::cast_bribe_unit` does `self.tech[a].bribes += 1` before
`unit_swap_team`, with `pub bribes: i32` at `crates/sim/src/tech.rs:768`–`771`
documented as `LeaderData +0x8f4`; a test asserts it
(`spells/cast/tests.rs:544`). **What is missing is the wire to the checksum
leaf**: `crates/sim/src/checksum/leaders.rs:190` lists `units_bribed` but
the value match (`leaders.rs:~1270`–`1305`) has no `"units_bribed"` arm,
so it falls to `_ => None`. A builder adds `"units_bribed" =>
one(i64::from(self.tech[w].bribes))`. Second gap, for the builder to
weigh: `Sim::cast_on` (`spells/cast.rs:965`–`976`) re-tests only "same
owner" and the CEO at cast time; step 2's allied refusal is not re-tested
there (the order-time `bribe_arm`, `spells/spell_valid_target_view.rs:179`,
carries an ally arm, with `teams_locked`).

### Plain game?

Yes: Spies (and the bribe) are in a plain skirmish, and the crate already
reaches it (`docs/GOLDEN.md` §178, item 3985, cited at `cast.rs:970`).
Whether the computer casts Bribe in a given capture is the capture's
question, not the code's.

### Confidence and not read

High for writer list, step and index. Medium for step 2's exact team-style
semantics (which lobby settings are 8 and 11 was not read). Not read: the
`caster == target` arm's purpose; the building arm beyond confirming it
never reaches `LAB_0067306a` (it returns at `LAB_006733c8`'s end or through
`capture_city`/`unpay_cast_costs`).

## `nukes_launched` (+0xa38)

### Writers

| function | site | store | value |
|---|---|---|---|
| `Unit::come_out (00617c10)` | `6188fa` (`incl 0xe3adc8(%eax)`, `eax = (byte)unit+0x9 · 0x6eec`) | decompile `:604`–`:605` | `+1` |
| `Leader::init (006e3930)` | `6e4a9c` (`movl %edx, 0xa38(%ebx)`) | decompile `:673` | 0 |

Export net: `->nukes_launched` only in `LeaderData::log_data` (a read,
`:2204`); `field_0xa38` in exactly the two above. Listing net: absolute
`0xe3adc8` once (`6188fa`); register-relative `0xa38(` stores: 12, of which
only `Leader::init`'s base is a `Leader*` (`come_out`'s own `0xa38(%esp)`
at `617c53` is a stack slot; the rest are `Constants`, `GraphicPieces`,
`TerrainMap`, stack frames).

### Event, step, predicate

In `Unit::come_out` (a unit leaving the building or transport it is
inside), after the exit placement and the computer-captain wipe and before
the `is_captain` test (`618844`..`6188fa`):

1. the unit's type answers `is(0x13b /* NUCLEARMISSILE */, 0)`
   (`ObjectData::is` → type vslot `0x60`; the Nuclear Missile **line**,
   so an upgrade of it answers too);
2. the unit has a current order — the order list's head node
   (`UnitData +0xdc`) non-null and its order pointer (`+0xcc`) non-null;
3. that order's vslot `0x100` returns non-null. The PDB names slot `0x100`
   `UnitOrder::get_strafe_order` (`LF_ONEMETHOD`, `vftable offset = 256`,
   field list `0x1A0D7`). Every order class's vtable in `vtables.txt`
   holds the folded `0x41bff0` (`xor eax,eax; ret`) there — read from the
   PE bytes, not the folded name — **except** `StrafeOrder`'s `UnitOrder`
   sub-vtable at `0xb47b08` (RTTI `.?AVStrafeOrder@@`, offset 76), whose
   slot is the thunk `0047ff54` → `lea -0x4c(%ecx), %eax`: the
   `StrafeOrder` itself. So: **the current order is a `StrafeOrder`**;
4. its `+0x8` and `+0xc` are both `≠ −1`. `StrafeOrder`'s first base is
   `AttackOrder`, whose first base is `TargetOrder` (PDB field lists
   `0x11D82`, `0x81BA`): these are `TargetOrder::ox` and `whom`. So:
   **the strafe has a target object**.

Then `nukes_launched[(byte)unit+0x9] += 1` (the unit's own player). Step
+1. Index: the coming-out unit's owner.

**What that predicate admits, by reading (not established):**
`Unit::add_strafe_order (005e48c0)` is the only maker of a `StrafeOrder`
(`OrdersMemManager::get_obj(.., STRAFE)` appears nowhere else in the
export), and its head turns a strafe by a unit with the missile bit
(`type +0x1e4 & 0x8000000`) on a target `Object::valid_target` accepts
into `add_air_attack_ground_order` at the target's point.
`AirAttackGroundOrder`'s slot `0x100` is the folded zero. And
`Build::do_missile_launch (00622670)`, the silo's launch, announces a nuke
only for `AIR_ATTACK_GROUND` (its `STRAFE && !is_ally(self, self)` arm is
dead: `LeaderData::is_ally (006edb50)` answers 1 for one's own `who`).
So a nuke launched the ordinary way, at a valid target, comes out with an
`AirAttackGroundOrder` and **does not count**. The counter steps only for a
nuke carrying a `StrafeOrder` with a target — one whose target
`valid_target` refused when the order was made, or a strafe restored by a
load. **This leaf may stay 0 in every plain game**; that is a reading of
four functions, not a diff, and a capture with a nuke launch would settle
it (`log_data` prints it under `nukes_launched`).

Callers of `come_out` (listing): `Build::do_missile_launch`,
`Build::process_ejection`, `Build::train` (×4), `Object::eject_contents`,
`Object::kill_contents`, `Object::do_damage`, `Object::do_launch`,
`Leader::blow_up_units`, `Leader::eject_my_shit_from_his_ass`,
`Group::action_eject_all`, `Unit::action_come_out`, `Unit::do_cast`,
`Unit::process`, `Setup::build_units`, `Setup::place_unit`,
`GraphicEvents::execute_game_events`, and itself (×3, the squad
recursion). The counter is in `come_out` itself, so any of these that
ejects a nuke holding a targeted strafe would count it — a silo destroyed
or captured during the launch countdown (`Build +0x7a`) is the case that
comes to mind; not followed.

### Reset

`Leader::init` (`:673`, top level, unconditional). Not in `reset_score`.

### Readers

`LeaderData::log_data` only (export and listing agree: the one absolute
reference is the increment; the register-relative `0xa38(` reads are
`ConquestLeader`, `ConquestSetupWin`, `TerrainMap`). No sim decision reads
it; it matters only to the checksum walk and the dump.

### Crate site

`crates/sim/src/units/garrison.rs:376`, `Sim::come_out` — the counter
goes after the placement (`come_out_place`), on the captain, gated on
`self.air_line_is(u, crate::air::airbase::NUCLEARMISSILE)` and
`matches!(self.current_order(u).map(|o| o.body),
Some(Body::Strafe(StrafeOrder { target: Some(_), .. })))` — the crate's
`StrafeOrder::target` is exactly `TargetOrder::ox/whom`
(`crates/sim/src/orders/order.rs:498`–`501`). Storage beside
`combat.nukes` (`crates/sim/src/combat/nuke.rs`). The crate's missile
launch (`crates/sim/src/air.rs:1799`, `do_missile_launch`) issues
`AirAttackGround`, matching the reading above, so a port would leave the
leaf at 0 in every game the crate plays today. `checksum/leaders.rs:259`
lists the leaf.

### Plain game?

The writer is reached in a plain game (every `come_out`), but by the
reading above its predicate is probably never true there: a nuke would
have to carry a targeted `StrafeOrder`. No trigger, CtW or multiplayer
gate is involved either way.

### Confidence and not read

High for the writer list and for the predicate as code (listing, PDB slot
name, PE vtable bytes, RTTI all agree). Low-to-medium for "never in a plain
game": not read are `Object::valid_target (00648ba0)`'s refusals for a
nuke's target, `Group::action_flight`'s `NUCLEARMISSILE` arm (`can_nuke`;
a SEAM in the crate), every ejection caller of `come_out`, and whether a
loaded save restores a `StrafeOrder` on a missile.

## Summary

| leaf | writers | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `repair_stamp` (+0x208) | `Unit::do_repair (005ee420)` `5eea98`; `Leader::init (006e3930)` `6e3e07` | `← frame` when the repairer's player is short of a good the step charges and `frame − stamp > 150`; stamped before the console test, for every player | `Leader::init` → 0 | `orders/jobs.rs` `do_repair_steps` already calls `feedback_stamp`/`set_feedback_stamp`; `SimDoRepair` stubs both (`jobs.rs:2105`–`2110`) | yes | high |
| `missiles_used` (+0x7c0) | `Ammo::do_damage (00678060)` `67860e`; `Leader::init` `6e3bbc` | +1 for `ammo.who` when a splash round of a missile-bit shooter that is not `is(NUCLEARMISSILE)` lands unshielded | `Leader::init` → 0 | `combat/fight/projectiles.rs` `Sim::land`, after the nuke arm; reader `economy/pricing.rs` `nuke_ramp_extra` (term unmodelled) | yes (Industrial/Modern age) | high (writers), medium-high (ocean ring) |
| `units_bribed` (+0x8f4) | `SpellType::cast_bribe (00672d00)` `67306e`; `Leader::reset_score (006e37f0)` `6e38bf` | +1 for the caster's player when Bribe takes a **unit** (not a building), before `swap_team` | `reset_score` (from `Leader::init` and `Setup::build_game`) → 0 | already counted (`spells/bribe.rs:42`, `tech.bribes`); not wired to `checksum/leaders.rs`'s value match | yes | high |
| `nukes_launched` (+0xa38) | `Unit::come_out (00617c10)` `6188fa`; `Leader::init` `6e4a9c` | +1 for the unit's owner when a Nuclear-Missile-line unit comes out holding a `StrafeOrder` with a target (`ox`,`whom` ≠ −1) | `Leader::init` → 0 | `units/garrison.rs:376` `Sim::come_out`, after placement | writer reached; predicate probably never true (ordinary launch is `AIR_ATTACK_GROUND`) | high (code), low-medium (reach) |

Against the README row's writer list (`Ammo::do_damage`, `Unit::do_repair`,
`cast_bribe`, `Unit::come_out`): **no contradiction**; each is the one
event writer of its leaf, and both nets find no other. What the row does
not say: the init/reset writers (`Leader::init` for three,
`Leader::reset_score` for `units_bribed`), and that `nukes_launched`'s
writer counts only a targeted `StrafeOrder`, which an ordinary nuke launch
does not carry.
