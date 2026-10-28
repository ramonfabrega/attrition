# The first melee -- a held Gunpowder age on Great Sahara, every seat a computer (item 3733, GOLDEN 149; the shape is GOLDEN 155).
# Lobby: map 7, --profile PLAYERS=2 STARTING_TECHNOLOGY=3 ENDING_TECHNOLOGY=3 STARTING_RESOURCES=7; who 0 the idle
# human (Nubians, tribe 4), who 1 the British (11, London), who 2 the Inca (2, Cuzco). run1412's start block reads
# `diplos` 0 between every pair of leaders on frame 0: all three at war, no `war` line.
# No `!ai off` (the AI is the point), no `select`, no issuer. The two `tech who=N all on` lines GOLDEN 155 names are
# NOT staged: `run_cmd`'s `all` arm gains every `is_tech_type` type, 0x220..0x274, and the seven ages are
# 0x220..0x226 -- it would lift the held age to Information (GOLDEN 149).
# Placement: the two territories do not touch on frame 0 (London's x 24-45 y 0-17, Cuzco's x 0-18 y 28-49, in
# cells); the squads stand in the open land between, who 1 on tile row 76 and who 2 on row 86, x 66..110.
# The age's never-seen lines are Nubian (Apedemak Archers 176, Camel Raider 193): they go to who 0, between.
600 resource who=1 all +100000
602 resource who=2 all +100000
610 add arquebusiers who=1 66,76
612 add arquebusiers who=2 66,86
614 add elite_pikemen who=1 70,76
616 add elite_pikemen who=2 70,86
618 add dragoon who=1 74,76
620 add dragoon who=2 74,86
622 add elite_light_cavalry who=1 78,76
624 add elite_light_cavalry who=2 78,86
626 add heavy_knight who=1 82,76
628 add heavy_knight who=2 82,86
630 add bombard who=1 86,76
632 add bombard who=2 86,86
634 add supply_wagon who=1 90,76
636 add supply_wagon who=2 90,86
638 add general who=1 94,76
640 add general who=2 94,86
642 add spy who=1 98,76
644 add spy who=2 98,86
646 add explorer who=1 102,76
648 add explorer who=2 102,86
650 add the_monarch who=1 106,76
652 add the_monarch who=2 106,86
654 add king's_yeomanry who=1 110,76
656 add inti_macemen who=2 110,86
658 add apedemak_archers who=0 86,81
660 add camel_raider who=0 90,81
