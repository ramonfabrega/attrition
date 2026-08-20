# Adjudication: PRODUCTION — reading A (docs/PRODUCTION.md + production.rs/lib.rs) vs reading B (rederive-production.md)

Method: every disagreement below was re-read in `~/ghidra-projects/decomp/funcs/<Class>/<method>@<addr>.c`, with `types.txt`/`vtables.txt`, and — where Ghidra dropped the argument of a devirtualised `ObjectData::is` call (vtable +0xb8) — settled from the raw bytes of `riseofnations.exe` with `llvm-objdump` (image base 0x400000, section .text VA = file VA). Type indices were mapped by position in `game/data/unitrules.xml` (units start 0x32) and `buildingrules.xml` (buildings start 0x19e).

Files: A = `docs/PRODUCTION.md`, `crates/sim/src/production.rs`, `crates/sim/src/lib.rs` (queue_up/cancel/queue_target/process_queues/advance_slot). B = `/Users/rf-studio/.claude/jobs/3c382923/tmp/rederive-production.md`.

---

## 1. Disagreements (behaviour-changing)

### D1. Parallel-slot recursion: every building (A's implementation) vs library only (B)

- **A (doc):** "It recurses into the next slot when `i + 1` is below both `queued` and `get_building_cities()`" — stated unconditionally in "The clock". **A (impl):** `lib.rs:508-520` `process_queues` applies `parallel_slots(library_cities, queued).max(1)` to **every** building, so a barracks owned by a player with 4 library cities advances 4 unit entries at once.
- **B:** the recursion sits inside the library branch of `do_queue`; "Non-library builds advance exactly slot 0 per frame."
- **Evidence:** `Build/do_queue@0061e410.c` — after the counter arithmetic: `iVar14 = is(?)` then `if (iVar14 != 0) { iVar14 = LeaderData::get_building_cities(...); if ((param_1 + 1 < iVar14) && (param_1 + 1 < queued)) do_queue(param_1+1); if (done) {...} }` else non-library path with no recursion. The dropped argument: bytes at 0x61eb76 are `8b 03 6a 00 68 b3 01 00 00` = `mov eax,[ebx]; push 0; push 0x1b3` → `is(0x1b3 /*LIBRARY*/, 0)`. (0x1b3 = Library by position in buildingrules.xml.)
- **Verdict: B right; A's doc is under-qualified and A's implementation is wrong.** `process_queues` must advance only slot 0 for non-library buildings; the `get_building_cities` fan-out applies only to a library (in practice the first library, since non-first libraries return early — see D6).

### D2. What a blocked head lets through: "zero → nothing; negative → non-caravan/helicopter" (A) vs "r ≤ 0 → first research entry; then r < 0 → non-caravan/helicopter" (B)

- **A:** `finished` zero (pop cap) → "nothing happens ... a pop cap blocks the whole queue"; negative → building looks for another slot "through `get_next_non_caravan` or `get_next_helicopter`". `lib.rs:543-544` maps both `Population` and `Limit` to `None`.
- **B:** on `r <= 0` and `slot == 0`: `n = get_next_non_unit(queued, who)` (first slot ≥ 1 that is not a unit type, or is a unit whose tech bit is unset — i.e. a research job) → `do_queue(n)`; only if that finds nothing, and `queued >= 2` and `r < 0`: `is(AIRBASE) ? get_next_helicopter : get_next_non_caravan`.
- **Evidence:** `Build/do_queue@0061e410.c` tail: `local_30 = finished(); if (0 < local_30) {unqueue...; return} if (param_1 != 0) return; iVar14 = BuildQueueData::get_next_non_unit(&queue, queued, who); if (0 < iVar14) { do_queue(iVar14); return; } if (queued < 2) return; if (-1 < local_30) return; ... is(?) ? get_next_helicopter : get_next_non_caravan; if (iVar14 >= 1) do_queue(iVar14)`. `BuildQueueData/get_next_non_unit@00630b10.c`: loop from slot 1, `return i` if not `is_unit_type` in [0x32,0x19d] or `LeaderData::has_tech(type) == 0`. The `is` before the helicopter branch pushes `0x1bf` (Airbase) at 0x61ece4.
- **Verdict: B right.** A pop-blocked head (r = 0) does let the first *research* entry behind it advance; a limit-blocked head (r = −1) first tries the same, then the non-caravan / helicopter search. Both redirections happen only when the blocked entry is slot 0. A's "pop cap blocks the whole queue" is wrong for mixed queues. Impl: `advance_slot` should, on `Population`/`Limit` at slot 0, try the first research-job slot ≥ 1 (and on `Limit`, the caravan/aircraft alternatives once those exist).

