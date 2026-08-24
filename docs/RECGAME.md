# The recorded game: the `.rcx` container

`docs/ORACLE.md` Part 1 derived the recorded-game container's shape from the
symbols in 2026-08-20 and deliberately stopped short of a reader, because a
reader written against no sample cannot be wrong in any detectable way. The
sample now exists — `ron.heavengames.com`'s downloads section, fileid 2170,
the one recording there that names EE, saved under the gitignored
`game/external-recgames/` — and this document is the full walk, read from the
decompile and checked byte-for-byte against that file.

**How this was established.** Every function named here was read in the
Ghidra export (`tools/ghidra/decomp/`): `RecordGame::read_header` /
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
  `init_record` mkdirs it, `finalize` discards a recording whose `empty` flag
  never cleared).
- **gzip end to end.** Every read in the walk goes through
  `LoadGame::walk_function`, which reads from a plain `FILE*` when one is
  open and otherwise `gzread`s; the shipped recording path opens the gzip
  arm. The sample is a bare gzip stream (NTFS OS byte), 199,968 bytes
  compressed, 1,676,994 decompressed.
- Everything below describes the **decompressed** stream. All integers are
  little-endian. There is no framing beyond the walk itself: the format is
  the flattened object graph, so the struct layouts *are* the format.

## 2. The three wire primitives

Everything in the header and rules is written by exactly three operations:

| primitive | bytes | meaning |
| --- | --- | --- |
| `walk_function(begin, end)` | `end - begin` | raw memcpy of a struct range, as laid out in memory |
| `walk_test(name)` | 1 | section marker: the low byte of the section-name `String`'s hash; the reader (`LoadGame::walk_test`) errors with "Error loading section, probably in *last section*" on mismatch |
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
| 29 | +0x18..+0x35, walked one byte at a time (the lobby options union `GameInfo_u_24`) | |
| 1 | +0x35..+0x36 | |

### 3.2 The eight player records

For each of the 8 lobby slots (`GameInfo.player[n]`, stride 0x8c):

| bytes | content |
| --- | --- |
| 1 | marker, player section (`0x0f` in the sample) |
| 2 | `Player.flags` (+0x30) |
| 0x39 | **only if `flags & 1`** (slot active): Player +0x00..+0x39 raw — the ten `synced_*` counters, `caravan_frame`, `pop_cap_frame`, then `flags`(u16), `tribe`, `who`, `team`, `handicap`, `play`, `pauses`, `diff` |
| 4 + 2n | only if active: `Player.name` (+0x40) |

`platform`/`platformID`/`net_player`/the `accum_*` bytes are not walked.

### 3.3 The GameInfo tail — version-forked

`sGameSaveVersion` selects the branch; `Game::run_playback` **forces it to
0x10** before `read_header` in this build, so the 2024 reader always takes
the second row. The sample file parses under it, so the 2017 writer used the
same layout (or one byte-compatible with it for an empty-mod game).

| `sGameSaveVersion` | layout |
| --- | --- |
| < 0x10 | four Strings: `scenario_script` (+0x4f4), `scenario_path` (+0x508), `scenario_dir` (+0x51c), mod path |
| ≥ 0x10 | 8 raw bytes (workshop id + download count), `scenario_script`, `scenario_path`, mod-install-dir String, 8 raw bytes (second workshop id), mod-name String |

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

Sequencing is in `Game::run`'s setup: **random-game info, then rules, then
packages**.

### 4.1 The random-game info blob

`write_random_game_info`: for each lobby slot with `flags & 1` and `who < 8`,
1 byte (`Player.team`) + 4 bytes (a leader-table field at
`leaders[who * 0x6eec] + 0x39c`); then 4 bytes once (`World` +0x30, the world
seed). Total `5 × active + 4`. **The playback side reads this blob into a
malloc'd buffer and frees it unused** (`read_random_game_info`) — everything
it duplicates already arrived inside GameInfo, so a reader can treat it as
skip-only. The sample: 5 active players → 29 bytes.

### 4.2 The rules section

`write_rules` = `Game::walk_rules_data`. The recording embeds the **loaded
rules tables**, so playback does not trust the install's XML:

1. 1 byte: rules section marker.
2. `Types::walk_rules_data`: 806 type objects (`types.list`, 0xc98/4), each
   dispatching its own virtual `walk_rules_data` (vtable +0xc4). **Not
   derived** — this is the variable-size middle, per-class walkers unread.
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
| 4 | `frame` |
| 4 | `play` (issuing player slot) |
| 4 | `valid` |
| 4 | `stamp` (`CommandPackage` +0x00) |
| 2 | `size` |
| size | `data` — the command payload |

`read_package`'s contract: called with the current frame, it reads the next
record's `frame` and, if it is in the future, seeks back 4 bytes and returns
0 — packages are consumed when their frame arrives. EOF flips the playback
semaphore. There is no count field; the stream runs to end of file
(ORACLE.md's "where read_package stops" is hereby confirmed: EOF).

The `data` payload's encoding is the **command format**, `CommandManager`'s
territory, and is deliberately out of this document's scope — it is the next
reading. What the reader surfaces without it: every package's frame, issuer,
stamp and size, which is already enough to segment the order stream per
player and frame.

## 5. What is not established

- **The per-class type walkers** (§4.2 step 2): 806 objects, virtual
  dispatch, unread. Until they are, the reader locates the tribes train by
  the landmark of §4.2 step 5 rather than by walking Types.
- **The marker-byte hash.** `String::generate_hash`'s low byte, but the
  decompiles of both `walk_test`s garble which of the two hash fields
  (`hash_value` vs `hash_value_insensitive`) supplies it, and the section
  names live in the runtime string table. Observed values instead: Game
  `0x16`, GameInfo `0x42`, player `0x50`, rules `0x92`, tribe `0x8f`.
- ~~**The +0x36..+0x38 gap**: the decompiler lost one call's arguments
  (§3.1's last row), so the 1-byte row could be 3 bytes.~~ **Settled by the
  sample**: with 1 byte the eight player records align exactly (one marker
  byte, eight times, `0x50`); with 3 they cannot.
- **The command payload encoding** (`CommandPackage.data`) — the next
  mechanic, not this one. Until it is read, the packages give frame, issuer,
  stamp and an opaque payload.
- **2017↔2024 drift, narrowly**: §6's table equality pins the balance and
  tribes spans for the sample, so the one span still resting on a 2024-only
  size is the **start** of the `Constants` block (0xd40) — if the 2017 block
  was a different size, the constants/types boundary is off for old files
  and nothing else moves. A recording made by *this* install has no such
  gap.
- Whether `write_ctw_header`/`read_ctw_header` (Conquer the World wraps the
  container) change anything — out of v1's scope with the rest of CtW.

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
