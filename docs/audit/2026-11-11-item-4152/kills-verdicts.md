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
| 1.3 | writer `cast_sabotage` unit arm, index `param_2` (caster's player) | yes | yes | agree | confirmed | 0x67222f; decompile `cast_sabotage` line 98 |
| 1.4 | writer `cast_counterintel`, index `ecx` = caster's player | yes | yes | agree | confirmed: `SpellType::cast (00676ce0)` loads edi from `0xc(%ebp)` at 0x676cec, esi from `0x14(%ebp)` at 0x676d6f, then `push esi; push 0x10(%ebp); mov edi,ecx; call 0x673740` | 0x676d8c, 0x673867 |
| 1.5 | step +1, `incl`, no decrement anywhere | yes | yes | agree | confirmed (no `dec`/`sub`/non-zero `mov` in either net) | nets in §0 |
| 1.6 | `take_damage` predicate: death branch, not a building (`+0x1c` = 0), captain (`+0xe8`), `curr_uber_size(0) == 1`, decoy bit `+0x68 & 1` clear, then `param_7 >= 0` | yes | yes | agree | confirmed, each test at an instruction: 0x652f4a/0x652f53 (+0x1c), 0x652f94..0x652fab (+0xe8, inlined as `+0x8e` bit 15), 0x652fce/0x652fd6 (`curr_uber_size(0)`), 0x652ff7/0x652ffb (decoy), 0x65300d/0x653011 (`param_7 < 0` leaves) | as listed |
| 1.7 | no ally, ownership or `param_8 >= 0` test before the store; the ally test after picks only the sign of `score_combat` (+0x44) | yes | yes | agree | confirmed: straight-line code 0x653017..0x653046; `is_ally` at decompile line 531 follows | 0x653046 |
| 1.8 | `take_damage` also steps victim's `+0xa58` and `leaders[param_8]+0xa5a` | yes | yes | agree | confirmed | 0x653026, 0x653033 |
| 1.9 | the decoy-overflow recursion credits the victim's own player one kill | yes | yes (medium) | agree | **corrected (unreachable in a plain game)**. The arm exists: after `die`, a decoy figure with whole points past its share strikes `get_captain` with `param_7 = captain, param_8 = own player` (pushes 0x653407..0x653426). But the captain it strikes is itself a decoy: `SpellType::cast_create_decoy (00674370)` sets `unit_masks |= 1` on every figure down the new squad's `o_down` chain (decompile lines 178-195, the store at 0x674779), so the captain's own decoy test (0x652ff7) skips both `units_lost` and `units_killed`. The only other `orl $1, 0x68(%reg)` stores are `ScenarioRead::load_unit_chunk` (0x9ab082, 0x9ab3cf, 0x9ab69f), a scenario's per-unit load; a mixed squad needs a scenario. | 0x674779; 0x652ff7 |
| 1.10 | sabotage gate: target `flags(+8) & 1` (live), else mana refund and return | medium | yes | agree | confirmed: the same bit `take_damage` tests first (its decompile line 55) | `cast_sabotage` decompile line 61 |
| 1.11 | sabotage unit arm: target not a building, a unit (`+0x18`), captain, `curr_uber_size(0) == 1`, decoy clear → target's `+0xa58`, caster's `+0xa5a`, caster's `units_killed`, caster's `score_combat += cost/5` | yes | yes | agree | confirmed | `cast_sabotage` decompile lines 75-101 |
| 1.12 | the sniped target then dies through vslot `+0x158` directly (dtype 3 with objmask `0x200000`, else 2), so victim's `units_lost` does not step | yes | yes | agree | confirmed; the die runs whether or not the counter's inner tests passed | `cast_sabotage` decompile lines 164-189 |
| 1.13 | counterintel: non-captain target replaced by its captain (`+0xe4`); gate target player ≠ caster, ≠ `leaders[caster].who` (+0x8), and not mutual `diplos == 2` (+0x74) | yes | yes | agree | confirmed (`0xe3a404 = 0xe3a390 + 0x74`) | `cast_counterintel` decompile lines 36-50 |
| 1.14 | counterintel inner test (unit, captain, alone, not decoy) → `units_killed` and `score_combat` only; no `+0xa58`, `+0xa5a` or `units_lost` | yes | partly (names only `units_lost`) | one-sided (first, on the two shorts) | confirmed | `cast_counterintel` decompile lines 52-67 |
| 1.15 | counterintel's `die(2,-1,0)` runs inside the diplomacy gate whatever the inner test said; outside the gate there is no die | implied | implied | agree | confirmed | decompile lines 68-71 |
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
