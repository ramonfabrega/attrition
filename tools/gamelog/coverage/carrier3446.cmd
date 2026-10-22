# Item 3446: the coverage pair's lobby (East Indies, human Nubians against the Persians, All Technologies, Toughest;
# run651/run652's game) on the click-free pool, AI on, the carrier staged by console and `@` lines (docs/GOLDEN.md §147).
# Cells are the dump's WORLD order (row-major in y; the 40 whales of run651 all sit on OCEAN that way): the human's
# coast is cell x=3 beside the open sea x=0..2, and the sea at cells (30..34, 40..47) is the Persians' nearest.
# `add` takes quarter cells (192 internal units).
#   600..604  the human's buckets, a Dock 0/2007 on the coast, an empty Aircraft Carrier 0/14 on the sea
#   606, 608  a who=1 Hoplite squad 1/15..17 and a General 1/18 beside the Persians' sea
#   609       a who=1 Barracks beside the carrier, the plane's target (1/2025)
#   610       the Dock trains a Carrier (351): `Build::train` -> `Unit::action_unqueue(0)` on the new carrier
#   612, 614  BUILDAIR on 0/14 by its craft's type, FIGHTERBOMBER 308 (`Options::do_spell` turns the plane's type into
#             BUILDAIR 661 only to find the caster; the wire carries 308): `num_queued` 1, then 2
#   620       the carrier's cancel, `@unqueueunit` (issue_unqueue's UnitOut overload): `Unit::action_unqueue(1)`, 2 -> 1
#   622..626  as who=1, the squad and the General walked into the sea: a soldier boat and an idle one
#   830       the plane 0/15 (trained on 817) strikes the Barracks; it lands on 0/14: `UnitData::has_repeat_air`
# The first unshared input is 600; `samegame.py --from 600` against run672, `rngcmp.py` against run652.
600 resource who=0 all +100000
602 add dock who=0 13,37
604 add aircraft_carrier who=0 6,37
606 add hoplite who=1 146,178
608 add general who=1 146,182
609 add barracks who=1 20,44
610 @queueup 0 351 1 2007
612 @spell 0 308 -1 -1 0 0 14
614 @spell 0 308 -1 -1 0 0 14
620 @unqueueunit 0 14
622 be 1
622 @move 1 24960 34176 15
624 @move 1 24960 35712 18
626 be 0
830 @strike 0 2025 1 15
