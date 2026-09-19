# Golden record, chapter two — the ranged line, and the ammunition.
#
# docs/GOLDEN.md §6. The premise: a ranged squad and a melee squad on neutral
# ground eight tiles apart, so the shooting half of `Unit::fight` runs before
# anything touches. Chapter one's squads are born in contact and never shoot.
# Staged with
#   tools/explore/golden_capture.sh OUT --map 14 --end-frame 900 \
#       --cmd-file tools/gamelog/golden/chapter2.cmd \
#       --log-window 605 900 \
#       --detail end:UNITS=3,GUYS=2,AMMO=5,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1
# through `tools/gamelog/viadriver.sh`, which is what gives Wine a window.
#
# The channel's rules this file obeys (docs/ORACLE.md, "The cheat channel"):
# a `!` line reaches the console-only half of `run_cmd`'s two switches and a
# bare line only the chat half; lines run in file order and a frame below its
# predecessor is clamped; and `no_mouse = 1`, so an `add` without an `x,y`
# silently reuses a stale cursor — every placement here carries one.
#
# Coordinates are TILES (docs/INPUT.md §11.3), four to a world cell. The arena
# is world cell (1,10) and its neighbours — unowned BASELAND, region 1,
# `blocked 0`, read off run105's own `[Start Game]` WORLD block — which is
# also where chapter one's squads stand, so the two chapters are comparable.
#
# ---------------------------------------------------------------------------
# THE PREDICTION, written before run112 so the capture can fail. A golden
# chapter takes no `captures.txt` stanza — run101-run105 have none, because
# this lane is `viadriver.sh golden_capture.sh` and not `runqueue.sh` — so
# the stanza's `check:` lines live here instead.
#
# check: eight `INFO cmd` records, every one returning 1 — the six staged
#        here plus `37 !ffwd 1` and `900 !quit`, which the capture adds.
#        `python3 tools/gamelog/cmdsran.py <trace>`
# check: `MAP_STYLE 14` and seed 12345 read back from the dump's GAME INFO.
# check: the window's block count asserted against the window asked for —
#        `--log-window 605 900`, so 605..900 with no gap (parked 373: a
#        truncated run reads as completely ordinary).
# check: nine units born — `add bowmen` and `add hoplite` and `add slinger`
#        are three figures each (docs/INPUT.md §11.5), so UNITDATA carries
#        0/6..0/8 from 611, 1/6..1/8 from 616 and 0/9..0/11 from 621.
#
# What would falsify the chapter, and it is the point of running it
# (docs/GOLDEN.md §6):
#
# - **No `AMMO` block on any frame after 610.** The ranged arm was never
#   entered and the bowmen closed to contact like hoplites. `AMMO` is off in
#   every capture on this disk but run17 and run109, and it is the record
#   this chapter exists for.
# - **An `AMMO` block from the Slingers beyond six tiles, or the Bowmen
#   beyond ten** — the `RANGE` column is not the reach the fight tests.
# - **An `ATTACKORDER` on the Bowmen at 611**, before the hoplites exist:
#   auto-engage is not choosing its target the way chapter one's melee
#   squads do.
# ---------------------------------------------------------------------------

# `ai` is console-only (table index 12). Every chapter's premise.
0 !ai off

# The Gunpowder age for both, so the dump carries the late-age fields and the
# ranged stats are the upgraded ones. `library` moves the epochs too, which
# `age` alone does not (docs/RUNS.md run101-run105).
600 library who=0 3
602 library who=1 3

# Bowmen reach ten tiles and carry ammunition; Hoplites reach zero. Eight
# tiles apart, the bowmen shoot while the hoplites close.
610 add bowmen who=0 4,40
615 add hoplite who=1 12,40

# A second ranged profile at six tiles' reach, in the same fight, so a frame
# that parts can be attributed to a reach rather than to `fight` at large.
620 add slinger who=0 4,43
