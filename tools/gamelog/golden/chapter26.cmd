# Golden record, chapter twenty-six — the research line: a technology
# queued at who=0's Library through the player's command, a second press
# while it researches, one behind it on a busy Library, a press of a
# technology already held, a Science epoch that re-prices the entry behind
# it, a cancel of that entry, and the same technology queued again.
#
# docs/GOLDEN.md §35 (item 883; docs/DECISIONS.md 41 §1 and §5, 49).
# Chapter thirteen's cast to 619 (a Barracks `0/2007` for who=0, a Chariot
# and a Hoplite squad, `!ai off`); who=0's own Library is `0/2005`, in its
# city from the start. Then:
#    620 `@queueup 0 572 2 2005`  The Art of War (572), num 2, on the idle
#                                 Library: the research arm, once (arm a)
#    640 `@queueup 0 572 1 2005`  the same while it researches (arm b)
#    650 `@queueup 0 551 1 2005`  Written Word (551) on the busy Library:
#                                 the second pass, behind it (arm c)
#    850 `@queueup 0 572 1 2005`  The Art of War again, held (arm d)
#    860 `@queueup 0 558 1 2005`  Barter (558) behind Written Word
#    870 `@queueup 0 132 1 2007`  Hoplites at the Barracks: a unit beside it
#   1040 `@unqueue 0 0 2005`      cancel Barter, re-priced, in progress (arm f)
#   1060 `@queueup 0 558 1 2005`  Barter again (arm g)
# Written Word's gain re-prices Barter in place (arm e). `@queueup` and
# `@unqueue` are chapter twenty-four's and twenty-five's verbs 18 and 19;
# the DLL passes the type through, so a technology needs no new verb.
#
#   run296 (item 883):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch26 \
#       --map 14 --end-frame 1492 --log-window 605 1492 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter26.cmd
#
# The window is 887 blocks. The last falsifier is Barter's second finish,
# 1242 by this crate; 1492 leaves 250 blocks past it.
#
# ---------------------------------------------------------------------------
# THE RESEARCH ARM, under the emulator before the capture (item 883,
# step 1; `Build::queue_up` stubbed, every call recorded):
# - `LeaderData::researching(t, -1, 0, 0)` first: a tech queued at any of
#   the player's active buildings, in the group or not, ends the command
#   with no call.
# - then two passes over the members sorted by `queued`: the first offers
#   the job only to a member whose own queue is empty, the second to any;
#   each live, finished member gets `queue_up(t, 1)`, a refusal goes on,
#   and the first acceptance ends the command. `num` is never read (2 and
#   0 each lay one entry).
# - a unit type with its bit clear takes the same arm, once; with its bit
#   set, every member gets it `num` times.
# The arm itself writes only the group's `+0x28` (`action_begin`).
#
# ---------------------------------------------------------------------------
# THE STAGING, walked by this crate through the command's own entry
# (`input::group_queue_up`, `input::unqueue`) on run292's start, which is
# this game to 619. No staging run: run297 was not used. Every value is
# this crate's, on the block it is visible:
# - 622: [572 at 100], 120 food; food 134, timber 241, wealth 113.
# - 642: unchanged, one entry (food 136 >= 120: not the price).
# - 652: [572 at 3200, 551 at 0], 120 timber 50 wealth; timber 123.
# - 822: The Art of War gained: epochs 1, Military 1, discovered 2 (the
#   Bark and the Trireme); [551 at 0].
# - 852: unchanged (food 155 >= 120: the bit, not the price).
# - 862: [551 at 4000, 558 at 0], 60 food 60 timber.
# - 872: the Barracks [132 at 100], 51 food 38 timber; food 45, timber 39.
# - 1023: Written Word gained: epochs 2, Science 1, discovered 3
#   (Boadicea); [558 at 0] re-priced 54/54, food and timber +6.
# - 1042: [] ; food and timber +54 (the re-priced record).
# - 1062: [558 at 100], 54 food 54 timber (the science discount).
# - 1106: the Hoplites out.
# - 1242: Barter gained: epochs 3, Commerce 1; [].
0 !ai off
606 add barracks who=0 14,74
610 add chariot who=0 20,60
614 add hoplite who=0 10,90
620 @queueup 0 572 2 2005
640 @queueup 0 572 1 2005
650 @queueup 0 551 1 2005
850 @queueup 0 572 1 2005
860 @queueup 0 558 1 2005
870 @queueup 0 132 1 2007
1040 @unqueue 0 0 2005
1060 @queueup 0 558 1 2005
