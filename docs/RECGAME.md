# The recorded game: the `.rcx` container

`docs/ORACLE.md` Part 1 derived the recorded-game container's shape from the
symbols in 2026-08-20 and deliberately stopped short of a reader, because a
reader written against no sample cannot be wrong in any detectable way. The
sample now exists — `ron.heavengames.com`'s downloads section, fileid 2170,
the one recording there that names EE, saved under the gitignored
`game/external-recgames/` — and this document is the full walk, read from the
decompile and checked byte-for-byte against that file.

**How this was established.** Every function named here was read in the
Ghidra export (`~/ghidra-projects/decomp`, outside the repo): `RecordGame::read_header` /
`write_header` (0x953710 / 0x952a50), `read_package` / `write_package`
(0x952d90 / 0x952fb0), `read_random_game_info` / `write_random_game_info`
(0x9533e0 / 0x9534e0), `read_rules` / `write_rules`, `init_playback`,
`Game::run_playback` (0x586ea0) and the setup stretch of `Game::run`
(0x584590) for the sequencing, `Game::walk_data` (0x589600),
`GameInfo::walk_data` (0x5d6570), `Game::walk_rules_data` (0x589550),
`Types::walk_rules_data` (0x669800), `Balance::walk_rules_data` (0x582cc0),
`String::walk_data` (0xa1b2d0), and the four `DataWalk` primitives
(`LoadGame`/`SaveGame` `walk_function` / `walk_test`). Struct offsets are the
PDB's, via `types.txt`. The sample file is a **2017-build** recording
(`00.2017.08.2100`) read against a **2024-build** decompile (`00.2024.06.20`
is in this install's executable); everywhere the two could disagree is flagged
below.

**Confidence.** High throughout the parsed stretch: the wire protocol (three
short primitives read end to end), the package record (unchanged since the
first reading), the section order (two call sites), and the whole header are
verified against the sample by the reader in `crates/rondata/src/recgame.rs`
— every byte between the gzip header and end of file is accounted for, and
the embedded combat table equals the composed one cell for cell (§6), which
a misaligned parse could not survive. The Types stretch of the rules section
is *not derived* — it is skipped by landmark (§4.2).

## 1. The container

- **Extension `.rcx`**, filename from `String::time_stamp` in the profile's
  recorded-games directory (`PlayerProfile::get_record_game_directory`;
  `init_record` mkdirs it). The extension's literal text lives in the
  runtime string table, so it is known from the sample, not the symbols.
- **A finished `.rcx` is a bare gzip stream — but it is not written that
  way.** ~~The shipped recording path opens the gzip arm.~~ Corrected by the
  second reading: `File::open`'s mode word is a bitmask (bit 0 set → zlib
  via `gz_open_unicode`, clear → stdio via `_wfsopen`), and
  `write_header` opens mode **2** — plain stdio — so the game records
  **uncompressed** for the whole match. `finalize` (0x952b40) then opens
  `recordgame.tmp` in mode **3** (zlib write), streams the raw file through
  it via `Filemap` + `File::write` (which routes to `gzwrite`), removes the
  raw file and renames the tmp over it — or just deletes everything if
  `empty` never cleared. `read_header` opens mode **1** (zlib read).
  **Consequence:** a recording whose game crashed before finalize is the
  same record, raw — a reader should sniff `1f 8b` rather than assume gzip.
  Two of this install's recordings are exactly that, and both stop
  mid-record (§8.1).
  The sample is finalized: 199,968 bytes compressed, 1,676,994 decompressed.
- Everything below describes the **decompressed** stream. All integers are
  little-endian. There is no framing beyond the walk itself: the format is
  the flattened object graph, so the struct layouts *are* the format.

## 2. The three wire primitives

Everything in the header and rules is written by exactly three operations:

| primitive | bytes | meaning |
| --- | --- | --- |
| `walk_function(begin, end)` | `end - begin` | raw memcpy of a struct range, as laid out in memory |
| `walk_test(name)` | 1 | section marker: the low byte of the section-name `String`'s **case-insensitive** hash (`hash_value_insensitive` at both use sites; the second reading settled the field the generate-side garble left open). On mismatch the reader (`LoadGame::walk_test`) reports "Error loading section, probably in *last section*" at severity 3 — **a warning; it keeps reading** |
| `String::walk_data` | 4 + 2·n | `u32` character count, then n UTF-16LE code units, no terminator; count 0 = empty string, nothing follows |

The section names live in the runtime string table (`int_str_array`), so the
marker values are not derivable from literals; the observed bytes are recorded
in §3/§4 and pinned by the reader. `DataWalk::checksum` gates a few
alternative branches in the walkers but is 0 for the recording paths.

## 3. The header

Written by `write_header` = `Game::walk_data(SaveGame)` then
`String::walk_data(info.save_name)`. In stream order:

### 3.1 Game → GameInfo

| bytes | content | sample |
| --- | --- | --- |
| 1 | marker, Game section | `0x16` |
| 1 | marker, GameInfo section | `0x42` |
| 4 + 2n | version string, literally `"(Version: <v>)"` — built from `Version::get_string` wrapped in localized parens | 26 chars, `(Version: 00.2017.08.2100)` |
| 4 | `GameInfo.version` (+0x00) | |
| 16 | +0x04..+0x14: `seed`, `checksum_deep`, `checksum_window_size`, `checksum_failure_threshold` | seed `0x144bd480` |
| 4 | +0x14 `flags` | |
| 29 | +0x18..+0x35, walked one byte at a time — the lobby options, named by `GameInfo_u_24_s_0`: `team_style, map_style, map_size, players, max_observers, game_speed, game_rules, difficulty, starting_town, starting_resources, starting_resources2, tech_cost, reveal_map, pop_limit, rush_rules, cannon_times, starting_technology, starting_technology2, ending_technology, elimination, victory, wonderwin, score_goal, popwin, time_limit, chairs, econwin, scenario_type, script_type` | sample: `difficulty = 5` — the Toughest the uploader promised; the per-player `diff` byte below stays 2 |
| 1 | +0x35..+0x36: `mods` | |

### 3.2 The eight player records

For each of the 8 lobby slots (`GameInfo.player[n]`, stride 0x8c):

| bytes | content |
| --- | --- |
| 1 | marker, player section (`0x0f` in the sample) |
| 2 | `Player.flags` (+0x30) |
| 0x39 | **only if `flags & 1`** (slot active): Player +0x00..+0x39 raw — the ten `synced_*` counters, `caravan_frame`, `pop_cap_frame`, then `flags`(u16), `tribe`, `who`, `team`, `handicap`, `play`, `pauses`, `diff` |
| 4 + 2n | only if active: `Player.name` (+0x40) |

`platform`/`platformID`/`net_player`/the `accum_*` bytes are not walked.
`Player::walk_data` (0x6ee2d0) is the same code un-inlined and is the cleaner
authority; note `flags` rides the wire **twice** per active slot — once alone,
once inside the 0x39-byte block.

### 3.3 The GameInfo tail — version-forked

`sGameSaveVersion` selects the branch; `Game::run_playback` **forces it to
0x10** before `read_header` in this build, so the 2024 reader always takes
the second row. The sample file parses under it, so the 2017 writer used the
same layout (or one byte-compatible with it for an empty-mod game).

| `sGameSaveVersion` | layout |
| --- | --- |
| < 0x10 | four Strings: `scenario_script` (+0x4f4), `scenario_path` (+0x508), `scenario_dir` (+0x51c), mod path |
| ≥ 0x10 | `u64` workshop-mod id, `scenario_script`, `scenario_path`, the resolved scenario/mod dir String, `u64` dropdown-mod id, dropdown-mod dir String |

On read, the two workshop ids drive Steam-Workshop mod resolution
(subscribe-now, error popups); the strings land in the fields named.

### 3.4 The Game fields and save_name

| bytes | content |
| --- | --- |
| 0x194 | `Game` +0x550..+0x6e4 raw: `frame` first, then the run-state stretch up to `balance` |
| 8 | `semaphore` (BitMask<256>) +0x00..+0x08: `bits`, `size` |
| `size` | the semaphore mask bytes (32 in practice) |
| 4 | `graphic_tick` (+0x844..+0x848) |
| 4 + 2n | `info.save_name` — **empty for a random-map recording**; non-empty means a scenario/saved-game recording, which changes §4's gating |

## 4. After the header

Sequencing is in `Game::run`'s setup: **[CtW header] → [embedded save] →
random-game info → rules → packages**. The two bracketed sections, absent
from the sample and from any standard random-map recording (second reading):

- **CtW** (`semaphore.ptr[2] & 2`): `write_ctw_header` runs
  `ConquestGame::walk_data` under a fresh walker. Unread further, with the
  rest of CtW.
- **Embedded save** (`save_name` non-empty): `write_save_game` is not a walk
  at all — it opens the named save file and copies it into the stream **one
  byte at a time to EOF, with no length prefix**, so a scenario recording
  cannot be skipped past without knowing the save format's own structure.

### 4.1 The random-game info blob

