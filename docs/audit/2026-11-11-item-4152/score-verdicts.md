# Item 4152, family 18 (`score`) — adjudication

- Family 18, slug `score`; the twelve leaves `score` (+0x18) … `score_combat` (+0x44).
- Inputs: `score-reading.md` (first), `score-second.md` (blind second), README table "The families".
- Adjudicator: Opus 5.5 (claude-opus-5-5), 2026-11-11.
- Sources: the listing (`/Users/rf-studio/.claude/jobs/c88e6820/tmp/r4152/aa/exe.s`, `llvm-objdump` of
  `riseofnations.exe`), the decompile export (`~/ghidra-projects/decomp`), `crates/sim/src`.
- Method: every part's integer formula checked instruction by instruction in the listing (operand
  order of each multiply, each divide's idiom and sign fix, each clamp's compare and branch sense,
  which counts sum into `n`). Settled per leaf below, appended as each closed.

Listing citations are instruction addresses in `riseofnations.exe`; export citations are
`funcs/<Class>/<name>@<addr>.c:<line>` under `~/ghidra-projects/decomp`, cited here by function
name and line.

## 0. Shared machinery (every leaf cites this)

| # | claim | first | second | status | verdict | deciding line |
|---|---|---|---|---|---|---|
| S1 | `compute_score` gate: frame 0, forced arg, `semaphore` byte `Game+0x820 & 0x40`, or `(who + frame) % 10 == 0` (signed `idiv`, remainder) | yes | yes | agree | **confirmed** | `6ec56c`–`6ec594` |
| S2 | body order: explored=0; units if bits 23\|24, clear 23; build; economy; pop=0; upgrades+research if bit 24, clear 24; territory; Armageddon test; sum | yes | yes | agree | **confirmed** | `6ec59a`–`6ec68a` |
| S3 | `strategy_all` calls `check_explore`, `plan_strategy`, `compute_score(0)`, `diplomacy` for each of 8 slots with `leader_flags & 3 == 3`; `do_frame` skips it when `Game+0x821 & 8` | yes | yes | agree | **confirmed** | `6ed440`–`6ed462`; `59243f`–`592448` |
| S4 | `get_armageddon = ARMAGEDDON (C+0xd14) + ARMAGEDDON_PER_TEAM (C+0xd1c) × num_sides (G+0x6a8) + ARMAGEDDON_PER_NATION (C+0xd18) × num_nations (G+0x6a0)`; ×3 if `starting_resources` (G+0x2d) is 7; ×2 if 5, 6, 11, 12; floor 100 if 8 | yes | yes | agree | **confirmed** | `594020`–`59407e`; names from `types.txt` Constants/Game |
| S5 | Armageddon test is `game->armageddon (G+0x6e0) >= get_armageddon()` → zero and return | yes | yes | agree | **confirmed** (`cmpl %eax,0x6e0(%ecx); jl` = sum branch) | `6ec612`–`6ec618` |
| S6 | `defeat_all`'s forced `compute_score(1)` only runs after Armageddon, so it only ever zeroes | yes | **no**: "end-of-game sweep", plain game | disagree | **corrected (first is right)**: `process_victory` calls `defeat_all` only on `get_armageddon() <= game->armageddon` | `730f0e`–`730f22` (`jl 0x730f2d` skips the call); export `process_victory:73`–`76` |
| S7 | dirty bit 23 (`0x800000`) set by `Unit::init (00612100)` (owner `< 8`, signed) and `Unit::close (0060ee50)` (owner byte `< 8`); bit 24 by `Leader::gain_tech (006dcb60)` at entry; cleared only by the two `andl` in `compute_score` | yes | yes | agree | **confirmed**. My net: every `orl $0x800000/$0x1000000/$0x1800000`, byte forms `orb $0x80,2(..)`/`orb $1,3(..)`, every `andl $0xff7fffff/$0xfeffffff`, export `leader_flags` lvalues, and every absolute `0xe3a390(%reg)` store. The other hits are `UnitData::unit_masks` (`0x68(%reg)`), a `+0x4` field of another record, the scenario editor's unit masks (`9a3eec`), and library code. The whole-word stores in `Game::init_rules_and_teams (00589bb0)` (`58a09f`, `58a0c1`, `58a0d8`) and `Game::init_tribes (0058a600)` (`58a68a`) read, modify and write the word back, so they keep bits 23 and 24 | `612d83`, `60eedb`, `6dcb99`; `6ec5b0`, `6ec5e1` |
| S8 | `get_score_value(t, p)` (vslot `+0x7c`, one body `006673b0` in all four `Type` vtables) = factor × Σ`costs[0..6]`; militia family priced from type 0x32 when `p == 0`; unit premium arm `trunc256(trunc256(RESEARCH_PREMIUM × UCF × S) × research_premium_cost(t))` | yes | yes | agree | **confirmed**. Details: the range tests read the type's **own** index (`TypeData+0x4`, `ecx` at `6673bc`), not the substituted one. Militia is still priced with UCF. The premium arm indexes `research_premium_cost` by the possibly substituted index, but substitution happens only when `p == 0`, so it never applies on that arm. The order is `RESEARCH_PREMIUM × (UCF × S)`, a signed `>>8` with a +255 bias for negatives, then `× rpc`, then the same divide | `6673b0`–`6674e9` |
| S9 | which `Type` vtable carries which `get_kill_value` | **open** (no `Type` vtable in `vtables.txt`) | PE bytes: UnitType `0xb41fd4+0x80 → 0061d3d0`, BuildType `0xb42b94+0x80 → 00639790`, ObjectType/TypeData → `00663a00` | one-sided | **confirmed (second)**. The PE bytes were re-read. `0xb41fd4` is the UnitType one: `UnitType::vector_deleting_destructor (00477620)` installs it (`47768c`). `0xb42b94` is the BuildType one: `BuildData::is_wonder (00472320)` compares against it (`472328`) | PE `.rdata` words at `0xb41fd4+0x80`, `0xb42b94+0x80`, `0xb43aac+0x80`, `0xb43e48+0x80` |
| S10 | `kill(t, o)`: unit `value(t,0) + (sc0+sc1) × (num_units_o[t−0x32] + num_queued_o[t])`; build the same × `BUILD_SUPPORT_FACTOR` (constants `+0x37c`); `o < 0` → `value(t,0)` | yes | yes | agree | **confirmed**. `n` is the owner's count and **includes the queued** ones. It is not `n − 1`: the kill is priced as the owner's next unit | `61d3d0`–`61d41e`; `639790`–`6397ef`; `663a00` |
| S11 | `reset_score (006e37f0)` zeroes all twelve; callers `Leader::init (006e3930)` and `Setup::build_game (005ac190)` only | yes | yes | agree | **confirmed** | calls `6e3b1a`, `5ad1e7` (the only two `calll 0x6e37f0`) |
| S12 | `semaphore` bit 6 setters: `check_victory` (game-over latch; it returns at its head once set), `RecordGame::read_package`, `TurnControl::toggle_pause`, `MPDropWin::on_button_clicked` | listed | "not read" | one-sided | **confirmed**: the four sites are the only `orb $0x40,0x820(..)`; `check_victory:30` returns while it is set, `:114`–`115` sets it | `59286e`, `952f75`, `957b9a`, `95acd6` |
| S13 | `LeaderData::walk_data (006d6750)` restores the leaves on a load | "presumably; not read" | copies `[who, +0x692a)` | one-sided | **not adjudicated**: it is not on a plain game's frame path, and the checksum slots are in `checksum/leaders.rs:41`–`52` either way | — |
| S14 | crate step site: `Sim::strategy_all` (`crates/sim/src/ai/ai_drive.rs:73`), comment `:87`; the human arm `continue`s at `:82`–`:85` before it, but the original computes the score for humans too | yes | yes | agree | **confirmed** | `ai_drive.rs:73`–`89`; `6ed440`–`6ed45b` (no human test) |
| S15 | crate home of `get_kill_value` | not named | "no crate function (grep finds none)" | one-sided | **corrected**: `economy::cost::kill_value_by` (`crates/sim/src/economy/cost.rs:1258`) already ports `BuildTypeData::get_kill_value`. It is swept against the original in `crates/sim/tests/sweep/build_type_data_get_kill_value.rs` and nothing calls it yet. Passing `factor = 1` with the unit arrays gives the unit form | `cost.rs:1252`–`1273` |
| S16 | **new**: `BitMask<44>::count (0047cd40)`, which `compute_economy_score` calls on `rare`, **writes** `rare.flags` (`LeaderData+0x6d98+0x8`): 1 when the count is 0, else 0. It returns 0 at once when that flag is already 1 | — | — | neither | **finding** (side effect, not a score leaf; whether the leaders checksum walks `BitMask::flags` was not checked) | `47cd78`, `47cd84` |

