# Golden record, chapter ninety-nine -- a Tank blown up by Sabotage keeps its disc (item 3228, GOLDEN 130; parked 3190).
# `Unit::close@0060ee50`'s `param_1` is the death type; `Object::close@00647160` keeps the occupancy disc of a
# `blocks_while_dead` type (`x`, only the Tank line carries it) for any nonzero one -- not only a combat death.
# Sabotage (0x280) on a vehicle is `cast_sabotage`'s `dtype` 3.
#   RON_STAGE=ch76:chapter99:1500
0 !ai off
600 library who=0 2
602 library who=1 2
606 add barracks who=1 80,80
608 add commando who=0 71,83
609 add commando who=0 73,83
610 add commando who=0 72,82
611 add commando who=0 72,84
616 add tank who=1 72,83
620 @spell 0 640 6 1 13824 15936 6
621 @spell 0 640 6 1 13824 15936 7
622 @spell 0 640 6 1 13824 15936 8
623 @spell 0 640 6 1 13824 15936 9
790 add hoplite who=0 74,84
791 add hoplite who=0 75,84
793 @move 0 13536 16032 10
793 @move 0 13536 16008 13
