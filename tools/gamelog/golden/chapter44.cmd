# Golden record, chapter forty-four — the idle search's head on a computer's
# fleet: a computer's `SIEGE` ship takes a land building across the regions
# and out of its range, and a human's does not.
# `Object::check_target@00649e00`'s head, `649f05`..`649f29`: the region
# test is dropped for a searcher with `unit_masks & 0x40000`,
# `has_objmask(0x40000)` and type `+0x218 == 1` (parked 1230).
#
# docs/GOLDEN.md §53 (item 1254; docs/DECISIONS.md 41 §1 and §5, 49, 56 §3).
# The middle lake (tiles 80..140, 36..82) and the south lake (141..206,
# 137..207) on the golden start, neither in anyone's territory.
#
#     0 `!ai off`                                   every chapter's
#   600 `add barracks who=0 110,99`                 K0 0/2007, the computer
#                                                   ship's mark, on the land
#                                                   south of the middle lake
#   601 `add barracks who=1 117,103`                V1 1/2006: who=1's eyes on
#                                                   K0 (the ship's LOS is 17
#                                                   tiles, and an unseen
#                                                   candidate is refused)
#   602 `add barracks who=1 176,226`                K1 1/2007, the human
#                                                   ship's mark, south of the
#                                                   south lake
#   603 `add barracks who=0 183,230`                V0 0/2008: who=0's eyes on
#                                                   K1
#   610 `add bomb_vessel who=1 110,73`              S1 1/6, the computer's
#                                                   Bomb Vessel: K0 at
#                                                   `attack_dist` 4200, out of
#                                                   its range (19 × 0xc0) and
#                                                   inside its search
#                                                   (`unit_respond_range ×
#                                                   0x180`, 4608)
#   612 `add bomb_vessel who=0 176,201`             S0 0/6, the human's: K1 at
#                                                   4008, out of range and
#                                                   inside its search
#                                                   ((19 + 1) × 0xc0 + 0x180,
#                                                   4224)
#
#   run484 (item 1254), the take:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch44 \
#       --map 14 --end-frame 1450 --log-window 605 1450 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=5,AMMO=5,DEATHS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter44.cmd
#   (cover=0 on the click-free lane.)
#
# The window: the walk has K0 dead on 1166 and S1 idle from 1180; 1450
# leaves 250 blocks past it.
#
# ---------------------------------------------------------------------------
# THE READINGS (docs/GOLDEN.md §53 has them whole).
#
# - `check_target`'s head, for a unit searcher and `param_7 == 0`: `other`
#   is `get_tregion` of the two tiles differing; it is cleared when the
#   searcher's `unit_masks & 0x40000` (every unit of a leader that is not a
#   plain human, `Unit::init@00612100:586`), `has_objmask(0x40000)` (vslot
#   `+0x148`, the `OBJ_MASK` letter `S`) and its type's domain is the sea.
#   Then `(defensive || other) && !is_in_range` refuses.
# - The `S` sea types: Siegeship (no tribe builds it), Bomb Vessel, Bomb
#   Ketch, Dreadnought, Battleship, Advanced Battleship, Catapult Ship.
#   The Bomb Vessel is `4-19rng`, `LOS 17`.
# - The idle search is `Unit::think`'s, `find_melee_target(-1)`: radius
#   `(r + 1) × 0xc0 + 0x180` for an aggressive ranged unit, raised to
#   `unit_respond_range × 0x180` under `unit_masks & 0x40000`.
# - `valid_target` refuses a candidate the searcher's player does not see,
#   above everything: hence V1 and V0.
#
# ---------------------------------------------------------------------------
# THE STAGING, walked by this crate on run466's start (RON_STAGE,
# RON_STAGE_ALL), trace frames:
# - 610: S1's first idle think takes K0 (`ATTACK`, `in_range 0`); it walks
#   south 19 frames to (21293, 14712) and strikes from 630, K0 at
#   `attack_dist` 3624.
# - 612 onwards: S0's think refuses K1 (4008, another region, a human's
#   ship) every time it looks; S0 never moves.
# - 671..1112: K0 loses 122 a hit; dead on 1166. S1 idle from 1180.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire. The walk prints the
# trace frame; its block is the next.
#
# 1. The exception: S1 with no `ATTACKORDER` on block 611 (the original
#    refuses K0 across the regions, as it did the Caravel's building on
#    East Indies 8907).
# 2. The computer's conjunct: S0 with an `ATTACKORDER` on K1 anywhere in
#    the window.
# 3. The vision: S1 taking nothing until V1's eyes are up is the walk's
#    first staging's answer (a later first think), not a refusal.

0 !ai off
600 add barracks who=0 110,99
601 add barracks who=1 117,103
602 add barracks who=1 176,226
603 add barracks who=0 183,230
610 add bomb_vessel who=1 110,73
612 add bomb_vessel who=0 176,201
