# The second melee -- a held Classical age on Great Sahara, every seat a computer (item 3774, GOLDEN 150; the shape is GOLDEN 155).
# Lobby: map 7, --profile PLAYERS=2 STARTING_TECHNOLOGY=1 ENDING_TECHNOLOGY=1 STARTING_RESOURCES=7, --seat-tribe 1=5
# --seat-tribe 2=7; who 0 the idle human (Nubians, tribe 4, Napata), who 1 the Greeks (5, Athens), who 2 the Egyptians
# (7, Thebes). run1449's capitals stand where run1412's did, so the first melee's placement holds (GOLDEN 149): who 1
# on tile row 76 and who 2 on row 86, x 66..110; who 0's lines between, on row 81.
# No `!ai off`, no `select`, no issuer, no `tech all` (it would lift the held age; GOLDEN 149).
# The age's nine never-seen lines (`rubric.py --tables 3`): each computer fields its own (the Companion, the Chariot),
# both field the Militia, and who 0 the rest -- its own Kushite Archers and Camel Archer, and the Royal Balamob
# Slingers, Barbarians, Bushi and Chevalier no seat owns: `parse_type` walks the names alone, so `add` does not
# test TRIBE_MASK (GOLDEN 150).
600 resource who=1 all +100000
602 resource who=2 all +100000
610 add companion who=1 66,76
612 add chariot who=2 66,86
614 add horse_archer who=1 70,76
616 add light_camel who=2 70,86
618 add light_horse who=1 74,76
620 add cataphract who=2 74,86
622 add phalanx who=1 78,76
624 add phalanx who=2 78,86
626 add javelineers who=1 82,76
628 add javelineers who=2 82,86
630 add archers who=1 86,76
632 add archers who=2 86,86
634 add catapult who=1 90,76
636 add catapult who=2 90,86
638 add scout who=1 94,76
640 add scout who=2 94,86
642 add general who=1 98,76
644 add general who=2 98,86
646 add spy who=1 102,76
648 add spy who=2 102,86
650 add the_despot who=1 106,76
652 add the_despot who=2 106,86
654 add militia who=1 110,76
656 add militia who=2 110,86
658 add kushite_archers who=0 78,81
660 add camel_archer who=0 82,81
662 add royal_balamob_slingers who=0 86,81
664 add barbarians who=0 90,81
666 add bushi who=0 94,81
668 add chevalier who=0 98,81
