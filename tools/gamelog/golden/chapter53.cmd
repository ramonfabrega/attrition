# Golden record, chapter fifty-three — the arms chapter fifty-two did not
# reach (DECISIONS 56 §3, `docs/GOLDEN.md` §63, item 1707): the member
# `find_melee_target` arm of `Group::target_opportunity@007107d0` (parked
# 1688), and the cavalry archer's two arms the unit tests alone hold, M5
# (`move_step`'s drop when the pivots do not bear) and M6 (`cavarch_fight`'s
# captain re-search), docs/AI.md §134.5 (parked 1689).
#
#   run772 (run771 was the same script with the Dragoon's pair added on 612 and
#   moved on 640, `~/ron-golden/ch53a`: its word is 652, the original's round
#   from the Dragoon's first strike, below):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch53 \
#       --map 14 --end-frame 1120 --log-window 605 1120 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=5,GROUPS=1,AMMO=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter53.cmd
#
# THE STAGING, walked by this crate on run582's start (RON_STAGE, stage_walk),
# trace frames:
# - the member arm: a Stockade `1/2006` at (215,128) west of an idle group
#   (a Scout `0/6` and two Hoplite squads, captains `0/7` and `0/10`, standing
#   at their `@move`'s end at x 45000 — 24 tiles from it). The Scout walks 40
#   tiles east *through* the Stockade's reach after the Hoplites have stopped:
#   a Stockade fires at a moving Scout and not at a standing one (`do_attack`'s
#   `(o + frame + 14) & 0x1f` gate), and ranks a combat unit first whenever one
#   is in reach, so the Hoplites stand beyond it. The Scout's wounds land
#   (818, 844, 872, 906, 942 here) with both captains' `get_action` null and
#   `order_type` NONE: arm 2, `find_melee_target` at `min(dist + 0xc0,
#   unit_respond_range * 0x240)`.
# - M5 and M6: the Dragoon `0/31` (FIRES_ON_MOVE) on a plain `@move` east along
#   a line of a player-1 Scout `1/6` and two Hoplite squads. The Scout is held
#   first (not combat-role); when the Hoplites come into reach the captain's
#   re-search changes the target (M6, 1002 here); the step drops the held
#   target when the pivots do not bear (M5, 1062 and 1064 here). The pair is added
#   on 942, after the member arm's last wound, so the Dragoon's round (the
#   word of run771) parts the walk after the arm it would otherwise hide.
# The two-order snap (parked 1500's converse) is not staged: no walk reaches
# `move_step`'s snap on a turn at all (docs/GOLDEN.md §63).

0 !ai off
602 add stockade who=1 215,128
606 add scout who=0 175,128
608 add 4 hoplites who=0 236,128
610 add 4 hoplites who=0 236,131
639 @move 0 45000 24600 6 7 10
942 add dragoon who=0 225,152
944 add scout who=1 232,148
946 add 2 hoplites who=1 238,148
970 @move 0 46800 29288 31
