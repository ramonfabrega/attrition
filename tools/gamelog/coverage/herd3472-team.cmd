# Item 3472: a herd on screen. run1077's lobby (Great Lakes, map 14, seed 12345, `--profile PLAYERS=2`, `ally 1`,
# `--ffwd-minute 6` so the scene is drawn from 5400). Herd 9 (`HERD` `cx 46 cy 47`) is objects 28-30 of who 8 at tile
# (186, 190) on run1077's block 5412, every one `visible 0`. `camera 186,190` puts the herd at the view's centre: a
# drawn frame with an animal on the on-screen list calls vtable +0x48 `(console->who, 1)` from
# `ObjectsOut::sort_guys@00656b00`, which is `AnimalData::is_seen@005d8160` on an Animal. The click at the view's
# centre (960, 445: `camera X,Y` draws its tile there, run1032 and run1060) is the second route, through
# `Console::left_click@007c3bc0`.
# Sight: the first take's `5404 add hoplite who=0 184,190` returned 1 and placed nothing (run1090-take1: 316
# subobjects on 5408, the human's objects 0-5 only), and with the camera on the herd and the herd `visible 0`
# `is_seen` was not entered. `explore all` (`reveal_map` 3, item 959) gives the sight instead; `is_seen` is entered
# before its own `reveal_map` test.
100 ally 1
5404 explore all
5406 camera 186,190
5408 @dumpall
5410 @click 960 445
5411 @dumpall
5420 @dumpall
