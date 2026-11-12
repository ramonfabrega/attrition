## 12. `pop_issues` and `misery`

### 12.1 Leaves

- `pop_issues`, +0x7e8, `int`. A plain count with no scale: how many times
  this frame a unit's training met the population cap.
- `misery`, +0x7ec, `int`. A constant 0.

### 12.2 The rule: `pop_issues`

**Reset.** `Leaders::process_all (006ed2a0)` stores 0 at the head of each
leader's step (the inlined head of the dead `Leader::process (006b88b0)`),
for slots with `leader_flags & 2` only. That step runs before strategy,
armies and objects, so the count each frame is that frame's alone. Also 0
from `Leader::init (006e3930)` and the bulk writers of the header; the save
loader restores the saved value.

**Step, building** (`Build::do_queue (0061e410)`, called once per building
per frame, and from the debug-only `Console::hurry_action (007c3810)`):

1. Early returns, before any slot is looked at:
   - the building's queue is empty (`queued`, BuildData +0x82, is 0);
   - it is a library (`LIBRARY` 0x1b3) that is not the leader's first
     library (`Leader::get_first_library (006db6c0)`), and the entry is not
     `DISBAND` (0x29a);
   - it is a missile silo (`MISSILESILO` 0x208) holding a missile
     (`inside_down`, ObjectData +0x28, `>= 0`), the entry's tech bit is set
     and the entry is not `DISBAND`.
2. The visited slot's type `t` counts when it is not a build type
   (`0x19e ≤ t ≤ 0x21e`), not `DISBAND`, `is_unit_type` (vtable +0xc), and
   its tech bit is set in `LeaderData +0x6c18`. Then, if
   `check_population (006e1330)` is true, which is the strict test

       pop_cap < control_cost(t) + control

   the count takes +1. The test runs before the slot's progress is advanced,
   whatever the progress.
3. Progress still advances on that visit at `accel_train`, clamped to
   `train_time`. The full clock is then refused by `Build::finished`, which
   runs the same population test first and returns 0.

**At most +1 per building per frame.** Only the first library visits more
than one slot (the slot+1 recursion is gated on `LIBRARY`); its extra slots
carry techs, which fail `is_unit_type`. Any other building visits a second
slot only from slot 0, after `Build::finished` refused the head:

- `finished` returned 0 (the population test, or anything else returning
  0): the fallback is `get_next_non_unit (00630b10)`, which returns only a
  non-unit or unresearched slot, so it cannot count;
- `finished` returned negative (the caravan or aircraft limit, tested after
  the population test), the building is an airbase (`AIRBASE` 0x1bf, for
  `get_next_helicopter`) or not (`get_next_non_caravan`), and
  `queued >= 2`: that fallback slot can count.

So a counted head never leads to a second count. The order buildings are
visited in does not change the frame total, only the partial value seen by
the in-frame reader below.

**Step, carrier** (`Unit::process (00610bc0)`): a carrier with
`num_queued` (short +0xa0) ≠ 0, owner `leader_flags & 2`, and
`t = current_upgrade(FIGHTERBOMBER 0x134)`. Each frame
`queue_time += accel_train` (times `ai_speed` when that exceeds 1); if
`queue_time >= train_time(t)` and `pop_cap < control_cost(t) + control`
(strict: equal launches), the count takes +1 and `queue_time -= accel`. The
clock sits at its last value, at or past `train_time`, so the next frame's
add crosses again: **+1 every frame for as long as the cap holds**, per
capped carrier.

**Readers.**

- `Army::find_target (006f69b0)` multiplies a candidate's score ×4 when the
  army's own leader's `pop_issues` ≠ 0, the candidate is another leader,
  and either stance word between them is 0. Since the reset precedes
  `Armies::process_all`, every bit-2 leader reads 0 there. The ×4 can fire
  only from `find_target` runs inside `Objects::process_all`:
  `Armies::emergency` from `Object::do_damage (0064a480)`, and through it
  `Army::process` and `send_navy` (which `find_target` itself calls). There
  the count is partial and depends on object order.
- `Leaders::end_process_all (006ed070)`: UI only.
- The dump: `GameLog::end_frame` runs after `end_process_all`, so it prints
  the whole frame's count.

The capped visit's other effects (flag `0x800`, `pop_cap_frame`, feedback,
`S_POP_LIMIT`, the `(o + frame) % 220` bubble) are console only.

### 12.3 The rule: `misery`

Every store writes 0, and nothing that decides anything reads it.

- **Writers**, all `= 0`: `Leader::calc_pop_cap (006dc490)`'s first store,
  before every branch (17 call sites); `Leader::gain_tech (006dcb60)`'s
  epoch arm (`is_tech_type`, `0x220 ≤ t < 0x275`, and `is_epoch_type`,
  `0x227 ≤ t < 0x243`; not gated on a first gain, which is moot); `Leader::init
  (006e3930)`; `Leaders::clear (006ed5d0)`; the save loader, restoring the
  saved 0. `Leader::calc_misery (006cdff0)` is `misery = 0` and has no
  caller.
