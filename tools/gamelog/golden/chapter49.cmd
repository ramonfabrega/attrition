# Golden record, chapter forty-nine — three arms the unit tests hold and no
# walk (DECISIONS 56 §3): a Tower's arrow past its owner's second age
# (`get_shot` gives the type's PROJ_SPEED, item 1380's K3), the sixteen-frame
# review under the Bombard's pack arm (1378, parked: see GOLDEN §58), and
# `get_cost`'s bump loop's JUMP arm (1352). docs/GOLDEN.md §58 (item 1393).
#
#   run576:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch49 \
#       --map 14 --end-frame 1500 --log-window 605 1500 --timeout 5400 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=5,GROUPS=1,AMMO=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter49.cmd
#
# Both players at the Gunpowder age (`library N` = age N). who=1's Tower T
# 1/2006 shoots who=0's Scouts at get_shot 1 (the type's PROJ_SPEED, not 90).
# who=0 owns a Catapult and queues the Bombard's research (two JUMP links
# ahead: Catapult -> Trebuchet -> Bombard; Bombard is jumpable) at the Siege
# Factory 0/2007, then trains Catapults: price (70 + 10) x 0.9 = 72 each.
0 !ai off
600 library who=0 3
600 library who=1 3
600 tech who=0 catapult on
601 resource who=0 all +2000
601 resource who=1 all +2000
602 add siege_factory who=0 200,150
603 add tower who=1 222,122
606 add scout who=0 226,128
620 @queueup 0 267 1 2007
622 @queueup 0 265 1 2007
640 @move 0 40000 24500 6
700 @queueup 0 265 1 2007
800 add scout who=0 226,128
802 @move 0 41000 22500 7
