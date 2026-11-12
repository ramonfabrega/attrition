# Item 4152, family 14 (`queued`): adjudication

- **Family**: 14, slug `queued`. Leaves: `scholar_militia` (+0x9f0); `barracks_queued` (+0xa10), `stable_queued` (+0xa14), `factory_queued` (+0xa18), `combat_queued` (+0xa1c), `dock_queued` (+0xa20), `air_queued` (+0xa24); all `int`.
- **Adjudicator**: Opus 5.5 (claude-opus-5-5), wave 3, 2026-11-11.
- **Inputs**: `queued-reading.md` (first), `queued-second.md` (blind second), the decompile export, the listing (`aa/exe.s`, the full `.text` dump) and `crates/sim`.
- **Asked to settle by name**: (a) the zero-floor predicate: does the guard skip a negative step only when the counter is exactly 0, so a step of -n from a positive value can go below zero; (b) the first reading's possible permanent +1 to `scholar_militia` from a recycled slot in `Unit::init (00612100)`.

Written per leaf as settled; the verdict table closes the file.

## Layout (both readings agree; confirmed)

A `Leader*`'s `field_0xNNN` is `LeaderData +0xNNN`: `Leader::track_queued (006e0f30)` stores at `0xa10(%edi)`..`0xa24(%edi)` with `edi = this`, and `Unit::process (00610bc0)` reaches the same leaves absolutely at `0xe3ada0(%esi)`..`0xe3adb4(%esi)` with `esi = who*0x6eec`. The first reading calls the 8-byte difference a suffix and the second puts `LeaderData` at offset 0 of `Leader`; that is the same statement. **Confirmed**, deciding lines `6e0f97` and `611357`.

---

## Leaf 1: `scholar_militia` (+0x9f0)

### Claims lined up

| claim | first | second | mark | verdict |
|---|---|---|---|---|
| writer: `Leader::track_unit_type (006e0dd0)`, `+= delta` at `6e0ecd` | yes | yes | agree | confirmed (`6e0ecd addl %edi,0x9f0(%esi)`) |
| writer: `Leader::init (006e3930)` `= 0` at `6e4c14` | yes | yes | agree | confirmed |
| writer: `LeaderData::walk_data (006d6750)` raw restore on load | yes | "no save/load path found" | one-sided (first) | **confirmed**: `walk_data` hands the byte range `&who .. +0x692a` to the walker (`LeaderData::walk_data (006d6750) decompile, line 17`), which covers +0x9f0 |
| no inlined copy of the arm; six `calll 0x6e0dd0` sites (`Unit::set_type` x2, `Unit::close`, `Objects::init_unit` x2, `SpellType::cast_create_decoy`) | yes | yes | agree | confirmed (my store net, below) |
| predicate: `uid >= 0`, tracked type `is(MILITIA 0x42, 0)`, then `f = units[who][uid] +0x54`; `f < 0` returns; `types[f].is(SCHOLARS 0x34, 0)` | yes (the `f < 0` exit not spelled) | yes, with `f < 0`: return | agree; the return one-sided (second) | **confirmed**: `6e0e8b js` (uid), `6e0e9d call *0x60` with `0x42`, `6e0eb1 movl 0x54(%eax),%ecx`, `6e0eb6 js 0x6e0f0f` (the epilogue, not the SPECIALFORCES arm), `6e0ec6 call *0x60` with `0x34`. A militia that fails the Scholar test also leaves at `6e0ecb je 0x6e0f0f`; only a non-militia or `uid < 0` reaches the SPECIALFORCES arm at `6e0ed5` |
| step: unconditional `+= delta`, no floor | implied | "no floor, no clamp" | agree | confirmed: `6e0ecd` is a bare `addl` |
| guard at each call site: `unit_masks & 1` clear, own captain (`+0x8e < 0`), and `control_cost != 0` or `is(FIGHTERBOMBER)` or government hero | yes | yes | agree | confirmed for `set_type`'s `+1` (`Unit::set_type (00612fa0) decompile, line 243-271`) |
| events: +1 at a Scholar's To Arms, -1 at its Back to Work and at its death | yes | yes | agree | confirmed (`cast_to_arms` writes `+0x54` before `set_type`; `cast_civilian` writes `+0x54` only when it is 0, `SpellType::cast_civilian (006704a0) decompile, line 29-38`) |
| reset: `Leader::init` only, skipped for a negative tribe (`6e3fee jl`) | yes | yes (no tribe caveat) | agree | confirmed: `6e3feb cmpl %eax,0xc(%ebp); 6e3fee jl 0x6e50f3`, then the zero run `6e4c09`..`6e4c5c` |
| readers: `TypeData::get_cost (00664090)` (Citizen: `- scholar_militia`; Scholar: `+ scholar_militia`) and `LeaderData::log_data (006e5110)` | yes | yes | agree | confirmed (`TypeData::get_cost (00664090) decompile, line 553,557`); my read net finds no other leader reader |
| stale-slot hazard: a militia type born in a recycled slot is counted from the previous occupant's `+0x54` | yes | no | one-sided (first) | **confirmed, and the routes named**: see below |
| `cast_immortal_hinf (00670600)` / `cast_immortal_arch (00670740)` could strand the count | no | raised, unread | one-sided (second) | **corrected: they cannot** on the code; see below |
| crate: derived in `Sim::worker_support` (`crates/sim/src/economy/pricing.rs:895`); a kept counter beside `Sim::track_unit_type` (`crates/sim/src/units/store.rs:755`) or exposed through `Sim::leader_long` (`crates/sim/src/checksum/leaders.rs:949`) | yes | yes | agree | confirmed: all three exist; `worker_support` tests `type_index` in `0x42..=0x44`, `rare` in `{0x34, 0x35}`, `captain`, `!decoy` |
| plain game: yes (To Arms, Back to Work are ordinary city commands) | yes | yes | agree | confirmed |

