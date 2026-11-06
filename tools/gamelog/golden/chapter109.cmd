# Golden record, chapter one hundred and nine — the Airbase's recall and
# scramble, two bound keys on chapter thirty-three's Airbase.
#
# docs/GOLDEN.md §159 (item 3912; AI §391.2 rows 4 and 5). Chapter
# thirty-three's lines to its 2280 launch patrol — the strike from 0/2007
# on 2260 (0/8, on 1/2006) and the patrol on 2280 (0/9, over P1) — then
# the Airbase selected and two keys pressed on it:
#    2290 `select 2007 who=0`
#    2292 `@key 124`   OPTION_AIRBASE_RECALL: 0/8 and 0/9 in flight
#    2312 `@key 125`   OPTION_AIRBASE_SCRAMBLE: 0/7 and 0/6 inside, full
#                      (0/6 from 2302), the recalled planes refuelling
#
# bindkey OPTION_AIRBASE_RECALL=124
# bindkey OPTION_AIRBASE_SCRAMBLE=125
#
#   run1506 (cover=0, the word) and run1506b (cover=1, the blind rows):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch109 \
#       --map 14 --end-frame 2600 --log-window 605 2600 --timeout 6600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --bindkey OPTION_AIRBASE_RECALL=124 --bindkey OPTION_AIRBASE_SCRAMBLE=125 \
#       --cmd-file tools/gamelog/golden/chapter109.cmd
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, written before the take (docs/journal/…-item-3912.md):
#
# 1. The keys reach their options: INFO 22 on 2292 names OPTION_AIRBASE_RECALL
#    (324) and on 2312 OPTION_AIRBASE_SCRAMBLE (329); KEYMAP_NONE means the
#    entry's selection test refused the console's `select` of a building.
# 2. The CM killers: no `process_recall 2293` / `process_scramble 2313`.
# 3. The recall (2294): 0/8 and 0/9 each one STRAFEORDER, no target,
#    home 0/2007, returning, mandatory 0, flags 0 (the reading: the Clear's
#    hangar half, then the walk); still the strike and the patrol (killer).
# 4. The scramble (2314): 0/7 and 0/6 each one AIRPATROLORDER over
#    0/2007's own point, flags 4 (the reading); none (killer); a landed,
#    refuelling 0/8 or 0/9 passed over (the mana_burn gate).
# 5. The HIT rows: Group::action_recall and action_scramble entered on
#    run1506b's trace; absent, the row stays NEVER.
#
# A call on trace frame F is on block F+2 (§17).
0 !ai off
600 library who=0 6
606 add airbase who=0 60,72
610 add fighter who=0 60,84
612 add bomber who=0 52,84
614 add bomber who=0 68,84
616 add barracks who=1 110,86
620 @strike 0 2006 1 7 8
640 @flight 0 2007 0 6
660 @flight 0 2007 0 7 8
664 @strike 0 2006 1 7 8
766 @strike 0 2006 1 6
1540 @queueup 0 287 1 2007
1600 @gatherpoint 0 11520 7680 0 2007
1700 @gatherpointadd 0 5760 5760 0 2007
1850 @gatherpoint 0 -1 -1 0 2007
2200 add barracks who=1 72,78
2260 @launchstrike 0 2006 1 2007
2280 @launchpatrol 0 13440 9600 2007
2290 select 2007 who=0
2292 @key 124
2312 @key 125
