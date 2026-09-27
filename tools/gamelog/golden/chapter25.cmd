# Golden record, chapter twenty-five — the cancel line: a Barracks' queue
# cancelled inside a run, across two types, on an infinite queue, and from
# the end.
#
# docs/GOLDEN.md §34 (item 884; docs/DECISIONS.md 41 §1 and §5, 49).
# Chapter thirteen's cast to 619 (a Barracks `0/2007` for who=0, a Chariot
# and a Hoplite squad, `!ai off`), then the player's production:
#    620 `@queueup 0 132 2 2007`  two Hoplites (type 132): a run of one type
#    640 `@queueup 0 170 1 2007`  Bowmen (type 170) behind them
#    700 `@unqueue 0 0 2007`      cancel slot 0, the head in progress: arm a
#    760 `@unqueue 0 0 2007`      slot 0 again, now [132*, 170]: arm b
#    800 `@buildmask 0 64 2007`   the infinite-queue button over [170*]
#    840 `@unqueue 0 -1 2007`     a single cancel on 4160: arm c
#    980 `@queueup 0 132 1 2007`  a Hoplite, then
#    990 `@queueup 0 170 1 2007`  Bowmen behind it
#   1000 `@unqueue 0 -1 2007`     −1 on [132*, 170]: arm d
# `@unqueue <who> <p> <b>…` is the DLL's call of `CommandManager::
# issue_unqueue@00942c40(b, p)` (verb 19, new here), once per building;
# `@queueup` and `@buildmask` are chapter twenty-four's verbs 18 and 17.
#
#   run292 (item 884):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch25 \
#       --map 14 --end-frame 1466 --log-window 605 1466 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter25.cmd
#
# The window is 861 blocks. The last falsifier is the Hoplite's finish
# behind arm d, 1216 by this crate; 1466 leaves 250 blocks past it.
#
# ---------------------------------------------------------------------------
# THE CANCEL, under the emulator before the capture (item 884, step 1).
#
# - `issue_unqueue(b, p)` appends 15 bytes, `30 [who][o][p][uid]`, and no
#   `group`; −1, −5 and −10 are carried as passed.
# - `Build::unqueue@006207c0(i, 1)` on a synthesized Barracks:
#   [132*, 132, 170] at i 0 removes slot 1 (the walk over the run), the
#   head's 5000 kept, the removed entry's recorded pairs refunded;
#   [132*, 170] at 0 removes the head and the Bowmen move up at 0;
#   `unqueue(0, 0)` (completion) removes the head with no walk and no
#   refund; `queued` reaching 0 clears 0x40.
# - `Build::action_unqueue@00620280(p)`: `p` a slot is `unqueue(p, 1)`; a
#   negative above −5 is `unqueue(queued − 1, 1)`; −5..−9 five from the
#   end; −10 and below all. With 0x40 set the bit is cleared first, and
#   for p ≥ −1 nothing is removed; −2 clears it and removes the last. An
#   empty queue is left alone, its bit too.
# Each removal writes `queued`, the entry's `job_counter` (0), `num_queued`
# at `leader + 0x5a22`, the AI tallies at `leader + 0xa10..`, the stockpile
# at `leader + 0x6eb8` (the three recorded pairs), and `options->rebuild`.
#
# ---------------------------------------------------------------------------
# THE STAGING, walked by this crate through the commands' own entries
# (`input::group_queue_up`, `input::unqueue`, `input::group_buildmask`) on
# run285's start, which is this game to 640 with the second Hoplite added.
# No staging run: run293 was not used. Every value is this crate's:
# - 622: [132 at 100, 132 at 0], paid 51 food 38 timber, then 53 and 41.
# - 642: + 170, paid 46 timber 56 wealth.
# - 702: [132 at 8100, 170]; food +53, timber +41 (the run's second).
# - 762: [170 at 100]; food +51, timber +38 (the head's own price).
# - 802: 4160.
# - 842: 4096; [170 at 8100]; nothing refunded.
# - 965: the Bowmen out; [] and no re-queue.
# - 982: [132 at 100]; 992: [132, 170], wealth 66 -> 10.
# - 1002: [132 at 2100]; timber +46, wealth +56 (the last slot's).
# - 1216: the Hoplites out.
0 !ai off
606 add barracks who=0 14,74
610 add chariot who=0 20,60
614 add hoplite who=0 10,90
620 @queueup 0 132 2 2007
640 @queueup 0 170 1 2007
700 @unqueue 0 0 2007
760 @unqueue 0 0 2007
800 @buildmask 0 64 2007
840 @unqueue 0 -1 2007
980 @queueup 0 132 1 2007
990 @queueup 0 170 1 2007
1000 @unqueue 0 -1 2007
