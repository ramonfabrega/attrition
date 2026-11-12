# misery: second (blind) reading

Slug `misery`. Leaves: `pop_issues` (LeaderData +0x7e8, int), `misery` (LeaderData +0x7ec, int).
Reader: Claude Opus 5.5 (claude-opus-5-5), blind to the first reading, 2026-11-11.

Sources: the decompile export (`~/ghidra-projects/decomp`), the full `.text` listing
(`exe.s`, llvm-objdump of `riseofnations.exe`), `rise_z.map`, a raw byte scan of the PE,
and (for two observations only) the archive's `gamelog.txt` dumps under `~/ron-golden/`.

**Layout check.** `Leader` (0x6eec) has no field list in `types.txt`; `LeaderData` (0x6ee4)
sits at `Leader + 0` — `Leader::calc_attrition (006cdea0)` casts `(LeaderData *)this` with no
adjustment and writes `field_0x7f0` = `att`, and `Leader::Leader (006d2bf0)` puts its vbtable at
`+0x6ee4`, past the `LeaderData` body. So a `Leader *this->field_0x7e8` is `pop_issues` and
`field_0x7ec` is `misery`; global slot `i` is `0xe3a390 + i*0x6eec`, so the absolute forms are
`0xe3ab78` and `0xe3ab7c`.

**Nets run.** Export: `->pop_issues` / `->misery` (only `LeaderData::log_data (006e5110)`),
`field_0x7e8` / `field_0x7ec` (Leader, Leaders, Unit, Army files, plus non-leader classes),
`[0x1f8]`..`[0x1fb]` int-indexing (finds `Leaders::end_process_all (006ed070)`, base biased by
`+8`), `+ 0x7e8` / `+ 0x7ec` byte arithmetic (none on leaders). Listing: every `0x7e8(` /
`0x7ec(` / `0xe3ab78` / `0xe3ab7c` operand (30 lines; 0xe3ab7c has none), and every
read-modify-write or store at displacements `0x7d8..0x7e7` (to catch a base biased by up to
`+0x10`: only `site_mark`/`pop_cap` writers, all with an unbiased `Leader *`). Block writes:
no `0x6ee4`/`0x1bb9` size anywhere; the one leader memset is `Leaders::clear (006ed5d0)`.

## Leaf 1: `pop_issues` (LeaderData +0x7e8, int)

### 1. Writers

