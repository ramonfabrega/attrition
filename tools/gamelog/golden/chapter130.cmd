# Golden record, chapter one hundred and thirty -- group orders over a member that is busy, and a mandatory attack's
# kill-only tail (item 3988, GOLDEN 181; parked 4002a). Chapter one hundred and twenty-eight's lobby (the golden start,
# map 14). Everything staged and ordered is who=0's (the human) but the Bribe's target and the attack's two Catapults.
#   Arm 1 (closed by reading, not staged): `GroupData::get_loc@0070e030`'s off-map leader. Every simulation caller
#          gates it away -- `Group::action_move_near` on `GroupData::is_on_map`, `Group::action_guard`'s scratch group
#          on the members on the map, `Group::action_patrol` skips an off-map land member -- so no command reaches it.
#   Arm 2: `UnitData::is_busy@0060a370` in `Group::action_patrol@007030c0`'s member walk. The spell half is
#          `TypeData::is_pack`/`is_unpack` (spell type vslots +0x50/+0x54), and the entering/exiting half is a head
#          `SpecialAnimOrder` of type 0 or 1, which ends on its first call and is in no dump on disk. So Spy `0/6`,
#          mid-Bribe on the who=1 Hoplite beside it, is NOT busy: a patrol over it alone replaces its cast at
#          QUEUE_NEW, and the Bribe never lands. Alone, because in a group the leader's `do_patrol` runs a
#          QUEUE_FIRST move whose `action_halt` clears every member, cast or not, the same frame (walked).
#   Arm 3: `Unit::fight@005fd4d0:332`-`:335`'s tail. Hoplites `0/7` attack the who=1 Catapult `1/9` from 25
#          half-tiles off with a move queued behind (`@moveadd`); Hoplites `0/10`, beside it, kill it first. The
#          Catapult is no squad's follower, so `get_captain` answers itself and there is no substitute; the next action
#          is the MOVE, so `0/7`'s squad kills the attack and walks, with no search -- though the who=1 Catapult
#          `1/10` stands beside the corpse, where the search would take it.
# Killers, before the take: (1) the Bribed Hoplite turns who=0, or `0/6` holds a CASTORDER past 1114 -- the Spy was
# busy; (2) `0/6` takes no PATROLORDER on 1115 -- the command missed; (3) any of `0/7`'s squad holds an attack on
# `1/10` after `1/9` dies -- the tail searched; (4) `1/9` dies to `0/7`'s squad, or not at all -- the staging missed.
0 !ai off
600 library who=0 2
602 library who=1 2
606 add spy who=0 100,100
1104 add hoplite who=1 104,100
1108 @spell 0 629 6 1 19968 19200 6
1114 @patrol 0 11520 24960 6
1190 add hoplite who=0 112,124
1192 add hoplite who=0 135,124
1194 add catapult who=1 137,124
1196 add catapult who=1 141,128
1200 @attack 0 9 1 7
1200 @attack 0 9 1 10
1201 @moveadd 0 21504 26880 7
