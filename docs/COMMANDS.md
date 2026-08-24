# The command payload: `CommandPackage.data`

`docs/RECGAME.md` §4.3 read the package stream — frame, play, valid, stamp,
size — and left the `data` payload as "CommandManager's territory, the next
reading". This is that reading: the encoding of every player input the engine
ever puts on the wire, which is the same encoding a recording stores and the
same stream a lockstep peer receives. With it, a `.rcx` recording becomes a
typed order stream — the input side of the replay diff.

**How this was established.** Read in the Ghidra export: the dispatcher
`CommandPackage::process` (0x94a700, the complete 0x00–0x51 switch), the walk
loop `process_all` (0x94c500), the appenders `add_command` (0x94bae0),
`add_group` (0x94bb60), `add_chat` (0x94c080), `add_spline` (0x94bfb0), the
network encoder `CommandPackage::send` (0x94c1e0), the turn pump
`CommandManager::process_turn` (0x93ef10) with `send_local_package` (0x940120)
and `end_process` (0x94c800), `process_group` (0x94a0c0) and representative
`process_*` bodies, plus a sweep of every `process_*`'s return value and every
`issue_*`'s `add_command` size argument (they agree everywhere). Struct
layouts are the PDB's, via `types.txt` — every `*Command` struct is present by
name. **Verified**: a decoder built from this document walks all 21,884
packages of the heavengames sample (`docs/RECGAME.md`) with **zero decode
errors and zero unknown command types**, every payload consumed to its exact
`size`.

**Confidence.** High for everything below: the dispatch table and sizes are
doubly derived (decode-side returns and encode-side arguments), the layouts
are the PDB's own, and the sample exercises 25 of the 82 commands including
all four variable-length paths' framing. Field *semantics* beyond what the
named `process_*` target functions state are inherited from the mechanics'
own documents (`docs/ORDERS.md`, `docs/CITIES.md`, …), not re-derived here.

## 1. The model

A `CommandPackage` is one player's input for one lockstep turn: up to 0x200
bytes of `data`, walked front to back by `process_all`. Each command starts
with one type byte; `process` dispatches on it and returns the bytes
consumed. There is no per-command length field, no alignment, no separator —
the type byte determines the size (four commands carry their own count
field). A wrong size is detected only as a package-level mismatch
("Size mismatch in CommandPackage::process_all", an ignorable error).

Commands that target units do not carry unit lists. A **`group` command
(type 0x00) precedes them** and sets the package's selection; every
subsequent unit-targeted command in the same package acts on that selection
(`process_group` pushes a `Group` and stores its index in
`CommandPackage.group`; `process_move_to` etc. call
`Group::action_move_to(groups[group], …)`). The selection persists until the
next group command in the stream.

The wire structs are the in-memory structs — `add_command` is a `memcpy` of
`size` bytes, so the PDB layouts are the format. They are packed from offset
+0x1 (no alignment padding on the wire); the one struct the PDB shows with
alignment padding (`LeaderOptionsCommand`, 0x24 in memory) travels packed as
0x21 bytes.

## 2. The turn pump, and what a recording stores

`CommandManager::process_turn`, three arms:

- **Single player** (`semaphore.ptr[0] & 4` clear): if the local package is
  non-empty, stamp it (`local_package_stamp`, contiguous from 1), **write it
  to the recording if recording** (`semaphore.ptr[0] & 8`), then
  `process_all` it. One package per turn, only on turns with input.
- **Multiplayer** (bit 4 set): `issue_check_sums` every turn;
  `send_local_package` adds `turn_data` + `camera` to the local package,
  stamps it and `CommandPackage::send`s it (the network encode, §5); then
  for each connected player the pump waits on that player's package FIFO,
  **writes each package to the recording as received** (before decode), and
  `process_all`s it. `end_process` then compares the eight players'
  checksums — the desync detector, with its recheck window and
  `checksum_failure_threshold` popup.
- **Playback** (`semaphore.ptr[0] & 0x10` set): `read_package` into the
  local package while records match the current frame, `process_all` each.

A fourth writer to the recording, outside the pump: `DropControl::
process_drops` synthesises a lone `ungraceful_player_drop` package locally
and writes it plain (it never passes through `send`) — the reason for §5's
exemption. And a fifth, textual only: the pump's invalid-package error path
writes the offending package before halting.

Consequences for a reader:

