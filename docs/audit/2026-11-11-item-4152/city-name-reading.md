# Item 4152, family 1 (`city-name`) — first reading

- Family: 1, slug `city-name`.
- Leaves: `city_name` (`LeaderData` +0x40c, int).
- Reader: Claude Opus 5.5 (`claude-opus-5-5`), first reading, 2026-11-11.
- Sources: the decompile export (`~/ghidra-projects/decomp`), the listing of
  `riseofnations.exe` (`llvm-objdump`), `crates/sim/src`.

## Layout check (before the leaf)

- `types.txt` 42152: `LeaderData +0x40c int city_name`; neighbours `city_mark`
  (+0x408) and `lost_city_stamp` (+0x410).
- `Leader` (0x6eec) holds `LeaderData` at offset **0**, not behind a prefix:
  `Leader::init (006e3930)` stores `who` to `0x8(%ebx)`, `tribe` to
  `0xc(%ebx)`, `gov = -1` to `0x14(%ebx)` and 0 to `0x40c(%ebx)` (listing
  6e3af9 to 6e3b14), and `Leader::lost_capital`/`recapture_capital` use
  `field_0x414`/`field_0x41c` as `lost_capital_stamp`/`lost_capital_modifier`.
  So in a `Leader*` method `field_0x40c` **is** `city_name`; `field_0x414` is
  `lost_capital_stamp`, not this leaf. The 8 extra bytes are a tail.
- The global array is `Leader[10]` at `0xe3a390`, stride 0x6eec:
  `Leaders` static init at 006ed919 pushes `0xe3a390, 0x6eec, 0xa` with
  `Leader::Leader (006d2bf0)` and `Leader::~Leader (0042de10)`;
  `Leaders::init (006ed850)` loops to `0xe7f8c8` = base + 10 x 0x6eec.
  Slot `i`'s `city_name` is `0xe3a79c + i x 0x6eec`.

## Leaf `city_name` (`LeaderData +0x40c`, int)

### 1. Writers, by both nets

Export net: `->city_name` hits only `LeaderData::log_data (006e5110):699`
(a reader). `field_0x40c` hits on a leader base: `City::generate_name
(00735c90)` lines 226-229 and 263-264, and `Leader::init (006e3930):139`.
The other `field_0x40c` hits (`IFaceEditorOptions::setup_selected`) are
another class. `field_0x414` (the offset plus 8) hits are all
`lost_capital_stamp` on a `Leader` whose base is 0 (above), not this leaf.

Listing net, whole `.text` disassembled once:
- absolute `0xe3a79c` (slot 0 and indexed): 8 sites. Stores: `incl
  0xe3a79c(%eax)` at 73614a and `movl $0x1, 0xe3a79c(%eax)` at 7362d2, both
  in `City::generate_name`. Reads: 736015, 73601d, 736144, 7362c9 (same
  function) and 9a791e (`ScenarioWrite::save_leaders_chunk (009a77c0)`).
- absolute slot addresses 1..7 (`0xe41688`, `0xe48574`, `0xe4f460`,
  `0xe5634c`, `0xe5d238`, `0xe64124`, `0xe6b010`): every hit is a read inside
  `City::generate_name`'s unrolled scan; slots 8 and 9 (`0xe71efc`, ...) have
  no hit at all.
- register-relative `0x40c(%reg)` as a store destination (any mnemonic,
  including single-operand `inc`/`dec`), every function in `.text`: Leader
  family only `Leader::init` (6e3b14). The rest are other classes:
  `ConquestLeader::ConquestLeader` (a 16-bit store, the CtW meta leader, not
  `LeaderData`), `ConquestMapLoader`, `GraphicWinXML::close`,
  `RTreeData::clear`, `RiverSectionOut`, `TerrainBorder` (ctor/dtor),
  `TileSetData` (x2), and `ScenarioRead::load_leaders_chunk (009acde0)` at
  9ad098, which is clearing the `GameInfo` player records (the decompile is
  `(pGVar7->info).player[0]...`, and the next store is a 16-bit write to
  `+0x414`, which is an int in `LeaderData`): not this leaf.
- no whole-record copy: no `$0x6ee4` or `$0x1bb9` operand anywhere; the four
  `$0x6eec` pushes are the array's constructor/destructor iterators.

**Writers, final:** `City::generate_name (00735c90)` (two stores) and
`Leader::init (006e3930)` (the clear). This agrees with the README row's
single named writer, and adds the second store in it and the clear.

### 2. The event, the step, the predicate

`City::generate_name (00735c90)` runs once per **named city record**: its
only caller is `City::init (00737050):203` (call at 737409), under `param_4
== 0`. `City::init`'s only caller is `Cities::init_city (007352c0)` (call at
7353ef), which passes its own third stack argument through; that function's
only caller is `Build::activate (00623e20)` (call at 623fc7), in the city
branch (`SubObjectData.flags & 0x20` on the building, the `else` at
activate:446, after `UnbuiltCities::remove_unbuilt_city`), passing
**activate's first argument**. So the counter can step only when a city
building is activated with first argument 0. `Build::activate` is vtable slot
`+0x1a8`; its callers by listing pushes (first argument is the last push):