`write_random_game_info`: for each lobby slot with `flags & 1` and `who < 8`,
1 byte (`Player.team`) + 4 bytes (a leader-table field at
`leaders[who * 0x6eec] + 0x39c` — **the leader's rolled nation**: in the
sample the five values are exactly the five slots' `tribe`s, in `who` order);
then 4 bytes once (`World` +0x30). Total `5 × active + 4`. **The playback side reads this blob into a
malloc'd buffer and frees it unused** (`read_random_game_info`) — everything
it duplicates already arrived inside GameInfo, so a reader can treat it as
skip-only. The sample: 5 active players → 29 bytes.

### 4.2 The rules section

`write_rules` = `Game::walk_rules_data`. The recording embeds the **loaded
rules tables**, so playback does not trust the install's XML:

1. 1 byte: rules section marker.
2. `Types::walk_rules_data`: 806 type objects (`types.list`, 0xc98/4), each
   dispatching its own virtual `walk_rules_data` (vtable +0xc4). The second
   reading derived the **per-class grammar**: every record opens with 90 raw
   bytes (`Type` +0x04..+0x5e) then the type's name String; the tails are 48
   raw bytes (`SpellType`), 27 + eight Strings (`TechType`), 152 + two
   `SimpleArray<u16>` (`ObjectType`; an array is `u32 len`, then if nonzero
   7 more header bytes and `len` u16s), +792 (`UnitType`), +49
   (`BuildType`), +68 (`GoodType`). A walk of exactly 806 records spans the
   sample's stretch precisely — but the stream carries **no class
   discriminator** (dispatch is positional through `types.list`), several
   tails admit the same continuation, and a dynamic program shows the parse
   is ambiguous without the loader's construction order. So the section is
   readable in principle but only against the type-table loader, and the
   reader still ends it by landmark.
3. `Constants` +0x00..+0xd40 raw (the whole loaded `Tuning` block, ending at
   `curr_element`), then +0x804..+0x808 again (`mongol_three_mil_cavalry` —
   4 redundant bytes).
4. `Balance::walk_rules_data`: **493 × 493 u16 walks — the composed combat
   table**, 486,098 bytes, the same table `rondata --types` checks from a
   `DUMP_ALL` dump. This is most of the file, and it makes any recording a
   combat-table ground-truth source that does not need the hang-prone
   `DUMP_ALL` run.
5. 24 tribes (`Tribe` stride 0x5f0): each 1 marker byte + raw +0x54..+0x6c
   (0x18 bytes) + raw +0x70..+0x5f0 (0x580 bytes) = **0x599 bytes each,
   fixed** — a 24-car train with the same marker byte every 0x599 bytes,
   which is the landmark the reader uses to end the rules section without
   deriving step 2. A stride-repeat alone false-positives on the zero runs
   inside Types, so the reader requires a **nonzero** marker and takes the
   **earliest** candidate from which the package stream parses clean to
   exact end of file — in the sample that is offset 0xf1e51, marker `0x8f`,
   and §6's table equality proves it is the true one.

Gating: rules are written only when `save_name` is empty, and read only when
`save_name` is empty **or** the recording's patch version is < 4 — a
scenario recording on a modern patch carries no rules section.

### 4.3 The package stream

To end of file, exactly as ORACLE.md Part 1 had it, one record per command
package:

| bytes | field |
| --- | --- |
| 4 | `frame` — **`Game.frame` at write time**, not a `CommandPackage` field |
| 4 | `play` (issuing player slot, `CommandPackage` +0x04) |
| 4 | `valid` — 1 on the sample's first record, 0 on the other 21,883 |
| 4 | `stamp` (`CommandPackage` +0x00 — wire order is not struct order) — contiguous 1..N in the sample, one per record |
| 2 | `size` (≤ 512, the buffer's size; 10 bytes for 83% of the sample's records) |
| size | `data` — the command payload |

`CommandPackage.group` (+0x0c) and the embedded `Random` are never
serialized, as the first reading had it.

`read_package`'s contract: called with the current frame, it reads the next
record's `frame` and, if it is in the future, seeks back 4 bytes and returns
0 — packages are consumed when their frame arrives. EOF flips the playback
semaphore. There is no count field; the stream runs to end of file
(ORACLE.md's "where read_package stops" is hereby confirmed: EOF).

~~The `data` payload's encoding is the **command format**, `CommandManager`'s
territory, and is deliberately out of this document's scope — it is the next
reading.~~ Read the same day: `docs/COMMANDS.md`, decoded by
`crate::commands` — every payload in the sample parses, 25,779 commands.

## 5. What is not established

- **The Types section's class sequence** (§4.2 step 2): the per-class
  grammar is now derived (second reading), but which of the 806 indices is
  which class is the loader's knowledge, not the file's — provably, the
  bytes alone are ambiguous. Until the `types.list` construction order is
  read, the reader locates the tribes train by the landmark of §4.2 step 5.
- **`GameInfo.version`'s packing.** The sample holds `0x06740124`, which
  does not decode under the `Version` struct's milestone/month/day/build
  bytes (day 116). `get_patch_version` compares it lexicographically and the
  sample takes the `> 3` arm (its random-info section exists), so the gate
  works; what the bytes mean is open. The banner *string* is written and
  never parsed back — playback checks no version at all, which is why old
  recordings are silently misread rather than refused.
- ~~**The marker-byte hash** — which of the two hash fields supplies it.~~
  The second reading settled it: `hash_value_insensitive`'s low byte, the
  field named at both the write and the compare. The section names still
  live in the runtime string table, so the observed values remain the
  practical anchor: Game `0x16`, GameInfo `0x42`, player `0x50`, rules
  `0x92`, tribe `0x8f`.
- ~~**The +0x36..+0x38 gap**: the decompiler lost one call's arguments
  (§3.1's last row), so the 1-byte row could be 3 bytes.~~ **Settled by the
  sample**: with 1 byte the eight player records align exactly (one marker
  byte, eight times, `0x50`); with 3 they cannot.
- ~~**The command payload encoding** (`CommandPackage.data`) — the next
  mechanic, not this one. Until it is read, the packages give frame, issuer,
  stamp and an opaque payload.~~ **Done, 2026-08-24**: `docs/COMMANDS.md`,
  the full 0x00–0x51 dispatch table with every payload in the sample
  decoding to exact size.
- **2017↔2024 drift, narrowly**: §6's table equality pins the balance and
  tribes spans for the sample, so the one span still resting on a 2024-only
  size is the **start** of the `Constants` block (0xd40) — if the 2017 block
  was a different size, the constants/types boundary is off for old files
  and nothing else moves. A recording made by *this* install has no such
  gap.
- Whether `write_ctw_header`/`read_ctw_header` (Conquer the World wraps the
  container) change anything — out of v1's scope with the rest of CtW.
- **Two issues in one turn** (item 4285, parked 4285b): the 2017 sample
  carries nine packages holding two or three `group`-and-order pairs, 38
  orders (`docs/CLIENT_INPUT.md` §30.2). `minted_packages` writes one order
  a package on strictly increasing frames, so a person who issues twice
  inside one turn is not written as the original writes them.

## 6. What the sample established

`rondata <install> --recgame <file>` parses the whole stream and then checks
the embedded combat table against the one the loader composes from the
install's own XML (`docs/COMBAT.md` §15.2–15.3). On the heavengames sample:

- **All 243,049 cells of the 493 × 493 are equal**, across the seven-year
  build gap (a 2017-build recording against the 2024-build install's data) —
  EE's composed combat table did not change between those patches, and the
  parse is byte-exact or the comparison could not survive.
- The check is pinned as an install-gated test
  (`the_embedded_combat_table_is_the_composed_one`) and it has failed on
  purpose once: one flipped byte in the balance span is caught and named.
- So a recording is now a **combat-table ground-truth source that does not
  need `DUMP_ALL`** — the switch that hangs the game (`docs/ORACLE.md`) —
  and any future recording, including ones this install makes, re-runs the
  same check for free.
- The order stream is real: 21,884 packages over frames 0..21044 (~23
  minutes at 15 fps), **every one issued by play 0** — in a 1-human-vs-4-AI
  lockstep game only real input travels; the AIs are recomputed on playback.
  Stamps run 1..21,884 contiguously; `valid` is 1 on the first record only;
  20,285 frames carry exactly one package.
- **The blind second reading confirmed the whole structure independently**
  (`docs/audit/2026-08-24-recgame.md`): its byte map of the 943-byte header
  is identical, and its backward dynamic program over the full 1.68 MB
  proved the package stream can start at exactly one offset — the same byte
  the landmark finds.

## 7. Playback-oracle preflight (2026-10-15)

The click-free runner accepts `--recording <absolute.rcx>` instead of lobby
options. It retains the source path/hash and reads only §3.1's seed/map prefix
for identity checking; this is **not** a second full container parser. A fresh
capture directory links the source as `input.rcx`, avoiding the original
command-line tokenizer's treatment of spaces and hyphens in filenames.

`RON_REPLAY` selects the native menu's playback result (13), after calling the
native String assignment for `record_game.file_name` from `sys.playback_file`.
Evidence: `SetupWin::exec@005c67f0` instructions at `005c6970..005c697f`,
`TopMenu::check_result@0059b1e0` at `0059b5c0`, and
`Main::run_game_loop@00595f40`'s playback arm. The instrument checks the
assignment and result bytes before patching. It neither loads simulation
state itself nor routes through solo setup. These readings retain blind-review
debt; run900's opening is executable evidence that this build plays the file.

**Measured opening:** run900, Seifer008's 2017 EE match, frames 0..36,
clean exit, map 7, five players, and all five backed-up settings restored.
The recording's seed **340513920** survives `rise.ini` seed **12345**.
`GAME INFO`/`GAMEINFO`, `FRAME` and closing records use the solo log's shape.
`GameLog::init@00933190` names the output **`gamelog_p.txt`**, not
`gamelog.txt`; the runner's stall detector and verifier use it. A playback receipt requires install → menu 13 → contiguous
frames → menu 21, with no solo-modal events, faults or dropped records.

**Absent evidence matters:** the existing Rust parser accounts for all
21,884 packages and 25,779 commands, but finds **zero** `CheckRandom`,
`CheckSums`, or `NextCheckSum` commands in this recording. Its sample test
pins that absence. The emitted checksum log is a new playback measurement,
not an embedded reference. No frame of agreement against recording checksums
can be reported for this file. The combat table still matches all 243,049
cells of this install; that does not certify dynamic playback fidelity.

**Scope correction:** a standard random-map recording does not embed a full
initial entity/world snapshot (§4). It carries setup and loaded rules; the
native game regenerates the world. Only the embedded-save arm carries a save.
A replay harness must explicitly choose reconstruction or a separately captured
starting-state oracle. No snapshot loader or sim parting floor is established
by this preflight. Full playback and its end condition are recorded separately
in run900's ledger/journal; the opening alone proves no end-to-end result.

**Seed control and logging coverage:** a second 37-frame run on the committed
instrument requests ini seed 54321 and still observes 340513920. All 37 RNG
states agree with the ini-12345 opening, as does the complete frame-1 value
block (terminal blocks excluded, no field exclusions). A wider 284-frame
attempt is retained for overlap: the leader-only long recipe agrees in all
1,981 complete printed leader records over frames 1..283, including the two
flag lines preceding each `LEADERDATA`; all 285 shared RNG frames agree.
These are native/native reproducibility checks. The long recipe records
leaders per frame, wider objects only at endgame. `end:MISC` alone emitted no frame
blocks and is not a sufficient playback-oracle recipe.

**Completed range:** run900's successful long recipe records trace frames
0..21044 and closes at dump frame 21045, with exit 0 and five settings restored.
There are 21,044 consecutive complete leader dump blocks (147,308 records).
All 21,045 RNG states match the earlier long trace, whose lifecycle failed.
Quit is explicitly scheduled at the final recorded frame; natural EOF shutdown
is not established. The 2017 recording's original-match fidelity and the
self-contained sim score remain unmeasured.

**The walk, minted (item 4336, 2026-11-17).** run900's captures are gone;
run1650 retakes the opening with the start dump and run1651 plays the file
whole with every frame's draws (`docs/RUNS.md`). Two rows, in
`rondata::diff::sample`:

- **The start built from the header.** The header read as a `GAMEINFO`
  block (`sample::header_lobby`) equals run1650's printed one on every
  keyed field but the version line: the playback prints the running
  build's `00.2024.06.2000`, the header its own `00.2017.08.2100`;
  `(int) version` is 108265764 on both. Handed to the start builder, it
  builds the playback's start block whole, **18,315 of 18,315**. It was
  minted at 17,163: this is the first team game on disk (`TEAM_STYLE` 12,
  the person alone on team 0, the four computers on team 1), and
  `build_game` builds a seat's teammates at once after it
  (`docs/START.md` §2 step 5).
- **The walk's word: 98 of 21,044.** The built start under the turn
  pump on the recording's own packages. A package acts as
  `GameInfo.player[play].who` (`process_group@0094a0c0`). The person is
  `play` 0 at `who` 3, and the stream used to take `play` for the
  leader, so every order went to who 0's units. On frame 98 both sides'
  first draw is `Animal::do_idle+0x83`. Theirs then draws at `+0x1a4`,
  `+0x1d4` and `+0x212`, and ours goes on to `Guy::set_anim+0x97a <
  Guy::inc_time+0x271`, which is their fourth draw: one animal's idle
  arm takes three draws ours does not.

**The word moved: 98 → 444 (item 4347, 2026-11-17).** run1654 retakes the
opening to frame 100 with every unit and figure printed over 84..101
(`docs/RUNS.md`). The animal on 98 is herd 0's peacock (`TypeIndex` 413,
`HERDPEACOCK`): theirs' `8/1`, ours' `8/0`. Positions and the herd's
`(19, 9, 19, 8)` agree on both sides. The pieces do not: run1654's `8/0`..`8/3`
wear 60078, 60079, 60080, 60078, ours wore 60079, 60080, 60078, 60079,
so the 99-frame idle ran out on a different bird. Ours' `8/0` stands
413 from the herd centre and takes the far arm, which draws nothing.
Theirs' `8/1` stands near and draws three times.
`GraphicPieces::get_unit_gpiece@0090c030:37` adds `o` to
`GameInfo.seed`, the lobby's (340513920 here), and this crate added it
to `world->seed` (48607). On every lobby at seed 12345 the two are 12345
and 7236, the same mod 3, so no walk parted on it before. The fix is
`sim::Sim::info_seed`, `sim::ai::Lobby::seed` from `GAMEINFO`'s
`(int)seed` (falling back to the world seed where none is printed).
The test is `a_gaia_figure_s_variant_is_the_lobby_seed_s_not_the_world_s`.
The person has ordered by then: `process_group` on 8, 9, 21 and 28
(run1650). run1654's window walks with the start: 84..91 agree whole.
**On 92 the person's scout `3/0` holds an `EXPLORE_TO` in theirs and a
`MOVE_TO` here**, the next parting by state, ahead of the draws' 444.

**Not established:** `Leader::diplomacy`'s war plan still reads the world seed
(`docs/LEADERS.md` §8.2, item 4341's).

**The word moved: 444 → 2888 (item 4351, 2026-11-17).** The package on
recording frame 91 is `group [0]` then `move_to (39037, 17733)` with
`orders` 1 (`MOVE_TO`) and `queued` 1 (`QUEUE_LAST`), the person's
shift-click. `Group::action_move_near@00704990` hands a scout (`role &
0x10` without `unit_masks & 0x40000`) to `add_move_facing_order` whatever
the group's size. There, a `QUEUE_LAST` `MOVE_TO` of a `role & 0x10` type
sets `unit_masks |= 0x4000000` and becomes an `EXPLORE_TO`
(`005e55c0:24`). This crate's closure answered no, so the scout walked a
plain `MOVE_TO`. The value diff on 92, run1654 `UNITS` for `3/0`
(`TypeIndex` 69, `SCOUT`): order index theirs 3, ours 1 before, 3 after,
toward (39048, 17736) on both. The person's next nine shift-clicks
(recording frames 98 to 112) are explores too. The bit is carried as
`sim::Unit::explore_queued` with its one setter, the clear every
`Unit::clear_orders` makes (in `Sim::close_orders`), and both readers:
`Unit::work@0060d180:122`, which turns a `MOVE_TO` at the head into an
`EXPLORE_TO` at `QUEUE_FIRST`, and the human block of
`Unit::think@005f6e40:231`, which clears it on an idle special unit with
no group. run1654's window agrees whole, 18 of 18 records (9 parted
before). The draws part at frame 2888, draw 32.

**Not established:** whether `work`'s arm or the
think's clear is reached on this file (the move is the conversion's,
scored by mutation in `docs/journal/2026-11-17-item-4351.md`).

**The hook's take and the word 2888 → 3019 (item 4359, 2026-11-18).**
run1662 plays the sample whole under the `RON_CHECKSUMS` hook (22,077
sets, `docs/RUNS.md`), and `diff::sample`'s
`the_sample_s_lines_against_the_hook` holds our walk's twelve lines
against it on every frame. Theirs' walls stay empty all game; the first
shot in flight is on 2929 and the first death object on 3365, so the
sample is the first walk on disk where ammo and deaths are not empty.
Our walk holds its first shot on 2929 too. By state, builds, leaders and
world part on frame 0 and goods on 1, though the start block agrees
record for record; units part on 92, a frame whose `UNITS` records agree
whole; then groups 927, guys 2920, cities 3176, items 3976
(`SAMPLE_LINES`). The draws' word moved twice. On 2887 the person's units
(sent on 2747) strike computer 4's capital `4/2000`, and
`take_damage`'s city-building arm rings the city's bell
(`Sim::city_struck_alarm`, `docs/AI.md` §416.2): 2888 → 2908. Without the
arm, run1662's cities line parts on 2888 (ours 3376104779, theirs
164251019). On 2901 computer 4's `ai economic` asks `was_city_raided`,
which this crate read as never, though the capital's light blow had
stamped `raid_stamp` 2887; it reads `CityData +0x18` now
(`was_city_raided@009fd3e0`): 2908 → 3019.

**The word 3019 read on run1674's window (item 4380, 2026-11-18).**
run1674 prints every unit and figure over 3013..3021 (`docs/RUNS.md`).
The 4359 hypothesis does not stand: every army of every computer is
empty in our walk over 3014..3019, and the units that part are not
soldiers. **One unit parts on every block, the person's SCOUT `3/0`**
(`TypeIndex` 69, land), on the tenth `EXPLORE_TO` of its queue, to
(42216, 31320). On 3013 ours stands at (39311, 32904) with the move's
`dest_x` 39912, and theirs at (39801, 32538) with 40680. The two paths agree
on their first 61 nodes. Ours holds three more, (41448, 32088), (40680, 32856) and
(39912, 32856). They are the rest of a grid detour our walk pushed on 2892
round computer 1's capital (the Woodcutter `1/2001` at (40128, 31680),
the Market `1/2010`, the Library `1/2005`), after both sides refused
the straight line on 2891 (`Unit::do_move+0xe84 < Unit::do_explore_to`,
one draw each). Theirs' scout at 3013 walks straight from the south to
(40680, 31320) through the ground ours went round. On 3019 theirs' scout passes
computer 1's MERCHANT `1/17` (`TypeIndex` 61), close enough for theirs'
draws 1 to 5 (two `move_step+0x823` pairs and a `turn_towards`) to be the
two units' steps; that is not checked unit by unit. On 3020 `1/17` takes a collision at (39826,
32220) in theirs and none in ours. **Not established:** whether theirs'
capital stands on the cells ours does on 2891, which needs a `BUILDS`
print there, and the grid route theirs took.

**The lines by range at the start (item 4384, 2026-11-18).** run1678
replays the sample to 100 with the range record and all twelve lines kept
(`RNG_MASK` 0xfff, `docs/RUNS.md`). Its 111 sets are run1662's. In
`diff::sample`, `the_sample_s_ranges_name_each_first_parting` holds each
line range by range (`SAMPLE_RANGES`). A leaders range starts at
`LeaderData +0x8`.

| line | frame | first parting: range record field: ours, theirs |
|---|---|---|
| units | 92 | 439 0 `unit_masks`: 8, 0x4000008 |
| builds | 0 | 10 2000 `ever_seen_completed`: 23, 3 |
| leaders | 0 | 1 0 `chat_status[3]`: 0, 1 |
| goods | 1 | 145 29 `ever_seen`: 0x04, 0x17 |
| world | 0 | 3626 `seen2` +0x2340: 0x04, 0x17 |

State 0 parts on 28 ranges, and all but nine are the team game's. Builds'
21 rows are `ever_seen_completed`. `Wall::activate` copies the owner's
`ally_mask` (`+0x6929`) into it, and theirs' mask for leaders 0, 1 and 2
reads 3, 2 and 7 on state 0, against ours' whole team, 0x17. Each
computer's `chat_status` toward the person is 1, and so is the person's
toward each computer. `Game::init_teams@0058ae70`'s tail writes 0 there
only for a teammate once any seated leader has one (`semaphore[0] & 0x80`).
Ours writes 0 on every seated slot, which is the free-for-all arm. The
person's `diplos`, `treaties` and `aggression` part on the unseated slots
5 to 7, where theirs holds a teammate's values. The nine rows left are
leader 2's `oil_patches` order, two buildings' `z` and six `seen2` cells.
Units' 92 is the explore bit, which item 4351 carried as
`sim::Unit::explore_queued`, outside the word `checksum/units.rs` builds.

**Item 4388: 24 of the 28 taken** (`SAMPLE_RANGES`). *Diff-backed* on
run1678's state 0, each by a mutation:

- **The mask.** `Setup::build_game@005ac190` calls `Leader::init` over
  `start_list`, the start permutation, and `Leader::init`'s loop ors a slot
  into `+0x6929` when `LeaderData::is_ally@006edb50` holds, which reads
  **both** leaders' `diplos`. A teammate not yet initialised is left out:
  the permutation 1, 0, 2, 3, 4 gives 2, 3, 7 and 0x17.
  `Wall::activate@0063e4b0` ors the stored byte. Builds 21 → 2 parted.
- **`chat_status`**, `Game::init_teams`' 0x80 arm: with a seated
  teammate anywhere, only a teammate pair reads 0.
  `Game::is_cooperative@00594810` is team style 8 to 10 (11 and 12 with
  two records on one `who`, not carried).
- **An unseated slot is the lobby's record 0.**
  `LeaderData::get_player@006ec0f0` returns 0 when no active record
  carries the leader's `who`. The sample's record 0 is the person (team 0),
  so the person's `Leader::init` writes slots 5 to 7 a teammate's row.
  Leaders 6 → 1 parted.

Four ranges were left; item 4394 took the `seen2` cells (not a unit's
reveal, as guessed) and goods' `ever_seen`.

**Items 4394 and 4397** (their journals), *diff-backed* on run1678, each
by a mutation: frame 0's `update_all_seen` (`Game::run@00584590:722`)
shares each team's `seen2`; `plan_strategy:314` marks `new_rares` goods
`ever_seen`; `Setup::reinit_game_units@005abf90` (`Game::init:629`)
samples every `z` again; `build_cities@005ab910` leaves the oil to
`compute_all_territory`. By sum: goods 1 → 4094, world 0 → 1716, builds
0 → 76, leaders 0 → 241. State 0 is whole by range.

**Item 4405: the window whole by range.** *Diff-backed* on run1678, each
by a mutation:

- **Builds on 76**: the person's `city_gather` (kind 0x1a1) lays five
  Farm points on `3/2000`, appended (`622efe`) in `city_down` order.
  `PtrLinkListAbstract<GatherPoint>::walk_data@004708a0` walks from
  `head_node->prev` by `prev`, last point first: `758 2000 gather[k]`
  32640|31872 and three more agree. Builds by sum 76 → 525.
- **Units on 92**: `unit_masks & 0x4000000` (`Unit::explore_queued`)
  is on the word: `439 0` 8|0x4000008 agrees. Units 92 → 532.

All twelve agree to 100.

**Items 4412, 4414: run1697, to 560**. *Diff-backed*, each by a
mutation:

- **Leaders on 241**: leader 4's `strong[2]` toward teammate 2, 32|0.
  `Leader::diplomacy`'s `6bd02a` (`leader_flags & 4`) skips the building
  lines and the war count for a computer target. → 2112.
- **Builds on 525**: the Lookout `3/2008`'s `flags` 1|0x41;
  `Object::init@00647750:39` ors `0x40` for a `DETECT` type. → 3029.
- **Units on 532**: the Citizen `3/6`'s `THINK` walks
  `UnitOrder::walk_data@0047f2b0`, the flags byte (`04`), which ours left
  out. → 2112.

run1710 holds every first parting to 3,100 (`SAMPLE_RANGES_1710`):
groups 927 → 2888 (4419, 4422; GROUPS §24.2); world 1716 → 2921, no
exit lights a disc (4424; VISION §6); units 2112 → 2700, leaders → 2989,
allies bleed nothing (4429; ATTRITION); units → 2888, a refused
slot moves (4432; GROUPS §6.6); units → 2892, a `GARRISON` walks
target and `search`, groups → 3053, the bell's group is pushed (4440;
CHECKSUM §17).

## 7.1 The 2017 sample's first parting

**Diamonds on state 7502 (item 4569).** On run1786, player 3 gains rare bit
22. Its five capped goods and its spare cap now agree at 3440, up from 3200.
Leaders agree through 7649. At that landing the first disagreement moved
to units and cities on 7539. [Diamonds comparison](journal/2026-11-24-item-4569.md)

**Scout 3/0 on state 7538 (item 4639).** The recorded package sets its
auto-explore bit. At the second checksum of that state the Scout's mask
is 258 and both city marks are still zero. By the next state cities 1
and 2 both carry player 3's bit, 8. Emitting those live fields moves
units to 7612 and cities to 7825. Units, guys and world now first part
at the birth of Merchant 3/23. Draws first part at 7613.
[Scout comparison](journal/2026-11-24-item-4639.md)

**Merchant 3/23 on state 7612 (item 4641).** The Market's rally point
overlaps Merchant 4/16. On run1822's state 7611, the original's lookup
at that point finds 4/16 with all units eligible and returns no unit when
it uses player 3's visibility. Applying that filter removes an extra
placement at (11880,9912). The final placement (12552,10584), move target
(14952,7608) and destination angle 465764352 match run1786. Units, figures
and world agree through 7934. Group 2's order first differs on 7935, draws
on 7949. Goods first differs on 8548, four frames earlier downstream.
[Merchant comparison](journal/2026-11-24-item-4641.md)

**Group 2 on state 7935 (item 4642).** Player 0 sends the same fifteen
members to (2860,30423), as in run1786. Farm 2/2006 has nine points of
old damage. Setting its current alert in the original changes the muster
cell from (3,39) to (6,45), the destination our previous walk chose.
Run1786 now agrees on all eleven recorded lines through state 8000.
Run1662 first differs at 8523. Run1651’s draws agree through 8533.
[Recorded group destinations](journal/2026-11-24-item-4642.md)

