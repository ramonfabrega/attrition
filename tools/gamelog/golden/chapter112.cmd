# Golden record, chapter one hundred and twelve -- a city's gather-all (item 3915, GOLDEN 162; AI 391.2 row 8, parked 3906c).
# The golden start's City `0/2000`, whose `city_down` chain is its Woodcutter's Camp `0/2001` (4224,28608), three Farms
# `0/2002`..`0/2004` and `0/2005`, `0/2006` (types 435, 436), is selected and the key bound to OPTION_VILLAGE_GATHER_ALL
# pressed: `Options::do_city_gather@0071cdb0` (a building selection, `COUNT_GATHER_POINT` and `COUNT_TYPE` 0x19e non-zero)
# calls `GroupOut::issue_city_gather@0070ac70` (a `group` and a `city_gather`, 0x12, t -1, queued 2); the pump runs
# `CommandPackage::process_city_gather@00948a10` -> `Group::action_city_gather@00701780` on 611: the City's list cleared,
# then down the chain every building in use that `is` a Farm, Woodcutter or Mine (0x1a1..0x1a3, the listing's three
# pushes) gives the City a point at its own position, action 1, QUEUE_LAST. Two Citizens queued at the City on 616 are
# born under those points.
# Falsifiers, before the take: (1) trace 610: `INFO 22` with KEYMAP_NONE (the entry refused the selection); (2) no
# `process_city_gather t: -1` on 611 on the CM channel (and with it the assumption that the capital counts as VILLAGE);
# (3) block 612: `0/2000`'s gather list not exactly four points, (4224,28608) first, then the three Farms' in chain
# order, each action 1 -- in particular a point on `0/2005` or `0/2006` (AI 391.2's "any building answering 0" reading);
# (4) the cover=1 twin: no HIT on `Group::action_city_gather@00701780` or `GroupOut::issue_city_gather@0070ac70`.
#   run1509 (cover=0), run1509b (cover=1, `--cover cover=1 --no-ffwd`):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch112 \
#       --map 14 --end-frame 1300 --log-window 605 1300 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --bindkey OPTION_VILLAGE_GATHER_ALL=124 \
#       --cmd-file tools/gamelog/golden/chapter112.cmd
# bindkey OPTION_VILLAGE_GATHER_ALL=124
0 !ai off
606 select 2000 who=0
610 @key 124
616 @queueup 0 50 2 2000
