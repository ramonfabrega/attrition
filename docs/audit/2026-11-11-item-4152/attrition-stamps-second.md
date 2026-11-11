# attrition-stamps — second (blind) reading

- Slug: `attrition-stamps`
- Leaves (`LeaderData`, all `int`): `attrition_stamp` (+0x1f4),
  `attrition_stamp2` (+0x1f8), `attrition_stamp3` (+0x1fc), `att` (+0x7f0),
  `give_att_disabled` (+0x7f8)
- Reader: Claude Opus 5.5 (claude-opus-5-5), blind to the first reading.
- Sources: `~/ghidra-projects/decomp` export, the `.text` listing at
  `tmp/r4152/aa/exe.s`, `types.txt`; `docs/ATTRITION.md`,
  `crates/sim/src/territory/attrition.rs` for the comparison.

## Layout check (applies to every leaf)

`Leader` (0x6eec) adds no fields over `LeaderData` (0x6ee4) in `types.txt`,
and every `Leader` method casts `this` to `LeaderData*` with no adjustment
(`Leader::calc_attrition (006cdea0)` passes `this` straight to
`LeaderData::has_preq`), so a `Leader*` field offset **is** the `LeaderData`
offset — no prefix. The global slot base `0xe3a390 + i·0x6eec` is confirmed by
`Game::do_frame (00591ef0)` (`mov $0xe3a390,%ebx` then `0x1f4(%ebx)`, step
`0x6eec`, bound `0xe71af0`) and by `Unit::process_attrition (005e11a0)`'s
`0xe3a404 + who·0x6eec + j·4` (= `diplos` at +0x74). So the absolute forms
are: `attrition_stamp` 0xe3a584, `attrition_stamp2` 0xe3a588,
`attrition_stamp3` 0xe3a58c, `att` 0xe3ab80, `give_att_disabled` 0xe3ab88.

**Trap met:** `ScenarioEditor` (size 0x204) has `update_leader_list` at its
own +0x1f4, and `ScenarioEditor::delete_leader (009a3a90)` /
`new_leader (009a1490)` store to `0x1f4(%eax)` off the editor singleton
(`0xe7f8dc`), not a leader. Those are not writers of any leaf here.

## 1. `attrition_stamp` (+0x1f4) — the border-violation warning throttle

### Writers (both nets)

Export net: `->attrition_stamp` matches only `LeaderData::log_data
(006e5110)` (a read). `field_0x1f4` on a leader matches
`Unit::process_attrition (005e11a0)` lines 194/245/248/270/281,
`Leader::init (006e3930)` 264, `Leader::gain_tech (006dcb60)` 2328, and
`Game::do_frame (00591ef0)` 225 (`pLVar19->list[0].field_0x1f4`).

Listing net: `0xe3a584` appears five times, all in `process_attrition`
(reads `5e14f9`, `5e1710`; stores `5e1586`, `5e1868`, `5e1880`). Register
forms: `Leader::init` `6e3dda` (`0x1f4(%ebx)`, `ebx = this`),
`Game::do_frame` `59232b` (`0x1f4(%ebx)`, `ebx` walks `0xe3a390` by
`0x6eec`), `Leader::gain_tech` `6e038a` (`0x1f4(%esi)`, `esi` walks the
leaders array). Rejected register hits: `Game::run_oos_recovery` `586b7f`
(base `0xc06214`, another object), `ScenarioRead::load_leaders_chunk`
`9acf74` (a string-bearing scenario record, not `LeaderData`), the
`ScenarioEditor` pair above, and the many non-leader `0x1f4(%reg)` stores in
`Scene::init`, `UnitType::init`, `Constants::init` etc.

