# Golden record, chapter ninety-six -- the death object's cull (item 3191, GOLDEN 126).
# Chapter 95's staging, run to 1500 so both dead Tanks' records end (152 frames after each death, parked 3189),
# and two who=0 hoplites added at 1000 beside the first dead Tank's cell, to cross the disc once it is released.
#   RON_STAGE=ch76:chapter96:1500
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
