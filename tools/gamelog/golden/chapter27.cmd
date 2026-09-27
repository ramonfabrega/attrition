# Golden record, chapter twenty-seven — the upgrade line: a unit upgrade
# researched at who=0's Barracks through the player's command, pressed
# again while it researches, the old type queued behind it, the finish
# that converts the standing squad and the queued entry, and the two
# presses after it (the old type, and the new one as a train job).
#
# docs/GOLDEN.md §36 (item 901; docs/DECISIONS.md 41 §1 and §5, 49).
# who=0 is Nubian (tribe 4); the line is Slingers (82) -> Javelineers
# (83), food and timber, no graft. Chapter thirteen's cast to 619 with a
# Slinger squad beside it; the Classical age and The Art of War are staged
# by two cheats before the window, each a whole `gain_tech`
# (docs/INPUT.md §11.11). Then:
#    620 `@queueup 0 83 2 2007`  Javelineers on the idle Barracks, num 2:
#                                the research arm, once (arm a)
#    640 `@queueup 0 83 1 2007`  the same while it researches (arm b)
#    650 `@queueup 0 82 1 2007`  Slingers behind it: a train job of the
#                                old type (arm c)
#   1000 `@queueup 0 82 1 2007`  Slingers after the gain (arm e)
#   1010 `@queueup 0 83 1 2007`  Javelineers after the gain: a train job
#                                now, not a research (arm f)
# The gain (arm d) is the research's own finish: `Build::finished` ->
# `Leader::gain_tech(83, x, y, 1, 1)`.
#
#   run300 (item 901):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch27 \
#       --map 14 --end-frame 1560 --log-window 605 1560 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter27.cmd
#
# The window is 955 blocks. The last falsifier is the second Javelineers
# squad's birth, 1307 by this crate's walk; 1560 leaves 253 blocks past it.
#
# ---------------------------------------------------------------------------
# THE FINISH, under the emulator before the capture (item 901, step 1):
# `Leader::gain_tech`'s unit arm, 6dd999..6e05d9, entered with a
# synthesized frame; the object loop, the queue loop (the real
# `BuildQueueData::get_queue` and `BuildQueue::set_queue`) and the bits
# loop run as shipped, every virtual slot and `track_queued` stubbed and
# recorded.
# - every active unit of the player whose type is `get_graft(t.from)`, or
#   whose `jump` chain reaches `t`, gets `set_type(t, 0)`, captain and
#   member alike; an inactive one is skipped;
# - every entry of every active building's queue that is such a unit type
#   is re-targeted in place, `set_queue(i, t, NULL, 1)`: the type written,
#   the progress and the recorded price kept, nothing paid or refunded,
#   between `track_queued(q, -1)` and `track_queued(t, +1)`;
# - an entry of `t` itself (the finishing research, or the same upgrade
#   queued at a second building) and an entry above `t` are left;
# - `from` and every type whose chain reaches `t` get the `tech` and
#   `obs_flags` bits.
# - a match by the `jump` chain decrements `t`, not the entry's type: the
#   register was overwritten by the walk (`6ddcf4`, `push esi`).
#
# ---------------------------------------------------------------------------
# THE STAGING, walked by this crate through the command's own entry
# (`input::group_queue_up`) on run296's start, which is this game to 599,
# with a prototype of the queue loop for the values it writes (item 901;
# run301 was not used). Every value is this crate's, on the block it is
# visible:
# - 605: ages 2, epochs 1, epoch[0] 1, discovered 8; knowledge and metal
#   100 (the Classical age's grant).
# - 617: the Slinger squad `0/10` (and its two figures).
# - 622: [83 at 100], 80 food 80 timber, once; food 174, timber 161.
# - 642: unchanged (food 176 and timber 162 would pay a second: the gate,
#   not the price).
# - 652: [83, 82 at 0], the Slingers 46 food 46 timber.
# - 922: Javelineers gained: discovered 9; `0/10`..`0/12` Javelineers;
#   [83 at 100] with the Slingers' 46/46 kept; nothing refunded.
# - 1002: unchanged (Slingers are obsolete).
# - 1012: [83, 83 at 0], the second at 46 food 46 timber (the train
#   price, not the research's 80/80).
# - 1111: a Javelineers squad out of the Barracks.
# - 1307: the second Javelineers squad out.
0 !ai off
600 age who=0 2
602 military who=0 1
606 add barracks who=0 14,74
610 add chariot who=0 20,60
614 add hoplite who=0 10,90
616 add slinger who=0 12,86
620 @queueup 0 83 2 2007
640 @queueup 0 83 1 2007
650 @queueup 0 82 1 2007
1000 @queueup 0 82 1 2007
1010 @queueup 0 83 1 2007
