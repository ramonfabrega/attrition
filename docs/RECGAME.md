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
style-1 donor serves a style-0 lobby. It is read from the user's install at
run time and never kept.

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
  needs a donor with the same options: one exists only for a lobby this
  install has recorded. Multiplayer, CtW and scenario recordings are out of
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

**Row 130, run1754's leaves**: the bomber `1/158` searches round itself
(item 4508, `find_new_bomber_target@005eb960`; units 3411 → 3515), 4522's
helicopters (3515 → 4583). Leaders whole (item 4520): from the Kremlin's
finish on 4573 to its spy on 4583 `leader_flags` holds `0x10000000`, which
the checksum never printed (range 128, byte 3: ours `0x0e`, theirs `0x1e`).

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
| 55 | run1762 | East Indies, Medieval | 6,001 | 10 | units 4275, groups 5576 |
| 75 | run1763 | Great Lakes, Gunpowder | 5,342, ended on 5341 | 11 | ammo 4004 |
| 95 | run1764 | East Indies, Information | 6,001 | 9 | leaders 5266, units 5303, guys 5909 |
| 115 | run1765 | Southwest Mesa, difficulty 2 | 6,001 | **12** | whole |
| 135 | run1766 | Great Sahara, difficulty 2 | 6,001 | **12** | whole |
| 155 | run1767 | Great Sahara, difficulty 2 | 6,001 | **12** | whole |

The original refused no file. In row 75, the one game it ended itself, our walk holds no `game_over` by that tick
(`over: None`). `docs/RUNS.md` run1760 to run1767 holds every value, and the pins hold both sides of every first
parting.

*Not established*: no row names a leaf, since each take holds sums only. Row 75 parts first by state (ammo on
4004, ours 3143699460, theirs 3217755213), then row 55 (units on 4275, ours 1914015254, theirs 708677162), then
row 95 (leaders on 5266, ours 871988666, theirs 426802620). Each needs a ranges take of its file, keeping the lines
it parts on (DECISIONS 148 §4), before its leaf can be named.
