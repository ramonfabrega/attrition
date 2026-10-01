# Golden record, chapter fifty — three arms the unit tests hold and no walk
# (DECISIONS 56 §3): `BuildData::get_shot`'s exact-type arms (1394: a Stockade
# at the first age flies at its PROJ_SPEED, the default's `ages` reads 0 and
# 90), `Group::target_opportunity`'s 15-frame cooldown and its member
# `find_melee_target` arm (1403's mutation: a combat-role captain beside a
# non-combat member under hit). The group's `+0x4b` arm (1335) was walked and
# is parked: docs/GOLDEN.md §59 (item 1404).
#
#   run577:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch50 \
#       --map 14 --end-frame 1100 --log-window 605 1100 --timeout 5400 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=5,GROUPS=1,AMMO=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter50.cmd
#
# Two Stockades (who=1, 1/2006 and 1/2007) at the first age; who=0's Scout
# 0/6 and a Hoplite squad 0/7..9, group-attack-moved through their range
# (the Scout leads, so the arrows land on a non-combat member of a group
# whose captain is combat-role; two hits two frames apart at 748 and 750
# make the cooldown's gate).
0 !ai off
602 add stockade who=1 222,122
604 add stockade who=1 226,118
606 add scout who=0 236,138
608 add hoplites who=0 244,146
640 @amove 0 38000 22000 6 7