| caller | args | a city reaches it? |
|---|---|---|
| `Setup::build_cities (005ab910)` 5ababf | (0,0,0) | yes: the starting city (type 0x19e) when the game's `starting_town` option is set and `leader_flags2 & 0x80` is clear |
| `Wall::do_construct (006434d0)` 643663 | (0,1,1) | yes: a city building finishing construction |
| `Leader::produce_building (006e1400)` 6e2e0a | (0, param_2, 0) | only when `game->frame == 0` (otherwise it sends builders, `action_swarm_around`, and returns): scenario and CtW placement |
| `Cities::capture_city (00733380)` 733687, 73390a | **(1,1,0)** | a captured city is re-initialised **without** a new name and **without** a step |
| `Build::check_capture (006276a0)` 627f82 | (0,1,0) | no: its city arm (`flags & 0x20`, :257) goes to `capture_city`; the (0,1,0) call is the non-city arm after an owner change |
| `Leader::defeat_by (006d1c80)` 6d20ec | (0,1,0) | no: guarded by `(flags & 0x20) == 0` (:159) |
| `City::find_buildings (007384c0)` 7388a8 | (0,1,0) | no: guarded by `(flags & 0x20) == 0` (:76) |
| `SpellType::cast_bribe (00672d00)` 6734e2 | (0,1,0) | no: a bribed city goes to `capture_city` (:154); the (0,1,0) is the non-city arm |
| `ScenarioRead::load_city_chunk`, `load_build_chunk`, `ScenarioEditor::add_build`, `activate_selected`, `ScenarioFuncSet::create_building` (0,1,1), `ConquestGame::build_cities` (x2), `Console::hurry_action` (0,1,1) | first arg 0 | yes, but scenario load, editor, triggers, CtW, console |
| `ScenarioFuncSet::switch_teams` | (1,1,0) | no step |

(The decompile prints `(0,1)`, `(0)` and `()` for several of these; the
listing has three pushes at each.)

Inside `generate_name`, three early exits take no step:
- the city's building type answers `is(0x213)` (`FORBIDDENCITY`): the name is
  the type's own;
- `game->semaphore[2] & 2` (Conquer the World) and the tribe is not
  barbarian (`Tribe +0x58`): `ConquestGame::get_city_name`, and only from
  frame 1;
- the tribe's XML file or its city list is missing or empty (error paths,
  name = tribe name).

Otherwise, with `own` = the city's owner slot (`City +0x5e`), `T` = own's
`tribe` (+0xc), `L` = the tribe's city-list length:

1. **Pick the slot that takes the increment.** Scan slots 0..7 (hard-coded,
   unrolled; slots 8 and 9 are never scanned, nor own if own >= 8) for
   `tribe == T`. Track the first matching slot and the largest `city_name`
   among the matches, replacing on strictly greater, so ties keep the lowest
   slot. The max starts at -1 and slot 0 enters only if its value is > -1
   (listing 73600b-73601d). **If that max is > 0 the winner is the slot that
   holds it; otherwise the winner is own** (`testl %ecx,%ecx; cmovgl`,
   73613b). There is **no liveness test**: a defeated or resigned leader of
   the same tribe still holds its counter and is still scanned. Slots that
   were never set up have `tribe = -1` (`Leaders::init` passes -1) and never
   match a real tribe.
2. Read `prev = city_name[own]` **before** the step (736144), then
   `city_name[winner] += 1` (73614a) and `n = city_name[winner]` after it.
   Step: **+1, a plain count, no scale.**
3. If `n >= L`: name = tribe name + a suffix string + `(n + 1)`; done.
4. Else if `n <= 1` or `prev == 0` (signed tests, 7361c4/7361cd): index =
   1 + the number of slots `0..own-1` whose tribe is `T`; **and if
   `city_name[own] == 0`, store 1 there** (7362c9/7362d2) - the second
   store. This is a leader's first named city when a tribe-mate already holds
   the shared count: the shared slot was stepped in 2, and own is marked 1.
5. Else: `k` = number of slots 0..7 with tribe `T` (own included);
   `m = L - k + 1`; `x = (T + world->seed) % m + 1` (`world +0x7c`, signed
   `idivl`); then step a Galois LFSR - if `x` is odd, `x ^= 0x170`; then `x
   >>= 1` (arithmetic) - counting only values `x < m`, until `n - 1` such
   values have been produced; index = `x + k`. If `L < index`, name =
   tribe name + suffix + `(index + 1)`, else the list's `index`-th entry.

So in the common skirmish (every leader a different nation) the winner is
always own, and `city_name[slot]` is simply the number of named city
foundings by that slot this game, the starting city included. With
tribe-mates, the count is shared: it accumulates in the lowest slot holding
the largest value once that value is positive, and each other tribe-mate
sits at 1 after its first city.

