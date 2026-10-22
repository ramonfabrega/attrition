# Item 3470: a resource taunt to a computer ally. run1077's lobby (Great Lakes, map 14, seed 12345, `--profile
# PLAYERS=2`, `ally 1`), without the minute-six fast-forward: no pick needs the drawn scene, so the taunt is sent at
# 600. `resource who=1 all +100000` lifts the ally's food over `Leader::process_taunt@006b8cc0`'s 0x95 test (who=1's
# food read 19 on run1077's 5412). The taunt: OPTION_CHAT_ALLIES (F14, VK 125), "4" (`NEED_FOOD`, which
# `Taunts::check_for_order@009809d0` maps to 1), Return (`@key 13`/`@char 13`, 3445). Case 1 reads both leaders'
# `type_avail(FOOD)`, the `gift_stamp` (0x194) under 4500 frames, then `action_offer` and
# `Leader::action_respond@006d03c0`. On 620 "5" (`NEED_TIMBER`, case 2) to the same ally: the fallback if food is
# refused; otherwise the `gift_stamp` refusal 20 frames after the first gift, or a second gift if `teams_locked` skips it.
# `Leader::receive_taunt@006b8bf0` queues any taunt but HELP for `frame + 0x3c`, and `process_taunts` runs it on that
# frame exactly: the two land near 660 and 680, so the dumps bracket those (run1086's first take ended on 640, before).
100 ally 1
100 resource who=1 all +100000
600 @key 125
600 @key 52
600 @char 52
600 @key 13
600 @char 13
620 @key 125
620 @key 53
620 @char 53
620 @key 13
620 @char 13
658 @dumpall
661 @dumpall
670 @dumpall
678 @dumpall
681 @dumpall
690 @dumpall
