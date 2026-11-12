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

`Lobby: the page sets N of 161 lobbies · the harness lobby's start built
from it`: **`lobby::LOBBY_PAGE`, 161 of `diff::START_LOBBIES` 161** at item
4184. The parser is `diff::floors::lobby_line` with
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

**Red, three ways.** A bent seed (12346) builds another `Sim`. On the
counter: the page's nation mapping off by one gives 0 of 161, the harness
lobby first parting on `PLAYER 0 tribe`, ours 5 theirs 4; the difficulty
control dropped gives 30 of 161 (the lobbies at `DIFFICULTY 0`), run1000
first parting on `DIFFICULTY`, ours 0 theirs 3. Each was one line,
reverted. `a_bent_event_parts_the_lobby` keeps a bent nation red as a test.

## 6. Confidence, and what is not established

**Diff-backed**: that the page's events reproduce all 161 lobbies' blocks,
line for line and field for field; that the harness lobby's start built
from the page is run53's built start, whole, on frames 0 and 40.

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
- whether a lobby with a map style other than Great Lakes starts. The
  page sets any style, but `from_lobby` composes only Great Lakes (14), and
  the client prints the walk's notes for any other style (`docs/START.md`
  §2). The counter scores the block, not the start;
- the window session by a person: the keys are tested through `Page::key`,
  and the window's own key mapping (`App::page_key`) was not driven by
  hand.
