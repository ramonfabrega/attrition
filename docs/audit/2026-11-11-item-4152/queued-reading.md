# Item 4152, family 14 (`queued`): first reading

- **Family**: 14, slug `queued`.
- **Leaves**: `scholar_militia` (`LeaderData` +0x9f0); `barracks_queued`
  (+0xa10), `stable_queued` (+0xa14), `factory_queued` (+0xa18),
  `combat_queued` (+0xa1c), `dock_queued` (+0xa20), `air_queued` (+0xa24);
  all `int`.
- **Reader**: Opus 5.5 (claude-opus-5-5), wave 1 first reading, 2026-11-11.
- **Sources**: the decompile export (`~/ghidra-projects/decomp`), the listing
  of `riseofnations.exe` (`llvm-objdump`), and `crates/sim`.

`Leader` is 0x6eec bytes and `LeaderData` 0x6ee4 (`types.txt`), so a
`Leader*` reaches a `LeaderData` field at that offset + 8: the family sits at
`Leader` +0x9f8 (`scholar_militia`) and +0xa18 … +0xa2c (the six queues).
In the global array the slot-`i` `LeaderData` is at `0xe3a390 + i*0x6eec`.

**The `Leader` vs `LeaderData` offset (settled first).** The size difference
is 8, but it is a *suffix*, not a prefix: `Leader::track_unit_type (006e0dd0)`
stores `barracks_units` .. `air_units` at `0x9f8(%esi)` .. `0xa0c(%esi)` with
`esi = this` (`Leader*`), and `Leader::track_queued (006e0f30)` stores the
six queues at `0xa10(%edi)` .. `0xa24(%edi)`; both read `who` at
`0x8(%esi)` and multiply it by `0x6eec` to reach `leaders.list[who]`. So a
`Leader*`'s `field_0xNNN` **is** `LeaderData +0xNNN`; no rebasing. The
listing net therefore searched `0x9f0`, `0xa10`..`0xa24` off any register and
the absolute `0xe3ad80`, `0xe3ada0`..`0xe3adb4`, plus `lea` of the same
displacements and every whole-record block move (`rep movs` / `memcpy` of
`0x6eec` or `0x6ee4`): none of the latter exists outside constructors and
diplomacy copies of other structures.

---

## Leaf 1: `scholar_militia` (`LeaderData +0x9f0`, int)

### 1. Writers, by both nets

| net | site | function | form |
|---|---|---|---|
| export `->scholar_militia` | none (only the two readers below print the name) | | |
| export `field_0x9f0` (Leader) | `track_unit_type`, last statement | `Leader::track_unit_type (006e0dd0)` | `+= delta` |
| listing | `6e0ecd addl %edi, 0x9f0(%esi)` | `Leader::track_unit_type (006e0dd0)` | `+= delta` |
| listing | `6e4c14 movl %eax, 0x9f0(%ebx)` (`eax = 0`) | `Leader::init (006e3930)` | `= 0` |
| listing | `LeaderData::walk_data (006d6750)` walks the byte range `[&who, +0x692a)` | save/load walker | raw restore on load |

Every other `0x9f0(%reg)` store in `.text` (`TerrainMap::*`,
`Constants::init`, `GraphicPieces::*`, `ConquestSetupWin::*`, `Game::do_frame`'s
`rep movsl` into `0x9f0(%ebx)` with `ebx = Game this`, `ScenarioEditor`) was
checked for its base: none is a `Leader*`. `Game::run_solo`'s
`cmpw $0, 0x9f0(%edx)` is a compare, not a store. **No inlined copy of the
`scholar_militia` arm exists**: the only `+= delta` is the one in
`track_unit_type`, so every step goes through that function's callers.

`track_unit_type`'s callers (both nets agree: the export's call names and the
listing's `calll 0x6e0dd0`):

| caller | site | delta | uid argument |
|---|---|---|---|
| `Unit::set_type (00612fa0)` | `613097` | −1, old type | the unit's own id (`+0xa`) |
| `Unit::set_type (00612fa0)` | `61361f` | +1, new type | own id |
| `Unit::close (0060ee50)` | `60f42a` | −1, current type | own id |
| `Objects::init_unit (0065e0c0)` | `65e228`, `65e4e4` | −1 | the new unit's id |
| `SpellType::cast_create_decoy (00674370)` | `67473b` | −1 | the decoy's id |

