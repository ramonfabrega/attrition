# Item 3455, the retake (run1075b): the coverage pair's lobby at cover=1 on the click-free pool, AI on, the computer's
# navy disbanded by console (docs/ARMY.md §6 step 1, §14). run1075's eleven human Battleships (2050..2053) lost to it:
# army 2 grew 6 -> 9 -> 11 -> 15 ships over its 2296, 2552 and 2808 ticks, so `num_standard < 1` never held.
# Here the game is the twin's until 2284. run714's block 2283 (the twin's window 2283..2539) holds the navy's
# group 72 whole: who=1 ships 49, 51, 59, 61, 71, 75, 76, 84, 87. `die o,who` each on 2284; army 2's tick is
# frame = 248 (mod 256), so 2296's `normalize` should read `num_standard < 1`, and `Army::process` call
# `send_here(city, MOVE_TO)` and `close`. `@dumpall` brackets 2296 and the next tick, 2552 (the ARMY record).
# The first unshared input is 2284; `samegame.py --from 2284` against run714.
2284 die 49,1
2284 die 51,1
2284 die 59,1
2284 die 61,1
2284 die 71,1
2284 die 75,1
2284 die 76,1
2284 die 84,1
2284 die 87,1
2296 @dumpall
2297 @dumpall
2552 @dumpall
2553 @dumpall
