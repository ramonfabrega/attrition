# Item 4313, the reseated taunt's take (`docs/LEADERS.md` §9.3): run1575's console lines and chat
# (`diplomacy4206.cmd` through 905) with the whos moved, for a human seated at who 1. The lobby is run1575's
# (East Indies, three seats, the Information age held, TEAM_STYLE 0) with `--human-who 1`: `LAST_WHO0=1` seats the
# human at who 1, the French at who 0 and the Russians at who 2 (item 4299's recon4299-1). The console's `meet`,
# `ally` and `resource who=` name a leader `who`: `ConsoleWin::run_cmd@007d6a70` indexes `leaders.list` by
# `parse_who`'s answer and acts from `MiscAccess::console->who` (cases 0x2c and 0x2f). OPTION_CHAT_ALLIES is F14
# (VK 125), by `--bindkey`.
# 900..902: meet and ally the French (who 0 now) from the console's seat (who 1), and lift the human's stock as
# run1575 did.
900 meet who=0
901 resource who=1 all +100000
902 ally 0
# 905: a resource taunt to allies, "4" (NEED_FOOD): `CommandPackage::process_chat` -> `Leader::receive_taunt` on the
# French, who write the sender's `who` into `incoming_taunt_who`, due 60 frames on; `Leader::process_taunt` then
# gifts a third of its food.
905 @key 125
905 @key 52
905 @char 52
905 @key 13
905 @char 13
