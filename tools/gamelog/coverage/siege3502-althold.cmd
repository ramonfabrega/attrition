# Item 3502: the golden start (map 14) at cover=1 on the click-free pool, the AI off: a catapult right-clicked onto
# an enemy Barracks with Alt held. `Console::execute_at_cursor@007c6630.c:328-401` reads ctrl, shift and alt through
# the exe's own `GetKeyState` import and, with alt alone held and a siege unit in the selection, issues
# `GroupOut::issue_siege_attack` on the `find_target(SEARCH_ENEMY)` hit; its command type 5 is
# `CommandPackage::process`'s one call of `process_siege_attack` (E2 of t51/draft-items-53.md).
# `@hold 18 616` answers VK_MENU's high byte on 610..616 (`tools/trace/hold.h`).
# Staging by precedent: `age` and `add catapult who=0 4,41` are chapter three's (chapter3.cmd); a golden-start take's
# first added unit is id 6 (minimap3444-ch9.cmd, click3437-ch11.cmd); `camera X,Y` then `@rclick 960 445` puts the
# cursor on tile (X, Y) (run1032's mapping, dock3455-rclick.cmd). The `@rclick` follows its `select` by four frames
# (3450: the card's options are built the frame after the select).
#
# The twin (run1137) is this file at cover=0: every input shared, so there is no `--from`.
0 !ai off
600 age who=0 2
602 age who=1 2
604 add catapult who=0 4,41
606 add barracks who=1 10,41
608 select 6 who=0
608 camera 10,41
610 @hold 18 616
612 @rclick 960 445
