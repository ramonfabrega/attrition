# Item 3453 (c): the peace pair's lobby (run847's game) on the click-free pool, peace restored after the rush rules'
# five minutes end (4500, `Leader::set_diplo` sets war on both sides; run1072). `peace 1` sets `diplos` 1 both ways;
# then the computer's `Leader::diplomacy` (:1052-1061: not at war, `rush_rules` 10 past its minutes x 900) calls
# `Leader::action_declare@006dab50(who, 0, free 1, 0)`: no `afford_dow`, no `pay_dow`, `diplos` back to 0.
# The first unshared input is 4600; `rngcmp.py` against run847.
4600 peace 1
