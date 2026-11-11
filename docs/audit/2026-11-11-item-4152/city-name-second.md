# city-name — second (blind) reading

- slug: `city-name`
- leaves: `city_name` (LeaderData +0x40c, int)
- reader: Claude Opus 5.5 (claude-opus-5-5), blind; has not opened the first reading
- sources: Ghidra export `~/ghidra-projects/decomp`, `types.txt`, PE listing (`aa/exe.s`)

## Layout check

`Leader` (0x6eec) holds `LeaderData` (0x6ee4) at offset 0: `Leader::Leader (006d2bf0)` puts its
`_vbtable_` at `+0x6ee4`, and `Leader::lost_capital (006dc870)` reads `this->field_0x414/0x41c`
as `lost_capital_stamp`/`lost_capital_modifier`. So in a `Leader` method, and in
`leaders.list[i].field_0x40c`, the raw offset is `LeaderData::city_name` with no shift. In
absolute form slot `i` is `0xe3a79c + i*0x6eec` (`0xe3a390 + 0x40c`).

## `city_name` (LeaderData +0x40c, int)

A per-leader counter of cities named from the civilization's city list. It is not a
string or an index into a string table held by the leader. It is the running count that
`City::generate_name` uses to pick the next name from the tribe's XML list.

### 1. Writers, by both nets

Export: `->city_name` hits only a reader (`LeaderData::log_data`). `field_0x40c` hits
`City::generate_name` (5 lines) and `Leader::init` (1 line). The other `field_0x40c` hits are
other classes (`GameReportWin`, `ConquestInvadeWin`, `IFaceEditorOptions`). `field_0x414`
(the shift-by-8 candidate) hits only `lost_capital_stamp` users, which confirms there is no shift.

Listing (`aa/exe.s`). All 4 absolute `0xe3a79c` sites are in `City::generate_name`; the one
other absolute reference is a reader in `ScenarioWrite::save_leaders_chunk`. Of the 51
`0x40c(%reg)` sites, I mapped each through `INDEX.tsv` and named the base:
- `6e3b14 movl %edx,0x40c(%ebx)` in `Leader::init (006e3930)`. `ebx` = `this` (Leader*). **Writer.**
- `9ad098 movl %eax,0x40c(%edx)` in `ScenarioRead::load_leaders_chunk (009acde0)`. `edx` =
  `GameAccess::game` (0xc061ec): this is the `info.player[]` clear (stride 0x8c, with a `movw`
  at +0x414), not a LeaderData. Not a writer.
- `6e6abf pushl 0x40c(%edi)` in `LeaderData::log_data (006e5110)`. Reader.
- The rest are other classes: `-0x40c(%ebp)` locals, `GraphicWinXML`, `TileSetData`,
  `RiverSectionOut` (a float 1.0), `TerrainBorder`, `RTreeData::clear`, `ConquestLeader`
  (`movw`), `IFaceEditorOptions`, `CP2PAuthPlayer`, and `ConquestGame`/`ConquestLeader` `lea`.

Whole-record writers, which the displacement net cannot see:
- `Leader::init (006e3930)`, when `param_2 < 0`: `memset(this, 0, 0x6929)` at `6e3995`
  (pushes `0x6929, 0, ebx`). This covers +0x40c.
- `Leaders::clear (006ed5d0)`: `memset(…, 0, 0x6929)` per leader. Called from
  `Game::close (00594b80)` and `Game::run_oos_recovery (005869e0)`.
- `LeaderData::walk_data (006d6750)` in input mode. If `leader_flags & 1`, it walks the raw
  byte range `[&who (+0x8), +0x692a)`, which contains +0x40c. Reached from
  `Leaders::walk_data`, `WalkDataGame::walk_data` (save-game load), and
  `LoadGame::verify_load`.

Writers that store a value:
1. **`Leader::init (006e3930)`**: unconditional `city_name = 0` (`6e3b14`), after the
   `+0x8/+0xc/+0x14/+0x6928` setup and before `reset_score`. Callers (listing `call 0x6e3930`):
   `Leaders::init`, `Setup::build_game`, `Setup::build_leader`, `MapScenario::make`,
   `ScenarioRead::load_leader_chunk` ×6, `ScenarioEditor::new_leader`,
   `ConquestGame::build_conquest_game`, `WalkDataGame::walk_data`.