### 3. What clears it

`Leader::init (006e3930):139` stores 0, before `reset_score` (which does not
touch it; it is not a score field). `Leader::init`'s callers (listing):
`Setup::build_leader (005abc70)`, `Setup::build_game (005ac190)` (each live
slot, `leader_flags & 1`), `Leaders::init (006ed850)` (all 10 slots, tribe
-1), `WalkDataGame::walk_data (005a2360)`, `MapScenario::make (0068c8f0)`,
`ConquestGame::build_conquest_game (00797c40)`, `ScenarioEditor::new_leader
(009a1490)`, `ScenarioRead::load_leader_chunk (009ac080)` x3. No per-frame
fold.

### 4. Readers

- `City::generate_name` itself: the value chooses the city's name (the
  only decision it feeds).
- `LeaderData::log_data (006e5110):699`: logged under the key `city_name`
  (string at 0xadf5a8, length 9). The crate already lists it:
  `crates/sim/src/checksum/leaders.rs:98` `("city_name", 0x40c, 4, 1)`, and
  `crates/rondata/src/diff/checksum_lines.rs:1772` `("city_name (unheld)",
  2)`.
- `ScenarioWrite::save_leaders_chunk (009a77c0)` 9a791e copies it into the
  saved leader record beside `who` and `tribe`. I found **no** store back in
  `ScenarioRead::load_leader_chunk` (no `0x40c(` store in its range, no
  whole-record copy), so on a scenario/save load the value would restart at
  `Leader::init`'s 0 and be re-stepped by the city chunk's (0,0,0)
  activations - not read through; see 7.

### 5. Where `crates/sim` has the event

`Sim::activate` (`crates/sim/src/cities/city.rs:2420`) calls
`Sim::init_city` (`city.rs:451`) at `city.rs:2509` when
`building_is_city(b)`, with `captured` standing for activate's first
argument (true only from `capture_view.rs:236`, the capture path). A builder
adds a per-leader `city_name` beside the other `LeaderData` stand-ins, steps
it in `init_city` (or at the 2509 call) when `!captured` and the city's
building type is not `FORBIDDENCITY` (0x213), with the tribe-shared winner
rule and the `= 1` store above, zeroes it where the crate runs
`Leader::init`, and reads it in `checksum/leaders.rs`'s field match (where
`city_mark` sits, ~line 1020). The crate has no city-name strings at all
(`ai/ai_host.rs:445` returns a synthetic `City {c}`, and says why), so the
LFSR and the list index are not needed for the counter: only steps 1, 2 and
4's store move it. CtW and the XML error exits are outside the crate.

### 6. Reached by a plain game?

Yes. A skirmish against the computer steps it at `Setup::build_cities`'
starting city (when the lobby's starting-town option is on; whether that is
the default was not checked) and at every city a
human or the AI founds and finishes (`Wall::do_construct`). Capture never
steps it. The tribe-shared branch needs two leaders of one nation, which a
skirmish lobby allows (random nations can collide). The frame-0
`produce_building`, scenario, editor, trigger, console and CtW callers are
outside a plain game.

### 7. Confidence and what is not read

- High on the writer set (two nets agree, the listing sweep covered every
  `0x40c(%reg)` store and every absolute slot address) and on the step
  arithmetic and predicate (read from the listing, 735ff3-7362d2).
- Not established: whether a skirmish AI can call `Leader::make_this` ->
  `produce_building` on frame 0 (then that path would also step it); which
  building types answer `is(0x213)` and carry `flags & 0x20`; whether a
  save/scenario load restores the value (I found no store in
  `load_leader_chunk`, but did not read `load_city_chunk`'s order against it);
  the `+0x1a8` caller list came from a grep for `+ 0x1a8))(` in the export
  and may miss a caller that reaches the slot another way (a non-`Build`
  class's `+0x1a8` is a different method; I checked each listed site's
  object came from `objects` via `+0xac`/`+0xb0` or `this`). The order of
  `Setup::build_game`'s `Leader::init` against `build_cities` was not traced
  line by line.

## Summary

| leaf | writers | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `city_name` (+0x40c) | `City::generate_name (00735c90)`: `+= 1` on the winner slot (73614a), `= 1` on own when 0 (7362d2); `Leader::init (006e3930)`: `= 0` (6e3b14) | +1 per named city record: `Build::activate` city branch with first arg 0 (starting city, construction finished; not capture), not `FORBIDDENCITY`, not CtW non-barbarian; winner = lowest slot 0..7 of own tribe with the largest value if > 0, else own; plus own = 1 when it is 0 on the first-name branch | `Leader::init` | `Sim::init_city` (`cities/city.rs:451`), called from `Sim::activate` (`city.rs:2509`) when `!captured`; read in `checksum/leaders.rs` | yes | high (listing-backed); frame-0 AI and load paths open |
