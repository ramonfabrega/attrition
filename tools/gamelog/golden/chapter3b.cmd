# Golden record, chapter three restaged — the siege line's dead zone, and
# the mounted line's speed, each in its own arena (run146, item 587).
#
# docs/GOLDEN.md §7, second capture. chapter3.cmd (run145) staged §7 as
# written, and two of its three falsifiers could not fire: the catapult is
# born PACKED and spent the fight unpacking, and the chariots shot the
# hoplites dead without moving. This file stages the same three unit types
# so that both can fire. It changes nothing in §7's design, only the timing
# and the ground (the commander's ruling, item 587).
#
# Two facts it is built on, both read before the run:
#
# - **The unpack is idle-driven, not target-driven.** `Unit::think_attack
#   @005f5a80`: a packing type holding `unit_masks & 0x80000` casts
#   `0x28c` (the unpack, printed `spell 652`) once its `idle` reaches 7 for
#   a human (21 for a computer), before any target search. run145's
#   catapult reached `idle 7` 75 frames after its birth (621 → 696) and
#   was unpacked 80 frames later (776). So a catapult born at 605 is ready
#   by about 761, with nothing in view.
# - **A ranged unit chases a target past its reach.** Chapter two's
#   slingers (reach 6) walked on their birth frame to one 8.6 tiles away.
#   The Chariot reaches 8 and sees 9 (`LOS` in tiles, docs/VISION.md), so
#   its enemy is born 10 tiles off.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch3b \
#       --map 14 --end-frame 1000 --log-window 605 1000 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=2,AMMO=5,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter3b.cmd
#
# The channel's rules (docs/ORACLE.md, "The cheat channel") are
# chapter3.cmd's: `!` for the console-only half, file order, and an `x,y`
# on every placement. Coordinates are TILES (docs/INPUT.md §11.3).
#
# **The ground, read off run145's own `[Start Game]` WORLD block** (item
# 587): arena A is chapter two's, world cell (1,10), tiles 4–15 × 40–43.
# Arena B is cells (0..3, 17), tiles 2–13 × 68–71, unowned, unblocked
# BASELAND in land region 1. That is 24 tiles south of arena A, twice the
# 12-tile respond floor and past every LOS here.
#
# ---------------------------------------------------------------------------
# THE PREDICTION, written before run146 so the capture can fail (item 587).
# Every one is a grep or a count; a line staged at frame F lands on block
# F + 1. Object numbers follow the order of the `add`s: player 0's next
# `o` is 6.
#
# check: nine `INFO cmd` records each returning 1 — the seven here plus
#        `37 !ffwd N` and `1000 !quit`.
# check: `MAP_STYLE 14`, seed 12345; `BEGIN FRAME` 1, then 605..999 with
#        no gap, then 1001: 397 blocks.
# check: births — the catapult `0/6` on 606, one unit with three type-265
#        figures; three separate Chariots `0/7..0/9` on 611, each
#        `o_up`/`o_down` −1 with two type-195 figures; arena B's hoplite
#        squad `1/6..1/8` on 616; arena A's `1/9..1/11` on 771.
# check: **the catapult unpacks with nothing in view**: `unit_masks
#        524288` from 606, a `CASTORDER` with `spell 652` on or near block
#        681 at `idle 7`, and `unit_masks 0` by about 761 — all before
#        arena A's hoplites exist. No `AMMO` from `0/6` before that.
# check: **the catapult fires** at arena A's hoplites while they stand 3
#        to 15 tiles off: an `AMMO` with `who 0 o 6 cur_time 1` after 771.
# check: arena A's hoplites close to melee on the catapult (a centre
#        distance under 570) while it lives.
# check: arena B — the chariots take their attack orders near 633–635 as
#        in run145, and at least one of `0/7..0/9` changes position on
#        three or more consecutive blocks before it first fires.
#
# What would falsify the chapter (docs/GOLDEN.md §7), read FIRST. This
# file predicts that NONE of the three fires, and that all three CAN:
#
# - **The catapult firing at a target inside three tiles**: an `AMMO` with
#   `who 0 o 6 cur_time 1` whose target's centre is under 570 units
#   (3 × 192 − 6) from the catapult's on the launch block, less the big
#   radii; under 480 units it fires outright, and 480..570 is read against
#   `attack_dist`. Reachable only if the hoplites close inside 570 while
#   the catapult lives; if they do not, the report says untested again.
# - **The chariots' per-block displacement equals the hoplites'**: the
#   largest one-block step of a walking chariot against arena B's
#   hoplites'. MOVES 30 against 25; run112 and run145 walked MOVES 25 at
#   28.0–28.4 units a block and MOVES 28 at ~30.6, so a chariot is
#   predicted near 32. Reachable only if a chariot walks.
# - **A leading count of 3 producing nine units or one** (or none): three
#   type-195 units on 611, as in run145.
# ---------------------------------------------------------------------------

# `ai` is console-only (table index 12).
0 !ai off

# The Classical age for both, as chapter3.cmd: `age` counts from one.
600 age who=0 2
602 age who=1 2

# Arena A's catapult, born first and alone, so it is unpacked before
# anything reaches it.
605 add catapult who=0 4,41

# Arena B: the chariots, and ten tiles east of them a hoplite squad for
# them to close on.
610 add 3 chariot who=0 2,68
615 add hoplite who=1 12,68

# Arena A's hoplites, eight tiles from the unpacked catapult: through its
# 3-15 band and into its dead zone.
770 add hoplite who=1 12,41