- A **single-player recording stores plain payloads** — the XOR/padding of
  §5 exists only on the network path. Verified: the sample decodes raw.
- A **multiplayer recording stores network-encoded payloads** (each remote
  package is written as it arrived; `process_command_package_data` copies
  wire bytes verbatim into the FIFO), and playback decodes them because
  the header's Game stretch carries both the seed and the whole semaphore —
  bit 4 restored, `run_playback` ORs in 0x10, so **a recording is
  self-decrypting** and a reader must apply §5's XOR exactly when the
  restored semaphore has bit 4.
- Bit 4's only writer in the export is `Game::run_gamespy` (0x587060) —
  the gate is precisely "network game" (second reading).
- `valid` is 1 only where `CommandManager::init` or the net handler
  (`process_command_package_data`) set it and `process_all` has not yet
  cleared it. In an SP recording that is **exactly the first record**
  (init sets it once; process_all runs after write_package and clears it
  for every later turn); in an MP recording it is **every wire package**
  (the net handler arms each), with the synthesised drop packages at 0 —
  a mode fingerprint a reader gets without parsing a single payload
  (second reading).
- The camera stream (one command per frame in the sample) is recorded
  input with playback-only effect: `process_camera` moves the camera only
  when a recording is playing (`record_game.flags.ptr[0] & 1`) or for MP
  spectators — it is how a replay reproduces the player's view.

## 3. The dispatch table

`Command.command_type`, one byte, 0x00–0x51. Sizes in bytes, including the
type byte; * marks a preceding group command on the issue side (the command
acts on the selection). Sample column: occurrences in the heavengames
recording's 21,884 packages.

| type | name | size | sample |
| --- | --- | --- | --- |
| 0x00 | group | 3 + 2·num | 1010 |
| 0x01 | begin | 1 | |
| 0x02 | stance* | 5 | 4 |
| 0x03 | form* | 0xd | |
| 0x04 | attack* | 0x11 | 80 |
| 0x05 | siege_attack* | 0xd | |
| 0x06 | swarm_around* | 0x11 | 9 |
| 0x07 | move_to* | 0x16 | 374 |
| 0x08 | move_near* | 0x1a | |
| 0x09 | attack_ground* | 0xa | |
| 0x0a | patrol* | 0xa | |
| 0x0b | launch_patrol* | 0x19 | |
| 0x0c | halt* | 1 | |
| 0x0d | transport* | 1 | |
| 0x0e | set_transport* | 5 | |
| 0x0f | board_ship* | 9 | |
| 0x10 | repair* | 0xd | |
| 0x11 | trade* | 0x15 | |
| 0x12 | city_gather* | 9 | 20 |
| 0x13 | gather* | 9 | 11 |
| 0x14 | garrison* | 0xd | 5 |
| 0x15 | disband* | 5 | 1 |
| 0x16 | gather_point* | 0x11 | 176 |
| 0x17 | spell* | 0x15 | 17 |
| 0x18 | queue_up* | 9 | 261 |
| 0x19 | build* | 0x19 | 31 |
| 0x1a | eject_all* | 0x11 | |
| 0x1b | alarm* | 1 | |
| 0x1c | flight* | 0x19 | |
| 0x1d | stop_spell* | 1 | |
| 0x1e | follow* | 0xd | |
| 0x1f | guard* | 0xd | 7 |
| 0x20 | unitmask* | 9 | 1 |
| 0x21 | buildmask* | 9 | 5 |
| 0x22 | hotkey* | 0x19 | 8 |
| 0x23 | recall* | 1 | |
| 0x24 | scramble* | 1 | |
| 0x25 | treaty | 0xd | |
| 0x26 | declare | 0xd | |
| 0x27 | clear_tributes | 9 | |
| 0x28 | clear_all | 9 | |
| 0x29 | accept | 9 | |
| 0x2a | reject | 9 | |
| 0x2b | tribute | 0x11 | |
| 0x2c | demand_tribute | 0x11 | |
| 0x2d | propose_attack | 0x11 | |
| 0x2e | buy | 0xd | 17 |
| 0x2f | sell | 0xd | 27 |
| 0x30 | unqueue | 0xf | 13 |
| 0x31 | come_out | 0xb | 4 |
| 0x32 | ping | 9 | |
| 0x33 | spline | 6 + 8·len | |
| 0x34 | speed_set | 5 | |
| 0x35 | speed_up | 1 | |
| 0x36 | speed_down | 1 | |
| 0x37 | mp_log | 1 | |
| 0x38 | check_random | 5 | |
| 0x39 | check_sums | 0x41 | |
| 0x3a | next_check_sum | 6 | |
| 0x3b | cheat_view_all | 5 | |
| 0x3c | cheat_give_techs | 5 | |
| 0x3d | cheat_zero_techs | 5 | |
| 0x3e | cheat_ai_speed_increase | 1 | |
| 0x3f | cheat_ai_speed_normal | 1 | |
| 0x40 | cheat_ai_toggle | 1 | |
| 0x41 | cheat_increase_buckets | 5 | |
| 0x42 | cheat_zero_buckets | 5 | |
| 0x43 | cheat_init_unit | 0x11 | |
| 0x44 | chat | 0x13 + 2·len | |
| 0x45 | chat_set | 9 | |
| 0x46 | resign | 5 | |
| 0x47 | quit | 7 | |
| 0x48 | camera | 0xa | 21044 |
| 0x49 | leader_options | 0x21 | 1 |
| 0x4a | turn_data | 0xb | |
| 0x4b | rename_city | 0x35 | |
| 0x4c | pause | 2 | 22 |
| 0x4d | cannon_time | 2 | |
| 0x4e | console_cmd | 0x209 | |
| 0x4f | player_speed | 9 | 2631 |
| 0x50 | ungraceful_player_drop | 3 | |
| 0x51 | marwan | 2 | |

