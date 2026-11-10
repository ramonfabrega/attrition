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
   `make_continents` overwrites). Other styles: **not built**.
5. **The start permutation**: `Game::start_list[k]`, a random permutation
   when the map randomises starts (`rand & 7`, 8 draws here), else
   identity; teammates placed consecutively in a team game unless `world
   +0x30 == 0x16` (ORDERS §9.2, S5). The installer recovers the visiting
   order from the trace (`leader_visit_order`). `randomize_starts` is
   `Map::Map@006a06e0`'s 1, which only East Meets West's
   `make_continents` writes again. **Built** (`start_permutation`, item
   4047), randomised; *diff-backed*: the eight draws from 894's word are
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
   **Not built**: `build_units` — `place_unit` (2 draws for the computer's
   scout, 3 for the human's), the citizens born at their buildings
   (`Guy::init_real`, 7 draws an empire), `Unit::come_out(u, 0)` off a
   footprint (no port takes a unit that is not inside), and the `GATHER`;
   the Large Town list; the Americans' farms and the Iroquois' Senate
   (parked 4047b); the pasture (none on Great Lakes).
8. **`World::analyze_map@006b58b0`, `World::compute_all_territory@006b5700`,
   `Herd::create_units@007417f0`** (44 draws here), the scripts.
   `compute_all_territory` and `analyze_map`'s region arm are ported for
   the live game; the herds are not built from a lobby. **Not built**
   (the territory agrees already: `build_cities`' own pass is the last
   writer of every owner on this lobby).

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

## 4. The harness lobby, compared

`rondata::diff::start::tests::the_harness_lobby_s_start_built_from_its_choices`
builds run9's lobby (`gamelog-run9-world6.txt`, `WORLD=6`, with run11's
setup trace and run3's heights borrowed) from its `GAMEINFO` block alone
and compares it with `build_sim`'s install of the same capture;
`RON_START_DIFF=1` prints the value diff (`=all`, every parted field).
**18,039 of 18,264 fields agree** (9,087 at item 4033, 9,089 at 4046).
Its sibling `the_harness_lobby_s_map_is_the_traced_setup_path` holds the
composed map against run11's setup trace at every checkpoint, the stream
at 955, 13457 and 1032, and the computer's `build_empire` against
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
- **The first parted row is now `CELL 2 37 flags`**, ours 512 theirs 640:
  0x80 round the human's farms. The human's farms, library and market
  stand elsewhere (`BUILD 0/2002 pos` ours `2112 31104` theirs `2688
  32064`) because the computer's `build_units` spends 2 + 7 draws ahead
  of them in the original and none here.
- After it: 11 footprint flags and 5 positions round the human's
  buildings, all 52 units on every field (the players' 12, gaia's 40) and
  the sync word — each downstream of `build_units` and `Herd::create_units`.

## 5. The counter

`Start: <built> of <N> lobbies` (`rondata::diff::floors::start_line`):
`START_BUILT` lobbies whose whole start block the builder reproduces, of
`START_LOBBIES` lobbies on disk with a `[Start Game]` dump. **0 of 161**
at item 4033. N is `tools/start_lobbies.py`'s scan of the kept captures:
a capture counts when its `GAME` block carries start records ahead of its
first frame (a `WORLD`, `UNITDATA`, `BUILDDATA` or `FULL DUMP`); its lobby
is its `GAMEINFO` block's text, `same_lobby`'s key — 645 captures, 161
lobbies. The harness lobby's test asserts `START_BUILT` against its own
comparison, so the counter cannot read 1 while that lobby parts.

## 6. Confidence, and what is not established

- **Diff-backed** (the comparison above): the extent on Small, the
  nations and human bits from `PLAYER`, `world->seed` after one nation
  draw and the re-seed, the war matrix; the Great Lakes map composed from
  the lobby (19 checkpoints of run11's trace, and every cell of the start
  block), the start permutation and the personality (the trace's words
  at 955 and 13457), the villages and the territory, the computer's
  pre-placed buildings (the start block, and run100's draw sites). The diplomacy row's formula was
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
- **Not established**: the team assignment and the handicaps; a
  composed chain for any style but Great Lakes, and Great Lakes at any
  other lobby; the default tileset draw's unavailable-name arm (a second
  draw, parked 3819b); a team game's consecutive teammates; `build_units`
  and every draw of it; the Large Town list; `Herd::create_units`.
  ORDERS §9.3 holds the reading of `place_unit`; it is not ported, and
  no port steps a unit that is not inside off a footprint
  (`Unit::come_out(u, 0)` at birth). ORDERS §9.2's placer prose puts the
  draw on the wrong arm (parked 4047a); the port is right.
- **Coverage**: one lobby compared of 161. The others need their own
  installed side (`build_sim` on each start dump) and are item 4034's.
