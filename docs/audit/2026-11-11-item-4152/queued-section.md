## 14. The six `*_queued` counters and `scholar_militia`

### 14.1 Leaves

- `scholar_militia`, +0x9f0, `int`.
- `barracks_queued` +0xa10, `stable_queued` +0xa14, `factory_queued`
  +0xa18, `combat_queued` +0xa1c, `dock_queued` +0xa20, `air_queued`
  +0xa24, all `int`.

All seven are zeroed by `Leader::init (006e3930)` for a tribe `>= 0` only
(`6e4c14` and `6e4c3e..6e4c5c`), never recounted, and restored raw by a load.

### 14.2 The rule

#### `scholar_militia`

**Writer.** One live store, at the end of `Leader::track_unit_type
(006e0dd0)` (`6e0ecd`). There is no inlined copy. It is reached from six
call sites: `Unit::set_type` twice, `Unit::close`, `Objects::init_unit`
twice and `SpellType::cast_create_decoy`. Each call site guards it the same
way: `unit_masks & 1` clear, the unit its own captain (`+0x8e < 0`), and
`control_cost != 0`, or `is(FIGHTERBOMBER)`, or a government hero.

**Step**, at every `track_unit_type(type, delta, uid)` that passes:

```
if uid >= 0 && type.is(MILITIA 0x42) {
    f = units[who][uid].former_type      // +0x54, signed
    if f >= 0 && types[f].is(SCHOLARS 0x34) { scholar_militia += delta }
}
```

There is no floor and no clamp. A militia that fails the Scholar test leaves
the function; only a non-militia or `uid < 0` goes on to the SPECIALFORCES
arm.

**Events.** +1 at a Scholar's To Arms; -1 at its Back to Work and at its
death.

**`former_type` (+0x54) lifecycle.** A port that keeps the counter needs
this:

- `cast_to_arms` sets it to the current type before the retype.
- `cast_civilian` sets it to Citizen only if it is 0, and otherwise keeps it.
- `cast_immortal_hinf (00670600)` and `cast_immortal_arch (00670740)` set
  it to the current immortal type.
- Death leaves it.
- `Unit::init (00612100)` zeroes it (`6121f6`) only **after** the birth +1.

**The stale-slot +1.** A unit born through `Objects::init_unit (0065e0c0)`
reaches `Unit::init`. That calls `Object::init`, then `SubObject::init`,
then `Unit::set_type` under `SET_TYPE_INIT` (`612182`). The birth +1 runs
there against the slot's **previous** occupant's `former_type`, because the
clear at `6121f6` comes later. `Unit::close` does not clear `+0x54`, and it
clears bit 0 of `unit_masks` (`60fa35`), so the stale +1 passes the mask
guard. `Objects::find_free (0065ad60)` hands
out the lowest free slot, so slots recycle constantly. A dead Scholar
militia leaves a Scholar id behind, and so does a Scholar that ever went to
arms and back. Two plain-game routes create a militia-typed unit this way:

- `SpellType::cast_create_decoy (00674370)` copies militia (its exclusions
  are `0x32..0x35`, caravans and `is(MERCHANT)`). Its own undo
  `track_unit_type(type, -1, new)` runs after `+0x54` is 0 and misses. It
  then sets `unit_masks |= 1`, so the decoy's death never takes the +1
  back. Net +1 for the life of the game.
- `Unit::swap_team (006118d0)` (from `cast_bribe`, `cast_assimilate`,
  `Leader::defeat_by`) re-creates the unit in the new owner's slot and does
  not copy `+0x54`. A bribed militia counts +1 for its new owner exactly
  when that slot is stale with a Scholar id, and its later -1 sees 0: a
  permanent +1. A genuine Scholar militia that is bribed loses its Scholar
  status. The old owner's -1 at close is correct, and the new owner never
  counts it.

Conversions in place (`cast_to_arms`, `Object::eject_contents (0064cd20)`'s
Citizen-to-militia turn) and the barge spells are not at risk. A port needs
the lowest-free-slot reuse modelled.

The immortal spells cannot strand the count. Their -1 is for the immortal
type, which does not answer `is(MILITIA)` (unless the shipped `is_list` says
otherwise).

**Readers.** `TypeData::get_cost (00664090)` subtracts it from the
Citizen's count and adds it to the Scholar's. `LeaderData::log_data
(006e5110)` logs it.

#### The six `*_queued` counters

**The rule**, `track_queued(who, t, d)`, out of line in
`Leader::track_queued (006e0f30)`:

