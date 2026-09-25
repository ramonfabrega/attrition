# Golden record, chapter twelve — the follow line, an issuer the AI never uses.
#
# docs/GOLDEN.md §20 (item 714; docs/DECISIONS.md 41 §1 and §5, 49). Chapter
# nine's harness, a fourth verb: each `@follow` line below is a call, from
# rontrace.dll, of the original's own `CommandManager::issue_follow@00941e70`
# with an unmodified follow pick's arguments (`Options::picked_spot@00721c40`
# through `GroupOut::issue_follow@00708980`, QUEUE_NEW), and the turn pump
# processes the command it appends as it processes a click (tools/trace/
# tracer.c, `issue_line`). No AI class reaches `Group::action_follow`, so
# no dump on disk holds a FOLLOWORDER.
#
#   run204 (item 714):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch12 \
#       --map 14 --end-frame 1150 --log-window 605 1150 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=2,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter12.cmd
#
# ---------------------------------------------------------------------------
# THE ISSUER, as the emulator ran it before the pair (item 714, step 1).
#
# `issue_follow` under unicorn on command_oracle.py's fixture, widened to
# who=0's objects 6..10 as live captains, with `(group, ox, whom, QUEUE_NEW
# 2)`: each call appends **18 bytes** — the 5-byte `group` (num 1, who 0,
# the one object) and a 13-byte `follow`, type 0x1e, `[ox i32][whom i32]
# [queued i32]` — and writes the package's size and data and the selection
# caches `CommandPackage::last_who_sent` (00c06354), `last_num_sent`
# (00cae5fb), `last_objects_sent` and `last_uids_sent` (through the
# imported memcpy). Nothing else: no unit, no order, no draw. The same
# selection again appends the 3-byte reuse and the follow (16); `queued`
# rides through as passed (QUEUE_FIRST 0, QUEUE_LAST 1); two captains make
# a 7-byte group. `use_mp_playback`, `semaphore & 0x10` and `semaphore & 4`
# each append nothing. **What the emulator cannot reach**: `CommandPackage::
# process_follow@009479c0` -> `Group::action_follow@006fd510` ->
# `Unit::add_follow_order@005e3f60`, and each frame's `Unit::do_follow@
# 005e65d0`.
#
# ---------------------------------------------------------------------------
# THE READING (docs/GOLDEN.md §20; docs/ORDERS.md §7.6, amended from the
# listing).
#
# `action_follow` gives every member that is `is_valid_unit`, `is_on_map`,
# not `is_plane`, and not the leader's own captain one FOLLOW (type 11) at
# QUEUE_NEW with the action bit: `ox/whom/uid` the leader. Each frame
# `do_follow` (with the order at the head) measures `d = vector_dist` from
# the follower to the leader and a standoff from the follower's `los`
# (`UnitData::los@006100c0`, the unit's `+0x3c`):
#     k = los * 0x60        when the follower is faster than the leader,
#         los * 0x300 / 5   otherwise (`UnitData::speed@0060aae0`),
#     k *= 2                while the leader `is_moving`,
#     s = clamp(los * 0x180 - k, 0x180, 0x600).
# `d <= s + 0xc0`: `set_anim(CHAR_DEFAULT, 0, 1)` and nothing else. Farther:
# the point `s` from the leader toward the follower, `find_nearby_spot`
# (then the point `s` behind the leader, then a ring `s..s+0xc0` round it,
# then the leader's own place), and a MOVE_TO leg at QUEUE_FIRST without
# the action bit, facing the leader's heading, and `do_move` the same frame.
#
# The cast's numbers (run190's dump: chariot `mylos` 9 and ~29 a frame,
# supply wagon 4 and ~25, hoplite 6 and ~23):
#   pair A, the Chariot 0/6 after the Supply Wagon 0/7: faster, so
#     s = 0x600 standing or walking (clamped), and it trails at 1536 and
#     moves when d > 1728;
#   pair B, the three Hoplites 0/8..0/10 after the Chariot 0/11: slower, so
#     s = 1383 (threshold 1575) while the chariot stands and s = 461
#     (threshold 653) while it walks — the doubling.
#
# ---------------------------------------------------------------------------
# THE PREMISE AND ITS KILLER, read before the run (docs/GOLDEN.md §3, point 5).
#
# The premise: **a player's follow command gives each commanded unit one
# FollowOrder on its leader; the follower stands while within its standoff,
# trails a walking leader by MOVE_TO legs to the standoff point, comes to
# rest when the leader stops, and re-trails when the leader turns.**
#
# 1. `check_accept_issue` and `process_group`'s player test, as in chapter
#    nine, with the same writers; none runs in a Quick Battle.
# 2. **`action_follow`'s early returns**: `GroupData::buildings` (+0x49),
#    written only by `Group::add@00714350`; `ox` or `whom` negative; the
#    leader not `is_valid_unit`, not `is_on_map` or `is_plane`; per member,
#    the same three, and the member being the leader's `get_captain` with
#    `who == whom` (0/6 against 0/7, 0/8..0/10 against 0/11: none). There
#    is no `is_ally` test.
# 3. **`do_follow`'s kill**: the leader not `UnitData::is_seen@00607a60`
#    by the follower's player — which answers 1 for a unit of that player,
#    so it cannot fire here — or the leader neither valid-on-the-map nor
#    inside a container.
# 4. The loops: the member loop is bounded by `group.num` (+0xc, below 0x80
#    in `Group::add`), here 1 and 3; the scenario sweep before it runs only
#    under `ScenarioData::ignore_orders`, whose writers are all
#    `ScenarioFuncSet`'s. `do_follow` has no loop but `find_nearby_spot`'s.
# 5. The ground (run190's start WORLD): pair A on BASELAND column x 4,
#    y 9..17, then row y 17, x 1..4; pair B on column x 15, y 14..19, then
#    row y 19, x 15..19. Chapter nine's sand is x 7..11. The nearest goody
#    boxes are (1, 19) and (16, 21), two cells off each line.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire (a call on trace frame F
# is processed before tick F+1, so its order is first on block F+2).
#
# 1. **The issuer does not reach the pump.** Trace frames 620 and 640: an
#    `INFO 17` with a refusal; or no `COMMANDMANAGER` group and follow text
#    between blocks 621/622 (641/642); or no move text for 700, 720, 880,
#    960 between F+1/F+2.
# 2. **Not one FollowOrder a unit on its leader.** Block 622, `0/6`; block
#    642, `0/8`..`0/10`: no type-11 order, an `ox/whom` other than 7/0
#    (11/0), a `uid` other than the leader's, or the action bit clear.
# 3. **A follower within its standoff does not stand.** Blocks 622..701
#    (`0/6`) and 642..721 (the squad): a leg above the FOLLOW, or a move.
# 4. **A follower does not trail a walking leader.** `0/6`: no MOVEORDER
#    leg above its FOLLOW by block 720 (the wagon is past 1728 by ~712), or
#    a leg without its FOLLOW under it. The squad: no leg by block 727 —
#    with the doubling they come as soon as the chariot walks (d ~768 > 653);
#    without it not before d > 1575, ~750.
# 5. **A follower does not come to rest.** 60 blocks after the leader's
#    stack empties (pair B ~828, pair A ~897): a follower with a leg, or
#    farther than its standing threshold (1728, 1575) from its leader. Or,
#    on any block to 1149, a follower whose FOLLOW is gone.
# 6. **A follower does not re-trail after the turn.** No leg on `0/6` after
#    block 962, or on the squad after 882; or, on block 1149, `0/6` farther
#    than 1728 from the wagon or a hoplite farther than 1575 from 0/11.
#
# This crate cannot take the command yet: it has no follow order, and its
# harness skips both `@follow` lines. The first parting is the capture's.
#
# check: `cmdsran.py` shows seven `INFO cmd` returning 1 (`0 !ai off`, the
#        four cheats, `37 !ffwd`, `1150 !quit`); the six `@` lines are
#        `INFO 17` records.
# check: `MAP_STYLE 14`, seed 12345, blocks 1 and 605..1149 at least.
# ---------------------------------------------------------------------------