An unknown type is an error ("Unknown net command received!!") and
`process` returns 0 — which would loop `process_all` forever but for the
package-size decrement going negative and tripping the mismatch error.

## 4. The layouts

All little-endian, packed, offsets from the command's type byte. Types below
are the PDB's: `int` i32, `Coord` i32 (world coordinates, `docs/ORDERS.md`),
`QueuePos` a 4-byte enum (the enqueue mode, `docs/ORDERS.md` §2),
`OrderIndex` a 4-byte enum. The full struct dump is reproducible from
`types.txt` (`struct /rise.pdb/<Name>Command`); the fields, tersely:

- **group** `[num u8][who i8][num × o i16]` — object indices under leader
  `who`, the issuing player's leader. `num = 0` means "the same selection as
  this player's previous group command" — the encoder emits the 3-byte form
  when the selection is unchanged (`last_*_sent` statics), and the decoder
  replays it from `last_objects_received[play]`/`last_uids_received[play]`,
  re-validating each object's uid (stale uid → silently dropped from the
  selection; on playback a missing object is the "broken replay" error).
  Captains drag their squad in on both sides; `unit_mark` bounds-checks.
- **stance** `[stance i32]`; **form** `[form i32][rotate i32][queued]`.
- **attack** `[ox i32][whom i32][ignore i32][queued]`; **siege_attack**
  `[ox][whom][queued]`; **swarm_around** `[ox][whom][queued][orders i32]`.
  `ox`/`whom` are the target's object index and owner throughout.
- **move_to** `[to_x i32][to_y i32][set_angle i32][angle i32][orders i8]
  [queued i8][form i8][width i8][disembark i8]` (0x16); **move_near** the
  same with `tolerance i32` third (0x1a).
- **attack_ground**, **patrol** `[to_x][to_y][queued i8]` (0xa);
  **launch_patrol** `[to_x][to_y][queued][shift i32][ctrl i32][alt i32]`.
- **halt**, **transport**, **alarm**, **stop_spell**, **recall**,
  **scramble**, **begin**, **speed_up**, **speed_down**, **mp_log**, and the
  three 1-byte cheats: type byte only.
- **set_transport** `[flag i32]`; **board_ship** `[ox][queued]`;
  **repair**, **garrison**, **follow**, **guard** `[ox][whom][queued]`;
  **trade** `[ox][whom][oxx i32][whose i32][queued]`.
- **city_gather** `[t i32][queued]`; **gather** `[ox][queued]`;
  **gather_point** `[x][y][action i32][add_to_end i32]`.
- **disband** `[all i32]`; **queue_up** `[type i32][num i32]`;
  **build** `[x][y][x2][y2][type][queued]`; **unqueue**
  `[who i32][o i32][type i32][uid i16]`; **come_out** `[who][o][uid i16]`.
