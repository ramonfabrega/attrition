# Item 3445: chapter seventy-four's lobby (run864's lines to 640) at cover=1 on the click-free pool, the human's
# city Napata `0/2000` renamed through its card: OPTION_RENAME_CITY is `BuildOut::make_options@00629ed0:159`'s
# `Options::add(OPTION_RENAME_CITY, 3, 0, …)`, the card's cell (3,0), (222, 935) on 3443's geometry.
# `Options::exec`'s 0x117 arm opens popup 6 (an edit box holding the name); `Popup::on_modal_end@007c13f0`
# case 6 sends `CommandManager::issue_rename_city`, and `CommandPackage::process_rename_city` calls
# `HotKeyGroups::find_group@00714d00` (GROUPS §38).
# A modal window stops `do_frame` in a solo game (`Game::loop@00591570`), so the letters and the Return
# go on the click's own frame: the popup is built inside the click. Each letter is its WM_KEYDOWN
# (`@key`, the VK) and its WM_CHAR (`@char`, the character), as Windows sends a typed letter. Return is
# the same pair: `Window::on_char@00a54ec0` calls `key_click_event(ch, latest_virtual)`, the VK the last
# key-down left, and only that path reaches `Popup::on_key_click@007c1960` (0xd: the default button);
# the key-down alone stops in `EditBox::on_key_click`, which keeps 0xd. Take 1 (run1064-take1) sent
# `@key 13` alone: the popup stayed up, the game held on 707 and the take timed out.
# `@dumpall` makes 705 (before) and 708 (after) whole dumps: `HOTKEYGROUPS` prints only from `dump_all`.
# The first unshared input is 700 (run864's `select 2005`); `samegame.py --from 700`.
0 !ai off
600 age who=0 8
601 resource who=0 all +50000
640 @queueup 0 572 1 2005
700 select 2000 who=0
705 @dumpall
706 @click 222 935
706 @key 75
706 @char 75
706 @key 69
706 @char 101
706 @key 82
706 @char 114
706 @key 77
706 @char 109
706 @key 65
706 @char 97
706 @key 13
706 @char 13
# If the popup is still up (the Return missed), Escape closes it a few seconds later so the take ends.
706 @after 8000 key 27
706 @after 8000 char 27
708 @dumpall
