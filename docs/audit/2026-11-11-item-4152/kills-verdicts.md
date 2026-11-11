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