- **eject_all** `[back_to_work i32][who i32][eject_o i32][eject_who i32]`;
  **flight** `[ox][whom][shift][ctrl][alt][orders OrderIndex]`.
- **spell** `[ox][whom][type i32][x][y]`.
- **unitmask**, **buildmask** `[mask i32][set i32]`; **hotkey**
  `[group i32][clear i32][valid i32][x f32][y f32][zoom i32]` — the two
  floats are camera state, renderer-only.
- **treaty**, **declare** `[who i32][whom i32][treaty i32]`;
  **clear_tributes**, **clear_all**, **accept**, **reject** `[who][whom]`;
  **tribute**, **demand_tribute** `[who][whom][good i32][amount i32]`;
  **propose_attack** `[who][whom][whose i32][onoff i32]`.
- **buy**, **sell** `[who i32][good i32][flags i32]`.
- **ping** `[x Coord][y Coord]`; **spline**
  `[spline_type u8][spline_flags u8][spline_cmd u8][len u16]
  [len × (x f32, y f32)]` — drawn annotations, renderer-only.
- **speed_set** `[speed i32]`; **check_random** `[seed u32]`;
  **check_sums** — 16 × u32, the per-subsystem checksums in
  `CheckSumsCommand`'s field order (units, builds, walls, ammo, deaths,
  groups, guys, leaders, cities, items, goods, world, rules, scenario_data,
  script_run_time, all); **next_check_sum** `[checksum_type u8][checksum u32]`.
- The 5-byte cheats: `[who i32]`; **cheat_init_unit** `[who][t][x][y]`.
- **chat** `[bits i32][taunt i32][taunt_num i32][len u32]
  [(len+1) × wchar]` — UTF-16, null terminator on the wire, `len` excludes
  it; `bits` is the recipient mask (−1 = all). Trimmed of spaces before
  send; in SP the receive side is also where chat cheats are parsed.
- **chat_set** `[status u8 × 8]`; **resign** `[play i32]`; **quit**
  `[play i32][replay u8][system_quit u8]`.
