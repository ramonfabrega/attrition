# Golden record, chapter ninety-eight -- a late unit crosses a released disc (item 3223, GOLDEN 129; parked 3204, 3190).
# Chapter 96's staging (both dead Tanks' records end by 1235, their discs, `blocks_while_dead`, released by
# `Sim::cull_deaths`), and two who=0 hoplites added at 1000 and (squads 0/17..0/19 and 0/20..0/22; their captains 0/17 and 0/20 ordered, a follower is refused) at 1002 to walk west across the first dead
# Tank's cell (it died at about (13949,16075), tile 72,83): the first capture where a unit paths over a released disc.
#   RON_STAGE=ch96:chapter98:1500
#
#
0 !ai off
600 library who=0 2
602 library who=1 2
606 add barracks who=1 80,80
608 add commando who=0 70,82
609 add commando who=0 72,88
616 add tank who=1 72,83
617 add tank who=1 74,89
618 add hoplite who=0 76,86
618 add hoplite who=0 78,84
619 add hoplite who=0 77,90
620 @spell 0 640 6 1 13824 15936 6
621 @spell 0 640 7 1 14208 17088 7
1000 add hoplite who=0 74,84
1000 add hoplite who=0 75,84
1002 @move 0 13536 16032 17
1002 @move 0 13536 16008 20