| writer | store | value | predicate |
|---|---|---|---|
| `Unit::process_attrition (005e11a0)` `5e1868` | stamp of the **unit's own** leader | `max(1, frame + ally_to_war_grace − 900)` | unit on ground owned by a leader it is **not at war with** (or war not yet allowed in a conquest-flag game), the stamp is 0 or `frame − stamp > 900`, **and** the pair broke an alliance (`broke_alliance[]` +0x2b0, either direction) less than `ally_to_war_delay` frames ago **and** that break is under `ally_to_war_grace` frames old |
| same, `5e1880` | own leader | `game.frame` | same branch, the throttle open, but not the grace case |
| same, `5e1586` | own leader | `game.frame` | the unit's leader **is** at war with the owner, the game's `team_style == 2` (Assassin), the owner is not the leader's `get_target()`, and the throttle is open (stamp 0 or `frame − stamp > 900`) |
| `Game::do_frame (00591ef0)` `59232b` | every leader with `leader_flags & 1` | 0 | the timed rush rule ends: `info.rush_rules ≥ 9` and `frame == rush_rules_table[rr].minutes · 900` (`0x3c` of the 0x58-byte record at `0xe80088`), and `team_style ∉ {0, 8, 11}`; zeroed before the war declarations of the same loop |
| `Leader::gain_tech (006dcb60)` `6e038a` | every leader with `leader_flags & 1` | 0 | an age tech whose index − `BASE_ITEMTYPES` equals a nonzero `rush_rules` (the age-based rule), `Game::war_allowed` was false before it, and `Game::teams_locked` is true |
| `Leader::init (006e3930)` `6e3dda` | this | 0 | setup |

The 900-frame window is strict: the throttle re-opens at `frame − stamp ≥ 901`
(`cmp $0x384; jle skip`). 900 frames = 60 s at 15 fps. The grace store sets
the stamp so that the throttle re-opens exactly when the grace ends
(`stamp + 900 = frame + grace`), floored to 1 so it is never read as "never
stamped". The store happens **whether or not** the local message shows: the
message and `S_ATTRITION_WARNING` (0xa) are gated on `who ==
console->who` and are the only console-dependent part; the store is not.
In the not-at-war branch, when `rush_rules != 0` and war is not allowed, the
console player also gets `Game::say_no_war` first, and only while the stamp
is still 0 (the 1392 note in
`crates/rondata/src/blind.rs`).

### Event, step, reset

- Event: a refresh of `Unit::process_attrition` for a unit standing on a
  peaceful-or-not-yet-war neighbour's ground (or a non-target's in Assassin).
  Step: set to the frame, not incremented. Units: sim frames.
- Reset: `Leader::init` (0); the end of the rush-rules peace, both its timed
  (`Game::do_frame`) and its age (`Leader::gain_tech`) forms, zero it for
  every leader in play.

### Readers

`Unit::process_attrition` (the two throttle tests) and `LeaderData::log_data`
only. Nothing in the damage path reads it; it is sync-visible state.

### Crate site

`Sim::process_attrition` (`crates/sim/src/territory/upkeep.rs:239`) calls
the pure `territory::attrition::process` (`attrition.rs:458`), whose doc
says it ports the original "minus the diplomacy bookkeeping and the
throttled on-screen warning". No stamp exists. A builder adds the three
stores in `upkeep.rs`'s refresh arm, next to the `Situation`'s `at_war` /
`in_war_grace` / `assassin` decision, which already carries the predicates.
The clear goes in `Sim::end_the_peace`
(`crates/sim/src/leaders/diplomacy.rs:73`) — **whose comment misnames this
store**: it says "`update_leader_list` (`+0x1f4`), zeroed for each leader,
is the interface's". `update_leader_list` is `ScenarioEditor+0x1f4`; the
`Game::do_frame` store is off `0xe3a390 + i·0x6eec`, i.e. this leaf. The
age-rule clear in `gain_tech` has no crate counterpart that I found (the
crate's `war_allowed` handles the age rule, but there is no age-arm
declaration loop in `diplomacy.rs`).

### Plain game?

Yes. A skirmish with a rush rule (the peace) reaches the not-at-war branch
and the timed clear; any unit walking onto ground of a leader at peace
reaches it without one. The Assassin store needs `team_style 2`, a lobby
option, not a scenario. The grace store needs a broken alliance, reachable
by diplomacy in a plain game.

### Confidence

High for the stores, their values and the 900 window (listing read at each
site). Medium on the exact `bVar4`/`bVar2` truth table of the grace branch:
I read it in the decompile and at the store (`-0x10(%ebp)`, `-0x8(%ebp)`),
not the whole flag computation in the listing. Not read: whether a saved
game restores `LeaderData` wholesale (I found no bulk copy of 0x6ee4 bytes
in `.text`; the save track was not read), and the `team_style 7`
neutral-skip arm of the clear loops.