- **Readers**: `LeaderData::log_data (006e5110)`, the save and the checksum.

A port holds 0 for every leader on every frame.

### 12.4 In the crate

- **No `pop_issues` storage exists.** The reset belongs beside
  `self.combat.retargets[who] = 0` in `Sim::tick`
  (`crates/sim/src/core/frame.rs:98`). Trap: that loop runs every player,
  while the original resets only leaders with `leader_flags & 2`; gate the
  new store on the crate's image of that bit.
- **Carrier arm**: `carrier_queue_tick` (`crates/sim/src/air/carrier_queue.rs:137`).
  The capped arm at `:155` already takes back the accel and returns; the +1
  goes there.
- **Building arm: missing.** The crate tests the cap only at completion
  (`economy/cost.rs:1248` `exceeds_population`, reached through
  `Handover::Population => Advanced::Blocked` at `cities/queue.rs:1575`).
  The +1 belongs on the per-visit path, `advance_slot_with`
  (`cities/queue.rs:1488`), for a researched unit entry, and on the fallback
  slot the crate already selects at `:1370`ff, **not** on
  `Advanced::Blocked`. The silo hold is already in `slot_plan`
  (`cities/queue.rs:1441`).
- **Reader: a seam.** `ai/army/target_score.rs:45` declares `f7e8`, used at
  `:144`; the live implementation returns 0 at `:453` (and `ai/army.rs:38`
  lists `pop_issues` among the fields whose multipliers are ×1). Wire it to
  the live count.
- **Mislabel**: `crates/rondata/src/diff/leader.rs:49` calls `pop_issues`
  "unread"; `find_target` reads it. `diff/leader.rs:44` calling `misery`
  unread is right.
- **Checksum**: both are in the layout (`checksum/leaders.rs:148`, `:149`)
  and both are `(unheld)` in `LEADERS_HELD_OUT` with count 2
  (`crates/rondata/src/diff/checksum_lines.rs`). Holding `misery` as 0
  removes its entry at no risk; `pop_issues` leaves the list once both
  arms are built.
- **`misery` needs no code** beyond emitting `Some(0)` and turning
  `diff/leader.rs:44` from an unmodelled key into a compared one.
  `crates/sim/src/economy.rs:305` (which cites `calc_pop_cap`'s tail) is a
  pointer, not a store site.

### 12.5 Reached by a plain game

`pop_issues`: yes. The reset every frame; the building arm whenever a
trained unit is queued at the cap; the carrier arm in a late naval game.
The ×4 reader is plain but narrow (a damage-driven emergency retarget while
the attacker's leader is capped). `hurry_action` is debug only.

`misery`: yes, trivially; its writers run, and write 0.

### 12.6 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/misery-*.md`); no diff backs it yet.

- `pop_issues`: 25 confirmed, 4 corrected, 1 open. High confidence on the
  writers, predicates, reset and the one-per-building bound; every one is
  listing-backed. The corrections that changed a rule: the second early
  return is the occupied silo, not an already-researched item; the slot+1
  recursion is library-only, not unconditional; a building adds at most +1
  per frame, not several; `diff/leader.rs:49`'s "unread" is wrong. The
  carrier arm repeats every frame while capped (the clock is pinned), not
  only at the frame the clock first fills.
- `misery`: 13 confirmed, 1 corrected (17 call sites of `calc_pop_cap`, by
  an exhaustive `call`/`jmp` count). High confidence.

**Witnesses, not pinned diffs.** Across the 948 kept `gamelog.txt` under
`~/ron-golden`, `pop_issues` never exceeds 9 and never shows a growing
series, which fits the per-frame reset and the one-per-building bound; and
no dump prints a nonzero `misery`.

### 12.7 Not established

- **Whether a leader without `leader_flags & 2` runs `do_queue`.**
  `do_queue` tests no leader flag. Such a leader's count is never reset, so
  it would only grow; the archive's maximum of 9 argues against it. A dump
  of a defeated slot settles it.
- **The ×4's reachability is structural.** No capture shows it changing a
  target; a capture with an emergency retarget by a capped leader would.
- **`Console::hurry_action`'s call to `do_queue(0)`** rests on one reading's
  listing citation, not re-traced. It is outside the order stream.
- **No diff yet.** A `pop_issues` diff against the dump, once the building
  arm is built, turns this section into an assertion; the dump key exists
  (`diff/leader.rs:49`). A constant-0 `misery` diff is cheaper still.
- That `misery` was a live mechanic emptied before ship is a guess from its
  shape; nothing in the sim depends on it.
