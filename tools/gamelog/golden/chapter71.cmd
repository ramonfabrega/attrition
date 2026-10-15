# Golden record, chapter seventy-one — the Senate's government patriot, born
# under upgraded Generals and Supply: `Unit::update_hits@0060e930`'s hero
# term and its supply term both apply to The Despot (item 2436,
# `docs/GOLDEN.md` §88; the arm is `docs/AI.md` §213, diff-backed there
# only on an AI Senate in the successor pair).
#
# THE STAGING, walked by this crate on run492's start (RON_STAGE=ch45:
# chapter71:1100, RON_STAGE_ALL):
# - 602: a Senate `0/2007`. 604..606 Forage, Supply and Logistics
#   (SUPPLY_WAGONS_1..3), 607..609 Tactics, Operations and Strategy
#   (GENERALS_UPGRADE_1..3). 610: Despotism (623) queued at the Senate.
# - 1011: the research finishes, `Build::finished`'s Senate tail trains
#   The Despot `0/6` (TypeIndex 352) at **659 hits**: 109, + 3^2 x 109 / 2
#   (the hero term, 599), + the supply term at level 3 (60). Without the
#   supply term on a unit that is both hero and supply: 599.
#
# WHAT WOULD FALSIFY IT. A call on trace frame F is on block F+2.
# 1. The Despot's `hits`/`myhits` 599 on its birth block: the supply term
#    is gated on a hero.
# 2. No Despot born, or born off 1011: the tail's gate or the research time.
# 3. 109: the upgrade counts are not read (levels 0).
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch71 \
#       --map 14 --end-frame 1120 --log-window 600 1120 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=5,DEATHS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter71.cmd
#
0 !ai off
600 library who=0 1
601 resource who=0 all +5000
602 add senate who=0 14,74
604 tech who=0 forage on
605 tech who=0 supply on
606 tech who=0 logistics on
607 tech who=0 tactics on
608 tech who=0 operations on
609 tech who=0 strategy on
610 @queueup 0 623 1 2007
