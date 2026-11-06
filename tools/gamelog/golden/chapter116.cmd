# Golden record, chapter one hundred and sixteen -- an Entrench (item 3919, GOLDEN 166; AI 392.3, parked 3907a).
# A who=0 General `0/6` and a who=0 Hoplite squad `0/7..9` two tiles off, then the General's Entrench (637, 0x27d):
# `@spell 0 637 -1 -1 0 0 6`, chapter thirty-nine's untargeted press with the craft changed. `craftrules.xml`: FROM
# General, FLAGS lm, MANA 1000, JOB_TIME 120, COST 20g/20w. `SpellType::cast_entrench@00672820` runs on do_cast's last
# frame: `Objects::find_units(SEARCH_FRIENDLY, radius get_radius x 0xc0)`, and each found captain of a type with objmask
# 0x20, `attack` (+0x1e8) non-zero and `role & 0x10000`, inside the radius by `attack_dist`, idle, with no FILTER_COMBAT
# enemy inside 0x600, on a tile its owner or an ally owns (not Antipater), has every unit of its squad chain (+0x90)
# take `close_orders`, `unit_masks |= 0x2000000` and `+0x5c = +0x50`.
# Falsifiers, before the take (the journal has them): (1) no `process_spell 621` on the CM channel; (2) block 622: no
# CASTORDER (spell 637) on `0/6`, or its mana_burn not +1000; (3) `0/6`'s spell_time not climbing to 120 (the cast
# frame ~741); (4) the cover=1 twin: no HIT on `SpellType::cast_entrench@00672820`; (5) the body: the Hoplites'
# `unit_masks` without 0x2000000 after the cast frame.
#   run1513 (cover=0), run1513b (cover=1, `--cover cover=1 --no-ffwd --ai-tribe <run1513's>`):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch116 \
#       --map 14 --end-frame 1000 --log-window 605 1000 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter116.cmd
0 !ai off
612 add general who=0 40,180
614 add hoplites who=0 42,180
620 @spell 0 637 -1 -1 0 0 6
