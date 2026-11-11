# Item 4152, family 12 (`misery`): adjudication

- Slug: `misery`. Leaves (`LeaderData`): `pop_issues` (+0x7e8, int),
  `misery` (+0x7ec, int).
- Readings: `misery-reading.md` (first), `misery-second.md` (blind second).
- Adjudicator: Claude Opus 5.5 (claude-opus-5-5), 2026-11-11.
- Method: the export (`~/ghidra-projects/decomp`), the full `.text`
  listing (`exe.s` in the job scratch), `crates/sim` grep. Nothing run of
  the original.

(Written per leaf as settled.)

## Nets re-run by the adjudicator

- Listing: every `0x7e8(`, `0x7ec(`, `0xe3ab78`, `0xe3ab7c` operand off a
  non-frame register (30 lines). Leader-typed hits are exactly the ones both
  readers list; the rest (`4361a6`/`4361b0` `Cliffs::~Cliffs (00436120)`,
  `56ddad`/`56ddc8`/`57ac48`/`57ac85` `Constants`, `650c19`
  `ObjectData::train_time (006508c0)`, `664639` `TypeData::get_cost (00664090)`,
  `6f0b4e`/`6f0b59` `Tribe::parse (006f0560)`, `8ffb6a`..`90ecc9`
  `GraphicPieces`, `93e9a5` `ChatSpace`) are other records. `0xe3ab7c`
  appears nowhere.
- Biased bases: every store or read-modify-write at displacement
  `0x7e0..0x7e4`. Leader-typed ones are `site_mark` (`6c916e`
  `Leader::make_stuff (006c8af0)`, `6cce60` `Leader::compute_sites (006cc950)`,
  `6e4e3c` `Leader::init (006e3930)`) and `pop_cap`, all off an unbiased
  `Leader*`; the others (`81abc0`, `922ecd`, `92366c`, `93e268`) are
  `IFaceResources`, `RTreeData`, `ChatSpace`. The absolute forms
  `0xe3ab70`/`0xe33c8c` (a `+8`-biased or a 1-based index) occur nowhere;
  `0xe3ab74` hits are all reads of `pop_cap`.
- Export: typed stores `field_0x7e8`/`field_0x7ec`/`[0x1f8..0x1fb]` over
  every leader-naming file: `Leader::process`, `calc_misery`, `calc_pop_cap`,
  `gain_tech`, `init`, `Leaders::process_all`, the same set.
- Callers of the four no-caller claims (`call`/`jmp` to `0x6b88b0`,
  `0x6b8690`, `0x6cdff0`): none. `calc_pop_cap (006dc490)`: 17 call sites.

**The writer lists of both readers are complete**, with one addition
the second missed: the save loader (below).

## Leaf 1: `pop_issues` (+0x7e8, int)

### Claims lined up

