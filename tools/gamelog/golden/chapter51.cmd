# Golden record, chapter fifty-one — the arms the unit tests hold and no walk
# (DECISIONS 56 §3, `docs/GOLDEN.md` §61, item 1423): `Group::target_opportunity`'s
# 15-frame cooldown, its member `find_melee_target` arm, and a building's
# `visible` clear gate (`Build::do_attack@006228f0`, `622978`).
#
#   run582:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch51 \
#       --map 14 --end-frame 1100 --log-window 605 1100 --timeout 5400 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=5,GROUPS=1,AMMO=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter51.cmd
#
# Chapter fifty's script with the attack-move one frame earlier, 639 for 640.
# THE STAGING, walked by this crate on run577's start (RON_STAGE, stage_walk),
# trace frames:
# - the cooldown: the group's `think_frame` (GROUPDATA `+0x38`) moves on the
#   Scout's hits at 699, 724 and 750 and not on the ones between (run577's
#   dump holds the same three values on its own staging).
# - the clear gate: the Stockade `1/2006`'s round that kills the Scout lands
#   on its phase 1, so its next call is on phase 0 with the latch up, finds no
#   target, and fires nothing: 778. The gate holds `visible` at 1 there; the
#   gate dropped clears it. Chapter fifty's own 640 leaves that call on phase 1.
# - the member arm is not reached: `0/7` always holds the `ATTACK_TO` of the
#   attack-move, so `Group::target_opportunity` takes its third arm. No
#   staging of a Stockade's shot puts the hit on the Scout while a Hoplite is
#   in reach; the arm is held by `fight::tests` alone.

0 !ai off
602 add stockade who=1 222,122
604 add stockade who=1 226,118
606 add scout who=0 236,138
608 add hoplites who=0 244,146
639 @amove 0 38000 22000 6 7