### D3. `could_queue`'s special case: dock + fishing boat (A) vs University + Scholar (B)

- **A:** "a dock queueing a fishing boat refuses when the queued count plus the current gatherer count would exceed six"; `production.rs` `QueueFail::Full` doc says "or a dock's six-boat rule".
- **B:** `this->is(UNIVERSITY 0x1a4)` and type ∈ {0x34, 0x35} (Scholar): `count_queue(1, 0x34) + num_gatherers(0,0) > 6 → QUEUE_FULL`.
- **Evidence:** `BuildData/could_queue@0062da50.c`: `is(0x1a4,0)` ... `iVar1 == 0x34 || iVar1 == 0x35` ... `(vtable+0x190)(1,0x34)` + `num_gatherers(this,0,0)`; `6 < sum → QUEUE_FULL`. Position map: 0x1a4 = University (7th record of buildingrules.xml), 0x34/0x35 = Scholar/Scholar (3rd/4th of unitrules.xml).
- **Verdict: B right, A wrong** (the gate is the University's six-scholar rule, not a dock rule). Doc and the `QueueFail::Full` comment need correcting; no arithmetic change in the impl (the rule is not implemented either way).

### D4. Completion of a research job for a unit type (A's impl spawns a unit; the original does not)

- **A (doc):** silent — "Completion" only covers "if type is a unit type and the availability bit is set: ... train". **A (impl):** `lib.rs:545-566` on `Trained` always spawns a unit, increments `by_type`, and sets `muster.researched[ty] = true` — so the *first* (research) entry yields both the bit and a unit.
- **B:** "Unit with bit unset (research completed), techs, and anything else: `Leader::gain_tech(type, x, y, 1, 1)` → sets the tech bit ... Return 1." No unit.
- **Evidence:** `Build/finished@00628490.c`: the `train(this, param_1); return 1;` path is inside `if (tech bit set)`; a unit type with the bit clear falls through every `else` to `LAB_00628520: Leader::gain_tech(...)`, then `return 1`. `Leader/gain_tech@006dcb60.c:303` `BitMask<256>::set(&this->field_0x6c0c, param_1, 1)`.
- **Verdict: B right; A's implementation diverges.** A research entry completing should set `researched[ty]`, remove the entry (no refund), and spawn nothing; the player queues again for the first trained unit (which then gets the ramp and `ACCEL_TRAIN`). Note the pop-cap / caravan / aircraft checks also do not apply to the research entry (they are inside the bit-set branch).

### D5. `refund_cost` scope: "every queued item" (A) vs "tech entries in the first library's queue, on a science-line epoch" (B)

- **A:** "When your science level rises, every queued item is re-priced where it sits."
- **B:** called from `gain_tech` only when the gained type is an epoch with `techtype.cat == 3`, `frame != 0`, and only for entries of the *first library's* queue that are tech types and not the tech just gained.
- **Evidence:** `Leader/gain_tech@006dcb60.c:165-191`: `is_epoch_type(param_1) && techtypes[param_1]+0x14 == 3 && frame != 0 && get_first_library >= 0` → loop `i < first_library->queued`: `TVar16 = get_queue(i); if (TVar16 != param_1 && is_tech_type(TVar16)) Build::refund_cost(lib, i)`. `Build/refund_cost@00620490.c` reads `techtypes[entry.type]+0x1c8` (tech age), so it is only meaningful for tech entries anyway.
- **Verdict: B right.** Arithmetic in `production::reprice` matches the decompile exactly (magic-multiply `-0x51eb851f >> 5` with sign fix = `-(x/100)`); only the doc's scope sentence is wrong. Unit entries in a barracks are never re-priced.

### D6. Frames to complete: "one extra frame past reaching the target" (A) vs "ceil(T/a) calls" (B §3 last bullet)

- **A:** `done = target <= job_counter` on the OLD counter, so an item completes one call after the call that lands on the target (tested in `done_is_read_before_the_increment`).
- **B:** same `done` rule in its own words, but then "frames to complete = ceil(T / a) do_queue calls after queuing".
- **Evidence:** `Build/do_queue@0061e410.c`: `iVar19 = old counter; iVar17 = (T == 1) ? 1 : iVar19; bVar8 = local_20 <= iVar17; new = min(iVar17 + accel, T)`; handover only `if (bVar8)`. With T = 5000, a = 100: old counter is `(k-1)*100` on call k, so `done` first on k = 51 = T/a + 1, not 50.
- **Verdict: A right; B's closing formula is off by one** (its own `done` statement is correct). Nothing to change in A.

### D7. Minor factual corrections to A (no arithmetic change)

- **Wonder 0x21d is the Supercollider, not the Pyramids** (A: "the Pyramids — wonder `0x21d` — which zero research time"). Position map: 0x20e Pyramids, 0x20f Colossus, 0x210 Hanging Gardens, ..., 0x21d Supercollider, 0x21e Space Program. `train_time` zeroes `t` for `has_wonder(0x21d)` twice (tech path and the research-modifier tail), excluding 0x29a/0x29b and excluding unit/build types in the second. **B right.**
- **`game.semaphore` "bit 3"** is bit 3 of byte 1 of the `BitMask<256>` — i.e. bit 11 (`BitMask::set(..., 0xb, 1)`). Same predicate, different naming. See O4.
- **`Build::process` does not call `do_queue` "unconditionally"**: it returns before it when `!is_active` (`field_0x8 & 4`, `process@0061edf0.c:60-69`) and, for subclasses, when `is_neutralized` (vtable slot index 0x60 = byte +0x180 = `WallData::is_neutralized`; B mislabels this slot as `BuildOut::draw_underlays`/+0x60 — the index 0x60 is the pointer index, the byte offset is 0x180). Both readings agree on "every frame the building is processed".

## 2. B states, A omits (would change behaviour once modelled)

- **B1. Research modifiers apply only to research jobs.** `train_time@006508c0.c` lines ~470-560: RESEARCH_FASTER (×3/4), Angkor Wat, Relics, CtW Great Thinker loop, handicap, lobby tech-cost (×2/3 or ×3/2), science speedup, Supercollider zero — are reached only if the type is a tech (0x220..0x274) or a unit/build type whose tech bit is **clear**; a train job jumps to `LAB_006518f9`. A's "tail" lists these among the thirty without the partition. Since `production::train_time` takes the tail as input, the caller must never feed research-only modifiers to a train job. **Verified; doc should state the partition.**
- **B2. Ordinary-tech catch-up.** Non-age, non-epoch techs: `n = TypeData::count_discovered()` (TechType vtable +0x74); if `n > 0`: `t = (num_players - n + 1) * t / (num_players + 1)`; and the Greek research bonus is applied only on the age/epoch paths (`LAB_006515d7`), not this one. A only lists the age/epoch formula. **Verified.**
- **B3. Unassimilated-city ×5/4 tail** applied to everything: `train_time` end — Build in a city whose owner-race ≠ who and (`flags & 0x20` or build_flags bit 4 clear) → `t = t*5/4` (signed shift). **Verified** (`train_time@006508c0.c` lines 600-615).
- **B4. Missile-silo gate in `do_queue`**: `is(0x208,1) && inside_down >= 0 && tech bit set && type != DISBAND → return` (no advance). Argument confirmed from bytes at 0x61e4b6 (`push 1; push 0x208`). **Verified.**
- **B5. Non-first library's queue never advances** (`do_queue` first gate `is(0x1b3,0) && first_library != this->o && type != DISBAND → return`), and `queue_up` on a library with no first library (`get_first_library < 0`) refuses with return 1. **Verified.**
- **B6. Infinite-queue flag** (`build_masks & 0x40`): on a train job completing, `do_queue` clears the bit, `queue_up(type, 0)` and sets it back on success; `unqueue` clears it when the queue empties. A does not mention it. **Verified** in `do_queue` (`field_0x60 & 0x40`) and `unqueue` tail.
- **B7. Library done-path**: `finished()` then `if (r != 0 && (this->field_0x8 & 1)) unqueue(slot, 0)`; the counter is not pinned to T there (it already equals T). **Verified** (bytes 0x61ed84-0x61eda0).
- **B8. `Build::train` can fail** (`Objects::init_unit < 0`) but `finished` still returns 1 and the entry is removed. **Verified** (`finished@00628490.c`: `train(this,param_1); return 1;`).

## 3. B closes open questions A listed as "not established" (each verified)

- **O1. `queue_size`:** `Build/init@00629740.c:226-240` — 20 (0x14) if `BuildTypeData::is_military_trainer` (build_flags & 0x40000000) or `is_training_building` (& 0x80000000); else 10 if `build_flags & 0x08000000`; else 2. `BuildQueue/init@006307d0.c` accepts only 2/10/20. **Closed.**
- **O2. The availability bit at `leader + 0x6c18`:** `types.txt:42368` `+0x6c0c BitMask<806> tech`, `BitMask<806>` has `ptr` at +0xc → `0x6c18`. It is the leader's **tech** bitmask; `Leader::gain_tech@006dcb60.c:303` sets it. `LeaderData::type_avail@006e33a0.c` returns 2 when preq met but bit unset, 4 when set. **Closed.**
- **O3. `BuildData::can_make`:** `can_make@0062db10.c` — active; unless semaphore bit 11 or DISBAND/DEPOPULATE: not neutralized, not unassimilated; `BuildTypeData::queue_here(type)`; `type_avail(type, p) != 0`; techs: 0 if `has_tech` or `researching`; units with bit set and `build_flags & 0x10` clear: return type_avail (3 if unassimilated and ≥2); spells/city-upgrade special cases. **Closed as far as the queue needs** (queue_here/type_eligible internals are the tech tree's).
- **O4. `semaphore` bit:** `ConsoleWin/run_cmd@007d6a70.c:1639` `BitMask<256>::set(&game->semaphore, 0xb, 1)` on entering the scenario editor (after `ScenarioEditor::init`), `:1570` reset on leaving. Also skips `can_make`'s neutralized/unassimilated gate and makes `can_pay_cost` return 10. **Closed: scenario editor.**
- **O5. Build-time ceilings:** `train_time` has exactly one cap (`iVar12 = local_8 * 3`), no second. **Closed: one ceiling, ×3.**
- **O6. `construct_time`:** B confirms it is the foundation clock, not the queue path — agrees with A; still not implemented.
- **O7. Order of tail modifiers:** B gives the full order (train: rate_base, ramp, handicap, The President, Mongol, Japanese, Chinese, British, French, German, Roman, TROOPS_FASTER, speed upgrades, Cotton, SPIES_GENERALS, Wool, Monarchy, Socialism, wonders, Kremlin/INSTANT_UNIT_BONUS return; research: RESEARCH_FASTER, Angkor, Relics, CtW, handicap, tech_cost, science speedup, Supercollider; common: unassimilated ×5/4, max 1). Spot-checked against the decompile in the order listed — matches.

## 4. Doubly confirmed (A and B agree, and the decompile agrees)

- `QueueItem` = 0x14 bytes: `+0 int job_counter, +4 short type, +6 short[3] good, +0xc short[3] cost` (`types.txt:19615`). `BuildData+0x82 uchar queued`, `+0x88 BuildQueue {int queue_size; QueueItem* queue}`.
- `set_queue@006309f0`: writes type; zeroes counter unless arg 4; records the first **three** non-zero of six goods, pads the rest with `good=-1, cost=0`. `un_queue@00630ad0`: `memcpy(slot, slot+1, (n-slot-1)*0x14)`.
- `queue_up@00620f40`: library redirect to `get_first_library` (type != DISBAND); `can_pay_cost` (vt +0x84) → `QUEUE_COST`; `could_queue` (`can_make` → `QUEUE_CANT_TRAIN`; `queued >= queue_size` → `QUEUE_FULL`); **pop cap not a queue gate**; `semaphore byte1 & 8` → zero costs else `pay_cost` (vt +0xd4) into the 6-array; `slot = queued++; set_queue(slot, type, costs, 0)`; `queued==1` → re-zero slot 0 counter; `num_queued[type]++` at `leader+0x5a22` (reached as `who*0x6eec + 0xe3fdb2 + type*2`); barracks/stable/factory/dock/air tallies at `+0xa10..+0xa24` (units with attack≠0); `ages_queued/epochs_queued` at `+0x67f4/5`. **Price charged on queue.**
- `Objects::process_all@0065dce0`: the `(frame+i)%10` is leader iteration order; every active object's `process` runs every frame; `Build::process@0061edf0:289` calls `do_queue(0)` once per frame.
- `do_queue` accelerator: build type (0x19e..0x21e) or DISBAND → `accel_construct`; unit with tech bit → `accel_train` (+ `check_population` for messages/`pop_issues++` only); else `accel_research`; `ai_speed > 1` multiplies. All three load as 100 via `get_fraction(…,100)` (`Constants::init:750/752/754`).
- Counter: `done = T <= old` (T==1 ⇒ old treated as 1); `new = min(old + a, T)`.
- `train_time` base: unit with bit clear → `research_time` (vt +0x70): `TypeData::research_time` = `(time * research_tick_premium) >> 8` unsigned; `UnitTypeData::research_time` = `(that * research_premium_time) >> 8` signed; else `time` (vt +0x6c) = `job_time*100` (`res_time*100` for an unlearned spell). `RESEARCH_TICK_PREMIUM`/`RESEARCH_PREMIUM` via `get_fraction(…,0x100)` = 256 (`:1162/:1164`); `research_premium_time` via `String::fraction(…,0x100)` (`UnitType::init:604`, "2" → 512); `job_extra_time` hand-parsed `num*100/den` (`:584-599`, `1/10tsx` → 10); `JOB_TIME` plain `get_text_num` (`:562-566`).
- Ramp (bit set only): `t = unit_rate_base*t/100` (120); `cap = t*3`; `t = num_units[type]*job_extra_time*unit_rate_progression + t` (75); `if (t<0||cap<0) t=0 else t=min(t,cap)`. `num_units` = `ushort[352]` at `+0x5762`, reached as `+0x56fe + type*2`; maintained by `Leader::track_unit_type@006e0dd0` from `init_unit`/`Unit::close`/`Unit::set_type` — **live units only, not queued**. Citizen: 6000 + 750/owned, cap 18000.
- Age/epoch catch-up: `n` over 8 leaders (active; teammate +1, opponent +2) where `my_ages + my_ages_queued + 1 < theirs` (ages) / `my_ages + my_epochs + 3 < theirs` (epochs); `t = (2N - n + 2)*t/(2N + 2)`; Greek `/(greek_research_speed+100)` after. Science speedup `t -= ((L-base)*tech_science_speedup*t)/100` via the same `-0x51eb851f` magic; `TECH_SCIENCE_SPEEDUP/DISCOUNT` = 10 via `get_item` (`:1130/:1132`). `INSTANT_UNIT_BONUS` / Kremlin+Spy → `return accel_train*5 - 1`. Chinese `_INSTANT` → `return 1`. `max(1, t)` at the end.
- `finished@00628490`: bit set: `pop_cap(+0x7e4) < control_cost(+0x2f0) + control(+0x940)` → 0; caravan (`unit_flags2 & 8`) and `get_units >= get_caravan_limit` → −1; `is(0x1bf Airbase)` and not helicopter (`+0x2b4 & 0x20` clear) and `num_aircraft_here >= num_aircraft_limit` → −1; else `train` → 1. Positive → `unqueue(slot, 0)` (no refund); zero/negative → entry stays at T.
- `unqueue@006207c0`: library forwarding; with refund, walk forward while `queue[i+1].type == queue[i].type`; `job_counter = 0`; `num_queued--` (floored); category counters; if refund `unpay_cost` adds back exactly the recorded 3 pairs (`unpay_cost@006206e0`); `un_queue` if not last; `queued--`.
- `refund_cost@00620490` per pair: `base = cost*100/(100 - disc*d)`; `new = base - ((d+1)*disc*base)/100`; stock `+= cost - new`; `cost = new`; `d = science_level(+0x6eb8→+0xf4) - techtypes[type].age(+0x1c8)`.

## 5. What must change

**docs/PRODUCTION.md**
1. "The clock": the recursion into slot `i+1` bounded by `get_building_cities` is the **library's** branch of `do_queue` (third `is(LIBRARY)`); non-library buildings advance slot 0 only (plus the blocked-head redirections below). [D1]
2. "Completion": on `finished() <= 0` at slot 0, `do_queue` advances the first research entry (`get_next_non_unit`); on −1 with none found and `queued >= 2`, the first non-caravan (or, at an Airbase, first helicopter) entry. A pop cap does not block research entries behind the head. [D2]
3. "Completion": a unit-type entry whose bit is clear completes through `Leader::gain_tech` — sets the bit, trains nothing, and is not subject to the pop/caravan/aircraft checks. [D4]
4. `could_queue` special case is University + Scholar (0x1a4; 0x34/0x35), `count_queue + num_gatherers > 6`. [D3]
5. `refund_cost` runs only for tech entries of the first library's queue, on gaining a science-line (cat 3) epoch. [D5]
6. Wonder 0x21d is the Supercollider. Add: research-only modifier block vs train-only block (B1); ordinary-tech catch-up `(P - n + 1)/(P + 1)` with Greek only on age/epoch (B2); unassimilated ×5/4 common tail (B3); silo gate and infinite-queue flag as noted (B4, B6).
7. Strike open questions: `queue_size` (20/10/2 from `Build::init`), availability bit = `LeaderData::tech` BitMask set by `gain_tech:303`, `can_make` (read), semaphore bit 11 = scenario editor, single ×3 ceiling.

**crates/sim/src/production.rs / lib.rs**
1. `process_queues`: drop the per-building `parallel_slots(...).max(1)` fan-out; only a library (first library) uses `get_building_cities`; everything else advances slot 0. [D1]
2. `advance_slot`: on `Population`/`Limit` at slot 0, advance the first slot ≥ 1 whose entry is a research job (`!researched[ty]`) — and, once caravans/aircraft exist, the non-caravan/helicopter fallback on `Limit`. [D2]
3. `advance_slot`: a research entry (`!researched[ty]`) completing sets `researched[ty] = true`, removes the entry with no refund, spawns no unit, bumps no `by_type`/`control`/`by_group`, and skips `hand_over`. [D4]
4. `QueueFail::Full` doc comment: University/scholar rule, not dock/fishing boat. [D3]
5. When a tail is wired in: research-only modifiers must not be applied to train jobs (B1).