### The stale-slot `+1` (asked by name): confirmed

The chain, each link read:

1. **The `+1` runs before `+0x54` is cleared.** `Unit::init (00612100)` calls `Object::init (00647750)` at `612182`; that calls `SubObject::init (00662300)`, which stores the new slot index into `+0xa` and calls vtable `+0x84` with mode 1 (`SubObject::init (00662300)` decompile, the `(**(code **)(*(int *)this + 0x84))(TVar1,1)` line). For a `Unit`, `+0x84` is `Unit::set_type` (`vtables.txt`, `Unit::vftable 00b417d0`, `+0x84`). Under `SET_TYPE_INIT` `set_type` skips the old-type `-1` and runs the new-type `+1` (`Unit::set_type (00612fa0) decompile, line 274`), and `track_unit_type` reads `units[who][uid] +0x54` with `uid` the slot itself. Only afterwards does `Unit::init` store `+0x54 = 0`, at **`6121f6 movl %eax,0x54(%esi)`** (`eax` zeroed at `6121d9`). Nothing between writes `+0x54`: `SubObject::set_type` and `Unit::set_type` name no `+0x54`.
2. **Nothing clears `+0x54` at death.** `Unit::init`'s first act, `612131 calll *0x150(%eax)`, is `Unit::close (0060ee50)` (`Unit::vftable +0x150`). Its body is skipped for an inactive slot (`60ee7a testb $0x1,0x8(%ebx); 60ee7e je 0x60f99b`), and its tail only clears `unit_masks` bit 0 (`60fa35 andl $-0x2,0x68(%ebx)`), which is what lets the stale `+1` pass the mask guard. No instruction in `60ee50`..`6121f5` stores to `0x54(%reg)`.
3. **Slots recycle constantly.** Unit objects are allocated once per slot and reused: `Objects::find_free (0065ad60)` returns the **lowest** slot whose `+0x8 & 1` is clear and whose reference short `+0x32` is 0 (`Objects::find_free (0065ad60) decompile, line 102-110`).
4. **A Scholar id is left behind in ordinary play.** A dead Scholar militia leaves its Scholar id in `+0x54`; so does a dead *Scholar* that ever went to arms and back, because `cast_civilian` keeps a non-zero `+0x54`.
5. **Militia-typed units are born through `Objects::init_unit (0065e0c0)`**, the path that reaches `Unit::init` (`Unit::vftable +0x8c`). Three routes:
   - `SpellType::cast_create_decoy (00674370)` copies nearby land units of the caster's side. Its exclusion list is the four civilian types `0x32..0x35`, caravans and `is(MERCHANT 0x3d)`, so militia **are** copied (`SpellType::cast_create_decoy (00674370) decompile, line 106-126`). After `init_unit` it runs its own undo, `track_unit_type(type, -1, new)`, but by then `+0x54` is 0, so the undo misses this leaf. Then it sets `unit_masks |= 1` (`:180-181`), so the decoy's death in `Unit::close` skips `track_unit_type` and never takes it back either. **Net +1 for the life of the game.**
   - `Unit::swap_team (006118d0)` (vtable `+0x94`, reached from `SpellType::cast_bribe (00672d00)`, `SpellType::cast_assimilate (00671820)` and `Leader::defeat_by (006d1c80)`) re-creates the unit in the new owner's slot via `init_unit`. It copies `+0x24`, `+0x3b`, `+0x50`, `+0x8`, `+0x68`, `+0xaa` and `+0xae`, **not `+0x54`** (`Unit::swap_team (006118d0) decompile, line 45-70`). So a bribed militia counts +1 for its new owner exactly when that slot is stale with a Scholar id. Its later `-1` sees 0 and misses: a permanent +1. A Scholar militia that is genuinely bribed loses its Scholar status: the old owner's `-1` at close is correct, and the new owner never counts it.
   - Scenario and cheat creators (`ScenarioFuncSet::add_unit`, `ScenarioEditor::add_unit`, `Game::action_cheat_init_unit`). `Build::train` matters only if some building trains a Militia-line type, which is a data question.

   Converted militia are not at risk: `cast_to_arms`, and `Object::eject_contents (0064cd20)`'s Citizen-to-militia turn, convert in place through `set_type` on a live slot. `Unit::come_out (00617c10)` and the transport spells create a *barge* and put the unit inside it, without re-creating the unit.