### 2. Event, step, predicate

`scholar_militia` counts **live militia that were Scholars before a To Arms**.
The arm, in my words, from the listing `6e0e87`..`6e0ed3`:

- `uid >= 0` (the `param_3` push), **and**
- the unit type being tracked answers `is(MILITIA = 0x42, strict = 0)` — the
  virtual `TypeData::is` slot `+0x60`, which for an object type is
  `ObjectTypeData::is (0065f7d0)`: the type's own id equals `0x42`, or `0x42`
  is in its `is_list`. So it is a *kind* test, not an id range: Militia,
  and whichever of Minuteman (`0x43`) and Partisan (`0x44`) list `MILITIA`
  among their kinds (the data was not read here), **and**
- `units[who][uid] +0x54 >= 0`, where `UnitData +0x54` is a union of
  `rare` / `air_alt` / `former_type` (`rise.pdb` field list, offset 84 for
  all three), **and**
- `types[former_type]` answers `is(SCHOLARS = 0x34, 0)`.

Then `scholar_militia += delta` (±1). If the type is not a militia (or
`uid < 0`), control goes instead to the `is(SPECIALFORCES = 0x4b)` arm,
which only toggles `leader_flags` bit `0x20000`; it never touches this leaf.

The events that reach it with a militia type:

- **To Arms** (`SpellType::cast_to_arms (00670880)`, from `SpellType::cast
  (00676ce0)` and the trigger function `ScenarioFuncSet::citizen_to_militia
  (00a00010)`): it writes `former_type = unit's current type id`, then calls
  `set_type(current_upgrade(MILITIA), SET_TYPE_NORMAL)`. In `set_type` the
  −1 runs for the *old* type (a Scholar is not `is(MILITIA)`, so the leaf
  is untouched) and the +1 runs for the militia type with `former_type`
  now the Scholar's id: **+1**.
- **Back to work** (`SpellType::cast_civilian (006704a0)`, from
  `Group::action_alarm (0070ec30)`, `SpellType::cast` and
  `ScenarioFuncSet::militia_to_citizen (00a000c0)`): `former_type` is kept
  (only a 0 is replaced, by the Citizen type) and the unit is set to
  `current_upgrade(former_type)`. The −1 runs for the militia type while
  `former_type` is still the Scholar: **−1**.
- **Death / removal** (`Unit::close (0060ee50)`): **−1**.
- **A militia line upgrade** applied to a live militia goes through
  `set_type(NORMAL)` too: −1 old militia, +1 new militia, net 0, provided
  both types answer `is(MILITIA)`.
- `Objects::init_unit`'s two −1 calls are the squad-follower undo (a member
  whose `+0x8e` captain index is set is removed from every count it was
  given at birth), and `cast_create_decoy`'s −1 undoes a decoy's count.
  Both act on a just-initialised unit whose `+0x54` was set to 0 by
  `Unit::init (00612100)` at `6121f6`, so `types[0]` (a good type, `FOOD`)
  answers `is(SCHOLARS)` false and the leaf is untouched — see the hazard
  below.

The guard both `set_type` sites and `Unit::close` share (decompile and
listing agree): `unit_masks` (`UnitData +0x68`) bit 0 clear, the unit is its
own captain (`+0x8e` negative, the default of the `is_captain` slot), and the
type has population (`+0x2f0 != 0`) **or** is `is(FIGHTERBOMBER = 0x134)` **or**
is a government hero (`+0x2b4 & 0x4000000`). `SET_TYPE_LOAD` (2) skips both
calls; `SET_TYPE_INIT` (1) runs only the +1; `SET_TYPE_NORMAL` (0) runs both.

**A hazard the predicate carries, found here and not confirmed by a run.**
`Unit::init (00612100)` calls `Object::init` (`612182`), which reaches
`SubObject::init (00662300)`'s `set_type(type, SET_TYPE_INIT)` (vtable slot
`+0x84` = `Unit::set_type`) and so the **+1 with the slot's old `+0x54`**;
only afterwards, at `6121f6`, does it store `former_type = 0`. A militia
type born into a recycled unit slot whose previous occupant left a Scholar
id in `+0x54` would therefore be counted, and its later −1 (seeing 0) would
not take it back: a permanent +1. It needs a militia *created* (not
converted) in a slot whose previous occupant was a scholar-born militia, or a
Scholar-typed `rare`/`air_alt` value; how often a plain game does that was
not measured. A builder that derives the count from live units (as the crate
does, §5) cannot reproduce it.

