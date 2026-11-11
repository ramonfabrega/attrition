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