**Forced March on frame 8522 (item 4643).** Player 3 repeats the preceding
selection of unit 27 and issues spell 636. Before ticking, run1824 records
a second checksum with ordinal 2. Its group 198 now has formation −1. The
recorded cast and all eleven kept lines agree on both sides of the package
and through 8535. Units and groups next differ at 9234, following the
assignment of hot key 55. The first unequal draw is on 9507.
[Forced March comparison](journal/2026-11-24-item-4643.md)

**Hot key 55 on frame 9233 (item 4644).** Five selected objects expand to
seven squad members. Copying their new pool slot restores all seven
members' group pointers. The original and the simulation agree on every
recorded line before the package, after it, and through state 9240.
Units next differ on 9506, groups on 9671. Draws first differ on 9507.
[Unit hot-key comparison](journal/2026-11-24-item-4644.md)

**Outdoor squad repair on 9505 (item 4645).** The regrown figure receives a
move to its own position. Its desired angle, order and next animation now match
the original. Units, guys, world and the draw stream next part on 9602,
when factory 3/2045 finishes unit 3/34 with an object as its rally point.
[Outdoor repair comparison](journal/2026-11-24-item-4645.md)

**The factory's object rally (item 4646).** Trainee 3/34 takes a move to
(8952,21240), leaving from (10872,20760). It previously left from
(11544,21624) and had no order. The entire 8001–10000 extension now
agrees on its eleven field lines. Both the first state difference and the
first unequal draw advance from 9602 to 10207.
[Factory rally comparison](journal/2026-11-24-item-4646.md)

**Leaving a city on the player's phase (item 4647).** Run1827's Senator
2/78 leaves Village 2/2022 on tick 10206. Once per 32 frames, phased by
owner, a computer-controlled occupant checks the city centre's alarm
and under-attack flags. When both are clear it takes the usual exit.
The two companions snap their previous facing to the captain's angle.
All eleven recorded lines agree over the exit. Buildings next differ on
10261 and the draw sequence on 11488.
[City exit, original predicate controls and tile-list boundary](journal/2026-11-24-item-4647.md)

**The retained gathering list (item 4648).** Building 2/2017 has 218 tiles
on 10260, 108 on 10261 and 68 on 10653. In each state the original list
has 320 allocated slots. Carrying the allocation as the list shrinks
moves buildings to 11487. The first state difference is now unit 16 on
10653. Draws hold to 11488.
[Gathering list comparison](journal/2026-11-24-item-4648.md)

## 8. The writer, our own file, and the original playing it (item 4058, 2026-11-10)

### 8.1 The round trip: every recording on disk, byte for byte

`recgame::write` is `parse`'s inverse, written from §2–§4 rather than from
`write_header`/`write_package`. `every_recording_on_disk_round_trips_byte_for_byte`
decompresses, parses, re-encodes every payload through `commands::encode`
(the decoder's inverse), writes, and compares the decompressed streams:
**50 of 50** (`RECORDINGS_ROUND_TRIPPED` of `RECORDINGS_ON_DISK`), 288,407
packages and 325,904 commands. The denominator is the bottle's `Logs\` (49,
this install's, 2026-08-20 to 2026-09-27, `00.2024.06.2000`) and the 2017
sample. The profile's own `Recorded Games` under `~/Documents` is not in it:
that directory is consent-gated, and run1036's ledger counted 942 files
there (parked 4058a).

**Which bytes are re-encoded and which are carried.** A span copied without
being parsed proves nothing about itself, so per section:

| section | re-encoded from parsed fields | carried as bytes |
| --- | --- | --- |
| §3.1 | the three markers, the version string, `GameInfo.version`, seed, the three checksum knobs, `flags`, the 30 option bytes | — |
| §3.2 | every slot's marker and `flags`; an active slot's 0x39-byte record field by field (`synced_*` ×10, `caravan_frame`, `pop_cap_frame`, `flags` again, `tribe`, `who`, `team`, `handicap`, `play`, `pauses`, `diff`) and its name | — |
| §3.3 | both u64 ids and the four strings | — |
| §3.4 | `frame`, the semaphore's `bits` and `size` (the mask's length), `graphic_tick`, `save_name` | `Game` +0x554..+0x6e4 (0x190 bytes); the 32 mask bytes |
| §4.1 | each `(team, nation)` pair and the closing `World` +0x30 word | — |
| §4.2 | the rules marker; the 4-byte re-walk, **derived** from the block's +0x804..+0x808; all 243,049 combat cells; the tribe marker | the Types stretch (500,334 bytes in all 50); `Constants` +0x00..+0xd40; the 24 tribes' 0x598-byte bodies |
| §4.3 | every package head, and every payload decoded and re-encoded | — |
| a cut tail | — | the partial record (below) |

Because the re-walk is derived, all 50 recordings confirm §4.2 step 3's
claim that the four bytes after the block are +0x804's.

**What the round trip found.**

- **Two recordings end mid-record**: `Playback - 2026.08.20 01'34'39 (Thu)`
  stops inside package 835's payload at byte 0x1002ed (frame 834, size 10,
  three bytes present; a 21-byte tail), and `Playback - 2026.08.24 17'20'55
  (Mon)` one byte into package 3,903's head at byte 0x115ff8. Both are
  uncompressed: games killed before `finalize` (0x952b40), as §1 predicted.
  Neither parsed before this item, because the tribe-train landmark wants
  the packages to reach end of file exactly. The reader now accepts a cut
  partial record, but **only when no candidate parses clean**, and keeps it
  in `RecGame::truncated` (`a_recording_cut_before_finalize_keeps_its_partial_record`).
- **`rename_city` carries stack bytes past its terminator.** Its 22-wchar
  name is copied whole, so whatever followed the NUL rides the wire:
  `17'27'47`'s frame 8516 "Norwich" and `20'17'33`'s frame 14401 "Napata"
  carry them. The decoder dropped them. `Command::RenameCity` now keeps them
  as `after_nul` (never read; a command built here pads with zeros).
- **Constant across all 50**: `Game`'s run-state stretch, the semaphore
  (`bits` 256, mask `08 00 …`), start frame 0, and the blob's closing word
  equal to `MAP_STYLE`. `graphic_tick` is not constant (15,805 to 29,911,208).
  The rules section takes **two values**: the three recordings with
  `STARTING_TECHNOLOGY` 3 part from the other 47 in 274 single bytes, all
  inside Types, each a type's value 2 lower (the age offsets). The 47 include
  the 2017 sample, on another map, another difficulty and another build.
  `what_our_file_borrows_is_constant_for_one_lobby_s_options` asserts this.

**Seen red**, each a one-line perturbation, run and reverted: an active
slot's `pauses` written +1 (0 of 50; the sample parts at byte 0xaa); the
re-walk taken from +0x800 (0 of 50, byte 0x7b37b); a camera's `zoom`
encoded +1 (0 of 50, byte 0xfa4ef); each combat cell byte-swapped (0 of
50); `after_nul` dropped (48 of 50, `17'27'47` parts at 0x13710f). The
constancy test with the rules split by difficulty instead of option 16
fails.

### 8.2 Our own file: the harness lobby

`recgame::from_lobby` builds a recording from the `GAMEINFO` block's fields
and `PLAYER` rows, the same input `diff::from_lobby` builds the start from
(`docs/START.md` §7), and a package list. Our file is run53's lobby (Great
Lakes, seed 12345, tribes 4 and 11) with `no_input_packages`, closing on
frame 24,000: `~/ron-golden/recordings/item4058-ours-greatlakes-24000.rcx`,
written by `the_lobby_s_recording_is_written_and_reads_back` under
`RON_RCX_OUT`.

