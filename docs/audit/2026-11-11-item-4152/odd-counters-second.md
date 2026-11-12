# odd-counters: blind second reading (item 4152)

Slug `odd-counters`. Leaves (all `int`, `LeaderData`): `repair_stamp` (+0x208), `missiles_used` (+0x7c0), `units_bribed` (+0x8f4), `nukes_launched` (+0xa38).
Reader: Claude Opus 5.5 (claude-opus-5-5), blind (has not opened the first reading or the files the brief bars).
Sources: the decompile export at `~/ghidra-projects/decomp`, the listing of `riseofnations.exe` (a full `.text` objdump), `crates/sim`.

## Ground the four leaves share

- `Leader` (size `0x6eec`) is `LeaderData` (size `0x6ee4`) at offset 0 plus a
  vbtable pointer at `+0x6ee4` (`Leader::Leader (006d2bf0)` writes
  `field_0x6ee4 = &_vbtable_`). So a `Leader` method's `this->field_0xNN` is
  the `LeaderData` offset unshifted: `Leader::reset_score (006e37f0)` fills
  seven `0xffffffff` from `field_0x8f8`, which is `age_stamp int[7]` at
  `LeaderData +0x8f8`, and `Setup::build_game (005ac190)` tests
  `*(byte*)leaders.list[i] & 1`, which is `leader_flags` at `+0x0`.
- Global array: `leaders` at `0xe3a390`, stride `0x6eec`, ten slots (loop
  bound `0xe71af0` in `build_game`). Absolute forms of the leaves are
  `0xe3a598` (`+0x208`), `0xe3ab50` (`+0x7c0`), `0xe3ac84` (`+0x8f4`),
  `0xe3adc8` (`+0xa38`).
