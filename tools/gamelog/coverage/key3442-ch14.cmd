# Item 3442: chapter fourteen's lobby (docs/GOLDEN.md §22, run210's lines) on the click-free pool, its second
# formation command sent as keystrokes (`tools/trace/key.h`) in place of run210's `740 @form`. The keys are
# bound in the lane's staged profile by `--bindkey` (F13..F16 = VK 124..127, keys no default binding uses):
#   OPTION_ALL_MILITARY=124  FORM_E_RIGHT=125  OPTION_FORM=126  OPTION_RIGHT=127
# The first unshared input is 736 (run210 has none there); `samegame.py --from 736`.
0 !ai off
610 add hoplite who=0 12,84
612 add hoplite who=0 18,84
614 add slinger who=0 15,88
620 @form 0 2 0 6 9 12
700 @move 0 2976 12000 6 9 12
# F13: KEYMAP_OPTION_ALL_MILITARY (254) -> option 105, the army selected
736 @key 124
# F14: KEYMAP_FORM_E_RIGHT (174) -> option 0x13 object 3 -> Options::do_formation@007215b0(3), on the walk
740 @key 125
# F15: KEYMAP_OPTION_FORM (170) -> option 0x13 object -1 -> do_formation(-1); process_form -> action_form ->
# GroupData::get_form_option@0070beb0 (the next formation) the frame after
800 @key 126
# F16: KEYMAP_OPTION_RIGHT (168) -> option 0x16 -> Options::do_rotate@0071dfd0(+0xaaaaaaa), form -2
860 @key 127
