# victim-stamps — adjudication (wave 3)

- Item 4152, family 4, slug `victim-stamps`. Leaves: `attack_stamp` (+0xf4 int[8]), `raid_stamp` (+0x114 int[8]), `ally_stamp` (+0x154 int[8]), `ctw_hero_stamp` (+0x200 int).
- Adjudicator: Claude Opus 5.5 (`claude-opus-5-5`), 2026-11-11.
- Inputs: `victim-stamps-reading.md` (first, R1) and `victim-stamps-second.md` (blind second, R2); the decompile export; the listing `aa/exe.s` (llvm-objdump of `.text`).
- Written per leaf as settled.

## 0. Shared ground, settled once

The two readings agree on the whole skeleton. Each shared claim was re-read
in the listing, and the verdicts below cite it.

| claim | R1 | R2 | verdict | deciding line |
|---|---|---|---|---|
| `Leader` has `LeaderData` at +0, slot `i` at `0xe3a390 + i·0x6eec` | yes | yes | **confirmed** | `64a8e7`/`64a8f9` (`imul 0x6eec; add 0xe3a390`) |
| the only game-time writer is `Object::do_damage (0064a480)` | yes | yes | **confirmed** | my own absolute-address scan of `exe.s` for all four leaves in slots 0..8: 18 hits in `do_damage` (`64ace6`..`64b9d8`) and 4 loads in `ScenarioFuncSet::was_raided (009fd140)` (`9fd19f`, `9fd21f`) and `was_attacked (009fd290)` (`9fd2ef`, `9fd36f`), nothing else |
| register-relative net finds no other store | yes | yes | **confirmed, with the net's limit stated** | my scan: indexed stores `0x(f4\|114\|154)(%r,%r,4)` in the whole `.text` are only `CommandPackage::process_turn_data` `943e52` and `TurnControl::process_turn_data` `956a85`, both off `[0xc06180]` beside a `0x168` bitmask, not a leader. Plain `0xNN(%r)` stores in Leader/Scenario/Conquest/Setup/Game functions: `ScenarioRead::load_leaders_chunk (009acde0)` `9ace7b`/`9acebe`/`9acf86` write `game->info.player[i]` (export line 28 on), `ScenarioRead::import (009b1430)` `9b1527`/`9b2090` write `[0xe7f8dc]+0x200`, `ScenarioEditor::init (009a4510)` `9a4984` writes its own `this`. None is a `LeaderData`. A store through a base other than the record start (as `Leader::init` uses `this+0xd4`) is outside this net, as it was for both readers |
| `this` = striker A, owner byte `A+0x9` = attacker `att`; `param_2` = victim `vic` | yes | yes | **confirmed** | `64ab6b` (`movzbl 0x9(%edi)`), `64a915` (`0xc(%ebp)`) |
| every stamp is `leaders[vic].X[att]`, index `att + vic·0x1bbb` | yes | yes | **confirmed** | `64a95d`/`64a965` saves `vic·0x1bbb` in `0x24(%ebp)`; each store adds `movzbl 0x9(%edi)` (e.g. `64ab6b`+`64ab77`) |
| the column is the raw owner byte, not `leaders[att].who` | implied | implied | **confirmed** | the stamps use `movzbl 0x9(%edi)`; only the unrelated `+0x94` flag store at `64a968` goes through `leaders[att]+0x8` |
| head exits: `count < 1` (`64a4ae`), decoy striker (`64a4dc`), `vic == att` (`64a932`) | yes | yes | **confirmed** | the only jumps out of `64a480..64a932` that skip the block are `64a4ae jle 64c86b`, `64a4dc jne 64c86b`, `64a934 je 64b9df` |
| the block runs before `take_damage`, so a killing blow stamps | yes | yes (before `take_damage_typed`) | **confirmed** | the block ends at `64b9df`, the take's setup; no branch inside it depends on the take |
| value stored is `game->frame` (`[0xc061ec]+0x550`) | yes | yes | **confirmed** | e.g. `64ae82`/`64ae87` |
| window `0x12c` = 300, strict `frame - stamp > 300` | yes | yes | **confirmed** | `cmpl $0x12c; jle` at `64ab85`, `64ad23`, `64aeb5`, `64b07d`, `64b207`, `64b651`, `64b783` |
| `heavy` (`-0x30`) zeroed at `64a946`, set only on the active-building path | yes | yes | **confirmed** | `64a946`; set at `64aa2e`, `64aa43` |
| `heavy` = siege term, or `T.damage(+0x24) + whole >= ceiling`, `ceiling = hits(0)/10` when T's type answers `+0x104` or `T.obj_flags & 0x20`, else `hits(0)/4`, both signed toward zero | yes | yes | **confirmed** | `64a9bc`/`64a9d3` select; `64a9e3..64a9ea` (`cltd; and 3; add; sar 2`); `64aa02..64aa11` (`0x66666667`, `sar 2`, sign add); `64aa23`/`64aa28 jl` |
| `whole` (`-0x24`) is this blow's scaled whole hits after the `>>4` split | yes | yes ("this_hit") | **confirmed** | `64a78a`/`64a78d`, `64a7ee`/`64a7f1`; no store to `T+0x24` anywhere in `64a480..64a932`, so the damage is as it stood before the blow |
| siege term = A's type answers `+0x10c` **and** T answers `+0x1c` (`is_wallbuild`) | yes | yes | **confirmed, and simplified** | `64a8b8`, `64a8d3`. `is_wallbuild` is 1 on every Build and Wall (`vtables.txt`: `Build::vftable` and `Wall::vftable` `+0x1c` are the folded `Buffer::is_pending_load (0041e0e0)`, `return 1`), so on the building arm the siege term reduces to "A's type is siege" |
| building arm: `T.obj_flags & 1`, `is_build` (+0x20), `is_active` (+0x4c), then `BuildData+0x72` city ≥ 0 | yes | yes | **confirmed** | `64a97a`, `64a986`, `64a996`, `64ab34`/`64ab3a` |
| worker arm: T `is_unit`, `is_worker` (type 0x32..0x35), the gather order (order vslot +0xe4) names an object of owner == `vic` at a non-negative index that `is_active_build`, in a city → the city's light arm | yes | yes | **confirmed** | `64aa65`, `64aa79` (`46fa10`..`46fa28`), `64aa86`, `64aa97`, `64aab3`/`64aabb`, `64aadc`, `64aaec cmp 0xc(%ebp)`, `64ab0f`. Heavy stays 0 on this path, so it always reaches `64b1f0` |
| a non-worker unit → CtW hero test; anything else → territory step | yes | yes | **confirmed** | `64aa6a je 64b56d`, `64aa80 je 64b3a0` |
| capital = `city_flags(+4) & 0x10` | yes | yes | **confirmed** | `64ab52` |
| territory step: signed `wdata[...]+0xf` under T's position | yes | yes | **confirmed** | `64b5b2`..`64b5c4` (`movsbl 0xf(%eax,%ecx,4)`) |
| raid branch when `land == vic` and ((T `is_build` and heavy == 0), or `is_worker`, or vslot +0xd0, or `is_merchant`); else attack branch | yes | yes | **confirmed** | `64b5cc jne 64b761`; `64b5dc`/`64b5e7 je 64b630`; `64b5f8`, `64b603`, `64b61f`; `64b62b jmp 64b764`. `is_worker (0046fa10)` leaves `ecx` alone, so the `+0xd0` call is on T |
| `is_merchant (0046d370)` = type id 0x3d, 0x3e or 0x190 | yes | named only | **confirmed** | `46d376`..`46d385`; the crate's `MERCHANTS` (`crates/sim/src/economy/calc_gather.rs:40`) matches |
| callers: `Unit::fight` ×3, `Ammo::do_damage` ×3, `Nuke::do_damage` ×2, itself ×1 | yes | yes | **confirmed** | `calll 0x64a480` at `5fedda`, `5ff1fb`, `5ff33d`, `678982`, `678b0a`, `678c6c`, `92c3ab`, `92c650` (both inside `Nuke::do_damage (0092bc80)`), `64c4b7` |