**Verdict: confirmed.** The `+1` is real code behaviour. It reaches a plain game through a General's decoy cast near a militia, or a Spy's bribe of a militia, either landing in a slot whose last occupant held a Scholar id. **Deciding lines**: `612182` (the `+1` inside `Object::init`) before `6121f6` (the clear); `60ee7e`/`60fa35` (close skips its body and keeps `+0x54`); `cast_create_decoy`'s mask set. **Open**: how often a plain game meets it, which only a capture can say. A probe: a leader with a Scholar sent to arms and then killed, then a General's decoy over a militia. Read `scholar_militia` from the leaders line before and after, and expect +1 from the decoy. This would also settle whether the `+0x8e` captain guard passes on a recycled slot. `+0x8e` is not reset by `Unit::init` before `Object::init`, so for a previous occupant that was its own captain the guard passes.

### Second reading's immortal-spell worry: corrected (no effect)

`cast_immortal_hinf` targets only a unit that answers `is(IMMORTALARCHERS 0xb8)`, and `cast_immortal_arch` only one that answers `is(0xa2)` (`SpellType::cast_immortal_hinf (00670600)` decompile, the `(*pcVar3)(0xb8,0)` test; `SpellType::cast_immortal_arch (00670740)` decompile, line 23). Each overwrites `+0x54` with the unit's current immortal type and then `set_type(NORMAL)`. The `-1` is for the immortal type, which does not answer `is(MILITIA)`, so the leaf is untouched. That holds unless the shipped `is_list` of an Immortal type names `MILITIA` (data, not read). **Deciding line**: the `is(0xb8)` / `is(0xa2)` target tests.