| writer | site (listing) | what it does |
|---|---|---|
| `Leaders::process_all (006ed2a0)` (body of `Leader::process (006b88b0)` inlined) | `6ed2ca movl $0, 0x7e8(%esi)` | clears it |
| `Leader::process (006b88b0)` | `6b88c6 movl $0, 0x7e8(%esi)` | same clear; **no caller** — no `call`/`jmp` to `0x6b88b0` in `.text`, no absolute `0x006b88b0` in the PE; it exists only as the inlined copy above |
| `Build::do_queue (0061e410)` | `61e5e2 incl 0xe3ab78(%eax)` | +1 (decompile line 192: `local_28[0x8218d]...curr_len + 1`, the export's field net misses it) |
| `Unit::process (00610bc0)` | `6111f9 incl 0xe3ab78(%esi)` | +1 |
| `Leader::init (006e3930)` | `6e4978 movl %ecx, 0x7e8(%ebx)` (`ecx` zeroed at `6e4949`) | sets 0 |
| `Leaders::clear (006ed5d0)` | `memset(leader + 0, 0, 0x6929)` per slot (`puVar1 = &field_0x6c14`, `- 0x1b05*4` = `+0`) | zeroes it with everything below `+0x6929` |

Non-leader hits rejected by `this` type: `Cliffs::~Cliffs`, `Constants::init`/`log_data`,
`Tribe::parse`, `GraphicPieces::*`, `ChatSpace::ChatSpace`, `ObjectData::train_time` and
`TypeData::get_cost` (both read `0x7e8`/`0x7ec` off `0xc061e4` = `GameAccessConst::constantsc`:
`mongol_stable_cost` / `mongol_stable_speed`).

### 2. The event, the step, the predicate

`pop_issues` counts, **within one frame**, how many production attempts of that leader were
turned away by the population cap. Step is **+1 per refused check**, unscaled (a count).

- **Building queues** — `Build::do_queue (0061e410)`, per slot visited. Reached from
  `Build::process (0061edf0)` as `do_queue(0)` (vtable `+0x1b4`, `61f3bf`) and re-entered by
  itself for `slot+1` (`61ecb0`, `61ed30`, `61ed78`: the next parallel slot, up to
  `get_building_cities` and the building's slot byte `+0x82`, and the `get_next_non_unit` /
  `get_next_helicopter` / `get_next_non_caravan` fallbacks). So one building can add more than 1
  per frame. Predicate, in order, for the slot's entry type `t`:
  the building has a queue (`+0x82 != 0`); it passed the two early returns (library/`DISBAND`,
  the occupied-silo hold); `t` is **not** a building type (`TypeData::is_build_type`, range
  `0x19e..0x21e`) and not `DISBAND` (`0x29a`); `t` is a unit-or-trainable type (`types[t]`
  vtable `+0xc` nonzero); `t`'s bit is set in the leader's `tech` BitMask (`LeaderData +0x6c0c`,
  bytes from `+0x6c18`; i.e. the type is *researched/available*, not a first-of-line research
  job); and `LeaderData::check_population (006e1330)` says `pop_cap < control_cost(t) +
  control` (`+0x7e4` < `UnitType +0x2f0` + `+0x940`, strict). **This sits where the slot's
  accelerator is chosen, before progress is added — it is not the completion test.** So it fires
  on *every* frame a researched unit entry is being trained while over the cap, not only when
  the clock is full. The accelerator stays `accel_train` either way (progress still accrues).
  Side effect on the console player only (UI): sets `game.info.player[play].flags |= 0x800`,
  stamps `pop_cap_frame`, red feedback plus `S_POP_LIMIT`, and a periodic `(id + frame) % 220`
  text bubble.
- **Carrier aircraft queue** — `Unit::process (00610bc0)`, listing `611158..6111f9`: a unit with
  `num_queued` (`UnitData +0xa0`) nonzero whose owner has `leader_flags & 2`; the type is
  `LeaderData::current_upgrade(0x134 = FIGHTERBOMBER)`; `queue_time` (`+0x64`) += `accel_train`
  (times `ai_speed` when > 1); once `queue_time >= train_time(t)`, if `control_cost(t) + control
  > pop_cap` then `pop_issues += 1` (same console-only 0x800 flag / message side effect). Here
  it is at completion only: the test is reached only on frames the clock is full.

### 3. What clears or resets it

- **Every frame**, at `Leader::process`'s head, inlined into `Leaders::process_all (006ed2a0)`,
  for each slot with `leader_flags & 2`; `Game::do_frame (00591ef0)` line 199, before
  `Leaders::strategy_all`, `Armies::process_all` and `Objects::process_all` (lines 267–275).
  The value at any instant is therefore the count so far this frame; at `Leaders::end_process_all`
  (line 279) and at log time it is the whole frame's count.
- `Leader::init (006e3930)` at game start (via `Leaders::init (006ed850)`); `Leaders::clear
  (006ed5d0)` from `Game::close (00594b80)` and `Game::run_oos_recovery (005869e0)`.
- A slot without `leader_flags & 2` is never cleared by `process_all`; `Unit::process` will not
  increment it either (same gate), but `Build::do_queue` does not test the flag. Whether a
  building of such a leader can reach `do_queue` was not read.

### 4. Who reads it

- `Leaders::end_process_all (006ed070)` (body of `Leader::end_process (006b8690)` inlined; that
  function also has no caller): if 0, clears `0x800` from every `game.info.player[]` row whose
  `who` is this leader; if nonzero and this is the console's leader, and more than 449 frames
  (`0x1c1`) since `pop_cap_frame`, and `pop_cap` is below the game's pop limit, re-issues the
  pop-limit feedback and sound. UI only.
- `Army::find_target (006f69b0)`, listing `6f72a7`, decompile line 269: when scoring a candidate
  target leader `c` for an army of leader `w` (`Army +0x94`), **if `w`'s own `pop_issues != 0`**,
  `c != w`, and either `w`'s stance word toward `c` (`LeaderData +0x74 + c*4`) or `c`'s toward
  `w` is 0, the candidate's score is shifted left by 2 (×4). A capped AI prefers to attack.
  Timing consequence (read, not diffed): `find_target` runs from `Army::do_marching (006f3df0)` /
  `Army::do_transporting (006f4690)` under `Army::process (006f93d0)`, whose callers are
  `Armies::process_all (006f3b00)` (do_frame line 272, **after** the clear and **before** the
  objects pass — it sees 0 for every leader with `leader_flags & 2`), `Armies::emergency
  (006f3250)` from `Object::do_damage` (inside `Objects::process_all`, so it sees a partial count
  that depends on the owner rotation), `Armies::diplo_change (006f30f0)` from `Leader::set_diplo`,
  and `Armies::send_navy (006f2c90)` from `find_target` itself. So the ×4 is live only on the
  in-frame paths.
- `LeaderData::log_data (006e5110)` (`LEADERDATA` dump) and the checksum walk (`crates/sim/src/
  checksum/leaders.rs:148` lists it inside the walked range).

### 5. Where `crates/sim` has the event

- Clear: `crates/sim/src/core/frame.rs:96-98`, the `Leader::process` head inside `Sim::tick`'s
  income loop, which already zeroes `self.combat.retargets[who]` (the original's `+0x9f4`, the
  very next store after `pop_issues` at `6ed2ca`/`6b88c6`). Note the crate loops every player;
  the original gates on `leader_flags & 2`.
- Building queue: `Sim::slot_plan` (`crates/sim/src/cities/queue.rs:1441`) is where the crate
  picks `Job::for_unit(researched)` — the same point as `do_queue`'s check; it is `&self`, so the
  increment lands in `Sim::advance_slot_with` (`queue.rs:1488`) or `advance_slot`, gated on
  "researched unit entry and `exceeds_population`" (`crates/sim/src/economy/cost.rs:1248`), **not**
  on `Advanced::Blocked` from `production::finished` (that is the completion-time check and
  misses every not-yet-full frame).
- Carrier: `Sim::carrier_queue_tick` (`crates/sim/src/air/carrier_queue.rs:137`), the
  `muster.cap < pop + control` branch at line 155.
- Reader: `TargetScoreView::f7e8` (`crates/sim/src/ai/army/target_score.rs:45`), whose live impl
  returns 0 at `target_score.rs:453`; the use is at `target_score.rs:143-146` and the seam is
  listed in `crates/sim/src/ai/army.rs:38`. `crates/rondata/src/diff/leader.rs:49` calls the
  counter "unread", which is not right: `find_target` reads it.
- `Leaders::end_process_all`: no counterpart (UI-only).

### 6. Reached by a plain game?

Yes. Any skirmish where a player queues researched units at the population cap reaches
`Build::do_queue`'s increment and the frame clear; carriers (a late naval unit) reach
`Unit::process`'s; the computer's armies reach `Army::find_target`. Nothing here is behind the
trigger layer, CtW or multiplayer, except `Leaders::clear` from `run_oos_recovery`.

**Dump evidence (archive, not a diff):** 948 `gamelog.txt` files print `pop_issues`; the busiest
show small bounded values held for hundreds of frames (run1401b: 924×0, 308×5; run1214: 804×0,
267×4, 1×3; run1175: 264×9; run881: 256×3), never a growing series — consistent with a
per-frame reset and a per-check count.

### 7. Confidence and gaps

High on the writers (both nets plus a biased-base sweep), the clear, the +1, and the
`check_population` predicate. Medium on the exact set of `do_queue` re-entries per frame (I read
the recursion conditions, not every fallback's internals). Not read: whether a non-`flags & 2`
leader's building can run `do_queue`; the checksum's call point within the frame; whether
`player.flags & 0x800` is read by any sim code; save/load (`LoadGame::verify_load` exists; no
whole-struct copy of `LeaderData` was found).

## Leaf 2: `misery` (LeaderData +0x7ec, int)

### 1. Writers

Every writer stores **zero**. There is no other value anywhere.

| writer | site (listing) | store |
|---|---|---|
| `Leader::calc_misery (006cdff0)` | `6cdff0 movl $0, 0x7ec(%ecx)`; the whole body is that store and `ret` | 0 |
| `Leader::calc_pop_cap (006dc490)` | `6dc49d movl $0, 0x7ec(%esi)`, its first act (decompile line 19) — `calc_misery` inlined | 0 |
| `Leader::gain_tech (006dcb60)` | `6de005 xorl %ecx,%ecx; 6de007 movl %ecx, 0x7ec(%ebx)` (decompile line 939) — `calc_misery` inlined | 0 |
| `Leader::init (006e3930)` | `6e497e movl %ecx, 0x7ec(%ebx)` (`ecx` zeroed at `6e4949`, decompile line 625) | 0 |
| `Leaders::clear (006ed5d0)` | the per-slot `memset(+0, 0, 0x6929)` | 0 |

`calc_misery` itself has no caller: no `call`/`jmp` to `0x6cdff0` in `.text` and no absolute
`0x006cdff0` in the PE (`rise_z.map` line 12750 names it, `leaders.obj`). It survives only as the
two inlined copies. The remaining listing hits on `0x7ec(` — `Cliffs::~Cliffs`,
`Constants::init` (`56ddc8`), `Constants::log_data`, `ObjectData::train_time` (`650c19`, off
`constantsc`), `Tribe::parse`, `GraphicPieces::close`/`~GraphicPieces`, the stack locals in
`573d4f`/`573d7d` — are other records. `0xe3ab7c` (the absolute form) occurs nowhere.

### 2. The event, the step, the predicate

None in effect. The name and the stub say a misery score was once recomputed alongside the
population cap and on tech gain; the shipped body is `misery = 0`. Events that run the inlined
reset:
- `calc_pop_cap` (18 call sites: `City::init`/`close`/`check_upgrade`, `Cities::capture_city`,
  `Build::activate`/`close`/`finished`, `Leader::calc_gather`, `Leader::gain_tech` ×4, the
  scenario editor's `add_build`/`toggle_ignore_popcap`, `ScenarioFuncSet::set_population_cap`);
- `gain_tech`, when the gained type's vtable `+0x28` and `+0x38` predicates hold (store sits
  before the `techtypes[t] +0x14 == 0` test that decides whether `calc_pop_cap` follows).
Step: none (0 → 0). Units: none.

### 3. What clears or resets it

All of the above; it is never anything but 0 from `Leader::init` on.

### 4. Who reads it

Only `LeaderData::log_data (006e5110)` (the `LEADERDATA` dump, decompile line 1420,
`pushl 0x7ec(%edi)` at `6e882b`) and the checksum walk over `[+0x8, +0x692a)`
(`crates/sim/src/checksum/leaders.rs:149`). No game-logic reader: the export's field net and the
listing's operand net both come back empty for leader-typed bases.

### 5. Where `crates/sim` has the event

Nothing to port: a builder holds a constant 0 for the leaf (`Sim::leader_image`'s `misery` leaf
can be `Some(0)` rather than `None`). The closest site, if one wanted the store for form's sake,
is the crate's `calc_pop_cap` (`crates/sim/src/economy/cost.rs`, `pop_cap`), but there is no
state to keep. `crates/rondata/src/diff/leader.rs:44` lists it as unmodelled ("the misery
counter is unread") — correct as to readers; it could be compared as 0.

### 6. Reached by a plain game?

Yes: `calc_pop_cap` runs on every city founding and building completion; `gain_tech` on every
tech. Both only ever write 0.

**Dump evidence (archive, not a diff):** across the 948 `gamelog.txt` files under `~/ron-golden/`
that print `LEADERDATA`, `rg '^\s*misery [^0]'` finds **no** line — every printed `misery` is 0.

### 7. Confidence and gaps

High. Five writers, all zero, by both nets plus the biased-base sweep (`0x7d8..0x7e7` stores
show only `site_mark`/`pop_cap` on leaders). Not read: save/load restoration of the leaders
array (none found as a block copy).

## Summary

| leaf | writers (Name (addr)) | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `pop_issues` +0x7e8 | `Build::do_queue (0061e410)` +1; `Unit::process (00610bc0)` +1; `Leaders::process_all (006ed2a0)` =0 (inlined `Leader::process (006b88b0)`, which has no caller); `Leader::init (006e3930)` =0; `Leaders::clear (006ed5d0)` memset 0 | +1 per population-cap refusal: in `do_queue` on every frame a researched unit entry is trained over the cap (`check_population`, `pop_cap < control_cost + control`, before progress, possibly several per building per frame); in `Unit::process` when a carrier's `FIGHTERBOMBER` clock is full and the cap refuses | every frame at the `Leader::process` head for `leader_flags & 2` slots (before AI, armies and objects); game init; `Game::close` / OOS recovery | clear: `core/frame.rs:96` beside `combat.retargets`; inc: `cities/queue.rs` `advance_slot_with`/`slot_plan` (not `finished`), `air/carrier_queue.rs:155`; reader: `ai/army/target_score.rs:453` (`f7e8` returns 0) | yes (queues at cap; carriers; AI armies) | high (writers, step, clear); medium (re-entry count per building) |
| `misery` +0x7ec | `Leader::calc_misery (006cdff0)` (no caller; inlined into) `Leader::calc_pop_cap (006dc490)` and `Leader::gain_tech (006dcb60)`; `Leader::init (006e3930)`; `Leaders::clear (006ed5d0)` — all store 0 | none (always 0) | always 0 | none; hold constant 0 | yes, but only 0 is ever written | high; archive dumps show no nonzero `misery` |
