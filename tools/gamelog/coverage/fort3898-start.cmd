# Item 3898: the golden start (map 14, Great Lakes' default lobby) at cover=1 on the click-free pool, run1494's line
# with a who=1 Fort added and killed by console (`docs/AI.md` §387). `Forts::close_fort@0073ec90` has one caller,
# `Build::close@00628980`, which calls it for an alive building whose type answers the fort slot (+0xfc) and whose
# `+0x78` fort index is not -1; `Build::activate` sets that index from `Forts::init_fort`. `add fort` without `NEW`
# completes the building at once; `die o,who` runs `Object::die` -> `close`. A fort never changes hands: a city's
# capture `die(3)`s every building but an active military trainer, and a fort is excluded from that arm (CITIES §8).
#
# who=1's buildings on block 624 of run1499 are 2000..2006, so the Fort is expected at 1/2007; `die 2008,1` covers an
# AI site laid on 620 first. `@dumpall` on 621 shows the Fort and its `FORT` slot; on 623 its death.
#
# Predicted: `Forts::close_fort@0073ec90` entered on 622. Killer: no `HIT` for 0073ec90 while block 623 has no who=1
# Fort (the `DEATHS` record or its absence from `BUILDDATA`). The twin (run1130) is run1129's file at cover=0: every
# input before 620 is shared, so `samegame.py --from 620`.
600 @cursor 70 33
620 add fort who=1 54,25
621 @dumpall
622 die 2007,1
622 die 2008,1
623 @dumpall