| section | computed from the lobby | borrowed from a donor | chosen |
| --- | --- | --- | --- |
| §3.1 | version string, `GameInfo.version`, seed, checksum knobs, `flags`, the 30 options | the Game and GameInfo markers | — |
| §3.2 | each slot's `flags` (both copies), `synced_*`, `pop_cap_frame`, `tribe`, `who`, `team`, `handicap`, `play`, `pauses`, `diff`; inactive slots' `flags` 0 | the player marker | `caravan_frame` 0 (not printed); names `Player <play + 1>` (the block prints the leader's name) |
| §3.3 | ids 0, strings empty (`MODS 0`) | — | — |
| §3.4 | `frame` 0, `save_name` empty | the 0x190-byte run-state stretch; the semaphore | `graphic_tick` 0 |
| §4.1 | `(team, tribe)` per active slot, `MAP_STYLE` | — | — |
| §4.2 | — | **the whole rules section**: Types, Constants, the combat table, the tribes | — |
| §4.3 | stamps, frames, the closing package | the frame-0 `leader_options` payload (the profile's auto-manage settings; the same 33 bytes on all 49 of this install's recordings) | — |

The donor is the first finished recording on disk with our version string
and our 30 option bytes, but for `TEAM_STYLE`; `from_lobby` refuses any
other (`donor_serves`), because §8.1 shows the rules move with the options.
**`TEAM_STYLE` alone does not move them** (items 4382, 4387): the three
pairs in the profile's `Recorded Games/` that differ in that byte alone
(East Indies, styles 0 and 1, difficulties 5, 5 and 0) carry equal rules,
run state, semaphore and markers (`mint_4387_the_demand_s_game`'s `MINT
pair` rows), and no Great Lakes game at style 0 has been recorded, so a
style-1 donor serves a style-0 lobby. **`PLAYERS` does not move them
either** (item 4527): the 23 finished donors that seat three on a lobby
another donor seats two on (17 Great Lakes, 6 East Indies) carry equal
rules, run state, semaphore and markers, and no donor of the profile's
1,654 seats three on any of the fifth battery's lobbies (§12.3), so a
donor may differ in `PLAYERS` too
(`the_borrowed_sections_move_with_neither_team_style_nor_players`, over
`RON_RECGAME_DIR`; seen red with one side's run state bent). No donor
seats four or more, so the widening is measured from two seats to three
alone. It is read from the user's install at run time and never kept.

**Against the original's own recording of this lobby's game**
(`Playback - 2026.08.25 00'39'07 (Tue)`, 24,001 packages over frames
0..24000), our header parts on **two fields** and no others: the AI slot's
`tribe`, 11 here against 24 there (the lobby's "random", rolled after the
header is written; ours takes the block's resolved nation), and
`graphic_tick`, 0 against 2,801,115. Markers, version, seed, knobs, flags,
all 30 options, all eight slots, the tail, the run state, the semaphore, the
blob and the rules are equal. The packages differ by design: theirs is a
`camera` every frame, `player_speed` telemetry and frame 0's
`leader_options`; ours is the `leader_options` and one empty package. Seen
red: names written `Player <play + 2>` adds both slots to the list.

### 8.3 The original playing it

The click-free lane opens our file and plays it (`docs/RUNS.md`, run1560a,
run1560b and run1560): 301, 2,401 and 24,001 frames, exit 0, seed 12345,
map 14, the two players. The original's playback agrees with run53's live
game on every frame's RNG word (`rngcmp.py`: 301, 2,401 and 24,001 in
common, 0 differing).

`the_original_s_playback_of_our_file_walks_in_lockstep` walks the built
start (`docs/START.md` §7.4) against each take, the playback's trace for
draws and its dump for records (`PLAYBACK_WALKED`):

- run1560a, `end:MISC,UNITS=3`: draws agree on **300 of 300** frames,
  count and sequence; **299 records compared whole, none parted**.
- run1560b, `end:MISC`: draws agree on **2,400 of 2,400** frames; one record,
  none parted.
- run1560, `end:MISC`, the game's whole length: draws agree on **24,000 of
  24,000** frames; one record, none parted.

**The value diff: nothing parted.** No frame, no record, no field. Seen red:
the lobby's seed given to the built start as 12346 parts the draws at frame
0 and all 299 records (first: frame 1, `unlinked_units` 12).

The playback's pace at the trace-only recipe is **11.7 frames a second**
(run1560b's runner); the full-length take, run1560, ran 28.7 minutes.

### 8.4 Confidence, and what is not established

- **Diff-backed**: the wire layout of every field §8.1 re-encodes (50
  recordings byte for byte, five perturbations red); the re-walk's
  derivation; the constancy of what our file borrows; the original opening
  our file and playing it in agreement with its own live game (24,001 frames
  of RNG words) and with this crate's built walk (24,000 frames of draws, 300
  records).
- **Read once, from the decompile**: that a size-0 package is legal
  (`read_package`'s `size != 0` guard, 0x952d90) and that end of file one
  frame after the last package ends playback (the `feof` arm). Our closing
  package sits on frame 24,000: run1560 read it in step and quit by command
  on that frame, so the size-0 package is now seen and the end of file one
  frame on is not.
- **Not established**: whether a slot written with tribe 24, as the original
  writes it, replays the same (both takes used 11); what reads
  `graphic_tick`; whether a playback without `camera` and `player_speed`
  differs in anything beyond units over 300 frames and draws over 24,000. The Types stretch is still not derived, so our file
  needs a donor with the same options but for `TEAM_STYLE` and `PLAYERS`:
  one exists only for a lobby this install has recorded. Multiplayer, CtW and scenario recordings are out of
  scope.

## 9. Minted orders: one list of packages, both sides (item 4059, 2026-11-10)

§8's file holds no orders. This step puts three in it — a unit moved, a
unit trained, a building placed — issued by a script, and holds our walk
against the original's playback of it frame for frame. One list of
packages is the single source of both: encoded into the file the original
plays, and applied to our walk through the order door.

### 9.1 The script and the packages

`diff::minted::script()` is a list of `(frame, play, commands)` in the
wire's typed vocabulary (`crate::commands`, `docs/COMMANDS.md` §1–§4).
`recgame::minted_packages` turns it into packages as a single-player
recording stores them (`docs/COMMANDS.md` §2): frame 0's `leader_options`
(§8.2's, borrowed), then each order its own package on its own frame,
then §8's closing empty package on 24,000. Stamps run 1..5, `valid` is 1
on the first record only, the payloads plain. `no_input_packages` is now
`minted_packages` with no orders, so §8's file is unchanged (its test
still pins it against the twin). The file is
`~/ron-golden/recordings/item4059-minted-greatlakes-24000.rcx`, 59,280
bytes, SHA-256 `9de7f0ae…387c4d`, written by
`the_minted_packages_round_trip_and_name_the_built_start_s_objects`
under `RON_RCX_OUT`.

All three are slot 0's (the person's seat, tribe 4, `flags` 7) in run53's
lobby, read off our own built start:

| package | frame | `group` | command | why this one |
| --- | --- | --- | --- | --- |
| 2 | 30 | `[0]` (the Scout) | `move_to` (5880, 31896), `set_angle` 0, `angle` 0, `orders` 1 (`MOVE_TO`), `queued` 2 (`QUEUE_NEW`), `form`/`width` −1, `disembark` 0 | 2,400 units due east over open grass; the field values are the 2017 sample's right-click but `queued`, which is a plain click's |
| 3 | 60 | `[2000]` (the Village) | `queue_up` `type` 50 (Citizen), `num` 1 | the first thing a person trains; 25 food against 200 on hand |
| 4 | 90 | `[1]` (a Citizen at the Woodcutter) | `build` (3552, 29856) ×2, `type` 417 (Farm), `queued` 2 | a person's usual first building: 52 timber against 200, available to slot 0, and the nearest site to the capital our placement check (`blocked_site`) calls clear on a 192-unit grid; the citizen nearest it builds |

Each package is a `group` (3 + 2 bytes) and one command: 27, 14 and 30
bytes. `the_minted_packages_round_trip…` writes the file, parses it back,
re-writes it byte for byte, decodes each payload to the script's commands
and checks every object, type, price and the site against the built start.
Seen red: the stamps minted from 3 (`frame, play, valid, stamp` lists
part at package 2).

### 9.2 The pump

`input::Stream::pump` is the turn pump's stream: a `group` sets the
issuer's selection, and `move_to`, `queue_up` and `build` act on it
through `group_move_to`, `group_queue_up` and `group_build`, the entries
the golden chapters issue (`CommandPackage::process_all@0094c500` walking
each package to `process_group`, `process_move_to`, `process_queue_up`,
`process_build`). Every other command takes `Stream::new`'s arm, and
`Stream::new` is unchanged, so no capture replay's pin moved. The two
modes could be one only by moving run7's pins (its three `queue_up`s,
which `new` skips); that is not done here. SEAM: each entry pushes its
own group, so a package with two commands behind one `group` pushes two;
the minted script never does.

`the_pump_moves_our_walk_in_exactly_the_script_s_places` walks the built
start under the first k orders and the first k + 1, and compares the two
`Sim`s whole on the frame after order k + 1 (`PUMPED_PLACES`): on 31 they
part on the Scout `0/0` and the move's scratch (group pool, path search);
on 61 on the Village `0/2000`, the stock, slot 0's tech record and the
roster's muster; on 91 on the builder `0/1`, the site `0/2007`, the Woodcutter
`0/2001` the builder left, the city and farm lists, the footprint's tiles
and the stock. Nothing else. Comparing against the order-free walk instead
would not isolate them: the move moves the draws on 41, and every later
frame differs everywhere. Seen red: `Stream::new` in place of the pump
leaves 61 and 91 empty and moves the Scout without a group.

### 9.3 The takes

The click-free lane, §8.3's line with the minted file (`docs/RUNS.md`
run1561a, run1561b):

- **run1561a**, `end:MISC,UNITS=3,BUILDS=7`, 300 frames: 33.1 s, exit 0,
  301 frames, seed 12345.
- **run1561b**, `end:MISC`, 2,400 frames: 177.0 s, exit 0, 2,401 frames,
  12.0 frames a second.
- **run1561**, `end:MISC`, the game's whole length (the driver's take):
  1,707.9 s, exit 0, 24,001 frames, 11.0 frames a second.

Against run53's live game of the same lobby, the playback's word is
identical to frame 40 and parts on 41 (`rngcmp.py`: 46 identical of
2,401): the orders reach the original's draws.

`the_original_s_playback_of_the_minted_file_walks_in_lockstep` walks the
built start under the pump against each take, as §8.3's walk
(`MINTED_WALKED`):

- run1561a: draws agree on **300 of 300** frames, count and sequence;
  **299 records compared whole, none parted.**
- run1561b: draws agree on **2,400 of 2,400**; one record, none parted.
- run1561: draws agree on **24,000 of 24,000**, count and sequence; one
  record, none parted. Seen red first: the row pinned `(0, 0, 0, 0)` printed
  `(24000, 24000, 1, 0)`.

**The value diff: nothing parted.** And because two sides that both
ignored an order would agree, what each order did is read on both sides
and pinned (`minted_effects`), on 31, 61, 91 and 299: the Scout at
(3514, 31896) with its first order's target (5880, 31896), then at
(4534, 31896), (5554, 31896), and on 299 on its point with no order; the
Village's live queue `[50]` on 61 and 91 and empty by 299, a citizen
trained; the site `0/2007`, type 417, at (3456, 29760) from 91 (the
original snaps the drop point where `snap_center` does); the builder's
list `gather`, then `move` and `build` on 91, then `gather` on 299, farming
what it built. Seen red: the dump side's live queue read one slot long
(`[-1]` on 31, `[50, 0]` on 61).

**Made to fail** (`our_walk_with_one_order_withheld_parts_at_that_order_s_frame`,
`WITHHELD`): our walk with one order withheld, against run1561a's playback
of the whole script, parts first on that order's frame plus one:

| withheld | first record | what parted, ours against theirs | draws part |
| --- | --- | --- | --- |
| the move (30) | 31 | `0/0` position (3480, 31896) against (3514, 31896); its order list 0 long against 1 | 40 |
| the `queue_up` (60) | 61 | `0/2000` `queued` 0 against 1 | 158 |
| the `build` (90) | 91 | `0/1` position (4008, 28296) against (4023, 28316); orders 1 long against 2; the Market `0/2006`'s `city_down` −1 against 2007 | 296 |

The draws' frame is `Built::frame_sites`' (the frame before the
gamelog's); the move's 40 is the same frame the original's own word
parts from run53 on.

### 9.4 Confidence, and what is not established

- **Diff-backed**: the package layout a minted file uses (the original
  opens it and plays every package); that a recording's `group` +
  `move_to`, `queue_up` and `build` reach the same mechanics as the
  golden chapters' console lines, with the same arguments, on the frame
  `crate::input`'s convention names (each order's effect is on `f + 1` on
  both sides, and its absence parts there); 300 frames of records and
  2,400 of draws.
- **Read, not diffed**: that `valid` 0 on a later package is what the
  original's own single-player recordings hold (`docs/COMMANDS.md` §2,
  confirmed on the 2017 sample's 21,883 records; the playback does not
  read it as far as this take shows).
- **Not established**: a package with two commands behind one `group` (the
  pump's seam above); the `num = 0` reuse of a previous selection on
  playback; `queued` other than 2, `orders` other than 1, any
  `set_angle`; a `build` drag (`x2`/`y2` ≠ `x`/`y`); a site the original
  refuses; orders from a second slot; any command beyond the three. ~~The
  full-length take (run1561, the driver's) is not yet run: past 2,400
  frames only the draws of 4058's walk are known for this lobby.~~ Run:
  run1561 above, 24,000 of 24,000. Past frame 299 it compares draws and
  the closing record, not every frame's records.

## 10. Recording the live window

`client --record OUT.rcx --play` opens the lobby page. Starting the page
builds the recording header from the selected choices and a matching local
donor using §8.2. The client issues slot 0's opening preferences to the
runner at frame 0. The first interactive frame defaults to 1. An explicit
`--frame N` opens on the state reached after the first N ticks, each of
which is still part of the recording. The recording supports a human in
slot 0. The exercised lobby is Great Lakes, seed 12345, tribes 4 and 11.

The recorder receives each order that the window sends to the turn pump.
Events issued between simulation ticks share a package in input order.
When paused, the frame stands still and the next turn can have another
package with that frame. The 512-byte cap applies to the whole package.
A refused order restores the input state and does not reach the sim.
Closing the window applies a final waiting order when the game is
running. Closing during a game pause saves through the last completed
tick and reports any inputs still waiting for resume. The completed
file is published at an unused path. Cancelling on the lobby page before
Start writes no recording.

The local replay check reads the saved header, builds the start and pumps
the file's decoded commands. On every state from 0 through the closing
frame inclusive it compares all twelve lines to the original's first
checksum set tagged one frame earlier (setup is −1). A held wall, shot or
death whose bytes we cannot emit is reported as `unemitted`. Each line
reports its agreeing states and the values on the first differing state.
Exit 0 means all states agree, 1 means a measured difference, and 2 means
the comparison could not complete.

```sh
cargo run --release -p rondata --example score_recording -- /path/to/install OUT.rcx /path/to/rontrace.log
```

For a native capture archive, pass its map directory in the same position. This
works with a person's recording whose last package carries orders, and with a
game that ended before a minted file's closing package.

```sh
cargo run --release -p rondata --example score_recording -- /path/to/install GAME.rcx /path/to/run/map-14
```

The directory supplies `receipt.json`, `rontrace.log` and, when retained,
`rontrace.ranges`. The command checks the recording's SHA-256, the successful
lifecycle and map receipt, the native frame sequence and closing count, the
detach record and checksum summaries. It compares the twelve lines on every
observed state from setup. Output distinguishes a scheduled capture window,
the recording's closing package and a verified early ending. Exit 0 applies
only to the stated interval. An invalid receipt or incomplete trace exits 2.
When the take holds sums and lines part, the last line printed is `one take
for every parted line:` with the capture arguments that keep those lines'
ranges and no others (`playback::joined_take`, DECISIONS 148 §4).

The first differing state names every line tied there. Each line's first
mismatch gives both checksums. With ranges from that take it also gives the
first differing range in fold order, our record and field, the byte offset
and both values. Field decoders are shared with the packet tests. Range
headers must match the checksum sets, and the original bytes used in a
diagnosis must reproduce that line's Adler. If the bytes are unavailable,
the report gives the state, trace frame, mask and capture window for a
follow-up on the same recording. The range hook records from setup until
the endpoint or its size cap. `--log-window` selects the native text dump.

In the short window take, two actions on frame 30 moved scout 0 to
(3281, 33851) and queued type 50 at building 2000. The 59,249-byte file
rebuilds byte for byte. All twelve lines match at states 0 through 427.
Changing only the move's X coordinate to 3409 parts units, groups and guys
on state 31, and world on 34.

The longer take closed on 9475 (10 minutes 31 seconds by the game's clock).
The scout disbanded on 8407. At state 8408 the original holds two death
records for object 0/0, pieces 371 and 13043, both with first frame 8407
and animation 15. Their Adler is 826547225. Item 4362 first reported an
unemitted line here. Item 4363 matches both occupied records byte for byte.
The dog leaves at state 8567 and the scout at 9094. Deaths agrees on all
9,476 states, up from 8,790. On the item 4363 tree, all twelve lines agreed through state
9210. Units and groups first differed on 9211 and finished at 9,211 and
9,470 agreeing states respectively. Item 4364 matches both lines through
the close. All twelve agree on every one of the 9,476 states. The units
comparison now includes all six attack orders and the new group receives
its role and speeds at the original boundary (CHECKSUM §17, ARMY §12).
The regression checks both whole games, including the closing state.
RECGAME §8's rule section requires a donor. The long game's death came
from the disband command.


### 10.1 The first recording with both sprite pools occupied (item 4365)

Window A starts on Great Lakes, seed 12345. The human selected object 2 and
issued an order on object 1/27 at state 9500. Both engines replay the saved
file through state 11720. The ordinary shot pool is occupied on 9426–10273
and the casualty pool on 9745–10896. Both lines match the original through
clearance and closing. The leaders line gains 350 states from the corrected
sprite timing. Units first differ at 9416 and buildings at 9452, both in
their flags. The immutable baseline, recordings and capture receipts are
in the item 4365 entry in RUNS.


Item 4366 closes the two flag differences. On state 9416 the only differing
unit byte was object 1/29's first flag, 17 versus 145. On 9452 the only
differing building byte was object 0/2004's first flag, 7 versus 23.
The independent take includes both sides of each boundary. All twelve
lines now agree throughout the 11,721 states, including the closing one.


### 10.2 A second combat recording on another seed (item 4367)

The seed-123 game has a separate input package at 9000 and closes at
14437. On the pre-batch branch, eight lines agreed throughout. After
item 4366 only ammo differed, on 359 states. Sweeping piece 120 on the
first game's snapshot closed those differences (COMBAT §22.6). The
original and the sim now agree on twelve lines for all 14,438 states.
The raw test compares the complete ammo and casualty pools on all states,
including the 680 and 1,019 occupied states respectively. The old packet
score of 562/1,053 and the frame-91 score of 8/12 both hold.


### 10.3 Parity held out: eight lobbies named before the original plays them (item 4413)

DECISIONS 144 §2 names the games by a rule fixed before any take: rows 0,
20, 40, 60, 80, 100, 120 and 140 of the 161 lobbies in the order `Start:`
counts them (`lobby::lobbies_on_disk`, as
`every_lobby_s_start_built_from_its_choices` enumerates it), each at its own
seed, the human idle, 6,000 frames. `diff::held_out::HELD_OUT` writes each
row out (its first capture, the `GAMEINFO` lines, each `PLAYER` row and a
digest of the whole block), and
`the_held_out_eight_are_the_rows_the_rule_names` holds it against what that
order yields, so the set cannot drift silently. The commit that pinned the
table, before any recording, is the declaration; the journal quotes it.

Each game is recorded by the client from its own lobby page, played by the
original under the checksum hook as `run1700 + k`, and scored by
`checksum::playback`. A game's first score (states agreeing per line, the
first parting with both values) is pinned as found and never re-pinned in
place: a fix moves a second column.

As found (item 4413, 2026-11-19), all eight taken, none whole: 64 of 96
lines agree on every state. The queue's `Parity held out:` line is held to
`SCORES` by `the_handoff_s_parity_held_out_line_is_the_pinned_scores`
(DECISIONS 145 §1). Two kinds of game came out.

- **The Ancient starts** (rows 0, 20, 120, 140) play to 6000. Rows 0, 20
  and 120 agree on ten lines on all 6,001 states; units parts first on
  2990, 3694 and 2694 and agrees again later (5,435, 5,073 and 4,815
  states), and guys parts on 3 or 4 states. Row 140 (difficulty 4) parts
  units for good on 1568, guys on 7 states, and leaders on setup's set
  alone (6,000 of 6,001).
- **The later starts** (rows 40, 60, 80, 100: Medieval, Gunpowder,
  Enlightenment, Information, each with `STARTING_RESOURCES 7` and
  `DIFFICULTY 5`) the original ended itself on 4973, 4074, 3836 and 3330,
  with our walk not over. They part for good early: row 40 groups 711 and
  units 1858; row 60 leaders 969 and units 1055; row 80 world on setup's
  set and units 2239; row 100 world 134 and units 1611. Seven, five, eight
  and five lines are whole.

The second column, since: rows 40, 80 and 100 moved on the groups line
with the tranche's packet fixes (66 of 96 lines now). The original refused
none of the eight client-recorded files. `SCORES` holds every value;
`docs/RUNS.md` run1700 to run1707.

**What it has not established.** The take holds sums, so no parting names
its leaf until a ranges take of the same file (items 4435, 4436). Why the
original ended the four later starts is not read from theirs: our walk of
row 100 has the computer's army at the human's burning capital on 3000,
which suggests a conquest and is our side only. All eight seat two
players with no team (DECISIONS 145 §2).

**Row 80's world line, by leaf (item 4436, run1720).** A ranges take of
row 80's file to its end (run1704's line under `RON_RANGES`,
`RNG_MASK=0xfffu`) names the leaf of the world line that parted on every
state: on state 0, section 6's tile masks (range 3624), eleven tiles
where the original holds bit `0x4` (`tile::BEHIND`) and ours does not.
They are the third ring the Chinese Town `1/2000` hides (corner (217, 81),
7 × 7): Large City's `BEHIND_HEIGHT` 3, where ours marked the Small
City's 2. `Setup::build_cities@005ab910` converts a Chinese village to a
Large City between `Build::init` and its start, so `Wall::start` marks
three rings; ours converted at `Build::activate`'s head, after the start
(`docs/START.md` §7.2). Built in `Setup::chinese_start_city`: row 80's
world 0 → 3,837 of 3,837 (`SCORES[4].moved`), nine lines whole of twelve
before and ten now; units 2239 and builds 2376 remain. Diff-backed by
run1720's ranges; the other seven rows did not move.

**The Ancient starts' first units parting, by leaf** (item 4435, run1716,
diff-backed). run1716 is row 120's file played again under the ranges hook.
On 2694 the computer's transport barge `1/15` (type 320) is born holding a
move. Its `collide_frame..play` range reads `form` (`+0xaa`) as 0 on theirs
and −1 on ours. Every other byte of the units line agrees on that state.
`Unit::init@00612100:81`–`86` writes `form` 9 for the four citizen ids and 0
for every other type. This crate's `transport_boat` skipped that write and
kept `Unit::new`'s −1. It now writes it, and so does the dock gull's birth
beside it. Rows 0 and 20 parted on the same birth: barge `1/15` on 2990 and
barge `1/14` on 3694, each the first parting state. In the second column,
units moved 4,815 → 5,861 on row 120 (first 2694 → 3250), 5,435 → 5,998 on
row 0 (2990 → 3157) and 5,073 → 5,997 on row 20 (3694 → 4180). On rows 0
and 20 units now parts only where guys does. Row 140 (units on 1568) did
not move. `row_120_s_barge_is_born_with_form_zero_on_run1716_s_ranges`
holds every units range on 2694.

*Not established*: ~~what parts on 3250. On row 120, units and guys both part
first on the figure of the scout `1/0`, the barge's passenger. On 3250 both
sides put it at (19080, 39768). Theirs also moves the figure's `last_x`,
`last_y`, `last_z` and `last_angle` onto that point, and ours keeps the
values from before it moved. Guys agrees again on 3253. Units disagrees on
140 states, so something more parts it after 3253, and that cause is not read.~~
Answered by item 4439, below: the passenger's landing writes its figures'
`last`; the later units parting is on 5864.

**A passenger lands with its figures' `last` on its seat** (item 4439,
run1716, diff-backed on row 120). `Unit::come_out@00617c10:519` seats a
passenger with `set_new_location(…, 1, 1)`, which copies each figure's point
and `z` into `last_*` (`Guy::set_new_location@005d86f0:66`–`70`); its tail's
`set_angle(…, 1)` (`:629`) writes `last_angle` (`Guy::set_angle@005d9010:13`–`16`).
`land_tail` wrote none of it. On 3250, run1716's units range 159 (the scout
`1/0`) read `last_x`, `last_y` 19080, 39768 theirs, 30849, 41470 ours. Rows 0
and 20 became whole; the rest it moved is journal 4439's.
`row_120_s_scout_lands_with_its_figure_s_last_on_its_point_on_run1716_s_ranges`
holds every units and guys range on 3250.

*Not established*: ~~what parts row 120's units on 5864, to the end. run1716
names it: range 1139, record 35, `collide_frame..play`, byte 90,
`cavarch_o` ours −1, theirs −2 (unread).~~ Answered by item 4443, below: the
units line now folds the −2 this crate already held. Whether rows 0 and 20's partings
were this cause rests on our walk and on the sums agreeing; neither has a
ranges take. A unit leaving a building (`garrison.rs`'s `come_out_place`)
takes the same `set_new_location(…, 1, 1)` and is not given the write; no
score parts on it yet.

**Row 100's world line, by leaf (item 4441, run1721).** A ranges take of
row 100's file to its end (run1705's line under `RON_RANGES`,
`RNG_MASK=0xfffu`) gives the leaf of the world line that parted on state
134. It is section 6's tile masks (range 3624), two bytes: `tile::BAD_PATH`
sits on (211, 108) in ours and on (208, 111) in the original. Both are
blocked corners of the computer's University `1/2006` (`0x1a4`, 4 × 4,
corner (208, 108)), started on frame 133. Blocking a tile clears its own
`BAD_PATH` and sets it on its neighbours, so which corners keep the bit
depends on the walk order. `BuildType::mask_me@006312a0` walks the footprint
column by column and ours walked it row by row (`docs/PATHFINDER.md`, the
`tcost` paragraph). Built, row 100's world goes 134 → 2,618 of
3,331 (`SCORES[5].moved`). It parts next on 2618, the state ammo parts on:
range 1392, record 1368, `cell`, byte 20, ours 0x02, theirs 0xff. Units
still parts first, on 1611: range 1170, record 31, `cavarch_o` ours −1,
theirs −2, the leaf rows 80 and 120 part on. Diff-backed by run1721.

**Row 60's leaders, by leaf (item 4444, run1722).** On 969 who=1's seven
`resource_cap` words are ours 2400, theirs 3200: its republic, taken on 968,
reaches theirs' caps the next frame (`calc_resource_caps@006ce900` runs every
frame) and ours only at reassembly, on 976 (`docs/ECONOMY.md`). Read every
frame, leaders is 4,075 of 4,075 (`SCORES[3].moved`); the row parts first on
1055, `cavarch_o`. Diff-backed by run1722.

**The units line folds `cavarch_o` −2** (item 4443, diff-backed on rows 80,
100 and 120). `Unit::cavarch_fight@005ff4b0:34` writes `0xfffe` after two
empty searches. The sim held it (`cavarch_idle`); the units range wrote −1.
run1716, 1720 and 1721 name the leaf (ours −1, theirs −2): row 120 on
5864, range 1139 (MLRS `1/35`); row 80 on 2239, range 3003; row 100 on 1611,
range 1170. Row 120 is now **whole**; units also moves on rows 40, 60, 80,
100 and 140 (journal 4443); row 60 parts first on 1561. With 4444: 3 of 8, 74 of 96 lines.

~~*Not established*: rows 80 and 100 next part units on `des_angle` (`+0x58`):
run1720 on 2367, range 2407; run1721 on 1793, range 1581. Unread.~~
Answered by item 4448.

**Row 60, whole** (4473, 4476, 4480): a push zeroes the empty last
slot's `ox`/`oy` (GROUPS §36.2); a dropped cavarch target keeps its uid (AI
§134.3); a `DETECT` building lights `seen3` (VISION §4).

**Rows 100, 10 and 130's deaths, whole** (4478, run1721): the nuke strikes in
`Objects::find_units@0065a620`'s order (PRODUCTION, the nuke).

**Row 100's units, whole** (4491, run1721 2758): the units line walks a
strafe's own ranges (CHECKSUM §18). ~~Ammo 3210 is the row's last parting.~~
Moved by 4495.

**Row 100's ammo, 3210 → 3312** (4495, run1721 3210): the Jet Fighter's
guided round walks `calc_from_dir`'s spline in the sizes of the recycled
`Spline` the nuke left (CHECKSUM §18.1). ~~Ammo 3312 is the row's last
parting.~~ Moved by 4497.

**Row 100, whole** (4497, run1721 3312): a bomb's round carries the
`accuracy` its pool slot's last round left, the Strategic Bomber `1/83`'s
260 in slot 2 (CHECKSUM §18.2).

## 11. Parity held out, second: eight more lobbies named before the original plays them (item 4462)

DECISIONS 145 §2, 2026-11-20. Rows 10, 30, 50, 70, 90, 110, 130 and 150 of the same order, declared
in `HELD_OUT_SECOND` (`d6adb6cf7`) before any file was recorded, each
recorded idle by the client's page to 6000 and played by the original as
`run1730 + k` under the checksum hook. The scores are pinned once in
`SCORES_SECOND`. Its found column replaces the first's as the estimate:
**1 of 8 whole, 65 of 96 lines** (the first's was 0 of 8, 64 of 96).

| row | run | lobby | states | whole | lowest first parting |
|---|---|---|---|---|---|
| 10 | run1730 | Lakota peace, ages 8..7, `RUSH_RULES 10` | 6,001 | 4 | builds and world 537 |
| 30 | run1731 | East Indies, Industrial | 4,548, ended on 4547 | 8 | world 1662 |
| 50 | run1732 | Great Sahara, Gunpowder | 4,455, ended on 4454 | 11 | groups 3352 |
| 70 | run1733 | Great Lakes, Medieval | 5,141, ended on 5140 | 11 | world 5054 |
| 90 | run1734 | East Indies, Ancient, difficulty 0 | 6,001 | 10 | units and guys 4021, one state each |
| 110 | run1735 | East Indies melee, Information, three seats | 6,001 | 5 | groups 1608 |
| 130 | run1736 | East Indies all-tech, ages 8..7 | 4,731, ended on 4730 | 4 | builds and world 553 |
| 150 | run1737 | Great Sahara, difficulty 4 | 6,001 | **12** | whole |


**Row 90 is whole** (item 4470, `SCORES_SECOND[4].moved`). run1738, the
ranges take of its file to 6000, names the leaf on 4021: units range 498
and guys range 150, the computer's transport barge (record 14, type 320)
born that tick, its figure's `last_angle` ours 1431655765
(`Angle::INITIAL`), theirs −609026048, its `angle` agreeing.
`SpellType::cast_transport@00670db0` turns the boat with
`Unit::set_angle(boat, caster +0x50, _, 1)` (`671027`–`67104f` push the
snap), and `Guy::set_angle@005d9010`'s snap writes `angle` and
`last_angle` both; the view had written the movement's facing alone.
The same fix moved the guys lines of rows 10, 30, 110 and 130 and the
first battery's row 40 (`docs/journal/2026-11-20-item-4470.md`); the
battery reads 4 of 8 whole, 76 of 96 lines, with 4480's row 70.
The original refused none of the eight files. Row 110 is the only row of
the sixteen with three seats, and it is played as its lobby stands.
`docs/RUNS.md` run1730 to run1737 holds every value; the pins hold both
sides of every first parting.

**Reach** (DECISIONS 147 §2). A game or a line is whole or it is not, so
the two counts read the batteries alike. The states a game runs before its
first parting, summed over a battery, of all its states, read 27 % on the
first as found (11,192 of 40,221) and 53 % on this one (22,788 of 42,879).
Both parsed lines carry it (`held_out::queue_line`). The third battery is
rows 5, 25, 45, 65, 85, 105, 125 and 145, named before any take (item
4487), taken when this one reads 4 of 8 now.

**Rows 10 and 130's builds are whole** (item 4483, `SCORES_SECOND[0]` and
`[6].moved`). run1739, the ranges take of row 10's file to 6000, names the
leaf on 537: builds range 746, the computer's SAM `1/2012` (`TypeIndex`
`0x20d`) finishing that tick, `job_counter` ours 0, theirs `0x40000000`,
every other byte of its record agreeing. `Wall::activate@0063e4b0:54`
zeroes both counters, then asks the type `is(0x20b, 0)` (`63e54d`–`63e57f`
push `0` and `0x20b` before the type's `+0x60`), AIRDEFENSE's line, and
sets `job_counter` to `0x40000000`; no later frame of the take changes it.
This crate's `Sim::activate` zeroed it alone. The same fix moved row 110's
builds 4221 → 5577 and the third battery's row 105 builds whole. The world
parting the two rows shared was 4480's `seen3`, and the leaders partings
behind it (1576, 1376) were 4499's `reg_gatherers`. Their next partings are
units, 1747 and 2354: on row 10 units range 1192, record 37, `inside_down`
ours −1, theirs 38.

*Not established*: ~~row 130's units parting is not read~~ (4508, below); the AIRDEFENSE counter's readers in the original are not listed,
since no take's value of it ever moves.

**Row 10's world is whole** (item 4504, `SCORES_SECOND[0].moved`, world
5,995 → 6,001). run1739's leaf on 2365: world range 6920, record 3039
(cell (39, 50) of the 60-wide map), `block.present` ours 1, theirs 0.
who=1's `1/43` (TI 104) steps (632, 813) → (633, 814) inside a freed
cell, and `CollCheck::move_unit@00682ad0`'s one-block form returns on the
null block, where this crate allocated it (`docs/COLLISION.md` §2.4).

**Row 10's leaders are whole** (item 4516, 5,801 → 6,001): a casting
citizen is an `xport` peasant, `docs/AI.md` §58.7.

**Row 10's units and guys are whole** (item 4522): two Attack
Helicopters set apart keep `dest_angle`, `docs/PRODUCTION.md`, "The
push's angle". **Row 10 is whole** (item 4523, ammo 5,962 → 6,001): a
falling aircraft's round, `docs/CHECKSUM.md` §18.4.

**Row 130 is whole**, from run1754's leaves: the bomber `1/158` searches
round itself (item 4508, `find_new_bomber_target@005eb960`), 4522's
helicopters, the Kremlin's spy-due `leader_flags & 0x10000000` (item
4520, leaders range 128), and its spy `1/169`'s `unit_masks2 & 0x400`
(item 4524, `Build::process@0061edf0:320`; units range 4183 on 4583).

### 11.1 Parity held out, third: rows 5 to 145 (item 4487)

DECISIONS 147 §2, 2026-11-21. Rows 5, 25, 45, 65, 85, 105, 125 and 145 of the same order, declared in
`HELD_OUT_THIRD` (`4a1b901a7`) before any file was recorded, each recorded idle by the client's page to 6000 and
played by the original as `run1743 + k` under the checksum hook. Each is the human against one computer, as its
lobby stands. The scores are pinned once in `SCORES_THIRD`, on `1f300eaa0`'s tree; this found column is the
estimate now: **5 of 8 whole, 90 of 96 lines, reach 88 %** (38,662 of 43,758 states), where the second read 1 of
8, 65 of 96 and 53 %, and the first 0 of 8, 64 of 96 and 27 %.

| row | run | lobby | states | whole | first partings (state) |
|---|---|---|---|---|---|
| 5 | run1743 | Great Lakes, difficulty 4 | 6,001 | 11 | ammo 4805 |
| 25 | run1744 | Himalayas, difficulty 0 | 6,001 | **12** | whole |
| 45 | run1745 | Great Sahara, Classical | 6,001 | **12** | whole |
| 65 | run1746 | Great Sahara, Enlightenment | 4,046, ended on 4045 | **12** | whole |
| 85 | run1747 | Great Lakes, Industrial | 4,472, ended on 4471 | 11 | groups 3831 |
| 105 | run1748 | East Indies, Modern | 5,235, ended on 5234 | 8 | leaders 1976, units 2983, groups 4355, builds 5162 |
| 125 | run1749 | Islands, toughest | 6,001 | **12** | whole |
| 145 | run1750 | Great Sahara, difficulty 3 | 6,001 | **12** | whole |

The merged tree after the take (`cc9ffc8b3`: 4478, 4489, 4490) moves none of the eight. The original refused no
file. In the three games it ended itself, our walk holds no `game_over` by that tick (`over: None`). `docs/RUNS.md`
run1743 to run1750 holds every value, and the pins hold both sides of every first parting.

**Row 85's leaf (item 4493, run1751, the ranges take of run1747's file).** State 3831, groups range 86, record 68
(who=1's slot 4, army 3), `id..march`: `num` ours 4, theirs 3; `list` ours `[117, 119, 120, 121]`, theirs `[117,
119, 120]`. On 3830 army 3 moves, and `Form::categorize`'s `Group::sort` kills the follower-first squad `1/119..1/121`
(only `1/121` listed, so only its `+0x80` is cleared) and re-adds it whole. A follower's `GROUP_MOVE` then refreshes
its order, and `Group::refresh_group_order@00713a50` opens with `normalize`, which prunes the member whose `+0x80`
no longer names the group. This crate's refresh ran only the speed tail; it now prunes a seated group
(`Sim::group_refresh_order`, `docs/GROUPS.md` §23.4). Row 85 is whole: **6 of 8, 91 of 96 lines**. No other row of
the three batteries moved. `row_85_s_refresh_prunes_the_member_its_sort_unpointed_on_run1751_s_ranges` holds every
groups range on 3830 and 3831.

**Row 105's leaf (item 4499, run1752, the ranges take of run1748's file).** State 1976, leaders range 129, record
1 (who=1), `who..ally_mask`, byte 4440: `reg_gatherers[1]` (`+0x1160`) ours 0, theirs 1. That is the only leaf
parting on 1976 or 1977. 1976 is who=1's first census sweep: theirs counts four region gatherers against
`gatherers` 3 (3 in region 11, 1 in region 1), and region 1 holds no peasant. One of who=1's citizens rides a sea
transport whose move ends in **sea region 65**. `Leader::plan_strategy`'s rider arm counts it with `incw
0x10de(%ebx,%esi,2)` at `6baa2f`, and the only test before that is the sign (`js` at `6baa29`). So
`reg_xport_peasants[65]` writes `0x10de + 130 = 0x1160`, which is the next `ushort[64]`'s second slot.
`Unit::think_civilian_transport@005f40d0` reads and writes the same array with no test either. This crate counted
the rider in its own slot for region 65, and the image drops a number past 63. It now lands the write where the
original's does (`Sim::xport_alias`, `docs/AI.md` §58). Row 105 is not whole: leaders 4,656 → 5,056, first 1976 →
3583, and the row's first parting 1976 → 2983 (units). The same alias moved four rows of the second battery on leaders:
row 10 3,976 → 5,801 (first 1576 → 3976), **row 30 3,748 → 4,548, whole**, row 110 4,995 → 5,995 (first 3176 →
5344), row 130 1,976 → 4,721 (first 1376 → 4573). The second battery's lines go 78 → 79, and the third's reach 89 % →
92 %. Nine East Indies pair packets have their leaders line whole (the `reg_gatherers` leaf they parted on), so
packets' lines go 899 → 908 on the merged tree. `row_105_s_rider_to_sea_region_65_lands_in_reg_gatherers_on_run1752_s_ranges` holds every leaders range on
1975 and 1976.

**Row 105's units leaf (item 4502, run1752).** State 2983, units range 655, record 17 (who=1's `1/17`),
`collide_frame..play` byte 38: `unit_masks2` ours 0, theirs 65536. On 2983 `1/17` turns from `TypeIndex` 0x163
to 0x165. `Unit::set_type@00612fa0:208`–`219`, like `Unit::init@00612100:253`–`264`, raises `unit_masks2 &
0x10000` on a type that `is(0x165, 1)` and clears it on any other, and a unit left holding the bit calls
`update_ceo_position` from nowhere to its own tile. This crate wrote neither the bit nor that call, and decided the
CEO by its type. Both now follow the original (`Sim::init_ceo_mask`; the set_type adapter's `is_ceo`), and the
crossing test reads the bit. Units go 2,983 → 3,359, first 2983 → 3359 (range 3707, record 127,
`myhits..launch_frames` byte 8, `inside_down` ours −1, theirs 128). That is the row's first parting too. Leaders
(3583) and groups (4355) do not move. `row_105_s_ceo_carries_the_ceo_bit_from_its_set_type_on_run1752_s_ranges` holds
every units range on 2982 and 2983.

**Row 105's Freighter (item 4511, run1752).** State 3359, units range 3707, record 127 (who=1's `1/127`),
`myhits..launch_frames` byte 8: `inside_down` ours −1, theirs 128. On 3359 who=1's Transport Freighter `1/135` (TI
322) is born and three German Infantry (TI 103) go aboard, `1/127`, `1/128`, `1/129`. `Object::insert_inside@
00647e90` appends at the bottom of the host's chain, so theirs links `1/135 → 1/127 → 1/128 → 1/129` by
`inside_down` and back by `inside_up`. This crate kept no chain for a unit host: the host named its last rider by
slot, and each rider named the host above and nothing below. It now keeps the order of entry (`Unit::boarded`,
`Sim::cargo_chain`; `docs/TRANSPORT.md` §6.5). Units go 3,359 → 5,235, whole. The row's first parting goes 3359 →
3583 (leaders); groups (4355) does not move. The same chain moves five rows of the other two batteries on units:
**row 40 2,487 → 4,974, whole** (the first battery 7 → 8 of 8); row 10 3,914 → 5,664 (first 1747 → 3771); row 30
2,334 → 4,114 (first 2334 → 3184); row 110 5,017 → 5,297 (first 2184 → 4492); row 130 3,034 → 4,422 (first 2354 →
3411). Packets' lines go 924 → 947 (28 rows, every one on units: 23 whole, 5 on fewer ranges).
`row_105_s_freighter_chains_its_riders_in_the_order_they_boarded_on_run1752_s_ranges` holds every units range on 3358
and 3359.

**Row 5's leaf (item 4507, run1753, the ammo take of run1743's file, `RNG_MASK=0x8u`).** State 4805, ammo range 1,
record 0 (slot 0), `rolling..start_roll_angle`: bytes 7 and 11, `launch.x, launch.y`, ours (4944, 30710), theirs
(4945, 30709), and bytes 41 and 84, the aim's `angle` and `dx`, which follow from the launch. It is the only parted
range; the stone beside it (slot 1) agrees. The round is who=1's Slinger `1/22` (type 82, piece 32), at (5064, 30744)
facing −1,097,334,784, loosing `CHAR_ATTACK2` at frame 22 on a building. Its socket is `launch::BAYS`' piece-32 row,
fitted from run17's twenty headings (`docs/COMBAT.md` §70.2). The original's own `get_position` on run1579's packet,
swept over all 361 whole degrees, says the three rows held 205, 324 and 345 of them, and this heading is one of the
misses. The rows are now the sweep's centroids, 361 of 361 each, and equal piece 384's (item 4277) to the digit. Row
5 is whole: **7 of 8, 93 of 96 lines, reach 95 %** as the third battery's moved column, on the tree merged with
4502. No other row of the three batteries moved; the third pair's window `1/22` on 5633 had pinned the same three
launch differences, and they close. `row_5_s_slinger_stone_leaves_from_the_original_s_socket_on_run1753_s_ranges` holds every ammo range on 4804
to 4806.

**Row 105 whole (item 4512, run1752).** *Leaders, 3583*: range 191, record 1, who=1's `oil_patches` `size` ours 10,
theirs 20. Both sides prune the list from twelve patches to ten (arm 4.2), length and elements agreeing;
`SimpleArray<int>::remove@00462e70` only shortens the length, so the capacity stays 20. The leader now keeps each
shrinking array's capacity (`Leader::oil_patches_size`, `mil_trainers_size`). Leaders 5,056 → 5,235, whole.
*Groups, 4355*: range 204, record 84 (who=1's slot 20), `num` ours 49, theirs 50. `think_scout` pushes `1/94`
(type 200, its `+0x80` naming slot 9) onto slot 22; `Groups::push_group@0070f9e0`'s second walk kills it from slot
9 alone, so slot 20 keeps listing it. This crate also swept every seat for the leaving squads (since item 465); the
sweep is gone (`docs/GROUPS.md` §30.3). Groups 5,217 → 5,235, whole, and **row 105 is whole**: the third battery 8
of 8 on the tree merged with 4507. The same fix closes golden chapter 41's three `group` rows (902, 968).
`row_105_s_pruned_oil_list_keeps_its_capacity_on_run1752_s_ranges` holds every leaders range on 3582 and 3583,
`row_105_s_pushed_unit_stays_listed_by_the_slot_it_left_on_run1752_s_ranges` every groups range on 4354 and 4355.

**At the pass of 2026-11-21** (DECISIONS 148 §2, §3). The three found columns read 0, 1 and 5 of 8 whole, 64, 65
and 90 of 96 lines, and 27, 53 and 88 % by reach. Every game of the twenty-four has an idle human. The fourth
battery is rows 15, 35, 55, 75, 95, 115, 135 and 155, named before any take (item 4510), taken when this one reads
7 of 8 now. Scored at the tip, row 105 parts three lines, each with its leaf on run1752's disk: units on 2983
(below), leaders on 3583 (below) and groups on 4355 (range 204, record 84, `num` ours 49, theirs 50). Row 10 parts
five, each on run1739's disk: units 1747 (`inside_down`), world 2365 (range 6920, record 3039, `block.present` ours
1, theirs 0), leaders 3976 (range 129, record 1, `who..ally_mask` byte 2488, ours 2, theirs 3), guys 4880 (range
1030, record 140, `last_angle` ours 610064529, theirs 555024384) and ammo 5178 (ours emits no line, theirs holds 3
ranges). Rows 40, 5, 130, 30 and 110 hold sums; each one's take keeps the lines it parts on (`0x1u`, `0x8u`,
`0x81u`, `0x801u`, `0x88bu`).

*Not established*: ~~row 5 names no leaf, since its take holds sums~~ (item 4507, above). ~~Row 105's next partings are units on 3359 (range
3707, record 127, `inside_down` ours −1, theirs 128; item 4502)~~ (whole, item 4511, above). ~~Row 105's next partings
are leaders on 3583 (range 191, record 1, `oil_patches` ours 10, theirs 20) and groups on 4355 (range 204, record 84,
`num` ours 49, theirs 50), both on run1752's disk.~~ (whole, item 4512, above.) `go_inside` and `come_out` still do not call
`update_ceo_position`. The alias is built for the rider arm and the transport's two accesses. The
kinds-8/0xe arm's unbounded write (`6bae11`) is still a seam here, and no capture has entered it.

## 12. Parity held out, fourth: eight more lobbies named before the original plays them (item 4510)

DECISIONS 148 §2, 2026-11-21. Rows 15, 35, 55, 75, 95, 115, 135 and 155 of the same order, declared in
`HELD_OUT_FOURTH` (`64617fe79`, 11:32 −05:00) before any file was recorded (the eight were written between 11:38
and 11:39), each recorded idle by the client's page to 6000 and played by the original as `run1760 + k` under the
checksum hook alone, with no ranges. Each is the human against one computer, as its lobby stands. The scores are
pinned once in `SCORES_FOURTH`, on `691ff3eba`'s tree; this found column is the estimate now: **5 of 8 whole, 90
of 96 lines, reach 91 %** (43,550 of 47,349 states), where the third read 5 of 8, 90 of 96 and 88 %, the second 1
of 8, 65 of 96 and 53 %, and the first 0 of 8, 64 of 96 and 27 %.

| row | run | lobby | states | whole | first partings (state) |
|---|---|---|---|---|---|
| 15 | run1760 | Southwest Mesa, difficulty 0 | 6,001 | **12** | whole |
| 35 | run1761 | Great Sahara, Classical | 6,001 | **12** | whole |
| 55 | run1762 | East Indies, Medieval | 6,001 | 10 → **12** | ~~units 4275~~ (item 4518), ~~groups 5576~~ (item 4531) |
| 75 | run1763 | Great Lakes, Gunpowder | 5,342, ended on 5341 | 11 | ammo 4004 |
| 95 | run1764 | East Indies, Information | 6,001 | 9 → **12** | ~~leaders 5266~~ (4519, §12.4), ~~units, guys 5909~~ (4524, 4536, §12.5) |
| 115 | run1765 | Southwest Mesa, difficulty 2 | 6,001 | **12** | whole |
| 135 | run1766 | Great Sahara, difficulty 2 | 6,001 | **12** | whole |
| 155 | run1767 | Great Sahara, difficulty 2 | 6,001 | **12** | whole |

The original refused no file. In row 75, the one game it ended itself, our walk holds no `game_over` by that tick
(`over: None`). `docs/RUNS.md` run1760 to run1767 holds every value, and the pins hold both sides of every first
parting.

*Not established*: ~~no row names a leaf, since each take holds sums only.~~ ~~Row 75 parts first by state (ammo on
4004, ours 3143699460, theirs 3217755213), then~~ (row 75 named and whole, item 4517, below) ~~row 55 parts first
(units on 4275, ours 1914015254, theirs 708677162), then~~ (row 55's units named and closed, item 4518, §12.1) ~~row 95 (leaders on 5266, ours 871988666, theirs 426802620)~~ (named and closed, item 4519, §12.4).
Each needs a ranges take of its file, keeping the lines it parts on (DECISIONS 148 §4), before its leaf can be named.

**Row 75's leaf (item 4517, run1768, the ammo take of run1763's file, `RNG_MASK=0x8u`).** State 4004, ammo range 1,
record 0 (slot 0), `rolling..start_roll_angle`: byte 7, `launch.x`, ours 3922, theirs 3923, and bytes 41, 83 and 84,
the aim's `angle` and `dx`, which follow from the launch. It is the game's first round: who=1's Crossbowmen `1/163`
(TI 172, land, piece 122) at (3960, 34440) facing −475,070,464, loosing `CHAR_ATTACK3` at frame 6 at the human's unit.
Its socket was item 3347's planar row in `launch::MEASURED`, one bolt off run982. The original's own `get_position`
on run1579's packet, swept over all 361 whole degrees for each of piece 122's three keys, admits one `launch::Bay`
region a key (133, 688 and 417 lattice points); the rows are their centroids, (11,399, 84,486), (14,863, 64,484) and
(15,287, 75,439), heights 166, 134 and 161, and they are piece 1178's rows (item 3712, the Asian style) over its scale
1.04 to the unit, an independent fit agreeing. run982's three bolts still land on their dump's points. Row 75 is
whole: **ammo 4,682 → 5,342, twelve lines to the original's end**, and the fourth battery's moved column reads 6 of 8.
`row_75_s_crossbow_bolt_leaves_from_the_original_s_socket_on_run1768_s_ranges` holds every ammo range on 4003 to
4013, and `data::launch::tests::the_original_s_release_agrees_at_every_heading_for_the_crossbowmen` the sweeps
(`RON_BAY_SWEEPS_4517`, outside git).

### 12.1 Row 55: a boat is born through `Unit::update_hits` (item 4518)

**How it was established.** run1769 is run1762's file again, with the ranges of units and groups kept to 6000
(`RNG_MASK=0x21u`, the scorer's line). Its sums are run1762's. On 4275 the units line grows from 6,052 ranges to
6,074 on both sides, and one range parts: **range 3947, record 142, `myhits..launch_frames` byte 0, `myhits`
(`+0x20`, the unit's max hit points): ours 172, theirs 192.** The unit is who=1's **Merchant Fleet `1/142`, `TI` 318**,
a sea unit, born on 4275. Its caster is the Armed Caravan `1/64` (TI 60): on 4274 it holds a cast order, and on 4275 it
is inside the fleet. Both caravans read 145/145 on both sides. Every other field of the fleet agrees, its damage
(`+0x24`) at 0 included.

**The direction: ours dropped a term.** Who=1 is tribe 22, the Dutch (`has_tribe_bonus(0x16)`).
`Unit::init@00612100:580` calls vslot `+0x15c`, which `Unit::vftable` names `Unit::update_hits@0060e930`, for every
unit `Objects::init_unit` makes. A boat is one of them. That function's Dutch arm (`0060eab4`..`0060eb98`) adds
`ages × DUTCH_HP_BONUS` to a merchant kind, a caravan (`UnitTypeData::is_caravan`, `unit_flags2 & 8`) or a supply
unit. Ours' ordinary birth took `Sim::unit_hits` (`units/store.rs`), which carries that arm. `transport_boat`
(`ai/transport.rs`) took the type's raw `HITS`, 172. It now takes `unit_hits`, and the fleet is 192.

**Diff-backed**: `row_55_s_dutch_merchant_fleet_is_born_with_its_update_hits_on_run1769_s_ranges` compares every
units range on 4274 and 4275, byte for byte, and names theirs' `1/142` by `TI` 318 and `myhits` 192.
**Moved**: units 5,833 → 6,001, whole; row 55 has eleven lines whole. ~~Groups part as found, on 5576: range 191,
record 83, `id..march` byte 8, `num` ours 42, theirs 43.~~ (closed, item 4531, §12.2)

*Not established*: which term of the arm the fleet enters by. Ours' fleet type carries `unit_flags2 & 8`, so it
enters as a caravan. This follows from ours' total, not from a read of theirs' predicate. No capture has a Marines
caster's boat born to a Dutch or Nubian owner, where `marines` re-runs `update_hits` after this birth.

### 12.2 Row 55: a group's list holds object numbers, so a reused number rejoins it (item 4531)

**How it was established.** On run1769's ranges, the groups line agrees through 5575 and parts on 5576 in one slot:
**range 191, record 83 (who=1's slot 19, army 5's group), `id..march` byte 8, `num`: ours 42, theirs 43.** Every other
field of the record agrees. The list differs by one entry: theirs keeps the short 44 at index 31, and ours dropped it.
The units line agrees on both states. `RON_DEBUG_UNIT=1/44` names the units. On 5575 no live `1/44` exists: the member
at index 31 is dead on both sides, and neither had pruned it. On 5576 **Elite Javelineers `1/44` (TI 84)** are born
into that number, at (35352, 41208), with no army.

**The direction: ours dropped a member theirs kept.** A probe's backtrace on 5576 (one env guard, two commits,
`6e0616103` and `1b2117753`, reverted whole in `a9b1ecf0d`) names ours' only prune: `build_train → come_out →
push_group → get_open_slot → get_num` on slot 83. The javelineers leave their building, and the open-slot search runs
`get_num` over who=1's slots. `Group::get_num@00714700`'s large arm (43 members, not under four) drops a member only
when `objects[who][short] +8 & 1` is clear. The list at `+0x8cc` is shorts, so on 5576 short 44 names the newborn, which
is active, and theirs keeps it. Ours' list held the dead occupant's slot in `Sim::units`, because a reused number takes
a new slot. So ours read the member as dead and dropped it.

**The build.** `Sim::add_unit` (`Unit::init`) now calls `seat_lists_take_the_number`. On a birth, every army's list and
every pool slot of the owner that still names a dead occupant of the new unit's number names the new unit instead.
`add_unit` already carries `path_recursion` and the chain words across a reused number the same way (§37.2 of
`docs/GROUPS.md`).

**Diff-backed**: `row_55_s_army_keeps_the_member_whose_number_a_newborn_took_on_run1769_s_ranges` compares every
groups range on 5575 and 5576, byte for byte. It names record 83's `num` 43 and short 44 at index 31, and ours' member
31: dead on 5575, the Elite Javelineers on 5576. **Mutation** (`tools/mutate.py`, the call removed): held, exit 101,
the one failed test that one, at range 191 on 5576. **Moved**: groups 5,989 → 6,001, whole; **row 55 has twelve lines
whole**, and the fourth battery is 7 of 8 whole.

*Not established*: the rebind covers the seated lists alone (army lists and pool slots). Other lists the original keeps
as shorts read the same way, but none is rebound here and no diff reaches them: a hotkey group, a building's garrison,
and a stack group's selection. Nothing has been read on what theirs' army 5 does with the javelineers after 5576. A
whole `normalize` would drop them, since their `+0x80` is not 83, but its first call on group 83 has not been located on
either side. Both sides' groups and units lines agree to 6000.

### 12.3 Parity held out, fifth: rows 3 to 143, three seats and a team (item 4527)

DECISIONS 149 §3, 150 §2 and §3, 2026-11-21. Rows 3, 23, 43, 63, 83, 103, 123 and 143 of the same order, each with
one seat and one team of two added, declared in `HELD_OUT_FIFTH` (`aece988eb`, 21:55 −05:00) before any file was
recorded (written 21:59 to 22:01). **The page sets both**: `Event::AddSeat` adds a computer
seat up to eight, `Event::Team` sets a seat's team from 0 to 8 (8 none), and the page's own rows make each by keys
(`docs/LOBBY.md` §2, §3). Teams 0 to 3 ally (`LeaderData::is_team`), and these rows' `TEAM_STYLE 1` locks them.

**Three seats, because the map holds three.** Every row is `MAP_SIZE 2`, which `rules.xml`'s `mapsizes` names
"Small (2/3-player)"; a fourth seat would move `MAP_SIZE` and leave the row's lobby. Each row keeps its human (seat 0,
tribe 4) and its computer (seat 1) and adds a computer of nation 0, 1, 2, 3, 5, 6, 7 or 8 (`HELD_OUT_FIFTH_SEATS`),
named by its nation's first leader. The team alternates: the human and the added computer are team 0 on rows 3, 43,
83 and 123, against the row's computer; the two computers are team 1 on rows 23, 63, 103 and 143, against the human.
The declaration's columns are the page's block after the row's own events and the additions, each through
`Choices::apply`.

**The recorder needed a widening.** No finished donor seats three on any of the eight lobbies, so none could be
written until `donor_serves` let a donor differ in `PLAYERS` (§8.2). Each file is `client --record <file> --headless
--page-of <capture> --seat <tribe> --team <a>=<t> --team <b>=<t> --frame 6000`; `--seat` and `--team` press the
page's keys after the capture's lobby (`Page::press`). `the_fifth_s_files_carry_the_declared_seats_and_team` reads
each file back against the declaration's rows.

Each was played by the original as `run1773 + k` under the checksum hook alone, with no ranges, between 22:05 and
23:07 −05:00, and pinned once in `SCORES_FIFTH` on `87ad488ce`'s tree. That
found column is the estimate: **1 of 8 whole, 30 of 96 lines, reach 48 %** (21,315 of 43,524 states), where the
fourth read 5 of 8, 90 of 96 and 91 % on two seats and no team. The original seated three and the team on every take.

| row | run | lobby | team | states | whole | lowest partings (state) |
|---|---|---|---|---|---|---|
| 3 | run1773 | Great Sahara, difficulty 3 | human + added | 6,001 | **12** | ~~3857, 4777, world 4990~~ whole (MAPGEN §35) |
| 23 | run1774 | Atlantic Sea Power | the computers | 6,001 | 10 | **leaders 0**, units 5695 |
| 43 | run1775 | Great Sahara, Ancient | human + added | 6,001 | 1 | ~~3192, 3223, 3299, 3442, 3833, 3899~~ 3995 |
| 63 | run1776 | East Indies, Enlightenment | the computers | 4,223, to 4222 | **12** | ~~1582, 1789, 3683, 3986~~ whole |
| 83 | run1777 | Great Lakes, Industrial | human + added | 6,001 | 1 | ~~1956, 2811, 3041, 3359, 3645, 3656, 3662~~ units 3706 |
| 103 | run1778 | East Indies, Modern | the computers | 3,295, to 3294 | **12** | ~~1750, 1757, 1790, 2557 (GROUPS §3.4), ammo 2860, world 2983~~ whole (COLLISION §2) |
| 123 | run1779 | Great Lakes, Gunpowder | human + added | 6,001 | 3 | ~~builds, groups 2977~~ whole (below) |
| 143 | run1780 | Great Sahara, difficulty 1 | the computers | 6,001 | **12** | whole |

The original ended rows 63 and 103 itself, both with the two computers allied against the idle human; our walk
holds no `game_over` by either end (`over: None`). `docs/RUNS.md` run1773 to run1780 holds every value, and the
pins hold both sides of every first parting.

*Not established*: no row names a leaf, since each take holds sums only. Row 23 is whole (§12.6). Row 63 is whole: leaders 1582
(COSTS, 4542), units 1789 (ARMY, 4553), ammo 3683 (`launch.rs`, 4558), builds 3986 (AI §29.5, 4563). Every other row parts first between 1582 and 3857, and needs a ranges take before its leaf is named.

**Row 123 whole (item 4546, run1785).** The leaf was who=1's producer 2000's `queued` on 2977, ours 3, theirs 2. who=2
left the one shared `find_counters[0x1c]` at 2011, who=1's count (its mark); `find_build@009e2970` resets such a
cursor to 2000 and starts at 2001, so who=1's three trainings on 2976 went 2007, 2000, 2007, where ours went 2000
first. The lap resets now (`ai_host::lap`, `find_unit`'s too; AI §11): twelve lines whole.

**Bays (4547, 4559).** 736 takes 1440's; 4559: eight (`launch::BAYS`, `pivot::RELEASES`; 1224 is 169's). 4561:
an unbuilt Village's close fixes borders. 4565: COMBAT §12.3; 4564: §43.2.

### 12.4 Row 95: a CEO going in leaves its city (item 4519)

**How it was established.** run1770 is run1764's file again, with the ranges of units, guys and leaders kept to 6000
(`RNG_MASK=0xc1u`, the scorer's line). Its sums are run1764's. On 5266 one leaders range parts: **range 128, record 1
(who=1), `flags` byte 3: ours `0x01`, theirs `0x03`**, so theirs' `leader_flags` holds `0x2000000`, the economy's dirty
bit (`docs/ECONOMY.md`, the rate's lazy period), and ours' does not. Every other leaders range agrees, and the units
and guys lines agree on 5265 and 5266. Theirs' trace on that frame has `Unit::do_cast` reaching
`Objects::init_unit`. In ours on the same frame, who=1's **CEO `1/19` (TI 357)**, which held a cast order (spell 650)
on 5265, is inside the newborn **Transport Freighter `1/184` (TI 322)**.

**The direction: ours skipped a call.** `Unit::go_inside@0061a2e0` opens, for a unit with the CEO bit (`+0x6c &
0x10000`), with `Unit::update_ceo_position@005f96b0(its tile, -1)`. That function `find_city_at`s the owner's city on
each side's tile centre and raises `0x2000000` when the two differ. The CEO stands in its city, and `-1` is no city,
so the bit rises. `come_out@00617c10:519`–`524` makes the mirror call after `set_new_location`, `(-1, the spot's
tile)`. Ours built the tile-crossing, `init` and `set_type` arms only.

**The build.** `Sim::board` and `Sim::go_inside` call `update_ceo_position(unit, its tile, None)` for each figure going
in. `land_passenger` and `come_out_place` call `(None, the spot's tile)` once the figure is placed.

**Diff-backed**: `row_95_s_ceo_dirties_its_economy_as_it_boards_its_freighter_on_run1770_s_ranges` compares every
leaders range on 5265 and 5266, byte for byte. It names record 1's dirty bit, clear on 5265 and set on 5266, and the CEO
`1/19` aboard on 5266 alone. **Mutation** (`tools/mutate.py`, `board`'s call removed): held, exit 101, failing that test
at range 128 on 5266 (`[3, 7, 130, 1, …]` against `[3, 7, 130, 3, …]`) and row 95's score pin. **Moved**: leaders
5,266 → 6,001, whole; **row 95 has ten lines whole**.

*Not established*: the `come_out` arm has no diff yet. It is built from the reading alone, at both of ours' exits. The
CEO does not leave the freighter before 6000 in ours, and no capture shows a CEO garrisoning a building. ~~Units and guys
part on 5909~~ (closed, §12.5).

### 12.5 Row 95: the MLRS bears from its pivot node (item 4536)

5909: record 99, the MLRS `1/99` (TI 273, piece 223), `turret_angles[0]` ours 687 << 16, theirs 680 << 16.
`pivot::NODES` lacked `(223, 4)`; built off run883b, diff-backed by
`row_95_s_mlrs_bears_from_its_pivot_node_on_run1770_s_ranges`. Story: its journal.

### 12.6 Row 23: setup claims under the colonized limit (item 4534)

`analyze_map` sets the player-region bit after every `build_empire` (`Setup::build_game@005ac190:881`), so who=0's
setup pass gives (16, 42) to no one: run1781's `oil_patches` `[2, 5]`. `docs/journal/2026-11-22-item-4534.md`.
Units whole (item 4538): Copper's `update_hits` arm, `docs/COMBAT.md` §4.3.

## 13. Parity held out, sixth: rows 13 to 153, three seats, a team and a scripted human, to 12,000 (item 4566)

DECISIONS 151 §6, 152 §4, 2026-11-22. Rows 13, 33, 53, 73, 93, 113, 133 and 153 of the order `Start:` counts, by the
fifth's recipe (§12.3): each keeps its human (seat 0, tribe 4) and its computer (seat 1, tribe 11, 8, 2, 11, 12, 11,
17, 6) and adds a computer of nation 9, 10, 13, 14, 15, 16, 18 or 19, none already seated and none the fifth added
(`HELD_OUT_SIXTH_SEATS`). The team alternates as the fifth's: the human and the added computer are team 0 on rows 13,
53, 93 and 133; the two computers are team 1 on rows 33, 73, 113 and 153. Two things are new. **Each game closes on
12,000**, twice the five before (`SIXTH_FRAMES`). **And the human plays from a script.**

**The minting serves a held-out lobby.** `recgame::minted_packages` (§9.1) takes any list of orders; only the
script's objects are one lobby's. So `diff::held_out::script_rule` reads §9's three orders off a row's own built start:

- **the move**: slot 0's unit `0`, sent 2,400 units along the first of four headings (toward the map's centre on x,
  on y, then away on x, on y) whose point is on the map and in the unit's own region (`tregion_alt`);
- **the train**: slot 0's building `2000` queues one Citizen (TI 50), available to slot 0;
- **the Farm**: the Farm available, the Citizen's and the Farm's prices together inside slot 0's stock, dropped on
  the tile centre nearest the capital within 12 tiles that `blocked_site` calls clear (ties by y, then x), and built
  by slot 0's Citizen nearest the drop point (ties by object number).

Frames 30, 60 and 90, and every field (`queued` 2, `orders` 1, `form`/`width` −1), are §9.1's
(`scripted_orders`). **On run53's lobby the rule gives §9's script package for package**
(`the_script_rule_gives_run53_s_lobby_the_minted_script`). A row the rule refused would be recorded idle; none was.
Unit `0` is a land unit on every row: Scout (TI 69) on 13, 53 and 153; Explorer (71) on 33 and 113; Commando (73) on
73; Special Forces (75) on 93; Elite Special Forces (77) on 133.

**The declaration is `dcaf33581`** (17:20 −05:00): `HELD_OUT_SIXTH` (the page's block per row, as §12.3's) and
`HELD_OUT_SIXTH_HUMANS` (each row's objects and points), held by
`the_sixth_held_out_eight_are_the_rows_the_rule_names_seated_and_scripted`, which builds each row's start from the
page and runs the rule. It went red with row 113's Farm point bent by one. No file existed before it.

**The files.** The client recorded each row idle to frame 1 (`client --record <draft> --headless --page-of <capture>
--seat <tribe> --team <a>=<t> --team <b>=<t> --frame 1`), so the header, donor and frame-0 package are the client's,
as the fifth's. `the_sixth_s_files_are_minted` builds the start from each draft's header, checks it gives the
declared human, and mints the packages again with the three orders, closing on 12,000:
`item4566-row<RRR>-12000.rcx`. `the_sixth_s_files_carry_the_declared_seats_team_and_orders` reads them back.
Playback (`checksum::playback`) applies the orders to our walk through `Stream::pump` (§9.2), as it does every
recording's packages.

Each was played by the original as `run1788 + k` under the checksum hook alone, between 17:31 and 18:51 −05:00, and
pinned once in `SCORES_SIXTH`. That found column is the estimate: **1 of 8 whole, 20 of 96 lines, reach 49 %**
(32,565 of 66,044 states); **inside the first 6,001 states, 59 %** (24,891 of 41,960), against the fifth's 48 %.

| row | run | lobby | team | states | whole | lowest parting (state) | reach in the first 6,001 |
|---|---|---|---|---|---|---|---|
| 13 | run1788 | Amazon | human + added | 12,001 | 2 | units, builds, groups, guys 7675 | 6,001 of 6,001 |
| 33 | run1789 | Great Sahara, Enlightenment | the computers | 3,944, ended on 3943 | 1 | units, cities 2411 | 2,411 of 3,944 |
| 53 | run1790 | East Indies, Classical | human + added | 12,001 | 1 | units, guys 3358 | 3,358 of 6,001 |
| 73 | run1791 | Great Sahara, Industrial | the computers | 3,438, ended on 3437 | 1 | units, cities 1563 | 1,563 of 3,438 |
| 93 | run1792 | Great Lakes, Modern | human + added | 12,001 | 1 | units 2342 | 2,342 of 6,001 |
| 113 | run1793 | Great Lakes, Gunpowder | the computers | 4,573, ended on 4572 | 1 | units, guys, cities 2259 | 2,259 of 4,573 |
| 133 | run1794 | Great Sahara, all techs | human + added | 6,085, ended on 6084 | 1 | leaders 956, builds 958; leaders 1757 (4575); units, builds and leaders 1905 (4585); **units 1906** (4594) | 956 of 6,001; 1,906 now |
| 153 | run1795 | Great Lakes, difficulty 2 | the computers | 12,001 | **12** | whole | 6,001 of 6,001 |

The original ended rows 33, 73 and 113, each with the two computers allied against the human, and row 133. Our walk
holds `game_over` from ~~10112~~ 9522 (item 4577) on row 93 and from 4118 on row 113, and none by the others' ends. Walls agree on every
state of all eight. `docs/RUNS.md` run1788 to run1795 holds every value; the pins hold both sides of every first
parting.

**Read against the sample** (152 §5). Row 153 runs whole to 12,000 under a script, and row 13 to 7675, past every
earlier battery's end. The other six part before 3400, as idle rows did, so the found column alone does not say
the sample's trouble is what a person's own orders reach: one row whole and one long reach are not a battery's
verdict.

**Row 133's first leaf, the battery's first** (item 4575; run1797, the file's ranges take, `docs/RUNS.md`). On 956
who=2, the added computer, rebuilds its make list; theirs' `make_list[3]` is a Jet Fighter (TI 295) ×10 at `val` 986,484,
no escrow, and ours' an ICBM (TI 316) ×1 at 2,000,000 with an escrow; on 958 theirs queues the four Jet Fighters at
`2/2011` and ours the ICBM at `2/2017`. **Theirs refused the nuke**: who=2's one enemy, who=1, has the Missile Shield
queued at `1/2005` (both sides, byte for byte), and `create_units`' nuke arm skips an enemy researching it
(`docs/AI.md` §158). Every line but walls moved; the row's first parting is leaders on 1757 (range 308, record 2,
`make_list[2]`: `t`'s low byte ours 0xba, theirs 0x11; `val` ours 6,749,936, theirs 6,477,032; `escrow` 1 and 0; `num` 1
and 3; `cat` 8 and 6).

**Row 133's second leaf** (item 4585; run1797's ranges, no take). Decoded whole, 1757's slot 2 is ours' Bunker (TI 442,
build record 28) at 6,750,000 with an escrow in who=2's capital. Theirs' is TI 273 at 6,478,056; ours carries that entry
in slot 3. Ours listed the Bunker again in slot 8 (`cat` 8), where theirs has the Market (TI 436) at 5,610,000 in its
second city. Ours computes that Market too. **Theirs never listed the Bunker.** The capital's `city_flags` are `0x405b`
on both sides. Bit `0x8` (`ever_attacked`) spares the tower arm's halving (`docs/AI.md` §167), so the Bunker is worth
13,500,000, and `check_income`'s 32-bit product (256 × that) wraps negative. Every line but walls moved again, and the
row's first parting is units, builds and leaders on 1905.

**Row 133's third leaf** (item 4594; run1797's ranges, no take). On 1905 the units, builds and leaders lines part on
one unit. Who=2's Citizen `2/72` (TI 0x32) holds ours' `MOVE_TO` and `GATHER` of its Oil Well `2/2008` against theirs'
one `REPAIR` of the capital `2/2000` (units range 3966). `2/2008`'s `gather_down` is 72 against −1 (builds range 2061).
Who=2's `flags` byte 3 is 0x00 against 0x02 (leaders range 265), the dirty bit a killed `GATHER` raises. **Theirs
acted**: the capital carries damage 5,949, past half its hits. On frame 1904 (`(1904 + 2000) & 63 == 0`)
`Build::process`'s repair recruit sends it the nearest not-busy citizen (`docs/AI.md` §167). The arm is built. The
row's first parting moves one state, to units alone on 1906. Builds go to 2166 and leaders to 2212.

*Diff-backed*: every number above (the hook's twelve sums on every state, ours against theirs), and row 133's leaf on
955 to 958 (every leaders range), and its third on 1904 and 1905 (every units, builds and leaders range). *Read, not diffed*:
that `script_rule`'s choices are what a person would make; only their effect on the original is compared. *Not
established*: no other row names a leaf, since each take holds sums only, but row 73. Item 4576 read it off run1798's
ranges: on 1563, who=2's Caravan `2/31` is homed on its ally who=1's city 3, and ours links its trade order. Item 4583
built why theirs does not (run1800's packet, `docs/CARAVAN.md` §4): `do_trade`'s link gate refuses a home that is not
the caravan owner's without `has_preq(BASE_BONUSTYPES)`, and `LAB_005eda11` gives the order again. That moved rows 73
(first parting 1563 → 2779), 33 (whole but ammo, first 3386) and 113 (whole but cities, first 3995). ~~none of the three
new partings is read.~~ Row 73's 2779 is read and built by item 4589 (`docs/ANIM.md` §16): who=2's ARMOREDCAR `2/70`,
idled by its guard stand, is put back on `CHAR_ATTACKWALK` by `Guy::move`'s moving walk; row 73 is whole but ammo,
~~first 3279, unread~~ first 3279, read and built by item 4595 (`docs/COMBAT.md` §24.4): who=2's Armored Car `2/32`,
on `CHAR_ATTACKWALK` at the gone Citizen `0/4`, releases on 3278 into `Objects::add_ammo`, which counts `graph_index`
1008 while `Ammo::init` refuses the round; **row 73 is whole on all twelve lines, 3,438 of 3,438**. Row 33's 3386 is named and closed by item 4591, below. ~~Row 113's 3995 is not read.~~ Row 113's 3995 is read and built by item 4592 (§13.4). Also not established: a script past §9's three orders, or one issued after frame 90; whether row
13's 7675 and the sample's 7055 are one mechanism.

**Row 93's first leaf: the President heals** (item 4577, run1799). run1799 is row 93's
file to 12,000 with the checksum and ranges hooks (`RNG_MASK=0xffbu`). On state 2342, units range 1359 (record 39,
who=1) parts on `healing` alone: ours 0, theirs 20. The unit is who=1's Volksgrenadiers `1/39` (TI 109, land, a
captain), walking under an `AttackTo` and guarding who=1's President `1/17` (TI 355, about 600 away). Its figure
`1/40` parts in the same set: `damage` ours 1, theirs 0; `damage_frac` ours 8, theirs 6. Both sides took the frame's
blow (+6), and theirs had taken the point off first. `(39 + 2341) % 20 == 0` is the 20-frame phase of
`process_healing@005e0670`'s three economic-patriot arms. The President's arm (`:179`–`:203`) heals on an allied cell
or one nobody owns. This crate carried it as a SEAM in `units/heal.rs` (parked 3686a). The Antipater/Wellington arm
(`ANTIPATER_HEAL_RATE` 20) was ruled out: no unit of TI 361 or 368 exists on 2340 to 2342. The arm is built as
`Sim::president_heal`, between the Senator's and the CEO's. **Units 2,342 → 2,686.** The row's first parting is now
2686, where units, builds and guys part on one state (units range 1145, record 31, `flags` ours 17, theirs −111).
Ammo goes 3,924 → 4,263. Deaths (3,464 → 3,406), items (4,832 → 3,127) and goods (4,140 → 3,056; first parting
3157 → 3056) lose agreeing states, all past 2686.

*Diff-backed*: the arm on allied ground, by every units range on 2341 to 2343
(`row_93_s_volksgrenadiers_heal_beside_their_president_on_run1799_s_ranges`). *Read, not diffed*: the unowned-cell
half (`iVar8 < 0 ||`, `005e0670:184`); `1/39` stood on allied ground, so only the unit test holds it.

**Row 33 whole: the Musketeer's socket** (item 4591, run1801, the ammo take of run1789's file, `RNG_MASK=0x8u`). State
3386, ammo range 4, record 4, `rolling..start_roll_angle`: byte 7, `launch.x`, ours 6906, theirs 6907, and bytes 19 and
41, which follow from the launch; `launch.y` 32948 agrees. It is the only parted range. The round is who=2's Musketeers
`2/25` (TI 99, land, piece 49) at (6840, 33096) facing 244,973,568, loosing `CHAR_ATTACK3` at frame 4 on 3385. Piece
49's rows were item 3691's fit to the disk's 92 rounds, each a feasible region's centre-most point. The original's own
`get_position`, after `init_position`, on run1579's packet, swept over all 361 whole degrees for each of the three keys,
admits one `launch::Bay` region a key (411, 196 and 113 lattice points); the fitted rows held 357, 361 and 329 of the
headings, and this one is a miss of the third. The rows are now the regions' centroids, (33,049, 139,808), (11,451,
166,599) and (12,330, 162,102), heights 218, 176 and 178. They are piece 1105's rows to the digit (item 3692, the
Asian-style Musketeer, fitted from run1361's packet), an independent fit agreeing. **Ammo 3,927 → 3,944: row 33 is
whole on twelve lines to the original's end 3,943**, the sixth's second whole row (2 of 8). No other row of the sixth
moved. `row_33_s_musket_ball_leaves_from_the_original_s_socket_on_run1801_s_ranges` holds every ammo range on 3385 to
3400, and `data::launch::tests::the_original_s_release_agrees_at_every_heading_for_the_musketeers` the sweeps
(`RON_BAY_SWEEPS_4591`, outside git). *Diff-backed*: the round's launch on 3386 and the row's twelve sums on every
state. *Read, not diffed*: the `CHAR_ATTACK1` row's four headings the old fit missed; no capture has a round on one.

