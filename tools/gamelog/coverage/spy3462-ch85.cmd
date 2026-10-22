# Item 3462: chapter eighty-five's lobby (docs/GOLDEN.md §108, run880/run1054's recipe) at cover=1 on the click-free
# pool: the second Spy's Bribe through the key and the pick, and Pilfer and Restock Supplies asked by `@spell`.
# The key is bound in the lane's staged profile by `--bindkey OPTION_SPY_BRIBE=124` (F13, which no default binding
# uses). `keymap_options[0x179]` is (option 4, 629): `Options::exec` -> `do_spell(BRIBE)` arms the pick; Bribe's
# FLAGS `fcbhm` carry no `d`, so `+0x1c8 & 8` is clear and `Options::picked_spot`'s OPTION_SPELL arm calls
# `SpellTypeData::find_target@00674920` on the click.
#   1106        the second Spy 0/7 selected; the camera on the control's Hoplites 1/10..12 (tile 122.8,101.8 on run1054's
#               1110), set 2.6 tiles past them as run1080's centre pick landed short; a Supply Wagon 0/8 at the capital
#   1110        F13 -> KEYMAP 0x179 -> do_spell(629), the pick armed
#   1112        the click on the view's centre -> picked_spot -> find_target (run880's `@spell` for 0/7 is dropped)
#   1230, 1232  Pilfer (630) on who=1's capital 1/2000 by Spy 0/6, Restock Supplies (638) by the wagon 0/8: both crafts
#               are `FROM None`, so `is_castable`'s head answers 0 for every caster and `SpellType::cast` reaches neither
#               arm; asked to show the refusal on the trace
# The first unshared input is 1106; `samegame.py --from 1106` against run880 and run1054.
0 !ai off
600 library who=0 2
602 library who=1 2
606 add spy who=0 190,60
608 add spy who=0 120,100
1100 add hoplite who=1 196,57
1101 add scout who=1 198,56
1102 add hoplite who=1 122,102
1106 select 7 who=0
1106 camera 125,104
1106 add supply who=0 18,158
1110 @key 124
1112 @spell 0 629 6 1 37600 11300 6
1112 @click 960 445
1230 @spell 0 630 2000 1 42336 16224 6
1232 @spell 0 638 -1 -1 0 0 8
