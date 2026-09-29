# Golden record, chapter thirty-eight — the air line under fire: two Bombers
# striking a Barracks past a Radar Air Defense, an Anti-Aircraft Battery and
# an Infantry squad, each flak round's hit roll by whether its Bomber flies
# low, and a Bomber shot down.
#
# docs/GOLDEN.md §47 (item 1102; docs/DECISIONS.md 41 §1 and §5, 49).
# A cast of its own on the golden start (1094), NOT chapter seventeen
# appended: the flight lines are seventeen's, proven by run223, and nothing
# else of that cast is wanted.
#
#     0 `!ai off`                                  every chapter's
#   600 `library who=0 6`                          the Modern Age
#   606 `add airbase who=0 60,72`                  0/2007, (11616, 13920)
#   610 `add bomber who=0 52,84`                   0/6
#   612 `add bomber who=0 68,84`                   0/7
#   616 `add barracks who=1 110,86`                T 1/2006, (21120, 16512)
#   618 `add radar_air_defense who=1 116,86`       R 1/2007, (22272, 16512):
#                                                  ANTI_AIR (`Z6`), a
#                                                  building, FLY_HIGH 33 and
#                                                  FLY_LOW 75, range 10
#   620 `add infantry who=1 104,86`                I 1/6..1/8: not ANTI_AIR,
#                                                  FLY_LOW 33 — the searcher
#                                                  whose ladder asks
#                                                  `is_flying_high`
#   622 `add anti-aircraft_battery who=1 120,80`   A 1/9: ANTI_AIR (`6VT`), a
#                                                  unit, 50 and 90, range 17
#   640 `@flight 0 2007 0 6 7`                     the pair home, a strafe
#                                                  with `returning 1`
#   644 `@strike 0 2006 1 6 7`                     the pair, flying, at T: an
#                                                  AIRPATROLORDER over its
#                                                  point (run223), then the
#                                                  strafe once T is seen
#
#   run404 (item 1102):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch38 \
#       --map 14 --end-frame 1762 --log-window 605 1762 --timeout 5400 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5,DEATHS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter38.cmd
#   (cover=0 on the click-free lane, 1011's split; `Ammo::init_crash`,
#   `UnitData::is_flying_low` and `is_flying_high` are NEVER rows, so run405
#   is the same script at cover=1 on the queue lane.)
#
# The window is 1,157 blocks: the last event the walk schedules is 0/6
# inside 0/2007 on 1512 had it lived (walk B, without the Battery); 1762
# leaves 250 blocks past it. The walk with the whole cast shoots 0/7 down on
# 921 and 0/6 on 1224.
#
# ---------------------------------------------------------------------------
# THE STAGING, walked by this crate on run397's start (RON_STAGE, stage_walk,
# RON_STAGE_ALL, RON_STAGE_MAP), trace frames:
# - 606..622: the cast as above; T and R in no one's land (cells (27, 21)
#   and (29, 21); who=1's land starts at cell x 47).
# - 622: A takes an ATTACKTO to (20136, 16440), I's spot (AI-driven, 0x40000).
# - 641: 0/6, 0/7 one STRAFEORDER each, home; 645: each an AIRPATROLORDER
#   over (21120, 16512).
# - 741: A's first ATTACKORDER on 0/7; 751: 0/7 struck (300 -> 287).
# - 745, 746: 0/7 and 0/6 push a strafe on T (the patrol's search).
# - 764: I and A an ATTACKTO to (38664, 13320)..(38904, 13560), the army's.
# - 787..: R's rounds on 0/7; 818: 0/7 re-targets R (`Building(15)`).
# - 921: 0/7 dead; 1224: 0/6 dead; T standing to 1800.
# Unbuilt, this crate spends no draw on a flak round and none on a crash:
# every round at a Bomber hits, and a Bomber at 0 simply dies.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire (§47 has the readings).
# The walk prints the trace frame; its block is the next (§17, §45).
#
# 1. The issues reach the pump: process_flight on 641 and 645.
# 2. The flight (blocks 642, 646): one STRAFEORDER a Bomber, returning 1, on
#    642; one AIRPATROLORDER over (21120, 16512) on 646 (run223's shape).
# 3. Each ANTI_AIR round at a Bomber (R's and A's), from the first (~741):
#    exactly one draw, at Ammo::init+0x432 when the Bomber is within 0x900
#    of its front order's target, at +0x463 otherwise; none at +0x49f..+0x548.
# 4. Each such round's miss: AMMO `flags & 0x10` set iff r % 100 >= the
#    shooter's FLY_LOW (low) or FLY_HIGH (high): A 90/50, R 75/33.
# 5. A missed round does not wound its Bomber (its landing block).
# 6. I at a Bomber: never while the Bomber is high; a round of I's spends
#    two draws at +0x49f and +0x4dc, or one at +0x49f (10% the Bomber's own
#    FLY_LOW first). May not fire: I leave on 764 in the walk.
# 7. A Bomber shot down (the frame its guy dies): one draw at
#    Ammo::init_crash+0x305; a new AMMO record with `rolling` (r % 7) - 3,
#    `num_guys` 1, `who`/`o` the Bomber's, `whom`/`ox` -1, `flags` & 2 set
#    and & 0x1c clear, `sz` its altitude; no death object (kill_guy's plane
#    arm, 659473 -> 659613, never reaches add_death).
# 8. None of it on 0/6 or 0/7 inside 0/2007.

0 !ai off
600 library who=0 6
606 add airbase who=0 60,72
610 add bomber who=0 52,84
612 add bomber who=0 68,84
616 add barracks who=1 110,86
618 add radar_air_defense who=1 116,86
620 add infantry who=1 104,86
622 add anti-aircraft_battery who=1 120,80
640 @flight 0 2007 0 6 7
644 @strike 0 2006 1 6 7
