# Item 3455: the coverage pair's lobby (East Indies, human Nubians against the Persians, All Technologies, Toughest;
# run651/run652's game) on the click-free pool, AI on, the computer's navy disbanded (docs/ARMY.md §6 step 1, §14).
# recon3455-1 (unfiled, `@dumpall` on 700..2100): the land army 0 is still mustering at 2100 (status 0x11), and the
# navy, army 2 (group 72, six ships, all standard), leaves muster on its 2040 tick (status 0x11 -> 0x12, target 1/2000,
# muster cell (46, 58)). Army 2 ticks at frame = 248 (mod 256): 2296, 2552, 2808.
# `add` takes quarter cells (192 internal units). At 2050 human Battleships on the ships' 2100 positions and on
# their muster cell, so the navy's standard line dies; the next tick's `normalize` reads `num_standard < 1` and
# `Army::process` calls `send_here(city, MOVE_TO)` and `close`. `@dumpall` brackets each tick (the ARMY record).
# The first unshared input is 2050; `samegame.py --from 2050` against run711 (blocks 1954..2210).
2040 @dumpall
2050 add battleship who=0 209,166
2050 add battleship who=0 211,167
2050 add battleship who=0 204,165
2051 add battleship who=0 227,190
2051 add battleship who=0 226,192
2052 add battleship who=0 221,233
2052 add battleship who=0 211,236
2052 add battleship who=0 216,234
2053 add battleship who=0 186,234
2053 add battleship who=0 184,232
2053 add battleship who=0 188,236
2296 @dumpall
2297 @dumpall
2552 @dumpall
2553 @dumpall
2808 @dumpall
2809 @dumpall
