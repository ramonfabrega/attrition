# lobby-words: second (blind) reading

Slug: `lobby-words`. Leaves: `cannon_time` (LeaderData +0x48, ulong), `handicap` (+0x4c, int), `multi_diff` (+0x50, int).
Reader: Claude Opus 5.5 (claude-opus-5-5), blind to the first reading.

Method: `->name` and `field_0x48/4c/50` over the export, then the plain
listing (`aa/exe.s`) for every absolute `0xe3a3d8`/`0xe3a3dc`/`0xe3a3e0`
displacement (slot-indexed stores into the global `leaders` array) and for
each candidate's own stores. `Leader` (0x6eec) holds `LeaderData` at +0
(the ctor `Leader::Leader (006d2bf0)` calls `LeaderData::LeaderData` on
`this` and writes its vbtable at +0x6ee4), so a `Leader*` offset equals the
`LeaderData` offset; no shift applies.

Shared context, re-derived here and used by all three leaves:

- **Setup order in a random-map game.** `Game::run (00584590)` calls
  `Game::init_rules_and_teams (00589bb0)` (call at 0x584885), which calls
  `Leaders::init (006ed850)` (0x589d04) and, near its end,
  `Game::init_teams` then `Game::init_handicaps (0058abf0)` (0x58a32d).
  Later `Game::init` → `Setup::build_game (005ac190)` (0x58d1b3) calls
  `Leader::init (006e3930)` once more per slot (0x5acef7), passing the
  slot's own `tribe` (`+0xc`).
- **`Leaders::init`** walks all ten slots (`0xe3a390`..`0xe7f8c8`, stride
  0x6eec) calling `Leader::init(slot, tribe = -1, slot)`.
- **`Leader::init`**: if `tribe < 0` (listing 0x6e3948 `test esi,esi; jns`)
  it `memset`s `[+0x0, +0x6929)` to zero (0x6e398c..0x6e3995) — which
  zeroes all three leaves — then, on both paths, stores `cannon_time`
  (0x6e3af3). With `tribe ≥ 0` (the `build_game` call for a seated slot)
  there is no clear, so `handicap` and `multi_diff` survive from
  `init_handicaps`.
- **`GameInfo` in `Game`** is at `Game+0xc`; its lobby bytes
  (`GameInfo_u_24_s_0`, 30 bytes) at `Game+0x24`, so `difficulty` is
  `Game+0x2b` and `cannon_times` `Game+0x33`; `player[8]` (stride 0x8c) at
  `Game+0x44`, so `player[0].flags` is `Game+0x74`. In `Player`: `flags`
  +0x30 (u16; bit 0 seated, bit 2 human — the bit `LeaderData::is_human
  (006ec170)` reads on `leader_flags`, and which `load_leader_chunk`
  mirrors between the two), `who` +0x33, `team` +0x34 (signed char),
  `handicap` +0x35, `diff` +0x38.
- **`Game::semaphore`** is a `BitMask<256>` at `Game+0x814`, bytes at
  `Game+0x820`. Bit 2 (`+0x820 & 4`) is the multiplayer bit (its gate on
  `issue_cannon_time` below; `docs/CENSUS.md` §385 names `run_gamespy` its
  one setter). Bit 26 is `+0x823 & 4`; I did not read what sets it.