2. **`City::generate_name (00735c90)`**: `incl 0xe3a79c(%eax)` (`73614a`) and
   `movl $1,0xe3a79c(%eax)` (`7362d2`).

### 2. The event, the step, the predicate

**Event.** A new `City` record is named. There is one call site, `City::init (00737050)` at
`737409`, taken iff `City::init`'s 4th argument is 0 (otherwise it closes the name string at
City+0x90). `City::init` is called only by `Cities::init_city (007352c0)`, which passes its
own 3rd argument through. `init_city`'s only caller is `Build::activate (00623e20)` at `623fc7`,
which passes **its own `param_1`**. That call sits in the branch where the building has the city
flag (`Build+8 & 0x20`). In that branch every activation makes a City record. So:
**a city building activated with `param_1 == 0` steps the counter, and one activated with
`param_1 == 1` (a transfer) does not.** `Build::activate` is virtual, `Build::vftable +0x1a8`.
Call sites with first argument 0:
- `Setup::build_cities (005ab910)` `(0,0,0)`: each player's start city.
- the construction-complete path (the export files it as `Wall::do_construct (006434d0)`; I did
  not check whether the body is folded) `(0,1,1)`: a built city finishing.
- `Leader::produce_building (006e1400)` `(0,param_2,0)`: instant placement. Its callers are
  `Setup::build_civ_specific`, `Setup::small_city_buildings`, `Setup::large_city_buildings`,
  `Leader::make_this`, `ConquestGame::build_cities`, and `ScenarioFuncSet::place_orphan_*`.
- `ScenarioRead::load_city_chunk`, `load_build_chunk`, `ScenarioEditor::*`,
  `ConquestGame::build_cities` `(0,0,0)`, and the console `hurry_action` `(0,1,1)`.

These never reach the step: `Cities::capture_city (00733380)` and
`ScenarioFuncSet::switch_teams` pass `(1,1,0)`. `Build::check_capture (006276a0)`,
`City::find_buildings (007384c0)` and `Leader::defeat_by (006d1c80)` pass `(0,1,0)` or
`(0,1)`, but only on buildings without the city flag: `check_capture` returns through
`capture_city` when the flag is set, and `defeat_by` excludes `+8 & 0x20` before its call at `6d20ec`.

**Gates inside `generate_name` before the step** (listing `735d51–735ff3`):
- (a) The City's building object has `is(0x213 = FORBIDDENCITY, 0)` (`ObjectTypeData::is`:
  its own type, or the type's `is_list`). Then the name is the type's own name. **No step.**
- (b) `game+0x822 & 2` (`semaphore` byte 2 bit 1, the Conquer the World flag;
  `Game::is_scenario (0043ec70)` tests the same bit) **and** `Tribe.barbarian` (+0x58) of
  the owner's tribe is 0. Then the name comes from `ConquestGame::get_city_name`, or
  nothing if `game+0x550` (frame) < 1. **No step.**
- (c) Otherwise the game loads the tribe file's (`Tribe.file`, +0x4) cities element. If that
  element is missing, or its list has 0 entries, the name is the tribe's name (`Tribe.name`,
  +0x18). **No step.**
- (d) Otherwise **the step happens**. `N` = number of names in the list.

**The step**, with `own` = the City's owner (`City+0x5e`, signed char) and `T` = `LeaderData.tribe`
(+0xc) of `own`:
1. Scan slots **0..7 only**. These are 8 hard-coded compares, `0xe3a39c … 0xe6ac10`, so slots
   8 and 9 are never scanned. Over the slots `k` whose tribe equals `T`, find the largest
   `city_name[k]`, taking the lowest `k` on a tie (strict `>` update, `jle` skip).
