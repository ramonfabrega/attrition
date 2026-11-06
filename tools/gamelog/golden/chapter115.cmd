# Golden record, chapter one hundred and fifteen -- a Disband queued at a Barracks (item 3918, GOLDEN 165; AI 392.3,
# parked 3907a). Chapter thirteen's cast to 614 (GOLDEN 21: `!ai off`, the Barracks `0/2007` at 14,74 -- city -1 on
# run285 -- a Chariot and a Hoplite squad), a second Barracks for who=0 at 28,160, inside the City `0/2000`'s city
# (chapter four's Temple there reads `city 0`), and three presses of the DLL's `@queueup` (verb 18,
# `CommandManager::issue_queue_up@00941be0`, which tests nothing of the type) with type 666, `DISBAND` (0x29a):
#   620: `0/2007`, outside any city: `Build::queue_up@00620f40` exempts DISBAND from the library forward; the entry's
#        time is `ObjectData::train_time@006508c0:51`'s, the Barracks' own (JOB_TIME 420, x100) / 2 = 21000, about
#        263 frames at the Hoplite's pace on run285 (188 -> 236 frames); `Build::do_queue` -> `Build::finished@00628490`
#        -> `LeaderData::has_spell(666)` -> `SpellType::cast@00676ce0(b, 0, 0, 0)` -> `is_castable(.., 1)` == 3 ->
#        `cast_disband@00671ed0`: `Build::plunder` (active), `Object::disband(b, 0)`.
#   640: the second Barracks (expected `0/2008`), in city 0: `is_castable`'s 0x29a arm asks `CityData::is_disbanding`,
#        which counts the city's buildings' queued 0x29a entries; at the finish its own entry still stands (the
#        unqueue follows `finished`), 1 > 1 is false, and the cast goes on.
#   660: the same Barracks again, a second DISBAND behind the first.
# Falsifiers, before the take (the journal's list): (1) `0/2007`'s `queued` not 1 on 622, or `0/2008`'s not 1 on 642
# -- `queue_up` or `Group::action_queue_up@006fdbb0`'s gate refused DISBAND; (2) `0/2007` still in BUILDS on the block
# after its entry's job_counter reaches 21000; (3) the same for `0/2008`; (4) `0/2008`'s `queued` 2 on 662 (the second
# press was taken); (5) the cover=1 twin: no HIT on `SpellType::cast_disband@00671ed0`.
#   run1512 (cover=0), run1512b (cover=1, `--cover cover=1 --no-ffwd --ai-tribe <run1512's who=1 tribe>`):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch115 \
#       --map 14 --end-frame 1100 --log-window 605 1100 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter115.cmd
0 !ai off
606 add barracks who=0 14,74
610 add chariot who=0 20,60
614 add hoplite who=0 10,90
616 add barracks who=0 28,160
620 @queueup 0 666 1 2007
640 @queueup 0 666 1 2008
660 @queueup 0 666 1 2008
