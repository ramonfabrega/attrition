# Golden record, chapter thirty-one — the gather point's other arms: a lone
# unit under a ground point, a squad under a point on a unit, a list of two
# points, and a citizen's gather and build arms.
#
# docs/GOLDEN.md §40 (item 955; docs/DECISIONS.md 41 §1 and §5, 49).
# Chapter thirty's cast to 614 (two who=0 Barracks `0/2007` and `0/2008`, a
# Chariot `0/6` and a Hoplite squad `0/7`..`0/9`, `!ai off`), one Chariot
# more, then the player's rally points, production and one build:
#    615 `add chariot who=0 10,80`            `0/10`, idle at (2040, 15480),
#                                             1,427 south-west of 2007
#    616 `@gatherpoint 0 4608 31104 0 2000`   the City's point on open
#                                             ground (tile 24,162)
#    618 `@queueup 0 50 3 2000`               three Citizens (type 50)
#    620 `@gatherpoint 0 2040 15480 1 2007`   2007's point on the Chariot
#                                             `0/10` (a click on a friendly
#                                             unit: its point and 1)
#    622 `@queueup 0 132 1 2007`              Hoplites (type 132) at 2007
#    700 `@gatherpoint 0 5184 12672 0 2008`   2008's first point (27,66)
#    702 `@gatherpointadd 0 4992 11136 0 2008` its second, appended (26,58)
#    710 `@queueup 0 132 1 2008`              Hoplites at 2008
#    740 `@gatherpoint 0 4224 28608 1 2000`   the City's point on its
#                                             Woodcutter `0/2001`, between
#                                             the first Citizen and the second
#    790 `@build 0 7296 34176 521 11`         a Lookout (521) site, built by
#                                             the first Citizen `0/11`
#    840 `@gatherpoint 0 7296 34176 1 2000`   the City's point on that site,
#                                             between the second and third
# `@gatherpointadd` is the DLL's verb 20 with `add_to_end` 1 (item 955),
# the shift-click's append; `@gatherpoint`, `@queueup` and `@build` are
# chapters thirty's, twenty-four's and eighteen's.
#
#   run338 (item 955):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch31 \
#       --map 14 --end-frame 1400 --log-window 605 1400 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter31.cmd
#
# The window is 795 blocks. The last staged event is the Lookout's finish,
# near 1150 by this crate; 1400 leaves 250 blocks past it.
#
# ---------------------------------------------------------------------------
# UNDER THE EMULATOR FIRST (item 955, step 1; a scratch script on
# tools/emu/callfn.py's machine, out of git). Build::add_gather_point@
# 00622e70, Build::clear_gather@00623180, BuildData::gather_inside@0046f180,
# get_first_gather@0046f140 and num_gather@0046f0a0 unchanged; malloc, free
# and memcpy adapted; the Airbase arm's unit calls hooked:
# - QUEUE_LAST appends at the tail and the head stays the first point
#   (LinkListBase::add@00470c60 pushes in front, and add_gather_point then
#   steps the head on by one): three presses give [p1, p2, p3], and
#   get_first_gather and gather_inside read p1. The save walk
#   (PtrLinkListAbstract::walk_data@004708a0) writes them tail first.
# - An "inside" head with a point appended reads inside; a point with an
#   "inside" point appended does not (action_gather_point keeps the first
#   from happening: an inside head turns add_to_end off).
# - The Airbase arm (build_masks & 8 and is(0x1bf)): every one of who's
#   air units homed at the base is re-ordered on every press, from the
#   whole list: the head's add_air_patrol_order(x, y, base, who, 1, ·)
#   (its QueuePos is the list's head node, a heap address), each later
#   point appended to the patrol's x/y arrays, an action-3 point an
#   add_strafe_order(o, who, base, who, 1, QUEUE_NEW, 0). QUEUE_NEW's
#   clear_gather sends a flying plane home (add_strafe_order(-1, -1, base,
#   who, 0, QUEUE_NEW, 0)). Not staged here: every air chapter stands at
#   library 6, and this cast is Ancient.
# What the emulator did not reach: Unit::come_out@00617c10's routing, which
# reads the world, the unit and the list at process time. The listing
# settled its registers (docs/GOLDEN.md §40).
#
# ---------------------------------------------------------------------------
# THE STAGING, walked by this crate through the commands' own entries on
# run312's start (the same game to 605), @gatherpointadd through
# input::group_gather_point with add_to_end. No staging run: run339 was not
# used. The frames and points are this crate's:
# - 617, 621, 701, 703, 741, 841: the presses; 791 the build.
# - 718: the Citizen `0/11` out at (4104, 31032), a MOVEORDER to (4632,
#   31128); 790 builds the Lookout `0/2009` at (7296, 34176), unfinished
#   to ~1150.
# - 824: the Citizen `0/12` out at (3576, 29928), a MOVEORDER to (4104,
#   28824) and a GATHERORDER on `0/2001`.
# - 858: the Hoplites `0/13`..`0/15` out at (2472, 14856), (2472, 15000),
#   (2328, 15000), each a GROUPATTACKTOORDER toward the Chariot's side.
# - 938: the Citizen `0/16` out at (3912, 31416), a MOVEORDER to (7032,
#   34056) and a BUILDORDER on `0/2009`.
# - 953: the Hoplites `0/17`..`0/19` out at (4584, 13656), (4584, 13800),
#   (4680, 13512), each one GROUPATTACKTOORDER toward (5208, 12696).
# The tiles, by this crate's TData mask: (24,162) 0x100, (27,66) and
# (26,58) 0, the Chariot's (10,80) 0; none 0x20, which find_nearby_spot
# refuses.
0 !ai off
606 add barracks who=0 14,74
608 add barracks who=0 22,74
610 add chariot who=0 20,60
614 add hoplite who=0 10,90
615 add chariot who=0 10,80
616 @gatherpoint 0 4608 31104 0 2000
618 @queueup 0 50 3 2000
620 @gatherpoint 0 2040 15480 1 2007
622 @queueup 0 132 1 2007
700 @gatherpoint 0 5184 12672 0 2008
702 @gatherpointadd 0 4992 11136 0 2008
710 @queueup 0 132 1 2008
740 @gatherpoint 0 4224 28608 1 2000
790 @build 0 7296 34176 521 11
840 @gatherpoint 0 7296 34176 1 2000
