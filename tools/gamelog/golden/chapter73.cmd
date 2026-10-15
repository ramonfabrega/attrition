# Golden record, chapter seventy-three — a wonder finished on a human: a
# Citizen starts a Supercollider in a founded city, the console's `finish`
# completes it, and the held wonder zeroes a research job's remaining time
# (item 2474, `docs/GOLDEN.md` §90; AI §216's wonder arm, diff-backed before
# only on the Mongol pair's AI Supercollider).
#
# THE GATE. `select <o>` then `finish` is `run_cmd` case 0x51,
# `Console::hurry_action@007c3810` with `this != 0`: an unfinished building
# takes `Build::activate(0, 1, 1)` through vslot 0x1a8, then `job_counter +=
# 10000`. `age who=0 8` alone (not `library`, which maxes the epochs and
# makes The Art of War unavailable) lifts the wonder's age; `tech who=0
# supercollider on` its prerequisite.
#
# THE STAGING, walked by this crate on run492's start (RON_STAGE=ch45:
# chapter73:900, RON_STAGE_ALL): a Small City `0/2007` at (40, 160), a
# Citizen `0/6` ordered to a Supercollider `0/2008` at (7776, 29472) — a
# site tile a probe found legal; (46, 160) is blocked by the city's own
# footprint and x >= 50 is water. The Library `0/2005` researches The Art of
# War from 640 (a control run, no `finish`, completes it on 841).
# - 700: `select 2008 who=0`, 701: `finish`: the wonder is active, held by
#   city 2007, and the research done on 701 on this crate's walk.
#
# WHAT WOULD FALSIFY IT. A call on trace frame F is on block F+2.
# 1. `0/2008` not active on 703 (or active with a `job_counter` other than
#    10000 plus the builder's share): `finish` is not `activate` + 10000.
# 2. The Art of War still queued past 705: the held wonder does not zero a
#    human's research time (`train_time`'s `has_wonder(0x21d)` arm).
# 3. The leader's `wonder_mark` and `num_buildings[127]` unchanged on 703.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch73 \
#       --map 14 --end-frame 760 --log-window 600 760 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=5,DEATHS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter73.cmd
#
0 !ai off
600 age who=0 8
601 resource who=0 all +50000
602 tech who=0 supercollider on
603 add small_city who=0 40,160
604 add citizen who=0 38,162
608 @build 0 7776 29472 541 6
640 @queueup 0 572 1 2005
700 select 2008 who=0
701 finish
