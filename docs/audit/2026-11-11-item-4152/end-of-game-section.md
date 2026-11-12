## 10. The end of the game

Defeat, victory and the wonder clock. `Leader::defeat (006ecb00)` and
`Leader::victory (006ec9b0)` write four leaves; the victory pass writes the
wonder pair.

### 10.1 Leaves

- `defeated_by`, +0x10, `int`: the captor's slot, −1 when none.
- `wonderwin_stamp`, +0x448, `int`: frames; the start frame while running,
  minus the banked time while stopped.
- `wonderwin_timer`, +0x44c, `int`: a 0/1 running flag.
- `victory_type`, +0x7d8, `int`.
- `defeat_type`, +0x7dc, `int`.
- `defeat_stamp`, +0x6924, `int`: a frame.

`defeated_by` is −1 and the rest 0 from `Leader::init (006e3930)`; under
`tribe < 0` the wonder pair is zero by the memset alone (`init` returns
before its explicit stores; same value). Nothing else resets any of them.

### 10.2 The rule

**`defeat(who, type, by, instant)`**, in this order:

1. **Captor stamp, at entry** (`Leader::defeat_by (006d1c80)`, its only
   caller is `defeat`). If `defeated_by[who] >= 0` (signed), skip this
   step. Otherwise `defeat_stamp[who] = frame`; then, if `by >= 0`,
   `defeated_by[who] = by` and, unless `elimination == 3`, the captor arm
   (`capture_city`) runs. A type ≠ 5 blow-up arm precedes the stamp and
   does not touch it.
2. **Guards.** Return if `flags & 0x40` (already defeated) or
   `semaphore[1] & 2` (balance mode).
3. `flags2 & 0x200` ← `instant`; `flags = (flags & ~2) | 0x40` (bit 0
   stays); `Armies::leader_defeated`; `flags2 |= 2`; `musical_chairs =
   frame`; `defeat_type[who] = type`.

So `defeated_by` and `defeat_type` are write-once; `defeat_stamp` is
re-stamped by every later call while `defeated_by` stays −1, even on a
defeated leader, which can also still receive `defeated_by` and run the
captor arm.

**Callers:**

| caller | type | by |
|---|---|---|
| `lost_a_city`, by `elimination` (`game+0x37`): 2 → 2 when `param_1 != 0`, unless tribe bonus `0x17` and the loser still holds its own capital; 3 → 3 when `param_2 != 0`; any other mode (0, 1, …) → 0 when `flags2 & 1` clear and cities (+0x3f8) 0 | 2 / 3 / 0 | the captor (may be −1) |
| `lost_a_city` GG arm: `param_1 != 0`, captor `>= 0`, `elimination == 1`, no wonder `0x213`, and some slot with `flags & 3 == 3` that is the captor or that the captor counts as ally holds preq `0x2b9` | 1, `instant` 1 | the captor |
| `Leader::process_elimination` (elimination 1; its by-self arm is dead) | 1 | `LeaderData::find_capital (006eb930)`'s holder; **self** when no live capital-flagged city is held by anyone (razed) |
| `Leader::gain_tech (006dcb60)`, when the gained tech is the `0xae4` bonus's preq: every slot `j ≠` gainer with `LeaderData::has_capital (006d54c0)` 0 whose holder is the gainer or the gainer's ally. `has_capital` is 1 unless `elimination == 1`; a foreign holder needs `j`'s `flags & 3 == 3` | 1, `instant` 1 | the gainer |
| `leave_game` with the capital timer (+0x418) running: under elimination 1 the `find_capital` holder (may be self), else self | 1 | holder / self |
| `leave_game` otherwise | 6 resign, 7 drop | −1 |
| `Unit::close (0060ee50)`: the **last** unit's close (count decremented first), owner `< 8`, units (+0x93c) 0, cities 0, `semaphore[1] & 8` and `leader_flags2 & 1` clear; no defeated test | 0 | −1 |
| the victory cascade | 5 | −1 |
| musical chairs (`GameDaemon::process_victory (00730ef0)`) | 4 | −1 |
| armageddon (`Game::defeat_all`) | 8 | −1 |
| the scenario | 9 | −1 |

No code writes `defeat_type` 10; it reaches the field only through
`LeaderData::walk_data (006d6750)`. In multiplayer `leave_game` defeats no
one when another seated human shares the leaver's `who`.

