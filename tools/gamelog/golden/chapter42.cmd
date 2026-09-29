# Golden record, chapter forty-two — a ring of who=1's Barracks with one
# single-tile corner, and four walkers inside it: an armed enemy at war, an
# unarmed enemy, the same armed enemy at peace, and an armed walker of the
# ring's own side. `Unit::resolve_block@005fccc0`, which `do_move` asks
# where `PathFinder::astar_path` flags a building tile on the tile grid.
#
# docs/GOLDEN.md §51 (item 1209; docs/DECISIONS.md 41 §1 and §5, 49).
# A cast of its own on the golden start, as chapters thirty-eight to
# forty-one.
#
#     0 `!ai off`                                   every chapter's
#   606..612 `add barracks who=1` ×7                1/2006..1/2012: a ring of
#                                                   4×4 footprints round the
#                                                   pocket x 216..219, y
#                                                   108..111 (tiles). The one
#                                                   single-tile crossing is
#                                                   1/2007's corner (220,107)
#   614 `add knight who=0 219,108`                  N 0/6, in the pocket
#   620 `@move 0 42720 20064 6`                     N out to (222,104), seven
#                                                   tiles away: the tile grid
#                                                   always (the draw's two
#                                                   cells, 0x600, are eight)
#   700 `add scout who=0 217,109`                   S 0/7, unarmed
#   706 `@move 0 42720 20064 7`                     S out the same way
#   800 `peace 1`                                   who=0 at peace with who=1
#   810 `@move 0 42720 20064 6`                     N out again, at peace
#   900 `add knight who=1 217,110`                  K 1/6, the ring's own side
#   906 `be 1` / `@move 1 42720 20064 6`, 907 `be 0` K out the same way (the
#                                                   seat twice, docs/GOLDEN.md
#                                                   §50)
#
#   run460 (item 1209):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch42 \
#       --map 14 --end-frame 1160 --log-window 605 1160 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=5,AMMO=5,DEATHS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter42.cmd
#   (cover=0 on the click-free lane; run461 is the same script at cover=1
#   on the queue lane, for `Unit::resolve_block@005fccc0`. `LEADERS=5` at
#   the end is the lowest level chapter eight has shown to print
#   `agendas[scan]` every frame.)
#
# The window is 555 blocks: the last scheduled line is 907, and 1160
# leaves 250 blocks past it.
#
# ---------------------------------------------------------------------------
# THE READINGS (docs/GOLDEN.md §51 has them whole).
#
# - `UnitData::invalid_loc@00607c30`'s building arm under `param_5`
#   (`607fb6`..`60800a`): a type with a base attack (`+0x1e8`) over a tile
#   `is_built_at` asks `find_any_building_at(t, who)`, and refuses (4) only
#   when the building is its own. The tile grid passes `param_5`
#   (`astar_path:812`, `valid_tcoord`); `calc_cost` prices such a tile
#   4000; `astar_path` flags its node, and the waypoint carries `0x10`.
# - `Unit::resolve_block@005fccc0`, on the stack's top: the tile refused
#   plainly and passed under `param_5`, the building found; not at war →
#   `agendas[owner] |= 2`, 0; at war → `add_attack_order(it, QUEUE_FIRST,
#   0, 0)`, 1. The own arm (`build_masks |= 1`) is dead: the second test
#   refuses an own building.
# - `do_move` asks it three times: on a flagged waypoint taken, on a
#   flagged `TAKE`, and unflagged after a near plan whose line fails.
#
# ---------------------------------------------------------------------------
# THE STAGING, walked by this crate on run437's start (RON_STAGE,
# RON_STAGE_ALL), trace frames:
# - 621: N's first tile plan is `[goal, (42360, 20664) flags 0x10]`; the
#   TAKE asks resolve_block, and N holds `[ATTACK, MOVE]` on 1/2007.
# - 622: the collision ladder's `find_new_target` replaces both with an
#   attack of its own (`QUEUE_NEW`), again every 30 frames.
# - 706: S's move dies the frame it is taken; S stays in the pocket.
# - 800: N's attack goes at peace; 810: N walks to the corner and loops
#   there from 811, `agendas[1]` of who=0 at 2.
# - 906: K's move dies the frame it is taken; K stays in the pocket.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire. The walk prints the
# trace frame; its block is the next.
#
# 1. The armed enemy never attacks 1/2007 from its move: no ATTACKORDER on
#    0/6 whose target is 1/2007 on block 622.
# 2. The armed enemy crosses without attacking: 0/6 outside the ring before
#    800 with no attack.
# 3. The unarmed enemy crosses: 0/7 outside the ring.
# 4. At peace no bit: `agendas[scan]` of who=0 at index 1 still 0 from 812.
# 5. The own side crosses: 1/6 outside the ring.

0 !ai off
606 add barracks who=1 215,106
607 add barracks who=1 219,106
608 add barracks who=1 222,110
609 add barracks who=1 218,114
610 add barracks who=1 222,114
611 add barracks who=1 214,110
612 add barracks who=1 214,114
614 add knight who=0 219,108
620 @move 0 42720 20064 6
700 add scout who=0 217,109
706 @move 0 42720 20064 7
800 peace 1
810 @move 0 42720 20064 6
900 add knight who=1 217,110
906 be 1
906 @move 1 42720 20064 6
907 be 0
