# Item 4152, family 12 (`misery`) — first reading

- Family: 12, slug `misery`
- Leaves: `pop_issues` (`LeaderData` +0x7e8, int), `misery` (`LeaderData` +0x7ec, int)
- Reader: Opus 5.5 (claude-opus-5-5), 2026-11-11
- Sources: the decompile export (`~/ghidra-projects/decomp/funcs`, `types.txt`), the listing of `riseofnations.exe` (`llvm-objdump`), `crates/sim/src`.

## Ground the offsets stand on

- `types.txt:42086` `Leader` is size `0x6eec` with no members listed; `LeaderData` (`types.txt:42088`) is `0x6ee4`, with `pop_cap` +0x7e4, `pop_issues` +0x7e8, `misery` +0x7ec, `att` +0x7f0. In every `Leader::` method the export's `this->field_0xNNN` is the `LeaderData` offset with **no prefix** (`Leader::calc_pop_cap (006dc490)` reads `this->field_0x8` as `who` and writes `field_0x7e4` as `pop_cap`; `Leader::calc_anti_attrition (006cdcc0)` writes the float `0x43800000` = 256.0 to `field_0x7f4`, `anti_att`'s type). The 8 extra bytes of `Leader` are at its end.
- The array: `Leaders::walk_data (006e38e0)` walks `LeaderData::walk_data` from `&leaders` to `0xe789dc` in steps of `0x6eec` (9 slots), so slot 0's `LeaderData` is at `0xe3a390`; `pop_issues` of slot `i` is `0xe3ab78 + i·0x6eec`, `misery` is `0xe3ab7c + i·0x6eec`.
- Nets run: `grep -- '->pop_issues\b'`, `'->misery\b'` (only `LeaderData::log_data (006e5110)`); `grep 'field_0x7e8\b\|field_0x7ec\b'`; int-index forms `[0x1f8]`..`[0x1fb]` (a `uint*` at `who` or at the base) and short-index `[0x3f4]`..`[0x3f7]` over the 2,530 export files that name a leader; byte-offset forms `+ 0x7e8)`/`+ 0x7ec)`. Listing: every operand `0x7e8(`, `0x7ec(`, `0xe3ab78`, `0xe3ab7c` (non-`ebp`/`esp`), and every *write* whose memory destination has a displacement in `0x7d8..0x7ec` (to catch a base biased by `+8`, `who`), each attributed to its function through `INDEX.tsv`. `0xe3ab7c` (absolute `misery`) occurs **nowhere** in `.text`.
- Non-leader hits, ruled out by the base register's type: `Cliffs::~Cliffs (00436120)`, `GraphicPieces::close (008feed0)`, `GraphicPieces::clear_local_data (0090c670)`, `GraphicPieces::inc_time (0090cac0)`, `GraphicPieces::~GraphicPieces (0090e720)`, `ChatSpace::ChatSpace (0093e8d0)` (each `this` of its own class); `Constants::init (00569a90)`, `Constants::log_data (00570170)`, `ObjectData::train_time (006508c0)`, `TypeData::get_cost (00664090)`, `Tribe::parse (006f0560)` (base `[0xc061e4]`, `GameAccess::constants`: `Constants +0x7e8 mongol_stable_cost`, `+0x7ec mongol_stable_speed`). `Leader::make_stuff (006c8af0)` and `Leader::compute_sites (006cc950)` write `0x7e0(this)`, which is `site_mark`, not a `who`-biased `pop_issues`.

## Leaf 1 — `pop_issues` (`LeaderData +0x7e8`, int)

A per-frame count of the times the leader was refused a unit by the population cap. It is zeroed at the head of the leader's frame step, stepped by +1 inside `Objects::process_all`, read by the end-of-frame step, and is never decremented.

### 1. Writers, both nets