2. `target` = that slot if the maximum is > 0, else `own`.
3. `old` = `city_name[own]`, read **before** the increment (`736144`).
4. **`city_name[target] += 1`** (`73614a`). `new` = the result.
5. If `new >= N`, the name is `Tribe.name + <loc string 0x15e0> + (new+1)`. Done; no further write.
6. Else if `new <= 1` or `old == 0`: the name index (1-based) is 1 + the number of slots
   `j < own` (no upper bound of 8 here) whose tribe equals `T`. Then **if
   `city_name[own] == 0`, set it to 1** (`7362d2`). If the index exceeds `N`, use the numbered
   name with that index; else use list[index−1].
7. Else: `S` = the number of slots 0..7 with tribe `T`, and `M = N − S + 1`. Start with
   `x = (T + world.seed) mod M + 1`. Here `world.seed` is `WorldData +0x7c`, at
   `GameAccess::world` (0xc06188); C `idiv`, so the remainder takes the dividend's sign.
   Iterate `x ← (x odd ? x ^ 0x170 : x) >> 1` (arithmetic shift). Count the values with
   `x < M`; stop at the `(new−1)`-th. The index is `x + S`, which goes to list[index−1]
   (or the numbered form if index > N, which cannot happen for `x < M`). No write to
   `city_name` here beyond step 4.

All values are plain ints; there are no floats or scaled units. Each named city adds one to
exactly one slot's counter, plus at most one more write of 0→1 to `own`. With players of
distinct civs, `target == own` always, so the counter is `own`'s count of cities named from
the list. That excludes Forbidden-City-typed cities, CtW names, transfers, and missing or
empty lists, but the count keeps rising past `N` (the numbered names). With two or more
players of one civ, the count of cities named across that group goes to the slot holding the
maximum, and each other member's own slot is set to 1 by its first naming.

Worked case. Slots 0 and 1 share a civ; every city is named in a plain game.
P0's start city: P0 0→1, using list[0]. P1's start city: the target is P0, so P0 1→2; P1's
`old` is 0, so the index is 2 (list[1]) and P1 0→1. P0's next city: P0 2→3, which takes the
LFSR branch with S=2.

### 3. What clears or resets it

`Leader::init` sets it to 0 every time, and to 0 through the `memset` as well when
`param_2 < 0`. `Leaders::clear` memsets it at `Game::close` and on OOS recovery. A save-game
load (`walk_data` input) restores the saved value. Nothing decrements it. Losing a city
(`City::close`, `lost_a_city`, `capture_city`) does not touch it: I grepped every writer,
and none sits on a loss path.

### 4. Readers

- `City::generate_name` itself (the scan, `old`, and `new`).
- `LeaderData::log_data (006e5110)`, `6e6abf`: the leader dump prints it under the label
  `city_name` (`.rdata 0xadf5a8`, UTF-16), right after `city_mark` (`0xadf5bc`).
- `LeaderData::walk_data (006d6750)`, output mode: saving, and **the sync checksum**
  (`CheckSums::check_one_leader (00937020)`, `check_leaders`, `check_all`), for leaders with
  `leader_flags & 1`. It sits inside the raw range `[+0x8, +0x692a)`.
- `ScenarioWrite::save_leaders_chunk (009a77c0)`, `9a791e`: it copies the value into the
  leader-properties chunk at +0x10 (`LeaderPropertiesChunk*.city_name_index`). No reader of
  chunk +0x10 exists in `ScenarioRead::load_leader_chunk (009ac080)`: its three branches read
  +0x0, +0x4, +0xc, +0x14, +0xdc, +0xe0 and +0xe8, and each calls `Leader::init`, which zeroes
  it. So a scenario file carries the value, but loading drops it.

### 5. Where `crates/sim` has the event

- The founding is `Sim::activate(b, captured, counted)`, `crates/sim/src/cities/city.rs:2420`.
  Its city branch at `city.rs:2506–2509` calls `Sim::init_city(who, b, captured, capital)`,
  `city.rs:451`. `captured` is `Build::activate`'s `param_1`. **A builder adds the counter in
  `Sim::init_city` (or right after it in `activate`) under `!transfer`**, using steps 1–4 and 6
  above. It needs the owner's tribe for each slot 0..7 and `N > 0`, plus gates (a) and (b).
  Setup's start city reaches the same `activate` path (`crates/sim/src/setup.rs:94`).
