# The third melee -- a held Information age on Great Sahara, every seat a computer (item 3797, GOLDEN 152; the shape is GOLDEN 155).
# Lobby: map 7, --profile PLAYERS=2 STARTING_TECHNOLOGY=7 ENDING_TECHNOLOGY=7 STARTING_RESOURCES=7, --seat-tribe 1=10
# --seat-tribe 2=13; who 0 the idle human (Nubians, tribe 4), who 1 the French (10), who 2 the Russians (13).
# The placement is the first two melees' (GOLDEN 149, 150): who 1 on tile row 76, who 2 on row 86, x 66..114; who 0's
# lines between, on row 81. No `!ai off`, no `select`, no issuer, no `tech all` (GOLDEN 149).
# The thirteen never-seen lines (`rubric.py --tables 3`): `parse_type@007e4050` compares `TypeData::name` (+0x60, the
# NAME column), exact then prefix, in TypeIndex order -- so ten are reachable by name and three are not, since each
# shares its NAME with an earlier generic type: Mirage 296 ("Jet Fighter", 295 first), HIND 312 ("Attack Helicopter",
# 311) and the Spanish Elite Special Forces 78 (77). Those three are staged by their TYPENAME as probes of that
# reading (falsifier (g), GOLDEN 152): the original should refuse each, where this crate's `type_named` asks the
# TYPENAME table first and would not.
600 resource who=1 all +100000
602 resource who=2 all +100000
610 add jet_fighter who=1 66,76
612 add jet_fighter who=2 66,86
614 add attack_helicopter who=1 70,76
616 add attack_helicopter who=2 70,86
618 add advanced_fighter who=1 74,76
620 add advanced_fighter who=2 74,86
622 add stealth_bomber who=1 78,76
624 add stealth_bomber who=2 78,86
626 add jet_fighter_bomber who=1 82,76
628 add jet_fighter_bomber who=2 82,86
630 add cruise_missile who=1 86,76
632 add cruise_missile who=2 86,86
634 add anti-aircraft_missile who=1 90,76
636 add anti-aircraft_missile who=2 90,86
638 add main_battle_tank who=1 94,76
640 add main_battle_tank who=2 94,86
642 add anti-tank_missile who=1 98,76
644 add anti-tank_missile who=2 98,86
646 add advanced_machine_gun who=1 102,76
648 add advanced_machine_gun who=2 102,86
650 add elite_special_forces who=1 106,76
652 add elite_special_forces who=2 106,86
654 add rocket_artillery who=1 110,76
656 add rocket_artillery who=2 110,86
658 add general who=1 114,76
660 add general who=2 114,86
662 add mirage who=1 62,76
664 add hind who=2 62,86
666 add eagle_fighter who=0 74,81
668 add fighter_bomber who=0 78,81
670 add v2_rocket who=0 82,81
672 add partisan who=0 86,81
674 add mg42 who=0 90,81
676 add recoilless_gun who=0 94,81
678 add elite_special_forces_spanish who=0 98,81
