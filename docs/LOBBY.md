# The lobby, a plain page

Item 4184, the lobby track's first (DECISIONS 128 §6). The client opens on a
page that sets the start's choices and starts the game from them through
the `--lobby` door (`docs/START.md` §7). It is **not the original's
interface**: one row a choice, in the debug overlay's font. Porting the
original's lobby is on the cut list and Ramon's to reopen.

Code: `rondata::lobby` (the model: the choices, the events, the block, the
planner, the scan of the disk), `rondata::free::FreeRunner::from_choices`
(the door), `client::lobby` (the page: rows, keys, lines) and the binary's
`--lobby` window, `--page-still`.

## 1. The block

What a lobby is, for this item, is the block the original prints of it:
`GameInfo::log_data@005d6040` under `BEGIN GAME INFO` / `BEGIN GAMEINFO`,
which the built start reads (`diff::from_lobby`). Its order, from the
function and the type stream:

- `(Version: <build>)`, then `(int) version <n>`: the executable's.
- The thirty settings, each one `uchar` of `GameInfo +0x18`
  (`GameInfo_u_24_s_0`), named by `gameinfo_strings`
  (its dynamic initializer, `@00405400`): `TEAM_STYLE`
  through `MODS` (`lobby::SETTINGS`).
- `(int)seed`, `checksum_deep`, `checksum_window_size`,
  `checksum_failure_threshold`, `flags` (`GameInfo +0x4` to `+0x14`).
- A `PLAYER` row for each of `Player[8]` whose `flags & 1`
  (`Player::log_data@006ee720`): `flags`, `tribe`, `who`, `team`,
  `handicap`, `play`, `pauses`, `diff`, `pop_cap_frame`, the ten `synced_*`
  counters, and the seat's name as a bare last line.

`Choices::block` writes exactly this, in this order, at the dump's
indentation.

## 2. The page's events

`lobby::Event` is one thing a person does on the page: a setting's entry
(`Set`), the seed typed (`Seed`), a flag on or off (`Flag`), a seat added
(the computer, the first nation, its first leader) or the last removed, a
seat's kind, nation, leader (by its place in the nation's file), typed
name, team and handicap. `Choices::apply` is the only writer of the page's
state, and refuses what the page cannot hold (an entry past its
category's count, a ninth seat, a name the block cannot print back, a
setting the page does not offer) and leaves the choices as they were.

The page opens on **the harness lobby** (`Choices::harness`, the events
`harness_events` on a blank page): run53's lobby, seed 12345, Great Lakes,
Small, the person's Nubians named `Player 1` against the computer's
British, William the Conqueror (the fifth leader of `tribes/british.xml`).

## 3. The page

`client::lobby::Page`: the rows (every offered setting, the seed, one row a
flag, five rows a seat, then add, remove and start), the cursor, and the
text typed into a seed or name row since the cursor came to it. Up and
Down move the cursor, Left and Right step a row's value (a name row steps
the nation's leaders), typing edits a seed or a name, Enter adds or
removes a seat or starts. Each key makes at most one event, applied
through `Choices::apply`; a half-typed name the block cannot print yet (an
empty name, a trailing space) waits for the next key.

Start opens the game: `World::open_game(Game::Page(choices))`, which is
`FreeRunner::from_choices`. That prints the choices' block, parses it as a
dump's is, and runs the same `from_lobby` and `built_from_lobby` the minted
script's walk runs on run53. The window then runs to `--frame`. With
`--still`, `--headless` or `--silhouette` there is no page and no person, and
`--lobby` builds run53's own block as before. The `clock` test's session
is unchanged.

## 4. The settings' categories

Each offered setting's byte is read as an index into one `rules.xml`
category (`lobby::CATEGORY`). The page shows the category's `title` and its
entries' `name`s, read from the install at run time (`Roster::read`). The
nations are `rules.xml`'s `TRIBES` in order (the index `PLAYER tribe` is),
named by each nation's own file, and so are their `<LEADER>`s.

**How the pairing is known.** It is by name: `MAP_STYLE` and `mapstyles`,
`STARTING_RESOURCES2` and `defenderresources` ("Defender Bonus:"). It is
not read from the original's lobby code. The index reading is confirmed
where a consumer reads it: `mapsizes` by `world_extent`, `mapstyles` by
Great Lakes being 14, `startingresources` by `Loaded::starting_resources`.
It is corroborated where a capture's name says what was chosen: the
`…-diff3-…` captures print `DIFFICULTY 3`, the `…-toughest-…` ones
`DIFFICULTY 5` ("Toughest", the sixth of `difficulties`). It is assumed
elsewhere. `gameinfo_flags` is read a bit an entry, by position; the
shipped file comments one entry out (`docs/FORMATS.md`), and which bit
the original gives each entry is not established.

