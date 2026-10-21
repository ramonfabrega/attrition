# Chapter thirteen's lobby, plus one `@comeout` line (item 3425,
# docs/GOLDEN.md §145). A coverage staging, not a chapter: no word is pinned
# on it, and the walk skips the line by name (parked 3425a).
#
# `chapter13.cmd`'s seven lines verbatim, then `800 @comeout 0 7`:
# `CommandManager::issue_come_out@00942c90(unit 0/7)`, the manager method
# nothing in the image calls (the unit panel's button goes through its twin,
# `UnitOut::issue_come_out@00607be0`). `0/7` is the Hoplites, garrisoned in
# the Barracks `0/2007` since 640: on run208's block 801 it is the chain's
# bottom (`inside_up 6`, `inside_down 8`), off the map, its captain's `flags
# 1` — active, as both the DLL's refusal 3 and `process_come_out@009465d0`
# ask — with uid 15. The 900 eject is kept, and finds the Chariot `0/6` alone.
#
# run1030, at `cover=1` on the click-free pool, chapter thirteen's recipe:
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch13co \
#       --map 14 --end-frame 1000 --log-window 605 1000 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=2,BUILDS=7,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter13-comeout.cmd --cover cover=1
#
# WHAT WOULD FALSIFY IT, read before the run. (1) The `I_ISSUE` on 800 is
# refused, or the package grows by anything but 11 (the command, no
# `group`). (2) The dump's COMMANDMANAGER line on block 801 is not
# `process_come_out` with `o` 7. (3) The trace does not enter
# `process_come_out@009465d0` on 800. (4) `0/7` is not on the map, with
# `inside_up -1`, by block 802. (5) `rngcmp.py` against run208's twin
# differs on a frame before 800.

0 !ai off

606 add barracks who=0 14,74

610 add chariot who=0 20,60
614 add hoplite who=0 10,90

620 @garrison 0 2007 0 6
640 @garrison 0 2007 0 7

800 @comeout 0 7

900 @eject 0 2007
