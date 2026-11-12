## 15. Missiles, repairs, bribes and nukes

### 15.1 Leaves

- `repair_stamp` (+0x208), `int`: the frame a repair was last refused.
- `missiles_used` (+0x7c0), `int`: V2 and Cruise Missile rounds landed.
- `units_bribed` (+0x8f4), `int`: units taken by Bribe.
- `nukes_launched` (+0xa38), `int`: nukes out holding a strafe.

Each leaf has exactly one event writer; nets over the whole listing found
no other. All four lie inside the header's bulk writers (and `Leader::close
(006b7f80)`'s memset, which has no direct caller).

### 15.2 The rule

**`repair_stamp`.** Writer `Unit::do_repair (005ee420)`, index the
repairer's owner (`unit+0x9`, a byte). Event: a repair step that is
refused. Before charging, `do_repair` walks the six goods `g` in `0..=5`. A good
counts when it is `type_avail(g, 1)` and has a nonzero price. Each counted
good has a due of 0 or 1 this step (1 when the truncated quotient before the
step is below the one after it; the float `due` is the repair mechanic's,
item 3346). The first counted good whose stock `< due[g]` (signed) refuses
the step:

1. kill the repairer's current order;
2. if `frame − repair_stamp[owner] ≤ 150` (signed, wrapping), return;
3. else `repair_stamp[owner] = frame`;
4. then the refusal message, for the console player only.

The stamp is written for every player, AI or human, before the console
test. It is an assignment, not a step. A refused call charges nothing (the
charge loop is reached only past the goods test). Edge: because `due` may be
0, a priced good with a **negative** stock refuses even when this step owes
none of it. Keep the comparison signed and against `due`.

Reset: 0 at `Leader::init (006e3930)`, an explicit store on both arms
(the memset runs only when its second argument is negative). Nothing
mid-game resets it; `reset_score` does not. Readers: `do_repair`'s
throttle and `log_data` only.

**`missiles_used`.** Writer `Ammo::do_damage (00678060)`, index the round's
`who` (`AmmoData+0x3c`, the shooter's player at launch). The path, in order:

1. Guards: object `o ≥ 0`, `who ≥ 0`, graphic-piece type `≠ 9`.
2. Splash arm only: `splash_area ≠ 0`.
3. The shooter object (`objects[who][o]`, the missile itself) has a type
   with `obj_masks & 0x8000000`; else straight to `hit_target`, no count.
4. Note `is(0x13b, 0)` (the Nuclear Missile line, non-strict).
5. Shield: find the landing tile's owner `L` (a signed byte). On an ocean
   tile, `L` is the first owned (`≥ 0`), on-map, non-ocean cell of a
   120-entry offset ring; none found leaves −1. If `L ≥ 0`, `L` holds
   `MISSILE_DEFENSE_BONUS` (693) and `L ≠ who`, the round closes, no count.
   The test is `≠`, not "enemy": an ally's shield stops it too.
6. A Nuclear Missile line shooter takes the nuke arm and returns, no count.
7. Otherwise the `S_NUKE_HIT` sound, then `missiles_used[who] += 1`, then
   `hit_target` and the splash.

Reset: 0 at `Leader::init`; never decremented. Reader: `get_support_count
(006da110)`, feeding `TypeData::get_cost (00664090)`. On its by-type
fall-through, for exact type `0x139` (`V2ROCKET`) or `0x13a`
(`CRUISEMISSILE`), it adds `missiles_used[player]` to the count. So each
landed V2 or Cruise round makes that player's next one dearer. The term
sits after the group-count arm (types with `progression & 1` and
`+0x1e8 ≠ 0` return first). In the shipped `unitrules.xml` both missiles
have `PROGRESSION 0`, so the term is live. The Americans' bomber discount
below it does not touch them.

**`units_bribed`.** Writer `SpellType::cast_bribe (00672d00)`, index the
caster's player (`param_2`). Exemptions first, none of which counts:

- own target (`param_2 == param_4`): a separate arm;
- an ally's unit (the target is the caster's `who`, or relation 2 both
  ways), unless teams are locked (`team_style ∉ {0, 8, 11}`) **and** the
  target leader lacks `leader_flags & 2`: `unpay_cast_costs`;
- a unit with `has_general(target, 0, 0x165) ≥ 0` (CEO cover):
  `unpay_cast_costs`;
- a non-unit target: the building/city arm, which never counts.

A unit past these meets no further test. Every message branch joins at one
point: `units_bribed[caster] += 1`, then `Unit::swap_team` (vslot `0x94`).
The count precedes the swap.

Reset: `Leader::reset_score (006e37f0)` stores 0. `Leader::init` calls it
(and has no store of its own), and `Setup::build_game (005ac190)` calls it
again for every slot with `leader_flags & 1`, inside its `param_2 == 0`
block. Never decremented. Readers: `AchieveChartPane::set_chart_data
(007f2c40)` (UI) and the checksum; no sim decision, not in `log_data`.

**`nukes_launched`.** Writer `Unit::come_out (00617c10)`, index the unit's
owner (`unit+0x9`). After placement, it counts only when all of these hold:

1. the unit `is(0x13b, 0)` (Nuclear Missile line);
2. its order list head `+0xdc` and current order `+0xcc` are non-null;
3. the current order's `get_strafe_order` (`UnitOrder` vslot `0x100`) is
   non-null, which only a `StrafeOrder` answers (every other order's slot is
   the folded zero, from the PE vtable bytes and RTTI);
4. that order's `ox` (+0x8) and `whom` (+0xc) are both `≠ −1`.

Then `nukes_launched[owner] += 1`. Every caller of `come_out` is a site: the
silo launch, ejections, the contents of a killed holder.

Which nukes qualify: `Build::do_missile_launch (00622670)` calls `come_out`
for every popped missile; its order test gates only the announcement and
sound (and its `STRAFE` arm is dead, as `is_ally(leaders[p], p)` is 1). And
`add_strafe_order (005e48c0)`, the only maker of a `StrafeOrder` (plus
`copy_order (0072f900)`, which copies one that exists), turns a strafe by a
bit-27 unit at a target `Object::valid_target (00648ba0)` accepts into
`add_air_attack_ground_order`. A Nuclear Missile has bit 27. So **a nuke
counts only when it was ordered to strafe an object that
`ObjectData::valid_target_const (006472c0)` refused at order time**: a
non-enemy, a slot outside `0..7`, an inactive, off-map or wrong-domain
target. A nuke ordered at a valid enemy, or at the ground, holds
`AirAttackGroundOrder` and does not count. `do_garrison`'s strafe passes
`−1,−1` and fails step 4.

Reset: 0 at `Leader::init`; never decremented. Reader: `log_data`.

### 15.3 In the crate

- **`repair_stamp`.** The event is `do_repair_steps`
  (`crates/sim/src/orders/jobs.rs:1802`), the throttle at `:1915`
  (`frame.wrapping_sub(feedback_stamp) < 0x97`, the same test), the signed
  `due` comparison at `:1912`, owner index at `:2102`. Storage is a stub:
  `SimDoRepair::feedback_stamp`/`set_feedback_stamp` at `:2105`–`2110`.
  **Trap:** the stub returns `i32::MIN`, which makes the throttle **always
  return** before the store, not "always due" as its SEAM says. Harmless
  while both arms are empty. Port: a per-player `i32`, 0 at init, behind
  those two methods.
- **`missiles_used`.** The event is `Sim::land`
  (`crates/sim/src/combat/fight/projectiles.rs:560`): shield at
  `:608`–`614`, nuke arm at `:617`–`620`. Put the `+1` keyed `p.owner`
  after the `is_nuke` return, under `mask::MISSILE`, before the splash
  walk. Storage beside `Nukes::used` (`combat/nuke.rs:76`). The reader is
  `nuke_ramp_extra` (`economy/pricing.rs:281`–`300`), whose doc says the
  term is unmodelled: add `if t == 0x139 || t == 0x13a { extra +=
  missiles_used[who] }`, on the by-type count only, where it already feeds
  (`of_type`, `:851`–`855`). **Trap:** `crates/sim/src/ids.rs` names
  `0x139` `CRUISE`, but `0x139` is `V2ROCKET` and `0x13a` `CRUISEMISSILE`
  (`air/airbase.rs:42`/`48` have them right). Key on the numbers.
- **`units_bribed`.** Already counted: `tech[a].bribes += 1` at
  `crates/sim/src/spells/bribe.rs:42`, field `PlayerTech::bribes`
  (`tech.rs:771`), test `spells/cast/tests.rs:544`. Not wired: the value
  match in `checksum/leaders.rs` falls to `_ => None` (`:1305`). Add
  `"units_bribed" => one(i64::from(self.tech[w].bribes))`. Gap:
  `Sim::cast_on` (`spells/cast.rs`) re-tests only same owner and CEO cover,
  not the ally refusal. The order-time `bribe_arm`
  (`spells/spell_valid_target_view.rs:183`–`186`) carries the same ally
  rule, so the gap shows only when an alliance forms between order and
  cast.
- **`nukes_launched`.** The event is `Sim::come_out`
  (`crates/sim/src/units/garrison.rs:376`). Port after placement, gated on
  `air_line_is(u, NUCLEARMISSILE)` and `Body::Strafe` with `target:
  Some(_)` (`StrafeOrder::target` is `ox/whom`, `orders/order.rs:498`–`501`).
  Already faithful: `do_missile_launch` (`air.rs:1799`) calls `come_out`
  unconditionally (`:1825`), and `add_strafe_steps`
  (`orders/order_list.rs:1854`, `:1873`) converts a missile's valid strafe.
- **Checksum.** Rows `checksum/leaders.rs:74`, `:141`, `:190`, `:259`. Each
  port removes its `("<leaf> (unheld)", 2)` entry from `LEADERS_HELD_OUT`
  (`crates/rondata/src/diff/checksum_lines.rs:1814`, `:1816`, `:1821`,
  `:1846`).

### 15.4 Reached by a plain game

- `repair_stamp`: yes, any repairer short of a charged good after frame 150.
- `missiles_used`: yes, from the Industrial age with V2 or Cruise Missiles
  landed.
- `units_bribed`: yes, Bribe on an enemy unit.
- `nukes_launched`: the writer runs on every `come_out`, but the predicate
  needs a nuke holding a strafe at a refused target, which an ordinary
  launch never carries. Expect 0 in plain play until a capture says
  otherwise. `ScenarioFuncSet::add_unit` (triggers) is among
  `add_strafe_order`'s callers, out of plain play.

### 15.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/odd-counters-*.md`); no diff backs it yet.