**Where a row's landing writes** (DECISIONS 154 §6). Each open row has a heading below. A landing writes under its own
row's heading and touches nothing above this paragraph: four updates conflicted on the one *Not established* paragraph
above, and two merges backed out on it. The table above is the found column and is not edited again; `SCORES_SIXTH` and
the queue's line hold what stands now.

### 13.1 Row 13

Open: units, builds, groups and guys part on 7675. run1788 holds sums (item 4579).

### 13.2 Row 53

Open: five lines part on 3713. 4660 moved 3358 → 3713: the stored heights (`docs/COLLISION.md` §3.3). Journals
4578, 4609, 4660.

### 13.3 Row 93

2686 → 2965 by 4590 (run1803; `docs/COMBAT.md` §68.1), 4606 (`docs/CHECKSUM.md` §18). Open: units on 2965: `2/51` takes the decoy `1/38`, theirs `1/39` (4652). 4608 (COMBAT §55.3), 4652 moved no parting. 4654 (run1830b): step 17's decoy attack is `ObjectData::attack`, theirs 240, ours 260; not built.

### 13.4 Row 113

**Whole: twelve lines of twelve, 4,573 of 4,573 states, to the original's end** (item 4592, run1802). Cities parted on
3995 with the sums' low halves equal and high halves 8 apart: the same bytes in another order. run1802's cities range
11, record 0, `vans[k]`, byte 4 (`who`) reads ours 1, theirs 2. who=1's city 0 holds `(3,2) (3,1)` on theirs, where
ours walked `(3,1) (3,2)`: who=1's Caravan `1/34` (TI 59, land) links on 3995, after who=2's `2/131` (TI 59, land).