`PLAYERS` is derived: it is the seats less one, which is what every lobby
on disk holds (1 on the 157 two-seat lobbies, 2 on the four three-seat
ones; `numberofplayers` is "Other Nations:").

## 5. The counter

`Lobby: the page sets N of 161 lobbies · starts M of 161, walked 600
frames`: **`lobby::LOBBY_PAGE`, 161 of `diff::START_LOBBIES` 161** at item
4184, and **`lobby::LOBBY_STARTS`, 161 of 161** at item 4215 (from no
clause; the line's second clause was "the harness lobby's start built from
it"). The parser is `diff::floors::lobby_line` with
`the_handoff_s_lobby_line_is_the_pinned_counter`, a driver's line.

The two sides, and what builds each:

- **Theirs**: `lobby::lobbies_on_disk`, a scan of every `gamelog-*.txt`
  under the logs directory read to its first frame. It ports
  `tools/start_lobbies.py`: a start dump is one whose `GAME` holds a
  `WORLD`, `UNITDATA`, `BUILDDATA` or `FULL DUMP` ahead of the first frame,
  and lobbies are grouped by the `GAMEINFO` block's text. Each lobby keeps
  its block's raw lines (`OnDisk::lines`) and its parsed fields
  (`Lobby::parse` on the file's own text, `gamelog::Log`). The scan's 161
  is asserted equal to `START_LOBBIES`, the Python scan's count.
- **Ours**: a blank page driven by key presses. `lobby::events_for` reads
  the lobby the way a person reads a lobby they mean to set; for each
  event, `Page::keys_for` gives the keys a person presses to make it, and
  each key goes through `Page::key`. Only the events the page emits are
  kept. A replay of those events alone is asserted to give the page's
  state. The page's block is then printed by `Choices::block`.
- **The comparison**, twice: the page's block lines (`Choices::lines`, no
  parser) against the dump's raw lines, and the page's block parsed
  against the dump's parsed fields (`Lobby::partings`, every `GAMEINFO`
  field and every `PLAYER` row field in order). That is `same_lobby`'s
  key, and stricter: `same_lobby` compares `GAMEINFO`'s own fields alone.
  The text comparison shares no helper with the page's state, and the
  field comparison shares only the dump's own parser. On run53 the parser
  reads 77 fields of the block's 79 lines (the two `BEGIN PLAYER` lines are
  the rows' heads), so no line goes unparsed. The root `GAME INFO` block's
  own fields (`RUN COUNT`, `Player`, `WHO`, the profile's) are outside
  `GAMEINFO` and not compared; the built start does not read them.

Both tests count 161: `rondata::lobby::tests::the_page_sets_every_lobby_on_disk`
(the planner's events applied) and
`client::lobby::tests::the_page_s_keys_set_every_lobby_on_disk` (the page's
own keys). The harness lobby is first and its events are the page's
opening.

**The start.** `lobby::tests::the_page_s_harness_lobby_builds_the_built_start`
builds the page's opening through `from_choices` and run53 through
`from_lobby`, and compares the two `Sim`s whole by their `{:?}` print, at
frame 0 and again after 40 frames. That is the start `START_WALKED` walks
24,000 of 24,000 frames from. `client::lobby::tests::start_on_the_page_opens_the_minted_lobby_s_game`
does the same through the client's `World`, scene for scene.

**Every lobby's start** (item 4215). `lobby::tests::the_page_starts_every_lobby_on_disk`
sets each of the 161 lobbies on a blank page (`events_for`), starts it
through `from_choices`, and builds the lobby's built start through
`from_lobby` on its first capture: the `Sim`s whole by their `{:?}` print
on frame 0, then both walked `LOBBY_WALK` 600 frames and compared again.
`LOBBY_STARTS` counts the lobbies equal at both. Each lobby also checks the
ground the client draws, `FreeRunner::ground` — the tileset the composed
map drew (`LakesMap::tileset`) and `World.seed` — against the capture: its
`WORLD seed`, and the tileset `mapgen::tileset_before` steps back from its
`map.cpp` 7805 word, or the style's one `TILECHANCE`: the seed is
dump-backed on 160 lobbies (one start prints no `WORLD`), the tileset on 142
(the rest print no 7805 word and draw among several). The two `Sim`s are built from one parser's fields,
so the frame-0 comparison holds what the block's 161 already hold, carried
through `from_lobby`; what it adds is the 600 frames, and that nothing of
the capture past its `GAMEINFO` enters the start.

**East Indies through the client.**
`client::lobby::tests::east_indies_from_the_page_opens_its_own_start`: the
first East Indies lobby (`MAP_STYLE 18`) set by the page's own keys
(`Page::of_capture`), opened as Start opens it (`Game::Page`), against
`from_lobby` on its capture: the scene on frame 0 and after 600 frames, and
the ground art's `(tileset, World.seed)` the built start's and the
capture's `WORLD seed`. Before item 4215 the client drew every game's ground
with the Great Lakes pair's tileset and seed (`lobby_tileset` on
`START_DUMP`); a built start now draws its own (`World::open_game`), and
the pair keeps the dump's. `client --page-of <capture> --headless --play
--frames 600` draws the page's game off-screen.

**Red, three ways.** A bent seed (12346) builds another `Sim`. On the
counter: the page's nation mapping off by one gives 0 of 161, the harness
lobby first parting on `PLAYER 0 tribe`, ours 5 theirs 4; the difficulty
control dropped gives 30 of 161 (the lobbies at `DIFFICULTY 0`), run1000
first parting on `DIFFICULTY`, ours 0 theirs 3. Each was one line,
reverted. `a_bent_event_parts_the_lobby` keeps a bent nation red as a test.
The starts (item 4215): `a_bent_page_parts_the_east_indies_start` keeps two
bends red, on the first East Indies lobby: the seed one on, and the
difficulty one on, each part on frame 0 and still after 600 frames.

## 6. Confidence, and what is not established

**Diff-backed**: that the page's events reproduce all 161 lobbies' blocks,
line for line and field for field; that the harness lobby's start built
from the page is run53's built start, whole, on frames 0 and 40; that every
lobby's start built from the page is its built start, whole, on frames 0
and 600 (item 4215; the built start itself is scored against each lobby's
dump by `Start:`, `docs/START.md` §5); and that the ground the client
draws for a built start has the capture's `WORLD seed`, and its tileset
where the capture prints the 7805 word or the style has one tileset.

**What the page cannot set** (`lobby::NOT_SET`). The block prints each of
these at the value every lobby on disk holds, so a lobby that held another
would part and be counted out. None does.

- `(Version: …)` and `(int) version`: the executable's build.
- `PLAYERS`: derived from the seats.
- `SCENARIO_TYPE`, `SCRIPT_TYPE`, `MODS`: scenarios and mods are cut from v1.
- `checksum_deep`, `checksum_window_size`, `checksum_failure_threshold`:
  the engine's, not a lobby's.
- A row's `flags` (derived from its kind: 7 a person, 1 the computer; bit
  0x2 not established), `who` and `play` (the seat's place), `pauses`,
  `pop_cap_frame` and the ten `synced_*` counters (the game keeps them,
  and they are 0 at the start).
- A row's `diff`: 2 on every seat on disk whatever `DIFFICULTY` says. Its
  writer is not read.
- A random nation: whether the block prints the drawn nation is not
  established.
- An open, closed or observing seat: no lobby on disk has one.
  `MAX_OBSERVERS` is offered, an observer seat is not.

**Not established**:

- the pairing of each setting with its category, past the consumers and
  corroborations of §4, and so whether a row's name is what the original
  shows for that byte (the harness lobby reads "Defender Bonus: Double
  Resources" for `STARTING_RESOURCES2 1`; 4184a);
- `gameinfo_flags`' bit order, and `handicap`'s and `team`'s
  printed meaning (every seat on disk prints `team 8` and `handicap 0`);
- that the original prints a typed name or a leader's name exactly as
  typed: names here are ASCII, one space between words;
- ~~whether a lobby with a map style other than Great Lakes starts~~: every
  lobby on disk starts from the page, on all 17 composed styles (item 4215,
  §5, `LOBBY_STARTS`). A style or a lobby no capture holds is not scored;
- the client's ground past its tileset and seed for a style other than
  Great Lakes: the art the tileset names is loaded as Great Lakes' is, and
  no photograph of another style's ground is compared. The camera is the
  pair's pinned one on every game, so a page's game opens on that point of
  its map, not on its person's capital (4215a);
- the window session by a person: the keys are tested through `Page::key`,
  and the window's own key mapping (`App::page_key`) was not driven by
  hand.
