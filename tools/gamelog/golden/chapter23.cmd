# Golden record, chapter twenty-three — the repeat line: an Airbase's repeat
# toggled off between two landings.
#
# docs/GOLDEN.md §32 (item 867; docs/DECISIONS.md 41 §1 and §5, 49).
# Chapter twenty-two whole, and one more line: `@buildmask 0 128 2007` on
# 1440 is the DLL's call of `CommandManager::issue_buildmask@00941f80` with
# 0x80 on the Airbase `0/2007` alone, the repeat button
# (`Options::set_air_repeat@0071c740`). By then the Fighter `0/6` has been
# inside since 1385 with its patrol kept (flags 0) and its tank refilling,
# and the Bombers `0/7`, `0/8` are flying home under theirs.
#
#   run281 (item 867):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch23 \
#       --map 14 --end-frame 1840 --log-window 605 1840 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter23.cmd
#
# The window is 1,235 blocks. The toggle is on 1442 and the last falsifier
# is `0/6`'s full tank on 1585 (286 on 1442, -2 a block): 1840 leaves 255
# blocks past it, and takes `0/7`'s full tank (~1789) in as well.
#
# ---------------------------------------------------------------------------
# THE ISSUER, under the emulator before the capture (item 867, step 1).
#
# On command_oracle.py's fixture with a building (`Build::vftable`) in
# who=0's registry: `issue_buildmask(group [b], 0x80, set)` appends the
# 5-byte `group` and a 9-byte `buildmask` (0x21, `[mask i32][set i32]`), 14
# bytes; the reuse is 12. **The wire's `set` is 1 whatever the third
# argument** (the listing never reads `[ebp+0x10]`). A unit and a building
# in one selection both go into the `group`. It writes no object.
# `Group::action_buildmask@006fc9a0` on a group with `buildings` (+0x49) set,
# `can_carry(AIR)` answering 1: 4232 -> 4104 and 4104 -> 4232 **with `set`
# 1 or 0** (the listing never reads `[ebp+0xc]`): it toggles the bit off the
# first member's state. It writes nothing with `buildings` 0, with
# `can_carry(AIR)` 0, or for the mask 0x08. What the emulator could not
# reach: `process_group`'s `Group::add` and `push_group` of a building
# group, and `process_buildmask`'s logging, at process time.
#
# ---------------------------------------------------------------------------
# THE STAGING, read off run265 (chapter twenty-two's capture, which is this
# game to 1500). Every predicate is a printed field, so no staging run was
# taken and run282 was not used (parked 821).
#
# - `0/2007`: `build_masks` 4232 (0x1088) on every block of run265.
# - `0/6`: inside `0/2007` from 1385, `mana_burn` 400 there, 286 on 1442,
#   one `AIRPATROLORDER`, flags 0, `oxx` 2007, `cruising_alt` 1400,
#   `returning` 0: land_plane kept it under the repeating base (ORDERS §40).
# - `0/7`, `0/8`: on the map on 1442, each an `AIRPATROLORDER` home with
#   `returning 1`, `mana_burn` 600 (the Bomber's cap). `0/7` is inside on
#   1489 in run265; `0/8` is not inside by 1499.
#
# ---------------------------------------------------------------------------
# THE READING (docs/GOLDEN.md §32 has it with the citations).
#
# 1. **The toggle.** `process_group` builds a group of the one building
#    (`Group::add` sets `buildings` from vslot 0x1c) and pushes it;
#    `process_buildmask` calls `action_buildmask(0x80, 1)`: `valid_buildmask`
#    admits the Airbase (`can_carry(AIR)`), its bit is set, so it is cleared:
#    4232 -> 4104.
# 2. **A plane that lands off a non-repeating base loses its orders there**:
#    `Unit::land_plane@005e9950`'s tail (`005e9a43`), vslot 0xf0 clear:
#    `unit_masks &= ~0x4000000`, the path emptied, `close_orders`. `0/7` on
#    1489 and `0/8` on its landing come in with no order.
# 3. **A plane already inside with an unflagged order is killed at its full
#    tank**: `Object::do_launch@0064f3b0` launches a plane with an order at
#    `mana_burn` 0 only if `has_repeat_air() || flags & 4`, else
#    `kill_current_order`. `0/6`'s patrol goes on 1585, and it stays inside.
# 4. **Nothing launches**: `0/7` at its full tank has no order to fly.
#
# THE PREMISE'S KILLER is `0/7`'s stack on its landing block and `0/6`'s on
# 1585. The writers: `land_plane`'s clear arm and its keep (`flags &= ~4`),
# `do_launch`'s kill and its launch (`come_out`), and no command after 1440.
# The writers of `build_masks & 0x80` (by `+0x60` in every spelling):
# `Build::init` (|= 0x88), `action_buildmask`, `ScenarioFuncSet::
# repeat_orders_enable`/`_disable`, `UnitBalance::next`, the zeroing
# constructors and `Wall::init`, `Wall::swap_team`'s copy; `Wall::process`'s
# whole-word store keeps the bit. Only `action_buildmask` runs here. The
# loops: `action_buildmask`'s members to `group.num` (1); `do_launch`'s
# chain from `inside_down` to the first -1.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire.
#
# 1. **The issue does not reach the pump.** Trace frame 1440: an INFO 17
#    with a refusal, or no `process_buildmask unitmask: 128` on 1441
#    (COMMANDMANAGER).
# 2. **The toggle is not the reading's.** `0/2007`'s `build_masks` on 1442:
#    4232 (no toggle: `set` read as a value, or the base refused), 4104 (the
#    reading), anything else (another bit); and not 4104 to the end.
# 3. **A landing off a non-repeating base keeps its order.** `0/7` inside
#    on 1489: no order (the reading), its `AIRPATROLORDER` kept (flags 0)
#    and killed at its full tank (the booking's reading), or kept and
#    relaunched then (the bit is no gate). The same for `0/8` at its
#    landing.
# 4. **The waiting patrol is not killed at its tank.** `0/6`'s patrol, flags
#    0: gone on 1442 (the toggle kills it), kept to 1584 and gone on 1585
#    with `0/6` still `inside_up 2007` (the reading), launched on 1585
#    (`inside_up` -1, on the EXIT's point), or kept past 1585.
# 5. **Something launches.** Any aircraft out of `0/2007` after 1442, any
#    `SPECIALANIMORDER` on any block.
#
# Falsifiers 3 and 4 test the premise's own unit, each plane's stack on
# its own block (711), and each splits the readings (parked 789).
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
1440 @buildmask 0 128 2007
