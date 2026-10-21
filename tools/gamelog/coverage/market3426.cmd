# The human's Market at cover=1 — item 3426 (`docs/NATIONS.md` §9.2), not a
# chapter: the golden start (map 14, the human Nubian, tribe 4, whose start
# holds a Market, `num_buildings[0x16]` 1 in run835's LEADERS=9 start dump)
# sells one lot of timber and buys one back through the tracer's leader verbs,
# before Coinage, so only the Nubian arm opens `BUY_SELL`.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/run1031 \
#       --map 14 --end-frame 700 --log-window 600 700 --timeout 3600 \
#       --detail end:LEADERS=2,BUILDS=7,CITIES=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/coverage/market3426.cmd --cover cover=1
#
# run1031b is the same line with `--cover cover=0`, the twin.
0 !ai off
605 resource who=0 all +1000
620 @sell 0 1 0
640 @buy 0 1 0
660 @sell 0 1 1
700 !quit
