# The fourth melee -- a sea fight on East Indies at the held Information age, every seat a computer (item 3847,
# GOLDEN 153; the shape is GOLDEN 155, the lever moved the map). The third melee's lobby on map 18:
# --profile PLAYERS=2 STARTING_TECHNOLOGY=7 ENDING_TECHNOLOGY=7 STARTING_RESOURCES=7, --seat-tribe 1=10
# --seat-tribe 2=13; who 0 the idle human (Nubians, tribe 4), who 1 the French (10), who 2 the Russians (13).
# The placement is read off run1482's start dump, its first WORLD block (GOLDEN 153): who 1 holds region 9 to the
# south-west, who 2 region 5 to the east, and the strait between them is region 65's cells x 34..43, y 37..41, every
# one `is_ocean` (land SANDY or OCEAN, flags 0). Who 1's ships stand on cell row 41 (tile y 166), who 2's on row 38
# (tile y 154), tile x 138..174; the Minutemen on each side's clear shore land (cells (31, 42) and (47, 37)); who 0's
# lines on region 6, the unowned island north of the strait, cells (37..40, 33..35).
# `machine_gun_german` is a probe: 124's NAME is "Machine Gun", 123's, so `add`'s walk names 123 (GOLDEN 153).
600 resource who=1 all +100000
602 resource who=2 all +100000
610 add missile_cruiser who=1 138,166
612 add missile_cruiser who=2 138,154
614 add attack_submarine who=1 144,166
616 add attack_submarine who=2 144,154
618 add advanced_battleship who=1 150,166
620 add advanced_battleship who=2 150,154
622 add destroyer who=1 156,166
624 add destroyer who=2 156,154
626 add aircraft_carrier who=1 162,166
628 add aircraft_carrier who=2 162,154
630 add transport_freighter who=1 168,166
632 add transport_freighter who=2 168,154
634 add fluyt who=1 174,166
636 add fluyt who=2 174,154
638 add minuteman who=1 126,170
640 add minuteman who=2 190,150
642 add camel_corps who=0 158,142
644 add royal_kushite_archers who=0 150,142
646 add heavy_camel_archer who=0 158,138
648 add machine_gun_german who=0 150,134