**A city's `vans` stand in the order the links were made.** `ArrayBase<CaravanLink>::add@0046e550` appends, with
`do_trade` as its one caller. `Array<CaravanLink>::remove@0046e4e0` keeps the rest in order, with `end_trade_route` as
its one caller. Ours stamps a link when `do_trade` makes it (`Caravan::linked_at`), and `Sim::city_links` walks by the
stamp, which also closes run1325c's packet (cities 2 ranges → 0). Held by
`row_113_s_city_walks_its_caravan_links_in_the_order_they_were_made_on_run1802_s_ranges` and
`cities_tests::a_city_s_caravan_links_stand_in_the_order_they_were_made`. Not established: a link `end_trade_route`
leaves standing because its end has closed (theirs keeps it; ours drops it).

### 13.5 Row 133

Open: ammo parts on 2547 (4659: deaths → 3206, Redoubt `dtype` 3). run1797's ranges hold the items' frames, *diff-backed*.

**Items 4597 to 4620**: units → 2376, builds → 2432, ammo → 2408, leaders → 2413, cities → 2448 (`docs/GROUPS.md`,
`docs/COMBAT.md` §84, a Temple's close, two walks by object number; their journals).

**Item 4628** (run1819's packet): units → 2440: `get_damage`'s rocky step reads the target's cell `flags & 8`.

**Item 4632**: units → 2543. On 2439 the Flamethrower `1/115` (`recharging` 30, its attack's `flags` 16) turns at
its target in theirs and swings in ours: `Unit::fight`'s emplaced latch (`:92`–`100`) takes the head order's `0x10`.

## 14. Parity held out, seventh: rows 8 to 148, the human ordering through the game, to 12,000 (item 4600)

DECISIONS 154 §7, 2026-11-23. Rows 8, 28, 48, 68, 88, 108, 128 and 148 of the order `Start:` counts, by the sixth's
recipe (§13): each keeps its human (seat 0, tribe 4) and its computer (seat 1, tribe 8, 13, 16, 11, 17, 5, 13, 3) and
adds a computer of nation 20, 21, 22, 23, 11, 17 or 0, none already seated on its row (`HELD_OUT_SEVENTH_SEATS`); 20
to 23 had never been added by a battery. **Row 108 adds no seat**: its own lobby already holds three (Greek and
Egyptian computers at seats 1 and 2), the most `MAP_SIZE 2` holds, so it takes the team alone (`NO_SEAT`). The team
alternates as the sixth's: the human and the added computer are team 0 on rows 8, 48, 88 and 128; the two computers
are team 1 on rows 28, 68, 108 and 148. Each game closes on 12,000 (`SEVENTH_FRAMES`).

**The one change: the human keeps ordering.** `ordering_rule` is §13's `script_rule` and a square. After §9's three
orders, on every 600th frame from 600 to 11,400 (nineteen rounds) one package carries two orders, as one turn of a
person's input does: the capital `2000` grouped and one Citizen queued (`queue_up`, num 1), then unit `0` grouped and
moved in §9.1's fields (`orders` 1, `queued` 2, `form`/`width` −1). Round `r` moves unit `0` 2,400 units along heading
`(i + r) % 4` from where round `r − 1` sent it, `i` being the heading §9's move took. So four moves close a square,
and the last corner is the first move's point. The points are named from the built start and not walked: a unit or a
capital gone by its round, or a corner the unit cannot reach, is part of the test. On all eight rows `i` is 0 (toward
the centre on x), and every corner is on the map and in unit `0`'s region. Unit `0` is a Scout (TI 69) on rows 8, 48,
68, 108, 128 and 148, Special Forces (75) on row 28 and an Explorer (71) on row 88: land units every one.
`the_ordering_rule_is_the_script_and_a_closed_square_on_run53_s_start` holds the rule on run53's lobby. There the
first three orders are §9's script, package for package.

**The declaration is `f7d2e41c6`** (11:16:48 −05:00): `HELD_OUT_SEVENTH` and `HELD_OUT_SEVENTH_HUMANS`, held by
`the_seventh_held_out_eight_are_the_rows_the_rule_names_seated_and_ordering`. It went red with row 8's first corner
bent by one. No file existed before it.

**The files and takes.** The drafts and the minting are the sixth's (`the_seventh_s_files_are_minted`,
`item4600-row<RRR>-12000.rcx`; `RON_RECGAME_DIR` names the donors' directory). Row 8 was minted and played alone first
as run1804: **the original took it** to 12,000 with no relaunch. The other seven were then minted and chained, one at
a time. Rows 128 and 148 waited for the disk (59 GB, under 60) and item 4611 took them (§14.6).

As found, pinned once in `SCORES_SEVENTH`: **1 of 8 whole, 6 taken, 20 of 72 lines, reach 59 %** (of the six's
states); **inside the first 6,001 states, 87 %**, against the sixth's 59 % as found.

| row | run | lobby | team | states | whole | lowest parting (state) | the last round before it | reach in the first 6,001 |
|---|---|---|---|---|---|---|---|---|
| 8 | run1804 | Great Sahara, difficulty 3 | human + added | 12,001 | 2 | units 9896 | 16, on 9600 | 6,001 of 6,001 |
| 28 | run1805 | East Indies, Modern | the computers | 4,248, ended on 4247 | 2 | units, builds, guys, world 3956 | 6, on 3600 | 3,956 of 4,248 |
| 48 | run1806 | Great Sahara, Classical | human + added | 12,001 | 1 | ammo 5607 | 9, on 5400 | 5,607 of 6,001 |
| 68 | run1807 | East Indies, Classical | the computers | 3,949, ended on 3948 | **12** | whole | — | 3,949 of 3,949 |
| 88 | run1808 | East Indies, Medieval | human + added | 12,001 | 1 | units, deaths, guys, leaders, world 2741 | 4, on 2400 | 2,741 of 6,001 |
| 108 | run1809 | Great Sahara, Classical, melee | the computers | 12,001 | 2 | units, builds, groups, guys 7421 | 12, on 7200 | 6,001 of 6,001 |
| 128 | run1810 | Great Lakes, difficulty 2 | human + added | 12,001 | 11 | leaders 3338 | 5, on 3000 | 3,338 of 6,001 |
| 148 | run1811 | Great Sahara, difficulty 4 | the computers | 12,001 | 1 | units 10879 | 18, on 10800 | 6,001 of 6,001 |

The original ended rows 28 and 68, each with the two computers allied against the human. Walls agree on every state
of all six, and items on rows 8, 28 and 68. `docs/RUNS.md` run1804 to run1809 holds every value, and the pins
hold both sides of every first parting.

**Read against the sixth.** Every lowest parting is past round 4, so the ordering human agrees through at least four
of its rounds on every row, and through sixteen on row 8. *Diff-backed*: the hook's twelve sums on every state, ours
against theirs. *Read, not diffed*: whether a parting is the ordering's own work or a computer's. Each take holds sums
only, and no row's leaf is named. The column "the last round before it" is a frame count and not a cause.

**Where a row's landing writes** (DECISIONS 154 §6). A landing writes under its own row's heading below and touches
nothing above it. The table is the found column and is not edited again.

### 14.1 Row 8

Open: units part on 9896, round 16's move on 9600 the last order before it. run1804 holds sums.

### 14.2 Row 28

Open: units, builds, guys and world part on 3956; the original ends the game on 4247. run1805 holds sums; run1813
holds units, builds, guys and world ranges to the end.

**Item 4613, the first leaf** (the seventh's; it joins `held_out::SPENT`). On 3956 units range 5, `0/0` (Special
Forces, TI 75) `targeted` ours 5, theirs 6; builds range 7, `0/2000` `targeted` 0 against 1. *Diff-backed*
(`row_28_s_bazookas_drop_their_chase_of_a_citizen_on_run1813_s_ranges`): who=1's Bazookas `1/35` and `1/52` chase
Citizen `0/7` on both sides through 3955, every range agreeing there; on 3955 theirs ends both walks and retargets
both, ours keeps them. The review and `do_move`'s dead-target arm are ruled out (`docs/journal/2026-11-23-item-4613.md`).

**Item 4619, the arm, and the row moves: 3956 → 4027.** *Read on a packet* (run1816, `RON_STATE_FRAME=3955`): a third
member of group 66, `1/25`, finds `0/0` in `Unit::think_attack` and hands it to `Group::target_opportunity`; each member
takes `Unit::target_opportunity`'s yield and re-searches. Ours refused the yield in `attack_action_yields` on the mask
`0x840002` alone; the original refuses only when the held target is siege and the attacker neither siege nor a
building (`005fffc0:150`–`:166`), and a shielded unit holding a building takes the range tests (`600440`). Both arms
are built. *Diff-backed* (the row 28 test, extended): on 3956, ours before | theirs, `1/35` `path.length` 11 | 0,
`unit_masks2` 0 | 16; `1/52` `flags` 17 | `0x91`, `path.length` 8 | 7; `0/0` `targeted` 5 | 6; all agree now.
`SCORES_SEVENTH[1].moved` 2 → 7 lines whole; the lowest parting ammo 4027 (`docs/journal/2026-11-23-item-4619.md`).

**Item 4626, the India Bazooka's bay: 4027 → 4167.** run1820 is row 28's file to 4028 with the ammo ranges. On 4027
ammo range 1, record 1 is who=2's `2/34`, a Bazooka (TI 139, land), firing at `0/2000` on 4026 under `CHAR_ATTACK2`
frame 7 from **piece 1849** (base 89 at art style 5), which `BAYS` lacked. run1816's packet holds its three
`AttachPos` rows; `fit_bay.py` fits each, 361 of 361 headings. *Diff-backed*
(`row_28_s_round_at_the_capital_on_run1820_s_ranges`), ours before | theirs: `launch` (12264, 12552) | (12208,
12465), `sz` 145 | 296, `total_time` 12 | 11. 7 → 9 lines whole (`docs/journal/2026-11-23-item-4626.md`).

**Item 4630, the India Infantry's bay, and the row is whole.** run1821 is row 28's file to 4168 with the ammo ranges.
On 4167 ammo range 1, record 0 is who=2's `2/50`, an Infantry (TI 102, land) on **piece 1812** (base 52 at art style
5), firing at `0/2000` on 4166 under `CHAR_ATTACK2` frame 4; `BAYS` had no row for it. run1816's packet holds its six
`AttachPos` rows, and `fit_bay.py` fits each to piece 52's regions exactly. *Diff-backed*
(`row_28_s_round_on_run1821_s_ranges`), ours before | theirs: `launch` (13032, 9000) | (12973, 9025), `sz` 24 | 144.
`SCORES_SEVENTH[1].moved`: 9 → 12 lines whole, 4,248 of 4,248 on every line.

### 14.3 Row 48

Open: ammo parts on 5607; units follow on 5940. run1806 holds sums.

### 14.4 Row 88

Open: ammo parts on 3028 (item 4662). run1808 holds sums, run1815 the ranges to 2991 (units, deaths, guys,
leaders and world), run1818 the ammo ranges to 2792. The stories are the items' journals.

**Item 4614: a rider comes out before its carrier dies, and a fire raft dies with its shot.** Both are diff-backed by
run1815's ranges.

- **The seam's fix** (DECISIONS 155 §9). `Object::kill_contents@00645550` calls `Unit::come_out(p, 0)` and then
  `Unit::die_uber(p, 0)` on each head of `inside_down`; this crate let each rider die aboard. `kill_contents` now runs
  `Sim::rider_come_out` (the boat's ring for the captain, the captain's for each member) and kills the squad in
  `die_uber`'s order, the captain last; a rider whose `come_out` refuses dies aboard through `close_inside_disc`. On
  2740 who=2's Peltasts `2/38`..`2/40` (TI 84) come ashore off the sunk barge `2/71` (TI 320) and die there. Off,
  world parts on 2741 (range 4934, record 1101, `block.present` ours 0, theirs 1); on, it agrees to 2861.
- **A fire raft dies with its shot.** On 2741 theirs' deaths slot 0 is who=1's Heavy Fire Raft `1/87` (TI 335); ours
  kept it alive. `Unit::fight@005fd4d0:900`–`906`: after `fire_ammo`, a type whose `unit_flags` carries `0x2000`
  (`uflags::SELF_DESTRUCT`) takes `Object::die(this, 3, −1, 0)`. Built in `fight_as`'s ranged arm, held by
  `combat::fight::tests::a_fire_raft_dies_with_its_shot`.

**Item 4622: a self-destructing shooter splashes from the shot's middle.** `Ammo::do_damage@00678060` at `678652`
calls the shooter's vslot `+0x18` (on `Unit`, `return 1`) and at `678675` tests its type's `unit_flags & 0x2000`; with
both, `678681`..`6786a0` set the splash centre to `((sx + ex) / 2, (sy + ey) / 2)`, truncated, which the cell walk and
both arms' distances read. Barge `2/61`'s `myhits..` on 2741 read ours 12, theirs 0. Built in `Sim::land`, held by
`combat::fight::tests::a_self_destructing_shooter_splashes_from_the_shot_s_middle`.

**Item 4625: the Carrack fires from its starboard battery.** Ammo record 0 on 2791 is who=1's Carrack `1/82` (TI 342,
piece 292) firing `CHAR_ATTACK2`'s node 1 at `2/61` on 2790: `sx, sy, sz` ours (16824, 11640, 0), theirs (16866,
11729, 35). **Ours acted**: `launch_point` had no row for piece 292. Its nine `RELEASEEVENT`s are swept on run1579's
packet and fitted, built as nine `BAYS` rows, held by `data::launch::tests::the_carrack_s_round_leaves_its_starboard_battery`
and `the_carrack_s_release_agrees_at_every_heading_but_one`. run1818 agrees on all twelve lines to 2792.

