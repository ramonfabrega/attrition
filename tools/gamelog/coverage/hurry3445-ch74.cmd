# Item 3445: chapter seventy-four's lobby (docs/GOLDEN.md §91, run864's lines) at cover=1 on the click-free pool,
# its 701 `finish` replaced by `hurry`. Take 1 (run1063-take1) kept `finish` and did not enter
# `Console::hurry_action@007c3810`: `finish` is `run_cmd` case 0x4f (the job counter +500000), and only
# `hurry` is case 0x51, the call to `hurry_action` (`run_cmd+0x52b8`; its other caller is `exec_keymap`).
# The first unshared input is 701 (run864's `finish`); `samegame.py --from 701`.
0 !ai off
600 age who=0 8
601 resource who=0 all +50000
640 @queueup 0 572 1 2005
700 select 2005 who=0
701 hurry
