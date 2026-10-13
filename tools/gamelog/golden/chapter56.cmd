# Golden record, chapter fifty-six — the rows chapter fifty-five left (item
# 1829, `docs/GOLDEN.md` §68): chapter fifty-five's staging whole (a captain
# Dragoon `0/6` east past a non-combat-role Citizen, a second Dragoon at a
# Stockade) and one Hoplite squad added at 763, the frame M6's re-search
# first rolls, within the Dragoon's reach and dearer than the Citizen: the
# search answers a different, in-reach unit, so `research && false`
# (the mutation 1798 found no walk held) strikes the Citizen instead and
# the frame's draws part (walked on run772's start, 764: 6 draws against 5).
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch56 \
#       --map 14 --end-frame 1010 --log-window 605 1010 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=5,GROUPS=1,AMMO=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter56.cmd
0 !ai off
602 add dragoon who=0 225,152
604 add dragoon who=0 225,176
606 add stockade who=1 218,176
712 add citizen who=1 243,151
712 @move 0 46800 29288 6
763 add hoplites who=1 237,150
