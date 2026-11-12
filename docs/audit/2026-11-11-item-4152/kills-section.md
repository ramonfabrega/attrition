## 2. The kill, loss and raze counters

### 2.1 Leaves

- `units_killed`, +0x80c, int.
- `units_lost`, +0x810, int.
- `buildings_razed`, +0x818, int.
- `buildings_lost`, +0x81c, int.

### 2.2 The rule

All four step by +1, never decrement, and no sim decision reads them.
Their readers are `LeaderData::log_data (006e5110)`, the end-of-game chart
(`AchieveChartPane::set_chart_data`), the checksum walk, and for the two
unit leaves `num_units_killed` / `num_units_lost` and an end-of-game Steam
stat. **Reset**: 0 at `Leader::reset_score (006e37f0)` (every live slot at
`Setup::build_game`, and in `Leader::init`); the bulk memsets, plus
`Leader::close (006b7f80)`, which also memsets `[+0, +0x6929)`; restored by
a load.

**"A countable unit death"**, the test the unit leaves share, is
`Object::take_damage (00652020)` reaching its death branch on a struck
figure that is:

1. not a building (vslot `+0x1c` answers 0);
2. its squad's captain (vslot `+0xe8`);
3. alone: `curr_uber_size(0) == 1`, no live `o_down` follower;
4. not a decoy (`unit_masks (+0x68) & 1` clear).

A non-captain figure fails test 2 and counts nothing, so a squad counts
once, at its last figure. The unit test (vslot `+0x18`) also answers 1 on
`Animal`, and there is no animal exemption to add. No plain-game
`take_damage` strikes an animal: gaia owns them from a slot ≥ 8, and
`valid_target_const` refuses `who > 7`.

**`units_lost`.** One writer, `take_damage`: on a countable unit death,
`units_lost[victim's player] += 1`. The store comes before the attacker
test, and `param_3` and `param_5` do not enter, so a death with no killer
counts: attrition (`Unit::suffer_attrition (005e1a10)` strikes through
vslot `+0x16c`), wall decay, triggers. Spell kills, `Object::disband` and
a direct `die` do not count it.

**`units_killed`.** Three writers:

- `take_damage`, on a countable unit death, and only when the call
  carries an attacker object (`param_7 >= 0`):
  `units_killed[param_8] += 1`, where `param_8` is the attacker's player.
  There is no ally, self or ownership test before the store. The `is_ally`
  test after it picks only the sign of `score_combat` (+0x44). Beside the
  store it also steps victim `+0xa58` and `leaders[param_8] +0xa5a`.
- `SpellType::cast_sabotage (006720e0)`, unit arm (a Sniper or Commando
  kill). The target must be live (`flags(+8) & 1`; otherwise the mana is
  refunded and the cast returns), and the inner test is a unit (`+0x18`),
  not a building, captain, alone, not a decoy. If it passes:
  `units_killed[caster] += 1`, target `+0xa58` and caster `+0xa5a` step,
  and `score_combat[caster] += cost / 5`. Either way the target then dies
  through vslot `+0x158` directly (dtype 3 with objmask `0x200000`, else
  2), so the victim's `units_lost` does not step.
- `SpellType::cast_counterintel (00673740)`. A non-captain target is
  replaced by its captain (`+0xe4`). The gate: the target's player is not
  the caster, not `leaders[caster].who` (+0x8), and not mutually allied
  (`diplos (+0x74) == 2` both ways). Inside the gate, if the target is a
  countable unit (unit, captain, alone, not a decoy),
  `units_killed[caster] += 1` and `score_combat` steps. Nothing else of
  this family steps: no `+0xa58`, `+0xa5a` or `units_lost`. Then
  `die(2, -1, 0)` runs inside the gate, whatever the inner test said.
  Outside the gate there is no die.