| writer | store | what |
|---|---|---|
| `Leaders::process_all (006ed2a0)` | `006ed2ca movl $0x0, 0x7e8(%esi)`, `esi` walking `&leaders` by `0x6eec` | **= 0** (the frame's zero; this is `Leader::process`'s head inlined) |
| `Leader::process (006b88b0)` | `006b88c6 movl $0x0, 0x7e8(%esi)` | = 0; the out-of-line copy of the same head. **No caller**: no `call 0x6b88b0` in `.text` and the four bytes `b0 88 6b 00` occur nowhere in the file. Dead as a separate body; live only as inlined into `process_all` |
| `Unit::process (00610bc0)` | `006111f9 incl 0xe3ab78(%esi)`, `esi = owner·0x6eec` | **+1** (the carrier arm, below) |
| `Build::do_queue (0061e410)` | `0061e5e2 incl 0xe3ab78(%eax)`, `eax = owner·0x6eec` (`-0x24(%ebp)`) | **+1** (the building-queue arm, below). The export prints this store as `local_28[0x8218d].string.curr_len += 1` — a mistyped local; the listing settles it |
| `Leader::init (006e3930)` | `006e4978 movl %ecx, 0x7e8(%ebx)`, `ecx` zeroed at `006e4949 xorl %ecx,%ecx` | = 0, straight-line after the `say_checksum` of `leaders.cpp` line `0x33ab` |
| `Leaders::clear (006ed5d0)` | `memset(slot, 0, 0x6929)` per slot (`006ed5e0`–`006ed5ee`, base `0xe40fa4 − 0x6c14`) | = 0 for `[0, 0x6929)` of each of 10 slots |
| `LeaderData::walk_data (006d6750)` | the `DataWalk` callback over `[&who, &field_0x692a)` when `leader_flags & 1` | on input (a save loaded), whatever the file holds |

### 2. The events, the step, the predicates

**(a) The frame's zero** — `Leaders::process_all (006ed2a0)`, called unconditionally from `Game::do_frame (00591ef0)` (`0059224f`). For each slot with `leader_flags & 2`, first statement of its body: clear `leader_flags` bit `0x40000`, `pop_issues = 0`, `field_0x9f4 = 0`. A slot without bit 2 keeps its last value.

