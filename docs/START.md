# The start of a game

How frame zero is built from a lobby's choices, with no dump of the
original: the start track's specification (`docs/TRACKS.md`, the start
row; `docs/DECISIONS.md` entry 115 §3). The walk was first read for the
harness in `docs/ORDERS.md` §9.1–9.4, which stays the reading of record
for each function's body; this document is the walk as a build order —
what runs, in what order, what each step draws, which of them this crate
carries, and what the comparison against the installed start says.

The oracle is the start block the original dumps at `GameLog::begin_game`
(`[Start Game]`, frame 0, after `Setup::build_game` and before any
`do_frame`; ORDERS §9.1). `rondata::diff::setup::build_sim` installs that
block into a `Sim` today; the start track replaces the installer with a
builder, lobby by lobby, and `rondata::diff::start` compares the two.

## 1. The draws are the spine

The setup path's checksum trace (`docs/ORACLE.md`, "The setup path's
checksum trace is the RNG state", run11) is the walk's own ledger: every
`say_checksum` on the path prints `game_random`'s word, so the number of
draws each step spends is counted, not inferred. On the harness lobby
(seed 12345, Great Lakes, Small, two players, Standard resources):

| step | draws | where |
|---|---|---|
| `init_rules_and_teams` — the AI's random nation | 1 | `rand % 24` (`init_tribes`) |
| `Setup::build_game` re-seeds from `info.seed`; `world->seed` | 1 | `rand % 0xffff + 1` |
| `Map::make` | 3, 5568, 1016, 3, 1824, 231, 2920 | the map maker (`docs/MAPGEN.md` §2) |
| `make_rivers`; `Terrain::init` | 173; 0 | |
| the start permutation | 8 | `rand & 7`, re-drawn until fresh |
| `Leader::init`, the AI; the human | 20; 0 | `random_personality` |
| the two `build_empire`s | 1293 | the placer, `place_unit`, the pasture |
| `Herd::create_units` | 44 | |
| the rest of `Game::init` | 0 | |

A builder that reproduces the walk lands on the trace's last word
(`0x3bd39ae9` here) and hands it to frame 0; one that spends a different
count anywhere parts on every draw after. The re-seed matters: the nation
roll and `init_starting_resources` (inside `init_rules_and_teams`) and
everything from the map on are **two walks of the same seed**, not one
stream (ORDERS §9.2, S1).

## 2. The walk, step by step

Each step names what it writes, how the claim was established, and whether
`rondata::diff::start::from_lobby` carries it (**built**) or stops before
it (**not built**).

1. **The extent — `Game::init@0058c480`.** `World::init@006b76f0(n, n)`,
   `n` the `rules.xml` `mapsizes` row `MAP_SIZE` indexes (the record's
   `DATA`, read at `+0x3c` of a 0x58-byte record); on map style 16 `n × 11
   / 10`, rounded up to even and capped at 100; halved (rounded up to even)
   when the game's `semaphore[1] & 2` is set, which is not a lobby field.
   *Read* (the decompile, lines 386–399), *diff-backed* on one lobby (60 on
   Small); the style-16 and half-size arms are read only. **Built**
   (`world_extent`), the half-size arm not.
