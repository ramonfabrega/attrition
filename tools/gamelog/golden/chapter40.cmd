# Golden record, chapter forty — the casts' other arms: a Citizen pressed
# before the Militia is held and one pressed on who=1's land, the second
# wounded there and converted both ways; a Militia's Civilian ahead of a
# repair, a build and a gather; and the City's alarm rung and cleared over
# a garrisoned Militia.
#
# docs/GOLDEN.md §49 (item 1167; docs/DECISIONS.md 41 §1 and §5, 49).
# Chapter thirty-nine's two killers no pin held (§48's table): the
# conversions' damage carried whole (run422 wounds no converting unit), and
# To Arms without the Militia test (run422 holds the Militia). And the
# unstaged issuers of the same casts: `Group::action_alarm@0070ec30`'s
# all-clear, and `add_cast_order(… 0x293 …)` in `Group::action_swarm_around
# @0070fbe0` (REPAIR and BUILD_AT) and `Group::action_gather@00700b90`.
# (`Group::action_repair@007020c0`'s own cast arm filters its members to
# Citizens, who cannot cast Civilian: unreachable.)
#
#     0 `!ai off`                                  every chapter's
#   600 `library who=0 1`                          the Classical Age:
#                                                  `has_preq(MILITIA)`
#   602 `peace 1`                                  who=0 and who=1 at peace:
#                                                  `PEACE_ATTRITION` 8 on
#                                                  who=0's units on who=1's
#                                                  land, and no fire
#   604 `add citizen who=0 30,150`                 A 0/6, at home
#   606 `@spell 0 660 -1 -1 0 0 6`                 To Arms on A: refused,
#                                                  the Militia not held
#   608 `tech who=0 militia on`                    the Militia's own bit
#   610 `add citizen who=0 186,106`                D 0/7, cell (46, 26):
#                                                  who=1's
#   612 `@spell 0 660 -1 -1 0 0 7`                 To Arms on D: refused,
#                                                  who=1's land
#   614 `add militia who=0 20,152`                 M1 0/8, beside 2006
#   616 `add militia who=0 34,176`                 M2 0/9, beside the site
#   618 `add militia who=0 26,146`                 M3 0/10, by the
#                                                  Woodcutter's Camp 2001
#   620 `add militia who=0 20,160`                 M4 0/11, by the City 2000
#   630 `@repair 0 2006 0 8`                       M1: the approach, the
#                                                  Civilian, the repair
#   640 `@build 0 7296 34176 427 9`                M2: a Barracks site (120
#                                                  timber; who=0 holds 242 on
#                                                  640 in run422), the
#                                                  approach, the Civilian,
#                                                  the build
#   650 `@gather 0 2001 10`                        M3: the gather at
#                                                  QUEUE_NEW, the Civilian
#                                                  in front
#   660 `@garrison 0 2000 0 11`                    M4 into the City
#   700 `@move 0 33408 20448 7`                    D, bled, walks to cell
#                                                  (43, 26): no one's
#   800 `@spell 0 660 -1 -1 0 0 7`                 To Arms on D, wounded
#   840 `@spell 0 659 -1 -1 0 0 7`                 Civilian on D, wounded
#   880 `@alarm 0 2000`                            the bell: the City's
#                                                  `city_flags` 0x4811 has
#                                                  bit 0 and not 0x40
#   900 `@alarm 0 2000`                            the all-clear: M4 inside,
#                                                  `cast_civilian`, eject
#
#   run430 (item 1167):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch40 \
#       --map 14 --end-frame 1150 --log-window 605 1150 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter40.cmd
#   (cover=0 on the click-free lane, a DLL built from this tree: `@alarm`
#   and `@gather` are its; run431 is the same script at cover=1 on the
#   queue lane.)
#
# The window is 545 blocks: the last event the script schedules is the
# all-clear on 900 and M4's eject after it; 1150 leaves 250 blocks past it.
#
# ---------------------------------------------------------------------------
# THE STAGING, walked before the capture (item 1167).
# - Off run422's dump (the golden start is every chapter's): the City 2000's
#   `city_flags` 18449 = 0x4811 on frame 0, bit 0 set and 0x40 clear, so the
#   first press rings; who=0's timber 242 on 640; who=1's land at cell
#   (45..46, 26), no one's at (38..44, 26), no feature on either.
# - Off run422: a Militia may take Civilian (C on 660), and `is_castable`'s
#   0x293 has no case of its own — castable wherever the caster stands.
# - Off this crate's walk: D bleeds 1 hit every 8 frames from ~632 (a lone
#   figure at peace) and stands at 34/50 when it leaves; To Arms on 800
#   makes it a Militia at 35/50 (frac 81), Civilian on 840 a Citizen at
#   36/50 (frac 76). A carried-whole conversion leaves 34 both times.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire (§49 has the readings).
# A call on trace frame F is on block F+2 (§17).
#
# 1. Every issue reaches the pump: 607, 613, 631, 641, 651, 661, 701, 801,
#    841, 881, 901.
# 2. The refusals: block 608, A holds no CASTORDER and is a Citizen; block
#    614, D likewise.
# 3. D bleeds: `attrition` 8 and `damage` rising on who=1's land from ~632.
# 4. D's To Arms (~806): a Militia whose damage is new_hits × ((damage << 8)
#    / hits) / 256, not its damage carried; its Civilian (~846) likewise.
# 5. M1 (block 632): a MOVEORDER, then a CASTORDER (659, ox −1, its own
#    point), then a REPAIRORDER on 2006; a Citizen once it has arrived.
# 6. M2 (block 642): 0/2007 a Barracks site; M2's MOVEORDER, CASTORDER,
#    BUILDORDER; a Citizen, then building.
# 7. M3 (block 652): a CASTORDER in front of a GATHERORDER on 2001; a
#    Citizen by ~657.
# 8. M4 inside the City by ~700.
# 9. The bell (block 882): the City's `city_flags` | 0x40; the Citizens in
#    its radius under GARRISONORDERs.
# 10. The all-clear (block 902): `city_flags` & ~0x40; the walking
#    Citizens' orders killed; M4 a Citizen with `rare` 50; ejected.

0 !ai off
600 library who=0 1
602 peace 1
604 add citizen who=0 30,150
606 @spell 0 660 -1 -1 0 0 6
608 tech who=0 militia on
610 add citizen who=0 186,106
612 @spell 0 660 -1 -1 0 0 7
614 add militia who=0 20,152
616 add militia who=0 34,176
618 add militia who=0 26,146
620 add militia who=0 20,160
630 @repair 0 2006 0 8
640 @build 0 7296 34176 427 9
650 @gather 0 2001 10
660 @garrison 0 2000 0 11
700 @move 0 33408 20448 7
800 @spell 0 660 -1 -1 0 0 7
840 @spell 0 659 -1 -1 0 0 7
880 @alarm 0 2000
900 @alarm 0 2000
