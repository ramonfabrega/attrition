# Golden record, chapter forty-seven — three arms the widenings hold and no
# walk (DECISIONS 56 §3): a Tower site standing when its leader gains the
# Keep, a squad put ashore facing away from its barge, and an age's snap on
# a unit walking a move whose angle is not its heading.
#
# docs/GOLDEN.md §56 (item 1310). A cast of its own on the golden start,
# chapter forty-six's Dock and lake:
#
#     0 `!ai off`                                   every chapter's
#   600 `library who=0 1`, 604 a who=0 Dock         chapter forty-three's
#   606 `add hoplite who=0 84,163`                  H 0/6, 0/7, 0/8: one
#                                                   squad, on the east shore
#   608 `add 3 citizen who=0 36,178`                Z 0/9, 0/10, 0/11
#   612 `@move 0 11904 34944 6`                     H west into the lake:
#                                                   barge B 0/12, the squad
#                                                   aboard facing south-west
#   620 `@build 0 7296 34176 439 9 10 11`           Z lay a Tower site T
#                                                   0/2008 and build it
#   676 `tech who=0 keep on`                        the Keep gained, T a site
#   678 `@move 0 3840 36864 9 10 11`                Z walk off: T alone
#   680 `add hoplite who=1 38,172`                  who=1's squad, onto T
#   860 `@move 0 14572 31200 12`                    B to the waterline
#  1100 `@move 0 15360 31200 12`                    B ashore, heading east:
#                                                   dead, the squad put out
#  1110 `age who=0 2`                               the snap, `0/8` mid-move
#
#   run514 (item 1310), the take:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch47 \
#       --map 14 --end-frame 1400 --log-window 605 1400 --timeout 5400 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=5,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter47.cmd
#
# The last line is 1110, the squad's moves end near 1134, and 1400 leaves
# 250 blocks past it.
#
# ---------------------------------------------------------------------------
# THE READINGS (docs/GOLDEN.md §56 has them whole).
#
# - `run_cmd`'s `tech` case (`007dd966`) names its argument through
#   `parse_type(·, "tubs", 2)`, whose `b` block walks the building types
#   `0x19e..0x21f` after the units: `keep` is the Keep (440), and the case
#   calls `Leader::gain_tech(who, 440, 0, 0, ·, 1)`.
# - `Leader::gain_tech@006dcb60`'s building loop (`6dde64`..`6ddf2b`) takes
#   every object of the leader's two building lists that is **in use**
#   (`testb $1, 8(obj)`) and whose type's `upgrade` (`TypeData +0x44`) is
#   the type gained, and `set_type`s it through vslot `+0x84` — the PE's
#   `0x640da0`, `Wall::set_type`: `decrement_stats`, `SubObject::set_type`,
#   `increment_stats`. Nothing there asks whether the site is finished.
# - A site's health is `construct_hits` less its damage, and
#   `construct_hits` is its job's share of the type's `HITS` (Tower 750,
#   Keep 1000): a Keep site takes a third more blows.
# - `come_out@6191f4`: a member put ashore is turned to its captain's angle
#   (item 1291); this squad boards heading south-west and its barge lands
#   heading east.
# - The age's snap writes the figure's facing and leaves the order's
#   `dest_angle` (`UnitData +0x58`) — item 1281. Its only writers are
#   `Unit::init` and `Unit::update_action`.
#
# ---------------------------------------------------------------------------
# THE STAGING, walked by this crate on run496's start (RON_STAGE,
# RON_STAGE_ALL, RON_STAGE_DRAWS), trace frames:
# - 621: T laid, Z building; 676: T a Keep site at 96 hits, 72 as a Tower.
# - 680: who=1's squad 1/6..1/8 at T; 965: T dead. With the site left a
#   Tower (1264's mutation B) it dies on 901, and the draws part on 915.
# - 707: H aboard B 0/12, facing −1541996544 (south-west); 1101: B dead at
#   (14572, 31200), 0/6..0/8 ashore at (14712, 31224), (14712, 31368) and
#   (14856, 31320), each under a group move east; arrived 1128..1134.
# - 1110: `0/8` walking, heading 780926976 against `dest_angle` 1073741824.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire. The walk prints the
# trace frame; its block is the next.
#
# 1. The in-use arm: T's `type` other than 440 on block 677.
# 2. T alive on block 966, or dead before it.
# 3. The landing: 0/12 alive on block 1102, or 0/6..0/8 `inside` 12 there;
#    0/7's and 0/8's `angle` not 0/6's 1101 value on 1102.
# 4. The snap: 0/8's `dest_angle` other than 1073741824 on block 1111.
# 5. An `I_ISSUE` refusal on any line (parked 1256).

0 !ai off
600 library who=0 1
604 add dock who=0 53,153
606 add hoplite who=0 84,163
608 add 3 citizen who=0 36,178
612 @move 0 11904 34944 6
620 @build 0 7296 34176 439 9 10 11
676 tech who=0 keep on
678 @move 0 3840 36864 9 10 11
680 add hoplite who=1 38,172
860 @move 0 14572 31200 12
1100 @move 0 15360 31200 12
1110 age who=0 2
