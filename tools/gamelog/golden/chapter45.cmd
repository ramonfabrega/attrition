# Golden record, chapter forty-five — `get_cost`'s library-line tail on the
# golden start: a military epoch under Despotism, a civic epoch with Dye held
# by a human's Merchant, and a commerce epoch with both held.
# `TypeData::get_cost@00664090`'s switch on the tech's line at `00666b7d`
# (docs/AI.md §99.8; parked 1261).
#
# docs/GOLDEN.md §54 (item 1268; docs/DECISIONS.md 41 §1 and §5, 49, 56 §3).
# The booking's goods — Furs, Silk, Papyrus — are on no deposit of this map
# (§54 has why no staging reaches them); these are the tail's steps it can.
#
#     0 `!ai off`                                   every chapter's
#   600 `resource who=0 all +500`                   the budget
#   602 `add merchant who=0 38,161`                 M 0/6, three tiles west
#                                                   of the Dye at tile
#                                                   (41, 161), in who=0's
#                                                   land: a human's packed
#                                                   Merchant deploys itself
#                                                   (`Unit::think`'s
#                                                   rare-collector arm,
#                                                   docs/ORDERS.md §23.2)
#   604 `tech who=0 despotism on`                   `has_preq(0x31c)`, the
#                                                   Despotism step's first
#                                                   test, holds
#   800 `@queueup 0 572 1 2005`                     A: The Art of War, the
#                                                   military line, at the
#                                                   Library 0/2005
#   802 `@queueup 0 565 1 2005`                     B: City State, the civic
#                                                   line, Dye held
#   804 `@queueup 0 558 1 2005`                     C: Barter, the commerce
#                                                   line, both held
#
#   run492 (item 1268), the take:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch45 \
#       --map 14 --end-frame 1650 --log-window 600 1650 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=5,DEATHS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter45.cmd
#   (cover=0 on the click-free lane.)
#
# The window: the walk finishes Barter on 1403; 1650 leaves 247 past it.
#
# ---------------------------------------------------------------------------
# THE READINGS (docs/GOLDEN.md §54 has them whole).
#
# - The tail, for a library epoch (`0x227..0x242`): each step is `(100 − x)
#   × cost / 100`, truncated toward zero. Military: Furs (`has_rare(0x15)`),
#   then the first of `has_preq(0x31c)`, `(0x31b)`, `(0x31a)` —
#   `DESPOTISM_MILITARY_CHEAPER3`, `2`, `1` (`+0x99c`, `+0x998`, `+0x994`),
#   15 each — then the Turks. Civic: Dye (`has_rare(10)`, `+0x8e0`, 25),
#   then the Persians. Commerce: Silk (`has_rare(11)`), then the Dutch.
# - `has_preq@006db810` of a bonus row whose one prerequisite is a
#   government, not itself a bonus row, is `has_tech` of that government:
#   the three `DESPOTISM` rows all name Despotism.
# - `has_rare@006e0770` is `rare | rare_conquest`; `rare` is rebuilt from
#   `rare_owned` every recompute, and `rare_owned` is the goods an idle,
#   deployed fisherman or merchant stands on (`Unit::do_gather`).
#
# ---------------------------------------------------------------------------
# THE STAGING, walked by this crate on run484's start (RON_STAGE,
# RON_STAGE_ALL), trace frames:
# - 602: M takes `[MOVE_TO (7320, 30936), CAST 656]`; it walks to 613 and
#   casts from 614; 761 it stands on its corner (7296, 30912), deployed.
# - 767: who=0's `rare` 0x10, Dye (bit 10 − 6).
# - 800: the Library's entry at 102 food (120 less Despotism's 15 %).
# - 802: 90 food (120 less Dye's 25 %). 804: 60 food 60 timber, whole.
# - The Art of War finishes 1000, City State 1201, Barter 1403.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire. The walk prints the
# trace frame; its block is the next.
#
# 1. Despotism's step: the Library's first entry, or who=0's food, at 120
#    on block 801.
# 2. Dye's step: the second entry at 120 on block 803; or the Merchant
#    still packed on block 762, or `rare` 0 on block 768 (the deploy, not
#    the price).
# 3. The line: the third entry below 60/60 on block 805 (Dye's or
#    Despotism's step on the commerce line).

0 !ai off
600 resource who=0 all +500
602 add merchant who=0 38,161
604 tech who=0 despotism on
800 @queueup 0 572 1 2005
802 @queueup 0 565 1 2005
804 @queueup 0 558 1 2005