### Writer completeness (my nets)

- **Listing**: every non-stack store, inc/dec, or-and, and `lea` with displacement `0x9f0` off any register, plus the absolute `0xe3ad80`, mapped through `INDEX.tsv`. The only leader stores are `6e0ecd` (`track_unit_type`) and `6e4c14` (`Leader::init`). The rest have non-leader bases: `HotKeyGroupOut`, `Constants::init`, `TileSet::init`, `TerrainMap`, `ConquestSetupWin`, the `SelectGroup` arrays, and `Game::do_frame`'s `lea 0x9f0(%ebx)` off the `Game`. There is no absolute store and no indexed (`disp(%r,%r,s)`) store in `0x9c0..0xa2f` anywhere in `.text`.
- **Export**: `->scholar_militia` appears only in `LeaderData::log_data`; `field_0x9f0` on a leader appears in `track_unit_type` and `get_cost`.
- **No writer missed by either reader.**

### Rule as a builder should port it

- **Counter**: `scholar_militia: i32` per leader, `0` at `Leader::init` (tribe >= 0); a load restores it raw.
- **Step**: at every `track_unit_type(type, delta, uid)` call, after the census arms: if `uid >= 0` and `type` answers `is(MILITIA)` and `f = units[uid].former_type` (`+0x54`, signed) is `>= 0` and `types[f]` answers `is(SCHOLARS)`, then `scholar_militia += delta`. No floor.
- **`former_type` lifecycle**, needed to reproduce it:
  - `cast_to_arms` sets it to the current type before the retype.
  - `cast_civilian` sets it to Citizen only if it is 0, and otherwise keeps it.
  - The immortal spells set it to the current type.
  - Death leaves it.
  - `Unit::init` zeroes it only **after** the birth `+1`. So a builder who keeps the counter must evaluate the birth `+1` against the slot's previous `former_type`, which needs slot reuse modelled as the original's lowest-free-slot rule.
- **Crate site**: today `Sim::worker_support` (`economy/pricing.rs:895`) derives it from live units, so it cannot carry the stale `+1`. A kept counter belongs in `Sim::track_unit_type` (`units/store.rs:755`) with the unit index added, exposed to `Sim::leader_long` (`checksum/leaders.rs:949`). The derived form's `0x42..=0x44` and `{0x34, 0x35}` are stand-ins for two kind tests, whose `is_list` data neither reader opened.
- **Plain game**: yes for To Arms, Back to Work and death; the stale `+1` is plain-game-reachable through decoy or bribe of a militia.
- **Confidence**: high on the writers, predicate and step (listing). High on the stale-`+1` mechanism (each link read). Its frequency is unmeasured.

---
## Leaves 2-7: the six `*_queued` counters (+0xa10 .. +0xa24), shared

### The zero-floor predicate (asked by name): settled

**Rule, from the listing of `Leader::track_queued (006e0f30)`.** Every add is written as: load the counter; if it is non-zero, add; if it is zero, add only when the delta is not negative. Read off `6e0f97 movl 0xa10(%edi),%eax; 6e0f9d testl; 6e0f9f jne 6e0fa5; 6e0fa1 testl %esi,%esi; 6e0fa3 js (skip); 6e0fa5 addl %esi,%eax; 6e0fa7 store`. The same shape repeats at `6e0fb6`/`6e0fcc`/`6e0feb`/`6e100a`/`6e102b` for the other five, and at `6e0f40`..`6e0f58` for the 16-bit `num_queued[t]` (`movzwl`/`testw`/`movw`).

So the guard is **"skip iff counter == 0 and delta < 0"**. Two consequences, the second of which neither reading states:

1. **`-n` from a positive value can go below zero.** A counter at `m` with `0 < m < n` given `-n` becomes `m - n < 0`. Both readings say this; **confirmed**.
2. **Once negative, nothing floors it.** Every inlined `-1` copy tests only `== 0`: `Build::unqueue (006207c0)` `6209d7 testl; 6209d9 je skip; 6209db decl`, and the same in `Build::clean_queue (00620b60)` (`620d8d`), `Unit::action_unqueue (005e1f20)` (`5e1fee`) and `Unit::process (00610bc0)` (`61135d`). A counter already below zero keeps falling on every `-1`. The `+1` copies are bare `incl` (`Build::queue_up (00620f40)` `621a2f`, `Group::action_spell (006fe1a0)` `6fe80c`). The first reading's "for them it is a true never below zero" is **corrected** to: it is a floor only while the counter has never gone negative. The rule is "skip a negative step at exactly 0", never "clamp at 0".
3. **Each counter is tested on itself.** In the Barracks and Stable arms, a skipped `barracks_queued`/`stable_queued` still falls through to `combat_queued`'s own test (`6e0fa3 js 0x6e0fcc`, `6e0fc2 js 0x6e0fcc`). Both readings agree; **confirmed**.

**Where `-n` comes from.** Two sites. The first is `Unit::close (0060ee50)`: `60efbc movzwl 0xa0(%ebx),%ecx; 60efc3 testw; je skip; 60efdb negl; push 0x134; call current_upgrade; 60efed call track_queued; 60eff4 movw $0,0xa0(%ebx)`. The second is `Leader::gain_tech (006dcb60)`'s carrier pair, `-a0` old FB then `+a0` new FB, at `6dda28`/`6dda47`.

Both land in `air_queued`. Each carrier `+1` also stepped `air_queued`, so in steady state `air_queued >= ` every carrier's own `num_queued`. Going below zero therefore needs an earlier drift of `air_queued` below a carrier's count. I found no plain-game source of such a drift. The gain_tech retarget nets zero inside a bin, and the carrier pair moves within the air bin. **Below zero is legal code but, as far as read, unreached in a plain game. Medium confidence.**

**The crate matches.** `Sim::untrack_queued` (`crates/sim/src/cities/queue.rs:2078`) tests `!= 0` on `queued_by_type`, `tech.queued` and `queued_by_group` separately, which is the original's skip at exactly 0 and not a clamp. It has no `-n` caller, because the carrier-loss path is missing (below).

### Claims lined up (shared)