**(b) A carrier's plane held at the cap** — `Unit::process (00610bc0)`, `00611158`–`006111f9`, per unit per frame. Predicate, in order:
- `UnitData +0xa0 num_queued` (short) ≠ 0, and the owner's `leader_flags & 2`;
- `t = LeaderData::current_upgrade(owner, 0x134)` (`0x134` = `FIGHTERBOMBER`, the line's current type);
- `queue_time (+0x64) += accel_train` (`[constants]+0x228`; times `*ai_speed` when that exceeds 1);
- `queue_time >= train_time(t)` (`ObjectData::train_time (006508c0)`), else nothing;
- `pop_cap < control_cost(t) + control`, strictly (`UnitType +0x2f0 control_cost`, `LeaderData +0x940 control`; `jle 0x6112d3` takes the equal case to the launch). Then **`pop_issues += 1`** and `queue_time -= ` the same accel, so the clock is pinned at the threshold and the test repeats next frame: **+1 per frame per capped carrier** for as long as it stays capped.
- The console player's message (sets `game.info.player[play].flags |= 0x800` and `pop_cap_frame = frame` once, plays `S_POP_LIMIT`) rides the same branch and is no sim state.

**(c) A building's unit slot visited at the cap** — `Build::do_queue (0061e410)`, `0061e577`–`0061e5e2`. `do_queue` is vtable slot `+0x1b4` of `Build`; `Build::process (0061edf0)` calls it with slot 0 every frame, and it recurses into later slots (`param_1 + 1` while below `get_building_cities` and the queue's length; and, from slot 0 only, the `get_next_non_unit` / `get_next_non_caravan` / `get_next_helicopter` fallbacks). After its early returns (a library slot not at the first library; a research building whose item's tech bit is already set), with `t` the slot's `TypeIndex`:
- `t` is not a build type (`0x19e ≤ t < 0x21f` goes to `accel_construct`) and not `DISBAND` (`0x29a`);
- `is_unit_type(t)` (vtable `+0xc`), **and** bit `t` set in the owner's `tech` mask (`LeaderData +0x6c0c BitMask<806>`, bits inline at `+0x6c18`; `testb %dl, 0xe40fa8(%esi,%ecx)`). A clear bit is a unit-line upgrade being *researched* and takes `accel_research` with no population test;
- `LeaderData::check_population(t)` (`006e1330`: `pop_cap < control_cost(t) + control`) nonzero → **`pop_issues += 1`**, then the console-only message (flag `0x800`, `pop_cap_frame`, strings `+0xa28`/`+0xa14`).
- The test sits **before** the slot's clock is advanced, so it fires on every visit while the item is at the cap, whatever its progress — not only at completion. A building can add more than one per frame through the recursion.
- `Console::hurry_action (007c3810)` also calls `do_queue(0)` (vtable `+0x1b4`); its callers are `Console::exec_keymap (007c2ce0)` and `ConsoleWin::run_cmd (007d6a70)`, a debug/cheat path outside the order stream.

Arithmetic: integer compares of populations (control units) and a +1; no float, no scale.

### 3. What clears it

The frame's zero (a) for every slot with `leader_flags & 2`; `Leader::init (006e3930)`; `Leaders::clear (006ed5d0)`; a save loaded (`LeaderData::walk_data`). `Leader::reset_score (006e37f0)` does not touch it (no hit in either net).

### 4. Who reads it

- **`Leaders::end_process_all (006ed070)`** (`Game::do_frame` `00592488`, after `Objects::process_all`), inlined `Leader::end_process`; the read is `puVar4[0x1f8]` with `puVar4 = &who`, i.e. displacement `0x7e0` off a `who`-biased pointer (the out-of-line `Leader::end_process (006b8690)`, `006b86ab cmpl $0x0, 0x7e8(%ecx)`, has no caller). Zero → clear `game.info.player[k].flags` bit `0x800` for the player(s) whose `who` matches; nonzero and the console's leader → every `> 0x1c1` (449) frames since `pop_cap_frame`, re-stamp it and, if `pop_cap` is below the lobby's `pop_limits[pop_limit]+0x3c`, the red "pop limit" feedback and `S_POP_LIMIT`. UI only.
- **`Army::find_target (006f69b0)`**, `006f72a7 cmpl $0x0, 0xe3ab78(%eax)`, `eax = army.who·0x6eec` (`ArmyData +0x94 who`): in the per-candidate-leader valuation, if the **army's own** leader has `pop_issues ≠ 0`, the candidate `c` is not that leader, and either stance word `LeaderData +0x74 + 4·c` of the army's leader or the reverse is 0, the candidate's value is shifted left 2 (**×4**). This changes a decision. Timing matters: `Game::do_frame` runs `Leaders::process_all` (the zero) → `Leaders::strategy_all` → `GameDaemon::process_all` → `Armies::process_all` → `Objects::process_all` (the steps) → `Leaders::end_process_all`. `find_target`'s callers are `Army::do_marching (006f3df0)` and `Army::do_transporting (006f4690)`, reached from `Army::process (006f93d0)`, whose callers are `Armies::process_all`, `Armies::send_navy`, `Armies::diplo_change` (from `Leader::set_diplo (006ec6a0)`) and `Armies::emergency` (from `Object::do_damage (0064a480)`). Through `Armies::process_all` the count has just been zeroed and nothing has stepped it, so the ×4 cannot fire there for a bit-2 leader; it can fire only through `Armies::emergency` during `Objects::process_all`, after one of the owner's capped producers has already been processed that frame (or through a `set_diplo` reached at that point). Reading-level; no capture checked.
- `LeaderData::log_data (006e5110)` (`006e8795 pushl 0x7e8(%edi)`), called from `GameLog::dump_leaders`, from `GameLog::dump_all` / `GameLog::full_dump`: the dump prints it. Where `dump_all` sits relative to `end_process_all` was not traced.
- `LeaderData::walk_data` (save). `crates/sim/src/checksum/leaders.rs:148` lists it in the crate's checksum layout; not re-checked here.

### 5. Where `crates/sim` has the events

- The zero: `Sim::tick`, `crates/sim/src/core/frame.rs` ~line 97, beside `self.combat.retargets[who] = 0`, whose comment already cites `Leader::process (006b88b0)`'s head (the `field_0x9f4` zero of the same three-store head).
- (b): `Sim::carrier_queue_tick`, `crates/sim/src/air/carrier_queue.rs:155`, the `muster.cap < price.pop + muster.control` arm that takes `queue_time -= accel` — the +1 goes there. The module's SEAMS list already names the `0x800` flag.
- (c): the crate's `do_queue` is `Sim::slot_plan` (`crates/sim/src/cities/queue.rs:1441`) and `Sim::advance_slot_with` (`:1488`). The crate tests the cap **only at completion** (`production::finished` → `Handover::Population` → `Advanced::Blocked`, `:1575`), which is `Build::finished (00628490)`'s test, not `do_queue`'s per-visit one. A builder adds the per-visit `check_population` (researched unit type, not build type, not disband) to `slot_plan`'s caller for each visited slot — the `plans` loop at `:1360` and the single-slot path at `:1434` — and must reproduce `do_queue`'s visit order (the recursion and the fallbacks) to count the same visits.
- The reader: `crates/sim/src/ai/army.rs:38` seams `pop_issues` (the ×4 is ×1). `crates/rondata/src/diff/leader.rs:49` keeps it as an unread dump key.

### 6. Plain game?

Yes for all three live writers. (a) every frame. (c) any skirmish where a player at the population cap has a trained unit type queued — the computer reaches the cap routinely. (b) needs an Aircraft Carrier with planes queued at the cap (a late-age naval game; the AI queues them through `Group::action_spell`, per `carrier_queue.rs`'s header, not re-read). `Console::hurry_action` is a debug path. The `find_target` ×4 is plain-game but narrow (§4).

### 7. Confidence and what is not read

High on the writer list and the predicates of (a)–(c): every store is listing-backed and each predicate was read in the listing around the store. Not read: `do_queue`'s fallback-selection functions (`BuildQueueData::get_next_non_unit` and siblings) and `get_building_cities`, so the exact number of visits per frame per building is not established; the slot's clock behaviour after a capped visit (whether progress is clamped) was not read; `Object::do_damage`'s call to `Armies::emergency` and its predicate were not read, so the reachability of the ×4 is a structural inference; `GameLog::dump_all`'s place in the frame was not traced. A store through a base biased by something other than `+0`/`+8` would escape the listing net; the export's typed-index net found only the `who`-biased read.

## Leaf 2 — `misery` (`LeaderData +0x7ec`, int)

**Every store writes zero.** The field is zeroed at creation and re-zeroed by three events; nothing in `.text` ever writes a nonzero value, and nothing but the dump and the save reads it. It is constant 0 in every reachable state (a save restores the 0 it saved).

### 1. Writers, both nets

| writer | store | value |
|---|---|---|
| `Leader::calc_misery (006cdff0)` | `006cdff0 movl $0x0, 0x7ec(%ecx); ret` — the whole body | 0. **No caller**: no `call 0x6cdff0`, the bytes `f0 df 6c 00` occur nowhere in the file, and `rise_z.map` gives the address that one name only (`?calc_misery@Leader@@QAEXXZ`, `leaders.obj`) |
| `Leader::calc_pop_cap (006dc490)` | `006dc49d movl $0x0, 0x7ec(%esi)`, its first store, before every branch | 0 |
| `Leader::gain_tech (006dcb60)` | `006de007 movl %ecx, 0x7ec(%ebx)`, `ecx` from `006de005 xorl %ecx,%ecx` | 0 (the inlined body of `calc_misery`) |
| `Leader::init (006e3930)` | `006e497e movl %ecx, 0x7ec(%ebx)`, `ecx = 0` | 0 |
| `Leaders::clear (006ed5d0)` | the per-slot `memset(…, 0, 0x6929)` | 0 |
| `LeaderData::walk_data (006d6750)` | the `[&who, &field_0x692a)` block on input | the saved value |

No absolute-address store (`0xe3ab7c`) exists, and no `who`-biased (`0x7e4(`) or typed-index (`[0x1f9]`/`[0x1fb]`) store to it was found.

### 2. The events and their predicates

- **`calc_pop_cap`**, at entry, on every call. Its callers (listing): `Build::activate (00623e20)` ×2, `Build::finished (00628490)`, `Build::close (00628980)` ×2, `Leader::calc_gather (006ceee0)`, `Leader::gain_tech` ×4, `Cities::capture_city (00733380)`, `City::init (00737050)`, `City::close (00737550)`, `City::check_upgrade (00738b20)`, `ScenarioEditor::add_build (0099cfd0)`, `ScenarioEditor::toggle_ignore_popcap (009a3a40)`, `ScenarioFuncSet::set_population_cap (009e8ef0)`.
- **`gain_tech`, the epoch arm** (`006ddfe2`–`006de01a`): when the gained type passes `is_tech_type` (vtable `+0x28`, `TypeData::is_tech_type (004706c0)`: `0x220 ≤ t < 0x275`; if not, the function jumps to its tail) **and** `is_epoch_type` (vtable `+0x38`, `TypeData::is_epoch_type (00470870)`: `0x227 ≤ t < 0x243`, 28 types; both checked by reading the vtable bytes at `TypeData::vftable 0x00b43e48` and `TechTypeData::vftable 0x00b43fe4`, which hold the same two targets, and the two bodies). Then `misery = 0`; then, if `techtypes[t] +0x14` (the tech's category) is 0, `calc_pop_cap`, which zeroes it again. Not gated on a first gain: the `has_tech` block at export line 220 is a separate statement and does not enclose this one.
- `init` / `clear`: a new game.

### 3. What clears it

All of the above; there is nothing else to clear.

### 4. Who reads it

`LeaderData::log_data (006e5110)` (`006e882b pushl 0x7ec(%edi)`, the dump), and `LeaderData::walk_data` (save). No load of `0x7ec(` off a `Leader*` in any other function; no decision reads it. `crates/sim/src/checksum/leaders.rs:149` lists it in the crate's checksum layout.

### 5. Where `crates/sim` has the events

None needed: a field constant at 0 is reproduced by any port that emits 0. If a builder wants the stores for symmetry: `Tech::gain`'s epoch arm (`crates/sim/src/tech.rs` ~line 1970, `Kind::Epoch` — note the crate's arm there runs only on a first gain, the original's zero does not, which is moot for a constant) and the crate's `calc_pop_cap` image (`crates/sim/src/economy.rs:305` cites its tail). `crates/rondata/src/diff/leader.rs:44` keeps it as an unread dump key; a diff of it against the dump would be a cheap constant-0 assertion.

### 6. Plain game?

`calc_pop_cap`, `gain_tech`'s epoch arm, `init` and `clear` are all reached by a plain skirmish; the `ScenarioEditor`/`ScenarioFuncSet` callers are the editor and the trigger layer. `calc_misery` is reached by nothing.

### 7. Confidence and what is not read

High that the field is written only with 0: both nets agree, the absolute address never appears, and every store's source register was traced to an `xor` or an immediate. Not read: whether some other biased base (not `+0`, not `+8`) stores to it — the export's typed forms showed none. That `misery` was once a live mechanic whose body was emptied before ship (the name `calc_misery`, the zero in `gain_tech` where a recompute would sit) is an inference from shape, not established.

## Summary

| leaf | writers (Name (addr)) | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `pop_issues` +0x7e8 | `Leaders::process_all (006ed2a0)` (=0; `Leader::process (006b88b0)` is its uncalled copy); `Unit::process (00610bc0)` (+1); `Build::do_queue (0061e410)` (+1); `Leader::init (006e3930)`; `Leaders::clear (006ed5d0)`; `LeaderData::walk_data (006d6750)` (load) | +1 per capped carrier per frame (`num_queued≠0`, bit 2, clock at `train_time`, `pop_cap < control_cost(FIGHTERBOMBER line) + control`); +1 per `do_queue` visit of a researched unit-type slot with `check_population` true, at any progress | 0 at the head of each bit-2 leader's frame step, before armies and objects; init; clear | zero: `Sim::tick` (`core/frame.rs` ~97); carrier: `carrier_queue_tick` (`air/carrier_queue.rs:155`); building: per-visit test missing — `slot_plan` callers (`cities/queue.rs:1360`, `:1434`); reader seamed at `ai/army.rs:38` | yes (carrier arm needs a late naval game; `hurry_action` is debug) | high on stores and predicates; visit count per building and the ×4's reach not fully read |
| `misery` +0x7ec | `Leader::calc_pop_cap (006dc490)`; `Leader::gain_tech (006dcb60)` (epoch arm); `Leader::init (006e3930)`; `Leaders::clear (006ed5d0)`; `Leader::calc_misery (006cdff0)` (uncalled); `LeaderData::walk_data (006d6750)` (load) | none — every store is 0 | everything above | none needed (constant 0); epoch arm at `tech.rs` ~1970 | yes (all live writers) | high |