### 3. Reset

`Leader::init (006e3930)` at `6e4c14`, on the straight-line block that also
zeroes the five `*_units` and six `*_queued`; the block is skipped only by
the early return at `6e3fee` taken when the tribe argument is negative
(`Leaders::init (006ed850)` calls it that way for every slot; the real
per-game init is `Setup::build_leader (005abc70)` / `Setup::build_game
(005ac190)` with a tribe). `Leader::reset_score (006e37f0)` does not touch
it (no store in its range). Loaded saves restore it raw through
`LeaderData::walk_data (006d6750)` (`param_1->input != 0`). No per-frame
fold and no periodic recount: unlike `peasants`/`scholars`, no sweep
re-derives it.

### 4. Readers

- **`TypeData::get_cost (00664090)`, decision-changing**: in the unit ramp,
  a Citizen (`0x32`, Korean `0x33`) is priced from
  `support(self) + support(0x42) + support(0x43) − scholar_militia +
  support(0x44)`, and a Scholar (`0x34`, Korean `0x35`) from
  `support(self) + scholar_militia`. So a Scholar sent to arms keeps
  raising the Scholar price and not the Citizen price.
- `LeaderData::log_data (006e5110)` (`6ea4bb`, the leaders line).
- `Unit::think_spellcaster (005f27a0)`'s `0x9f0(%esi)` is off a types list
  (`0xc061a8`), not a leader: not a reader.

### 5. Crate site

`crates/sim/src/economy/pricing.rs` `Sim::worker_support` already applies
the `get_cost` term, but **derives** the count from live units each call
(militia `type_index` in `0x42..=0x44`, `u.rare` in `{0x34, 0x35}`, alive,
captain, not a decoy) instead of keeping it. Differences a builder should
weigh: the original's tests are kind tests (`is(MILITIA)`, `is(SCHOLARS)`),
not id ranges; it gates on the population / fighter-bomber / hero /
`unit_masks` predicate rather than `decoy`; and the derived form cannot carry
the stale-slot hazard above. A kept counter would sit beside
`Sim::track_unit_type` in `crates/sim/src/units/store.rs` (which already
mirrors `num_units`, `control`, `active` and the census civilians from the
same function), taking the unit id as the original's fourth argument.

### 6. Plain game?

Yes. To Arms and Back to Work are ordinary city spells (`SpellType::cast`)
open to a human in a skirmish, and `Group::action_alarm` reaches
`cast_civilian`; the computer's use of To Arms was not traced. The trigger
functions are a second, scenario-only route.

### 7. Confidence and not read

High on the writer set (both nets, one arithmetic site) and the arm's
predicate (listing read instruction by instruction). Medium on which militia
types answer `is(MILITIA)` and which scholar types answer `is(SCHOLARS)`: the
`is_list` data was not read. Not followed: the AI's caller of To Arms; how
`Group::action_alarm` decides. `Unit::init`'s vtable `+0x150` call at
`612131`, before `Object::init`, is `Unit::close (0060ee50)`, whose body
(`60ee50`..`60fa49`) has no store to `+0x54`, so it does not close the
hazard; `Objects::find_free (0065ad60)`'s decompile names no `+0x54`, and
other slot-recycling paths were not read for one. The hazard is a reading, not a capture.

---

## Leaves 2–7: the six `*_queued` counters (`+0xa10` .. `+0xa24`, int)

The six share one rule and one writer set, so the rule and the writers are
stated once here and each leaf's own branch follows in §2a–§2f.

### 1. Writers, by both nets

