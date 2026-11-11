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

## 1. `attack_stamp` (`LeaderData +0xf4`, int[8])

| claim | R1 | R2 | status | verdict | deciding line |
|---|---|---|---|---|---|
| writer: `do_damage` capital heavy, `64ace6` | yes | yes | agree | **confirmed** | `64ab7e` gate on `capital_stamp[att]` (`0xe3a4c4`), `64ab8a jle 64ad11`; the store `64ace6` comes after the console message (`64ab9e jne 64acc7` lands at `64acc7`, before the store) |
| writer: plain city heavy, `64b037` | yes | yes | agree | **confirmed** | `64aeb2` (`subl 0x14(%edx)`, the city's `attack_stamp`), `64aeba jle 64b043` skips both leader stores |
| writer: light city arm (capital or not, and the worker arm), `64b383` | yes | yes | agree | **confirmed** | `64b201` (`subl 0x18(%ebx)`, the city's `raid_stamp`), `64b20c jle 64b38d` skips both leader stores; the message exit `64b220 jne 64b35b` lands before them |
| writer: territory attack branch, `64b9d8` | yes | yes | agree | **confirmed** | `64b77c`/`64b788 jle 64b9df`; the message exits `64b799 jne 64b9c6`, `64b7b4 jne 64b9c6` land on the store |
| writer: `Leader::init (006e3930)` zeroes all eight | yes (`6e3c12`) | yes | agree | **confirmed, unconditional** | `6e3b74` `esi = this+0xd4`; `6e3c12` `movl %eax, 0x20(%esi)` with `eax = 0`; back edge `6e3d90 cmpl $8, %edi` / `6e3d93 jl 6e3c0d`; the `param_2 < 0` test (`6e3c45`) comes after the stores |
| writer: `Leaders::clear (006ed5d0)` | yes | — | one-sided (R1) | **confirmed** | export `Leaders/clear` line 13: `memset(slot+0, 0, 0x6929)` per record (the pointer starts at `+0x6c14`, less `0x1b05·4`) |
| `LeaderData::walk_data (006d6750)` restores it on load | yes, as a writer | as the load path | agree | **confirmed** | walked range `[+0x8, +0x692a)` (`crates/sim/src/checksum/leaders.rs:6`, `docs/CHECKSUM.md` line 437) covers +0xf4 |
| README lead: `Leader::plan_strategy` and `end_process` write it | denied | denied (all stores in `do_damage`) | agree | **confirmed: not writers** | no `0xf4/0x114/0x154/0x200` displacement and no `0xe3a4xx` address in `006b9620..006bc190` or `006b8690..006b88b0` (my scan of `exe.s`) |
| no console or ally gate on the store | yes | yes | agree | **confirmed** | as the four rows above: every message exit jumps to the store |
| several arms on one blow: the territory test then sees `frame - frame = 0` and writes nothing new | yes | yes | agree | **confirmed** | `64b77c` reads the value the arm just wrote |
| no mid-game reset | yes | yes | agree | **confirmed** | absolute scan: only `do_damage` stores |
| reader: `do_damage` `64b77c` | yes | yes | agree | **confirmed** | `64b77c` |
| reader: `was_attacked (009fd290)` | "true when non-zero and `(frame-stamp)/15 <= seconds`; 0 seconds means ever" | "0 → false; `n > 0` → `/15 <= n`; a second form scans all live attackers" | partial | **corrected (R1 incomplete)** | export `ScenarioFuncSet/was_attacked` lines 15–50: slots are 1-based (`*param - 1`); a victim outside 0..7 or not live (`+0` word without bits 1 and 2) answers −1. If the attacker slot is in 0..7 and live: stamp 0 → 0; `n > 0` → `(frame - stamp)/15 <= n`; `n <= 0` → 1 (not only `n == 0`). Otherwise it scans attackers 0..7, live, not the victim, with a non-zero stamp, and answers 1 on the first that passes (`n == 0`, or within `n`); a negative `n` in that form matches nothing |
| reader: `LeaderData::log_data (006e5110)` | line 288 | ~287 | agree | **confirmed** | export line 288, `piVar3[0x20]` off `diplos` (+0x74), so +0xf4 |
| reader: the checksum walk | yes | yes | agree | **confirmed** | `crates/sim/src/checksum/leaders.rs:61` |
| no AI or economy reader | yes | yes | agree | **confirmed** | absolute scan; an indexed `0xf4(%r,%r,4)` access in the whole `.text` occurs only in `BuildTypeOut::set_parse_data`, `Form::compute_dests`, `Wall::update_hits` and the two `process_turn_data`, none off a leader |
| crate: no row; `checksum/leaders.rs:61` lists it; `leader_image` has no arm | yes | yes | agree | **confirmed** | `leaders.rs:61`; `"capital_stamp"` arm at `:998`; `_ => None` at `:1305` |
| crate: arms 1–3 in `Sim::city_alarm` (`delivery.rs:280`), the territory step new in `Sim::do_damage_round` (`delivery.rs:70`) after the `city_alarm` call at `:190`, before the take at `:192` | yes | yes | agree | **confirmed** | `delivery.rs:190`, `:192`, `:280` |
| crate: row beside `AiLeader::capital_stamp` (`ai.rs:704`) | yes | yes | agree | **confirmed** | `crates/sim/src/ai.rs:704` |
| crate: `city_alarm` returns early on a non-city building, a city-less one and a unit, but the territory step must run for every cross-owner blow | yes | yes | agree | **confirmed** | `delivery.rs` `city_alarm` returns at the worker branch, `_ => return false` and `let Some(c) ... else { return false }`; the original reaches `64b56d` from all of them |
| crate: the light arm has no `frame - c.raid_stamp > 300` gate today (it writes the city's stamp unconditionally, which is right), and the leader stores need that gate | — | yes | one-sided (R2) | **confirmed** | `64b20c jle 64b38d` skips only the leader stores; `64b398` writes the city's +0x18 on both paths |
| plain game: yes | yes | yes | agree | **confirmed** | no semaphore or mode test on any of the four arms |

**The rule for a builder.** `leaders[vic].attack_stamp[att]`, sim frames,
initial 0, zeroed by `Leader::init`. For every blow with `att != vic` that
passes the head returns, before the take, the four arms below in this
order. `heavy` is computed for any target with `obj_flags & 1`, `is_build`
and `is_active`: it is the attacker's type being siege, or
`damage + whole >= (is_defense || flags & 0x20 ? hits(0)/10 : hits(0)/4)`,
with signed truncating division. It is computed before the city test.

1. A city building of `vic`, heavy, capital: if `frame - capital_stamp[att] > 300`, write `attack_stamp[att]`, `raid_stamp[att]` and `capital_stamp[att]` = `frame`.
2. A city building of `vic`, heavy, not the capital: if `frame - city.attack_stamp > 300`, write `raid_stamp[att]` and `attack_stamp[att]` = `frame`. Then, always, set `city.attack_stamp` and `city.raid_stamp` to `frame`.
3. A city building, light (capital or not), or a worker whose gather order names an active building of `vic`'s in a city: if `frame - city.raid_stamp > 300`, write `raid_stamp[att]` and `attack_stamp[att]` = `frame`. Then, always, set `city.raid_stamp` to `frame`.
4. Every blow, after 1–3: take `land` = the `who` of T's cell. If `land != vic`, or T is not (a building with heavy 0, or a worker, caravan or merchant), and `frame - attack_stamp[att] > 300`, write `attack_stamp[att] = frame`.

The crate site is `city_alarm` for 1–3, plus a new territory step in
`do_damage_round` between `delivery.rs:190` and `:192`. The step also
covers the spill. A plain game reaches it. **Confidence: high**: every
store, gate and exit was read in the listing. **Open:** a gaia striker's
column 8 (§0).

## 2. `raid_stamp` (`LeaderData +0x114`, int[8])

| claim | R1 | R2 | status | verdict | deciding line |
|---|---|---|---|---|---|
| writer: capital heavy `64acf8`, same gate as `attack_stamp` `64ace6` | yes | yes | agree | **confirmed** | `64ab7e`/`64ab8a`; `64acf8` |
| writer: plain city heavy `64b025`, gate on the **city's** `attack_stamp` | yes | yes | agree | **confirmed** | `64aeb2`; `64b025` stores before `64b037` |
| writer: light city / worker arm `64b373`, gate on the city's `raid_stamp` | yes | yes | agree | **confirmed** | `64b201`; `64b373` stores before `64b383` |
| writer: territory raid branch `64b755`, gate on this row | yes | yes | agree | **confirmed** | `64b64a`/`64b656 jle 64b9df`; message exits `64b66a jne 64b743` and `64b684 jne 64b743` land on the store |
| no city arm stamps one of `raid_stamp`/`attack_stamp` without the other | implied | stated | agree | **confirmed** | the three city-arm store pairs `64ace6`+`64acf8`, `64b025`+`64b037`, `64b373`+`64b383` share their gates |
| the raid and attack branches are exclusive on one blow | yes | yes | agree | **confirmed** | `64b75c jmp 64b9df` after the raid store skips the attack test |
| raid branch predicate (land == `vic` and a light or inactive building, a worker, a caravan or a merchant) | yes | yes | agree | **confirmed** | §0 territory row |
| a city-less active building keeps its own heavy; an inactive building, or one with `obj_flags & 1` clear, is light here | yes | yes | agree | **confirmed** | heavy is set at `64aa2e`/`64aa43`, before the city test `64ab3a js 64b56d`; the inactive path `64a99b je 64aa56` never sets it |
| a wall in its owner's land takes the attack branch | open (`+0xd0` on a wall) | yes | one-sided (R2) | **confirmed** | §0, `Wall::vftable` `+0x20` and `+0xd0` both `return 0` |
| writer: `Leader::init` `6e3c15` (`0x40(%esi)`) | yes | yes (`puVar23[0x10]`) | agree | **confirmed** | `6e3c15`, inside the 8-pass loop |
| writer: `Leaders::clear`; restored by `walk_data` | yes | load path only | agree / one-sided | **confirmed** | as in §1 |
| reader: `was_raided (009fd140)`, the same shape as `was_attacked` | yes (body unread) | yes | agree | **confirmed** | export `ScenarioFuncSet/was_raided` lines 15–31: the same body, reading `+0x114`; the §1 correction (`n <= 0` → true when the stamp is non-zero) applies to both |
| reader: `log_data` `piVar3[0x28]` | line 300 | yes | agree | **confirmed** | export line 300 (`0x74 + 0xa0 = 0x114`) |
| crate: `city_alarm` already writes the city's `raid_stamp` | yes | yes | agree | **confirmed** | `delivery.rs`, the worker branch, the `!heavy` branch and the plain-heavy tail |
| crate: `is_worker`, `is_merchant` (`calc_gather.rs:555`), caravan predicates exist | yes | `is_merchant` unchecked | one-sided (R1) | **confirmed** | `crates/sim/src/economy/calc_gather.rs:555` (`MERCHANTS` at `:40`); `is_caravan` on the unit adapter (`crates/sim/src/units/set_type_adapter.rs:94`); `Sim::is_worker` is the one `city_alarm` already calls |
| crate: a tile-owner lookup is needed | `World::owner_at` (`world.rs:2013`) | "needed" | one-sided (R1) | **confirmed** | `crates/sim/src/world.rs:2013` → `owner()` returns `self.who[i]`, the cell's `who` (not `who2`), which is the field `64b5c4` reads (`+0xf`). `Owner::None`/`Ambiguous` never equal a player, which matches the signed compare against `vic` |
| plain game: yes | yes | yes | agree | **confirmed** | no mode test |

**The rule for a builder.** `leaders[vic].raid_stamp[att]`, sim frames,
initial 0, zeroed by `Leader::init`. It is written in arms 1–3 of §1,
always together with `attack_stamp[att]` under the same gate. In the
territory step it is written instead of the attack test: when
`land == vic` and T is (a building with heavy 0, an inactive or flag-less
building included), a worker (type 0x32..0x35), a caravan or a merchant
(0x3d, 0x3e, 0x190), and `frame - raid_stamp[att] > 300`, then
`raid_stamp[att] = frame`. The site is the same as §1. A plain game
reaches it. **Confidence: high.** **Open:** gaia's column 8 (§0).