**Item 4631: a transport's kill plunders as its cargo.** On 2883 who=1's `encrypt` reads `bucket[1]` (timber) ours
15,500, theirs 15,485, and `bucket[2]` (wealth) ours 17,667, theirs 17,767. On 2882 the Carrack's round sinks `2/61`
carrying who=2's Despot `2/24` (TI 0x160), with who=1's Despot `1/14` 2,115 away (reach 2,304). Both sides draw four
times in `Unit::plunder@00605b00` (theirs at `+0x4e5`). **Ours acted**: it asked the barge whether it was a hero, and
paid the Despot's 15 in the least-stocked good. `Object::take_damage@00652020` at `653090` tests the victim type's
`+0x2b4 & 0x10` and at `6530a5`..`6530a9` takes `inside_down_who` and `inside_down`, so the ally test and the
GENERAL/CARA/SUPPLYWAGON `is` tests read the cargo's head. An empty transport (`js 0x6533b2` at `6530ba`) skips the
plunder and the Inca refund. `plunder` redirects the same way, so `GENERAL_PLUNDER` 100 beats the 15 and is paid in
wealth. Built as `Sim::plunder_victim` (which `score_victim` reads) in `unit_plunder` and `inca_refund`. Held by
`cities_tests::a_transport_s_kill_plunders_as_its_cargo`; the mutation with the redirect dropped fails it.

