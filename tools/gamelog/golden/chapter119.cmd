# Golden record, chapter one hundred and nineteen -- a Disband at a Temple in a city (item 3934, GOLDEN 169; parked 3930a).
# Chapter one hundred and eighteen's staging (GOLDEN 168) with `add temple who=0 28,160` (chapter four's Temple tile,
# city 0) in place of the second Barracks. A Temple's `BUILD_FLAGS` are `jam`: no `e`, so `NO_CITY` is clear and
# `SpellTypeData::is_castable@00675bc0`'s 0x29a case goes on to the city arm that the Barracks (`ean`) skipped:
#   620: `0/2007` (Barracks, 14,74, city -1) lays a DISBAND (run1517: 622..831, gone on 832);
#   640, 660: the Temple `0/2008`'s presses, refused whole by `Group::action_queue_up`'s research arm while `0/2007`'s
#        entry stands (as run1517's Barracks was);
#   840: no entry stands, so `Build::queue_up@00620f40` -> `could_queue` -> `can_make` -> `queue_here(0x29a)` (1 for
#        DISBAND whatever the building, `00639d10`'s head) -> `type_avail(leader, 0x29a, 1)` (the craft on the leader,
#        as run1517 answered it) -> `has_spell(0x29a)` -> `is_castable(b, 0, 0)`: the Temple is not a city
#        (`+8 & 0x20` clear) and sits in city 0, so `CityData::is_disbanding@007380a0` is asked; 0 > 0 is false and
#        the entry is laid on 842. The Temple's JOB_TIME is the Barracks' 420, so 21000 on 1051; `Build::finished` ->
#        `SpellType::cast` -> `is_castable(.., 1)` asks `is_disbanding` again, 1 > 1 is false, and the Temple closes
#        on 1052.
# Killers, before the take (the journal's list): (1) the cover=1 twin enters no `CityData::is_disbanding` on trace
# frame 841, the press's; (2) `0/2008` reads `queued 0` on 842 -- an arm of `can_make` refused a Temple;
# (3) `0/2008` still in BUILDS on 1052 -- the finish's `is_disbanding` counted more than its own entry;
# (4) the entry runs other than 100 a frame to 21000.
#   run1519 (cover=0), run1519b (cover=1, --twin), both with the golden start's lobby pinned:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch119 \
#       --map 14 --end-frame 1200 --log-window 605 1200 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --ai-tribe 11 --profile DIFFICULTY=0 --profile STARTING_RESOURCES=1 \
#       --profile STARTING_TECHNOLOGY=0 --profile ENDING_TECHNOLOGY=7 \
#       --cmd-file tools/gamelog/golden/chapter119.cmd
0 !ai off
606 add barracks who=0 14,74
610 add chariot who=0 20,60
614 add hoplite who=0 10,90
616 add temple who=0 28,160
620 @queueup 0 666 1 2007
640 @queueup 0 666 1 2008
660 @queueup 0 666 1 2008
840 @queueup 0 666 1 2008
