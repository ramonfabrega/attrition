# Item 3460: the coverage pair's lobby (East Indies, the Persians, All Technologies, DIFFICULTY 5) with
# `--profile TEAM_STYLE=0` (Diplomacy) on the click-free pool: a human's paid declaration of war.
# At TEAM_STYLE 0 the leaders start at peace (`diplos[1]` 1 both ways on 896, recon3460-1), so no `peace 1` line;
# they have not met (`treaties[1]` 0), and an unmet row carries no Declare button (3444), so `meet who=1` first.
# OPTION_DIPLO is bound to F18 (VK 129) by `--bindkey`; the Declare button's point is read off a paused window
# capture of recon3460-3 at 911 (1920x1080, no title bar): game (955,366), 3444's point to a pixel.
# `Options::do_declare_war(1)` passes :98's team-style test at 0, `LeaderData::afford_dow` (the stock is 100000
# up), `Game::war_allowed` (RUSH_RULES 0), and execs popup 0, which is modal: the Return goes on the click's own
# frame (`@key 13` and `@char 13`, 3445). `Popup::on_modal_end` case 0 -> `CommandManager::issue_declare` ->
# `CommandPackage::process_declare` (a `COMMANDMANAGER=1` line) -> `Leader::action_declare@006dab50(1, 0, 0, 0)`
# -> `Leader::pay_dow@006d2b10`, then `set_diplo` to war.
# If the popup is still up (the Return missed), Escape closes it a few seconds later so the take ends.
900 meet who=1
901 resource who=0 all +100000
905 @key 129
910 @click 955 366
910 @key 13
910 @char 13
910 @after 8000 key 27
910 @after 8000 char 27