## 2. `attrition_stamp2` (+0x1f8) — first-attrition-damage latch

### Writers (both nets)

Export: `field_0x1f8` on a leader in `Unit::suffer_attrition (005e1a10)`
(test line 19, store line 53) and `Leader::init (006e3930)` 265;
`->attrition_stamp2` only in `LeaderData::log_data (006e5110)`.
Listing: `0xe3a588` twice, both in `suffer_attrition` — `5e1a25` `cmpl
$0x0` and `5e1b8a` `movl %eax` with `eax = [0xc061ec]+0x550` (`game.frame`).
Register form: `Leader::init` `6e3de6` (`0x1f8(%ebx)`, 0). No other leader
store found.

### Event, step, predicate

`Unit::suffer_attrition` runs once per attrition damage tick on a unit (its
two callers are both in `Unit::process (00610bc0)`, `610d35` and `6117fe`).
If the unit's leader's stamp2 is 0, it is set to `game.frame`
**unconditionally** — the message pair and `S_ATTRITION_WARNING` inside are
gated on `who == console->who`, the cell having an owner (`who ≥ 0`), and
`!is_ally(leaders[who], who)` (the unit's leader against **itself**, as the
crate's `suffer_view.rs` header also records), but the store sits outside
that gate. Once nonzero it is never re-armed: there is no window test, so
it is a once-per-game latch holding the frame of that leader's first
attrition damage. Step: a frame number. Exempt: nothing — every unit of
every slot that takes an attrition tick reaches it.

Edge: a first tick on frame 0 would store 0 and leave the latch open; no
attrition tick can occur on frame 0 in practice (no strength yet).

### Reset

`Leader::init` only. Not cleared at the rush-rules end, unlike stamp1.

### Readers

`suffer_attrition`'s own test and `log_data`.

### Crate site

`crates/sim/src/territory/suffer_view.rs`: the walk
`suffer_attrition_steps` already calls `v.warned_at(who)` and
`v.stamp_warned(who)` in the original's place, but `SimSuffer`'s
implementations are `warned_at → 0` (line 152) and `stamp_warned → {}`
(line 155). A builder stores `frame` into a per-leader word there and makes
`warned_at` read it. Nothing else changes, since the console arm is closed
in the headless sim.

### Plain game?

Yes: any unit that takes one attrition tick in a skirmish.

### Confidence

High (two absolute sites, one init, store read in the listing). Not read:
save/load restoration.

## 3. `attrition_stamp3` (+0x1fc) — first-building-attrition latch

### Writers (both nets)

Export: `field_0x1fc` on a leader in `Wall::process (00640450)` (test 288,
store 300) and `Leader::init (006e3930)` 266; `->attrition_stamp3` only in
`LeaderData::log_data`. Listing: `0xe3a58c` twice, both in `Wall::process`
— `640b3a` `cmpl $0x0` and `640bb7` `movl %eax` with `eax = game.frame`.
Register form: `Leader::init` `6e3df1`. Rejected: `Leader::init` `6e3d63`
`0x1fc(%esi)` — `esi` there walks a per-opponent `[8]` array inside the
leader (stride 4, neighbours at `0x13c`/`0x15c`/`0x19c(%esi)`), not this
word; `Leader::init` `6e4b06` is a 16-bit store to another record; the
`ScenarioEditor::delete_leader` store is the editor's.

### Event, step, predicate

`Wall::process` (every building, walls included) runs its territory block
when `(frame + o) % 16 == 0` (`o` = the object's index `+0xa`), the owner's
`disable_building_attrition` (+0x804) is 0, and then either war is allowed
(`rush_rules == 0` or `Game::war_allowed`) **and** `(frame + o) % 32 == 0`,
or war is not allowed (the 16-frame cadence alone). The cell's owner (or,
for a type whose vslot `0x108` answers true, `BuildTypeData::check_enemy_adjacent`
over the footprint) must be `≥ 0`, not the building's own leader, and not
allied (`is_ally`). Then:

- **unstarted** building (`is_started`, `flags & 2` when not overridden):
  `Object::disband(this, 0)`, then **unconditionally** the console message
  (string `0x123cc`, `S_ATTRITION_WARNING`, console only) and
  `stamp3 = frame` — the stamp test is skipped on this path (listing
  `6409d9 call disband; jmp 640b43`, past the `640b3a` test);
- **started**: 8 hit points through vslot `0x16c` (`take_damage`, args
  `8,0,1,-1,1,0x55555555,-1,-1`); if it died, return; if stamp3 is nonzero,
  return; otherwise the console message and `stamp3 = frame`.

So it latches the frame of the leader's first building to erode or be
removed by hostile ground, and an unstarted site's removal **re-stamps**
every time (it is not a pure latch on that arm).

### Reset

`Leader::init` only.

### Readers

`Wall::process`'s own test and `log_data`.

### Crate site

`Sim::process_building` in `crates/sim/src/cities/city.rs`, the block under
"A building in enemy territory bleeds" (around lines 2997–3011): the
`disband_building` arm (store unconditionally) and the survivor after
`building_attrition` (store if zero). The crate's cadence is
`build::ENEMY_TERRITORY_PERIOD = 32`, documenting the 16-frame peace arm as
unmodelled (`crates/sim/src/cities/build.rs:1083`). No stamp exists today.

### Plain game?

Yes: a wall or building left on ground that becomes a non-allied neighbour's
(borders move with cities, forts, civic techs), or a foundation laid there.

### Confidence

High for the stores and the disband-path quirk (listing). Medium on the
16/32 cadence split (read in the decompile, matches the crate's own note;
the listing's `0x8000000f` / `0x8000001f` masks were seen, not traced in
full).

## 4. `att` (+0x7f0) — the attrition strength a leader inflicts

### Writers (both nets)

Export: `->att` only in reads (`LeaderData::get_attrition (006e0b60)`,
`LeaderData::log_data (006e5110)`). `field_0x7f0` on a leader:
`Leader::calc_attrition (006cdea0)` line 50 (store) and `Leader::init
(006e3930)` line 626 (0). Listing: no `0xe3ab80` anywhere. Register stores
to `0x7f0(%reg)`: `Leader::calc_attrition` `6cdfdc` (`ebx = this`) and
`Leader::init` `6e4984` (`ebx = this`, 0). The rest are other classes:
`Constants::init` `56dde3` (`mongol_attrition`), `IFaceResources`,
`TileSetData`, `ChatSpace`. No inlined copy of the formula: its signature
load `0x1d4(%eax,%edi,4)` off `[0xc061f0]` occurs once in `.text`.

`Leader::init` zeroes `att` and `give_att_disabled` together (`6e4984`,
`6e498a`) and then calls `calc_attrition` (`6e49bc`), so init's 0 is
immediately overwritten.

### The formula (`Leader::calc_attrition (006cdea0)`, from the listing)

All `int`; `C` = `GameAccess::constants` (`0xc061f0`).

1. **Disabled.** If `give_att_disabled != 0`: `att = 0`, done — no
   multipliers, no floor (`6cdea6 jne 6cdfdc` with `edi = 0`).
2. **Steps.** `n` = the length of the leading run of held techs
   `ATTRITION1..ATTRITION4` (`TypeIndex` 0x2dd..0x2e0), tested in order by
   `LeaderData::has_preq` and stopping at the first one missing; `0 ≤ n ≤
   4`. The loop body also carries a `BUY_SELL` (0x2ad) arm that would count
   a step as held on `has_tribe_bonus(4)`; it compares the loop index to
   0x2ad and can never match in 0x2dd..0x2e0 — dead. (The decompile prints
   the step as `+ NUM_ITEMTYPES`; the listing is `incl %esi`, step 1.)
3. **Base.** `x = 0` if `n == 0`, else `x = C.attrition_improved[n − 1]`
   (`C+0x1d4 + 4n` = `C+0x1d8 + 4(n−1)`). The decompile prints this as
   `attrition_upgrade[n + 3]` — same address, wrong name: `attrition_upgrade`
   is `int[4]` at `C+0x1c8`, `attrition_improved` `int[4]` at `C+0x1d8`.
   Loaded by `Constants::init (00569a90)` with `String::convert_int` per
   entry, plain integers (shipped 1, 2, 4, 8).
4. **Four multipliers, in this order**, each only if its predicate holds,
   each `x = (C.pct + 100) · x / 100` (signed, truncating toward zero —
   magic `0x51eb851f`, `sar 5`, add the sign bit) and then **`if x == 0 then
   x = 1`** (`cmove` on the zero flag of the quotient):
   - `has_wonder(0x212)` (COLOSSEUM): `C.colosseum_attrition` (`C+0x470`);
   - `has_tribe_bonus(0xd)` (Russians, `Tribe::parse` case 0xd):
     `C.russian_attrition` (`C+0x74c`);
   - `game.semaphore.ptr[2] & 2` (the conquest flag) **and** the leader's
     byte `+0x6900` nonzero (that is `num_bonus_cards[10]`, `uchar[38]` at
     +0x68f6): `C.ctw_attrition` (`C+0xa0c`);
   - `has_wonder(0x21a)` (KREMLIN): `C.kremlin_attrition` (`C+0x510`).
5. `att = x`.

**The floor fires on zero, whatever produced the zero.** With `n == 0`
(`x = 0`) any one applying multiplier gives `(pct+100)·0/100 = 0 → 1`. So
a leader with **no attrition step** but a Colosseum, a Kremlin, the Russian
bonus, or the CtW card has `att = 1`, not 0. The only way to `att = 0` with
any multiplier applying is `give_att_disabled`.

Shipped constants: `colosseum_attrition` 50, `russian_attrition` 100,
`ctw_attrition` 50, `kremlin_attrition` 100 (`rules.xml`, read via
`get_item`, plain integers). All four at 4 steps: 8 → 12 → 24 → 36 → 72.

`has_tribe_bonus(0xd)` (`LeaderData::has_tribe_bonus (006e1370)`) is 0 when
`game.info.flags & 4`, or when the game has no starting town and the leader
has `city_num == 0`, or `tribe < 0`; otherwise it is the CtW racial-power
bit, else (unless `leader_flags2 & 0x40`) `tribes[tribe].bonus == 0xd`.

### When it is recomputed (callers of `calc_attrition`)

Three direct calls (listing `calll 0x6cdea0`): `Leader::calc_unit_stats
(006cf970)` at its first instruction (`6cf974`); `Leader::gain_tech
(006dcb60)` `6dd3d6`, only when the gained type's vslot `0x38` is true,
its techtype record's `+0x14 == 1`, and the leader's encrypted civic level
(`data_encrypted+0xec ^ 0x63187`) equals `C+0xad4` (named
`ctw_wonder_min_civics` in `types.txt`), together with `calc_pop_cap`,
`calc_anti_attrition` and `Region::fix_borders`; and `Leader::init`.
`calc_unit_stats` runs from `Leader::process (006b88b0)` /
`Leaders::process_all (006ed2a0)` when `leader_flags & 0x4000000` (cleared
as it runs), and from `Leader::set_age`, `Leader::set_epoch`, and many
`ScenarioFuncSet` / `ScenarioEditor` functions. The flag `0x4000000` is set
by `Leader::gain_tech` (twice, `6dd993`, `6e0315`), `Leader::lose_tech`
(`6d2afd`), `Build::activate` (`626f0b`), `Build::close` (`628f3f`),
`Leader::gather` (`6ce3d0`) and `Leader::calc_gather` (`6cf5ea`). So in a
plain game `att` is refreshed on the leader's next `process` after any
tech gained or lost, any building activated or closed (the two wonders),
and on the gather paths.

### Readers

`UnitData::get_attrition (00608fd0)` (`0x7f0(%ecx,%eax)` at `608fe6`): if
the owner's `att == 0` or `C.attrition == 0`, the period input is 0;
otherwise `att` is the divisor, first scaled by
`((attrition_aged_up · agediff + 100) · att + 99) / 100` when the owner is
not behind the victim in age. `LeaderData::get_attrition (006e0b60)` returns
it but has no direct caller or pushed address in `.text` (probably a folded
or dead accessor; not chased). `LeaderData::log_data`.

### Comparison with `docs/ATTRITION.md` §"1. Strength" and the crate

| claim | `docs/ATTRITION.md` | `attrition.rs` `strength` (l.42) / `upkeep.rs` `Sim::calc_attrition` (l.24) | listing |
|---|---|---|---|
| step count indexes `ATTRITION_IMPROVED` [1,2,4,8] | yes | `attrition_improved[steps−1]` | **agrees** (`C+0x1d8 + 4(n−1)`) |
| leading run, stop at first missing | implied | `take_while` | **agrees** |
| order Colosseum → Russian → CtW → Kremlin, truncating | yes | yes | **agrees** |
| floor of 1 | "a floor of 1" | `.max(1)` after each multiplier | the original's is **`== 0 → 1`**, not `max(1, ·)`; they differ only for a negative quotient (`pct < −100`), unreachable with shipped values |
| zero steps | "No steps means no attrition at all" | `if tech_steps == 0 { return 0 }` **before** the multipliers | **disagrees**: the original applies the multipliers to `x = 0` and floors each to 1, so zero steps plus any applying multiplier is `att = 1` |
| `give_att_disabled` | table lists it as "suppresses the above" | not modelled (no field) | `att = 0`, multipliers skipped |
| Colosseum / Kremlin arms | applied | `StrengthMods` false always (no owned wonder in the crate) | applied |
| when recomputed | — | `Sim::apply_gained` (`crates/sim/src/tech/gain.rs:601`, line 608) on tech gain | on the next `Leader::process` after flag `0x4000000` (tech gain/loss, `Build::activate`/`close`, gather), plus `set_age`/`set_epoch`, init |

**The zero-step divergence is reachable in a plain game**: every Russian
player with at least one city (or in a starting-town game) has `att = 1`
from its first `calc_unit_stats`, before it holds `ATTRITION1`; the crate
gives 0, so its territory inflicts nothing where the original inflicts
level-1 attrition (doubled to 2 only once a step is held: `1 · 200/100 =
2`). The same holds for a Colosseum or Kremlin owner with no step. Whether
a captured game shows it depends on a Russian (or Colosseum/Kremlin) player
whose ground a hostile unit enters before Allegiance; I did not search the
capture ledger. `gain_tech`'s `russian_attrition_upgrades` arm (`gain_tech`
decompile ~1763) gives the Russians the attrition steps free, which would
shorten but not close that window — not read in full.