1. If `t < 0`, stop.
2. Step `num_queued[t]` (u16) by `d`, unless it is 0 and `d < 0`.
3. Require `0x32 <= t < 0x19e` (`TypeData::is_unit_type (004707e0)`, the
   only implementation of vtable slot `+0xc`) and `attack (+0x1e8) != 0`.
4. Choose by `w = where (+0x40)`, by equality:
   - `0x1ab`: barracks and combat;
   - `0x1ac`: stable and combat (not Auto Plant `0x1ad`);
   - `0x1ae`: factory (Siege Factory, not `FACTORY 0x1af`);
   - `0x1b0`: dock (not `0x1b1` or `0x1b2`);
   - otherwise: air if `domain (+0x218) == 2`, else nothing.
5. For each chosen counter `c`: `if !(c == 0 && d < 0) { c += d }`, each
   tested on itself. A skipped barracks or stable step still runs combat's
   own test.

**Never a clamp.** The guard skips a negative step only at exactly 0. A
`-n` from `0 < m < n` gives `m - n < 0`. Every inlined `-1` copy tests only
`== 0` (then `decl`), so a negative counter keeps falling on each `-1`. The
`+1` copies are a bare `incl`.

**Writers.** These are inlined copies of the rule, devirtualized to
`0x32 <= t < 0x19e`, and the out-of-line callers:

| site | `d` |
|---|---|
| `Build::queue_up (00620f40)`, queue append | +1 |
| `Group::action_spell (006fe1a0)`, BUILDAIR | +1 |
| `Build::unqueue (006207c0)`, completion and cancel | -1 |
| `Build::clean_queue (00620b60)`, per entry, last first | -1 |
| `Unit::action_unqueue (005e1f20)`, carrier plane cancelled | -1 |
| `Unit::process (00610bc0)`, carrier plane finished (absolute stores) | -1 |
| `Unit::close (0060ee50)` `60efed`, carrier lost with `n` queued, then its `num_queued (+0xa0) = 0` | -n on `current_upgrade(FB)` |
| `Leader::gain_tech (006dcb60)` `6dda28`/`6dda47`, FB-line upgrade, per live carrier | -a0 on old FB, +a0 on new FB |
| `Leader::gain_tech` `6ddd04`/`6ddd4a`, unit-line upgrade, per queue entry | -1 on the entry's type, +1 on the new |

On the retarget's graft-chain arm the -1 is the **new** type's, so
`num_queued[old]` stays one high. Within one bin it nets to zero on the six.
`Build::unqueue` is called from `Build::do_queue` (completion),
`Build::action_unqueue` (cancel) and three `Build::activate (00623e20)`
sites. The four `Wall::*` copies are dead: a wall's type is a build type.

**`air_queued`** is the only leaf with `-n` writers. Each carrier +1 also
stepped it, so `air_queued >=` every carrier's `num_queued` in steady
state, and going below zero needs an earlier drift not found in a plain
game.

**Readers.** `LeaderData::get_support_count (006da110)` takes five bins
(`queued + units`, gated on `+0x2f4 & 1` and attack; no combat arm) and
feeds `TypeData::get_cost`. `log_data` logs all six; it is
`combat_queued`'s only reader.

### 14.3 In the crate

- **`scholar_militia` is derived, not kept.** `Sim::worker_support`
  (`crates/sim/src/economy/pricing.rs:895`) counts live units with
  `type_index` in `0x42..=0x44`, `rare` in `{0x34, 0x35}`, `captain`,
  `!decoy`. That cannot carry the stale +1. Those ranges are stand-ins for
  two kind tests whose `is_list` data was not opened. The kept counter goes
  in `Sim::track_unit_type` (`crates/sim/src/units/store.rs:755`), with the
  unit index added, and is exposed to `Sim::leader_long`
  (`crates/sim/src/checksum/leaders.rs:949`).
- **The queued bins.** `Muster::queued_by_group` is moved by
  `Sim::track_tree_queued` (`crates/sim/src/cities/queue.rs:2017`) and
  `Sim::untrack_queued` (`:2078`), and retargeted by
  `Sim::retarget_queued_to` (`:1964`). Carriers are in
  `crates/sim/src/air/carrier_queue.rs` (`carrier_action_spell :60`,
  `carrier_queue_tick :137`, `unit_action_unqueue :167`). `untrack_queued`
  tests `!= 0` on `queued_by_type`, `tech.queued` and `queued_by_group`
  separately: the original's skip at 0, not a clamp.
