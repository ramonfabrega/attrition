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

