# Item 4152, family 2 (`kills`): adjudication

- Slug: `kills`. Leaves (`LeaderData`): `units_killed` (+0x80c, int),
  `units_lost` (+0x810, int), `buildings_razed` (+0x818, int),
  `buildings_lost` (+0x81c, int).
- Readings: `kills-reading.md` (first), `kills-second.md` (blind second).
- Adjudicator: Claude Opus 5.5 (claude-opus-5-5), 2026-11-11.
- Method: the export (`~/ghidra-projects/decomp`), the full `.text`
  listing (`exe.s` in the job scratch), `crates/sim` grep. Nothing run of
  the original.

(Written per leaf as settled.)

## 0. The shared ground, re-run

**Base.** Agree, confirmed. `leaders` is `0xe3a390`, slot stride `0x6eec`,
`LeaderData` at +0 of `Leader`; the four leaves sit at slot-0 addresses
`0xe3ab9c`, `0xe3aba0`, `0xe3aba8`, `0xe3abac`. The decompile's hidden store
in `Object::take_damage (00652020)` line 511 (`local_8[0x20d].field_0x31a0`,
`local_8 = param_8·0x6eec`) is `0x20d·0x6eec + 0x31a0 = 0xe3ab9c`; its
neighbours on lines 509 and 535 resolve the same way to +0xa5a and +0x44.

**Listing net, absolute (my own grep of `exe.s`).** Exactly the 11 sites
both readers list: 9 `incl` (0x64ba49, 0x652f5b, 0x653007, 0x653046,
0x67222f, 0x67277d, 0x673867) and 4 loads (0x583ef8, 0x583fbb, 0x9e98cf,
0x9e98ff). Slot-biased forms (base ± k·0x6eec, k = 1..7): zero hits.

**Listing net, register-relative stores.** 140 stores to
`0x80c|0x810|0x818|0x81c(%reg)` in 59 functions (the first reading counted
138, the second 133: different regexes, same functions). Only
`Leader::reset_score (006e37f0)` is a leader. Every `0x81c(%reg)` store is
either off a register loaded from `GameAccess::game` (`0xc061ec`) or off
`this` in a `Game::` member (the 0/1/2 game-state word); `GraphicPieces`,
`RTreeData`, `TileSet`, `NetDaemon`, `Constants`, `TerrainMap`, `hpPrintf*`
are other records. `Object::do_damage`'s `incl 0x80c(%edx,%eax,4)` at
0x64a558 is off `GameAccess::game`, not the leaders array (agree, confirmed).

**Export net.** `->units_killed|…` has only `LeaderData::log_data
(006e5110)` lines 1456/1469/1482/1495; `field_0x80c|810|818|81c` names the
same eight functions both readers list (`CoordInfo::add_geometry`'s is a
`TerrainData`). No writer missing from either reading.

**Reader net (loads).** Register-relative loads of the four offsets
outside the known readers are all off other globals (`0xc061e4` in
`BuildTypeData::calc_gather`, `0xc061f0` in `Setup::build_civ_specific`,
0x63ba44 and 0x5ab8d2) or other records. No sim decision reads a leaf.

**`take_damage`'s callers.** Every `calll *0x16c(%reg)` in `.text`: the
eight both readers name plus `Window::on_mouse_wheel` (a `Window` vtable).
No direct `call 0x652020`, no register load of slot `+0x16c` that is then
called, and no vtable overrides slot `+0x16c` (13 vtables, all
`Object::take_damage`). The second reading's "these nine plus
`Window::on_mouse_wheel`" in its §2.2 is a count slip for eight.

**Vslots by vtable (`vtables.txt`).** `Build` and `Wall`: `+0x1c` is
`Buffer::is_pending_load (0041e0e0)` (`return 1`), `+0x18` is
`Window::get_button (0041bff0)` (`return 0`). `Unit` and `Animal`: `+0x18`
returns 1, `+0x1c` returns 0, `+0xe8` is `UnitData::is_captain (0046ceb0)`.
`Build`/`Wall` `+0x4c` is `WallData::is_active (00472350)`, which is
`flags(+8) & 4`; the crate calls the slot "finished"
(`crates/sim/src/combat/take_damage_view.rs:228`). This decides the one
real disagreement (leaf 3).

