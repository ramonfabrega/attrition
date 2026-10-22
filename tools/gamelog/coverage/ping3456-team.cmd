# Item 3456: the team lobby on the click-free pool. Great Lakes (map 14), seed 12345, `--profile PLAYERS=2`: a human
# and two computers (every `LAST_SLOT1..7` is already a computer; `PLAYERS` is GameInfo option 3, 1 in every pinned
# lobby). `ally 1` allies the human and player 1 by console; player 2, live and allied to neither, keeps
# `Leader::set_diplo@006ec6a0`'s count non-zero, so the game runs on (GOLDEN's two-player allied victory does not fire).
# The help taunt: OPTION_CHAT_ALLIES (bound to F14, VK 125) opens the chat to allies; "11" is `help_here`, id `HELP`,
# which `Taunts::check_for_order@009809d0` maps to TAUNT_HELP (16); `LeaderOut::issue_chat` takes a number of under
# four characters as a taunt. Return is `@key 13` and `@char 13` (3445: the edit box keeps a bare key-down's 0xd).
# The ping: OPTION_PING (F15, VK 126), then a left click on the map: `Console::down` with OPTION_PING and
# `selecting_spot` → `Options::picked_spot` → `LeaderOut::issue_ping` → `process_ping` → `Leader::action_ping@006d18a0`
# on every active leader. The ally's army arm needs the taunt inside 225 frames (ARMY §26): the first ping is 12 frames
# after it, the second 292, with no taunt between. `!ffwd 6` (5400) leaves the scene drawn for the pick.
100 ally 1
5410 @key 125
5410 @key 49
5410 @char 49
5410 @key 49
5410 @char 49
5410 @key 13
5410 @char 13
5412 @dumpall
5420 @key 126
5422 @click 960 540
5423 @dumpall
5430 @dumpall
5700 @key 126
5702 @click 960 540
5703 @dumpall
