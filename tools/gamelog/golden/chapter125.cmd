# Golden record, chapter one hundred and twenty-five -- a Library closed with a technology queued (item 3969, GOLDEN 175; T71-7).
# `Build::close@00628980` asks at its entry (`0x628a2d`) whether the building is its owner's first library, and at
# `:468`-`:521` hands that library's queue to `get_first_library`'s next answer -- appended, progress and price kept, no
# count moved -- or, with none, `clean_queue(1)`: a refund, each entry's `num_queued[type]` down by its own type
# (`Build::clean_queue@00620b60:75`-`:77`), a technology's at its TypeIndex. Written Word (551, `num_queued[501]`):
#   603/606: a Small City `0/2007` at (40, 160) and a second Library `0/2008` in it; `0/2005`, the start's, stays first.
#   620: Written Word at `0/2005`.
#   640: `die 2005,0` -- the first Library closes with it queued: the hand-off to `0/2008`.
#   680: `die 2008,0` -- the last Library closes: the refund, `num_queued[501]` 1 -> 0.
#   700/720: a Library at the old site (slot 2005 again, free since 670) and Written Word queued at it -- the order
#        this crate refused for good while the count stood at 1.
# Walked on ch122's start (RON_STAGE=ch122:chapter125:760, RON_STAGE_ALL): built, `0/2008` holds the entry on 640,
# the 680 close refunds 120 and 50, `0/2005` holds it on 721; with the fix off, `0/2008` is empty on 640 (the close
# refunded) and nothing queues on 720.
# Killers, before the take (the journal's list; a cheat on trace frame F shows on block F+2):
#   (1) the cover=1 twin enters no `Build::close` on trace frame 640 -- the cheat never closed the Library;
#   (2) `0/2008` reads `queued 0` on 642 -- the original refunds rather than hands off, and T71-7's reading is wrong;
#   (3) `num_queued[501]` reads anything but 1 on 642 or anything but 0 on 682 -- the hand-off moves a count, or the
#       refund does not take it off the technology;
#   (4) `0/2005` reads `queued 0` on 722 -- the original refuses the re-queue too.
#   run1540 (cover=0), run1540b (cover=1), both with the golden start's lobby pinned:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch125 \
#       --map 14 --end-frame 760 --log-window 610 760 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=9,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --ai-tribe 11 --profile DIFFICULTY=0 --profile STARTING_RESOURCES=1 \
#       --profile STARTING_TECHNOLOGY=0 --profile ENDING_TECHNOLOGY=7 \
#       --cmd-file tools/gamelog/golden/chapter125.cmd
0 !ai off
601 resource who=0 all +3000
603 add small_city who=0 40,160
606 add library who=0 34,152
620 @queueup 0 551 1 2005
640 die 2005,0
680 die 2008,0
700 add library who=0 10,158
720 @queueup 0 551 1 2005
