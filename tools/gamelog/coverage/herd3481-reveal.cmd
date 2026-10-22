# Item 3481: the herd's gate. run1090's lobby (Great Lakes, map 14, seed 12345, `--profile PLAYERS=2`, `ally 1`,
# `--ffwd-minute 6`) with `reveal 0` added. The packet run1100 (`RON_STATE_FRAME=5408`, `step4.py` on
# `ObjectsOut::sort_all@00657320`) admits herd 9 (objects 28-30 of who 8) past +0x20, +0x44, `is_active` and
# `is_on_map`. It then skips vtable +0x48 because `gamec->semaphore[0]` is 0x0a, and `& 0x42` is nonzero. Bit 1 is
# the lane profile's `StartReveal=1` (`Game::solo_checks@00588110`). `reveal 0` is `run_cmd` case 0x57 through
# `parse_binary`, which does `set(semaphore, 1, 0)`. With the bit poked clear on the packet,
# `AnimalData::is_seen@005d8160` is entered 12 times. `explore all` stays, so `is_seen`'s own `reveal_map` test
# still answers 1.
100 ally 1
5404 explore all
5405 reveal 0
5406 camera 186,190
5410 @dumpall