2. **`Game::init_rules_and_teams@00589bb0`** — in order: the starting age
   (`starting_technology == 9` is random), `Map::set_map@006a1340` (a
   random style draws `r % 17 + 6` until its table admits it; a fixed
   style draws nothing), `Game::init_starting_resources@0058a500` (ORDERS
   §9.4: a draw per good only when the lobby row has a spread),
   `Leaders::init`, the team assignment (`Game::init_teams@0058ae70`), the
   handicaps, and the nations (`rand % 24` for a random one). **Built**:
   the lobby, the nations off the `PLAYER` rows, the human bit (`flags &
   4`), the opening techs and the goods (`Sim::new`'s `starting_goods`). The
   nation draw is not walked: `build_game` re-seeds right after, so nothing
   at frame 0 observes it. **Not built**: a random
   style, a random age, a spread row's draws (refused, with a note), the
   team assignment. (The war matrix is not this step's: it is
   `Leader::init`'s, step 6.)
3. **`Setup::build_game@005ac190`'s head**: re-seed from `info.seed`,
   `frame = tick = 0`, the market tables, `Objects::clear@0065d740` (per
   player `unit_mark = 0`, `build_mark = 2000`, `wall_mark = 3000`), and
   `world->seed = rand % 0xffff + 1`. **Built**; *diff-backed* (`WORLD
   seed` 7236 on both sides).
4. **The map — `Map::make@0068bc90`, `Terrain::init@00850f70`,
   `mark_forest`.** The start cells are the map maker's (`world->start_x/
   start_y`). Every phase of fifteen styles is ported and checked at its
   own checkpoint (`docs/MAPGEN.md` §15, the `Mapgen:` line), each driven
   from the trace's 7805 word and the start dump's `World.seed`.
   **Built for Great Lakes** (item 4047, `rondata::mapgen::lakes`): the
   chain on one stream from `setup.cpp` 716, `load_map_data`'s two
   tileset draws and `base_edge` walked forward (`tileset_draw`), then
   `Terrain::mark_forest` (`MapGrid::mark_forest`: a forest tile of a
   FOREST cell gives 0x8 to twelve tiles up-left of it, the offsets read
   off the PE). `world_from_grid` stands the `Sim` on the grid as
   `world_from` stands it on a dump. *Diff-backed* on the harness lobby:
   seed and parts 2, 5, 6 and 10–13 at all 19 checkpoints 7805 → 884
   (part 5 at 7805 excepted: `World::init`'s cells, which
   `make_continents` overwrites). **Built for East Indies and Great
   Sahara** (item 4186), **and for Southwest Mesa and the eleven styles
   `mapgen`'s chain walk carries** (item 4196: Old World, Amazon Basin,
   Himalayas, African Watering Hole, Mediterranean, Australian Outback,
   New World, Colonial Powers, East Meets West, Sea Power, Nile Delta).
   Past Great Lakes' and East Indies' order, every style takes the
   chain walk's tail: `Terrain::init` without heights when no cell
   opens the half-land gate, Great Lakes' half-land path when one does,
   the rivers, the heights on the grid the rivers left, the troughs with
   `move_goody`'s good arm, `adjust_for_mountains` and `Cliffs::init`.
   Its mining lists are laid on a Mesa map (`Sim::place_cliffs`), and
   are inert elsewhere. `Map::make` also copies the style's six
   territory limits into `world+0x38..+0x4c`. Each
   `PLAYER_/COLONIZED_TERRITORY_{BASE,CIVIC,CITY}_HARD_LIMIT` the file
   names is written by `Map::init_map_data@0069f250` over the rules'
   `TERRITORY_LIMIT_*` (`rondata::mapgen::territory_limits`; Nile
   Delta and Sea Power 48/8/8, New World 44/4/4 and 32/4/4). East Meets
   West refuses a team game, and Colonial Powers refuses team style 7
   (both arms unread). **Not built**: Warring States and British Isles,
   which have no mask port.
5. **The start permutation**: `Game::start_list[k]`, a random permutation
   when the map randomises starts (`rand & 7`, 8 draws here), else
   identity; teammates placed consecutively in a team game unless `world
   +0x30 == 0x16` (ORDERS §9.2, S5). The installer recovers the visiting
   order from the trace (`leader_visit_order`). `randomize_starts` is
   `Map::Map@006a06e0`'s 1. Only East Meets West's
   `make_continents@00696640` writes it again, to 0, at its head
   (`Map+0x2ec`, the type record; item 4196). There the permutation is
   the identity and draws nothing. *Diff-backed* on run1524's lobby: with
   the randomised permutation its seats swap, `BUILD 0/2000 pos` ours
   28512 30816 against theirs 11616 12384. **Built** (`start_permutation`,
   item 4047), randomised; *diff-backed*: the eight draws from 894's word are
   4, 7, 6, 1, 0, 3, 2, 5, each fresh first time, landing on 955's
   `0x9991b076`. Teammates: not built (no team game on disk).
6. **`Leader::init@006e3930(who, tribe, who)`** for every active player:
   the leader's records, and for a computer leader the personality roll
   (`Personality::roll`, 20 draws here) and the script choice. The roll is
   ported and reproduces the trace's far word from its near one (ORACLE);
   what it needs is the stream's word at its bracket, which steps 4–5
   produce. **Built** (item 4047): `init_leader_ai` for each computer
   seat in the permutation's order, after `load_scripts`; *diff-backed*:
   seat 1 lands on 13457's `0xf2299eda` and its `PERSONALITY` agrees.
   **The diplomacy row** is the same call's, and draws nothing: for each
   other leader `u`, `diplos[u]` (`LeaderData +0x74`, written as
   `puVar23[-0x18]` off a cursor at `good_deeds`, `+0xd4`, lines 199–227 of
   the decompile) is left alone in a conquest game (`semaphore[2] & 2`);
   else 2 when `LeaderData::is_team@006ebd30(u, 0)` holds at frame 0 (the
   same `PLAYER` `team` in `0..4`, refused under team style 7 to a seat on
   team 8); else 1 when `RUSH_RULES` is above the leader's `starting_age`;
   else 0, war, when `Game::teams_locked@00594880` (team style not 0, 8 or
   11), and 1 otherwise. Then `semaphore[1] & 2` forces 0 off the
   diagonal, which no lobby field sets. **Built** (`Sim::init_diplos`,
   item 4046), ahead of the map, which it does not read; the
   `semaphore[1]` arm and the loop's other writes (`treaties`,
   `aggression`) not.
7. **`Setup::build_empire@005abb80(who, slot)`** for each player in start
   order: `Setup::build_cities@005ab910` (the village at the start cell,
   `Build::init` + `Build::activate(0, 0, 0)`, finished and active; then
   `Setup::small_city_buildings@005aae10` or the large list, then
   `Setup::build_civ_specific@005ab760`); every pre-placed building through
   `Leader::produce_building@006e1400`'s frame-0 arm (the spiral, a
   `rand % 500` per candidate, the woodcutter's forest cube, the 2 × 2
   jitter; `Objects::init_build` + `Build::activate(0, 1, 0)`); then
   `Setup::build_units@005aafc0` — `Setup::get_starting_citizens@005aaf50`'s
   `(n, ordered)`, the scout through `Setup::place_unit@005abca0` (up to 60
   tries, `rand % circle_radius[r]` each), and the citizens at their
   buildings with a `GATHER` each (ORDERS §9.3). The ids are
   `Objects::find_free@0065ad60`'s: scout `0`, citizens `1..`, city
   `2000`, buildings `2001..` in production order. The installer carries
   the citizens' assignment rule (`start_of_game`) and takes every position
   from the dump. **Built through the pre-placed buildings** (item 4047):
   the k-th empire built takes start cell k; `build_cities`' village
   (`snap_center` of the cell's corner, `init_build`, `activate(b, false,
   false)`) and its `compute_all_territory` (`sync_territory`,
   `settle_borders`); `pre_placed_buildings` — the Small Town list and
   `build_civ_specific` through `produce_building`'s frame-0 arm (a forward
   on the setup door). The draws, against run100's draw-site trace of the
   same lobby: the village none; the camp's `Objects::init_build` 244
   (`Build::find_gather_tiles`); each farm its candidates
   (`produce_building+0xc99`), jitter (`+0x1805`) and `Farms::add`; the
   library its jitter and `find_road`'s 189 for the city's roads. The
   computer's six sites and the stream after each are the original's.
   **`build_units` built** (item 4063, `diff::start::build_units`), inside
   each empire after its buildings: `(n, ordered)` and the lobby's and the
   Lakota's adjustments; the scout (`SCOUT` through the graft and
   `current_upgrade`, `leader_unit_type`) through `Sim::place_unit` — the
   anchor the city's own point, sixty tries of `rand % circle_radius[2]`,
   the cell test (`place_unit_admits`: the anchor's region, `flags` 0x30,
   0x100 and 0x8000 clear, `land` not 1 or 2, no object on the cell, the
   centre tile's mask without bit 14 or low bits 3), `init_unit` at the
   cell centre and no `come_out`; then each citizen at its building by
   §9.3's four steps, born on the building's point, stepped off by
   `Sim::come_out_own_point` (`Unit::come_out(u, 0)`'s no-host arm: the
   sweep round its own point from `block_radius` out
   `UNIT_DISEMBARK_DISTANCE`, `FILTER_NOT_ME`, then collision-off), and
   given its `GATHER`. The draws, against run100: the computer's scout 2
   tries, the human's 3, then `Guy::init_real` 7 an empire — every word
   the trace's, through seat 0's last unit (1148's `0x1675cd7d`).
   **The nation arms built** (item 4186; the constants `rules.xml`
   ships, now in `Tuning` and its drift check): past a Nomad start the
   Spaniards' `SPANISH_EXTRA_SCOUT` (1) more scouts, and one more under
   `reveal_map > 1`, and the Dutch two `MERCHANTDUTCH` (`TypeIndex` 62),
   each through `place_unit` from the city's cell after the first scout;
   the Koreans' `KOREAN_CITIZENS[0]` (1) more citizens; the Americans'
   farms, `AMERICANS_STARTING_FARMS` (3) of them in
   `small_city_buildings` whatever bonus 0x13 says, and `n − 3 + farms`
   citizens on a Small Town only past 3 (inert as shipped). The Greek
   scholars' arm is inert while `GREEK_START_SCHOLARS` is 0, as shipped;
   its `go_inside` is refused by name past 0.
   **Not built**: `place_unit`'s Nomad ring and its `Build::train` after
   sixty refusals (each refused by name); the Large Town list; the
   Iroquois' Senate (parked 4047b). **The
   pasture built** (item 4034): a starting farm `Farms::add`'s coin makes
   type 1 is stocked inside its `Build::activate` — five owner-9 animals,
   four draws each — because the builder walks the setup stream
   (`CitiesState::walks_setup`); the installer, which does not, still borrows
   them. *Diff-backed* on 16 Great Lakes lobbies: on 15 the dump's
   `FARMS` `farm_type` and the trace's sync word, on run1515b's (neither
   printed) every building and unit placed after it.
8. **`World::analyze_map@006b58b0`, `World::compute_all_territory@006b5700`,
   `Herd::create_units@007417f0`** (44 draws here), the scripts.
   **`compute_all_territory` built** (item 4063: `sync_territory`,
   `settle_borders`; it changes no owner here — `build_cities`' own pass
   is the last writer of every one). **Its per-cell goods scan, the
   rares' half, built** (item 4093): before the first frame
   `settle_borders` runs `Sim::claim_rares_from_owners`, here and at
   `build_cities`' own call, so a rare on a cell a leader owns and has
   seen goes to `new_rare` and takes the leader's `ever_seen` bit by
   region and cell (`docs/ECONOMY.md`, "Setup's goods scan"). On this
   lobby the setup scan meets good 15 for seat 1 (`AMBER`, cell
   (52, 18)), and frame 0's `GOOD` `ever_seen` is run9's on all 35 printed
   goods. **`analyze_map` built** (item 4147, `docs/MAPGEN.md` §36): its
   ten tallies kept on `World::record`, the region bits 4 and 8 and the
   sea class; the ten equal the function counted over the installed
   start's frame-0 state (`diff::start_walk`), and the start dump prints
   eight of them. **The herds built** (item 4063):
   `herds_from_goods` — one herd per herd good of the composed map
   (`Objects::init_good@00653f30`'s `Herds::init_herd`: FISH →
   `HERDFISH` 0x19b, WHALES → `HERDWHALE` 0x19c, 0x21 → `HERDSHEEP`
   0x198, and the four others), numbered in the goods list's slot order;
   then `Setup::build_game@005ac190`'s own loop over the herd list (the
   body of `Herds::start_herds`, inlined: the function itself is entered
   only by the scenario editor's load) founds each live one through
   `Sim::found_herd`, the crate's view over item 3391's swept
   `herd_create_view` (owner 8, `init_unit` at the cell centre, one
   `Guy::init_real` draw, `find_nearby_spot(…, 0xc0, -1, 0, 0x55555555,
   FILTER_NOT_ME, …)`, the angle draw for a type that is neither fish nor
   whale). On this lobby 13 herds — one of sheep (4 animals, 4 angle
   draws), twelve of fish (3 each) — 40 animals, 44 draws, landing on
   1240's `0x3bd39ae9`.

Then `Setup::reinit_game_units` re-reads every object's `z` from the
terrain, and `begin_game` dumps.

## 3. The start block, as compared

`rondata::diff::start::start_block` reads a `Sim` at frame 0 as records in
the walk's order, the same projection for both sides:

- `LEADER w`: `tribe`, `human`, `bucket` (the six goods), `ages`, `techs`
  (the count owned), `at_war` (the row over the players);
- `WORLD`: `xs`, `ys`, `seed`;
- `CELL x y`: `land`, `flags`, `val`, `region` (whether the cell has one —
  the two sides number regions differently), in row-major order;
- `LEADER w`: `personality`, for a computer leader;
- `BUILD who/o`: `type`, `pos`, `active`, every live building of a player;
- `UNIT who/o`: `type`, `pos`, `health`, `orders` (the list's order
  indices) — the players' units, then gaia's (owners 8 and 9);
- `CELL x y`: `owner` (the territory);
- `GAME`: `rng`, the word frame 0 draws from.

`compare_start` lists every field the two disagree on, in the installed
block's order, then the records only the built side holds. **A field the
installer does not take from the dump compares the crate with itself**:
`bucket`, `ages` and `techs` are `Sim::new`'s and `start_techs`' on both
sides, so their agreement is quiet, not proven. The dump-backed fields are
`tribe`, `human`, `at_war` (the dump's `diplos`), the extent and seed, the
cells (`WORLD=6`), the cities, buildings and units with their orders, the
personality (from the trace's bracket) and the sync word.

**And the reverse: a field the installer cannot take from the dump is
its own stand-in**, so a parting there is no parting. The every-lobby
count (§5) drops and counts three, per lobby: a starting pasture's five
(owner 9, which no dump prints) when no trace lends their seeds; the sync
word when no setup trace for the lobby is on disk; and a computer
leader's personality when neither a trace bracket nor the start dump's
own `PERSONALITY` block (`LEADERS=9`) holds it — where the block is
printed, the built side is held to it. It adds one record the block does
not carry: `FARM who/o farm_type`, each farm against the dump's `FARMS`
list, which is what backs a pasture's existence.

## 4. The harness lobby, compared

`rondata::diff::start::tests::the_harness_lobby_s_start_built_from_its_choices`
builds run9's lobby (`gamelog-run9-world6.txt`, `WORLD=6`, with run11's
setup trace and run3's heights borrowed) from its `GAMEINFO` block alone
and compares it with `build_sim`'s install of the same capture;
`RON_START_DIFF=1` prints the value diff (`=all`, every parted field).
**All 18,264 fields agree** (item 4063; 9,087 at item 4033, 9,089 at
4046, 18,039 at 4047), and the herd list equals the start dump's
`HERDS` (13 herds, every cell, walk centre, type and alive bit, in
order). Its sibling `the_harness_lobby_s_map_is_the_traced_setup_path`
holds the composed map against run11's setup trace at every checkpoint,
the stream at 955, 13457, 1032, 1148, 1169, 1240 and 1244, and both
`build_empire`s — each building and each empire's units — against
run100's draw sites.

- **The war matrix agrees** (item 4046; 4033 had it first parted, ours
  `[false, false]`, theirs `[false, true]`). Its writer is `Leader::init`'s
  diplomacy row (§2 step 6), found by counting the writers of `+0x74` on a
  leader record in every spelling: `diplos[` (no writer by name),
  `field_0x74 +` on `leaders.list` (`Leader::set_diplo@006ec6a0`, whose
  callers are live play, the console, CtW and scenarios; the scenario
  editor; `ScenarioRead::load_leader_chunk`, the scenario path), the bulk zeroing
  (`Leaders::clear@006ed5d0`'s `memset` of each record's first `0x6929`
  bytes, and `Leader::init`'s own fresh arm, `param_2 < 0`), and the
  cursor spelling `puVar23[-0x18]` inside `Leader::init`'s per-leader loop.
  `Game::do_frame`'s `set_diplo(…, 0)` (the end of a rush's peace) runs
  after the dump. On this lobby (`TEAM_STYLE 1`, `RUSH_RULES 0`, both seats
  on team 8) the row is 0 off the diagonal.
- **The map agrees** (item 4047): every cell's `land`, region, `val` and
  `flags` as the map maker and `mark_forest` leave them, 8,056 fields.
  Then the personality (1), the villages' wear on the values and the
  territory (314 `val`, 527 `owner`, 8 flags, 6 `BUILD` fields), and the
  computer's six pre-placed buildings and the human's camp (32 fields).
- **No row parts** (item 4063). At item 4047 the first was `CELL 2 37
  flags`, ours 512 theirs 640: the human's farms, library and market
  stood elsewhere (`BUILD 0/2002 pos` ours `2112 31104` theirs `2688
  32064`) because the computer's `build_units` spent 2 + 7 draws ahead
  of them; after it 11 flags, 5 positions, all 52 units on every field
  and the sync word. `build_units` and the herds closed all 225.

## 5. The counter

`Start: <built> of <N> lobbies` (`rondata::diff::floors::start_line`):
`START_BUILT` lobbies whose whole start block the builder reproduces, of
`START_LOBBIES` lobbies on disk with a `[Start Game]` dump. **0 of 161**
at item 4033. N is `tools/start_lobbies.py`'s scan of the kept captures:
a capture counts when its `GAME` block carries start records ahead of its
first frame (a `WORLD`, `UNITDATA`, `BUILDDATA` or `FULL DUMP`); its lobby
is its `GAMEINFO` block's text, `same_lobby`'s key — 645 captures, 161
lobbies. **1 of 161** at item 4063: the harness lobby.

**The harness is every lobby** (item 4034,
`diff::start::tests::every_lobby_s_start_built_from_its_choices`): for
each lobby `lobby::lobbies_on_disk` finds, its richest capture (cells,
then a setup trace, then objects) is installed by `build_sim`, the
lobby's other captures lent by `borrow_from_siblings`, against
`from_lobby` of its `GAMEINFO`, by §3's rules. `START_BUILT` is the count
whose list parts nowhere, and `START_CENSUS` pins every other lobby's
first parting by record and field; `RON_START_LOBBY=n,…` with
`RON_START_DIFF=1` (or `=all`) prints one lobby's value diff. **35 of
161** at item 4034: 14 Great Lakes lobbies already whole but never
compared, 16 a starting pasture released, 4 whose one parting was a
personality no dump holds, and the harness lobby. Of the other 126, 117
are the fifteen styles with no composed chain (East Indies 53, Great
Sahara 47), parting at their first cell; 8 are the Great Lakes lobbies
with a Spanish (3), American (2), Dutch, Korean or Greek seat; one is
4033a's scratch capture, whose dump prints neither cells nor `diplos`.
**114 of 161** at item 4186: the chain composed for East Indies (42 of
its 53 lobbies) and Great Sahara (37 of 47) by the same walk
(`rondata::mapgen::lakes::composed_map`). Of the 47 left, 29 are the
nation arms at a `CELL flags` (Great Lakes 8, East Indies 11, Great
Sahara 10: a Spanish, American, Dutch, Korean or Greek seat, or a lobby
with one among several), 17 the fourteen styles still without a chain at
their first cell, and one 4033a's. **143 of 161** the same item: the
nation arms (§2 step 7) release all 29. **158 of 161** at item 4196:

- Southwest Mesa composed (3), then the eleven chained styles (one lobby
  each, 7 on the first walk).
- Nile Delta, Sea Power and New World once the style's territory limits
  are read, and East Meets West once its permutation is the identity
  (§2 steps 4 and 5).
- 4033a's thin lobby.

A dump that prints no cells, no `diplos`, or a building without its
`orig_type` leaves the installer on its own defaults there. Those rows
are stand-ins. The rows such a dump does print stay compared: on the
thin lobby, the buildings' places and the units. A one-unit nudge to
either parts it. The three left are Warring States (run1122) and British
Isles (run1127, run316), at their first cell, with no mask port.

## 6. Confidence, and what is not established

- **Diff-backed** (the comparison above): the extent on Small, the
  nations and human bits from `PLAYER`, `world->seed` after one nation
  draw and the re-seed, the war matrix; the Great Lakes map composed from
  the lobby (19 checkpoints of run11's trace, and every cell of the start
  block), the start permutation and the personality (the trace's words
  at 955 and 13457), the villages and the territory, both empires'
  pre-placed buildings, `build_units` on a Small Town (the scout's
  `place_unit`, the citizens' assignment, `Unit::come_out(u, 0)`'s
  no-host arm, the `GATHER`s) and the herds (the start block, run100's
  draw sites, run11's words at 1148, 1169, 1240 and 1244, and the start
  dump's `HERDS`). The diplomacy row's formula was
  also scored against **every start dump on disk** (item 4046, a scratch
  scan of each log's `GAMEINFO` against its `LEADERDATA diplos[scan]`):
  633 logs, 1,334 ordered pairs, every one predicted — 1,322 at war under
  `TEAM_STYLE 1`, 12 at peace under `RUSH_RULES 10`.
- **Read once, from the decompile**: the extent's style-16 and half-size
  arms; that `GAMEINFO`'s `PLAYER tribe` is the nation *after* the random
  roll — which is what the builder takes; the roll's one draw is spent
  on a stream the re-seed discards, so whether a nation was chosen or
  rolled changes nothing at frame 0.
- **Read only**: the diplomacy row's team arm (2), its unlocked styles
  (0, 8, 11), its conquest and `semaphore[1]` arms, and the style-7
  refusal — no capture on disk has a team game or an unlocked style.
  The style-7 refusal is equivalent at frame 0 (a seat on team 8 already
  fails the `0..4` test), so no lobby can tell it apart.
- **Diff-backed on 35 lobbies** (item 4034, §5): Great Lakes at 34
  lobbies besides the harness's, among them every starting age 0 to 7,
  difficulties 0 to 5, `STARTING_RESOURCES` 7 (19 lobbies) and AI seats
  of seventeen nations; the starting pasture on 16 (its `FARMS`
  `farm_type` and the trace's sync word; the five animals' species and
  places are in no dump, 4034b).
- **Diff-backed on 79 more** (item 4186): East Indies and Great Sahara
  composed from `setup.cpp` 716 on one stream, each lobby's every cell,
  building, unit and the setup trace's sync word. Great Sahara's
  `init_map_data` draws on its one `TILECHANCE` (`GS_Desert` 100): the
  gate is the chances' sum, `T − 1 < 1`, not the entry count; with the
  count's gate restored, lobby 3 (run1000) parts at `CELL 35 0 flags`,
  ours 512 theirs 0, 12,045 of 18,136 agreeing.
- **Diff-backed on 29 more** (item 4186): the Spanish, Dutch, Korean and
  American arms on Great Lakes, East Indies and Great Sahara, every
  building and unit and the sync word; each arm's mutation parts its
  lobbies (the item's journal). The Greek lobbies (6) are backed only in
  that the arm is inert at 0 scholars; the `reveal_map > 1` scout is read
  only — every Spanish lobby on disk is `REVEAL_MAP` 1.
- **Diff-backed on 14 more** (item 4196): Southwest Mesa (3) and the
  eleven chained styles, one lobby each. On each lobby every cell,
  building and unit agrees, and so does the setup trace's sync word.
  The territory limits are backed on Nile Delta, Sea Power and New World,
  which part on cell owners without them. East Meets West's permutation
  is backed on run1524, whose seats swap with it randomised. A Mesa
  map's mining lists equal those `build_sim` lays from the trace's 7805
  word, but that is the crate against itself, held to the trace's word:
  no dump prints `cliff_mining_data`.
- **Thin, on 1** (4034a, item 4196): 4033a's scratch lobby, whose dump
  prints no cells, no `diplos` and no building types. Its 13 buildings'
  places, its units, the extent and the seed are backed; its cells, war
  row and three buildings' types (0/2005, 0/2006, 1/2005, which
  `start_of_game`'s object-number list does not name) are stand-ins
  (4,330 rows).
- **Not established**: the team assignment and the handicaps; a
  composed chain for Warring States or British Isles (no mask port);
  East Meets West in a team game and Colonial Powers' outsiders (both
  refused); the default tileset
  draw's unavailable-name arm (a second
  draw, parked 3819b); a team game's consecutive teammates; the Large
  Town list; `build_units` on a Nomad or City-only start (`place_unit`'s
  rings 8 and 24, its `Build::train` arm), or with `starting_resources`
  8; the Spaniards' `reveal_map > 1` scout, the Greek scholars past 0 and
  the Americans' citizens past 3 farms (read only, item 4186); a
  `place_unit` cell refused by the object test alone (the crate threads
  no building, so a building's own cell stands in for its chain);
  a herd of horse, bear, bison or peacock, a whale (one
  animal, the unset angle), and a map whose goods list reuses a slot
  (the herds are numbered in slot order; the original numbers them in
  `init_good` order — they agree unless a slot was reused). ORDERS §9.2's
  placer prose puts the draw on the wrong arm (parked 4047a), and §9.3's
  "steps off the footprint" is not what the no-host arm does (parked
  4063a); the port is right in both.
- **Coverage**: every lobby compared, 161 of 161 (item 4034); 35 whole,
  114 and then 143 at item 4186, 158 at item 4196.
  Two of the 35 have no setup trace on disk (run1515b's and run1541b's
  lobbies), so their sync word is not backed; four have neither a
  bracket nor a `PERSONALITY` block for the computer leader (run3's,
  run5's, run597's, run1541b's), so on three the trace's sync word holds
  the roll's twenty draws and on run1541b's nothing does.
  Past the block — every take the installer makes, and the lobby's game
  walked from the built start — is §7 (item 4057).

## 7. Frame zero wider than the block, and the game run from it

Item 4057. §3's block reads 18,264 fields; `diff::setup::build_sim`
writes more than that, and a walk reads all of it. This section is every
take the installer makes, whether the built start (`from_lobby`, wrapped
as a walk's `Built` by `built_from_lobby`) carries it, and the lobby's
long capture walked from the built start with no dump installed.

**The instrument.** `diff::start_walk` (test-only) compares the two
`Sim`s whole: each `{:#?}` print flattened to `(path, value)` leaves,
buildings and units keyed by `(who, o)` rather than store position
(`RON_START_WHOLE=1` prints it take by take). The first comparison, on
item 4063's tip, parted on 25,984 leaf paths; most were the install's
art tables (`roster.art.*`) that only the installer copied, then the
fog, goods, goody boxes, mountains, territory limits and region flags.
Two things it parts on are not starts: the object stores are in a
different order (the installer makes them in the dump's order, the walk
in the original's creation order — seat 1's empire first), so every
index-valued field (`cities[*]`, `farm_order`, a citizen's `GATHER`
building, `territory_won`'s city index) differs while the objects agree;
and `trace.phase_marks`, the installer's setup bookkeeping.

## 7.1 The takes

Each row is one take of `build_sim`, in its order: what the installer
reads, what the built start does, and how the row is known. **Built**:
the built start computes it and a comparison holds it equal.
**Defaulted**: the built start leaves a default where the installer
takes a value. **Dump-only**: no build exists from the lobby's choices
without a mechanic nobody has written, or the installer's own value is
an assumption the dump cannot settle.

| take | installer | built start | status, evidence |
|---|---|---|---|
| extent | `WORLD xs ys` | `world_extent` | built; block |
| cells: land, region, region2, flags, val, goods, blocked, solid, bad, owners | `WORLD` cells | the map chain and the walk (§2 steps 4, 7, 8) | built; block (land, flags, val, region, owner), whole comparison (the rest) |
| `land_sub` | `WORLD` cells | the chain's, and `Terrain::object_placed` under each started building (`docs/MAPGEN.md` §35) | built; wide (every cell agrees since item 4136) |
| tile masks | `WORLD` tiles | the chain's and the walk's, with `Wall::mark_behind_tiles` (`docs/MAPGEN.md` §35) | built; wide (every tile agrees since item 4136) |
| corner heights | run3's `DUMP_ALL` heights | the chain's singles | built; 1,422 of 58,081 corners differ from the dump's six-decimal print by one unit in the single's last place; not projected, since the print cannot settle it |
| `seen2` | `WORLD` fog | `World::init`'s zeroed grid, every reveal of the walk, and `Object::update_seen_ally` (§7.2) | built; wide (14,400 agree) |
| `seen` | a copy of `seen2` | the walk's own | dump-only: the copy is the installer's assumption; `update_seen_ally` writes `seen2` alone, so 273 fog cells differ; not projected |
| `sea_map` | `WORLD` scalar | `sea_map_after_make` | built; whole comparison |
| territory limits | `WORLD` scalars | the rules' triple, `Map::load_map_data`'s copy | built, red first (ours `None`); wide |
| region flags | `REGIONS` | 0x4 on the land region only | defaulted: §7.3 |
| site values | `WORLD` cells, written back | the villages' own wear | built; block |
| lobby, `Setup` sync, `game_seed` | `GAMEINFO`, `WORLD seed` | the same | built; block |
| animation lengths (`art.lengths`) | `DUMP_ALL` guys | none | dump-only, and not read where the install speaks: `art.piece_lengths` wins (`docs/ANIM.md` §3.2); the walk agrees without it |
| the install's art, craft, release, track and pivot tables | the install | the same, `install_art` (shared) before the first unit | built; whole comparison |
| placed mountains | `MOUNTAINS` | the grid's `mountains`, laid from their templates | built; wide |
| cliffs | the Mesa generator | the composed chain's `Cliffs::init` lists on a Mesa map (item 4196) | not this lobby's |
| nations, human bits, `diplos` | `LEADER` | `PLAYER` rows, `init_diplos` | built; block |
| opening techs, scripts, personality | `start_techs`, the trace's brackets | the walk | built; block |
| goods | `GOOD` | the grid's goods list, each at `GoodTypeData::snap_center` of its cell (oil at the centre) | built, red first (36 absent, then 19 off by half a tile); wide |
| cities, buildings, units, orders | `CITY`, `BUILDDATA`, `UNIT` | the walk | built; block |
| guy clocks | `DUMP_ALL` guys | `Guy::init_real` | built; the walk (frame 40 parted while the pieces were missing, below) |
| a player unit's pieces | `GUY gpiece` | `Sim::unit_gpiece` off the install | built once `install_art` ran before the first unit; the walk |
| gaia's pieces | `GUY gpiece` | none | dump-only: `first_bird_piece` is a runtime pointer no file states (`Sim::piece_of`); the walk agrees without it |
| gaia's facing | a default, `0x55555555` | `Herd::create_units`' angle draw | built; the installer's is the default (40 differ) |
| a gaia animal's herd | the first herd of its type | the herd that founded it | built; the installer's is an approximation (33 differ) |
| the scout's `squad_size` | the dump's figure count, 2 | 1 | not established which the walk reads; the walk agrees |
| farms: type, heights, list order | `FARMS` | `Farms::add`, creation order | built; whole comparison, keyed |
| pasture animals | the trace | none here | not this lobby's (no pasture on it; §2 step 7) |
| herds | `HERDS` | `herds_from_goods` | built; §4 |
| goody boxes | `ITEM` | the cells' `down` −3, by `down_who` | built, red first (22 absent); wide |
| building `ever_seen` | the dump's | `update_seen_ally`'s mask | built, red first (2 part); wide |
| `new_rares` | a replay off the dump's fog | the walk's own reveals and setup's goods scan (item 4093) | dump-only: ~~the installer's replay gives seat 1 rare 15 and the walk none~~ both give seat 1 `[15]` (item 4093); the original's frame-0 list is in no dump, and its `ever_seen` bit is: run9's `GOOD 15` 2, ours 2 |
| oil claims | a replay off the owners | the walk's territory | built; whole comparison |
| the sync word | the trace's last word | the walk | built; block |
| a building's `z` | the setup block | its own terraform | not read by the simulation (the renderer's) |
| a cell's `down`, `down_who` | the dump's chain heads | the map's (−1, −2, −3) | dump-only as cell bytes: the simulation threads its chains in `terrain.chain_heads`; 28 cells differ; not established that anything reads the cell's copy |
| corrections: frame words, clocks, re-seats | the siblings' `DUMP_ALL` | none | not taken by either walk of §7.4 |

**The counts**: 37 takes. **Built 25**, equal on every comparison since
item 4136 (the two that parted, `land_sub` and tile bit 0x4, are the
started buildings' writes, §7.3). **Defaulted 1** — region flags. **Dump-only 7** — `seen`, the
animation lengths, gaia's pieces, `new_rares`, the cells' chain heads,
building `z`, the corrections. Two not established (the heights' last
place, the scout's `squad_size`) and two not this lobby's (cliffs,
pasture).

## 7.2 What the walk needed that the block did not show

- **The start's fog.** The walk parted on frame 0, draw 26 of 40
  (`Unit::think_scout+0x64c` against `+0x436`): the built start had no
  `seen2` at all, which `seen2` reads as "always seen". With
  `World::init`'s zeroed grid every reveal the walk already makes landed,
  and 273 fog cells were still dark — every one inside radius 10 of a
  village. `Setup::build_cities` ends its village with
  `Object::update_seen_ally`: radius the building's line of sight, or,
  for a city's own building (`flags & 0x20`), `LeaderData::get_radius` of
  its type capped at 64 and halved into fog cells (20 → 10 here); every
  point of `circle_radius[r]` round the village's fog cell takes the
  reached leaders' bits in `seen2` alone, and the building's `ever_seen`
  takes them too. Reached: the owner, every leader allied with it both
  ways, and every human when the owner is human (the `semaphore[0] & 4`
  gate is no lobby field; the `game_rules == 8` arm is read and not
  carried). Ported as `Sim::update_seen_ally_build`
  (`crates/sim/src/vision/seen_ally.rs`); the per-cell `WData.was_seen`
  byte it also sets is not kept. After it every fog cell agrees.
- **The install's takes before the first unit.** The walk then parted on
  frame 40, draw 0: theirs `Guy::set_anim+0x97a < Guy::inc_time+0x271`,
  ours a farm's. Every guy of the built start had `gpiece −1`:
  `install_art` ran after `from_lobby`, so `Sim::piece_of` found no piece
  table when each figure was born. Run inside `from_lobby` before the
  map's objects, every player figure takes its piece and the walk runs
  to the trace's end.
- **The region map.** A walk's city comparison reads `reg` through
  `Built::region_map`; `built_from_lobby` first left it empty and every
  record of run12 and run13 parted on `CITY 0/2000 reg`, ours −1 theirs
  1. It rides `FromLobby::region_map` now.

## 7.3 The wide projection

`diff::start::start_wide` projects the takes §3's block does not read:
`FOG fx fy` (`seen2`), `GOOD i` (type, position, live bit), `ITEM i`
(position, live bit, `ever_seen`), `MOUNTAIN i` (origin, solid cells,
tiles), each building's `ever_seen`, `WORLD territory_limits`, `REGION n
flags` (by the original's number), `TILE tx ty mask` and `CELL x y
land_sub` — 75,835 fields on the harness lobby.
`start_walk::the_harness_lobby_s_start_is_the_installed_one_wide` holds
every one equal but one, pinned by its count (three until item 4136), and
§3's block and `START_BUILT` are untouched:

- **`REGION 0 flags`**: ours 0x4, theirs 0xa4. The land region's 0x20
  and 0x80 have no writer found (East Indies shows 0xa0 on every land
  region, `docs/TRANSPORT.md`); `Map::region_flags`, the style's word,
  is 0 on every corpus style (`docs/MAPGEN.md`). Building it needs that
  writer.
- ~~**`TILE mask`, 131 tiles**: bit 0x4 set in the dump and in neither
  side's own writes; its writer is not identified.~~ `Wall::start`'s
  `mark_behind_tiles`, `tile::BEHIND` (item 4136, `docs/MAPGEN.md` §35).
- ~~**`CELL land_sub`, 8 cells**, four beside each village (ours 3 or 1,
  theirs 2); not identified.~~ `Wall::start`'s `Terrain::object_placed`
  at the tileset's `weatherbase` (item 4136, `docs/MAPGEN.md` §35).

The walk reads the one left and still runs to its end (§7.4), so it moves
no draw on this game.

## 7.4 The lobby's own walk from the built start

`start_walk::the_lobby_walked_from_the_built_start` walks run53's game
(the harness lobby, 24,000 frames, no input) from `built_from_lobby`'s
`Built`: no start dump installed and no correction — no frame's word, no
clock, no re-seat. Every frame's draws are compared with run53's trace
(`rontrace-run53.log`, the word as `run53_s_24000_frames_…` counts it:
the first frame whose draw count parts), and every record of the game's
captures whole (`compare`, the harness's own): run53's two, run12's
`DUMP_ALL` head, the windows run13, run125, run130, run135, run136,
run163, run174, run226, run243, run294 and run80 — 2,770 records. The
control is the same walk from `build_sim`'s install with its corrections
off. (run3, the heights' sibling, is another game of this lobby: its
frame 1 parts from both starts, and it is left out.)

- **The word: 24,000 of 24,000 on the built start**, the installed
  start's own (`LONG_WORD_GREAT_LAKES`, pinned `START_WALKED`). Both
  sides' draws first part on frame 24,000, the trace's last and partial
  frame, at the same draw.
- **The records: 10 of 2,770 part, the same ten from both starts** —
  each window's opening block (its three cities unlinked, and on run136,
  run163 and run226 one citizen a few units off), the residue a window's
  first block carries. The test holds the two walks' per-capture counts
  and first parted record equal.
- The installed start without its corrections also runs to 24,000 — so
  on this game the corrections change nothing, as `audit.rs` holds for
  the word and the clocks.

**Red, and the perturbation.** The walk went red twice before it stood:
frame 0 (the fog) and frame 40 (the pieces), each with its value diff in
§7.2. With the village's `update_seen_ally` removed (one line, reverted)
the wide test parts on 273 `FOG` and 2 `BUILD ever_seen` fields, and the
walk's word is 0, first parting `Unit::think_scout+0x64c` against
`+0x436` at draw 26.

## 7.5 Confidence, and what is not established

- **Diff-backed**: every built row of §7.1 by the wide projection or the
  whole comparison; the fog, the goods' positions, the goody boxes, the
  mountains, the territory limits and the buildings' `ever_seen` by
  `start_wide`; the whole start by the walk — 24,000 frames of draws and
  2,770 records alike from both starts.
- **Read once, from the decompile**: `update_seen_ally`'s reach (the
  allied and human arms; no capture has an ally or two humans), its
  non-city radius (the building's line of sight unscaled; only villages
  call it in a start), and the `semaphore[0] & 4` and `game_rules == 8`
  arms; that a style XML's territory-limit override is read by
  `Map::init_map_data` (Great Lakes names none, and the built start does
  not read one).
- **Not established**: §7.3's three; which of `squad_size`'s two values
  the walk reads (it agrees either way); whether anything reads a cell's
  own `down` copy; the original's frame-0 `new_rares` (its count's
  first print is frame 91's `known_rares 1` for seat 1, which ours
  holds; item 4093); the gaia pieces'
  source; the heights below the dump's print. One lobby: any other needs
  its own composed map (`docs/START.md` §6) and its own captures.
- **The lobby page** (item 4184): the choices this walk reads are set by
  a person on the client's page, and the harness lobby's start built from
  the page is this one, whole (`docs/LOBBY.md` §5).