| # | claim | first | second | mark | verdict |
|---|---|---|---|---|---|
| 1 | `Leaders::process_all (006ed2a0)` zeroes it per frame for slots with `leader_flags & 2`, the inlined head of `Leader::process (006b88b0)` | yes | yes | agree | **confirmed**: `process_all:18`/`:22`, listing `6ed2ca` |
| 2 | `Leader::process (006b88b0)` has no caller | yes | yes | agree | **confirmed**: no `call`/`jmp 0x6b88b0` in `exe.s` |
| 3 | `Unit::process (00610bc0)` +1, carrier arm: `num_queued` (short +0xa0) ≠ 0, owner `leader_flags & 2`, `t = current_upgrade(FIGHTERBOMBER 0x134)`, `queue_time += accel_train (×ai_speed if >1)`, `queue_time >= train_time(t)`, then `pop_cap < control_cost(t) + control` strict | yes | yes | agree | **confirmed**: `611158`..`6111f9`; the strict test is `6111e7 cmpl 0xe3ab74,%eax; jle 0x6112d3` (equal launches) |
| 4 | the carrier test repeats every frame while capped (clock pinned) | "+1 per frame per capped carrier" | "at completion only: reached only on frames the clock is full" | disagree in emphasis | **confirmed (first)**: `6112c8 subl -0x10(%ebp), 0x64(%ebx)` takes back exactly the accel added at `6111bd`, so the clock sits at its last value, at or past `train_time`, and the next frame's add re-crosses: +1 every frame for as long as the cap holds. The second's sentence is true but undersells it |
| 5 | `Build::do_queue (0061e410)` +1 per visit of a slot whose type is not a build type (`0x19e..0x21e`), not `DISBAND` (`0x29a`), `is_unit_type` (vtable +0xc), tech bit set in `LeaderData +0x6c18`, and `check_population (006e1330)` true | yes | yes | agree | **confirmed**: listing `61e543`..`61e5e2`; `6e1330` is the strict `pop_cap < control_cost + control` (same test as `Build::finished:33`) |
| 6 | the test sits before the slot's clock is advanced, any progress | yes | yes | agree | **confirmed**: the increment is in the accelerator choice (`do_queue:190`); the clamp of progress is after (`do_queue:263`–`:281`) |
| 7 | after a capped visit, progress still accrues at `accel_train` | not read (open) | yes | one-sided | **confirmed (second)**: `do_queue:230` resets the accelerator to `local_14 = accel_train` on the capped arm too; progress is `min(progress + accel, train_time)` (`:276`–`:281`). The clock fills and then `Build::finished` refuses it on the same pop test (`finished:33`, returns 0) |
| 8 | early return: building has no queue (`queued`, BuildData +0x82, == 0) | no | yes | one-sided | **confirmed**: `61e433 cmpb $0x0, 0x82(%ebx); je` exit; `types.txt` names +0x82 `queued` |
| 9 | early return: a library that is not the leader's first library, entry not `DISBAND` | yes | yes ("library/DISBAND") | agree | **confirmed**: `61e465`..`61e4ae`, `is(0x1b3 LIBRARY, 0)` then `get_first_library (006db6c0)` vs the building's `o` |
| 10 | the second early return | "a research building whose item's tech bit is already set" | "the occupied-silo hold" | disagree | **corrected (first wrong, second right)**: `61e4b4`..`61e508` is `is(0x208 MISSILESILO, 1)`, `inside_down` (ObjectData +0x28) `>= 0`, the entry's tech bit set, not `DISBAND`. A silo with a missile inside; the crate already has it (`cities/queue.rs` `slot_plan`, "A silo with a missile inside holds its clock") |
| 11 | `do_queue` recurses to slot+1 while below `get_building_cities` and `queued` | yes, unconditional | yes, unconditional | agree, **both wrong** | **corrected**: the slot+1 recursion is gated on `is(0x1b3 LIBRARY, 0)` (`61eb79`..`61eb9b`, `jne 0x61ed49` to the `get_building_cities` arm); only the first library has parallel slots |
| 12 | fallbacks from slot 0: `get_next_non_unit`, then `get_next_helicopter` / `get_next_non_caravan` | yes | yes | agree | **confirmed with a predicate both omitted**: the helicopter/caravan pick is `is(0x1bf AIRBASE, 0)` (`61ece4`), not the library test; it runs only when `finished` returned **negative** (`61ecd6`) and `queued >= 2` |
| 13 | one building can add more than one per frame | yes | yes | agree, **both wrong** | **corrected: at most +1 per building per frame**. A library's extra slots carry techs, which fail `is_unit_type` and take `accel_research`. A non-library building visits one more slot only from slot 0 and only after `finished` refused the head. `finished` runs the pop test first (`finished:33`, return 0) and the caravan and aircraft limits after it (`:51`, `:66`, return -1). So a counted head always yields 0 → `get_next_non_unit (00630b10)`, which returns only a non-unit or unresearched slot (`get_next_non_unit:43`, `has_tech == 0`), and that slot cannot count. Only an uncounted head refused by a limit (-1) leads to a fallback slot that can count. The archive agrees: across 948 dumps `pop_issues` never exceeds 9 |
| 14 | `do_queue` itself does not test `leader_flags & 2` | no | yes | one-sided | **confirmed** (no leader-flag load in `0061e410`..`0061edf0`). Whether a leader without bit 2 still has buildings reaching `do_queue` is **open**. Such a leader's count is never cleared, so it would only grow; a dump of a defeated slot settles it |
| 15 | the console-only side effects (flag `0x800`, `pop_cap_frame`, feedback, `S_POP_LIMIT`, the `(o + frame) % 220` bubble) | yes (minus the bubble) | yes | agree | **confirmed**, UI only (`do_queue:197`–`:255`) |
| 16 | `Console::hurry_action (007c3810)` also calls `do_queue(0)` (debug path) | yes | no | one-sided | **confirmed** by the first's listing citation; not re-traced. Outside the order stream |
| 17 | resets: `Leader::init (006e3930)` `6e4978`, `Leaders::clear (006ed5d0)` memset `0x6929` from `0xe3a390` per slot | yes | yes | agree | **confirmed**: `6e4949 xorl %ecx,%ecx` and ecx untouched to `6e4978`; `6ed5e0`..`6ed5ee`. `clear`'s callers are `586dd6` (`run_oos_recovery`) and `594faa` (`Game::close`) |
| 18 | `LeaderData::walk_data (006d6750)` writes it on load | yes | "save/load not read; no block copy found" | one-sided | **confirmed (first)**: `walk_data:16` walks `[&who, &field_0x692a)` when `leader_flags & 1`, and on input that block is filled from the file |
| 19 | reader: `Leaders::end_process_all (006ed070)`, `+8`-biased `[0x1f8]`, UI-only | yes | yes | agree | **confirmed** |
| 20 | reader: `Army::find_target (006f69b0)` ×4 when the army's own leader's count ≠ 0, candidate ≠ that leader, and either stance word is 0 | yes | yes | agree | **confirmed**: `6f72a7`..`6f72da` (`0xe3a404(%eax,%edx,4)` = `+0x74 + 4c` of the army's leader, then `leaders[c] +0x74 + 4·who`; `shll $0x2`) |
| 21 | ×4 timing: zero before `Armies::process_all`, so it can fire only through in-frame paths during `Objects::process_all` (`Armies::emergency` from `Object::do_damage`) | yes | yes | agree | **confirmed**: `do_frame:199` clear, `:272` armies, `:275` objects; callers by listing: `emergency` ← `Object::do_damage` `64bc12`; `Army::process` ← `send_navy`, `diplo_change`, `emergency`, `process_all` |
| 22 | `send_navy`'s place | "a caller of `Army::process`" | "called from `find_target` itself" | complementary | **both confirmed**: `6f8687` (`find_target` → `send_navy`), `6f2d56` (`send_navy` → `Army::process`) |
| 23 | the dump prints the whole frame's count | not traced (open) | yes | one-sided | **confirmed (second)**: `GameLog::end_frame` is `do_frame:315`, after `end_process_all` at `:279`; `end_frame:12` runs `full_dump` |
| 24 | crate: clear at `core/frame.rs` ~96 beside `combat.retargets[who] = 0` | yes | yes (notes the crate loops every player, the original gates on bit 2) | agree | **confirmed**: `crates/sim/src/core/frame.rs:98` |
| 25 | crate: carrier +1 at `carrier_queue_tick`, `air/carrier_queue.rs:155` | yes | yes | agree | **confirmed**: `:137` fn, `:155` the capped arm |
| 26 | crate: building +1 is missing; the crate tests the cap only at completion; the increment belongs in the per-visit path (`slot_plan`'s caller / `advance_slot_with`), not on `Advanced::Blocked` | yes | yes | agree | **confirmed**: `cities/queue.rs:1441` `slot_plan`, `:1488` `advance_slot_with`, `:1575` `Handover::Population => Advanced::Blocked`; `economy/cost.rs:1248` `exceeds_population` |
| 27 | crate reader: `ai/army.rs:38` seam; `ai/army/target_score.rs:45` `f7e8`, live impl returns 0 at `:453`, used at `:144` | seam only | all three | agree (second fuller) | **confirmed** |
| 28 | `crates/rondata/src/diff/leader.rs:49` calls it "unread" | noted, not contested | "not right: `find_target` reads it" | disagree | **corrected (second right)**: the reason string is wrong; claim 20 is the reader |
| 29 | checksum layout lists it, `checksum/leaders.rs:148` | yes | yes | agree | **confirmed** |
| 30 | plain game | yes | yes | agree | **confirmed** for the clear, the building arm and the carrier arm (late naval); the ×4 is plain but narrow |

### The rule as a builder should port it

- **State**: `pop_issues: [i32; players]`, a plain count, no scale.
- **Reset**: `= 0` at the head of each leader's frame step, for leaders with
  `leader_flags & 2` only, before strategy, armies and objects
  (`Sim::tick`, `core/frame.rs:98`, gated on the crate's image of bit 2).
  Also 0 at init; the save loader restores the saved value.
- **Step, building** (`Build::do_queue (0061e410)`): when the building has
  a non-empty queue and passes the two early returns (a library not the
  first; a silo holding a missile), and the visited slot's type `t` is a
  trained unit (not build, not `DISBAND`, `is_unit_type`, tech bit set), then
  `pop_cap < control_cost(t) + control` adds +1 **before** progress is
  added. Progress still advances at `accel_train` (clamped to `train_time`).
  **At most +1 per building per frame**: the head slot, or the
  airbase/caravan fallback slot after a head refused by a limit.
  Crate site: `advance_slot_with` (`cities/queue.rs:1488`) for a researched
  unit entry, and the fallback slot the crate already selects at `:1370`ff.
  Order across buildings does not matter for the frame total, only for the
  in-frame ×4 below.
- **Step, carrier** (`Unit::process (00610bc0)`): `num_queued != 0`, owner
  bit 2, `queue_time += accel`, `queue_time >= train_time(t)` and
  `pop_cap < control_cost(t) + control` → +1 and `queue_time -= accel`.
  That is +1 every frame while capped. Crate site: `carrier_queue_tick`'s
  arm at `air/carrier_queue.rs:155`.
- **Readers**: `Army::find_target`'s ×4 (`target_score.rs:144`, wire
  `f7e8` at `:453` to the live count). It matters only for `find_target` runs
  inside `Objects::process_all` (`Armies::emergency` from a damage event),
  where the count is partial and depends on object order. Under
  `Armies::process_all` it reads 0 for every bit-2 leader.
  `end_process_all` is UI only.
- **Plain game**: yes.
- **Confidence**: high on writers, predicates, reset and the
  one-per-building bound (listing-backed). The archive dumps fit
  (0..9, never a growing series).
- **Open**: (a) whether a leader without `leader_flags & 2` runs
  `do_queue`. Its count would never reset, which the 948-dump maximum of 9
  argues against. (b) The ×4's reachability is structural; no capture shows
  it changing a target. (c) `hurry_action` was not re-traced. (d) A diff of
  `pop_issues` against the dump, once the building arm is built, turns all
  of this into an assertion. It is the cheapest next step, since the dump
  key exists (`diff/leader.rs:49`).
