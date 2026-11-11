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

