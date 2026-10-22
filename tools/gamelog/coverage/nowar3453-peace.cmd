# Item 3453: the peace pair's lobby (East Indies, human Nubians against the Lakota, RUSH_RULES 10; run847/run848's
# game) on the click-free pool, the human's soldiers walked into the Lakota's land before the peace ends (4500).
# `Unit::process_attrition` (:196-204): a unit of the console's leader in a non-enemy's land, `rush_rules` 10 and
# `war_allowed` 0, the leader's `attrition_stamp` (+0x1f4) 0 -> `Game::say_no_war@00592b00`, then the stamp is the frame.
# Cells off run846's block-2 WORLD record (60x60, row-major in y): the Lakota's land is cells (37..56, 42..58) around
# Maka Sica (39264,40032); cell (39,41) is open land on its north-west edge. `add` takes quarter cells.
#   1386  a who=0 Hoplite squad at quarter (158,166), cell (39,41): 0/14..16 (the human's units are 0/0..13)
#   1388  the squad walked to cell (44,46), the Lakota's land
# The first unshared input is 1386; `samegame.py --from 1386` against run848, `rngcmp.py` against run847.
1386 add hoplite who=0 158,166
1388 @move 0 34176 35712 14 15 16