**Settled where one reader left it open.**

- **A wall target** (R2 claimed it, R1 left `+0xd0` on a wall open):
  **confirmed, attack branch**. `Wall::vftable` `+0x20` (`is_build`) and
  `+0xd0` are the folded `Window::get_button (0041bff0)`, `return 0`
  (`vtables.txt`, the `Wall::vftable` block). A wall never enters the
  building arm and, on its owner's land, is neither a light building nor
  a worker, caravan or merchant. The same folded `return 0` is `Build`'s
  `+0xd0`, so a building's caravan test answers 0, as R1 assumed.
- **Which object is `this` on the `Ammo::do_damage` path** (R2 open):
  **settled: the shooter**, not the projectile. `Ammo::do_damage`'s call
  passes `objects[ammo+0x3c].list[ammo+0x40]` as `this` (the export's
  `Ammo/do_damage` file, line 130). So `att` is the owner byte of the
  object that fired.
- **The spill onto builders** (`64c4b7`, after the take): both readers
  list it as a caller, but neither says what it does to the stamps. It
  re-enters `do_damage` and meets only the three head exits, so **each
  spilled hit runs the stamp block for its own target's owner**. The
  crate's `spill_onto_builders` calls `do_damage_round`
  (`crates/sim/src/combat/fight/delivery.rs`, about line 435), so a
  territory step placed in `do_damage_round` covers the spill.

**A finding neither reader made: gaia as the attacker (open).** All three
arrays are `int[8]`, and the column is the raw owner byte. The colour list
in `docs/ORACLE.md` ("`RED ... ORANGE GAIA`", parse_who) puts gaia at slot 8,
and `Leaders::clear (006ed5d0)` walks 10 records (`puVar1 < 0xe864dc`, from
`+0x6c14` of slot 0, stride `0x6eec`). So a gaia-owned striker would write
column 8: `attack_stamp[8]` is `raid_stamp[0]`, `raid_stamp[8]` is
`capital_stamp[0]`, and `ally_stamp[8]` is `tribute_stamp[0]`. Row 8 (gaia
as the victim) is in bounds. Nothing here shows that a gaia object ever
calls `do_damage` in a plain game. **To settle it:** grep a capture's
`DeathsSync` or `UnitsSync` for damage dealt by `who 8`, or read whether
any gaia type has an attack. Until then a port should take `att` in 0..7
and assert it.