| claim | first | second | mark | verdict |
|---|---|---|---|---|
| out-of-line rule `track_queued`: `t >= 0`; `num_queued[t]` step; `is_unit_type` (vtable `+0xc`); `attack (+0x1e8) != 0`; branch on `where (+0x40)` by equality: `0x1ab` barracks+combat, `0x1ac` stable+combat, `0x1ae` factory, `0x1b0` dock, else `domain (+0x218) == 2` air, else nothing | yes | yes | agree | **confirmed** on the listing `6e0f30`..`6e1044` (`6e0f38 js`, `6e0f6a call *0xc`, `6e0f80 cmpl $0,0x1e8`, `6e0f90`/`6e0faf`/`6e0fe4`/`6e1003` the four `cmpl`, `6e1022 cmpl $2,0x218`) |
| guard: skip iff counter == 0 and delta < 0, per counter | yes | yes | agree | confirmed; consequence 2 above added |
| inline copies: `Build::queue_up` (+1), `Build::unqueue` (-1), `Build::clean_queue` (-1 per entry), `Unit::action_unqueue` (-1), `Unit::process` (-1, absolute), `Group::action_spell` (+1), four `Wall::*` | yes | yes | agree | confirmed by my store net (below) |
| `Wall::*` copies are dead: a wall's type is a build type and `TypeData::is_unit_type (004707e0)` is the only implementation of slot `+0xc` | yes (asserted) | yes (override "not checked") | agree; the override check one-sided (first) | **confirmed**: all 12 `+0xc` slots in `vtables.txt` are `TypeData::is_unit_type` |
| inline copies devirtualize `is_unit_type` to `0x32 <= t < 0x19e` | yes | yes | agree | confirmed (`620989 cmpl $0x4707e0`; `620997 cmpl $0x32`; `6209a0 cmpl $0x19e`) |
| `track_queued` callers: `Unit::close` `60efed` (`-num_queued` of `current_upgrade(FB)`, then `num_queued = 0`) and `Leader::gain_tech` `6dda28`/`6dda47` (carrier pair) and `6ddd04`/`6ddd4a` (queue retarget) | yes | yes | agree | confirmed (`60efbc`..`60eff4` above) |
| retarget arm: on the graft-chain match, the `-1` is the **new** type's | yes | yes | agree | **confirmed**: the chain loop `6ddcce`..`6ddceb` exits to `6ddcf4` only with `esi == edi` (the new type from `0x5c(%ebp)`); the direct match `6ddca8 je 6ddcf4` keeps `esi = q`. Within one bin it nets to zero on these six |
| `Build::unqueue`'s callers include `Build::activate (00623e20)` (three `+0x1c8` sites) | no | yes | one-sided (second) | **confirmed as callers** (`6261b1`, `626391`, `626571`, all `calll *0x1c8`; `Build::vftable +0x1c8` is `Build::unqueue`). **Open**: whether the item cancelled there (`BuildData::find_in_queue` after a `BuildData::count_queue` test) is ever an attacking unit type. Only then would it step the family. Settled by reading `Build::activate` lines `1460`..`1640` for the item's origin |
| `Build::unqueue` from `Build::do_queue` (completion, `61ec35`/`61eda0`) and `Build::action_unqueue` (cancel) | yes | yes | agree | confirmed (my `calll *0x1c8` net) |
| `Build::train`'s `Unit::action_unqueue` on a fresh carrier changes nothing (`num_queued == 0`) | not confirmed | yes | one-sided (second) | **confirmed**: `Unit::init` does not reset `+0xa0`, but a freed carrier's `+0xa0` was zeroed by `Unit::close` (`60eff4`), and `action_unqueue` returns at 0 |
| reset: `Leader::init` `6e4c3e`..`6e4c5c`, tribe-gated; no recount | yes | yes | agree | confirmed |
| load: `LeaderData::walk_data` raw | yes | not found | one-sided (first) | confirmed (`LeaderData::walk_data (006d6750)` decompile, line 17; range covers +0xa10..+0xa24) |
| readers: `LeaderData::get_support_count (006da110)` (five bins, `queued + units`, gated on `+0x2f4 & 1` and attack; no combat arm) feeding `TypeData::get_cost`; `LeaderData::log_data` | yes | yes | agree | confirmed (`LeaderData::get_support_count (006da110) decompile, line 15-30`) |
| `combat_queued` has no non-log reader | yes | yes | agree | confirmed by my read net on `0xa1c` (only the writers' own loads and non-leader bases) |
| crate: `Muster::queued_by_group` via `Sim::track_tree_queued` (`cities/queue.rs:2017`) / `Sim::untrack_queued` (`:2078`); `Sim::retarget_queued_to` (`:1964`); carriers in `air/carrier_queue.rs` (`carrier_action_spell :60`, `carrier_queue_tick :137`, `unit_action_unqueue :167`) | yes | yes | agree | confirmed: all exist |
| crate missing `Unit::close`'s `-num_queued` and `gain_tech`'s carrier pair (a SEAM named in `Sim::upgrade_units_to`, `cities/queue.rs:1890`) | yes | yes | agree | confirmed |
| deriving the six from `queued_by_type` reads one high after a graft-chain retarget; derive from `queued_by_group` instead | no | yes | one-sided (second) | **confirmed**. The `-1` lands on the new type's `num_queued`, which is skipped when it is 0, while the bin (already holding the entry) falls. So `num_queued[old]` stays one high, and a sum over types would carry that. The original's bins do not |
| crate bins are per-`where_` (`load.rs`: `group = if attack != 0 { where_ }`), the original has exactly five bins and an air fallthrough | yes | yes | agree | confirmed as a structural difference; whether they coincide on the shipped data is open (neither read the `where`/`domain` columns) |
| plain game: yes for all live writers; carriers in the Modern ages | yes | yes | agree | confirmed |

### Writer completeness (my nets)

**Listing.** Every non-stack store / inc / dec / or / and with displacement `0xa10`, `0xa14`, `0xa18`, `0xa1c`, `0xa20` or `0xa24` off any register, plus the absolutes `0xe3ada0`..`0xe3adb4`. That is **75 stores in exactly 12 functions**:

| functions | stores |
|---|---|
| `Leader::track_queued` | 6 |
| `Build::queue_up` | 7 |
| `Build::unqueue` | 6 |
| `Build::clean_queue` | 6 |
| `Unit::action_unqueue` | 6 |
| `Unit::process` | 6, absolute |
| `Group::action_spell` | 7 |
| `Wall::increment_stats` | 7 |
| `Wall::decrement_stats`, `Wall::activate`, `Wall::close` | 6 each |
| `Leader::init` | 6 |

The first reading's "77 register-relative + 6 absolute" count is not reproduced, but its function set is the same. Every other hit has a non-leader base, the same list both readers gave: `Constants::init`, `TerrainMap`, `ConquestSetupWin`, `GraphicPieces`, `TileSet`, `TerrainOut::one_time_init`, `ScenarioEditor::modify_terrain_map`, and `ConsoleWin::run_cmd`'s `7da495` off `0xc06214`. There is no indexed store into the range.

**Export.** No `->*_queued` writer.

**No writer missed by either reader.**

### Rule as a builder should port it (all six)

`track_queued(who, t, d)`:
1. If `t < 0`, do nothing.
2. Step `num_queued[t]` (u16) by `d` unless it is 0 and `d < 0`.
3. Require `0x32 <= t < 0x19e` and `unittypes[t].attack != 0`; otherwise stop.
4. Choose counters by `w = unittypes[t].where`:
   - `0x1ab`: `barracks` and `combat`;
   - `0x1ac`: `stable` and `combat`;
   - `0x1ae`: `factory`;
   - `0x1b0`: `dock`;
   - otherwise `air` if `domain == 2`, else nothing.
5. For each chosen counter `c`: `if !(c == 0 && d < 0) { c += d }`. Never clamp. A negative counter keeps moving.

Callers and `d`:

| caller | `d` |
|---|---|
| queue append (`Build::queue_up`) | +1 |
| `Group::action_spell` BUILDAIR | +1 |
| completion and cancel (`Build::unqueue`) | -1 |
| whole queue cleared, per entry, last first (`Build::clean_queue`) | -1 |
| carrier plane cancelled (`Unit::action_unqueue`) | -1 |
| carrier plane finished (`Unit::process`) | -1 |
| carrier lost with `n` queued (`Unit::close`), then the carrier's `num_queued = 0` | -n on `current_upgrade(FB)` |
| FB-line upgrade (`gain_tech`), per live carrier | -a0 on old FB, +a0 on new FB |
| unit-line upgrade (`gain_tech`), per queue entry | -1 on the entry's type (the new type on the chain arm), +1 new |

Reset is `Leader::init` (tribe >= 0); a load restores raw. **Plain game**: yes.

#### `barracks_queued` (+0xa10)
`where == 0x1ab` and armed, always paired with `combat_queued` (whose test is its own). Stores, one per function: `6e0fa7`, `621a2f`, `6209dc`, `620d92`, `5e1ff3`, `611362`, `6fe80c`; reset `6e4c3e`. Crate: `queued_by_group[BARRACKS]`, exposed to `Sim::leader_long`. **Confirmed; high.**

#### `stable_queued` (+0xa14)
`where == 0x1ac` (not the Auto Plant `0x1ad`), paired with `combat_queued`. Reset `6e4c44`. Crate: `queued_by_group[STABLE]`. **Confirmed; high.**

#### `factory_queued` (+0xa18)
`where == SIEGEFACTORY 0x1ae` (not `FACTORY 0x1af`); no `combat_queued`. Reset `6e4c4a`. Crate: `queued_by_group[SIEGEFACTORY]`. **Confirmed; high.**

#### `combat_queued` (+0xa1c)
It is the Barracks and Stable arms only, with its own skip test (`6e0fcc`..`6e0fdc`; inline `6209fc`..`620a07`), and only `log_data` reads it. Crate: none. The port is a counter moved at the `track_tree_queued` / `untrack_queued` sites with its own `!= 0` test. Or it is `queued_by_group[BARRACKS] + queued_by_group[STABLE]`, which is exact only while all three are moved by paired ±1 from zero. That holds in a plain game, because `-n` never reaches these bins: it is FB-only, and FB is air. **Confirmed; high.**

#### `dock_queued` (+0xa20)
`where == 0x1b0` (not Anchorage `0x1b1` or Shipyard `0x1b2`). Reset `6e4c56`. Crate: `queued_by_group[DOCK]`. **Confirmed; high.**

#### `air_queued` (+0xa24)
`where` none of the four, and `domain == 2`. It is the only leaf with `-n` writers (`Unit::close` `60efed`; `gain_tech` `6dda28`). Reset `6e4c5c`. Crate: airport queues via `queued_by_group[<airport where_>]` plus `air/carrier_queue.rs`. **Missing**: `Unit::close`'s `-num_queued` and `gain_tech`'s carrier pair. A separate counter is the safer port, since several `where_`s may train armed air types, and they share one counter and one test. **Confirmed; high on the rule, medium on which buildings train `domain 2` types (data).**

---

## Verdict table

| leaf | claims confirmed / corrected / open | the rule in one line |
|---|---|---|
| `scholar_militia` +0x9f0 | 13 / 1 / 1 | In `track_unit_type`: `+= delta` when `uid >= 0`, the type `is(MILITIA)` and `types[units[uid].+0x54].is(SCHOLARS)`. No floor; reset at `Leader::init`. The birth `+1` reads the recycled slot's stale `+0x54` before `Unit::init` zeroes it (**confirmed**; reached by decoy or bribe of a militia); its frequency is open. Immortal spells cannot strand it (second reading corrected) |
| `barracks_queued` +0xa10 | 16 / 1 / 1 | armed, `where == 0x1ab`; `+= d` unless (counter == 0 and `d < 0`); paired with `combat_queued` |
| `stable_queued` +0xa14 | 16 / 1 / 1 | armed, `where == 0x1ac`; same test; paired with `combat_queued` |
| `factory_queued` +0xa18 | 16 / 1 / 1 | armed, `where == 0x1ae` (Siege Factory); same test |
| `combat_queued` +0xa1c | 16 / 1 / 1 | the Barracks and Stable arms, with its own skip-at-exactly-0 test; read only by `log_data` |
| `dock_queued` +0xa20 | 16 / 1 / 1 | armed, `where == 0x1b0`; same test |
| `air_queued` +0xa24 | 16 / 1 / 1 | armed, `where` not one of the four, `domain == 2`; same test; the only `-n` writers (carrier loss, carrier FB upgrade), which can take it below zero, after which `-1`s keep it falling |

For the six queued leaves, the one correction is the floor wording: a negative counter is not floored by the `-1` copies. The one open item is whether `Build::activate`'s cross-queue cancel ever carries an attacking unit type. For `scholar_militia`, the open item is the frequency of the stale `+1` in a plain game.