**`victory(who, type, instant)`.** Return if `flags & 0x60` (won or
defeated). Set `0x20`, `flags2 & 0x100` ← `instant`, `victory_type[who] =
type`. Then, for every other slot with `flags & 3 == 3`: set `0x80`; if
that slot counts `who` as ally (`is_ally(that, who)`), `victory(type,
instant)` for it, else `defeat(5, −1, 0)`. A prior winner not allied to a
new winner can be defeated, since `defeat` tests only `0x40`. Pair
`victory_type` with the won flag: 0 is also conquest.

Type by caller: 0 conquest (`Game::check_victory (005926b0)`,
`Leader::set_diplo (006ec6a0)`, `TurnControl::toggle_pause (00957ab0)`),
1 wonder, 2 territory, 3 `gain_tech` under VICTORY 9, 4–6 the other arms
of `process_victory`.

**`check_victory`'s gate.** Return if `semaphore[0] & 0x40`. Count seated
humans and live humans (`flags & 0x43 == 3`). If a human lives, or there
are none, or balance mode: return unless every live leader is mutually
allied (`diplos == 2` both ways) with the first live one. Then latch
`semaphore[0] |= 0x40` and compute

    A = armageddon_per_team·num_sides + armageddon_per_nation·num_nations + armageddon

(×3 at starting resources 7, ×2 at 5, 6, 11, 12, floor 100 at 8). Only if
`game.armageddon < A`: walk slots with `flags & 3 == 3`, setting `0x80`;
the first undefeated is the candidate; if any later live, not-won slot is
not mutually allied with it, set `semaphore[2] |= 0x40` and declare no
winner; else `victory(candidate, 0, 0)`. All humans dead and the computers
unallied ends the game with no `victory_type`.

**The wonder arm**, in `process_victory`, every frame under `VICTORY` ∈
{0, 1, 6}, with `W = Game::wonder_winning (005948a0)` read once. For slots
0..7 with `flags & 3 == 3`, in order:

- **Off side** (`W < 0`, or `i ≠ W`, `who_i ≠ W` and not mutually allied
  with `W`): if the flag is set, clear it and `stamp −= frame`.
- **On side.** `gg` = some slot of all ten with `flags & 1` (bit 0 only,
  so a defeated teammate counts) that is `i`, has `who == i`, or is
  mutually allied with `i`, holds preq `0x2b9`.
  - `VICTORY != 0`, or (not multiplayer and `num_nations <= 1`), or `gg`:
    if no earlier on-side slot was handled this frame, `victory(i, 1, gg)`.
  - Else flag clear: set it, `stamp += frame`, event 9.
  - Else, if no earlier on-side slot was handled: `remaining =
    wonder_timer() − (frame − stamp)`; `victory(i, 1, 0)` iff
    `remaining == 0` **exactly** (the territory arm wins on `<= 0`; do not
    copy it).
  - Mark handled. The walk never stops early.

`Game::wonder_timer (005944f0)` = 0 if `constants+0xd08` = `T` (`WONDER_TIMER`)
`< 1`, else `max(1, (M/2 + world[0]·T) / M)`, signed, `M/2` the signed
halve, `M` the current `map_sizes` row's +0x144. At 0 the clock never wins.

**The pay-back** (`Leaders::process_all (006ed2a0)`, inside its `flags & 2`
slot gate, after `process_elimination` and the +0x414 and +0x440
pay-backs): if `TIMER_REFRESH_RATIO` (`constants+0xd00`) ≠ 0, `frame %
ratio == 0`, `wonderwin_stamp < 0` and the flag is 0: `stamp += 1`.
`Leader::process (006b88b0)`'s copy is dead.

**Readers that change a decision.** Only `wonderwin_timer`:
`Army::find_target` (×10 and `about_to_win`), `Leader::diplomacy`,
`Game::is_victory_timer (00594950)`, `Wall::process (00640450)` (a rival
not allied to the owner with the flag adds one). The rest feed UI, stats
and the save.

### 10.3 In the crate

- `Sim::defeat(who, by)` (`crates/sim/src/cities/city.rs:623`) carries no
  type or instant, and returns at once on `leaders.defeated[who]` (:624).
  The port puts step 1 ahead of that return; `defeat_type` goes after it.
- `Sim::resign` (`crates/sim/src/leaders/resign.rs:43`) is type 6. Under
  a running `lost_capital_timer` the original passes 1 with `by` = holder
  or self: a plain-game divergence. It kills only planes, and the crate
  has no `Unit::close` last-unit `defeat(0, −1, 0)`, so a resigned
  leader's re-stamp is missing.
- No `Sim::victory`. The SEAM at `crates/sim/src/territory/victory.rs:21`
  says the game never ends by territory; five captures contradict it.
  `victory(2, ·)` from `process_victory_territory` (:31) is the first port.
