# lost-capital: second (blind) reading

Item 4152, wave 2. Slug `lost-capital`. Leaves of `LeaderData` (size `0x6ee4`):
`lost_capital_stamp` (+0x414, int), `lost_capital_timer` (+0x418, int),
`lost_capital_modifier` (+0x41c, int). Reader: Claude Opus 5.5 (claude-opus-5-5),
blind to the first reading, the README, CHECKSUM.md §20 and CITIES.md §8.5.

Sources: the export at `~/ghidra-projects/decomp` (`funcs/`, `types.txt`,
`INDEX.tsv`) and the `.text` listing `r4152/aa/exe.s`. `Leader` (0x6eec) is
`LeaderData` (0x6ee4) at offset 0 plus a vbtable pointer at `+0x6ee4`
(`Leader::Leader (006d2bf0)` line 19), so a `Leader*`'s `field_0x414/418/41c`
is the `LeaderData` field itself; the slot-`i` absolute forms are `0xe3a7a4`,
`0xe3a7a8`, `0xe3a7ac` (+ `i·0x6eec`).

## Shared ground: the events, read once

Names used below. *Owner* is the leader whose capital is concerned; *frame*
is `Game+0x550` (`game->frame`); city flag word is `City+4`; bit `0x10` of it
is "this city is its holder's capital"; `City+0x68` is a byte mask of leaders
whose capital this city was and who no longer hold it (`City::capture
(00736c40)` lines 96-97 add the old holder's bit when a `0x10` city changes
hands and clear `0x10`; lines 145-150 restore `0x10` and drop the bit when the
new holder's bit is in the mask, i.e. retaking your own former capital makes
it your capital again).

`LeaderData::find_capital (006eb930)`, `(out idx, out owner, skip_idx,
skip_owner)`: first scans the leader's own cities `0..city_mark` for a live
(`flags&1`) city with `0x10`, skipping `(skip_idx, skip_owner)` when
`skip_owner == who`; found → `idx`, `owner = who`. Otherwise scans every
other slot's cities for a live city whose `+0x68` has this leader's bit
(gated on *this* leader's `leader_flags & 3 == 3`, not the other slot's);
found → `idx`, `owner = that slot`. Otherwise `idx = -1`, `owner = who`.

`LeaderData::has_wonder (006ebc10)`: bit 1 = holds a live instance of the
type; bit 2 = the type is in `conquest_wonders` with the CtW civics minimum
(CtW only). `0x213` is `FORBIDDENCITY` (`crates/sim/src/cities/build.rs:98`).

**E1, capital lost by capture** — `Cities::capture_city (00733380)`
(`param_1` captor, `param_2` city index in the old owner's list, `param_3`
old owner), decompile line 1137-1144, call site `0x735092`: calls
`Leader::lost_capital(old_owner, captor)` when the captured city has `0x10`,
**and** `has_wonder(old_owner, FORBIDDENCITY)` is 0 or 2 (bit 1 clear: the
old owner holds no live Forbidden City), **and** `find_capital(old_owner,
skip = (param_2, param_3))` returns `idx < 0` or an owner other than the old
owner (no other own capital city left).

**E2, capital lost when the Forbidden City closes** — `Build::close
(00628980)` decompile line 433-455, listing `0x6290a4`-`0x62912e`: on the
wonder branch (the decompiler's `unaff_ESI != 0`; its else removes an
unbuilt wonder), when the closing building `is(0x213)` and
`game->semaphore` byte 1 bit `0x8` is clear, and `info.elimination != 3`:
`find_capital(owner, -1, -1)`; if the returned owner is not the building's
owner (the owner holds no own capital; a city carrying its bit is held by
another slot), call `lost_capital(owner, holder)` then
`lost_a_city(owner, 1, 1, holder)`. With tribe bonus `0x17`, a second
`find_capital` skipping the first hit may substitute that city's holder when
its flag word is negative (bit `0x8000`). (`elimination == 3` instead runs
`lost_a_city` over every city whose `+0x60` is the owner.)

**E3, capital recaptured** — `Cities::capture_city`, decompile line
1146-1153, call site `0x7350e2`: the captor's new city (`local_14`, its index
in the captor's list, set from the new building's `+0x72` at line 212) has
`0x10` (only when the captor retook its own former capital, per
`City::capture`), and `find_capital(captor, skip = (local_14, captor))`
returns `idx < 0` or a foreign owner → `Leader::recapture_capital(captor)`.

**E4, the Forbidden City activates** — `Build::activate (00623e20)` decompile
line 1411-1429, listing `0x625f64`-`0x625fc8`: inside the wonder branch after
`Wonders::init_wonder` returned `>= 0`, not gated on the `captured` argument
(the `param_1 == 0` block closes at line 1295): when the building `is(0x213)`
and the building owner's (`Build+9`) timer is nonzero — same body as E3's
stop (below) plus a message.

**E5, the per-frame pay-back** — `Leaders::process_all (006ed2a0)` (the live
copy; `Leader::process (006b88b0)` holds the same lines but has no direct
`call`/`jmp` and its address appears nowhere else in `.text`, so it is
inlined-only/dead): for each slot with `leader_flags & 2`, after
`process_elimination`, when `TIMER_REFRESH_RATIO != 0` and
`frame % TIMER_REFRESH_RATIO == 0`: if `stamp < 0 && timer == 0` then
`stamp += 1`. (Same shape for `+0x440/+0x444` and `+0x448/+0x44c`.)

**R, the countdown** — `Leader::process_elimination (006b8a20)`, every frame
for each `leader_flags & 2` slot, before E5: when `info.elimination == 1`
and `timer != 0`: `elapsed = frame - stamp`; `limit =
Game::retake_capital(who)`; if `limit <= elapsed` → `defeat(this, 1, by)`
with `by` the owner `find_capital(-1,-1)` returns (the holder of the
capital). `Game::retake_capital (00594530)`: `base = 0` if `RETAKE_CAPITAL <
1`, else `max(1, (xs·RETAKE_CAPITAL + std/2) / std)` with `xs` =
`World+0x0` (map width) and `std` = `+0x144` of the record at `*(0xe7fca8)`
(the Standard map size's width, as `crates/sim/src/world/analyze.rs:17`
names it), C truncating division; then `limit = trunc(base·modifier / 256)`.
When `game->semaphore` byte 2 bit `0x2` is set: `has_wonder(FC)==2` → ×2; or
when it is 0 and the other of slots 0/1 is active at diplo 0: halved if that
slot has the FC (CtW-shaped; not read further). `RETAKE_CAPITAL` and
`TIMER_REFRESH_RATIO` load by plain `get_item` (`Constants::init (00569a90)`
lines 2402-2405); rules.xml gives `3600` and `5`.

