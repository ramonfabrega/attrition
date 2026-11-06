# Golden record, chapter one hundred and eleven -- a disband (item 3914, GOLDEN 161; AI 391.2 row 7, parked 3906c).
# Chapter nine's lines to 610 (GOLDEN 17, run180): a who=0 Chariot `0/6` (UBER_SIZE 1, the lone unit) born on 610 at
# tile 16,36, idle. On 620 it is selected and on 622 the key bound to OPTION_DISBAND (option 0xf, KeyMapType 147) pressed:
# `Options::exec@007188c0`'s 0xf arm calls `Options::do_disband@0071df20(0)`, which after `GroupOut::validate_disband@
# 0070a4a0` (0 for a unit group: `+0x49` clear) issues a `group` and a `disband` (0x15, all 0); the pump runs
# `CommandPackage::process_disband@00948660` -> `Group::action_disband@0070e260(0)` on 623: the last member, a unit
# (vslot 0x20 answers 0), goes to `Object::disband(o, 0)`: a unit's vslot 0xc answers 0, so no builders loop, no refund,
# no last_razed; not a plain building and on the map, so `Random::get(0, 0xffff)` and `die((r % 3) + 1, -1, 0)` --
# `Unit::close`'s death roll after it.
# Falsifiers, before the take: (1) trace 622: `INFO 22` with KEYMAP_NONE (the entry refused the selection); (2) no
# `process_disband all: 0 ... 623` on the CM channel; (3) `0/6` alive on block 624; (4) a who=0 stockpile that moves on
# 624 (a refund this reading says a unit never takes); (5) the cover=1 twin: no HIT on `Group::action_disband@0070e260`,
# `Options::do_disband@0071df20` or `CommandPackage::process_disband@00948660`.
#   run1508 (cover=0), run1508b (cover=1, `--cover cover=1 --no-ffwd`):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch111 \
#       --map 14 --end-frame 900 --log-window 605 900 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,DEATHS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --bindkey OPTION_DISBAND=124 \
#       --cmd-file tools/gamelog/golden/chapter111.cmd
# bindkey OPTION_DISBAND=124
0 !ai off
610 add chariot who=0 16,36
620 select 6 who=0
622 @key 124
