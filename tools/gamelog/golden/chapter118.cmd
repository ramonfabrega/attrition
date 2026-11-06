# Golden record, chapter one hundred and eighteen -- a Disband in a city (item 3930, GOLDEN 168; parked 3918a).
# Chapter one hundred and fifteen's staging whole (GOLDEN 165: `0/2007` at 14,74, city -1, holds a DISBAND from 622
# and is gone on 832; `0/2008` at 28,160, city 0, refused at 641 and 661 by `Group::action_queue_up`'s research arm
# while `0/2007`'s entry stood), and one more press of the DLL's `@queueup` with type 666 at `0/2008`:
#   840: no DISBAND entry stands anywhere (run1512: `0/2008` reads `queued 0`, city 0, flags 7 from 617 to the
#        trace's end), so the research arm passes and `Build::queue_up@00620f40` -> `could_queue` -> `can_make` ->
#        `has_spell(0x29a)` -> `SpellTypeData::is_castable@00675bc0(b, 0, 0)` reaches its city arm: `0/2008` is not a
#        city (`+8 & 0x20` clear) and sits in city 0, so `CityData::is_disbanding@007380a0` is asked; 0 > 0 is false
#        and the entry is laid. Its time is the Barracks' own, 21000 (no city, Senate or Lakota scale), so it
#        reaches 21000 on 1051; `Build::finished` -> `SpellType::cast(b, 0, 0, 0)` -> `is_castable(.., 1)` asks
#        `is_disbanding` again, 1 > 1 is false, and `cast_disband@00671ed0` closes it on 1052.
# Killers, before the take (the journal's list): (1) the cover=1 twin enters no `CityData::is_disbanding` on trace
# frame 841, the press's; (2) `0/2008` reads `queued 0` on 842 -- the research arm, or another arm of `can_make`,
# refused again; (3) `0/2008` still in BUILDS on the block after its entry's job_counter reaches 21000 -- the
# finish's `is_disbanding` counted more than its own entry; (4) `0/2008`'s entry runs other than 100 a frame to
# 21000 -- a city scaling this crate does not read.
#   run1517 (cover=0), run1517b (cover=1), both with the golden start's lobby pinned:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch118 \
#       --map 14 --end-frame 1200 --log-window 605 1200 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --ai-tribe 11 --profile DIFFICULTY=0 --profile STARTING_RESOURCES=1 \
#       --profile STARTING_TECHNOLOGY=0 --profile ENDING_TECHNOLOGY=7 \
#       --cmd-file tools/gamelog/golden/chapter118.cmd
0 !ai off
606 add barracks who=0 14,74
610 add chariot who=0 20,60
614 add hoplite who=0 10,90
616 add barracks who=0 28,160
620 @queueup 0 666 1 2007
640 @queueup 0 666 1 2008
660 @queueup 0 666 1 2008
840 @queueup 0 666 1 2008
