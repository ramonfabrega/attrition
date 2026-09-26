# Golden record, chapter twenty-two — the launch line: a strike from inside
# a base.
#
# docs/GOLDEN.md §31 (item 836; docs/DECISIONS.md 41 §1 and §5, 49). Chapter
# seventeen whole, and one more line: `@strike 0 2006 1 6` on 766 is the
# DLL's call of `CommandManager::issue_flight@00941d40` with ATTACK (10) on
# the Fighter `0/6`, which has stood inside its Airbase `0/2007` since 722
# with its tank refilling. `Object::do_launch@0064f3b0`, the arm of
# `SpecialAnimOrder` no traced game reaches (§30), is what should put it out.
#
#   run265 (item 836):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch22 \
#       --map 14 --end-frame 1500 --log-window 605 1500 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter22.cmd
#
#   Run 2026-09-26, one take: 1,046 s, 365 MB, 458,240 GROUPDATA. The
#   strike on 768, the launch on 778 (the tank's first 0), 36 rounds from
#   924, returning on 1178, inside again on 1385. Falsifier 4 fired on
#   `launch_frames` alone: it stays 0 once the base is empty (do_launch's
#   counter is under `inside_down >= 0`). docs/RUNS.md, run265.
#
# `GUYS=4` with `GROUPS=1`, and `AMMO=5`: a strike is a round (parked 783),
# and the Fighter's rounds on the Barracks are the strike's only mark on it.
# The window is 895 blocks: the first parting is expected on 768 (a value)
# and 778 (a draw), and the chapter's last falsifier is the Fighter's second
# landing, near 1330 by the reading; 1500 leaves 722 blocks past the word.
#
# ---------------------------------------------------------------------------
# THE ISSUER, under the emulator before the capture (item 836, step 1).
#
# On command_oracle.py's fixture, `issue_flight(group, 2006, 1, ATTACK, 0, 0,
# 0)` appends the same 30 bytes (the 5-byte `group`, the 25-byte `flight`
# 0x1c) with the captain on the map (`inside_up` −1), inside `0/2007` at
# `mana_burn` 24, and inside at 0: the issuer reads neither. It writes no
# unit; the reuse is 28 bytes; `use_mp_playback`, `semaphore & 0x10` and
# `semaphore & 4` append nothing. Everything else is at process time:
# `process_flight` -> `Group::action_flight@006fb260`'s inside arm ->
# `add_strafe_order`, and then `do_launch` in the base's `Build::process`.
#
# ---------------------------------------------------------------------------
# THE STAGING, read off run223 (chapter seventeen's capture, which is this
# game to the new line). Every predicate is a printed field, so no staging
# run was taken (parked 821).
#
# - `0/6` is inside `0/2007` from block 722 (`inside_up 2007`), at (11424,
#   14005); the base at (11616, 13920), `inside_down 6`, `launch_frames 15`,
#   `build_masks` 4232 (bit 8: `Build::process` runs `do_launch`).
# - The tank, `mana_burn`: +1 a block on the map from the `add` (1 on 611),
#   112 on 722, −2 a block inside (`AIR_UNIT_MANA_RECHARGE`), 24 on 766, 0
#   first on **778**.
# - The Barracks `1/2006`: `ever_seen` 2 from 618 and **3 from 762**, so
#   who=0 has seen it by 766. Damage from 822; gone on 1080.
# - The reach: `vector_dist(9504, 2592)` = 9857, against `mana` 400 (the
#   FIGHTER row) times `get_speed` 75 = 30000.
#
# ---------------------------------------------------------------------------
# THE READING (docs/GOLDEN.md §31 has it with the citations).
#
# 1. **`action_flight`'s inside arm gives the strike.** A member not on a
#    strafe whose `get_inside` is the base passes `valid_target` (the
#    Barracks' `ever_seen`), the target's `MISSILE_DEFENSE_BONUS` (who=1
#    has none), the reach `dist <= mana * get_speed(x, y, 1)`, then
#    `is_ally || war_allowed`: `add_strafe_order(2006, 1, 2007, 0, 1,
#    QUEUE_NEW, 1)`, a strafe on the target, `returning 0`, home the base.
# 2. **A plane inside runs no `work`** (`Unit::process@00610bc0`'s inside
#    arm), so the strike waits in the base.
# 3. **`do_launch` launches a plane with an order only on a full tank.**
#    `launch_frames` counts to `FRAMES_BETWEEN_LAUNCHES` (15, the PE's
#    `.data`) and saturates; each plane in the chain with an order and
#    `mana_burn == 0` whose current order has the action bit is added to
#    `launching`, and the first is `come_out(0)`, `launch_frames` 0. Units
#    are processed before buildings, so the launch shares the block the
#    tank first reads 0.
# 4. **`come_out`'s tail is `do_spec_anim`'s EXIT at an AIRBASE**: the
#    plane faces 0, is placed at the base's point less 0xc0, (11424,
#    13920), bank and pitch zeroed; the kill drops it from `launching`; then
#    `work`: `do_strafe` flies its first step in the same call. A Fighter
#    draws its `cruising_alt` on `(6 + 778) & 7 == 0`.
# 5. **The tank burns again**: +1 a block from 779, 400 on 1178, when
#    `check_fuel` sets `returning 1`; the flight home and a second
#    `land_plane`.
#
# THE PREMISE'S KILLER is `0/6`'s order stack and `inside_up` on 768–778.
# The writers: `action_flight`'s inside arm (the only adder for a plane
# inside); `do_launch`'s kill of an unflagged order and of a strike with
# an invalid target and point; `come_out`'s `close_orders`, which spares a
# fixed-wing plane. The loops: `action_flight`'s members to `group.num`
# (1); `do_launch`'s chain from `inside_down` to the first −1 (`0/6`
# alone).
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire.
#
# 1. **The issue does not reach the pump.** Trace frame 766: an INFO 17
#    with a refusal, or no `process_flight` on 767 (COMMANDMANAGER).
# 2. **The inside strike is refused.** Block 768: `0/6` with no order, or
#    anything but one `STRAFEORDER` on `1/2006`, `mandatory 1`, flags 4,
#    `AIRORDER oxx 2007 whose 0 cruising_alt 1600 returning 0`, `xx/yy`
#    (21120, 16512); `0/6` still `inside_up 2007`.
# 3. **The tank does not gate the launch.** Splits three readings: `0/6`
#    on the map on 768 (no gate); on 778, the block `mana_burn` first
#    reads 0 (the reading); on 779 (the gate a frame behind).
# 4. **The launch is not the EXIT's placement.** Block 778: `0/6` more
#    than a step and a half (112) from (11424, 13920); `0/2007`'s
#    `launch_frames` not 0 on 778 and 1..15 on 779..793; any
#    `SPECIALANIMORDER` on any block.
# 5. **The strike does not fly at its target.** From 778: `0/6`'s current
#    order not the `STRAFEORDER` on `1/2006` while the Barracks stands and
#    the tank lasts; no round of `0/6`'s on `1/2006` (AMMO) by 1178.
# 6. **The tank is not the reading's.** `0/6`'s `mana_burn` not 1 on 779
#    and 400 on 1178; `returning` not 1 on 1178.
# 7. **It does not come home.** `0/6` not inside `0/2007` again by 1499.
#
# Falsifiers 2 to 4 test the premise's own unit, `0/6`'s stack and
# `inside_up` on its blocks, never a block's first row (711).
#
# A call on trace frame F is on block F+2 (§17).
0 !ai off
600 library who=0 6
606 add airbase who=0 60,72
610 add fighter who=0 60,84
612 add bomber who=0 52,84
614 add bomber who=0 68,84
616 add barracks who=1 110,86
620 @strike 0 2006 1 7 8
640 @flight 0 2007 0 6
660 @flight 0 2007 0 7 8
664 @strike 0 2006 1 7 8
766 @strike 0 2006 1 6
