# Golden record, chapter seventy-four — `finish` on a finished building with
# a queue: the Library's research is paid its train time (item 2500,
# `docs/GOLDEN.md` §91).
#
# THE GATE. `select <o>` then `finish` is `run_cmd` case 0x51,
# `Console::hurry_action@007c3810` with `this != 0` (listing `7c3a33`:
# `cmpl $0, this; movl $10000; cmovne train_time`): a selected building that
# holds a queue pays the head's train time to the head (a library's queue is
# the owner's first library's), then `job_counter += 10000`, then
# `do_queue(0)`. The `this == 0` form (10000 to the head) is the keymap's
# (`exec_keymap` case 0x45, `cheat_keys`); no command reaches it. The unit
# arm (`is(0x15f)`, an Aircraft Carrier) wants a ship on water and is not
# staged: parked 2501.
#
# THE STAGING is chapter seventy-three's with the wonder and the city taken
# out: the Library `0/2005` researches The Art of War from 640 (a control
# with no `finish` completes it on 841); `700 select 2005`, `701 finish`.
#
# WHAT WOULD FALSIFY IT.
# 1. The research still queued on block 704 in the dump: the arm adds
#    nothing a human's Library queue (or `do_queue(0)` does not run at once).
# 2. Done at 841 anyway or a frame other than 701/702: the added amount is
#    not the train time (10000 would finish it as well: the train time of
#    the entry is ~ 200 frames' worth, so the two parts differ only in
#    where the leftover goes; read the entry's `job_counter` on 701).
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch74 \
#       --map 14 --end-frame 760 --log-window 600 760 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=5,DEATHS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter74.cmd
#
0 !ai off
600 age who=0 8
601 resource who=0 all +50000
640 @queueup 0 572 1 2005
700 select 2005 who=0
701 finish
