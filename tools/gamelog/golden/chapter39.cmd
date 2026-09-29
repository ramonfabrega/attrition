# Golden record, chapter thirty-nine — the spell issuer's untargeted crafts:
# a Citizen's To Arms and a Militia's Civilian, both ways, and a General's
# Create Decoys beside two squads.
#
# docs/GOLDEN.md §48 (item 1111; docs/DECISIONS.md 41 §1 and §5, 49).
# A cast of its own on the golden start (1094). Every craft here is
# untargeted (`craftrules.xml` FLAGS without b/c/d), so each line is the
# UI's own button: `Options::do_spell@0071d7a0` calls `target_spell(type,
# -1, -1, 0, 0)` for a craft with `spell_flags & 0xe` clear, and the DLL's
# `@spell` passes the same.
#
#     0 `!ai off`                                  every chapter's
#   600 `library who=0 1`                          the Classical Age: To
#                                                  Arms' `has_preq(MILITIA)`
#   604 `tech who=0 militia`                       Militia's own tech bit:
#                                                  `has_tech` reads it, and
#                                                  neither the library nor
#                                                  the age sets it (run424)
#   606 `add citizen who=0 30,150`                 A 0/6, (6168, 29112)
#   608 `add citizen who=0 34,150`                 B 0/7, (6648, 28920)
#   610 `add militia who=0 38,150`                 C 0/8, (7416, 28920):
#                                                  `rare` 0, never converted
#   612 `add general who=0 40,180`                 G 0/9, (7800, 34680)
#   614 `add hoplites who=0 44,180`                H 0/10..0/12
#   616 `add slingers who=0 40,184`                S 0/13..0/15
#   620 `@spell 0 660 -1 -1 0 0 6`                 To Arms (0x294) on A
#   640 `@spell 0 660 -1 -1 0 0 7`                 To Arms on B, kept
#   660 `@spell 0 659 -1 -1 0 0 8`                 Civilian (0x293) on C:
#                                                  `rare` 0, so the Citizen
#   700 `@spell 0 659 -1 -1 0 0 6`                 Civilian on A: `rare` 50
#   720 `@spell 0 634 -1 -1 0 0 9`                 Create Decoys (0x27a) on G
#
#   run422 (item 1111):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch39 \
#       --map 14 --end-frame 1100 --log-window 605 1100 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter39.cmd
#   (cover=0 on the click-free lane; the three casts are NEVER rows, so
#   run423 is the same script at cover=1 on the queue lane.)
#
# The window is 495 blocks: the last event the reading schedules is the
# decoys on ~823 (Create Decoys' JOB_TIME 100 from the order on 722); 1100
# leaves 250 blocks past it. A decoy lives (general_upgrade + 2) ×
# DECOY_TIME / 2 = 2,500 frames, so none expires inside the window.
#
# ---------------------------------------------------------------------------
# UNDER THE EMULATOR FIRST (item 1111; scratch scripts on tools/emu/hooks.py
# and on step4.py's machinery, out of git).
# - cast_to_arms@00670880 and cast_civilian@006704a0 on a synthesized unit,
#   a damage sweep: frac = (damage << 8) / hits (idiv), then set_type, then
#   damage = new_hits × frac / 256 toward zero. A Citizen at 7 of 40 is a
#   Militia at 8 of 50; a Militia at 1 of 50 a Citizen at 0.
# - run424, a RON_STATE_FRAME=619 packet of this script without its @spell
#   lines (and, as taken, without the tech line):
#   * is_castable(To Arms, A) = 0: has_preq(MILITIA) 1 and has_tech 0 — the
#     Militia type's bit is clear. Leader::gain_tech(MILITIA) sets it (the
#     console `tech` verb's parse_type category reads "tubs" off the packet:
#     unit types included), and then is_castable = 3 on A and B.
#   * Civilian on C: `rare` 0 -> 50 (the PEASANTS arm), set_type(50, 0), one
#     Random::get(0, 0xffff) inside set_type, form 9.
#   * To Arms on A: `rare` 50, set_type(66, 0) with its draw.
#   * Create Decoys on G: the first candidate from the top of who=0's list
#     is S's captain 0/13; find_nearby_spot(G, radius 384) = (7800, 34296);
#     init_unit(0, Slingers, 7800, 34296). (The run then stops in a Steam
#     stats call; the capture has the rest.)
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire (§48 has the readings).
# A call on trace frame F is on block F+2 (§17).
#
# 1. The issues reach the pump: process_spell on 621, 641, 661, 701, 721.
# 2. The order: block 622, A holds one CASTORDER (spell 660, paid 1, ox −1),
#    and no MOVEORDER; its mana_burn unchanged (MANA 0).
# 3. The conversion: five do_cast frames (JOB_TIME 5); A a Militia (type 66)
#    with rare 50, form 0, hits 50, damage 0, and its stack empty — by 627.
# 4. C's Civilian: a Citizen (50) with rare 50 (not 0), form 9, by 667.
# 5. A's Civilian: a Citizen with rare still 50, form 9, by 707.
# 6. G's order (block 722): one CASTORDER (spell 634, paid 1); mana_burn
#    +1000; no bucket of who=0 down.
# 7. The decoys (~823): two new squads, Slingers then Hoplites (the list
#    walked down from the top), each figure unit_masks & 1 and mana_burn 0,
#    inside 1,728 of G; G's mana_burn back to 1000 − … (no refund).
# 8. A decoy ages: its mana_burn +1 a frame from its birth, while G's falls.

0 !ai off
600 library who=0 1
604 tech who=0 militia
606 add citizen who=0 30,150
608 add citizen who=0 34,150
610 add militia who=0 38,150
612 add general who=0 40,180
614 add hoplites who=0 44,180
616 add slingers who=0 40,184
620 @spell 0 660 -1 -1 0 0 6
640 @spell 0 660 -1 -1 0 0 7
660 @spell 0 659 -1 -1 0 0 8
700 @spell 0 659 -1 -1 0 0 6
720 @spell 0 634 -1 -1 0 0 9
