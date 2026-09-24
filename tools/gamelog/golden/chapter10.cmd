# Golden record, chapter ten — the patrol line, an issuer the AI never uses.
#
# docs/GOLDEN.md §18 (item 693; docs/DECISIONS.md 41 §1 and §5, 49). Chapter
# nine's harness, a second verb: each `@patrol` line below is a call, from
# rontrace.dll, of the original's own `CommandManager::issue_patrol@00941800`
# with a patrol click's arguments (`WorldMap::on_right_up@008c7050:206`,
# QUEUE_NEW), and the turn pump processes the command it appends as it
# processes a click (tools/trace/tracer.c, `issue_line`). The cast is chapter
# nine's — a Chariot and a Hoplite squad, Ancient, `!ai off` — with the squad
# seated east of the sand. No capture of a patrol existed before this one:
# chapter nine's move orders were ones the AI already issues (parked 692),
# and no AI class calls `Group::action_patrol`.
#
#   run184 (item 693):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch10 \
#       --map 14 --end-frame 1250 --log-window 605 1250 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=2,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter10.cmd
#
# ---------------------------------------------------------------------------
# THE ISSUER, as the emulator ran it before the pair (item 693, step 1).
#
# `issue_patrol` under unicorn on command_oracle.py's fixture, widened to
# who=0's objects 6..10 as live captains, with these two calls' own arguments
# `(group, x, y, QUEUE_NEW 2)`: each call appends **15 bytes** — the 5-byte
# `group` (num 1, who 0, the one object) and a 10-byte `patrol`, type 0x0a,
# `[to_x i32][to_y i32][queued i8]` — and writes the package's size and data
# and the selection caches `CommandPackage::last_who_sent` (00c06354) and
# `last_num_sent` (00cae5fb) directly, `last_objects_sent`/`last_uids_sent`
# through the imported memcpy. Nothing else: no unit, no order, no draw. The
# same selection again appends the 3-byte `num 0` reuse and the patrol (13);
# `queued` rides through as passed (1 for a shift-click). `use_mp_playback`,
# `semaphore & 0x10` and `semaphore & 4` each append nothing, as for the move.
# **What the emulator cannot reach** is everything the chapter measures:
# `CommandPackage::process_patrol@00949380` → `Group::action_patrol@007030c0`
# → `Unit::add_patrol_order@005e4560` per member, and each leg's
# `Unit::do_patrol@005f1910`.
#
# ---------------------------------------------------------------------------
# THE READING (docs/GOLDEN.md §18 has it with its citations).
#
# `action_patrol` takes the group's location — `GroupData::get_loc@0070e030`,
# the leader's position when the group has no standing order point — as the
# patrol's first point and the click as its second, both snapped to the
# 48-unit grid (`div_3_table[v >> 4] * 0x30 + 0x18`), and gives **every**
# member a `GroupPatrolOrder` (type 22) through `add_patrol_order`: the two
# points, `waypoint 0`, the action bit, `id = (group.id + frame*10)*100 +
# order_num`, the leader's object and who, the member's index. There is no
# plain `PatrolOrder`: `add_patrol_order` asks for GROUP_PATROL alone, and a
# lone unit's group is pushed too (`process_group` forces `push_group`), so
# the chariot is the leader of a group of one.
#
# Each frame the head order is a patrol, `do_patrol` on the **leader**
# (`order.leader == o && order.who == who`) steps `waypoint` mod the point
# count and calls `Group::action_move_to(group, point, QUEUE_FIRST, ATTACK_TO)`
# — a patrol is a loop of attack-moves. A group's QUEUE_FIRST copies the
# leader's action-bit orders aside (`Group::set_up_insert@0070e520`), halts
# every member, issues the leg at QUEUE_NEW, and re-issues the copies at
# QUEUE_LAST (`Group::finish_insert@0070e620`), whose case 0x16 is
# `Group::redo_patrol_order@00706d90`: every member gets a fresh patrol with
# the leader's two points, id and waypoint. A **follower** whose head order is
# the patrol only idles (`set_anim(CHAR_DEFAULT)`).
#
# So after each turn every unit's stack is [the leg, the patrol]: for the
# chariot an `ATTACKTOORDER` (a group of one takes `add_move_facing_order`),
# for the squad three `GroupAttackToOrder`s under the leg's own id, one leader.
#
# ---------------------------------------------------------------------------
# THE PREMISE AND ITS KILLER, read before the run (docs/GOLDEN.md §3, point 5).
#
# The premise: **a player's patrol command, issued through the original's
# issuer, gives each commanded unit a GroupPatrolOrder between where its
# group stands and the click, and the leader walks it as alternating
# attack-moves, turning at each end, with its followers re-ordered at each
# turn.**
#
# 1. `CommandManager::check_accept_issue@00940a70` answering 0, as in
#    chapter nine; its writers are the same (`use_mp_playback`: the
#    constructor and `set_mp_playback@0093ee70`; `semaphore & 4`:
#    `Game::run_gamespy@00587060`). None runs in a Quick Battle.
# 2. `process_group`'s player test, as in chapter nine; the DLL issues only
#    for the console's player (refusal 1).
# 3. **`Group::action_patrol`'s early returns** — the one new killer:
#    `GroupData::buildings` (+0x49) set, whose one writer is `Group::add@
#    00714350` from the added object's `is_building`, and the DLL names only
#    unit captains; no leader (`find_leader < 0`), or `get_loc` answering 1;
#    a plane leader or any air member (`count(COUNT_DOMAIN, 4)`), which goes
#    to `action_air_patrol` instead; and per member, `UnitData::is_busy@
#    0060a370` — a head cast order or a unit entering or exiting — which
#    skips the member. None applies to a fresh chariot or hoplite squad.
# 4. The member loop is bounded by `group.num` (`+0xc`, at most 0x80 in
#    `Group::add`), here 1 and 3; `do_patrol`'s step is `waypoint + 1` against
#    `x_pos.length`, which `add_patrol_order` sets to exactly 2 and
#    `redo_patrol_order` extends only past 2, which nothing here makes.
# 5. The walks stay on BASELAND of region 1 and off every goody box's cell
#    (the nearest, (16, 21), is five cells off the squad's line); the
#    nearest animals, at cells (26, 12) and (27, 16), are twelve cells east
#    of the squad's line; who=1 is ~35 cells off and `!ai off`. The chariot
#    walks the column x 4, y 9..14, the squad x 14, y 10..15, the sand
#    strip of region 65 (x 7..11) between them.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire (logger block N is the
# state after tick N−1; a call on trace frame F is processed after block F+1
# is written and before tick F+1, so the patrol is first on block F+2, and a
# leader's first `do_patrol` runs on tick F+1 too).
#
# 1. **The issuer does not reach the pump.** Trace frame 620 or 640: an
#    `INFO 17` with a refusal; or no `COMMANDMANAGER` `process_group` and
#    `process_patrol` text between blocks 621/622 (641/642).
# 2. **The patrol is not one GroupPatrolOrder a unit between the group's
#    place and the click.** Block 622, `0/6`; block 642, `0/7`..`0/9`: no
#    `GroupPatrolOrder` (type 22), a `PATROLORDER` alone, points other than
#    the leader's snapped seat and the snapped click — (3480, 11160) for the
#    chariot, (11160, 11928) for the squad — or, for the squad, three that
#    do not share one id and leader `0/7`.
# 3. **The legs are not the leader's attack-moves.** Block 622 (642): no
#    `ATTACKTOORDER` on `0/6` above its patrol with `waypoint 1`, or anything
#    but three `GroupAttackToOrder`s under one id above the squad's patrols.
# 4. **The patrol does not turn.** The block the chariot's (the captain's)
#    leg empties within a tile (two cells) of its point, and no new leg to
#    the other point with `waypoint` stepped on the next block; or on block
#    1249 fewer than two turns for either. Predicted by speed alone
#    (run180's chariot ~29 and captain ~23 internal units a frame, legs of
#    ~4,100 and ~3,850): the chariot turns near 770, 915, 1060 and 1205, the
#    squad near 820, 1000 and 1180.
#
# check: `cmdsran.py` shows five `INFO cmd` returning 1 (the three cheats,
#        `37 !ffwd`, `1250 !quit`); the two `@` lines are `INFO 17` records.
# check: `MAP_STYLE 14`, seed 12345, blocks 1 and 605..1249 at least.
# ---------------------------------------------------------------------------

# `ai` is console-only (table index 12). Every chapter's premise.
0 !ai off

# The cast, Ancient, one lever each: a Chariot (`UBER_SIZE` 1, `0/6`) at
# chapter nine's seat, cell (4, 9); Hoplites (`UBER_SIZE` 3, captain `0/7`
# with `0/8`, `0/9` on its `o_down` chain) east of the sand, cell (14, 10).
610 add chariot who=0 16,36
612 add hoplite who=0 56,40

# THE LEVER: two player patrols through the issuer, internal coordinates.
# The chariot, from its seat to cell (4, 14)'s centre.
620 @patrol 0 3456 11136 6
# The squad, by its captain alone, from its seat to cell (14, 15)'s centre.
640 @patrol 0 11136 11904 7