The export names none of the six (`->barracks_queued` etc. appear only in
the readers `LeaderData::get_support_count (006da110)` and
`LeaderData::log_data (006e5110)`). Every writer is a `field_0xa1N` /
`field_0xa2N` on `leaders.list[i]`, and the listing finds the same set: 77
register-relative stores and 6 absolute stores (`0xe3ada0`..`0xe3adb4` +
`who·0x6eec`). The rest of the `0xa10`..`0xa24` displacement hits are off
non-leader bases (`TerrainMap`, `Constants`, `GraphicPieces`, `TileSet`,
`ConquestSetupWin`, `HotKeyGroupOut`, `ScenarioEditor`, stack frames in
`ConquestGame::build_units` and `ScenarioFuncSet::add_unit`, and
`ConsoleWin::run_cmd`'s `0xa18(%ecx)`, off the global at `0xc06214` — see §7).

The rule (`track_queued`'s body) is **inlined at seven sites** besides the
function itself; each copy recomputes the same branch chain from the queued
type:

| writer | stores | delta | event |
|---|---|---|---|
| `Leader::track_queued (006e0f30)` | `6e0fa7`..`6e103b` | ±n, guarded | called by the two below |
| ↳ `Unit::close (0060ee50)` `60efed` | | `−num_queued` of the closing unit, type `current_upgrade(FIGHTERBOMBER)` | a carrier dies or is removed with fighters still queued; its `num_queued` (`UnitData +0xa0`) is then zeroed |
| ↳ `Leader::gain_tech (006dcb60)` `6dda28`/`6dda47` | | `−a0` old, `+a0` new | a FIGHTERBOMBER-line upgrade (`is(0x134)`): every live unit that answers `is(AIRCRAFTCARRIER = 0x15f, strict 1)` moves its own queued count from the old fighter-bomber (`local_20`) to the new |
| ↳ `Leader::gain_tech (006dcb60)` `6ddd04`/`6ddd4a` | | −1 old, +1 new | a unit upgrade retypes every building queue entry of the predecessor (see the note below) |
| `Build::queue_up (00620f40)` | `621a2f`..`621a79` (`incl`) | +1, no guard | a building accepts a unit into its queue (after `queued++` and `BuildQueue::set_queue`, beside `num_queued[t]++`); the leader is the building's owner (`+0x9`) |
| `Build::unqueue (006207c0)` | `6209dc`..`620a57` | −1, only if non-zero | one entry leaves a building's queue: a cancel (`Build::action_unqueue (00620280)` → vtable `+0x1c8`) **and** a completion (`Build::do_queue (0061e410)` → `+0x1c8`, two sites) |
| `Build::clean_queue (00620b60)` | `620d92`..`620e0d` | −1 per entry | the whole queue emptied, last entry first; callers `Build::close`, `Build::swap_team`, `City::close`, `Leader::victory`, `Leader::defeat`, and scenario `clear_build_queue` / `delete_leader` |
| `Unit::action_unqueue (005e1f20)` | `5e1ff3`..`5e206e` | −1, only if non-zero | a carrier's own queue gives back one `current_upgrade(FIGHTERBOMBER)` (a human cancel via `CommandPackage::process_unqueue (009466f0)`; `Build::train (0062f9b0)` on a newly trained carrier; scenario paths) |
| `Unit::process (00610bc0)` | `611362`..`6113dd` (absolute) | −1, only if non-zero | a carrier finishes a fighter-bomber (accumulator `+0x64` past `train_time`, pop cap clear), then `Objects::init_unit` |
| `Group::action_spell (006fe1a0)` | `6fe80c`..`6fe856` (`incl`) | +1, no guard | the BUILDAIR spell (`spelltypes[0x295]`) on a carrier with fewer than 10 queued and aircraft room: pays, `num_queued++` on the carrier, `num_queued[t]++` |
| `Wall::activate (0063e4b0)`, `Wall::close (0063f6e0)`, `Wall::increment_stats (00643270)`, `Wall::decrement_stats (00642da0)` | 25 stores | ±1 | **dead for this family**: the type each copy tests is the wall's own (`this +0x18 → +4`), a build type, and `TypeData::is_unit_type (004707e0)` — the only implementation of slot `+0xc` — is `0x32 <= id < 0x19e`, which no build type (`0x19e..0x21f`) meets. The `num_queued` short beside it is live; the family arm never runs |
| `Leader::init (006e3930)` | `6e4c3e`..`6e4c5c` | `= 0` | see §3 |
| `LeaderData::walk_data (006d6750)` | byte range | raw | save load |

**The guard is not a floor.** `track_queued` and every inlined decrement
skip a step only when the counter is **exactly 0 and the delta negative**;
a counter of 1 given −3 becomes −2. The inlined decrements are all −1, so
for them it is a true "never below zero"; `track_queued`'s −n (carrier
death, carrier upgrade) is the only way below zero, and only if the bin
holds fewer than the carrier's own queued count. The increments are
unguarded.

**The retarget note** (`gain_tech`, listing `6ddc4f`..`6ddd4a`, read here
and agreeing with the crate's `retarget_queued_to`): for each queue entry
`q` that is a unit type, if `q == graft(new.from)` the decrement is `q`'s
(`esi = q` at `6ddcf4`). Otherwise the code walks `q`'s upgrade chain
(`+0x48`, through `get_graft`) until it reaches the new type or runs out;
on reaching it, `esi` holds the **new** type when `6ddcf4` runs, so the −1
is the new type's and `q` is never decremented. For this family the two
types sit in one bin in every case where they share `where`, attack and
domain, so the bin nets to zero (−1 new, +1 new, and the eventual completion
−1 new against the original +1 for `q`); the residue is `num_queued[q]`,
not this family. If an upgrade chain ever crossed bins, the bin would drift.

### 2. The rule (all six)

Given a queued unit type `t` and a delta:

1. `t >= 0` and `t` is a unit type (`0x32 <= t < 0x19e`); else nothing.
2. `unittypes[t].attack` (`UnitTypeData +0x1e8`) non-zero; else nothing.
   So civilians (Citizens, Scholars, Merchants, Caravans, Supply Wagons and
   any zero-attack type) never touch the family.
3. Branch on `unittypes[t].where` (`TypeData +0x40`, a `TypeIndex`, compared
   by **equality**, not by line):

| `where` | steps |
|---|---|
| `BARRACKS` (`0x1ab`) | `barracks_queued` and `combat_queued` |
| `STABLE` (`0x1ac`) | `stable_queued` and `combat_queued` |
| `SIEGEFACTORY` (`0x1ae`) | `factory_queued` only |
| `DOCK` (`0x1b0`) | `dock_queued` only |
| anything else, and `domain` (`UnitTypeData +0x218`) `== 2` (air) | `air_queued` only |
| anything else | nothing |

Each of the two stores in the first two rows carries its own guard: a
`barracks_queued` at 0 with a −1 is skipped while `combat_queued` beside it
is still decremented if non-zero (listing `6e0f97`..`6e0fdc`).

#### 2a. `barracks_queued` (`+0xa10`)
Steps for an attacking unit trained at the Barracks (`where == 0x1ab`).
Paired with `combat_queued` on every step.

#### 2b. `stable_queued` (`+0xa14`)
`where == STABLE (0x1ac)`. Paired with `combat_queued`. **Not** the Auto Plant
(`0x1ad`): a type whose `where` is the Auto Plant itself steps nothing.

#### 2c. `factory_queued` (`+0xa18`)
`where == SIEGEFACTORY (0x1ae)`, despite the name; a type whose `where` is
`FACTORY (0x1af)` steps nothing. Does **not** step `combat_queued`.

#### 2d. `combat_queued` (`+0xa1c`)
The sum of the Barracks and Stable arms only: siege, naval and air never
step it. Its guard is independent of the arm's.

#### 2e. `dock_queued` (`+0xa20`)
`where == DOCK (0x1b0)`. Anchorage (`0x1b1`) and Shipyard (`0x1b2`) as a
`where` step nothing. No `combat_queued`.

#### 2f. `air_queued` (`+0xa24`)
The fallthrough: `where` none of the four **and** `domain == 2`
(`ids::domain::DOMAIN_AIR` in the crate). In practice the carrier's
fighter-bombers (`Group::action_spell`, `Unit::process`,
`Unit::action_unqueue`, `Unit::close`, `gain_tech`'s carrier arm) and any
attacking aircraft queued at a building. No `combat_queued`.

Which shipped unit types have which `where` (whether every land attacker
names the base building of its line, e.g. `BARRACKS` rather than an upgraded
form) is a data question this reading did not open; the original compares
the field exactly as loaded.

### 3. Reset

`Leader::init (006e3930)` zeroes all six at `6e4c3e`..`6e4c5c`, in the same
block as `scholar_militia` (same early-return caveat for a negative tribe).
`Leader::reset_score (006e37f0)` does not touch them. No per-frame fold and
no periodic recount. Loaded through `LeaderData::walk_data`.

### 4. Readers

- **`LeaderData::get_support_count (006da110)`, decision-changing**, via
  `TypeData::get_cost (00664090)` (its only caller): for a unit type with
  `UnitTypeData +0x2f4 & 1` (progression by group) and attack, the count is
  `<bin>_queued + <bin>_units` by the same `where`/domain chain (barracks,
  stable, siege factory, dock, air). That count is the unit's price ramp:
  each one queued raises the next one's price. `combat_queued` is **not**
  read here (there is no combat arm in `get_support_count`).
- `combat_queued`'s non-log reader: none found. `BuildOut::make_options`'s
  `0xa1c(%ecx)` is off the spell-types list (`0xc0ab10`), and
  `GroupOut::validate_spell`'s `0xa10(%eax)` likewise; `ConsoleWin::tell_behind`
  (`0xa18(%eax)`, base `0xc06214`) and `IFaceMainBase::on_redraw`
  (`movswl 0xa20(%ebx)`) are UI bases, not leaders.
- `LeaderData::log_data (006e5110)`, all six (`6ea80d`..`6ea911`).

### 5. Crate site

`crates/sim/src/cities/queue.rs` already keeps the bin counts:
`Sim::track_tree_queued` (+) and `Sim::untrack_queued` (−1, "never below
zero"), with `Muster::queued_by_group` (`crates/sim/src/units/unit.rs`) as the
store and `Sim::retarget_queued_to` for `gain_tech`'s retarget. Two
differences from the original that a builder of these six leaves would
meet:

- **The bin key.** The crate's `UnitType::group` is
  `if attack != 0 { where_ } else { None }`
  (`crates/rondata/src/load.rs:1325`), one bin per `where` building, mapped
  through `build_tree`. The original has exactly five bins plus
  `combat_queued`, chosen by equality with four `TypeIndex` values and a
  domain fallthrough: a type whose `where` is not one of the four and is not
  air steps nothing, and every air attacker shares one bin whatever its
  `where`. Whether the crate's per-`where` bins coincide with the five on
  the shipped data was not checked.
- **`combat_queued` has no counterpart**: it would be
  `barracks + stable` on the same statements, with its own zero test.
- The carrier writers are in `crates/sim/src/air/carrier_queue.rs`:
  `Sim::carrier_action_spell` (BUILDAIR, +1 through `track_tree_queued`),
  `Sim::carrier_queue_tick` (`Unit::process`'s completion, −1 through
  `untrack_queued`) and `Sim::unit_action_unqueue` (−1). **Not found in the
  crate**: `Unit::close`'s `track_queued(current FIGHTERBOMBER, −num_queued)`
  for a carrier that dies with fighters queued, and `gain_tech`'s carrier arm
  (`−a0` old, `+a0` new); `grep -rn num_queued crates/sim/src` finds no
  writer of `Unit::num_queued` outside `carrier_queue.rs` and tests. A
  carrier sunk with queued fighters would leave `air_queued` high in the
  crate and not in the original.

### 6. Plain game?

Yes for all live writers. `Build::queue_up` / `Build::unqueue` /
`Build::do_queue` are every skirmish's training queue, human
(`CommandPackage::process_queue_up`, `process_unqueue`) and computer
(`Leader::produce_unit (006cb9e0)`). `Build::clean_queue` runs whenever a
building with a queue is destroyed or captured. The carrier writers need an
Aircraft Carrier and the BUILDAIR order, which a skirmish reaches in the
Modern ages. `gain_tech`'s two arms run on every unit-line upgrade. The
`ScenarioFuncSet` / `ScenarioEditor` callers are a second route only. The
`Wall::*` copies never step the family.

### 7. Confidence and not read

High on the writer set (both nets agree; every store attributed)
and on the rule (listing read for `track_queued`, `queue_up`, `unqueue`; the
decompile for the rest, whose branch chains are byte-identical in shape).
`ConsoleWin::run_cmd (007d6a70)`'s `movl %eax, 0xa18(%ecx)` at `7da495`
loads `ecx` from the global at `0xc06214` (the same base
`ConsoleWin::tell_behind` reads `0xa18` from), not from the leaders array:
not a writer. Every store in the net is therefore attributed. Not read: `Build::do_queue`'s
two `+0x1c8` call sites line by line (I take them as the completion path
from their placement after `train_time`); `Build::train`'s
`Unit::action_unqueue(1)` on a newly trained carrier (if the new carrier's
`num_queued` is 0 the call returns at once, which I did not confirm);
`Leader::produce_unit`'s path into `queue_up`; the unit data's `where`,
`attack` and `domain` columns.

---

## Against the README row

The row lists `Leader::track_queued` and `Unit::process` for the six queued
counters, and `track_unit_type`'s militia-from-a-Scholar arm for
`scholar_militia`. All three are confirmed. The row leaves out writers:

- six further **inlined** copies of the queued rule: `Build::queue_up
  (00620f40)` (+1), `Build::unqueue (006207c0)` (−1, cancel and completion),
  `Build::clean_queue (00620b60)` (−1 per entry), `Unit::action_unqueue
  (005e1f20)` (−1), `Group::action_spell (006fe1a0)` (+1, BUILDAIR), and
  four `Wall::*` copies that exist but cannot step the family;
- `track_queued`'s own callers, `Unit::close (0060ee50)` (−n) and
  `Leader::gain_tech (006dcb60)` (four sites);
- `Leader::init (006e3930)` (zero) and `LeaderData::walk_data (006d6750)`
  (load) for all seven.

"Each its own zero floor" is not quite right: the guard skips a negative
step only when the counter is **exactly** 0, so `track_queued`'s −n can take
a counter below zero; only the −1 copies behave as a floor.

---

## Summary

| leaf | writers (Name (addr)) | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `scholar_militia` +0x9f0 | `Leader::track_unit_type (006e0dd0)`, from `Unit::set_type (00612fa0)` ×2, `Unit::close (0060ee50)`, `Objects::init_unit (0065e0c0)` ×2, `SpellType::cast_create_decoy (00674370)` | ±1 when the tracked type `is(MILITIA)` and the unit's `former_type` (`+0x54`) `is(SCHOLARS)`: +1 at To Arms, −1 at Back to Work and at death | `Leader::init (006e3930)` (tribe ≥ 0); load | derived, not kept: `Sim::worker_support`, `economy/pricing.rs`; a kept one beside `Sim::track_unit_type`, `units/store.rs` | yes (To Arms / Back to Work) | high on writers and arm; medium on which types answer the two kind tests; a stale-slot +1 hazard is a reading only |
| `barracks_queued` +0xa10 | `Leader::track_queued (006e0f30)` (callers `Unit::close`, `Leader::gain_tech`); inlined in `Build::queue_up (00620f40)`, `Build::unqueue (006207c0)`, `Build::clean_queue (00620b60)`, `Unit::action_unqueue (005e1f20)`, `Unit::process (00610bc0)`, `Group::action_spell (006fe1a0)` | ±1 per queued attacking unit with `where == BARRACKS (0x1ab)`; with `combat_queued` | `Leader::init`; load | `Muster::queued_by_group` via `track_tree_queued` / `untrack_queued`, `cities/queue.rs` (bin keyed by `where`) | yes | high |
| `stable_queued` +0xa14 | same set | `where == STABLE (0x1ac)`; with `combat_queued` | same | same | yes | high |
| `factory_queued` +0xa18 | same set | `where == SIEGEFACTORY (0x1ae)` (not `FACTORY 0x1af`); no combat | same | same | yes | high |
| `combat_queued` +0xa1c | same set | Barracks + Stable arms only, own zero test | same | **none**: no counterpart in the crate | yes | high |
| `dock_queued` +0xa20 | same set | `where == DOCK (0x1b0)` | same | same as barracks | yes | high |
| `air_queued` +0xa24 | same set (carrier writers dominate) | `where` none of the four and `domain == 2` | same | same; `air/carrier_queue.rs` has BUILDAIR, completion and cancel, but not `Unit::close`'s −n nor `gain_tech`'s carrier move | yes (Modern carriers) | high |

All seven: read by `LeaderData::log_data (006e5110)`; the six (not
`combat_queued`) by `LeaderData::get_support_count (006da110)` for the price
ramp in `TypeData::get_cost (00664090)`, which also reads `scholar_militia`.
