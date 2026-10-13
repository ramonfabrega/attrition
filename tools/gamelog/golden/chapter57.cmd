# Golden record, chapter fifty-seven — chapter fifty-six's staging with the death
# list printed (item 1863, `docs/GOLDEN.md` §69). Run786's `end:` detail has no
# `DEATHS`, so `death:extra` (ours 1, theirs 0) on 874, 900 and 961 compared this
# crate's three Hoplite corpses against a record nobody asked for. The script is
# chapter fifty-six's whole; only the end detail grows `DEATHS=1`.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch57 \
#       --map 14 --end-frame 1010 --log-window 605 1010 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=5,GROUPS=1,AMMO=5,DEATHS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter57.cmd
0 !ai off
602 add dragoon who=0 225,152
604 add dragoon who=0 225,176
606 add stockade who=1 218,176
712 add citizen who=1 243,151
712 @move 0 46800 29288 6
763 add hoplites who=1 237,150