The caster's player reaches counterintel through `SpellType::cast
(00676ce0)`'s third argument.

**`take_damage`'s return**, which the raze writers test: 0 when the
target is still alive or is a clamped city building; 1 from the
site-abandon disband; 1 after `die` (vslot `+0x158`) when the dead
object was not a decoy; 2 after the decoy-overflow arm.

**`buildings_razed`.** Two writers:

- `Object::do_damage (0064a480)`: when its `take_damage` call returns
  non-zero and the target is a building (`+0x1c`),
  `buildings_razed[this's player] += 1`. There is no owner or ally test,
  so razing an ally's or one's own building counts. The later `is_ally`
  picks the sign of `score_combat`, floored at 0 on the minus side. On a
  unit kill `do_damage` returns before this. `this` is the shooter:
  `Ammo::do_damage (00678060)` passes the ammo's recorded shooter, and
  `Nuke::do_damage (0092bc80)` passes the object the nuke effect records.
- `cast_sabotage`, building arm:
  `take_damage(dmg, 0, 3, -1, 1, 0x55555555, -1, -1, 0)` on the building
  itself (`Build`'s vslot `+0xac` returns `this`), with
  `dmg = max(+0x114 / 2, the type-indexed floor)`. A non-zero return gives
  `buildings_razed[caster] += 1`.

On a building, "non-zero" is either:

- **death**: accumulated damage ≥ hits, and not a city building. A city
  is clamped and returns 0, for every caller, because the clamp is not
  under the `param_5 == 0` block; or
- **the site-abandon disband**, which needs all four of:
  - unfinished: vslot `+0x4c` = `WallData::is_active (00472350)` =
    `flags & 4`, clear;
  - the owner is a computer player (`leader_flags & 4` clear);
  - `hits(0) ≤ 2·damage`;
  - `build_masks (+0x60) & 0x2000`, whose only writer is
    `Leader::check_orphaned_buildings (006c9f20)`.

  The disband returns 1. It credits a raze while the owner's
  `buildings_lost` does not step.

**`buildings_lost`.** One writer, `take_damage`: when a building or wall
segment (`+0x1c` non-zero) reaches the death branch,
`buildings_lost[owner] += 1`, then `ScenarioData::last_razed[who] = o`,
then straight to `die`. There is no attacker test and no `param_5` test,
so combat, sabotage, triggers and border decay all count. Border decay is
`Wall::process (00640450)` striking
`take_damage(8, 0, 1, -1, 1, 0x55555555, -1, -1, …)` on a started building
that stands on another, non-allied player's tile. It reaches every
building, not only walls: `Build::process` calls `Wall::process` directly.
Not counted:

- a finished city building (clamped);
- the site-abandon disband;
- `Wall::process`'s disband of an unstarted site;
- `Object::disband` and a direct `die`.

So, summed over players, `units_lost ≠ units_killed` (attackerless
deaths, spell kills) and `buildings_lost ≠ buildings_razed` (abandon
disbands, attackerless decay).

### 2.3 In the crate

None of the four is held. Each exists only as a checksum name
(`crates/sim/src/checksum/leaders.rs:157-161`), and each is `(unheld)`,
count 2, in `LEADERS_HELD_OUT`
(`crates/rondata/src/diff/checksum_lines.rs:1769, 1770, 1847, 1848`). A
port gives each a value arm and removes its four entries. The sites, all
in `crates/sim/src/combat/fight/delivery.rs` unless named:

- **`take_damage` unit deaths**: `Sim::take_damage_typed` (`:469`), its
  `Taken::Died` arm (`:537`), gated on `alone` (`:515`,
  `takes_remainder_share` in `units/store.rs:667`, the same captain ∧
  size-1 test) and `!decoy` (`:530`), before `close_count`.
  - `units_lost` goes to the struck unit's owner, with no attacker test.
  - `units_killed` goes to `owner_of(_by)` only when `_by` is an object.
    The crate has no `param_7 < 0` notion there, so the
    attacker-present test is the builder's to carry.
- **Attrition deaths**: `Sim::die_by_attrition` (`:844`, called from
  `territory/upkeep.rs:307`) bypasses `take_damage_typed`, so `units_lost`
  goes there too. **Trap**: read the captain/alone/decoy test *before*
  its `relink_squad`.
- **Sniper/Commando**: `Sim::cast_sniper_unit` (`:806`). It needs the
  caster's player, which its `(v, dtype)` signature lacks. It credits
  `units_killed` only, never `units_lost`.
- **Counterintel**: `Sim::cast_counterintel`
  (`crates/sim/src/spells/cast.rs:1031`, SEAM doc at `:1028`). It
  credits `units_killed` only.
- **`do_damage` razes**: `Sim::do_damage_round`'s
  `if let Obj::Building(b) = target { if killed {` block (`:212`). The
  count goes to `owner_of(attacker)`, *before and outside* the plunder's
  unit / non-splash / non-air / non-ally gate.
- **Sabotage razes**: `Sim::cast_sabotage_building` (`:884`). It needs
  the caster's player.
- **Building deaths**: `Sim::damage_building`'s `Taken::Died` arm
  (`:1041`), before `die_building(b)`, for the building's owner. Do not
  put it in `die_building`, which other paths reach. The city clamp
  (`:1025`) returns before the arm. This one function serves:
  - combat (`take_damage_typed`);
  - sabotage (`cast_sabotage_building`);
  - border decay (`cities/city.rs:3128`, from `building_attrition`,
    which rightly serves every building).
- **Absent**: the site-abandon disband. `damage_building` has no `0x2000`
  arm, and the bit itself is a SEAM (`crates/sim/src/ai/ai_make.rs:936`).
  Until both land, the crate under-counts `buildings_razed` and fires
  nothing for those sites.
- `last_razed` is not carried on this path either; it is outside this
  family.

### 2.4 Reached by a plain game

All four, in a skirmish against the computer:

- combat kills and razes;
- attrition and border-decay losses;
- the abandon arm, when a computer's orphaned unfinished site is struck
  to half its hits (computer players run `check_orphaned_buildings`).

Sniper, Commando and counterintel are skirmish units' spells; whether the
computer casts them is unread. The decoy-overflow self-credit (§2.5) needs
a scenario.

### 2.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/kills-*.md`); no diff backs it yet.
High confidence: every store and every test was settled at a listing
instruction.

The listing net found exactly 11 absolute sites (7 `incl` stores, 4
loads), no slot-biased forms, and only `reset_score` among the 140
register-relative stores to these offsets. `take_damage`'s callers through
vslot `+0x16c` are eight, and no vtable overrides that slot.

| leaf | confirmed / corrected / open |
|---|---|
| `units_killed` | 22 / 1 / 0 |
| `units_lost` | 12 / 0 / 0 |
| `buildings_razed` | 15 / 1 / 0 |
| `buildings_lost` | 13 / 0 / 0 |

Corrections that changed a rule:

- **The decoy-overflow self-credit is unreachable in a plain game.**
  After `die`, a decoy figure with whole points past its share strikes
  its captain with `param_8` = its own player. But
  `SpellType::cast_create_decoy (00674370)` sets `unit_masks |= 1` on
  every figure of the new squad, so the captain fails test 4 and nothing
  counts. The only other writer of that bit is the scenario unit loader
  (`ScenarioRead::load_unit_chunk`), so a mixed squad needs a scenario.
- **The abandon arm's vslot `+0x4c` means "finished"** (`flags & 4`), not
  "a city". The arm strikes an *unfinished* building. The city test is a
  different slot pair (`+0x20` and `flags & 0x20`).

Settled where a reading left it open:

- `do_damage`'s `this` from ammo and nukes is the shooter;
- every `Build` reaches `Wall::process`;
- `Build`'s `+0xac` is `this`.

No kept-dump value is cited.

### 2.6 Not established

- **Spell castability upstream.** When a counterintel or sabotage cast
  happens at all: `SpellType::cast`'s pre-check (the call at `0x6763c0`)
  and `is_valid_target`. This decides how often the spell writers fire,
  not what they count. A reading of those two settles it, or a capture
  where a player casts them.
- **Whether the computer casts Sniper, Commando or counterintel** in a
  skirmish. This is unread, and its AI scripts or a capture settle it.
- **No value diff.** The rules rest on readings alone until a port's
  checksum arms meet the leaders line of a kept run. Any capture with a
  kill, a raze, an attrition death and a border-decay loss on its frames
  exercises all four.
