# Golden record, chapter seventy-five — `finish` behind a population-capped
# queue head: the held-head arm of `Console::hurry_action@007c3810`'s queue
# arm (item 2519, `docs/GOLDEN.md` §92; parked 2503).
#
# THE GATE. `finish` on a selected building with a queue pays one entry the
# head's train time: the head, unless its counter already equals its own
# train time — a done head the original holds — and then
# `BuildQueueData::get_next_non_unit@00630b10`'s entry, the first behind it
# whose type is not a unit type or whose availability bit is clear. Chapter
# seventy-four held the head arm behind a Library's research; no capture
# held a *unit* head done and refused. Here a Barracks' Bowmen head is done
# and refused by the population cap (`Build::finished`: `cap < control + pop`,
# 36 against 25), and a Special Forces `0/…` queued behind it (a unit type
# whose tech is clear and researchable at age 8: `type_avail` 2, the one such
# type on the golden start) takes the frame's `do_queue` fallback every frame
# and the `finish`'s train time once.
#
# THE STAGING (walked by this crate on run492's start, `RON_STAGE=ch45:chapter75:960`):
# `age who=0 8`, `resource who=0 all +50000`, a Barracks `0/2007` at (14,74),
# thirty Citizens added (control 36 over the Ancient cap of 25), Bowmen (170)
# queued on 640 and Special Forces (75) on 641 at the Barracks; the head's
# train time is 20280 and it is done and refused from 843. `900 select 2007`,
# `901 finish`. Not 801: the head has not reached its train time yet and the
# plain head arm pays it instead.
#
# WHAT WOULD FALSIFY IT.
# 1. The Special Forces entry's `job_counter` on 902 other than the head's
#    train time past its own running counter (ours 5600 on 899, 26280 on
#    902): `get_next_non_unit` does not take the done head's place.
# 2. The head's counter on 902 above its train time: the head, not the
#    entry behind it, was paid.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch75 \
#       --map 14 --end-frame 960 --log-window 600 960 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=5,DEATHS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter75.cmd
#
0 !ai off
600 age who=0 8
601 resource who=0 all +50000
606 add barracks who=0 14,74
612 add 30 citizen who=0 12,80
640 @queueup 0 170 1 2007
641 @queueup 0 75 1 2007
900 select 2007 who=0
901 finish