**Item 4633: a boat's damage comes ashore on its squad's tail.** On 2902 units range 3709 reads `damage` ours 12, theirs 0,
on who=2's Elite Javelineers captain `2/29` (TI 84), and theirs' 12 is on the tail `2/31`. On 2901 the barge `2/65` (TI
320, 44 of 50) unloads them. `Unit::same_damage@005f9400` walks the receiving squad from its last live figure up `o_up`,
owing the larger damage fraction of `myhits` less the squad's `total_damage@00609c00`: 30/256 of 110 is 12, on the tail.
Built in `Sim::same_damage` (`docs/TRANSPORT.md` §6.4), held by
`ai::transport::tests::a_boat_s_damage_comes_ashore_on_its_squad_s_tail`.

The score, `SCORES_SEVENTH[4].moved` (12,001 states). 4614: deaths and world 2,741 → 2,862, groups → 2,883, guys →
2,851. 4622: units → 2,807, leaders → 2,879; first 2741 → 2791. 4625: units → 2,902, ammo → 3,056, deaths → 2,922,
groups → 2,928, guys → 2,907, leaders → 2,883, items → 7,829, world → 2,909; first 2791 → 2883. 4631: leaders 2,883 →
2,926. 4633: units → 2,985, builds → 3,015, ammo → 3,170, guys → 2,985, leaders → 3,011, goods → 3,543; first 2902 →
2985. 4662: units → 3,033, builds → 3,144, ammo → 3,391, deaths → 3,463, guys → 3,069, leaders → 3,134, world → 3,086;
first 2985 → 3028.

**Item 4662: a stale attack is cleared, not retargeted.** On 2984 who=1's Trebuchet `1/61` (TI 266) ends its
attack-ground; the Attack under it names `2/63` at uid 93, a number another barge holds. `Unit::work@0060d180:300`–`:340`
marks the object at `(whom, ox)` in danger and, on the uid mismatch, writes `ox`/`whom` −1 and returns; `fight` searches on
2985. On 2985 `order.target` ours (64, 2, 94) → (−1, −1, 93), theirs (−1, −1, 93). Held by
`orders::reused_target_tests::an_attack_on_a_reused_number_is_cleared_and_searches_a_frame_later`.

**The next word, 3028: ammo**, past every range on disk (run1815 to 2991, run1818 to 2792): a take is owed.

**Not established.** `Unit::resolve_unit_collision@005f9d30:79`'s suicide arm is not built: no capture has been shown
to reach it. A building shooter always splashes from the landing (`Build` and `Wall` fill vslot `+0x18` with a
`return 0`). The Carrack's node-2 `CHAR_ATTACK2` row (frame 24) is one unit off at heading 283; piece 128's six
node-0 attack keys still fall back to the figure's square on row 88 after 2883. A transport's head rider is taken as
the lowest live slot aboard, not read off `inside_down`'s ring order; no capture yet kills a mixed cargo. `eject_riders` walks riders in slot order too (4633a): the chain order moved nothing on run1815.

### 14.5 Row 108

Open: units, builds, groups and guys part on 7421. run1809 holds sums.

### 14.6 Rows 128 and 148

Taken by item 4611, found on `6fa43acc3`'s tree; with item 4630's row 28 the line is `(2, 8)`. Open: row 128's leaders on 3338, row 148's
units on 10879. Both hold sums.

**Item 4618** (piece 37's bays): row 148 has 11 lines whole, every one but units, which still part on 10879.

**Item 4635, row 128's Russian age, and the row is whole.** run1828 holds row 128's leaders ranges to 12000. On 3338
leaders range 1, record 0, `reg_terr[1]`: ours 198, theirs 0; who=1's 495 | 104 and who=2's 447 | 0, with
`reg_known_rares[1]` zeroed. On 3336 who=1 (tribe 13, the Russians) enters the Classical age on both sides.
`Leader::gain_tech@006dcb60:1305`–`:1322` calls `Regions::fix_all_borders` for a temple or fort border prerequisite,
a Civic epoch, or an age under tribe bonus 0xd, whether the table moves or not; ours fixed only on a moved table.
*Diff-backed* (`row_128_s_russian_age_fixes_the_borders_on_run1828_s_ranges`). `SCORES_SEVENTH[6].moved`: 11 → 12
lines whole, 12,001 of 12,001 on every line (`docs/journal/2026-11-25-item-4635.md`).
