# Golden record, chapter one hundred and ten -- a spell stopped mid-hold (item 3913, GOLDEN 160; AI 391.2 row 6, parked 3906c).
# Chapter nineteen's lines (GOLDEN 27, run245): a who=0 Spy `0/6` casts the Informer (639) on a who=1 Barracks `1/2006`; it
# stands on its ring on 755 and holds the cast for forty frames, `spell_time` 1..39 on 756..794, the cast landing on 795.
# Mid-hold the Spy is selected and the key bound to OPTION_QUEUE pressed: `Options::exec@007188c0`'s 0xa6 arm, on a unit
# whose `num_queued` (+0xa0) is 0, calls `GroupOut::issue_stop_spell@00708a20` (a `group` and a one-byte `stop_spell`, 0x1d);
# the pump runs `CommandPackage::process_stop_spell@00947cc0` -> `Group::action_stop_spell@006fd7a0` on 771: each member whose
# action is a CASTORDER (type 0xe) has its orders cleared (`+0x68 &= ~0x4000000`, `+0xc0 = 0`, `close_orders(0)`,
# `clear_partial_path`, `update_action`) and `spell_time` (+0x98) set to 0.
# Falsifiers, before the take: (1) trace 770: `INFO 22` with KEYMAP_NONE (the entry refused the selection); (2) no
# `process_stop_spell 771` on the CM channel; (3) block 772: `0/6` still holding the CASTORDER, or `spell_time` not 0;
# (4) `1/2006`'s `infiltrated` taking 1 anywhere in the window -- the cast landed anyway; (5) the cover=1 twin: no HIT on
# `Group::action_stop_spell@006fd7a0` or `GroupOut::issue_stop_spell@00708a20`.
#   run1507 (cover=0), run1507b (cover=1, `--cover cover=1 --no-ffwd`):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch110 \
#       --map 14 --end-frame 900 --log-window 605 900 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --bindkey OPTION_QUEUE=124 \
#       --cmd-file tools/gamelog/golden/chapter110.cmd
# bindkey OPTION_QUEUE=124
0 !ai off
600 library who=0 2
602 library who=1 2
606 add barracks who=1 80,80
610 add spy who=0 60,82
620 @spell 0 639 2006 1 15360 15360 6
766 select 6 who=0
770 @key 124
