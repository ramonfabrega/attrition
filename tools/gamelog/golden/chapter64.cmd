# Golden record, chapter sixty-four — the Bark's bit and a player's stance
# button (item 2165, `docs/GOLDEN.md` §79; parked 2100).
#
# Three arms of one staging (DECISIONS 56 §3: what the AI track built and no
# walk holds):
#
# 1. `Unit::init@00612100:604-611`: a unit of the Bark line (`is(BARK)`,
#    TypeIndex 0x143) sets `unit_masks2 |= 4`, and, for a computer's leader
#    (`leader_flags & 4` clear, `LeaderData::is_human` inlined), calls
#    `Unit::set_stance@00605310(3, 0)`: RAID. Every AI Bark on disk carries
#    `unit_masks2 4` and `stance 3` (run155's `1/34`); no golden chapter
#    holds a Bark, and this crate sets neither.
# 2. `Unit::fight@005fd4d0:396-400`: with `unit_masks2 & 4`, the captain's
#    re-search predicate is "the target's role has 0x10000 and its type's
#    `unit_flags & 0x2000` (`n`, destroyed when it attacks) is clear" in
#    place of "not combat role" (parked 2100). A Bark re-finds its target
#    off a warship and keeps a fishing boat.
# 3. A player's stance button: `CommandManager::issue_stance@00941530` →
#    `CommandPackage::process_stance@00949ed0` → `Group::action_stance@
#    0070d440`. For a human (`leader_flags & 4` set), options 1, 2 and 5
#    `repath` and `kill_current_order(0)` a current order without the action
#    bit (`flags & 4`); every member takes `+0x8 |= 0x10`. The AI's armies
#    reach `action_stance` and never this arm (`docs/GROUPS.md` §8). The DLL
#    had no stance verb: `@stance <who> <s> <o>…` (item 2165).
#
# THE STAGING, walked by this crate on run127's start (RON_STAGE=ch5,
# RON_STAGE_ALL): chapter five's sea, region 70. Two human Barks `0/6` on
# (60, 180) and `0/7` on (62, 180); who=1's Trireme `1/6` on (64, 188) and
# fishing boat `1/7` on (58, 188); who=1's Bark `1/8` on (76, 192), born to
# a computer leader. 680: `0/7` (attacking since 633 in the walk, an order it
# found) is set to STAND_GROUND (2). 720: `0/7` cycles forward (−1) from the
# group's modal option.
#
# CENSUS ROWS the lobby reaches, never backed (`census.py --unbacked`):
# `CommandManager::issue_stance@00941530`, `CommandPackage::process_stance@
# 00949ed0`, `Group::action_stance@0070d440`, `GroupData::get_stance_type@
# 0070d370`, `GroupData::get_stance_option@0070bab0` (the −1 cycle),
# `Unit::set_stance@00605310` (the computer Bark's birth).
#
# WHAT WOULD FALSIFY IT, read before the run. (1) An `I_ISSUE` refused on 680
# or 720 (`issuesmatch.py --none-refused`). (2) `1/8`'s first block reads
# `stance 0`, or `0/6`'s `stance 3`: the leader bit read backwards. (3) No
# Bark carries `unit_masks2 4`: the lineage test is not `is(0x143)`. (4) On
# block 681 `0/7` still holds its ATTACK: the human arm is not 1/2/5's, or its
# order carries the action bit. (5) On block 721 `0/7`'s `stance` is not 3:
# the cycle does not read the modal option.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch64 \
#       --map 14 --end-frame 820 --log-window 605 820 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter64.cmd
0 !ai off

610 add bark who=0 60,180
612 add bark who=0 62,180
615 add trireme who=1 64,188
618 add fisher who=1 58,188
621 add bark who=1 76,192

680 @stance 0 2 7
720 @stance 0 -1 7