- **Lobby defaults.** `GameInfo::default_setting (005d7200)` is a jump
  table over the 30 lobby bytes in struct order (table at 0x5d723c, read
  from the PE): index 7 `difficulty` → 0, index 15 `cannon_times` → 0.
  `PlayerProfile::default_profile (005ceca0)` fills the profile's
  `game_params` from it. A kept dump confirms the whole default row:
  `gamelog-run1-fulldump.txt`'s `GAME INFO` prints `DIFFICULTY 0`,
  `CANNON_TIMES 0` and every other byte equal to the table (TEAM_STYLE 1,
  MAP_STYLE 14, MAP_SIZE 2, …), the human's `PLAYER` block `flags 7 … team 8
  handicap 0 … diff 2`, the computer's `flags 1 … team 8 handicap 0 … diff 2`.

## Leaf 1: `cannon_time` (+0x48, ulong)

### 1. Writers

Export, `->cannon_time`: **no hit at all** (no reader prints the name).
`field_0x48` on a `Leader`/`leaders.list[]` base: `Leader::init (006e3930)`
and `TurnControl::start_cannon_time (00956500)`. Every other `field_0x48`
hit is on a different class (`Wall`, `City`, `Army`, `Group`, `Unit`,
`Build`, `Ammo`, `World`, `UnitBalance`); the `Leader::init` hit at
0x6e3f6a is `LeaderDataEncrypt+0x48` through the `+0x6eb8` pointer, not
ours.

Listing, absolute `0xe3a3d8(%reg)`: three sites —
0x9412f1 (`cmpl $0, …` — read, `CommandManager::issue_cannon_time
(00941260)`), 0x95658a (read) and **0x9565eb (store)**, both in
`start_cannon_time`, base `edi = who·0x6eec` (0x956584 `imul $0x6eec,%edx,%edi`).
Register-relative: **0x6e3af3** `mov %eax,0x48(%ebx)` in `Leader::init`,
`ebx = this` (`Leader*`, 0x6e3942 `mov %ecx,%ebx`).
Bulk: `LeaderData::walk_data (006d6750)` walks `[+0x8, +0x692a)` for a
seated slot; with `DataWalk::input != 0` (saved-game load) that restores
the leaf. `Leaders::init`'s path through `Leader::init` memsets it to 0
before the store.

| writer | store | value |
|---|---|---|
| `Leader::init (006e3930)` | 0x6e3af3 | `cannon_times.list[game.info.cannon_times].data[0]` |
| `TurnControl::start_cannon_time (00956500)` | 0x9565eb | `cannon_time − 1` |
| `LeaderData::walk_data (006d6750)` | bulk | the saved value (load only) |
| `Leader::init`, `tribe < 0` arm | memset | 0, immediately overwritten by the row above |

`Leader::Leader (006d2bf0)` does not touch +0x48.

### 2. Event, step, predicate

- **Seed.** `Leader::init` reads the lobby byte `Game+0x33`
  (`movzbl 0x33(%eax)`; `eax = GameAccess::game`, `0xc061ec`), multiplies
  by 0x58 (`sizeof(Category)`), adds it to `cannon_times.list.list`
  (`0xe800e0`) and loads `+0x3c`, `Category::data[0]`. Unconditional: both
  the `tribe < 0` and `tribe ≥ 0` paths reach it. `Categories::init
  (006f21d0)` stores each `<DATA>` through `String::convert_int` into
  `data[k]` (plain integer, no scaling). `Game::init_rules_data (0058ec60)`
  loads `cannon_times` with count 5; `data/rules.xml`'s `cannontimes`
  category gives the five rows `0, 1, 2, 4, 8` (None, 1, 2, 4, 8 Cannon
  Times). **Units: a count of uses.**
- **Step: −1 per use.** `start_cannon_time(who)` (from
  `CommandPackage::process_cannon_time (009454f0)`, from
  `CommandPackage::process`, i.e. command 0x4d off the order stream) steps
  it when **all** of: `who ≥ 0`; `turn_control.cannon_time_who < 0` (no
  cannon time already running; `cmpl $0,0x24(%esi)`); `cannon_time != 0`.
  On step it also sets `cannon_time_who = who`, `cannon_time_stamp =
  game.frame`, `cannon_saved_speed = speed`, `cannon_time_start =
  game.tick`, and if the speed is not 0 drops it to 0 with
  `set_new_target_time(200)`. With `cannon_time == 0` it only posts an
  "invalid order" feedback to the issuing console's player. No slot or kind
  is exempt beyond those three tests.
- **Issuer.** The one issuer, `CommandManager::issue_cannon_time
  (00941260)` (from `Options::exec`), builds command byte 0x4d only when
  `semaphore` bit 2 is set (`testb $4,0x820(%eax); jne`); otherwise it posts
  a message and returns. With bit 2 it also checks the local player's
  `cannon_time` as an **unsigned** `> 0` (`cmpl $0; ja`) before sending.
  No AI path calls it (no other caller of `issue_cannon_time` or
  `start_cannon_time` in the listing).

### 3. Reset

Every `Leader::init` re-seeds it from the lobby (`Leaders::init` and
`Setup::build_game` both, at game start; also each scenario/editor/CtW
leader build). Nothing in the frame loop resets it. `end_cannon_time` (from
`TurnControl::check_cannon_time`) clears `cannon_time_who`, not this leaf.

### 4. Readers

`start_cannon_time` (the zero test) and `issue_cannon_time` (the unsigned
local test). `LeaderData::log_data (006e5110)` does **not** print it. The
checksum walk reads it as part of `[+0x8, +0x692a)`.

### 5. Crate

No event: `crates/sim` has no cannon-time command, no `cannon_times`
lobby byte (`ai::Lobby` in `crates/sim/src/ai.rs` carries `difficulty` but
not `cannon_times`), and the leaf is listed only in
`crates/sim/src/checksum/leaders.rs` `LEADER_FIELDS`; `Sim::leader_long`
falls through to `None` for it. A builder would add a `cannon_times` byte
to `ai::Lobby` (parsed from the dump's `CANNON_TIMES`, rows `0,1,2,4,8`) and
return `cannon_times_row[lobby.cannon_times]` from `leader_long`'s match;
the decrement belongs with command 0x4d, which the crate does not port
(`crates/sim/src/sweep/` has `command_package_process_cannon_time` and
`turn_control_end_cannon_time` sweeps only).

### 6. Plain game

The seed is reached by every game. The step is **not**: its only issuer is
gated on `semaphore` bit 2 (multiplayer), and the AI never issues 0x4d. In
a solo skirmish the leaf is constant for the whole game at
`rows[CANNON_TIMES]`; with the default lobby byte 0 it is **0** for every
slot (seated or not). A solo player who picks "4 Cannon Times" in the lobby
(if the combo is offered in solo — not read) would hold 4, still never
stepped.

### 7. Confidence

High for the writers and the seed (listing-confirmed stores, rows from the
loader and the XML, default byte from the jump table and a dump). Not read:
whether the solo setup window shows the cannon-times combo; what else
might write `+0x48` through a `Leader*` held in a register I did not trace
(the register-relative net below +0x200 is noisy — I covered every
`Leader`/`LeaderData` method and every `leaders.list[...]`
`field_0x48` in the export, not every `0x48(%reg)` in `.text`).

