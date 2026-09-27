# Golden record, chapter twenty-nine — the repeat launch: a landed patrol
# relaunched under the repeat bit, and an aircraft trained at the Airbase.
#
# docs/GOLDEN.md §38 (item 915; docs/DECISIONS.md 41 §1 and §5, 49).
# Chapter twenty-two whole — no repeat toggle, so the Airbase keeps
# `Build::init`'s bit — and one more line: `@queueup 0 287 1 2007` on 1540
# is the DLL's call of `CommandManager::issue_queue_up@00941be0` for one
# Biplane at the Airbase `0/2007`. By then `0/6` has been inside since 1385
# with its kept patrol (flags 0) and `0/7`, `0/8` land on 1489 and 1513 with
# theirs; each tank first reads 0 on 1585, 1789 and 1813.
#
#   run308 (item 915):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch29 \
#       --map 14 --end-frame 2070 --log-window 605 2070 --timeout 5400 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter29.cmd
#
# The window is 1,465 blocks: the last relaunch is `0/8`'s on 1813, and 2070
# leaves 257 blocks past it, with `0/6`'s empty tank (1985) inside.
#
# ---------------------------------------------------------------------------
# UNDER THE EMULATOR (item 915, step 1; the tables are in §38).
#
# `Object::do_launch@0064f3b0` on a synthesized base: under 4232 a full,
# unflagged patrol is launched (`returning` 0, `launching`, `come_out(0)`,
# the counter 0); under 4104 it is killed. A plane with no order is passed
# over. **The walk ends at a launch**: the next link is the launched
# plane's own `inside_down` (`64f806`), which `remove_from_inside` sets -1.
# `Build::train@0062f9b0` at a `CARRY_AIR` trainer (`62fac0`) with no gather
# point calls no `come_out`: the plane stays inside with no order.
#
# ---------------------------------------------------------------------------
# THE STAGING, walked by this crate through `input::group_queue_up` on
# run281's start (the same game to 1440); run309 was not used.
#
# - 1542: the Biplane queued, 85 metal and 85 oil (134 -> 49, 100 -> 15).
#   The Fighter (289) is `CantTrain`: `library who=0 6` stops at the
#   Industrial Age, the Biplane's.
# - 1585: `0/6` relaunched, its patrol kept with flags 0.
# - 1746: the Biplane `0/9` born. This crate brings it out on the EXIT.
# - 1789: `0/7` relaunched; 1813: `0/8`.
# - 1985: `0/6`'s tank empty (`mana_burn` 400).
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire (§38 has the readings).
#
# 1. The issue does not reach the pump: trace frame 1540, an INFO 17 with a
#    refusal, or no `process_queue_up` on 1541 (COMMANDMANAGER).
# 2. The press (1542): `0/2007` one Biplane queued, metal and oil -85 each.
# 3. The first relaunch (1585): `0/6` out on the EXIT with its patrol,
#    flags 0 (the reading); inside with no order (killed); inside with the
#    patrol (kept).
# 4. The trained aircraft (1746): `inside_up 2007`, no order, `mana_burn` 0
#    (the reading); out on (11424, 13920) (the EXIT, parked 843); out on the
#    ring (the ordinary exit). Never launched to 2069.
# 5. The second and third relaunches (1789, 1813), the Biplane behind each.
# 6. The counter with the Biplane inside (1814..1829): 1..15, then 15; not
#    0 (an empty base).
# 7. The relaunched patrol's tank (1985): `mana_burn` 400, `returning` 1.
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
1540 @queueup 0 287 1 2007