## 1. `score` (+0x18)

| claim | status | verdict | deciding line |
|---|---|---|---|
| writers: `compute_score` (sum `6ec68a`, zero `6ec660`), `reset_score` | agree | **confirmed**. Every absolute `0xe3a3a8` hit is a load or a `movl $0xe3a3a8,%reg` pointer walk whose loops only read (`6bc9c9`, `6c0bd9`, `731ea1`). The export's `field_0x18` lvalues in `Leader*` methods are only `compute_score` and `reset_score`. A register-relative net (stores `0x18`–`0x44(%reg)` within 25 lines of a `0xe3a390`/`0xc061e0` base) found only reused registers | `6ec660`, `6ec68a` |
| step: `score = combat + wonders + research + upgrades + economy + buildings + units + territory + pop + explored`, i32 wrapping; `score_units_2` is not a term | agree | **confirmed**. The territory term is the value just computed, held in `edi`, not re-read. The sum's operand order is exactly the one listed | `6ec66d`–`6ec68a` |
| predicate: recompute frame (S1), else unchanged; Armageddon → 0 | agree | **confirmed** | S1, S5 |
| reader `Army::find_target (006f69b0):372` | **disagree**. First: fires when the target's score > 4·mine/3. Second: when 4·target/3 < mine | **corrected (second is right)**. `local_c` loops over candidate target leaders (`:157`–`159`, `local_28 = c·0x6eec`); `Army+0x94` is `who`, the army's owner. The arm fires when `trunc((score[c] << 2) / 3) < score[army owner]`, i.e. **my score exceeds 4/3 of the candidate's**. The `<<2` wraps at 32 bits and `/3` truncates (`0x55555556`) | `6f74e1`–`6f750e` (`cmpl %ecx,0xe3a3a8(%eax)`, `eax` = army `who`; `jg` taken) |
| reader `process_victory`, `victory == 5`: the live leader with the strictly lowest `score` is defeated; a tie at the minimum defeats nobody | first: exact; second: "lowest/tied" | **confirmed (first)**. In the walk, equal to the running minimum from another slot sets the tie flag, and strictly lower clears it; `jne` on the flag skips the defeat | `731eb0`–`731eed` |
| reader `Leader::diplomacy`: count of live leaders scoring above me, top scorer | agree | **confirmed** (`cmpl 0xe3a3a8(%edx),%ecx` and `cmovle`) | `6bc9d0`–`6bca1c` |
| crate: no computation; readers seamed at `leaders/diplomacy.rs:155`, `:385`, `ai/army/target_score.rs:457` | agree (each named some) | **confirmed** (all three exist) | — |
| plain game: yes via `strategy_all`; `defeat_all` only after Armageddon | see S6 | **corrected** (second's "plain-game end sweep") | `730f1f` |

**Rule to port.** In `Sim::strategy_all`, for each slot with `leader_flags & 3 == 3`, humans
included, between `plan_strategy` and `diplomacy`, run `compute_score(w, forced=false)`. It
recomputes when `frame == 0 || forced || sem_bit6 || (w + frame) % 10 == 0`, where `frame` is
the value before `do_frame` increments it. On a recompute it runs the parts as in S2 and sets
`score = combat + wonders + research + upgrades + economy + buildings + units + territory`
(`pop` and `explored` are 0 by construction), as i32 wrapping adds. If
`armageddon >= get_armageddon()`, it zeroes the eleven leaves listed in §2 (not
`score_units_2`) instead. Reset: `reset_score` at setup, which is the crate's default 0.
Confidence high, listing read whole. Open: `get_team_score` past its head, and the three
non-victory bit-6 setters' contexts.

## 2. `score_explored` (+0x1c)

| claim | status | verdict | deciding line |
|---|---|---|---|
| writers: `compute_score` 0 at `6ec5a0` (on every recompute) and `6ec636` (Armageddon); `compute_explore_score (006bc5e0)` stores 0 and is dead | agree | **confirmed**: no `calll 0x6bc5e0`, and the 4-byte value `0x006bc5e0` occurs **0 times** in the PE file, so no pointer to it exists either | `6ec5a0`; PE byte search |
| step: always 0; does not read `explored` (`+0x9d4`) | agree | **confirmed** | `6ec5a0` |
| crate: default 0 matches; checksum slot `checksum/leaders.rs:42` | agree | **confirmed** | — |

**Rule.** It is 0 always: write 0 on each recompute (a no-op from the default). Confidence high.
Nothing open.

## 3. `score_territory` (+0x20)

| claim | status | verdict | deciding line |
|---|---|---|---|
| step: `territory (LeaderData+0x9d8 of leaders[who]) × 1000 / land_size`, `imul` then `cltd; idivl`, signed and truncating; on every recompute, no dirty bit | agree | **confirmed** | `6ec5e7`–`6ec604` |
| `land_size` is `WorldData+0x78` with no prefix ahead of it in `World` | agree | **confirmed**: `World` (0x174) has no vtable in `vtables.txt` and holds `WorldData` (0x16c) at offset 0. `check_explore:25`–`37` walks `+0x28`/`+0x24` as `reg_ys`/`reg_xs`, which matches `WorldData` | `types.txt` WorldData `+0x78 land_size`; `6ec5ff` |
| no Armageddon test of its own; zeroed in the arm (`6ec628`) | second explicit, first implicit | **confirmed** | `6ec628` |
| crate inputs `Holdings::territory`/`land_size` (`economy.rs:554`–`555`) | agree | **confirmed** to exist. Whether they refresh with the same timing as `LeaderData::territory` is open for both readers | — |

**Rule.** On each recompute, `score_territory = territory * 1000 / land_size` (i32, truncating
toward zero; the original faults on `land_size == 0`). Confidence high. Open: whether the crate's
`territory` is current at `strategy_all` time.

## 4. `score_units` (+0x24) and 5. `score_units_2` (+0x28)

One function writes both: `Leader::compute_unit_score (006bc500)`.

| claim | status | verdict | deciding line |
|---|---|---|---|
| writers: `compute_unit_score` zero `6bc50c`/`6bc513`, add `6bc5b4`/`6bc5c6`; `compute_score` zeroes `+0x24` in the Armageddon arm (`6ec62f`) but **not** `+0x28`; `reset_score` | agree | **confirmed** | `6ec61a`–`6ec660` (no `0x28`) |
| gate: recompute frame **and** `leader_flags & 0x1800000` | agree | **confirmed** | `6ec59a`, `6ec5a7` |
| loop `t ∈ [0x32, 0x192)`, exactly the 352 entries of `num_units` | agree | **confirmed** (`num_units` is `ushort[352]`) | `6bc533`, `6bc5ca` |
| `n = num_queued[t] + num_units[t − 0x32]` (`+0x5a22 + 2t`, `+0x56fe + 2t`), both u16, skip when 0 | agree | **confirmed**: `n` includes the queued ones | `6bc549`–`6bc55e` |
| `u = trunc( ( trunc( ((n−1)·sup)·n / 2 ) + sv·n ) / 5 )`, `sup = unittypes[t].support_cost[0]+[1]`, `sv = value(t,0)`; no BSF | agree | **confirmed** term by term. `sv·n` is `6bc571`. `(n−1)·sup` is `6bc592`, then `·n` at `6bc595`. `/2` is `cltd; sub; sar` (`6bc598`–`6bc5a2`). Then `+ sv·n` (`6bc5a4`). `/5` is `0x66666667`, `sar 1`, plus the sign bit (`6bc5a9`–`6bc5b2`). Note: `(n−1)·n` is always even, so the `/2` is exact unless the product wraps; only the `/5` truncates | `6bc56f`–`6bc5b4` |
| `score_units_2 += u` only when `unittypes[t].attack (+0x1e8) != 0` (32-bit compare) | agree | **confirmed** | `6bc5bd`–`6bc5c6` |
| Armageddon inside: both zeroed at the head, then return before the loop | agree | **confirmed** | `6bc50c`–`6bc52b` |
| `score_units_2` has no decision reader and is not summed | agree | **confirmed**: no absolute `0xe3a3b8` reference in `.text`, and no `->score_units_2` in the export | net above |
| crate inputs: `Muster::by_type`/`queued_by_type` (`units/unit.rs:838`, `:840`); dirty-bit hooks at unit creation (`units/store.rs:48` `add_unit`), close (`combat/fight/delivery.rs:1245` `close_unit`), tech gain (`tech/gain.rs:189`) | agree (second names the lines) | **confirmed**: all exist | — |

**Rule.** When `recompute && (dirty23 || dirty24)`: set both to 0. If Armageddon has not come,
then for each unit type `t` in `0x32..0x192` with `n = units[t] + queued[t] > 0`: compute
`u = (value(t,0)*n + ((n-1)*(sc0+sc1)*n)/2) / 5` (i32 wrapping multiplies, truncating divides),
add `u` to `score_units`, and add it to `score_units_2` as well when `attack != 0`. Afterwards
clear `dirty23` (not `dirty24`). `score_units_2` survives the Armageddon arm. Plain game: yes.
Confidence high. Open (both readers): whether every unit removal goes through `Unit::close`;
the crate's meaning of `queued_by_type` for units.

## 6. `score_buildings` (+0x2c) and 11. `score_wonders` (+0x40)

One function writes both: `Leader::compute_build_score (006bc3f0)`.

| claim | status | verdict | deciding line |
|---|---|---|---|
| writers: zero `6bc3f7`/`6bc3fe`, add `6bc4c7`/`6bc4ec`; Armageddon zero `6ec621`/`6ec644`; `reset_score` | agree | **confirmed** | as cited |
| on every recompute, no dirty bit | agree | **confirmed** (`6ec5b6` is unconditional) | `6ec5b8` |
| loop `t ∈ [0x19e, 0x21f)` = the 129 entries of `num_buildings` | agree | **confirmed** | `6bc41e`, `6bc4cb` |
| `n = num_queued[t] + num_buildings[t − 0x19e]` (`+0x5a22`, `+0x5222 + 2t`) | agree | **confirmed**: `n` includes the queued ones, which for a building are inactive sites (`Wall::decrement_stats (00642da0)` takes `num_queued` down when the building is not active) | `6bc42c`–`6bc441` |
| `b = sv·n + trunc( ((n−1)·((sc0+sc1)·BSF))·n / 2 )` | agree | **confirmed**: `sup·BSF` first (`6bc47d`), then `·(n−1)` (`6bc484`), then `·n` (`6bc487`), then `/2` (`6bc48b`–`6bc48e`), then `+ sv·n` (`6bc490`). BSF is constants `+0x37c` | `6bc456`–`6bc490` |
| wonder iff the type's own index (`TypeData+0x4`) is in `[0x20e, 0x21f)` (the inlined `is_wonder_type`, vslot `+0x1c` compared to `0x470780`): `score_wonders += trunc(b/3)`; else `score_buildings += trunc(b/5)` | agree | **confirmed**. `/3` is `0x55555556` plus the sign bit; `/5` is `0x66666667`, `sar 1`, plus the sign bit. All four `Type` vtables hold `0x470780` at `+0x1c`, so the inline arm always runs | `6bc49c`–`6bc4ec`; PE vtable words |

**Rule.** On each recompute, set both to 0. If Armageddon has not come, then for each building
type `t` in `0x19e..0x21f` with `n = buildings[t] + queued[t] > 0`: compute
`b = value(t,0)*n + ((n-1)*((sc0+sc1)*BSF)*n)/2`. Add `b/3` to `score_wonders` when `t` is in
`0x20e..0x21f`, else add `b/5` to `score_buildings`. Plain game: yes. Confidence high. Open:
nothing in the formula; the crate's `num_sites_of` as `num_queued` is the crate's own claim.

## 7. `score_economy` (+0x30)

| claim | status | verdict | deciding line |
|---|---|---|---|
| writers: zero `6bc277`, adds `6bc2f0`/`6bc324`/`6bc34d`; Armageddon zero `6ec61a`; `reset_score` | agree | **confirmed** | as cited |
| goods `g = 0..5` (six) gated by `type_avail(g, 1) != 0` (argument order: `g` first, then 1) | agree | **confirmed** (`edi` runs `0x600..0x618` in steps of 4; push 1, then push `g`) | `6bc2a1`–`6bc2ad`, `6bc331` |
| `a = trunc(((bucket[g] ^ 0x8221) − game->starting[g]) / 20)`, clamped to `[0, 100]` | agree | **confirmed**. The subtraction comes before the divide. `/20` is `0x66666667`, `sar 3`, plus the sign bit. Then `js` → 0 and `> 100` → 100, so the clamp follows the truncation. `starting` is `Game+0x600` (`types.txt` `int[6] starting`); the first reader's note that the export's `info.player+0x5bc` print is the same word holds | `6bc2af`–`6bc2f0` |
| `b = min(trunc((income[g] ^ 0x90236) / 80), 200)`, **no floor** | agree | **confirmed**: `income` is at `data_encrypted+0x94`; `/80` is `sar 5` on the high word; `cmovg` caps at 200; there is no lower bound | `6bc2f3`–`6bc324` |
| `+= 25 × BitMask<44>::count(rare)` (`+0x6d98`) | agree | **confirmed**: `count` is a popcount of the `size` bytes at `+0xc` (`bit_count` per byte). See S16 for its write to `rare.flags` | `6bc33d`–`6bc34d`; `47cd40`–`47cd8d` |
| crate inputs `Ledger::bucket`, `Ledger::income`, `Holdings::starting`, `Holdings::rare` | agree | **confirmed** to exist. Whether the crate's `income` is the same word (units, and the moment it is taken) is **open**, as both readers said: the crate documents `income` as "the net rate the interface shows" and `rate` as sixteenths | `economy.rs:679`–`699` |

**Rule.** On each recompute, set to 0. If Armageddon has not come, then for each good
`g in 0..6` with `type_avail(g, strict=1) != 0`, add
`clamp((bucket[g] - starting[g]) / 20, 0, 100) + min(income[g] / 80, 200)` (truncating
divides; a negative income subtracts). Then add `25 * popcount(rare)`. Plain game: yes.
Confidence high on the arithmetic. Open: `type_avail` for goods (`has_preq` and `type_eligible`
were not read), and the crate's `income` units.

## 8. `score_pop` (+0x34)

| claim | status | verdict | deciding line |
|---|---|---|---|
| writers: `compute_score` 0 at `6ec5ca` (each recompute) and `6ec659`; `compute_pop_score (006bc190)` is dead | agree | **confirmed**: no call, 0 PE pointer occurrences | `6ec5ca` |
| always 0 | agree | **confirmed** | — |

**Rule.** 0 always. Confidence high.

## 9. `score_unit_upgrades` (+0x38)

| claim | status | verdict | deciding line |
|---|---|---|---|
| writers: zero `6bc1a3`, add `6bc230`; Armageddon zero `6ec652`; `reset_score` | agree | **confirmed** | as cited |
| gate: recompute frame **and** bit 24 | agree | **confirmed** | `6ec5c4`, `6ec5d1` |
| loop `t ∈ [0x32, 0x192)`; counted iff (`has_tech(t)` or `researching(t, −1, 0, 1)`) and `tech_at_start` bit `t` clear (bytes at `+0x6c8c` = BitMask `+0xc`, bit `t & 7` from `rol`) and `unit_flags (+0x2b4) & 0x80` clear | agree | **confirmed**. Push order at `6bc1dc`–`6bc1e1` is 1, 0, −1, `t`, so `researching(t, −1, 0, 1)`. `rolb $0x32` starts the bit at `0x32 & 7` | `6bc1c2`–`6bc20d` |
| `researching`'s 4th argument 1 means an exact type match only | agree | **confirmed**: with `param_3 == 0` (a unit is not a build type), every active building from object 2000 up is scanned. A queue slot counts if it equals `t`, or (when `param_4 == 0` only) if both are unit types, neither has an upgrade, and `t`'s vslot `+0x60` accepts the queued type. `param_4 = 1` removes that second arm | export `researching:22`–`45` |
| adds `trunc(value(t,1)/5)` (the premium arm; no militia substitution) | agree | **confirmed** | `6bc214`–`6bc230`; S8 |

**Rule.** When `recompute && dirty24`: set to 0. If Armageddon has not come, then for each unit
type `t` in `0x32..0x192` that is (held per `has_tech`, or queued as an exact match in an active
building of mine) and not `tech_at_start[t]` and not `unit_flags & 0x80`, add
`value(t,1)/5`, where `value(t,1) = ((RP*(UCF*Σcosts))/256 * rpc(t))/256` with every divide
truncating toward zero. Plain game: yes. Confidence high. Open (both): `has_tech` for a unit
type, and the meaning of `researching`'s vslot `+0x4c` "active" test.

## 10. `score_research` (+0x3c)

| claim | status | verdict | deciding line |
|---|---|---|---|
| writers: zero `6bc363`, add `6bc3db`; Armageddon zero `6ec64b`; `reset_score` | agree | **confirmed** | as cited |
| gate: recompute and bit 24; loop `t ∈ [0x220, 0x275)`; counted iff (`has_tech` or `researching(t, −1, 0, 0)`) and not `tech_at_start`; no `unit_flags` test; adds `trunc(value(t,0)/5)` = `trunc(TCF·Σcosts/5)` | agree | **confirmed**. Push order 0, 0, −1, `t` (`6bc39c`–`6bc3a0`). The `param_4 == 0` arm needs both types to be unit types, so it is inert for techs | `6bc381`–`6bc3e7` |

**Rule.** When `recompute && dirty24` (after §9, then clear `dirty24`): set to 0. If Armageddon
has not come, then for each `t` in `0x220..0x275` (ages included) that is held or queued exactly,
and not `tech_at_start[t]`, add `(TECH_COST_FACTOR * Σcosts[t]) / 5`. Plain game: yes.
Confidence high. Open: `has_tech` past its head.

## 12. `score_combat` (+0x44)

| claim | status | verdict | deciding line |
|---|---|---|---|
| writers: `Object::do_damage (0064a480)` `64bb02`/`64bb1b`/`64bb50`; `Object::take_damage (00652020)` `65313e`/`653146`/`65316d`; `SpellType::cast_sabotage (006720e0)` `672252`, `6727a0`; `SpellType::cast_counterintel (00673740)` `67388e`; `compute_score` Armageddon zero `6ec63d`; `reset_score` | agree | **confirmed**: these ten are every absolute `0xe3a3d4` reference in `.text`. The register-relative net finds no other store off a leader base. `compute_combat_score (006bc250)` is an uncalled `ret` with 0 PE pointers | grep of `0xe3a3d4` |
| amount `trunc(kill(t, o)/5)` (`0x66666667`, `sar 1`, plus the sign bit) | agree | **confirmed** at all six arithmetic sites | e.g. `65312b`–`65313c` |
| **unit kill** (`take_damage`): the victim is not a building (vslot `+0x1c`; a building takes `+0x81c += 1` and leaves); captain (vslot `+0xe8`, inline `+0x8e >> 15`); `curr_uber_size(0) == 1`; `unit_masks & 1` clear → victim owner `+0x810 += 1`; **then only if `param_7 >= 0`**: victim `+0xa58`, killer `param_8` `+0xa5a`, `+0x80c`; redirect when `unit_flags & 0x10` to `(+0x3e, +0x28)`; id `< 0` or vslot `+0x18` false → no score; `is_ally(K, o)` → subtract with a floor at 0, else add; no killer-kind test and no splash test | agree | **confirmed** line by line. The floor is `jns` after the `subl`, else store 0, so a result that goes negative becomes 0. `is_ally` is called on `leaders[K]` with `o` the (possibly redirected) owner. The kill type is the redirected object's `type->type` | `652f46`–`653173` |
| order: the unit credit precedes the victim's `die`, so `num_units_o[t]` still counts the victim | both "likely" | **confirmed**: the arm ends at `653173`; vslot `+0x158` (`die`) is called at `6533cd` | `6533cd` |
| **building raze** (`do_damage`): the target's type is captured before its `take_damage` (vslot `+0x16c`); `take_damage != 0` and the target is a building (vslot `+0x1c`) → `+0x818 += 1`; `is_ally(A, param_2)` → subtract with floor (`cmpl $0; jge`, else 0), else add; `kill(t, param_2)` evaluated after `take_damage` | agree | **confirmed** | `64b9f1`–`64b9f7` (type captured), `64ba18`, `64ba34`, `64babb`–`64bb50` |
| whether `num_buildings_o[t]` still counts the razed building when `kill` runs | **open** in both (second: "likely no") | **settled: it does not**. `take_damage` calls vslot `+0x158` (`die`) before it returns (`6533cd`). For a Build that slot is `Wall::die` (by COMDAT folding), which thunks to `Object::die (00647080)`. `Object::die` first calls vslot `+0x150`, which is `Build::close` (`vtables.txt` Build `+0x150`; export `Object::die:13`). `Build::close (00628980)` calls `Wall::decrement_stats (00642da0)` (`628a80`; export `Build::close:97`–`99`, when `build_flags & 0x10` or in a city). That takes `num_buildings` down for an active building, or `num_queued` for an inactive one. So the raze is priced at the owner's count **after** the building is removed. The same holds for sabotage's building arm | `6533cd`; `628a80`; export `Object::die:13`, `Build::close:99` |
| **sabotage**, unit arm: not a building, a unit (vslot `+0x18`), captain, last figure, not a decoy → victim `+0xa58`, caster `+0xa5a`, `+0x80c`, `+= kill(t, param_4)/5`; no ally test, no floor | agree | **confirmed** | `672182`–`672252` |
| **sabotage**, building arm: `take_damage(dmg, 0, 3, −1, 1, 0x55555555, −1, −1, 0)` returned non-zero → caster `+0x818`, `+= kill(t, param_4)/5`; `param_7 = −1`, so no double count | agree | **confirmed** from the push order | `672745`–`6727a0` |
| **counterintel**: target climbed to its captain; target owner ≠ caster and not allied both ways (`diplos == 2` each side, inlined) → unit, captain, last figure, not decoy → `+0x80c`, `+= kill(t, o)/5` | agree | **confirmed** | `6737c5`–`67388e` |
| crate: unit arm in `take_damage_typed` (`combat/fight/delivery.rs:537`–`543`, beside `unit_plunder`), building arm in `do_damage` (`:212`–`:222`, beside `plunder_kill`), sabotage SEAMs `:804`, `:882` | agree (the first named `damage_building :957`, the second `do_damage`) | **confirmed**, with a note: the building credit's home is `do_damage`'s `killed` arm at `delivery.rs:215`, which has the attacker. `damage_building` (`:957`) has only an optional `by`. The score predicate is wider than plunder's: any attacker, splash included, and the allied case subtracts | `delivery.rs:212`–`222` |
| crate: counterintelligence | **disagree**. First: "no crate cast found". Second: `spells/cast.rs:1022`–`1028` SEAM | **corrected (second is right)**: `Sim::cast_counterintel` exists (`crates/sim/src/spells/cast.rs:1031`) with the SEAM for `units_killed` and `score_combat` at `:1028` | `cast.rs:1028`–`1040` |
| **new: the ally test for an owner past the table.** `LeaderData::is_ally (006edb50)` reads `this->diplos[param_1]` from `int[8]`, while `Leaders::list` has 10 slots and gaia's units are owner 8 (and 9; `crates/sim/src/gaia.rs:93`). When a gaia unit is the victim, `take_damage` calls `is_ally(K, 8)`. That reads `diplos[8]`, which is `treaties[0]` (`LeaderData+0x94`), and `leaders[8].diplos[K]`. The crate's `Sim::is_ally` (`leaders/relations.rs:99`) says no caller asks for a leader outside the table. **This caller does**, whenever a hunted animal is killed by an object (`param_7 >= 0`) | neither | **open**: whether `treaties[0]` and `leaders[8].diplos[K]` are ever both 2. If they are not, every gaia kill adds `kill(t, 8)/5`, priced on leader slot 8's own counts. A `LEADERDATA` dump across a hunt, or a frame-snapshot read of the two words, settles it. A builder should not answer "not allied" from the crate's guard without that check | export `is_ally:7`–`9`; `types.txt` `+0x74 diplos int[8]`, `+0x94 treaties`; `Leaders` `Leader[10]` |

**Rule.** It is an accumulator, never recounted. At each event, `v = kill(t, o) / 5`
(truncating), with `kill` as in S10 and `o` the victim's owner:
- unit killed in `take_damage`: the victim is a non-building captain on its last figure and not
  a decoy, the killer object id is `>= 0`, and the (redirected) victim passes vslot `+0x18`. Then
  `sc[K] = is_ally(K, o) ? max(sc[K] - v, 0) : sc[K] + v`. This comes before the victim dies
  (the count includes it).
- building razed in `do_damage`: the target died and is a building. Same ally rule, credited to
  the attacker's owner, priced after the building's close (the count excludes it).
- sabotage, unit arm or building arm: `sc[caster] += v`.
- counterintelligence on a non-allied single-figure captain: `sc[caster] += v`.
Reset: `reset_score` and the Armageddon arm. Plain game: yes for the combat arms. The spells
need a spy or commando cast. Confidence high on the stores, amounts, floors and predicates
(listing). Open: the gaia ally read above; the meaning of the `unit_flags & 0x10` redirect;
whether the AI casts sabotage or counterintelligence in a skirmish.

## Verdict table

| leaf | confirmed / corrected / open | the rule in one line |
|---|---|---|
| shared (S1–S16) | 12 / 2 / 1 (+1 new finding, S16) | `compute_score` runs for every live slot (humans included) on `frame==0 \|\| forced \|\| sem6 \|\| (who+frame)%10==0`; `defeat_all`'s forced call is after Armageddon only (S6, corrected); dirty bits 23 (`Unit::init`/`Unit::close`) and 24 (`gain_tech`), cleared only by `compute_score`; `kill_value_by` (`economy/cost.rs:1258`) already ports the BuildType kill value (S15, corrected) |
| `score` | 6 / 2 / 0 | on a recompute frame `combat+wonders+research+upgrades+economy+buildings+units+territory` (i32), else unchanged; 0 after Armageddon; `find_target` fires when `4·score[c]/3 < score[me]` (corrected) |
| `score_explored` | 3 / 0 / 0 | always 0 |
| `score_territory` | 4 / 0 / 1 | `territory*1000/land_size`, truncating, every recompute |
| `score_units` | 9 / 0 / 1 (shared with `score_units_2`) | when recompute and dirty 23 or 24: `Σ_{t∈[0x32,0x192)} (value(t,0)·n + (n−1)·sup·n/2)/5`, `n = units + queued` |
| `score_units_2` | (as above) | the same terms over types with `attack != 0`; not summed into `score`, not zeroed by the Armageddon arm |
| `score_buildings` | 6 / 0 / 0 (shared with `score_wonders`) | every recompute: `Σ_{non-wonder t∈[0x19e,0x21f)} (value(t,0)·n + (n−1)·(sup·BSF)·n/2)/5`, `n = buildings + queued` |
| `score_economy` | 6 / 0 / 2 | every recompute: `Σ_{avail g<6} clamp((bucket−starting)/20,0,100) + min(income/80,200)`, `+ 25·popcount(rare)` |
| `score_pop` | 2 / 0 / 0 | always 0 |
| `score_unit_upgrades` | 4 / 0 / 1 | when recompute and dirty 24: `Σ` over unit types held or exactly queued, not at start, not `unit_flags & 0x80`, of `value(t,1)/5` (8.8 premium, truncating twice) |
| `score_research` | 2 / 0 / 1 | when recompute and dirty 24: `Σ_{t∈[0x220,0x275)}` held or exactly queued, not at start, of `TCF·Σcosts/5` |
| `score_wonders` | (as `score_buildings`) | the building term over `t∈[0x20e,0x21f)`, `/3` instead of `/5` |
| `score_combat` | 10 / 1 / 1 | accumulates `±kill(t,o)/5` per captain-last-figure kill (with a killer object) or building raze; subtract with a floor at 0 for an ally; spells always add; the unit count includes the victim, the building count excludes the razed one (settled); a gaia victim's ally read is out of bounds (open, new) |

**What changed from the readings.** Four corrections:
- `defeat_all` only follows Armageddon (S6; the second reader was wrong).
- `Army::find_target`'s score clause runs the other way round (§1; the first reader was wrong).
- The crate has `cast_counterintel` (§12; the first reader was wrong).
- The crate has `kill_value_by` (S15; both readers missed it).

One open question became settled: the razed building is out of `num_buildings` before its kill
value is read. Two new findings: the gaia owner's out-of-table `is_ally` read on the kill arm
(§12, open), and `BitMask<44>::count` writing `rare.flags` (S16). Every integer formula in the
five parts, `get_score_value` and both `get_kill_value`s was checked against the listing
instruction by instruction, and none was found wrong. That covers every multiply's order, every
divide's idiom, every clamp's direction, and which counts make up `n`. No diff backs any of
this. `LeaderData::log_data (006e5110)` prints `score`, so a value diff becomes possible once a
port exists.
