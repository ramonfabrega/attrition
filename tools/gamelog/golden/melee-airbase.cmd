# The fifth melee -- the fourth's lobby with an Airbase a computer (item 3868, GOLDEN 156; the shape is GOLDEN 155
# as the sixty-sixth pass amended it: a chapter that moves no lever stages the trainer its lines need; the staging is
# GOLDEN 154.3's Proposal A without the third seat). melee-sea.cmd's lobby and lines, run1482's start: --profile
# PLAYERS=2 STARTING_TECHNOLOGY=7 ENDING_TECHNOLOGY=7 STARTING_RESOURCES=7, --seat-tribe 1=10 --seat-tribe 2=13, map 18,
# seed 12345; who 0 the idle human (Nubians, 4), who 1 the French (10), who 2 the Russians (13).
# The two lines this script adds, before the first unit `add` at 610: an Airbase (building type 33, 5x9 tiles) for
# each computer, each centred on a tile inside its own capital's city -- `find_city_at` answers the capital -- whose
# site `blocked_site` calls clear at frame 603 on the harness's staged walk of run1482 (the AI has built over the
# nearer frame-0 sites by then), with every centre of the 5x5 around it clear too. Who 1's capital is at tile
# (96, 212), who 2's at (212, 96). Inside a city, `Wall::activate`'s call takes `Wall::increment_stats`' active arm,
# which files the Airbase in `reg_buildings` and `mil_trainers` (AI.md 29), so the computers' make lists may buy
# the Jet Fighter (MIRAGE, 296) and the Attack Helicopter (HIND, 312) GOLDEN 154 says each owns from frame 0.
600 resource who=1 all +100000
602 resource who=2 all +100000
603 add airbase who=1 95,193
605 add airbase who=2 220,107
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
