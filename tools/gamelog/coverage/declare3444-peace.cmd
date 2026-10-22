# Item 3444: the peace pair's lobby (East Indies, the Lakota, RUSH_RULES 10; run847/run848's game) on the click-free
# pool, the human's declaration of war through the diplomacy window. The leaders meet on 1474 (run1016c), and only
# then does the window's row carry a Declare button (a paused window capture at 316 shows none, at 1484 one).
# OPTION_DIPLO is bound to F18 (VK 129) by `--bindkey`; the Declare button's point is read off the 1484 capture
# (window (954,398), game (954,367)): IFaceDiploNeg -> Options::do_declare_war(1).
# The first unshared input is 1480; `samegame.py --from 1480` against run848.
1480 @key 129
1484 @click 954 367