- **Both nets, run once for all four.**
  - Export: `->repair_stamp` / `->missiles_used` / `->nukes_launched` hit only
    readers (`LeaderData::log_data (006e5110)`, `LeaderData::get_support_count
    (006da110)`); `->units_bribed` hits nothing. `field_0x208`, `field_0x7c0`,
    `field_0x8f4`, `field_0xa38` on a `leaders.list[...]` or `Leader*` base:
    the writers below and nothing else (the many `field_0x208` hits are
    `Objects`/`ObjectType`/`UnitType`/`BuildType`, other records).
    The `Leader`-shifted forms are the same offsets (prefix 0).
  - Listing (the full `.text` objdump): every absolute address
    `A` in `[0xe3a390 − 2·0x6eec, 0xe3a390 + 10·0x6eec)` with
    `(A − 0xe3a390) mod 0x6eec ∈ {0x208, 0x7c0, 0x8f4, 0xa38}` — a scan that
    also catches index-biased bases — gives exactly five instructions on the
    leaders array (`5eea82` read and `5eea98` store in `Unit::do_repair`,
    `6188fa` in `Unit::come_out`, `67306e` in `SpellType::cast_bribe`,
    `67860e` in `Ammo::do_damage`) plus two hits at `0xe33c64` inside
    `PublishWorkshopWin` (a UI string object, not a leader). Register-relative
    stores ending in `0x208(%r)`, `0x7c0(%r)`, `0x8f4(%r)`, `0xa38(%r)` (and
    the `(%r,%r[,s])` and `leal` forms) map to `Leader::init (006e3930)` and
    `Leader::reset_score (006e37f0)` among `Leader`/`LeaderData` methods; the
    rest are `Constants::init`, `GameSpyGroup`, `IFace*`, `TerrainMap`,
    `GraphicPieces`, `ChatSpace`, `AchieveChartPane`, `TileSetData`,
    `ConquestSetupWin`, `ScenarioRead::load_leaders_chunk` (its `0x208(%edx)`
    has `edx = *0xc061ec = GameAccess::game`, the lobby's player records).
  - Whole-record clears: `memset(this, 0, 0x6929)` in `Leader::close
    (006b7f80)`, `Leader::init (006e3930)` (`6e398c`) and `Leaders::clear
    (006ed5d0)` — `0x6929` covers all four leaves. The scenario loaders
    (`ScenarioRead::load_leader_chunk (009ac080)`, `load_leaders_chunk`) write
    none of them; a scan for `$0x6929`, `$0x1bbb`, `$0x6eec`, `$0x4554c`
    found no other memset/copy.
  - **The save walk is a bulk writer the offset nets cannot see.**
    `Leaders::walk_data (006e38e0)` calls `LeaderData::walk_data (006d6750)`
    on each slot, which walks the byte range `[+0x8 who, +0x692a)` as one
    block when `leader_flags & 1` (`walk(&this->who, &this->field_0x692a)`).
    All four leaves lie inside it, so a `DataWalk` in input mode (a saved
    game loaded through `WalkDataGame::walk_data (005a2360)`) overwrites
    them with the saved values, and output mode writes them out. It restores,
    it does not step; I list it under each leaf as "restore".

## `repair_stamp` (`LeaderData +0x208`, int)

**1. Writers.**
- `Unit::do_repair (005ee420)`, store `5eea98` (`movl %edx, 0xe3a598(%ecx)`,
  `ecx = owner·0x6eec`, `edx = game->frame` from `0x550(GameAccess::game)`).
  Decompile line 262.
- `Leader::init (006e3930)`: the opening `memset(this,0,0x6929)` and an
  explicit `field_0x208 = 0` at `6e3e07` (decompile line 260).
- `Leader::close (006b7f80)`, `Leaders::clear (006ed5d0)`: the `0x6929` memset.
- Restore: `LeaderData::walk_data (006d6750)` (the save walk's block).

**2. Event, step, predicate.** It is a frame stamp, not a counter: the
throttle on the "cannot afford the repair" complaint. In `do_repair`, after
the repair amount for this frame is known (`amount`, hit points ×1, nonzero),
the unit's owner `p = unit+0x9` walks the six goods `g = 0..5` it has
(`LeaderData::type_avail(g,1)`); for each with a nonzero per-type cost
(vslot `0x78` of the site's type), it computes a due of 0 or 1 (the
float32 bucket test on hit points against `max_hits / cost`, not this leaf's
business). The first good whose stock (`*(leader+0x6eb8)[g] ^ 0x8221`, the
encrypted holdings, in the holdings' own units) is **signed-less than** its
due (`0` against `1`) ends the call:
1. `Unit::kill_current_order(0)` (`5eea66`);
2. `d = frame − repair_stamp[p]` (`5eea82`); if `d <= 150` (`cmpl $0x96; jle`
   at `5eea88`, signed 32-bit) return with nothing more;
3. else `repair_stamp[p] = frame` (`5eea98`), and only if `p ==
   console->who` (`0x298(*0xc06210)`) the red feedback and `S_INVALID_ORDER`.

So the stamp is written for every player, AI or human, console or not; only
the message is console-gated. Units: sim frames. The constant is `0x96` =
150 frames inclusive, i.e. a new stamp needs `frame − stamp >= 151`. With the
cleared value 0, no stamp is written before frame 151 even on a refusal.
No goods are charged on the refusing call (the charge loop runs only after
all six pass).

**3. Reset.** `Leader::init` (memset + explicit 0), `Leader::close`,
`Leaders::clear`. Nothing else; `Leader::reset_score` does not touch it.

**4. Readers.** `Unit::do_repair` itself (`5eea82`, the throttle) and
`LeaderData::log_data (006e5110)` line 608 (the leaders dump). The crate's
checksum walk lists it (`crates/sim/src/checksum/leaders.rs:74`). No gameplay
reader beyond the throttle.

**5. Crate.** The whole branch exists: `do_repair_steps` in
`crates/sim/src/orders/jobs.rs:1802` (the `0x97` test at ~line 1916) over
`DoRepairView`, whose `feedback_stamp`/`set_feedback_stamp` are no-ops in
`SimDoRepair` (`crates/sim/src/orders/jobs.rs:2105`, "SEAM: the complaint's
throttle stamp is not carried", returns `i32::MIN`). A builder adds a
per-player `repair_stamp` (beside the other leader stamps, e.g. on
`PlayerTech`) and wires those two methods; `rondata`'s checksum lines carry
it as `repair_stamp (unheld)` (`crates/rondata/src/diff/checksum_lines.rs:1821`).

**6. Plain game.** Yes. `do_repair` runs for any citizen on a REPAIR order
against its own (or a mutually allied) damaged building; a skirmish player,
AI included, with an empty stock of a good the repair charges reaches it.

**7. Confidence.** High for the writer set (both nets, plus the absolute
scan) and for the 150-frame throttle and the reset set (listing read). The
float bucket that decides the due was not re-derived (it is the repair
mechanic, not this leaf). Not read: whether any savegame path restores it
(none found by the bulk-writer scan; the scenario chunks do not carry it).

## `missiles_used` (`LeaderData +0x7c0`, int)

**1. Writers.**
- `Ammo::do_damage (00678060)`, `67860e` `incl 0xe3ab50(%eax)` with
  `eax = round.who·0x6eec` (`imull $0x6eec, 0x3c(%ebx)` at `678607`; `ebx` is
  the round, `AmmoData +0x3c who`). Decompile line 297.
- `Leader::init (006e3930)`: memset, and explicit `field_0x7c0 = 0`
  (`6e3bbc`, decompile line 164).
- `Leader::close (006b7f80)`, `Leaders::clear (006ed5d0)`: memset.
- Restore: `LeaderData::walk_data (006d6750)` (the save walk's block).
- Register-relative `0x7c0(%r)` stores elsewhere are other records
  (`IFaceRenderManager::clear_all`, `AchieveChartPane::on_redraw`,
  `IFaceResources::IFaceResources`, `GraphicPieces::~GraphicPieces`,
  `ChatSpace::clear`).

**2. Event, step, predicate.** `+1` to the **firing round's player**
(`AmmoData +0x3c who`), once per call of `Ammo::do_damage` that takes this
path:
1. the round has a shooter (`who >= 0`, `o >= 0`, `+0x3c`/`+0x40`) and its
   graphic piece type is not 9 (`GraphicPieces::get_gpiece_type != 9`);
2. the round's `splash_area` (`+0x30`) is **nonzero** (the zero-splash arm
   never reaches it);
3. the shooter object `objects[who][o]`'s type has `obj_masks` (`ObjectTypeData
   +0x1e4`) bit `0x8000000` — the missile flag (`678200`; otherwise straight
   to `hit_target` at `678614`, no count);
4. **not shot down**: the landing cell's territory owner `L` (from
   `world+0x134` per tile, `+0xf` byte; on ocean, the first owned land cell
   of a 120-entry offset ring at `0xadcaf4`/`0xadc404`) is not `>= 0` with
   `LeaderData::has_preq(L, MISSILE_DEFENSE_BONUS)` and `L != who` — that
   case messages and closes the round at `67845d`, no count;
5. **not of the nuke line**: `shooter.is(NUCLEARMISSILE = 0x13b, 0)`
   (`ObjectData::is`, vslot `0xb8`, args pushed at `678212`/`678214`;
   non-strict lineage: the type itself, its graft and its `from` chain, so
   `ICBM 0x13c` is in it) is zero (`cmpl $0, -0x1c(%ebp); je 6785f0` at
   `67846d`). The nonzero case is the nuke arm (`nuke_stamp`, `nukes_used`,
   `Nuke::add_nuke`) and returns without touching this leaf.

On that path: `SoundGlobal::play(S_NUKE_HIT (0x50))` at the landing, then
`missiles_used[who] += 1` (`67860e`), then the ordinary `hit_target` /
`check_hit` / splash walk from `678614`. Step 1 per detonating round; the
count includes rounds whose shooter is the player's own missile regardless
of whether the splash hurts anything. Which types carry the `0x8000000` bit
and are outside the nuke line was not enumerated from data; the reader below
names V2 Rocket `0x139` and Cruise Missile `0x13a`.

**3. Reset.** `Leader::init` (memset + explicit 0), `Leader::close`,
`Leaders::clear`. Not `reset_score`. Never decremented: a spent missile
counts forever.

**4. Readers.** `LeaderData::get_support_count (006da110)` adds it to the
count of a type **index** `0x139` or `0x13a` (exact equality, not lineage),
after `num_queued[t] + high_buildings[t+0x4f]` — but only on the fall-through
path: a type with `progression & 1` and `attack != 0` returns its
group count first (barracks/stable/factory/dock by `where`, or air by
`domain == 2`) and never adds this term. `get_support_count` is called only
from `TypeData::get_cost (00664090)` (four sites, `66529b`..`6652fe`), so the
leaf makes the next V2/Cruise Missile dearer. Unlike `nukes_used`, `get_cost`
adds no second lineage term for missiles (`get_cost` line 559 is `is(0x13b)`
→ `nukes_used` only). Also `LeaderData::log_data (006e5110)` line 1306.

**5. Crate.** The event's site is
`crates/sim/src/combat/fight/projectiles.rs` `land` (~line 600): the shield
test (`profile(shooter).has(mask::MISSILE)` … `missile_defense_held`) and then
`if self.is_nuke(p.shooter) { self.nuke_land(..); return; }`; the counter
goes right after that `return`, under the same `mask::MISSILE` test, keyed by
`p.owner`, beside `Nukes::used` in `crates/sim/src/combat/nuke.rs`. The
reader's site is `nuke_ramp_extra` in `crates/sim/src/economy/pricing.rs:286`,
whose doc says "The matching `missiles_used` term (`0x139`, `0x13a`) is not
modelled". Aside, not this leaf's: the original runs `hit_target`/`check_hit`
for a missile round **after** the shield and nuke tests (`678614`), while
`land` calls them before (`projectiles.rs:576`); I did not check whether that
order is observable.

**6. Plain game.** Yes. V2 Rockets and Cruise Missiles are trained at a
Missile Silo in an ordinary skirmish (Industrial/Modern ages) and the AI uses
them; no trigger, CtW or multiplayer gate on this path.

**7. Confidence.** High for the writer set and the branch structure (listing
read from `678200` to `67861b`). Medium on which types satisfy step 3 and
not step 5 (not enumerated from the tables). Not read: `hit_target` /
`check_hit` themselves, and whether one missile can produce more than one
round (I assumed one, as the crate's `missile_dies` after `fire_ammo` implies).

## `units_bribed` (`LeaderData +0x8f4`, int)

**1. Writers.**
- `SpellType::cast_bribe (00672d00)`, `67306e` `incl 0xe3ac84(%eax)`,
  `eax = -0x18(%ebp)` = `local_1c = param_2·0x6eec` (the caster's player).
  Decompile line 387.
- `Leader::reset_score (006e37f0)`, `6e38bf` `movl %eax, 0x8f4(%ecx)` (eax 0),
  decompile line 48. Called from `Leader::init (006e3930)` (`6e3b1a`,
  decompile line 140) and from `Setup::build_game (005ac190)` (`5ad1e7`) for
  every slot with `leader_flags & 1`, inside `build_game`'s `param_2 == 0`
  block (the block that also runs `Leader::init` at its line 618; I did not
  name `param_2`, but the save-walk restore needs that block skipped).
- `Leader::init`, `Leader::close`, `Leaders::clear`: the `0x6929` memset.
- Restore: `LeaderData::walk_data (006d6750)`.
- No explicit store in `Leader::init` other than through `reset_score`; other
  `0x8f4(%r)` stores are `GameSpyGroup`, `Constants::init`, `IFaceMainBase`,
  `IFaceRenderManager`-family records.

**2. Event, step, predicate.** `+1` to the **caster's** player (`param_2`;
`param_1` caster object, `param_3` target object, `param_4` target's player)
when a Bribe is cast on a **unit** and gets past every refusal:
1. `param_2 == param_4` (own target): a separate arm (vslot `0xd8`, then the
   target's `+0x96` short drawn down by at most the spell's `+0x1d0`) and
   return — no count.
2. Refused with `unpay_cast_costs` (no count) when **both** hold:
   `param_4 == leaders[param_2].who` (`+0x8`) or the relation at `+0x74`
   is 2 in both directions (`leaders[param_2].rel[param_4] == 2` and
   `leaders[param_4].rel[leaders[param_2].who] == 2`); **and** the game's
   `team_style` is 0, 8 or 11, or `leaders[param_4].leader_flags & 2`.
3. Refused (no count) when the target answers vslot `0x18` nonzero (the unit
   test; `Unit::vftable +0x18` is the folded nonzero answer, `Build` and
   `Object` return 0) and `ObjectData::has_general(target, 0, 0x165 THECEO)`
   is `>= 0` (a CEO's cover).
4. A target that is not a unit (vslot `0x18` zero) goes to the building/city
   arm (assimilation through `Cities::capture_city`, or the building swap with
   `mask_me`), which **returns without touching this leaf**.
5. The unit arm: console-only messages and sounds
   (`S_SPY_OURS_DONE_BRIBE`, `S_SPY_THEIRS_DONE_BRIBE`), then at
   `LAB_0067306a` `units_bribed[param_2] += 1` and the target's vslot `0x94`
   (`Unit::swap_team`) with the caster's player pushed (`pushl %ebx` at
   `67306d`). Every message branch, including "neither side is the console"
   (`if (param_4 != console->who) goto LAB_0067306a`), reaches the increment.

Units: one per successful unit bribe. The count is written before
`swap_team`, so it counts a bribe whose swap later builds nothing (no record
for the new owner).

**3. Reset.** `Leader::reset_score` (from `Leader::init` and new-game
`build_game`), the three memsets. Never decremented.

**4. Readers.** `AchieveChartPane::set_chart_data (007f2c40)` case 5 (the
end-of-game statistics row, after `forts_built +0x8f0`; decompile line 237).
`LeaderData::log_data` does **not** print it (neither `->units_bribed` nor a
`0x8f4` read appears in it). No sim reader.

**5. Crate.** Already counted: `Sim::cast_bribe_unit`
(`crates/sim/src/spells/bribe.rs:37`) does `self.tech[a].bribes += 1` before
`unit_swap_team`; the field is `PlayerTech::bribes`
(`crates/sim/src/tech.rs:769`, "nothing here reads it"), tested at
`crates/sim/src/spells/cast/tests.rs:544`. The caller is `cast_on`
(`crates/sim/src/spells/cast.rs:960`..`975`), whose refusals are the
own-target test and `has_general(v, 0x165)` only — refusal 2 above (the
alliance / `team_style` / `leader_flags & 2` test) is not there, so a bribe
on a locked ally's unit would count here and not in the original. The
checksum record still prints it `unheld`
(`crates/rondata/src/diff/checksum_lines.rs:1846`): nothing wires
`bribes` to the `units_bribed` row (`crates/sim/src/checksum/leaders.rs:190`
has the row, no `"units_bribed" =>` arm). Also note the crate passes the
caster's owner where the original passes `param_2`; the same thing for a
unit-cast spell.

**6. Plain game.** Yes: a Spy's Bribe on an enemy unit in skirmish (the AI's
spies cast it too through `units/think/spellcaster.rs` in the crate). No
trigger or CtW gate on the unit arm.

**7. Confidence.** High for the writers, the step and the unit-arm
predicate (listing read at `67306a`..`67307f`). Medium on refusal 2's
meaning (`who` at `+0x8` and the `team_style` values 0/8/11 were not named).
Not read: `SpellType::cast (00676ce0)`'s own gates before it calls
`cast_bribe` (range, mana, target kinds Bribe admits).

## `nukes_launched` (`LeaderData +0xa38`, int)

**1. Writers.**
- `Unit::come_out (00617c10)`, `6188fa` `incl 0xe3adc8(%eax)`,
  `eax = unit+0x9 (owner)·0x6eec`. Decompile line 604.
- `Leader::init (006e3930)`: memset, and explicit `field_0xa38 = 0`
  (`6e4a9c`, decompile line 673).
- `Leader::close`, `Leaders::clear`: memset.
- Restore: `LeaderData::walk_data (006d6750)`.
- Other `0xa38(%r)` stores are `Constants::init`, `GraphicPieces`,
  `TerrainMap`, a stack slot. `Unit::think_scout`/`think_carry_civilian`
  read their own `0xa38`.

**2. Event, step, predicate.** `+1` to the coming-out unit's owner
(`UnitData +0x9`), in `Unit::come_out` after the unit is placed and the
AI-captain order wipe (`6187be`..`618836`), at `61883b`..`6188fa`:
1. the unit is of the nuke line: `is(NUCLEARMISSILE 0x13b, 0)` (vslot `0xb8`
   or the type's `+0x60`, args pushed at `61883d`/`61883f`; non-strict, so
   ICBM counts);
2. its order list is not empty (`orderlist +0x14` head, `+0x4` current
   `UnitOrder*`);
3. the current order answers `UnitOrder` vslot `0x100` non-null. By the PDB
   (`LF_ONEMETHOD` in `UnitOrder`'s field list `0x1A0D7`, "vftable offset =
   256") slot `0x100` is **`get_strafe_order`**; the base answer is the
   folded `xor eax,eax` (every order vtable in `vtables.txt` shows
   `Window::get_button` there), and `StrafeOrder`'s thunk (`0047ff54` →
   `47ff50`, `lea -0x4c(%ecx)`) returns the `StrafeOrder` itself. So: the
   current order is a **STRAFE** order;
4. that order's target is set: `+0x8 != −1` and `+0xc != −1`. `StrafeOrder`
   has `AttackOrder` at offset 0, which has `TargetOrder` at 0, whose `+0x8`
   is `ox` and `+0xc` is `whom` (types.txt `TargetOrder`).

So a nuke counts when it comes out of its silo on a STRAFE order at an
object. `Build::do_missile_launch (00622670)` releases a silo's missile whose
order is `AIR_ATTACK_GROUND` or `STRAFE` and then calls `Unit::come_out`
(decompile line 106); **a nuke fired at the ground (`AIR_ATTACK_GROUND`)
fails step 3 and is not counted**. Any other `come_out` of a nuke whose head
order is a targeted STRAFE (a silo emptied by `Object::kill_contents`,
`Object::eject_contents`, `Leader::blow_up_units`, … — the twenty call sites
the listing shows) would count too; I did not check that such a nuke can
hold that order there.

**3. Reset.** `Leader::init` (memset + explicit 0), `Leader::close`,
`Leaders::clear`. Not `reset_score`. Never decremented.

**4. Readers.** `LeaderData::log_data (006e5110)` line 2204 (prints it).
No sim reader found: the absolute scan shows only the writer, and the
register-relative `0xa38` reads among leader methods are only `log_data`.
(`ConquestLeader::*_territory*` read a `0xa38` of `ConquestLeader`, CtW, not
this record.) Not to be confused with `nukes_used +0x7bc` (the landing count
the price ramp reads) or `nukes_in_flight +0xa3c`.

**5. Crate.** No counter. The event's site is `Sim::come_out`
(`crates/sim/src/units/garrison.rs:376`), reached from
`Sim::do_missile_launch` (`crates/sim/src/air.rs:1799`); the increment goes
after `come_out_place`, under `self.air_line_is(u, NUCLEARMISSILE)` and
`self.current_order(u)` being `orders::Body::Strafe` with a target object.
`do_missile_launch`'s doc already marks the nuke arm's other writes as
not carried. The checksum row exists (`crates/sim/src/checksum/leaders.rs:259`)
and prints `unheld` (`crates/rondata/src/diff/checksum_lines.rs:1816`).

**6. Plain game.** Yes: Nuclear Missiles/ICBMs from a Missile Silo in a
skirmish that reaches the Atomic age with nukes on (the nuke option is a
lobby setting, not a trigger); the AI fires them. Only the STRAFE-at-a-unit
launches step it.

**7. Confidence.** High for the writer set and steps 1, 2 and 4 (listing
`61883b`..`618900`). Medium-high for step 3: the slot is named from the PDB's
`LF_ONEMETHOD` offset, which the `-GL` trap note says to prefer over
`vtables.txt`; I did not read `StrafeOrder`'s actual vtable from the PE bytes.
Not read: how the player's "launch at a unit" command builds the order (that
STRAFE is the order a nuke at an object takes is inferred from
`do_missile_launch`'s two accepted kinds).

## Summary

| leaf | writers (Name (addr)) | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `repair_stamp` +0x208 | `Unit::do_repair (005ee420)` store `5eea98`; `Leader::init (006e3930)` 0; memsets in `Leader::init`, `Leader::close (006b7f80)`, `Leaders::clear (006ed5d0)`; restore `LeaderData::walk_data (006d6750)` | `= frame` (a stamp) when a repair is refused for a good's stock < its 0/1 due, order killed, and `frame − stamp > 150` (`cmpl $0x96; jle`); every player, the message alone console-gated | `Leader::init`, `Leader::close`, `Leaders::clear` | `do_repair_steps`, `orders/jobs.rs:1802`; stub `SimDoRepair::feedback_stamp`/`set_feedback_stamp`, `orders/jobs.rs:2105` | yes | high (the float due not re-derived) |
| `missiles_used` +0x7c0 | `Ammo::do_damage (00678060)` `67860e`; `Leader::init` 0; memsets; restore `walk_data` | `+1` to the round's `who` per round with a shooter, `gpiece_type != 9`, `splash_area != 0`, shooter type `obj_masks & 0x8000000`, not shot down by the landing owner's `MISSILE_DEFENSE_BONUS` (owner ≠ who), and not `is(0x13b, 0)` | `Leader::init`, `Leader::close`, `Leaders::clear` | `combat/fight/projectiles.rs` `land`, after the nuke arm's `return`; reader `economy/pricing.rs:286` `nuke_ramp_extra` (term for `0x139`/`0x13a` missing) | yes | high on writers/branch; medium on which types qualify |
| `units_bribed` +0x8f4 | `SpellType::cast_bribe (00672d00)` `67306e`; `Leader::reset_score (006e37f0)` 0 (from `Leader::init` and new-game `Setup::build_game (005ac190)`); memsets; restore `walk_data` | `+1` to the caster's player per Bribe on an enemy **unit** past the refusals (own target, locked-ally test, CEO cover); building/city arm does not count | `reset_score`, the memsets | already counted: `spells/bribe.rs:42` `tech[a].bribes += 1`; not wired to the checksum row; `cast_on` lacks the locked-ally refusal | yes | high; medium on refusal 2's terms |
| `nukes_launched` +0xa38 | `Unit::come_out (00617c10)` `6188fa`; `Leader::init` 0; memsets; restore `walk_data` | `+1` to the unit's owner when a nuke-line unit (`is(0x13b,0)`) comes out with a current **STRAFE** order (`UnitOrder` vslot `0x100` = `get_strafe_order`) whose `ox`/`whom` are both ≠ −1; ground-targeted (`AIR_ATTACK_GROUND`) launches do not count | `Leader::init`, `Leader::close`, `Leaders::clear` | none; `Sim::come_out`, `units/garrison.rs:376`, via `Sim::do_missile_launch`, `air.rs:1799` | yes (nukes on) | high; slot `0x100` named from the PDB, PE vtable bytes not read |