# `ai` is console-only (table index 12). Every chapter's premise.
0 !ai off

# The cast, Ancient, one lever each. Pair A: a Chariot (`0/6`) at chapter
# nine's seat, cell (4, 9), and its leader, a Supply Wagon (`0/7`), two
# cells south on cell (4, 11). Pair B: a Hoplite squad (captain `0/8` with
# `0/9`, `0/10`) on cell (15, 14), and its leader, a Chariot (`0/11`), one
# cell south on cell (15, 15).
610 add chariot who=0 16,36
612 add supply who=0 16,44
614 add hoplite who=0 60,56
616 add chariot who=0 60,60

# THE LEVERS: two player follows through the issuer, `@follow <who> <ox>
# <whom> <o>`. The chariot follows the wagon.
620 @follow 0 7 0 6
# The squad, by its captain, follows the chariot.
640 @follow 0 11 0 8

# The leaders walk, through chapter nine's issuer. The wagon south to cell
# (4, 17)'s centre; the chariot south to cell (15, 19)'s.
700 @move 0 3456 13440 7
720 @move 0 11904 14976 11

# And turn, each after it has stood: the chariot east to cell (19, 19), the
# wagon west to cell (1, 17).
880 @move 0 14976 14976 11
960 @move 0 1152 13440 7