**`take_damage`'s return.** Agree, confirmed: 0 alive or a clamped city;
1 from the site-abandon disband (0x652aa2 `call Object::disband`, then
`movl $1,%eax` at 0x652aa9); after `die` (vslot `+0x158`, 0x6533cd) 1 when
the dead object was not a decoy (`testl %esi,%esi; je 0x652aa7`, 0x6533d3)
and 2 after the decoy-overflow arm (0x65342e).

## 1. `units_killed` (+0x80c, int)

### Claims lined up

| # | claim | first | second | mark | verdict | deciding line |
|---|---|---|---|---|---|---|
| 1.1 | writer `take_damage`, `incl 0xe3ab9c(%eax)` | yes | yes | agree | confirmed | 0x653046 |
| 1.2 | its index is `param_8` (ebx), the attacker's player | yes | yes | agree | confirmed: ebx is reloaded from `0x24(%ebp)` at 0x652065, 0x652bbd, 0x652cd3, 0x652d60; every outside jump into 0x652d63..0x65302d (from 0x652aba, 0x652adf, 0x652ae9, 0x652b10, 0x652bea, 0x652d00) leaves from a point where ebx holds `param_8` | 0x65302d `imull $0x6eec,%ebx` |
| 1.3 | writer `cast_sabotage` unit arm, index `param_2` (caster's player) | yes | yes | agree | confirmed | 0x67222f; decompile `cast_sabotage` line 102 |
| 1.4 | writer `cast_counterintel`, index `ecx` = caster's player | yes | yes | agree | confirmed: `SpellType::cast (00676ce0)` loads edi from `0xc(%ebp)` at 0x676cec, esi from `0x14(%ebp)` at 0x676d6f, then `push esi; push 0x10(%ebp); mov edi,ecx; call 0x673740` | 0x676d8c, 0x673867 |
| 1.5 | step +1, `incl`, no decrement anywhere | yes | yes | agree | confirmed (no `dec`/`sub`/non-zero `mov` in either net) | nets in §0 |
| 1.6 | `take_damage` predicate: death branch, not a building (`+0x1c` = 0), captain (`+0xe8`), `curr_uber_size(0) == 1`, decoy bit `+0x68 & 1` clear, then `param_7 >= 0` | yes | yes | agree | confirmed, each test at an instruction: 0x652f4a/0x652f53 (+0x1c), 0x652f94..0x652fab (+0xe8, inlined as `+0x8e` bit 15), 0x652fce/0x652fd6 (`curr_uber_size(0)`), 0x652ff7/0x652ffb (decoy), 0x65300d/0x653011 (`param_7 < 0` leaves) | as listed |
| 1.7 | no ally, ownership or `param_8 >= 0` test before the store; the ally test after picks only the sign of `score_combat` (+0x44) | yes | yes | agree | confirmed: straight-line code 0x653017..0x653046; `is_ally` at decompile line 531 follows | 0x653046 |
| 1.8 | `take_damage` also steps victim's `+0xa58` and `leaders[param_8]+0xa5a` | yes | yes | agree | confirmed | 0x653026, 0x653033 |
| 1.9 | the decoy-overflow recursion credits the victim's own player one kill | yes | yes (medium) | agree | **corrected (unreachable in a plain game)**. The arm exists: after `die`, a decoy figure with whole points past its share strikes `get_captain` with `param_7 = captain, param_8 = own player` (pushes 0x653407..0x653426). But the captain it strikes is itself a decoy: `SpellType::cast_create_decoy (00674370)` sets `unit_masks |= 1` on every figure down the new squad's `o_down` chain (decompile lines 179-194, the store at 0x674779), so the captain's own decoy test (0x652ff7) skips both `units_lost` and `units_killed`. The only other `orl $1, 0x68(%reg)` stores are `ScenarioRead::load_unit_chunk` (0x9ab082, 0x9ab3cf, 0x9ab69f), a scenario's per-unit load; a mixed squad needs a scenario. | 0x674779; 0x652ff7 |
| 1.10 | sabotage gate: target `flags(+8) & 1` (live), else mana refund and return | medium | yes | agree | confirmed: the same bit `take_damage` tests first (its decompile line 55) | `cast_sabotage` decompile line 62 |
| 1.11 | sabotage unit arm: target not a building, a unit (`+0x18`), captain, `curr_uber_size(0) == 1`, decoy clear → target's `+0xa58`, caster's `+0xa5a`, caster's `units_killed`, caster's `score_combat += cost/5` | yes | yes | agree | confirmed | `cast_sabotage` decompile lines 77-106 |
| 1.12 | the sniped target then dies through vslot `+0x158` directly (dtype 3 with objmask `0x200000`, else 2), so victim's `units_lost` does not step | yes | yes | agree | confirmed; the die runs whether or not the counter's inner tests passed | `cast_sabotage` decompile lines 164-188 |
| 1.13 | counterintel: non-captain target replaced by its captain (`+0xe4`); gate target player ≠ caster, ≠ `leaders[caster].who` (+0x8), and not mutual `diplos == 2` (+0x74) | yes | yes | agree | confirmed (`0xe3a404 = 0xe3a390 + 0x74`) | `cast_counterintel` decompile lines 40-53 |
| 1.14 | counterintel inner test (unit, captain, alone, not decoy) → `units_killed` and `score_combat` only; no `+0xa58`, `+0xa5a` or `units_lost` | yes | partly (names only `units_lost`) | one-sided (first, on the two shorts) | confirmed | `cast_counterintel` decompile lines 54-70 |
| 1.15 | counterintel's `die(2,-1,0)` runs inside the diplomacy gate whatever the inner test said; outside the gate there is no die | implied | implied | agree | confirmed | `cast_counterintel` decompile line 73 |
| 1.16 | resets: `reset_score` (from `Leader::init` 0x6e3b1a and `Setup::build_game` 0x5ad1e7), `Leaders::clear`'s and `Leader::init`'s `memset 0x6929` | yes | yes | agree | confirmed (`reset_score` decompile line 23; callers by listing) | 0x6e3b1a, 0x5ad1e7, 0x586dd6, 0x594faa |
| 1.17 | `Leader::close (006b7f80)` also memsets [+0, +0x6929) | no | yes | one-sided (second) | confirmed | 0x6b7f82 `pushl $0x6929`; `Leader::close` decompile line 11 |
| 1.18 | `LeaderData::walk_data (006d6750)` restores the leaf on a load | reader only | writer on load | one-sided (second) | confirmed: it walks the raw range [&who, +0x692a) | `walk_data` decompile line 17 |
| 1.19 | readers: `log_data`, `num_units_killed`, end-of-game Steam stat `gStatID[4]`, `AchieveChartPane::set_chart_data`, the checksum walk; no sim decision | yes | yes | agree | confirmed (`determine_end_of_game…` line 278; reader net §0) | 0x583ef8, 0x9e98ff |
| 1.20 | crate: `take_damage_typed` (`combat/fight/delivery.rs:469`), its `Taken::Died` arm (`:537`), using `alone` (`:515`, `takes_remainder_share`, `units/store.rs:667`) and `decoy` (`:530`) | yes | yes | agree | confirmed; `takes_remainder_share` is captain ∧ `curr_uber_size == 1`, the same test | `delivery.rs:515`, `:537` |
| 1.21 | crate: `cast_sniper_unit` (`delivery.rs:806`), needs the caster's player, which its `(v, dtype)` signature lacks | site yes | site + signature | agree (second adds the signature) | confirmed | `delivery.rs:806` |
| 1.22 | crate: `cast_counterintel`, `spells/cast.rs:1030`/`:1031` | :1030 | :1031 | agree | confirmed: fn at `:1031`, its SEAM doc at `:1028` | `spells/cast.rs:1031` |
| 1.23 | plain game: yes, all three | yes | yes | agree | confirmed for combat; the two spells are skirmish units' (whether the computer casts them is unread by all three) | — |

**Predicate spot-checks beyond the readings.** The unit test is vslot
`+0x18`, which also answers 1 on `Animal`. Not a correction: gaia's animals
are owned by a slot ≥ 8, `ObjectData::valid_target_const`'s first test
refuses `who > 7` (the crate's note at `combat/fight/target_valid.rs:293`),
and the nuke ring walks only the eight players' units
(`combat/nuke.rs:237`), so no plain-game `take_damage` strikes an animal.
Recorded so a builder does not add an animal exemption that the original
lacks.

### The rule as a builder should port it

- **Writers.** (a) `Object::take_damage (00652020)`: on the death branch
  of a non-building whose struck figure is its squad's captain, alone
  (no live `o_down` follower), and not a decoy — and only when the call
  carries an attacker object (`param_7 >= 0`) — `units_killed[attacker's
  player] += 1`. Beside it `deaths_current_frame[victim] += 1`,
  `kills_current_frame[attacker] += 1`. No ally, self or ownership
  exemption. (b) `SpellType::cast_sabotage (006720e0)`, unit arm (Sniper
  / Commando kill of a live, non-building unit target that is a lone,
  non-decoy captain): `units_killed[caster's player] += 1`, with
  target `+0xa58` and caster `+0xa5a`. (c) `SpellType::cast_counterintel
  (00673740)`: target (as its captain) not the caster's, not
  `leaders[caster].who`'s, not mutually allied; then if a lone
  non-decoy captain unit, `units_killed[caster] += 1`, and nothing else
  of this family.
- **Step.** +1, int.
- **Reset.** 0 at `reset_score` (every live slot at `Setup::build_game`,
  and in `Leader::init`); the `memset`s of `Leaders::clear`,
  `Leader::init` (empty slot) and `Leader::close`; restored by a load.
- **Crate sites.** `Sim::take_damage_typed`'s `Taken::Died` arm
  (`crates/sim/src/combat/fight/delivery.rs:537`), crediting
  `owner_of(_by)` when `_by` is an object (the attacker-present test is
  the builder's to carry; the crate has no `param_7 < 0` notion there),
  gated on `alone && !decoy`, before `close_count`; `Sim::cast_sniper_unit`
  (`delivery.rs:806`, needs the caster's player passed in);
  `Sim::cast_counterintel` (`crates/sim/src/spells/cast.rs:1031`).
- **Plain game.** Yes. The decoy self-credit does not occur in one.
- **Confidence.** High: listing at every store and test. Open: the
  castability checks upstream of both spells (`SpellType::cast`'s
  `0x6763c0` call for counterintel, `is_valid_target`), which decide when
  a cast happens but not what it counts.

## 2. `units_lost` (+0x810, int)

### Claims lined up

| # | claim | first | second | mark | verdict | deciding line |
|---|---|---|---|---|---|---|
| 2.1 | one writer, `take_damage`, `incl 0xe3aba0(%eax)`, index the struck object's own player (`movzbl 0x9(%edi)`) | yes | yes | agree | confirmed | 0x652fdc, 0x653001, 0x653007 |
| 2.2 | predicate: death branch, not a building, captain, `curr_uber_size(0) == 1`, decoy clear | yes | yes | agree | confirmed (the same four tests as 1.6, before the store) | 0x652f4a..0x652ffb |
| 2.3 | stepped before and independent of `param_7 >= 0`: attrition, wall decay and trigger deaths count, with no killer | yes | yes | agree | confirmed: the `param_7` test at 0x65300d comes after the store; `Unit::suffer_attrition (005e1a10)` calls slot `+0x16c` at 0x5e1bd1 | 0x653007 then 0x65300d |
| 2.4 | `param_5` and `param_3` do not enter | — | yes | one-sided (second) | confirmed: neither is read between 0x652f46 and 0x653007 | listing 0x652f46..0x653007 |
| 2.5 | not stepped by sniper/sabotage unit kills, counterintel kills, `Object::disband`, direct `die` | yes | yes | agree | confirmed by the nets (no other store to +0x810 of a leader) | §0 nets |
| 2.6 | resets as leaf 1 (incl. `Leader::close`, load) | partly | yes | as 1.16-1.18 | confirmed | `reset_score` 0x6e3823 |
| 2.7 | readers: `log_data`, `num_units_lost`, Steam `gStatID[7]`, chart pane; no sim decision | yes | yes | agree | confirmed | `determine_end_of_game…` line 330; 0x9e98cf |
| 2.8 | crate: `take_damage_typed` Died arm (`delivery.rs:537`), for the struck unit's owner, gated on `alone && !decoy`, no attacker test | yes | yes | agree | confirmed | `delivery.rs:515`, `:530`, `:537` |
| 2.9 | crate: also `Sim::die_by_attrition` (`delivery.rs:844`, from `territory/upkeep.rs:307`), since the crate's attrition kill bypasses `take_damage_typed` | yes | yes | agree | confirmed: `die_by_attrition` runs `close_count`, `relink_squad`, … with no counter; the captain/alone/decoy test must be read **before** its `relink_squad` | `delivery.rs:844`-`:856` |
| 2.10 | not at `cast_sniper_unit` or `cast_counterintel` | implied | yes | agree | confirmed | 1.12, 1.14 |
| 2.11 | summed over players `units_lost ≠ units_killed` | implied | yes | agree | confirmed, with 1.9's correction: the decoy self-credit is not one of the reasons in a plain game | — |
| 2.12 | plain game: yes (combat and attrition) | yes | yes | agree | confirmed | — |

Open in both readings, settled here: "can `suffer_attrition` strike a
non-captain figure" (first). It does not matter for the counter: a
non-captain struck figure fails the `+0xe8` test and counts nothing, as
in combat; the squad counts once, at its last figure. A non-`take_damage`
death path (boat contents, nuke) counts only if it goes through slot
`+0x16c`, and the full sweep in §0 says the eight callers are all there
is.

### The rule as a builder should port it

- **Writer.** `Object::take_damage (00652020)` alone: on the death branch
  of a non-building whose struck figure is a lone (no live follower),
  non-decoy squad captain, `units_lost[victim's player] += 1`, whatever
  the attacker (none included).
- **Step.** +1, int. **Reset.** As leaf 1.
- **Crate sites.** `Sim::take_damage_typed`'s `Taken::Died` arm
  (`crates/sim/src/combat/fight/delivery.rs:537`), and
  `Sim::die_by_attrition` (`delivery.rs:844`) before its `relink_squad`.
  Not the two spell sites.
- **Plain game.** Yes.
- **Confidence.** High. Nothing open.

## 3. `buildings_razed` (+0x818, int)

### Claims lined up

| # | claim | first | second | mark | verdict | deciding line |
|---|---|---|---|---|---|---|
| 3.1 | writer `Object::do_damage (0064a480)`, `incl 0xe3aba8(%eax)`, index the shooter's (`this`'s) player | yes | yes | agree | confirmed | 0x64ba3f..0x64ba49 |
| 3.2 | gate: the `take_damage` call at 0x64ba18 returned non-zero, and the target answers `+0x1c` (building); a unit kill returns from `do_damage` there | yes | yes | agree | confirmed | 0x64ba20 `je`, 0x64ba39 `je`; decompile lines 896-902 |
| 3.3 | no owner or ally test before the step; the later `is_ally` picks the sign of `score_combat` (floored at 0 on the minus side) | yes | yes | agree | confirmed | decompile lines 903-924 |
| 3.4 | writer `cast_sabotage` building arm, `incl 0xe3aba8(%esi)`, index caster's player | yes | yes | agree | confirmed | 0x67277d |
| 3.5 | sabotage strikes `take_damage(dmg, 0, 3, -1, 1, 0x55555555, -1, -1, 0)` on the object vslot `+0xac` returns; dmg = max(`+0x114`/2, the type-indexed floor) | yes | yes | agree | confirmed; first's open "which object `+0xac` returns" settled: `Build`'s `+0xac` is `GraphicWinXML::get_graphicwinxml (0041c000)`, `return this`, so the building itself | 0x67275d; `vtables.txt` `Build::vftable +0xac` |
| 3.6 | non-zero includes the site-abandon disband, which credits a raze with no `buildings_lost` | yes | yes | agree | confirmed | 0x652aa2..0x652aa9 |
| 3.7 | the abandon arm's second test, vslot `+0x4c` = 0 | "not a city" | "not active: a construction site" | **disagree** | **corrected for the first reading**: on `Build` and `Wall` slot `+0x4c` is `WallData::is_active (00472350)`, `flags(+8) & 4`, the crate's "finished" (`take_damage_view.rs:228`). The arm strikes an **unfinished** building. The city test is a different slot pair (`+0x20` and `flags & 0x20`, decompile lines 176, 291-293). | `take_damage` decompile line 352; `WallData::is_active` body |
| 3.8 | the rest of the abandon arm: owner's `leader_flags & 4` clear (a computer player), `hits(0) <= 2·damage`, the building's `+0x60 & 0x2000` | yes | yes | agree | confirmed. The bit's only writer is `Leader::check_orphaned_buildings (006c9f20)` (decompile line 279); the crate keeps no such bit (`crates/sim/src/ai/ai_make.rs:936`, SEAM) | `take_damage` decompile lines 352-360 |
| 3.9 | exempt: a city building (clamped, returns 0) | yes | yes | agree | confirmed: the clamp is not under the `param_5 == 0` block (the block closes at decompile line 134; the clamp sits in the `else` of line 176's slot `+0x20` test), so it holds for every caller | decompile lines 216-302 |
| 3.10 | `do_damage`'s `this` from `Ammo::do_damage` and `Nuke::do_damage` | unread | unread | open in both | settled: `Ammo::do_damage (00678060)` passes `objects[ammo+0x3c][ammo+0x40]`, the ammo's shooter (decompile line 130); `Nuke::do_damage (0092bc80)` passes `objects[nuke_effect+164][nuke_effect+192]`, the object the effect records (lines 135-138, 286). Either way the credit goes to that object's owner | as cited |
| 3.11 | resets as leaf 1 | yes | yes | agree | confirmed | `reset_score` 0x6e383b |
| 3.12 | readers: `log_data`, chart pane; not the end-of-game stats, no script | yes | yes | agree | confirmed (`set_chart_data` line 63) | — |
| 3.13 | crate: `do_damage_round` (`delivery.rs:70`), the `if let Obj::Building(b) = target { if killed {` block (`:211`-`:221`), before and outside the plunder's unit/non-splash/non-air/non-ally gate, for `owner_of(attacker)` | yes | yes | agree | confirmed | `delivery.rs:192`, `:211` |
| 3.14 | crate: `cast_sabotage_building` (`delivery.rs:884`), needs the caster's player | yes | yes | agree | confirmed | `delivery.rs:884` |
| 3.15 | crate: the abandon arm is absent (`damage_building` has no `0x2000` disband) | yes | yes | agree | confirmed | `delivery.rs:957`-`:1050` |
| 3.16 | plain game: yes; the abandon arm needs a computer's abandoned site struck to half its hits | yes | yes | agree | confirmed (computer players run `check_orphaned_buildings`) | — |

### The rule as a builder should port it

- **Writers.** (a) `Object::do_damage (0064a480)`: when its `take_damage`
  call on the target returns non-zero and the target is a building,
  `buildings_razed[shooter's player] += 1` — whoever owns the building,
  ally or self included. (b) `SpellType::cast_sabotage (006720e0)`,
  building arm: `take_damage` non-zero → `buildings_razed[caster] += 1`.
- **What "non-zero" is on a building.** Death (damage ≥ hits, not a
  finished city building) **or** the site-abandon disband: unfinished
  (`flags & 4` clear), owned by a computer player (`leader_flags & 4`
  clear), `hits ≤ 2·damage`, and `build_masks & 0x2000` set by
  `Leader::check_orphaned_buildings`. The second credits a raze while the
  owner's `buildings_lost` does not step.
- **Step.** +1, int. **Reset.** As leaf 1.
- **Crate sites.** `Sim::do_damage_round`'s building `killed` block
  (`crates/sim/src/combat/fight/delivery.rs:211`), outside the plunder
  gate; `Sim::cast_sabotage_building` (`delivery.rs:884`). The abandon arm
  needs `build_masks & 0x2000` and its disband in `damage_building` first.
- **Plain game.** Yes.
- **Confidence.** High on writers, indices and both arms (listing). Open:
  nothing that changes the count.

## 4. `buildings_lost` (+0x81c, int)

### Claims lined up

| # | claim | first | second | mark | verdict | deciding line |
|---|---|---|---|---|---|---|
| 4.1 | one writer, `take_damage`, `incl 0xe3abac(%eax)`, index the dead building's own player | yes | yes | agree | confirmed | 0x652f4f, 0x652f55, 0x652f5b |
| 4.2 | predicate: death branch and vslot `+0x1c` non-zero (building or wall segment); then `ScenarioData::last_razed[who] = o` and straight to `die` | yes | yes | agree | confirmed | 0x652f4a..0x652f70 (`movl %ecx, 0xcc2190(,%eax,4)` at 0x652f69) |
| 4.3 | no attacker test, no `param_5` test | yes | yes | agree | confirmed (neither is read in 0x652f46..0x652f70) | as 4.2 |
| 4.4 | counts combat, sabotage, wall/building decay in hostile land, trigger and editor deaths | yes | yes | agree | confirmed (the eight callers, §0) | 0x640b22, 0x64ba18, 0x67275d |
| 4.5 | `Wall::process (00640450)`'s hit: `take_damage(8, 0, 1, -1, 1, 0x55555555, -1, -1, …)` on a started building on a tile of another, non-allied player; an unstarted one there is disbanded instead | yes | yes | agree | confirmed | `Wall::process` decompile lines 220-284 |
| 4.6 | does a `Build` (not only a `Wall`) reach `Wall::process`'s territory arm? | asserted (wall) | open | one-sided / open in second | **settled, yes**: `Build`'s slot `+0x9c` is `Build::process`, which calls `Wall::process` directly (`call 0x640450` at 0x61ee11, `Build::process` decompile line 60). The crate's `building_attrition` (`cities/city.rs:3123`) is right to serve every building | 0x61ee11 |
| 4.7 | not counted: a finished city building (clamped), the site-abandon disband, `Wall::process`'s disband of an unstarted site, `Object::disband`, direct `die` | yes | yes | agree | confirmed | 0x652aa2; `Wall::process` line 280 |
| 4.8 | summed over players `buildings_lost ≠ buildings_razed` | yes | yes | agree | confirmed | — |
| 4.9 | resets as leaf 1 | yes | yes | agree | confirmed | `reset_score` 0x6e3835 |
| 4.10 | readers: `log_data`, chart pane; no script, no Steam stat, no sim decision | yes | yes | agree | confirmed (`set_chart_data` line 65) | — |
| 4.11 | crate: `Sim::damage_building`'s `Taken::Died` arm (`delivery.rs:1041`), before `die_building(b)`, for the building's owner; not in `die_building`, which other paths also reach; the city clamp at `:1025` returns before it | yes | yes | agree | confirmed: one function serves combat (`take_damage_typed` → `:599`), sabotage (`cast_sabotage_building`) and attrition (`city.rs:3128`, `damage_building(b, hit, None, frame, true)`) | `delivery.rs:1041` |
| 4.12 | the crate does not carry `last_razed` on this path | yes | — | one-sided (first) | confirmed; outside this family | — |
| 4.13 | plain game: yes | yes | yes | agree | confirmed | — |

### The rule as a builder should port it

- **Writer.** `Object::take_damage (00652020)` alone: when a building or
  wall segment (vslot `+0x1c`) reaches the death branch — accumulated
  damage at or above its hits, and not a finished city building —
  `buildings_lost[owner] += 1`, whatever struck it, attacker or none.
- **Step.** +1, int. **Reset.** As leaf 1.
- **Crate site.** `Sim::damage_building`'s `Taken::Died` arm
  (`crates/sim/src/combat/fight/delivery.rs:1041`), before
  `die_building`.
- **Plain game.** Yes: combat razing, and border decay of any started
  building (not only walls) standing in a non-allied player's territory.
- **Confidence.** High. Nothing open.

## Verdict table

| leaf | claims confirmed / corrected / open | the rule in one line |
|---|---|---|
| `units_killed` +0x80c | 22 / 1 / 0 (+1 noted open upstream: spell castability) | +1 to the attacker's player when a lone non-decoy squad captain dies in `take_damage` with an attacker object (no ally/self exemption); +1 to the caster on a Sniper/Sabotage unit kill or a counterintel kill of such a captain; the decoy-overflow self-credit cannot fire in a plain game (decoy squads are all-decoy) |
| `units_lost` +0x810 | 12 / 0 / 0 | +1 to the victim's player when a lone non-decoy squad captain dies through `take_damage`, attacker or none (attrition counts; spell kills do not); crate sites `take_damage_typed` and `die_by_attrition` |
| `buildings_razed` +0x818 | 15 / 1 / 0 | +1 to the shooter's (or sabotaging caster's) player when `take_damage` returns non-zero on a building: its death, or the computer's unfinished abandoned site (`flags & 4` clear, `hits ≤ 2·damage`, `build_masks & 0x2000`) disbanded; no ally exemption |
| `buildings_lost` +0x81c | 13 / 0 / 0 | +1 to the owner when a building or wall segment dies through `take_damage` (combat, sabotage, border decay of any started building, triggers); not a city, not an abandon-disband |

The one correction against both readings: 1.9, the decoy self-credit is
unreachable without a scenario. The one correction against one reading:
3.7, the abandon arm's slot `+0x4c` is "finished", not "a city". One
open question of the second reading closed in its favour of the crate:
4.6, every `Build` reaches `Wall::process`.
