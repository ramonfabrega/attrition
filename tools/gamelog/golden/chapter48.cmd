# Golden record, chapter forty-eight — three arms the unit tests or a
# widening hold and no walk (DECISIONS 56 §3), item 1350's M2–M4, all on one
# site: a who=1 Tower site under construction that stands on who=0's land
# (building attrition, `docs/CITIES.md` §9.5), is struck by a who=0 squad
# (the carry in the progress a blow takes) and bombed (an aircraft's blow
# takes none).
#
# docs/GOLDEN.md §57 (item 1358). A cast of its own on the golden start:
#
#     0 `!ai off`                                   every chapter's
#   600 `library who=0 6`                           the bomber's age
#                                                   (chapter thirty-eight's)
#   600 `library who=1 1`                           the Art of War: who=1 may
#                                                   lay a Tower
#   601 `resource who=1 all +2000`
#   602 `add 3 citizen who=1 218,120`               A 1/6, 1/7, 1/8
#   604 `be 1`, 606 `@build 1 42624 23424 439 6 7 8`, 608 `be 0`
#                                                   T 1/2006, a Tower site at
#                                                   cell (55, 30), who=1's
#                                                   southern edge
#   610 `add airbase who=0 200,150`                 0/2007
#   612 `add bomber who=0 196,158`                  0/6
#   640 `@flight 0 2007 0 6`                        the bomber home
#   650 `add fort who=0 222,178`                    F 0/2008 at cell (55, 44):
#                                                   its claim, at who=0's
#                                                   Civic 6, takes T's ground
#   860 `add scout who=0 226,128`                   S 0/7: who=0's eyes on T
#   900 `add elite_pikemen who=0 224,127`           P 0/8, 0/9, 0/10
#   902 `@attack 0 2006 1 8`                        P on T, builders working
#   950 `@strike 0 2006 1 6`                        the bomber on T
#  1040 `@move 0 42720 30000 8`                     P walked off
#
#   run550 (item 1358), the take:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch48 \
#       --map 14 --end-frame 1750 --log-window 605 1750 --timeout 5400 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=5,GROUPS=1,AMMO=5,DEATHS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter48.cmd
#
# T dies on 1482 in the walk, and 1750 leaves 268 blocks past it.
#
# ---------------------------------------------------------------------------
# THE READINGS (docs/GOLDEN.md §57 has them whole).
#
# - `Object::take_damage@00652020`: a site loses `lost × 50` of its
#   `job_counter`, `lost` the blow's whole hits with its carry
#   (`0065230b`, `0065239b`); an attacker in use whose type's `domain`
#   (`+0x218`) is 2 skips it (`0065234e`–`0065237c`), and so does the
#   attrition flag `param_5` (`00652341`). Item 1350.
# - `Wall::process`: a started building on a tile an enemy owns takes
#   `take_damage(8, …, attrition = 1)` every 32 frames, phased by `o`
#   (`docs/CITIES.md` §9.5). No capture on disk had one.
# - A building target is seen through its `ever_seen` byte
#   (`BuildData::is_seen@0062e1a0`, `docs/VISION.md` §10.4): an attack
#   ordered on T before who=0 has seen it is invalid, and the squad
#   retargets a builder. S is there to light T first.
# - A city or fort on unowned ground is a region's first only
#   (`docs/CITIES.md` §2.5), so a who=1 site cannot stand in neutral land
#   where who=1 has its capital; the enemy land comes to T instead.
#
# ---------------------------------------------------------------------------
# THE STAGING, walked by this crate on run514's start (RON_STAGE,
# RON_STAGE_ALL, RON_STAGE_DRAWS), trace frames:
# - 606: T laid, A building it at 183 a frame; F's claim reaches T's cell
#   by 682.
# - 682: T's first enemy-land hit, eight whole, `job_counter` untouched;
#   every 32 frames after (714, 746, …).
# - 934..1034: P's blows on T, with sixteenths (71/10 on 934); the builders
#   at a quarter rate under attack.
# - 1093..1110, 1220..1237, 1347..1364: the bomber's three passes, each
#   leaving `job_counter` where the builders had it.
# - 1444: T finished, at 109 hits; 1474..1482 the fourth pass, and T dead
#   on 1482.
# - The mutations, walked against the staging unmutated
#   (`RON_STAGE_DRAWS`): the progress from `hit.whole` (no carry) parts the
#   draws on 1448; an aircraft's blow costing progress on 1360; attrition
#   costing progress on 1450.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire. The walk prints the
# trace frame; its block is the next.
#
# 1. T absent on block 607, or gone (disbanded unstarted) when the border
#    reaches it.
# 2. The enemy land: T's `damage` unchanged on block 683.
# 3. Attrition costing progress: T's `job_counter` on block 683 short of
#    block 682's plus one frame's work.
# 4. The carry: T's `damage_frac` nonzero after P's first blow (block 935).
# 5. The air arm: T's `job_counter` on block 1094 below block 1093's.
# 6. T finished on a block other than 1445.
# 7. An `I_ISSUE` refusal on any line (parked 1256).

0 !ai off
600 library who=0 6
600 library who=1 1
601 resource who=1 all +2000
602 add 3 citizen who=1 218,120
604 be 1
606 @build 1 42624 23424 439 6 7 8
608 be 0
610 add airbase who=0 200,150
612 add bomber who=0 196,158
640 @flight 0 2007 0 6
650 add fort who=0 222,178
860 add scout who=0 226,128
900 add elite_pikemen who=0 224,127
902 @attack 0 2006 1 8
950 @strike 0 2006 1 6
1040 @move 0 42720 30000 8