- **SEAM: no `-n`.** `Unit::close`'s `-num_queued` and `gain_tech`'s
  carrier pair are missing (named in `Sim::upgrade_units_to`,
  `cities/queue.rs:1890`). Both land in `air_queued`.
- **Trap: derive from the bins, not from `queued_by_type`.** After a
  graft-chain retarget, `num_queued[old]` stays one high and a sum over
  types carries it; the bins do not.
- **Trap: bin shape.** The crate keys bins per `where_` (`load.rs`:
  `group = if attack != 0 { where_ }`). The original has exactly four
  `where` bins and an air fallthrough on `domain`. Whether they coincide on
  the shipped data is open. A separate `air_queued` counter is the safer
  port, since several `where_`s may train armed air types.
- **`combat_queued`** has no crate home. Port it as a counter with its own
  `!= 0` test at the barracks/stable sites. Or port it as
  `queued_by_group[BARRACKS] + queued_by_group[STABLE]`, which is exact
  while all three move by paired ±1 from zero. That holds in a plain game,
  because `-n` is FB-only and FB is air.
- **Checksum.** `Sim::leader_long` has no arm for the seven. Each is
  `("<leaf> (unheld)", 2)` in `LEADERS_HELD_OUT`
  (`crates/rondata/src/diff/checksum_lines.rs`: `scholar_militia :1822`,
  `barracks_queued :1766`, `stable_queued :1836`, `factory_queued :1786`,
  `combat_queued :1773`, `dock_queued :1783`, `air_queued :1754`). A port
  adds the arm to `leader_long` and removes the entry.

### 14.4 Reached by a plain game

Yes, for every live writer; carriers and the FB upgrade in the Modern
ages. The
stale `scholar_militia` +1 is reached through a General's decoy over a
militia, or a Spy's bribe of a militia, landing in a slot that last held a
Scholar id. Scenario and cheat creators (`ScenarioFuncSet::add_unit`,
`ScenarioEditor::add_unit`, `Game::action_cheat_init_unit`) are out of
plain play. `air_queued` below zero is legal code but, as far as read,
unreached in a plain game.

### 14.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/queued-*.md`); no diff backs it yet.

Verdicts: `scholar_militia` 13 confirmed, 1 corrected, 1 open; each queued
leaf 16 confirmed, 1 corrected, 1 open. The writer nets over the listing
found 75 stores in exactly 12 functions for the six, and 2 leader stores
for `scholar_militia`. No writer was missed. The corrections:

- **The floor.** "Never below zero" was corrected to "skip a negative step
  at exactly 0". A negative counter is not floored by the `-1` copies. This
  changes the rule.
- **The immortal spells** cannot strand `scholar_militia`. No change to the
  rule.

Confidence is high on writers, predicates and steps (from the listing:
`6e0ecd`, `6e0f97..6e0fa7`, `6209d7`), and high on the stale +1 mechanism
(each link read). It is medium on below-zero `air_queued` being unreached,
and medium on which buildings train `domain 2` types (data). No kept-dump
value is cited.

### 14.6 Not established

- **How often a plain game meets the stale `scholar_militia` +1.** A
  capture settles it: a Scholar sent to arms and killed, then a General's
  decoy over a militia, reading `scholar_militia` from the leaders line
  before and after (expect +1). The same probe settles whether the `+0x8e`
  captain guard passes on a recycled slot. `+0x8e` is not reset before
  `Object::init`, so the read predicts it passes when the previous
  occupant was its own captain.
- **`Build::activate`'s cross-queue cancel.** Does the cancelled item
  (`BuildData::find_in_queue` after `BuildData::count_queue`) ever have an
  attacking unit type? Only then does it step the six. Reading
  `Build::activate`'s item origin settles it.
- **Data not opened.** Each of these is settled by reading the shipped
  table:
  - the `is_list` of MILITIA and SCHOLARS (the crate's `0x42..=0x44` and
    `{0x34, 0x35}`);
  - whether an Immortal type's `is_list` names MILITIA;
  - whether any building trains a Militia-line type through `Build::train`;
  - the `where`/`domain` columns that say whether the crate's per-`where_`
    bins coincide with the original's four bins and air fallthrough.
- **A plain-game drift of `air_queued` below a carrier's count**, the only
  way to below zero. None was found. A capture that loses a carrier with
  planes queued, diffed on `air_queued`, would witness the `-n`.
