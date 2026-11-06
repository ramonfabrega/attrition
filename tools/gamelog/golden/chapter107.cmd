# Golden record, chapter one hundred and seven -- a who=1 Fort dies inside its owner's territory (item 3901, GOLDEN 157; parked 3898a).
# `Forts::close_fort@0073ec90` calls `Region::fix_borders` for a fort whose cell has a region (AI 387); run1502's Fort stood in
# the open and no block either side of its death was diffed for the border. Here the Fort stands on cell (48, 28), inside
# who=1's frame-0 territory and one cell from its south-west edge, so its own claim pushes the border out; it is added on 620
# (complete at once: `Build::activate` -> `Forts::init_fort`, a fix), and killed by the console's `die` on 700 (`Object::die`
# -> `Build::close` -> `close_fort`, a second fix). `@dumpall` blocks carry `WORLD` (every cell's `who`/`who2`) either side.
# This crate's walk (RON_STAGE_OWNERS=1): the Fort's claim reaches the map on 623..627 (who=1 261 -> 442 cells, x 37..58,
# y 17..38) and its death's on 703..707 (442 -> 261, the same five bands). Falsifiers, before the take: (1) block 708 holds
# the Fort's cells still who=1 -- the border unmoved by the death; (2) a cell outside x 37..58, y 17..38 changes owner across
# 699..716 -- a region change beyond the fort's claim; (3) `fix_borders` not reached: settled from disk, run1502's cover=1
# trace enters `Region::fix_borders@00680f60` first on 622, its `die` frame, beside `close_fort`.
#   RON_STAGE=ch106:chapter107:760 RON_STAGE_OWNERS=1
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch107 \
#       --map 14 --end-frame 760 --log-window 605 760 --timeout 5400 \
#       --detail end:UNITS=3,BUILDS=7,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter107.cmd
0 !ai off
619 @dumpall
620 add fort who=1 193,113
622 @dumpall
624 @dumpall
626 @dumpall
628 @dumpall
699 @dumpall
700 die 2006,1
700 @dumpall
701 @dumpall
702 @dumpall
703 @dumpall
704 @dumpall
705 @dumpall
706 @dumpall
707 @dumpall
708 @dumpall
716 @dumpall