Verdicts by leaf (confirmed / corrected / open): `repair_stamp` 15 / 1 / 0,
`missiles_used` 16 / 0 / 0, `units_bribed` 15 / 1 / 0, `nukes_launched`
10 / 2 / 1. The corrections that changed a rule:

- `do_missile_launch`'s order test gates the announcement, not the
  release; `come_out` runs for every popped missile.
- A nuke strafing a valid object does not keep a `StrafeOrder`:
  `add_strafe_order` converts it to an air-attack-ground. This is what
  makes `nukes_launched` near-dead in plain play.

The other two corrections changed no rule: `Leader::init`'s memset is
conditional (the explicit store zeroes `repair_stamp` either way), and
Bribe's own-target arm draws down the caster's `+0x96`, not the target's
(not a count). Added by the adjudication: the negative-stock edge of the
repair refusal, and the shipped `PROGRESSION 0` of both missiles, which
keeps the `missiles_used` price term live.

Confidence is high for every writer and predicate, each read from the
listing over its branch (`5ee99f`..`5eeaa4`, `678186`..`67861b`, every
route into `67306a`, `61883d`..`6188fa`), and `get_strafe_order` from the
PE vtable bytes and RTTI. No kept dump was consulted.

### 15.6 Not established

- **Whether any plain-game path gives a nuke a strafe at a refused
  target** (the console's right-click on a non-enemy, `Group::action_flight`'s
  `NUCLEARMISSILE` arm, the AI's nuke choice); not read. A capture with
  nuke launches read under `log_data`'s `nukes_launched` settles it: a
  value above 0 after launches at enemy targets refutes the reading.
- **Which types carry `obj_masks` bit 27 outside the nuke line**, and
  **whether a missile ever lands more than one round per detonation** (both
  readings assume one). Data, held in the crate's `mask::MISSILE`. A
  capture with a V2 landing and a later V2 priced settles both.
- **Whether a stock can go negative**, which makes the repair edge
  reachable. The economy's question; the charge clamps only the second
  ledger.
- **`hit_target` order.** The crate calls it at `projectiles.rs:577`,
  before the shield and nuke tests; the original only after them. It
  matters only if it has side effects on a round that then dies; not
  adjudicated.
- **Bribe's own-target arm** skips `unpay_cast_costs` while the crate's
  refuses with `unpay_cast_mana`; not adjudicated, the bribe owner's.