- **camera** `[zoom u8][x_loc i32][y_loc i32]`.
- **leader_options** `[who i32][peasants i32][peasants_wait i32]
  [buildings i32][BitMask<32> raw, 0x10 bytes]` — the auto-manage
  settings; the BitMask travels as its full in-memory struct
  (bits/size/flags/ptr, the PDB's `LeaderOptionData` +0x10 field type —
  the decompiler's `BitMask<8>` stack local is the same 16 bytes).
- **turn_data** `[ping_time u16][frame_average u16][wait_time u16]
  [game_lag u16][forced_loads u16]` — MP turn telemetry.
- **rename_city** `[who i32][o i32][name wchar × 22]` — fixed 0x35, the
  name field always fully present.
- **console_cmd** `[mouse_x Coord][mouse_y Coord][cmd wchar × 256]` — fixed
  0x209. ~~The `~` console travels in lockstep too.~~ Corrected by the
  second reading: 0x209 > `add_command`'s 0x200 cap, so
  `issue_console_cmd`'s append **always fails** — the dispatcher handles a
  command nothing can send, and typed console commands act locally only
  (`Console::on_key_down` emits `hotkey` commands, nothing else emits
  0x4e).
- **player_speed** — 8 × u8, the `accum_*` input-telemetry counters in
  `PlayerSpeedCommand`'s field order.
- **ungraceful_player_drop** `[play u8][state u8]` — injected by the net
  layer, exempt from §5's XOR.
- **pause**, **cannon_time**, **marwan** `[state u8]`.

## 5. The multiplayer obfuscation layer

Absent in single player (both gates are `semaphore.ptr[0] & 4`). Two
mechanisms, both keyed to `GameInfo.seed` — which every peer already shares:

- **Padding.** After appending each command, the encoder draws
  `Random::get(0, 2)` from a `Random` seeded with `info.seed`
  (`CommandPackage.padding`, re-seeded on every package clear) and grows
  `size` by that draw — 0 or 1 under the game's exclusive-upper `get`
  (`docs/COMBAT.md` §9.4) — garbage gaps between commands. The decoder
  draws from its own `Random` seeded identically at the top of
  `process_all` and skips the same gaps. One draw per command, encoder and
  decoder in lockstep.
  (`add_group`/`add_chat`/`add_spline` take the same draw.)
- **XOR.** `CommandPackage::send` builds the NetMsg (type 7:
  `[07][stamp u32][play u8][size u16][data]`) XORing each u16 of the
  payload with `(seed >> 8) & 0xffff`; a trailing odd byte is copied
  **unencrypted**. `process_all` undoes it in place before walking.
  Exempt: a package that is exactly one `ungraceful_player_drop`
  (`data[0] == 'P'`, size ≤ 4) when both bits 4 and 0x10 are set —
  `DropControl::process_drops` synthesises that package locally and writes
  it to the recording without ever passing `send`, so it is stored plain
  (§2).

Because MP recordings store packages as received (§2), both layers are
present in an MP `.rcx` and a reader must reverse them: XOR first (whole
u16s, odd tail byte plain), then walk commands skipping the seeded padding
draws. An SP recording has neither.

## 6. What is not established

- **MP-recording decode is derived, not exercised.** The XOR/padding
  reversal of §5 is read from `send`/`process_all` but the only sample is
  single-player; no MP `.rcx` has been decoded. The heavengames corpus
  (~245 recordings, `docs/RECGAME.md`) likely contains MP games if one is
  wanted.
- **Field semantics are pointers, not re-derivations.** Each `process_*`
  hands its fields to a documented mechanic (`Group::action_move_to`,
  `Build::train` …); where a field's meaning matters downstream, the
  mechanic's own document is the authority. The `QueuePos`/`OrderIndex`
  enum values are in `docs/ORDERS.md`.
- **`marwan` (0x51)** starts/stops something via a `MarwanCommand.start`
  byte (`process_marwan`, 0x943660) — an EE-era addition (Twitch/Marwan
  integration?); unread beyond its size.
- **`begin` (0x01)** is processed as a no-op frame marker
  (`process_begin`); per the no-issuer sweep above it is provably never
  emitted — a vestige.
- ~~Three unit commands (`move_near`, `board_ship`, `repair`) have no
  `CommandManager::issue_*` wrapper — they are built and appended directly
  by their UI/AI call sites.~~ Corrected by the second reading, and
  confirmed by a sweep of all 36 `add_command` callers outside
  `CommandManager`: **nothing emits them at all.** `move_near`, `repair`,
  `board_ship`, `begin` and `cheat_init_unit` are dispatched-but-never-
  issued — legacy or cut paths. Their wire formats stay pinned by their
  `process_*` sizes.
- `CommandPackage.group` and the embedded `Random` are per-process state a
  *recording* never stores (`write_package`'s 18-byte head puts `frame`
  where `group` sits — `docs/RECGAME.md` §4.3). A **save game** is
  different: `CommandPackage::walk_data` walks the 0x12-byte head
  `stamp, play, valid, group, size` — group included (second reading;
  confirmed against `CommandManager::walk_data`'s literal addresses).

## 7. Second reading — landed

The blind second reading ran the same day (Opus, isolated from this
document and the implementation; `docs/audit/2026-08-24-commands.md` holds
the adjudication). The dispatch table, sizes, layouts, selection model,
obfuscation layer and recording placement were all re-derived identically —
the table doubly derived on each side, four ways total. Four corrections
are amended inline above, each marked "second reading": `console_cmd` can
never be sent (§4), the save-game walker serialises `group` (§6), five
commands have no issuer at all (§6), and the `leader_options` tail is
`BitMask<32>` (§4). Adopted additions: bit 4's writer pinned to
`run_gamespy`, the self-decrypting-recording chain, and the `valid` mode
fingerprint (§2). One count went the first reading's way: `write_package`
has four textual call sites, not three — the extra one is the pump's
invalid-package error path, and nothing downstream changes.

## 8. What the sample established

The decoder over the heavengames sample (§ "How this was established"):
21,884 packages, 257,889 bytes of payload, 25,779 commands, zero errors,
every package consumed exactly. The stream's shape: `camera` on 21,044 of the run's
21,045 frames (playback's eye), `player_speed` telemetry every ~8th
package (2,631), and the actual game: 1,010 selections, 374 `move_to`,
261 `queue_up`, 176 `gather_point`, 80 `attack`, 31 `build`, 44
market transactions (17 `buy` + 27 `sell`), 22 pauses, one `disband`.
First package (frame 0, `valid = 1`): a lone `leader_options` — the
player's auto-citizen settings pushed at game start. Every stamp
contiguous, every payload plain — the SP path of §2 exactly as derived.
