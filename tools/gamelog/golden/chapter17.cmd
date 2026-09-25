# Golden record, chapter seventeen — the flight line: an issuer the AI
# rarely takes.
#
# docs/GOLDEN.md §25 (item 746; docs/DECISIONS.md 41 §1 and §5, 49). Chapter
# nine's harness, two more verbs: `@flight` and `@strike` are calls, from
# rontrace.dll, of the original's own `CommandManager::issue_flight@00941d40`
# with `orders` MOVE_TO (1) and ATTACK (10). `Console::execute_at_cursor@
# 007c6630:2849` passes MOVE_TO through `GroupOut::issue_flight@00708b10` for
# aircraft right-clicked on their own base, and `:2835` ATTACK for aircraft
# right-clicked on an enemy, with `ctrl` and `alt` 0; the DLL passes `shift`
# 0 too.
#
#   run223 (item 746):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch17 \
#       --map 14 --end-frame 1400 --log-window 605 1400 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter17.cmd
#
# `GUYS=4` with `GROUPS=1` and neither `DEATHS` nor `AMMO`: run215's and
# run219's levels, the line whose pool printed (parked 733). The window runs
# to 1400 because the pair's tanks run dry near 1212 and the chapter's last
# falsifier is their landing; the first parting is expected near 642, so
# the runway past it is far more than 250 blocks.
#
# ---------------------------------------------------------------------------
# THE ISSUER, as the emulator ran it before the pair (item 746, step 1).
#
# Under unicorn on command_oracle.py's fixture, widened to who=0's objects 6
# and 7 as live captains, `issue_flight(group, ox, whom, orders, shift, 0,
# 0)` appends a 25-byte `flight` (type 0x1c) behind the group: `[ox][whom]
# [shift][ctrl][alt][orders]`, each as passed, and a `group` of one (5
# bytes) or two (7) for a fresh selection, the 3-byte reuse for a repeated
# one. MOVE_TO and ATTACK differ in the `orders` word alone; the issuer tests
# neither the target nor the aircraft. A non-captain is dropped;
# `use_mp_playback`, `semaphore & 0x10` and `semaphore & 4` each append
# nothing. It writes the package and the selection caches and nothing else:
# **no order, no unit, no draw.** What makes a class is at process time:
# `CommandPackage::process_flight@00947db0` -> `Group::action_flight@
# 006fb260` -> `Unit::add_strafe_order@005e48c0`.
#
# ---------------------------------------------------------------------------
# THE READING (docs/GOLDEN.md §25; docs/ORDERS.md §1.1, §1.2, §11.1).
#
# 1. **The class is `StrafeOrder` (type 16), never a bare `AirOrder`.**
#    `AirOrder` has no `OrderIndex` of its own: it is a base of
#    `StrafeOrder : AttackOrder, AirOrder` (0x54), `AirPatrolOrder` and
#    `AirAttackGroundOrder`, and the pool has no case for it. The flight's
#    one constructor is `add_strafe_order`'s `get_obj(STRAFE)`;
#    `StrafeOrder::log_data@0047fd80` prints `STRAFEORDER`, then
#    `AttackOrder::log_data` and `AirOrder::log_data@0047fa40` (`AIRORDER`,
#    its `UNITORDER`, `cruising_alt`, `sharp_turn`, …), then `xx yy`.
#    `add_strafe_order(ox, whom, home_o, home_who, mandatory, queue,
#    action)` writes target `ox/whom/uid` (+8/+0xc/+0x10), `mandatory`
#    (+0x1c), the `AirOrder`'s home `oxx/whose` (+0x28/+0x2c),
#    `cruising_alt` 0x640 (+0x30), `returning` (+0x3c) = 1 and `xx/yy`
#    (+0x40/+0x44) = −1 for no target, else 0 and the target's point, and
#    the action bit.
# 2. **`action_flight`'s gates, per member** (`local_2c < group.num`: 1 for
#    the Fighter, 2 for the pair). The target must be live (`+8 & 1`); for
#    MOVE_TO it must be the group's own and `can_carry(AIR)`, and each
#    member needs `can_carry(base, o, who)` and not `is(FIGHTERBOMBER 0x134)`
#    unless the base is its `home_base`. Then:
#    - **on a `STRAFE` already** (`UnitData::order_type`): the order is
#      re-pointed, no new one. ATTACK writes the target's `ox/whom/uid` and
#      point and, with fuel left (`mana_left`), `returning 0`, `mandatory 1`
#      and the action bit;
#    - **otherwise**, the "inside" is the air order's home for
#      `AIR_PATROL`/`AIR_ATTACK_GROUND` (`is_air@0046f000`: 16, 17, 24),
#      else `ObjectData::get_inside@00651a80` — `inside_up`, −1 for an
#      aircraft `add` placed. **An ATTACK needs that inside ≥ 0**; MOVE_TO
#      does not. MOVE_TO then gives `add_strafe_order(−1, −1, base, who, 1,
#      QUEUE_NEW, 1)` unless the member is already inside that base; ATTACK
#      asks `valid_target`, `MISSILE_DEFENSE_BONUS`, the tank's reach
#      (`dist <= mana * vslot 0x17c`) and war, then `add_strafe_order(ox,
#      whom, inside, inside_who, 1, QUEUE_NEW, 1)`.
#    So **an unbased aircraft on the ground takes no strike at all**, with
#    no feedback: the member is skipped.
# 3. **The flight.** `Unit::do_strafe@005eab00` flies the order through
#    `Unit::do_air_physics@005e86d0`. On every eighth frame (`(o + frame) &
#    7 == 0`) a **non-bomber** redraws its `cruising_alt` as `(r % 7 + 13)
#    * 100` from `Random::get(0, 0xffff)`; a Bomber (`is(BOMBER 0x130)`)
#    holds 0x640 with no draw. With `returning` set, `Unit::check_fuel@
#    005e9be0` aims at the home base's point less 0xc0 in x, and when the
#    Manhattan distance falls under 1.5 steps `Unit::land_plane@005e9950`
#    clears a mandatory target-less strafe (`clear_orders`) and adds a
#    `SpecialAnimOrder` (type 25) on the base; the landing ends in
#    `go_inside`. A target-less strafe never re-targets.
# 4. **The tank.** `Unit::process@00610bc0` adds one to `mana_burn` a frame
#    while `inside_up < 0` and refills `AIR_UNIT_MANA_RECHARGE` (2) a frame
#    inside. `check_fuel` sets `returning` when `mana() − mana_burn` is 0:
#    the Fighter (400) born on 610 would run dry on 1010 unbased, and each
#    Bomber (600) born on 612 and 614 runs dry on **1212** and **1214**
#    (`BOMBING_MANA_COST` is 0). Past that it flies home and lands.
# 5. **The base does not launch an idle plane.** `Object::do_launch@
#    0064f3b0` walks the base's `inside_down` chain and launches only a
#    plane with an order (`+0xd8 != 0`) and a full tank (`mana_burn` 0),
#    one per `FRAMES_BETWEEN_LAUNCHES`. A landed plane's stack is empty.
#    `MAX_AIRCRAFT_PER_AIRBASE` is 10; three fit.
#
# ---------------------------------------------------------------------------
# THE CAST, on chapter nine's open ground north of Napata (run215's start
# `WORLD`: cells x 11–19, y 14–23 are BASELAND, region 1, owner −1; the
# sandy lakes are x 7–11 and x 19–35). No unit of either side is within
# twelve tiles of the pad (the human's idle search, `UNIT_RESPOND_RANGE ×
# 0xc0`).
#
# - who=0's Airbase at tile (60, 72), cell (15, 18): **`0/2007`**, the id
#   run175 gave who=0's first staged building (and run208 its Barracks).
# - a Fighter `0/6` on the pad at tile (60, 84), (11616, 16224);
# - a Bomber pair, `0/7` at tile (52, 84) and `0/8` at (68, 84);
# - who=1's Barracks at tile (110, 86), cell (27, 21): **`1/2006`**, the id
#   run175 gave who=1's first staged building. It is ~42 tiles from the pad
#   and ~52 from the base.
#
# A call on trace frame F is processed between F+1 and F+2 and is on block
# F+2 (§17).
#
# ---------------------------------------------------------------------------
# THE PREMISE'S KILLER, and its writers (§3, point 5; parked 667).
#
# The killer is **a unit's order stack on the block after each call**:
# anything but one `STRAFEORDER` (type 16) where one is predicted, or any
# order where none is. The class's writer on this path is
# `add_strafe_order`'s `get_obj(STRAFE)`; its target and `returning` are
# rewritten by `action_flight`'s STRAFE arm, by `do_strafe` (a dead target
# with a valid point becomes an `AirPatrolOrder` over it; a live one is
# re-targeted `QUEUE_FIRST` every sixteenth frame by `find_new_bomber_
# target`), by `check_fuel` (`returning`) and by `land_plane`. The one
# other way to a strike, `add_air_attack_ground_order`, needs the type's
# `0x8000000` (a missile) and is not reached. The loops the premise rests
# on: `action_flight`'s member loop to `group.num`; `check_fuel`'s base
# search over `objects` 2000..`[who]+0x184` and 0..`[who]+0x15c`;
# `do_launch`'s chain to the first −1.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire.
#
# 1. **The issue does not reach the pump.** Trace frames 620, 640, 660 and
#    664: an INFO 17 with a refusal, or no processed `flight` on the next
#    frame (COMMANDMANAGER).
# 2. **An unbased aircraft takes a strike.** Block 622: `0/7` or `0/8` with
#    any order. The DLL's call is accepted; the refusal is `action_flight`'s.
# 3. **The class is not a strafe, or not the home's.** Block 642: `0/6`
#    without exactly one `STRAFEORDER` (type 16), `ox −1`, `whom −1`,
#    `mandatory 1`, the action bit, and an `AIRORDER` naming `0/2007` with
#    `returning 1`. Block 662: the same for `0/7` and `0/8`.
# 4. **The strike is a new order, or not the target's.** Block 666: `0/7`
#    or `0/8` with more than one order, or a strafe whose target is not
#    `1/2006`, or `returning` not 0, or home not `0/2007`.
# 5. **The aircraft does not fly.** From 642 (`0/6`) and 662 (the pair): a
#    position that never leaves its pad, or an air altitude that stays 0.
# 6. **The landing is not the base's.** No `SPECIALANIMORDER` (type 25) on
#    `0/2007` before `inside_up` reads 2007; `0/6` not inside by block 760.
# 7. **The strike does not reach the point.** No damage on `1/2006` by
#    block 900, or a bomber that never comes within its range of it.
# 8. **They do not come home.** `returning` not set on the pair by 1230,
#    or `0/7` and `0/8` not inside `0/2007` by 1400; and three planes
#    inside the base (`inside_up` 2007 on `0/6`, `0/7`, `0/8`) at the end.
#
# Falsifiers 2 to 4 test the premise's own unit, the order stack of each
# commanded aircraft on its processed block, never the first row of the
# block (711).
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
