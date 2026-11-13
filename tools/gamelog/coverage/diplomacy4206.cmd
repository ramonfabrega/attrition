# Item 4206, step VII-a of the leaders chain: the diplomacy leaves' take (`docs/LEADERS.md` §7 to §9).
# run1482's lobby (East Indies, three seats: the idle human Nubians, the French who=1 and the Russians who=2, the
# Information age held, Deathmatch) at `--profile TEAM_STYLE=0`, where every seat starts at peace. Three seats,
# because an alliance between the two seats of a 1v1 is the game's end: `Leader::set_diplo` calls victory once every
# live leader is mutually allied (`docs/LEADERS.md` §10.2; recon4206-1 closed on 903, the frame after `ally 1`).
# OPTION_DIPLO is F18 (VK 129) and OPTION_CHAT_ALLIES F14 (VK 125), by `--bindkey`. The window's points are read off
# recon4206-2 (the diplomacy window: Declare at (955,366), the contact button at (775,366)) and recon4206-3 (the
# negotiation window: the peace check (676,499), the alliance check (770,499), wealth's Tribute (858,405), PROPOSE
# (1206,564)), each a paused window-only photograph at 611, 1920x1080.
# 900..902: meet and ally the French by console (`meet` is `treaty_on`, `ally` is `set_diplo(0, 1, 2)` both ways),
# and lift the human's stock for `LeaderData::afford_dow`.
900 meet who=1
901 resource who=0 all +100000
902 ally 1
# 905: a resource taunt to allies, "4" (NEED_FOOD): `CommandPackage::process_chat` -> `Leader::receive_taunt` on the
# ally, due 60 frames on; `Leader::process_taunt` then gifts a third of its food (`gift_stamp`, `last_taunt`,
# `taunt_frame`).
905 @key 125
905 @key 52
905 @char 52
905 @key 13
905 @char 13
# 980..985: the diplomacy window, and the Declare on the ally, confirmed by Return on the click's own frame:
# `Leader::action_declare` turns an allied pair's request into peace, counts `dow` (block B) and stamps
# `broke_alliance`.
980 @key 129
985 @click 955 366
985 @key 13
985 @char 13
# 1000: the Declare again, now at peace: war, a second `dow` and a `blacken`.
1000 @click 955 366
1000 @key 13
1000 @char 13
# 1015..1026: the negotiation window; the peace check (`issue_treaty(1, 1)`) and PROPOSE with nothing given
# (`issue_accept`: the human's `action_respond(0, 1, 1)` commits and sets the French `agendas[0] |= 4`, so its next
# `Leader::diplomacy` pass answers). recon4206-5: the French accept it (`made_peace` 1027 both ways).
1015 @click 775 366
1020 @click 676 499
1026 @click 1206 564
# 1040..1050: an alliance asked for with nothing given (the alliance check, `issue_treaty(1, 2)`, then PROPOSE ten
# frames on; recon4206-4's PROPOSE one frame after its check issued nothing): a proposal the computer finds short
# (`counteroffer`).
1040 @click 770 499
1050 @click 1206 564
# 1080: close the windows.
1080 @key 27
1080 @char 27
1081 @key 27
1081 @char 27
