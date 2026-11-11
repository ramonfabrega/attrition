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

## Leaf 1: `lost_capital_stamp` (+0x414, int)

### 1. Writers, both nets

Export: `->lost_capital_stamp` appears only in `LeaderData::log_data
(006e5110)` line 724 (a read). `field_0x414` on a `Leader`/`leaders.list[i]`:
`Leader::lost_capital (006dc870)` 54-57, `Leader::recapture_capital
(006d1ba0)` 17-18, `Leaders::process_all (006ed2a0)` 50-52, `Leader::process
(006b88b0)` 46-47, `Build::activate (00623e20)` 1419-1424, `Leader::init
(006e3930)` 310; reads in `Leader::process_elimination (006b8a20)` 17. The
other two hits (`GameReportWin::close (007b91b0)`, `ConquestInvadeWin::close
(0074f3d0)`) are window objects, not leaders.

Listing: absolute `0xe3a7a4` — `0x625f90` (`subl` frame) and `0x625fc8`
(store), both `Build::activate`; `0x6b8a4f` a read (`process_elimination`).
Register-relative stores `, 0x414(%reg)`: `0x6b8994` (`Leader::process`),
`0x6d1bd7` (`recapture_capital`), `0x6dc968`/`0x6dc976` (`lost_capital`),
`0x6e3f89` (`Leader::init`), `0x6ed394` (`process_all`). Rejected by base type:
`0x48ff1c`/`0x48ffcd` (`ConquestMapLoader`), `0x7a3b14` (`ConquestLeader`
ctor), `0x88c285` (`RiverSectionOut`), `0x8afaf0`/`0x8afb0e`/`0x8afbf9`
(`TerrainBorder`), `0x92e20b` (a stack slot), `0x9ad0a4`
(`ScenarioRead::load_leaders_chunk (009acde0)`, base `%edx = *(0xc061ec)` =
`GameAccess::game`, clearing `game->info`). The `0x414(%ecx,%eax)` load at
`0x80c407` is `IFaceMainBase::draw_timers (0080b060)` (a read). Not caught by
either net: a biased base (`0x40c(%reg)` off `Leader+8`, say); none was
seen, but that class cannot be excluded by grep.

Bulk writers: `Leader::init` with `param_2 < 0` runs `memset(this, 0,
0x6929)` (listing `0x6e398c`-`0x6e3995`) — `Leaders::init (006ed850)` calls
it with `-1` for every slot. `LeaderData::walk_data (006d6750)` walks the raw
bytes `[&who, +0x692a)` through the `DataWalk`, so a load (`input != 0`)
overwrites all three leaves (gated on `leader_flags & 1`).

### 2. Event, step, predicate

- **E1/E2 `lost_capital`**, only when `timer == 0` (it sets `timer = 1`
  first): after the modifier is computed (leaf 3),
  `stamp = trunc(stamp·256 / modifier) + frame` (`shl 8; cltd; idiv`, then
  `add game+0x550`). At rest `stamp <= 0` (see 3), so this is the loss frame
  moved back by the carried debt rescaled by the new modifier.
  `modifier >= 0x80` here, so no division by zero.
- **E3 `recapture_capital`** and **E4 `Build::activate` FC arm**, only when
  `timer != 0`: `stamp = trunc((stamp - frame)·modifier / 256)`, the
  `(x + ((x>>31)&0xff)) >> 8` idiom, i.e. truncation toward zero. Since
  `stamp <= frame` while running, the result is `<= 0`: the time run, in
  frames ×`modifier/256`, kept as a negative debt.
- **E5 pay-back**: `stamp += 1` every `TIMER_REFRESH_RATIO` (5) frames while
  `stamp < 0 && timer == 0` — the debt decays one frame per five.
- Units: frames, offset by a signed debt; the modifier is 8.8 (`0x100` = 1).
- Predicates are E1-E4's above: which leader is the old owner/captor; the
  Forbidden City (`0x213`) exempts E1 and drives E2/E4. The writes in
  `lost_capital`/`recapture_capital` are **not** gated on
  `info.elimination`; only their sound and message are (`== 1`).

### 3. Clear / reset

`Leader::init (006e3930)` stores 0 unconditionally (`0x6e3f89`, decompile
310, top-level, every call: `Setup::build_leader (005abc70)`,
`Setup::build_game (005ac190)` 618, `Leaders::init (006ed850)`, scenario and
CtW setups); the `param_2 < 0` memset also zeroes it. No other writer sets 0.
A save load restores it raw.

### 4. Readers

`Leader::process_elimination (006b8a20)` (`frame - stamp` against
`Game::retake_capital`), `IFaceMainBase::draw_timers (0080b060)` (UI:
remaining `= retake_capital(who) - (frame - stamp)`, floored at 0),
`LeaderData::log_data (006e5110)` 724 (the log/checksum), and its own
writers E1/E3/E4/E5.

### 5. Crate site

No field. `Sim::capture_city` (`crates/sim/src/cities/city.rs:3169`) keeps
`Tally::lost_capital_frame` (`city.rs:122`): set to `Some(frame)` at
3211-3219 when the city `was_capital` and it is `None`, cleared at 3220-3224
when the captor's new city is a capital. That is the shape of E1/E3 without
the Forbidden City exemption, without `find_capital`, without the debt and
without the modifier; nothing else reads it. A builder adds
`lost_capital_stamp`/`_timer`/`_modifier` beside `popwin_stamp`/`popwin_timer`
(`crates/sim/src/ai.rs:705-713`, the same stamp-debt shape), E1/E3 in
`Sim::capture_city`, E2 in `Sim::close_building` (`city.rs:2782`; no
Forbidden City arm there today), E4 beside `kremlin_activate`
(`crates/sim/src/cities/nations.rs:467`, called from `Sim::activate`,
`city.rs:2475`), and E5's arm in `Sim::refresh_victory_timers`
(`crates/sim/src/territory/victory.rs:88`, called from
`crates/sim/src/core/frame.rs:194`; it pays back only `popwin_stamp` now).
The checksum row exists (`crates/sim/src/checksum/leaders.rs:100`) and the
diff lists the key as unheld (`crates/rondata/src/diff/checksum_lines.rs:1810`).
R (`process_elimination`) is unported: `lost_a_city` (`city.rs:613`) is the
only elimination rule, and `ai.lobby.elimination` (`crates/sim/src/ai.rs:1088`,
default 1) is read only by the AI.

### 6. Plain game

Yes. E1 fires on any skirmish capture of a capital whose owner lacks a live
Forbidden City, in every elimination mode; E3 on retaking one's own former
capital; E5 every fifth frame for any leader carrying a debt. E2/E4 need the
Forbidden City, a buildable wonder, so they are plain-game but rare. A
corroborating data point, not a diff: `crates/rondata/src/diff/korean_ancient_pair.rs:326`
records the original's `lost_capital_stamp 5639` on the defeat block of frame
5639 — E1 from a zero stamp gives exactly `frame`.

### 7. Confidence and what was not read

High for the arithmetic and the E1/E3/E5 predicates (decompile and listing
agree). Medium for E2's entry predicate (the decompiler's `unaff_ESI`, the
`semaphore[1] & 8` bit, unnamed) and for whether E2 runs during a capture of
the Forbidden City itself (the order of `Build::close` against line 1137 in
`capture_city` was not traced). Not read: `Leader::defeat`, which does not
name the field in the export; the `semaphore` bits' meanings.