- The crate has no city naming at all: no `generate_name`, no tribe city list, no
  FORBIDDENCITY test on a city record. `ai/ai_host.rs:445 city_name(c)` is an unrelated AI
  script label. The counter needs `N` only to decide gate (c), not for the step itself.
- The checksum already lists the leaf: `crates/sim/src/checksum/leaders.rs:98`
  `("city_name", 0x40c, 4, 1)`. `crates/rondata/src/diff/checksum_lines.rs:1772` records it as
  `city_name (unheld)`. A builder adds a held arm beside `"city_mark"` / `"cities_built"` in the
  name match at `checksum/leaders.rs:~1026–1037`, with storage in `self.cities.city_tally[w]`
  or a new per-leader field. The coverage list in `crates/rondata/src/diff/coverage.rs:295`
  already names it among the logged leader ints.

### 6. Reached by a plain game?

Yes. A skirmish against the computer reaches `Leader::init` (setup), `Setup::build_cities`
(each start city: a step), and construction completion (every newly built city: a step).
That step runs in the plain branch (c)/(d), because the CtW bit is clear. `Leaders::clear` is
reached at game close. Not reached in a plain game: the ConquestGame naming branch (b), the
scenario loaders and editor, `ScenarioFuncSet` placement, and the console hurry. Gate (a)
needs a city record whose building `is` FORBIDDENCITY. I did not establish whether any
plain-game city building type lists FORBIDDENCITY in its `is_list`, or whether the Forbidden
City wonder makes a City record. A check against the type tables (`rondata`) would settle it.

### 7. Confidence, and what I did not read

**High** for the writer set: both nets were run; the 51 register sites and the 4 absolute
sites were each mapped to a function, and the whole-record writers were added by reading.
**High** for steps 1–6: I read every instruction from `735ff3` to `7362dc`, and they agree
with the decompile. **Medium-high** for step 7's LFSR: the listing was read and the
arithmetic is plain, but I did not prove the loop terminates for every `M`, and did not test
a negative `T + seed`. `Setup::build_game` writes `seed = x % 0xffff + 1`, and I did not read
`x`'s sign.

Not read:
- the body of `ConquestGame::get_city_name`;
- the XML element tags at `int_str_array+0xb90` and `+0x2864`;
- the string at `loc_str_array_orig+0x15e0`;
- whether `Leader::produce_building`'s Setup callers (`small_`/`large_city_buildings`,
  `build_civ_specific`) ever place a *city* building in a plain start (if they do, each steps);
- the full `Build::activate` city branch, beyond the `0x20` split, line 446 onward, and the
  `init_city` call;
- `Leader::make_this`'s callers.

## Summary

| leaf | writers (Name (addr)) | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `city_name` (+0x40c, int) | `City::generate_name (00735c90)` (`incl` at 73614a, `=1` at 7362d2); `Leader::init (006e3930)` (`=0` at 6e3b14, plus a `memset 0x6929` when `param_2<0`); `Leaders::clear (006ed5d0)` (memset); `LeaderData::walk_data (006d6750)` (save-game load, raw range) | +1 per city named from the tribe's XML list (`Build::activate` of a city building with `param_1==0` → `Cities::init_city` → `City::init` → `generate_name`), on the same-civ slot 0..7 holding the largest count (or on `own` if every count is 0). On the first naming, `own` goes 0→1 when it was not the target. No step for a FORBIDDENCITY-typed city, CtW non-barbarian names, transfers (`param_1==1`), or a missing or empty list | `Leader::init` (setup, scenario load), `Leaders::clear` (`Game::close`, OOS recovery); never decremented | none yet: add under `!transfer` in `Sim::init_city` (`crates/sim/src/cities/city.rs:451`, called from `Sim::activate` `:2509`); checksum leaf `checksum/leaders.rs:98`, now `unheld` (`rondata/src/diff/checksum_lines.rs:1772`) | yes: start cities (`Setup::build_cities`) and every completed city; gate (a) reachability not established | high (writers, steps 1–6, listing-checked); medium-high (step 7 LFSR termination and sign) |