- The wonder arm is a new `process_victory_wonder` in `core/frame.rs`
  before `process_victory_territory` (`crates/sim/src/core/frame.rs:222`),
  using `Sim::wonder_winning` (`crates/sim/src/ai/ai_build.rs:1439`); the
  pair sits beside `popwin_*` in `crates/sim/src/ai.rs`, with
  `WONDER_TIMER`.
- **Trap:** `refresh_victory_timers` (`territory/victory.rs:88`) is called
  for every player (`core/frame.rs:94`, :194) with no alive gate, so a
  defeated leader's banked `popwin_stamp` is paid back where the original
  skips it. Add the gate for both pairs.
- **Trap:** the `w44c` stub returns 0 (`crates/sim/src/ai/army/target_gate.rs:428`);
  30 captures hold a live 1. `Wall::process`'s SEAM is
  `crates/sim/src/cities/site_recruit.rs:157`.
- `crates/rondata/src/diff/third_pair.rs:432` calls run600's 17380 a quit;
  the dump shows `defeat_type` 5, the cascade.
- Checksum: the six leaves are listed in `crates/sim/src/checksum/leaders.rs`
  (:39, :113, :114, :144, :145, :310) and `leader_long` (:949) answers them
  through its catch-all (:1305); `popwin_*` (:1060) is the model. Each is
  an `(unheld)` entry, count 2, of `LEADERS_HELD_OUT`
  (`crates/rondata/src/diff/checksum_lines.rs:1753`; read from the code).

### 10.4 Reached by a plain game

The captures' lobby is elimination 1 and VICTORY 0 (run600's dump line
4727; the crate's default, `crates/sim/src/ai.rs:1132`). So a skirmish
against the computer reaches: `lost_a_city` type 0 and its GG arm,
`process_elimination`, the `gain_tech` GG arm, `Unit::close`, resign (6)
and its capital-timer arm (1), the victory cascade (5), victories 0, 1
and 2, the wonder clock's start, and `check_victory`. Behind lobby
options: `lost_a_city` types 2 and 3 (elimination 2, 3), armageddon (8),
VICTORY 9's type 3. Behind the scenario: type 9. Multiplayer: drop (7)
and the shared-`who` arm. Type 10: a load only.

### 10.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/end-of-game-*.md`); no diff backs it
yet. Counts (confirmed / corrected / open): `defeated_by` 13/3/1,
`defeat_stamp` 10/0/1, `victory_type` 10/1/1, `defeat_type` 9/2/1,
`wonderwin_timer` 14/2/1, `wonderwin_stamp` 8/0/1. Every store, guard and
push was read in the listing; confidence is high throughout.

Corrections that changed a rule: `process_elimination`'s `by` falls back
to self; the `gain_tech` arm exists only under elimination 1, with the
gainer's view of alliance; elimination 1 is the plain game, so the type-1
arms are plain-game; `check_victory` can end with no winner; `defeat_type`
10 has no writer; `gg` tests bit 0, counting defeated teammates.

Witnesses from an archive sweep of 3,805 dumps, **not pinned diffs**:
run600's closing frame 17380, slot 1 `victory_type` 2 and slot 0
`defeat_stamp` 17380, `defeat_type` 5, `defeated_by` −1 (the cascade);
the same shape in run972b, run981b, run1256, run1316. run1200: slot 0
`defeated_by` 1, `defeat_stamp` 5639, `defeat_type` 0. run24: slot 1
`defeated_by` 0, 3363 frames after its capital fell (the shape of
`process_elimination`). `defeat_type` 6 in the start captures, 9 in
run1113/1114. `wonderwin_timer` 1 in 30 captures (run652 … run1423c).

### 10.6 Not established

- **The stop, the banked stamp, the pay-back** (and the crate's missing
  alive gate): no capture holds a negative `wonderwin_stamp`.
- **A wonder win.** No `victory_type` 1 in the archive; nor `M`'s row
  (`0xe7fca8`) and `world[0]` (`*0xc06188`) in `wonder_timer`. A reading
  of both, or a capture that runs the clock out.
- **A re-stamp.** No file shows two `defeat_stamp` values.
- **Resign under a capital timer.** No `defeat_type` 1 in the archive; it
  would show `defeated_by` = the quitter's own slot.
- **The territory expiry's frame** for run600 (from `popwin_stamp` 14293)
  is the territory family's arithmetic; `Game::popwin_timer` for its map
  is unread.
- **`lost_a_city`'s `param_1`, `param_2` and captor** are derived by its
  callers (`Build::close`), owned by §11.
- **The pins:** run600's closing block and run1200's loss, as
  `rondata::diff` assertions once the leaves are held.