A second, smaller divergence: the crate recomputes only on tech gain;
`lose_tech` and wonder completion/loss (`Build::activate` / `Build::close`)
also refresh `att` in the original.

### Crate site

`Sim::calc_attrition` in `crates/sim/src/territory/upkeep.rs` (line 24),
writing `territory.players[w].strength`, and the pure
`territory::attrition::strength` in `crates/sim/src/territory/attrition.rs`
(line 42). The fix for the zero-step case is in `strength`: drop the early
return, start `x` at 0, and floor with `if x == 0 { 1 }`. A `give_att_disabled`
check belongs at the top of the same function.

### Plain game?

Yes: tech gain, Russians, Colosseum, Kremlin are all skirmish content. The
CtW arm and `give_att_disabled` are not.

### Confidence

High for the formula (every instruction of `006cdea0` read in the listing).
Medium on the recompute triggers (flag setters found by grepping `orl
$0x4000000`/`$0xc000000`; a `bts`, byte-wise or register-built set would
escape that net, and I did not read each setter's predicate). Not read:
the `gain_tech` ordering against `Leader::process` within a frame (one-frame
timing of the refresh).

## 5. `give_att_disabled` (+0x7f8) — scenario switch: this leader inflicts no attrition

### Writers (both nets)

Export: `field_0x7f8` on a leader in `Leader::init (006e3930)` 627 (0); the
`->give_att_disabled` name matches nothing. Listing: `0xe3ab88` three
times — `ScenarioFuncSet::enable_give_attrition (00a02900)` `a02926`
(store 0), `ScenarioFuncSet::disable_give_attrition (00a02940)` `a02966`
(store 1), and the read in `Unit::process_attrition` `5e132c`. Register
form: `Leader::init` `6e498a` (0). Rejected `0x7f8(%reg)` stores:
`Constants::init` `56de19` (`mongol_free_forage`), `IFaceRenderManager`,
`Cliffs`, `GraphicPieces` (byte stores), `Leader::plan_strategy` `6b9bff`
(a 16-bit store into another record); `Leader::gain_tech` `6df4fe` reads
`C+0x7f8`, not the leader.

### Event, step, predicate

Both script functions take one argument, a player number `p` in 1..8;
slot `p − 1` must have `leader_flags & 1` and `& 2`, else they return −1.
`disable_give_attrition` sets 1, `enable_give_attrition` sets 0; each
returns 1. They are registered as script callables (their addresses are
pushed at `9d3fd7` / `9d4005`). Neither calls `calc_attrition` nor sets
flag `0x4000000`, so `att` keeps its old value until the next refresh:
after `disable`, `att` stays nonzero until then, but
`Unit::process_attrition` reads this flag directly on the **territory
owner** (`leaders[owner].give_att_disabled != 0 → return 0`, before the
unit-kind exemptions), so the damage stops at once; after `enable`, `att`
stays 0 until the next refresh, so attrition resumes only then.

### Reset

`Leader::init` (0).

### Readers

`Leader::calc_attrition` (forces `att = 0`) and `Unit::process_attrition`
(the owner's flag exempts every unit on that owner's ground).

### Crate site

None. `territory::attrition::process` has no such exemption and
`PlayerState` no such field; `checksum/leaders.rs:152` lists the offset but
emits nothing for it. It sits behind the trigger layer, which is cut.

### Plain game?

No — only the scenario trigger layer writes it. In a skirmish it is 0 from
`Leader::init` to the end.

### Confidence

High (two absolute stores, both functions read whole in the listing).

## Summary

| leaf | writers | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `attrition_stamp` +0x1f4 | `Unit::process_attrition (005e11a0)` ×3; `Game::do_frame (00591ef0)`; `Leader::gain_tech (006dcb60)`; `Leader::init (006e3930)` | `= frame` when the 900-frame throttle is open (unit on a not-at-war owner's ground, or a non-target's in Assassin); `= max(1, frame + ally_to_war_grace − 900)` in the broken-alliance grace | 0 at init and at the rush-rules end (timed: `do_frame`; age: `gain_tech`) for every leader in play | none; would go in `Sim::process_attrition` (`territory/upkeep.rs:239`), clear in `Sim::end_the_peace` (`leaders/diplomacy.rs:73`, whose comment misnames this store as `ScenarioEditor`'s `update_leader_list`) | yes (peace / rush rules) | high; grace truth table medium |
| `attrition_stamp2` +0x1f8 | `Unit::suffer_attrition (005e1a10)`; `Leader::init` | `= frame` once, on the leader's first attrition damage tick; never re-armed | init only | `SimSuffer::warned_at` / `stamp_warned` stubs, `territory/suffer_view.rs:152–155` | yes | high |
| `attrition_stamp3` +0x1fc | `Wall::process (00640450)`; `Leader::init` | `= frame` on the first hostile-ground erosion of a started building (if 0), and **every time** an unstarted site there is disbanded | init only | `Sim::process_building`, `cities/city.rs` ~2997–3011 | yes | high; cadence medium |
| `att` +0x7f0 | `Leader::calc_attrition (006cdea0)`; `Leader::init` (0, then calc) | recomputed: 0 if `give_att_disabled`; else `attrition_improved[n−1]` (0 if n=0), then ×(100+pct)/100 for Colosseum, Russian, CtW, Kremlin, each floored `0 → 1` | init; every `calc_unit_stats` | `Sim::calc_attrition` (`territory/upkeep.rs:24`) → `attrition::strength` (`territory/attrition.rs:42`) — **diverges at zero steps** (returns 0 before the multipliers; original gives 1 to a Russian / Colosseum / Kremlin leader) | yes | high (formula); medium (triggers) |
| `give_att_disabled` +0x7f8 | `ScenarioFuncSet::enable_give_attrition (00a02900)` (0), `ScenarioFuncSet::disable_give_attrition (00a02940)` (1); `Leader::init` (0) | set/clear by script, slot = arg − 1, live leaders only | init | none (trigger layer cut) | no | high |
