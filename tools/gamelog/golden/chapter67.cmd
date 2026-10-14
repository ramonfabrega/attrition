# Golden record, chapter sixty-seven — a Stable's unit under Monarchy: the
# train time's arm the AI track built and its price's arm nobody did (item
# 2331, `docs/GOLDEN.md` §84; parked 2295, DECISIONS 56 §3).
#
# Three arms of one staging:
#
# 1. `ObjectData::train_time@006508c0:315`-`339`: a unit whose trainer
#    (`UnitTypeData +0x40`) is the Stable (`0x1ac`, by equality) of a player
#    holding `MONARCHY_2` takes `MONARCHY_UNIT_PROD2`, else `MONARCHY_1`
#    `MONARCHY_UNIT_PROD`; the time is `t x 100 / (x + 100)`. Built by item
#    2293 (`docs/AI.md` §217), held by its unit test, inert on every walk: no
#    capture finishes a Stable unit under Monarchy (parked 2295).
# 2. `TypeData::get_cost@00664090:657`-`665`, the train arm after
#    `MILITARY_UNIT_DISCOUNT`: the same trainer under `has_preq(MONARCHY_1)`
#    or `has_preq(MONARCHY_2)` takes `(100 - MONARCHY_UNIT_PROD) x cost /
#    100` — `_PROD2` is never read there (`docs/COSTS.md`, "Governments").
#    Built by this item.
# 3. The trainer by equality: a Barracks' Hoplites under Monarchy take
#    neither.
#
# Every Stable unit needs the Classical Age (`unitrules.xml`'s `PREQ0`), so
# `library who=0 1` first (chapter forty-nine's verb). Light Horse, TypeIndex
# 209 (unit 159), `JOB_TIME` 175, 60 food 40 timber; Monarchy's two bonus
# rows (`0x320`, `0x321`) both name Monarchy alone, and its King patriot is
# the Senate's tail, which this staging has none of.
#
# THE STAGING, walked by this crate on run492's start (RON_STAGE=ch45:
# chapter67:1150, RON_STAGE_ALL), trace frames:
# - 602: the Stable `0/2007` on tile (14, 74); 604 the Barracks `0/2008` on
#   (24, 74).
# - 610: Light Horse A queued, 60 food 40 timber (611). 612: Hoplites queued
#   at the Barracks, 50 food 30 timber.
# - 800: A out, `0/6`. 800: Monarchy gained.
# - 802: Light Horse B queued at **45 food 30 timber** (61/41 ramped, a
#   quarter off, truncated). 804: Hoplites queued behind the first at 51/31,
#   whole.
# - 839: the first Hoplites out (`0/7`-`0/9`). **977: B out, `0/10`** (0.8x
#   of the ramped time). 1074: the second Hoplites (`0/11`-`0/13`).
#
# CENSUS ROWS the staging reaches, never backed (`census.py --unbacked`):
# `ObjectData::train_time@006508c0` and `TypeData::get_cost@00664090`.
#
# WHAT WOULD FALSIFY IT, read before the run. A call on trace frame F is on
# block F+2.
# 1. The price arm: B's queue entry at 61/41 (or who=0's food down 61) on
#    block 804 — no cost arm; at 46/31 — the discount before the ramp.
# 2. The train arm: B born on any frame but 977 while A's 800 and the
#    Hoplites' 839 agree — the arm, or its place in the tail.
# 3. Equality: the second Hoplites at anything but 51/31, or born off 1074.
# 4. A patriot (`THEMONARCH`, TypeIndex 354) born to who=0: the King rides
#    the government's gain without a Senate.
#
#   run845 (item 2331), the click-free lane:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch67 \
#       --map 14 --end-frame 1330 --log-window 600 1330 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=5,DEATHS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter67.cmd
#
# The window: the last falsifier is the second Hoplites' birth, 1074;
# 1330 leaves 256 past it.
0 !ai off
600 library who=0 1
601 resource who=0 all +2000
602 add stable who=0 14,74
604 add barracks who=0 24,74
610 @queueup 0 209 1 2007
612 @queueup 0 132 1 2008
800 tech who=0 monarchy on
802 @queueup 0 209 1 2007
804 @queueup 0 132 1 2008
