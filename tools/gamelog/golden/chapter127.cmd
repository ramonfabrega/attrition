# Golden record, chapter one hundred and twenty-seven -- three spells with every spy upgrade held (item 3985, GOLDEN 178;
# the territory adjudication 3's rows 47 and 49). Chapter eighty-eight's lobby (run871's start, both sides Medieval).
# who=0 is granted Tactics, Operations and Strategy, the prerequisites of SPIES_UPGRADE_1..3, so
# `LeaderData::get_spy_upgrade@006e1090` answers 3 and `SpellTypeData::get_range@00676a80` adds
# 3 x SPY_BRIBE_UPGRADE_RANGE (2) x 192 to a Bribe and 3 x SPY_INFORMER_UPGRADE_RANGE (3) x 192 to an Informer.
#   Arm 1: Spy `0/6` Bribes a who=1 Hoplite 28 half-tiles off; it walks to 12 half-tiles (+ the Hoplite's radius), not 6.
#   Arm 2: Spy `0/7` casts the Informer on a who=1 Barracks 30 half-tiles off; the halving `(r + 1) / 2` on a building
#          comes after the upgrade term, so it stops at 9.5 half-tiles (+ the footprint), not 5.
#   Arm 3: Spy `0/8` Bribes a who=1 Catapult 20 half-tiles off. A siege target (type vslot +0x10c,
#          `UnitTypeData::is_siege`) takes one tile, 192, in place of the whole sum, so it walks to 1 half-tile
#          (+ the radius), not 12 -- and not 6, the row alone.
# The booking's third arm, `cast_bribe`'s CEO refusal refund, is not staged: `Unit::do_cast@005ebfe0:422` asks
# `is_valid_target` on every frame of the hold, the land frame included, and Bribe's arm there opens with the same
# `has_general(0, 0x165)` and own-target refusals, so `cast_bribe`'s copies refund nothing a unit's cast can reach.
# Blow Tread and Restock Supplies, the booking's other two arms, are castable by no unit on shipped data (FROM/FROM2
# `none`, PREQ0 `Disable`), so they are not staged (the journal, and GOLDEN 178).
# Killers, before the take: (1) arm 1's Spy casting from beyond 12 half-tiles plus the radius, or stopping at 6 --
# the upgrade term is not in the original's walk-in; (2) arm 2's stopping at 5 + the footprint -- the halving is of the
# base; (3) arm 3's stopping beyond 1 half-tile + the radius -- no siege collapse; (4) the cover=1 twin not entering `get_range` or `get_spy_upgrade`.
#   run1543 (cover=0), run1543b (cover=1, `--cover cover=1 --no-ffwd --twin ~/ron-golden/ch127`):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch127 \
#       --map 14 --end-frame 1600 --log-window 1095 1600 --timeout 3600 \
#       --detail end:UNITS=3,DEATHS=1,LEADERS=2,BUILDS=7 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter127.cmd
0 !ai off
600 library who=0 2
602 library who=1 2
603 tech who=0 tactics on
604 tech who=0 operations on
605 tech who=0 strategy on
608 add spy who=0 100,100
609 add spy who=0 100,140
610 add spy who=0 190,60
1100 add hoplite who=1 128,100
1101 add barracks who=1 130,140
1102 add catapult who=1 210,60
1112 @spell 0 629 6 1 24576 19200 6
1112 @spell 0 639 2006 1 24960 26880 7
1112 @spell 0 629 9 1 40320 11520 8
